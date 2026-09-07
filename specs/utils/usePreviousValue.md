# `usePreviousValue` — behavior spec

Unit: `packages/utils/src/usePreviousValue` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/usePreviousValue.test.tsx` (235 lines, 11 tests, all passing
against a local `TestComponent` wrapper that calls the hook in its render body and exposes the
result through a render-prop child: `packages/utils/src/usePreviousValue.test.tsx:12-15`).

`TODO.md` has no `wraps-external:` field for this unit (TODO.md:194-198) — it is original Base UI
code, not a wrapper around a third-party npm package, so nothing is delegated to an external
package. It is also not in the `needs-batched-mining` set (single small test file), so this is a
single non-batched spec.

## Public API surface (props, parts, subcomponents)

- Single named export: the hook `usePreviousValue`. It is a plain React hook consumed inside a
  component's render body with exactly one argument — the value to track — and its return value is
  read synchronously during that same render (`packages/utils/src/usePreviousValue.test.tsx:12-15`).
- Returns the tracked value from the previous committed render, or `null` when there is no previous
  committed render (first render). Proven for an initial value of a string
  (`packages/utils/src/usePreviousValue.test.tsx:20-32`), a number
  (`packages/utils/src/usePreviousValue.test.tsx:54-63`), `NaN`
  (`packages/utils/src/usePreviousValue.test.tsx:77-88`), `0`
  (`packages/utils/src/usePreviousValue.test.tsx:109-118`), a string again on a fresh mount
  (`packages/utils/src/usePreviousValue.test.tsx:127-138`), an object
  (`packages/utils/src/usePreviousValue.test.tsx:147-162`), and `undefined`
  (`packages/utils/src/usePreviousValue.test.tsx:171-182`).
- Generic over the tracked value type. The consumer types the hook's result as `string | null` and
  the compiler accepts it; at runtime the value is `null` on first render and a `string`
  (`typeof === 'string'`) after one change
  (`packages/utils/src/usePreviousValue.test.tsx:218-234`).
- No props object, no parts, no subcomponents, no context, no render output of its own. N/A — the
  hook is not a component; every test renders a user-provided wrapper whose children return `null`
  (`packages/utils/src/usePreviousValue.test.tsx:24-27`).

## State model (controlled/uncontrolled, defaults, transitions)

- Fully internal, uncontrolled state: the hook keeps the last-seen tracked value itself; the only
  external input is the value passed per render. There is no controlled/uncontrolled prop duality —
  every test drives the tracked value through props of a wrapper component via
  `render(...)` / `setProps(...)` and merely observes the hook's output
  (`packages/utils/src/usePreviousValue.test.tsx:36-51`).
- Initial state: `null`, used as the "no previous value yet" sentinel on the first render
  (`packages/utils/src/usePreviousValue.test.tsx:31`).
- Transition on a render where the tracked value differs from the last-seen value: the hook returns
  the last-seen (previous) value. Proven across a string chain `'first'` → `'second'` → `'third'`
  (`packages/utils/src/usePreviousValue.test.tsx:47-51`), a number/boolean chain
  (`packages/utils/src/usePreviousValue.test.tsx:67-74`), a change to `NaN`
  (`packages/utils/src/usePreviousValue.test.tsx:105-106`), zero-sign changes
  (`packages/utils/src/usePreviousValue.test.tsx:120-124`), and object identity changes
  (`packages/utils/src/usePreviousValue.test.tsx:164-168`).
- Transition on a render where the tracked value is equal to the last-seen value: the hook does NOT
  advance — it keeps returning the older previous value. Two rerenders caused solely by an
  unrelated prop (`unrelatedProp: 1`, then `2`) leave the result at the initial `null`
  (`packages/utils/src/usePreviousValue.test.tsx:140-144`). Likewise, re-setting `NaN` while it is
  already `NaN` (with an unrelated prop forcing the rerender) leaves the result at `null`
  (`packages/utils/src/usePreviousValue.test.tsx:90-91`).
- Equality semantics (when is a change "detected"): the observed behavior matches `Object.is`:
  - `NaN` is treated as equal to a previous `NaN` (unchanged): the result stays `null` when
    `NaN` → `NaN` with an unrelated-prop rerender
    (`packages/utils/src/usePreviousValue.test.tsx:88-91`).
  - A change to `NaN` from a different value is detected: `1` → `NaN` returns `1`
    (`packages/utils/src/usePreviousValue.test.tsx:105-106`).
  - `+0` and `-0` are distinguished in both directions: `0` → `-0` returns `0`, then `-0` → `0`
    returns `-0` (`packages/utils/src/usePreviousValue.test.tsx:120-124`). Under plain `===`,
    `0 === -0` would suppress both transitions, which the tests prove does not happen.
- Objects are compared by reference identity, not deep equality: three distinct object literals
  produce three change transitions, and the returned previous value is the same object instance
  (`toBe`) as the one passed on the prior render
  (`packages/utils/src/usePreviousValue.test.tsx:147-169`).
- `undefined` and `null` are tracked as first-class values, distinct from each other and from the
  initial sentinel: initial `undefined` → result `null`
  (`packages/utils/src/usePreviousValue.test.tsx:174-182`); `undefined` → `null` returns
  `undefined` (`packages/utils/src/usePreviousValue.test.tsx:184-185`); `null` → `'defined'`
  returns `null` (`packages/utils/src/usePreviousValue.test.tsx:187-188`); `'defined'` →
  `undefined` returns `'defined'`
  (`packages/utils/src/usePreviousValue.test.tsx:190-191`).
- There is no way to reset, set, or read the internal state other than changing the tracked value;
  no test exercises any reset path.

## Keyboard interactions

N/A — non-visual utility. The suite registers no keyboard handlers and asserts no key-driven
behavior (`packages/utils/src/usePreviousValue.test.tsx:1-235`).

## Focus management

N/A — non-visual utility. No focus behavior is implemented or asserted
(`packages/utils/src/usePreviousValue.test.tsx:1-235`).

## Accessibility (roles, aria-*, id linking)

N/A — non-visual utility. It renders nothing, sets no attributes, and creates no ARIA
relationships; the suite asserts no roles or `aria-*` attributes
(`packages/utils/src/usePreviousValue.test.tsx:1-235`).

## DOM structure & portal behavior

N/A — the hook creates no DOM and performs no portal behavior. Every test's wrapper renders
`null` as its tree (`packages/utils/src/usePreviousValue.test.tsx:24-27`), and no test asserts
anything about the DOM. The hook's entire observable surface is the value it returns during
render.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — the hook emits no DOM events and registers no listeners. The only observable "signal" is the
updated return value on the next committed render
(`packages/utils/src/usePreviousValue.test.tsx:47-51`). There is no event name, payload, bubbling,
or `preventDefault` semantics to model.

## Edge cases (rapid interactions, unmount, nesting)

- Rapid/batched value changes: three `setProps` calls (`'first'`, `'second'`, `'third'`) issued
  inside a single `act()` batch produce only one commit with the final value, and the hook reports
  the value from before the batch (`'initial'`) — i.e., the hook is render-synchronized and never
  observes intermediate values that React never committed
  (`packages/utils/src/usePreviousValue.test.tsx:207-215`).
- Re-renders without value changes are inert (the same-state case above): the previous value is not
  overwritten or "bumped" on equal-value renders
  (`packages/utils/src/usePreviousValue.test.tsx:140-144`).
- Special numeric values (`NaN`, `+0`/`-0`) behave per `Object.is` semantics as detailed in the
  state model section (`packages/utils/src/usePreviousValue.test.tsx:77-125`).
- Unmount: no test in this suite unmounts the wrapper or asserts anything about cleanup, so no
  unmount behavior is established by tests. UNVERIFIED — no test asserts this; suite coverage ends
  at `packages/utils/src/usePreviousValue.test.tsx:1-235`.
- Nesting: no test composes the hook in nested components or asserts cross-instance independence,
  so no nesting behavior is established by tests. UNVERIFIED — no test asserts this; suite
  coverage ends at `packages/utils/src/usePreviousValue.test.tsx:1-235`.
- StrictMode/double-render behavior: no test renders with `strict: true`, so double-invocation
  behavior is not established by tests. UNVERIFIED — no test asserts this; suite coverage ends at
  `packages/utils/src/usePreviousValue.test.tsx:1-235`.

## Shared harness dependencies

- The test file imports `createRenderer` and `act` from `@mui/internal-test-utils`
  (`packages/utils/src/usePreviousValue.test.tsx:3`) — an external npm package, NOT a repo-local
  `#test-utils` or `packages/react/test/` harness file.
- From that harness the suite uses: `render()` returning `setProps` for driving prop changes
  (`packages/utils/src/usePreviousValue.test.tsx:18`, `:36`, `:47`), and the re-exported `act()`
  to batch multiple `setProps` calls into one commit
  (`packages/utils/src/usePreviousValue.test.tsx:207-211`).
- No repo-local shared harness files (`#test-utils`, `packages/react/test/*`) are imported by this
  unit's tests; there are no other harness dependencies to read for this spec.
