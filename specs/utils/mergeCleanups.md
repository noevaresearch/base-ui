# `mergeCleanups` — behavior spec

Unit: `packages/utils/src/mergeCleanups` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/mergeCleanups.test.ts` (the only test file for this unit).

## Public API surface (props, parts, subcomponents)

- Single named export `mergeCleanups`, imported from the sibling module `./mergeCleanups`. No
  components, no props object, no parts, no subcomponents
  (`packages/utils/src/mergeCleanups.test.ts:2`).
- Signature as exercised: variadic — accepts an arbitrary-length argument list mixing cleanup
  functions with falsy placeholder values. The single call passes two functions interleaved with
  `undefined`, `false`, and `null`: `mergeCleanups(first, undefined, false, null, second)`
  (`packages/utils/src/mergeCleanups.test.ts:9`).
- Return value: a callable cleanup function. The result of `mergeCleanups(...)` is immediately
  invoked with zero arguments (`mergeCleanups(...)()`)
  (`packages/utils/src/mergeCleanups.test.ts:9`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — stateless pure combinator. No state, defaults, or transitions exist; the only observable
behavior is the side effects (invocations) performed when the returned function is called
(`packages/utils/src/mergeCleanups.test.ts:5-14`).

## Keyboard interactions

N/A — non-visual utility. No test asserts any keyboard behavior.

## Focus management

N/A — no focus behavior is implemented or asserted by any test.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test.

## DOM structure & portal behavior

N/A — the utility creates no DOM, renders nothing, and performs no portal behavior. It only
composes JavaScript functions (`packages/utils/src/mergeCleanups.test.ts:5-14`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no DOM events are emitted, dispatched, or asserted by any test. The only "invocation"
semantics are direct function calls on the merged cleanup, covered under Public API surface and
Edge cases (`packages/utils/src/mergeCleanups.test.ts:5-14`).

## Edge cases (rapid interactions, unmount, nesting)

Proven behaviors:

- Falsy placeholders are skipped: `undefined`, `false`, and `null` interleaved between real
  cleanups do not break execution — both surrounding cleanup functions still run exactly once
  (`first` and `second` each asserted `toHaveBeenCalledTimes(1)`)
  (`packages/utils/src/mergeCleanups.test.ts:9-12`).
- Each real cleanup is invoked exactly once per call of the merged cleanup
  (`packages/utils/src/mergeCleanups.test.ts:11-12`).
- Order preservation: cleanups run in argument order — the first argument's cleanup is invoked
  before the last argument's cleanup, asserted via
  `first.mock.invocationCallOrder[0] < second.mock.invocationCallOrder[0]`
  (`packages/utils/src/mergeCleanups.test.ts:13`), with the argument order established by the call
  itself (`packages/utils/src/mergeCleanups.test.ts:9`).
- Unmount/nesting/rapid-interaction lifecycle: N/A — the utility has no lifecycle, component
  tree, or timing of its own; no test covers call-pattern edge cases
  (`packages/utils/src/mergeCleanups.test.ts:5-14`).

Unproven behaviors (must not be relied on as specified):

- Argument forwarding: whether arguments passed to the merged cleanup are forwarded to each
  constituent cleanup. UNVERIFIED — inferred from `packages/utils/src/mergeCleanups.test.ts:9`
  (the merged function is only ever invoked with zero arguments), no test asserts forwarding.
- Re-invocation: whether calling the merged cleanup more than once re-runs the cleanups or guards
  against double-invocation. UNVERIFIED — inferred from `packages/utils/src/mergeCleanups.test.ts:9-13`,
  no test asserts this (the merged function is invoked exactly once).
- Throwing cleanups: whether a cleanup that throws prevents later cleanups from running.
  UNVERIFIED — inferred from `packages/utils/src/mergeCleanups.test.ts:5-14`, no test asserts this.
- Truthy non-function values: only the falsy values `undefined`, `false`, `null` are exercised as
  placeholders; whether other non-function values (e.g. `0`, `''`, truthy non-functions) are
  skipped is UNVERIFIED — inferred from `packages/utils/src/mergeCleanups.test.ts:9`, no test
  asserts this.

## Shared harness dependencies

None. The test file imports only `expect`/`vi`/`describe`/`it` from `vitest` and `mergeCleanups`
from its sibling module (`packages/utils/src/mergeCleanups.test.ts:1-2`). There is no
`#test-utils` or `packages/react/test` harness dependency.
