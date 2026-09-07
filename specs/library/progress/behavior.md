# Progress — behavior spec

Mined from the Progress test suite only (`packages/react/src/progress/**/*.test.tsx`). Every
non-trivial claim cites the test lines that prove it; anything a test does not assert is marked
`UNVERIFIED`. The `library: progress` entry in `TODO.md` has no `wraps-external:` field, so all
behavior is derived from this unit's own tests.

## Public API surface (props, parts, subcomponents)

- `Progress.Root` — the owner component. Exercised props: `value` (number, or `null`/non-finite for
  indeterminate) `packages/react/src/progress/root/ProgressRoot.test.tsx:32-54`,
  `packages/react/src/progress/root/ProgressRoot.test.tsx:246-258`; `min`/`max`
  `packages/react/src/progress/root/ProgressRoot.test.tsx:121-178`; `format`
  (`Intl.NumberFormatOptions`) `packages/react/src/progress/root/ProgressRoot.test.tsx:287-331`;
  `locale` (BCP 47 string, e.g. `'de-DE'`)
  `packages/react/src/progress/root/ProgressRoot.test.tsx:333-354`; `getAriaValueText`
  (`(formattedValue: string | null, value: number | null) => string`)
  `packages/react/src/progress/root/ProgressRoot.test.tsx:261-284`. Forwards a ref to an
  `HTMLDivElement` `packages/react/src/progress/root/ProgressRoot.test.tsx:26-29`.
- `Progress.Label` — accessible label part; ref instance is `HTMLSpanElement`
  `packages/react/src/progress/label/ProgressLabel.test.tsx:10-15`; accepts an explicit `id`
  that becomes the label element's id `packages/react/src/progress/label/ProgressLabel.test.tsx:25,40-43`.
- `Progress.Value` — renders the formatted value; ref instance is `HTMLSpanElement`
  `packages/react/src/progress/value/ProgressValue.test.tsx:9-14`. Children may be omitted
  (renders the formatted value as text) `packages/react/src/progress/value/ProgressValue.test.tsx:17-26`
  or be a render function receiving `(formattedValue: string, value: number | null)`
  `packages/react/src/progress/value/ProgressValue.test.tsx:47-80`.
- `Progress.Track` — the bar container; ref instance is `HTMLDivElement`
  `packages/react/src/progress/track/ProgressTrack.test.tsx:8-13`.
- `Progress.Indicator` — the filled portion; ref instance is `HTMLDivElement`
  `packages/react/src/progress/indicator/ProgressIndicator.test.tsx:9-14`; supports the `render`
  prop (rendered as a `<span />` in tests) `packages/react/src/progress/indicator/ProgressIndicator.test.tsx:16-32`.
- All five parts compose as plain children of `Progress.Root` in tests (no compound registration
  API exercised beyond JSX nesting) `packages/react/src/progress/root/ProgressRoot.test.tsx:11-21`.

## State model (controlled/uncontrolled, defaults, transitions)

- Value is fully prop-driven (controlled); there is no internal value state or uncontrolled mode
  exercised. A `setProps({ value: 77 })` immediately updates `aria-valuenow` to `'77'`
  `packages/react/src/progress/root/ProgressRoot.test.tsx:56-61`. UNVERIFIED — inferred from
  `packages/react/src/progress/root/ProgressRoot.test.tsx:56-61`: no test asserts a `defaultValue`
  prop or any uncontrolled behavior.
- Three statuses drive `data-*` attributes on every part (root, label, value, track, indicator):
  `data-indeterminate`, `data-progressing`, `data-complete`. Exactly one is present at a time
  `packages/react/src/progress/root/ProgressRoot.test.tsx:64-118`.
- Status transitions proven by cycling `value` `null → 50 → 100 → null` with `setProps`; every part
  updates its attributes synchronously at each step
  `packages/react/src/progress/root/ProgressRoot.test.tsx:65-118`. `data-complete` is also present
  when the value exceeds max (45 with `max={40}`)
  `packages/react/src/progress/root/ProgressRoot.test.tsx:215-225`.
- Normalization: indicator width is the value normalized into `[min, max]` as a percentage — value 30
  in `[20, 40]` → width `'50%'` `packages/react/src/progress/root/ProgressRoot.test.tsx:121-138`;
  values outside the range clamp (50 in `[0, 40]` → `'100%'`, 10 in `[20, 40]` → `'0%'`)
  `packages/react/src/progress/root/ProgressRoot.test.tsx:140-178`.
