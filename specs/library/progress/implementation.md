# Progress — implementation spec

Mined from the non-test source under `packages/react/src/progress/`. Companion to
`specs/library/progress/behavior.md` (the WHAT); this document is the WHY/HOW. Behavior already
documented there is cited by section name, not restated.

## State machine / hooks used

Progress has no reducer, no `useControlled`, and no controlled/uncontrolled machinery. The only
`React.useState` in the unit is `labelId` (`packages/react/src/progress/root/ProgressRoot.tsx:36`).
Everything else — status, percentage, formatting — is derived inline during render, which is why
prop changes (behavior.md, "State model" section: "immediately updates") require no effects at all.

Status derivation is a single render-time branch keyed on the indeterminacy sentinel
`value != null && Number.isFinite(value)` (`packages/react/src/progress/root/ProgressRoot.tsx:49`)
— this one condition is what makes `null`, `NaN`, and `±Infinity` behave identically (behavior.md,
"Edge cases" section). Inside the branch, one pass computes the whole derived tuple
(`packages/react/src/progress/root/ProgressRoot.tsx:50-53`):

1. `valueToPercent(value, min, max)` normalizes to a percentage (`packages/react/src/utils/valueToPercent.ts:1-3` — literally `((value - min) * 100) / (max - min)`).
2. The NaN guard `Number.isNaN(rawPercentage) ? 0 : rawPercentage` followed by `clamp(…, 0, 100)`
   (`packages/react/src/progress/root/ProgressRoot.tsx:51`) is the mechanism behind the degenerate
   `min === max` outcome in behavior.md's "State model" section: when `min === max` the divisor in
   `valueToPercent` is `0`, so the raw percentage is `NaN` (or `±Infinity` for out-of-range values),
   coerced to `0` before clamping. The clamped *raw* value (`packages/react/src/progress/root/ProgressRoot.tsx:52`) is computed separately, which is why `aria-valuenow` still shows `'5'` while text/width show `0%`.
3. `status = clampedValue === max ? 'complete' : 'progressing'`
   (`packages/react/src/progress/root/ProgressRoot.tsx:53`) — note `complete` is keyed on the
   *clamped* value equaling `max`, which is why out-of-range overshoot (45 with `max={40}`) is
   also `data-complete` (behavior.md, "State model" section). The indeterminate case never enters
   the branch, so `status` keeps its initializer `'indeterminate'`
   (`packages/react/src/progress/root/ProgressRoot.tsx:42`).

`status` is the sole member of `ProgressRootState`, wrapped in a `React.useMemo` keyed on `status`
(`packages/react/src/progress/root/ProgressRoot.tsx:62`) so the state object identity (and hence
the context value and data-attribute recomputation) is stable across renders that don't change
status.

Formatting is also render-time: `format ? formatNumber(clampedValue, locale, format) :
formatNumber(percentageValue / 100, locale, { style: 'percent' })`
(`packages/react/src/progress/root/ProgressRoot.tsx:56-58`). The default path formats the
*normalized ratio*, not the raw value — the mechanism behind the default-percent behavior in
behavior.md's "State model" section. `formatNumber` returns `''` for `null` and caches
`Intl.NumberFormat` instances per `(locale, options)`
(`packages/utils/src/formatNumber.ts:19-28`, cache at `packages/utils/src/formatNumber.ts:3-17`).
Because all of this happens during render, the `format`/`locale` reactivity noted in behavior.md's
"State model" section is ordinary re-rendering — no effect or memo invalidation is involved.
`getAriaValueText(formattedValue, value)` is likewise invoked inline
(`packages/react/src/progress/root/ProgressRoot.tsx:69-71`), explaining the "(clamped formatted,
raw second arg)" contract in behavior.md's "Accessibility" section.

Hooks by call site:

