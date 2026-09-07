# `internals` — behavior spec (mined from tests)

Unit: `infra: internals` (TODO.md has no `wraps-external:` field for this unit, so all behavior
below is mined directly from the unit's own test files). The unit is a grab-bag of headless
infrastructure: pure utilities (`RequestQueue`, `TimeoutManager`, `filter`, `getStateAttributesProps`,
`itemEquality`, `resolveValueLabel`, `stateAttributesMapping`, `composite/composite`),
temporal adapters, and React hooks (`useButton`, `useAnchorPositioning`, `useAnimationsFinished`,
`useRenderElement`, `useValueChanged`, composite list/root). Each claim below cites the test that
proves it.

## Public API surface (props, parts, subcomponents)

- `RequestQueue` — class constructed with `{ fetchFn, maxConcurrentRequests?, getKeyId? }`;
  methods exercised by tests: `queue(keys)` (returns a promise), `getRequestStatus(key)`,
  `setRequestSettled(key)`, `clearPendingRequest(key)`, `clear()`
  `packages/react/src/internals/RequestQueue.test.ts:15-38`
- `TimeoutManager` — class with `start(key, delayMs, fn)`, `clear(key)`, `clearAll()`
  `packages/react/src/internals/TimeoutManager.test.ts:13-65`
- `scrollIntoViewIfNeeded` (from `internals/composite/composite`) — called as
  `scrollIntoViewIfNeeded(scrollContainer, element, 'rtl', 'horizontal')`
  `packages/react/src/internals/composite/composite.test.ts:30-36`
- `getFilter({ locale })` — returns a filter value that is cached per `Intl.Locale` instance
  `packages/react/src/internals/filter.test.ts:5-10`
- `getStateAttributesProps(state, mapping?)` — pure function returning props; accepts an optional
  per-field custom mapping whose callbacks receive the field value and may return a props object
  or `null` `packages/react/src/internals/getStateAttributesProps.test.ts:51-82`
- `itemEquality` — exports `defaultItemEquality`, `findSelectionIndex(items, selectedValues,
  comparer, isMultiple)`, `resolveSelectedIndex(index, itemValue, registry, selectedValues,
  comparer, currentIndex)` `packages/react/src/internals/itemEquality.test.ts:2-8` and
  `packages/react/src/internals/itemEquality.test.ts:93-105`
- `resolveValueLabel` — exports `isGroupedItems`, `hasNullItemLabel`, `resolveSelectedLabel`
  `packages/react/src/internals/resolveValueLabel.test.ts:2-4`
- `stateAttributesMapping` — exports `transitionStatusMapping` (keyed by state field name);
  `field-constants/constants` exports `fieldValidityMapping`
  `packages/react/src/internals/stateAttributesMapping.test.ts:2-3`
- `TemporalAdapterDateFns` — constructed with options such as `{ locale }`; the shared harness is
  configured with `adapter.date(str, timezone)`, `setTimezone`, and getters
  `getYear`/`getMonth`/`getDate`/`getHours`
  `packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.test.ts:7-13`
- `TemporalAdapterLuxon` — constructed with `{ locale }`; supports setting a default timezone via
  Luxon `Settings.defaultZone` `packages/react/src/internals/temporal-adapter-luxon/TemporalAdapterLuxon.test.ts:9-16`
- `useButton` — params `{ native?, disabled?, focusableWhenDisabled?, tabIndex?, composite? }`;
  returns `{ getButtonProps, buttonRef }` where `getButtonProps(props?)` merges consumer props
  `packages/react/src/internals/use-button/useButton.test.tsx:24-28` and
  `packages/react/src/internals/use-button/useButton.test.tsx:120-128`
- `useAnchorPositioning` — the test imports `useAnchorPositioningWithHook(params, useFloating)`;
  params exercised: `anchor`, `mounted`, `positionMethod`, `side`, `align`, `sideOffset`,
  `alignOffset`, `collisionBoundary`, `collisionPadding`, `sticky`, `arrowPadding`,
  `disableAnchorTracking`, `keepMounted`, `collisionAvoidance`, `shift`
  `packages/react/src/internals/useAnchorPositioning.test.tsx:25-47`. Returns positioning data
  including `refs.setFloating` used to mark the floating element
  `packages/react/src/internals/useAnchorPositioning.test.tsx:49-54`
- `useAnimationsFinished(ref, disableCancelCheck?, batch?)` — returns `runOnceAnimationsFinish(
  onFinished, signal | null)` `packages/react/src/internals/useAnimationsFinished.test.tsx:36-51`
- `useRenderElement(defaultElement, componentProps, options)` — options include `state`, `ref`
  (single ref or array), `props` (object, props-getter, or array), and `enabled`; the consumer's
  `render` and `className`/`style` props are stripped from `componentProps` before passing
  `packages/react/src/internals/useRenderElement.test.tsx:21-28` and
  `packages/react/src/internals/useRenderElement.test.tsx:59-67`
- `useValueChanged(value, onChange)` — calls `onChange` when the value changes
  `packages/react/src/internals/useValueChanged.test.tsx:12-15`
- `CompositeList` — props `elementsRef`, `labelsRef`, `onMapChange`;
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:29-35` and
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:93`
- `useCompositeListItem` — params `{ index?, guess?, label?, textRef?, metadata? }`; returns
  `{ ref, index }` `packages/react/src/internals/composite/list/CompositeList.test.tsx:13-19` and
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:975-978`
- `CompositeRoot` — props exercised: `orientation` (`'horizontal' | 'vertical' | 'both'`),
  `highlightedIndex`, `onHighlightedIndexChange`, `loopFocus`, `enableHomeAndEndKeys`,
  `disabledIndices`, `modifierKeys`, `onLoop`, `grid` (built by `gridNavigation({ cols, dense?,
  itemSizes? })`), `onMapChange`
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:51-72`,
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:347-358`,
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:524`,
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:766-775`, and
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:1327-1331`
- `CompositeItem` — props exercised: `tag` (`'button'`), `metadata`, `refs` (array containing the
  forwarded ref), `render`, plus DOM props like `data-testid`
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:1301-1308` and
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:1332-1343`

## State model (controlled/uncontrolled, defaults, transitions)

- `CompositeRoot` highlighted index works in controlled mode
  (`highlightedIndex` + `onHighlightedIndexChange`) and uncontrolled mode (no props); in both,
  arrow keys move the highlight and focus together
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:60-108` and
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:110-148`
- `CompositeRoot` uses an active item's explicit `index` as the initial highlighted index: with
  `highlightedIndex={0}` and an item registered at index 2 carrying the active marker, the root
  calls `onHighlightedIndexChange(2)`
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:150-163`
- `CompositeRoot` default highlighted index is `0` — the first item gets `tabindex="0"` and others
  `-1` unless `disabledIndices` excludes index 0, in which case the initial tab stop moves to the
  first enabled item; if every listed index is disabled the tab stop stays on the first item
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:938-963`
- `CompositeRoot` on item removal: removing an item before the highlighted one keeps the tab stop
  on the same element; removing the highlighted item moves the tab stop back into range (first
  item); an "active" item (`data-composite-item-active`) retains the tab stop over the first item;
  items that cannot hold a tab stop (`display: none`, `aria-disabled`) are skipped; if no item can
  hold it, the first item keeps the tab stop
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:1065-1215`
- `CompositeList` registry state: `elementsRef.current` / `labelsRef.current` mirror the mounted
  items in DOM order, and are emptied on list unmount
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:29-43`
- `CompositeList` publishes a `Map<Element, metadata>` via `onMapChange`; each published map is
  aligned with the current `elementsRef` contents, and one publication happens per commit where
  registration changes `packages/react/src/internals/composite/list/CompositeList.test.tsx:80-117`
- `CompositeList` indexes: items with an explicit `index` register into that index-addressed slot
  (negative indexes are ignored); automatic indexes fill around reserved explicit slots; when an
  explicit index is removed the item falls back to an automatic index; when an item unmounts the
  remaining items' `data-index` values are reassigned
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:119-140`,
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:142-162`,
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:294-307`,
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:343-383`, and
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:473-492`
- `useCompositeListItem` index defaults to `-1` before registration resolves: on the server, items
  render with `data-index="-1"`; hydration then resolves real indexes
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:1108-1137`; an item rendered
  without a parent `CompositeList` also reports `data-index="-1"`
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:1140-1153`
- `useCompositeListItem` with `guess: true` assigns correct indexes during the first render without
  a corrective re-render (render count 1 per item in non-strict mode)
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:385-420`
- `RequestQueue` request status model: `'pending'` while `fetchFn` is running, `'queued'` while
  waiting for a concurrency slot, `'unknown'` for keys never queued or no longer tracked
  `packages/react/src/internals/RequestQueue.test.ts:26-38` and
  `packages/react/src/internals/RequestQueue.test.ts:53-63`
- `useRenderElement` `enabled: false` renders nothing and does not call props getters; toggling
  `enabled` across rerenders mounts/unmounts the element, and the forwarded ref is set/cleared
  accordingly `packages/react/src/internals/useRenderElement.test.tsx:200-235`
- `useButton` params are reflected in the returned props: `disabled` (attribute on native host),
  `focusableWhenDisabled` (keeps the button focusable), `tabIndex` (default `0`, or the explicit
  value) `packages/react/src/internals/use-button/useButton.test.tsx:227-264` and
  `packages/react/src/internals/use-button/useButton.test.tsx:119-133`
- `useAnimationsFinished` `batch: true` opts the callback into batched flushing; default is
  per-callback commits `packages/react/src/internals/useAnimationsFinished.test.tsx:38` and
  `packages/react/src/internals/useAnimationsFinished.test.tsx:140-204`
- `getStateAttributesProps`, `itemEquality`, `resolveValueLabel`, `stateAttributesMapping`,
  `getFilter`, `TimeoutManager` are stateless/pure from the caller's perspective (no controlled
  state is exercised) — N/A beyond the API surface above.

