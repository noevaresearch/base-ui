# The demo chrome: what upstream renders around a demo, and what this port renders instead

Measurement + scoping report, 2026-09-16. Written to be handed to the forward loop as a work order,
not as a narrative: every claim below is either a citation into the oracle (`docs/src/components/…`),
a measurement off a live render, or an explicit statement that something is unknown.

**Verdict in one line:** upstream wraps every demo in a *panel pair* — a playground
(`role="figure" aria-label="Component demo"`) and a collapsible code panel
(`role="figure" aria-label="Component demo code"`) whose 36px toolbar carries the file tabs, the
styling-method selector, the external-playground link, a more-actions menu (View source on GitHub /
Copy link to source), the copy-code control and the Show code / Hide code trigger. This port renders
**the playground only**. The code panel and all nine toolbar controls are absent — not styled
differently, *absent*: measured 0 of them on every route probed.

## 1. The observation that prompted this, restated precisely

The owner's report — "baseui has lot of tabs showcasing file + module css, then an option for
tailwind, then one more tab for StackBlitz, and then the point to GitHub and copy link" — is a
description of `DemoToolbar`. It is accurate, and the components behind each part are:

| What is seen | Upstream component | Source |
| --- | --- | --- |
| file tabs (`index.tsx`, `index.module.css`) | `DemoFileSelector` (built on `Tabs`) | `docs/src/components/Demo/DemoFileSelector.tsx:15-83` |
| the "tailwind" option | `DemoVariantSelector` (built on `Select`), labelled `Styling method` | `docs/src/components/Demo/DemoVariantSelector.tsx:5-51` |
| the StackBlitz tab | the external-playground button, which is **CodeSandbox on WebKit** | `docs/src/components/Demo/Demo.tsx:154-171` |
| "point to github" and "copy link" | the more-actions `Menu` with two items | `docs/src/components/Demo/Demo.tsx:184-213` |
| the code block beneath | `DemoCodeBlock` (copy control, collapse trigger) | `docs/src/components/Demo/DemoCodeBlock.tsx:36-96` |
| the whole thing | `DemoToolbar` inside `div[role=figure][aria-label="Component demo code"]`, inside a `Collapsible.Root` | `docs/src/components/Demo/Demo.tsx:225-267` |

## 2. Method (re-runnable)

Both apps were live on this box, so this is a render measurement, not source archaeology:

```
# upstream (the React oracle)          # this port
http://127.0.0.1:3005/…                http://127.0.0.1:3177/…

node ralph/scripts/probe-demo-toolbar.mjs --route react/components/<name> [--json /tmp/out.json]
# knobs, for a heavy route or a loaded box:
#   --wait <ms>            per-side settle before extracting (default 9000)
#   PROBE_EVAL_TIMEOUT_MS  CDP evaluate ceiling (default 150000)
#   PROBE_MAX_DEMOS        demo frames collected per side (default 8)
#   PROBE_CDP_PORT         devtools port (default 9889, its own)
```

`ralph/scripts/probe-demo-toolbar.mjs` (added with this report) dumps, per demo container and per
side: the container rect, the `role="figure"` landmarks, the file-tab labels, every control with its
`aria-label`/text/`href`, the `<pre>` count, and the presence of the copy / collapse / styling-method
/ playground-link / more-actions controls. It launches its own Chrome on port 9889 so it cannot
collide with a harness browser that is already up.

### 2a. Measured: `react/components/checkbox`

Upstream — one demo container, `.DemoRoot` **669x288**:

- figures: `Component demo` (`DemoPlaygroundInner`, 667x128) and `Component demo code` (667x158)
- tabs (2): `index.tsx`, `index.module.css`, each an `<a href="#hero:css-modules:index.tsx">`
- controls (10): the two tabs, then `Styling method` (combobox, text "CSS Modules"), `Open in
  StackBlitz`, `More actions` — **each appearing twice** (the in-scroll mobile copy and the sticky
  desktop copy, `docs/src/components/Demo/Demo.tsx:236-248`) — then `Copy code` and the `Show code` collapse trigger
- `<pre>`: 1

This port — one demo container, `.docs-demo` **669x128**:

