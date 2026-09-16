#!/usr/bin/env node
/* eslint-disable no-console */

// audit-instruments.mjs — look for the disease, not the symptom.
//
// The phantom dependency ("Phase A complete" blocking 33 items) was one instance of a class: a REFERENCE or a CHECK
// that cannot do what it claims. This walks the instrument layer mechanically and reports, per class:
//
//   1. DEAD GATES        a gate script nothing invokes — it measures nothing and nobody notices
//   2. ADVISORY GATES    a gate wired with `|| true`, i.e. a check that can never fail its caller
//   3. DANGLING REFS     specs:/crate:/routes: values in the ledger that point at files or routes that do not exist
//   4. PHANTOM DEPS      blocked-by targets that resolve to no item id (the class that parked the component lane)
//   5. DOUBLE-SPAWNED    scripts that invoke other gates, so one run measures the same thing twice (browser cost)
//   6. UNREACHABLE BARS  a bar that no measured value could ever satisfy (0/0 ratios, floor above the maximum)
//
// Read-only. Nothing here writes, builds, or starts a browser — it is deliberately cheap so it can be run often,
// which is the only way a class of defect stops recurring.

import fs from 'node:fs';
import path from 'node:path';

const ROOT = process.cwd();
const S = (...p) => path.join(ROOT, ...p);
const read = (p) => { try { return fs.readFileSync(p, 'utf8'); } catch { return ''; } };
const exists = (p) => fs.existsSync(path.isAbsolute(p) ? p : S(p));

const findings = [];
const add = (cls, detail) => findings.push({ cls, detail });

// ---- the corpus: gate scripts, orchestrators, the ledger, the workflows ---------------------------------
const gateDir = S('ralph/scripts');
const gates = fs.existsSync(gateDir)
  ? fs.readdirSync(gateDir).filter((f) => /\.(mjs|sh)$/.test(f) && !f.startsWith('lib'))
  : [];
const orchestrators = ['ralph/scripts/run-regression.sh', 'ralph/scripts/scorecard-sweep.sh', 'ralph/driver/ralph-baseui-hermes.sh', '/data/scripts/ralph-watchdog.sh']
  .map(read).join('\n');
const workflows = (() => {
  try {
    return fs.readdirSync(S('.github/workflows')).map((f) => read(S('.github/workflows', f))).join('\n');
  } catch { return ''; }
})();
const prompt = read(S('ralph/prompts/stage3-forward-loop.md'));
const callers = orchestrators + workflows + prompt;
const todo = read(S('TODO.md'));

// ---- 1. dead gates --------------------------------------------------------------------------------------
for (const g of gates) {
  if (g.startsWith('audit-') || g.startsWith('build-status') || g.startsWith('status-report')) continue;
  if (!callers.includes(g)) add('DEAD GATE', `ralph/scripts/${g} — nothing invokes it (regression, sweep, driver, watchdog, workflows, prompt)`);
}

// ---- 2. advisory gates: a check that cannot fail --------------------------------------------------------
const regression = read(S('ralph/scripts/run-regression.sh'));
for (const line of regression.split('\n')) {
  if (!/node ralph\/scripts\/[a-z-]+\.mjs/.test(line)) continue;
  if (!/\|\|\s*true/.test(line)) continue;
  const gate = (line.match(/node ralph\/scripts\/([a-z-]+\.mjs)/) || [])[1];
  add('ADVISORY GATE', `${gate} is invoked with "|| true" — it prints but cannot fail the run`);
}

