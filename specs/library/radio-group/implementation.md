# radio-group — implementation spec

Companion to `behavior.md` (same directory), which documents WHAT happens; this file explains WHY/HOW
from the non-test source files:

- `packages/react/src/radio-group/index.ts`
- `packages/react/src/radio-group/RadioGroup.tsx`
- `packages/react/src/radio-group/RadioGroup.spec.tsx`
- `packages/react/src/radio-group/RadioGroupContext.ts`
- `packages/react/src/radio-group/RadioGroupDataAttributes.ts`

The unit's `TODO.md` entry (`library: radio-group`, `TODO.md:487-493`) has no `wraps-external:`
field — behavior is fully internal to the repo; there is no third-party package whose internals this
spec delegates to.

`index.ts` exports only `RadioGroup` and its types (`packages/react/src/radio-group/index.ts:1-3`);
this closes the UNVERIFIED flag in behavior.md's "Public API surface" — the group exports no
subcomponents of its own.

## State machine / hooks used

### In `RadioGroup.tsx`

- `useControlled` — `packages/react/src/radio-group/RadioGroup.tsx:72-77`. The controlled/
  uncontrolled duality of behavior.md's "State model" lives here: `externalValue` (the `value` prop)
  vs `defaultValue`. The hook resolves the mode once on first render and its setter no-ops when
  controlled (`packages/utils/src/useControlled.ts:41-45,82-91`), so the gate below can commit
  unconditionally in uncontrolled mode while controlled consumers ignore the write.
- `setCheckedValue` (wrapped in `useStableCallback`) —
  `packages/react/src/radio-group/RadioGroup.tsx:80-90`. This is the single commit gate for every
  value change in the unit: it invokes `onValueChange` first, then only unwraps into
  `useControlled`'s setter when `eventDetails.isCanceled` is false. All cancel semantics in
  behavior.md ("State model" → Cancellation; "Events") funnel through this one check — the group
  never writes state directly, and children call this gate from their hidden input's `onChange`
  (child-side cancel handling: `packages/react/src/radio/root/RadioRoot.tsx:184-203`).
- Generic `Value` typing — the component is `forwardRef`-wrapped then cast back to a generic callable
  (`packages/react/src/radio-group/RadioGroup.tsx:31-34,276-278`) because `forwardRef` erases
  generics; `RadioGroup.spec.tsx` pins the resulting inference (`value={narrowedValue}` narrows
  `onValueChange`, explicit `<RadioGroup<string | null>>` admits `null`).
- Field/labelable/form/fieldset context reads (see "Context providers/consumers" below), then two
  merges derived from them: `disabled = fieldDisabled || disabledProp`
  (`packages/react/src/radio-group/RadioGroup.tsx:68`) and `name = fieldName ?? nameProp`
  (`packages/react/src/radio-group/RadioGroup.tsx:69`) — the latter is behavior.md's
  "Field name takes precedence" rule. `useBaseUiId(idProp)` —
  `packages/react/src/radio-group/RadioGroup.tsx:70` (prefixing wrapper at
  `packages/react/src/internals/useBaseUiId.ts:9-11`).
- Group-local `touched` state — `packages/react/src/radio-group/RadioGroup.tsx:78`. Despite its
  name this is NOT the Field's touched flag; it is an "auto-select-on-focus armed" flag: set in
  `onKeyDownCapture` for any Arrow keydown (`packages/react/src/radio-group/RadioGroup.tsx:249-254`),
  exposed via context, consumed (and reset) by the child's `onFocus`, which clicks its hidden input
  so the newly arrow-focused radio selects itself
  (`packages/react/src/radio/root/RadioRoot.tsx:153-161`). That child round-trip, not the capture
  handler, is what makes behavior.md's "arrow keys move focus AND select" work. The Field-level
  touched (`data-touched`) is a separate flag: set by the group's blur handler
  (`packages/react/src/radio-group/RadioGroup.tsx:241`) or by a child after a committed change
  (`packages/react/src/radio/root/RadioRoot.tsx:202`).
- Getter-based `controlRef` — `packages/react/src/radio-group/RadioGroup.tsx:92-100`. A
  `useMemo`'d ref object whose `.current` getter resolves live through `validation.getInputControl()`,
  i.e. the field-validation registry's representative-input pick (first eligible invalid input, else
  first eligible — `packages/react/src/field/root/useFieldValidation.ts:51-66,118-121`). The group
  owns no input itself; this lazy ref is how Form error-focusing (behavior.md "Focus management")
  lands on the right child radio at focus time instead of registration time.
