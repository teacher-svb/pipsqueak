---
id: "window-capture-implementation-2026-09-23"
status: "backlog"
priority: "low"
assignee: null
epic: "application capture"
dueDate: null
created: "2026-09-23T07:35:50.000Z"
modified: "2026-09-23T07:35:50.000Z"
completedAt: null
labels: ["optional"]
order: "c1"
---
# Capture a specific application window

Optional milestone, built on the display capture pipeline ([[pipewire-stream-consume-2026-09-23]], [[frame-render-pipeline-2026-09-23]]) using findings from [[research-app-capture-2026-09-23]].

- Request `window` source type from the portal instead of `monitor`
- Let the user pick the target application window via the compositor's picker
- Reuse the existing PipeWire consume + render pipeline unchanged
- Handle the source window closing/disappearing mid-stream
