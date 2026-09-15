# Checkbox — docs page spec (Stage 1: docs content mining)

Source page: `docs/src/app/(docs)/react/components/checkbox/page.mdx` (121 lines).
Cross-check baseline: `specs/library/checkbox/behavior.md` (test-mined) — referenced below by section name only, not restated.
Demo source files (`./demos/hero`, etc.) are out of scope here; Stage 2 (`specs/docs-content/checkbox/demos.json`) mines those.

## Page structure (headings, in order)

- `# Checkbox` — page title (`docs/src/app/(docs)/react/components/checkbox/page.mdx:1`)
- Subtitle: "An easily stylable checkbox component." (`docs/src/app/(docs)/react/components/checkbox/page.mdx:3`)
- `<Meta name="description">` — "A high-quality, unstyled React checkbox component that is easy to customize." (`docs/src/app/(docs)/react/components/checkbox/page.mdx:4-7`)
- Hero demo embed — imports `DemoCheckboxBasic` from `./demos/hero` and renders it before any heading (`docs/src/app/(docs)/react/components/checkbox/page.mdx:9-11`)
- `## Usage guidelines` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:13`)
- `## Anatomy` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:17`)
- `## Examples` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:29`)
  - `### Labeling a checkbox` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:31`)
  - `### Rendering as a native button` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:44`)
  - `### Form integration` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:74`)
- `## API reference` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:90`)
  - `### Root` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:94`)
  - `### Indicator` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:98`)
- `export const metadata` SEO keywords block closing the page (`docs/src/app/(docs)/react/components/checkbox/page.mdx:102-121`)

## Prose claims about component behavior

1. "Form controls must have an accessible name", creatable via a `<label>` element or the `Field` component, with links to the labeling example and the forms guide (`docs/src/app/(docs)/react/components/checkbox/page.mdx:15`). Cross-check: consistent with behavior.md § Accessibility (label wrapping, `aria-labelledby`, Field `for`/id linking are all test-proven there).
2. Anatomy guidance: "Import the component and assemble its parts" — the component is imported from `@base-ui/react/checkbox` and consists of `Checkbox.Root` wrapping `Checkbox.Indicator` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:19-27`). Cross-check: consistent with behavior.md § Public API surface (only `Root` and `Indicator` are public parts).
3. "An enclosing `<label>` is the simplest labeling pattern" (`docs/src/app/(docs)/react/components/checkbox/page.mdx:33`). Cross-check: consistent with behavior.md § Edge cases (wrapping label clicks toggle the checkbox) and § Accessibility (implicit label association sets `aria-labelledby`).
4. "By default, `<Checkbox.Root>` renders a `<span>` element to support enclosing labels. Prefer rendering the checkbox as a native button when using sibling labels (`htmlFor`/`id`)." (`docs/src/app/(docs)/react/components/checkbox/page.mdx:46`). Cross-check: the span default and `nativeButton` are test-proven in behavior.md § Public API surface and § DOM structure; the sibling-label pairing is consistent with behavior.md § Accessibility (sibling `htmlFor` label produces fallback `aria-labelledby`).
5. "Native buttons with wrapping labels are supported by using the `render` callback to avoid invalid HTML, so the hidden input is placed outside the label" (`docs/src/app/(docs)/react/components/checkbox/page.mdx:58`). Cross-check: `nativeButton` + `render` composition is test-proven in behavior.md § DOM structure and § Edge cases, but that spec never asserts the hidden input's placement relative to a wrapping label or the invalid-HTML rationale — flagged under Discrepancies.
6. "Use Field to handle label associations and form integration" (`docs/src/app/(docs)/react/components/checkbox/page.mdx:76`). Cross-check: consistent with behavior.md § Public API surface (Field.Root composition) and § State model (Field lifecycle data attributes).

## API tables referenced (props/parts documented on this page)

