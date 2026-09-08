# Behavior spec — `infra: utils` (React-internal shared infra, `packages/react/src/utils/`)

Scope: the twelve test files listed for this unit. The unit has **no** `wraps-external:` field in
`TODO.md` (`TODO.md:292-297`); the only external Floating UI involvement is `hideMiddleware`,
whose custom `hide` middleware is *tested against* `@floating-ui/react-dom`'s native `hide` for
parity (`packages/react/src/utils/hideMiddleware.test.ts:2-8`), not delegated to it.

This directory is a grab-bag of independent utilities. Sections below are grouped per util within
each required section.

## Public API surface (props, parts, subcomponents)

- `getPseudoElementBounds(element)` and `isMouseWithinBounds(event, element)` are exported from
  `packages/react/src/utils/getPseudoElementBounds` `packages/react/src/utils/getPseudoElementBounds.test.ts:3`.
- `hide` is a Floating UI middleware (an object with a `fn(state)` returning `data.referenceHidden`)
  exported from `packages/react/src/utils/hideMiddleware` `packages/react/src/utils/hideMiddleware.test.ts:8`, `packages/react/src/utils/hideMiddleware.test.ts:52-56`.
- `ListboxSeparator` is a React component; it is also re-exported as `Autocomplete.Separator`,
  `Combobox.Separator`, and `Select.Separator` with identical rendered behavior
  `packages/react/src/utils/listbox-separator/ListboxSeparator.test.tsx:43-56`. Props proven:
  `orientation` (`'horizontal' | 'vertical'`) `packages/react/src/utils/listbox-separator/ListboxSeparator.test.tsx:26-41`; arbitrary data attributes pass through (`data-testid`)
  `packages/react/src/utils/listbox-separator/ListboxSeparator.test.tsx:18`.
- `popupStateMapping`, `triggerOpenStateMapping`, `pressableTriggerOpenStateMapping` each expose an
  `open(boolean)` function; `popupStateMapping` also exposes `anchorHidden(boolean)`
  `packages/react/src/utils/popupStateMapping.test.ts:2-6`.
- `popups/inlineRect` exports `createInlineMiddleware`, `getInlineRectTriggerProps`,
  `updateInlineRectCoords`, and the `InlineRectCoords` type `{ x, y, lineIndex, element }`
  `packages/react/src/utils/popups/inlineRect.test.ts:3-8`, `packages/react/src/utils/popups/inlineRect.test.ts:89-91`.
- `popups` barrel exports `applyPopupOpenChange`, `createInitialPopupStoreState`, `createPopupOpenState`,
  `PopupStoreContext`, `PopupStoreState`, `PopupStoreSelectors`, `PopupTriggerMap`,
  `popupStoreSelectors`, `useImplicitActiveTrigger`, `usePopupInteractionProps`,
  `useTriggerDataForwarding`, `useTriggerRegistration`
  `packages/react/src/utils/popups/popupStoreUtils.test.tsx:7-20`.
- `PopupTriggerMap` is a class with `add(id, element)`, `delete(id)`, `getById(id)`, `hasElement(el)`,
  `hasMatchingElement(predicate)`, and a `size` getter `packages/react/src/utils/popups/popupTriggerMap.test.ts:6-16`.
- `popupStoreSelectors.isOpenedByTrigger(state, triggerId)` is a selector over `PopupStoreState`
  `packages/react/src/utils/popups/store.test.ts:14-24`.
- `scrollEdges` exports `normalizeScrollOffset(offset, max)` and the constant
  `SCROLL_EDGE_TOLERANCE_PX` `packages/react/src/utils/scrollEdges.test.ts:2`.
- `useIsHydrating()` is a hook returning a boolean `packages/react/src/utils/useIsHydrating.test.tsx:10-14`.
- `useRegisteredLabelId(id, setLabelId)` is a hook taking a label id and a state setter, returning an id
  `packages/react/src/utils/useRegisteredLabelId.test.tsx:19-21`.
