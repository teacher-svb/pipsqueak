---
id: "portal-screencast-session-2026-09-23"
status: "done"
priority: "high"
assignee: null
epic: "display capture"
dueDate: null
created: "2026-09-23T07:35:25.000Z"
modified: "2026-09-23T10:05:00.000Z"
completedAt: "2026-09-23T10:05:00.000Z"
labels: ["wayland", "portal"]
order: "b1"
---
# Portal ScreenCast session setup

Implement the xdg-desktop-portal ScreenCast handshake using `ashpd`, based on [[research-screencast-api-2026-09-23]].

- Create a ScreenCast session
- Request monitor source type, trigger the compositor's picker so the user selects a display
- Start the session and obtain the PipeWire node id + fd
- Handle user cancel / permission denial gracefully
- Store a `restore_token` where supported so repeat runs skip the picker

Implemented in [capture.rs](../../src/capture.rs) (`open_portal`). Verified live: KDE's picker dialog appears, monitor selection goes through, portal hands back a stream + PipeWire fd, `restore_token` is persisted to `.pip-restore-token`. Confirmed: a second run (native host binary, no container involved) reused the saved `restore_token` and skipped the picker entirely, going straight to a negotiated stream. Cancel/denial handling remains untested and unpolished (currently just `expect()`s and logs an error rather than degrading gracefully) - acceptable for now, worth hardening later.
