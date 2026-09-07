# Select — implementation spec

Companion to [behavior.md](./behavior.md) (WHAT); this document explains WHY/HOW from the source
files under `packages/react/src/select/`. The TODO.md entry for `library: select` has no
`wraps-external:` field (`TODO.md:498-504`), so there is no external-package delegation to
account for — everything below is derived from this repo's own source.

The spine of the implementation is a single external store, not `useState`: the Root owns two
`useControlled` pairs, mirrors them plus ~20 derived fields into a `ReactStore`, and every other
part is a thin selector-subscriber over that store. React context carries only the store itself,
a render-phase prop bag, and the floating-ui root context.

## State machine / hooks used

### Root store (the state machine)

- **Controlled pairs.** `value` and `open` are each handled by `useControlled`
  (`packages/react/src/select/root/SelectRoot.tsx:107-119`); in multiple mode the uncontrolled
  default coerces to an array (`packages/react/src/select/root/SelectRoot.tsx:109`).
- **The store.** A `ReactStore<State, SelectStoreContext, typeof selectors>` is created once via
  `useRefWithInit` (`packages/react/src/select/root/SelectRoot.tsx:142-193`). State fields are
  enumerated in `packages/react/src/select/store.ts:10-47`; per-item selectors (`isActive`,
  `isSelected`, `isSelectedByFocus`, `hasSelectedValue`, `hasNullItemLabel`) live in
  `packages/react/src/select/store.ts:91-132`. `isSelected` deliberately compares against
  `state.value` rather than `selectedIndex` so a stale index can't mark the wrong item selected
  while the popup is open (`packages/react/src/select/store.ts:128-131`). Refs and command
  callbacks are *non-reactive* `context` members (`packages/react/src/select/store.ts:53-78`) —
  writes to them never notify subscribers.
- **Command registration.** `setValue`, `setOpen`, `handleScrollArrowVisibility`, and
  `onOpenChangeComplete` are seeded as `NOOP` in the constructor and re-assigned every render by
  `store.useContextCallback`, which wraps each in `useStableCallback`
  (`packages/react/src/select/root/SelectRoot.tsx:430-433`,
  `packages/utils/src/store/ReactStore.ts:188-195`). This is how descendants commit values or
  close the popup without prop drilling.
- **Prop-bag propagation.** The floating hooks' merged `triggerProps`/`popupProps` are pushed
  into the store by `useOnFirstRender` before the parts' first render, then kept in sync by
  `store.useSyncedValues` (a layout-effect `store.update`) — both are required because the parts
  read these bags through selectors, not context
  (`packages/react/src/select/root/SelectRoot.tsx:437-459`,
  `packages/utils/src/store/ReactStore.ts:90-114`). `useSyncedValues` also syncs `id`, `modal`,
  `multiple`, `value`, `open`, `mounted`, `transitionStatus`, `items`, the stringifier props, and
  `openMethod` (`packages/react/src/select/root/SelectRoot.tsx:444-459`).
- **Mount/transition machine.** `useTransitionStatus(open)` produces `mounted` +
  `transitionStatus` (`packages/react/src/select/root/SelectRoot.tsx:139`,
  `packages/react/src/internals/useTransitionStatus.ts:18-26`); `mounted` stays true through the
  exit transition, and `useOpenChangeComplete` fires `handleUnmount` when `open` settles false
  (`packages/react/src/select/root/SelectRoot.tsx:301-310`). With `actionsRef` present,
  automatic unmount is disabled and unmounting is handed to the consumer through
  `React.useImperativeHandle` exposing `unmount()`
  (`packages/react/src/select/root/SelectRoot.tsx:290-299,312`).
- **Open-method retention.** `useOpenInteractionType` records keyboard/mouse/touch; the rendered
  method falls back to `usePreviousValue(openMethod)` so the closing transition still knows how
  the popup was opened (`packages/react/src/select/root/SelectRoot.tsx:140,200-201`).
- **Selected-index sync.** A layout effect recomputes `selectedIndex` from `valuesRef` via
  `findSelectionIndex` whenever value/items equality change, but defers the store write while
  `open` so a controlled change mid-popup doesn't fight the item-claim logic
  (`packages/react/src/select/root/SelectRoot.tsx:242-257`). Items claim the index themselves in
  a layout effect (`packages/react/src/select/item/SelectItem.tsx:75-110`).
