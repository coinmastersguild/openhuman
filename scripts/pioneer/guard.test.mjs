import assert from 'node:assert/strict';
import test from 'node:test';
import { allowedPushUrl, requirePushTarget } from './guard.mjs';

test('allows only our exact public fork URLs', () => {
  for (const value of ['https://github.com/coinmastersguild/openhuman.git', 'git@github.com:coinmastersguild/openhuman.git']) assert.equal(allowedPushUrl(value), true);
});
test('refuses upstream, other repositories, local paths and host suffixes', () => {
  for (const value of ['https://github.com/tinyhumansai/openhuman.git', 'git@github.com:tinyhumansai/openhuman.git', 'https://github.com/coinmastersguild/another.git', 'https://github.com.evil.test/coinmastersguild/openhuman.git', 'https://github.com/coinmastersguild/openhuman.git?redirect=upstream', '/tmp/upstream', '']) assert.throws(() => requirePushTarget(value), /upstream writes are prohibited/);
});