- The page embeds no literal prop tables; both API reference sections render generated type-table components imported from `./types` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:92`).
- `### Root` renders `<TypesCheckbox.Root />` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:94-96`).
- `### Indicator` renders `<TypesCheckbox.Indicator />` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:98-100`).
- Parts documented on the page: `Checkbox.Root` and `Checkbox.Indicator` only — matching behavior.md § Public API surface, which likewise lists just these two public parts.
- Props demonstrated in prose/examples rather than tables: `nativeButton` (element-render example, `docs/src/app/(docs)/react/components/checkbox/page.mdx:52`; render-callback example, `:60-72`), `render` (element form `:52`, callback form `:64-69`), `id` (`:52`), and `Field.Root`'s `name` (`:81`).

## Code snippets embedded directly in the .mdx (not pulled from demos/)

1. Anatomy: import from `@base-ui/react/checkbox` plus `<Checkbox.Root><Checkbox.Indicator /></Checkbox.Root>` assembly, `title="Anatomy"` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:21-27`).
2. Wrapping-label pattern: `<label><Checkbox.Root /> Accept terms and conditions</label>`, marked with `// @highlight` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:35-42`).
3. Sibling-label pattern with native button: `<label htmlFor>` + `<Checkbox.Root id="notifications-checkbox" nativeButton render={<button />}>` with a `Checkbox.Indicator` child, using the `@highlight-text "nativeButton" "render={<button />}"` directive (`docs/src/app/(docs)/react/components/checkbox/page.mdx:48-56`).
4. Render-callback pattern: `nativeButton` with `render={(buttonProps) => <label><button {...buttonProps} /> Enable notifications</label>}`, wrapped in `@highlight-start`/`@highlight-end` markers (`docs/src/app/(docs)/react/components/checkbox/page.mdx:60-72`).
5. Form integration: `<Form><Field.Root name="stayLoggedIn"><Field.Label><Checkbox.Root /> Stay logged in for 7 days</Field.Label></Field.Root></Form>`, `title="Using Checkbox in a form"` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:78-88`).

Snippets 2–5 reference `Checkbox`/`Form`/`Field` without their own imports; only snippet 1 shows an import statement. Snippets are illustrative and are not the hero demo (the hero demo is the imported `./demos/hero` component, `docs/src/app/(docs)/react/components/checkbox/page.mdx:9-11`).

## Discrepancies (docs page vs. behavior.md)

- Direct mismatches: none found — every test-proven claim the page makes (span default, `nativeButton`, label patterns, Field composition) agrees with behavior.md.
- Docs claims with no counterpart (not contradicted, but unproven) in behavior.md:
  - The rationale "renders a `<span>` … to support enclosing labels" (`docs/src/app/(docs)/react/components/checkbox/page.mdx:46`) — behavior.md § Public API surface proves the span default but never asserts the enclosing-label motivation.
  - "to avoid invalid HTML, so the hidden input is placed outside the label" for native-button + wrapping-label render callbacks (`docs/src/app/(docs)/react/components/checkbox/page.mdx:58`) — behavior.md § DOM structure asserts the hidden input exists and doesn't steal ids in `nativeButton` mode, but says nothing about its placement relative to a wrapping label or about HTML validity.
  - Marketing claims in the subtitle/meta ("easily stylable", "high-quality, unstyled") (`docs/src/app/(docs)/react/components/checkbox/page.mdx:3-7`) — no behavioral counterpart; not test-verifiable.
- behavior.md content absent from the page (absence is not a contradiction, noted for downstream completeness): `indeterminate`, `onCheckedChange`/`eventDetails.cancel()`, form submission values (`value`/`uncheckedValue`), keyboard interactions (Space toggles, Enter submits under Form), `CheckboxGroup`/`parent` composition, and Indicator mount/animation lifecycle are all covered by behavior.md but never mentioned in the page prose.

## Cross-links to other docs pages

- `[forms guide](/react/handbook/forms)` → `/react/handbook/forms` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:15`).
- `[Labeling a checkbox](#labeling-a-checkbox)` → same-page anchor (`docs/src/app/(docs)/react/components/checkbox/page.mdx:15`).
- `[Field](/react/components/field)` → `/react/components/field` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:76`).
- Local (non-cross-link) imports on the page: `./demos/hero` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:9`) and `./types` (`docs/src/app/(docs)/react/components/checkbox/page.mdx:92`).