- **Value side effects.** `useValueChanged` runs `clearErrors`, Field `setDirty` (via
  `isSelectedValueDirty`), and validation `change` only when the value actually changes
  (`packages/react/src/select/root/SelectRoot.tsx:259-264`).
- **Cancel-gating.** Both `setOpen` and `setValue` invoke the user handler first and bail if
  `eventDetails.isCanceled` — the single mechanism behind the `cancel()` behaviors documented in
  behavior.md's "State model" section
  (`packages/react/src/select/root/SelectRoot.tsx:266-288,314-324`).

### Floating-ui hook composition (all in Root)

`useFloatingRootContext` wires trigger/positioner elements with `open`/`setOpen`
(`packages/react/src/select/root/SelectRoot.tsx:336-343`); `useClick` with `event: 'mousedown'`
opens (`packages/react/src/select/root/SelectRoot.tsx:348-351`), `useDismiss` closes
(`packages/react/src/select/root/SelectRoot.tsx:353`), `useListNavigation` drives ArrowUp/Down
highlight with `focusItemOnHover` tied to `highlightItemOnHover` and a guard that keeps the
highlight during the close transition (`packages/react/src/select/root/SelectRoot.tsx:355-370`),
and `useTypeahead` matches against `labelsRef`, committing directly on closed-trigger matches and
moving the highlight on open-popup matches
(`packages/react/src/select/root/SelectRoot.tsx:372-395`). The reference/floating prop bags are
memoized with `mergeProps` (`packages/react/src/select/root/SelectRoot.tsx:398-425`) and
`listNavigation.item` becomes the shared `itemProps` bag distributed through
`SelectRootPropsContext` (`packages/react/src/select/root/SelectRoot.tsx:427-428`).

### Per-part hooks

- **Trigger**
  (`packages/react/src/select/trigger/SelectTrigger.tsx`): `useButton` for disabled semantics
  (`packages/react/src/select/trigger/SelectTrigger.tsx:90-93`); three `useTimeout`s — focus,
  mouseup-listener registration, and the 400 ms `SELECTED_DELAY` that arms
  `selectionRef.allow*MouseUp` so the opening click's mouseup can't commit an accidental
  selection (`packages/react/src/select/trigger/SelectTrigger.tsx:26,97-99,101-126`). `onFocus`
  closes an item-aligned popup (the popup overlaps the focused trigger) and pre-mounts items via
  a `forceMount` timeout (`packages/react/src/select/trigger/SelectTrigger.tsx:146-155`);
  `onBlur` skips Field touched/commit when focus moves into the positioner
  (`packages/react/src/select/trigger/SelectTrigger.tsx:157-169`); `onMouseDown` registers a
  once-only document mouseup that closes with reason `cancelOpen` when the release lands outside
  trigger and positioner (`packages/react/src/select/trigger/SelectTrigger.tsx:170-204`).
- **Item** (`packages/react/src/select/item/SelectItem.tsx`): `React.memo`-wrapped;
  `useCompositeListItem({ guess: true, label, textRef })` registers into `listRef`/`labelsRef`
  (`packages/react/src/select/item/SelectItem.tsx:47-51`); `useButton` with
  `focusableWhenDisabled: true, composite: true`
  (`packages/react/src/select/item/SelectItem.tsx:115-120`). The mouse/touch/keyboard gating
  machine is three local refs (`pointerTypeRef`, `allowMouseSelectionRef`, and the shared
  `selectionRef.dragY` accumulator) (`packages/react/src/select/item/SelectItem.tsx:112-113`)
  plus the shared `selectionRef` writes in the pointer handlers
  (`packages/react/src/select/item/SelectItem.tsx:192-231`) — this is the implementation of the
  pointer-event rules documented in behavior.md's "Events" section.
- **Popup** (`packages/react/src/select/popup/SelectPopup.tsx`): `useAnimationFrame` schedules
  scroll-arrow recompute (`packages/react/src/select/popup/SelectPopup.tsx:85`); the shared
  `scrollHandlerRef` is filled via `useImperativeHandle` so both the popup's own `onScroll` and
  `List`'s drive the same handler
  (`packages/react/src/select/popup/SelectPopup.tsx:178,453-458`);
  `useOpenChangeComplete` forwards `onOpenChangeComplete(true)` on the open transition
  (`packages/react/src/select/popup/SelectPopup.tsx:180-188`); a window `resize` listener closes
  item-aligned popups because their geometry is measured once
  (`packages/react/src/select/popup/SelectPopup.tsx:423-435`).
