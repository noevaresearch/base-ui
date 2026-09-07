# Drawer — Implementation Spec (whole-unit index)

Stage 2 mining for `library: drawer`. Companion to `specs/library/drawer/behavior.md` (the WHAT;
part files under `specs/library/drawer/parts/`). This file explains the state machine, hook
composition, context graph, and DOM decisions that produce that behavior, citing only the unit's
source files (plus the internal modules named in the dependencies section). The unit has no
`wraps-external:` field in `TODO.md:375-382` (confirmed), so no third-party delegation applies —
everything below is derived from `packages/react/src/drawer/` itself.

The single most important implementation fact: **Drawer is a thin shell over Dialog.** The open
state machine, modal/dismiss/focus wiring, portal, and the Trigger/Close/Title/Description parts are
inherited wholesale from `dialog/`; the drawer-specific code is the snap-point machine, the two
`useSwipeDismiss` gesture surfaces, the nested-drawer coordination graph, the published visual-state
channels, and the virtual-keyboard coordinator.

## State machine / hooks used

### Open state: delegated to the Dialog store

`Drawer.Root` owns no open state. `useRenderDialogRoot('drawer', …)` (`packages/react/src/drawer/root/DrawerRoot.tsx:235-247`)
builds the `DialogStore`, mounts the dialog interactions, and renders the payload render-prop child;
the drawer result is wrapped in `DrawerRootContext.Provider` (`packages/react/src/drawer/root/DrawerRoot.tsx:249`).
Every drawer-side transition goes through `store.setOpen(false|true, createChangeEventDetails(REASONS.…))`
with a drawer-legal reason (`packages/react/src/drawer/root/DrawerRoot.tsx:464`,
`packages/react/src/drawer/viewport/DrawerViewport.tsx:633`,
`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:302-307`).

The one drawer-specific mutation on that machine is `handleOpenChange` (`packages/react/src/drawer/root/DrawerRoot.tsx:152-171`):
it forwards to the user's `onOpenChange` and, unless `eventDetails.isCanceled`, resets the active
snap point to the default when closing with snap points — re-using the same cancelable channel
(behavior.md, part `root`). This is also why `cancel()` semantics work for free: the details object
flows through Dialog's open pipeline (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:69-84`).

### Snap-point machine: `useControlled` + cancelable setter + reconciliation

- `useControlled({ controlled: snapPointProp, default: resolvedDefaultSnapPoint })` (`packages/react/src/drawer/root/DrawerRoot.tsx:71-76`),
  with `resolvedDefaultSnapPoint = defaultSnapPoint ?? snapPoints[0] ?? null` (`packages/react/src/drawer/root/DrawerRoot.tsx:67-68`).
- `setActiveSnapPoint` (`packages/react/src/drawer/root/DrawerRoot.tsx:81-96`) calls
  `onSnapPointChange` first and commits state only when `!details.isCanceled` — the mechanism behind
  the selective `cancel()` documented in behavior.md (part `root`).
- `resolvedActiveSnapPoint` (`packages/react/src/drawer/root/DrawerRoot.tsx:98-115`) reconciles:
  controlled values pass through; uncontrolled values that are `null` or absent from `snapPoints`
  fall back to the default. This reconciler is what makes "invalid entries dropped / duplicates keep
  the last" work at the state level — stale state is tolerated and resolution drops it later.
- Resolution/geometry lives in `useDrawerSnapPoints` (`packages/react/src/drawer/root/useDrawerSnapPoints.ts:84-197`),
  called independently by Viewport (`packages/react/src/drawer/viewport/DrawerViewport.tsx:97-104`)
  and Popup (`packages/react/src/drawer/popup/DrawerPopup.tsx:154`). It measures the viewport
  (`offsetHeight`, falling back to `documentElement.clientHeight`, plus root font-size for `rem`) via
  `useIsoLayoutEffect` + `ResizeObserver` (`packages/react/src/drawer/root/useDrawerSnapPoints.ts:92-116`),
  resolves each entry (`px`/`rem` strings, numbers ≤ 1 as viewport fractions, non-finite → null,
  `packages/react/src/drawer/root/useDrawerSnapPoints.ts:30-64`), clamps height to
  `min(popupHeight, viewportHeight)`, derives `offset = popupHeight − height`, and dedupes by height
  within 1px keeping the last (reverse scan, `packages/react/src/drawer/root/useDrawerSnapPoints.ts:145-160`).
  `getSnapPointSwipeMovement` (`packages/react/src/drawer/root/useDrawerSnapPoints.ts:21-28`) is the
  sqrt-damped overshoot used by both consumers.

