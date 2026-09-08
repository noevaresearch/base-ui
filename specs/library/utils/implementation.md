# Implementation spec — `infra: utils` (`packages/react/src/utils/`)

Companion to `specs/library/utils/behavior.md` (the WHAT). This document explains the state
machines, hook composition, context usage, and DOM decisions that produce the documented
behavior; it does not re-describe tested behavior — refer to behavior.md by section name.
The unit has no `wraps-external:` field (`TODO.md:291-297`), so nothing is delegated to an
external package: every dependency listed here is an in-repo internal (or npm package) that a
port must carry with it. The directory is a grab-bag of independent utilities, so sections are
grouped per subsystem rather than per file.

## State machine / hooks used

### Popup store core (`popups/`)

`store.ts` is a plain state shape + selector table with no hooks of its own
(`packages/react/src/utils/popups/store.ts:12-81`, `packages/react/src/utils/popups/store.ts:168-206`).
All reactivity lives in the `ReactStore` from `@base-ui/utils/store`; selectors are pure
functions over `PopupStoreState`, each layering external-prop override over internal state:
`openProp ?? open` (`packages/react/src/utils/popups/store.ts:142`),
`triggerIdProp ?? activeTriggerId` (`packages/react/src/utils/popups/store.ts:140`),
`popupElement?.id ?? floatingId` (`packages/react/src/utils/popups/store.ts:144-147`).

Hooks in `popupStoreUtils.ts`:

- `usePopupRootStore` (`packages/react/src/utils/popups/popupStoreUtils.ts:73-96`) — store
  creation on behalf of a Root. `useId` supplies `floatingId`, `useFloatingParentNodeId`
  detects nesting, and `useRefWithInit` guarantees the factory runs exactly once
  (`packages/react/src/utils/popups/popupStoreUtils.ts:81-84`); `useSyncedFloatingRootContext`
  then bridges the popup store to the floating root context so controlled props and open
  changes flow both ways (`packages/react/src/utils/popups/popupStoreUtils.ts:86-93`).
- `PopupHandleAttachment` (`packages/react/src/utils/popups/popupStoreUtils.ts:108-120`) — a
  null-rendering component whose `useIsoLayoutEffect` attaches the Root's store to a handle.
  It must be an ordinary layout effect (not render-phase) so a suspended/abandoned store never
  leaks onto the handle, and so descendants calling the handle during the same commit see it
  attached (`packages/react/src/utils/popups/popupStoreUtils.ts:99-107`).
- `useTriggerRegistration` (`packages/react/src/utils/popups/popupStoreUtils.ts:144-183`) — a
  `useStableCallback` ref-callback that tracks the live registration as a
  `(store, id, element)` triple in a ref, so unregistering targets the store the element was
  actually registered in (`packages/react/src/utils/popups/popupStoreUtils.ts:148-175`). The
  callback is stable precisely because downstream ref mergers retain the first identity they
  were given (`packages/react/src/utils/popups/popupStoreUtils.ts:130-138`).
- `useTriggerDataForwarding` (`packages/react/src/utils/popups/popupStoreUtils.ts:318-388`) —
  composes registration with active-trigger data claiming. Subscribes via
  `store.useState('isMountedByTrigger', triggerId)`; two `useIsoLayoutEffect`s handle
  (a) migration of an already-registered element across store/id changes
  (`packages/react/src/utils/popups/popupStoreUtils.ts:371-374`) and (b) refreshing
  `activeTriggerElement` + trigger-owned state while mounted
  (`packages/react/src/utils/popups/popupStoreUtils.ts:376-385`).
- `useImplicitActiveTrigger` (`packages/react/src/utils/popups/popupStoreUtils.ts:416-537`) —
  the Root-side reconciliation loop. Subscribes to `open`, `triggerCount`,
  `activeTriggerId`, and `activeTriggerElement`; one `useIsoLayoutEffect` recomputes ownership
  on every change (`packages/react/src/utils/popups/popupStoreUtils.ts:437-536`). Key
  mechanism: `resolvedActiveTriggerIdRef` distinguishes "active trigger unmounted" from
  "pending id that has not matched a registered trigger yet"
  (`packages/react/src/utils/popups/popupStoreUtils.ts:424`, `packages/react/src/utils/popups/popupStoreUtils.ts:471-477`);
  a lost active trigger defers the close through `queueMicrotask` so a same-tick replacement
  trigger can register first (`packages/react/src/utils/popups/popupStoreUtils.ts:509-527`).
  When open with no active trigger and exactly one registration, that trigger is claimed
  implicitly (`packages/react/src/utils/popups/popupStoreUtils.ts:488-496`).
- `useOpenStateTransitions` (`packages/react/src/utils/popups/popupStoreUtils.ts:554-601`) —
  mounts/unmounts the popup: wraps `useTransitionStatus` (internals), syncs
  `mounted`/`transitionStatus`/`preventUnmountingOnClose` into the store via
  `useSyncedValues` (`packages/react/src/utils/popups/popupStoreUtils.ts:571-575`), and drives
  unmount through `useOpenChangeComplete` on the popup ref
  (`packages/react/src/utils/popups/popupStoreUtils.ts:589-598`). `forceUnmount` clears
  active-trigger ownership and notifies `onOpenChangeComplete(false)`
  (`packages/react/src/utils/popups/popupStoreUtils.ts:577-587`).
- `usePopupInteractionProps` (`packages/react/src/utils/popups/popupStoreUtils.ts:605-624`) —
  `useSyncedValues` write-through plus an unmount-only `useIsoLayoutEffect` that resets all
  three props bags to `EMPTY_OBJECT` (behavior.md, State model → `usePopupInteractionProps`).
- `usePopupRootSync` (`packages/react/src/utils/popups/popupStoreUtils.ts:626-645`) — two
  `useIsoLayoutEffect`s clearing the component-specific `openMethod` state on close and on
  unmount; not covered by any unit test (see the last section).
- `applyPopupOpenChange` (`packages/react/src/utils/popups/popupStoreUtils.ts:241-308`) — the
  shared open-change sequence (order documented in behavior.md, Events →
  `applyPopupOpenChange`). Implementation details beyond the tests: `attachPreventUnmountOnClose`
  installs a closure flag on the event details and returns a reader, so the
  `preventUnmountOnClose()` decision made inside `onOpenChange` is consumed after it returns
  (`packages/react/src/utils/popups/popupStoreUtils.ts:223-231`,
  `packages/react/src/utils/popups/popupStoreUtils.ts:266`,
  `packages/react/src/utils/popups/popupStoreUtils.ts:283`); the store commit for hover opens
  is flushed with `ReactDOM.flushSync` so `getAnimations()` observes the new state in the same
  task (`packages/react/src/utils/popups/popupStoreUtils.ts:302-307`).
