---
id: "display-selection-ui-2026-09-23"
status: "done"
priority: "medium"
assignee: null
epic: "display capture"
dueDate: null
created: "2026-09-23T07:35:40.000Z"
modified: "2026-09-23T10:05:00.000Z"
completedAt: "2026-09-23T10:05:00.000Z"
labels: ["ux"]
order: "b4"
---
# Display selection & end-to-end capture milestone

Tie [[portal-screencast-session-2026-09-23]], [[pipewire-stream-consume-2026-09-23]] and [[frame-render-pipeline-2026-09-23]] together into the working "display capture" milestone.

- On launch, trigger the portal's monitor picker (compositor-native UI) so the user picks which display to mirror
- Re-request selection on demand (e.g. a hotkey or menu item) without restarting the app
- Verify full path: pick monitor -> PipeWire stream -> rendered live in the borderless window
- Manual test on at least one Wayland compositor (e.g. Sway or GNOME)

**Display capture milestone verified end-to-end**, live: KDE's monitor picker appears on launch, the selected monitor's PipeWire stream connects, and it renders in the borderless window - confirmed by the user ("Yes, it's showing my screen"). A second run reused the saved `restore_token` and skipped the picker entirely.

The "re-request selection on demand" bullet (a hotkey/menu item to switch monitors without restarting) is *not* implemented - currently the only way to change monitor is to delete `.pip-restore-token` and restart. Splitting that off as its own follow-up task ([[reselect-monitor-hotkey-2026-09-23]]) rather than blocking this milestone on it, since the core ask ("capturing and showing a stream of a connected display") is done. Only tested on KDE/KWin so far, not Sway/GNOME.