- **Positioner**
  (`packages/react/src/select/positioner/SelectPositioner.tsx`):
  `controlledAlignItemWithTrigger` is React state seeded from the prop and reset while unmounted —
  this is how the popup can *demote itself* to standard anchoring mid-open
  (`packages/react/src/select/positioner/SelectPositioner.tsx:75-82`, set by
  `packages/react/src/select/popup/SelectPopup.tsx:362`). `alignItemWithTriggerActive` also
  requires `openMethod !== 'touch'`
  (`packages/react/src/select/positioner/SelectPositioner.tsx:77-78`). `useAnchorPositioning` is
  called with `keepMounted: true` so the positioner element always exists for measurement
  (`packages/react/src/select/positioner/SelectPositioner.tsx:96-112`). `CompositeList.onMapChange`
  is the item-set reconciliation hook
  (`packages/react/src/select/positioner/SelectPositioner.tsx:141-207`).
- **Scroll arrows**
  (`packages/react/src/select/scroll-arrow/SelectScrollArrow.tsx`): one shared component
  parameterized by `direction`; `useTransitionStatus(visible)` + `useOpenChangeComplete` give the
  arrows their own enter/exit animation lifecycle
  (`packages/react/src/select/scroll-arrow/SelectScrollArrow.tsx:45-70`); a `useTimeout`
  self-reschedules a 40 ms `scrollNextItem` loop while hovered
  (`packages/react/src/select/scroll-arrow/SelectScrollArrow.tsx:41,127-130`); a ref-counting
  layout effect maintains `hasScrollArrows` in the store
  (`packages/react/src/select/scroll-arrow/SelectScrollArrow.tsx:47-60`).
- **ItemIndicator**
  (`packages/react/src/select/item-indicator/SelectItemIndicator.tsx`): split into a gate
  component that returns `null` unless `keepMounted || selected` and a memoized `Inner`, so
  unselected indicators pay no hook cost
  (`packages/react/src/select/item-indicator/SelectItemIndicator.tsx:22-25,32-44`); its
  transition lifecycle is independent via `useTransitionStatus(selected)` and
  `useOpenChangeComplete`
  (`packages/react/src/select/item-indicator/SelectItemIndicator.tsx:44,64-74`).
- **Group/GroupLabel**: the group owns `labelId` state and exposes `setLabelId` through
  `SelectGroupContext`; `GroupLabel` registers its id in a layout effect and the cleanup only
  clears the id if it still owns it — this produces the newest-label-wins registration semantics
  documented in behavior.md's accessibility section
  (`packages/react/src/select/group/SelectGroup.tsx:19-27`,
  `packages/react/src/select/group-label/SelectGroupLabel.tsx:25-30`).

## Context providers/consumers

| Context | Provided by | Consumed by | What crosses |
| --- | --- | --- | --- |
| `SelectRootContext` | Root (`packages/react/src/select/root/SelectRoot.tsx:498`) | every part via `useSelectRootContext` | the store itself |
| `SelectRootPropsContext` | Root (`packages/react/src/select/root/SelectRoot.tsx:499`) | Trigger, Item, Popup, List via `useSelectRootPropsContext` | `disabled`, `readOnly`, `required`, `multiple`, `highlightItemOnHover`, shared `itemProps` |
| `SelectFloatingContext` | Root (`packages/react/src/select/root/SelectRoot.tsx:500`) | Positioner, Popup | the `FloatingRootContext` |
| `SelectPositionerContext` | Positioner (`packages/react/src/select/positioner/SelectPositioner.tsx:209-227`) | Popup, List, Arrow, scroll arrows | resolved positioning output, `alignItemWithTriggerActive`, its setter, scroll-arrow refs |
| `SelectItemContext` | Item (`packages/react/src/select/item/SelectItem.tsx:240-250`) | ItemText, ItemIndicator | `selected`, `index`, `textRef`, `selectedByFocus` |
| `SelectGroupContext` | Group (`packages/react/src/select/group/SelectGroup.tsx:40`) | GroupLabel | `labelId`, `setLabelId` |

The prop bag is deliberately a *render-phase* context rather than a store sync: descendant ref
callbacks must see current props during the same commit
(`packages/react/src/select/root/SelectRootContext.ts:7-10`). Shared `itemProps` (from
`useListNavigation.item`) is distributed this way so every Item gets identical navigation
behavior without Root re-rendering through the store per item.

