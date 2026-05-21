import sys
import os
import json
import logging
import ctypes

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    handlers=[
        logging.StreamHandler(),
        logging.FileHandler("logs/eyeforge.log", encoding="utf-8"),
    ],
)
logger = logging.getLogger(__name__)


def _qt_message_handler(mode, context, message):
    if "UpdateLayeredWindowIndirect" in message:
        return


def _is_first_run():
    try:
        with open("config.json", "r", encoding="utf-8") as f:
            cfg = json.load(f)
            return not cfg.get("wizard_done", False)
    except (FileNotFoundError, json.JSONDecodeError):
        return True


def main():
    from PyQt5.QtWidgets import QApplication
    from PyQt5.QtCore import qInstallMessageHandler
    from PyQt5.QtGui import QFont, QIcon
    from src.ui.main_window import MainWindow

    try:
        qInstallMessageHandler(_qt_message_handler)
        os.environ["QT_AUTO_SCREEN_SCALE_FACTOR"] = "1"
        QApplication.setAttribute(0x100, True)
        try:
            ctypes.windll.shell32.SetCurrentProcessExplicitAppUserModelID("eyeforge.app.1")
        except Exception:
            pass
        app = QApplication(sys.argv)
        app.setApplicationName("EyeForge")

        icon_path = os.path.join(os.path.dirname(__file__), "src", "logo.ico")
        if os.path.exists(icon_path):
            app.setWindowIcon(QIcon(icon_path))

        font = QFont("Microsoft YaHei UI", 9)
        app.setFont(font)

        if _is_first_run():
            from src.ui.wizard import FirstRunWizard
            wizard = FirstRunWizard()
            if wizard.exec_():
                cfg = wizard.get_config()
                try:
                    os.makedirs("logs", exist_ok=True)
                    with open("config.json", "w", encoding="utf-8") as f:
                        json.dump(cfg, f, ensure_ascii=False, indent=2)
                except Exception as e:
                    logger.error(f"Failed to save config after wizard: {e}")
            else:
                default = {
                    "language": "zh", "llm_provider": "openai",
                    "openai_api_key": "", "openai_model": "gpt-4o",
                    "anthropic_api_key": "", "anthropic_model": "claude-3-5-sonnet-20241022",
                    "ollama_base_url": "http://localhost:11434", "ollama_model": "llava",
                    "custom_api_key": "", "custom_base_url": "https://api.openai.com/v1",
                    "custom_model": "gpt-4o",
                    "gemini_api_key": "", "gemini_model": "gemini-2.5-flash",
                    "screenshot_quality": 95, "action_delay": 0.5,
                    "theme": "dark", "font_size": 9, "wizard_done": True,
                    "hotkey_float": "ctrl+shift+e", "hotkey_voice": "ctrl+shift+v",
                    "wakeword_enabled": True, "wakeword_list": "computer",
                    "porcupine_access_key": "",
                    "ws_enabled": False, "ws_host": "0.0.0.0", "ws_port": 8765, "ws_token": "",
                    "wc_enabled": False, "wc_host": "0.0.0.0", "wc_port": 8800, "wc_token": "",
                    "wcom_enabled": False, "wcom_corp_id": "", "wcom_agent_id": "", "wcom_secret": "", "wcom_token": "", "wcom_aes_key": "",
                    "dt_enabled": False, "dt_app_key": "", "dt_app_secret": "", "dt_webhook": "",
                    "qq_enabled": False, "qq_mode": "ws", "qq_ws_host": "127.0.0.1", "qq_ws_port": 6700, "qq_bot_token": "", "qq_bot_appid": "",
                }
                try:
                    os.makedirs("logs", exist_ok=True)
                    with open("config.json", "w", encoding="utf-8") as f:
                        json.dump(default, f, ensure_ascii=False, indent=2)
                except Exception as e:
                    logger.error(f"Failed to save default config: {e}")

        window = MainWindow()
        window.show()

        sys.exit(app.exec_())
    except Exception as e:
        logger.critical(f"Fatal error: {e}", exc_info=True)
        sys.exit(1)


if __name__ == "__main__":
    main()
