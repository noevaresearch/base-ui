# Tooltip — implementation part: trigger, provider

Batch scope: `packages/react/src/tooltip/trigger/` and `packages/react/src/tooltip/provider/`.
Companion to `specs/library/tooltip/behavior.md` (WHAT) — WHY/HOW only for these files.
Shared code under `packages/react/src/floating-ui-react/`, `packages/react/src/utils/popups/`,
and `packages/react/src/internals/` is cited as external-unit dependencies, not re-derived.

## State machine / hooks used

- Store resolution order: optional root context (`useTooltipRootContext(true)`), then
  `usePopupHandleStore(handle)` — a `useSyncExternalStore` view of whichever store the handle
  currently exposes — and the handle store wins when present; if neither exists the trigger throws
  the "must be either used within a <Tooltip.Root> component or provided with a handle" error
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:91-98`; hook in
  `packages/react/src/utils/popups/usePopupHandleStore.ts:17-36`). This is what lets the same
  component work contained, detached, and across a root handoff (behavior.md, structural note).
- Per-trigger identity: `useBaseUiId(idProp)` generates the `base-ui-`-prefixed trigger id that
  keys the registry (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:100`; wrapper in
  `packages/react/src/internals/useBaseUiId.ts:9-11`). `store.useState('isTriggerActive', id)` and
  `store.useState('isOpenedByTrigger', id)` are parameterized selectors
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:101-102`), so `state.open` on the
  rendered element reflects *this trigger's* ownership, not the popup's global open state.
- Registration + data forwarding: `useTriggerDataForwarding` registers the DOM element under
  `thisTriggerId` and forwards `{ payload, closeOnClick, closeDelay }` into the store whenever this
  trigger is (or becomes) the active one
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:109-118`; implementation in
  `packages/react/src/utils/popups/popupStoreUtils.ts:318-388`, including the layout-effect
  migration between stores that makes detached→attached handoffs work).
- Delay group participation: `useDelayGroup(floatingRootContext, { open: isOpenedByThisTrigger })`
  returns `activeIdRef`, `delayRef`, `isInstantPhase`, `hasProvider`
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:121-126`), and the trigger syncs
  `isInstantPhase` back into the tooltip store so Popup/Positioner/Arrow/Viewport can read it
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:129`). This hook is also what closes the
  *previous* tooltip with reason `none` when a new one opens inside a provider
  (`packages/react/src/floating-ui-react/components/FloatingDelayGroup.tsx:219-254`).
- Open-delay resolution, `getOpenDelay()`
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:142-148`): `0` while the delay group is
  active (`hasProvider && activeIdRef.current != null`), else `delay ?? providerDelay ?? OPEN_DELAY`
  — the precedence chain behind behavior.md's "Delay resolution order".
- Hover interaction is delegated to `useHoverReferenceInteraction`
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:175-193`) with the tooltip-specific
  configuration: `mouseOnly: true`, `move: false` (only `mouseenter`/`mouseleave` listeners — no
  `mousemove` tracking on the trigger), `restMs: getOpenDelay` (the "rest before open" delay),
  a `delay()` function that resolves the close delay from the provider group when the trigger has
  no own `closeDelay` (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:181-186` using
  `getDelay` from `packages/react/src/floating-ui-react/hooks/useHoverShared.ts:46-57`),
  `handleClose: safePolygon()` unless `disableHoverablePopup` or `trackCursorAxis === 'both'`,
  `isClosing` reading `transitionStatus === 'ending'`, and a `shouldOpen` veto that returns false
  while a nested trigger is hovered
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:190-192`).
- Focus interaction: `useFocus(floatingRootContext, { enabled: !disabled }).reference`
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:195`) — opens immediately with the
  `trigger-focus` reason (no delay configured) and closes on blur; the shared hook also suppresses
  re-open after Escape/press dismissal
  (`packages/react/src/floating-ui-react/hooks/useFocus.ts:131-243`).
- Nested-trigger suppression (the tooltip-owned part of the nested-tooltip behavior): the trigger
  tracks `isNestedTriggerHoveredRef` plus a dedicated `nestedTriggerOpenTimeout` (`useTimeout`) and
  a local `pointerTypeRef` (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:137-140`).
  `closestEnabledTooltipTrigger` walks shadow roots via `getRootNode()`/`host` to find the nearest
  enabled `[data-base-ui-tooltip-trigger]`
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:52-65`), and `getTargetElement` resolves
  the event target from `composedPath()` first for the same reason
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:33-50`).
  `detectNestedTriggerHover` clears the hover/rest/nested timeouts whenever a nested trigger is
  entered (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:162-173`).
- `handleNestedTriggerHover` (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:197-243`):
  (1) if a nested trigger is hovered and *this* popup is open because of hover, close it with the
  `trigger-hover` reason — deliberately restricted to `lastOpenChangeReason === REASONS.triggerHover`
  so focus/controlled opens are not clobbered; (2) when the pointer leaves a nested trigger back
  into the parent trigger area, schedule a local reopen after `getOpenDelay()` — required because
  `move: false` means the hover hook only hears `mouseenter`/`mouseleave` on the parent itself and
  never sees this transition (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:233-241`).
- Click suppression: `onPointerDown`/`onClick` write `closeOnClick` into the store and call
  `store.cancelPendingOpen` when `closeOnClick && !open`
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:274-285`), which clears a pending
  delayed hover-open without a public open-state change (see `parts/root-store.implementation.md`).