Cross-package contexts consumed: `useFieldRootContext` (Root, Trigger, Label),
`useLabelableContext`/`useLabelableId` (Root, Trigger), `useFormContext` (Root),
`useRegisterFieldControl` (Root), `useToolbarRootContext(true)` (Popup, optional — only for
key-swallowing), `useCSPContext` (Popup, for the style-element nonce), `useDirection` (Popup, for
rtl alignment math).

## DOM/portal strategy and why

- **Root renders no element.** It renders the three contexts, a single visually-hidden `<input>`
  for form association/autofill, and one extra hidden input per selected value in multiple mode
  (`packages/react/src/select/root/SelectRoot.tsx:497-573`). The shared input is nameless in
  multiple mode — its value is irrelevant because per-value entries are submitted by
  `hiddenInputs` (`packages/react/src/select/root/SelectRoot.tsx:203-211,475-495`). The input's
  `onFocus` forwards to the trigger (with `focusVisible: true`, Chrome 144+), and `onChange` is
  the autofill entry point: it sets `store.forceMount` and defers matching to a microtask so the
  item DOM exists before `valuesRef`/`labelsRef` are read
  (`packages/react/src/select/root/SelectRoot.tsx:506-556`).
- **Portal.** `SelectPortal` renders `FloatingPortal` only when `mounted || forceMount`
  (`packages/react/src/select/portal/SelectPortal.tsx:20-27`). `forceMount` is the "pre-mount"
  channel: trigger focus and autofill mount the item tree before `open`, which is what makes
  closed-trigger typeahead, autofill matching, and first-open aligned measurement possible
  (`packages/react/src/select/trigger/SelectTrigger.tsx:150-155`,
  `packages/react/src/select/root/SelectRoot.tsx:554`).
- **Positioner always exists.** `useAnchorPositioning` runs with `keepMounted: true`; when not
  `mounted` the element is rendered `hidden` and `inert` through the shared `usePositioner`
  helper (`packages/react/src/select/positioner/SelectPositioner.tsx:111,130-137`,
  `packages/react/src/utils/usePositioner.tsx:23-43`). This is why a kept-mounted positioner can
  be "parked" after close rather than unmounted (behavior.md, "DOM structure & portal behavior").
- **Item-aligned mode is inline-style, not floating-ui output.** When active, the positioner gets
  `position: fixed` (`packages/react/src/select/positioner/SelectPositioner.tsx:25,114-115`) and
  the popup's layout effect rewrites geometry directly: `left` clamped to viewport padding,
  explicit `height`, `maxHeight: 'none'`, `marginTop/Bottom`, popup `height: '100%'`,
  `scrollTop` placement, and a computed `--transform-origin` from the selected item-text rect
  (`packages/react/src/select/popup/SelectPopup.tsx:330-390`,
  `packages/react/src/select/positioner/SelectPositionerCssVars.ts:25`). It aligns the selected
  item's `ItemText` rect with the trigger's `Value` rect
  (`packages/react/src/select/popup/SelectPopup.tsx:305-321`); with no selection it falls back to
  the first item's text (`packages/react/src/select/popup/SelectPopup.tsx:263-271`). Demotion to
  standard anchoring happens near viewport edges, below `minHeight`, or under Safari pinch-zoom
  (`packages/react/src/select/popup/SelectPopup.tsx:352-364`). All style mutations are
  bracketed by saving/clearing originals via `clearStyles`
  (`packages/react/src/select/popup/SelectPopup.tsx:197-227`,
  `packages/react/src/select/popup/utils.ts:1-5`), and transforms are temporarily forced off
  with `!important` while measuring (`packages/react/src/select/popup/SelectPopup.tsx:567-594`).
- **Dual scroller ownership.** Without an explicit `List`, the popup itself is the listbox
  (`role="listbox"`, `${id}-list`, `aria-multiselectable`, `aria-readonly`) and owns the
  `onScroll` handler; with a `List`, the popup demotes to `role="presentation"` and the List
  takes the role/id/scroll duties (`packages/react/src/select/popup/SelectPopup.tsx:437-464`,
  `packages/react/src/select/list/SelectList.tsx:30-45`). The scrollbar-hiding style element
  (nonce-aware via CSP context) is emitted by whichever element scrolls
  (`packages/react/src/select/popup/SelectPopup.tsx:462-463,480`,
  `packages/react/src/select/list/SelectList.tsx:41-42`).
