# Combobox — implementation spec (Stage 2: implementation mining)

Mechanism notes for the whole unit. The ground truth of WHAT happens is `specs/library/combobox/behavior.md` (cited below by section name only); this file explains HOW the sources produce it. All paths are repo-relative. Everything is first-party: the unit delegates to no external package, and the headless core `packages/react/src/combobox/root/AriaCombobox.tsx` is additionally reused by the sibling Autocomplete unit (see the last section).

## State machine / hooks used

### The store

The root builds exactly one `ReactStore` per combobox, created once through `useRefWithInit` (`packages/react/src/combobox/root/AriaCombobox.tsx:453-542`) and typed as `ComboboxStore` (`packages/react/src/combobox/store.ts:205`). The store is the single source of truth every part subscribes to (see "One store drives all parts" in behavior.md).

State shape (`packages/react/src/combobox/store.ts:10-63`), grouped by role:

- Identity/labels: `id`, `labelId`, `items`, `itemToStringLabel`, `isItemEqualToValue`.
- Selection: `selectedValue`, `selectionMode` (`'single' | 'multiple' | 'none'`), `selectedIndex`, `activeIndex` — both indices are stored as *filtered-list coordinates*, not DOM order.
- Popup lifecycle: `open`, `mounted`, `transitionStatus`, `forceMounted`, `openMethod`.
- Structure: `inline`, `modal`, `grid`, `virtualized`, `inputInsidePopup`, `inputOwnsFormValue`.
- Elements: `positionerElement`, `listElement`, `popupId`, `triggerElement`, `inputElement`, `inputGroupElement`, `popupSide`.
- Pre-computed prop bags: `popupProps`, `listProps`, `inputProps`, `triggerProps`, `itemProps` — built once in the root and spread by the parts, so all floating hooks run only in the root.
- Config passthroughs: `disabled`, `readOnly`, `required`, `name`, `form`, `openOnInputClick`, `autoHighlight`, `submitOnItemClick`, `hasInputValue`.

The store is two-tier. Reactive `State` is read through `selectors` (`packages/react/src/combobox/store.ts:122-203`), which include derived predicates: `hasSelectedValue` (`packages/react/src/combobox/store.ts:134-143`, `[]` reads as "no value" in multiple mode), `hasSelectionChips` (`packages/react/src/combobox/store.ts:129-132`), `isActive` (`packages/react/src/combobox/store.ts:157`), and `isSelected`, which fans an array `selectedValue` through `compareItemEquality` (`packages/react/src/combobox/store.ts:158-167`). Non-reactive shared values live in `ComboboxStoreContext` (`packages/react/src/combobox/store.ts:69-120`): the refs every part registers into (`listRef`, `labelsRef`, `valuesRef`, `popupRef`, `inputRef`, `chipsContainerRef`, `clearRef`, `emptyRef`, start/end dismiss refs, `keyboardActiveRef`, `pointerDownItemRef`, `selectionEventRef`) plus command slots (`setOpen`, `setInputValue`, `setSelectedValue`, `setIndices`, `handleSelection`, `forceMount`, `requestSubmit`, `onOpenChangeComplete`). Commands are seeded with `NOOP` at construction (`packages/react/src/combobox/root/AriaCombobox.tsx:516-525`) and bound to the real `useStableCallback` implementations via `store.useContextCallback` (`packages/react/src/combobox/root/AriaCombobox.tsx:1437-1444`), so children can call commands on first render before the root's callbacks exist.

### Controlled/uncontrolled resolution

Three independent `useControlled` calls resolve open, value, and inputValue (see "State model" in behavior.md):

- `selectedValue` — default is `EMPTY_ARRAY` in multiple mode, else `defaultSelectedValue` (`packages/react/src/combobox/root/AriaCombobox.tsx:293-298`).
- `inputValue` — default derived once on first render from the selected value in single mode when neither `inputValue` nor `defaultInputValue` is given (`packages/react/src/combobox/root/AriaCombobox.tsx:314-329`).
- `open` — default `defaultOpen` (`packages/react/src/combobox/root/AriaCombobox.tsx:331-336`).

The public `ComboboxRoot` is a thin adapter that renames the value API onto the internal one — `value/defaultValue/onValueChange` → `selectedValue/defaultSelectedValue/onSelectedValueChange`, `multiple` → `selectionMode`, `autoComplete` → `formAutoComplete` (`packages/react/src/combobox/root/ComboboxRoot.tsx:23-31`) — and omits internal props from its type surface (`packages/react/src/combobox/root/ComboboxRoot.tsx:47-68`).

Every mutator is a `useStableCallback` that fires the user callback first, bails on `eventDetails.isCanceled`, then commits:

- `setOpen` (`packages/react/src/combobox/root/AriaCombobox.tsx:750-829`) also implements the close-animation query freeze: when closing with a changed query it captures `closeQuery` so filtering stays stable during the exit (see "Edge cases" in behavior.md), releases it on reopen via `handleInterruptedReopen` (`packages/react/src/combobox/root/AriaCombobox.tsx:728-748`), clears the input immediately for `inputInsidePopup`/inline, and commits Field touched/focused/validation on focus-out closes.
- `setInputValue` (`packages/react/src/combobox/root/AriaCombobox.tsx:648-726`) classifies typed input vs autofill by `inputType`, schedules `pendingQueryHighlightRef` for post-filter highlighting, releases a frozen `closeQuery` when typing proves the popup stays open, and resets non-virtualized list scroll on query change.
- `setSelectedValue` (`packages/react/src/combobox/root/AriaCombobox.tsx:831-854`) fills the input from the new selection when the input lives outside the popup.
- `handleSelection` (`packages/react/src/combobox/root/AriaCombobox.tsx:856-920`) is the funnel for all item-originated commits (item click, Enter, typeahead via `selectionEventRef`): it swaps in the override event, short-circuits on link targets, toggles the value in multiple mode, and clears the query only when the user was filtering.
- `setIndices` (`packages/react/src/combobox/root/AriaCombobox.tsx:615-646`) writes `activeIndex`/`selectedIndex` atomically in one `store.update` and emits `onItemHighlighted` through `emitHighlight` with `lastHighlightRef` dedup (`packages/react/src/combobox/root/AriaCombobox.tsx:600-613`; sentinel constants in `packages/react/src/combobox/root/utils/constants.ts:1-5`).

### Prop → store syncing and effects

