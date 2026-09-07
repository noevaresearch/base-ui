# `useValueAsRef` — behavior spec

Unit: `packages/utils/src/useValueAsRef` (Phase A util → crate `leptos-ui-utils`, per
TODO.md:219-223; no `wraps-external:` field is present, so this is original Base UI code, not a
wrapper around a third-party npm package).

Source of truth: **none.** This unit has no dedicated test file — `testFiles` is empty in the
generated manifest (`ralph/generated/utils.json:399-406`, `ralph/generated/utils.json:402`) and no
`useValueAsRef.test.*` / `.spec.*` file exists anywhere in the repo. Every claim below is
therefore source-derived and UNVERIFIED by definition; nothing in this spec is proven by a test.
The hook is exercised indirectly by component suites that consume it, but those suites are
outside this spec's scope per the Stage 1 mining rules. Stage 3 must not treat this spec as
test-proven behavior and should encode these expectations as its own Rust tests in
`crates/leptos-ui-utils` rather than binding to a reference suite.

It is also not in the `needs-batched-mining` set (TODO.md lists only combobox, drawer,
floating-ui-react, menu, number-field, select there), so this is a single non-batched spec. The
whole implementation is 30 lines (`packages/utils/src/useValueAsRef.ts:1-30`).

## Public API surface (props, parts, subcomponents)

- Single named export: the hook `useValueAsRef<T>(value: T)` — one generic type parameter, one
  positional argument, no options object. UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:10`, no test asserts this.
- Returns a single mutable object (not a tuple, not a state pair) whose identity is stable across
  renders of the calling component. UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:11-18`, no test asserts this.
- The returned object has exactly three own properties: `current: T`, `next: T`, and
  `effect: () => void`. UNVERIFIED — inferred from `packages/utils/src/useValueAsRef.ts:22-28`,
  no test asserts this.
- `effect` is part of the returned surface, not a private closure: consumers receive it and could
  invoke it manually (it performs the same `next` → `current` copy). UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:22-28`, no test asserts this.
- Documented contract (JSDoc): "Untracks the provided value by turning it into a ref to remove
  its reactivity" and "access the passed value inside `React.useEffect` without causing the
  effect to re-run when the value changes". UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:5-9`, no test asserts this.
- No props object, no parts, no subcomponents, no context, no render output. The module is
  marked `'use client'` (React Server Components boundary). UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:1-19`, no test asserts this.

## State model (controlled/uncontrolled, defaults, transitions)

- Not a controlled/uncontrolled API at all: the hook holds no React state (no `useState`) — all
  state lives in a single ref-allocated object created on the first render via
  `useRefWithInit(createLatestRef, value)`. UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:11`, no test asserts this.
- The factory produces `{ current: value, next: value, effect }`, so on the first render
  `current === next === value` (the initial argument). UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:21-30`, no test asserts this.
- Init-once semantics are delegated to `useRefWithInit` (a sentinel-based ref whose
  initialization function runs at most once, on first render — see
  `packages/utils/src/useRefWithInit.ts:16-20` and its own spec at specs/utils/useRefWithInit.md).
  Consequence: the object is created at most once per component instance and its identity never
  changes. UNVERIFIED — inferred from `packages/utils/src/useValueAsRef.ts:11`, no test asserts
  this.
- Transition per render: during the render phase the hook synchronously writes the newest
  `value` into `latest.next` (`packages/utils/src/useValueAsRef.ts:13`). This happens on every
  render, including the first (where it is redundant with the factory).
  UNVERIFIED — inferred from `packages/utils/src/useValueAsRef.ts:13`, no test asserts this.
- Transition per commit: an effect with no dependency array copies `latest.current = latest.next`
  after every commit (`packages/utils/src/useValueAsRef.ts:15-16`,
  `packages/utils/src/useValueAsRef.ts:25-27`). The disabled `react-hooks/exhaustive-deps` lint
  rule on line 15 is evidence the always-run sync is intentional. UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:15-16`, no test asserts this.
- One-commit lag: during the render that introduces a new `value`, `current` still holds the
  previous commit's value; only after that commit's layout effect does `current` catch up to
  `next`. `next` is therefore "the value as of the latest render" and `current` is "the value as
  of the latest commit". UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:13-27`, no test asserts this.
- On the client the sync runs as `React.useLayoutEffect` (`packages/utils/src/useIsoLayoutEffect.ts:6`),
  i.e. after DOM mutation but before paint and before passive effects of the same commit — this
  is the mechanism behind the JSDoc guarantee that a consumer's `useEffect` can read the newest
  value without re-running. UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:16` and `packages/utils/src/useIsoLayoutEffect.ts:6`, no
  test asserts this.
- No reset, no dependency array, no way to opt out of the per-commit sync, no equality check
  gating either transition. UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:10-19`, no test asserts this.

