import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

for (const name of ['release-production.yml', 'release-staging.yml', 'release-packages.yml', 'build-desktop.yml', 'build-ci-image.yml']) {
  test(`inherited publisher ${name} cannot run in this fork`, () => {
    const text = readFileSync(new URL(`../../.github/workflows/${name}`, import.meta.url), 'utf8');
    const jobs = text.slice(text.indexOf('jobs:')).split(/(?=^  [A-Za-z0-9_-]+:$)/m).slice(1);
    assert.ok(jobs.length > 0);
    for (const job of jobs) {
      const guard = job.match(/^    if: (.+)$/m)?.[1] ?? '';
      assert.ok(guard.includes("github.repository == 'tinyhumansai/openhuman'"), job.split('\n')[0]);
      assert.ok(guard === "github.repository == 'tinyhumansai/openhuman'" || /^\$\{\{ github\.repository == 'tinyhumansai\/openhuman' && \(.*\) \}\}$/.test(guard), 'publisher guard must fence the entire existing expression');
    }
  });
}
