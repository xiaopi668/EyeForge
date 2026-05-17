import json
import logging
from PyQt5.QtWidgets import (
    QApplication, QDialog, QVBoxLayout, QHBoxLayout, QFormLayout,
    QLineEdit, QComboBox, QSpinBox, QDoubleSpinBox,
    QPushButton, QLabel, QTabWidget, QWidget, QMessageBox
)
from PyQt5.QtCore import Qt

logger = logging.getLogger(__name__)


class SettingsDialog(QDialog):
    def __init__(self, config: dict, parent=None):
        super().__init__(parent)
        self.config = config.copy()
        self.lang = self.config.get("language", "zh")
        self.setWindowTitle("设置 - Settings")
        self.setMinimumWidth(500)
        self._init_ui()

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
        self._general_tab_widget.setLayout(general_layout)
        tabs.addTab(self._general_tab_widget, self._tr("常规", "General"))

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

    def _build_provider_section(self, fields: list, pull_provider: str, model_field: QLineEdit) -> QWidget:
        group = QWidget()
        layout = QVBoxLayout(group)
        layout.setContentsMargins(0, 8, 0, 0)
        form = QFormLayout()
        for field_label, field_widget in fields:
            form.addRow(field_label, field_widget)
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

        layout.addStretch()
        self._on_provider_change(self.config.get("llm_provider", "openai"))
        return widget

    def _on_provider_change(self, provider: str):
        self._openai_group.setVisible(provider == "openai")
        self._anthropic_group.setVisible(provider == "anthropic")
        self._ollama_group.setVisible(provider == "ollama")
        self._custom_group.setVisible(provider == "custom")

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

            from PyQt5.QtWidgets import QDialog, QVBoxLayout, QListWidget, QDialogButtonBox
            dialog = QDialog(self)
            dialog.setWindowTitle(f"{self._tr('选择模型', 'Select Model')} - {provider}")
            dialog.resize(400, 500)
            dl = QVBoxLayout(dialog)
            search = QLineEdit()
            search.setPlaceholderText(self._tr("搜索模型...", "Search models..."))
            dl.addWidget(search)
            lst = QListWidget()
            lst.addItems(models)
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
                selected = lst.currentItem().text()
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

    def _save(self):
        self.config["llm_provider"] = self._provider_combo.currentData()
        self.config["openai_api_key"] = self._openai_key.text()
        self.config["openai_model"] = self._openai_model.text()
        self.config["anthropic_api_key"] = self._anthropic_key.text()
        self.config["anthropic_model"] = self._anthropic_model.text()
        self.config["ollama_base_url"] = self._ollama_url.text()
        self.config["ollama_model"] = self._ollama_model.text()
        self.config["custom_api_key"] = self._custom_key.text()
        self.config["custom_base_url"] = self._custom_url.text()
        self.config["custom_model"] = self._custom_model.text()
        self.config["screenshot_quality"] = self._quality_spin.value()
        self.config["action_delay"] = self._delay_spin.value()
        lang_text = self._lang_combo.currentText()
        self.config["language"] = "zh" if "zh" in lang_text else "en"
        self.config["theme"] = self._theme_combo.currentText()
        self.config["font_size"] = self._font_spin.value()

        try:
            with open("config.json", "w", encoding="utf-8") as f:
                json.dump(self.config, f, ensure_ascii=False, indent=2)
            self.accept()
        except Exception as e:
            QMessageBox.critical(self, self._tr("错误", "Error"), f"{self._tr('保存配置失败:', 'Failed to save config:')}\n{e}")

    def get_config(self) -> dict:
        return self.config
