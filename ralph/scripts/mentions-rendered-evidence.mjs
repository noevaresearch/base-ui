#!/usr/bin/env node
/* eslint-disable no-console */

// mentions-rendered-evidence.mjs — can this tree's RENDERED React-mentions verdict be read WITHOUT a browser?
//
// WHY THIS EXISTS
// ---------------
// CONTRACT.md requirement 6 has two halves: the port's own page SOURCE must not name upstream's package or a React
// API (`check-react-mentions.mjs --source`, pure text, runs anywhere), and no RENDERED route may either (`--all`,
// which drives a browser). The rendered half is the one that caught the live defect — pages shipping upstream's JSX
// while every structural gate stayed green — so it is not decoration, and `run-regression.sh` deliberately gates
// BOTH halves for the `docs-copy:` lane.
//
// On 2026-09-16 the box's browser budget (`ralph/scripts/lib/browser-budget.mjs`) started refusing every rendered
// gate with exit 2 = UNMEASURED, and the runner reported that REFUSAL as a defect: the `docs-copy:` lane became
// locally un-closeable — not because a page leaked React, but because the ruler could not be picked up. The budget
// module's own contract says callers map exit 2 to UNMEASURED rather than FAIL. And an UNMEASURED axis must never
// PASS either, so "the runner cannot run it here" is not an answer on its own. This script is the third option: it
// reads the rendered measurement that IS on record — `ralph/logs/visual/react-mentions.md`, written by a real
// Chrome-for-Testing run, and the CI scorecard's per-route `react mentions` axis — and decides whether that
// measurement still SPEAKS FOR THIS TREE. It starts no browser (pure text + git), so the loop can afford it on
// every iteration.
//
// WHAT MAKES A RECORDED MEASUREMENT ADMISSIBLE — all four, or it is not evidence:
//   1. it EXISTS and is COMMITTED AT HEAD (what is committed is real state; a working-tree report cannot be tied
//      to a tree, and `--route` runs rewrite the same file);
//   2. it is CLEAN for the classes being claimed (see `--fail-on`; the same per-item ownership split the live run
//      uses, so a snippet-language defect owned by another item cannot be charged to this one — and cannot be
//      excused by it either);
//   3. it is SITE-WIDE, not a single-route scan (a `--route` run writes the same report path, so a 1-route report
//      would otherwise stand in for every route);
//   4. it is FRESH — no file that can change its verdict has changed since the commit that recorded it
//      (VERDICT_PATHS: the page sources, the allow files, the alias package, and the classifier itself).
// A stale or absent record is NOT a pass: it means re-measure — `RALPH_BROWSER_GATES=1 node
// ralph/scripts/check-react-mentions.mjs --all` here, or CI (`.github/workflows/measure-port.yml`).
//
// Usage: node ralph/scripts/mentions-rendered-evidence.mjs [--fail-on cls,cls] [--json]
// Exit 0 = an admissible rendered measurement is on record; 1 = it is not (reason printed); 3 = the self-test
// below failed, i.e. the instrument is wrong and its verdict must not be trusted either way.

import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const ROOT = path.resolve(import.meta.dirname, '../..');
const REPORT = 'ralph/logs/visual/react-mentions.md';
const SCORECARD = 'ralph/generated/scorecard.jsonl';
const GATED = ['react-api', 'package-react', 'snippet-react'];

// The files that can change a rendered React-mentions verdict. Kept SHORT on purpose: a path list that grows
// without a reason turns "fresh" into "never fresh" and the gate into noise. crates/leptos-ui is deliberately
// not here — a component port changes what a page RENDERS, not the prose the check scans, and adding it would
// invalidate the record on almost every iteration. `check-react-mentions.mjs` is not here either, for the
// opposite reason: it changes often (it is the file the loop keeps fixing), so it is tracked by the RULER
// fingerprint it stamps into the report instead of by its history — a moved `if` invalidates nothing, a changed
// rule invalidates the record. `lib/source-scope.mjs` holds the scope rules the classifier's verdicts depend on
// and is NOT covered by that fingerprint, so it stays here.
export const VERDICT_PATHS = [
  'crates/docs-app/src', // the page sources, install_ref.rs, snippet_language.rs
  'specs/docs-content', // the mirrored page specs and their react-allow.json records
  'packages/leptos', // the alias package a page tells the reader to install
  'ralph/scripts/lib/source-scope.mjs', // the shared page-copy/snippet-language scope the classifier reads
];

