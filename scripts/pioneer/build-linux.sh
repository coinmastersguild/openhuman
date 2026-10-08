#!/usr/bin/env bash
# Build public fork source and its patched native module; no installation/restart.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
[[ "$(uname -s)" == Linux ]] || { echo 'Pioneer tenant build requires Linux' >&2; exit 1; }
export PATH="$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.96.1
scripts/pioneer/apply-patches.sh
jobs="${PIONEER_BUILD_JOBS:-8}"
profile="${PIONEER_BUILD_PROFILE:-release}"
output="$root/pioneer/artifacts"
mkdir -p "$output"
# Local build and packaged module retain TinyBus's normal manifest/ABI checks.
( cd vendor/tinycomputer; ../../scripts/ci-cancel-aware.sh cargo build --locked --profile "$profile" --jobs "$jobs" -p tinycomputer --lib )
module="vendor/tinycomputer/target/$profile/libtinycomputer.so"
install -m 644 "$module" "$output/libtinycomputer.so"
export PIONEER_TINYCOMPUTER_SHA256="$(sha256sum "$output/libtinycomputer.so" | awk '{print $1}')"
printf '"libtinycomputer.so" = "%s"\n' "$PIONEER_TINYCOMPUTER_SHA256" > "$output/modules.toml"
scripts/ci-cancel-aware.sh cargo build --locked --profile "$profile" --jobs "$jobs" -p openhuman-cli --bin openhuman-core --no-default-features --features bin-tools,http-server,modules,mcp,skills,flows,scheduler-gate,jev
install -m 755 "target/$profile/openhuman-core" "$output/openhuman-core"
( cd "$output"; sha256sum openhuman-core libtinycomputer.so > SHA256SUMS )
printf 'source=%s\nupstream=%s\nmodule_source=%s\nprofile=%s\n' "$(git rev-parse HEAD)" 2eb8a33049915cc5e4b6c10c5c9f2ab99862e0ea 16446e009c3c2158e5a1bfec24b041d547776e30 "$profile" > "$output/BUILD.txt"
echo "$output"

# Revisions above identify tracked sources; dependency changes are exact pinned patches.
if ! git diff --quiet --ignore-submodules=all HEAD; then printf 'tracked_source_dirty=true\n' >> "$output/BUILD.txt"; fi
sha256sum pioneer/*.patch >> "$output/BUILD.txt"
