import json
import logging
from PyQt5.QtWidgets import (
    QWizard, QWizardPage, QVBoxLayout, QHBoxLayout, QFormLayout,
    QLabel, QComboBox, QLineEdit, QSpinBox, QDoubleSpinBox,
    QPushButton, QMessageBox, QApplication,
)
from PyQt5.QtCore import Qt

from src.utils.multimodal import is_multimodal
from src.utils.crypto import encrypt

logger = logging.getLogger(__name__)

_PROVIDERS = [
    ("openai", "OpenAI"),
    ("anthropic", "Anthropic"),
    ("ollama", "Ollama"),
    ("gemini", "Gemini"),
    ("custom", "自定义"),
]


class LanguagePage(QWizardPage):
    def __init__(self):
        super().__init__()
        self.setTitle("语言 / Language")
        layout = QVBoxLayout(self)
        layout.addWidget(QLabel("请选择界面语言：\nPlease select the UI language:"))
        self.lang_combo = QComboBox()
        self.lang_combo.addItems(["中文 (zh)", "English (en)"])
        layout.addWidget(self.lang_combo)
        layout.addStretch()

    def selected(self):
        return "zh" if "zh" in self.lang_combo.currentText() else "en"


class ModelPage(QWizardPage):
    def __init__(self):
        super().__init__()
        self.setTitle("AI 模型 / AI Model")
        layout = QVBoxLayout(self)
        form = QFormLayout()

        self.provider_combo = QComboBox()
        for val, label in _PROVIDERS:
            self.provider_combo.addItem(label, val)
        form.addRow("Provider: / 提供商:", self.provider_combo)

        self.api_key = QLineEdit()
        self.api_key.setPlaceholderText("留空可跳过 / leave blank to skip")
        form.addRow("API Key: / API 密钥:", self.api_key)

        self.model = QLineEdit()
        self.model.setPlaceholderText("gpt-4o")
        form.addRow("Model: / 模型:", self.model)

        self.model_status = QLabel("")
        form.addRow("", self.model_status)
        self.model.textChanged.connect(self._update_model_status)

        self.base_url = QLineEdit()
        self.base_url.setPlaceholderText("Base URL")
        self.url_label = QLabel("Base URL:")
        form.addRow(self.url_label, self.base_url)

        self.provider_combo.currentIndexChanged.connect(self._on_provider_index)

        layout.addLayout(form)

        btn_row = QHBoxLayout()
        self.fetch_btn = QPushButton("📥 拉取模型 / Fetch Models")
        self.fetch_btn.clicked.connect(self._fetch_models)
        self.test_btn = QPushButton("测试连接 / Test")
        self.test_btn.clicked.connect(self._test_connection)
        self.test_status = QLabel("")
        btn_row.addWidget(self.fetch_btn)
        btn_row.addWidget(self.test_btn)
        btn_row.addWidget(self.test_status)
        btn_row.addStretch()
        layout.addLayout(btn_row)
        layout.addStretch()

    def _get_provider(self):
        return self.provider_combo.currentData()

    def _update_model_status(self, text: str):
        self.model_status.setText("🟢 多模态 / Multimodal" if is_multimodal(text) else "⚪ 未知 / Unknown")

    def initializePage(self):
        self._on_provider(self._get_provider())
        self._update_model_status(self.model.text())

    def _on_provider_index(self):
        self._on_provider(self._get_provider())

    def _on_provider(self, p: str):
        hints = {
            "openai": ("https://api.openai.com/v1", "gpt-4o"),
            "anthropic": ("https://api.anthropic.com", "claude-3-5-sonnet-20241022"),
            "ollama": ("http://localhost:11434", "llava"),
            "gemini": ("", "gemini-2.5-flash"),
            "custom": ("https://api.openai.com/v1", "gpt-4o"),
        }
        url, model = hints.get(p, ("", ""))
        self.base_url.setText(url)
        visible = p not in ("anthropic", "gemini")
        self.base_url.setVisible(visible)
        self.url_label.setVisible(visible)
        self.model.setText(model)

    def _test_connection(self):
        from src.ai.llm_client import LLMClient
        cfg = self._build_config()
        client = LLMClient(provider=cfg["llm_provider"], config=cfg)
        if not client.is_available():
            QMessageBox.warning(self, "提示 / Notice", "API Key 未填写 / API Key missing")
            return
        self.test_btn.setEnabled(False)
        self.test_btn.setText("测试中... / Testing...")
        self.test_status.setText("")
        QApplication.processEvents()
        try:
            resp = client.chat(messages=[{"role": "user", "content": "Reply 'OK' only."}])
            if resp:
                self.test_status.setText("✓ OK")
        except Exception as e:
            self.test_status.setText("✗ 失败 / Fail")
        finally:
            self.test_btn.setEnabled(True)
            self.test_btn.setText("测试连接 / Test")

    def _fetch_models(self):
        import requests
        from PyQt5.QtWidgets import QDialog, QVBoxLayout, QListWidget, QDialogButtonBox

        prov = self._get_provider()
        cfg = self._build_config()
        self.fetch_btn.setEnabled(False)
        self.fetch_btn.setText("获取中... / Fetching...")
        QApplication.processEvents()

        models = []
        try:
            if prov == "ollama":
                url = cfg.get("ollama_base_url", "").strip()
                if not url:
                    raise ValueError("需要填写 Ollama URL / Ollama URL required")
                r = requests.get(f"{url.rstrip('/')}/api/tags", timeout=10)
                r.raise_for_status()
                models = [m["name"] for m in r.json().get("models", [])]
            elif prov == "openai":
                k = cfg.get("openai_api_key", "").strip()
                if not k:
                    raise ValueError("需要填写 API Key / API Key required")
                r = requests.get("https://api.openai.com/v1/models", headers={"Authorization": f"Bearer {k}"}, timeout=15)
                r.raise_for_status()
                models = [m["id"] for m in r.json().get("data", [])]
            elif prov == "anthropic":
                k = cfg.get("anthropic_api_key", "").strip()
                if not k:
                    raise ValueError("需要填写 API Key / API Key required")
                r = requests.get("https://api.anthropic.com/v1/models", headers={"x-api-key": k, "anthropic-version": "2023-06-01"}, timeout=15)
                r.raise_for_status()
                models = [m["name"] for m in r.json().get("data", [])]
            elif prov == "gemini":
                k = cfg.get("gemini_api_key", "").strip()
                if not k:
                    raise ValueError("需要填写 API Key / API Key required")
                r = requests.get("https://generativelanguage.googleapis.com/v1beta/models?key=" + k, timeout=15)
                r.raise_for_status()
                models = [m["name"].replace("models/", "") for m in r.json().get("models", [])]
            elif prov == "custom":
                k = cfg.get("custom_api_key", "").strip()
                u = cfg.get("custom_base_url", "").strip()
                if not k or not u:
                    raise ValueError("需要填写 API Key 和 URL / API Key and URL required")
                r = requests.get(f"{u.rstrip('/')}/models", headers={"Authorization": f"Bearer {k}"}, timeout=15)
                r.raise_for_status()
                models = [m["id"] for m in r.json().get("data", [])]

            if not models:
                QMessageBox.information(self, "提示 / Notice", "未找到模型 / No models found")
                return

            annotated = [f"{m}  {'🟢' if is_multimodal(m) else '⚪'}" for m in models]
            dialog = QDialog(self)
            dialog.setWindowTitle(f"选择模型 / Select Model - {prov}")
            dialog.resize(450, 500)
            dl = QVBoxLayout(dialog)
            search = QLineEdit()
            search.setPlaceholderText("搜索... / Search...")
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
                self.model.setText(lst.currentItem().text().split("  ")[0].strip())
        except Exception as e:
            QMessageBox.warning(self, "错误 / Error", str(e))
        finally:
            self.fetch_btn.setEnabled(True)
            self.fetch_btn.setText("📥 拉取模型 / Fetch Models")

    def _build_config(self):
        prov = self._get_provider()
        return {
            "llm_provider": prov,
            "openai_api_key": self.api_key.text() if prov == "openai" else "",
            "openai_model": self.model.text(),
            "anthropic_api_key": self.api_key.text() if prov == "anthropic" else "",
            "anthropic_model": self.model.text(),
            "ollama_base_url": self.base_url.text() if prov == "ollama" else "",
            "ollama_model": self.model.text(),
            "gemini_api_key": self.api_key.text() if prov == "gemini" else "",
            "gemini_model": self.model.text(),
            "custom_api_key": self.api_key.text() if prov == "custom" else "",
            "custom_base_url": self.base_url.text() if prov == "custom" else "",
            "custom_model": self.model.text(),
        }


