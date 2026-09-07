# Combobox docs page content spec

Mined from `docs/src/app/(docs)/react/components/combobox/page.mdx` only. The component's own
behavior is covered by `specs/library/combobox/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/combobox/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/combobox/demos.json`).

## Page structure (headings, in order)

- `# Combobox` (h1) — `docs/src/app/(docs)/react/components/combobox/page.mdx:1`
- `## Usage guidelines` — `docs/src/app/(docs)/react/components/combobox/page.mdx:13`
- `## Anatomy` — `docs/src/app/(docs)/react/components/combobox/page.mdx:20`
- `## Item values` — `docs/src/app/(docs)/react/components/combobox/page.mdx:74`
- `## Examples` — `docs/src/app/(docs)/react/components/combobox/page.mdx:90`
  - `### Typed wrapper component` — `docs/src/app/(docs)/react/components/combobox/page.mdx:92`
  - `### Value selection with IDs` — `docs/src/app/(docs)/react/components/combobox/page.mdx:109`
  - `### Multiple select` — `docs/src/app/(docs)/react/components/combobox/page.mdx:177`
    - `#### Keeping the filter after selection` — `docs/src/app/(docs)/react/components/combobox/page.mdx:224`
  - `### Input inside popup` — `docs/src/app/(docs)/react/components/combobox/page.mdx:258`
  - `### Grouped` — `docs/src/app/(docs)/react/components/combobox/page.mdx:278`
  - `### Async search (single)` — `docs/src/app/(docs)/react/components/combobox/page.mdx:309`
  - `### Async search (multiple)` — `docs/src/app/(docs)/react/components/combobox/page.mdx:317`
  - `### Creatable` — `docs/src/app/(docs)/react/components/combobox/page.mdx:325`
  - `### Virtualized` — `docs/src/app/(docs)/react/components/combobox/page.mdx:333`
    - `#### Memoizing items` — `docs/src/app/(docs)/react/components/combobox/page.mdx:341`
- `## API reference` — `docs/src/app/(docs)/react/components/combobox/page.mdx:363`
  - `### Root` — `docs/src/app/(docs)/react/components/combobox/page.mdx:367`
  - `### Label` — `docs/src/app/(docs)/react/components/combobox/page.mdx:371`
  - `### Value` — `docs/src/app/(docs)/react/components/combobox/page.mdx:375`
  - `### Icon` — `docs/src/app/(docs)/react/components/combobox/page.mdx:379`
  - `### Input` — `docs/src/app/(docs)/react/components/combobox/page.mdx:383`
  - `### InputGroup` — `docs/src/app/(docs)/react/components/combobox/page.mdx:387`
  - `### Clear` — `docs/src/app/(docs)/react/components/combobox/page.mdx:391`
  - `### Trigger` — `docs/src/app/(docs)/react/components/combobox/page.mdx:395`
  - `### Chips` — `docs/src/app/(docs)/react/components/combobox/page.mdx:399`
  - `### Chip` — `docs/src/app/(docs)/react/components/combobox/page.mdx:403`
  - `### ChipRemove` — `docs/src/app/(docs)/react/components/combobox/page.mdx:407`
  - `### List` — `docs/src/app/(docs)/react/components/combobox/page.mdx:411`
  - `### Portal` — `docs/src/app/(docs)/react/components/combobox/page.mdx:415`
  - `### Backdrop` — `docs/src/app/(docs)/react/components/combobox/page.mdx:419`
  - `### Positioner` — `docs/src/app/(docs)/react/components/combobox/page.mdx:423`
  - `### Popup` — `docs/src/app/(docs)/react/components/combobox/page.mdx:427`
  - `### Arrow` — `docs/src/app/(docs)/react/components/combobox/page.mdx:431`
  - `### Status` — `docs/src/app/(docs)/react/components/combobox/page.mdx:435`
  - `### Empty` — `docs/src/app/(docs)/react/components/combobox/page.mdx:439`
  - `### Collection` — `docs/src/app/(docs)/react/components/combobox/page.mdx:443`
  - `### Row` — `docs/src/app/(docs)/react/components/combobox/page.mdx:447`
  - `### Item` — `docs/src/app/(docs)/react/components/combobox/page.mdx:451`
  - `### ItemIndicator` — `docs/src/app/(docs)/react/components/combobox/page.mdx:455`
  - `### Group` — `docs/src/app/(docs)/react/components/combobox/page.mdx:459`
  - `### GroupLabel` — `docs/src/app/(docs)/react/components/combobox/page.mdx:463`
  - `### Separator` — `docs/src/app/(docs)/react/components/combobox/page.mdx:467`
