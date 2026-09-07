# Dialog — Implementation Spec

Mined from source. `specs/library/dialog/behavior.md` is ground truth for WHAT happens; this
document explains WHY/HOW. Where behavior is referenced, it is cited by section name, not restated.

## State machine / hooks used

### Store ownership and creation

- `DialogRoot` is a thin `fastComponent` wrapper delegating to `useRenderDialogRoot('dialog', props)`
  (`packages/react/src/dialog/root/DialogRoot.tsx:16-20`). The same render function powers Drawer and
  AlertDialog via the `mode` parameter (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:35-39,106`).
- One `DialogStore` per Root, created exactly once via `usePopupRootStore`
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:49-63`), which seeds it with a `useId`-derived
  `floatingId`, the Floating-UI-tree `nested` flag, and a synced `FloatingRootStore`
  (`packages/react/src/utils/popups/popupStoreUtils.ts:73-96`). The store is owned by the Root instance,
  not by the handle — the comment at `packages/react/src/dialog/root/useRenderDialogRoot.tsx:45-48` states this explicitly, and it is
  what produces behavior.md's "Switching the `handle` prop re-attaches without resetting" and the
  fresh-state-on-remount entries in its State model section.
- State shape: shared popup state (`open`, `openProp`, `mounted`, `transitionStatus`,
  `floatingRootContext`, `payload`, `activeTriggerId`/`activeTriggerElement`, `popupProps`, …) plus
  dialog-specific `modal`, `disablePointerDismissal`, `openMethod`, `nested`, `nestedOpenDialogCount`,
  `nestedOpenDrawerCount`, `titleElementId`, `descriptionElementId`, `viewportElement`, `role`
  (`packages/react/src/dialog/store/DialogStore.ts:16-27`). Defaults in `createInitialState`
  (`packages/react/src/dialog/store/DialogStore.ts:115-137`): `modal: true`,
  `disablePointerDismissal: false`, `role: 'dialog'`.
- The store is a `ReactStore` (`@base-ui/utils/store`), providing `useState(selector)` subscriptions,
  `useSyncedValues`, `useControlledProp`, `useContextCallback`, `useStateSetter`,
  `useSyncedValueWithCleanup` used throughout the parts.

### Controlled/uncontrolled resolution

- Consumers read derived selectors, never raw state: `open` is `openProp ?? open` and `activeTriggerId`
  is `triggerIdProp ?? activeTriggerId` (`packages/react/src/utils/popups/store.ts:140-142`). Controlled
  `open`/`triggerId` therefore work by prop override with no imperative sync.
- The Root syncs props into the store after creation: `store.useControlledProp('openProp', openProp)`,
  `store.useControlledProp('triggerIdProp', triggerIdProp)`
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:65-66`), `store.useSyncedValues(rootState)` for
  `modal`/`disablePointerDismissal`/`nested`/`role` (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:43,68`), and user callbacks
  into store context via `useContextCallback('onOpenChange'/'onOpenChangeComplete')`
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:69-70`).

### The open/close transition pipeline

All open/close requests converge on `DialogStore.setOpen`
(`packages/react/src/dialog/store/DialogStore.ts:74-97`):

1. Attaches `preventUnmountOnClose()` to the event details, which sets the store flag
   `preventUnmountingOnClose` (`packages/react/src/dialog/store/DialogStore.ts:78-80`).
2. On close with no explicit trigger, back-fills `eventDetails.trigger` from the current active trigger
   element so focus return and ARIA resets keep using the old trigger until the close completes
   (`packages/react/src/dialog/store/DialogStore.ts:82-86`).
3. Calls `context.onOpenChange` (the user callback) and returns early if `eventDetails.isCanceled`
   (`packages/react/src/dialog/store/DialogStore.ts:88-92`) — the cancellation semantics of behavior.md's Events section.
4. Dispatches on the floating root context (this is the internal `openchange` event stream behavior.md
   observes via `floatingRootContext.context.events`) and commits `createPopupOpenState`
   (`packages/react/src/dialog/store/DialogStore.ts:94-96`; `packages/react/src/utils/popups/popupStoreUtils.ts:190-221`), which clears
   `preventUnmountingOnClose` on open and preserves the active trigger while closing so exit animations
   and focus return keep working.
- Dialog intentionally does not use the shared `applyPopupOpenChange`
  (`packages/react/src/utils/popups/popupStoreUtils.ts:241-308`): it has no hover path, so no
  `instantType` logic, and it adds the trigger backfill above (see the unexplained section).

### mounted / transitionStatus / completion machine

- `useOpenStateTransitions(open, store)` (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:78`;
  `packages/react/src/utils/popups/popupStoreUtils.ts:554-601`) derives `mounted` + `transitionStatus`
  via `useTransitionStatus` and auto-unmounts on close-completion through `useOpenChangeComplete`
  (`packages/react/src/internals/useOpenChangeComplete.tsx:9-28`, `useAnimationsFinished`-based), unless
  `preventUnmountingOnClose` is set — implementing the `preventUnmountOnClose()` /
  `actionsRef.unmount()` pairing in behavior.md's State model section.