- `ProgressRoot`: `React.forwardRef` (`packages/react/src/progress/root/ProgressRoot.tsx:18`), `React.useState` for `labelId` (`.tsx:36`), `React.useMemo` for `state` (`.tsx:62`) and for the context value (`.tsx:83-92`), `useRenderElement` (`.tsx:94-99`).
- `ProgressLabel`: `useRegisteredLabelId(idProp, setLabelId)` (`packages/react/src/progress/label/ProgressLabel.tsx:24`). That hook resolves the id via `useBaseUiId` (`packages/react/src/utils/useRegisteredLabelId.ts:10`, which prefixes generated ids with `base-ui-` per `packages/react/src/internals/useBaseUiId.ts:9-10`), then registers/unregisters with `useIsoLayoutEffect` (`packages/react/src/utils/useRegisteredLabelId.ts:12-17`). The cleanup is conditional — `setLabelId((currentId) => (currentId === id ? undefined : currentId))` — so a label only clears the root's `aria-labelledby` if it still owns the slot. This ownership check is the mechanism behind the live re-association and unmount-clearing documented in behavior.md's "Accessibility" section (including the id-change case: new id → effect re-runs → setter updates; old label's cleanup runs after and no-ops).
- Every part: `React.forwardRef` + `useRenderElement` as the only composition primitive.

The `data-*` status engine: all five parts pass the same `progressStateAttributesMapping`
(`packages/react/src/progress/root/stateAttributesMapping.ts:5-17`) to `useRenderElement`. It maps
the single `status` state key onto exactly one bare attribute (`''` value) of
`data-progressing`/`data-complete`/`data-indeterminate` (constants in
`packages/react/src/progress/root/ProgressRootDataAttributes.ts:4-12`), returning `null` otherwise.
`getStateAttributesProps` (`packages/react/src/internals/getStateAttributesProps.ts:15-21`) routes
the `status` key through this custom mapping instead of the default `data-${key}` lowercasing
(`packages/react/src/internals/getStateAttributesProps.ts:24-28`) — without the mapping, `status`
would render as `data-status="progressing"` rather than the three distinct attributes asserted in
behavior.md's "State model" section. The one-of-three exclusivity is guaranteed structurally:
`status` is a single string, and the mapping emits one attribute per value.

## Context providers/consumers

One provider: `ProgressRootContext.Provider` wraps the rendered root element
(`packages/react/src/progress/root/ProgressRoot.tsx:101-103`). The context value is memoized on
`[formattedValue, percentageValue, setLabelId, state, value]`
(`packages/react/src/progress/root/ProgressRoot.tsx:83-92`); since `state`'s identity only changes
when `status` flips and `setLabelId` is a stable setter, downstream re-renders are driven purely by
value/formatting/status changes.

Context shape (`packages/react/src/progress/root/ProgressRootContext.tsx:5-21`):
`formattedValue: string`, `percentageValue: number | null` (clamped 0–100), `value: number | null`
(raw prop), `setLabelId`, `state`. Note the deliberate split: the *clamped/normalized* values
(`percentageValue`, `formattedValue`) and the *raw* `value` are both exposed, because
`ProgressValue`'s render function needs the raw sentinel as its second argument (behavior.md,
"Edge cases" section) while `ProgressIndicator` needs the percentage.

Consumers, all through `useProgressRootContext`
(`packages/react/src/progress/root/ProgressRootContext.tsx:28-37`), which throws the
`ProgressRootContext is missing` error recorded in behavior.md's "Edge cases" section:

- `ProgressTrack` — `state` only (`packages/react/src/progress/track/ProgressTrack.tsx:21`). The track contributes no computed DOM of its own; it exists to host the status attributes and be a styling/styling-anchor surface for the indicator.
- `ProgressIndicator` — `percentageValue`, `state` (`packages/react/src/progress/indicator/ProgressIndicator.tsx:21`). The width driver.
- `ProgressValue` — `value`, `formattedValue`, `state` (`packages/react/src/progress/value/ProgressValue.tsx:20`).
- `ProgressLabel` — `setLabelId`, `state` (`packages/react/src/progress/label/ProgressLabel.tsx:22`). The only child→parent data flow in the unit: it *writes* the label id back into root state via the context setter.

## DOM/portal strategy and why

