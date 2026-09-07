# Tooltip — implementation part: portal, positioner

Batch scope: `packages/react/src/tooltip/portal/` and `packages/react/src/tooltip/positioner/`.
Companion to `specs/library/tooltip/behavior.md` (WHAT) — WHY/HOW only for these files.
Shared code under `packages/react/src/internals/`, `packages/react/src/utils/`, and
`packages/react/src/floating-ui-react/` is cited as external-unit dependencies, not re-derived.

## State machine / hooks used

### Portal

- `TooltipPortal` reads `store.useState('mounted')` and renders nothing unless
  `mounted || keepMounted` (`packages/react/src/tooltip/portal/TooltipPortal.tsx:21-27`) — this is
  the gate behind behavior.md's "by default the closed tooltip is fully unmounted" vs
  `keepMounted` staying in the DOM. `mounted` (not `open`) is the right signal because it stays
  true through the exit transition while `open` is already false.
- When rendering, it publishes `keepMounted` through `TooltipPortalContext` and delegates the
  actual portal DOM to `FloatingPortalLite`
  (`packages/react/src/tooltip/portal/TooltipPortal.tsx:29-33`) — the "Lite" variant skips the
  tabbable/focus-management logic of the full `FloatingPortal`
  (`packages/react/src/utils/FloatingPortalLite.tsx:10-14`), which matches a tooltip: nothing
  inside is tabbable by design.
- `useTooltipPortalContext` throws `"Base UI: <Tooltip.Portal> is missing."` when no portal
  ancestor exists (`packages/react/src/tooltip/portal/TooltipPortalContext.ts:6-12`) — the
  positioner uses it as its composition guard.

### Positioner

- All placement math is delegated to `useAnchorPositioning`
  (`packages/react/src/tooltip/positioner/TooltipPositioner.tsx:57-74`), fed the store's
  `floatingRootContext` (so Floating UI's `referenceElement` is the *active trigger element* kept
  in sync by the root, see `parts/root-store.implementation.md`), `mounted`, and `keepMounted`
  from the portal context. Tooltip-specific defaults applied before the call:
  `side = 'top'`, `align = 'center'`, `sideOffset = 0`, `alignOffset = 0`,
  `collisionBoundary = 'clipping-ancestors'`, `collisionPadding = 5`, `arrowPadding = 5`,
  `sticky = false`, `disableAnchorTracking = false`, and
  `collisionAvoidance = POPUP_COLLISION_AVOIDANCE` (permissive `fallbackAxisSide: 'end'`)
  (`packages/react/src/tooltip/positioner/TooltipPositioner.tsx:30-43`;
  `packages/react/src/internals/constants.ts:22-28`).
- Store-driven inputs: `open`, `mounted`, `trackCursorAxis`, `disableHoverablePopup`,
  `floatingRootContext`, `instantType`, `transitionStatus`, and `adaptiveOrigin`
  (`packages/react/src/tooltip/positioner/TooltipPositioner.tsx:48-55`). `adaptiveOrigin` is
  normally `undefined` and only becomes set while a `Viewport` is mounted (see
  `parts/popup-arrow-viewport.implementation.md`).
- The positioner state object adds a tooltip-only `instant` synthesis:
  `trackCursorAxis !== 'none' ? 'tracking-cursor' : instantType`
  (`packages/react/src/tooltip/positioner/TooltipPositioner.tsx:76-92`) — cursor-following must
  never animate, independent of the delay-group instant phase.
- Rendering goes through the shared `usePositioner`
  (`packages/react/src/tooltip/positioner/TooltipPositioner.tsx:94-101`), which applies the
  positioning styles, `role: 'presentation'`, `hidden: !mounted`, and — the tooltip-specific
  knob — `inert: !open || trackCursorAxis === 'both' || disableHoverablePopup`, which the shared
  helper turns into inline `pointer-events: none`
  (`packages/react/src/utils/usePositioner.tsx:27-43`). This single flag implements both
  behavior.md claims about hoverability: `disableHoverablePopup` and `trackCursorAxis="both"`.
- The rendered ref stack registers the DOM node as `store.positionerElement`
  (`packages/react/src/tooltip/positioner/TooltipPositioner.tsx:98` via
  `store.useStateSetter('positionerElement')`), which is what the synced floating root context
  feeds to Floating UI as the floating element
  (`packages/react/src/floating-ui-react/hooks/useSyncedFloatingRootContext.ts:53-55`).

## Context providers/consumers

- Consumes `TooltipRootContext` (store) and `TooltipPortalContext` (keepMounted) — the latter
  doubling as the "must be inside Portal" guard
  (`packages/react/src/tooltip/positioner/TooltipPositioner.tsx:45-46`).
