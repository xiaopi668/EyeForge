# Changelog

| Date | Type | Description |
|------|------|-------------|
| 2026-05-22 | Release | **v1.5.0-beta.1** — OpenClaw-compatible skills system + float window chat UI + major stability fixes |
| 2026-05-22 | Added | **OpenClaw-compatible skills system** — SKILL.md + reference.md + examples/ + scripts/ directory structure, ZIP import, 6 built-in skills |
| 2026-05-22 | Added | **Settings → Skills** — skill list with enable/disable toggle, ZIP import, edit any file in skill directory, delete |
| 2026-05-22 | Added | **Skill injection into AI context** — SKILL.md instructions, reference.md docs, examples all injected into system prompt; AI calls via `{"type":"skill","name":"...","script":"..."}` |
| 2026-05-22 | Added | **Float window chat UI** — message bubbles (user green right/AI gray left), collapsible shell items |
| 2026-05-22 | Added | **Float window standalone agent thread** — runs its own EyeForgeAgent, no longer delegates to main window |
| 2026-05-22 | Added | **Float window red error bubbles** — API errors shown as red message bubbles |
| 2026-05-22 | Added | **Channel messages use full agent** — WeChat/WebSocket handlers use EyeForgeAgent pipeline, only final result exposed |
| 2026-05-22 | Added | **QQ channel config optimized** — go-cqhttp (WebSocket) vs Official Bot API in QGroupBox, dynamic show/hide |
| 2026-05-22 | Fixed | **Main thread freezing** — screenshot signal changed to bytes, QR login uses pyqtSignal for thread-safe UI updates |
| 2026-05-22 | Fixed | **Lazy ScreenCapture init** — MSS deferred to first capture_pil() call, no more __init__ lag |
| 2026-05-22 | Fixed | **Fresh agent per task** — removed continuation logic to prevent context overflow |
| 2026-05-22 | Fixed | **API rate limit GUI freeze** — OpenAI client max_retries=0, errors returned immediately |
| 2026-05-22 | Fixed | **Float window white box / truncated text** — transparent QScrollArea viewport, hidden empty status label, bubble max-width 400 |
| 2026-05-22 | Fixed | **Settings dialog crash** — restored overwritten _update_tab method, added missing import os |
