# Meter behavior spec

Mined from the five test files under `packages/react/src/meter/`. The `TODO.md` entry for
`library: meter` (`TODO.md:426-432`) has no `wraps-external:` field and no
`needs-batched-mining: true`, so the behavior below is derived entirely from the component's
own tests.

## Public API surface (props, parts, subcomponents)

- `Meter.Root` is the context provider; the other parts must be rendered inside it. Rendering
  `Meter.Label` outside `Meter.Root` rejects with "Base UI: MeterRootContext is missing. Meter
  parts must be placed within <Meter.Root>." `packages/react/src/meter/label/MeterLabel.test.tsx:49-59`
- Parts exercised by tests: `Meter.Label`, `Meter.Track`, `Meter.Indicator`, `Meter.Value`, all
  nested under `Meter.Root`, with `Meter.Indicator` inside `Meter.Track`.
  `packages/react/src/meter/root/MeterRoot.test.tsx:21-26`
- Parts are optional and independently composable: `Meter.Root` with no children still exposes
  the meter role. `packages/react/src/meter/root/MeterRoot.test.tsx:216-221`; `Meter.Value` also
  works without a Track/Indicator. `packages/react/src/meter/root/MeterRoot.test.tsx:45-49`
- `Meter.Root` props exercised by the tests: `value`, `min`, `max`, `locale`, `format`
  (`Intl.NumberFormatOptions`), and `getAriaValueText(formattedValue, value)`.
  `packages/react/src/meter/root/MeterRoot.test.tsx:21`, `packages/react/src/meter/root/MeterRoot.test.tsx:126`, `packages/react/src/meter/root/MeterRoot.test.tsx:236`, `packages/react/src/meter/root/MeterRoot.test.tsx:103-105`
- All five parts pass the shared conformance suite (props spread, ref forwarding, render prop,
  className), so `ref`, `className`, `style`, `render`, and arbitrary props (e.g. `data-testid`)
  forward to each part's root element. `packages/react/src/meter/root/MeterRoot.test.tsx:13-16`, `packages/react/src/meter/indicator/MeterIndicator.test.tsx:9-14`, `packages/react/src/meter/label/MeterLabel.test.tsx:10-15`, `packages/react/src/meter/track/MeterTrack.test.tsx:8-13`, `packages/react/src/meter/value/MeterValue.test.tsx:9-14`
- `Meter.Value` children may be omitted (renders the formatted value), a node, or a render
  function. `packages/react/src/meter/value/MeterValue.test.tsx:17-26`, `packages/react/src/meter/value/MeterValue.test.tsx:47-63`

## State model (controlled/uncontrolled, defaults, transitions)

- `value` is fully controlled: every test renders `Meter.Root` with an explicit `value`
  (including conformance, which uses `<Meter.Root value={50} />`), and no uncontrolled/default
  usage is exercised. UNVERIFIED — inferred from
  `packages/react/src/meter/root/MeterRoot.test.tsx:13`, no test renders `Meter.Root` without `value`.
- Defaults: `min` 0 and `max` 100 (`aria-valuemin="0"`, `aria-valuemax="100"` without passing
  those props). `packages/react/src/meter/root/MeterRoot.test.tsx:31-33`
- Derived value model proven by the tests: the raw `value` is clamped into `[min, max]`
  (150→100, -10→0, NaN→0) to produce `aria-valuenow` and the indicator width, while the
  normalized ratio `(value-min)/(max-min)` (clamped to [0,1]; 0 when `min===max`) drives the
  default percent text and indicator width. `packages/react/src/meter/root/MeterRoot.test.tsx:188-222`, `packages/react/src/meter/indicator/MeterIndicator.test.tsx:16-52`
- Transitions are pure prop re-derivations: changing `value` via `setProps` updates
  `aria-valuenow`, `aria-valuetext`, the visible `Meter.Value` text, and the indicator width in
  the same render. `packages/react/src/meter/root/MeterRoot.test.tsx:70-97`
- Changing `min`, `max`, and `value` together re-derives range attributes, formatted text, and
  indicator width. `packages/react/src/meter/root/MeterRoot.test.tsx:154-186`
- No internal interactive state machine exists (meter is display-only); nothing beyond prop
  derivation is asserted. UNVERIFIED — inferred from the absence of state assertions across all
  five test files.

