#!/usr/bin/env node
/* eslint-disable no-console */

// Sandbox parity guard — does the sandbox render the SAME demo the docs page renders?
//
// WHY THIS EXISTS
// ---------------
// `examples/leptos-sandbox` cannot depend on `crates/docs-app` (the docs app resolves
// `base-ui-leptos` from the workspace path; the sandbox must resolve the PUBLISHED crate from
// crates.io — that is the whole point of it), so the demo bodies exist TWICE: once in
// `crates/docs-app/src/pages/<component>_page.rs`, once in `examples/leptos-sandbox/src/demos.rs`.
// A demo fixed on the docs page and not in the copy is invisible: every existing gate stays green,
// the docs render the fix, and the sandbox keeps serving the old component to the reader who is
// editing it. That is the "gap with no owner" class this repo treats as P0.
//
// This is the INTERIM guard, not the fix. The structural fix is one demos crate consumed by both
// (docs workspace patching `base-ui-leptos` to the local path via `[patch.crates-io]`, the sandbox
// resolving the same crate from crates.io) — the single-artifact principle upstream already applies
// to its rendered demo, its code panel and its StackBlitz export
// (`docs/src/utils/demoExportOptions.ts:606-612` reads `variant.source` + `variant.extraFiles`,
// delivered by the generated client provider in `@mui/internal-docs-infra/cli/ensureDemoClients.mjs`).
// Until that lands, drift is a build failure here instead of a surprise in the browser.
//
// COVERAGE IS ENFORCED: every `slug` in the sandbox registry must have a PAIRS entry below. A demo
// added to the sandbox without a pair FAILS this guard rather than passing silently, so the guard
// cannot decay into "checks the one demo it was written for".
//
// USAGE
//   node ralph/scripts/check-sandbox-parity.mjs [--slug accordion-hero] [--verbose]
// Exit code is non-zero on any drift, so it is usable as a loop/CI gate.

import fs from 'fs/promises';
import path from 'path';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const SANDBOX_REGISTRY = 'examples/leptos-sandbox/src/demos.rs';

// slug → where the same demo lives on each side. `docsComponent` is the docs page's component name;
// the sandbox's component name is derived from the registry entry's `mount` closure, because that is
// what actually gets mounted.
const PAIRS = {
  'accordion-hero': {
    docs: { file: 'crates/docs-app/src/pages/accordion_page.rs', component: 'AccordionHeroDemo' },
  },
};

const args = process.argv.slice(2);
const onlySlug = args.includes('--slug') ? args[args.indexOf('--slug') + 1] : null;
const verbose = args.includes('--verbose');

const stripComments = (text) =>
  text
    .split('\n')
    .filter((line) => !/^\s*(\/\/|\/\*|\*)/.test(line))
    .join('\n');

const normalize = (text) => stripComments(text).replace(/\s+/g, ' ').trim();

/** Extract a `fn NAME(...) -> ... { ... }` body (brace-matched, string-aware) with its line number. */
function extractFn(src, name) {
  const re = new RegExp(`^\\s*(?:pub\\s+)?(?:async\\s+)?fn\\s+${name}\\s*[<(]`, 'm');
  const m = re.exec(src);
  if (!m) return null;
  const line = src.slice(0, m.index).split('\n').length;
  let i = src.indexOf('{', m.index);
  if (i < 0) return null;
  let depth = 0;
  let inStr = false;
  let inChar = false;
  let escaped = false;
  for (let j = i; j < src.length; j += 1) {
    const c = src[j];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (c === '\\') {
      escaped = true;
      continue;
    }
    if (inStr) {
      if (c === '"') inStr = false;
      continue;
    }
    if (inChar) {
      if (c === "'") inChar = false;
      continue;
    }
    if (c === '"') {
      inStr = true;
      continue;
    }
    if (c === "'") {
      // A lifetime (`&'static`) is not a char literal; only treat it as one when it closes inline.
      if (!/^'[^']*'/.test(src.slice(j))) inChar = true;
      continue;
    }
    if (c === '{') depth += 1;
    if (c === '}') {
      depth -= 1;
      if (depth === 0) return { text: src.slice(i, j + 1), line };
    }
  }
  return null;
}

/** Extract a `const NAME: &str = "...";` value with its line number. */
function extractConst(src, name) {
  const re = new RegExp(`^\\s*(?:pub\\s+)?const\\s+${name}\\s*:`, 'm');
  const m = re.exec(src);
  if (!m) return null;
  const line = src.slice(0, m.index).split('\n').length;
  const eq = src.indexOf('=', m.index);
  if (eq < 0) return null;
  let inStr = false;
  let escaped = false;
  for (let j = eq; j < src.length; j += 1) {
    const c = src[j];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (c === '\\') {
      escaped = true;
      continue;
    }
    if (c === '"') inStr = !inStr;
    if (c === ';' && !inStr) return { text: src.slice(eq + 1, j), line };
  }
  return null;
}

