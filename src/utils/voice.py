import logging
import threading
from typing import Optional

logger = logging.getLogger(__name__)

_recorder = None


def is_available() -> bool:
    try:
        import speech_recognition as sr
        return True
    except ImportError:
        return False


def transcribe(timeout: int = 5, phrase_limit: int = 10) -> Optional[str]:
    try:
        import speech_recognition as sr
        recognizer = sr.Recognizer()
        with sr.Microphone() as source:
            recognizer.adjust_for_ambient_noise(source, duration=0.3)
            audio = recognizer.listen(source, timeout=timeout, phrase_time_limit=phrase_limit)
        text = recognizer.recognize_google(audio, language="zh-CN")
        return text
    except sr.WaitTimeoutError:
        return None
    except sr.UnknownValueError:
        return None
    except sr.RequestError as e:
        logger.warning(f"Speech recognition request error: {e}")
        return None
    except Exception as e:
        logger.warning(f"Speech recognition error: {e}")
        return None


class VoiceThread(threading.Thread):
    def __init__(self, callback, timeout=5, phrase_limit=10):
        super().__init__(daemon=True)
        self.callback = callback
        self.timeout = timeout
        self.phrase_limit = phrase_limit
        self._abort = False

    def run(self):
        result = transcribe(self.timeout, self.phrase_limit)
        if not self._abort:
            self.callback(result)

    def abort(self):
        self._abort = True
