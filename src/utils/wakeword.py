import os
import logging
import struct
import threading
from typing import Callable

logger = logging.getLogger(__name__)

_running = False
_thread = None


def start(callback: Callable[[str], None], wake_words: list = None,
          access_key: str = "", lang: str = "zh"):
    global _running, _thread
    stop()
    try:
        import pvporcupine
        import pyaudio
    except ImportError as e:
        logger.warning(f"Wake word dependencies missing: {e}")
        return

    if not access_key:
        logger.warning("Porcupine access_key is required")
        return

    try:
        builtin_keywords = []
        custom_paths = []
        for ww in (wake_words or []):
            ww = ww.strip()
            if not ww:
                continue
            if os.path.isfile(ww):
                custom_paths.append(ww)
            elif ww.lower() in pvporcupine.KEYWORDS:
                builtin_keywords.append(ww.lower())
            else:
                logger.warning(f"Unknown wake word '{ww}', treating as .ppn path")
                custom_paths.append(ww)

        if not builtin_keywords and not custom_paths:
            builtin_keywords = ["computer"]

        porcupine = pvporcupine.create(
            access_key=access_key,
            keywords=builtin_keywords or None,
            keyword_paths=custom_paths or None,
        )
    except Exception as e:
        logger.warning(f"Failed to create Porcupine: {e}")
        return

    _running = True

    labels = builtin_keywords + [os.path.basename(p) for p in custom_paths]

    def _loop():
        pa = pyaudio.PyAudio()
        try:
            stream = pa.open(
                rate=porcupine.sample_rate,
                channels=1,
                format=pyaudio.paInt16,
                input=True,
                frames_per_buffer=porcupine.frame_length,
            )
            logger.info(f"Porcupine listening for: {labels}")
            while _running:
                pcm = stream.read(porcupine.frame_length, exception_on_overflow=False)
                pcm_unpacked = struct.unpack_from("h" * porcupine.frame_length, pcm)
                result = porcupine.process(pcm_unpacked)
                if result >= 0:
                    keyword = labels[result]
                    logger.info(f"Wake word detected: {keyword}")
                    callback(keyword)
        except Exception as e:
            logger.warning(f"Porcupine loop error: {e}")
        finally:
            try:
                stream.stop_stream()
                stream.close()
            except Exception:
                pass
            pa.terminate()
            porcupine.delete()

    _thread = threading.Thread(target=_loop, daemon=True)
    _thread.start()


def stop():
    global _running, _thread
    _running = False
    if _thread and _thread.is_alive():
        _thread.join(timeout=2)
    _thread = None


def is_available() -> bool:
    try:
        import pvporcupine
        import pyaudio
        return True
    except ImportError:
        return False
