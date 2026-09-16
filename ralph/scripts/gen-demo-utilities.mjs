#!/usr/bin/env node
/* eslint-disable no-console */

// gen-demo-utilities.mjs — make the ported demos' Tailwind-variant class strings live.
//
// WHY
// ---
// Upstream's docs ship every demo twice: a `css-modules` variant (which its own pages render) and
// a `tailwind` variant that carries the same design as a utility class string. The port's pages
// copied the TAILWIND class strings verbatim — and this crate compiles no Tailwind, so they were
// inert. Measured consequences on 2026-09-16 (Chrome for Testing, 1280px): the checkbox demo's
// control laid out as a full-width block (768x41 against upstream's 150x20), the button rendered
// bare text (53x24 against upstream's bordered 72x32), `check-visual-budget.mjs` read 84.42%
// widget parity on button, and checkbox/meter were not comparable at all (region fault).
//
// WHAT IT DOES
// ------------
// Reads the utility class tokens the port's pages actually use, resolves each one against
// UPSTREAM'S OWN COMPILED STYLESHEET (the running docs site — the same oracle
// `check-visual-budget.mjs` measures against), and rewrites a marked, generated section at the
// foot of `crates/docs-app/style/main.css` with those rules verbatim: the utility declarations,
// the `@media`/`@supports` context each rule sits in, the `@property` registrations and fallback
// declarations the `--tw-*` variables need, and the theme custom properties the rules read.
// Nothing is invented and nothing is hand-tuned; re-running is idempotent.
//
// The section is emitted UNLAYERED and LAST, which is how a utility beats an earlier chrome rule
// of equal specificity — the precedence upstream gets from `@layer utilities` winning over its
// components layer.
//
// USAGE
//   node ralph/scripts/gen-demo-utilities.mjs            # both dev servers up (see README of check-visual-budget.mjs)
//   node ralph/scripts/gen-demo-utilities.mjs --check    # exit 1 if the section is stale

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..', '..');
const PAGES = path.join(ROOT, 'crates/docs-app/src/pages');
const MAIN_CSS = path.join(ROOT, 'crates/docs-app/style/main.css');
const UPSTREAM = process.env.UPSTREAM_DOCS_BASE || 'http://127.0.0.1:3005';
const BEGIN = '/* == DEMO UTILITIES (generated) — BEGIN == */';
const END = '/* == DEMO UTILITIES (generated) — END == */';

// A token is utility-shaped if it is lowercase and made of class-ish characters. Note the
// resolution step below is what actually selects the utilities: a token is emitted only if
// upstream's own compiled stylesheet defines a rule for it, so bare utilities (`border`, `flex`,
// `block`) come through without a hand-maintained keyword list and prose never does.
const TOKEN_RE = /^[a-z0-9][a-z0-9.:\-/_[\]%#(),]*$/;
// 4000, not 600: the demo class constants are long — `DEMO_BUTTON_CLASS` is 712 characters — and a
// lower cap silently SKIPPED the whole literal, which is how `disabled:*` and `data-disabled:*`
// came to be missing from the first generated section (caught by the drift guard in
// src/demo_styles.rs, which flagged 14 tokens the section did not define).
const STRING_RE = /"([^"\\\n]{1,4000})"/g;

function utilityTokens(src) {
  const out = [];
  for (const m of src.matchAll(STRING_RE)) {
    for (const t of m[1].split(/\s+/)) {
      if (t && TOKEN_RE.test(t)) out.push(t);
    }
  }
  return out;
}

