# Tooltip — implementation part: popup, arrow, viewport

Batch scope: `packages/react/src/tooltip/popup/`, `packages/react/src/tooltip/arrow/`, and
`packages/react/src/tooltip/viewport/`. Companion to `specs/library/tooltip/behavior.md` (WHAT) —
WHY/HOW only for these files. Shared code under `packages/react/src/utils/` and
`packages/react/src/internals/` is cited as external-unit dependencies, not re-derived.

## State machine / hooks used

### Popup

- `TooltipPopup` reads the store for `open`, `instantType`, `transitionStatus`, `popupProps`,
  `floatingRootContext`, `disabled`, and `closeDelay`
  (`packages/react/src/tooltip/popup/TooltipPopup.tsx:30-36`). `closeDelay` in the store is the
  *active trigger's* forwarded value (written by `useTriggerDataForwarding`, see
  `parts/trigger-provider.implementation.md`), so the popup-side hover handling always uses the
  owning trigger's setting.
- `useOpenChangeComplete({ open, ref: store.context.popupRef, onComplete })` reports the *open*
  completion: it calls `store.context.onOpenChangeComplete?.(true)` only in the `open` branch
  (`packages/react/src/tooltip/popup/TooltipPopup.tsx:38-46`). The *close* completion is reported
  from the root's `useOpenStateTransitions` `forceUnmount` path instead
  (`packages/react/src/utils/popups/popupStoreUtils.ts:577-598`) — the two halves of behavior.md's
  `onOpenChangeComplete` live in different components.
- `useHoverFloatingInteraction(floatingContext, { enabled: !disabled, closeDelay })`
  (`packages/react/src/tooltip/popup/TooltipPopup.tsx:48-51`) attaches the popup-side hover
  listeners: mouseenter on the popup cancels the pending close timeout (which is what makes the
  popup hoverable — the "survive the pointer trip" behavior), mouseleave schedules a close after
  `closeDelay`, and moving into another enabled trigger hands the popup over instead of closing
  (`packages/react/src/floating-ui-react/hooks/useHoverFloatingInteraction.ts:158-257`).
- Element assembly (`packages/react/src/tooltip/popup/TooltipPopup.tsx:63-73`): the ref stack
  registers `store.context.popupRef` (used by transition-completion detection) and
  `store.popupElement` (fed to Floating UI as the floating element when a `Viewport` is present,
  via `treatPopupAsFloatingElement: false` + `positionerElement` default — see the sync hook in
  `packages/react/src/floating-ui-react/hooks/useSyncedFloatingRootContext.ts:53-55`); props are
  `FOCUSABLE_POPUP_PROPS` (`tabIndex: -1` + the focusable marker attribute,
  `packages/react/src/utils/popups/popupStoreUtils.ts:32-35`), the store's `popupProps` (the
  dismiss-hook handlers pushed by the root's `TooltipInteractions`, i.e. where Escape closes from
  a document-level listener), and `getDisabledMountTransitionStyles(transitionStatus)` which adds
  `style="transition: none"` during the `starting` phase
  (`packages/react/src/internals/getDisabledMountTransitionStyles.ts:5-9`,
  `packages/react/src/internals/constants.ts:5`) — that, combined with the positioning hook's
  `opacity: 0` pre-position style
  (`packages/react/src/internals/useAnchorPositioning.ts:529-531`), is the mechanism behind
  behavior.md's "inline `opacity: 0` pre-positioning is removed before user CSS transitions run".
- State → attributes: `popupTransitionStateMapping` maps `open` to `data-open`/`data-closed` and
  `transitionStatus` to `data-starting-style`/`data-ending-style`
  (`packages/react/src/utils/popupStateMapping.ts:63-70`); `side`, `align`, and `instant` fall
  through to the default state→`data-*` kebab-case fallback
  (`packages/react/src/internals/getStateAttributesProps.ts:25-27`) producing
  `data-side`/`data-align`/`data-instant`.

### Arrow

- Pure consumer of the positioner context: takes `arrowRef`, `side`, `align`,
  `arrowUncentered`, `arrowStyles` (`packages/react/src/tooltip/arrow/TooltipArrow.tsx:23`) and
  merges `arrowRef` into the element ref so the shared `arrow()` middleware measures the real
  element (`packages/react/src/tooltip/arrow/TooltipArrow.tsx:36-41`; middleware call site in
  `packages/react/src/internals/useAnchorPositioning.ts:372-382`, which substitutes a throwaway
  element when no arrow is rendered so `transform-origin` math still works).
- Renders `style: arrowStyles` (absolute `top`/`left` from the middleware) and hardcodes
  `aria-hidden: true` (`packages/react/src/tooltip/arrow/TooltipArrow.tsx:39`).
- State `{ open, side, align, uncentered, instant }` with `popupStateMapping` produces
  `data-open`/`data-closed`, `data-side`, and (via the default fallback)
  `data-uncentered`/`data-align`/`data-instant`
  (`packages/react/src/tooltip/arrow/TooltipArrow.tsx:28-40`;
  `packages/react/src/utils/popupStateMapping.ts:48-61`;
  `packages/react/src/internals/getStateAttributesProps.ts:25-27`). `arrowUncentered` is the
  middleware's `centerOffset !== 0`
  (`packages/react/src/internals/useAnchorPositioning.ts:603`) — the "arrow padding exceeds
  available space" signal.

### Viewport

- `TooltipViewport` is a thin adapter over the shared `usePopupViewport`, passing the store, the
  resolved `side` from the positioner context, and the children
  (`packages/react/src/tooltip/viewport/TooltipViewport.tsx:23-32`).
