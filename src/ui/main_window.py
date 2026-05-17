import json
import os
import html
import logging
from datetime import datetime
from typing import Optional

from PyQt5.QtWidgets import (
    QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QTextEdit, QLineEdit, QPushButton, QLabel,
    QMessageBox, QApplication, QSystemTrayIcon, QMenu, QAction,
    QTabWidget, QSplitter, QFrame
)
from PyQt5.QtCore import Qt, QThread, pyqtSignal, QTimer
from PyQt5.QtGui import QFont, QIcon, QPixmap, QTextCursor

from src.ui.settings_dialog import SettingsDialog
from src.ui.overlay import ClickOverlay, ScreenPreview
from src.core.agent import EyeForgeAgent, StepCallback
from src.version import VERSION
from src.utils.updater import check_update

APP_TITLE = "EyeForge"

logger = logging.getLogger(__name__)


class AgentWorker(QThread):
    step_signal = pyqtSignal(dict)
    error_signal = pyqtSignal(str)
    complete_signal = pyqtSignal()
    screenshot_signal = pyqtSignal(str)
    status_signal = pyqtSignal(str)

    def __init__(self, agent: EyeForgeAgent, task: str, is_continuation: bool = False):
        super().__init__()
        self.agent = agent
        self.task = task
        self.is_continuation = is_continuation

        class _Callback(StepCallback):
            def __init__(self, worker):
                self.worker = worker
            def on_step(self, data):
                self.worker.step_signal.emit(data)
            def on_error(self, err):
                self.worker.error_signal.emit(err)
            def on_complete(self):
                self.worker.complete_signal.emit()
            def on_screenshot(self, b64):
                self.worker.screenshot_signal.emit(b64)
            def on_status(self, msg):
                self.worker.status_signal.emit(msg)

        self.agent.callback = _Callback(self)

    def run(self):
        if self.is_continuation:
            self.agent.continue_with(self.task)
        else:
            self.agent.run(self.task)


