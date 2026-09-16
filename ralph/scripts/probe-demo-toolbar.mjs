// Demo-chrome probe: what does the demo toolbar/panel actually render on each side?
//
//   node ralph/scripts/probe-demo-toolbar.mjs --route react/components/checkbox
//   node ralph/scripts/probe-demo-toolbar.mjs --route react/components/button --json /tmp/tb.json
//
// Reports, per demo container, the observable surface of upstream's Demo chrome
// (docs/src/components/Demo/Demo.tsx): the bordered panel pair (DemoPlayground /
// DemoPlaygroundInner), the file tabs (DemoFileSelector -> Tabs.Root, role=tab), the
// styling-method variant selector (aria-label="Styling method"), the external playground
// link (aria-label="Open in StackBlitz"/"Open in CodeSandbox"), the more-actions menu
// (aria-label="More actions" -> "View source on GitHub" / "Copy link to source"), and the
// code panel's own chrome (Copy code button, Show code/Hide code trigger, <pre> count).
//
// It is deliberately a SURFACE inventory, not a score: it answers "which of upstream's
// controls exist here at all", which is the question the ledger's done-when clauses need
// and which no pixel or text-recall metric can answer.
//
// Own devtools port, so it cannot collide with a harness browser that is up.
import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';

const PORT = Number(process.env.PROBE_CDP_PORT || 9889);
const CHROME = process.env.CHROME || '/data/tools/chrome-wrapper.sh';
const UPSTREAM_BASE = process.env.UPSTREAM_DOCS_BASE || 'http://127.0.0.1:3005';
const LEPTOS_BASE = process.env.LEPTOS_DOCS_BASE || 'http://127.0.0.1:3177';
// Upstream's heavier routes (accordion: 3 demos, each with a full source listing) need longer than
// the first version's 7s settle + 60s evaluate: measured 2026-09-16, accordion died with
// `cdp timeout Runtime.evaluate`, i.e. the probe reported nothing rather than a wrong number.
// Both windows are therefore configurable, and the per-demo collection is capped so a page with
// many demos cannot make the payload itself the timeout.
const EVAL_TIMEOUT_MS = Number(process.env.PROBE_EVAL_TIMEOUT_MS || 150000);
const DEFAULT_SETTLE_MS = Number(process.env.PROBE_SETTLE_MS || 9000);
const MAX_DEMOS = Number(process.env.PROBE_MAX_DEMOS || 8);

const argv = process.argv.slice(2);
const arg = (name, dflt) => {
  const i = argv.indexOf('--' + name);
  return i >= 0 && argv[i + 1] ? argv[i + 1] : dflt;
};
const route = arg('route', 'react/components/checkbox');
const jsonOut = arg('json', '');
const sides = [
  { name: 'upstream', base: UPSTREAM_BASE, sel: '.DemoRoot' },
  { name: 'leptos', base: LEPTOS_BASE, sel: '.docs-demo, .docs-demo-with-source' },
];

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
      setTimeout(() => { if (this.pending.has(id)) { this.pending.delete(id); res({ error: { message: 'cdp timeout ' + method } }); } }, EVAL_TIMEOUT_MS);
    });
  }
}

// No backticks and no template literals in the browser-side source (a stray backtick
// terminates the outer template literal); string concat only.
const BROWSER_FN = (sel) => '(() => {\n' +
  '  const SEL = ' + JSON.stringify(sel) + ';\n' +
  '  const cls = (n) => (n.className && n.className.baseVal !== undefined ? n.className.baseVal : n.className || "").toString().slice(0, 70);\n' +
  '  const describe = (c, i) => {\n' +
  '    const r = c.getBoundingClientRect();\n' +
  '    const figures = [...c.querySelectorAll("[role=figure]")];\n' +
  '    const tabs = [...c.querySelectorAll("[role=tab]")];\n' +
  '    const ctrls = [...c.querySelectorAll("button, a[href], [role=combobox], select, [role=menuitem]")];\n' +
  '    return {\n' +
  '      i: i, cls: cls(c), w: Math.round(r.width), h: Math.round(r.height),\n' +
  '      figures: figures.map((f) => ({ label: f.getAttribute("aria-label"), cls: cls(f), w: Math.round(f.getBoundingClientRect().width), h: Math.round(f.getBoundingClientRect().height) })),\n' +
  '      tabCount: tabs.length,\n' +
  '      tabLabels: tabs.map((t) => (t.textContent || "").trim()),\n' +
  '      controls: ctrls.map((el) => ({ tag: el.tagName.toLowerCase(), role: el.getAttribute("role") || "", aria: el.getAttribute("aria-label") || "", text: (el.textContent || "").trim().slice(0, 26), href: (el.getAttribute("href") || "").slice(0, 64), cls: cls(el) })),\n' +
  '      preCount: c.querySelectorAll("pre").length,\n' +
  '      copyCodeButton: !!c.querySelector("[aria-label=\'Copy code\']"),\n' +
  '      collapseTriggerText: /Hide code/.test(c.textContent) ? "Hide code" : (/Show code/.test(c.textContent) ? "Show code" : null),\n' +
  '      stylingMethodSelect: !!c.querySelector("[aria-label=\'Styling method\']"),\n' +
  '      playgroundLink: !!c.querySelector("[aria-label=\'Open in StackBlitz\'], [aria-label=\'Open in CodeSandbox\']"),\n' +
  '      moreActionsMenu: !!c.querySelector("[aria-label=\'More actions\']"),\n' +
  '    };\n' +
  '  };\n' +
  '  const nodes = [...document.querySelectorAll(SEL)];\n' +
  '  const bodyText = document.body.innerText || "";\n' +
  '  return {\n' +
  '    url: location.href,\n' +
  '    selector: SEL,\n' +
  '    count: nodes.length,\n' +
  '    demos: nodes.slice(0, ' + MAX_DEMOS + ').map(describe),\n' +
  '    pageWant: {\n' +
  '      stylingMethod: document.querySelectorAll("[aria-label=\'Styling method\']").length,\n' +
  '      stackBlitzText: (bodyText.match(/StackBlitz/g) || []).length,\n' +
  '      viewSourceText: (bodyText.match(/View source on GitHub/g) || []).length,\n' +
  '      copySourceText: (bodyText.match(/Copy link to source/g) || []).length,\n' +
  '      fileTabTotal: document.querySelectorAll("[role=tab]").length,\n' +
  '      demoFigures: document.querySelectorAll("[role=figure]").length,\n' +
  '    },\n' +
  '  };\n' +
  '})()';

