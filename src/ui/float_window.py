import os
import logging
from PyQt5.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLineEdit,
    QPushButton, QLabel, QApplication, QScrollArea,
    QFrame, QTextEdit,
)
from PyQt5.QtCore import Qt, QTimer
from PyQt5.QtGui import QIcon, QFont

from src.utils.voice import VoiceThread, is_available as voice_available

logger = logging.getLogger(__name__)


class ShellItem(QFrame):
    def __init__(self, command: str, output: str, parent=None):
        super().__init__(parent)
        self._expanded = False
        self._command = command
        self._output = output
        self._init_ui()

    def _init_ui(self):
        self.setStyleSheet("ShellItem { background: #3d3d3d; border: 1px solid #555; border-radius: 4px; margin: 2px 0; }")
        layout = QVBoxLayout(self)
        layout.setContentsMargins(8, 4, 8, 4)
        layout.setSpacing(2)

        self._header = QLabel("> shell")
        self._header.setStyleSheet("color: #00d4aa; font-family: monospace; font-size: 12px;")
        self._header.setCursor(Qt.PointingHandCursor)
        self._header.mousePressEvent = lambda e: self._toggle()
        layout.addWidget(self._header)

        self._detail = QTextEdit()
        self._detail.setReadOnly(True)
        self._detail.setVisible(False)
        self._detail.setMaximumHeight(0)
        self._detail.setStyleSheet(
            "QTextEdit { background: #2d2d2d; color: #ccc; border: none; "
            "font-family: monospace; font-size: 11px; padding: 4px; }"
        )
        layout.addWidget(self._detail)

    def _toggle(self):
        self._expanded = not self._expanded
        if self._expanded:
            detail = f"$ {self._command}\n\n{self._output}"
            self._detail.setPlainText(detail)
            self._detail.setVisible(True)
            self._detail.setMaximumHeight(300)
            self._header.setText("▾ shell")
            self._header.setStyleSheet("color: #fdcb6e; font-family: monospace; font-size: 12px;")
        else:
            self._detail.setVisible(False)
            self._detail.setMaximumHeight(0)
            self._header.setText("> shell")
            self._header.setStyleSheet("color: #00d4aa; font-family: monospace; font-size: 12px;")


class ChatBubble(QFrame):
    def __init__(self, text: str, is_user: bool, parent=None):
        super().__init__(parent)
        self.setStyleSheet(
            "ChatBubble { background: transparent; border: none; }"
        )
        layout = QVBoxLayout(self)
        layout.setContentsMargins(4, 2, 4, 2)

        align = Qt.AlignRight if is_user else Qt.AlignLeft
        bg = "#005a4e" if is_user else "#3d3d3d"
        fg = "#eee"

        self._label = QLabel(text)
        self._label.setWordWrap(True)
        self._label.setStyleSheet(
            f"background: {bg}; color: {fg}; padding: 8px 12px; "
            f"border-radius: 8px; font-size: 13px; max-width: 360px;"
        )
        self._label.setAlignment(align)
        layout.addWidget(self._label, 0, align)

    def add_shell(self, command: str, output: str):
        layout = self.layout()
        item = ShellItem(command, output)
        layout.addWidget(item, 0, Qt.AlignLeft)


