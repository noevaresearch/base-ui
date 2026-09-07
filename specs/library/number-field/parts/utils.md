# NumberField — private utils (parse, validate, getViewportRect) — behavior mined from tests

Scope: pure-utility tests in `packages/react/src/number-field/utils/` — `parse.test.ts`, `validate.test.ts`, `getViewportRect.test.ts`. These are private utilities of the NumberField unit (not `packages/utils/src` public utils). No React rendering occurs in these tests; everything below is proven input→output behavior of the three modules.

## Public API surface (props, parts, subcomponents)

N/A for props/parts/subcomponents — no component is rendered. The tests pin the observable signatures of six exported functions:

- `getNumberLocaleDetails(locale: string)` returns `{ decimal, group, currency, percent, unit }`; for `en-US` it is `{ decimal: '.', group: ',', currency: undefined, percent: undefined, unit: undefined }` `packages/react/src/number-field/utils/parse.test.ts:7-14`.
- `parseNumber(input: string, locale?: string, formatOptions?: Intl.NumberFormatOptions)` returns `number | null` (null on invalid/empty/Infinity-like input) — see usages throughout `packages/react/src/number-field/utils/parse.test.ts:18-323`.
- `isNumeralChar(char: string)` returns `boolean` `packages/react/src/number-field/utils/parse.test.ts:326-341`.
- `removeFloatingPointErrors(value: number, formatOptions?: Intl.NumberFormatOptions)` returns `number` `packages/react/src/number-field/utils/validate.test.ts:44-429`.
- `toValidatedNumber(...)` is called through a test adapter with a 9-argument positional signature: `(value, step, minWithDefault, maxWithDefault, minWithZeroDefault, format, snapOnStep, small, clamp)` `packages/react/src/number-field/utils/validate.test.ts:30-41`. Default fixture options: `step: 1`, `minWithDefault: Number.MIN_SAFE_INTEGER`, `maxWithDefault: Number.MAX_SAFE_INTEGER`, `minWithZeroDefault: 0`, `format: undefined`, `snapOnStep: true`, `small: false`, `clamp: true` `packages/react/src/number-field/utils/validate.test.ts:4-16`.
- `getViewportRect(teleportDistance: number | undefined, element: HTMLElement)` returns `{ left, top, right, bottom }` `packages/react/src/number-field/utils/getViewportRect.test.ts:18-38`.

## State model (controlled/uncontrolled, defaults, transitions)

N/A for React state — these are pure functions with no internal state. The closest analogs proven by tests:

- Null pass-through: `toValidatedNumber(null, ...)` returns `null` `packages/react/src/number-field/utils/validate.test.ts:432-434`.
- Value-validation pipeline ordering (each step proven by a dedicated test):
  1. Step snapping happens before format rounding — `roundingMode` is applied *after* step validation (1.239 with step 0.001, format `{ maximumFractionDigits: 2, roundingMode: 'floor' }` → 1.23) `packages/react/src/number-field/utils/validate.test.ts:603-615`.
  2. Format rounding happens before the final clamp — percent rounding that crosses max/min is clamped back afterward (0.01236 → 0.0124 rounds past max 0.01235, final result 0.01235 `packages/react/src/number-field/utils/validate.test.ts:617-630`; 0.01234 floors to 0.0123 below min 0.01235, final result 0.01235 `packages/react/src/number-field/utils/validate.test.ts:632-647`).
  3. Clamping also happens *before* rounding for non-integer bounds — 0.4 with min 0.6 and `maximumFractionDigits: 0` clamps to 0.6 first, then rounds to 1 (round-first would produce 0 then clamp to 0.6, disagreeing with the displayed "1") `packages/react/src/number-field/utils/validate.test.ts:659-674`.
  4. Clamping happens after snapping, so non-step-aligned bounds are reachable — raw 13 with step 3 lands on max 10, not on the step-aligned 9 `packages/react/src/number-field/utils/validate.test.ts:761-772`; raw −13 with step −3 lands on min −10 `packages/react/src/number-field/utils/validate.test.ts:774-785`.
  5. Floating-point cleanup runs on stepped values (`0.2 + 0.1` → 0.3) `packages/react/src/number-field/utils/validate.test.ts:649-657`.