async function look(side) {
  const list = await fetch('http://127.0.0.1:' + PORT + '/json/list').then((r) => r.json());
  const tab = list.find((t) => t.type === 'page');
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((r) => ws.addEventListener('open', r));
  const cdp = new CDP(ws);
  await cdp.send('Page.enable');
  await cdp.send('Runtime.enable');
  const target = side.base + '/' + route;
  const settle = Number(arg('wait', String(DEFAULT_SETTLE_MS)));

  // A navigation that does not land on the requested origin must NEVER be scored: measured
  // 2026-09-16, one run evaluated the LEPTOS selector against the UPSTREAM document (the port's
  // dev server was being rebuilt by a concurrent iteration, so `Page.navigate` did not take), and
  // the printed numbers looked like a legitimate reading of the wrong side. Confirm the document
  // origin before extracting, retry the navigation, and fail loudly as an ERROR if it never lands.
  let href = '';
  let navError = '';
  for (let attempt = 1; attempt <= 3; attempt += 1) {
    const nav = await cdp.send('Page.navigate', { url: target });
    navError = (nav && nav.error && nav.error.message) || (nav && nav.result && nav.result.errorText) || '';
    await new Promise((r) => setTimeout(r, settle));
    const h = await cdp.send('Runtime.evaluate', { expression: 'location.href', returnByValue: true });
    href = (h.result && h.result.result && h.result.result.value) || '';
    if (href.indexOf(side.base + '/') === 0) {
      break;
    }
    console.log('  [navigate retry ' + attempt + '] ' + side.name + ' landed on ' + JSON.stringify(href) +
      ' instead of ' + target + (navError ? ' (nav error: ' + navError + ')' : ''));
  }
  if (href.indexOf(side.base + '/') !== 0) {
    ws.close();
    return {
      side: side.name,
      base: side.base,
      error: 'navigation never reached ' + target + ' (last document ' + JSON.stringify(href) + ')',
    };
  }

  const res = await cdp.send('Runtime.evaluate', { expression: BROWSER_FN(side.sel), returnByValue: true, awaitPromise: false });
  ws.close();
  const v = res.result && res.result.result && res.result.result.value;
  if (!v) return { side: side.name, base: side.base, error: JSON.stringify(res).slice(0, 400) };
  return Object.assign({ side: side.name, base: side.base }, v);
}

const chrome = spawn(CHROME, ['--remote-debugging-port=' + PORT, ...CHROME_FLAGS, 'about:blank'], { stdio: 'ignore' });
await new Promise((r) => setTimeout(r, 2500));
const results = [];
try {
  for (const s of sides) results.push(await look(s));
} finally {
  chrome.kill('SIGKILL');
}

for (const r of results) {
  console.log('\n================ ' + r.side + '  ' + (r.url || r.base + '/' + route));
  if (r.error) { console.log('  ERROR ' + r.error); continue; }
  console.log('  demo containers (' + r.selector + '): ' + r.count);
  console.log('  page-wide: ' + JSON.stringify(r.pageWant));
  for (const d of r.demos.slice(0, 4)) {
    console.log('  --- demo[' + d.i + '] .' + d.cls + '  ' + d.w + 'x' + d.h);
    console.log('      figures: ' + JSON.stringify(d.figures));
    console.log('      tabs(' + d.tabCount + '): ' + JSON.stringify(d.tabLabels));
    console.log('      controls(' + d.controls.length + '): ' + JSON.stringify(d.controls.slice(0, 10)));
    console.log('      pre=' + d.preCount + ' copyCodeBtn=' + d.copyCodeButton + ' collapse=' + d.collapseTriggerText +
      ' stylingSelect=' + d.stylingMethodSelect + ' playgroundLink=' + d.playgroundLink + ' moreActions=' + d.moreActionsMenu);
  }
}

if (jsonOut) {
  writeFileSync(jsonOut, JSON.stringify({ route, results }, null, 1));
  console.log('\nwrote ' + jsonOut);
}