- `usePopupViewport` owns the whole morph state machine; key decisions relevant to the tooltip
  behavior claims: it registers the `adaptiveOrigin` middleware on the store for the positioner
  to pick up and removes it on unmount
  (`packages/react/src/utils/usePopupViewport.tsx:112-117` — the switch between transform and
  top/left positioning behavior.md observes); it captures a *DOM clone* of the current content in
  a plain wrapper element whenever content is stable, because previous React nodes may be
  stateful (`packages/react/src/utils/usePopupViewport.tsx:204-224`); on trigger change it moves
  the captured node into a `data-previous` container (`inert`, absolutely positioned, carrying
  `--popup-width`/`--popup-height` frozen dimensions) alongside the keyed `data-current`
  container (`packages/react/src/utils/usePopupViewport.tsx:226-275`); the current container's
  key bumps on trigger change and again when a lagging payload arrives
  (`packages/react/src/utils/usePopupViewport.tsx:364-396`); cleanup after the current container's
  animations finish uses `useAnimationsFinished` + `useAnimationFrame` +
  an `AbortController` that is re-armed when the current container remounts mid-morph
  (`packages/react/src/utils/usePopupViewport.tsx:137-202`); and
  `data-activation-direction` is derived from the center-to-center offset between previous and
  new trigger rects with a 5px tolerance per axis
  (`packages/react/src/utils/usePopupViewport.tsx:169-176,307-362`).
- The viewport state exposes `activationDirection`, `transitioning`, and the tooltip `instantType`
  (`packages/react/src/tooltip/viewport/TooltipViewport.tsx:26-38`);
  `popupViewportStateMapping` maps `activationDirection` explicitly and lets `transitioning`/`instant`
  fall through to the default `data-transitioning`/`data-instant` mapping
  (`packages/react/src/utils/usePopupViewport.tsx:21-30`;
  `packages/react/src/internals/getStateAttributesProps.ts:25-27`).

## Context providers/consumers

- All three consume `TooltipRootContext` (store) and `TooltipPositionerContext`
  (`packages/react/src/tooltip/popup/TooltipPopup.tsx:27-28`,
  `packages/react/src/tooltip/arrow/TooltipArrow.tsx:22-23`,
  `packages/react/src/tooltip/viewport/TooltipViewport.tsx:23-24`).
- Nothing here provides context to siblings; the popup is the leaf of the
  `Root → Trigger → Portal → Positioner → Popup` chain (behavior.md, section
  "DOM structure & portal behavior").

## DOM/portal strategy and why

- The popup renders a `<div>` *inside* the portal (via the positioner), so all positioning stays
  on the positioner element while content/animation attributes live on the popup.
- The arrow renders a `<div>` (despite conforming as a generic `Element` in tests) with inline
  arrow styles and `aria-hidden`.
- The viewport renders a `<div>` whose children are replaced by the `data-current`/
  `data-previous` container structure during morphs; the previous container is populated
  imperatively via `replaceChildren` with the cloned nodes
  (`packages/react/src/utils/usePopupViewport.tsx:267-275`) — React never re-renders the old
  content, which is what allows arbitrary stateful children to be animated out.

## Dependencies on other Base UI internals

- `packages/react/src/internals/` — `useRenderElement`, `useOpenChangeComplete`,
  `getDisabledMountTransitionStyles`, `useAnchorPositioning` types (`Side`/`Align`),
  `useTransitionStatus` type, `getStateAttributesProps` (default state mapping).
- `packages/react/src/utils/` — `popupStateMapping`/`popupTransitionStateMapping`,
  `popupViewportStateMapping`, `usePopupViewport`, `FOCUSABLE_POPUP_PROPS` (from
  `utils/popups/popupStoreUtils.ts`).
- `packages/react/src/floating-ui-react/` — `useHoverFloatingInteraction`
  (popup-side hover close).
- `packages/utils/` — `inertValue`, `useAnimationFrame`, `usePreviousValue`,
  `useIsoLayoutEffect`, `useStableCallback` (consumed inside the shared viewport hook).
- No `wraps-external:` field exists on this unit's `TODO.md` entry (`TODO.md:565-575`).

## Anything in source not explained by any test

- `FOCUSABLE_POPUP_PROPS` gives the popup `tabIndex: -1`
  (`packages/react/src/utils/popups/popupStoreUtils.ts:32-35`): behavior.md explicitly records
  popup focusability as UNVERIFIED; the source answer is "focusable programmatically, not
  tabbable", but no test asserts either.
- `--popup-width`/`--popup-height` on the `data-previous` container
  (`packages/react/src/utils/usePopupViewport.tsx:240-251`): no tooltip test asserts these CSS
  variables.
- The `data-starting-style`/`data-ending-style` choreography inside the viewport morph
  (`packages/react/src/utils/usePopupViewport.tsx:253,259` and the re-arm effect at 184-202):
  tooltip tests observe the containers, `inert`, and `data-transitioning`, but the
  starting-style toggling that replays the entry transition after a lagging-payload remount is
  only covered indirectly (behavior.md, "a lagging payload update … restarts its entry
  transition").
- The popup's `useOpenChangeComplete` reports open-completion only; nothing in the tooltip tests
  distinguishes which component fires the close-completion (root's `forceUnmount`) — the
  split responsibility is unobservable from tests and recorded here for the fixture stage.
- `TooltipViewportCssVars` also re-declares the popup width/height variables
  (`packages/react/src/tooltip/viewport/TooltipViewportCssVars.ts:6-12`) — documentation surface
  for the same values set by the shared hook, not separate runtime output.