## Keyboard interactions

- N/A — no keyboard event is dispatched or asserted in any of the five test files; the meter is
  a non-interactive display component. The only interactive elements in the suite are plain
  external `<button>`s used to mutate label state.
  `packages/react/src/meter/label/MeterLabel.test.tsx:27-32`

## Focus management

- N/A — no focus-related assertion (tabindex, focus, blur) appears in any of the five test
  files. `packages/react/src/meter/root/MeterRoot.test.tsx:1-300`

## Accessibility (roles, aria-*, id linking)

- The root element has `role="meter"`. `packages/react/src/meter/root/MeterRoot.test.tsx:29`
- `aria-valuenow` is the clamped raw value: `30` for value 30, `100` for value 150, `0` for
  -10 or NaN, and the raw value itself (`5`) when `min===max===value`.
  `packages/react/src/meter/root/MeterRoot.test.tsx:31`, `packages/react/src/meter/root/MeterRoot.test.tsx:188-222`
- `aria-valuemin`/`aria-valuemax` default to `0`/`100` and track the props on rerender.
  `packages/react/src/meter/root/MeterRoot.test.tsx:32-33`, `packages/react/src/meter/root/MeterRoot.test.tsx:171-172`, `packages/react/src/meter/root/MeterRoot.test.tsx:180-181`
- Default `aria-valuetext` is the locale-formatted percent of the normalized ratio — e.g.
  `30` → percent of 0.3, `value={0.5} min={0} max={1}` → 50%, `min={20} max={40} value={30}` →
  50%, `min===max` or NaN → 0% — and it rounds fractional values like the displayed text.
  `packages/react/src/meter/root/MeterRoot.test.tsx:34`, `packages/react/src/meter/root/MeterRoot.test.tsx:52`, `packages/react/src/meter/root/MeterRoot.test.tsx:136`, `packages/react/src/meter/root/MeterRoot.test.tsx:150`, `packages/react/src/meter/root/MeterRoot.test.tsx:205`, `packages/react/src/meter/root/MeterRoot.test.tsx:211`, `packages/react/src/meter/root/MeterRoot.test.tsx:56-68`
- Default `aria-valuetext` always equals the visible `Meter.Value` text content (same formatter,
  same locale). `packages/react/src/meter/root/MeterRoot.test.tsx:53`, `packages/react/src/meter/root/MeterRoot.test.tsx:67`
- With the `format` prop, `aria-valuetext` becomes the Intl-formatted clamped value (e.g. USD
  currency of 30; of the clamped 100 when value is 150). `packages/react/src/meter/root/MeterRoot.test.tsx:247`, `packages/react/src/meter/root/MeterRoot.test.tsx:270-271`
- `getAriaValueText(formattedValue, value)` overrides `aria-valuetext` only: it receives the
  formatted (clamped) value and the raw value — with value 150 and USD format it is last called
  with `("$100.00"-style formatted clamped value, 150)` — and never affects the visible
  `Meter.Value` text. `packages/react/src/meter/root/MeterRoot.test.tsx:114-117`, `packages/react/src/meter/root/MeterRoot.test.tsx:272-273`
- `aria-labelledby` on the root points at the `Meter.Label` element's id (a user-supplied id such
  as `label-a` is honored). `packages/react/src/meter/root/MeterRoot.test.tsx:35-37`, `packages/react/src/meter/label/MeterLabel.test.tsx:40`

## DOM structure & portal behavior

- Element per part: `Meter.Root` renders a `<div>`
  (`packages/react/src/meter/root/MeterRoot.test.tsx:13-16`), `Meter.Track` a `<div>`
  (`packages/react/src/meter/track/MeterTrack.test.tsx:8-13`), `Meter.Indicator` a `<div>`
  (`packages/react/src/meter/indicator/MeterIndicator.test.tsx:9-14`), `Meter.Label` a `<span>`
  (`packages/react/src/meter/label/MeterLabel.test.tsx:10-15`), and `Meter.Value` a `<span>`
  (`packages/react/src/meter/value/MeterValue.test.tsx:9-14`) — all per the conformance
  `refInstanceof` assertions.
- Label association is by id, not nesting: `Meter.Label` can be a direct child of Root while
  Track/Indicator/Value form a separate subtree. `packages/react/src/meter/root/MeterRoot.test.tsx:21-26`
