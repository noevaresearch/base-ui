# Input — implementation spec (Stage 2: implementation mining)

Unit: `library: input`. Mined from the unit's source files (`packages/react/src/input/index.ts`,
`packages/react/src/input/Input.tsx`, `packages/react/src/input/InputDataAttributes.ts`,
`packages/react/src/input/Input.spec.tsx`) plus the code those files delegate to. WHAT is
documented in behavior.md (referenced below by section name); this spec explains WHY/HOW.

The single most important structural fact: **Input is a pure delegation wrapper.** The component
is a `React.forwardRef` whose entire body is `return <Field.Control ref={forwardedRef} {...props} />`
`packages/react/src/input/Input.tsx:12-17`. It calls no hooks, reads no context, and owns no
state; `packages/react/src/input/index.ts:1-2` is a bare re-export of the component and its type
surface. Every prop, state bit, and DOM decision is produced by `Field.Control`
(`packages/react/src/field/control/FieldControl.tsx`), so the mining below walks that delegation
target while keeping Input's own files as the unit boundary.

## State machine / hooks used

There is no local state machine in this unit. All state lives in the Field.Root context, and the
control is a registered view over it. Hooks, each cited at its call site in
`packages/react/src/field/control/FieldControl.tsx` (Input itself is hookless,
`packages/react/src/input/Input.tsx:12-17`):

- `useFieldRootContext` `packages/react/src/field/control/FieldControl.tsx:51-62` — pulls the
  Field state machine: `state` (touched/dirty/filled/focused/valid/disabled), `name`,
  `disabled`, the `setTouched`/`setDirty`/`setFocused`/`setFilled` setters, `validityData`,
  `validationMode`, and the `validation` API.
- `useFormContext` `packages/react/src/field/control/FieldControl.tsx:63` — `clearErrors`,
  `formElementRef`, `submitCountRef`.
- Derived (no hook): `disabled = fieldDisabled || disabledProp`
  `packages/react/src/field/control/FieldControl.tsx:65`, `name = fieldName ?? nameProp`
  `packages/react/src/field/control/FieldControl.tsx:66`, and the merged state object
  `{ ...fieldState, disabled }` `packages/react/src/field/control/FieldControl.tsx:68-71`. That
  object is the component's entire "state" and the direct source of its `data-*` attributes
  (see DOM section).
- `useLabelableContext` / `useLabelableId` `packages/react/src/field/control/FieldControl.tsx:73-75`
  — `labelId` for `aria-labelledby` and a stable id that coordinates with Field.Label.
- `useControlled` `packages/react/src/field/control/FieldControl.tsx:77-82` with a manual
  `isControlled = valueProp !== undefined` check
  `packages/react/src/field/control/FieldControl.tsx:84-85`; the controlled value is serialized
  with `String()` because DOM values are always strings
  `packages/react/src/field/control/FieldControl.tsx:86-87`. In controlled mode the DOM value
  flows from the `value` prop (`...(isControlled ? { value } : { defaultValue })`
  `packages/react/src/field/control/FieldControl.tsx:138`); in uncontrolled mode the DOM owns
  the value and Field reads it back through `validation.inputRef` (`getValueFromInput`,
  `packages/react/src/field/control/FieldControl.tsx:89`).
- `useStableCallback` `packages/react/src/field/control/FieldControl.tsx:89` —
  `getValueFromInput` closure over `validation.inputRef`.
- `useRegisterFieldControl` `packages/react/src/field/control/FieldControl.tsx:91-98` —
  registers this control into Field.Root's fields registry (see Context section).
- `useIsoLayoutEffect` `packages/react/src/field/control/FieldControl.tsx:100-105` — syncs
  `filled` from the serialized value (or the live DOM value when uncontrolled) whenever it
  changes.
- `useValueChanged` `packages/react/src/field/control/FieldControl.tsx:107-116` (implementation
  `packages/react/src/internals/useValueChanged.ts:6-17`) — the *programmatic* value-change
  path: when the serialized value differs from the previous one, it calls `clearErrors(name)`,
  recomputes `dirty` against `validityData.initialValue`, and runs `validation.change`. This is
  what fires when a controlled `value` prop changes (not the user typing — typing goes through
  `onChange` below).
- `React.useRef` (local `inputRef`) `packages/react/src/field/control/FieldControl.tsx:118` and
  `useTimeout` (`enterValidationTimeout`)
  `packages/react/src/field/control/FieldControl.tsx:119`.
