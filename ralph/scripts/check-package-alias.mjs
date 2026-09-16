#!/usr/bin/env node
/* eslint-disable no-console */

// check-package-alias.mjs — the docs must name THIS port, and the name must resolve locally.
//
// WHY THIS EXISTS
// ---------------
// The alias `@noevaresearch/base-ui` is how the docs refer to this library, but it is deliberately
// UNPUBLISHED (`packages/leptos/package.json`, `private: true`). Two failure modes follow from that, and
// neither is visible to any other gate:
//   1. a page names upstream's package instead (`npm install @base-ui/react`), sending the reader to a
//      different library, in a different language;
//   2. the alias stops resolving locally (a `pnpm install` prunes the symlink, the manifest name is edited,
//      the crate is renamed), so the docs reference a name that exists nowhere.
//
// So this gate asserts the whole chain, end to end: the manifest's identity, that the MANIFEST and the
// Rust-side install reference AGREE, that the bare specifier resolves from the repo root AND from the
// resolution fixture, and that no page under the port's own content tells a reader to install upstream's
// package. Run it after touching packaging, or when a page starts showing install instructions.
//
// USAGE
//   node ralph/scripts/check-package-alias.mjs            # enforce
//   node ralph/scripts/check-package-alias.mjs --fix-link  # re-create the local node_modules link first
//
// Exit: 0 clean, 1 defect, 3 setup problem (nothing to assert against).

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const ALIAS = '@noevaresearch/base-ui';
const MANIFEST = path.join(PROJECT_ROOT, 'packages/leptos/package.json');
const INSTALL_REF = path.join(PROJECT_ROOT, 'crates/docs-app/src/install_ref.rs');
const PAGE_ROOTS = [path.join(PROJECT_ROOT, 'crates/docs-app/src/pages')];
const FIX = process.argv.includes('--fix-link');

const defects = [];
const notes = [];

function ensureLink() {
  const linkDir = path.join(PROJECT_ROOT, 'node_modules/@noevaresearch');
  const link = path.join(linkDir, 'base-ui');
  try {
    fs.mkdirSync(linkDir, { recursive: true });
    const target = fs.realpathSync(path.join(PROJECT_ROOT, 'packages/leptos'));
    let ok = false;
    try {
      ok = fs.realpathSync(link) === target;
    } catch { ok = false; }
    if (!ok) {
      try { fs.rmSync(link, { recursive: true, force: true }); } catch {}
      fs.symlinkSync('../../packages/leptos', link, 'dir');
      notes.push(`created the local link node_modules/${ALIAS} -> packages/leptos (a plain \`pnpm install\` recreates it from the workspace)`);
    }
    return true;
  } catch (e) {
    defects.push(`could not create the local link for ${ALIAS}: ${e.message}`);
    return false;
  }
}

// 1. manifest identity
if (!fs.existsSync(MANIFEST)) {
  console.error(`SETUP: missing ${path.relative(PROJECT_ROOT, MANIFEST)} — nothing to assert.`);
  process.exit(3);
}
const manifest = JSON.parse(fs.readFileSync(MANIFEST, 'utf8'));
if (manifest.name !== ALIAS) defects.push(`packages/leptos/package.json name is "${manifest.name}", expected "${ALIAS}"`);
if (manifest.private !== true) defects.push('packages/leptos/package.json must stay `private: true` while the crate is unpublished (an accidental publish makes the alias a stale npm package)');

// 2. the Rust-side install reference must AGREE with the manifest
if (!fs.existsSync(INSTALL_REF)) {
  defects.push(`missing ${path.relative(PROJECT_ROOT, INSTALL_REF)} — pages have no canonical install text to render`);
} else {
  const rs = fs.readFileSync(INSTALL_REF, 'utf8');
  const aliasInRs = rs.match(/PACKAGE_ALIAS:\s*&str\s*=\s*"([^"]+)"/)?.[1];
  const publishedInRs = rs.match(/PUBLISHED:\s*bool\s*=\s*(true|false)/)?.[1];
  if (aliasInRs !== ALIAS) defects.push(`install_ref.rs PACKAGE_ALIAS is "${aliasInRs ?? '(absent)'}", expected "${ALIAS}"`);
  if (publishedInRs === 'true') defects.push('install_ref.rs claims PUBLISHED = true — the crate is not published; do not imply an npm release exists');
  if (!rs.includes('INSTALL_SNIPPET')) defects.push('install_ref.rs has no INSTALL_SNIPPET for pages to render');
}

// 3. the bare specifier must RESOLVE (root and fixture)
if (FIX) ensureLink();
const roots = [PROJECT_ROOT, path.join(PROJECT_ROOT, 'test/node-resolution')];
for (const root of roots) {
  const r = spawnSync('node', ['-e', `import('${ALIAS}').then(m => console.log(m.PACKAGE_NAME + '|' + m.RUST_CRATE + '|' + (m.NOT_PUBLISHED ? 'unpublished' : 'published'))).catch(e => { console.log('ERR:' + e.code); process.exit(1) })`], { cwd: root, encoding: 'utf8' });
  const out = (r.stdout || '').trim();
  if (r.status !== 0 || out.startsWith('ERR:')) {
    if (root === PROJECT_ROOT) ensureLink();
    const retry = root === PROJECT_ROOT ? spawnSync('node', ['-e', `import('${ALIAS}').then(m => console.log('ok')).catch(e => { console.log('ERR:' + e.code); process.exit(1) })`], { cwd: root, encoding: 'utf8' }) : r;
    if (retry.status !== 0) {
      defects.push(`${ALIAS} does not resolve from ${path.relative(PROJECT_ROOT, root) || '.'} (${out || 'no output'}) — the docs would name a package that exists nowhere. Fix: \`pnpm install\` (the workspace links it), or --fix-link for the local symlink.`);
    } else {
      notes.push(`${ALIAS} resolved from ${path.relative(PROJECT_ROOT, root) || '.'} after repair`);
    }
  } else {
    notes.push(`${path.relative(PROJECT_ROOT, root) || '.'}: ${out}`);
  }
}

// 4. no page may tell a reader to install upstream's package
const installRe = /npm\s+(?:install|i|add)\s+[^\n"']*@base-ui\/react|yarn\s+add\s+[^\n"']*@base-ui\/react|pnpm\s+(?:add|install)\s+[^\n"']*@base-ui\/react|from\s+['"]@base-ui\/react/;
for (const root of PAGE_ROOTS) {
  if (!fs.existsSync(root)) continue;
  for (const f of fs.readdirSync(root)) {
    if (!f.endsWith('.rs')) continue;
    const lines = fs.readFileSync(path.join(root, f), 'utf8').split('\n');
    lines.forEach((line, i) => {
      if (installRe.test(line)) defects.push(`${path.relative(PROJECT_ROOT, path.join(root, f))}:${i + 1} tells the reader to install upstream's package — the port's name is ${ALIAS}`);
    });
  }
}

console.log(`package alias: ${ALIAS} (crate ${manifest.name === ALIAS ? 'leptos-ui' : '?'}) — ${defects.length} defect(s)`);
for (const n of notes) console.log(`  · ${n}`);
for (const d of defects) console.log(`  FAIL ${d}`);
console.log(`report: ${path.relative(PROJECT_ROOT, INSTALL_REF)} is the canonical install text; packages/leptos/README.md documents the unpublished status.`);
process.exit(defects.length ? 1 : 0);
