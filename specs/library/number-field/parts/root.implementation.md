# NumberField — implementation part: root, context, stepper-button hook, barrels, type spec

Batch scope: `packages/react/src/number-field/index.ts`, `packages/react/src/number-field/index.parts.ts`,
and `packages/react/src/number-field/root/` (`NumberFieldRoot.tsx`, `NumberFieldRootContext.ts`,
`NumberFieldRootDataAttributes.ts`, `useNumberFieldStepperButton.ts`, `NumberFieldRoot.spec.tsx`).
Companion to `specs/library/number-field/behavior.md` (WHAT) and
`specs/library/number-field/parts/root.md` (root-batch WHAT) — this file is WHY/HOW only for these
files. Behavior is cited by section, not restated.

## State machine / hooks used

**Root is a two-layer state machine: a numeric contract state and a display-text state, plus four
refs that carry interaction bookkeeping between events.**

- Controlled/uncontrolled `value` is handled by `useControlled` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:110-115`);
  `useValueAsRef` mirrors it into `valueRef` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:117`)
  so step math can read full-precision numeric state inside stable callbacks without re-render
  churn — this is the mechanism behind behavior.md's "Dirty-input authority vs full-precision
  numeric state" (`specs/library/number-field/behavior.md:43`).
- The display layer is separate React state: `inputValue` initialized by formatting the value
  during the first render (SSR included) (`packages/react/src/number-field/root/NumberFieldRoot.tsx:139-144`).
  This is what makes the "Display vs value locale split" (`specs/library/number-field/behavior.md:47`)
  possible: typed text lives in `inputValue`, the numeric contract in `value`, and they only
  re-converge at commit points.
- `allowInputSyncRef` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:136`) is the
  dirty-text authority flag. While it is `false`, two gates hold: the every-render sync effect
  that reformats external `value` changes into the input bails out
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:306-318`), and `setValue` skips
  overwriting the visible text (`packages/react/src/number-field/root/NumberFieldRoot.tsx:260-266`).
  The ref is shared through context (`packages/react/src/number-field/root/NumberFieldRootContext.ts:17`);
  flipping it back to `true` at commit points (blur, wheel, stepper press —
  `packages/react/src/number-field/root/useNumberFieldStepperButton.ts:72`) is owned by the units
  that run those commits, so the exact re-sync choreography for "external value changes during
  typing" (`specs/library/number-field/behavior.md:37`) spans this batch and the input batch.
- `setValue` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:213-273`) is the single
  mutation funnel behind behavior.md's "One validation pipeline feeds every mutation source"
  (`specs/library/number-field/behavior.md:41`). It classifies the reason: anything with the
  `input-` prefix (plus `REASONS.none`, the autofill reason) is *direct text entry* and may bypass
  clamping under `allowOutOfRange`; step-based reasons always clamp
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:221-224`) — the mechanism behind
  "`allowOutOfRange` trades typing-clamp for native `rangeOverflow`, while steppers/wheel still
  clamp" (`specs/library/number-field/behavior.md:13`, `specs/library/number-field/parts/root.md:27`).
  It then runs `toValidatedNumber` with a directional step amount only when a `direction` is
  present in the details (`packages/react/src/number-field/root/NumberFieldRoot.tsx:226-236`).
- `shouldFireChange` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:241-243`) fires
  `onValueChange` for input reasons even when validation normalizes the value back to the current
  one — that is how "typing `4` with `min={5}` reports clamped `5` while the text stays `4`"
  (`specs/library/number-field/parts/root.md:26-28`) produces a callback despite an unchanged
  numeric state. A cancel veto (`details.isCanceled`) returns `false` *before* `setValueUnwrapped`
  and before `lastChangedValueRef` is refreshed
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:248-251`) — the veto semantics of
  behavior.md's "Two-callback commit model with a cancel veto" (`specs/library/number-field/behavior.md:42`).
  After an applied change, `hasPendingCommitRef` is set (`packages/react/src/number-field/root/NumberFieldRoot.tsx:255`)
  and cleared by the stable `onValueCommitted` wrapper (`packages/react/src/number-field/root/NumberFieldRoot.tsx:129-134`);
  `lastChangedValueRef` is updated unconditionally at the end of `setValue` except on the cancel
  early-return (`packages/react/src/number-field/root/NumberFieldRoot.tsx:258`).
