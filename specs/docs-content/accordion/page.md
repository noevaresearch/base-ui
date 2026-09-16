# Accordion docs page content spec

Mined from `docs/src/app/(docs)/react/components/accordion/page.mdx` only. The component's own
behavior is covered by `specs/library/accordion/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/accordion/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/accordion/demos.json`).

## Page structure (headings, in order)

- `# Accordion` (h1) — `docs/src/app/(docs)/react/components/accordion/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/accordion/page.mdx:13`
- `## Examples` — `docs/src/app/(docs)/react/components/accordion/page.mdx:30`
  - `### Open multiple panels` — `docs/src/app/(docs)/react/components/accordion/page.mdx:32`
  - `### Hidden until found` — `docs/src/app/(docs)/react/components/accordion/page.mdx:40`
- `## API reference` — `docs/src/app/(docs)/react/components/accordion/page.mdx:52`
  - `### Root` — `docs/src/app/(docs)/react/components/accordion/page.mdx:56`
  - `### Item` — `docs/src/app/(docs)/react/components/accordion/page.mdx:60`
  - `### Header` — `docs/src/app/(docs)/react/components/accordion/page.mdx:64`
  - `### Trigger` — `docs/src/app/(docs)/react/components/accordion/page.mdx:68`
  - `### Panel` — `docs/src/app/(docs)/react/components/accordion/page.mdx:72`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A set of collapsible panels with headings." — `docs/src/app/(docs)/react/components/accordion/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React accordion component that displays a set of collapsible panels with headings." — `docs/src/app/(docs)/react/components/accordion/page.mdx:4-7`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/accordion/page.mdx:9-11`
- Demo imports interleaved with the Examples subsections (`./demos/multiple` at `docs/src/app/(docs)/react/components/accordion/page.mdx:36`, `./demos/hidden-until-found` at `docs/src/app/(docs)/react/components/accordion/page.mdx:48`)
- `TypesAccordion` import for the API reference — `docs/src/app/(docs)/react/components/accordion/page.mdx:54`
- Trailing `export const metadata` SEO keywords block (10 keywords, e.g. 'React Accordion', 'Accessible Accordion') — `docs/src/app/(docs)/react/components/accordion/page.mdx:76-89`

## Prose claims about component behavior

- Page describes the component as "A set of collapsible panels with headings." and, in the meta
  description, as "unstyled". Naming-level claim only; consistent with the five-part API
  (Root/Item/Header/Trigger/Panel, with Header rendering a heading element) in behavior.md
  "Public API surface (props, parts, subcomponents)" and "DOM structure & portal behavior".
  `docs/src/app/(docs)/react/components/accordion/page.mdx:3`, `docs/src/app/(docs)/react/components/accordion/page.mdx:4-7`
- Anatomy usage guidance: "Import the component and assemble its parts", showing the assembly
  `Accordion.Root` > `Accordion.Item` > `Accordion.Header` (wrapping `Accordion.Trigger`) with
  `Accordion.Panel` as a sibling of the Header inside the Item, imported from the
  `@base-ui/react/accordion` namespace. The part set and namespace import match behavior.md
  "Public API surface (props, parts, subcomponents)"; behavior.md does not itself assert the
  Header-wraps-Trigger nesting (its Header coverage is structural only).
  `docs/src/app/(docs)/react/components/accordion/page.mdx:15-28`
- "You can set up the accordion to allow multiple panels to be open at the same time using the
  `multiple` prop." Matches behavior.md "State model (controlled/uncontrolled, defaults,
  transitions)": with `multiple`, each item opens/closes independently and only the toggled item
  is affected.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:34`
- "The `hiddenUntilFound` prop hides closed panels with `hidden="until-found"` so the browser can
  search their contents and reveal the matching panel automatically. It can be set on each
  `Accordion.Panel`, or once on `Accordion.Root` to apply to all panels." The per-panel/root
  placement and the `hidden="until-found"` rendering match behavior.md "DOM structure & portal
  behavior" (`hiddenUntilFound` (root or panel) renders the closed panel with
  `hidden="until-found"`). The browser search/reveal part of the claim is not exercised by any
  test recorded in behavior.md (only the attribute rendering is).
  `docs/src/app/(docs)/react/components/accordion/page.mdx:42`
- Find-in-page usage instruction: press Ctrl+F (Cmd+F on macOS) and search for "restocking" —
  "the browser opens the closed panel containing the match." Not covered by behavior.md (no test
  asserts browser find-in-page reveal); see Discrepancies.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:44`
- "When `hiddenUntilFound` is enabled, closed panels always remain mounted in the DOM, which also
  makes their contents indexable by search engines." The always-mounted claim matches behavior.md
  "Edge cases (rapid interactions, unmount, nesting)" (`hiddenUntilFound` forces panels to stay
  mounted; `keepMounted={false}` is ignored). The search-engine indexability claim is a docs-only
  assertion with no test counterpart; see Discrepancies.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:44`
