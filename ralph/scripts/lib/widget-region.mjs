// widget-region.mjs — ONE definition of "the component's own rendered region" and of how two
// sides' regions are compared. Shared by `visual-diff.mjs` (which produces the report the fidelity
// gate scores) and `visual-gap-report.mjs` (which names the gaps), because the two scripts each
// carried their own copy of this logic with different selector lists and no shared test — one
// measured number, two computations (the snippet-language module has the same rule: share the
// classifier, never copy it per caller).
//
// Why the region needed redefining at all (measured 2026-09-16):
//   * UPSTREAM'S DOM has four nested demo containers — `div.demo` > `div.DemoRoot` >
//     `div.DemoPlayground` > `div.DemoPlaygroundInner` — and only the INNERMOST is the demo's own
//     rendering area; its siblings (`div.DemoRoot > ... > div.DemoCodeBlock`) hold the source panel
//     with the file tabs and the code. `querySelector('[class*=PlaygroundInner], .docs-demo,
//     [class~=demo], .DemoRoot, [class*=demo]')` therefore did NOT do what it looks like: a selector
//     LIST returns the first match in DOCUMENT ORDER, so the outer `div.demo` (768x319, containing a
//     `<pre>`) won over the inner `div.DemoPlaygroundInner` (766x128). The parts filter then kept
//     that container's own `role="figure"` box plus the panel's copy control, and the "widget" rect
//     spanned the demo AND its source panel (782x333) while the port's spanned its button alone
//     (69x40).
//   * `compare()` in png.mjs compares only the MIN-overlap of the two images it is given
//     (`Math.min(up.width, lx.width)`), so cropping each side to ITS OWN rect and comparing those
//     did not compare the two components: it scored the port's 69x40 button against the top-left
//     69x40 corner of upstream's demo+panel crop — mostly blank panel, and 86.96% "identical".
//
// The two rules that make the number mean what the ledger claims it means:
//   1. the region is the demo's own rendering area on each side — the smallest demo-ish container
//      that actually holds a control — and the parts inside it exclude the source panel, the page
//      chrome, and any container that merely holds the component (upstream's `role="figure"`);
//   2. both sides are cropped to a COMMON size (the larger of the two rects) before comparing, and a
//      pair of rects that are not the same kind of thing (a panel vs a control) is reported as a
//      fault instead of being scored.

import { crop, compare } from './png.mjs';

/**
 * Version of the REGION DEFINITION below. Bump it whenever the definition changes shape: a recorded
 * `widgetParity` is only comparable with a measurement taken by the same definition. The 2026-09-16
 * change (v2) replaced a per-side crop compared over the min-overlap with a playground-scoped region
 * compared over a common crop, which redefined the number outright — the numbers that stood before it
 * (checkbox "96.30", meter "95.18", button "86.96") were not parities at all, so `check-visual-budget`
 * refuses to treat them as a floor (it skips the widget-regression comparison for an entry recorded
 * under an older version instead of inventing a regression, the same rule the baseline's formula note
 * applies to the snippet-purity term).
 */
export const WIDGET_REGION_VERSION = 2;

/**
 * The browser-side region probe, as source. Evaluated by CDP on both sides of a route, so it must
 * be self-contained (no closure over node-land values) and depend on nothing but `document`.
 * Returns `{ widgetRect, demoRect, scope }`.
 */