- `createPopupOpenState` (`packages/react/src/utils/popups/popupStoreUtils.ts:190-221`) — the
  pure next-state builder tested directly (behavior.md, State model → `createPopupOpenState`).
  Why it preserves the previous trigger on close: exit animations and focus return need the
  old anchor (`packages/react/src/utils/popups/popupStoreUtils.ts:208-213`).

`popupHandle.ts` — `BasePopupHandle` is a class, not a hook. It owns: the attachment stack
(`attachedStores`, `packages/react/src/utils/popups/popupHandle.ts:78`), the current store
pointer + listener set (`packages/react/src/utils/popups/popupHandle.ts:84-89`),
`attachStore`/cleanup that restores the *previous* still-mounted root on detach
(`packages/react/src/utils/popups/popupHandle.ts:177-187`), a dev-only `AnimationFrame`
deferred warning for overlapping roots (`packages/react/src/utils/popups/popupHandle.ts:155-175`),
and `openByTrigger`, which resolves a trigger id by searching the whole attachment stack
newest-first and then the fallback store — needed because a detached trigger migrates its
registration one commit after the root attaches
(`packages/react/src/utils/popups/popupHandle.ts:235-241`). `serverStore` always returns the
fallback store because a handle can be shared by concurrent SSR requests and must never record
a live root store during render (`packages/react/src/utils/popups/popupHandle.ts:124-130`).

`usePopupHandleStore.ts` — `useSyncExternalStore` over the handle's store pointer
(`packages/react/src/utils/popups/usePopupHandleStore.ts:35`); the server snapshot reads
`handle.serverStore` (`packages/react/src/utils/popups/usePopupHandleStore.ts:35`). This is how
detached triggers follow a handle from fallback store to root store without any React context.

`popupTriggerMap.ts` — a `Map<string, Element>` plus a module-scoped dev-only
`WeakMap<map, WeakMap<Element, id>>` reverse index so duplicate-id registration is O(1) to
detect and is tree-shaken in production
(`packages/react/src/utils/popups/popupTriggerMap.ts:9-20`,
`packages/react/src/utils/popups/popupTriggerMap.ts:41-61`).

### Swipe-to-dismiss (`useSwipeDismiss.ts`)

React state is deliberately minimal: `currentSwipeDirection`, `isSwiping`, `dragDismissed`
(`packages/react/src/utils/useSwipeDismiss.ts:131-135`); everything else — start position,
offsets, baselines, thresholds, velocity samples, pointer-capture bookkeeping — lives in ~25
refs (`packages/react/src/utils/useSwipeDismiss.ts:137-162`) so gesture math never triggers
renders. All handlers are `useStableCallback` (`handleStart`
`packages/react/src/utils/useSwipeDismiss.ts:527`, `handleEnd`
`packages/react/src/utils/useSwipeDismiss.ts:748`, `handleMove`
`packages/react/src/utils/useSwipeDismiss.ts:862`, `setSwiping`
`packages/react/src/utils/useSwipeDismiss.ts:164`, `updateSwipeProgress`
`packages/react/src/utils/useSwipeDismiss.ts:190`, `syncDragStyles`
`packages/react/src/utils/useSwipeDismiss.ts:220`, `moveNative`
`packages/react/src/utils/useSwipeDismiss.ts:992`); the prop-factories (`getDragStyles`,
`getPointerProps`, `getTouchProps`, `reset`) are `useCallback`
(`packages/react/src/utils/useSwipeDismiss.ts:273`,
`packages/react/src/utils/useSwipeDismiss.ts:1002`,
`packages/react/src/utils/useSwipeDismiss.ts:1029`,
`packages/react/src/utils/useSwipeDismiss.ts:1042`).

The gesture is a state machine across four event handlers:

1. **Pending** — `pointerdown`/`touchstart` records the start position and evaluates `canStart`
   but does not yet start the gesture; actual start is deferred to the first move
   (`packages/react/src/utils/useSwipeDismiss.ts:545-563`).
2. **Started** — `startSwipeAtPosition` re-checks scrollable targets (touch only), the
   interactive-element ignore selector, and scrollable ancestors; snapshots the element's
   computed transform as the drag origin, captures the pointer, fires `onSwipeStart`, and
   flips `swiping` on (`packages/react/src/utils/useSwipeDismiss.ts:354-425`).
3. **Moving** — `handleMoveCore`: the first move only re-baselines (absorbing the iOS
   touchstart/touchmove gap; kept only when `trackDrag`, since a `trackDrag: false` consumer
   needs the full flick distance) (`packages/react/src/utils/useSwipeDismiss.ts:587-603`);
   direction locks once both axes are allowed and movement exceeds 1px
   (`packages/react/src/utils/useSwipeDismiss.ts:630-636`); the intended direction is chosen
   once and latched (`packages/react/src/utils/useSwipeDismiss.ts:638-670`); movement backward
   past 10px from the max displacement marks a change-of-mind
   (`packages/react/src/utils/useSwipeDismiss.ts:677-685`); displacement in unsupported
   directions is damped exponentially rather than zeroed
   (`packages/react/src/utils/useSwipeDismiss.ts:469-482`).
4. **Released** — `handleEnd` computes gesture velocity from `timeStamp`s (clamped to a 50ms
   minimum) and release velocity from the last movement sample (16ms minimum, 80ms staleness
   cutoff) (`packages/react/src/utils/useSwipeDismiss.ts:783-811`); `onRelease` may override
   the threshold decision; a latched change-of-mind cancels unless overridden
   (`packages/react/src/utils/useSwipeDismiss.ts:825-847`). A trailing `buttons: 0`
   `pointermove` is treated as the release itself — its displacement is applied *before* the
   decision (`packages/react/src/utils/useSwipeDismiss.ts:883-897`,
   `packages/react/src/utils/useSwipeDismiss.ts:985-987`), and a non-primary button takeover
   cancels outright (`packages/react/src/utils/useSwipeDismiss.ts:878-881`).

