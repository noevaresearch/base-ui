# Number Field docs page content spec

Mined from `docs/src/app/(docs)/react/components/number-field/page.mdx` only. The component's own
behavior is covered by `specs/library/number-field/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/number-field/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/number-field/demos.json`).

## Page structure (headings, in order)

- `# Number Field` (h1) — `docs/src/app/(docs)/react/components/number-field/page.mdx:1`
- `## Usage guidelines` — `docs/src/app/(docs)/react/components/number-field/page.mdx:14`
- `## Anatomy` — `docs/src/app/(docs)/react/components/number-field/page.mdx:18`
- `## API reference` — `docs/src/app/(docs)/react/components/number-field/page.mdx:37`
  - `### Root` — `docs/src/app/(docs)/react/components/number-field/page.mdx:41`
  - `### ScrubArea` — `docs/src/app/(docs)/react/components/number-field/page.mdx:45`
  - `### ScrubAreaCursor` — `docs/src/app/(docs)/react/components/number-field/page.mdx:49`
  - `### Group` — `docs/src/app/(docs)/react/components/number-field/page.mdx:53`
  - `### Decrement` — `docs/src/app/(docs)/react/components/number-field/page.mdx:57`
  - `### Input` — `docs/src/app/(docs)/react/components/number-field/page.mdx:61`
  - `### Increment` — `docs/src/app/(docs)/react/components/number-field/page.mdx:65`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A numeric input element with increment and decrement buttons, and a scrub area." — `docs/src/app/(docs)/react/components/number-field/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React number field component with increment and decrement buttons, and a scrub area." — `docs/src/app/(docs)/react/components/number-field/page.mdx:5-8`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/number-field/page.mdx:10-12`
- `TypesNumberField` import for the API reference — `docs/src/app/(docs)/react/components/number-field/page.mdx:39`
- Trailing `export const metadata` SEO keywords block (14 keywords, e.g. 'React Number Field', 'Number Spinner', 'Spin Button', 'Scrub Area Input') — `docs/src/app/(docs)/react/components/number-field/page.mdx:69-86`

## Prose claims about component behavior

- Subtitle describes the component as "A numeric input element with increment and decrement
  buttons, and a scrub area." Naming-level claim only; consistent with the part set recorded in
  behavior.md "Public API surface (props, parts, subcomponents)" (Increment/Decrement render
  native buttons; ScrubArea is a span hosting ScrubAreaCursor) and "DOM structure & portal
  behavior". `docs/src/app/(docs)/react/components/number-field/page.mdx:3`
- Meta description adds "high-quality, unstyled" — positioning-level claim only; behavior.md
  records no styling behavior one way or the other. `docs/src/app/(docs)/react/components/number-field/page.mdx:5-8`
- Usage guideline: "**Form controls must have an accessible name**: It can be created using a
  `<label>` element or the `Field` component. See the forms guide." Consistent with behavior.md
  "Accessibility (roles, aria-*, id linking)", which records Field integration linking
  `Field.Label` via `for` and appending `Field.Description` to `aria-describedby`. The bare
  `<label>`-element path is generic forms guidance; no NumberField-specific test for it is
  recorded in behavior.md (see Discrepancies). `docs/src/app/(docs)/react/components/number-field/page.mdx:16`
- Anatomy usage guidance: "Import the component and assemble its parts", showing the seven-part
  assembly (`NumberField.Root` containing a `ScrubArea` wrapping `ScrubAreaCursor`, plus a `Group`
  containing `Decrement`, `Input`, `Increment` in that order), imported from the
  `@base-ui/react/number-field` namespace. The part set matches behavior.md "Public API surface
  (props, parts, subcomponents)"; the grouping/order shown is the docs page's own structural
  suggestion (see Discrepancies). `docs/src/app/(docs)/react/components/number-field/page.mdx:20-35`

These are the only prose claims on the page; the page contains no other behavioral guidance.

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the seven parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesNumberField.Root />`,
  `### ScrubArea` → `<TypesNumberField.ScrubArea />`,
  `### ScrubAreaCursor` → `<TypesNumberField.ScrubAreaCursor />`,
  `### Group` → `<TypesNumberField.Group />`,
  `### Decrement` → `<TypesNumberField.Decrement />`,
  `### Input` → `<TypesNumberField.Input />`,
  `### Increment` → `<TypesNumberField.Increment />`.
  `docs/src/app/(docs)/react/components/number-field/page.mdx:41-67`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesNumberField } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/number-field/page.mdx:39`
- Parts documented on this page: Root, ScrubArea, ScrubAreaCursor, Group, Decrement, Input,
  Increment — the same seven-part set recorded in behavior.md "Public API surface (props, parts,
  subcomponents)". `docs/src/app/(docs)/react/components/number-field/page.mdx:41-67`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- One fenced code block, the Anatomy snippet (` ```jsx title="Anatomy" `): imports
  `{ NumberField }` from `@base-ui/react/number-field` and assembles
  `<NumberField.Root>` containing `<NumberField.ScrubArea>` (wrapping
  `<NumberField.ScrubAreaCursor />`) and `<NumberField.Group>` containing
  `<NumberField.Decrement />`, `<NumberField.Input />`, `<NumberField.Increment />` in that
  order. `docs/src/app/(docs)/react/components/number-field/page.mdx:22-35`
