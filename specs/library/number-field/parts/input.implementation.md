# NumberField.Input — implementation part: the leaf event-dispatch input

Batch scope: `packages/react/src/number-field/input/NumberFieldInput.tsx`,
`packages/react/src/number-field/input/NumberFieldInputDataAttributes.ts`, and the type-only
`packages/react/src/number-field/input/NumberFieldInput.spec.tsx`. Companion to
`specs/library/number-field/behavior.md` (unit-index WHAT) and
`specs/library/number-field/parts/input.md` (input-batch WHAT) — this file is WHY/HOW only for
these three files. Behavior is cited by section, not restated. The unit's `TODO.md` entry
(`TODO.md:444-451`) has no `wraps-external:` field (matching `specs/library/number-field/behavior.md:3`),
so no third-party package delegation applies.

## State machine / hooks used

**The Input owns almost no state of its own — it is the event-dispatch leaf of Root's two-layer
state machine. All authority state (numeric `value`, display `inputValue`, the dirty-sync flag,
commit bookkeeping) lives in Root and reaches the Input through context refs.** What the Input
contributes is the choreography that flips those shared flags at the right moments.

- Local state is exactly two refs: `blockRevalidationRef`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:85`) and `pendingCaretRef`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:86`). Everything else the handlers
  read and write (`allowInputSyncRef`, `lastChangedValueRef`, `hasPendingCommitRef`, `valueRef`,
  `formatOptionsRef`, `inputRef`) is Root-owned, shared via
  `packages/react/src/number-field/root/NumberFieldRootContext.ts:8-36` — the ref-heavy context
  shape lets these handlers read current interaction state without prop-drilling or stale closures.
- `allowInputSyncRef` is the dirty-text authority flag defined by Root (the mechanism behind
  "Dirty-input authority vs full-precision numeric state",
  `specs/library/number-field/behavior.md:43`). The Input drives its whole lifecycle: set `false`
  on every user text mutation (`onChange` at `packages/react/src/number-field/input/NumberFieldInput.tsx:234`,
  `onPaste` at `packages/react/src/number-field/input/NumberFieldInput.tsx:447`) and back to
  `true` at commit points — blur (`packages/react/src/number-field/input/NumberFieldInput.tsx:151`)
  and keyboard steps that change the value
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:380-383`). This matches the split
  of ownership documented in the root implementation: the units that run commits own the flag
  flips.
- `pendingCaretRef` implements deferred caret restoration for paste. `onPaste` splices text into
  the *controlled* value, so the browser's native caret placement is lost; the handler records
  `selectionStart + pastedData.length`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:448`) and a `useIsoLayoutEffect`
  applies `setSelectionRange` after the re-render settles
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:92-98`) — the mechanism behind the
  paste caret edge ("paste inserts at the caret with the caret left after the pasted text",
  `specs/library/number-field/behavior.md:37`). The effect runs dep-less and is guarded by the
  ref's null check, so it is a no-op on every non-paste render.
- `blockRevalidationRef` coordinates blur commits with `useValueChanged`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:100-109`), which fires
  `clearErrors(name)` plus Field `validation.change` whenever the numeric value changes. The blur
  handler sets the flag before its own `setValue`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:202`) so a value the user never
  "changed" in Field's eyes (a blur normalization) doesn't trigger onChange-mode revalidation
  unless the Field is configured for it (`shouldValidateOnChange()`,
  `packages/react/src/number-field/input/NumberFieldInput.tsx:103-106`); it is reset when the
  cancel path returns or when validation normalizes back to the current value (the
  `useValueChanged` effect wouldn't fire to reset it in that case,
  `packages/react/src/number-field/input/NumberFieldInput.tsx:204-213`).
- `useRegisterFieldControl(inputRef, id, value, undefined, !disabled, nameProp)`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:88`) registers the *visible* input
  with the Field form registry (control ref, id, current value, disabled gate, name). The
  internals are the field-register-control unit's
  (`packages/react/src/internals/field-register-control/useRegisterFieldControl.ts:21-45`) and are
  not re-derived here.
