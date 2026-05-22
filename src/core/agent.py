import json
import time
import logging
from typing import Optional

from src.core.screen import ScreenCapture
from src.core.actions import ActionController
from src.core.vision import VisionProcessor
from src.ai.llm_client import LLMClient
from src.ai.prompts import get_system_prompt, get_task_prompt

logger = logging.getLogger(__name__)


class StepCallback:
    def on_step(self, step_data: dict):
        pass

    def on_error(self, error: str):
        pass

    def on_complete(self):
        pass

    def on_result(self, result: str):
        pass

    def on_screenshot(self, image_base64: str):
        pass

    def on_shell_command(self, command: str, output: str):
        pass

    def on_status(self, message: str):
        pass


class EyeForgeAgent:
    def __init__(self, config: dict, callback: StepCallback = None):
        self.config = config
        self.callback = callback or StepCallback()
        self.screen = ScreenCapture()
        self.vision = VisionProcessor(
            quality=config.get("screenshot_quality", 70),
        )
        self.actions = ActionController(
            action_delay=config.get("action_delay", 0.5)
        )
        self.llm = LLMClient(
            provider=config.get("llm_provider", "openai"),
            config=config,
        )
        self.running = False
        self.max_steps = 50
        self._messages = []
        self._step_count = 0
        self._language = "zh"
        self._sw = 0
        self._sh = 0
        self._has_session = False

    def execute_action(self, action_data) -> tuple:
        try:
            if isinstance(action_data, str):
                action_type = action_data
                params = {}
            elif isinstance(action_data, dict):
                action_type = action_data.get("type", "")
                params = action_data
            else:
                return False, ""

            sw, sh = self.screen.get_screen_size()

            if action_type == "screen_capture":
                return False, "screen_capture"

            elif action_type == "shell":
                cmd = params.get("command", "")
                if not cmd:
                    return False, "(shell command is empty)"
                output = self.actions.execute_shell(cmd)
                self.callback.on_shell_command(cmd, output)
                logger.info(f"Shell output: {output[:200]}")
                return False, f"命令执行结果:\n{output}"

            elif action_type == "click_ratio":
                xr = params.get("x_ratio", 0.5)
                yr = params.get("y_ratio", 0.5)
                x = int(xr * sw)
                y = int(yr * sh)
                self.actions.click(x, y)
                return False, f"已点击 ({xr:.2f}, {yr:.2f})"

            elif action_type == "click":
                self.actions.click(params["x"], params["y"])
            elif action_type == "double_click":
                self.actions.double_click(params["x"], params["y"])
            elif action_type == "right_click":
                self.actions.right_click(params["x"], params["y"])
            elif action_type == "move":
                self.actions.move(params["x"], params["y"])
            elif action_type == "type":
                self.actions.type_text(params.get("text", ""))
            elif action_type == "press_key":
                self.actions.press_key(params.get("key", ""))
            elif action_type == "hotkey":
                self.actions.hotkey(*params.get("keys", []))
            elif action_type == "scroll":
                self.actions.scroll(params.get("clicks", 0))
            elif action_type == "wait":
                time.sleep(params.get("seconds", 1))
            elif action_type == "complete":
                result_text = params.get("result", params.get("data", {}).get("result", ""))
                self.callback.on_complete()
                if result_text:
                    self.callback.on_result(result_text)
                return True, ""
            else:
                logger.warning(f"Unknown action type: {action_type}")
                return False, f"(unknown action: {action_type})"

            return False, f"已执行 {action_type}"
        except Exception as e:
            logger.error(f"Action execution error: {e}")
            self.callback.on_error(str(e))
            return False, f"(error: {e})"

    def _init_session(self, task: str):
        self._language = self.config.get("language", "zh")
        use_vision = self.config.get("use_vision", True)
        system_prompt = get_system_prompt(self._language, use_vision)
        self._sw, self._sh = self.screen.get_screen_size()
        user_prompt = get_task_prompt(task, self._sw, self._sh, self._language)
        self._messages = [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt},
        ]
        self._step_count = 0
        self._has_session = True

    def run(self, task: str):
        if not self.llm.is_available():
            self.callback.on_error("LLM 未配置，请在设置中配置 API Key")
            return

        self.running = True
        self._init_session(task)

        if not self.llm.ensure_model(progress_callback=self.callback.on_status):
            self.callback.on_error("模型拉取失败，请检查 Ollama 服务或模型名称")
            self.running = False
            return

        self._loop()

    def continue_with(self, task: str):
        if not self.llm.is_available():
            self.callback.on_error("LLM 未配置，请在设置中配置 API Key")
            return

        self.running = True
        if not self._has_session:
            self._init_session(task)
        else:
            user_prompt = get_task_prompt(task, self._sw, self._sh, self._language)
            self._messages.append({"role": "user", "content": user_prompt})

        self._loop()

    def _process_response(self, response: str) -> tuple:
        parsed = self.llm.parse_action(response)
        if not parsed:
            self.callback.on_error(f"无法解析 LLM 响应: {response[:200]}")
            return None, None

        thought = parsed.get("thought", "")
        action = parsed.get("action", {})

        self.callback.on_step({
            "step": self._step_count,
            "thought": thought,
            "action": action,
            "raw_response": response,
        })

        return action, thought

    def _loop(self):
        while self.running and self._step_count < self.max_steps:
            self._step_count += 1
            try:
                response = self.llm.chat(self._messages)
                if not response:
                    self.callback.on_error("LLM 响应为空，请检查 API 配置")
                    break

                action, thought = self._process_response(response)
                if action is None:
                    continue

                if action.get("type") == "screen_capture":
                    screenshot = self.screen.capture_pil()
                    image_b64 = self.vision.image_to_base64(screenshot)
                    self.callback.on_screenshot(image_b64)
                    self._messages.append({"role": "assistant", "content": response})
                    self._messages.append({"role": "user", "content": "已截取屏幕，请分析画面并决定下一步操作。"})
                    response2 = self.llm.chat(self._messages, image_base64=image_b64)
                    if not response2:
                        self.callback.on_error("LLM 响应为空")
                        break
                    action, thought = self._process_response(response2)
                    if action is None:
                        continue
                    response = response2

                self._messages.append({"role": "assistant", "content": response})
                done, feedback = self.execute_action(action)
                if feedback:
                    self._messages.append({"role": "user", "content": feedback})

                if done:
                    break

            except Exception as e:
                logger.error(f"Agent loop error: {e}")
                self.callback.on_error(str(e))
                break

        self.running = False
        if self._step_count >= self.max_steps:
            self.callback.on_error("已达到最大执行步数")

    def stop(self):
        self.running = False