- No other code blocks exist in the page. The page's only other rendered content is the hero
  demo component (`<DemoNumberFieldHero />`, imported from `./demos/hero`); its code lives in a
  demo file and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/number-field/page.mdx:10-12`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/number-field/behavior.md`.
Items flagged for ambiguity, partial coverage, or omission:

- Nesting ambiguity (not a mismatch): the Anatomy snippet nests `ScrubArea` (with its
  `ScrubAreaCursor`) as a direct child of `Root`, a sibling of `Group`; behavior.md's composition
  line ("DOM structure & portal behavior") reads "Root (div) > Group (div) > Input +
  Increment/Decrement (buttons) + ScrubArea (span)", which could be read as ScrubArea inside
  Group. No test recorded in behavior.md asserts sibling/nesting order among the parts (only the
  hidden input's exact position is explicitly unasserted), so the two sources are compatible —
  flagged so downstream passes do not over-derive part nesting from either.
  `docs/src/app/(docs)/react/components/number-field/page.mdx:25-33`
- Docs-only claim, not covered by behavior.md: creating an accessible name "using a `<label>`
  element". behavior.md "Accessibility (roles, aria-*, id linking)" only records Field-based
  labeling (`Field.Label` linked via `for`); no test covers a bare `<label>` element associated
  with the NumberField input. The `Field` component path of the same claim is covered.
  `docs/src/app/(docs)/react/components/number-field/page.mdx:16`
- Omission (docs page vs. behavior.md): the page prose describes no interactive behavior —
  stepping, scrubbing, hold-to-repeat, wheel scrub, formatting/locale handling, the two-callback
  commit model, validation, and `disabled`/`readOnly` semantics are all absent from the page text
  (delegated to the generated `TypesNumberField` tables and to demos). Intentional page
  leanness, not a contradiction; behavior.md remains the authoritative behavior source.
  `docs/src/app/(docs)/react/components/number-field/page.mdx:14-67`

## Cross-links to other docs pages

- One internal cross-link: the forms guide at `/react/handbook/forms`, referenced from the
  accessible-name usage guideline. `docs/src/app/(docs)/react/components/number-field/page.mdx:16`
- Same-page imports (not cross-page links): `./demos/hero` (`DemoNumberFieldHero`) and `./types`
  (`TypesNumberField`) are imported and rendered on this page itself.
  `docs/src/app/(docs)/react/components/number-field/page.mdx:10`, `docs/src/app/(docs)/react/components/number-field/page.mdx:39`
