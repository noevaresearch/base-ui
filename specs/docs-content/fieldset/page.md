# Fieldset docs page content spec

Mined from `docs/src/app/(docs)/react/components/fieldset/page.mdx` only. The component's own
behavior is covered by `specs/library/fieldset/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/fieldset/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/fieldset/demos.json`).

## Page structure (headings, in order)

- `# Fieldset` (h1) — `docs/src/app/(docs)/react/components/fieldset/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/fieldset/page.mdx:14`
- `## API reference` — `docs/src/app/(docs)/react/components/fieldset/page.mdx:26`
  - `### Root` — `docs/src/app/(docs)/react/components/fieldset/page.mdx:30`
  - `### Legend` — `docs/src/app/(docs)/react/components/fieldset/page.mdx:34`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A native fieldset element with an easily stylable legend." — `docs/src/app/(docs)/react/components/fieldset/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React fieldset component with an easily stylable legend." — `docs/src/app/(docs)/react/components/fieldset/page.mdx:5-8`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/fieldset/page.mdx:10-12`
- `TypesFieldset` import for the API reference — `docs/src/app/(docs)/react/components/fieldset/page.mdx:28`
- Trailing `export const metadata` SEO keywords block (12 keywords, e.g. 'React Fieldset', 'Custom Legend Styling') — `docs/src/app/(docs)/react/components/fieldset/page.mdx:38-53`

## Prose claims about component behavior

The page is prose-light: outside the Subtitle/meta description and the Anatomy usage line, it
contains no narrative text; the API reference renders generated type components only.

- Page describes the component as "A native fieldset element with an easily stylable legend."
  The "native fieldset element" claim matches behavior.md "DOM structure & portal behavior"
  (Root renders a native `<fieldset>` with an `HTMLFieldSetElement` ref) and "Accessibility
  (roles, aria-*, id linking)" (implicit `group` role). The "easily stylable legend" claim is a
  docs-only styling assertion with no behavior.md counterpart; it is consistent with behavior.md
  recording that the Legend renders an `HTMLDivElement` (not a native `<legend>` element) in
  "DOM structure & portal behavior", but neither source states the stylability rationale. See
  Discrepancies.
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:3`
- Meta description positions the component as "A high-quality, unstyled React fieldset component
  with an easily stylable legend." Naming/positioning-level claim only; the "unstyled" claim is
  consistent with the headless rendering recorded throughout behavior.md (no intrinsic styling is
  asserted anywhere in it).
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:5-8`
- Anatomy usage guidance: "Import the component and assemble its parts", showing a two-part
  assembly: `<Fieldset.Root>` containing a self-closing `<Fieldset.Legend />`, imported from the
  `@base-ui/react/fieldset` namespace. The two-part set (Root, Legend) and namespace import match
  behavior.md "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:16`, `docs/src/app/(docs)/react/components/fieldset/page.mdx:18-24`
- The page prose makes no claims about the `disabled` prop, nested-fieldset disabled
  propagation, legend `id`/`aria-labelledby` linking, SSR/hydration behavior, or the Legend
  context requirement — those behaviors are recorded in behavior.md ("State model
  (controlled/uncontrolled, defaults, transitions)", "Accessibility (roles, aria-*, id linking)",
  "Edge cases") but are not surfaced in the page text. See Discrepancies for the omission notes.

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the two parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesFieldset.Root />`,
  `### Legend` → `<TypesFieldset.Legend />`.
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:26-36`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesFieldset } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:28`
- Parts documented on this page: Root, Legend — the same two-part set recorded in behavior.md
  "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:30-36`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- One fenced code block, the Anatomy snippet (` ```jsx title="Anatomy" `):
  imports `{ Fieldset }` from `@base-ui/react/fieldset` and assembles
  `<Fieldset.Root>` containing `<Fieldset.Legend />`, with no children, props, or other parts
  shown.
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:18-24`
- No other code blocks exist in the page. The hero demo (`<DemoFieldsetHero />`) is imported and
  rendered at the top of the page from `./demos/hero`; its code lives in a demo file and is out
  of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:10-12`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/fieldset/behavior.md`.
Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: the legend is "easily stylable" (Subtitle and
  meta description). behavior.md records only that the Legend renders an `HTMLDivElement`-typed
  element ("DOM structure & portal behavior"); no stylability claim or test exists there.
  Plausibly motivated by the non-native legend element, but that rationale is stated in neither
  source.
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:3`, `docs/src/app/(docs)/react/components/fieldset/page.mdx:5-8`
- Omission (docs page vs. behavior.md): behavior.md "Public API surface (props, parts,
  subcomponents)" records that `Fieldset.Legend` throws when rendered outside `<Fieldset.Root>`
  (`Base UI: FieldsetRootContext is missing. Fieldset parts must be placed within <Fieldset.Root>.`);
  the page never states this context requirement. Omission of a developer-facing caveat, not a
  contradiction.
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:26-36`
- Omission (docs page vs. behavior.md): the `disabled` prop and its propagation semantics
  (ancestor-disabled rule, native `disabled` attribute, `data-disabled` on nested `Field.Root`
  and rendered group roots) are recorded in behavior.md ("State model (controlled/uncontrolled,
  defaults, transitions)", "Accessibility (roles, aria-*, id linking)", "Edge cases") but never
  appear in the page prose; they are only reachable via the generated `TypesFieldset.Root` table
  or the hero demo, both outside this file's prose scope. Not a contradiction.
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:30-32`
- Omission (docs page vs. behavior.md): the automatic `aria-labelledby` linking between Root and
  Legend (generated id, custom-id honoring, id-change tracking, unmount removal, and the SSR
  pre-hydration behavior) is recorded in behavior.md "Accessibility (roles, aria-*, id linking)"
  but is not mentioned anywhere on the page. Not a contradiction.
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:30-36`
- Note (not a mismatch): the Anatomy snippet renders `<Fieldset.Legend />` with no children;
  behavior.md "Public API surface (props, parts, subcomponents)" records that Legend accepts
  children rendered as legend content. Minimal-usage snippet, not a contradiction.
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:21-23`

## Cross-links to other docs pages

- N/A — the page contains no internal links to other Base UI documentation pages and no external
  links.
- The hero demo (`./demos/hero`) and the generated types module (`./types`) are imported and
  rendered on this page itself; they are same-page imports, not cross-page links.
  `docs/src/app/(docs)/react/components/fieldset/page.mdx:10`, `docs/src/app/(docs)/react/components/fieldset/page.mdx:28`
