#!/usr/bin/env bash
# Tauri system dependencies for ubuntu runners.
#
# The runner's default apt mirror is azure.archive.ubuntu.com. It sometimes stalls for many minutes or fails
# to resolve at all (seen: 397s and 882s against a usual 20-55s; "Temporary failure resolving"), and at times
# archive.ubuntu.com does not answer from the runners either. apt then spends ~20s per host and retry on each
# dead mirror before giving up. So:
# - probe azure, archive.ubuntu.com and an independent mirror (kernel.org) for a few seconds each, log why a
#   probe failed, and when azure does not answer, point apt at the mirrors that do;
# - bound every attempt and retry, and probe again before the second and third attempt.
# Extra args are passed to `apt-get install` (e.g. --no-install-recommends).
set -euo pipefail

packages=(libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf)
apt_opts=(-o Acquire::Retries=3 -o Acquire::http::Timeout=30 -o Acquire::https::Timeout=30 -o DPkg::Lock::Timeout=60)
mirrors=/etc/apt/apt-mirrors.txt

# `timeout` runs apt directly (not through sudo): a signal sent to sudo does not stop apt, and the limit did not hold
apt_get() { sudo timeout -k 10 180 apt-get "${apt_opts[@]}" "$@"; }

# archive mirrors in order of preference; the first is the runner default, the last is independent of Canonical and Azure
candidates=(
  http://azure.archive.ubuntu.com/ubuntu
  https://archive.ubuntu.com/ubuntu
  https://mirrors.edge.kernel.org/ubuntu
)
codename=$(. /etc/os-release && echo "${VERSION_CODENAME:-jammy}") || codename=jammy

# answers within 5s; on failure says why (DNS or connection) so the log shows what broke; a missing curl counts as "does not answer"
probe() {
  local url=$1 host err ip
  host=${url#*://}
  host=${host%%/*}
  if err=$(curl -fsS -o /dev/null --max-time 5 "$url/dists/$codename/InRelease" 2>&1); then
    return 0
  fi
  ip=$(getent hosts "$host" | head -n 1 | cut -d' ' -f1 || true)
  echo "probe failed: $host (resolves to: ${ip:-nothing}): $err"
  return 1
}

# While azure answers the runner's own list is left alone. When it does not, the list is rewritten with the mirrors
# that answer, so apt does not spend ~20s per host on dead ones. If nothing answers the list is kept: the probe
# itself may be wrong, and apt gets its chance.
select_mirrors() {
  local ok=() url i=0
  for url in "${candidates[@]}"; do
    if probe "$url"; then ok+=("$url"); fi
  done
  if [ "${#ok[@]}" -eq 0 ]; then
    echo "::warning::no apt mirror answers the probe"
  elif [ "${ok[0]}" != "${candidates[0]}" ]; then
    echo "azure.archive.ubuntu.com does not answer; using: ${ok[*]}"
    for url in "${ok[@]}"; do
      i=$((i + 1))
      echo "$url/ priority:$i"
    done | sudo tee "$mirrors" >/dev/null
  fi
}

select_mirrors

for attempt in 1 2 3; do
  if [ "$attempt" -gt 1 ]; then
    # a timeout can kill dpkg mid-run; finish what it left before trying again
    sudo dpkg --configure -a || true
    select_mirrors
  fi
  if apt_get update && apt_get install -y "$@" "${packages[@]}"; then
    exit 0
  fi
  echo "::warning::apt attempt $attempt failed or timed out"
  [ "$attempt" -lt 3 ] && sleep 5
done
echo "::error::could not install system dependencies"
exit 1
