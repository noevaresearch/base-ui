#!/usr/bin/env node
// visual-diff.mjs — pixel + structural diff between upstream React docs and Leptos docs.
// Screenshots both routes headless, compares via pixel sampling (no deps).
// Usage: CHROME=... node ralph/scripts/visual-diff.mjs --route react/components/checkbox

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { decodePng, compare, encodePng } from './lib/png.mjs';
import { launchChrome, killChrome, FRUGAL_CHROME_FLAGS, openHarnessTab } from './lib/browser.mjs';
import { refuseBrowserWork } from './lib/browser-budget.mjs';
import { WIDGET_REGION_JS, regionParity } from './lib/widget-region.mjs';
import { SNIPPET_LANG_JS } from './lib/snippet-lang.mjs';

// A browser must not be started on this box without an explicit allowance (see lib/browser-budget.mjs). Placed at
// module top, before the launch below, because this script started a real Chromium with no guard at all. It is the
// predecessor of the widget-region scorer `check-visual-budget.mjs` now uses, kept for pixel-level diffs, and its
// lock discipline comment below shows it is reached by hand — the exact shape of "a guard in a file nobody runs":
// the gate that everyone DOES run must not be the one that leaks the 1.4 GB launch. Exit 2 = UNMEASURED.
refuseBrowserWork('visual-diff.mjs', "node ralph/scripts/scorecard-latest.mjs --route react/<kind>/<name>  (the committed CI scorecard; check-visual-budget.mjs is the gate whose numbers are recorded)");

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
  // A REAL page target, opened explicitly: `/json/list[0]` is whatever came first in a list (an orphan
  // Chrome's dead target, on a box that has run this harness before) and navigating a dead target returns
  // an empty DOM with no error — which the scorer then reads as an empty page. See lib/browser.mjs.
  const tab = await openHarnessTab(9888);
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
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
      // The two region rects come from the SHARED probe (ralph/scripts/lib/widget-region.mjs).
      // Why shared — both this script and visual-gap-report.mjs used to carry their own copy of
      // this logic (different selector lists, no code-panel exclusion in one of them), and both
      // cropped each side to its OWN rect and compared only the min-overlap, which scored the
      // port's button against the top-left corner of upstream's demo+source panel. The module
      // documents that measurement and the two rules that make the number a parity:
      //   widgetRect — the component's own rendered control + label (the 97% bar).
      //   demoRect   — the demo container upstream wraps it in (div.demo / DemoRoot /
      //                DemoPlayground; ours is div.docs-demo): its frame/chrome is the
      //                docs-chrome demo-panels work, not the component.
      ...(${WIDGET_REGION_JS}),
      links: m.querySelectorAll('a').length,
      inputs: m.querySelectorAll('input,button').length,
      textLen: m.textContent.replace(/\\\\s+/g, ' ').length,
      // Snippet language: a mirrored page must demonstrate the PORT's API. A page carrying
      // upstream's React source has the right word count and the wrong framework, so it must not
      // score as content parity. The classifier is the SHARED one (ralph/scripts/lib/snippet-lang.mjs),
      // injected as its own source exactly the way WIDGET_REGION_JS above is: the private copy that
      // used to live here carried the bare capitalized-tag React heuristic without the Leptos-exclusive
      // escape, so it scored the port's own idiomatic AccordionRoot markup as React and the block
      // was then excluded from scoring — the defect crates/docs-app/src/snippet_language.rs documents
      // and names this file as a place to fix (fix the classifier, never the page).
      // NOTE: no backticks anywhere inside this template literal — one terminates the string.
      snippets: (${SNIPPET_LANG_JS})([...m.querySelectorAll('pre')].map(p => p.textContent || ''))
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

  // Region-scoped parity over a COMMON crop (lib/widget-region.mjs `regionParity`). The component
  // is the thing that must match closely; page-level distance is dominated by prose and code, which
  // differ by design here. A pair of regions that are not the same kind of thing (a panel vs a
  // control) is reported as a FAULT rather than scored — scoring it is the defect that made this
  // number meaningless (see the module's header).
  const regions = {
    widget: [report.upstreamStats?.widgetRect, report.leptosStats?.widgetRect],
    demo: [report.upstreamStats?.demoRect, report.leptosStats?.demoRect],
  };
  for (const [name, [upRect, lxRect]] of Object.entries(regions)) {
    if (!upRect || !lxRect) {
      report[`${name}Parity`] = null;
      report[`${name}DiffPercent`] = null;
      report[`${name}Fault`] = null;
      report[`${name}Rect`] = { upstream: upRect || null, leptos: lxRect || null };
      continue;
    }
    const region = regionParity(upImg, lxImg, upRect, lxRect);
    report[`${name}DiffPercent`] = region.diffPercent;
    report[`${name}Parity`] = region.parity;
    report[`${name}Fault`] = region.fault;
    report[`${name}Rect`] = region.rects;
    report[`${name}CommonCrop`] = region.commonCrop;
    if (region.comparable) {
      // Write the crops that were actually compared — not a re-crop, which is how the two callers
      // drifted apart before.
      fs.writeFileSync(`${OUT}/upstream-${name}.png`, encodePng(region.crops.upstream.width, region.crops.upstream.height, region.crops.upstream.data));
      fs.writeFileSync(`${OUT}/leptos-${name}.png`, encodePng(region.crops.leptos.width, region.crops.leptos.height, region.crops.leptos.data));
    }
  }
} catch (e) { report.pixelDiff = String(e); report.componentParity = null; }
fs.writeFileSync(`${OUT}/report.json`, JSON.stringify(report, null, 1));
console.log(JSON.stringify(report, null, 1));
killChrome(chrome);
releaseLock();
