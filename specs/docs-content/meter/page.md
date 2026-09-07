# Meter docs page content spec

Mined from `docs/src/app/(docs)/react/components/meter/page.mdx` only. The component's own
behavior is covered by `specs/library/meter/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/meter/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/meter/demos.json`).

## Page structure (headings, in order)

- `# Meter` (h1) — `docs/src/app/(docs)/react/components/meter/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/meter/page.mdx:13`
- `## API reference` — `docs/src/app/(docs)/react/components/meter/page.mdx:29`
  - `### Root` — `docs/src/app/(docs)/react/components/meter/page.mdx:33`
  - `### Track` — `docs/src/app/(docs)/react/components/meter/page.mdx:37`
  - `### Indicator` — `docs/src/app/(docs)/react/components/meter/page.mdx:41`
  - `### Value` — `docs/src/app/(docs)/react/components/meter/page.mdx:45`
  - `### Label` — `docs/src/app/(docs)/react/components/meter/page.mdx:49`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A graphical display of a numeric value within a range." — `docs/src/app/(docs)/react/components/meter/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React meter component that provides a graphical display of a numeric value." — `docs/src/app/(docs)/react/components/meter/page.mdx:4-7`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/meter/page.mdx:9-11`
- `TypesMeter` import for the API reference section — `docs/src/app/(docs)/react/components/meter/page.mdx:31`
- Trailing `export const metadata` SEO keywords block (14 keywords, e.g. 'React Meter', 'Fuel
  Gauge', 'Accessible Meter') — `docs/src/app/(docs)/react/components/meter/page.mdx:53-70`

## Prose claims about component behavior

- Subtitle positioning claim: "A graphical display of a numeric value within a range." The
  "within a range" framing is consistent with behavior.md "State model (controlled/uncontrolled,
  defaults, transitions)" (`value` clamped into `[min, max]`; `min` 0 / `max` 100 defaults) and
  "Accessibility (roles, aria-*, id linking)" (`aria-valuemin`/`aria-valuemax` exposed on the
  root). `docs/src/app/(docs)/react/components/meter/page.mdx:3`
- Meta description: "A high-quality, unstyled React meter component that provides a graphical
  display of a numeric value." The "unstyled" positioning is a library-level claim with no
  counterpart in behavior.md; see Discrepancies.
  `docs/src/app/(docs)/react/components/meter/page.mdx:4-7`
- Anatomy usage guidance: "Import the component and assemble its parts", showing the assembly
  `Meter.Root` containing `Meter.Label`, `Meter.Track` with `Meter.Indicator` inside, and
  `Meter.Value`, imported from the `@base-ui/react/meter` namespace. The five-part set and the
  Indicator-inside-Track nesting match behavior.md "Public API surface (props, parts,
  subcomponents)" (parts exercised by tests: `Meter.Label`, `Meter.Track`, `Meter.Indicator`,
  `Meter.Value`, all nested under `Meter.Root`, with `Meter.Indicator` inside `Meter.Track`).
  The flat placement of `Meter.Label` and `Meter.Value` as direct children of `Meter.Root` is
  consistent with behavior.md "DOM structure & portal behavior" (label association is by id,
  not nesting; parts are optional and independently composable — `Meter.Value` also works
  without a Track/Indicator) and "Accessibility (roles, aria-*, id linking)" (`aria-labelledby`
  points at the label element's id). `docs/src/app/(docs)/react/components/meter/page.mdx:15-26`
- The page makes no other prose claims: no props are named in the text, and there is no usage
  guidance beyond the Anatomy sentence — no keyboard, focus, styling, or caveat text anywhere on
  the page. All prop documentation is delegated to the generated `TypesMeter` reference
  components under "API reference". `docs/src/app/(docs)/react/components/meter/page.mdx:13-51`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the five parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesMeter.Root />`, `### Track` →
  `<TypesMeter.Track />`, `### Indicator` → `<TypesMeter.Indicator />`, `### Value` →
  `<TypesMeter.Value />`, `### Label` → `<TypesMeter.Label />`.
  `docs/src/app/(docs)/react/components/meter/page.mdx:33-51`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesMeter } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page, and the page contains no
  literal Markdown API tables. `docs/src/app/(docs)/react/components/meter/page.mdx:31`
- Parts documented on this page: Root, Track, Indicator, Value, Label — the same five-part set
  recorded in behavior.md "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/meter/page.mdx:33-51`
- The page has no "Additional types" section — the API reference is the last content section
  before the trailing metadata export. `docs/src/app/(docs)/react/components/meter/page.mdx:33-70`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- **"Anatomy"** (jsx): imports `{ Meter }` from `@base-ui/react/meter` and assembles
  `<Meter.Root>` containing `<Meter.Label />`, `<Meter.Track>` with `<Meter.Indicator />`
  inside it, and `<Meter.Value />`.
  `docs/src/app/(docs)/react/components/meter/page.mdx:17-27`
- No other code blocks exist in the page. The hero demo at the top is a rendered component
  (`<DemoMeterHero />` imported from `./demos/hero`); its code lives in demo files and is out of
  scope here (Stage 2). `docs/src/app/(docs)/react/components/meter/page.mdx:9-11`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/meter/behavior.md`.
Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: the meta description's "high-quality, unstyled"
  positioning. behavior.md records no styling coverage; the claim is library copy, not a
  behavior assertion. `docs/src/app/(docs)/react/components/meter/page.mdx:4-7`
- Docs-only claim, not covered by behavior.md: importing the component from the
  `@base-ui/react/meter` namespace. behavior.md does not record package import paths for any
  part. `docs/src/app/(docs)/react/components/meter/page.mdx:18`
- Omission (behavior.md documents, page never mentions in prose): `value`, `min`, `max`,
  `locale`, `format`, and `getAriaValueText` — all props exercised by the tests ("Public API
  surface (props, parts, subcomponents)") — are never named in the page's text; their
  documentation is delegated entirely to the generated `TypesMeter.*` reference components.
  Likewise the clamping/ratio derivation and the default percent `aria-valuetext` behavior
  ("State model (controlled/uncontrolled, defaults, transitions)", "Accessibility (roles,
  aria-*, id linking)") receive no prose treatment. Omissions of documented behavior, not
  contradictions of anything the page does say. `docs/src/app/(docs)/react/components/meter/page.mdx:13-51`
- Note (not a mismatch): the API reference lists the parts in the order Root, Track, Indicator,
  Value, Label, while the Anatomy snippet shows Label second (Root, Label, Track > Indicator,
  Value); behavior.md prescribes neither ordering.
  `docs/src/app/(docs)/react/components/meter/page.mdx:20-26`, `docs/src/app/(docs)/react/components/meter/page.mdx:33-51`

## Cross-links to other docs pages

- N/A — the page contains no internal links to other Base UI documentation pages and no external
  links. `docs/src/app/(docs)/react/components/meter/page.mdx:1-70`
- The demo component (`./demos/hero`) and the generated API reference (`./types`) are imported
  and rendered on this page itself; they are same-page imports, not cross-page links.
  `docs/src/app/(docs)/react/components/meter/page.mdx:9-11`, `docs/src/app/(docs)/react/components/meter/page.mdx:31`
