#!/usr/bin/env node
/* eslint-disable no-console */

// gate-selftest.mjs — instruments get tested like code.
//
// WHY THIS EXISTS
// ---------------
// Every failure today was in the measuring instrument, not in the port and not in the model's reasoning: the
// harness scored a dead Chrome as a parity failure; a copy bar was enforced on items that never claimed it; a
// language classifier excluded OUR OWN snippets from scoring, which silently zeroed five of the nine page
// axes (leptosSnippets: 5, leptosLines: 0) and made every page mathematically unpassable for hours while the
// loop dutifully tried to satisfy it. An untested instrument is the single biggest tax on this loop: it costs
// the wasted work AND the trust spent re-checking afterwards.
//
// So each gate contract gets a self-test with BOTH directions:
//   * must-flag fixtures  — a report that violates the invariant has to be caught (proves the checker works)
//   * must-pass fixtures  — a clean report has to be accepted (proves it is not just always angry)
// and then the same invariants are evaluated against the REAL artifacts on disk. Fixtures prove the checker;
// the real artifacts are the findings. An invariant that cannot fail is not an invariant.

import fs from 'node:fs';
import path from 'node:path';
import { INVARIANTS, evaluate, scoreWithheldReason } from './lib/metric-invariants.mjs';
import { spawnSync } from 'node:child_process';

const ROOT = process.cwd();
const REPORT_DIR = path.join(ROOT, 'ralph/logs/visual');

// The canonical invariant set lives in lib/metric-invariants.mjs (see that file for the why of each).

// ---- fixtures: prove the checker both ways ---------------------------------------------------------------
const FIXTURES = [
  { id: 'clean-leptos-report', expect: {}, // no invariant may fire
    report: { route: 'fixture/clean', upstreamSnippets: 9, leptosSnippets: 5, score: 82,
              metrics: { upstreamLines: 200, leptosLines: 180, leptosChars: 5000, leptosElements: 40,
                         lengthSimilarity: 0.9, reactToReactBlocks: 0, blocksExcludedFromScoring: 0 } } },
  { id: 'dropped-our-snippets', expect: { 'snippets-scored-not-dropped': true, 'no-react-to-react-scoring': true,
                                           'no-score-from-excluded-extraction': true },
    report: { route: 'fixture/dropped', upstreamSnippets: 9, leptosSnippets: 5, score: 28,
              metrics: { upstreamLines: 28, leptosLines: 0, leptosChars: 0, leptosElements: 0,
                         lengthSimilarity: 0, reactToReactBlocks: 3, blocksExcludedFromScoring: 5 } } },
  { id: 'react-to-react-scored', expect: { 'no-react-to-react-scoring': true },
    report: { route: 'fixture/reactpair', upstreamSnippets: 4, leptosSnippets: 1, score: 90,
              metrics: { upstreamLines: 40, leptosLines: 38, lengthSimilarity: 0.95, reactToReactBlocks: 2, blocksExcludedFromScoring: 0 } } },
  // The other direction of the same rule, and the shape the producer writes after this iteration's fix:
  // React blocks ARE present (2 of 2) and every one is excluded from scoring, so there is no fidelity
  // number to publish and the report withholds it. This must be ACCEPTED — the P0 finding and the FAIL
  // exit belong to the page (ownership: the snippet-translation items), not to the instrument.
  { id: 'react-blocks-excluded-not-scored', expect: {},
    report: { route: 'fixture/reactpair-excluded', upstreamSnippets: 4, leptosSnippets: 2, score: null,
              scoreWithheld: true, scoreWithheldReason: "2 of 2 block(s) are upstream's own React code",
              metrics: { upstreamLines: 40, leptosLines: null, lengthSimilarity: null,
                         reactToReactBlocks: 2, blocksExcludedFromScoring: 2 } } },
  // Every extracted block excluded, yet a number was published — the stale-report defect in one fixture.
  // reactToReactBlocks is deliberately 0 so this isolates the new class: blocks can be excluded without any
  // React-to-React pair (upstream's own stylesheet, mirrored verbatim, is a PERMITTED copy — see the
  // language-gate note in snippet-ergonomics.mjs), and the published number is still arithmetic on nothing.
  { id: 'all-blocks-excluded-yet-scored', expect: { 'no-score-from-excluded-extraction': true },
    report: { route: 'fixture/allext', upstreamSnippets: 14, leptosSnippets: 5, score: 28,
              metrics: { upstreamLines: 96, leptosLines: null, lengthSimilarity: null,
                         reactToReactBlocks: 0, blocksExcludedFromScoring: 5 } } },
  { id: 'ratio-against-empty-side', expect: { 'length-similarity-consistency': true },
    report: { route: 'fixture/emptyside', upstreamSnippets: 3, leptosSnippets: 2, score: 10,
              metrics: { upstreamLines: 50, leptosLines: 20, lengthSimilarity: 0, reactToReactBlocks: 0, blocksExcludedFromScoring: 0 } } },
  { id: 'no-upstream-yet-scored', expect: { 'upstream-side-present': true },
    report: { route: 'fixture/noupstream', upstreamSnippets: 0, leptosSnippets: 4, score: 73,
              metrics: { upstreamLines: 0, leptosLines: 30, lengthSimilarity: 0, reactToReactBlocks: 0, blocksExcludedFromScoring: 0 } } },
  { id: 'empty-both-sides-scored', // THE CI RUN OF 2026-09-16, verbatim: zero blocks on both sides, score 45
    expect: { 'upstream-side-present': true, 'no-blocks-scored': true },
    report: { route: 'fixture/emptyboth', upstreamSnippets: 0, leptosSnippets: 0, score: 45,
              metrics: { upstreamElements: 0, leptosElements: 0, upstreamLines: 0, leptosLines: 0,
                         upstreamMeanAttrs: 0, leptosMeanAttrs: 0, lengthSimilarity: 0, naming: 100,
                         props: 100, brevity: 100, namespaceStyle: 100,
                         snippetLanguages: { total: 0, leptos: 0, react: 0, other: 0 },
                         reactToReactBlocks: 0, blocksExcludedFromScoring: 0 } } },
  { id: 'refusal-scores-nothing', expect: { 'report-not-refused': true, 'no-blocks-scored': true, 'upstream-side-present': true },
    report: { route: 'fixture/refused', refused: true, score: 45, upstreamSnippets: 0, leptosSnippets: 0,
              metrics: {} } },
  { id: 'clean-refusal', // a refusal that claims no measurement is FINE — it is the honest shape
    expect: {},
    report: { route: 'fixture/refused-clean', refused: true, score: null, upstreamSnippets: 0, leptosSnippets: 0,
              reason: 'the extractor found no blocks on either side' } },
];