Prop bags must be observable before parts render (parts read them with `useStore` during render), so `useOnFirstRender` seeds them synchronously (`packages/react/src/combobox/root/AriaCombobox.tsx:1448-1457`), and a `useIsoLayoutEffect` re-publishes the full `syncedValues` object on every change (`packages/react/src/combobox/root/AriaCombobox.tsx:1490-1501`). `inputOwnsFormValue` is folded into that same `update` because `ComboboxInput` writes it from a ref callback earlier in the commit — the comment at `packages/react/src/combobox/root/AriaCombobox.tsx:1490-1495` documents the no-intermediate-snapshot requirement (see "Form-value ownership" invariant in behavior.md). The store seeds `inputOwnsFormValue: selectionMode === 'none'` initially to keep server HTML free of duplicate names (`packages/react/src/combobox/root/AriaCombobox.tsx:511-514`).

Effect-driven state transitions:

- `syncSelectedIndex` (`packages/react/src/combobox/root/AriaCombobox.tsx:996-1041`) re-derives `selectedIndex` against `flatFilteredValues` when the query freeze releases, against the live `valuesRef` registry otherwise, and clears `pointerDownItemRef` on state-driven closes so stale drags can't poison the next open.
- `packages/react/src/combobox/root/AriaCombobox.tsx:1043-1048` syncs `valuesRef`/`listRef` lengths when the `items` prop drives the list.
- The pending-highlight effect (`packages/react/src/combobox/root/AriaCombobox.tsx:1050-1181`) consumes `pendingQueryHighlightRef` after filtered items are derived (so `onItemHighlighted` sees the new list), auto-highlights index 0, restores the highlight to the selected item after a query clear via `queueMicrotask` (items re-register in a follow-up commit), and re-emits/clears highlights when the active index falls off the end of the list.
- Field integration: `setFilled` from value/inputValue shape (`packages/react/src/combobox/root/AriaCombobox.tsx:1183-1191`).
- Empty list + `autoHighlight` clears the active index to avoid a double ArrowDown (`packages/react/src/combobox/root/AriaCombobox.tsx:1195-1199`).
- Six `useValueChanged` watchers (`packages/react/src/combobox/root/AriaCombobox.tsx:1265-1270`) translate state deltas into side effects: query change marks `queryChangedAfterOpen`, controlled-open changes release a frozen query, `selectedValue` changes clear Field errors, set dirty, run validation, and mirror the label into the input for single mode without an explicit `inputValue` (`packages/react/src/combobox/root/AriaCombobox.tsx:1221-1243`); `items`/`selectedLabelString` changes re-run the mirror (`packages/react/src/combobox/root/AriaCombobox.tsx:1248-1252`); `inputValue` changes drive dirty/validation in `selectionMode: 'none'` (`packages/react/src/combobox/root/AriaCombobox.tsx:1254-1263`).
- Unmount convergence: `useOpenChangeComplete` on the popup (or the closest `role="dialog"` ancestor for inline-in-Dialog) calls `handleUnmount`, which resets mounted/query/indices and reconciles the input to the selection (`packages/react/src/combobox/root/AriaCombobox.tsx:976-992`); `actionsRef.unmount()` exposes the same `handleUnmount` imperatively (`packages/react/src/combobox/root/AriaCombobox.tsx:994`).

### Derived-items pipeline (mechanism)

- `items` is normalized: plain arrays/groups stay as-is; `createItems()` collections (an opaque branded type, `packages/react/src/combobox/items/itemCollection.ts:35-38`) are detected and their `data`/`value` accessors extracted (`packages/react/src/combobox/root/AriaCombobox.tsx:162-177`); a non-collection object throws a `Base UI:` error (`packages/react/src/combobox/root/AriaCombobox.tsx:166-173`).
- `createComboboxItems` lazily indexes data into a `valueToItem` map on first access, first-occurrence-wins with a dev warning for duplicate derived values (`packages/react/src/combobox/items/createItems.ts:98-130`); `findCollectionItem` honors custom equality after an exact-map miss (`packages/react/src/combobox/items/itemCollection.ts:8-25`).
- Labels resolve live at read time — collection data, then the current `externalWindow` (an index over the `filteredItems` prop, `packages/react/src/combobox/root/AriaCombobox.tsx:186-209`), then `itemToStringLabel` — never cached at selection time (`packages/react/src/combobox/root/AriaCombobox.tsx:214-227`).
- The default filter is `createCollatorItemFilter` over `Intl.Collator` (`packages/react/src/combobox/root/utils/index.ts:23-34`); a `filter: null` prop disables filtering entirely (`packages/react/src/combobox/root/AriaCombobox.tsx:300-310`). The single-selection bypass that shows all items while the query equals the selection's label lives in `createSingleSelectionCollatorFilter` (`packages/react/src/combobox/root/utils/index.ts:40-68`), mirrored by `shouldBypassFiltering`/`shouldIgnoreExternalFiltering` in the root (`packages/react/src/combobox/root/AriaCombobox.tsx:343-355`).
- `filteredItems` memo applies the query and `limit` over flat or grouped shapes (`packages/react/src/combobox/root/AriaCombobox.tsx:362-438`); `flatFilteredValues` flattens and projects to selection values via the collection's `value` accessor (`packages/react/src/combobox/root/AriaCombobox.tsx:443-451`).
- Public hooks: `useComboboxFilter` (`packages/react/src/combobox/root/utils/useFilter.ts:33-49`), `useFilteredItems` (`packages/react/src/combobox/root/utils/useFilteredItems.ts:7-10`), `createItems` re-exported from `packages/react/src/combobox/index.parts.ts:29-31`.

### Floating hooks (composition order)

- `useFloatingRootContext` wires open state and element refs; the reference element is the Trigger when the input is inside the popup, otherwise the Input (`packages/react/src/combobox/root/AriaCombobox.tsx:1272-1279`). Inline mode forces `open: true` into the floating context (`packages/react/src/combobox/root/AriaCombobox.tsx:1273`).
- `useClick` — mousedown-only, `toggle: false`, reason `inputPress`, 100ms touch open delay outside-popup to avoid flip flicker (`packages/react/src/combobox/root/AriaCombobox.tsx:1323-1331`).
- `useDismiss` — sloppy mouse / intentional touch outside press, with an `outsidePress` predicate exempting the trigger, clear button, chips container, and input group (`packages/react/src/combobox/root/AriaCombobox.tsx:1333-1353`); Escape bubbling is enabled for inline lists so parent popups can close (`packages/react/src/combobox/root/AriaCombobox.tsx:1343`).
- `useListNavigation` — `virtual: true` (DOM focus never moves), optional `gridNavigation`, `loopFocus`/`allowEscape`, `resetOnPointerLeave: !keepHighlight`, and an `onNavigate` that routes into `setIndices` tagging keyboard vs pointer via `keyboardActiveRef` (`packages/react/src/combobox/root/AriaCombobox.tsx:1355-1389`).
- `useTypeahead` on the closed Trigger commits selections from `valuesRef` (`packages/react/src/combobox/trigger/ComboboxTrigger.tsx:106-119`).
- Prop bags are assembled with `mergeProps` (rightmost props win; handlers chain right-to-left and `preventBaseUIHandler` vetoes earlier handlers — `packages/react/src/merge-props/mergeProps.ts:17-19`): `inputProps` = listNavigation.reference + grid-caret guard + dismiss.reference + click.reference + role.reference (`packages/react/src/combobox/root/AriaCombobox.tsx:1391-1414`), `popupProps` = focusable-popup defaults + dismiss.floating (`packages/react/src/combobox/root/AriaCombobox.tsx:1416-1419`), `listProps` = listNavigation.floating + role.floating (`packages/react/src/combobox/root/AriaCombobox.tsx:1421-1424`), `itemProps` = listNavigation.item minus `onFocus` so item focus can't hijack navigation state (`packages/react/src/combobox/root/AriaCombobox.tsx:1426-1435`).

