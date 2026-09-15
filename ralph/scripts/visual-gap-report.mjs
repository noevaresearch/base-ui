#!/usr/bin/env node
/* eslint-disable no-console */

// visual-gap-report.mjs — turn the fidelity score into an actionable work list.
//
// WHY THIS EXISTS
// ---------------
// `check-visual-budget.mjs` says a route scores 65/100. That is a verdict, not feedback: an
// iteration cannot act on "14.6% of pixels differ". This script renders the same route on
// both apps and reports *named, located* gaps — "upstream has a 260px sidebar with 38 nav
// links, this page has none", "upstream's code blocks carry 41 coloured tokens, these carry
// 0", "upstream renders 2 API tables, this renders 0" — plus the computed typography that
// differs and a pixel-diff mask image a vision-capable iteration can look at.
//
// It is the missing half of the parity loop: measure (check-visual-budget) -> **diagnose
// (this)** -> fix a named gap -> re-measure. Aimed at the ledger's Phase E target of >=90%
// parity per ported route.
//
// OUTPUT (written under ralph/logs/visual/)
// -----------------------------------------
//   <route>.md          — prioritised gap list with a suggested fix per gap (for the model)
//   <route>.json        — the same findings machine-readable
//   <route>-mask.png    — pixel-diff heatmap (white = differs from upstream)
//   <route>-sbs.png     — side-by-side composite (upstream top, ours bottom)
//
// USAGE
//   node ralph/scripts/visual-gap-report.mjs --route react/components/checkbox
//   node ralph/scripts/visual-gap-report.mjs --todo-id "docs-content: components/checkbox"
//
// Requires both dev servers (see check-visual-budget.mjs). Exits 0 always — it is a
// diagnosis tool, not a gate; a missing upstream render exits 2 with a NOTE.

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { decodePng, compare } from './lib/png.mjs';

// This box runs a 512-task cgroup cap shared with the Hermes gateway, the Ralph loop and cargo
// builds. A default Chrome launch is ~20 processes and 100+ threads, which was enough to make
// forks fail box-wide (`fork: retry: Resource temporarily unavailable`, and git's credential
// helper dying with newosproc/EAGAIN). Hence: single renderer process, no zygote, and a lock so
// only ONE harness browser is ever alive.
const BROWSER_LOCK = '/tmp/ralph-browser-harness.lock';
function acquireBrowserLock(timeoutMs = 180000) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    try {
      const fd = fs.openSync(BROWSER_LOCK, 'wx');
      fs.writeSync(fd, String(process.pid));
      fs.closeSync(fd);
      return () => { try { fs.unlinkSync(BROWSER_LOCK); } catch {} };
    } catch (e) {
      if (e.code !== 'EEXIST') return () => {};
      // stale lock (holder gone) is reclaimed
      try {
        const pid = Number(fs.readFileSync(BROWSER_LOCK, 'utf8').trim());
        if (pid && !fs.existsSync(`/proc/${pid}`)) { fs.unlinkSync(BROWSER_LOCK); continue; }
      } catch {}
      if (Date.now() > deadline) throw new Error(`timed out waiting for ${BROWSER_LOCK} (another harness browser is running)`);
      spawnSync('sleep', ['2']);
    }
  }
}

// Which build was measured? The Ralph loop rebuilds target/site under us, so a score is only
// meaningful if it names the artifact it scored. Reported in the output and the report files.
async function servedBuild(base) {
  try {
    const r = await fetch(`${base}/pkg/docs-app.wasm`, { method: 'HEAD' });
    return { bytes: Number(r.headers.get('content-length') || 0), lastModified: r.headers.get('last-modified') || 'unknown' };
  } catch { return { bytes: 0, lastModified: 'unreachable' }; }
}

const CHROME_FLAGS = [
  '--headless=new', '--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage',
  '--no-zygote', '--disable-features=site-per-process,IsolateOrigins,Translate,BackForwardCache',
  '--renderer-process-limit=1', '--disable-background-timer-throttling', '--mute-audio',
];

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const CHROME = process.env.CHROME || '/data/tools/chrome-wrapper.sh';
const UPSTREAM_BASE = process.env.UPSTREAM_DOCS_BASE || 'http://127.0.0.1:3005';
const LEPTOS_BASE = process.env.LEPTOS_DOCS_BASE || 'http://127.0.0.1:3177';
const OUT_DIR = path.join(PROJECT_ROOT, 'ralph/logs/visual');
const PORT = 9899;

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
if (!route) {
  console.error('usage: visual-gap-report.mjs --route react/components/<name> | --todo-id "<id>"');
  process.exit(1);
}

