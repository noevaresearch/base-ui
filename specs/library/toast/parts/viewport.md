# Toast.Viewport behavior spec (batch: viewport)

Evidence base: the Toast.Viewport test file (1121 lines, read in full) plus the harness/aux files listed under Shared harness dependencies. Every claim ends with a citation to the lines that assert it; behavior only implied by comments or test setup is marked UNVERIFIED. Below, "viewport" means the Toast.Viewport element and "toast root" means the Toast.Root element rendered by the shared List helper.

## Public API surface (props, parts, subcomponents)

- Toast.Viewport is exported from the Toast namespace imported from @base-ui/react/toast. `packages/react/src/toast/viewport/ToastViewport.test.tsx:4`
- It renders a div: the conformance options assert the forwarded ref is an instance of window.HTMLDivElement. `packages/react/src/toast/viewport/ToastViewport.test.tsx:12-17`
- It must be mounted inside Toast.Provider: the conformance render wrapper always supplies a Provider, and mounting the Viewport alone rejects with "Base UI: useToastManager must be used within <Toast.Provider>." `packages/react/src/toast/viewport/ToastViewport.test.tsx:14-16` `packages/react/src/toast/viewport/ToastViewport.test.tsx:141-151`
- Toasts are passed as children of the Viewport; the shared List helper maps each toast in the manager to a Toast.Root (containing Toast.Title, Toast.Description, Toast.Close, Toast.Action) rendered directly under the Viewport. `packages/react/src/toast/viewport/ToastViewport.test.tsx:22-24` `packages/react/src/toast/utils/test-utils.tsx:30-38`
- The conformance suite runs with no skips, so it proves: custom DOM props (lang, data-*, style) forward to the default element and to render-prop function/element variants; ref forwarding; the render prop in both function and element form with ref and className merging; and string className application. `packages/react/src/toast/viewport/ToastViewport.test.tsx:12-17` `packages/react/test/describeConformance.tsx:44-49`
- Provider/manager APIs exercised in these tests: Toast.useToastManager (add and close) `packages/react/src/toast/viewport/ToastViewport.test.tsx:57-69`; Toast.createToastManager `packages/react/src/toast/viewport/ToastViewport.test.tsx:1024`; Provider props timeout (set to 0) `packages/react/src/toast/viewport/ToastViewport.test.tsx:73` `packages/react/src/toast/viewport/ToastViewport.test.tsx:1027`, limit (set to 0) `packages/react/src/toast/viewport/ToastViewport.test.tsx:1000`, and toastManager `packages/react/src/toast/viewport/ToastViewport.test.tsx:1027`.

## State model (controlled/uncontrolled, defaults, transitions)

