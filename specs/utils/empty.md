# `empty` — behavior spec

Unit: `packages/utils/src/empty` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/empty.test.ts` (the only test file for this unit).

## Public API surface (props, parts, subcomponents)

- Single module `packages/utils/src/empty` with named exports only; no components, no props, no
  parts, no subcomponents. The tests import `EMPTY_ARRAY` and `EMPTY_OBJECT` by name
  (`packages/utils/src/empty.test.ts:2`).
- `EMPTY_ARRAY`: a module-level singleton array value. Tests prove it is frozen
  (`Object.isFrozen(EMPTY_ARRAY)` → `true`, `packages/utils/src/empty.test.ts:6`) and zero-length
  (`toHaveLength(0)`, `packages/utils/src/empty.test.ts:7`).
- `EMPTY_OBJECT`: a module-level singleton object value. Tests prove it is frozen
  (`Object.isFrozen(EMPTY_OBJECT)` → `true`, `packages/utils/src/empty.test.ts:11`).
- A third export, `NOOP`, exists in the module but no test imports or asserts anything about it:
  UNVERIFIED — inferred from `packages/utils/src/empty.ts:1`, no test asserts this.

## State model (controlled/uncontrolled, defaults, transitions)

N/A — no component state, controlled/uncontrolled duality, or transitions. The exports are
module-level constants shared by every importer; the only state-like property asserted is that
the singletons are frozen, i.e. not meant to be mutated
(`packages/utils/src/empty.test.ts:6`, `packages/utils/src/empty.test.ts:11`).

## Keyboard interactions

N/A — non-visual data utility. No test asserts any keyboard behavior.

## Focus management

N/A — no focus behavior is implemented or asserted by any test.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test.

## DOM structure & portal behavior

N/A — the module renders nothing, creates no DOM, and performs no portal behavior. The whole
suite is pure value assertions (`packages/utils/src/empty.test.ts:4-13`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are emitted, dispatched, or asserted by any test.

## Edge cases (rapid interactions, unmount, nesting)

Proven behaviors:

- Shared-array safety: `EMPTY_ARRAY` is frozen and empty, so it can be handed out as a shared
  fallback value without callers observing per-instance length mutations
  (`packages/utils/src/empty.test.ts:5-8`).
- Shared-object safety: `EMPTY_OBJECT` is frozen (`packages/utils/src/empty.test.ts:10-12`).

Unproven behaviors (downstream stages must not rely on them being specified here):

- Write attempts actually throwing (e.g. mutating `EMPTY_ARRAY` through a widened `T[]` alias, or
  adding properties to `EMPTY_OBJECT`): UNVERIFIED — inferred from
  `packages/utils/src/empty.ts:3-6`, no test asserts a throw; only `Object.isFrozen` is asserted.
- `EMPTY_OBJECT` having zero own properties: UNVERIFIED — inferred from
  `packages/utils/src/empty.ts:8`, no test asserts key count (only frozen-ness is asserted,
  `packages/utils/src/empty.test.ts:11`).
- `NOOP` being callable / returning `undefined`: UNVERIFIED — inferred from
  `packages/utils/src/empty.ts:1`, no test asserts this.
- Identity stability across separate import sites or HMR (same reference everywhere), rapid
  repeated reads, unmount, and nesting: N/A — module constants with no lifecycle; no test covers
  these (`packages/utils/src/empty.test.ts:4-13`).

## Shared harness dependencies

None. The test file imports only `describe`/`it`/`expect` from `vitest` and the unit's own
exports from its sibling module (`packages/utils/src/empty.test.ts:1-2`). There is no
`#test-utils` or `packages/react/test` harness dependency, and no `packages/utils/src/testUtils.ts`
usage.
