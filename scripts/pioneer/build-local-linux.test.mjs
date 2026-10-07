import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, chmodSync, readFileSync, readdirSync, rmSync, cpSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

const script = resolve('scripts/pioneer/build-local-linux.sh');
function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'pioneer-local-package-test-'));
  mkdirSync(join(root, 'scripts/pioneer'), { recursive: true });
  cpSync(script, join(root, 'scripts/pioneer/build-local-linux.sh'));
  writeFileSync(join(root, 'LICENSE'), 'GPL-3.0 fixture\n');
  mkdirSync(join(root, 'stubs'));
  const stub = (name, body) => { const path = join(root, 'stubs', name); writeFileSync(path, body); chmodSync(path, 0o755); };
  stub('uname', '#!/bin/sh\ncase "$1" in -s) echo Linux;; -m) echo "${TEST_ARCH:-x86_64}";; esac\n');
  stub('cargo', `#!/bin/sh\nset -eu\n[ -z "\${PIONEER_TINYCOMPUTER_SHA256+x}" ]\n[ -z "\${PIONEER_LOCAL_RUNTIME+x}" ]\nprintf '%s\\n' "$*" > build-args\nmkdir -p target/release\nprintf 'generic-fork-core\\n' > target/release/openhuman-core\nchmod 755 target/release/openhuman-core\n`);
  const git = (...args) => { const result = spawnSync('git', args, { cwd: root, encoding: 'utf8' }); assert.equal(result.status, 0, result.stderr); return result.stdout.trim(); };
  git('init', '-q'); git('add', 'scripts', 'LICENSE');
  git('-c', 'user.name=Package Test', '-c', 'user.email=test@example.invalid', 'commit', '-qm', 'public source fixture');
  const source = git('rev-parse', 'HEAD');
  const run = (extra = {}, version = 'pioneer-local-v0.1.0') => spawnSync('bash', ['scripts/pioneer/build-local-linux.sh', version], {
    cwd: root, encoding: 'utf8', env: { ...process.env, PATH: join(root, 'stubs') + ':' + process.env.PATH,
      PIONEER_TINYCOMPUTER_SHA256: 'must-not-be-compiled', PIONEER_LOCAL_RUNTIME: '1', ...extra },
  });
  return { root, source, run, cleanup: () => rmSync(root, { recursive: true, force: true }) };
}

test('generic fork package clears tenant compile switches and includes only public payload', () => {
  const f = fixture();
  try {
    writeFileSync(join(f.root, 'HOST_ONLY'), 'private fixture that must not enter package');
    const result = f.run(); assert.equal(result.status, 0, result.stderr);
    const args = readFileSync(join(f.root, 'build-args'), 'utf8');
    assert.match(args, /--locked/); assert.match(args, /--no-default-features/);
    assert.match(args, /bin-tools,http-server,mcp,skills,flows,scheduler-gate/);
    assert.doesNotMatch(args, /(?:^|,)modules(?:,|$)|(?:^|,)jev(?:,|$)/);
    const dir = join(f.root, 'pioneer/local-artifacts');
    const archive = readdirSync(dir).find(name => name.endsWith('.tar.gz'));
    const listing = spawnSync('tar', ['-tzf', join(dir, archive)], { encoding: 'utf8' });
    assert.equal(listing.status, 0, listing.stderr);
    assert.deepEqual(listing.stdout.trim().split('\n').sort(), ['BUILD.txt', 'LICENSE', 'SOURCE.txt', 'openhuman-core']);
    assert.match(readFileSync(join(dir, 'BUILD.txt'), 'utf8'), new RegExp(`source=${f.source}`));
    assert.match(readFileSync(join(dir, 'SOURCE.txt'), 'utf8'), new RegExp(`https://github.com/coinmastersguild/openhuman/tree/${f.source}`));
    assert.match(readFileSync(join(dir, 'SHA256SUMS'), 'utf8'), new RegExp(archive.replaceAll('.', '\\.')));
  } finally { f.cleanup(); }
});

test('local package refuses unsupported architecture before compiling', () => {
  const f = fixture(); try { const r = f.run({ TEST_ARCH: 'aarch64' }); assert.notEqual(r.status, 0); assert.match(r.stderr, /amd64/); }
  finally { f.cleanup(); }
});

test('local package refuses uncommitted tracked source', () => {
  const f = fixture(); try { writeFileSync(join(f.root, 'LICENSE'), 'edited'); const r = f.run(); assert.notEqual(r.status, 0); assert.match(r.stderr, /clean committed/); }
  finally { f.cleanup(); }
});

test('local package rejects a version that could escape its output directory', () => {
  const f = fixture(); try { const r = f.run({}, '../outside'); assert.notEqual(r.status, 0); assert.match(r.stderr, /version/); }
  finally { f.cleanup(); }
});