### Nested-drawer coordination: a third, distributed machine

Root keeps `popupHeight`, `frontmostHeight`, `hasNestedDrawer`, `nestedSwiping` state plus a
`nestedSwipeProgressStore` external store (`packages/react/src/drawer/root/DrawerRoot.tsx:61-65`,
factory at `packages/react/src/drawer/root/DrawerRoot.tsx:393-417`). The store is
`useSyncExternalStore`-shaped (`getSnapshot`/`subscribe`/`set`) with NaN→0 normalization and
change-dedup; it exists so per-frame swipe progress can cross from a child drawer to its parent's
popup without re-rendering the parent tree (behavior.md, "Nested-drawer coordination"). The child→
parent binding is described under Context below.

### Gesture engines: two `useSwipeDismiss` instances with opposite roles

Both surfaces share one hook (`packages/react/src/utils/useSwipeDismiss.ts:86`), configured
differently:

- Viewport = dismiss engine (`packages/react/src/drawer/viewport/DrawerViewport.tsx:305-680`):
  imperative drag tracking on the popup ref (`movementCssVars` → `--drawer-swipe-movement-x/y`),
  `ignoreScrollableAncestors: true` (the viewport arbitrates scrolling itself), callbacks
  `canStart`/`onProgress`/`onRelease`/`onDismiss` implementing the release math from behavior.md
  (part `viewport`): fast-swipe cutoff, threshold (50% of size, min 10px,
  `packages/react/src/drawer/viewport/DrawerViewport.tsx:1070-1076`), velocity-projected target
  offset with `snapToSequentialPoints` adjacency forcing
  (`packages/react/src/drawer/viewport/DrawerViewport.tsx:554-618`), and reversal-velocity rejection
  (`packages/react/src/drawer/viewport/DrawerViewport.tsx:504-510`).
