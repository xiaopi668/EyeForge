import logging
from PyQt5.QtWidgets import QWidget, QLabel, QApplication
from PyQt5.QtCore import Qt, QRect, QTimer
from PyQt5.QtGui import QPainter, QColor, QPen, QFont, QPixmap

logger = logging.getLogger(__name__)


class ClickOverlay(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowFlags(
            Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.Tool
        )
        self.setAttribute(Qt.WA_TranslucentBackground)
        self.setAttribute(Qt.WA_ShowWithoutActivating)
        self._target_x = -1
        self._target_y = -1
        self._opacity = 0.8
        self._fade_step = 0
        self._timer = QTimer(self)
        self._timer.timeout.connect(self._fade_out)

    def show_click(self, x: int, y: int):
        self._target_x = x
        self._target_y = y
        self._opacity = 0.8
        self._fade_step = 0
        try:
            self.show()
            self.raise_()
            self.update()
            self._timer.start(50)
        except Exception as e:
            logger.debug(f"Overlay show failed: {e}")

    def _fade_out(self):
        self._fade_step += 1
        if self._fade_step > 15:
            self._timer.stop()
            try:
                self.hide()
            except Exception:
                pass
            return
        self._opacity = max(0, 0.8 - self._fade_step * 0.05)
        try:
            self.update()
        except Exception:
            pass

    def paintEvent(self, event):
        if self._target_x < 0:
            return
        painter = QPainter(self)
        painter.setRenderHint(QPainter.Antialiasing)

        pen = QPen(QColor(255, 0, 0, int(255 * self._opacity)))
        pen.setWidth(3)
        painter.setPen(pen)
        brush = QColor(255, 0, 0, int(50 * self._opacity))
        painter.setBrush(brush)

        r = 20
        painter.drawEllipse(
            self._target_x - r, self._target_y - r, r * 2, r * 2
        )

        pen.setWidth(2)
        painter.setPen(pen)
        painter.drawLine(self._target_x - r - 5, self._target_y,
                         self._target_x + r + 5, self._target_y)
        painter.drawLine(self._target_x, self._target_y - r - 5,
                         self._target_x, self._target_y + r + 5)

    def resize_to_screen(self, x: int, y: int, width: int, height: int):
        self.setGeometry(x, y, width, height)


class ScreenPreview(QLabel):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setAlignment(Qt.AlignCenter)
        self.setMinimumSize(320, 240)
        self.setStyleSheet("background-color: #1a1a1a; border: 1px solid #333;")
        self._source_pixmap = None
        self._markers = []

    def set_placeholder(self, text: str):
        self.setText(text)

    def update_preview(self, pixmap: QPixmap):
        self._source_pixmap = pixmap
        self._markers.clear()
        self._render()

    def add_marker(self, x: int, y: int, label: str = ""):
        if self._source_pixmap is None:
            return
        sw = self._source_pixmap.width()
        sh = self._source_pixmap.height()
        if sw == 0 or sh == 0:
            return
        pw, ph = self._preview_size()
        if pw == 0 or ph == 0:
            return
        scale_x = pw / sw
        scale_y = ph / sh
        scale = min(scale_x, scale_y)
        display_w = int(sw * scale)
        display_h = int(sh * scale)
        offset_x = (pw - display_w) // 2
        offset_y = (ph - display_h) // 2
        self._markers.append({
            "x": int(x * scale + offset_x),
            "y": int(y * scale + offset_y),
            "label": label,
        })
        self._render()

    def _preview_size(self):
        return self.width(), self.height()

    def _render(self):
        if self._source_pixmap is None:
            return
        pw, ph = self._preview_size()
        scaled = self._source_pixmap.scaled(
            pw, ph, Qt.KeepAspectRatio, Qt.SmoothTransformation,
        )
        if self._markers:
            from PyQt5.QtGui import QPainter, QColor, QPen, QFont
            marked = scaled.copy()
            painter = QPainter(marked)
            painter.setRenderHint(QPainter.Antialiasing)
            for m in self._markers:
                pen = QPen(QColor(255, 0, 0))
                pen.setWidth(3)
                painter.setPen(pen)
                painter.drawEllipse(m["x"] - 8, m["y"] - 8, 16, 16)
                painter.drawLine(m["x"] - 12, m["y"], m["x"] + 12, m["y"])
                painter.drawLine(m["x"], m["y"] - 12, m["x"], m["y"] + 12)
                if m["label"]:
                    painter.setFont(QFont("Microsoft YaHei UI", 8))
                    painter.drawText(m["x"] + 12, m["y"] - 8, m["label"])
            painter.end()
            self.setPixmap(marked)
        else:
            self.setPixmap(scaled)