- `useSwipeDismiss(options)` is a hook. Options proven by tests: `enabled`, `directions`
  (`['down']`, `['right']`, `['down','right']`), `elementRef`, `movementCssVars` (`{x, y}` CSS var
  names), `ignoreScrollableAncestors`, `swipeThreshold`, `onSwipeStart`, `onProgress`,
  `onSwipingChange`, `onDismiss`, `onCancel`, `onRelease`. The returned object exposes
  `getDragStyles()`, `getPointerProps()`, `getTouchProps()`, `reset()`, and a `swiping` boolean
  state `packages/react/src/utils/useSwipeDismiss.test.tsx:7-19`, `packages/react/src/utils/useSwipeDismiss.test.tsx:56-68`, `packages/react/src/utils/useSwipeDismiss.test.tsx:656-674`, `packages/react/src/utils/useSwipeDismiss.test.tsx:1312-1326`.

## State model (controlled/uncontrolled, defaults, transitions)

### Popup store (`popups/store`, `popups/popupStoreUtils`)

- The store is a `ReactStore` (from `@base-ui/utils/store`) created with
  `createInitialPopupStoreState(new PopupTriggerMap())`, context `{ triggerElements, popupRef,
  onOpenChangeComplete }`, and `popupStoreSelectors` `packages/react/src/utils/popups/popupStoreUtils.test.tsx:34-48`.
- `open` controlled vs internal: `popupStoreSelectors.isOpenedByTrigger` prefers the controlled
  `openProp` when present — `openProp: true` wins over internal `open: false`, and `openProp: false`
  wins over internal `open: true` `packages/react/src/utils/popups/store.test.ts:15-35`. When
  uncontrolled (no `openProp`), the internal `open` decides
  `packages/react/src/utils/popups/store.test.ts:37-55`. The selector also requires the queried
  trigger id to equal `state.activeTriggerId` `packages/react/src/utils/popups/store.test.ts:57-67`.
- `triggerCount` transitions: 0 while the popup is closed (registrations do not notify the store)
  `packages/react/src/utils/popups/popupStoreUtils.test.tsx:230-249`; becomes 1+ only while `open` is
  true (`packages/react/src/utils/popups/popupStoreUtils.test.tsx:379-392`); resets to 0 when the popup
  closes `packages/react/src/utils/popups/popupStoreUtils.test.tsx:741-761`.
- Active trigger claiming: when a closed popup with exactly one registered trigger opens, that
  trigger is implicitly claimed — `activeTriggerId` and `activeTriggerElement` are set to it
  `packages/react/src/utils/popups/popupStoreUtils.test.tsx:394-416`.
- Active-trigger reconciliation while open:
  - Replacing the active trigger element under the same id keeps the popup open and moves
    `activeTriggerElement` to the replacement `packages/react/src/utils/popups/popupStoreUtils.test.tsx:592-633`.
  - If the active trigger element gets registered under a different id than its DOM id, the popup
    stays open and `activeTriggerId` follows the registered id
    `packages/react/src/utils/popups/popupStoreUtils.test.tsx:635-660`.
  - A handoff updating `activeTriggerId` to another trigger's DOM id is reassociated to that
    trigger's rendered id (`'dom-id-2'` → `'registered-2'`) without changing `open` or
    `triggerCount` `packages/react/src/utils/popups/popupStoreUtils.test.tsx:662-698`.
- `createPopupOpenState` (next-state builder for open changes):
  - Opening clears a previous `preventUnmountingOnClose` request (input state is not mutated)
    `packages/react/src/utils/popups/popupStoreUtils.test.tsx:831-839`.
  - Closing sets `preventUnmountingOnClose: true` when requested (4th argument)
    `packages/react/src/utils/popups/popupStoreUtils.test.tsx:841-847`.
  - Closing without a trigger preserves the previous `activeTriggerId`/`activeTriggerElement`
    `packages/react/src/utils/popups/popupStoreUtils.test.tsx:849-859`.
