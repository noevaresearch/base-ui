#!/usr/bin/env node
/* eslint-disable no-console */

// check-page.mjs — ONE verdict per mirrored page, across every axis.
//
// WHY THIS EXISTS
// ---------------
// The axes were scattered across separate items, and that let a page be "done" while carrying a hollow
// example: the accordion item satisfied `copy >= 95%` and `snippets react=0` with five flattened snippets of
// 12 lines against upstream's 474, length similarity 2.5%, attribute density 0.0 vs 1.2. Each condition was
// true; the page was not finished. Nothing owned the PAGE.
//
// So this runs every axis for a single route and prints one verdict. Rules:
//   * an axis that cannot be measured reports UNMEASURED and NEVER passes (five routes currently have an
//     unmeasurable widget region — "unmeasured" must not read as "fine");
//   * required ergonomics axes are length similarity (>=80%) and attribute density (>=0.8x upstream), because
//     those are what a hollow stub fails;
//   * with --strict, any FAIL or UNMEASURED exits 1.
//
// Progress is then reported as PAGES PASSING THE SCORECARD, not items done.
//
// USAGE
//   node ralph/scripts/check-page.mjs --route react/components/button [--strict] [--json]

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import { evaluate as reportInvariants } from './lib/metric-invariants.mjs';
import path from 'node:path';

const ROOT = path.resolve(import.meta.dirname, '../..');
const OUT = path.join(ROOT, 'ralph/logs/scorecard');
const LEPTOS = process.env.LEPTOS_DOCS_BASE || 'http://127.0.0.1:3177';
const strict = process.argv.includes('--strict');
const jsonOut = process.argv.includes('--json');
// WITH --json, STDOUT CARRIES EXACTLY ONE LINE: THE RECORD. Every human-readable line this script prints
// (the axis table, the instrument warnings) goes to stderr instead, so a caller can safely do
// `node check-page.mjs --route X --json >> scorecard.jsonl`. It could not: the CI aggregate ended up with 1037
// non-record lines in a 17-route file, all of them prose that a redirect had swept into the data file — a writer
// that cannot be redirected safely is how a data file fills with sentences, and how a route count became wrong.
if (jsonOut) console.log = (...args) => console.error(...args);

function arg(name) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] && !process.argv[i + 1].startsWith('--') ? process.argv[i + 1] : null;
}
const route = arg('route');
if (!route) {
  console.error('usage: check-page.mjs --route react/components/<name> [--strict] [--json]');
  process.exit(3);
}
const name = route.split('/').pop();

/** Run an audit script; UNMEASURABLE is its own outcome, never a pass. */
function run(script, args, { timeout = 280000 } = {}) {
  const r = spawnSync('node', [path.join(ROOT, 'ralph/scripts', script), ...args], { cwd: ROOT, encoding: 'utf8', timeout, maxBuffer: 32 * 1024 * 1024 });
  const out = `${r.stdout || ''}${r.stderr || ''}`;
  if (/UNMEASURABLE|unreachable|did not finish/i.test(out) || r.status === 2) return { status: 'UNMEASURED', code: r.status ?? -1, out };
  if (r.status === 0) return { status: 'PASS', code: 0, out };
  return { status: 'FAIL', code: r.status ?? -1, out };
}
const num = (re, text, dflt = null) => { const m = String(text).match(re); return m ? Number(m[1]) : dflt; };

const axes = [];

// 1. structure (mount + headings)
// A page that did not MOUNT is not a structural failure — the loop rebuilds `target/site` while the harness
// serves it, so the wasm can 404 mid-build and the page renders shell-only. Every other script in this
// harness classifies that as UNMEASURABLE and refuses to score it; the first version of this axis passed
// playwright-diff's exit code straight through, so three routes in the second sweep read "structure FAIL"
// while the same command passed when re-run minutes later. An impossible measurement must never be reported
// as a verdict — least of all in the number used to track progress.
{
  const struct = run('playwright-diff.mjs', ['--route', route]);
  const mounted = /"leptosMounted"\s*:\s*true/.test(struct.out);
  const notMounted = /"leptosMounted"\s*:\s*false/.test(struct.out) || /panic|Not found|ECONNREFUSED|timeout/i.test(struct.out);
  axes.push({
    axis: 'structure',
    bar: 'mounts + headings',
    ...struct,
    status: struct.status === 'PASS' ? 'PASS' : (notMounted || !mounted) ? 'UNMEASURED' : struct.status,
    note: struct.status !== 'PASS' && (notMounted || !mounted) ? 'page did not mount (rebuild race or server) — not scored' : undefined,
  });
}

