# `useOnMount` — behavior spec

Unit: `packages/utils/src/useOnMount` (Phase A util → crate `leptos-ui-utils`).
Spec target per `TODO.md`: `specs/utils/useOnMount.md` (`TODO.md:224-228`). The `TODO.md` entry
has no `wraps-external:` field and no `needs-batched-mining` flag — this is original Base UI
code, not a wrapper around a third-party npm package, and the whole unit fits in one mining pass.

Source of truth: **none.** No test file exists for this unit anywhere in the repo — the generated
unit inventory lists `testFiles: []` for `useOnMount` (`ralph/generated/utils.json:350-352`), and
a repo-wide import search finds no test that references `useOnMount`. Every claim below is
therefore UNVERIFIED — inferred from the unit's source and its single direct dependency
(`packages/utils/src/useOnMount.ts`, `packages/utils/src/empty.ts`), with no test asserting it.
Stage 2/3 must treat this spec as a source-derived description of current implementation
behavior, not as test-proven behavior, and should consider adding a
`packages/utils/src/useOnMount.test.tsx` suite before porting.

The unit is a single 13-line file: one hook whose entire body delegates to `React.useEffect` with
a stable empty dependency array (`packages/utils/src/useOnMount.ts:8-12`).

## Public API surface (props, parts, subcomponents)

All UNVERIFIED — no test asserts any of this.

- Single named export: the hook `useOnMount(fn: React.EffectCallback)`. No props object, no parts,
  no subcomponents, no default export, no class (`packages/utils/src/useOnMount.ts:8`).
- The parameter `fn` is typed as React's `EffectCallback` — a function optionally returning a
  cleanup (destructor) function or `undefined` (`packages/utils/src/useOnMount.ts:8`).
- The module is marked `'use client'` (`packages/utils/src/useOnMount.ts:1`).
- Documented intent, from the unit's own JSDoc: "A React.useEffect equivalent that runs once,
  when the component is mounted" (`packages/utils/src/useOnMount.ts:5-7`).
- The only import besides React is `EMPTY_ARRAY` from the sibling `empty` module
  (`packages/utils/src/useOnMount.ts:3`).

## State model (controlled/uncontrolled, defaults, transitions)

All UNVERIFIED — no test asserts any of this.

- Not a state primitive: no controlled/uncontrolled semantics, no defaults, no internal state, no
  transitions owned by the hook. The body is a single pass-through call to `React.useEffect`
  (`packages/utils/src/useOnMount.ts:11`).
- The only "state machine" is React's own effect lifecycle, which the hook adopts wholesale:
  mount → effect body (`fn`) runs once after the commit → if `fn` returned a cleanup, it runs on
  unmount; nothing runs on ordinary re-renders because the dependency list never changes
  (`packages/utils/src/useOnMount.ts:11`, `packages/utils/src/empty.ts:6`).

## Keyboard interactions

N/A — non-visual utility. No keyboard behavior is implemented, and no test asserts any.

## Focus management

N/A — no focus behavior is implemented, and no test asserts any.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented, and no test asserts any.

## DOM structure & portal behavior

N/A — the utility renders nothing, creates no DOM, and performs no portal behavior. It has no
host-environment coupling at all: the entire implementation is one `React.useEffect` call with no
global `window`/`document` access (`packages/utils/src/useOnMount.ts:8-12`). UNVERIFIED — no test
asserts this.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no DOM events are emitted, dispatched, or observed; no bubbling or `preventDefault`
semantics exist. The hook passes `fn` to `React.useEffect` unchanged, and React invokes effect
callbacks with zero arguments — any payload a caller needs must be closed over by `fn` itself
(`packages/utils/src/useOnMount.ts:8`, `packages/utils/src/useOnMount.ts:11`). UNVERIFIED — no
test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

All UNVERIFIED — no test asserts any of this.

- Re-renders never re-run the effect: the dependency list is `EMPTY_ARRAY`, a module-level
  `Object.freeze([])` singleton that is referentially stable for the lifetime of the process
  (`packages/utils/src/useOnMount.ts:11`, `packages/utils/src/empty.ts:6`, rationale at
  `packages/utils/src/empty.ts:5`), so React sees equal deps on every render and skips the effect.
- Identity changes to `fn` are deliberately ignored: because deps never change, React keeps the
  first render's callback and the cleanup it returned; closures captured on later renders are
  never observed. This staleness-by-design is documented in-source — the
  `react-hooks/exhaustive-deps` rule is disabled around the call (`packages/utils/src/useOnMount.ts:10-12`)
  and the TODO notes "no need to put `fn` in the dependency array" once react-compiler linting is
  enabled (`packages/utils/src/useOnMount.ts:9`). Callers that need fresh values must route them
  through a ref (this is why consumers pass `disposeEffect`-style thunks bound to stable
  instances).
- Unmount cleanup: the `React.EffectCallback` contract admits returning a destructor
  (`packages/utils/src/useOnMount.ts:8`), and the hook forwards `fn`'s return value to React
  unchanged (`packages/utils/src/useOnMount.ts:11`), so a returned cleanup function runs when the
  calling component unmounts, and a `fn` that returns `undefined` registers no cleanup.
- StrictMode/double-invoke: the hook adds no guard against React 18+ development StrictMode's
  mount → unmount → remount simulation, so `fn` (and its cleanup) may run twice in that mode —
  plain `React.useEffect` semantics apply untouched (`packages/utils/src/useOnMount.ts:11`).
- Server rendering: the effect body does not run during server rendering — standard
  `React.useEffect` behavior, with the client-only intent additionally signaled by the
  `'use client'` directive (`packages/utils/src/useOnMount.ts:11`,
  `packages/utils/src/useOnMount.ts:1`).
- Nesting / multiple call sites: there is no shared module-level state; each call site of
  `useOnMount` registers its own independent effect slot, so sibling or nested consumers do not
  interfere (`packages/utils/src/useOnMount.ts:8-12`).
- Error propagation: `fn` is invoked directly by React with no try/catch in the hook, so an
  exception thrown by `fn` surfaces through React's normal error handling, not through this unit
  (`packages/utils/src/useOnMount.ts:11`).
- Rapid interactions: N/A in the event sense — the unit observes no input events. The only
  rapid-repetition scenario is rapid re-renders, covered above (deps never change, effect never
  re-runs) (`packages/utils/src/useOnMount.ts:11`, `packages/utils/src/empty.ts:6`).

## Shared harness dependencies

None. There is no test file for this unit at all (`ralph/generated/utils.json:350-352` lists
`testFiles: []`), so no `#test-utils` or `packages/react/test` harness dependency exists for this
unit. When a test suite is added, this section must be revisited.
