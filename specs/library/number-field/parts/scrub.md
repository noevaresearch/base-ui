# NumberField scrub parts — behavior spec (mined from tests)

Covers `NumberField.ScrubArea` (including touch/pointer guards, sensitivity, teleport, direction, click passthrough) and `NumberField.ScrubAreaCursor`. Sources: `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx`, `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.gecko.test.tsx`, `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx`.

## Public API surface (props, parts, subcomponents)

- `NumberField.ScrubArea` renders a `span`: conformance asserts the forwarded ref resolves to `HTMLSpanElement`, mounted inside a `NumberField.Root` wrapper `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:63-68`.
- `NumberField.ScrubAreaCursor` also renders a `span`; its conformance suite wraps it in `NumberField.Root > NumberField.ScrubArea` with a manually supplied `NumberFieldScrubAreaContext.Provider` whose value is `{ isScrubbing: true, isTouchInput: false, isPointerLockDenied: false, scrubAreaCursorRef }` `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:11-16`, `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:22-35`.
- ScrubArea props exercised by tests: `pixelSensitivity` (number) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:459-548`, `teleportDistance` (number) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:551-598`, `direction` (`'horizontal' | 'vertical'`) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:600-661`, and `onClick` (plus arbitrary DOM props like `data-testid`, `style`) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:663-677`, `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:563-569`.
- Root props exercised in scrub tests: `defaultValue`, `readOnly`, `step`, `snapOnStep`, `onValueChange`, `onValueCommitted` `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:140-148`, `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:342-378`, `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:465-470`.
- Conformance suite (run for both parts via `describeConformance`) proves: props spread (`lang`, `data-*`, `style`) onto the default element and onto `render` function/element output; `render` accepts a function or JSX element with ref and className merging; className string applies; ref attaches with `instanceof HTMLSpanElement` `packages/react/test/describeConformance.tsx:44-49`, `packages/react/test/conformanceTests/propForwarding.tsx:23-95`, `packages/react/test/conformanceTests/renderProp.tsx:41-144`, `packages/react/test/conformanceTests/className.tsx:20-23`, `packages/react/test/conformanceTests/refForwarding.tsx:27-39`.
- `NumberFieldScrubAreaContext` is the shared context consumed by the cursor; rendering `NumberField.ScrubAreaCursor` outside `NumberField.ScrubArea` rejects with the error `'Base UI: NumberFieldScrubAreaContext is missing. NumberFieldScrubArea parts must be placed within <NumberField.ScrubArea>.'` `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:46-62`.

## State model (controlled/uncontrolled, defaults, transitions)

- The `NumberField.Root` element carries a `data-scrubbing` attribute while a scrub is active: absent initially, present after pointerdown+pointermove, removed on pointerup `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:137`, `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:295-300`, `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:256-260`.
- During scrubbing, `onValueChange` fires one or more times (per accumulated pixel movement); on pointerup, `onValueCommitted` fires exactly once with the last `onValueChange` value `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:342-378`.
- Scrubbing never starts when the field is `readOnly`: pointerdown leaves no `data-scrubbing` `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:140-148`.
- If the `ScrubArea` unmounts mid-scrub (before pointerup), the root's scrubbing state is cleared — the root does not stay stuck with `data-scrubbing` `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:276-308`.
- React `<Activity>` hide/show does not leave scrubbing live: hiding mid-scrub tears effects down; after revealing, a bare `pointermove` (no pointer down) does not change the value and the input keeps its previous value `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:413-457`.
- Value updates are cumulative across the whole drag, including direction reversals: movements of -10, +5, then -2 produce input values `-10`, `-5`, `-7` in sequence `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:177-207`.
- Scrubbing after a paste syncs from the pasted value: pasting `20` (with `onValueChange` receiving 20) then scrubbing +2px yields 22 in both the input and the last `onValueChange` call `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:310-340`.

## Keyboard interactions

N/A — none of the three test files exercise keyboard behavior on the scrub parts.

## Focus management

- A mouse `pointerdown` on the scrub area focuses the `NumberField.Input`, and a selection the consumer establishes inside `onFocus` (via `event.currentTarget.select()`) survives: after pointerdown, the input has focus with `selectionStart === 0` and `selectionEnd === input.value.length` `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:150-169`.

## Accessibility (roles, aria-*, id linking)

