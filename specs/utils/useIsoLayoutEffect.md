# `useIsoLayoutEffect` — behavior spec

Unit: `packages/utils/src/useIsoLayoutEffect` (Phase A util → crate `leptos-ui-utils`).
Source of truth: **none.** This unit has no dedicated test file — `testFiles` is empty in the
generated manifest (`ralph/generated/utils.json:324-330`) and no `useIsoLayoutEffect.test.*` /
`.spec.*` file exists next to the source. Every claim below is therefore source-derived and
UNVERIFIED by definition; nothing in this spec is proven by a test. Consumer suites that call
this hook inside their own test components (checkbox, dialog, drawer, toast, etc.) are outside
this spec's scope per the Stage 1 mining rules.

`TODO.md` has no `wraps-external:` field for this unit (`TODO.md:174-178`) — it is original Base
UI code, not a wrapper around a third-party npm package. It is also not in the
`needs-batched-mining` set, so this is a single non-batched spec.

## Public API surface (props, parts, subcomponents)

- Single named export: `useIsoLayoutEffect`. It is exported as a module-level constant, not a
  function declaration — the binding happens once when the module is evaluated
  (`packages/utils/src/useIsoLayoutEffect.ts:6`).
- The exported value is the result of a ternary on `typeof document !== 'undefined'`:
  `React.useLayoutEffect` when a `document` global exists, otherwise a module-local `noop`
  (`packages/utils/src/useIsoLayoutEffect.ts:6`).
- Browser binding: identical to `React.useLayoutEffect` — a full passthrough with no wrapper
  logic, so it accepts an effect callback plus React's optional dependency-array argument and
  follows React's effect contract (callback may return a cleanup function).
  UNVERIFIED — inferred from `packages/utils/src/useIsoLayoutEffect.ts:6`, no test asserts this.
- Server/non-DOM binding: the private module-local `noop` (an empty arrow function, not exported)
  (`packages/utils/src/useIsoLayoutEffect.ts:4`), so callers may invoke the hook with the same
  signature on the server. UNVERIFIED — inferred from
  `packages/utils/src/useIsoLayoutEffect.ts:4-6`, no test asserts this.
- No props object, no parts, no subcomponents, no context. The module is marked `'use client'`
  (React Server Components boundary). UNVERIFIED — inferred from
  `packages/utils/src/useIsoLayoutEffect.ts:1`, no test asserts this.

## State model (controlled/uncontrolled, defaults, transitions)

- Stateless utility: the unit holds no state of its own; in the browser it *is* React's
  `useLayoutEffect`, and on the server it is a no-op. Controlled/uncontrolled semantics and
  defaults: N/A.
- The only "transition" is the environment selection, which is evaluated exactly once at module
  import time and frozen for the lifetime of the process — it is never re-evaluated per render or
  per call, so the binding cannot change at runtime. UNVERIFIED — inferred from
  `packages/utils/src/useIsoLayoutEffect.ts:6`, no test asserts this.

## Keyboard interactions

N/A — non-visual utility. No keyboard behavior is implemented
(`packages/utils/src/useIsoLayoutEffect.ts:1-6`).

## Focus management

N/A — non-visual utility. No focus behavior is implemented
(`packages/utils/src/useIsoLayoutEffect.ts:1-6`).

## Accessibility (roles, aria-*, id linking)

N/A — non-visual utility. It renders nothing and sets no attributes
(`packages/utils/src/useIsoLayoutEffect.ts:1-6`).

## DOM structure & portal behavior

N/A — the utility creates no DOM, renders no elements, and performs no portal behavior. Its only
host-environment coupling is an existence check on the global `document` to pick the effect
implementation (`packages/utils/src/useIsoLayoutEffect.ts:6`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — the utility emits no DOM events and registers no event listeners itself. In the browser it
defers entirely to React's layout-effect lifecycle (synchronous after DOM mutations, before
paint); on the server the callback is simply discarded. UNVERIFIED — inferred from
`packages/utils/src/useIsoLayoutEffect.ts:4-6`, no test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

- SSR / non-DOM environments: with `typeof document === 'undefined'`, the hook resolves to `noop`,
  the standard suppression for React's "`useLayoutEffect` does nothing on the server" warning when
  a component using it is server-rendered or rendered in a non-DOM test environment.
  UNVERIFIED — inferred from `packages/utils/src/useIsoLayoutEffect.ts:6`, no test asserts this.
- Module-evaluation-time binding: the `typeof document` check runs once at first import. If a
  `document` global is injected (or removed) after the module has been evaluated, the binding does
  not switch. UNVERIFIED — inferred from `packages/utils/src/useIsoLayoutEffect.ts:6`, no test
  asserts this.
- Unmount/cleanup: in the browser, cleanup-function semantics (callback return value invoked on
  unmount or before re-run) are whatever `React.useLayoutEffect` provides — the unit adds no
  wrapper. On the server, the callback and any returned cleanup are silently discarded by `noop`.
  UNVERIFIED — inferred from `packages/utils/src/useIsoLayoutEffect.ts:4-6`, no test asserts this.
- Dependency-array handling: the unit performs no normalization, comparison, or defaulting of
  deps; behavior is exactly React's (missing array ⇒ run every render). UNVERIFIED — inferred
  from `packages/utils/src/useIsoLayoutEffect.ts:6`, no test asserts this.
- Rapid invocation / nesting: N/A — the unit has no tree presence and no internal accounting, so
  repeated or nested calls are just ordinary React effect scheduling
  (`packages/utils/src/useIsoLayoutEffect.ts:1-6`).

## Shared harness dependencies

None. There is no test file for this unit at all, so no `#test-utils`, `packages/react/test`, or
other shared harness file is imported by any test of this unit.
