#!/usr/bin/env node
/* eslint-disable no-console */

// check-react-mentions.mjs — this port must SPEAK Leptos, not React.
//
// WHY THIS EXISTS
// ---------------
// Every other gate accepts a page that is structurally perfect, visually close and full of prose while
// its text still says "React". The port's whole claim is that a reader of these docs learns to build
// with the Leptos port — so a mention of React is either a deliberate, reasoned reference to the
// upstream library, or a leak of upstream's own text into a page that is supposed to teach this
// framework. Nothing distinguished those two until now, and "the docs check passes" was reported on
// pages whose install line pointed at somebody else's npm package.
//
// The rule applied here:
//   * the package this port tells people to install is `base-ui-leptos` (locally mapped — see
//     packages/leptos/package.json; not published). Upstream's `@base-ui/react` and any npm link to it
//     are ALWAYS a defect, because they send the reader to a different library.
//   * React APIs and JSX in prose or snippets (`useState`, `forwardRef`, `ReactElement`, `props.children`,
//     `from 'react'`, `JSX`, react.dev) are a defect: this port's equivalents are signals, `#[component]`
//     props and `view!` markup.
//   * crediting the original work is ALLOWED ("it is okay to reference the original work of React, but
//     not in the Rust codebase that is generated"): provenance/credit sentences are classified
//     `attribution` and counted, not failed. Attribution-shaped matches are checked FIRST so React code
//     can never be excused as a mention.
//   * any other bare "React" is a WARN: reword it to Leptos, or record it in the page's allow file with
//     a reason so the decision is made once instead of re-litigated every iteration.
//
// USAGE
//   node ralph/scripts/check-react-mentions.mjs --route react/components/checkbox
//   node ralph/scripts/check-react-mentions.mjs --all                 # every mirrored route AND the source scan
//   node ralph/scripts/check-react-mentions.mjs --source              # our Rust page sources only
//   node ralph/scripts/check-react-mentions.mjs --todo-id "<id>"       # route or `routes:` from the ledger
//   … --fail-on react-api,package-react   # exit code covers THESE classes only; every class is still
//                                         # printed, and the re-homed ones are named with their owner
//
// Exit: 0 clean (allowing listed references), 1 violations, 2 could not measure (never a pass),
// 3 permission/IO problem.
//
// CLASSES: `react-api` (a React API where this port uses Leptos — the API-type-column class),
// `package-react` (an install reference, or prose/link pointing at upstream's package or site), and
// `snippet-react` (a React package inside a mirrored EXAMPLE block — snippet LANGUAGE, owned by the
// `docs-chrome: snippet translation` items and measured per route by visual-gap-report's `react > 0`
// P0, check-page's snippet-language axis and snippetLanguage purity). The split exists so a gate can
// hold an item to the axis its own done-when claims without hiding the rest; see
// ralph/scripts/lib/source-scope.mjs.
//
// ALLOW FILES: specs/docs-content/<name>/react-allow.json
//   { "allow": [ { "match": "Ported from the React implementation of Base UI.", "reason": "provenance" } ] }
// A matching string must be present for each WARN to be tolerated; a bare "allow all" is not supported,
// because that is how a leak becomes permanent.

import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { launchChrome, killChrome, FRUGAL_CHROME_FLAGS, isResourceFailure, taskPressure, openHarnessTab } from './lib/browser.mjs';
import { cfgTestRanges, inRanges, INSTALL_COMMAND_RE, codeBlockRanges, codeBlockAt, isSnippetLanguageHit, propsChildrenIsRustFieldAccess } from './lib/source-scope.mjs';
import { refuseBrowserWork } from './lib/browser-budget.mjs';

// THE GUARD GOES BEFORE ANY BROWSER CODE — this is the fix for a real incident, not tidiness. The first version of
// this check sat near the END of the file while launchChrome() is called ~200 lines earlier, so a run could start
// chromium, do its work, and only then be refused: it left 8 browser processes resident for 19 minutes (~700 MB of a
// 4 GB cgroup), with the node parent still alive. A guard placed after the thing it guards is not a guard. The
// rendered modes (--all, --route) need a browser, so they are refused HERE, before the launch, and only when the
// allowance is absent; --source is pure text and always allowed.
if (process.argv.includes('--all') || process.argv.includes('--route')) {
  refuseBrowserWork('check-react-mentions.mjs', "node ralph/scripts/check-react-mentions.mjs --source  (cheap, no browser) and the rendered 'react mentions' axis in the CI scorecard");
}

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const OUT_DIR = path.join(PROJECT_ROOT, 'ralph/logs/visual');
// `--no-report`: scan without writing. A per-route ADVISORY scan must not overwrite the SITE-WIDE rendered report
// that `mentions-rendered-evidence.mjs` reads as evidence for the docs-copy lane — one `--route` run would turn that
// record into a single-route file and silently invalidate every claim resting on it (see run-regression.sh).
const NO_REPORT = process.argv.includes('--no-report');
const LEPTOS_BASE = process.env.LEPTOS_DOCS_BASE || 'http://127.0.0.1:3177';
const PORT = 9896;
const PACKAGE_ALIAS = 'base-ui-leptos';