- `useIsoLayoutEffect` `packages/react/src/field/control/FieldControl.tsx:121-125` — autofocus
  reconciliation: if `autoFocus` is set and the element is already the (shadow-DOM-safe)
  `activeElement`, set `focused` true.
- `useRenderElement` `packages/react/src/field/control/FieldControl.tsx:127-213` — the render
  pipeline (DOM section).

State transitions all happen inside handlers merged onto the element:

- `onChange` `packages/react/src/field/control/FieldControl.tsx:139-159` — always fires
  `onValueChange` with a details object (`createChangeEventDetails(REASONS.none,
  event.nativeEvent)` `packages/react/src/field/control/FieldControl.tsx:141`); if controlled,
  returns immediately (the `value` prop is the source of truth, so a consumer who
  rejects/rewrites the input never pollutes field state, comment
  `packages/react/src/field/control/FieldControl.tsx:144-146`); if uncontrolled, sets `dirty`
  (vs `validityData.initialValue`) and `filled`, then — only when the native event wasn't
  default-prevented and the details aren't canceled — `clearErrors` + `validation.change`
  `packages/react/src/field/control/FieldControl.tsx:150-158`.
- `onFocus` `packages/react/src/field/control/FieldControl.tsx:160-162` — `setFocused(true)`.
- `onBlur` `packages/react/src/field/control/FieldControl.tsx:163-187` — `setTouched(true)`,
  `setFocused(false)`; when `validationMode === 'onBlur'`, commits validation; in controlled
  mode it schedules a `queueMicrotask` re-read of the DOM value and commits again only if the
  user's blur handler normalized the value to something new and non-initial
  `packages/react/src/field/control/FieldControl.tsx:175-184`.
- `onKeyDown` `packages/react/src/field/control/FieldControl.tsx:188-207` — Enter on an
  INPUT-tag element: `setTouched(true)`; if the control sits inside the Form element
  (`form === formElementRef.current`) and the event isn't default-prevented, it records
  `submitCountRef` and starts a 0ms `enterValidationTimeout` that commits only if Form's
  implicit submission didn't already bump the counter — a fallback race with Form's own Enter
  handling `packages/react/src/field/control/FieldControl.tsx:193-202`; outside a Form it
  commits directly `packages/react/src/field/control/FieldControl.tsx:203-205`.

## Context providers/consumers

Input is a leaf: it consumes three contexts and provides none. Nothing crosses the boundary
downward — it renders a single host element with no children wiring.

1. `FieldRootContext` (consumed at
   `packages/react/src/field/control/FieldControl.tsx:51-62` via `useFieldRootContext`,
   `packages/react/src/internals/field-root-context/FieldRootContext.ts`). This is the dominant
   boundary crossing: field state + setters + `validityData` + the `validation` engine
   (`inputRef`, `change`, `commit`, `getValidationProps`, `registeredInputs`) all come from
   here. The context default
   `packages/react/src/internals/field-root-context/FieldRootContext.ts:32-61` supplies `NOOP`
   setters, `DEFAULT_FIELD_ROOT_STATE`, and inert validation (no-op `commit`/`change`,
   pass-through `getValidationProps`), which is why Input "automatically works" standalone: the
   data-attributes freeze at defaults and validation is skipped. Crossing *upward*: the control
   registers itself into Root via `useRegisterFieldControl`
   `packages/react/src/field/control/FieldControl.tsx:91-98` — it upserts
   `{ controlRef, id, value, getValue, enabled(!disabled), name }` keyed by a stable `Symbol`
   into Root's fields Map
   (`packages/react/src/internals/field-register-control/useRegisterFieldControl.ts:21-38`),
   re-registers in place on value change so Map insertion order is preserved
   `packages/react/src/internals/field-register-control/useRegisterFieldControl.ts:19-20`,
   registers `undefined` when disabled, and unregisters on unmount
   `packages/react/src/internals/field-register-control/useRegisterFieldControl.ts:40-45`.
2. `FormContext` (consumed at `packages/react/src/field/control/FieldControl.tsx:63`) —
   `clearErrors` (called on change/Enter), `formElementRef` (implicit-submission detection,
   `packages/react/src/field/control/FieldControl.tsx:193`), `submitCountRef` (the Enter-race
   check, `packages/react/src/field/control/FieldControl.tsx:195-201`).
3. `LabelableContext` (consumed at `packages/react/src/field/control/FieldControl.tsx:73`) —
   `labelId`, applied as `aria-labelledby`
   `packages/react/src/field/control/FieldControl.tsx:136` and used to derive the default `id`
   via `useLabelableId` `packages/react/src/field/control/FieldControl.tsx:75`.

