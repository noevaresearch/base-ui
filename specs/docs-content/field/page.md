# Field docs page content spec

Mined from `docs/src/app/(docs)/react/components/field/page.mdx` only. The component's own
behavior is covered by `specs/library/field/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/field/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/field/demos.json`).

## Page structure (headings, in order)

- `# Field` (h1) — `docs/src/app/(docs)/react/components/field/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/field/page.mdx:14`
- `## API reference` — `docs/src/app/(docs)/react/components/field/page.mdx:31`
  - `### Root` — `docs/src/app/(docs)/react/components/field/page.mdx:35`
  - `### Label` — `docs/src/app/(docs)/react/components/field/page.mdx:39`
  - `### Control` — `docs/src/app/(docs)/react/components/field/page.mdx:43`
  - `### Description` — `docs/src/app/(docs)/react/components/field/page.mdx:47`
  - `### Item` — `docs/src/app/(docs)/react/components/field/page.mdx:51`
  - `### Error` — `docs/src/app/(docs)/react/components/field/page.mdx:55`
  - `### Validity` — `docs/src/app/(docs)/react/components/field/page.mdx:59`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A component that provides labeling and validation for form controls." — `docs/src/app/(docs)/react/components/field/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React field component that provides labeling and validation for form controls." — `docs/src/app/(docs)/react/components/field/page.mdx:5-8`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/field/page.mdx:10-12`
- Anatomy lead-in text ("Import the component and assemble its parts:") with its fenced code block — `docs/src/app/(docs)/react/components/field/page.mdx:16-29`
- `TypesField` import for the API reference — `docs/src/app/(docs)/react/components/field/page.mdx:33`
- Trailing `export const metadata` SEO keywords block (11 keywords, e.g. 'React Field Component', 'Accessible Form Field') — `docs/src/app/(docs)/react/components/field/page.mdx:63-77`

The page has no `## Examples` section: the only demo rendered is the hero at the top, and the
API reference follows the Anatomy section directly.
`docs/src/app/(docs)/react/components/field/page.mdx:10-31`

## Prose claims about component behavior

- Page describes the component as "A component that provides labeling and validation for form
  controls." and, in the meta description, as "unstyled". The labeling/validation framing is
  consistent with behavior.md: label↔control, description, and error association are covered
  under "Accessibility (roles, aria-*, id linking)", and validation under "State model
  (controlled/uncontrolled, defaults, transitions)" and "Events (names, payload shape, bubbling,
  preventDefault semantics)". The "unstyled" wording is a naming-level claim; behavior.md makes
  no styling assertions either way.
  `docs/src/app/(docs)/react/components/field/page.mdx:3`, `docs/src/app/(docs)/react/components/field/page.mdx:5-8`
- Anatomy usage guidance: "Import the component and assemble its parts", showing the assembly
  `Field.Root` with `Field.Label`, `Field.Control`, `Field.Description`, `Field.Item`,
  `Field.Error`, and `Field.Validity` as direct children, imported from the
  `@base-ui/react/field` namespace. The part set and namespace import match behavior.md
  "Public API surface (props, parts, subcomponents)" exactly (the same seven parts from a
  single `Field` barrel).
  `docs/src/app/(docs)/react/components/field/page.mdx:16-29`
- Those are the page's only prose claims: beyond the Anatomy lead-in and code block, the page
  contains no usage guidance, caveats, or prop-level explanation — the API reference section is
  entirely generated components.
  `docs/src/app/(docs)/react/components/field/page.mdx:16`, `docs/src/app/(docs)/react/components/field/page.mdx:31-61`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the seven parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesField.Root />`, `### Label` →
  `<TypesField.Label />`, `### Control` → `<TypesField.Control />`, `### Description` →
  `<TypesField.Description />`, `### Item` → `<TypesField.Item />`, `### Error` →
  `<TypesField.Error />`, `### Validity` → `<TypesField.Validity />`.
  `docs/src/app/(docs)/react/components/field/page.mdx:31-61`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesField } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/field/page.mdx:33`
- Parts documented on this page: Root, Label, Control, Description, Item, Error, Validity — the
  same seven-part set recorded in behavior.md "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/field/page.mdx:35-61`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- One fenced code block, the Anatomy snippet (` ```jsx title="Anatomy" `):
  imports `{ Field }` from `@base-ui/react/field` and assembles `<Field.Root>` containing
  `<Field.Label />`, `<Field.Control />`, `<Field.Description />`, `<Field.Item />`,
  `<Field.Error />`, and `<Field.Validity />` as direct children, closed with
  `</Field.Root>;`.
  `docs/src/app/(docs)/react/components/field/page.mdx:18-29`
- No other code blocks exist in the page. The hero demo (`<DemoFieldHero />`) is imported from
  `./demos/hero` and rendered at the top of the page; its code lives in demo files and is out
  of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/field/page.mdx:10-12`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/field/behavior.md`.
Items flagged for lack of coverage or omission:

- Omission (docs page vs. behavior.md): the page's only behavioral claim is the subtitle/meta
  framing that Field "provides labeling and validation for form controls", but the page contains
  no prose or examples about validation at all — nothing on `validate`, `validationMode`,
  `validationDebounceTime`, error display, dirty/touched, or Form integration, all of which
  behavior.md documents ("State model (controlled/uncontrolled, defaults, transitions)",
  "Events (names, payload shape, bubbling, preventDefault semantics)", "Edge cases (rapid
  interactions, unmount, nesting)"). Omission of coverage, not a contradiction of any claim.
  `docs/src/app/(docs)/react/components/field/page.mdx:3`, `docs/src/app/(docs)/react/components/field/page.mdx:5-8`
- Note (not a mismatch): the page's Anatomy renders `<Field.Validity />` with no children;
  behavior.md "Events (names, payload shape, bubbling, preventDefault semantics)" records
  `Field.Validity` as a children/render-function component that receives the validity state and
  "renders nothing itself" in tests, so the childless form shown would publish nothing on its
  own. The Anatomy is a part listing; behavior.md does not assert the anatomy's nesting.
  `docs/src/app/(docs)/react/components/field/page.mdx:27`
- Note (not a mismatch): the Anatomy places `Field.Item` as a direct child of `Field.Root`
  alongside Label/Control/Description/Error; behavior.md "Edge cases (rapid interactions,
  unmount, nesting)" only describes `Field.Item` scoping `disabled` to wrapped checkboxes/radios
  inside groups and does not assert Root-level placement either way.
  `docs/src/app/(docs)/react/components/field/page.mdx:25`

## Cross-links to other docs pages

- N/A — the page contains no internal links to other Base UI documentation pages and no
  external links.
- Same-page imports only: the hero demo (`./demos/hero`, rendered as `<DemoFieldHero />`) and
  the generated API reference (`./types`, imported as `TypesField`); these are same-page
  imports, not cross-page links.
  `docs/src/app/(docs)/react/components/field/page.mdx:10`, `docs/src/app/(docs)/react/components/field/page.mdx:33`
