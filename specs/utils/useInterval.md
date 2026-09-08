# `useInterval` — behavior spec

Unit: `packages/utils/src/useInterval` (Phase A util → crate `leptos-ui-utils`).
Spec target per `TODO.md`: `specs/utils/useInterval.md` (`TODO.md:204-208`). The `TODO.md` entry
has no `wraps-external:` field — this is original Base UI code, not a wrapper around a third-party
npm package.

Source of truth: **none.** No test file exists for this unit anywhere in the repo — the generated
unit inventory lists `testFiles: []` for `useInterval` (`ralph/generated/utils.json:316-321`), and
a repo-wide import search finds no test that references `useInterval`. Every claim below is
therefore UNVERIFIED — inferred from the unit's source and its direct dependencies
(`packages/utils/src/useInterval.ts`, `packages/utils/src/useTimeout.ts`,
`packages/utils/src/useOnMount.ts`, `packages/utils/src/useRefWithInit.ts`), with no test
asserting it. Stage 2/3 must treat this spec as a source-derived description of current
implementation behavior, not as test-proven behavior, and should consider adding a
`packages/utils/src/useInterval.test.ts` suite before porting.

The unit is a single 42-line file: a hook `useInterval()` plus a `Interval` class extending the
`Timeout` class from the sibling `useTimeout` module (`packages/utils/src/useInterval.ts:2-10`).

## Public API surface (props, parts, subcomponents)

All UNVERIFIED — no test asserts any of this.

- Named exports of the module: the `Interval` class and the `useInterval()` hook
  (`packages/utils/src/useInterval.ts:10`, `packages/utils/src/useInterval.ts:36`). No components,
  no props object, no parts, no subcomponents.
- The module is marked `'use client'` (`packages/utils/src/useInterval.ts:1`).
- Class API:
  - `Interval.create()` static factory returning `new Interval()` (`packages/utils/src/useInterval.ts:11-13`).
  - Instance `start(delay: number, fn: Function)`: clears any previously scheduled call, then
    schedules `fn` on `setInterval(fn, delay)` (`packages/utils/src/useInterval.ts:18-23`).
  - Instance `clear` (an arrow-function class property, so it is pre-bound per instance): if an
    interval is pending, `clearInterval` it and reset the internal id to `EMPTY`
    (`packages/utils/src/useInterval.ts:25-30`).
  - Inherited from `Timeout` (parent class at `packages/utils/src/useTimeout.ts:9-41`):
    `isStarted()` returning `currentId !== EMPTY` (`packages/utils/src/useTimeout.ts:27-29`) and
    `disposeEffect` returning `this.clear` as a mount-effect cleanup
    (`packages/utils/src/useTimeout.ts:38-40`). `Interval` overrides `start` and redefines `clear`
    but does not override `isStarted`/`disposeEffect` (`packages/utils/src/useInterval.ts:10-31`).
  - `currentId` field initialized to the `EMPTY` sentinel (`packages/utils/src/useTimeout.ts:14`,
    `packages/utils/src/useInterval.ts:8` where `EMPTY = 0`).
- Hook API: `useInterval()` takes no arguments, creates one `Interval` instance per component
  (via `useRefWithInit(Interval.create)`), registers the unmount cleanup, and returns the
  `Interval` instance itself — callers invoke `start`/`clear` on the returned object
  (`packages/utils/src/useInterval.ts:36-42`).

## State model (controlled/uncontrolled, defaults, transitions)

All UNVERIFIED — no test asserts any of this.

- Not a form/state primitive: no controlled/uncontrolled semantics, no user-facing state.
- Internal state is a single id slot (`currentId`) acting as a latch between "no interval pending"
  (`EMPTY`, the value `0`) and "interval pending" (a positive host id)
  (`packages/utils/src/useInterval.ts:8`, `packages/utils/src/useTimeout.ts:14`).
- Transitions:
  - `start(delay, fn)` first clears any pending interval, then stores the new `setInterval` id —
    so the model is "one live interval per instance, most recent `start` wins"
    (`packages/utils/src/useInterval.ts:18-23`).
  - `clear()` moves a pending id back to `EMPTY`; the guard makes clearing an idle instance a
    no-op (`packages/utils/src/useInterval.ts:25-30`).
  - Unmount forces `EMPTY` via the mount-registered cleanup (see Edge cases).
