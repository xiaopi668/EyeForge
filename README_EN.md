# EyeForge 🧿

**AI Screen Control Assistant** — Let AI models see your screen and control your mouse and keyboard.

> [中文](README.md) | English



## Features

- **Screen Awareness** — Captures your screen in real-time and sends it to multimodal AI models for analysis
- **Intelligent Control** — AI autonomously plans steps, controls mouse clicks, movements, and keyboard input
- **Multi-Model Support** — Supports OpenAI, Anthropic, Ollama, and any OpenAI-compatible custom API (requires multimodal models such as gpt-4o, claude-3-5-sonnet, llava, etc.)
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
