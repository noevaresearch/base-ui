# Field — implementation spec (Stage 2: implementation mining)

Companion to the behavior spec (see `specs/library/field/behavior.md`, treated as ground truth for WHAT).
References to it below use its section names in quotes. Citations target the unit's non-test source
files, plus the internals they call into (those are needed for the dependencies section). This unit's
`TODO.md` entry (TODO.md:383-389) has no `wraps-external:` field, so no external-package delegation is
being replaced here.

Mined from (non-test sources under `packages/react/src/field`):

- root: `FieldRoot.tsx`, `useFieldValidation.ts`, `FieldRootDataAttributes.ts`
- control: `FieldControl.tsx`, `FieldControlDataAttributes.ts` (+ `FieldControl.spec.tsx`, see "Anything in source not explained by any test")
- label: `FieldLabel.tsx`, `FieldLabelDataAttributes.ts`
- description: `FieldDescription.tsx`, `FieldDescriptionDataAttributes.ts`
- error: `FieldError.tsx`, `FieldErrorDataAttributes.ts`
- item: `FieldItem.tsx`, `FieldItemContext.ts`, `FieldItemDataAttributes.ts`
- validity: `FieldValidity.tsx`; utils: `getCombinedFieldValidityData.ts`; barrels: `index.ts`, `index.parts.ts`

## State machine / hooks used

### Root-level state (`FieldRoot`)

- Four independent booleans — `touchedState`, `dirtyState`, `filled`, `focused` — each with a setter
  shared through context: `packages/react/src/field/root/FieldRoot.tsx:50-53`. Controlled overrides are
  resolved at render (`dirtyProp ?? dirtyState`, same for touched): `packages/react/src/field/root/FieldRoot.tsx:55-56`.
- When `dirty` is controlled, the internal setter becomes a no-op and a `useIsoLayoutEffect` mirrors the
  prop into `markedDirtyRef` so callback-time readers see the controlled value:
  `packages/react/src/field/root/FieldRoot.tsx:63-85`. `markedDirtyRef` (set once on the first dirty
  transition, `packages/react/src/field/root/FieldRoot.tsx:74-76`) is the input to the `valueMissing`
  suppression rule in the validator (see below) — this is the mechanism behind "State model": dirty-gated
  `valueMissing` and the controlled-dirty tests in "Public API surface".
- Name resolution: `effectiveName = name ?? registeredFieldName` (`packages/react/src/field/root/FieldRoot.tsx:59-61`);
  the fallback is the *registered control's* own name, recorded only when the root has none
  (`packages/react/src/internals/field-register-control/useFieldControlRegistration.ts:111,157-159`).
  Root `name` wins over the control's (`packages/react/src/internals/field-register-control/useFieldControlRegistration.ts:73`,
  `packages/react/src/field/control/FieldControl.tsx:66`); see the discrepancy flag at the end.
- The whole validity model is one `useState` object seeded with `DEFAULT_VALIDITY_STATE`, whose
  `valid: null` encodes the neutral phase ("State model": three-phase shape):
  `packages/react/src/field/root/FieldRoot.tsx:98-104`, `packages/react/src/internals/field-constants/constants.ts:4-16`.
- App-controlled invalidity: `invalid = invalidProp === true || hasFormError` where `formError` is looked
  up in the `<Form>` errors record by `effectiveName`: `packages/react/src/field/root/FieldRoot.tsx:93-96`.
  The single derived `valid` implements both disabled rules of the "State model"/"Accessibility" sections —
  app-controlled invalidity survives `disabled`, computed validity is suppressed:
  `packages/react/src/field/root/FieldRoot.tsx:109`.
