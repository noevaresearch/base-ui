# useControlled — behavior spec

Unit: `useControlled` (a React hook in `packages/utils/src`). The `TODO.md` entry for this unit
(TODO.md:144-148) has no `wraps-external:` field, so no third-party delegation applies — all
behavior below is derived from the unit's own two test files:
`packages/utils/src/useControlled.test.tsx` (runtime behavior) and
`packages/utils/src/useControlled.spec.ts` (compile-time type behavior).

## Public API surface (props, parts, subcomponents)

- Not a component — a React hook. It has no parts, no subcomponents, and no rendered output of
  its own; consumers destructure a `[value, setValue]` tuple from its return value
  `packages/utils/src/useControlled.test.tsx:19`.
- Called with a single options object of three fields: `controlled` (the externally controlled
  value, may be `undefined`), `default` (the uncontrolled initial value), and `name` (a string
  label surfaced in dev warning messages) `packages/utils/src/useControlled.test.tsx:19-23`.
- The returned setter is a standard React state setter typed
  `React.Dispatch<React.SetStateAction<...>>` `packages/utils/src/useControlled.test.tsx:7-10`.
- Type-level narrowing (asserted via `expectType`, see Shared harness dependencies):
  - When `default` may be `undefined`, `value` is `T | undefined` and the setter accepts
    `SetStateAction<T | undefined>` `packages/utils/src/useControlled.spec.ts:5-14`.
  - When `default` is a defined value (`false`), `value` narrows to `T` and the setter to
    `SetStateAction<T>` `packages/utils/src/useControlled.spec.ts:16-25`.
  - An explicit generic argument (`useControlled<boolean>`) produces the same two shapes
    `packages/utils/src/useControlled.spec.ts:27-47`.
- The `name` option is interpolated into every warning the hook emits — the observed messages
  embed `"TestComponent"` (`packages/utils/src/useControlled.test.tsx:22` used in warnings at
  `packages/utils/src/useControlled.test.tsx:70-74` and
  `packages/utils/src/useControlled.test.tsx:127-131`) and `"TestHook"`
  (`packages/utils/src/useControlled.test.tsx:86` used in the warning at
  `packages/utils/src/useControlled.test.tsx:104-108`).

## State model (controlled/uncontrolled, defaults, transitions)

- Uncontrolled mode: rendering with only `default` initializes the exposed value to that default
  (`1`) `packages/utils/src/useControlled.test.tsx:30-42`; calling the setter (`setValueState(2)`)
  updates the exposed value to the new value (`2`)
  `packages/utils/src/useControlled.test.tsx:44-48`.
- Controlled mode: rendering with a `controlled` prop exposes that prop's value (`1`) directly;
  the test never calls the setter in this mode `packages/utils/src/useControlled.test.tsx:51-62`.
- Transition uncontrolled → controlled: an initially uncontrolled instance (no `controlled`, no
  `default`) renders without a warning, but re-rendering with a `controlled` value emits the dev
  warning "Base UI: A component is changing the uncontrolled value state of TestComponent to be
  controlled." `packages/utils/src/useControlled.test.tsx:64-75`.
- Transition controlled → uncontrolled: re-rendering a controlled instance with
  `controlled: undefined` emits the dev warning "Base UI: A component is changing the controlled
  value state of TestHook to be uncontrolled."
  `packages/utils/src/useControlled.test.tsx:104-108`. On that switch the exposed value falls
  back to the `default` value (`'default'`) `packages/utils/src/useControlled.test.tsx:110`, and
  a subsequent setter call (`'next'`) leaves the exposed value unchanged at `'default'`
  `packages/utils/src/useControlled.test.tsx:112-116`.
- `default` change detection (uncontrolled): changing `default` after initialization emits the
  dev warning "Base UI: A component is changing the default value state of an uncontrolled
  TestComponent after being initialized." `packages/utils/src/useControlled.test.tsx:120-132`.
- No `default`-change warning while controlled: with `controlled` provided, supplying and then
  changing `default` emits no warning at any point
  `packages/utils/src/useControlled.test.tsx:134-148`.
- Warn-once semantics: only the first post-initialization `default` change warns; later changes
  (`1 → 2`, then back to `0`) emit no further warnings
  `packages/utils/src/useControlled.test.tsx:222-242`. The same warn-only-on-first-change
  behavior holds for defaults containing React elements/functions (change to the function item
  warns; subsequent changes do not) `packages/utils/src/useControlled.test.tsx:244-278`.
