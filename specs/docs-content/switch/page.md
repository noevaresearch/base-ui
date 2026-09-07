# Switch — docs page spec (Stage 1: docs content mining)

Source page: `docs/src/app/(docs)/react/components/switch/page.mdx` (118 lines).
Cross-check baseline: `specs/library/switch/behavior.md` (test-mined) — referenced below by section name only, not restated.
Demo source files (`./demos/hero`, etc.) are out of scope here; Stage 2 (`specs/docs-content/switch/demos.json`) mines those.

## Page structure (headings, in order)

- `# Switch` — page title (`docs/src/app/(docs)/react/components/switch/page.mdx:1`)
- Subtitle: "A control that indicates whether a setting is on or off." (`docs/src/app/(docs)/react/components/switch/page.mdx:3`)
- `<Meta name="description">` — "A high-quality, unstyled React switch component that indicates whether a setting is on or off." (`docs/src/app/(docs)/react/components/switch/page.mdx:4-7`)
- Hero demo embed — imports `DemoSwitchHero` from `./demos/hero` and renders it before any heading (`docs/src/app/(docs)/react/components/switch/page.mdx:9-11`)
- `## Usage guidelines` (`docs/src/app/(docs)/react/components/switch/page.mdx:13`)
- `## Anatomy` (`docs/src/app/(docs)/react/components/switch/page.mdx:17`)
- `## Examples` (`docs/src/app/(docs)/react/components/switch/page.mdx:29`)
  - `### Labeling a switch` (`docs/src/app/(docs)/react/components/switch/page.mdx:31`)
  - `### Rendering as a native button` (`docs/src/app/(docs)/react/components/switch/page.mdx:44`)
  - `### Form integration` (`docs/src/app/(docs)/react/components/switch/page.mdx:74`)
- `## API reference` (`docs/src/app/(docs)/react/components/switch/page.mdx:90`)
  - `### Root` (`docs/src/app/(docs)/react/components/switch/page.mdx:94`)
  - `### Thumb` (`docs/src/app/(docs)/react/components/switch/page.mdx:98`)
- `export const metadata` SEO keywords block closing the page (`docs/src/app/(docs)/react/components/switch/page.mdx:102-118`)

## Prose claims about component behavior

