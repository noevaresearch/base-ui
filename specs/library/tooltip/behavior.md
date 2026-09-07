# Tooltip — behavior spec (Stage 1: behavior mining)

Unit: `packages/react/src/tooltip/`. Mined from the unit's own test files only. The `TODO.md`
entry for `library: tooltip` has no `wraps-external:` field, so no third-party delegation applies.

Structural note that applies to most claims below: the main suite in
`packages/react/src/tooltip/root/TooltipRoot.test.tsx:42-46` runs its entire body three times —
for "contained triggers" (trigger nested inside `Tooltip.Root`), "detached triggers" (trigger
wired to the root through `Tooltip.createHandle()`), and "multiple detached triggers" (two
triggers sharing one handle). Every behavior asserted inside that block therefore holds for all
three wirings unless stated otherwise.

## Public API surface (props, parts, subcomponents)

Namespace `Tooltip` exposes the parts `Root`, `Trigger`, `Portal`, `Positioner`, `Popup`,
`Arrow`, `Viewport` (imported as `Tooltip` from `@base-ui/react/tooltip` and used as
`Tooltip.Root` etc. throughout, e.g. `packages/react/src/tooltip/arrow/TooltipArrow.test.tsx:13-19`)
plus a static `Tooltip.createHandle()` factory
(`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1191`).

- `Tooltip.Root` props exercised by tests: `open`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:214-220`), `defaultOpen`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:207`), `onOpenChange(open, details)`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:135-138`), `onOpenChangeComplete(open)`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:427`), `disabled`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:788`), `trackCursorAxis` — values `'x'`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:934`) and `'both'`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:918`) are exercised, `disableHoverablePopup`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:884`), `actionsRef` exposing `unmount()`
  and `close()` (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:346-351`), `handle`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1223`), `defaultTriggerId`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1225`), `triggerId`
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:125`), and a render-prop
  children form receiving `{ payload }`
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:628-645`). `TrackCursorAxis`
  value `'y'` is never exercised — UNVERIFIED — inferred from `packages/react/src/tooltip/root/TooltipRoot.test.tsx:915-946`, no test asserts it.
- `Tooltip.Trigger` props exercised: `delay`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:251`), `closeDelay`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:274`), `closeOnClick`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1098`), `disabled`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:806`), `handle`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2969`), `payload`
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:630`), `id`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1200`), `render`
  (`packages/react/src/tooltip/trigger/TooltipTrigger.test.tsx:86`), children
  (`packages/react/src/tooltip/trigger/TooltipTrigger.test.tsx:26`).
- `Tooltip.Portal` prop: `keepMounted`
  (`packages/react/src/tooltip/portal/TooltipPortal.test.tsx:22`).
- `Tooltip.Positioner` props: `side` (`packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:113`),
  `align` (`packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:138`), `sideOffset`
  (number `packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:72`; function
  `packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:90-93`), `alignOffset`
  (number `packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:187`; function
  `packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:205-207`), `arrowPadding`
  (`packages/react/src/tooltip/arrow/TooltipArrow.test.tsx:36`).
- `Tooltip.Popup` renders children
  (`packages/react/src/tooltip/popup/TooltipPopup.test.tsx:22-34`).
- `Tooltip.Arrow` is styled with `width`/`height` inline styles in tests
  (`packages/react/src/tooltip/arrow/TooltipArrow.test.tsx:38`); no other props exercised.
- `Tooltip.Viewport` renders children
  (`packages/react/src/tooltip/viewport/TooltipViewport.test.tsx:26-45`).
