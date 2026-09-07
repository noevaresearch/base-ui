# PreviewCard — behavior spec (mined from tests)

Unit: `packages/react/src/preview-card/`. TODO.md entry has no `wraps-external:` field — behavior below is mined entirely from this unit's own test files. Every claim cites a test file line; claims without test coverage are marked UNVERIFIED.

## Public API surface (props, parts, subcomponents)

Exported from `@base-ui/react/preview-card`:

- `PreviewCard.Root` — state owner. Props exercised by tests: `open`, `defaultOpen`, `onOpenChange`, `onOpenChangeComplete`, `actionsRef` (`PreviewCard.Root.Actions` with `close()` and `unmount()`), `handle`, `triggerId`, `defaultTriggerId`, plus a function-children API `({ payload }) => ReactNode` `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:1026-1032`
- `PreviewCard.createHandle()` — factory for a handle linking detached triggers to a root; generic over the payload type (`PreviewCard.createHandle<number>()`). Handle exposes `open(triggerId)`, `close()`, and `isOpen` `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:94-106`
- `PreviewCard.Trigger` — renders an anchor (`refInstanceof: window.HTMLAnchorElement`) by default; accepts `href`, `delay`, `closeDelay`, `payload`, `handle`, `id`, `tabIndex`, and `render` (a custom element, e.g. `<div />`) `packages/react/src/preview-card/trigger/PreviewCardTrigger.test.tsx:8-13`
- `PreviewCard.Portal` — portal wrapper; accepts `keepMounted` and `container` `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:930-934`
- `PreviewCard.Positioner` — positioning wrapper; accepts `side`, `align`, `sideOffset` (number or function), `alignOffset` (number or function), `anchor` (ref), `collisionBoundary`, `collisionPadding` `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:112-130`
- `PreviewCard.Popup` — content surface; renders children `packages/react/src/preview-card/popup/PreviewCardPopup.test.tsx:54-66`
- `PreviewCard.Arrow` — `refInstanceof: window.HTMLDivElement`, rendered inside `Popup` `packages/react/src/preview-card/arrow/PreviewCardArrow.test.tsx:8-21`
- `PreviewCard.Backdrop` — `refInstanceof: window.HTMLDivElement`, rendered as a sibling of `Positioner` inside `Portal` `packages/react/src/preview-card/backdrop/PreviewCardBackdrop.test.tsx:22-27`
- `PreviewCard.Viewport` — optional inner container inside `Popup` that hosts morphing content containers `packages/react/src/preview-card/viewport/PreviewCardViewport.test.tsx:46-65`

Context errors (each part validates its required ancestors):

- `Popup` outside `Root`: "Base UI: PreviewCardRootContext is missing. PreviewCard parts must be placed within <PreviewCard.Root>." `packages/react/src/preview-card/popup/PreviewCardPopup.test.tsx:22-32`
- `Popup` outside `Positioner`: "Base UI: PreviewCardPositionerContext is missing. PreviewCardPositioner parts must be placed within <PreviewCard.Positioner>." `packages/react/src/preview-card/popup/PreviewCardPopup.test.tsx:34-52`
- `Viewport` outside `Positioner`: same PreviewCardPositionerContext error as Popup `packages/react/src/preview-card/viewport/PreviewCardViewport.test.tsx:26-44`
- `Positioner` outside `Portal`: "Base UI: <PreviewCard.Portal> is missing." `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:87-101`
- `Trigger` outside `Root` without a handle: "Base UI: <PreviewCard.Trigger> must be either used within a <PreviewCard.Root> component or provided with a handle." `packages/react/src/preview-card/trigger/PreviewCardTrigger.test.tsx:15-25`

The root suite runs shared popup conformance tests (`popupConformanceTests`) with `triggerMouseAction: 'hover'` across contained-trigger, detached-trigger, and multiple-detached-trigger layouts `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:31-46`

## State model (controlled/uncontrolled, defaults, transitions)

