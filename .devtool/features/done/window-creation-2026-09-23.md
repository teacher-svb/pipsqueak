---
id: "window-creation-2026-09-23"
status: "done"
priority: "medium"
assignee: null
epic: "borderless window"
dueDate: null
created: "2026-09-23T07:32:08.545Z"
modified: "2026-09-23T07:50:46.000Z"
completedAt: "2026-09-23T07:50:46.000Z"
labels: []
order: "a0"
---
# Window creation

Create a window on Linux/Wayland

Done via `winit` 0.30 (`ApplicationHandler`/`create_window`), run inside a devcontainer since the host had no Rust toolchain. A `softbuffer`-filled buffer was required for the Wayland compositor to map the surface at all — see [main.rs](../../../src/main.rs). Verified live under KWin/Wayland.