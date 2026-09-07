# `formatNumber` — behavior spec

Unit: `packages/utils/src/formatNumber` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/formatNumber.test.ts` (the only test file for this unit).
No `wraps-external:` field exists in this unit's `TODO.md` entry (`TODO.md:54-58`), so no
third-party delegation note applies; the formatting itself is exercised against the platform
`Intl.NumberFormat` API in the assertions.

## Public API surface (props, parts, subcomponents)

- Two named exports: `getFormatter` and `formatNumber`
  (`packages/utils/src/formatNumber.test.ts:2`). No components, no props objects, no parts.
- `getFormatter(locale, options)` as exercised:
  - Accepts `undefined` as the locale (`packages/utils/src/formatNumber.test.ts:14`) and
    `Intl.Locale` instances (`packages/utils/src/formatNumber.test.ts:20-21`).
  - `options` is an `Intl.NumberFormatOptions` object; the tests use
    `{ currency: 'USD', style: 'currency', minimumFractionDigits: 2, maximumFractionDigits: 2 }`
    (`packages/utils/src/formatNumber.test.ts:4-9`).
  - Returns a formatter object exposing `resolvedOptions()`, whose `locale` reflects the input
    locale (`'fr-FR'`, `'en-US'`) (`packages/utils/src/formatNumber.test.ts:24-25`).
- `formatNumber(value, locale, options)` as exercised:
  - `value` accepts a number (`1234.56`, `0.1234`) or `null`
    (`packages/utils/src/formatNumber.test.ts:35,39,43`).
  - `locale` accepts `undefined` (`packages/utils/src/formatNumber.test.ts:35`) or a BCP 47
    string such as `'en-US'` (`packages/utils/src/formatNumber.test.ts:39,43`).
  - Returns a string in every asserted case (`packages/utils/src/formatNumber.test.ts:35,39,43`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — no controlled/uncontrolled semantics, defaults, or state transitions exist for a pure
formatting util. The only stateful behavior is `getFormatter`'s cache, and it is proven as:

- Identity caching per (locale, options): two calls with `undefined` locale and two separately
  constructed but structurally equal options objects return the *same* instance (`toBe`), so the
  cache key is derived from option content, not object identity
  (`packages/utils/src/formatNumber.test.ts:4-9,13-17`).
- Distinct `Intl.Locale` inputs (`fr-FR` vs `en-US`) with equal options produce *distinct*
  formatter instances (`not.toBe`) (`packages/utils/src/formatNumber.test.ts:19-26`).

## Keyboard interactions

N/A — non-visual string/number utility. No test asserts any keyboard behavior.

## Focus management

N/A — no focus behavior is implemented or asserted by any test.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test.

## DOM structure & portal behavior

N/A — the utility renders nothing, creates no DOM, and performs no portal behavior. It only
returns formatted strings (`packages/utils/src/formatNumber.test.ts:35,39,43`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are emitted, dispatched, or asserted by any test.

## Edge cases (rapid interactions, unmount, nesting)

Proven behaviors:

- `null` value formats to the empty string `''` (with an `'en-US'` locale and currency options)
  (`packages/utils/src/formatNumber.test.ts:42-44`).
- Currency formatting matches the platform: the output of
  `formatNumber(1234.56, undefined, getOptions())` is exactly equal (`toBe`) to
  `new Intl.NumberFormat(undefined, { style: 'currency', currency: 'USD' }).format(1234.56)`
  (`packages/utils/src/formatNumber.test.ts:30-36`).
- Percent style: `formatNumber(0.1234, 'en-US', { style: 'percent' })` returns `'12%'`
  (`packages/utils/src/formatNumber.test.ts:38-40`), i.e. the fractional part is dropped in the
  default percent configuration asserted here.
- Locale resolution: a formatter built from `new Intl.Locale('fr-FR')` reports
  `resolvedOptions().locale === 'fr-FR'`, and `en-US` likewise
  (`packages/utils/src/formatNumber.test.ts:24-25`).

Unproven behaviors (callers in this repo must not rely on them being specified here):

- `undefined`/`NaN`/`Infinity`/negative number values (only `null` among non-numbers is
  asserted): UNVERIFIED — inferred from `packages/utils/src/formatNumber.test.ts:30-44`, no test
  asserts these.
- Whether a string locale (`'en-US'`) and an equivalent `Intl.Locale('en-US')` share one cache
  entry, cache eviction/size limits, or the cache key for the `undefined` locale: UNVERIFIED —
  inferred from `packages/utils/src/formatNumber.test.ts:13-26`, no test asserts this (only the
  `undefined`-locale and distinct-`Intl.Locale` cache behaviors are proven).
- Invalid options/locale rejection behavior (throwing or fallback): UNVERIFIED — inferred from
  `packages/utils/src/formatNumber.test.ts:1-46`, no test asserts this.
- Rapid repeated calls, nesting, or unmount lifecycle: N/A — pure functions with no lifecycle;
  no test covers call-pattern edge cases (`packages/utils/src/formatNumber.test.ts:1-46`).

## Shared harness dependencies

None. The test file imports only `describe`/`it`/`expect` from `vitest` and
`getFormatter`/`formatNumber` from its sibling module
(`packages/utils/src/formatNumber.test.ts:1-2`). There is no `#test-utils`, `packages/react/test`,
or other shared harness import.
