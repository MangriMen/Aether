#!/usr/bin/env bash
# Run the core checks on a branch of origin inside the dev image, without CI.
# Usage: scripts/linux-dev/check.sh [branch] [cargo-package-flags...]
# The branch is cloned into a Docker volume, so nothing is built through /mnt/*.
set -euo pipefail

branch="${1:-main}"
shift || true
packages=("$@")
[ ${#packages[@]} -gt 0 ] || packages=(-p aether-core -p aether-core-plugin-api)

root="$(git rev-parse --show-toplevel)"
remote="$(git -C "$root" remote get-url origin)"
image=aether-dev

docker build -q --target base -t "$image" "$root/scripts/linux-dev" >/dev/null
docker run --rm \
  -v aether-cache:/cache -v aether-work:/work \
  -e BRANCH="$branch" -e REMOTE="$remote" -e PKGS="${packages[*]}" \
  "$image" bash -euxc '
    if [ -d repo/.git ]; then git -C repo fetch --prune origin; else git clone "$REMOTE" repo; fi
    cd repo
    git checkout --force --detach "origin/$BRANCH"
    rustup toolchain install
    cargo test --locked $PKGS
    cargo clippy --locked --all-targets $PKGS -- -D warnings
  '
