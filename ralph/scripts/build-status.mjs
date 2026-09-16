#!/usr/bin/env node
/* eslint-disable no-console */

// build-status.mjs — the per-component progress table, as DATA.
//
// WHY IT EXISTS
// -------------
// "Progress" was two numbers that no one could reproduce by hand: items-done in the ledger (a planning
// metric) and pages-passing-the-scorecard (the quality metric). Neither told you WHICH component was behind,
// or WHY. This walks the artefacts the gates already write and emits one row per component with the axes
// that decide it, so the answer to "how is checkbox doing?" is a table, not a feeling.
//
// The output feeds the docs site's /status page (committed as crates/docs-app/status.json and regenerated in
// CI before the site build), so the breakdown is visible where the port actually lives rather than only in a
// chat message.
//
// AXES PER COMPONENT (each one is measured, never inferred):
//   parts      — documented upstream parts exposed as `Component::Part`  (check-part-surface --strict)
//   namespaced — a test exercises `<Component::…>`                        (check-component-strict)
//   snippets   — code blocks: leptos / react / other                      (snippet-ergonomics JSON)
//   copy       — prose blocks matched against upstream                    (check-copy-fidelity JSON)
//   page       — page parity score (0.6*pixel + 0.4*content)              (visual-baseline.json)
//   widget     — component-region parity                                  (visual-baseline.json)
//   verdict    — pages passing EVERY scorecard axis                       (scorecard.jsonl)
//
// USAGE
//   node ralph/scripts/build-status.mjs                 # writes crates/docs-app/status.json
//   node ralph/scripts/build-status.mjs --print         # and print a human table

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const ROOT = path.resolve(import.meta.dirname, '../..');
const OUT = path.join(ROOT, 'crates/docs-app/status.json');
const VIS = path.join(ROOT, 'ralph/logs/visual');
const print = process.argv.includes('--print');

const readJson = (p) => { try { return JSON.parse(fs.readFileSync(p, 'utf8')); } catch { return null; } };

// ---- the component list: the units the ledger tracks, from the mined specs (the authority on intent)
const componentDirs = fs.existsSync(path.join(ROOT, 'specs/library'))
  ? fs.readdirSync(path.join(ROOT, 'specs/library'), { withFileTypes: true }).filter((d) => d.isDirectory()).map((d) => d.name)
  : [];

// ---- parts surface: one strict run over everything, parsed per component
const partRows = {};
{
  const r = spawnSync('node', ['ralph/scripts/check-part-surface.mjs', '--json'], { cwd: ROOT, encoding: 'utf8' });
  const j = (() => { try { return JSON.parse((r.stdout || '').slice((r.stdout || '').indexOf('{'))); } catch { return null; } })();
  if (j?.components) for (const c of j.components) partRows[c.name] = c;
}

// ---- namespaced-path evidence, straight from the pin file the crate ships
const pinFile = path.join(ROOT, 'crates/leptos-ui/tests/part_surface.rs');
const pins = fs.existsSync(pinFile) ? fs.readFileSync(pinFile, 'utf8') : '';
const pascal = (n) => n.split(/[-_]/).map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join('');

// ---- scorecard verdicts per route (the page-level truth)
const scorecard = {};
if (fs.existsSync(path.join(ROOT, 'ralph/generated/scorecard.jsonl'))) {
  for (const line of fs.readFileSync(path.join(ROOT, 'ralph/generated/scorecard.jsonl'), 'utf8').split('\n')) {
    if (!line.trim()) continue;
    const d = readJson(line) || (() => { try { return JSON.parse(line); } catch { return null; } })();
    if (d?.route) scorecard[d.route.split('/').pop()] = d;
  }
}

const baseline = readJson(path.join(ROOT, 'ralph/generated/visual-baseline.json'))?.routes || {};