- `NumberField.ScrubArea` has `role="presentation"` (queried via `screen.queryByRole('presentation')`) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:70-77`.
- The cursor file repeats a "has presentation role" check, but it renders only `<NumberField.ScrubArea />` there, so that test re-proves the ScrubArea role, not the cursor's own role attribute `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:37-44`. A direct assertion that the cursor element itself carries `role="presentation"`: UNVERIFIED — inferred from `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:37-44`, no test asserts this.
- No `aria-*` attributes or id linking (e.g. labelledby) are asserted anywhere in these three files.

## DOM structure & portal behavior

- `ScrubArea` mounts as a plain `span` child of `NumberField.Root`; the test fixtures nest `NumberField.Input`, a label, and/or `NumberField.ScrubAreaCursor` inside it `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:117-123`, `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:683-690`.
- The cursor element only exists while scrubbing with a mouse and after pointer lock is granted:
  - With `Element.prototype.requestPointerLock` stubbed to resolve, pressing the mouse on the scrub area renders the cursor (found by testid after a short wait) `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:64-92`.
  - The cursor is unmounted on pointerup: after dispatching window `pointerup`, `screen.queryByTestId('cursor')` is `null` `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:256-261`.
  - With touch input, no cursor renders at all `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:127-146`.
  - If `requestPointerLock` throws (lock denied), the cursor never renders and the stub was called `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:148-180`.
  - If pointer lock resolves late (after the user already tapped and released), the cursor does not remain rendered `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:182-220`.
  - Real headless Chromium denies pointer lock and would unmount the virtual cursor mid-test, so tests stub `document.body.requestPointerLock` to `Promise.resolve()` to keep it alive `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:553-556`, `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:219-221`.
- Only one cursor renders even with multiple scrub areas: pressing on scrub-area-1 leaves exactly one `scrub-area-cursor` testid in the document `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:94-125`.
- The cursor is styled via inline `style.transform`:
  - It compensates for visual viewport zoom: with `visualViewport.scale === 2` its transform contains `scale(0.5)`; after scale becomes 4 (observed on the next window `pointermove`), the transform updates to `scale(0.25)` `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:236-247`.
  - With `teleportDistance={100}` on a fixed-position 20×20 scrub area, moving +5000px wraps the virtual cursor to the left bound (`transform` contains `translate3d(50px,`), and moving -5000px wraps to the right bound (`translate3d(170px,`) — i.e. the virtual cursor teleports around the allowed bounds symmetric around the scrub area's center/left offset `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:551-598`.
- The bare cursor element has `offsetWidth === 0` (no intrinsic size) so its wrap coordinates equal the bounds themselves `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:580-581`.
- No portal behavior is asserted for either part (no portal-related conformance or test): UNVERIFIED — inferred from `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:63-68`, no test asserts this.

## Events (names, payload shape, bubbling, preventDefault semantics)

- Tests synthesize real `PointerEvent`s. The `pointerdown` helper targets the element's rect center and the `pointermove` helper accumulates `clientX`/`clientY` and sets `movementX`/`movementY` on a bubbling event `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:12-36`.
- `pointerdown` guard: non-primary buttons (`button: 1`) are ignored — no scrubbing starts `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:130-138`.
- Touch handling on `touchstart`:
  - A single-touch scrub is cancelable-and-canceled: `fireEvent.touchStart` returns `false` (the event was prevented) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:97-103`.
  - Multi-touch gestures (two touches, e.g. pinch-zoom) are left to the browser: the event is not canceled (`fireEvent.touchStart` returns `true`) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:105-111`.
  - Touch objects are built with `new Touch({ identifier, target, clientX, clientY })` when `Touch` is available, falling back to a plain `{ clientX, clientY }` object otherwise `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:80-85`.
- Listeners: while scrubbing, `pointermove` and `pointerup` handlers are attached to `window` with capture (`addEventListener` observed with those types, and on scrub end `removeEventListener` is called with the same handler and `capture = true` as the third argument) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:249-267`.
- Movement after the scrub ends is ignored: dispatching `pointerup` then a further `movementX: 10` leaves the value unchanged at `10` (test skipped on Gecko, see below) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:382-411`.
- Click passthrough: `onClick` on `ScrubArea` fires when the area is clicked without scrubbing `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:663-677`; clicks on child elements (a `<label>`) fire both the child's handler and bubble to the ScrubArea's `onClick` `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:679-697`.
- Gecko-specific (gecko-gated file, `vi.mock` forces `platform.engine.gecko = true`; skipped in jsdom/WebKit): after `pointerup`, the scrub is still live — `onValueCommitted` has NOT fired and `data-scrubbing` is still present — because Firefox needs the pointer lock to outlive the release; after advancing fake timers by 20ms, `onValueCommitted` fires exactly once with the final scrubbed value (10) and `data-scrubbing` is removed `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.gecko.test.tsx:7-18`, `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.gecko.test.tsx:26-72`. Correspondingly, the "ignores movement after end" test in the main file is `it.skipIf(platform.engine.gecko)` with a comment explaining Gecko defers the pointer lock release by 20ms `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:380-382`.

## Edge cases (rapid interactions, unmount, nesting)

- Rapid direction reversals accumulate correctly: -10 → +5 → -2 yields -10 → -5 → -7 `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:190-207`.
- `pixelSensitivity` gating and sub-threshold accumulation (with `pixelSensitivity={5}`): moves smaller than the threshold produce no change (multiple ±2 moves keep value `0`); four consecutive 1px moves still produce `0`; the 5th 1px move produces `1`; a single 5px move adds +1 (→ `6`); a -4px move is ignored (→ `6`); a -1px move subtracts (→ `5`); a +5px move adds +5 (→ `10`) — movement accumulates until |accumulated| ≥ sensitivity, then the whole accumulated amount applies `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:484-548`.
- `pixelSensitivity={0}` with a zero-distance move does not change the value and does not call `onValueChange`; the fixture uses `step={1} snapOnStep` on `defaultValue={3.5}` so a zero-amount step would observably snap 3.5 → 3 if it fired `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:460-482`.
- `direction="vertical"`: horizontal movement (`movementX: 10`) is ignored (value stays `0`); upward movement (negative `movementY: -10`) increases the value to `10`; downward `movementY: 4` decreases to `6` `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:601-633`.
- `direction="horizontal"`: horizontal `movementX: 10` scrubs to `10`; a following vertical `movementY: 10` is ignored (value stays `10`) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:635-660`.
- Unmount mid-scrub clears root scrubbing state (no stuck `data-scrubbing`) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:276-308`.
- `<Activity mode="hidden">` mid-scrub then reveal: no resumed scrubbing from a bare subsequent `pointermove` (no pointer down) `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:446-456`.
- Nesting two `ScrubArea`s in one root is supported and only the actively scrubbed area renders its cursor `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:100-121`.
- Rapid tap (press+release) with a pointer-lock promise that resolves 30ms later: cursor is not rendered/remaining afterwards `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:204-217`.

## Platform gating summary

- The main scrub-area file skips all pointer-lock/scrubbing-mechanics tests in jsdom and WebKit via an early `return` when `isJSDOM || isWebKit` (only conformance, role, touch-guard, and pointerdown-guard tests run there); comment: "Only run the following tests in Chromium/Firefox" `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:172-175`.
- The cursor file is entirely skipped on WebKit ("This component doesn't render on WebKit") `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:18-19`.
- The gecko file uses `clock.withFakeTimers()` and is `describe.skipIf(isJSDOM || platform.engine.webkit)` `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.gecko.test.tsx:21-24`.

## Shared harness dependencies

- `packages/react/test/index.ts` — the `#test-utils` barrel; re-exports `createRenderer`, `describeConformance`, `isJSDOM`, pointer helpers (`firePointer`, `enterWithMouse`, `moveMouse`), and wait utils used across the repo `packages/react/test/index.ts:1-14`.
- `packages/react/test/createRenderer.ts` — wraps `render` in `act`, and adds `rerender`/`setProps` helpers (used by the unmount/Activity tests via `setProps`) `packages/react/test/createRenderer.ts:31-43`.
- `packages/react/test/describeConformance.tsx` — runs the "Base UI component API" suite; full suite = `propsSpread`, `refForwarding`, `renderProp`, `className`, filtered by `only`/`skip` `packages/react/test/describeConformance.tsx:44-68`.
- `packages/react/test/conformanceTests/refForwarding.tsx` — proves the ref attaches and is `instanceof` the configured `refInstanceof` (`HTMLSpanElement` for both parts) `packages/react/test/conformanceTests/refForwarding.tsx:32-38`.
- `packages/react/test/conformanceTests/propForwarding.tsx` — proves arbitrary props (`lang`, `data-*`, `style`) reach both the default element and the `render` function/element output `packages/react/test/conformanceTests/propForwarding.tsx:23-127`.
- `packages/react/test/conformanceTests/renderProp.tsx` — proves `render` as function or element works, refs merge, and classNames merge between component and render-prop element `packages/react/test/conformanceTests/renderProp.tsx:41-178`.
- `packages/react/test/conformanceTests/className.tsx` — proves a string `className` is applied to the rendered element `packages/react/test/conformanceTests/className.tsx:20-23`.
- `packages/utils/src/testUtils.ts` — source of `isJSDOM` (`/jsdom/.test(window.navigator.userAgent)`) used for platform gating `packages/utils/src/testUtils.ts:4`.
- `@mui/internal-test-utils` (external) — supplies `screen`, `act`, `fireEvent`, `reactMajor`, and `user`/`user.pointer` used for click and pointer simulation `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:3`, `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:3`.
