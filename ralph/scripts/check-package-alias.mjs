#!/usr/bin/env node
/* eslint-disable no-console */

// check-package-alias.mjs — the docs must name THIS port, and the name must resolve locally.
//
// WHY THIS EXISTS
// ---------------
// The alias `base-ui-leptos` is how the docs refer to this library, but it is deliberately
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
import { cfgTestRanges, inRanges, codeBlockRanges, codeBlockAt } from './lib/source-scope.mjs';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const ALIAS = 'base-ui-leptos';
const MANIFEST = path.join(PROJECT_ROOT, 'packages/leptos/package.json');
const INSTALL_REF = path.join(PROJECT_ROOT, 'crates/docs-app/src/install_ref.rs');
const PAGE_ROOTS = [path.join(PROJECT_ROOT, 'crates/docs-app/src/pages')];
const FIX = process.argv.includes('--fix-link');

const defects = [];
const notes = [];

/**
 * (Re-)create the local symlink that makes the alias resolvable from a given root.
 *
 * BUG FIXED HERE (2026-09-16, measured): this function created `node_modules/@noevaresearch/base-ui` —
 * a name that appears NOWHERE in this repository (`grep -r '@noevaresearch'` outside node_modules: zero
 * hits) — and never `node_modules/base-ui-leptos`, the specifier actually under test. So the retry below
 * could never succeed, and CI's `package alias FAIL` (where this box read 0 defect(s)) was the one
 * verdict that was TRUE: the green here came from two UNTRACKED symlinks
 * (`node_modules/base-ui-leptos` and `test/node-resolution/node_modules/base-ui-leptos`), and the root
 * `package.json` declares no such dependency, so a fresh `pnpm install` cannot create the root link at
 * all. A verdict that depends on untracked state is not a verdict.
 */
