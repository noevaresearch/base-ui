# Input — behavior spec (Stage 1: behavior mining)

Unit: `library: input`. Its TODO.md entry has no `wraps-external:` field and no
`needs-batched-mining:` field, so this spec was mined in one pass over the unit's full test list.

Test files mined: `packages/react/src/input/Input.test.tsx` (the sole test file; 13 lines).

Scope note: `Input.test.tsx` contains no hand-written behavioral tests. It delegates entirely to
the shared conformance harness `describeConformance`, running the full default suite (no `skip`
or `only` overrides) `packages/react/src/input/Input.test.tsx:9-12`,
`packages/react/test/describeConformance.tsx:44-49`,
`packages/react/test/describeConformance.tsx:55-67`. Everything proven below is what that harness
asserts. Any Input behavior not listed here (value handling, validation, focus, keyboard, events)
is UNVERIFIED by this unit's tests.

## Public API surface (props, parts, subcomponents)

- `<Input />` is a single-part component with no subcomponents exercised by tests; the entire
  suite mounts the bare element with no wrapper parts
  `packages/react/src/input/Input.test.tsx:9-12`.
- The default rendered element is a native `<input>`: the conformance config sets
  `refInstanceof: window.HTMLInputElement` and the ref test asserts the attached ref instance is
  an `instanceof` that constructor `packages/react/src/input/Input.test.tsx:10`,
  `packages/react/test/conformanceTests/refForwarding.tsx:35-37`.