/** Parse `check-react-mentions.mjs`'s rendered report. Returns null when it is not that report. */
export function parseReport(md) {
  const text = String(md ?? '');
  const at = text.match(/^Generated (\S+) by check-react-mentions\.mjs(?: — ruler fingerprint (\w+))?\.$/m);
  const totals = text.match(/\*\*Totals: (\d+) defect\(s\) — (\d+) gated[^*]*across (\d+) route\(s\)\.\*\*/);
  if (!at || !totals) return null;
  // Per-class counts come from the defect bullets, not from the totals line: the totals line names the classes
  // statically, so it cannot tell a `snippet-react` defect from a `package-react` one — and that distinction is
  // the ledger's own ownership split.
  const perClass = {};
  for (const line of text.split('\n')) {
    const m = line.match(/^(?:FAIL|RE-HOMED) \*\*([a-z][a-z-]*)\*\* — /);
    if (m) perClass[m[1]] = (perClass[m[1]] ?? 0) + 1;
  }
  return { generatedAt: at[1], fingerprint: at[2] ?? null, defects: Number(totals[1]), gated: Number(totals[2]), routes: Number(totals[3]), perClass };
}

/**
 * The verdict for a recorded report. Pure: every fact it needs is passed in, so the fixtures below can pin it.
 * `claim` is the class list the calling item owns; a defect in a class nobody claimed is reported, never charged.
 */
export function judgeReport(facts, claim = GATED) {
  if (!facts.parsed) return { ok: false, reason: 'no rendered report on record (ralph/logs/visual/react-mentions.md is absent or is not that report)' };
  if (!facts.atHead) return { ok: false, reason: 'the rendered report differs from HEAD (an uncommitted report cannot be tied to a tree — commit it or re-measure)' };
  if (facts.routes < 2) return { ok: false, reason: `the record covers ${facts.routes} route(s): that is a single-route scan, not the site-wide measurement this clause needs` };
  if (!facts.parsed.fingerprint) return { ok: false, reason: 'the record carries no ruler fingerprint (it predates the fingerprint instrumentation), so it cannot be shown to have been measured by the classifier running today — re-measure' };
  if (facts.ruler && facts.parsed.fingerprint !== facts.ruler) return { ok: false, reason: `the RULER changed since the record was made (fingerprint ${facts.parsed.fingerprint} -> ${facts.ruler}): its verdicts belong to the older classifier — re-measure` };
  const claimed = (claim ?? GATED).filter((c) => GATED.includes(c));
  if (claim && claim.length && !claimed.length) return { ok: false, reason: `--fail-on names no class this check knows (${claim.join(', ')}); a claim that matches nothing must not read as a clean record` };
  const charged = Object.entries(facts.parsed.perClass).filter(([cls, n]) => numbered(n) && claimed.includes(cls));
  const uncharged = Object.entries(facts.parsed.perClass).filter(([cls, n]) => numbered(n) && !claimed.includes(cls));
  if (charged.length) return { ok: false, reason: `the record is NOT clean for the class(es) this item claims: ${charged.map(([c, n]) => `${c} x${n}`).join(', ')}` };
  const attributable = Object.values(facts.parsed.perClass).reduce((n, v) => n + (numbered(v) ? Number(v) : 0), 0);
  if (attributable < facts.parsed.defects) return { ok: false, reason: `the record reports ${facts.parsed.defects} defect(s) but only ${attributable} carry a class this script can attribute — an unaccountable defect is treated as this item's` };
  if (facts.changed.length) return { ok: false, reason: `the record is STALE: ${facts.changed.length} verdict-changing path(s) changed since it was recorded (${facts.since.slice(0, 8)}) — ${facts.changed.slice(0, 4).join(', ')}${facts.changed.length > 4 ? ', …' : ''}` };
  const note = uncharged.length ? ` (${uncharged.map(([c, n]) => `${c} x${n}`).join(', ')} belong to another item, reported not charged)` : '';
  return { ok: true, reason: `clean for ${claimed.join(', ')}: 0 defect(s) across ${facts.routes} route(s), recorded ${facts.parsed.generatedAt} by ruler ${facts.parsed.fingerprint}, no verdict-changing path changed since${note}` };
}

