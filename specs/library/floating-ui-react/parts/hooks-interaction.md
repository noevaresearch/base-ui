# floating-ui-react — interaction hooks (behavior spec)

Scope note: this spec mines ONLY the behavior proven by the unit tests of the interaction hooks in `packages/react/src/floating-ui-react/hooks/`. Positioning math (x/y computation, middleware, platform internals) is delegated to `@floating-ui/react-dom` and `@floating-ui/utils` and is out of scope; the Rust-equivalent crate for Stage 3 is `floating-ui-leptos` (https://floating-ui.rustforweb.org/frameworks/leptos.html), which should be bound against instead of reimplementing delegated behavior. Every non-trivial claim cites the proving test (or harness/fixture) file and lines.

## Public API surface (props, parts, subcomponents)

### useClick

- `useClick(context, props)` is composed with `useFloating`'s returned `context`; its element props are consumed through `getReferenceProps` / `getFloatingProps` and spread onto the reference (a `button`, or an `input` when `typeable`) and the floating element respectively (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:10-37`).
- Options proven:
  - `toggle` (boolean): when `false`, repeated clicks keep the popup open instead of toggling (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:106-115`); default behavior toggles open/closed on repeated clicks (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:92-104`).
  - `event`: `'mousedown'` makes both open and close come from the mousedown event path (pointerdown + mousedown + click dispatch) (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:117-135`); `'mousedown-only'` opens from mousedown and ignores the trailing `click` event (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:137-147`).
  - `ignoreMouse` (boolean): when `true`, a mouse `pointerdown` + `click` sequence does not open (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:149-158`).
  - `touchOpenDelay` (number, ms): a touch `pointerdown` + `click` does not open immediately; opening happens after the delay elapses (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:160-177`). With `event: 'mousedown'`, the deferred path additionally requires the pending animation-frame callbacks to run before the timer produces the open (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:179-211`). Closing is never delayed by it (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:213-224`).
  - `reason`: overrides the reason reported to `onOpenChange`; e.g. `reason={REASONS.inputPress}` with a typeable input reference yields details `{ reason: REASONS.inputPress }` (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:226-237`).
  - `stickIfOpen` (boolean): combined with `useHover`, `true` keeps a hover-opened popup open on the first click (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:239-267`); `false` closes it on the first click (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:269-297`). Default value is UNVERIFIED — inferred from `packages/react/src/floating-ui-react/hooks/useClick.test.tsx:239-267`, no test asserts the default.

### useClientPoint

- `useClientPoint(context, { enabled, axis })` positions the floating element at the pointer; the returned element props are consumed via `getReferenceProps`, `getTriggerProps`, or `getFloatingProps` (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:41-111`).
- `axis`: `'x'` tracks only horizontal movement (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:324-340`), `'y'` only vertical (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:342-358`); default is both axes (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:171-242`).
- `enabled`: `false` detaches tracking and restores the DOM reference rect (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:287-299`); default `true` (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:41-60`).
- The rendered position is exposed through `elements.reference.getBoundingClientRect()`; with `axis` restrictions the untracked coordinate comes from the element rect (or 0 for a zero-size element) (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:143-169`, `:324-358`). The actual coordinate→position algorithm is delegated to `@floating-ui/react-dom`; Rust equivalent: `floating-ui-leptos`.

### useDismiss

- `useDismiss(context, props)` with element props consumed via `getReferenceProps` / `getFloatingProps` (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:34-69`).
- Options proven:
  - `escapeKey` (boolean, default true): `false` disables Escape dismissal (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:498-503` vs default `:94-99`).
  - `outsidePress` (boolean | function, default true): `false` disables outside-press dismissal (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:505-509`); a function guard receives the event and can veto — `outsidePress={(event) => !(event.target as Element)?.closest('[data-testid="ignore"]')}` (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:1364-1381`), `() => false` keeps open (`:264-268`), `() => true` dismisses (`:551-555`).
  - `referencePress` (function returning boolean): when the guard returns true, a `pointerdown` (and a native `click`) on the reference dismisses with reason `REASONS.triggerPress` (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:252-262`); returning `false` prevents dismissal (`:511-515`).
  - `outsidePressEvent`: `'intentional'` enables the press-observation model described under Edge cases (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:1025-1506`); `'sloppy'` is exercised in the strict-mode marker test (`:187-250`).
  - `bubbles` (boolean | `{ escapeKey?: boolean; outsidePress?: boolean }`): controls whether dismissal cascades to parent floating elements (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:558-848`).
  - `capture` (boolean | `{ escapeKey?: boolean; outsidePress?: boolean }`): controls whether listeners run in the capture phase so they observe events that are later stopped (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:851-1022`).
- Exports a `normalizeProp` helper that normalizes the `bubbles`/`capture` shapes (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:605-647`, `:851-902`).

### useFloating

- `useFloating({ open, onOpenChange, ... })` returns `{ refs, context, elements, floatingStyles }`; `open` + `onOpenChange` form the controlled-state contract (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:21-27`, `packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:1520-1538`).
- Options seen in tests: `open`, `onOpenChange`, `nodeId`, `placement`, `middleware`, `whileElementsMounted` (fixture only: `packages/react/test/floating-ui-tests/Popover.tsx:113-120`), and `rootContext` (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:23-26`).
- An alternative entry point `useBaseUIFloating({ rootContext })` binds to an externally owned `FloatingRootStore` (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:28-44`).
- Related exported parts used throughout the tests: `FloatingNode`, `FloatingTree`, `useFloatingNodeId`, `useFloatingParentNodeId`, `FloatingPortal`, `FloatingFocusManager` (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:8-19`).
- Positioning itself (placement/middleware/`floatingStyles` computation) is delegated to `@floating-ui/react-dom`; Rust equivalent: `floating-ui-leptos`.

### useFocus

- `useFocus(context, { delay })` — element props via `getReferenceProps` / `getFloatingProps`; the only proven option is `delay` (`packages/react/src/floating-ui-react/hooks/useFocus.test.tsx:24-26`).

### useHover

- `useHover(context, props)` — element props via `getReferenceProps` / `getFloatingProps` (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:12-26`).
- Options proven:
  - `delay`: number (symmetric) or `{ open?, close? }` in ms (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:49-133`).
  - `restMs` (number, ms): open only after the pointer rests (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:136-171`).
  - `handleClose`: a closeness function — `safePolygon({ blockPointerEvents })` (fixture usage: `packages/react/test/floating-ui-tests/Popover.tsx:128-136`; option plumbing proven via `useHoverReferenceInteraction` below).
- `safePolygon({ blockPointerEvents })` is the exported handleClose factory (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:18-22`).

### useHoverReferenceInteraction

- Internal split-out of hover reference handling: `useHoverReferenceInteraction(context, { handleClose, mouseOnly, restMs, delay, move, triggerElementRef, isClosing })` returns hover element props to spread on a wrapper (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:48-54`, `:250-255`, `:323-329`).
- `move: false` disables move-based (restMs) handling in these tests; `mouseOnly: true` with `pointerType: 'mouse'` events (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:48-54`).
- Companion `useHoverInteractionSharedState(rootStore)` exposes `handleCloseOptions` (e.g. `blockPointerEvents`) that updates synchronously during render (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:13-32`).
- `triggerElementRef` identifies the active trigger when handlers live on a wrapper; `isClosing: () => boolean` lets an owner report an in-progress close transition (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:301-329`).

## State model (controlled/uncontrolled, defaults, transitions)

### useClick

- Open state is fully controlled by the consumer: `useFloating({ open, onOpenChange })` with `onOpenChange(nextOpen, details)` calling `setOpen(nextOpen)` (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:20-27`). No uncontrolled mode is exercised.

### useClientPoint

- No open state of its own; the test toggles `isOpen` via a plain button and `useFloating({ open: isOpen, onOpenChange: setIsOpen })` (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:52-56`). Tracking is active by default (`enabled: true`) and deactivated by `enabled: false` (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:287-299`).
- When a virtual `positionReference` is set on the root store, disabling the hook resets `store.state.positionReference` back to the DOM reference, and unmount clears it to `null` (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:302-322`).

### useDismiss

- Controlled open with default `true` in the shared fixture; every dismissal funnels through `onOpenChange(false, details)` (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:39-56`).
- Defaults: `escapeKey: true` (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:94-99`), `outsidePress: true` (`:181-185`); `bubbles` normalizes to `escapeKey: false, outsidePress: true` (`:606-611`); `capture` normalizes to `escapeKey: false, outsidePress: true` (`:853-858`); `bubbles: false`/`capture: false` set both to `false` (`:613-619`, `:877-883`); `{ escapeKey: X }` leaves outsidePress at its default, `{ outsidePress: X }` leaves escapeKey `false` (`:630-646`, `:885-901`).
- Transition: a dismissal may be vetoed — calling `details.cancel()` inside `onOpenChange` keeps the popup open and the Escape keydown is then not default-prevented (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:115-147`).

### useFloating

- With `rootContext`, state lives in an external `FloatingRootStore` whose `state` exposes `open`, `transitionStatus`, `referenceElement`, `domReferenceElement`, `floatingElement`, `triggerElements`, and whose `context` exposes `dataRef`, `events`, `floatingId`, `triggerElements` (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:9-21`, `:79-102`; `packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:27-39`).
- Rendering with `useFloating({ rootContext })` syncs the store's element state from the refs: after render `store.state.referenceElement` / `domReferenceElement` / `floatingElement` point at the DOM nodes (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:84-88`).
- `refs.setPositionReference(virtualElement)` switches `refs.reference.current` / `elements.reference` to the virtual element while `store.state.referenceElement` and `refs.domReference.current` keep the DOM node (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:90-102`).
- `context.rootStore`, `context.dataRef`, and `context.events` are identity-shared with the store (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:79-83`).
- Switching the `rootContext` prop between stores makes `useFloating` bind to the new store without leaking the old store's floating element (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:46-61`).

### useFocus

- Controlled open via `useFloating({ open, onOpenChange: setOpen })` (`packages/react/src/floating-ui-react/hooks/useFocus.test.tsx:19-23`).

### useHover

- Controlled open (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:12-18`). Defaults: no open delay, and close delay 0 — with `delay={{ open: 500 }}`, mouseleave closes immediately (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:105-121`).
- Transitions: open fires immediately on mouseenter by default (`:33-39`) or after `delay`/`restMs`; a pending delayed open is canceled if the DOM reference changes/unmounts (`:231-247`).

### useHoverReferenceInteraction

- State derives from the shared root store; `handleCloseOptions` (e.g. `blockPointerEvents`) is updated during render so it is observable synchronously on rerender (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:13-32`). Tests simulate a close transition by setting `context.rootStore.state.transitionStatus = 'ending'` while `open` is false (`:246-248`).

## Keyboard interactions

### useClick

- N/A — no keyboard activation (Enter/Space) is tested (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:1-298`).

### useClientPoint

- N/A (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:1-523`).

### useDismiss

- Escape keydown anywhere (dispatched on `document.body`) dismisses the open popup by default (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:94-99`); `escapeKey: false` disables it (`:498-503`).
- Escape while an IME composition is active (compositionstart → keyDown Escape → compositionend) does not dismiss; a subsequent Escape after composition ends does (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:149-179`).
- Escape bubbles across a nested floating tree only when `bubbles.escapeKey` is true: with `bubbles` on all levels one Escape closes outer and inner (`:785-801`); with `bubbles={{ escapeKey: false }}` the first Escape closes only the inner and a second closes the outer (`:803-824`); mixed configurations stop at the innermost non-bubbling level (`:826-847`). Without a `FloatingTree` connection, Escape affects only the innermost popup (`:716-783`).
- `capture.escapeKey: false` makes dismissal respect a `stopPropagation()` on the keydown (overlay stops Escape; inner closes first, then outer on a second press) (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:904-919`, `:997-1022`).

### useFloating

- N/A (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:1-103`).

### useFocus

- N/A — the only test guards against focus-restoration reopening, not keyboard activation (`packages/react/src/floating-ui-react/hooks/useFocus.test.tsx:17-51`).

### useHover

- N/A (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:1-369`).

### useHoverReferenceInteraction

- N/A (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:1-373`).

## Focus management

### useClick

- N/A (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:1-298`).

### useClientPoint

- Opening via a focus event with reason `REASONS.triggerFocus` (through `context.rootStore.setOpen(true, createChangeEventDetails(REASONS.triggerFocus, new FocusEvent('focus'), domReference))`) restores the DOM reference as the positioning target: the rect reverts from the tracked mouse point to the element rect and later mousemoves do not move it (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:74-81`, `:469-523`).

### useDismiss

- `FloatingFocusManager` wrapping the floating element changes outside-press semantics: clicking an element that is outside the React tree ("third-party", appended directly to `document.body`) does NOT dismiss while focus is managed (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:270-304`); without the focus manager the same outside click dismisses (`:181-185`).
- Nested floating elements each wrapped in `FloatingFocusManager` (modal and non-modal combinations, and no manager at all) still resolve inside/outside correctly: clicking the child floating keeps both open; clicking the parent floating dismisses only the child (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:389-494`).
- The bubble/capture suites wrap dialogs in `FloatingFocusManager` inside `FloatingPortal` (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:581-586`, `:943-951`).

### useFloating

- N/A directly; element syncing with an external store is covered under State model (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:63-102`).

### useFocus

- Focus-restoration guard: after `window` blur and a subsequent focus event on the reference, a `useFocus` popup with `delay: 100` does NOT reopen (no tooltip after 200ms) (`packages/react/src/floating-ui-react/hooks/useFocus.test.tsx:17-51`).

### useHover

- N/A (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:1-369`).

### useHoverReferenceInteraction

- N/A (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:1-373`).

## Accessibility (roles, aria-*, id linking)

### useClick

- The hooks themselves add no proven aria attributes; the fixtures apply `role="tooltip"` to the floating element purely as a test handle (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:34`).

### useClientPoint

- N/A (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:1-523`).

### useDismiss

- N/A for aria wiring; fixtures use `role="tooltip"` / `role="dialog"` as handles (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:63`, `:288`).

### useFloating

- `context.floatingId` is the id-linking primitive: the Popover fixture wires `aria-controls={open ? context.floatingId : undefined}` on the trigger and `id={context.floatingId}` on the floating element, plus fixture-owned `aria-haspopup`, `aria-expanded`, `aria-labelledby`, `aria-describedby` and `React.useId`-derived ids (`packages/react/test/floating-ui-tests/Popover.tsx:140-163`). This wiring is fixture-level, not hook-emitted.

### useFocus

- N/A (`packages/react/src/floating-ui-react/hooks/useFocus.test.tsx:1-52`).

### useHover

- N/A; fixture uses `role="tooltip"` (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:23`).

### useHoverReferenceInteraction

- N/A (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:1-373`).

## DOM structure & portal behavior

### useClick

- The floating element is conditionally rendered only while open; no portal is exercised (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:31-36`).

### useClientPoint

- The reference element is a zero-size, `pointer-events: none` div; the floating element is rendered only while open and the hook re-positions by replacing `elements.reference`'s rect (virtual reference) rather than moving DOM (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:84-108`).

### useDismiss

- Clicks on children portaled out of the floating element via `FloatingPortal` still count as "inside" and do not dismiss (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:517-549`).
- `FloatingPortal` accepts a `container` — tests portal into an open shadow root; outside clicks on `document.body` dismiss even though the floating element lives in the shadow root, both without and with `FloatingFocusManager` (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:306-387`).
- Nested floating elements may use different portal containers (one default, one a custom div) and still nest correctly with `useClick` + `useDismiss` + `FloatingFocusManager modal={false}` (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:1509-1577`).

### useFloating

- N/A — no DOM structure assertions beyond refs (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:1-103`).

### useFocus

- N/A (`packages/react/src/floating-ui-react/hooks/useFocus.test.tsx:1-52`).

### useHover

- N/A (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:1-369`).

### useHoverReferenceInteraction

- N/A (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:1-373`).

## Events (names, payload shape, bubbling, preventDefault semantics)

### useClick

- Listens (via spread props) to `click` by default; `event: 'mousedown'`/`'mousedown-only'` shift the trigger to the pointerdown/mousedown path; `pointerdown` `pointerType` is inspected (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:56-68`, `:117-158`).
- Payload: `onOpenChange(nextOpen, details)` with `details.reason`; `reason` option overrides it (e.g. `REASONS.inputPress`) (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:226-237`).
- Touch path is keyed off `pointerdown` `pointerType: 'touch'` and defers via `touchOpenDelay` (`:160-211`).

### useClientPoint

- Listens to `mousemove` on the reference (and trigger) props; while open, a `window`-level `mousemove` listener tracks the pointer anywhere (verified by dispatching on `document.body`) (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:171-241`).
- The window listener is registered only while the floating element is open — window mousemove while closed is ignored (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:202-215`).
- Moving the cursor onto the floating element removes the window listener (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:360-395`); returning to the reference re-attaches it (`:397-467`).
- Coordinates come from `clientX`/`clientY`; with `axis: 'x'`/`'y'` only the tracked axis updates (`:124-141`, `:324-358`).

### useDismiss

- Document-level listeners (registered while open): `keydown` for Escape (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:94-113`), and for outside press `pointerdown` / `mousedown` / `click` / `pointerup` / `pointercancel` sequences plus passive `{ capture: true, passive: true }` `touchstart`/`touchmove`/`touchend` listeners on `document` (`:73-92`, `:1025-1506`).
- Payload: `onOpenChange(false, details)` with `details.reason` — `REASONS.outsidePress` for outside press, `REASONS.escapeKey` for Escape, `REASONS.triggerPress` for reference press (asserted in the shared fixture) (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:44-54`).
- preventDefault semantics: a dismissing Escape keydown is default-prevented (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:101-113`); if the close is canceled via `details.cancel()`, the event is NOT default-prevented (`:115-147`).
- Bubbling: outside press and Escape dismissals cascade through the `FloatingTree` per the `bubbles` option — default outsidePress bubbles, escapeKey does not (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:649-848`); React-tree `stopPropagation()` on pointerdown/keydown is bypassed by `capture: true` listeners (`:904-995`) and respected when capture is off (`:997-1022`).

### useFloating

- N/A — no event listeners asserted (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:1-103`).

### useFocus

- Listens to focus/blur lifecycle including the window `blur` event; the guard is against reopen after tab-blur + refocus (`packages/react/src/floating-ui-react/hooks/useFocus.test.tsx:39-51`).

### useHover

- Listens (via spread props) to `mouseenter` / `mouseleave` and `mousemove` (for `restMs`), using `movementX`/`movementY` to distinguish real movement (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:33-47`, `:136-213`).
- Payload: `onOpenChange(nextOpen, data)` with `data.reason === REASONS.triggerHover` on both open and close (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:249-276`).
- Target resolution uses the native event path, not React's synthetic `event.target`: a `mousemove` on a child whose `composedPath` is skewed still does not emit an openchange, and the popup stays open (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:278-320`).
- `mouseleave` from the reference with `relatedTarget` = the floating element closes the popup (default, no `handleClose`) (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:215-229`).

### useHoverReferenceInteraction

- Listens to `pointerenter`/`mouseenter` on the wrapper and `mousemove` on the trigger; a `mousemove` with `movementX: 10` over the active trigger emits no redundant openchange (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:77-85`).
- Same native-path target resolution as useHover: skewed `composedPath` on a child `mousemove` produces no openchange (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:136-158`).
- Moving over a child trigger registered as disabled (`data-trigger-disabled` attribute + `triggerElements.add('disabled-trigger', node)`) in wrapper fallback mode is treated as inactive, producing exactly one `onOpenChange` call while the tooltip remains rendered (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:190-221`).

## Edge cases (rapid interactions, unmount, nesting)

### useClick

- Repeated rapid clicks toggle closed by default and stay open with `toggle: false` (`packages/react/src/floating-ui-react/hooks/useClick.test.tsx:92-115`); the mousedown-only path swallows the trailing click to avoid double-toggling (`:137-147`).

### useClientPoint

- Closing or disabling removes the window listener; a final `document.body` mousemove after disable leaves the rect at the DOM reference's rect (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:244-300`).
- Unmount with a virtual `positionReference` set clears `store.state.positionReference` to `null` (no stale reference retained) (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:302-322`).
- Rapid successive mousemoves reposition on each event (`packages/react/src/floating-ui-react/hooks/useClientPoint.test.tsx:171-201`).

### useDismiss

- `outsidePressEvent: 'intentional'` press-observation model (all citations `packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx`):
  - A drag starting inside the floating element and releasing outside does not close (`:1026-1033`); the reverse drag (press outside, release inside) neither closes nor lets the gesture's trailing `click` close (`:1035-1046`); the one-shot suppression is consumed by the first post-drag outside click, and a further outside press+click closes (`:1048-1076`).
  - A trailing `click` (`detail: 1`) whose press began before open does not close (`:1078-1091`); compatibility `mousedown`/`mouseup`/`click` events belonging to the `pointerdown` that opened the popup do not count as a new press (`:1093-1127`).
  - Keyboard-generated `detail: 0` clicks with no press close immediately (`:1129-1135`); press-less clicks reporting `pointerType: 'mouse'` (Android AT) also close — click `detail` is the discriminator (`:1137-1146`).
  - A press observed in a previous open session does not leak into a reopen (`:1148-1187`), including same-batch close+reopen where React never renders `open === false` (`:1189-1234`); a redundant `setOpen(true)` dispatch mid-gesture keeps the press on record (`:1236-1272`); the press survives listener re-attachment caused by a prop change mid-gesture (`:1274-1286`).
  - A `pointerdown`-only press (no compat `mousedown`) counts (`:1288-1296`); non-primary-button presses (`button: 2`) do not (`:1298-1311`); cancelled presses (`pointercancel`) do not (`:1313-1327`).
  - Inside clicks never dismiss and do not consume the drag suppression (`:1329-1362`); a drag ending on an `outsidePress`-ignored target does not consume the next outside click (`:1364-1381`).
  - A press start prevented inside (`onPointerDown preventDefault`) suppresses only the immediate synthetic outside click; after a tick (or after `pointercancel`), the next outside click closes (`:1383-1506`).
- Unmount/strict mode: unmounting the component that owns the dismiss interaction clears the "inside" marker, so a later outside press dismisses even under `React.StrictMode` (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:187-250`).
- Nesting: `bubbles.outsidePress` default `true` closes all stacked dialogs on one outside press; `false` confines it to the innermost (`packages/react/src/floating-ui-react/hooks/useDismiss.test.tsx:649-712`); Escape analogues (`:715-848`); nested floating elements with different portal containers keep independent open state (`:1509-1577`).

### useFloating

- Rerendering with a different `rootContext` re-binds cleanly; the previous store's `floatingElement` is preserved and the new store's is set (`packages/react/src/floating-ui-react/hooks/useFloating.test.tsx:46-61`).

### useFocus

- Focus restore after window blur does not reopen (`packages/react/src/floating-ui-react/hooks/useFocus.test.tsx:39-51`).

### useHover

- Rapid enter/leave around the delay boundary: open fires exactly at the delay expiry, not before (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:50-84`); leaving before the open delay elapses cancels the open (`:105-121`).
- `restMs` timer resets on significant movement (`movementX/Y: 10`) but not on minor movement (`movementX: 1, movementY: 0`) (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:136-213`).
- `restMs` is combined correctly with a nullish open delay (`delay={{ close: 100 }}` still respects `restMs: 100`) (`:123-133`).
- Unmounting/changing the DOM reference cancels a pending delayed open (`:231-247`).
- `blockPointerEvents` from `safePolygon` is cleaned up when the trigger changes, keeping parent popovers clickable after the child closes (`packages/react/src/floating-ui-react/hooks/useHover.test.tsx:322-368`, fixture `packages/react/test/floating-ui-tests/Popover.tsx:128-136`).
- Touch + restMs behavior is a `test.todo` — UNVERIFIED — inferred from `packages/react/src/floating-ui-react/hooks/useHover.test.tsx:173-184`, no test asserts this.

### useHoverReferenceInteraction

- Delegated wrapper mode: re-entering the same trigger during an active close transition (`transitionStatus: 'ending'`) reopens immediately, bypassing the open delay; `onOpenChange` is called twice with the second call `true` (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:224-299`).
- The same immediate-reopen works when the closing state is provided externally via `isClosing: () => !open` (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:301-373`).
- `handleCloseOptions.blockPointerEvents` reflects prop changes synchronously across rerenders (`packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.test.tsx:13-32`).

## Shared harness dependencies

- `packages/react/test/index.ts` — the `#test-utils` barrel: re-exports `useTestInteractions`, `isJSDOM` (via `@base-ui/utils/testUtils`), `firePointer`/`enterWithMouse`/`moveMouse`, `waitFor*` helpers and others used across these tests (`packages/react/test/index.ts:1-14`).
- `packages/react/test/useTestInteractions.ts` — provides `useTestInteractions(propsList)`, the harness that merges each interaction hook's `reference`/`floating`/`item`/`trigger` element-prop objects into `getReferenceProps`/`getFloatingProps`/`getItemProps`/`getTriggerProps`; floating props always include `tabIndex: -1` and the focusable data attribute, and duplicate event handlers are chained with the first non-undefined return value winning (`packages/react/test/useTestInteractions.ts:30-67`, `:81-84`, `:107-151`).
- `packages/react/test/floating-ui-tests/Popover.tsx` — nested-Popover fixture used by `useHover.test.tsx` for the `blockPointerEvents` cleanup scenario; composes `useHover` (with `safePolygon({ blockPointerEvents: true })`), `useClick`, `useDismiss({ bubbles })`, `FloatingNode`/`FloatingTree`, `FloatingPortal`, `FloatingFocusManager`, and fixture-level aria wiring (`packages/react/test/floating-ui-tests/Popover.tsx:102-176`, `:179-192`).
- `@mui/internal-test-utils` — external (non-repo) test package supplying `render`, `screen`, `fireEvent`, `act`, `flushMicrotasks`, `waitFor`, `userEvent` wrappers; imported directly by every batch test (e.g. `packages/react/src/floating-ui-react/hooks/useClick.test.tsx:2`).