function arg(name, dflt) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 ? (process.argv[i + 1] && !process.argv[i + 1].startsWith('--') ? process.argv[i + 1] : true) : dflt;
}

// ---- classification ---------------------------------------------------------------------------
// Ordered: the first class that matches a line wins, so a package the reader is told to install is
// never excused as "just a mention".
// RULE ORDER MATTERS. The first fail-class that matches wins, so React CODE is never excused as
// "just a mention"; attribution is checked FIRST, because crediting the original work is explicitly
// allowed — the user's rule is "it is okay to reference the original work of React, but not in the
// Rust codebase that is generated". So: a provenance/credit sentence is fine; React APIs, upstream
// package imports and install instructions in the port's own (generated) code are not.
const ATTRIBUTION_RE = /ported from|porting note|based on|upstream(?:'s)? (?:react|implementation|library)|the react implementation|react version of|base ??ui for react|original react (?:work|implementation|docs)|credit|acknowledge|thanks to|mirrors the react|same as react/i;

const RULES = [
  { cls: 'attribution', severity: 'ok', re: ATTRIBUTION_RE, why: 'a credited reference to the original React work — allowed' },
  { cls: 'package-react', severity: 'fail', re: /@base-ui\/react|@mui\/[a-z-]+|npmjs\.com\/package\/@base-ui|reactjs\.org|react\.dev|\bfrom\s+['"]react['"]|require\(['"]react['"]\)/i, why: 'ships upstream\'s package/import inside the generated port — the reader must be pointed at ' + PACKAGE_ALIAS },
  { cls: 'react-api', severity: 'fail', re: /\b(useState|useEffect|useRef|useMemo|useCallback|useReducer|forwardRef|createContext|ReactDOM|createRoot|React\.[A-Z]\w*|HTMLProps|props\.children|JSXElement|JSX)\b/, why: 'a React API where this port uses Leptos (signals, props, view!)' },
  { cls: 'package-ours', severity: 'ok', re: new RegExp(PACKAGE_ALIAS.replace(/[/@.]/g, (m) => '\\' + m)), why: 'the port\'s own package name — expected' },
  // THE BARE WORD, not a fragment of an identifier. `\breact\b` matches inside `floating-ui-react`
  // (a hyphen is a word boundary), so the upstream package/unit NAME was reported as an unreviewed
  // "mention of the word React" — measured 2026-09-16 on `status_data.rs`'s component list, where the
  // unit is named, not the framework discussed. The lookaround requires the word to stand alone, which
  // is what the class claims to find; a leak that actually ships React is caught by the two FAIL rules
  // above (`@base-ui/react`, `from 'react'`, `react.dev`) and never depended on this WARN rule.
  { cls: 'react-word', severity: 'warn', re: /(?<![\w-])react(?![\w-])/i, why: 'the bare word "React" — allowed only as a recorded, reasoned reference to upstream' },
];

export function classifyLine(line) {
  const out = [];
  let attribution = false;
  for (const r of RULES) {
    if (r.re.test(line)) {
      if (r.cls === 'attribution') attribution = true;
      // A credited reference is PERMITTED (CONTRACT.md requirement 6: "the bare word React is permitted
      // only as a recorded reference to the upstream library (provenance, a migration note, 'ported
      // from')"), so a line already classified as attribution must not ALSO be reported as an
      // unreviewed bare mention — that is the same sentence counted twice, once as allowed and once as
      // a warning, and it made every provenance sentence in the tree look like an open decision
      // (measured: `install_ref.rs`'s PROVENANCE, and both lines of form_page.rs's upstream-reference
      // paragraph). This is the script's own documented rule ("any OTHER bare React is a WARN") and the
      // classifier's own stated order ("attribution is checked FIRST"). Safety: it can only suppress a
      // WARN — the FAIL rules are untouched (a line that both credits upstream and ships a React API
      // still fails, pinned by the self-test), and an allow-file entry can only ever excuse a warn.
      if (r.cls === 'react-word' && attribution) continue;
      out.push(r);
      if (r.severity === 'fail') break;   // ok/warn do not stop the scan
    }
  }
  return out;
}

function allowListFor(name) {
  const p = path.join(PROJECT_ROOT, 'specs/docs-content', name, 'react-allow.json');
  if (!fs.existsSync(p)) return [];
  try {
    const j = JSON.parse(fs.readFileSync(p, 'utf8'));
    return Array.isArray(j.allow) ? j.allow : [];
  } catch {
    return [];
  }
}

function appAllowList() {
  const p = path.join(PROJECT_ROOT, 'specs/docs-app/react-allow.json');
  if (!fs.existsSync(p)) return [];
  try {
    const j = JSON.parse(fs.readFileSync(p, 'utf8'));
    return Array.isArray(j.allow) ? j.allow : [];
  } catch {
    return [];
  }
}

// Which page's allow file governs a SOURCE file. This used to be `path.basename(path.dirname(f))`, which
// for every page source (`crates/docs-app/src/pages/<name>_page.rs`) resolves to the literal string
// "pages" — a directory that is not a docs-content entry, so the lookup returned an empty list for every
// page on the site. The consequence, measured 2026-09-16: `specs/docs-content/otp-field/react-allow.json`
// existed, listed the exact sentence, and the source scan still reported that sentence as an unreviewed
// React mention — the decision was on the record and the gate could not read it, so this item's own
// done-when clause ("every tolerated mention listed with a reason in specs/docs-content/<name>/react-allow.json")
// was unreachable by construction. The rendered scan (which keys off the route basename) had always read
// the right file, so the two scans disagreed about the same page.
//
// Ownership is decided by what the file IS, not by where it sits: a `pages/<name>_page.rs` file keys its
// mirrored page (`snake → kebab`, so `otp_field_page.rs` → `otp-field`, matching the docs-content
// directory and the route basename) WHEN THAT PAGE EXISTS — and otherwise falls through to the app-level
// `specs/docs-app/react-allow.json`, because `status_page.rs` is the app's own /status report, not a
// mirror of any upstream page. Every other file under `crates/docs-app/src` (the chrome: `install_ref.rs`,
// `reference.rs`, `status_data.rs`) is app-level for the same reason. An allow entry can only ever excuse
// a WARN — the FAIL classes are evaluated before this list is consulted.
function allowListForSourceFile(rel) {
  const page = rel.match(/^crates\/docs-app\/src\/pages\/(.+)_page\.rs$/);
  if (page) {
    const name = page[1].replace(/_/g, '-');
    if (fs.existsSync(path.join(PROJECT_ROOT, 'specs/docs-content', name))) return allowListFor(name);
  }
  return appAllowList();
}

function isAllowed(line, allow) {
  return allow.some((a) => typeof a?.match === 'string' && a.match.length > 0 && line.includes(a.match) && typeof a?.reason === 'string' && a.reason.trim().length > 0);
}

// ---- self-test: the classifier, both directions ----------------------------------------------
// The repo's rule for instruments (see ralph/scripts/gate-selftest.mjs): a gate contract gets a
// self-test with BOTH directions — a violating line must be flagged, a permitted one must not —
// because an invariant that cannot fail is not an invariant, and a gate that cannot be read is worse
// than no gate. This classifier has now bitten twice, both times as a false positive that made a
// decided question look open (a package NAME's own hyphenated identifier read as the framework word;
// an approved attribution re-reported as an unreviewed mention), so its controls ship with it.
const CLASSIFIER_FIXTURES = [
  { id: 'ships-upstreams-package', line: "import { Checkbox } from '@base-ui/react/checkbox';", fail: ['package-react'] },
  { id: 'react-hook-in-prose', line: 'const [checked, setChecked] = useState(defaultChecked);', fail: ['react-api'] },
  { id: 'default-react-import', line: "import { useState } from 'react';", fail: ['package-react'] },
  { id: 'jsx-in-a-snippet', line: '<Checkbox.Root render={<span />}>JSX</Checkbox.Root>', fail: ['react-api'] },
  // An approved credit NEVER excuses a defect — the fail classes do not consult the attribution rule.
  { id: 'credit-cannot-excuse-a-hook', line: 'Ported from the React implementation; this used useState for that.', fail: ['react-api'], ok: ['attribution'] },
  // Permitted: a reasoned reference to upstream, and it must not ALSO count as an open question.
  { id: 'provenance-line-is-not-a-warn', line: 'Ported from the React implementation of Base UI — the same behaviour, expressed with Leptos signals.', ok: ['attribution'], warn: [] },
  { id: 'upstream-reference-is-not-a-warn', line: "Upstream's React docs submit this demo through React DOM's useActionState.", ok: ['attribution'], warn: [] },
  // A package/unit NAME is an identifier, not a mention of the framework.
  { id: 'hyphenated-unit-name-is-not-a-warn', line: 'component: "floating-ui-react",', warn: [], fail: [] },
  // …but the bare word, wherever it stands alone, still is one.
  { id: 'bare-word-after-a-slash-warns', line: '<th>"snippets leptos/react"</th>', warn: ['react-word'] },
  { id: 'bare-word-in-prose-warns', line: 'The React props object is merged here.', warn: ['react-word'] },
];

function assertClassifierSane() {
  const problems = [];
  for (const fx of CLASSIFIER_FIXTURES) {
    const out = classifyLine(fx.line);
    const got = {
      fail: out.filter((c) => c.severity === 'fail').map((c) => c.cls),
      warn: out.filter((c) => c.severity === 'warn').map((c) => c.cls),
      ok: out.filter((c) => c.severity === 'ok').map((c) => c.cls),
    };
    for (const cls of fx.fail ?? []) if (!got.fail.includes(cls)) problems.push(`${fx.id}: expected a ${cls} FAIL, got fails [${got.fail}]`);
    for (const cls of fx.warn ?? []) if (!got.warn.includes(cls)) problems.push(`${fx.id}: expected a ${cls} WARN, got warns [${got.warn}]`);
    for (const cls of fx.ok ?? []) if (!got.ok.includes(cls)) problems.push(`${fx.id}: expected ${cls} to be ok, got [${got.ok}]`);
    if (Array.isArray(fx.warn) && fx.warn.length === 0 && got.warn.length) problems.push(`${fx.id}: expected NO warn, got [${got.warn}]`);
    if (Array.isArray(fx.fail) && fx.fail.length === 0 && got.fail.length) problems.push(`${fx.id}: expected NO fail, got [${got.fail}]`);
  }
  // The allow-file direction, which this item's own clause rests on: a warn is excused only by a
  // listed match that carries a reason. A blank reason and a non-matching line must both stay warned,
  // so an allow file can never become a blanket silence.
  const sentence = 'filled, or use `onValueComplete` to react to completion without submitting.';
  const listed = [{ match: 'to react to completion without submitting', reason: 'upstream sentence, mirrored verbatim' }];
  if (!isAllowed(sentence, listed)) problems.push('allow file: a matching entry with a reason must excuse the warn');
  if (isAllowed(sentence, [{ match: 'to react to completion without submitting', reason: '   ' }])) problems.push('allow file: a blank reason must NOT excuse a warn');
  if (isAllowed(sentence, [{ match: 'a phrase that is not there', reason: 'x' }])) problems.push('allow file: a non-matching entry must not excuse a warn');
  if (isAllowed('some unrelated prose', listed)) problems.push('allow file: an unrelated line must not be excused');
  return problems;
}

// ---- sources ----------------------------------------------------------------------------------
function sourcesFromArgs() {
  const route = arg('route', null);
  const todoId = arg('todo-id', null);
  const all = arg('all', false) || arg('source', false);
  if (route && route !== true) return [String(route)];
  if (todoId && todoId !== true) {
    const r = spawnSync('node', ['ralph/scripts/get-todo-field.mjs', String(todoId), 'routes'], { cwd: PROJECT_ROOT, encoding: 'utf8' });
    const list = (r.stdout || '').trim().split(',').map((s) => s.trim()).filter(Boolean);
    if (list.length) return list.map((s) => `react/${s}`);
    const m = String(todoId).match(/components\/([a-z0-9-]+)/);
    if (m) return [`react/components/${m[1]}`];
  }
  if (all) {
    const dirs = fs.readdirSync(path.join(PROJECT_ROOT, 'specs/docs-content'), { withFileTypes: true }).filter((d) => d.isDirectory()).map((d) => d.name);
    const rs = [];
    for (const d of dirs) {
      const p = path.join(PROJECT_ROOT, 'specs/docs-content', d, 'page.md');
      if (!fs.existsSync(p)) continue;
      const body = fs.readFileSync(p, 'utf8');
      const m = body.match(/react\/(?:components|utils)\/[a-z0-9-]+/);
      rs.push(m ? m[0] : `react/components/${d}`);
    }
    return [...new Set(rs)];
  }
  return [];
}

// ---- rendered scan ----------------------------------------------------------------------------
const PAGE_PROBE = `(() => {
  const main = document.querySelector('main') || document.body;
  const out = [];
  for (const el of main.querySelectorAll('p, li, h1, h2, h3, h4, td, th, blockquote, dd, dt, a, code, pre, figcaption')) {
    if (el.querySelector('p, li, h1, h2, h3, h4, td, th, blockquote, pre')) continue;   // leaf only
    const t = (el.textContent || '').replace(/\\s+/g, ' ').trim();
    if (!t) continue;
    out.push({ kind: el.tagName.toLowerCase(), text: t.slice(0, 400), href: el.tagName === 'A' ? (el.getAttribute('href') || '') : '' });
  }
  return JSON.stringify(out);
})()`;

async function scanRendered(routes) {
  fs.mkdirSync(OUT_DIR, { recursive: true });
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'reactcheck-'));
  let chrome = null;
  try {
    const launched = await launchChrome([...FRUGAL_CHROME_FLAGS, '--window-size=1280,2200'], { tmpDir: tmp, port: PORT, waitMs: 45000 });
    chrome = launched.child;
    const tab = await openHarnessTab(PORT);
    const ws = new WebSocket(tab.webSocketDebuggerUrl);
    await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
    let id = 0; const pending = new Map();
    ws.addEventListener('message', (ev) => { const m = JSON.parse(ev.data); if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); } });
    const send = (method, params = {}) => new Promise((res) => { const i = ++id; pending.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); setTimeout(() => { if (pending.has(i)) { pending.delete(i); res({ error: { message: 'timeout' } }); } }, 60000); });
    await send('Page.enable');
    await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 2200, deviceScaleFactor: 1, mobile: false });

    const results = [];
    for (const route of routes) {
      await send('Page.navigate', { url: `${LEPTOS_BASE}/${route}` });
      await new Promise((r) => setTimeout(r, 2500));
      for (let i = 0; i < 60; i++) {
        const t = await send('Runtime.evaluate', { expression: `document.readyState + '|' + (document.querySelector('h1') ? '1' : '0')`, returnByValue: true });
        if (t.result?.result?.value === 'complete|1') break;
        await new Promise((r) => setTimeout(r, 500));
      }
      let prev = -1; let stable = 0; let leaves = [];
      for (let i = 0; i < 30; i++) {
        await new Promise((r) => setTimeout(r, 700));
        const r = await send('Runtime.evaluate', { expression: PAGE_PROBE, returnByValue: true });
        leaves = r.result?.result?.value ? JSON.parse(r.result.result.value) : [];
        if (leaves.length === prev && leaves.length > 0) { stable++; if (stable >= 2) break; } else stable = 0;
        prev = leaves.length;
      }
      const name = route.split('/').pop();
      const allow = allowListFor(name);
      const findings = { fail: [], warn: [], ok: 0, attribution: 0 };
      for (const leaf of leaves) {
        // The port mirrors upstream's URL shape on purpose (`/react/components/checkbox`), so a link's
        // PATH must never count as a React mention — only what the reader SEES, plus the href tested
        // against the package rule alone (an npm link IS a defect; a route path is not).
        const line = leaf.text;
        const classes = classifyLine(line);
        if (leaf.href && /@base-ui\/react|npmjs\.com\/package\/|react\.dev|reactjs\.org|from=['"]react['"]/i.test(leaf.href)) {
          classes.push({ cls: 'package-react', severity: 'fail', why: 'link target points at upstream\'s package/site instead of ' + PACKAGE_ALIAS });
        }
        // Snippet LANGUAGE vs page copy, the same split the source scan makes (see
        // ralph/scripts/lib/source-scope.mjs): a React import shown INSIDE a rendered code block is a
        // defect of the snippet's language — measured per route by visual-gap-report's `react > 0` P0,
        // check-page's snippet-language axis and snippetLanguage purity, and owned by the
        // `docs-chrome: snippet translation` items. An install COMMAND is never excused, fenced or not.
        const renderedSnippet = leaf.kind === 'pre' || leaf.kind === 'code';
        for (const c of classes) {
          if (c.cls === 'package-react' && renderedSnippet && !INSTALL_COMMAND_RE.test(line)) {
            c.cls = 'snippet-react';
            c.why = 'a React package inside a rendered code block — snippet LANGUAGE, owned by the `docs-chrome: snippet translation` items (their done-when is per-route `visual-gap-report … react=0`)';
          }
          // `props.children` inside a code leaf that reads as Rust is this port's own field access,
          // not an upstream API (see ralph/scripts/lib/source-scope.mjs).
          if (c.cls === 'react-api' && renderedSnippet && propsChildrenIsRustFieldAccess(line, { renderedCodeText: line })) {
            classes.splice(classes.indexOf(c), 1);
          }
        }
        for (const c of classes) {
          if (c.severity === 'ok') { if (c.cls === 'attribution') findings.attribution++; else findings.ok++; continue; }
          if (c.severity === 'warn' && isAllowed(line, allow)) { continue; }
          findings[c.severity].push({ cls: c.cls, why: c.why, text: line.slice(0, 240) });
        }
      }
      results.push({ route, name, leaves: leaves.length, allowEntries: allow.length, ...findings, measured: leaves.length > 0 });
    }
    return results;
  } finally {
    killChrome(chrome);
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

// ---- source scan (catches content that is not rendered yet) ------------------------------------
// SCOPE — reader-facing content only. The first version of this scan swept every .rs/.md under
// crates/ and specs/ and reported 483 files with 29,069 "warnings", almost all of them from
// specs/library/*/implementation.md: those are REVERSE-ENGINEERING NOTES whose whole purpose is to
// describe upstream's React source (`React.useState<ImageLoadingStatus>`, cited file paths, JSX) while
// the port is rebuilt. A reader never sees them, and a gate that fails on them is a gate nobody can
// satisfy. So: the port's own page sources, and the mirrored page specs minus their React↔Rust mapping
// tables (where naming React is the point of the table).
//
// TWO FURTHER CARVE-OUTS, both the SAME scope question and both measured 2026-09-16 (see
// ralph/scripts/lib/source-scope.mjs, which holds the shared answer):
//   * `#[cfg(test)]` items are not page copy. The `snippet_language_guard` modules keep verbatim
//     upstream snippets as POSITIVE CONTROLS, so their "every snippet teaches the port" assertions
//     cannot pass vacuously; this scan excluded `*_test.rs` files by NAME and simply never carried
//     the same intent to the inline `mod` form those guards actually use (10 of this item's own
//     library's 58 defects were such fixtures).
//   * `crates/docs-app/src/snippet_language.rs` IS the shared classifier — it lists upstream's
//     markers (`has("@base-ui/react")`) because detecting them is its job.
// Neither carve-out touches a reader-visible string, and the rendered mode below (`--all`) still
// reads every route's actual DOM, so nothing here becomes unfalsifiable.
const INSTRUMENT_FILES = new Set(['crates/docs-app/src/snippet_language.rs']);

function scanSource() {
  const targets = [];
  const walk = (dir, filter) => {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, e.name);
      if (e.isDirectory()) walk(p, filter);
      else if (filter(e.name)) targets.push(p);
    }
  };
  const appSrc = path.join(PROJECT_ROOT, 'crates/docs-app/src');
  if (fs.existsSync(appSrc)) walk(appSrc, (n) => n.endsWith('.rs') && !/_test\.rs$|^render_test/.test(n));
  // NOT scanned: specs/docs-content/*/page.md and specs/library/**. Both are MIRROR ANALYSES — they
  // document what upstream's page and source do (`React.useMemo(..., [users])`, `imports Menubar from
  // '@base-ui/react/menubar'`, cited .tsx paths) precisely so the port can be rebuilt. Scanning them
  // produced ~29k warnings about the subject being studied. The question this gate asks is what the
  // PORT teaches, so its scope is the port: crates/docs-app/src (its page content and snippet data),
  // plus the rendered page itself in rendered mode.

  const perFile = [];
  for (const f of targets) {
    const rel = path.relative(PROJECT_ROOT, f);
    if (INSTRUMENT_FILES.has(rel)) continue;
    const allow = allowListForSourceFile(rel);
    const findings = { fail: [], warn: [] };
    const body = fs.readFileSync(f, 'utf8');
    const testRanges = cfgTestRanges(body);
    const snippetRanges = codeBlockRanges(body);
    const lines = body.split('\n');
    lines.forEach((raw, i) => {
      const line = raw.trim();
      if (!line || line.startsWith('//')) return;                     // comments are not page copy
      if (inRanges(testRanges, i + 1)) return;                        // `#[cfg(test)]` fixtures are not page copy
      // Mapping-table rows name React deliberately (React A.B ↔ Rust A::B); upstream citations quote
      // upstream's own file paths and APIs. Neither is page copy the reader is taught from.
      if (line.startsWith('|') || /packages\/react\/|packages\/utils\//.test(line)) return;
      // A mirrored route path is not a React mention: /react/components/field is this docs site's own
      // namespace. Strip route-shaped tokens before the word rule sees them.
      const scannable = line.replace(/\/react\/(?:components|utils|handbook)\/[a-z0-9-]+/g, '<route>');
      const classes = classifyLine(scannable);
      for (const c of classes) {
        if (c.severity === 'ok') continue;          // includes credited references to the original work
        // SNIPPET LANGUAGE vs page copy. A React package inside a mirrored EXAMPLE block is the
        // snippet-translation items' class, not "a page tells the reader to install upstream's
        // package": an install COMMAND is fatal wherever it appears (source-scope.mjs tests that
        // first), and the snippet class is measured per route by visual-gap-report's `react > 0`
        // P0, check-page's snippet-language axis and snippetLanguage purity. It stays a FAIL here
        // and is printed in full — the class is re-homed, never hidden.
        if (c.cls === 'package-react' && isSnippetLanguageHit(line, i + 1, snippetRanges)) {
          findings.fail.push({ cls: 'snippet-react', line: i + 1, text: line.slice(0, 200), why: 'a React package inside a mirrored EXAMPLE block — snippet LANGUAGE, owned by the `docs-chrome: snippet translation` items (their done-when is per-route `visual-gap-report … react=0`)' });
          continue;
        }
        // `props.children` is a React idiom in a JSX example and an ordinary struct-field access in
        // this port's own Rust (`let children = props.children;`). Only the former is a defect, and
        // the distinction is the block's declared Lang, not a guess about the words.
        if (c.cls === 'react-api' && propsChildrenIsRustFieldAccess(scannable, { block: codeBlockAt(snippetRanges, i + 1) })) continue;
        if (c.severity === 'warn' && isAllowed(scannable, allow)) continue;
        findings[c.severity].push({ cls: c.cls, line: i + 1, text: line.slice(0, 200), why: c.why });
      }
    });
    if (findings.fail.length || findings.warn.length) perFile.push({ file: rel, ...findings });
  }
  return perFile;
}

