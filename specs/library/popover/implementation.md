# Popover — implementation spec (mined from source)

Unit: `popover` (packages/react/src/popover). Companion to `specs/library/popover/behavior.md`,
which is the ground truth for WHAT happens; this document explains the state machine, hook
composition, context usage, and DOM decisions that produce it. Claims cite non-test source files
with `` `path:line` ``. The TODO.md entry for this unit (TODO.md:455-461) has no `wraps-external:`
field, so no behavior is delegated to a third-party npm package (see Dependencies).

## State machine / hooks used

### Store-centric state model

The popover has no `useState` open/close state in any part. All state lives in a `PopoverStore`
instance — a class extending `ReactStore` from `@base-ui/utils/store`
(`packages/react/src/popover/store/PopoverStore.ts:77`) — created exactly once per Root and shared
by reference through `PopoverRootContext`. Parts subscribe to slices via the store's hook methods:

- `store.useState(selector)` — `ReactStore.useState`
  (`packages/utils/src/store/ReactStore.ts:171-186`); used everywhere, e.g. Root reads
  `open`/`mounted`/`payload` (`packages/react/src/popover/root/PopoverRoot.tsx:51-53`).
- `store.useControlledProp(key, prop)` (`packages/utils/src/store/ReactStore.ts:120`) — Root syncs
  the controlled `open` and `triggerId` props
  (`packages/react/src/popover/root/PopoverRoot.tsx:48-49`). The controlled/uncontrolled split is
  resolved inside selectors: `open = state.openProp ?? state.open` and
  `activeTriggerId = state.triggerIdProp ?? state.activeTriggerId`
  (`packages/react/src/utils/popups/store.ts:140-142`).
- `store.useContextCallback` (`packages/utils/src/store/ReactStore.ts:188`) — installs
  `onOpenChange`/`onOpenChangeComplete` into store context rather than render scope
  (`packages/react/src/popover/root/PopoverRoot.tsx:55-56`), so `setOpen` can invoke them from
  anywhere (including detached handles) without prop drilling.
- `store.useSyncedValue(s)` / `useSyncedValueWithCleanup` — parts publish render-derived values
  back into the store: `modal` (`packages/react/src/popover/root/PopoverRoot.tsx:64-66`),
  `focusManagerModal` (`packages/react/src/popover/popup/PopoverPopup.tsx:72`),
  `titleElementId`/`descriptionElementId`
  (`packages/react/src/popover/title/PopoverTitle.tsx:24`,
  `packages/react/src/popover/description/PopoverDescription.tsx:24`).
- `store.useStateSetter` (`packages/utils/src/store/ReactStore.ts:203`) — popup and positioner
  register their DOM elements (`packages/react/src/popover/popup/PopoverPopup.tsx:74`,
  `packages/react/src/popover/positioner/PopoverPositioner.tsx:134`).

The store is created by `usePopupRootStore` (`packages/react/src/popover/root/PopoverRoot.tsx:40`,
factory at `:119-121`; shared impl `packages/react/src/utils/popups/popupStoreUtils.ts:73-96`),
which resolves `floatingId` via `useId` and `nested` via `useFloatingParentNodeId()` on first
render, then wires the synced floating root context. Initial state comes from
`createInitialPopupStoreState` (`packages/react/src/utils/popups/store.ts:83-117`), which also
creates the `FloatingRootStore` (the floating-ui state machine the parts drive) and is overlaid
with popover-specific defaults in `createInitialState`
(`packages/react/src/popover/store/PopoverStore.ts:189-217` — note `open: true` forces
`mounted: true` so `defaultOpen` renders content on first paint, matching behavior.md's
`defaultOpen` section).

### The open-change pipeline (`PopoverStore.setOpen`)

`setOpen(nextOpen, eventDetails)` (`packages/react/src/popover/store/PopoverStore.ts:95-168`) is
the single mutation entry every interaction funnels into (trigger click/hover, Escape, outside
press, Close, `actionsRef.close()`, handle `open()`/`close()`):

1. Classifies the change: hover (`reason === 'triggerHover'`), keyboard click (`triggerPress` with
   `event.detail === 0`), dismiss-close (`escapeKey` or reason `null`)
   (`packages/react/src/popover/store/PopoverStore.ts:99-104`).
2. Attaches `eventDetails.preventUnmountOnClose()` via `attachPreventUnmountOnClose`
   (`packages/react/src/popover/store/PopoverStore.ts:106-108`;
   `packages/react/src/utils/popups/popupStoreUtils.ts:223-231`) — the deferred-unmount flag
   behind behavior.md's `preventUnmountOnClose()` section.
3. Backfills `eventDetails.trigger` for a `closePress` with no trigger (resolves registered
   active trigger → active element → undefined; `packages/react/src/popover/store/PopoverStore.ts:112-122`),
   matching the `onOpenChange` trigger fallback in behavior.md.
4. Calls `context.onOpenChange` and aborts if `eventDetails.isCanceled`
   (`packages/react/src/popover/store/PopoverStore.ts:124-128`) — this is the `cancel()` support.