- Expansion is reflected by a data-expanded attribute on the viewport element: added on mouseenter of a toast root, removed on mouseleave. `packages/react/src/toast/viewport/ToastViewport.test.tsx:253-257`
- mouseenter on the viewport element itself also sets data-expanded. `packages/react/src/toast/viewport/ToastViewport.test.tsx:917-918`
- Keyboard focus inside the viewport keeps data-expanded across mouseleave. `packages/react/src/toast/viewport/ToastViewport.test.tsx:278-281`
- Touch swipe: pointerdown + pointermove with pointerType touch on a toast root expands the viewport and keeps it expanded even when mouseleave fires; pointerup ends the swipe and collapses it. `packages/react/src/toast/viewport/ToastViewport.test.tsx:309-339`
- pointercancel during a swipe without a preceding mouseleave: the toast root loses data-swiping but the viewport stays expanded. `packages/react/src/toast/viewport/ToastViewport.test.tsx:371-391`
- pointercancel after mouseleave already fired: swiping ends and the viewport collapses. `packages/react/src/toast/viewport/ToastViewport.test.tsx:431-441`
- A touch pointerdown outside the viewport collapses it and resumes the dismiss timers (toast removed after a later 5001ms tick); a touch pointerdown on the viewport itself keeps it expanded with timers still paused; a mouse pointerdown outside is ignored (stays expanded, timers still paused). `packages/react/src/toast/viewport/ToastViewport.test.tsx:627-632` `packages/react/src/toast/viewport/ToastViewport.test.tsx:651-656` `packages/react/src/toast/viewport/ToastViewport.test.tsx:676-681`
- Window/document listener binding tracks store emptiness: after the first toast is added, window keydown/blur/focus and document pointerdown are each registered exactly once; when the last toast closes they are each removed exactly once; re-adding registers them again (counts reach 2). `packages/react/src/toast/viewport/ToastViewport.test.tsx:92-130`
- Dismiss-timer pause state (fake timers): paused while a toast is hovered (a 5001ms tick leaves the toast mounted) `packages/react/src/toast/viewport/ToastViewport.test.tsx:461-466`; paused while the viewport is focused via F6 `packages/react/src/toast/viewport/ToastViewport.test.tsx:509-514`; resumes after mouseleave (toast removed between 4999 and 5001 ms later) `packages/react/src/toast/viewport/ToastViewport.test.tsx:484-494`; resumes when focus leaves the viewport (see Focus management).
- The dismiss countdown preserves elapsed time across window blur: active 1000ms, blurred for at least 5000ms, then on window focus it expires between 3999 and 4001 ms later (present at tick 3999, gone after 2 more ms). `packages/react/src/toast/viewport/ToastViewport.test.tsx:751-771`
- All of this state is internal; no test in the file passes expansion-, pause-, or focus-related props to the Viewport. `packages/react/src/toast/viewport/ToastViewport.test.tsx:9-1121`

## Keyboard interactions

- F6 focuses the viewport element when a toast is on screen. `packages/react/src/toast/viewport/ToastViewport.test.tsx:163-168`
- F6 is handled by a window-level keydown listener: a keydown dispatched directly on the owner window (no target element) focuses the viewport, including when the viewport is portaled into another document (iframe). `packages/react/src/toast/viewport/ToastViewport.test.tsx:99-104`
- F6 also focuses the viewport when no toast element is rendered (Provider limit set to 0). `packages/react/src/toast/viewport/ToastViewport.test.tsx:1008-1015`
- Pressing F6 pauses the dismiss timers (toast survives a 5001ms tick). `packages/react/src/toast/viewport/ToastViewport.test.tsx:509-514`
- Tab from the focused viewport moves focus to the first toast root. `packages/react/src/toast/viewport/ToastViewport.test.tsx:183-187`
- shift+Tab on the first toast returns focus to the element focused before entering the viewport (the trigger button). `packages/react/src/toast/viewport/ToastViewport.test.tsx:203-207`
- shift+Tab from the focused viewport restores focus to the pre-F6 element and resumes timers (toast removed after the next 5001ms tick). `packages/react/src/toast/viewport/ToastViewport.test.tsx:529-544`
- If the pre-F6 element lived inside the viewport (a toast close button), shift+Tab returns focus there and the timers stay paused (toast survives the 5001ms tick). `packages/react/src/toast/viewport/ToastViewport.test.tsx:562-577`
- Forward Tab (no shift) from the focused viewport does not restore focus to the previous element, and the timers stay paused. `packages/react/src/toast/viewport/ToastViewport.test.tsx:594-603`
- With two toasts, tabbing after F6 walks: first toast root, first close button, first action button, last toast root, last close button, last action button; one more Tab lands on the trigger button that follows the viewport. (Despite the test title, only forward Tab presses occur; the shift+Tab variants are covered by the tests above.) `packages/react/src/toast/viewport/ToastViewport.test.tsx:225-234`

## Focus management