- The Indicator is sized by percentage of its container: in a 100px-wide Root, value 33 yields
  computed `left: 0px` and `width: 33px`, and value 0 yields `insetInlineStart: 0px` and
  `width: 0px` (logical inline-start property, RTL-friendly by construction). Chromium-only
  suite. `packages/react/src/meter/indicator/MeterIndicator.test.tsx:54-70`, `packages/react/src/meter/indicator/MeterIndicator.test.tsx:72-87`
- N/A — no portal behavior: no test references portals, and all parts render in place under
  Root. `packages/react/src/meter/root/MeterRoot.test.tsx:20-27`

## Events (names, payload shape, bubbling, preventDefault semantics)

- N/A — no event-handler props, custom events, bubbling, or `preventDefault` semantics are
  asserted in any of the five test files. The only dispatched events are `fireEvent.click` on
  plain external buttons that mutate label state, unrelated to Meter's own API.
  `packages/react/src/meter/label/MeterLabel.test.tsx:42-45`

## Edge cases (rapid interactions, unmount, nesting)

- Rapid prop updates: two consecutive `setProps({ value })` calls (50→77) each fully
  re-synchronize `aria-valuenow`, `aria-valuetext`, the Value text, and the indicator width.
  `packages/react/src/meter/root/MeterRoot.test.tsx:74-96`
- Simultaneous range change: updating `min`, `max`, and `value` in one `setProps` keeps
  attributes, formatted text, and indicator width consistent (50%→75%).
  `packages/react/src/meter/root/MeterRoot.test.tsx:158-185`
- Degenerate inputs are normalized, not crashed on: value above max clamps to 100%, below min
  to 0%, `min===max` yields a finite 0% indicator width (never `NaN%`/`Infinity%`), and NaN
  value behaves as 0. `packages/react/src/meter/root/MeterRoot.test.tsx:188-222`, `packages/react/src/meter/indicator/MeterIndicator.test.tsx:17-51`
- Label lifecycle: changing the label element's `id` re-links `aria-labelledby`, and unmounting
  the label removes `aria-labelledby` from the root entirely (no stale id).
  `packages/react/src/meter/label/MeterLabel.test.tsx:17-47`
- Render-function children of `Meter.Value` receive fresh `(formattedValue, rawValue)` arguments
  on every value change (30→60). `packages/react/src/meter/value/MeterValue.test.tsx:65-85`
- UNVERIFIED — unmounting the whole meter, rendering multiple/nested meters, and out-of-range
  arguments passed to `Meter.Value` render-function children are not covered by any test.

## Shared harness dependencies

- All five test files import `createRenderer`/`describeConformance` (and, for the Indicator,
  `isJSDOM`) from the `#test-utils` alias, which maps to `packages/react/test/index.ts`
  (`packages/react/package.json:110`).
  `packages/react/src/meter/root/MeterRoot.test.tsx:4`, `packages/react/src/meter/indicator/MeterIndicator.test.tsx:4`, `packages/react/src/meter/label/MeterLabel.test.tsx:5`, `packages/react/src/meter/track/MeterTrack.test.tsx:3`, `packages/react/src/meter/value/MeterValue.test.tsx:4`
- `describeConformance` (`packages/react/test/describeConformance.tsx:44-49`) runs the
  `propsSpread`, `refForwarding`, `renderProp`, and `className` conformance suites for every
  part; the meter tests pass no `skip`, so all four run against each part with its
  `refInstanceof` target. `packages/react/src/meter/root/MeterRoot.test.tsx:13-16`
- `createRenderer` wraps `@mui/internal-test-utils`' renderer; `render` is awaited and returns
  `{ setProps }`, used for the rerender-synchronization tests.
  `packages/react/src/meter/root/MeterRoot.test.tsx:74-81`
- `isJSDOM` (re-exported from `@base-ui/utils/testUtils` via `packages/react/test/index.ts:1`)
  gates the Indicator's Chromium-only computed-style suite.
  `packages/react/src/meter/indicator/MeterIndicator.test.tsx:54`
- `screen` and `fireEvent` come from `@mui/internal-test-utils`.
  `packages/react/src/meter/root/MeterRoot.test.tsx:3`, `packages/react/src/meter/label/MeterLabel.test.tsx:4`
