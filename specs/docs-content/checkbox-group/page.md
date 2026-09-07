# checkbox-group — docs page spec

Mined from the docs page file only:

- `docs/src/app/(docs)/react/components/checkbox-group/page.mdx` (162 lines)

Cross-checked against `specs/library/checkbox-group/behavior.md` (the component's own behavior spec, mined from its tests). That spec is cited by section name; its claims are NOT re-derived here.

## Page structure (headings, in order)

- H1 `# Checkbox Group` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:1`
- `<Subtitle>` one-liner (not a heading, but part of the page header block): "Provides shared state to a series of checkboxes." — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:3`
- `<Meta name="description">` for the page: "A high-quality, unstyled React checkbox group component that provides a shared state for a series of checkboxes." — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:4-7`
- Hero demo render: imports `DemoCheckboxGroupHero` from `./demos/hero` and renders it immediately after the header block — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:9-11`
- H2 `## Usage guidelines` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:13`
- H2 `## Anatomy` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:17`
- H2 `## Examples` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:30`
  - H3 `### Labeling a checkbox group` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:32`
  - H3 `### Rendering as a native button` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:52`
  - H3 `### Form integration` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:89`
  - H3 `### Parent checkbox` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:122`
  - H3 `### Nested parent checkbox` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:136`
- H2 `## API reference` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:142`
- `export const metadata` block with SEO keywords (React Checkbox Group, Checkbox Group Component, Grouped Checkboxes, Multiple Selection, Checkbox List, Multi-Select Checkboxes, Parent Child Checkbox, Accessible Checkbox Group, Form Fieldset Control, Headless React Components, Base UI) — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:148-162`

Additional demo imports rendered inline mid-page (demo source files themselves are out of scope here — Stage 2):

- `import { DemoCheckboxGroupParent } from './demos/parent'` + render — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:132-134`
- `import { DemoCheckboxGroupNested } from './demos/nested'` + render — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:138-140`
- `import { TypesCheckboxGroup } from './types'` + render — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:144-146`

## Prose claims about component behavior

Only claims made by the page text itself, each cross-checked against the behavior spec (`specs/library/checkbox-group/behavior.md`).

- "Provides shared state to a series of checkboxes." (Subtitle) — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:3`. Consistent with the behavior spec's context-driven model ("Public API surface" section: the group works purely through context with `Checkbox.Root` children, with no subcomponents exported by the group itself — though that spec marks the no-subcomponents point UNVERIFIED).
- Meta description repeats the shared-state framing and adds "high-quality, unstyled React checkbox group component" — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:6`. Marketing framing; no behavioral claim to check beyond shared state.
- "Form controls must have an accessible name": created using `<label>` elements, or the `Field` and `Fieldset` components — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:15`. Consistent with the behavior spec's "Accessibility" section, which proves both implicit/explicit `Field.Label` association and plain `<label>` wrapping (`aria-labelledby` referencing the label's id).
- Anatomy: "Checkbox Group is composed together with [Checkbox]. Import the components and place them together" — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:19`. Consistent with the behavior spec's "Public API surface" section (children are `Checkbox.Root` items used throughout both suites).
- "Label the group with `aria-labelledby` and a sibling label element" — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:34`. Consistent with the behavior spec's "Accessibility" section: the group root receives `aria-labelledby` pointing at the label id.
- "An enclosing `<label>` is the simplest labeling pattern for each checkbox" — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:41`. Consistent with the behavior spec's "Accessibility" section: checkboxes wrapped in plain `<label>` elements each get their own accessible name via `aria-labelledby`.
- "By default, `<Checkbox.Root>` renders a `<span>` element to support enclosing labels. Prefer rendering each checkbox as a native button when using sibling labels (`htmlFor`/`id`)." — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:54`. The "renders a `<span>`" default is a `checkbox`-unit claim (not covered by the checkbox-group behavior spec — that spec only proves the DOM pairing of exposed element + hidden input in its "DOM structure & portal behavior" section and that `nativeButton` renders a `<button>`); nothing here contradicts the group spec.
- "Native buttons with wrapping labels are supported by using the `render` callback to avoid invalid HTML, so the hidden input is placed outside the label" — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:69`. The "hidden input placed outside the label" placement detail with `nativeButton` is not directly asserted by the checkbox-group behavior spec (its "DOM structure & portal behavior" section only pins hidden-input placement for the non-`nativeButton` case, and "Accessibility" says each label points at the labelable element — the button itself with `nativeButton`); not contradicted, just not proven by this unit's tests.
- "Use [Field] and [Fieldset] for group labeling and form integration" — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:91`. Consistent with the behavior spec's "Public API surface" section (Field/Fieldset/Form integration exercised across four test describes) and its "Field/Form value projection semantics" subsection.
- Parent-checkbox recipe (three numbered steps): 1. Make `<CheckboxGroup>` a controlled component; 2. Pass an array of all the child checkbox values to the `allValues` prop; 3. Add the `parent` boolean prop to the parent `<Checkbox.Root>` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:124-128`. Consistent with the behavior spec's "Public API surface" (`allValues` declares the full set of child values for parent-checkbox toggling; `parent` marks a parent/tri-state checkbox) and "State model" (controlled via `value` + `onValueChange`).
- "The group controls the parent checkbox's [indeterminate] state when some, but not all, child checkboxes are checked." — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:130`. Consistent with the behavior spec's "State model" parent tri-state (all → parent `aria-checked="true"`, some → `'mixed'`, none → `'false'`); note the docs use the word "indeterminate" while the tests verify `aria-checked="mixed"` — same concept, different terminology (see Discrepancies).

