# `useStableCallback` — behavior spec

Unit: `packages/utils/src/useStableCallback` (Phase A util → crate `leptos-ui-utils`).
Source of truth: **none.** This unit has no dedicated test file — `testFiles` is empty in the
generated manifest (`ralph/generated/utils.json:383-390`) and no `useStableCallback.test.*` /
`.spec.*` file exists anywhere in the repo. Every claim below is therefore source-derived and
UNVERIFIED by definition; nothing in this spec is proven by a test. Exactly one consumer suite
imports the hook today (`packages/react/src/internals/useAnimationsFinished.test.tsx:5`), but
consumer suites that call this hook inside their own test components are outside this spec's
scope per the Stage 1 mining rules.

`TODO.md` has no `wraps-external:` field for this unit (`TODO.md:242-246`) — it is original Base
UI code, not a wrapper around a third-party npm package. It is also not in the
`needs-batched-mining` set, so this is a single non-batched spec.

## Public API surface (props, parts, subcomponents)

- Single named export: `useStableCallback<T extends Callback>(callback: T | undefined): T`, a
  function declaration with one generic parameter bounded by the module-local
  `type Callback = (...args: any[]) => any` (`packages/utils/src/useStableCallback.ts:14`,
  `packages/utils/src/useStableCallback.ts:35`). UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:14-35`, no test asserts this.
- The parameter is nullable: callers may pass `T | undefined`, while the return type stays `T`
  (`packages/utils/src/useStableCallback.ts:35`). UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:35`, no test asserts this.
- Documented contract (JSDoc, `packages/utils/src/useStableCallback.ts:25-34`): the returned
  function is "always the same between renders"; it is non-reactive to any values it captures
  (safe to list as a dependency of `React.useMemo`/`React.useEffect` without re-triggering them);
  it must only be called inside effects and event handlers, never during render (which throws an
  error); and it is "a more permissive version of React 19.2's `React.useEffectEvent`" in that it
  can be passed through contexts and called in event handler props, not just effects. UNVERIFIED —
  inferred from `packages/utils/src/useStableCallback.ts:25-34`, no test asserts this.
- No props object, no parts, no subcomponents, no context. The module is marked `'use client'`
  (React Server Components boundary). UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:1`, no test asserts this.
- The hook is built on two sibling utils: `SafeReact` (for the effect-primitive references) and
  `useRefWithInit` (for the once-per-instance record). Its behavior therefore depends on
  `SafeReact.useInsertionEffect` resolving through the safe-React indirection rather than the
  global React import. UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:2-5`, no test asserts this.

## State model (controlled/uncontrolled, defaults, transitions)

- Not a stateful component: controlled/uncontrolled semantics and user-facing defaults are N/A.
  What it has instead is a per-hook-instance, two-slot internal record created exactly once via
  `useRefWithInit(createStableCallback).current`
  (`packages/utils/src/useStableCallback.ts:36`).
- The record's shape (`packages/utils/src/useStableCallback.ts:16-23`): `next` — "The next value
  for callback"; `callback` — "The function to be called by trampoline. This must fail during the
  initial render phase."; `trampoline` — the stable function handed back to the caller; `effect` —
  the promotion step. UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:16-23`, no test asserts this.
- Initial state: `next` is `undefined` and `callback` is the module-local `assertNotCalled`
  sentinel (`packages/utils/src/useStableCallback.ts:43-45`). UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:42-52`, no test asserts this.
- Per-render transition: `stable.next = callback` is assigned unconditionally on every render
  (overwriting the previous value, including with `undefined` when the hook is called with
  `undefined`) (`packages/utils/src/useStableCallback.ts:37`). UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:37`, no test asserts this.
- Per-commit transition: the scheduled insertion effect promotes `callback = next`, making the
  latest render's function the one the trampoline dispatches to
  (`packages/utils/src/useStableCallback.ts:38`, `packages/utils/src/useStableCallback.ts:47-49`).
  UNVERIFIED — inferred from `packages/utils/src/useStableCallback.ts:47-49`, no test asserts this.
- The `trampoline` is created once inside `createStableCallback` and is never reassigned, so the
  reference returned to the caller (`packages/utils/src/useStableCallback.ts:39`) is identical for
  the lifetime of the hook instance regardless of how the wrapped function's identity changes.
  UNVERIFIED — inferred from `packages/utils/src/useStableCallback.ts:46-52`, no test asserts this.
- Effect-primitive selection is itself module-level frozen state: `useSafeInsertionEffect` is
  `SafeReact.useInsertionEffect` only if it exists (React 17 has none) *and* it is not the same
  function as `SafeReact.useLayoutEffect` (Preact aliases it that way and "fires too late");
  otherwise the fallback is the identity `(fn) => fn()`, which runs the promotion synchronously
  during render (`packages/utils/src/useStableCallback.ts:5-12`). The check is evaluated once at
  module evaluation and never re-run. UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:5-12`, no test asserts this.

## Keyboard interactions

