# EyeForge 🧿

**AI Screen Control Assistant** — Let AI models see your screen and control your mouse and keyboard.

> [中文](README.md) | English
>
> 📋 [Changelog](CHANGELOG_EN.md) | 📦 [Download](https://github.com/xiaopi668/EyeForge/releases)

## Features

- **Screen Awareness** — Captures your screen in real-time and sends it to multimodal AI models
- **Intelligent Control** — AI autonomously plans steps, controls mouse clicks, movements, and keyboard input
- **Multi-Model Support** — OpenAI, Anthropic, Ollama, Gemini, and any OpenAI-compatible custom API (requires **multimodal** models)
- **GUI** — PyQt5 modern interface with dark/light themes and instant Chinese/English switching
- **Global Hotkeys** — `Ctrl+Shift+E` quick input / `Ctrl+Shift+V` voice input, fully customizable
- **Wake Word** — Picovoice Porcupine offline wake word, no internet needed, zero latency
- **Voice Input** — Google Web Speech real-time transcription
- **Click Animation** — Red ripple effect displayed on screen for each click
- **Encrypted Storage** — API keys encrypted with Fernet + PBKDF2

## Quick Start

```bash
pip install -r requirements.txt
python main.py
```

Or download [EyeForge_Setup.exe](https://github.com/xiaopi668/EyeForge/releases) for one-click installation.

## Usage

### First Launch
The setup wizard guides you through: choose language → configure AI model (enter API Key and model name, fetch model list 🟢 multimodal / ⚪ unknown) → set hotkeys → capture settings.

### Running a Task
Type a task in the input field (e.g., "Open Calculator and compute 1024×768"), click **▶ Start** — the AI analyzes your screen and executes actions. Click **⏸ Pause** at any time.

### Quick Input
- Press `Ctrl+Shift+E` to open the floating input window
- Press `Ctrl+Shift+V` for voice input mode

### Wake Word
Get a free **AccessKey** from [Picovoice Console](https://console.picovoice.ai/) and enter it in Settings → General → Picovoice AccessKey. Enable wake word, say **"Computer"** — the input window appears automatically. See [Wake Word Training Guide](docs/wake-word-training-guide.md) for custom keywords.

### System Tray
Minimize to tray on close. Double-click to restore, right-click for update check or quit.

## Supported Models

| Provider | Requirements |
|----------|-------------|
| OpenAI | API Key (e.g. gpt-4o) |
| Anthropic | API Key (e.g. claude-3-5-sonnet) |
| Ollama | Server URL (default http://localhost:11434) |
| Gemini | API Key (e.g. gemini-2.5-flash) |
| Custom | API Key + Base URL (any OpenAI-compatible service) |

> All providers **require multimodal models** (vision-capable). Non-vision models cannot analyze screenshots.

## Personalization

- **Language** — Switch between 中文 and English instantly
- **Theme** — Dark (default) and Light modes
- **Font Size** — 8–14px adjustable
- **Screenshot Quality** — Compression quality sent to AI (10–100)
- **Action Delay** — Interval between AI-executed actions

## Tech Stack

- **Python 3.10+** | **PyQt5** — GUI
- **pyautogui** — Mouse & keyboard control
- **mss** — High-speed capture
- **Pillow / OpenCV** — Image processing
- **cryptography** — API key encryption
- **pvporcupine** — Offline wake word
- **SpeechRecognition** — Voice transcription

## Project Structure

```
EyeForge/
├── main.py                 # Entry point
├── config.json             # Config (encrypted API keys)
├── requirements.txt        # Dependencies
├── install.bat             # Environment setup script
├── start.bat / start.exe   # Launcher
├── logs/                   # Logs and debug screenshots
├── src/
│   ├── version.py          # Version number
│   ├── logo.ico            # App icon
│   ├── core/
│   │   ├── screen.py       # Screen capture
│   │   ├── actions.py      # Mouse/keyboard control
│   │   ├── vision.py       # Image processing
│   │   └── agent.py        # AI main loop
│   ├── ai/
│   │   ├── prompts.py      # System prompts (bilingual)
│   │   └── llm_client.py   # LLM client (5 providers)
│   ├── ui/
│   │   ├── main_window.py  # Main window
│   │   ├── settings_dialog.py # Settings dialog
│   │   ├── float_window.py # Quick input floating window
│   │   ├── wizard.py       # First-run wizard
│   │   └── overlay.py      # Click animation overlay
│   └── utils/
│       ├── crypto.py       # Fernet encryption
│       ├── hotkey.py       # Global hotkeys (RegisterHotKey)
│       ├── voice.py        # Speech recognition
│       ├── wakeword.py     # Wake word detection
│       ├── updater.py      # Update checker
│       └── multimodal.py   # Multimodal model detection
└── docs/
    ├── 唤醒词训练教程.md
    └── wake-word-training-guide.md
```

## Q&A

#### Which AI models are supported?
OpenAI (GPT-4o etc.), Anthropic (Claude 3.5/4 etc.), Ollama (LLaVA etc.), Gemini, and any OpenAI-compatible custom service.

#### Why multimodal?
The model needs to "see" screenshots. Non-multimodal models (GPT-3.5, Claude 3 Haiku) cannot process images.

#### Are API keys secure?
Encrypted with `cryptography` (Fernet + PBKDF2) in `config.json` — never stored in plain text. Do not commit `config.json`.

#### How to update?
Settings → Update tab, or visit [GitHub Releases](https://github.com/xiaopi668/EyeForge/releases).

#### How to reset config?
Delete `config.json` while the app is closed, then restart.

#### Multiple monitors?
Yes. Operations use ratio coordinates (0~1), click animation adapts to all displays.

#### What does 🟢/⚪ mean?
🟢 = multimodal (vision-capable), ⚪ = not recognized as multimodal. 🟢 recommended.

#### How to use local models?
Set provider to Ollama, enter local URL and model name (e.g., `llava`). Model pulled on first use.

## Disclaimer

EyeForge acts as a bridge between AI and your keyboard/mouse. The AI's behavior is determined by its own model and algorithms. Users are solely responsible for outcomes. Do not use on devices you do not control or leave running unattended.