- Uncontrolled: hovering the trigger opens the card after an open delay; unhovering closes it after a close delay. Tests drive this with the shared `OPEN_DELAY` / `CLOSE_DELAY` constants imported from `packages/react/src/preview-card/utils/constants.ts` (the concrete default values are not asserted numerically by any test) `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:56-92`
- `defaultOpen: true` renders the card open on mount and stays uncontrolled — the card still closes on trigger unhover `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:282-343`
- `defaultOpen` is ignored when `open` is controlled: `open={false}` keeps it closed and `open={true}` keeps it open `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:297-321`
- Controlled `open` + `onOpenChange(nextOpen, details)`: `onOpenChange` fires exactly once per transition (twice for an open+close cycle) and is not called when state does not change `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:132-176`
- A popup opened by trigger hover closes on hover-out of the positioner; a popup opened externally (state set programmatically, including `defaultOpen`) does NOT close on hover-out `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:178-242`
- `delay` uses rest-type timing: with `delay={100}` the card is still closed immediately after hover and opens only after the 100ms tick `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:370-391`
- `closeDelay` defers close: content remains present immediately after `mouseLeave` and is removed only after the delay elapses `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:393-418`
- With `delay={0}`, opening happens synchronously inside the hover handler (the test asserts visibility without awaiting) `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:400-404`
- Active-trigger tracking: `triggerId` / `defaultTriggerId` control which trigger the card is anchored to; the root's function children receive `{ payload }` taken from the active trigger's `payload` prop, `undefined` when no trigger is active `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:114-136`
- `defaultOpen` + `defaultTriggerId` renders open with that trigger's payload on mount `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:843-883`
- Programmatically setting `open` + `triggerId` (e.g. from `onOpenChange` reading `details.trigger?.id`) fully controls open state and payload `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:778-841`
- Switching the active trigger (hover or focus) updates the payload and repositions while keeping the same popup/positioner DOM nodes `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:741-776`
- While moving hover from an already-active trigger to another trigger, the new trigger's `delay` is ignored and the card switches immediately `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:427-463`
- When the active trigger unmounts, the card closes; the close is cancelable via `details.cancel()`, in which case the card stays open with the same payload `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:626-739`
- Handle-backed detached roots: imperative calls on a handle with no mounted root are ignored (with a console warning containing "no root using this handle is mounted") both before attachment and after detachment `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:93-212`
- `handle.open('missing')` throws synchronously when the trigger id is not registered; `handle.isOpen` stays false `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:242-262`
- A handle attached to more than one simultaneously mounted root triggers a deferred warning containing "more than one mounted root"; a transient overlap during root handoff is resolved — opening by trigger id during the overlap succeeds and the popup stays associated through the handoff `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:264-362`
- A `defaultOpen` root whose detached trigger migrates to it after the initial commit stays open with `onOpenChange` never called `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:33-91`
- `actionsRef.current.close()` closes the card; the resulting `onOpenChange` last call has `reason: REASONS.imperativeAction` and the trigger's `data-popup-open` attribute is removed `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:522-548`
- `onOpenChangeComplete(nextOpen)` fires after each transition settles: on close without an exit animation, on close after the exit animation finishes, on open without an enter animation, and on open after the enter animation finishes; it is NOT called on mount when the card starts closed `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:551-734`

## Keyboard interactions

- Focus-visible on the trigger opens the card (the test is jsdom-only because the browser path requires `:focus-visible`) `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:94-111`
- Blur on the trigger closes the card (the test ticks `CLOSE_DELAY` before asserting removal) `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:113-126`
- `Escape` (keydown dispatched on `document.body`) closes the open card; afterwards a plain re-hover of the trigger reopens it `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:447-475`
- After Escape-close, focusing a different trigger reopens the card `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:512-549`
- Tabbing to a trigger opens the card positioned above the trigger's first line (`side="top"`) `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:976-1024`
- No other keyboard interactions (arrow keys, Enter, etc.) are asserted by any test — N/A beyond the above.

## Focus management