`swipeThreshold` is snapshotted at gesture start (why mid-gesture changes don't apply —
behavior.md, State model → `useSwipeDismiss`): the numeric prop is copied into a ref at start
(`packages/react/src/utils/useSwipeDismiss.ts:398`) and a function threshold is resolved per
direction at latch time (`packages/react/src/utils/useSwipeDismiss.ts:174-188`,
`packages/react/src/utils/useSwipeDismiss.ts:668`). Drag styles are written imperatively with
snapshot/restore of the element's own inline `transition`/`transform`
(`packages/react/src/utils/useSwipeDismiss.ts:220-253`), and `getDragStyles()` reads the ref
rather than the lagging state so React reconciliation cannot strip the mid-gesture transform
(`packages/react/src/utils/useSwipeDismiss.ts:1002-1027`). Pointer capture is wrapped in a
try/catch that swallows only `NotFoundError` (`packages/react/src/utils/useSwipeDismiss.ts:66-84`).

### Viewport / auto-resize (`usePopupViewport.tsx` + `usePopupAutoResize.ts`)

`usePopupAutoResize` is a measurement cycle in one `useIsoLayoutEffect`
(`packages/react/src/utils/usePopupAutoResize.ts:46-150`), with `useAnimationsFinished`
(`packages/react/src/utils/usePopupAutoResize.ts:29`) and `useAnimationFrame`
(`packages/react/src/utils/usePopupAutoResize.ts:31`) from the internal hooks. Two paths:

- **Initial open** (per open, not per mount): override `position: static`, `transform: none`,
  and the positioner's available-size CSS vars, measure intrinsic size with
  `getCssDimensions`, commit it as the positioner's CSS-var size, restore
  (`packages/react/src/utils/usePopupAutoResize.ts:89-106`).
- **Content change while open**: measure the new intrinsic size, *freeze* the popup at the
  previous dimensions via the popup CSS vars, then in the next animation frame transition to
  the new size and reset the vars to `auto` once animations finish
  (`packages/react/src/utils/usePopupAutoResize.ts:108-139`).

`getPopupAnchoringStyles` pins the popup to the anchor edge for physical `top`/`left` sides so
size changes grow away from the anchor instead of shifting it
(`packages/react/src/utils/usePopupAutoResize.ts:188-203`). All sizing flows through the
`CommonPopupCssVars`/`CommonPositionerCssVars` custom properties
(`packages/react/src/utils/usePopupAutoResize.ts:228-240`).

`usePopupViewport` composes it and adds the morphing-container choreography
(`packages/react/src/utils/usePopupViewport.tsx:75-294`). It subscribes to seven store fields
(`packages/react/src/utils/usePopupViewport.tsx:80-86`), tracks the previous open trigger with
`usePreviousValue` (`packages/react/src/utils/usePopupViewport.tsx:88`), and derives a remount
key from active-trigger-id + payload so a lagging payload forces one extra DOM subtree
replacement (`packages/react/src/utils/usePopupViewport.tsx:367-396`). On trigger change, the
layout effect stores the captured DOM clone as `previousContentNode`, computes the
center-to-center offset between triggers for `data-activation-direction`
(`packages/react/src/utils/usePopupViewport.tsx:156-176`,
`packages/react/src/utils/usePopupViewport.tsx:345-362`), and the render output switches to
`data-previous`/`data-current` siblings with `data-starting-style`/`data-ending-style`
choreography (`packages/react/src/utils/usePopupViewport.tsx:227-265`). The unkeyed
`useIsoLayoutEffect` clones the current container's DOM into a wrapper after *every* commit —
DOM clones, not React elements, because previous content may be stateful
(`packages/react/src/utils/usePopupViewport.tsx:204-224`), and a second effect moves the clone
into the previous container (`packages/react/src/utils/usePopupViewport.tsx:268-275`). Cleanup
is armed after animations finish via `AbortController` + `useAnimationFrame`, and is re-armed
if the current container remounts mid-transition (`packages/react/src/utils/usePopupViewport.tsx:184-202`,
`packages/react/src/utils/usePopupViewport.tsx:137-146`). The `adaptiveOrigin` middleware is
registered into the store for the viewport's lifetime
(`packages/react/src/utils/usePopupViewport.tsx:112-117`).

### Interaction-type & click-semantics hooks

- `useOpenInteractionType` (`packages/react/src/utils/useOpenInteractionType.ts:44-62`) —
  `useState` for the open method; props derive from `useEnhancedClickHandler` (pointerdown +
  click pair from `@base-ui/utils`) (`packages/react/src/utils/useOpenInteractionType.ts:28`);
  `useValueChanged` resets the method on close (`packages/react/src/utils/useOpenInteractionType.ts:49-53`).
  The iOS Safari fallback maps an empty `interactionType` (hitslop `mousedown` without
  `pointerdown`) to `'touch'` (`packages/react/src/utils/useOpenInteractionType.ts:17-23`).
- `useMixedToggleClickHandler` (`packages/react/src/utils/useMixedToggleClickHandler.ts:12-43`) —
  an `ignoreClickRef` plus a document-level `once: true` `click` listener armed on `mousedown`;
  the armed `click` calls `event.preventBaseUIHandler()` so the closing half of a mixed
  mousedown/click toggle doesn't immediately reopen. The once-listener is what self-cleans
  without effect bookkeeping (`packages/react/src/utils/useMixedToggleClickHandler.ts:22-33`).
- `useFocusableWhenDisabled` (`packages/react/src/utils/useFocusableWhenDisabled.ts:4-61`) —
  pure `React.useMemo` prop derivation with no state: native buttons get the `disabled`
  attribute unless `focusableWhenDisabled` (then `aria-disabled` + a `Tab`-only `onKeyDown`
  guard, `packages/react/src/utils/useFocusableWhenDisabled.ts:23-27`,
  `packages/react/src/utils/useFocusableWhenDisabled.ts:45-47`); non-native buttons always get
  `aria-disabled` and a `tabIndex` of -1 when disabled and not focusable-when-disabled
  (`packages/react/src/utils/useFocusableWhenDisabled.ts:30-36`). `undefined` is never
  assigned so merged props can still set these keys
  (`packages/react/src/utils/useFocusableWhenDisabled.ts:18-19`).

### Environment / label / scroll-lock hooks

