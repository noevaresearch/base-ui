# Autocomplete — docs-content page spec (Stage 1: docs mining)

Mined from `docs/src/app/(docs)/react/components/autocomplete/page.mdx` (309 lines) only. The component's
own behavior is NOT re-derived here — cross-checks reference `specs/library/autocomplete/behavior.md` by
section name. Demo source files under `demos/` are intentionally out of scope (Stage 2:
`specs/docs-content/autocomplete/demos.json`); this spec only notes where the page embeds demo components.

## Page structure (headings, in order)

- H1 `Autocomplete` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:1`) with Subtitle "An input that suggests options as you type." (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:3`) and a Meta description calling it "a high-quality, unstyled React autocomplete component that renders an input with a list of filtered options" (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:4-7`)
- Hero demo embedded immediately after the title (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:9-11`)
- H2 `Usage guidelines` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:13`)
- H2 `Anatomy` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:19`)
- H2 `Item values` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:63`)
- H2 `Examples` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:67`)
  - H3 `Async search` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:69`)
  - H3 `Inline autocomplete` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:77`)
  - H3 `Grouped` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:85`)
  - H3 `Fuzzy matching` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:116`)
  - H3 `Limit results` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:124`)
  - H3 `Auto highlight` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:132`)
  - H3 `Command palette` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:142`)
  - H3 `Grid layout` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:150`)
  - H3 `Virtualized` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:158`)
    - H4 `Memoizing items` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:166`)
