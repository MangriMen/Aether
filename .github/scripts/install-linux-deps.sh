#!/usr/bin/env bash
# Tauri system dependencies for ubuntu runners.
#
# apt has no overall timeout, and a slow or dead Ubuntu mirror makes it hang for minutes (seen: 397s and
# 882s against a usual 20-55s). Bound every attempt and retry, so a bad mirror costs one attempt, not the job.
# Extra args are passed to `apt-get install` (e.g. --no-install-recommends).
set -euo pipefail

packages=(libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf)
apt_opts=(-o Acquire::Retries=3 -o Acquire::http::Timeout=30 -o Acquire::https::Timeout=30 -o DPkg::Lock::Timeout=60)

for attempt in 1 2 3; do
  if timeout 180 sudo apt-get "${apt_opts[@]}" update &&
    timeout 180 sudo apt-get "${apt_opts[@]}" install -y "$@" "${packages[@]}"; then
    exit 0
  fi
  echo "::warning::apt attempt $attempt failed or timed out"
  sleep 5
done
echo "::error::could not install system dependencies"
exit 1
