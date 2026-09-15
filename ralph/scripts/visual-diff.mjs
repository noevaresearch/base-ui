#!/usr/bin/env node
// visual-diff.mjs — pixel + structural diff between upstream React docs and Leptos docs.
// Screenshots both routes headless, compares via pixel sampling (no deps).
// Usage: CHROME=... node ralph/scripts/visual-diff.mjs --route react/components/checkbox

import { spawn } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const CHROME = process.env.CHROME || '/data/tools/chrome-wrapper.sh';
function arg(n, d) { const i = process.argv.indexOf(`--${n}`); return i >= 0 && process.argv[i+1] ? process.argv[i+1] : d; }
const route = arg('route', 'react/components/checkbox');
const LEPTOS = arg('leptos', `http://127.0.0.1:3177/${route}`);
const UPSTREAM = arg('upstream', `http://127.0.0.1:3005/${route}`);
const OUT = arg('out', `/tmp/visual-diff-${route.split('/').pop()}`);

fs.mkdirSync(OUT, { recursive: true });
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'visdiff-'));
let chrome = null;
async function portUp() {
  try { const r = await fetch('http://127.0.0.1:9888/json/version', { signal: AbortSignal.timeout(2000) }); return r.ok; }
  catch { return false; }
}
if (!(await portUp())) {
  chrome = spawn(CHROME, [`--user-data-dir=${tmp}`, '--remote-debugging-port=9888',
    '--headless=new', '--no-sandbox', '--disable-gpu', '--window-size=1280,2000', 'about:blank'], { stdio: 'ignore' });
  for (let i = 0; i < 60; i++) { if (await portUp()) break; await new Promise(r => setTimeout(r, 500)); }
  if (!(await portUp())) { console.error('chrome devtools port never came up'); process.exit(1); }
}
process.on('exit', () => { try { chrome && chrome.kill('SIGKILL'); } catch {} });

class CDP {
  constructor(ws) { this.ws = ws; this.id = 0; this.pending = new Map();
    ws.addEventListener('message', ev => { const m = JSON.parse(ev.data);
      if (m.id && this.pending.has(m.id)) { const r = this.pending.get(m.id); this.pending.delete(m.id); r(m); } });
  }
  send(method, params = {}) { const id = ++this.id;
    return new Promise(res => { this.pending.set(id, res); this.ws.send(JSON.stringify({ id, method, params })); setTimeout(() => { if (this.pending.has(id)) { this.pending.delete(id); res({ error: { message: 'timeout' } }); } }, 60000); });
  }
}
async function shoot(url, name) {
  const tabs = await fetch('http://127.0.0.1:9888/json/list').then(r => r.json());
  const ws = new WebSocket(tabs[0].webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  const cdp = new CDP(ws);
  await cdp.send('Page.enable'); await cdp.send('Emulation.setDeviceMetricsOverride',
    { width: 1280, height: 2000, deviceScaleFactor: 1, mobile: false });
  await cdp.send('Page.navigate', { url });
  // wait for mount + settle
  for (let i = 0; i < 60; i++) {
    await new Promise(r => setTimeout(r, 500));
    const t = await cdp.send('Runtime.evaluate', { expression: `document.readyState + '|' + (document.querySelector('main h1,h1')?'1':'0')`, returnByValue: true });
    if (t.result?.result?.value === 'complete|1') break;
  }
  await new Promise(r => setTimeout(r, 2500));
  const shot = await cdp.send('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(`${OUT}/${name}.png`, Buffer.from(shot.result.data, 'base64'));
  // structural stats
  const stats = await cdp.send('Runtime.evaluate', { expression: `(() => {
    const m = document.querySelector('main') || document.body;
    return JSON.stringify({
      headings: [...m.querySelectorAll('h1,h2,h3')].map(h => h.textContent.trim()),
      demos: m.querySelectorAll('[class*=demo]').length,
      tables: m.querySelectorAll('table').length,
      codeBlocks: m.querySelectorAll('pre,code').length,
      links: m.querySelectorAll('a').length,
      inputs: m.querySelectorAll('input,button').length,
      textLen: m.textContent.replace(/\\s+/g, ' ').length
    });
  })()`, returnByValue: true });
  const v = JSON.parse(stats.result.result.value);
  ws.close();
  return v;
}
async function pixelDiff(a, b) {
  // decode PNGs via offscreen canvas in a fresh tab
  const tabs = await fetch('http://127.0.0.1:9888/json/list').then(r => r.json());
  const ws = new WebSocket(tabs[0].webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  const cdp = new CDP(ws);
  const b64 = (f) => fs.readFileSync(f).toString('base64');
  const expr = `(() => new Promise((resolve) => {
    const load = (src) => new Promise(r2 => { const i = new Image(); i.onload = () => r2(i); i.src = src; });
    Promise.all([load('data:image/png;base64,${b64(`${OUT}/leptos.png`)}'), load('data:image/png;base64,${b64(`${OUT}/upstream.png`)}')]).then(([A, B]) => {
      const w = Math.min(A.width, B.width), h = Math.min(A.height, B.height);
      const c1 = new OffscreenCanvas(w, h), c2 = new OffscreenCanvas(w, h);
      c1.getContext('2d').drawImage(A, 0, 0); c2.getContext('2d').drawImage(B, 0, 0);
      const d1 = c1.getContext('2d').getImageData(0, 0, w, h).data;
      const d2 = c2.getContext('2d').getImageData(0, 0, w, h).data;
      let diff = 0, total = w * h;
      for (let i = 0; i < d1.length; i += 4) {
        if (Math.abs(d1[i]-d2[i]) > 16 || Math.abs(d1[i+1]-d2[i+1]) > 16 || Math.abs(d1[i+2]-d2[i+2]) > 16) diff++;
      }
      resolve((diff / total * 100).toFixed(2) + '% pixels differ (' + diff + '/' + total + ')');
    });
  }))()`;
  const r = await cdp.send('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true });
  ws.close();
  return r.result?.result?.value ?? r.error?.message ?? 'pixel diff failed';
}
const upstreamStats = await shoot(UPSTREAM, 'upstream').catch(e => ({ error: String(e) }));
const leptosStats = await shoot(LEPTOS, 'leptos').catch(e => ({ error: String(e) }));
const report = { route, upstreamStats, leptosStats };
try { report.pixelDiff = await pixelDiff(); } catch (e) { report.pixelDiff = String(e); }
fs.writeFileSync(`${OUT}/report.json`, JSON.stringify(report, null, 1));
console.log(JSON.stringify(report, null, 1));
if (chrome) chrome.kill('SIGKILL');