/** Item names a chunk of Rust text mentions — used to follow a demo's references transitively. */
function referencedItems(text, candidates) {
  const found = [];
  for (const name of candidates) {
    if (new RegExp(`\\b${name}\\b`).test(text)) found.push(name);
  }
  return found;
}

/** All item names (fns + consts) in a Rust file. */
function itemNames(src) {
  const names = new Set();
  for (const m of src.matchAll(/^\s*(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/gm)) names.add(m[1]);
  for (const m of src.matchAll(/^\s*(?:pub\s+)?const\s+([A-Za-z_][A-Za-z0-9_]*)/gm)) names.add(m[1]);
  return [...names];
}

/** Read the sandbox registry: slug → mounted component name. */
async function readRegistry() {
  const src = await fs.readFile(path.join(PROJECT_ROOT, SANDBOX_REGISTRY), 'utf8');
  const entries = [];
  const re = /slug:\s*"([^"]+)"[\s\S]*?mount:\s*\|\|\s*view!\s*\{\s*<([A-Za-z_][A-Za-z0-9_]*)/g;
  for (const m of src.matchAll(re)) entries.push({ slug: m[1], component: m[2] });
  return { src, entries };
}

/** Collect the demo's items on one side: the component plus everything it references, transitively. */
function collectItems(src, componentName, extraNames) {
  const candidates = itemNames(src);
  const wanted = new Set([componentName, ...extraNames]);
  const items = new Map();
  let frontier = [componentName];
  while (frontier.length) {
    const name = frontier.pop();
    if (items.has(name)) continue;
    const found = extractFn(src, name) ?? extractConst(src, name);
    if (!found) continue;
    items.set(name, found);
    frontier = frontier.concat(referencedItems(found.text, candidates).filter((n) => !items.has(n) && !wanted.has(n)));
  }
  // Extra names are helpers shared by several demos on the docs page (e.g. the FAQ row builder); they
  // are part of the demo's rendering, so they are compared even though the component only uses them
  // through another helper.
  for (const name of extraNames) {
    if (items.has(name)) continue;
    const found = extractFn(src, name) ?? extractConst(src, name);
    if (found) items.set(name, found);
  }
  return items;
}

const problems = [];
const notes = [];

const registry = await readRegistry();
if (registry.entries.length === 0) {
  problems.push(`no demo entries parsed out of ${SANDBOX_REGISTRY} — the registry format changed and this guard is now blind`);
}

