# otp-field — behavior spec (Stage 1: mined from tests)

Mined exclusively from this unit's test files:

- `packages/react/src/otp-field/input/OTPFieldInput.test.tsx`
- `packages/react/src/otp-field/root/OTPFieldRoot.react17.test.tsx`
- `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx`
- `packages/react/src/otp-field/utils/otp.test.ts`

The unit's TODO.md entry contains no `wraps-external:` field, so no third-party package delegation applies; all behavior below is proven by tests in this unit. Each claim ends with a citation to the test file and line(s) that prove it.

## Public API surface (props, parts, subcomponents)

- Namespace export `OTPField` is imported from `@base-ui/react/otp-field` and used as `OTPField.Root`, `OTPField.Input`, `OTPField.Separator`. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:5`
- `OTPField.Root` renders an `HTMLDivElement` (conformance `refInstanceof: window.HTMLDivElement`). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:15-18`
- `OTPField.Root` props exercised by tests: `length`, `children`, `defaultValue`, `value`, `name`, `required`, `validationType` (`'alpha' | 'alphanumeric' | 'numeric' | 'none'`), `normalizeValue`, `onValueChange`, `onValueInvalid`, `onValueComplete`, `disabled`, `readOnly`, `mask`, `autoComplete`, `autoSubmit`, `form`, `inputMode`, plus plain-div props (`id`, `data-testid`, `aria-describedby`, `aria-labelledby`). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:22-30`
- `length` takes the slot count; `OTPField.Root.Props` is the public props type name referenced by tests. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:20`
- `OTPField.Input` renders a native `HTMLInputElement` (conformance `refInstanceof: window.HTMLInputElement`) and must be rendered inside `OTPField.Root`. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:19-24`
- `OTPField.Input` accepts composed handlers `onMouseDown`, `onFocus`, `onBlur`, an `aria-label` prop (with first-slot-specific semantics, see Accessibility), and a `type` override (`type="tel"` wins over internal masking). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:819-829`
- `OTPField.Separator` renders its children (e.g. a `-` text node) between groups of slots; it does not affect slot counting. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:94-118`
- Pure utils in `packages/react/src/otp-field/utils/otp.ts` are exercised directly: `stripOTPWhitespace`, `normalizeOTPValue`, `replaceOTPValue`, `removeOTPCharacter`. `packages/react/src/otp-field/utils/otp.test.ts:2`

## State model (controlled/uncontrolled, defaults, transitions)

- Uncontrolled mode: `defaultValue` is split one character per slot after stripping whitespace, filtering to the allowed charset, and clamping to the slot count (`'12a34b56'` with `length=6` yields slots `1..6`). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:60-65`
- An overlong `defaultValue` is clamped to the rendered slot count, and the hidden validation input carries the clamped value (`'123456'`). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:67-79`
- Slots are assigned values by render order of the `OTPField.Input` children (slot indexes come from child position, not any prop). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:81-92`
- Controlled mode: rerendering with a new `value` prop updates all slot values (`'123456'` → `'654321'`). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:120-128`
- Value transitions: typing multiple characters into the first slot fills consecutive slots; typing into a later slot replaces that slot and the following ones. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:834-852`
- Typing/pasting is filtered per `validationType`: `alpha` keeps only letters (`'1a2b3Cd4'` → `'abCd'`), `alphanumeric` keeps only `[a-zA-Z0-9]` (`'A1-B2c3'` → `'A1B2c3'`). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:132-155`
- `validationType: 'none'` disables built-in filtering and uses `normalizeValue` (if provided) for custom normalization. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:222-234`
- With a non-`none` `validationType`, custom `normalizeValue` composes after built-in validation: `'a!'` → built-in strips `'!'` → `normalizeValue('a')` → `'A'`, then focus advances. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:236-256`
- `disabled` and `readOnly` both prevent all value changes and suppress `onValueChange`. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:762-794`
- Calling `eventDetails.cancel()` inside `onValueChange` rejects the transition: the value is unchanged and focus stays on the current slot (typing, backspace, and paste paths all honor cancel). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:156-175`
- Cancelled backspace: value and focus unchanged. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:639-659`
- Cancelled paste: value and focus unchanged. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:694-713`
- Root state attributes: `data-focused` set while any slot (including readonly slots) has focus and removed on blur; `data-filled` set once a value is entered; `data-complete` set when all slots are filled; `data-disabled` and `data-readonly` mirror the props. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1482-1519`
- Input-level state attributes: `data-complete` on every input when complete; `data-disabled` / `data-readonly` per slot. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:730-750`
- Controlled commit semantics: focus advance and completion are deferred until the controlled value actually changes. A stale controlled change (value applied long after the interaction) does not move focus. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:927-964`
- An asynchronously accepted controlled completion (value committed via `setTimeout`) does fire `onValueComplete` once the value lands. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:586-623`
- A stale controlled completion attempt (value applied later, unrelated to the interaction) does not fire `onValueComplete`. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:546-584`
- A controlled change from one complete value to another complete value does not fire `onValueComplete` again. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:625-646`
- Utils layer behavior (pure functions, same rules as above): `normalizeOTPValue` strips whitespace, filters per validation type, applies custom normalization after built-in filtering, then clamps to length; negative length yields `''`; `replaceOTPValue` splices new chars starting at a slot index (custom normalization composes; suffix chars preserved when a middle replacement shrinks); `removeOTPCharacter` deletes one char at an index and is a no-op for out-of-bounds indexes. `packages/react/src/otp-field/utils/otp.test.ts:14-93`

## Keyboard interactions

- `ArrowRight` moves focus to the next slot; `ArrowLeft` moves to the previous slot (LTR). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:77-91`
- Direction is flipped in RTL: `ArrowLeft` moves forward, `ArrowRight` moves backward. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:93-111`
- `ArrowUp` moves focus to the first slot; `ArrowDown` moves focus to the first empty slot (the "empty end" slot). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:292-306`
- `ArrowDown` that moves focus to the empty end slot also stops propagation (parent `onKeyDown` is not called). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:308-326`
- `ArrowDown` on the empty end slot keeps focus there; `ArrowDown` on the final slot when the value is complete keeps focus there. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:328-352`
- `Home` moves focus to the first slot; `End` moves focus to the empty end slot. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:376-402`
- `Ctrl/Cmd + ArrowLeft` jumps to the first slot and `Ctrl/Cmd + ArrowRight` jumps to the empty end slot (LTR). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:404-421`
- Same modifier behavior in RTL with directions swapped. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:423-444`
- Typing a character into a slot fills it and moves focus to the next slot; typing into the last slot keeps focus there and selects the just-typed character (selection 0–1). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:125-154`
- Typing an invalid character keeps the filled slot selected (value unchanged, selection 0–1, focus stays). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:177-192`
- Typing the same character already in a filled slot still advances focus to the next slot. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:276-290`
- Typing the same character into the final slot of a complete value does not reselect (no `select()` call, focus stays). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:354-374`
- `Backspace` on a filled slot deletes its character and moves focus to the previous slot (value `'1234'`, backspace on slot 1 → `'134'`, focus slot 0). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:594-607`
- `Backspace` on an empty non-first slot deletes the previous slot's character and moves focus there. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:609-622`
- After backspacing into the previous slot, that slot's value is selected (selection 0–1). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:678-692`
- `Ctrl/Cmd + Backspace` clears all slots and moves focus to the first slot. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:624-637`
- `Delete` removes the current character, keeps focus on the slot, and selects the emptied slot (selection 0–1). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:661-676`
- `Delete` on an empty slot and `Backspace` on an already-empty first slot change nothing and do not fire `onValueChange`. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:892-925`
- Readonly mode: all navigation keys (arrows, Home/End) keep working; `Delete`/`Backspace` are blocked from changing the value and keep focus. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:446-514`
- Disabled mode: vertical arrow keydowns are left unhandled (not canceled, propagate to ancestor handlers). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:471-496`

## Focus management

- Focusing a later empty slot redirects focus to the first empty slot (focus slot 4 of a field with value `'12'` → focus lands on slot 2). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:113-123`
- Mouse down on a slot selects its value (selection 0–1). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:194-203`
- A composed `onMouseDown` handler that calls `event.preventDefault()` prevents the internal select/focus handling (input does not receive focus; the event is canceled). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:205-221`
- A composed `onFocus` handler that calls `event.preventDefault()` keeps DOM focus on the input but prevents the internal focus state (root does not get `data-focused`). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:223-243`
- A composed `onBlur` handler that calls `event.preventDefault()` lets DOM focus move away (an outside button receives focus) but preserves the internal focus state (root keeps `data-focused`). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:245-274`
- Tab moves focus out of the field to the next tabbable element after the active slot (no tab trapping). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:570-592`
- The hidden validation input redirects focus to the first visible slot when focused. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1036-1049`
- Hidden-input autofill moves focus to the last slot and focus is preserved there when the autofill is later cleared. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1051-1076`
- Paste into a middle slot moves focus to the slot after the last pasted character. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:715-728`

