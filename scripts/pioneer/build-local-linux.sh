#!/usr/bin/env bash
# Generic local core from public fork source. Never installs or publishes anything.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
version="${1:-}"
[[ "$version" =~ ^pioneer-local-v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'A pioneer-local-vN.N.N version is required' >&2; exit 1; }
[[ "$(uname -s)" == Linux && "$(uname -m)" == x86_64 ]] || { echo 'Local package currently supports Linux amd64 only' >&2; exit 1; }
git diff --quiet --ignore-submodules=all HEAD || { echo 'Package requires clean committed tracked source' >&2; exit 1; }
# A local package must not inherit the Beast-only compiled trust/routing switch.
unset PIONEER_TINYCOMPUTER_SHA256 PIONEER_LOCAL_RUNTIME
export RUSTUP_TOOLCHAIN=1.96.1
scripts/pioneer/apply-patches.sh
features=bin-tools,http-server,mcp,skills,flows,scheduler-gate
cargo build --locked --release --jobs "${PIONEER_BUILD_JOBS:-8}" -p openhuman-cli --bin openhuman-core --no-default-features --features "$features"
output="$root/pioneer/local-artifacts"
mkdir -p "$output"
install -m 755 target/release/openhuman-core "$output/openhuman-core"
install -m 644 LICENSE "$output/LICENSE"
source="$(git rev-parse HEAD)"
printf 'source=%s\nprofile=release\narchitecture=x86_64-unknown-linux-gnu\nfeatures=%s\npioneer_tenant_pin=absent\nnative_modules=absent\n' "$source" "$features" > "$output/BUILD.txt"
printf 'Corresponding public source: https://github.com/coinmastersguild/openhuman/tree/%s\nInitialize recursive pinned submodules as recorded in that commit.\nBuild: scripts/pioneer/build-local-linux.sh %s\nLicense: GPL-3.0, see LICENSE.\n' "$source" "$version" > "$output/SOURCE.txt"
python3 - "$output" "$version" <<'PY'
import gzip, hashlib, pathlib, struct, sys, tarfile
output, version = pathlib.Path(sys.argv[1]), sys.argv[2]
core = output / 'openhuman-core'
with core.open('rb') as f:
    header = f.read(20)
if len(header) != 20 or header[:6] != b'\x7fELF\x02\x01' or struct.unpack('<H', header[18:20])[0] != 62:
    raise SystemExit('Package refuses a non-amd64 ELF core')
archive = output / f'openhuman-core-{version}-x86_64-unknown-linux-gnu.tar.gz'
# Explicit allowlist: never archive a checkout, build directory or host configuration.
with archive.open('wb') as raw, gzip.GzipFile(filename='', fileobj=raw, mode='wb', mtime=0) as gz, tarfile.open(fileobj=gz, mode='w') as tar:
    for name in ['openhuman-core', 'LICENSE', 'BUILD.txt', 'SOURCE.txt']:
        path = output / name
        info = tar.gettarinfo(str(path), arcname=name)
        info.uid = info.gid = info.mtime = 0
        info.uname = info.gname = ''
        with path.open('rb') as data:
            tar.addfile(info, data)
def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            result.update(block)
    return result.hexdigest()
(output / 'SHA256SUMS').write_text(f'{digest(core)}  openhuman-core\n{digest(archive)}  {archive.name}\n')
print(archive)
PY
