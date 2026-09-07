#!/usr/bin/env node
/* eslint-disable no-console */

// Citation-integrity checker for specs/**/*.md.
//
// Every spec produced by Stage 1/2 spec-mining cites its claims as backtick-wrapped
// `path/to/file.ext:123` or `path/to/file.ext:123-145` references. This script:
//   1. Resolves every citation to a real file + line (hard failure if missing).
//   2. In `record` mode, snapshots a small content window around each citation into a
//      sidecar `<spec>.citations.json` file next to the spec.
//   3. In `check` mode (default), re-hashes that window and compares against the sidecar,
//      distinguishing "line number drifted but content is the same" (soft warning) from
//      "the cited content no longer exists / changed" (hard failure) — this is what lets
//      later Ralph iterations trust a citation instead of a stale or hallucinated one.
//
// Usage:
//   node ralph/scripts/check-citations.mjs record [--scope <dir-or-file>]
//   node ralph/scripts/check-citations.mjs check  [--scope <dir-or-file>]   (default)
//
// Exit code is non-zero on any hard failure, so this is usable as a loop/CI gate.

import fs from 'fs/promises';
import path from 'path';
import crypto from 'crypto';
import { globby } from 'globby';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const DEFAULT_SPECS_ROOT = path.join(PROJECT_ROOT, 'specs');
const WINDOW_MARGIN = 2; // lines of context on each side of a citation, for drift detection

// Matches backtick-wrapped citations: `path/to/file.ext:123` or `path/to/file.ext:12-34`.
// Parens are allowed in the path so docs routes like `docs/src/app/(docs)/.../page.mdx:1` parse.
const CITATION_PATTERN = /`([\w./()-]+\.\w+):(\d+)(?:-(\d+))?`/g;

function toRepoRelative(absolutePath) {
  return path.relative(PROJECT_ROOT, absolutePath).replace(/\\/g, '/');
}

function parseArgs(argv) {
  const mode = argv[2] === 'record' ? 'record' : 'check';
  const scopeIndex = argv.indexOf('--scope');
  const scope = scopeIndex !== -1 ? argv[scopeIndex + 1] : null;
  return { mode, scope };
}

async function findSpecFiles(scope) {
  const root = scope ? path.resolve(PROJECT_ROOT, scope) : DEFAULT_SPECS_ROOT;
  const stat = await fs.stat(root).catch(() => null);
  if (!stat) {
    return [];
  }
  if (stat.isFile()) {
    return [root];
  }
  // *.md covers prose specs; *.json covers machine-readable specs such as demos.json
  // (stage2-docs-mining.md) — excluding the *.citations.json sidecars this script itself writes.
  const files = await globby('**/*.{md,json}', { cwd: root, absolute: true });
  return files.filter((f) => !f.endsWith('.citations.json'));
}

function extractCitations(specContent) {
  const citations = [];
  let match = CITATION_PATTERN.exec(specContent);
  while (match !== null) {
    const [, citedPath, startStr, endStr] = match;
    citations.push({
      citedPath,
      startLine: Number(startStr),
      endLine: endStr ? Number(endStr) : Number(startStr),
    });
    match = CITATION_PATTERN.exec(specContent);
  }
  return citations;
}

function hashWindow(lines, startLine, endLine) {
  const from = Math.max(0, startLine - 1 - WINDOW_MARGIN);
  const to = Math.min(lines.length, endLine + WINDOW_MARGIN);
  const windowText = lines.slice(from, to).join('\n');
  return crypto.createHash('sha256').update(windowText).digest('hex');
}

async function resolveCitation(specFileAbs, citation) {
  const citedAbs = path.resolve(PROJECT_ROOT, citation.citedPath);
  const content = await fs.readFile(citedAbs, 'utf8').catch(() => null);
  if (content === null) {
    return { ok: false, reason: `cited file does not exist: ${citation.citedPath}` };
  }
  const lines = content.split('\n');
  if (citation.startLine < 1 || citation.endLine > lines.length) {
    return {
      ok: false,
      reason: `cited line range ${citation.startLine}-${citation.endLine} is out of bounds for ${citation.citedPath} (${lines.length} lines)`,
    };
  }
  return { ok: true, hash: hashWindow(lines, citation.startLine, citation.endLine) };
}

function sidecarPathFor(specFileAbs) {
  if (specFileAbs.endsWith('.md')) {
    return specFileAbs.replace(/\.md$/, '.citations.json');
  }
  // .json specs (e.g. demos.json) must not be clobbered: append instead of replace.
  return `${specFileAbs}.citations.json`;
}

function citationKey(citation) {
  return `${citation.citedPath}:${citation.startLine}-${citation.endLine}`;
}

async function processSpecFile(specFileAbs, mode, results) {
  const specRel = toRepoRelative(specFileAbs);
  const content = await fs.readFile(specFileAbs, 'utf8');
  const citations = extractCitations(content);

  if (citations.length === 0) {
    return;
  }

  const sidecarPath = sidecarPathFor(specFileAbs);
  const existingSidecar = await fs
    .readFile(sidecarPath, 'utf8')
    .then((s) => JSON.parse(s))
    .catch(() => null);

  const newSidecar = {};

  for (const citation of citations) {
    const key = citationKey(citation);
    const resolved = await resolveCitation(specFileAbs, citation);

    if (!resolved.ok) {
      results.hardFailures.push(`${specRel}: ${resolved.reason}`);
      continue;
    }

    newSidecar[key] = resolved.hash;

    if (mode === 'check') {
      const recordedHash = existingSidecar ? existingSidecar[key] : undefined;
      if (recordedHash === undefined) {
        results.softWarnings.push(
          `${specRel}: citation ${key} has no recorded baseline (run 'record' mode after authoring)`,
        );
      } else if (recordedHash !== resolved.hash) {
        results.hardFailures.push(
          `${specRel}: citation ${key} content has drifted since it was recorded — the cited assertion may no longer say what the spec claims`,
        );
      }
    }
  }

  if (mode === 'record') {
    await fs.writeFile(sidecarPath, `${JSON.stringify(newSidecar, null, 2)}\n`, 'utf8');
  }

  results.citationsChecked += citations.length;
}

async function main() {
  const { mode, scope } = parseArgs(process.argv);
  const specFiles = await findSpecFiles(scope);

  const results = { citationsChecked: 0, hardFailures: [], softWarnings: [] };

  for (const specFileAbs of specFiles) {
    await processSpecFile(specFileAbs, mode, results);
  }

  console.log(
    `Checked ${results.citationsChecked} citations across ${specFiles.length} spec file(s) [mode=${mode}]`,
  );

  if (results.softWarnings.length > 0) {
    console.warn(`\n${results.softWarnings.length} warning(s):`);
    results.softWarnings.forEach((w) => console.warn(`  - ${w}`));
  }

  if (results.hardFailures.length > 0) {
    console.error(`\n${results.hardFailures.length} FAILURE(s):`);
    results.hardFailures.forEach((f) => console.error(`  - ${f}`));
    process.exitCode = 1;
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
