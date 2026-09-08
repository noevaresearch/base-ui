# Toggle — implementation spec (Stage 2: implementation mining)

Companion to `behavior.md` (same directory), which documents WHAT happens; this file explains WHY/HOW
from the unit's non-test source files:

- `packages/react/src/toggle/index.ts`
- `packages/react/src/toggle/Toggle.tsx`
- `packages/react/src/toggle/ToggleDataAttributes.ts`

The unit's `TODO.md` entry (`library: toggle`, `TODO.md:545-551`) has no `wraps-external:` field —
behavior is fully internal to the repo; there is no third-party package whose internals this spec
delegates to.

## State machine / hooks used

### `useControlled` — the entire pressed-state duality (`packages/react/src/toggle/Toggle.tsx:63-68`)

The controlled/uncontrolled/grouped tri-state of behavior.md's "State model" collapses into one
polymorphic hook argument: the `controlled` input is `groupContext ? value !== undefined &&
groupValue.indexOf(value) > -1 : pressedProp`.

- Standalone uncontrolled: `pressedProp` is `undefined`, so `useControlled` seeds internal state
  from `defaultPressed` (defaulted to `false` at `packages/react/src/toggle/Toggle.tsx:30`) and the
  returned setter actually writes (`packages/utils/src/useControlled.ts:42,82-89`).
- Standalone controlled: `pressedProp` defined; the setter is a no-op
  (`packages/utils/src/useControlled.ts:84-86`) and external prop flips are the only state source
  (behavior.md "State model" controlled bullet).
- Grouped: the expression `value !== undefined && groupValue.indexOf(value) > -1` is *always a
  boolean*, never `undefined`, so a grouped Toggle is always controlled — and controlled **by the
  group**, not by `pressed` (which is shadowed entirely). The membership probe runs per render, so a
  group-value change re-derives `pressed` without any local write. Because `useControlled` captures
  `isControlled` once on first render (`packages/utils/src/useControlled.ts:41`) and the grouped
  expression can never become `undefined`, the mode is stable for the component's lifetime.

This is why the click handler can end with an unconditional `setPressedState(nextPressed)`
(`packages/react/src/toggle/Toggle.tsx:104`): one write site, three commit semantics — real local
commit (uncontrolled), no-op (controlled-by-prop), no-op pending group re-derivation
(controlled-by-group). `useControlled`'s dev-only mode-switch/default-change warnings
(`packages/utils/src/useControlled.ts:47-80`) are the only guard around this.

### Click handler — the transition machine and cancel protocol (`packages/react/src/toggle/Toggle.tsx:84-105`)

The `onClick` in the base props object is the unit's only state transition. Ordered gates:

1. `nextPressed = !pressed` from the render-time snapshot
   (`packages/react/src/toggle/Toggle.tsx:85`).
2. One shared `details` object: `createChangeEventDetails(REASONS.none, event.nativeEvent)`
   (`packages/react/src/toggle/Toggle.tsx:86`; factory at
   `packages/react/src/internals/createBaseUIEventDetails.ts:118-149` — a closure-scoped `canceled`
   flag read through the `isCanceled` getter). The in-source comment
   (`packages/react/src/toggle/Toggle.tsx:88-89`) states the design: the same `details` object is
   handed to the user callback *and* the group commit so a single `cancel()` vetoes both consumers —
   the mechanism behind behavior.md's "State model" cancellation and grouped-cancellation bullets.
3. Gate 1 (`packages/react/src/toggle/Toggle.tsx:92-94`): `onPressedChange` runs **before any
   commit** (`packages/react/src/toggle/Toggle.tsx:90`), so a user cancel suppresses the group value
   change and the local state change in one check.
4. Gate 2 (`packages/react/src/toggle/Toggle.tsx:96-98`): `groupContext?.setGroupValue?.(value,
   nextPressed, details)` — only when `value` is truthy (see `useBaseUiId` below). ToggleGroup's own
   committer re-checks `details.isCanceled` on its side; the shared object is the contract.
5. Gate 3 (`packages/react/src/toggle/Toggle.tsx:100-102`): a *second* `isCanceled` check after the
   group commit — the reverse veto. If ToggleGroup's `onValueChange` calls `cancel()` on the same
   details object, the Toggle's own pressed state also does not change.