- SwipeArea = open engine (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:323-394`) with
  `trackDrag: false` (event-only; the area applies movement styles itself in `applySwipeMovement`,
  `packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:209-271`). It opens mid-drag: once the
  direction-attributed displacement passes 1px and the drawer is closed, `onProgress` calls
  `openDrawer` (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:356-364`); release commits
  by distance ratio or velocity (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:366-392`).

Two timing mechanisms implement behavior.md's "Layout-phase timing contract":

- `ReactDOM.flushSync` in `startSwipeRelease` (`packages/react/src/drawer/viewport/DrawerViewport.tsx:432-454`):
  `data-ending-style` + `data-swipe-dismiss` and the release scalar are committed synchronously at
  pointer-up so the popup never paints "stuck" between release and close-animation start.
- Controlled-mode optimistic rollback (`packages/react/src/drawer/viewport/DrawerViewport.tsx:654-675`):
  after `store.setOpen(false, …)`, an animation frame (`useAnimationFrame`, `controlledDismissFrame`,
  `packages/react/src/drawer/viewport/DrawerViewport.tsx:124`) re-checks `store.select('open')`; if
  the controlled parent rejected the close, the dismissal animation is reverted and the snap point
  saved in `pendingSwipeCloseSnapPointRef` (`packages/react/src/drawer/viewport/DrawerViewport.tsx:549`)
  is restored. The cancel path before that (`packages/react/src/drawer/viewport/DrawerViewport.tsx:635-645`)
  does the same synchronously. This is the machinery behind the "controlled parents learn of intent
  even when they ignore it" behavior.

### CloseWatcher (Android back) mini-machine

`DrawerProviderReporter` — a null-rendering component injected ahead of user children
(`packages/react/src/drawer/root/DrawerRoot.tsx:220-233`, body at `packages/react/src/drawer/root/DrawerRoot.tsx:419-477`) —
creates one `CloseWatcher` while `open && isTopmost && platform.os.android`, re-checking
`store.select('open')` in the handler, and reports open state into the `DrawerProvider` registry.
`isTopmost` is `nestedOpenDialogCount === 0` (`packages/react/src/drawer/root/DrawerRoot.tsx:429`).

### Hook inventory (name → call sites)

- `useControlled` — `packages/react/src/drawer/root/DrawerRoot.tsx:71`.
- `useStableCallback` — the standard wrapper for every callback that crosses a context value, the
  gesture hook, or an effect: `packages/react/src/drawer/root/DrawerRoot.tsx:81,117,125,138,142,147,152`;
  `packages/react/src/drawer/viewport/DrawerViewport.tsx:174,179,185,194`;
  `packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:121,135,273`;
  `packages/react/src/drawer/popup/DrawerPopup.tsx:180`;
  `packages/react/src/drawer/provider/DrawerProvider.tsx:22,38`;
  `packages/react/src/drawer/root/useDrawerSnapPoints.ts:92`;
  `packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:110,121,160,169,533,539,557`.
- `useIsoLayoutEffect` — all registration/notification/reset effects (the "before passive effects"
  ordering): `packages/react/src/drawer/root/DrawerRoot.tsx:431,441`;
  `packages/react/src/drawer/viewport/DrawerViewport.tsx:803,822,836,849`;
  `packages/react/src/drawer/popup/DrawerPopup.tsx:215,243,270,282`;
  `packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:114,403,409`;
  `packages/react/src/drawer/indent/DrawerIndent.tsx:40`;
  `packages/react/src/drawer/root/useDrawerSnapPoints.ts:104`.
- `useAnimationFrame` — controlled-dismiss rollback check
  (`packages/react/src/drawer/viewport/DrawerViewport.tsx:124`) and keyboard alignment settle loop
  (`packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:107`).
- `useTimeout` — bounded keyboard realign passes (150ms × 4,
  `packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:108,384-394`).
- `useSwipeDismiss` — twice, as described above.
- `useDrawerSnapPoints` — viewport + popup (shared resolution, see above).
- `useRenderElement` — Popup (`packages/react/src/drawer/popup/DrawerPopup.tsx:357`), Backdrop
  (`packages/react/src/drawer/backdrop/DrawerBackdrop.tsx:35`), Content
  (`packages/react/src/drawer/content/DrawerContent.tsx:22`), SwipeArea
  (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:442`), Indent
  (`packages/react/src/drawer/indent/DrawerIndent.tsx:75`), IndentBackground
  (`packages/react/src/drawer/indent-background/DrawerIndentBackground.tsx:36`).
- `useDialogRootContext` — every part (store subscription via `store.useState`/`store.select`).
- `useDrawerRootContext` / `useDrawerProviderContext` / `useDrawerViewportContext` /
  `useDrawerVirtualKeyboardContext` — the four drawer contexts (next section). Note
  `useDrawerRootContext` throws the `Base UI:` guard error when non-optional
  (`packages/react/src/drawer/root/DrawerRootContext.ts:102-114`), and Root reads its *parent's*
  context optionally (`packages/react/src/drawer/root/DrawerRoot.tsx:54`).
- `useOpenChangeComplete` — popup open-complete only
  (`packages/react/src/drawer/popup/DrawerPopup.tsx:295-303`).
