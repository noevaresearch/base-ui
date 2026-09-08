# `warn` — behavior spec

Unit: `packages/utils/src/warn` (Phase A util → crate `leptos-ui-utils`, per `TODO.md:280-284`).
The unit is a pre-configured instance of the sibling `createLogOnce` factory with no dedicated
test file of its own. Source of truth: `packages/utils/src/createLogOnce.test.ts` (the suite of
the factory this unit wraps; the only test file exercising this unit's behavior) plus the unit's
own source for the API surface. The sibling util has its own spec (`specs/utils/createLogOnce.md`);
this spec scopes to what the `warn` unit adds on top of it.

## Public API surface (props, parts, subcomponents)

- Two named exports, no default export: `warn` and `reset` (`packages/utils/src/warn.ts:3`,
  `packages/utils/src/warn.ts:5`).
- `warn` is created by calling the `createLogOnce` factory with severity `'warn'` and the prefix
  `'Base UI'`, yielding a logger callable (`packages/utils/src/warn.ts:3`). The factory call
  signature and logger call signature are proven by the factory's suite: the factory takes a
  severity and an optional prefix string (`packages/utils/src/createLogOnce.test.ts:15`,
  `packages/utils/src/createLogOnce.test.ts:24`), and the returned logger is invoked with one or
  more string message arguments (`packages/utils/src/createLogOnce.test.ts:16`,
  `packages/utils/src/createLogOnce.test.ts:32`).
- `reset` is re-exported unchanged from `./createLogOnce`; it takes no arguments and clears
  deduplication state (`packages/utils/src/warn.ts:5`; behavior proven at
  `packages/utils/src/createLogOnce.test.ts:49-51`).
- UNVERIFIED — inferred from `packages/utils/src/warn.ts:3`, no test asserts this: that the prefix
  literal is exactly `'Base UI'` in the emitted output. The prefix-joining behavior itself is
  proven, but only with the test's own `'My Library'` prefix
  (`packages/utils/src/createLogOnce.test.ts:19`); no test exercises the `warn` unit instance, so
  its concrete `'Base UI: ...'` output string is not asserted anywhere.
- No components, no props object, no parts, no subcomponents — this is a console-logging utility
  (`packages/utils/src/warn.ts:1-5`).

## State model (controlled/uncontrolled, defaults, transitions)

- Not applicable in the React controlled/uncontrolled sense — no component, no state, no render.
  The only hidden state is the `createLogOnce` dedup registry that the `warn` logger consults.
- A message emitted through the same logger twice emits to the console exactly once; the second
  identical call is a no-op (`packages/utils/src/createLogOnce.test.ts:16-18`).
- Severity is part of the dedup key: a `'warn'`-severity logger and an `'error'`-severity logger
  each emit the identical message `'message'` once — neither suppresses the other
  (`packages/utils/src/createLogOnce.test.ts:39-42`). For this unit that means an error-channel
  logger elsewhere in the library does not suppress `warn`'s emissions, and vice versa.
- `reset()` clears the registry: after `reset()`, a logger that already emitted `'message'` emits
  it again, for two total console calls (`packages/utils/src/createLogOnce.test.ts:47-51`).
- UNVERIFIED — inferred from `packages/utils/src/createLogOnce.test.ts:39-42`, no test asserts
  this: whether dedup state is scoped per logger instance or shared globally per severity. The
  severity-independence test creates two loggers but of different severities, so it cannot
  distinguish per-logger from per-severity-global state. This matters for Stage 3: whether
  `warn('msg')` and `createLogOnce('warn')('msg')` (same message, same severity) dedup against
  each other is not established by any test.
- Transition summary: unseen (severity, message) → logged on first emit; identical emits
  suppressed until `reset()` returns the entry to unseen
  (`packages/utils/src/createLogOnce.test.ts:16-18`,
  `packages/utils/src/createLogOnce.test.ts:47-51`).

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

N/A — the utility renders nothing and creates no DOM, portals, or elements; its only observable
output is a call to a `console` method (`packages/utils/src/createLogOnce.test.ts:14`,
`packages/utils/src/createLogOnce.test.ts:23`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- No DOM events of any kind. The output channel is fixed at creation: the `'warn'` severity used
  by this unit routes to `console.warn` (`packages/utils/src/createLogOnce.test.ts:14-19`,
  `packages/utils/src/createLogOnce.test.ts:30-33`,
  `packages/utils/src/createLogOnce.test.ts:46-51` — every warn-severity test spies on and
  asserts against `console.warn`); the `'error'` severity routes to `console.error` as proven by
  the contrast case (`packages/utils/src/createLogOnce.test.ts:23-26`,
  `packages/utils/src/createLogOnce.test.ts:38-42`).
- Payload shape: the console method receives a single already-joined string argument, not the raw
  argument list (`packages/utils/src/createLogOnce.test.ts:19`,
  `packages/utils/src/createLogOnce.test.ts:33` — `toHaveBeenCalledWith` is given exactly one
  string).
  - With a prefix, prefix and message are joined with `': '` (`'My Library: message'`)
    (`packages/utils/src/createLogOnce.test.ts:19`). For this unit's fixed `'Base UI'` prefix the
    analogous shape is `Base UI: <message>` — UNVERIFIED — inferred from
    `packages/utils/src/warn.ts:3` plus `packages/utils/src/createLogOnce.test.ts:19`, no test
    asserts the literal `'Base UI'` output.
  - With multiple message arguments, messages are joined with a single space (`'first second'`)
    (`packages/utils/src/createLogOnce.test.ts:32-33`).
  - UNVERIFIED — inferred from `packages/utils/src/warn.ts:3` plus
    `packages/utils/src/createLogOnce.test.ts:32-33`, no test asserts this: the combination of a
    prefix with multiple messages. The multi-message test creates its logger without a prefix, so
    the exact output of `warn('first', 'second')` (presumably `Base UI: first second`) is not
    established.
- No bubbling or preventDefault semantics apply (no DOM involvement).

## Edge cases (rapid interactions, unmount, nesting)

- Rapid/repeated interaction — the core contract: two synchronous back-to-back calls to the same
  logger with the same message produce exactly one console call
  (`packages/utils/src/createLogOnce.test.ts:16-18`).
- Re-arming: `reset()` (re-exported by this unit, `packages/utils/src/warn.ts:5`) makes an already
  emitted message log again (`packages/utils/src/createLogOnce.test.ts:45-51`); the test suite
  itself relies on calling `reset()` in `beforeEach` for isolation between tests
  (`packages/utils/src/createLogOnce.test.ts:5-7`).
- Cross-severity isolation: the same message emitted via warn-severity and error-severity loggers
  is emitted once per severity, not globally deduped
  (`packages/utils/src/createLogOnce.test.ts:39-42`).
- UNVERIFIED — no test asserts this: same-severity cross-logger dedup (see State model above),
  dedup behavior for different messages from the same logger, whether the prefix participates in
  the dedup key, and behavior for zero or non-string arguments
  (`packages/utils/src/createLogOnce.test.ts:1-53` covers none of these).
- Unmount: N/A — nothing is mounted; no test mounts a component
  (`packages/utils/src/createLogOnce.test.ts:1-53`).
- Nesting: N/A — no nested structures exist; the only composition exercised is creating multiple
  independent loggers (`packages/utils/src/createLogOnce.test.ts:39-42`).

## Shared harness dependencies

None. The behavior-covering test file imports only `vitest` APIs and the sibling unit under test
`./createLogOnce` — no `#test-utils`, no `packages/react/test/` harness file
(`packages/utils/src/createLogOnce.test.ts:1-2`). The unit source itself depends only on
`./createLogOnce` (`packages/utils/src/warn.ts:1`).
