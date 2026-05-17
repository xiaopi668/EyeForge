_VISION_PREFIXES = [
    # OpenAI
    "gpt-4o", "gpt-4-turbo", "gpt-4-vision","gpt-5","gpt-5-turbo","gpt-5.1","gpt-5.2","gpt-5.3","gpt-5.4","gpt-5.4-mini",
    # Anthropic
    "claude-3-5-sonnet", "claude-3-opus", "claude-3-sonnet", "claude-3-haiku","claude-4.5",
    # Ollama
    "llava", "bakllava", "moondream", "cogvlm", "glm4v", "qwen-vl",
    "minicpm-v", "deepseek-vl", "internvl", "xcomposer", "fuyu",
    # Gemini
    "gemini",
    # Common vision keywords
    "vision", "vl", "multimodal","kimi-k2.6","GLM-4.6V","GLM-4.1V-Thinking","GLM-5V-Turbo"
]


def is_multimodal(model: str) -> bool:
    m = model.lower().strip()
    for prefix in _VISION_PREFIXES:
        if m.startswith(prefix):
            return True
    return False


def check_model(model: str) -> str:
    return "🟢" if is_multimodal(model) else "⚪"
