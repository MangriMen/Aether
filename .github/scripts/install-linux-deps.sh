#!/usr/bin/env bash
# Tauri system dependencies for ubuntu runners.
#
# The runner's default apt mirror is azure.archive.ubuntu.com. It sometimes stalls for many minutes or fails
# to resolve at all (seen: 397s and 882s against a usual 20-55s; "Temporary failure resolving"). Bound every
# attempt and retry, and from the second attempt drop it from the mirror list, so apt goes straight to the
# canonical archive.ubuntu.com that the list already has. A bad mirror costs one attempt, not the job.
# Extra args are passed to `apt-get install` (e.g. --no-install-recommends).
set -euo pipefail

packages=(libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf)
apt_opts=(-o Acquire::Retries=3 -o Acquire::http::Timeout=30 -o Acquire::https::Timeout=30 -o DPkg::Lock::Timeout=60)
mirrors=/etc/apt/apt-mirrors.txt

# `timeout` runs apt directly (not through sudo): a signal sent to sudo does not stop apt, and the limit did not hold
apt_get() { sudo timeout -k 10 180 apt-get "${apt_opts[@]}" "$@"; }

for attempt in 1 2 3; do
  if [ "$attempt" -gt 1 ]; then
    # a timeout can kill dpkg mid-run; finish what it left before trying again
    sudo dpkg --configure -a || true
    # keep the file as it is when azure is its only entry
    if [ -f "$mirrors" ] && grep -q 'azure\.archive\.ubuntu\.com' "$mirrors" &&
      grep -qv 'azure\.archive\.ubuntu\.com' "$mirrors"; then
      echo "dropping azure.archive.ubuntu.com from $mirrors"
      sudo sed -i '/azure\.archive\.ubuntu\.com/d' "$mirrors"
    fi
  fi
  if apt_get update && apt_get install -y "$@" "${packages[@]}"; then
    exit 0
  fi
  echo "::warning::apt attempt $attempt failed or timed out"
  [ "$attempt" -lt 3 ] && sleep 5
done
echo "::error::could not install system dependencies"
exit 1
