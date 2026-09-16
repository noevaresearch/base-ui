#!/usr/bin/env node
// release/set-version.mjs <x.y.z> — write one version into every manifest that must agree on it.
//
// The published version is NOT committed back to the repo (see release-state.mjs: a workflow that
// commits its bump feeds the commit counter that triggered it). Instead the workflow checks out the
// release commit, runs this script in the checkout, and publishes from the rewritten tree. The repo
// keeps `0.1.0` as the working version; the registry carries the real sequence 0.1.1, 0.1.2, …
//
// Two kinds of occurrence are touched, and nothing else:
//   * `version = "…"` inside [workspace.package]        — the crates' own version
//   * `version = "…"` on a line that also has `path = ` — a path dependency, which crates.io
//     requires to carry an explicit version requirement matching what was actually published
// A third kind matters and is deliberately NOT touched: a dependency that pins a THIRD-PARTY crate
// (`leptos = { version = "0.7" }`) has no `path =`, so it is left alone.

import fs from 'node:fs';
import path from 'node:path';

const ROOT = path.resolve(import.meta.dirname, '..');
const MANIFESTS = [
  'Cargo.toml',
  'crates/leptos-ui/Cargo.toml',
  'crates/leptos-ui-internals/Cargo.toml',
  'crates/leptos-ui-utils/Cargo.toml',
  'crates/docs-app/Cargo.toml',
];

const version = process.argv[2];
if (!/^0\.1\.\d+$/.test(version ?? '')) {
  console.error(`usage: node release/set-version.mjs <x.y.z>   (this port releases 0.1.<patch> only)`);
  process.exit(2);
}

let totalChanges = 0;
for (const rel of MANIFESTS) {
  const file = path.join(ROOT, rel);
  const before = fs.readFileSync(file, 'utf8');
  let inWorkspacePackage = false;
  let changed = 0;

  const after = before
    .split('\n')
    .map((line) => {
      if (/^\[/.test(line.trim())) inWorkspacePackage = line.trim() === '[workspace.package]';
      const ownVersion = inWorkspacePackage && /^\s*version\s*=\s*"/.test(line);
      const pathDep = /path\s*=\s*"/.test(line) && /\bversion\s*=\s*"/.test(line);
      if (!ownVersion && !pathDep) return line;
      const next = line.replace(/version\s*=\s*"[^"]*"/, `version = "${version}"`);
      if (next !== line) changed += 1;
      return next;
    })
    .join('\n');

  if (changed) fs.writeFileSync(file, after);
  totalChanges += changed;
  console.log(`${changed ? 'updated' : 'no-change'}  ${rel}  (${changed} line(s))`);
}

if (!totalChanges) {
  console.error('nothing changed — the manifests do not look like this script expects; refusing to continue');
  process.exit(1);
}
console.log(`\nmanifest version is now ${version} (${totalChanges} line(s) across ${MANIFESTS.length} manifests)`);
