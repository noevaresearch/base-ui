#!/usr/bin/env node
/* eslint-disable no-console */

// check-copy-fidelity.mjs — the PROSE half of the mirror, which nothing measured until now.
//
// THE GAP THIS CLOSES
// -------------------
// Every existing check looks at structure, pixels, code or specs:
//   * playwright-diff      — does the route mount, do headings match
//   * check-visual-budget  — pixels, widget parity, content recall (of which prose is only a LENGTH)
//   * snippet-ergonomics   — the code blocks
//   * check-docs-contract  — whether a spec carries its snippet/behaviour table
// So the website COPY — the sentences a reader actually reads, which is most of a docs page — was
// measured only as "how many characters", a number that missing sentences, wrong wording, a
// paraphrase, or an invented paragraph all move in unhelpful directions. A page can score 85 on
// fidelity with half its prose missing, because tables and affordances pad the character count.
//
// WHAT IT COMPARES
// ----------------
// Block by block, upstream's prose against this port's prose, CODE EXCLUDED (the code blocks are
// snippet-ergonomics' job; this is the copy). Blocks are paragraphs, list items, headings, table
// cells, blockquotes and definition items. Each is normalised (whitespace, quotes/dashes, inline
// code markers) and matched by word-set similarity against the port's blocks:
//
//   MATCHED   >= 0.80 similar — the same sentence, allowably re-flowed
//   CHANGED   0.50-0.80      — recognisably the same idea, worded differently: review it
//   MISSING   < 0.50         — upstream says it, this page does not
//
// It also flags REACT-MENTIONS: prose that still says "React"/"JSX" verbatim. Some of that is correct
// (prose about the upstream library), some is copy that needed adapting to the port — the report
// lists them so the distinction is made deliberately rather than by transcription.
//
// USAGE
//   node ralph/scripts/check-copy-fidelity.mjs --route react/components/checkbox
//   node ralph/scripts/check-copy-fidelity.mjs --todo-id "docs-chrome: snippet translation (batch 1)" --target 95
//
// Writes ralph/logs/visual/<component>-copy.md and .json. `--target <pct>` exits 1 below the coverage
// bar (suggested: 95 — prose is copy, not framework-specific code, so it should match closely).

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { launchChrome, killChrome, FRUGAL_CHROME_FLAGS, taskPressure } from './lib/browser.mjs';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const OUT_DIR = path.join(PROJECT_ROOT, 'ralph/logs/visual');
const UPSTREAM_BASE = process.env.UPSTREAM_DOCS_BASE || 'http://127.0.0.1:3005';
const LEPTOS_BASE = process.env.LEPTOS_DOCS_BASE || 'http://127.0.0.1:3177';
const PORT = 9895;
const MATCH_AT = 0.8;
const CHANGED_AT = 0.5;

function arg(name, dflt) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] && !process.argv[i + 1].startsWith('--') ? process.argv[i + 1] : dflt;
}
function reachable(url) {
  const r = spawnSync('curl', ['-s', '-o', '/dev/null', '-w', '%{http_code}', '--max-time', '20', url], { encoding: 'utf8' });
  return (r.stdout || '').trim() === '200';
}

const route = arg('route', null) || (() => {
  const m = (arg('todo-id', '') || '').match(/components\/([a-z0-9-]+)/i);
  return m ? `react/components/${m[1]}` : null;
})();
const target = arg('target', null) === null ? null : Number(arg('target'));
if (!route) {
  console.error('usage: check-copy-fidelity.mjs --route react/components/<name> [--target 95]');
  process.exit(1);
}

