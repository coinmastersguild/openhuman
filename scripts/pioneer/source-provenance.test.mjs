import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const verifier = resolve('scripts/pioneer/source-provenance.py');
const overlays = [
  ['vendor/tinycomputer', 'tinycomputer-local.patch'],
  ['vendor/tinycomputer/vendor/tinyinference', 'tinyinference-decisions-local.patch'],
  ['vendor/pioneer-tinyjevclient', 'tinyjevclient-local.patch'],
];
function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'pioneer-source-provenance-'));
  function git(path, ...args) {
    const result = spawnSync('git', ['-c', 'user.name=Provenance Test', '-c', 'user.email=test@example.invalid', ...args], { cwd: path, encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr + result.stdout); return result.stdout;
  }
  function repo(path) {
    mkdirSync(path, { recursive: true }); git(path, 'init', '-q');
    mkdirSync(join(path, 'src')); writeFileSync(join(path, 'src/lib.rs'), 'pub fn value() -> u8 { 1 }\n');
    writeFileSync(join(path, '.gitignore'), '/target/\n*.local\n');
    git(path, 'add', '.'); git(path, 'commit', '-qm', 'public dependency');
  }
  repo(root);
  for (const path of ['vendor/tinycomputer', 'vendor/tinycomputer/vendor/tinyinference', 'vendor/pioneer-tinyjevclient', 'vendor/tinyagents']) repo(join(root, path));
  // Record nested gitlinks before recording the superproject's vendor pins.
  git(join(root, 'vendor/tinycomputer'), 'add', 'vendor/tinyinference');
  git(join(root, 'vendor/tinycomputer'), 'commit', '-qm', 'nested public dependency pin');
  mkdirSync(join(root, 'pioneer'));
  for (const [path, patch] of overlays) {
    const full = join(root, path); writeFileSync(join(full, 'src/lib.rs'), 'pub fn value() -> u8 { 2 }\n');
    writeFileSync(join(root, 'pioneer', patch), git(full, 'diff', '--binary', '--full-index', 'HEAD'));
    git(full, 'checkout', '--', 'src/lib.rs');
  }
  git(root, 'add', 'pioneer', 'vendor'); git(root, 'commit', '-qm', 'public overlays and recursive pins');
  const check = (mode = 'before') => spawnSync('python3', [verifier, root, mode], { encoding: 'utf8' });
  const apply = () => { for (const [path, patch] of overlays) git(join(root, path), 'apply', join(root, 'pioneer', patch)); };
  return { root, git, check, apply, cleanup: () => rmSync(root, { recursive: true, force: true }) };
}

test('accepts clean recursive source and exactly the three committed overlays', () => {
  const f = fixture(); try {
    assert.equal(f.check().status, 0, f.check().stderr); f.apply();
    assert.equal(f.check().status, 0, f.check().stderr);
    assert.equal(f.check('after').status, 0, f.check('after').stderr);
  } finally { f.cleanup(); }
});

test('rejects a changed recursive gitlink HEAD before building', () => {
  const f = fixture(); try {
    const sub = join(f.root, 'vendor/tinycomputer/vendor/tinyinference');
    writeFileSync(join(sub, 'src/lib.rs'), 'pub fn private_value() {}\n');
    f.git(sub, 'add', '.'); f.git(sub, 'commit', '-qm', 'unpublished dependency');
    const r = f.check(); assert.notEqual(r.status, 0); assert.match(r.stderr, /gitlink|revision/i);
  } finally { f.cleanup(); }
});

test('rejects an uninitialized dependency instead of checking its parent repository', () => {
  const f = fixture(); try {
    rmSync(join(f.root, 'vendor/tinyagents/.git'), { recursive: true });
    const r = f.check(); assert.notEqual(r.status, 0); assert.match(r.stderr, /initialized|repository/i);
  } finally { f.cleanup(); }
});

test('rejects an extra tracked vendor edit even when the public overlay is applied', () => {
  const f = fixture(); try {
    f.apply(); writeFileSync(join(f.root, 'vendor/tinycomputer/src/lib.rs'), 'pub fn private_value() {}\n');
    const r = f.check(); assert.notEqual(r.status, 0); assert.match(r.stderr, /overlay|tracked/i);
  } finally { f.cleanup(); }
});

test('rejects staged vendor dirt and dirty dependencies without an approved overlay', () => {
  const f = fixture(); try {
    const sub = join(f.root, 'vendor/tinyagents'); writeFileSync(join(sub, 'src/lib.rs'), 'pub fn hidden() {}\n');
    f.git(sub, 'add', '.'); const r = f.check(); assert.notEqual(r.status, 0); assert.match(r.stderr, /tracked/i);
  } finally { f.cleanup(); }
});

test('rejects an untracked Rust input in a recursively pinned dependency', () => {
  const f = fixture(); try {
    writeFileSync(join(f.root, 'vendor/tinycomputer/vendor/tinyinference/src/extra.rs'), 'pub fn private_input() {}\n');
    const r = f.check(); assert.notEqual(r.status, 0); assert.match(r.stderr, /untracked/i);
  } finally { f.cleanup(); }
});

test('rejects an ignored source input while allowing generated target output', () => {
  const f = fixture(); try {
    const sub = join(f.root, 'vendor/tinyagents'); mkdirSync(join(sub, 'target')); writeFileSync(join(sub, 'target/output.rs'), 'generated');
    assert.equal(f.check().status, 0, f.check().stderr);
    writeFileSync(join(sub, 'src/payload.local'), 'ignored compiler input');
    const r = f.check(); assert.notEqual(r.status, 0); assert.match(r.stderr, /ignored|source/i);
  } finally { f.cleanup(); }
});

test('post-apply verification requires the complete published overlay, not a clean substitute', () => {
  const f = fixture(); try {
    const r = f.check('after'); assert.notEqual(r.status, 0); assert.match(r.stderr, /overlay/i);
  } finally { f.cleanup(); }
});