## DOM/portal strategy and why

No portal, no wrapper — one native leaf host. `useRenderElement('input', componentProps, {...})`
`packages/react/src/field/control/FieldControl.tsx:127`. The "why" is that Input *is* the form
control: native `<input>` semantics (typing, value, form participation, accessibility) are kept
intact and Base UI only layers field-state bookkeeping on top. Customization through `render`
may replace the host element entirely; the component keeps spreading its merged props onto
whatever element is produced — this is exactly the behavior documented in behavior.md →
*DOM structure & portal behavior* (inline rendering, tolerated wrapper nesting, ref/className
merging).

The final element props are assembled by the `useRenderElement` pipeline
(`packages/react/src/internals/useRenderElement.tsx`):

- Prop layers, in merge order `packages/react/src/field/control/FieldControl.tsx:130-211`:
  (1) intrinsic props — `id`, `disabled`, `name`, `ref: validation.inputRef`,
  `aria-labelledby`, `autoFocus`, the `value`/`defaultValue` switch, and the four handlers
  above; (2) `elementProps` — the user's arbitrary DOM props; (3) a props-getter
  `(props) => validation.getValidationProps(disabled, props)`
  `packages/react/src/field/control/FieldControl.tsx:210` which appends `aria-invalid` when the
  field is invalid and not disabled
  (`packages/react/src/field/root/useFieldValidation.ts:367-376`). Because `mergeProps` chains
  event handlers right-to-left
  (`packages/react/src/merge-props/mergeProps.ts:17-23`,
  `packages/react/src/merge-props/mergeProps.ts:221-250`), a user-supplied
  `onChange`/`onBlur` runs *before* the intrinsic one and can suppress the intrinsic handler
  with `event.preventBaseUIHandler()`; `className` concatenates and `style` shallow-merges
  (`packages/react/src/merge-props/mergeProps.ts:166-184`). Note this handler-chaining semantic
  is itself untested at the unit level (behavior.md → *Events* is N/A).
- State → `data-*` attributes: `stateAttributesMapping: fieldValidityMapping`
  `packages/react/src/field/control/FieldControl.tsx:212` overrides the tri-state `valid` key —
  `null` → no attribute, `true` → `data-valid`, `false` → `data-invalid`
  (`packages/react/src/internals/field-constants/constants.ts:34-43`). Every other state key
  uses the default mapping
  (`packages/react/src/internals/getStateAttributesProps.ts:24-28`): truthy →
  `data-{lowercased key}` with `''`. Combined with the state object built at
  `packages/react/src/field/control/FieldControl.tsx:68-71`, this produces exactly the
  attribute set that `packages/react/src/input/InputDataAttributes.ts` documents
  (`data-disabled`, `data-valid`, `data-invalid`, `data-touched`, `data-dirty`, `data-filled`,
  `data-focused`) — but at runtime those attributes come from `FieldControlDataAttributes` +
  the state keys, **not** from the input unit's constants file (see the last section).