const numbered = (n) => Number.isFinite(Number(n)) && Number(n) > 0;

/** The CI scorecard's per-route `react mentions` axis — a second recorded measurement, same four conditions. */
export function judgeScorecard(facts, claim = GATED) {
  if (!facts.rows.length) return { ok: false, reason: 'the CI scorecard records no route (run .github/workflows/measure-port.yml, or read ralph/generated/scorecard.jsonl)' };
  if (!facts.atHead) return { ok: false, reason: 'the CI scorecard differs from HEAD (uncommitted measurements cannot be tied to a tree)' };
  const bad = facts.rows.filter((r) => r.status !== 'PASS');
  if (bad.length) {
    const sample = bad.slice(0, 3).map((r) => `${r.route}: ${r.status}`).join(', ');
    return { ok: false, reason: `the CI scorecard's rendered mentions axis is not clean on ${bad.length} of ${facts.rows.length} route(s) — ${sample}${bad.length > 3 ? ', …' : ''}` };
  }
  if (facts.changed.length) return { ok: false, reason: `the CI scorecard is STALE: ${facts.changed.slice(0, 4).join(', ')} changed since it was recorded (${facts.since.slice(0, 8)})` };
  return { ok: true, reason: `CI measured zero React/package defects on all ${facts.rows.length} recorded route(s) (scorecard recorded ${facts.since.slice(0, 8)}), no verdict-changing path changed since` };
}