- figures: none. tabs: none. controls: none. `<pre>`: none.
- page-wide: `stylingMethod 0`, `fileTabTotal 0`, `stackBlitzText 0`, `viewSourceText 0`,
  `copySourceText 0`

**The port's rect is exactly upstream's playground rect (669x128 vs 667x128).** The 158px code
panel and the 36px toolbar are the whole of the difference: the port renders the half of the demo
container that upstream calls the playground and nothing else.

### 2b. Measured: `react/components/button`

Upstream — 2 demo containers, both with their file tabs, `Styling method`, playground link,
more-actions menu, `Copy code` and `Show code`, and `pre=1`.

This port — 2 `.docs-demo` containers; each contains exactly **one** control, and it is the demo's
own `<button>Submit</button>`. `pre=0`, no tabs, no chrome.

### 2c. Every route probed, side by side

| route | side | demo frames | figure landmarks | file tabs | toolbar controls | code `pre` | styling-method | playground link | more actions | copy code | collapse |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| checkbox | upstream | 1 (669x288) | 2 | 2 | 10 | 1 | yes | yes | yes | yes | `Show code` |
| checkbox | **this port** | 1 (669x128) | 0 | 0 | 0 | 0 | no | no | no | no | none |
| button | upstream | 2 | 4 | 4 | 21 | 2 | yes | yes | yes | yes | `Show code` |
| button | **this port** | 2 | 0 | 0 | 2 (the demo's own button) | 0 | no | no | no | no | none |
| meter | upstream | 1 (669x288) | 2 | 2 | 10 | 1 | yes | yes | yes | yes | `Show code` |
| meter | **this port** | 1 (669x128) | 0 | 0 | 0 | 0 | no | no | no | no | none |
| accordion | upstream | 3 (669x336, 669x336, 669x373) | 6 | 6 | 40 | 3 | yes | yes | yes | yes | `Show code` |
| accordion | **this port** | 3 (669x3025, 669x2945, 669x3260) | 1 unlabelled each | 0 | 3 / 3 / 4 (its own triggers) | 1 each | no | no | no | no | none |

Three consequences, all measured:

1. **The port's frames are upstream's playground and nothing else.** checkbox 669x128 and meter
   669x128 against upstream's 669x288 container; the difference is the 36px toolbar plus the 158px
   panel.
2. **The figure count is the sharpest single number.** Every upstream demo contributes exactly two
   `role="figure"` landmarks (`Component demo`, `Component demo code`); the port contributes none.
   The `role=figure` elements the port does have are its page-level code panels
   (`crates/docs-app/src/code_block.rs` renders `CodeBlockRoot MdFigure`), which is a different
   upstream component — so the accordion route's "1 unlabelled figure per demo" is the *page-level*
   panel, not a demo landmark, and must not be counted as one.
3. **The missing collapse is not cosmetic; it is page height.** Same route, same three demos,
   measured on both sides: upstream renders each in a container of **669x336, 669x336 and 669x373**
   (playground 667x176/176/213, then the code panel 667x158, then the 36px toolbar), while this port
   renders the same three as **669x3025, 669x2945 and 669x3260** — and the port's inline listing is
   itself the tall object (the page-level panel inside each frame measures 621x2691, 621x2611 and
   621x2851, and it is the unlabelled `CodeBlockRoot MdFigure` figure that row 2 above refers to).
   Upstream shows that code behind one `Show code` button; a reader here scrolls about nine screens
   per demo instead. That is what the 8-line collapse rule in §4.1 costs when it is absent, and it is
   why the frame-structure item is worth doing before any tab exists.

### 2d. What this rules out

The absence is not "the toolbar renders but is unstyled". `fileTabTotal`, `stylingMethod`,
`viewSourceText` and `copySourceText` are 0 **page-wide**, and the port's stylesheet contains none of
upstream's chrome classes (`grep -c` in `crates/docs-app/style/main.css`: `DemoToolbar` 0,
`DemoTabsList` 0, `DemoTab` 0, `DemoToolbarActions` 0, `DemoCodeBlock` 0, `DemoCollapseButton` 0,
`DemoSourceBrowser` 0). This is unported structure, and it is already recorded as a parity gap by the
gap report ("missing demo file tabs" is one of its named gaps) — which is why the ledger's
`docs-chrome: demo panels (bordered container + file tabs)` item exists.

