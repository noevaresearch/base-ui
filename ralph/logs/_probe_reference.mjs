// Scratch probe (this iteration): what does each side's `main` actually contain, and which of
// upstream's generated-reference rows are VISIBLE at 1280px? The gap report measures shape and
// looks; this reads the oracle's own DOM before the ported reference section is written, so the
// ported markup is copied from upstream's render instead of guessed.
//
//   node ralph/logs/_probe_reference.mjs react/components/button
//
// Zero-dep, same CDP plumbing as ralph/scripts/visual-gap-report.mjs (its browser lock is shared
// so this cannot run while a harness browser is up).
import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const PORT = 9887;
const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const CHROME = process.env.CHROME || '/data/tools/chrome-wrapper.sh';
const UPSTREAM_BASE = process.env.UPSTREAM_DOCS_BASE || 'http://127.0.0.1:3005';
const LEPTOS_BASE = process.env.LEPTOS_DOCS_BASE || 'http://127.0.0.1:3177';
const route = process.argv[2] || 'react/components/button';
const MODE = process.argv[3] || 'shape';

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

// Computed-style dump for the selectors the ported reference section must match, measured off
// upstream's own render at 1280px. Only non-default values are printed, plus the element's rect.
const STYLE_SELECTORS = [
  'h2#api-reference',
  'section.ReferenceAccordionRoot',
  '.ReferenceHeaderRow',
  '.AccordionHeaderCell',
  '.AccordionHeaderCellInner',
  '.ReferenceHeaderTypeCell',
  '.ReferenceHeaderDefaultCell',
  '.ReferenceHeaderIconCell',
  'summary.ReferenceTrigger',
  '.ReferenceNameCell',
  '.ReferenceNameCell .AccordionScrollableInner',
  '.ReferenceTypeCell',
  '.ReferenceDefaultCell',
  '.ReferenceIconWrap',
  '.ReferenceIcon',
  '.AccordionPanel',
  '.AccordionContent',
  'dl.ReferenceContent',
  '.DescriptionListItem',
  '.DescriptionTerm',
  '.DescriptionListDetails',
  '.DescriptionListInner',
  '.ReferenceDescription',
  '.ReferenceTableRoot',
  'table.TableRootTable',
  'thead.TableHead',
  'tbody.TableBody',
  '.TableColumnHeader',
  '.TableCell',
  '.TableCellInner',
  '.ReferenceCompactPanel',
  'details.AccordionItem',
];

const STYLE_PROPS = [
  'display', 'gridTemplateColumns', 'gap', 'columnGap', 'rowGap', 'alignItems', 'justifyContent',
  'minHeight', 'height', 'width', 'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
  'marginTop', 'marginBottom', 'borderTop', 'borderBottom', 'borderRadius', 'fontSize',
  'lineHeight', 'fontWeight', 'fontFamily', 'color', 'backgroundColor', 'overflowX', 'overflowY',
  'whiteSpace', 'position', 'boxSizing', 'flexDirection', 'clipPath', 'textOverflow',
];

const STYLE_PROBE = `(() => {
  const m = document.querySelector('main') || document.body;
  const skip = new Set(['none', 'normal', 'auto', 'visible', 'rgba(0, 0, 0, 0)', '0px', 'static', 'start', 'stretch']);
  const out = {};
  for (const sel of ${JSON.stringify(STYLE_SELECTORS)}) {
    const el = m.querySelector(sel);
    if (!el) { out[sel] = null; continue; }
    const cs = getComputedStyle(el);
    const o = {};
    for (const p of ${JSON.stringify(STYLE_PROPS)}) {
      const v = cs[p];
      if (v && !skip.has(v)) o[p] = v;
    }
    const r = el.getBoundingClientRect();
    o._tag = el.tagName;
    o._rect = [Math.round(r.x), Math.round(r.y), Math.round(r.width), Math.round(r.height)];
    o._visible = cs.display !== 'none' && cs.visibility !== 'hidden' && r.width > 0;
    out[sel] = o;
  }
  const visibleRows = [...m.querySelectorAll('details.AccordionItem')].map((d) => {
    const r = d.getBoundingClientRect();
    return [d.querySelector('summary')?.id || d.textContent.trim().slice(0, 24), Math.round(r.y), Math.round(r.height), getComputedStyle(d).display];
  });
  out['_detailsRows'] = visibleRows;
  const attrSection = m.querySelectorAll('section.ReferenceAccordionRoot')[1];
  const attrTable = m.querySelector('.ReferenceTableRoot');
  out['_dataAttrRepresentations'] = {
    accordionSection: attrSection ? getComputedStyle(attrSection).display : null,
    table: attrTable ? getComputedStyle(attrTable).display : null,
  };
  return JSON.stringify(out, null, 1);
})()`;

