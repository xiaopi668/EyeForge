# Changelog

| Date | Type | Description |
|------|------|-------------|
| 2026-05-23 | Release | **v1.5.0-beta.2** — Sidebar UI + Settings embedded in main window |
| 2026-05-23 | Added | **Main window sidebar** — QListWidget (56px) with 🏠 Home and ⚙ Settings icons, accent highlight for active page |
| 2026-05-23 | Added | **Settings embedded mode** — Settings page embedded in main window's QStackedWidget, no longer a separate modal dialog |
| 2026-05-23 | Added | **SettingsDialog.embedded** — hidden cancel button, save uses `saved` signal instead of `accept()`, notifies parent via signal |
| 2026-05-23 | Changed | **Home toolbar** — standalone settings button removed, replaced by sidebar ⚙ |