- `useTriggerRegistration` + `useBaseUiId` — SwipeArea registry entry
  (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:108-117`).
- `FloatingFocusManager` — popup focus trap/return
  (`packages/react/src/drawer/popup/DrawerPopup.tsx:396-407`); `initialFocus` defaults to the popup
  ref, `false` keeps the trigger focused (`packages/react/src/drawer/popup/DrawerPopup.tsx:305`).
- Store escape hatches: `store.useStateSetter('popupElement')`
  (`packages/react/src/drawer/popup/DrawerPopup.tsx:307`), `store.select` for synchronous reads
  (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:215-217`,
  `packages/react/src/drawer/root/DrawerRoot.tsx:461`).

## Context providers/consumers

Five contexts; what crosses each boundary and who consumes it:

1. **`DrawerRootContext`** (`packages/react/src/drawer/root/DrawerRootContext.ts:100`; provider
   `packages/react/src/drawer/root/DrawerRoot.tsx:249`) — consumed by Popup, Viewport, SwipeArea,
   and `useDrawerSnapPoints`. Carries configuration (`swipeDirection`, `snapToSequentialPoints`,
   `snapPoints`, `swipeAreaActiveRef`), snap state, measurements (`popupHeight`, `frontmostHeight`),
   and the bidirectional nested-drawer channel. The channel works by *callback re-binding*: a nested
   Root reads its parent's context (`packages/react/src/drawer/root/DrawerRoot.tsx:54-59`) and
   republishes the parent's `onNested*` callbacks to its own subtree as `notifyParent*`
   (`packages/react/src/drawer/root/DrawerRoot.tsx:191-194`); the child's Popup then reports
   frontmost height and presence (`packages/react/src/drawer/popup/DrawerPopup.tsx:270-293`) and the
   child's Viewport reports swiping/progress
   (`packages/react/src/drawer/viewport/DrawerViewport.tsx:290-303`, `packages/react/src/drawer/viewport/DrawerViewport.tsx:822-834`).
   High-frequency progress bypasses React: the child viewport writes
   `nestedSwipeProgressStore` (`packages/react/src/drawer/root/DrawerRoot.tsx:142-145`) and the
   parent popup subscribes and writes CSS directly
   (`packages/react/src/drawer/popup/DrawerPopup.tsx:243-268`).
2. **`DialogRootContext`** (provided by `useRenderDialogRoot`, `packages/react/src/dialog/root/useRenderDialogRoot.tsx:92`) —
   the shared state backbone every part reads (open/mounted/nested/modal/popupElement/
   viewportElement/transitionStatus/nestedOpenDrawerCount/popupRef/backdropRef/popupProps). Popup
   additionally calls `useDialogPortalContext()` purely as a presence assertion
   (`packages/react/src/drawer/popup/DrawerPopup.tsx:153`; it throws when the Portal is missing).
3. **`DrawerViewportContext`** (`packages/react/src/drawer/viewport/DrawerViewportContext.tsx:11`;
   provider `packages/react/src/drawer/viewport/DrawerViewport.tsx:1025-1027`) — Viewport → Popup
   only. Carries the live gesture state: `swiping`, `getDragStyles`, `swipeStrength`, and
   `setSwipeDismissed` (`packages/react/src/drawer/viewport/DrawerViewport.tsx:867-875`). Its
   presence/absence is also the marker Popup's dev warning keys on
   (`packages/react/src/drawer/popup/DrawerPopup.tsx:152,164-178`) and whether drag styles are
   rendered at all (`packages/react/src/drawer/popup/DrawerPopup.tsx:333`).
4. **`DrawerProviderContext`** (`packages/react/src/drawer/provider/DrawerProviderContext.ts:11`;
   provider `packages/react/src/drawer/provider/DrawerProvider.tsx:54-56`) — the cross-root
   registry. Written by `DrawerProviderReporter` (open/removed state,
   `packages/react/src/drawer/root/DrawerRoot.tsx:420-443`) and read as `active` by
   Indent/IndentBackground (`packages/react/src/drawer/indent/DrawerIndent.tsx:33-35`,
   `packages/react/src/drawer/indent-background/DrawerIndentBackground.tsx:29-30`); the
   `visualStateStore` inside it is written by Viewport and SwipeArea
   (`packages/react/src/drawer/viewport/DrawerViewport.tsx:208-211`,
   `packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:265-268`) and read by Indent
   (`packages/react/src/drawer/indent/DrawerIndent.tsx:40-69`). Registration is idempotent
   Set membership (`packages/react/src/drawer/provider/DrawerProvider.tsx:22-36`), so a closed
   drawer costs zero re-renders (behavior.md, part `content-indent-provider`).
