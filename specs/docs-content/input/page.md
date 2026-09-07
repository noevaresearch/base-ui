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
