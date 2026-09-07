# Navigation Menu implementation spec

WHY/HOW companion to `behavior.md` (the WHAT). Explains the state machine, hook composition,
context wiring, and DOM/portal decisions that produce the documented behavior, mined from the
non-test sources listed in the unit's file inventory. The unit's `TODO.md` entry
(`TODO.md:433-439`) has **no `wraps-external:` field**, so there is no external-package
delegation to state here: everything below is implemented inside Base UI itself.

## State machine / hooks used

### Root — a single-value state machine

The root is not a boolean open/close machine. Its whole state is one value: the currently open
item, or nothing.

- `useControlled` wraps `value`/`defaultValue` into a single `value` state with change
  notification (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:81-86`). This is
  what makes controlled/`defaultValue`/uncontrolled modes (behavior.md → State model) one code
  path. `open` is derived, not stored: `open = value != null`
  (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:89`). Falsy-but-defined item
  values (`0`, `''`, `false`) therefore count as open for free — no extra handling exists for
  them (behavior.md → State model).
- `setValue` (stable via `useStableCallback`) is the only mutation path
  (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:152-185`). It: records the
  close reason when the next value is nullish; fires `onValueChange` only when the value actually
  changes; honors `eventDetails.isCanceled()` (behavior.md → Events); clears
  `activationDirection`/`floatingRootContext` on close; and propagates nested-menu link-press
  closes up to the parent root's `setValue(null, …)` (behavior.md → Events, nested close
  propagation).
- Element/state slices are plain `useState` refs of DOM nodes so children can register
  themselves: `positionerElement`, `popupElement`, `viewportElement`, `viewportTargetElement`,
  `activationDirection`, `floatingRootContext`, `viewportInert`
  (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:94-105`). The root never
  renders the popup itself; it only holds coordinates to it.
- Mount/unmount lifecycle: `useTransitionStatus(open)` yields `mounted` + `transitionStatus`
  (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:120`), which drive
  `data-starting-style`/`data-ending-style` consumers (behavior.md → State model).
  `useOpenChangeComplete` runs twice — once keyed on `popupElement`, once on
  `viewportTargetElement` — and calls `handleUnmount` on close completion
  (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:217-237`). Two hooks are
  needed because inline (portal-less) nested menus have no popup; their viewport target is the
  last thing to unmount.
- Manual unmount opt-out (behavior.md → State model, `actionsRef.unmount()`): providing
  `actionsRef` disables both `useOpenChangeComplete` hooks (`enabled: !actionsRef`) and
  `React.useImperativeHandle` exposes `unmount` → `handleUnmount`
  (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:214-215`).
- `handleUnmount` implements the return-focus policy: reasons in `blockedReturnFocusReasons`
  (`triggerHover`, `outsidePress`, `focusOut`) skip refocusing the previous trigger; otherwise
  focus returns to `prevTriggerElementRef` only if focus is on `body` or inside the popup
  (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:31-35`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:187-212`). This one branch
  produces three documented behaviors: Escape returns focus to the trigger, outside press and
  hover-close leave focus where it landed (behavior.md → Focus management).
- Controlled-close size preservation (behavior.md → State model, externally-triggered close)
  comes from a close-side layout effect that snapshots the positioner's last fixed
  `--positioner-width/-height` and re-applies it to both elements via `setSharedFixedSize`
  before the exit transition runs
  (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:122-146`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:37-55`,
  `packages/react/src/navigation-menu/utils/setSharedFixedSize.ts:4-20`). Reading the vars
  instead of measuring the popup is deliberate: during a controlled close the popup is already
  in its exit render and can report 0.
- A `value`-scoped effect resets `viewportInert` after every item change
  (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:148-150`).

### Root — the floating tree

