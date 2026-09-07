# `fastObjectShallowCompare` — behavior spec

Unit: `packages/utils/src/fastObjectShallowCompare` (Phase A util → crate `leptos-ui-utils`).
Source of truth: none — this unit has no test file. The generated unit manifest records
`testFiles: []` for it (`ralph/generated/utils.json:70-76`). Every behavioral claim below is
therefore UNVERIFIED, inferred from the implementation only; Stage 3 must treat this spec as a
description of current behavior, not a proven contract backed by tests.

## Public API surface (props, parts, subcomponents)

- Single module with a single named export `fastObjectShallowCompare`; no components, no props
  objects, no parts, no subcomponents. UNVERIFIED — inferred from
  `packages/utils/src/fastObjectShallowCompare.ts:4`, no test asserts this.
- Signature: called with exactly two positional arguments `a` and `b`, both sharing the type
  parameter `T extends Record<string, any> | null` (i.e. plain objects or `null`). UNVERIFIED —
  inferred from `packages/utils/src/fastObjectShallowCompare.ts:4`, no test asserts this.
- Return value: a boolean on every path — `true` from the identity fast path
  (`packages/utils/src/fastObjectShallowCompare.ts:5-7`), `false` from the non-object guard
  (`packages/utils/src/fastObjectShallowCompare.ts:8-10`), and the boolean key-count comparison
  (`packages/utils/src/fastObjectShallowCompare.ts:31`). UNVERIFIED — inferred from these lines,
  no test asserts this.
- Provenance: the file carries a comment linking the implementation to mui-x's
  `x-internals/fastObjectShallowCompare` (`packages/utils/src/fastObjectShallowCompare.ts:1`).
  This is vendored-source provenance only — the unit's `TODO.md` entry has no `wraps-external:`
  field (`TODO.md:44-48`), so there is no runtime delegation to a third-party npm package and no
  external crate for Stage 3 to bind against; the Rust port must implement the algorithm itself.

## State model (controlled/uncontrolled, defaults, transitions)

N/A — stateless pure function. Each call is independent: no module-level mutable state exists
beyond the one-time `Object.is` alias binding (`packages/utils/src/fastObjectShallowCompare.ts:2`),
and no caching or memoization is performed. UNVERIFIED — inferred from
`packages/utils/src/fastObjectShallowCompare.ts:1-32`, no test asserts this.

## Keyboard interactions

N/A — non-visual comparison utility. No keyboard behavior exists to assert.

## Focus management

N/A — no focus behavior exists to assert.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior exists to assert.

## DOM structure & portal behavior

N/A — the utility renders nothing, creates no DOM, and performs no portal behavior. UNVERIFIED —
inferred from `packages/utils/src/fastObjectShallowCompare.ts:1-32`, no test asserts this.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are emitted or dispatched. UNVERIFIED — inferred from
`packages/utils/src/fastObjectShallowCompare.ts:1-32`, no test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

All of the following are UNVERIFIED — inferred from the cited implementation lines; no test
asserts any of them:

- Identity fast path: if `a === b` the function returns `true` without comparing keys
  (`packages/utils/src/fastObjectShallowCompare.ts:5-7`). Because this check precedes the object
  guard, both operands being `null` also returns `true` here.
- Non-object rejection: if either operand is not an `instanceof Object` (any primitive, or a
  `null` operand that is not `===` to the other), the function returns `false`
  (`packages/utils/src/fastObjectShallowCompare.ts:8-10`). Note `null instanceof Object` is
  `false`, so `fastObjectShallowCompare(null, {})` would return `false`.
- Shallow value comparison with `Object.is` semantics: for each enumerable string key of `a`,
  values are compared via `Object.is` (`packages/utils/src/fastObjectShallowCompare.ts:19`), so
  (if later verified) `NaN` values would compare equal and `+0`/`-0` unequal — unlike `===`.
- Two objects with zero enumerable keys on both sides compare equal via the final key-count
  check (`packages/utils/src/fastObjectShallowCompare.ts:31`). UNVERIFIED — inferred from that
  line, no test asserts this.
- Nested objects/arrays are compared by reference, not deeply: fresh structurally-equal nested
  objects on each side yield `false` unless both sides reference the same object. UNVERIFIED —
  inferred from `packages/utils/src/fastObjectShallowCompare.ts:19`, no test asserts this.
- Key-set asymmetry is rejected in both directions: a key of `a` missing from `b` fails the
  `key in b` check (`packages/utils/src/fastObjectShallowCompare.ts:22-24`), and an extra key
  present only on `b` fails the final key-count equality
  (`packages/utils/src/fastObjectShallowCompare.ts:28-31`).
- Enumeration is `for...in`, so inherited enumerable properties of both operands are counted and
  compared, and the presence check on `b` also consults its prototype chain
  (`packages/utils/src/fastObjectShallowCompare.ts:16-25`). Symbol-keyed and non-enumerable own
  properties are skipped by `for...in` and thus ignored by the comparison. UNVERIFIED — inferred
  from `packages/utils/src/fastObjectShallowCompare.ts:16-30`, no test asserts this.
- Getter side effects: reading `a[key]` and `b[key]` invokes any accessor properties on those
  objects once per compared key, and a `false` result can short-circuit before all getters run.
  UNVERIFIED — inferred from `packages/utils/src/fastObjectShallowCompare.ts:16-25`, no test
  asserts this.
- Arrays and functions would be treated as objects (`instanceof Object` is `true` for both) and
  compared key-by-key / index-by-index rather than rejected. UNVERIFIED — inferred from
  `packages/utils/src/fastObjectShallowCompare.ts:8-31`, no test asserts this.
- Rapid repeated calls, unmount, and self-nesting (using one comparison's result inside another
  call's operands): N/A — pure stateless function with no lifecycle
  (`packages/utils/src/fastObjectShallowCompare.ts:1-32`).

## Shared harness dependencies

None. The unit has no test file at all (`ralph/generated/utils.json:70-76` lists
`testFiles: []`), so no `#test-utils`, `packages/react/test`, or other shared harness file is
involved.