- Difference from the parent `Timeout`: `Timeout.start` resets `currentId` to `EMPTY` immediately
  before invoking `fn` (one-shot semantics, `packages/utils/src/useTimeout.ts:21-24`), whereas
  `Interval.start` deliberately does not reset the id inside the interval callback — the id stays
  live across every tick until `clear()` runs, which is what makes the inherited `isStarted()`
  meaningful for a repeating timer (`packages/utils/src/useInterval.ts:20-22`).
- Instance identity: the hook memoizes the `Interval` instance with `useRefWithInit`, which
  initializes exactly once (guard against an `UNINITIALIZED` sentinel) and is stable across
  re-renders of the calling component (`packages/utils/src/useInterval.ts:37`,
  `packages/utils/src/useRefWithInit.ts:16-22`).

## Keyboard interactions

N/A — non-visual utility. No keyboard behavior is implemented, and no test asserts any.

## Focus management

N/A — no focus behavior is implemented, and no test asserts any.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented, and no test asserts any.

## DOM structure & portal behavior

N/A — the utility creates no DOM, renders nothing, and performs no portal behavior. Its only
host-environment coupling is the global `setInterval`/`clearInterval` pair used to schedule and
cancel the repeating callback (`packages/utils/src/useInterval.ts:20-22`,
`packages/utils/src/useInterval.ts:25-30`). UNVERIFIED — no test asserts this.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no DOM events are emitted, dispatched, or observed; no bubbling or `preventDefault`
semantics exist. The closest analog is user-callback invocation, and even that carries no
payload: the `setInterval` wrapper invokes `fn()` with zero arguments — no timestamp or
event-like object is forwarded (`packages/utils/src/useInterval.ts:20-22`). UNVERIFIED — no test
asserts this.

## Edge cases (rapid interactions, unmount, nesting)

All UNVERIFIED — no test asserts any of this.

- Rapid re-`start`: because `start` clears before scheduling, calling `start` repeatedly on the
  same instance replaces the pending interval instead of stacking intervals — at most one live
  interval exists per instance, and callbacks from superseded `start` calls never fire
  (`packages/utils/src/useInterval.ts:18-23`).
- Re-`start` with the same delay does not reset the phase of an already-running interval in a
  special way beyond the clear-and-reschedule — the old interval is cancelled and a fresh
  `setInterval` begins a full delay cycle (`packages/utils/src/useInterval.ts:18-23`).
- Double-`clear` safety: `clear` is guarded on `currentId !== EMPTY`, so clearing an idle or
  already-cleared instance neither throws nor calls `clearInterval(0)`
  (`packages/utils/src/useInterval.ts:25-30`).
- Unmount: the hook registers `timeout.disposeEffect` through `useOnMount`, which is a
  `React.useEffect(fn, EMPTY_ARRAY)` running once on mount; the effect returns `this.clear` as
  React's cleanup, so any pending interval is automatically `clearInterval`-ed when the calling
  component unmounts (`packages/utils/src/useInterval.ts:39`, `packages/utils/src/useOnMount.ts:8-12`,
  `packages/utils/src/useTimeout.ts:38-40`).
- First tick timing: `fn` does not run on `start`; the first invocation happens after one full
  `delay` (plain `setInterval` semantics — there is no leading/initial call)
  (`packages/utils/src/useInterval.ts:20-22`).
- Nesting / multiple consumers: each call site of `useInterval()` gets its own independent
  `Interval` instance (per-component ref), so nested or sibling consumers do not share scheduling
  state (`packages/utils/src/useInterval.ts:36-42`,
  `packages/utils/src/useRefWithInit.ts:16-22`).
- No delay clamping or validation: `delay` is passed straight to `setInterval` (host-environment
  clamping, e.g. the 4 ms minimum for nested timers in HTML, applies but is not enforced here)
  (`packages/utils/src/useInterval.ts:18-23`).

## Shared harness dependencies

None. There is no test file for this unit at all (`ralph/generated/utils.json:316-321` lists
`testFiles: []`), so no `#test-utils` or `packages/react/test` harness dependency exists for this
unit. When a test suite is added, this section must be revisited.