- `useIsHydrating` (`packages/react/src/utils/useIsHydrating.ts:20-22`) — `useSyncExternalStore`
  with a constant `false` client snapshot, constant `true` server snapshot, and a `NOOP`
  subscribe (`packages/react/src/utils/useIsHydrating.ts:4-14`). The hydration signal is
  React's own server→client snapshot swap, not an event; the "store" never changes.
- `useRegisteredLabelId` (`packages/react/src/utils/useRegisteredLabelId.ts:6-20`) —
  `useBaseUiId` for the label id, then a `useIsoLayoutEffect` that pushes it into the parent's
  `setLabelId` state; the cleanup only clears *its own* registration by comparing current value
  to its id, which is what makes late-arriving labels safe (behavior.md, Edge cases →
  `useRegisteredLabelId`).
- `useAnchoredPopupScrollLock` (`packages/react/src/utils/useAnchoredPopupScrollLock.ts:18-43`) —
  `useState` + `useIsoLayoutEffect` measuring the positioner against the viewport
  `clientWidth` with a 20px tolerance (`packages/react/src/utils/useAnchoredPopupScrollLock.ts:11`,
  `packages/react/src/utils/useAnchoredPopupScrollLock.ts:26-40`), feeding `useScrollLock`
  (`packages/react/src/utils/useAnchoredPopupScrollLock.ts:42`). Touch opens only lock scroll
  when the popup is effectively full-width, preserving outside-swipe dismissal otherwise.
- `useTriggerFocusGuards` (`packages/react/src/utils/popups/useTriggerFocusGuards.ts:38-95`) —
  not a state machine but two focus handlers for the trigger's pre/post focus guards:
  `flushSync` close on guard focus, then manual `getTabbableBeforeElement`/`getTabbableAfterElement`
  relocation, including a skip loop that walks past tabbables inside the positioner
  (`packages/react/src/utils/popups/useTriggerFocusGuards.ts:78-90`), and a rerouting branch
  when focus lands outside the positioner
  (`packages/react/src/utils/popups/useTriggerFocusGuards.ts:62-66`).
- `useClosePartCount`/`useClosePartRegistration` (`packages/react/src/utils/closePart.tsx:12-37`) —
  count-up/count-down ref-counting state with a `useStableCallback` register returning its own
  cleanup; children register in a `useIsoLayoutEffect`.

### Render helpers (stateless)

- `ListboxSeparator` (`packages/react/src/utils/listbox-separator/ListboxSeparator.tsx:12-25`)
  and `usePositioner` (`packages/react/src/utils/usePositioner.tsx:23-44`) are thin
  `useRenderElement` wrappers; the positioner adds `role="presentation"`, `hidden`, inert
  pointer-events, mount-transition suppression, and `popupStateMapping`
  (`packages/react/src/utils/usePositioner.tsx:34-43`).
- `FocusGuard` (`packages/react/src/utils/FocusGuard.tsx:10-42`) — visually hidden focusable
  span; `role="button"` is applied only when VoiceOver + WebKit are detected in a
  `useIsoLayoutEffect` (`packages/react/src/utils/FocusGuard.tsx:16-24`), because VoiceOver's
  virtual cursor only fires `onFocus` on role-button elements.
- `FloatingPortalLite` (`packages/react/src/utils/FloatingPortalLite.tsx:15-38`) delegates node
  creation to `useFloatingPortalNode` and portals children into it.
- `InternalBackdrop` (`packages/react/src/utils/InternalBackdrop.tsx:6-35`) computes a clip-path
  polygon from the `cutout` element's rect *during render*
  (`packages/react/src/utils/InternalBackdrop.tsx:13-16`) — a deliberate render-phase DOM read
  for a fixed-inset, non-interactive layer.

## Context providers/consumers

- `ClosePartContext` (`packages/react/src/utils/closePart.tsx:10`) is the only true React
  context defined in this unit. The provider side is `useClosePartCount` (consumed by
  `PopoverPopup` in `packages/react/src/popover/popup/PopoverPopup.tsx`), which publishes
  `{ register }` and derives `hasClosePart`; the consumer side is `useClosePartRegistration`
  (consumed by `PopoverClose` in `packages/react/src/popover/close/PopoverClose.tsx`), which
  registers on mount. What crosses the boundary: a stable register function; what flows back:
  a boolean telling the popup whether any Close part is rendered inside it.
- Popup stores cross component boundaries as plain objects, not through any context defined
  here. `PopupStoreContext` (`packages/react/src/utils/popups/store.ts:119-136`) is the store's
  *context* type — the mutable `triggerElements` map, `popupRef`, and the `onOpenChange`/
  `onOpenChangeComplete` callbacks — shared by a Root with its Trigger/Popup/Positioner/Viewport
  parts. Each popup family (Dialog, Popover, Menu, Tooltip, PreviewCard) creates its own React
  context for the store; this unit only defines the shape, the selectors, and the hooks that
  read/write it.
- The handle boundary is subscription-based: `PopupHandleStoreProvider`
  (`packages/react/src/utils/popups/popupHandle.ts:17-36`) + `usePopupHandleStore` let detached
  triggers re-render when the exposed store pointer flips between fallback and root store.
  Crossing the boundary: the store object itself; consumers: the Trigger parts of the five
  popup families via `usePopupHandleStore`.
- One data flow crosses through store *state* rather than props: `usePopupViewport` registers
  the `adaptiveOrigin` middleware into the store
  (`packages/react/src/utils/usePopupViewport.tsx:112-117`) and the positioner machinery reads
  `middlewareData.adaptiveOrigin`'s `sideX`/`sideY` for transform-origin
  (`packages/react/src/internals/useAnchorPositioning.ts:497`); each popup store declares the
  optional `adaptiveOrigin` field on its extended state (e.g.
  `packages/react/src/popover/store/PopoverStore.ts:36-59`).
- `popupStoreStateMapping`/`popupViewportStateMapping` (`packages/react/src/utils/usePopupViewport.tsx:21-30`)
  are consumed by the four Viewport parts (Popover, Menu, Tooltip, PreviewCard) to translate
  the `activationDirection` state into `data-activation-direction`.

## DOM/portal strategy and why

- **Portal**: only `FloatingPortalLite` portals DOM
  (`packages/react/src/utils/FloatingPortalLite.tsx:32-37`). It exists as the cheap variant of
  `FloatingPortal` specifically because it omits the tabbable/focus-management logic — used by
  Tooltip, PreviewCard, and Toast portals, where focus is managed by hooks instead
  (`packages/react/src/utils/FloatingPortalLite.tsx:10-14`). The `container` prop accepts an
  element, a `ShadowRoot`, or a ref to either
  (`packages/react/src/utils/FloatingPortalLite.tsx:7-8`), which is how popup content mounts
  inside shadow roots.
