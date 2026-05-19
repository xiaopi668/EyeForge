import logging
import ctypes
from ctypes import wintypes
from typing import Callable

from PyQt5.QtCore import QObject, pyqtSignal, QAbstractNativeEventFilter
from PyQt5.QtWidgets import QApplication

logger = logging.getLogger(__name__)

WM_HOTKEY = 0x0312

_MOD_ALT = 0x0001
_MOD_CTRL = 0x0002
_MOD_SHIFT = 0x0004
_MOD_WIN = 0x0008

_user32 = ctypes.windll.user32

_key_map = {}
_next_id = 1000


def _parse_keys(hotkey: str):
    parts = hotkey.lower().split("+")
    mod = 0
    vk = 0
    for p in parts:
        p = p.strip()
        if p in ("ctrl", "control"):
            mod |= _MOD_CTRL
        elif p in ("alt",):
            mod |= _MOD_ALT
        elif p in ("shift",):
            mod |= _MOD_SHIFT
        elif p in ("win", "windows", "super"):
            mod |= _MOD_WIN
        elif p == "e":
            vk = ord("E")
        elif p == "v":
            vk = ord("V")
        elif p == "a":
            vk = ord("A")
        elif p == "d":
            vk = ord("D")
        elif p == "s":
            vk = ord("S")
        elif p == "f":
            vk = ord("F")
        elif p == "r":
            vk = ord("R")
        elif p == "t":
            vk = ord("T")
        elif p == "space":
            vk = 0x20
        elif p.startswith("0x"):
            vk = int(p, 16)
        elif p.isdigit():
            vk = ord(p) if len(p) == 1 else int(p)
    return mod, vk


def _register(mod: int, vk: int, action: str) -> bool:
    global _next_id
    if not vk:
        return False
    fid = _next_id
    _next_id += 1
    ok = _user32.RegisterHotKey(None, fid, mod, vk)
    if ok:
        _key_map[fid] = action
    return bool(ok)


def _unregister_all():
    for fid in list(_key_map.keys()):
        _user32.UnregisterHotKey(None, fid)
    _key_map.clear()


class _HotkeyEmitter(QObject):
    triggered = pyqtSignal(str)


class _WinFilter(QAbstractNativeEventFilter):
    def nativeEventFilter(self, event_type, message):
        if event_type == "windows_generic_MSG":
            msg = ctypes.wintypes.MSG.from_address(message.__int__())
            if msg.message == WM_HOTKEY:
                fid = msg.wParam
                action = _key_map.get(fid)
                if action:
                    _emitter.triggered.emit(action)
                    return True, 0
        return False, 0


_emitter = _HotkeyEmitter()
_filter = _WinFilter()
_installed = False


def start(key: str, action: str):
    mod, vk = _parse_keys(key)
    if _register(mod, vk, action):
        logger.info(f"Win hotkey registered: {key} -> {action}")
    else:
        logger.warning(f"Failed to register hotkey: {key}")


def stop_all():
    _unregister_all()


def connect(callback: Callable[[str], None]):
    global _installed
    if not _installed:
        QApplication.instance().installNativeEventFilter(_filter)
        _installed = True
    _emitter.triggered.connect(callback)


def disconnect(callback: Callable[[str], None]):
    try:
        _emitter.triggered.disconnect(callback)
    except Exception:
        pass


def is_available() -> bool:
    try:
        ctypes.windll.user32.RegisterHotKey
        return True
    except Exception:
        return False
