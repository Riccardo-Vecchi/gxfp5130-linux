#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
upstreams="$root/.upstreams"
mkdir -p "$upstreams"

clone_or_update() {
  local name="$1"
  local url="$2"
  local rev="$3"
  local dir="$upstreams/$name"

  if [[ ! -d "$dir/.git" ]]; then
    git clone "$url" "$dir"
  fi

  git -C "$dir" fetch --all --tags
  git -C "$dir" checkout "$rev"
}

clone_or_update gxfp_linux_driver https://github.com/Void755/gxfp_linux_driver.git 7bd8354
clone_or_update gxfpmoc https://github.com/Void755/gxfpmoc.git 91995aa
clone_or_update open-fprintd https://github.com/uunicorn/open-fprintd.git b707373

patch -d "$upstreams/gxfpmoc" -p1 < "$root/patches/gxfpmoc-mbedtls-and-capture-once.patch"

echo "Fetched and patched upstreams under $upstreams"
