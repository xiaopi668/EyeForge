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

    def on_screenshot(self, image_base64: str):
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

    def execute_action(self, action_data: dict) -> bool:
        action_type = action_data.get("type", "")
        try:
            sw, sh = self.screen.get_screen_size()
            if action_type == "click_ratio":
                x = int(action_data.get("x_ratio", 0.5) * sw)
                y = int(action_data.get("y_ratio", 0.5) * sh)
                self.actions.click(x, y)
            elif action_type == "click":
                self.actions.click(action_data["x"], action_data["y"])
            elif action_type == "double_click":
                self.actions.double_click(action_data["x"], action_data["y"])
            elif action_type == "right_click":
                self.actions.right_click(action_data["x"], action_data["y"])
            elif action_type == "move":
                self.actions.move(action_data["x"], action_data["y"])
            elif action_type == "type":
                self.actions.type_text(action_data.get("text", ""))
            elif action_type == "press_key":
                self.actions.press_key(action_data.get("key", ""))
            elif action_type == "hotkey":
                self.actions.hotkey(*action_data.get("keys", []))
            elif action_type == "scroll":
                self.actions.scroll(action_data.get("clicks", 0))
            elif action_type == "wait":
                time.sleep(action_data.get("seconds", 1))
            elif action_type == "complete":
                self.callback.on_complete()
                return True
            else:
                logger.warning(f"Unknown action type: {action_type}")
            return False
        except Exception as e:
            logger.error(f"Action execution error: {e}")
            self.callback.on_error(str(e))
            return False

    def _init_session(self, task: str):
        self._language = self.config.get("language", "zh")
        system_prompt = get_system_prompt(self._language)
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

    def _loop(self):
        while self.running and self._step_count < self.max_steps:
            self._step_count += 1
            try:
                screenshot = self.screen.capture_pil()
                image_b64 = self.vision.image_to_base64(screenshot)

                self.callback.on_screenshot(image_b64)

                msg = self._messages[-1]
                response = self.llm.chat(self._messages, image_base64=image_b64)
                if not response:
                    self.callback.on_error("LLM 响应为空，请检查 API 配置")
                    break

                parsed = self.llm.parse_action(response)
                if not parsed:
                    self.callback.on_error(f"无法解析 LLM 响应: {response[:200]}")
                    continue

                thought = parsed.get("thought", "")
                action = parsed.get("action", {})

                self.callback.on_step({
                    "step": self._step_count,
                    "thought": thought,
                    "action": action,
                    "raw_response": response,
                })

                self._messages.append({"role": "assistant", "content": response})
                self._messages.append({"role": "user", "content": "观察结果：" + thought})

                done = self.execute_action(action)
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
