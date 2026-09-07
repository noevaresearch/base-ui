# Combobox — store & internal utilities (batch: store-utils)

Behavior mined from characterization/unit tests only. Sources:
`packages/react/src/combobox/store.test.tsx`, `packages/react/src/combobox/utils/handleInputPress.test.ts`, `packages/react/src/combobox/utils/parts.test.ts`, `packages/react/src/combobox/utils/useInitialLiveRegionTextMutation.test.tsx`.

Note: `packages/react/src/combobox/store.test.tsx` exercises the store through the `Autocomplete` component family (`Autocomplete.Root/Trigger/Portal/Positioner/Input`), which is the combobox machinery under another public name; claims below are about the combobox store and parts.

## Public API surface (props, parts, subcomponents)

- These tests are infrastructure-level; no combobox part-name registry contents are asserted anywhere in them. UNVERIFIED — inferred from `packages/react/src/combobox/utils/parts.test.ts:1-3`, no test asserts a part-name registry (the `./parts` module tested there exports `clickHighlightedItem` and `getIndexAfterChipRemoval` utilities, not part names).
- Public-facing props observed driving the store: `Autocomplete.Root` accepts `items`, `inline`, `name`, `onOpenChange`; `Autocomplete.Portal` accepts `keepMounted`; parts `Autocomplete.Trigger` (accepts a `ref` callback), `Autocomplete.Positioner`, `Autocomplete.Input` are used. `packages/react/src/combobox/store.test.tsx:56-69`, `packages/react/src/combobox/store.test.tsx:156-181`
- Internal util exports proven: `handleInputPress(event, store, boolean)` from `combobox/utils/handleInputPress` (called as `handleInputPress(event, store, false)`) `packages/react/src/combobox/utils/handleInputPress.test.ts:3-4,27`; `getIndexAfterChipRemoval(index, count)` and `clickHighlightedItem(store, index, nativeEvent)` from `combobox/utils/parts` `packages/react/src/combobox/utils/parts.test.ts:2,7,19,38`.
- `useInitialLiveRegionTextMutation` is a generic hook (`useInitialLiveRegionTextMutation<HTMLDivElement>()`) returning a ref, and its module also exports the constant `INITIAL_LIVE_REGION_TEXT_MUTATION_RESET_DELAY`. `packages/react/src/combobox/utils/useInitialLiveRegionTextMutation.test.tsx:5-8,29,53`
- The combobox store is reachable by any descendant via `useComboboxRootContext()`, which returns the `ComboboxStore` instance. `packages/react/src/combobox/store.test.tsx:34-37,120-123`

## State model (controlled/uncontrolled, defaults, transitions)

Store state fields proven to exist and be readable:
- `state.inline` and `state.inputOwnsFormValue` are delivered to `store.subscribe` listeners as a single state snapshot object. `packages/react/src/combobox/store.test.tsx:83-86`
- `state.inputInsidePopup` exists and is `true` when the `Input` is rendered inside the `Portal`/`Popup`. `packages/react/src/combobox/store.test.tsx:61-67,199`
- State is also readable synchronously as `store.state.<key>` (e.g. `store.state.inputInsidePopup`, `store.state.inputOwnsFormValue`). `packages/react/src/combobox/store.test.tsx:199-200,214`

Store APIs:
- `store.subscribe(listener)` pushes a state snapshot to the listener on each change and returns an `unsubscribe` function. `packages/react/src/combobox/store.test.tsx:84-91`
- `store.useState(key)` provides per-key selector subscriptions; a `React.memo` component using `store.useState('inline')` and `store.useState('inputOwnsFormValue')` re-renders on changes and observes the settled values. `packages/react/src/combobox/store.test.tsx:120-126,142`

