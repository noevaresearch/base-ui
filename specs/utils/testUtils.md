# `testUtils` — behavior spec

Unit: `packages/utils/src/testUtils` (Phase A util → crate `leptos-ui-utils`).
Source of truth: **none for runtime behavior** — this unit has no test file of its own (no
`packages/utils/src/testUtils.test.*` exists). The only test files that exercise it are five
type-test `.spec.ts` files in `packages/utils/src` that import `expectType` from it
(`packages/utils/src/addEventListener.spec.ts:1`, `packages/utils/src/useControlled.spec.ts:2`,
`packages/utils/src/store/Store.spec.ts:1`, `packages/utils/src/store/ReactStore.spec.ts:1`,
`packages/utils/src/store/createSelector.spec.ts:1`) — compile-time assertions only; none of
them executes the module's runtime code or asserts any runtime value. Every runtime-behavior
claim below is therefore UNVERIFIED by tests and inferred from the unit's own source; Stage 3
must not treat this spec as test-proven behavior and should encode these expectations as its
own Rust tests (`crates/leptos-ui-utils`) rather than binding to a reference suite.

## Public API surface (props, parts, subcomponents)

- Single module with three named exports; no default export, no components, no props, no parts,
  no subcomponents (`packages/utils/src/testUtils.ts:4-21`):
  - `isJSDOM` — a `boolean` constant, exported at module scope as
    `export const isJSDOM = /jsdom/.test(window.navigator.userAgent);`
    (`packages/utils/src/testUtils.ts:4`), documented with the JSDoc "Whether the test runs in
    JSDOM environment" (`packages/utils/src/testUtils.ts:1-3`).
  - `IfEquals<T, U, Y = unknown, N = never>` — a type-only export; the conditional type
    `(<G>() => G extends T ? 1 : 2) extends <G>() => G extends U ? 1 : 2 ? Y : N` resolves to
    `Y` only when `T` and `U` are *exactly* the same type and to `N` otherwise, attributed to
    the linked Stack Overflow technique for testing exact type identity
    (`packages/utils/src/testUtils.ts:6-8`).
  - `expectType<Expected, Actual>(_actual: IfEquals<Actual, Expected, Actual>): void {}` — a
    function whose parameter type collapses to `Actual` when `Actual` and `Expected` are
    identical and to `never` otherwise, so passing a wrongly-typed value is a TypeScript
    compile error; the function body is empty, i.e. it is a pure compile-time assertion with
    zero runtime cost or checking (`packages/utils/src/testUtils.ts:21`). Its JSDoc documents
    the intended usage — `Expected` declared at the call site, `Actual` almost always a
    `typeof value` expression — with the worked example
    `` `expectType<number | string, typeof value>(value)` `` failing to compile when `value` is
    not exactly `number | string` (`packages/utils/src/testUtils.ts:10-20`, example at
    `packages/utils/src/testUtils.ts:16`).
