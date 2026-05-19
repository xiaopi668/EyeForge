# Changelog

| Date | Type | Description |
|------|------|-------------|
| 2026-05-19 | Release | **v1.0 Stable** — Full feature coverage |
| 2026-05-19 | Added | **Wake word detection** — Picovoice Porcupine offline wake word, built-in/custom keywords, requires AccessKey |
| 2026-05-19 | Added | **Floating quick-input window** — `Ctrl+Shift+E` opens a floating input box with voice button |
| 2026-05-19 | Added | **Global hotkeys** — Windows RegisterHotKey, no admin required, customizable combos |
| 2026-05-19 | Added | **Voice input** — Google Web Speech real-time transcription, triggered via 🎤 button |
| 2026-05-19 | Added | **Installer** — `EyeForge_Setup.exe` with one-click setup and optional desktop shortcut |
| 2026-05-19 | Added | **Encrypted API key storage** — Fernet + PBKDF2 encryption, never plaintext in config.json |
| 2026-05-19 | Added | **Update checker** — Check GitHub/GitCode for latest release in Settings |
| 2026-05-19 | Added | **First-run wizard** — 4-step guide: language → model → hotkeys → capture settings |
| 2026-05-19 | Added | **Non-multimodal model warning** — Detection and warning on task start and settings save |
| 2026-05-17 | Added | **Installer** — `EyeForge_Setup.exe` with one-click setup and optional desktop shortcut |
| 2026-05-17 | Added | **Picovoice AccessKey support** — Wake word now requires an AccessKey in Settings; supports built-in and custom wake words |
| 2026-05-17 | Fixed | **Unknown model warning buttons broken** — `setDefaultText()` → `setDefaultButton()`, three buttons (Continue Anyway / Change Model / Cancel) now work correctly |
| 2026-05-17 | Fixed | **Windows taskbar icon not showing** — Added `SetCurrentProcessExplicitAppUserModelID` in `main.py` |
| 2026-05-17 | Added | **Non-multimodal model warning on settings save** — `_save()` checks model, shows popup with Save Anyway / Cancel options |
| 2026-05-17 | Fixed | **Agent not stopping after task completion** — `execute_action` now handles string actions (e.g. `"complete"`), prompts emphasize action must be an object |
