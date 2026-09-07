# `inertValue` — behavior spec

Unit: `packages/utils/src/inertValue` (Phase A util → crate `leptos-ui-utils`).
Source of truth: **none** — this unit has no test file. `ralph/generated/utils.json` lists
`testFiles: []` for it (`ralph/generated/utils.json:126-131`), and a repo-wide search of
`*.test.*` files for `inertValue` returns zero hits. Every behavioral claim below is therefore
UNVERIFIED by tests and inferred from the unit's own source; Stage 3 must not treat this spec as
test-proven behavior and should encode these expectations as its own Rust tests
(`crates/leptos-ui-utils`) rather than binding to a reference suite.

## Public API surface (props, parts, subcomponents)

- Single module `packages/utils/src/inertValue` with one named export: the pure function
  `inertValue(value?: boolean): boolean | undefined` — no components, no props object, no parts,
  no subcomponents (`packages/utils/src/inertValue.ts:3`).
- Input is an optional boolean; the declared return type is `boolean | undefined`
  (`packages/utils/src/inertValue.ts:3`).
- Observed call-site convention (UNVERIFIED — inferred from source, no test asserts this): the
  result is assigned to a DOM element's `inert` attribute/prop, computed from an open/active
  flag, e.g. `inert={inertValue(!open)}` (`packages/react/src/dialog/portal/DialogPortal.tsx:37`,
  `packages/react/src/tabs/panel/TabsPanel.tsx:73`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — a pure, stateless function with no controlled/uncontrolled duality, defaults, or
transitions. The only state-like behavior is a module-load-time snapshot of the React major
version used by the version gate (UNVERIFIED — inferred from
`packages/utils/src/reactVersion.ts:3`, no test asserts this): `parseInt(React.version, 10)` is
evaluated once when the module is first imported, so the branch taken by every call is fixed per
module instance and does not track later changes to `React.version`.

## Keyboard interactions

N/A — non-visual data utility. No keyboard behavior is implemented or asserted anywhere.

## Focus management

N/A — the function computes a value only; any focus-suppression effect of `inert` is delegated to
the browser via the attribute the callers set. No focus behavior is implemented or asserted by
any test.

## Accessibility (roles, aria-*, id linking)

N/A — sets no roles, aria attributes, or id links. Its entire accessibility relevance is that the
value it returns is consumed as the `inert` attribute by callers (UNVERIFIED — inferred from
`packages/react/src/dialog/portal/DialogPortal.tsx:37`, no test asserts this); the semantics of
`inert` itself are native browser behavior outside this unit.

## DOM structure & portal behavior

N/A — creates no DOM, renders nothing, performs no portal behavior.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are emitted, dispatched, or handled.

## Edge cases (rapid interactions, unmount, nesting)

Version-dependent return mapping (all UNVERIFIED — inferred from
`packages/utils/src/inertValue.ts:3-9`, no test asserts this):

- React ≥ 19: input is returned as-is — `inertValue(true)` → `true`, `inertValue(false)` →
  `false`, `inertValue()` → `undefined` (`packages/utils/src/inertValue.ts:4-5`).
- React < 19 (the "compatibility with React < 19" branch,
  `packages/utils/src/inertValue.ts:7-8`): truthy input returns the string `'true'` (type-cast,
  not converted, to `boolean | undefined`), and falsy input (`false` / `undefined`) returns
  `undefined`. Runtime values are therefore `'true' | undefined` on React < 19 despite the
  declared `boolean | undefined` return type — a deliberate type-level lie that Stage 3 should
  not replicate (Leptos has no React version constraint to satisfy).
- Version gate: `isReactVersionAtLeast(19)` compares the module-load-time major version against
  19 with `>=` (`packages/utils/src/reactVersion.ts:3`,
  `packages/utils/src/reactVersion.ts:7-8`), so majors 17 and 18 take the string branch and 19+
  take the passthrough branch.
- Rapid interactions, unmount, and nesting: N/A — a pure function with no lifecycle, no internal
  state, and no DOM involvement; repeated or interleaved calls cannot interfere with each other
  (UNVERIFIED — inferred from `packages/utils/src/inertValue.ts:3-9`, no test asserts this).

## Shared harness dependencies

None. The unit has no test file at all, so no test harness is involved: no `#test-utils` import,
no file under `packages/react/test/`, and no shared utils-test harness. The module's only import
is its sibling `reactVersion` (`packages/utils/src/inertValue.ts:1`).