- `## useFilter` (hook reference, not under API reference) — `docs/src/app/(docs)/react/components/combobox/page.mdx:471`
- `## useFilteredItems` (hook reference) — `docs/src/app/(docs)/react/components/combobox/page.mdx:479`
- `## createItems` (util reference) — `docs/src/app/(docs)/react/components/combobox/page.mdx:485`

Non-heading page furniture, in document order:

- `<Subtitle>` — "An input combined with a list of predefined items to select." — `docs/src/app/(docs)/react/components/combobox/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React combobox component that renders an input combined with a list of predefined items to select." — `docs/src/app/(docs)/react/components/combobox/page.mdx:4-7`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/combobox/page.mdx:9-11`
- Demo imports interleaved with the Examples subsections (`./demos/create-items` at `docs/src/app/(docs)/react/components/combobox/page.mdx:147-149`, `./demos/multiple` at `docs/src/app/(docs)/react/components/combobox/page.mdx:182-184`, `./demos/input-inside-popup` at `docs/src/app/(docs)/react/components/combobox/page.mdx:262-264`, `./demos/grouped` at `docs/src/app/(docs)/react/components/combobox/page.mdx:305-307`, `./demos/async-single` at `docs/src/app/(docs)/react/components/combobox/page.mdx:313-315`, `./demos/async-multiple` at `docs/src/app/(docs)/react/components/combobox/page.mdx:321-323`, `./demos/creatable` at `docs/src/app/(docs)/react/components/combobox/page.mdx:329-331`, `./demos/virtualized` at `docs/src/app/(docs)/react/components/combobox/page.mdx:337-339`)
- `TypesCombobox` import for the API reference — `docs/src/app/(docs)/react/components/combobox/page.mdx:365`
- Trailing `export const metadata` SEO keywords block (16 keywords, e.g. 'React Combobox', 'Filterable Select', 'Tags Input', 'Async Combobox') — `docs/src/app/(docs)/react/components/combobox/page.mdx:491-510`

## Prose claims about component behavior

- Positioning guidance: "Combobox is a filterable Select" — use it when the input is restricted
  to predefined selectable items whose list is filterable via an input, and "Prefer using Combobox
  over Select when the number of items is sufficiently large to warrant filtering." Usage guidance;
  consistent with behavior.md "Public API surface (props, parts, subcomponents)" (`filter`,
  `filteredItems`, `items` props) and the "Derived-value pipeline" cross-cutting item (locale-aware
  collator filter).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:15`
- "Combobox does not allow free-form text input. For search widgets, consider using
  [Autocomplete](/react/components/autocomplete) instead." Consistent with behavior.md "State
  model (controlled/uncontrolled, defaults, transitions)": inputValue is "a synchronized mirror of
  the resolved selection" and Escape discards an uncommitted query, so typed text never becomes a
  free-form value.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:16`
- "Avoid when not rendering an input": use [Select](/react/components/select) instead of Combobox
  if no input is being rendered, "which includes accessibility features specific to a listbox
  without an input." Docs-only routing guidance; behavior.md covers Combobox a11y with an input
  present and does not address input-less listboxes.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:17`
- Accessible-name rule: "Form controls must have an accessible name" — label `<Combobox.Input>`
  with a native `<label>`, `<Field.Label>`, or `aria-label`; `<Combobox.Label>` labels
  `<Combobox.Trigger>` and is intended for the input-inside-popup pattern "where the trigger is
  the form control." The Label-labels-Trigger half matches behavior.md "Accessibility (roles,
  aria-*, id linking)" (Combobox.Label labels the Trigger only, dev warning otherwise); the
  trigger-as-form-control framing matches behavior.md's two-sided form-value ownership invariant
  and the input-inside-popup composition in "Whole-unit cross-cutting behavior". The native
  label/Field.Label/aria-label authoring guidance is docs-only.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:18`
