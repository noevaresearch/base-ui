#!/usr/bin/env node
/* eslint-disable no-console */

// Enumerates packages/utils/src into a manifest for Stage 1/2 spec-mining fan-out.
// Unlike enumerate-components.mjs (one unit per top-level directory), packages/utils/src is
// mostly flat: most utils are a single `<name>.ts` (+ `<name>.test.ts`) pair at the top level,
// with two subdirectories (platform/, store/) that group related utils the way a component
// directory does. This groups each top-level file pair as its own unit and each subdirectory
// as its own unit, rather than one unit per file (which would fragment tightly coupled helpers)
// or one unit for the whole package (which would defeat the point of bounded fan-out tasks).
//
// Output: ralph/generated/utils.json
//
// Usage: node ralph/scripts/enumerate-utils.mjs [--check <expectedUnitCount>]

import fs from 'fs/promises';
import path from 'path';
import { globby } from 'globby';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const SRC_ROOT = path.join(PROJECT_ROOT, 'packages/utils/src');
const OUTPUT_PATH = path.join(PROJECT_ROOT, 'ralph/generated/utils.json');

function toRepoRelative(absolutePath) {
  return path.relative(PROJECT_ROOT, absolutePath).replace(/\\/g, '/');
}

async function enumerateSubdirUnit(dirName) {
  const dirAbs = path.join(SRC_ROOT, dirName);
  const allFiles = await globby('**/*.{ts,tsx}', { cwd: dirAbs });
  const testFiles = [];
  const srcFiles = [];
  for (const relFile of allFiles) {
    const repoRelPath = `${toRepoRelative(dirAbs)}/${relFile}`;
    if (/\.test\.tsx?$/.test(relFile)) {
      testFiles.push(repoRelPath);
    } else {
      srcFiles.push(repoRelPath);
    }
  }
  return {
    name: dirName,
    kind: 'dir',
    testFiles: testFiles.sort(),
    srcFiles: srcFiles.sort(),
  };
}

async function main() {
  const entries = await fs.readdir(SRC_ROOT, { withFileTypes: true });
  const subdirs = entries.filter((e) => e.isDirectory()).map((e) => e.name).sort();
  const topLevelFiles = entries
    .filter((e) => e.isFile() && /\.tsx?$/.test(e.name))
    .map((e) => e.name)
    .sort();

  const units = [];

  for (const dirName of subdirs) {
    units.push(await enumerateSubdirUnit(dirName));
  }

  // Group top-level files by base name (the part before the first dot), so `addEventListener.ts`,
  // `addEventListener.test.ts`, and the type-check-only `addEventListener.spec.ts` all group
  // together, and a descriptively-named variant test like `useIdleCallback.fallback.test.ts`
  // still groups under `useIdleCallback` rather than becoming its own fake unit.
  const byBaseName = new Map();
  for (const fileName of topLevelFiles) {
    const isTest = /\.(test|spec)\.tsx?$/.test(fileName);
    const baseName = fileName.split('.')[0];
    if (!byBaseName.has(baseName)) {
      byBaseName.set(baseName, { srcFiles: [], testFiles: [] });
    }
    const entry = byBaseName.get(baseName);
    const repoRelPath = `${toRepoRelative(SRC_ROOT)}/${fileName}`;
    if (isTest) {
      entry.testFiles.push(repoRelPath);
    } else {
      entry.srcFiles.push(repoRelPath);
    }
  }

  for (const [baseName, files] of byBaseName) {
    units.push({
      name: baseName,
      kind: 'file',
      testFiles: files.testFiles,
      srcFiles: files.srcFiles,
    });
  }

  units.sort((a, b) => a.name.localeCompare(b.name));

  await fs.mkdir(path.dirname(OUTPUT_PATH), { recursive: true });
  await fs.writeFile(OUTPUT_PATH, `${JSON.stringify(units, null, 2)}\n`, 'utf8');

  const totalTestFiles = units.reduce((sum, u) => sum + u.testFiles.length, 0);
  console.log(`Wrote ${toRepoRelative(OUTPUT_PATH)}`);
  console.log(`  ${units.length} spec-mining units under packages/utils/src`);
  console.log(`  ${totalTestFiles} total *.test.ts files`);

  const checkIndex = process.argv.indexOf('--check');
  if (checkIndex !== -1) {
    const expected = Number(process.argv[checkIndex + 1]);
    if (Number.isFinite(expected) && units.length !== expected) {
      console.error(`Expected ${expected} units, found ${units.length}.`);
      process.exitCode = 1;
    }
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
