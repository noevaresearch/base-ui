# otp-field — implementation spec (Stage 2: mined from source)

Companion to behavior.md (ground truth for WHAT happens). This document explains the WHY/HOW of
the source: the state machine, hook composition, context boundaries, and DOM decisions that
produce the documented behavior. Source files mined:

- `packages/react/src/otp-field/root/OTPFieldRoot.tsx`
- `packages/react/src/otp-field/root/OTPFieldRootContext.ts`
- `packages/react/src/otp-field/input/OTPFieldInput.tsx`
- `packages/react/src/otp-field/utils/otp.ts`
- `packages/react/src/otp-field/utils/stateAttributesMapping.ts`
- `packages/react/src/otp-field/index.ts`, `packages/react/src/otp-field/index.parts.ts`
- `packages/react/src/otp-field/root/OTPFieldRootDataAttributes.ts`, `packages/react/src/otp-field/input/OTPFieldInputDataAttributes.ts`

The unit's TODO.md entry (`TODO.md:452-458`) has no `wraps-external:` field, so no external
package delegation applies; everything below is derived from this repo's source.

## State machine / hooks used

### The core design: raw stored value + normalize-on-read + commit queue

The root keeps an unusually clean split between *stored* and *derived* value:

- `useControlled` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:95-100`) stores the raw
  committed string for both controlled and uncontrolled mode. Normalization is never baked into
  state — the render-visible `value` is re-derived on every render via
  `normalizeOTPValue` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:136`), and every write
  path re-normalizes before committing (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:228`).
  This is why controlled and uncontrolled mode behave identically for filtering/clamping, and why
  a late-arriving controlled value is normalized the same way as a typed one.
- `useValueAsRef` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:137`) mirrors that
  normalized value into `valueRef` so event handlers always compare against the latest committed
  value instead of a stale closure.

All interactions funnel through one write gate, `setValue`
(`packages/react/src/otp-field/root/OTPFieldRoot.tsx:226-264`):

1. Normalize the incoming value (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:228`).
2. Decide *completion eligibility* (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:229-236`):
   only `inputChange`/`inputPaste` reasons qualify, the normalized value must reach `length`, and
   the previous value must not already be complete — unless the reason is a paste. That final
   clause is what makes both of these documented behaviors true simultaneously: "Keyboard
   interactions" (typing the same character into the final slot of a complete value does not
   reselect/refire) and "Edge cases" (re-pasting an identical complete value re-fires
   `onValueComplete`).
3. Short-circuit when nothing changed — but still fire a qualifying completion from the
   short-circuit branch (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:238-244`). This is
   the re-paste path: no value change, no `onValueChange`, yet `onValueComplete` fires.
4. Fire `onValueChange` and honor `details.isCanceled` — returning `null` means "rejected", which
   callers treat as "don't queue focus" (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:246-250`).
   The cancellable details object is produced by `createChangeEventDetails`
   (`packages/react/src/internals/createBaseUIEventDetails.ts`), whereas
   `onValueInvalid`/`onValueComplete` receive non-cancellable `createGenericEventDetails` objects.
5. Commit via `setValueUnwrapped`, then enqueue a pending completion
   (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:252-260`). A commit that leaves the value
   incomplete explicitly discards any pending completion
   (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:258-260`).
6. Return the committed value (or `null`) so callers can decide whether to queue focus.

### The commit queue: why focus/completion are never applied synchronously

Interactions never focus or fire completion directly. They *enqueue against a value*:

- `queueFocusInput(index, value)` writes `pendingFocusRef`
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:170-172`).
- `useValueChanged(value, ...)` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:199-224`,
  implemented in `packages/react/src/internals/useValueChanged.ts:6-17` as a ref-diffing layout
  effect) drains both `pendingFocusRef` and `pendingCompleteValueRef` only after the value
  actually changes, guarded by exact equality with the queued value
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:210` and `:220`).

This queue-then-drain pattern is the entire mechanism behind the controlled-commit semantics in
behavior.md's "State model" section: focus advance and completion wait until the value lands, and
stale controlled updates (queued value ≠ landed value) are dropped without side effects. The same
drain also performs Field/Form integration side effects: `clearErrors(name)`, `setDirty`, and
`validation.change` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:200-203`).