- Refs: `useRenderElement` merges the forwarded ref plus the local `inputRef`
  `packages/react/src/field/control/FieldControl.tsx:128` with the element-level
  `validation.inputRef` `packages/react/src/field/control/FieldControl.tsx:135` into a single
  merged ref (`packages/react/src/internals/useRenderElement.tsx:99-103`; the render-prop
  element's own ref is merged in as well via `getReactElementRef`). The local `inputRef` exists
  only for the autofocus focus check
  `packages/react/src/field/control/FieldControl.tsx:122`; `validation.inputRef` is the
  Field-owned read path for the live DOM value.
- `render` evaluation: a function is called as `(props, state)`; an element is cloned with
  merged props (`packages/react/src/internals/useRenderElement.tsx:164-196`), with a dev-only
  guard/error for invalid elements and a dev warning for uppercase-named render functions
  (`packages/react/src/internals/useRenderElement.tsx:182-194`,
  `packages/react/src/internals/useRenderElement.tsx:208-230`).

## Dependencies on other Base UI internals

Direct imports of the unit:

- `../field` — `Field.Control` (the entire runtime implementation) and the `FieldControlState`
  type `packages/react/src/input/Input.tsx:4` / `packages/react/src/input/Input.tsx:16` /
  `packages/react/src/input/Input.tsx:34` (barrel `packages/react/src/field/index.ts:1`).
- `internals/types` — `BaseUIComponentProps` `packages/react/src/input/Input.tsx:3`, which
  supplies the `className`/`render`/`style` API and the `preventBaseUIHandler` event typing
  (`packages/react/src/internals/types.ts:36-61`).

Transitive, via `Field.Control` — the set a port must account for, grouped:

- `internals/`: `useRenderElement` (+ `getStateAttributesProps`, `resolveClassName`,
  `resolveStyle`, `merge-props`/`mergePropsN`/`mergeClassNames`, `useValueChanged`,
  `createBaseUIEventDetails` + `reasons`, `field-constants/constants` for
  `fieldValidityMapping`) `packages/react/src/field/control/FieldControl.tsx:14-21`; contexts
  `field-root-context/FieldRootContext`, `form-context/FormContext`,
  `labelable-provider/{LabelableContext,useLabelableId}`,
  `field-register-control/useRegisterFieldControl`
  `packages/react/src/field/control/FieldControl.tsx:9-13`.
- `@base-ui/utils/`: `useControlled`, `useIsoLayoutEffect`, `owner` (`ownerDocument`),
  `useStableCallback`, `useTimeout` `packages/react/src/field/control/FieldControl.tsx:3-7`;
  plus (through `useRenderElement`) `useMergedRefs`/`useMergedRefsN`, `getReactElementRef`,
  `mergeObjects`, `warn`, `empty` (`packages/react/src/internals/useRenderElement.tsx:2-6`).
- `floating-ui-react`: only `activeElement`
  `packages/react/src/field/control/FieldControl.tsx:21`, used for the shadow-DOM-safe
  autofocus check `packages/react/src/field/control/FieldControl.tsx:121-125`.
- Upstream component: the `field` unit. Field.Root owns the state machine, the validation
  engine (`useFieldValidation`), and the form integration that this unit consumes; Input's
  contract is meaningful only relative to those (plus `form`, whose `FormContext` and implicit
  submission participate in Enter handling
  `packages/react/src/field/control/FieldControl.tsx:193-202`).

The unit's TODO.md entry (`library: input`) has no `wraps-external:` field, so there is no
external-package delegation to declare.

## Anything in source not explained by any test

- **`packages/react/src/input/InputDataAttributes.ts` is dead code at runtime.** No source file
  imports it and `packages/react/src/input/index.ts:1-2` does not re-export it; a search across
  `packages/react/src` only matches the unrelated sibling `ComboboxInputDataAttributes`
  (`packages/react/src/combobox/utils/stateAttributesMapping.ts:5`). The actual `data-*`
  attributes are emitted from `FieldControlDataAttributes` via `fieldValidityMapping` and the
  default state mapping (`packages/react/src/internals/field-constants/constants.ts:34-43`,
  `packages/react/src/internals/getStateAttributesProps.ts:24-28`), and the constants here
  duplicate those values. Flag for the golden-fixture stage and audit loop: nothing at runtime
  reads this module; a port should treat it as documentation of the Field-driven attribute set,
  not as an independent contract.
- **Everything value/validation/focus/keyboard-shaped is unverified at the unit level.**
  behavior.md → *State model*, *Focus management*, *Events*, and *Keyboard interactions* are all
  N/A, yet the source implements a rich machine (controlled/uncontrolled branching, dirty/filled
  tracking, blur/Enter commit semantics, the Form implicit-submission race). The behavior is
  explained by the Field.Control delegation, but no Input-level test pins any of it; unit-level
  golden fixtures must be derived from this spec and the field unit's tests, not from
  Input.test.tsx (which is conformance-only per behavior.md → *Shared harness dependencies*).
- **`onValueChange` / `value` / `defaultValue` props are re-declared on `InputProps`**
  `packages/react/src/input/Input.tsx:23-31` (with `defaultValue` typed as
  `Field.Control.Props['defaultValue']` and `value` as React's input value). Pure type
  narrowing over the delegation target; no test covers these three props on Input.
- **The forwarded ref is widened to `React.ForwardedRef<HTMLElement>`**
  `packages/react/src/input/Input.tsx:14`, looser than the default `<input>` element; only the
  default-render `instanceof HTMLInputElement` fact is proven (behavior.md → *Public API
  surface*). The widening is exercised by the type-level fixture
  `packages/react/src/input/Input.spec.tsx:4-6` (a `HTMLTextAreaElement` ref with
  `render={<textarea />}`), which exists to typecheck that composition and carries no runtime
  assertions — listed here so it isn't mistaken for behavioral coverage.
