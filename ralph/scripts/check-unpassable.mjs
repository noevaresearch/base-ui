#!/usr/bin/env node
/* eslint-disable no-console */

// check-unpassable.mjs — when an item cannot pass, say so AND say what to do.
//
// WHY THIS EXISTS
// ---------------
// Three separate tune, the same way: a gate demanded a number the item's own done-when never claimed.
//   * `docs-copy: install lines + React type columns` was held to >=95% prose coverage on 18 routes while its
//     done-when never mentioned copy → un-passable → red every iteration → blocked → re-picked → five
//     iterations, four ledger-only commits, React types 51 -> 51, commit rate 22/hr -> 2/hr.
//   * the same item carried a done-when clause ("page-level state is reported by the page scorecard") that no
//     gate could satisfy from its lane.
//   * `docs-parity` was gated on two axes its done-when did not claim.
// The loop did the honest thing each time — it reported failure — and then had no way to conclude "this is
// not a work problem". So it worked the item again. What it needed was a sentence like: *this item cannot
// pass, here is why, here is the smallest change that makes it passable, and here is the alternative.*
//
// WHAT IT DOES
//   1. reads the item's `done-when` and extracts the axes it CLAIMS (keyword map below);
//   2. determines the axes whose gates currently FAIL, from the gates themselves (cheap ones are run, browser
//      ones are read from their latest report so this stays fast enough to run every iteration);
//   3. classifies the mismatch:
//        UNPASSABLE  — a failing gate is on an axis the done-when does not claim (the item is being held to
//                      someone else's bar)
//        UNVERIFIABLE— the done-when claims an axis no gate measures (a claim nobody can check)
//        WORK        — failing axes are all claimed: this is ordinary unfinished work, keep going
//   4. prints a MOVE-FORWARD RECIPE for the chosen path, with the exact command to confirm.
//
// The recipe is deliberately prescriptive, because the loop's failure mode here is not laziness — it is
// having no move available other than "try the same thing again".
//
// USAGE
//   node ralph/scripts/check-unpassable.mjs --todo-id "<id>"
//   node ralph/scripts/check-unpassable.mjs --todo-id "<id>" --hard copy,parity   # simulate gate bars (testing)
// Exit: 0 passable/ordinary work, 1 the item cannot pass as scoped, 3 nothing to check.
//
// NOTE: `--hard` exists so the detector itself can be tested. A detector that has only ever reported "fine"
// is not evidence of anything.

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const ROOT = path.resolve(import.meta.dirname, '../..');
const VIS = path.join(ROOT, 'ralph/logs/visual');

function arg(name) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] && !process.argv[i + 1].startsWith('--') ? process.argv[i + 1] : null;
}
const todoId = arg('todo-id');
if (!todoId) { console.error('usage: check-unpassable.mjs --todo-id "<id>" [--hard a,b]'); process.exit(3); }

// ---- 1. what the item CLAIMS to be held to ------------------------------------------------------
const AXIS_CLAIM = {
  parts: /part surface|Component::Part|namespaced part|`Component::/i,
  'namespaced-path': /namespaced path|namespaced component|`<[A-Z][A-Za-z]+::/i,
  snippets: /snippet|react=0|react: 0|example|view! markup/i,
  length: /length similarity|80% of upstream|example length/i,
  attributes: /attribute density|attribute/i,
  copy: /copy coverage|prose coverage|copy fidelity/i,
  parity: /page parity|visual fidelity|>=90|≥90/i,
  widget: /widget/i,
  mentions: /React mention|React API|React type|leptos.?only|ReactElement|no react/i,
  alias: /install|alias|package name|base-ui-leptos/i,
  structure: /structure|mounts|headings/i,
  tests: /test|hygiene|coverage vocabulary/i,
};

