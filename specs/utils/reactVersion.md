# `reactVersion` — behavior spec

Unit: `packages/utils/src/reactVersion` (Phase A util → crate `leptos-ui-utils`).
Source of truth: **none** — this unit has no test file. A repo-wide search of `*.test.*` files
for `isReactVersionAtLeast` / `reactVersion` returns zero hits. Every behavioral claim below is
therefore UNVERIFIED by tests and inferred from the unit's own source; Stage 3 must not treat
this spec as test-proven behavior and should encode these expectations as its own Rust tests
(`crates/leptos-ui-utils`) rather than binding to a reference suite.

## Public API surface (props, parts, subcomponents)

- Single module `packages/utils/src/reactVersion` with one named export: the pure predicate
  `isReactVersionAtLeast(reactVersionToCheck: SupportedVersions): boolean` — no components, no
  props object, no parts, no subcomponents
  (`packages/utils/src/reactVersion.ts:7-8`).
- The argument is constrained to the union type `SupportedVersions = 17 | 18 | 19`
  (`packages/utils/src/reactVersion.ts:5`), so callers can only ask about React major versions
  17, 18, or 19; any other major is unrepresentable at the type level (UNVERIFIED — inferred
  from `packages/utils/src/reactVersion.ts:5`, no test asserts this).
- Observed call sites pass only the literal `19` and use the result to gate React-19-only
  behavior: ref extraction (`packages/utils/src/getReactElementRef.ts:15`), the `inert`
  attribute's value shape (`packages/utils/src/inertValue.ts:4`), and whether
  `useSyncExternalStore`'s native "get version" semantics can be relied on
  (`packages/utils/src/store/useStore.ts:12`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — a pure, stateless function with no controlled/uncontrolled duality, defaults, or
transitions. The only state-like behavior is a module-load-time snapshot: `majorVersion` is
computed once when the module is first imported as `parseInt(React.version, 10)`
(`packages/utils/src/reactVersion.ts:3`). Consequences (all UNVERIFIED — inferred from
`packages/utils/src/reactVersion.ts:3`, no test asserts this):

- Every call reads the same frozen value; the predicate's answers are fixed per module instance
  and do not track later mutations of `React.version` (which React does not perform in practice).
- `parseInt` truncates at the first non-numeric segment, so a real React version string such as
  `'19.1.0'` yields major `19`.
- If `React.version` were missing or unparseable, `parseInt` yields `NaN`; because all `>=`
  comparisons with `NaN` are `false`, the predicate would then return `false` for every input,
  i.e. fail closed toward the pre-19 code paths.

## Keyboard interactions

N/A — non-visual, non-DOM utility. No keyboard behavior is implemented or asserted anywhere.

## Focus management

N/A — the function computes a boolean only and has no focus involvement. Its only indirect
focus relevance is that callers use its result to decide attribute/prop shapes that affect
inertness (`packages/utils/src/inertValue.ts:4`), which is outside this unit.

## Accessibility (roles, aria-*, id linking)

N/A — sets no roles, aria attributes, or id links, and renders nothing.

## DOM structure & portal behavior

N/A — creates no DOM, renders nothing, performs no portal behavior.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are emitted, dispatched, listened for, or handled.

## Edge cases (rapid interactions, unmount, nesting)

- Comparison semantics (UNVERIFIED — inferred from
  `packages/utils/src/reactVersion.ts:7-8`, no test asserts this): the check is
  `majorVersion >= reactVersionToCheck`, i.e. inclusive of the requested major. With
  `SupportedVersions` restricted to 17/18/19: requesting `17` returns `true` on every supported
  React major; requesting `18` returns `true` on 18 and 19, `false` on 17; requesting `19`
  returns `true` only on 19+ and `false` on 17 and 18 — this last case is the one all three
  observed call sites depend on
  (`packages/utils/src/getReactElementRef.ts:15`, `packages/utils/src/inertValue.ts:4`,
  `packages/utils/src/store/useStore.ts:12`).
- Snapshot-vs-live timing (UNVERIFIED — inferred from
  `packages/utils/src/reactVersion.ts:3`, no test asserts this): because the major version is
  captured at module load, the predicate would misreport if a host application somehow swapped
  to a different React major after first import (e.g. dual React instances where this module is
  bound to an older copy); there is no re-validation or per-call re-read of `React.version`.
- Rapid interactions, unmount, and nesting: N/A — a pure function with no lifecycle, no internal
  mutable state, and no DOM involvement; repeated or interleaved calls cannot interfere with
  each other (UNVERIFIED — inferred from `packages/utils/src/reactVersion.ts:7-8`, no test
  asserts this).

## Shared harness dependencies

None. The unit has no test file at all, so no test harness is involved: no `#test-utils` import,
no file under `packages/react/test/`, and no shared utils-test harness. The module's only import
is `react` itself (`packages/utils/src/reactVersion.ts:1`).
