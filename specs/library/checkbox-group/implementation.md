# checkbox-group — implementation spec

Companion to `behavior.md` (same directory), which documents WHAT happens; this file explains WHY/HOW
from the non-test source files:

- `packages/react/src/checkbox-group/CheckboxGroup.tsx`
- `packages/react/src/checkbox-group/CheckboxGroupContext.ts`
- `packages/react/src/checkbox-group/CheckboxGroupDataAttributes.ts`
- `packages/react/src/checkbox-group/index.ts`
- `packages/react/src/checkbox-group/useCheckboxGroupParent.ts`

The unit's `TODO.md` entry (`library: checkbox-group`, `TODO.md:343-349`) has no `wraps-external:`
field — behavior is fully internal to the repo; there is no third-party package whose internals this
spec delegates to.

## State machine / hooks used

### In `CheckboxGroup.tsx`

- `useControlled` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:62-67`. The controlled/
  uncontrolled duality of behavior.md's "State model" lives here: `externalValue` (the `value` prop)
  vs `defaultValue`. The nullish normalization at `packages/react/src/checkbox-group/CheckboxGroup.tsx:60`
  (`defaultValueProp ?? EMPTY_ARRAY`) is what makes `defaultValue={null}` behave as an empty array
  without crashing.
- `useStableCallback` wrapping `setValue` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:69-79`.
  This is the single commit gate for every value change in the unit: it invokes `onValueChange`
  first, then only unwraps into `useControlled`'s setter if `eventDetails.isCanceled` is false. All
  cancel semantics in behavior.md ("State model" → Cancellation; "Events" → both child and parent
  cancel bullets) funnel through this one check — neither child nor parent code writes state directly.
- `useCheckboxGroupParent` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:81-85`, wired with
  `allValues`, the current `value`, and the gated `setValue` as `onValueChange`. Passing the gated
  setter (not the raw prop) is why a parent checkbox's `cancel()` behaves identically to a child's.
- `useLabelableId({ id: null })` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:89`. Passing
  `id: null` registers *no* control id in the labelable scope, so `Field.Label` omits `htmlFor` and
  the group is instead named by `aria-labelledby` (the parameter's documented contract in
  `packages/react/src/internals/labelable-provider/useLabelableId.ts:86-89`). This is the mechanism
  behind behavior.md's "Accessibility" bullet about a sibling `Field.Label` labeling the group.
- `useBaseUiId(idProp)` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:91` (prefixing wrapper
  at `packages/react/src/internals/useBaseUiId.ts:9-11`).
- Getter-based `controlRef` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:94-101`. A
  `useMemo`'d ref object whose `.current` getter resolves live through `validation.getInputControl()`,
  i.e. the field-validation registry's "representative input" pick (first eligible invalid input,
  else first eligible — `packages/react/src/field/root/useFieldValidation.ts:51-66,118-121`). The
  group owns no input itself; this lazy ref is how Form error-focusing (behavior.md "Focus
  management") lands on the right child checkbox at focus time instead of registration time.
- `getFormValue` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:103-121`. The group's
  form-value projection override: it intersects the logical `value` with the set of successful
  native inputs drawn from `validation.registeredInputs` (hidden checkbox inputs registered by
  mounted children), keeping an entry only when `registration.value !== undefined && input.checked &&
  isEligibleInput(input, formElement)` and the logical value contains it. This single function
  produces the split documented in behavior.md's "Field/Form value projection semantics": validation
  sees the full logical array, submission sees only selected+enabled+mounted+same-form children.
  Unmounted children drop out because child unmount deletes their registry entry; exclusion of
  *parent* checkboxes is enforced child-side (the parent checkbox registers no input) — the group's
  filter only ever sees what children registered.
- `useRegisterFieldControl(controlRef, id, value, getFormValue, !!fieldName && !disabled, fieldName)`
  — `packages/react/src/checkbox-group/CheckboxGroup.tsx:123`. Registers the group as the field's
  control with `getFormValue` as the value-getter override. The `enabled` flag suppresses
  registration entirely for unnamed or disabled groups (the hook then registers `undefined` —
  `packages/react/src/internals/field-register-control/useRegisterFieldControl.ts:21-38`, which also
  documents why re-registration happens in place: to keep the form's field ordering stable).
- `useIsoLayoutEffect` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:125-127`:
  `setFilled(value.length > 0)`, the direct source of behavior.md's `data-filled` rule (any non-empty
  logical value counts, even without a matching rendered checkbox, because it reads `value`, not the
  registry).
