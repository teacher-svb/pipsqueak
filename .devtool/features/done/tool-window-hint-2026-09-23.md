---
id: "tool-window-hint-2026-09-23"
status: "done"
priority: "medium"
assignee: null
epic: "borderless window"
dueDate: null
created: "2026-09-23T08:05:00.000Z"
modified: "2026-09-23T08:10:00.000Z"
completedAt: "2026-09-23T08:10:00.000Z"
labels: ["wayland", "wm"]
order: "a4"
---
# Tool/utility window hint for tiling WMs

Make tiling window managers treat the window as floating rather than tiling it, building on [[window-creation-2026-09-23]] and [[borderless-decorations-2026-09-23]].

Wayland has no client-settable window-type property equivalent to X11's `_NET_WM_WINDOW_TYPE_UTILITY` — floating vs. tiling is compositor/user-config policy. Implemented the two portable hints available:

- Fixed size (`with_min_inner_size` == `with_max_inner_size`, 480x270): sway/hyprland/i3 auto-float windows that declare a fixed size. Side effect: window is no longer resizable.
- Stable `app_id`/WM_CLASS via `with_name("pip", "pip")`, so a WM rule can target it explicitly (e.g. sway: `for_window [app_id="pip"] floating enable`).
- `with_x11_window_type(vec![WindowType::Utility])` as a fallback hint in case winit ever runs under XWayland/X11 instead of native Wayland.

See [main.rs](../../../src/main.rs). Verified under KWin + Krohnkite (tiling script): window floats instead of tiling, and still shows/drags/closes correctly. Not tested on sway/hyprland/river — the fixed-size heuristic should cover those too, but unconfirmed.
