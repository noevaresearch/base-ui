# `getReactElementRef` — behavior spec

Unit: `packages/utils/src/getReactElementRef` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/getReactElementRef.test.tsx` (the only test file for this
unit). The `TODO.md` entry has no `wraps-external:` field, so nothing here is delegated to a
third-party package.

## Public API surface (props, parts, subcomponents)

- Single named export `getReactElementRef`. No components, no props object, no parts, no
  subcomponents (`packages/utils/src/getReactElementRef.test.tsx:3`).
- Signature as exercised: `getReactElementRef(element)` — one positional argument that may be any
  value. Tests pass the boolean `false`, `undefined`, the number `1`, an array of React elements,
  and React elements (`<div>`/`<React.Fragment>` JSX) (`packages/utils/src/getReactElementRef.test.tsx:6-12`,
  `packages/utils/src/getReactElementRef.test.tsx:17`, `packages/utils/src/getReactElementRef.test.tsx:24-28`,
  `packages/utils/src/getReactElementRef.test.tsx:34`).
- Return value is exactly `null` (asserted with `toBe(null)`, i.e. the value `null`, not
  `undefined`) whenever the input is not a ref-carrying React element
  (`packages/utils/src/getReactElementRef.test.tsx:6-9`, `packages/utils/src/getReactElementRef.test.tsx:12`,
  `packages/utils/src/getReactElementRef.test.tsx:30`, `packages/utils/src/getReactElementRef.test.tsx:36`).
- Return value is the element's own ref object when the input is a host element carrying a `ref`
  prop; the returned value is compared with `toBe` against the `React.createRef` object, so the
  ref is returned by strict identity — no wrapping or unwrapping
  (`packages/utils/src/getReactElementRef.test.tsx:15-19`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — stateless pure function over its single argument. No state, no defaults, no transitions;
each call is independent and there is nothing to control (`packages/utils/src/getReactElementRef.test.tsx:5-37`).

## Keyboard interactions

N/A — non-visual utility. No test asserts any keyboard behavior.

## Focus management

N/A — no focus behavior is implemented or asserted by any test.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test.

## DOM structure & portal behavior

N/A — the suite is pure-call: it imports only vitest and React and never calls a renderer, so no
DOM is created and no portal behavior exists or is asserted
(`packages/utils/src/getReactElementRef.test.tsx:1-3`, `packages/utils/src/getReactElementRef.test.tsx:5-37`).
Note the input elements are JSX element descriptions, never mounted; the function inspects the
element object, not any mounted instance (`packages/utils/src/getReactElementRef.test.tsx:17`,
`packages/utils/src/getReactElementRef.test.tsx:24-28`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are emitted, dispatched, or asserted by any test.

## Edge cases (rapid interactions, unmount, nesting)

Proven behaviors:

- Non-React-element inputs return `null`: the boolean `false`, `undefined`, and the number `1`
  (`packages/utils/src/getReactElementRef.test.tsx:6-9`).
- An array of React elements is rejected as a whole and returns `null` — there is no per-item
  unwrapping of children arrays in the tested case (`packages/utils/src/getReactElementRef.test.tsx:11-12`).
- A `React.Fragment` element returns `null` even though it has element children — fragments are
  not treated as ref-carrying elements (`packages/utils/src/getReactElementRef.test.tsx:22-31`).
- A host element created without a `ref` prop returns `null`
  (`packages/utils/src/getReactElementRef.test.tsx:33-37`).
- Identity preservation for the happy path: the value returned equals the exact ref object
  supplied via the `ref` prop (`toBe`) (`packages/utils/src/getReactElementRef.test.tsx:15-19`).

Unproven behaviors (downstream consumers must not rely on them being specified here):

- Callback (function) refs: no test asserts that an element whose `ref` prop is a function returns
  that function; UNVERIFIED — inferred from `packages/utils/src/getReactElementRef.test.tsx:15-19`,
  no test asserts this (only a `React.createRef` object ref is exercised).
- Explicit `ref={null}` or `ref={undefined}` props: UNVERIFIED — inferred from
  `packages/utils/src/getReactElementRef.test.tsx:33-37`, no test asserts this (the tested
  ref-less element omits the prop entirely).
- `null` passed as the input value itself: UNVERIFIED — inferred from
  `packages/utils/src/getReactElementRef.test.tsx:6-9`, no test asserts this (`undefined` is
  tested; `null` is not).
- Array input whose elements do carry refs: UNVERIFIED — inferred from
  `packages/utils/src/getReactElementRef.test.tsx:11-12`, no test asserts this (the tested array's
  elements are ref-less).
- Refs on non-host element types (forwardRef components, class components, `React.memo`/lazy
  wrappers, portals, string legacy refs): UNVERIFIED — inferred from
  `packages/utils/src/getReactElementRef.test.tsx:15-19`, no test asserts these (only a plain
  `<div>` host element is exercised).
- Rapid repeated calls, nesting (using one call's result as another call's input), or unmount
  lifecycle interaction: N/A — pure function with no lifecycle; no test covers call-pattern edge
  cases (`packages/utils/src/getReactElementRef.test.tsx:5-37`).

## Shared harness dependencies

None. The test file imports only `expect`/`describe`/`it` from `vitest`, `React` (used for
`createRef`, `Fragment`, and JSX syntax), and the unit's own sibling module
(`packages/utils/src/getReactElementRef.test.tsx:1-3`). There is no `#test-utils` or
`packages/react/test` harness import, and no other component's test files are involved.