5. **`DrawerVirtualKeyboardContext`** (`packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardContext.tsx:13`;
   provider `packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:650-654`) —
   Provider → Viewport only (`packages/react/src/drawer/viewport/DrawerViewport.tsx:133`). Viewport
   forwards the touch lifecycle (`onTouchStart`/`onTouchEnd`/`onTouchCancel` from React handlers,
   `onTouchMove` from its *native capture* listener,
   `packages/react/src/drawer/viewport/DrawerViewport.tsx:765-784`) because the viewport claims
   touchmoves with `stopPropagation()`; without this forwarding the keyboard's tap-vs-drag
   threshold never sees claimed moves (`packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardContext.tsx:6-8`).

The virtual keyboard provider itself never renders DOM; it computes the inset on the viewport
element (`packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:198-203`),
applies scroll slack to the field's scroll ancestor
(`packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:121-158`),
and restores everything through snapshot/restore pairs.

## DOM/portal strategy and why

- **Re-export shell.** Trigger, Close, Title, Description, Portal are direct Dialog re-exports with
  narrowed types (`packages/react/src/drawer/trigger/DrawerTrigger.tsx:13`,
  `packages/react/src/drawer/close/DrawerClose.tsx:12`, `packages/react/src/drawer/title/DrawerTitle.tsx:12`,
  `packages/react/src/drawer/description/DrawerDescription.tsx:12`, `packages/react/src/drawer/portal/DrawerPortal.tsx:13`).
  Viewport wraps `DialogViewport` instead of reimplementing the positioning container, merging
  drawer handlers over it and suppressing the generic `data-nested-dialog-open` attribute in favor
  of drawer-specific popup attributes (`packages/react/src/drawer/viewport/DrawerViewport.tsx:894-1028`,
  suppression at `packages/react/src/drawer/viewport/DrawerViewport.tsx:1020-1023`).
- **Sheet positioning without floating-ui.** The popup is not Floating-UI-positioned; it is a
  translated sheet driven by CSS vars: `--drawer-snap-point-offset` (sign-flipped for `up`,
  `packages/react/src/drawer/popup/DrawerPopup.tsx:326-331`), `--drawer-swipe-movement-x/y`, and
  height vars. All gesture math is measured (`offsetWidth/offsetHeight`,
  `packages/react/src/drawer/viewport/DrawerViewport.tsx:1070-1072`), which is why the popup reports
  its own `offsetHeight` to Root via ResizeObserver
  (`packages/react/src/drawer/popup/DrawerPopup.tsx:180-241`) and why heavy behavior is
  Chromium-only (behavior.md, "Geometry probing is intrinsic").
- **High-frequency CSS vars bypass React.** The swipe vars are registered once per app as
  non-inheriting custom properties (module flag + `CSS.registerProperty`,
  `packages/react/src/drawer/popup/DrawerPopup.tsx:27-88`) and written imperatively to
  `element.style` by Viewport (backdrop progress/height,
  `packages/react/src/drawer/viewport/DrawerViewport.tsx:213-227`), SwipeArea (popup movement +
  `transition: none` snapshot/restore, `packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:244-263,281-284`),
  and the keyboard provider (inset). React-rendered styles carry only static defaults (popup
  `packages/react/src/drawer/popup/DrawerPopup.tsx:373-387`; backdrop
  `packages/react/src/drawer/backdrop/DrawerBackdrop.tsx:43-49`; indent
  `packages/react/src/drawer/indent/DrawerIndent.tsx:80-83`). The stated rationale is style-recalc
  cost in deep subtrees (`packages/react/src/drawer/popup/DrawerPopup.tsx:31-36`).
