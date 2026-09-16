#!/usr/bin/env node
/* eslint-disable no-console */

// snippet-ergonomics.mjs — does the port's example code READ like upstream's example code?
//
// WHY THIS EXISTS
// ---------------
// Behaviour parity and even "the snippet is Leptos, not React" parity are not the whole job. The
// mirrored examples must also be ergonomically comparable: upstream teaches
//
//     <Checkbox.Root><Checkbox.Indicator /></Checkbox.Root>
//
// while a port that teaches
//
//     checkbox_root_view(CheckboxRootViewProps { .. })
//
// is faithful in behaviour and alien in style — a reader comparing the two pages learns a different
// mental model, and the docs quietly advertise the port's internals instead of its API. The
// convention upstream establishes is dotted namespacing (`<Checkbox.Root>`), attribute-shaped props,
// same nesting shape and comparable brevity.
//
// WHAT IT MEASURES
// ----------------
// It renders a route on both apps, extracts every code block's text, and compares their STRUCTURE:
//
//   treeShape     — the sequence of element/component tags in each snippet, as a tree signature
//                   (upstream's `<Checkbox.Root>` maps to an expected `<CheckboxRoot>`), matched by
//                   longest-common-subsequence. Catches "the example composes different parts".
//   naming        — dotted-namespace parity: upstream `A.B` should appear as a single capitalised
//                   component whose name carries both parts (CheckboxRoot / ui::Checkbox::Root),
//                   not as a lowercase function call.
//   props         — mean attribute count per element, upstream vs ours: props as attributes vs
//                   props as a struct literal is the ergonomic gap the reader feels first.
//   rawCallSmell  — count of internal-shaped calls in our snippets (`*_view(`, `Props {`, `cx(`
//                   used where upstream writes an attribute). These are never upstream idiom.
//   brevity       — lines ours / lines upstream: a faithful example should not be dramatically
//                   longer than the thing it mirrors.
//
// Output: a 0..100 ergonomics score plus NAMED findings per snippet (the work list), written to
// ralph/logs/visual/<component>-snippets.md and .json. Exit 0 unless `--target <n>` is given, in
// which case a score below the target exits 1 — the same gate shape as the visual budget.
//
// USAGE
//   node ralph/scripts/snippet-ergonomics.mjs --route react/components/checkbox
//   node ralph/scripts/snippet-ergonomics.mjs --todo-id "docs-content: components/checkbox" --target 85

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { launchChrome, killChrome, FRUGAL_CHROME_FLAGS, taskPressure } from './lib/browser.mjs';
import { astAvailable, loadGrammars, reactElementTree, leptosElementTree, compareTrees } from './lib/ast-compare.mjs';
import { classifyAll, classifySnippet, verbatimRatio } from './lib/snippet-lang.mjs';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const OUT_DIR = path.join(PROJECT_ROOT, 'ralph/logs/visual');
const UPSTREAM_BASE = process.env.UPSTREAM_DOCS_BASE || 'http://127.0.0.1:3005';
const LEPTOS_BASE = process.env.LEPTOS_DOCS_BASE || 'http://127.0.0.1:3177';
const PORT = 9893;

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
// The simple, first-order bar the user asked to start from: snippet size within 20% of upstream.
const lengthFloor = Number(arg('length-floor', '0.8'));
if (!route) {
  console.error('usage: snippet-ergonomics.mjs --route react/components/<name> [--target 85]');
  process.exit(1);
}

// ---------------------------------------------------------------- extraction

const SNIPPET_PROBE = `JSON.stringify([...document.querySelectorAll('main pre, article pre, pre')]
  .map(p => p.textContent || '')
  .filter(t => t.trim().length > 0)
  .slice(0, 60))`;

