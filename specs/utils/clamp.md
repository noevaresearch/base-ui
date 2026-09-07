# `clamp` — behavior spec

Unit: `packages/utils/src/clamp` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/clamp.test.ts` (the only test file for this unit).

## Public API surface (props, parts, subcomponents)

- Single named export `clamp`. No components, no props object, no parts.
- Signature as exercised: `clamp(value, min, max)` — three positional numeric arguments. The
  argument roles are pinned by the assertions: the first argument is the value being clamped and
  the second/third are the lower/upper bounds, since `clamp(1, 2, 4)` returns `2` (the second
  argument) and `clamp(5, 2, 4)` returns `4` (the third argument)
  (`packages/utils/src/clamp.test.ts:6-7`).
- Return value: a number. All asserted returns are the bound values themselves (`2`, `4`, `-1`),
  returned as strict-equal primitives via `toBe` (`packages/utils/src/clamp.test.ts:6-8`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — stateless pure function. No state, defaults, or transitions exist; every call is
independent (`packages/utils/src/clamp.test.ts:5-9`).

## Keyboard interactions

N/A — non-visual numeric utility. No test asserts any keyboard behavior.

## Focus management

N/A — no focus behavior is implemented or asserted by any test.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test.

## DOM structure & portal behavior

N/A — the utility renders nothing, creates no DOM, and performs no portal behavior. It is a pure
numeric computation (`packages/utils/src/clamp.test.ts:5-9`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are emitted, dispatched, or asserted by any test.

## Edge cases (rapid interactions, unmount, nesting)

Proven behaviors:

- Below-minimum clamp: a value lower than `min` returns `min` — `clamp(1, 2, 4)` → `2`
  (`packages/utils/src/clamp.test.ts:6`) and, with negative bounds, `clamp(-5, -1, 5)` → `-1`
  (`packages/utils/src/clamp.test.ts:8`).
- Above-maximum clamp: a value higher than `max` returns `max` — `clamp(5, 2, 4)` → `4`
  (`packages/utils/src/clamp.test.ts:7`).

Unproven behaviors (callers in this repo must not rely on them being specified here):

- In-range passthrough (value strictly between `min` and `max` returned unchanged): UNVERIFIED —
  inferred from `packages/utils/src/clamp.test.ts:5-9`, no test asserts this (all three assertions
  exercise out-of-range values only).
- Boundary equality (value exactly equal to `min` or `max`): UNVERIFIED — inferred from
  `packages/utils/src/clamp.test.ts:5-9`, no test asserts this.
- `NaN` inputs/outputs, `Infinity` bounds, non-number arguments, `min > max` inversion, integer
  coercion, and negative-zero: UNVERIFIED — inferred from `packages/utils/src/clamp.test.ts:5-9`,
  no test asserts any of these.
- Rapid repeated calls, nesting (using `clamp` output as another `clamp` input), or any
  interaction with unmount lifecycle: N/A — pure function with no lifecycle; no test covers
  call-pattern edge cases (`packages/utils/src/clamp.test.ts:5-9`).

## Shared harness dependencies

None. The test file imports only `describe`/`it`/`expect` from `vitest` and `clamp` from its
sibling module (`packages/utils/src/clamp.test.ts:1-2`). There is no `#test-utils` or
`packages/react/test` harness dependency, and no `packages/utils/src/testUtils.ts` usage.
