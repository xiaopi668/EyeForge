use std::io::Cursor;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::multipart::{Form, Part};
use serde::Serialize;

use crate::config::Config;
use crate::crypto;

#[derive(Debug, Clone, Serialize)]
pub struct VoiceDevice {
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceTranscript {
    pub text: String,
    pub seconds: u32,
    pub sample_rate: u32,
    pub channels: u16,
}

pub fn list_input_devices() -> Result<Vec<VoiceDevice>, String> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|device| device.description().ok())
        .map(|device| device.name().to_string());

    let devices = host
        .input_devices()
        .map_err(|error| error.to_string())?
        .filter_map(|device| {
            let name = device.description().ok()?.name().to_string();
            Some(VoiceDevice {
                is_default: default_name.as_ref().is_some_and(|value| value == &name),
                name,
            })
        })
        .collect::<Vec<_>>();

    Ok(devices)
}

pub async fn transcribe_default_input(
    seconds: u32,
    config: Config,
) -> Result<VoiceTranscript, String> {
    let capture = tokio::task::spawn_blocking(move || capture_default_input(seconds))
        .await
        .map_err(|error| error.to_string())??;

    let text = transcribe_wav(
        capture.wav_bytes,
        capture.sample_rate,
        capture.channels,
        &config,
    )
    .await?;

    Ok(VoiceTranscript {
        text,
        seconds,
        sample_rate: capture.sample_rate,
        channels: capture.channels,
    })
}

struct CapturedAudio {
    wav_bytes: Vec<u8>,
    sample_rate: u32,
    channels: u16,
}

fn capture_default_input(seconds: u32) -> Result<CapturedAudio, String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No default microphone found".to_string())?;
    let supported = device
        .default_input_config()
        .map_err(|error| error.to_string())?;

    let sample_rate = supported.sample_rate();
    let channels = supported.channels();
    let samples = Arc::new(Mutex::new(Vec::<i16>::new()));
    let sink = Arc::clone(&samples);

    let err_fn = |error| eprintln!("voice stream error: {error}");

    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &supported.clone().into(),
                move |data: &[f32], _| {
                    if let Ok(mut guard) = sink.lock() {
                        guard.extend(data.iter().map(|sample| float_to_i16(*sample)));
                    }
                },
                err_fn,
                None,
            )
            .map_err(|error| error.to_string())?,
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &supported.clone().into(),
                move |data: &[i16], _| {
                    if let Ok(mut guard) = sink.lock() {
                        guard.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            )
            .map_err(|error| error.to_string())?,
        cpal::SampleFormat::U16 => device
            .build_input_stream(
                &supported.clone().into(),
                move |data: &[u16], _| {
                    if let Ok(mut guard) = sink.lock() {
                        guard.extend(data.iter().map(|sample| u16_to_i16(*sample)));
                    }
                },
                err_fn,
                None,
            )
            .map_err(|error| error.to_string())?,
        other => {
            return Err(format!("Unsupported sample format: {other:?}"));
        }
    };

    stream.play().map_err(|error| error.to_string())?;
    std::thread::sleep(std::time::Duration::from_secs(seconds.max(1) as u64));
    drop(stream);

    let pcm = samples
        .lock()
        .map_err(|_| "audio sample buffer poisoned".to_string())?
        .clone();

    let mut cursor = Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    {
        let mut writer =
            hound::WavWriter::new(&mut cursor, spec).map_err(|error| error.to_string())?;
        for sample in pcm {
            writer
                .write_sample(sample)
                .map_err(|error| error.to_string())?;
        }
        writer.finalize().map_err(|error| error.to_string())?;
    }

    Ok(CapturedAudio {
        wav_bytes: cursor.into_inner(),
        sample_rate,
        channels,
    })
}

async fn transcribe_wav(
    wav_bytes: Vec<u8>,
    _sample_rate: u32,
    _channels: u16,
    config: &Config,
) -> Result<String, String> {
    match config.llm_provider.as_str() {
        "openai" => {
            transcribe_openai_compatible(
                "https://api.openai.com/v1",
                &crypto::decrypt(&config.openai_api_key),
                "gpt-4o-mini-transcribe",
                wav_bytes,
            )
            .await
        }
        "custom" => {
            transcribe_openai_compatible(
                &config.custom_base_url,
                &crypto::decrypt(&config.custom_api_key),
                "whisper-1",
                wav_bytes,
            )
            .await
        }
        other => Err(format!(
            "Voice transcription is not wired for provider `{other}` yet"
        )),
    }
}

async fn transcribe_openai_compatible(
    base_url: &str,
    api_key: &str,
    model: &str,
    wav_bytes: Vec<u8>,
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("Missing API key for voice transcription".into());
    }

    let endpoint = format!("{}/audio/transcriptions", base_url.trim_end_matches('/'));
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(|e| e.to_string())?,
    );

    let file_part = Part::bytes(wav_bytes)
        .file_name("capture.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;
    let form = Form::new()
        .part("file", file_part)
        .text("model", model.to_string());

    let response: serde_json::Value = reqwest::Client::new()
        .post(endpoint)
        .headers(headers)
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    response["text"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "Invalid voice transcription response".into())
}

fn float_to_i16(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}

fn u16_to_i16(value: u16) -> i16 {
    (value as i32 - 32_768) as i16
}