`TreeContext` splits the component so the root context exists before the element renders. It
registers a `FloatingNode` under a `useFloatingNodeId()` and publishes the id through
`NavigationMenuTreeContext`; a top-level (non-nested) root additionally wraps everything in
`FloatingTree` (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:313-352`,
`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:303-306`). Nested-ness is
detected with `useFloatingParentNodeId() != null`
(`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:78`). This tree membership is
the substrate for: `isOutsideMenuEvent` child-node checks, safePolygon scoping, and nested
`setValue` propagation.

### Trigger — where open/close actually happens

The trigger owns nearly all interaction machinery; the root only stores results.

- Timing primitives: `useTimeout` (patient-click threshold) and three `useAnimationFrame`
  handles (`mutationFrame`, `resizeFrame`, `sizeFrame`) used to schedule size transitions
  across frames (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:112-115`).
- `useFloatingRootContext` publishes this trigger as `reference` and the popup/viewport as
  `floating` (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:475-482`);
  the active trigger's context is hoisted into root state via
  `setFloatingRootContext` so the Positioner/Viewport/List can consume it
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:546-551`). This is
  the "positioner follows the active trigger" mechanism.
- Hover: `useHoverReferenceInteraction` with `safePolygon` handleClose, `move: false`,
  `restMs: mounted && positionerElement ? 0 : delay` (delay elapses once pre-mount, then 0),
  `delay: { close: closeDelay }`
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:518-529`). Click:
  `useClick` with `stickIfOpen` and `toggle: isActiveItem`
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:536-540`).
- `stickIfOpen` is the patient-click gate: after a hover open, a timeout started with
  `PATIENT_CLICK_THRESHOLD` flips it off; clicks before that keep the menu open
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:449-458`,
  behavior.md → State model).
- `handleOpenChange` guards: touch never hover-opens; hover-close from a non-active trigger is
  ignored (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:431-447`).
  Hover changes are flushed synchronously (`ReactDOM.flushSync`) so DOM state is consistent
  within the pointer event (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:468-472`).
- `handleActivation` computes `activationDirection` geometrically (comparing the previous
  trigger's rect against the next one, per orientation) inside `flushSync`
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:553-568`) —
  behavior.md → State model, `data-activation-direction`. It also resets the floating
  `openEvent` on non-click activations so hover→click→hover handoff doesn't wedge the popup
  open, applies the safePolygon pointer-events lock (deferred via `queueMicrotask` when
  switching items while open), and sets the value with `triggerHover`/`triggerPress` reasons
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:570-614`).
- Keyboard open: `onKeyDown` maps `ArrowDown` (horizontal) or the direction-mirrored
  `ArrowLeft`/`ArrowRight` (vertical) to `setValue(itemValue, … listNavigation …)` and then
  runs the same `handleOpenEvent` path; nested triggers opt out so arrows reach the parent
  composite root (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:675-692`,
  behavior.md → Keyboard interactions).
- Blur close: `onBlur` delegates to `isOutsideMenuEvent` and closes with the `focusOut` reason
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:693-707`,
  `packages/react/src/navigation-menu/utils/isOutsideMenuEvent.ts:16-35`). The utility closes
  only when the related target is outside the popup, the root, and every floating child node in
  the tree — which is why `relatedTarget: null` link blurs don't close (behavior.md → Focus
  management).
- `useButton` (with `focusableWhenDisabled: true`) supplies native-button semantics
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:710-714`); disabled is
  re-checked in `handleOpenEvent` (`…:618-621`), so `onValueChange` is never called for disabled
  triggers (behavior.md → State model).

### Trigger — popup sizing orchestration

The popup/positioner CSS-variable size machine (behavior.md → DOM structure, sizing vars) lives
in the trigger because the trigger knows when "its" item became active:

- `handleValueChange` is the main sizing step: cancel pending auto-size reset, clear fixed
  sizes, measure (`getCssDimensions`), fix both popup and positioner vars, then one
  `sizeFrame.request` later set the measured popup size and schedule the auto-size reset via
  `useAnimationsFinished(popupElement)`
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:184-221`,
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:131`,
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:170-182`).
- `handleInterruptedMutationResize` handles the mid-transition mutation case: it re-fixes the
  current size, waits two animation frames, clears, re-measures, and re-fixes
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:223-256`) — behavior.md
  → Edge cases (interruptible mutation resizing, zero-size preservation).