const rows = componentDirs.map((name) => {
  const route = `react/components/${name}`;
  const snip = readJson(path.join(VIS, `${name}-snippets.json`));
  const copy = readJson(path.join(VIS, `${name}-copy.json`));
  const base = baseline[route] || baseline[`react/utils/${name}`] || null;
  const sc = scorecard[name] || scorecard[`utils-${name}`] || null;
  const parts = partRows[name] || null;
  const hasPin = new RegExp(`<${pascal(name)}::[A-Z]`).test(pins);
  const m = snip?.metrics || {};
  return {
    component: name,
    docsRoute: fs.existsSync(path.join(ROOT, 'crates/docs-app/src/pages', `${name.replace(/-/g, '_')}_page.rs`)) ? route : null,
    parts: parts ? { exposed: parts.exposed ?? null, documented: parts.documented ?? null, ok: parts.missing === 0 } : null,
    namespacedPath: hasPin ? 'pinned' : parts?.documented ? 'missing' : 'n/a',
    snippets: m.snippetLanguages ? { ...m.snippetLanguages, reactToReact: m.reactToReactBlocks ?? 0 } : null,
    exampleLengthPct: m.lengthSimilarity ?? null,
    attributeRatio: m.upstreamMeanAttrs ? Number((m.leptosMeanAttrs / m.upstreamMeanAttrs).toFixed(2)) : null,
    ergonomics: snip?.score ?? null,
    copyCoverage: copy?.coverage ?? null,
    pageParity: base?.score ?? null,
    widgetParity: base?.widgetParity ?? null,
    pageVerdict: sc?.verdict ?? null,
    failingAxes: sc ? sc.axes.filter((a) => a.status !== 'PASS').map((a) => a.axis) : null,
  };
});

// ---- the two headline numbers, computed the same way the report computes them
const scorecardRoutes = Object.keys(scorecard);
const passing = scorecardRoutes.filter((r) => scorecard[r].verdict === 'PASS');
const todo = fs.readFileSync(path.join(ROOT, 'TODO.md'), 'utf8');
let items = 0; let done = 0;
for (const m of todo.matchAll(/^- \[( |x)\] ([^\n]+)\n((?: {6}[^\n]*\n)*)/gm)) { items++; if (m[1] === 'x') done++; }

const payload = {
  generatedAt: new Date().toISOString(),
  headline: {
    itemsDone: done, itemsTotal: items, itemsPct: Number(((100 * done) / Math.max(items, 1)).toFixed(1)),
    pagesMeasured: scorecardRoutes.length, pagesPassing: passing.length, mirroredRoutes: 18,
    pagesPassingPct: Number(((100 * passing.length) / 18).toFixed(1)),
    explanation: 'Items-done counts ledger entries marked done. Pages-passing counts mirrored routes where EVERY scorecard axis passes (structure, page parity >=90, widget >=97, snippet language react=0, example length >=80% of upstream, attribute density >=0.8x, copy >=95%, zero React mentions, package alias clean). An unmeasured axis never counts as a pass, so a route with no fresh measurement is not counted as passing.',
  },
  components: rows.sort((a, b) => (a.component < b.component ? -1 : 1)),
};
fs.mkdirSync(path.dirname(OUT), { recursive: true });
fs.writeFileSync(OUT, `${JSON.stringify(payload, null, 1)}\n`);

if (print) {
  const cell = (v, w = 9) => (v === null || v === undefined ? '—' : String(v)).padEnd(w).slice(0, w);
  console.log(`items ${done}/${items} (${payload.headline.itemsPct}%) | pages passing ${passing.length}/${scorecardRoutes.length} measured of 18`);
  console.log(`\n${'component'.padEnd(18)}${'parts'.padEnd(9)}${'pin'.padEnd(9)}${'snips l/r'.padEnd(11)}${'len%'.padEnd(7)}${'attrs'.padEnd(7)}${'copy%'.padEnd(7)}${'page'.padEnd(7)}${'widget'.padEnd(8)}verdict`);
  for (const r of payload.components) {
    const sn = r.snippets ? `${r.snippets.leptos}/${r.snippets.react}` : null;
    console.log(`${r.component.padEnd(18)}${cell(r.parts ? `${r.parts.exposed}/${r.parts.documented}` : null)}${cell(r.namespacedPath)}${cell(sn, 11)}${cell(r.exampleLengthPct, 7)}${cell(r.attributeRatio, 7)}${cell(r.copyCoverage, 7)}${cell(r.pageParity, 7)}${cell(r.widgetParity, 8)}${r.pageVerdict ?? 'unmeasured'}`);
  }
}
console.log(`status written: ${path.relative(ROOT, OUT)}`);
