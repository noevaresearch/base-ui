# Select scroll arrows — behavior mined from tests

Scope: exactly these test files:

- `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx` (shared behavior tests exercising both arrows)
- `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx`
- `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx`

Every claim below is limited to what these tests assert. Anything not asserted is marked `UNVERIFIED`.

## Public API surface (props, parts, subcomponents)

- Parts exist as `Select.ScrollDownArrow` and `Select.ScrollUpArrow`, imported from `@base-ui/react/select` (`packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:2`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:2`).
- Both parts accept `keepMounted`: the conformance minimal element is `<Select.ScrollDownArrow keepMounted />` (`packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:9`) and `<Select.ScrollUpArrow keepMounted />` (`packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:9`), and all behavior tests pass `keepMounted` on every arrow (e.g. `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:76`, `:100`).
- With `keepMounted`, the arrow element stays in the DOM even when not visible: in the "hides the arrow" test the element is still queryable and is asserted to *not* have `data-visible` (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:397-406`).
- Arbitrary DOM props are forwarded to the arrow's root element. Conformance (run via `describeConformance`) proves forwarding of `lang`, `data-foobar`, and inline `style` to (a) the default element, (b) a `render` function, and (c) a `render` JSX element, for both arrows (`packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:9-18`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:9-18`; harness behavior: `packages/react/test/conformanceTests/propForwarding.tsx:23-95`).
- `data-testid` works on the arrows (used to query them, e.g. `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:297-305`).
- The ref resolves to a `window.HTMLDivElement` instance for both arrows, per `refInstanceof: window.HTMLDivElement` in the conformance options (`packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:10`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:10`; the harness asserts `expect(instance).toBeInstanceOf(refInstanceof)` in `packages/react/test/conformanceTests/refForwarding.tsx:32-38`).
- The rendered down arrow contains the literal text `▼` and the up arrow the literal text `▲`; tests locate them exclusively via `screen.getByText('▼')` / `screen.getByText('▲')` (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:107-108`, `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:85`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:85`, `:173`).
- No other props are asserted in these files (no `onClick`, no callback props, no `nativeButton`); any further API is UNVERIFIED — inferred from `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:1-441`, no test asserts it.

## State model (controlled/uncontrolled, defaults, transitions)

- N/A for controlled/uncontrolled state of the arrows themselves: no controlled/uncontrolled prop transitions are exercised in these three files.
- All tests render `Select.Root` with the `open` prop set (e.g. `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:71`, `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:13`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:13`); the tests never open/close the select, so open/close transitions are UNVERIFIED — inferred from `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:13`, no test asserts a transition.
- The only observed component state is the `data-visible` attribute on the arrow element:
  - Present when there is content to scroll in the arrow's direction: with an item-less popup-as-scroller at a mid offset (`scrollTop = 100` of a 400/200 scroller), the down arrow gains `data-visible` after a hover move (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:305-316`), and symmetrically the up arrow does too (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:352-361`).
  - Absent when already scrolled to the edge: at `scrollTop = 200` (exact max for 400/200), the down arrow does not have `data-visible` (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:397-406`).
  - These assertions are made after a `mousemove` plus a 40 ms timer advance; whether `data-visible` is computed on initial mount before any pointer interaction is UNVERIFIED — inferred from `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:308-316`, no test asserts it.

## Keyboard interactions

N/A — no keyboard events are dispatched or asserted in any of the three files (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:1-441`, `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:1-280`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:1-193`).

## Focus management

N/A — no focus is queried, moved, or asserted in any of the three files (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:1-441`, `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:1-280`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:1-193`).

## Accessibility (roles, aria-*, id linking)

N/A — no roles, `aria-*` attributes, or id links are asserted on the arrows in any of the three files (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:1-441`, `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:1-280`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:1-193`).

## DOM structure & portal behavior

- The arrows are rendered inside the select's portal tree. The canonical composition used by the behavior tests is `Select.Root open` > `Select.Trigger` > `Select.Portal` > `Select.Positioner alignItemWithTrigger={false}` > `Select.Popup` > (ScrollUpArrow, Select.List with items, ScrollDownArrow) (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:70-105`).
- The `alignItemWithTrigger={false}` prop is set on `Select.Positioner` in every behavior test (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:74`, `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:29`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:29`); what this prop does is UNVERIFIED — inferred from `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:29`, no test asserts its semantics.
- The conformance wrapper renders each arrow directly under `Select.Root open` > `Select.Positioner` with no `Select.Portal` and no `Select.Popup`, and the component still renders (`packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:11-17`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:11-17`).
- Each arrow is an `HTMLDivElement` (see Public API section above, `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:10`).
- Scroller resolution:
  - With `Select.List` rendered, the List's element is the scroller whose `scrollTop` the arrows write (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:77-99`, `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:32-53`).
  - With no `Select.List`, the arrows fall back to scrolling the `Select.Popup` element itself: a hover move on the down arrow increases the Popup's stubbed `scrollTop` (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:221-273`).
  - With neither List nor Popup, pointer interaction is a safe no-op (see Edge cases).

