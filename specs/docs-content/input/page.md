# Input — docs page spec (Stage 1: docs content mining)

Source page: `docs/src/app/(docs)/react/components/input/page.mdx` (51 lines).
Cross-check baseline: `specs/library/input/behavior.md` (test-mined) — referenced below by section name only, not restated.
Demo source files (`./demos/hero`) are out of scope here; Stage 2 (`specs/docs-content/input/demos.json`) mines those.

## Page structure (headings, in order)

- `# Input` — page title (`docs/src/app/(docs)/react/components/input/page.mdx:1`)
- Subtitle: "A native input element that automatically works with [Field](/react/components/field)." (`docs/src/app/(docs)/react/components/input/page.mdx:3-5`)
- `<Meta name="description">` — "A high-quality, unstyled React input component." (`docs/src/app/(docs)/react/components/input/page.mdx:7`)
- Hero demo embed — imports `DemoInputHero` from `./demos/hero` and renders it before any heading (`docs/src/app/(docs)/react/components/input/page.mdx:9-11`)
- `## Usage guidelines` (`docs/src/app/(docs)/react/components/input/page.mdx:13`)
- `## Anatomy` (`docs/src/app/(docs)/react/components/input/page.mdx:17`)
- `## API reference` (`docs/src/app/(docs)/react/components/input/page.mdx:27`)
- `export const metadata` SEO keywords block closing the page (`docs/src/app/(docs)/react/components/input/page.mdx:33-51`)

No sub-headings exist under `## Usage guidelines`, `## Anatomy`, or `## API reference`.

## Prose claims about component behavior

1. "A native input element" (`docs/src/app/(docs)/react/components/input/page.mdx:3-5`). Cross-check: consistent with behavior.md § Public API surface and § DOM structure (the default rendered element is a native `<input>`, proven via `refInstanceof: window.HTMLInputElement`).
2. "[Input] automatically works with [Field](/react/components/field)" (`docs/src/app/(docs)/react/components/input/page.mdx:3-5`). Cross-check: behavior.md § Accessibility is N/A beyond native `<input>` semantics and no Field composition is exercised anywhere in the test suite — flagged under Discrepancies.
3. "Form controls must have an accessible name": it can be created using a `<label>` element or the `Field` component, with a link to the forms guide (`docs/src/app/(docs)/react/components/input/page.mdx:15`). Cross-check: behavior.md § Accessibility is N/A (the suite makes no labeling or `aria-*` assertions); this is native-platform usage guidance, unproven there but not contradicted.
4. Anatomy guidance: "Import the component and use it as a single part" — imported from `@base-ui/react/input` (`docs/src/app/(docs)/react/components/input/page.mdx:19-25`). Cross-check: consistent with behavior.md § Public API surface (`<Input />` is a single-part component with no subcomponents).

## API tables referenced (props/parts documented on this page)

- The page embeds no literal prop tables; the API reference renders a generated type-table component `<TypesInput />` imported from `./types` (`docs/src/app/(docs)/react/components/input/page.mdx:29-31`).
- `## API reference` has no part sub-headings — the only part documented on the page is `<Input />` itself, matching behavior.md § Public API surface, which lists just the single root part.
- No props are demonstrated in the page prose or its single snippet; the page documents no prop names at all.

## Code snippets embedded directly in the .mdx (not pulled from demos/)

1. Anatomy: import from `@base-ui/react/input` plus a bare `<Input />` element, `title="Anatomy"` (`docs/src/app/(docs)/react/components/input/page.mdx:21-25`).

This is the page's only inline snippet; it contains no `@highlight` directives and shows a single-part usage with no props. The hero demo is not an inline snippet — it is the imported `./demos/hero` component (`docs/src/app/(docs)/react/components/input/page.mdx:9-11`), reserved for Stage 2.

## Discrepancies (docs page vs. behavior.md)

- Direct mismatches: none found — every test-proven claim the page makes (native `<input>` default root, single-part API) agrees with behavior.md § Public API surface and § DOM structure.
- Docs claims with no counterpart (not contradicted, but unproven) in behavior.md:
  - "automatically works with Field" (`docs/src/app/(docs)/react/components/input/page.mdx:3-5`) — behavior.md § Accessibility is N/A and the test suite never composes Input with Field, so the Field integration claim rests on native semantics, not tests.
  - The accessible-name guidance (label or Field, `docs/src/app/(docs)/react/components/input/page.mdx:15`) — same N/A status in behavior.md § Accessibility.
  - Marketing claims in the subtitle/meta ("high-quality, unstyled") (`docs/src/app/(docs)/react/components/input/page.mdx:3-7`) — no behavioral counterpart; not test-verifiable.