class CapturePage(QWizardPage):
    def __init__(self):
        super().__init__()
        self.setTitle("截屏设置 / Capture Settings")
        layout = QFormLayout(self)
        self.quality = QSpinBox()
        self.quality.setRange(10, 100)
        self.quality.setValue(95)
        layout.addRow("截图质量 / Quality (1-100):", self.quality)
        self.delay = QDoubleSpinBox()
        self.delay.setRange(0.0, 5.0)
        self.delay.setSingleStep(0.1)
        self.delay.setValue(0.5)
        layout.addRow("操作延迟 / Action Delay (s):", self.delay)
        self.theme_combo = QComboBox()
        self.theme_combo.addItems(["dark", "light"])
        layout.addRow("主题 / Theme:", self.theme_combo)


class FirstRunWizard(QWizard):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("EyeForge - 首次设置 / First Run Setup")
        self.setMinimumSize(540, 460)
        self.lang_page = LanguagePage()
        self.model_page = ModelPage()
        self.capture_page = CapturePage()
        self.addPage(self.lang_page)
        self.addPage(self.model_page)
        self.addPage(self.capture_page)
        self.setStartId(0)

    def get_config(self) -> dict:
        lang = self.lang_page.selected()
        cfg = {
            "language": lang,
            "llm_provider": self.model_page._get_provider(),
            "openai_api_key": "", "openai_model": "gpt-4o",
            "anthropic_api_key": "", "anthropic_model": "claude-3-5-sonnet-20241022",
            "ollama_base_url": "http://localhost:11434", "ollama_model": "llava",
            "custom_api_key": "", "custom_base_url": "https://api.openai.com/v1", "custom_model": "gpt-4o",
            "gemini_api_key": "", "gemini_model": "gemini-2.5-flash",
            "screenshot_quality": self.capture_page.quality.value(),
            "action_delay": self.capture_page.delay.value(),
            "theme": self.capture_page.theme_combo.currentText(),
            "font_size": 9, "wizard_done": True,
        }
        prov = cfg["llm_provider"]
        key = self.model_page.api_key.text().strip()
        model = self.model_page.model.text().strip()
        url = self.model_page.base_url.text().strip()
        if prov == "openai":
            cfg["openai_api_key"] = encrypt(key); cfg["openai_model"] = model or "gpt-4o"
        elif prov == "anthropic":
            cfg["anthropic_api_key"] = encrypt(key); cfg["anthropic_model"] = model or "claude-3-5-sonnet-20241022"
        elif prov == "ollama":
            cfg["ollama_base_url"] = url or "http://localhost:11434"; cfg["ollama_model"] = model or "llava"
        elif prov == "gemini":
            cfg["gemini_api_key"] = encrypt(key); cfg["gemini_model"] = model or "gemini-2.5-flash"
        elif prov == "custom":
            cfg["custom_api_key"] = encrypt(key); cfg["custom_base_url"] = url or "https://api.openai.com/v1"; cfg["custom_model"] = model or "gpt-4o"
        return cfg