No portals anywhere — every part renders in place, consistent with behavior.md's "DOM structure &
portal behavior" section. This is structural, not incidental: Progress is a purely presentational
leaf with no popups, positioning, focus, or events (behavior.md's N/A sections), so the
floating-ui/portal machinery other Base UI components use has nothing to do here. Default tags are
declared as the first argument to `useRenderElement`: `div` for Root
(`packages/react/src/progress/root/ProgressRoot.tsx:94`), Track
(`packages/react/src/progress/track/ProgressTrack.tsx:23`), and Indicator
(`packages/react/src/progress/indicator/ProgressIndicator.tsx:32`); `span` for Label
(`packages/react/src/progress/label/ProgressLabel.tsx:26`) and Value
(`packages/react/src/progress/value/ProgressValue.tsx:28`).

Indicator sizing is inline style, not CSS vars or classes
(`packages/react/src/progress/indicator/ProgressIndicator.tsx:23-30`): determinate →
`{ insetInlineStart: 0, height: 'inherit', width: `${percentageValue}%` }`; indeterminate → `{}`.
The choices: `insetInlineStart` (logical property) anchors the fill to the inline start so it
mirrors under RTL without extra code; `height: 'inherit'` lets the fill adopt whatever height the
consumer gives the track; and the empty style object in the indeterminate case is what produces
`indicator.style.width === ''` (behavior.md, "DOM structure & portal behavior" section) — the
component deliberately leaves the indeterminate fill's animation to consumer CSS.

Label association is state-based, not DOM-query-based: the root renders
`'aria-labelledby': labelId` from its `useState` (`packages/react/src/progress/root/ProgressRoot.tsx:65`), and the label pushes its generated-or-explicit id up through `setLabelId` via
`useRegisteredLabelId` (see hooks section). This explains the dynamic behaviors in behavior.md's
"Accessibility" section (id change re-links, unmount clears) without any
`getElementById`-style coupling.

`ProgressValue` computes its children at render: indeterminate swaps the render-function's first
argument to the literal `'indeterminate'` and the text content to `null`
(`packages/react/src/progress/value/ProgressValue.tsx:24-26`) — the sentinel documented in
behavior.md's "Edge cases" section — otherwise it renders `formattedValue` or calls the child
function with `(formattedValueArg, value)` (`packages/react/src/progress/value/ProgressValue.tsx:33-37`).

The root additionally appends a visually hidden `<span role="presentation">` containing the text
`x` after the user's children (`packages/react/src/progress/root/ProgressRoot.tsx:73-80`), styled
with the `visuallyHidden` style object (`packages/utils/src/visuallyHidden.ts:14-19`); the inline
comment ties it to forcing NVDA to read the label (mui/base-ui#4184). See the untested-behavior
section below.

## Dependencies on other Base UI internals

The `library: progress` entry in `TODO.md` (`TODO.md:473-479`) has **no `wraps-external:` field**,
so there is no external-package delegation to document — everything below is in-repo and must be
reimplemented/ported by the Rust crate work.

In-package (`packages/react/src`):

- `internals/useRenderElement` — imported by all five parts; it *is* the render pipeline:
  - props-array merging via `mergePropsN` (`packages/react/src/internals/useRenderElement.tsx:121-129`) — every part passes `[computedDefaults, elementProps]`, with user props winning per-key;
  - state→`data-*` attributes via `getStateAttributesProps` (`packages/react/src/internals/useRenderElement.tsx:76-78`);
  - `className`/`style` resolution (including function-of-state forms) (`packages/react/src/internals/useRenderElement.tsx:73-74`);
  - render-prop evaluation — function-form is *called* `(props, state)`, element-form is `cloneElement`d with merged props, with dev-only validation of invalid elements and a dev warning for `render={Component}` misuse (`packages/react/src/internals/useRenderElement.tsx:158-206`, `208-230`);
  - ref merging — forwarded ref + render-element ref + user ref via `useMergedRefs`/`useMergedRefsN` (`packages/react/src/internals/useRenderElement.tsx:94-104`);
  - lazy render-prop unwrapping for RSC/Flight (`packages/react/src/internals/useRenderElement.tsx:145-156`).
- `internals/getStateAttributesProps` — the mapping engine described above (`packages/react/src/internals/getStateAttributesProps.ts:5-31`).
- `internals/useBaseUiId` → `@base-ui/utils/useId` — `base-ui-`-prefixed id generation for the label (`packages/react/src/internals/useBaseUiId.ts:9-10`).
- `utils/useRegisteredLabelId` — label id registration lifecycle (`packages/react/src/utils/useRegisteredLabelId.ts:6-20`).
- `utils/valueToPercent` — normalization formula (`packages/react/src/utils/valueToPercent.ts:1-3`).
- `internals/types` — `BaseUIComponentProps`/`HTMLProps` that every part's props interface extends (e.g. `packages/react/src/progress/root/ProgressRoot.tsx:115`).

Cross-package (`@base-ui/utils`):

- `clamp` (`packages/react/src/progress/root/ProgressRoot.tsx:5`) — both the percentage and raw-value clamps.
- `formatNumber` (`packages/react/src/progress/root/ProgressRoot.tsx:4`) — with its `Intl.NumberFormat` cache (`packages/utils/src/formatNumber.ts:3-17`).
- `visuallyHidden` (`packages/react/src/progress/root/ProgressRoot.tsx:3`).
- `useIsoLayoutEffect` (via `useRegisteredLabelId`) and `useMergedRefs`/`useMergedRefsN` (via `useRenderElement`).

Explicitly **not** used by this unit: `floating-ui-react`, `use-render`, `useControlled`,
`useStableCallback`, portals, or the owner/`contains`/`activeElement` realm utilities — Progress
has no events, focus, or positioning. For dependency-graph purposes this is the lightest tier of
Base UI component: `useRenderElement` + `clamp`/`formatNumber`/`visuallyHidden` + one
id-registration util.

## Anything in source not explained by any test

Flagged explicitly for the fixture stage and backward-looking audit; none of these are asserted in
the test suite mined for behavior.md:

1. **NVDA workaround span.** The root always renders a visually hidden `<span role="presentation">`
   containing the text `x` after user children
   (`packages/react/src/progress/root/ProgressRoot.tsx:76-78`), styled by `visuallyHidden`
   (`packages/utils/src/visuallyHidden.ts:14-19`). No test asserts its existence, content, or
   styles, and behavior.md does not mention it. A port must decide whether to replicate this
   NVDA-specific hack (the inline comment cites mui/base-ui#4184).
2. **`role="presentation"` on the label.** `ProgressLabel` hardcodes `role: 'presentation'` into
   its default props (`packages/react/src/progress/label/ProgressLabel.tsx:32`), stripping the
   span's semantics so only the root's progressbar role is exposed. Untested and absent from
   behavior.md's "Accessibility" section.
3. **`aria-hidden` on `ProgressValue`.** The value part always renders `'aria-hidden': true`
   (`packages/react/src/progress/value/ProgressValue.tsx:33`). This is untested and in tension
   with behavior.md's "Accessibility" section, which describes the part's text as what "screen
   readers announcing the part's text" would get — per source, the element is hidden from AT and
   the root's `aria-valuetext`/`aria-labelledby` do the announcing. Fixtures should treat
   `aria-hidden` as ground truth.
4. **`height: 'inherit'` on the indicator.** The computed-style tests assert only
   `insetInlineStart` and `width`; the third inline style key
   (`packages/react/src/progress/indicator/ProgressIndicator.tsx:28`) is untested.
5. **Per-part `*DataAttributes.ts` files are dead code.** Only
   `ProgressRootDataAttributes.ts` is imported anywhere (by
   `packages/react/src/progress/root/stateAttributesMapping.ts:3`);
   `ProgressIndicatorDataAttributes.ts`, `ProgressLabelDataAttributes.ts`,
   `ProgressTrackDataAttributes.ts`, and `ProgressValueDataAttributes.ts` are not imported by any
   file. All five parts share the root's `progressStateAttributesMapping`. A port can model this
   as a single shared status→attribute mapping; the four orphan files are documentation-only.
6. **Non-function node children on `Progress.Value` are silently ignored at runtime.** The props
   type restricts `children` to `null | render function`
   (`packages/react/src/progress/value/ProgressValue.tsx:49-55`), and the implementation
   destructures `children` out and only consumes it in the function branch — a plain node child
   would be replaced by the formatted text (`packages/react/src/progress/value/ProgressValue.tsx:33-37`). No test covers this, and the dev-mode warning machinery in
   `useRenderElement` does not catch it.
7. **`formatNumber`'s formatter cache** (`packages/utils/src/formatNumber.ts:3-17`) is a pure
   performance detail invisible to behavior tests; a port needs `Intl`-equivalent output, not the
   cache.
