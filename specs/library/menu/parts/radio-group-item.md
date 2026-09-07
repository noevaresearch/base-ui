## Public API surface (props, parts, subcomponents)

- `Menu.RadioGroup` renders a `div` element. `packages/react/src/menu/radio-group/MenuRadioGroup.test.tsx:15-17`
- `Menu.RadioGroup` accepts a `defaultValue` prop for uncontrolled initial selection. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:158`
- `Menu.RadioGroup` accepts a controlled `value` prop. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:63`
- `Menu.RadioGroup` accepts an `onValueChange` callback prop, invoked with the new value and an `eventDetails` object as a second argument. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:247-271,281-286`
- `Menu.RadioGroup` accepts a `disabled` prop that disables all descendant `Menu.RadioItem`s. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:407-470`
- `Menu.RadioItem` accepts a required `value` prop identifying the option. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:23,159`
- `Menu.RadioItem` accepts a `closeOnClick` prop (boolean) controlling whether clicking the item closes the menu; default is not to close. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:341-393`
- `Menu.RadioItem` accepts a `disabled` prop to disable an individual item while leaving siblings interactive. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:473-545`
- `Menu.RadioItem` supports the `focusableWhenDisabled` behavior implicitly — disabled items (via group or item `disabled`) remain focusable but do not respond to interaction. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:395-470,473-545`
- `Menu.RadioItem` accepts standard event-handler props `onClick`, `onKeyDown`, `onKeyUp` that are forwarded/merged with internal handling. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:410-420,485-499`
- `Menu.RadioItem` accepts a `render` prop for custom root element rendering (e.g., rendering as `li`). `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:81-106`
- `Menu.RadioItem` accepts an `id` prop. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:82`
- `Menu.RadioItemIndicator` renders a `span` element by default. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:11`
- `Menu.RadioItemIndicator` accepts a `keepMounted` prop to keep it in the DOM regardless of checked state (used for exit-animation testing and conformance). `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:10,68`
- `Menu.RadioItemIndicator` accepts standard props such as `className`, `data-testid`, and `onAnimationEnd`. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:134-139`
- `Menu.RadioItem` must be rendered within `Menu.RadioGroup`; otherwise it throws an error with message "Base UI: MenuRadioGroupContext is missing. MenuRadioGroup parts must be placed within <Menu.RadioGroup>." `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:36-52`
- `Menu.RadioItemIndicator` must be rendered within `Menu.RadioItem`; otherwise it throws an error with message "Base UI: MenuRadioItemContext is missing. MenuRadioItem parts must be placed within <Menu.RadioItem>." `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:29-39`

## State model (controlled/uncontrolled, defaults, transitions)

- Uncontrolled mode: `Menu.RadioGroup` accepts `defaultValue` to set the initial selected value without an external `value`/`onValueChange` pairing needed for initial render. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:158,184,217,255,282,312,349,376`
- Controlled mode: `Menu.RadioGroup` accepts `value` (e.g., `value={value}` with external React state) to drive the checked item. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:63,132,182,193`
- Selecting an item (via click or Enter/Space key while focused) updates the checked item's `aria-checked` and `data-checked` attributes. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:170-175,203-205`
- `onValueChange` is called once per selecting interaction, with the new value as the first argument. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:247-272`
- Calling `eventDetails.cancel()` inside `onValueChange` prevents the value/selection state from updating: the item does not gain `aria-checked="true"` or `data-checked`. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:274-303`
- Selection state persists across the menu closing and reopening when the popup content is not unmounted (`Menu.Portal keepMounted`) and the radio group is uncontrolled with `defaultValue`. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:305-338`
- When the radio group (or an individual item) is `disabled`, clicking/pressing Enter/Space on the item does not trigger `onValueChange`, `onClick`, or the item's `onKeyDown` handler, and `aria-checked`/`data-checked` do not change. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:395-470,473-545`
- Pressing Space while an active typeahead session is in progress does not select the currently focused/highlighted item (Chromium-only test, skipped under JSDOM). `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:208-245`

## Keyboard interactions

- `ArrowDown` from the trigger moves focus into the menu onto the first `Menu.RadioItem`. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:192-201`
- Pressing `Space` or `Enter` while a `Menu.RadioItem` has focus selects it (sets `data-checked`). `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:177-206`
- Pressing `ArrowDown` while focus is on a `Menu.RadioItem` moves focus to the next non-disabled `Menu.RadioItem`, skipping disabled items in both directions (wrapping observed from item2 back to item1 when item2 is followed by ArrowDown and only two items exist). `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:453-455,531-544`
- Typing letters (typeahead) while focus is within radio items moves focus to the item whose text matches, without affecting selection immediately (Chromium-only). `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:227-245`
- On a disabled item, `ArrowDown` still moves focus to the next item even though the item itself doesn't respond to `Enter`/`Space`/click. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:453-455`
- `ArrowDown` on the highlighted item causes only the losing and gaining items to re-render (not unrelated items) — verified via render-count spies (Chromium-only, perf test). `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:126-147`

## Focus management