## API tables referenced (props/parts documented on this page)

- The page contains NO literal Markdown API tables. The `## API reference` section consists solely of a rendered React component: `import { TypesCheckboxGroup } from './types'` + `<TypesCheckboxGroup />` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:142-146`.
- Props/parts named in the page's prose and inline snippets (no tables, just usage): `allValues` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:127`; `parent` (boolean prop on `<Checkbox.Root>`) — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:128`; `aria-labelledby` (on the group) — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:34`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:38`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:58`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:73`; `value` (per-checkbox) — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:46`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:62`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:75`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:101`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:107`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:113`; `nativeButton` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:62`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:76`; `render` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:62`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:78-83`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:97`; `htmlFor`/`id` (sibling-label pattern) — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:54`, `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:60-62`; `name` (via `Field.Root name=...`) — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:96`.
- N/A otherwise: no props table, no parts table, no return-value table exists in the .mdx itself.

## Code snippets embedded directly in the .mdx (not pulled from demos/)

All snippets below are inline in the page file (fenced code blocks); the three `<Demo... />` renders and `<TypesCheckboxGroup />` are component imports, not snippets, and their sources are Stage 2's scope.

- **"Anatomy"** (jsx): imports `Checkbox` from `@base-ui/react/checkbox` and `CheckboxGroup` from `@base-ui/react/checkbox-group`; shows `<CheckboxGroup>` wrapping a self-closing `<Checkbox.Root />` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:21-28`.
- **"Using aria-labelledby to label a checkbox group"** (tsx): a `<div id="protocols-label">Allowed network protocols</div>` sibling + `<CheckboxGroup aria-labelledby="protocols-label">{/* ... */}</CheckboxGroup>` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:36-39`.
- **"Using an enclosing label to label a checkbox"** (tsx, `@highlight` on the `<label>` wrapper): `<label><Checkbox.Root value="http" />HTTP</label>` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:43-50`.
- **"Sibling label pattern with a native button"** (tsx): sibling label with `htmlFor="protocol-http"`, and `<Checkbox.Root id="protocol-http" value="http" nativeButton render={<button />}>` containing a `<Checkbox.Indicator />`, inside a `CheckboxGroup aria-labelledby="protocols-label"` — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:56-67`.
- **"Render callback"** (tsx): `<Checkbox.Root value="http" nativeButton render={(buttonProps) => (<label><button {...buttonProps} />HTTP</label>)} />` — native button rendered via callback so the `<label>` wraps the `<button>` element without invalid HTML — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:71-87`.
- **"Using Checkbox Group in a form"** (tsx): `<Form>` → `<Field.Root name="allowedNetworkProtocols">` → `<Fieldset.Root render={<CheckboxGroup />}>` with `<Fieldset.Legend>`, three `Field.Item`/`Field.Label` items wrapping `<Checkbox.Root value=...>` (`http`, `https`, `ssh`) — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:93-120`.

## Discrepancies (docs page vs. behavior.md)

No hard contradictions found. Terminology and scope notes:

- **"indeterminate" vs `aria-checked="mixed"`**: the page says the group "controls the parent checkbox's indeterminate state when some, but not all, child checkboxes are checked" (`docs/src/app/(docs)/react/components/checkbox-group/page.mdx:130`), while the behavior spec's "State model" section proves the mixed state as parent `aria-checked='mixed'` (`packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:58-85`). Same behavior, different words; the docs link the term to the checkbox page's `#CheckboxRoot-indeterminate` anchor. Not a factual mismatch.
- **Unproven-by-this-spec claims the page makes** (not discrepancies, but not covered by `specs/library/checkbox-group/behavior.md` either — they belong to the `checkbox` unit's spec): the default `<Checkbox.Root>` renders a `<span>` (`docs/src/app/(docs)/react/components/checkbox-group/page.mdx:54`) and the hidden-input placement rule with `nativeButton` + wrapping label, "so the hidden input is placed outside the label" (`docs/src/app/(docs)/react/components/checkbox-group/page.mdx:69`).
- **Silent omissions** (docs page says nothing; behavior spec covers it — informational only, not errors): group-level `disabled` forcing children disabled, `defaultValue`/uncontrolled mode, `onValueChange` cancellation via `eventDetails.cancel()`, empty-string as a valid value, form-submission value projection rules (disabled/parent/unmounted checkboxes excluded), and `aria-controls` on the parent checkbox — all in the behavior spec's "State model", "Events", "Field/Form value projection semantics", and "Accessibility" sections; none are claimed on the page, so nothing to contradict.

## Cross-links to other docs pages

Markdown links in the page body (excluding the three `./demos/*` and `./types` relative imports listed under Page structure):

- `[forms guide](/react/handbook/forms)` — from the accessible-name guideline — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:15`
- `[Labeling a checkbox group](#labeling-a-checkbox-group)` — same-page anchor link (not another page, listed for completeness) — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:15`
- `[Checkbox](/react/components/checkbox)` — Anatomy intro — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:19`
- `[Field](/react/components/field)` — Form integration intro — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:91`
- `[Fieldset](/react/components/fieldset)` — Form integration intro — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:91`
- `[indeterminate](/react/components/checkbox#CheckboxRoot-indeterminate)` — Parent checkbox section — `docs/src/app/(docs)/react/components/checkbox-group/page.mdx:130`