- `incrementValue` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:275-294`) seeds an
  empty field with `0` and deliberately passes *no direction* so the seed is validated but not
  directionally snapped (`packages/react/src/number-field/root/NumberFieldRoot.tsx:280-285`); with
  a numeric current value it delegates `prevValue + amount * direction` to `setValue` with a
  `direction` detail (the `snapOnStep` input).
- `getStepAmount` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:203-211`) maps
  Alt → `smallStep`, Shift → `largeStep`, else `step` (internally `stepProp === 'any' ? 1 :
  stepProp`, `packages/react/src/number-field/root/NumberFieldRoot.tsx:96`); the raw `'any'` is
  still passed to the hidden input for native `stepMismatch` validation
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:526`) — one prop, two consumers,
  which is why "step='any' never mismatches" (`specs/library/number-field/parts/root.md:96`) and
  interactive stepping by base 1 coexist.
- `getAllowedNonNumericKeys` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:146-201`)
  derives the typeable-character set from the *formatter's* own parts (`getFormatParts`) rather
  than a hardcoded list, so multi-char currency/unit symbols and locale literals are allowed
  exactly when the format renders them; percent/permille variants are tolerated beyond what the
  formatter emits, and minus signs are gated on `minWithDefault < 0 || allowOutOfRange`
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:193-198`) so native underflow
  validation is reachable from the keyboard (`specs/library/number-field/parts/root.md:41`).
- The wheel path is a native, non-passive listener attached to the visible input because React's
  `onWheel` is passive and could not `preventDefault`
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:355-364`). The handler implements the
  focus requirement via `activeElement(ownerDocument(...))`
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:366-371`), the pinch-zoom and
  horizontal/shift axis-swap gating
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:373-383`), and treats each tick as a
  discrete commit gated on an actual change
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:389-403`) — the semantics documented
  in "Wheel preventDefault semantics" and "Rapid wheel at a boundary"
  (`specs/library/number-field/parts/root.md:81-82`, `specs/library/number-field/parts/root.md:88`).
- `focusInput` stores caret-at-end via `setSelectionRange` *before* calling `focus()`
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:341-353`) — every engine restores the
  stored selection on programmatic focus, while a consumer selection set in `onFocus` still wins
  (`specs/library/number-field/parts/root.md:50`).
- The iOS `inputmode` widening is a layout effect keyed on `minWithDefault` and gated on
  `platform.os.ios` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:320-339`), the
  mechanism for `specs/library/number-field/parts/root.md:31-32`.
- Root state exposed to attributes is a memo merging `fieldState` with local keys
  (`value`, `inputValue`, `scrubbing`, …)
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:420-431`); `isScrubbing` is Root-owned
  state (`packages/react/src/number-field/root/NumberFieldRoot.tsx:98`) flipped by the scrub unit
  through `setIsScrubbing` in context
  (`packages/react/src/number-field/root/NumberFieldRootContext.ts:30`) and surfaced as
  `data-scrubbing` (`packages/react/src/number-field/root/NumberFieldRootDataAttributes.ts:4`).

**The stepper button hook is the shared implementation for Increment/Decrement — one function,
parameterized by `isIncrement`** (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:26-33`):

- Boundary disable is derived, not prop-driven: `isAtBoundary` compares the numeric state against
  `maxWithDefault`/`minWithDefault` by direction
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:62-64`).
- `commitValue` (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:70-99`)
  syncs dirty typed text before a step: if the input is already synced it only refreshes
  `lastChangedValueRef` from `valueRef` (so a later canceled step can't commit a stale value —
  the mechanism behind "canceled steps after external changes never commit stale values",
  `specs/library/number-field/behavior.md:37`); if the input is dirty it parses the display text
  and calls `setValue` with no direction (no spurious intermediate snap before the real step,
  `packages/react/src/number-field/root/useNumberFieldStepperButton.ts:83-91`), syncing
  `valueRef` only when not canceled
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:95-97`).
- Hold-to-repeat is delegated to `usePressAndHold`: `tick` steps via `incrementValue` with the
  modifier-aware `getStepAmount(triggerEvent)`, `onStop` commits once from
  `lastChangedValueRef ?? valueRef`
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:101-118`) — the
  "stepper pointer model" (`specs/library/number-field/parts/stepper-group.md:66-72`) with the
  `??` fallback ensuring a no-op/canceled first tick still commits the current value exactly once.
- `onClick` guards `defaultPrevented`/disabled/`shouldSkipClick` (the pointerdown+click
  double-fire guard), then `commitValue` → `incrementValue` → conditional single
  `onValueCommitted` (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:130-152`).
