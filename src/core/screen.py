import mss
import mss.tools
import numpy as np
from PIL import Image
from PyQt5.QtGui import QImage, QPixmap


def _detect_scale():
    import pyautogui
    sct = mss.mss()
    native = sct.monitors[1]
    nw, nh = native["width"], native["height"]
    vw, vh = pyautogui.size()
    return vw / nw if nw else 1.0, vh / nh if nh else 1.0


class ScreenCapture:
    def __init__(self):
        self.sct = mss.mss()
        self._monitor_index = 1
        self.scale_x, self.scale_y = _detect_scale()

    @property
    def monitors(self):
        return self.sct.monitors

    @property
    def current_monitor(self):
        return self.sct.monitors[self._monitor_index]

    def set_monitor(self, index: int):
        if 0 < index < len(self.sct.monitors):
            self._monitor_index = index
            self.scale_x, self.scale_y = _detect_scale()

    def _resize_to_virtual(self, img: Image.Image) -> Image.Image:
        if self.scale_x == 1.0 and self.scale_y == 1.0:
            return img
        new_w = max(1, round(img.width * self.scale_x))
        new_h = max(1, round(img.height * self.scale_y))
        return img.resize((new_w, new_h), Image.LANCZOS)

    def capture_pil(self, region=None) -> Image.Image:
        if region:
            monitor = {
                "left": int(region["left"]),
                "top": int(region["top"]),
                "width": int(region["width"]),
                "height": int(region["height"]),
            }
        else:
            monitor = self.current_monitor
        sct_img = self.sct.grab(monitor)
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