- The single `inputProps` object
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:111-453`) is the state machine's
  transition table, keyed by DOM event:

  - `onFocus` (`packages/react/src/number-field/input/NumberFieldInput.tsx:128-135`) only flips
    Field focus state (`setFocused(true)`); it is the one handler that intentionally ignores
    `readOnly` — the "readOnly suppresses mutation, never focus" cross-cutting rule
    (`specs/library/number-field/behavior.md:45`).
  - `onBlur` (`packages/react/src/number-field/input/NumberFieldInput.tsx:136-227`) is the
    three-branch commit endpoint behind "Two-callback commit model"
    (`specs/library/number-field/behavior.md:42`) and "Blur commits the validated number"
    (`specs/library/number-field/parts/input.md:19-21`). It first reads-then-clears dirty
    authority (`hadManualInput`, `packages/react/src/number-field/input/NumberFieldInput.tsx:148-151`)
    and records `hasPendingCommitRef` before clearing it
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:149`). Branch one (empty text):
    `setValue(null, inputClear)` with cancel respect, an `onBlur`-mode `validation.commit(null)`,
    and a commit gated on interaction so a never-touched empty field reports nothing
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:153-169`). Branch two
    (unparseable text): bail, leaving the raw text — the overflow edge
    (`specs/library/number-field/parts/input.md:92`). Branch three (parseable): resolve the
    committed number — keep the authoritative `value` when the text is untouched display output
    with no rounding options, apply `removeFloatingPointErrors` when explicit rounding options
    exist, else use the parse
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:180-192`) — the precision
    preservation behind "no-edit focus/blur cycles are no-ops"
    (`specs/library/number-field/parts/input.md:26`) and "blur rounds AND fires onValueChange with
    the rounded number" (`specs/library/number-field/parts/input.md:30`). It then funnels through
    `setValue` (re-reading the post-clamp `lastChangedValueRef` as the committed value,
    `packages/react/src/number-field/input/NumberFieldInput.tsx:200-214`), commits
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:215-220`), and finally
    normalizes only the *display text* to the canonical format
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:222-226`) — the display/value
    split of `specs/library/number-field/behavior.md:47`.
  - `onChange` (`packages/react/src/number-field/input/NumberFieldInput.tsx:228-268`) implements
    split-phase typing (`specs/library/number-field/behavior.md:13`): text always lands in
    `inputValue` for whitelist-passing strings, while `setValue` fires only when
    `parseNumber` succeeds (`packages/react/src/number-field/input/NumberFieldInput.tsx:261-267`) —
    so unparseable-but-locale-valid partial input (e.g. a trailing decimal separator) survives as
    text without a numeric update (`specs/library/number-field/parts/input.md:47`). The whitelist
    itself
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:246-255`) admits numerals,
    minus-detecting chars, the formatter-derived allowed keys, and bidi/format controls — the
    last so RTL-locale format controls don't reject strings `parseNumber` will strip
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:252-254`). It also carries a
    React #9023 workaround keyed on the native event's `defaultPrevented`
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:229-232`), the consumer
    `preventDefault` semantics of `specs/library/number-field/parts/input.md:77`.
  - `onKeyDown` (`packages/react/src/number-field/input/NumberFieldInput.tsx:269-407`) is a gate-
    then-dispatch design. Gating: selection-aware single-sign logic (a sign key is admitted only
    when absent, all-selected, or the existing sign is inside the selection,
    `packages/react/src/number-field/input/NumberFieldInput.tsx:301-320`) and one-per-symbol
    logic for decimal/currency/percent
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:322-329`) — the mechanisms
    behind `specs/library/number-field/parts/input.md:47-49`. The allowlist early-return
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:335-348`) admits IME composition
    (`which === 229`, a Safari-specific probe per the in-file comment — composition behavior is
    UNVERIFIED by tests, `specs/library/number-field/parts/input.md:50`), non-step Alt
    combinations (Alt+ArrowUp/Down *is* the smallStep path,
    `packages/react/src/number-field/input/NumberFieldInput.tsx:332-333`),
    ctrl/meta, admitted non-numerics, numerals, and `NAVIGATE_KEYS`
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:34-42`). Dispatch: Home/End
    resolve a boundary value only when the bound is defined
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:350-356`); anything
    multi-character that isn't a step key or boundary jump falls through to the browser
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:358-362`). The step branch
    parses dirty text as `currentValue` but passes `null` when the input is synced so stepping
    uses full-precision numeric state
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:364-370`), then
    `preventDefault` + `stopPropagation`
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:374-376`), refreshes
    `lastChangedValueRef` from `valueRef` when synced so a canceled step can't commit a stale
    value (`packages/react/src/number-field/input/NumberFieldInput.tsx:384-390`), and calls
    `incrementValue` (ArrowUp/Down) or `setValue(boundaryValue)` (Home/End)
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:392-400`), emitting a single
    `onValueCommitted` only when the change applied
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:402-406`). The dirty snapshot at
    the top of the handler deliberately does *not* clear the flag — navigation keys must not
    discard dirty authority (`packages/react/src/number-field/input/NumberFieldInput.tsx:276-280`),
    the mechanism behind `specs/library/number-field/parts/input.md:23`.
  - `onPaste` (`packages/react/src/number-field/input/NumberFieldInput.tsx:408-452`) reads the
    clipboard inside try/catch (dev-only `warn` with `SafeReact.captureOwnerStack` on failure,
    `packages/react/src/number-field/input/NumberFieldInput.tsx:415-428` — the warning behavior of
    `specs/library/number-field/parts/input.md:80-81`), then splices at the caret/selection
    instead of replacing the whole value
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:433-442`). A null parse vetoes
    the paste entirely (no dirty flag, no callbacks); a valid parse marks dirty, defers the
    caret, and applies value+text with reason `inputPaste`
    (`packages/react/src/number-field/input/NumberFieldInput.tsx:444-451`).
- Rendering goes through `useRenderElement`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:455-460`), merging the forwarded
  ref with Root's shared `inputRef`, the Root `state`, and `stateAttributesMapping`.

