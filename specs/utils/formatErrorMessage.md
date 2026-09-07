# `formatErrorMessage` — behavior spec

Unit: `packages/utils/src/formatErrorMessage` (Phase A util → crate `leptos-ui-utils`, spec
path per `TODO.md:49-53`). The unit's `TODO.md` entry has no `wraps-external:` field, so no
behavior is delegated to a third-party npm package and there is no external crate to bind
against — all behavior below is unit-local string formatting to be reimplemented in Stage 3.
Source of truth: the unit's sole test file `packages/utils/src/formatErrorMessage.test.ts`
(49 lines). Everything asserted here is proven by that file; anything beyond it is marked
UNVERIFIED.

## Public API surface (props, parts, subcomponents)

- Default export `formatErrorMessage`, invoked as
  `formatErrorMessage(code, ...args)` — a leading numeric error code followed by variadic
  string arguments, returning a string (`packages/utils/src/formatErrorMessage.test.ts:7`,
  `packages/utils/src/formatErrorMessage.test.ts:14`).
- Code-only message contract: `formatErrorMessage(123)` returns
  `Base UI error #123; visit https://base-ui.com/production-error?code=123 for the full message.`
  (`packages/utils/src/formatErrorMessage.test.ts:8-10`). The default formatter's prefix is
  `Base UI` and its base URL is `https://base-ui.com/production-error`
  (`packages/utils/src/formatErrorMessage.test.ts:9`).
- With args, the message contract extends the query with one `args%5B%5D=<arg>` pair per
  argument, in argument order:
  `formatErrorMessage(456, 'arg1', 'arg2')` returns
  `Base UI error #456; visit https://base-ui.com/production-error?code=456&args%5B%5D=arg1&args%5B%5D=arg2 for the full message.`
  (`packages/utils/src/formatErrorMessage.test.ts:15-17`).
- Named export `createFormatErrorMessage(baseUrl, prefix)`: a two-argument factory that
  returns a formatter function with the same call shape as the default export —
  `createFormatErrorMessage('https://example.com/errors', 'My Library')` produces
  `customFormatter`, and `customFormatter(789)` returns
  `My Library error #789; visit https://example.com/errors?code=789 for the full message.`
  (`packages/utils/src/formatErrorMessage.test.ts:23-27`). A second instance
  `('https://custom.dev/error-page', 'Custom UI')` behaves identically with args
  (`packages/utils/src/formatErrorMessage.test.ts:31-38`).
- The message scaffold is fixed across all four output assertions; only the prefix and base
  URL vary: `<prefix> error #<code>; visit <baseUrl>?code=<code>[&args%5B%5D=<arg>...] for
  the full message.` — the tokens `error`, `#`, `; visit `, `?code=`, and
  ` for the full message.` are constant in every expected string
  (`packages/utils/src/formatErrorMessage.test.ts:8-10`,
  `packages/utils/src/formatErrorMessage.test.ts:15-17`,
  `packages/utils/src/formatErrorMessage.test.ts:25-27`,
  `packages/utils/src/formatErrorMessage.test.ts:36-38`,
  `packages/utils/src/formatErrorMessage.test.ts:44-46`).
- No props, no parts, no subcomponents — this is a plain string-formatting utility, not a
  React component (`packages/utils/src/formatErrorMessage.test.ts:1-49`).
- UNVERIFIED — no test asserts the relationship between the default export and
  `createFormatErrorMessage` (e.g. whether the default export is the result of
  `createFormatErrorMessage` with the defaults above); Stage 3 may implement the default
  formatter directly or via the factory, provided outputs match
  (`packages/utils/src/formatErrorMessage.test.ts:1-49`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A as a controlled/uncontrolled concept — this is a pure string-formatting utility. No
state, defaults, or transitions are observable: every invocation returns a string derived
only from the factory arguments (prefix, base URL) and the call arguments (code, args)
(`packages/utils/src/formatErrorMessage.test.ts:6-47`). No test observes mutable state,
caching, or interference between calls on the same formatter instance
(`packages/utils/src/formatErrorMessage.test.ts:1-49`).

## Keyboard interactions

N/A — non-visual utility. No keyboard behavior exists or is tested
(`packages/utils/src/formatErrorMessage.test.ts:1-49`).

## Focus management

N/A — non-visual utility. Nothing focusable exists in this unit
(`packages/utils/src/formatErrorMessage.test.ts:1-49`).

## Accessibility (roles, aria-*, id linking)

N/A — the unit returns a plain message string; no roles, aria attributes, or id linking are
produced or asserted. Whether callers throw/present this string in an accessible way is out
of this unit's tested scope (`packages/utils/src/formatErrorMessage.test.ts:1-49`).

## DOM structure & portal behavior

N/A — the utility creates no DOM elements, renders nothing, and performs no portal behavior
(`packages/utils/src/formatErrorMessage.test.ts:1-49`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — the unit emits no events (DOM or custom). Its only observable output is the returned
message string (`packages/utils/src/formatErrorMessage.test.ts:1-49`).

## Edge cases (rapid interactions, unmount, nesting)

- Zero args: the query string is exactly `?code=<code>` with no `args%5B%5D` pairs
  (`packages/utils/src/formatErrorMessage.test.ts:8-10`).
- Multiple args: each argument becomes its own `args%5B%5D` pair appended after `code`, in
  call order, joined with `&` (`packages/utils/src/formatErrorMessage.test.ts:15-17`).
- Special characters in args are encoded, not passed through: a space becomes `+`
  (`hello world` → `hello+world`) and `&` becomes `%26` (`foo&bar` → `foo%26bar`) —
  `application/x-www-form-urlencoded` (URLSearchParams-style) value encoding
  (`packages/utils/src/formatErrorMessage.test.ts:44-46`). The `args` key itself is emitted
  percent-encoded as the literal `args%5B%5D` in every args-bearing output
  (`packages/utils/src/formatErrorMessage.test.ts:16`,
  `packages/utils/src/formatErrorMessage.test.ts:37`,
  `packages/utils/src/formatErrorMessage.test.ts:45`).
- Repeated/rapid invocation: N/A in the interaction sense — no lifecycle, no components.
  Distinct formatter instances created from the same factory arguments behave identically;
  no test observes cross-call interference (`packages/utils/src/formatErrorMessage.test.ts:1-47`).
- Unmount/nesting: N/A — no component tree exists in this unit
  (`packages/utils/src/formatErrorMessage.test.ts:1-49`).
- Untested areas — do not rely on them in Stage 3 without adding tests first: non-string or
  `null`/`undefined` args, empty-string args, non-integer or negative codes, base URLs that
  already contain a query string or fragment, and prefixes containing special characters.
  UNVERIFIED — no test asserts any of these (`packages/utils/src/formatErrorMessage.test.ts:1-49`).

## Shared harness dependencies

None. The test file's only imports are `expect`, `describe`, `it` from `vitest` and the unit
itself `./formatErrorMessage` (`packages/utils/src/formatErrorMessage.test.ts:1-2`). No
`#test-utils` alias and no file under `packages/react/test/` is imported, so no shared
harness file was read.
