#!/usr/bin/env node
/* eslint-disable no-console */

// scorecard-latest.mjs — what the RENDERED measurement says about a route, without booting a browser.
//
// WHY THIS EXISTS
// ---------------
// The loop's own regression defers its browser gates to CI (measure-port.yml), because this box is a 4 GB cgroup
// where a chromium (~1.4 GB) plus a rust build (~1.5 GB) runs at 95% and the kernel starts killing processes —
// usually the iteration's own tool call. But deferring in run-regression.sh was only half the job: the prompt still
// told the agent to run `check-page.mjs`, `snippet-ergonomics.mjs`, `check-visual-budget.mjs` and
// `check-copy-fidelity.mjs` directly, and an agent follows its prompt. So the browser came back.
//
// This gives the loop the same information the browser gates produced — parsed from the measurement CI already ran
// and committed — as a one-liner, so "check the rendered parity" stops meaning "start Chromium".
//
// Usage: node ralph/scripts/scorecard-latest.mjs --route react/components/checkbox [--json]

import fs from 'node:fs';
import path from 'node:path';

const ROOT = process.cwd();
const FILE = path.join(ROOT, 'ralph/generated/scorecard.jsonl');
const arg = (n) => {
  const i = process.argv.indexOf(`--${n}`);
  return i >= 0 && process.argv[i + 1] && !process.argv[i + 1].startsWith('--') ? process.argv[i + 1] : null;
};
const route = arg('route');
const jsonOut = process.argv.includes('--json');
if (!route) {
  console.error('usage: scorecard-latest.mjs --route react/components/<name> [--json]');
  process.exit(2);
}

let rows = [];
try {
  rows = fs.readFileSync(FILE, 'utf8').split('\n').filter((l) => l.trim()).map((l) => {
    try { return JSON.parse(l); } catch { return null; }
  }).filter((r) => r && r.route === route && Array.isArray(r.axes));
} catch {
  console.error(`scorecard-latest: could not read ${FILE}`);
  process.exit(2);
}
if (!rows.length) {
  // Not a failure of the route: it means CI has not measured this route yet. Say exactly that, because the honest
  // signal is UNMEASURED and never passes — an item must not be closed on the strength of a missing measurement.
  console.log(`${route}: UNMEASURED — no CI measurement on record yet (see .github/workflows/measure-port.yml)`);
  process.exit(1);
}
rows.sort((a, b) => String(a.generatedAt ?? '').localeCompare(String(b.generatedAt ?? '')));
const latest = rows[rows.length - 1];
const ageMin = Math.round((Date.now() - Date.parse(latest.generatedAt)) / 60000);

if (jsonOut) {
  process.stdout.write(JSON.stringify(latest) + '\n');
  process.exit(latest.verdict === 'PASS' ? 0 : 1);
}

console.log(`${route} — measured ${ageMin}m ago by CI (${latest.generatedAt})`);
for (const a of latest.axes) {
  const val = a.value === null || a.value === undefined ? '' : ` = ${a.value}`;
  console.log(`  ${a.status.padEnd(10)} ${a.axis.padEnd(20)} bar ${String(a.bar).padEnd(20)}${val}`);
}
console.log(`  verdict: ${latest.verdict}  (fails ${latest.fails}, unmeasured ${latest.unmeasured})`);
if (ageMin > 180) {
  console.log(`  NOTE: this measurement is ${Math.round(ageMin / 60)}h old. Stale numbers are still the truth about an`);
  console.log('        older commit — do not treat them as evidence for work committed since.');
}
process.exit(latest.verdict === 'PASS' ? 0 : 1);