- Handle object: `Tooltip.createHandle<T>()` returns a handle with `open(triggerId)` and
  `close()` methods (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:1586-1594`)
  and an `isOpen` boolean (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:254`).
  `handle.open(id)` accepts a payload-carrying trigger id
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:1638-1641`).

Conformance: `Trigger` renders to an `HTMLButtonElement` by default
(`packages/react/src/tooltip/trigger/TooltipTrigger.test.tsx:15-20`); `Popup`, `Positioner`,
`Portal`, and `Viewport` render to `HTMLDivElement` (`packages/react/src/tooltip/popup/TooltipPopup.test.tsx:10`,
`packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:18`,
`packages/react/src/tooltip/portal/TooltipPortal.test.tsx:11`,
`packages/react/src/tooltip/viewport/TooltipViewport.test.tsx:11`); `Arrow` conforms as a generic
`window.Element` (`packages/react/src/tooltip/arrow/TooltipArrow.test.tsx:10`).

Context-missing errors (all thrown at render time):

- Trigger outside any root/handle: "Base UI: <Tooltip.Trigger> must be either used within a
  <Tooltip.Root> component or provided with a handle."
  (`packages/react/src/tooltip/trigger/TooltipTrigger.test.tsx:22-32`)
- Positioner outside root: "Base UI: TooltipRootContext is missing. Tooltip parts must be placed
  within <Tooltip.Root>." (`packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:28-38`)
- Positioner inside root but outside portal: "Base UI: <Tooltip.Portal> is missing."
  (`packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:40-54`)
- Popup outside positioner: "Base UI: TooltipPositionerContext is missing. TooltipPositioner parts
  must be placed within <Tooltip.Positioner>." (`packages/react/src/tooltip/popup/TooltipPopup.test.tsx:36-54`)

## State model (controlled/uncontrolled, defaults, transitions)

- Uncontrolled open: hover (pointerdown + mouseenter + mousemove) starts the open delay; the
  popup appears after the delay elapses
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:50-64`); mouseleave closes
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:66-83`).
- Controlled open (`open` prop): `onOpenChange` fires exactly on transitions — once when opening
  and once when closing (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:125-165`), and is
  not called when an interaction would not change the state
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:167-202`).
- `defaultOpen: true` opens on mount and stays uncontrolled afterwards (a later mouseleave closes
  it) (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:230-244`). When `open` is also
  provided, `defaultOpen` is ignored: `open={false}` keeps it closed
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:214-220`).
- Close transitions and their `onOpenChange` `details.reason` values: Escape → the escape-key
  reason constant (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1051-1054`); imperative
  `actionsRef.close()` → the imperative-action reason constant
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:411-414`); active-trigger unmount → the
  literal reason `'none'`
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:815-818`).
- Open transitions: hover after `delay` (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:250-267`),
  focus (see Focus management), imperative `handle.open(triggerId)`
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:1565-1600`).
- Close delay: with a trigger `closeDelay`, the popup remains visible immediately after
  mouseleave and unmounts only once the delay elapses
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:270-294`).
- Delay resolution order: the trigger's `delay` overrides the `Provider`'s `delay`
  (`packages/react/src/tooltip/provider/TooltipProvider.test.tsx:79-109`); the provider `delay`
  gates the open otherwise (`packages/react/src/tooltip/provider/TooltipProvider.test.tsx:18-51`),
  and `delay={0}` opens immediately
  (`packages/react/src/tooltip/provider/TooltipProvider.test.tsx:53-77`). The default delay when
  no prop is given is the shared `OPEN_DELAY` constant imported by the tests
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:7`); its numeric value (600) is
  UNVERIFIED — inferred from `packages/react/src/tooltip/utils/constants.ts:1`, no test asserts the number.
- Provider "instant" group behavior: `Provider` `timeout` defines a window during which moving
  from one tooltip to an adjacent one opens the next immediately (delay bypassed)
  (`packages/react/src/tooltip/provider/TooltipProvider.test.tsx:226-247`); a per-trigger `delay`
  is still honored outside that instant phase
  (`packages/react/src/tooltip/provider/TooltipProvider.test.tsx:249-287`), and once the timeout
  elapses the full delay is required again
  (`packages/react/src/tooltip/provider/TooltipProvider.test.tsx:289-315`). Provider `closeDelay`
  waits before hiding and always uses the latest prop value
  (`packages/react/src/tooltip/provider/TooltipProvider.test.tsx:112-195`).
