# `createLogOnce` — behavior spec

Unit: `packages/utils/src/createLogOnce` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/createLogOnce.test.ts` (the only test file for this unit).

## Public API surface (props, parts, subcomponents)

- Two named exports: `createLogOnce` and `reset` (`packages/utils/src/createLogOnce.test.ts:2`).
- `createLogOnce(severity, prefix?)` is a factory: first argument is the severity, exercised with
  the literal values `'warn'` and `'error'` (`packages/utils/src/createLogOnce.test.ts:15`,
  `packages/utils/src/createLogOnce.test.ts:24`); second argument is an optional prefix string,
  exercised as `'My Library'` (`packages/utils/src/createLogOnce.test.ts:15`) and omitted in other
  tests (`packages/utils/src/createLogOnce.test.ts:31`).
- The factory returns a logger function. The logger is invoked with one or more message-string
  arguments: one argument in most tests (`packages/utils/src/createLogOnce.test.ts:16`) and two
  arguments in the multi-message test (`packages/utils/src/createLogOnce.test.ts:32`).
- `reset` takes no arguments and returns nothing that is consumed; it is called between tests and
  inside a test to clear deduplication state (`packages/utils/src/createLogOnce.test.ts:6`,
  `packages/utils/src/createLogOnce.test.ts:49`).
- No components, no props object, no parts — this is a console-logging utility.

## State model (controlled/uncontrolled, defaults, transitions)

- There is hidden mutable state: a "seen" registry consulted by the logger. Repeated calls with
  the same message on the same logger emit to the console exactly once — the second (and any
  further) identical call is a no-op (`packages/utils/src/createLogOnce.test.ts:16-18`).
- The dedup key includes the severity: a `'warn'` logger and an `'error'` logger created
  separately each emit the identical message `'message'` once — neither suppresses the other
  (`packages/utils/src/createLogOnce.test.ts:39-42`).
- `reset()` clears that registry: after `reset()`, a logger that had already emitted `'message'`
  emits it again, for a total of two console calls for the same logger and message
  (`packages/utils/src/createLogOnce.test.ts:47-51`).
- Transition summary: unseen (severity, message) → logged on first emit; further identical emits
  are suppressed until `reset()` returns the entry to unseen
  (`packages/utils/src/createLogOnce.test.ts:16-18`, `packages/utils/src/createLogOnce.test.ts:47-51`).
- Untested (no assertions exist, behavior not established by the test file): whether dedup is
  shared across two logger instances of the *same* severity, whether different messages from the
  same logger are deduped independently, whether the prefix participates in the dedup key, and
  behavior for zero or non-string arguments.

## Keyboard interactions

N/A — non-visual console utility; no DOM is involved and no test asserts keyboard behavior
(`packages/utils/src/createLogOnce.test.ts:1-53`).

## Focus management

N/A — no focus behavior is implemented or asserted by any test
(`packages/utils/src/createLogOnce.test.ts:1-53`).

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test
(`packages/utils/src/createLogOnce.test.ts:1-53`).

## DOM structure & portal behavior

N/A — the utility renders nothing and creates no DOM or portals; its only observable output is a
call to a `console` method (`packages/utils/src/createLogOnce.test.ts:14`,
`packages/utils/src/createLogOnce.test.ts:23`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- No DOM events. The output channel is selected by severity: `'warn'` calls `console.warn`
  (`packages/utils/src/createLogOnce.test.ts:14-19`, `packages/utils/src/createLogOnce.test.ts:30-33`,
  `packages/utils/src/createLogOnce.test.ts:46-51`) and `'error'` calls `console.error`
  (`packages/utils/src/createLogOnce.test.ts:23-26`, `packages/utils/src/createLogOnce.test.ts:38-42`).
- Payload shape: the console method receives a single string argument, not the raw argument list.
  - With a prefix, prefix and message are joined as `'My Library: message'` (prefix, `': '`,
    message) (`packages/utils/src/createLogOnce.test.ts:19`).
  - Without a prefix, the message string is passed through unchanged, with no leading/trailing
    separator (`packages/utils/src/createLogOnce.test.ts:26`).
  - With multiple message arguments, the messages are joined with a single space: `'first second'`
    (`packages/utils/src/createLogOnce.test.ts:32-33`).
- No bubbling or preventDefault semantics apply (no DOM involvement).

## Edge cases (rapid interactions, unmount, nesting)

- Immediate repeated calls (back-to-back, no reset in between) are deduplicated: two synchronous
  calls to the same logger with the same message produce one console call
  (`packages/utils/src/createLogOnce.test.ts:16-18`).
- Same message through different severities is not deduplicated across severities: warn and error
  each emit once (`packages/utils/src/createLogOnce.test.ts:39-42`).
- `reset()` between tests is the documented way to re-arm logging; it is invoked in `beforeEach`
  so tests are isolated from prior dedup state (`packages/utils/src/createLogOnce.test.ts:5-7`).
- Unmount: N/A — nothing is mounted; no test mounts a component
  (`packages/utils/src/createLogOnce.test.ts:1-53`).
- Nesting: N/A — no nested structures exist; the only composition exercised is creating multiple
  independent loggers, whose dedup state is per-severity
  (`packages/utils/src/createLogOnce.test.ts:39-42`).

## Shared harness dependencies

None. The test file imports only `vitest` APIs and the unit under test itself — no `#test-utils`,
no `packages/react/test/` harness file (`packages/utils/src/createLogOnce.test.ts:1-2`).