## Keyboard interactions

- `useButton` non-native (`native: false`, span host): `Enter` and `Space` both activate (fire
  click) after Tab-focus `packages/react/src/internals/use-button/useButton.test.tsx:19-41`
- `useButton` does not set a `type` prop on a non-native host
  `packages/react/src/internals/use-button/useButton.test.tsx:43-54`
- `useButton` non-composite: `Space` fires keydown, then click on keyup
  `packages/react/src/internals/use-button/useButton.test.tsx:281-308`; `Enter` fires keydown then
  click `packages/react/src/internals/use-button/useButton.test.tsx:786-807`
- `useButton` with `composite: true`: `Space` fires click on keydown (and keyup handlers still run);
  applies to native buttons, non-native spans, and links (`<a>`)
  `packages/react/src/internals/use-button/useButton.test.tsx:310-365`
- `useButton` inside a `CompositeRoot` context behaves as composite even with `composite`
  unspecified (Space clicks on keydown) — for both native and non-native hosts; `composite: false`
  explicitly restores keyup activation inside a composite root
  `packages/react/src/internals/use-button/useButton.test.tsx:688-753` and
  `packages/react/src/internals/use-button/useButton.test.tsx:755-784`
- `useButton` `composite: true` role-dependent Space handling when keydown is prevented: links with
  `role="menuitem"` and elements with `role="gridcell"` do not click; elements with `role="switch"`
  still click `packages/react/src/internals/use-button/useButton.test.tsx:367-431`
