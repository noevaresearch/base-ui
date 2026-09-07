# Combobox — Input / Trigger / InputGroup (behavior spec)

Behavior mined from test files only. Batch scope: `ComboboxInput.test.tsx`, `ComboboxInput.android.test.tsx`, `ComboboxInput.gecko.test.tsx`, `ComboboxTrigger.test.tsx`, `ComboboxInputGroup.test.tsx`. No external-package delegation exists for this unit's TODO entry.

## Public API surface (props, parts, subcomponents)

- `Combobox.Input` renders a native `<input>` (conformance asserts the ref is an `HTMLInputElement`) and must be rendered inside `Combobox.Root`. `packages/react/src/combobox/input/ComboboxInput.test.tsx:12-17`
- `Combobox.Input` supports the `render` prop to become a `<textarea>`; it stays editable when typed into and does not receive an invalid `type` attribute. `packages/react/src/combobox/input/ComboboxInput.test.tsx:93-119`
- `Combobox.Trigger` renders a native `<button>` (conformance: `refInstanceof: window.HTMLButtonElement`, `button: true`). `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:15-21`
- `Combobox.Trigger` also supports `render` (e.g. a `<div>` with `nativeButton={false}`); a disabled non-native trigger still blocks opening. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:106-122`
- `Combobox.InputGroup` renders a `<div>` (ref is `HTMLDivElement`) and accepts a `role` prop — by default it is exposed as `role="group"`. `packages/react/src/combobox/input-group/ComboboxInputGroup.test.tsx:11-16`, `packages/react/src/combobox/input-group/ComboboxInputGroup.test.tsx:197-205`
- All three parts pass the standard conformance suites: custom props (`lang`, `data-*`, `style`) are forwarded to the default element and to `render`-prop elements. `packages/react/src/combobox/input/ComboboxInput.test.tsx:12-17`, `packages/react/test/conformanceTests/propForwarding.tsx:23-95`
- Related parts exercised alongside the input: `Combobox.Chips`/`Combobox.Chip`/`Combobox.ChipRemove`, `Combobox.Value` (render-prop inside chips), `Combobox.Clear`, and chips-less trees where the input lives inside `Combobox.Chips` directly. `packages/react/src/combobox/input/ComboboxInput.test.tsx:382-448`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:902-962`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:450-501`
- Root-level props observed driving these parts: `disabled`, `readOnly`, `required`, `multiple`, `inline`, `items`, `defaultValue`, `defaultOpen`, `defaultInputValue`, `openOnInputClick`, `filter`, `onOpenChange`, `onValueChange`, `onInputValueChange`. `packages/react/src/combobox/input/ComboboxInput.test.tsx:835-843`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:229-241`
- `Combobox.Trigger` sets `tabindex="-1"` whenever it is not the main anchor (an `Combobox.Input` exists outside the popup), regardless of open state. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:23-41`

## State model (controlled/uncontrolled, defaults, transitions)

- Value: `defaultValue` seeds selection (single string or array for `multiple`); `onValueChange` reports mutations. `packages/react/src/combobox/input/ComboboxInput.test.tsx:322-357`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:902-932`
- Input value: `defaultInputValue` seeds the text; a fully controlled variant is driven by `inputValue` + `onInputValueChange` (test binds a text-input state to both). `packages/react/src/combobox/input/ComboboxInput.gecko.test.tsx:27-29`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:576-584`
- Open state: `defaultOpen` starts with the popup open; `onOpenChange(open, details)` observes transitions. `packages/react/src/combobox/input/ComboboxInput.test.tsx:705-726`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:377-402`
- Clicking the Input opens the popup with reason `input-press`; `openOnInputClick={false}` suppresses click-open (used by the composition tests and asserted directly in the InputGroup tests). `packages/react/src/combobox/input/ComboboxInput.test.tsx:289-318`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:835-843`, `packages/react/src/combobox/input-group/ComboboxInputGroup.test.tsx:108-135`
- Clicking the Trigger toggles the popup: open on first click, close on second; each transition calls `onOpenChange` once. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:348-375`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:377-402`
- Clearing the input text clears a single-selection value: after `user.clear`, no rendered option carries `aria-selected="true"` when the popup reopens. `packages/react/src/combobox/input/ComboboxInput.test.tsx:322-357`
- Escape lifecycle (single selection): Escape with the popup open restores the committed value into the input and closes the popup; a second Escape (popup closed, value selected) clears the input value. `packages/react/src/combobox/input/ComboboxInput.test.tsx:489-501`
- `readOnly` is reactive: toggling the prop on/off toggles the trigger's `data-readonly` style hook. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:229-241`
- `disabled` resolution order: local part prop wins over Root, Root wins/propagates to parts, and Field.Root `disabled` inherits into both Input and Trigger. `packages/react/src/combobox/input/ComboboxInput.test.tsx:31-53`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:57-79`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:78-87`
- Placeholder state is derived from selection: `data-placeholder` is present with no value (including `multiple` with `[]`) and absent once a value is selected. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:1217-1303`
- Clear-on-blur of the input: UNVERIFIED — inferred from `packages/react/src/combobox/input/ComboboxInput.test.tsx:964-1014` (Tab-out only closes the popup; no test in this batch asserts the input text is reset on blur).

## Keyboard interactions

On the Input:

- Typing a character opens the popup. `packages/react/src/combobox/input/ComboboxInput.test.tsx:359-380`
- Typing is blocked under `readOnly`: the input value stays `''` and `onInputValueChange` is never called. `packages/react/src/combobox/input/ComboboxInput.test.tsx:222-246`
- `Home` moves the caret to position 0 and `End` moves it to the value length; with overflowing content (Chromium only) they also scroll the input to the matching horizontal position. `packages/react/src/combobox/input/ComboboxInput.test.tsx:503-539`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:541-574`
- In Gecko RTL, Home/End caret semantics are inverted: `Home` sets `selectionStart` to the value length and `End` sets it to 0. `packages/react/src/combobox/input/ComboboxInput.gecko.test.tsx:24-41`
- `Escape` with the popup closed clears a selected single value (see also propagation semantics under Events). `packages/react/src/combobox/input/ComboboxInput.test.tsx:496-501`
- In `multiple` mode with the popup closed, Escape clears the whole value to `[]` and stops propagation; if the value is already empty, or the combobox is `inline`, Escape propagates to ancestor handlers. `packages/react/src/combobox/input/ComboboxInput.test.tsx:785-833`
- `Backspace` in an empty input deletes the last *rendered* chip (not the last selected value when fewer chips are rendered) and, when no chips are rendered at all, removes the last selected value. `packages/react/src/combobox/input/ComboboxInput.test.tsx:902-962`
- Backspace is inert while `disabled` or `readOnly`: the chip remains after `{backspace}`. `packages/react/src/combobox/input/ComboboxInput.test.tsx:413-414`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:446-447`
- `ArrowLeft` with the caret at the start of the input moves focus to the last rendered chip and begins chip-highlight navigation; ArrowRight past the last chip or ArrowLeft before the first chip returns focus to the input; `Delete`/`Backspace` while a chip is highlighted removes it and lands focus on the adjacent chip; typing a character returns focus to the input and inserts it. `packages/react/src/combobox/input/ComboboxInput.test.tsx:609-668`
- With no chips rendered, `ArrowLeft` keeps focus on the input. `packages/react/src/combobox/input/ComboboxInput.test.tsx:670-685`
- `Control+ArrowDown` (modified arrow navigation) does not close an open popup. `packages/react/src/combobox/input/ComboboxInput.test.tsx:705-726`
- An IME `Enter` keydown (`keyCode`/`which` 229) with no highlighted item does not select anything (`onValueChange` not called). `packages/react/src/combobox/input/ComboboxInput.test.tsx:728-749`
- Enter-to-select a highlighted item from the Input, and Enter form-submit behavior, are not covered by this batch: UNVERIFIED — inferred from `packages/react/src/combobox/input/ComboboxInput.test.tsx:728-749`, no test asserts this.

On the Trigger:

- `ArrowDown`/`ArrowUp` on a focused Trigger open the popup and move focus to the combobox (the input inside the popup); `Escape` then closes it. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:404-438`
- Under `readOnly`, the Trigger still opens on `ArrowDown`, `ArrowUp`, `Enter`, and `Space`, each calling `onOpenChange` exactly once. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:266-295`
- Closed-state typeahead: typing on a focused Trigger selects the first matching item while the popup stays closed; repeated typing cycles to the next match after the current selection (native-select-like), wrapping past the end. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:914-947`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:997-1037`
- Typeahead only matches mounted, indexed item labels (removing an item excludes it), and a label registered without a value never commits a selection. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:1099-1157`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:202-226`
- On a `readOnly` Trigger, typeahead never commits a value (`onValueChange` not called; `data-placeholder` remains). `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:297-323`
- Arrow keys do not open the popup when the anchored reference is a plain `<textarea>` (Input rendered outside the popup as textarea). `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:440-454`

## Focus management

- Opening via Trigger `ArrowDown`/`ArrowUp` moves focus from the Trigger to the combobox input inside the popup. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:404-438`
- After closing via Escape (typeahead tests), focus returns to the Trigger asynchronously. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:984-987`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:1027-1029`
- Tab out of the Input closes the popup and moves focus to the next tabbable element; the same holds when repeating the open→Tab cycle. `packages/react/src/combobox/input/ComboboxInput.test.tsx:964-1014`
- Chip highlight navigation is stateful across focus returns: the highlight index survives the input regaining focus, so subsequent Arrow keys continue from the highlighted chip; navigating past either end returns focus to the input. `packages/react/src/combobox/input/ComboboxInput.test.tsx:609-668`
- Removing/emptying the item list clears the inline highlight: after all items are removed and the input is clicked again, `aria-activedescendant` is not restored. `packages/react/src/combobox/input/ComboboxInput.test.tsx:751-783`
- Mousedown on `Combobox.InputGroup` padding (outside the input itself) focuses the input and opens the popup; with `openOnInputClick={false}` it focuses without opening; under Field `disabled` it neither focuses nor opens; under `readOnly` it focuses and opens. `packages/react/src/combobox/input-group/ComboboxInputGroup.test.tsx:47-77`, `packages/react/src/combobox/input-group/ComboboxInputGroup.test.tsx:108-135`, `packages/react/src/combobox/input-group/ComboboxInputGroup.test.tsx:137-166`, `packages/react/src/combobox/input-group/ComboboxInputGroup.test.tsx:168-195`
- The test titled "should move focus to clear button when pressing Escape and popup is closed" only asserts value mutations, not focus on the Clear button: UNVERIFIED — inferred from `packages/react/src/combobox/input/ComboboxInput.test.tsx:450-501`, no test asserts this.
- Typing on the Input with a controlled `inputValue` preserves caret position through React-driven value updates, including insertions mid-string. `packages/react/src/combobox/input/ComboboxInput.test.tsx:576-607`