- `onPointerDown` resets `lastChangedValueRef` to `null` as a per-hold result slot, commits dirty
  text, and focuses the input only for non-touch-like pointers
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:153-170`) — which is how
  "a primary-button `pointerdown` focuses the input (pen does not)"
  (`specs/library/number-field/parts/stepper-group.md:44-45`) is implemented: the touch/pen
  classification itself lives in `usePressAndHold`'s `isTouchLikePointerType`.
- `useButton` merges native-button semantics with `focusableWhenDisabled: true` and
  `disabled: disabled || readOnly`
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:173-180`) — the readOnly
  steppers get button-disabled semantics (hence `aria-disabled`) while `data-readonly` styling
  survives via state, and `aria-readonly` is deliberately never set
  (`specs/library/number-field/parts/root.md:60`).

The barrels are pure re-export plumbing: `index.parts.ts` maps internal names to the public
namespace (`Root`, `Group`, `Increment`, `Decrement`, `Input`, `ScrubArea`, `ScrubAreaCursor`)
(`packages/react/src/number-field/index.parts.ts:1-7`) and `index.ts` re-exports `NumberField`
plus every part's type namespace (`packages/react/src/number-field/index.ts:1-9`).

## Context providers/consumers

- `NumberFieldRootContext` is provided once, wrapping *both* the rendered root element and the
  hidden input (`packages/react/src/number-field/root/NumberFieldRoot.tsx:491-535`), with a
  memoized value (`packages/react/src/number-field/root/NumberFieldRoot.tsx:433-482`). Its shape
  (`packages/react/src/number-field/root/NumberFieldRootContext.ts:8-36`) is deliberately
  ref-heavy (`valueRef`, `allowInputSyncRef`, `lastChangedValueRef`, `hasPendingCommitRef`,
  `formatOptionsRef`, `inputRef`) so consumer units read current interaction state inside event
  handlers without prop-drilling or tearing across renders.
- The consumer hook `useNumberFieldRootContext` throws the "must be placed within
  `<NumberField.Root>`" error on a missing provider
  (`packages/react/src/number-field/root/NumberFieldRootContext.ts:42-51`) — the context-misuse
  error contract asserted for `Input`/`ScrubAreaCursor` in behavior.md
  (`specs/library/number-field/behavior.md:9`).
- `useNumberFieldStepperButton` consumes it for bounds, the shared refs, the mutation funnel
  (`setValue`/`incrementValue`/`getStepAmount`), `focusInput`, `id` (used as `aria-controls`),
  and `state` (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:43-59`).
- Root itself consumes two upstream Base UI contexts: `useFieldRootContext` for Field integration
  (`setDirty`, `validityData`, `disabled`, `setFilled`, `name`, `state`, `validation` —
  `packages/react/src/number-field/root/NumberFieldRoot.tsx:83-92`; the `data-*` Field attributes
  contract is `specs/library/number-field/parts/root.md:65`) and `useFormContext` for
  `clearErrors` on autofill changes (`packages/react/src/number-field/root/NumberFieldRoot.tsx:92,513`).
- `useLabelableId` provides the id shared with `Field.Label`'s `for` linking
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:108`; behavior:
  `specs/library/number-field/parts/root.md:64`), and `validation.getValidationProps` on the
  hidden input wires `aria-invalid` and server-error focus
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:495-516`; behavior:
  `specs/library/number-field/parts/root.md:52`, `specs/library/number-field/parts/root.md:63`).
- `NumberFieldRootDataAttributes.ts` declares the state→attribute vocabulary (`data-scrubbing`,
  `data-disabled`, `data-readonly`, `data-required`, validity/touched/dirty/filled/focused —
  `packages/react/src/number-field/root/NumberFieldRootDataAttributes.ts:1-40`); the mapping from
  state keys to attributes is the shared `stateAttributesMapping`
  (`packages/react/src/number-field/utils/stateAttributesMapping.ts:5-9`), which nulls
  `value`/`inputValue` (never rendered as attributes) and spreads `fieldValidityMapping`.

## DOM/portal strategy and why

- Root renders a `div` through `useRenderElement` with the merged state and
  `stateAttributesMapping` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:484-489`) —
  the conformance-asserted element (`specs/library/number-field/parts/root.md:7`).
