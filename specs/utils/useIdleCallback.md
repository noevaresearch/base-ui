# `useIdleCallback` — behavior spec

Unit: `packages/utils/src/useIdleCallback` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/useIdleCallback.test.tsx` and
`packages/utils/src/useIdleCallback.fallback.test.ts` (the only test files for this unit).

`TODO.md` has no `wraps-external:` field for this unit — it is original Base UI code, not a
wrapper around a third-party npm package, so behavior is mined directly from the unit's own test
files. The unit also has no `needs-batched-mining` flag (2 test files, ~200 test lines total).

The suite has two halves: a main suite that stubs the global `requestIdleCallback` /
`cancelIdleCallback` with a Map-backed manual queue and tests the `IdleCallback` class plus the
`useIdleCallback` hook, and a fallback suite that removes both globals and uses fake timers to
prove the timer-based degradation path.

## Public API surface (props, parts, subcomponents)

- Named exports exercised by the tests: the `IdleCallback` class and the `useIdleCallback` hook,
  both imported from the sibling module `./useIdleCallback`
  (`packages/utils/src/useIdleCallback.test.tsx:5-8`). No components, no props object, no parts,
  no subcomponents.
- `IdleCallback.create()` is a static factory; two successive calls return distinct instances and
  each is an `instanceof IdleCallback` (`packages/utils/src/useIdleCallback.test.tsx:44-50`).
- Instance method `start(callback)` accepts a zero-arg callback
  (`packages/utils/src/useIdleCallback.test.tsx:56`).
- Instance method `clear()` cancels the pending callback
  (`packages/utils/src/useIdleCallback.test.tsx:70`).
- Instance method `disposeEffect()` returns the `clear` function by identity
  (`expect(dispose).toBe(idleCallback.clear)`), making it suitable as an effect cleanup
  (`packages/utils/src/useIdleCallback.test.tsx:110-111`).
- Instance property `currentId` is observable and is `null` after a scheduled callback has run
  (`packages/utils/src/useIdleCallback.test.tsx:98`).
- Hook form `useIdleCallback()` takes no arguments and returns the same `IdleCallback`-typed
  scheduler (`packages/utils/src/useIdleCallback.test.tsx:123-137`).

## State model (controlled/uncontrolled, defaults, transitions)

- Not a form/state primitive: controlled/uncontrolled semantics do not apply (N/A).
- Instance lifecycle state is a single "pending or not" slot: `start` schedules, and a subsequent
  `start` on the same instance supersedes the previous callback — only the newest callback runs
  and the superseded one never fires
  (`packages/utils/src/useIdleCallback.test.tsx:76-87`).
- After a scheduled callback has run, the instance transitions back to "no pending callback":
  `currentId` is reset to `null`, explicitly so that a later `clear()` cannot cancel an unrelated
  callback (`packages/utils/src/useIdleCallback.test.tsx:97-98`), and the instance can be
  `start`ed again (`packages/utils/src/useIdleCallback.test.tsx:100-102`).
- Dispatch is asynchronous relative to the scheduling task: immediately after `start(fn)` the
  callback has not been invoked; it only runs once idle callbacks are flushed
  (`packages/utils/src/useIdleCallback.test.tsx:56-62`).
- Host-environment-dependent dispatch path: with `requestIdleCallback` present, dispatch goes
  through the stubbed global (`packages/utils/src/useIdleCallback.test.tsx:26-31`); with both
  globals absent, the same `start`/`clear`/supersede behavior is delivered through pending
  timers instead (`vi.runOnlyPendingTimers()` fires the scheduled callback)
  (`packages/utils/src/useIdleCallback.fallback.test.ts:26-31`).
- Scheduling is per-instance: two callbacks started on two different instances are both delivered
  by one flush of the manual queue (`packages/utils/src/useIdleCallback.test.tsx:19-23`), while
  two callbacks started on one instance collapse to the newest only
  (`packages/utils/src/useIdleCallback.test.tsx:81-86`).

## Keyboard interactions

N/A — non-visual utility. No test asserts any keyboard behavior.

## Focus management

N/A — no focus behavior is implemented or asserted by any test.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test.

## DOM structure & portal behavior

N/A — the utility creates no DOM, renders nothing (`render(<Test />)` with a component returning
`null`, `packages/utils/src/useIdleCallback.test.tsx:126-129`), and performs no portal behavior.
Its only host-environment coupling is dispatch through the ambient `requestIdleCallback` /
`cancelIdleCallback` globals: both test files stub these globals *before* dynamically importing
the module in `beforeAll` (`packages/utils/src/useIdleCallback.test.tsx:25-32`,
`packages/utils/src/useIdleCallback.fallback.test.ts:7-13`), implying the module depends on the
ambient globals. Whether that dependency is captured at import time or resolved per call is
UNVERIFIED — no test distinguishes the two.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no DOM events are emitted, dispatched, or asserted by any test. The scheduled callback is
invoked with no asserted arguments (the tests pass argument-less `vi.fn()`s, e.g.
`packages/utils/src/useIdleCallback.test.tsx:54`), so any payload shape is
UNVERIFIED — no test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

Proven behaviors:

- Rapid supersede: calling `start` twice in a row on one instance leaves only the second callback
  scheduled — the first never fires, the second fires exactly once
  (`packages/utils/src/useIdleCallback.test.tsx:81-86`; same supersede semantics proven on the
  no-`requestIdleCallback` fallback path, `packages/utils/src/useIdleCallback.fallback.test.ts:26-31`).
- Cancel-then-flush safety: `clear()` after `start()` guarantees the callback never runs when idle
  callbacks are flushed (`packages/utils/src/useIdleCallback.test.tsx:69-73`; also proven on the
  fallback path with fake timers, `packages/utils/src/useIdleCallback.fallback.test.ts:33-37`).
- Reuse after completion: an instance whose callback already ran can be `start`ed again and the
  callback fires a second time (`packages/utils/src/useIdleCallback.test.tsx:93-102`).
- Stale-id safety: because `currentId` resets to `null` after the callback runs, a later `clear()`
  on the same instance cannot cancel an unrelated callback
  (`packages/utils/src/useIdleCallback.test.tsx:97-98`).
- Unmount: a pending callback started via the hook's scheduler is canceled when the component
  unmounts — after `unmount()` and a flush, the callback has not been called
  (`packages/utils/src/useIdleCallback.test.tsx:148-154`).
- `disposeEffect()` as cleanup: the returned dispose function is `clear` itself and canceling via
  it prevents the callback from firing (`packages/utils/src/useIdleCallback.test.tsx:110-116`).
- Fallback environment: with `requestIdleCallback` and `cancelIdleCallback` both absent, the full
  schedule → supersede → cancel cycle still works via the timer-based path
  (`packages/utils/src/useIdleCallback.fallback.test.ts:21-38`).

Unproven behaviors (must not be relied on as specified):

- Timing/latency of the fallback path: the fallback test only proves ordering under
  `vi.runOnlyPendingTimers()`; the actual delay value and any timeout-budget semantics are
  UNVERIFIED — no test asserts this.
- Whether `start` on an already-pending instance returns anything, or whether `clear`/`start` are
  idempotent when no callback is pending, is UNVERIFIED — no test asserts this.
- Nesting: whether a callback may call `start` again on the same instance from within the idle
  callback is UNVERIFIED — no test asserts this (the reuse test re-`start`s from outside the
  callback, `packages/utils/src/useIdleCallback.test.tsx:100`).
- Whether `useIdleCallback`'s hook-level unmount cancellation uses `disposeEffect` internally is
  UNVERIFIED — no test asserts this; only the observable outcome (pending callback canceled on
  unmount) is proven (`packages/utils/src/useIdleCallback.test.tsx:148-154`).

## Shared harness dependencies

- `@mui/internal-test-utils`: the main suite uses `createRenderer()` to obtain an async `render`
  that returns `rerender` and `unmount` handles, used for the stability and unmount tests
  (`packages/utils/src/useIdleCallback.test.tsx:121`, `:131-133`, `:148-151`). No
  `#test-utils`-style repo-local harness is imported.
- Both suites otherwise rely only on vitest APIs (`vi.stubGlobal`, `vi.useFakeTimers`,
  `vi.fn`, `expect`, standard lifecycle hooks) and the unit's own exports; the manual idle queue
  (`scheduledCallbacks` Map + `flushIdleCallbacks`) is defined locally inside the main test file
  (`packages/utils/src/useIdleCallback.test.tsx:10-23`).