6. `setPressedState(nextPressed)` last (`packages/react/src/toggle/Toggle.tsx:104`), with the
   semantics per mode described above.

The reason is always `REASONS.none` (`packages/react/src/toggle/Toggle.tsx:86`;
`packages/react/src/internals/reasons.ts:3`) — Toggle has exactly one transition trigger, matching
behavior.md's "Events" (single callback, no reason variation).

Disabled clicks never reach this handler: the native `disabled` attribute blocks the DOM click, and
`useButton`'s onClick guard additionally preventDefaults when disabled
(`packages/react/src/internals/use-button/useButton.ts:104-110`) — behavior.md's "Edge cases"
disabled-interaction bullet.

### `useBaseUiId(valueProp || undefined)` — id as group key, not DOM id (`packages/react/src/toggle/Toggle.tsx:43-44`)

Wrapper at `packages/react/src/internals/useBaseUiId.ts:9-11` over
`packages/utils/src/useId.ts:32-37`. Two non-obvious decisions:

- The resolved `value` is **never rendered as an `id` attribute** (it appears in no props entry of
  `packages/react/src/toggle/Toggle.tsx:81-109`); it is purely the logical membership key probed
  against `groupValue` (`packages/react/src/toggle/Toggle.tsx:64`) and committed to the group
  (`packages/react/src/toggle/Toggle.tsx:97`).
- `|| undefined` (comment at `packages/react/src/toggle/Toggle.tsx:43`) normalizes falsy `value`
  props (`""`) to a generated `base-ui-*` id. With React ≥17 the generated id exists from first
  render, so `value` is always truthy and the `if (value)` gate
  (`packages/react/src/toggle/Toggle.tsx:96`) only matters on the React ≤17 fallback path, where the
  generated id materializes only after mount (`packages/utils/src/useId.ts:8-22`).
- Dev-only effect (`packages/react/src/toggle/Toggle.tsx:50-61`): warns when a grouped Toggle has no
  explicit `value` (gated on `groupContext.isValueInitialized`,
  `packages/react/src/toggle-group/ToggleGroupContext.ts:14-18`), because a generated id can never
  match the group's value set.

### `useButton({ disabled, native: nativeButton })` — button semantics factory (`packages/react/src/toggle/Toggle.tsx:70-73`)

Delegates to `packages/react/src/internals/use-button/useButton.ts:14-243`:

- `getButtonProps` (`packages/react/src/internals/use-button/useButton.ts:91-232`): disabled-gated
  `onClick`/`onMouseDown`/`onPointerDown`; keyboard-activation synthesis (Enter on keydown, Space on
  keyup) for non-native renders and Space-on-keydown for composite items
  (`packages/react/src/internals/use-button/useButton.ts:138-152,155-175,207-216`); and the native/non-native attribute split
  `{ type: 'button' }` vs `{ role: 'button' }` (`packages/react/src/internals/use-button/useButton.ts:226`).
- `useFocusableWhenDisabled` (`packages/react/src/utils/useFocusableWhenDisabled.ts:4-61`): for a
  standalone Toggle (not composite) it emits `tabIndex: 0` plus the native `disabled` attribute
  (`packages/react/src/utils/useFocusableWhenDisabled.ts:30-36,45-47`); for a grouped Toggle (composite) it emits
  `aria-disabled` *and* keeps the native `disabled` (`packages/react/src/utils/useFocusableWhenDisabled.ts:38-47`). This is
  the source of the disabled attributes behavior.md's "Accessibility" records.
- `updateDisabled` + `buttonRef` (`packages/react/src/internals/use-button/useButton.ts:72-89,234-237`): strips native `disabled` only for
  composite items that are meant to stay focusable when disabled. Toggle never passes
  `focusableWhenDisabled`, so the native-disabled branch always applies and this path never fires
  for a Toggle — matching the itemMetadata comment at
  `packages/react/src/toggle/Toggle.tsx:118-119`.
- Composite detection: `useButton` internally calls `useCompositeRootContext(true)`
  (`packages/react/src/internals/use-button/useButton.ts:25-26`). A standalone Toggle is not a composite item (the real `<button>` handles
  Space/Enter natively); a grouped Toggle is (ToggleGroup provides a `CompositeRoot`,
  `packages/react/src/toggle-group/ToggleGroup.tsx:8,111`), so its Space activation is dispatched
  synthetically on keydown and the native keyup Space is suppressed to avoid double activation
  (`packages/react/src/internals/use-button/useButton.ts:138-152,186-196`).