- Real call sites follow exactly that JSDoc-documented shape, always as top-of-call
  `expectType<Expected, typeof value>(value)` statements inside `.spec.ts` type tests, e.g.
  asserting hook return shapes (`packages/utils/src/useControlled.spec.ts:12-13`), event
  object types (`packages/utils/src/addEventListener.spec.ts:5`), store/sub-store instances
  (`packages/utils/src/store/Store.spec.ts:7`, `packages/utils/src/store/ReactStore.spec.ts:57`),
  and selector return types
  (`packages/utils/src/store/createSelector.spec.ts:19`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — the module holds no state machine, no controlled/uncontrolled duality, and no
transitions. The only state-like behavior is a module-load-time snapshot: `isJSDOM` is
computed once when the module is first imported, by testing the substring `jsdom` against
`window.navigator.userAgent` (`packages/utils/src/testUtils.ts:4`). Consequences (all
UNVERIFIED — inferred from `packages/utils/src/testUtils.ts:4`, no test asserts this):

- Every reader observes the same frozen boolean for the lifetime of the module instance; later
  changes to the user-agent string are not observed.
- The type parameters `Y = unknown` / `N = never` are compile-time defaults of `IfEquals`, not
  runtime state (`packages/utils/src/testUtils.ts:7`).

## Keyboard interactions

N/A — non-visual, non-DOM utility. No keyboard behavior is implemented, listened for, or
asserted anywhere.

## Focus management

N/A — the unit computes no values used for focus and performs no focus management.

## Accessibility (roles, aria-*, id linking)

N/A — sets no roles, aria attributes, or id links, and renders nothing. Its only accessibility
relevance is indirect: `isJSDOM` lets layout-dependent test suites skip assertions in jsdom
that only hold in a real browser, keeping browser-only a11y/layout checks meaningful.

## DOM structure & portal behavior

N/A — creates no DOM, renders nothing, performs no portal behavior. The sole DOM-adjacent
touchpoint is reading `window.navigator.userAgent` at import time
(`packages/utils/src/testUtils.ts:4`) — UNVERIFIED — inferred from
`packages/utils/src/testUtils.ts:4`, no test asserts this: the module assumes `window` exists
at import time, so importing it in a bare Node environment without a DOM global would throw,
while every supported test environment (jsdom, browser) provides `window`.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are emitted, dispatched, listened for, or handled. (`expectType` does appear
in tests that assert the *payload types* of other modules' event listeners, e.g. pinning a
pointerdown listener's event parameter to `PointerEvent`
(`packages/utils/src/addEventListener.spec.ts:5`), but the unit itself participates in no
event system.)

## Edge cases (rapid interactions, unmount, nesting)

- Rapid interactions / unmount / nesting: N/A — a stateless module with no lifecycle, no
  cleanup, and no DOM involvement; repeated imports yield the same constants, and there is
  nothing to unmount or nest (UNVERIFIED — inferred from `packages/utils/src/testUtils.ts:4`
  and `packages/utils/src/testUtils.ts:21`, no test asserts this).
- Environment detection is a loose substring test, not an anchored or vendor-based check: any
  user-agent string containing `jsdom` anywhere classifies the run as jsdom
  (`packages/utils/src/testUtils.ts:4`) — UNVERIFIED — inferred from
  `packages/utils/src/testUtils.ts:4`, no test asserts this.
- `expectType` fails silently at runtime by design: because the body is empty
  (`packages/utils/src/testUtils.ts:21`), a type mismatch only surfaces as a TypeScript error
  in the consuming `.spec.ts` file; if such a file were ever compiled with type checking
  skipped, the assertion would pass vacuously (UNVERIFIED — inferred from
  `packages/utils/src/testUtils.ts:21`, no test asserts this).
- Exactness boundary of `IfEquals`: per its construction it is strict identity, so e.g. a
  value typed `string` does not satisfy `expectType<string | number, typeof value>(value)`
  even though the two are mutually assignable in one direction — this strictness is what the
  consuming type tests rely on to pin down unions vs. primitives
  (`packages/utils/src/testUtils.ts:6-8`, usage at
  `packages/utils/src/useControlled.spec.ts:12-13`) — UNVERIFIED — inferred from
  `packages/utils/src/testUtils.ts:6-8`, no test asserts this directly (it is exercised only
  transitively by compile-time usage).

## Shared harness dependencies

- The unit's own test files: none exist, so it imposes no harness dependency of its own.
- The unit *is itself* shared harness: `packages/react/test/index.ts` — the module behind the
  `#test-utils` alias — re-exports it wholesale via `export * from '@base-ui/utils/testUtils'`
  (`packages/react/test/index.ts:1`), which is how library component type tests obtain
  `expectType` (e.g. `packages/react/src/select/root/SelectRoot.spec.tsx:2`) and how test
  suites obtain `isJSDOM` for environment gating. One harness file also imports it directly
  rather than through the barrel: `packages/react/test/resetBrowserPointer.ts:1`.
- Distribution surface: the package's exports map publishes the module as
  `@base-ui/utils/testUtils` through the `./*` wildcard (`"./*": "./src/*.ts"`), while
  deliberately nulling `./*.test` and `./*.spec` so test files are never shipped
  (`packages/utils/package.json:12-18`).
- Consuming test files within this repo (all compile-time consumers of `expectType`):
  `packages/utils/src/addEventListener.spec.ts:1`, `packages/utils/src/useControlled.spec.ts:2`,
  `packages/utils/src/store/Store.spec.ts:1`, `packages/utils/src/store/ReactStore.spec.ts:1`,
  `packages/utils/src/store/createSelector.spec.ts:1`.