## Context providers/consumers

The Input provides nothing; it is a pure consumer of four contexts:

- `NumberFieldRootContext` via `useNumberFieldRootContext`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:7`, destructured at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:56-77`): the Root `state` plus the
  mutation funnel (`setValue`, `incrementValue`, `getStepAmount`, `getAllowedNonNumericKeys`),
  the shared refs (`allowInputSyncRef`, `formatOptionsRef`, `inputRef`, `lastChangedValueRef`,
  `hasPendingCommitRef`, `valueRef`), the locale/format inputs (`locale`, `inputMode`), and the
  identity/config fields (`id`, `name`/`nameProp`, `min`, `max`). The context-misuse throw for an
  Input outside Root is the provider hook's own guard
  (`packages/react/src/number-field/root/NumberFieldRootContext.ts:42-51`), asserted at
  `specs/library/number-field/parts/input.md:7`.
- `FormContext` via `useFormContext`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:11`, consumed at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:80`): only `clearErrors`, called in
  the `useValueChanged` callback so any user-driven value change clears server errors — the Form
  "error clearing on change" lifecycle (`specs/library/number-field/behavior.md:37`).
- `FieldRootContext` via `useFieldRootContext`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:9`, consumed at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:81-82`): `validationMode`,
  `setTouched`/`setFocused` (driven by the Input's own focus/blur handlers, gated on
  `defaultPrevented` — `specs/library/number-field/parts/input.md:55-56`), `invalid`,
  `shouldValidateOnChange`, and the `validation` object used for change/commit and the final
  `getValidationProps` merge.
- `LabelableContext` via `useLabelableContext`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:12`, consumed at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:83`): `labelId`, wired as
  `aria-labelledby` (`packages/react/src/number-field/input/NumberFieldInput.tsx:124`) — the
  Field.Label linking described in `specs/library/number-field/behavior.md:25`.
