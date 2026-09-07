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
