import json
import logging
from typing import Optional

logger = logging.getLogger(__name__)


from src.utils.crypto import decrypt


class LLMClient:
    def __init__(self, provider: str = "openai", config: dict = None):
        self.provider = provider
        self.config = config or {}
        self._client = None
        self._init_client()

    def _init_client(self):
        if self.provider == "openai":
            import openai
            api_key = decrypt(self.config.get("openai_api_key", ""))
            if api_key:
                self._client = openai.OpenAI(api_key=api_key)
            else:
                self._client = None
        elif self.provider == "anthropic":
            import anthropic
            api_key = decrypt(self.config.get("anthropic_api_key", ""))
            if api_key:
                self._client = anthropic.Anthropic(api_key=api_key)
            else:
                self._client = None
        elif self.provider == "ollama":
            self._client = True
        elif self.provider == "custom":
            import openai
            api_key = decrypt(self.config.get("custom_api_key", ""))
            base_url = self.config.get("custom_base_url", "https://api.openai.com/v1")
            if api_key and base_url:
                self._client = openai.OpenAI(api_key=api_key, base_url=base_url)
            else:
                self._client = None
        elif self.provider == "gemini":
            import openai
            api_key = decrypt(self.config.get("gemini_api_key", ""))
            if api_key:
                self._client = openai.OpenAI(
                    api_key=api_key,
                    base_url="https://generativelanguage.googleapis.com/v1beta/openai/"
                )
            else:
                self._client = None
        else:
            self._client = None

    def is_available(self) -> bool:
        return self._client is not None

    def get_model_name(self) -> str:
        key_map = {
            "openai": "openai_model",
            "anthropic": "anthropic_model",
            "ollama": "ollama_model",
            "custom": "custom_model",
            "gemini": "gemini_model",
        }
        return self.config.get(key_map.get(self.provider, ""), "")

    def chat(self, messages: list, image_base64: str = None) -> Optional[str]:
        if self.provider == "openai":
            return self._chat_openai(messages, image_base64)
        elif self.provider == "anthropic":
            return self._chat_anthropic(messages, image_base64)
        elif self.provider == "ollama":
            return self._chat_ollama(messages, image_base64)
        elif self.provider == "custom":
            return self._chat_custom(messages, image_base64)
        elif self.provider == "gemini":
            return self._chat_custom(messages, image_base64)
        return None

    def simple_chat(self, message: str) -> Optional[str]:
        from src.ai.prompts import CHAT_PROMPT_ZH, CHAT_PROMPT_EN
        lang = self.config.get("language", "zh")
        system = CHAT_PROMPT_ZH if lang == "zh" else CHAT_PROMPT_EN
        messages = [
            {"role": "system", "content": system},
            {"role": "user", "content": message},
        ]
        return self.chat(messages)

    def _chat_openai(self, messages: list, image_base64: str = None) -> Optional[str]:
        if not self._client:
            return None
        try:
            openai_messages = []
            for msg in messages:
                if msg["role"] == "system":
                    openai_messages.append({"role": "system", "content": msg["content"]})
                elif msg["role"] == "user":
                    content = [{"type": "text", "text": msg["content"]}]
                    if image_base64:
                        content.append({
                            "type": "image_url",
                            "image_url": {"url": f"data:image/jpeg;base64,{image_base64}"}
                        })
                    openai_messages.append({"role": "user", "content": content})

            response = self._client.chat.completions.create(
                model=self.get_model_name(),
                messages=openai_messages,
                max_tokens=1024,
            )
            return response.choices[0].message.content
        except Exception as e:
            logger.error(f"OpenAI API error: {e}")
            return None

    def _chat_anthropic(self, messages: list, image_base64: str = None) -> Optional[str]:
        if not self._client:
            return None
        try:
            system_content = ""
            anthropic_messages = []
            for msg in messages:
                if msg["role"] == "system":
                    system_content = msg["content"]
                elif msg["role"] == "user":
                    content = [{"type": "text", "text": msg["content"]}]
                    if image_base64:
                        content.append({
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": "image/jpeg",
                                "data": image_base64,
                            }
                        })
                    anthropic_messages.append({"role": "user", "content": content})

            response = self._client.messages.create(
                model=self.get_model_name(),
                system=system_content,
                messages=anthropic_messages,
                max_tokens=1024,
            )
            return response.content[0].text
        except Exception as e:
            logger.error(f"Anthropic API error: {e}")
            return None

    def ensure_model(self, progress_callback=None) -> bool:
        if self.provider != "ollama":
            return True
        import requests
        base_url = self.config.get("ollama_base_url", "http://localhost:11434")
        model = self.get_model_name()
        try:
            resp = requests.get(f"{base_url}/api/tags", timeout=10)
            if resp.status_code == 200:
                models = resp.json().get("models", [])
                if any(m["name"] == model or m["name"].startswith(model + ":") for m in models):
                    return True
            return self._pull_model(base_url, model, progress_callback)
        except requests.RequestException:
            return self._pull_model(base_url, model, progress_callback)

    def _pull_model(self, base_url: str, model: str, progress_callback=None) -> bool:
        import requests
        try:
            if progress_callback:
                progress_callback(f"正在拉取模型 {model} ...")
            resp = requests.post(
                f"{base_url}/api/pull",
                json={"name": model, "stream": False},
                timeout=600,
            )
            if resp.status_code == 200:
                if progress_callback:
                    progress_callback(f"模型 {model} 拉取完成")
                return True
            if progress_callback:
                progress_callback(f"拉取模型失败: {resp.text}")
            return False
        except requests.RequestException as e:
            if progress_callback:
                progress_callback(f"拉取模型失败: {e}")
            return False

    def _chat_ollama(self, messages: list, image_base64: str = None) -> Optional[str]:
        import requests
        try:
            base_url = self.config.get("ollama_base_url", "http://localhost:11434")
            ollama_messages = []
            for msg in messages:
                content = msg["content"]
                if msg["role"] == "user" and image_base64:
                    content = [
                        {"type": "text", "text": msg["content"]},
                        {"type": "image_url", "image_url": f"data:image/jpeg;base64,{image_base64}"}
                    ]
                ollama_messages.append({"role": msg["role"], "content": content})

            payload = {
                "model": self.get_model_name(),
                "messages": ollama_messages,
                "stream": False,
            }
            response = requests.post(
                f"{base_url}/api/chat",
                json=payload,
                timeout=120,
            )
            response.raise_for_status()
            data = response.json()
            content = data.get("message", {}).get("content", "")
            return content
        except Exception as e:
            logger.error(f"Ollama API error: {e}")
            return None

    def _chat_custom(self, messages: list, image_base64: str = None) -> Optional[str]:
        if not self._client:
            return None
        try:
            openai_messages = []
            for msg in messages:
                if msg["role"] == "system":
                    openai_messages.append({"role": "system", "content": msg["content"]})
                elif msg["role"] == "user":
                    content = [{"type": "text", "text": msg["content"]}]
                    if image_base64:
                        content.append({
                            "type": "image_url",
                            "image_url": {"url": f"data:image/jpeg;base64,{image_base64}"}
                        })
                    openai_messages.append({"role": "user", "content": content})

            response = self._client.chat.completions.create(
                model=self.get_model_name(),
                messages=openai_messages,
                max_tokens=1024,
            )
            return response.choices[0].message.content
        except Exception as e:
            logger.error(f"Custom API error: {e}")
            return None

    def parse_action(self, response_text: str) -> Optional[dict]:
        try:
            if "```json" in response_text:
                json_str = response_text.split("```json")[1].split("```")[0].strip()
            elif "```" in response_text:
                json_str = response_text.split("```")[1].split("```")[0].strip()
            else:
                json_str = response_text.strip()
            parsed = json.loads(json_str)
            if "action" in parsed:
                return parsed
            return None
        except (json.JSONDecodeError, IndexError):
            logger.warning(f"Failed to parse LLM response: {response_text[:200]}")
            return None