## Accessibility (roles, aria-*, id linking)

- The native Input is exposed as `role="combobox"` even while the popup is closed (queried via `getByRole('combobox')` before any interaction). `packages/react/src/combobox/input/ComboboxInput.test.tsx:342-344`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:510`
- A non-input control (textarea `render`) carries the combobox ARIA contract (`role="combobox"`, `aria-expanded`, `aria-controls`) only while the popup is open; an `inline` combobox exposes them from the start, plus `aria-haspopup="listbox"` and `aria-autocomplete="list"`. `packages/react/src/combobox/input/ComboboxInput.test.tsx:123-157`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:161-183`
- `aria-controls` on the input (and trigger) is linked to the rendered listbox/dialog `id`. `packages/react/src/combobox/input/ComboboxInput.test.tsx:154-156`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:905-909`
- `disabled` renders the native `disabled` attribute on the Input and the Trigger; `readOnly` renders `aria-readonly="true"` plus the native `readonly` attribute on the Input and propagates to the listbox (`aria-readonly="true"` on the listbox) and to chips (`aria-readonly` on Chip). `packages/react/src/combobox/input/ComboboxInput.test.tsx:20-29`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:44-55`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:186-197`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:219`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:443-444`
- `required` sets `aria-required="true"` on the Input; on the Trigger it is set only when the input lives inside the popup. `packages/react/src/combobox/input/ComboboxInput.test.tsx:276-287`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:812-833`
- When the input is outside the popup, the Trigger is a plain button: no `role`, no `aria-readonly` (the input carries that state). When the input is inside the popup, the Trigger itself takes `role="combobox"` and `aria-readonly="true"` under `readOnly`. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:847-861`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:835-845`
- Trigger (main-anchor mode) ARIA state: closed → `tabindex="0"`, `aria-expanded="false"`, `aria-haspopup="dialog"`, no `aria-controls`; open → `aria-expanded="true"` and `aria-controls` pointing at the popup dialog id. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:863-910`
- Highlighted-option tracking via `aria-activedescendant` on the input: set after `ArrowDown`, removed when the highlighted slot's items are removed, and cleared by an empty composition update while the popup stays open. `packages/react/src/combobox/input/ComboboxInput.test.tsx:776-782`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:892-899`
- Style hooks surfaced for users: `data-popup-side` (input and trigger, Chromium only), `data-list-empty` (input and trigger), `data-readonly` (trigger), `data-placeholder` (trigger), plus `data-highlighted` on options during drag selection. `packages/react/src/combobox/input/ComboboxInput.test.tsx:1018-1050`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:1161-1193`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:1052-1072`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:1195-1215`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:229-241`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:1217-1303`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:514`