Transition semantics (the core characterization):
- Flipping `inline` from false to true (input inside popup) is published atomically: no subscriber snapshot ever observes `inline === true && inputOwnsFormValue === false` (guards a forward split), and — guarding a reverse-order split — after deduplication there is exactly one distinct changed snapshot. Both assertions are load-bearing and neither subsumes the other. `packages/react/src/combobox/store.test.tsx:94-112`
- The inline transition legitimately emits three raw notifications (one from the synchronization effect, two from prop-bag identity churn on the same re-render), so raw notification count is not a stable contract. `packages/react/src/combobox/store.test.tsx:102-104`
- The `useState` subscription path converges on the value the root computes (`inline:true, inputOwnsFormValue:true` → observed as `'true:true'`), not the value `ComboboxInput` writes from its ref callback. This does not by itself detect split transactions (React batches the listener sweeps); transaction shape is guarded by the direct-subscribe test. `packages/react/src/combobox/store.test.tsx:137-142`
- Ownership values: with the input inside the popup and `inline=false`, `inputOwnsFormValue` is `false` (a hidden input owns the form value instead). `packages/react/src/combobox/store.test.tsx:199-200`; when the input is not inside the popup, the input itself owns the form value (it is the sole named control, proven at DOM level). `packages/react/src/combobox/store.test.tsx:224-235,239-241`
- Unmounting and remounting the popup input replays its ref callback (the other writer of `inputOwnsFormValue`); after remount the ownership invariants still hold (`inputOwnsFormValue === false`, popup input nameless, exactly one `aria-hidden` named control). `packages/react/src/combobox/store.test.tsx:208-217`
- Controlled/uncontrolled `value`/`inputValue` syncing: UNVERIFIED — inferred from `packages/react/src/combobox/store.test.tsx:9-16`, no test in this batch asserts value/inputValue state or controlled syncing (these tests intentionally cover only the inline/ownership synchronization slice).
- `selectionMode` is always `'none'` for `Autocomplete.Root`. UNVERIFIED — inferred from `packages/react/src/combobox/store.test.tsx:40-41`, no test asserts this.

## Keyboard interactions

- ArrowDown on the Trigger opens the combobox: a `keydown` `KeyboardEvent` with `key: 'ArrowDown'` dispatched (bubbling) on the trigger element causes `onOpenChange` to be called with `true` as the first argument. `packages/react/src/combobox/store.test.tsx:156-165,183-185`
- Store commands are available at the earliest possible moment: the ArrowDown handler reads the `setOpen` command off the store at call time, and the open succeeds even when dispatched synchronously inside the Trigger's `ref` callback during the first commit (before ancestor layout effects run). `packages/react/src/combobox/store.test.tsx:145-152`
- No other keys are covered in this batch. UNVERIFIED — inferred from `packages/react/src/combobox/store.test.tsx:145-186`, no test asserts Enter/Escape/ArrowUp/etc. behavior.

## Focus management

- `handleInputPress`, when the press event's target is not an Element (e.g. a text node), focuses the input via `store.context.inputRef.current.focus()` — the mock's `focus` is called exactly once. `packages/react/src/combobox/utils/handleInputPress.test.ts:8-13,23-30`
- What press does when open/closed/disabled/readOnly: UNVERIFIED — inferred from `packages/react/src/combobox/utils/handleInputPress.test.ts:18-25`, no test asserts those branches; the mock only shows the store carries `state.openOnInputClick` (set to `false` in the mock) and `context.inputRef`, and the third boolean parameter is passed `false`.

## Accessibility (roles, aria-*, id linking)

- The visible input carries `role="combobox"` when it is not inside the popup (i.e. when it owns the form value), both in server-rendered output and after hydration. `packages/react/src/combobox/store.test.tsx:233-241`
- When the input is inside the popup, a separate hidden control carries the form `name` and is marked `aria-hidden="true"`; the popup input itself has no `name` attribute. `packages/react/src/combobox/store.test.tsx:204-206,215-217`
- Live-region announcement text: `useInitialLiveRegionTextMutation` appends a word joiner (`\u2060`) to non-empty text nodes of the referenced element (mutating `Status` → `Status\u2060`) to force an initial announcement, and leaves empty text nodes untouched. `packages/react/src/combobox/utils/useInitialLiveRegionTextMutation.test.tsx:24-42`
- ARIA attribute SSR coverage for the root: explicitly out of scope here (covered by `ComboboxRoot.test.tsx` per its comment). `packages/react/src/combobox/store.test.tsx:221-223`

## DOM structure & portal behavior