- `useButton` preserves native form semantics for composite native buttons: Space on a
  `type="submit"` button submits the form and on `type="reset"` resets it (once, on keydown)
  `packages/react/src/internals/use-button/useButton.test.tsx:506-562`
- `CompositeRoot` linear list: `ArrowDown`/`ArrowUp` move highlight+focus between items;
  `ArrowRight`/`ArrowLeft` for horizontal orientation (inverted in RTL)
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:85-147` and
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:408-461`
- `CompositeRoot` `Home`/`End` move focus to first/last item — only when `enableHomeAndEndKeys` is
  set `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:302-344`
- `CompositeRoot` grid navigation (`grid={gridNavigation({ cols: 3 })}`): arrows move by row/column
  with wrap-around between rows (ArrowDown from the last cell wraps to the same column in the
  first row), `Home`/`End` with `enableHomeAndEndKeys`
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:538-601` and
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:639-678`
- `CompositeRoot` grid keydown prevention: keys outside the configured orientation are prevented
  (e.g. ArrowDown with `orientation="horizontal"` + grid is prevented)
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:522-536`
- `CompositeRoot` default prevention matrix for list mode: horizontal prevents ArrowRight but not
  ArrowDown; vertical prevents ArrowDown but not ArrowRight; `both` prevents both
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:217-242`
- `CompositeRoot` modifier keys: by default any of Shift/Ctrl/Alt/Meta blocks arrow navigation; with
  `modifierKeys={['Alt', 'Meta']}` the listed modifiers are allowed and navigation proceeds while
  unlisted ones (Shift, Ctrl) still block
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:1218-1283`
- All other sub-units: N/A (no keyboard interactions tested).

## Focus management

- Roving tabindex: the highlighted `CompositeItem` gets `tabindex="0"`, all others `-1`; highlight
  and focus move together on arrow keys, in controlled and uncontrolled mode
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:83-147`
- `CompositeRoot` re-sorts item order when items (or their containers) are reordered; after the
  re-sort commits, the roving tab stop belongs to the first item in the new DOM order
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:244-300`
- `CompositeRoot` `disabledIndices`: navigation skips the listed indexes
  (`packages/react/src/internals/composite/root/CompositeRoot.test.tsx:965-999`); conversely, items
  disabled in the DOM (`aria-disabled`, `disabled` attribute) remain navigable when their index is
  not listed `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:1001-1062`
- `CompositeRoot` grid `disabledIndices` are skipped during arrow navigation (grid: 4 → 8 skips row
  containing 5/6/7 target per the cols math)
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:744-762`
- `CompositeRoot` `onLoop` is called when a navigation wraps (list and grid) with
  `(event, prevIndex, nextIndex, elementsRef)`; its return value is used as the target index —
  returning `prevIndex` keeps focus on the current item; with `loopFocus={false}` there is no
  looping and `onLoop` is never called
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:346-380`,
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:603-711`, and
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:382-406`
- `CompositeRoot` yields to native inputs: focusing a native `<input>` inside the composite selects
  its whole value so the first arrow key returns control to the textbox instead of moving focus
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:165-215`
- `useButton` `focusableWhenDisabled: true` on a disabled button: focus and blur handlers fire
  (Tab in/out works) but click, keydown, and keyup are suppressed; on a native button inside a
  composite, the hook force-overrides the disabled attribute so it stays focusable even after the
  button's ref changes
  `packages/react/src/internals/use-button/useButton.test.tsx:169-223` and
  `packages/react/src/internals/use-button/useButton.test.tsx:135-167`