- `useValueChanged(value, …)` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:129-141` (utility
  at `packages/react/src/internals/useValueChanged.ts:6-17`, a layout-effect "previous value"
  watcher). On each committed change it: (1) calls `clearErrors(fieldName)` from `useFormContext`,
  (2) recomputes dirty by comparing `value` against `validityData.initialValue` with
  `areArraysEqual` (source of behavior.md's `data-dirty` appear/disappear rule), and (3) calls
  `validation.change(value)`, which routes into the field-validation boundary/debounce machinery
  (`packages/react/src/field/root/useFieldValidation.ts:349-365`) — the source of the
  inputless-group validation timing rules in behavior.md's "Edge cases".
- `useRenderElement('div', …)` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:161-174`. The
  shared element renderer: merges the base props (`id`, `role: 'group'`, `aria-labelledby: labelId`)
  with user `elementProps` and `getDescriptionProps` (appending `Field.Description` ids to
  `aria-describedby` — behavior.md "Accessibility"), and applies `stateAttributesMapping:
  fieldValidityMapping` (`packages/react/src/internals/field-constants/constants.ts:34`) so field
  validity state renders as the `data-*` attributes behavior.md's `data-dirty`/`data-filled` bullets
  observe. The state object it maps from is `{ ...fieldState, disabled }` at
  `packages/react/src/checkbox-group/CheckboxGroup.tsx:143-146`.

### In `useCheckboxGroupParent.ts`

- `uncontrolledStateRef` — `packages/react/src/checkbox-group/useCheckboxGroupParent.ts:13`, seeded
  with the initial `value`. Only child commits write it
  (`packages/react/src/checkbox-group/useCheckboxGroupParent.ts:122-124`, gated on
  `!eventDetails.isCanceled`); parent clicks only read it. It therefore serves as the
  "pre-'all'-cycle snapshot" that behavior.md's parent-toggle-cycle bullet observes being restored.
- `disabledStatesRef` — `packages/react/src/checkbox-group/useCheckboxGroupParent.ts:14`, a
  value→disabled `Map` that children report into; the parent toggle reads it to compute its target
  sets (see below).
