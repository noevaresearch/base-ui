#!/usr/bin/env node
/* eslint-disable no-console */

// env-health.mjs — is the TEST ENVIRONMENT itself broken? Ask before blaming the port.
//
// THE PROBLEM THIS SOLVES
// -----------------------
// Everything that stalled this loop was the environment lying, and the loop could not tell that apart from its own
// failure. Its own words, from its commits and logs today:
//   * "the CI scorecard's rendered axes are UNMEASURED on every route"  (the loop found MY regression)
//   * "rests on mentions-rendered-evidence.mjs → report REFUSED"        (it wrote a tool the box forbids to run)
//   * an item whose acceptance clause named a rendered run this cgroup cannot perform
//   * a phantom dependency no item could satisfy; 36 clauses naming a fixtures.json that does not exist
// In every case the loop behaved correctly and the INSTRUMENT was wrong. A loop that cannot distinguish those two
// things will grind, retry, or — worse — "fix" the port to satisfy a broken measurement.
//
// So this runs FIRST, cheaply, with no browser and no build, and answers one question per subsystem: is this thing
// working? A subsystem that is BROKEN means: do not interpret any result that depends on it.
//
// Verdicts, and what the loop must do with them:
//   OK        — trust results from this subsystem
//   DEGRADED  — usable, but treat borderline failures as suspect and say so
//   BROKEN    — anything measured through it is UNMEASURED, never a failed page; record the reason and move on
//
// Exit codes: 0 OK · 3 DEGRADED · 4 BROKEN   (distinct from a gate's 1/2 so callers cannot confuse a broken
// environment with a failing check — the confusion that cost this project most of a day).
//
// Usage: node ralph/scripts/env-health.mjs [--json] [--quiet]

import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const ROOT = process.cwd();
const jsonOut = process.argv.includes('--json');
const quiet = process.argv.includes('--quiet');
const read = (p) => { try { return fs.readFileSync(path.isAbsolute(p) ? p : path.join(ROOT, p), 'utf8'); } catch { return ''; } };
const num = (p) => { const v = Number(read(p).trim()); return Number.isFinite(v) ? v : null; };

const checks = [];
const add = (name, status, detail, action) => checks.push({ name, status, detail, ...(action ? { action } : {}) });

// ---- 1. memory: the constraint that killed 51 processes ------------------------------------------------
const cur = num('/sys/fs/cgroup/memory.current');
const max = num('/sys/fs/cgroup/memory.max');
if (cur && max) {
  const pct = Math.round((cur * 100) / max);
  const oom = (read('/sys/fs/cgroup/memory.events').match(/^oom_kill (\d+)/m) || [])[1] ?? '?';
  const status = pct >= 90 ? 'BROKEN' : pct >= 80 ? 'DEGRADED' : 'OK';
  add('memory', status, `${Math.round(cur / 1048576)}MB / ${Math.round(max / 1048576)}MB (${pct}%), oom_kill total ${oom}`,
    status === 'BROKEN' ? 'do not start a browser or a build; the kernel will kill something (probably your tool call)' : null);
} else add('memory', 'UNKNOWN', 'cgroup memory files unreadable');

// ---- 2. can a browser be started at all here? -----------------------------------------------------------
const allowed = process.env.RALPH_BROWSER_GATES === '1';
add('browser', allowed ? 'OK' : 'DEGRADED',
  allowed ? 'browser gates allowed in this environment' : 'browser gates REFUSED here by lib/browser-budget.mjs (4 GB cgroup)',
  allowed ? null : 'any rendered axis must come from CI; read it with scorecard-latest.mjs, do NOT write a local renderer');

// ---- 3. the two reference servers a rendered measurement needs -------------------------------------------
for (const [name, url] of [['port-docs(3177)', 'http://127.0.0.1:3177/'], ['upstream-react(3005)', 'http://127.0.0.1:3005/']]) {
  const r = spawnSync('curl', ['-s', '-o', '/dev/null', '-w', '%{http_code}', '--max-time', '6', url], { encoding: 'utf8' });
  const code = (r.stdout || '').trim();
  add(name, code === '200' ? 'OK' : 'BROKEN', `HTTP ${code || 'no response'}`,
    code === '200' ? null : 'part comparisons are impossible; results that depend on this reference are UNMEASURED');
}

