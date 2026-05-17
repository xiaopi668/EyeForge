# EyeForge 🧿

**AI Screen Control Assistant** — Let AI models see your screen and control your mouse and keyboard.

> [中文](README.md) | English
>
> 📋 [Changelog](CHANGELOG_EN.md) | 📦 [GitHub Releases](https://github.com/xiaopi668/EyeForge/releases) | 📦 [GitCode Releases](https://gitcode.com/xiaopi668/EyeForge/releases)



## Features

- **Screen Awareness** — Captures your screen in real-time and sends it to **multimodal** AI models for analysis
- **Intelligent Control** — AI autonomously plans steps, controls mouse clicks, movements, and keyboard input
- **Multi-Model Support** — Supports OpenAI, Anthropic, Ollama, and any OpenAI-compatible custom API (requires **multimodal** models such as gpt-4o, claude-3-5-sonnet, llava, etc.)
- **Graphical Interface** — PyQt5-based GUI with dark/light theme switching
- **Internationalization** — UI supports both 中文 and English, switchable on the fly
- **Click Visualization** — Displays a red ripple animation on screen for each click action

## Quick Start

```bash
# Install dependencies
pip install -r requirements.txt

# Launch
python main.py
```

## Usage

1. Click **⚙ Settings**, select an AI provider and enter your API Key
2. Type a task in the input field (e.g., "Open Calculator and compute 1024×768")
3. Click **▶ Start** — the AI will analyze your screen and execute actions
4. Click **⏸ Pause** at any time to suspend the task

## Supported Models

| Provider | Requirements |
|----------|-------------|
| OpenAI | API Key (e.g. gpt-4o) |
| Anthropic | API Key (e.g. claude-3-5-sonnet) |
| Ollama | Server URL (default http://localhost:11434) |
| Custom | API Key + Base URL (any OpenAI-compatible service) |

## Personalization

- **Language** — Switch between 中文 and English in Settings, UI updates immediately
- **Theme** — Dark mode (default) and Light mode supported
- **Font Size** — Adjustable global font size (8–14px)
- **Screenshot Quality** — Controls image compression quality sent to AI (10–100)
- **Action Delay** — Adjusts the interval between AI-executed mouse/keyboard actions

## Project Structure

```
EyeForge/
├── main.py                 # Entry point
├── config.json             # Configuration file
├── requirements.txt        # Dependencies
├── logs/                   # Logs and debug screenshots
└── src/
    ├── core/
    │   ├── screen.py       # Screen capture
    │   ├── actions.py      # Mouse/keyboard control
    │   ├── vision.py       # Image processing
    │   └── agent.py        # AI main loop
    ├── ai/
    │   ├── prompts.py      # System prompts
    │   └── llm_client.py   # LLM client
    └── ui/
        ├── main_window.py  # Main window
        ├── settings_dialog.py # Settings dialog
        └── overlay.py      # Click animation overlay
```

## Tech Stack

- **Python 3.10+**
- **PyQt5** — GUI framework
- **pyautogui** — Mouse and keyboard control
- **mss** — High-speed screen capture
- **Pillow / OpenCV** — Image processing
- **OpenAI / Anthropic SDK** — AI model API calls

## Disclaimer

EyeForge acts solely as a bridge between the AI model and your keyboard and mouse, translating the AI's decisions into actual mouse and keyboard operations. The AI's behavior is determined by its own model and algorithms, not by EyeForge. Users are solely responsible for the outcomes of AI operations. Do not use it on devices you do not own, or leave it running unattended for extended periods.

## Q&A

### Which AI models are supported?
OpenAI (GPT-4o, etc.), Anthropic (Claude 3.5/4, etc.), Ollama (LLaVA and other local models), Gemini, and any OpenAI-compatible custom service.

### Why is a multimodal model required?
EyeForge needs the model to "see" the screen to decide the next action. Non-multimodal models (e.g., GPT-3.5, Claude 3 Haiku) cannot process images.

### How do I configure it?
On first launch, a setup wizard will guide you through: select language → enter API Key and model name → configure capture settings. You can also modify everything later in the Settings dialog.

### How do I update?
Go to Settings → Update tab, click "Check Update"; or visit [GitHub Releases](https://github.com/xiaopi668/EyeForge/releases) / [GitCode Releases](https://gitcode.com/xiaopi668/EyeForge/releases) directly.

### Does it support multiple monitors?
Yes, click animations automatically adapt to all monitors.

### How do I switch API providers?
Go to Settings → AI Model tab, select a different provider, and fill in the corresponding API Key and model name.

### Is it secure?
API keys are encrypted with the `cryptography` library and stored in `config.json` — never in plain text. Do not commit `config.json` to public repositories.

### How do I stop the agent mid-task?
Click the "⏸ Pause" button to stop the current task. You can then continue or enter a new task.

### What screen resolutions are supported?
Any resolution works, including multiple monitors. Operations use ratio coordinates (0~1) and auto-adapt to your actual screen.

### How do I switch the language?
Go to Settings → General tab, select "中文" or "English". It takes effect immediately after closing.

### What does 🟢/⚪ mean?
When fetching model lists, 🟢 indicates the model supports vision (multimodal), ⚪ means it was not recognized as multimodal. 🟢 models are recommended.

### Why is the model list empty?
Check that your API Key is correct and your network is working. Some custom services may not support the `/models` endpoint.

### Can I use multiple API providers at once?
Only one provider can be active at a time. Switch providers in the Settings dialog.

### What if the agent behaves incorrectly?
Check the log panel on the right for error details. Common causes: invalid API Key, non-multimodal model, network timeout.

### How do I reset the configuration?
Close the app, delete `config.json`, and restart. The first-run wizard will appear again.

### How do I use a local model?
Set the provider to Ollama, enter the local URL (default `http://localhost:11434`) and model name (e.g., `llava`). The model will be pulled automatically on first use.