- State → attributes: the memoized state (`packages/react/src/field/root/FieldRoot.tsx:111-121`) is rendered
  through `useRenderElement` with `stateAttributesMapping: fieldValidityMapping`
  (`packages/react/src/field/root/FieldRoot.tsx:186-191`). The mapping folds the tri-state `valid` into
  `data-valid` / `data-invalid` and emits nothing for `null`
  (`packages/react/src/internals/field-constants/constants.ts:34-43`); every other truthy state key becomes
  a generic `data-<key>` attribute (`packages/react/src/internals/getStateAttributesProps.ts:24-28`).
  The `Field*DataAttributes.ts` files are pure string constants for these hooks; `FieldError` additionally
  re-exports the transition-status hooks (`packages/react/src/field/error/FieldErrorDataAttributes.ts:31-38`,
  `packages/react/src/internals/stateAttributesMapping.ts:10-20`).

### The validation engine (`useFieldValidation`)

Instantiated once by the root (`packages/react/src/field/root/FieldRoot.tsx:123-134`) and shared with all
parts (and with form-aware controls) through context. Its `commit` closure is the field's real state machine
(`packages/react/src/field/root/useFieldValidation.ts:123-347`):

- Epoch guard: `validationCommitIdRef` is bumped on every `change`/`commit`; an awaited async result whose
  epoch is stale is discarded — the mechanism for "Edge cases": stale async results and dropped in-flight
  validations (`packages/react/src/field/root/useFieldValidation.ts:99,124-125,323-325`).
- Entry points: `change(value, cancelPending?)` clears the timer, honors cancellation, and either debounces
  `commit` (only while validating on change *and* the value is non-empty) or commits immediately with
  `revalidate = !validateOnChange`: `packages/react/src/field/root/useFieldValidation.ts:349-365`.
- The `revalidate` branch is "Revalidation on change (once invalid)": a resolved `valueMissing` publishes
  immediately (validity + new value), while any other native error defers to the next blur/submit:
  `packages/react/src/field/root/useFieldValidation.ts:244-276`.
- Representative input election: a field can own several inputs (checkbox/radio groups). Inputs register into
  a `Map` (`packages/react/src/field/root/useFieldValidation.ts:98,108-116`); the elected representative is
  the first eligible *currently-invalid* input in mount order, else the first eligible one
  (`packages/react/src/field/root/useFieldValidation.ts:51-66,228-232`). The shared `inputRef` is only a
  fallback when nothing is registered.
- Native verdict: `getState` copies `ValidityState` flags, suppressing `valueMissing` while the field is not
  dirty (`packages/react/src/field/root/useFieldValidation.ts:194-223`, suppression at 218-221). Controls
  barred from validation (`willValidate === false`) get a synthetic all-false state
  (`packages/react/src/field/root/useFieldValidation.ts:238-242` with `makeState` at 68-70).
- Custom-validity ownership (the "Edge cases": `setCustomValidity` interplay): Base UI's message is written
  to the real input via `setCustomValidity`, remembering the message it displaced; clearing restores the
  displaced message only if the control still shows Base UI's own (or is barred, where `validationMessage`
  is hidden): `packages/react/src/field/root/useFieldValidation.ts:101-103,167-182`. Message text is
  normalized `\r\n`/`\r` → `\n` at 170.
- Async pending rules ("State model": async validation pending): while a validator is in flight the field
  publishes neutral validity, except that a previous custom error is kept outside `'onSubmit'` mode and
  fresh native failures are always kept: `packages/react/src/field/root/useFieldValidation.ts:303-317`. A
  *rejected* validator never publishes, so the previously published error stays and keeps blocking
  submission: `packages/react/src/field/root/useFieldValidation.ts:319-325`.
- Native/external errors take precedence outside onChange validation — the custom validator is only invoked
  when native constraints pass or the mode validates on change:
  `packages/react/src/field/root/useFieldValidation.ts:286-289`.
- Form-values projection: `validate(value, formValues)` builds `formValues` from every registered field in
  the `<Form>` registry ("Events": `validate(value, formValues)`):
  `packages/react/src/field/root/useFieldValidation.ts:293-300`.
- Publication is dual: React state via `setValidityData` for rendering, plus a write of the *combined*
  validity (field validity merged with external `invalid`) into the `<Form>` registry entry:
  `packages/react/src/field/root/useFieldValidation.ts:127-150,184-192`, combining helper
  `packages/react/src/field/utils/getCombinedFieldValidityData.ts:7-18`.