- All observed warnings are `Base UI:`-prefixed and are asserted as dev-time `console.error`
  output via the harness `toErrorDev` matcher (see Shared harness dependencies); the uncontrolled
  initial renders in those same tests assert `not.toErrorDev()`
  `packages/utils/src/useControlled.test.tsx:66-68`.

## Keyboard interactions

N/A — headless state hook. No test in this unit exercises any keyboard event.

## Focus management

N/A — headless state hook. No test in this unit exercises focus.

## Accessibility (roles, aria-*, id linking)

N/A — the hook produces no DOM and no test asserts any role, aria attribute, or id linkage.

## DOM structure & portal behavior

N/A — the hook renders nothing and no portal behavior exists. The runtime harness wraps it in a
render-prop component whose own output is decided by the caller (returning `null` in most tests)
`packages/utils/src/useControlled.test.tsx:18-25`.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — the hook emits no events, accepts no callbacks, and has no bubbling/preventDefault
semantics. Its only outward channel is the returned setter, which every runtime test calls with
a plain value (`setValueState(2)` `packages/utils/src/useControlled.test.tsx:44-48`;
`result.current[1]('next')` `packages/utils/src/useControlled.test.tsx:112-116`). The
functional-updater form (`setValue(prev => next)`) is implied by the `SetStateAction` setter type
`packages/utils/src/useControlled.spec.ts:13` but UNVERIFIED — inferred from
`packages/utils/src/useControlled.spec.ts:13`, no runtime test calls the setter with an updater
function.

## Edge cases (rapid interactions, unmount, nesting)

- Exotic `default` values must not spuriously warn or throw on initial render: `NaN`
  `packages/utils/src/useControlled.test.tsx:150-154`, arrays
  `packages/utils/src/useControlled.test.tsx:156-169`, objects containing React elements
  `packages/utils/src/useControlled.test.tsx:171-186`, objects containing functions
  `packages/utils/src/useControlled.test.tsx:188-205`, and `bigint`
  `packages/utils/src/useControlled.test.tsx:207-220`.
- `null`/`undefined` default handling: an initial `default` of `null` renders without warning;
  changing it to `undefined` warns once; changing it back to `null` does not warn again
  `packages/utils/src/useControlled.test.tsx:280-299`.
- Mode switching is exercised in both directions (uncontrolled→controlled
  `packages/utils/src/useControlled.test.tsx:64-75`; controlled→uncontrolled
  `packages/utils/src/useControlled.test.tsx:77-117`).
- No test covers unmount/remount cleanup, rapid repeated setter calls, or multiple simultaneous
  instances interacting; no behavior is asserted for those scenarios.

## Shared harness dependencies

- `@mui/internal-test-utils` (npm package), imported for `createRenderer` and `act`
  `packages/utils/src/useControlled.test.tsx:4`; `createRenderer()` provides the environment-
  aware `render` used throughout `packages/utils/src/useControlled.test.tsx:28`.
- The `toErrorDev` / `not.toErrorDev` chai matchers used pervasively
  (`packages/utils/src/useControlled.test.tsx:66-74` and every other warning assertion) are
  provided by this harness's chai plugin, which registers `toErrorDev` as a matcher asserting on
  captured `console.error` output (see `node_modules/@mui/internal-test-utils/chaiPlugin.mjs`
  and its type declaration `node_modules/@mui/internal-test-utils/chaiTypes.d.ts`; no line
  citations for `node_modules` files since they are not stable across installs).
- `renderHook` from `@testing-library/react` `packages/utils/src/useControlled.test.tsx:3`, used
  for the controlled→uncontrolled hook-level test
  `packages/utils/src/useControlled.test.tsx:78-88`.
- `./testUtils` (resolves to `packages/utils/src/testUtils.ts`), imported by the type spec
  `packages/utils/src/useControlled.spec.ts:2`; provides `expectType<Expected, Actual>`, a no-op
  runtime function that produces a compile error unless the two types are identical
  `packages/utils/src/testUtils.ts:10-21`. The type tests are compile-time-only: the exercising
  functions are explicitly voided `packages/utils/src/useControlled.spec.ts:49-52`.
- No `#test-utils` alias or `packages/react/test/` harness files are imported by this unit's
  tests.