### Other hooks

- `useTransitionStatus` (`packages/react/src/combobox/root/AriaCombobox.tsx:573`), reused per-part by Clear (`packages/react/src/combobox/clear/ComboboxClear.tsx:69-86`) and ItemIndicator (`packages/react/src/combobox/item-indicator/ComboboxItemIndicator.tsx:44-74`) for their own enter/exit unmounting.
- `useOpenInteractionType` supplies `openMethod` plus `triggerProps` (an `onClick`/`onPointerDown` pair that records the opening interaction type; `packages/react/src/combobox/root/AriaCombobox.tsx:574`, `packages/react/src/utils/useOpenInteractionType.ts:19-44`).
- `useValueAsRef(triggerElement)` gives a stable trigger ref for field-control registration (`packages/react/src/combobox/root/AriaCombobox.tsx:571`).
- Field/form integration: `useFormContext` (error clearing), `useFieldRootContext` (dirty/touched/focused/validation/`setFilled`), `useRegisterFieldControl` (`packages/react/src/combobox/root/AriaCombobox.tsx:144,145-155,578-585`).
- `useLabelableId` for the root id (`packages/react/src/combobox/root/AriaCombobox.tsx:158`) and `useCoreFilter` for the collator (`packages/react/src/combobox/root/AriaCombobox.tsx:159`).
- Local component state: IME composition mirror in Input (`packages/react/src/combobox/input/ComboboxInput.tsx:97-100`), `highlightedChipIndex` in Chips (`packages/react/src/combobox/chips/ComboboxChips.tsx:28-36`), `labelId` in Group (`packages/react/src/combobox/group/ComboboxGroup.tsx:24`), `useTimeout` for the trigger's force-mount-on-focus (`packages/react/src/combobox/trigger/ComboboxTrigger.tsx:80,173`) and the live-region marker reset (`packages/react/src/combobox/utils/useInitialLiveRegionTextMutation.ts:49`).

## Context providers/consumers

The root nests five providers (`packages/react/src/combobox/root/AriaCombobox.tsx:1643-1655`):

All five are defined in `packages/react/src/combobox/root/ComboboxRootContext.tsx`:

| Context | Carries | Consumers |
| --- | --- | --- |
| `ComboboxRootContext` (line 17) | the `ComboboxStore` | every part via `useComboboxRootContext`, which throws outside a Root (`packages/react/src/combobox/root/ComboboxRootContext.tsx:30-38`) |
| `ComboboxFloatingContext` (line 18-20) | the floating-ui `FloatingRootContext` | Trigger (typeahead + click hooks, `packages/react/src/combobox/trigger/ComboboxTrigger.tsx:77,106,121`), Popup (focus manager, `packages/react/src/combobox/popup/ComboboxPopup.tsx:41,128`), Positioner (anchor positioning, `packages/react/src/combobox/positioner/ComboboxPositioner.tsx:54,74`), List (`floatingId`, `packages/react/src/combobox/list/ComboboxList.tsx:30,67`) |
| `ComboboxHasItemsContext` (line 24) | whether the root `items` prop exists | Item — selects between registry-driven and self-registration index bookkeeping (`packages/react/src/combobox/item/ComboboxItem.tsx:60,96-133`) |
| `ComboboxDerivedItemsContext` (line 6-15, 21-23) | `{ query, hasItems, filteredItems, flatFilteredValues }` | List (`packages/react/src/combobox/list/ComboboxList.tsx:32`), Collection (`packages/react/src/combobox/collection/ComboboxCollection.tsx:17-20`), Empty (`packages/react/src/combobox/empty/ComboboxEmpty.tsx:29`), the virtualized Item wrapper (`packages/react/src/combobox/item/ComboboxItem.tsx:236`), and `useListEmpty` (`packages/react/src/combobox/utils/parts.ts:21-23`) which feeds the `listEmpty` state of Input/Trigger/InputGroup/Popup/Positioner; public `useFilteredItems` reads it too |
| `ComboboxInputValueContext` (line 27-28) | the raw `inputValue` | Input (`packages/react/src/combobox/input/ComboboxInput.tsx:68`), Trigger (`packages/react/src/combobox/trigger/ComboboxTrigger.tsx:78`), Clear (`packages/react/src/combobox/clear/ComboboxClear.tsx:51`). Kept out of the store on purpose — comment cites mui/base-ui#2703 |

Sub-tree contexts:

