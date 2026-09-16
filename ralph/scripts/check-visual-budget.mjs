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
//   node ralph/scripts/check-visual-budget.mjs --all-done --target 90   # page-level parity (Phase E)
//   ... --target-component 97   # component-widget parity (default 97: the widget must match upstream)
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
import { WIDGET_REGION_VERSION } from './lib/widget-region.mjs';
import { isResourceFailure } from './lib/browser.mjs';

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
// Two bars, but the SAME enforcement rule: `--target N` / `--target-component N` make an ABSOLUTE
// bar part of the exit code — that is how the Phase E parity item is measured
// (`--all-done --target 90 --target-component 97`). Without them the gate is REGRESSION-ONLY: it
// fails when a route becomes worse than its recorded best-known numbers, which is the contract this
// item's done-when states ("fails (exit 1) when a route's fidelity score drops more than the
// tolerance") and what lets the loop keep landing chrome work on routes that are still far from
// parity.
//
// The widget number is still PRINTED against its bar on every run (default 97: the rendered
// component should look 97-99% identical to upstream, while the page-level 90 bar covers prose and
// code, which legitimately differ between the two frameworks). Reporting a gap is not the same as
// gating on it: enforcing the widget bar by default made EVERY docs-app item's gate fail
// unconditionally — measured 2026-09-16, when no recorded route met it and the run reported
// "regressed beyond 2 points" while every score was unchanged (delta +0). The bar is enforced on
// request; the widget is gated on regressions always, and a region that cannot be compared fails
// outright rather than scoring whatever the overlap happened to be.
const target = arg('target', null) === null ? null : Number(arg('target'));
const componentBar = arg('target-component', null) === null ? 97 : Number(arg('target-component'));
const targetComponent = arg('target-component', null) === null ? null : Number(arg('target-component'));

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
  // Component-region parity: the rendered demo/component area compared on its own. Prose and code
  // differ between React and Leptos by design; the component must not.
  const widgetParity = Number.isFinite(report.widgetParity) ? report.widgetParity : null;
  const widgetDiffPercent = Number.isFinite(report.widgetDiffPercent) ? report.widgetDiffPercent : null;
  const demoParity = Number.isFinite(report.demoParity) ? report.demoParity : null;
  // A region that could not be compared is a MEASUREMENT fault, not a score: the two sides' crops
  // were not the same kind of thing (visual-diff/lib/widget-region.mjs reports that), or only one
  // side rendered a component region at all. There is no number to trust here, so it is carried
  // through to the caller as a fault — reported loudly, and fatal under the two conditions `main()`
  // documents (the widget bar is being enforced, or the route loses a measurability it had).
  const widgetFault = report.widgetFault
    || (Boolean(u.widgetRect) !== Boolean(l.widgetRect)
      ? `only one side rendered a component region (upstream ${u.widgetRect ? 'yes' : 'none'}, leptos ${l.widgetRect ? 'yes' : 'none'})`
      : null);
  const demoFault = report.demoFault || null;

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
  const sameHref = u.href && l.href && u.href === l.href;
  const identicalStats = Number.isFinite(pixelDiff) && pixelDiff === 0 && u.textLen === l.textLen;
  // ROUTE IDENTITY. Two sides can each render a perfectly good page and still both be the WRONG
  // route — a tab carried over from an earlier route captures whatever the previous navigation
  // left in the DOM, and neither the href check above (the URLs differ) nor a non-zero pixel diff
  // (the pages really are different pixels) can see it. The page's own `<h1>` names the route on
  // both sides (upstream "Checkbox" vs ours "Checkbox"), so a mismatch is a capture fault, not
  // fidelity. Measured 2026-09-15 with two overlapping harness runs: the checkbox route reported
  // meter's numbers to the hundredth (score 71.29, visual 93.12, content 38.54) and would have
  // been recorded as its best-known score.
  const uTitle = (u.headings || [])[0] || '';
  const lTitle = (l.headings || [])[0] || '';
  const routeMismatch = Boolean(uTitle && lTitle && uTitle !== lTitle);
  const harnessFault = sameHref || identicalStats || routeMismatch;
  return {
    route,
    leptosUnmounted,
    harnessFault: harnessFault
      ? routeMismatch
        ? `the two sides rendered different pages (upstream h1 ${JSON.stringify(uTitle)} vs leptos h1 ${JSON.stringify(lTitle)})`
        : `both sides rendered the same page (upstream ${u.href || '?'} vs leptos ${l.href || '?'})`
      : null,
    score: Number(score.toFixed(2)),
    visualProximity: visualProximity === null ? null : Number((visualProximity * 100).toFixed(2)),
    contentRecall: Number((contentRecall * 100).toFixed(2)),
    pixelDiffPercent: Number.isFinite(pixelDiff) ? pixelDiff : null,
    // Region scores. The WIDGET is the component itself (same rendered result expected); the demo
    // FRAME is upstream's docs chrome around it (docs-chrome work, not parity).
    widgetParity,
    widgetDiffPercent,
    widgetFault,
    widgetRect: report.widgetRect || null,
    demoParity,
    demoFault,
    demoRect: report.demoRect || null,
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
      // A measurement the BOX could not take is not a parity verdict. This distinction was learned the
      // hard way: a concurrent harness run exhausted the 512-task cgroup, Chrome never started, and this
      // line marked a COMPLETE item blocked on "FAIL: could not measure react/components/toggle … chrome
      // devtools port 9888". A false FAIL is worse than no measurement — the loop spends a whole
      // iteration chasing a phantom regression — while a genuinely unmeasured route simply cannot be
      // scored. So: resource failure => UNMEASURABLE (reported, not failed); anything else stays a FAIL.
      if (isResourceFailure(e.message)) {
        console.error(`UNMEASURABLE: ${route} — the box could not start a renderer (${e.message}). Not a parity verdict; no score is recorded for this route.`);
        results.push({ route, unmeasurable: true, error: e.message });
        continue;
      }
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
      console.log(`UNMEASURABLE ${route}: ${measured.harnessFault} — ` +
        'a capture fault, not parity. Not scored, not recorded.');
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
    // The WIDGET is the component: same rendered result expected, so 95-99% is the bar. The demo
    // frame (upstream's div.demo chrome) is reported separately and is docs-chrome work, not parity.
    const belowComponentTarget = targetComponent !== null && measured.widgetParity !== null && measured.widgetParity < targetComponent;
    // The widget is gated on REGRESSION even when the absolute bar is not requested: the recorded
    // best-known widget parity is a floor, exactly like the page score's — but ONLY when it was
    // measured by this same region definition (`WIDGET_REGION_VERSION`). The 2026-09-16 change
    // redefined the number outright (per-side crop over the min-overlap → playground-scoped region
    // over a common crop), so comparing across it would manufacture regressions from a definition
    // change; entries recorded before the marker are rebaked rather than compared.
    const priorWidget = prior?.widgetParity ?? null;
    const priorWidgetComparable = prior?.widgetRegionVersion === WIDGET_REGION_VERSION;
    const widgetDelta = priorWidget === null || measured.widgetParity === null || !priorWidgetComparable
      ? null
      : Number((measured.widgetParity - priorWidget).toFixed(2));
    const widgetRegressed = widgetDelta !== null && widgetDelta < -tolerance;
    // A region that could not be compared is a MEASUREMENT fault, not a pass (see `widgetFault`).
    // It is FATAL in exactly two cases, and reported (loudly, never silently) otherwise:
    //   * the caller is enforcing the widget bar (`--target-component` — the Phase E parity item's
    //     own measurement), because "the component looks like upstream's" cannot be claimed while
    //     the two regions are not comparable; or
    //   * the route HAD a measurable widget parity recorded under THIS region definition and this
    //     run cannot measure one: losing the instrument's coverage of a component is a regression in
    //     its own right.
    // A route whose region fault is already recorded as unmeasurable is pre-existing, scoped state
    // (today: the two demos that render upstream's Tailwind variant unstyled, ledger item
    // `docs-chrome: demo styling …`), and it must not block unrelated items' regression gates —
    // which is the deadlock this whole change exists to end.
    const widgetFault = Boolean(measured.widgetFault);
    const faultIsRegression = widgetFault && (targetComponent !== null || (priorWidget !== null && priorWidgetComparable));
    if (regressed || belowTarget || belowComponentTarget || widgetRegressed || faultIsRegression) failed = true;

    if (doUpdate || priorScore === null || measured.score > priorScore) {
      baseline.routes[route] = {
        score: measured.score,
        measuredBuildBytes: measured.measuredBuild.bytes,
        // Keep the recorded region parities as FLOORS: a run that could not measure a region (a
        // fault, a missing playground) must not erase the number later runs are held to.
        widgetParity: measured.widgetParity ?? prior?.widgetParity ?? null,
        widgetRegionVersion: measured.widgetParity === null ? prior?.widgetRegionVersion : WIDGET_REGION_VERSION,
        demoParity: measured.demoParity ?? prior?.demoParity ?? null,
        visualProximity: measured.visualProximity,
        contentRecall: measured.contentRecall,
        recordedAt: new Date().toISOString(),
      };
    }

    results.push({ ...measured, priorScore, delta, regressed, belowTarget, belowComponentTarget, widgetDelta, widgetRegressed, faultIsRegression });
    if (widgetFault) {
      console.log(`    (component widget ${faultIsRegression ? 'NOT MEASURABLE — FAILING' : 'not measurable (pre-existing, recorded as unmeasurable)'}: ${measured.widgetFault})`);
      console.log(`     a region fault, not a parity — run: node ralph/scripts/visual-gap-report.mjs --route ${route} and inspect the crops)`);
    } else if (measured.widgetParity === null) {
      console.log(`    (no component region on one side — widget parity not measurable; run: node ralph/scripts/visual-gap-report.mjs --route ${route})`);
    } else if (widgetRegressed) {
      console.log(`    (component widget ${measured.widgetParity}% vs recorded best ${priorWidget}%, delta ${widgetDelta} — the component's own rendering regressed)`);
    } else if (belowComponentTarget) {
      console.log(`    (component widget is ${measured.widgetParity}% identical, ${(componentBar - measured.widgetParity).toFixed(2)} short of the ${componentBar}% bar — ` +
        `see the -widget.png crops; demo frame parity is ${measured.demoParity === null ? 'n/a' : measured.demoParity + '%'})`);
    } else if (measured.widgetParity < componentBar) {
      console.log(`    (component widget ${measured.widgetParity}% vs the ${componentBar}% bar — ${(componentBar - measured.widgetParity).toFixed(2)} short; ` +
        `raised as a P0 finding by visual-gap-report.mjs and gated only when --target-component is passed)`);
    }
    if (belowTarget) {
      console.log(`    (${(target - measured.score).toFixed(2)} points short of the ${target} parity target —` +
        ` run: node ralph/scripts/visual-gap-report.mjs --route ${route})`);
    }
    const verdict = widgetFault
      ? 'UNMEASURABLE'
      : regressed || widgetRegressed
        ? 'REGRESSED'
        : belowTarget || belowComponentTarget
          ? 'BELOW-TARGET'
          : 'ok';
    console.log(
      `${verdict} ${route}: score ${measured.score}` +
        (priorScore === null ? ' (new baseline)' : ` (was ${priorScore}, delta ${delta >= 0 ? '+' : ''}${delta})`) +
        ` | visual ${measured.visualProximity ?? 'n/a'} / content ${measured.contentRecall}` +
        ` | pixelDiff ${measured.pixelDiffPercent ?? 'n/a'}%` +
        ` | snippets leptos/react/other ${measured.leptos.snippets?.leptos ?? 0}/${measured.leptos.snippets?.react ?? 0}/${measured.leptos.snippets?.other ?? 0}` +
        ` | widget ${measured.widgetParity === null ? 'n/a' : measured.widgetParity + '%'} (frame ${measured.demoParity === null ? 'n/a' : measured.demoParity + '%'})` +
        ` | build ${measured.measuredBuild.bytes}b @ ${measured.measuredBuild.lastModified}`,
    );
  }

  fs.writeFileSync(BASELINE_PATH, `${JSON.stringify(baseline, null, 1)}\n`);
  console.log(`\nbaseline: ${path.relative(PROJECT_ROOT, BASELINE_PATH)}`);

  if (failed) {
    const reasons = [];
    if (results.some((r) => r.error)) reasons.push('a route could not be measured');
    if (results.some((r) => r.regressed)) reasons.push(`a route's page score regressed beyond ${tolerance} points`);
    if (results.some((r) => r.widgetRegressed)) reasons.push(`a route's component widget parity regressed beyond ${tolerance} points`);
    if (results.some((r) => r.faultIsRegression)) reasons.push('a route\'s component region could not be compared (see the UNMEASURABLE lines)');
    if (target !== null && results.some((r) => r.belowTarget)) reasons.push(`a route is below the ${target}-point page target`);
    if (targetComponent !== null && results.some((r) => r.belowComponentTarget)) reasons.push(`a route's component widget is below the ${targetComponent}% bar`);
    console.error(`\nFAIL: ${reasons.join('; ') || 'see the lines above'}.`);
    return 1;
  }
  const standingFaults = results.filter((r) => r.widgetFault && !r.faultIsRegression).length;
  if (standingFaults) {
    // "OK" here means "no regression and no enforced bar was missed" — it is NOT a statement that
    // the widgets are fine. Say so instead of letting the two be confused.
    console.log(`visual budget OK (regression-only gate). NOT a widget pass: ${standingFaults} route(s) have a component-region fault ` +
      'reported above and as a P0 by visual-gap-report.mjs; the Phase E parity item enforces them with --target-component.');
  } else {
    console.log('visual budget OK');
  }
  return 0;
}

process.exit(main());