- `getValidationProps` is the only source of `aria-invalid` — applied when computed-invalid and neither the
  root nor the receiving part is disabled; it also merges `aria-describedby` through the labelable scope:
  `packages/react/src/field/root/useFieldValidation.ts:367-376`.

### Hook inventory (name → call site)

- `useControlled` — value model: `packages/react/src/field/control/FieldControl.tsx:77-82`; the controlled
  branch is serialized to a string for dirty/filled comparisons
  (`packages/react/src/field/control/FieldControl.tsx:84-87`).
- `useValueChanged` — makes *programmatic* controlled-value changes behave like user input (clears form
  errors, recomputes dirty, triggers `validation.change`):
  `packages/react/src/field/control/FieldControl.tsx:107-116`, internals
  `packages/react/src/internals/useValueChanged.ts:6-17`. This is the mechanism for "State model":
  external controlled value changes sync state and trigger validation.
- `useRegisterFieldControl` — control-side registration (see Context): `packages/react/src/field/control/FieldControl.tsx:91-98`.
- `useFieldControlRegistration` — root-side registration + imperative validate (see Context):
  `packages/react/src/field/root/FieldRoot.tsx:136-146`.
- `useIsoLayoutEffect` — dirty-prop mirroring (`packages/react/src/field/root/FieldRoot.tsx:63-67`),
  filled-on-mount sync (`packages/react/src/field/control/FieldControl.tsx:100-105`), autoFocus/focus-state
  hydration (`packages/react/src/field/control/FieldControl.tsx:121-125`), description/error message-id
  registration (`packages/react/src/field/description/FieldDescription.tsx:36-46`,
  `packages/react/src/field/error/FieldError.tsx:58-68`).
- `useStableCallback` — `validate`/`setDirty`/`setTouched`/`shouldValidateOnChange`
  (`packages/react/src/field/root/FieldRoot.tsx:46,69,80,87`), `commit`/`change`/`getInputControl`
  (`packages/react/src/field/root/useFieldValidation.ts:118,123,349`).
- `useTimeout` — validation debounce (`packages/react/src/field/root/useFieldValidation.ts:96`) and the
  Enter-key implicit-submission fallback (`packages/react/src/field/control/FieldControl.tsx:119,198-202`).
- `useImperativeHandle` — `actionsRef.validate` bound to the registration wrapper
  (`packages/react/src/field/root/FieldRoot.tsx:148-150`).
- `useLabelableId` — control id generation + association registration
  (`packages/react/src/field/control/FieldControl.tsx:75`).
- `useTransitionStatus` + `useOpenChangeComplete` — FieldError mount/exit animation ("DOM structure": the
  `data-starting-style`/`data-ending-style` hooks): `packages/react/src/field/error/FieldError.tsx:56,102-110`.
  Render-phase message keying keeps the last message as `children` while exiting
  (`packages/react/src/field/error/FieldError.tsx:95-100,123`) and `enabled: mounted` suppresses rendering
  only after the exit animation completes (`packages/react/src/field/error/FieldError.tsx:128-135`).

### Control event wiring (the DOM→state machine boundary)

All control behavior lives in one `useRenderElement` props list
(`packages/react/src/field/control/FieldControl.tsx:127-213`):

- `onChange`: fires `onValueChange(inputValue, details)` with a cancelable details object
  (`packages/react/src/field/control/FieldControl.tsx:139-159`, details via
  `packages/react/src/internals/createBaseUIEventDetails.ts:118-149` and `REASONS.none` from
  `packages/react/src/internals/reasons.ts:1-5`). A controlled component returns early after the callback —
  state syncs from the `value` prop via `useValueChanged` instead, so a value the consumer rejects or
  rewrites never reaches field state (146-148). Dirty is updated *before* `validation.change` because the
  validator reads `markedDirtyRef` (150-152). If the native `input` event was `preventDefault`-ed or
  `details.cancel()` was called, dirty/filled still update but validation and error-clearing are skipped
  (154-158) — the "Events": native input-event-prevention semantics.