- **Positioner/popup split**: `usePositioner` renders the outer `div[role="presentation"]` that
  Floating UI actually positions; the popup element is its child. `inert` positioners get
  `pointerEvents: none` so the whole subtree stops receiving pointer events while still
  rendering (`packages/react/src/utils/usePositioner.tsx:30-32`), and
  `getDisabledMountTransitionStyles` suppresses transitions on the mount commit
  (`packages/react/src/utils/usePositioner.tsx:39`).
- **Imperative styling over React state** for high-frequency visual updates: swipe drag styles
  (`packages/react/src/utils/useSwipeDismiss.ts:220-253`) and auto-resize measurement
  (`packages/react/src/utils/usePopupAutoResize.ts:67-84`) write inline styles directly and
  restore originals afterwards, because per-move or per-measure renders would be too costly
  and would fight CSS transitions. Both snapshot the element's pre-existing inline styles so
  consumer-authored styles (e.g. `scale(0.9)`) survive drag/reset cycles.
- **Shadow-DOM-safe lookups**: `getElementAtPoint` takes a `getRootNode()` result (document or
  shadow root) rather than the owner document, because `Document.elementFromPoint` retargets
  shadow content to the host and would break `contains()` checks against a popup inside that
  root (`packages/react/src/utils/getElementAtPoint.ts:3-5`); `useSwipeDismiss` resolves the
  element at the gesture point through the swiped element's own root
  (`packages/react/src/utils/useSwipeDismiss.ts:323-328`). `scrollable.ts` uses floating-ui's
  `getParentNode` because it crosses shadow boundaries and slots, so scrollable ancestors in
  the light DOM are still found from shadow content
  (`packages/react/src/utils/scrollable.ts:46-48`, `packages/react/src/utils/scrollable.ts:66-68`).
- **Backdrop**: `InternalBackdrop` is a fixed-inset, `role="presentation"` layer marked
  `data-base-ui-inert` so Floating UI's outside-press detection treats it as having existed
  before the popup rendered (`packages/react/src/utils/InternalBackdrop.tsx:22-24`); the
  `cutout` prop punches a rectangular hole via `clip-path` so designated elements stay
  interactive (`packages/react/src/utils/InternalBackdrop.tsx:13-16`).
- **Focus guards**: `FocusGuard` spans are visually hidden but focusable (`tabIndex: 0`,
  `packages/react/src/utils/FocusGuard.tsx:26-30`); they are `aria-hidden` unless the VoiceOver
  role is applied (`packages/react/src/utils/FocusGuard.tsx:37`). The trigger-side guard
  handlers live in `useTriggerFocusGuards` (above).
- **Adaptive origin**: `adaptiveOrigin` is a Floating UI middleware that only acts when the
  popup has a CSS transition (`packages/react/src/utils/adaptiveOriginMiddleware.ts:20-29`);
  it re-derives x/y from the offset-parent (visual viewport for `fixed` strategy, document
  element or offset parent otherwise) so that for `left`/`top` sides the coordinate is
  expressed from the opposite edge (`packages/react/src/utils/adaptiveOriginMiddleware.ts:31-63`),
  letting a size transition keep its transform-origin visually anchored. The
  `AdaptiveOriginMiddleware` type is a self-contained stand-in for Floating UI's `Middleware`
  type purely to keep `tsc` declaration emit portable
  (`packages/react/src/utils/adaptiveOriginConstants.ts:6-12`).
- **Synthetic activation clicks**: `dispatchClickWithModifiers` constructs an untrusted
  `PointerEvent('click')` that preserves the source event's modifier state (which `click()`
  always drops) and defaults `detail: 0` (keyboard convention) with an opt-in `detail: 1` for
  mouse-gesture semantics (`packages/react/src/utils/dispatchClickWithModifiers.ts:10-35`) —
  the shared mechanism behind Checkbox/Radio/Switch/Menu item activation.

## Dependencies on other Base UI internals

No `wraps-external:` field exists for this unit (`TODO.md:291-297`), so all of the following
are dependencies a port must reproduce, not delegate.

**`@base-ui/utils/*` (packages/utils workspace package):**

- `owner` (`ownerDocument`/`ownerWindow`): `packages/react/src/utils/adaptiveOriginMiddleware.ts:1`,
  `packages/react/src/utils/getPseudoElementBounds.ts:1`,
  `packages/react/src/utils/getElementTransform.ts:1`,
  `packages/react/src/utils/dispatchClickWithModifiers.ts:1`,
  `packages/react/src/utils/useAnchoredPopupScrollLock.ts:3`,
  `packages/react/src/utils/useMixedToggleClickHandler.ts:3`,
  `packages/react/src/utils/useSwipeDismiss.ts:4`,
  `packages/react/src/utils/usePopupViewport.tsx:9`
- `useIsoLayoutEffect`: `packages/react/src/utils/closePart.tsx:3`,
  `packages/react/src/utils/FocusGuard.tsx:3`,
  `packages/react/src/utils/useAnchoredPopupScrollLock.ts:5`,
  `packages/react/src/utils/useRegisteredLabelId.ts:3`,
  `packages/react/src/utils/usePopupAutoResize.ts:4`,
  `packages/react/src/utils/usePopupViewport.tsx:7`,
  `packages/react/src/utils/popups/popupStoreUtils.ts:9`
- `useStableCallback`: `packages/react/src/utils/closePart.tsx:4`,
  `packages/react/src/utils/popups/popupStoreUtils.ts:8`,
  `packages/react/src/utils/useSwipeDismiss.ts:3`,
  `packages/react/src/utils/useOpenInteractionType.ts:3`,
  `packages/react/src/utils/usePopupAutoResize.ts:5`,
  `packages/react/src/utils/usePopupViewport.tsx:8`
- `useScrollLock`: `packages/react/src/utils/useAnchoredPopupScrollLock.ts:4`
- `useAnimationFrame` (`AnimationFrame`): `packages/react/src/utils/usePopupAutoResize.ts:3`,
  `packages/react/src/utils/usePopupViewport.tsx:5`,
  `packages/react/src/utils/popups/popupHandle.ts:1`
