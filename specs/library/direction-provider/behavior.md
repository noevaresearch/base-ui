# DirectionProvider — behavior spec

Mined from `packages/react/src/direction-provider/DirectionProvider.test.tsx` (the unit's entire test suite, 42 lines / 2 tests). The unit is a context provider + hook pair; most interaction sections are N/A because a provider has no interactive DOM surface, and no tests exist beyond context-value propagation.

## Public API surface (props, parts, subcomponents)

- The unit exports `DirectionProvider` (component), `useDirection` (hook), and the `TextDirection` type, all imported from `@base_ui/react/direction-provider` in the suite. `packages/react/src/direction-provider/DirectionProvider.test.tsx:3-7`
- The only prop exercised by tests is `direction`, given the values `'rtl'` and `'ltr'` (both members of `TextDirection`). `packages/react/src/direction-provider/DirectionProvider.test.tsx:33-38`
- No other props, parts, or subcomponents are asserted by any test.
- `useDirection()` returns a string value that renders as text (`'ltr'` / `'rtl'`); the suite consumes it in a probe component that displays it directly. `packages/react/src/direction-provider/DirectionProvider.test.tsx:11-14`

## State model (controlled/uncontrolled, defaults, transitions)

- Default value: outside any `<DirectionProvider>`, `useDirection()` returns `'ltr'`. `packages/react/src/direction-provider/DirectionProvider.test.tsx:27-31`
- Inside a provider, `useDirection()` returns the provider's configured `direction` value (`'rtl'` when `direction="rtl"`). `packages/react/src/direction-provider/DirectionProvider.test.tsx:33-36`
- Transition: updating the provider's `direction` prop (`'rtl'` → `'ltr'`) updates the value seen by descendants live, without remounting them. `packages/react/src/direction-provider/DirectionProvider.test.tsx:38-41`
- No internal/uncontrolled state is asserted anywhere in the suite; the only source of the context value proven by tests is the `direction` prop (plus the `'ltr'` default outside a provider). UNVERIFIED — inferred from absence across `packages/react/src/direction-provider/DirectionProvider.test.tsx:24-42`, no test asserts an uncontrolled fallback inside a provider (e.g. omitting the `direction` prop).

## Keyboard interactions

N/A — no keyboard tests exist for this unit; it is a context provider with no interactive surface asserted by tests (`packages/react/src/direction-provider/DirectionProvider.test.tsx:24-42` contains no keyboard usage).

## Focus management

N/A — no focus-related assertions exist in the suite (`packages/react/src/direction-provider/DirectionProvider.test.tsx:24-42` contains no focus usage).

## Accessibility (roles, aria-*, id linking)

No test asserts any rendered DOM attribute of the provider itself — there are no `dir`, role, or `aria-*` assertions anywhere in the suite. UNVERIFIED — inferred from `packages/react/src/direction-provider/DirectionProvider.test.tsx:24-42`, no test asserts whether `<DirectionProvider>` renders a DOM element or sets a `dir` attribute.

## DOM structure & portal behavior

- Tests only observe DOM rendered by the test's own probe (`<span data-testid="direction">`); the provider's own DOM output is never asserted. `packages/react/src/direction-provider/DirectionProvider.test.tsx:11-14`, `packages/react/src/direction-provider/DirectionProvider.test.tsx:30`
- No portal behavior is tested. UNVERIFIED — inferred from `packages/react/src/direction-provider/DirectionProvider.test.tsx:24-42`, no test asserts this.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are dispatched or asserted in the suite (`packages/react/src/direction-provider/DirectionProvider.test.tsx:24-42` contains no event usage).

## Edge cases (rapid interactions, unmount, nesting)

- Not covered. There are no tests for nested providers (a provider inside another provider), unmount behavior, or rapid repeated prop changes. UNVERIFIED — inferred from `packages/react/src/direction-provider/DirectionProvider.test.tsx:24-42`, no test asserts these.
- The only dynamic interaction covered is a single prop update (`'rtl'` → `'ltr'`) via `setProps`, which propagates correctly. `packages/react/src/direction-provider/DirectionProvider.test.tsx:38-41`

## Shared harness dependencies

- `createRenderer` from `#test-utils` (subpath mapped in `packages/react/package.json:110`; re-exported at `packages/react/test/index.ts:3`). It wraps `@mui/internal-test-utils`'s renderer; its `render` runs inside `act`. `packages/react/test/createRenderer.ts:27-43`
- `setProps(newProps)` re-clones the originally rendered element with the new props and rerenders inside `act` — this is the mechanism behind the proven live `rtl → ltr` context transition. `packages/react/test/createRenderer.ts:38-40`
- `screen` queries come from the external npm package `@mui/internal-test-utils` (not a repo harness file). `packages/react/src/direction-provider/DirectionProvider.test.tsx:8`