## DOM structure & portal behavior

- The popup is rendered through `Combobox.Portal` → `Combobox.Positioner` → `Combobox.Popup` (role `dialog`) → `Combobox.List` (role `listbox`); the input/trigger tests always mount list content inside this tree and query it via `getByRole('listbox')`. `packages/react/src/combobox/input/ComboboxInput.test.tsx:55-76`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:903`
- `Combobox.Portal` accepts `keepMounted`, which keeps list content mounted after close (tested for typeahead parity). `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:997-1037`
- `inline` mode renders the `Combobox.List` directly inside `Combobox.Root` (no portal) with the popup permanently "open" (`open` prop used in tests); the input's combobox ARIA attributes are applied immediately. `packages/react/src/combobox/input/ComboboxInput.test.tsx:161-183`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:751-765`
- `Combobox.InputGroup` wraps input + trigger (and chips) in a `role="group"` container whose padding is an interaction surface: clicking it neither dismisses an open popup nor counts as an outside click. `packages/react/src/combobox/input-group/ComboboxInputGroup.test.tsx:18-45`
- Chips can be rendered by `Combobox.Value`'s render prop inside `Combobox.Chips`, interleaved with the input; only rendered chips count for Backspace removal. `packages/react/src/combobox/input/ComboboxInput.test.tsx:902-932`
- The Input may also live inside the popup itself (`Combobox.Popup` containing `Combobox.Input`), in which case the Trigger becomes the closed-state combobox anchor. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:812-821`

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onOpenChange(open, details)` fires with `details.reason`: `input-press` when the Input is clicked, `trigger-press` when the Trigger is clicked, and `cancel-open` when a press that opened the popup is released away from the trigger. `packages/react/src/combobox/input/ComboboxInput.test.tsx:315-317`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:480-481`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:732-733`
- `onValueChange(value, details)`: receives the new selection — `[]` on Escape-clear in multiple mode, the committed string on drag-select commit, and the shortened array on Backspace chip/value removal. `packages/react/src/combobox/input/ComboboxInput.test.tsx:799`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:518-521`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:930-931`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:960-961`
- `onInputValueChange(value, event)`: fired with the composed text and the raw event during Android IME composition; never fired when typing is blocked by `readOnly`. `packages/react/src/combobox/input/ComboboxInput.android.test.tsx:31-35`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:245`
- Escape propagation rules: propagation is stopped only when Escape actually mutates state (clearing a non-empty multiple value); an already-empty multiple value or a closed inline combobox lets Escape bubble to ancestor `onKeyDown` handlers. `packages/react/src/combobox/input/ComboboxInput.test.tsx:785-833`
- A disabled non-native Trigger swallows native `keydown` (dispatched `ArrowDown` KeyboardEvent) — `onOpenChange` is not called. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:106-122`
- Field integration: blurring the Trigger runs Field validation with the current value as the first argument (single selection validates the selected value; Autocomplete.Trigger validates its query). `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:157-170`
- Pointer semantics on the Trigger (press-open flow): `pointerdown`+`mousedown` on the Trigger opens the popup immediately; on release, a `mouseup` more than 5px outside the trigger bounds closes it with reason `cancel-open`, while a release within (or near) the bounds keeps it open. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:703-808`
- Drag selection commits: with the popup opened by a trigger press, `mousemove` over an option highlights it (`data-highlighted`) and `mouseup` over the option commits it via `onValueChange` — both when the input is outside and inside the popup; releasing without ever hovering an item commits nothing. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:486-560`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:667-699`