// ---- 1. the checker tests itself ------------------------------------------------------------------------
// `evaluate()` returns the INVARIANT OBJECTS that fired (see lib/metric-invariants.mjs), not ids. This loop
// treated the result as a Set of ids (`fired.has(w)`) and crashed on its second fixture with
// `TypeError: fired.has is not a function` — MEASURED 2026-09-16 on the CI runner AND on the dev box, so
// the one guard that exists to catch "a report that measured nothing" has never run past fixture one.
let checkerBroken = 0;
console.log('=== self-test of the invariant checker (fixtures)');
for (const fx of FIXTURES) {
  const firedIds = new Set(evaluate(fx.report).map((inv) => inv.id));
  const want = new Set(Object.keys(fx.expect));
  const missed = [...want].filter((w) => !firedIds.has(w));
  const noise = [...firedIds].filter((f) => !want.has(f));
  const ok = missed.length === 0 && noise.length === 0;
  if (!ok) checkerBroken += 1;
  console.log(`  ${ok ? 'ok  ' : 'FAIL'} ${fx.id.padEnd(26)} ${ok ? (want.size ? `flagged ${[...want].join(',')}` : 'accepted clean') : `missed=[${missed}] spurious=[${noise}]`}`);
}
if (checkerBroken) console.log(`  → ${checkerBroken} fixture(s) failed: THE CHECKER ITSELF IS WRONG — fix it before reading any finding below.`);

