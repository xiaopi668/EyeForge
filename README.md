# EyeForge

纯 Rust 的 AI 桌面助手与网关。

当前入口：
- 桌面端：`src-rs/target/debug/eye-forge-rs.exe`
- Web UI：`http://127.0.0.1:9178/`
- WebSocket：`ws://127.0.0.1:9178/ws`

## 现状

- Python 运行后端已移除，项目改为 Rust 原生执行链
- Rust 桌面端负责配置、任务输入、执行日志和网关状态
- Rust 网关在 `9178` 端口同时托管 Web UI 和 WebSocket API
- Rust runtime 已支持：
  - `shell`
  - `wait`
  - `open`
  - `click`
  - `type`
  - `hotkey`
  - `scroll`
  - `screenshot`
  - `complete`
- 普通自然语言任务会先走 Rust 原生 LLM 规划，再编译成动作队列执行

## 快速开始

需要先安装 Rust 工具链：
- [https://rustup.rs/](https://rustup.rs/)

### 安装 / 构建

```powershell
install.bat
```

### 启动

```powershell
start.bat
```

启动后：
- 桌面窗口会打开
- 网关会监听 `http://127.0.0.1:9178/`

## WebSocket 协议

连接地址：
- `ws://127.0.0.1:9178/ws`

消息流：
1. `auth`
2. `task`
3. `result`

示例：

```json
{
  "type": "task",
  "task": "{\"actions\":[{\"type\":\"wait\",\"seconds\":0.5},{\"type\":\"complete\",\"result\":\"ok\"}]}"
}
```

## 仓库结构

```text
EyeForge/
├── src-rs/               # Rust 桌面端、网关、运行时
├── web-ui/               # Rust 网关托管的浏览器前端
├── docs/                 # 文档
├── install.bat           # Rust 构建脚本
├── start.bat             # Rust 启动脚本
├── setup_stub.cs         # 安装器 stub
├── CHANGELOG.md
└── README_EN.md
```

## 说明

当前项目已经脱离 Python 后端，但并不意味着所有历史功能都已 1:1 迁完。

还会继续迁移和增强的部分：
- 更完整的 LLM 规划链
- 视觉理解
- 更多桌面自动化动作
- 语音 / 唤醒词
- 多平台通道桥