// ---- fixtures: prove the judge both ways -------------------------------------------------------------------
// An instrument whose excuses are untested is how a gate silently swallows a real defect. Each fixture is a fact
// set plus the claim, and expectations are pinned in both directions: a clean record must be ADMITTED, every
// broken one must be REFUSED with the right reason.
const CLEAN_REPORT = { generatedAt: '2026-09-16T18:50:39.284Z', fingerprint: 'abc123def456', defects: 0, gated: 0, routes: 41, perClass: {} };
const RULER = { ruler: 'abc123def456', since: 'aaaa1111', changed: [] };
const FIXTURES = [
  { id: 'clean-site-wide-fresh', claim: GATED, facts: { parsed: CLEAN_REPORT, atHead: true, routes: 41, ...RULER }, expect: true, wantReason: /clean for react-api, package-react, snippet-react/ },
  { id: 'defect-in-a-claimed-class', claim: GATED, facts: { parsed: { ...CLEAN_REPORT, defects: 1, gated: 1, perClass: { 'react-api': 1 } }, atHead: true, routes: 41, ...RULER }, expect: false, wantReason: /react-api x1/ },
  { id: 'defect-in-an-unclaimed-class', claim: ['react-api', 'package-react'], facts: { parsed: { ...CLEAN_REPORT, defects: 2, gated: 2, perClass: { 'snippet-react': 2 } }, atHead: true, routes: 41, ...RULER }, expect: true, wantReason: /snippet-react x2 belong to another item/ },
  { id: 'same-defect-when-claimed', claim: GATED, facts: { parsed: { ...CLEAN_REPORT, defects: 2, gated: 2, perClass: { 'snippet-react': 2 } }, atHead: true, routes: 41, ...RULER }, expect: false, wantReason: /snippet-react x2/ },
  { id: 'unaccountable-defect-count', claim: ['react-api'], facts: { parsed: { ...CLEAN_REPORT, defects: 3, gated: 3, perClass: { 'snippet-react': 1 } }, atHead: true, routes: 41, ...RULER }, expect: false, wantReason: /only 1 carry a class/ },
  { id: 'single-route-scan-is-not-the-site', claim: GATED, facts: { parsed: { ...CLEAN_REPORT, routes: 1 }, atHead: true, routes: 1, ...RULER }, expect: false, wantReason: /covers 1 route\(s\)/ },
  { id: 'uncommitted-report-is-not-evidence', claim: GATED, facts: { parsed: CLEAN_REPORT, atHead: false, routes: 41, ...RULER }, expect: false, wantReason: /differs from HEAD/ },
  { id: 'record-without-a-fingerprint', claim: GATED, facts: { parsed: { ...CLEAN_REPORT, fingerprint: null }, atHead: true, routes: 41, ...RULER }, expect: false, wantReason: /no ruler fingerprint/ },
  { id: 'ruler-changed-since-the-record', claim: GATED, facts: { parsed: CLEAN_REPORT, atHead: true, routes: 41, changed: [], since: 'aaaa1111', ruler: 'ffff99999999' }, expect: false, wantReason: /RULER changed/ },
  { id: 'stale-after-a-page-change', claim: GATED, facts: { parsed: CLEAN_REPORT, atHead: true, routes: 41, changed: ['crates/docs-app/src/pages/checkbox_page.rs'], since: 'bbbb2222', ruler: 'abc123def456' }, expect: false, wantReason: /STALE/ },
  { id: 'unparseable-report', claim: GATED, facts: { parsed: null, atHead: true, routes: 0, ...RULER }, expect: false, wantReason: /no rendered report on record/ },
  { id: 'claim-that-names-no-known-class', claim: ['not-a-class'], facts: { parsed: CLEAN_REPORT, atHead: true, routes: 41, ...RULER }, expect: false, wantReason: /names no class this check knows/ },
];
const SCORECARD_FIXTURES = [
  { id: 'ci-all-routes-pass', claim: GATED, facts: { rows: [{ route: 'react/components/checkbox', status: 'PASS' }, { route: 'react/components/button', status: 'PASS' }], atHead: true, changed: [], since: 'cccc3333' }, expect: true },
  { id: 'ci-unmeasured-route', claim: GATED, facts: { rows: [{ route: 'react/components/accordion', status: 'PASS' }, { route: 'react/components/avatar', status: 'UNMEASURED' }], atHead: true, changed: [], since: 'cccc3333' }, expect: false, wantReason: /avatar: UNMEASURED/ },
  { id: 'ci-fail-route', claim: GATED, facts: { rows: [{ route: 'react/components/button', status: 'FAIL' }], atHead: true, changed: [], since: 'cccc3333' }, expect: false, wantReason: /button: FAIL/ },
  { id: 'ci-empty', claim: GATED, facts: { rows: [], atHead: true, changed: [], since: 'cccc3333' }, expect: false, wantReason: /records no route/ },
];

function selfTest() {
  const failures = [];
  for (const f of FIXTURES) {
    const got = judgeReport(f.facts, f.claim);
    if (got.ok !== f.expect) failures.push(`${f.id}: expected admissible=${f.expect}, got ${got.ok} (${got.reason})`);
    else if (f.wantReason && !f.wantReason.test(got.reason)) failures.push(`${f.id}: reason did not name the defect — ${got.reason}`);
  }
  for (const f of SCORECARD_FIXTURES) {
    const got = judgeScorecard(f.facts, f.claim);
    if (got.ok !== f.expect) failures.push(`${f.id}: expected admissible=${f.expect}, got ${got.ok} (${got.reason})`);
    else if (f.wantReason && !f.wantReason.test(got.reason)) failures.push(`${f.id}: reason did not name the defect — ${got.reason}`);
  }
  // The parser is part of the instrument too: the real report's shape must parse, and a foreign file must not.
  const real = parseReport(fs.existsSync(path.join(ROOT, REPORT)) ? fs.readFileSync(path.join(ROOT, REPORT), 'utf8') : '');
  if (real === null && fs.existsSync(path.join(ROOT, REPORT))) failures.push('parseReport: the report on disk did not parse (shape changed?)');
  const foreign = parseReport('# something else\n\nhello\n');
  if (foreign !== null) failures.push('parseReport: accepted a file that is not the rendered mentions report');
  return failures;
}