- `status` state — `packages/react/src/checkbox-group/useCheckboxGroupParent.ts:16`:
  `'on' | 'off' | 'mixed'`. This is the parent checkbox's *toggle-phase* machine, distinct from the
  derived props `checked`/`indeterminate`
  (`packages/react/src/checkbox-group/useCheckboxGroupParent.ts:24-25`), which are computed purely
  from `value.length` vs `allValues.length` and feed the parent's rendered `aria-checked`. The phase
  machine in `onCheckedChange` (`packages/react/src/checkbox-group/useCheckboxGroupParent.ts:88-103`):
  - If the current state is a plain all-or-nothing set (`allOnOrOff`,
    `packages/react/src/checkbox-group/useCheckboxGroupParent.ts:76-86`), the click is a simple flip:
    full → `none`, otherwise → `all`. `none`/`all` are disabled-aware sets
    (`packages/react/src/checkbox-group/useCheckboxGroupParent.ts:66-74`): `all` includes every
    non-disabled value plus disabled-but-checked values (which can't change); `none` keeps only
    disabled-but-checked values. This is exactly the mechanism behind behavior.md's "Disabled
    children are not toggled by the parent" bullets.
  - Otherwise (a genuinely mixed set), the phase advances `mixed → on → off` with payloads
    `all → none → uncontrolledStateRef` respectively — the third click restoring the pre-cycle
    snapshot. A canceled click skips the `setStatus`
    (`packages/react/src/checkbox-group/useCheckboxGroupParent.ts:101-103`), so a canceled attempt
    stays in its phase and the next click retries it (behavior.md's canceled-parent-cycle bullet).
- `childIdsState` — `packages/react/src/checkbox-group/useCheckboxGroupParent.ts:20-22`: a state
  object wrapping a *mutable* `Map<string, readonly string[]>` (child value → registered element
  ids). The in-source comment gives both rationales: a `Map` (not a plain object) so consumer values
  like `'constructor'` can't collide with `Object.prototype` (behavior.md's prototype-key bullet),
  and wrapper-replacement-only updates so re-renders don't clone a growing registry.
  `registerChildId` (`packages/react/src/checkbox-group/useCheckboxGroupParent.ts:29-51`) dedupes
  per value, supports multiple ids per value, and returns an unregister closure that drops unmounted
  children — the full mechanism behind behavior.md's `aria-controls` bullets (registered children
  only, custom ids preserved, duplicates both listed, unmounts dropped).
- `getParentProps` — `packages/react/src/checkbox-group/useCheckboxGroupParent.ts:53-107`. Returns
  `indeterminate`, `checked`, `aria-controls` (flatMap of registry ids for `allValues`, `undefined`
  when empty — hence no attribute during SSR before children register), and the parent's own
  `onCheckedChange`. Because these props are attached only to the parent checkbox, a parent click
  calls only the parent's `onCheckedChange` and never a child's (behavior.md "Events").
- `getChildProps(childValue)` — `packages/react/src/checkbox-group/useCheckboxGroupParent.ts:109-129`.
  Per-child `checked: value.includes(childValue)` (behavior.md's `aria-checked` 'true'/'false'
  per-child rule) and an `onCheckedChange` that builds the next array by push/splice, forwards to the
  group's gated `setValue`, and on non-canceled commits updates `uncontrolledStateRef` and resets the
  phase to `'mixed'`. Resetting on child clicks is what makes the next parent click take the
  mixed→all transition regardless of how the mixed state arose.
- The hook returns a memoized handle (`packages/react/src/checkbox-group/useCheckboxGroupParent.ts:131-139`)
  containing `getParentProps`, `getChildProps`, `registerChildId`, and `disabledStatesRef`; that whole
  handle flows into context as `parent`.

## Context providers/consumers

- `CheckboxGroupContext.Provider` wraps the single rendered root element
  (`packages/react/src/checkbox-group/CheckboxGroup.tsx:176-178`); the context value is memoized at
  `packages/react/src/checkbox-group/CheckboxGroup.tsx:148-159`.
- Context shape — `packages/react/src/checkbox-group/CheckboxGroupContext.ts:9-24`: `value`,
  `setValue` (the gated committer), `allValues`, `parent` (the entire `useCheckboxGroupParent`
  return), `disabled` (group ∧ field), `validation` (the `UseFieldValidationReturnValue` handle from
  the surrounding Field), and `registerControlId`. The last is re-exported from the labelable scope
  with an explicit contract comment
  (`packages/react/src/checkbox-group/CheckboxGroupContext.ts:19-23`): a checkbox that observes the
  same function identity knows it shares the group's labelable scope and therefore is not the field's
  control.
- The unit exports no subcomponents; its only in-tree consumer is `Checkbox.Root`
  (`packages/react/src/checkbox/root/CheckboxRoot.tsx:89` calls `useCheckboxGroupContext()`), which
  uses `getChildProps` for its checked state, `getParentProps` for parent-checkbox rendering,
  `registerChildId`/`disabledStatesRef` to report its rendered id and disabled state, and
  `validation` to register its hidden native input into the same registry the group's
  `getFormValue`/`controlRef` read. This context shape *is* the group↔checkbox contract.