- `useButton` Enter activation works when the keyboard event originates inside an open shadow root
  attached to the button host `packages/react/src/internals/use-button/useButton.test.tsx:56-114`
- Items sharing one DOM node (outer `CompositeItem` rendering a nested `CompositeItem`): Tab lands
  on the first item and arrows traverse the shared-node items in order, under both strict and
  non-strict rendering `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:1377-1398`

## Accessibility (roles, aria-*, id linking)

- `useButton` non-native host renders with `role="button"` (queried via `getByRole('button')`)
  `packages/react/src/internals/use-button/useButton.test.tsx:33` and server-side rendered output
  carries `role="button"` on the span
  `packages/react/src/internals/use-button/useButton.test.tsx:810-822`
- `useButton` disabled native button renders the `disabled` property/attribute in SSR
  `packages/react/src/internals/use-button/useButton.test.tsx:824-833`
- `useButton` custom roles are preserved and honored: `role="menuitem"`, `role="gridcell"`,
  `role="switch"` all take effect (Space activation differs per role as described above)
  `packages/react/src/internals/use-button/useButton.test.tsx:367-431`
- `useButton` dev warnings: errors if `nativeButton: true` but the ref is not a `<button>`, and if
  `nativeButton: false` but the ref is a `<button>`, with messages explaining the accessibility/
  forms impact and the fix
  `packages/react/src/internals/use-button/useButton.test.tsx:836-879`