- Inputs that trigger sizing: activation layout effect
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:410-429`), a
  `MutationObserver` on `currentContentRef` (childList/subtree/characterData plus `hidden`
  attribute for keepMounted switches; skips the `starting` transition phase)
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:353-408`), a
  window-`resize` listener routed through `resizeFrame`
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:332-351`), and a
  `ResizeObserver` that keeps `prevSizeRef` fresh as the last-known-good size
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:313-330`).
- Stale-size protection (behavior.md → Edge cases, rapid hover switching):
  `popupAutoSizeResetRef` is shared root state so a newly active trigger can abort the previous
  trigger's scheduled reset; ownership is checked by item value
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:138-146`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:113-118`).
  `skipAutoSizeSyncRef` suppresses the initial open-size reset once a switch already started
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:629-636`).

### List / Item / Link — composite structure

- `NavigationMenuItem` mints the item identity: explicit `value` or a generated
  `useBaseUiId()` fallback, published via `NavigationMenuItemContext`
  (`packages/react/src/navigation-menu/item/NavigationMenuItem.tsx:23-37`).
- `NavigationMenuList` is the composite root: `CompositeRoot` (roving focus, orientation, no
  loop) wraps the `ul`; arrow-key events are stopPropagation'd so they never leak to outer
  handlers (behavior.md → Keyboard interactions); when nested it skips `CompositeRoot` and the
  key guard entirely so triggers participate in the parent content's composite navigation
  (`packages/react/src/navigation-menu/list/NavigationMenuList.tsx:75-100`,
  `packages/react/src/navigation-menu/list/NavigationMenuList.tsx:102-124`).
- The list also hosts the floating-side interactions for the whole menu:
  `useHoverFloatingInteraction` (close delay on the floating side) and `useDismiss` with
  `outsidePressEvent: 'intentional'`; the outside-press filter returns `false` when the target
  is any navigation trigger (matched by `NAVIGATION_MENU_TRIGGER_IDENTIFIER`), which is what
  makes clicking a different trigger switch instead of dismiss
  (`packages/react/src/navigation-menu/list/NavigationMenuList.tsx:48-64`,
  `packages/react/src/navigation-menu/utils/constants.ts:3`). Both are gated on
  `floatingRootContext` being present and disabled until a popup/positioner exists
  (`packages/react/src/navigation-menu/list/NavigationMenuList.tsx:42-46`,
  `packages/react/src/navigation-menu/list/NavigationMenuList.tsx:66`).
- `NavigationMenuLink` is a `CompositeItem` anchor; `closeOnClick` closes with the `linkPress`
  reason and `onBlur` reuses `isOutsideMenuEvent` with the `focusOut` reason
  (`packages/react/src/navigation-menu/link/NavigationMenuLink.tsx:41-64`); `active` maps to
  `aria-current="page"` (`…:42`).

### Content — per-item transition machine

Each `NavigationMenuContent` runs its own mini open/close machine on top of the root's:

- `open = popupMounted && value === itemValue`
  (`packages/react/src/navigation-menu/content/NavigationMenuContent.tsx:61`) — the content is
  "open" only when the popup exists AND its item is active; per-item `data-open`/transition
  states derive from this, not from root `open`.
- `useTransitionStatus(open)` + `useOpenChangeComplete` manage enter/exit and unmount; a
  synchronous reset handles the popup unmounting before the content's exit finishes so the next
  open re-enters via `starting`
  (`packages/react/src/navigation-menu/content/NavigationMenuContent.tsx:68-84`).
- Re-entry while still mounted (switching back to a panel mid-exit) can't rely on the callback
  ref firing again, so a layout effect re-assigns `currentContentRef` — this keeps the
  trigger's MutationObserver pointed at the right content node
  (`packages/react/src/navigation-menu/content/NavigationMenuContent.tsx:86-94`).

### Positioner — anchor following and instant states

- `instant` starts `true` when initially open (`defaultValue`), released one tick later; window
  `resize` re-enables it for 100ms via `useTimeout`
  (`packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.tsx:80-81`,
  `packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.tsx:141-168`) —
  behavior.md → DOM structure (`data-instant`).
- Element rendering is delegated to the shared `usePositioner` helper: `role="presentation"`,
  `hidden: !mounted`, `inert` styling (pointer-events: none) while closed, transition-style
  suppression, and the positioner CSS vars live in `positioning.positionerStyles`
  (`packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.tsx:170-177`).
- Collision avoidance defaults differ for nested (inline) menus vs top-level menus
  (`packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.tsx:62-64`); the
  `shift` middleware is configured with `rootBoundary: 'layoutViewport'`
  (`packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.tsx:126`) — the
  exact configuration spied in behavior.md → DOM structure.

## Context providers/consumers

Five React contexts plus the floating tree cross the boundary:

| Context | Provider | Consumers | Carries |
|---|---|---|---|
| `NavigationMenuRootContext` | Root (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:296`) | every part via `useNavigationMenuRootContext` (`packages/react/src/navigation-menu/root/NavigationMenuRootContext.ts:59-75`) | open/value/setValue, mounted/transitionStatus, the four element refs+setters, activationDirection, floatingRootContext, focus-guard refs, `prevTriggerElementRef`, shared `popupAutoSizeResetRef`, delay/closeDelay, orientation, nested, viewportInert (`packages/react/src/navigation-menu/root/NavigationMenuRootContext.ts:12-49`) |
| `NavigationMenuTreeContext` | Root's `TreeContext` (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:348`) | Trigger, Content, List, Link, Positioner | the floating `nodeId` string |
| `NavigationMenuItemContext` | Item (`packages/react/src/navigation-menu/item/NavigationMenuItem.tsx:34`) | Trigger, Content, Icon via `useNavigationMenuItemContext` (`packages/react/src/navigation-menu/item/NavigationMenuItemContext.ts:12-20`, throws outside Item) | `{ value }` item identity |
| `NavigationMenuPositionerContext` | Positioner (`packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.tsx:180`) | Popup, Arrow (required), Viewport (optional) | the full anchor-positioning result: side/align/anchorHidden/arrowRef/arrowStyles/arrowUncentered/positionerStyles (`packages/react/src/navigation-menu/positioner/NavigationMenuPositionerContext.ts:5`) |
| `NavigationMenuPortalContext` | Portal (`packages/react/src/navigation-menu/portal/NavigationMenuPortal.tsx:29`) | Positioner (throws if missing, `packages/react/src/navigation-menu/portal/NavigationMenuPortalContext.ts:6-12`) | `keepMounted` boolean |
| `NavigationMenuDismissContext` | List (`packages/react/src/navigation-menu/list/NavigationMenuList.tsx:104-123`) | Trigger | the `useDismiss` element props so trigger presses participate in dismissal |

