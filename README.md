# EyeForge 🧿

**AI 屏幕操控助手** — 让大模型看到你的屏幕，并替你操作鼠标和键盘。

> 中文 | [English](README_EN.md)
>
> 📋 [更新日志](CHANGELOG.md) | 📦 [GitHub Releases](https://github.com/xiaopi668/EyeForge/releases) | 📦 [GitCode Releases](https://gitcode.com/xiaopi668/EyeForge/releases)



## 功能

- **屏幕感知** — 实时截取屏幕画面，发送给多模态大模型分析
- **智能操控** — AI 自动规划操作步骤，控制鼠标点击、移动、键盘输入
- **多模型支持** — 支持 OpenAI、Anthropic、Ollama 以及任意兼容 OpenAI API 的自定义服务（需使用**多模态**模型，如 gpt-4o、claude-3-5-sonnet、llava 等）
- **图形界面** — 基于 PyQt5 的现代 GUI，支持深色/浅色主题切换
- **国际化** — 界面支持中文和 English，一键切换
- **点击可视化** — 每次点击在屏幕上显示红色光圈动画

## 快速开始

```bash
# 安装依赖
pip install -r requirements.txt

# 启动
python main.py
```

## 使用流程

1. 点击 **⚙ 设置**，选择 AI 模型提供商并填入 API Key
2. 在输入框中描述任务（如"打开计算器并计算 1024×768"）
3. 点击 **▶ 开始执行**，AI 将自动分析屏幕并执行操作
4. 可随时点击 **⏸ 暂停执行** 暂停任务

## 支持的模型

| 提供商 | 配置要求 |
|--------|---------|
| OpenAI | API Key（模型如 gpt-4o） |
| Anthropic | API Key（模型如 claude-3-5-sonnet） |
| Ollama | 服务地址（默认 http://localhost:11434） |
| Custom | API Key + Base URL（任何兼容 OpenAI API 的服务） |

## 个性化设置

- **语言** — 设置中切换 中文 / English，界面文字即时生效
- **主题** — 支持深色模式（默认）和浅色模式
- **字体大小** — 可调节全局字体大小（8–14px）
- **截图质量** — 控制发送给 AI 的图片压缩质量（10–100）
- **操作延迟** — 调整 AI 执行鼠标/键盘操作间的间隔
## v0.5.1版本更新日志
    [v0.5.0...v0.5.1](CHANGELOG.md)
## 项目结构

```
EyeForge/
├── main.py                 # 入口
├── config.json             # 配置文件
├── requirements.txt        # 依赖
├── logs/                   # 日志和调试截图
└── src/
    ├── core/
    │   ├── screen.py       # 屏幕截图
    │   ├── actions.py      # 鼠标/键盘控制
    │   ├── vision.py       # 图像处理
    │   └── agent.py        # AI 主循环
    ├── ai/
    │   ├── prompts.py      # 系统提示词
    │   └── llm_client.py   # LLM 客户端
    └── ui/
        ├── main_window.py  # 主窗口
        ├── settings_dialog.py # 设置对话框
        └── overlay.py      # 点击动画
```

## 技术栈

- **Python 3.10+**
- **PyQt5** — 图形界面
- **pyautogui** — 鼠标键盘控制
- **mss** — 高速屏幕截图
- **Pillow / OpenCV** — 图像处理
- **OpenAI / Anthropic SDK** — AI 模型调用

## 免责声明

EyeForge 仅作为 AI 与您的键盘鼠标之间的连接桥梁，将 AI 的决策转化为实际的鼠标和键盘操作。AI 的行为由其自身的模型和算法决定，与 EyeForge 无关。使用者应对 AI 的操作结果自行负责，请勿在不受你控制的设备上使用，或在无人值守时长时间运行。

## Q&A

#### 支持哪些 AI 模型？
OpenAI（GPT-4o 等）、Anthropic（Claude 3.5/4 等）、Ollama（LLaVA 等本地模型）、Gemini、以及任何兼容 OpenAI 接口的自定义服务。

#### 为什么需要多模态模型？
EyeForge 需要模型能"看懂"屏幕截图才能决定下一步操作。非多模态模型（如 GPT-3.5、Claude 3 Haiku）无法处理图片。

#### 如何配置？
首次启动会弹出向导，选择语言 → 填写 API Key 和模型名 → 设置截屏参数。也可以在设置对话框中随时修改。

#### 如何更新？
设置 → 更新标签页，点击检查更新；或直接访问 [GitHub Releases](https://github.com/xiaopi668/EyeForge/releases) / [GitCode Releases](https://gitcode.com/xiaopi668/EyeForge/releases)。

#### 可以多显示器吗？
可以，点击动画会自动适配所有显示器。

#### 如何更换 API 提供商？
设置 → AI 模型标签页，选择不同的提供商，填写对应的 API Key 和模型名即可。

#### 是否安全？
API 密钥使用 `cryptography` 库加密存储在 `config.json` 中，不会明文保存。请勿将 `config.json` 提交到公开仓库。

#### 如何中途停止任务？
点击「⏸ 暂停执行」按钮即可停止当前任务，之后可以继续或输入新任务。

#### 支持哪些屏幕分辨率？
任意分辨率均可，支持多显示器。操作使用比例坐标（0~1），自动适配实际屏幕。

#### 如何切换语言？
设置 → 常规标签页，选择「中文」或「English」，关闭设置后立即生效。

#### 🟢/⚪ 标注是什么意思？
拉取模型列表时，🟢 表示该模型支持多模态（可识图），⚪ 表示未识别为多模态模型。建议优先使用 🟢 模型。

#### 为什么拉取模型列表为空？
请检查 API Key 是否正确、网络是否通畅。部分自定义服务可能不兼容 `/models` 接口。

#### 可以同时使用多个 API 提供商吗？
同一时间只能使用一个提供商。如需更换，在设置中选择其他提供商即可。

#### 任务执行异常怎么办？
查看右侧日志面板了解具体错误信息。常见原因：API Key 无效、模型不支持多模态、网络超时。

#### 如何重置配置？
关闭程序后，删除 `config.json` 文件，下次启动会重新弹出首次运行向导。

#### 如何使用本地模型？
设置提供商为 Ollama，填写本地地址（默认 `http://localhost:11434`）和模型名（如 `llava`），首次使用会自动拉取模型。