## Snippet & behaviour contract

Per `specs/docs-content/CONTRACT.md`: every snippet on this page must show the **port's** API, and
each demo must reproduce upstream's behaviour. Authored 2026-09-15 after the mirrored page was found
carrying upstream's React source verbatim in all five of its code blocks (gap report probe:
`{total: 5, leptos: 0, react: 5}`) — structure checks passed while the page taught the wrong
framework.

This page mirrors four teachable examples plus the hero demo. The Leptos column names what the
snippet must show; the port's real surface is `leptos_ui::checkbox` (`checkbox_root_view` in `crates/leptos-ui/src/checkbox/root.rs`, the indicator view alongside it) composed in `view!`
syntax, with state read through the crate's signals.

| example (upstream citation) | Leptos snippet to show | behavioural obligations (cited) | observable that proves it |
| --- | --- | --- | --- |
| Anatomy — assemble the parts (`docs/src/app/(docs)/react/components/checkbox/page.mdx:19-27`) | import the port's checkbox module and nest its root + indicator views in `view!` | `specs/library/checkbox/behavior.md` § Public API surface (only Root and Indicator are public parts) | the rendered tree contains the port's root element wrapping the indicator element; no non-public part is referenced |
| Hero demo (`docs/src/app/(docs)/react/components/checkbox/page.mdx:9-11`, source cited in `demos.json`) | the port's root + indicator composed the way the hero demo composes them, with the same class names the demo passes | `specs/library/checkbox/behavior.md` § State model (uncontrolled default), § Accessibility (hidden input, `aria-checked`), § DOM structure | a real click funnels through the hidden input's change event, flips `aria-checked`, the input's `checked` property and the data-* hooks, and unmounts the indicator (render_test.rs `docs-content: components/checkbox`) |
| Labeling a checkbox — wrapping label (`docs/src/app/(docs)/react/components/checkbox/page.mdx:31-41`) | the port's root inside a `<label>` in `view!`, no `htmlFor`/`id` | § Accessibility (implicit label association produces `aria-labelledby`) | clicking the wrapping label toggles the checkbox; `aria-labelledby` resolves to the label |
| Rendering as a native button — sibling label (`docs/src/app/(docs)/react/components/checkbox/page.mdx:44-56`) | the port's root with `id`, `nativeButton` and a `render`-equivalent replacing the tag with `<button>` | § DOM structure (default `span`, `nativeButton` honored), § Accessibility (sibling `htmlFor` label yields the fallback `aria-labelledby`) | the rendered element is a `<button>`; a sibling label with the same `htmlFor` toggles it |
| Render callback — wrapping label + native button (`docs/src/app/(docs)/react/components/checkbox/page.mdx:58-72`) | the port's composition rendering a `<button>` **inside** a `<label>`, keeping the hidden input outside the label | § DOM structure (native button + wrapping label must not produce invalid HTML: the input is placed outside the label) | the DOM shows the button inside the label and the hidden input outside it |
| Form integration (`docs/src/app/(docs)/react/components/checkbox/page.mdx:74-88`) | the port's root inside its `Field` port, with the label association the Field provides | `specs/library/checkbox/behavior.md` § Accessibility (Field `for`/id linking), § Events (form submission carries the value) | the field's label toggles the checkbox; the form value reflects the checked state |

Gaps carried open against this contract (do not mark the page's snippet work done over them):

* the upstream page's five code blocks include the file-selector tabs and copy chrome; those are
  `docs-chrome: code blocks`/`demo panels` scope, not snippet-language scope.
* the `render` callback row's invalid-HTML rationale is **not** asserted by
  `specs/library/checkbox/behavior.md` (already recorded in this spec's Discrepancies section) — the
  port must not claim it without a citation, per `CONTRACT.md` requirement 3.
* the playground/StackBlitz affordance upstream renders is out of scope for the port (no service
  integration); recorded here so the omission is deliberate rather than forgotten.
