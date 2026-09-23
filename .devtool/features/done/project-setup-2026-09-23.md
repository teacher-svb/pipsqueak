---
id: "project-setup-2026-09-23"
status: "done"
priority: "high"
assignee: null
epic: "borderless window"
dueDate: null
created: "2026-09-23T07:35:05.000Z"
modified: "2026-09-23T07:50:46.000Z"
completedAt: "2026-09-23T07:50:46.000Z"
labels: ["setup"]
order: "a1"
---
# Project setup

Initialize the Rust project and pick the core crates.

- `cargo init`, edition 2021+
- Pick windowing crate for Wayland (e.g. `winit`) and confirm it supports borderless/undecorated windows and `drag_window()` for client-side moves
- Pick rendering approach for drawing captured frames (e.g. `wgpu` or `softbuffer`)
- Add `ashpd` (xdg-desktop-portal client) and `pipewire-rs` as future dependencies for capture milestone (can be added when that work starts)
- Set up basic CI/build check (`cargo build`, `cargo clippy`)

Done: host had no Rust toolchain at all, so went with a devcontainer instead of a bare-metal install (see [.devcontainer/](../../../.devcontainer/)). Chose `winit` (Wayland-capable, has `drag_window()`) + `softbuffer` for now (swap/extend for real rendering in the capture milestone). `ashpd`/`pipewire-rs` added once the display-capture epic started.

**Revised during display-capture work:** the devcontainer is now build-only, not run-in. Running the app inside the container needed real workarounds - the session D-Bus only trusts its owning uid (fixed by setting `USER vscode` to match host uid 1000), and Debian splits the PipeWire client runtime/config across packages that aren't the `-dev` one (hit a "can't load client.conf" error, didn't find the right package). Chasing that further wasn't worth it: the host already has a full Wayland/PipeWire/D-Bus session, so the built binary just runs directly on host (`target/` is bind-mounted, so it's already there after a container build) - see [README](../../../README.md). Devcontainer no longer mounts any host sockets/devices, just builds. Also added named Docker volumes (`pip-cargo-registry`, `pip-cargo-git`) to cache `~/.cargo` across container runs - without them every `docker run` is a fresh container and re-downloads the whole dependency tree; note Docker creates named volumes root-owned by default, needing a `chown` to uid 1000 once.
