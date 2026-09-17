#!/usr/bin/env node
// cdp-diff.mjs — differential check: Leptos docs page vs upstream React docs page.
// Drives Chrome for Testing over raw CDP (no deps; Node 22 has WebSocket).
// Usage: node ralph/scripts/playwright-diff.mjs --todo-id "docs-content: components/checkbox"
//   [--route react/components/checkbox] [--upstream http://localhost:3005/react/components/checkbox]
//   [--leptos http://localhost:3177/react/components/checkbox]
// Exit 0 = pass, 1 = fail (prints a JSON report), 2 = UNMEASURED — this box refused the browser
// (see lib/browser-budget.mjs); a refusal is never a failed differential.

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import { launchChrome, killChrome, FRUGAL_CHROME_FLAGS } from './lib/browser.mjs';
import { refuseBrowserWork } from './lib/browser-budget.mjs';
import os from 'node:os';
import path from 'node:path';

// A browser must not be started on this box without an explicit allowance (see lib/browser-budget.mjs), and the
// guard has to sit where launchChrome is actually reached — module top, before the launch below. WHY IT WAS
// MISSING HERE, MEASURED: this script is spawned by `check-page.mjs` (route-shaped ids) and by
// `run-regression.sh`'s docs-pair step, and it was the only launcher `check-page` drives that had no guard, so a
// whole-page run started a real Chromium (~1.4 GB against this box's 4096 MB cgroup) and the `structure` axis came
// back a live PASS while the five sibling axes honestly reported UNMEASURED — a false green in the number used to
// track progress, produced by the one check that was supposed to be the scorecard's cheapest axis.
// Exit 2 is this harness's UNMEASURED contract (never "the differential failed"); CI names its allowance with
// RALPH_BROWSER_GATES=1 and still runs this check, so the measurement is deferred, not lost.
refuseBrowserWork('playwright-diff.mjs', "node ralph/scripts/scorecard-latest.mjs --route react/<kind>/<name>  (the committed CI scorecard, whose `structure` axis IS this check)");

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
const CHROMEDRIVER = process.env.CHROMEDRIVER || '/data/tools/chromedriver-wrapper.sh';

// ---- args ----
function arg(name, dflt) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : dflt;
}
let todoId = arg('todo-id', '');
const LEPTOS = arg('leptos', '');
const UPSTREAM = arg('upstream', '');

// Derive route from todo id: "docs-content: components/checkbox" -> react/components/checkbox
if (!LEPTOS && todoId) {
  const m = todoId.match(/components\/([a-z0-9-]+)/i);
  if (!m) { console.error(`cannot derive route from todo-id "${todoId}"`); process.exit(1); }
  var route = `react/components/${m[1]}`;
}
const routeArg = arg('route', LEPTOS ? new URL(LEPTOS).pathname.slice(1) : route);

const leptosUrl = LEPTOS || `http://127.0.0.1:3177/${routeArg}`;
const upstreamUrl = UPSTREAM || `http://127.0.0.1:3005/${routeArg}`;

// ---- CDP plumbing ----
function httpGetJson(url) {
  return fetch(url).then(r => r.json());
}
async function newTab(port) {
  for (let i = 0; i < 20; i++) {
    try {
      const r = await fetch(`http://127.0.0.1:${port}/json/new?about:blank`, { method: 'PUT' });
      const text = await r.text();
      if (!text.trim().startsWith('Using unsa')) return JSON.parse(text);
    } catch {}
    await new Promise(r => setTimeout(r, 250));
  }
  throw new Error('could not open new CDP tab');
}
class CDP {
  constructor(ws) { this.ws = ws; this.id = 0; this.pending = new Map(); this.handlers = [];
    ws.addEventListener('message', (ev) => {
      const m = JSON.parse(ev.data);
      if (m.id && this.pending.has(m.id)) { const {resolve, reject} = this.pending.get(m.id); this.pending.delete(m.id);
        m.error ? reject(new Error(m.error.message)) : resolve(m.result); }
      else if (m.method) this.handlers.forEach(h => h(m));
    });
  }
  send(method, params = {}) {
    const id = ++this.id;
    return new Promise((resolve, reject) => {
      this.pending.set(id, {resolve, reject});
      this.ws.send(JSON.stringify({id, method, params}));
      setTimeout(() => { if (this.pending.has(id)) { this.pending.delete(id); reject(new Error('cdp timeout ' + method)); } }, 45000);
    });
  }
  on(fn) { this.handlers.push(fn); }
}
async function connectToTab(tab) {
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  return new CDP(ws);
}
// wait for predicate over Runtime.evaluate
async function waitFor(cdp, expr, timeoutMs = 30000) {
  const t0 = Date.now();
  for (;;) {
    const r = await cdp.send('Runtime.evaluate', { expression: expr, returnByValue: true });
    if (r.result && r.result.value) return r.result.value;
    if (Date.now() - t0 > timeoutMs) return false;
    await new Promise(r => setTimeout(r, 300));
  }
}

