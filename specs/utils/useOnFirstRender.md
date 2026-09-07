# `useOnFirstRender` — behavior spec

Unit: `packages/utils/src/useOnFirstRender` (Phase A util → crate `leptos-ui-utils`).
Source of truth: **none.** This unit has no dedicated test file — `testFiles` is empty in the
generated manifest (`ralph/generated/utils.json:342-347`) and no `useOnFirstRender.test.*` /
`.spec.*` file exists anywhere in the repo. Every claim below is therefore source-derived and
UNVERIFIED by definition; nothing in this spec is proven by a test. The hook is exercised
indirectly by component suites that consume it, but those suites are outside this spec's scope
per the Stage 1 mining rules. Stage 3 must not treat this spec as test-proven behavior and
should encode these expectations as its own Rust tests in `crates/leptos-ui-utils` rather than
binding to a reference suite.

`TODO.md` has no `wraps-external:` field for this unit (TODO.md:184-188) — it is original Base
UI code, not a wrapper around a third-party npm package. It is also not in the
`needs-batched-mining` set, so this is a single non-batched spec. The whole implementation is
10 lines (`packages/utils/src/useOnFirstRender.ts:1-10`).

## Public API surface (props, parts, subcomponents)

- Single named export: the hook `useOnFirstRender(fn)`, taking exactly one argument
  (`packages/utils/src/useOnFirstRender.ts:4`).
- The parameter is typed as the bare `Function` interface — no generic parameter, no typing of
  the callback's parameters or return value. UNVERIFIED — inferred from
  `packages/utils/src/useOnFirstRender.ts:4`, no test asserts this.
- Returns nothing (no return statement), so the hook's only observable behavior is the side
  effect of invoking `fn`. UNVERIFIED — inferred from
  `packages/utils/src/useOnFirstRender.ts:4-9`, no test asserts this.
- No props object, no parts, no subcomponents, no context. UNVERIFIED — inferred from
  `packages/utils/src/useOnFirstRender.ts:1-10`, no test asserts this.
- The module is marked `'use client'` (React Server Components boundary). UNVERIFIED — inferred
  from `packages/utils/src/useOnFirstRender.ts:1`, no test asserts this.

## State model (controlled/uncontrolled, defaults, transitions)

- Uncontrolled, fully internal: the hook holds exactly one piece of state — a ref (a mutable
  latch) initialized to `true`, acting as a "has run" flag (`packages/utils/src/useOnFirstRender.ts:5`).
- Transition: on the first execution, the latch is flipped to `false` and then `fn()` is
  invoked. The flip happens BEFORE the call (`packages/utils/src/useOnFirstRender.ts:6-8`).
  Consequence: once `fn` has been invoked, the latch can never return to `true` for this
  component instance. UNVERIFIED — inferred from
  `packages/utils/src/useOnFirstRender.ts:5-9`, no test asserts this.
- Subsequent renders observe `ref.current === false` and do nothing — the hook is a permanent
  no-op after its first run (`packages/utils/src/useOnFirstRender.ts:6-9`). There is no way to
  reset the latch, read it, or re-arm it; no dependencies array exists to trigger a re-run.
  UNVERIFIED — inferred from `packages/utils/src/useOnFirstRender.ts:4-9`, no test asserts this.
- No controlled/uncontrolled prop duality and no external state source. UNVERIFIED — inferred
  from `packages/utils/src/useOnFirstRender.ts:4-9`, no test asserts this.

## Keyboard interactions

N/A — non-visual utility. No keyboard behavior is implemented
(`packages/utils/src/useOnFirstRender.ts:1-10`).

## Focus management

N/A — non-visual utility. No focus behavior is implemented
(`packages/utils/src/useOnFirstRender.ts:1-10`).

## Accessibility (roles, aria-*, id linking)

N/A — non-visual utility. It renders nothing and sets no attributes
(`packages/utils/src/useOnFirstRender.ts:1-10`).

## DOM structure & portal behavior

N/A — the utility creates no DOM, renders no elements, and performs no portal behavior; its
entire surface is a render-time guard plus a callback invocation
(`packages/utils/src/useOnFirstRender.ts:4-9`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — the utility emits no DOM events and registers no listeners. The only "signal" it produces
is the synchronous invocation of the caller-supplied `fn` during the render phase; `fn` receives
no arguments from the hook and the hook discards `fn`'s return value. UNVERIFIED — inferred from
`packages/utils/src/useOnFirstRender.ts:8`, no test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

- Render-phase timing: `fn` runs synchronously during render, not inside `useEffect` /
  `useLayoutEffect`, so it executes before any effect of that same commit and runs even if the
  commit never paints. UNVERIFIED — inferred from
  `packages/utils/src/useOnFirstRender.ts:6-8`, no test asserts this.
- Throw / re-entrancy: because the latch is flipped before `fn` is invoked, if `fn` throws, or
  if `fn` triggers a synchronous re-render of the calling component, the latch is already
  `false` — `fn` is never retried or re-invoked by the hook. UNVERIFIED — inferred from
  `packages/utils/src/useOnFirstRender.ts:7-8`, no test asserts this.
- Changing `fn` identity: later renders ignore the argument entirely; only the `fn` in scope
  during the first executing render is called, and the hook stores no reference to it between
  renders. UNVERIFIED — inferred from `packages/utils/src/useOnFirstRender.ts:4-9`, no test
  asserts this.
- StrictMode double render: the ref object created at line 5 persists across React's StrictMode
  double-invocation of the same fiber's render, so the second pass observes `ref.current ===
  false` and skips — `fn` runs once per mount, not once per render pass. UNVERIFIED — inferred
  from `packages/utils/src/useOnFirstRender.ts:5-8`, no test asserts this.
- Unmount / remount: the latch lives in `useRef`, i.e. per component instance. A remount creates
  a fresh instance with a fresh ref initialized to `true`, so `fn` runs again on the new
  instance; there is no module-level or global latch. UNVERIFIED — inferred from
  `packages/utils/src/useOnFirstRender.ts:5`, no test asserts this.
- Nesting: multiple call sites (or nested components each calling the hook) get independent
  latches; no state is shared between instances. UNVERIFIED — inferred from
  `packages/utils/src/useOnFirstRender.ts:5`, no test asserts this.
- Rapid repeated invocations: N/A — the hook is invoked once per render by React's rules of
  hooks; the latch guarantees at most one `fn` call per instance lifetime regardless of how many
  renders occur. UNVERIFIED — inferred from
  `packages/utils/src/useOnFirstRender.ts:5-9`, no test asserts this.

## Shared harness dependencies

None. There is no test file for this unit at all — the manifest lists `testFiles: []`
(`ralph/generated/utils.json:344`) — so no `#test-utils`, `packages/react/test`, or other shared
harness file is imported by any test of this unit. The module's only import is `react` itself
(`packages/utils/src/useOnFirstRender.ts:2`).
