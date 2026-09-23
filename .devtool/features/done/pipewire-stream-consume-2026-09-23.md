---
id: "pipewire-stream-consume-2026-09-23"
status: "done"
priority: "high"
assignee: null
epic: "display capture"
dueDate: null
created: "2026-09-23T07:35:30.000Z"
modified: "2026-09-23T10:05:00.000Z"
completedAt: "2026-09-23T10:05:00.000Z"
labels: ["pipewire"]
order: "b2"
---
# Consume the PipeWire video stream

Connect to the PipeWire node handed back by [[portal-screencast-session-2026-09-23]] and receive frames.

- Connect `pipewire-rs` stream to the node id/fd from the portal
- Negotiate a usable video format (e.g. BGRx/RGBx, or dmabuf if supported)
- Receive frame buffers in a callback/loop without blocking the UI thread
- Handle stream errors and remote-closed session (e.g. compositor stops sharing)

Implemented in [capture.rs](../../../src/capture.rs) (`run_pipewire`, `decode_frame`): connects the stream to the portal's fd/node id, negotiates BGRx/RGBx/RGBA/BGRA/RGB (skips YUV formats), decodes into the packed u32 layout the renderer wants, on a dedicated OS thread. Hit a runtime snag first: the devcontainer only had `libpipewire-0.3-dev` (headers), not the runtime's `client.conf`/modules, so `MainLoopBox`/`ContextBox` failed with "Creation failed". Rather than chase the exact Debian package, switched to running the built binary directly on the host (which already has the full PipeWire runtime) instead of inside the container — see [README](../../../README.md). That resolved it immediately. Confirmed live: `capture: negotiated format VideoFormat::BGRx (2560x1600)` and the decoded frame renders correctly (see [[frame-render-pipeline-2026-09-23]]). Remote-close/error handling is still just logging, not a graceful fallback - fine for now.