- `ComboboxPortalContext` — `Portal` provides its `keepMounted` flag (`packages/react/src/combobox/portal/ComboboxPortal.tsx:31-35`); `Positioner` consumes it to keep positioning alive while hidden (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:55,86`). The hook throws when `Portal` is missing (`packages/react/src/combobox/portal/ComboboxPortalContext.tsx:6-12`).
- `ComboboxPositionerContext` — the positioning slice (`side`, `align`, `arrowRef`, `arrowUncentered`, `arrowStyles`, `anchorHidden`, `isPositioned`; `packages/react/src/combobox/positioner/ComboboxPositionerContext.tsx:5-14`), provided by Positioner (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:124`). Required consumers: Popup (`packages/react/src/combobox/popup/ComboboxPopup.tsx:40`), Arrow (`packages/react/src/combobox/arrow/ComboboxArrow.tsx:23`). Optional (presence-probing) consumers: Input uses it to detect "inside popup" (`packages/react/src/combobox/input/ComboboxInput.tsx:63-64`), List uses it to decide whether to register itself as the positioner element fallback (`packages/react/src/combobox/list/ComboboxList.tsx:31,71`).
- `ComboboxItemContext` — `{ selected, textRef }` from Item (`packages/react/src/combobox/item/ComboboxItem.tsx:208-218`); consumed by ItemIndicator (`packages/react/src/combobox/item-indicator/ComboboxItemIndicator.tsx:20,40`); throws outside Item (`packages/react/src/combobox/item/ComboboxItemContext.ts:11-18`).
- `ComboboxRowContext` — a boolean set by Row (`packages/react/src/combobox/row/ComboboxRow.tsx:25`); Item switches `role="option"` → `role="gridcell"` on it (`packages/react/src/combobox/item/ComboboxItem.tsx:59,163`).
- `ComboboxGroupContext` — `{ labelId, setLabelId, items }` from Group (`packages/react/src/combobox/group/ComboboxGroup.tsx:26-33,48-50`); GroupLabel writes its id into it (newest-label-wins cleanup included, `packages/react/src/combobox/group-label/ComboboxGroupLabel.tsx:21-30`); throws outside Group (`packages/react/src/combobox/group/ComboboxGroupContext.ts:18-26`).
- `GroupCollectionContext` — `{ items }` provided by `GroupCollectionProvider` when `Group` receives `items` (`packages/react/src/combobox/group/ComboboxGroup.tsx:52-54`; `packages/react/src/combobox/collection/GroupCollectionContext.tsx:14-24`); Collection prefers it over root filtered items for group-scoped rendering (`packages/react/src/combobox/collection/ComboboxCollection.tsx:18-20`).
- `ComboboxChipsContext` — `{ highlightedChipIndex, setHighlightedChipIndex, chipsRef }` from Chips (`packages/react/src/combobox/chips/ComboboxChips.tsx:53-66`); consumers: Chip (arrow/backspace navigation, `packages/react/src/combobox/chip/ComboboxChip.tsx:29`) and Input (chip-key handling while the input has focus, `packages/react/src/combobox/input/ComboboxInput.tsx:62,143-189`).
- `ComboboxChipContext` — `{ index }` from Chip (`packages/react/src/combobox/chip/ComboboxChip.tsx:127-135`); ChipRemove uses it to know which value to splice out (`packages/react/src/combobox/chip-remove/ComboboxChipRemove.tsx:33`); throws outside Chip (`packages/react/src/combobox/chip/ComboboxChipContext.ts:10-18`).

Framework contexts consumed from outside the unit (Field, Labelable, Form, Direction) are catalogued in the dependencies section.

## DOM/portal strategy and why

### Nesting and mount gating

`Portal → Positioner → Popup → List (+ Empty/Group/Item)`. `Portal` renders `FloatingPortal` only when `mounted || keepMounted || forceMounted` (`packages/react/src/combobox/portal/ComboboxPortal.tsx:19-35`). `forceMounted` is a store flag set by the root's `forceMount` command (`packages/react/src/combobox/root/AriaCombobox.tsx:587-594`): trigger focus/mousedown and autofill force-mount the closed list so item labels register for typeahead and matching. This is the mechanism behind closed-state typeahead and the "list stays mounted while closed" notes in behavior.md.

### Anchoring

`Positioner` resolves its anchor as `inputGroupElement ?? inputElement`, or the `triggerElement` when the input is inside the popup (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:69-70`) — matching "anchors to the InputGroup when present, else the Input (never the Trigger)" in behavior.md, with the Trigger taking over only for the dialog-like composition. `useAnchorPositioning` receives `keepMounted` from the portal context and `lazyFlip: true` (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:72-89`); the `--available-height` CSS var that must be seeded before sizing is applied inside that hook (`packages/react/src/internals/useAnchorPositioning.ts:353-360`). The positioner element is rendered `hidden` when unmounted and `inert` while closed via the shared `usePositioner` helper (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:114-121`). The resolved side is published to the store for `data-popup-side` attributes (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:106-108`), and `usePopupSide` nulls it while the positioner is unmounted (`packages/react/src/combobox/utils/parts.ts:10-16`). Modal open locks scroll via `useAnchoredPopupScrollLock` (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:91-96`) and renders `InternalBackdrop` with a cutout over the anchored control (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:123-130`).

### Two structural compositions

The Input decides which composition it belongs to at ref time: `hasPositionerParent || inline` marks it "inside popup", and its ref callback publishes `inputElement`, `inputInsidePopup`, and `inputOwnsFormValue` together into the store (`packages/react/src/combobox/input/ComboboxInput.tsx:102-116`). Everything downstream keys off `inputInsidePopup`:

- **Role holder.** The root's `role.reference` bag is computed once — SSR-safe assumption that the control is an input, `role="combobox"`, `aria-expanded` (permanently `true` when `inline`), `aria-haspopup` (`grid`/`listbox`), `aria-controls` pointing at the list id, `aria-autocomplete: none` under `readOnly` (`packages/react/src/combobox/root/AriaCombobox.tsx:1281-1318`) — and is merged into `inputProps`, so the Input element carries it in both compositions (`packages/react/src/combobox/input/ComboboxInput.tsx:194-197`). `aria-activedescendant` is contributed by `useListNavigation`'s reference props as `${id}-${activeIndex}` (`packages/react/src/floating-ui-react/hooks/useListNavigation.ts:756-761,930-935`), matching the item id convention `${rootId}-${index}` (`packages/react/src/combobox/item/ComboboxItem.tsx:79`). In the input-outside-popup composition this makes the Input the sole combobox; in the input-inside-popup composition the Trigger is *additionally* promoted with hand-authored ARIA — `role="combobox"`, `tabIndex 0` (vs `-1` demotion outside), `aria-haspopup="dialog"`, `aria-controls` = stored/default popup id (`packages/react/src/combobox/trigger/ComboboxTrigger.tsx:153-163,90-98`) — and the popup becomes `role="dialog"` instead of `presentation` (`packages/react/src/combobox/popup/ComboboxPopup.tsx:90`).
- **Ids.** Outside-popup: the Input takes the root id; inside-popup: the Trigger takes the root id and the popup gets `${rootId}-popup` (`packages/react/src/combobox/input/ComboboxInput.tsx:94`; `packages/react/src/combobox/trigger/ComboboxTrigger.tsx:86-87`). The popup id convention lives in one helper shared by popup (writer) and trigger (`aria-controls`) reader (`packages/react/src/combobox/root/utils/index.ts:13-15`; written with DOM-id preference at `packages/react/src/combobox/popup/ComboboxPopup.tsx:54-62`).
- **Field ownership.** Inside the popup the Input is wrapped in a default (empty) `FieldRootContext` so it stops claiming Field state, leaving one field-control owner: the hidden control bound to the Trigger ref via `useRegisterFieldControl(inputInsidePopup ? triggerRef : inputRef, ...)` (`packages/react/src/combobox/input/ComboboxInput.tsx:95,118-120,460-466`; `packages/react/src/combobox/root/AriaCombobox.tsx:578-585`). This implements the two-sided "exactly one named control" invariant in behavior.md.