- `forceUnmount` (`packages/react/src/utils/popups/popupStoreUtils.ts:577-587`) is the manual unmount
  path; it also fires `onOpenChangeComplete(false)` (the close-side completion). The open-side completion
  is wired separately in `DialogPopup` via its own `useOpenChangeComplete` on `popupRef`
  (`packages/react/src/dialog/popup/DialogPopup.tsx:47-55`), so `onOpenChangeComplete(true)` fires when
  the popup's CSS animations finish.
- `usePopupRootSync` resets `openMethod` to `null` on close and on unmount
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:76`; `packages/react/src/utils/popups/popupStoreUtils.ts:626-645`).

### Interaction wiring (`DialogInteractions`)

- The Root renders a `DialogInteractions` component (defined in
  `packages/react/src/dialog/root/useDialogRoot.ts:10-127`) only while `open || mounted`
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:89,94-100`) — dismiss listeners, scroll lock, and nested counting exist only
  while relevant.
- `useDismiss` (floating-ui-react) is configured with:
  - `outsidePressEvent` returning `'intentional'` when a backdrop element currently exists (internal or
    user), else `'sloppy'` mouse for `modal === 'trap-focus'` (so `aria-hidden` is removed immediately on
    outside press) and `'sloppy'` touch (`packages/react/src/dialog/root/useDialogRoot.ts:30-40`) — the mechanism behind behavior.md's
    "mousedown alone does not close" Events entry.
  - A `outsidePress` guard chain: `outsidePressEnabledRef`, main-button-only, and single-finger touch
    rules (`touchend` accepted only when exactly one touch changed and none remain)
    (`packages/react/src/dialog/root/useDialogRoot.ts:41-63`) — the touch edge cases in behavior.md's Events section.
  - Modal target resolution: close only when the press hit the `internalBackdropRef` or `backdropRef`
    element, or is inside the popup subtree but not on a nested portal container
    (`data-base-ui-portal`) (`packages/react/src/dialog/root/useDialogRoot.ts:65-80`) — this is what implements the
    "sibling dialogs dismiss one at a time, via their own backdrops" Edge case in behavior.md.
  - `escapeKey: isTopmost`, where `isTopmost = ownNestedOpenDialogs === 0`
    (`packages/react/src/dialog/root/useDialogRoot.ts:27,83`) — only the innermost open dialog handles Escape.
- `useScrollLock(open && modal === true, popupElement)` (`packages/react/src/dialog/root/useDialogRoot.ts:86`) — scroll lock only for
  full modal, not `'trap-focus'`, matching the `modal` prop contract in behavior.md's Public API section.
- Nested counting state machine: local React state `ownNestedOpenDialogs`/`ownNestedOpenDrawers`
  (`packages/react/src/dialog/root/useDialogRoot.ts:25-26`); notification to the parent happens in a `useIsoLayoutEffect` calling
  `parentContext.onNestedDialogOpen(ownNestedOpenDialogs + 1, ...)` (self-inclusive count), with a
  cleanup that zeroes the counts on close/unmount (`packages/react/src/dialog/root/useDialogRoot.ts:96-112`) — the layout-effect timing
  guarantee in behavior.md's Edge cases section; the parent store receives it through
  `store.useContextCallback('onNestedDialogOpen', ...)` (`packages/react/src/dialog/root/useDialogRoot.ts:88-93`), and the counts are
  mirrored into store state via `usePopupInteractionProps` (`packages/react/src/dialog/root/useDialogRoot.ts:114-124` →
  `packages/react/src/utils/popups/popupStoreUtils.ts:605-624`), which feeds `--nested-dialogs` and
  `data-nested-dialog-open` on Popup/Viewport.