- H2 `API reference` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:191`) with H3 entries in this order: `Root`, `Value`, `Input`, `InputGroup`, `Trigger`, `Icon`, `Clear`, `List`, `Portal`, `Backdrop`, `Positioner`, `Popup`, `Arrow`, `Status`, `Empty`, `Collection`, `Row`, `Item`, `Group`, `GroupLabel`, `Separator` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:195-277`)
- H2 `useFilter` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:279`)
- H2 `useFilteredItems` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:286`)
- Trailing `export const metadata` block with SEO keywords such as 'React Autocomplete', 'Typeahead', 'Autosuggest', 'Accessible Combobox' (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:292-309`)

Each example section body is a one-sentence intro plus a demo component import/render (e.g. `DemoAutocompleteAsync` at `docs/src/app/(docs)/react/components/autocomplete/page.mdx:73-75`); their code content is Stage 2's scope.

## Prose claims about component behavior

- "An input that suggests options as you type." (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:3`) — consistent with behavior.md § State model (popup opens on typing).
- The component "renders an input with a list of filtered options" (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:6`) — consistent with behavior.md § Public API surface (`mode='list'` default filters supplied items).
- Avoid Autocomplete when selection state is needed: use Combobox instead if the selection should be remembered and the input value cannot be custom (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:15`) — docs-only usage guidance; the Combobox comparison is outside behavior.md's scope (no contradiction).
- Unlike Combobox, Autocomplete's input can contain free-form text, as its suggestions only _optionally_ autocomplete the text (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:15`) — consistent with behavior.md § State model (inline completion is opt-in via `mode`).
- The input can act as a filter for command items that perform an action when clicked, rendered inside the popup (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:16`) — consistent with behavior.md § Focus management (popup-input composition) and § Accessibility (aria passthrough on the popup).
- A form control must have an accessible name, created via a `<label>` element or the `Field` component (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:17`) — consistent with behavior.md § Accessibility (Field.Label links `aria-labelledby`, Field.Description links `aria-describedby`).
- Each `<Autocomplete.Item>` takes a `value` prop identifying it; pass the item being rendered so props like `itemToStringValue` receive it (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:65`) — consistent with behavior.md § Public API surface (Item `value` string or object) and § Edge cases (object matching by `label`/`value`, `itemToStringValue` fallback).
- Async search example: load items asynchronously while typing and render custom status content (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:71`) — consistent with behavior.md § Edge cases (`filter=null` disables default filtering for the async-results pattern).
- Inline autocomplete: autofill the input with the highlighted item while navigating with arrow keys using the `mode` prop, which "Accepts `aria-autocomplete` values `list`, `both`, `inline`, or `none`" (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:79`) — accepted values match behavior.md § Public API surface (`mode`); the page does not state the default (`list`, per behavior.md § State model).
- Grouped: organize options with `<Autocomplete.Group>` and `<Autocomplete.GroupLabel>` to add section headings inside the popup (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:87`).
- Groups are represented by an array of objects with an `items` property (an array of individual items per group); an extra property such as `value` provides the heading text (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:89`) — no coverage in behavior.md (Group/GroupLabel untested); see Discrepancies.
- Fuzzy matching finds relevant results even when the query doesn't exactly match the item text (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:118`) — docs-only framing; behavior.md § Edge cases confirms a custom `filter` replaces the locale-aware default.
- Limit results: cap the number of visible items with the `limit` prop and guide users to refine their query using `<Autocomplete.Status>` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:126`) — `limit` has no coverage in behavior.md; see Discrepancies.
- Auto highlight: the first matching item can be automatically highlighted as the user types via the `autoHighlight` prop on `<Autocomplete.Root>`; set it to `"always"` when the highlight should always be present, "such as when the list is rendered inline within a dialog" (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:134`) — consistent with behavior.md § Public API surface (`autoHighlight` boolean or `'always'`) and § Edge cases (`autoHighlight='always'` highlights immediately on open).
- `autoHighlight` "can be combined with the `keepHighlight` and `highlightItemOnHover` props to configure how the highlight behaves during mouse interactions" (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:136`) — `keepHighlight` matches behavior.md § Edge cases; `highlightItemOnHover` has no coverage in behavior.md; see Discrepancies.
- Command palette: use the autocomplete input to filter a list of command items that perform an action when clicked (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:144`) — same claim shape as the usage guideline at `docs/src/app/(docs)/react/components/autocomplete/page.mdx:16`.
- Grid layout: display items in a grid by wrapping each row in `<Autocomplete.Row>` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:152`) — Row is untested in this unit; see Discrepancies.
- Virtualized: efficiently handle large datasets with a virtualization library like `@tanstack/react-virtual` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:160`) — perf guidance, out of behavior.md's scope.
- Memoizing items is a simpler alternative to virtualization for datasets up to roughly 1,000 items: wrap each item in `React.memo` and pass the item as a prop so unchanged items skip re-rendering; memoization speeds up typing but not opening, and with enough items the mount cost dominates so virtualization becomes necessary for a fast open on low-end devices (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:168`) — docs-only perf guidance, no behavioral contradiction.
- `useFilter` "Matches items against a query using `Intl.Collator` for robust string matching" and "is used when externally filtering items" (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:281-282`) — consistent in spirit with behavior.md § Edge cases (locale-aware default filter, e.g. `locale="tr"`); the `Intl.Collator` mechanism itself is not stated in behavior.md.
- `useFilteredItems` "Returns the internally filtered items when called inside `<Autocomplete.Root>`" (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:288`) — no coverage in behavior.md (hook untested in this unit); see Discrepancies.

## API tables referenced (props/parts documented on this page)

The page embeds no inline prop tables; all API tables are generated `<TypesAutocomplete.* />` components imported from `./types` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:193`).