- `useEnhancedClickHandler` (`InteractionType`): `packages/react/src/utils/useOpenInteractionType.ts:4`,
  `packages/react/src/utils/popups/popupStoreUtils.ts:6`
- `platform`: `packages/react/src/utils/FocusGuard.tsx:4`,
  `packages/react/src/utils/getPseudoElementBounds.ts:2`,
  `packages/react/src/utils/useOpenInteractionType.ts:5`
- `visuallyHidden`: `packages/react/src/utils/FocusGuard.tsx:5`
- `inertValue`: `packages/react/src/utils/usePopupViewport.tsx:4`
- `usePreviousValue`: `packages/react/src/utils/usePopupViewport.tsx:6`
- `clamp`: `packages/react/src/utils/scrollEdges.ts:1`,
  `packages/react/src/utils/useSwipeDismiss.ts:5`
- `store` (`ReactStore`): `packages/react/src/utils/popups/store.ts:1`,
  `packages/react/src/utils/popups/popupStoreUtils.ts:4`,
  `packages/react/src/utils/NullStore.ts:1`
- `empty` (`EMPTY_OBJECT`/`NOOP`): `packages/react/src/utils/popups/store.ts:2`,
  `packages/react/src/utils/popups/popupStoreUtils.ts:5`,
  `packages/react/src/utils/useMixedToggleClickHandler.ts:4`,
  `packages/react/src/utils/usePopupAutoResize.ts:6`,
  `packages/react/src/utils/popups/usePopupHandleStore.ts:4`
- `useId`: `packages/react/src/utils/popups/popupStoreUtils.ts:7`
- `useRefWithInit`: `packages/react/src/utils/popups/popupStoreUtils.ts:10`

**`floating-ui-react` (in-repo vendored fork under `packages/react/src/floating-ui-react/`):**

- `FloatingRootContext` type + `FloatingRootStore`:
  `packages/react/src/utils/popups/store.ts:3-4`
- `useSyncedFloatingRootContext`: `packages/react/src/utils/popups/popupStoreUtils.ts:13-16`
- `useFloatingParentNodeId` (FloatingTree): `packages/react/src/utils/popups/popupStoreUtils.ts:12`
- `useFloatingPortalNode` (FloatingPortal): `packages/react/src/utils/FloatingPortalLite.tsx:5`
- utils barrel — `contains`, `getTarget`, `getNextTabbable`, `getTabbableBeforeElement`,
  `getTabbableAfterElement`, `isOutsideEvent`, `FOCUSABLE_ATTRIBUTE`:
  `packages/react/src/utils/popups/useTriggerFocusGuards.ts:4-11`,
  `packages/react/src/utils/useSwipeDismiss.ts:6`,
  `packages/react/src/utils/popups/popupStoreUtils.ts:11`
- `Middleware` type: `packages/react/src/utils/adaptiveOriginMiddleware.ts:3`
- `Dimensions` type: `packages/react/src/utils/usePopupAutoResize.ts:9`,
  `packages/react/src/utils/usePopupViewport.tsx:14`

**`internals/` (packages/react/src/internals/):**

- `useRenderElement` (+ `UseRenderElementComponentProps`):
  `packages/react/src/utils/usePositioner.tsx:3-6`,
  `packages/react/src/utils/listbox-separator/ListboxSeparator.tsx:4`
- `getDisabledMountTransitionStyles`: `packages/react/src/utils/usePositioner.tsx:7`
- `getStateAttributesProps` (`StateAttributesMapping` type):
  `packages/react/src/utils/collapsibleOpenStateMapping.ts:1`,
  `packages/react/src/utils/popupStateMapping.ts:1`
- `stateAttributesMapping` (`transitionStatusMapping`): `packages/react/src/utils/popupStateMapping.ts:2`
- `useTransitionStatus` (`TransitionStatus`): `packages/react/src/utils/popups/store.ts:5`,
  `packages/react/src/utils/popups/popupStoreUtils.ts:17`
- `useOpenChangeComplete`: `packages/react/src/utils/popups/popupStoreUtils.ts:18`
- `useAnimationsFinished`: `packages/react/src/utils/usePopupAutoResize.ts:7`,
  `packages/react/src/utils/usePopupViewport.tsx:11`
- `useValueChanged`: `packages/react/src/utils/useOpenInteractionType.ts:6`
- `useBaseUiId`: `packages/react/src/utils/useRegisteredLabelId.ts:4`
- `useAnchorPositioning` (`Side` type): `packages/react/src/utils/usePopupAutoResize.ts:10`,
  `packages/react/src/utils/usePopupViewport.tsx:15`
- `createBaseUIEventDetails` + `REASONS`: `packages/react/src/utils/popups/popupHandle.ts:3-6`,
  `packages/react/src/utils/popups/popupStoreUtils.ts:21-24`,
  `packages/react/src/utils/popups/useTriggerFocusGuards.ts:12-16`
- `TransitionStatusDataAttributes`: `packages/react/src/utils/CommonPopupDataAttributes.ts:1`
- `types` (`HTMLProps`, `BaseUIEvent`, `Orientation`, `BaseUIComponentProps`):
  `packages/react/src/utils/popups/store.ts:7`,
  `packages/react/src/utils/useMixedToggleClickHandler.ts:5`,
  `packages/react/src/utils/listbox-separator/ListboxSeparator.tsx:3`,
  `packages/react/src/utils/FloatingPortalLite.tsx:4`
- `noop` (`NOOP`): `packages/react/src/utils/useIsHydrating.ts:2`

**Other in-repo packages/components:**

- `direction-provider` (`useDirection`): `packages/react/src/utils/usePopupViewport.tsx:16`
- `collapsible` panel/trigger data attributes:
  `packages/react/src/utils/collapsibleOpenStateMapping.ts:2-3`

**Floating UI npm packages** (types and DOM helpers only; the one piece of Floating UI
*behavior* in this unit, `hideMiddleware`, is an in-repo reimplementation tested for parity
against the native middleware — behavior.md, Events → Other):

- `@floating-ui/react-dom` (`Middleware`, `VirtualElement`):
  `packages/react/src/utils/hideMiddleware.ts:1`, `packages/react/src/utils/popups/inlineRect.ts:2`
- `@floating-ui/utils` (`getSide`, `round`, `Dimensions`):
  `packages/react/src/utils/adaptiveOriginMiddleware.ts:2`,
  `packages/react/src/utils/getCssDimensions.ts:1`