// ---- exit policy ------------------------------------------------------------------------------
// `--fail-on <classes>` decides which defect CLASSES make this process exit non-zero. It changes the
// exit code ONLY: every file, every class and every count is still printed, and the classes left out
// of the gate are named in the output with the item that owns them. That is the difference between
// re-homing a class and dropping it — see ralph/scripts/lib/source-scope.mjs for why the two gates
// that must exit 0 per-item need this, and `run-regression.sh` for which item claims which class.
// Default: every class fails (unchanged behaviour for any caller that does not pass the flag).
const ALL_FAIL_CLASSES = ['react-api', 'package-react', 'snippet-react'];
const failOnArg = arg('fail-on', null);
const GATED = failOnArg && failOnArg !== true ? String(failOnArg).split(',').map((s) => s.trim()).filter(Boolean) : ALL_FAIL_CLASSES;
const UNGATED = ALL_FAIL_CLASSES.filter((c) => !GATED.includes(c));

// ---- the RULER's fingerprint ---------------------------------------------------------------------------------
// A rendered verdict is only as current as the classifier that produced it, and `mentions-rendered-evidence.mjs`
// reads a rendered report written by an EARLIER run and must decide whether that record still speaks for this tree.
// File mtimes cannot answer that: moving an `if` statement invalidates nothing, while one changed character in a
// rule invalidates every verdict the record holds. So the RULES are hashed — the rule table, the attribution
// pattern, and the rendered extractor's own source — and the hash travels inside the report. A record whose
// fingerprint differs from today's is STALE and must be re-measured; a record that matches is evidence. This is
// what keeps the fallback path from being a loophole: it cannot quietly outlive the ruler that produced it.
// (The scope rules in `lib/source-scope.mjs` are verdict-changing too, and are tracked by that file's own history
// instead — it changes rarely, and when it does a re-measurement is genuinely warranted.)
export const RULER_FINGERPRINT = createHash('sha256').update(JSON.stringify({
  rules: RULES.map((r) => [r.cls, r.severity, String(r.re)]),
  attribution: String(ATTRIBUTION_RE),
  rendered: scanRendered.toString(),
})).digest('hex').slice(0, 12);
// Cheap, pure-text, and side-effect free: prints the fingerprint and stops before ANY scan or browser work, so a
// caller (run-regression.sh via mentions-rendered-evidence.mjs) can ask "is the ruler still the one that wrote
// that report?" without paying for a measurement it cannot afford.
if (process.argv.includes('--ruler-fingerprint')) {
  console.log(RULER_FINGERPRINT);
  process.exit(0);
}