- There are **no portals** in this batch. The notable DOM decision is the *second* element Root
  renders: a hidden native `input[type=number]` placed as a sibling of the root element inside the
  provider (`packages/react/src/number-field/root/NumberFieldRoot.tsx:491-535`). It exists for
  three jobs at once: form submission (`name`, `form`, `min`, `max`, raw `step`, `required`,
  native validity like `stepMismatch`/`rangeOverflow`), browser autofill (its `onChange` is the
  autofill entry point, with a React #9023 workaround checking `event.nativeEvent.defaultPrevented`
  — `packages/react/src/number-field/root/NumberFieldRoot.tsx:499-515`), and Field validation
  (`validation.getValidationProps`, `validation.inputRef` merged into `hiddenInputRef`,
  `validation.change` — `packages/react/src/number-field/root/NumberFieldRoot.tsx:106,495,514`).
  It is `aria-hidden` + `tabIndex={-1}` + visually hidden styles so it never receives direct
  interaction, and its `onFocus` forwards to the visible input via `focusInput`
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:496-498,517-533`) — the hidden-input
  focus-forwarding behavior (`specs/library/number-field/parts/root.md:49-53`). Tests only assert
  it is reachable from `document`, not its position (`specs/library/number-field/parts/root.md:71`);
  in source it always renders after the root element, unconditionally.
- Steppers render native `<button>` elements through the same `useRenderElement` pipeline, with
  `aria-label` "Increase"/"Decrease", `aria-controls` → input id, and `tabIndex={-1}` with the
  rationale that keyboard users operate the input directly while touch screen readers can still
  reach the buttons (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:120-189`;
  behavior: `specs/library/number-field/parts/root.md:59-60`).
- Focus routing converges on the visible input through the shared `inputRef`: hidden-input focus
  forwarding, stepper pointerdown, and the wheel focus check all go through
  `focusInput`/`activeElement` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:345-353,496-498`;
  `specs/library/number-field/behavior.md:46`).

## Dependencies on other Base UI internals

Imports cited by path (internals not re-derived here):

- `packages/utils/` — `addEventListener` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:3`),
  `useControlled` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:4`),
  `useStableCallback` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:5`),
  `useIsoLayoutEffect` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:6`),
  `useValueAsRef` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:7,125`),
  `useForcedRerendering` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:8,123`),
  `useMergedRefs` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:9`),
  `visuallyHidden`/`visuallyHiddenInput` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:10,532`),
  `ownerDocument` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:11,368`),
  `platform` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:12,322`),
  `formatNumber` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:13,143,265,313`).
- `packages/react/src/floating-ui-react/utils` — `activeElement`
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:14,368`).
- `packages/react/src/internals/` — `FieldRootContext` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:16`),
  `FormContext` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:17`),
  `FieldRoot` state type (`packages/react/src/number-field/root/NumberFieldRoot.tsx:18`),
  `useLabelableId` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:19`),
  `useRenderElement` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:22`;
  `packages/react/src/number-field/root/useNumberFieldStepperButton.ts:4`),
  `createBaseUIEventDetails` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:36-42`;
  `packages/react/src/number-field/root/useNumberFieldStepperButton.ts:8-11`),
  `REASONS` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:43`;
  `packages/react/src/number-field/root/useNumberFieldStepperButton.ts:14`),
  `use-button` (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:5`),
  `usePressAndHold` + `isTouchLikePointerType` (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:6`),
  shared component types (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:3`).