// 2. page + widget parity, snippet language
const budget = run('check-visual-budget.mjs', ['--route', route]);
const score = num(/score\s+([\d.]+)/, budget.out, null);
const widget = num(/widget\s+([\d.]+)%/, budget.out, null);
axes.push({ axis: 'page parity', bar: '>=90', ...budget, value: score, status: score === null ? 'UNMEASURED' : budget.status === 'UNMEASURED' ? 'UNMEASURED' : score >= 90 ? 'PASS' : 'FAIL' });
axes.push({ axis: 'widget parity', bar: '>=97', ...budget, value: widget, status: widget === null ? 'UNMEASURED' : widget >= 97 ? 'PASS' : 'FAIL' });

// 3. snippet ergonomics: the axes that a hollow stub fails
//
// TWO RULES, both learned from the same CI run (35136883087, all 17 routes):
//   * FRESHNESS. The report on disk may predate this run — on the runner it is whatever the checkout
//     carried (this box's last good sweep), and scoring it as CI's measurement is how a local number
//     becomes a CI number. Only a report this run wrote may be scored.
//   * REFUSAL. A gate that could not measure says so (`refused: true`, no score). Its record must never
//     be read as a figure.
const ergoT0 = Date.now();
const ergo = run('snippet-ergonomics.mjs', ['--route', route]);
const reportPath = path.join(ROOT, `ralph/logs/visual/${name}-snippets.json`);
const ergoRefused = ergo.status === 'UNMEASURED';
let m = {};
let staleReport = null;
let upstreamMissing = false;
// INSTRUMENT GUARD: if the report violates a metric invariant, the size-derived axes below are 0 BY
// CONSTRUCTION and must read UNMEASURED — never FAIL. Reporting a broken ruler as a failed page is exactly what
// made these pages mathematically unpassable for hours while the loop kept dutifully trying to satisfy them.
// ANY fired invariant disqualifies the report, not just two of them: `upstream-side-present` fired on every
// route of that run, was printed, and changed nothing — the run still reported `example length FAIL 0`
// (blaming the page for a missing reference) and `snippet language PASS` (a language verdict from zero blocks).
let instrumentBroken = false;
let noBlocks = false;
try {
  const raw = JSON.parse(fs.readFileSync(reportPath, 'utf8'));
  const generatedAt = Date.parse(raw.generatedAt || '');
  if (!Number.isFinite(generatedAt) || generatedAt < ergoT0 - 2000) {
    staleReport = raw.generatedAt || 'no generatedAt';
  } else if (raw.refused === true) {
    staleReport = 'the gate refused to measure this route';
    if (raw.reason) console.log(`  instrument: ${raw.reason}`);
  } else {
    m = raw.metrics || {};
    const violations = reportInvariants(raw);
    if (violations.length) console.log(`  instrument: ${violations.map((v) => v.id).join(', ')} -> size-derived axes forced UNMEASURED (see ralph/scripts/gate-selftest.mjs)`);
    instrumentBroken = violations.length > 0;
    noBlocks = (raw.leptosSnippets ?? 0) === 0;
    upstreamMissing = (raw.upstreamSnippets ?? 0) === 0;
  }
} catch { /* an unreadable report is already UNMEASURED via the null checks below */ }
if (staleReport) console.log(`  instrument: no FRESH snippets report for this route (${staleReport}) — ergonomics axes UNMEASURED, not read from a file this run did not write`);
// A report that violated an invariant cannot carry a SIZE measurement: those axes read null -> UNMEASURED,
// because a size number computed on an empty side, a dropped set or a null ratio is arithmetic, not a page.
// The LANGUAGE axis is deliberately NOT nulled by `instrumentBroken`: a react-to-react block IS its own
// finding (`no-react-to-react-scoring`), the gate raises a P0 for it and the run FAILS — so nulling the
// block count would hide a P0 behind an UNMEASURED. MEASURED, CI run 35141983661: the first cut of this
// guard reported merge-props/use-render (one upstream block each) as UNMEASURED on language where the gate
// itself says FAIL. What DOES make the language axis unmeasurable is a missing side: with no upstream
// reference the ≥90%-verbatim net cannot run at all, so a copy could not be caught and `react = 0` would be
// a green from a comparison that never happened.
if (instrumentBroken) { m.lengthSimilarity = null; m.leptosMeanAttrs = null; m.upstreamMeanAttrs = null; }
const languageUnmeasurable = noBlocks || upstreamMissing;
const reactBlocks = m.reactToReactBlocks ?? null;
const lengthSim = m.lengthSimilarity ?? null;
const attrRatio = m.upstreamMeanAttrs ? Number((m.leptosMeanAttrs / m.upstreamMeanAttrs).toFixed(2)) : null;
// The language axis judges ONLY the language. Inheriting the length floor here (as a first cut did) made a
// page with react=0 report a language FAIL, which is exactly the kind of cross-wired axis that hides what
// is actually wrong — length has its own axis and its own bar. It also does not pass on NOTHING: a `react = 0`
// read from zero extracted blocks is not a language verdict (it was PASS on all 17 routes of the CI run whose
// reports carried `snippetLanguages.total 0`), so a page with no blocks measured is UNMEASURED here.
axes.push({ axis: 'snippet language', bar: 'react = 0', ...ergo, value: reactBlocks,
  status: (reactBlocks === null || languageUnmeasurable) ? 'UNMEASURED' : reactBlocks === 0 ? 'PASS' : 'FAIL',
  note: languageUnmeasurable && reactBlocks === 0 ? 'no comparable code blocks on both sides — a language verdict needs the port\'s blocks AND the upstream reference' : undefined });
