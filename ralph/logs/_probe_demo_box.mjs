// Scratch probe (this iteration): why is the port's checkbox demo control full-width (784px) when
// upstream's is 166px? Dump the ancestor chain of the demo's <label> on both sides with the
// computed display/width of every level, so the difference is read off the DOM, not guessed.
//
//   node ralph/logs/_probe_demo_box.mjs react/components/checkbox "Enable notifications"
//
// Same CDP plumbing as ralph/scripts/visual-gap-report.mjs (its own devtools port, so it cannot
// collide with a harness browser that is up).
import { spawn } from 'node:child_process';

const PORT = 9887;
const CHROME = process.env.CHROME || '/data/tools/chrome-wrapper.sh';
const UPSTREAM_BASE = process.env.UPSTREAM_DOCS_BASE || 'http://127.0.0.1:3005';
const LEPTOS_BASE = process.env.LEPTOS_DOCS_BASE || 'http://127.0.0.1:3177';
const route = process.argv[2] || 'react/components/checkbox';
const needle = process.argv[3] || 'Enable notifications';

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
      setTimeout(() => { if (this.pending.has(id)) { this.pending.delete(id); res({ error: { message: `cdp timeout ${method}` } }); } }, 60000);
    });
  }
}

const PROBE = (needle) => `(() => {
  const needle = ${JSON.stringify(needle)};
  const main = document.querySelector('main') || document.body;
  const all = [...main.querySelectorAll('*')];
  const el = all.find((n) => n.textContent.trim() === needle && n.children.length === 0)
    || all.find((n) => n.textContent.includes(needle) && /^(LABEL|DIV|SPAN)$/.test(n.tagName));
  if (!el) return { error: 'not found' };
  const chain = [];
  let n = el;
  while (n && n !== document.documentElement) {
    const cs = getComputedStyle(n);
    const r = n.getBoundingClientRect();
    chain.push({
      tag: n.tagName.toLowerCase(),
      cls: (n.className && n.className.baseVal !== undefined ? n.className.baseVal : n.className || '').toString().slice(0, 120),
      w: Math.round(r.width), h: Math.round(r.height),
      display: cs.display, width: cs.width, minWidth: cs.minWidth, maxWidth: cs.maxWidth,
      alignSelf: cs.alignSelf, flex: cs.flex, flexDirection: cs.flexDirection, padding: cs.padding,
      inlineSize: cs.inlineSize, justify: cs.justifyContent,
    });
    n = n.parentElement;
  }
  return { chain: chain.slice(0, 9) };
})()`;

async function look(base, label) {
  const list = await fetch(`http://127.0.0.1:${PORT}/json/list`).then((r) => r.json());
  let tab = list.find((t) => t.type === 'page');
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((r) => ws.addEventListener('open', r));
  const cdp = new CDP(ws);
  await cdp.send('Page.enable');
  await cdp.send('Runtime.enable');
  await cdp.send('Page.navigate', { url: `${base}/${route}` });
  await new Promise((r) => setTimeout(r, 6000));
  const res = await cdp.send('Runtime.evaluate', { expression: PROBE(needle), returnByValue: true, awaitPromise: false });
  console.log(`\n=== ${label} ${base}/${route}`);
  const v = res.result && res.result.result && res.result.result.value;
  if (!v) { console.log('  no value', JSON.stringify(res).slice(0, 300)); }
  else if (v.error) { console.log('  ' + v.error); }
  else for (const c of v.chain) console.log(`  ${c.tag}.${c.cls}\n      ${c.w}x${c.h} display=${c.display} width=${c.width} maxWidth=${c.maxWidth} alignSelf=${c.alignSelf} flex=${c.flex} dir=${c.flexDirection}`);
  ws.close();
}

const chrome = spawn(CHROME, [`--remote-debugging-port=${PORT}`, ...CHROME_FLAGS, 'about:blank'], { stdio: 'ignore' });
await new Promise((r) => setTimeout(r, 2500));
try {
  await look(UPSTREAM_BASE, 'upstream');
  await look(LEPTOS_BASE, 'leptos');
} finally {
  chrome.kill('SIGKILL');
}