## Keyboard interactions

N/A — non-visual utility. No keyboard behavior is implemented
(`packages/utils/src/useValueAsRef.ts:1-30`).

## Focus management

N/A — non-visual utility. No focus behavior is implemented
(`packages/utils/src/useValueAsRef.ts:1-30`).

## Accessibility (roles, aria-*, id linking)

N/A — non-visual utility. It renders nothing and sets no attributes
(`packages/utils/src/useValueAsRef.ts:1-30`).

## DOM structure & portal behavior

N/A — the utility creates no DOM, renders no elements, and performs no portal behavior; it is
value-type agnostic and never inspects the DOM (`packages/utils/src/useValueAsRef.ts:1-30`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — the utility emits no DOM events and registers no listeners. The only callable behavior it
exposes besides reading `current`/`next` is the returned `effect()` function, which when invoked
(automatically per commit, or manually by a consumer) performs the `next` → `current` copy with
no arguments and no return value. UNVERIFIED — inferred from
`packages/utils/src/useValueAsRef.ts:25-27`, no test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

- Rapid successive value changes: `next` is overwritten on each render and the sync effect copies
  once per commit, so intermediate values collapse — `current` ends at the last rendered value,
  and values from renders that never commit are never observable in `current`.
  UNVERIFIED — inferred from `packages/utils/src/useValueAsRef.ts:13-27`, no test asserts this.
- Value changes without a re-render: nothing changes; the hook only runs during render, so
  mutating the underlying value externally has no effect on `current`/`next`.
  UNVERIFIED — inferred from `packages/utils/src/useValueAsRef.ts:10-19`, no test asserts this.
- No referential-equality shortcut: `latest.next = value` is executed unconditionally every
  render, even when `value` is referentially identical, and new object/array identities each
  render are simply stored (the hook never compares values). UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:13`, no test asserts this.
- StrictMode double render: the ref-allocated object persists across React's StrictMode
  double-invocation of the same fiber's render, so the factory runs once and the redundant
  `next = value` write is idempotent. UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:11-13`, no test asserts this.
- Concurrent/discarded renders: `next` is mutated during render (intentional render-phase
  mutation — it is the mechanism of the hook), so a discarded concurrent render can leave `next`
  pointing at a value that was never committed until the next render overwrites it or the
  committed effect syncs. Render output is not pure with respect to `next`.
  UNVERIFIED — inferred from `packages/utils/src/useValueAsRef.ts:13`, no test asserts this.
- Non-DOM environments: `useIsoLayoutEffect` resolves to a `noop` when `document` is undefined
  (`packages/utils/src/useIsoLayoutEffect.ts:6`), so in such environments the per-commit sync
  never runs and `current` stays at the initial value after mount while only `next` tracks
  updates. UNVERIFIED — inferred from `packages/utils/src/useValueAsRef.ts:16` and
  `packages/utils/src/useIsoLayoutEffect.ts:6`, no test asserts this.
- Unmount: no cleanup function is registered and nothing is torn down; the object simply becomes
  garbage retaining its last `current`/`next` values. UNVERIFIED — inferred from
  `packages/utils/src/useValueAsRef.ts:15-18`, no test asserts this.
- Remount: a fresh component instance allocates a fresh object with `current`/`next` seeded from
  the new initial value; there is no module-level or cross-instance state.
  UNVERIFIED — inferred from `packages/utils/src/useValueAsRef.ts:11` and
  `packages/utils/src/useValueAsRef.ts:21-30`, no test asserts this.
- Nesting / multiple call sites: each call site gets an independent object; no state is shared
  between instances. UNVERIFIED — inferred from `packages/utils/src/useValueAsRef.ts:11`, no test
  asserts this.

## Shared harness dependencies

None. There is no test file for this unit at all — the manifest lists `testFiles: []`
(`ralph/generated/utils.json:399-406`) — so no `#test-utils`, `packages/react/test`, or other
shared harness file is imported by any test of this unit. The module's only imports are two
sibling utils, `useIsoLayoutEffect` (`packages/utils/src/useValueAsRef.ts:2`,
`packages/utils/src/useIsoLayoutEffect.ts:1-6`) and `useRefWithInit`
(`packages/utils/src/useValueAsRef.ts:3`, `packages/utils/src/useRefWithInit.ts:1-23`), each of
which has its own spec.