- `clamp: false` disables both clamp passes but keeps rounding: 12.349 with format `{ maximumFractionDigits: 2 }`, min 0 / max 10, `clamp: false` → 12.35 (rounded to format precision but deliberately kept out of range so native overflow validation can report it) `packages/react/src/number-field/utils/validate.test.ts:744-759`; and 12 with min 0 / max 10, `clamp: false` → 12 `packages/react/src/number-field/utils/validate.test.ts:436-447`.

## Keyboard interactions

N/A — no keyboard behavior is exercised by these pure-utility tests.

## Focus management

N/A — no focus behavior is exercised by these pure-utility tests.

## Accessibility (roles, aria-*, id linking)

N/A — no ARIA behavior is exercised by these pure-utility tests.

## DOM structure & portal behavior

No component DOM is rendered and nothing is portaled. The only DOM-touching util is `getViewportRect`, which measures an existing element:

- Input fixture: a `position:fixed; left:100px; top:100px; width:20px; height:20px` div appended to `document.body` `packages/react/src/number-field/utils/getViewportRect.test.ts:8-12`.
- With a positive `teleportDistance` (100), the element's `getBoundingClientRect()` is padded by **half** the teleport distance on every side: `{ left: rect.left - 50, top: rect.top - 50, right: rect.right + 50, bottom: rect.bottom + 50 }` `packages/react/src/number-field/utils/getViewportRect.test.ts:18-27`.
- With `teleportDistance: 0`, the result collapses to the exact element rect `packages/react/src/number-field/utils/getViewportRect.test.ts:29-38`.
- With `teleportDistance: undefined` in a browser, the **visual viewport** is used: mocked `offsetLeft: 7, offsetTop: 11, width: 300, height: 400` yields `{ left: 7, top: 11, right: 307, bottom: 411 }` (browser-only, `it.skipIf(isJSDOM)`) `packages/react/src/number-field/utils/getViewportRect.test.ts:40-65`.
- With `teleportDistance: undefined` in jsdom (no `window.visualViewport`), it falls back to the document element: `{ left: 0, top: 0, right: document.documentElement.clientWidth, bottom: document.documentElement.clientHeight }` (jsdom-only, `it.skipIf(!isJSDOM)`) `packages/react/src/number-field/utils/getViewportRect.test.ts:67-76`.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are dispatched or observed by these pure-utility tests.

## Edge cases (rapid interactions, unmount, nesting)

Unmount/nesting/rapid-interaction cases do not apply (pure functions). The exhaustive edge-case inventory that IS proven:

### parseNumber — accepted formats

- Default `Intl.NumberFormat().format(1234.56)` round-trips to 1234.56 `packages/react/src/number-field/utils/parse.test.ts:18-21`.
- Percent handling:
  - `12%` → 0.12 **by default** (trailing `%` divides by 100 even without `style` options) `packages/react/src/number-field/utils/parse.test.ts:23-25`.
  - `12%` with `{ style: 'percent' }` → 0.12 `packages/react/src/number-field/utils/parse.test.ts:49-51`.
  - `12%` with `{ style: 'unit', unit: 'percent' }` → **12** (no ÷100 scaling for unit-percent) `packages/react/src/number-field/utils/parse.test.ts:64-66`.
  - Interleaved percent sign is stripped: `1%2` → 0.12 (percent style) / 12 (unit percent style) `packages/react/src/number-field/utils/parse.test.ts:68-71`.
  - Prefix-percent locales round-trip: tr-TR `Intl` percent format of 0.0123 parses back to 0.0123 `packages/react/src/number-field/utils/parse.test.ts:53-58`.
  - Percent variants: fullwidth `１２％` `packages/react/src/number-field/utils/parse.test.ts:75`, `12％` `packages/react/src/number-field/utils/parse.test.ts:237`, small `12﹪` `packages/react/src/number-field/utils/parse.test.ts:238`, Arabic `12٪` `packages/react/src/number-field/utils/parse.test.ts:239`.
  - Scientific percent: `1e-7%` → 1e-9 `packages/react/src/number-field/utils/parse.test.ts:60-62`; scientific permille `1e-7‰` → 1e-10 `packages/react/src/number-field/utils/parse.test.ts:171-173`.
