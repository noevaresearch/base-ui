# `useTimeout` — behavior spec

Unit: `packages/utils/src/useTimeout` (Phase A util → crate `leptos-ui-utils`).
Spec target per `TODO.md`: `specs/utils/useTimeout.md` (`TODO.md:247-251`). The `TODO.md` entry
has no `wraps-external:` field — this is original Base UI code, not a wrapper around a
third-party npm package — and no `needs-batched-mining:` field, so no batched mining applies.

Source of truth: **none.** No test file exists for this unit anywhere in the repo — the generated
unit inventory lists `testFiles: []` for `useTimeout` (`ralph/generated/utils.json:392-397`), and a
repo-wide search finds no test that tests it directly (the only test files mentioning it consume it
as a utility while testing other components, which is out of scope here). Every claim below is
therefore UNVERIFIED — inferred from the unit's source and its direct dependencies
(`packages/utils/src/useTimeout.ts`, `packages/utils/src/useRefWithInit.ts`,
`packages/utils/src/useOnMount.ts`), with no test asserting it. Stage 2/3 must treat this spec as a
source-derived description of current implementation behavior, not as test-proven behavior, and
should consider adding a `packages/utils/src/useTimeout.test.ts` suite before porting.

The unit is a single 52-line file: a `Timeout` class plus a `useTimeout()` hook returning a stable
instance of it (`packages/utils/src/useTimeout.ts:9-41`, `packages/utils/src/useTimeout.ts:46-52`).
It is the base class of the sibling `Interval` unit — `Interval extends Timeout` and relies on the
inherited `isStarted`/`disposeEffect` members (see `specs/utils/useInterval.md`,
`packages/utils/src/useInterval.ts:2-10`), so any port of this file is a prerequisite for that one.

## Public API surface (props, parts, subcomponents)

All UNVERIFIED — no test asserts any of this.

- Named exports of the module: the `Timeout` class (`packages/utils/src/useTimeout.ts:9`) and the
  `useTimeout()` hook (`packages/utils/src/useTimeout.ts:46`). No components, no props object, no
  parts, no subcomponents.
- The module is marked `'use client'` (`packages/utils/src/useTimeout.ts:1`).
- Class API:
  - `Timeout.create()` static factory returning `new Timeout()` (`packages/utils/src/useTimeout.ts:10-12`).
  - Instance `start(delay: number, fn: Function)`: clears any previously scheduled call, then
    schedules `fn` once via the global `setTimeout(fn, delay)` (`packages/utils/src/useTimeout.ts:19-25`).
    `start` is a prototype method — it is not pre-bound (see Edge cases).
  - Instance `isStarted()`: returns `this.currentId !== EMPTY` (`packages/utils/src/useTimeout.ts:27-29`).
  - Instance `clear` (an arrow-function class property, so it is pre-bound per instance): if a
    timeout is pending, `clearTimeout` it and reset the internal id to `EMPTY`
    (`packages/utils/src/useTimeout.ts:31-36`).
  - Instance `disposeEffect` (also a pre-bound arrow property): returns `this.clear`, shaped as a
    `React.EffectCallback` whose cleanup is the clear function (`packages/utils/src/useTimeout.ts:38-40`).
  - Internal field `currentId: TimeoutId` initialized to the `EMPTY` sentinel, where
    `EMPTY = 0` and `TimeoutId = number` (`packages/utils/src/useTimeout.ts:5`,
    `packages/utils/src/useTimeout.ts:7`, `packages/utils/src/useTimeout.ts:14`).
- Hook API: `useTimeout()` takes no arguments, creates one `Timeout` instance per component (via
  `useRefWithInit(Timeout.create)`), registers the unmount cleanup through `useOnMount`, and
  returns the `Timeout` instance itself — callers invoke `start`/`clear`/`isStarted` on the
  returned object (`packages/utils/src/useTimeout.ts:46-52`).

## State model (controlled/uncontrolled, defaults, transitions)

All UNVERIFIED — no test asserts any of this.

- Not a form/state primitive: no controlled/uncontrolled semantics, no user-facing state.
- Internal state is a single id slot (`currentId`) acting as a latch between "no timeout pending"
  (`EMPTY`, the value `0`) and "timeout pending" (a host-assigned id)
  (`packages/utils/src/useTimeout.ts:7`, `packages/utils/src/useTimeout.ts:14`).
- Transitions:
  - `start(delay, fn)` first clears any pending timeout, then stores the new `setTimeout` id —
    the model is "one live timeout per instance, most recent `start` wins"
    (`packages/utils/src/useTimeout.ts:19-25`).
  - Firing: the `setTimeout` wrapper resets `currentId` to `EMPTY` **before** invoking `fn` —
    a fired one-shot timeout leaves the instance idle again
    (`packages/utils/src/useTimeout.ts:21-24`).
  - `clear()` moves a pending id back to `EMPTY`; the guard makes clearing an idle instance a
    no-op (`packages/utils/src/useTimeout.ts:31-36`).
  - Unmount forces `EMPTY` via the mount-registered cleanup (see Edge cases).