## Events (names, payload shape, bubbling, preventDefault semantics)

Only two DOM events are exercised, both dispatched directly on the arrow element:

- `mousemove` — starts the hover auto-scroll. The event init uses `movementX`/`movementY`:
  - A move with `movementX: 0, movementY: 0` (zero pointer movement) does NOT start scrolling; after 400 ms the scroller's `scrollTop` is still 0 (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:120-137`).
  - A nonzero `movementY` (e.g. `{ movementX: 0, movementY: 1 }` on the down arrow, `{ movementX: 0, movementY: -1 }` on the up arrow) starts scrolling in the arrow's direction (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:144`, `:169`, `:198`, `:355`; `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:91-94`; `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:91-94`).
  - Whether a nonzero `movementX` alone (no `movementY` change) starts scrolling is UNVERIFIED — inferred from `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:152` (a `{movementX: 1, movementY: 1}` move arrives while a scroll is already scheduled), no test asserts it in isolation.
- `mouseleave` — stops the auto-scroll loop. After `fireEvent.mouseLeave(downArrow)`, advancing 400 ms leaves `scrollTop` unchanged at the value reached before the leave (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:164-188`).
- Timing semantics proven by the fake-timer tests:
  - The first scroll step happens within 40 ms of the initiating move (tests advance exactly 40 ms and assert a step occurred) (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:169-176`, `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:96-100`).
  - A second `mousemove` arriving before the scheduled scroll fires does not postpone it: move at t=0, advance 30 ms, move again, advance 15 ms → `scrollTop > 0` by t=45 ms (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:139-162`).
  - After `mouseleave` no further steps occur within 400 ms (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:180-184`).
  - After the scroll edge is reached, the loop stops rather than writing no-op `scrollTop` values: the write counter does not change over a further 400 ms (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:206-215`).
- Bubbling: not asserted. N/A.
- `preventDefault` semantics: not asserted. N/A.
- Custom/Base UI events (named events with detail payloads): none are asserted in these files. Any such events are UNVERIFIED — inferred from `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:1-441`, no test asserts them.
- Click, wheel, touch, and drag interactions on the arrows: UNVERIFIED — inferred from `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:1-441`, no test asserts them.

Scroll targets proven (the observable effect of the events):

- Down arrow, fractional remaining space: with `scrollTop = 19.5`, `scrollHeight = 100.5`, `clientHeight = 60` (true max = 40.5), one hover step lands exactly on `40.5` — it snaps to the true bottom, not to an item boundary (`packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:20-104`).
- Down arrow, fractional item boundary: with `scrollTop = 71.81818389892578`, `scrollHeight = 598`, `clientHeight = 440`, items 32 px tall, one hover step lands exactly on `104` — it advances past a next item whose bottom is fractionally within the visible bottom (`packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:106-192`).
- Down arrow, trailing content: with `scrollTop = 390`, `scrollHeight = 600`, `clientHeight = 200`, and the last item bottom at 400, one hover step lands exactly on `400` (the true scroll bottom including content past the last item) (`packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:194-279`).
- Up arrow, fractional previous-item top: with `scrollTop = 72.18181610107422`, `scrollHeight = 598`, `clientHeight = 440`, items at offsets 32/71.5/110/142 (32 px tall), one hover step lands exactly on `32` — it keeps advancing when the previous item top is fractionally within the visible top (`packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:20-104`).
- Up arrow, no earlier item: with `scrollTop = 100` and all items at offsets 300/340/380 (below the viewport top), one hover step lands exactly on `0` (`packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:106-192`).
- Sub-pixel top edge: with `initialScrollTop = 0.4`, hovering the up arrow normalizes `scrollTop` to exactly `0` and then stops (no further writes) (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:190-219`).

## Edge cases (rapid interactions, unmount, nesting)