const PROBE = MODE === 'styles' ? STYLE_PROBE : `(() => {
  const m = document.querySelector('main') || document.body;
  const info = (sel) => {
    const el = m.querySelector(sel);
    if (!el) return null;
    const cs = getComputedStyle(el);
    const r = el.getBoundingClientRect();
    return {
      display: cs.display, visibility: cs.visibility, overflow: cs.overflow,
      rect: [Math.round(r.x), Math.round(r.y), Math.round(r.width), Math.round(r.height)],
      childCount: el.children.length,
    };
  };
  const api = document.getElementById('api-reference');
  const ar = api ? api.getBoundingClientRect() : null;
  return JSON.stringify({
    main: { tag: m.tagName, class: m.className },
    counts: {
      pre: m.querySelectorAll('pre').length,
      code: m.querySelectorAll('code').length,
      preOrCode: m.querySelectorAll('pre,code').length,
      table: m.querySelectorAll('table').length,
      tr: m.querySelectorAll('tr').length,
      links: m.querySelectorAll('a').length,
      headings: m.querySelectorAll('h1,h2,h3').length,
      demoish: m.querySelectorAll('[class*=demo]').length,
      details: m.querySelectorAll('details').length,
    },
    textLen: m.textContent.replace(/\\s+/g, ' ').length,
    headings: [...m.querySelectorAll('h1,h2,h3')].map((h) => h.textContent.trim()),
    apiInsideMain: api ? m.contains(api) : null,
    apiTop: ar ? Math.round(ar.y) : null,
    docHeight: document.documentElement.scrollHeight,
    visible: {
      propsHeaderRow: info('.ReferenceHeaderRow'),
      referenceNameCell: info('.ReferenceNameCell'),
      referenceTypeCell: info('.ReferenceTypeCell'),
      referenceDefaultCell: info('.ReferenceDefaultCell'),
      referenceIconWrap: info('.ReferenceIconWrap'),
      accordionItem: info('.AccordionItem'),
      dataAttrTableRoot: info('.ReferenceTableRoot'),
      tableRootTable: info('table.TableRootTable'),
      additionalTypeWrapper: info('.AdditionalTypeWrapper'),
      additionalTypeCodeBlock: info('.AdditionalTypeCodeBlock'),
    },
    referenceAccordionRoots: m.querySelectorAll('.ReferenceAccordionRoot').length,
    headerRows: m.querySelectorAll('.ReferenceHeaderRow, .AccordionHeaderRow').length,
    dataAttrAccordionHeaderText: (() => {
      const rows = [...m.querySelectorAll('.AccordionHeaderRow')];
      return rows.map((r) => r.textContent.trim());
    })(),
    summaryOuter: (() => {
      const s = m.querySelector('summary.ReferenceTrigger');
      return s ? s.outerHTML.slice(0, 1400) : null;
    })(),
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
  // Wait for THIS navigation to commit AND for the route to mount. The port's page is a wasm boot
  // loader that renders client-side, so "main h1" only appears after the app starts. Reusing one
  // tab for both sides read the OLD document (measured: the LEPTOS probe once returned upstream's
  // numbers verbatim), which is why each side gets its own tab, as the harness does.
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
  if (!last.startsWith('true|complete|true')) {
    console.error(`[${label}] waiting for ${base}/${route}: last=${{ last }} → after 50s, observed "${last}"`);
  }
  await new Promise((r) => setTimeout(r, 2500));
  const r = await cdp.send('Runtime.evaluate', { expression: PROBE, returnByValue: true });
  if (r.result?.exceptionDetails) {
    console.error(`[${label}] probe threw:`, JSON.stringify(r.result.exceptionDetails).slice(0, 500));
    return null;
  }
  return JSON.parse(r.result.result.value);
}

const userDataDir = fs.mkdtempSync('/tmp/probe-ref-');
const chrome = spawn(CHROME, [...CHROME_FLAGS, `--user-data-dir=${userDataDir}`, `--remote-debugging-port=${PORT}`, 'about:blank'], { stdio: 'ignore' });
try {
  for (let i = 0; i < 40; i++) {
    const r = spawnSync('curl', ['-s', '-o', '/dev/null', '-w', '%{http_code}', `http://127.0.0.1:${PORT}/json/version`], { encoding: 'utf8' });
    if ((r.stdout || '').trim() === '200') break;
    await new Promise((res) => setTimeout(res, 500));
  }
  const cdpUp = await connect();
  const cdpLx = await connect();

  for (const [base, label, cdp] of [[UPSTREAM_BASE, 'UPSTREAM', cdpUp], [LEPTOS_BASE, 'LEPTOS', cdpLx]]) {
    const out = await shoot(cdp, base, label);
    console.log(`\n===== ${label} ${base}/${route} =====`);
    console.log(out === null ? '(probe failed)' : JSON.stringify(out, null, 1));
  }
} finally {
  chrome.kill('SIGKILL');
  spawnSync('sleep', ['1']);
  try { fs.rmSync(userDataDir, { recursive: true, force: true }); } catch {}
}