- The state→attribute vocabulary this part renders is declared in
  `packages/react/src/number-field/input/NumberFieldInputDataAttributes.ts:1-40`
  (`data-scrubbing`, `data-disabled`, `data-readonly`, `data-required`, validity/touched/dirty/
  filled/focused pairs); the actual mapping is the shared
  `packages/react/src/number-field/utils/stateAttributesMapping.ts:5-9`, which nulls
  `value`/`inputValue` and spreads `fieldValidityMapping`, passed at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:459`.

## DOM/portal strategy and why

- One leaf element: a native `<input type="text">`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:118`) rendered inline through
  `useRenderElement`. **No portals** — consistent with the unit-wide finding
  (`specs/library/number-field/behavior.md:29`, `specs/library/number-field/parts/input.md:69`).
- `type="text"` (never `number` or `email`) is load-bearing: the visible text is locale-formatted
  (`"1,234.56"`, `١٫٢٣٩`, …) which a number input cannot hold, and the paste caret restore and
  step logic depend on `selectionStart`/`selectionEnd` always being present. The in-file comment
  states that overriding `type` with a selection-less one is unsupported
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:434-437`).
- Mobile keyboard hints come from context (`inputMode`,
  `packages/react/src/number-field/input/NumberFieldInput.tsx:116`); the iOS widening decision is
  Root's, not this batch's (`specs/library/number-field/behavior.md:48`).
- Text-entry hardening: `autoComplete`/`autoCorrect` off and `spellCheck=false`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:119-121`) because the value is
  numeric-formatted, not natural language; `suppressHydrationWarning`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:125-127`) because server/client
  locale differences make the formatted text legitimately mismatch on hydration.
- A11y wiring is attribute-level only: `aria-roledescription: 'Number field'`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:122`), `aria-invalid` gated on
  `!disabled && invalid`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:123`), `aria-labelledby` from the
  Labelable context
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:124`). There is deliberately no
  spinbutton role or `aria-valuenow/min/max`
  (`specs/library/number-field/behavior.md:25`).
- Prop merging order at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:458` is
  `[inputProps, elementProps, props => validation.getValidationProps(disabled, props)]`: built-ins
  first, consumer props can override them, and Field validation props (`aria-describedby`,
  `aria-invalid` from server errors) are appended last so they survive consumer overrides.
- Ref merging (`packages/react/src/number-field/input/NumberFieldInput.tsx:456`) combines the
  consumer `forwardedRef` with Root's `inputRef` — the shared handle every focus-routing path
  (hidden-input forwarding, stepper press, scrub, wheel focus check) converges on
  (`specs/library/number-field/behavior.md:46`).

## Dependencies on other Base UI internals

Imports cited by path (internals not re-derived here):

- `packages/utils/src/safeReact.ts` — `SafeReact.captureOwnerStack` for the dev-only paste
  warning (`packages/react/src/number-field/input/NumberFieldInput.tsx:3`, used at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:420`).
- `packages/utils/src/warn.ts` — deduped dev warning
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:4`, used at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:421`).
- `packages/utils/src/useIsoLayoutEffect.ts`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:5`, used at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:92`).
- `packages/utils/src/formatNumber.ts` — canonical display-text normalization
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:6`, used at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:223`).
- `packages/react/src/number-field/root/NumberFieldRootContext.ts` — the provider consumer hook
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:7`).
- `packages/react/src/number-field/root/NumberFieldRoot.tsx` — the `NumberFieldRootState` type
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:23`).
- `packages/react/src/internals/types.ts` — `BaseUIComponentProps`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:8`).
- `packages/react/src/internals/field-root-context/FieldRootContext.ts`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:9`).
- `packages/react/src/internals/field-register-control/useRegisterFieldControl.ts`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:10`, used at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:88`).
- `packages/react/src/internals/form-context/FormContext.ts`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:11`).
- `packages/react/src/internals/labelable-provider/LabelableContext.ts`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:12`).
- Own-unit utils batch: `packages/react/src/number-field/utils/parse.ts` —
  `getNumberLocaleDetails`, `isNumeralChar`, `parseNumber`, and the regex constants
  `ANY_MINUS_RE`/`ANY_PLUS_RE`/`ANY_MINUS_DETECT_RE`/`ANY_PLUS_DETECT_RE`/
  `FORMAT_CONTROL_DETECT_RE` (`packages/react/src/number-field/input/NumberFieldInput.tsx:13-22`);
  `packages/react/src/number-field/utils/validate.ts` — `hasNumberFormatRoundingOptions`,
  `removeFloatingPointErrors` (`packages/react/src/number-field/input/NumberFieldInput.tsx:32`,
  used at `packages/react/src/number-field/input/NumberFieldInput.tsx:178,189`).
