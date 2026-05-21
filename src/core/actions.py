import subprocess
import time
import pyautogui
import logging

logger = logging.getLogger(__name__)

pyautogui.FAILSAFE = True
pyautogui.PAUSE = 0.1


class ActionController:
    def __init__(self, action_delay: float = 0.5):
        self.delay = action_delay
        self.screen_width, self.screen_height = pyautogui.size()

    def click(self, x: int, y: int, button: str = "left", clicks: int = 1):
        pyautogui.click(x, y, clicks=clicks, button=button)
        time.sleep(self.delay)
        logger.info(f"Click at ({x}, {y})")

    def double_click(self, x: int, y: int):
        self.click(x, y, clicks=2)

    def right_click(self, x: int, y: int):
        self.click(x, y, button="right")

    def move(self, x: int, y: int, duration: float = 0.2):
        pyautogui.moveTo(x, y, duration=duration)
        time.sleep(self.delay * 0.5)

    def drag(self, x1: int, y1: int, x2: int, y2: int, duration: float = 0.3):
        pyautogui.moveTo(x1, y1, duration=duration * 0.5)
        pyautogui.drag(x2 - x1, y2 - y1, duration=duration)
        time.sleep(self.delay)

    def type_text(self, text: str, interval: float = 0.02):
        pyautogui.write(text, interval=interval)
        time.sleep(self.delay)

    def press_key(self, key: str):
        pyautogui.press(key)
        time.sleep(self.delay * 0.5)

    def hotkey(self, *keys: str):
        pyautogui.hotkey(*keys)
        time.sleep(self.delay)

    def scroll(self, clicks: int, x=None, y=None):
        pyautogui.scroll(clicks, x=x, y=y)
        time.sleep(self.delay * 0.5)

    def get_mouse_position(self):
        return pyautogui.position()

    def screenshot(self, region=None):
        return pyautogui.screenshot(region=region)

    def execute_shell(self, command: str, timeout: int = 15) -> str:
        """Execute a shell command and return its output."""
        logger.info(f"Shell: {command}")
        try:
            result = subprocess.run(
                command,
                shell=True,
                capture_output=True,
                text=True,
                timeout=timeout,
            )
            output = ""
            if result.stdout:
                output += result.stdout
            if result.stderr:
                output += result.stderr
            if result.returncode != 0:
                output += f"\n(exit code: {result.returncode})"
            return output.strip() or "(no output)"
        except subprocess.TimeoutExpired:
            return "(command timed out)"
        except Exception as e:
            return f"(error: {e})"