- `usePopupInteractionProps` stores `{ activeTriggerProps, inactiveTriggerProps, popupProps }` into
  store state and replaces all three with fresh `{}` objects on unmount
  `packages/react/src/utils/popups/popupStoreUtils.test.tsx:799-827`.
- `popupId` selector: `useSyncedFloatingRootContext` syncs `floatingId` into the store, and
  `select('popupId')` returns it; an empty string floating id yields `undefined`; an explicit
  `popupElement.id` on the open state wins over the generated `floatingId`
  `packages/react/src/utils/popups/popupStoreUtils.test.tsx:764-796`.

### `useIsHydrating`

- Boolean state: `false` for client-only mounts; `true` before hydration of server-rendered markup;
  transitions to `false` once `hydrate()` completes
  `packages/react/src/utils/useIsHydrating.test.tsx:16-38`.

### `useSwipeDismiss`

- `swiping` state transitions true at gesture start and false at gesture end, observable via
  `onSwipingChange(true)` then `onSwipingChange(false)` and via the `swiping` field
  `packages/react/src/utils/useSwipeDismiss.test.tsx:920-972`, `packages/react/src/utils/useSwipeDismiss.test.tsx:673`.
- `swipeThreshold` is snapshotted when a gesture starts: changing the prop mid-gesture (50 → 10)
  does not affect the active gesture's dismissal decision; the next gesture uses the new value
  `packages/react/src/utils/useSwipeDismiss.test.tsx:822-918`.
- Default `swipeThreshold` is 40px for a `['down']`-direction box: a 35px displacement does not
  dismiss while a 100px one does `packages/react/src/utils/useSwipeDismiss.test.tsx:1155-1229`, `packages/react/src/utils/useSwipeDismiss.test.tsx:1079-1153`; a custom `swipeThreshold: 10` dismisses at 20px `packages/react/src/utils/useSwipeDismiss.test.tsx:743-820`.
- Drag position state is exposed through `getDragStyles()` (transform + `transition: 'none'` while
  swiping) and the `movementCssVars` CSS custom properties (`--y: '40px'` for a 40px drag);
  `--y` returns to `'0px'` when the gesture cancels or resets
  `packages/react/src/utils/useSwipeDismiss.test.tsx:605-653`, `packages/react/src/utils/useSwipeDismiss.test.tsx:1059`.

## Keyboard interactions

N/A — no test in this unit asserts keyboard-driven behavior. The closest is that
`usePopupInteractionProps` merely *stores* whatever handlers the caller passes (a test uses
`onKeyDown` as sample content of `inactiveTriggerProps`), without asserting any keyboard semantics
`packages/react/src/utils/popups/popupStoreUtils.test.tsx:802-804`, `packages/react/src/utils/popups/popupStoreUtils.test.tsx:816-824`.

## Focus management

- `getInlineRectTriggerProps(coordsRef, open)` returns an `onFocus` handler that clears the stored
  inline coords (resets positioning to the whole-trigger rect) when the trigger receives focus
  `packages/react/src/utils/popups/inlineRect.test.ts:87-96`.
- No other focus-management behavior is asserted anywhere in this unit's tests. N/A otherwise.

## Accessibility (roles, aria-*, id linking)

- `ListboxSeparator` renders `role="presentation"`, a `data-orientation` attribute reflecting the
  `orientation` prop (default `horizontal`), and deliberately no `aria-orientation`
  `packages/react/src/utils/listbox-separator/ListboxSeparator.test.tsx:17-41`.
- `useRegisteredLabelId` maintains an `aria-labelledby` link: the hook registers the label's id via
  the `setLabelId` setter, and the target's `aria-labelledby` follows the newest registered label
  (see Edge cases for the ordering guarantee) `packages/react/src/utils/useRegisteredLabelId.test.tsx:19-21`, `packages/react/src/utils/useRegisteredLabelId.test.tsx:43-54`.