- Pressing F6 renders focus guard element(s) carrying data-base-ui-focus-guard; the test then fires focus on a guard with relatedTarget set to the viewport element to simulate tabbing onward. `packages/react/src/toast/viewport/ToastViewport.test.tsx:987-992`
- Guard focus lands on the first toast that is not animating out: with the newest toast closed but still mounted mid-exit-animation, the older surviving toast receives focus and the animating-out one does not. `packages/react/src/toast/viewport/ToastViewport.test.tsx:971-995`
- If no toast can receive focus, guard focus returns focus to the pre-viewport element. `packages/react/src/toast/viewport/ToastViewport.test.tsx:1012-1020`
- Closing every toast while a toast is focused hands focus back to the pre-viewport trigger. `packages/react/src/toast/viewport/ToastViewport.test.tsx:1044-1049`
- When the focused toast and another toast are closed together, focus skips the animating-out toast (which carries data-ending-style) and lands on the next surviving toast. `packages/react/src/toast/viewport/ToastViewport.test.tsx:1085-1095`
- Closing toasts while focus has never entered the viewport does not move focus. `packages/react/src/toast/viewport/ToastViewport.test.tsx:1110-1118`
- Moving focus out of the viewport (calling focus() on an outside element) resumes the dismiss timers: combined with the F6 pause test, the toast being removed within the following 5001ms tick proves the countdown resumed rather than staying paused. `packages/react/src/toast/viewport/ToastViewport.test.tsx:497-515` `packages/react/src/toast/viewport/ToastViewport.test.tsx:697-705`
- While the window is blurred, viewport blur does not resume timers (toast survives a 10000ms tick). `packages/react/src/toast/viewport/ToastViewport.test.tsx:849-865`

## Accessibility (roles, aria-*, id linking)

- No test asserts a role, aria-live, aria-labelledby, or any other aria attribute on the viewport element itself; the viewport's landmark semantics are UNVERIFIED — inferred from `packages/react/src/toast/viewport/ToastViewport.test.tsx:9-1121`, no test asserts this.
- The only aria-adjacent behavior appears in a test comment: the toast close button is described as aria-hidden until the viewport is expanded, which is why the test queries the aria-label attribute selector with querySelectorAll instead of getByRole. UNVERIFIED — inferred from `packages/react/src/toast/viewport/ToastViewport.test.tsx:979-984`, no test asserts the aria-hidden state itself.
- The close button rendered by the shared helper carries aria-label "close-press" (test setup, not component behavior). `packages/react/src/toast/utils/test-utils.tsx:35`
- The a11y-facing keyboard behaviors proven are F6 access to the viewport and the focus-guard tab flow (see Keyboard interactions / Focus management). `packages/react/src/toast/viewport/ToastViewport.test.tsx:153-169` `packages/react/src/toast/viewport/ToastViewport.test.tsx:987-992`

## DOM structure & portal behavior

- The viewport element is an HTMLDivElement. `packages/react/src/toast/viewport/ToastViewport.test.tsx:13`
- Toast roots render inside the viewport as children and are ordered newest-first in the DOM: after adding oldest, middle, newest, getAllByTestId('root') yields newest, middle, oldest (ordering asserted via toHaveTextContent on the second entry). `packages/react/src/toast/viewport/ToastViewport.test.tsx:1077-1078`
- The viewport can be rendered through ReactDOM.createPortal into a container belonging to another document (an iframe); in that case its window and document listeners attach to that owner window/document (verified by spying on the iframe window/document addEventListener). `packages/react/src/toast/viewport/ToastViewport.test.tsx:72-97`
- In a real browser (test is skipped in jsdom), after a toast is added the viewport element's inline style exposes a non-empty --toast-frontmost-height CSS variable, i.e. the measured frontmost toast height is written onto the viewport. `packages/react/src/toast/viewport/ToastViewport.test.tsx:19-35`
- Attributes observed: viewport data-expanded; toast root data-swiping during a touch swipe (removed on pointercancel); toast root data-ending-style while a closing toast waits for its exit animation (it stays mounted until the animation finishes, then is removed). `packages/react/src/toast/viewport/ToastViewport.test.tsx:254` `packages/react/src/toast/viewport/ToastViewport.test.tsx:379-390` `packages/react/src/toast/viewport/ToastViewport.test.tsx:920-921` `packages/react/src/toast/viewport/ToastViewport.test.tsx:940-946`
- Focus guards render with the data-base-ui-focus-guard attribute after F6. `packages/react/src/toast/viewport/ToastViewport.test.tsx:991`

