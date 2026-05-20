# EyeForge 🧿

**AI 屏幕操控助手** — 让大模型看到你的屏幕，并替你操作鼠标和键盘。

> 中文 | [English](README_EN.md)
>
> 📋 [更新日志](CHANGELOG.md) | 📦 [下载](https://github.com/xiaopi668/EyeForge/releases)

## 功能

- **屏幕感知** — 实时截取屏幕，发送给多模态大模型分析
- **智能操控** — AI 自动规划步骤，控制鼠标点击、移动、键盘输入
- **多模型支持** — OpenAI、Anthropic、Ollama、Gemini，及任意兼容 OpenAI API 的自定义服务（需**多模态**模型）
- **图形界面** — PyQt5 现代 GUI，深色/浅色主题，中/英双语即时切换
- **全局热键** — `Ctrl+Shift+E` 快捷输入 / `Ctrl+Shift+V` 语音输入，可自定义
- **语音唤醒** — Picovoice Porcupine 离线唤醒，无需联网，零延迟
- **语音输入** — Google Web Speech 实时转写
- **点击可视化** — 每次点击在屏幕上显示红色光圈动画
- **加密存储** — API Key 使用 Fernet + PBKDF2 加密保存

## 快速开始

```bash
# 安装依赖
pip install -r requirements.txt

# 启动
start.exe
```

或下载 [EyeForge_Setup.exe](https://github.com/xiaopi668/EyeForge/releases) 一键安装。

## 使用流程

### 首次启动
运行后弹出设置向导：选择语言 → 配置 AI 模型（填写 API Key 和模型名，可拉取模型列表 🟢 多模态 / ⚪ 未知） → 设置快捷键 → 截屏参数。

### 执行任务
在输入框描述任务（如"打开计算器并计算 1024×768"），点击 **▶ 开始执行**，AI 将自动分析屏幕并执行操作。可随时点击 **⏸ 暂停执行**。

### 快捷输入
- 按 `Ctrl+Shift+E` 弹出悬浮窗，输入任务回车发送
- 按 `Ctrl+Shift+V` 弹出悬浮窗并开始语音识别

### 语音唤醒
需先到 [Picovoice Console](https://console.picovoice.ai/) 注册获取 **AccessKey**，填入设置 → 常规 → Picovoice AccessKey。开启语音唤醒后，说 **"Computer"** 即可弹出输入窗并录音。自定义唤醒词训练详见 [唤醒词训练教程](docs/唤醒词训练教程.md)。

### 系统托盘
关闭窗口最小化到托盘，双击恢复，右键可检查更新或退出。

## 支持的模型

| 提供商 | 配置要求 |
|--------|---------|
| OpenAI | API Key（如 gpt-4o） |
| Anthropic | API Key（如 claude-3-5-sonnet） |
| Ollama | 服务地址（默认 http://localhost:11434） |
| Gemini | API Key（gemini-2.5-flash 等） |
| Custom | API Key + Base URL（任意 OpenAI 兼容服务） |

> 所有提供商**必须使用多模态模型**（能识别图片），非多模态模型无法分析屏幕截图。

## 个性化设置

- **语言** — 设置中切换 中文 / English，即时生效
- **主题** — 深色模式（默认）和浅色模式
- **字体大小** — 8–14px 可调
- **截图质量** — 控制发送给 AI 的图片压缩质量（10–100）
- **操作延迟** — 调整 AI 执行操作的间隔

## 技术栈

- **Python 3.10+** | **PyQt5** — GUI
- **pyautogui** — 鼠标键盘控制
- **mss** — 高速截图
- **Pillow / OpenCV** — 图像处理
- **cryptography** — API Key 加密
- **pvporcupine** — 离线语音唤醒
- **SpeechRecognition** — 语音转写

## 项目结构

```
EyeForge/
├── main.py                 # 入口
├── config.json             # 配置（API Key 加密存储）
├── requirements.txt        # 依赖
├── install.bat             # 环境安装脚本
├── start.bat / start.exe   # 启动脚本
├── logs/                   # 日志和调试截图
├── src/
│   ├── version.py          # 版本号
│   ├── logo.ico            # 应用图标
│   ├── core/
│   │   ├── screen.py       # 屏幕截图
│   │   ├── actions.py      # 鼠标/键盘控制
│   │   ├── vision.py       # 图像处理
│   │   └── agent.py        # AI 主循环
│   ├── ai/
│   │   ├── prompts.py      # 系统提示词（中英双语）
│   │   └── llm_client.py   # LLM 客户端（5 种提供商）
│   ├── ui/
│   │   ├── main_window.py  # 主窗口
│   │   ├── settings_dialog.py # 设置对话框
│   │   ├── float_window.py # 快捷输入悬浮窗
│   │   ├── wizard.py       # 首次运行向导
│   │   └── overlay.py      # 点击动画
│   └── utils/
│       ├── crypto.py       # Fernet 加密
│       ├── hotkey.py       # 全局热键（RegisterHotKey）
│       ├── voice.py        # 语音识别
│       ├── wakeword.py     # 语音唤醒（Porcupine）
│       ├── updater.py      # 更新检查
│       └── multimodal.py   # 多模态模型检测
└── docs/
    ├── 唤醒词训练教程.md
    └── wake-word-training-guide.md
```

## Q&A

#### 支持哪些 AI 模型？
OpenAI（GPT-4o 等）、Anthropic（Claude 3.5/4 等）、Ollama（LLaVA 等本地模型）、Gemini、以及兼容 OpenAI 接口的自定义服务。

#### 为什么必须用多模态模型？
EyeForge 需要模型能"看懂"屏幕截图。非多模态模型（如 GPT-3.5、Claude 3 Haiku）无法处理图片。

#### API Key 安全吗？
使用 `cryptography`（Fernet + PBKDF2）加密后存入 `config.json`，永不明文保存。请勿将 `config.json` 提交到公开仓库。

#### 如何更新？
设置 → 更新标签页检查更新，或访问 [GitHub Releases](https://github.com/xiaopi668/EyeForge/releases)。

#### 如何重置配置？
关闭程序后删除 `config.json`，下次启动重新弹出向导。

#### 支持多显示器吗？
支持。操作使用比例坐标（0~1），点击动画自动适配所有屏幕。

#### 如何中途停止任务？
点击 **⏸ 暂停执行** 即可停止，之后可继续或输入新任务。

#### 🟢/⚪ 标注是什么意思？
拉取模型列表时，🟢 表示支持多模态（可识图），⚪ 表示未识别为多模态。建议优先使用 🟢 模型。

#### 如何使用本地模型？
提供商设为 Ollama，填写本地地址和模型名（如 `llava`），首次使用自动拉取模型。

## 免责声明

EyeForge 仅作为 AI 与您键盘鼠标之间的桥梁。AI 的行为由其自身的模型和算法决定。使用者应对 AI 的操作结果自行负责，请勿在不受控制的设备上使用或无人值守时长时间运行。