- Defaults: `aria-valuemin` `'0'` and `aria-valuemax` `'100'` with only `value` provided
  `packages/react/src/progress/root/ProgressRoot.test.tsx:46-48`. Default formatting is
  locale percent of the normalized ratio (0.3 → `(0.3).toLocaleString(undefined, { style: 'percent' })`)
  `packages/react/src/progress/root/ProgressRoot.test.tsx:49-52`.
- `min === max` degenerate range normalizes to: `aria-valuenow` = the value (`'5'`), valuetext of
  0%, indicator width `'0%'`
  `packages/react/src/progress/root/ProgressRoot.test.tsx:227-244`.
- Format/locale are reactive: changing the `format` prop (`usd` → `eur`) updates the rendered value
  text in the same commit `packages/react/src/progress/root/ProgressRoot.test.tsx:312-330`.

## Keyboard interactions

N/A — no test in this suite exercises any keyboard event, key, or keyboard-driven behavior on any
Progress part.

## Focus management

N/A — no test asserts focusability, `tabIndex`, focus movement, or focus restoration for any part.
The progressbar is queried only via `getByRole('progressbar')`
`packages/react/src/progress/root/ProgressRoot.test.tsx:43`.

## Accessibility (roles, aria-*, id linking)

- The root element exposes `role="progressbar"` (queried by role)
  `packages/react/src/progress/root/ProgressRoot.test.tsx:43`.
- `aria-valuenow` is the clamped value: `'30'` in range
  `packages/react/src/progress/root/ProgressRoot.test.tsx:46`; `'40'`/`'20'` when the raw value
  overshoots/undershoots `packages/react/src/progress/root/ProgressRoot.test.tsx:153`,
  `packages/react/src/progress/root/ProgressRoot.test.tsx:173`; omitted entirely when indeterminate
  `packages/react/src/progress/root/ProgressRoot.test.tsx:83`,
  `packages/react/src/progress/root/ProgressRoot.test.tsx:253`.
- `aria-valuemin`/`aria-valuemax` reflect the range props (`'20'`/`'40'`)
  `packages/react/src/progress/root/ProgressRoot.test.tsx:154`,
  `packages/react/src/progress/root/ProgressRoot.test.tsx:174`.
- `aria-valuetext` defaults to the locale-formatted percent of the normalized value
  `packages/react/src/progress/root/ProgressRoot.test.tsx:49-52`, and is the string
  `'indeterminate progress'` when indeterminate
  `packages/react/src/progress/root/ProgressRoot.test.tsx:84`,
  `packages/react/src/progress/root/ProgressRoot.test.tsx:115`,
  `packages/react/src/progress/root/ProgressRoot.test.tsx:254`.
- `getAriaValueText` overrides `aria-valuetext`; it is invoked with `(formattedValue, rawValue)` —
  the formatted value is the *clamped* value's formatting while the second argument is the *raw*
  prop value (e.g. formatted 40, raw 50 when out of range)
  `packages/react/src/progress/root/ProgressRoot.test.tsx:180-213`. In the indeterminate state it is
  called with `('', null)`
  `packages/react/src/progress/root/ProgressRoot.test.tsx:279-282`.
- `aria-labelledby` on the root links to the `Progress.Label` element's id. The association is
  live: changing the label's `id` updates `aria-labelledby`, and unmounting the label removes the
  attribute entirely `packages/react/src/progress/label/ProgressLabel.test.tsx:17-47`.
- `Progress.Value` publishes the formatted value as its text content (empty when indeterminate),
  so screen readers announcing the part's text get the formatted number
  `packages/react/src/progress/root/ProgressRoot.test.tsx:85,95,105,116`.

## DOM structure & portal behavior

- Each part renders a real element with a forwarded ref: Root → `HTMLDivElement`
  `packages/react/src/progress/root/ProgressRoot.test.tsx:26-29`; Label → `HTMLSpanElement`
  `packages/react/src/progress/label/ProgressLabel.test.tsx:14`; Value → `HTMLSpanElement`
  `packages/react/src/progress/value/ProgressValue.test.tsx:13`; Track → `HTMLDivElement`
  `packages/react/src/progress/track/ProgressTrack.test.tsx:12`; Indicator → `HTMLDivElement`
  `packages/react/src/progress/indicator/ProgressIndicator.test.tsx:13`.
- The Indicator is positioned from the inline start and sized as a percentage width in the
  determinate state: computed `insetInlineStart: '0px'` and `width: '33%'` for `value={33}`
  `packages/react/src/progress/indicator/ProgressIndicator.test.tsx:17-32`; zero-width at `value={0}`
  (computed `width: '0px'`) `packages/react/src/progress/indicator/ProgressIndicator.test.tsx:34-49`.
  In the indeterminate state the indicator carries no inline width (`indicator.style.width === ''`)
  and no extra computed styles are asserted
  `packages/react/src/progress/root/ProgressRoot.test.tsx:86`,
  `packages/react/src/progress/indicator/ProgressIndicator.test.tsx:51-63`.
