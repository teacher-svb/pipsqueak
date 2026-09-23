---
id: "frame-render-pipeline-2026-09-23"
status: "done"
priority: "high"
assignee: null
epic: "display capture"
dueDate: null
created: "2026-09-23T07:35:35.000Z"
modified: "2026-09-23T10:05:00.000Z"
completedAt: "2026-09-23T10:05:00.000Z"
labels: ["rendering"]
order: "b3"
---
# Render captured frames into the window

Take frames from [[pipewire-stream-consume-2026-09-23]] and draw them into the window from [[window-creation-2026-09-23]].

- Upload each incoming frame buffer as a texture (via chosen renderer, e.g. `wgpu`/`softbuffer`)
- Handle format conversion if needed (e.g. BGRx -> RGBA)
- Scale/fit the source resolution to the window size, preserving aspect ratio
- Keep frame updates smooth (avoid tearing/stalling the event loop) and drop/skip frames if the renderer falls behind

Implemented in [main.rs](../../../src/main.rs) (`blit_scaled`, `App::redraw`): decoded frames are already in `softbuffer`'s packed u32 layout (no per-frame format conversion needed - `decode_frame` in [capture.rs](../../../src/capture.rs) does that once), so this is a nearest-neighbor scale-to-fit with letterboxing into the fixed 480x270 window. The capture thread writes into a `Arc<Mutex<Option<CapturedFrame>>>` and wakes the winit event loop via `EventLoopProxy`/`UserEvent::FrameReady` -> `request_redraw()`; naturally drops frames if the redraw can't keep up, since only the latest frame is kept. Verified live: the window shows the actual monitor content, scaled to fit, confirmed by the user.