- **Content boundary marker.** `Drawer.Content` renders a deliberately non-public
  `data-drawer-content` attribute (`packages/react/src/drawer/content/DrawerContent.tsx:24`,
  constant at `packages/react/src/drawer/content/drawerContentAttribute.ts:1`) used as a hit-test
  boundary so non-touch drags never start from content descendants
  (`packages/react/src/drawer/viewport/DrawerViewport.tsx:58,913,1066-1068`).
- **SwipeArea as a registered pseudo-trigger.** It is an in-place `div` (no portal) that registers
  its generated id in the dialog store's `triggerElements` registry via `useTriggerRegistration`
  (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:108-117`; the layout effect exists
  because the id resolves after the first commit on React 17,
  `packages/react/src/utils/popups/popupStoreUtils.ts:144`), so the aria/active-trigger machinery
  treats it as a trigger while the element itself is `role="presentation" aria-hidden` with
  directional `touch-action` and `pointer-events: none` when disabled
  (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:448-453`). During a swipe-open it
  disables outside-press dismissal and re-enables it via a deterministic document-capture
  pointerdown/click guard that skips the gesture's own trailing release click
  (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:130-167`); the handoff to the viewport
  is `swipeAreaActiveRef`, which makes the viewport skip `resetSwipe()` on reopen to avoid a
  one-frame flash (`packages/react/src/drawer/viewport/DrawerViewport.tsx:836-847`).
- **Backdrop.** Ref-merged into the store's `backdropRef` so gesture surfaces can write to it
  imperatively (`packages/react/src/drawer/backdrop/DrawerBackdrop.tsx:37`), `hidden` while
  unmounted, inert (`pointerEvents: 'none'`) while closed, and skipped entirely for nested drawers
  unless `forceRender` (`packages/react/src/drawer/backdrop/DrawerBackdrop.tsx:42-53`).
- **Shadow-root-safe hit testing.** All `elementFromPoint` probes go through
  `getElementAtPoint(getRootNode(), …)` (`packages/react/src/drawer/viewport/DrawerViewport.tsx:359-361,908-913,965-971`;
  `packages/react/src/utils/getElementAtPoint.ts:10-13`) and traversal uses the shadow-safe
  `contains`/`getTarget`/`activeElement` from floating-ui-react utils.

## Dependencies on other Base UI internals

Grouped by package; each entry names the drawer-side import site. This is the section intended to
replace `ralph/scripts/generate-todo.mjs`'s coarse `blocked-by: [Phase A complete]` default
(`TODO.md:378`) with precise per-unit requirements.

- **`dialog` package (the dominant dependency — a drawer port needs the Dialog core first):**
  `useRenderDialogRoot` (`packages/react/src/drawer/root/DrawerRoot.tsx:22`), `DialogRootContext` /
  `useDialogRootContext` (`packages/react/src/drawer/viewport/DrawerViewport.tsx:11` and every
  part), `DialogViewport` + data attributes (`packages/react/src/drawer/viewport/DrawerViewport.tsx:12-13`),
  `DialogPortal` (`packages/react/src/drawer/portal/DrawerPortal.tsx:3`) and `DialogPortalContext`
  (`packages/react/src/drawer/popup/DrawerPopup.tsx:19`), `DialogTrigger`/`DialogClose`/
  `DialogTitle`/`DialogDescription` (re-exports above), `DialogHandle`
  (`packages/react/src/drawer/handle.ts:1-15` — nominal typing only, all behavior inherited). Under
  this sit `DialogStore`, `usePopupRootStore`, `usePopupRootSync`, `useOpenStateTransitions`,
  `useImplicitActiveTrigger`, and `DialogInteractions`
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:9-15,49-100`).
- **`floating-ui-react`:** `FloatingFocusManager` (`packages/react/src/drawer/popup/DrawerPopup.tsx:9,396-407`),
  shadow-safe traversal utils `activeElement`/`contains`/`getTarget`/`isInteractiveElement`
  (`packages/react/src/drawer/viewport/DrawerViewport.tsx:35`,
  `packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:11-16`),
  `isVirtualClick` (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:22`), and
  `@floating-ui/utils/dom` DOM helpers `isElement` (`packages/react/src/drawer/viewport/DrawerViewport.tsx:4`)
  and `getComputedStyle`/`getParentNode`/`isHTMLElement`
  (`packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:3`).
- **`utils/useSwipeDismiss`** — the shared gesture engine used by both surfaces, plus `getDisplacement`
  and the `SwipeDirection` type (`packages/react/src/drawer/viewport/DrawerViewport.tsx:23-28`,
  `packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:13`,
  `packages/react/src/drawer/root/DrawerRootContext.ts:3`).
- **Small utils:** `utils/scrollable` (`findScrollableTouchTarget`,
  `packages/react/src/drawer/viewport/DrawerViewport.tsx:38`,
  `packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:17`),
  `utils/getElementAtPoint` (`packages/react/src/drawer/viewport/DrawerViewport.tsx:40`),
  `utils/getElementTransform` (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:14`),
  `utils/popups` (`FOCUSABLE_POPUP_PROPS`, `useTriggerRegistration`, `PayloadChildRenderFunction`,
  `packages/react/src/drawer/popup/DrawerPopup.tsx:25`,
  `packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:20`,
  `packages/react/src/drawer/root/DrawerRoot.tsx:25`),
  `utils/popupStateMapping` (`popupTransitionStateMapping` + `CommonPopupDataAttributes`,
  `packages/react/src/drawer/popup/DrawerPopup.tsx:15`,
  `packages/react/src/drawer/popup/DrawerPopupDataAttributes.ts:1`).