- Popup state is exposed only through data attributes, not aria attributes:
  `data-open`/`data-closed` on the popup, `data-anchor-hidden` on the popup when the anchor is
  hidden, `data-popup-open` on triggers (plus `data-pressed` for pressable triggers) while open
  `packages/react/src/utils/popupStateMapping.test.ts:11-32`.

## DOM structure & portal behavior

- `ListboxSeparator` renders a `HTMLDivElement` (refs forwarded to it are instances of it)
  `packages/react/src/utils/listbox-separator/ListboxSeparator.test.tsx:12-15`.
- Everything else in this unit is hooks/plain functions with no DOM output of their own; the
  `useSwipeDismiss` hook styles its host element imperatively (writes `transform`,
  `transition`, and the configured movement CSS custom properties onto `elementRef.current`)
  `packages/react/src/utils/useSwipeDismiss.test.tsx:605-653`.
- Portal behavior: N/A — no test in this unit exercises portalling.

## Events (names, payload shape, bubbling, preventDefault semantics)

### `applyPopupOpenChange`

- Order of operations for a non-canceled change: context `onOpenChange(open, details)` →
  `onBeforeDispatch` callback → `floatingRootContext.dispatchOpenChange(open, details)` → store
  `update(...)` (exactly once) `packages/react/src/utils/popups/popupStoreUtils.test.tsx:904-919`.
- If `onOpenChange` calls `details.cancel()`, the change is short-circuited: `onBeforeDispatch`,
  `dispatchOpenChange`, and `update` are never invoked `packages/react/src/utils/popups/popupStoreUtils.test.tsx:921-934`.
- `extraState` is merged into the store update, but `open` always reflects the requested
  `nextOpen`, overriding any `open` present in `extraState`
  `packages/react/src/utils/popups/popupStoreUtils.test.tsx:936-947`.
- The change `reason` maps to `instantType`: `triggerFocus` → `'focus'`; `triggerPress` (close) →
  `'dismiss'`; `escapeKey` (close) → `'dismiss'`; `triggerHover` → the key is present but
  `undefined`; `none` → the `instantType` key is absent entirely
  `packages/react/src/utils/popups/popupStoreUtils.test.tsx:949-971`.

### `useSwipeDismiss` event handling

- Pointer events are attached via `getPointerProps()`, touch events via `getTouchProps()`
  `packages/react/src/utils/useSwipeDismiss.test.tsx:17`, `packages/react/src/utils/useSwipeDismiss.test.tsx:156`.
- A `pointerdown` that was default-prevented (on the target or a descendant) never starts a swipe:
  `onSwipeStart` is not called and the movement CSS var stays `'0px'`
  `packages/react/src/utils/useSwipeDismiss.test.tsx:1666-1724`.
- The first `pointermove` after pointerdown only establishes the baseline; subsequent moves drive
  the transform/CSS vars `packages/react/src/utils/useSwipeDismiss.test.tsx:623-653`, `packages/react/src/utils/useSwipeDismiss.test.tsx:285-295`.
- `touchmove` is never default-prevented by this hook, both for unsupported and supported
  directions `packages/react/src/utils/useSwipeDismiss.test.tsx:268-332`.
- `pointerup` ends the gesture and decides dismissal (past threshold → `onDismiss`);
  `onRelease` receives details `{ velocityX, velocityY, releaseVelocityX, releaseVelocityY }` and
  returning `false` from `onRelease` overrides/prevents dismissal
  `packages/react/src/utils/useSwipeDismiss.test.tsx:743-820`, `packages/react/src/utils/useSwipeDismiss.test.tsx:1392-1477`, `packages/react/src/utils/useSwipeDismiss.test.tsx:1312-1390`.
