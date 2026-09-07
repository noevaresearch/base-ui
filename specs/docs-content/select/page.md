# Select docs page content spec

Mined from `docs/src/app/(docs)/react/components/select/page.mdx` only. The component's own
behavior is covered by `specs/library/select/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/select/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/select/demos.json`).

## Page structure (headings, in order)

- `# Select` (h1) — `docs/src/app/(docs)/react/components/select/page.mdx:1`
- `## Usage guidelines` — `docs/src/app/(docs)/react/components/select/page.mdx:13`
- `## Anatomy` — `docs/src/app/(docs)/react/components/select/page.mdx:19`
- `## Positioning` — `docs/src/app/(docs)/react/components/select/page.mdx:56`
- `## Examples` — `docs/src/app/(docs)/react/components/select/page.mdx:72`
  - `### Typed wrapper component` — `docs/src/app/(docs)/react/components/select/page.mdx:74`
  - `### Formatting the value` — `docs/src/app/(docs)/react/components/select/page.mdx:89`
  - `### Labeling a select` — `docs/src/app/(docs)/react/components/select/page.mdx:132`
  - `### Placeholder values` — `docs/src/app/(docs)/react/components/select/page.mdx:146`
  - `### Multiple selection` — `docs/src/app/(docs)/react/components/select/page.mdx:179`
  - `### Object values` — `docs/src/app/(docs)/react/components/select/page.mdx:187`
  - `### Grouped` — `docs/src/app/(docs)/react/components/select/page.mdx:196`
- `## API reference` — `docs/src/app/(docs)/react/components/select/page.mdx:227`
  - `### Root` — `docs/src/app/(docs)/react/components/select/page.mdx:231`
  - `### Label` — `docs/src/app/(docs)/react/components/select/page.mdx:235`
  - `### Trigger` — `docs/src/app/(docs)/react/components/select/page.mdx:239`
  - `### Value` — `docs/src/app/(docs)/react/components/select/page.mdx:243`
  - `### Icon` — `docs/src/app/(docs)/react/components/select/page.mdx:247`
  - `### Backdrop` — `docs/src/app/(docs)/react/components/select/page.mdx:251`
  - `### Portal` — `docs/src/app/(docs)/react/components/select/page.mdx:255`
  - `### Positioner` — `docs/src/app/(docs)/react/components/select/page.mdx:259`
  - `### Popup` — `docs/src/app/(docs)/react/components/select/page.mdx:263`
  - `### List` — `docs/src/app/(docs)/react/components/select/page.mdx:267`
  - `### Arrow` — `docs/src/app/(docs)/react/components/select/page.mdx:271`
  - `### Item` — `docs/src/app/(docs)/react/components/select/page.mdx:275`
  - `### ItemText` — `docs/src/app/(docs)/react/components/select/page.mdx:279`
  - `### ItemIndicator` — `docs/src/app/(docs)/react/components/select/page.mdx:283`
  - `### Group` — `docs/src/app/(docs)/react/components/select/page.mdx:287`
  - `### GroupLabel` — `docs/src/app/(docs)/react/components/select/page.mdx:291`
  - `### ScrollUpArrow` — `docs/src/app/(docs)/react/components/select/page.mdx:295`
  - `### ScrollDownArrow` — `docs/src/app/(docs)/react/components/select/page.mdx:299`
  - `### Separator` — `docs/src/app/(docs)/react/components/select/page.mdx:303`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A common form component for choosing a predefined value in a dropdown menu." —
  `docs/src/app/(docs)/react/components/select/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React select component for choosing a
  predefined value in a dropdown menu." —
  `docs/src/app/(docs)/react/components/select/page.mdx:4-7`
- Hero demo import and render (`./demos/hero`) before the first heading —
  `docs/src/app/(docs)/react/components/select/page.mdx:9-11`
- Demo imports interleaved with the Examples subsections (`./demos/multiple` at
  `docs/src/app/(docs)/react/components/select/page.mdx:183`, `./demos/object-values` at
  `docs/src/app/(docs)/react/components/select/page.mdx:192`, `./demos/grouped` at
  `docs/src/app/(docs)/react/components/select/page.mdx:223`)
- `TypesSelect` import for the API reference —
  `docs/src/app/(docs)/react/components/select/page.mdx:229`
- Trailing `export const metadata` SEO keywords block (20 keywords, e.g. 'React Select Component',
  'Dropdown Select', 'Accessible Select') —
  `docs/src/app/(docs)/react/components/select/page.mdx:307-330`

## Prose claims about component behavior

- Page describes the component as "A common form component for choosing a predefined value in a
  dropdown menu." and, in the meta description, as "unstyled". Naming-level claim only; consistent
  with the compound-part API in behavior.md "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/select/page.mdx:3`, `docs/src/app/(docs)/react/components/select/page.mdx:4-7`