- Without `keepMounted`, a closed `Combobox.Portal` renders null, so children (popup input and its ref callback) never mount; with `keepMounted` the popup input is mounted while closed. The "renders null" statement is a motivating comment, but the test's dependence on `keepMounted` proves the mounted-while-closed side. `packages/react/src/combobox/store.test.tsx:61-76`
- Form-value ownership is a two-sided invariant: exactly one element in the document carries the given `name` in every configuration tested (popup input present, popup input unmounted, popup input remounted, SSR, hydrated). `packages/react/src/combobox/store.test.tsx:201-206,211,216,234,240`
- The named hidden control spans shapes: the comment notes controls carrying a form name can be the visible `combobox` input, a visually hidden `textbox`, or `type="hidden"` inputs rendered by `multiple` mode. UNVERIFIED — inferred from `packages/react/src/combobox/store.test.tsx:24-29`, no test asserts the `multiple` hidden-input shape.

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onOpenChange` payload: the first call's first argument is `true` (the requested open state). `packages/react/src/combobox/store.test.tsx:183-185`
- A bubbling `keydown` (`KeyboardEvent`, `bubbles: true`) dispatched on the trigger element reaches the trigger's handler even from within a ref callback. `packages/react/src/combobox/store.test.tsx:160-164`
- `handleInputPress` calls `event.preventDefault()` exactly once for a non-Element `nativeEvent.target` (the mock event is shaped `{ currentTarget, nativeEvent, preventDefault }`). `packages/react/src/combobox/utils/handleInputPress.test.ts:12-17,29`
- `clickHighlightedItem` invokes the DOM `click()` method of the highlighted item element taken from `store.context.listRef`, and while that click runs, `store.context.selectionEventRef.current` holds the originating event (a `KeyboardEvent('Enter')` in the test); the ref is cleared back to `null` after the call. `packages/react/src/combobox/utils/parts.test.ts:24-42`

## Edge cases (rapid interactions, unmount, nesting)

- Event target is a text node rather than an Element: `handleInputPress` still preventDefaults and focuses the input (no crash). `packages/react/src/combobox/utils/handleInputPress.test.ts:7-31`
- Chip removal leaving no chips: `getIndexAfterChipRemoval(0, 1)` returns `undefined` (no next index to highlight). `packages/react/src/combobox/utils/parts.test.ts:6-8`
- Highlighted item not rendered (empty `listRef`): `clickHighlightedItem` is a no-op — `selectionEventRef.current` stays `null` and no click occurs. `packages/react/src/combobox/utils/parts.test.ts:10-22`
- Popup input unmount → remount (via prop toggle) replays the input's ref callback and the ownership invariant is re-established identically. `packages/react/src/combobox/store.test.tsx:210-217`
- StrictMode: the ownership test runs under the StrictMode renderer, exercising callback-ref replay and the double-invoked render that re-runs the root's context assignment. `packages/react/src/combobox/store.test.tsx:190-195`
- SSR/hydration: form-value ownership and `role="combobox"` survive `renderToString` → `hydrate()`. `packages/react/src/combobox/store.test.tsx:220-242`
- Live-region hook with no attached ref: renders and does nothing (text content unaffected). `packages/react/src/combobox/utils/useInitialLiveRegionTextMutation.test.tsx:13-22`
- Live-region reset timing: after `INITIAL_LIVE_REGION_TEXT_MUTATION_RESET_DELAY` elapses (fake timers), the reset does not overwrite text the author changed before the reset (`'Updated'` survives the tick). `packages/react/src/combobox/utils/useInitialLiveRegionTextMutation.test.tsx:44-68`

## Shared harness dependencies

- `packages/react/test/index.ts` — barrel re-exporting `createRenderer` (used by `packages/react/src/combobox/store.test.tsx:4` and `packages/react/src/combobox/utils/useInitialLiveRegionTextMutation.test.tsx:3`).
- `packages/react/test/createRenderer.ts` — wraps `@mui/internal-test-utils`'s `createRenderer`; its `render` runs inside `act` and returns `setProps` (clones the element with new props and rerenders) plus `renderToString`/`hydrate`/`clock` from the shared renderer. (External package `@mui/internal-test-utils`, source of `screen`, was not read.)
