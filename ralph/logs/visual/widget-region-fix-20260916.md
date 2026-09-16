# Widget-region fix — what the "component widget parity" number used to measure

**Date**: 2026-09-16 · **Item**: `docs-fidelity: visual budget gate` (reopened) · **Crate**: docs-app
(scripts only — no Rust source was touched) · **Measured build**: wasm 35,803,229 bytes @ 2026-09-16
04:08:20 GMT (unchanged by this work; the change is in `ralph/scripts/`).

Supersedes the widget numbers recorded on 2026-09-16 (`checkbox 96.30`, `meter 95.18`, `button
86.96`, in `docs-parity`'s note and in the first `-widget.png` crops). Those were not parities: the
number was produced by comparing two crops of **different kinds of thing**, and `compare()` only
compares the overlap. This file is the disclosure the ledger's baseline rule asks for: what the old
number measured, the corrected number for each route, and why the correction is a measurement fix
rather than a score rebase.

## 1. What the old number measured

`check-visual-budget.mjs` scores two regions per route: the **widget** (the component itself, bar
97%) and the **demo frame** (upstream's demo chrome, informational). Both were computed by cropping
**each side to its own rect** and calling `compare(upCrop, lxCrop)`, and `compare()`
(`ralph/scripts/lib/png.mjs:126-128`) compares only `Math.min(up.width, lx.width)` ×
`Math.min(up.height, lx.height)` — the **overlap**. With `ralph/logs/visual/button.json`'s measured
rects:

| side | widget rect | what the crop contained |
| --- | --- | --- |
| upstream | 782×333 | the Submit button **and** the demo's source panel (`index.tsx` / `index.module.css` tabs, the highlighted code, the copy control) |
| leptos | 69×40 | the Submit button alone |

The number reported was therefore the similarity of the port's 69×40 button to the top-left 69×40
corner of upstream's demo+panel crop — an area that is mostly blank panel. "86.96%" was a statement
about white space. The crops are still visible in git history (`button-upstream-widget.png` at
04:31: the button *and* the code panel; `button-leptos-widget.png`, 847 bytes: the word "Submit").

Why upstream's rect spanned the source panel — measured on the live DOM, twice:

* the region's scope came from `querySelector('[class*=PlaygroundInner], .docs-demo, [class~=demo],
  .DemoRoot, [class*=demo]')`. A selector **list** resolves in DOCUMENT ORDER, not list order, so the
  outer `div.demo` (768×319, which contains the panel's `<pre>`) won over its own child
  `div.DemoPlaygroundInner` (766×128, the demo's real rendering area);
* the parts filter then kept that outer container's own `role="figure"` box plus elements from the
  panel, so the union spanned demo + panel.

And why the *fix attempt* of 2026-09-16 04:28 (commit d1c66a5b0, "stop the widget region from
swallowing the demo's code panel") did not fix it: it added the panel's `role=tab`/code descendants
to the exclusion list, but the scope was still `div.demo`, so the panel's `<pre>` remained inside the
region's bounding box. Measured after that commit: upstream 782×333 (a source panel), port 69×40 (a
control) — the same defect, still scored.

## 2. Why every docs-app item's gate was failing

`targetComponent` defaulted to **97 and was enforced** (`check-visual-budget.mjs:81`, `:305`), while
no recorded route met it — so `check-visual-budget.mjs --all-done`, which `run-regression.sh` step 5
runs for every `crate: docs-app` item (and which step 5's route-ful branch runs for every docs page),
exited 1 unconditionally. The failure message then said `FAIL: visual fidelity regressed beyond 2
points` even when every score was unchanged (`delta +0`) — a regression report about a bar, not a
regression. Net effect: **no docs-app item could pass its own gate**, including the API-reference-
tables item whose work is already landed, and step 7's own rule ("fails for ANY reason → mark the
item blocked") would have written false `blocked` state across the ledger.

## 3. The fix

* `ralph/scripts/lib/widget-region.mjs` (new) — ONE definition of the region, shared by
  `visual-diff.mjs` (the report the gate scores) and `visual-gap-report.mjs` (which names the gaps).
  They each carried their own copy before, with different selector lists and no shared rule.
  * the scope is the demo playground **by identity**: `[class*=PlaygroundInner]` (upstream) or the
    port's `div.docs-demo`, first in document order that holds a control, with a structural fallback
    that refuses a container holding a source panel. The two heuristics that look obvious are both
    wrong here and both were measured on the live DOM: document-order `querySelector` picks the outer
    `div.demo` (which contains the panel), and "smallest container holding a control" picks the
    panel's own `DemoToolbar` (766×36, 13 controls: the file tabs and selectors) over the playground
    (766×128) — the first attempt at this fix produced exactly that, and reported "upstream none,
    leptos yes";
  * parts exclude the page chrome and the source panel, and exclude any container that merely HOLDS
    the component (upstream marks its playground `role="figure"`), while keeping display roots that
    legitimately own their parts (`meter`, `slider`, `progressbar`, …);
  * `regionParity()` crops both sides to a **common** size (the larger rect, aligned at each side's
    own top-left, which is 8px above-left of the control on both sides) before comparing, and returns
    `comparable: false` with a `fault` naming both rects when they are not the same kind of thing
    (≥3× in a dimension or ≥4× in area). The crops written to disk are the crops that were compared.
* `check-visual-budget.mjs` — the widget bar is **printed** every run and **enforced on request**
  (`--target-component`, exactly the flag `docs-parity`'s done-when names), and the widget is gated
  on **regression** against its recorded best-known value like the page score; a region that cannot
  be compared now FAILS loudly instead of scoring whatever the overlap was; the recorded
  `widgetParity`/`demoParity` are preserved as floors when a run cannot measure them; and the FAIL
  line names the rule that failed (page regression, widget regression, region fault, absolute bars)
  instead of always saying "regressed beyond 2 points".
* `visual-gap-report.mjs` — same shared region; a region fault is a **P0 finding** ("component widget
  not measurable"), never a silently-absent number.
* `run-regression.sh` — its two visual-gate failure messages now point at the FAIL line's rule.

## 4. Corrected numbers, measured at this tree

| route | widget parity (old → new) | page score | visual | content | pixelDiff |
| --- | --- | --- | --- | --- | --- |
| react/components/button | "86.96%" (panel vs button) → **84.42%** (88×48 vs 69×40) | 86.54 (+0) | 91.65 | 78.88 | 8.35% |
| react/components/checkbox | "96.30%" (panel vs control) → **NOT MEASURABLE** (upstream 166×36 vs leptos 784×57) | 85.12 (+0) | 88.76 | 79.65 | 11.24% |
| react/components/meter | "95.18%" (panel vs control) → **NOT MEASURABLE** (upstream 256×56 vs leptos 784×42) | 67.39 (+0) | 93.76 | 27.83 | 6.24% |

The blended page score is untouched by this change — the region terms were never part of it — and all
three routes reproduced their recorded scores exactly (`delta +0`), so nothing was rebased to hide a
drop. What changed in `ralph/generated/visual-baseline.json` is only the region fields: `button`'s
`widgetParity` floor is now the honest 84.42, and the two routes that cannot be compared record
`null` (their previous floors are kept by the `??` guard in the record block).

Button's 84.42 is a real, actionable gap and its crops show it directly:
`ralph/logs/visual/button-upstream-widget.png` is a **bordered** 72×32 button; the port's
`button-leptos-widget.png` is bare text at 53×24, with no border, padding or background. See §5.

## 5. The measurement's first real finding: the demos are upstream's Tailwind variant, unstyled

Checkbox and meter cannot be compared at all because the port's "component" is **full-width**
(784px) where upstream's is the control itself (166px / 256px). Root cause, cited:

* the port's demos carry upstream's **tailwind** variant class strings verbatim —
  `crates/docs-app/src/pages/button_page.rs:82` (`DEMO_BUTTON_CLASS`) is
  `docs/src/app/(docs)/react/components/button/demos/hero/tailwind/index.tsx:6` character for
  character, and `crates/docs-app/src/pages/checkbox_page.rs:71-79` is
  `.../checkbox/demos/hero/tailwind/index.tsx:6-13`;
* upstream's docs render the **css-modules** variant by default — the live page's elements carry
  `index-module__w8A2EG__Label` / `…__Checkbox` / `…__Indicator`, styled by
  `.../checkbox/demos/hero/css-modules/index.module.css` (and `styles.Button` for button);
* the port ships a hand-written stylesheet (`crates/docs-app/style/main.css`; the served
  `/pkg/docs-app.css` is 21,136 bytes) that contains **none** of those utilities and none of the
  CSS-module class names — measured: `gap-2` 0 hits, `items-center` 0, `shrink-0` 0, `text-sm` 0,
  `index-module` 0.

So the demo class strings are inert: the checkbox demo's
`<label class="flex items-center gap-2 text-sm …">` lays out as a full-width block (768×41 vs
upstream's 150×20) and its `<span role="checkbox" class="flex size-4 …">` measures 768×16 instead of
16×16; the button renders without its border/padding/height (53×24 vs 72×32). This is a
demo-fidelity defect on every mirrored page, not a docs-chrome nicety, and it is scoped into the
ledger as its own item (`docs-chrome: demo styling …`) with these citations — it was invisible while
the widget number was measuring background.