- "Each `<Combobox.Item>` takes a `value` prop identifying it. Pass the item being rendered", and
  "That item is what `value`, `defaultValue`, and `onValueChange` receive." Consistent with
  behavior.md "State model" (single mode holds a primitive or object) and the "Value lifecycle
  converges on `onValueChange`" cross-cutting item; behavior.md does not itself assert the
  pass-the-rendered-item default convention.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:76`, `docs/src/app/(docs)/react/components/combobox/page.mdx:88`
- Generic typing: a wrapper's third `Item` type parameter "is what lets the wrapper accept a
  `Combobox.createItems()` collection, whose rendered item type differs from its selection value.
  Omit it and a collection infers `Value` as the source item type, surfacing an error on
  `defaultValue` rather than on `items`." Typing-only claim; consistent with behavior.md's
  createItems description in "State model" (derives selection value distinct from the item).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:107`
- `Combobox.createItems` with `getValue`/`getLabel` derives "each item's selection value and
  display label"; with static data create the collection at module scope and "Pass the derived ID
  to the `value` prop of `<Combobox.Item>`, not the item being rendered." Matches behavior.md
  "State model" and "Derived-value pipeline" (`createItems({getValue, getLabel})` accessors).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:111-113`
- "Selection props and events use the derived IDs, while list rendering continues to receive the
  original items. The derived label is also used for filtering and typeahead." Consistent with
  behavior.md "Derived-value pipeline" (labels resolve live from collection data; collator filter;
  trigger typeahead).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:132`
- "When the data is loaded or replaced at runtime, memoize the collection on it so that it is
  rebuilt only when the data changes." Usage guidance; adjacent to behavior.md "Edge cases" (items
  identity changes preserve the typed filter) but the memoization advice itself is docs-only.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:134`
- Rule of thumb: "use primitive items for simple lists, object values when the selected record is
  useful application state, and `createItems()` when selection should use a stable primitive ID
  while rendering and filtering still use object records." Docs-only selection-model guidance.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:151`
- "Stable IDs preserve item matching when an async data library replaces row objects during a
  refetch. With object values, use `isItemEqualToValue` to compare their IDs instead." Consistent
  with behavior.md "State model" (labels resolve live; async items restore derived labels) and the
  `isItemEqualToValue` prop in "Public API surface".
  `docs/src/app/(docs)/react/components/combobox/page.mdx:153`
- External filtering: "`items` represents records known to the app, while `filteredItems`
  represents the current result window displayed in the popup. Pass source items rather than
  derived values to `filteredItems`, preserving the flat or grouped structure of `items`."
  Consistent with the `filteredItems` prop in behavior.md "Public API surface" and the
  "custom filter/filteredItems windows" step of "Derived-value pipeline"; the source-items and
  flat-or-grouped-structure requirement is docs-only (see Discrepancies).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:155`
- "Include the selected user's record in `knownUsers` so its label remains available when it is
  outside `searchResults`. Alternatively, provide `itemToStringLabel` when the label can be derived
  from the selected value alone." Consistent with behavior.md "State model" (per-value degradation
  to the raw value) and the `itemToStringLabel` prop in "Public API surface".
  `docs/src/app/(docs)/react/components/combobox/page.mdx:175`
- "The combobox can allow multiple selections by adding the `multiple` prop to `<Combobox.Root>`"
  and "Selection chips are rendered with `<Combobox.Chip>` inside the input that can be removed."
  Matches behavior.md "State model" (multiple mode holds an array where `[]` means no value) and
  the value/chips parts in "Public API surface".
  `docs/src/app/(docs)/react/components/combobox/page.mdx:179-180`
- Screen-reader chip guidance: add `aria-description` to `<Combobox.Chip>` and `<Combobox.Input>`,
  plus `aria-label` on `<Combobox.Chips>` and `<Combobox.ChipRemove>`; "Base UI does not ship
  these strings. Translate them together with the rest of your interface." Docs-only authoring
  guidance; behavior.md "Accessibility" and "Keyboard interactions" cover chip roles and removal
  keys but not these attributes (see Discrepancies).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:186-187`
- "Visible chips can be limited by slicing the selected values rendered in `<Combobox.Value>`."
  Docs-only pattern guidance; consistent with Value/Chip rendering in behavior.md "DOM structure &
  portal behavior" (Chips render inline outside the portal).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:189`
