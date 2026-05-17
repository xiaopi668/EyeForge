# Changelog

| Date | Type | Description |
|------|------|-------------|
| 2026-05-17 | Fixed | **Unknown model warning buttons broken** — `setDefaultText()` → `setDefaultButton()`, three buttons (Continue Anyway / Change Model / Cancel) now work correctly |
| 2026-05-17 | Fixed | **Windows taskbar icon not showing** — Added `SetCurrentProcessExplicitAppUserModelID` in `main.py` |
| 2026-05-17 | Added | **Non-multimodal model warning on settings save** — `_save()` checks model, shows popup with Save Anyway / Cancel options |
| 2026-05-17 | Fixed | **Agent not stopping after task completion** — `execute_action` now handles string actions (e.g. `"complete"`), prompts emphasize action must be an object |