### Focus state

Three atoms of React state in the root: `inputCount`
(`packages/react/src/otp-field/root/OTPFieldRoot.tsx:140`, fed by CompositeList's `onMapChange`),
`focusedIndex` (`:141`, initialized to the first empty slot), and `focused` (`:142`). The derived
`activeIndex` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:144-146`) switches meaning with
the `focused` flag — while focused it is the user's slot, while blurred it trails the value
length — and is the single driver of the roving `tabIndex` on slots
(`packages/react/src/otp-field/input/OTPFieldInput.tsx:102`).

Focus handlers live in the root so blur/focus decisions can span slots:

- `handleInputFocus` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:272-284`): targeting a
  slot past the value length redirects to the first empty slot (behavior.md "Focus management");
  otherwise the index is adopted, `focused`/`focusedIndex` are set, the Field's `setFocused` is
  updated, and the slot's text is selected.
- `handleInputBlur` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:286-298`): a
  `contains` check against the root means moving focus between slots never leaves the field, so
  touch/focus state and `onBlur` validation only trigger when focus exits the whole group
  (behavior.md "Edge cases": slot-to-slot blur does not validate).
- `focusInput` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:163-168`) clamps to the number
  of registered slots, then `focus()` + `select()` — the source of the ubiquitous 0–1 selection
  semantics documented throughout behavior.md.

### Hook inventory (call sites)

Root (`packages/react/src/otp-field/root/OTPFieldRoot.tsx`):

- `useControlled` — `:95-100`
- `React.useRef` ×4 — `rootRef` `:102`, `inputRefs` `:103`, `pendingFocusRef` `:104-107`,
  `pendingCompleteValueRef` `:108-111`
- `React.useMemo` — `firstInputRef` `:112-120` (notably *not* a real ref: a getter object whose
  `.current` reads `inputRefs.current[0]`, so Field registration and label detection can observe
  the first slot without knowing when it mounts), `state` `:311-324`, `contextValue` `:326-377`
- `useLabelableId` — `:122` (id generation + SSR-safe ids)
- `useAriaLabelledBy` — `:123` (explicit prop → Field label id → DOM label discovery fallback)
- `useIsoLayoutEffect` — `:148-150`, syncing `filled` into the Field context
- dev-only `useOTPFieldRootDevWarnings` — `:153-159`, defined `:647-674` (two `React.useEffect`s;
  called conditionally under a `NODE_ENV` guard with an eslint-disable, the codebase's standard
  dev-warning pattern)
- `useRegisterFieldControl` — `:161`, registering `firstInputRef` + `id` + `value` with the Field
- `useStableCallback` ×6 — `focusInput` `:163`, `queueFocusInput` `:170`, `setValue` `:226`,
  `reportValueInvalid` `:266`, `handleInputFocus` `:272`, `handleInputBlur` `:286`
- `useValueChanged` — `:199-224`
- `React.useCallback` — `getInputId` `:300-309` (first slot gets the root id; later slots get
  `{id}-{index+1}`)
- `useRenderElement` — `:379-391`

Input (`packages/react/src/otp-field/input/OTPFieldInput.tsx`):

- `useOTPFieldRootContext` — `:15`, called `:39-63` (throws outside a Root, per behavior.md
  "Edge cases")
- `useCompositeListItem({ guess: true })` — `:65`. `guess: true` seeds the index from render
  order to avoid a post-mount re-render for flat lists; the CompositeList commit flush corrects
  wrong guesses before paint
  (`packages/react/src/internals/composite/list/useCompositeListItem.ts:39-54`). Slot identity is
  therefore registration/DOM-order based — the mechanism behind behavior.md's "State model" claim
  that slot indexes come from child position.
