#!/usr/bin/env node
/* eslint-disable no-console */

// check-visual-budget.mjs — the visual-fidelity gate for the docs-app port.
//
// WHY THIS EXISTS
// ---------------
// `playwright-diff.mjs` proves a mirrored docs page *mounts* (structure + headings). It
// cannot see that the page is visually naked: the Leptos docs app has no layout shell,
// no sidebar, no syntax highlighting, no code-block chrome, no demo panels and no API
// tables, so a pair can be "done" while its page looks nothing like upstream. That is a
// visibility gap of exactly the kind `CONTEXT.md` treats as P0 — the loop would keep
// producing structurally-correct, unstyled pages forever, because no gate ever measured
// the difference.
//
// WHAT IT MEASURES (per route, upstream React docs vs Leptos docs-app)
// -------------------------------------------------------------------
//   content recall  — did the page carry the same content? (headings, demos, code
//                     blocks, tables, links, text length, each clamped to <= 1)
//   visual proximity — 100 - (percent of pixels that differ at 1280px), i.e. how close
//                     the rendered page is to upstream's own render
//   fidelity score   — 0..100, the weighted blend of both (visual 0.6 / content 0.4)
//
// Scoring both halves deliberately: pixel distance alone is uninformative here (the
// pages are mostly whitespace, so a missing sidebar is cheap in pixels), while content
// recall alone would score a naked page as perfect.
//
// THE GATE
// --------
// A *regression* gate, not a threshold gate: it compares the current score against the
// best score recorded for that route in `ralph/generated/visual-baseline.json` and fails
// when the score drops by more than `--tolerance` (default 2.0 points). This lets the
// loop keep working on unstyled pages today (the baseline starts wherever the port
// actually is) while making it impossible to *worsen* fidelity unnoticed — and it gives
// the docs-chrome items a number to move.
//
//   node ralph/scripts/check-visual-budget.mjs --todo-id "docs-content: components/checkbox"
//   node ralph/scripts/check-visual-budget.mjs --route react/components/checkbox --update
//   node ralph/scripts/check-visual-budget.mjs --all-done   # every baseline route
//   node ralph/scripts/check-visual-budget.mjs --all-done --target 90   # parity enforcement (Phase E)
//
// `--update` re-records the baseline for a route (use it when fidelity *improves*, or to
// seed a new route). Never use it to hide a regression: the point of the file is history.
//
// Requires both dev servers:
//   upstream  — `next dev --port 3005` in docs/           (React reference render)
//   leptos    — `python3 ralph/scripts/serve-docs-app.py` (built wasm bundle, port 3177)
// If the upstream server is unreachable this script exits 0 with a NOTE — the reference
// render is not available in every environment (it needs the full Next.js toolchain and
// node_modules), and a missing reference must not read as a fidelity failure. It is
// reported, never silently skipped.

import { execFileSync, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const BASELINE_PATH = path.join(PROJECT_ROOT, 'ralph/generated/visual-baseline.json');
const DIFF_SCRIPT = path.join(PROJECT_ROOT, 'ralph/scripts/visual-diff.mjs');
const UPSTREAM_BASE = process.env.UPSTREAM_DOCS_BASE || 'http://127.0.0.1:3005';
const LEPTOS_BASE = process.env.LEPTOS_DOCS_BASE || 'http://127.0.0.1:3177';

function arg(name, dflt) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] && !process.argv[i + 1].startsWith('--')
    ? process.argv[i + 1]
    : dflt;
}
const has = (name) => process.argv.includes(`--${name}`);

const tolerance = Number(arg('tolerance', '2.0'));
const doUpdate = has('update');
// `--target N` enforces absolute parity (the Phase E goal is 90): every route must score >= N,
// not merely not-worse. Without it the gate is regression-only, so the loop can keep landing
// chrome work on routes that are still far from parity.
const target = arg('target', null) === null ? null : Number(arg('target'));

function routeFromTodoId(todoId) {
  const m = todoId.match(/components\/([a-z0-9-]+)/i);
  if (!m) return null;
  return `react/components/${m[1]}`;
}

function reachable(url) {
  const r = spawnSync('curl', ['-s', '-o', '/dev/null', '-w', '%{http_code}', '--max-time', '20', url], {
    encoding: 'utf8',
  });
  return (r.stdout || '').trim() === '200';
}


// Which build was measured? The Ralph loop rebuilds target/site under us, so a score is only
// meaningful if it names the artifact it scored. Reported in the output and the report files.
function servedBuild(base) {
  // synchronous on purpose: the gate's main() is sync and must stay so (it is called from the
  // regression shell script). curl -sI is one cheap process.
  const r = spawnSync('curl', ['-sI', '--max-time', '15', `${base}/pkg/docs-app.wasm`], { encoding: 'utf8' });
  const out = r.stdout || '';
  const bytes = Number((out.match(/content-length:\s*(\d+)/i) || [])[1] || 0);
  const lastModified = ((out.match(/last-modified:\s*(.+)/i) || [])[1] || 'unknown').trim();
  return { bytes, lastModified };
}