// ---- 3. dangling references in the ledger ---------------------------------------------------------------
for (const m of todo.matchAll(/^\s+specs:\s*(.+)$/gm)) {
  for (const raw of m[1].split(',')) {
    const ref = raw.trim().replace(/\s*#.*$/, '');
    if (!ref || /^<|…/.test(ref)) continue;
    if (/^[\w./-]+$/.test(ref) && !exists(ref)) add('DANGLING REF', `specs: ${ref} does not exist on disk`);
  }
}
for (const m of todo.matchAll(/^\s+crate:\s*([^\s#]+)/gm)) {
  const crate = m[1].trim();
  if (/^[\w-]+$/.test(crate) && /^[a-z]/.test(crate) && !exists(`crates/${crate}`) && !exists(crate)) {
    add('DANGLING REF', `crate: ${crate} has no crates/${crate} directory`);
  }
}
const knownRoutes = (() => { try { return new Set(JSON.parse(read(S('ralph/generated/routes.json')))); } catch { return null; } })();
if (knownRoutes) {
  for (const m of todo.matchAll(/^\s+routes:\s*(.+)$/gm)) {
    for (const r of m[1].split(/[,\s]+/)) {
      const route = r.trim().replace(/^react\//, 'react/');
      if (route.startsWith('react/') && !knownRoutes.has(route)) add('DANGLING ROUTE', `routes: ${route} is not in ralph/generated/routes.json`);
    }
  }
}

// ---- 4. phantom dependencies (the class that parked the component lane) ---------------------------------
const ids = new Set([...todo.matchAll(/^- \[[ x]\] ([^\n]+?)\s*$/gm)].map((m) => m[1]));
for (const m of todo.matchAll(/^\s+blocked-by:\s*\[([^\]]*)\]/gm)) {
  for (const raw of m[1].split(',')) {
    const dep = raw.trim();
    if (!dep || dep.startsWith('#')) continue;
    if (!ids.has(dep)) add('PHANTOM DEP', `blocked-by "${dep}" resolves to no item id — its item can never unblock`);
  }
}

// ---- 5. double-spawned measurement ---------------------------------------------------------------------
const spawns = new Map();
for (const g of gates) {
  const src = read(S('ralph/scripts', g));
  for (const m of src.matchAll(/node ralph\/scripts\/([a-z-]+\.mjs)/g)) {
    if (m[1] === g) continue;
    if (!spawns.has(m[1])) spawns.set(m[1], []);
    spawns.get(m[1]).push(g);
  }
}
for (const [callee, callersOf] of spawns) {
  const uniq = [...new Set(callersOf)];
  if (uniq.length >= 2) add('DOUBLE-SPAWNED', `${callee} is invoked by ${uniq.length} other gates (${uniq.slice(0, 3).join(', ')}) — one check run can measure it repeatedly, and these are the browser-costly ones`);
}

// ---- 6. bars that cannot be satisfied ------------------------------------------------------------------
// A floor at/below zero passes always; a bar of exactly 0 on a ratio is either free or impossible depending on how it
// is used; a bar above 100 on a percentage can never pass. Only flag the decidable cases, and say why.
const BAR_PATTERNS = [
  { re: /--target\s+(\d+)/g, what: 'copy target' },
  { re: />=?\s*(\d+)\s*%?/g, what: 'inline bar' },
];
for (const f of [regression, prompt, read(S('ralph/scripts/check-page.mjs'))]) {
  for (const { re, what } of BAR_PATTERNS) {
    for (const m of f.matchAll(re)) {
      const v = Number(m[1]);
      if (what === 'copy target' && (v === 0 || v > 100)) add('UNREACHABLE BAR', `${what} ${v} — a percentage outside (0,100] cannot be satisfied by any page`);
      if (what === 'inline bar' && v > 100) add('UNREACHABLE BAR', `${what} >= ${v} — no percentage can exceed 100`);
    }
  }
}

// ---- report ---------------------------------------------------------------------------------------------
const byClass = {};
for (const f of findings) (byClass[f.cls] ??= []).push(f.detail);
console.log(`audit-instruments: ${findings.length} finding(s) across ${Object.keys(byClass).length} class(es)\n`);
const ORDER = ['PHANTOM DEP', 'DANGLING REF', 'DANGLING ROUTE', 'UNREACHABLE BAR', 'DEAD GATE', 'ADVISORY GATE', 'DOUBLE-SPAWNED'];
for (const cls of ORDER) {
  const list = byClass[cls];
  if (!list) continue;
  console.log(`${cls} — ${list.length}`);
  for (const d of list.slice(0, 6)) console.log(`  · ${d}`);
  if (list.length > 6) console.log(`  · …and ${list.length - 6} more`);
  console.log('');
}
if (!findings.length) console.log('nothing found — which would itself be suspicious, given the phantom dependency survived a day of review.');

// With --strict-phantoms, only the phantom-dependency class fails the run — the one class that is clean today and
// cheap to keep clean, because its failure mode is silent and permanent (an item parked forever) while the other
// classes are pre-existing noise. A gate that fails on 133 known findings is a gate people learn to ignore.
if (process.argv.includes('--strict-phantoms')) {
  const phantoms = byClass['PHANTOM DEP'] ?? [];
  if (phantoms.length) {
    console.error(`audit-instruments: ${phantoms.length} phantom dependency/ies — an unsatisfiable blocked-by parks its item forever:`);
    for (const d of phantoms) console.error(`  · ${d}`);
    process.exit(1);
  }
  console.log('audit-instruments: 0 phantom dependencies');
  process.exit(0);
}

// exit 0: this is a report, not a gate. Making it a gate before the existing findings are cleared would block work
// on pre-existing noise, which is how a real signal gets buried.
process.exit(0);