- `CompositeRoot` does NOT add `aria-orientation` to its element even when `orientation` is set
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:49-58`
- `aria-disabled` on a `CompositeItem` marks it as unable to hold the tab stop during removal
  reassignment `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:1165-1194`
- No `id` linking (`aria-*` idrefs) is asserted anywhere in this unit's tests — N/A otherwise.
- Temporal adapters, `RequestQueue`, `TimeoutManager`, `filter`, `getStateAttributesProps`,
  `itemEquality`, `resolveValueLabel`, `stateAttributesMapping`, `CompositeList`,
  `useAnchorPositioning`, `useAnimationsFinished`, `useRenderElement`, `useValueChanged`: N/A
  (no accessibility behavior asserted in their tests).

## DOM structure & portal behavior

- `CompositeList` registers items rendered through `ReactDOM.createPortal` into a shadow root
  attached outside React (no common ancestor to observe); registration and cleanup still work
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:843-875`
- `CompositeList` uses one `MutationObserver` per shared mutation root (three items under one div
  produce exactly one `observe` call on that div)
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:682-702`
- `CompositeList` observes DOM moves made outside React and updates indexes accordingly (a leaf
  re-appended to its container gets the last index), including reorders of keyed groups and
  reorders mixed with unrelated mutations; unrelated leaf-node mutations (e.g. removing a `<span>`
  badge) do not trigger republish
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:650-680`,
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:704-762`,
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:764-802`, and
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:804-841`
- `CompositeList` SSR: items server-render with `data-index="-1"` and hydrate to their real indexes
  without a mismatch under Strict Mode
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:1114-1137`
- `CompositeRoot` is a single wrapper element around items (first child queried directly); items
  render as plain elements by default or via `tag="button"` / a `render` element
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:57` and
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:1301-1343`
- `useRenderElement` renders the default tag, or the consumer's `render` element/function; render
  functions receive merged props and state and may return a different tag (e.g. `<span>`);
  React.lazy elements and React 19 Flight-shaped lazy wrappers are supported as render elements
  `packages/react/src/internals/useRenderElement.test.tsx:271-295`,
  `packages/react/src/internals/useRenderElement.test.tsx:399-415`, and
  `packages/react/src/internals/useRenderElement.test.tsx:492-514`
- `useAnchorPositioning` wires the floating element through `refs.setFloating` and positions it
  relative to an anchor ref; the underlying floating algorithm is `useFloating` from the vendored
  `internals/floating-ui-react` (imported and passed into the hook by the test)
  `packages/react/src/internals/useAnchorPositioning.test.tsx:4-8` and
  `packages/react/src/internals/useAnchorPositioning.test.tsx:49-54`
- `scrollIntoViewIfNeeded` scrolls a container via `element.scrollTo({ left, top, behavior: 'auto' })`,
  accounting for the element's `scroll-margin-left` when computing the target position in RTL
  horizontal mode `packages/react/src/internals/composite/composite.test.ts:30-36`
- Portal behavior for the other sub-units: N/A.

## Events (names, payload shape, bubbling, preventDefault semantics)

- `useButton` synthesizes DOM `click` events from keyboard activation: Enter on keydown, Space on
  keyup (non-composite) or keydown (composite / inside a composite root); a full Space press fires
  exactly one click even on native composite buttons
  `packages/react/src/internals/use-button/useButton.test.tsx:281-342`,
  `packages/react/src/internals/use-button/useButton.test.tsx:462-480`, and
  `packages/react/src/internals/use-button/useButton.test.tsx:688-753`
