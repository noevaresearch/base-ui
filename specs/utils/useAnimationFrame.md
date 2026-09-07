# `useAnimationFrame` — behavior spec

Unit: `packages/utils/src/useAnimationFrame` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/useAnimationFrame.test.ts` (the only test file for this unit).

`TODO.md` has no `wraps-external:` field for this unit — it is original Base UI code, not a
wrapper around a third-party npm package, so behavior is mined directly from the unit's own test
file.

The suite is minimal: one test exercising the static `AnimationFrame` API through a mocked global
`requestAnimationFrame` (`packages/utils/src/useAnimationFrame.test.ts:14-31`). Claims below are
split accordingly between proven and UNVERIFIED (source-inferred) behavior.

## Public API surface (props, parts, subcomponents)

- Named exports exercised by the test: the `AnimationFrame` class and the
  `resetAnimationFrameScheduler` function, both imported from the sibling module
  `./useAnimationFrame` (`packages/utils/src/useAnimationFrame.test.ts:2`). No components, no
  props object, no parts, no subcomponents.
- Static form: `AnimationFrame.request(fn)` accepts a frame callback and returns an id
  (`firstId`) that is a plain value usable by later calls (`packages/utils/src/useAnimationFrame.test.ts:21`).
- Static form: `AnimationFrame.cancel(id)` accepts a previously returned id
  (`packages/utils/src/useAnimationFrame.test.ts:22`).
- `resetAnimationFrameScheduler()` is callable with zero arguments and is safe to call repeatedly
  (the suite calls it in both `beforeEach` and `afterEach`)
  (`packages/utils/src/useAnimationFrame.test.ts:5-12`).
- The test suite exercises only the static/class API. The `useAnimationFrame()` hook and the
  instance API (`AnimationFrame.create`, instance `request`/`cancel`/`disposeEffect`, `currentId`)
  are part of the module but no test asserts them. UNVERIFIED — inferred from
  `packages/utils/src/useAnimationFrame.ts:109-156`, no test asserts this.

## State model (controlled/uncontrolled, defaults, transitions)

- Process-global scheduler: two `AnimationFrame.request` calls made back-to-back share scheduling
  state — the second callback is delivered by manually firing the frame callback captured for the
  first request, which only works if both requests land in the same global queue
  (`packages/utils/src/useAnimationFrame.test.ts:21-30`).
- Live-callback accounting invariant: the scheduler tracks how many queued entries are still live;
  after canceling the same id twice, the later-registered callback still fires exactly once,
  proving the double cancel decremented the live count only once (the second cancel is a no-op)
  (`packages/utils/src/useAnimationFrame.test.ts:22-30`).
- Controlled/uncontrolled semantics, defaults, and user-facing state transitions: N/A — this is
  not a form/state primitive.

## Keyboard interactions

N/A — non-visual utility. No test asserts any keyboard behavior.

## Focus management

N/A — no focus behavior is implemented or asserted by any test.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test.

## DOM structure & portal behavior

N/A — the utility creates no DOM, renders nothing, and performs no portal behavior. Its only
host-environment coupling is that frame dispatch goes through the global `requestAnimationFrame`:
the test intercepts `globalThis.requestAnimationFrame` with a spy and captures the scheduler's
frame registration (`packages/utils/src/useAnimationFrame.test.ts:16-19`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no DOM events are emitted, dispatched, or asserted by any test. The closest analog,
frame-callback invocation, is covered under Public API surface, State model, and Edge cases. Note
one proven structural fact: user callbacks are not handed to the native `requestAnimationFrame`
directly — the mock captures an internal frame function which, when invoked, dispatches the
user-registered callback (`packages/utils/src/useAnimationFrame.test.ts:16-30`).

## Edge cases (rapid interactions, unmount, nesting)

Proven behaviors:

- Double-cancel safety: canceling the same frame id more than once neither throws nor corrupts
  accounting — after `AnimationFrame.cancel(firstId)` runs twice, a subsequently requested
  callback still runs exactly once when the pending native frame fires
  (`packages/utils/src/useAnimationFrame.test.ts:22-30`).
- Post-cancel registration still dispatches: a callback requested after an earlier request was
  canceled is invoked exactly once (`toHaveBeenCalledTimes(1)`) when the captured frame is fired
  (`packages/utils/src/useAnimationFrame.test.ts:26-30`).

Unproven behaviors (must not be relied on as specified):

- That a canceled callback never fires: the canceled callback is an anonymous no-op and the test
  never asserts it was not invoked (`packages/utils/src/useAnimationFrame.test.ts:21-28`).
  UNVERIFIED — inferred from `packages/utils/src/useAnimationFrame.ts:78-82`, no test asserts this.
- Timestamp forwarding: whether the native frame timestamp is passed through to user callbacks.
  The test fires the frame with `0` and only asserts the callback ran.
  UNVERIFIED — inferred from `packages/utils/src/useAnimationFrame.ts:34-49`, no test asserts this.
- Batched dispatch of multiple live callbacks in a single frame. UNVERIFIED — inferred from
  `packages/utils/src/useAnimationFrame.ts:45-49`, no test asserts this (only one live callback
  exists at fire time in the test).
- Coalescing: whether multiple requests while a frame is pending result in a single native
  `requestAnimationFrame` registration. The test does not assert how many entries the mock
  captured (`packages/utils/src/useAnimationFrame.test.ts:16-30`). UNVERIFIED — inferred from
  `packages/utils/src/useAnimationFrame.ts:66-69`, no test asserts this.
- `useAnimationFrame()` hook lifecycle: stable instance across renders and automatic cancel of any
  pending frame on unmount. UNVERIFIED — inferred from
  `packages/utils/src/useAnimationFrame.ts:150-156`, no test asserts this.
- Instance `request` clearing any previously scheduled call before scheduling the new one.
  UNVERIFIED — inferred from `packages/utils/src/useAnimationFrame.ts:127-133`, no test asserts this.
- `resetAnimationFrameScheduler` semantics beyond "callable reset": dropping all pending callbacks
  and continuing the id sequence so pre-reset cancel calls cannot cancel post-reset callbacks.
  UNVERIFIED — inferred from `packages/utils/src/useAnimationFrame.ts:96-107`, no test asserts this
  (the suite only calls it as a bare reset between tests, `packages/utils/src/useAnimationFrame.test.ts:5-12`).
- Dev-only re-scheduling when the global `requestAnimationFrame` implementation changes while a
  frame is pending (fake-timer teardown guard). UNVERIFIED — inferred from
  `packages/utils/src/useAnimationFrame.ts:58-69`, no test asserts this.

## Shared harness dependencies

None. The test file imports only vitest APIs (`expect`, `vi`, `describe`, `beforeEach`,
`afterEach`, `it`) and the unit's own exports
(`packages/utils/src/useAnimationFrame.test.ts:1-2`). There is no `#test-utils` or
`packages/react/test` harness dependency.