- `onBlur`: marks touched + unfocused; in `'onBlur'` mode commits the DOM value, and for controlled inputs
  re-commits one microtask later if a blur handler normalized the value (skipping a rewrite back to the
  initial value): `packages/react/src/field/control/FieldControl.tsx:163-187`.
- `onKeyDown` (Enter): marks touched and commits; if the input belongs to the surrounding `<Form>`, an
  implicit submission may run after keydown, so validation is scheduled in a 0ms timeout and skipped when
  the form's submit count already advanced: `packages/react/src/field/control/FieldControl.tsx:188-207`.
- `getValidationProps(disabled, props)` is applied as the final props layer (210), injecting
  `aria-invalid`/`aria-describedby`.

## Context providers/consumers

Four contexts cross this unit's boundary:

- **`FieldRootContext`** — provided once per `Field.Root` (`packages/react/src/field/root/FieldRoot.tsx:152-184,193`;
  shape and NOOP default: `packages/react/src/internals/field-root-context/FieldRootContext.ts:12-63`).
  Payload: the derived state, `invalid`, resolved `name`, `validityData` + `setValidityData`, the four
  state setters, `validationMode`, `shouldValidateOnChange`, `registerFieldControl`, and the whole
  `validation` object (`commit`/`change`/`inputRef`/`registerInput`/`getValidationProps`).
  Consumers inside the unit: Control (`packages/react/src/field/control/FieldControl.tsx:51-62`), Label
  (`packages/react/src/field/label/FieldLabel.tsx:33`), Description
  (`packages/react/src/field/description/FieldDescription.tsx:27`), Error
  (`packages/react/src/field/error/FieldError.tsx:36`), Validity
  (`packages/react/src/field/validity/FieldValidity.tsx:17`), Item (`packages/react/src/field/item/FieldItem.tsx:29`).
  The hook takes an `optional` flag: Label/Description/Error/Validity/Item pass `false` and throw outside a
  Root, while Control uses the default and silently falls back to the NOOP context
  (`packages/react/src/internals/field-root-context/FieldRootContext.ts:65-75`).
  This context is also the *inbound* boundary for the form-aware controls (Checkbox, CheckboxGroup, Radio,
  RadioGroup, Select, NumberField, Slider, Switch — "Public API surface": those register into the field when
  nested); they are outside this unit and consume `registerFieldControl` + `validation` from it.
- **`LabelableContext` / `LabelableProvider`** — scoped label/description association. Two nesting scopes:
  one per `Field.Root` (`packages/react/src/field/root/FieldRoot.tsx:207-209`) and one per `Field.Item`
  (`packages/react/src/field/item/FieldItem.tsx:43-47`), which is how per-item labels associate with their
  own checkbox/radio ("Accessibility": Field.Item label association). The provider owns: the elected
  `controlId` (registration keeps the current selection across unmount/remount churn and preserves it while
  a hidden subtree keeps its DOM: `packages/react/src/internals/labelable-provider/LabelableProvider.tsx:14-60`),
  the `labelId`, `messageIds`, and `getDescriptionProps` which dedupes and merges user-provided
  `aria-describedby` (68-81). Consumers: Control reads `labelId` and registers its id via `useLabelableId`
  (`packages/react/src/field/control/FieldControl.tsx:73,75`; registration logic
  `packages/react/src/internals/labelable-provider/useLabelableId.ts:32-82`); Label renders `htmlFor` /
  click-focus through `useLabel` (`packages/react/src/field/label/FieldLabel.tsx:42-46`; hook
  `packages/react/src/internals/labelable-provider/useLabel.ts:10-77` — native branch returns
  `id`+`htmlFor`+`onMouseDown`, non-native branch returns `onClick`+`onPointerDown` that focuses the control
  by id); Description/Error push their ids into `messageIds`
  (`packages/react/src/field/description/FieldDescription.tsx:36-46`,
  `packages/react/src/field/error/FieldError.tsx:58-68`); the validator reads `controlId` and
  `getDescriptionProps` (`packages/react/src/field/root/useFieldValidation.ts:94,131,370`).