- Dev-only nativeButton mismatch warning (`packages/react/src/internals/use-button/useButton.ts:36-66`) backs the `nativeButton` prop
  contract.

### State → data attributes and the two render paths

- `state: ToggleState = { disabled, pressed }` (`packages/react/src/toggle/Toggle.tsx:75-78`) feeds
  both render paths. `getStateAttributesProps`
  (`packages/react/src/internals/getStateAttributesProps.ts:24-28`) maps a `true` state value to
  `data-pressed`/`data-disabled` (empty string) and omits the attribute for `false` — the generic
  mechanism whose two resulting names `packages/react/src/toggle/ToggleDataAttributes.ts:4,8`
  document (the constants are declarative; the attributes are produced by the state mapping, not by
  importing them).
- `useRenderElement('button', componentProps, { enabled: !groupContext, state, ref: refs, props })`
  (`packages/react/src/toggle/Toggle.tsx:111-116`;
  `packages/react/src/internals/useRenderElement.tsx:22-48`). `enabled: !groupContext` is the
  path switch: when grouped, the standalone render is disabled (returns `null`) and the
  `CompositeItem` JSX below renders instead. Refs `[buttonRef, forwardedRef]`
  (`packages/react/src/toggle/Toggle.tsx:80`) merge via `useMergedRefsN`
  (`packages/react/src/internals/useRenderElement.tsx:99-101`).
- Props merge order (`packages/react/src/toggle/Toggle.tsx:81-109`: base props, `elementProps`,
  `getButtonProps`) interacts with `mergePropsN`'s rightmost-handler-first rule
  (`packages/react/src/merge-props/mergeProps.ts:17,221-250`): on click, `getButtonProps`' disabled
  guard runs first, then the user's `onClick`, then Toggle's state machine — `getButtonProps` is a
  props-getter that receives the merged-so-far props and chains its external `onClick`
  (`packages/react/src/merge-props/mergeProps.ts:126-131,204-219`; `packages/react/src/internals/use-button/useButton.ts:91-110`).
- Render-prop evaluation (function call vs `cloneElement`) lives in
  `packages/react/src/internals/useRenderElement.tsx:158-206`; the literal `button` tag renders `<button type="button">`
  (`packages/react/src/internals/useRenderElement.tsx:232-235`), doubling the `type: 'button'` guarantee from `getButtonProps`.
- Grouped branch (`packages/react/src/toggle/Toggle.tsx:125-138`): `<CompositeItem tag="button"
  …>` (`packages/react/src/internals/composite/item/CompositeItem.tsx:9-34`) calls
  `useCompositeItem`, which registers the item in the composite list and returns roving-tabindex
  props — `tabIndex: isHighlighted ? 0 : -1`, focus-chasing `onFocus`, hover-focus `onMouseMove`
  (`packages/react/src/internals/composite/item/useCompositeItem.ts:26-42`). This is the mechanism
  behind behavior.md's "Focus management" grouped-`tabIndex` observation. `CompositeItem`
  re-enters `useRenderElement` with the same `state` and the same `props` array
  (`packages/react/src/internals/composite/item/CompositeItem.tsx:27-33`), so data attributes and handlers are identical on both paths; only
  composite props are added on top.
- `itemMetadata` (`packages/react/src/toggle/Toggle.tsx:120-123`): memoized
  `{ disabled, focusableWhenDisabled: false }` typed as `ToolbarRoot.ItemMetadata` — metadata that
  Toolbar reads to compute `disabledIndices` (in-source comment), passed through `CompositeItem`'s
  `metadata`.

Hook inventory: `React.forwardRef` (`packages/react/src/toggle/Toggle.tsx:24`), `useBaseUiId` (`:44`),
`useToggleGroupContext`/`useContext` (`:45`), dev-only `React.useEffect` (`:50-61`),
`useControlled` (`:63-68`), `useButton` (wrapping `useFocusableWhenDisabled`,
`useCompositeRootContext`, `useIsoLayoutEffect`, `useStableCallback`) (`:70-73`), `React.useMemo`
(`:120-123`), `useRenderElement` (wrapping `useMergedRefs`/`useMergedRefsN`) (`:111-116` and again
inside `CompositeItem`).