Notable data flows across these boundaries:

- The active trigger's `FloatingRootContext` travels Trigger → root state → Positioner/Viewport
  (`domReferenceElement` is read from it at
  `packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.tsx:108` and
  `packages/react/src/navigation-menu/viewport/NavigationMenuViewport.tsx:91`), which is how a
  single positioner re-anchors to whichever trigger is active.
- Focus-guard span refs (`beforeInside/afterInside/beforeOutside/afterOutside`) are allocated in
  the root and attached by both Trigger (outside guards) and Viewport (inside guards) — the two
  halves of the tab-cycle documented in behavior.md → Focus management.
- `useDirection` (a global Base UI context, not navigation-specific) feeds RTL handling in the
  trigger keyboard map and popup corner pinning
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:110`,
  `packages/react/src/navigation-menu/popup/NavigationMenuPopup.tsx:28`).

## DOM/portal strategy and why

- Root renders `nav` at top level, `div` when nested — a nested "menu" is content inside another
  popup, so a second `nav` landmark would be wrong
  (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:341`).
- `Portal` renders `FloatingPortal` (body by default, `container` overridable) only while
  `mounted || keepMounted` and publishes `keepMounted`
  (`packages/react/src/navigation-menu/portal/NavigationMenuPortal.tsx:19-32`). Portal placement
  escapes the nav's stacking context and lets the popup survive list re-renders.