- Upstream contexts consumed by the group itself: `useFieldRootContext`
  (`packages/react/src/checkbox-group/CheckboxGroup.tsx:47-55` — field disabled/name/state/validation
  handles, `setFilled`/`setDirty`, `validityData`), `useLabelableContext`
  (`packages/react/src/checkbox-group/CheckboxGroup.tsx:56` — `labelId`, `registerControlId`,
  `getDescriptionProps`), and `useFormContext`
  (`packages/react/src/checkbox-group/CheckboxGroup.tsx:57` — `clearErrors` and the form
  `elementRef`).

## DOM/portal strategy and why

- The group renders one plain `div` via `useRenderElement` with `role: 'group'` and
  `aria-labelledby` (`packages/react/src/checkbox-group/CheckboxGroup.tsx:161-174`). No portal is
  used anywhere in this unit's source — the component is deliberately portal-inert.
- The unit owns no form control. Everything a Form needs from it flows through two indirections:
  (1) the lazy `controlRef` resolving to a child-registered representative input
  (`packages/react/src/checkbox-group/CheckboxGroup.tsx:94-101`), and (2) `getFormValue`
  (`packages/react/src/checkbox-group/CheckboxGroup.tsx:103-121`) reading the child-maintained
  `validation.registeredInputs` registry. Because Field registration is context-driven rather than
  DOM-driven, behavior.md's portal bullets (portaled checkboxes participating in submission,
  `form`-attribute opt-outs, fieldset-disabled exclusion) need no group-side code — eligibility is
  decided by `isEligibleInput`
  (`packages/react/src/field/root/useFieldValidation.ts:32-44`, whose doc comment states the
  context-crosses-portals policy the group relies on).
- The group is the field's *labeling* boundary but not its *control*: `useLabelableId({ id: null })`
  plus `aria-labelledby: labelId` (`packages/react/src/checkbox-group/CheckboxGroup.tsx:89,168`) make
  the group element itself the accessible name holder, while per-checkbox ids and label associations
  remain the child unit's concern.

## Dependencies on other Base UI internals

Shared `@base-ui/utils` hooks:

- `useControlled` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:3`
- `useIsoLayoutEffect` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:4`
- `useStableCallback` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:5`;
  `packages/react/src/checkbox-group/useCheckboxGroupParent.ts:3`
- `EMPTY_ARRAY` from `@base-ui/utils/empty` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:6`;
  `packages/react/src/checkbox-group/useCheckboxGroupParent.ts:4`
- `areArraysEqual` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:7`

`internals/`:

- `internals/useBaseUiId` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:8`
- `internals/useRenderElement` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:9` (shared
  element renderer: prop merging, state→data-attribute mapping)
- `internals/field-root-context/FieldRootContext` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:13`
- `internals/field-register-control/useRegisterFieldControl` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:14`
- `internals/labelable-provider/LabelableContext` + `useLabelableId` —
  `packages/react/src/checkbox-group/CheckboxGroup.tsx:15-16`;
  `packages/react/src/checkbox-group/CheckboxGroupContext.ts:7`
- `internals/field-constants/constants` (`fieldValidityMapping`) —
  `packages/react/src/checkbox-group/CheckboxGroup.tsx:18`
- `internals/createBaseUIEventDetails` + `internals/reasons` (change-event details/`REASONS.none`) —
  `packages/react/src/checkbox-group/CheckboxGroup.tsx:20-21`;
  `packages/react/src/checkbox-group/CheckboxGroupContext.ts:5-6`;
  `packages/react/src/checkbox-group/useCheckboxGroupParent.ts:5-6`
- `internals/form-context/FormContext` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:22`
- `internals/useValueChanged` — `packages/react/src/checkbox-group/CheckboxGroup.tsx:23`