- `disabled` resolution: trigger-level `disabledProp ?? rootDisabled` from the store
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:131-132`) — root `disabled` wins by
  default but a trigger can opt back in; the root also masks `open` at read time (root part).

## Context providers/consumers

- Consumes `TooltipRootContext` (or a handle store with the same narrowed shape,
  `TooltipHandleStore` in `packages/react/src/tooltip/store/TooltipStore.ts:59-62`) and
  `TooltipProviderContext` — the provider's `delay` number
  (`packages/react/src/tooltip/provider/TooltipProviderContext.ts:7-11`, consumed at
  `packages/react/src/tooltip/trigger/TooltipTrigger.tsx:120`).
- Indirectly participates in the delay-group context created by `TooltipProvider`'s
  `FloatingDelayGroup` (below).
- Produces nothing for other parts directly; everything it contributes (registration, payload,
  `closeOnClick`, `closeDelay`, `isInstantPhase`) flows through the store.

## DOM/portal strategy and why

- Renders a `<button>` via `useRenderElement`
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:250-293`), merging, in order: hover
  props, focus props, the root-provided `triggerProps` (only when this trigger mounts the popup or
  when cursor tracking is on — `shouldApplyRootTriggerProps`,
  `packages/react/src/tooltip/trigger/TooltipTrigger.tsx:245-246`), the local handlers, the
  trigger `id`, and two marker attributes:
  `data-trigger-disabled` when disabled and the internal
  `data-base-ui-tooltip-trigger` identifier only while *enabled*
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:286-288` — the enabled-only marker is
  what makes `closestEnabledTooltipTrigger` skip disabled nested triggers, which is why a disabled
  nested trigger lets the parent open, per behavior.md "Nested tooltips").
- State `{ open: isOpenedByThisTrigger }` with `triggerOpenStateMapping` yields
  `data-popup-open` (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:248,292`; mapping in
  `packages/react/src/utils/popupStateMapping.ts:30-37`).
- Ref stack `[forwardedRef, registerTrigger, triggerElementRef]`
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:252`) — the registration ref is stable
  so merged-ref consumers keep working across store migrations.

## Provider specifics

- `TooltipProvider` wraps children in `FloatingDelayGroup` with `{ open: delay, close: closeDelay }`
  and `timeoutMs: timeout (default 400)` and publishes the raw `delay` through
  `TooltipProviderContext` (`packages/react/src/tooltip/provider/TooltipProvider.tsx:12-24`).
  The instant-phase window, adjacent-tooltip open takeover, and the `closeDelay` honor-all-latest
  semantics are implemented inside `FloatingDelayGroup`/`useDelayGroup`
  (`packages/react/src/floating-ui-react/components/FloatingDelayGroup.tsx:141-281`) — cited, not
  re-derived here. The default `timeout = 400` lives at
  `packages/react/src/tooltip/provider/TooltipProvider.tsx:13`.

## Dependencies on other Base UI internals

- `packages/react/src/floating-ui-react/` — `safePolygon`, `useDelayGroup`, `useFocus`,
  `useHoverReferenceInteraction`, `useHoverInteractionSharedState` (shared per-store hover
  instance holding pointer type and the open/rest timeouts,
  `packages/react/src/floating-ui-react/hooks/useHoverInteractionSharedState.ts:11-52`),
  `getDelay` (`hooks/useHoverShared.ts`), `contains` (`utils/element`),
  `isMouseLikePointerType` (`utils/event`).
- `packages/react/src/utils/popups/` — `useTriggerDataForwarding`, `usePopupHandleStore`.
- `packages/react/src/internals/` — `useRenderElement`, `useBaseUiId`,
  `createBaseUIEventDetails`, `REASONS`, `BaseUIEvent` type.
- `packages/utils/` — `fastComponentRef`, `useTimeout`, `useValueAsRef`.
- `@floating-ui/utils/dom` — `isElement`.
- `packages/react/src/utils/popupStateMapping.ts` — `triggerOpenStateMapping`.
- No `wraps-external:` field exists on this unit's `TODO.md` entry (`TODO.md:569-579`).

## Anything in source not explained by any test

- The provider `timeout` default `400` (`packages/react/src/tooltip/provider/TooltipProvider.tsx:13`):
  tests exercise the instant window with explicit values; no test asserts the default number.
- The internal `data-base-ui-tooltip-trigger` identifier attribute
  (`packages/react/src/tooltip/trigger/TooltipTrigger.tsx:31`): exercised only implicitly through
  nested-trigger tests; no test queries the attribute itself.
- `TooltipTrigger.spec.tsx` (`packages/react/src/tooltip/trigger/TooltipTrigger.spec.tsx:1-6`) is
  a type-only test file (render-prop `props` must be typed, not `any`); it has no runtime
  counterpart and is invisible to the test suite.
- The `guardStaleOpen` option exists on the shared hover hook but the tooltip never enables it —
  noted only to record that Chrome's dropped-`mouseleave` backup is deliberately not used for
  tooltips (no test could cover its absence).