// ---- the real artifacts ------------------------------------------------------------------------------------
function git(args) {
  try { return execFileSync('git', args, { cwd: ROOT, encoding: 'utf8' }).trim(); } catch { return null; }
}
function atHead(rel) {
  try {
    execFileSync('git', ['ls-files', '--error-unmatch', rel], { cwd: ROOT, stdio: 'ignore' });
    execFileSync('git', ['diff', '--quiet', 'HEAD', '--', rel], { cwd: ROOT, stdio: 'ignore' });
    return true;
  } catch { return false; }
}
function factsFor(rel, paths = VERDICT_PATHS) {
  const since = git(['log', '-1', '--format=%H', '--', rel]) ?? '';
  const changed = since ? (git(['diff', '--name-only', `${since}..HEAD`, '--', ...paths]) ?? '').split('\n').filter(Boolean) : [];
  return { atHead: atHead(rel), changed, since };
}
/** Today's ruler fingerprint, asked of the classifier itself (pure text, no browser, no side effects). */
function currentRuler() {
  try {
    return execFileSync('node', [path.join(ROOT, 'ralph/scripts/check-react-mentions.mjs'), '--ruler-fingerprint'], { cwd: ROOT, encoding: 'utf8' }).trim() || null;
  } catch { return null; }
}
function scorecardRows() {
  try {
    return fs.readFileSync(path.join(ROOT, SCORECARD), 'utf8').split('\n').filter((l) => l.trim()).flatMap((l) => {
      try {
        const row = JSON.parse(l);
        const axis = (row.axes ?? []).find((a) => a.axis === 'react mentions');
        return axis ? [{ route: row.route, status: axis.status }] : [];
      } catch { return []; }
    });
  } catch { return []; }
}

const arg = (name) => {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] && !process.argv[i + 1].startsWith('--') ? process.argv[i + 1] : null;
};
const jsonOut = process.argv.includes('--json');
const claimArg = arg('fail-on');
const claim = claimArg ? claimArg.split(',').map((s) => s.trim()).filter(Boolean) : GATED;

const failures = selfTest();
if (failures.length) {
  console.error('mentions-rendered-evidence: SELF-TEST FAILED — the instrument is wrong, do not trust either verdict:');
  for (const f of failures) console.error(`  ${f}`);
  process.exit(3);
}

const ruler = currentRuler();
const reportFacts = { ...factsFor(REPORT), routes: 0, parsed: null, ruler };
if (fs.existsSync(path.join(ROOT, REPORT))) {
  reportFacts.parsed = parseReport(fs.readFileSync(path.join(ROOT, REPORT), 'utf8'));
  reportFacts.routes = reportFacts.parsed?.routes ?? 0;
}
const report = judgeReport(reportFacts, claim);
// The CI scorecard's rows carry no fingerprint of their own, so for THIS source the ruler file counts as a
// verdict-changing path in its own right: a scorecard written before the classifier changed cannot speak for the
// classifier running today. Stricter than the report path on purpose — it has the weaker record to start from.
const ciPaths = [...VERDICT_PATHS, 'ralph/scripts/check-react-mentions.mjs'];
const cardFacts = factsFor(SCORECARD, ciPaths);
const ci = judgeScorecard({ ...cardFacts, rows: scorecardRows() }, claim);

const result = { claim, ruler, report, ci, admissible: report.ok || ci.ok };

if (jsonOut) {
  process.stdout.write(JSON.stringify(result) + '\n');
} else {
  console.log(`rendered React-mentions evidence — claiming clean: ${claim.join(', ')}`);
  console.log(`  recorded rendered report: ${report.ok ? 'ADMITTED' : 'REFUSED'} — ${report.reason}`);
  console.log(`  CI scorecard's axis:      ${ci.ok ? 'ADMITTED' : 'REFUSED'} — ${ci.reason}`);
  console.log(`  (starts at ${REPORT}; a browser run ${'`RALPH_BROWSER_GATES=1 node ralph/scripts/check-react-mentions.mjs --all`'} rewrites it, and it must be committed to count)`);
  console.log(`  verdict: ${result.admissible ? 'MEASURED (clean)' : 'UNMEASURED — this is NOT a pass'}`);
}
process.exit(result.admissible ? 0 : 1);
