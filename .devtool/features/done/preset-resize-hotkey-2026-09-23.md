---
id: "preset-resize-hotkey-2026-09-23"
status: "done"
priority: "medium"
assignee: null
epic: "borderless window"
dueDate: null
created: "2026-09-23T10:30:00.000Z"
modified: "2026-09-23T10:45:00.000Z"
completedAt: "2026-09-23T10:45:00.000Z"
labels: ["wayland", "ux"]
order: "a5"
---
# Resize via app-specific shortcut, not drag-resize

Free drag-resize and reliable auto-floating on tiling WMs turned out to be mutually exclusive on Wayland: every tiling tool checked (sway, hyprland, i3's documented heuristic, and Krohnkite/KWin empirically) floats a window based on it declaring a fixed size (`min == max`); a window that's ever freely resizable loses that signal everywhere, not just on one WM. [[tool-window-hint-2026-09-23]]'s X11-utility-type hint turned out to be dead code here too - it only applies over XWayland, and this app runs native Wayland.

So instead of drag-resize: `+`/`-` step through a fixed list of 16:9 presets (320x180 up to 960x540). The window is always non-resizable (`min == max`) at whatever the current preset is, so it keeps floating at every step.

- Match on the *logical* key (`KeyEvent::logical_key`, the produced character), not the physical one - physical-key matching (`PhysicalKey::Code(KeyCode::Equal/Minus)`) put `+`/`-` on the wrong physical keys for non-US layouts (confirmed broken on the user's AZERTY layout, where US `Equal`/`Minus` physical positions don't produce `+`/`-`)
- `window.set_min_inner_size` / `set_max_inner_size` are generic (`Option<S: Into<Size>>`); passing `Some(x.into())` is ambiguous (nothing constrains the target of `.into()`) - pass the concrete `LogicalSize` directly instead

Implemented in [main.rs](../../../src/main.rs) (`SIZE_PRESETS`, `apply_preset`, the `WindowEvent::KeyboardInput` arm). Verified live on the user's AZERTY layout: `+`/`-` step through presets correctly, window stays floating throughout under KWin/Krohnkite.
