# Radio — implementation spec (Stage 2: mined from source)

Unit: `radio` (`packages/react/src/radio`). Companion to `specs/library/radio/behavior.md`, which is the ground truth for WHAT the unit does (mined from its tests); this document explains HOW the source produces that behavior. Citations point at source files only (this unit's own files plus the internals they import), never at test files. `RadioGroup` is a separate unit (`library: radio-group`) and is treated here only as a consumed context, not re-derived.

The `TODO.md` entry for `library: radio` (TODO.md:476-482) declares no `wraps-external:` field, so there is no external-package delegation to document: nothing in this unit is outsourced to a third-party npm package, and all behavior below is derived from this unit's own source.

## State machine / hooks used

### Radio.Root — a stateless selection proxy

The Root owns no checked state. `checked` is a pure per-render derivation (`packages/react/src/radio/root/RadioRoot.tsx:83`):

- **Group mode** — `useRadioGroupContext()` returned a provider (`packages/react/src/radio/root/RadioRoot.tsx:53`): `checked = checkedValue === value`, so selection truth lives entirely in the group's context state.
- **Standalone mode** — no group: `checked = value === ''`. This branch exists so the Root renders sensibly without a group, but no behavior test exercises a checked standalone Root (flagged in the last section).

The group context is destructured with per-field fallbacks so every member is optional (`packages/react/src/radio/root/RadioRoot.tsx:55-67`); `setCheckedValue` and `setTouched` fall back to `NOOP` (`packages/react/src/radio/root/RadioRoot.tsx:64-65`). The mode flag `isRadioGroup` (`packages/react/src/radio/root/RadioRoot.tsx:227`) selects both the rendering path and the `checked` rule.

State mutation is delegated end-to-end: the hidden input's `onChange` builds `createChangeEventDetails(REASONS.none, event.nativeEvent)` and calls `setCheckedValue(value, details)` (`packages/react/src/radio/root/RadioRoot.tsx:184-203`), honoring the cancelable-decision protocol (`details.isCanceled`, `packages/react/src/internals/createBaseUIEventDetails.ts:118-149`) before marking the Field touched (`packages/react/src/radio/root/RadioRoot.tsx:202`). The visible span never sets state itself; the group's re-render flows back down as a new `checked` derivation.

Hook inventory (call sites):

- `useRadioGroupContext` — `packages/react/src/radio/root/RadioRoot.tsx:53`; context defined at `packages/react/src/radio-group/RadioGroupContext.ts:24`.
- `useFieldRootContext`, `useFieldItemContext`, `useLabelableContext` — `packages/react/src/radio/root/RadioRoot.tsx:69-76`.
- `React.useCallback` — field-validation `registerInput` closure (`packages/react/src/radio/root/RadioRoot.tsx:88-93`).
- `useMergedRefs` — folds `inputRefProp`, local `inputRef`, group `registerInputRef`, and the field `registerInput` into one hidden-input ref (`packages/react/src/radio/root/RadioRoot.tsx:94`).
- `useIsoLayoutEffect` ×2 — (1) syncs Field `filled` from the input's native checked state on mount (`packages/react/src/radio/root/RadioRoot.tsx:96-100`); (2) registers the hidden input with the group's roving-focus registry `registerInputRef`, explicitly deregistering (`registerInputRef(null)`) when a checked radio is disabled (`packages/react/src/radio/root/RadioRoot.tsx:102-113`).
- `useBaseUiId` — arbitrary id for the visible control when it does not own the labelable id (`packages/react/src/radio/root/RadioRoot.tsx:115`, consumed at `:131`).
- `useLabelableId` — resolves the effective control id: consumer `id` > Field-registered `controlId` > `base-ui-`-prefixed default (`packages/react/src/radio/root/RadioRoot.tsx:116`; resolution at `packages/react/src/internals/labelable-provider/useLabelableId.ts:82`).
- `useAriaLabelledBy` — the fallback `aria-labelledby` mechanism behind behavior.md's "Accessibility" label-fallback bullets: when `!nativeButton`, it re-scans the DOM after every commit for a `<label>` associated with the hidden input (parent label, next sibling with matching `htmlFor`, or `.labels`) and auto-generates a label id when the label lacks one (`packages/react/src/radio/root/RadioRoot.tsx:118-124`; logic at `packages/react/src/internals/labelable-provider/useAriaLabelledBy.ts:22-31`, `:36-47`, `:49-70`). Running after every commit (dep-free effect) is why the fallback tracks label mount/unmount and `id` prop changes without prop-driven deps.
- `useButton` — invoked with `{ disabled, native: nativeButton, composite: false }` (`packages/react/src/radio/root/RadioRoot.tsx:164-168`); supplies `getButtonProps` (merged into the props chain at `packages/react/src/radio/root/RadioRoot.tsx:233`) and `buttonRef` (merged at `packages/react/src/radio/root/RadioRoot.tsx:229`).
- `React.useMemo` — the `RadioRootState` object (`packages/react/src/radio/root/RadioRoot.tsx:214-223`), which doubles as the context value (`packages/react/src/radio/root/RadioRoot.tsx:225`) and the data-attribute source.

Event wiring that produces behavior.md's "Events" section:

- Root `onKeyDown` unconditionally preventDefaults Enter (`packages/react/src/radio/root/RadioRoot.tsx:132-138`). `useButton`'s Enter→click path bails on `event.defaultPrevented` (`packages/react/src/internals/use-button/useButton.ts:165-168`), so a radio never activates via Enter — Space only, matching native `<input type="radio">` semantics even in `nativeButton` mode where the real `<button>` would otherwise Enter-activate.
- Root `onClick` bails on already-default-prevented/disabled/readOnly events, then preventDefaults and re-dispatches a synthetic click on the hidden input, preserving modifier state (`packages/react/src/radio/root/RadioRoot.tsx:139-152`; `packages/react/src/utils/dispatchClickWithModifiers.ts:19-36` — untrusted bubbling `click`, `detail: 0`).
- Input `onClick` stopPropagation (`packages/react/src/radio/root/RadioRoot.tsx:204-208`) is the counterweight: the real user click bubbles to ancestors exactly once from the span, while the implementation-detail click dispatched on the input is swallowed at the input itself.
- Input `onFocus` forwards focus to the visible span (`packages/react/src/radio/root/RadioRoot.tsx:209-211`) so keyboard focus always lives on the `role="radio"` element; Root `onFocus` clicks the input when the group flagged `touched` and resets the flag (`packages/react/src/radio/root/RadioRoot.tsx:153-161`) — the Field-touched handoff for label-click activation.

Within `useButton` (with `composite: false` forced, so its composite Space-on-keydown branch never applies): Space on a non-native span dispatches a click on keyup (`packages/react/src/internals/use-button/useButton.ts:207-216`), which lands in Root's `onClick` above; in `nativeButton` mode the native `<button>` activates natively and `useButton` instead contributes `type: 'button'` (`packages/react/src/internals/use-button/useButton.ts:226`).

### Radio.Indicator — mount/exit-animation state machine

The Indicator is a pure function of `RadioRootState.checked` (behavior.md's "Indicator mount semantics") plus a small animation status machine from `useTransitionStatus(rendered)` (`packages/react/src/radio/indicator/RadioIndicator.tsx:25-27`):

- `rendered = rootState.checked` — no local boolean; `mounted` and `transitionStatus` are the only state, owned by `useTransitionStatus` (`packages/react/src/internals/useTransitionStatus.ts:17-29`).
- Checking an unmounted indicator flips `mounted` to `true` and sets `'starting'` during render (`packages/react/src/internals/useTransitionStatus.ts:31-34`), which is what produces a `data-starting-style` enter phase on every open; unchecking a mounted one sets `'ending'` during render (`packages/react/src/internals/useTransitionStatus.ts:36-38`); the `'starting'` status is cleared on the next animation frame (`packages/react/src/internals/useTransitionStatus.ts:58-72`) so starting styles apply only to the first frame(s).
- `shouldRender = keepMounted || mounted` (`packages/react/src/radio/indicator/RadioIndicator.tsx:36`) with an early `return null` otherwise (`packages/react/src/radio/indicator/RadioIndicator.tsx:57-59`).
- `useOpenChangeComplete({ batch: true, enabled: !rendered, open: rendered })` (`packages/react/src/radio/indicator/RadioIndicator.tsx:45-55`; `packages/react/src/internals/useOpenChangeComplete.tsx:9-28`) waits for the exit animation/transition to finish (or fires immediately when none is defined) and then `setMounted(false)` — this is exactly behavior.md's "Indicator unmount timing" split: immediate removal without an animation, removal only after `data-ending-style` finishes with one. `indicatorRef` (`packages/react/src/radio/indicator/RadioIndicator.tsx:34`) is the observed node.
- `transitionStatus` is folded into the state object (`packages/react/src/radio/indicator/RadioIndicator.tsx:29-32`) so it reaches the data attributes via the mapping below.

### Data-attribute mapping (both parts)

`packages/react/src/radio/utils/stateAttributesMapping.ts:7-20` turns the state object into the style hooks documented in behavior.md's "State model" (`data-checked`/`data-unchecked` mutual exclusivity): the `checked` boolean maps to `data-checked` XOR `data-unchecked` (`packages/react/src/radio/utils/stateAttributesMapping.ts:8-13`; constants at `packages/react/src/radio/root/RadioRootDataAttributes.ts:4-8`), `transitionStatus` maps to `data-starting-style`/`data-ending-style` via the shared `transitionStatusMapping` (`packages/react/src/radio/utils/stateAttributesMapping.ts:14`), and Field `valid` maps to `data-valid`/`data-invalid` via `fieldValidityMapping` (`packages/react/src/radio/utils/stateAttributesMapping.ts:15`). `useRenderElement` applies the mapping through `getStateAttributesProps` (`packages/react/src/internals/useRenderElement.tsx:76-78`). The per-part `*DataAttributes.ts` files (`packages/react/src/radio/root/RadioRootDataAttributes.ts`, `packages/react/src/radio/indicator/RadioIndicatorDataAttributes.ts`) re-declare the same strings for docs/typing; the indicator's `startingStyle`/`endingStyle` come from the shared transition constants (`packages/react/src/radio/indicator/RadioIndicatorDataAttributes.ts:26-30`).

## Context providers/consumers

**Provided:**

- `RadioRootContext` — the provider wraps BOTH the visible element and the hidden input (`packages/react/src/radio/root/RadioRoot.tsx:248-265`); its value is the memoized `RadioRootState` (`packages/react/src/radio/root/RadioRoot.tsx:225`; type at `packages/react/src/radio/root/RadioRootContext.ts:5-7`). The only in-unit consumer is `RadioIndicator` via `useRadioRootContext` (`packages/react/src/radio/indicator/RadioIndicator.tsx:23`), which throws a diagnostic error outside a Root (`packages/react/src/radio/root/RadioRootContext.ts:9-18`) — the source-side answer to behavior.md's unverified nesting question. Because the state spreads `fieldState` (`packages/react/src/radio/root/RadioRoot.tsx:216`), Field validity/touched/dirty/filled flow into the Indicator's data attributes with no extra wiring.

**Consumed** (each optional; the Root tolerates all of them being absent):

- `RadioGroupContext` (`packages/react/src/radio/root/RadioRoot.tsx:53`; shape at `packages/react/src/radio-group/RadioGroupContext.ts:7-22`) — the selection owner: `checkedValue`/`setCheckedValue`, `name`, `form`, `disabled`/`readOnly`/`required`, `touched`/`setTouched`, `validation`, `registerInputRef`. Its presence/absence switches the render path (next section) and the `checked` rule.
- Field contexts — `useFieldRootContext` (`packages/react/src/radio/root/RadioRoot.tsx:69-74`) contributes `setTouched`, `setFilled`, `fieldState`, and a field-level `disabled`; `useFieldItemContext` (`packages/react/src/radio/root/RadioRoot.tsx:75`) contributes a per-item `disabled`. Merging uses first-truthy-wins: `disabled = fieldDisabled || fieldItemContext.disabled || disabledGroup || disabledProp`, while `readOnly` and `required` prefer the group over the local prop (`packages/react/src/radio/root/RadioRoot.tsx:78-80`).
- `LabelableContext` (`packages/react/src/radio/root/RadioRoot.tsx:76`) — `labelId` (from a Field `Label`) joins the `aria-labelledby` resolution via `useAriaLabelledBy`; `getDescriptionProps` (Field `Description`) is merged into the root props chain (`packages/react/src/radio/root/RadioRoot.tsx:234`).

## DOM/portal strategy and why

No portals anywhere in this unit (consistent with behavior.md's "DOM structure & portal behavior" N/A). Each Root renders exactly two sibling DOM nodes:

1. **The visible control** — `span[role="radio"]` by default (`packages/react/src/radio/root/RadioRoot.tsx:126-131`, default tag at `packages/react/src/radio/root/RadioRoot.tsx:240`). It carries all ARIA (`aria-checked`, `aria-labelledby`, `id`) and every interactive handler; it is the focus/keyboard/composite target users interact with.
2. **A hidden `<input type="radio">`** (`packages/react/src/radio/root/RadioRoot.tsx:170-212`, rendered at `packages/react/src/radio/root/RadioRoot.tsx:264` with `suppressHydrationWarning`) — `aria-hidden: true`, `tabIndex: -1`, visually hidden via `visuallyHiddenInput` when the group has a `name` (absolute positioning so it stays in form flow) or the generic `visuallyHidden` otherwise (`packages/react/src/radio/root/RadioRoot.tsx:176-178`; styles at `packages/utils/src/visuallyHidden.ts:14-24`). Why it exists: it is the form participant (`name`/`form`/`value`/`checked`/`required`/`readOnly` native submit semantics, `packages/react/src/radio/root/RadioRoot.tsx:171-183`), the label-association target, and the pivot of the click re-dispatch choreography. Note the Root's `value` prop is an identity tag for the group comparison, but it IS serialized onto this input (`value: serializeValue(value)`, `packages/react/src/radio/root/RadioRoot.tsx:179`; `packages/react/src/internals/serializeValue.ts:1-13`, `null` → `''`); behavior.md's "never forwarded to the DOM" claim is about the visible control only.

**ID choreography** (behavior.md's "Accessibility" ID-linking bullets): the consumer `id` resolves through `useLabelableId` (`inputId`) and goes to the visible control in `nativeButton` mode and to the hidden input otherwise (`hiddenInputId` is `undefined` when `nativeButton`, `packages/react/src/radio/root/RadioRoot.tsx:117`; selection at `packages/react/src/radio/root/RadioRoot.tsx:131`).

**Click choreography** (the one-ancestor-click invariant in behavior.md's "Events"): span click → `preventDefault()` + `dispatchClickWithModifiers(input, event)` → the input checks natively → React `onChange` → `setCheckedValue` → group re-render. The input's `onClick` stopPropagation keeps the synthetic click from re-reaching ancestors, so exactly one click event (the original user click) propagates upward.

**Role resolution detail:** `rootProps` sets `role: 'radio'` (`packages/react/src/radio/root/RadioRoot.tsx:127`) while `useButton` injects a `role: 'button'` fallback for non-native elements (`packages/react/src/internals/use-button/useButton.ts:226`). The explicit role wins because props-getters in the props array receive previously-merged props as their argument and non-handler conflicts are rightmost-wins: `getButtonProps` is invoked with the `rootProps`+`elementProps` merge (`packages/react/src/internals/useRenderElement.tsx:121-129`) and merges that argument last internally (`packages/react/src/internals/use-button/useButton.ts:226-229`; ordering semantics at `packages/react/src/merge-props/mergeProps.ts:14-17`).

**Group-mode rendering:** when a RadioGroup is present, the visible element is rendered by `CompositeItem` instead of the local `useRenderElement` call (`packages/react/src/radio/root/RadioRoot.tsx:250-260` vs `packages/react/src/radio/root/RadioRoot.tsx:240-246` with `enabled: !isRadioGroup` — a disabled `useRenderElement` returns null, `packages/react/src/internals/useRenderElement.tsx:40-42`). Why: `CompositeItem` layers `useCompositeItem`'s roving-focus/arrow-key props and ref on top of the same props array (`packages/react/src/internals/composite/item/CompositeItem.tsx:25-33`), which is how behavior.md's "Keyboard interactions" ArrowDown row is produced. The Root's own contributions to navigation are the registered hidden input (`packages/react/src/radio/root/RadioRoot.tsx:102-113`) and the `data-composite-item-active` marker on the checked item (`ACTIVE_COMPOSITE_ITEM`, `packages/react/src/radio/root/RadioRoot.tsx:130`; constant at `packages/react/src/internals/composite/constants.ts:1`).

Render/className/style merging for both parts is the standard `useRenderElement` pipeline (`packages/react/src/radio/indicator/RadioIndicator.tsx:38-43`), with `stateAttributesMapping` supplying the `data-*` hooks; the `render`-prop ref/className/prop forwarding asserted by conformance comes from ref merging (`packages/react/src/internals/useRenderElement.tsx:100-116`) and `evaluateRenderProp` (`packages/react/src/internals/useRenderElement.tsx:158-206`).

Public surface is thin glue: `packages/react/src/radio/index.parts.ts:1-2` exports `Root`/`Indicator` under the `Radio` namespace, and `packages/react/src/radio/index.ts:1-4` re-exports the state/props types.

## Dependencies on other Base UI internals

Grouped by internal path (imports cited at `packages/react/src/radio/root/RadioRoot.tsx:3-26` and `packages/react/src/radio/indicator/RadioIndicator.tsx:3-9`):

- `internals/useRenderElement` — element/props/state/render-prop pipeline for both parts (`packages/react/src/radio/root/RadioRoot.tsx:240-246`, `packages/react/src/radio/indicator/RadioIndicator.tsx:38-43`).
- `internals/use-button` (`useButton`) — button semantics, disabled handling, Space/Enter activation, dev-mode native/non-native mismatch warnings (`packages/react/src/radio/root/RadioRoot.tsx:15`, `:164-168`).
- `internals/composite/item/CompositeItem` + `internals/composite/constants` (`ACTIVE_COMPOSITE_ITEM`) — group navigation integration, rendered only when a RadioGroup context exists (`packages/react/src/radio/root/RadioRoot.tsx:16-17`, `:130`, `:250-260`).
- `internals/useTransitionStatus` + `internals/useOpenChangeComplete` — Indicator mount/exit-animation lifecycle (`packages/react/src/radio/indicator/RadioIndicator.tsx:8-9`, `:27`, `:45-55`).
- `internals/stateAttributesMapping` (`transitionStatusMapping`) + `internals/field-constants/constants` (`fieldValidityMapping`) — shared attribute mappings (`packages/react/src/radio/utils/stateAttributesMapping.ts:3-4`, `:14-15`).
- `internals/labelable-provider` — `useLabelableContext`, `useAriaLabelledBy`, `useLabelableId` (`packages/react/src/radio/root/RadioRoot.tsx:21-23`, `:76`, `:116-124`).
- `internals/field-root-context/FieldRootContext` + `field/item/FieldItemContext` — Field state/touched/disabled integration (`packages/react/src/radio/root/RadioRoot.tsx:19-20`, `:69-75`).
- `radio-group/RadioGroupContext` — sibling component unit that owns selection state; this unit consumes it and its selection semantics cannot be specified without it (`packages/react/src/radio/root/RadioRoot.tsx:24`, `:53`).
- `utils/dispatchClickWithModifiers` — synthetic click re-dispatch (`packages/react/src/radio/root/RadioRoot.tsx:12`, `:151`).
- `internals/createBaseUIEventDetails` + `internals/reasons` + `internals/noop` — cancelable change details, `REASONS.none`, no-group fallbacks (`packages/react/src/radio/root/RadioRoot.tsx:8-10`, `:194-196`).
- `internals/serializeValue` — `value` → input `value` attribute (`packages/react/src/radio/root/RadioRoot.tsx:25`, `:179`).
- `internals/useBaseUiId` — id generation (`packages/react/src/radio/root/RadioRoot.tsx:13`, `:115`).
- `@base-ui/utils` (cross-package): `useMergedRefs` (`packages/react/src/radio/root/RadioRoot.tsx:3`, `:94`), `useIsoLayoutEffect` (`packages/react/src/radio/root/RadioRoot.tsx:4`, `:96`, `:102`; also inside the hooks above), `visuallyHidden`/`visuallyHiddenInput` (`packages/react/src/radio/root/RadioRoot.tsx:5`, `:177`), `EMPTY_OBJECT` (`packages/react/src/radio/root/RadioRoot.tsx:6`, `:179`, `:237`).
- **Notably NOT used:** `floating-ui-react` (no positioning anywhere in this unit) and `use-render` (render-prop handling lives in `useRenderElement`). The Indicator's dependency surface is much smaller than the Root's — `useRenderElement`, `useRadioRootContext`, `useTransitionStatus`, `useOpenChangeComplete`, and the shared mapping only.

No `wraps-external:` delegation exists for this unit (TODO.md:476-482 declares none), so no external package's internals are treated as given here.

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit loop; none of these are covered by behavior.md's proven claims:

- **Standalone checked rule.** `checked = value === ''` when no group is present (`packages/react/src/radio/root/RadioRoot.tsx:83`). Conformance renders Root standalone but asserts nothing about its checked state; no test ever shows a standalone Root checked. The local-render branch gated by `enabled: !isRadioGroup` (`packages/react/src/radio/root/RadioRoot.tsx:241-246`, `:261-263`) is likewise exercised only through conformance's generic suites.
- **`value` on the hidden input.** `serializeValue(value)` lands on the input's `value` attribute (`packages/react/src/radio/root/RadioRoot.tsx:179`), including `JSON.stringify` for non-string values (`packages/react/src/internals/serializeValue.ts:5-11`). The unit's only value-forwarding test asserts the visible control lacks a `value` attribute; the input's attribute and serialization rules are untested.
- **`onChange` cancellation + touched.** The `details.isCanceled` early return and `setFieldTouched(true)` (`packages/react/src/radio/root/RadioRoot.tsx:198-202`) are not exercised by this unit's tests; selection is proven, the cancelable-decision protocol is not.
- **Enter suppression.** The unconditional Enter preventDefault in Root `onKeyDown` (`packages/react/src/radio/root/RadioRoot.tsx:132-138`) has no test in this unit (behavior.md's "Keyboard interactions" covers ArrowDown only).
- **Focus behaviors.** Input→span focus forwarding (`packages/react/src/radio/root/RadioRoot.tsx:209-211`), the `touched`-driven re-click plus `setTouched(false)` in Root `onFocus` (`packages/react/src/radio/root/RadioRoot.tsx:153-161`), and the disabled+checked `registerInputRef(null)` deregistration (`packages/react/src/radio/root/RadioRoot.tsx:107-110`) are all untested here — behavior.md's "Focus management" section declares this territory UNVERIFIED/N-A.
- **Field sync effect.** The `setFilled(true)` hydration effect reading `inputRef.current?.checked` (`packages/react/src/radio/root/RadioRoot.tsx:96-100`) is untested.
- **`inputRef` prop.** Declared (`packages/react/src/radio/root/RadioRoot.tsx:46`, `:329-331`) and merged (`:94`); no test asserts it resolves to the hidden input.
- **`readOnly` / `required`.** Both accepted (`packages/react/src/radio/root/RadioRoot.tsx:42-43`), merged (`:79-80`), forwarded to the input (`:182-183`), mapped to `data-readonly`/`data-required` (`packages/react/src/radio/root/RadioRootDataAttributes.ts:16-20`), and gated in `onClick`/`onChange` (`:140`, `:190`); this unit's tests assert only `disabled`.
- **`ACTIVE_COMPOSITE_ITEM` marker.** `data-composite-item-active` on the checked item (`packages/react/src/radio/root/RadioRoot.tsx:130`) is consumed by composite internals; no test in this unit asserts the attribute.
- **`suppressHydrationWarning`.** On the input (`packages/react/src/radio/root/RadioRoot.tsx:264`) — SSR-related, untestable in this unit's environments.
- **Named-input style switch.** `visuallyHiddenInput` vs `visuallyHidden` depending on whether the group provides `name` (`packages/react/src/radio/root/RadioRoot.tsx:177`) is untested.
- **`RadioRoot.spec.tsx`** (`packages/react/src/radio/root/RadioRoot.spec.tsx:1-12`) is a type-level fixture, not a behavior test: it pins the `Value` generic (accepts string/number/`null`, rejects a mismatched literal via `@ts-expect-error`). It contributes no runtime behavior and explains nothing about the state machine — it exists purely for the typechecker.
