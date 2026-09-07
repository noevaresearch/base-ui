# NumberField — implementation part: steppers (`Increment` / `Decrement`) and `Group`

Batch scope: `packages/react/src/number-field/decrement/NumberFieldDecrement.tsx`,
`packages/react/src/number-field/decrement/NumberFieldDecrementDataAttributes.ts`,
`packages/react/src/number-field/group/NumberFieldGroup.tsx`,
`packages/react/src/number-field/group/NumberFieldGroupDataAttributes.ts`,
`packages/react/src/number-field/increment/NumberFieldIncrement.tsx`,
`packages/react/src/number-field/increment/NumberFieldIncrementDataAttributes.ts`.
Companion to `specs/library/number-field/behavior.md` (WHAT) and
`specs/library/number-field/parts/stepper-group.md` (batch WHAT) — this file is WHY/HOW only for
these six files. The shared stepper machinery lives in the root batch and is only cited here
(`specs/library/number-field/parts/root.implementation.md`), per the scope rule.

## State machine / hooks used

**This batch owns no state machine — it is two one-line delegations and one state projection,
which is the design point.**

- `NumberFieldIncrement` and `NumberFieldDecrement` are `React.forwardRef` components whose entire
  render body is a single call to the shared stepper hook, differing only in the `isIncrement`
  boolean: `true` (`packages/react/src/number-field/increment/NumberFieldIncrement.tsx:17`) vs
  `false` (`packages/react/src/number-field/decrement/NumberFieldDecrement.tsx:17`). Every
  behavioral difference between the two parts — step direction, boundary disable (`max` vs `min`),
  `aria-label` (`Increase`/`Decrease`), and the change reason (`incrementPress`/`decrementPress`) —
  is derived from that one boolean inside the hook
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:25-33`, whose doc comment
  states the parts "differ only in the direction they step and the boundary … at which they become
  disabled"). The two public components are therefore pure API shells: JSDoc + per-part
  prop/state types + namespace, zero logic of their own.
- The hook-composition chain the wrappers delegate into is: context read → derived disabled →
  `usePressAndHold` → `useButton` → `useRenderElement`
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:43-189`) — all root-batch
  internals, not re-derived here (see
  `specs/library/number-field/parts/root.implementation.md:102-137`). Note the hook is named
  `use*` but *returns the rendered element*: calling it is the component's whole render, so the
  wrapper components declare no hooks of their own and no conditional rendering — every React
  state, ref, and effect involved in stepping is created once, inside the one shared hook
  instance, rather than duplicated per part.
