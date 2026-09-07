# Radio — docs-content spec (Stage 1: docs page mining)

Source page: `docs/src/app/(docs)/react/components/radio/page.mdx` (155 lines).
Cross-checked against `specs/library/radio/behavior.md` (cited below by section name only).
Demo source files were intentionally not opened (Stage 2 scope: `specs/docs-content/radio/demos.json`).

## Page structure (headings, in order)

1. `# Radio` (h1) — `docs/src/app/(docs)/react/components/radio/page.mdx:1`
2. `## Usage guidelines` — `docs/src/app/(docs)/react/components/radio/page.mdx:13`
3. `## Anatomy` — `docs/src/app/(docs)/react/components/radio/page.mdx:17`
4. `## Examples` — `docs/src/app/(docs)/react/components/radio/page.mdx:32`
5. `### Labeling a radio group` — `docs/src/app/(docs)/react/components/radio/page.mdx:34`
6. `### Rendering as a native button` — `docs/src/app/(docs)/react/components/radio/page.mdx:54`
7. `### Form integration` — `docs/src/app/(docs)/react/components/radio/page.mdx:91`
8. `## API reference` — `docs/src/app/(docs)/react/components/radio/page.mdx:118`
9. `### RadioGroup` — `docs/src/app/(docs)/react/components/radio/page.mdx:120`
10. `### Root` — `docs/src/app/(docs)/react/components/radio/page.mdx:130`
11. `### Indicator` — `docs/src/app/(docs)/react/components/radio/page.mdx:134`

Non-heading page furniture: `<Subtitle>` under the h1 (`docs/src/app/(docs)/react/components/radio/page.mdx:3`), a `<Meta>` description block (`docs/src/app/(docs)/react/components/radio/page.mdx:4-7`), an imported hero demo rendered at the top (`docs/src/app/(docs)/react/components/radio/page.mdx:9-11`), and a trailing `export const metadata` keywords block (`docs/src/app/(docs)/react/components/radio/page.mdx:138-155`).

## Prose claims about component behavior