- `@floating-ui/utils/dom` (`getComputedStyle`, `isHTMLElement`, `getParentNode`,
  `isLastTraversableNode`): `packages/react/src/utils/getCssDimensions.ts:2`,
  `packages/react/src/utils/scrollable.ts:1-6`

**Other npm:** `react-dom` (`createPortal`, `flushSync`):
`packages/react/src/utils/FloatingPortalLite.tsx:35`,
`packages/react/src/utils/popups/popupStoreUtils.ts:3`,
`packages/react/src/utils/usePopupViewport.tsx:3`,
`packages/react/src/utils/popups/useTriggerFocusGuards.ts:3`;
`use-sync-external-store/shim`: `packages/react/src/utils/useIsHydrating.ts:1`,
`packages/react/src/utils/popups/usePopupHandleStore.ts:3`.

**Downstream consumers (what this unit feeds):** the five popup families' stores and handles
(`NullStore`, `PopupTriggerMap`, `createInitialPopupStoreState`, `BasePopupHandle` subclasses —
Dialog, Popover, Menu, Tooltip, PreviewCard); their Positioners (`usePositioner`,
`useAnchoredPopupScrollLock`, `adaptiveOrigin`, `hideMiddleware` into
`internals/useAnchorPositioning`); their Triggers (`useOpenInteractionType`,
`useMixedToggleClickHandler`, `useTriggerFocusGuards`,
`getPseudoElementBounds`/`isMouseWithinBounds`); the four Viewport parts (`usePopupViewport`,
`popupViewportStateMapping`); Drawer (`useSwipeDismiss`, `scrollable`, `getElementAtPoint`,
`getElementTransform`); PreviewCard (`createInlineMiddleware`, `getInlineRectTriggerProps`,
`updateInlineRectCoords`); Tabs indicator (`getCssDimensions`, `getElementTransform`);
Slider/Progress/Meter/Fieldset (`valueToPercent`, `useRegisteredLabelId` via
`internals/labelable-provider`, `getDefaultLabelId`/`resolveAriaLabelledBy`); ScrollArea and
Select scrolling (`styleDisableScrollbar`, `scrollEdges`); `useRenderElement`
(`resolveClassName`, `resolveStyle`); `useButton` and Checkbox/Radio/Switch/Menu items
(`dispatchClickWithModifiers`, `useFocusableWhenDisabled`); Collapsible/Accordion
(`collapsibleOpenStateMapping`); Popover (`closePart`); Select/Autocomplete/Combobox
(`ListboxSeparator`).

## Anything in source not explained by any test

behavior.md's scope is this unit's twelve test files. Coverage below is verified by symbol
search across all `*.test.*` files in `packages/react/src` and `packages/utils/src`
("no test anywhere" = no test file references the symbol; component-level behavioral coverage
is noted where it exists but the unit spec has no record of it).

### Submodules with no test anywhere

- `adaptiveOriginMiddleware.ts` + `adaptiveOriginConstants.ts` — the transition gating
  (`packages/react/src/utils/adaptiveOriginMiddleware.ts:20-29`), offset-parent/visualViewport
  dimension resolution (`packages/react/src/utils/adaptiveOriginMiddleware.ts:31-49`), and the
  left/top coordinate flip + `sideX`/`sideY` report
  (`packages/react/src/utils/adaptiveOriginMiddleware.ts:51-63`) are entirely untested.
- `closePart.tsx` — count registration/cleanup, `hasClosePart` derivation: no test.
- `dispatchClickWithModifiers.ts` — modifier preservation and the `detail` convention
  (`packages/react/src/utils/dispatchClickWithModifiers.ts:19-35`): no test anywhere;
  behavior is only implied by Checkbox/Radio/Switch tests that consume it.
- `FocusGuard.tsx` — the VoiceOver/WebKit `role="button"` sniff
  (`packages/react/src/utils/FocusGuard.tsx:16-24`): no test (screen-reader-only path).
- `getElementAtPoint.ts` — root retargeting contract
  (`packages/react/src/utils/getElementAtPoint.ts:3-5`): no test.
- `getElementTransform.ts` — 6-value vs 16-value matrix parsing and scale extraction
  (`packages/react/src/utils/getElementTransform.ts:16-30`): no test; its longhand caveat
  (`translate`/`rotate`/`scale` not reflected in `transform`) is a documented-but-untested
  limitation.
- `scrollable.ts` — `allowOverflowIntent` mode (`packages/react/src/utils/scrollable.ts:18-20`),
  `hasScrollableAncestor` (`packages/react/src/utils/scrollable.ts:41-58`), and shadow-boundary
  traversal via `getParentNode`: no direct test; exercised only indirectly through
  `useSwipeDismiss` scroll-gating tests and Drawer/Select component tests.
- `styles.tsx` — the `styleDisableScrollbar` stylesheet injection with nonce/precedence
  (`packages/react/src/utils/styles.tsx:3-12`): no test.
- `NullStore.ts` — the inert-store contract (all mutators no-oped deliberately, including the
  base-class-routing caveat at `packages/react/src/utils/NullStore.ts:16-19`): no test.
- `useAnchoredPopupScrollLock.ts` — the 20px full-width tolerance rule
  (`packages/react/src/utils/useAnchoredPopupScrollLock.ts:11`): no test.
- `useFocusableWhenDisabled.ts` — the full `aria-disabled`/`disabled`/`tabIndex`/composite
  decision matrix (`packages/react/src/utils/useFocusableWhenDisabled.ts:20-58`): no test
  anywhere; behavior surfaces only through Button/Menu/Toolbar component tests.
- `useMixedToggleClickHandler.ts` — the ignore-click suppression window
  (`packages/react/src/utils/useMixedToggleClickHandler.ts:22-33`): no test.
- `useOpenInteractionType.ts` / `useOpenMethodTriggerProps` — the iOS fallback
  (`packages/react/src/utils/useOpenInteractionType.ts:17-23`) and method reset-on-close: no
  direct test; only observable via components' `openMethod`-driven styling.
- `useTriggerFocusGuards.ts` — tab-out close + focus relocation including the
  positioner-contained skip loop
  (`packages/react/src/utils/popups/useTriggerFocusGuards.ts:56-91`): no test.
