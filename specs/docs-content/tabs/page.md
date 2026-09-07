# Tabs docs page content spec

Mined from `docs/src/app/(docs)/react/components/tabs/page.mdx` only. The component's own
behavior is covered by `specs/library/tabs/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/tabs/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/tabs/demos.json`).

## Page structure (headings, in order)

- `# Tabs` (h1) — `docs/src/app/(docs)/react/components/tabs/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/tabs/page.mdx:14`
- `## Examples` — `docs/src/app/(docs)/react/components/tabs/page.mdx:30`
  - `### Animated panels` — `docs/src/app/(docs)/react/components/tabs/page.mdx:32`
  - `### Links` — `docs/src/app/(docs)/react/components/tabs/page.mdx:41`
- `## API reference` — `docs/src/app/(docs)/react/components/tabs/page.mdx:62`
  - `### Root` — `docs/src/app/(docs)/react/components/tabs/page.mdx:66`
  - `### List` — `docs/src/app/(docs)/react/components/tabs/page.mdx:70`
  - `### Tab` — `docs/src/app/(docs)/react/components/tabs/page.mdx:74`
  - `### Indicator` — `docs/src/app/(docs)/react/components/tabs/page.mdx:78`
  - `### Panel` — `docs/src/app/(docs)/react/components/tabs/page.mdx:82`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A component for toggling between related panels on the same page." — `docs/src/app/(docs)/react/components/tabs/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React tabs component for toggling between related panels on the same page." — `docs/src/app/(docs)/react/components/tabs/page.mdx:5-8`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/tabs/page.mdx:10`, `docs/src/app/(docs)/react/components/tabs/page.mdx:12`
- Demo import interleaved with the Examples subsections (`./demos/animated-panels` at `docs/src/app/(docs)/react/components/tabs/page.mdx:37`, rendered at `docs/src/app/(docs)/react/components/tabs/page.mdx:39`)
- `TypesTabs` import for the API reference — `docs/src/app/(docs)/react/components/tabs/page.mdx:64`
- Trailing `export const metadata` SEO keywords block (17 keywords, e.g. 'React Tabs', 'Keyboard Navigation Tabs', 'Controlled Tabs') — `docs/src/app/(docs)/react/components/tabs/page.mdx:86-106`

## Prose claims about component behavior

- Page describes the component as "A component for toggling between related panels on the same
  page." and, in the meta description, as "unstyled". Naming-level purpose claim only; consistent
  with the five-part selection/panel model in behavior.md "Public API surface (props, parts,
  subcomponents)".
  `docs/src/app/(docs)/react/components/tabs/page.mdx:3`, `docs/src/app/(docs)/react/components/tabs/page.mdx:5-8`
- Anatomy usage guidance: "Import the component and assemble its parts", showing the assembly
  `Tabs.Root` > `Tabs.List` (containing `Tabs.Tab` and `Tabs.Indicator`) with `Tabs.Panel` as a
  sibling of the List, imported from the `@base-ui/react/tabs` namespace. The part set, namespace
  import, and Indicator-inside-List nesting all match behavior.md "Public API surface (props,
  parts, subcomponents)" and "DOM structure & portal behavior".
  `docs/src/app/(docs)/react/components/tabs/page.mdx:16-28`
- "Animate panels as they activate using the `data-starting-style` and `data-ending-style`
  attributes." Matches behavior.md "DOM structure & portal behavior" (panels receive
  `data-starting-style` on mount and complete an enter transition). Nuance on `data-ending-style`
  provenance is flagged under Discrepancies.
  `docs/src/app/(docs)/react/components/tabs/page.mdx:34`
- "The `data-activation-direction` attribute indicates which direction the newly active tab is
  relative to the previously active one, letting panels slide in from the correct side." The
  attribute semantics match behavior.md "State model (controlled/uncontrolled, defaults,
  transitions)" (`data-activation-direction` with `'none'`/`'right'`/`'left'`/`'down'`/`'up'`).
  The implication that panels carry the attribute themselves is flagged under Discrepancies.
  `docs/src/app/(docs)/react/components/tabs/page.mdx:35`
- "Use the `render` prop and set `nativeButton={false}` on `<Tabs.Tab>` to render tabs as anchor
  elements." Matches behavior.md "Public API surface (props, parts, subcomponents)" (both props
  proven on `Tabs.Tab`) and "DOM structure & portal behavior" (`nativeButton={false}` + `render`
  lets a tab render as `<a>` while keeping full selection behavior, including a router `Link`
  with its `href`).
  `docs/src/app/(docs)/react/components/tabs/page.mdx:43`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the five parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesTabs.Root />`, `### List` →
  `<TypesTabs.List />`, `### Tab` → `<TypesTabs.Tab />`, `### Indicator` →
  `<TypesTabs.Indicator />`, `### Panel` → `<TypesTabs.Panel />`.
  `docs/src/app/(docs)/react/components/tabs/page.mdx:62-84`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesTabs } from './types';`); no props, prop types, defaults, or prop descriptions
  are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/tabs/page.mdx:64`