- inputRef machinery (the unit's most intricate piece):
  - `groupInputRef` / `firstEnabledInputRef` — `packages/react/src/radio-group/RadioGroup.tsx:101-102`:
    the current public representative and the first enabled registered input (the null-value
    fallback).
  - `setInputRef` — `packages/react/src/radio-group/RadioGroup.tsx:108-122`: forwards to the public
    `inputRef` (function refs may return a cleanup, which is propagated), tracks `groupInputRef`,
    returns the cleanup.
  - `registerInputRef` (wrapped in `useStableCallback`) —
    `packages/react/src/radio-group/RadioGroup.tsx:124-157`. Children attach this to their hidden
    input (part of the child's merged ref, `packages/react/src/radio/root/RadioRoot.tsx:94`). It
    skips null/disabled inputs, remembers the first enabled one, and re-points the public ref when
    the input is checked, no representative exists yet, or the current representative is disabled —
    the mechanism behind behavior.md's "Disabled radios are skipped" and "ref updates when selection
    moves" bullets. Its unmount cleanup re-checks live state rather than trusting captured state
    (comment at `packages/react/src/radio-group/RadioGroup.tsx:139-141`) because a re-registration
    (child layout effect, `packages/react/src/radio/root/RadioRoot.tsx:102-113`) can re-point the
    ref after attach.
  - Design constraint (in-source comment, `packages/react/src/radio-group/RadioGroup.tsx:104-107`):
    the registry (`validation.registeredInputs`) is authoritative for validation and form-value
    projection, so the group deliberately never writes `validation.inputRef` — a stale, unmounted
    radio left there would become the Field's fallback once the registry empties and keep blocking
    submission.
- `getFormValue` (wrapped in `useStableCallback`) —
  `packages/react/src/radio-group/RadioGroup.tsx:159-172`. The group's form-value projection
  override: with no surrounding Form element it returns the logical `checkedValue ?? null`; with one,
  it scans `validation.registeredInputs` and returns `checkedValue ?? null` only if some registered
  input is both checked and eligible (`isEligibleInput`,
  `packages/react/src/field/root/useFieldValidation.ts:32-44`), else `null`. This single function
  produces the split behavior.md documents between native `FormData`/`onFormSubmit` outcomes:
  disabled checked radios, radios bound to another `form`, and fully unmounted groups all project
  `null`, while a portaled-but-registered selection still projects its value.
- `useRegisterFieldControl(controlRef, id, checkedValue ?? null, getFormValue, !disabled, nameProp)`
  — `packages/react/src/radio-group/RadioGroup.tsx:174`. Registers the group as the field's control
  with `getFormValue` as the value-getter override; re-registration updates the form's fields Map
  entry in place so field ordering stays stable
  (`packages/react/src/internals/field-register-control/useRegisterFieldControl.ts:18-38`), with an
  unmount cleanup that unregisters (`packages/react/src/internals/field-register-control/useRegisterFieldControl.ts:40-45`).
  Note it gates only on `!disabled` and forwards the raw `nameProp` (not the field-resolved `name`).
- `useValueChanged(checkedValue, …)` — `packages/react/src/radio-group/RadioGroup.tsx:176-189`
  (utility at `packages/react/src/internals/useValueChanged.ts:6-17`, a layout-effect "previous
  value" watcher). On each committed change it: (1) calls `clearErrors(name)` from `useFormContext`
  (behavior.md's "external Form errors clear on change" bullet), (2) recomputes dirty against
  `validityData.initialValue`, (3) sets `filled` (`checkedValue != null`), (4) calls
  `validation.change(checkedValue)`, routing into the field-validation boundary/debounce machinery
  (`packages/react/src/field/root/useFieldValidation.ts:349-365`) — the source of the
  external-revalidation and onBlur/onSubmit timing rules in behavior.md, and (5) when the value
  becomes `null`, imperatively re-points the public ref to `firstEnabledInputRef` (outside React's
  ref lifecycle, comment at `packages/react/src/radio-group/RadioGroup.tsx:186`) — behavior.md's
  "clearing to null keeps the ref on the first radio's input".
- `ariaLabelledby = labelId ?? fieldsetContext?.legendId` —
  `packages/react/src/radio-group/RadioGroup.tsx:191`: Field.Label beats Fieldset.Legend by `??`
  short-circuit; a user-supplied `aria-labelledby` beats both because it overwrites the default in
  the props merge (see "DOM/portal strategy").
- State object — `packages/react/src/radio-group/RadioGroup.tsx:193-198`: `{ ...fieldState,
  disabled, required, readOnly }`, mapped to the group's `data-*` attributes at render time.
- Root event handlers — `packages/react/src/radio-group/RadioGroup.tsx:229-255`: `onFocus` marks the
  field focused (236-238); `onBlur` guards with `contains(event.currentTarget, event.relatedTarget)`
  (239-248) so intra-group focus moves (radio → radio) never touch or validate, and commits
  validation when `validationMode === 'onBlur'` and focus left the group subtree — the exact
  mechanism for behavior.md's `onBlur` validation bullet. `onKeyDownCapture` (249-254) arms the
  group-local `touched` on the capture phase, before any child handler runs.
- Rendering is delegated, not direct: the group renders no element itself. It hands
  `render`/`className`/`style`, the state, a props array, and the forwarded ref to `CompositeRoot`
  (`packages/react/src/radio-group/RadioGroup.tsx:259-273`), which internally calls
  `useRenderElement` (`packages/react/src/internals/composite/root/CompositeRoot.tsx:66-71`), runs
  the roving-tabindex keyboard machinery via `useCompositeRoot`
  (`packages/react/src/internals/composite/root/CompositeRoot.tsx:44-64`), provides
  `CompositeRootContext` downward, and registers children through `CompositeList`
  (`packages/react/src/internals/composite/root/CompositeRoot.tsx:73-94`). Two non-default
  parameters: `enableHomeAndEndKeys={false}` (`packages/react/src/radio-group/RadioGroup.tsx:271`)
  and `modifierKeys={MODIFIER_KEYS}` where `MODIFIER_KEYS = [SHIFT]`
  (`packages/react/src/radio-group/RadioGroup.tsx:8,23,272`).
- Roving tabindex HOW (delegated to composite): each child marks itself with
  `ACTIVE_COMPOSITE_ITEM` when checked (`packages/react/src/radio/root/RadioRoot.tsx:130`); on map
  change `useCompositeRoot` picks the attributed item as the initial tab stop
  (`packages/react/src/internals/composite/root/useCompositeRoot.ts:139-150`) — hence "the checked
  radio owns tabindex 0 initially". When items unmount, the tab stop is relocated with
  `getFallbackIndex`, which prefers the active (checked) item
  (`packages/react/src/internals/composite/root/useCompositeRoot.ts:110-134,345-365`) — hence
  "removing the highlighted radio moves the tab stop to the checked radio". Arrow navigation is
  direction-aware (RTL swaps horizontal keys,
  `packages/react/src/internals/composite/root/useCompositeRoot.ts:221-226`), wraps with loop focus
  (default `loopFocus: true`,
  `packages/react/src/internals/composite/root/useCompositeRoot.ts:76,282-300`), and moves focus in
  a microtask after highlighting
  (`packages/react/src/internals/composite/root/useCompositeRoot.ts:302-316`). The modifier guard
  (`packages/react/src/internals/composite/root/useCompositeRoot.ts:212-214,367-377`) allows
  navigation when the only held modifier is in the `modifierKeys` exemption list — `SHIFT` here is
  behavior.md's "Shift does not block navigation" bullet.
- Space/Enter activation is child-side, not group-side: the group contributes nothing for Space or
  Enter. The child's `useButton` (non-native, non-composite config,
  `packages/react/src/radio/root/RadioRoot.tsx:164-168`) dispatches the activation click on Space
  keyUP (`packages/react/src/internals/use-button/useButton.ts:207-216`) — behavior.md's
  "Space fires on keyup" — and the child pre-prevents Enter's keydown default
  (`packages/react/src/radio/root/RadioRoot.tsx:132-138`) so the button keydown path bails
  (`packages/react/src/internals/use-button/useButton.ts:165-168`) — behavior.md's "Enter does not
  select".

## Context providers/consumers

- `RadioGroupContext.Provider` wraps the single rendered root element
  (`packages/react/src/radio-group/RadioGroup.tsx:257-274`); the context value is memoized at
  `packages/react/src/radio-group/RadioGroup.tsx:200-227`.
- Context shape — `packages/react/src/radio-group/RadioGroupContext.ts:7-22`: `disabled`,
  `readOnly`, `required`, `form`, `name`, `checkedValue`, `setCheckedValue` (the gated committer),
  `touched`/`setTouched` (the group-local arm flag), `validation` (the surrounding Field's
  `UseFieldValidationReturnValue` handle, typed at
  `packages/react/src/radio-group/RadioGroupContext.ts:3,20`), and `registerInputRef`.
- The unit's only in-tree consumer is `Radio.Root`
  (`packages/react/src/radio/root/RadioRoot.tsx:24` imports the hook; read with per-field fallbacks
  at `packages/react/src/radio/root/RadioRoot.tsx:53-67`, so the component also works standalone
  outside a group). Division of labor:
  - `checkedValue` → child derives `checked` (`packages/react/src/radio/root/RadioRoot.tsx:83`),
    feeding `aria-checked` and `data-checked`/`data-unchecked` (behavior.md "Accessibility").
  - `setCheckedValue` → child's hidden-input `onChange` commits through the gate
    (`packages/react/src/radio/root/RadioRoot.tsx:196`).
  - `registerInputRef` → merged onto the hidden input plus a re-registration layout effect
    (`packages/react/src/radio/root/RadioRoot.tsx:94,102-113`).
  - `validation` → child registers its hidden input into the field registry
    (`packages/react/src/radio/root/RadioRoot.tsx:88-93`) and applies
    `validation.getValidationProps` for `aria-invalid`/description linking
    (`packages/react/src/radio/root/RadioRoot.tsx:235-237`).
  - `name`/`form` → copied onto the hidden input
    (`packages/react/src/radio/root/RadioRoot.tsx:173,175`).
  - `touched`/`setTouched` → the arrow-selects round-trip described above.
  - `disabled`/`readOnly`/`required` → merged into the child's effective flags
    (`packages/react/src/radio/root/RadioRoot.tsx:78-80`).
- `Radio.Indicator` does NOT consume `RadioGroupContext`; it reads `RadioRootContext`, which is
  simply the child's derived state (`packages/react/src/radio/root/RadioRoot.tsx:225`), so indicator
  checked/transition state flows transitively
  (`packages/react/src/radio/indicator/RadioIndicator.tsx:23-27`).
- Upstream contexts consumed by the group itself: `useFieldRootContext`
  (`packages/react/src/radio-group/RadioGroup.tsx:52-63` — field disabled/name/state/validation
  handles, `setFieldTouched`/`setFocused`/`setDirty`/`setFilled`, `validationMode`, `validityData`),
  `useLabelableContext` (`packages/react/src/radio-group/RadioGroup.tsx:64` — `labelId`),
  `useFormContext` (`packages/react/src/radio-group/RadioGroup.tsx:65` — `clearErrors` and the form
  `elementRef`, with a NOOP/null default outside a Form,
  `packages/react/src/internals/form-context/FormContext.ts:33-46`), and
  `useFieldsetRootContext(true)` (`packages/react/src/radio-group/RadioGroup.tsx:66` — optional
  overload, `packages/react/src/fieldset/root/FieldsetRootContext.ts:12-22` — for `legendId`).
- Downstream, `CompositeRoot` adds its own `CompositeRootContext` + `CompositeList` registration
  layer between the group and its items
  (`packages/react/src/internals/composite/root/CompositeRoot.tsx:83-94`). Children also consume
  Field/Labelable contexts directly (`packages/react/src/radio/root/RadioRoot.tsx:19-23`), so the
  group only supplies what is group-specific.

## DOM/portal strategy and why

- The group renders one plain `div` (composite tag default
  `packages/react/src/internals/composite/root/CompositeRoot.tsx:38`) with `role: 'radiogroup'`,
  `aria-required`/`aria-disabled`/`aria-readonly` (all `|| undefined` so unset props render nothing),
  and `aria-labelledby` (`packages/react/src/radio-group/RadioGroup.tsx:229-255`). No portal is used
  anywhere in this unit's source — the component is deliberately portal-inert, matching behavior.md's
  "The group itself never portals its own DOM".
- User overrides win by merge order: the props array is `[defaultProps, elementProps,
  validation.getValidationProps]` (`packages/react/src/radio-group/RadioGroup.tsx:264-268`), and
  `mergeProps` resolves scalar conflicts rightmost-wins while composing event handlers
  (`packages/react/src/merge-props/mergeProps.ts:14-15,177-183`; props-getter entries receive the
  merged result of everything before them,
  `packages/react/src/merge-props/mergeProps.ts:25-28`). That is the single mechanism behind
  behavior.md's override bullets (`role="switch"`, explicit `aria-labelledby` beating
  Field.Label/Fieldset.Legend).
- Group `data-*` attributes are not hand-written: the state object flows through
  `useRenderElement` → `getStateAttributesProps`, whose default mapping lowercases each truthy state
  key into `data-{key}` (`packages/react/src/internals/getStateAttributesProps.ts:24-28`) —
  producing `data-disabled`/`data-readonly`/`data-required` plus the field-state attributes — with
  `fieldValidityMapping` overriding `valid` into `data-valid`/`data-invalid`
  (`packages/react/src/internals/field-constants/constants.ts:34-43`, wired at
  `packages/react/src/radio-group/RadioGroup.tsx:270`). `RadioGroupDataAttributes.ts` declares only
  `data-disabled` as the unit's named constant
  (`packages/react/src/radio-group/RadioGroupDataAttributes.ts:4`).
- The group owns no form control and renders no inputs. Each radio's hidden `<input type="radio">`
  is the child's DOM, rendered as a sibling of the exposed element
  (`packages/react/src/radio/root/RadioRoot.tsx:264`) with its `name`/`form` taken from context, its
  `value` serialized via `serializeValue`
  (`packages/react/src/radio/root/RadioRoot.tsx:175,179`), and visually-hidden styling
  (`packages/react/src/radio/root/RadioRoot.tsx:177`).
- Portal tolerance needs no group-side code: field registration and value projection are
  context-driven, and `isEligibleInput`'s doc comment states the policy that portaled inputs still
  belong to the surrounding Form unless explicitly associated elsewhere
  (`packages/react/src/field/root/useFieldValidation.ts:26-44`). The group's only contribution is
  the `getFormValue` eligibility scan described above. This is the mechanism behind behavior.md's
  "Portals" and `form`-association bullets.

## Dependencies on other Base UI internals

Shared `@base-ui/utils` hooks:

- `useControlled` — `packages/react/src/radio-group/RadioGroup.tsx:3`
- `useStableCallback` — `packages/react/src/radio-group/RadioGroup.tsx:4` (three call sites: the
  commit gate, `registerInputRef`, `getFormValue`)

`internals/`:

- `internals/types` (`BaseUIComponentProps`, `HTMLProps`) — `packages/react/src/radio-group/RadioGroup.tsx:5`
- `internals/useBaseUiId` — `packages/react/src/radio-group/RadioGroup.tsx:6`
- `internals/composite/composite` (`SHIFT`) — `packages/react/src/radio-group/RadioGroup.tsx:8`
- `internals/composite/root/CompositeRoot` — `packages/react/src/radio-group/RadioGroup.tsx:9`
  (the entire rendering + roving-tabindex + direction layer; transitively `useRenderElement`,
  `useCompositeRoot`, `CompositeList`, and `useDirection` from
  `internals/direction-context/DirectionContext` —
  `packages/react/src/internals/composite/root/CompositeRoot.tsx:11,42` — which is where RTL
  behavior.md claims originate)
- `internals/field-root-context/FieldRootContext` — `packages/react/src/radio-group/RadioGroup.tsx:10`
- `internals/field-register-control/useRegisterFieldControl` — `packages/react/src/radio-group/RadioGroup.tsx:11`
- `internals/field-constants/constants` (`fieldValidityMapping`) — `packages/react/src/radio-group/RadioGroup.tsx:12`
- `internals/form-context/FormContext` — `packages/react/src/radio-group/RadioGroup.tsx:16`
- `internals/labelable-provider/LabelableContext` — `packages/react/src/radio-group/RadioGroup.tsx:17`
- `internals/useValueChanged` — `packages/react/src/radio-group/RadioGroup.tsx:18`
- `internals/createBaseUIEventDetails` + `internals/reasons` (change-event details,
  `REASONS.none`) — `packages/react/src/radio-group/RadioGroup.tsx:20-21`
- `internals/merge-props/mergeProps` (transitively, via the render pipeline) — merge/override
  semantics above

Field subsystem (the deepest coupling):

- `field/root/useFieldValidation` — `isEligibleInput` imported directly
  (`packages/react/src/radio-group/RadioGroup.tsx:14`), and the `validation` handle typed from it
  and shared through context (`packages/react/src/radio-group/RadioGroupContext.ts:3,20`)
- `field/root/FieldRoot` — `FieldRootState` type only
  (`packages/react/src/radio-group/RadioGroup.tsx:13`)

Fieldset subsystem:

- `fieldset/root/FieldsetRootContext` — `packages/react/src/radio-group/RadioGroup.tsx:15`

`floating-ui-react`:

- Exactly one function: `contains`, for the blur-relatedTarget guard
  (`packages/react/src/radio-group/RadioGroup.tsx:7,240`)

Cross-component contract:

- `radio/root/RadioRoot.tsx` is the sole consumer of this unit's context (import at
  `packages/react/src/radio/root/RadioRoot.tsx:24`), and `radio/indicator/RadioIndicator.tsx`
  consumes the state secondhand via `RadioRootContext`. Any Rust/Leptos replacement must reproduce
  this provider/consumer contract (checked-state derivation, gated commit, hidden-input
  registration, auto-select arm flag) and the composite item/root protocol, not just the group's own
  rendering.

Not used: portal machinery, `use-render`, and direct `useRenderElement` calls are absent from this
unit.

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit loop:

1. `enableHomeAndEndKeys={false}` (`packages/react/src/radio-group/RadioGroup.tsx:271`) makes
   Home/End deliberately inert. No test in this unit's suite presses Home or End, so the
   "must do nothing" contract is unpinned.
2. Modifier asymmetry. Only `SHIFT` is exempted from the composite modifier guard
   (`packages/react/src/radio-group/RadioGroup.tsx:23,272`;
   `packages/react/src/internals/composite/root/useCompositeRoot.ts:367-377`), so Ctrl/Alt/Meta+Arrow
   should NOT navigate — untested. Additionally, `onKeyDownCapture`
   (`packages/react/src/radio-group/RadioGroup.tsx:249-254`) runs before that guard and marks
   `touched`/`focused` on any Arrow keydown, so e.g. Ctrl+Arrow arms the flag (and via the child's
   focus auto-select could select on the next focus) without navigation actually occurring —
   untested interaction.
3. `aria-required` is rendered on the group root when `required`
   (`packages/react/src/radio-group/RadioGroup.tsx:232`), but no test in
   `RadioGroup.test.tsx` asserts it and behavior.md's "Accessibility" section does not list it
   (its `required` coverage is `data-required` and native validation).
4. inputRef/registry separation rationale. The in-source comment
   (`packages/react/src/radio-group/RadioGroup.tsx:104-107`) documents a failure mode — a stale,
   unmounted radio written into `validation.inputRef` would become the Field's fallback once the
   registry empties and keep blocking submission — that the design prevents but no test exercises.
5. No-Form fallback in `getFormValue`: when `elementRef.current` is null it returns the raw logical
   `checkedValue ?? null` unfiltered (`packages/react/src/radio-group/RadioGroup.tsx:160-163`),
   observable only through a Field `validate`'s `formValues` projection when the group is in a
   Field but not a Form. All behavior.md validation tests run inside a `Form`.
6. Registration parameter choices: `useRegisterFieldControl` forwards the raw `nameProp` (not the
   field-resolved `name`) and gates only on `!disabled`
   (`packages/react/src/radio-group/RadioGroup.tsx:174`) — so an unnamed group still registers a
   field entry (with `name: undefined`) and a disabled group unregisters entirely. No test isolates
   either combination in the form's fields map (checkbox-group, by contrast, also gates on having a
   field name).
7. id duality: the generated `base-ui-*` id (`packages/react/src/radio-group/RadioGroup.tsx:70`)
   feeds only field registration (line 174); the rendered root carries `idProp` only (line 230), so
   the group root gets no `id` attribute unless the user passes one. No test asserts the absence of
   an auto-generated id (behavior.md only covers explicit `id` forwarding).
8. The change-event reason type admits only `'none'`
   (`packages/react/src/radio-group/RadioGroup.tsx:341`); no test asserts
   `eventDetails.reason`, and the suite never imports `REASONS`.
9. `RadioGroup.spec.tsx` is type-only coverage (generic `Value` inference from narrow literal
   arrays, explicit generics, and `null` handling, `packages/react/src/radio-group/RadioGroup.spec.tsx:1-57`)
   with no runtime counterpart in the suite — the null-typed `value`/`defaultValue` flows
   (`packages/react/src/radio-group/RadioGroup.spec.tsx:39-57`) are only exercised at runtime via
   ordinary controlled tests.