- Filter-after-selection behavior: "In `multiple` mode, selecting an item clears the typed filter
  and closes the popup when the input is rendered outside it"; to keep the query, "cancel the
  relevant change request" — the `item-press` close request in `onOpenChange` when the input is
  outside the popup, or the clear request in `onInputValueChange` marked with
  `eventDetails.isItemPress` when it is inside. "The typed filter still resets once the popup
  closes." The cancel() mechanism matches behavior.md "Events" (`cancel()` vetoes transitions,
  `'item-press'` reason), but the default clear-on-select/close-on-select multiple-mode behavior
  and the `isItemPress` details flag are not recorded in behavior.md (see Discrepancies).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:226`, `docs/src/app/(docs)/react/components/combobox/page.mdx:228`, `docs/src/app/(docs)/react/components/combobox/page.mdx:256`
- "`<Combobox.Input>` can be rendered inside `<Combobox.Popup>` to create a searchable select
  popup." Matches behavior.md "Whole-unit cross-cutting behavior" (input-inside-popup is one of the
  two structural compositions) and "DOM structure & portal behavior".
  `docs/src/app/(docs)/react/components/combobox/page.mdx:260`
- Input-inside-popup labeling: "Use `<Combobox.Label>` to provide a visible label for the combobox
  trigger in this pattern", and "`<Combobox.Label>` renders a `<div>`, so clicking it focuses the
  combobox trigger without opening the popup." The labeling role matches behavior.md
  "Accessibility"; the div element type and click-focuses-trigger-without-opening claim are not
  recorded in behavior.md (see Discrepancies).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:266`, `docs/src/app/(docs)/react/components/combobox/page.mdx:276`
- Grouping: "Organize related options with `<Combobox.Group>` and `<Combobox.GroupLabel>` to add
  section headings inside the popup", and "Groups are represented by an array of objects with an
  `items` property, which itself is an array of individual items for each group. An extra property,
  such as `value`, can be provided for the heading text." Group/GroupLabel parts match behavior.md
  "Public API surface" and "Accessibility" (group/labelling wiring); the grouped input-data shape
  is docs-only (see Discrepancies).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:280`, `docs/src/app/(docs)/react/components/combobox/page.mdx:282`
- Async search (single and multiple): "Load items from a remote source by fetching on input
  changes. Keep the selected item in the `items` list so it remains available while new results
  stream in. This pattern avoids needing to load items upfront." Fetching on input changes implies
  `onInputValueChange` (behavior.md "Events"); the keep-selected-item advice matches the label
  degradation behavior in behavior.md "State model".
  `docs/src/app/(docs)/react/components/combobox/page.mdx:311`, `docs/src/app/(docs)/react/components/combobox/page.mdx:319`
- Creatable: "Create a new item when the filter matches no items, opening a creation `<Dialog>`."
  Docs-only pattern description; behavior.md has no creation flow (its closest counterparts are
  `Empty` and the "no free-form value" model).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:327`
- Virtualization: "Efficiently handle large datasets using a virtualization library like
  `@tanstack/react-virtual`." Consistent with behavior.md "DOM structure & portal behavior"
  (items register at filtered indexes, leaving empty slots under virtualization) and the
  `virtualized` prop in "Public API surface".
  `docs/src/app/(docs)/react/components/combobox/page.mdx:335`
