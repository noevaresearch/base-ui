# Tooltip — implementation spec (Stage 2: implementation mining)

Unit: `packages/react/src/tooltip/`. This spec explains WHY/HOW the behavior documented in
`specs/library/tooltip/behavior.md` is produced. It was mined in batched form (the unit's
`TODO.md` entry sets `needs-batched-mining: true`, `TODO.md:569-579`); the per-subdirectory
partition below mirrors the unit's own file layout, and each part file carries the depth:

- `parts/root-store.implementation.md` — `root/`, `store/`, `utils/constants.ts`, barrel files.
  The `TooltipStore` state machine (extension of the shared popup store), `TooltipRoot`'s
  effect set (disabled-close, instant-type coordination, payload reset), the `actionsRef`
  imperative surface, and the `TooltipHandle`/fallback-store pair behind detached triggers
  (`packages/react/src/tooltip/store/TooltipStore.ts:64-98`,
  `packages/react/src/tooltip/root/TooltipRoot.tsx:49-148`,
  `packages/react/src/tooltip/store/TooltipHandle.ts:12-54`).
- `parts/trigger-provider.implementation.md` — `trigger/`, `provider/`. Store resolution
  (context or handle), trigger registration and per-trigger data forwarding, the
  hover/focus/delay-group hook composition with tooltip-specific options (`mouseOnly`,
  `move: false`, `restMs` = resolved open delay, `safePolygon` unless hoverability is disabled),
  the tooltip-owned nested-trigger suppression/reopen machinery, and the provider's delay-group
  wiring (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:91-296`,
  `packages/react/src/tooltip/provider/TooltipProvider.tsx:12-24`).
- `parts/portal-positioner.implementation.md` — `portal/`, `positioner/`. The `mounted`-gated
  portal with `keepMounted` propagation by context, the composition-guard errors, and the
  positioner as a thin configuration of the shared anchor-positioning engine — including the
  `inert` → `pointer-events: none` rule that encodes `disableHoverablePopup`/`trackCursorAxis="both"`
  (`packages/react/src/tooltip/portal/TooltipPortal.tsx:21-33`,
  `packages/react/src/tooltip/positioner/TooltipPositioner.tsx:45-108`).
- `parts/popup-arrow-viewport.implementation.md` — `popup/`, `arrow/`, `viewport/`. The popup's
  split `onOpenChangeComplete` responsibility (open here, close in the root), popup-side hover
  close with the active trigger's `closeDelay`, the arrow as a pure consumer of the positioning
  context, and the viewport's morph state machine (DOM-clone snapshots, keyed remounts,
  adaptive-origin registration)
  (`packages/react/src/tooltip/popup/TooltipPopup.tsx:27-75`,
  `packages/react/src/tooltip/arrow/TooltipArrow.tsx:22-43`,
  `packages/react/src/tooltip/viewport/TooltipViewport.tsx:23-45`).

## Cross-cutting implementation details (whole-unit level)

These only make sense across parts; each part file covers its own slice.

### One store, passed as context

A single `TooltipStore` instance is created by `TooltipRoot` (via `usePopupRootStore`) and is
itself the context value of `TooltipRootContext` — there is no intermediate snapshot object
(`packages/react/src/tooltip/root/TooltipRoot.tsx:49-61,141`;
`packages/react/src/tooltip/root/TooltipRootContext.ts:5-7`). Every part subscribes with
parameterized store selectors (`useState('isTriggerActive', id)`,
`useState('triggerProps', isMountedByThisTrigger)`) rather than receiving computed props, which
is how contained, detached, and handle-migrated triggers all read identical state. Detached
triggers read the same interface from a handle-exposed store (`TooltipHandleStore` pick,
`packages/react/src/tooltip/store/TooltipStore.ts:59-62`) whose fallback variant makes every
mutation a no-op (`packages/react/src/tooltip/store/TooltipStore.ts:107-116`).

### The open-change pipeline

All open/close requests converge on `store.setOpen` → the shared `applyPopupOpenChange`
(`packages/react/src/tooltip/store/TooltipStore.ts:82-89`,
`packages/react/src/utils/popups/popupStoreUtils.ts:241-308`), in this order:
`onOpenChange` (cancellable via `details.cancel()`) → `floatingRootContext.dispatchOpenChange`
(notifies the floating-ui interaction hooks) → state commit with
`createPopupOpenState` (which carries `preventUnmountOnClose`, keeps the previous active trigger
during exit for animation/focus, and maps reason → `instantType`: focus opens get `'focus'`,
press/Escape closes get `'dismiss'`, hover gets `undefined`). Hover commits are flushed
synchronously so `getAnimations()` observes the new state. The reason literal for every
`onOpenChange` callback in behavior.md's "Events" section originates from
`packages/react/src/internals/reason-parts.ts:1-42`.

On top of that shared pipeline, `TooltipRoot` forces `instantType: 'delay'` in two cases —
opening inside the provider's instant phase, or closing with reason `'none'` (another tooltip
opened) — and restores the previous value afterwards
(`packages/react/src/tooltip/root/TooltipRoot.tsx:103-119`). This is the mechanism behind both
`data-instant="delay"` on adjacent tooltips and the immediate unmount of an exiting tooltip when
a sibling opens.

### Active-trigger model

Triggers register their DOM element in a shared `PopupTriggerMap` on the store context and
forward trigger-owned data (`payload`, `closeOnClick`, `closeDelay`) whenever they are or become
active (`packages/react/src/utils/popups/popupStoreUtils.ts:318-388`, called from
`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:109-118`). The root's
`useImplicitActiveTrigger(..., { closeOnActiveTriggerUnmount: true })` claims the sole trigger
when none is set, reconciles id/element changes, and closes with reason `'none'` (cancellable)
when the active trigger unmounts
(`packages/react/src/utils/popups/popupStoreUtils.ts:416-537`,
`packages/react/src/tooltip/root/TooltipRoot.tsx:82`). Controlled ownership is expressed through
the `triggerIdProp ?? state.activeTriggerId` selector, so `triggerId`/`defaultTriggerId` and
`handle.open(id)` are the same mechanism as organic hover/focus opens.

### Handle attachment lifecycle

`PopupHandleAttachment` is rendered by the root before interactions and children so its layout
effect attaches the live store to the handle before any descendant effect runs; detached triggers
follow the store pointer via `useSyncExternalStore` and migrate their registration in a layout
effect keyed on store/id (`packages/react/src/utils/popups/popupStoreUtils.ts:98-120`,
`packages/react/src/utils/popups/usePopupHandleStore.ts:17-36`,
`packages/react/src/utils/popups/popupStoreUtils.ts:369-374`). The base handle class owns the
attachment stack (transient root overlap), the dev warnings, and the missing-trigger error
(`packages/react/src/utils/popups/popupHandle.ts:68-287`).

### Floating-side wiring

The store holds a `FloatingRootStore` whose `referenceElement` is the active trigger element and
whose `floatingElement` is the positioner element, kept in sync each render by
`useSyncedFloatingRootContext` (`packages/react/src/floating-ui-react/hooks/useSyncedFloatingRootContext.ts:38-114`).
The trigger-attached hooks (`useHoverReferenceInteraction`, `useFocus`) and the root-attached
hooks (`useDismiss`, `useClientPoint` in `TooltipInteractions`) all communicate exclusively
through that context's `setOpen` and event bus; the popup receives its share of the props
(`popupProps` — the dismiss handlers) through the store rather than through React nesting, which
is why Escape works with listeners mounted nowhere near the popup DOM.

## Dependencies on other Base UI internals (unit-wide)

Aggregated from the part files; the per-part files list the exact call sites.

- `packages/react/src/utils/popups/` — store factory, open-change pipeline, trigger
  registration/forwarding, implicit active trigger, transitions, handle base class.
- `packages/react/src/floating-ui-react/` — `useHoverReferenceInteraction`,
  `useHoverFloatingInteraction`, `useFocus`, `useDismiss`, `useClientPoint`, `useDelayGroup`,
  `FloatingDelayGroup`, `safePolygon`, `useHoverInteractionSharedState`, `getDelay`, tree utils,
  `useSyncedFloatingRootContext`, `FloatingRootStore`, portal node creation.
- `packages/react/src/internals/` — `useAnchorPositioning` (+ `arrow`, `hide`,
  `adaptiveOrigin` middleware integration), `useRenderElement`, `useBaseUiId`,
  `useOpenChangeComplete`, `useTransitionStatus`, `getDisabledMountTransitionStyles`,
  `createBaseUIEventDetails`, `REASONS`, `POPUP_COLLISION_AVOIDANCE`, state-attributes mapping.
- `packages/react/src/utils/` — `usePositioner`, `usePopupViewport`, `usePopupAutoResize`,
  `FloatingPortalLite`, `popupStateMapping`, `PopupTriggerMap`, `adaptiveOriginMiddleware`.
- `packages/utils/` — `ReactStore`/`NullStore`, `fastComponent`/`fastComponentRef`,
  `useTimeout`, `useIsoLayoutEffect`, `useStableCallback`, `useValueAsRef`,
  `useAnimationFrame`, `mergeProps`, `EMPTY_OBJECT`.
- `@floating-ui/*` — `@floating-ui/utils/dom` (`isElement`), `@floating-ui/react-dom`
  (re-exported middleware primitives used by the shared positioning engine).

No `wraps-external:` field exists on this unit's `TODO.md` entry (`TODO.md:569-579`), so no
external-package delegation applies — everything above is internal to this repository.

## Anything in source not explained by any test (consolidated)

The full details with citations live in each part's final section; consolidated list for the
golden-fixture stage and the backward-looking audit loop:

- Payload-clear effect in the root when open with a null active trigger id
  (`parts/root-store.implementation.md`).
- `instantType` restore mechanism after the instant phase ends
  (`parts/root-store.implementation.md`).
- Provider `timeout` default `400` (`parts/trigger-provider.implementation.md`).
- Internal `data-base-ui-tooltip-trigger` marker attribute
  (`parts/trigger-provider.implementation.md`).
- `TooltipTrigger.spec.tsx` / `TooltipPositioner.spec.tsx` are type-only test files with no
  runtime counterpart (`parts/trigger-provider.implementation.md`,
  `parts/portal-positioner.implementation.md`).
- `'tracking-cursor'` instant value on the positioner
  (`parts/portal-positioner.implementation.md`).
- Positioner `role="presentation"` and `data-anchor-hidden`
  (`parts/portal-positioner.implementation.md`).
- Positioner defaults `side='top'`/`align='center'` (behavior.md flags them UNVERIFIED; source
  values recorded) (`parts/portal-positioner.implementation.md`).
- `--positioner-width`/`--positioner-height` and viewport `--popup-width`/`--popup-height` CSS
  variables (`parts/portal-positioner.implementation.md`,
  `parts/popup-arrow-viewport.implementation.md`).
- Popup `tabIndex: -1` focusability (behavior.md flags popup focusability as UNVERIFIED; source
  answer recorded) and the popup/root split of `onOpenChangeComplete` responsibility
  (`parts/popup-arrow-viewport.implementation.md`).
- The viewport's `data-starting-style`/`data-ending-style` replay choreography
  (`parts/popup-arrow-viewport.implementation.md`).
