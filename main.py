import sys
import os
import logging

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


def main():
    from PyQt5.QtWidgets import QApplication
    from PyQt5.QtCore import qInstallMessageHandler
    from PyQt5.QtGui import QFont
    from src.ui.main_window import MainWindow

    try:
        qInstallMessageHandler(_qt_message_handler)
        os.environ["QT_AUTO_SCREEN_SCALE_FACTOR"] = "1"
        QApplication.setAttribute(0x100, True)
        app = QApplication(sys.argv)
        app.setApplicationName("EyeForge")

        font = QFont("Microsoft YaHei UI", 9)
        app.setFont(font)

        window = MainWindow()
        window.show()

        sys.exit(app.exec_())
    except Exception as e:
        logger.critical(f"Fatal error: {e}", exc_info=True)
        sys.exit(1)


if __name__ == "__main__":
    main()
