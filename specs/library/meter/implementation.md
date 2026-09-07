# Meter — implementation spec (Stage 2: implementation mining)

Ground truth for WHAT happens is `specs/library/meter/behavior.md`; this document explains the
state machine, hook composition, context usage, and DOM decisions that produce it. Source files:

- `packages/react/src/meter/root/MeterRoot.tsx` (plus `MeterRootContext.ts`)
- `packages/react/src/meter/label/MeterLabel.tsx`
- `packages/react/src/meter/track/MeterTrack.tsx`
- `packages/react/src/meter/indicator/MeterIndicator.tsx`
- `packages/react/src/meter/value/MeterValue.tsx`
- `packages/react/src/meter/index.ts` / `index.parts.ts` (barrels)

The unit's TODO entry (`TODO.md:426-432`) has no `wraps-external:` field — no external package is
delegated to; everything below is derived from first-party source.

## State machine / hooks used

There is **no state machine**. The meter is a display-only, fully-controlled component: the only
React state in the unit is the label id, and every value-related output (aria attributes, formatted
text, indicator width) is derived inline during render from the props. This is the mechanism behind
behavior.md's *State model* observation that "transitions are pure prop re-derivations" — there are
no effects, reducers, or stores involved in value handling, so a `setProps` call re-derives
everything in the same render pass with no intermediate states.

### Value derivation pipeline (render-phase, in `MeterRoot`)

`MeterRoot` destructures props with defaults `max = 100`, `min = 0`
(`packages/react/src/meter/root/MeterRoot.tsx:21-33`) and computes, in order:

1. `rawPercentage = valueToPercent(valueProp, min, max)` — the naive
   `((value - min) * 100) / (max - min)` (`packages/react/src/utils/valueToPercent.ts:1-3`). A
   `min === max` range (or `±Infinity` bounds) makes the denominator `0`/`∞` and yields `NaN`.
   The comment at `packages/react/src/meter/root/MeterRoot.tsx:37` records the reasoning: `clamp`
   handles infinity, but NaN must be intercepted before it propagates into CSS widths.
2. `percentageValue = clamp(NaN ? 0 : rawPercentage, 0, 100)`
   (`packages/react/src/meter/root/MeterRoot.tsx:39`) — the context-facing percentage behind the
   indicator width and the default percent text.
3. `clampedValue = clamp(NaN ? min : valueProp, min, max)`
   (`packages/react/src/meter/root/MeterRoot.tsx:40`) — the raw-value face shown as
   `aria-valuenow`. NaN falls back to `min` (which is `0` under the defaults, producing the
   "NaN→0" pin in behavior.md; with a custom `min` it would be that `min` — see gaps).
4. `formattedValue` (`packages/react/src/meter/root/MeterRoot.tsx:44-46`): with `format` the
   **clamped** value is formatted via `formatNumber(clampedValue, locale, format)`; without it, the
   **percentage** is formatted as `{ style: 'percent' }`. Formatting the clamped value is what
   keeps visible text, `aria-valuetext`, `aria-valuenow`, and the indicator consistent
   (behavior.md, *State model* and *Accessibility*).
5. `ariaValuetext` (`packages/react/src/meter/root/MeterRoot.tsx:48-51`): `getAriaValueText`
   receives `(formattedValue, valueProp)` — the formatted *clamped* value first, the **raw**
   (unclamped) prop second, matching behavior.md's *Accessibility* pin. It only overrides the
   aria string, never `formattedValue` itself, because the two are separate variables.

`formatNumber` is a thin wrapper over a module-level `Map` cache of `Intl.NumberFormat` instances
keyed by `JSON.stringify` of locale + options
(`packages/utils/src/formatNumber.ts:3-17`, `packages/utils/src/formatNumber.ts:19-28`), so the
per-render derivation does not re-allocate formatters for repeated (locale, options) pairs.

### Hooks (per call site)

| Hook | Call site | Role |
| --- | --- | --- |
| `React.useState` | `packages/react/src/meter/root/MeterRoot.tsx:35` | `labelId` — the only state in the unit |
| `React.useMemo` | `packages/react/src/meter/root/MeterRoot.tsx:70-78` | stable context value, deps `[formattedValue, percentageValue, setLabelId, valueProp]` |
| `useRenderElement` | `packages/react/src/meter/root/MeterRoot.tsx:80`, `packages/react/src/meter/label/MeterLabel.tsx:25`, `packages/react/src/meter/track/MeterTrack.tsx:19`, `packages/react/src/meter/indicator/MeterIndicator.tsx:22`, `packages/react/src/meter/value/MeterValue.tsx:22` | shared render pipeline for all five parts |
| `useMeterRootContext` | `packages/react/src/meter/root/MeterRootContext.ts:16-25`, consumed at `packages/react/src/meter/label/MeterLabel.tsx:21`, `packages/react/src/meter/indicator/MeterIndicator.tsx:20`, `packages/react/src/meter/value/MeterValue.tsx:20` | context read + missing-root guard |
| `useRegisteredLabelId` | `packages/react/src/meter/label/MeterLabel.tsx:23` | label id generation + registration into root state |