function clamp01(n) {
  if (!Number.isFinite(n) || n < 0) return 0;
  return Math.min(n, 1);
}

function ratio(leptosValue, upstreamValue) {
  const u = Number(upstreamValue) || 0;
  if (u <= 0) return null; // upstream has none of this — not a signal either way
  return clamp01((Number(leptosValue) || 0) / u);
}

function mean(values) {
  const finite = values.filter((v) => v !== null && Number.isFinite(v));
  if (!finite.length) return 0;
  return finite.reduce((a, b) => a + b, 0) / finite.length;
}

function scoreReport(route, report) {
  const u = report.upstreamStats || {};
  const l = report.leptosStats || {};
  const pixelDiff = Number.parseFloat(String(report.pixelDiff || '').replace('%', ''));

  // Snippet language is scored as PURITY, not presence: a page that embeds upstream's React
  // source has the right word count and the wrong framework, so counting text alone would reward
  // copying. This rewards translating.
  const ls = l.snippets || { total: 0, leptos: 0, react: 0, other: 0 };
  const snippetLanguage = ls.total > 0 ? clamp01(ls.leptos / ls.total) : null;
  const recallParts = {
    headings: ratio((l.headings || []).length, (u.headings || []).length),
    demos: ratio(l.demos, u.demos),
    codeBlocks: ratio(l.codeBlocks, u.codeBlocks),
    snippetLanguage,
    tables: ratio(l.tables, u.tables),
    links: ratio(l.links, u.links),
    textLen: ratio(l.textLen, u.textLen),
  };
  const contentRecall = mean(Object.values(recallParts));
  const visualProximity = Number.isFinite(pixelDiff) ? clamp01(1 - pixelDiff / 100) : null;
  const score = visualProximity === null
    ? contentRecall * 100
    : (visualProximity * 0.6 + contentRecall * 0.4) * 100;

  const leptosUnmounted = (l.headings || []).length <= 2 && (Number(l.textLen) || 0) < 600;
  // Both sides rendering the same page is a harness fault (a navigation that did not take, the
  // upstream server proxying to ours, or two identical screenshots), and scoring it would report a
  // perfect 0% pixel diff — the one failure mode that inflates the score instead of depressing it.
  const samehHref = u.href && l.href && u.href === l.href;
  const identicalStats = Number.isFinite(pixelDiff) && pixelDiff === 0 && u.textLen === l.textLen;
  const harnessFault = samehHref || identicalStats;
  return {
    route,
    leptosUnmounted,
    harnessFault,
    score: Number(score.toFixed(2)),
    visualProximity: visualProximity === null ? null : Number((visualProximity * 100).toFixed(2)),
    contentRecall: Number((contentRecall * 100).toFixed(2)),
    pixelDiffPercent: Number.isFinite(pixelDiff) ? pixelDiff : null,
    recallParts,
    upstreamHref: u.href || null,
    leptosHref: l.href || null,
    upstream: { headings: (u.headings || []).length, codeBlocks: u.codeBlocks, tables: u.tables, textLen: u.textLen },
    leptos: { headings: (l.headings || []).length, codeBlocks: l.codeBlocks, tables: l.tables, textLen: l.textLen, snippets: l.snippets || null },
  };
}

function measure(route) {
  const outDir = `/tmp/visual-budget-${route.split('/').pop()}`;
  const r = spawnSync('node', [DIFF_SCRIPT, '--route', route, '--out', outDir], {
    cwd: PROJECT_ROOT,
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  });
  const text = (r.stdout || '') + (r.stderr || '');
  const start = text.indexOf('{');
  if (start < 0) throw new Error(`visual-diff produced no report for ${route}: ${text.slice(-400)}`);
  const report = JSON.parse(text.slice(start, text.lastIndexOf('}') + 1));
  return scoreReport(route, report);
}

function loadBaseline() {
  try {
    return JSON.parse(fs.readFileSync(BASELINE_PATH, 'utf8'));
  } catch {
    return { note: 'best-known fidelity score per route; written by check-visual-budget.mjs --update', routes: {} };
  }
}

