# `error` — behavior spec

Unit: `packages/utils/src/error` (Phase A util → crate `leptos-ui-utils`).
Source of truth: **none** — this unit has no test file. `ralph/generated/utils.json` lists
`testFiles: []` for this unit, and no `error.test.ts` / `error.spec.ts` exists under
`packages/utils/src/`. Every behavioral claim below is therefore marked UNVERIFIED per the
mining rules; nothing here is proven by a test.

## Public API surface (props, parts, subcomponents)

- Module with two named exports and no default export:
  - `error` — a preconfigured logger instance created by calling
    `createLogOnce('error', 'Base UI')` at module scope
    (`packages/utils/src/error.ts:3`). UNVERIFIED — inferred from
    `packages/utils/src/error.ts:3`, no test asserts this export or its call shape.
  - `reset` — re-exported from `./createLogOnce` (`packages/utils/src/error.ts:5`).
    UNVERIFIED — inferred from `packages/utils/src/error.ts:5`, no test asserts this
    re-export.
- No props, no parts, no subcomponents — it is a plain function-object export, not a React
  component. UNVERIFIED — inferred from `packages/utils/src/error.ts:1-5`, no test asserts
  this.
- Publicly reachable as `@base_ui/utils/error` via the package wildcard export
  `"./*": "./src/*.ts"` (`packages/utils/package.json:15-17`; test entrypoints are explicitly
  nulled). UNVERIFIED as a consumer-facing contract — inferred from
  `packages/utils/package.json:15-17`, no test asserts the resolution.
- Delegation note (Stage 3 binding): the unit's entire algorithm is delegated to the sibling
  in-repo util `createLogOnce` (`packages/utils/src/error.ts:1-3`); the only unit-local logic
  is fixing the channel argument to `'error'` and the prefix to `'Base UI'`. The Rust
  equivalent should bind `error` as a thin instance/wrapper over the ported `createLogOnce`
  (behavior specified in `specs/utils/createLogOnce.md` and mined from its own tests there) —
  do not re-derive the once-logging algorithm from this unit; no test in this unit constrains
  it.

## State model (controlled/uncontrolled, defaults, transitions)

N/A as a controlled/uncontrolled component concept. The unit has one state-bearing aspect:
`error` is a module-level singleton `const` created once at import time
(`packages/utils/src/error.ts:3`), so any internal once-per-message dedup state held by the
`createLogOnce` instance persists for the lifetime of the module registry. UNVERIFIED —
inferred from `packages/utils/src/error.ts:3`, no test asserts singleton identity or state
persistence across imports or across calls. No defaults or transitions are testable from this
unit.

## Keyboard interactions

N/A — non-visual logging utility. No test asserts any keyboard behavior (no test file exists
for this unit).

## Focus management

N/A — non-visual logging utility. No focus behavior is implemented
(`packages/utils/src/error.ts:1-5`).

## Accessibility (roles, aria-*, id linking)

N/A — non-visual logging utility. No accessibility behavior is implemented or asserted by any
test.

## DOM structure & portal behavior

N/A — the utility renders nothing, creates no DOM, and performs no portal behavior
(`packages/utils/src/error.ts:1-5`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no DOM events are emitted, dispatched, or asserted by any test. The unit's only
observable output is whatever logging `createLogOnce('error', 'Base UI')` performs when the
returned function is invoked with a message (`packages/utils/src/error.ts:3`). UNVERIFIED —
inferred from `packages/utils/src/error.ts:3`, no test asserts the call signature, the console
destination, the `'Base UI'` prefix rendering, or the once-per-message suppression; those
aspects belong to `specs/utils/createLogOnce.md`.

## Edge cases (rapid interactions, unmount, nesting)

- Rapid repeated calls: UNVERIFIED — inferred from `packages/utils/src/error.ts:3`, no test
  asserts whether repeat calls with the same message are suppressed (the `createLogOnce`
  contract) or how distinct messages behave.
- Unmount lifecycle: N/A — no React lifecycle exists in this unit
  (`packages/utils/src/error.ts:1-5`); it is import-time initialized.
- Nesting: N/A — nothing to nest; the unit exposes a logging function, not a component.
- `reset` interaction with the shared `error` instance: UNVERIFIED — inferred from
  `packages/utils/src/error.ts:5`, no test asserts that calling the re-exported `reset`
  clears the dedup state of the `error` singleton (or whether `reset` is instance-scoped or
  global), though the re-export originates from the same module that constructs the instance.
- Sole in-repo consumer: `packages/utils/src/useControlled.ts:5` imports `error` from
  `./error` — citation recorded as a consumption fact, not a behavior proof
  (`packages/utils/src/useControlled.ts:5`).

## Shared harness dependencies

None. No test files exist for this unit, so no `#test-utils`, `packages/react/test`, or
`packages/utils/src/testUtils.ts` dependency is possible. Should a test file be added later at
`packages/utils/src/error.test.ts`, this spec must be re-mined.