- Memoization guidance: "Memoizing each item is a simpler alternative to virtualization for
  datasets up to roughly 1,000 items ... While memoization speeds up typing, it does not speed up
  opening; with a large enough number of items, the mount cost dominates, and virtualization
  becomes necessary to keep the open interaction fast on low-end devices." Docs-only performance
  guidance with no behavior.md counterpart.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:343`
- `useFilter`: "Matches items against a query using `Intl.Collator` for robust string matching.
  This hook is used when externally filtering items. Pass the result to the `filter` prop."
  Consistent with behavior.md "Derived-value pipeline" (locale-aware collator filter) and the
  `filter` prop in "Public API surface".
  `docs/src/app/(docs)/react/components/combobox/page.mdx:473-475`
- `useFilteredItems`: "Returns the internally filtered items when called inside
  `<Combobox.Root>`." Consistent with the hook list in behavior.md "Public API surface".
  `docs/src/app/(docs)/react/components/combobox/page.mdx:481`
- `createItems`: "Normalizes items into a collection for the `items` prop of `<Combobox.Root>`,
  deriving each item's selection value and label before rendering." Consistent with behavior.md
  "State model" and "Derived-value pipeline".
  `docs/src/app/(docs)/react/components/combobox/page.mdx:487`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents 26 parts, each as its own heading rendering a generated
  reference component: `### Root` → `<TypesCombobox.Root />`, `### Label` → `<TypesCombobox.Label />`,
  `### Value` → `<TypesCombobox.Value />`, `### Icon` → `<TypesCombobox.Icon />`,
  `### Input` → `<TypesCombobox.Input />`, `### InputGroup` → `<TypesCombobox.InputGroup />`,
  `### Clear` → `<TypesCombobox.Clear />`, `### Trigger` → `<TypesCombobox.Trigger />`,
  `### Chips` → `<TypesCombobox.Chips />`, `### Chip` → `<TypesCombobox.Chip />`,
  `### ChipRemove` → `<TypesCombobox.ChipRemove />`, `### List` → `<TypesCombobox.List />`,
  `### Portal` → `<TypesCombobox.Portal />`, `### Backdrop` → `<TypesCombobox.Backdrop />`,
  `### Positioner` → `<TypesCombobox.Positioner />`, `### Popup` → `<TypesCombobox.Popup />`,
  `### Arrow` → `<TypesCombobox.Arrow />`, `### Status` → `<TypesCombobox.Status />`,
  `### Empty` → `<TypesCombobox.Empty />`, `### Collection` → `<TypesCombobox.Collection />`,
  `### Row` → `<TypesCombobox.Row />`, `### Item` → `<TypesCombobox.Item />`,
  `### ItemIndicator` → `<TypesCombobox.ItemIndicator />`, `### Group` → `<TypesCombobox.Group />`,
  `### GroupLabel` → `<TypesCombobox.GroupLabel />`, `### Separator` → `<TypesCombobox.Separator />`.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:365-469`
- Three hook/util sections render generated references with `hideDescription`:
  `<TypesCombobox.useFilter hideDescription />` under `## useFilter`,
  `<TypesCombobox.useFilteredItems hideDescription />` under `## useFilteredItems`, and
  `<TypesCombobox.createItems hideDescription />` under `## createItems`.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:471-489`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesCombobox } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:365`
- Parts documented on this page (26): Root, Label, Value, Icon, Input, InputGroup, Clear, Trigger,
  Chips, Chip, ChipRemove, List, Portal, Backdrop, Positioner, Popup, Arrow, Status, Empty,
  Collection, Row, Item, ItemIndicator, Group, GroupLabel, Separator. behavior.md "Public API
  surface (props, parts, subcomponents)" enumerates a 21-part set that omits Icon, Backdrop,
  Arrow, Status, and Separator (see Discrepancies); hooks/utils documented here (useFilter,
  useFilteredItems, createItems) are also listed in behavior.md "Public API surface", which
  additionally names `useComboboxRootContext` — not documented on this page.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:367-469`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

Twelve fenced code blocks, all written inline in the page (none pulled from `demos/`):

- ` ```jsx title="Anatomy" ` — imports `{ Combobox }` from `@base-ui/react/combobox` and assembles
  the full part tree: `Combobox.Root` > `Combobox.Label` + `Combobox.InputGroup`
  (`Input`, `Trigger`, `Icon`, `Clear`, `Value`, `Chips` > `Chip` > `ChipRemove`) +
  `Combobox.Portal` > `Backdrop` > `Positioner` > `Popup` (`Arrow`, `Status`, `Empty`,
  `List` > `Row` > `Item` > `ItemIndicator`, `Separator`, `Group` > `GroupLabel`, `Collection`).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:24-72`
- ` ```jsx title="Using item objects as values" ` — `Combobox.List` render prop mapping each item
  to `<Combobox.Item key={item.id} value={item}>`.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:78-86`
- ` ```tsx title="Specifying generic type parameters" ` — typed `MyCombobox<Value, Multiple extends
  boolean | undefined = false, Item = Value>` wrapper forwarding `Combobox.Root.Props`.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:96-105`
- ` ```jsx title="Using item IDs as values" ` — module-scope `Combobox.createItems(users, { getValue,
  getLabel })` passed to `items`, with `<Combobox.Item value={user.id}>` in the List render prop.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:115-130`
- ` ```jsx title="Creating a collection from dynamic data" ` — same collection wrapped in
  `React.useMemo(..., [users])`.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:136-145`
- ` ```tsx title="Displaying a window of async results" ` — Root with `items` (memoized
  `createItems` over `knownUsers`), `filteredItems={searchResults}`, `value`/`onValueChange`.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:157-173`
- ` ```tsx title="Limiting visible chips" ` — `Combobox.Value` render prop slicing the selected
  array to a `CHIP_LIMIT` of 3, rendering a "+N more" span, per-chip `aria-description`/`ChipRemove`
  `aria-label`, and an `aria-description` on `Combobox.Input`.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:191-222`
- ` ```jsx title="Input outside the popup" ` — `multiple` Root whose `onOpenChange` cancels the
  `item-press` close request (`eventDetails.reason === 'item-press'` → `eventDetails.cancel()`).
  `docs/src/app/(docs)/react/components/combobox/page.mdx:230-241`