## Context providers/consumers

Toggle is a pure consumer — it provides no context of its own.

- `ToggleGroupContext`, read via `useToggleGroupContext()`
  (`packages/react/src/toggle/Toggle.tsx:45`; context object at
  `packages/react/src/toggle-group/ToggleGroupContext.ts:21-23`). What crosses the boundary
  (`packages/react/src/toggle-group/ToggleGroupContext.ts:6-19`): `value` (readonly membership array), `setGroupValue` (the group's
  gated committer, which receives Toggle's shared `details` object), `disabled` (group-wide), and
  `isValueInitialized` (dev-warning gate only). The context's *presence or absence* is the unit's
  single branch point: it switches state derivation (prop vs membership probe), the render path
  (`useRenderElement` vs `CompositeItem`), and arms the dev warning.
- Indirect consumership through internals: `useCompositeRootContext` inside `useButton`
  (`packages/react/src/internals/use-button/useButton.ts:25`) and inside `useCompositeItem` (`packages/react/src/internals/composite/item/useCompositeItem.ts:17-18`), provided by
  ToggleGroup's `CompositeRoot` (`packages/react/src/toggle-group/ToggleGroup.tsx:8,111`). This is
  how a grouped Toggle learns roving-tabindex highlighting and registers into the composite list.
- Outbound metadata contract: `ToolbarRoot.ItemMetadata` (type-only import,
  `packages/react/src/toggle/Toggle.tsx:9,120`) — the shape Toolbar's composite rendering reads for
  `disabledIndices`.

## DOM/portal strategy and why

- Single native `<button>`, no wrapper element, and **no portal anywhere in the unit** — neither
  `index.ts` nor `Toggle.tsx` touches portal machinery; behavior.md's "DOM structure & portal
  behavior" portal row is N/A by construction, not by omission.
- Standalone path: `useRenderElement('button', …)` → `<button type="button" {...props}>`
  (`packages/react/src/internals/useRenderElement.tsx:232-235`). Grouped path: `CompositeItem tag="button"`
  (`packages/react/src/toggle/Toggle.tsx:128`) — same element type plus composite props, so the DOM
  contract (one native button, `ref instanceof HTMLButtonElement`) holds in both modes.
- Native-button enforcement: `nativeButton = true` default (`packages/react/src/toggle/Toggle.tsx:38`)
  makes `getButtonProps` emit `type: 'button'` (`packages/react/src/internals/use-button/useButton.ts:226`) and dev-warn if the `render`
  prop substitutes a non-`<button>` (`packages/react/src/internals/use-button/useButton.ts:36-54`). The `form` and `type` props are
  destructured out and deliberately discarded (`packages/react/src/toggle/Toggle.tsx:32,36` —
  "never participates in form validation", "cannot change button type"): a Toggle can never become a
  submit button.
- `aria-pressed: pressed` in the base props (`packages/react/src/toggle/Toggle.tsx:83`); React
  stringifies ARIA booleans, which is exactly how the always-present `'true'`/`'false'` values
  behavior.md's "Accessibility" section records are produced.
- Why the native default matters: Space/Enter activation and disabled-click suppression come free
  from the browser; `useButton`'s keyboard-synthesis machinery only activates for non-native renders
  or composite (grouped) items.

## Dependencies on other Base UI internals

Shared `@base-ui/utils` hooks:

- `useControlled` — `packages/react/src/toggle/Toggle.tsx:3`
- `error` — `packages/react/src/toggle/Toggle.tsx:4` (used directly and via `useControlled`/`useButton`)

`internals/`:

- `internals/useBaseUiId` — `packages/react/src/toggle/Toggle.tsx:5`
- `internals/useRenderElement` — `packages/react/src/toggle/Toggle.tsx:6` (shared element renderer:
  prop merging, ref merging, state→data-attribute mapping, render-prop evaluation)
- `internals/types` (`BaseUIComponentProps`, `NativeButtonProps`) —
  `packages/react/src/toggle/Toggle.tsx:7`
- `internals/use-button/useButton` — `packages/react/src/toggle/Toggle.tsx:10` (transitively pulls
  `utils/useFocusableWhenDisabled`, `utils/dispatchClickWithModifiers`,
  `internals/composite/root/CompositeRootContext`, and `merge-props`)
- `internals/composite/item/CompositeItem` — `packages/react/src/toggle/Toggle.tsx:11` (composite
  subsystem: list registration + roving tabindex), with `internals/composite/item/useCompositeItem`
  beneath it
- `internals/createBaseUIEventDetails` — `packages/react/src/toggle/Toggle.tsx:12-15`
- `internals/reasons` (`REASONS.none`) — `packages/react/src/toggle/Toggle.tsx:16`

Cross-component units (the precise per-component dependency the TODO `blocked-by` field should
encode):

- `toggle-group/ToggleGroupContext` — `packages/react/src/toggle/Toggle.tsx:8`. Hard runtime
  dependency: standalone rendering never touches it, but grouped rendering consumes the context and
  calls `setGroupValue` through it. Toggle (item) and ToggleGroup (root) form a provider/consumer
  pair, bridged transitively through `CompositeRoot`/`CompositeItem`.
- `toolbar/root/ToolbarRoot` — type-only (`packages/react/src/toggle/Toggle.tsx:9`): the
  `ItemMetadata` shape Toggle emits for Toolbar's `disabledIndices`.

Not used: `floating-ui-react`, the `use-render` package, and portal machinery are absent from this
unit.

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit loop:

1. `data-pressed` is declared (`packages/react/src/toggle/ToggleDataAttributes.ts:4`) and produced
   by the state mapping (`packages/react/src/toggle/Toggle.tsx:75-78`), but behavior.md records no
   test asserting `data-pressed` on a Toggle — the suite asserts `aria-pressed` only. (`data-disabled`
   *is* asserted.)
2. The reverse veto (gate 3, `packages/react/src/toggle/Toggle.tsx:100-102`): the grouped-cancellation
   test covers Toggle-side `cancel()` suppressing the group's `onValueChange` (behavior.md "State
   model" grouped-cancellation bullet); no test covers ToggleGroup's `onValueChange` calling
   `cancel()` and thereby suppressing the Toggle's own pressed state through the same shared details
   object.
3. The `nativeButton={false}` path: the prop default (`packages/react/src/toggle/Toggle.tsx:38`) and useButton's
   `role="button"` + keyboard-synthesis machinery (`packages/react/src/internals/use-button/useButton.ts:130-175,226`) are untested in this
   unit's suite — behavior.md's render-prop conformance only substitutes a native `button`.
4. Discarded `form`/`type` props (`packages/react/src/toggle/Toggle.tsx:32,36`): no test pins that
   `type="submit"` (or a `form` id) is ignored and never reaches the DOM.
5. Falsy `value` normalization (`valueProp || undefined`, `packages/react/src/toggle/Toggle.tsx:43-44`): `value=""` silently
   becomes a generated id; no test covers it. Relatedly, the `if (value)` group-commit gate
   (`packages/react/src/toggle/Toggle.tsx:96`) is only falsy pre-mount on the React ≤17 id fallback
   (`packages/utils/src/useId.ts:8-22`) — untested.
6. The dev-only warning for a valueless Toggle inside a ToggleGroup
   (`packages/react/src/toggle/Toggle.tsx:50-61`) is asserted by no test in this unit's suite.
7. The runtime counterpart of that warning: on the client the generated id is truthy, so `if (value)`
   passes and a valueless grouped Toggle commits its random `base-ui-*` id into the group value on
   click (`packages/react/src/toggle/Toggle.tsx:96-98`) — the "issues" the warning text describes. No test exercises this
   path; only the dev warning stands between users and corrupted group values.
8. `itemMetadata` (`{ disabled, focusableWhenDisabled: false }`, `packages/react/src/toggle/Toggle.tsx:120-123`) is a
   Toolbar-only contract (`disabledIndices`); no test in this unit exercises Toolbar consuming it.
9. Grouped keyboard differences: inside ToggleGroup's `CompositeRoot`, Space is dispatched
   synthetically on keydown and keyup Space is suppressed (`packages/react/src/internals/use-button/useButton.ts:138-152,186-196`). No test
   in this unit's suite dispatches keyboard events at all (behavior.md "Keyboard interactions" is
   N/A), so neither the standalone-native nor the grouped-composite activation timing is pinned.