### List and item registration

`List` renders the `listProps` bag — `role="listbox"`/`"grid"`, `aria-multiselectable`, `aria-readonly` (moved to the combobox element in grid mode), `id` = the floating context's `floatingId` — plus Enter handling that delegates to `clickHighlightedItem` and capture listeners that maintain `keyboardActiveRef` (`packages/react/src/combobox/list/ComboboxList.tsx:63-106`). When used without a Positioner (inline or bare), the List registers itself as `positionerElement` (`packages/react/src/combobox/list/ComboboxList.tsx:71`). A function child is implicitly wrapped in `Collection` so per-item subscriptions don't cascade from the list's own prop subscriptions (`packages/react/src/combobox/list/ComboboxList.tsx:56-61`). Non-virtualized lists wrap in `CompositeList` over the store's `listRef`; typeahead labels come from the registry except when the `items` prop derives them (`packages/react/src/combobox/list/ComboboxList.tsx:111-124`).

Items resolve their index from (in order) the `index` prop, the filtered window (virtualized fallback, `packages/react/src/combobox/item/ComboboxItem.tsx:228-250`), or composite registration order (`packages/react/src/combobox/item/ComboboxItem.tsx:52-56,69`), and write `listRef[index]`/`valuesRef[index]` as sparse slots — the "empty slots under virtualization" behavior (`packages/react/src/combobox/item/ComboboxItem.tsx:82-107`). Each item also re-asserts `selectedIndex` when its composite position changes, which is what keeps closed-state typeahead correct under reordering (`packages/react/src/combobox/item/ComboboxItem.tsx:109-133`). Pointer handling keeps focus on the input: `pointerdown`/`mousedown` are default-prevented (with a non-primary-pointer guard against multi-touch double commits), and `mouseup` only commits when the gesture started on the item, the item is highlighted, and no drag was in progress (`packages/react/src/combobox/item/ComboboxItem.tsx:161-200`). Keyboard Enter never dispatches a synthetic event; it sets `selectionEventRef` and calls `listItem.click()` so the item's own handler attributes the commit (`packages/react/src/combobox/utils/parts.ts:46-57`), and `handleSelection` substitutes that override event into the change details (`packages/react/src/combobox/root/AriaCombobox.tsx:856-861`). `submitOnItemClick` (internal) flushes the selection synchronously and requests form submit (`packages/react/src/combobox/item/ComboboxItem.tsx:148-159`; `packages/react/src/combobox/root/AriaCombobox.tsx:922-927`). The item component is `React.memo`'d with a stable-branch guard so flipping `virtualized` cannot silently remount an item (`packages/react/src/combobox/item/ComboboxItem.tsx:258-284`).

### Hidden form control

The root renders the actual form control after `children` (`packages/react/src/combobox/root/AriaCombobox.tsx:1546-1641`): a single hidden `<input>` for scalar modes, plus one hidden input per value in multiple mode (`packages/react/src/combobox/root/AriaCombobox.tsx:1526-1544`). Naming is zero-sum: `hiddenInputName` is `undefined` in multiple mode and in `selectionMode: 'none'` when the visible input itself owns the form value (`packages/react/src/combobox/root/AriaCombobox.tsx:1522-1524`). The control redirects focus to the visible control (input → trigger fallback) on focus (`packages/react/src/combobox/root/AriaCombobox.tsx:1552-1559`) and implements browser autofill: case-insensitive match against serialized values then rendered labels, gated on `readOnly`/`disabled`, forcing the closed list mounted first, committing via `setSelectedValue` in a microtask (`packages/react/src/combobox/root/AriaCombobox.tsx:1561-1623`). Serialization goes through `fieldStringValue` (`packages/react/src/combobox/root/AriaCombobox.tsx:544-553`) and `itemToStringValue` (`packages/react/src/combobox/root/AriaCombobox.tsx:1515-1520`). Styling switches between `visuallyHiddenInput` (named, focusable for autofill) and `visuallyHidden` (`packages/react/src/combobox/root/AriaCombobox.tsx:1634`).

### Modal mode

`focusManagerModal` is true when the input is outside the popup or `modal` is set (`packages/react/src/combobox/popup/ComboboxPopup.tsx:125-131`; `packages/react/src/combobox/input/ComboboxInput.tsx:93`). In that mode: `FloatingFocusManager` runs modal (Tab cycling, inert outside content), a hidden start-dismiss button renders before the input and an end-dismiss button after the popup content, and both are declared as inside elements so focus traversal treats them as popup content (`packages/react/src/combobox/input/ComboboxInput.tsx:468-475`; `packages/react/src/combobox/popup/ComboboxPopup.tsx:135-144`; button internals `packages/react/src/combobox/utils/ComboboxInternalDismissButton.tsx:15-45`). Default focus behavior: touch-open focuses the popup element (Android keyboard suppression), otherwise the input; `initialFocus={false}` and `finalFocus` passthrough are honored, and final focus is disabled by default outside-popup so focus never leaves the input (`packages/react/src/combobox/popup/ComboboxPopup.tsx:107-123`).

### Inline mode and live regions

Inline forces the floating context open (`packages/react/src/combobox/root/AriaCombobox.tsx:1273`), reports `aria-expanded="true"` permanently (`packages/react/src/combobox/root/AriaCombobox.tsx:1282-1285`), dismisses via Escape only with bubbling enabled (`packages/react/src/combobox/root/AriaCombobox.tsx:1343`), and clears the input immediately on close instead of deferring to unmount (`packages/react/src/combobox/root/AriaCombobox.tsx:800-809`). When an inline list is composed inside a Dialog, transition completion is observed on the closest `[role="dialog"]` ancestor rather than the popup (`packages/react/src/combobox/root/AriaCombobox.tsx:972-981`).

`Empty` and `Status` are the live regions: `role="status"`, `aria-live="polite"`, `aria-atomic` (`packages/react/src/combobox/empty/ComboboxEmpty.tsx:35-46`; `packages/react/src/combobox/status/ComboboxStatus.tsx:26-37`). The initial-announcement marker — a WORD JOINER appended to the last text node and removed after 200ms, skipped on iOS — is implemented once in `useInitialLiveRegionTextMutation` (`packages/react/src/combobox/utils/useInitialLiveRegionTextMutation.ts:6-9,29-62`). `Empty` also shares its ref with the root's `emptyRef`, which gates the Escape-bubbling branch when no items match (`packages/react/src/combobox/root/AriaCombobox.tsx:756-766`).

