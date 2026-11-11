#!/usr/bin/env bash
# Tauri system dependencies for ubuntu runners.
#
# The runner's default apt mirror is azure.archive.ubuntu.com. It sometimes stalls for many minutes or fails
# to resolve at all (seen: 397s and 882s against a usual 20-55s; "Temporary failure resolving"). apt then
# spends ~20s per host and retry on it before giving up. So:
# - probe azure and the canonical archive.ubuntu.com for a few seconds first, and drop azure from the mirror
#   list when it does not answer but the archive does, so apt goes straight to the archive;
# - bound every attempt and retry, dropping azure from the second attempt on in any case.
# Extra args are passed to `apt-get install` (e.g. --no-install-recommends).
set -euo pipefail

packages=(libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf)
apt_opts=(-o Acquire::Retries=3 -o Acquire::http::Timeout=30 -o Acquire::https::Timeout=30 -o DPkg::Lock::Timeout=60)
mirrors=/etc/apt/apt-mirrors.txt
azure='azure\.archive\.ubuntu\.com'

# `timeout` runs apt directly (not through sudo): a signal sent to sudo does not stop apt, and the limit did not hold
apt_get() { sudo timeout -k 10 180 apt-get "${apt_opts[@]}" "$@"; }

# same scheme and host as apt uses; a missing curl counts as "does not answer"
probe() { curl -fsS -o /dev/null --max-time 5 "$1" 2>/dev/null; }

# keep the file as it is when azure is not in it or is its only entry
drop_azure() {
  if [ -f "$mirrors" ] && grep -q "$azure" "$mirrors" && grep -qv "$azure" "$mirrors"; then
    echo "dropping azure.archive.ubuntu.com from $mirrors"
    sudo sed -i "/$azure/d" "$mirrors"
  fi
}

codename=$(. /etc/os-release && echo "$VERSION_CODENAME")
if ! probe "http://azure.archive.ubuntu.com/ubuntu/dists/$codename/InRelease"; then
  if probe "https://archive.ubuntu.com/ubuntu/dists/$codename/InRelease"; then
    echo "azure.archive.ubuntu.com does not answer, archive.ubuntu.com does"
    drop_azure
  else
    echo "::warning::neither azure.archive.ubuntu.com nor archive.ubuntu.com answers the probe"
  fi
fi

for attempt in 1 2 3; do
  if [ "$attempt" -gt 1 ]; then
    # a timeout can kill dpkg mid-run; finish what it left before trying again
    sudo dpkg --configure -a || true
    drop_azure
  fi
  if apt_get update && apt_get install -y "$@" "${packages[@]}"; then
    exit 0
  fi
  echo "::warning::apt attempt $attempt failed or timed out"
  [ "$attempt" -lt 3 ] && sleep 5
done
echo "::error::could not install system dependencies"
exit 1