- `React.useRef` — `inputRef` `:66` (used only by the dev aria-label warning)
- `useDirection` — `:67` (RTL arrow-key mapping in the keydown handler `:194-196`)
- dev-only `React.useEffect` — `:76-89` (first-slot `aria-label` warning)
- `useRenderElement` — `:323-328`

Utils (`packages/react/src/otp-field/utils/otp.ts:39-118`) are pure functions — no hooks. The
normalization pipeline is: strip whitespace → apply `validationType` regexp → apply user
`normalizeValue` → re-apply validation → clamp by Unicode code point
(`normalizeOTPValueWithDetails`, `packages/react/src/otp-field/utils/otp.ts:51-77`), threading a
`didRejectCharacters` flag through every shrinking step to drive `onValueInvalid`.
`replaceOTPValue` (`:92-110`) splices normalized chars at a slot index and re-normalizes the
concatenation, which is what preserves the suffix when a middle edit shrinks.

### Input event handlers are thin adapters

Every handler in `inputProps`
(`packages/react/src/otp-field/input/OTPFieldInput.tsx:91-321`) follows the same shape: normalize
via `normalizeOTPValueWithDetails` (rejections → `reportValueInvalid`), mutate via
`replaceOTPValue`/`removeOTPCharacter`, then delegate the commit to context `setValue` and, on
success, `queueFocusInput`. Notable internal decisions:

- `onMouseDown` `:111-118` and `onFocus` `:119-125` call `event.preventDefault()` *before* the
  internal handling checks, which is how composed-handler cancellation (behavior.md "Focus
  management") suppresses the internal select/focus path while the DOM event still ran.
- `onKeyDown` `:185-268`: handled navigation keys go through `stopEvent`
  (`packages/react/src/floating-ui-react/utils/event.ts:3-6`, preventDefault +
  stopPropagation). `readOnly` bails after the navigation block (`:222-224`), which is why
  navigation stays alive in readOnly but Delete/Backspace are blocked; `disabled` bails before
  everything (`:186-188`), which is why disabled keydowns are neither canceled nor stopped.
  The Ctrl/Cmd+Backspace clear is `:237-241`, Delete is `:243-247`, plain Backspace deletes the
  current or previous slot (`:262-267`), and a same-character retype over a full selection skips
  the state round-trip entirely (`:249-260`).
- `onChange` `:133-184` has a three-way branch when normalization empties the input: a true clear
  emits `removeOTPCharacter` with the `inputClear` reason (`:153-158`); an invalid edit into a
  filled slot restores the DOM value and reselects without touching state (`:159-162`); otherwise
  `replaceOTPValue` + focus queue (`:166-183`).
- `onPaste` `:269-320` reads the clipboard inside try/catch (dev warn on throw), then calls
  `event.preventDefault()` *after* a successful read (`:291`) — so a paste without clipboard data
  is still consumed (empty string) while a *throwing* read leaves the event untouched, matching
  the two distinct failure modes in behavior.md "Events" and "Edge cases".

## Context providers/consumers

- `OTPFieldRootContext` (`packages/react/src/otp-field/root/OTPFieldRootContext.ts:6-32`, created
  `:32`) is provided by the root (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:400`) and
  consumed by the input (`packages/react/src/otp-field/input/OTPFieldInput.tsx:63`). What crosses
  the boundary: the write API (`setValue`, `reportValueInvalid`), the focus API (`focusInput`,
  `queueFocusInput`, `getInputId`, `handleInputFocus`, `handleInputBlur`), per-slot config
  (`length`, `pattern`, `inputMode`, `autoComplete`, `mask`, `form`, `disabled`, `readOnly`,
  `required`, `validationType`, `normalizeValue`), shared state (`value`, `state`, `invalid`,
  `activeIndex`), and label wiring (`inputAriaLabelledBy`). The input holds zero state of its own
  beyond its dev-warning ref — all value/focus decisions are root-owned.
- `getOTPFieldInputState` (`packages/react/src/otp-field/root/OTPFieldRootContext.ts:46-57`) is
  exported beside the context and called by the input
  (`packages/react/src/otp-field/input/OTPFieldInput.tsx:70`) to derive per-slot state (root state
  spread + `value`/`index`/`filled` overrides). This shared helper is how input-level
  `data-filled`/`data-complete` (behavior.md "State model") fall out of the same state object that
  drives root attributes.
- Upstream contexts are consumed by the root only:
  `useFieldRootContext` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:76-88`) provides the
  Field plumbing (`setDirty`, `validityData`, `fieldDisabled`, `setFilled`, `invalid`, `fieldName`,
  `fieldState`, `validation`, `validationMode`, `setFocused`, `setTouched`);
  `useFormContext` (`:89`) provides `clearErrors` (invoked on every value change at `:200`);
  `useLabelableContext` (`:90`) provides `getDescriptionProps` and `labelId`. Field-derived state
  reaches the slots only indirectly, via `OTPFieldRootContext` (`invalid` and the spread
  `fieldState`) — the input never touches Field contexts directly, which is what makes
  behavior.md's "Accessibility" claim (aria-invalid applied to all slots from a Field error) a
  one-way flow.
- `CompositeListContext` is provided by the root's `CompositeList` wrapper
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:394-399`) and consumed by the input's
  `useCompositeListItem` (`packages/react/src/otp-field/input/OTPFieldInput.tsx:65`) for index
  assignment.
- `OTPField.Separator` is not an otp-field component at all:
  `packages/react/src/otp-field/index.parts.ts:3` re-exports the
  shared Separator (`packages/react/src/separator/Separator`). It registers with no list and reads
  no otp-field context, which is the structural reason it cannot affect slot counting (behavior.md
  "Public API surface").

## DOM/portal strategy and why

No portal exists anywhere in the unit, consistent with behavior.md's "DOM structure & portal
behavior" section (N/A). That is a viable strategy here because:

- An OTP field is an inline form widget; slots must flow in the user's layout, and nothing needs
  to escape a clipping or stacking context.
- Slot indexing tolerates arbitrary wrapper elements because CompositeList resolves order by
  document position among registered nodes
  (`packages/react/src/internals/composite/list/CompositeList.tsx:255` and `:298-302`) and
  re-checks ordering with a MutationObserver when nodes move
  (`packages/react/src/internals/composite/list/CompositeList.tsx:89-140`).

Key DOM decisions:

- The root is a `div[role="group"]` rendered through `useRenderElement`
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:379-391`) with merged
  `aria-describedby`/`aria-labelledby` and `rootStateAttributesMapping`.
- Slots are native single-character controlled inputs: `value[index] ?? ''`
  (`packages/react/src/otp-field/input/OTPFieldInput.tsx:69`). Only the first slot carries
  `maxLength` — the inline comment at
  `packages/react/src/otp-field/input/OTPFieldInput.tsx:100-101` explains it suppresses
  password-manager bubbles on later slots — and only the first slot carries the `autoComplete`
  hint (`:96`).
- A hidden validation input is rendered as a sibling *after* the group (inside the context
  provider), gated on a valid `length`
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:402-456`). It is `aria-hidden` with
  `tabIndex={-1}` (`:452-453`), and its style switches between `visuallyHiddenInput` (absolute
  positioning, used when `name` is present so it participates in form submission) and
  `visuallyHidden` (fixed positioning, otherwise) — `packages/react/src/otp-field/root/OTPFieldRoot.tsx:454`
  with the style definitions in `packages/utils/src/visuallyHidden.ts:14-24`. Focusing it
  redirects to slot 0 (`:405-407`), and its `onChange` treats autofill as an `inputChange` while
  queueing focus to the last filled slot (`:428-435`) — the mechanism behind the autofill
  behaviors in behavior.md's "Focus management" and "Events" sections.
- The `inputRefs` array (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:103`) is maintained
  by CompositeList's ref sync, giving `focusInput` an ordered target list, while
  `onMapChange → setInputCount` (`:396-398`) feeds the dev warnings about rendered-slot-count
  mismatches.