- State typing is identity: each part's state interface is an empty extension of the root state —
  `NumberFieldDecrementState extends NumberFieldRootState`
  (`packages/react/src/number-field/decrement/NumberFieldDecrement.tsx:20`),
  `NumberFieldIncrementState extends NumberFieldRootState`
  (`packages/react/src/number-field/increment/NumberFieldIncrement.tsx:20`),
  `NumberFieldGroupState extends NumberFieldRootState`
  (`packages/react/src/number-field/group/NumberFieldGroup.tsx:33`). The steppers' *rendered*
  state is that root state with exactly one local override — the hook merges its derived
  `disabled` over it (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:182`) —
  which is the mechanism behind "the render-state object passed to a `className` function has
  `disabled` true when either the root or the button itself is disabled"
  (`specs/library/number-field/parts/stepper-group.md:10`). Group's rendered state is the raw
  root state, untouched.
- The steppers' props add `NativeButtonProps` (the `nativeButton` flag,
  `packages/react/src/internals/types.ts:63-71`) on top of
  `BaseUIComponentProps<'button', State>` (`packages/react/src/internals/types.ts:36-61`)
  (`packages/react/src/number-field/decrement/NumberFieldDecrement.tsx:22-23`;
  `packages/react/src/number-field/increment/NumberFieldIncrement.tsx:22-23`); Group has only
  `BaseUIComponentProps<'div', State>` (`packages/react/src/number-field/group/NumberFieldGroup.tsx:35`).
  The stateful `className`/`style` callback contract is typed at these aliases — that is where the
  render-state object handed to consumer callbacks is defined.
- The three `*DataAttributes.ts` files are pure constant vocabularies — the identical ten
  `data-*` strings as the root file (`packages/react/src/number-field/root/NumberFieldRootDataAttributes.ts:1-40`),
  duplicated per part (`packages/react/src/number-field/decrement/NumberFieldDecrementDataAttributes.ts:1-40`,
  `packages/react/src/number-field/group/NumberFieldGroupDataAttributes.ts:1-40`,
  `packages/react/src/number-field/increment/NumberFieldIncrementDataAttributes.ts:1-40`). They
  document that every part projects the *full* root state onto attributes — including
  `data-scrubbing`, whose flag is owned by Root and flipped by the scrub batch, so steppers and
  Group expose it even though neither scrubs.
- Group owns no hooks beyond the context read: destructure `render`/`className`/`style`
  (`packages/react/src/number-field/group/NumberFieldGroup.tsx:19`), read `state` from context
  (`packages/react/src/number-field/group/NumberFieldGroup.tsx:21`), one `useRenderElement` call
  (`packages/react/src/number-field/group/NumberFieldGroup.tsx:23-28`). It is a stateless
  projection of root state onto a `div`.
- Public naming is namespace + barrel plumbing: per-part `Props`/`State` namespaces
  (`packages/react/src/number-field/decrement/NumberFieldDecrement.tsx:25-27`;
  `packages/react/src/number-field/increment/NumberFieldIncrement.tsx:25-27`;
  `packages/react/src/number-field/group/NumberFieldGroup.tsx:37-39`) re-exported under the public
  names by `packages/react/src/number-field/index.parts.ts:2-4` and type-exported by
  `packages/react/src/number-field/index.ts:1-9`.

## Context providers/consumers

- All three parts are consumers of the single `NumberFieldRootContext` provider the Root batch
  establishes; none of this batch's six files provides a context.
- Group consumes the context directly but destructures only `state`
  (`packages/react/src/number-field/group/NumberFieldGroup.tsx:21`) — a deliberately read-only
  slice. Group is display-only, so the rest of the context shape (shared refs, mutation funnels,
  `onValueCommitted` — `packages/react/src/number-field/root/NumberFieldRootContext.ts:8-36`) is
  irrelevant to it.
- The steppers have *zero context plumbing in this batch's files*: their entire context
  consumption (bounds, the shared interaction refs, `setValue`/`incrementValue`/`getStepAmount`,
  `focusInput`, `id`, `state`, `onValueCommitted`) happens inside the delegated hook
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:43-59`). That wrapper/hook
  split is what keeps the two public components' bodies to a single line.
- The context-misuse guard is inherited, not implemented here: `useNumberFieldRootContext` throws
  the "must be placed within `<NumberField.Root>`" error on a missing provider
  (`packages/react/src/number-field/root/NumberFieldRootContext.ts:42-51`). Group reaches it via
  its direct call (`packages/react/src/number-field/group/NumberFieldGroup.tsx:21`); the steppers
  reach it only transitively through the hook.

## DOM/portal strategy and why

- No portals and no conditional rendering anywhere in this batch; each part always renders exactly
  one element. This matches the behavior doc, where nothing popup/portal-related is exercised for
  these parts (`specs/library/number-field/parts/stepper-group.md:62`).
- Group renders a `div` with `role: 'group'` hardcoded as the *first* entry of the props array and
  consumer `elementProps` as the second
  (`packages/react/src/number-field/group/NumberFieldGroup.tsx:23-28`, role at
  `packages/react/src/number-field/group/NumberFieldGroup.tsx:26`). `useRenderElement` folds the
  props array left-to-right so later entries win
  (`packages/react/src/internals/useRenderElement.tsx:122-125`;
  `packages/react/src/merge-props/mergeProps.ts:110-112`), so a consumer `role` prop overrides the
  built-in one while everything else merges normally. The group role is the accessible grouping of
  input + steppers (`specs/library/number-field/parts/stepper-group.md:55`).
- All state attributes on Group come from the shared `stateAttributesMapping` passed as the
  mapping option (`packages/react/src/number-field/group/NumberFieldGroup.tsx:27`), which nulls
  `value`/`inputValue` and spreads the Field validity mapping
  (`packages/react/src/number-field/utils/stateAttributesMapping.ts:5-9`) — that is how the
  ten-attribute vocabulary the DataAttributes files document gets applied without Group writing a
  single attribute itself.
