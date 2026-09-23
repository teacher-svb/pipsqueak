---
id: "research-screencast-api-2026-09-23"
status: "done"
priority: "high"
assignee: null
epic: "display capture"
dueDate: null
created: "2026-09-23T07:35:20.000Z"
modified: "2026-09-23T09:15:00.000Z"
completedAt: "2026-09-23T09:15:00.000Z"
labels: ["research", "wayland"]
order: "b0"
---
# Research screen capture API for Wayland

Wayland has no direct client screen-grab API; capture goes through the `xdg-desktop-portal` ScreenCast portal, which hands back a PipeWire stream.

- Confirm `org.freedesktop.portal.ScreenCast` D-Bus interface flow: `CreateSession` -> `SelectSources` -> `Start` -> PipeWire node id
- Check portal backend availability on target compositors (e.g. `xdg-desktop-portal-wlr`, `-gnome`, `-kde`)
- Evaluate `ashpd` crate as the Rust wrapper for this D-Bus flow
- Evaluate `pipewire-rs` for consuming the resulting stream
- Note any permission/dialog prompts the user will see and whether a `restore_token` can skip repeat prompts
- Document findings to inform [[portal-screencast-session-2026-09-23]]

Confirmed via ashpd's own upstream example (`client/examples/screen_cast_pw.rs`) and docs.rs source: `ashpd::desktop::screencast::Screencast` (`create_session` -> `select_sources` -> `start` -> `open_pipe_wire_remote`) gives a PipeWire node id + fd; `pipewire` crate 0.10.1 (`MainLoopBox`/`ContextBox`/`StreamBox`) consumes it. `restore_token` + `PersistMode::ExplicitlyRevoked` skips the picker on repeat runs. Backend confirmed working: xdg-desktop-portal-kde (user's KDE Plasma/KWin session).