- Usage guideline: "Select is not filterable, aside from basic keyboard typeahead functionality to
  find items by focusing and highlighting them. Prefer Combobox instead of Select when the number
  of items is sufficiently large to warrant filtering." The no-filtering claim is consistent with
  behavior.md (no filtering behavior appears in any recorded section); behavior.md "Keyboard
  interactions" however characterizes typeahead differently — closed-trigger typeahead commits the
  matched value — while the page frames typeahead as find-by-focus/highlight. See Discrepancies.
  `docs/src/app/(docs)/react/components/select/page.mdx:15`
- Usage guideline: "The select popup by default overlaps its trigger so the selected item's text is
  aligned with the trigger's value text. This behavior can be disabled or customized." Matches
  behavior.md "DOM structure & portal behavior" (item-aligned mode is the default and vertically
  centers the first selected item with the trigger).
  `docs/src/app/(docs)/react/components/select/page.mdx:16`
- Usage guideline: "Form controls must have an accessible name. Prefer `<Select.Label>`, or provide
  an `aria-label` on `<Select.Trigger>` when no visible label is rendered." Consistent with
  behavior.md "Accessibility (roles, aria-*, id linking)" (Label sets the trigger's
  `aria-labelledby`); the page adds the `aria-label` fallback, which behavior.md does not record.
  `docs/src/app/(docs)/react/components/select/page.mdx:17`
- Anatomy usage guidance: "Import the component and assemble its parts", showing the assembly
  `Select.Root` > (`Select.Label`, `Select.Trigger` containing `Select.Value` and `Select.Icon`) >
  `Select.Portal` > `Select.Backdrop` > `Select.Positioner` > `Select.Popup` > (`Select.ScrollUpArrow`,
  `Select.Arrow`, `Select.List` containing `Select.Item` (`Select.ItemText`, `Select.ItemIndicator`),
  `Select.Separator`, `Select.Group` > `Select.GroupLabel`, `Select.ScrollDownArrow`), imported from
  the `@base-ui/react/select` namespace. The namespace import matches behavior.md "Public API surface
  (props, parts, subcomponents)", and the Backdrop-before-Positioner sibling order matches
  behavior.md "DOM structure & portal behavior" (the backdrop is the positioner's previous sibling);
  but the anatomy includes `Select.Separator`, a part behavior.md's 18-part enumeration does not
  list. See Discrepancies.
  `docs/src/app/(docs)/react/components/select/page.mdx:23-54`
- Positioning: "`<Select.Positioner>` has a special prop called `alignItemWithTrigger` which causes
  the positioning to act differently by default from other `Positioner` components. The prop makes
  the select popup overlap the trigger so the selected item's text is aligned with the trigger's
  value text." Matches behavior.md "DOM structure & portal behavior" (default item-aligned mode).
  `docs/src/app/(docs)/react/components/select/page.mdx:58-59`
- Positioning: "For styling, `data-side` is `"none"` on the `.Popup` and `.Positioner` parts when
  the mode is active." Matches behavior.md "DOM structure & portal behavior" (`data-side="none"` is
  exposed); behavior.md does not attribute the attribute to specific parts.
  `docs/src/app/(docs)/react/components/select/page.mdx:61`