// ── CSS parsing: an entry is (at-rule stack, declaration). The innermost non-at-rule entry on a
// declaration's stack is its selector.
function parseDeclarations(src) {
  src = src.replace(/\/\*[\s\S]*?\*\//g, '');
  const out = [];
  const stack = [];
  let buf = '';
  let i = 0;
  while (i < src.length) {
    const c = src[i];
    if (c === '{') {
      const prelude = buf.trim();
      buf = '';
      const atRule = prelude.startsWith('@');
      const nestable = /^@(media|supports|layer|container|scope|starting-style)\b/.test(prelude);
      if (atRule && !nestable) {
        // e.g. @property / @font-face: capture the whole block, keep it verbatim
        let depth = 1;
        let j = i + 1;
        while (j < src.length && depth > 0) {
          if (src[j] === '{') depth += 1;
          else if (src[j] === '}') depth -= 1;
          j += 1;
        }
        out.push({ stack: [...stack], block: { prelude, body: src.slice(i + 1, j - 1) } });
        i = j;
        buf = '';
        continue;
      }
      stack.push(prelude);
    } else if (c === '}') {
      buf = '';
      if (stack.length) stack.pop();
    } else if (c === ';') {
      if (buf.trim() && stack.length) out.push({ stack: [...stack], decl: buf.trim() });
      buf = '';
    } else {
      buf += c;
    }
    i += 1;
  }
  return out;
}

async function fetchUpstreamCss() {
  const pageUrl = `${UPSTREAM}/react/components/button`;
  const res = await fetch(pageUrl);
  if (!res.ok) throw new Error(`GET ${pageUrl} -> ${res.status} (is the upstream docs server up?)`);
  const html = await res.text();
  const hrefs = [...html.matchAll(/href="([^"]+\.css)"/g)].map((m) => m[1]).filter((h) => !h.startsWith('#'));
  const chunks = [];
  for (const href of [...new Set(hrefs)]) {
    const url = href.startsWith('http') ? href : UPSTREAM + href;
    const r = await fetch(url);
    if (!r.ok) throw new Error(`GET ${url} -> ${r.status}`);
    chunks.push({ url, css: await r.text() });
  }
  if (!chunks.length) throw new Error('no stylesheets found on the upstream page');
  return chunks;
}

function classTokensOf(selector) {
  return [...selector.matchAll(/\.((?:\\.|[A-Za-z0-9_-])+)/g)].map((m) => m[1].replace(/\\(.)/g, '$1'));
}

function varRefs(text) {
  return [...text.matchAll(/var\(\s*(--[a-z0-9-]+)/g)].map((m) => m[1]);
}

async function main() {
  const check = process.argv.includes('--check');
  const chunks = await fetchUpstreamCss();

  const themeVars = new Map();          // --color-neutral-950 -> value
  const propertyBlocks = new Map();     // --tw-border-style -> "@property { ... }" body
  const fallbackVars = new Map();       // --tw-border-style -> "solid"  (the @supports fallback)
  const rules = new Map();              // "ctx\u0000selector" -> declarations[]
  const byToken = new Map();            // class token -> [ruleKey]

  for (const { css } of chunks) {
    for (const entry of parseDeclarations(css)) {
      if (entry.block) {
        const name = entry.block.prelude.match(/^@property\s+(--[a-z0-9-]+)/);
        if (name) propertyBlocks.set(name[1], entry.block.body.trim());
        // Tailwind's fallback for browsers without @property: `*, :before, :after, ::backdrop { --tw-x: initial }`
        if (/^\*,\s*:before/.test(entry.block.prelude)) {
          for (const m of entry.block.body.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/g)) {
            if (!fallbackVars.has(m[1])) fallbackVars.set(m[1], m[2].trim());
          }
        }
        continue;
      }
      const decl = entry.decl;
      const owner = entry.stack[entry.stack.length - 1];
      if (!owner || owner.startsWith('@')) {
        const m = decl.match(/^(--[a-z0-9-]+)\s*:\s*(.+)$/);
        if (m) themeVars.set(m[1], m[2].trim());
        continue;
      }
      if (/^\*,/.test(owner)) {
        // Tailwind's fallback block for engines without @property.
        const m = decl.match(/^(--[a-z0-9-]+)\s*:\s*(.+)$/);
        if (m && !fallbackVars.has(m[1])) fallbackVars.set(m[1], m[2].trim());
        continue;
      }
      const m = decl.match(/^(--[a-z0-9-]+)\s*:\s*(.+)$/);
      if (m && entry.stack.includes('@layer theme')) {
        themeVars.set(m[1], m[2].trim());
        continue;
      }
      // the @layer wrappers are dropped: main.css is deliberately unlayered (its banner,
      // deviation 1), and the section is emitted last so utilities still win.
      const ctx = entry.stack.slice(0, -1).filter((at) => !at.startsWith('@layer'));
      const key = `${ctx.join('|')}\u0000${owner}`;
      if (!rules.has(key)) rules.set(key, []);
      rules.get(key).push(decl);
    }
  }

  for (const [key, decls] of rules) {
    const selector = key.split('\u0000')[1];
    for (const tok of classTokensOf(selector)) {
      if (!byToken.has(tok)) byToken.set(tok, []);
      byToken.get(tok).push(key);
    }
  }

  // ── the port's own tokens
  const tokens = new Set();
  for (const f of fs.readdirSync(PAGES).filter((f) => f.endsWith('.rs'))) {
    for (const t of utilityTokens(fs.readFileSync(path.join(PAGES, f), 'utf8'))) tokens.add(t);
  }

  const mainCss = fs.readFileSync(MAIN_CSS, 'utf8');
  // The previously generated section must not count as "already styled by main.css", or re-running
  // would see its own output and shrink to nothing.
  const prevStart = mainCss.indexOf(BEGIN);
  const prevEnd = mainCss.indexOf(END);
  const handWritten = prevStart >= 0 && prevEnd > prevStart
    ? mainCss.slice(0, prevStart) + mainCss.slice(prevEnd + END.length)
    : mainCss;
  const declaredSelectors = new Set([...handWritten.matchAll(/\.((?:\\.|[A-Za-z0-9_-])+)/g)]
    .map((m) => m[1].replace(/\\(.)/g, '$1')));

  const chosen = new Map();   // ruleKey -> decls
  const unresolved = [];
  const owned = [];
  for (const t of tokens) {
    if (declaredSelectors.has(t)) { owned.push(t); continue; }
    if (!byToken.has(t)) { unresolved.push(t); continue; }
    for (const key of byToken.get(t)) chosen.set(key, rules.get(key));
  }

  // ── theme custom properties the chosen rules read, transitively
  const needed = new Set();
  const queue = [];
  for (const decls of chosen.values()) queue.push(...varRefs(decls.join(' ')));
  while (queue.length) {
    const v = queue.pop();
    if (needed.has(v)) continue;
    needed.add(v);
    if (themeVars.has(v)) queue.push(...varRefs(themeVars.get(v)));
  }
  const twVars = [...needed].filter((v) => v.startsWith('--tw-'));

  const atCtx = new Map();
  for (const key of chosen.keys()) {
    const ctx = key.split('\u0000')[0];
    if (ctx) for (const at of ctx.split('|')) atCtx.set(at, (atCtx.get(at) || 0) + 1);
  }

  const L = [];
  L.push(BEGIN);
  L.push('/* ═══════════════════════════════════════════════════════════════════════════════════');
  L.push(' * DEMO UTILITIES — generated by ralph/scripts/gen-demo-utilities.mjs. Do not hand-edit.');
  L.push(' *');
  L.push(' * WHY: upstream ships every demo twice — a `css-modules` variant (which its pages');
  L.push(' * render) and a `tailwind` variant carrying the same design as a utility string. The');
  L.push(' * ported pages copied the TAILWIND strings verbatim and this crate compiles no');
  L.push(' * Tailwind, so they were inert: the checkbox demo\'s control laid out as a full-width');
  L.push(' * block (768x41 against upstream\'s 150x20), the button rendered bare text (53x24');
  L.push(' * against upstream\'s bordered 72x32), widget parity read 84.42% on button, and');
  L.push(' * checkbox/meter were not comparable at all.');
  L.push(' *');
  L.push(' * WHAT: the rules those class strings name, copied VERBATIM from upstream\'s own');
  L.push(` * compiled stylesheet (${UPSTREAM} — the same oracle`);
  L.push(' * ralph/scripts/check-visual-budget.mjs measures against): the utility declarations');
  L.push(' * with the @media/@supports context each sits in, the @property registrations and');
  L.push(' * fallback declarations its --tw-* variables need, and the theme custom properties');
  L.push(' * they read. Nothing here is invented or hand-tuned.');
  L.push(' *');
  L.push(' * REGENERATE: node ralph/scripts/gen-demo-utilities.mjs (needs the upstream dev server)');
  L.push(' *');
  L.push(' * CASCADE: emitted UNLAYERED and LAST, so a utility beats an earlier chrome rule of');
  L.push(' * equal specificity — the precedence upstream gets from `@layer utilities` winning');
  L.push(' * over its components layer.');
  L.push(` * SELECTORS: ${chosen.size} rule(s) over ${tokens.size} token(s) in the pages.`);
  L.push(' * ═══════════════════════════════════════════════════════════════════════════════════ */');
  L.push('');

  const definedHere = new Map([...themeVars].filter(([k]) => needed.has(k)));
  if (definedHere.size) {
    L.push('/* theme custom properties the rules below read (upstream docs/src/css/index.css @theme) */');
    L.push(':root {');
    for (const [k, v] of [...definedHere].sort()) L.push(`  ${k}: ${v};`);
    L.push('}');
    L.push('');
  }

  if (twVars.length) {
    L.push('/* Tailwind\'s registered custom properties, for the --tw-* variables used below */');
    for (const v of twVars) {
      if (!propertyBlocks.has(v)) continue;
      L.push(`@property ${v} {`);
      for (const line of propertyBlocks.get(v).split('\n')) L.push('  ' + line.trim());
      L.push('}');
    }
    const withFallback = twVars.filter((v) => fallbackVars.has(v));
    if (withFallback.length) {
      L.push('/* fallback for engines without @property (the same block upstream ships) */');
      L.push('*, :before, :after, ::backdrop {');
      for (const v of withFallback) L.push(`  ${v}: ${fallbackVars.get(v)};`);
      L.push('}');
    }
    L.push('');
  }

  for (const [key, decls] of chosen) {
    const [ctx, selector] = key.split('\u0000');
    const ats = ctx ? ctx.split('|') : [];
    for (const at of ats) L.push(`${at} {`);
    L.push(`${selector} {`);
    for (const d of decls) L.push(`  ${d};`);
    L.push('}');
    for (const _ of ats) L.push('}');
    L.push('');
  }
  L.push(END);
  const section = L.join('\n');

  const start = mainCss.indexOf(BEGIN);
  const end = mainCss.indexOf(END);
  const next = start >= 0 && end > start
    ? mainCss.slice(0, start) + section + mainCss.slice(end + END.length)
    : mainCss.replace(/\s*$/, '\n\n') + section + '\n';

  const suspicious = unresolved.filter((t) => t.includes('-') && !t.includes(':') && !/^(docs|api)-/.test(t) && !/[A-Z]/.test(t));
  console.log(`upstream chunks: ${chunks.length}, rules indexed: ${rules.size}, tokens in pages: ${tokens.size}`);
  console.log(`emitted: ${chosen.size} rule(s), ${definedHere.size} theme var(s), ${twVars.length} --tw-* var(s), ${section.length} bytes`);
  console.log(`at-rule contexts: ${[...atCtx].map(([k, v]) => `${k.slice(0, 40)}x${v}`).join(', ') || '(none)'}`);
  console.log(`already styled by main.css (${owned.length}): ${owned.slice(0, 12).join(' ')}${owned.length > 12 ? ' …' : ''}`);
  console.log(`unresolved-looking tokens (${suspicious.length}): ${suspicious.slice(0, 40).join(' ')}`);

  if (check) {
    if (next !== mainCss) {
      console.error('STALE: crates/docs-app/style/main.css is not what this generator produces.');
      process.exit(1);
    }
    console.log('up to date.');
    return;
  }
  if (next === mainCss) {
    console.log('main.css already up to date.');
    return;
  }
  fs.writeFileSync(MAIN_CSS, next);
  console.log(`wrote ${MAIN_CSS}`);
}

main().catch((err) => {
  console.error(err.message);
  process.exit(2);
});