- `useButton` preventDefault semantics: `event.preventDefault()` on the Enter keydown cancels
  activation; on the Space keyup cancels activation (non-composite); for composite links/gridcells
  a prevented Space keydown cancels click (unless the role is `switch`)
  `packages/react/src/internals/use-button/useButton.test.tsx:634-686` and
  `packages/react/src/internals/use-button/useButton.test.tsx:367-431`
- `useButton` `event.preventBaseUIHandler()` (Base UI extension, distinct from `preventDefault`)
  on keydown blocks the composite Space click; on non-composite buttons it blocks both Enter
  (keydown) and Space (keyup) activation
  `packages/react/src/internals/use-button/useButton.test.tsx:564-594` and
  `packages/react/src/internals/use-button/useButton.test.tsx:596-632`
- `useRenderElement` attaches real event handlers that receive a Base UI event wrapper exposing
  `preventBaseUIHandler()`; calling it does not throw, for single prop objects, multi-prop arrays
  (when the handler is first), and obscure events like `contextmenu`
  `packages/react/src/internals/useRenderElement.test.tsx:136-198`
- `CompositeRoot` `onLoop` payload: `(event, prevIndex, nextIndex, elementsRef)` where `event.key`
  is the pressed key, indexes are registry positions, and `elementsRef.current[i]` is the item
  element; the callback's return value overrides the target index
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:346-380` and
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:603-711`
- `CompositeRoot` `onHighlightedIndexChange` receives the new numeric index
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:150-163`
- `CompositeRoot` keydown defaultPrevention: navigation keys inside the active orientation have
  their default prevented; keys outside it are left allowed (list) or prevented (grid with a
  configured orientation) `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:217-242`
  and `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:522-536`
- `CompositeList` `onMapChange` payload: `Map<Element, { index: number, ...metadata }>` — item
  metadata passed to `useCompositeListItem({ metadata })` is published alongside the index
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:968-991`
- `useAnchorPositioning` passes a `shift` middleware config through to floating-ui: `rootBoundary`
  is `undefined` by default (visual viewport) and `crossAxis` defaults to `false`; the configured
  `shift.rootBoundary`/`shift.crossAxis` values are forwarded as given
  `packages/react/src/internals/useAnchorPositioning.test.tsx:64-79`
