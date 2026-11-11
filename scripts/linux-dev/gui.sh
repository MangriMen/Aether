#!/usr/bin/env bash
# Run the Tauri app from a branch of origin inside the gui image.
# Usage: scripts/linux-dev/gui.sh [branch] [command...]   (default command: bun run desktop:dev)
#   default:      the window is drawn through WSLg (Docker Desktop exposes it under /run/desktop/mnt/host/wslg)
#   HEADLESS=dir: the window goes to a virtual display (Xvfb) instead, and a screenshot is saved to
#                 dir/shot.png once the window is up. WSLg screenshots come out black, this ones do not.
#                 In the command, DISPLAY=:99 and `xdotool`/`scrot` drive and capture the window.
#   NESTED=dir:   like HEADLESS, but the virtual display (Xephyr) is itself a window in WSLg, so you
#                 can watch while `xdotool`/`scrot` in the container drive and capture it.
# WebKitGTK has no Chrome DevTools Protocol, so the app cannot be driven that way on Linux.
set -euo pipefail
export MSYS_NO_PATHCONV=1   # Git Bash on Windows would rewrite the container paths below

branch="${1:-main}"
shift || true
cmd="${*:-bun run desktop:dev}"

root="$(git rev-parse --show-toplevel)"
remote="$(git -C "$root" remote get-url origin)"
image=aether-dev-gui

docker build -q --target gui -t "$image" "$root/scripts/linux-dev" >/dev/null
tty=()
[ -t 0 ] && tty=(-it)
display=(-v /run/desktop/mnt/host/wslg:/mnt/wslg -v /run/desktop/mnt/host/wslg/.X11-unix:/tmp/.X11-unix)
setup=""
if [ -n "${HEADLESS:-}${NESTED:-}" ]; then
  out="${HEADLESS:-$NESTED}"
  mkdir -p "$out"
  server='Xvfb :99 -screen 0 1600x1000x24'
  if [ -n "${NESTED:-}" ]; then
    server='DISPLAY=:0 Xephyr :99 -screen 1280x800 -title aether-screen'
    # only the X11 socket: with the Wayland one mounted GTK ignores DISPLAY and draws into WSLg directly
    display=(-v /run/desktop/mnt/host/wslg/.X11-unix:/tmp/.X11-unix)
  else
    display=()
  fi
  display+=(-v "$out":/out)
  setup="$server >/tmp/xserver.log 2>&1 & sleep 2; export GDK_BACKEND=x11 WAYLAND_DISPLAY= DISPLAY=:99"'
         ( for i in $(seq 1 60); do sleep 5; xdotool search --name Aether >/dev/null 2>&1 && { sleep 20; scrot -o /out/shot.png; break; }; done ) &'
fi
name=()
[ -n "${NAME:-}" ] && name=(--name "$NAME")   # lets `docker exec` reach the container
docker run --rm "${tty[@]}" "${name[@]}" \
  -v aether-cache:/cache -v aether-work:/work "${display[@]}" \
  -e BRANCH="$branch" -e REMOTE="$remote" -e CMD="$cmd" -e SETUP="$setup" \
  "$image" bash -euxc '
    if [ -d repo/.git ]; then git -C repo fetch --prune origin; else git clone "$REMOTE" repo; fi
    cd repo
    git checkout --force --detach "origin/$BRANCH"
    rustup toolchain install
    bun install --frozen-lockfile
    eval "$SETUP"
    eval "$CMD"
  '
