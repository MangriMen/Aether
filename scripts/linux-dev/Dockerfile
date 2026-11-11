# Mirrors the CI runner (ubuntu-22.04). The Rust version is not pinned here:
# rustup reads it from rust-toolchain.toml on first use.
#   base: enough for `cargo test`/`clippy` on the library crates
#   gui:  adds the Tauri system libraries, Node/Bun and Java to run the desktop app
FROM ubuntu:22.04 AS base

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential ca-certificates curl git pkg-config libssl-dev sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# rustup itself lives in the image; toolchains, registry and build output go to the
# /cache volume so they survive container rebuilds.
ENV PATH=/usr/local/cargo/bin:$PATH
RUN CARGO_HOME=/usr/local/cargo RUSTUP_HOME=/usr/local/rustup \
    sh -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none'

RUN useradd -m -u 1000 dev && mkdir -p /cache /work && chown dev:dev /cache /work
USER dev
ENV CARGO_HOME=/cache/cargo \
    RUSTUP_HOME=/cache/rustup \
    CARGO_TARGET_DIR=/cache/target \
    SQLX_OFFLINE=true \
    CARGO_TERM_COLOR=always
WORKDIR /work

FROM base AS gui
USER root
# Same packages as .github/scripts/install-linux-deps.sh, plus a JRE and the GL/audio
# libraries Minecraft loads, and xdotool to drive the window.
RUN apt-get update && apt-get install -y --no-install-recommends \
        libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf \
        openjdk-21-jre libgl1 libgl1-mesa-dri libglx-mesa0 libopenal1 \
        libxrandr2 libxxf86vm1 libxcursor1 xdotool xvfb xserver-xephyr x11-apps imagemagick scrot unzip \
    && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://deb.nodesource.com/setup_24.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*
# github.com release downloads from inside Docker are flaky; the npm registry is not
RUN npm install -g bun@1.3.11
USER dev
ENV XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir \
    WAYLAND_DISPLAY=wayland-0 \
    DISPLAY=:0 \
    WEBKIT_DISABLE_DMABUF_RENDERER=1