Field subsystem (the deepest coupling):

- `field/root/useFieldValidation` — `isEligibleInput` imported directly
  (`packages/react/src/checkbox-group/CheckboxGroup.tsx:12`), and the `validation` handle typed from
  it and shared through context (`packages/react/src/checkbox-group/CheckboxGroupContext.ts:3,18`)
- `field/root/FieldRoot` — `FieldRootState` type only
  (`packages/react/src/checkbox-group/CheckboxGroup.tsx:11`)

Cross-component contract:

- `checkbox/root/CheckboxRoot.tsx` is the sole consumer of this unit's context (import at
  `packages/react/src/checkbox/root/CheckboxRoot.tsx:29`, read at line 89). Any Rust/Leptos
  replacement must reproduce this provider/consumer contract, not just the group's own rendering.

Not used: `floating-ui-react`, `use-render`, and portal machinery are absent from this unit.

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit loop:

1. `data-disabled` on the group root. The root state includes `disabled`
   (`packages/react/src/checkbox-group/CheckboxGroup.tsx:143-146`) and
   `packages/react/src/checkbox-group/CheckboxGroupDataAttributes.ts:4` declares
   `data-disabled` as the unit's only custom data attribute, but no test in this unit's suite
   asserts `data-disabled` on the group root (tests assert `aria-disabled` on children only —
   behavior.md "Accessibility").
2. No-Form fallback in `getFormValue`. When `elementRef.current` is null (group used with `Field`
   but no `Form`), `getFormValue` returns the raw logical `value` unfiltered
   (`packages/react/src/checkbox-group/CheckboxGroup.tsx:104-107`). No test covers value projection
   in the Field-without-Form configuration.
3. Group-level registration suppression. `useRegisterFieldControl` is disabled (registers
   `undefined`) for unnamed or group-disabled groups
   (`packages/react/src/checkbox-group/CheckboxGroup.tsx:123`). No test isolates this field-level
   unregistration from child-level exclusion via `isEligibleInput` — e.g. that a disabled named group
   disappears from the form's fields map entirely.
4. Form-error clearing on value change. `clearErrors(fieldName)` runs on every committed change
   (`packages/react/src/checkbox-group/CheckboxGroup.tsx:130-132`). Related tests exercise
   validation-driven error clearing (behavior.md "Edge cases" inputless-group bullets, and
   `CheckboxGroup.test.tsx:610,621`), but no test asserts a Form-level error (from the `errors` prop)
   disappearing purely because the group's value changed.
5. Uncheck of an absent value in `getChildProps`. `newValue.splice(newValue.indexOf(childValue), 1)`
   (`packages/react/src/checkbox-group/useCheckboxGroupParent.ts:117`) would remove the last element
   if `indexOf` returns −1. Correctness rests on the invariant that child checked state is always
   derived from the group value (`packages/react/src/checkbox-group/useCheckboxGroupParent.ts:111`),
   making a stale uncheck unreachable — an invariant no test asserts (e.g. under concurrent
   controlled-value resets mid-event).
6. Parent-cycle snapshot without prior child clicks. `uncontrolledStateRef` is seeded with the
   initial `value` (`packages/react/src/checkbox-group/useCheckboxGroupParent.ts:13`), so the
   "restore" phase of the toggle cycle can fire with a snapshot that came from props rather than
   from a child commit. behavior.md's restore test reaches that phase only through child clicks
   (`behavior.md`, "State model" → parent-checkbox tri-state, third bullet); the props-seeded path
   is untested.
7. `value`-array identity in `setValue`. `setValue` forwards the proposed array to `onValueChange`
   and only then commits it (`packages/react/src/checkbox-group/CheckboxGroup.tsx:69-79`), so a
   consumer mutating the array inside the handler would observe the mutation in state. No test pins
   this aliasing behavior (all tests pass fresh arrays built by `getChildProps`/`getParentProps`).