- Cancellation: `onOpenChange` details `cancel()` prevents the open while uncontrolled
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:977-1000`) and a canceled unmount-close
  keeps the tooltip open when the active trigger unmounts
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:763-822`).
- Deferred unmount: `details.preventUnmountOnClose()` keeps the popup mounted through one close;
  the following close unmounts normally
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:297-342`). `actionsRef.current.unmount()`
  forces the unmount immediately
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:344-382`).
- `onOpenChangeComplete(open)` fires after open/close finishes — immediately when the
  corresponding CSS transition/animation is absent, or when the enter/exit animation finishes; it
  is not called on a mount that starts closed
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:418-449`,
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:451-504`,
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:506-536`,
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:538-589`,
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:591-605`).
- Disabled: root `disabled` blocks both hover-open and focus-open
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:786-803`), closes an already-open tooltip
  when it becomes disabled (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:816-843`), does
  not throw when combined with `defaultOpen`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:845-849`), and wins over a trigger that
  opts back in with `disabled={false}`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:859-878`). Trigger-level `disabled` also
  blocks focus-open (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:805-814`).
- Active-trigger state: the popup content reflects the active trigger's `payload`
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:625-660`); `triggerId`
  plus `onOpenChange` `details.trigger?.id` allows fully programmatic control of which trigger is
  active (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:860-928`);
  `defaultTriggerId` selects the initially active trigger for a `defaultOpen` root
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:1459-1494`).
- Handle state: `handle.isOpen` tracks open state
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:80-84`); calls made with
  no mounted root are ignored with a "no root using this handle is mounted" warning
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:242-287`); after a root
  unmounts, the handle reads closed and later calls are ignored until a root re-attaches, at which
  point state resets (closed, no payload)
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:289-361`).
- SSR/hydration: a `defaultOpen` detached root stays open (per `handle.isOpen`) from the server
  render until its trigger hydrates, then marks the trigger `data-popup-open`
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:39-97`); moving open
  ownership to a trigger that has not hydrated keeps the tooltip open without calling
  `onOpenChange`
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:99-175`).

## Keyboard interactions

- Escape closes the tooltip; the test presses Escape via `keydown` on `document.body`, so the
  listener is global rather than scoped to the popup, and `onOpenChange` receives the escape-key
  reason (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1032-1055`).
- Trigger focus opens and trigger blur closes the tooltip (see Focus management;
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:85-120`).
- No other keyboard interactions (Tab traversal, arrow keys) are asserted for any tooltip part —
  N/A beyond the above. Any additional keyboard behavior is UNVERIFIED — inferred from
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:1032-1055`, no test asserts it.

## Focus management

- Focusing the trigger opens the tooltip without a hover delay (browser-gated test;
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:85-99`); blurring closes it
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:101-120`).
- With multiple triggers sharing one root/handle, focusing any trigger opens the tooltip and
  blurring closes it (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:585-623`,
  `packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:1035-1076`).
- Focus handoff between two sibling triggers keeps the popup mounted (open state and trigger count
  unchanged) while switching the active payload and moving `data-popup-open` to the new trigger
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:662-709`).
- Focusing a disabled trigger while another trigger's tooltip is open closes the tooltip
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:1078-1116`).
- A focus-opened outer tooltip stays open when the pointer moves onto a nested trigger inside it
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2295-2338`), and a focus-opened inner
  tooltip closes when the inner trigger loses focus
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2417-2459`).
- Focusing a nested tooltip trigger does not open the outer tooltip
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2382-2415`).
- The tooltip popup itself is never focused in any test; whether it is focusable is UNVERIFIED —
  inferred from `packages/react/src/tooltip/root/TooltipRoot.test.tsx:1433-1502`, no test asserts it.

## Accessibility (roles, aria-*, id linking)

- The popup conformance suite for tooltip is configured with hover triggering and no
  `expectedPopupRole`, so no `role`, `aria-expanded`, or `aria-haspopup` assertions run for
  tooltips (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:27-40` with
  `packages/react/test/popupConformanceTests.tsx:68-77`): the popup's ARIA role is
  UNVERIFIED — inferred from `packages/react/src/tooltip/root/TooltipRoot.test.tsx:27-40`, no test asserts it.
- `Tooltip.Arrow` is hidden from assistive technology via `aria-hidden="true"`
  (`packages/react/src/tooltip/arrow/TooltipArrow.test.tsx:47-55`).
- State attributes on the trigger: `data-popup-open` appears while open and is removed as soon as
  `open` becomes false even when the unmount is deferred
  (`packages/react/src/tooltip/trigger/TooltipTrigger.test.tsx:34-78`); `data-trigger-disabled` is
  applied when the root is disabled
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:851-857`).
- Open-state attribute on the popup: `data-open`
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:222-225`).
- A trigger rendered over a disabled toolbar button is exposed with `aria-disabled="true"` rather
  than the `disabled` attribute so it can still be hovered
  (`packages/react/src/tooltip/trigger/TooltipTrigger.test.tsx:110-139`).
