<img src="data/pipsqueak-logo.svg" alt="pipsqueak" width="360">

Borderless picture-in-picture window mirroring a monitor, on Linux/Wayland.

## Dev workflow: build in container, run on host

The host has no Rust toolchain, so building happens in the devcontainer
(`.devcontainer/`). But the app is a normal Wayland/PipeWire/D-Bus client —
running it *inside* the container turned out to need real workarounds
(the session D-Bus only trusts its owning uid; Debian splits the PipeWire
client runtime config across packages that aren't the obvious `-dev` one)
for no benefit, since the host already has a full desktop session with all
of that available. So:

1. Build inside the container:
   ```sh
   docker build -t pip-devcontainer -f .devcontainer/Dockerfile .devcontainer
   docker run --rm -v "$(pwd)":/workspace -w /workspace \
     -v pip-cargo-registry:/home/vscode/.cargo/registry \
     -v pip-cargo-git:/home/vscode/.cargo/git \
     pip-devcontainer cargo build
   ```
   (The named volumes cache `~/.cargo` across runs - without them, every
   `docker run` is a fresh container and re-downloads all ~180 crates from
   scratch. If you ever recreate them, `chown` to uid 1000 first - Docker
   creates named volumes root-owned by default, which the container's
   non-root `vscode` user can't write to.)
2. Run the produced binary directly on the host - `target/` is bind-mounted,
   so it's already sitting at `./target/debug/pipsqueak` after the
   container build:
   ```sh
   ./target/debug/pipsqueak
   ```

If you're opening this in VS Code's Dev Containers extension, it'll use
`.devcontainer/devcontainer.json` for the build step the same way; still run
the binary from a host terminal afterwards, not from inside the container.

## Restore token

First run pops the portal's monitor picker. It saves a `restore_token` to
`$XDG_STATE_HOME/pipsqueak/restore_token` (or `~/.local/state/pipsqueak/`)
so later runs skip the picker.

## License

GPL-3.0-or-later - see [LICENSE](LICENSE).