- Own-unit files outside this batch (the utils batch) — `parse` helpers
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:23-32`;
  `parseNumber` at `packages/react/src/number-field/root/useNumberFieldStepperButton.ts:7`),
  `toValidatedNumber` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:33`),
  event-details types (`packages/react/src/number-field/root/NumberFieldRoot.tsx:34-35`;
  `packages/react/src/number-field/root/useNumberFieldStepperButton.ts:12`),
  and `stateAttributesMapping` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:21,488`;
  `packages/react/src/number-field/root/useNumberFieldStepperButton.ts:16,188`).
- This unit's `TODO.md` entry has no `wraps-external:` field (`TODO.md:440-447`), so no
  third-party package delegation applies.

## Anything in source not explained by any test

- `NumberFieldRoot.spec.tsx` is a **type-only** spec with no runtime assertions: it pins the
  discriminated-union narrowing of event details (wheel → `WheelEvent` and explicitly not
  `PointerEvent`; `incrementPress` → `PointerEvent | MouseEvent | TouchEvent`; `inputClear` →
  `InputEvent | FocusEvent | Event` and not `KeyboardEvent`; commit `scrub` → `PointerEvent`;
  handler value always `number | null`)
  (`packages/react/src/number-field/root/NumberFieldRoot.spec.tsx:14-66`). No behavior test covers
  these compile-time guarantees.
- `hasPendingCommitRef` is written (`packages/react/src/number-field/root/NumberFieldRoot.tsx:255`)
  and cleared (`packages/react/src/number-field/root/NumberFieldRoot.tsx:131`) in this batch but
  never read here — its reader is another unit via context
  (`packages/react/src/number-field/root/NumberFieldRootContext.ts:21`), and no test in this
  batch's files exercises its lifecycle.
- The SSR/hydration strategy — formatting the initial `inputValue` on the server and suppressing
  the expected locale-mismatch warning via `suppressHydrationWarning`
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:139-143,533`) — is a documented
  trade-off with no test coverage in this batch.
- The hidden-input style switch between `visuallyHiddenInput` (when `name` is set) and
  `visuallyHidden` otherwise (`packages/react/src/number-field/root/NumberFieldRoot.tsx:532`) is
  untested; the tests only assert the input exists and carries form attributes
  (`specs/library/number-field/parts/root.md:71`).
- Stepper cosmetics/semantics with no batch assertions: the `SELECT_NONE_STYLE` user-select lock
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:18-21,128`), the
  `tabIndex={-1}` keyboard-exclusion rationale
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:124-127`), and
  `focusableWhenDisabled` on `useButton`
  (`packages/react/src/number-field/root/useNumberFieldStepperButton.ts:179`).
- The wheel effect's dependency array lists `lastChangedValueRef`/`valueRef`
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:408-417`) even though refs are
  stable — inert entries, re-runs are driven by the real deps only.
- Classifying autofill as `REASONS.none` *and* treating `none` as a direct-entry (`input-`-class)
  reason for the `allowOutOfRange` clamp decision
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:221-224,508`) is not asserted
  anywhere: the asserted reason list (`specs/library/number-field/parts/root.md:77-78`) never
  includes `'none'`.
- The compact-notation exclusion in `getAllowedNonNumericKeys` — rejecting `K`/`M` suffixes
  because `parseNumber` can't reverse them
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:162-165`) — is a comment-documented
  decision; this batch's tests exercise accepted symbols but never the compact exclusion itself.
- The public `inputRef` prop resolves to the *hidden* input, not the visible textbox
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:78,106`; JSDoc at
  `packages/react/src/number-field/root/NumberFieldRoot.tsx:663-666`) — no test in this batch's
  files asserts which element the forwarded ref lands on.
- `minWithZeroDefault = min ?? 0` (`packages/react/src/number-field/root/NumberFieldRoot.tsx:102`)
  is passed to `toValidatedNumber` as the zero-anchored bound
  (`packages/react/src/number-field/root/NumberFieldRoot.tsx:231`); no root-batch test
  distinguishes that anchor choice — its semantics live in the validate util (utils batch).