- Parts documented on this page: Root, List, Tab, Indicator, Panel (docs heading order places
  Indicator before Panel) — the same five-part set recorded in behavior.md "Public API surface
  (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/tabs/page.mdx:66-84`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- Anatomy snippet (` ```jsx title="Anatomy" `): imports `{ Tabs }` from `@base-ui/react/tabs`
  and assembles `<Tabs.Root>` > `<Tabs.List>` containing `<Tabs.Tab />` then
  `<Tabs.Indicator />`, with `<Tabs.Panel />` as a sibling of the List.
  `docs/src/app/(docs)/react/components/tabs/page.mdx:18-28`
- Links snippet (` ```jsx title="Tabs as links" `): imports `{ Tabs }` from
  `@base-ui/react/tabs` and `Link` from `next/link`; shows `<Tabs.Tab nativeButton={false}
  render={<Link href="/overview" />} value="overview">Overview</Tabs.Tab>` inside a
  `<Tabs.List>`, with `@highlight-start` / `@highlight-text "nativeButton={false}" "render"` /
  `@highlight-end` comment markers wrapping the highlighted tab and a `{/* ... */}` placeholder
  for the rest.
  `docs/src/app/(docs)/react/components/tabs/page.mdx:45-60`
- No other code blocks exist in the page. The hero and "Animated panels" examples render demo
  components (`<DemoTabsHero />`, `<DemoTabsAnimatedPanels />`) imported from `./demos/*`; their
  code lives in demo files and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/tabs/page.mdx:10-12`, `docs/src/app/(docs)/react/components/tabs/page.mdx:37-39`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/tabs/behavior.md`.
Items flagged for provenance gaps or nuance:

- Coverage mismatch: the page implies the `data-activation-direction` attribute is available on
  panels ("letting panels slide in from the correct side"). behavior.md "State model
  (controlled/uncontrolled, defaults, transitions)" proves the attribute is stamped on the
  `Tabs.Root` element and on each tab element, and reaches panels only via render-prop state
  (`state.tabActivationDirection`); no test recorded in behavior.md asserts
  `data-activation-direction` on a panel DOM element. Verify against the implementation or the
  Stage 2 demo code before trusting either source.
  `docs/src/app/(docs)/react/components/tabs/page.mdx:35`
- Provenance nuance (not a contradiction): the page presents both `data-starting-style` and
  `data-ending-style` as attributes for animating panels "as they activate". behavior.md "DOM
  structure & portal behavior" proves `data-starting-style` on mount/enter but ties
  `data-ending-style` to the exit path (received before unmount, panel removed after the exit
  animation).
  `docs/src/app/(docs)/react/components/tabs/page.mdx:34`
- Note (not a mismatch): the Links example uses `next/link`'s `Link`, while behavior.md "DOM
  structure & portal behavior" records react-router `Link` in its equivalent test; both exercise
  the same `nativeButton={false}` + `render` anchor-rendering mechanism.
  `docs/src/app/(docs)/react/components/tabs/page.mdx:43`, `docs/src/app/(docs)/react/components/tabs/page.mdx:47`, `docs/src/app/(docs)/react/components/tabs/page.mdx:53`
- Note (not a mismatch): the page's prose covers only animation and anchor rendering; all other
  behavior.md-proven surface (`value`/`defaultValue` control, `orientation`, `activateOnFocus`,
  `loopFocus`, `keepMounted`, `renderBeforeHydration`, keyboard interactions, automatic
  re-selection fallbacks) is left to the generated API tables with no inline description in the
  .mdx.
  `docs/src/app/(docs)/react/components/tabs/page.mdx:30-60`

## Cross-links to other docs pages

- N/A — the page contains no internal links to other Base UI documentation pages and no external
  links.
- The `<Link href="/overview" />` usage is example code inside the Links snippet (a `next/link`
  import), not a docs cross-link.
  `docs/src/app/(docs)/react/components/tabs/page.mdx:47`, `docs/src/app/(docs)/react/components/tabs/page.mdx:53`
- Demo (`./demos/hero`, `./demos/animated-panels`) and types (`./types`) imports are same-page
  imports, not cross-page links.
  `docs/src/app/(docs)/react/components/tabs/page.mdx:10`, `docs/src/app/(docs)/react/components/tabs/page.mdx:37`, `docs/src/app/(docs)/react/components/tabs/page.mdx:64`
