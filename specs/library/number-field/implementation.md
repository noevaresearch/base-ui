# NumberField — implementation spec (Stage 2: implementation mining)

Ground truth for WHAT happens is `specs/library/number-field/behavior.md` (with per-batch depth in
`parts/*.md`); this document explains the state machine, hook composition, context usage, and DOM
decisions that produce it. Source files (all non-test files under
`packages/react/src/number-field/`):

- `root/NumberFieldRoot.tsx`, `root/NumberFieldRootContext.ts`, `root/useNumberFieldStepperButton.ts`
- `input/NumberFieldInput.tsx`, `increment/NumberFieldIncrement.tsx`, `decrement/NumberFieldDecrement.tsx`
- `group/NumberFieldGroup.tsx`, `scrub-area/NumberFieldScrubArea.tsx`, `scrub-area/NumberFieldScrubAreaContext.ts`
- `scrub-area-cursor/NumberFieldScrubAreaCursor.tsx`
- `utils/{parse,validate,getViewportRect,stateAttributesMapping,types,constants}.ts`
- `index.ts` / `index.parts.ts` (barrels), six `*DataAttributes.ts` constants files
- `root/NumberFieldRoot.spec.tsx`, `input/NumberFieldInput.spec.tsx` (compile-time type specs — see gaps)