- `Positioner` renders a plain presentation `div` inside the portal; it is `hidden` while
  unmounted and inert while closed
  (`packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.tsx:170-177`). It
  must exist as a separate layer from the popup because the positioner's box is the stable
  thing anchored to the trigger while the popup inside it resizes/animates
  (`--positioner-width/-height` vs `--popup-width/-height`,
  `packages/react/src/navigation-menu/positioner/NavigationMenuPositionerCssVars.ts:30-35`,
  `packages/react/src/navigation-menu/popup/NavigationMenuPopupCssVars.ts:5-10`).
- `Popup` renders `nav` with `tabIndex: -1` and an id; when positioned on an origin side
  (`top`, physical left) it pins itself to the far corner of the positioner with absolute
  offsets so size transitions grow away from the anchor instead of shifting the anchor
  (`packages/react/src/navigation-menu/popup/NavigationMenuPopup.tsx:40-62`) — this is the
  mechanism behind behavior.md → DOM structure, "grows leftward"/pinned-edge assertions.
- `Viewport` is the clipping box that hosts the active panel; it has two structural variants:
  with a Positioner, children render directly and the guards wrap the viewport element; without
  one (inline nested), children are wrapped in an inner div registered as
  `viewportTargetElement` and the guards wrap that
  (`packages/react/src/navigation-menu/viewport/NavigationMenuViewport.tsx:119-131`). The inner
  target exists so Content can portal into the visually correct box while the guards remain
  positioned around it.
- `Content` does not render in place: it portals into `viewportTargetElement || viewportElement`
  (`packages/react/src/navigation-menu/content/NavigationMenuContent.tsx:134`,
  `…:160-173`) and registers as a `FloatingNode` under the menu's tree node. Moving the panel
  out of the list subtree is what lets one shared popup cross-fade/resize between triggers while
  the triggers never re-mount (behavior.md → DOM structure). Three render modes:
  1. normal — portal into the viewport container once it exists;
  2. `keepMounted` with no portal container yet (SSR/pre-hydration) — render inline, `hidden`
     (`packages/react/src/navigation-menu/content/NavigationMenuContent.tsx:136-153`), which
     produces the server-HTML and hydration behavior in behavior.md → DOM structure;
  3. `keepMounted` with a portal container — always portal, `hidden` attribute when its item is
     inactive (`…:135`, `…:168`).
- Focus guards: the active trigger renders a before/after pair of outside guards plus a
  visually-hidden `aria-owns` span bridging trigger→viewport for AT
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:736-781`); the viewport
  renders before/after inside guards around the panel
  (`packages/react/src/navigation-menu/viewport/NavigationMenuViewport.tsx:39-63`). Guards
  redirect focus inward (into the panel) or outward (to the tabbable neighbor of the trigger)
  depending on which side the focus came from — implementing the wrap-around tab order in
  behavior.md → Focus management without a focus *trap*.
- The positioner installs capture-phase `focusin`/`focusout` listeners that enable/disable focus
  inside the portal when focus arrives from outside
  (`packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.tsx:84-106`) — the
  tabbable-portal pattern that lets tabbing *through* the panel work while it stays inert to
  stray focus.

## Dependencies on other Base UI internals

This unit is a heavy consumer of shared internals. (No `wraps-external:` field exists on the
unit's TODO.md entry, so no external package is delegated to; `@floating-ui/utils/dom` is the
only direct third-party import, used for `isHTMLElement` in the root's unmount logic,
`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:3`.)

- **`floating-ui-react/`** (the vendored Floating UI layer — the largest dependency):
  - Tree/registry: `FloatingTree`, `FloatingNode`, `useFloatingNodeId`,
    `useFloatingParentNodeId` (Root), `useFloatingTree` (Trigger, Link)
    (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:8-14`,
    `packages/react/src/navigation-menu/link/NavigationMenuLink.tsx:3`).
  - Interaction hooks: `useFloatingRootContext`, `useClick`, `safePolygon`,
    `useHoverReferenceInteraction` (Trigger),
    `useHoverFloatingInteraction`, `useDismiss` (List)
    (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:12-18`,
    `packages/react/src/navigation-menu/list/NavigationMenuList.tsx:4`).
  - `useHoverInteractionSharedState` + the pointer-events mutation helpers that implement the
    safePolygon lock (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:19-23`).
  - `FloatingPortal` (Portal,
    `packages/react/src/navigation-menu/portal/NavigationMenuPortal.tsx:3`).
  - Utils: `contains`, `activeElement`, `getTarget`, `isOutsideEvent`, `getNextTabbable`,
    `getPreviousTabbable`, `getTabbableAfterElement`, `stopEvent`, `disableFocusInside`,
    `enableFocusInside`, `getNodeChildren`, `getEmptyRootContext` (shadow-DOM-safe traversal per
    repo guidelines; used in Trigger, List, Content, Link, Viewport, Positioner).
