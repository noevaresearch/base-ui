#!/usr/bin/env node
// visual-diff.mjs — pixel + structural diff between upstream React docs and Leptos docs.
// Screenshots both routes headless, compares via pixel sampling (no deps).
// Usage: CHROME=... node ralph/scripts/visual-diff.mjs --route react/components/checkbox

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { decodePng, compare, crop, encodePng } from './lib/png.mjs';
import { launchChrome, killChrome, FRUGAL_CHROME_FLAGS } from './lib/browser.mjs';

// Shared resource discipline for this box: a 512-task cgroup cap is shared with the Hermes
// gateway, the Ralph loop and cargo builds, so a default Chrome launch (~20 procs, 100+
// threads) can starve the whole box of forks. One renderer, no zygote, and a lock so only one
// harness browser is alive at a time.
//
// THE LOCK IS HELD FOR THE WHOLE RUN, not only for the launch: `tabs[0]` (see `shoot`) is shared
// state, so two harness processes that both find the devtools port already up take the same lock
// never and drive the same tab — measured 2026-09-15, when two overlapping budget runs made the
// checkbox route report the meter route's numbers (score 71.29 / visual 93.12 / content 38.54,
// byte-equal to meter's line in the other run), which is exactly the "credit for parity that was
// never measured" class the caller's guards exist to stop. Serialising here is what makes a
// measurement attributable to one route at a time.
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
try {
  const launched = await launchChrome(FRUGAL_CHROME_FLAGS, { tmpDir: tmp, port: 9888, waitMs: 45000 });
  chrome = launched.child;
} catch (e) {
  console.error(String(e.message || e));
  process.exit(1);
}
process.on('exit', () => { try { killChrome(chrome); releaseLock(); } catch {} });

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
      href: location.href,
      // The component region: the union of this page's rendered component parts, excluding the
      // docs chrome (sidebar/header). Parity is judged separately here — the prose legitimately
      // differs (Leptos snippets vs upstream's React), the component must not.
      // Two regions, because "the component" and "the component's frame" are different things:
      //   widgetRect — the component's own rendered control + label: this is what must look
      //                95-99% identical (same visual result, different framework).
      //   demoRect   — the demo container upstream wraps it in (div.demo / DemoRoot /
      //                DemoPlayground; ours is div.docs-demo): its frame/chrome is the
      //                docs-chrome demo-panels work, not the component.
      // Upstream has no <article> (main > div.QuickNavContainer > div.QuickNavContent), so a
      // structural walk from a scope element finds nothing there — the container class is the
      // reliable handle on both sides.
      widgetRect: (() => {
        const main = m;
        const demo = main.querySelector('[class~=demo], .DemoRoot, .DemoPlayground, .docs-demo, [class*=demo]');
        const scope = demo || main;
        const parts = [...scope.querySelectorAll('label,input:not([type=hidden]),[role=checkbox],[role=switch],[role=slider],[role=radio],[role=combobox],[role=listbox],[role=tab],select,textarea,button')]
          .filter((el) => !el.closest('nav,aside,header') && el.getClientRects().length > 0);
        let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
        for (const el of parts) {
          const b = el.getBoundingClientRect();
          if (b.width < 2 || b.height < 2) continue;
          x0 = Math.min(x0, b.x); y0 = Math.min(y0, b.y);
          x1 = Math.max(x1, b.x + b.width); y1 = Math.max(y1, b.y + b.height);
        }
        if (!Number.isFinite(x0)) return null;
        const pad = 8;
        return { x: Math.max(0, Math.round(x0 - pad)), y: Math.max(0, Math.round(y0 - pad)),
                 w: Math.round(x1 - x0 + pad * 2), h: Math.round(y1 - y0 + pad * 2), parts: parts.length };
      })(),
      demoRect: (() => {
        const demo = m.querySelector('[class~=demo], .DemoRoot, .DemoPlayground, .docs-demo, [class*=demo]');
        if (!demo) return null;
        const b = demo.getBoundingClientRect();
        if (b.width < 20 || b.height < 10) return null;
        return { x: Math.max(0, Math.round(b.x)), y: Math.max(0, Math.round(b.y)), w: Math.round(b.width), h: Math.round(b.height),
                 cls: (typeof demo.className === 'string' ? demo.className : '').slice(0, 60) };
      })(),
      links: m.querySelectorAll('a').length,
      inputs: m.querySelectorAll('input,button').length,
      textLen: m.textContent.replace(/\\\\s+/g, ' ').length,
      // Snippet language: a mirrored page must demonstrate the PORT's API. A page carrying
      // upstream's React source has the right word count and the wrong framework, so it must not
      // score as content parity. Backslashes are doubled — this sits inside a template literal.
      snippets: (() => {
        const texts = [...m.querySelectorAll('pre')].map(p => p.textContent || '');
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
        const out = { total: texts.length, leptos: 0, react: 0, other: 0 };
        for (const t of texts) {
          const r = looksReact(t), l = looksLeptos(t);
          if (l && !r) out.leptos++;
          else if (r) out.react++;
          else out.other++;
        }
        return out;
      })()
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
  const upImg = decodePng(fs.readFileSync(`${OUT}/upstream.png`));
  const lxImg = decodePng(fs.readFileSync(`${OUT}/leptos.png`));
  const px = compare(upImg, lxImg);
  report.pixelDiff = `${px.percent}% pixels differ (${px.diffPixels}/${px.totalPixels})`;
  report.pixelDiffPercent = px.percent;
  fs.writeFileSync(`${OUT}/mask.png`, px.maskPng);
  fs.writeFileSync(`${OUT}/overlay.png`, px.overlayPng);

  // Region-scoped parity: crop each side to its OWN component rect and compare those crops. The
  // component is the thing that must match closely (>=95%); page-level distance is dominated by
  // prose and code, which differ by design here.
  const regions = {
    widget: [report.upstreamStats?.widgetRect, report.leptosStats?.widgetRect],
    demo: [report.upstreamStats?.demoRect, report.leptosStats?.demoRect],
  };
  for (const [name, [upRect, lxRect]] of Object.entries(regions)) {
    const up = upRect && lxRect ? upRect : null;
    if (!upRect || !lxRect) {
      report[`${name}Parity`] = null;
      report[`${name}DiffPercent`] = null;
      report[`${name}Rect`] = { upstream: upRect || null, leptos: lxRect || null };
      continue;
    }
    const upCrop = crop(upImg, upRect);
    const lxCrop = crop(lxImg, lxRect);
    const r = compare(upCrop, lxCrop);
    report[`${name}DiffPercent`] = r.percent;
    report[`${name}Parity`] = Number((100 - r.percent).toFixed(2));
    report[`${name}Rect`] = { upstream: upRect, leptos: lxRect };
    fs.writeFileSync(`${OUT}/upstream-${name}.png`, encodePng(upCrop.width, upCrop.height, upCrop.data));
    fs.writeFileSync(`${OUT}/leptos-${name}.png`, encodePng(lxCrop.width, lxCrop.height, lxCrop.data));
  }
} catch (e) { report.pixelDiff = String(e); report.componentParity = null; }
fs.writeFileSync(`${OUT}/report.json`, JSON.stringify(report, null, 1));
console.log(JSON.stringify(report, null, 1));
killChrome(chrome);
releaseLock();
