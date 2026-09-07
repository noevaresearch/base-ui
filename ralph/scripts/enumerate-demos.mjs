#!/usr/bin/env node
/* eslint-disable no-console */

// Enumerates every docs page + its demos under docs/src/app/(docs)/react into a manifest for
// Stage 1/2 docs-content spec-mining fan-out.
//
// This mirrors the *demo-resolution shape* of docs/scripts/generateLlmTxt/demoProcessor.mjs
// (page -> demos -> {tailwind, css-modules} variants) without depending on its internal
// @mui/internal-docs-infra loader pipeline, which is not standalone-invokable and exists to
// extract rendered code content rather than to enumerate file paths, which is all Stage 1
// spec-mining needs (it reads the files itself, citing line numbers directly).
//
// Output: ralph/generated/docs-content.json
//
// Usage: node ralph/scripts/enumerate-demos.mjs [--check <expectedDemoCount>]

import fs from 'fs/promises';
import path from 'path';
import { globby } from 'globby';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
// Filesystem literal: the route group folder is named "(docs)" on disk.
const DOCS_REACT_ROOT = path.join(PROJECT_ROOT, 'docs/src/app/(docs)/react');
const OUTPUT_PATH = path.join(PROJECT_ROOT, 'ralph/generated/docs-content.json');

function toRepoRelative(absolutePath) {
  return path.relative(PROJECT_ROOT, absolutePath).replace(/\\/g, '/');
}

async function enumerateDemosForPage(pageDirAbs) {
  const demosRootAbs = path.join(pageDirAbs, 'demos');
  let demoNames;
  try {
    demoNames = (await fs.readdir(demosRootAbs, { withFileTypes: true }))
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name)
      .sort();
  } catch (error) {
    if (error.code === 'ENOENT') {
      return [];
    }
    throw error;
  }

  const demos = [];
  for (const demoName of demoNames) {
    const demoDirAbs = path.join(demosRootAbs, demoName);
    const variantFiles = await globby('{tailwind,css-modules}/index.{ts,tsx}', {
      cwd: demoDirAbs,
    });

    const tailwind = variantFiles.find((f) => f.startsWith('tailwind/'));
    const cssModules = variantFiles.find((f) => f.startsWith('css-modules/'));

    demos.push({
      name: demoName,
      dir: toRepoRelative(demoDirAbs),
      tailwind: tailwind ? `${toRepoRelative(demoDirAbs)}/${tailwind}` : null,
      cssModules: cssModules ? `${toRepoRelative(demoDirAbs)}/${cssModules}` : null,
      isHero: demoName === 'hero',
    });
  }

  return demos;
}

async function main() {
  const pageFiles = await globby('**/page.mdx', { cwd: DOCS_REACT_ROOT, absolute: true });
  pageFiles.sort();

  const pages = [];
  for (const pageFileAbs of pageFiles) {
    const pageDirAbs = path.dirname(pageFileAbs);
    const demos = await enumerateDemosForPage(pageDirAbs);
    const docsReactRootRel = toRepoRelative(DOCS_REACT_ROOT);
    const pageDirRel = toRepoRelative(pageDirAbs);
    const route = pageDirRel === docsReactRootRel ? '' : pageDirRel.replace(`${docsReactRootRel}/`, '');

    pages.push({
      // e.g. "components/dialog" or "handbook/composition"; "" for the root react/page.mdx
      route,
      pagePath: toRepoRelative(pageFileAbs),
      demos,
    });
  }

  await fs.mkdir(path.dirname(OUTPUT_PATH), { recursive: true });
  await fs.writeFile(OUTPUT_PATH, `${JSON.stringify(pages, null, 2)}\n`, 'utf8');

  const totalDemos = pages.reduce((sum, p) => sum + p.demos.length, 0);
  console.log(`Wrote ${toRepoRelative(OUTPUT_PATH)}`);
  console.log(`  ${pages.length} docs pages under docs/src/app/(docs)/react`);
  console.log(`  ${totalDemos} total demo directories`);

  const checkIndex = process.argv.indexOf('--check');
  if (checkIndex !== -1) {
    const expected = Number(process.argv[checkIndex + 1]);
    if (Number.isFinite(expected) && totalDemos !== expected) {
      console.error(
        `Expected ${expected} total demos, found ${totalDemos}. ` +
          'This may be fine (docs content changed) but re-verify before trusting downstream specs.',
      );
      process.exitCode = 1;
    }
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
