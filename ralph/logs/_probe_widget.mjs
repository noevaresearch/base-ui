#!/usr/bin/env node
// _probe_widget.mjs — ONE-OFF diagnostic (not part of any gate).
//
// Why: the widget region must be the demo's OWN rendering area on both sides. `visual-diff.mjs` and
// `visual-gap-report.mjs` each used to compute that region with their own `querySelector` over a
// demo-ish class list, which resolves in DOCUMENT ORDER — so on upstream the outer `div.demo` (which
// also contains the demo's source panel) won over the inner `div.DemoPlaygroundInner`, and the
// measured "widget" was the port's button scored against the top-left corner of upstream's
// demo+panel crop. The region now comes from ralph/scripts/lib/widget-region.mjs, and this probe
// dumps what that module sees per side (every candidate container, which were rejected and why, the
// chosen scope, every part with its rejection reason, and the resulting rects) so a fault is
// diagnosed from the live DOM instead of guessed at.
//
// Usage: node ralph/logs/_probe_widget.mjs --route react/components/button
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { launchChrome, killChrome, FRUGAL_CHROME_FLAGS } from '../../ralph/scripts/lib/browser.mjs';
import { WIDGET_REGION_JS } from '../../ralph/scripts/lib/widget-region.mjs';

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

const DIAG = `(() => {
  const rectOf = (el) => { const b = el.getBoundingClientRect(); return { x: Math.round(b.x), y: Math.round(b.y), w: Math.round(b.width), h: Math.round(b.height) }; };
  const CONTROL = 'button,input:not([type=hidden]),select,textarea,label,[role]';
  const CHROME_SELECTOR = 'nav,aside,header,pre,code,figure,[class*=Code],[class*=code],[class*=Tabs],[class*=Selector],[class*=Clipboard],[class*=Copy]';
  const main = document.querySelector('main') || document.body;
  const candidates = [...main.querySelectorAll('[class*=PlaygroundInner], .DemoRoot, .DemoPlayground, .docs-demo, [class*=demo], [class*=Demo]')]
    .map((el) => ({
      cls: String(el.className).slice(0, 60),
      rect: rectOf(el),
      controls: el.querySelectorAll(CONTROL).length,
      chrome: el.closest(CHROME_SELECTOR) !== null,
      pres: el.querySelectorAll('pre').length,
    }));
  const usable = candidates.filter((c) => !c.chrome && c.controls > 0).sort((a, b) => a.rect.w * a.rect.h - b.rect.w * b.rect.h);
  const region = ${WIDGET_REGION_JS};
  return JSON.stringify({ href: location.href, h1: (document.querySelector('h1') || {}).textContent, candidates, usable, region }, null, 1);
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
  const out = await cdp.send('Runtime.evaluate', { expression: DIAG, returnByValue: true });
  console.log(`\n========== ${name}: ${url} ==========`);
  console.log(out.result?.result?.value || JSON.stringify(out));
  ws.close();
}
killChrome(chrome);
lock();