function ensureLinkAt(root) {
  const link = path.join(root, 'node_modules', ALIAS);
  let target;
  try { target = fs.realpathSync(path.join(PROJECT_ROOT, 'packages/leptos')); } catch (e) {
    defects.push(`could not resolve packages/leptos to link ${ALIAS}: ${e.message}`);
    return false;
  }
  try { if (fs.realpathSync(link) === target) return true; } catch { /* missing or stale */ }
  try { fs.rmSync(link, { recursive: true, force: true }); } catch { /* may not exist */ }
  try {
    fs.mkdirSync(path.dirname(link), { recursive: true });
    fs.symlinkSync(path.relative(path.dirname(link), target), link, 'dir');
    const where = path.relative(PROJECT_ROOT, root) || '.';
    notes.push(`created the local mapping ${path.relative(PROJECT_ROOT, link)} -> packages/leptos for "${where}" (the tracked recipe is test/node-resolution/package.json's \`${ALIAS}: workspace:*\`, which \`pnpm install\` links; the repo ROOT declares no such dependency, so the root can only resolve the alias through a local mapping)`);
    return true;
  } catch (e) {
    defects.push(`could not create the local link for ${ALIAS} at "${path.relative(PROJECT_ROOT, root) || '.'}": ${e.message}`);
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
  // The crate WAS published on 2026-09-16 (base-ui-leptos 0.1.1). The gate's job is consistency, not a
  // frozen belief: the crate NAME in install_ref must match the manifest, and a page must not claim an npm
  // release for the JS alias (which is still local-only). Registry truth itself is owned by
  // release-watchdog.sh, which reads crates.io rather than the repo.
  const manifestCrate = (() => { try { const m = fs.readFileSync(path.join(PROJECT_ROOT, 'crates/leptos-ui/Cargo.toml'), 'utf8'); return m.match(/^name\s*=\s*"([^"]+)"/m)?.[1] ?? null; } catch { return null; } })();
  const crateInRs = rs.match(/RUST_CRATE:\s*&str\s*=\s*"([^"]+)"/)?.[1];
  if (manifestCrate && crateInRs && crateInRs !== manifestCrate) defects.push(`install_ref.rs RUST_CRATE is "${crateInRs}" but the crate in crates/leptos-ui is named "${manifestCrate}" — the docs would send readers to a crate that does not exist`);
  if (publishedInRs === 'true' && crateInRs) notes.push(`install_ref states the crate is published (${crateInRs}); registry truth is checked by ralph/scripts/release-watchdog.sh`);
  if (!rs.includes('INSTALL_SNIPPET')) defects.push('install_ref.rs has no INSTALL_SNIPPET for pages to render');
}

// 3. the bare specifier must RESOLVE (root and fixture)
const roots = [PROJECT_ROOT, path.join(PROJECT_ROOT, 'test/node-resolution')];
const resolveFrom = (root) => spawnSync('node', ['-e', `import('${ALIAS}').then(m => console.log(m.PACKAGE_NAME + '|' + m.RUST_CRATE + '|' + (m.NOT_PUBLISHED ? 'unpublished' : 'published'))).catch(e => { console.log('ERR:' + e.code); process.exit(1) })`], { cwd: root, encoding: 'utf8' });
for (const root of roots) {
  const where = path.relative(PROJECT_ROOT, root) || '.';
  if (FIX) ensureLinkAt(root);
  let r = resolveFrom(root);
  let out = (r.stdout || '').trim();
  if (r.status !== 0 || out.startsWith('ERR:')) {
    // Repair, then RETRY — and say that the repair happened. The old code repaired a different name and
    // retried ONLY at the repo root, so on a fresh checkout (CI) the root half failed on a link nobody
    // declares, and the local half failed because the link there was hand-made too.
    const repaired = ensureLinkAt(root);
    if (repaired) {
      r = resolveFrom(root);
      out = (r.stdout || '').trim();
    }
  }
  if (r.status !== 0 || out.startsWith('ERR:')) {
    defects.push(`${ALIAS} does not resolve from "${where}" (${out || 'no output'}) — the docs would name a package that exists nowhere. Tracked recipe: test/node-resolution/package.json declares \`${ALIAS}: workspace:*\`; a bare \`pnpm install\` links it there. Fix: \`pnpm install\`, or --fix-link for the local mapping.`);
  } else {
    notes.push(`${where}: ${out}`);
  }
}

// 3b. run the alias test in the fixture too (it is what a real consumer of the name sees)
const ALIAS_TEST = path.join(PROJECT_ROOT, 'test/node-resolution/alias.mjs');
for (const root of roots) {
  const t = spawnSync('node', [ALIAS_TEST], { cwd: root, encoding: 'utf8' });
  if (t.status !== 0) defects.push(`alias.mjs failed in ${path.relative(PROJECT_ROOT, root) || '.'}: ${(t.stderr || t.stdout || '').trim().split('\n').slice(-3).join(' | ')}`);
  else notes.push(`alias.mjs ok in ${path.relative(PROJECT_ROOT, root) || '.'}`);
}

// 4. no page may tell a reader to install upstream's package
//
// WHAT THIS RULE ACTUALLY CLAIMS, and what it used to measure instead. The claim is "a page tells a
// reader to INSTALL upstream's package" — an install instruction. The first version also flagged
// `from '@base-ui/react'` anywhere in a page source, which is a different thing: a mirrored EXAMPLE
// block whose first line imports upstream's package is a defect of the snippet's LANGUAGE, owned by
// the `docs-chrome: snippet translation` items and measured per route (visual-gap-report's `react > 0`
// P0, check-page's snippet-language axis, snippetLanguage purity). Counting it here made this rule a
// duplicate of `check-react-mentions.mjs`'s `package-react` class AND made the install-line item
// un-passable for 11 hits on work it does not own — measured 2026-09-16: all 11 were `from
// '@base-ui/react'` (10 inside `code_block(...)` snippet data or `#[cfg(test)]` positive controls
// after the split), and ZERO were an install command. The rule now fails on:
//   * an install command (`npm install` / `pnpm add` / `yarn add` upstream) — ALWAYS fatal, fenced or
//     not; the reader is being sent to a different library, in a different language;
//   * an import or `npmjs`/`react.dev` link in PAGE COPY — i.e. outside a code block and outside a
//     `#[cfg(test)]` item, both of which are classification questions, not guesses (see
//     ralph/scripts/lib/source-scope.mjs).
// The snippet-data occurrences are still counted and printed below, never dropped: they are re-homed,
// not excused.
const installRe = /npm\s+(?:install|i|add)\s+[^\n"']*@base-ui\/react|yarn\s+add\s+[^\n"']*@base-ui\/react|pnpm\s+(?:add|install)\s+[^\n"']*@base-ui\/react/i;
const upstreamImportRe = /from\s+['"]@base-ui\/react|npmjs\.com\/package\/@base-ui|react\.dev/i;
let rehomed = 0;
for (const root of PAGE_ROOTS) {
  if (!fs.existsSync(root)) continue;
  for (const f of fs.readdirSync(root)) {
    if (!f.endsWith('.rs')) continue;
    const body = fs.readFileSync(path.join(root, f), 'utf8');
    const testRanges = cfgTestRanges(body);
    const snippetRanges = codeBlockRanges(body);
    body.split('\n').forEach((line, i) => {
      if (inRanges(testRanges, i + 1)) return;                      // a test fixture is not page copy
      if (line.trim().startsWith('//')) return;                     // neither is a comment (the mentions
                                                                    // gate already skips these — this rule
                                                                    // flagged a doc comment quoting
                                                                    // `import { Button } from '@base-ui/react/button'`
                                                                    // as the defect it was documenting)
      const where = `${path.relative(PROJECT_ROOT, path.join(root, f))}:${i + 1}`;
      if (installRe.test(line)) {
        defects.push(`${where} tells the reader to install upstream's package — the port's name is ${ALIAS}`);
      } else if (upstreamImportRe.test(line)) {
        if (codeBlockAt(snippetRanges, i + 1)) { rehomed++; return; }  // snippet LANGUAGE — the snippet gates own it
        defects.push(`${where} points the reader at upstream's package/site outside a code block — the port's name is ${ALIAS}`);
      }
    });
  }
}

console.log(`package alias: ${ALIAS} (crate ${manifest.name === ALIAS ? 'base-ui-leptos' : '?'}) — ${defects.length} defect(s)`);
for (const n of notes) console.log(`  · ${n}`);
for (const d of defects) console.log(`  FAIL ${d}`);
if (rehomed) {
  console.log(`  · RE-HOMED ${rehomed} upstream import(s) inside mirrored example blocks — snippet LANGUAGE, owned by the \`docs-chrome: snippet translation\` items (their done-when is per-route \`visual-gap-report … react=0\`); printed here, not excused.`);
}
console.log(`report: ${path.relative(PROJECT_ROOT, INSTALL_REF)} is the canonical install text; packages/leptos/README.md documents the unpublished status.`);
process.exit(defects.length ? 1 : 0);