### Inline chips/clear vs dismissal

Chips and Clear render outside the portal but count as "inside": the root's `outsidePress` predicate exempts `chipsContainerRef`, `clearRef`, the trigger, and the input group (`packages/react/src/combobox/root/AriaCombobox.tsx:1344-1352`). The chips container registers `chipsContainerRef`, takes `role="toolbar"` for NVDA browse-mode, and forwards presses to `handleInputPress`, which focuses the input and optionally opens with reason `inputPress` while ignoring interactive child targets (`packages/react/src/combobox/chips/ComboboxChips.tsx:38-51`; `packages/react/src/combobox/utils/handleInputPress.ts:8-38`); `InputGroup` does the same with a chips-container ignore predicate (`packages/react/src/combobox/input-group/ComboboxInputGroup.tsx:55-70`). Chip removal funnels into `setSelectedValue` with reason `chipRemovePress`, respects `details.isPropagationAllowed`, and clears a matching active item highlight (`packages/react/src/combobox/chip-remove/ComboboxChipRemove.tsx:74-113`); Clear follows the shared contract with reasons `clearPress` and explicit index clearing (`packages/react/src/combobox/clear/ComboboxClear.tsx:99-124`).

## Dependencies on other Base UI internals

Nothing is delegated to an external package; the unit does import from these first-party locations and two npm packages:

### `@base_ui/utils` (workspace `packages/utils`)