Parts with an API reference entry, in page order: `Root` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:195-197`), `Value` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:199-201`), `Input` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:203-205`), `InputGroup` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:207-209`), `Trigger` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:211-213`), `Icon` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:215-217`), `Clear` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:219-221`), `List` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:223-225`), `Portal` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:227-229`), `Backdrop` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:231-233`), `Positioner` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:235-237`), `Popup` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:239-241`), `Arrow` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:243-245`), `Status` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:247-249`), `Empty` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:251-253`), `Collection` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:255-257`), `Row` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:259-261`), `Item` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:263-265`), `Group` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:267-269`), `GroupLabel` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:271-273`), `Separator` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:275-277`).

Hooks with API reference entries: `useFilter` rendered via `<TypesAutocomplete.useFilter hideDescription />` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:279-284`) and `useFilteredItems` rendered via `<TypesAutocomplete.useFilteredItems hideDescription />` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:286-290`).

## Code snippets embedded directly in the .mdx (not pulled from demos/)

Exactly three fenced snippets; all other example bodies are demo placeholders.

1. "Anatomy" (```jsx, `docs/src/app/(docs)/react/components/autocomplete/page.mdx:23-61`): the full part tree — `Root` wrapping `InputGroup` (containing `Input`, `Trigger`, `Icon`, `Clear`, `Value`) and `Portal > Backdrop > Positioner > Popup` (containing `Arrow`, `Status`, `Empty`, and `List` which holds `Row > Item`, `Separator`, `Group > GroupLabel`, and `Collection`) — imported from `@base-ui/react/autocomplete` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:24`).
2. "Example" (```tsx, `docs/src/app/(docs)/react/components/autocomplete/page.mdx:91-110`): the grouped data shape — a `ProduceGroupItem` interface with `value: string` and `items: string[]` (both marked `// @highlight`), and a `groups` array of `Fruits`/`Vegetables` objects.
3. "Memoizing list items" (```tsx, `docs/src/app/(docs)/react/components/autocomplete/page.mdx:170-189`): a `Suggestion` interface, a `React.memo`-wrapped `SuggestionItem` rendering `Autocomplete.Item value={item}` with label/description spans, and `<Autocomplete.List>` with a render-prop child keyed by `item.id`.

## Discrepancies (docs page vs. behavior.md)

- `limit` prop: the page documents it as the way to "Limit the number of visible items" (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:126`), but `specs/library/autocomplete/behavior.md` never mentions `limit` (grep-confirmed) — no test coverage. `specs/library/autocomplete/implementation.md:70` explicitly lists `limit` among inherited-but-undocumented props with "no autocomplete JSDoc and no autocomplete test". Docs page documents surface the library specs treat as accidental leakage; trust neither without verification.
- `highlightItemOnHover` prop: named on the page as combinable with `autoHighlight`/`keepHighlight` (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:136`), but it appears nowhere in `specs/library/autocomplete/behavior.md` (grep-confirmed) nor in `specs/library/autocomplete/implementation.md` — a docs-only, test-unverifiable claim.
- Documented-but-untested parts: behavior.md § Public API surface lists only `Root`, `Input`, `Trigger`, `Value`, `InputGroup`, `Portal`, `Positioner`, `Popup`, `List`, `Item` as exercised by tests, while the page's Anatomy (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:26-56`) and API reference (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:195-277`) additionally document `Icon`, `Clear`, `Backdrop`, `Arrow`, `Status`, `Empty`, `Collection`, `Row`, `Group`, `GroupLabel`, `Separator` — `specs/library/autocomplete/implementation.md:73` confirms ten barrel parts (plus `useFilter`/`useFilteredItems`) are untested under the Autocomplete name. Coverage gap, not contradiction.
- Grouped-items data shape (`{ value, items }` arrays, `docs/src/app/(docs)/react/components/autocomplete/page.mdx:89`): no runtime coverage in behavior.md; `specs/library/autocomplete/implementation.md:71` notes grouping is "type-tested only".
- `mode` default: the page lists accepted values without naming a default (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:79`); behavior.md § State model establishes `mode='list'` as the default. Omission on the docs side, not a contradiction.
- No direct contradictions found: every page claim checkable against behavior.md (autoHighlight/`'always'`, keepHighlight, `itemToStringValue` flow, `mode` values, accessible-name wiring, locale-aware filtering) agrees with the corresponding behavior.md section.

## Cross-links to other docs pages

- [Combobox](/react/components/combobox) — recommended instead of Autocomplete when selection state is needed (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:15`).
- [Forms guide](/react/handbook/forms) — referenced for accessible naming of form controls (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:17`).
- External (not a docs page): [`React.memo`](https://react.dev/reference/react/memo) in the memoization guidance (`docs/src/app/(docs)/react/components/autocomplete/page.mdx:168`).
