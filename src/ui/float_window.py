import os
import logging
from PyQt5.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLineEdit,
    QPushButton, QLabel, QApplication,
)
from PyQt5.QtCore import Qt, pyqtSignal
from PyQt5.QtGui import QIcon, QFont

from src.utils.voice import VoiceThread, is_available as voice_available

logger = logging.getLogger(__name__)


class FloatWindow(QWidget):
    task_submitted = pyqtSignal(str)

    def __init__(self, config: dict = None):
        super().__init__()
        self.config = config or {}
        self._voice_thread = None
        self._init_ui()

    def _init_ui(self):
        self.setWindowTitle("EyeForge")
        self.setWindowFlags(Qt.WindowStaysOnTopHint | Qt.FramelessWindowHint | Qt.Window)
        self.setAttribute(Qt.WA_ShowWithoutActivating)
        self.setFixedSize(480, 130)
        self.setStyleSheet("background-color: #2d2d2d; border: 1px solid #555; border-radius: 10px;")

        icon_path = os.path.join(os.path.dirname(__file__), "..", "..", "src", "logo.ico")
        if os.path.exists(icon_path):
            self.setWindowIcon(QIcon(icon_path))

        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 10, 16, 10)

        title = QLabel("🧿 EyeForge")
        title.setStyleSheet("font-size: 14px; font-weight: bold; color: #00d4aa; border: none;")
        layout.addWidget(title)

        self.input_field = QLineEdit()
        self.input_field.setStyleSheet("""
            QLineEdit {
                padding: 8px; font-size: 13px; border: 1px solid #555;
                border-radius: 6px; background: #3d3d3d; color: #eee;
            }
            QLineEdit:focus { border-color: #00d4aa; }
        """)
        self.input_field.returnPressed.connect(self._submit)
        layout.addWidget(self.input_field)

        btn_row = QHBoxLayout()
        self.submit_btn = QPushButton()
        self.submit_btn.setStyleSheet("""
            QPushButton { background: #00d4aa; color: #111; font-weight: bold;
                          padding: 6px 20px; border-radius: 6px; border: none; }
            QPushButton:hover { background: #00b894; }
        """)
        self.submit_btn.clicked.connect(self._submit)

        self.voice_btn = QPushButton("🎤")
        self.voice_btn.setStyleSheet("""
            QPushButton { background: #6c5ce7; color: white; font-weight: bold;
                          padding: 6px 16px; border-radius: 6px; font-size: 16px; border: none; }
            QPushButton:hover { background: #5a4bd1; }
            QPushButton:disabled { background: #555; }
        """)
        self.voice_btn.clicked.connect(self._start_voice)

        self.close_btn = QPushButton("✕")
        self.close_btn.setStyleSheet("""
            QPushButton { background: transparent; color: #888; border: none;
                          font-size: 18px; padding: 4px 8px; }
            QPushButton:hover { color: #e17055; }
        """)
        self.close_btn.clicked.connect(self.hide)

        btn_row.addWidget(self.submit_btn)
        btn_row.addWidget(self.voice_btn)
        btn_row.addStretch()
        btn_row.addWidget(self.close_btn)
        layout.addLayout(btn_row)

        self._retranslate()
        self._center_on_screen()

    def _retranslate(self):
        en = self.config.get("language", "zh") == "en"
        self.input_field.setPlaceholderText("Enter a task..." if en else "输入任务...")
        self.submit_btn.setText("▶ Send" if en else "▶ 发送")
        self.voice_btn.setToolTip("Voice Input" if en else "语音输入")
        if not voice_available():
            self.voice_btn.setToolTip("Install speech_recognition" if en else "需要安装 speech_recognition 库")

    def _center_on_screen(self):
        screen = QApplication.primaryScreen()
        if screen:
            geo = screen.geometry()
            x = (geo.width() - self.width()) // 2
            y = (geo.height() - self.height()) // 2 - 60
            self.move(x, y)

    def _submit(self):
        text = self.input_field.text().strip()
        if text:
            self.task_submitted.emit(text)
            self.input_field.clear()
            self.hide()

    def _start_voice(self):
        if self._voice_thread and self._voice_thread.is_alive():
            return
        self.voice_btn.setEnabled(False)
        self.voice_btn.setText("🎤 ...")
        QApplication.processEvents()

        def on_result(text):
            self.voice_btn.setEnabled(True)
            self.voice_btn.setText("🎤")
            if text:
                self.input_field.setText(text)
            else:
                self.voice_btn.setText("🎤 ✗")

        self._voice_thread = VoiceThread(on_result)
        self._voice_thread.start()

    def show_float(self):
        self.input_field.clear()
        self._center_on_screen()
        self.showNormal()
        self.raise_()
        self.activateWindow()
        self.input_field.setFocus()

    def keyPressEvent(self, event):
        if event.key() == Qt.Key_Escape:
            self.hide()
        super().keyPressEvent(event)

    def closeEvent(self, event):
        self.hide()
        event.ignore()