for (const entry of registry.entries) {
  if (onlySlug && entry.slug !== onlySlug) continue;
  const pair = PAIRS[entry.slug];
  if (!pair) {
    problems.push(
      `sandbox demo "${entry.slug}" (${entry.component}) has no PAIRS entry — either add its docs page to this guard or the copy is unguarded`,
    );
    continue;
  }

  const docsSrc = await fs.readFile(path.join(PROJECT_ROOT, pair.docs.file), 'utf8');
  const sandboxSrc = registry.src;

  const docsItems = collectItems(docsSrc, pair.docs.component, pair.docs.extraNames ?? []);
  const sandboxItems = collectItems(sandboxSrc, entry.component, pair.docs.extraNames ?? []);

  console.log(`\nsandbox parity — ${entry.slug}`);
  console.log(`  docs:    ${pair.docs.file} (${pair.docs.component})`);
  console.log(`  sandbox: ${SANDBOX_REGISTRY} (${entry.component})`);

  const names = [...new Set([...docsItems.keys(), ...sandboxItems.keys()])].sort();
  let compared = 0;

  // The component is compared as ONE logical item across its two names (the docs page's
  // `AccordionHeroDemo` vs the registry's mount `AccordionHero`), so the union above must not also
  // treat each name as an independent item — the first version did, and reported "MISSING
  // AccordionHero / MISSING AccordionHeroDemo" for a demo whose body was in fact identical.
  {
    const a = docsItems.get(pair.docs.component);
    const b = sandboxItems.get(entry.component);
    compared += 1;
    if (!a || !b) {
      problems.push(
        `${entry.slug}: component body missing on one side (docs "${pair.docs.component}"=${Boolean(a)}, sandbox "${entry.component}"=${Boolean(b)})`,
      );
      console.log(`  MISSING  component (docs=${Boolean(a)}, sandbox=${Boolean(b)})`);
    } else {
      const norm = (t) =>
        t
          .replace(new RegExp(pair.docs.component, 'g'), 'DEMO')
          .replace(new RegExp(entry.component, 'g'), 'DEMO');
      if (norm(normalize(a.text)) !== norm(normalize(b.text))) {
        problems.push(
          `${entry.slug}: component body differs\n    docs:    ${pair.docs.file}:${a.line}\n    sandbox: ${SANDBOX_REGISTRY}:${b.line}\n    docs   : ${normalize(a.text).slice(0, 240)}\n    sandbox: ${normalize(b.text).slice(0, 240)}`,
        );
        console.log(`  DIFFERS  component (docs:${a.line} vs sandbox:${b.line})`);
      } else {
        console.log(`  OK       component (${pair.docs.component} == ${entry.component})`);
      }
    }
  }

  for (const name of names) {
    if (name === entry.component || name === pair.docs.component) continue;

    const a = docsItems.get(name);
    const b = sandboxItems.get(name);
    compared += 1;
    if (!a || !b) {
      problems.push(`${entry.slug}: "${name}" exists only on the ${a ? 'docs' : 'sandbox'} side — the two demos have diverged structurally`);
      console.log(`  UNPAIRED ${name} (${a ? 'docs only' : 'sandbox only'})`);
      continue;
    }
    if (normalize(a.text) !== normalize(b.text)) {
      problems.push(
        `${entry.slug}: "${name}" differs\n    docs:    ${pair.docs.file}:${a.line}\n    sandbox: ${SANDBOX_REGISTRY}:${b.line}\n    docs   : ${normalize(a.text).slice(0, 240)}\n    sandbox: ${normalize(b.text).slice(0, 240)}`,
      );
      console.log(`  DIFFERS  ${name} (docs:${a.line} vs sandbox:${b.line})`);
    } else {
      console.log(`  OK       ${name}`);
    }
  }
  notes.push(`${entry.slug}: ${compared} item(s) compared`);
  if (verbose) console.log(`  (items: ${names.join(', ')})`);
}

console.log('');
for (const n of notes) console.log(`  ${n}`);

// --- coverage, the other direction ------------------------------------------------------------
// The pairs above only prove that what IS in the registry matches the docs. A demo that exists on a
// docs page and was never registered in the sandbox is invisible to that check — so the count is
// reported here every run. It is a NOTE by default (the loop must not be blocked by 20 unregistered
// demos) and a FAILURE under `--strict`, which is what the coverage item's done-when uses.
const docsDemos = new Set();
const pagesDir = path.join(PROJECT_ROOT, 'crates/docs-app/src/pages');
for (const file of (await fs.readdir(pagesDir)).filter((f) => f.endsWith('.rs'))) {
  const src = await fs.readFile(path.join(pagesDir, file), 'utf8');
  // A Set, not an array: some pages declare the same demo fn more than once (a test module inside the
  // page file re-declares it), and counting those twice inflated the uncovered list the first run.
  for (const m of src.matchAll(/pub\s+fn\s+([A-Za-z0-9_]*Demo)\s*\(/g)) {
    docsDemos.add(`${file}::${m[1]}`);
  }
}
const docsDemoList = [...docsDemos].sort();
const coveredDocsDemos = new Set(Object.values(PAIRS).map((p) => p.docs.component));
const uncovered = docsDemoList.filter((d) => !coveredDocsDemos.has(d.split('::')[1]));
console.log(
  `coverage: ${docsDemoList.length} demo component(s) on docs pages, ${Object.keys(PAIRS).length} with a sandbox pair, ${uncovered.length} uncovered`,
);
console.log(
  '          (counts the `pub fn *Demo` shape only — a demo composed inline inside a page fn is not visible to this count)',
);
if (uncovered.length) {
  console.log(`  uncovered (first 10): ${uncovered.slice(0, 10).join(', ')}`);
  if (args.includes('--strict')) {
    problems.push(`${uncovered.length} docs demo(s) have no sandbox pair — the registry does not cover them (${uncovered.slice(0, 5).join(', ')}${uncovered.length > 5 ? ', …' : ''})`);
  }
}

if (problems.length) {
  console.log(`\nRESULT: RED — ${problems.length} parity problem(s):\n`);
  for (const p of problems) console.log(`  * ${p}\n`);
  console.log('The sandbox exists so a reader can edit the ported demo against the PUBLISHED crate;');
  console.log('when its body drifts from the docs page, the reader edits a demo nobody else is testing.');
  process.exit(1);
}
console.log('RESULT: GREEN — the sandbox renders the same demo bodies the docs pages render.');
