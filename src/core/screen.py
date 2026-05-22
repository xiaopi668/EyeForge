import mss
import mss.tools
import numpy as np
from PIL import Image
from PyQt5.QtGui import QImage, QPixmap


class ScreenCapture:
    def __init__(self):
        self._sct = None
        self._monitor_index = 1
        self.scale_x = 1.0
        self.scale_y = 1.0

    def _ensure_init(self):
        if self._sct is not None:
            return
        import pyautogui
        self._sct = mss.mss()
        native = self._sct.monitors[1]
        nw, nh = native["width"], native["height"]
        vw, vh = pyautogui.size()
        self.scale_x = vw / nw if nw else 1.0
        self.scale_y = vh / nh if nh else 1.0

    @property
    def monitors(self):
        self._ensure_init()
        return self._sct.monitors

    @property
    def current_monitor(self):
        self._ensure_init()
        return self._sct.monitors[self._monitor_index]

    def set_monitor(self, index: int):
        self._ensure_init()
        if 0 < index < len(self._sct.monitors):
            self._monitor_index = index
            import pyautogui
            native = self._sct.monitors[self._monitor_index]
            nw, nh = native["width"], native["height"]
            vw, vh = pyautogui.size()
            self.scale_x = vw / nw if nw else 1.0
            self.scale_y = vh / nh if nh else 1.0

    def _resize_to_virtual(self, img: Image.Image) -> Image.Image:
        if self.scale_x == 1.0 and self.scale_y == 1.0:
            return img
        new_w = max(1, round(img.width * self.scale_x))
        new_h = max(1, round(img.height * self.scale_y))
        return img.resize((new_w, new_h), Image.LANCZOS)

    def capture_pil(self, region=None) -> Image.Image:
        self._ensure_init()
        if region:
            monitor = {
                "left": int(region["left"]),
                "top": int(region["top"]),
                "width": int(region["width"]),
                "height": int(region["height"]),
            }
        else:
            monitor = self.current_monitor
        sct_img = self._sct.grab(monitor)
        img = Image.frombytes("RGB", sct_img.size, sct_img.rgb)
        return self._resize_to_virtual(img)

    def capture_numpy(self, region=None) -> np.ndarray:
        img = self.capture_pil(region)
        return np.array(img)

    def capture_pixmap(self, region=None) -> QPixmap:
        img = self.capture_pil(region)
        qimg = QImage(
            img.tobytes(), img.width, img.height, QImage.Format_RGB888
        )
        return QPixmap.fromImage(qimg)

    def get_screen_size(self):
        import pyautogui
        return pyautogui.size()
