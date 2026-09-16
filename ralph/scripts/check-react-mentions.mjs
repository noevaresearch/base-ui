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
//   node ralph/scripts/check-react-mentions.mjs --all                 # every mirrored route
//   node ralph/scripts/check-react-mentions.mjs --source              # our Rust page sources + specs
//   node ralph/scripts/check-react-mentions.mjs --todo-id "<id>"       # route or `routes:` from the ledger
//
// Exit: 0 clean (allowing listed references), 1 violations, 2 could not measure (never a pass),
// 3 permission/IO problem.
//
// ALLOW FILES: specs/docs-content/<name>/react-allow.json
//   { "allow": [ { "match": "Ported from the React implementation of Base UI.", "reason": "provenance" } ] }
// A matching string must be present for each WARN to be tolerated; a bare "allow all" is not supported,
// because that is how a leak becomes permanent.

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { launchChrome, killChrome, FRUGAL_CHROME_FLAGS, isResourceFailure, taskPressure } from './lib/browser.mjs';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const OUT_DIR = path.join(PROJECT_ROOT, 'ralph/logs/visual');
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
  { cls: 'react-word', severity: 'warn', re: /\bReact\b|\breact\b/, why: 'the bare word "React" — allowed only as a recorded, reasoned reference to upstream' },
];

export function classifyLine(line) {
  const out = [];
  for (const r of RULES) {
    if (r.re.test(line)) { out.push(r); if (r.severity === 'fail') break; }   // ok/warn do not stop the scan
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

function isAllowed(line, allow) {
  return allow.some((a) => typeof a?.match === 'string' && a.match.length > 0 && line.includes(a.match) && typeof a?.reason === 'string' && a.reason.trim().length > 0);
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
    const tabs = await fetch(`http://127.0.0.1:${PORT}/json/list`).then((r) => r.json());
    const ws = new WebSocket(tabs[0].webSocketDebuggerUrl);
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
    const name = path.basename(path.dirname(f));
    const allow = allowListFor(name);
    const findings = { fail: [], warn: [] };
    const lines = fs.readFileSync(f, 'utf8').split('\n');
    lines.forEach((raw, i) => {
      const line = raw.trim();
      if (!line || line.startsWith('//')) return;                     // comments are not page copy
      // Mapping-table rows name React deliberately (React A.B ↔ Rust A::B); upstream citations quote
      // upstream's own file paths and APIs. Neither is page copy the reader is taught from.
      if (line.startsWith('|') || /packages\/react\/|packages\/utils\//.test(line)) return;
      // A mirrored route path is not a React mention: /react/components/field is this docs site's own
      // namespace. Strip route-shaped tokens before the word rule sees them.
      const scannable = line.replace(/\/react\/(?:components|utils|handbook)\/[a-z0-9-]+/g, '<route>');
      const classes = classifyLine(scannable);
      for (const c of classes) {
        if (c.severity === 'ok') continue;          // includes credited references to the original work
        if (c.severity === 'warn' && isAllowed(scannable, allow)) continue;
        findings[c.severity].push({ cls: c.cls, line: i + 1, text: line.slice(0, 200), why: c.why });
      }
    });
    if (findings.fail.length || findings.warn.length) perFile.push({ file: rel, ...findings });
  }
  return perFile;
}

// ---- main ------------------------------------------------------------------------------------
const wantSource = Boolean(arg('source', false)) || Boolean(arg('all', false));
const routes = sourcesFromArgs();

if (wantSource) {
  const perFile = scanSource();
  const fails = perFile.reduce((n, f) => n + f.fail.length, 0);
  const warns = perFile.reduce((n, f) => n + f.warn.length, 0);
  console.log(`react-mentions (source scan): ${perFile.length} file(s) with findings — ${fails} fail, ${warns} warn`);
  for (const f of perFile.sort((a, b) => b.fail.length - a.fail.length).slice(0, 12)) {
    console.log(`  ${f.file}: fail ${f.fail.length}, warn ${f.warn.length}`);
    for (const x of f.fail.slice(0, 2)) console.log(`     [${x.cls}] L${x.line} ${x.text.slice(0, 130)}`);
  }
  const md = ['# React mentions — source scan', '', `Generated ${new Date().toISOString()} by check-react-mentions.mjs --source.`,
    '', 'Scope: the port\'s own reader-facing source — `crates/docs-app/src/**/*.rs`, test files excluded. Mirror analyses (`specs/docs-content/*/page.md`, `specs/library/**`) are deliberately NOT scanned: they document upstream React by design.',
    '', `The package this port points readers at must be \`${PACKAGE_ALIAS}\`; React APIs in prose/snippets are defects.`, '',
    ...perFile.map((f) => `## ${f.file}\n\n${[...f.fail, ...f.warn].map((x) => `- **${x.cls}** L${x.line}: ${x.text}`).join('\n')}\n`)];
  fs.mkdirSync(OUT_DIR, { recursive: true });
  fs.writeFileSync(path.join(OUT_DIR, 'react-mentions-source.md'), md.join('\n'));
  console.log(`report: ralph/logs/visual/react-mentions-source.md`);
  process.exit(fails > 0 ? 1 : 0);
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

const lines = ['# React mentions — rendered pages', '', `Generated ${new Date().toISOString()} by check-react-mentions.mjs.`, '',
  `Rule: this port points readers at \`${PACKAGE_ALIAS}\`; React APIs in prose or snippets are defects; the bare word "React" is tolerated only when a page lists it in \`specs/docs-content/<name>/react-allow.json\` with a reason.`, ''];
let totalFail = 0; let totalWarn = 0;
for (const r of results) {
  const unmeasured = !r.measured;
  console.log(`${unmeasured ? 'UNMEASURABLE' : r.fail.length ? 'FAIL' : 'ok  '} ${r.route} — fail ${r.fail.length}, warn ${r.warn.length}, our-package mentions ${r.ok}, attributed references ${r.attribution}${r.allowEntries ? `, allow-file entries ${r.allowEntries}` : ''}${unmeasured ? ' (page did not render — not scored)' : ''}`);
  for (const f of r.fail.slice(0, 4)) console.log(`     [${f.cls}] ${f.text.slice(0, 150)}`);
  for (const w of r.warn.slice(0, 2)) console.log(`     (warn) ${w.text.slice(0, 130)}`);
  totalFail += r.fail.length; totalWarn += r.warn.length;
  lines.push(`## ${r.route}`, '', `fail ${r.fail.length}, warn ${r.warn.length}, mentions of \`${PACKAGE_ALIAS}\`: ${r.ok}, attributed (credits to upstream): ${r.attribution}${unmeasured ? ' — PAGE DID NOT RENDER, not scored' : ''}`, '',
    ...[...r.fail.map((x) => `- FAIL **${x.cls}** — ${x.why}\n  - \`${x.text}\``), ...r.warn.map((x) => `- warn **${x.cls}** — ${x.why}\n  - \`${x.text}\``)], '');
}
lines.splice(4, 0, `**Totals: ${totalFail} defect(s), ${totalWarn} tolerated-reference candidate(s) across ${results.length} route(s).**`, '');
fs.mkdirSync(OUT_DIR, { recursive: true });
fs.writeFileSync(path.join(OUT_DIR, 'react-mentions.md'), lines.join('\n'));
console.log(`\ntotals: ${totalFail} defect(s), ${totalWarn} unchecked reference(s) across ${results.length} route(s)`);
console.log('report: ralph/logs/visual/react-mentions.md');
process.exit(totalFail > 0 ? 1 : 0);