// ---- main ------------------------------------------------------------------------------------
// The instrument tests itself before it reports anything: a classifier that has stopped telling
// "ships upstream's framework" apart from "credits upstream's framework" is the single most expensive
// failure this loop can have, because every number downstream of it reads green.
const selftestProblems = assertClassifierSane();
if (selftestProblems.length) {
  console.error('CLASSIFIER SELF-TEST FAILED — fix the instrument before reading any finding below:');
  for (const p of selftestProblems) console.error(`  - ${p}`);
  process.exit(1);
}
const wantVerbose = Boolean(arg('verbose', false));
if (wantVerbose) console.log(`classifier self-test: ok (${CLASSIFIER_FIXTURES.length} fixtures + 4 allow-file directions)`);
// `--all` means BOTH scopes. It used to short-circuit into the source scan alone, so the "on any
// rendered route" half its own documentation (and `docs-copy: Leptos-only mentions`'s done-when)
// relies on was never measured — the rendered branch below was reachable only via `--route` /
// `--todo-id`. A gate that cannot run is worse than no gate, because its silence reads as green.
const wantAll = Boolean(arg('all', false));
const wantSource = Boolean(arg('source', false)) || wantAll;
const routes = sourcesFromArgs();
let sourceGated = 0;


// A browser must not be started on this box without an explicit allowance (see lib/browser-budget.mjs).
if (wantSource) {
  const perFile = scanSource();
  const all = perFile.flatMap((f) => f.fail);
  const gated = all.filter((x) => GATED.includes(x.cls));
  const ungated = all.filter((x) => UNGATED.includes(x.cls));
  const warns = perFile.reduce((n, f) => n + f.warn.length, 0);
  console.log(`react-mentions (source scan): ${perFile.length} file(s) with findings — ${all.length} fail (${gated.length} gated: ${GATED.join(',')}), ${warns} warn`);
  for (const f of perFile.sort((a, b) => b.fail.length - a.fail.length).slice(0, 12)) {
    console.log(`  ${f.file}: fail ${f.fail.length}, warn ${f.warn.length}`);
    for (const x of f.fail.slice(0, 2)) console.log(`     [${x.cls}] L${x.line} ${x.text.slice(0, 130)}`);
  }
  if (ungated.length) {
    console.log(`\nRE-HOMED (${ungated.length} ${UNGATED.join('/')} defect(s)) — a FAIL class this caller does not gate, owned by another item:`);
    for (const x of ungated.slice(0, 20)) console.log(`   [${x.cls}] ${x.why}`);
  }
  const md = ['# React mentions — source scan', '', `Generated ${new Date().toISOString()} by check-react-mentions.mjs --source.`,
    '', 'Scope: the port\'s own reader-facing source — `crates/docs-app/src/**/*.rs`, test files excluded (`*_test.rs` and inline `#[cfg(test)]` items, which keep upstream snippets as positive controls), and the shared classifier module `snippet_language.rs` excluded because listing upstream\'s markers is its job. Mirror analyses (`specs/docs-content/*/page.md`, `specs/library/**`) are deliberately NOT scanned: they document upstream React by design.',
    '', `The package this port points readers at must be \`${PACKAGE_ALIAS}\`; React APIs in prose/snippets are defects.`,
    '', `Classes: \`react-api\` (a React API where this port uses Leptos — the type-column class), \`package-react\` (an install reference or prose pointing at upstream's package/site), \`snippet-react\` (a React package inside a mirrored EXAMPLE block — snippet LANGUAGE, owned by the \`docs-chrome: snippet translation\` items and measured per route by visual-gap-report / check-page / snippetLanguage purity).`,
    '', `This run gates: ${GATED.join(', ')}${UNGATED.length ? `; re-homed (printed, not gated here): ${UNGATED.join(', ')}` : ''}.`, '',
    ...perFile.map((f) => `## ${f.file}\n\n${[...f.fail, ...f.warn].map((x) => `- **${x.cls}** L${x.line}: ${x.text}`).join('\n')}\n`)]
    .join('\n');
  fs.mkdirSync(OUT_DIR, { recursive: true });
  if (!NO_REPORT) {
    fs.writeFileSync(path.join(OUT_DIR, 'react-mentions-source.md'), md);
    console.log(`report: ralph/logs/visual/react-mentions-source.md`);
  }
  sourceGated = gated.length;
  if (!wantAll) process.exit(gated.length > 0 ? 1 : 0);
}