## Edge cases (rapid interactions, unmount, nesting)

- Unmount during a press: a pending `mouseup` after the Trigger unmounts must not fire `onOpenChange` (rerender removing the trigger between `mousedown` and a body `mouseup`). `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:172-200`
- Stale pointer state across close/reopen: an item `pointerdown` that never releases, followed by Escape-close, must not poison the next drag-select; the shared pointerdown ref is cleared on close so the later gesture commits exactly once. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:612-665`
- Multi-touch: a non-primary touch landing on a different item must not cause a double commit (once on `mouseup`, once on `click`); only the primary pointer's gesture commits (Chromium only). `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:562-610`
- IME composition: Android composition `change` events propagate through `onInputValueChange` during composition; an empty composition update closes the popup when `openOnInputClick={false}` (input clicks don't open) and otherwise clears the highlight while staying open. `packages/react/src/combobox/input/ComboboxInput.android.test.tsx:23-36`, `packages/react/src/combobox/input/ComboboxInput.test.tsx:835-900`
- IME Enter guard: a 229 `keyCode` Enter with no highlight never selects. `packages/react/src/combobox/input/ComboboxInput.test.tsx:728-749`
- Custom input elements: an `<input type="number">` has `selectionStart === null`; ArrowLeft treats null as "beginning" and still navigates to chips. `packages/react/src/combobox/input/ComboboxInput.test.tsx:687-703`
- Nested part interactions: a Chip mousedown inside an `InputGroup` is handled exactly once (`onOpenChange` called once) — the group does not double-handle chip presses. `packages/react/src/combobox/input-group/ComboboxInputGroup.test.tsx:79-106`
- Clicking padding inside an open `InputGroup` does not dismiss the popup (it is treated as inside the combobox). `packages/react/src/combobox/input-group/ComboboxInputGroup.test.tsx:18-45`
- Typeahead registry resilience: after open→close cycles (which mount/unmount the list), typeahead still works under both StrictMode settings and with `keepMounted`, and stays correct after items are reordered while closed. `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:949-995`, `packages/react/src/combobox/trigger/ComboboxTrigger.test.tsx:1039-1097`
- Horizontal scroll sync on Home/End for overflowing inputs is verified only in Chromium (skipped in jsdom). `packages/react/src/combobox/input/ComboboxInput.test.tsx:541-574`

## Shared harness dependencies

Harness files under `packages/react/test/` read to interpret these tests:

- `packages/react/test/index.ts` — `#test-utils` entry; re-exports `createRenderer`, `describeConformance`, `isJSDOM` (from `@base-ui/utils/testUtils`), `firePointer`, `wait`, etc.
- `packages/react/test/createRenderer.ts` — act-wrapped `render`, plus `rerender`/`setProps`; exposes the `user` interaction helper from `@mui/internal-test-utils`.
- `packages/react/test/describeConformance.tsx` — runs the conformance suite (props spread, ref forwarding, render prop, className) used for Input, Trigger, and InputGroup.
- `packages/react/test/conformanceTests/propForwarding.tsx` — proves custom prop/`style` forwarding to default and `render` elements.
- `packages/react/test/conformanceTests/refForwarding.tsx` — proves the ref attaches and is an instance of the expected element type.
- `packages/react/test/conformanceTests/renderProp.tsx` — proves `render` as element/function, ref and className merging.
- `packages/react/test/conformanceTests/className.tsx` — proves string `className` application.

External helper package used by the tests but not part of `#test-utils`: `@mui/internal-test-utils` (`fireEvent`, `screen`, `waitFor`, `act`, `flushMicrotasks`); not read per batch rules.