class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.config = self._load_config()
        self.agent: Optional[EyeForgeAgent] = None
        self.worker: Optional[AgentWorker] = None
        self.overlay = ClickOverlay()

        icon_path = os.path.join(os.path.dirname(__file__), "..", "..", "src", "logo.ico")
        if os.path.exists(icon_path):
            from PyQt5.QtGui import QIcon
            self.setWindowIcon(QIcon(icon_path))

        self._init_ui()
        self._init_tray()
        self._update_size()

    def _load_config(self) -> dict:
        default = {
            "llm_provider": "openai",
            "openai_api_key": "",
            "openai_model": "gpt-4o",
            "anthropic_api_key": "",
            "anthropic_model": "claude-3-5-sonnet-20241022",
            "ollama_base_url": "http://localhost:11434",
            "ollama_model": "llava",
            "custom_api_key": "",
            "custom_base_url": "https://api.openai.com/v1",
            "custom_model": "gpt-4o",
            "gemini_api_key": "",
            "gemini_model": "gemini-2.5-flash",
            "screenshot_quality": 95,
            "action_delay": 0.5,
            "language": "zh",
            "theme": "dark",
            "font_size": 9,
            "wizard_done": True,
        }
        if os.path.exists("config.json"):
            try:
                with open("config.json", "r", encoding="utf-8") as f:
                    loaded = json.load(f)
                    default.update(loaded)
            except Exception as e:
                logger.error(f"Failed to load config: {e}")
        return default

    def _update_size(self):
        screen = QApplication.primaryScreen()
        if screen:
            g = screen.geometry()
            self.overlay.resize_to_screen(g.x(), g.y(), g.width(), g.height())

    def _init_ui(self):
        self.setWindowTitle(f"EyeForge v{VERSION} - AI 屏幕操控助手")
        self.setMinimumSize(900, 650)
        self.resize(1000, 720)

        central = QWidget()
        self.setCentralWidget(central)
        main_layout = QVBoxLayout(central)

        self.header_label = QLabel(f"🧿 EyeForge v{VERSION} — AI 屏幕操控助手")
        self.header_label.setStyleSheet("font-size: 18px; font-weight: bold; padding: 8px; color: #00d4aa;")
        main_layout.addWidget(self.header_label)

        splitter = QSplitter(Qt.Horizontal)

        left_panel = QWidget()
        left_layout = QVBoxLayout(left_panel)
        left_layout.setContentsMargins(0, 0, 0, 0)

        self.preview = ScreenPreview()
        lang = self.config.get("language", "zh")
        self.preview.set_placeholder("Waiting for screenshot..." if lang == "en" else "等待截图...")
        left_layout.addWidget(self.preview)

        control_frame = QFrame()
        control_frame.setFrameStyle(QFrame.StyledPanel)
        control_layout = QVBoxLayout(control_frame)

        task_row = QHBoxLayout()
        self.task_label = QLabel("任务:")
        self.task_label.setStyleSheet("font-weight: bold;")
        self.task_input = QLineEdit()
        self.task_input.setPlaceholderText("在此输入你想让 AI 执行的任务...")
        self.task_input.returnPressed.connect(self._start_task)
        task_row.addWidget(self.task_label)
        task_row.addWidget(self.task_input)
        control_layout.addLayout(task_row)

        btn_row = QHBoxLayout()
        self.start_btn = QPushButton("▶ 开始执行")
        self.start_btn.setStyleSheet(
            "QPushButton { background-color: #00d4aa; color: black; font-weight: bold; padding: 8px 16px; }"
            "QPushButton:hover { background-color: #00b894; }"
        )
        self.settings_btn = QPushButton("⚙ 设置")
        self.settings_btn.setStyleSheet("padding: 8px 16px; border: 1px solid #666; border-radius: 4px;")

        self.start_btn.clicked.connect(self._toggle_task)
        self.settings_btn.clicked.connect(self._open_settings)

        btn_row.addWidget(self.start_btn)
        btn_row.addWidget(self.settings_btn)
        btn_row.addStretch()
        control_layout.addLayout(btn_row)

        left_layout.addWidget(control_frame)

        right_panel = QWidget()
        right_layout = QVBoxLayout(right_panel)
        right_layout.setContentsMargins(0, 0, 0, 0)

        self.log_label = QLabel("执行日志")
        self.log_label.setStyleSheet("font-weight: bold; font-size: 14px; padding: 4px;")
        right_layout.addWidget(self.log_label)

        self.log_output = QTextEdit()
        self.log_output.setReadOnly(True)
        self.log_output.setStyleSheet(
            "background-color: #1e1e1e; color: #d4d4d4; font-family: 'Consolas', 'Courier New'; font-size: 12px;"
        )
        right_layout.addWidget(self.log_output)

        splitter.addWidget(left_panel)
        splitter.addWidget(right_panel)
        splitter.setSizes([400, 600])
        main_layout.addWidget(splitter)

        status_bar = self.statusBar()
        self.status_label = QLabel("就绪")
        status_bar.addPermanentWidget(self.status_label)

        theme = self.config.get("theme", "dark")
        font_size = self.config.get("font_size", 9)
        lang = self.config.get("language", "zh")
        self._apply_theme(theme)
        self._apply_font(font_size)
        self._retranslate_ui(lang)

    def _apply_theme(self, theme: str):
        self._current_theme = theme
        if theme == "light":
            bg_main = "#f5f5f5"
            bg_log = "#fff"
            fg = "#333"
            border = "#ccc"
            input_bg = "#fff"
            input_fg = "#333"
            btn_bg = "#e0e0e0"
            btn_fg = "#333"
            btn_hover = "#d0d0d0"
            tab_bg = "#e8e8e8"
            preview_bg = "#eee"
            preview_border = "#ccc"
            status_bg = "#e0e0e0"
            log_font = "'Consolas', 'Courier New'"
        else:
            bg_main = "#2d2d2d"
            bg_log = "#252525"
            fg = "#d4d4d4"
            border = "#555"
            input_bg = "#3c3c3c"
            input_fg = "#d4d4d4"
            btn_bg = "#3c3c3c"
            btn_fg = "#d4d4d4"
            btn_hover = "#4a4a4a"
            tab_bg = "#383838"
            preview_bg = "#1a1a1a"
            preview_border = "#333"
            status_bg = "#252525"
            log_font = "'Consolas', 'Courier New'"

        ss = f"""
        QMainWindow {{ background-color: {bg_main}; }}
        QWidget {{ background-color: {bg_main}; color: {fg}; }}
        QLineEdit {{ background-color: {input_bg}; color: {input_fg}; border: 1px solid {border}; padding: 6px; border-radius: 4px; }}
        QTextEdit {{ background-color: {bg_log}; color: {fg}; border: 1px solid {border}; }}
        QPushButton {{ background-color: {btn_bg}; color: {btn_fg}; border: none; padding: 6px 12px; border-radius: 4px; }}
        QPushButton:hover {{ background-color: {btn_hover}; }}
        QLabel {{ background: transparent; }}
        QFrame {{ background: transparent; }}
        QStatusBar {{ background-color: {status_bg}; }}
        QTabWidget::pane {{ background-color: {tab_bg}; border: 1px solid {border}; }}
        QTabBar::tab {{ background-color: {btn_bg}; color: {btn_fg}; padding: 6px 16px; border: 1px solid {border}; border-bottom: none; border-radius: 4px 4px 0 0; }}
        QTabBar::tab:selected {{ background-color: {tab_bg}; }}
        """
        self.setStyleSheet(ss)

        self.log_output.setStyleSheet(
            f"background-color: {bg_log}; color: {fg}; font-family: {log_font}; font-size: 12px;"
        )
        self.preview.setStyleSheet(
            f"background-color: {preview_bg}; border: 1px solid {preview_border}; color: {fg};"
        )
        self.statusBar().setStyleSheet(
            f"background-color: {status_bg};"
        )
        self.status_label.setStyleSheet(
            f"color: {fg}; background: transparent; padding: 0 4px;"
        )
        self.header_label.setStyleSheet(
            "font-size: 18px; font-weight: bold; padding: 8px; color: #00d4aa;"
        )
        self.log_label.setStyleSheet(
            "font-weight: bold; font-size: 14px; padding: 4px;"
        )

    def _apply_font(self, size: int):
        font = QFont("Microsoft YaHei UI", size)
        self.setFont(font)

    def _retranslate_ui(self, lang: str):
        if lang == "en":
            self.setWindowTitle(f"EyeForge v{VERSION} - AI Screen Control Assistant")
            self.header_label.setText(f"🧿 EyeForge v{VERSION} — AI Screen Control Assistant")
            self.task_label.setText("Task:")
            self.task_input.setPlaceholderText("Enter a task for AI to execute...")
            self.start_btn.setText("▶ Start")
            self.settings_btn.setText("⚙ Settings")
            self.log_label.setText("Execution Log")
            self.status_label.setText("Ready")
            self.preview.set_placeholder("Waiting for screenshot...")
        else:
            self.setWindowTitle(f"EyeForge v{VERSION} - AI 屏幕操控助手")
            self.header_label.setText(f"🧿 EyeForge v{VERSION} — AI 屏幕操控助手")
            self.task_label.setText("任务:")
            self.task_input.setPlaceholderText("在此输入你想让 AI 执行的任务...")
            self.start_btn.setText("▶ 开始执行")
            self.settings_btn.setText("⚙ 设置")
            self.log_label.setText("执行日志")
            self.status_label.setText("就绪")
            self.preview.set_placeholder("等待截图...")
        if hasattr(self, "tray"):
            self._update_tray_menu()

    def _init_tray(self):
        self.tray = QSystemTrayIcon(self)
        self.tray.setToolTip(f"EyeForge v{VERSION}")
        self._update_tray_menu()
        self.tray.activated.connect(
            lambda reason: self.show() if reason == QSystemTrayIcon.DoubleClick else None
        )
        self.tray.show()

    def _update_tray_menu(self):
        lang = self.config.get("language", "zh")
        tray_menu = QMenu()
        show_action = QAction("Show Window" if lang == "en" else "显示窗口", self)
        show_action.triggered.connect(self.show)
        update_action = QAction("Check Update" if lang == "en" else "检查更新", self)
        update_action.triggered.connect(self._check_update_dialog)
        quit_action = QAction("Quit" if lang == "en" else "退出", self)
        quit_action.triggered.connect(QApplication.quit)
        tray_menu.addAction(show_action)
        tray_menu.addSeparator()
        tray_menu.addAction(update_action)
        tray_menu.addSeparator()
        tray_menu.addAction(quit_action)
        self.tray.setContextMenu(tray_menu)

    def _check_update_dialog(self):
        lang = self.config.get("language", "zh")
        from PyQt5.QtWidgets import QDialog, QVBoxLayout, QLabel, QPushButton, QDialogButtonBox
        from PyQt5.QtGui import QDesktopServices
        from PyQt5.QtCore import QUrl

        dialog = QDialog(self)
        dialog.setWindowTitle("Check Update" if lang == "en" else "检查更新")
        dialog.resize(420, 220)
        layout = QVBoxLayout(dialog)

        layout.addWidget(QLabel("⏳ Checking..." if lang == "en" else "⏳ 正在检查更新..."))
        dialog.show()
        QApplication.processEvents()

        result = check_update()
        for i in reversed(range(layout.count())):
            widget = layout.itemAt(i).widget()
            if widget:
                widget.deleteLater()

        current_ver = result["current"]
        latest_ver = result["latest"]
        has_update = result["update_available"]

        if latest_ver != "Unknown":
            status = (f'<b>{"Current" if lang == "en" else "当前版本"}:</b> v{current_ver}<br>'
                      f'<b>{"Latest" if lang == "en" else "最新版本"}:</b> v{latest_ver}<br>'
                      f'<b>{"Source" if lang == "en" else "来源"}:</b> {result["source"] or "GitHub"}')
            layout.addWidget(QLabel(status))
            layout.addWidget(QLabel(" "))
            if has_update:
                info = QLabel('<b style="color:#00d4aa;">✶ New version available!</b>' if lang == "en"
                              else '<b style="color:#00d4aa;">✶ 有新版本可用！</b>')
            else:
                info = QLabel("✓ Up to date" if lang == "en" else "✓ 已是最新版本")
            layout.addWidget(info)
            layout.addWidget(QLabel(" "))
            btn_row = QHBoxLayout()
            gh_btn = QPushButton("📦 GitHub Releases")
            gh_btn.clicked.connect(lambda: QDesktopServices.openUrl(QUrl(result["github_url"])))
            gc_btn = QPushButton("📦 GitCode Releases")
            gc_btn.clicked.connect(lambda: QDesktopServices.openUrl(QUrl(result["gitcode_url"])))
            btn_row.addWidget(gh_btn)
            btn_row.addWidget(gc_btn)
            layout.addLayout(btn_row)
        else:
            layout.addWidget(QLabel("✗ Check failed" if lang == "en" else "✗ 检查更新失败"))
            layout.addWidget(QLabel("Please check network or visit manually:" if lang == "en" else "请检查网络或手动访问："))
            btn_row = QHBoxLayout()
            gh_btn = QPushButton("📦 GitHub")
            gh_btn.clicked.connect(lambda: QDesktopServices.openUrl(QUrl(result["github_url"])))
            gc_btn = QPushButton("📦 GitCode")
            gc_btn.clicked.connect(lambda: QDesktopServices.openUrl(QUrl(result["gitcode_url"])))
            btn_row.addWidget(gh_btn)
            btn_row.addWidget(gc_btn)

        layout.addStretch()
        buttons = QDialogButtonBox(QDialogButtonBox.Ok)
        buttons.accepted.connect(dialog.accept)
        layout.addWidget(buttons)
        dialog.exec_()

    def _log(self, message: str, level: str = "info"):
        timestamp = datetime.now().strftime("%H:%M:%S")
        is_light = getattr(self, "_current_theme", "dark") == "light"
        info_color = "#333" if is_light else "#d4d4d4"
        ts_color = "#555" if is_light else "#888"
        colors = {"info": info_color, "action": "#00d4aa", "error": "#e17055", "thought": "#74b9ff"}
        color = colors.get(level, info_color)
        prefix = {"info": "ℹ", "action": "▸", "error": "✖", "thought": "💭"}.get(level, " ")
        safe = html.escape(message)
        html_s = f'<span style="color: {ts_color};">[{timestamp}]</span> <span style="color: {color};">{prefix} {safe}</span><br>'
        self.log_output.insertHtml(html_s)
        self.log_output.moveCursor(QTextCursor.End)
        logger.info(f"[{level}] {message}")

    def _toggle_task(self):
        if self.worker and self.worker.isRunning():
            self._pause_task()
        else:
            self._start_task()

    def _start_task(self):
        task = self.task_input.text().strip()
        lang = self.config.get("language", "zh")
        if not task:
            title = "Notice" if lang == "en" else "提示"
            msg = "Please enter a task description" if lang == "en" else "请输入任务描述"
            QMessageBox.warning(self, title, msg)
            return

        if self.agent is None or not self.agent._has_session:
            self.agent = EyeForgeAgent(self.config)
        else:
            self.agent.config = self.config

        if not self.agent.llm.is_available():
            title = "Notice" if lang == "en" else "提示"
            msg = "LLM not configured. Please set up API Key in Settings." if lang == "en" else "LLM 未配置，请先在设置中填写 API Key"
            QMessageBox.warning(self, title, msg)
            return

        prov = self.config.get("llm_provider", "")
        model_key = {"openai": "openai_model", "anthropic": "anthropic_model",
                     "ollama": "ollama_model", "gemini": "gemini_model", "custom": "custom_model"}
        model_name = self.config.get(model_key.get(prov, ""), "")
        from src.utils.multimodal import is_multimodal
        if model_name and not is_multimodal(model_name):
            title = "Warning" if lang == "en" else "警告"
            msg = (f'The model "{model_name}" may not support vision.\n'
                   f'EyeForge requires a multimodal model.\n\nContinue anyway?') if lang == "en" else (
                f'模型 "{model_name}" 可能不支持视觉识别。\n'
                f'EyeForge 需要多模态模型。\n\n是否继续？')
            ret = QMessageBox.warning(self, title, msg,
                                      QMessageBox.Yes | QMessageBox.No,
                                      QMessageBox.No)
            if ret == QMessageBox.No:
                return

        self.start_btn.setText("⏸ Pause" if lang == "en" else "⏸ 暂停执行")
        self.start_btn.setStyleSheet(
            "QPushButton { background-color: #fdcb6e; color: black; font-weight: bold; padding: 8px 16px; }"
            "QPushButton:hover { background-color: #e0a800; }"
        )
        self.task_input.setEnabled(False)
        if not self.agent._has_session:
            self.log_output.clear()
        self.status_label.setText("⏳ Executing..." if lang == "en" else "⏳ 执行中...")
        log_prefix = "Task" if lang == "en" else "任务"
        self._log(f"{log_prefix}: {task}", "info")

        self.worker = AgentWorker(self.agent, task, is_continuation=self.agent._has_session)
        self.worker.step_signal.connect(self._on_step)
        self.worker.error_signal.connect(self._on_error)
        self.worker.complete_signal.connect(self._on_complete)
        self.worker.screenshot_signal.connect(self._on_screenshot)
        self.worker.status_signal.connect(self._on_status)
        self.worker.finished.connect(self._on_worker_finished)
        self.worker.start()

    def _pause_task(self):
        if self.agent:
            self.agent.stop()
        lang = self.config.get("language", "zh")
        msg = "⏸ Paused" if lang == "en" else "⏸ 已暂停"
        self._log(msg, "error")
        self._reset_controls()

    def _on_step(self, data: dict):
        step = data.get("step", 0)
        thought = data.get("thought", "")
        action = data.get("action", {})
        action_type = action.get("type", "unknown")
        lang = self.config.get("language", "zh")

        if thought:
            self._log(f"[Step {step}] {thought}", "thought")
        action_label = "Action" if lang == "en" else "动作"
        self._log(f"{action_label}: {action_type} {json.dumps(action)}", "action")

        if action_type == "click_ratio":
            sw, sh = self.agent.screen.get_screen_size() if self.agent else (1920, 1080)
            x = int(action.get("x_ratio", 0.5) * sw)
            y = int(action.get("y_ratio", 0.5) * sh)
            self.overlay.show_click(x, y)
            self.preview.add_marker(x, y, f"Step {step}")
        elif action_type in ("click", "double_click", "right_click"):
            x, y = action.get("x", 0), action.get("y", 0)
            self.overlay.show_click(x, y)
            self.preview.add_marker(x, y, f"Step {step}")

    def _on_error(self, error: str):
        lang = self.config.get("language", "zh")
        prefix = "Error" if lang == "en" else "错误"
        self._log(f"{prefix}: {error}", "error")
        self.status_label.setText(f"✖ {prefix}: {error[:50]}")

    def _on_complete(self):
        self._log("✅ Task complete!" if self.config.get("language", "en") == "en" else "✅ 任务完成!", "info")
        self._reset_controls()

    def _on_status(self, message: str):
        self._log(message, "info")
        self.status_label.setText(message)

    def _on_screenshot(self, image_b64: str):
        from PyQt5.QtGui import QPixmap
        from PyQt5.QtCore import QByteArray, QBuffer
        import base64
        img_data = base64.b64decode(image_b64)
        pixmap = QPixmap()
        pixmap.loadFromData(img_data, "JPEG")

        debug_path = "logs/last_screenshot.jpg"
        try:
            with open(debug_path, "wb") as f:
                f.write(img_data)
        except Exception:
            pass

        w = pixmap.width()
        h = pixmap.height()
        lang = self.config.get("language", "zh")
        size_label = "Screenshot size" if lang == "en" else "截图尺寸"
        quality_label = "quality" if lang == "en" else "质量"
        self._log(f"{size_label}: {w}x{h}, {quality_label}: {self.config.get('screenshot_quality', 95)}", "info")

        self.preview.update_preview(pixmap)

    def _on_worker_finished(self):
        self._reset_controls()

    def _reset_controls(self):
        lang = self.config.get("language", "zh")
        self.start_btn.setText("▶ Start" if lang == "en" else "▶ 开始执行")
        self.start_btn.setStyleSheet(
            "QPushButton { background-color: #00d4aa; color: black; font-weight: bold; padding: 8px 16px; }"
            "QPushButton:hover { background-color: #00b894; }"
        )
        self.task_input.setEnabled(True)
        self.status_label.setText("Ready" if lang == "en" else "就绪")

    def _open_settings(self):
        old_lang = self.config.get("language")
        old_theme = self.config.get("theme")
        dialog = SettingsDialog(self.config, self)
        if dialog.exec_():
            self.config = dialog.get_config()
            new_lang = self.config.get("language")
            new_theme = self.config.get("theme")
            if old_lang != new_lang:
                self.agent = None
                msg = "Language changed, session will restart" if new_lang == "en" else "语言已更改，会话将重新开始"
                self._log(msg, "info")
            if old_theme != new_theme:
                self._apply_theme(new_theme)
            self._apply_font(self.config.get("font_size", 9))
            self._retranslate_ui(new_lang)

    def _reset_session(self):
        self.agent = None
        self._reset_controls()

    def closeEvent(self, event):
        if self.agent:
            self.agent.stop()
        event.accept()