5. Dispatches to the floating root context
   (`packages/react/src/popover/store/PopoverStore.ts:130`), which is what runs floating-ui's
   interaction hooks and focus manager side effects.
6. Commits `createPopupOpenState` (`packages/react/src/utils/popups/popupStoreUtils.ts:190-221`):
   clears any stale `preventUnmountingOnClose` on open, preserves the previous active
   trigger id/element when closing so exit animations and focus return target the right trigger.
7. Hover opens/closes are committed inside `ReactDOM.flushSync` so `getAnimations()` observes the
   attribute change, and every hover change arms the patient-click window: `stickIfOpen: true` +
   `stickIfOpenTimeout.start(PATIENT_CLICK_THRESHOLD, ...)` (500 ms,
   `packages/react/src/internals/constants.ts:4`) — the impatient/patient-click machinery
   (`packages/react/src/popover/store/PopoverStore.ts:146-157`).
8. Maps `instantType` (`click` for keyboard clicks, `dismiss` for dismiss-closes, `focus` for
   focus-out; `packages/react/src/popover/store/PopoverStore.ts:159-167`) — consumed by CSS via
   `data-instant` and by the positioner/viewport to skip transitions.

Notably, Popover does NOT use the shared `applyPopupOpenChange`
(`packages/react/src/utils/popups/popupStoreUtils.ts:241-308`) that Tooltip and PreviewCard use —
it keeps its own pipeline because of the patient-click `stickIfOpen` logic and the wider
dismiss-close classification (`reason == null` included). Any future store refactor must preserve
both divergences.

### Mounted/transition state machine

`useOpenStateTransitions(open, store, onUnmount)`
(`packages/react/src/popover/root/PopoverRoot.tsx:60-62`; impl
`packages/react/src/utils/popups/popupStoreUtils.ts:554-601`) owns `mounted`/`transitionStatus`:

- Wraps `useTransitionStatus` and syncs the results into the store; the synced
  `preventUnmountingOnClose` is forced false while open so each close cycle starts clean
  (`packages/react/src/utils/popups/popupStoreUtils.ts:566-575`).
- `useOpenChangeComplete` (ref = popup element) triggers `forceUnmount` after the exit
  animation — skipped entirely when `preventUnmountOnClose` was requested
  (`packages/react/src/utils/popups/popupStoreUtils.ts:589-598`), which is why a prevented close
  keeps the popup mounted (behavior.md).
- `forceUnmount` clears active-trigger ownership, unmounts, and fires
  `onOpenChangeComplete(false)` (`packages/react/src/utils/popups/popupStoreUtils.ts:577-587`);
  the Root's `onUnmount` callback additionally resets `stickIfOpen` and `openChangeReason`
  (`packages/react/src/popover/root/PopoverRoot.tsx:60-62`).
- The open-complete half of `onOpenChangeComplete` is fired separately by the Popup's own
  `useOpenChangeComplete` (`packages/react/src/popover/popup/PopoverPopup.tsx:56-64`).
- `actionsRef` exposes `unmount: forceUnmount` and `close: () => store.setOpen(false,
  imperativeAction)` via `React.useImperativeHandle`
  (`packages/react/src/popover/root/PopoverRoot.tsx:74-81`).

### Root hook inventory (`packages/react/src/popover/root/PopoverRoot.tsx`)

- `usePopoverRootStore` (`:40`, wrapper at `:112-127`) — also disposes the patient-click
  `Timeout` on unmount (`:124`; the `Timeout` itself is from `@base-ui/utils/useTimeout`,
  created in `createInitialContext`, `packages/react/src/popover/store/PopoverStore.ts:226`).
- `store.useControlledProp` ×2 (`:48-49`), `store.useState` ×3 (`:51-53`),
  `store.useContextCallback` ×2 (`:55-56`), `store.useSyncedValues({ modal })` (`:64-66`).
- `usePopupRootSync` (`:58`; `packages/react/src/utils/popups/popupStoreUtils.ts:626-645`) —
  resets `openMethod` to null on close and on store unmount so a stale interaction type cannot
  leak into the next open.
- `useImplicitActiveTrigger` (`:59`; `packages/react/src/utils/popups/popupStoreUtils.ts:416-537`)
  — layout-effect reconciliation over the trigger registry: claims the only registered trigger
  when none is active, refreshes the active element identity, reassociates an active id whose
  element re-registered under a different id, and (with
  `closeOnActiveTriggerUnmount`, off for popover) optionally closes. This is the engine behind
  behavior.md's detached-trigger robustness claims.
