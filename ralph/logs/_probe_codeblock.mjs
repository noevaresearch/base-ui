// Scratch probe (this iteration): what does an embedded snippet's code block actually look like on
// upstream's render, and what does the port currently emit? The gap report measures counts; this
// reads the oracle's own DOM so the ported chrome is copied from upstream instead of guessed.
//
//   node ralph/logs/_probe_codeblock.mjs react/components/checkbox
//
// Same CDP plumbing as ralph/scripts/visual-gap-report.mjs (its own port, so it cannot collide with
// a harness browser that is up).
import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const PORT = 9888;
const CHROME = process.env.CHROME || '/data/tools/chrome-wrapper.sh';
const UPSTREAM_BASE = process.env.UPSTREAM_DOCS_BASE || 'http://127.0.0.1:3005';
const LEPTOS_BASE = process.env.LEPTOS_DOCS_BASE || 'http://127.0.0.1:3177';
const route = process.argv[2] || 'react/components/checkbox';
const MODE = process.argv[3] || 'blocks';

const CHROME_FLAGS = [
  '--headless=new', '--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage',
  '--no-zygote', '--disable-features=site-per-process,IsolateOrigins,Translate,BackForwardCache',
  '--renderer-process-limit=1', '--disable-background-timer-throttling', '--mute-audio',
];

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

const TITLES_PROBE = `(() => {
  const m = document.querySelector('main') || document.body;
  const titles = [...m.querySelectorAll('.CodeBlockPanelTitle')].map((t) => t.textContent.trim());
  const roots = [...m.querySelectorAll('.CodeBlockRoot')].map((r) => {
    const pre = r.querySelector('pre');
    const codeEl = r.querySelector('pre code');
    const lang = codeEl ? [...codeEl.classList].find((c) => c.startsWith('language-')) : null;
    const title = r.querySelector('.CodeBlockPanelTitle');
    return {
      title: title ? title.textContent.trim() : null,
      lang,
      firstLine: codeEl ? (codeEl.textContent || '').split('\\n')[0].slice(0, 60) : null,
      lines: codeEl ? (codeEl.textContent || '').split('\\n').length : null,
      hasCopy: !!r.querySelector('button[aria-label="Copy code"]'),
      inDemo: !!r.closest('[class*=Playground], .docs-demo, [class*=DemoRoot], [class*=Demo]'),
    };
  });
  return JSON.stringify({ panelTitles: titles, rootCount: roots.length, roots }, null, 1);
})()`;

const PROBE = MODE === 'titles' ? TITLES_PROBE : `(() => {
  const m = document.querySelector('main') || document.body;
  const pres = [...m.querySelectorAll('pre')];
  const chainOf = (el, n) => {
    const out = []; let e = el;
    for (let k = 0; k < n && e; k++) {
      const c = typeof e.className === 'string' ? e.className : '';
      out.push(e.tagName.toLowerCase() + (c ? '.' + c.trim().split(/\\s+/).join('.') : ''));
      e = e.parentElement;
    }
    return out;
  };
  const blocks = pres.map((p, i) => {
    const cs = getComputedStyle(p);
    const inDemo = !!p.closest('[class*=Playground], .docs-demo, [class*=DemoRoot], [class*=Demo]');
    const toks = [...p.querySelectorAll('*')];
    const coloured = toks.filter((e) => {
      const c = getComputedStyle(e).color;
      return c && c !== cs.color && c !== 'rgba(0, 0, 0, 0)';
    });
    const sample = coloured.slice(0, 6).map((e) => ({
      tag: e.tagName.toLowerCase(),
      cls: typeof e.className === 'string' ? e.className : '',
      color: getComputedStyle(e).color,
      txt: (e.textContent || '').slice(0, 24),
    }));
    const palette = {};
    for (const e of coloured) {
      const c = getComputedStyle(e).color;
      palette[c] = (palette[c] || 0) + 1;
    }
    return {
      i,
      inDemo,
      chain: chainOf(p, 6),
      pre: {
        color: cs.color, fontFamily: cs.fontFamily.slice(0, 60), fontSize: cs.fontSize,
        lineHeight: cs.lineHeight, padding: cs.padding, margin: cs.margin,
        background: cs.backgroundColor, borderRadius: cs.borderRadius, display: cs.display,
      },
      childTags: [...p.children].map((c) => c.tagName.toLowerCase() + (typeof c.className === 'string' && c.className ? '.' + c.className.trim().split(/\\s+/).join('.') : '')),
      tokTotal: toks.length,
      colouredTotal: coloured.length,
      palette,
      sample,
      inner: p.innerHTML.slice(0, 700),
      text: (p.textContent || '').slice(0, 120),
    };
  });
  // the block's own container (root) + panel: upstream wraps an embedded fence in
  // .CodeBlockRoot with a .CodeBlockPanel header holding the copy control
  const root = m.querySelector('.CodeBlockRoot');
  const panel = m.querySelector('.CodeBlockPanel');
  const rootInfo = (el) => {
    if (!el) return null;
    const cs = getComputedStyle(el);
    const r = el.getBoundingClientRect();
    return {
      tag: el.tagName.toLowerCase(), cls: el.className,
      rect: [Math.round(r.x), Math.round(r.y), Math.round(r.width), Math.round(r.height)],
      display: cs.display, background: cs.backgroundColor, border: cs.border, borderRadius: cs.borderRadius,
      padding: cs.padding, margin: cs.margin, position: cs.position,
      html: el.outerHTML.slice(0, 900),
    };
  };
  const copyButtons = [...m.querySelectorAll('button')]
    .filter((b) => /copy/i.test((b.getAttribute('aria-label') || '') + ' ' + b.textContent))
    .slice(0, 3)
    .map((b) => ({ lbl: b.getAttribute('aria-label'), html: b.outerHTML.slice(0, 400), chain: chainOf(b, 4) }));
  return JSON.stringify({
    mainClass: m.className,
    preCount: pres.length,
    embeddedCount: blocks.filter((b) => !b.inDemo).length,
    root: rootInfo(root),
    panel: rootInfo(panel),
    panelTitle: (() => { const t = m.querySelector('.CodeBlockPanelTitle'); return t ? t.textContent : null; })(),
    copyButtons,
    blocks,
  }, null, 1);
})()`;