- A `pointermove` with `buttons: 0` (primary button already released, no `pointerup` observed) acts
  as a release: it cancels the gesture when displacement is below threshold (resetting `--y` to
  `'0px'`, `onSwipingChange(false)`, no `onDismiss`) and commits (`onDismiss`, no `onCancel`) when
  past threshold; the final `buttons: 0` move's displacement is applied before the release decision
  `packages/react/src/utils/useSwipeDismiss.test.tsx:974-1077`, `packages/react/src/utils/useSwipeDismiss.test.tsx:1079-1153`, `packages/react/src/utils/useSwipeDismiss.test.tsx:1155-1229`.
- Callbacks fired: `onSwipeStart` (gesture start), `onProgress(progress)` on moves,
  `onSwipingChange(boolean)`, `onDismiss()`, `onCancel()`, `onRelease(details)`
  `packages/react/src/utils/useSwipeDismiss.test.tsx:131`, `packages/react/src/utils/useSwipeDismiss.test.tsx:382-384`, `packages/react/src/utils/useSwipeDismiss.test.tsx:969-971`, `packages/react/src/utils/useSwipeDismiss.test.tsx:1151-1152`, `packages/react/src/utils/useSwipeDismiss.test.tsx:1471-1473`.
- Velocity semantics: `velocityX` is computed from the gesture timeline using event `timeStamp`s
  (50px over 200ms → 0.25 px/ms) `packages/react/src/utils/useSwipeDismiss.test.tsx:1420-1473`;
  `releaseVelocityX` comes from the latest movement segment (20px over 16ms → 1.25 px/ms)
  `packages/react/src/utils/useSwipeDismiss.test.tsx:1548-1573`; very short durations are clamped
  when computing velocity (30px over ≤50ms window → 0.6 px/ms)
  `packages/react/src/utils/useSwipeDismiss.test.tsx:1579-1664`.

### `getInlineRectTriggerProps` handlers

- `onMouseMove`: while closed, stores `{ x, y, lineIndex, element }` where `lineIndex` is the index
  of the client rect containing the pointer; stores `undefined` when the trigger does not wrap
  (single client rect); while open it does not update
  `packages/react/src/utils/popups/inlineRect.test.ts:111-135`.
- `onMouseEnter`: does not update stored coords while open
  `packages/react/src/utils/popups/inlineRect.test.ts:137-149`.
- `updateInlineRectCoords` updates the stored coords directly from a native mouse event regardless
  of open state `packages/react/src/utils/popups/inlineRect.test.ts:151-163`.

### Other

- `hide` middleware is a pure middleware computation over `platform.detectOverflow` (called twice,
  both times with `elementContext: 'reference'`); it emits `data.referenceHidden`
  `packages/react/src/utils/hideMiddleware.test.ts:52-63`.
- Scroll-position events are handled by `normalizeScrollOffset` (see State model/Edge cases);
  no DOM event listeners are asserted for it. N/A otherwise.

## Edge cases (rapid interactions, unmount, nesting)

### `getPseudoElementBounds` / `isMouseWithinBounds`

- In jsdom the function never reads pseudo-element styles (`getComputedStyle` is not called) and
  returns the element's own rect `packages/react/src/utils/getPseudoElementBounds.test.ts:10-21`; in
  real browsers it expands to include pseudo-element bounds (element 20×10 at (100,50) with a 40×30
  `::before` yields `left: 90, right: 130, top: 40, bottom: 70`)
  `packages/react/src/utils/getPseudoElementBounds.test.ts:23-59`.
- `isMouseWithinBounds` tolerates up to 5px of drift outside each edge (e.g. 95 is within for a
  left edge at 100, 94 is not) `packages/react/src/utils/getPseudoElementBounds.test.ts:61-79`.

### `hideMiddleware`