- `usePopupInteractionProps` stores dismiss-derived prop bundles into the store:
  `activeTriggerProps: dismiss.reference`, `inactiveTriggerProps: dismiss.trigger`,
  `popupProps: dismiss.floating` (`packages/react/src/dialog/root/useDialogRoot.ts:114-124`) — how dismissal/focus behavior reaches the
  trigger and popup elements without prop drilling.

### Trigger registry and active-trigger machine

- `DialogTrigger` resolves its store from context or handle: `usePopupHandleStore(handle)` (a
  `useSyncExternalStore` over `handle.subscribeStore`,
  `packages/react/src/utils/popups/usePopupHandleStore.ts:17-36`) falling back to
  `useDialogRootContext(true)`; it throws if neither exists
  (`packages/react/src/dialog/trigger/DialogTrigger.tsx:38-45`).
- `useTriggerDataForwarding` registers the trigger element into `store.context.triggerElements` (a
  `PopupTriggerMap`) and applies trigger-owned state (payload, active-trigger claim)
  (`packages/react/src/dialog/trigger/DialogTrigger.tsx:54-61`; `packages/react/src/utils/popups/popupStoreUtils.ts:318-388`, element
  registration at `:144-183`).
- `useImplicitActiveTrigger` on the Root claims the sole registered trigger when none is active and
  reconciles id/element changes across re-registrations (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:77`;
  `packages/react/src/utils/popups/popupStoreUtils.ts:416-537`).
- Per-trigger ARIA comes from parameterized selectors: `isOpenedByTrigger` → `aria-expanded`,
  `triggerPopupId` → `aria-controls` (`packages/react/src/dialog/trigger/DialogTrigger.tsx:49-50,92-95`; selectors at
  `packages/react/src/utils/popups/store.ts:183-201`).
- Trigger presses loop through the floating root context: `useClick(floatingContext)`
  (`packages/react/src/dialog/trigger/DialogTrigger.tsx:68`), and `usePopupRootStore` wires the FloatingRootStore's `onOpenChange` to
  `store.setOpen` (`packages/react/src/utils/popups/popupStoreUtils.ts:92`; kept fresh at
  `packages/react/src/floating-ui-react/hooks/useSyncedFloatingRootContext.ts:110`).
- `useOpenMethodTriggerProps` records the interaction type into `openMethod` on open attempts
  (`packages/react/src/dialog/trigger/DialogTrigger.tsx:69-74`; `packages/react/src/utils/useOpenInteractionType.ts:8-37`) — driving
  behavior.md's keyboard/touch `initialFocus` interaction-type entries in its Focus management section.
- Detached-trigger lifecycle: `PopupHandleAttachment` attaches the Root's store to the handle in a
  layout effect that is deliberately rendered before interactions and children
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:92-100`; `packages/react/src/utils/popups/popupStoreUtils.ts:98-120`), so
  same-commit imperative opens resolve (behavior.md Edge cases).
- `DialogHandle` extends `BasePopupHandle` with a `NullStore` fallback and
  `throwOnMissingTrigger = false`, so `open(id)` with an unregistered id warns ("No trigger found")
  instead of throwing (`packages/react/src/dialog/store/DialogHandle.ts:14-32`; base behavior at
  `packages/react/src/utils/popups/popupHandle.ts:104-108,243-265`). `BasePopupHandle.attachStore` keeps
  an attachment stack, defers the "more than one mounted root" overlap warning by an animation frame,
  and restores the previous root's store on detach (`packages/react/src/utils/popups/popupHandle.ts:151-187`)
  — the transient-overlap/StrictMode semantics in behavior.md's State model section.

### Hook inventory (call sites)

- `usePopupRootStore` — `packages/react/src/dialog/root/useRenderDialogRoot.tsx:49`
- `store.useControlledProp` — `packages/react/src/dialog/root/useRenderDialogRoot.tsx:65-66`
- `store.useSyncedValues` — `packages/react/src/dialog/root/useRenderDialogRoot.tsx:68`; via `usePopupInteractionProps` at
  `packages/react/src/dialog/root/useDialogRoot.ts:114-124`
