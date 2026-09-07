# Tooltip — implementation part: root, store, handle, constants, index

Batch scope: `packages/react/src/tooltip/root/`, `packages/react/src/tooltip/store/`,
`packages/react/src/tooltip/utils/`, and the barrel files. Companion to
`specs/library/tooltip/behavior.md` (WHAT) — this file is WHY/HOW only for these files.
Shared code under `packages/react/src/utils/popups/`, `packages/react/src/floating-ui-react/`,
and `packages/react/src/internals/` is cited as external-unit dependencies, not re-derived.

## State machine / hooks used

- `TooltipStore` extends `ReactStore` (`packages/react/src/tooltip/store/TooltipStore.ts:64-68`)
  and is the single state machine for the whole component. Its state extends the generic
  `PopupStoreState` with tooltip-only keys: `disabled`, `instantType`, `isInstantPhase`,
  `trackCursorAxis`, `disableHoverablePopup`, `openChangeReason`, `closeOnClick`, `closeDelay`,
  `adaptiveOrigin` (`packages/react/src/tooltip/store/TooltipStore.ts:19-29`). It adds
  tooltip-specific selectors on top of `popupStoreSelectors`
  (`packages/react/src/tooltip/store/TooltipStore.ts:35-47`).
- `store.setOpen` is the only open-transition entry point; it delegates to the shared
  `applyPopupOpenChange` and contributes `openChangeReason` as `extraState`
  (`packages/react/src/tooltip/store/TooltipStore.ts:82-89`). That shared call fires
  `onOpenChange`, honors `details.cancel()`, dispatches to the floating-root context, maps the
  reason to an `instantType`, and commits state synchronously (`flushSync`) for hover opens —
  see `packages/react/src/utils/popups/popupStoreUtils.ts:241-308`. The reason literals come from
  `packages/react/src/internals/reason-parts.ts:1-42` (`none`, `triggerHover`, `triggerFocus`,
  `triggerPress`, `escapeKey`, `disabled`, `imperativeAction` are the tooltip-relevant subset).
- `store.cancelPendingOpen` is the click-suppression path: it dispatches a `trigger-press` close
  through `floatingRootContext.dispatchOpenChange` only — bypassing `applyPopupOpenChange`, so no
  public open-state change is reported while the pending hover-open is cleared
  (`packages/react/src/tooltip/store/TooltipStore.ts:91-97`).
- `createNullTooltipStore` builds the inert fallback store used by detached triggers while no root
  is attached: a real initial state/context (so the trigger registry works pre-attachment) wrapped
  in `NullStore` with `setOpen`/`cancelPendingOpen` stubbed to `NOOP`
  (`packages/react/src/tooltip/store/TooltipStore.ts:107-116`).
- `TooltipRoot` creates the store exactly once via `usePopupRootStore`, passing `defaultOpen`,
  `openProp`, `defaultTriggerId`, and `triggerId` into the initial state, plus the `floatingId`
  and `nested` flag resolved on first render
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:49-61`). The factory lives in
  `packages/react/src/utils/popups/popupStoreUtils.ts:73-96` and also wires the synced floating
  root context.
- Controlled/uncontrolled props sync through `store.useControlledProp('openProp', openProp)` and
  `store.useControlledProp('triggerIdProp', triggerIdProp)`
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:63-64`); the effective `open` is
  `openProp ?? state.open` via the store's `open` selector, and the effective active trigger id is
  `triggerIdProp ?? state.activeTriggerId`. Callbacks are context-synced with
  `store.useContextCallback` (`packages/react/src/tooltip/root/TooltipRoot.tsx:66-67`).
- `disabled` is stored in the state (synced via `store.useSyncedValues`
  `packages/react/src/tooltip/root/TooltipRoot.tsx:76-80`) but the read side masks instead of
  closing: `const open = !disabled && openState`
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:69-70`). A separate layout effect actively
  closes an already-open tooltip when `disabled` turns on, with the `disabled` reason
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:97-101`) — that is the transition behind
  behavior.md's "closes an already-open tooltip when it becomes disabled".