## Dependencies on other Base UI internals

This section enumerates exactly what a reimplementation must provide, replacing the coarse
`blocked-by: [Phase A complete]` default in TODO.md:451.

`@base-ui/utils` (packages/utils):

- `safeReact` — `SafeReact.captureOwnerStack` in dev warnings
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:3`, `packages/react/src/otp-field/input/OTPFieldInput.tsx:3`)
- `useControlled` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:4`)
- `useIsoLayoutEffect` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:5`)
- `useStableCallback` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:6`; six call sites)
- `useValueAsRef` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:7`)
- `visuallyHidden` / `visuallyHiddenInput` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:8`)
- `warn` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:9`,
  `packages/react/src/otp-field/input/OTPFieldInput.tsx:4`)
- `owner` — `ownerDocument` for the `form` id lookup
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:10`, used `:180`)

Cross-feature internals (packages/react/src):

- `floating-ui-react/utils` — `contains`
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:11`, used in blur handling `:287`) and
  `stopEvent` (`packages/react/src/otp-field/input/OTPFieldInput.tsx:5`, used for every handled
  navigation/deletion key)
- `internals/composite/list` — `CompositeList`
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:12`) and `useCompositeListItem`
  (`packages/react/src/otp-field/input/OTPFieldInput.tsx:6`); this is the slot-ordering backbone
- `internals/field-root-context` — `useFieldRootContext`
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:13`); the Field integration surface
- `internals/field-register-control` — `useRegisterFieldControl`
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:14`)
- `internals/form-context` — `useFormContext` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:16`)
- `internals/labelable-provider` — `useLabelableContext` (`:17`), `useAriaLabelledBy` (`:18`),
  `useLabelableId` (`:19`)
- `internals/useRenderElement` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:20`,
  `packages/react/src/otp-field/input/OTPFieldInput.tsx:9`) — element rendering, state→data-*
  attribute emission, ref/handler composition
- `internals/useValueChanged` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:21`) — the
  commit-queue drain primitive
- `internals/createBaseUIEventDetails` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:23-28`,
  `packages/react/src/otp-field/input/OTPFieldInput.tsx:10-13`) — cancellable vs generic event
  details
- `internals/reasons` (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:29`,
  `packages/react/src/otp-field/input/OTPFieldInput.tsx:14`)
- `internals/types` — `BaseUIComponentProps`
  (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:22`,
  `packages/react/src/otp-field/input/OTPFieldInput.tsx:7`)
- Via `packages/react/src/otp-field/utils/stateAttributesMapping.ts:1-2`:
  `internals/field-constants` (`fieldValidityMapping`) and the `StateAttributesMapping` type from
  `internals/getStateAttributesProps` — these decide which state keys become data-* attributes
  (notably `value`/`length`/`index` are mapped to null)

Other components:

- `separator` — a full component dependency: `OTPField.Separator` *is* the shared Separator
  (`packages/react/src/otp-field/index.parts.ts:3`), not a reimplementation
- `field/root/FieldRoot` — type-only (`FieldRootState` extends the root state,
  `packages/react/src/otp-field/root/OTPFieldRoot.tsx:15`)

Structural notes:

- The data-attribute constant files
  (`packages/react/src/otp-field/root/OTPFieldRootDataAttributes.ts`,
  `packages/react/src/otp-field/input/OTPFieldInputDataAttributes.ts`) are pure documented
  constants with no runtime consumer in this unit — actual attribute emission is generic via
  `useRenderElement` + `stateAttributesMapping`.