The unit's TODO entry (`library: number-field`, `TODO.md:440-447`) has no `wraps-external:` field —
only `crate: leptos-ui`, `blocked-by`, `needs-batched-mining`, and `docs-pair`. Nothing is delegated
to an external npm package, so every mechanism below is derived from first-party source (consistent
with behavior.md's front matter).

## State machine / hooks used

NumberField has **no reducer/store**. The "state machine" is a set of plain `React.useState` values
plus six **shared mutable refs** that arbitrate between the authoritative numeric state and the
visible text. Understanding the refs is understanding the component; they are all published on the
root context (see *Context providers/consumers*) precisely so that child event handlers can read
and write them without re-rendering anything.

### The arbitration refs (all allocated in `NumberFieldRoot`)

| Ref | Allocated | Written | Read |
| --- | --- | --- | --- |
| `inputRef` | `packages/react/src/number-field/root/NumberFieldRoot.tsx:105` | assigned by `Input`'s merged ref (`packages/react/src/number-field/input/NumberFieldInput.tsx:455-460`) | `focusInput` (`NumberFieldRoot.tsx:345-353`), wheel listener (`NumberFieldRoot.tsx:357-418`), stepper/scrub focus + caret restore |
| `allowInputSyncRef` | `NumberFieldRoot.tsx:136` | `false` while the user types/pastes (`NumberFieldInput.tsx:234`, `:447`); reset `true` on blur, step keys, wheel, scrub (`NumberFieldInput.tsx:151`, `:381-383`; `NumberFieldRoot.tsx:387`; `NumberFieldScrubArea.tsx:223`); `true` during hold commits (`useNumberFieldStepperButton.ts:71-72`) | gates the `value`→text sync effect (`NumberFieldRoot.tsx:306-318`) and the text rewrite in `setValue` (`NumberFieldRoot.tsx:264-266`) |
| `hasPendingCommitRef` | `NumberFieldRoot.tsx:127` | `true` after `setValue` applies a change (`NumberFieldRoot.tsx:255`) | cleared by the `onValueCommitted` wrapper (`NumberFieldRoot.tsx:129-134`); read by `Input.onBlur` to decide whether a programmatic change still needs a commit (`NumberFieldInput.tsx:149`, `:196`) |
| `lastChangedValueRef` | `NumberFieldRoot.tsx:137` | `setValue` records the validated result (`NumberFieldRoot.tsx:258`); refreshed to `valueRef.current` before steps that might be canceled (`useNumberFieldStepperButton.ts:79`, `NumberFieldInput.tsx:388-390`); reset to `null` per hold (`useNumberFieldStepperButton.ts:162`) | every commit call site commits *this*, not the pre-validation value: keyboard (`NumberFieldInput.tsx:405`), wheel (`NumberFieldRoot.tsx:399-403`), stepper click/stop (`useNumberFieldStepperButton.ts:115-116`, `:148-151`), scrub pointerup (`NumberFieldScrubArea.tsx:166-169`) |
| `valueRef` | `NumberFieldRoot.tsx:117` via `useValueAsRef` | mirrored from `value`; manually refreshed after a non-canceled dirty-input commit (`useNumberFieldStepperButton.ts:95-97`) | step math base for `incrementValue` (`NumberFieldRoot.tsx:277`) and boundary disabling (`useNumberFieldStepperButton.ts:62-64`) |
| `formatOptionsRef` | `NumberFieldRoot.tsx:125` via `useValueAsRef` | mirrored from `format` | formatting/parsing inside event handlers without effect re-runs |

The mechanism behind behavior.md's *Whole-unit cross-cutting behavior* pin "dirty-input authority
vs full-precision numeric state": typed text lives in `inputValue` state while `allowInputSyncRef`
is `false`, and step interactions branch on `hadManualInput = !allowInputSyncRef.current` — parsing
the *text* only when it is dirty (`NumberFieldInput.tsx:280`, `:364-370`;
`useNumberFieldStepperButton.ts:70-99`), otherwise stepping from `valueRef`'s full-precision number.
`hasPendingCommitRef` + `lastChangedValueRef` implement the two-callback commit model: continuous
mutations arm the pending flag, and the discrete endpoint commits exactly the last validated value
(or the fallback current value when every tick was a no-op/cancel — the `?? valueRef.current`
fallbacks cited above).

### Hook call sites

**Root (`root/NumberFieldRoot.tsx`):**

| Hook | Call site | Role |
| --- | --- | --- |
| `useFieldRootContext` / `useFormContext` | `NumberFieldRoot.tsx:83-92` | Field state merge (`disabled` at `:94`, `name` at `:95`), `setDirty`/`setFilled`, `clearErrors`, `validation` |
| `useControlled` | `NumberFieldRoot.tsx:110-115` | controlled/uncontrolled `value` (the *State model* contract) |
| `useValueAsRef` | `NumberFieldRoot.tsx:117`, `:125` | `valueRef`, `formatOptionsRef` |
| `useIsoLayoutEffect` | `NumberFieldRoot.tsx:119-121` | `setFilled(value !== null)` before paint |
| `useForcedRerendering` | `NumberFieldRoot.tsx:123` (called at `:269`) | re-render when formatting changes without a numeric change |
| `useStableCallback` | `NumberFieldRoot.tsx:129`, `:146`, `:203`, `:213`, `:275`, `:345` | `onValueCommitted`, `getAllowedNonNumericKeys`, `getStepAmount`, `setValue`, `incrementValue`, `focusInput` |
| `React.useState` | `NumberFieldRoot.tsx:98`, `:143`, `:144` | `isScrubbing`; `inputValue` (lazy-initialized to `formatNumber(value, …)` for SSR); `inputMode` |
| `React.useRef` | `NumberFieldRoot.tsx:105-137` | the arbitration refs table above |
| `useIsoLayoutEffect` | `NumberFieldRoot.tsx:306-318` | the single `value`→`inputValue` sync point (gated on `allowInputSyncRef`) |
| `useIsoLayoutEffect` | `NumberFieldRoot.tsx:320-339` | iOS `inputMode` widening (behavior.md *Accessibility*) |
| `React.useEffect` | `NumberFieldRoot.tsx:357-418` | non-passive native `wheel` listener on the visible input (React's synthetic wheel is passive, so `preventDefault` requires the native listener — see *DOM/portal strategy*) |
| `React.useMemo` | `NumberFieldRoot.tsx:420-431`, `:433-482` | `state` object and context value |
| `useRenderElement` | `NumberFieldRoot.tsx:484-489` | renders the root `div` with `stateAttributesMapping` |

`setValue` (`NumberFieldRoot.tsx:213-273`) is the single validation funnel: it classifies the
reason as direct-entry (`input-` prefix or `none`, `:221`) vs step-based to decide clamping under
`allowOutOfRange` (`:224`), runs `toValidatedNumber` (`:226-236`), fires `onValueChangeProp` and
honors the `details.isCanceled` veto (`:245-251`, returning `false` so callers skip paired
commits), applies via `setValueUnwrapped` + `setDirty` + pending flag (`:253-255`), syncs formatted
text only when allowed (`:264-266`), and forces a re-render (`:269`). `incrementValue`
(`:275-294`) seeds a `null` field with `0` (no direction, so it is never directionally snapped,
`:280-285`) or adds `amount * direction`.

**Input (`input/NumberFieldInput.tsx`):** `useNumberFieldRootContext` (`:56-77`),
`useFormContext`/`useFieldRootContext`/`useLabelableContext` (`:80-83`),
`useRegisterFieldControl(inputRef, …)` (`:88`), a `useIsoLayoutEffect` caret restore for pastes
driven by `pendingCaretRef` (`:85`, `:92-98`), and `useValueChanged(value, …)` (`:100-109`) which
fires Field `clearErrors` + `validation.change` on actual value transitions, with
`blockRevalidationRef` suppressing the redundant revalidation after the blur commit's own
`setValue` (`:85`, `:103-106`, `:202`, `:208-213`). All behavior lives in one `inputProps` object
(`:111-453`): `onFocus` (`:128-135`), `onBlur` commit/reformat pipeline (`:136-227`), `onChange`
character-gated parse (`:228-268`), `onKeyDown` stepping (`:269-407`), `onPaste` splice
(`:408-452`). It renders via `useRenderElement('input', …)` merging
`[inputProps, elementProps, validation.getValidationProps]` (`:455-460`).

**Steppers (`increment/`, `decrement/`, shared `root/useNumberFieldStepperButton.ts`):**
`NumberFieldIncrement`/`NumberFieldDecrement` are 1-line wrappers that delegate to the shared hook
with an `isIncrement` boolean (`packages/react/src/number-field/increment/NumberFieldIncrement.tsx:13-18`,
`packages/react/src/number-field/decrement/NumberFieldDecrement.tsx:13-18`) — the hook differs the
two only by direction and the boundary checked (`useNumberFieldStepperButton.ts:29-64`, disabled at
`value >= max` / `value <= min`, `:62-64`). Its own hooks: `usePressAndHold`
(`useNumberFieldStepperButton.ts:101-118`) supplies the hold auto-repeat state machine — immediate
`tick` on press, 400 ms warm-up then 60 ms repeat ticks (constants
`packages/react/src/number-field/utils/constants.ts:1-2` passed nowhere: the hook defaults match,
`packages/react/src/internals/usePressAndHold.ts:10-14`), a `{ once: true }` window `pointerup`
listener calling `onStop` (`usePressAndHold.ts:139-148`), touch-intent detection with a
50 ms/3-moves grace (`usePressAndHold.ts:206-222`), and mouseleave/mouseenter pause/resume
(`usePressAndHold.ts:246-265`) — plus `shouldSkipClick` for the double-fire guard
(`usePressAndHold.ts:275-283`). `tick` steps (`useNumberFieldStepperButton.ts:104-111`); `onStop`
commits (`:112-117`). `useButton` (`:173-180`) provides native-button semantics with
`focusableWhenDisabled` and the readOnly→`disabled` mapping (behavior.md *Accessibility*).
`onClick` (`:130-152`) and `onPointerDown` (`:153-170`) both run `commitValue` first — the
dirty-text sync (parse + `setValue` with no direction, refreshing `valueRef` only when not
canceled, `:70-99`) — then one step; `focusInput` is gated to non-touch-like pointers
(`:164-167`, `isTouchLikePointerType` at
`packages/react/src/internals/usePressAndHold.ts:19-21`, which also treats pen as touch).

**ScrubArea (`scrub-area/NumberFieldScrubArea.tsx`):** `useTimeout` (`:69`) for the Gecko
pointer-lock release delay; `useStableCallback` for `onScrub` (`:82-123`, cursor wrap math) and
`onScrubbingChange` (`:125-146`, which wraps its two state updates in `ReactDOM.flushSync` so the
cursor element exists before the first `pointermove` transform, `:127-130`); `React.useState` for
`isTouchInput`/`isPointerLockDenied`/`isScrubbing` (`:71-73`). One `React.useEffect`
(`:148-261`) registers the capture-phase window `pointerup`/`pointermove` listeners only while
scrubbing (the `isScrubbingRef` ref, not the state, is the truth for "pointer actually down",
`:202-204`): `pointermove` accumulates a `cumulativeDelta` and steps only past `pixelSensitivity`
(`:213-229`), and `pointerup` exits pointer lock, clears scrub state, commits
(`:157-196`), deferring 20 ms on Gecko (`:189-195`), and synthesizes a `click` on the
original `pointerdown` target when nothing moved (the browser click was suppressed by
`preventDefault`, `:171-182`). Two more effects: unmount-mid-scrub cleanup (`:265-278`) and the
single-touch `touchstart` preventDefault (`:281-297`). The `async onPointerDown`
(`:302-339`) does the mouse-only `preventDefault` + `focusInput` (`:310-313`) and the
pointer-lock request/denial (`:320-338`), re-emitting scrub state in a `finally` after the
await resolves (`:330-337`).

**ScrubAreaCursor:** stateless besides `domElement` (`:39`); renders conditionally via
`useRenderElement(…, { enabled: shouldRender })`
(`packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.tsx:41-56`) and
registers its DOM node into the parent-owned `scrubAreaCursorRef` (`:46`).

**Group (`group/NumberFieldGroup.tsx`):** stateless; reads only `state` (`:21`) and renders
`role: 'group'` via `useRenderElement` (`:23-28`).

**Utils:** pure functions — `parseNumber`/`getFormatParts`/`getNumberLocaleDetails`
(`packages/react/src/number-field/utils/parse.ts:107-221`, `:75-77`, `:79-105`),
`toValidatedNumber`/`removeFloatingPointErrors`/`snapToStep`
(`packages/react/src/number-field/utils/validate.ts:92-136`, `:33-75`, `:77-90`),
`getViewportRect` (`packages/react/src/number-field/utils/getViewportRect.ts:4-34`),
`isNumeralChar` (`parse.ts:33-35`), and the character-class constants the Root/Input handlers
consume (`parse.ts:13-64`). `toValidatedNumber` is the ordered pipeline behind behavior.md's
*State model* validation pin: snap (directional, or nearest for alt/smallStep; base is `min ?? 0`
when unbounded, `validate.ts:109-116` with `:111`) → clamp (`:121-123`) →
floating-point/format rounding (`:134`) → re-clamp (`:135`), with a no-op early return for
non-stepping, non-rounding values so parsed input keeps full precision (`:130-132`).

## Context providers/consumers

Two React contexts are owned by this unit:

1. **`NumberFieldRootContext`** (`packages/react/src/number-field/root/NumberFieldRootContext.ts:8-36`,
   created `:38-40`, provided at `NumberFieldRoot.tsx:491-492` wrapping both the rendered `div`
   and the hidden input). It carries: the six arbitration refs; the stable callbacks
   (`setValue`, `incrementValue`, `getStepAmount`, `focusInput`, `onValueCommitted`,
   `getAllowedNonNumericKeys`, `setInputValue`, `setIsScrubbing`); resolved config
   (`minWithDefault`/`maxWithDefault`/`min`/`max`, `id`, `name` + raw `nameProp`, `inputMode`,
   `locale`); and the memoized `state` object. Consumers, all through the throwing accessor
   `useNumberFieldRootContext` (`NumberFieldRootContext.ts:42-51` — the context-misuse error
   asserted in behavior.md's *Public API surface*):
   - `Group` — reads `state` only (`NumberFieldGroup.tsx:21`);
   - `Input` — reads the most (`NumberFieldInput.tsx:56-77`);
   - steppers — via `useNumberFieldStepperButton` (`useNumberFieldStepperButton.ts:43-59`);
   - `ScrubArea` (`NumberFieldScrubArea.tsx:47-58`);
   - `ScrubAreaCursor` — reads only `state` (`NumberFieldScrubAreaCursor.tsx:35`).
2. **`NumberFieldScrubAreaContext`**
   (`packages/react/src/number-field/scrub-area/NumberFieldScrubAreaContext.ts:4-13`, provided at
   `NumberFieldScrubArea.tsx:349-363`, consumed only by the cursor
   `NumberFieldScrubAreaCursor.tsx:36-37` through the throwing
   `useNumberFieldScrubAreaContext`, `ScrubAreaContext.ts:15-23`). It flows
   `isScrubbing`/`isTouchInput`/`isPointerLockDenied` down for the cursor's render gate and,
   notably, carries `scrubAreaCursorRef` **downward** so the child can register its DOM node back
   into the parent — the parent's `onScrub`/`onScrubbingChange` manipulate the cursor element
   imperatively (`NumberFieldScrubArea.tsx:75-80`, `:132-144`) without ever re-rendering it.

Cross-boundary (consumed, not owned): `useFieldRootContext` (Root `:83-91`; Input `:81-82`) merges
Field state and exposes `validation`; `useFormContext` (Root `:92`; Input `:80`) for `clearErrors`;
`useLabelableContext` (Input `:83`) for `labelId` → `aria-labelledby`. Data flows one way (config
and commands down); the only upward signals are the user callbacks (`onValueChange`/
`onValueCommitted`) and Field mutations (`setDirty`, `setFilled`, `setTouched`, `setFocused`).

Because every callback is `useStableCallback`-wrapped and refs are shared, child handlers observe
live values (`valueRef`, `allowInputSyncRef`) directly instead of stale closures — this is what
makes behavior.md's rapid/interleaved edge cases (external change mid-typing, canceled steps after
external changes) correct without re-render churn.

## DOM/portal strategy and why

- **Root `div` + hidden sibling input.** `useRenderElement('div', …)` renders the shell
  (`NumberFieldRoot.tsx:484-489`); the provider wraps the div *and* a sibling hidden
  `<input type="number">` rendered after it (`:491-535`). The split-input strategy is the core DOM
  decision: the visible `NumberField.Input` is `type="text"` (`NumberFieldInput.tsx:118`) because
  locale-aware characters, IME, caret control, and paste splicing are impossible with native
  `type=number`; the hidden number input exists purely for the platform — `name`/`form`/`required`
  for FormData (`NumberFieldRoot.tsx:519-520`, `:527-529`), native `min`/`max`/`step` for
  `stepMismatch`/`rangeOverflow` validation (`:522-526`, with `step="any"` passed through raw even
  though interactive stepping uses 1, `:96` vs `:526`), and browser autofill (its `onChange`
  at `:499-515`). It is `aria-hidden` + `tabIndex={-1}` (`:530-531`), styled with
  `visuallyHiddenInput` when named and plain `visuallyHidden` otherwise (`:532`), and
  `suppressHydrationWarning` guards the server/client locale mismatch (`:533`; same flag on the
  visible input, `NumberFieldInput.tsx:127`).
- **Focus is routed, never trapped.** `focusInput` pre-sets the caret to the end via
  `setSelectionRange` before `focus()` because programmatic focus lands the caret at the start
  (Chrome/Firefox) or selects all (Safari) (`NumberFieldRoot.tsx:341-353`) — the mechanism behind
  behavior.md's *Focus management* pins. The hidden input forwards its own `onFocus` to
  `focusInput` (`:496-498`).
- **One portal, cursor only.** `ScrubAreaCursor` is the only portaled element:
  `ReactDOM.createPortal(element, ownerDocument(domElement).body)`
  (`NumberFieldScrubAreaCursor.tsx:58`). It is `position: fixed` at the top-left corner
  (`:13-18`) and positioned purely with `style.transform` translate3d + `scale(1/visualViewport.scale)`
  (`NumberFieldScrubArea.tsx:75-80`) — imperative transform updates move the cursor without React
  renders — and the portal escapes any transformed/overflow-clipped ancestor while the OS pointer
  is locked. Everything else renders inline (behavior.md, *DOM structure & portal behavior*).
  The cursor's existence is itself conditional: `enabled: shouldRender` in `useRenderElement`
  (`NumberFieldScrubAreaCursor.tsx:41-56`; the `enabled` flag suppresses state attributes, refs,
  and props at `packages/react/src/internals/useRenderElement.tsx:65-104`), so it unmounts on
  pointerup as pinned in behavior.md.
- **Window listeners, not portals, for transient gestures.** Scrub attach capture-phase
  `pointermove`/`pointerup` on `ownerWindow(inputRef.current)` while scrubbing
  (`NumberFieldScrubArea.tsx:233-237`); holds register a `{ once: true }` window `pointerup`
  (`usePressAndHold.ts:139-148`); wheel uses a native non-passive listener on the visible input
  because React's synthetic wheel is passive and cannot `preventDefault`
  (`NumberFieldRoot.tsx:355-406`).
- **Stepper semantics.** Native `<button>`s with `aria-label` "Increase"/"Decrease",
  `aria-controls={id}`, `tabIndex={-1}` (keyboard users step on the input; touch screen readers
  still reach them — the source comment at `useNumberFieldStepperButton.ts:124-127`),
  and a `user-select: none` + `-webkit-user-select: none` style (`:18-21`, `:128`).
  `useButton` merges `getButtonProps` into the render (`:184-189`), mapping readOnly to
  disabled-but-focusable (`:173-180`).
- **ScrubArea is a presentation span.** `role: 'presentation'` plus `touchAction: 'none'` and
  both `user-select` spellings (`NumberFieldScrubArea.tsx:21-25`, `:300-301`) so touch drags don't
  scroll and text under the drag isn't selected (the move handler also preventDefaults,
  `:206-207`). The cursor span is likewise `role: 'presentation'`
  (`NumberFieldScrubAreaCursor.tsx:50`).
- **State attributes via mapping.** Every part passes `stateAttributesMapping`
  (`packages/react/src/number-field/utils/stateAttributesMapping.ts:5-9`), which nulls out
  `value`/`inputValue` (they must never become `data-*` attributes) and spreads
  `fieldValidityMapping` — the machinery producing the `data-scrubbing`/`data-disabled`/… surface
  declared in the six identical `*DataAttributes.ts` files (e.g.
  `packages/react/src/number-field/root/NumberFieldRootDataAttributes.ts:1-40`).

## Dependencies on other Base UI internals

Direct imports, by package/module (import lines are inside this unit's files):

- **`@base-ui/utils/*` (`packages/utils`):** import lines `NumberFieldRoot.tsx:3-13` cover
  `addEventListener`, `useControlled`, `useStableCallback`, `useIsoLayoutEffect`,
  `useValueAsRef`, `useForcedRerendering`, `useMergedRefs`, `visuallyHidden`(+Input),
  `ownerDocument` (`owner`), `platform`, `formatNumber`. Additionally: `ownerWindow`/`mergeCleanups`
  (`NumberFieldScrubArea.tsx:4-8`), `platform` (`NumberFieldScrubAreaCursor.tsx:4`),
  `useTimeout` (`NumberFieldScrubArea.tsx:9`; internally `usePressAndHold.ts:5`),
  `useInterval` (`usePressAndHold.ts:6`), `clamp` (`utils/validate.ts:1`),
  `getFormatter` (`utils/parse.ts:1`), `warn` + `SafeReact` (`NumberFieldInput.tsx:3-4`),
  `NOOP`/`EMPTY_OBJECT` from `empty` (`usePressAndHold.ts:4`,
  `packages/react/src/internals/createBaseUIEventDetails.ts:1`).
- **Vendored `floating-ui-react` utils (in-repo, not the external package):** `activeElement`
  (`NumberFieldRoot.tsx:14`, used for the wheel focus guard `:368`) and `getTarget`
  (`NumberFieldScrubArea.tsx:19`, used `:317`). Shadow-DOM-safe traversal per repo convention.
- **`internals/`:** `useRenderElement` + `getStateAttributesProps` types
  (Root `:22`; Input `:25`; Group `:7`; ScrubArea `:15`; Cursor `:11`; stepper `:4`;
  `stateAttributesMapping` type via `utils/stateAttributesMapping.ts:1`), `use-button`
  (`useNumberFieldStepperButton.ts:5`), `usePressAndHold` + `isTouchLikePointerType`
  (`useNumberFieldStepperButton.ts:6`),
  `createBaseUIEventDetails` (Root `:36-42`; Input `:26-29`; stepper `:8-11`; ScrubArea `:17`)
  — the `cancel()`/`isCanceled` veto object consumed by `setValue`
  (`createBaseUIEventDetails.ts:118-149`) and the generic details for commits (`:151-166`) —
  `reasons` (`REASONS`: Root `:43`; Input `:31`; stepper `:14`; ScrubArea `:18`;
  `utils/types.ts:1`), `field-root-context` (Root `:16`; Input `:9`), `form-context`
  (Root `:17`; Input `:11`), `labelable-provider` (`useLabelableId` Root `:19`;
  `useLabelableContext` Input `:12`), `field-register-control` (Input `:10`),
  `useValueChanged` (Input `:30`; implementation
  `packages/react/src/internals/useValueChanged.ts:6-17`), `types`
  (`BaseUIComponentProps`/`NativeButtonProps`/`HTMLProps`), and the `FieldRootState` type
  (Root `:18`) that `NumberFieldRootState` extends (`NumberFieldRoot.tsx:669`).
- **Intra-unit graph:** Root ← context ← {Group, Input, steppers, ScrubArea, Cursor};
  ScrubArea ← scrub context ← Cursor; steppers ← stepper hook + `usePressAndHold`;
  everything ← `utils/parse.ts` + `utils/validate.ts` (+ `utils/getViewportRect.ts` for ScrubArea,
  `utils/constants.ts` referenced by tests and matching `usePressAndHold` defaults,
  `utils/stateAttributesMapping.ts` for all renderers, `utils/types.ts` for parameter shapes).
- **Barrels:** `index.parts.ts:1-7` aliases the seven parts (`Root`, `Group`, `Increment`,
  `Decrement`, `Input`, `ScrubArea`, `ScrubAreaCursor`); `index.ts:1-9` namespaces them as
  `NumberField` and re-exports all part types.
- **Not used:** `use-render` (render props are handled by `useRenderElement`), portals other than
  the cursor, `useAnimationFrame`, anchor positioning, `DirectionProvider`, and
  `details.allowPropagation`/`isPropagationAllowed` (exist on the details object but are never
  consulted in this unit — only `cancel`/`isCanceled` matter, see gaps).

For `ralph/scripts/generate-todo.mjs` dependency computation, number-field's precise internal
surface is: `internals/useRenderElement` (+ `getStateAttributesProps`), `internals/use-button`,
`internals/usePressAndHold`, `internals/createBaseUIEventDetails`, `internals/reasons`,
`internals/field-root-context`, `internals/form-context`, `internals/labelable-provider`,
`internals/field-register-control`, `internals/useValueChanged`, `internals/types`,
`internals/field-constants` (via `stateAttributesMapping`), the vendored
`floating-ui-react/utils` (`activeElement`, `getTarget`), and the `@base-ui/utils` members listed
above (`useControlled`, `useStableCallback`, `useIsoLayoutEffect`, `useValueAsRef`,
`useForcedRerendering`, `useMergedRefs`, `visuallyHidden`, `owner`, `platform`, `formatNumber`,
`addEventListener`, `mergeCleanups`, `useTimeout`, `useInterval`, `clamp`, `warn`, `safeReact`,
`empty`).

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit; each item is
present in source but absent from behavior.md's mined coverage:

1. **`Escape` is a navigation key.** `NAVIGATE_KEYS` includes `'Escape'`
   (`NumberFieldInput.tsx:41`), so Escape is never preventDefault-ed and leaves the value alone.
   behavior.md's *Keyboard interactions* lists Backspace/Delete/ArrowLeft/ArrowRight/Tab/Enter only.
2. **`aria-roledescription="Number field"`** on the visible input (`NumberFieldInput.tsx:122`),
   overridable via a documented prop (`:467-478`). behavior.md's *Accessibility* never mentions it.
3. **`aria-labelledby={labelId}`** (`NumberFieldInput.tsx:124`). behavior.md covers `Field.Label`
   linking via `for` and `Description` via `aria-describedby`, but not the input-side labelledby.
4. **Input hardening attributes:** `type="text"`, `autoComplete="off"`, `autoCorrect="off"`,
   `spellCheck="false"` (`NumberFieldInput.tsx:118-121`) are unasserted implementation choices.
5. **SSR/hydration shape.** `inputValue` lazy-initializes to the server-formatted value and both
   inputs carry `suppressHydrationWarning` (`NumberFieldRoot.tsx:139-143`, `:533`;
   `NumberFieldInput.tsx:125-127`). No SSR/hydration test exists in the mined corpus.
6. **Paste clipboard-read failure.** If reading `clipboardData` throws, the paste is dropped and a
   dev-only `warn` with `SafeReact.captureOwnerStack` fires (`NumberFieldInput.tsx:413-428`). No
   test forces the throw.
7. **Pointer-lock denial UX.** `requestPointerLock` rejection sets `isPointerLockDenied`
   (`NumberFieldScrubArea.tsx:328-329`) and suppresses the cursor render
   (`NumberFieldScrubAreaCursor.tsx:41-42`); behavior.md only records the jsdom/WebKit skips and
   Chromium stubs, not the denial path itself.
8. **Synthetic no-move click.** After a scrub pointerup with zero movement, the code dispatches a
   manual bubbling `MouseEvent('click')` on the original `pointerdown` target
   (`NumberFieldScrubArea.tsx:171-182`) because pointerdown preventDefault suppressed the native
   click. The observable click-bubbling is pinned (behavior.md, *Events*), but the no-movement
   condition and dispatch target are implementation details no assertion pins directly.
9. **Hold scroll-cancel.** `usePressAndHold` aborts an active hold when the pointer moves more
   than 8 px (`usePressAndHold.ts:231-245`), and prevents `contextmenu` during holds
   (`:122-132`). behavior.md's stepper pointer model covers mouseleave/mouseenter and taps, not
   these.
10. **Zero-anchored snap base.** With `min` undefined, the snap base is `min ?? 0`
    (`NumberFieldRoot.tsx:102`; `utils/validate.ts:111`). behavior.md says the grid is
    "min-anchored"; the unbounded-min zero anchor is untested.
11. **`name` vs `nameProp` asymmetry.** The hidden input uses Field-resolved `name`
    (`NumberFieldRoot.tsx:95`, `:520`) while `useRegisterFieldControl` receives the raw `nameProp`
    (`NumberFieldInput.tsx:88`) — no test distinguishes the two registration paths.
12. **`getViewportRect` third fallback.** The `documentElement.clientWidth/Height` branch
    (`utils/getViewportRect.ts:28-33`) beyond the `teleportDistance` and `visualViewport` branches
    is not covered by the mined utils pins.
13. **Autofill reason is `REASONS.none`.** The hidden-input autofill path labels its details
    `REASONS.none` (`NumberFieldRoot.tsx:508`); behavior.md's asserted-reason list never includes
    `'none'`, and the type keeps it only "for consistency" (`NumberFieldRoot.tsx:712-713`).
14. **The `.spec.tsx` files are type-level only.** `NumberFieldRoot.spec.tsx` asserts
    compile-time reason→native-event narrowing (`:14-36`) and `NumberFieldInput.spec.tsx` asserts
    render-callback prop types (`:5-10`); nothing executes them at runtime. The fixture stage
    should not look for runtime behavior there, and the audit should know these type contracts
    (e.g. wheel details expose `WheelEvent`, not `PointerEvent`) have no runtime enforcement.
15. **`details.allowPropagation` is dead surface here.** The veto model only consumes
    `cancel`/`isCanceled` (`createBaseUIEventDetails.ts:133-141`); no number-field call site
    checks `isPropagationAllowed`.
16. **Hidden-input style split.** `style={name ? visuallyHiddenInput : visuallyHidden}`
    (`NumberFieldRoot.tsx:532`) changes how the hidden input is hidden depending on whether it is
    form-submittable — visible to no test in the mined corpus (behavior.md, *DOM structure &
    portal behavior*, notes position is unasserted; the style split is likewise unasserted).