- `packages/react/src/number-field/utils/stateAttributesMapping.ts`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:24`, used at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:459`).
- `packages/react/src/internals/useRenderElement.tsx`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:25`, used at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:455`).
- `packages/react/src/internals/createBaseUIEventDetails.ts` — `createChangeEventDetails` /
  `createGenericEventDetails` (`packages/react/src/number-field/input/NumberFieldInput.tsx:26-29`).
- `packages/react/src/internals/useValueChanged.ts`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:30`, used at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:100`).
- `packages/react/src/internals/reasons.ts` — the `REASONS` constants
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:31`).

## Anything in source not explained by any test

- `packages/react/src/number-field/input/NumberFieldInput.spec.tsx:4-10` is a **type-only** spec
  with no runtime assertions: it pins that the `render` callback receives the native input props
  (`disabled`/`readOnly` typed `boolean | undefined`). The `NumberFieldInputState extends
  NumberFieldRootState` alias
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:465`) and the typed
  `'aria-roledescription'` prop
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:472-477`) are likewise compile-time
  surface with no behavior test.
- `'aria-roledescription'` — both the default `'Number field'`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:122`) and consumer override via the
  prop are untested; the input batch asserts no aria attributes at all
  (`specs/library/number-field/parts/input.md:63`).
- `Escape` in `NAVIGATE_KEYS`
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:34-42`): the keyboard spec lists
  Backspace/Delete/Arrows/Tab/Enter and IME 229 as never-preventDefault-ed
  (`specs/library/number-field/behavior.md:17`), but Escape as a pass-through member of the set is
  asserted nowhere.
- The multi-character pass-through rule
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:358-362`): PageUp, Insert, F-keys,
  and Home/End *without* defined bounds reach the browser untouched. The tests only cover Home/End
  with bounds present (`specs/library/number-field/parts/input.md:37`).
- `useRegisterFieldControl` on the visible input
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:88`): no input-batch test asserts
  the registration or its FormData contribution; the form-submission behavior in the spec is
  attributed to Root's hidden input (`specs/library/number-field/behavior.md:37`). The split —
  hidden input for native submission, visible input registered for Field — is untested here.
- The `value !== null` arm of the empty-blur commit gate
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:163-167`): the "never report a
  commit for a pristine empty field" suppression is tested for no-edit cycles
  (`specs/library/number-field/parts/input.md:26`), but the arm that *enables* the null commit
  when the field holds a value it never displayed edits for is not distinguished by any test.
- Blur sets `allowInputSyncRef.current = true` before the empty-clear branch runs
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:151` vs the cancel return at
  `packages/react/src/number-field/input/NumberFieldInput.tsx:157-159`): a canceled clear-blur
  therefore re-enables display sync while the raw text stays (the text staying is asserted,
  `specs/library/number-field/parts/input.md:25`; the sync-flag side effect is not).
- The `blockRevalidationRef` reset when validation normalizes back to the current value
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:209-213`) is a defensive branch
  for a case `useValueChanged` can't handle; no input-batch test drives a blur commit that
  normalizes to the same value (Field validation lifecycle lives in the root batch,
  `specs/library/number-field/behavior.md:37`).
- The unsupported-`type`-override stance
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:434-437`) is a documented decision
  with no test rendering the Input with a different `type`.
- The dep-less caret-restore effect
  (`packages/react/src/number-field/input/NumberFieldInput.tsx:92-98`) runs after every render and
  relies solely on the ref null-check for cost; the observable caret behavior is covered
  (`specs/library/number-field/behavior.md:37`), the every-render mechanics are an implementation
  detail no test could distinguish.