- **Modal backdrop.** Modal mode renders an internal `InternalBackdrop` (with the trigger as
  `cutout`) as the positioner's sibling — this is the backdrop behavior.md describes, and it is
  distinct from the public `Select.Backdrop`, which is purely a user-composable styled div
  (`packages/react/src/select/positioner/SelectPositioner.tsx:228`,
  `packages/react/src/select/backdrop/SelectBackdrop.tsx:39-56`). Scroll locking (including the
  touch full-width exception) lives in `useAnchoredPopupScrollLock`
  (`packages/react/src/select/positioner/SelectPositioner.tsx:89-94`,
  `packages/react/src/utils/useAnchoredPopupScrollLock.ts:28-44`).
- **Type-only spec files.** `SelectRoot.spec.tsx` and `SelectPositioner.spec.tsx` are
  compile-time API assertions (e.g. `keepMounted` must not exist on Positioner), not runtime
  tests (`packages/react/src/select/root/SelectRoot.spec.tsx:28-45`,
  `packages/react/src/select/positioner/SelectPositioner.spec.tsx:3-4`).

## Dependencies on other Base UI internals

Everything select imports, by area (this is the per-component dependency record):

- **`floating-ui-react`** (vendored at `packages/react/src/floating-ui-react/`, a Floating UI
  fork): `useFloatingRootContext`, `useClick`, `useDismiss`, `useListNavigation`, `useTypeahead`
  (Root); `FloatingFocusManager` (Popup — `modal={false}`, `returnFocus={finalFocus}`,
  `openInteractionType={openMethod}`); `FloatingPortal` (Portal); utils `contains`,
  `getFloatingFocusElement` (Trigger), `isVirtualClick` (Item), `platform` (Popup).
- **`internals/`**: `useRenderElement` (every part); `useAnchorPositioning` (Positioner);
  `composite/list` (`CompositeList` in Positioner, `useCompositeListItem` in Item);
  `useTransitionStatus` (Root, ItemIndicator, scroll arrows); `useOpenChangeComplete` (Root,
  Popup, ItemIndicator, scroll arrows); `useValueChanged` (Root); `use-button` (Trigger, Item);
  `createBaseUIEventDetails` + `reasons` (everywhere events are raised); `itemEquality`
  (`compareItemEquality`, `findSelectionIndex`, `findItemIndex`, `removeItem`,
  `resolveSelectedIndex`, `defaultItemEquality`, `isSelectedValueDirty` — Root, Item,
  Positioner); `resolveValueLabel` (`stringifyAsValue`, `stringifyAsLabel`,
  `resolveSelectedLabel`, `resolveMultipleLabels`, `hasNullItemLabel`, `Group` — Root, Value,
  store); `stateAttributesMapping` mappings (Trigger, Popup, Backdrop, ItemIndicator, scroll
  arrows); `getDisabledMountTransitionStyles` (Popup); `field-root-context`, `form-context`,
  `labelable-provider`, `field-register-control` (Root, Trigger, Label); `useBaseUiId`
  (GroupLabel); `csp-context`, `direction-context` (Popup); `noop`, `types` (shared).
- **`packages/react/src/utils/`**: `usePositioner` (Positioner); `useAnchoredPopupScrollLock`
  (Positioner); `useOpenInteractionType` (Root); `scrollEdges`
  (`getMaxScrollOffset`, `normalizeScrollOffset`, `SCROLL_EDGE_TOLERANCE_PX` — Root, Popup,
  scroll arrows); `popupStateMapping` mappings (Trigger, Icon, Popup, Backdrop, Arrow);
  `popups.FOCUSABLE_POPUP_PROPS` (Root); `styles.styleDisableScrollbar` (Popup, List);
  `InternalBackdrop` (Positioner); `constants.DROPDOWN_COLLISION_AVOIDANCE` (Positioner);
  `getPseudoElementBounds.isMouseWithinBounds` (Trigger); `resolveAriaLabelledBy` (Trigger,
  Label); `listbox-separator.ListboxSeparator` (Separator delegates its whole implementation —
  `packages/react/src/select/separator/SelectSeparator.tsx:27-29`).
- **Sibling component context**: `toolbar/root/ToolbarRootContext` (Popup key-swallowing,
  `packages/react/src/select/popup/SelectPopup.tsx:448-452`).