1. "Form controls must have an accessible name", creatable via a `<label>` element or the `Field` component, with links to the labeling example and the forms guide (`docs/src/app/(docs)/react/components/switch/page.mdx:15`). Cross-check: consistent with behavior.md § Accessibility (sibling `htmlFor` labels produce a fallback `aria-labelledby`, and `Field.Label` associations are test-proven there).
2. Anatomy guidance: "Import the component and assemble its parts" — the component is imported from `@base-ui/react/switch` and consists of `Switch.Root` wrapping `Switch.Thumb` (`docs/src/app/(docs)/react/components/switch/page.mdx:19-27`). Cross-check: consistent with behavior.md § Public API surface (only `Root` and `Thumb` are public parts, and `Switch.Thumb` must be placed within `Switch.Root` per its context-missing error).
3. "An enclosing `<label>` is the simplest labeling pattern" (`docs/src/app/(docs)/react/components/switch/page.mdx:33`). Cross-check: consistent with behavior.md § Accessibility (label linking through the hidden input) and § State model (`readOnly` blocks label clicks, implying label clicks toggle the switch by default).
4. "By default, `<Switch.Root>` renders a `<span>` element to support enclosing labels. Prefer rendering the switch as a native button when using sibling labels (`htmlFor`/`id`)." (`docs/src/app/(docs)/react/components/switch/page.mdx:46`). Cross-check: the span default and `nativeButton` are test-proven in behavior.md § Public API surface and § DOM structure; the sibling-label pairing is consistent with behavior.md § Accessibility (sibling `htmlFor` label produces fallback `aria-labelledby`; with `nativeButton` the user's `id` is placed on the visible switch element, not the hidden input).
5. "Native buttons with wrapping labels are supported by using the `render` callback to avoid invalid HTML, so the hidden input is placed outside the label" (`docs/src/app/(docs)/react/components/switch/page.mdx:58`). Cross-check: `nativeButton` + `render` composition is test-proven in behavior.md § DOM structure, but that spec never asserts the hidden input's placement relative to a wrapping label or the invalid-HTML rationale — flagged under Discrepancies.
6. "Use Field to handle label associations and form integration" (`docs/src/app/(docs)/react/components/switch/page.mdx:76`). Cross-check: consistent with behavior.md § State model (Field lifecycle data attributes: `data-touched`, `data-dirty`, `data-filled`, `data-focused`, `data-disabled`) and § Accessibility (`Field.Label` implicit/explicit association semantics).

## API tables referenced (props/parts documented on this page)

- The page embeds no literal prop tables; both API reference sections render generated type-table components imported from `./types` (`docs/src/app/(docs)/react/components/switch/page.mdx:92`).
- `### Root` renders `<TypesSwitch.Root />` (`docs/src/app/(docs)/react/components/switch/page.mdx:94-96`).
- `### Thumb` renders `<TypesSwitch.Thumb />` (`docs/src/app/(docs)/react/components/switch/page.mdx:98-100`).
- Parts documented on the page: `Switch.Root` and `Switch.Thumb` only — matching behavior.md § Public API surface, which likewise lists just these two public parts.
- Props demonstrated in prose/examples rather than tables: `nativeButton` (element-render example, `docs/src/app/(docs)/react/components/switch/page.mdx:52`; render-callback example, `docs/src/app/(docs)/react/components/switch/page.mdx:62`), `render` (element form, `docs/src/app/(docs)/react/components/switch/page.mdx:52`; callback form, `docs/src/app/(docs)/react/components/switch/page.mdx:64-69`), `id` (`docs/src/app/(docs)/react/components/switch/page.mdx:52`), and `Field.Root`'s `name` (`docs/src/app/(docs)/react/components/switch/page.mdx:81`).

## Code snippets embedded directly in the .mdx (not pulled from demos/)

1. Anatomy: import from `@base-ui/react/switch` plus `<Switch.Root><Switch.Thumb /></Switch.Root>` assembly, `title="Anatomy"` (`docs/src/app/(docs)/react/components/switch/page.mdx:21-27`).
2. Wrapping-label pattern: `<label><Switch.Root /> Notifications</label>`, marked with `// @highlight` (`docs/src/app/(docs)/react/components/switch/page.mdx:35-42`).
3. Sibling-label pattern with native button: `<label htmlFor="notifications-switch">` + `<Switch.Root id="notifications-switch" nativeButton render={<button />}>` with a `Switch.Thumb` child, using the `@highlight-text "nativeButton" "render={<button />}"` directive (`docs/src/app/(docs)/react/components/switch/page.mdx:48-56`).
4. Render-callback pattern: `nativeButton` with `render={(buttonProps) => <label><button {...buttonProps} /> Notifications</label>}`, wrapped in `@highlight-start`/`@highlight-end` markers (`docs/src/app/(docs)/react/components/switch/page.mdx:60-72`).
5. Form integration: `<Form><Field.Root name="notifications"><Field.Label><Switch.Root /> Notifications</Field.Label></Field.Root></Form>`, `title="Using Switch in a form"` (`docs/src/app/(docs)/react/components/switch/page.mdx:78-88`).

Snippets 2–5 reference `Switch`/`Form`/`Field` without their own imports; only snippet 1 shows an import statement. Snippets are illustrative and are not the hero demo (the hero demo is the imported `./demos/hero` component, `docs/src/app/(docs)/react/components/switch/page.mdx:9-11`).

## Discrepancies (docs page vs. behavior.md)

- Direct mismatches: none found — every test-proven claim the page makes (span default, `nativeButton`, label patterns, Field composition) agrees with behavior.md.
- Docs claims with no counterpart (not contradicted, but unproven) in behavior.md:
  - The rationale "renders a `<span>` … to support enclosing labels" (`docs/src/app/(docs)/react/components/switch/page.mdx:46`) — behavior.md § Public API surface proves the span default but never asserts the enclosing-label motivation.
  - "to avoid invalid HTML, so the hidden input is placed outside the label" for native-button + wrapping-label render callbacks (`docs/src/app/(docs)/react/components/switch/page.mdx:58`) — behavior.md § DOM structure asserts the hidden input exists and, with `nativeButton`, doesn't take the user's `id`, but says nothing about its placement relative to a wrapping label or about HTML validity.
  - Marketing claims in the subtitle/meta ("high-quality, unstyled") (`docs/src/app/(docs)/react/components/switch/page.mdx:3-7`) — no behavioral counterpart; not test-verifiable.
- behavior.md content absent from the page (absence is not a contradiction, noted for downstream completeness): the full state API (`checked`, `defaultChecked`, `onCheckedChange` with `eventDetails.cancel()`), `disabled`/`readOnly`/`required`/`aria-*` semantics, form submission values (`value`/`uncheckedValue`, `form`), keyboard interactions (`Enter`/`Space`), validation (`aria-invalid`, `Field.Error`), `data-*` state attributes, and the `SwitchRootContext`-missing error are all covered by behavior.md but never mentioned in the page prose.

## Cross-links to other docs pages

- `[forms guide](/react/handbook/forms)` → `/react/handbook/forms` (`docs/src/app/(docs)/react/components/switch/page.mdx:15`).
- `[Labeling a switch](#labeling-a-switch)` → same-page anchor (`docs/src/app/(docs)/react/components/switch/page.mdx:15`).
- `[Field](/react/components/field)` → `/react/components/field` (`docs/src/app/(docs)/react/components/switch/page.mdx:76`).
- Local (non-cross-link) imports on the page: `./demos/hero` (`docs/src/app/(docs)/react/components/switch/page.mdx:9`) and `./types` (`docs/src/app/(docs)/react/components/switch/page.mdx:92`).
