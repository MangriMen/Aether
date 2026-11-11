#!/usr/bin/env bash
# Tauri system dependencies for ubuntu runners.
#
# The runner's default apt mirror is azure.archive.ubuntu.com. It sometimes stalls for many minutes or fails
# to resolve at all (seen: 397s and 882s against a usual 20-55s; "Temporary failure resolving"). Bound every
# attempt and retry, and from the second attempt use the canonical archive.ubuntu.com instead, so a bad mirror
# costs one attempt, not the job.
# Extra args are passed to `apt-get install` (e.g. --no-install-recommends).
set -euo pipefail

packages=(libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf)
apt_opts=(-o Acquire::Retries=3 -o Acquire::http::Timeout=30 -o Acquire::https::Timeout=30 -o DPkg::Lock::Timeout=60)
mirrors=/etc/apt/apt-mirrors.txt

# `timeout` runs apt directly (not through sudo): a signal sent to sudo does not stop apt, and the limit did not hold
apt_get() { sudo timeout -k 10 180 apt-get "${apt_opts[@]}" "$@"; }

for attempt in 1 2 3; do
  if [ "$attempt" -gt 1 ] && [ -f "$mirrors" ]; then
    echo "using archive.ubuntu.com instead of azure.archive.ubuntu.com"
    sudo sed -i 's|azure\.archive\.ubuntu\.com|archive.ubuntu.com|g' "$mirrors"
  fi
  if apt_get update && apt_get install -y "$@" "${packages[@]}"; then
    exit 0
  fi
  echo "::warning::apt attempt $attempt failed or timed out"
  sleep 5
done
echo "::error::could not install system dependencies"
exit 1