- **`FormContext`** — the `<Form>` bridge. Root reads `errors`/`validationMode`/`submitCountRef`
  (`packages/react/src/field/root/FieldRoot.tsx:26`); Control reads `clearErrors`/`elementRef`/`submitCountRef`
  (`packages/react/src/field/control/FieldControl.tsx:63`); the validator and registration hook read/write the
  `formRef.current.fields` registry (`packages/react/src/internals/form-context/FormContext.ts:13-28`);
  Field.Error reads `errors` directly for form-error-driven rendering
  (`packages/react/src/field/error/FieldError.tsx:39-43`). All reads are safe outside a `<Form>` because the
  context default is an inert registry (`packages/react/src/internals/form-context/FormContext.ts:33-50`).
- **`FieldItemContext`** — `{ disabled }` provided per `Field.Item` (`packages/react/src/field/item/FieldItemContext.ts:4-14`,
  provider at `packages/react/src/field/item/FieldItem.tsx:34,45`); Label and Description OR it into their
  disabled state so items inside a group render `data-disabled` without disabling siblings
  (`packages/react/src/field/label/FieldLabel.tsx:34,39`,
  `packages/react/src/field/description/FieldDescription.tsx:28,33`).

Additionally the root optionally reads **`FieldsetRootContext`** to inherit `<Fieldset.Root disabled>`
(`packages/react/src/field/root/FieldRoot.tsx:44,48`; context
`packages/react/src/fieldset/root/FieldsetRootContext.ts:10-22`).

Registration pair (how a control becomes "the" control of a field):

- Control side: `useRegisterFieldControl` re-registers in place on every change (delete+re-add would reorder
  the registry Map) and unregisters on unmount/disabled:
  `packages/react/src/internals/field-register-control/useRegisterFieldControl.ts:16-45`.
- Root side: `useFieldControlRegistration` owns ownership semantics — the initial dirty baseline is captured
  exactly once and belongs to the field, not to a control instance
  (`packages/react/src/internals/field-register-control/useFieldControlRegistration.ts:86-103`); a replaced
  control cancels the previous one's pending validation (`change(undefined, true)`, 150-153); the registry
  entry carries `getValue`/`name`/`controlRef`/`validityData`/`validate` (65-78,113-120); the imperative
  `validate` wrapper marks the field dirty before committing (53-63).

## DOM/portal strategy and why

- **No portals.** No part of the unit renders through a portal; every part renders inline via
  `useRenderElement` with the defaults `div`/`input`/`label`/`p`/`div`/`div`
  (`packages/react/src/field/root/FieldRoot.tsx:186-191`, `packages/react/src/field/control/FieldControl.tsx:127`,
  `packages/react/src/field/label/FieldLabel.tsx:78`, `packages/react/src/field/description/FieldDescription.tsx:48`,
  `packages/react/src/field/error/FieldError.tsx:117`, `packages/react/src/field/item/FieldItem.tsx:36`).
  This matches "DOM structure & portal behavior": nothing is portaled and nothing needs to be — Field has no
  floating layer.
- **Membership is context-driven, not DOM-position-driven.** Whether an input belongs to the surrounding
  `<Form>` is decided by `isEligibleInput` — excluded when `:disabled`, or explicitly associated with another
  form via the `form` attribute; a portaled input with no `form` association still participates because React
  context crosses portals: `packages/react/src/field/root/useFieldValidation.ts:25-44`. This is the deliberate
  design that lets a Field live inside a dialog/popup while its validation and submitted values stay attached
  to the form.
- **The DOM is used as constraint-validation state.** Custom messages are written to the real input with
  `element.setCustomValidity` and native messages are read back via `validity`/`validationMessage`
  (`packages/react/src/field/root/useFieldValidation.ts:167-182,283-289`), so the field interoperates with
  native form semantics (blocked submission, `:invalid` styling) instead of reimplementing them.
