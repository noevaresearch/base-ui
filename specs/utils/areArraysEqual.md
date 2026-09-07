# `areArraysEqual` — behavior spec

Unit: `packages/utils/src/areArraysEqual` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/areArraysEqual.test.ts` (the only test file for this unit).

## Public API surface (props, parts, subcomponents)

- Single named export `areArraysEqual`. No components, no props object, no parts.
- Signature as exercised: `areArraysEqual(a, b, itemComparer?)` where `a` and `b` are arrays and
  `itemComparer` is an optional `(x, y) => boolean` callback
  (`packages/utils/src/areArraysEqual.test.ts:38-43`).
- Return value: a boolean indicating array equality
  (`packages/utils/src/areArraysEqual.test.ts:5-15`).
- Third parameter is optional: every comparison without custom semantics is made in 2-argument
  calls (`packages/utils/src/areArraysEqual.test.ts:5-36`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — pure synchronous function with no state. Default comparison semantics: items are compared
with `Object.is`-style equality (`packages/utils/src/areArraysEqual.test.ts:26-32`), and
structurally identical-but-distinct objects are NOT equal by default
(`packages/utils/src/areArraysEqual.test.ts:34-36`).

## Keyboard interactions

N/A — non-visual utility, no keyboard behavior is exercised.

## Focus management

N/A — non-visual utility; no focus behavior is exercised.

## Accessibility (roles, aria-*, id linking)

N/A — non-visual utility; no DOM or ARIA behavior is exercised.

## DOM structure & portal behavior

N/A — non-visual utility; renders nothing and performs no DOM work.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — emits no events. The only callback it invokes is the caller-supplied item comparer, which
receives one element from each array per comparison. Its result is authoritative: when the
comparer returns `false` for every pairwise call, `areArraysEqual` returns `false` even if both
arguments are the identical array reference (`packages/utils/src/areArraysEqual.test.ts:49-55`).

## Edge cases (rapid interactions, unmount, nesting)

- Same-order, same-element arrays are equal: `areArraysEqual([1, 2, 3], [1, 2, 3]) → true`
  (`packages/utils/src/areArraysEqual.test.ts:5-7`).
- Any differing element makes the arrays unequal: `[1, 2, 3]` vs `[1, 2, 4] → false`
  (`packages/utils/src/areArraysEqual.test.ts:9-11`).
- Length mismatch alone makes the arrays unequal, without needing element-level divergence:
  `[1, 2, 3]` vs `[1, 2] → false` (`packages/utils/src/areArraysEqual.test.ts:13-15`).
- Passing the identical array reference as both arguments returns `true` under default semantics
  (`packages/utils/src/areArraysEqual.test.ts:17-20`). This is not an unconditional fast path:
  the supplied comparer is still consulted for same-reference inputs and can force a `false`
  result (`packages/utils/src/areArraysEqual.test.ts:49-55`).
- Two empty arrays are equal (`packages/utils/src/areArraysEqual.test.ts:22-24`).
- `NaN` compares equal to `NaN` (Object.is semantics): `areArraysEqual([NaN], [NaN]) → true`
  (`packages/utils/src/areArraysEqual.test.ts:26-28`).
- `0` and `-0` compare unequal (Object.is semantics): `areArraysEqual([0], [-0]) → false`
  (`packages/utils/src/areArraysEqual.test.ts:30-32`).
- Default item comparison is by reference: `[ { id: 1 } ]` vs a separate `[ { id: 1 } ] → false`
  (`packages/utils/src/areArraysEqual.test.ts:34-36`).
- The provided comparer fully replaces the default equality for item comparisons:
  `(x, y) => x.id === y.id` makes structurally-equal object arrays compare equal
  (`packages/utils/src/areArraysEqual.test.ts:38-43`), and a looser `===` comparer can override
  the default Object.is semantics (making `[0]` and `[-0]` equal)
  (`packages/utils/src/areArraysEqual.test.ts:45-47`).
- UNVERIFIED — inferred from `packages/utils/src/areArraysEqual.test.ts:34-43`, no test asserts
  how nested arrays or deeper object graphs are compared beyond one level of custom comparer.

## Shared harness dependencies

None. The test file imports only Vitest and the local `./areArraysEqual` module; it uses no
`#test-utils` or files under `packages/react/test/`
(`packages/utils/src/areArraysEqual.test.ts:1-2`).
