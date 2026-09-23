---
id: "switch-monitor-hotkey-2026-09-23"
status: "done"
priority: "medium"
assignee: null
epic: "display capture"
dueDate: null
created: "2026-09-23T11:00:00.000Z"
modified: "2026-09-23T12:15:00.000Z"
completedAt: "2026-09-23T12:15:00.000Z"
labels: ["ux", "portal"]
order: "b6"
---
# Switch monitor via shortcut

Supersedes [[reselect-monitor-hotkey-2026-09-23]] (same goal, different mechanism than originally planned). `Tab` re-opens the portal picker to choose a different monitor while the app is running.

Original plan was an in-process restart: cancel the running capture, forget the restore token, spawn a fresh capture generation in the same process. Implemented and mostly worked - cancellation and teardown were correct (`pw_main_loop.run()` was replaced with a manual `while !cancel { loop_.iterate(200ms) }` poll, since `MainLoopBox`/`ContextBox`/etc. wrap raw pointers and aren't `Send`, so there's no way to reach across threads and call `.quit()`; the portal `Session` is now also kept alive and explicitly `.close()`d instead of just dropped, which only tears down the local handle, not the server-side session) - but a *second* portal session from the same process consistently hung at `select_sources`, no picker, no error. Confirmed via `journalctl --user _PID=<xdg-desktop-portal-kde>`: it logs `PipeWire remote error: -2 target not found` on every such attempt. Looks like a limitation in xdg-desktop-portal-kde's ScreenCast backend around a second session on the same D-Bus connection/sender, not a bug in our teardown (which logs confirm completes correctly beforehand).

Workaround: `relaunch_for_switch` in [main.rs](../../../src/main.rs) cancels + joins the current capture generation (so its `session.close()` actually runs - `exec()` replaces the process image outright and would otherwise abandon that thread mid-cleanup), forgets the restore token, then `std::process::Command::new(current_exe).exec()`s a fresh process. Fresh D-Bus connection sidesteps whatever the portal-side issue is. Trade-off: a brief visible window flash/relaunch instead of a seamless in-place switch; window position/size preset resets to default since neither is persisted.

Also fixed along the way: `logical_key` (not `physical_key`) needed for `+`/`-` in [preset-resize-hotkey-2026-09-23](preset-resize-hotkey-2026-09-23.md) - same lesson applies here, `Tab` is a named key so it's layout-independent regardless.

Verified live: Tab relaunches the app and the portal picker reopens correctly.
