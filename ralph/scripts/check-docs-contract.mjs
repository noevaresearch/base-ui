#!/usr/bin/env node
/* eslint-disable no-console */

// check-docs-contract.mjs — spec-level feedback for mirrored docs pages.
//
// WHY THIS EXISTS
// ---------------
// The structure gates (playwright-diff.mjs, check-visual-budget.mjs) watch a mirrored page's
// *shape*. They cannot see that the page teaches the wrong framework: the checkbox page passed
// every structural check while carrying upstream's React source in all five of its code blocks.
// Behaviour fidelity is therefore a SPEC obligation (`specs/docs-content/CONTRACT.md`) and this
// script is how the loop hears about it: it reports which mirrored pages have not yet declared
// their `## Snippet & behaviour contract` section, and which pages the ledger has already marked
// done without one.
//
// ADVISORY BY DEFAULT, ON PURPOSE
// -------------------------------
// Exit 0 with a report unless `--strict`. The contract sections must be authored deliberately
// (they carry citations and behavioural observables), so `run-regression.sh` reports this rather
// than blocking an iteration on legacy pages. The Phase E item
// `docs-spec: snippet & behaviour contract on every mirrored page` is where `--strict` exit 0 is
// the acceptance measurement.
//
// USAGE
//   node ralph/scripts/check-docs-contract.mjs            # report missing contracts
//   node ralph/scripts/check-docs-contract.mjs --strict   # exit 1 when any are missing
//   node ralph/scripts/check-docs-contract.mjs --todo-id "<id>"

import fs from 'node:fs';
import path from 'node:path';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const TODO_PATH = path.join(PROJECT_ROOT, 'TODO.md');
const SPECS_ROOT = path.join(PROJECT_ROOT, 'specs/docs-content');
const SECTION = '## Snippet & behaviour contract';
const STRICT = process.argv.includes('--strict');

function arg(name, dflt) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] && !process.argv[i + 1].startsWith('--') ? process.argv[i + 1] : dflt;
}

// read the ledger: which mirrored pages exist, which are done, where their specs live
const todo = fs.readFileSync(TODO_PATH, 'utf8');
const items = [];
for (const block of todo.split(/\n(?=- \[[ x]\] )/)) {
  const head = block.match(/^- \[( |x)\] (.+)$/m);
  if (!head) continue;
  const id = head[2].trim();
  if (!/^docs-content: /.test(id)) continue;
  const field = (n) => (block.match(new RegExp(`^\\s+${n}: (.*)$`, 'm')) || [])[1] || '';
  items.push({ id, checked: head[1] === 'x', specs: field('specs'), status: field('status') });
}

function contractState(item) {
  // the item's specs field lists mined spec files; the page spec is the one that carries the contract
  const files = (item.specs.match(/specs\/docs-content\/[^\s,]+/g) || []).slice();
  const pageSpecs = files.filter((f) => /page\.md$/.test(f));
  if (!pageSpecs.length) return { state: 'no-page-spec', files };
  const withContract = pageSpecs.filter((f) => {
    try { return fs.readFileSync(path.join(PROJECT_ROOT, f), 'utf8').includes(SECTION); } catch { return false; }
  });
  if (withContract.length === pageSpecs.length) return { state: 'ok', files: pageSpecs };
  if (withContract.length) return { state: 'partial', files: pageSpecs };
  return { state: 'missing', files: pageSpecs };
}

const only = arg('todo-id', null);
const rows = items
  .filter((i) => !only || i.id === only || i.id === (items.find((x) => x.id === only)?.id ?? ''))
  .map((i) => ({ ...i, contract: contractState(i) }));

const ok = rows.filter((r) => r.contract.state === 'ok');
const missing = rows.filter((r) => r.contract.state === 'missing');
const partial = rows.filter((r) => r.contract.state === 'partial');
const noSpec = rows.filter((r) => r.contract.state === 'no-page-spec');

console.log(`docs-contract: ${rows.length} mirrored page item(s) — ${ok.length} carry the contract, ` +
  `${missing.length} missing, ${partial.length} partial, ${noSpec.length} without a page spec`);

const doneWithout = [...missing, ...partial].filter((r) => r.status === 'done');
if (doneWithout.length) {
  console.log('\nMarked done without a snippet & behaviour contract (these are the silent ones — see');
  console.log('specs/docs-content/CONTRACT.md; each needs its contract table authored from the');
  console.log('behaviour spec it cites):');
  for (const r of doneWithout) console.log(`  - ${r.id}  (${r.contract.files.join(', ')})`);
}
if (missing.length > doneWithout.length) {
  const rest = missing.filter((r) => r.status !== 'done');
  console.log(`\nNot started yet (${rest.length}):`);
  for (const r of rest.slice(0, 20)) console.log(`  - ${r.id}`);
}
if (ok.length) {
  console.log(`\nContracted (${ok.length}): ${ok.map((r) => r.id.replace('docs-content: ', '')).join(', ')}`);
}

if (STRICT && (missing.length || partial.length)) {
  console.error(`\nFAIL (--strict): ${missing.length + partial.length} mirrored page(s) lack a ${SECTION} section.`);
  process.exit(1);
}
console.log(STRICT ? '\ndocs contract OK' : '\n(advisory — pass --strict to enforce)');