// ---- 4. is the MEASUREMENT trustworthy, or does it read unmeasured everywhere? ---------------------------
const sc = read('ralph/generated/scorecard.jsonl').split('\n').filter(Boolean);
let rows = [];
for (const l of sc) { try { const r = JSON.parse(l); if (r.route && Array.isArray(r.axes)) rows.push(r); } catch { /* counted below */ } }
if (!rows.length) {
  add('scorecard', 'BROKEN', 'no parseable measurement records — nothing may be read from it',
    'a measurement file that cannot be parsed is not "no data": it is a broken writer. Never fall back to a score.');
} else {
  const malformed = sc.length - rows.length;
  const newest = rows.map((r) => r.generatedAt).sort().pop();
  const ageH = newest ? (Date.now() - Date.parse(newest)) / 3600000 : null;
  const rendered = ['example length', 'attribute density', 'snippet language'];
  const allUnmeasured = rows.every((r) => rendered.every((ax) => (r.axes.find((a) => a.axis === ax) || {}).status === 'UNMEASURED'));
  const status = allUnmeasured ? 'BROKEN' : malformed > rows.length * 0.2 ? 'DEGRADED' : 'OK';
  add('scorecard', status === 'OK' && ageH > 6 ? 'DEGRADED' : status,
    `${rows.length} route(s), newest ${newest ?? 'never'} (${ageH === null ? '?' : ageH.toFixed(1)}h old)` +
    `${malformed ? `, ${malformed} malformed line(s)` : ''}${allUnmeasured ? ', EVERY rendered axis reads UNMEASURED on EVERY route' : ''}`,
    allUnmeasured ? 'the rendered half of the measurement is not running (check the CI runner\'s browser allowance and build job) — treat rendered axes as UNMEASURED, never as failed'
      : ageH > 6 ? 'stale: it describes an older commit, not your work' : null);
}

// ---- 5. are the instruments themselves sound? ------------------------------------------------------------
const selftest = path.join(ROOT, 'ralph/scripts/gate-selftest.mjs');
if (fs.existsSync(selftest)) {
  const r = spawnSync('node', [selftest], { cwd: ROOT, encoding: 'utf8', timeout: 120000 });
  const out = `${r.stdout || ''}${r.stderr || ''}`;
  const broken = /checker self-test FAILED/i.test(out);
  add('instruments', broken ? 'BROKEN' : r.status === 0 ? 'OK' : 'DEGRADED',
    broken ? 'the invariant checker fails its own fixtures' : r.status === 0 ? 'self-test green' : 'reports findings (see gate-selftest output)',
    broken ? 'no number produced by these gates may be trusted until the checker passes its own fixtures' : null);
}

// ---- 6. the ledger: can an item be satisfied at all? ----------------------------------------------------
const audit = path.join(ROOT, 'ralph/scripts/audit-instruments.mjs');
if (fs.existsSync(audit)) {
  const r = spawnSync('node', [audit, '--strict-phantoms'], { cwd: ROOT, encoding: 'utf8', timeout: 60000 });
  add('ledger-deps', r.status === 0 ? 'OK' : 'BROKEN',
    r.status === 0 ? 'no phantom dependencies' : 'a blocked-by target resolves to no item id',
    r.status === 0 ? null : 'that item can never unblock: fix the dependency before working it');
}

// ---- verdict --------------------------------------------------------------------------------------------
const broken = checks.filter((c) => c.status === 'BROKEN');
const degraded = checks.filter((c) => c.status === 'DEGRADED');
const verdict = broken.length ? 'ENVIRONMENT BROKEN' : degraded.length ? 'DEGRADED' : 'OK';
fs.mkdirSync(path.join(ROOT, 'ralph/generated'), { recursive: true });
fs.writeFileSync(path.join(ROOT, 'ralph/generated/env-health.json'),
  JSON.stringify({ checkedAt: new Date().toISOString(), verdict, checks }, null, 2));

if (jsonOut) {
  process.stdout.write(JSON.stringify({ verdict, checks }) + '\n');
} else if (!quiet || broken.length) {
  console.log(`environment: ${verdict}`);
  for (const c of checks) console.log(`  ${c.status.padEnd(8)} ${c.name.padEnd(22)} ${c.detail}`);
  if (broken.length) {
    console.log('\nENVIRONMENT BROKEN means: do not conclude anything about the port from the systems above.');
    for (const c of broken) if (c.action) console.log(`  · ${c.name}: ${c.action}`);
  }
}
process.exit(broken.length ? 4 : degraded.length ? 3 : 0);