## Events (names, payload shape, bubbling, preventDefault semantics)

- Listener inventory on the owner window: keydown, blur, focus; on the owner document: pointerdown. Each is added and removed exactly once per non-empty store cycle. `packages/react/src/toast/viewport/ToastViewport.test.tsx:92-97` `packages/react/src/toast/viewport/ToastViewport.test.tsx:110-119`
- The window blur and focus listeners are registered with capture enabled: the tests identify them by spying on addEventListener and matching the event type plus a third argument of true. `packages/react/src/toast/viewport/ToastViewport.test.tsx:725-730`
- mouseenter/mouseleave on a toast root (or on the viewport) toggle data-expanded, subject to the focus-visible and swipe rules in State model. `packages/react/src/toast/viewport/ToastViewport.test.tsx:253-257` `packages/react/src/toast/viewport/ToastViewport.test.tsx:917-918`
- Touch swipe handling consumes pointerdown, pointermove, pointerup and pointercancel with pointerType touch fired on the toast root (events dispatched with bubbles true). `packages/react/src/toast/viewport/ToastViewport.test.tsx:309-338` `packages/react/src/toast/viewport/ToastViewport.test.tsx:363-391`
- A document-level pointerdown with pointerType touch outside the viewport collapses the viewport and resumes timers; the same event on the viewport keeps it expanded; pointerType mouse outside is ignored. `packages/react/src/toast/viewport/ToastViewport.test.tsx:627-632` `packages/react/src/toast/viewport/ToastViewport.test.tsx:651-653` `packages/react/src/toast/viewport/ToastViewport.test.tsx:674-681`
- Window blur pauses the dismiss timers and window focus resumes them; the tests dispatch synthetic FocusEvents whose composedPath returns the window, indicating window-level blur/focus is classified through the event's composed path. `packages/react/src/toast/viewport/ToastViewport.test.tsx:741-771`
- While the window is blurred, mouseleave `packages/react/src/toast/viewport/ToastViewport.test.tsx:816-819` and viewport blur `packages/react/src/toast/viewport/ToastViewport.test.tsx:861-865` do not resume timers.
- No custom events, event payload shapes, bubbling guarantees, or preventDefault semantics are asserted anywhere in the file. `packages/react/src/toast/viewport/ToastViewport.test.tsx:9-1121`

## Edge cases (rapid interactions, unmount, nesting)