- No test asserts an `aria-describedby` or any id/aria link between trigger and popup — UNVERIFIED
  — inferred from `packages/react/src/tooltip/trigger/TooltipTrigger.test.tsx:80-108`, no test asserts it.

## DOM structure & portal behavior

- Composition under test is `Tooltip.Root` → `Tooltip.Trigger` → `Tooltip.Portal` →
  `Tooltip.Positioner` → `Tooltip.Popup` (→ `Tooltip.Arrow`/`Tooltip.Viewport` inside the popup),
  e.g. `packages/react/src/tooltip/arrow/TooltipArrow.test.tsx:13-19`.
- Trigger renders a `button` by default (looked up via `getByRole('button')` throughout,
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:53`) and can render a custom element with
  its own DOM `id` via the `render` prop while still opening the tooltip
  (`packages/react/src/tooltip/trigger/TooltipTrigger.test.tsx:80-108`).
- Portal: by default the closed tooltip is fully unmounted
  (`packages/react/src/tooltip/portal/TooltipPortal.test.tsx:37-41`); with `keepMounted` the closed
  popup stays in the DOM but is inaccessible
  (`packages/react/src/tooltip/portal/TooltipPortal.test.tsx:31-35`).
- Positioner placement: without a `Viewport`, positioning is applied via inline `transform`
  (`packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:295-312`); with a `Viewport`
  it switches to top/left positioning (`packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:314-332`)
  and updates live when the `Viewport` mounts or unmounts
  (`packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:334-369`).
- Positioner offsets: `sideOffset` and `alignOffset` accept numbers or functions receiving a data
  object (positioner/anchor dimensions) and shift the popup by the given amount
  (`packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:65-104`,
  `packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:180-219`). Offset callbacks
  read the resolved (flipped) `side`/`align` — `side="left"` resolves to the opposite side,
  `align="start"` to `end`, and logical `inline-start` to `inline-end`
  (`packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:106-177`,
  `packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:221-293`). The default
  `side`/`align` is UNVERIFIED — inferred from
  `packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:56-82`, no test asserts the default values.
- Hoverability of the positioner: `disableHoverablePopup` sets inline `pointer-events: none` on
  the positioner (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:881-912`);
  `trackCursorAxis="both"` makes it inert the same way, while `"x"` keeps it hoverable
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:915-946`).
- Cursor tracking: with `trackCursorAxis="x"` the positioner tracks the cursor on the first
  delayed hover (popup center within 2px of cursor X)
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1274-1309`), stops tracking after the prop
  is disabled while closed (re-centers on the trigger)
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1311-1373`), and refreshes the tracked
  position on close/reopen (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1375-1431`).
- Arrow: carries `data-side` mirroring the resolved side and `data-open` when open
  (`packages/react/src/tooltip/arrow/TooltipArrow.test.tsx:47-55`); `data-uncentered` is applied
  when the arrow cannot point at the anchor (arrow padding exceeds available space) and omitted
  otherwise (browser-gated;
  `packages/react/src/tooltip/arrow/TooltipArrow.test.tsx:57-67`).
- Viewport: children render inside a `[data-current]` container
  (`packages/react/src/tooltip/viewport/TooltipViewport.test.tsx:26-45`); during trigger switches
  with animations enabled, a `[data-previous]` container (marked `inert`, holding the old content)
  coexists with `[data-current]` and both are removed after the animation
  (`packages/react/src/tooltip/viewport/TooltipViewport.test.tsx:133-235`); the viewport carries
  `data-transitioning` while a morph runs
  (`packages/react/src/tooltip/viewport/TooltipViewport.test.tsx:311-312`) and
  `data-activation-direction` with space-separated `right`/`left`/`down`/`up` tokens describing the
  movement between triggers, with small (≈5px) deltas dropped and horizontal- or vertical-only
  movement reduced to a single token
  (`packages/react/src/tooltip/viewport/TooltipViewport.test.tsx:468-601`).
