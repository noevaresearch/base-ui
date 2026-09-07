#!/usr/bin/env node
/* eslint-disable no-console */

// Enumerates every top-level directory under packages/react/src into a component/util
// manifest for Stage 1/2 spec-mining fan-out. Adapts the glob pattern used by
// test/regressions/fixtures.ts (which globs demos/fixtures for visual regression testing)
// to instead enumerate source+test files per top-level directory.
//
// Output: ralph/generated/components.json
//
// Usage: node ralph/scripts/enumerate-components.mjs [--check <expectedCount>]

import fs from 'fs/promises';
import path from 'path';
import { globby } from 'globby';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const SRC_ROOT = path.join(PROJECT_ROOT, 'packages/react/src');
const OUTPUT_PATH = path.join(PROJECT_ROOT, 'ralph/generated/components.json');

function toRepoRelative(absolutePath) {
  return path.relative(PROJECT_ROOT, absolutePath).replace(/\\/g, '/');
}

async function listTopLevelDirs(root) {
  const entries = await fs.readdir(root, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();
}

async function enumerateDir(dirName) {
  const dirAbs = path.join(SRC_ROOT, dirName);
  const dirRel = toRepoRelative(dirAbs);

  const allFiles = await globby('**/*.{ts,tsx}', { cwd: dirAbs });
  const testFiles = [];
  const srcFiles = [];

  for (const relFile of allFiles) {
    const repoRelPath = `${dirRel}/${relFile}`;
    if (/\.test\.tsx?$/.test(relFile)) {
      testFiles.push(repoRelPath);
    } else {
      srcFiles.push(repoRelPath);
    }
  }

  return {
    name: dirName,
    srcDir: dirRel,
    hasTests: testFiles.length > 0,
    testFiles: testFiles.sort(),
    srcFiles: srcFiles.sort(),
  };
}

async function main() {
  const dirNames = await listTopLevelDirs(SRC_ROOT);
  const components = await Promise.all(dirNames.map(enumerateDir));

  await fs.mkdir(path.dirname(OUTPUT_PATH), { recursive: true });
  await fs.writeFile(OUTPUT_PATH, `${JSON.stringify(components, null, 2)}\n`, 'utf8');

  const totalTestFiles = components.reduce((sum, c) => sum + c.testFiles.length, 0);
  console.log(`Wrote ${toRepoRelative(OUTPUT_PATH)}`);
  console.log(`  ${components.length} top-level directories under packages/react/src`);
  console.log(`  ${totalTestFiles} total *.test.tsx files`);

  const checkIndex = process.argv.indexOf('--check');
  if (checkIndex !== -1) {
    const expected = Number(process.argv[checkIndex + 1]);
    if (Number.isFinite(expected) && components.length !== expected) {
      console.error(
        `Expected ${expected} top-level directories, found ${components.length}. ` +
          'This may be fine (the source tree changed) but re-verify before trusting downstream specs.',
      );
      process.exitCode = 1;
    }
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
