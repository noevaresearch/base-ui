#!/usr/bin/env node
// visual-diff.mjs — pixel + structural diff between upstream React docs and Leptos docs.
// Screenshots both routes headless, compares via pixel sampling (no deps).
// Usage: CHROME=... node ralph/scripts/visual-diff.mjs --route react/components/checkbox

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { decodePng, compare } from './lib/png.mjs';

// Shared resource discipline for this box: a 512-task cgroup cap is shared with the Hermes
// gateway, the Ralph loop and cargo builds, so a default Chrome launch (~20 procs, 100+
// threads) can starve the whole box of forks. One renderer, no zygote, and a lock so only one
// harness browser is alive at a time.
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
const CHROME_FLAGS = [
  '--headless=new', '--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage',
  '--no-zygote', '--disable-features=site-per-process,IsolateOrigins,Translate,BackForwardCache',
  '--renderer-process-limit=1', '--mute-audio',
];


const CHROME = process.env.CHROME || '/data/tools/chrome-wrapper.sh';
function arg(n, d) { const i = process.argv.indexOf(`--${n}`); return i >= 0 && process.argv[i+1] ? process.argv[i+1] : d; }
const route = arg('route', 'react/components/checkbox');
const LEPTOS = arg('leptos', `http://127.0.0.1:3177/${route}`);
const UPSTREAM = arg('upstream', `http://127.0.0.1:3005/${route}`);
const OUT = arg('out', `/tmp/visual-diff-${route.split('/').pop()}`);

fs.mkdirSync(OUT, { recursive: true });
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'visdiff-'));
let chrome = null;
let releaseLock = () => {};
async function portUp() {
  try { const r = await fetch('http://127.0.0.1:9888/json/version', { signal: AbortSignal.timeout(2000) }); return r.ok; }
  catch { return false; }
}
if (!(await portUp())) {
  releaseLock = acquireBrowserLock();
  chrome = spawn(CHROME, [...CHROME_FLAGS, `--user-data-dir=${tmp}`, '--remote-debugging-port=9888',
    '--window-size=1280,2000', 'about:blank'], { stdio: 'ignore' });
  for (let i = 0; i < 60; i++) { if (await portUp()) break; await new Promise(r => setTimeout(r, 500)); }
  if (!(await portUp())) { console.error('chrome devtools port never came up'); process.exit(1); }
}
process.on('exit', () => { try { chrome && chrome.kill('SIGKILL'); releaseLock(); } catch {} });

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
const upstreamStats = await shoot(UPSTREAM, 'upstream').catch(e => ({ error: String(e) }));
const leptosStats = await shoot(LEPTOS, 'leptos').catch(e => ({ error: String(e) }));
const report = { route, upstreamStats, leptosStats };
try {
  const px = compare(decodePng(fs.readFileSync(`${OUT}/upstream.png`)), decodePng(fs.readFileSync(`${OUT}/leptos.png`)));
  report.pixelDiff = `${px.percent}% pixels differ (${px.diffPixels}/${px.totalPixels})`;
  report.pixelDiffPercent = px.percent;
  fs.writeFileSync(`${OUT}/mask.png`, px.maskPng);
  fs.writeFileSync(`${OUT}/overlay.png`, px.overlayPng);
} catch (e) { report.pixelDiff = String(e); }
fs.writeFileSync(`${OUT}/report.json`, JSON.stringify(report, null, 1));
console.log(JSON.stringify(report, null, 1));
if (chrome) chrome.kill('SIGKILL');
releaseLock();