- Portal behavior: N/A — no test renders any Progress part through a portal; all parts are rendered
  inline as children of Root. UNVERIFIED — inferred from
  `packages/react/src/progress/root/ProgressRoot.test.tsx:11-21`, no test asserts portal support.
- All parts receive the same `data-*` status attributes as the root (attribute synchronization
  across the composed tree) `packages/react/src/progress/root/ProgressRoot.test.tsx:70-118`.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no test asserts any DOM event listeners, event-emitting callbacks (`onValueChange` or
similar), bubbling behavior, or `preventDefault` semantics for any Progress part. The only callback
prop asserted is `getAriaValueText`, a formatting hook invoked during render (payload shape covered
under Accessibility above)
`packages/react/src/progress/root/ProgressRoot.test.tsx:261-284`,
`packages/react/src/progress/root/ProgressRoot.test.tsx:180-213`.

## Edge cases (rapid interactions, unmount, nesting)

- Non-finite values (`NaN`, `Number.POSITIVE_INFINITY`, `Number.NEGATIVE_INFINITY`) are treated as
  indeterminate exactly like `null`: `data-indeterminate` present, no `aria-valuenow`, valuetext
  `'indeterminate progress'`, empty `Progress.Value`, no indicator width
  `packages/react/src/progress/root/ProgressRoot.test.tsx:246-258`.
- The `Progress.Value` render function receives `('indeterminate', rawValue)` as its arguments for
  both `value={null}` and `value={NaN}` (second argument preserves the raw sentinel, first argument
  is the fixed string `'indeterminate'`)
  `packages/react/src/progress/value/ProgressValue.test.tsx:66-79`.
- Repeated status cycling (indeterminate → progressing → complete → indeterminate in one mounted
  component) leaves every part correctly attributed at each step — the suite's stand-in for rapid
  interaction churn `packages/react/src/progress/root/ProgressRoot.test.tsx:65-118`.
- Out-of-range values are clamped, not rejected, for `aria-valuenow`, the formatted value text, and
  the indicator width `packages/react/src/progress/root/ProgressRoot.test.tsx:140-178`.
- Unmount of the label part clears `aria-labelledby` from the progressbar
  `packages/react/src/progress/label/ProgressLabel.test.tsx:45-46`.
- Rendering a part outside `Progress.Root` throws a descriptive error:
  `'Base UI: ProgressRootContext is missing. Progress parts must be placed within <Progress.Root>.'`
  (proven for `Progress.Label`; the other parts share the context but are not individually tested)
  `packages/react/src/progress/label/ProgressLabel.test.tsx:49-59`.
- Nesting: N/A — no test nests one Progress inside another or asserts any nested-context behavior.

## Shared harness dependencies

- All five test files import `createRenderer` and `describeConformance` from `#test-utils`, which
  maps to `packages/react/test/index.ts` (`"#test-utils": "./test/index.ts"` in
  `packages/react/package.json:110`) `packages/react/src/progress/root/ProgressRoot.test.tsx:4`,
  `packages/react/src/progress/indicator/ProgressIndicator.test.tsx:4`,
  `packages/react/src/progress/label/ProgressLabel.test.tsx:5`,
  `packages/react/src/progress/value/ProgressValue.test.tsx:4`,
  `packages/react/src/progress/track/ProgressTrack.test.tsx:3`.
- `createRenderer` wraps `@mui/internal-test-utils`'s renderer in `act()` and adds async
  `rerender`/`setProps` helpers used for the prop-transition tests
  `packages/react/test/createRenderer.ts:27-49`.
- `describeConformance` runs the shared per-part conformance suite (prop forwarding, ref
  forwarding, render prop, className) against each part
  `packages/react/test/describeConformance.tsx:44-68`; it is the sole proof of ref forwarding and
  `render`-prop support for the parts.
- `isJSDOM` (re-exported from `@base-ui/utils/testUtils` through the harness index
  `packages/react/test/index.ts:1`) gates the computed-style Indicator tests to non-JSDOM
  environments `packages/react/src/progress/indicator/ProgressIndicator.test.tsx:4,16`.
- `screen`/`fireEvent` come from `@mui/internal-test-utils`
  `packages/react/src/progress/root/ProgressRoot.test.tsx:2`,
  `packages/react/src/progress/label/ProgressLabel.test.tsx:4`.
