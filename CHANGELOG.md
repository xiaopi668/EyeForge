# 更新日志

| 日期 | 类型 | 内容 |
|------|------|------|
| 2026-05-17 | 修复 | **未知模型弹窗按钮失效** — `setDefaultText()` → `setDefaultButton()`，三按钮（继续使用/更换模型/取消）正常显示 |
| 2026-05-17 | 修复 | **Windows 任务栏图标不显示** — `main.py` 添加 `SetCurrentProcessExplicitAppUserModelID` |
| 2026-05-17 | 新增 | **设置保存时非多模态模型警告** — `_save()` 中检查模型，弹窗可选继续保存或取消 |
| 2026-05-17 | 修复 | **任务完成后 Agent 未停止** — `execute_action` 兼容字符串 action（如 `"complete"`），提示强调 action 必须为对象 |