Deliberately absent: `useControlled` (the `value` prop is required and uncontrolled mode does not
exist — `value: number` with no default at
`packages/react/src/meter/root/MeterRoot.tsx:121`), `useStableCallback` (no event handlers),
`useTimeout`/`useAnimationFrame` (no timers), any focus hook (non-interactive, behavior.md
*Focus management*).

### Label id registration sub-mechanism

`useRegisteredLabelId` (`packages/react/src/utils/useRegisteredLabelId.ts:6-19`) is the entire
implementation of behavior.md's *Edge cases* label lifecycle:

- `useBaseUiId(idProp)` (`packages/react/src/utils/useRegisteredLabelId.ts:10`) resolves the id:
  a user-supplied id wins, otherwise one is generated with a `base-ui-` prefix
  (`packages/react/src/internals/useBaseUiId.ts:9-11`), which wraps `@base-ui/utils/useId`
  (`packages/utils/src/useId.ts:32-41`: `React.useId` when available, an incrementing global
  counter otherwise).
- A `useIsoLayoutEffect` keyed on `[id, setLabelId]`
  (`packages/react/src/utils/useRegisteredLabelId.ts:12-17`) pushes the id into root state via
  `setLabelId(id)` and its cleanup clears it with a functional update that only resets
  `currentId === id ? undefined : currentId`. The functional guard means a *new* label's
  registration is never clobbered by an *old* label's cleanup when ids swap.
- Layout-effect timing is load-bearing: registration (and cleanup on id change/unmount) happens
  before paint, so `aria-labelledby` on the root is never stale for a visible frame — this is what
  behavior.md's "changing id re-links / unmounting removes the attribute" pins fall out of.

### Render pipeline per part

Every part funnels through `useRenderElement` — `render` prop evaluation, `className`/`style`
resolution, and ref merging all come from that shared kernel
(`packages/react/src/internals/useRenderElement.tsx:22-48`); none of the five parts adds state
attributes (see gaps). Part-specific contributions are merged as `props: [internal, elementProps]`
arrays, and `mergeProps` resolves plain-prop conflicts rightmost-wins
(`packages/react/src/merge-props/mergeProps.ts:14-15`), so user props always override the
internal defaults.

## Context providers/consumers

`MeterRootContext` (`packages/react/src/meter/root/MeterRootContext.ts:14`) carries exactly four
members (`packages/react/src/meter/root/MeterRootContext.ts:4-12`): `formattedValue`,
`percentageValue`, `setLabelId`, and `value` (the raw, unclamped prop —
`packages/react/src/meter/root/MeterRoot.tsx:75`).

Consumption by part:

- `MeterLabel` reads only `setLabelId` (`packages/react/src/meter/label/MeterLabel.tsx:21`) — the
  write side of the id lift described above.
- `MeterIndicator` reads only `percentageValue`
  (`packages/react/src/meter/indicator/MeterIndicator.tsx:20`) and turns it into a CSS width.
- `MeterValue` reads `value` + `formattedValue`
  (`packages/react/src/meter/value/MeterValue.tsx:20`) and exposes them as the
  `(formattedValue, rawValue)` render-function arguments
  (`packages/react/src/meter/value/MeterValue.tsx:27`).
- `MeterTrack` reads **nothing** — it is a pure structural passthrough with no context access
  (`packages/react/src/meter/track/MeterTrack.tsx:13-23`); see gaps.

`MeterRoot` renders its element *inside* the provider it creates
(`packages/react/src/meter/root/MeterRoot.tsx:85`), with the user's children (plus the hidden NVDA
span) attached as the element's own children via `defaultProps.children`
(`packages/react/src/meter/root/MeterRoot.tsx:60-67`), so the whole part subtree is within the
boundary. The boundary is one-directional: parts read derived values; the only data flowing back
up is the label id through `setLabelId`, and every context consumer re-renders because the memoized
context value (`packages/react/src/meter/root/MeterRoot.tsx:70-78`) changes identity whenever
`formattedValue`/`percentageValue`/`valueProp` change.

`useMeterRootContext` throws the `Base UI:`-prefixed error documented in behavior.md's *Public API
surface* (`packages/react/src/meter/root/MeterRootContext.ts:19-21`).

## DOM/portal strategy and why

- **No portal.** No file in the unit imports any portal utility; all parts render in place
  (behavior.md, *DOM structure & portal behavior*). Element mapping: Root → `<div>`,
  Track → `<div>`, Indicator → `<div>`, Label → `<span>`, Value → `<span>`.