- Disabled `Menu.RadioItem`s (whether disabled via the group or individually) remain focusable — `item.focus()` succeeds and `toHaveFocus()` passes — matching a "focusable when disabled" pattern. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:436-437,514-515`
- `ArrowDown` moves focus from a disabled item to the next item in the group. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:453-455,531-533`
- Keyboard navigation (`ArrowDown` from trigger) results in focus landing on the (only) `menuitemradio`, confirmed via `waitFor(() => expect(item).toHaveFocus())`. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:196-201`

## Accessibility (roles, aria-*, id linking)

- `Menu.RadioGroup` exposes role `group`. `packages/react/src/menu/radio-group/MenuRadioGroup.test.tsx:14-17`
- `Menu.RadioItem` exposes role `menuitemradio`. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:114,170,197,227,267,298,324,335,363,388,431,509`
- Selected `Menu.RadioItem` has `aria-checked="true"`; unselected items have `aria-checked="false"`. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:173,243,301`
- Selected `Menu.RadioItem` has `data-checked` attribute (empty-string value); unselected items lack `data-checked`. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:174,204,302,337`
- Disabled `Menu.RadioItem`s carry a `data-disabled` attribute; non-disabled siblings do not. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:433-434,511-512`
- `Menu.RadioItemIndicator` exposes `data-ending-style` attribute while its exit animation is in progress. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:218`

## DOM structure & portal behavior

- `Menu.RadioGroup` renders as a `div` (confirmed by `refInstanceof: window.HTMLDivElement` in conformance tests and role `group` query). `packages/react/src/menu/radio-group/MenuRadioGroup.test.tsx:9-17`
- `Menu.RadioItem` conformance is tested with `refInstanceof: window.HTMLDivElement`, rendered inside `Menu.Root open` and a `MenuRadioGroupContext.Provider`. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:23-34`
- `Menu.RadioItem` supports custom root rendering via the `render` prop (e.g. as an `li`) while still being queried by role `menuitemradio`. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:64-114`
- `Menu.RadioItemIndicator` renders as a `span` (confirmed by `refInstanceof: window.HTMLSpanElement`), nested within `Menu.RadioItem`, `Menu.RadioGroup`, `Menu.Popup`, `Menu.Positioner`, `Menu.Portal`, `Menu.Root`. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:10-27`
- Without `keepMounted`, `Menu.RadioItemIndicator` is removed from the DOM once unchecked and (if defined) its exit animation finishes; it is present beforehand with `data-ending-style` during the exit transition. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:212-223`
- Without an exit animation defined, an unchecked, non-`keepMounted` `Menu.RadioItemIndicator` is removed from the DOM after the next animation frame(s) resolve. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:80-97`
- With `keepMounted` on `Menu.RadioItemIndicator`, it stays mounted regardless of checked state (used as the "always mounted" sibling indicator in animation tests). `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:68,142`
- The radio group's popup content can be kept mounted via `Menu.Portal keepMounted`, which preserves the underlying (unmounted-from-view) DOM/state across close/reopen so that selection state survives. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:309,329-338`

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onValueChange(value, eventDetails)`: called once when a `Menu.RadioItem` is selected (clicked, or via Enter/Space when focused); first argument is the new value. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:247-272`
- `eventDetails` passed to `onValueChange` exposes a `cancel()` method; calling it prevents the selection from being applied (no `aria-checked`/`data-checked` change). `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:281-303`
- `onClick`, `onKeyDown`, `onKeyUp` props passed to `Menu.RadioItem` are not invoked when the item (or its containing group) is `disabled`, for `Enter` keydown, `Space` keyup, or `click`. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:439-469,517-529`
- On a non-disabled item, `Enter` keydown triggers the item's `onKeyDown` handler, a `click`-equivalent (`onClick`), and `onValueChange`, each exactly once. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:535-539`
- `onAnimationEnd` on `Menu.RadioItemIndicator` fires when its CSS exit animation completes. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:138,160-162`

## Edge cases (rapid interactions, unmount, nesting)

- Rendering `Menu.RadioItem` outside a `Menu.RadioGroup` throws synchronously (rejects render) with a specific "Base UI:" prefixed error. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:36-52`
- Rendering `Menu.RadioItemIndicator` outside a `Menu.RadioItem` throws synchronously (rejects render) with a specific "Base UI:" prefixed error. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:29-39`
- Pressing Space twice in rapid succession during an active typeahead session does not trigger `onValueChange` and leaves the item unchecked (Chromium-only). `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:239-244`
- Repeated open/close/reopen of the menu (via clicking the trigger) with `keepMounted` preserves the previously selected `Menu.RadioItem`'s checked state. `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:321-338`
- When the controlling value changes externally (e.g., a "Close"/"Select b" button toggling React state from `a` to `b`), the previously-checked item's indicator either unmounts (default), plays an exit animation and then unmounts, or stays mounted with `data-ending-style` if `keepMounted`, depending on configuration and animation presence. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:41-99,101-163,165-224`
- Nesting: `Menu.RadioItemIndicator` is validated nested three levels deep inside `Menu.RadioItem` > `Menu.RadioGroup` > `Menu.Popup` (with an extra doubly-nested `Menu.Popup` in one animation test, seemingly incidental) > `Menu.Positioner` > `Menu.Portal` > `Menu.Root`. `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.test.tsx:58-76`
- A perf-oriented test confirms that unrelated sibling `Menu.RadioItem`s do not rerender when only highlight moves between two other items (Chromium-only, strict-mode doubled render counts). `packages/react/src/menu/radio-item/MenuRadioItem.test.tsx:54-148`

## Shared harness dependencies

- `#test-utils` resolves to `packages/react/test/index.ts` and provides `createRenderer`, `describeConformance`, and `isJSDOM`, used by all three test files. `packages/react/test/index.ts`