- Props proven on the public API (all via conformance suites):
  - `ref` — attaches to the rendered element
    `packages/react/test/conformanceTests/refForwarding.tsx:32-38`.
  - `className` — accepted as a string and applied to the root element
    `packages/react/test/conformanceTests/className.tsx:20-23`; also accepted as a function
    (resolved className form) and merged under customization
    `packages/react/test/conformanceTests/renderProp.tsx:163-178`.
  - `render` — accepts a render function
    `packages/react/test/conformanceTests/renderProp.tsx:41-59` or a React element
    `packages/react/test/conformanceTests/renderProp.tsx:61-76`; the props handed to a render
    function include `key` (the test destructures it out to reuse as the custom element's key)
    `packages/react/test/conformanceTests/renderProp.tsx:44-49`.
  - Arbitrary DOM props (`lang`, `data-*`, `data-testid`, `style`) spread onto the default root
    element `packages/react/test/conformanceTests/propForwarding.tsx:23-36` and onto custom
    elements whether `render` is a function or an element
    `packages/react/test/conformanceTests/propForwarding.tsx:38-80`; `style` forwarding is
    asserted separately for the component prop, the render-function form, and the JSX-element
    form `packages/react/test/conformanceTests/propForwarding.tsx:82-127`.
- Caveat: in the prop-forwarding and render-prop suites the custom element is a `div`, because
  Input does not set `testRenderPropWith` and the harness default is `'div'`
  `packages/react/test/conformanceTests/propForwarding.tsx:14`,
  `packages/react/test/conformanceTests/renderProp.tsx:14-16`. Those suites therefore prove
  prop/className/ref merging against a `div` host; only the ref suite proves the default root is
  an `<input>`.

## State model (controlled/uncontrolled, defaults, transitions)

N/A — this unit's test suite contains no tests for `value`, `defaultValue`, `onChange`, or any
state transitions. Any stateful behavior of Input is UNVERIFIED — inferred from
`packages/react/src/input/Input.test.tsx:1-13`, no test asserts this.

## Keyboard interactions

N/A — no keyboard interactions are exercised or asserted by this unit's tests.

## Focus management

N/A — no focus behavior is asserted. The only related, proven fact is that the default root is
an `HTMLInputElement` instance (a natively focusable element)
`packages/react/src/input/Input.test.tsx:10`,
`packages/react/test/conformanceTests/refForwarding.tsx:35-37`. Anything beyond that
(autofocus, select-on-focus, data-attributes reflecting focus) is UNVERIFIED — inferred from
`packages/react/src/input/Input.test.tsx:9-12`, no test asserts this.

## Accessibility (roles, aria-*, id linking)

N/A — the suite makes no explicit role, `aria-*`, or id-linking assertions. The only
accessibility-relevant proven fact is that the default element is a native `<input>`, so native
platform semantics apply rather than library-managed ARIA
`packages/react/src/input/Input.test.tsx:10`. Any additional aria wiring is UNVERIFIED —
inferred from `packages/react/src/input/Input.test.tsx:9-12`, no test asserts this.

## DOM structure & portal behavior

- Default DOM: one root element (the native `<input>`) receives forwarded props as attributes;
  after render, a `data-testid` passed to Input resolves on that root node via
  `screen.getByTestId('root')` `packages/react/test/conformanceTests/propForwarding.tsx:29-35`.
- No portal behavior is exercised by the tests: the component renders inline where placed, and
  when customized with `render: <Wrapper />` both the user's extra wrapper element and the
  custom render-target element appear in the DOM (the wrapper is rendered as part of the
  customized tree, not detached) `packages/react/test/conformanceTests/renderProp.tsx:54-58`,
  `packages/react/test/conformanceTests/renderProp.tsx:71-75`.
- Customization may replace the root with another host element while the component keeps
  spreading its props/children onto it, for both function and element forms of `render`
  `packages/react/test/conformanceTests/renderProp.tsx:41-76`.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no event dispatch, payload, bubbling, or preventDefault semantics are asserted by this
unit's tests. UNVERIFIED — inferred from `packages/react/src/input/Input.test.tsx:1-13`, no test
asserts any event behavior.

## Edge cases (rapid interactions, unmount, nesting)

- Nesting (the only edge case covered): the component tolerates being wrapped by an extra
  element when `render` supplies one — a ref passed to `<Input />` still resolves to the custom
  render-target element (its `tagName` is the custom host's and it carries the custom
  `data-testid`) `packages/react/test/conformanceTests/renderProp.tsx:93-113`.
- Ref merging under customization: a ref on `<Input />` and a ref on the `render` element both
  resolve to the same underlying custom node
  `packages/react/test/conformanceTests/renderProp.tsx:115-144`.
- className merging under customization: className from component props (string or function
  form) and className from the `render` element are both present on the final node
  `packages/react/test/conformanceTests/renderProp.tsx:146-161`,
  `packages/react/test/conformanceTests/renderProp.tsx:163-178`.
- Rapid interactions, unmount cleanup, SSR/hydration: N/A — no tests cover them (UNVERIFIED —
  inferred from `packages/react/src/input/Input.test.tsx:1-13`, no test asserts this).

## Shared harness dependencies

`Input.test.tsx` imports and uses only the shared conformance harness — it has no
component-specific test logic. Harness files read (and their own dependencies):

- `packages/react/test/describeConformance.tsx` — orchestrator; defines the four-test full suite
  (propsSpread, refForwarding, renderProp, className) and runs every test in it unless
  `skip`/`only` is passed (Input passes neither), plus the `wrappingAllowed` option (default
  `true`) `packages/react/test/describeConformance.tsx:37-41`,
  `packages/react/test/describeConformance.tsx:44-49`,
  `packages/react/test/describeConformance.tsx:55-67`.
- `packages/react/test/conformanceTests/propForwarding.tsx` — propsSpread suite.
- `packages/react/test/conformanceTests/refForwarding.tsx` — refForwarding suite.
- `packages/react/test/conformanceTests/renderProp.tsx` — renderProp suite.
- `packages/react/test/conformanceTests/className.tsx` — className suite.
- `packages/react/test/conformanceTests/utils.ts` — `throwMissingPropError` guard shared by the
  suites.
- `packages/react/test/createRenderer` — referenced by the harness only as a type import
  (`BaseUIRenderResult`), not invoked `packages/react/test/describeConformance.tsx:12`.
- External npm harness `@mui/internal-test-utils` (not a repo file): supplies `createRenderer`,
  which Input's test instantiates directly `packages/react/src/input/Input.test.tsx:3`, and
  `createDescribe`, `screen`, `flushMicrotasks`, `randomStringValue` used by the suites
  `packages/react/test/conformanceTests/propForwarding.tsx:3`,
  `packages/react/test/conformanceTests/renderProp.tsx:3`.