### 2e. Two instrument caveats, both measured on this box

Neither is a property of either app; both are why the numbers above are trusted rather than assumed.

- **A wrong-side read was caught, not scored.** The first version of the probe navigated and then
  extracted without checking which document it had landed on. One run evaluated the *Leptos* selectors
  against the *upstream* document (the port's dev server was being rebuilt by a concurrent iteration;
  `Page.navigate` returned `net::ERR_EMPTY_RESPONSE` and the old document stayed), and the printed
  result looked like a legitimate reading of the port — it would have been read as "the port has 2 file
  tabs". The probe now confirms `location.href` starts with the requested origin, retries up to three
  times, and returns an `ERROR` line that names the document it actually landed on rather than a
  number. The capture above is from after that fix.
- **Upstream's accordion row needed a second attempt, and the first failure was the box.** The first
  run of that route died with SIGABRT (exit 134) alongside `Warning: Failed to load CA certificates off
  thread: resource temporarily unavailable`, with the cgroup at **506 of 512 tasks** and load average
  11.6 — the port's dev server was being rebuilt by a concurrent iteration while a
  `cargo-leptos watch --release` sandbox build also ran. That is a resource limit, not a defect in
  either app; the row in §2c is from the re-run with the box at 247 tasks, and both document origins
  were confirmed by the probe before it extracted.

## 3. What the port renders today, and why

- 31 demo wrappers across the ported pages are a bare `<div class="docs-demo" data-demo="…">`
  (`crates/docs-app/src/pages/checkbox_page.rs:770`, `crates/docs-app/src/pages/avatar_page.rs:283`, `crates/docs-app/src/pages/button_page.rs:474`, …).
- `.docs-demo` is a faithful port of upstream's `.DemoPlaygroundInner` — its geometry is pinned by a
  host guard (`crates/docs-app/src/demo_styles.rs:242-263`, "the demos' controls stop being
  content-width and the widget region becomes unmeasurable again").
- Three accordion demos additionally stack a source listing: `<div class="docs-demo
  docs-demo-with-source"> <div class="docs-demo-stack"> {demo} {code_block(Lang::Rust, "", SNIPPET)}
  </div></div>` (`crates/docs-app/src/pages/accordion_page.rs:500-505`).
- `.docs-demo-stack` and `.docs-demo-with-source` are **port inventions**: upstream has no such
  wrapper. They exist so a source listing does not sit inside `.docs-demo` as a flex ROW sibling of
  the control (the comment at `crates/docs-app/style/main.css:1499-1518` records the measurement that forced them: the
  accordion collapsed to 69px wide and the widget region became NOT MEASURABLE). They are honest
  scaffolding, and the real code panel **supersedes** them — the panel's home is a `Collapsible`
  below the playground, not a stack next to it.

## 4. The demo *is* chrome-sensitive: the oracle's own structure, cited

```
.DemoRoot                                    docs/src/components/Demo/Demo.tsx:218
  ├─ <span id="{slug}"> per file             docs/src/components/Demo/Demo.tsx:219-221   (deep-link targets)
  ├─ .DemoPlayground > [role=figure]         docs/src/components/Demo/DemoPlayground.tsx:10-25
  │                    [aria-label="Component demo"] .DemoPlaygroundInner
  └─ Collapsible.Root (open = expanded)       docs/src/components/Demo/Demo.tsx:225-229
       └─ div[role=figure]                    docs/src/components/Demo/Demo.tsx:230
            [aria-label="Component demo code"]
            ├─ .DemoToolbar                   docs/src/components/Demo/Demo.tsx:231-249   (height 2.25rem; docs/src/components/Demo/Demo.css:54-93)
            │    ├─ ScrollArea (mobile: tabs + actions scroll together)  docs/src/components/Demo/Demo.tsx:236-246
            │    └─ .DemoToolbarActions sticky copy (desktop)            docs/src/components/Demo/Demo.tsx:248
            └─ DemoCodeBlock                  docs/src/components/Demo/Demo.tsx:251-266
                 ├─ scroll areas + Copy code button  docs/src/components/Demo/DemoCodeBlock.tsx:46-53, :83
                 └─ Collapsible.Trigger "Show code"/"Hide code"  :85-92
```

Three behaviours in that structure are easy to miss and are part of parity, not decoration:

1. **The code panel collapses by line count.** `collapsibleLinesThreshold = 8`
   (`docs/src/components/Demo/DemoCodeBlock.tsx:40`): fewer than 8 lines renders flat with no trigger (`docs/src/components/Demo/DemoCodeBlock.tsx:44-56`); 8 or more
   renders collapsed behind `Show code` (`docs/src/components/Demo/DemoCodeBlock.tsx:85-92`). This port renders every demo listing fully
   expanded, always. That is a *behavioural*, testable divergence, not a styling one.
2. **Collapsing scrolls the page back.** `onOpenChange` measures the trigger before and after and
   calls `scrollBy({top: delta, behavior: 'instant'})` when the trigger ends up above the viewport
   (`docs/src/components/Demo/Demo.tsx:114-142`).
3. **The file tabs are anchors, not just buttons.** Each tab is `<a href="#{demo}:{variant}:{file}">`
   with Ctrl/Cmd-click passing through to a new browser tab (`docs/src/components/Demo/DemoFileSelector.tsx:41-58, :70`), and
   the anchors they point at are minted per file in the container (`docs/src/components/Demo/Demo.tsx:219-221`). Measured
   hrefs: `#hero:css-modules:index.tsx`.
4. **The toolbar's controls appear twice** (in-scroll mobile copy + sticky desktop copy, both
   mounted so state survives a resize — the comment at `docs/src/components/Demo/Demo.tsx:232-235`). A port that renders one
   copy is a visible divergence at some viewport widths, and the measured DOM above shows both.