- Positioning: "To prevent the select popup from overlapping its trigger, set the
  `alignItemWithTrigger` prop to `false`", and the default is `true`. Matches behavior.md
  "DOM structure & portal behavior" (aligned mode is the default; non-aligned mode uses transform
  positioning).
  `docs/src/app/(docs)/react/components/select/page.mdx:63-64`
- Positioning caveat: "Interaction type dependent: the `alignItemWithTrigger` positioning mode is
  disabled if touch was the pointer type used to open the popup." Not covered by behavior.md (its
  touch coverage is about interaction-type preservation across rapid reopen cycles, in
  "Edge cases (rapid interactions, unmount, nesting)"); see Discrepancies.
  `docs/src/app/(docs)/react/components/select/page.mdx:66`
- Positioning caveat: "Viewport space dependent: there must be enough space in the viewport ...
  otherwise, it falls back to the default positioning mode. This can be customized by setting
  `min-height` on the `<Select.Positioner>` element; a smaller value will fallback less often."
  Not covered by behavior.md ("Edge cases" records `max-height` clamps, not a viewport-space
  alignment fallback or a `min-height` tuning knob); see Discrepancies.
  `docs/src/app/(docs)/react/components/select/page.mdx:67-68`
- Positioning caveat: "the trigger must be at least 20px from the edges of the top and bottom of
  the viewport, or it will also fall back." Not covered by behavior.md; see Discrepancies.
  `docs/src/app/(docs)/react/components/select/page.mdx:69`
- Positioning caveat: "Other positioning props are ignored: props like `side` or `align` have no
  effect unless the prop is set to `false` or when in fallback mode." Not explicitly recorded in
  behavior.md (it distinguishes aligned vs non-aligned modes but never states that `side`/`align`
  are inert in aligned mode); see Discrepancies.
  `docs/src/app/(docs)/react/components/select/page.mdx:70`
- Typed wrapper example prose: "The following example shows a typed wrapper around the Select
  component with correct type inference and type safety", using
  `Select.Root.Props<Value, Multiple>` with `Multiple extends boolean | undefined = false`.
  behavior.md records the Root prop list but not this generic parameterization; see Discrepancies.
  `docs/src/app/(docs)/react/components/select/page.mdx:76`, `docs/src/app/(docs)/react/components/select/page.mdx:82-85`
- "By default, the `<Select.Value>` component renders the raw `value`." behavior.md "Public API
  surface (props, parts, subcomponents)" records Value's children/placeholder surface but not this
  default rendering; see Discrepancies.
  `docs/src/app/(docs)/react/components/select/page.mdx:91`
- "Passing the `items` prop to `<Select.Root>` instead renders the matching label for the rendered
  value." Consistent with behavior.md "Public API surface (props, parts, subcomponents)" (`items`
  accepts an object map or `{ value, label }` array with ReactNode labels).
  `docs/src/app/(docs)/react/components/select/page.mdx:93`
- "A function can also be passed as the `children` prop of `<Select.Value>` to render a formatted
  value." Matches behavior.md "Public API surface (props, parts, subcomponents)" (Value accepts a
  render-function child).
  `docs/src/app/(docs)/react/components/select/page.mdx:110`
- "To avoid lookup, object values for each item can also be used." Cross-reference into the page's
  own Object values section.
  `docs/src/app/(docs)/react/components/select/page.mdx:130`
- "Use `<Select.Label>` to provide a visible label for the select trigger" and "`<Select.Label>`
  renders a `<div>`, so clicking it focuses the select trigger without opening the popup." Matches
  behavior.md "Focus management" (label integration focuses the trigger without opening) and
  "Public API surface (props, parts, subcomponents)" (Label is a div).
  `docs/src/app/(docs)/react/components/select/page.mdx:134`, `docs/src/app/(docs)/react/components/select/page.mdx:144`
- "To show a placeholder value, use the `placeholder` prop on `<Select.Value>`." Matches behavior.md
  "Public API surface (props, parts, subcomponents)" (Value accepts `placeholder`).
  `docs/src/app/(docs)/react/components/select/page.mdx:148`