- Popup animation attributes: `data-instant="focus"` when opened by focus and no attribute when
  opened by hover (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:949-974`);
  `data-instant="delay"` is applied to an adjacent tooltip's popup only while it is opening
  within the provider instant window and removed during its exit
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:608-682`). Any inline `opacity: 0`
  pre-positioning is removed before user CSS transitions run, so no unintended opacity transition
  fires (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:740-783`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onOpenChange(open: boolean, details)` is the only change callback; `details.reason` carries the
  trigger reason (escape-key, imperative-action, `'none'` literals asserted via the imported
  REASONS constants; `packages/react/src/tooltip/root/TooltipRoot.test.tsx:1051-1054`,
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:411-414`,
  `packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:815-818`), and
  `details.trigger` exposes the active trigger object with an `id`
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:870-872`).
- `details.cancel()` vetoes the transition
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:977-1000`).
- `details.preventUnmountOnClose()` defers unmounting through one close cycle
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:297-342`).
- `details.allowPropagation()` lets the Escape keydown propagate (spy confirms zero
  `stopPropagation` calls) while the tooltip still closes — i.e. Escape-close stops propagation by
  default (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1002-1029`).
- `onOpenChangeComplete(open)` fires after the open/close transition settles (see State model;
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:418-605`).
- Pointer events consumed internally: hover opens (mouseenter/mousemove, optionally pointerdown),
  unhover closes (mouseleave, after `closeDelay`), clicking the trigger while open closes it
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1117-1135`) unless
  `closeOnClick={false}` (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1137-1155`), and a
  click/pointerdown on the trigger before the open delay elapses suppresses that open
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1057-1095`) unless `closeOnClick={false}`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1097-1115`). After a click-close, hover
  re-opens the trigger (fresh pointerenter events can be missed; mouse events suffice;
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:1157-1185`).
- No custom DOM events emitted by tooltip parts are asserted (no `onMouseDown`-style public event
  props appear in tests) — N/A beyond the callbacks above.

## Edge cases (rapid interactions, unmount, nesting)

- Hoverable popup: by default the tooltip survives the pointer trip from trigger to popup and
  closes only after leaving the popup (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1433-1489`);
  with `disableHoverablePopup` it closes during that trip
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1491-1501`).
- Crossing gaps between triggers without a `closeDelay` keeps the tooltip open when moving across
  spaced detached triggers (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1504-1555`).
- Nested tooltips: hovering a nested trigger never opens the outer tooltip (at one, two, or three
  nesting levels; `packages/react/src/tooltip/root/TooltipRoot.test.tsx:1562-1600`,
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:1602-1662`,
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:1664-1719`); moving the pointer back to the
  parent trigger area re-opens the parent after its delay, restarting the full delay
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1721-1761`,
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:1805-1856`,
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:2190-2245`); a disabled outer trigger does
  not open (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1763-1803`); a disabled nested
  trigger lets the parent open (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2461-2502`).
- Already-open outer tooltip closes when the pointer moves onto a nested trigger (bubbling
  mouseover detection; `packages/react/src/tooltip/root/TooltipRoot.test.tsx:2247-2293`) and when
  moving from the outer popup to a nested trigger
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2744-2798`); hover movements over a nested
  trigger (safePolygon path) do not open the outer tooltip
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2800-2844`).
- Pending parent reopen is canceled if the pointer leaves the parent trigger entirely
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2095-2144`) and rapid moves back onto the
  nested trigger do not re-open the outer tooltip
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1993-2043`); hovering the nested tooltip's
  popup does not reopen the outer (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2045-2093`).