export function widgetRegionProbe() {
  const rectOf = (el) => {
    const b = el.getBoundingClientRect();
    return { x: b.x, y: b.y, w: b.width, h: b.height };
  };
  const CONTROL = 'button,input:not([type=hidden]),select,textarea,label,[role]';
  // Chrome that is not the component: the page's nav/header, and the demo's own source panel (file
  // tabs, file selector, copy control, the code itself). Used twice — to reject code-chrome
  // CONTAINERS from the scope candidates, and to reject code-chrome PARTS.
  const CHROME_SELECTOR =
    'nav,aside,header,pre,code,figure,[class*=Code],[class*=code],[class*=Tabs],[class*=Selector],[class*=Clipboard],[class*=Copy]';
  const main = document.querySelector('main') || document.body;

  // 1. The demo's OWN rendering area.
  //
  //    Upstream nests four demo containers — div.demo > div.DemoRoot > div.DemoPlayground >
  //    div.DemoPlaygroundInner — and only the innermost one is the rendered component; its siblings
  //    hold the SOURCE PANEL, whose own wrappers are demo-named too (DemoCollapsibleRoot >
  //    DemoToolbar > DemoToolbarScrollAreaRoot, carrying the file tabs and the copy control). Two
  //    consequences, both measured on the live DOM (2026-09-16):
  //      * selecting by `querySelector` with a selector LIST is wrong — it returns the first match in
  //        DOCUMENT ORDER, so the outer `div.demo` (768x319, which CONTAINS the panel's `<pre>`) won
  //        over the inner playground;
  //      * "the smallest container that holds a control" is wrong too — the panel's DemoToolbar
  //        (766x36, 13 controls) is smaller than the playground (766x128), and on the upstream side it
  //        became the scope, with every one of its parts then excluded as chrome: a region with no
  //        component in it at all.
  //    So the playground is selected by what it IS — the container upstream names for it, or the
  //    wrapper the port renders its demos in — with a structural fallback that refuses a container
  //    holding a source panel.
  const firstMatching = (selector, ok) => {
    for (const el of main.querySelectorAll(selector)) if (ok(el)) return el;
    return null;
  };
  const holdsControl = (el) => el.querySelector(CONTROL) !== null;
  const hasSourcePanel = (el) => el.querySelector('pre,[role=tablist]') !== null;
  const scope =
    firstMatching('[class*=PlaygroundInner], .docs-demo', holdsControl)
    || firstMatching(
      '[class*=PlaygroundInner], .DemoRoot, .DemoPlayground, .docs-demo, [class*=demo], [class*=Demo]',
      (el) => holdsControl(el) && el.closest(CHROME_SELECTOR) === null && !hasSourcePanel(el),
    )
    || main;

  // 2. Chrome that is not the component (the CHROME_SELECTOR above): kept as a belt-and-braces
  //    part filter for the port side, where the demo container may later hold the source panel
  //    `docs-chrome: demo panels` adds.
  // Widget roles that legitimately OWN their rendered parts (a meter's track, a slider's thumb):
  // these are the component, so they are kept even though they contain other matched elements.
  const DISPLAY_ROOTS = ['meter', 'progressbar', 'slider', 'spinbutton', 'textbox', 'combobox', 'listbox', 'img'];
  const parts = [];
  for (const el of scope.querySelectorAll(CONTROL)) {
    if (el.closest(CHROME_SELECTOR)) continue;
    if (el.getClientRects().length === 0) continue;
    const role = el.getAttribute('role');
    const isLabel = el.tagName === 'LABEL';
    const displayRoot = role !== null && DISPLAY_ROOTS.indexOf(role) >= 0;
    // A container that merely HOLDS the component is not the component: upstream marks its demo
    // playground `role="figure"`, and a union over containers is exactly what made the old region
    // span the whole demo. Labels are exempt — a label wrapping its control is the label's own box.
    if (!isLabel && !displayRoot && el.querySelector(CONTROL)) continue;
    parts.push(el);
  }

  const box = (list, pad) => {
    let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
    for (const el of list) {
      const r = rectOf(el);
      if (r.w < 2 || r.h < 2) continue;
      x0 = Math.min(x0, r.x);
      y0 = Math.min(y0, r.y);
      x1 = Math.max(x1, r.x + r.w);
      y1 = Math.max(y1, r.y + r.h);
    }
    if (!Number.isFinite(x0)) return null;
    return {
      x: Math.max(0, Math.round(x0 - pad)),
      y: Math.max(0, Math.round(y0 - pad)),
      w: Math.round(x1 - x0 + pad * 2),
      h: Math.round(y1 - y0 + pad * 2),
      parts: list.length,
    };
  };

  const b = scope.getBoundingClientRect();
  const demoRect = b.width >= 20 && b.height >= 10
    ? { x: Math.round(b.x), y: Math.round(b.y), w: Math.round(b.width), h: Math.round(b.height), cls: String(scope.className || scope.tagName).slice(0, 60) }
    : null;
  return {
    widgetRect: box(parts, 8),
    demoRect,
    // Which container was chosen and which elements are the "component" in it — carried in the
    // report so a callers' region fault (or a wide region) can be diagnosed from the measured data
    // instead of re-derived with a separate probe (see ralph/logs/_probe_widget.mjs).
    region: {
      scope: String(scope.className || scope.tagName).slice(0, 60),
      parts: parts.map((el) => {
        const r = rectOf(el);
        return {
          tag: el.tagName,
          role: el.getAttribute('role'),
          cls: String(el.className).slice(0, 50),
          text: (el.textContent || '').trim().slice(0, 20),
          rect: { x: Math.round(r.x), y: Math.round(r.y), w: Math.round(r.w), h: Math.round(r.h) },
        };
      }),
    },
  };
}

/** The probe above, as the expression string the CDP `Runtime.evaluate` calls embed. */
export const WIDGET_REGION_JS = `(${widgetRegionProbe.toString()})()`;

/**
 * Compare two sides' regions as REGIONS: crop both to a common size (the larger of the two rects,
 * aligned at each side's own top-left, which is 8px above-left of the control on both sides), then
 * compare those equally-sized crops. Returns `comparable: false` (with a `fault` naming both rects)
 * when the pair are not the same kind of thing — the defect class this module exists to stop, where
 * a panel-vs-control crop used to be scored as a parity number.
 */
export function regionParity(upImg, lxImg, upRect, lxRect, { maxDimensionRatio = 3, maxAreaRatio = 4 } = {}) {
  const dimRatio = Math.max(
    upRect.w / Math.max(1, lxRect.w),
    lxRect.w / Math.max(1, upRect.w),
    upRect.h / Math.max(1, lxRect.h),
    lxRect.h / Math.max(1, upRect.h),
  );
  const areaRatio = Math.max(
    (upRect.w * upRect.h) / Math.max(1, lxRect.w * lxRect.h),
    (lxRect.w * lxRect.h) / Math.max(1, upRect.w * upRect.h),
  );
  const w = Math.max(upRect.w, lxRect.w);
  const h = Math.max(upRect.h, lxRect.h);
  const crops = {
    upstream: crop(upImg, { x: upRect.x, y: upRect.y, w, h }),
    leptos: crop(lxImg, { x: lxRect.x, y: lxRect.y, w, h }),
  };
  const comparable = dimRatio <= maxDimensionRatio && areaRatio <= maxAreaRatio;
  const r = comparable ? compare(crops.upstream, crops.leptos) : null;
  return {
    comparable,
    fault: comparable
      ? null
      : `the two sides' regions are not the same kind of thing — upstream ${upRect.w}x${upRect.h} vs leptos ` +
        `${lxRect.w}x${lxRect.h} (${dimRatio.toFixed(1)}x in a dimension, ${areaRatio.toFixed(1)}x in area), so their crops are not a parity`,
    parity: r ? Number((100 - r.percent).toFixed(2)) : null,
    diffPercent: r ? r.percent : null,
    commonCrop: { w, h },
    rects: { upstream: upRect, leptos: lxRect },
    crops,
  };
}