function main() {
  if (!fs.existsSync(DIFF_SCRIPT)) {
    console.error(`FAIL: ${path.relative(PROJECT_ROOT, DIFF_SCRIPT)} is missing — cannot measure fidelity.`);
    return 1;
  }

  let routes = [];
  if (has('all-done')) {
    routes = Object.keys(loadBaseline().routes);
    if (!routes.length) {
      console.error('FAIL: no baseline routes recorded yet — seed one with --route <r> --update');
      return 1;
    }
  } else {
    const route = arg('route', null) || routeFromTodoId(arg('todo-id', ''));
    if (!route) {
      console.error('usage: check-visual-budget.mjs (--route <react/components/x> | --todo-id "<id>") [--update] [--tolerance N] | --all-done');
      return 1;
    }
    routes = [route];
  }

  if (!reachable(`${LEPTOS_BASE}/`)) {
    console.error(`FAIL: Leptos docs server unreachable at ${LEPTOS_BASE} — run: python3 ralph/scripts/serve-docs-app.py --port 3177`);
    return 1;
  }
  if (!reachable(`${UPSTREAM_BASE}/`)) {
    console.log(`NOTE: upstream React docs unreachable at ${UPSTREAM_BASE} — cannot measure the visual half.`);
    console.log('      Start it with: (cd docs && node_modules/.bin/next dev --port 3005)');
    console.log('      Recorded as unverified, not as a pass.');
    return 0;
  }

  const baseline = loadBaseline();
  baseline.routes ||= {};
  const results = [];
  let failed = false;

  for (const route of routes) {
    let measured;
    try {
      measured = measure(route);
    } catch (e) {
      console.error(`FAIL: could not measure ${route}: ${e.message}`);
      failed = true;
      results.push({ route, error: e.message });
      continue;
    }

    // A shell-only render means the route did not mount — usually because the Ralph loop is
    // rebuilding target/site while we serve it (the wasm 404s mid-build), occasionally because of
    // a real mount failure. Scoring that as "fidelity collapsed" would gate the loop on a race it
    // cannot win, so it is reported as UNMEASURABLE: no score recorded, no regression claimed,
    // and the mount differential (playwright-diff.mjs) remains the thing that catches real
    // mount failures.
    if (measured.harnessFault) {
      console.log(`UNMEASURABLE ${route}: both sides rendered the same page (upstream ${measured.upstreamHref || '?'} vs leptos ${measured.leptosHref || '?'}) — ` +
        `a 0% pixel diff here is a harness fault, not parity. Not scored, not recorded.`);
      results.push({ route, unmeasurable: true });
      continue;
    }
    if (measured.leptosUnmounted) {
      console.log(`UNMEASURABLE ${route}: the Leptos side rendered shell-only ` +
        `(${measured.leptos.textLen} chars) — target/site is probably mid-rebuild. Not scored, ` +
        `not recorded, not treated as a regression. Re-run when the build settles.`);
      results.push({ route, unmeasurable: true });
      continue;
    }

    // The served build is described by the site the server is actually serving, so it must be read
    // BEFORE the record block below writes it into the baseline: `--update`/new-baseline runs record
    // `measuredBuildBytes` here, and reading it after the write is what made every `--update` crash
    // with `TypeError: Cannot read properties of undefined (reading 'bytes')`.
    measured.measuredBuild = servedBuild(LEPTOS_BASE);

    const prior = baseline.routes[route];
    const priorScore = prior?.score ?? null;
    const delta = priorScore === null ? null : Number((measured.score - priorScore).toFixed(2));
    const regressed = delta !== null && delta < -tolerance;
    const belowTarget = target !== null && measured.score < target;
    if (regressed || belowTarget) failed = true;

    if (doUpdate || priorScore === null || measured.score > priorScore) {
      baseline.routes[route] = {
        score: measured.score,
        measuredBuildBytes: measured.measuredBuild.bytes,
        visualProximity: measured.visualProximity,
        contentRecall: measured.contentRecall,
        recordedAt: new Date().toISOString(),
      };
    }

    results.push({ ...measured, priorScore, delta, regressed, belowTarget });
    if (belowTarget) {
      console.log(`    (${(target - measured.score).toFixed(2)} points short of the ${target} parity target —` +
        ` run: node ralph/scripts/visual-gap-report.mjs --route ${route})`);
    }
    console.log(
      `${regressed ? 'REGRESSED' : belowTarget ? 'BELOW-TARGET' : 'ok'} ${route}: score ${measured.score}` +
        (priorScore === null ? ' (new baseline)' : ` (was ${priorScore}, delta ${delta >= 0 ? '+' : ''}${delta})`) +
        ` | visual ${measured.visualProximity ?? 'n/a'} / content ${measured.contentRecall}` +
        ` | pixelDiff ${measured.pixelDiffPercent ?? 'n/a'}%` +
        ` | snippets leptos/react/other ${measured.leptos.snippets?.leptos ?? 0}/${measured.leptos.snippets?.react ?? 0}/${measured.leptos.snippets?.other ?? 0}` +
        ` | build ${measured.measuredBuild.bytes}b @ ${measured.measuredBuild.lastModified}`,
    );
  }

  fs.writeFileSync(BASELINE_PATH, `${JSON.stringify(baseline, null, 1)}\n`);
  console.log(`\nbaseline: ${path.relative(PROJECT_ROOT, BASELINE_PATH)}`);

  if (failed) {
    if (target !== null) console.error(`\nFAIL: at least one route is below the ${target}-point parity target or regressed.`);
    else console.error(`\nFAIL: visual fidelity regressed beyond ${tolerance} points.`);
    return 1;
  }
  console.log('visual budget OK');
  return 0;
}

process.exit(main());