- The steppers' DOM decisions (native `<button>`, `aria-label` Increase/Decrease, `aria-controls`
  → input id, `tabIndex={-1}` keyboard exclusion, `user-select: none`) are made entirely inside
  the delegated hook (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:120-128`;
  behavior: `specs/library/number-field/parts/stepper-group.md:52-54`;
  `specs/library/number-field/parts/root.implementation.md:199-203`). This batch's contribution to
  the DOM contract is the element type selection and *not* overriding anything: the wrappers pass
  only props/render/ref through, so the hook's defaults are the only defaults.

## Dependencies on other Base UI internals

Imports cited by path (internals not re-derived here):

- Root batch (own unit, outside this batch):
  - `useNumberFieldStepperButton` — the entire steppers implementation
    (`packages/react/src/number-field/increment/NumberFieldIncrement.tsx:4`;
    `packages/react/src/number-field/decrement/NumberFieldDecrement.tsx:4`; defined
    `packages/react/src/number-field/root/useNumberFieldStepperButton.ts:29-33`).
  - `useNumberFieldRootContext` — Group's context read
    (`packages/react/src/number-field/group/NumberFieldGroup.tsx:3`; defined
    `packages/react/src/number-field/root/NumberFieldRootContext.ts:42-51`).
  - `NumberFieldRootState` — type-only import in all three parts
    (`packages/react/src/number-field/decrement/NumberFieldDecrement.tsx:5`;
    `packages/react/src/number-field/increment/NumberFieldIncrement.tsx:5`;
    `packages/react/src/number-field/group/NumberFieldGroup.tsx:4`; defined
    `packages/react/src/number-field/root/NumberFieldRoot.tsx:669`).
- `packages/react/src/internals/types.ts` — `BaseUIComponentProps` and `NativeButtonProps`
  (`packages/react/src/number-field/decrement/NumberFieldDecrement.tsx:3`;
  `packages/react/src/number-field/increment/NumberFieldIncrement.tsx:3`; defined
  `packages/react/src/internals/types.ts:36-61`,
  `packages/react/src/internals/types.ts:63-71`).
- `packages/react/src/internals/useRenderElement.tsx` — Group's render pipeline
  (`packages/react/src/number-field/group/NumberFieldGroup.tsx:7`; props-array fold at
  `packages/react/src/internals/useRenderElement.tsx:122-125`); also the final call of the
  delegated stepper hook
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:184-189`).
- `packages/react/src/merge-props/mergeProps.ts` — the left-to-right overwrite fold that
  `useRenderElement` delegates to (`packages/react/src/merge-props/mergeProps.ts:110-112`);
  cited only for the Group role-override semantics above.
- Own-unit utils batch: `stateAttributesMapping`
  (`packages/react/src/number-field/group/NumberFieldGroup.tsx:6`; defined
  `packages/react/src/number-field/utils/stateAttributesMapping.ts:5-9`).
- External delegation: none. This unit's `TODO.md` entry has no `wraps-external:` field
  (`TODO.md:440-447`), so no third-party package delegation applies to this batch.

## Anything in source not explained by any test

- The three `*DataAttributes.ts` files are **runtime dead code**: no source file imports
  `NumberFieldIncrementDataAttributes`, `NumberFieldDecrementDataAttributes`, or
  `NumberFieldGroupDataAttributes` anywhere in the repo, and the package barrel does not
  re-export them (`packages/react/src/number-field/index.ts:1-9`); they appear only in inventory
  JSON (`ralph/generated/components.json:876-880`). They are documentation vocabulary — and no
  test asserts any of the ten `data-*` attributes on these parts
  (`specs/library/number-field/parts/stepper-group.md:14`); the conformance suite only proves
  user-supplied `data-*` forwarding (`specs/library/number-field/parts/stepper-group.md:12`).
- The `data-scrubbing` entry in all three vocabularies
  (`packages/react/src/number-field/decrement/NumberFieldDecrementDataAttributes.ts:4`;
  `packages/react/src/number-field/group/NumberFieldGroupDataAttributes.ts:4`;
  `packages/react/src/number-field/increment/NumberFieldIncrementDataAttributes.ts:4`) implies
  steppers and Group wear `data-scrubbing` while a ScrubArea drag is active (shared root state);
  no test in this batch asserts that attribute on any of the three parts.
- Group's missing-provider error path is untested: the Group test file only asserts `role: group`
  (`specs/library/number-field/parts/stepper-group.md:55`); the "must be placed within
  `<NumberField.Root>`" error is asserted for Input/ScrubAreaCursor, not Group
  (`specs/library/number-field/parts/root.implementation.md:153-157`).
- The role-override semantics — a consumer `role` winning over the built-in `role: 'group'` via
  merge order — are inferred from the props-merge pipeline, not asserted by any test in this
  batch (conformance covers `lang`/`data-*`/`style` forwarding, not `role`).
- The empty state interfaces (`…State extends NumberFieldRootState`) are public-API type aliases
  only; nothing behavioral distinguishes them from the root state beyond the `className`-function
  state object already noted (`specs/library/number-field/parts/stepper-group.md:10`).