- "Older browsers that don't support `hidden="until-found"` keep panels hidden until their trigger
  opens them, and find-in-page skips over the contents." Not covered by behavior.md (no test
  addresses unsupported browsers); see Discrepancies.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:46`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the five parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesAccordion.Root />`,
  `### Item` → `<TypesAccordion.Item />`, `### Header` → `<TypesAccordion.Header />`,
  `### Trigger` → `<TypesAccordion.Trigger />`, `### Panel` → `<TypesAccordion.Panel />`.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:52-74`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesAccordion } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:54`
- Parts documented on this page: Root, Item, Header, Trigger, Panel — the same five-part set
  recorded in behavior.md "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/accordion/page.mdx:56-74`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- One fenced code block, the Anatomy snippet (` ```jsx title="Anatomy" `):
  imports `{ Accordion }` from `@base-ui/react/accordion` and assembles
  `<Accordion.Root>` > `<Accordion.Item>` > `<Accordion.Header>` containing
  `<Accordion.Trigger />`, with `<Accordion.Panel />` as a sibling of the Header.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:17-28`
- No other code blocks exist in the page. The "Open multiple panels" and "Hidden until found"
  examples render demo components (`<DemoAccordionMultiple />`, `<DemoAccordionHiddenUntilFound />`,
  plus `<DemoAccordionHero />` at the top) imported from `./demos/*`; their code lives in demo
  files and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/accordion/page.mdx:9-11`, `docs/src/app/(docs)/react/components/accordion/page.mdx:36-38`, `docs/src/app/(docs)/react/components/accordion/page.mdx:48-50`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/accordion/behavior.md`.
Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: browser find-in-page automatically revealing the
  matching closed panel (and the Ctrl/Cmd+F "restocking" walkthrough). behavior.md only verifies
  that closed panels render with `hidden="until-found"` under `hiddenUntilFound` ("DOM structure
  & portal behavior" section); no test asserts the search/reveal flow.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:42`, `docs/src/app/(docs)/react/components/accordion/page.mdx:44`
- Docs-only claim, not covered by behavior.md: kept-mounted `hiddenUntilFound` panels are
  "indexable by search engines" as a consequence of staying mounted.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:44`
- Docs-only claim, not covered by behavior.md: older browsers without `hidden="until-found"`
  support keep panels hidden until their trigger opens them, and find-in-page skips their
  contents (graceful-degradation behavior).
  `docs/src/app/(docs)/react/components/accordion/page.mdx:46`
- Omission (docs page vs. behavior.md): behavior.md "Edge cases (rapid interactions, unmount,
  nesting)" records that combining `hiddenUntilFound` with `keepMounted={false}` produces a
  `console.warn` ("Base UI: The `keepMounted={false}` prop ... is ignored when `hiddenUntilFound`
  is enabled ..."); the docs page states panels "always remain mounted" but never mentions this
  warning. Omission of a developer-facing caveat, not a contradiction of the mounted outcome.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:44`
- Note (not a mismatch): the page's Anatomy nests `Accordion.Trigger` inside `Accordion.Header`;
  behavior.md's Header coverage ("Public API surface (props, parts, subcomponents)") treats Header
  only as a structural wrapper and does not assert the header/trigger nesting either way.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:22-24`

## Cross-links to other docs pages

- N/A — the page contains no internal links to other Base UI documentation pages.
- One external link: MDN reference for the HTML `hidden` global attribute
  (`https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/hidden`),
  used to explain `hiddenUntilFound`. `docs/src/app/(docs)/react/components/accordion/page.mdx:42`
- Demo components (`./demos/hero`, `./demos/multiple`, `./demos/hidden-until-found`) are imported
  and rendered on this page itself; they are same-page imports, not cross-page links.
  `docs/src/app/(docs)/react/components/accordion/page.mdx:9`, `docs/src/app/(docs)/react/components/accordion/page.mdx:36`, `docs/src/app/(docs)/react/components/accordion/page.mdx:48`

## Snippet & behaviour contract

Per `specs/docs-content/CONTRACT.md` requirement 3: every example this page teaches, the Leptos
snippet that must carry it, the behavioural obligations cited to
`specs/library/accordion/behavior.md`, and the observable that proves each one. Authored 2026-09-16
by the `docs-content: components/accordion (prose + snippet completion)` item, which found the page's
single embedded code block carrying upstream's React source verbatim (gap-report probe:
`{total: 1, leptos: 0, react: 1}`) while every structural check passed.

The port's real surface is `leptos_ui`'s five parts — `AccordionRoot`, `AccordionItem`,
`AccordionHeader`, `AccordionTrigger`, `AccordionPanel` (`crates/leptos-ui/src/accordion/mod.rs`),
each a `#[component]` usable directly in `view!` markup — with state owned by the root's
single-open algebra (`crates/leptos-ui/src/accordion/mod.rs:100`). Upstream's names map
one-to-one onto them: `Accordion.Root` ↔ `AccordionRoot`, and so on down the tree.

The `## API reference` section carries no snippet of its own: it is upstream's generated
`<TypesAccordion.Root />` … `<TypesAccordion.Panel />` render, transcribed from
`docs/src/app/(docs)/react/components/accordion/types.md` and reproduced through the ported
reference primitives, and its obligations are the generated content itself (the row sets and order
are pinned by the page's `reference_content_guard`).

| example (upstream citation) | Leptos snippet to show | behavioural obligations (cited) | observable that proves it |
| --- | --- | --- | --- |
| Anatomy — assemble the parts (`docs/src/app/(docs)/react/components/accordion/page.mdx:17-28`) | import the port's five parts and nest them in `view!`: root > item > header > trigger, panel as the header's sibling inside the item | `specs/library/accordion/behavior.md` § Public API surface (the five parts, and which element each renders), § DOM structure & portal behavior (Header renders the heading element the trigger sits in) | the rendered tree contains the port's root `div` > item `div` > header `h3` wrapping the trigger `button`, with the panel `div` as the header's sibling inside the item (`render_test.rs`, `docs-content: components/accordion`) |
| Hero demo (`docs/src/app/(docs)/react/components/accordion/page.mdx:9-11`; source per `demos.json` entry `hero`) | the port's five parts composed the way the hero composes them, with the same upstream class names the demo passes, no value props on Root | `specs/library/accordion/behavior.md` § State model (uncontrolled, single-open default, all panels initially closed), § Accessibility (trigger `aria-expanded`/`aria-controls`), § DOM structure & portal behavior (closed panel unmounts unless kept mounted) | a real click on a trigger opens exactly one panel and closes the previously open one — the single-open algebra through the real machine (`accordion_hero_demo_toggles_through_the_real_port`) |
| Open multiple panels (`docs/src/app/(docs)/react/components/accordion/page.mdx:34-38`; source per `demos.json` entry `multiple`) | the same composition with `multiple=true` on the root | § State model (with `multiple`, each item opens/closes independently and only the toggled item is affected) | opening a second panel leaves the first open — two panels open at once through the real root (`accordion_multiple_demo_keeps_independent_panels_open`) |
| Hidden until found (`docs/src/app/(docs)/react/components/accordion/page.mdx:42-50`; source per `demos.json` entry `hidden-until-found`) | the same composition with `hidden_until_found=true` on the root | § DOM structure & portal behavior (a closed panel renders with `hidden="until-found"`), § Edge cases (`hiddenUntilFound` forces panels to stay mounted; `keepMounted={false}` is ignored) | after a full open/close cycle the closed panel is still in the DOM carrying `hidden="until-found"` (`accordion_hidden_until_found_demo_keeps_closed_panels_mounted`) |

Gaps carried open against this contract (do not mark this page's snippet work done over them):

* the Anatomy snippet's `AccordionTrigger`/`AccordionPanel` carry their content inline. Upstream's
  self-closing `<Accordion.Trigger />` / `<Accordion.Panel />` pass no children because React's
  `children` is optional; this port's parts take a required `children` prop, so the self-closing form
  does not compile (`crates/leptos-ui/src/accordion/mod.rs:473`, `crates/leptos-ui/src/accordion/mod.rs:634`).
  Upstream's own demo passes the questions and answers the same way.
* the snippet's `AccordionRoot`/`Accordion::Root` spelling: the contract's mapping table describes the
  namespaced form, which is `library: namespaced part surface (ported batch)`'s surface and does not
  exist yet. This page teaches the port's current public API, per the snippet-translation queue's own
  instruction to translate to it now rather than wait for the namespaced batch.
* the snippet-language probe classified an idiomatic Leptos `view!` composition as React (its JSX-tag
  heuristic matches `<AccordionRoot>`). The classifier's exclusive-marker rule was added by this item
  in `crates/docs-app/src/snippet_language.rs` and both probe copies; the finding is recorded in
  `ralph/logs/spec-discrepancies.md`, per that file's "a false `react` count must not be 'fixed' by
  avoiding idiomatic Leptos".
* the generated type-definition bodies inside the `Additional Types` panels
  (`Accordion.Root.State`, `Accordion.Root.ChangeEventReason`, …) are **not** carried: their bodies
  are TypeScript code blocks, which `docs-chrome: code blocks` owns. The panels render in upstream's
  default state, hidden until targeted, and the reveal machinery is that item's too.
* upstream's page affordances (`View as Markdown`, `View source`, StackBlitz) and the demo file tabs
  are `docs-chrome` scope, not snippet-language scope.
* the find-in-page reveal walkthrough (`docs/src/app/(docs)/react/components/accordion/page.mdx:44`)
  and the search-engine indexability claim are docs-only assertions with no behavioural counterpart in
  `specs/library/accordion/behavior.md` — already recorded in this file's Discrepancies section; the
  page keeps upstream's prose but no test claims the behaviour.