- Controlled or focus-opened outer tooltips are not closed by nested-trigger hover
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2295-2338`,
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:2340-2380`), and an already-open outer
  tooltip is not re-announced when a scheduled reopen fires (`onOpenChange` stays silent;
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:1858-1928`).
- Touch pointers: the local reopen path is not triggered by touch
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2660-2702`) and mouse hover works after a
  touch interaction (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2704-2742`).
- Shadow DOM: nested-trigger suppression works through shadow roots (composed-path traversal;
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:2504-2573`), and a composed path that
  starts with a ShadowRoot or is empty is handled without breaking the suppression
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:2575-2622`).
- Mount/unmount of the active trigger: the tooltip closes when the active trigger unmounts
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:711-761`,
  `packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:1166-1215`); switching
  triggers reuses the popup/positioner DOM nodes
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:824-858`,
  `packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:1328-1363`) and leaves no
  inline `scale` style on the popup
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:1496-1562`); the
  positioner re-anchors to the newly active trigger
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:1431-1451`).
- Rapid trigger switching with a Viewport keeps the latest morph transition active (newest
  `[data-current]` has one animation; `[data-previous]` holds the previous content)
  (`packages/react/src/tooltip/viewport/TooltipViewport.test.tsx:237-313`), and a lagging payload
  update that remounts the current container mid-morph restarts its entry transition without
  truncating the previous container's exit
  (`packages/react/src/tooltip/viewport/TooltipViewport.test.tsx:315-466`); the `current` container
  remounts on every active-trigger change
  (`packages/react/src/tooltip/viewport/TooltipViewport.test.tsx:72-122`).
- Exiting tooltip unmounts as soon as another tooltip opens (long exit animation is cut short;
  `packages/react/src/tooltip/root/TooltipRoot.test.tsx:684-738`).
- Handle edge cases: `open()` with an unregistered trigger id throws an error naming the id and
  leaves the handle closed
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:391-411`); a trigger
  declared after its root registers fine
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:363-389`); two mounted
  roots sharing one handle trigger a deferred "more than one mounted root" warning
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:413-449`); during a
  transient root overlap, opening by trigger id resolves against the outgoing root without error
  and the popup stays associated through the handoff
  (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:450-512`); a
  default-open root survives a prevented close, root unmount, and open remount cycle
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:1189-1272`).

## Shared harness dependencies

Tests import the shared harness `#test-utils` (mapped to `packages/react/test/index.ts`, which
re-exports the utilities: `packages/react/test/index.ts:1-14`):

- `createRenderer` — act-wrapped `render` with `rerender`/`setProps`, plus the fake `clock` used
  via `clock.withFakeTimers()` and `clock.tick` (`packages/react/test/createRenderer.ts:27-49`).
- `describeConformance` — per-part conformance suite (ref type, render, render-prop, className,
  props forwarding) used by every part test above.
- `popupConformanceTests` — shared popup conformance; for tooltip it is configured with
  `triggerMouseAction: 'hover'` and no expected popup role, asserting the `open` prop mounts the
  popup and a closed popup is absent, plus (browser-only) removal when no exit animation is
  defined (`packages/react/test/popupConformanceTests.tsx:6-15`,
  `packages/react/test/popupConformanceTests.tsx:125-153`,
  configured at `packages/react/src/tooltip/root/TooltipRoot.test.tsx:27-40`).
- `resetBrowserPointer` — run in `beforeEach` because hover tests leave the real pointer resting
  on a trigger (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:13`,
  `packages/react/src/tooltip/provider/TooltipProvider.test.tsx:10`,
  `packages/react/src/tooltip/trigger/TooltipTrigger.test.tsx:11`).
- `waitForPositioned`, `waitSingleFrame`, `advanceReactClock`, `isJSDOM` — used for
  layout/animation gating and fake-clock advancement
  (`packages/react/src/tooltip/positioner/TooltipPositioner.test.tsx:5`,
  `packages/react/src/tooltip/viewport/TooltipViewport.test.tsx:5`,
  `packages/react/src/tooltip/provider/TooltipProvider.test.tsx:4`).
- `@mui/internal-test-utils` (external package, not read here): `screen`, `fireEvent`, `act`,
  `waitFor`, `flushMicrotasks`, `ignoreActWarnings`, `randomStringValue`.
- Environment gating: layout/animation-dependent tests are browser-only via
  `it.skipIf(isJSDOM)`/`describe.skipIf(isJSDOM)`; SSR/hydration tests are jsdom-only via
  `it.skipIf(!isJSDOM)` (`packages/react/src/tooltip/root/TooltipRoot.detached-triggers.test.tsx:39-97`).
- Unit-internal constants imported by tests: `OPEN_DELAY` (default open delay;
  `packages/react/src/tooltip/provider/TooltipProvider.test.tsx:5`) and `REASONS`
  (`packages/react/src/tooltip/root/TooltipRoot.test.tsx:8`); their literal values live in source
  files outside this mining scope, so numeric/string literals are quoted above only where a test
  asserts them directly.