- Parity with native Floating UI `hide` on reference-overflow thresholds: reference 10×20 at (1,2);
  overflow `{top:19,right:9,bottom:19,left:9}` → not hidden; any side reaching 20% (top/bottom 20,
  right/left 10) → hidden `packages/react/src/utils/hideMiddleware.test.ts:41-65`.
- An all-zero reference rect is reported hidden by the custom middleware even though native
  Floating UI reports it visible `packages/react/src/utils/hideMiddleware.test.ts:67-78`.

### `ListboxSeparator`

- Both `horizontal` and `vertical` orientations render the same attribute set (no `aria-orientation`
  in either) `packages/react/src/utils/listbox-separator/ListboxSeparator.test.tsx:26-41`.

### `inlineRect` middleware

- Returns `{}` (no reset) when: the reference does not wrap (single client rect)
  `packages/react/src/utils/popups/inlineRect.test.ts:313-323`; the reference has no client rects at
  all `packages/react/src/utils/popups/inlineRect.test.ts:325-342`; the computed inline rect equals
  the current reference rect `packages/react/src/utils/popups/inlineRect.test.ts:344-363`.
- Reuses the captured `lineIndex` when client coordinates have become stale (rects moved between
  capture and middleware run) `packages/react/src/utils/popups/inlineRect.test.ts:195-227`.
- Ignores stored coords captured on a different element `packages/react/src/utils/popups/inlineRect.test.ts:365-396`.
- Accepts a virtual reference whose `contextElement` is the trigger
  `packages/react/src/utils/popups/inlineRect.test.ts:398-429`.
- Non-`bottom` placements use edge-aligned union rects: for `right` the rect spans from line 0's
  left to the widest line's right; for `left` it aligns to the widest line's edges
  `packages/react/src/utils/popups/inlineRect.test.ts:229-283`; disjoint two-line triggers without
  coords fall back to a spanning rect `packages/react/src/utils/popups/inlineRect.test.ts:285-311`.
- Client-space line rects are converted through the positioning platform's `getElementRects` when
  provided, invoked exactly once `packages/react/src/utils/popups/inlineRect.test.ts:431-469`.

### `PopupTriggerMap`

- `add` under a reused id replaces the old element `packages/react/src/utils/popups/popupTriggerMap.test.ts:18-30`; adding the same element twice under the same id does not duplicate
  `packages/react/src/utils/popups/popupTriggerMap.test.ts:45-54`.
- In non-production, registering the same element under a second id throws
  `Base UI: A trigger element cannot be registered under multiple IDs in PopupTriggerMap.`
  `packages/react/src/utils/popups/popupTriggerMap.test.ts:56-71,114-124`; in production it does not
  throw and both ids resolve to the element `packages/react/src/utils/popups/popupTriggerMap.test.ts:126-146`.
- Deleting an id releases the element's claim: re-registering under a new id then works, while
  claims held by *other* ids are unaffected by an unrelated delete
  `packages/react/src/utils/popups/popupTriggerMap.test.ts:73-97`; an element evicted by id reuse is
  free to claim a new id `packages/react/src/utils/popups/popupTriggerMap.test.ts:99-112`.

### Trigger registration & unmount (`popupStoreUtils`)

- Closed-popup registrations go only into `context.triggerElements` (the map), never notifying the
  store (`store.set` not called, `triggerCount` stays 0), and id changes migrate the registration
  `packages/react/src/utils/popups/popupStoreUtils.test.tsx:230-276`.
- The register callback is stable across store migration and always acts on the *current* store —
  a retained callback from before the migration unregisters from the old store and registers into
  the new one `packages/react/src/utils/popups/popupStoreUtils.test.tsx:278-319`.
- The ref-only pattern (callback merged into an element ref, mirroring `Drawer.SwipeArea`)
  registers once the id resolves after the first commit and follows later id changes
  `packages/react/src/utils/popups/popupStoreUtils.test.tsx:321-377`.