- `store.useContextCallback` — `packages/react/src/dialog/root/useRenderDialogRoot.tsx:69-70`; `packages/react/src/dialog/root/useDialogRoot.ts:88-93`
- `store.useState` (incl. parameterized selectors) — `packages/react/src/dialog/popup/DialogPopup.tsx:30-43`, `packages/react/src/dialog/trigger/DialogTrigger.tsx:48-50,81`,
  `packages/react/src/dialog/backdrop/DialogBackdrop.tsx:23-26`, `packages/react/src/dialog/viewport/DialogViewport.tsx:25-29`, `packages/react/src/dialog/portal/DialogPortal.tsx:24-26`, `packages/react/src/dialog/close/DialogClose.tsx:30`
- `store.useStateSetter` — `packages/react/src/dialog/popup/DialogPopup.tsx:62` (`popupElement`), `packages/react/src/dialog/viewport/DialogViewport.tsx:31`
  (`viewportElement`)
- `store.useSyncedValueWithCleanup` — `packages/react/src/dialog/title/DialogTitle.tsx:24` (`titleElementId`),
  `packages/react/src/dialog/description/DialogDescription.tsx:24` (`descriptionElementId`) — the ARIA id linking + cleanup on unmount from
  behavior.md's Accessibility section
- `useIsoLayoutEffect` — `packages/react/src/dialog/root/useDialogRoot.ts:96`; inside `PopupHandleAttachment`,
  `useTriggerDataForwarding`, `usePopupRootSync`
- `React.useImperativeHandle` — `packages/react/src/dialog/root/useRenderDialogRoot.tsx:80-87` (`actionsRef`: `unmount` →
  `forceUnmount`, `close` → `setOpen(false, imperativeAction)`)
- `useDismiss` — `packages/react/src/dialog/root/useDialogRoot.ts:29`
- `useScrollLock` — `packages/react/src/dialog/root/useDialogRoot.ts:86`
- `useClick` — `packages/react/src/dialog/trigger/DialogTrigger.tsx:68`
- `useOpenMethodTriggerProps` — `packages/react/src/dialog/trigger/DialogTrigger.tsx:69`
- `useButton` — `packages/react/src/dialog/trigger/DialogTrigger.tsx:63-66`, `packages/react/src/dialog/close/DialogClose.tsx:32-35`
- `useBaseUiId` — `packages/react/src/dialog/trigger/DialogTrigger.tsx:47`, `packages/react/src/dialog/title/DialogTitle.tsx:22`, `packages/react/src/dialog/description/DialogDescription.tsx:22`
- `useOpenChangeComplete` — `packages/react/src/dialog/popup/DialogPopup.tsx:47`
- `useRenderElement` — every rendered part (`packages/react/src/dialog/popup/DialogPopup.tsx:71`, `packages/react/src/dialog/trigger/DialogTrigger.tsx:83`,
  `packages/react/src/dialog/backdrop/DialogBackdrop.tsx:33`, `packages/react/src/dialog/close/DialogClose.tsx:45`, `packages/react/src/dialog/title/DialogTitle.tsx:26`, `packages/react/src/dialog/description/DialogDescription.tsx:26`,
  `packages/react/src/dialog/viewport/DialogViewport.tsx:44`)
- `fastComponent` / `fastComponentRef` — `packages/react/src/dialog/root/DialogRoot.tsx:16`, `packages/react/src/dialog/trigger/DialogTrigger.tsx:22`: collapses multiple
  store subscriptions into one per store on React 19+ (`packages/utils/src/fastHooks.ts:29-41`)
- `DialogClose` mechanics: the built-in handler runs only while `open`
  (`packages/react/src/dialog/close/DialogClose.tsx:39-43`), and its props order
  `[{onClick}, elementProps, getButtonProps]` (`packages/react/src/dialog/close/DialogClose.tsx:45-49`) means the user's handler runs
  first in the merge; `event.preventBaseUIHandler()` sets `baseUIHandlerPrevented` on the synthetic event
  and suppresses the internal handler (`packages/react/src/merge-props/mergeProps.ts:229-248,268-274`) —
  the Close prevention semantics in behavior.md's Events section.

## Context providers/consumers

