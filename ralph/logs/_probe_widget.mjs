#!/usr/bin/env node
// _probe_widget.mjs — ONE-OFF diagnostic (not part of any gate).
//
// Why: `docs-fidelity: visual budget gate`'s widget region is measured per side with
//   main.querySelector('[class*=PlaygroundInner], .docs-demo, [class~=demo], .DemoRoot, [class*=demo]')
// and `querySelector` with a selector LIST returns the first match in DOCUMENT ORDER, not the first
// item in the list — so on upstream the outer `div.demo` (which also contains the demo's code panel)
// can win over the inner `div.DemoPlaygroundInner`. This probe dumps, for one route, every candidate
// container and every element the parts filter keeps/rejects, so the region fix is based on the live
// DOM instead of on class-name guesses.
//
// Usage: node ralph/logs/_probe_widget.mjs --route react/components/button
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { launchChrome, killChrome, FRUGAL_CHROME_FLAGS } from '../../ralph/scripts/lib/browser.mjs';

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
      try {
        const pid = Number(fs.readFileSync(BROWSER_LOCK, 'utf8').trim());
        if (pid && !fs.existsSync(`/proc/${pid}`)) { fs.unlinkSync(BROWSER_LOCK); continue; }
      } catch {}
      if (Date.now() > deadline) throw new Error(`timed out waiting for ${BROWSER_LOCK}`);
      spawnSync('sleep', ['2']);
    }
  }
}

function arg(n, d) { const i = process.argv.indexOf(`--${n}`); return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : d; }
const route = arg('route', 'react/components/button');
const TARGETS = [
  ['upstream', arg('upstream', `http://127.0.0.1:3005/${route}`)],
  ['leptos', arg('leptos', `http://127.0.0.1:3177/${route}`)],
];

const PROBE = `(() => {
  const rectOf = (el) => { const b = el.getBoundingClientRect(); return { x: Math.round(b.x), y: Math.round(b.y), w: Math.round(b.width), h: Math.round(b.height) }; };
  const m = document.querySelector('main') || document.body;
  const SEL = '[class*=PlaygroundInner], .docs-demo, [class~=demo], .DemoRoot, [class*=demo]';
  const cands = [...m.querySelectorAll(SEL)].map((el) => ({
    cls: String(el.className).slice(0, 70),
    rect: rectOf(el),
    controls: el.querySelectorAll('button,[role],input:not([type=hidden]),label').length,
  }));
  const first = m.querySelector(SEL);
  const scope = first || m;
  const parts = [...scope.querySelectorAll('label,input:not([type=hidden]),[role],select,textarea,button')].map((el) => ({
    tag: el.tagName,
    role: el.getAttribute('role'),
    cls: String(el.className).slice(0, 55),
    text: (el.textContent || '').trim().slice(0, 24),
    rect: rectOf(el),
    visible: el.getClientRects().length > 0,
    inNav: !!el.closest('nav,aside,header'),
    inCode: !!el.closest('pre,code,figure,[class*=Code],[class*=code],[class*=PlaygroundCode],[class*=Tabs],[class*=Selector]'),
    chain: (() => { const out = []; let p = el.parentElement; for (let i = 0; i < 7 && p; i++) { out.push(String(p.className || p.tagName).slice(0, 45)); p = p.parentElement; } return out; })(),
  }));
  const kept = parts.filter((p) => p.visible && !p.inNav && p.role !== 'tab' && !p.inCode);
  const box = (list) => { let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity; for (const p of list) { x0 = Math.min(x0, p.rect.x - 8); y0 = Math.min(y0, p.rect.y - 8); x1 = Math.max(x1, p.rect.x + p.rect.w + 8); y1 = Math.max(y1, p.rect.y + p.rect.h + 8); } return Number.isFinite(x0) ? { x: x0, y: y0, w: x1 - x0, h: y1 - y0 } : null; };
  return JSON.stringify({
    href: location.href,
    h1: (document.querySelector('h1') || {}).textContent,
    candidates: cands,
    docOrderFirst: { cls: String(scope.className).slice(0, 70), rect: rectOf(scope), containsPre: scope.querySelectorAll('pre').length },
    partsCount: parts.length,
    keptCount: kept.length,
    kept,
    rejected: parts.filter((p) => !kept.includes(p)).map((p) => ({ tag: p.tag, role: p.role, cls: p.cls, why: p.inNav ? 'nav' : p.role === 'tab' ? 'tab' : p.inCode ? 'inCode' : 'hidden' })),
    widgetRectNow: box(kept),
  }, null, 1);
})()`;

const lock = (() => { try { return acquireBrowserLock(); } catch (e) { console.error(String(e)); process.exit(1); } })();
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'widgetprobe-'));
let chrome = null;
try {
  const launched = await launchChrome(FRUGAL_CHROME_FLAGS, { tmpDir: tmp, port: 9889, waitMs: 45000 });
  chrome = launched.child;
} catch (e) {
  console.error(String(e.message || e));
  lock();
  process.exit(1);
}
process.on('exit', () => { try { killChrome(chrome); lock(); } catch {} });

class CDP {
  constructor(ws) { this.ws = ws; this.id = 0; this.pending = new Map();
    ws.addEventListener('message', (ev) => { const msg = JSON.parse(ev.data);
      if (msg.id && this.pending.has(msg.id)) { const r = this.pending.get(msg.id); this.pending.delete(msg.id); r(msg); } });
  }
  send(method, params = {}) { const id = ++this.id;
    return new Promise((res) => { this.pending.set(id, res); this.ws.send(JSON.stringify({ id, method, params })); setTimeout(() => { if (this.pending.has(id)) { this.pending.delete(id); res({ error: { message: 'timeout' } }); } }, 60000); });
  }
}

for (const [name, url] of TARGETS) {
  const tabs = await fetch('http://127.0.0.1:9889/json/list').then((r) => r.json());
  const ws = new WebSocket(tabs[0].webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  const cdp = new CDP(ws);
  await cdp.send('Page.enable');
  await cdp.send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 2000, deviceScaleFactor: 1, mobile: false });
  await cdp.send('Page.navigate', { url });
  for (let i = 0; i < 60; i++) {
    await new Promise((r) => setTimeout(r, 500));
    const t = await cdp.send('Runtime.evaluate', { expression: `document.readyState + '|' + (document.querySelector('main h1,h1')?'1':'0')`, returnByValue: true });
    if (t.result?.result?.value === 'complete|1') break;
  }
  await new Promise((r) => setTimeout(r, 2000));
  const out = await cdp.send('Runtime.evaluate', { expression: PROBE, returnByValue: true });
  console.log(`\n========== ${name}: ${url} ==========`);
  console.log(out.result?.result?.value || JSON.stringify(out));
  ws.close();
}
killChrome(chrome);
lock();