- **Root carries the entire ARIA surface.** `role="meter"` plus
  `aria-valuemin`/`aria-valuemax`/`aria-valuenow`/`aria-valuetext`/`aria-labelledby` are all set in
  `defaultProps` on the single root element
  (`packages/react/src/meter/root/MeterRoot.tsx:53-59`); the parts contribute no aria of their own.
  This is why behavior.md's *Accessibility* pins all resolve against the root element, and why the
  parts are freely reorderable/removable.
- **Indicator sizing is inline CSS, not measurement.** The fill is `width: ${percentageValue}%`
  anchored with `insetInlineStart: 0` and `height: 'inherit'`
  (`packages/react/src/meter/indicator/MeterIndicator.tsx:26-30`). Using the logical
  `insetInlineStart` property is why the Chromium test observes `left: 0px` in LTR while the
  declaration stays RTL-correct by construction (behavior.md, *DOM structure & portal behavior*).
  Percentage width needs no layout measurement, no resize observers, and no effect — it re-derives
  with the render, which is what makes the rapid-prop-update pins in behavior.md's *Edge cases*
  trivially synchronous. `height: 'inherit'` defers vertical sizing to the Track.
- **Label association by state-lifted id, not DOM query.** Rather than the root searching its
  subtree for a label element, the label registers its own id upward
  (`packages/react/src/utils/useRegisteredLabelId.ts:12-17`) and the root renders
  `aria-labelledby` from state (`packages/react/src/meter/root/MeterRoot.tsx:54`). This works with
  any tree shape — including the behavior.md case where Label is a sibling of the
  Track/Indicator/Value subtree — and survives custom `render` props on the label without ref
  tricks.
- **Hidden NVDA workaround span.** `MeterRoot` appends, after the user's children, a
  `role="presentation"` visually-hidden `<span>` containing the text `x`
  (`packages/react/src/meter/root/MeterRoot.tsx:63-65`), citing mui/base-ui#4184: NVDA reads the
  label only when a presentational text node follows it inside the meter. The styles come from
  `@base-ui/utils/visuallyHidden` (`packages/utils/src/visuallyHidden.ts:14-19`: clip-path/1px/
  `position: fixed` hiding that stays accessible to assistive tech). No test observes this node.
- **Value text is aria-hidden.** `MeterValue` marks its visible text `aria-hidden: true`
  (`packages/react/src/meter/value/MeterValue.tsx:26`) because the root's `aria-valuetext` already
  exposes the value to assistive tech — the visible span would otherwise double-announce.

## Dependencies on other Base UI internals

Direct imports of the unit, by consumer:

- **Render kernel:** `internals/useRenderElement` — `packages/react/src/meter/root/MeterRoot.tsx:9`,
  `packages/react/src/meter/label/MeterLabel.tsx:6`, `packages/react/src/meter/track/MeterTrack.tsx:5`,
  `packages/react/src/meter/indicator/MeterIndicator.tsx:6`,
  `packages/react/src/meter/value/MeterValue.tsx:6`. Used for `render` prop evaluation,
  `className`/`style` resolution, and ref merging on every part; the unit uses no `state` objects
  and no `stateAttributesMapping` (all `Meter*State` interfaces are empty —
  `packages/react/src/meter/root/MeterRoot.tsx:88`).
- **Types:** `internals/types` — `BaseUIComponentProps`/`HTMLProps`
  (`packages/react/src/meter/root/MeterRoot.tsx:7` and one import per part).
- **React utils:** `utils/valueToPercent`
  (`packages/react/src/meter/root/MeterRoot.tsx:8`,
  `packages/react/src/utils/valueToPercent.ts:1-3`) and `utils/useRegisteredLabelId`
  (`packages/react/src/meter/label/MeterLabel.tsx:7`).
- **`@base-ui/utils` package:** `clamp`, `formatNumber`, `visuallyHidden`
  (`packages/react/src/meter/root/MeterRoot.tsx:3-5`).
- **Transitive (via `useRegisteredLabelId`, relevant for a port):** `internals/useBaseUiId`
  (`packages/react/src/utils/useRegisteredLabelId.ts:4`) and the `@base-ui/utils` hooks
  `useIsoLayoutEffect` (`packages/react/src/utils/useRegisteredLabelId.ts:3`) and `useId`
  (`packages/react/src/internals/useBaseUiId.ts:2`).
- **Explicitly not used:** `floating-ui-react` (nothing to position), `use-render` (`render` is
  handled entirely by `useRenderElement`), `useControlled`, portal/overflow/focus machinery,
  `DirectionProvider` context, and the `preventBaseUIHandler` event system (no event handlers
  exist anywhere in the unit).