const todo = fs.readFileSync(path.join(ROOT, 'TODO.md'), 'utf8');
const start = todo.indexOf(`- [ ] ${todoId}`) >= 0 ? todo.indexOf(`- [ ] ${todoId}`) : todo.indexOf(`- [x] ${todoId}`);
if (start < 0) { console.error(`no ledger item "${todoId}"`); process.exit(3); }
let end = todo.indexOf('\n- [', start + 10);
if (end < 0) end = todo.length;
const block = todo.slice(start, end);
const doneWhen = (block.match(/^ {6}done-when: ([\s\S]*?)(?=\n {6}[a-z-]+:)/m) || [, ''])[1];
const status = (block.match(/^ {6}status: (.+)$/m) || [, '?'])[1].trim();
const routes = (block.match(/^ {6}routes: (.+)$/m) || [, ''])[1].split(',').map((s) => s.trim()).filter(Boolean);

const claimed = Object.entries(AXIS_CLAIM).filter(([, re]) => re.test(doneWhen)).map(([a]) => a);

// ---- 2. which axes are actually failing? --------------------------------------------------------
const forced = (arg('hard') || '').split(',').map((s) => s.trim()).filter(Boolean);

// ENFORCED axes are declared by the gate file itself (`# axis: name` above each hard clause), so this cannot
// drift from what actually runs. The first version of this detector inferred them from "which axis is failing
// somewhere" and promptly reported an item UNPASSABLE on copy/length/attributes that its run never enforced —
// a detector that cries wolf is the same defect it exists to catch.
function enforcedAxes(id) {
  const reg = fs.readFileSync(path.join(ROOT, 'ralph/scripts/run-regression.sh'), 'utf8').split('\n');
  const out = new Set();
  let pending = null;
  for (const line of reg) {
    const m = line.match(/#\s*axis:\s*([a-z,-]+)/);
    if (m) { pending = m[1].split(','); continue; }
    if (!pending) continue;
    const cond = line.match(/TODO_ID"\s*==\s*([^\]]+)/g);
    if (cond) {
      const patterns = cond.map((c) => c.replace(/^TODO_ID"\s*==\s*/, '').trim().replace(/^"|"$/g, '').split('||')[0]);
      for (const pat of patterns) {
        const rx = new RegExp('^' + pat.replace(/[-/\\^$*+?.()|[\]{}]/g, '\\$&').replace(/\\\*/g, '.*') + '$');
        if (rx.test(id)) for (const a of pending) out.add(a.trim());
      }
    }
    if (/then\s*$/.test(line) || /^\s*else/.test(line)) pending = null;   // clause ended
  }
  return [...out];
}
const enforced = forced.length ? new Set(forced) : new Set(enforcedAxes(todoId));
// `--hard X` must ALSO mark X as failing: it exists to simulate "this bar is enforced and red", which is how
// the detector is tested against the situation that actually stalled the loop. The first version only added it
// to the enforced set, so the simulated case reported WORK — a detector whose own test says "all fine" is
// worse than no detector, because it would pass review.
const failing = new Set(forced);
const detail = [];

function readLatest(name, kind) {
  const p = path.join(VIS, `${name}-${kind}.json`);
  try { return JSON.parse(fs.readFileSync(p, 'utf8')); } catch { return null; }
}

if (!forced.length) {
  // cheap gates: run them (no browser)
  const r1 = spawnSync('node', ['ralph/scripts/check-react-mentions.mjs', '--source'], { cwd: ROOT, encoding: 'utf8' });
  if (r1.status !== 0) { failing.add('mentions'); detail.push('check-react-mentions --source is red (React APIs/package names in the port\'s own page content)'); }
  const r2 = spawnSync('node', ['ralph/scripts/check-package-alias.mjs'], { cwd: ROOT, encoding: 'utf8' });
  if (r2.status !== 0) { failing.add('alias'); detail.push('check-package-alias is red (install references or a page naming upstream\'s package)'); }
  // browser gates: read their latest reports rather than re-running (each costs a Chrome launch)
  for (const r of routes.slice(0, 18)) {
    const n = r.split('/').pop();
    const snip = readLatest(n, 'snippets');
    if (snip?.metrics) {
      if (enforced.has('snippets') && (snip.metrics.reactToReactBlocks ?? 0) > 0) { failing.add('snippets'); detail.push(`${n}: ${snip.metrics.reactToReactBlocks} snippet block(s) are upstream's React code`); }
      if (enforced.has('length') && (snip.metrics.lengthSimilarity ?? 100) < 80) { failing.add('length'); detail.push(`${n}: example length ${snip.metrics.lengthSimilarity}% (bar 80%)`); }
      if (enforced.has('attributes') && snip.metrics.upstreamMeanAttrs && (snip.metrics.leptosMeanAttrs / snip.metrics.upstreamMeanAttrs) < 0.8) { failing.add('attributes'); detail.push(`${n}: attribute density ${(snip.metrics.leptosMeanAttrs / snip.metrics.upstreamMeanAttrs).toFixed(2)}x (bar 0.8x)`); }
    }
    const copy = readLatest(n, 'copy');
    if (enforced.has('copy') && copy && typeof copy.coverage === 'number' && copy.coverage < 95) { failing.add('copy'); detail.push(`${n}: copy coverage ${copy.coverage}% (bar 95%)`); }
  }
}

const unclaimedFails = [...failing].filter((a) => !claimed.includes(a));
const unverifiable = claimed.filter((a) => !['parts', 'namespaced-path', 'snippets', 'length', 'attributes', 'copy', 'parity', 'widget', 'mentions', 'alias', 'structure', 'tests'].includes(a));

// ---- 3. verdict + recipe ------------------------------------------------------------------------
console.log(`UNPASSABLE CHECK — ${todoId}`);
console.log(`  status: ${status}   routes: ${routes.length || '(none)'}`);
console.log(`  done-when claims: ${claimed.join(', ') || '(nothing recognisable)'}`);
console.log(`  gates enforced for THIS item: ${[...enforced].join(', ') || '(none — nothing hard-gated)'}`);
console.log(`  of those, currently failing: ${[...failing].join(', ') || '(none detected)'}`);
if (detail.length) for (const d of detail.slice(0, 8)) console.log(`     · ${d}`);

let verdict = 'WORK';
if (unclaimedFails.length) verdict = 'UNPASSABLE';
else if (unverifiable.length && !failing.size) verdict = 'UNVERIFIABLE';

console.log(`\n  VERDICT: ${verdict}`);
if (verdict === 'UNPASSABLE') {
  console.log(`  WHY: ${unclaimedFails.join(', ')} ${unclaimedFails.length === 1 ? 'is' : 'are'} enforced for this item but the`);
  console.log(`  done-when does not claim ${unclaimedFails.length === 1 ? 'it' : 'them'} — the item is being held to a bar that belongs to`);
  console.log(`  another item's lane, so no amount of correct work here turns the regression green.`);
  console.log(`\n  MOVE-FORWARD RECIPE — pick ONE, then stop working this item:`);
  console.log(`    (a) NARROW THE GATE if the bar truly is someone else's: edit ralph/scripts/run-regression.sh so the`);
  console.log(`        clause for ${unclaimedFails.join('/')} matches only the item ids whose done-when claims it, keeping it`);
  console.log(`        measured-and-printed for the rest. Confirm with:`);
  console.log(`          bash ralph/scripts/run-regression.sh "${todoId}"`);
  console.log(`    (b) SPLIT THE ITEM if the scope is real but too big: create a new item for ${unclaimedFails.join('/')}`);
  console.log(`        with its own done-when and blocked-by, then narrow this item's done-when to what it can reach.`);
  console.log(`    (c) If neither is right, record a blocked-reason naming this verdict — and do NOT re-pick this item`);
  console.log(`        expecting a different outcome; the ledger is where that decision belongs.`);
} else if (verdict === 'UNVERIFIABLE') {
  console.log(`  WHY: the done-when claims ${unverifiable.join(', ')}, which no gate measures — so "done" would be an`);
  console.log(`  unverifiable claim. Add the gate, or reword the clause to something measurable.`);
} else {
  console.log(`  Every failing axis is one this item claims, so this is ordinary unfinished work — keep going (see the`);
  console.log(`  per-axis reports in ralph/logs/visual/ and the page scorecard for the work list).`);
}
process.exit(verdict === 'WORK' ? 0 : 1);