- `DialogRootContext` carries the `DialogStore` object itself, not a prop bundle
  (`packages/react/src/dialog/root/DialogRootContext.ts:5`) — each part subscribes to exactly the state
  slices it needs.
  - Provider: Root (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:92`).
  - Throwing consumers (`useDialogRootContext()`): `DialogPopup` (`packages/react/src/dialog/popup/DialogPopup.tsx:28`), `DialogPortal`
    (`packages/react/src/dialog/portal/DialogPortal.tsx:23`), `DialogBackdrop` (`packages/react/src/dialog/backdrop/DialogBackdrop.tsx:21`), `DialogClose`
    (`packages/react/src/dialog/close/DialogClose.tsx:29`), `DialogTitle` (`packages/react/src/dialog/title/DialogTitle.tsx:20`), `DialogDescription`
    (`packages/react/src/dialog/description/DialogDescription.tsx:20`), `DialogViewport` (`packages/react/src/dialog/viewport/DialogViewport.tsx:23`). The thrown error is the
    "DialogRootContext is missing" message in behavior.md's DOM structure section
    (`packages/react/src/dialog/root/DialogRootContext.ts:12-16`).
  - Optional consumers (`useDialogRootContext(true)`): the Root itself reads the parent store to compute
    `nested` for the nesting-count wiring (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:41-42`), and `DialogTrigger` falls
    back to a handle store when no root ancestor exists (`packages/react/src/dialog/trigger/DialogTrigger.tsx:38-45`).
- `DialogPortalContext` carries a single boolean `keepMounted`
  (`packages/react/src/dialog/portal/DialogPortalContext.ts:4`), provided by `DialogPortal`
  (`packages/react/src/dialog/portal/DialogPortal.tsx:34`) and consumed by `DialogPopup` (`packages/react/src/dialog/popup/DialogPopup.tsx:45`) and `DialogViewport`
  (`packages/react/src/dialog/viewport/DialogViewport.tsx:22`); the hook throws "Base UI: <Dialog.Portal> is missing." when absent
  (`packages/react/src/dialog/portal/DialogPortalContext.ts:6-11`) — the portal-missing error in behavior.md's DOM structure section.
  `DialogBackdrop` does not consume it (see the unexplained section).
- Store context (not React context) bridges DOM to logic via refs: `popupRef` (popup element → focus and
  completion), `backdropRef` (user backdrop → dismissal target), `internalBackdropRef` (portal-rendered
  backdrop), `outsidePressEnabledRef` (`packages/react/src/dialog/store/DialogStore.ts:29-35,139-149`). Refs avoid re-renders when
  elements attach/detach.
- Store-prop bundles replace prop drilling: `popupProps` consumed by `DialogPopup`
  (`packages/react/src/dialog/popup/DialogPopup.tsx:33,74`), `triggerProps` by `DialogTrigger` (`packages/react/src/dialog/trigger/DialogTrigger.tsx:81,88`), both produced
  by `usePopupInteractionProps` inside `DialogInteractions`.
- Cross-boundary to arbitrary children of Root: the payload render prop `children({ payload })`
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:101`), typed via `handle: DialogHandle<Payload>` inference (type-level spec
  `packages/react/src/dialog/root/DialogRoot.spec.tsx:13-28`).
- `PopupHandleAttachment` (renders `null`) is the handle↔store boundary link
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:93`).

## DOM/portal strategy and why

- Root and `DialogInteractions` render no DOM. Rendered elements: trigger `<button>`
  (`packages/react/src/dialog/trigger/DialogTrigger.tsx:83`), close `<button>` (`packages/react/src/dialog/close/DialogClose.tsx:45`), popup `<div>` (`packages/react/src/dialog/popup/DialogPopup.tsx:71`),
  backdrop `<div>` (`packages/react/src/dialog/backdrop/DialogBackdrop.tsx:33`), viewport `<div>` (`packages/react/src/dialog/viewport/DialogViewport.tsx:44`), title `<h2>`
  (`packages/react/src/dialog/title/DialogTitle.tsx:26`), description `<p>` (`packages/react/src/dialog/description/DialogDescription.tsx:26`), portal wrapper `<div>`
  (`packages/react/src/dialog/portal/DialogPortal.tsx:35` ref target).