For `ralph/scripts/generate-todo.mjs` dependency computation, meter's precise internal surface is:
`internals/useRenderElement`, `internals/types`, `internals/useBaseUiId` (transitive),
`utils/valueToPercent`, `utils/useRegisteredLabelId`, and the `@base-ui/utils` members `clamp`,
`formatNumber`, `visuallyHidden`, `useIsoLayoutEffect`, `useId` — and nothing else. No focus,
portal, positioning, or state-attribute machinery. The public subpath export is
`./meter` → `src/meter/index.ts` (`packages/react/package.json:51`), whose barrels alias the
parts to `Root`/`Track`/`Indicator`/`Value`/`Label`
(`packages/react/src/meter/index.parts.ts:1-5`) and re-export all part types
(`packages/react/src/meter/index.ts:1-7`).

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit:

1. **Hidden NVDA span.** The appended `role="presentation"` visually-hidden `<span>` with text `x`
   (`packages/react/src/meter/root/MeterRoot.tsx:63-65`) is asserted by no test — behavior.md never
   mentions it. A port that omits it changes screen-reader behavior (mui/base-ui#4184) while
   passing every fixture.
2. **`role="presentation"` on `Meter.Label`.** The label renders a span whose role is explicitly
   presentation (`packages/react/src/meter/label/MeterLabel.tsx:30`). behavior.md's
   *Accessibility* section documents the `aria-labelledby` link but never the label's own role;
   untested.
3. **`aria-hidden` on `Meter.Value`.** The visible value span is hidden from assistive tech
   (`packages/react/src/meter/value/MeterValue.tsx:26`); no test asserts it. Dropping it would
   double-announce without failing any current test.
4. **`Meter.Track` needs no Root.** Track is the only part that never calls
   `useMeterRootContext` (`packages/react/src/meter/track/MeterTrack.tsx:13-23`), so rendering
   `Meter.Track` outside `Meter.Root` does **not** throw, unlike Label/Indicator/Value. No test
   covers this asymmetry.
5. **behavior.md over-claims about `Meter.Value` children.** The props type allows only
   `null | ((formattedValue, value) => ReactNode)`
   (`packages/react/src/meter/value/MeterValue.tsx:36-41`), and the runtime replaces *any*
   non-function children with the formatted string
   (`packages/react/src/meter/value/MeterValue.tsx:27`). behavior.md's *Public API surface* lists
   "a node" as a supported children shape citing
   `packages/react/src/meter/value/MeterValue.test.tsx:17-26`, but that test range
   only covers omitted children; a plain-node child is untyped and silently discarded at runtime.
   The audit loop should reconcile this.
6. **NaN fallback targets `min`, not `0`.** A NaN `value` yields `clampedValue = min`
   (`packages/react/src/meter/root/MeterRoot.tsx:40`) — with custom `min` the `aria-valuenow` would
   be that min, while `percentageValue` still collapses to `0`
   (`packages/react/src/meter/root/MeterRoot.tsx:39`). Only the default `min = 0` case is pinned
   (behavior.md, *Accessibility*); the general rule is untested, as are infinite `min`/`max` bounds
   (the explicit NaN guard at `:37-39` exists for them).
7. **User override of internal ARIA/role attributes.** Because user `elementProps` sit rightmost in
   the merge (`packages/react/src/meter/root/MeterRoot.tsx:80-83`,
   `packages/react/src/merge-props/mergeProps.ts:14-15`), a user-supplied `role`, `aria-valuetext`,
   or `aria-labelledby` prop replaces the internal value (`aria-labelledby` also being the prop
   documented on `MeterRootProps`,
   `packages/react/src/meter/root/MeterRoot.tsx:94`). The conformance suite only checks arbitrary
   props spread; precedence over the *internal* aria values is untested.
8. **`Indicator`'s `height: 'inherit'`.** Only `left`/`width` computed styles are asserted
   (behavior.md, *DOM structure & portal behavior*); the `height: 'inherit'` +
   `insetInlineStart: 0` declarations
   (`packages/react/src/meter/indicator/MeterIndicator.tsx:27-28`) are unobserved implementation
   details.
9. **SSR/hydration shape of `aria-labelledby`.** The label id is registered in a layout effect, so
   server-rendered markup contains no `aria-labelledby` until hydration runs the registration. No
   test covers SSR output.
10. **Empty state objects.** All `Meter*State` interfaces are empty
    (`packages/react/src/meter/root/MeterRoot.tsx:88` and each part's `*State`), so render
    callbacks receive an empty `state` and no `data-*` state attributes are ever emitted — a
    structural difference from most Base UI components that no test pins.
11. **Barrel/export shape.** `packages/react/src/meter/index.ts:1-7` (namespace + part types),
    `packages/react/src/meter/index.parts.ts:1-5` (Root/Track/Indicator/Value/Label aliases), and
    the `./meter` subpath (`packages/react/package.json:51`) are constrained only by the tests'
    imports resolving.