- Instance identity: the hook memoizes the `Timeout` instance with `useRefWithInit`, which
  initializes exactly once (guarded against an `UNINITIALIZED` sentinel) and is stable across
  re-renders of the calling component (`packages/utils/src/useTimeout.ts:47`,
  `packages/utils/src/useRefWithInit.ts:16-22`).

## Keyboard interactions

N/A — non-visual utility. No keyboard behavior is implemented, and no test asserts any.

## Focus management

N/A — no focus behavior is implemented, and no test asserts any.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented, and no test asserts any.

## DOM structure & portal behavior

N/A — the utility creates no DOM, renders nothing, and performs no portal behavior. Its only
host-environment coupling is the global `setTimeout`/`clearTimeout` pair used to schedule and
cancel the callback (`packages/utils/src/useTimeout.ts:21-24`,
`packages/utils/src/useTimeout.ts:31-36`). It resolves these as bare globals rather than through
an `ownerWindow` lookup, so it performs no realm handling of its own. UNVERIFIED — no test
asserts this.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no DOM events are emitted, dispatched, or observed; no bubbling or `preventDefault`
semantics exist. The closest analog is user-callback invocation, and even that carries no
payload: the `setTimeout` wrapper invokes `fn()` with zero arguments — no timestamp or event-like
object is forwarded (`packages/utils/src/useTimeout.ts:21-24`). UNVERIFIED — no test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

All UNVERIFIED — no test asserts any of this.

- Rapid re-`start`: because `start` clears before scheduling, calling `start` repeatedly on the
  same instance replaces the pending timeout instead of stacking timeouts — at most one live
  timeout exists per instance, and callbacks from superseded `start` calls never fire
  (`packages/utils/src/useTimeout.ts:19-25`).
- Re-entrancy from inside `fn`: since the wrapper resets `currentId` to `EMPTY` before invoking
  `fn`, code inside `fn` observes an idle instance — `isStarted()` returns `false`, `clear()` is
  a no-op, and calling `start()` re-arms a fresh timer cleanly
  (`packages/utils/src/useTimeout.ts:21-24`, `packages/utils/src/useTimeout.ts:27-29`).
- Double-`clear` safety: `clear` is guarded on `currentId !== EMPTY`, so clearing an idle or
  already-fired instance neither throws nor calls `clearTimeout(0)`
  (`packages/utils/src/useTimeout.ts:31-36`).
- Pre-binding asymmetry: `clear` and `disposeEffect` are arrow-function instance properties, so
  they can be detached and passed around (the hook does exactly this, passing
  `timeout.disposeEffect` to `useOnMount`), whereas `start` and `isStarted` are prototype methods
  that must be invoked as `timeout.start(...)` (`packages/utils/src/useTimeout.ts:31-40`,
  `packages/utils/src/useTimeout.ts:49`).
- Unmount: the hook registers `timeout.disposeEffect` through `useOnMount`, which is
  `React.useEffect(fn, EMPTY_ARRAY)` running once on mount; the effect returns `this.clear` as
  React's cleanup, so any pending timeout is automatically `clearTimeout`-ed when the calling
  component unmounts and `fn` never runs post-unmount (`packages/utils/src/useTimeout.ts:49`,
  `packages/utils/src/useTimeout.ts:38-40`, `packages/utils/src/useOnMount.ts:8-12`).
- StrictMode-style remount: because the cleanup runs on every effect teardown, a dev-mode
  simulate-remount between two mounts would also clear any timeout started during the first
  mount period; the instance itself survives (it lives in a ref, not effect state)
  (`packages/utils/src/useRefWithInit.ts:16-22`, `packages/utils/src/useOnMount.ts:8-12`).
- First-run timing: `fn` does not run on `start`; the single invocation happens after one full
  `delay` (plain `setTimeout` semantics — there is no leading/initial call)
  (`packages/utils/src/useTimeout.ts:21-24`).
- No delay validation: `delay` is passed straight to `setTimeout` (host-environment clamping, e.g.
  the 4 ms minimum for nested timers in HTML, applies but is not enforced here), and the id is
  cast to `number` for typing because Node.js types are enabled in development
  (`packages/utils/src/useTimeout.ts:24`).
- Nesting / multiple consumers: each call site of `useTimeout()` gets its own independent
  `Timeout` instance (per-component ref), so nested or sibling consumers do not share scheduling
  state (`packages/utils/src/useTimeout.ts:46-52`,
  `packages/utils/src/useRefWithInit.ts:16-22`).

## Shared harness dependencies

None. There is no test file for this unit at all (`ralph/generated/utils.json:392-397` lists
`testFiles: []`), so no `#test-utils` or `packages/react/test` harness dependency exists for this
unit. When a test suite is added, this section must be revisited.
