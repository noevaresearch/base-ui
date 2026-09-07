# Progress docs page content spec

Mined from `docs/src/app/(docs)/react/components/progress/page.mdx` only. The component's own
behavior is covered by `specs/library/progress/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/progress/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/progress/demos.json`).

## Page structure (headings, in order)

- `# Progress` (h1) — `docs/src/app/(docs)/react/components/progress/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/progress/page.mdx:13`
- `## API reference` — `docs/src/app/(docs)/react/components/progress/page.mdx:29`
  - `### Root` — `docs/src/app/(docs)/react/components/progress/page.mdx:33`
  - `### Track` — `docs/src/app/(docs)/react/components/progress/page.mdx:37`
  - `### Indicator` — `docs/src/app/(docs)/react/components/progress/page.mdx:41`
  - `### Value` — `docs/src/app/(docs)/react/components/progress/page.mdx:45`
  - `### Label` — `docs/src/app/(docs)/react/components/progress/page.mdx:49`

Non-heading page furniture, in document order:

- `<Subtitle>` — "Displays the status of a task that takes a long time." — `docs/src/app/(docs)/react/components/progress/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React progress bar component that displays the status of a task that takes a long time." — `docs/src/app/(docs)/react/components/progress/page.mdx:4-7`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/progress/page.mdx:9-11`
- `TypesProgress` import for the API reference — `docs/src/app/(docs)/react/components/progress/page.mdx:31`
- Trailing `export const metadata` SEO keywords block (14 keywords, e.g. 'React Progress Bar', 'Indeterminate Progress') — `docs/src/app/(docs)/react/components/progress/page.mdx:53-70`

## Prose claims about component behavior

- Page describes the component as one that "Displays the status of a task that takes a long
  time." and, in the meta description, as "unstyled". Purpose- and naming-level claims only;
  consistent with the `progressbar` role and value/valuetext rendering in behavior.md
  "Accessibility (roles, aria-*, id linking)". behavior.md makes no styling assertions either
  way, so the "unstyled" claim is unchecked (not contradicted).
  `docs/src/app/(docs)/react/components/progress/page.mdx:3`, `docs/src/app/(docs)/react/components/progress/page.mdx:4-7`
- Anatomy usage guidance: "Import the component and assemble its parts", showing the assembly
  `Progress.Root` > `Progress.Label`, `Progress.Track` (containing `Progress.Indicator`), and
  `Progress.Value` as siblings, imported from the `@base_ui/react/progress`-style namespace
  `@base-ui/react/progress`. The five-part set (Root/Label/Track/Indicator/Value) matches
  behavior.md "Public API surface (props, parts, subcomponents)"; behavior.md records all five
  parts as plain children of `Progress.Root` and does not assert the Track-contains-Indicator
  nesting either way (see Discrepancies).
  `docs/src/app/(docs)/react/components/progress/page.mdx:15-27`
- N/A beyond the above — the page body contains no other behavioral prose: no props are
  described, no usage guidance beyond assembly, and no caveats are stated anywhere in the
  .mdx text. `docs/src/app/(docs)/react/components/progress/page.mdx:15`, `docs/src/app/(docs)/react/components/progress/page.mdx:29-51`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the five parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesProgress.Root />`,
  `### Track` → `<TypesProgress.Track />`, `### Indicator` → `<TypesProgress.Indicator />`,
  `### Value` → `<TypesProgress.Value />`, `### Label` → `<TypesProgress.Label />`.
  `docs/src/app/(docs)/react/components/progress/page.mdx:29-51`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesProgress } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/progress/page.mdx:31`
- Parts documented on this page, in page order: Root, Track, Indicator, Value, Label — the same
  five-part set recorded in behavior.md "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/progress/page.mdx:33-51`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- One fenced code block, the Anatomy snippet (` ```jsx title="Anatomy" `): imports
  `{ Progress }` from `@base-ui/react/progress` and assembles `<Progress.Root>` containing
  `<Progress.Label />`, `<Progress.Track>` wrapping `<Progress.Indicator />`, and
  `<Progress.Value />` as siblings. No props are passed to any part in the snippet.
  `docs/src/app/(docs)/react/components/progress/page.mdx:17-27`
- No other code blocks exist in the page. The only rendered component besides the Anatomy is
  the hero demo (`<DemoProgressHero />`), imported from `./demos/hero`; its code lives in a
  demo file and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/progress/page.mdx:9-11`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/progress/behavior.md`.
Items flagged for lack of coverage or omission:

- Omission (docs page vs. behavior.md): the page body never mentions indeterminate behavior at
  all — behavior.md "State model (controlled/uncontrolled, defaults, transitions)" and "Edge
  cases (rapid interactions, unmount, nesting)" record `value={null}`/non-finite →
  `data-indeterminate`, `'indeterminate progress'` valuetext, and empty `Progress.Value`, none
  of which is explained in the .mdx text. The only trace is the trailing SEO keyword block
  listing 'Determinate Progress' and 'Indeterminate Progress' as search keywords, concepts the
  page body never introduces. `docs/src/app/(docs)/react/components/progress/page.mdx:53-70`
- Omission (docs page vs. behavior.md): no props are documented in page prose or shown in the
  Anatomy snippet, while behavior.md "Public API surface (props, parts, subcomponents)" records
  exercised props (`value`, `min`/`max`, `format`, `locale`, `getAriaValueText`). The API
  reference delegates all prop documentation to the generated `./types` components. Minimal
  page, not a contradiction. `docs/src/app/(docs)/react/components/progress/page.mdx:17-27`, `docs/src/app/(docs)/react/components/progress/page.mdx:31`
- Note (not a mismatch): the page's Anatomy nests `Progress.Indicator` inside `Progress.Track`;
  behavior.md "DOM structure & portal behavior" covers Indicator positioning/sizing
  (inline-start origin, percentage width) but records part composition only as "plain children
  of `Progress.Root`" and does not assert the Track>Indicator containment either way.
  `docs/src/app/(docs)/react/components/progress/page.mdx:22-24`
- Note (not a mismatch): the API reference orders parts Root, Track, Indicator, Value, Label,
  while the Anatomy snippet orders them Root, Label, Track, Indicator, Value; both are
  consistent with behavior.md "Public API surface (props, parts, subcomponents)", which
  imposes no part ordering.
  `docs/src/app/(docs)/react/components/progress/page.mdx:20-25`, `docs/src/app/(docs)/react/components/progress/page.mdx:33-51`

## Cross-links to other docs pages

- N/A — the page contains no internal links to other Base UI documentation pages and no
  external links. `docs/src/app/(docs)/react/components/progress/page.mdx:1-70`
- Same-directory module imports (`./demos/hero` at `docs/src/app/(docs)/react/components/progress/page.mdx:9`,
  `./types` at `docs/src/app/(docs)/react/components/progress/page.mdx:31`) are rendered on
  this page itself; they are same-page imports, not cross-page links.