- Focus is not moved into the popup on open; no test asserts any focus trap or return-focus behavior — UNVERIFIED — inferred from absence across `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx`, no test asserts this.
- Focus/blur on any of multiple triggers (contained or handle-detached) opens/closes the shared card; switching focus between triggers while open swaps payload immediately (the second trigger's `delay={2000}` is ignored) `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:465-588`
- The active trigger carries a `data-popup-open` attribute while open (observed on focus-open and imperative handle-open) `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:68-69`

## Accessibility (roles, aria-*, id linking)

- The default trigger renders as a link (`getByRole('link')` in every trigger-driven test; conformance `refInstanceof: window.HTMLAnchorElement` with `href="#"`) `packages/react/src/preview-card/trigger/PreviewCardTrigger.test.tsx:8-13`
- Trigger id linking: the root's `triggerId` / `defaultTriggerId` and the handle's `open(triggerId)` reference the trigger's DOM `id`; `onOpenChange` details expose `details.trigger` with `.id` so consumers can mirror active-trigger state `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:778-841`
- While open, the active trigger element has `data-popup-open`; it is removed on close `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:1718-1731`
- State attributes on the popup: `data-open` / `data-closed` for the resting states, `data-starting-style` / `data-ending-style` during enter/exit transitions (tests bind CSS animations to `[data-ending-style]` and `[data-starting-style]`) `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:1461-1463`
- The positioner exposes the resolved side via `data-side` (e.g. `data-side="top"` after collision flip) `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:965-973`
- The previous viewport morph container is marked `inert` during transitions `packages/react/src/preview-card/viewport/PreviewCardViewport.test.tsx:211-217`
- No test asserts `aria-expanded`, `aria-haspopup`, `aria-describedby`, or popup↔trigger `aria-*` id linkage — UNVERIFIED — inferred from absence across `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx`, no test asserts this.

## DOM structure & portal behavior

- Composition: `Root` → (`Trigger`, `Portal` → (`Backdrop`?, `Positioner` → `Popup` → (`Arrow` | `Viewport`)?)); all floating parts render through the portal `packages/react/src/preview-card/backdrop/PreviewCardBackdrop.test.tsx:22-27`
- `Portal` accepts a `container`; the positioner subtree lands inside the provided element `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:950-965`
- Without `keepMounted`, the positioner/popup are removed from the DOM when closed; with `keepMounted`, the positioner stays mounted and gets a `hidden` attribute while closed `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:806-809`
- The `Backdrop` gets inline `pointer-events: none` while open `packages/react/src/preview-card/backdrop/PreviewCardBackdrop.test.tsx:33-35`
- Default placement is below the trigger, start-aligned: with no `side`/`align` set, the positioner's x equals the trigger's left edge and y equals trigger bottom + `sideOffset` `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:112-130`
- `sideOffset` / `alignOffset` accept numbers or functions receiving a data object exposing `positioner.width`, `anchor.width`, and the resolved logical `side` / `align` `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:132-151`
- Requested `side`/`align` are flipped on collision and the flipped values are what offset callbacks read: `side="left"` resolves to `right`, `align="start"` (with `side="right"`) resolves to `end`, logical `side="inline-start"` resolves to `inline-end` `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:153-224`
- Without `Viewport`, the positioner is placed via inline `transform`; with a `Viewport` inside the popup, it switches to `top`/`left` positioning (`style.transform` empty) `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:342-379`
- Multiline inline triggers (inline element wrapping across lines) anchor to the hovered line's client rect, not the bounding box: y tracks the hovered line's bottom + `sideOffset`, x stays within the hovered line's horizontal span `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:381-438`
- With a delayed open on a multiline trigger, the positioner uses the latest hovered line at the moment the delay elapses `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:440-490`
- Page scroll does not break hovered-line alignment (tested after `window.scrollTo(0, 1000)`) `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:492-549`
- While open, the card stays anchored to the originally opened line: re-entering the trigger on another line plus a `resize` event triggers repositioning but the card remains on the first line's rect `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:551-610`
- Controlled `open` (no hover coordinates) anchors a multiline trigger to its LAST client rect; focus-open anchors to its FIRST client rect `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:687-732`
- Hovered-line coordinates are cleared after close, so a later controlled reopen of the same trigger falls back to the side-aligned rect; they are also cleared when the open was caused by focus (`reason === 'trigger-focus'`) `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:734-821`
- Stale hovered coordinates from a previous trigger are ignored when a controlled trigger switch reuses the popup — the card repositions to the new trigger's rect `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:823-908`
- `Positioner.anchor` (ref) overrides the trigger as the positioning anchor, combinable with `collisionBoundary` and `collisionPadding` in a clipped `keepMounted` portal container `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:910-974`

## Events (names, payload shape, bubbling, preventDefault semantics)

- No custom DOM events are emitted by any part (no `emit`/`onEvent` coverage) — bubbling semantics N/A.
- Open/close are driven by pointer interaction events: `mouseEnter`+`mouseMove` open, `mouseLeave` closes, `Escape` keydown closes; the close on hover-out is observed by dispatching `mouseLeave` on the positioner element (popup's parent) `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:1000-1021`
- `onOpenChange(nextOpen, eventDetails)` payload: `nextOpen` boolean; `eventDetails.cancel()` aborts the transition (prevents opening while uncontrolled) `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:421-441`
- `eventDetails.preventUnmountOnClose()` keeps the positioner mounted after close until `actionsRef.current.unmount()` is invoked `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:479-520`
- `eventDetails.reason` observed values: `'trigger-focus'` (focus-open on a multiline trigger), `REASONS.imperativeAction` (handle/`actionsRef.close()`), and `'none'` (close caused by the active trigger unmounting) `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:1036-1044`
- `eventDetails.trigger` is the trigger element (or null), carrying its `id` `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:786-792`
- Handle imperative API: `handle.open(triggerId)` opens anchored to that trigger (setting its payload) and marks it `data-popup-open`; `handle.close()` closes it `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:1696-1776`
- No test asserts `preventDefault` semantics on native DOM events inside PreviewCard — UNVERIFIED — inferred from absence across `packages/react/src/preview-card/trigger/PreviewCardTrigger.test.tsx`, no test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

- Reopen timing around the close lifecycle (detached triggers, real animations): re-hovering trigger A during its close transition reopens it immediately (no open delay); after the close lifecycle fully finishes, a fresh hover honors the open delay again `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:1552-1693`
- Entering trigger B while trigger A's close transition is running opens B's content immediately; once A's close transition has finished, B's own `delay` is respected `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:1414-1550`
- After switching triggers, the popup has no inline `scale` style that would override CSS transitions (`popup.style.scale === ''`) `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:1358-1412`
- With `keepMounted`, closing hides the positioner and reopening a different trigger repositions it to the new trigger `packages/react/src/preview-card/root/PreviewCardRoot.detached-triggers.test.tsx:1019-1073`
- Nested preview cards: clicking a nested trigger keeps the parent card open; a press that starts inside the nested popup and ends outside does not close the parent; hovering a nested trigger keeps the parent open `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:739-869`
- Nested race condition: leaving all previews partway through the close delay and re-entering the parent popup keeps the parent open while the child still closes; re-hovering the child trigger then reopens the child; moving into the child popup keeps both open through the close delay `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:874-957`
- Synchronized closing: when the pointer leaves both nested popups, the parent popup closes as soon as the child popup closes `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:960-1022`
- Viewport morphing with multiple triggers: during a trigger switch, a `[data-previous]` container (holding the old content, `inert`) coexists with a `[data-current]` container (new content) under a `[data-transitioning]` ancestor; both transition containers are cleaned up after the animation finishes `packages/react/src/preview-card/viewport/PreviewCardViewport.test.tsx:117-230`
- Rapid trigger switching (1→2→3→1) through morph transitions converges on the correct content without dropped states `packages/react/src/preview-card/viewport/PreviewCardViewport.test.tsx:232-299`
- The viewport marks the active container with `data-current`; switching the active trigger remounts the current container (new DOM node) `packages/react/src/preview-card/viewport/PreviewCardViewport.test.tsx:46-115`
- The viewport sets a space-separated `data-activation-direction` attribute describing travel between the previous and new trigger positions: `right`/`left` for horizontal, `up`/`down` for vertical, both for diagonal, and an empty value when both deltas are within ~5px tolerance `packages/react/src/preview-card/viewport/PreviewCardViewport.test.tsx:301-437`
- Window `resize` re-triggers positioning of an open card `packages/react/src/preview-card/positioner/PreviewCardPositioner.test.tsx:604-607`
- Unmount paths: `actionsRef.unmount()` removes the positioner even when `preventUnmountOnClose()` was used; closing normally removes the positioner from the DOM `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:479-548`

## Shared harness dependencies

- `#test-utils` → `packages/react/test/` (index `packages/react/test/index.ts`): `createRenderer` (`packages/react/test/createRenderer.ts`), `describeConformance` (`packages/react/test/describeConformance.tsx`), `popupConformanceTests` (`packages/react/test/popupConformanceTests.tsx`), `waitForPositioned` (`packages/react/test/waitForPositioned.ts`), `waitSingleFrame` (`packages/react/test/wait.ts`), `advanceReactClock` (`packages/react/test/advanceReactClock.ts`), `resetBrowserPointer` (`packages/react/test/resetBrowserPointer.ts`), `isJSDOM` flag.
- `@mui/internal-test-utils` (external npm package, not a repo file): `render`, `user`, `screen`, `fireEvent`, `waitFor`, `flushMicrotasks`, `act`, `ignoreActWarnings`, `randomStringValue`.
- `@base-ui/utils/useRefWithInit` used by the root test helpers to create stable handles `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:5`
- Environment globals used by tests: `globalThis.BASE_UI_ANIMATIONS_DISABLED` toggles animation behavior (set `true` by default in the root suites, `false` for animation/morph tests) `packages/react/src/preview-card/root/PreviewCardRoot.test.tsx:21-23`