- Provides `TooltipPositionerContext` with the resolved positioning result — narrowed to
  `side`, `align`, `arrowRef`, `arrowUncentered`, `arrowStyles`
  (`packages/react/src/tooltip/positioner/TooltipPositionerContext.ts:5-12`) — consumed by
  `Popup`, `Arrow`, and `Viewport`. `useTooltipPositionerContext` throws the
  "TooltipPositionerContext is missing" error
  (`packages/react/src/tooltip/positioner/TooltipPositionerContext.ts:14-22`).

## DOM/portal strategy and why

- The portal renders a plain `<div>` into `document.body` (or the given `container`, which may be
  a `ShadowRoot` — `packages/react/src/tooltip/portal/TooltipPortal.tsx:47-48`), so the popup
  escapes clipping/stacking-context ancestors; the positioner div inside it is the Floating UI
  "floating element" that receives the inline positioning styles.
- With no `Viewport`, positioning arrives as Floating UI's default inline `transform` on the
  positioner; with a `Viewport` (which registers the `adaptiveOrigin` middleware) the shared
  positioning hook switches to absolute `top`/`left` (or `right`/`bottom`) coordinates instead
  (`packages/react/src/internals/useAnchorPositioning.ts:503-533`,
  `packages/react/src/utils/adaptiveOriginMiddleware.ts:6-72`) — exactly the two positioning modes
  behavior.md observes ("positioning is applied via inline transform" vs "switches to top/left").
- `keepMounted` semantics also flow into positioning: a keep-mounted positioner pairs elements
  with `autoUpdate` differently than the mount-per-open default
  (`packages/react/src/internals/useAnchorPositioning.ts:484-495,573-578`).
- CSS variables exposed on the positioner: `--available-width`, `--available-height`,
  `--anchor-width`, `--anchor-height`, `--transform-origin` are written by the shared positioning
  middleware (`packages/react/src/internals/useAnchorPositioning.ts:350-449`); the
  `TooltipPositionerCssVars` module additionally documents `--positioner-width`/`--positioner-height`
  (`packages/react/src/tooltip/positioner/TooltipPositionerCssVars.ts:26-37`).
- Data attributes on the positioner (`data-open`/`data-closed`/`data-side`/`data-align`/
  `data-anchor-hidden`) come from the state mapping: `open`/`anchorHidden` via
  `popupStateMapping` and `side`/`align` via the default state→`data-*` kebab-case fallback
  (`packages/react/src/utils/popupStateMapping.ts:48-61`,
  `packages/react/src/internals/getStateAttributesProps.ts:25-27`).
- `TooltipPositioner.spec.tsx` pins the API: `keepMounted` must be a type error on the positioner
  (`packages/react/src/tooltip/positioner/TooltipPositioner.spec.tsx:3-4`) — keepMounted belongs
  to the Portal, and the positioner obtains it from context instead.

## Dependencies on other Base UI internals

- `packages/react/src/internals/` — `useAnchorPositioning` (+ `Side`/`Align` types),
  `POPUP_COLLISION_AVOIDANCE` (`constants.ts`).
- `packages/react/src/utils/` — `usePositioner`, `FloatingPortalLite`
  (which wraps `packages/react/src/floating-ui-react/components/FloatingPortal`'s
  `useFloatingPortalNode`).
- `packages/utils/` — none directly; both parts use plain `React.forwardRef`.
- No `wraps-external:` field exists on this unit's `TODO.md` entry (`TODO.md:565-575`).

## Anything in source not explained by any test

- The synthesized `'tracking-cursor'` instant value
  (`packages/react/src/tooltip/positioner/TooltipPositioner.tsx:82`): no tooltip test asserts
  `data-instant="tracking-cursor"` on the positioner; behavior.md's `data-instant` coverage only
  includes `focus` and `delay`.
- Default `side = 'top'` / `align = 'center'`
  (`packages/react/src/tooltip/positioner/TooltipPositioner.tsx:31-32`): behavior.md flags the
  default as UNVERIFIED; the source value is stated here for the record.
- `role="presentation"` on the positioner element
  (`packages/react/src/utils/usePositioner.tsx:38`): no tooltip test asserts the role.
- `--positioner-width`/`--positioner-height` are exported as public CSS variables but never set by
  any tooltip source path (they are consumer-facing hooks, not component output) — flagged so the
  fixture stage does not expect them in DOM output.
- The `hide` middleware's `anchorHidden` result is surfaced in state/attributes but no tooltip
  test asserts `data-anchor-hidden`.