- "With placeholders, users cannot clear selected values using the select itself. If the select
  value should be clearable from the popup (instead of an external "reset" button), use a `null`
  item rendered in the list itself." Not covered by behavior.md (no clearability semantics are
  recorded); see Discrepancies.
  `docs/src/app/(docs)/react/components/select/page.mdx:163`
- "Add the `multiple` prop to the `<Select.Root>` component to allow multiple selections." Matches
  behavior.md "Public API surface (props, parts, subcomponents)" and "State model
  (controlled/uncontrolled, defaults, transitions)" (array values, popup stays open in multiple
  mode).
  `docs/src/app/(docs)/react/components/select/page.mdx:181`
- "Select items can use objects as values instead of primitives. This lets you access the full
  object in custom render functions, and can avoid needing to specify `items` for lookup."
  Consistent with behavior.md "Public API surface (props, parts, subcomponents)"
  (`itemToStringLabel`/`itemToStringValue` handle object-valued items).
  `docs/src/app/(docs)/react/components/select/page.mdx:189-190`
- "Organize related options with `<Select.Group>` and `<Select.GroupLabel>` to add section headings
  inside the popup." Consistent with behavior.md "Accessibility (roles, aria-*, id linking)"
  (group/group-labelledby registration).
  `docs/src/app/(docs)/react/components/select/page.mdx:198`
- "Groups are represented by an array of objects with an `items` property, which itself is an array
  of individual items for each group. An extra property, such as `value`, can be provided for the
  heading text when rendering the group label." Not covered by behavior.md (its `items` coverage
  covers flat maps/arrays and object-value mapping hooks, not the grouped shape); see
  Discrepancies.
  `docs/src/app/(docs)/react/components/select/page.mdx:200`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents nineteen parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesSelect.Root />`, `### Label` →
  `<TypesSelect.Label />`, `### Trigger` → `<TypesSelect.Trigger />`, `### Value` →
  `<TypesSelect.Value />`, `### Icon` → `<TypesSelect.Icon />`, `### Backdrop` →
  `<TypesSelect.Backdrop />`, `### Portal` → `<TypesSelect.Portal />`, `### Positioner` →
  `<TypesSelect.Positioner />`, `### Popup` → `<TypesSelect.Popup />`, `### List` →
  `<TypesSelect.List />`, `### Arrow` → `<TypesSelect.Arrow />`, `### Item` →
  `<TypesSelect.Item />`, `### ItemText` → `<TypesSelect.ItemText />`, `### ItemIndicator` →
  `<TypesSelect.ItemIndicator />`, `### Group` → `<TypesSelect.Group />`, `### GroupLabel` →
  `<TypesSelect.GroupLabel />`, `### ScrollUpArrow` → `<TypesSelect.ScrollUpArrow />`,
  `### ScrollDownArrow` → `<TypesSelect.ScrollDownArrow />`, `### Separator` →
  `<TypesSelect.Separator />`.
  `docs/src/app/(docs)/react/components/select/page.mdx:231-305`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesSelect } from './types';`); no props, prop types, defaults, or prop descriptions
  are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/select/page.mdx:229`