if (!routes.length) {
  console.error('usage: --route react/components/<name> | --todo-id "<id>" | --all | --source');
  process.exit(3);
}

if (!/^2/.test(spawnSync('curl', ['-s', '-o', '/dev/null', '-w', '%{http_code}', '--max-time', '20', `${LEPTOS_BASE}/`], { encoding: 'utf8' }).stdout.trim())) {
  console.error(`FAIL: Leptos docs server unreachable at ${LEPTOS_BASE} — cannot measure, refusing to report clean.`);
  process.exit(2);
}

let results;
try {
  results = await scanRendered(routes);
} catch (e) {
  const p = taskPressure();
  console.error(`${isResourceFailure(e.message) ? 'UNMEASURABLE' : 'FAIL'}: ${e.message}${p ? ` (cgroup ${p.current}/${p.max})` : ''}`);
  process.exit(isResourceFailure(e.message) ? 2 : 1);
}

const lines = ['# React mentions — rendered pages', '', `Generated ${new Date().toISOString()} by check-react-mentions.mjs — ruler fingerprint ${RULER_FINGERPRINT}.`, '',
  `Rule: this port points readers at \`${PACKAGE_ALIAS}\`; React APIs in prose or snippets are defects; the bare word "React" is tolerated only when a page lists it in \`specs/docs-content/<name>/react-allow.json\` with a reason.`, ''];