- **ARIA wiring is all id-based and hydration-aware:** the control renders `id` + `aria-labelledby: labelId`
  (`packages/react/src/field/control/FieldControl.tsx:132-137`); the label renders `htmlFor` only in native
  mode and suppresses it when the elected `controlId` is `null` (group controls named via `aria-labelledby`,
  "Accessibility": group naming — the null convention is documented at
  `packages/react/src/internals/labelable-provider/useLabelableId.ts:85-90`); descriptions/errors contribute
  to `aria-describedby` through `messageIds` (see Context section). The label id is only published after mount
  (`packages/react/src/utils/useRegisteredLabelId.ts:12-17`), which is why `aria-labelledby` never appears in
  SSR markup ("Accessibility": aria-labelledby only after hydration).
- **Style hooks are computed centrally** from the shared state object (see State machine section); parts get
  identical `data-*` sets because they all spread the same root state (`...fieldState`) — Control overrides
  only `disabled` (`packages/react/src/field/control/FieldControl.tsx:68-71`), Label/Description OR in the
  item's `disabled`.
- **The public barrel** assembles the namespace API (`Field.Root`, …) and re-exports types:
  `packages/react/src/field/index.parts.ts:1-9`, `packages/react/src/field/index.ts:1-9`.

## Dependencies on other Base UI internals

In-repo dependencies (import → purpose), excluding test files:

- `internals/useRenderElement` (`packages/react/src/internals/useRenderElement.tsx:22-48`) — every part's
  rendering, ref merging, state attributes, `render` prop evaluation.
- `internals/field-root-context` — the shared field context (public subpath export,
  `packages/react/package.json:79-81`).
- `internals/field-register-control` — `useFieldControlRegistration` (root) and `useRegisterFieldControl`
  (control); also a public subpath export (`packages/react/package.json:79-81`).
- `internals/labelable-provider` — `LabelableProvider`, `LabelableContext`, `useLabel`, `useLabelableId`;
  plus `utils/useRegisteredLabelId`.
- `internals/form-context` — `<Form>` bridge consumed by Root, Control, Error, and both field hooks.
- `internals/field-constants` — `DEFAULT_VALIDITY_STATE`, default state, `fieldValidityMapping` (public
  subpath export, `packages/react/package.json:79-81`).
- `internals/useValueChanged`, `internals/useTransitionStatus`, `internals/useOpenChangeComplete`,
  `internals/useBaseUiId`, `internals/getStateAttributesProps`, `internals/stateAttributesMapping`
  (transition-status attributes), `internals/createBaseUIEventDetails` + `internals/reasons`
  (onValueChange details), `internals/merge-props` (used inside `getValidationProps`,
  `packages/react/src/field/root/useFieldValidation.ts:369`), `internals/types` (`BaseUIComponentProps`,
  `HTMLProps`).
- `floating-ui-react/utils` — only the shadow-DOM-safe DOM helpers: FieldControl imports `activeElement`
  (`packages/react/src/field/control/FieldControl.tsx:21`), re-exported from
  `packages/react/src/floating-ui-react/utils/element.ts:3,8` (originating in `@base-ui/utils/shadowDom`).
- `fieldset/root/FieldsetRootContext` — optional disabled inheritance.
- `@base-ui/utils` (workspace package): `useControlled`, `useIsoLayoutEffect`, `useStableCallback`,
  `useTimeout`, `useRefWithInit` (`packages/react/src/field/root/useFieldValidation.ts:98`), `useId` (via `useBaseUiId`,
  `packages/react/src/internals/useBaseUiId.ts:9-11`), `useMergedRefs`/`getReactElementRef` (inside
  `useRenderElement`), `useAnimationFrame` (inside `useTransitionStatus`,
  `packages/react/src/internals/useTransitionStatus.ts:4`), `owner` (`ownerDocument`,
  `packages/react/src/field/control/FieldControl.tsx:5`), `error` (dev warnings:
  `packages/react/src/field/label/FieldLabel.tsx:3`), `warn` (via `useRenderElement`), `empty`
  (`EMPTY_OBJECT`), `safeReact` (`captureOwnerStack` in the label dev warning,
  `packages/react/src/field/label/FieldLabel.tsx:4,59,67`).