- Parts documented on this page: Root, Label, Trigger, Value, Icon, Backdrop, Portal, Positioner,
  Popup, List, Arrow, Item, ItemText, ItemIndicator, Group, GroupLabel, ScrollUpArrow,
  ScrollDownArrow, Separator — the same set as behavior.md "Public API surface (props, parts,
  subcomponents)" plus `Separator`, which that spec's 18-part enumeration omits (see
  Discrepancies).
  `docs/src/app/(docs)/react/components/select/page.mdx:231-305`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- Eight fenced code blocks:
  1. Anatomy snippet (` ```jsx title="Anatomy" `): namespace import from `@base-ui/react/select`
     and the full part assembly described under Prose claims above.
     `docs/src/app/(docs)/react/components/select/page.mdx:23-54`
  2. Typed wrapper (` ```tsx title="Specifying generic type parameters" `): a `MySelect<Value,
     Multiple extends boolean | undefined = false>` wrapper forwarding
     `Select.Root.Props<Value, Multiple>`.
     `docs/src/app/(docs)/react/components/select/page.mdx:78-87`
  3. `items` prop (` ```jsx title="items prop" `): an array of `{ value, label }` objects whose
     first entry is `{ value: null, label: 'Select theme' }`, passed to `<Select.Root items={...}>`.
     `docs/src/app/(docs)/react/components/select/page.mdx:95-108`
  4. Function child of Value (` ```jsx title="Lookup map" `): an object map passed to a
     `(value: keyof typeof items) => <span style={{ fontFamily: value }}>{items[value]}</span>`
     render function inside `<Select.Value>`.
     `docs/src/app/(docs)/react/components/select/page.mdx:112-128`
  5. Label snippet (` ```tsx title="Using Select.Label to label a select" `):
     `<Select.Label>Theme</Select.Label>` inside `<Select.Root>`.
     `docs/src/app/(docs)/react/components/select/page.mdx:136-142`
  6. Placeholder snippet (` ```jsx title="Placeholder item" `): `<Select.Value
     placeholder="Select theme" />` with an `items` array containing no `null` entry.
     `docs/src/app/(docs)/react/components/select/page.mdx:150-161`
  7. Clearable snippet (` ```jsx title="Clearable item" `): the same `items` array with a
     `{ value: null, label: 'Select theme' }` entry highlighted, rendered through a bare
     `<Select.Value />`.
     `docs/src/app/(docs)/react/components/select/page.mdx:165-177`
  8. Grouped data shape (` ```tsx title="Example" `): a `ProduceGroupItem` interface with `value:
     string` and `items: string[]`, plus a `groups` array of two groups ('Fruits', 'Vegetables').
     `docs/src/app/(docs)/react/components/select/page.mdx:202-221`
- The snippets carry editor-highlight directives (`@highlight`, `@highlight-text`,
  `@highlight-start`/`@highlight-end`) marking the instructive lines.
  `docs/src/app/(docs)/react/components/select/page.mdx:96`, `docs/src/app/(docs)/react/components/select/page.mdx:138`, `docs/src/app/(docs)/react/components/select/page.mdx:158`
- The hero, Multiple selection, Object values, and Grouped example bodies render demo components
  (`<DemoSelectHero />`, `<DemoSelectMultiple />`, `<DemoSelectObjectValues />`,
  `<DemoSelectGrouped />`) imported from `./demos/*`; their code lives in demo files and is out of
  scope here (Stage 2).
  `docs/src/app/(docs)/react/components/select/page.mdx:9-11`, `docs/src/app/(docs)/react/components/select/page.mdx:183-185`, `docs/src/app/(docs)/react/components/select/page.mdx:192-194`, `docs/src/app/(docs)/react/components/select/page.mdx:223-225`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/select/behavior.md`.
Items flagged for lack of coverage or omission:

- Part-set mismatch: the page's Anatomy includes `Select.Separator` and its API reference documents
  a `### Separator` section (19 parts total); behavior.md "Public API surface (props, parts,
  subcomponents)" enumerates 18 parts and Separator appears in none of its part paragraphs. The
  behavior spec under-covers a part the docs treat as first-class.
  `docs/src/app/(docs)/react/components/select/page.mdx:44`, `docs/src/app/(docs)/react/components/select/page.mdx:303-305`
- Docs-only claim, not covered by behavior.md: the item-aligned positioning mode is disabled when
  touch was the pointer type used to open the popup. behavior.md's touch coverage ("Edge cases
  (rapid interactions, unmount, nesting)") only records interaction-type preservation across rapid
  open/close cycles.
  `docs/src/app/(docs)/react/components/select/page.mdx:66`
- Docs-only claim, not covered by behavior.md: viewport-space fallback of the aligned mode —
  insufficient vertical space falls back to default positioning, tunable via `min-height` on the
  Positioner ("a smaller value will fallback less often"), plus the concrete 20px trigger-to-viewport-edge
  threshold. behavior.md records fractional-geometry clamps but no alignment-fallback heuristic or
  threshold.
  `docs/src/app/(docs)/react/components/select/page.mdx:67-69`
