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
import path from 'node:path';

const ROOT = path.resolve(import.meta.dirname, '../..');
const OUT = path.join(ROOT, 'ralph/logs/scorecard');
const LEPTOS = process.env.LEPTOS_DOCS_BASE || 'http://127.0.0.1:3177';
const strict = process.argv.includes('--strict');
const jsonOut = process.argv.includes('--json');

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
axes.push({ axis: 'structure', bar: 'mounts + headings', ...run('playwright-diff.mjs', ['--route', route]) });

// 2. page + widget parity, snippet language
const budget = run('check-visual-budget.mjs', ['--route', route]);
const score = num(/score\s+([\d.]+)/, budget.out, null);
const widget = num(/widget\s+([\d.]+)%/, budget.out, null);
axes.push({ axis: 'page parity', bar: '>=90', ...budget, value: score, status: score === null ? 'UNMEASURED' : budget.status === 'UNMEASURED' ? 'UNMEASURED' : score >= 90 ? 'PASS' : 'FAIL' });
axes.push({ axis: 'widget parity', bar: '>=97', ...budget, value: widget, status: widget === null ? 'UNMEASURED' : widget >= 97 ? 'PASS' : 'FAIL' });

// 3. snippet ergonomics: the axes that a hollow stub fails
const ergo = run('snippet-ergonomics.mjs', ['--route', route]);
let m = {};
try { m = JSON.parse(fs.readFileSync(path.join(ROOT, `ralph/logs/visual/${name}-snippets.json`), 'utf8')).metrics || {}; } catch {}
const reactBlocks = m.reactToReactBlocks ?? null;
const lengthSim = m.lengthSimilarity ?? null;
const attrRatio = m.upstreamMeanAttrs ? Number((m.leptosMeanAttrs / m.upstreamMeanAttrs).toFixed(2)) : null;
// The language axis judges ONLY the language. Inheriting the length floor here (as a first cut did) made a
// page with react=0 report a language FAIL, which is exactly the kind of cross-wired axis that hides what
// is actually wrong — length has its own axis and its own bar.
axes.push({ axis: 'snippet language', bar: 'react = 0', ...ergo, value: reactBlocks, status: reactBlocks === null ? 'UNMEASURED' : reactBlocks === 0 ? 'PASS' : 'FAIL' });
axes.push({ axis: 'example length', bar: '>=80% of upstream', ...ergo, value: lengthSim, status: lengthSim === null ? 'UNMEASURED' : lengthSim >= 80 ? 'PASS' : 'FAIL' });
axes.push({ axis: 'attribute density', bar: '>=0.8x upstream', ...ergo, value: attrRatio, status: attrRatio === null ? 'UNMEASURED' : attrRatio >= 0.8 ? 'PASS' : 'FAIL' });

// 4. copy (prose)
axes.push({ axis: 'copy coverage', bar: '>=95%', ...run('check-copy-fidelity.mjs', ['--route', route, '--target', '95']) });

// 5. React mentions on the rendered page
axes.push({ axis: 'react mentions', bar: '0 defects', ...run('check-react-mentions.mjs', ['--route', route]) });

// 6. the port's name (repo-wide, not per route — reported once)
axes.push({ axis: 'package alias', bar: 'resolves, 0 defects', ...run('check-package-alias.mjs', []) });

const fails = axes.filter((a) => a.status === 'FAIL').length;
const unmeasured = axes.filter((a) => a.status === 'UNMEASURED').length;

if (jsonOut) console.log(JSON.stringify({ route, generatedAt: new Date().toISOString(), axes: axes.map(({ axis, bar, status, value }) => ({ axis, bar, status, value })), fails, unmeasured, verdict: fails || unmeasured ? 'NOT DONE' : 'PASS' }, null, 1));
else {
  console.log(`\nPAGE SCORECARD — ${route}`);
  for (const a of axes) {
    const pad = a.axis.padEnd(18);
    const val = a.value === null || a.value === undefined ? '' : ` (${a.value})`;
    console.log(`  ${a.status.padEnd(10)} ${pad} bar ${String(a.bar).padEnd(20)}${val}`);
  }
  console.log(`\n  verdict: ${fails || unmeasured ? 'NOT DONE' : 'PASS'} — ${fails} failing axis/axes, ${unmeasured} unmeasured (an unmeasured axis is never a pass)`);
  const REASON = {
    structure: /FAIL|panic|not found|timeout/i,
    'page parity': /score\s+([\d.]+)/i,
    'widget parity': /widget\s+([\d.]+)%/i,
    'snippet language': /LANGUAGE|react-to-react|are upstream's React code/i,
    'example length': /length check|length similarity/i,
    'attribute density': /attribute density|meanAttrs|attributes per element/i,
    'copy coverage': /copy fidelity/i,
    'react mentions': /^FAIL|defect/i,
    'package alias': /FAIL|defect\(s\)/i,
  }[a.axis] || /FAIL|UNMEASURABLE/i;
  for (const a of axes.filter((x) => x.status !== 'PASS')) {
    const reason = (a.out || '').split('\n').find((l) => REASON.test(l));
    if (reason) console.log(`    ${a.axis}: ${reason.trim().slice(0, 150)}`);
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