- Add, close-all, re-add cycles rebind owner listeners exactly once per cycle with no duplicates accumulating. `packages/react/src/toast/viewport/ToastViewport.test.tsx:87-130`
- Cross-realm nesting: the viewport portaled into an iframe document stays fully functional — F6 dispatched on the iframe window focuses it, and listener add/remove counts are observed in the iframe realm. `packages/react/src/toast/viewport/ToastViewport.test.tsx:72-104` `packages/react/src/toast/viewport/ToastViewport.test.tsx:110-130`
- Pointer-cancel ordering matters: cancel without leave keeps the viewport expanded; cancel after leave collapses it. `packages/react/src/toast/viewport/ToastViewport.test.tsx:342-392` `packages/react/src/toast/viewport/ToastViewport.test.tsx:394-442`
- Window blur while hovering: a following mouseleave must not resume the timer (the comment guards against off-screen expiry; the toast survives a 10000ms tick). `packages/react/src/toast/viewport/ToastViewport.test.tsx:805-819`
- Deferred collapse during exit animation: hovering the viewport, closing the newest toast (which enters data-ending-style), then mouseleave and window blur — the collapse is applied only after the closing toast finishes its exit animation and unmounts, leaving exactly one toast and no data-expanded. The test forces real animation timing by flipping globalThis.BASE_UI_ANIMATIONS_DISABLED to false and resolving a stubbed getAnimations promise. `packages/react/src/toast/viewport/ToastViewport.test.tsx:905-947` `packages/react/src/toast/viewport/ToastViewport.test.tsx:873-874`
- The touch-swipe tests stub setPointerCapture and releasePointerCapture on the toast root with no-ops, implying the swipe captures the pointer on the toast root. UNVERIFIED — inferred from `packages/react/src/toast/viewport/ToastViewport.test.tsx:300-307`, no test asserts this directly.
- Not covered by any test in this file: window resize behavior, the default timeout value, viewport role/aria attributes, event preventDefault semantics, and custom viewport events or payloads. `packages/react/src/toast/viewport/ToastViewport.test.tsx:9-1121`

## Shared harness dependencies

- `packages/react/test/createRenderer.ts:27-49` — wraps render in act and returns rerender/setProps; the Viewport suite takes render from it at the top level and again (aliased to renderFakeTimers together with clock) inside the timers and focus-management describes. `packages/react/src/toast/viewport/ToastViewport.test.tsx:10` `packages/react/src/toast/viewport/ToastViewport.test.tsx:445-447` `packages/react/src/toast/viewport/ToastViewport.test.tsx:957-959`
- `packages/react/test/describeConformance.tsx:44-49` — the default suite is propsSpread, refForwarding, renderProp, className; `packages/react/test/describeConformance.tsx:55-68` — tests run unless skipped; the Viewport block passes neither skip nor only. `packages/react/src/toast/viewport/ToastViewport.test.tsx:12-17`
- `packages/react/test/conformanceTests/propForwarding.tsx:23-127` — forwards lang, data-foobar and style to the default element and to the render-prop function/element variants.
- `packages/react/test/conformanceTests/refForwarding.tsx:32-38` — asserts the ref instance satisfies refInstanceof (HTMLDivElement for the Viewport).
- `packages/react/test/conformanceTests/renderProp.tsx:41-178` — render prop as function and as element; ref passing/merging and className merging onto the custom element.
- `packages/react/test/conformanceTests/className.tsx:20-23` — a string className lands on the rendered element.
- `packages/react/test/index.ts:1-11` — harness barrel re-exporting shared test utils; the suite imports createRenderer, describeConformance and isJSDOM from it. `packages/react/src/toast/viewport/ToastViewport.test.tsx:5`
- `packages/react/test/advanceReactClock.ts:4-8` — async clock helper; not used directly in this file (the timers suites tick synchronously via clock.tick). `packages/react/src/toast/viewport/ToastViewport.test.tsx:464`
- `packages/react/test/addVitestMatchers.ts:33-47` — registers the toEqualDateTime matcher; not used in this file.
- The render result exposes a user object (user.click, user.keyboard, user.tab) and tests also use act, fireEvent, screen and waitFor from the shared test-utils package. `packages/react/src/toast/viewport/ToastViewport.test.tsx:6` `packages/react/src/toast/viewport/ToastViewport.test.tsx:20` `packages/react/src/toast/viewport/ToastViewport.test.tsx:165-166` `packages/react/src/toast/viewport/ToastViewport.test.tsx:204-205`
- `packages/react/src/toast/utils/test-utils.tsx:6-25` — Button helper adds a toast with title, description and actionProps; `packages/react/src/toast/utils/test-utils.tsx:30-38` — List helper renders Toast.Root (data-testid "root") containing Toast.Title, Toast.Description, Toast.Close (aria-label "close-press") and Toast.Action.