- `React.useEffect` clearing `stickIfOpenTimeout` whenever closed (`:68-72`).
- `PopoverInteractions` (`:227-260`), rendered only while `open || mounted` (`:83-88`):
  `useDismiss` from floating-ui-react with `outsidePressEvent` `intentional` for mouse in plain
  modal, `sloppy` under `'trap-focus'`, and `sloppy` for touch
  (`:236-243` — the comment explains it removes outside `aria-hidden` immediately when trapping
  focus), then publishes the dismiss prop bags into store state via `usePopupInteractionProps`
  (`:253-257`; `packages/react/src/utils/popups/popupStoreUtils.ts:605-624`). Trigger and Popup
  later read `triggerProps`/`popupProps` from the store
  (`packages/react/src/popover/trigger/PopoverTrigger.tsx:106`,
  `packages/react/src/popover/popup/PopoverPopup.tsx:43`) — this indirection is how dismissal
  handlers attach to elements rendered in different subtrees. The mount/unmount of
  `PopoverInteractions` is what makes dismissal "rewire" correctly across close/reopen cycles
  (behavior.md, Dismiss-interaction rewiring).
- `PopoverRoot` wraps itself in `<FloatingTree>` when there is no enclosing popup root context
  (`:100-110`) — the top-most popup in a tree registers the floating tree that nested popups
  (nested popovers, comboboxes, menus inside the popup) use for coordinated dismissal.

### Trigger hook composition (`packages/react/src/popover/trigger/PopoverTrigger.tsx`)