async function connect() {
  const tab = JSON.parse(await (await fetch(`http://127.0.0.1:${PORT}/json/new?about:blank`, { method: 'PUT' })).text());
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  const cdp = new CDP(ws);
  await cdp.send('Page.enable');
  await cdp.send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 2400, deviceScaleFactor: 1, mobile: false });
  return cdp;
}

async function shoot(cdp, base, label) {
  await cdp.send('Page.navigate', { url: `${base}/${route}` });
  let last = '';
  for (let i = 0; i < 100; i++) {
    const t = await cdp.send('Runtime.evaluate', {
      expression: `[location.href.startsWith(${JSON.stringify(base)}), document.readyState, !!document.querySelector('main h1,h1')].join('|')`,
      returnByValue: true,
    });
    last = t.result?.result?.value || '';
    if (last.startsWith('true|complete|true')) break;
    await new Promise((r) => setTimeout(r, 500));
  }
  if (!last.startsWith('true|complete|true')) console.error(`[${label}] route never mounted: "${last}"`);
  await new Promise((r) => setTimeout(r, 2500));
  const r = await cdp.send('Runtime.evaluate', { expression: PROBE, returnByValue: true });
  if (r.result?.exceptionDetails) {
    console.error(`[${label}] probe threw:`, JSON.stringify(r.result.exceptionDetails).slice(0, 500));
    return null;
  }
  return JSON.parse(r.result.result.value);
}

const userDataDir = fs.mkdtempSync('/tmp/probe-cb-');
const chrome = spawn(CHROME, [...CHROME_FLAGS, `--user-data-dir=${userDataDir}`, `--remote-debugging-port=${PORT}`, 'about:blank'], { stdio: 'ignore' });
try {
  for (let i = 0; i < 40; i++) {
    const r = spawnSync('curl', ['-s', '-o', '/dev/null', '-w', '%{http_code}', `http://127.0.0.1:${PORT}/json/version`], { encoding: 'utf8' });
    if ((r.stdout || '').trim() === '200') break;
    await new Promise((res) => setTimeout(res, 500));
  }
  const cdpUp = await connect();
  const cdpLx = await connect();
  const outDir = path.resolve(import.meta.dirname, '..');
  for (const [base, label, cdp] of [[UPSTREAM_BASE, 'UPSTREAM', cdpUp], [LEPTOS_BASE, 'LEPTOS', cdpLx]]) {
    const out = await shoot(cdp, base, label);
    console.log(`\n===== ${label} ${base}/${route} =====`);
    console.log(out === null ? '(probe failed)' : JSON.stringify(out, null, 1));
    if (out) fs.writeFileSync(path.join(outDir, `logs/visual/_probe_codeblock_${label.toLowerCase()}${MODE === 'titles' ? '_titles' : ''}.json`), JSON.stringify(out, null, 1));
  }
} finally {
  chrome.kill('SIGKILL');
  spawnSync('sleep', ['1']);
  try { fs.rmSync(userDataDir, { recursive: true, force: true }); } catch {}
}