- ` ```jsx title="Input inside the popup" ` — `multiple` Root whose `onInputValueChange` cancels
  when `eventDetails.isItemPress`.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:243-254`
- ` ```tsx title="Using Combobox.Label to label a combobox" ` — `<Combobox.Label>Favorite
  fruit</Combobox.Label>` as the first child of `Combobox.Root`.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:268-274`
- ` ```tsx title="Example" ` — `ProduceGroupItem` interface (`value: string`, `items: string[]`)
  and a `groups` array of two such objects, the data shape for grouped items.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:284-303`
- ` ```tsx title="Memoizing list items" ` — `React.memo`-wrapped `FruitItem` rendering
  `<Combobox.Item value={item}>` with `ItemIndicator`, used as the `Combobox.List` render prop.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:345-361`

Snippets contain docs-only `@highlight`, `@highlight-start`/`@highlight-end` comment markers
(docs syntax-highlighting annotations, not runtime code) — `docs/src/app/(docs)/react/components/combobox/page.mdx:197-199`, `docs/src/app/(docs)/react/components/combobox/page.mdx:208-210`, `docs/src/app/(docs)/react/components/combobox/page.mdx:270`, `docs/src/app/(docs)/react/components/combobox/page.mdx:287`, `docs/src/app/(docs)/react/components/combobox/page.mdx:294`.

Demo components rendered on the page (`DemoComboboxHero`, `DemoComboboxCreateItems`,
`DemoComboboxMultiple`, `DemoComboboxInputInsidePopup`, `DemoComboboxGrouped`,
`DemoComboboxAsyncSingle`, `DemoComboboxAsyncMultiple`, `DemoComboboxCreatable`,
`DemoComboboxVirtualized`) are imported from `./demos/*`; their code lives in demo files and is
out of scope here (Stage 2) — `docs/src/app/(docs)/react/components/combobox/page.mdx:9`, `docs/src/app/(docs)/react/components/combobox/page.mdx:147`, `docs/src/app/(docs)/react/components/combobox/page.mdx:182`, `docs/src/app/(docs)/react/components/combobox/page.mdx:262`, `docs/src/app/(docs)/react/components/combobox/page.mdx:305`, `docs/src/app/(docs)/react/components/combobox/page.mdx:313`, `docs/src/app/(docs)/react/components/combobox/page.mdx:321`, `docs/src/app/(docs)/react/components/combobox/page.mdx:329`, `docs/src/app/(docs)/react/components/combobox/page.mdx:337`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/combobox/behavior.md`.
Items flagged for lack of coverage or part-set mismatch:

- Part-set mismatch (page vs. behavior.md): the page documents 26 parts in its API reference and
  anatomy, including `Icon`, `Backdrop`, `Arrow`, `Status`, and `Separator`. behavior.md "Public
  API surface (props, parts, subcomponents)" enumerates a 21-part set without those five
  (`Backdrop`, `Arrow`, `Icon`, and `Separator` appear nowhere in behavior.md; `Status` is covered
  only by the "Whole-unit cross-cutting behavior" live-region protocol item, not the API-surface
  enumeration). The docs page, not behavior.md, is the authoritative public part list here;
  behavior.md's enumeration appears incomplete relative to the shipped API.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:24-72`, `docs/src/app/(docs)/react/components/combobox/page.mdx:367-469`