- The instant-animation coordinator is a layout effect over `transitionStatus`,
  `isInstantPhase`, and `lastOpenChangeReason`
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:103-119`). Two cases force
  `instantType: 'delay'`: (1) opening during the provider's instant phase, and (2) a close whose
  reason is `REASONS.none` (i.e. closing because another tooltip opened). The prior `instantType`
  is stashed in `previousInstantTypeRef` and restored when neither condition holds
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:93-95,115-118`).
- A small effect clears `payload` whenever the popup is open with no active trigger
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:121-127`).
- Active-trigger ownership: `useImplicitActiveTrigger(store, { closeOnActiveTriggerUnmount: true })`
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:82`) — the shared hook reconciles the trigger
  registry against the active id and requests a close (reason `none`) after a microtask when the
  active trigger unmounts, so a same-tick replacement can register first
  (`packages/react/src/utils/popups/popupStoreUtils.ts:416-537`).
- Mount/transition lifecycle: `useOpenStateTransitions(open, store)` returns `forceUnmount` and
  `transitionStatus` (`packages/react/src/tooltip/root/TooltipRoot.tsx:83`). It syncs
  `mounted`/`transitionStatus`/`preventUnmountingOnClose` into the store and unmounts after the
  exit transition unless `preventUnmountingOnClose` was set
  (`packages/react/src/utils/popups/popupStoreUtils.ts:554-601`); `forceUnmount` is what
  `actionsRef.unmount()` exposes.
- `actionsRef` is wired with `React.useImperativeHandle`: `unmount: forceUnmount`, and
  `close: () => store.setOpen(false, details(REASONS.imperativeAction))`
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:129-136`).
- Floating-side interactions live in a null-rendering child, `TooltipInteractions`
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:247-280`). It runs `useDismiss` with
  `referencePress: () => store.select('closeOnClick')` (lazy getter so the latest trigger-level
  prop is honored) and `useClientPoint` with the `trackCursorAxis` mapping, then merges the two
  hooks' `reference` props (identical object for active/inactive triggers, see the comment at
  `packages/react/src/tooltip/root/TooltipRoot.tsx:267-268`) and pushes them into the store via
  `usePopupInteractionProps` as `activeTriggerProps`/`inactiveTriggerProps`/`popupProps`
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:269-277`). The whole child is gated by
  `shouldRenderInteractions = open || mounted || (!disabled && trackCursorAxis !== 'none')`
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:138`) — cursor capture must exist before the
  open so the first delayed hover already has a cursor position.
- `TooltipHandle` extends the shared `BasePopupHandle` with `createNullTooltipStore` as the
  fallback and `'Tooltip'` as the warning prefix
  (`packages/react/src/tooltip/store/TooltipHandle.ts:12-18`). `open(triggerId)` maps to
  `openByTrigger` (throws on an unregistered id — anchored-popup behavior), `close()` to
  `closePopup`, and `isOpen` reads the attached store's `open` selector
  (`packages/react/src/tooltip/store/TooltipHandle.ts:28-46`). The base class owns the attachment
  stack, the "no root mounted" warnings, and the missing-trigger error
  (`packages/react/src/utils/popups/popupHandle.ts:215-287`). Factory:
  `createTooltipHandle` (`packages/react/src/tooltip/store/TooltipHandle.ts:52-54`).
- `OPEN_DELAY = 600` is the default open delay when neither trigger `delay` nor a provider
  `delay` is set (`packages/react/src/tooltip/utils/constants.ts:1`).
- The barrel maps parts into the `Tooltip` namespace, including `Provider` and
  `createHandle`/`Handle` (`packages/react/src/tooltip/index.parts.ts:1-12`); `index.ts`
  re-exports all part type namespaces (`packages/react/src/tooltip/index.ts:1-10`).

## Context providers/consumers

