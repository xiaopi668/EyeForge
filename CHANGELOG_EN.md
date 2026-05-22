# Changelog

| Date | Type | Description |
|------|------|-------------|
| 2026-05-22 | Release | **v1.1 Stable** — Multi-platform channels + stability fixes |
| 2026-05-22 | Added | **Unified Channels panel** — WebSocket, WeChat (iLink), WeCom, DingTalk, QQ all in one settings tab |
| 2026-05-22 | Added | **Native WeChat iLink client** — Removed OpenClaw dependency, direct iLink Bot API, QR code login |
| 2026-05-22 | Added | **QR code login** — Built-in QR display for WeChat channel, no external tools needed |
| 2026-05-22 | Fixed | **Settings window flicker** — Off-screen construction + QTimer centering, replacing opacity/updatesEnabled hacks |
| 2026-05-22 | Fixed | **System tray icon disappearing** — Removed QSystemTrayIcon parent, setQuitOnLastWindowClosed=False, explicit show() on minimize, QMenu instance attribute to prevent GC, `_init_tray()` was never called |