axes.push({ axis: 'example length', bar: '>=80% of upstream', ...ergo, value: lengthSim, status: (lengthSim === null || ergoRefused) ? 'UNMEASURED' : lengthSim >= 80 ? 'PASS' : 'FAIL' });
axes.push({ axis: 'attribute density', bar: '>=0.8x upstream', ...ergo, value: attrRatio, status: (attrRatio === null || ergoRefused) ? 'UNMEASURED' : attrRatio >= 0.8 ? 'PASS' : 'FAIL' });

// 4. copy (prose)
axes.push({ axis: 'copy coverage', bar: '>=95%', ...run('check-copy-fidelity.mjs', ['--route', route, '--target', '95']) });

// 5. React mentions on the rendered page
axes.push({ axis: 'react mentions', bar: '0 defects', ...run('check-react-mentions.mjs', ['--route', route]) });

// 6. the port's name (repo-wide, not per route — reported once)
axes.push({ axis: 'package alias', bar: 'resolves, 0 defects', ...run('check-package-alias.mjs', []) });

const fails = axes.filter((a) => a.status === 'FAIL').length;
const unmeasured = axes.filter((a) => a.status === 'UNMEASURED').length;

if (jsonOut) process.stdout.write(JSON.stringify({ route, generatedAt: new Date().toISOString(), axes: axes.map(({ axis, bar, status, value }) => ({ axis, bar, status, value })), fails, unmeasured, verdict: fails || unmeasured ? 'NOT DONE' : 'PASS' }) + '\n');
if (!jsonOut) {
  console.log(`\nPAGE SCORECARD — ${route}`);
  for (const a of axes) {
    const pad = a.axis.padEnd(18);
    const val = a.value === null || a.value === undefined ? '' : ` (${a.value})`;
    console.log(`  ${a.status.padEnd(10)} ${pad} bar ${String(a.bar).padEnd(20)}${val}${a.note ? ` — ${a.note}` : ''}`);
  }
  console.log(`\n  verdict: ${fails || unmeasured ? 'NOT DONE' : 'PASS'} — ${fails} failing axis/axes, ${unmeasured} unmeasured (an unmeasured axis is never a pass)`);
}
// WHY THE REASONS PRINT EVEN WITH --json (they go to stderr; see the jsonOut redirect at the top): on the
// runner this is the ONLY record of what each gate saw — check-page captures a sub-gate's output instead of
// streaming it, so the CI log of run 35136883087 held nothing but "── <route>" and "report: <file>" for a
// run in which two axes were FAIL and four were UNMEASURED. A red axis whose reason is invisible is an axis
// nobody can act on; the loop cannot even tell a missing reference from a missing port.
{
  const REASON = {
    structure: /FAIL|panic|not found|timeout/i,
    'page parity': /score\s+([\d.]+)/i,
    'widget parity': /widget\s+([\d.]+)%/i,
    'snippet language': /LANGUAGE|react-to-react|are upstream's React code/i,
    'example length': /length check|length similarity/i,
    'attribute density': /attribute density|meanAttrs|attributes per element/i,
    'copy coverage': /copy fidelity|UNMEASURABLE/i,
    'react mentions': /^FAIL|defect|UNMEASURABLE/i,
    'package alias': /FAIL|defect\(s\)|SETUP/i,
  };
  for (const a of axes.filter((x) => x.status !== 'PASS')) {
    const reason = (a.out || '').split('\n').find((l) => (REASON[a.axis] || /FAIL|UNMEASURABLE/i).test(l)) || a.note || null;
    if (reason) console.log(`    ${a.status} ${a.axis}: ${reason.trim().slice(0, 220)}`);
  }
}

fs.mkdirSync(OUT, { recursive: true });
fs.writeFileSync(path.join(OUT, `${name}.md`), [
  `# Page scorecard — ${route}`, '', `Generated ${new Date().toISOString()} by check-page.mjs.`,
  '', ...axes.map((a) => `- **${a.status}** — ${a.axis} (bar ${a.bar})${a.value !== null && a.value !== undefined ? ` value ${a.value}` : ''}`),
  '', `Verdict: **${fails || unmeasured ? 'NOT DONE' : 'PASS'}** (${fails} failing, ${unmeasured} unmeasured)`, '',
].join('\n'));
console.log(`report: ralph/logs/scorecard/${name}.md`);

process.exit(strict && (fails || unmeasured) ? 1 : 0);
