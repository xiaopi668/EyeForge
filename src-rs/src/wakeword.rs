use std::path::Path;
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustpotter::{AudioFmt, Rustpotter, RustpotterConfig};

use crate::config::Config;

struct WakewordHandle {
    stop: mpsc::Sender<()>,
    thread: JoinHandle<()>,
}

#[derive(Clone)]
struct WakewordConfig {
    model_paths: Vec<String>,
}

static WAKEWORD: OnceLock<Mutex<Option<WakewordHandle>>> = OnceLock::new();

fn state() -> &'static Mutex<Option<WakewordHandle>> {
    WAKEWORD.get_or_init(|| Mutex::new(None))
}

pub fn restart(config: &Config) -> Result<String, String> {
    stop();

    if !config.wakeword_enabled {
        return Ok("Wake word disabled".into());
    }

    let wake_config = build_config(config)?;
    let labels = wake_config
        .model_paths
        .iter()
        .map(|path| model_key(path))
        .collect::<Vec<_>>()
        .join(", ");

    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();

    let thread = thread::spawn(move || {
        let result = run_listener(wake_config, stop_rx, ready_tx);
        if let Err(error) = result {
            eprintln!("wake word listener stopped: {error}");
        }
    });

    match ready_rx.recv_timeout(Duration::from_secs(8)) {
        Ok(Ok(())) => {
            *state()
                .lock()
                .map_err(|_| "wake word state poisoned".to_string())? = Some(WakewordHandle {
                stop: stop_tx,
                thread,
            });
            Ok(format!("Wake word listener running for: {labels}"))
        }
        Ok(Err(error)) => {
            let _ = stop_tx.send(());
            let _ = thread.join();
            Err(error)
        }
        Err(_) => {
            let _ = stop_tx.send(());
            let _ = thread.join();
            Err("Wake word listener did not start in time".into())
        }
    }
}

pub fn stop() {
    let mut guard = match state().lock() {
        Ok(value) => value,
        Err(_) => return,
    };

    if let Some(handle) = guard.take() {
        let _ = handle.stop.send(());
        let _ = handle.thread.join();
    }
}

fn build_config(config: &Config) -> Result<WakewordConfig, String> {
    let model_paths = parse_wake_words(&config.wakeword_list);
    if model_paths.is_empty() {
        return Err("Wake word model list is empty".into());
    }

    for path in &model_paths {
        if !Path::new(path).exists() {
            return Err(format!(
                "Wake word `{path}` is not a Rustpotter model/reference file path"
            ));
        }
    }

    Ok(WakewordConfig { model_paths })
}

fn run_listener(
    config: WakewordConfig,
    stop_rx: mpsc::Receiver<()>,
    ready_tx: mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No default microphone found".to_string())?;
    let supported = device
        .default_input_config()
        .map_err(|error| error.to_string())?;
    let sample_rate = supported.sample_rate();

    let mut rustpotter_config = RustpotterConfig::default();
    rustpotter_config.fmt = AudioFmt {
        sample_rate: sample_rate as usize,
        channels: 1,
        ..AudioFmt::default()
    };

    let mut rustpotter = Rustpotter::new(&rustpotter_config)?;
    for path in &config.model_paths {
        rustpotter.add_wakeword_from_file(&model_key(path), path)?;
    }

    let frame_length = rustpotter.get_samples_per_frame();
    let (frames_tx, frames_rx) = mpsc::channel::<Vec<i16>>();
    let stream = build_input_stream(device, supported, frame_length, frames_tx)?;
    stream.play().map_err(|error| error.to_string())?;
    let _ = ready_tx.send(Ok(()));

    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }

        match frames_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(frame) => {
                if let Some(detection) = rustpotter.process_samples(frame) {
                    eprintln!(
                        "Wake word detected: {} ({:.2})",
                        detection.name, detection.score
                    );
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    drop(stream);
    Ok(())
}

fn build_input_stream(
    device: cpal::Device,
    supported: cpal::SupportedStreamConfig,
    frame_length: usize,
    frames_tx: mpsc::Sender<Vec<i16>>,
) -> Result<cpal::Stream, String> {
    let channels = supported.channels() as usize;
    let stream_config: cpal::StreamConfig = supported.clone().into();
    let err_fn = |error| eprintln!("wake word audio stream error: {error}");
    let mut buffer = Vec::<i16>::new();

    match supported.sample_format() {
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _| {
                    push_samples(
                        data.iter().map(|v| float_to_i16(*v)),
                        channels,
                        frame_length,
                        &frames_tx,
                        &mut buffer,
                    )
                },
                err_fn,
                None,
            )
            .map_err(|error| error.to_string()),
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    push_samples(
                        data.iter().copied(),
                        channels,
                        frame_length,
                        &frames_tx,
                        &mut buffer,
                    )
                },
                err_fn,
                None,
            )
            .map_err(|error| error.to_string()),
        cpal::SampleFormat::U16 => device
            .build_input_stream(
                &stream_config,
                move |data: &[u16], _| {
                    push_samples(
                        data.iter().map(|v| u16_to_i16(*v)),
                        channels,
                        frame_length,
                        &frames_tx,
                        &mut buffer,
                    )
                },
                err_fn,
                None,
            )
            .map_err(|error| error.to_string()),
        other => Err(format!("Unsupported microphone sample format: {other:?}")),
    }
}

fn push_samples(
    samples: impl Iterator<Item = i16>,
    channels: usize,
    frame_length: usize,
    frames_tx: &mpsc::Sender<Vec<i16>>,
    buffer: &mut Vec<i16>,
) {
    for (idx, sample) in samples.enumerate() {
        if idx % channels == 0 {
            buffer.push(sample);
        }
        if buffer.len() >= frame_length {
            let frame = buffer.drain(..frame_length).collect::<Vec<_>>();
            let _ = frames_tx.send(frame);
        }
    }
}

fn parse_wake_words(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn model_key(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
        .to_string()
}

fn float_to_i16(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}

fn u16_to_i16(value: u16) -> i16 {
    (value as i32 - 32_768) as i16
}