- Store resolution: `usePopoverRootContext(true)` (optional) ?? `usePopupHandleStore(handle)`
  (`:49-51`; `packages/react/src/utils/popups/usePopupHandleStore.ts:17-36` —
  `useSyncExternalStore` on the handle's store pointer). Missing both throws the error recorded
  in behavior.md (`:52-56`). Detached triggers therefore re-render and re-register when a root
  attaches/detaches.
- `useBaseUiId` for the trigger id (`:58`), then reactive selector reads: `isTriggerActive`,
  `floatingRootContext`, `isOpenedByThisTrigger`, `triggerPopupId` (`:59-62`; selectors in
  `packages/react/src/utils/popups/store.ts:183-201`, including the
  "owns popup or is only trigger" rule that gives a single trigger its `aria-controls`).
- `useTriggerDataForwarding` (`:66-76`; `packages/react/src/utils/popups/popupStoreUtils.ts:318-388`)
  — merges a stable registration ref into the element ref list; registration writes
  `payload`/`disabled`/`openOnHover`/`closeDelay` into the store when the trigger is active
  (`:70-75`), and an `useIsoLayoutEffect` migration moves the registration between stores when
  the handle pointer changes (`packages/react/src/utils/popups/popupStoreUtils.ts:371-374`).
- `useHoverReferenceInteraction` (`:83-96`) — trigger-side hover-open: `restMs: delay`
  (rest-type delay, default `OPEN_DELAY = 300` from
  `packages/react/src/popover/utils/constants.ts:1`), `delay: { close: closeDelay }`,
  `handleClose: safePolygon()`, `mouseOnly`, disabled while a touch-opened press is in progress,
  and gated on `transitionStatus !== 'ending'` so re-hover during the close transition takes the
  `mouseenter` fast path (behavior.md's reopen fast path).
- `useClick(floatingContext, { stickIfOpen })` (`:98`) — click-open whose behavior changes while
  the patient-click window is armed; the stickiness decision itself is state owned by the store.
- `useOpenMethodTriggerProps` (`:99-104`; `packages/react/src/utils/useOpenInteractionType.ts:8-41`)
  — enhanced click handler records whether the open was mouse/touch/keyboard (`openMethod`).
- `useButton` (`:108-111`) supplies native/non-native button semantics (`nativeButton`).
- State attributes: custom `stateAttributesMapping` picks
  `pressableTriggerOpenStateMapping` (adds `data-pressed`) only when the open reason is
  `triggerPress`, otherwise plain `triggerOpenStateMapping` (`:113-121`; mappings in
  `packages/react/src/utils/popupStateMapping.ts:30-46`) — the `data-popup-open`/`data-pressed`
  matrix in behavior.md.
- Focus guards: `useTriggerFocusGuards` (`:123-124`;
  `packages/react/src/utils/popups/useTriggerFocusGuards.ts:38-95`) — on pre-guard focus it
  `flushSync`-closes with `focusOut` and moves focus to the previous tabbable; on post-trigger
  guard focus it closes and walks forward past the positioner's tabbables (using
  `isOutsideEvent`/`contains`/`getNextTabbable` from floating-ui-react utils). Guards render as
  `FocusGuard` spans flanking the trigger only when this trigger mounted the popup and
  `focusManagerModal` is false (`:156-164`); a keyed fragment keeps the trigger's DOM node
  identical whether guards are present or not (`:152-154`).
- `focusManagerModal` is store state written by the Popup (`:71-72` below) — the guard placement
  switch behind behavior.md's "guards only inside the popup when a Close part is rendered".

### Popup hook composition (`packages/react/src/popover/popup/PopoverPopup.tsx`)

- `useClosePartCount` (`:37`; `packages/react/src/utils/closePart.tsx`) counts rendered
  `<Popover.Close>` parts; `focusManagerModal = modal !== false && hasClosePart` (`:71-72`) —
  focus trapping is enabled by a Close part, exactly as documented in the Root `modal` JSDoc.
- `useHoverFloatingInteraction` (`:66`) — floating-side hover-close with the active trigger's
  `closeDelay` (forwarded into the store by the trigger's registration), disabled for a disabled
  active trigger.
- `resolvedInitialFocus` defaults to `createDefaultInitialFocus(store.context.popupRef)`
  (`:68-69`; `packages/react/src/utils/popups/popupStoreUtils.ts:42-45`) — touch opens focus the
  popup element itself to avoid the virtual keyboard; everything else defers to the focus manager
  default (first tabbable), matching behavior.md's initialFocus section.
- `FloatingFocusManager` (`:108-123`) gets: `modal={focusManagerModal}`,
  `disabled={!mounted || openReason === REASONS.triggerHover}` (hover-opens never steal focus),
  `initialFocus`, `returnFocus={finalFocus}`, `restoreFocus="popup"`,
  `previousFocusableElement={activeTriggerElement}` and
  `nextFocusableElement={store.context.triggerFocusTargetRef}` (the tab-cycle endpoints that make
  Tab-out land after the trigger), and `beforeContentFocusGuardRef` (guard target when the
  positioner is exiting). These two refs live in store context
  (`packages/react/src/popover/store/PopoverStore.ts:41-43`) so trigger and popup can coordinate
  without prop drilling.
- Toolbar nesting: `useToolbarRootContext(true)` (`:36`) + `onKeyDown` stopping propagation of
  `COMPOSITE_KEYS` (`:95-99`) so arrow keys typed in the popup don't move a surrounding Toolbar
  (behavior.md, composite keys).
- `getDisabledMountTransitionStyles(transitionStatus)` (`:101`) suppresses mount animations and
  `popupTransitionStateMapping` (`:104`) drives `data-open`/`data-closed`/`data-starting-style`/
  `data-ending-style`.

### Positioner hook composition (`packages/react/src/popover/positioner/PopoverPositioner.tsx`)

- `useAnchorPositioning` (`:74-92`; impl `packages/react/src/internals/useAnchorPositioning.ts:128-633`)
  — the entire positioning state machine: builds a `useFloating` instance over the
  store-owned `floatingRootContext`, configures offset/flip/shift/size/arrow/hide middleware from
  props (incl. `collisionAvoidance`, defaulting to `POPUP_COLLISION_AVOIDANCE`,
  `packages/react/src/internals/constants.ts:26-28`), resolves logical sides via `useDirection`,
  writes the positioner CSS vars, and returns
  `{ positionerStyles, arrowStyles, arrowRef, arrowUncentered, side, align, physicalSide,
  anchorHidden, refs, context, isPositioned, update }`
  (`packages/react/src/internals/useAnchorPositioning.ts:604-631`). The `context` returned here
  is exactly what `PopoverPositionerContext` hands to the Popup for `FloatingFocusManager`.
- Trigger-change effect (`:98-123`) — `useIsoLayoutEffect` comparing the previous and current
  `domReferenceElement`; on change it clears `instantType` (so the move animates) and re-arms
  `'trigger-change'` once `useAnimationsFinished` reports the popup settled. This is the
  mechanism behind behavior.md's "popup carries no inline scale style after a trigger switch".
- `useAnchoredPopupScrollLock` (`:127-132`;
  `packages/react/src/utils/useAnchoredPopupScrollLock.ts:18-44`) — scroll lock only for
  `modal === true` non-hover opens, and for touch opens only when the positioner is within 20px
  of full viewport width (behavior.md's touch scroll-lock rules).
- `usePositioner` (`:144-151`; `packages/react/src/utils/usePositioner.tsx:23-44`) — shared
  positioner renderer: `role="presentation"`, `hidden={!mounted}`, `pointerEvents: none` while
  `inert={!open}`, mount-transition style suppression, and `popupStateMapping` attributes.
- `InternalBackdrop` (`:155-157`): for `modal === true` non-hover opens, a fixed full-viewport
  `role="presentation"` div with a polygon `clipPath` cut out around the trigger
  (`packages/react/src/utils/InternalBackdrop.tsx:16-35`) rendered *before* the `FloatingNode` so
  it is the positioner's previous sibling (behavior.md's internal backdrop assertion), carrying
  `data-base-ui-inert` so floating-ui's outside-press detection treats it correctly.
- `FloatingNode`/`useFloatingNodeId` (`:56,158`) register the positioner in the `FloatingTree`.

### Portal (`packages/react/src/popover/portal/PopoverPortal.tsx`)

- Renders nothing until `mounted || keepMounted` (`:24-27`), provides `PopoverPortalContext`
  (`keepMounted`) and delegates to `FloatingPortal` (`:29-33`), which resolves `container`
  (element or ShadowRoot, falling back to body) and `createPortal`s a div into it
  (`packages/react/src/floating-ui-react/components/FloatingPortal.tsx:105-142`).

### Viewport (`packages/react/src/popover/viewport/PopoverViewport.tsx`)

- Delegates to the shared `usePopupViewport` (`:28-32`;
  `packages/react/src/utils/usePopupViewport.tsx:75-396`): morph containers
  (`data-previous`/`data-current`), activation-direction tracking with tolerance, auto-resize via
  `usePopupAutoResize` (which writes `--popup-width`/`--popup-height`,
  `packages/react/src/utils/usePopupAutoResize.ts:129-130,231-232`), and — importantly for the
  positioner — injects the `adaptiveOrigin` middleware into the store while mounted
  (`packages/react/src/utils/usePopupViewport.tsx:112-117`), which the Positioner then passes to
  `useAnchorPositioning` (`packages/react/src/popover/positioner/PopoverPositioner.tsx:68,91`).
  This invisible store round-trip is how "positioner switches to top/left positioning when a
  Viewport is present" (behavior.md) is implemented.

### Small parts

- **Title/Description**: `useBaseUiId` + `store.useSyncedValueWithCleanup` publish their ids
  (`packages/react/src/popover/title/PopoverTitle.tsx:22-24`,
  `packages/react/src/popover/description/PopoverDescription.tsx:22-24`); the Popup consumes them
  as `aria-labelledby`/`aria-describedby` (`packages/react/src/popover/popup/PopoverPopup.tsx:93-94`).
- **Arrow**: pulls `arrowRef`/`side`/`align`/`arrowUncentered`/`arrowStyles` from
  `PopoverPositionerContext` and merges `arrowRef` into its ref list
  (`packages/react/src/popover/arrow/PopoverArrow.tsx:24-38`) — the arrow middleware measures this
  element through the same ref the positioner created.
- **Close**: `useClosePartRegistration` (`packages/react/src/popover/close/PopoverClose.tsx:37`)
  increments the popup's close-part count, and `onClick` calls
  `store.setOpen(false, createChangeEventDetails(REASONS.closePress, ...))` (`:43-45`).
- **Backdrop**: pure function of store state — `hidden={!mounted}`, `role="presentation"`,
  `pointerEvents: none` when opened by hover, and `userSelect`/`WebkitUserSelect: none`
  (`packages/react/src/popover/backdrop/PopoverBackdrop.tsx:38-46`).

### Handle and detached triggers

- `PopoverHandle` extends `BasePopupHandle`
  (`packages/react/src/popover/store/PopoverHandle.ts:12-18`) with
  `open(triggerId)` → `openByTrigger` and `close()` → `closePopup`
  (`packages/react/src/utils/popups/popupHandle.ts:215-287`); `isOpen` reads the attached store's
  `open` selector (`packages/react/src/popover/store/PopoverHandle.ts:44-46`).
- Its fallback store is `createNullPopoverStore` — a frozen `NullStore` view of popover state with
  `setOpen: NOOP` but a real trigger registry, so detached triggers can register before any root
  exists (`packages/react/src/popover/store/PopoverStore.ts:171-187`). The narrowed
  `PopoverHandleStore` view (`:72-75`) is the only surface detached triggers see.
- The Root attaches its store via the `PopupHandleAttachment` component rendered inside the
  provider (`packages/react/src/popover/root/PopoverRoot.tsx:87`;
  `packages/react/src/utils/popups/popupStoreUtils.ts:108-120`) — an ordinary layout effect so
  descendants registering in the same commit land on the live store.
- `BasePopupHandle` keeps an attachment stack (transient multi-root overlaps restore the previous
  root), warns on overlapping roots after a deferred animation frame, and resolves
  `open(id)` against the whole attachment stack plus the fallback registry before throwing
  (`packages/react/src/utils/popups/popupHandle.ts:74-187,215-265`) — the machinery behind
  behavior.md's detached-trigger robustness section.

## Context providers/consumers

Four React contexts cross part boundaries (all popover-internal, none shared with other
components):

1. **`PopoverRootContext`** — the `PopoverStore` instance itself is the context value
   (`packages/react/src/popover/root/PopoverRootContext.ts:5-7`). Provided by Root
   (`packages/react/src/popover/root/PopoverRoot.tsx:86`); consumed by every part via
   `usePopoverRootContext` (`:9-18`). Optional mode (`true`) is used only by Trigger (handle
   support) and by Root itself (nesting detection); all other parts hard-throw the
   "PopoverRootContext is missing" error recorded in behavior.md.
2. **`PopoverPositionerContext`** — `{ side, align, arrowRef, arrowUncentered, arrowStyles,
   context }` (`packages/react/src/popover/positioner/PopoverPositionerContext.ts:6-17`),
   provided by the Positioner (`packages/react/src/popover/positioner/PopoverPositioner.tsx:154`).
   Consumers: Popup (side/align state attributes, `:78-79`), Arrow (arrow wiring, `:24`), Viewport
   (side for morph anchoring, `packages/react/src/popover/viewport/PopoverViewport.tsx:24`). The
   `context` member is the floating-ui `FloatingContext` the Popup needs for
   `FloatingFocusManager` — the focus manager must be a *descendant* of the Positioner for
   floating-ui's node tree to be consistent, which is why Popup throwing outside Positioner is a
   hard error.
3. **`PopoverPortalContext`** — the `keepMounted` boolean
   (`packages/react/src/popover/portal/PopoverPortalContext.ts:4`), provided by Portal
   (`packages/react/src/popover/portal/PopoverPortal.tsx:30`), consumed by Positioner
   (`packages/react/src/popover/positioner/PopoverPositioner.tsx:55`) and forwarded into
   `useAnchorPositioning` (`:88`) so a kept-mounted popup keeps its collision/autoUpdate setup
   alive while closed.
4. **`ClosePartContext`** — close-part counting (`packages/react/src/utils/closePart.tsx`). The
   Popup re-provides the inner value around its element
   (`packages/react/src/popover/popup/PopoverPopup.tsx:37,122`) so nested popups inside the
   popup count their own Close parts independently; Close registers via
   `useClosePartRegistration`.

Cross-component reads *into* other components (popover as consumer): `useToolbarRootContext(true)`
in the Popup for composite-key suppression (`packages/react/src/popover/popup/PopoverPopup.tsx:36`),
and `useDirection` (DirectionProvider) inside `useAnchorPositioning` and `usePopupViewport` for
logical side resolution.

Two non-React channels cross the same boundaries: store **context fields** — `triggerElements`
(`PopupTriggerMap`), `popupRef`, `triggerFocusTargetRef`, `beforeContentFocusGuardRef`,
`stickIfOpenTimeout` (`packages/react/src/popover/store/PopoverStore.ts:39-44`) — connect trigger
and popup focus machinery; and store **state** carries trigger-owned data
(`payload`, `openOnHover`, `closeDelay`, `disabled`) from Trigger registration to Popup/Root
(`packages/react/src/popover/trigger/PopoverTrigger.tsx:70-75`), which is how the
children-as-function `{ payload }` render (`packages/react/src/popover/root/PopoverRoot.tsx:89`)
and popup-side hover-close tuning work for detached triggers.

## DOM/portal strategy and why

- **Portal layer**: `Popover.Portal` renders a plain `div` (the `FloatingPortal` element) into
  `document.body`, a caller-supplied element, or a `ShadowRoot`
  (`packages/react/src/floating-ui-react/components/FloatingPortal.tsx:105-142`). ShadowRoot
  support matters because dismissal, focus, and owner-document lookups throughout the stack go
  through floating-ui's shadow-safe utils. The portal element exists only while
  `mounted || keepMounted` (`packages/react/src/popover/portal/PopoverPortal.tsx:24-27`), so a
  closed non-keepMounted popover leaves nothing in the DOM; `keepMounted` flips positioning into
  a persistent mode via `PopoverPortalContext`.
- **Positioner as a separate element**: the positioner `div` (`role="presentation"`) owns the
  `position: absolute`-style inline positioning styles from floating-ui
  (`packages/react/src/utils/usePositioner.tsx:28-42`), while the Popup inside it is the
  `role="dialog"` element with the stable `floatingId`
  (`packages/react/src/popover/popup/PopoverPopup.tsx:90-91`). Splitting them lets the positioner
  stay `hidden` and `pointerEvents: none` during exit transitions
  (`packages/react/src/utils/usePositioner.tsx:38-41`) without unmounting the dialog, and gives
  CSS a stable transform origin variable set (`--transform-origin` etc.) to animate against —
  the transform-origin computation (anchor center / start / end / arrow / shifted) lives in
  `useAnchorPositioning`'s middleware, not in the component.
- **Trigger switching without remounts**: because the popup/positioner elements are keyed to the
  store, not to a trigger, switching `activeTriggerId` only changes floating-ui's reference
  element. The positioner's trigger-change effect re-arms `instantType` around the animation so
  the same DOM nodes glide between anchors (behavior.md's "reuses the same popup and positioner
  DOM nodes").
- **Modal infrastructure is positioner-local**: `InternalBackdrop` is rendered inside the
  Positioner subtree, immediately before the positioned node
  (`packages/react/src/popover/positioner/PopoverPositioner.tsx:153-159`), satisfying the
  "previous sibling" DOM relationship asserted in behavior.md while keeping the backdrop's
  lifecycle tied to `mounted && trueModalNonHover`. The trigger cut-out is recomputed from
  `triggerElement.getBoundingClientRect()` on each render
  (`packages/react/src/utils/InternalBackdrop.tsx:18-25`).
- **Focus guards around the trigger** are inline `FocusGuard` spans
  (`packages/react/src/utils/FocusGuard.tsx` — visually hidden, `tabIndex={0}`, optional
  `role="button"` for VoiceOver) rendered as trigger siblings inside a keyed fragment
  (`packages/react/src/popover/trigger/PopoverTrigger.tsx:152-164`). When the Popup detects a
  Close part (`focusManagerModal`), it syncs that into the store and the trigger stops rendering
  guards — the FloatingFocusManager renders inside-popup guards instead
  (`packages/react/src/popover/popup/PopoverPopup.tsx:120`), producing the two guard layouts
  behavior.md documents. The keyed fragment is what keeps the trigger's DOM node stable across
  that switch.
- **State attributes** come from `getStateAttributesProps` auto-mapping plus the shared popup
  mappings: `data-open`/`data-closed`/`data-starting-style`/`data-ending-style`
  (`packages/react/src/utils/popupStateMapping.ts:62-70`), `data-side`/`data-align`/
  `data-uncentered` on the arrow, `data-side`/`data-align`/`data-anchor-hidden` on the positioner,
  `data-popup-open`/`data-pressed` on the trigger, `data-instant` via fallback auto-serialization
  (`packages/react/src/internals/getStateAttributesProps.ts:17-24`).

## Dependencies on other Base UI internals

No `wraps-external:` field exists on this unit's TODO entry (TODO.md:455-461), so nothing here is
delegated to an external npm package and no Rust crate replacement is pending on that axis. The
only npm-external runtime dependencies reached by this unit are `@floating-ui/react-dom` +
`@floating-ui/utils` (re-exported through the vendored shim, below) and
`use-sync-external-store` (`packages/react/src/utils/popups/usePopupHandleStore.ts:3`).

Everything else is in-repo, in five layers:

1. **`floating-ui-react`** — a vendored fork of `@floating-ui/react` living at
   `packages/react/src/floating-ui-react/` (its `index.ts` re-exports positioning primitives from
   npm `@floating-ui/react-dom`). Popover imports:
   `useDismiss` + `FloatingTree` (Root, `packages/react/src/popover/root/PopoverRoot.tsx:4`),
   `useClick`/`safePolygon`/`useHoverReferenceInteraction` (Trigger,
   `packages/react/src/popover/trigger/PopoverTrigger.tsx:14`),
   `useHoverFloatingInteraction` + `FloatingFocusManager` (Popup,
   `packages/react/src/popover/popup/PopoverPopup.tsx:5`),
   `FloatingNode`/`useFloatingNodeId` (Positioner,
   `packages/react/src/popover/positioner/PopoverPositioner.tsx:5`),
   `FloatingPortal` (Portal, `packages/react/src/popover/portal/PopoverPortal.tsx:3`), plus the
   internal pieces the shared popup layer uses (`FloatingRootStore`,
   `useSyncedFloatingRootContext`, `useFloatingParentNodeId`, DOM/focus utils like `contains`,
   `getNextTabbable`, `isOutsideEvent`, `FOCUSABLE_ATTRIBUTE`).
   This is the largest single dependency: open/close dispatch, dismissal, hover, positioning, and
   focus management all terminate here.
2. **`utils/popups/`** — the cross-component popup layer shared with Menu, Dialog, Tooltip,
   PreviewCard: `usePopupRootStore`, `usePopupRootSync`, `useOpenStateTransitions`,
   `usePopupInteractionProps`, `useTriggerDataForwarding`, `useTriggerRegistration`,
   `useImplicitActiveTrigger`, `PopupHandleAttachment`, `usePopupHandleStore`, `BasePopupHandle`,
   `PopupTriggerMap`, `createPopupOpenState`, `attachPreventUnmountOnClose`,
   `createInitialPopupStoreState`, `popupStoreSelectors`, `FOCUSABLE_POPUP_PROPS`,
   `createDefaultInitialFocus`, `useTriggerFocusGuards` (call sites cited throughout the
   State machine section above). The `PopupStoreState`/`popupStoreSelectors` shape
   (`packages/react/src/utils/popups/store.ts:12-206`) is the contract every popup store shares;
   PopoverStore extends it with its own state keys (`packages/react/src/popover/store/PopoverStore.ts:24-37`).
3. **`internals/`**: `useRenderElement` (every part), `useButton` (Trigger, Close),
   `useAnchorPositioning` (Positioner), `useTransitionStatus`, `useOpenChangeComplete`,
   `useAnimationsFinished`, `getDisabledMountTransitionStyles`, `getStateAttributesProps`,
   `createBaseUIEventDetails` + `REASONS` (`packages/react/src/internals/reasons.ts`),
   `constants` (`PATIENT_CLICK_THRESHOLD`, `CLICK_TRIGGER_IDENTIFIER`,
   `POPUP_COLLISION_AVOIDANCE`), `composite` (`COMPOSITE_KEYS`), `useBaseUiId`, and the
   `types` (`BaseUIComponentProps`, `NativeButtonProps`).
4. **Component-agnostic `utils/`**: `usePositioner`, `usePopupViewport`, `usePopupAutoResize`,
   `useOpenInteractionType`, `popupStateMapping` (+ its `Common*DataAttributes` re-exports),
   `closePart`, `FocusGuard`, `InternalBackdrop`, `useAnchoredPopupScrollLock`, `NullStore`,
   `adaptiveOriginMiddleware`/`adaptiveOriginConstants` (the store-state-safe middleware type),
   `CommonPopupCssVars`/`CommonPositionerCssVars` (the CSS var names mirrored by this unit's
   `*CssVars.ts` files), `hideMiddleware`.
5. **`@base-ui/utils` (packages/utils)**: `store` (`ReactStore`), `useTimeout`, `useAnimationFrame`
   (handle overlap warning, `packages/react/src/utils/popups/popupHandle.ts:1,165`),
   `useStableCallback`, `useIsoLayoutEffect`, `useId`, `useRefWithInit`, `usePreviousValue`,
   `useValueAsRef`, `inertValue`, `empty` (`EMPTY_OBJECT`/`NOOP`), `owner` (`ownerDocument`,
   `ownerWindow`), `useScrollLock`, `useEnhancedClickHandler` (`InteractionType`), `platform`,
   `visuallyHidden`, `fastHooks` (`fastComponent`/`fastComponentRef` for Root/Trigger),
   `use-sync-external-store` shim (via `usePopupHandleStore`).

Cross-component coupling to record for dependency extraction: popover → toolbar
(`ToolbarRootContext`, read-only), popover → direction-provider/`DirectionContext`, and the
*reverse* edge that nested popovers depend on popover's `FloatingTree` + store
`floatingRootContext` (any component rendering inside a popover popup joins its floating tree).
For the future `blocked-by` computation: this unit is implementable only after `floating-ui-react`,
`utils/popups`, `useAnchorPositioning`, `useRenderElement`/`useButton`, and the `@base-ui/utils`
primitives exist; `Viewport` additionally requires `usePopupViewport`/`usePopupAutoResize`.

## Anything in source not explained by any test

Flagged explicitly; these are the gaps the golden-fixture stage should probe first:

- **Store `disabled` → popup hover-close gating.** The active trigger's `disabled` is forwarded
  into store state (`packages/react/src/popover/trigger/PopoverTrigger.tsx:70-75`) and read by the
  Popup to disable hover-close (`packages/react/src/popover/popup/PopoverPopup.tsx:52,66`). No
  test asserts that a disabled active trigger stops an open hover-popover from closing.
- **CSS vars beyond `--transform-origin`.** `PopoverPopupCssVars.ts` (`--popup-width`,
  `--popup-height`, written by `usePopupAutoResize`) and `PopoverPositionerCssVars.ts`
  (`--available-width`, `--available-height`, `--anchor-width`, `--anchor-height`,
  `--positioner-width`, `--positioner-height`, written via `CommonPositionerCssVars` inside
  `useAnchorPositioning`, `packages/react/src/internals/useAnchorPositioning.ts:37-38`) are
  declared as public API but only `--transform-origin` is asserted by tests;
  behavior.md:91 already marks `--available-height` UNVERIFIED — the same holds for all the rest.
- **`data-instant`.** Exported from `PopoverPopupDataAttributes.ts` but never added by a mapping;
  it reaches the DOM only through the fallback auto-serialization of the `instant` state key
  (`packages/react/src/internals/getStateAttributesProps.ts:17-24`). No popover test asserts
  `data-instant` in any form, including the keyboard-click → `'click'` mapping
  (`packages/react/src/popover/store/PopoverStore.ts:100-102,160-161`).
- **`data-uncentered` on Arrow** (`arrowUncentered` state,
  `packages/react/src/popover/arrow/PopoverArrow.tsx:30-31`) — no test asserts it; tests only
  cover transform-origin pointing at the arrow.
- **`data-anchor-hidden` on Positioner** (`anchorHidden` from the hide middleware,
  `packages/react/src/popover/positioner/PopoverPositioner.tsx:140` + mapping) — no popover test
  exercises anchor-hiding.
- **`data-base-ui-click-trigger` marker** on the trigger element
  (`packages/react/src/popover/trigger/PopoverTrigger.tsx:140`;
  `packages/react/src/internals/constants.ts:7`) — consumed by `FloatingFocusManager` and
  `DialogTrigger`, but no popover test asserts its presence.
- **`focusManagerModal` store round-trip.** The Popup computes focus-trap enablement and syncs it
  back to the store for the Trigger's guard placement
  (`packages/react/src/popover/popup/PopoverPopup.tsx:71-72` →
  `packages/react/src/popover/trigger/PopoverTrigger.tsx:81,156`). The two guard layouts are
  tested (behavior.md, modal focus guards), but the sync mechanism itself (e.g. that a popup
  without Close under `modal` leaves trigger guards in place) is only covered indirectly.
- **Type-only specs.** `packages/react/src/popover/root/PopoverRoot.spec.tsx` pins payload
  generics (children-as-function payload type, `createHandle<number>()` payload flow) and
  `packages/react/src/popover/positioner/PopoverPositioner.spec.tsx:3` pins that `keepMounted` is
  not a Positioner prop; neither is a runtime behavior and neither appears in behavior.md's
  claims.
- **Dead-looking exports.** `PopoverHandle` is exported twice from the parts barrel — as the
  `createHandle` factory and as the `Handle` type/value
  (`packages/react/src/popover/index.parts.ts:12-14`); tests only ever use `createHandle`, so the
  direct `Handle` export is unexercised.