- behavior.md content absent from the page (absence is not a contradiction, noted for downstream completeness): the `render` prop (function and element forms), `className` merging, arbitrary DOM prop spreading, ref forwarding, and customization ref/className merging are all proven in behavior.md § Public API surface, § DOM structure, and § Edge cases but never mentioned in the page prose. Conversely, the page is silent on value handling, validation, focus, keyboard, and events — matching behavior.md, which marks those areas UNVERIFIED by tests.
- The SEO keyword "Textarea Alternative" (`docs/src/app/(docs)/react/components/input/page.mdx:43`) implies a relationship to the Textarea component that no prose or behavior.md section supports.

## Cross-links to other docs pages

- `[Field](/react/components/field)` → `/react/components/field` (`docs/src/app/(docs)/react/components/input/page.mdx:3-5`).
- `[forms guide](/react/handbook/forms)` → `/react/handbook/forms` (`docs/src/app/(docs)/react/components/input/page.mdx:15`).
- Local (non-cross-link) imports on the page: `./demos/hero` (`docs/src/app/(docs)/react/components/input/page.mdx:9`) and `./types` (`docs/src/app/(docs)/react/components/input/page.mdx:29`).

## Snippet & behaviour contract

Per `specs/docs-content/CONTRACT.md` requirement 5, this table is part of this page's done. Authored
2026-09-16 by the `docs-content: components/input` iteration, which is the pick the contract's own
exception covers ("or the page's own item"); the spec carried no such section when the page was
mirrored, which is why `node ralph/scripts/check-docs-contract.mjs` listed this page among the pages
lacking a contract.

This page teaches two examples: the hero demo (`docs/src/app/(docs)/react/components/input/page.mdx:9-11`)
and the single inline Anatomy fence (`docs/src/app/(docs)/react/components/input/page.mdx:21-25`).
The port's real surface is the `#[component] Input` in `crates/leptos-ui/src/input.rs:184-229`, used
in `view!` markup — `Input` is a single-part component with no subcomponents upstream
(`specs/library/input/behavior.md` § Public API surface), so there is no `Input::Part` tree to teach,
and the snippet does not show a flattened `*_view(..)` call either.

| example (upstream citation) | Leptos snippet to show | behavioural obligations (cited) | observable that proves it |
| --- | --- | --- | --- |
| Anatomy — import and use it as a single part (`docs/src/app/(docs)/react/components/input/page.mdx:21-25`) | `use leptos_ui::Input;` then `view! { <Input /> }` — the port's component in `view!` markup | `specs/library/input/behavior.md` § Public API surface (the unit is a single root part with no subcomponents) | the rendered tree contains a native `<input>` element, and the block classifies as Leptos rather than upstream's JSX (`crates/docs-app/src/pages/input_page.rs`'s `snippet_language_guard`, the browser-free copy of the probe's rules) |
| Hero demo (`docs/src/app/(docs)/react/components/input/page.mdx:9-11`; source `docs/src/app/(docs)/react/components/input/demos/hero/tailwind/index.tsx:3-12`) | the port's `Input` inside upstream's wrapping `<label>`, with the demo's `placeholder` through the port's `element_attributes` rest bag and the demo's `className` as the port's `class` | `specs/library/input/behavior.md` § Public API surface (single root part) and § DOM structure (the default rendered element is a native `<input>`); `specs/docs-content/input/demos.json` entry 1 (`stateManaged: "none"` — the browser owns the value; no state, no handlers) | `crates/docs-app/src/render_test.rs`'s `input_page_renders_the_mirrored_structure_and_its_hero_demo`: the label renders with the text "Name", the `<input>` carries the demo's class verbatim, and one browser turn after mount the `placeholder` attribute is present — the `element_attributes` bag is applied by the control's own mount effect (`crates/leptos-ui/src/field/field_control.rs:584-601`), so a synchronous assert would read the seed |

Gaps carried open against this contract (do not mark more of this page done over them):

* **The Field-integration claim is upstream's guidance, not a test-proven port behaviour.** The
  subtitle and the Usage-guidelines bullet say the input "automatically works with" `Field`; the
  unit's mined behaviour spec marks § Accessibility N/A and never composes the two (already recorded
  in this spec's Discrepancies section). The page therefore repeats upstream's sentence and teaches
  the accessible-name routes it names, without asserting a port-level obligation the behaviour spec
  does not prove.
* **The port does not expose `render`** (nor `ref`): the delegation target builds a fixed `<input>`
  (`crates/leptos-ui/src/input.rs:29-46`), so the API-reference row states the gap and names the
  ledger items that own it (`library: the view paths drop render's element form`,
  `library: Field.Control's port surface carries no ref slot`). The row does not transcribe
  upstream's rationale for behaviour this port does not have.
* **The state-aware `className`/`style` function form is not exposed** by this component's
  `InputViewProps` (the static spelling only, `crates/leptos-ui/src/input.rs:76-79`), while the
  generated `types.md` type for both props offers one upstream. The `Input.State` prose describes
  the state a state-aware writer receives in this port; the function form is a gap of the same
  family as `render`/`ref`, and is stated rather than promised.
* **Upstream's demo chrome** (file tabs, the code panel, the styling-method selector) is
  `docs-chrome: demo frame structure` / `demo file tabs` scope, not this page's snippet work.