- `useControlled` — the three controlled states (`packages/react/src/combobox/root/AriaCombobox.tsx:3`).
- `useIsoLayoutEffect` — all layout-phase syncs in root/popup/item/group-label/positioner (`packages/react/src/combobox/root/AriaCombobox.tsx:4`).
- `useOnFirstRender` — pre-render prop-bag seeding (`packages/react/src/combobox/root/AriaCombobox.tsx:5`).
- `useStableCallback` — every command/callback closure in root, input, trigger, positioner, list, input-group (`packages/react/src/combobox/root/AriaCombobox.tsx:6`).
- `useMergedRefs` — hidden input ref (`packages/react/src/combobox/root/AriaCombobox.tsx:7`) and dismiss button (`packages/react/src/combobox/utils/ComboboxInternalDismissButton.tsx:3`).
- `useValueAsRef` — stable trigger ref (`packages/react/src/combobox/root/AriaCombobox.tsx:8`).
- `visuallyHidden`/`visuallyHiddenInput` — hidden control and dismiss-button styling (`packages/react/src/combobox/root/AriaCombobox.tsx:9`).
- `useRefWithInit` — store creation and initial input value (`packages/react/src/combobox/root/AriaCombobox.tsx:10`).
- `ReactStore` (`@base_ui/utils/store`) — the store engine (`packages/react/src/combobox/store.ts:1`).
- `EMPTY_ARRAY`/`EMPTY_OBJECT` (`@base_ui/utils/empty`) — stable defaults (`packages/react/src/combobox/root/AriaCombobox.tsx:12`).
- `platform` — Android/Gecko/iOS platform branches in Input and the live-region hook (`packages/react/src/combobox/input/ComboboxInput.tsx:4`; `packages/react/src/combobox/utils/useInitialLiveRegionTextMutation.ts:3`).
- `useTimeout` — trigger focus timing and marker reset (`packages/react/src/combobox/trigger/ComboboxTrigger.tsx:4`).
- `ownerDocument` — trigger mouseup listener (`packages/react/src/combobox/trigger/ComboboxTrigger.tsx:5`).
- `inertValue` — modal backdrop inertness (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:5`).
- `InteractionType` type from `useEnhancedClickHandler` (`packages/react/src/combobox/store.ts:2`; `packages/react/src/combobox/popup/ComboboxPopup.tsx:3`).
- `error` — dev warnings for duplicate `createItems` values and misused `Combobox.Label` (`packages/react/src/combobox/items/createItems.ts:2`; `packages/react/src/combobox/label/ComboboxLabel.tsx:3`).
- `SafeReact` — `captureOwnerStack` in the Label warning (`packages/react/src/combobox/label/ComboboxLabel.tsx:4`).

### npm: `@floating-ui/utils/dom`

- `isHTMLElement` in the root's scroll-reset guard (`packages/react/src/combobox/root/AriaCombobox.tsx:13`); `isElement` in `handleInputPress` (`packages/react/src/combobox/utils/handleInputPress.ts:1`).

### `packages/react/src/floating-ui-react` (vendored floating-ui)

- Root: `ElementProps`, `getOverflowAncestors`, `useDismiss`, `useFloatingRootContext`, `useListNavigation`, `useClick` (`packages/react/src/combobox/root/AriaCombobox.tsx:14-21`); `gridNavigation` (`:22`); `contains`/`getTarget` utils (`:23`).
- Trigger: `useTypeahead`, `useClick`, `stopEvent`, `contains`, `getTarget` (`packages/react/src/combobox/trigger/ComboboxTrigger.tsx:17,22`).
- Popup: `FloatingFocusManager` (`packages/react/src/combobox/popup/ComboboxPopup.tsx:5`).
- Portal: `FloatingPortal` (`packages/react/src/combobox/portal/ComboboxPortal.tsx:3`).
- List: `stopEvent` (`packages/react/src/combobox/list/ComboboxList.tsx:14`); Input: `stopEvent` (`packages/react/src/combobox/input/ComboboxInput.tsx:19`); Chip/ChipRemove: `stopEvent` (`packages/react/src/combobox/chip/ComboboxChip.tsx:10`; `packages/react/src/combobox/chip-remove/ComboboxChipRemove.tsx:8`); InputGroup: `contains` (`packages/react/src/combobox/input-group/ComboboxInputGroup.tsx:13`).

### `packages/react/src/internals/*`

- `createBaseUIEventDetails` + `REASONS` — the shared details/cancel contract and all reason strings (root, input, trigger, chips, chip-remove, clear, dismiss button: `packages/react/src/combobox/root/AriaCombobox.tsx:24-30`; `packages/react/src/combobox/chip/ComboboxChip.tsx:11-12`; `packages/react/src/combobox/clear/ComboboxClear.tsx:12-13`).
- `useOpenChangeComplete` — root unmount, popup open-complete, Clear/ItemIndicator exit-complete (`packages/react/src/combobox/root/AriaCombobox.tsx:39`).
- `FieldRootContext` / `useFieldRootContext` — Field state, validation props, dirty/touched/focused (root, input, trigger, clear, input-group, label; `packages/react/src/combobox/root/AriaCombobox.tsx:40`).
- `field-constants` — `DEFAULT_FIELD_STATE_ATTRIBUTES`, `fieldValidityMapping` (`packages/react/src/combobox/input/ComboboxInput.tsx:16`; `packages/react/src/combobox/utils/stateAttributesMapping.ts:4`).
- `useRegisterFieldControl` — form-control registration (root: `packages/react/src/combobox/root/AriaCombobox.tsx:41`).
- `FormContext`/`useFormContext` — `clearErrors` (root: `packages/react/src/combobox/root/AriaCombobox.tsx:42`).
- `labelable-provider` — `useLabelableId` (root, trigger), `LabelableContext` (input), `useLabel` (label: `packages/react/src/combobox/root/AriaCombobox.tsx:43`; `packages/react/src/combobox/label/ComboboxLabel.tsx:10`).
- `useTransitionStatus` + `TransitionStatusDataAttributes` mapping — root, popup state, clear, item indicator (`packages/react/src/combobox/root/AriaCombobox.tsx:46`).
- `types` (`BaseUIComponentProps`, `HTMLProps`, `NativeButtonProps`, `NonNativeButtonProps`) — all parts' prop types (`packages/react/src/combobox/root/AriaCombobox.tsx:49`).
- `useValueChanged` — delta watchers (root: `packages/react/src/combobox/root/AriaCombobox.tsx:50`).
- `noop` (`NOOP`) — command seeding (`packages/react/src/combobox/root/AriaCombobox.tsx:51`).
- `resolveValueLabel` — `stringifyAsLabel`, `stringifyAsValue`, `Group`, `flattenLeafItems`, `isGroupedItems`, `resolveSelectedLabel`, `resolveMultipleLabels`, `hasNullItemLabel` (root, value, items: `packages/react/src/combobox/root/AriaCombobox.tsx:54-60`).
- `itemEquality` — `compareItemEquality`, `defaultItemEquality`, `findItemIndex`, `findSelectionIndex`, `isSelectedValueDirty`, `removeItem`, `selectedValueIncludes`, `resolveSelectedIndex` (root, item, chip-remove, itemCollection: `packages/react/src/combobox/root/AriaCombobox.tsx:61-69`).
- `direction-context`/`useDirection` — RTL for grid navigation, chip keys, input caret (root, input, chip: `packages/react/src/combobox/root/AriaCombobox.tsx:71`).
- `useAnchorPositioning` — positioner positioning + Side/Align types (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:9-14`).
- `useRenderElement` — every rendered part's element factory (input, trigger, popup, positioner, list, item, chips, chip, chip-remove, clear, input-group, label, icon, arrow, backdrop, empty, status, group, group-label, row, item-indicator, separator consumers).
- `useBaseUiId` — input and group-label ids (`packages/react/src/combobox/input/ComboboxInput.tsx:6`).
- `use-button` — trigger, item, chip-remove, clear, dismiss-button button semantics (`packages/react/src/combobox/trigger/ComboboxTrigger.tsx:8`).
- `composite/list` — `CompositeList` (list, chips) and `useCompositeListItem` (item, chip) (`packages/react/src/combobox/list/ComboboxList.tsx:13`; `packages/react/src/combobox/item/ComboboxItem.tsx:10`).
- `getStateAttributesProps` (`StateAttributesMapping` type) and `stateAttributesMapping` (`transitionStatusMapping`) — popup/clear/backdrop/item-indicator attribute mapping (`packages/react/src/combobox/popup/ComboboxPopup.tsx:14-15`).
- `getDisabledMountTransitionStyles` — popup mount-transition suppression (`packages/react/src/combobox/popup/ComboboxPopup.tsx:17`).
- `filter` (`getFilter`, `Filter` types) — collator core behind `useFilter` (`packages/react/src/combobox/root/utils/useFilter.ts:4-8`).

### `packages/react/src/utils/*`

- `useOpenInteractionType` — openMethod/triggerProps (root: `packages/react/src/combobox/root/AriaCombobox.tsx:47`).
- `scrollable` (`isScrollableY`) — query-change list reset (root: `packages/react/src/combobox/root/AriaCombobox.tsx:48`).
- `popups` (`FOCUSABLE_POPUP_PROPS`) — popup prop defaults (root: `packages/react/src/combobox/root/AriaCombobox.tsx:52`).
- `popupStateMapping` — `popupStateMapping`, `pressableTriggerOpenStateMapping`, `triggerOpenStateMapping` for clear/backdrop/arrow/input state attributes (`packages/react/src/combobox/clear/ComboboxClear.tsx:14`; `packages/react/src/combobox/utils/stateAttributesMapping.ts:1`).
- `usePositioner` — positioner element rendering helper (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:19`).
- `useAnchoredPopupScrollLock` — modal scroll lock (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:20`).
- `InternalBackdrop` — modal inert backdrop (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx:18`).
- `getPseudoElementBounds` (`isMouseWithinBounds`) — trigger drag-away cancel detection (`packages/react/src/combobox/trigger/ComboboxTrigger.tsx:18`).
- `resolveAriaLabelledBy` (`resolveAriaLabelledBy`, `getDefaultLabelId`) — trigger/label ARIA labelling (`packages/react/src/combobox/trigger/ComboboxTrigger.tsx:25`; `packages/react/src/combobox/label/ComboboxLabel.tsx:11`).
- `listbox-separator` — `ComboboxSeparator` is a re-export of `ListboxSeparator` with no own element logic (`packages/react/src/combobox/separator/ComboboxSeparator.tsx:3,30-32`).

### `packages/react/src/merge-props`

- `mergeProps` — all prop-bag composition (root: `packages/react/src/combobox/root/AriaCombobox.tsx:53`).

### Sibling components (`@base_ui/react` or relative component dirs)

- No runtime imports of sibling components. The only cross-component references are type-only: `FieldRootState` from `../../field/root/FieldRoot` in Input, Trigger, and Label state types (`packages/react/src/combobox/input/ComboboxInput.tsx:10`; `packages/react/src/combobox/trigger/ComboboxTrigger.tsx:19`; `packages/react/src/combobox/label/ComboboxLabel.tsx:7`).
- Reverse dependency (outside the unit, importing into it): `packages/react/src/autocomplete/root/AutocompleteRoot.tsx:3-4` imports `AriaCombobox` and `useCoreFilter` and renders the headless core with `selectionMode="none"` (`packages/react/src/autocomplete/root/AutocompleteRoot.tsx:105-122`).

### Not used

No imports of `use-render`/`use-form` npm packages (element rendering is `internals/useRenderElement`; form integration is `internals/form-context` + `useRegisterFieldControl`), and no imports of Popover/Dialog/Select/Menu component code.

## Anything in source not explained by any test

Gaps and speculation flags for the fixture/audit stage:

1. **`ComboboxPositionerCssVars.ts` is dead code.** Its five exported CSS-var name constants (`packages/react/src/combobox/positioner/ComboboxPositionerCssVars.ts:5-25`) are referenced nowhere in `packages/` or `docs/` (grep confirms zero importers). The real `--available-height` seeding happens generically inside `packages/react/src/internals/useAnchorPositioning.ts:353-360` via `CommonPositionerCssVars`. Nothing in behavior.md corresponds to this file.
2. **The headless core is shared with Autocomplete.** `AriaCombobox` is rendered directly by `packages/react/src/autocomplete/root/AutocompleteRoot.tsx:105-122`, which exercises props that the combobox test suite (and therefore behavior.md) never covers: `filterQuery` (inline-completion query override, `packages/react/src/combobox/root/AriaCombobox.tsx:83,350`), `fillInputOnItemPress` (`packages/react/src/combobox/root/AriaCombobox.tsx:135,843-852`), `submitOnItemClick` (`packages/react/src/combobox/root/AriaCombobox.tsx:141`; flushSync+`requestSubmit` path `packages/react/src/combobox/item/ComboboxItem.tsx:148-159`), and `keepHighlight` → `resetOnPointerLeave` (`packages/react/src/combobox/root/AriaCombobox.tsx:127,1367`). behavior.md's store-utils part only mentions Autocomplete because the store tests are shared. Fixture work targeting `selectionMode: 'none'` semantics must account for this second consumer.
3. **The Input inside the popup also carries combobox ARIA.** The `role.reference` bag is merged into `inputProps` unconditionally, so in the input-inside-popup composition both the promoted Trigger and the popup's Input carry `role="combobox"` (the input additionally carries `aria-activedescendant`; the trigger carries `aria-haspopup="dialog"`). behavior.md documents only the trigger promotion; the double-role structure of the implemented DOM is not stated anywhere.
4. **Closed-state Escape clears the selection.** `ComboboxInput`'s keydown has a `!mounted` branch that clears inputValue and the selected value on Escape when the popup is closed, with conditional `stopPropagation` (`packages/react/src/combobox/input/ComboboxInput.tsx:375-391`). behavior.md documents Escape during open/close animation ("discards an uncommitted query"), not this cleared-selection-while-closed path.
5. **Inline highlight restore on refocus.** Input blur in inline mode stashes `lastActiveIndexRef` and refocus restores the highlight (guarding sparse `valuesRef` slots with `Object.hasOwn`) (`packages/react/src/combobox/input/ComboboxInput.tsx:208-237`). No behavior section covers highlight persistence across inline blur/refocus.
6. **Render-phase state reset in Chips.** `ComboboxChips` resets `highlightedChipIndex` to `undefined` during render whenever the popup is open (`packages/react/src/combobox/chips/ComboboxChips.tsx:32-34`) — an unusual render-phase setState that pairs with the "chip focus vs popup open are mutually exclusive" behavior but whose mechanism (and safety) is undocumented.
7. **Null-item placeholder suppression in `Value`.** `ComboboxValue` suppresses the `placeholder` when the `items` prop contains a null-valued item, via the `hasNullItemLabel` selector (`packages/react/src/combobox/store.ts:145-147`; `packages/react/src/combobox/value/ComboboxValue.tsx:23-31`). The prop JSDoc mentions it; no behavior section does.
8. **Escape bubbling gated on missing `Empty`.** When the filtered list is empty, `items` is set, and no `Empty` is rendered, `setOpen` marks the Escape as propagation-allowed so a parent popup can also close (`packages/react/src/combobox/root/AriaCombobox.tsx:756-766`). behavior.md's nesting edge case covers nested content not dismissing, not this inverse branch.
9. **Autofill double force-mount.** Besides `forceMount()`, the hidden input's autofill handler sets the sticky `forceMounted` store flag when serialized matching misses under the `items` prop, so rendered labels get registered for matching (`packages/react/src/combobox/root/AriaCombobox.tsx:1613-1621`). The behavior spec documents autofill matching generally, not this two-tier mount mechanism (nor that the flag never resets).
10. **Inline-in-Dialog transition observation.** The root observes close-completion on the closest `role="dialog"` ancestor when `inline` (an explicitly documented interop hack with a comment acknowledging third-party modal libraries) (`packages/react/src/combobox/root/AriaCombobox.tsx:972-981`). behavior.md mentions Dialog composition but not this `[role="dialog"]` string-matching mechanism or its "closest animated element" limitation.
11. **Default decorative children.** `Icon` renders `'▼'` (`packages/react/src/combobox/icon/ComboboxIcon.tsx:23`), `ItemIndicator` renders `'✔️'` (`packages/react/src/combobox/item-indicator/ComboboxItemIndicator.tsx:57`), and `Clear` renders `'x'` (`packages/react/src/combobox/clear/ComboboxClear.tsx:94`) as default children — cosmetic constants with no behavior coverage; fixtures must not snapshot them accidentally.
12. **Type-contract-only spec file.** `packages/react/src/combobox/root/ComboboxRoot.spec.tsx` is a compile-time contract suite (collection variance, filter receiving source items for `createItems` collections, `multiple` lifting, ts-expect-error guards) with no runtime assertions; the compile-time group-shape rejection in `createItems` (`packages/react/src/combobox/items/createItems.ts:21-48`) likewise has no runtime counterpart beyond the collection-shape throw (`packages/react/src/combobox/root/AriaCombobox.tsx:166-173`).
13. **`Store.update` key-iteration subtlety in Clear.** Clear passes an explicit object (not `undefined` fields) to avoid `setIndices` overwriting `selectedIndex` — documented only in an inline comment (`packages/react/src/combobox/clear/ComboboxClear.tsx:116-118`). Fixture expectations about `selectedIndex` after clear depend on this.
14. **Grid caret preservation.** With `grid` and no highlighted item, ArrowLeft/ArrowRight are stripped from floating-ui's handler via `preventBaseUIHandler` so the input keeps native caret movement (`packages/react/src/combobox/root/AriaCombobox.tsx:1396-1408`). behavior.md documents grid row/column navigation but not this editing-mode carve-out.
