---
id: "borderless-decorations-2026-09-23"
status: "done"
priority: "high"
assignee: null
epic: "borderless window"
dueDate: null
created: "2026-09-23T07:35:10.000Z"
modified: "2026-09-23T07:50:46.000Z"
completedAt: "2026-09-23T07:50:46.000Z"
labels: ["wayland"]
order: "a2"
---
# Borderless window decorations

Make the window created in [[window-creation-2026-09-23]] borderless (no title bar, no server-side decorations).

- Disable window decorations at creation time
- Verify no compositor-drawn border/titlebar appears under a Wayland compositor (e.g. Sway, GNOME/Mutter)
- Window should still be resizable/positionable programmatically even without decorations

Done via `WindowAttributes::with_decorations(false)` in [main.rs](../../../src/main.rs). Verified no titlebar/border under KWin/Wayland.