// --- React side: parse JSX out of a snippet ---
function reactSignature(text) {
  const elements = [];
  const tagRe = /<(\/?)([A-Za-z][\w.:]*)((?:"[^"]*"|'[^']*'|[^>"'])*?)(\/?)>/g;
  let m;
  while ((m = tagRe.exec(text)) !== null) {
    const [, closing, tag, attrText, selfClose] = m;
    if (closing) continue;
    const attrs = (attrText.match(/[A-Za-z_][\w-]*\s*=/g) || []).length
      + (/\.\.\./.test(attrText) ? 1 : 0);
    elements.push({ tag, attrs, selfClose: Boolean(selfClose) });
  }
  return {
    elements,
    dots: elements.filter((e) => e.tag.includes('.')).length,
    hooks: (text.match(/\buse[A-Z]\w*/g) || []).length,
    propsStructs: 0,
    viewFns: 0,
    lines: text.split('\n').filter((l) => l.trim()).length,
  };
}

// --- Leptos side: parse view! markup and Rust shapes ---
function leptosSignature(text, { lookForLeptos = true } = {}) {
  const elements = [];
  // view! markup: <div ...>, <CheckboxRoot .../>, <ui::Button/>
  const tagRe = /<(\/?)([A-Za-z_][\w:]*)((?:"[^"]*"|'[^']*'|[^>"'])*?)(\/?)>/g;
  let m;
  while ((m = tagRe.exec(text)) !== null) {
    const [, closing, tag, attrText, selfClose] = m;
    if (closing) continue;
    const attrs = (attrText.match(/[A-Za-z_][\w-]*\s*=/g) || []).length;
    elements.push({ tag, attrs, selfClose: Boolean(selfClose) });
  }
  // internal-shaped calls: the smell this checker exists for
  const viewFns = (text.match(/\b[a-z][a-z0-9_]*_view\s*\(/g) || []).length;
  const propsStructs = (text.match(/\b[A-Z]\w*Props\s*\{/g) || []).length;
  const cxCalls = (text.match(/\bcx\s*\(/g) || []).length;
  return {
    elements,
    dots: elements.filter((e) => e.tag.includes(':')).length,
    hooks: lookForLeptos ? (text.match(/\b(use_[a-z_]+|Signal|RwSignal|Memo)\b/g) || []).length : 0,
    propsStructs,
    viewFns,
    cxCalls,
    lines: text.split('\n').filter((l) => l.trim()).length,
  };
}

// ---------------------------------------------------------------- comparison

/** Canonical dotted/snake component name -> comparable token, so Checkbox.Root ~ CheckboxRoot ~ ui::Checkbox::Root. */
function canon(name) {
  return name
    .replace(/::/g, '.')
    .replace(/_/g, '.')
    .toLowerCase()
    .split('.')
    .filter(Boolean)
    .join('');
}

function isComponentTag(tag) {
  const last = tag.split(/[.:]/).pop() || '';
  return /^[A-Z]/.test(last);
}

function lcsRatio(a, b) {
  if (!a.length || !b.length) return 0;
  const dp = Array.from({ length: a.length + 1 }, () => new Array(b.length + 1).fill(0));
  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      dp[i][j] = a[i - 1] === b[j - 1] ? dp[i - 1][j - 1] + 1 : Math.max(dp[i - 1][j], dp[i][j - 1]);
    }
  }
  return dp[a.length][b.length] / Math.max(a.length, b.length);
}

function comparePage(upTexts, lxTexts) {
  const findings = [];
  const upElements = [];
  const lxElements = [];
  let upLines = 0;
  let lxLines = 0;
  let upAttrs = 0;
  let upAttrElements = 0;
  let lxAttrs = 0;
  let lxAttrElements = 0;
  let upDots = 0;
  let lxDots = 0;
  let viewFns = 0;
  let propsStructs = 0;
  let cxCalls = 0;
  const dottedUpstream = [];

  upTexts.forEach((t) => {
    const s = reactSignature(t);
    upElements.push(...s.elements.map((e) => ({ ...e, canon: canon(e.tag) })));
    upLines += s.lines;
    upDots += s.dots;
    for (const e of s.elements) {
      if (isComponentTag(e.tag) && e.tag.includes('.')) dottedUpstream.push(e.tag);
      upAttrs += e.attrs;
      upAttrElements++;
    }
  });
  lxTexts.forEach((t) => {
    const s = leptosSignature(t);
    lxElements.push(...s.elements.map((e) => ({ ...e, canon: canon(e.tag) })));
    lxLines += s.lines;
    lxDots += s.dots;
    viewFns += s.viewFns;
    propsStructs += s.propsStructs;
    cxCalls += s.cxCalls;
    for (const e of s.elements) {
      lxAttrs += e.attrs;
      lxAttrElements++;
    }
  });

  const upTags = upElements.map((e) => e.canon);
  const lxTags = lxElements.map((e) => e.canon);
  const treeShape = lcsRatio(upTags, lxTags);

  // naming: how many of upstream's dotted components have a comparable component here?
  const uniqDotted = [...new Set(dottedUpstream.map(canon))];
  // suffix-matched, so a namespaced path such as ui::Checkbox::Root counts as Checkbox.Root's
  // counterpart (equality would score the port's own recommended idiom as a miss)
  const lxComponentNames = [...new Set(lxElements.filter((e) => isComponentTag(e.tag)).map((e) => e.canon))];
  // Spelling style: upstream's `A.B` should appear as `A::B` (module path, capitalised part), not as
  // a flattened `AB`. Suffix matching accepts either as a counterpart; this metric rewards the
  // faithful spelling, which is the ergonomic claim the ledger makes.
  const dottedCount = lxTexts.reduce((n, t) => n + (t.match(/<\s*[A-Z]\w*::[A-Z]\w*/g) || []).length, 0);
  const flatCount = lxTexts.reduce((n, t) => n + (t.match(/<\s*[A-Z]\w*(?!::)\s*\/?>/g) || []).length, 0);
  const namespaceStyle = dottedCount + flatCount === 0 ? (uniqDotted.length ? 0 : 1) : dottedCount / (dottedCount + flatCount);
  const matched = uniqDotted.filter((d) => lxComponentNames.some((n) => n === d || n.endsWith(d)));
  const naming = uniqDotted.length ? matched.length / uniqDotted.length : 1;

  const meanAttrsUp = upAttrElements ? upAttrs / upAttrElements : 0;
  const meanAttrsLx = lxAttrElements ? lxAttrs / lxAttrElements : 0;
  const props = meanAttrsUp > 0 ? Math.min(1, meanAttrsLx / meanAttrsUp) : 1;

  const brevity = upLines > 0 ? Math.min(1, upLines / Math.max(lxLines, 1)) : 1;

  // The deliberately simple check to start from: how close are the two sides in raw size? Compared
  // on both non-empty lines and characters, since line counts alone can hide very different code.
  const upChars = upTexts.reduce((n, t) => n + t.replace(/\s+/g, ' ').trim().length, 0);
  const lxChars = lxTexts.reduce((n, t) => n + t.replace(/\s+/g, ' ').trim().length, 0);
  const ratio = (a, b) => (a > 0 && b > 0 ? Math.min(a, b) / Math.max(a, b) : 0);
  const lineSimilarity = ratio(upLines, lxLines);
  const charSimilarity = ratio(upChars, lxChars);
  const lengthSimilarity = Math.min(lineSimilarity, charSimilarity);

  // findings — each names the snippet-level evidence and the fix
  if (uniqDotted.length && matched.length < uniqDotted.length) {
    const missing = uniqDotted.filter((d) => !lxComponentNames.some((n) => n === d || n.endsWith(d)));
    findings.push({
      severity: 'P0',
      area: 'namespaced components',
      gap: `upstream teaches ${uniqDotted.length} dotted component(s) (${[...new Set(dottedUpstream)].slice(0, 6).join(', ')}); ${missing.length} have no comparable component in this page's snippets (${missing.slice(0, 6).join(', ')})`,
      fix: 'expose the parts as namespaced components used with view! markup (the upstream idiom, e.g. a `mod ui { pub fn Button() -> impl IntoView }` used as `<ui::Button />`, or a CheckboxRoot-style component), so examples read like `<Checkbox.Root>` instead of a function call with a props struct',
    });
  }
  if (viewFns > 0 || propsStructs > 0) {
    findings.push({
      severity: 'P0',
      area: 'raw call shape',
      gap: `the snippets call internal-shaped APIs ${viewFns} time(s) (\`*_view(...)\`) and build ${propsStructs} props struct(s) (\`…Props { .. }\`) where upstream writes an element with attributes`,
      fix: 'move the props into view! attributes and name the parts like upstream; props structs are the port internals, not the teaching surface',
    });
  }
  if (treeShape < 0.6) {
    findings.push({
      severity: 'P1',
      area: 'tree shape',
      gap: `the element/component sequence matches upstream only ${(treeShape * 100).toFixed(0)}% (LCS over ${upTags.length} upstream / ${lxTags.length} local elements)`,
      fix: 'mirror the example composition the way upstream teaches it — same parts, same nesting, same order',
    });
  }
  if (props < 0.5) {
    findings.push({
      severity: 'P2',
      area: 'attribute density',
      gap: `upstream averages ${meanAttrsUp.toFixed(1)} attribute(s) per element, this page ${meanAttrsLx.toFixed(1)}`,
      fix: 'show the same props as attributes on the same elements so the reader sees the API surface upstream documents',
    });
  }
  if (lengthSimilarity < lengthFloor) {
    findings.push({
      severity: 'P1',
      area: 'length against upstream',
      gap: `snippet size similarity ${(lengthSimilarity * 100).toFixed(0)}% (bar ${(lengthFloor * 100).toFixed(0)}%) — ${lxLines} lines / ${lxChars} chars here vs ${upLines} lines / ${upChars} chars upstream`,
      fix: 'bring the examples to upstream\'s size class: shorten what the port adds (props structs, wiring) and show the same amount of code upstream shows, or add the examples upstream carries that this page lacks',
    });
  }
  if (brevity < 0.55) {
    findings.push({
      severity: 'P2',
      area: 'brevity',
      gap: `the mirrored snippets are ${(lxLines / Math.max(upLines, 1)).toFixed(2)}x upstream's line count (${lxLines} vs ${upLines})`,
      fix: 'trim the examples to the shape upstream shows; a reader comparing pages should not see the port as markedly more verbose',
    });
  }

  // The regex signature scores always; the AST metrics (when available) carry extra weight because
  // they are parsed rather than scanned.
  const score = Math.round(
    (treeShape * 0.2 + naming * namespaceStyle * 0.2 + props * 0.12 + brevity * 0.08 +
     lengthSimilarity * 0.15 + treeShape * 0.2 + namespaceStyle * 0.05) * 100,
  );

  return {
    score,
    metrics: {
      treeShape: Number((treeShape * 100).toFixed(1)),
      naming: Number((naming * 100).toFixed(1)),
      props: Number((props * 100).toFixed(1)),
      brevity: Number((brevity * 100).toFixed(1)),
      upstreamElements: upTags.length,
      leptosElements: lxTags.length,
      dottedUpstream: uniqDotted.length,
      dottedMatched: matched.length,
      viewFnCalls: viewFns,
      namespacedComponentTags: dottedCount,
      flattenedComponentTags: flatCount,
      namespaceStyle: Number((namespaceStyle * 100).toFixed(1)),
      propsStructs,
      cxCalls,
      upstreamLines: upLines,
      leptosLines: lxLines,
      upstreamMeanAttrs: Number(meanAttrsUp.toFixed(2)),
      leptosMeanAttrs: Number(meanAttrsLx.toFixed(2)),
      upstreamChars: upChars,
      leptosChars: lxChars,
      lineSimilarity: Number((lineSimilarity * 100).toFixed(1)),
      charSimilarity: Number((charSimilarity * 100).toFixed(1)),
      lengthSimilarity: Number((lengthSimilarity * 100).toFixed(1)),
    },
    findings,
  };
}

// ---------------------------------------------------------------- run

async function main() {
  if (!reachable(`${LEPTOS_BASE}/`)) {
    console.error(`FAIL: Leptos docs server unreachable at ${LEPTOS_BASE} — run: python3 ralph/scripts/serve-docs-app.py --port 3177`);
    return 2;
  }
  if (!reachable(`${UPSTREAM_BASE}/`)) {
    console.error(`NOTE: upstream React docs unreachable at ${UPSTREAM_BASE} — cannot compare snippet ergonomics without the reference.`);
    return 0;
  }

  fs.mkdirSync(OUT_DIR, { recursive: true });
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'sniperg-'));
  let chrome = null;
  try {
    const launched = await launchChrome([...FRUGAL_CHROME_FLAGS, '--window-size=1280,2400'], { tmpDir: tmp, port: PORT, waitMs: 45000 });
    chrome = launched.child;
    const tabs = await fetch(`http://127.0.0.1:${PORT}/json/list`).then((r) => r.json());
    const ws = new WebSocket(tabs[0].webSocketDebuggerUrl);
    await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
    let id = 0;
    const pending = new Map();
    ws.addEventListener('message', (ev) => {
      const m = JSON.parse(ev.data);
      if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); }
    });
    const send = (method, params = {}) => new Promise((res) => {
      const i = ++id; pending.set(i, res); ws.send(JSON.stringify({ id: i, method, params }));
      setTimeout(() => { if (pending.has(i)) { pending.delete(i); res({ error: { message: 'timeout' } }); } }, 60000);
    });
    await send('Page.enable');
    await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 2400, deviceScaleFactor: 1, mobile: false });

    const grab = async (url) => {
      await send('Page.navigate', { url });
      await new Promise((r) => setTimeout(r, 2500));
      for (let i = 0; i < 60; i++) {
        const t = await send('Runtime.evaluate', { expression: `document.readyState + '|' + (document.querySelector('h1') ? '1' : '0')`, returnByValue: true });
        if (t.result?.result?.value === 'complete|1') break;
        await new Promise((r) => setTimeout(r, 500));
      }
      await new Promise((r) => setTimeout(r, 1500));
      const r = await send('Runtime.evaluate', { expression: SNIPPET_PROBE, returnByValue: true });
      const v = r.result?.result?.value;
      return v ? JSON.parse(v) : [];
    };

    const upTexts = await grab(`${UPSTREAM_BASE}/${route}`);
    const lxTexts = await grab(`${LEPTOS_BASE}/${route}`);

    // ---------------------------------------------------------------------------------------------
    // LANGUAGE GATE — the one thing this score must never reward.
    //
    // The metrics below measure how closely an example tracks upstream's SHAPE. Applied to a snippet
    // that IS upstream's code, that comparison is React-to-React: the copy matches itself perfectly and
    // a fidelity number would call the paste the ideal port. Asked for directly: "I hope the code
    // fidelity does not include react to react comparison because I did see react code in leptos site
    // example". It must not — so this is checked before scoring, and it is not a warn:
    //   * a block classified `react` (or ≥90% character-identical to an upstream block) is EXCLUDED from
    //     every positive metric — no credit, no partial credit;
    //   * it is a P0 finding, and the run FAILS even without --target, because no gate may pass while the
    //     port is teaching another framework's source;
    //   * the report states the count explicitly, so the number can never be misread as fidelity for a page
    //     that is actually a copy.
    // ---------------------------------------------------------------------------------------------
    const lang = classifyAll(lxTexts);
    // The ≥90% net answers "is this local block upstream's own source?", so the PERMITTED copy has to
    // be excluded from it: when the best-matching upstream block is itself LANGUAGE-NEUTRAL (`other` —
    // upstream's stylesheet, a shell command, a JSON blob), mirroring it verbatim is what
    // `specs/docs-content/CONTRACT.md` requirement 1 asks for ("a snippet may legitimately be
    // language-neutral ... those are `other` and are fine"), not a framework copy. Measured
    // 2026-09-16: `react/components/avatar` read `2 leptos / 0 react / 1 other` — the `other` block is
    // upstream's CSS, character-identical by design — and this check raised a P0 that called the page
    // "another framework's source" and FAILED the run, which no translation could ever clear without
    // DELETING a block upstream shows. The net keeps every other case: a local block ≥90% identical to
    // an upstream block whose own class is `react` (upstream's code) still counts, and so does a local
    // block classified `react` outright.
    const upClasses = upTexts.map((u) => classifySnippet(u));
    const verbatim = lxTexts
      .map((t) => {
        let best = { ratio: 0, upCls: null };
        upTexts.forEach((u, i) => {
          const r = verbatimRatio(t, u);
          if (r > best.ratio) best = { ratio: r, upCls: upClasses[i] };
        });
        return { t, ratio: best.ratio, cls: classifySnippet(t), upCls: best.upCls };
      })
      .filter((x) => x.cls !== 'leptos' && x.ratio >= 0.9);
    // A permitted copy is still EXCLUDED from the positive metrics below (the point of the exclusion
    // is that a copy matching itself scores nothing, and that applies to upstream's stylesheet as much
    // as to its JSX) — it is only kept OUT of the accusation. The two halves differ exactly there.
    const reactToReact = lang.react + verbatim.filter((v) => v.cls !== 'react' && v.upCls !== 'other').length;
    const scoringTexts = lxTexts.filter((t) => classifySnippet(t) !== 'react' && !verbatim.some((v) => v.t === t));
    const result = comparePage(upTexts, scoringTexts);
    result.metrics.snippetLanguages = lang;
    result.metrics.reactToReactBlocks = reactToReact;
    result.metrics.blocksExcludedFromScoring = lxTexts.length - scoringTexts.length;
    if (reactToReact > 0) {
      result.findings.unshift({
        severity: 'P0',
        area: 'react-to-react comparison (not fidelity)',
        gap: `${reactToReact} of ${lxTexts.length} snippet block(s) on this page are upstream's own React code (or ≥90% character-identical to it). React-vs-React is excluded from every metric here — a copy matching itself is not a port.`,
        fix: 'translate these blocks to this port\'s API in view! markup (imports from `leptos_ui`, `<Component::Part />` elements, attributes as props). Do not score the page until `react: 0`.',
      });
    }

    // Precision layer: real ASTs instead of the regex signature, when the pinned grammar pair is
    // installed (see ralph/scripts/lib/ast-compare.mjs for the version contract).
    if (astAvailable()) {
      try {
        const { Parser, js, html } = await loadGrammars();
        const jsParser = new Parser(); jsParser.setLanguage(js);
        const htmlParser = new Parser(); htmlParser.setLanguage(html);
        const upTree = upTexts.flatMap((t) => reactElementTree(jsParser, js, t));
        // scoringTexts, NOT lxTexts: excluded (React/verbatim) blocks must contribute nothing anywhere.
        // Shipping lxTexts here let a page's React copy feed the AST metrics — measured on avatar, whose
        // score rose to 41/100 with `AST shape 63.6%, naming 100%` while every block was excluded from the
        // score: the leak this gate exists to close, found by testing the gate itself.
        const lxTree = scoringTexts.flatMap((t) => leptosElementTree(htmlParser, html, t));
        const ast = compareTrees(upTree, lxTree);
        result.metrics.astShape = Number((ast.shape * 100).toFixed(1));
        result.metrics.astNaming = Number((ast.naming * 100).toFixed(1));
        result.metrics.astUpNodes = ast.upNodes;
        result.metrics.astLxNodes = ast.lxNodes;
        result.metrics.astUpMaxDepth = ast.upMaxDepth;
        result.metrics.astLxMaxDepth = ast.lxMaxDepth;
        result.metrics.astUpAttributes = ast.upAttributes;
        result.metrics.astLxAttributes = ast.lxAttributes;
        result.ast = ast;
        // rescore with the parsed metrics folded in
        result.score = Math.round(
          (result.metrics.treeShape / 100 * 0.25 + ast.naming * 0.2 + result.metrics.props / 100 * 0.12 +
           result.metrics.brevity / 100 * 0.08 + result.metrics.lengthSimilarity / 100 * 0.15 + ast.shape * 0.2) * 100,
        );
        if (ast.missingComponents.length) {
          result.findings.unshift({
            severity: 'P0',
            area: 'namespaced components (AST)',
            gap: `parsed ASTs: upstream teaches ${[...new Set(upTree.filter((e) => e.name && e.name.includes('.')).map((e) => e.name))].join(', ') || 'none'}; these have no counterpart node here: ${ast.missingComponents.join(', ')}`,
            fix: 'expose the parts as namespaced components usable as view! markup (e.g. `mod ui { pub fn Button() -> impl IntoView }` used as `<ui::Button />`), so the port\'s examples parse to the same tree shape upstream teaches',
          });
        }
        if (ast.upMaxDepth !== ast.lxMaxDepth) {
          result.findings.push({
            severity: 'P2',
            area: 'nesting depth (AST)',
            gap: `deepest node depth: upstream ${ast.upMaxDepth}, port ${ast.lxMaxDepth}`,
            fix: 'mirror the example nesting upstream shows',
          });
        }
      } catch (e) {
        result.metrics.astError = String(e.message || e).slice(0, 160);
      }
    } else {
      result.metrics.astAvailable = false;
    }
    const name = route.split('/').pop();

    const md = [
      `# Snippet ergonomics — ${route}`,
      '',
      `Generated by \`ralph/scripts/snippet-ergonomics.mjs\` at ${new Date().toISOString()} (both apps rendered at 1280px).`,
      '',
      `**Ergonomics score: ${result.score}/100**${target === null ? '' : ` (target ${target})`}`,
      '',
      '| metric | value |',
      '| --- | --- |',
      ...Object.entries(result.metrics).map(([k, v]) => `| ${k} | ${v} |`),
      '',
      '## How to close the gap (highest severity first)',
      '',
      ...(result.findings.length
        ? result.findings.flatMap((f) => [`- **${f.severity} · ${f.area}** — ${f.gap}`, `  - fix: ${f.fix}`])
        : ['No ergonomic gaps detected by these probes.']),
      '',
    ].join('\n');

    fs.writeFileSync(path.join(OUT_DIR, `${name}-snippets.md`), md);
    fs.writeFileSync(path.join(OUT_DIR, `${name}-snippets.json`), `${JSON.stringify({ route, generatedAt: new Date().toISOString(), ...result, upstreamSnippets: upTexts.length, leptosSnippets: lxTexts.length }, null, 1)}\n`);

        console.log(`snippet ergonomics: ${route} — ${result.score}/100` +
      `${target === null ? '' : ` (target ${target})`} | length ${result.metrics.lengthSimilarity}% tree ${result.metrics.treeShape}% naming ${result.metrics.naming}% props ${result.metrics.props}% brevity ${result.metrics.brevity}%`);
    if (result.metrics.astShape !== undefined) {
      console.log(`  AST: shape ${result.metrics.astShape}% naming ${result.metrics.astNaming}% | upstream ${result.metrics.astUpNodes} nodes (depth ${result.metrics.astUpMaxDepth}, ${result.metrics.astUpAttributes} attrs) vs port ${result.metrics.astLxNodes} nodes (depth ${result.metrics.astLxMaxDepth}, ${result.metrics.astLxAttributes} attrs)`);
    }
    console.log(`  component spelling: ${result.metrics.namespacedComponentTags} namespaced (<A::B/>) vs ${result.metrics.flattenedComponentTags} flattened (<AB/>) — style ${result.metrics.namespaceStyle}%`);
    console.log(`  upstream ${result.metrics.upstreamElements} elements / ${result.metrics.upstreamLines} lines; port ${result.metrics.leptosElements} elements / ${result.metrics.leptosLines} lines` +
      `; dotted components ${result.metrics.dottedMatched}/${result.metrics.dottedUpstream} matched; raw view-fn calls ${result.metrics.viewFnCalls}; props structs ${result.metrics.propsStructs}`);
    for (const f of result.findings) console.log(`  ${f.severity} ${f.area}: ${f.gap}`);
    console.log(`\nreport: ralph/logs/visual/${name}-snippets.md`);

    const lengthOk = result.metrics.lengthSimilarity / 100 >= lengthFloor;
    if (!lengthOk) {
      console.log(`  length check: ${result.metrics.lengthSimilarity}% vs the ${(lengthFloor * 100).toFixed(0)}% bar — FAIL`);
    }
    if (result.metrics.reactToReactBlocks > 0) {
      console.error(`\nFAIL: ${result.metrics.reactToReactBlocks} snippet block(s) are upstream's React code — the page teaches another framework, and a React-to-React comparison scores nothing here. Translate them to this port's view! markup first.`);
      return 1;
    }
    if (!lengthOk || (target !== null && result.score < target)) {
      console.error(`\nFAIL: ergonomics ${result.score}/100` +
        `${target === null ? '' : ` (target ${target})`}, length similarity ${result.metrics.lengthSimilarity}%` +
        ` (bar ${(lengthFloor * 100).toFixed(0)}%).`);
      return 1;
    }
    console.log(`  length check: ${result.metrics.lengthSimilarity}% (bar ${(lengthFloor * 100).toFixed(0)}%) — ok`);
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