- Docs-only claim, not covered by behavior.md: `onInputValueChange`'s details object carries an
  `isItemPress` flag (`eventDetails.isItemPress`) used to identify selection-driven input clears.
  behavior.md "Events (names, payload shape, bubbling, preventDefault semantics)" records the
  details contract (`reason`, `event`, `cancel()`) and reason strings but never `isItemPress`.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:228`, `docs/src/app/(docs)/react/components/combobox/page.mdx:243-254`
- Docs-only claim, not covered by behavior.md: in `multiple` mode, selecting an item clears the
  typed filter and closes the popup (when the input is outside it), and the typed filter still
  resets once the popup closes. behavior.md's state model describes inputValue freeze/frozen-query
  around close animations but does not record this select-clears-filter/close-on-select default.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:226`, `docs/src/app/(docs)/react/components/combobox/page.mdx:256`
- Docs-only claim, not covered by behavior.md: `<Combobox.Label>` renders a `<div>` and clicking it
  focuses the combobox trigger without opening the popup. behavior.md "Accessibility" covers only
  the Label's labeling target (Trigger) and its dev warning.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:276`
- Docs-only authoring guidance, not covered by behavior.md: the screen-reader chip recipe
  (`aria-description` on Chip and Input, `aria-label` on Chips and ChipRemove; Base UI ships no
  such strings). behavior.md covers chip roles, keyboard removal, and focus return but not these
  author-supplied attributes.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:186-187`
- Docs-only claim, not covered by behavior.md: `filteredItems` must receive source items (not
  derived values) and preserves the flat or grouped structure of `items`; `items` represents
  records known to the app while `filteredItems` is the displayed result window. behavior.md
  records `filteredItems` as a prop and a filter window but not this structural contract.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:155`
- Docs-only claim, not covered by behavior.md: grouped data is an array of objects each holding an
  `items` array plus an optional heading property (`value`). behavior.md documents Group/GroupLabel
  wiring but not the input data shape.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:282`
- Omission (docs page vs. behavior.md): the page prose never mentions many root props behavior.md
  records — `grid`, `inline`, `modal`, `virtualized` (prose name-checks virtualization only via a
  library), `autoHighlight`, `highlightItemOnHover`, `loopFocus`, `openOnInputClick`, `limit`,
  `name`/`autoComplete`/`required`/`form` — those are reachable only through the generated API
  tables. Not a contradiction; a discoverability gap between page prose and the mined test surface.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:363-469`

## Cross-links to other docs pages

- [Select](/react/components/select) — linked twice in Usage guidelines: as the comparison component
  Combobox filters like ("similar to Select but whose items are filterable using an input") and as
  the recommended alternative when no input is rendered.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:15`, `docs/src/app/(docs)/react/components/combobox/page.mdx:17`
- [Autocomplete](/react/components/autocomplete) — recommended for simple search widgets since
  Combobox does not allow free-form text input.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:16`
- [Forms guide](/react/handbook/forms) — linked from the accessible-name guideline.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:18`
- Internal anchors (same-page, not cross-page): `#input-inside-popup` from the Usage guidelines
  (`docs/src/app/(docs)/react/components/combobox/page.mdx:18`) and `#value-selection-with-ids`
  from Item values (`docs/src/app/(docs)/react/components/combobox/page.mdx:88`).
- One external link: `https://react.dev/reference/react/memo` (React.memo reference) in the
  Memoizing items guidance. `docs/src/app/(docs)/react/components/combobox/page.mdx:343`
- Demo components (`./demos/hero`, `./demos/create-items`, `./demos/multiple`,
  `./demos/input-inside-popup`, `./demos/grouped`, `./demos/async-single`, `./demos/async-multiple`,
  `./demos/creatable`, `./demos/virtualized`) are imported and rendered on this page itself; they
  are same-page imports, not cross-page links.
  `docs/src/app/(docs)/react/components/combobox/page.mdx:9`, `docs/src/app/(docs)/react/components/combobox/page.mdx:147`, `docs/src/app/(docs)/react/components/combobox/page.mdx:182`, `docs/src/app/(docs)/react/components/combobox/page.mdx:262`, `docs/src/app/(docs)/react/components/combobox/page.mdx:305`, `docs/src/app/(docs)/react/components/combobox/page.mdx:313`, `docs/src/app/(docs)/react/components/combobox/page.mdx:321`, `docs/src/app/(docs)/react/components/combobox/page.mdx:329`, `docs/src/app/(docs)/react/components/combobox/page.mdx:337`