- **`internals/`**:
  - `useRenderElement` (all rendered parts),
    `useTransitionStatus` (Root, Content), `useOpenChangeComplete` (Root, Content),
    `useBaseUiId` (Item, Popup).
  - `useAnchorPositioning` — the positioning engine; the navigation-specific wrapper
    `useNavigationMenuAnchorPositioning` calls `useAnchorPositioningWithHook` with the
    navigation menu's floating hook variant, because the active trigger supplies the root store
    only *after* the positioner has rendered
    (`packages/react/src/navigation-menu/utils/useNavigationMenuAnchorPositioning.ts:13-17`).
  - `CompositeRoot`/`CompositeItem` (roving-focus composite system: List, Content, Trigger, Link)
    (`packages/react/src/navigation-menu/list/NavigationMenuList.tsx:7`,
    `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:46`).
  - `useButton` (Trigger), `useAnimationsFinished` (Trigger auto-size reset),
    `getDisabledMountTransitionStyles` (Popup, Arrow first-frame transition suppression),
    `stateAttributesMapping` (`transitionStatusMapping` in Content),
    `createBaseUIEventDetails`/`REASONS` (all `setValue` call sites),
    `constants` (`PATIENT_CLICK_THRESHOLD`, `ownerVisuallyHidden`,
    `POPUP_COLLISION_AVOIDANCE`, `DROPDOWN_COLLISION_AVOIDANCE`),
    `direction-context` (`useDirection`), `types` (`BaseUIComponentProps`, `HTMLProps`,
    `NativeButtonProps`).
- **`utils/` (component-shared)**: `FocusGuard` (Trigger, Viewport),
  `popupStateMapping` family — `pressableTriggerOpenStateMapping` (Trigger),
  `popupTransitionStateMapping` (Popup, Backdrop), `triggerOpenStateMapping` (Icon),
  `popupStateMapping` (Arrow, Positioner via `usePositioner`) — these produce the
  `data-open`/`data-starting-style`/`data-side` attribute families documented per part in the
  `*DataAttributes.ts` modules; `usePositioner` (shared positioner element renderer),
  `adaptiveOriginMiddleware` (keeps the anchor stable while size and position transition
  together), `getCssDimensions` (Trigger measuring).
- **`@base-ui/utils/*`**: `useControlled`, `useStableCallback`, `useIsoLayoutEffect`,
  `useTimeout`, `useAnimationFrame`, `useValueAsRef` (Trigger `isActiveItemRef`),
  `useId` (Viewport), `inertValue` (Content, Viewport), `addEventListener`, `mergeCleanups`
  (Positioner listeners), `ownerDocument`/`ownerWindow`, `EMPTY_ARRAY`/`EMPTY_OBJECT`,
  `mergeProps` (Trigger reference props).