- Unmount behavior of the active trigger while open, with `useImplicitActiveTrigger(store, {
  closeOnActiveTriggerUnmount: true })`:
  - Closing: an implicitly claimed trigger unmounting during the claim commit closes the popup via
    `setOpen(false, { reason: 'none' })`, clearing `activeTriggerId`/`activeTriggerElement`
    `packages/react/src/utils/popups/popupStoreUtils.test.tsx:418-434`; the same happens when the
    active trigger unregisters while open `packages/react/src/utils/popups/popupStoreUtils.test.tsx:436-475`;
    and when the active trigger unmounts after registering in a count-neutral commit
    `packages/react/src/utils/popups/popupStoreUtils.test.tsx:532-590`.
  - Staying open: when ownership returns to a pending trigger (a different unresolved id, or no
    active trigger) `packages/react/src/utils/popups/popupStoreUtils.test.tsx:477-530`; when the active
    trigger element is replaced under the same id `packages/react/src/utils/popups/popupStoreUtils.test.tsx:592-633`;
    when the active element registers under another id `packages/react/src/utils/popups/popupStoreUtils.test.tsx:635-660`.
  - Default (no option): active-trigger ownership is preserved as stale state and the popup does
    not close when the active trigger unmounts `packages/react/src/utils/popups/popupStoreUtils.test.tsx:700-739`.

### `useSwipeDismiss`

- Scroll gating: with `ignoreScrollableAncestors: true`, a swipe starting inside a scrollable
  descendant never starts (`onSwipeStart` not called, `--y` stays `'0px'`)
  `packages/react/src/utils/useSwipeDismiss.test.tsx:56-136`; when the page scroller is `body`,
  touch swipes are still allowed (browser-only test) `packages/react/src/utils/useSwipeDismiss.test.tsx:138-196`;
  but a scrollable descendant on the *other* axis still gates the swipe away from its start edge
  (browser-only test) `packages/react/src/utils/useSwipeDismiss.test.tsx:198-266`.
- `onProgress` is relative to element size (50px move on a 200px-wide element → 0.25) and keeps
  firing when progress is clamped to 0 by reverse movement
  `packages/react/src/utils/useSwipeDismiss.test.tsx:334-384`, `packages/react/src/utils/useSwipeDismiss.test.tsx:386-467`.
- Movement in the unsupported direction applies exponential damping rather than a hard zero
  (`--y` is non-zero after an upward move on a down-only box)
  `packages/react/src/utils/useSwipeDismiss.test.tsx:469-513`.
- `reset()` and gesture end restore the element's own inline `transform`/`transition` (e.g.
  `scale(0.9)` / `opacity 200ms ease` are preserved around drag writes)
  `packages/react/src/utils/useSwipeDismiss.test.tsx:515-603`.
- A render that commits during a gesture before the `swiping` state flushes still emits the
  current drag transform and `transition: 'none'` from `getDragStyles()` (so React reconciliation
  cannot strip the mid-gesture transform) `packages/react/src/utils/useSwipeDismiss.test.tsx:655-741`.
- Touch ending over a scrollable descendant resets the gesture (`onSwipingChange(false)`, `--y`
  back to `'0px'`) `packages/react/src/utils/useSwipeDismiss.test.tsx:1231-1310`.
- StrictMode (`strict: true`) and non-strict renders behave identically for threshold snapshotting
  `packages/react/src/utils/useSwipeDismiss.test.tsx:822-918`.

### `scrollEdges`

- `normalizeScrollOffset` returns 0 for a non-positive max; snaps offsets within tolerance to the
  nearest edge (0.5/10 → 0, 9.5/10 → 10); keeps mid-range values unchanged; and picks the closest
  edge when tolerances overlap (at max = tolerance: 0.4·max → 0, 0.6·max → max)
  `packages/react/src/utils/scrollEdges.test.ts:5-29`.

### `useRegisteredLabelId`

