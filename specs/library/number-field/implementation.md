# NumberField — whole-unit implementation index

Stage 2 synthesis over the five batches declared in `specs/library/number-field/parts/PLAN.json`.
This file is a map, not a restatement: WHAT behavior lives in `specs/library/number-field/behavior.md`
and the per-batch `parts/*.md` docs; WHY/HOW depth lives in the five `parts/<batch>.implementation.md`
files indexed below. Only cross-batch coordination that no single part owns is derived here.

## Batch map

- **root** — Owns the unit's core machinery: the two-layer state machine (numeric `value` vs display
  `inputValue`), the ref-heavy `NumberFieldRootContext`, the single mutation funnel
  (`setValue`/`incrementValue`), the wheel path, the hidden form/autofill input, and the shared
  `useNumberFieldStepperButton` hook both steppers delegate into. Depth:
  `specs/library/number-field/parts/root.implementation.md:10-207` (state machine, context, DOM).
- **stepper-group** — Increment/Decrement are one-line delegations into the shared stepper hook and
  Group is a stateless `role="group"` projection of root state; all stepping logic lives in the root
  batch. Depth: `specs/library/number-field/parts/stepper-group.implementation.md:14-126`.
- **input** — The event-dispatch leaf: an `<input type="text">` whose handler table flips Root's
  shared flags at the right moments (typing dirties, blur and keyboard steps commit and re-arm),
  plus paste caret restoration and Field/Form wiring. Depth:
  `specs/library/number-field/parts/input.implementation.md:12-228`.
- **scrub** — ScrubArea runs a pointer-driven state machine (pointer lock, window-level capture
  listeners, virtual-cursor portal) that produces only `(amount, direction, reason)` calls into the
  root funnel; ScrubAreaCursor is a stateless gated view over that state. Depth:
  `specs/library/number-field/parts/scrub.implementation.md:5-37`.
- **utils** — Pure, React-free foundation: `parseNumber`, the ordered `toValidatedNumber` validation
  pipeline, `removeFloatingPointErrors`, stepper timing constants, and scrub-cursor viewport bounds.
  Depth: `specs/library/number-field/parts/utils.implementation.md:5-40`.

## Cross-batch coordination

- **One context hub.** Root provides `NumberFieldRootContext` exactly once, wrapping both the root
  element and the hidden input (`packages/react/src/number-field/root/NumberFieldRoot.tsx:491-535`);
  its ref-heavy shape (`packages/react/src/number-field/root/NumberFieldRootContext.ts:8-36`) is the
  substrate every other batch consumes — Group directly, steppers via the shared hook, Input and
  ScrubArea/Cursor destructuring their own slices
  (`specs/library/number-field/parts/root.implementation.md:144-157`,
  `specs/library/number-field/parts/input.implementation.md:154-167`,
  `specs/library/number-field/parts/scrub.implementation.md:24-29`).
- **`allowInputSyncRef` has no single owner.** Root defines the dirty-text authority flag and gates
  its sync effect and text overwrite on it; Input sets it `false` on every typing/paste mutation and
  back to `true` at blur and keyboard commits; the stepper hook re-arms it at press commit and scrub
  re-arms it before each step. Its value at any moment is the union of flips performed by whichever
  batch ran the last interaction
  (`specs/library/number-field/parts/root.implementation.md:25-34`,
  `specs/library/number-field/parts/input.implementation.md:26-35`,
  `specs/library/number-field/parts/scrub.implementation.md:15`).
- **The two-callback commit model spans four batches.** `onValueChange` fires per applied change via
  Root's `setValue`; `onValueCommitted` fires once per interaction end, and `lastChangedValueRef` is
  the mailbox each batch commits from: stepper hold-stop and scrub pointerup read it (with a
  `?? valueRef` fallback), Input blur re-reads it post-clamp, and Root's cancel veto skips both its
  update and the commit. `hasPendingCommitRef` is likewise produced by Root (set in `setValue`,
  cleared in the stable `onValueCommitted` wrapper) and consumed by Input (snapshot in blur)
  (`specs/library/number-field/parts/root.implementation.md:45-56`,
  `specs/library/number-field/parts/root.implementation.md:108-131`,
  `specs/library/number-field/parts/input.implementation.md:70-94`,
  `specs/library/number-field/parts/scrub.implementation.md:16`).
- **One mutation funnel, many sources.** Typing, blur, paste, keyboard steps, stepper clicks/holds,
  wheel ticks, and scrub movement all converge on Root's `setValue`/`incrementValue`, whose
  classification feeds utils' ordered validation pipeline; Input's split-phase typing decides only
  *whether* to call the funnel and scrub never computes a value at all
  (`specs/library/number-field/parts/root.implementation.md:35-44`,
  `specs/library/number-field/parts/utils.implementation.md:14-21`,
  `specs/library/number-field/parts/input.implementation.md:95-108`,
  `specs/library/number-field/parts/scrub.implementation.md:22`).
- **State→attribute projection is shared, and `data-scrubbing` crosses batches.** Every rendered
  part passes the same `stateAttributesMapping`
  (`specs/library/number-field/parts/utils.implementation.md:26-31`); `data-scrubbing` documents the
  one state whose owner and writer differ: Root owns `isScrubbing`, the scrub batch flips it via
  `setIsScrubbing`, and all seven `*DataAttributes` vocabularies render it
  (`specs/library/number-field/parts/root.implementation.md:96-100`,
  `specs/library/number-field/parts/stepper-group.implementation.md:60-67`,
  `specs/library/number-field/parts/scrub.implementation.md:26-29`).
- **Focus converges on one handle.** Hidden-input focus forwarding, stepper pointerdown, scrub mouse
  press, and the wheel focus check all route through the shared `inputRef`/`focusInput` to the
  visible input (`specs/library/number-field/parts/root.implementation.md:204-207`,
  `specs/library/number-field/parts/input.implementation.md:225-228`).
- **Layering has one back-edge, and it is type-only.** utils is the React-free foundation consumed
  by root, input, steppers, and scrub; its only dependency on them is a type-only import of
  `NumberFieldRootState`, so there is no runtime cycle
  (`specs/library/number-field/parts/utils.implementation.md:52-54`).
- **Portal discipline is unit-wide: exactly one portal.** Only the scrub cursor portals (to the
  owner document's body); root, input, and stepper-group render inline with no portals
  (`specs/library/number-field/parts/scrub.implementation.md:34`,
  `specs/library/number-field/parts/root.implementation.md:184-186`,
  `specs/library/number-field/parts/input.implementation.md:196-198`,
  `specs/library/number-field/parts/stepper-group.implementation.md:102-104`).