// ---- extraction ----
const SNAPSHOT_JS = `(() => {
  const main = document.querySelector('main') || document.body;
  const norm = (el) => {
    const out = [];
    const walk = (n) => {
      if (n.nodeType === 3) { const t = n.textContent.replace(/\\s+/g, ' ').trim(); if (t) out.push('#text:' + t); return; }
      if (n.nodeType !== 1) return;
      const tag = n.tagName.toLowerCase();
      const cls = typeof n.className === 'string' ? n.className.trim().split(/\\s+/).filter(Boolean).sort().join('.') : '';
      out.push('<' + tag + (cls ? ' ' + cls : '') + '>');
      for (const c of n.children) walk(c);
    };
    walk(main);
    return out;
  };
  return { headings: [...main.querySelectorAll('h1,h2,h3')].map(h => h.textContent.trim()), tree: norm() };
})()`;

// ---- chrome/chromedriver lifecycle ----
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'cdp-diff-'));
const releaseLock = acquireBrowserLock();
const { child: chrome } = await launchChrome(FRUGAL_CHROME_FLAGS, { tmpDir: tmp, port: 9777, waitMs: 45000 });
process.on('exit', () => { try { killChrome(chrome); releaseLock(); fs.rmSync(tmp, {recursive: true, force: true}); } catch {} });

async function main() {
  // wait for devtools port
  let port = 9777;
  for (let i = 0; i < 40; i++) {
    try { await httpGetJson(`http://127.0.0.1:${port}/json/version`); break; } catch { await new Promise(r => setTimeout(r, 250)); }
  }
  const report = { todoId, route: routeArg, leptosUrl, upstreamUrl, pass: false, checks: {} };

  // --- Leptos side ---
  const tab1 = await newTab(port);
  const cdp1 = await connectToTab(tab1);
  await cdp1.send('Page.enable');
  await cdp1.send('Page.navigate', { url: leptosUrl });
  const mounted = await waitFor(cdp1, `document.body.children.length > 0 && (document.querySelector('main,h1') !== null)`, 45000);
  report.checks.leptosMounted = !!mounted;
  if (mounted) {
    // let hydration/effects settle
    await new Promise(r => setTimeout(r, 1500));
    const snap = await cdp1.send('Runtime.evaluate', { expression: SNAPSHOT_JS, returnByValue: true });
    report.leptos = snap.result.value;
  }
  await cdp1.send('Page.close').catch(() => {});

  // --- Upstream React side (optional; pass without it = structure-only mode) ---
  if (UPSTREAM || process.env.DIFF_UPSTREAM === '1') {
    try {
      const tab2 = await newTab(port);
      const cdp2 = await connectToTab(tab2);
      await cdp2.send('Page.enable');
      await cdp2.send('Page.navigate', { url: upstreamUrl });
      const upMounted = await waitFor(cdp2, `document.body.children.length > 0 && (document.querySelector('main,h1') !== null)`, 45000);
      report.checks.upstreamMounted = !!upMounted;
      if (upMounted) {
        await new Promise(r => setTimeout(r, 1500));
        const snap = await cdp2.send('Runtime.evaluate', { expression: SNAPSHOT_JS, returnByValue: true });
        report.upstream = snap.result.value;
      }
      await cdp2.send('Page.close').catch(() => {});
    } catch (e) { report.checks.upstreamError = String(e.message || e); }
  }

  // --- verdicts ---
  if (report.leptos) {
    report.checks.hasH1 = report.leptos.headings.length > 0 && report.leptos.headings[0].length > 0;
    report.checks.nonEmptyTree = report.leptos.tree.length > 10;
    if (report.upstream && report.upstream.headings) {
      const lh = report.leptos.headings.map(h => h.toLowerCase());
      const uh = report.upstream.headings.map(h => h.toLowerCase());
      report.checks.headingsSubset = uh.filter(h => lh.includes(h)).length / Math.max(uh.length, 1);
      report.checks.pass = report.checks.leptosMounted && report.checks.hasH1 &&
        report.checks.nonEmptyTree && report.checks.headingsSubset >= 0.8;
    } else {
      report.checks.pass = report.checks.leptosMounted && report.checks.hasH1 && report.checks.nonEmptyTree;
    }
  }
  report.pass = !!report.checks.pass;
  console.log(JSON.stringify(report, null, 1));
  killChrome(chrome);
  releaseLock();
  process.exit(report.pass ? 0 : 1);
}
main().catch(e => { console.error('fatal:', e); killChrome(chrome); process.exit(1); });