- `DialogPortal` wraps `FloatingPortal` (default target `<body>`; `container` accepts
  `HTMLElement | ShadowRoot | ref`, `packages/react/src/dialog/portal/DialogPortal.tsx:56-57`), render-gated by `mounted || keepMounted`
  (`packages/react/src/dialog/portal/DialogPortal.tsx:28-31`). FloatingPortal marks the portal element `data-base-ui-portal`
  (`packages/react/src/floating-ui-react/components/FloatingPortal.tsx:51,132`), which the modal
  outside-press logic uses to exclude clicks on nested portal containers from "inside" presses
  (`packages/react/src/dialog/root/useDialogRoot.ts:76`).
- The internal modal backdrop is rendered inside the portal, before the user's children:
  `<InternalBackdrop ref={store.context.internalBackdropRef} inert={inertValue(!open)}>` when
  `mounted && modal === true` (`packages/react/src/dialog/portal/DialogPortal.tsx:36-38`). `InternalBackdrop` is a fixed inset-0
  `role="presentation"` div carrying `data-base-ui-inert` so Floating UI treats it as a pre-existing
  element for outside-press purposes (`packages/react/src/utils/InternalBackdrop.tsx:18-34`). This is the
  click target that `useDismiss.outsidePress` requires for modal dialogs (`packages/react/src/dialog/root/useDialogRoot.ts:70-78`) —
  i.e. behavior.md's sibling-dialogs entry is implemented by requiring the press to land on *this*
  dialog's own backdrop element.
- No positioner: the popup element itself is passed to Floating UI as the floating element
  (`usePopupRootStore(store, true)` — the `treatPopupAsFloatingElement` flag at
  `packages/react/src/dialog/root/useRenderDialogRoot.tsx:49-63`; `popupElement` selected as the floating element at
  `packages/react/src/floating-ui-react/hooks/useSyncedFloatingRootContext.ts:53-55`). Dialog does not
  position itself; the optional `Dialog.Viewport` is the user's positioning/scroll container instead
  (`packages/react/src/dialog/viewport/DialogViewport.tsx:10-15`).
- Focus management is delegated to `FloatingFocusManager` wrapping the popup element
  (`packages/react/src/dialog/popup/DialogPopup.tsx:97-110`): `modal={modal !== false}` (traps for `true` and `'trap-focus'`),
  `initialFocus` resolved from the prop or the touch-aware default (`packages/react/src/dialog/popup/DialogPopup.tsx:57-58`; default at
  `packages/react/src/utils/popups/popupStoreUtils.ts:42-45`), `returnFocus={finalFocus}`,
  `closeOnFocusOut={!disablePointerDismissal}` (non-modal focus-out close), `restoreFocus="popup"`
  (focus returns to the popup itself when a focused child is removed — behavior.md's Focus management
  section), `openInteractionType={openMethod}`, `disabled={!mounted}`.
