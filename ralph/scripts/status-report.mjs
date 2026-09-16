#!/usr/bin/env node
/* eslint-disable no-console */

// status-report.mjs — the ONE command that answers "where is this loop".
//
// WHY THIS EXISTS
// ---------------
// Answering "is it moving" today meant a dozen bespoke shell one-liners per question, each costing a round trip
// and each inviting a fresh misreading — including one where I compared a UTC driver timestamp against local
// time, announced an 80-minute stalled iteration, and had to retract it. Worse, the numbers I quoted carried no
// validity flag: "example length 0" was repeated as a content gap while the instrument that produced it had
// dropped our own snippets from scoring.
//
// So this prints the canonical state in one place, and every measured number carries how old it is. A number
// whose instrument is broken is printed as INVALID, not as a value: no number here can be quoted as a page
// verdict unless the invariant self-test says the instrument is sound.

import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { evaluate } from './lib/metric-invariants.mjs';

const ROOT = process.cwd();
const now = Date.now();
const ago = (ts) => {
  const d = (now - ts) / 1000;
  if (d < 90) return `${Math.round(d)}s ago`;
  if (d < 5400) return `${Math.round(d / 60)}m ago`;
  if (d < 21600) return `${(d / 3600).toFixed(1)}h ago`;
  return `${Math.round(d / 3600)}h ago`;
};
const sh = (cmd, args, opts = {}) => {
  try { return execFileSync(cmd, args, { cwd: ROOT, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'], ...opts }).trim(); }
  catch { return ''; }
};
const readJson = (p) => { try { return JSON.parse(fs.readFileSync(path.join(ROOT, p), 'utf8')); } catch { return null; } };
const readJsonl = (p) => {
  try {
    return fs.readFileSync(path.join(ROOT, p), 'utf8').split('\n').filter(Boolean).map((l) => { try { return JSON.parse(l); } catch { return null; } }).filter(Boolean);
  } catch { return []; }
};

// ---- 1. planning count (ledger) — explicitly NOT a progress measure -------------------------------------
const todo = (() => { try { return fs.readFileSync(path.join(ROOT, 'TODO.md'), 'utf8'); } catch { return ''; } })();
const items = [...todo.matchAll(/^- \[([ x])\] ([^\n]+)\n((?: {6}[^\n]*\n)*)/gm)];
const done = items.filter((i) => i[1] === 'x');
const open = items.filter((i) => i[1] !== 'x');
const family = (s) => (s.split(':')[0] || 'other').trim();
const openByFamily = {};
for (const i of open) openByFamily[family(i[2])] = (openByFamily[family(i[2])] ?? 0) + 1;
const malformed = open.filter((i) => /status:\s*[^,\n]*,\s*$/m.test(i[3]));

console.log('PLANNING COUNT — ledger entries marked done (NOT a progress measure: a finished utils: clamp and a');
console.log('finished 1000-line component page both count as 1, and the cheap families are already closed)');
console.log(`  items done: ${done.length} / ${items.length}   (open ${open.length})`);
console.log(`  open by family: ${Object.entries(openByFamily).sort((a, b) => b[1] - a[1]).map(([k, v]) => `${k} ${v}`).join(' · ') || '(none)'}`);
if (malformed.length) console.log(`  ⚠ malformed status on ${malformed.length} item(s): ${malformed.map((m) => m[2].slice(0, 40)).join(' | ')}`);

// ---- 2. product measure (pages passing every axis), with instrument validity ------------------------------
const rows = readJsonl('ralph/generated/scorecard.jsonl');
const reportDir = path.join(ROOT, 'ralph/logs/visual');
const reports = fs.existsSync(reportDir) ? fs.readdirSync(reportDir).filter((f) => f.endsWith('-snippets.json')) : [];
let brokenInstruments = 0;
for (const f of reports) {
  const r = readJson(path.join('ralph/logs/visual', f));
  if (r && evaluate(r).length) brokenInstruments += 1;
}
const instrumentSound = brokenInstruments === 0 && reports.length > 0;
const axisFails = {};
let latest = null;
for (const d of rows) {
  if (!latest || (d.generatedAt ?? '') > (latest.generatedAt ?? '')) latest = d;
  for (const a of d.axes ?? []) if (a.status === 'FAIL') axisFails[a.axis] = (axisFails[a.axis] ?? 0) + 1;
}
const passing = rows.filter((d) => (d.axes ?? []).length && (d.axes ?? []).every((a) => a.status === 'PASS')).length;
const SIZE_AXES = ['example length', 'attribute density'];
console.log('\nPRODUCT MEASURE — pages passing EVERY axis (this is the number the port is judged by)');
console.log(`  ${passing} of ${rows.length} measured route(s) pass every axis` + (rows.length ? `  (latest measurement ${latest?.generatedAt ? ago(Date.parse(latest.generatedAt)) : 'unknown'})` : ''));
if (!instrumentSound) {
  console.log(`  ⚠ INSTRUMENT NOT SOUND — ${brokenInstruments} of ${reports.length} snippet report(s) violate a metric invariant.`);
  console.log('    Any axis derived from our snippet text is INVALID until `node ralph/scripts/gate-selftest.mjs` is green:');
  console.log(`      ${SIZE_AXES.join(', ')} — a broken ruler cannot fail a page, only lie about it.`);
}
console.log('  failing axes across measurements:' + (Object.keys(axisFails).length ? '' : ' (none)'));
for (const [axis, n] of Object.entries(axisFails).sort((a, b) => b[1] - a[1])) {
  const invalid = !instrumentSound && SIZE_AXES.includes(axis) ? '   ← INVALID (instrument broken)' : '';
  console.log(`    ${axis.padEnd(20)} ${n}/${rows.length}${invalid}`);
}

// ---- 3. the loop itself ---------------------------------------------------------------------------------
const iters = Number(sh('bash', ['-lc', "pgrep -cf 'ralph[-]baseui-hermes'"])) || 0;
const logs = (() => { try { return fs.readdirSync(path.join(ROOT, 'ralph/logs/stage3')).filter((f) => f.startsWith('hermes-')).map((f) => ({ f, t: fs.statSync(path.join(ROOT, 'ralph/logs/stage3', f)).mtimeMs })).sort((a, b) => b.t - a.t); } catch { return []; } })();
console.log(`\nTHE LOOP — iterations alive: ${iters}`);
if (logs[0]) {
  const started = logs[0].f.match(/(\d{8})-(\d{6})/);
  const when = started ? `${started[1].slice(0, 4)}-${started[1].slice(4, 6)}-${started[1].slice(6)} ${started[2].slice(0, 2)}:${started[2].slice(2, 4)}Z` : 'unknown';
  console.log(`  newest iteration log written ${ago(logs[0].t)} (started ${when})`);
  console.log(`  its log: ralph/logs/stage3/${logs[0].f}`);
}
const commits6h = (sh('git', ['log', '--since=6.hours', '--format=%ct']).split('\n').filter(Boolean).length) || 0;
const last = sh('git', ['log', '-1', '--format=%ct|%h %s']);
const [lts, lmsg] = last.split('|');
console.log(`  commits in the last 6h: ${commits6h} (${(commits6h / 6).toFixed(1)}/h)   newest: ${lmsg?.slice(0, 74) ?? '—'} ${lts ? `(${ago(Number(lts) * 1000)})` : ''}`);

// ---- 4. the site ----------------------------------------------------------------------------------------
const deploy = sh('bash', ['-lc', "cd " + ROOT + " && PATH=/data/bin:$PATH gh run list --repo noevaresearch/base-ui --workflow=deploy-docs-app.yml --limit 1 --json createdAt,conclusion,headSha --jq '.[0] | \"\\(.conclusion) \\(.headSha[0:9]) \\(.createdAt)\"' 2>/dev/null"]);
console.log('\nTHE SITE');
console.log(`  last docs deploy: ${deploy || 'unknown'}  →  https://baseui.noevaresearch.com`);

// ---- 5. what to do about it -----------------------------------------------------------------------------
console.log('\nREAD THIS WAY: planning count is how much of the PLAN is checked off; product measure is how much of');
console.log('the PRODUCT works. When they disagree by a lot, the plan is full of cheap units — look at the open');
console.log('by family line, not at the ratio.');
process.exit(0);