### 4a. And the tabs are conditional — in a way the port must copy, not flatten

`DemoFileSelector` returns **null** when a demo has 1 file or fewer (`docs/src/components/Demo/DemoFileSelector.tsx:60-62`). Upstream's demo
folders:

```
checkbox/demos/hero/index.ts            createDemoWithVariants(import.meta.url, {CssModules, Tailwind})
checkbox/demos/hero/css-modules/index.tsx + index.module.css     <- 2 files => tabs render
checkbox/demos/hero/tailwind/index.tsx                           <- 1 file  => NO tabs
```

So the *same demo* shows two tabs under CSS Modules and none under Tailwind. A port that hard-codes
"2 tabs" or "0 tabs" is wrong under one of the two variants — the machinery is the parity, and the
right port is a component that decides, exactly as upstream's does. Likewise the variant selector
renders only when `demo.variants.length > 1` (`docs/src/components/Demo/Demo.tsx:175-182`).

## 5. Dependencies: this is not a free-standing chrome job

Upstream builds its toolbar out of **three components this port has not ported**, plus two it has:

| needed by | component | state in `crates/leptos-ui/src/` | ledger item |
| --- | --- | --- | --- |
| file tabs | `Tabs` | **MISSING** | `library: tabs` (TODO.md:989, not-started) |
| styling-method selector | `Select` | **MISSING** | `library: select` (TODO.md:955, not-started) |
| toolbar scroll + code viewport | `ScrollArea` | **MISSING** | `library: scroll-area` (TODO.md:948, not-started) |
| code panel | `Collapsible` | present (`collapsible/`, 6 files) | `library: collapsible` done |
| more-actions menu | `Menu` | present (`menu/`, 26 files) | `library: menu` done (TODO.md:821) |

Two honest responses, and the choice belongs to the owner:

- **(i) Faithful** — the toolbar waits on `library: tabs` + `library: scroll-area` (and the selector
  on `library: select`). Cost: the chrome lands after three component ports. Benefit: the chrome is
  built from the same parts upstream uses, dogfooded through our own `Collapsible` and `Menu`.
- **(ii) Interim** — the container, figures and anchors, the copy control and the collapse trigger
  land now on the ported `Collapsible` (no missing dependency), with the tab strip and the variant
  selector deferred. Cost: two controls arrive later. Benefit: the page-level fidelity gap closes
  immediately and in bounded pieces.