- Mount visibility is encoded as `hidden={!mounted}` on popup (`packages/react/src/dialog/popup/DialogPopup.tsx:81`), viewport
  (`packages/react/src/dialog/viewport/DialogViewport.tsx:52`), and backdrop (`packages/react/src/dialog/backdrop/DialogBackdrop.tsx:40`) rather than conditional DOM presence,
  so `keepMounted` portals and exit animations share one mechanism; the viewport additionally disables
  pointer events while closed (`packages/react/src/dialog/viewport/DialogViewport.tsx:53-55`) and renders only while `keepMounted || mounted`
  (`packages/react/src/dialog/viewport/DialogViewport.tsx:42-45`). The backdrop is the only part whose *rendering* is independently gated:
  `enabled: forceRender || !nested` (`packages/react/src/dialog/backdrop/DialogBackdrop.tsx:48`) — the nested-backdrop suppression from
  behavior.md's DOM structure section, using the `nested` flag derived from the parent store
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:41-43`).
- Element identity flows through merged refs into the store rather than context: popup
  `ref: [forwardedRef, store.context.popupRef, setPopupElement]` (`packages/react/src/dialog/popup/DialogPopup.tsx:93`), viewport
  (`packages/react/src/dialog/viewport/DialogViewport.tsx:47`), backdrop (`packages/react/src/dialog/backdrop/DialogBackdrop.tsx:35`) — one shared node for dismissal, focus,
  and completion logic.
- Composite navigation keys (arrow keys etc.) are stopped at the popup boundary so they do not leak into
  outer composites (`packages/react/src/dialog/popup/DialogPopup.tsx:11,82-86`).

## Dependencies on other Base UI internals

This unit has no `wraps-external:` field in `TODO.md` — it is a self-contained React implementation with
no external-package delegation to name.

- `floating-ui-react` (in-repo Floating UI fork; the heaviest dependency):
  - `useDismiss` — `packages/react/src/dialog/root/useDialogRoot.ts:5,29` (outside press, escape, touch rules, press-event classification)
  - `useClick` — `packages/react/src/dialog/trigger/DialogTrigger.tsx:13,68`
  - `FloatingPortal` — `packages/react/src/dialog/portal/DialogPortal.tsx:4,35`
  - `FloatingFocusManager` — `packages/react/src/dialog/popup/DialogPopup.tsx:4,98` (focus trap, initial/final focus, `restoreFocus`,
    `closeOnFocusOut`)
  - `FloatingRootStore` — instantiated inside the shared popup state
    (`packages/react/src/utils/popups/store.ts:93`) and synced by `useSyncedFloatingRootContext`
    (`packages/react/src/floating-ui-react/hooks/useSyncedFloatingRootContext.ts:82-111`)
  - DOM helpers `contains`, `getTarget` — `packages/react/src/dialog/root/useDialogRoot.ts:6,65,76`
- `packages/react/src/utils/popups/` (the shared popup framework; dialog is one client of it):
  `usePopupRootStore`, `PopupHandleAttachment`, `useTriggerDataForwarding`, `useTriggerRegistration`,
  `useImplicitActiveTrigger`, `useOpenStateTransitions`, `usePopupInteractionProps`, `usePopupRootSync`,
  `createPopupOpenState`, `FOCUSABLE_POPUP_PROPS`, `createDefaultInitialFocus`,
  `PayloadChildRenderFunction` (`popupStoreUtils.ts`); `PopupStoreState`/`PopupStoreContext`/
  `popupStoreSelectors`/`PopupTriggerMap`/`createInitialPopupStoreState` (`store.ts`,
  `popupTriggerMap.ts`); `BasePopupHandle` (`popupHandle.ts`); `usePopupHandleStore`
  (`usePopupHandleStore.ts`).
- `packages/react/src/internals/`: `useRenderElement` + merge-props event semantics, `useButton`,
  `useBaseUiId`, `useTransitionStatus`, `useOpenChangeComplete`/`useAnimationsFinished`,
  `createBaseUIEventDetails` + `REASONS`, `transitionStatusMapping`,
  `composite` (`COMPOSITE_KEYS`), `constants` (`CLICK_TRIGGER_IDENTIFIER`), `types`
  (`BaseUIComponentProps`, `NativeButtonProps`).
- `packages/react/src/utils/`: `popupStateMapping` (`popupStateMapping`, `triggerOpenStateMapping`,
  `CommonPopupDataAttributes`/`CommonTriggerDataAttributes`), `InternalBackdrop`,
  `useOpenInteractionType`, `NullStore`.
- `@base-ui/utils/` (public utils package): `store` (ReactStore), `useScrollLock`, `useIsoLayoutEffect`,
  `useId`, `useStableCallback`, `useRefWithInit`, `useEnhancedClickHandler` (`InteractionType`),
  `inertValue`, `fastHooks`, `platform`, `empty`.
- Data-attribute/CSS-var modules are pure constants (`packages/react/src/dialog/popup/DialogPopupDataAttributes.ts:22-25`,
  `DialogBackdropDataAttributes.ts`, `DialogCloseDataAttributes.ts`, `DialogTriggerDataAttributes.ts`,
  `DialogViewportDataAttributes.ts`, `packages/react/src/dialog/popup/DialogPopupCssVars.ts:5`) shared with AlertDialog/Drawer through
  `dialogStateAttributesMapping` (`packages/react/src/dialog/utils/stateAttributesMapping.ts:11-22`).
- Cross-unit reuse of this unit's internals: `useRenderDialogRoot` is parameterized by
  `mode: 'dialog' | 'drawer' | 'alert-dialog'` (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:106`), and drawer parts consume
  dialog-store fields (`viewportElement`, drawer counts — see unexplained section). For
  `ralph/scripts/generate-todo.mjs`: the coarse `blocked-by: [Phase A complete]` default can be narrowed
  to — dialog requires the `utils/popups` framework, `floating-ui-react` interactions, and
  `@base-ui/utils/store`; it does not depend on any other component unit.