- Rapid pointer movement: a second `mousemove` before the scheduled scroll fires does not restart or postpone the scroll timer — the scroll still happens by 45 ms total (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:139-162`).
- Stationary cursor over the arrow (zero-movement `mousemove`, as browsers dispatch when content scrolls beneath a fixed cursor) does not start auto-scrolling, even after 400 ms (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:120-137`).
- Pointer leaving the arrow terminates the loop; no steps occur afterwards (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:164-188`).
- Reaching the scroll edge cancels the loop instead of spinning on no-op `scrollTop` writes — the write count is stable over 400 ms (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:206-215`).
- Scroller missing entirely (arrow under `Select.Positioner` with no `Select.Popup`/`Select.List`): a `mousemove` plus 400 ms of timer advancement must not throw; the arrow bails out instead of dereferencing a missing scroller (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:412-440`).
- Scroller with no registered `Select.Item`s: scrollability reflection still works mid-scroll (`data-visible` present on both arrows at `scrollTop = 100`; `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:275-365`) and correctly hides at the exact bottom edge (`data-visible` absent at `scrollTop = 200` of 400/200; `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:367-410`).
- Popup-as-scroller fallback (no `Select.List` rendered) still scrolls on hover (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:221-273`).
- Fractional geometry throughout: fractional `scrollTop`, `scrollHeight`, and `offsetTop` values all produce exact integer or exact decimal landing offsets (see Events section; `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:20-104`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:20-104`).
- Unmount behavior: UNVERIFIED — inferred from `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:1-441`, no test asserts unmount.
- Nesting (select inside select, arrow inside arrow): UNVERIFIED — inferred from `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:1-441`, no test asserts it.

## Shared harness dependencies

The tests import `createRenderer` and `describeConformance` from `#test-utils`, which resolves to `packages/react/test/index.ts`:

- `packages/react/test/index.ts:1-14` — barrel re-export. It re-exports `@base-ui/utils/testUtils` (which provides `isJSDOM`, `packages/utils/src/testUtils.ts:4`), and named exports `advanceReactClock`, `createRenderer`, `describeConformance`, pointer helpers (`enterWithMouse`, `firePointer`, `moveMouse`), `mergeRefs`, `popupConformanceTests`, `resetBrowserPointer`, `useTestInteractions`, `./wait`, `waitForPositioned`, and `describeGregorianAdapter`. The three scroll-arrow test files only use `createRenderer` and `describeConformance` from this barrel (`packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:4`, `packages/react/src/select/scroll-up-arrow/SelectScrollUpArrow.test.tsx:4`, `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:4`).

Harness files read and relevant to these tests:

- `packages/react/test/createRenderer.ts:27-49` — `createRenderer(globalOptions?)` wraps the shared `@mui/internal-test-utils` renderer. Its `render` runs `originalRender` inside `act(async …)` and returns the result augmented with `rerender` (also act-wrapped) and `setProps` (clones the element with new props and rerenders) (`packages/react/test/createRenderer.ts:31-43`). The arrow tests only destructure `{ render }` from it (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:46`, `packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:7`).
- `packages/react/test/describeConformance.tsx:51-70` — `describeConformance(minimalElement, getOptions)` describes a "Base UI component API" suite; it filters the full suite (`propsSpread` → `testPropForwarding`, `refForwarding` → `testRefForwarding`, `renderProp`, `className`) by the `only`/`skip` options and registers the `after` hook via `afterAll` (`packages/react/test/describeConformance.tsx:44-68`). Because the arrow tests pass no `only`/`skip`, the full suite runs. Both arrow conformance blocks supply only `refInstanceof: window.HTMLDivElement` and a custom `render` that mounts the node inside `<Select.Root open><Select.Positioner>…</Select.Positioner></Select.Root>` (`packages/react/src/select/scroll-down-arrow/SelectScrollDownArrow.test.tsx:9-18`).
- `packages/react/test/conformanceTests/refForwarding.tsx:27-39` — `testRefForwarding` attaches a `React.createRef()` to a clone of the minimal element, renders it through the provided `render`, and asserts `expect(instance).toBeInstanceOf(refInstanceof)` (`packages/react/test/conformanceTests/refForwarding.tsx:9-38`). This is the actual assertion behind the "ref resolves to HTMLDivElement" claim.
- `packages/react/test/conformanceTests/propForwarding.tsx:10-128` — `testPropForwarding` renders the minimal element (via the provided `render`, defaulting `testRenderPropWith` to `'div'`) and asserts: arbitrary props (`lang`, `data-foobar`) land on the default element (`:23-36`), on a `render` function custom element (`:38-59`), and on a `render` JSX custom element (`:61-80`); and inline `style` (e.g. `color: green`) survives on the custom root in all three forms (`:82-127`). Each render is followed by `flushMicrotasks()` before querying (`packages/react/test/conformanceTests/propForwarding.tsx:31`).
- `packages/react/test/wait.ts:4-18` — exports `wait(ms)` (setTimeout-based promise) and `waitSingleFrame()` (requestAnimationFrame-based promise). Neither is called by the three scroll-arrow test files; they are harness surface only.
- `packages/utils/src/testUtils.ts:4` — `isJSDOM` (`/jsdom/.test(window.navigator.userAgent)`), re-exported through `packages/react/test/index.ts:1`. Not used by the three scroll-arrow test files directly (they run in both environments via `createRenderer` and stub geometry instead, per the comment at `packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:7-9`).

External harness pieces the tests use directly but which live outside `packages/react/test/` (described only as used by the tests, not read): `act`, `fireEvent`, `screen` from `@mui/internal-test-utils` (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:3`), plus Vitest's `vi.useFakeTimers` / `vi.advanceTimersByTime` / `vi.useRealTimers` used to drive the hover-scroll scheduling (`packages/react/src/select/scroll-arrow/SelectScrollArrow.test.tsx:121-135`).
