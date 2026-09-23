---
id: "window-drag-move-2026-09-23"
status: "done"
priority: "high"
assignee: null
epic: "borderless window"
dueDate: null
created: "2026-09-23T07:35:15.000Z"
modified: "2026-09-23T07:50:46.000Z"
completedAt: "2026-09-23T07:50:46.000Z"
labels: ["wayland", "input"]
order: "a3"
---
# Move window by dragging

Since the window has no decorations ([[borderless-decorations-2026-09-23]]), the compositor gives no titlebar to drag. Implement client-driven move.

- Listen for mouse button-down anywhere on the window
- On drag, request an interactive move from the compositor (xdg_toplevel `move`, exposed by winit as `Window::drag_window()`)
- Confirm this works under Wayland (no fallback needed there; X11 requires a different path, out of scope for now)
- Window should stop moving on button release

Done via `Window::drag_window()` on left-button-press in [main.rs](../../../src/main.rs). Verified live: click-drag moves the window under KWin/Wayland.