- Docs-only claim, not covered by behavior.md: `side`/`align` (and similar positioning props) have
  no effect while `alignItemWithTrigger` is active unless set to `false` or in fallback mode.
  behavior.md "DOM structure & portal behavior" describes the two positioning modes but never
  states the other props are ignored in aligned mode.
  `docs/src/app/(docs)/react/components/select/page.mdx:70`
- Docs-only claim, not covered by behavior.md: with a `placeholder`, users cannot clear selected
  values using the select itself, and a `null` item in the list is the pattern for popup-side
  clearing. behavior.md records the `placeholder` prop's existence but no clearability semantics.
  `docs/src/app/(docs)/react/components/select/page.mdx:163`
- Docs-only claim, not covered by behavior.md: `<Select.Value>` renders the raw `value` by default,
  and the `items` prop swaps in the matching label. behavior.md "Public API surface (props, parts,
  subcomponents)" records Value's children/placeholder surface and the `items` input shape, but not
  the default raw rendering or the label-lookup rendering outcome.
  `docs/src/app/(docs)/react/components/select/page.mdx:91`, `docs/src/app/(docs)/react/components/select/page.mdx:93`
- Docs-only claim, not covered by behavior.md: the grouped `items` data shape (array of objects
  with an `items` property plus an extra property such as `value` for the group heading text).
  behavior.md's `items` coverage covers flat maps/arrays and object-value mapping hooks only.
  `docs/src/app/(docs)/react/components/select/page.mdx:200`
- Docs-only API-surface claim, not covered by behavior.md: the generic parameterization
  `Select.Root.Props<Value, Multiple>` for building typed wrappers. behavior.md records the Root
  prop list but not generic type parameters.
  `docs/src/app/(docs)/react/components/select/page.mdx:82-85`
- Characterization nuance (not a contradiction): the page says keyboard typeahead finds items "by
  focusing and highlighting them" (page.mdx:15), while behavior.md "Keyboard interactions" records
  closed-trigger typeahead committing the matched value; behavior.md does not record typeahead
  focus/highlight semantics inside the popup. `docs/src/app/(docs)/react/components/select/page.mdx:15`
- Omission (docs page vs. behavior.md): the Positioning section describes `alignItemWithTrigger`
  behavior without mentioning behavior.md "Focus management" item — when an item-aligned popup is
  open, focus landing on the trigger closes it with `onOpenChange(false, { reason: 'none' })` —
  a user-visible consequence of the default mode the docs page does not call out. Omission of a
  caveat, not a contradiction. `docs/src/app/(docs)/react/components/select/page.mdx:56-70`

## Cross-links to other docs pages

- Links to the Combobox component page (`/react/components/combobox`) in the "Prefer Combobox for
  large lists" guideline. `docs/src/app/(docs)/react/components/select/page.mdx:15`
- Links to the forms handbook (`/react/handbook/forms`) in the accessible-name guideline.
  `docs/src/app/(docs)/react/components/select/page.mdx:17`
- Same-page anchors (not cross-page links): `/react/components/select#positioning` (page.mdx:16),
  `#labeling-a-select` (page.mdx:17), and `#object-values` (page.mdx:130).
  `docs/src/app/(docs)/react/components/select/page.mdx:16`, `docs/src/app/(docs)/react/components/select/page.mdx:17`, `docs/src/app/(docs)/react/components/select/page.mdx:130`
- Demo components (`./demos/hero`, `./demos/multiple`, `./demos/object-values`, `./demos/grouped`)
  are imported and rendered on this page itself; they are same-page imports, not cross-page links.
  `docs/src/app/(docs)/react/components/select/page.mdx:9`, `docs/src/app/(docs)/react/components/select/page.mdx:183`, `docs/src/app/(docs)/react/components/select/page.mdx:192`, `docs/src/app/(docs)/react/components/select/page.mdx:223`