- `../../form` — type-only (`Form.Values`, `Form.ValidationMode`), no runtime dependency.
- `@floating-ui/utils/dom` — `isHTMLElement` inside `useLabel`
  (`packages/react/src/internals/labelable-provider/useLabel.ts:3`).

Direction of dependency: Field is a *leaf provider* for the form-aware controls — Checkbox, CheckboxGroup,
Radio, RadioGroup, Select, NumberField, Slider, Switch, Input, Combobox triggers, etc. consume
`FieldRootContext` (registration + validation API) and `LabelableContext`; none of them are imported by this
unit. Any port plan must treat Field's context contracts (not the components) as the shared dependency of
those units. `LabelableProvider` is likewise consumed by other components' labelable parts outside this unit.

`wraps-external:` — N/A (no delegation declared; confirmed in TODO.md:383-389 and restated by the behavior spec).

## Anything in source not explained by any test

Explicit gaps — no test in this unit's suite pins these; flagged for the golden-fixture and audit stages:

1. **Fieldset disabled inheritance.** `FieldRoot` merges `useFieldsetRootContext(true)?.disabled` into its
   disabled state (`packages/react/src/field/root/FieldRoot.tsx:44,48`), but no Field test renders a
   `<Fieldset.Root>` (grep over `packages/react/src/field` finds only the import). Whatever coverage exists
   lives in the fieldset unit, not here.
2. **Portaled-input participation and `form`-attribute opt-out.** The rules in `isEligibleInput` — portaled
   inputs still belong to the surrounding Form unless an explicit `form` attribute opts out
   (`packages/react/src/field/root/useFieldValidation.ts:32-44`) — are asserted by no Field test; the
   behavior spec itself marks portal behavior UNVERIFIED. Fixture stage should decide whether this is
   Field-level or Form-level behavior.
3. **Empty-value debounce bypass.** `change` only debounces when `value !== ''`
   (`packages/react/src/field/root/useFieldValidation.ts:358`): clearing a field with
   `validationDebounceTime` set validates immediately, while every debounce test
   ("Edge cases": debounce semantics) uses non-empty values, so the bypass is unobserved.
4. **Imperative `validate()` marks the field dirty.** `useFieldControlRegistration`'s `validate` sets
   `markedDirtyRef.current = true` before committing
   (`packages/react/src/internals/field-register-control/useFieldControlRegistration.ts:53-63`). The
   `actionsRef` test ("Public API surface": imperative handle) depends on this indirectly — a required,
   never-touched control would otherwise suppress `valueMissing` and render no error — but no test asserts
   `data-dirty` after `actionsRef.validate()`.
5. **behavior.md wording discrepancy (name precedence).** The behavior spec's "Public API surface" and
   "Edge cases" prose say `Field.Control`'s `name` "takes precedence" over `Field.Root`'s. The source says
   the opposite: the root name wins and the control name is a dynamic fallback
   (`packages/react/src/field/control/FieldControl.tsx:66`,
   `packages/react/src/internals/field-register-control/useFieldControlRegistration.ts:73,111-119`,
   JSDoc at `packages/react/src/field/root/FieldRoot.tsx:272-273`), matching the cited tests' own titles
   ("uses the Field.Control name *fallback* when the Field.Root name is removed"). The implementation spec
   defers to the source; the backward-looking audit should correct behavior.md's prose.
6. **`FieldControl.spec.tsx` is not a behavior test.** It is a type-only fixture (7 lines, no assertions)
   pinning that `ref` typing works when `render={<textarea />}` replaces the default `<input>`
   (`packages/react/src/field/control/FieldControl.spec.tsx:4-7`). It contributes no runtime behavior to
   mine and should be excluded from fixture generation.