- Late-arriving label ordering: when a second (newer) label registers, `aria-labelledby` switches
  to it, and when the *older* label later unmounts, its cleanup does not clear or revert the newer
  label's registration `packages/react/src/utils/useRegisteredLabelId.test.tsx:43-54`.

## Shared harness dependencies

- `#test-utils` resolves to the barrel `packages/react/test/index.ts`, which re-exports
  `@base-ui/utils/testUtils` (including `isJSDOM`), `advanceReactClock`, `createRenderer`,
  `describeConformance`, the pointer helpers (`enterWithMouse`, `firePointer`, `moveMouse`),
  `mergeRefs`, `popupConformanceTests`, `resetBrowserPointer`, `useTestInteractions`, `./wait`,
  `waitForPositioned`, and `describeGregorianAdapter` `packages/react/test/index.ts:1-14`.
  - `isJSDOM` is `/jsdom/.test(window.navigator.userAgent)` `packages/utils/src/testUtils.ts:4`;
    used for environment gating by `packages/react/src/utils/getPseudoElementBounds.test.ts:2`, `packages/react/src/utils/popups/popupTriggerMap.test.ts:2`,
    and `packages/react/src/utils/useSwipeDismiss.test.tsx:4`.
  - `createRenderer` wraps `@mui/internal-test-utils`'s renderer, awaiting `act` around
    `render`/`rerender`/`setProps`     `packages/react/test/createRenderer.ts:27-49`; used by
    `packages/react/src/utils/listbox-separator/ListboxSeparator.test.tsx:10`, `packages/react/src/utils/useIsHydrating.test.tsx:8`, `packages/react/src/utils/useRegisteredLabelId.test.tsx:8`,
    and `packages/react/src/utils/useSwipeDismiss.test.tsx:54`.
  - `describeConformance` runs the "Base UI component API" suite (propsSpread, refForwarding,
    renderProp, className) `packages/react/test/describeConformance.tsx:44-70`; used only by
    `packages/react/src/utils/listbox-separator/ListboxSeparator.test.tsx:12-15` with `refInstanceof: window.HTMLDivElement`.
  - `firePointer` fires pointer events that honor an explicit `timeStamp` (rejecting falsy stamps)
    so velocity logic is deterministic; it is required by lint for timing-dependent tests
    `packages/react/test/pointer.ts:23-61`; used by the three velocity tests in
    `packages/react/src/utils/useSwipeDismiss.test.tsx:4` and `packages/react/src/utils/useSwipeDismiss.test.tsx:1420-1467`.
- `@mui/internal-test-utils` (external npm package) is imported directly by several tests:
  `screen`/`waitFor`/`fireEvent`/`flushMicrotasks` `packages/react/src/utils/useSwipeDismiss.test.tsx:3`,
  `flushMicrotasks` `packages/react/src/utils/popups/popupStoreUtils.test.tsx:4`,
  `screen`/`waitFor` `packages/react/src/utils/useIsHydrating.test.tsx:3`,
  `screen` `packages/react/src/utils/useRegisteredLabelId.test.tsx:3`,
  `screen` `packages/react/src/utils/listbox-separator/ListboxSeparator.test.tsx:5`.
- `@testing-library/react` primitives (`act`, `render`, `screen`, `waitFor`) are imported directly
  in `popupStoreUtils.test.tsx` `packages/react/src/utils/popups/popupStoreUtils.test.tsx:3`.
- No test in this unit reads any other component's test files; the only cross-unit imports are of
  source modules under test or referenced infrastructure (`@base-ui/utils/store`,
  `@base-ui/utils/useIsoLayoutEffect`, `../../floating-ui-react`, `../../internals/*` —
  `packages/react/src/utils/popups/popupStoreUtils.test.tsx:5-6,21-24`) and the external
  `@floating-ui/react-dom` used as a parity oracle by `packages/react/src/utils/hideMiddleware.test.ts:2-8`.