**Why not a CSS-only tab strip:** a `role="tab"` list without the component's roving focus and
`aria-selected` management is an accessibility divergence *and* a new invented shape — the same class
of defect as the gap report's P0 for React snippets. The tab strip's *visual* chrome
(`.DemoTabsList`, `.DemoTab`, `.DemoToolbar`'s two 1px separators, `docs/src/components/Demo/Demo.css:54-93, :196-215`) is
CSS and can be ported by the `docs-chrome` lane whenever the strip lands; its *behaviour* is `Tabs`.

## 6. The styling method, and the paths that exist for it

Upstream's demo sources come in two variants on disk
(`demos/<demo>/{css-modules,tailwind}/index.tsx`, plus `css-modules/index.module.css`), and the
selector switches between them. This port renders **one** variant: the Tailwind one, with the
utilities generated from upstream's compiled stylesheet into `style/main.css`
(`ralph/scripts/gen-demo-utilities.mjs`, guarded by `crates/docs-app/src/demo_styles.rs:173-207`).

So "show the Tailwind option" is not a chrome task at all — it is a *content* task, and it already
exists. What does not exist is a **second** variant for the selector to switch to. Upstream's second
variant is CSS Modules: one stylesheet per demo in which class names are local
(`index.module.css`, referenced as `styles.foo`) and hashed at build time. The owner's proposal —
wire stylance, or write our own css-loader — is the right question, and the options are:

- **(a) `stylance`** (`stylance` proc macro + `stylance-cli`): `import_crate_style!(s, "src/x.module.css")`
  brings class names in as constants; the CLI bundles `.module.css` files with a hash derived from the
  file's path relative to the crate manifest. Needs wiring in **both** builders this repo has: the
  docs app (`cargo leptos`, `crates/docs-app`) and the sandbox (`Trunk`,
  `examples/leptos-sandbox`, which assembles its own `index.html` and would take stylance as a
  `[[hooks]] pre_build`). A class name that does not exist in the CSS is a compile error, and unused
  ones warn — both are properties this repo would actually use.
- **(b) extend the existing generator.** The port already generates a CSS section at build time
  (`gen-demo-utilities.mjs`) and already has the guard that reads pages + stylesheet and fails when a
  named class has no rule (`demo_styles.rs`). A css-loader that reads each demo's
  `css-modules/index.module.css`, emits deterministic `name-<hash>` rules into the same generated
  section, and emits a Rust module of class constants for the pages, is the *same* machinery this
  port already maintains — no new build step in either builder.
- **(c) no second variant, no selector.** Absence is faithful where upstream renders nothing, but
  here upstream renders the selector on every demo with two variants. So (c) is only correct if we
  also record the selector as a permanent, disclosed divergence.

**Recommendation, with its reason:** (b) for the variant that makes the selector honest, because the
port's problem is not "hashing class names" (its generator does that job's shape already) but
"produce upstream's css-modules rules as a second live variant", and (b) puts that in the pipeline
that a) already runs in both builders and b) already has a host guard. `stylance` is the better tool
if we want real scoping semantics (`:global`, nesting, compile-time name checking) as a *crate-wide*
convention rather than one variant of the docs demos; adopting it is a bigger decision than this
report, and it should be taken as its own item if taken. **Open decision for the owner.**

