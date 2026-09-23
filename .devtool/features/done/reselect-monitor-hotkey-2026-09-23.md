---
id: "reselect-monitor-hotkey-2026-09-23"
status: "done"
priority: "low"
assignee: null
epic: "display capture"
dueDate: null
created: "2026-09-23T10:05:00.000Z"
modified: "2026-09-23T12:15:00.000Z"
completedAt: "2026-09-23T12:15:00.000Z"
labels: ["ux"]
order: "b5"
---
# Re-select monitor without restarting

Split off from [[display-selection-ui-2026-09-23]]: right now the only way to change which monitor is mirrored is to delete `.pip-restore-token` and restart the app.

- Add a hotkey or menu item that re-triggers the portal's monitor picker while the app is running
- Tear down the current PipeWire stream/session cleanly before starting the new one
- Update the saved `restore_token` to the new selection

Implemented as [[switch-monitor-hotkey-2026-09-23]] - a process re-exec on `Tab` rather than an in-process restart, after the in-process approach hit an apparent xdg-desktop-portal-kde limitation. See that task for the full story.