- Key event details: composite key handling inspects `event.key` (`' '`/`'Enter'`) and modifier
  flags (`shiftKey`, `ctrlKey`, `altKey`, `metaKey`)
  `packages/react/src/internals/use-button/useButton.test.tsx:301-307` and
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:1233-1247`
- No custom bubbling behavior is asserted for these events beyond standard DOM bubbling used in
  tests — N/A otherwise.

## Edge cases (rapid interactions, unmount, nesting)

- `RequestQueue`: does not re-queue an already-pending key; enforces `maxConcurrentRequests`;
  `setRequestSettled` and `clearPendingRequest` both start the next queued item and drop the key to
  `'unknown'`; `clear()` resets everything; a rejected `fetchFn` removes the key and processing
  continues to the next item; duplicate complex keys deduplicate via `getKeyId`; keys process in
  FIFO order `packages/react/src/internals/RequestQueue.test.ts:40-151`
- `TimeoutManager`: `start` with the same key replaces the pending timeout (first callback never
  fires); `clear` on a missing or already-fired key is safe; `clearAll` cancels everything
  `packages/react/src/internals/TimeoutManager.test.ts:24-75`
- `CompositeList` unmount/mount churn: refs are cleaned up on unmount; items mounting/unmounting
  from nested state updates are added/removed in correct DOM positions; a mounted item that stops
  rendering an element deregisters and the tail reindexes; replacing a ref object or the render
  target re-registers without spurious publications; changing an explicit index re-registers the
  item at the new slot (element refs cycle through detach/attach) while automatic index shifts do
  NOT detach the item's refs
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:22-43`,
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:199-292`,
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:309-383`,
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:422-471`, and
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:494-531`
- `CompositeList` out-of-React DOM surgery: items detached outside React are excluded from the
  registry and skipped during order verification; moves outside React reindex
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:533-605`
- `CompositeList` React 18 Strict Mode: guessed-index items stay populated even when the strict
  replay empties arrays; guesses are not consumed by explicitly indexed siblings
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:45-78` and
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:164-197`
- `CompositeList` Suspense: when an outer boundary repeatedly suspends, the registry never
  publishes an empty map; every published snapshot contains all items aligned with
  `elementsRef`/`labelsRef` `packages/react/src/internals/composite/list/CompositeList.test.tsx:994-1105`
- `CompositeList` labels: label resolution order is explicit `label` prop → `textRef` → element
  text; an explicit `null` label means "no label" and does not fall back to text; unmounting drops
  the label slot; label updates propagate
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:898-965`
- `CompositeRoot` nested items sharing a DOM node: the outer registration owns the published
  metadata; updating only the inner item's registration data must not transfer ownership (precedence
  comes from ref attachment order), under both strict and non-strict rendering
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:1377-1415`
- `useAnimationsFinished`: a canceled animation's wait does not complete if a replacement animation
  appears, and completes if there is no replacement; batched callbacks finishing in the same
  microtask commit once; a callback whose signal aborts while the batch is flushing is skipped; by
  default each callback commits separately so later callbacks observe earlier updates (a popup
  reopened by an earlier completion does not unmount)
  `packages/react/src/internals/useAnimationsFinished.test.tsx:56-138`,
  `packages/react/src/internals/useAnimationsFinished.test.tsx:140-249`, and
  `packages/react/src/internals/useAnimationsFinished.test.tsx:251-334`
- `useRenderElement`: merged refs update when the ref shape changes (single ↔ array ↔ different
  single), stale handlers are removed; className/style functions returning `undefined` fall back to
  internal values; frozen `EMPTY_OBJECT` state does not throw when className/style are provided;
  invalid render elements (non-element values, or Flight wrappers unwrapping to null/false/0/'')
  throw a development error "The `render` prop was provided an invalid React element"; a disabled
  hook does not unwrap a pending lazy element
  `packages/react/src/internals/useRenderElement.test.tsx:237-268`,
  `packages/react/src/internals/useRenderElement.test.tsx:103-134`,
  `packages/react/src/internals/useRenderElement.test.tsx:652-682`,
  `packages/react/src/internals/useRenderElement.test.tsx:584-632`, and
  `packages/react/src/internals/useRenderElement.test.tsx:563-580`
- `useRenderElement` render-function naming: warns when the function name starts with an uppercase
  letter (including acronym prefixes like `UIInput`); does not warn for lowercase callbacks,
  SCREAMING_SNAKE_CASE, `useCallback`-inferred names, or React elements
  `packages/react/src/internals/useRenderElement.test.tsx:297-397`
- `useButton`: nested non-native composite buttons (two `useButton` calls merged on one element)
  fire a single click per Space press; duplicate clicks are avoided for native composite buttons;
  Space submits/resets forms exactly once
  `packages/react/src/internals/use-button/useButton.test.tsx:482-504`,
  `packages/react/src/internals/use-button/useButton.test.tsx:462-480`, and
  `packages/react/src/internals/use-button/useButton.test.tsx:506-562`
- `useValueChanged`: transitioning `0 → -0` is not a change (no callback) while `-0` is retained as
  the previous value; the next real change fires once with `Object.is(-0)`-identical payload; holds
  under both Strict and non-Strict rendering
  `packages/react/src/internals/useValueChanged.test.tsx:7-31`
- `resolveValueLabel`: values matching `Object.prototype` member names (`constructor`, `toString`,
  `hasOwnProperty`, `__proto__`) fall back to the stringified label unless an own key exists;
  nullish own-key labels fall back to the stringified value; the `'null'` record key still resolves
  the null-value placeholder `packages/react/src/internals/resolveValueLabel.test.ts:123-148`
- `itemEquality` semantics: `+0` and `-0` are distinct (like `Object.is`); `NaN` and `null` match
  themselves; `undefined` items/values never match; array holes left by unmounted items never
  match; selection anchoring reads the selected values once instead of rescanning per item
  `packages/react/src/internals/itemEquality.test.ts:40-90`
- `resolveSelectedIndex` handoff: the index is claimed only by the earliest rendered selected item;
  an earlier holder that is deselected or has left the registry lets a later item take over
  `packages/react/src/internals/itemEquality.test.ts:107-135`
- Temporal adapters: date-only strings preserve the calendar date in negative UTC offsets and named
  timezones (parsed as UTC midnight, not local-rolled-back), while datetime strings keep local
  getters; `setTimezone` uses `withTimeZone` duck-typing for TZDate-like objects from a different
  library copy `packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.test.ts:15-63`

## Shared harness dependencies

- `#test-utils` (alias for `packages/react/test/index.ts`) is imported by:
  `useButton.test.tsx` (`createRenderer`, `isJSDOM`)
  `packages/react/src/internals/use-button/useButton.test.tsx:5`,
  `CompositeList.test.tsx` (`createRenderer`, `mergeRefs`)
  `packages/react/src/internals/composite/list/CompositeList.test.tsx:5`,
  `CompositeRoot.test.tsx` (`isJSDOM`)
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:11`,
  `useAnchorPositioning.test.tsx` (`createRenderer`)
  `packages/react/src/internals/useAnchorPositioning.test.tsx:3`,
  `useAnimationsFinished.test.tsx` (`createRenderer`)
  `packages/react/src/internals/useAnimationsFinished.test.tsx:6`,
  `useRenderElement.test.tsx` (`createRenderer`)
  `packages/react/src/internals/useRenderElement.test.tsx:4`, and both temporal adapter tests
  (`describeGregorianAdapter`)
  `packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.test.ts:3` /
  `packages/react/src/internals/temporal-adapter-luxon/TemporalAdapterLuxon.test.ts:5`
- `packages/react/test/createRenderer.ts` wraps `@mui/internal-test-utils`'s renderer: `render` is
  awaited inside `act` and the result gains async `rerender`/`setProps` helpers; `strict` is a
  create-renderer option `packages/react/test/createRenderer.ts:27-49`
- `packages/react/test/index.ts` re-exports `@base-ui/utils/testUtils` (source of `isJSDOM`) and
  the temporal `describeGregorianAdapter` suite
  `packages/react/test/index.ts:1-14`
- The shared `describeGregorianAdapter` harness (`packages/react/test/describeGregorianAdapter/`)
  supplies the bulk of temporal-adapter coverage — date/now parsing, `setTimezone`, equality
  (`isEqual`, `isSame*`, `isAfter`, `isBefore`, `isWithinRange`), arithmetic (`startOfWeek`,
  `endOfMonth`, `addMonths`, `addWeeks`), differences (`differenceIn*`), plus formats and
  localization suites — parameterized per adapter via `{ adapter, adapterFr, setDefaultTimezone,
  createDateInFrenchLocale }`
  `packages/react/src/internals/temporal-adapter-luxon/TemporalAdapterLuxon.test.ts:9-16`; the
  suite's method-level describes live in `packages/react/test/describeGregorianAdapter/testComputations.ts:37-809`
- `@mui/internal-test-utils` (`act`, `fireEvent`, `screen`, `waitFor`, `flushMicrotasks`,
  `reactMajor`) is an external npm dependency (v2.0.18-canary.25 in the root `package.json`), not a
  repo harness file
  `packages/react/src/internals/useAnimationsFinished.test.tsx:3` and
  `packages/react/src/internals/composite/root/CompositeRoot.test.tsx:3-10`
