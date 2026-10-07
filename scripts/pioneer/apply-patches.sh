#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
apply_pinned() {
  local source="$root/$1" expected="$2" patch="$root/pioneer/$3"
  [[ "$(git -C "$source" rev-parse HEAD)" == "$expected" ]] || { echo 'Pioneer patch refuses a different source revision' >&2; exit 1; }
  git -C "$source" config remote.origin.pushurl DISABLED_UPSTREAM_WRITES
  if git -C "$source" apply --reverse --check "$patch" 2>/dev/null; then return; fi
  git -C "$source" apply --check "$patch"
  git -C "$source" apply "$patch"
}
apply_pinned vendor/tinycomputer 16446e009c3c2158e5a1bfec24b041d547776e30 tinycomputer-local.patch
apply_pinned vendor/tinycomputer/vendor/tinyinference c144d609b50cbe64c9af69ac3aa64518cce11c90 tinyinference-decisions-local.patch
apply_pinned vendor/pioneer-tinyjevclient 84b3983c7e1e14f7515658ceafabb6c4e7967c94 tinyjevclient-local.patch
