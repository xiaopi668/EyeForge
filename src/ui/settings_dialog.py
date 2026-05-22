import json
import logging
import io
import threading
from urllib.request import urlopen
from urllib.parse import quote
from PyQt5.QtWidgets import (
    QApplication, QDialog, QVBoxLayout, QHBoxLayout, QFormLayout,
    QLineEdit, QComboBox, QSpinBox, QDoubleSpinBox,
    QPushButton, QLabel, QTabWidget, QWidget, QMessageBox,
    QListWidget, QStackedWidget, QGroupBox, QFrame
)
from PyQt5.QtCore import Qt, QTimer
from src.utils.multimodal import is_multimodal
from src.utils.crypto import encrypt
from src.version import VERSION
from src.utils.updater import check_update

logger = logging.getLogger(__name__)


class SettingsDialog(QDialog):
    def __init__(self, config: dict, parent=None):
        super().__init__(parent)
        self.config = config.copy()
        self.lang = self.config.get("language", "zh")
        self.setWindowTitle("设置 - Settings")
        self.setMinimumWidth(480)
        self.setMinimumHeight(400)
        self.move(-10000, -10000)
        self._init_ui()
        QTimer.singleShot(0, self._center_on_parent)

    def _center_on_parent(self):
        if self.parent() and self.parent().isVisible():
            center = self.parent().geometry().center()
        else:
            center = QApplication.primaryScreen().geometry().center()
        rect = self.geometry()
        self.move(center.x() - rect.width() // 2, center.y() - rect.height() // 2)

    def _tr(self, zh: str, en: str) -> str:
        return en if self.lang == "en" else zh

    def _init_ui(self):
        layout = QVBoxLayout(self)
        tabs = QTabWidget()

        tabs.addTab(self._llm_tab(), self._tr("AI 模型", "AI Model"))
        tabs.addTab(self._capture_tab(), self._tr("截屏设置", "Capture"))
        self._general_tab_widget, general_layout = QWidget(), QFormLayout()
        self._lang_combo = QComboBox()
        self._lang_combo.addItems(["中文 (zh)", "English (en)"])
        self._lang_combo.setCurrentText(
            "中文 (zh)" if self.lang == "zh" else "English (en)"
        )
        general_layout.addRow(self._tr("语言 / Language:", "Language / 语言:"), self._lang_combo)
        self._theme_combo = QComboBox()
        self._theme_combo.addItems(["dark", "light"])
        self._theme_combo.setCurrentText(self.config.get("theme", "dark"))
        general_layout.addRow(self._tr("主题 / Theme:", "Theme / 主题:"), self._theme_combo)
        self._font_spin = QSpinBox()
        self._font_spin.setRange(8, 14)
        self._font_spin.setValue(self.config.get("font_size", 9))
        general_layout.addRow(self._tr("字体大小 / Font Size:", "Font Size / 字体大小:"), self._font_spin)
        self._hotkey_float = QLineEdit(self.config.get("hotkey_float", "ctrl+shift+e"))
        general_layout.addRow(self._tr("快捷输入 / Quick Input:", "Quick Input / 快捷输入:"), self._hotkey_float)
        self._hotkey_voice = QLineEdit(self.config.get("hotkey_voice", "ctrl+shift+v"))
        general_layout.addRow(self._tr("语音输入 / Voice Input:", "Voice Input / 语音输入:"), self._hotkey_voice)
        self._wakeword_check = QComboBox()
        self._wakeword_check.addItems(["关闭 / Off", "开启 / On"])
        self._wakeword_check.setCurrentText("开启 / On" if self.config.get("wakeword_enabled", False) else "关闭 / Off")
        general_layout.addRow(self._tr("语音唤醒 / Wake Word:", "Wake Word / 语音唤醒:"), self._wakeword_check)
        self._wakeword_list = QLineEdit(self.config.get("wakeword_list", "computer"))
        general_layout.addRow(self._tr("唤醒词 / Keywords:", "Keywords / 唤醒词:"), self._wakeword_list)
        self._porcupine_key = QLineEdit(self.config.get("porcupine_access_key", ""))
        self._porcupine_key.setEchoMode(QLineEdit.Password)
        general_layout.addRow(self._tr("Picovoice AccessKey / 密钥:", "Picovoice AccessKey / 密钥:"), self._porcupine_key)
        self._general_tab_widget.setLayout(general_layout)
        self._wakeword_check.currentIndexChanged.connect(self._toggle_wakeword_rows)
        self._toggle_wakeword_rows()
        tabs.addTab(self._general_tab_widget, self._tr("常规", "General"))

        self._channels_tab = self._channels_tab()
        tabs.addTab(self._channels_tab, self._tr("通道", "Channels"))

        self._update_tab = self._update_tab()
        tabs.addTab(self._update_tab, self._tr("更新", "Update"))

        layout.addWidget(tabs)

        btn_layout = QHBoxLayout()
        save_btn = QPushButton(self._tr("保存", "Save"))
        cancel_btn = QPushButton(self._tr("取消", "Cancel"))
        save_btn.clicked.connect(self._save)
        cancel_btn.clicked.connect(self.reject)
        btn_layout.addStretch()
        btn_layout.addWidget(save_btn)
        btn_layout.addWidget(cancel_btn)
        layout.addLayout(btn_layout)

    def _toggle_wakeword_rows(self):
        visible = "开启" in self._wakeword_check.currentText()
        self._wakeword_list.setVisible(visible)
        self._porcupine_key.setVisible(visible)
        layout = self._general_tab_widget.layout()
        if isinstance(layout, QFormLayout):
            lbl1 = layout.labelForField(self._wakeword_list)
            lbl2 = layout.labelForField(self._porcupine_key)
            if lbl1:
                lbl1.setVisible(visible)
            if lbl2:
                lbl2.setVisible(visible)

    @staticmethod
    def _check_multimodal(model: str) -> str:
        return "🟢" if is_multimodal(model) else "⚪"

    def _build_provider_section(self, fields: list, pull_provider: str, model_field: QLineEdit) -> QWidget:
        group = QWidget()
        layout = QVBoxLayout(group)
        layout.setContentsMargins(0, 8, 0, 0)
        form = QFormLayout()
        for field_label, field_widget in fields:
            form.addRow(field_label, field_widget)
        status_label = QLabel("")
        form.addRow("", status_label)
        model_field.textChanged.connect(
            lambda t, lbl=status_label: lbl.setText(
                f"🟢 {self._tr('多模态', 'Multimodal')}"
                if self._check_multimodal(t) == "🟢"
                else f"⚪ {self._tr('未知', 'Unknown')}"
            )
        )
        layout.addLayout(form)
        pull_row = QHBoxLayout()
        pull_status = QLabel("")
        pull_btn = QPushButton(self._tr("📥 拉取模型", "📥 Fetch Models"))
        pull_btn.setStyleSheet(
            "QPushButton { background-color: #6c5ce7; color: white; font-weight: bold; padding: 4px 12px; }"
            "QPushButton:hover { background-color: #5a4bd1; }"
            "QPushButton:disabled { background-color: #555; }"
        )
        pull_btn.clicked.connect(lambda: self._fetch_models(pull_provider, pull_btn, pull_status, model_field))
        pull_row.addWidget(pull_btn)
        pull_row.addWidget(pull_status)
        pull_row.addStretch()
        layout.addLayout(pull_row)
        return group

    def _llm_tab(self):
        widget = QWidget()
        layout = QVBoxLayout(widget)

        provider_row = QHBoxLayout()
        provider_row.addWidget(QLabel(self._tr("提供商:", "Provider:")))
        self._provider_combo = QComboBox()
        provider_names = [
            ("openai", self._tr("OpenAI", "OpenAI")),
            ("anthropic", self._tr("Anthropic", "Anthropic")),
            ("ollama", self._tr("Ollama", "Ollama")),
            ("gemini", self._tr("Gemini", "Gemini")),
            ("custom", self._tr("自定义", "Custom")),
        ]
        for val, label in provider_names:
            self._provider_combo.addItem(label, val)
        self._provider_combo.setCurrentIndex(
            max(0, next((i for i, (v, _) in enumerate(provider_names) if v == self.config.get("llm_provider", "openai")), 0))
        )
        self._provider_combo.currentIndexChanged.connect(lambda: self._on_provider_change(self._provider_combo.currentData()))
        provider_row.addWidget(self._provider_combo)
        provider_row.addSpacing(16)
        self._test_btn = QPushButton(self._tr("测试连接", "Test Connection"))
        self._test_btn.clicked.connect(self._test_connection)
        self._test_btn.setStyleSheet(
            "QPushButton { background-color: #00b894; color: white; font-weight: bold; padding: 4px 12px; }"
            "QPushButton:hover { background-color: #00a381; }"
            "QPushButton:disabled { background-color: #555; }"
        )
        provider_row.addWidget(self._test_btn)
        self._test_status = QLabel("")
        provider_row.addWidget(self._test_status)
        provider_row.addStretch()
        layout.addLayout(provider_row)

        self._openai_key = QLineEdit(self.config.get("openai_api_key", ""))
        self._openai_key.setEchoMode(QLineEdit.Password)
        self._openai_model = QLineEdit(self.config.get("openai_model", "gpt-4o"))
        self._openai_group = self._build_provider_section([
            (self._tr("API 密钥:", "API Key:"), self._openai_key),
            (self._tr("模型:", "Model:"), self._openai_model),
        ], "openai", self._openai_model)
        layout.addWidget(self._openai_group)

        self._anthropic_key = QLineEdit(self.config.get("anthropic_api_key", ""))
        self._anthropic_key.setEchoMode(QLineEdit.Password)
        self._anthropic_model = QLineEdit(self.config.get("anthropic_model", "claude-3-5-sonnet-20241022"))
        self._anthropic_group = self._build_provider_section([
            (self._tr("API 密钥:", "API Key:"), self._anthropic_key),
            (self._tr("模型:", "Model:"), self._anthropic_model),
        ], "anthropic", self._anthropic_model)
        layout.addWidget(self._anthropic_group)

        self._ollama_url = QLineEdit(self.config.get("ollama_base_url", "http://localhost:11434"))
        self._ollama_model = QLineEdit(self.config.get("ollama_model", "llava"))
        self._ollama_group = self._build_provider_section([
            (self._tr("地址:", "URL:"), self._ollama_url),
            (self._tr("模型:", "Model:"), self._ollama_model),
        ], "ollama", self._ollama_model)
        layout.addWidget(self._ollama_group)

        self._custom_key = QLineEdit(self.config.get("custom_api_key", ""))
        self._custom_key.setEchoMode(QLineEdit.Password)
        self._custom_url = QLineEdit(self.config.get("custom_base_url", "https://api.openai.com/v1"))
        self._custom_model = QLineEdit(self.config.get("custom_model", "gpt-4o"))
        self._custom_group = self._build_provider_section([
            (self._tr("API 密钥:", "API Key:"), self._custom_key),
            (self._tr("基础地址:", "Base URL:"), self._custom_url),
            (self._tr("模型:", "Model:"), self._custom_model),
        ], "custom", self._custom_model)
        layout.addWidget(self._custom_group)

        self._gemini_key = QLineEdit(self.config.get("gemini_api_key", ""))
        self._gemini_key.setEchoMode(QLineEdit.Password)
        self._gemini_model = QLineEdit(self.config.get("gemini_model", "gemini-2.5-flash"))
        self._gemini_group = self._build_provider_section([
            (self._tr("API 密钥:", "API Key:"), self._gemini_key),
            (self._tr("模型:", "Model:"), self._gemini_model),
        ], "gemini", self._gemini_model)
        layout.addWidget(self._gemini_group)

        layout.addStretch()
        self._on_provider_change(self.config.get("llm_provider", "openai"))
        return widget

    def _on_provider_change(self, provider: str):
        self._openai_group.setVisible(provider == "openai")
        self._anthropic_group.setVisible(provider == "anthropic")
        self._ollama_group.setVisible(provider == "ollama")
        self._custom_group.setVisible(provider == "custom")
        self._gemini_group.setVisible(provider == "gemini")

    def _build_test_config(self) -> dict:
        return {
            "llm_provider": self._provider_combo.currentData(),
            "openai_api_key": self._openai_key.text(),
            "openai_model": self._openai_model.text(),
            "anthropic_api_key": self._anthropic_key.text(),
            "anthropic_model": self._anthropic_model.text(),
            "ollama_base_url": self._ollama_url.text(),
            "ollama_model": self._ollama_model.text(),
            "custom_api_key": self._custom_key.text(),
            "custom_base_url": self._custom_url.text(),
            "custom_model": self._custom_model.text(),
            "gemini_api_key": self._gemini_key.text(),
            "gemini_model": self._gemini_model.text(),
        }

    def _test_connection(self):
        from src.ai.llm_client import LLMClient

        test_config = self._build_test_config()
        client = LLMClient(
            provider=test_config["llm_provider"],
            config=test_config,
        )
        if not client.is_available():
            QMessageBox.warning(self, self._tr("测试失败", "Test Failed"), self._tr("请先填写必要的 API Key", "Please fill in the required API Key first"))
            return

        self._test_btn.setEnabled(False)
        self._test_btn.setText(self._tr("测试中...", "Testing..."))
        self._test_status.setText("")
        self._test_status.setStyleSheet("color: #fdcb6e;")
        testing_msg = self._tr("正在测试", "Testing")
        QMessageBox.information(self, self._tr("测试", "Test"), f"{testing_msg} {test_config['llm_provider']} {self._tr('连接...', 'connection...')}\n{self._tr('模型:', 'Model:')} {client.get_model_name()}")
        QApplication.processEvents()

        try:
            response = client.chat(
                messages=[{"role": "user", "content": "回复'Hello! Connection successful.' 不加任何其他内容"}],
            )
            if response:
                self._test_status.setText(self._tr("✓ 连接成功", "✓ Connected"))
                self._test_status.setStyleSheet("color: #00b894; font-weight: bold;")
                QMessageBox.information(
                    self, self._tr("测试成功", "Success"),
                    f"{self._tr('连接成功！', 'Connected!')}\n\n{self._tr('模型响应:', 'Response:')}\n{response.strip()}"
                )
                logger.info(f"Connection test successful: {response.strip()}")
            else:
                self._test_status.setText(self._tr("✗ 无响应", "✗ No Response"))
                self._test_status.setStyleSheet("color: #e17055; font-weight: bold;")
                QMessageBox.warning(self, self._tr("测试失败", "Test Failed"), self._tr("API 返回空响应，请检查配置", "API returned empty response, please check your configuration"))
        except Exception as e:
            self._test_status.setText(self._tr("✗ 连接失败", "✗ Connection Failed"))
            self._test_status.setStyleSheet("color: #e17055; font-weight: bold;")
            QMessageBox.critical(self, self._tr("测试失败", "Test Failed"), f"{self._tr('连接测试失败:', 'Connection test failed:')}\n{e}")
            logger.error(f"Connection test failed: {e}")
        finally:
            self._test_btn.setEnabled(True)
            self._test_btn.setText(self._tr("测试连接", "Test Connection"))

    def _fetch_models(self, provider: str, btn: QPushButton, status: QLabel, model_field: QLineEdit):
        import requests
        config = self._build_test_config()
        btn.setEnabled(False)
        btn.setText(self._tr("获取中...", "Fetching..."))
        status.setText("")
        QApplication.processEvents()

        models = []
        try:
            if provider == "ollama":
                base_url = config.get("ollama_base_url", "").strip()
                if not base_url:
                    raise ValueError(self._tr("请先填写 Ollama URL", "Please enter Ollama URL first"))
                resp = requests.get(f"{base_url.rstrip('/')}/api/tags", timeout=10)
                resp.raise_for_status()
                raw = resp.json().get("models", [])
                models = [m["name"] for m in raw]

            elif provider == "openai":
                api_key = config.get("openai_api_key", "").strip()
                if not api_key:
                    raise ValueError(self._tr("请先填写 API Key", "Please enter API Key first"))
                resp = requests.get(
                    "https://api.openai.com/v1/models",
                    headers={"Authorization": f"Bearer {api_key}"},
                    timeout=15,
                )
                resp.raise_for_status()
                raw = resp.json().get("data", [])
                models = [m["id"] for m in raw]

            elif provider == "anthropic":
                api_key = config.get("anthropic_api_key", "").strip()
                if not api_key:
                    raise ValueError(self._tr("请先填写 API Key", "Please enter API Key first"))
                resp = requests.get(
                    "https://api.anthropic.com/v1/models",
                    headers={"x-api-key": api_key, "anthropic-version": "2023-06-01"},
                    timeout=15,
                )
                resp.raise_for_status()
                raw = resp.json().get("data", [])
                models = [m["id"] or m.get("name", "") for m in raw]

            elif provider == "gemini":
                api_key = config.get("gemini_api_key", "").strip()
                if not api_key:
                    raise ValueError(self._tr("请先填写 API Key", "Please enter API Key first"))
                resp = requests.get(
                    "https://generativelanguage.googleapis.com/v1beta/models?key=" + api_key,
                    timeout=15,
                )
                resp.raise_for_status()
                raw = resp.json().get("models", [])
                models = [m["name"].replace("models/", "") for m in raw if "vision" in m.get("supportedGenerationMethods", []) or True]

            elif provider == "custom":
                api_key = config.get("custom_api_key", "").strip()
                base_url = config.get("custom_base_url", "").strip()
                if not api_key or not base_url:
                    raise ValueError(self._tr("请先填写 API Key 和 Base URL", "Please enter API Key and Base URL"))
                resp = requests.get(
                    f"{base_url.rstrip('/')}/models",
                    headers={"Authorization": f"Bearer {api_key}"},
                    timeout=15,
                )
                resp.raise_for_status()
                raw = resp.json().get("data", [])
                models = [m["id"] for m in raw]

            if not models:
                status.setText(self._tr("✗ 无可用模型", "✗ No models available"))
                status.setStyleSheet("color: #e17055; font-weight: bold;")
                QMessageBox.information(self, self._tr("提示", "Notice"), self._tr("未获取到任何模型", "No models found"))
                return

            annotated = [f"{m}  {self._check_multimodal(m)}" for m in models]

            from PyQt5.QtWidgets import QDialog, QVBoxLayout, QListWidget, QDialogButtonBox
            dialog = QDialog(self)
            dialog.setWindowTitle(f"{self._tr('选择模型', 'Select Model')} - {provider}")
            dialog.resize(450, 500)
            dl = QVBoxLayout(dialog)
            search = QLineEdit()
            search.setPlaceholderText(self._tr("搜索模型...", "Search models..."))
            dl.addWidget(search)
            lst = QListWidget()
            lst.addItems(annotated)
            dl.addWidget(lst)
            buttons = QDialogButtonBox(QDialogButtonBox.Ok | QDialogButtonBox.Cancel)
            buttons.accepted.connect(dialog.accept)
            buttons.rejected.connect(dialog.reject)
            dl.addWidget(buttons)

            search.textChanged.connect(lambda t: _filter(lst, t))

            def _filter(l, t):
                for i in range(l.count()):
                    item = l.item(i)
                    item.setHidden(t.lower() not in item.text().lower())

            if dialog.exec_() and lst.currentItem():
                selected = lst.currentItem().text().split("  ")[0].strip()
                model_field.setText(selected)
                status.setText(self._tr("✓ 已选", "✓ Selected") + f" {selected}")
                status.setStyleSheet("color: #00b894; font-weight: bold;")
            else:
                status.setText("")

        except ValueError as e:
            status.setText(self._tr("✗ 参数不足", "✗ Missing parameters"))
            status.setStyleSheet("color: #e17055; font-weight: bold;")
            QMessageBox.warning(self, self._tr("提示", "Notice"), str(e))
        except requests.RequestException as e:
            status.setText(self._tr("✗ 获取失败", "✗ Fetch Failed"))
            status.setStyleSheet("color: #e17055; font-weight: bold;")
            QMessageBox.critical(self, self._tr("失败", "Failed"), f"{self._tr('获取模型列表失败:', 'Failed to fetch model list:')}\n{e}")
        finally:
            btn.setEnabled(True)
            btn.setText(self._tr("📥 拉取模型", "📥 Fetch Models"))

    def _capture_tab(self):
        widget = QWidget()
        layout = QFormLayout(widget)

        self._quality_spin = QSpinBox()
        self._quality_spin.setRange(10, 100)
        self._quality_spin.setValue(self.config.get("screenshot_quality", 95))
        layout.addRow(self._tr("截图质量 (1-100):", "Quality (1-100):"), self._quality_spin)

        self._delay_spin = QDoubleSpinBox()
        self._delay_spin.setRange(0.0, 5.0)
        self._delay_spin.setSingleStep(0.1)
        self._delay_spin.setValue(self.config.get("action_delay", 0.5))
        layout.addRow(self._tr("操作延迟 (秒):", "Action Delay (s):"), self._delay_spin)

        return widget

    def _channels_tab(self):
        widget = QWidget()
        layout = QVBoxLayout(widget)

        self._channel_list = QListWidget()
        self._channel_list.setMaximumWidth(160)
        self._channel_list.setMinimumWidth(120)

        self._channel_stack = QStackedWidget()
        self._channel_pages = {}

        channels = [
            ("WebSocket", self._tr("通用 WebSocket", "Generic WebSocket")),
            (self._tr("微信 / WeChat", "WeChat / 微信"), self._tr("原生 iLink API 客户端", "Native iLink API client")),
            (self._tr("企业微信 / WeCom", "WeCom / 企业微信"), self._tr("企业微信回调服务", "WeCom callback server")),
            (self._tr("钉钉 / DingTalk", "DingTalk / 钉钉"), self._tr("钉钉开放平台机器人", "DingTalk Open Platform bot")),
            (self._tr("QQ", "QQ"), self._tr("go-cqhttp / QQ 官方机器人", "go-cqhttp / QQ Official Bot")),
        ]

        makers = [self._make_ws_page, self._make_wc_page, self._make_wcom_page,
                  self._make_dingtalk_page, self._make_qq_page]

        placeholder = QWidget()
        self._channel_stack.addWidget(placeholder)
        for i, (name, tip) in enumerate(channels):
            item = self._channel_list.addItem(name)
            self._channel_list.item(i).setToolTip(tip)
            self._channel_pages[i] = None

        def on_channel_change(idx):
            if idx < 0 or idx >= len(makers):
                return
            if self._channel_pages.get(idx) is None:
                page = makers[idx]()
                self._channel_pages[idx] = page
                self._channel_stack.addWidget(page)
            self._channel_stack.setCurrentWidget(self._channel_pages[idx])

        self._channel_list.currentRowChanged.connect(on_channel_change)

        h_layout = QHBoxLayout()
        h_layout.addWidget(self._channel_list)
        h_layout.addWidget(self._channel_stack, 1)
        layout.addLayout(h_layout)
        layout.addStretch()

        self._channel_list.setCurrentRow(0)
        return widget

    def _make_ws_page(self):
        page = QWidget()
        form = QFormLayout(page)
        form.setSpacing(10)
        self._ws_enabled = QComboBox()
        self._ws_enabled.addItems(["关闭 / Off", "开启 / On"])
        self._ws_enabled.setCurrentText("开启 / On" if self.config.get("ws_enabled", False) else "关闭 / Off")
        form.addRow(self._tr("服务开关:", "Server:"), self._ws_enabled)
        self._ws_host = QLineEdit(self.config.get("ws_host", "0.0.0.0"))
        form.addRow(self._tr("监听地址:", "Host:"), self._ws_host)
        self._ws_port = QSpinBox()
        self._ws_port.setRange(1024, 65535)
        self._ws_port.setValue(self.config.get("ws_port", 8765))
        form.addRow(self._tr("监听端口:", "Port:"), self._ws_port)
        self._ws_token = QLineEdit(self.config.get("ws_token", ""))
        self._ws_token.setEchoMode(QLineEdit.Password)
        form.addRow(self._tr("认证令牌:", "Auth Token:"), self._ws_token)
        tip = QLabel(self._tr(
            "通用 WebSocket 接入，适用于无官方 API 的平台。\n"
            "任意客户端连接后发送 JSON 即可执行任务。",
            "Generic WebSocket endpoint for platforms without official APIs.\n"
            "Any client can connect and send JSON to execute tasks.",
        ))
        tip.setWordWrap(True)
        tip.setStyleSheet("color: #888; font-size: 11px;")
        form.addRow(tip)
        self._ws_status = QLabel(self._tr("状态: 未启动", "Status: Not running"))
        form.addRow(self._tr("状态:", "Status:"), self._ws_status)
        from src.utils import websocket_server as ws_mod
        if ws_mod.is_running():
            self._ws_status.setText(self._tr("状态: 运行中", "Status: Running"))
        return page

    def _make_wc_page(self):
        page = QWidget()
        form = QFormLayout(page)
        form.setSpacing(10)
        self._wc_enabled = QComboBox()
        self._wc_enabled.addItems(["关闭 / Off", "开启 / On"])
        self._wc_enabled.setCurrentText("开启 / On" if self.config.get("wc_enabled", False) else "关闭 / Off")
        form.addRow(self._tr("服务开关:", "Server:"), self._wc_enabled)
        self._wc_token = QLineEdit(self.config.get("wc_token", ""))
        self._wc_token.setEchoMode(QLineEdit.Password)
        form.addRow(self._tr("Bot Token:", "Bot Token:"), self._wc_token)

        self._wc_scan_btn = QPushButton(self._tr("📱 扫码登录微信", "📱 QR Login (WeChat)"))
        self._wc_scan_btn.clicked.connect(self._wc_qr_login)
        form.addRow(self._wc_scan_btn)

        tip = QLabel(self._tr(
            "原生接入微信 iLink Bot API，无需任何外部依赖。\n"
            "点击上方按钮扫码登录，Token 自动填入。",
            "Native WeChat iLink API client, no external dependencies.\n"
            "Click the button above to scan QR code and auto-fill token.",
        ))
        tip.setWordWrap(True)
        tip.setStyleSheet("color: #888; font-size: 11px;")
        form.addRow(tip)
        self._wc_status = QLabel(self._tr("状态: 未启动", "Status: Not running"))
        form.addRow(self._tr("状态:", "Status:"), self._wc_status)
        from src.channels import wechat as wc_mod
        if wc_mod.is_running():
            self._wc_status.setText(self._tr("状态: 运行中", "Status: Running"))
        return page

    def _ensure_channel_pages(self):
        makers = [self._make_ws_page, self._make_wc_page, self._make_wcom_page,
                  self._make_dingtalk_page, self._make_qq_page]
        for idx, maker in enumerate(makers):
            if self._channel_pages.get(idx) is None:
                page = maker()
                self._channel_pages[idx] = page
                self._channel_stack.addWidget(page)

    def _wc_qr_login(self):
        from PyQt5.QtWidgets import QDialog, QVBoxLayout, QLabel
        from PyQt5.QtCore import Qt
        from PyQt5.QtGui import QPixmap

        dialog = QDialog(self)
        dialog.setWindowTitle(self._tr("微信扫码登录", "WeChat QR Login"))
        dialog.setMinimumSize(320, 380)
        dl = QVBoxLayout(dialog)

        self._qr_image = QLabel()
        self._qr_image.setAlignment(Qt.AlignCenter)
        self._qr_image.setMinimumSize(280, 280)
        dl.addWidget(self._qr_image)

        self._qr_status = QLabel(self._tr("正在获取二维码...", "Getting QR code..."))
        self._qr_status.setAlignment(Qt.AlignCenter)
        dl.addWidget(self._qr_status)

        close_btn = QPushButton(self._tr("关闭", "Close"))
        close_btn.clicked.connect(dialog.reject)
        dl.addWidget(close_btn)

        dialog.show()

        def progress(msg: str):
            if msg.startswith("qrcode_ready:"):
                data_url = msg[len("qrcode_ready:"):]
                try:
                    qr_api = f"https://api.qrserver.com/v1/create-qr-code/?size=300x300&data={quote(data_url)}"
                    img_data = urlopen(qr_api, timeout=10).read()
                    pixmap = QPixmap()
                    pixmap.loadFromData(img_data)
                    self._qr_image.setPixmap(pixmap.scaled(
                        280, 280, Qt.KeepAspectRatio, Qt.SmoothTransformation))
                    self._qr_status.setText(self._tr("请用微信扫描二维码", "Scan QR code with WeChat"))
                except Exception as e:
                    self._qr_status.setText(self._tr(f"加载二维码失败: {e}", f"Failed to load QR: {e}"))
            else:
                self._qr_status.setText(msg)

        def login_thread():
            from src.channels import wechat as wc_mod
            try:
                result = wc_mod.qr_login(progress_callback=progress)
                token = result["token"]
                self._wc_token.setText(token)
                self._qr_status.setText(self._tr("✓ 登录成功！Token 已填入", "✓ Login success! Token filled"))
                QTimer.singleShot(1000, dialog.accept)
            except Exception as e:
                self._qr_status.setText(self._tr(f"登录失败: {e}", f"Login failed: {e}"))

        t = threading.Thread(target=login_thread, daemon=True)
        t.start()
        dialog.exec_()

    def _make_wcom_page(self):
        page = QWidget()
        form = QFormLayout(page)
        form.setSpacing(10)
        self._wcom_enabled = QComboBox()
        self._wcom_enabled.addItems(["关闭 / Off", "开启 / On"])
        self._wcom_enabled.setCurrentText("开启 / On" if self.config.get("wcom_enabled", False) else "关闭 / Off")
        form.addRow(self._tr("服务开关:", "Server:"), self._wcom_enabled)

        self._wcom_corp_id = QLineEdit(self.config.get("wcom_corp_id", ""))
        form.addRow(self._tr("企业 ID (CorpID):", "Corp ID:"), self._wcom_corp_id)
        self._wcom_agent_id = QLineEdit(self.config.get("wcom_agent_id", ""))
        form.addRow(self._tr("应用 ID (AgentID):", "Agent ID:"), self._wcom_agent_id)
        self._wcom_secret = QLineEdit(self.config.get("wcom_secret", ""))
        self._wcom_secret.setEchoMode(QLineEdit.Password)
        form.addRow(self._tr("应用 Secret:", "Secret:"), self._wcom_secret)
        self._wcom_token = QLineEdit(self.config.get("wcom_token", ""))
        form.addRow(self._tr("回调 Token:", "Callback Token:"), self._wcom_token)
        self._wcom_aes_key = QLineEdit(self.config.get("wcom_aes_key", ""))
        form.addRow(self._tr("AES 密钥:", "AES Key:"), self._wcom_aes_key)

        tip = QLabel(self._tr(
            "企业微信自建应用回调服务。\n"
            "在企业微信管理后台 -> 应用管理 -> 自建应用 -> 接收消息\n"
            "设置回调 URL 指向本服务地址即可。\n"
            "详情: https://developer.work.weixin.qq.com/document/path/90238",
            "WeCom self-built app callback server.\n"
            "Configure in WeCom admin console -> App Management ->\n"
            "Self-built App -> Receive Messages -> set callback URL.\n"
            "Details: https://developer.work.weixin.qq.com/document/path/90238",
        ))
        tip.setWordWrap(True)
        tip.setStyleSheet("color: #888; font-size: 11px;")
        form.addRow(tip)
        return page

    def _make_dingtalk_page(self):
        page = QWidget()
        form = QFormLayout(page)
        form.setSpacing(10)
        self._dt_enabled = QComboBox()
        self._dt_enabled.addItems(["关闭 / Off", "开启 / On"])
        self._dt_enabled.setCurrentText("开启 / On" if self.config.get("dt_enabled", False) else "关闭 / Off")
        form.addRow(self._tr("服务开关:", "Server:"), self._dt_enabled)

        self._dt_app_key = QLineEdit(self.config.get("dt_app_key", ""))
        form.addRow(self._tr("AppKey:", "AppKey:"), self._dt_app_key)
        self._dt_app_secret = QLineEdit(self.config.get("dt_app_secret", ""))
        self._dt_app_secret.setEchoMode(QLineEdit.Password)
        form.addRow(self._tr("AppSecret:", "AppSecret:"), self._dt_app_secret)
        self._dt_webhook = QLineEdit(self.config.get("dt_webhook", ""))
        form.addRow(self._tr("Webhook URL:", "Webhook URL:"), self._dt_webhook)

        tip = QLabel(self._tr(
            "通过钉钉开放平台机器人 API 接入。\n"
            "1. 在钉钉开放平台创建机器人\n"
            "2. 配置出网 Webhook 地址指向本服务\n"
            "3. 使用 AppKey/AppSecret 调用服务端 API",
            "Connect via DingTalk Open Platform Bot API.\n"
            "1. Create a bot on DingTalk Open Platform\n"
            "2. Configure outgoing webhook URL to point here\n"
            "3. Use AppKey/AppSecret for server-side API calls",
        ))
        tip.setWordWrap(True)
        tip.setStyleSheet("color: #888; font-size: 11px;")
        form.addRow(tip)
        return page

    def _make_qq_page(self):
        page = QWidget()
        form = QFormLayout(page)
        form.setSpacing(10)
        self._qq_enabled = QComboBox()
        self._qq_enabled.addItems(["关闭 / Off", "开启 / On"])
        self._qq_enabled.setCurrentText("开启 / On" if self.config.get("qq_enabled", False) else "关闭 / Off")
        form.addRow(self._tr("服务开关:", "Server:"), self._qq_enabled)

        self._qq_mode = QComboBox()
        self._qq_mode.addItems([
            self._tr("go-cqhttp WebSocket", "go-cqhttp WebSocket"),
            self._tr("QQ 官方机器人 API", "QQ Official Bot API"),
        ])
        self._qq_mode.setCurrentText(
            self._qq_mode.itemText(0) if self.config.get("qq_mode", "ws") == "ws"
            else self._qq_mode.itemText(1)
        )
        form.addRow(self._tr("接入方式:", "Mode:"), self._qq_mode)

        self._qq_ws_host = QLineEdit(self.config.get("qq_ws_host", "127.0.0.1"))
        form.addRow(self._tr("WebSocket 地址:", "WS Host:"), self._qq_ws_host)
        self._qq_ws_port = QSpinBox()
        self._qq_ws_port.setRange(1024, 65535)
        self._qq_ws_port.setValue(self.config.get("qq_ws_port", 6700))
        form.addRow(self._tr("WebSocket 端口:", "WS Port:"), self._qq_ws_port)

        self._qq_bot_token = QLineEdit(self.config.get("qq_bot_token", ""))
        self._qq_bot_token.setEchoMode(QLineEdit.Password)
        form.addRow(self._tr("Bot Token:", "Bot Token:"), self._qq_bot_token)
        self._qq_bot_appid = QLineEdit(self.config.get("qq_bot_appid", ""))
        form.addRow(self._tr("Bot AppID:", "Bot AppID:"), self._qq_bot_appid)

        tip = QLabel(self._tr(
            "go-cqhttp 方式使用 WebSocket 反向连接本服务。\n"
            "QQ 官方机器人需在 https://bot.q.qq.com 申请。\n"
            "也可直接使用上方 WebSocket 通道通用接入。",
            "go-cqhttp connects via reverse WebSocket.\n"
            "QQ Official Bot requires application at https://bot.q.qq.com.\n"
            "Can also use the generic WebSocket channel above.",
        ))
        tip.setWordWrap(True)
        tip.setStyleSheet("color: #888; font-size: 11px;")
        form.addRow(tip)
        return page

    def _update_tab(self):
        widget = QWidget()
        layout = QVBoxLayout(widget)
        from PyQt5.QtGui import QDesktopServices
        from PyQt5.QtCore import QUrl

        layout.addWidget(QLabel(
            f'<b>{"Current Version" if self.lang == "en" else "当前版本"}:</b> v{VERSION}'))
        layout.addWidget(QLabel(" "))

        self._update_status = QLabel("")
        layout.addWidget(self._update_status)

        self._update_btn = QPushButton(
            self._tr("🔄 检查更新", "🔄 Check Update"))
        self._update_btn.setStyleSheet(
            "QPushButton { background-color: #6c5ce7; color: white; font-weight: bold; padding: 6px 16px; }"
            "QPushButton:hover { background-color: #5a4bd1; }")
        self._update_btn.clicked.connect(self._do_check_update)
        layout.addWidget(self._update_btn)

        self._update_links = QHBoxLayout()
        self._gh_btn = QPushButton("📦 GitHub Releases")
        self._gh_btn.clicked.connect(
            lambda: QDesktopServices.openUrl(QUrl("https://github.com/xiaopi668/EyeForge/releases")))
        self._gc_btn = QPushButton("📦 GitCode Releases")
        self._gc_btn.clicked.connect(
            lambda: QDesktopServices.openUrl(QUrl("https://gitcode.com/xiaopi668/EyeForge/releases")))
        self._update_links.addWidget(self._gh_btn)
        self._update_links.addWidget(self._gc_btn)
        layout.addLayout(self._update_links)

        layout.addStretch()
        return widget

    def _do_check_update(self):
        self._update_btn.setEnabled(False)
        self._update_btn.setText(self._tr("检查中...", "Checking..."))
        self._update_status.setText("⏳ " + (self._tr("正在检查...", "Checking...")))
        QApplication.processEvents()

        result = check_update()

        if result["latest"] != "Unknown":
            info = (f'<b>{"Latest" if self.lang == "en" else "最新版本"}:</b> v{result["latest"]}<br>'
                    f'<b>{"Source" if self.lang == "en" else "来源"}:</b> {result["source"] or "GitHub"}')
            if result["update_available"]:
                info += f'<br><b style="color:#00d4aa;">✶ {"New version available!" if self.lang == "en" else "有新版本可用！"}</b>'
            else:
                info += f'<br>✓ {"Up to date" if self.lang == "en" else "已是最新版本"}'
            self._update_status.setText(info)
        else:
            self._update_status.setText(
                self._tr("✗ 检查更新失败 (网络或服务异常)", "✗ Update check failed (network or service error)"))

        self._update_btn.setEnabled(True)
        self._update_btn.setText(self._tr("🔄 检查更新", "🔄 Check Update"))

    def _save(self):
        self._ensure_channel_pages()
        prov = self._provider_combo.currentData()
        model_key = {"openai": "openai_model", "anthropic": "anthropic_model",
                     "ollama": "ollama_model", "gemini": "gemini_model", "custom": "custom_model"}
        model_field = {"openai": self._openai_model, "anthropic": self._anthropic_model,
                       "ollama": self._ollama_model, "gemini": self._gemini_model, "custom": self._custom_model}
        model_name = model_field[prov].text().strip()
        from src.utils.multimodal import is_multimodal
        if model_name and not is_multimodal(model_name):
            mb = QMessageBox(self)
            mb.setWindowTitle(self._tr("警告", "Warning"))
            mb.setText(self._tr(
                f'模型 "{model_name}" 可能不支持视觉识别。\nEyeForge 需要多模态模型。',
                f'The model "{model_name}" may not support vision.\nEyeForge requires a multimodal model.'))
            btn_continue = mb.addButton(self._tr("继续保存", "Save Anyway"), QMessageBox.YesRole)
            btn_cancel = mb.addButton(self._tr("取消", "Cancel"), QMessageBox.RejectRole)
            mb.setDefaultButton(btn_continue)
            mb.setEscapeButton(btn_cancel)
            mb.exec_()
            if mb.clickedButton() == btn_cancel:
                return

        self.config["llm_provider"] = prov
        self.config["openai_api_key"] = encrypt(self._openai_key.text())
        self.config["openai_model"] = self._openai_model.text()
        self.config["anthropic_api_key"] = encrypt(self._anthropic_key.text())
        self.config["anthropic_model"] = self._anthropic_model.text()
        self.config["ollama_base_url"] = self._ollama_url.text()
        self.config["ollama_model"] = self._ollama_model.text()
        self.config["custom_api_key"] = encrypt(self._custom_key.text())
        self.config["custom_base_url"] = self._custom_url.text()
        self.config["custom_model"] = self._custom_model.text()
        self.config["gemini_api_key"] = encrypt(self._gemini_key.text())
        self.config["gemini_model"] = self._gemini_model.text()
        self.config["screenshot_quality"] = self._quality_spin.value()
        self.config["action_delay"] = self._delay_spin.value()
        lang_text = self._lang_combo.currentText()
        self.config["language"] = "zh" if "zh" in lang_text else "en"
        self.config["theme"] = self._theme_combo.currentText()
        self.config["font_size"] = self._font_spin.value()
        self.config["hotkey_float"] = self._hotkey_float.text().strip()
        self.config["hotkey_voice"] = self._hotkey_voice.text().strip()
        self.config["wakeword_enabled"] = "开启" in self._wakeword_check.currentText()
        self.config["wakeword_list"] = self._wakeword_list.text().strip()
        self.config["porcupine_access_key"] = self._porcupine_key.text().strip()
        self.config["ws_enabled"] = "开启" in self._ws_enabled.currentText()
        self.config["ws_host"] = self._ws_host.text().strip()
        self.config["ws_port"] = self._ws_port.value()
        self.config["ws_token"] = self._ws_token.text().strip()
        self.config["wc_enabled"] = "开启" in self._wc_enabled.currentText()
        self.config["wc_token"] = self._wc_token.text().strip()
        self.config["wcom_enabled"] = "开启" in self._wcom_enabled.currentText()
        self.config["wcom_corp_id"] = self._wcom_corp_id.text().strip()
        self.config["wcom_agent_id"] = self._wcom_agent_id.text().strip()
        self.config["wcom_secret"] = self._wcom_secret.text().strip()
        self.config["wcom_token"] = self._wcom_token.text().strip()
        self.config["wcom_aes_key"] = self._wcom_aes_key.text().strip()
        self.config["dt_enabled"] = "开启" in self._dt_enabled.currentText()
        self.config["dt_app_key"] = self._dt_app_key.text().strip()
        self.config["dt_app_secret"] = self._dt_app_secret.text().strip()
        self.config["dt_webhook"] = self._dt_webhook.text().strip()
        self.config["qq_enabled"] = "开启" in self._qq_enabled.currentText()
        self.config["qq_mode"] = "ws" if "go-cqhttp" in self._qq_mode.currentText() else "official"
        self.config["qq_ws_host"] = self._qq_ws_host.text().strip()
        self.config["qq_ws_port"] = self._qq_ws_port.value()
        self.config["qq_bot_token"] = self._qq_bot_token.text().strip()
        self.config["qq_bot_appid"] = self._qq_bot_appid.text().strip()

        try:
            with open("config.json", "w", encoding="utf-8") as f:
                json.dump(self.config, f, ensure_ascii=False, indent=2)
            self.accept()
        except Exception as e:
            QMessageBox.critical(self, self._tr("错误", "Error"), f"{self._tr('保存配置失败:', 'Failed to save config:')}\n{e}")

    def get_config(self) -> dict:
        return self.config