## Accessibility (roles, aria-*, id linking)

- The root is exposed with `role="group"` (tests query it via `getByRole('group')`, including an accessible name from the label). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:111-117`
- `Field.Label` associates with the first slot: the label's `for` attribute equals the first input's id. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:651-663`
- `Field.Description` is applied to the group, not to individual slots: the group gets `aria-labelledby` (label id) and `aria-describedby` (`external-description` id followed by the Field description id). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:665-680`
- When `Field.Label`/`Field.Description` wrap the whole field, every slot gets `aria-labelledby` pointing at the label id, and slots do not get `aria-describedby` for the description. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:752-769`
- A native `<label htmlFor={rootId}>` gives every slot the same accessible name. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:771-791`
- Root `aria-describedby` is forwarded to the group. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:716-720`
- Root `aria-labelledby` is forwarded to the group only; slots never receive it. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:722-737`
- Default autocomplete: first slot `autocomplete="one-time-code"`, other slots `autocomplete="off"`. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:740-747`
- `autoComplete` prop overrides the default on both the first slot and the hidden input. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:749-757`
- Built-in `validationType` sets `inputMode` and `pattern` attributes: visible slots get a single-character `pattern` (e.g. `[a-zA-Z0-9]{1}`); the hidden input gets a length-sized `pattern` (e.g. `[a-zA-Z0-9]{6}` during SSR, `\d{6}` for numeric) plus `minlength`/`maxlength` equal to `length`. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1460-1478`
- `validationType="none"` omits the hidden input's `pattern`. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:190-197`
- A custom `inputMode` is honored when `validationType="none"` and can override the built-in one otherwise (applied to visible slots and the hidden input). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:199-217`
- An `aria-label` on the first slot is ignored when a shared label (native label or `Field.Label`) is associated — the shared label wins; `aria-label` on later slots is kept. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:793-813`
- A development warning fires when the first slot has an `aria-label` but no associated label exists: "Base UI: <OTPField.Input> ignores `aria-label` on the first input."; no warning when a native label or `Field.Label` is present. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:815-890`
- SSR renders unique slot ids derived from the root id: root id `verification-code` produces input ids `verification-code` through `verification-code-6`. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1438-1458`
- With the React 17 id fallback (no `SafeReact.useId`), generated slot ids are omitted during SSR until the client fallback assigns them. `packages/react/src/otp-field/root/OTPFieldRoot.react17.test.tsx:21-33`
- Invalid state comes only from the Field wrapper: `aria-invalid="true"` is applied to all slots when a Base UI `Form`/`Field` error exists (and not when disabled); no `aria-invalid` is set outside a Field. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1150-1208`

## DOM structure & portal behavior

- Structure: root `div[role="group"]` containing N native `input[type=textbox-role]` slots (one per child) in render order; arbitrary wrapper divs between slots are allowed and do not affect slot counting; `OTPField.Separator` renders its children inline between groups. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:94-118`
- Only the first slot gets `maxlength={length}`; other slots have no `maxlength`. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:74-77`
- A hidden validation input is rendered when `name` is provided (`input[name="otp"]`), carrying the joined value for forms. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1030-1034`
- Without a `name`, the hidden input falls back to id `<root-id>-hidden-input`. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1521-1525`
- `mask` renders all slots as `input[type="password"]`; a per-slot `type` prop overrides it. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:811-829`
- Portal behavior: N/A — no test in this suite asserts portal usage or mount points outside the normal tree (elements are queried via `screen`/`document.querySelector` without asserting a portal container).

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onValueChange(value: string, eventDetails)` fires once per accepted value change; `eventDetails.reason` is one of the shared reason constants imported from `packages/react/src/internals/reasons` (`REASONS.inputChange`, `REASONS.inputClear`, `REASONS.inputPaste`, `REASONS.keyboard`) — the literal string values of these constants are not asserted by tests. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:9`
- Reason mapping: typing fires `input-change` with the new value; clearing a slot by input fires `input-clear` with `''`; pasting fires `input-paste` with the pasted-and-normalized value; backspace deletion fires `keyboard` with the resulting value. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:295-319`
- `onValueInvalid(rawRejectedValue: string, eventDetails)` fires whenever characters were rejected/removed on the way to the committed value: built-in filtering before the OTP updates (`reason: inputChange`), custom normalization removing or expanding characters, and pastes (`reason: inputPaste`). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:323-450`
- Hidden-input autofill behaves like a typed change: `onValueChange`, `onValueInvalid` (raw autofilled string, `inputChange`), and `onValueComplete` all fire with the composed normalization result. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1078-1148`
- `onValueComplete(value: string, eventDetails)` fires exactly when the OTP becomes complete: by typing (`inputChange`) or by pasting (`inputPaste`), including pastes into a middle slot that complete the value; it does not fire for incomplete values. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:454-544`
- `eventDetails.cancel()` on `onValueChange` suppresses the transition and consequently prevents `onValueComplete` from firing for a completion-making paste. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:517-533`
- Paste events are consumed by the internal handler: a `paste` event dispatched without clipboard data is still default-prevented (`dispatchEvent` returns `false`) and the value is unchanged. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:560-568`
- Keydown events for handled navigation keys are default-prevented (`fireEvent.keyDown` returns `false` for ArrowUp/ArrowDown boundary moves). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:292-306`
- ArrowDown-to-end-slot focus moves additionally stop propagation (ancestor `onKeyDown` not called). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:308-326`
- Keydowns in disabled mode are neither canceled nor stopped (dispatchEvent returns `true` and an ancestor `onKeyDown` receives them). `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:471-496`
- Development warning on clipboard read failure during paste: console warns "Base UI: <OTPField.Input> could not read clipboard text during paste handling." once, value is unchanged, no crash. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:531-558`

## Edge cases (rapid interactions, unmount, nesting)

- Repeated identical complete pastes each fire `onValueComplete` (3 pastes → 3 events, including an identical re-paste of an already complete value). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:480-497`
- Stale controlled updates (value committed long after the interaction, tested with fake timers) never retroactively move focus or fire completion. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:546-584`
- Async accepted controlled changes drive focus/completion once the value actually lands. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:966-1003`
- Clipboard failures degrade gracefully: read error → warn once and continue; missing `clipboardData` → ignore the paste. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:531-568`
- `length` validation in development: warns when the rendered input count mismatches `length` (with plural/singular wording, e.g. "Received `length={6}` but rendered 5 inputs." / "rendered 1 input.") and when `length` is not a positive integer (`0`, `-1`, `3.7`, `NaN`, `Infinity`). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1527-1595`
- Hidden-input autofill is entirely ignored when `readOnly` or `disabled` (with or without a Field wrapper): no value change, no `onValueChange`/`onValueInvalid`/`onValueComplete`. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1150-1208`
- Form integration: a `required` hidden input blocks `form.checkValidity()` while incomplete and passes when complete. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1008-1028`
- Field validation in `onBlur` mode validates only after focus leaves the whole OTP field — blurring one slot into another slot does not validate. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:682-712`
- `autoSubmit` (default off): without it, completion never submits the owning form. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1223-1241`
- `autoSubmit` submits the owning form with the named value on completion; falls back gracefully when `form.requestSubmit` is absent (no submit, completion still fires). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1243-1286`
- Auto-submitting a Base UI `Form` avoids `flushSync`-inside-lifecycle errors; invalid forms block auto-submit, render the `Field.Error`, and keep focus on the first invalid field instead. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1288-1367`
- `form` prop: auto-submit targets the externally associated form by id; a `form` id that resolves to a non-form element means the ancestor form is not submitted. `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1390-1436`
- Rendering `OTPField.Input` outside `OTPField.Root` throws: "Base UI: OTPFieldRootContext is missing. OTPField parts must be placed within <OTPField.Root>." `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:892-902`
- Unmount behavior: N/A — no test in this suite asserts unmount cleanup.

## Shared harness dependencies

- `#test-utils` maps to `packages/react/test/index.ts`, which re-exports `@base-ui/utils/testUtils` plus local helpers. All three component test files import `createRenderer`, `describeConformance`, and `isJSDOM` from it. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:9`
- The root tests additionally import the shared reason constants from `../../internals/reasons` (i.e. `packages/react/src/internals/reasons`) and only assert identity with those constants (`REASONS.inputChange`, `REASONS.inputClear`, `REASONS.inputPaste`, `REASONS.keyboard`). `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:9`
- `SafeReact` from `@base-ui/utils/safeReact` is spied (`captureOwnerStack`) in warning tests; the React 17 test file mocks the same module to strip `captureOwnerStack`/`useId`. `packages/react/src/otp-field/root/OTPFieldRoot.react17.test.tsx:6-16`
- External (non-repo) harness: `@mui/internal-test-utils` provides `act`, `fireEvent`, `screen`, and `render`/`renderToString`; `@testing-library/user-event` provides `user` in a few tests. `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:3-5`