Cross-unit takeaway for dependency ordering: a Rust/Leptos port of navigation-menu needs, at
minimum, the floating tree/registry, hover/click/dismiss interaction primitives with
safePolygon, an anchor-positioning engine with shift + adaptive origin, the composite roving
focus system, focus guards, transition-status state attributes, and the shared positioner
renderer before any navigation-menu behavior can be reproduced.

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit; none of these
are asserted in the mined test suite (behavior.md documents what *is* asserted).

1. **`aria-controls` and the `aria-owns` span.** The trigger sets
   `aria-controls` to the popup id while active
   (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:662`) and renders a
   visually-hidden `aria-owns` span pointing at the viewport
   (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:749`). behavior.md →
   Accessibility already marks the id-linking surface UNVERIFIED; the `aria-owns` bridge is an
   additional untested detail. Fixtures will need to decide whether these are contractual.
2. **Backdrop's full surface is untested.** Its tests only pin the element type. The
   `role="presentation"`, `hidden: !mounted`, and `user-select`/`-webkit-user-select` defaults
   (`packages/react/src/navigation-menu/backdrop/NavigationMenuBackdrop.tsx:32-39`) and its
   transition data attributes have no assertions.
3. **Icon's default content.** The icon renders the literal `▼` child and `aria-hidden`
   (`packages/react/src/navigation-menu/icon/NavigationMenuIcon.tsx:32`); tests only assert
   `data-popup-open`.
4. **`viewportInert` focus-loop prevention.** The viewport goes `inert` (only in the
   no-positioner variant) when a blur leaves it toward something that is neither inside it nor
   the trigger (`packages/react/src/navigation-menu/viewport/NavigationMenuViewport.tsx:104-118`);
   it is reset on value change (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:148-150`),
   on trigger focus
   (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:664-669`), and in the
   after-outside guard
   (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:752-757`). No test
   exercises the inert state itself.
5. **Positioner tabbable-portal focus management.** The capture-phase
   `enableFocusInside`/`disableFocusInside` listener pair
   (`packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.tsx:84-106`) is
   inferred by the focus-order tests but never directly asserted (no test checks
   `inert`-toggling inside the portal).
6. **Portal `container` prop.** Forwarded to `FloatingPortal`
   (`packages/react/src/navigation-menu/portal/NavigationMenuPortal.tsx:47-51`) but untested.
7. **Positioner default tuning.** `collisionPadding = 5`, `arrowPadding = 5`, `sticky = false`,
   `disableAnchorTracking = false`, and the nested-vs-top-level `collisionAvoidance` split
   (`packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.tsx:59-67`) are
   untested (only `side` and the `shift.rootBoundary` config are pinned).
8. **Inactive keepMounted content is `inert` until focused.** Besides `hidden`, closed
   keepMounted content gets absolute top-left positioning and `inert` until it receives focus
   (`packages/react/src/navigation-menu/content/NavigationMenuContent.tsx:110-132`); tests cover
   `hidden` and SSR presence but not the inert-until-focus behavior.
9. **`REASONS.none` in the change-reason union**
   (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:419-427`) is never produced
   by any call site in this unit; it exists in the public type only.
10. **Nested root element type.** A nested Root renders `div`, not `nav`
    (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:341`). Nested-menu tests
    exercise behavior, and conformance checks `HTMLElement`, so the nav-vs-div choice for
    nested roots is not pinned anywhere.
11. **Inline-menu unmount path.** The root's second `useOpenChangeComplete` keyed on
    `viewportTargetElement` (`packages/react/src/navigation-menu/root/NavigationMenuRoot.tsx:228-237`)
    serves portal-less inline menus, but the unmount tests only cover the popup-keyed path.
12. **Type-level contracts** (not runtime behavior): `NavigationMenuRoot.spec.tsx` pins the
    `Root` generics (`Value` defaults to `any`, `onValueChange` gets `Value | null`) and
    `NavigationMenuLink.spec.tsx` pins native `<a>` props in the Link render callback. These are
    compile-time tests absent from behavior.md's runtime inventory.