- Per-mille: `12‰` → 0.012 and `12؉` → 0.012 (÷1000) `packages/react/src/number-field/utils/parse.test.ts:89-92`.
- Numeral systems:
  - Arabic-Indic `١٬٢٣٤٫٥٦` → 1234.56 (test unconditionally skips via `skip()` — comment: browsers don't support Arabic numerals) `packages/react/src/number-field/utils/parse.test.ts:27-31`; `١٢٪` → 0.12 `packages/react/src/number-field/utils/parse.test.ts:37-39`.
  - Han numerals: `一,二三四.五六` → 1234.56 `packages/react/src/number-field/utils/parse.test.ts:33-35`; `一二%` → 0.12 `packages/react/src/number-field/utils/parse.test.ts:41-43`; full range `九八七六五四三二一〇` → 9876543210 `packages/react/src/number-field/utils/parse.test.ts:316-318`; both zero forms `零` and `〇` → 0 `packages/react/src/number-field/utils/parse.test.ts:320-323`.
  - Fullwidth digits + punctuation: `１，２３４．５６` → 1234.56 `packages/react/src/number-field/utils/parse.test.ts:73-76`; full range `０１２３４５６７８９` → 123456789 `packages/react/src/number-field/utils/parse.test.ts:307-309`.
  - Persian digits: `۱۲۳۴` → 1234, `۱۲٫۳۴` → 12.34, `۱۲٪` → 0.12 `packages/react/src/number-field/utils/parse.test.ts:78-82`; Persian digits + Arabic separators `۱۲٬۳۴۵٫۶۷` → 12345.67 `packages/react/src/number-field/utils/parse.test.ts:84-87`; full Persian range `۹۸۷۶۵۴۳۲۱۰` → 9876543210 `packages/react/src/number-field/utils/parse.test.ts:303-305`; full Arabic-Indic range `٩٨٧٦٥٤٣٢١٠` → 9876543210 (jsdom-only via `it.skipIf(!isJSDOM)`, comment: browser Intl is inconsistent) `packages/react/src/number-field/utils/parse.test.ts:311-314`.
  - Arabic punctuation with ASCII digits: `1٬234٫56` → 1234.56 `packages/react/src/number-field/utils/parse.test.ts:242-244`.
- Signs (leading AND trailing positions accepted):
  - ASCII: `+1234` → 1234, `1234+` → 1234, `-1234` → -1234, `1234-` → -1234 `packages/react/src/number-field/utils/parse.test.ts:217-222`.
  - Unicode minus: U+2212 `−1234` and trailing `1234−` → -1234; fullwidth plus `＋1234` and trailing `1234＋` → 1234 `packages/react/src/number-field/utils/parse.test.ts:99-104`. Minus variants: figure dash ‒, en dash –, em dash —, fullwidth hyphen －, small hyphen ﹣ all → negative `packages/react/src/number-field/utils/parse.test.ts:224-230`. Plus variant: ﹢ → 123 `packages/react/src/number-field/utils/parse.test.ts:232-234`.
- Separators / grouping:
  - Mixed-locale dots: `1.234.567.89` → 1234567.89 (last `.` is the decimal, previous ones removed) `packages/react/src/number-field/utils/parse.test.ts:194-197`.
  - Consecutive dots collapse, keeping only the last as decimal: `1..5` → 1.5, `123..456..789.01` → 123456789.01, `....5` → 0.5 `packages/react/src/number-field/utils/parse.test.ts:293-297`.
  - Mixed separators are locale-sensitive: `1.234.567,89` → 1234567.89 under `fr-FR` but 1234.56789 under `en-US` `packages/react/src/number-field/utils/parse.test.ts:199-205`.
  - `de-DE`: `1.234,56` → 1234.56, `1.234.567,89` → 1234567.89 `packages/react/src/number-field/utils/parse.test.ts:259-262`.
  - French space groupings: narrow no-break `\u202F`, thin `\u2009`, figure `\u2007`, no-break `\u00A0` all stripped (e.g. `1\u202F234,56` → 1234.56) `packages/react/src/number-field/utils/parse.test.ts:246-251`; `fr-FR` `Intl` output round-trips, including with a trailing minus → -1234.5 `packages/react/src/number-field/utils/parse.test.ts:106-110`.
  - Swiss apostrophe grouping: both `1’234.56` (U+2019) and `1'234.56` (straight) under `de-CH` → 1234.56 `packages/react/src/number-field/utils/parse.test.ts:253-257`.
- Invisible characters: LRM/RLM (`\u200E`, `\u200F`) stripped `packages/react/src/number-field/utils/parse.test.ts:94-97`; mid-input bidi format controls LRE…PDF ignored (`1\u202A234\u202C.56` → 1234.56) `packages/react/src/number-field/utils/parse.test.ts:282-285`.
- Currency & units (only when options specify them):
  - `$1,234.56` with `{ style: 'currency', currency: 'USD' }` → 1234.56 `packages/react/src/number-field/utils/parse.test.ts:112-116`; EUR prefix (en-US) and suffix (fr-FR) formats round-trip `packages/react/src/number-field/utils/parse.test.ts:264-273`; scientific currency codes (`EUR` code + scientific notation) round-trip to 12300 `packages/react/src/number-field/utils/parse.test.ts:118-129`.
  - Units: `12 kg` with `{ style: 'unit', unit: 'kilogram' }` → 12 `packages/react/src/number-field/utils/parse.test.ts:175-177`; compound units `12 km/h`, `12 m/s` → 12 `packages/react/src/number-field/utils/parse.test.ts:275-280`.
  - Localized scientific notation: `ar-EG` and `fa-IR` formats of 12345 parse to 12300 `packages/react/src/number-field/utils/parse.test.ts:131-139`; percent-scientific 0.0000012345 → 0.00000123 `packages/react/src/number-field/utils/parse.test.ts:141-154`; tiny percent-scientific 0.0000000000012345 → 0.00000000000123 `packages/react/src/number-field/utils/parse.test.ts:156-169`.

### parseNumber — rejected formats (all → null)

- Invalid text: `'invalid'` `packages/react/src/number-field/utils/parse.test.ts:45-47`.
- Empty / whitespace-only: `''` and `'   '` `packages/react/src/number-field/utils/parse.test.ts:207-210`.
- Lone signs: `'-'` and `'+'` `packages/react/src/number-field/utils/parse.test.ts:212-215`.
- Infinity-like: `'Infinity'`, `'-Infinity'`, `'∞'` `packages/react/src/number-field/utils/parse.test.ts:179-183`; with sign/spaces: `' +Infinity '`, `' -∞ '`, `'+Infinity'` `packages/react/src/number-field/utils/parse.test.ts:287-291`.
- Overflow to Infinity: `Number.MAX_VALUE` formatted as percent parses to null `packages/react/src/number-field/utils/parse.test.ts:185-192`.

### isNumeralChar

- Accepts a digit from every supported system: ASCII `0`/`9`, Arabic-Indic `٠`/`٩`, Persian `۰`/`۹`, fullwidth `０`/`９`, Han `零`/`〇`/`九` → true `packages/react/src/number-field/utils/parse.test.ts:327-332`.
- Rejects separators, signs, letters, space: `.`, `,`, `-`, `+`, `%`, `٫`, `a`, `' '` → false (comment: otherwise input validation would accept unparseable strings) `packages/react/src/number-field/utils/parse.test.ts:334-340`.

### removeFloatingPointErrors

- Binary-noise cleanup: `0.2 + 0.1` → 0.3 `packages/react/src/number-field/utils/validate.test.ts:45-47`; `-0.1 - 0.2` → -0.3 `packages/react/src/number-field/utils/validate.test.ts:49-51`; `0.1 + 0.7` → 0.8 and `0.1 + 0.2 + 0.3` → 0.6 (16-digit noise patterns) `packages/react/src/number-field/utils/validate.test.ts:63-66`.
- Precision preservation without a format: 0.0005 and 1.23456 pass through unchanged (precision finer than 3 fraction digits preserved) `packages/react/src/number-field/utils/validate.test.ts:53-56`; `MAX_SAFE_INTEGER`/`MIN_SAFE_INTEGER` exact `packages/react/src/number-field/utils/validate.test.ts:58-61`.
- Delta cap: from the 2^19 binade (~5.2e5) up, a single ULP exceeds the cleanup delta, so `1000000.1 + 0.2` is returned raw and NOT cleaned to 1000000.3 (avoids corrupting real precision) `packages/react/src/number-field/utils/validate.test.ts:68-74`.
- Non-finite values untouched: `Infinity` → `Infinity`, `-Infinity` → `-Infinity`, `NaN` → NaN `packages/react/src/number-field/utils/validate.test.ts:76-80`.
- Format-aware rounding:
  - `maximumFractionDigits: 1` on `0.2 + 0.1` → 0.3 `packages/react/src/number-field/utils/validate.test.ts:90-92`.
  - `roundingMode: 'floor'` respected: 1.239 → 1.23 `packages/react/src/number-field/utils/validate.test.ts:94-101`; for negatives: -1.239 floor → -1.24, trunc → -1.23 `packages/react/src/number-field/utils/validate.test.ts:118-131`.
  - `halfEven` at ties: 1.235 → 1.24 and 1.245 → 1.24 `packages/react/src/number-field/utils/validate.test.ts:103-116`.
  - Compact notation cannot round-trip ("1.2K"), so rounding resolves with standard notation: 1234.567 → 1234.6 `packages/react/src/number-field/utils/validate.test.ts:82-88`.
  - `roundingIncrement` respected: 1.26 with minFrac 1 / maxFrac 1 / increment 5 → 1.5 `packages/react/src/number-field/utils/validate.test.ts:358-366`.
  - `roundingMode` respected even with no digit-precision options: 1.2399 with only `minimumIntegerDigits: 1` + floor → 1.239 `packages/react/src/number-field/utils/validate.test.ts:349-356`.
  - Only-`minimumFractionDigits` provided: resolved max is used — 1.234567 with minFrac 5 → 1.23457 `packages/react/src/number-field/utils/validate.test.ts:426-428`.
  - `roundingPriority: 'morePrecision'` honored across fraction+significant digits: 1.2399 → 1.239 `packages/react/src/number-field/utils/validate.test.ts:276-285`.
  - Significant digits: 12345 with maxSig 3 + floor → 12300 `packages/react/src/number-field/utils/validate.test.ts:172-179`.
- Percent style rounds at **display scale** (×100 then back):
  - 0.01236 with percent/maxFrac 2 → 0.0124; 0.01239 with floor → 0.0123 `packages/react/src/number-field/utils/validate.test.ts:133-147`; also with only `minimumFractionDigits: 2` `packages/react/src/number-field/utils/validate.test.ts:162-170`.
  - Exact boundaries preserved across `floor/ceil/trunc/expand`: 0.0046 → 0.0046 for each `packages/react/src/number-field/utils/validate.test.ts:149-160`.
  - With maxFrac 16: 0.001234567890123456 unchanged `packages/react/src/number-field/utils/validate.test.ts:233-241`; 0.0046 with floor unchanged `packages/react/src/number-field/utils/validate.test.ts:243-256`.
  - Significant-digit percent: 0.01239 (percent, maxSig 3, floor) → 0.0123 `packages/react/src/number-field/utils/validate.test.ts:181-189`; 0.009995 (percent, maxSig 2, floor) → 0.0099 with Intl output identical to the raw value `packages/react/src/number-field/utils/validate.test.ts:191-203`; tiny 0.000001234 → 0.0000012 preserved `packages/react/src/number-field/utils/validate.test.ts:205-217`; `roundingPriority` with percent significant digits falls back to percent fraction defaults: 0.0123456 → 0.0123 `packages/react/src/number-field/utils/validate.test.ts:262-274`.
  - Directional-rounding boundary: 0.01230000001 with percent/maxFrac 2/ceil → 0.0124 (meaningful precision above the boundary preserved; Intl output matches raw) `packages/react/src/number-field/utils/validate.test.ts:219-231`.
  - Overflow guard: `Number.MAX_VALUE` with percent scaling returns the original finite value `packages/react/src/number-field/utils/validate.test.ts:368-376`.
- NOT percent-scaled: `style: 'unit', unit: 'percent'` (1.239 → 1.23) `packages/react/src/number-field/utils/validate.test.ts:287-296` and `style: 'currency'` (1.239 → 1.23) `packages/react/src/number-field/utils/validate.test.ts:298-307`.
- Other styles: scientific notation rounds at the formatted scale — 0.000123456 → 0.000123 `packages/react/src/number-field/utils/validate.test.ts:378-389`; scientific currency codes: 12345 → 12300 `packages/react/src/number-field/utils/validate.test.ts:309-319`; non-invertible scientific zero buckets return the original value (12345 with minFrac 0/maxFrac 0/roundingIncrement 5) `packages/react/src/number-field/utils/validate.test.ts:391-405`, while invertible ones round (1 → 0 with the same format) `packages/react/src/number-field/utils/validate.test.ts:407-416`.
- Sign handling: values are preserved when `signDisplay: 'never'` hides the sign (-1.239 → -1.24, Intl output matches raw) `packages/react/src/number-field/utils/validate.test.ts:321-332`; negative accounting currency rounds to -1.24 `packages/react/src/number-field/utils/validate.test.ts:334-347`.
- Grouping/style ignored when they don't affect the number: 1000 → 1000 `packages/react/src/number-field/utils/validate.test.ts:418-420`; 1000 with currency USD → 1000 `packages/react/src/number-field/utils/validate.test.ts:422-424`.
- Intl cap: values preserved when Intl supports > 20 fraction digits — 1e-21 with maxFrac 21 → 1e-21 `packages/react/src/number-field/utils/validate.test.ts:258-260`.

### toValidatedNumber — snapping to step

- Null input → null `packages/react/src/number-field/utils/validate.test.ts:432-434`.
- Increment direction (positive `step`) snaps **down** (floor-like) to a step multiple: 5 → 5 `packages/react/src/number-field/utils/validate.test.ts:486-488`; 5.5 → 5 `packages/react/src/number-field/utils/validate.test.ts:490-492`; -0.3 → -1 `packages/react/src/number-field/utils/validate.test.ts:494-496`; step 5: 9 → 5 `packages/react/src/number-field/utils/validate.test.ts:507-514`, 12 → 10 `packages/react/src/number-field/utils/validate.test.ts:516-523`.
- Decrement direction (negative `step`) snaps **up** (ceil-like): step -1: 5 → 5 `packages/react/src/number-field/utils/validate.test.ts:537-544`; 5.5 → 6 `packages/react/src/number-field/utils/validate.test.ts:546-553`; -0.3 → 0 `packages/react/src/number-field/utils/validate.test.ts:555-562`; step -5: 9 → 10 `packages/react/src/number-field/utils/validate.test.ts:573-580`, 12 → 15 `packages/react/src/number-field/utils/validate.test.ts:582-589`.
- `step: undefined` (within bounds) leaves values unchanged: 5.5 → 5.5 both ways `packages/react/src/number-field/utils/validate.test.ts:498-505`, `packages/react/src/number-field/utils/validate.test.ts:564-571`.
- `snapOnStep: false` preserves the exact value: 9.7 with step 5 `packages/react/src/number-field/utils/validate.test.ts:525-533`; 12.3 with step -5 `packages/react/src/number-field/utils/validate.test.ts:591-599`.
- `small: true` switches snapping to **nearest** rounding: 0.15 with step 0.1 → 0.2 `packages/react/src/number-field/utils/validate.test.ts:721-730`; -0.15 with step -0.1 → -0.2 `packages/react/src/number-field/utils/validate.test.ts:732-741`.
- Fractional steps don't get stuck on floating-point noise: 100.1 + 0.1 (raw 100.19999999999999) with step 0.1 snaps to 100.2, not back to 100.1 `packages/react/src/number-field/utils/validate.test.ts:677-687`; 100.1 − 0.1 with step -0.1 → 100 `packages/react/src/number-field/utils/validate.test.ts:689-698`; 0.01 + 0.01 with step 0.01 → 0.02 `packages/react/src/number-field/utils/validate.test.ts:700-708`; 3 + 0.2 + 0.2 with step 0.2 and min 3 → 3.4 `packages/react/src/number-field/utils/validate.test.ts:710-719`.

### toValidatedNumber — precision preservation

- Parsed input with no step arithmetic preserves >15 significant digits: 1.234567890123456 and 0.1234567890123456 pass through `packages/react/src/number-field/utils/validate.test.ts:449-457`.
- Stepped arithmetic noise is cleaned: 0.1 + 0.7 with step 0.1 → 0.8 `packages/react/src/number-field/utils/validate.test.ts:459-463`.
- But coarse cleanup does not destroy large-magnitude precision: 100000000000000.1 + 0.1 with step 0.1 returns the raw stepped value unchanged `packages/react/src/number-field/utils/validate.test.ts:465-471`; a high-significance `step` of 0.1234567890123456 passes through unchanged `packages/react/src/number-field/utils/validate.test.ts:473-483`.

### getViewportRect — environment fallbacks

- jsdom (no `window.visualViewport`) falls back to the document element's client dimensions `packages/react/src/number-field/utils/getViewportRect.test.ts:67-76`; browsers use the real/mocked visual viewport `packages/react/src/number-field/utils/getViewportRect.test.ts:40-65`; the visual-viewport test asserts the result cannot coincide with the document element's bounds by mocking a pinch-zoomed viewport (offset 7/11, size 300×400) `packages/react/src/number-field/utils/getViewportRect.test.ts:47-53`.

## Shared harness dependencies

- `#test-utils` maps to `packages/react/test/index.ts` (`packages/react/package.json:110`). It is a barrel re-exporting `@base-ui/utils/testUtils` plus local helpers (`createRenderer`, `firePointer`, `wait`, etc.) `packages/react/test/index.ts:1-14`.
- The only harness symbol these three test files consume is `isJSDOM`, imported in `packages/react/src/number-field/utils/parse.test.ts:2` and `packages/react/src/number-field/utils/getViewportRect.test.ts:2`. `isJSDOM` is defined as a user-agent sniff `/jsdom/.test(window.navigator.userAgent)` in `packages/utils/src/testUtils.ts:4` and re-exported through the barrel at `packages/react/test/index.ts:1`.
- `isJSDOM` drives environment-skipped tests only: the jsdom-only full Arabic-Indic digit-range test `packages/react/src/number-field/utils/parse.test.ts:311-314`, the browser-only visual-viewport test `packages/react/src/number-field/utils/getViewportRect.test.ts:40`, and the jsdom-only document-element fallback test `packages/react/src/number-field/utils/getViewportRect.test.ts:67`.
- `validate.test.ts` imports no harness at all (only vitest + the module under test) `packages/react/src/number-field/utils/validate.test.ts:1-2`.
- No other shared harness file (renderers, pointer helpers, conformance suites) is used by this batch.