let totalFail = 0; let totalWarn = 0; let totalGated = 0;
for (const r of results) {
  const unmeasured = !r.measured;
  const gatedFails = r.fail.filter((f) => GATED.includes(f.cls));
  const ungatedFails = r.fail.filter((f) => !GATED.includes(f.cls));
  console.log(`${unmeasured ? 'UNMEASURABLE' : gatedFails.length ? 'FAIL' : 'ok  '} ${r.route} — fail ${r.fail.length} (${gatedFails.length} gated), warn ${r.warn.length}, our-package mentions ${r.ok}, attributed references ${r.attribution}${r.allowEntries ? `, allow-file entries ${r.allowEntries}` : ''}${unmeasured ? ' (page did not render — not scored)' : ''}`);
  for (const f of gatedFails.slice(0, 4)) console.log(`     [${f.cls}] ${f.text.slice(0, 150)}`);
  for (const f of ungatedFails.slice(0, 6)) console.log(`     (re-homed) [${f.cls}] ${f.text.slice(0, 120)}`);
  for (const w of r.warn.slice(0, 2)) console.log(`     (warn) ${w.text.slice(0, 130)}`);
  totalFail += r.fail.length; totalWarn += r.warn.length; totalGated += gatedFails.length;
  lines.push(`## ${r.route}`, '', `fail ${r.fail.length} (${gatedFails.length} gated: ${GATED.join(', ')}), warn ${r.warn.length}, mentions of \`${PACKAGE_ALIAS}\`: ${r.ok}, attributed (credits to upstream): ${r.attribution}${unmeasured ? ' — PAGE DID NOT RENDER, not scored' : ''}`, '',
    ...[...r.fail.map((x) => `${GATED.includes(x.cls) ? 'FAIL' : 'RE-HOMED'} **${x.cls}** — ${x.why}\n  - \`${x.text}\``), ...r.warn.map((x) => `- warn **${x.cls}** — ${x.why}\n  - \`${x.text}\``)], '');
}
lines.splice(4, 0, `**Totals: ${totalFail} defect(s) — ${totalGated} gated (${GATED.join(', ')}), ${totalFail - totalGated} re-homed to another item (${UNGATED.join(', ') || 'none'}); ${totalWarn} tolerated-reference candidate(s) across ${results.length} route(s).**`, '');
lines.splice(5, 0, '', `Source scan this run: ${sourceGated} gated defect(s) (see react-mentions-source.md).`);
const totalLine = `totals: ${totalFail} defect(s) — ${totalGated} gated, ${totalFail - totalGated} re-homed; source scan ${sourceGated} gated; ${totalWarn} unchecked reference(s) across ${results.length} route(s)`;
if (!NO_REPORT) {
  fs.mkdirSync(OUT_DIR, { recursive: true });
  fs.writeFileSync(path.join(OUT_DIR, 'react-mentions.md'), lines.join('\n'));
}
console.log(`\n${totalLine}`);
if (!NO_REPORT) console.log('report: ralph/logs/visual/react-mentions.md');
process.exit((totalGated > 0 || sourceGated > 0) ? 1 : 0);
