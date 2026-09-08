# Button behavior spec

Mined from `packages/react/src/button/Button.test.tsx`. No `wraps-external:` field in the
button `TODO.md` entry (`TODO.md:329-335`), so the behavior below is derived entirely from the
component's own tests.

## Public API surface (props, parts, subcomponents)

- `Button` is a single root component; no sub-parts are exercised by the tests. By default it
  renders a native `<button>` element, proven by the conformance `refInstanceof:
  window.HTMLButtonElement` assertion. `packages/react/src/button/Button.test.tsx:10-14`
- Props directly exercised by the tests:
  - `render` — accepts both a JSX element (`<a href="#target" />`, `<span />`) and a function;
    the resulting element replaces the root. `packages/react/src/button/Button.test.tsx:20-24`, `packages/react/src/button/Button.test.tsx:52`
  - `nativeButton={false}` — switches from native-button semantics to ARIA-button semantics on a
    custom element. `packages/react/src/button/Button.test.tsx:21`, `packages/react/src/button/Button.test.tsx:50`
  - `disabled` — native buttons get the `disabled` attribute. `packages/react/src/button/Button.test.tsx:118`
  - `focusableWhenDisabled` — keeps a `disabled` button focusable. `packages/react/src/button/Button.test.tsx:184`
  - `onClick`, `onMouseDown`, `onPointerDown`, `onKeyDown`, `onMouseMove` — public event-handler
    props observed in tests. `packages/react/src/button/Button.test.tsx:107-113`, `packages/react/src/button/Button.test.tsx:219-224`
  - `className`, `style`, arbitrary attributes (`lang`, `data-*`), and `ref` — forwarded to the
    root element per the conformance suite. `packages/react/test/conformanceTests/propForwarding.tsx:33-35`, `packages/react/test/conformanceTests/propForwarding.tsx:92-94`, `packages/react/test/conformanceTests/className.tsx:20-23`, `packages/react/test/conformanceTests/refForwarding.tsx:32-37`
- `nativeButton` and `disabled` combine with `render`: the custom element receives the semantics
  regardless of tag (an `<a>` stays an `<a>` with button role). `packages/react/src/button/Button.test.tsx:26-27`

## State model (controlled/uncontrolled, defaults, transitions)

- The tests never assert a separately-managed internal state machine; `disabled` is the only
  observable state and is fully controlled by the caller's prop. UNVERIFIED — inferred from
  behavior, no test asserts the absence of internal state.
- Transition exercised: toggling `disabled` from `false` to `true` *after* the button is focused
  keeps focus on it (no focus loss on becoming disabled). `packages/react/src/button/Button.test.tsx:270-281`
- Defaults proven: default root is a native `<button>` (`refInstanceof`), so `nativeButton`
  default must be `true`. `packages/react/src/button/Button.test.tsx:12`; default `tabindex` on a
  custom element is `0`. `packages/react/src/button/Button.test.tsx:64`

## Keyboard interactions

- Native button, non-disabled: keyboard activation is delegated to the native element; the tests
  do not intercept Space/Enter on native buttons beyond the disabled case. UNVERIFIED — inferred
  from `packages/react/src/button/Button.test.tsx:8-14`, no test asserts native keyup click counts.
- Custom element (`nativeButton={false}`), Space: on `keydown` with `key: ' '` the component calls
  `preventDefault()` (the synthetic event fires back `false`), so pressing Space on an `<a>` link
  does not scroll the page; the subsequent `keyup` fires the click handler once and the link
  navigates (`window.location.hash` changes). `packages/react/src/button/Button.test.tsx:32-39`
- Custom element (`nativeButton={false}`), Enter and Space: both keys dispatch a real click event
  on every activation. `packages/react/src/button/Button.test.tsx:69-70`
- Custom element (`nativeButton={false}`), modifier-key state: the click dispatched from keyboard
  activation preserves modifier keys held during the keypress (e.g. `shiftKey: true`). `packages/react/src/button/Button.test.tsx:92-95`
- Disabled (native and custom, with or without `focusableWhenDisabled`): Space and Enter produce no
  `keydown` and no `click` handler calls, i.e. keyboard activation is fully suppressed.
  `packages/react/src/button/Button.test.tsx:126-132`, `packages/react/src/button/Button.test.tsx:164-170`, `packages/react/src/button/Button.test.tsx:203-209`, `packages/react/src/button/Button.test.tsx:314-320`

## Focus management

- Native button, disabled: rendered with the `disabled` attribute and is skipped by Tab (never
  gains focus). `packages/react/src/button/Button.test.tsx:122-123`
- Custom element, non-disabled: sets `tabindex="0"` and is reachable via Tab (`[Tab]` focuses it).
  `packages/react/src/button/Button.test.tsx:64-67`
- Custom element, disabled: sets `tabindex="-1"` and is skipped by Tab.
  `packages/react/src/button/Button.test.tsx:158-161`
- `focusableWhenDisabled` (native and custom): removes the `disabled`/focus-blocking behavior,
  sets `tabindex="0"`, and the button gains focus via Tab. `packages/react/src/button/Button.test.tsx:197-200`, `packages/react/src/button/Button.test.tsx:306-311`
- A `disabled` button that becomes disabled while focused retains focus.
  `packages/react/src/button/Button.test.tsx:272-281`

## Accessibility (roles, aria-*, id linking)

- Custom element, non-disabled: gets `role="button"` and `tabindex="0"`; queryable as a button by
  accessible name (`getByRole('button', { name: 'Save' })`). `packages/react/src/button/Button.test.tsx:60-64`
