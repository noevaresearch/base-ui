# `stringifyLocale` — behavior spec

Unit: `packages/utils/src/stringifyLocale.ts` (native util, no `wraps-external:` in `TODO.md`). Behavior mined exclusively from `packages/utils/src/stringifyLocale.test.ts` (the unit's entire test suite: 1 test, 4 assertions).

## Public API surface (props, parts, subcomponents)

- A single named export, the pure function `stringifyLocale`, imported as `import { stringifyLocale } from './stringifyLocale'` with no other exports from the module. `packages/utils/src/stringifyLocale.test.ts:1-2`
- Callable with **zero arguments** (no-arg call is valid and returns a value). `packages/utils/src/stringifyLocale.test.ts:6`
- Callable with a **plain locale string** (e.g. `'en-US'`). `packages/utils/src/stringifyLocale.test.ts:7`
- Callable with an **`Intl.Locale` instance** (e.g. `new Intl.Locale('fr-FR')`). `packages/utils/src/stringifyLocale.test.ts:8`
- Callable with an **array** mixing strings and `Intl.Locale` instances (e.g. `['fr-FR', new Intl.Locale('en-US')]`). `packages/utils/src/stringifyLocale.test.ts:9`
- Always returns a `string` (all four assertions compare against `toBe(...)` string values). `packages/utils/src/stringifyLocale.test.ts:6-9`
- The test name states the intended purpose: "stringifies Intl locale arguments for cache keys". `packages/utils/src/stringifyLocale.test.ts:5`

Parts/subcomponents: N/A (non-component util).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — pure function with no internal state, props, or transitions; each call maps its input to an output deterministically. `packages/utils/src/stringifyLocale.test.ts:5-10`

## Keyboard interactions

N/A — non-DOM string utility; no keyboard behavior. `packages/utils/src/stringifyLocale.test.ts:5-10`

## Focus management

N/A — non-DOM string utility; no focus behavior. `packages/utils/src/stringifyLocale.test.ts:5-10`

## Accessibility (roles, aria-*, id linking)

N/A — non-DOM string utility; renders nothing and has no accessibility surface. `packages/utils/src/stringifyLocale.test.ts:5-10`

## DOM structure & portal behavior

N/A — produces a plain string only; no DOM nodes are created, queried, or portaled. `packages/utils/src/stringifyLocale.test.ts:5-10`

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — emits no events. `packages/utils/src/stringifyLocale.test.ts:5-10`

## Edge cases (rapid interactions, unmount, nesting)

Proven behaviors:

- **No argument / `undefined` locale** returns the empty string `''`. `packages/utils/src/stringifyLocale.test.ts:6`
- **String passthrough**: a locale string is returned unchanged (`'en-US'` → `'en-US'`), i.e. no normalization/canonicalization is applied. `packages/utils/src/stringifyLocale.test.ts:7`
- **`Intl.Locale` instance** is stringified to its locale identifier (`new Intl.Locale('fr-FR')` → `'fr-FR'`). `packages/utils/src/stringifyLocale.test.ts:8`
- **Array input** is flattened one level into a comma-joined string preserving element order, with `,` (no space) as the separator, and mixed element types (string + `Intl.Locale`) are handled uniformly (`['fr-FR', new Intl.Locale('en-US')]` → `'fr-FR,en-US'`). `packages/utils/src/stringifyLocale.test.ts:9`

Not covered by any test (no inference about actual behavior is made):

- UNVERIFIED — nested arrays (array containing an array): no test in `packages/utils/src/stringifyLocale.test.ts:5-10` asserts this.
- UNVERIFIED — `null` as an explicit argument (distinct from a no-arg call): no test in `packages/utils/src/stringifyLocale.test.ts:5-10` asserts this.
- UNVERIFIED — empty array input: no test in `packages/utils/src/stringifyLocale.test.ts:5-10` asserts this.
- UNVERIFIED — `Intl.Locale` instances constructed with extension/options data (e.g. `-u-` extensions, `script`/`region` options) or invalid locale strings (throwing vs. stringified): no test in `packages/utils/src/stringifyLocale.test.ts:5-10` asserts this.

Rapid interactions / unmount: N/A — stateless pure function with no lifecycle; only "edge cases" are the input-shape cases above.

## Shared harness dependencies

None. The test file imports only `vitest` primitives (`expect`, `describe`, `it`) and the unit itself; it does not import `#test-utils` or any file under `packages/react/test/`. `packages/utils/src/stringifyLocale.test.ts:1-2`