// ---- CDP plumbing (same zero-dep approach as visual-diff.mjs) ----
class CDP {
  constructor(ws) {
    this.ws = ws; this.id = 0; this.pending = new Map();
    ws.addEventListener('message', (ev) => {
      const m = JSON.parse(ev.data);
      if (m.id && this.pending.has(m.id)) { const r = this.pending.get(m.id); this.pending.delete(m.id); r(m); }
    });
  }
  send(method, params = {}) {
    const id = ++this.id;
    return new Promise((res) => {
      this.pending.set(id, res);
      this.ws.send(JSON.stringify({ id, method, params }));
      setTimeout(() => { if (this.pending.has(id)) { this.pending.delete(id); res({ error: { message: `cdp timeout ${method}` } }); } }, 90000);
    });
  }
}
async function connect() {
  const r = await fetch(`http://127.0.0.1:${PORT}/json/new?about:blank`, { method: 'PUT' });
  const tab = JSON.parse(await r.text());
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  const cdp = new CDP(ws);
  await cdp.send('Page.enable');
  await cdp.send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 2400, deviceScaleFactor: 1, mobile: false });
  return cdp;
}
async function settled(cdp) {
  for (let i = 0; i < 80; i++) {
    const t = await cdp.send('Runtime.evaluate', {
      expression: `document.readyState + '|' + (document.querySelector('main h1,h1') ? '1' : '0')`,
      returnByValue: true,
    });
    if (t.result?.result?.value === 'complete|1') break;
    await new Promise((r) => setTimeout(r, 500));
  }
  await new Promise((r) => setTimeout(r, 2000)); // effects/animations settle
}
async function evalJson(cdp, expr) {
  // Two attempts: with and without awaitPromise. A wrong choice there yields "no value" rather
  // than an error, and a silent null would read as "no gaps" instead of "not measured".
  let lastError = null;
  for (const awaitPromise of [true, false]) {
    const r = await cdp.send('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise });
    const res = r.result;
    if (res?.exceptionDetails) {
      lastError = res.exceptionDetails.exception?.description || res.exceptionDetails.text || 'evaluate threw';
      continue;
    }
    const v = res?.result?.value;
    if (v === undefined) { lastError = lastError || 'evaluate returned no value'; continue; }
    try { return JSON.parse(v); } catch { return v; }
  }
  if (lastError) console.error(`  [probe] ${String(lastError).slice(0, 300)}`);
  return null;
}
async function shoot(cdp, file) {
  const s = await cdp.send('Page.captureScreenshot', { format: 'png' });
  if (s.result?.data) fs.writeFileSync(file, Buffer.from(s.result.data, 'base64'));
}

// ---- the page probe: everything we compare, in one round trip ----
const PROBE = `(() => {
  const main = document.querySelector('main') || document.body;
  const cs = (el) => el ? getComputedStyle(el) : null;
  const rect = (el) => el ? el.getBoundingClientRect() : null;
  const px = (v) => v === null || v === undefined ? null : Math.round(parseFloat(v) || 0);

  // typography of the first instance of each tag
  const typo = {};
  for (const tag of ['h1','h2','h3','p','li','code','pre','table','a']) {
    const el = main.querySelector(tag);
    const c = cs(el);
    typo[tag] = el ? {
      fontSize: px(c.fontSize), fontWeight: c.fontWeight, lineHeight: px(c.lineHeight),
      marginTop: px(c.marginTop), marginBottom: px(c.marginBottom),
      color: c.color, fontFamily: c.fontFamily.split(',')[0].replace(/"/g,''),
    } : null;
  }

  // ---- structural chrome inventory, measured the same way on both sides ----
  const linkTexts = [...main.querySelectorAll('a')].map(a => a.textContent.trim()).filter(Boolean);
  const navLike = [...document.querySelectorAll('nav,aside,[role=navigation]')];
  const navRect = navLike.length ? rect(navLike[0]) : null;

  // sidebar: the widest left-edge column outside the article that holds many links
  let sidebar = { present: false, width: 0, linkCount: 0 };
  for (const el of document.querySelectorAll('div,aside,nav,ul')) {
    const r = rect(el);
    if (!r || r.width < 120 || r.width > 420 || r.height < 200) continue;
    if (r.left > 40) continue;
    const links = el.querySelectorAll('a').length;
    if (links >= 5 && r.height > 300) {
      if (r.width > sidebar.width) sidebar = { present: true, width: Math.round(r.width), linkCount: links };
    }
  }

  // header: a short full-width band at the top carrying links/buttons
  let header = { present: false, height: 0, controls: 0 };
  for (const el of document.querySelectorAll('header,div,nav')) {
    const r = rect(el);
    if (!r || r.height < 30 || r.height > 140 || r.width < 600 || r.top > 120) continue;
    const controls = el.querySelectorAll('a,button,input').length;
    if (controls >= 2 && r.width > 600) { header = { present: true, height: Math.round(r.height), controls }; break; }
  }

  // code blocks + syntax highlighting: how many tokens inside <pre> are coloured
  // differently from their pre (a robust proxy for a highlighter having run)
  const pres = [...main.querySelectorAll('pre')];
  let colouredTokens = 0, codeBlocks = pres.length, copyButtons = 0;
  for (const pre of pres) {
    const base = cs(pre).color;
    for (const el of pre.querySelectorAll('*')) {
      const c = cs(el);
      if (c && c.color !== base && c.color !== 'rgba(0, 0, 0, 0)') colouredTokens++;
    }
  }
  copyButtons = [...main.querySelectorAll('button,[role=button]')].filter(b => /copy/i.test((b.getAttribute('aria-label')||'') + ' ' + b.textContent)).length;


  // ---- code snippet language: a mirrored page must demonstrate the PORT's API, not upstream's ----
  // A page that embeds React source is not "content parity" — it is the wrong framework with the
  // right word count, and it inflates any text-based recall score. Classify every snippet.
  // NOTE: backslashes are doubled throughout — this lives inside a template literal, and Node
  // processes its escapes before the browser ever evaluates the code.
  const snippetTexts = [...main.querySelectorAll('pre')].map(p => p.textContent || '');
  const looksReact = (t) =>
    /@base-ui\\/react|@mui\\//.test(t) ||
    /import\\s+[\\s\\S]{0,120}?\\sfrom\\s+['"]/.test(t) ||
    /useState|useRef|useEffect|useCallback/.test(t) ||
    /className=|onClick=\\{|\\{props|=>\\s*\\(|=>\\s*\\{/.test(t) ||
    /<\\/?[A-Z][A-Za-z]*(\\.[A-Z][A-Za-z]*)?[\\s/>]/.test(t);
  const looksLeptos = (t) =>
    /use leptos/.test(t) ||
    /leptos_ui|leptos-ui/.test(t) ||
    /view!|#\\[component\\]|->\\s*impl\\s+IntoView|cx\\(|Signal<|RwSignal|ReadSignal|Memo<|on:click|prop:|attr:/.test(t);
  const snippets = { total: snippetTexts.length, leptos: 0, react: 0, other: 0 };
  for (const t of snippetTexts) {
    const r = looksReact(t), l = looksLeptos(t);
    if (l && !r) snippets.leptos++;
    else if (r) snippets.react++;
    else snippets.other++;
  }
  // demo chrome: panels + file-selector tabs
  const tabs = main.querySelectorAll('[role=tab]').length;
  const demos = main.querySelectorAll('[class*=demo],[class*=Demo]').length;

  // api tables
  const tables = main.querySelectorAll('table').length;
  const tableRows = main.querySelectorAll('table tr').length;

  // source/markdown affordances upstream renders in the page header area
  const affordances = [...main.querySelectorAll('a,button')].map(e => e.textContent.trim())
    .filter(t => /view as|view source|markdown|stackblitz|copy/i.test(t));

  // layout geometry of the article column
  const article = main.querySelector('article') || main.querySelector('[class*=content]') || main;
  const ar = rect(article);

  return JSON.stringify({
    headings: [...main.querySelectorAll('h1,h2,h3')].map(h => h.textContent.trim()),
    typo,
    chrome: { sidebar, header, codeBlocks, colouredTokens, copyButtons, tabs, demos, tables, tableRows, affordances },
    snippets,
    nav: { containers: navLike.length, width: navRect ? Math.round(navRect.width) : 0, links: linkTexts.length },
    layout: { articleWidth: ar ? Math.round(ar.width) : null, articleLeft: ar ? Math.round(ar.left) : null,
              bodyFontSize: px(cs(document.body).fontSize), bodyFamily: cs(document.body).fontFamily.split(',')[0].replace(/"/g,'') },
    textLen: main.textContent.replace(/\\s+/g,' ').length,
    links: linkTexts.length,
  });
})()`;

function pct(n, d) { return d ? Math.round((n / d) * 100) : 0; }

function buildFindings(up, lx) {
  const f = [];
  const push = (severity, area, gap, fix) => f.push({ severity, area, gap, fix });

  // --- chrome presence gaps (ordered by how much of the pixel distance they own) ---
  if (up.chrome.sidebar.present && !lx.chrome.sidebar.present) {
    push('P0', 'sidebar', `upstream renders a ${up.chrome.sidebar.width}px sidebar carrying ${up.chrome.sidebar.linkCount} links; this page has none`,
      'port upstream\'s layout shell: a left column with the route tree (docs/src/app/(docs)/layout.tsx + the nav component) so every route gets the sidebar');
  } else if (up.chrome.sidebar.present && lx.chrome.sidebar.present) {
    const dw = Math.abs(up.chrome.sidebar.width - lx.chrome.sidebar.width);
    if (dw > 20) push('P2', 'sidebar', `sidebar width differs: ${lx.chrome.sidebar.width}px vs upstream ${up.chrome.sidebar.width}px`,
      'match upstream\'s sidebar width/typography rather than an approximation');
  }
  if (up.chrome.header.present && !lx.chrome.header.present) {
    push('P1', 'header', `upstream renders a ${up.chrome.header.height}px header with ${up.chrome.header.controls} controls; this page has none`,
      'port the docs header (title, nav links, search surface) from docs/src/app/(docs)/layout.tsx');
  }
  if (up.chrome.affordances.length && !lx.chrome.affordances.length) {
    push('P2', 'page affordances', `upstream shows ${up.chrome.affordances.length} page actions (${up.chrome.affordances.slice(0, 4).join(' / ')}); this page shows none`,
      'add the page-action row (View as Markdown / View source) to the docs page shell');
  }

  // --- code block chrome + highlighting ---
  if (up.chrome.codeBlocks > 0) {
    const ratio = pct(lx.chrome.codeBlocks, up.chrome.codeBlocks);
    if (ratio < 80) push('P1', 'code blocks', `this page has ${lx.chrome.codeBlocks} <pre> blocks vs upstream's ${up.chrome.codeBlocks} (${ratio}%)`,
      'render the embedded snippets through the ported code-block component so each code fence becomes its own block');
    if (up.chrome.colouredTokens > 0 && lx.chrome.colouredTokens === 0) {
      push('P0', 'syntax highlighting', `upstream\'s code carries ${up.chrome.colouredTokens} coloured tokens; this page has 0 — the code is unstyled monochrome`,
        'port the highlighter (upstream pre-computes tokens via rehype/Shiki into CodeBlockPreComputed) and render the token spans with their colours');
    } else if (up.chrome.colouredTokens > 0 && pct(lx.chrome.colouredTokens, up.chrome.colouredTokens) < 60) {
      push('P1', 'syntax highlighting', `coloured tokens ${lx.chrome.colouredTokens} vs upstream ${up.chrome.colouredTokens}`,
        'extend the highlighter coverage — some snippets render unstyled');
    }
    if (up.chrome.copyButtons > 0 && lx.chrome.copyButtons === 0) {
      push('P2', 'code copy control', `upstream renders ${up.chrome.copyButtons} copy affordance(s) on code; this page renders none`,
        'add the copy-to-clipboard control to the code-block chrome');
    }
  }

  // --- demo chrome ---
  if (up.chrome.tabs > 0 && lx.chrome.tabs === 0) {
    push('P1', 'demo file tabs', `upstream renders ${up.chrome.tabs} tab(s) for demo source files; this page renders none`,
      'wrap each demo in upstream\'s Demo container with the react/tailwind/css-modules file selector');
  }
  if (up.chrome.tables > 0 && lx.chrome.tables === 0) {
    push('P0', 'API reference tables', `upstream renders ${up.chrome.tables} table(s) (${up.chrome.tableRows} rows); this page renders 0 — props/state are prose`,
      'render the generated types tables (name/type/default/description) over the ported types.md instead of paragraphs');
  }

  // --- code snippet language: the page must demonstrate the PORT, not upstream ---
  if (lx.snippets && lx.snippets.react > 0) {
    push('P0', 'code snippets show React source',
      `${lx.snippets.react} of ${lx.snippets.total} code block(s) still contain React source (JSX, hooks, or \`@base-ui/react\` imports) instead of the Leptos port's own API`,
      "translate each embedded snippet to its Leptos equivalent (leptos_ui parts + view! syntax) as you mirror the page — a mirrored page that teaches React is not a port of it, and its presence also inflates content-recall scoring");
  } else if (lx.snippets && lx.snippets.total > 0 && lx.snippets.leptos === 0) {
    push('P1', 'code snippets language',
      `${lx.snippets.total} snippet(s) present but none identify as Leptos (\`use leptos\`, \`view!\`, leptos_ui parts)`,
      'check that embedded examples show the port\'s own API rather than framework-neutral pseudo-code');
  }

  // --- content volume ---
  const textRatio = pct(lx.textLen, up.textLen);
  if (textRatio < 85) {
    push(textRatio < 60 ? 'P0' : 'P1', 'content volume', `page text is ${lx.textLen} chars vs upstream ${up.textLen} (${textRatio}%)`,
      'the missing tables/affordances above are the usual cause — re-measure after they land');
  }

  // --- typography drift ---
  for (const tag of ['h1', 'h2', 'p', 'code', 'pre', 'li']) {
    const a = up.typo[tag], b = lx.typo[tag];
    if (!a || !b) continue;
    const bits = [];
    if (a.fontSize !== b.fontSize) bits.push(`font-size ${b.fontSize}px vs ${a.fontSize}px`);
    if (a.fontWeight !== b.fontWeight) bits.push(`weight ${b.fontWeight} vs ${a.fontWeight}`);
    if (a.lineHeight !== b.lineHeight) bits.push(`line-height ${b.lineHeight}px vs ${a.lineHeight}px`);
    if (a.fontFamily !== b.fontFamily) bits.push(`font ${b.fontFamily} vs ${a.fontFamily}`);
    if (bits.length) push('P2', `typography <${tag}>`, bits.join(', '), `match upstream\'s ${tag} styles in the docs stylesheet`);
  }

  // --- layout geometry ---
  if (up.layout.articleWidth && lx.layout.articleWidth && Math.abs(up.layout.articleWidth - lx.layout.articleWidth) > 60) {
    push('P1', 'layout width', `content column ${lx.layout.articleWidth}px vs upstream ${up.layout.articleWidth}px`,
      'match the docs grid so the article sits in the same column width/offset');
  }
  if (up.layout.bodyFamily !== lx.layout.bodyFamily) {
    push('P1', 'fonts', `body font "${lx.layout.bodyFamily}" vs upstream "${up.layout.bodyFamily}"`,
      'load the docs font stack (upstream docs/src/css/index.css @theme) in the docs-app stylesheet');
  }

  const rank = { P0: 0, P1: 1, P2: 2 };
  return f.sort((x, y) => rank[x.severity] - rank[y.severity]);
}

function renderMarkdown(route, up, lx, findings, diff, build) {
  const lines = [];
  lines.push(`# Visual gap report — ${route}`);
  lines.push('');
  lines.push(`Generated by \`ralph/scripts/visual-gap-report.mjs\` at ${new Date().toISOString()}.`);
  lines.push(`Measured build: wasm ${build.bytes} bytes, last-modified ${build.lastModified}.`);
  lines.push('Both sides rendered at 1280px in Chrome for Testing: upstream React docs on');
  lines.push(`${UPSTREAM_BASE}, Leptos docs-app on ${LEPTOS_BASE}.`);
  lines.push('');
  lines.push(`**Pixel difference: ${diff.percent}%** (${diff.diffPixels} of ${diff.totalPixels} px)`);
  if ((lx.headings || []).length <= 2 && lx.textLen < 600) {
    lines.push('');
    lines.push('> **WARNING: the Leptos side rendered shell-only.** These numbers measure a page that');
    lines.push('> did not mount its route, not a page that is merely unstyled. Fix the mount first.');
  }
  lines.push('');
  lines.push(`- mask (white/bright = differs): \`ralph/logs/visual/${route.split('/').pop()}-mask.png\``);
  lines.push(`- overlay (upstream grey, ours red): \`ralph/logs/visual/${route.split('/').pop()}-sbs.png\``);
  lines.push('');
  lines.push('## Prioritised gaps');
  lines.push('');
  if (!findings.length) {
    lines.push('No named gaps detected by the structural probes — remaining distance is sub-threshold.');
  }
  for (const f of findings) {
    lines.push(`- **${f.severity} · ${f.area}** — ${f.gap}`);
    lines.push(`  - fix: ${f.fix}`);
  }
  lines.push('');
  lines.push('## Measured inventory');
  lines.push('');
  lines.push('| signal | upstream | leptos |');
  lines.push('| --- | --- | --- |');
  const rows = [
    ['headings', up.headings.length, lx.headings.length],
    ['sidebar', up.chrome.sidebar.present ? `${up.chrome.sidebar.width}px / ${up.chrome.sidebar.linkCount} links` : 'none', lx.chrome.sidebar.present ? `${lx.chrome.sidebar.width}px / ${lx.chrome.sidebar.linkCount} links` : 'none'],
    ['header', up.chrome.header.present ? `${up.chrome.header.height}px` : 'none', lx.chrome.header.present ? `${lx.chrome.header.height}px` : 'none'],
    ['links', up.links, lx.links],
    ['code blocks', up.chrome.codeBlocks, lx.chrome.codeBlocks],
    ['coloured code tokens', up.chrome.colouredTokens, lx.chrome.colouredTokens],
    ['copy controls', up.chrome.copyButtons, lx.chrome.copyButtons],
    ['demo tabs', up.chrome.tabs, lx.chrome.tabs],
    ['tables / rows', `${up.chrome.tables} / ${up.chrome.tableRows}`, `${lx.chrome.tables} / ${lx.chrome.tableRows}`],
    ['snippets (leptos/react/other)', up.snippets ? `${up.snippets.leptos}/${up.snippets.react}/${up.snippets.other}` : 'n/a', lx.snippets ? `${lx.snippets.leptos}/${lx.snippets.react}/${lx.snippets.other}` : 'n/a'],
    ['page text chars', up.textLen, lx.textLen],
    ['article width px', up.layout.articleWidth, lx.layout.articleWidth],
    ['body font', up.layout.bodyFamily, lx.layout.bodyFamily],
  ];
  for (const r of rows) lines.push(`| ${r[0]} | ${r[1]} | ${r[2]} |`);
  lines.push('');
  lines.push('## Typography (upstream vs leptos)');
  lines.push('');
  lines.push('| tag | upstream | leptos |');
  lines.push('| --- | --- | --- |');
  for (const tag of ['h1', 'h2', 'h3', 'p', 'li', 'code', 'pre', 'table', 'a']) {
    const a = up.typo[tag], b = lx.typo[tag];
    const fmt = (t) => t ? `${t.fontSize}px/${t.lineHeight}px ${t.fontWeight} ${t.fontFamily}` : '(absent)';
    lines.push(`| ${tag} | ${fmt(a)} | ${fmt(b)} |`);
  }
  lines.push('');
  return lines.join('\n');
}

async function main() {
  if (!reachable(`${LEPTOS_BASE}/`)) {
    console.error(`FAIL: Leptos docs server unreachable at ${LEPTOS_BASE} — run: python3 ralph/scripts/serve-docs-app.py --port 3177`);
    process.exit(2);
  }
  if (!reachable(`${UPSTREAM_BASE}/`)) {
    console.error(`NOTE: upstream React docs unreachable at ${UPSTREAM_BASE} — cannot diagnose gaps without the reference render.`);
    console.error('      Start it: (cd docs && node_modules/.bin/next dev --port 3005)');
    process.exit(2);
  }

  fs.mkdirSync(OUT_DIR, { recursive: true });
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'gaprep-'));
  const releaseLock = acquireBrowserLock();
  const chrome = spawn(CHROME, [...CHROME_FLAGS, `--user-data-dir=${tmp}`, `--remote-debugging-port=${PORT}`,
    '--window-size=1280,2400', 'about:blank'], { stdio: 'ignore' });
  let ok = false;
  try {
    for (let i = 0; i < 60; i++) {
      try { await fetch(`http://127.0.0.1:${PORT}/json/version`, { signal: AbortSignal.timeout(2000) }); ok = true; break; }
      catch { await new Promise((r) => setTimeout(r, 500)); }
    }
    if (!ok) throw new Error('chrome devtools port never came up');

    const name = route.split('/').pop();
    const build = await servedBuild(LEPTOS_BASE);
    let lxUnmounted = false;
    const upPng = path.join(tmp, 'up.png'), lxPng = path.join(tmp, 'lx.png');

    const cdpUp = await connect();
    let up = null;
    for (let attempt = 1; attempt <= 3 && !up; attempt++) {
      // `next dev` compiles routes on first request and can return an unfinished document while
      // it does; a null probe means "not measurable yet", never "no gaps".
      await cdpUp.send('Page.navigate', { url: `${UPSTREAM_BASE}/${route}` });
      await settled(cdpUp);
      up = await evalJson(cdpUp, PROBE);
      if (!up) await new Promise((r) => setTimeout(r, 8000));
    }
    if (!up) throw new Error(`upstream probe returned nothing for ${route} after 3 attempts (is next dev still compiling? see /tmp/next-dev.log)`);
    await shoot(cdpUp, upPng);

    let cdpLx = await connect();
    await cdpLx.send('Page.navigate', { url: `${LEPTOS_BASE}/${route}` });
    await settled(cdpLx);
    let lx = await evalJson(cdpLx, PROBE);
    // A shell-only render means the route did not mount (mid-rebuild target/site, a panic, or a
    // router miss). Measuring that as "the page is unstyled" would be a lie, so retry once and
    // then say so explicitly.
    const looksUnmounted = (l) => l && l.textLen < Math.max(600, 0.25 * (up.textLen || 0)) && (l.headings || []).length <= 2;
    if (looksUnmounted(lx)) {
      await new Promise((r) => setTimeout(r, 12000));
      await cdpLx.send('Page.navigate', { url: `${LEPTOS_BASE}/${route}?_retry=1` });
      await settled(cdpLx);
      lx = await evalJson(cdpLx, PROBE);
      if (looksUnmounted(lx)) lxUnmounted = true;
    }
    await shoot(cdpLx, lxPng);

    const px = compare(decodePng(fs.readFileSync(upPng)), decodePng(fs.readFileSync(lxPng)));

    let findings = buildFindings(up, lx);
    if (lxUnmounted) {
      findings = [{
        severity: 'P0',
        area: 'route did not render',
        gap: `this route rendered shell-only (${lx.textLen} chars, ${lx.headings.length} heading(s)) even after a retry — the page content is missing entirely`,
        fix: 'fix the mount before any styling work: run `node ralph/scripts/playwright-diff.mjs --todo-id "<id>"` to see the failure, and check for a wasm panic (the docs_app_start panic hook logs it as a console error). Fidelity numbers in this report are meaningless until it renders.',
      }, ...findings];
    }
    const md = renderMarkdown(route, up, lx, findings, px, build);

    fs.writeFileSync(path.join(OUT_DIR, `${name}.md`), md);
    fs.writeFileSync(path.join(OUT_DIR, `${name}.json`), `${JSON.stringify({ route, measuredBuild: build, generatedAt: new Date().toISOString(), pixelDiff: { percent: px.percent, diffPixels: px.diffPixels, totalPixels: px.totalPixels }, findings, upstream: up, leptos: lx }, null, 1)}\n`);
    fs.writeFileSync(path.join(OUT_DIR, `${name}-mask.png`), px.maskPng);
    fs.writeFileSync(path.join(OUT_DIR, `${name}-overlay.png`), px.overlayPng);

    console.log(`visual gap report: ${route} — ${px.percent}% pixels differ, ${findings.length} named gap(s)`);
    console.log(`  measured build: wasm ${build.bytes} bytes @ ${build.lastModified}`);
    for (const f of findings) console.log(`  ${f.severity} ${f.area}: ${f.gap}`);
    console.log(`\nreport: ralph/logs/visual/${name}.md`);
    console.log(`images: ralph/logs/visual/${name}-mask.png, ralph/logs/visual/${name}-overlay.png`);
  } catch (e) {
    console.error(`FAIL: ${e.message}`);
    process.exitCode = 2;
  } finally {
    chrome.kill('SIGKILL');
    releaseLock();
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

main();