- Custom element, disabled: gets `aria-disabled="true"` plus `data-disabled`, and no `disabled`
  attribute. `packages/react/src/button/Button.test.tsx:155-157`
- Native element, disabled: gets the native `disabled` attribute and `data-disabled`, and
  explicitly does NOT get `aria-disabled`. `packages/react/src/button/Button.test.tsx:118-120`
- `focusableWhenDisabled` + `disabled` (native and custom): `aria-disabled="true"` + `data-disabled`,
  no `disabled` attribute, `tabindex="0"`. `packages/react/src/button/Button.test.tsx:194-197`, `packages/react/src/button/Button.test.tsx:305-308`
- No `aria-*` id-linking (e.g. `aria-labelledby`/`id`) is asserted anywhere in the tests. N/A for
  id linking; no test covers it.

## DOM structure & portal behavior

- Default root is a native `<button>` element. `packages/react/src/button/Button.test.tsx:12`
- With `render`, the custom element is the root: no wrapper element is added around it — an `<a>`
  remains tag ‘A’ (`packages/react/src/button/Button.test.tsx:27`) and a `<span>` remains ‘SPAN’ (`packages/react/src/button/Button.test.tsx:62`).
- The `render` element's own props (e.g. `href`, `onClick`, `onClickCapture`) compose with the
  Button's props on the same element rather than nesting. `packages/react/src/button/Button.test.tsx:21-24`, `packages/react/src/button/Button.test.tsx:50-56`
- `render` as a function receives forwarded props, including `style`, and may render a custom
  component that receives the forwarded ref; component and Button `className`s merge.
  `packages/react/test/conformanceTests/renderProp.tsx:41-58`, `packages/react/test/conformanceTests/renderProp.tsx:93-113`, `packages/react/test/conformanceTests/renderProp.tsx:146-161`, `packages/react/test/conformanceTests/propForwarding.tsx:97-112`
- No portal: the Button renders in place and no portal/`createPortal` behavior is tested. N/A for
  portal behavior.

## Events (names, payload shape, bubbling, preventDefault semantics)

- Keyboard activation on a non-native button dispatches a real, document-level `click` event with
  normal capture/bubble propagation order: capture-phase handler on the render element fires, then
  the render element's bubble handler, then the Button's `onClick`, then an ancestor's `onClick` —
  all once per activation (2 total across Enter + Space). `packages/react/src/button/Button.test.tsx:72-75`
- The synthesized click from keyboard activation preserves modifier state on the event object
  (`shiftKey: true`). `packages/react/src/button/Button.test.tsx:95`
- Space `keydown` on a non-native button calls `preventDefault()` (verified via the synthetic
  event returning `false`); this prevents the page from scrolling on Space-activation of a link.
  `packages/react/src/button/Button.test.tsx:32-33`
- Disabled suppresses all of `click`, `mousedown`, `pointerdown`, and `keydown` (0 invocations) for
  native, custom-element, and `focusableWhenDisabled` variants. `packages/react/src/button/Button.test.tsx:129-132`, `packages/react/src/button/Button.test.tsx:167-170`, `packages/react/src/button/Button.test.tsx:206-209`, `packages/react/src/button/Button.test.tsx:317-320`
- With `focusableWhenDisabled`, mouse movement events (`mousemove`) still fire on the disabled
  button while activation (`click`) stays blocked. `packages/react/src/button/Button.test.tsx:233-239`

## Edge cases (rapid interactions, unmount, nesting)

- Rapid keyboard activation: Enter and Space in sequence each produce exactly one `click`
  (2 total), including capture/render/ancestor handlers. `packages/react/src/button/Button.test.tsx:69-75`
- Becoming disabled during an interaction: clicking the button once fires the click and toggles
  `disabled` to `true`; subsequent mouse clicks, Enter, and Space all stop firing the handler while
  focus is retained. `packages/react/src/button/Button.test.tsx:270-281`
- Nesting: clicks dispatched by keyboard activation bubble up through an ancestor element's `onClick`.
  `packages/react/src/button/Button.test.tsx:48-49`, `packages/react/src/button/Button.test.tsx:75`
- UNVERIFIED — unmounting mid-interaction is not covered by any test.

## Shared harness dependencies

- `#test-utils` (module alias `./test/index.ts`, defined at `packages/react/package.json:110`)
  provides `describeConformance`, `createRenderer`, and `isJSDOM` used by this test.
  `packages/react/src/button/Button.test.tsx:5`
- `describeConformance` (`packages/react/test/describeConformance.tsx:44-49`) runs four suites with
  `button: true`: `propsSpread`/prop forwarding, `refForwarding`, `renderProp`, and `className`.
  Button's options set `refInstanceof: window.HTMLButtonElement` (`packages/react/src/button/Button.test.tsx:10-14`), so the suite asserts the default root is a native button, refs attach and merge on render-prop elements, and props/`className`/`style` forward to both default and customized roots.
  `packages/react/test/conformanceTests/refForwarding.tsx:32-37`, `packages/react/test/conformanceTests/renderProp.tsx:115-144`
- `createRenderer` (`packages/react/test/createRenderer.ts:27-48`) wraps
  `@mui/internal-test-utils`'s renderer and awaits `act`; the returned `render` is a promise and
  the test's `user` object comes from the testing-library user-event setup. `packages/react/test/createRenderer.ts:31-43`
- `isJSDOM` is re-exported from `@base-ui/utils/testUtils` via `#test-utils`
  (`packages/react/test/index.ts:1`) and gates Chromium-only tests. `packages/react/src/button/Button.test.tsx:212`
- `fireEvent`, `screen`, and `waitFor` come from `@mui/internal-test-utils`.
  `packages/react/src/button/Button.test.tsx:4`