- `popupHandle.ts` (`BasePopupHandle`) — attachment stack restore semantics
  (`packages/react/src/utils/popups/popupHandle.ts:177-187`), the dev overlap warning
  (`packages/react/src/utils/popups/popupHandle.ts:155-175`), the stack-wide trigger-id search
  (`packages/react/src/utils/popups/popupHandle.ts:235-241`), and `throwOnMissingTrigger`:
  no unit test; only exercised indirectly through component/detached-trigger tests
  (e.g. `MenuRoot.detached-triggers.test.tsx`).
- `usePopupHandleStore.ts` — store-pointer subscription and server snapshot: no test.
- `valueToPercent.ts` (`packages/react/src/utils/valueToPercent.ts:1-3`): no test (division by
  a `max === min` range is unguarded — behavior undefined).
- `resolveAriaLabelledBy.ts`, `resolveClassName.ts`, `resolveRef.ts`, `resolveStyle.ts` — no
  direct tests; covered implicitly through `useRenderElement`/labelable-provider component
  tests.
- `collapsibleOpenStateMapping.ts` and the five `Common*` constant modules — attribute/constant
  definitions with no logic; covered transitively by data-attribute assertions in component
  tests, but never asserted in this unit.

### Submodules with component-level coverage but no unit spec record

- `usePopupViewport.tsx` / `usePopupAutoResize.tsx` — the four Viewport parts have their own
  test files (`PopoverViewport.test.tsx`, `MenuViewport.test.tsx`, `TooltipViewport.test.tsx`,
  `PreviewCardViewport.test.tsx`), so the morphing choreography has *some* behavioral
  coverage, but none of the internal mechanisms documented above (clone-capture per commit,
  payload-lag double remount, starting-style re-arm, adaptiveOrigin registration) is asserted
  directly. Golden fixtures should treat the viewport choreography as under-specified.
- `usePositioner.tsx` — covered only via every popup family's Positioner tests.

### Tested modules with untested surface (gaps within the twelve files)

- `scrollEdges.getMaxScrollOffset` (`packages/react/src/utils/scrollEdges.ts:5-7`) — exported
  and consumed by Select (`SelectPopup`, `SelectScrollArrow`, `SelectRoot`) but absent from
  behavior.md and from `scrollEdges.test.ts`.
- `store.ts` selectors other than `isOpenedByTrigger`/`popupId`: `open`, `mounted`,
  `transitionStatus`, `floatingRootContext`, `triggerCount`, `preventUnmountingOnClose`,
  `payload`, `activeTriggerId`, the mounted-gating of `activeTriggerElement`
  (`packages/react/src/utils/popups/store.ts:178`), `isTriggerActive`, `triggerProps`,
  `popupElement`, `positionerElement`, and in particular `triggerPopupId` +
  `triggerOwnsOpenPopupOrIsOnlyTrigger` (`packages/react/src/utils/popups/store.ts:155-166`,
  `packages/react/src/utils/popups/store.ts:200-201`), whose "only trigger" fallback has no
  test. Also untested: `createInitialPopupStoreState`'s `FloatingRootStore` construction with
  `syncOnly: true` (`packages/react/src/utils/popups/store.ts:93-103`).
- `popupStoreUtils.ts` exports with no unit test: `usePopupRootStore`
  (`packages/react/src/utils/popups/popupStoreUtils.ts:73-96`), `PopupHandleAttachment`
  (`packages/react/src/utils/popups/popupStoreUtils.ts:108-120`), `useOpenStateTransitions`
  (`packages/react/src/utils/popups/popupStoreUtils.ts:554-601`), `usePopupRootSync`
  (`packages/react/src/utils/popups/popupStoreUtils.ts:626-645`),
  `createDefaultInitialFocus` (`packages/react/src/utils/popups/popupStoreUtils.ts:42-45`),
  `FOCUSABLE_POPUP_PROPS` (`packages/react/src/utils/popups/popupStoreUtils.ts:32-35`), and the
  `preventUnmountOnClose()` flag path through `applyPopupOpenChange` (only the cancellation
  short-circuit is tested; the flag reader at
  `packages/react/src/utils/popups/popupStoreUtils.ts:283` is not).
- `useSwipeDismiss.ts` options and returns with no test: `canStart`
  (`packages/react/src/utils/useSwipeDismiss.ts:550-558`,
  `packages/react/src/utils/useSwipeDismiss.ts:909-914`), `trackDrag: false`
  (`packages/react/src/utils/useSwipeDismiss.ts:222-227`,
  `packages/react/src/utils/useSwipeDismiss.ts:592-602`), `moveNative`
  (`packages/react/src/utils/useSwipeDismiss.ts:992-1000`), `ignoreSelectorWhenTouch`
  (`packages/react/src/utils/useSwipeDismiss.ts:378`), the interactive-element ignore selector
  (`packages/react/src/utils/useSwipeDismiss.ts:37`,
  `packages/react/src/utils/useSwipeDismiss.ts:377-380`), the `swipeDirection`/`dragDismissed`
  return fields (`packages/react/src/utils/useSwipeDismiss.ts:1057-1058`), the
  non-primary-button takeover cancel (`packages/react/src/utils/useSwipeDismiss.ts:878-881`),
  the scroll-edge *allowance* path (`canSwipeFromScrollEdgeOnPendingMove`,
  `packages/react/src/utils/useSwipeDismiss.ts:484-525` — tests cover only the gating side),
  the change-of-mind reversal threshold `REVERSE_CANCEL_THRESHOLD` affecting the release
  decision (`packages/react/src/utils/useSwipeDismiss.ts:677-685`,
  `packages/react/src/utils/useSwipeDismiss.ts:825-831`), and the release-velocity staleness
  zeroing (`packages/react/src/utils/useSwipeDismiss.ts:807-810`).
- `inlineRect.ts` — the ±2px hit slop in `findLineIndex`
  (`packages/react/src/utils/popups/inlineRect.ts:96-104`) is not documented or tested.
- `InternalBackdrop.tsx` — the render-phase `cutout` measurement and clip-path polygon
  (`packages/react/src/utils/InternalBackdrop.tsx:13-16`) is not asserted anywhere;
  DialogRoot tests exercise the backdrop but not the cutout math.
- `getCssDimensions.ts` — no direct test in this unit; only indirect numeric assertions via
  TabsIndicator tests.
- `FloatingPortalLite.tsx` — exercised by `FloatingPortal.test.tsx` at the shared-component
  level; no assertions specific to the Lite variant's contract (no tabbable logic).