- The subtitle describes Radio as "An easily stylable radio button component" — `docs/src/app/(docs)/react/components/radio/page.mdx:3`
- The meta description calls it "A high-quality, unstyled React radio button component that is easy to style" — `docs/src/app/(docs)/react/components/radio/page.mdx:6`
- Usage guideline: form controls must have an accessible name, created using `<label>` elements or the `Field` and `Fieldset` components, with links to the labeling example and the forms guide — `docs/src/app/(docs)/react/components/radio/page.mdx:15`. Behavior.md has no Field/Fieldset coverage to confirm or contradict this (out of the `radio` unit's scope).
- "Radio is always placed within Radio Group" — `docs/src/app/(docs)/react/components/radio/page.mdx:19`. Behavior.md's "Edge cases" section marks outside-group behavior UNVERIFIED (tests only prove Root behavior inside a group), so the docs state a stronger rule than the tests establish; see Discrepancies.
- The anatomy snippet shows the composition `RadioGroup` > `Radio.Root` > `Radio.Indicator`, imported from `@base-ui/react/radio` and `@base-ui/react/radio-group` — `docs/src/app/(docs)/react/components/radio/page.mdx:21-30`. Consistent with behavior.md's "Public API surface" section (two entry points) and "DOM structure" section (Indicator renders inside Root).
- Label the group with `aria-labelledby` pointing at a sibling label element — `docs/src/app/(docs)/react/components/radio/page.mdx:36`. Behavior.md's "Accessibility" section covers ID linking at the per-radio level, not group-level `aria-labelledby`; not contradicted, just untested there.
- "An enclosing `<label>` is the simplest labeling pattern for each radio" — `docs/src/app/(docs)/react/components/radio/page.mdx:43`
- "By default, `<Radio.Root>` renders a `<span>` element to support enclosing labels" — `docs/src/app/(docs)/react/components/radio/page.mdx:56`. Matches behavior.md's "Public API surface" section (span default confirmed by conformance).
- "Prefer rendering each radio as a native button when using sibling labels (`htmlFor`/`id`)" — `docs/src/app/(docs)/react/components/radio/page.mdx:56`. Behavior.md's "Accessibility" section proves the nativeButton + sibling-label path works (consumer `id` lands on the visible control; clicking the label activates the radio), so usage guidance and tests agree.
- "Native buttons with wrapping labels are supported by using the `render` callback to avoid invalid HTML, so the hidden input is placed outside the label" — `docs/src/app/(docs)/react/components/radio/page.mdx:71`. Behavior.md's "DOM structure" section only proves hidden-input placement (`nextElementSibling`) in the `render={<button />}` + sibling-label configuration; the render-callback wrapping-label configuration and the invalid-HTML rationale are not test-covered. See Discrepancies.
- "Use Field and Fieldset for group labeling and form integration" — `docs/src/app/(docs)/react/components/radio/page.mdx:93`. No Field/Fieldset coverage in behavior.md to cross-check.
- RadioGroup "Provides a shared state to a series of radio buttons. Renders a `<div>` element." — `docs/src/app/(docs)/react/components/radio/page.mdx:124`. Behavior.md (mined from the `radio` unit only) treats RadioGroup as a separate entry point and never asserts its rendered element, so this claim is uncorroborated there; the "shared state" framing is consistent with behavior.md's "State model" section (checked state derived from the surrounding group).

## API tables referenced (props/parts documented on this page)

No prop tables are written inline in the .mdx. The API reference section references three generated table components; the parts documented on this page are:

- `RadioGroup` — `### RadioGroup` heading with one line of prose, then `<TypesRadioGroup />` (`docs/src/app/(docs)/react/components/radio/page.mdx:120-126`), imported from `../radio-group/types` (`docs/src/app/(docs)/react/components/radio/page.mdx:122`)
- `Radio.Root` — `### Root` heading with `<TypesRadio.Root />` (`docs/src/app/(docs)/react/components/radio/page.mdx:130-132`), via the `TypesRadio` import from `./types` (`docs/src/app/(docs)/react/components/radio/page.mdx:128`)
- `Radio.Indicator` — `### Indicator` heading with `<TypesRadio.Indicator />` (`docs/src/app/(docs)/react/components/radio/page.mdx:134-136`)

The generated tables' contents live in the types modules, which were not opened for this stage; specific prop rows are therefore not enumerated here.

## Code snippets embedded directly in the .mdx (not pulled from demos/)

Six snippets are embedded inline (the hero demo at `docs/src/app/(docs)/react/components/radio/page.mdx:9-11` is imported, not embedded):

1. `docs/src/app/(docs)/react/components/radio/page.mdx:21-30` — title "Anatomy", jsx. Imports `Radio` from `@base-ui/react/radio` and `RadioGroup` from `@base-ui/react/radio-group`; shows `<RadioGroup>` wrapping `<Radio.Root>` wrapping `<Radio.Indicator />`.
2. `docs/src/app/(docs)/react/components/radio/page.mdx:38-41` — title "Using aria-labelledby to label a radio group", tsx. A `<div id="storage-type-label">Storage type</div>` sibling plus `<RadioGroup aria-labelledby="storage-type-label">{/* ... */}</RadioGroup>`.
3. `docs/src/app/(docs)/react/components/radio/page.mdx:45-52` — title "Using an enclosing label to label a radio button", tsx. `<label>` wrapping `<Radio.Root value="ssd" />` and the text "SSD"; marked with `// @highlight` comments.
4. `docs/src/app/(docs)/react/components/radio/page.mdx:58-69` — title "Sibling label pattern with a native button", tsx. `<RadioGroup defaultValue="ssd" aria-labelledby="storage-type">` containing a `<label htmlFor="storage-type-ssd">SSD</label>` sibling and `<Radio.Root value="ssd" id="storage-type-ssd" nativeButton render={<button />}>` with a `<Radio.Indicator />` child; uses an `@highlight-text "nativeButton" "render={<button />}"` marker.
5. `docs/src/app/(docs)/react/components/radio/page.mdx:73-89` — title "Render callback", tsx. Same group setup; `<Radio.Root value="ssd" nativeButton render={(buttonProps) => (<label><button {...buttonProps} />SSD</label>)} />`; marked with `@highlight-start`/`@highlight-end`.
6. `docs/src/app/(docs)/react/components/radio/page.mdx:95-116` — title "Using Radio Group in a form", tsx. `<Form>` > `<Field.Root name="storageType">` > `<Fieldset.Root render={<RadioGroup />}>` with `<Fieldset.Legend>Storage type</Fieldset.Legend>` and two `<Field.Item>` blocks, each a `<Field.Label>` wrapping `<Radio.Root value="ssd" />` (SSD) or `<Radio.Root value="hdd" />` (HDD).

## Discrepancies (docs page vs. behavior.md)

1. "Radio is always placed within Radio Group" (`docs/src/app/(docs)/react/components/radio/page.mdx:19`) is a categorical usage rule. Behavior.md's "Edge cases" section explicitly marks whether Root/Indicator outside their required parents throw or degrade as UNVERIFIED — no test asserts it. Not a contradiction, but the docs assert more than the tests prove.
2. "Renders a `<div>` element" for RadioGroup (`docs/src/app/(docs)/react/components/radio/page.mdx:124`): behavior.md's "Public API surface" section covers only the `radio` unit and lists RadioGroup as a separate entry point never asserted by its tests, so the `<div>` claim is uncorroborated (not contradicted).
3. "the hidden input is placed outside the label" in the render-callback pattern (`docs/src/app/(docs)/react/components/radio/page.mdx:71`): behavior.md's "DOM structure" section proves hidden-input placement as `nextElementSibling` only in the `render={<button />}` + sibling-label configuration; the render-callback wrapping-label configuration is not tested, and the "avoid invalid HTML" rationale is likewise untested. Docs-only claim, flagged rather than trusted.
4. Accessible name via `Field`/`Fieldset` (`docs/src/app/(docs)/react/components/radio/page.mdx:15`, `:93`): behavior.md contains no Field/Fieldset coverage, so these claims are uncorroborated by the behavior spec.
5. Verified-consistent points (no mismatch): default `<span>` render (`docs/src/app/(docs)/react/components/radio/page.mdx:56` vs behavior.md "Public API surface"), `nativeButton` + `htmlFor`/`id` labeling (`docs/src/app/(docs)/react/components/radio/page.mdx:58-69` vs behavior.md "Accessibility"), and Indicator nested inside Root (`docs/src/app/(docs)/react/components/radio/page.mdx:21-30` vs behavior.md "DOM structure").

## Cross-links to other docs pages

- Same-page anchor `[Labeling a radio group](#labeling-a-radio-group)` — `docs/src/app/(docs)/react/components/radio/page.mdx:15`
- `[forms guide](/react/handbook/forms)` — `docs/src/app/(docs)/react/components/radio/page.mdx:15`
- `[Field](/react/components/field)` — `docs/src/app/(docs)/react/components/radio/page.mdx:93`
- `[Fieldset](/react/components/fieldset)` — `docs/src/app/(docs)/react/components/radio/page.mdx:93`
- Internal module imports (not docs-page links, listed for completeness): `./demos/hero` (`docs/src/app/(docs)/react/components/radio/page.mdx:9`), `../radio-group/types` (`docs/src/app/(docs)/react/components/radio/page.mdx:122`), `./types` (`docs/src/app/(docs)/react/components/radio/page.mdx:128`)