// ---- 1b. the PRODUCER's half of the same two rules (pure function, no browser) --------------------------
// `snippet-ergonomics.mjs` needs a browser to run at all, so its decision to withhold a number was, until now,
// untestable on this box — while it is the half that actually decides what the loop reads. The rule lives in
// lib/metric-invariants.mjs beside its audit-side twin, and these cases pin both directions: it MUST withhold
// in the two shapes the invariants forbid, and it must NOT withhold on a healthy extraction (otherwise a
// broken gate would hide every page's real score and the loop would chase UNMEASURED forever).
const WITHHOLD_CASES = [
  { id: 'react-blocks-present', inputs: { reactToReactBlocks: 2, extractedBlocks: 2, scorableBlocks: 0 }, withhold: true },
  { id: 'react-block-among-scorable', inputs: { reactToReactBlocks: 1, extractedBlocks: 5, scorableBlocks: 4 }, withhold: true },
  { id: 'nothing-admitted', inputs: { reactToReactBlocks: 0, extractedBlocks: 5, scorableBlocks: 0 }, withhold: true },
  { id: 'healthy-extraction', inputs: { reactToReactBlocks: 0, extractedBlocks: 5, scorableBlocks: 5 }, withhold: false },
  { id: 'partial-exclusion', inputs: { reactToReactBlocks: 0, extractedBlocks: 5, scorableBlocks: 3 }, withhold: false },
  { id: 'no-blocks-extracted', inputs: { reactToReactBlocks: 0, extractedBlocks: 0, scorableBlocks: 0 }, withhold: false },
];
let producerBroken = 0;
console.log('\n=== self-test of the producer\'s withhold rule (snippet-ergonomics.mjs -> lib/metric-invariants.mjs)');
for (const c of WITHHOLD_CASES) {
  const reason = scoreWithheldReason(c.inputs);
  const got = reason !== null;
  const ok = got === c.withhold && (!got || typeof reason === 'string');
  if (!ok) producerBroken += 1;
  console.log(`  ${ok ? 'ok  ' : 'FAIL'} ${c.id.padEnd(26)} ${got ? 'WITHHOLDS' : 'publishes'}` +
    `${ok ? '' : ` (expected ${c.withhold ? 'WITHHOLDS' : 'publishes'})`}`);
}
if (producerBroken) console.log(`  → ${producerBroken} case(s) failed: the producer and the checker disagree — fix lib/metric-invariants.mjs before reading any finding below.`);

// ---- 1c. the naming-parity rule's own unit test (synthetic trees, no browser, no repo state)
// It was written and then never invoked by anything: a checker nobody runs implies coverage that does not exist.
// It belongs here rather than in run-regression because it tests the INSTRUMENT (does a namespaced path count as
// upstream's dotted counterpart, and does an unrelated short name NOT match?) rather than the port.
const namingParity = path.join(ROOT, 'ralph/scripts/check-naming-parity.mjs');
if (fs.existsSync(namingParity)) {
  const r = spawnSync('node', [namingParity], { cwd: ROOT, encoding: 'utf8' });
  const okNP = r.status === 0;
  console.log(`\n=== naming-parity self-test: ${okNP ? 'ok' : 'FAIL'}`);
  if (!okNP) { console.log((r.stdout || '').trim().split('\n').slice(-4).join('\n')); checkerBroken += 1; }
}

// ---- 2. the invariants applied to the real artifacts (these are findings) -------------------------------
console.log('\n=== invariants over real reports (ralph/logs/visual/*-snippets.json)');
const files = fs.existsSync(REPORT_DIR) ? fs.readdirSync(REPORT_DIR).filter((f) => f.endsWith('-snippets.json')) : [];
let violations = 0;
const byInvariant = {};
for (const f of files) {
  let r;
  try { r = JSON.parse(fs.readFileSync(path.join(REPORT_DIR, f), 'utf8')); } catch { continue; }
  const fired = evaluate(r);
  for (const inv of fired) {
    violations += 1;
    (byInvariant[inv.id] ??= []).push({ route: r.route || f, detail: inv.detail ? inv.detail(r) : '' });
  }
}
if (!files.length) console.log('  (no reports on disk)');
for (const [id, entries] of Object.entries(byInvariant)) {
  const inv = INVARIANTS.find((i) => i.id === id);
  console.log(`  VIOLATION ${id} — ${entries.length} report(s): ${entries.slice(0, 5).map((e) => e.route).join(', ')}${entries.length > 5 ? `, +${entries.length - 5}` : ''}`);
  // The DETAIL is the point: a violation line that does not say WHAT was measured cannot be acted on, and
  // on the runner this text is the only evidence the loop ever sees (the sub-gates' output is captured).
  if (entries[0]?.detail) console.log(`            measured: ${entries[0].detail}`);
  if (inv) console.log(`            why it matters: ${inv.why.split('. ')[0]}.`);
}
console.log(`\n  ${files.length} report(s) checked, ${violations} invariant violation(s)${checkerBroken ? ', checker self-test FAILED' : ', checker self-test ok'}${producerBroken ? ', producer withhold-rule self-test FAILED' : ''}`);

// exit 1 when anything is wrong: a broken checker, a producer that disagrees with it, or a broken
// instrument. Either way, no number from these reports may be quoted as a page verdict until this is green.
process.exit(checkerBroken || producerBroken || violations ? 1 : 0);