// ---- extraction: prose blocks only (code and code-adjacent chrome excluded) ----
const PROSE_PROBE = `(() => {
  // The content region: prefer the article, and never read navigation. A TOC ("On this page") and a
  // sidebar are page furniture, so counting them as copy produces findings that are not about copy —
  // and it made a heading plus its table cells read as one glued sentence ("API referenceRootIndicator").
  const article = document.querySelector('main article') || document.querySelector('article') || document.querySelector('main') || document.body;
  const SKIP = 'nav, aside, footer, header, [role=navigation], [class*=toc], [class*=TOC], [class*=TableOfContents], [class*=sidebar], [class*=Sidebar], [class*=breadcrumb], [class*=Breadcrumb]';
  // Leaf-level prose only: an element that contains another block of the same kinds is a container, and
  // joining its descendants into one string is exactly the artefact above.
  const KINDS = 'p, li, h1, h2, h3, h4, td, th, blockquote, dd, dt, figcaption';
  const BLOCK = 'p, li, h1, h2, h3, h4, td, th, blockquote, dd, dt, figcaption, div, section, ul, ol, table, tbody, tr, thead';
  const out = [];
  const seen = new Set();
  const push = (kind, text) => {
    const t = (text || '').replace(/\\s+/g, ' ').trim();
    if (t.length < 12) return;           // fragments and labels are not copy
    if (seen.has(t)) return;
    seen.add(t);
    out.push({ kind, text: t });
  };
  for (const el of article.querySelectorAll(KINDS)) {
    if (el.closest(SKIP)) continue;
    if (el.closest('pre, [class*=Playground], [class*=playground], [class*=toolbar], [class*=Toolbar], [class*=demo], [class*=Demo]')) continue;
    // Affordance labels are furniture too: an API table's row header carries a "Props"/"Hide" toggle,
    // which is a control, not copy. Measured before this filter: "Checkbox.Root.PropsHide" and
    // "API referenceRootIndicator" both read as missing prose.
    if (el.querySelector('button, [role=button], input, select, textarea')) continue;
    if (el.closest('button, [role=button]')) continue;
    if (/^(hide|show)$/i.test((el.textContent || '').trim())) continue;
    if (el.querySelector(BLOCK)) continue;   // container, not a leaf prose block
    // Keep inline <code>: it carries the prop/element names the sentence is ABOUT ("use the \`checked\`
    // prop instead"). Dropping it silently deleted the subject of half the API prose and made our own
    // probe the cause of the mismatches. Only multi-line <pre> samples are excluded — those are
    // snippet-ergonomics' territory.
    const clone = el.cloneNode(true);
    for (const c of clone.querySelectorAll('pre')) c.remove();
    // Upstream's reference headings carry a control link: <h3>Checkbox.Root.Props<a class=
    // "AdditionalTypeBackLink">Hide</a></h3>. The heading is real copy (this port should expose the
    // equivalent section), the "Hide" link is furniture — so strip the control, keep the heading.
    for (const c of clone.querySelectorAll('a[class*=BackLink], [class*=BackLink], button, [role=button], [class*=VisuallyHidden], [class*=visually-hidden]')) c.remove();
    const text = (clone.textContent || '').trim();
    if (/^(hide|show)$/i.test(text)) return;
    push(el.tagName.toLowerCase(), text.replace(/\s+(hide|show)$/i, ''));
  }
  return JSON.stringify(out);
})()`;