- **`@base-ui/utils/`**: `useControlled`, `useIsoLayoutEffect`, `useStableCallback`,
  `useTimeout`, `useAnimationFrame`, `useValueAsRef`, `useMergedRefs`, `useRefWithInit`,
  `useOnFirstRender`, `usePreviousValue`, `ReactStore` (`store`), `visuallyHidden`,
  `inertValue`, `owner` (`ownerDocument`/`ownerWindow`), `isElementDisabled`, `clamp`,
  `platform`, `addEventListener`, `empty` (`EMPTY_ARRAY`/`EMPTY_OBJECT`),
  `useEnhancedClickHandler` (`InteractionType`), `useScrollLock` (indirect, via
  `useAnchoredPopupScrollLock`).
- **`@floating-ui/utils`**: `rectToClientRect` (Popup rect normalization).

## Anything in source not explained by any test

Per behavior.md's own UNVERIFIED markers (Home/End/PageUp keys; non-`none` open reasons), plus
gaps found only by reading the source — none of these are covered by any claim in behavior.md:

1. **`inputRef` prop on Root** (`packages/react/src/select/root/SelectRoot.tsx:473,586`) —
   merged with `validation.inputRef` onto the hidden input, but behavior.md's proven-prop list
   for Root never mentions it; no test exercises it.
2. **Trigger mouseup-outside close**
   (`packages/react/src/select/trigger/SelectTrigger.tsx:170-204`) — the reason `cancelOpen`
   close and the Firefox-timing guard (`timeoutMouseDown.start(0, …)` before adding the document
   mouseup listener) have no corresponding test; behavior.md's Events section explicitly says
   only `reason: 'none'` and `cancel()` are asserted.
3. **Hidden-input focus forwarding with `focusVisible: true`**
   (`packages/react/src/select/root/SelectRoot.tsx:506-511`) — the comment itself flags it as
   supported only from Chrome 144; nothing asserts focus-visible propagation to the trigger.
4. **Trigger role re-assertion for nested renders**
   (`packages/react/src/select/trigger/SelectTrigger.tsx:211-213`) — `props.role = 'combobox'`
   is forced after `getButtonProps` so `<Toolbar.Button render={<Select.Trigger />}>` keeps the
   combobox role; the Toolbar integration tests only cover ArrowRight swallowing.
5. **`SelectArrow` returns `null` in item-aligned mode**
   (`packages/react/src/select/arrow/SelectArrow.tsx:42-44`) — the chrome part of behavior.md is
   conformance-only and never asserts the arrow's absence under aligned positioning.
6. **`SelectLabel` strips runtime `id`**
   (`packages/react/src/select/label/SelectLabel.tsx:23-25`) — elementProps `id` is deleted so
   the label id stays derived from the root; untested.
7. **Positioner item-removal restores the *mount-time* value when possible**
   (`packages/react/src/select/root/SelectRoot.tsx:137`,
   `packages/react/src/select/positioner/SelectPositioner.tsx:158-175`) — when the selected
   item's value leaves the item set in single mode, the code first tries
   `initialValueRef.current` (captured at first render) if that value is still among remaining
   items, and only falls back to `null`. behavior.md's item-set reconciliation summary describes
   the reset as "to `null` / `defaultValue`"; the initial-value-restore branch is finer-grained
   than any documented or (visibly) tested claim.
8. **`required` suppression in multiple mode**
   (`packages/react/src/select/root/SelectRoot.tsx:475,564`) — the shared hidden input is
   `required={required && !(multiple && hasSelectedValue)}`, and the shared input drops `name`
   entirely in multiple mode; only the per-value named inputs are documented/tested.
9. **`hasScrollArrows` ↔ scrollbar-hiding class coupling**
   (`packages/react/src/select/scroll-arrow/SelectScrollArrow.tsx:47-60`,
   `packages/react/src/select/list/SelectList.tsx:41-42`) — the arrows' mount-count ref drives
   whether List gets the `styleDisableScrollbar` class (only for non-touch); scroll-arrow tests
   assert `data-visible`/scrolling behavior, not this styling contract.
10. **Focus-triggered `forceMount` pre-mount**
    (`packages/react/src/select/trigger/SelectTrigger.tsx:150-155`) — focusing the trigger
    mounts the portal tree ~one tick before `open` purely so items are measurable at first
    paint; the popup-conformance "only-after-open" assertion operates on visibility, so this
    pre-mount path is observable in the DOM but pinned by no test.

Items 2, 3, 4, 7, and 10 are the ones most likely to bite a reimplementation: they encode
browser-specific workarounds (Firefox mouseup timing, Chrome focus-visible, aligned-mode
measurement ordering) whose only spec is the source comment.