- `TooltipRootContext.Provider value={store}` — the context value *is* the store instance, not a
  snapshot object (`packages/react/src/tooltip/root/TooltipRoot.tsx:141`; type alias
  `packages/react/src/tooltip/root/TooltipRootContext.ts:5-7`). Every downstream part
  (`Trigger`, `Portal`, `Positioner`, `Popup`, `Arrow`, `Viewport`) consumes it through
  `useTooltipRootContext`, which throws the "TooltipRootContext is missing" error unless called
  with `optional: true` (`packages/react/src/tooltip/root/TooltipRootContext.ts:9-20`).
- `PopupHandleAttachment` is rendered as a direct child when a `handle` prop is present
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:142`) and attaches the live store in a layout
  effect; it is intentionally rendered before interactions and user children so the attachment
  lands before descendant layout effects
  (`packages/react/src/utils/popups/popupStoreUtils.ts:98-120`).
- The store's context object carries `onOpenChange`/`onOpenChangeComplete` callbacks, the shared
  `popupRef`, and the `triggerElements` registry (`packages/react/src/tooltip/store/TooltipStore.ts:141-148`).

## DOM/portal strategy and why

- `TooltipRoot` renders no DOM element. It renders the context provider, the optional
  null-rendering handle attachment and interactions child, then `children` — calling the
  render-prop form with `{ payload }` for the detached-triggers API
  (`packages/react/src/tooltip/root/TooltipRoot.tsx:140-148`; type
  `PayloadChildRenderFunction` in `packages/react/src/utils/popups/popupStoreUtils.ts:390-392`).
- No portal/positioning decisions exist at this level; the popup subtree mounts under `Portal`
  (see `parts/portal-positioner.implementation.md`).

## Dependencies on other Base UI internals

- `packages/react/src/utils/popups/` — `usePopupRootStore`, `PopupHandleAttachment`,
  `useImplicitActiveTrigger`, `useOpenStateTransitions`, `usePopupInteractionProps`,
  `PayloadChildRenderFunction` (`popupStoreUtils.ts`), `applyPopupOpenChange`,
  `createInitialPopupStoreState`, `PopupStoreContext`, `popupStoreSelectors`, `PopupTriggerMap`,
  `PopupTriggerStoreKeys` (`store.ts`, `popupTriggerMap.ts`), `BasePopupHandle`
  (`popupHandle.ts`).
- `packages/react/src/floating-ui-react/` — `useDismiss`, `useClientPoint`
  (used in `TooltipInteractions`).
- `packages/react/src/internals/` — `createBaseUIEventDetails`, `REASONS`.
- `packages/utils/` — `fastComponent`/`fastComponentRef` (`@base-ui/utils/fastHooks`; batches the
  store's `useSyncExternalStore` subscriptions, `packages/utils/src/fastHooks.ts:29-63`),
  `ReactStore` (`@base-ui/utils/store`), `NullStore`, `useIsoLayoutEffect`, `mergeProps`,
  `EMPTY_OBJECT`/`NOOP`.
- No `wraps-external:` field exists on this unit's `TODO.md` entry (`TODO.md:565-575`), so no
  third-party package delegation applies.

## Anything in source not explained by any test

- The payload-clear effect (`packages/react/src/tooltip/root/TooltipRoot.tsx:121-127`): no test
  asserts that `payload` is dropped while open with a null active trigger id; the reset-on-unmount
  tests only cover handle-level state.
- The `previousInstantTypeRef` restore branch (`packages/react/src/tooltip/root/TooltipRoot.tsx:115-118`):
  tests cover `data-instant="delay"` appearing during the group instant phase, but no test asserts
  the *restored* instant value after the phase ends (they assert absence of the attribute, not the
  restore mechanism).
- The `flushSync` hover-open commit is an internal of the shared `applyPopupOpenChange`; tooltip
  tests only observe its downstream effect (animation timing), so the mechanism itself is only
  covered indirectly (see behavior.md, section "Popup animation attributes").
- `TooltipRootState` is an empty interface (`packages/react/src/tooltip/root/TooltipRoot.tsx:151`)
  — the root keeps no component-level React state; everything lives in the store. Not asserted
  anywhere (and not directly assertable), recorded here for completeness.