## Anything in source not explained by any test

Flagged for the golden-fixture stage and the backward-looking audit; none of these are asserted by
behavior.md:

1. `viewportElement` is written by `DialogViewport` (`packages/react/src/dialog/viewport/DialogViewport.tsx:31`) and declared/selected on
   the dialog store (`packages/react/src/dialog/store/DialogStore.ts:25,47,125`), but nothing inside `packages/react/src/dialog` reads
   it. Its only consumers are drawer parts (`packages/react/src/drawer/viewport/DrawerViewport.tsx:799`,
   `packages/react/src/drawer/root/useDrawerSnapPoints.ts:87-116`,
   `packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:94-99`) — a
   latent cross-unit dependency carried by the shared `useRenderDialogRoot` pipeline that no dialog test
   covers.
2. The `nestedOpenDrawerCount`/`isDrawer` half of the nesting-count pair (`packages/react/src/dialog/root/useDialogRoot.ts:26,101`) is
   unexercised by dialog tests — behavior.md's cross-type nesting entry uses AlertDialog, not Drawer.
3. `closeOnFocusOut={!disablePointerDismissal}` (`packages/react/src/dialog/popup/DialogPopup.tsx:102`): a non-modal dialog closing when
   focus moves outside is documented in the `disablePointerDismissal` JSDoc (`packages/react/src/dialog/root/DialogRoot.tsx:56-57`) but
   no dialog test drives it; behavior.md only covers the outside-press sense of the prop.
4. `REASONS.focusOut` is in the `onOpenChange` reason union (`packages/react/src/dialog/root/DialogRoot.tsx:102`) but is never produced
   or asserted by dialog tests — presumably emitted by FloatingFocusManager's focus-out close path,
   which is itself untested per #3.
5. The `data-base-ui-click-trigger` marker on the trigger (`packages/react/src/dialog/trigger/DialogTrigger.tsx:9,91`;
   `packages/react/src/internals/constants.ts:7`) is consumed by FloatingFocusManager
   (`packages/react/src/floating-ui-react/components/FloatingFocusManager.tsx:374`); no dialog test
   asserts the attribute.
6. `onKeyDown` `COMPOSITE_KEYS` stopPropagation on the popup (`packages/react/src/dialog/popup/DialogPopup.tsx:11,82-86`): no dialog
   test.
7. `inert={inertValue(!open)}` on the internal backdrop (`packages/react/src/dialog/portal/DialogPortal.tsx:37`): no dialog test asserts
   inert toggling (also noted as untested in `specs/utils/inertValue.md`).
8. Backdrop inline `userSelect`/`WebkitUserSelect` (`packages/react/src/dialog/backdrop/DialogBackdrop.tsx:41-44`) and viewport
   `pointerEvents: none` while closed (`packages/react/src/dialog/viewport/DialogViewport.tsx:53-55`): style-only, untested.
9. Enforcement asymmetry vs behavior.md: only `Dialog.Popup` and `Dialog.Viewport` require
   `Dialog.Portal` (via `useDialogPortalContext`, `packages/react/src/dialog/popup/DialogPopup.tsx:45` / `packages/react/src/dialog/viewport/DialogViewport.tsx:22`);
   `DialogBackdrop` renders fine without a portal, and behavior.md's DOM structure section claims the
   portal is required for Backdrop too while its cited test only exercises Viewport.
10. `openMethod` values beyond `'keyboard'`/`'touch'` (`'mouse'`, `'pen'`) and the iOS touch fallback in
    `useOpenMethodTriggerProps` (`packages/react/src/utils/useOpenInteractionType.ts:17-23`) are
    untested (platform-dependent); dialog tests only pin keyboard/touch.
11. `positionerElement` exists in the shared popup state but is never set by any dialog source (dialogs
    have no positioner); it is dead weight for this unit.
12. `DialogStore.setOpen` does not use the shared `applyPopupOpenChange` (no hover/`instantType` logic;
    custom trigger backfill at `packages/react/src/dialog/store/DialogStore.ts:82-86`) — a divergence from the other popup families with
    no dedicated test; the "old trigger passed to `onOpenChange` on close" consequence is covered only
    indirectly by the focus-return entries in behavior.md's Focus management section and the
    `eventDetails.trigger` entry in its Events section.