class FloatWindow(QWidget):

    def __init__(self, config: dict = None):
        super().__init__()
        self.config = config or {}
        self._voice_thread = None
        self._messages = []
        self._init_ui()

    def _init_ui(self):
        self.setWindowTitle("EyeForge")
        self.setWindowFlags(Qt.WindowStaysOnTopHint | Qt.FramelessWindowHint | Qt.Window)
        self.setAttribute(Qt.WA_ShowWithoutActivating)
        self.setFixedSize(480, 500)
        self.setStyleSheet("background-color: #2d2d2d; border: 1px solid #555; border-radius: 10px;")

        icon_path = os.path.join(os.path.dirname(__file__), "..", "..", "src", "logo.ico")
        if os.path.exists(icon_path):
            self.setWindowIcon(QIcon(icon_path))

        layout = QVBoxLayout(self)
        layout.setContentsMargins(8, 8, 8, 8)
        layout.setSpacing(6)

        header_row = QHBoxLayout()
        title = QLabel("🧿 EyeForge")
        title.setStyleSheet("font-size: 14px; font-weight: bold; color: #00d4aa; border: none;")
        header_row.addWidget(title)
        header_row.addStretch()
        self._status_label = QLabel("")
        self._status_label.setStyleSheet("color: #888; font-size: 11px;")
        header_row.addWidget(self._status_label)
        self._close_btn = QPushButton("✕")
        self._close_btn.setStyleSheet(
            "QPushButton { background: transparent; color: #888; border: none; "
            "font-size: 18px; padding: 2px 6px; }"
            "QPushButton:hover { color: #e17055; }"
        )
        self._close_btn.clicked.connect(self.hide)
        header_row.addWidget(self._close_btn)
        layout.addLayout(header_row)

        self._scroll = QScrollArea()
        self._scroll.setWidgetResizable(True)
        self._scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)
        self._scroll.setStyleSheet(
            "QScrollArea { background: transparent; border: none; }"
            "QScrollBar:vertical { width: 6px; background: #2d2d2d; }"
            "QScrollBar::handle:vertical { background: #555; border-radius: 3px; }"
        )
        self._chat_widget = QWidget()
        self._chat_layout = QVBoxLayout(self._chat_widget)
        self._chat_layout.setAlignment(Qt.AlignTop)
        self._chat_layout.setSpacing(4)
        self._chat_layout.setContentsMargins(4, 4, 4, 4)
        self._scroll.setWidget(self._chat_widget)
        layout.addWidget(self._scroll, 1)

        self.input_field = QLineEdit()
        self.input_field.setStyleSheet(
            "QLineEdit { padding: 8px; font-size: 13px; border: 1px solid #555; "
            "border-radius: 6px; background: #3d3d3d; color: #eee; }"
            "QLineEdit:focus { border-color: #00d4aa; }"
        )
        self.input_field.returnPressed.connect(self._submit)
        layout.addWidget(self.input_field)

        btn_row = QHBoxLayout()
        self.submit_btn = QPushButton()
        self.submit_btn.setStyleSheet(
            "QPushButton { background: #00d4aa; color: #111; font-weight: bold; "
            "padding: 6px 20px; border-radius: 6px; border: none; }"
            "QPushButton:hover { background: #00b894; }"
        )
        self.submit_btn.clicked.connect(self._submit)

        self.voice_btn = QPushButton("🎤")
        self.voice_btn.setStyleSheet(
            "QPushButton { background: #6c5ce7; color: white; font-weight: bold; "
            "padding: 6px 16px; border-radius: 6px; font-size: 16px; border: none; }"
            "QPushButton:hover { background: #5a4bd1; }"
            "QPushButton:disabled { background: #555; }"
        )
        self.voice_btn.clicked.connect(self._start_voice)

        btn_row.addWidget(self.submit_btn)
        btn_row.addWidget(self.voice_btn)
        btn_row.addStretch()
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
        if not text:
            return
        self.input_field.clear()
        self._add_user_message(text)
        self._status_label.setText(self._tr("⏳ 执行中...", "⏳ Executing..."))
        self.submit_btn.setEnabled(False)
        self.input_field.setEnabled(False)

        from src.core.agent import EyeForgeAgent, StepCallback

        class FloatCallback(StepCallback):
            def __init__(self, fw):
                self.fw = fw
                self._shells = []
            def on_shell_command(self, cmd, output):
                self._shells.append((cmd, output))
            def on_result(self, result: str):
                bubble = ChatBubble(result, False, self.fw)
                for cmd, output in self._shells:
                    bubble.add_shell(cmd, output)
                self.fw._chat_layout.addWidget(bubble)
                QTimer.singleShot(0, self.fw._scroll_to_bottom)
            def on_error(self, error: str):
                self.fw._add_ai_message(f"✗ {error}")
            def on_status(self, msg):
                self.fw._status_label.setText(msg)

        cb = FloatCallback(self)

        def _run():
            try:
                agent = EyeForgeAgent(self.config, cb)
                agent.run(text)
            finally:
                self.submit_btn.setEnabled(True)
                self.input_field.setEnabled(True)
                self._status_label.setText("")

        import threading
        t = threading.Thread(target=_run, daemon=True)
        t.start()

    def _tr(self, zh: str, en: str) -> str:
        return en if self.config.get("language", "zh") == "en" else zh

    def _add_user_message(self, text: str):
        bubble = ChatBubble(text, True, self)
        self._chat_layout.addWidget(bubble)
        QTimer.singleShot(0, self._scroll_to_bottom)

    def _add_ai_message(self, text: str):
        bubble = ChatBubble(text, False, self)
        self._chat_layout.addWidget(bubble)
        QTimer.singleShot(0, self._scroll_to_bottom)

    def _scroll_to_bottom(self):
        scrollbar = self._scroll.verticalScrollBar()
        scrollbar.setValue(scrollbar.maximum())

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