N/A — non-visual utility. It registers no key handlers and has no keyboard behavior
(`packages/utils/src/useStableCallback.ts:1-62`).

## Focus management

N/A — non-visual utility. It renders nothing and manipulates no focus
(`packages/utils/src/useStableCallback.ts:1-62`).

## Accessibility (roles, aria-*, id linking)

N/A — non-visual utility. It emits no DOM, sets no attributes, and owns no ids
(`packages/utils/src/useStableCallback.ts:1-62`).

## DOM structure & portal behavior

N/A — the utility creates no DOM nodes and performs no portal behavior. Its only
host-environment coupling is React hook scheduling (insertion effect vs. synchronous fallback)
(`packages/utils/src/useStableCallback.ts:5-12`, `packages/utils/src/useStableCallback.ts:38`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- The unit emits no events and registers no DOM event listeners itself; it *wraps* callbacks that
  consumers use as event handlers (`packages/utils/src/useStableCallback.ts:35-40`).
  UNVERIFIED — inferred from `packages/utils/src/useStableCallback.ts:35-52`, no test asserts this.
- Dispatch shape: the trampoline forwards **all** call arguments positionally to the currently
  promoted callback via rest/spread — `(...args) => stable.callback?.(...args)` — and returns
  whatever that callback returns (`packages/utils/src/useStableCallback.ts:46`). No argument
  mutation, wrapping, or prevention semantics are applied by the unit. UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:46`, no test asserts this.
- If the promoted callback is `undefined` (the hook was last called with `undefined` and the
  promotion already ran), the trampoline call is a silent no-op returning `undefined`
  (`packages/utils/src/useStableCallback.ts:46`, `packages/utils/src/useStableCallback.ts:47-49`).
  UNVERIFIED — inferred from `packages/utils/src/useStableCallback.ts:46`, no test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

- **Render-phase call on the initial render (development):** before any insertion effect has run,
  `callback` is the `assertNotCalled` sentinel, which throws
  `new Error('Base UI: Cannot call an event handler while rendering.')` when
  `process.env.NODE_ENV !== 'production'` (`packages/utils/src/useStableCallback.ts:43-45`,
  `packages/utils/src/useStableCallback.ts:54-62`). UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:54-62`, no test asserts this.
- **Render-phase call in production:** the sentinel's body is guarded by the `NODE_ENV` check, so
  in production it does nothing and the call degrades to a no-op returning `undefined` instead of
  throwing (`packages/utils/src/useStableCallback.ts:55-61`). UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:54-62`, no test asserts this.
- **Render-phase call on later renders:** the promotion only runs in an effect, so during render
  N+1 (before its insertion effect) `callback` still holds the function from commit N — a
  render-phase call then invokes that *stale* previous callback rather than throwing (the throw is
  only guaranteed for the initial render, per the type comment "This must fail during the initial
  render phase", `packages/utils/src/useStableCallback.ts:19`). UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:37-49`, no test asserts this.
- **React 17 / Preact fallback environments:** when `useSafeInsertionEffect` is the identity
  fallback, the promotion `callback = next` executes synchronously during render
  (`packages/utils/src/useStableCallback.ts:12`, `packages/utils/src/useStableCallback.ts:38`) —
  a consequence is that the trampoline is returned with the current render's callback already
  active, so the initial-render render-phase throw never occurs in those environments.
  UNVERIFIED — inferred from `packages/utils/src/useStableCallback.ts:6-12`, no test asserts this.
- **Rapid re-renders / rapidly changing closures:** `next` is simply overwritten every render and
  only the latest value survives promotion; the trampoline identity never changes, so consumers
  that keyed effects/memos on the returned reference never re-run because of it
  (`packages/utils/src/useStableCallback.ts:37-39`). UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:37-39`, no test asserts this.
- **Unmount:** no cleanup is registered — the insertion effect only promotes and returns nothing;
  after unmount the trampoline keeps dispatching to the last promoted callback with no
  mounted-guard (`packages/utils/src/useStableCallback.ts:38`, `packages/utils/src/useStableCallback.ts:47-49`).
  UNVERIFIED — inferred from `packages/utils/src/useStableCallback.ts:47-49`, no test asserts this.
- **Multiple instances / nesting:** each call site of the hook gets its own independent record
  from `useRefWithInit`, so any number of `useStableCallback` usages (including within one
  component) neither interfere nor share state (`packages/utils/src/useStableCallback.ts:36`).
  UNVERIFIED — inferred from `packages/utils/src/useStableCallback.ts:36`, no test asserts this.
- **Error message convention:** the thrown message uses the public-package `Base UI:` prefix and
  says what happened (calling an event handler while rendering) — there is no documentation link
  or remedy appended (`packages/utils/src/useStableCallback.ts:58-60`). UNVERIFIED — inferred from
  `packages/utils/src/useStableCallback.ts:54-62`, no test asserts this.

## Shared harness dependencies

None. There is no test file for this unit at all, so no `#test-utils`, `packages/react/test`, or
other shared harness file is imported by any test of this unit.
