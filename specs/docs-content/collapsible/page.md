# Docs-content spec: Collapsible page

Mined from the docs page only (`docs/src/app/(docs)/react/components/collapsible/page.mdx`, 74 lines).
Component-behavior ground truth lives in `specs/library/collapsible/behavior.md` — its sections are
cited by name below, not restated. Demo sources (`./demos/hero`, `./types`) are out of scope here;
Stage 2 (`specs/docs-content/collapsible/demos.json`) covers them.

## Page structure (headings, in order)

- H1 `# Collapsible` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:1`
- `<Subtitle>` one-liner (page header block, not a heading): "A collapsible panel controlled by a button." — `docs/src/app/(docs)/react/components/collapsible/page.mdx:3`
- `<Meta name="description">` SEO block — `docs/src/app/(docs)/react/components/collapsible/page.mdx:4-7`
- Hero demo render: imports `DemoCollapsibleHero` from `./demos/hero` and renders it immediately after the header block — `docs/src/app/(docs)/react/components/collapsible/page.mdx:9-11`
- H2 `## Anatomy` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:13`
- H2 `## Examples` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:26`
  - H3 `### Hidden until found` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:28`
- H2 `## API reference` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:44`
  - H3 `### Root` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:48`
  - H3 `### Trigger` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:52`
  - H3 `### Panel` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:56`
- `export const metadata` block with SEO keywords (React Collapsible, Collapsible Component, Expandable Panel, Toggle Panel Button, Disclosure Widget, Show/Hide Content, Accordion Item, Accessible Disclosure, Headless React Components, Collapsible Trigger Panel, Base UI) — `docs/src/app/(docs)/react/components/collapsible/page.mdx:60-74`

## Prose claims about component behavior

- "A collapsible panel controlled by a button." (Subtitle) — `docs/src/app/(docs)/react/components/collapsible/page.mdx:3`. Consistent with the behavior spec's "DOM structure & portal behavior" section (Trigger renders a native `button`) and "Public API surface" section (Root/Trigger/Panel parts).
- Meta description: "A high-quality, unstyled React collapsible component that displays a panel controlled by a button." — `docs/src/app/(docs)/react/components/collapsible/page.mdx:6`. Marketing framing; no behavioral claim beyond the trigger-button pairing already covered above.
- Anatomy intro: "Import the component and assemble its parts" — `docs/src/app/(docs)/react/components/collapsible/page.mdx:15`. Consistent with the behavior spec's "Public API surface" section (`Collapsible.Root`, `Collapsible.Trigger`, `Collapsible.Panel` imported from the `@base-ui/react/collapsible` namespace).
- "The `hiddenUntilFound` prop hides the closed panel with `hidden="until-found"` so the browser can search its contents with find-in-page—Ctrl+F (Cmd+F on macOS)—and reveal the panel when a match is found." — `docs/src/app/(docs)/react/components/collapsible/page.mdx:30`. Consistent with the behavior spec's "State model" section (`hiddenUntilFound` forces the closed panel to stay mounted with `hidden="until-found"`) and its "Events" section (a native `beforematch` event opens such a panel through `onOpenChange`; the page never names `beforematch`, it only says "when a match is found").
- "The closed panel always remains mounted in the DOM, which also makes its contents indexable by search engines." — `docs/src/app/(docs)/react/components/collapsible/page.mdx:30`. The always-mounted half matches the behavior spec's "State model" section; the search-engine-indexability rationale is a docs-only claim the behavior spec neither proves nor contradicts.
- "Older browsers that don't support `hidden="until-found"` keep the panel hidden until its trigger opens it, and find-in-page skips over the contents." — `docs/src/app/(docs)/react/components/collapsible/page.mdx:32`. Not covered by the behavior spec (no cross-browser fallback behavior is tested); not contradicted either.
- The Hidden until found example defers its interactive demo to the Accordion page — `docs/src/app/(docs)/react/components/collapsible/page.mdx:42`. Navigation claim only; no behavior.

## API tables referenced (props/parts documented on this page)

- No literal Markdown tables. The `## API reference` section renders generated type tables via `import { TypesCollapsible } from './types'` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:46` — with one generated component per part:
  - `### Root` → `<TypesCollapsible.Root />` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:48-50`
  - `### Trigger` → `<TypesCollapsible.Trigger />` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:52-54`
  - `### Panel` → `<TypesCollapsible.Panel />` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:56-58`
- Props named in the page's prose and inline snippets (not in tables): `hiddenUntilFound` (on `<Collapsible.Panel>`) — `docs/src/app/(docs)/react/components/collapsible/page.mdx:30`, `docs/src/app/(docs)/react/components/collapsible/page.mdx:38`. The full prop lists live inside the `./types` generated components and are out of scope here (Stage 2).
- The three parts documented match the behavior spec's "Public API surface" section exactly (Root, Trigger, Panel; no additional parts).

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- **"Anatomy"** (jsx, `title="Anatomy"`): `import { Collapsible } from '@base-ui/react/collapsible'` followed by `<Collapsible.Root>` wrapping `<Collapsible.Trigger />` and `<Collapsible.Panel />` — `docs/src/app/(docs)/react/components/collapsible/page.mdx:17-24`.
- **"Searchable hidden panel"** (tsx, `title="Searchable hidden panel"`): `<Collapsible.Root>` with `<Collapsible.Trigger>Shipping details</Collapsible.Trigger>` and `<Collapsible.Panel hiddenUntilFound>Standard shipping takes 3–5 business days.</Collapsible.Panel>`; carries an inline `{/* @highlight-text "hiddenUntilFound" */}` marker comment — `docs/src/app/(docs)/react/components/collapsible/page.mdx:34-40`. Static snippet embedded in the page; the page points to the Accordion docs page for the interactive version (`docs/src/app/(docs)/react/components/collapsible/page.mdx:42`).

## Discrepancies (docs page vs. behavior.md)

- None. No claim on the page contradicts `specs/library/collapsible/behavior.md`. Two docs-only claims are simply not covered (and not contradicted) by the behavior spec: the search-engine-indexability rationale for keeping the panel mounted (`docs/src/app/(docs)/react/components/collapsible/page.mdx:30`) and the older-browser fallback for `hidden="until-found"` (`docs/src/app/(docs)/react/components/collapsible/page.mdx:32`). The page's `hiddenUntilFound` claims otherwise align with the behavior spec's "State model", "Accessibility", and "Events" sections (the page's "reveal the panel when a match is found" corresponds to the spec's `beforematch`-open path).

## Cross-links to other docs pages

- `[Accordion example](/react/components/accordion#hidden-until-found)` — end of the Hidden until found example — `docs/src/app/(docs)/react/components/collapsible/page.mdx:42`
- `[hidden="until-found"](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/hidden)` — MDN, external site (not a docs page; listed for completeness) — `docs/src/app/(docs)/react/components/collapsible/page.mdx:30`