Whichever path: a selector that toggles between two renders that are identical is fake chrome, worse
than no selector. The proving observable for that item must therefore be that the two variants
render *different class names* (and that both variants' rules exist in the served stylesheet), not
merely that a `<button aria-label="Styling method">` exists.

## 7. The controls that need no new component

- **Copy code** (`docs/src/components/Demo/Demo.tsx:256-265`, `docs/src/components/Demo/DemoCodeBlock.tsx:83`): a `GhostButton` with
  `aria-label="Copy code"` whose icon swaps to a check for 2s. The port already has this *shape* one
  level up (`crate::code_block`'s page-level panel, ported by the DONE item
  `docs-chrome: code blocks`). **Do not reuse it here**: upstream has two distinct components —
  `CodeBlockPreComputedContent` for a page's `.mdx` fences and `DemoCodeBlock` for a demo's listing —
  and conflating them invents a shape that matches neither (`crates/docs-app/src/code_block.rs:651-654` documents the
  page-level panel's own contract).
- **Show code / Hide code** (`docs/src/components/Demo/DemoCodeBlock.tsx:85-92`): on the ported `Collapsible`, with the 8-line
  threshold rule from §4.1.
- **View source on GitHub / Copy link to source** (`docs/src/components/Demo/Demo.tsx:184-213`): on the ported `Menu`. The URL
  is derived, not stored: upstream builds it from `SOURCE_CODE_REPO` + `LIB_VERSION` env and the
  demo's own `import.meta.url`, then appends the selected variant's kebab-cased subdirectory
  (`docs/src/utils/getGitHubDemoUrl.ts:19-34, :56-76`). The port's analogue is a **pure function**
  over (repo, ref, page source path, line range) → URL, unit-testable on the host, with the
  expectation written out literally in the test rather than recomputed by the same function.
  Note this is where a port has to decide something upstream never did: our demo source lives in
  `crates/docs-app/src/pages/<name>_page.rs`, not in upstream's `demos/<demo>/<variant>/` tree, so
  the link must point at *our* file and *our* pushed ref. **Open decision: which repo/ref the link
  targets** (`noevaresearch/base-ui` at the pushed branch is the only one where the file exists).

## 8. What already exists for the external playground (owner's correction)

The sandbox is built and is not part of this report's open work:

- `examples/leptos-sandbox/` — a CSR app (`Trunk`, own workspace root, consumes the **published**
  `base-ui-leptos` from crates.io) that mounts ported demos and selects one with `?demo=<slug>`.
- `.github/workflows/sandbox-mirror.yml` force-pushes it to the root of the `sandbox` branch, because
  CodeSandbox reads `.devcontainer/`/`.codesandbox/` from a template's own root and discards its
  memory snapshot on every commit to the backing branch.
- The docs-side control is already ledgered: `sandbox: the docs-side control — a ported DemoToolbar
  carrying "Open in CodeSandbox"` (TODO.md:3250), whose done-when already demands the toolbar as the
  control's home rather than a stray link.
- **The owner's decision: the link is CodeSandbox, not StackBlitz** — and that is upstream-sanctioned
  rather than a divergence, because upstream renders exactly this arm on WebKit
  (`docs/src/components/Demo/Demo.tsx:154-171`: `platform.engine.webkit ? CodeSandbox : StackBlitz`), and because StackBlitz
  cannot run a Rust toolchain.

## 9. How "done" gets proven (named observables)

No score alone proves any of this, and a green test that only checks "a toolbar exists" proves
nothing. Each item must land with:

1. **Surface parity, by probe:** `node ralph/scripts/probe-demo-toolbar.mjs --route <route>` — the
   port's per-demo inventory must match upstream's for that route on the axes the item claims (tab
   labels, control `aria-label`s, figure labels, copy/collapse presence). This is the observable that
   answers "is it there at all".
2. **A host-testable derivation for anything computed** (the GitHub URL; the tab href slugs; the
   collapse-by-line-count decision), asserted against a literally written expectation.
3. **A wasm structure test** in `crates/docs-app/src/render_test.rs` pinning the container's shape for
   one named route (`role="figure"` pair, toolbar, control set), in the shape of the existing per-page
   structure guards.
4. **Non-regression on the fidelity instrument:** `node ralph/scripts/check-visual-budget.mjs --todo-id
   "<item>"` must not drop the route's recorded page score, and
   `node ralph/scripts/check-page.mjs --route <route>` must be the page verdict that decides the page.
5. **One honest negative:** for the collapse rule, a demo with fewer than 8 lines must render **no**
   `Show code` trigger. A port that always renders the trigger passes every positive assertion.

## 10. The gaps this report is NOT claiming

- It does not claim the toolbar is the only remaining fidelity gap, nor that porting it reaches the
  90/97 bars: page scores were 85.12 (checkbox), 86.54 (button), 67.39 (meter) at the last recorded
  build, and the gap report also names sidebar/code chrome, API tables, fonts and demo styling.
- It does not claim the port should copy upstream's `index.tsx`/`index.module.css` **file names**.
  The port's demo source is Rust; the honest tab labels are the port's own real files, and the
  deep-link slugs are `{demo}:{variant}:{file}` over the port's own identity.
- It does not claim upstream's per-demo file counts for routes not probed. Four routes were measured
  end to end on both sides here (checkbox, button, meter, accordion); the probe is the tool for any
  other route and needs no modification to run one.
