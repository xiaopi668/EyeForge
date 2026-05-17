# EyeForge 🧿

**AI 屏幕操控助手** — 让大模型看到你的屏幕，并替你操作鼠标和键盘。

> 中文 | [English](README_EN.md)



## 功能

- **屏幕感知** — 实时截取屏幕画面，发送给多模态大模型分析
- **智能操控** — AI 自动规划操作步骤，控制鼠标点击、移动、键盘输入
- **多模型支持** — 支持 OpenAI、Anthropic、Ollama 以及任意兼容 OpenAI API 的自定义服务（需使用多模态模型，如 gpt-4o、claude-3-5-sonnet、llava 等）
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