- **`internals/`:** `useRenderElement` + `BaseUIComponentProps`,
  `createBaseUIEventDetails` + `REASONS` (`packages/react/src/drawer/root/DrawerRoot.tsx:16-20`),
  `stateAttributesMapping`/`TransitionStatusDataAttributes`
  (`packages/react/src/drawer/viewport/DrawerViewport.tsx:37`), `useOpenChangeComplete`
  (`packages/react/src/drawer/popup/DrawerPopup.tsx:20`), `useBaseUiId`
  (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:19`), `mergeProps`
  (`packages/react/src/drawer/viewport/DrawerViewport.tsx:14`), `COMPOSITE_KEYS`
  (`packages/react/src/drawer/popup/DrawerPopup.tsx:21`), `NOOP`
  (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:10`), `BASE_UI_SWIPE_IGNORE_SELECTOR`
  (`packages/react/src/drawer/viewport/DrawerViewport.tsx:39`).
- **`@base-ui/utils` (public utils package):** `useControlled`, `useIsoLayoutEffect`,
  `useStableCallback`, `useAnimationFrame`, `useTimeout`, `addEventListener`, `owner`
  (`ownerDocument`/`ownerWindow`), `platform` (Android gating for CloseWatcher,
  `packages/react/src/drawer/root/DrawerRoot.tsx:8,448`), `clamp`
  (`packages/react/src/drawer/root/useDrawerSnapPoints.ts:6`),
  `error` + `SafeReact.captureOwnerStack` for the dev warning
  (`packages/react/src/drawer/popup/DrawerPopup.tsx:3-4,171-176`), `EMPTY_OBJECT`
  (`packages/react/src/drawer/popup/DrawerPopup.tsx:8`).

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit loop; none of these
are recorded in behavior.md or asserted by any drawer test:

1. **The swipe-release strength pipeline (`--drawer-swipe-strength`).** `resolveSwipeRelease`
   computes a 0.1–1 scalar from remaining distance ÷ clamped release velocity
   (`packages/react/src/drawer/viewport/DrawerViewport.tsx:231-288`), stored via `flushSync` into
   `swipeRelease` state (`packages/react/src/drawer/viewport/DrawerViewport.tsx:120,440-453`),
   published as `swipeStrength` on `DrawerViewportContext`
   (`packages/react/src/drawer/viewport/DrawerViewport.tsx:867-875`), rendered by the popup as
   `--drawer-swipe-strength` (`packages/react/src/drawer/popup/DrawerPopup.tsx:383-386`) and
   defaulted to `'1'` on the backdrop (`packages/react/src/drawer/backdrop/DrawerBackdrop.tsx:48`),
   with the custom property registered as `<number>` initial `1`
   (`packages/react/src/drawer/popup/DrawerPopup.tsx:64-72`). Grep finds no `strength` in any
   drawer test or in `specs/library/drawer/` — only the docs CSS consumes it as a
   transition-duration multiplier (`docs/src/components/MobileNav.css:42`). The fixtures stage must
   define the oracle for this scalar (documented contract in
   `packages/react/src/drawer/popup/DrawerPopupCssVars.ts:31-35`).
2. **`data-expanded` on the popup.** State `expanded: activeSnapPoint === 1`
   (`packages/react/src/drawer/popup/DrawerPopup.tsx:313`) mapped to the attribute
   (`packages/react/src/drawer/popup/DrawerPopupDataAttributes.ts:20-22`). No drawer test or
   behavior part mentions it; nothing pins whether `expanded` means "value 1" specifically vs
   full-height.
3. **SwipeArea's accessibility/style shell.** `role="presentation"`, `aria-hidden="true"`, and the
   inline `touch-action`/`pointer-events` contract
   (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.tsx:448-453`) are unasserted —
   `describeConformance` covers props/ref/renderProp/className only
   (`packages/react/src/drawer/swipe-area/DrawerSwipeArea.test.tsx:298`), and no test checks
   aria-hidden or touch-action. A port could ship this div without them and pass every test.
4. **Backdrop static style contract.** There is no `DrawerBackdrop.test.tsx` at all (backdrop
   assertions in behavior.md come from viewport/swipe-area suites exercising transient vars only).
   Unasserted: closed `pointerEvents: 'none'`, `userSelect`/`WebkitUserSelect: 'none'`, and the
   default `--drawer-swipe-progress: 0` / `hidden: !mounted`
   (`packages/react/src/drawer/backdrop/DrawerBackdrop.tsx:42-49`).
5. **Nested-count asymmetry.** Keyboard suppression and CloseWatcher topmost gating read
   `nestedOpenDialogCount` (`packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:93`,
   `packages/react/src/drawer/root/DrawerRoot.tsx:426`) while viewport/popup swipe suppression reads
   `nestedOpenDrawerCount` (`packages/react/src/drawer/viewport/DrawerViewport.tsx:109`,
   `packages/react/src/drawer/popup/DrawerPopup.tsx:142`). Both counters exist on the store
   (`packages/react/src/dialog/store/DialogStore.ts:21-22`) but no drawer test distinguishes them.
   The source implies keyboard coordination and the back gesture also yield to nested *Dialogs*
   (any popup kind), not only nested drawers — behavior.md's virtual-keyboard part records only
   "inert when a nested drawer is open", so this broader suppression is unverified.
6. **Popup's `hidden` + focusability defaults.** `hidden: !mounted`, `tabIndex: -1` and the
   focusable marker from `FOCUSABLE_POPUP_PROPS`
   (`packages/react/src/drawer/popup/DrawerPopup.tsx:366-367`,
   `packages/react/src/utils/popups/popupStoreUtils.ts:32-35`) are inherited Dialog conventions
   with no drawer-level assertion (the popup-focus behavior is tested; these attributes are not).

Nothing else in the unit's source was found without test or behavior.md coverage: snap resolution
math (including `rem`, invalid strings, clamping, dedup) is covered by the jsdom snap suites
(`packages/react/src/drawer/root/DrawerSnapPoints.test.tsx:57-93`), the registry/handle/aria
surfaces by the jsdom root suite, and gesture geometry by the Chromium-only suites as recorded in
behavior.md's part index.
