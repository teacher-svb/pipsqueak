FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# This image is for BUILDING only, not running the app. Running inside a
# container turned out to need real work to get right (session D-Bus only
# trusts its owning uid; the Debian PipeWire client runtime/config isn't in
# the -dev/-common/-modules packages) - work that's moot anyway since the
# host already has a full Wayland/PipeWire/D-Bus session. So: build here,
# then run the resulting `target/debug/pip` binary directly on the host
# (target/ is bind-mounted, so it's already there). The dev headers below
# are enough for that; no runtime PipeWire/D-Bus packages needed.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libwayland-dev \
    libxkbcommon-dev \
    libegl1-mesa-dev \
    libgl1-mesa-dev \
    libgles2-mesa-dev \
    mesa-vulkan-drivers \
    libvulkan-dev \
    vulkan-tools \
    libpipewire-0.3-dev \
    libspa-0.2-dev \
    libdbus-1-dev \
    clang \
    cmake \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt

USER vscode