// ---- normalisation + similarity ----
function normalise(s) {
  return s
    .replace(/[’‘]/g, "'")
    .replace(/[“”]/g, '"')
    .replace(/[–—]/g, '-')
    .replace(/`/g, '')
    .replace(/\s+/g, ' ')
    .replace(/\s*([.,;:!?])\s*/g, '$1 ')
    .trim()
    .toLowerCase();
}
function words(s) {
  return new Set(normalise(s).split(/[^a-z0-9']+/).filter((w) => w.length > 1));
}
// Our rendered tables can carry two upstream cells' sentences in one block (upstream splits a prop's
// description from its controlled/uncontrolled note across cells; the port renders one cell). A
// containment match means the copy IS on the page, so it counts — flagged so the report still shows it.
function containment(a, b) {
  const A = normalise(a);
  const B = normalise(b);
  return A.length >= 24 && B.includes(A);
}
function similarity(a, b) {
  const A = words(a);
  const B = words(b);
  if (!A.size || !B.size) return 0;
  let inter = 0;
  for (const w of A) if (B.has(w)) inter++;
  return inter / (A.size + B.size - inter); // Jaccard
}

const REACT_RE = /\breact\b|\bjsx\b|@base-ui\/react/i;

function compareCopy(upBlocks, lxBlocks) {
  const ours = lxBlocks.map((b) => b.text);
  const findings = [];
  let matched = 0;
  let changed = 0;
  let missing = 0;
  let mergedCount = 0;
  for (const block of upBlocks) {
    let best = 0;
    let bestText = null;
    let merged = false;
    for (const cand of ours) {
      const s = similarity(block.text, cand);
      if (s > best) { best = s; bestText = cand; }
      if (containment(block.text, cand)) { best = Math.max(best, 1); bestText = cand; merged = true; }
    }
    const verdict = best >= MATCH_AT ? 'matched' : best >= CHANGED_AT ? 'changed' : 'missing';
    if (verdict === 'matched') { matched++; if (merged) mergedCount++; }
    else if (verdict === 'changed') { changed++; findings.push({ verdict, similarity: Number(best.toFixed(2)), upstream: block.text.slice(0, 220), ours: (bestText || '').slice(0, 220) }); }
    else { missing++; findings.push({ verdict, similarity: Number(best.toFixed(2)), upstream: block.text.slice(0, 220), ours: null }); }
  }
  const reactMentions = upBlocks.filter((b) => REACT_RE.test(b.text)).length;
  const ourReactMentions = lxBlocks.filter((b) => REACT_RE.test(b.text)).length;
  const coverage = upBlocks.length ? (matched / upBlocks.length) * 100 : 100;
  return { coverage: Number(coverage.toFixed(1)), matched, changed, missing, mergedCells: mergedCount, total: upBlocks.length, ourBlocks: lxBlocks.length, reactMentions, ourReactMentions, findings };
}

async function main() {
  if (!reachable(`${LEPTOS_BASE}/`)) { console.error(`FAIL: Leptos docs server unreachable at ${LEPTOS_BASE}`); return 2; }
  if (!reachable(`${UPSTREAM_BASE}/`)) { console.error(`NOTE: upstream React docs unreachable at ${UPSTREAM_BASE} — copy cannot be compared without the reference render.`); return 0; }

  fs.mkdirSync(OUT_DIR, { recursive: true });
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'copycheck-'));
  let chrome = null;
  try {
    const launched = await launchChrome([...FRUGAL_CHROME_FLAGS, '--window-size=1280,2400'], { tmpDir: tmp, port: PORT, waitMs: 45000 });
    chrome = launched.child;
    const tabs = await fetch(`http://127.0.0.1:${PORT}/json/list`).then((r) => r.json());
    const ws = new WebSocket(tabs[0].webSocketDebuggerUrl);
    await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
    let id = 0; const pending = new Map();
    ws.addEventListener('message', (ev) => { const m = JSON.parse(ev.data); if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); } });
    const send = (method, params = {}) => new Promise((res) => { const i = ++id; pending.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); setTimeout(() => { if (pending.has(i)) { pending.delete(i); res({ error: { message: 'timeout' } }); } }, 60000); });
    await send('Page.enable');
    await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 2400, deviceScaleFactor: 1, mobile: false });

    // Readiness, then stability. Readiness alone is not enough: the probe once fired while this port's
    // page was still rendering and reported 0% coverage on a page that actually carries 43.2% — a
    // number the loop would have chased as a total content gap. So the block count must stop changing
    // before anything is scored.
    const grab = async (url) => {
      await send('Page.navigate', { url });
      await new Promise((r) => setTimeout(r, 2500));
      for (let i = 0; i < 60; i++) {
        const t = await send('Runtime.evaluate', { expression: `document.readyState + '|' + (document.querySelector('h1') ? '1' : '0')`, returnByValue: true });
        if (t.result?.result?.value === 'complete|1') break;
        await new Promise((r) => setTimeout(r, 500));
      }
      let prev = -1;
      let stable = 0;
      let blocks = [];
      for (let i = 0; i < 30; i++) {
        await new Promise((r) => setTimeout(r, 700));
        const r = await send('Runtime.evaluate', { expression: PROSE_PROBE, returnByValue: true });
        blocks = r.result?.result?.value ? JSON.parse(r.result.result.value) : [];
        if (blocks.length === prev && blocks.length > 0) { stable++; if (stable >= 2) break; }
        else stable = 0;
        prev = blocks.length;
      }
      return blocks;
    };

    const up = await grab(`${UPSTREAM_BASE}/${route}`);
    const lx = await grab(`${LEPTOS_BASE}/${route}`);
    if (up.length >= 8 && lx.length < up.length * 0.25) {
      console.error(`UNMEASURABLE: upstream renders ${up.length} prose block(s) and this page ${lx.length} — that is a render that did not finish (or a shell), not a copy verdict. Re-run when the route is serving content.`);
      return 3;
    }
    const res = compareCopy(up, lx);
    const name = route.split('/').pop();

    const md = [
      `# Copy fidelity — ${route}`,
      '',
      `Generated by \`ralph/scripts/check-copy-fidelity.mjs\` at ${new Date().toISOString()} (both apps rendered at 1280px, prose blocks only — code excluded).`,
      '',
      `**Coverage: ${res.coverage}%** of upstream's ${res.total} prose block(s) ${target === null ? '' : `(target ${target}%)`}` +
        ` — matched ${res.matched} (${res.mergedCells} of them inside a block that merges sentences the upstream render splits across cells), changed ${res.changed}, missing ${res.missing}. This page renders ${res.ourBlocks} block(s).`,
      '',
      `React/JSX mentions: upstream ${res.reactMentions}, this page ${res.ourReactMentions}. A port may legitimately keep references to the upstream library; what it must not do is transcribe copy that describes React as if it were this port.`,
      '',
      '## Copy that differs (review these)',
      '',
      ...(res.findings.length
        ? res.findings.slice(0, 40).flatMap((f) => [
            `- **${f.verdict.toUpperCase()}** (${f.similarity}) — upstream: ${f.upstream}${f.ours ? `\n  - ours: ${f.ours}` : '  - ours: (no counterpart found)'}`,
          ])
        : ['Every upstream prose block has a close counterpart.']),
      '',
    ].join('\n');

    fs.writeFileSync(path.join(OUT_DIR, `${name}-copy.md`), md);
    fs.writeFileSync(path.join(OUT_DIR, `${name}-copy.json`), `${JSON.stringify({ route, generatedAt: new Date().toISOString(), ...res }, null, 1)}\n`);

    console.log(`copy fidelity: ${route} — ${res.coverage}% coverage (matched ${res.matched}/${res.total}, changed ${res.changed}, missing ${res.missing})` +
      `${target === null ? '' : ` | target ${target}%`}`);
    for (const f of res.findings.slice(0, 6)) {
      console.log(`  ${f.verdict.toUpperCase()} (${f.similarity}) upstream: ${f.upstream.slice(0, 110)}`);
    }
    if (res.findings.length > 6) console.log(`  … ${res.findings.length - 6} more in ralph/logs/visual/${name}-copy.md`);
    console.log(`report: ralph/logs/visual/${name}-copy.md`);

    if (target !== null && res.coverage < target) {
      console.error(`\nFAIL: copy coverage ${res.coverage}% is below the ${target}% target.`);
      return 1;
    }
    return 0;
  } catch (e) {
    const p = taskPressure();
    console.error(`FAIL: ${e.message}${p ? ` (cgroup tasks ${p.current}/${p.max})` : ''}`);
    return 2;
  } finally {
    killChrome(chrome);
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

process.exit(await main());
