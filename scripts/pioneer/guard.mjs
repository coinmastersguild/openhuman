import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

export const repository = 'coinmastersguild/openhuman';
export function allowedPushUrl(value) {
  return [
    'https://github.com/coinmastersguild/openhuman.git',
    'https://github.com/coinmastersguild/openhuman',
    'git@github.com:coinmastersguild/openhuman.git',
    'ssh://git@github.com/coinmastersguild/openhuman.git',
  ].includes(value);
}
export function requirePushTarget(value) {
  if (!allowedPushUrl(value)) throw new Error('Pioneer pushes are allowed only to coinmastersguild/openhuman; upstream writes are prohibited.');
}
export function configureCheckout() {
  const git = (...args) => execFileSync('git', args, { encoding: 'utf8' }).trim();
  requirePushTarget(git('remote', 'get-url', 'origin'));
  git('remote', 'set-url', '--push', 'origin', 'https://github.com/coinmastersguild/openhuman.git');
  if (git('remote').split('\n').includes('upstream')) git('remote', 'set-url', '--push', 'upstream', 'DISABLED_UPSTREAM_WRITES');
  git('config', 'remote.pushDefault', 'origin');
  execFileSync('gh', ['repo', 'set-default', repository], { stdio: 'inherit' });
}
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv[2] === '--configure') configureCheckout();
    else requirePushTarget(process.argv[2] ?? '');
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