- Type surface: `packages/react/src/otp-field/index.ts` exports the namespace plus the root/input
  types; `packages/react/src/otp-field/index.parts.ts:1-3` wires the three parts.

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit loop. Each item was
verified absent from the unit's test files by search:

1. **`enterKeyHint`** on slots (`done` on the last, `next` otherwise,
   `packages/react/src/otp-field/input/OTPFieldInput.tsx:99`) — no test asserts it.
2. **`autoCorrect: 'off'` and `spellCheck: 'false'`**
   (`packages/react/src/otp-field/input/OTPFieldInput.tsx:97-98`) — no test asserts either.
3. **Hidden-input presentation attributes**: `type="text"`
   (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:439`), `aria-hidden`
   (`:452`), `tabIndex={-1}` (`:453`), and the `visuallyHiddenInput` vs `visuallyHidden` style
   switch (`:454`). The *focus redirect* off the hidden input is tested (behavior.md "Focus
   management"), but these attributes and the two-style strategy are not.
4. **`form` attribute forwarding to visible slots**
   (`packages/react/src/otp-field/input/OTPFieldInput.tsx:104`) — tests exercise the `form` prop
   only for auto-submit targeting (behavior.md "Edge cases"); the attribute on slot inputs is
   never asserted.
5. **`mergeAriaIds` deduplication** (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:637-640`)
   — whitespace-splitting and merging are exercised (behavior.md "Accessibility": external
   description id followed by the Field description id), but the dedupe/uniqueness guarantee is
   not.
6. **Pending-completion reset on an incomplete commit**
   (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:258-260`) — the stale-value guards are
   tested via unrelated-value mismatch (behavior.md "Edge cases"), but the specific "intervening
   incomplete commit cancels a previously queued completion" reset path is not directly asserted.
7. **Two utils lack direct unit tests**: `getOTPValidationConfig`
   (`packages/react/src/otp-field/utils/otp.ts:31-37`) and `normalizeOTPValueWithDetails`
   (`:51-77`) are not among the utils behavior.md's "Public API surface" section lists as directly
   exercised (only `stripOTPWhitespace`, `normalizeOTPValue`, `replaceOTPValue`,
   `removeOTPCharacter` are); they are covered only indirectly through component tests (pattern
   attributes, rejection events).
8. **Hidden-input `id` suppression when named**
   (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:440`) — the `<root-id>-hidden-input`
   fallback is tested (behavior.md "DOM structure & portal behavior"), but the inverse condition
   (no id when `name` is present) is not asserted.
9. **Unmount/cleanup paths** — behavior.md's "Edge cases" section marks unmount N/A, so none of
   the teardown logic is pinned by tests: CompositeList's ref-clearing and dirty-flag reset
   (`packages/react/src/internals/composite/list/CompositeList.tsx:172-200`), the composite item's
   unregister-on-ref-detach
   (`packages/react/src/internals/composite/list/useCompositeListItem.ts:62-82`), and Field
   deregistration (`packages/react/src/internals/field-register-control/useRegisterFieldControl.ts:40-45`).
10. **StopPropagation breadth asymmetry** — `stopEvent` uniformly prevents default *and* stops
    propagation for every handled navigation key
    (`packages/react/src/floating-ui-react/utils/event.ts:3-6`), but tests only assert
    propagation-stopping for `ArrowDown` (behavior.md "Keyboard interactions" / "Events"). The
    source behavior is broader than the tested claim — an implementation matching only the tests
    would under-propagate.
11. **Implicit label-id minting** — `useAriaLabelledBy` assigns a generated id to a native label
    that lacks one (`packages/react/src/internals/labelable-provider/useAriaLabelledBy.ts:42-44`);
    the native-label tests assert the accessible-name outcome (behavior.md "Accessibility"), not
    this DOM mutation.
