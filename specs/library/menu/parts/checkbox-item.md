## Public API surface (props, parts, subcomponents)

- `Menu.CheckboxItem` accepts a `checked` prop (boolean) that sets the initial/controlled checked ARIA/data state — tested with both `true` and `false` values (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:104-128`).
- `Menu.CheckboxItem` accepts an `onCheckedChange` callback invoked on toggle, receiving the new checked boolean as the first argument (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:263-291`).
- `onCheckedChange` receives event details as a second argument that exposes a `cancel()` method to prevent the toggle (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:293-321,300-303`).
- `Menu.CheckboxItem` accepts a `closeOnClick` boolean prop; when `true` the menu closes on item click, and it does not close by default (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:356-402`).
- `Menu.CheckboxItem` accepts a `disabled` prop; when set, the item is focusable but `onClick`, `onKeyDown`, `onKeyUp`, and `onCheckedChange` handlers are not invoked in response to Enter/Space/click (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:404-451`).
- `Menu.CheckboxItem` supports `render` prop for custom root element rendering (a `LoggingRoot` forwardRef component rendering `<li>`) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:33-59`).
- `Menu.CheckboxItem` accepts standard event handler props `onClick`, `onKeyDown`, `onKeyUp` in addition to `onCheckedChange` (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:406-421`).
- `Menu.CheckboxItemIndicator` is a subcomponent that must be rendered inside `Menu.CheckboxItem`; rendering it outside throws (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:31-41`).
- `Menu.CheckboxItemIndicator` accepts a `keepMounted` prop that keeps the indicator element in the DOM regardless of checked state / exit animation (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:14-29,128-134`).
- `Menu.CheckboxItemIndicator` accepts standard DOM passthrough props such as `data-testid`, `className`, and `onAnimationEnd` (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:128-133`).
- `Menu.CheckboxItemIndicator`'s ref instance is `window.HTMLSpanElement`, per conformance test configuration (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:14-16`).
- `Menu.CheckboxItem`'s ref instance is `window.HTMLDivElement`, per conformance test configuration (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:16-21`).

## State model (controlled/uncontrolled, defaults, transitions)

- With no `checked` prop supplied, clicking the item toggles state from unchecked to checked, then back to unchecked on a second click (uncontrolled behavior) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:131-158`).
- Supplying `checked={true}` or `checked={false}` sets the initial `aria-checked` and `data-checked`/`data-unchecked` attributes accordingly (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:104-128`).
- Clicking a checked item toggles it to `aria-checked="false"` / `data-unchecked` and vice versa (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:151-157`).
- Pressing Space toggles `data-checked` then `data-unchecked` on successive presses (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:185-189`).
- Pressing Enter toggles the item to `data-checked` (Chromium-only test) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:229-261`).
- When `onCheckedChange`'s event details call `cancel()`, the checked state does not change: `aria-checked` remains `"false"` and `data-checked` is not present after a click (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:293-321`).
- When the menu is closed and reopened (with `keepMounted` on `Menu.Portal` and `modal={false}`), the checkbox item's checked state persists: after checking the item, closing and reopening the menu, `aria-checked` remains `"true"` and `data-checked` is still present (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:323-353`).
- When `disabled`, interaction (click, Enter, Space) does not change checked state and does not invoke `onCheckedChange` (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:404-451`).
- `Menu.CheckboxItemIndicator`'s presence in the DOM is tied to the parent `Menu.CheckboxItem`'s checked state: unchecking it (via `checked={false}`) without `keepMounted` and without animation removes the indicator from the DOM once any pending animation frame/exit-animation logic completes (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:55-93,96-153,155-209`).
- With `keepMounted` on `Menu.CheckboxItemIndicator`, the indicator stays mounted (not `hidden`) even while unchecked, and an exit animation (`onAnimationEnd`) still fires when the parent item becomes unchecked (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:96-153`).
- Without `keepMounted`, when unchecked and an exit animation is defined via CSS (`data-ending-style` attribute present), the indicator remains mounted with `data-ending-style` until the animation finishes, after which it is removed from the DOM (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:155-209`).

## Keyboard interactions

- Space key toggles the checked state of the focused checkbox item (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:160-190`).
- Enter key toggles the checked state of the focused checkbox item (Chromium-only) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:229-261`).
- ArrowDown moves focus from the trigger/previous item to the next `menuitemcheckbox` (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:174-183,247-257`).
- During an active typeahead session (triggered by typing characters that match item text, e.g. "Item T"), pressing Space does not toggle the checked state of the currently-highlighted item and `onCheckedChange` is not called (Chromium-only) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:192-227`).
- When `disabled`, pressing Enter (keydown) or Space (keyup) does not invoke `onKeyDown`, `onKeyUp`, `onClick`, or `onCheckedChange` handlers (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:404-451`).
- ArrowDown/ArrowUp keyboard rerender behavior: pressing ArrowDown on the currently-focused/highlighted first item causes only the losing-highlight item and gaining-highlight item to rerender; items whose highlighted/selected state does not change do not rerender (Chromium-only perf test) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:23-99`).

## Focus management

- Clicking the trigger opens the menu and the item can then be interacted with via mouse (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:108-128,131-158`).
- Focusing the trigger and pressing ArrowDown moves DOM focus to the `menuitemcheckbox`, verified via `toHaveFocus()` (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:174-183,247-257`).
- A `disabled` checkbox item can still receive programmatic focus (`item.focus()`) — `focusableWhenDisabled` behavior — while remaining non-interactive for keyboard/click/checked-change handlers (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:404-451`).
- Typeahead session moves focus from one item to another matching item ("Item Two") when typing "Item T" (Chromium-only) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:209-219`).

## Accessibility (roles, aria-*, id linking)

- `Menu.CheckboxItem` renders with ARIA role `menuitemcheckbox`, queried via `screen.getByRole('menuitemcheckbox')` / `getAllByRole('menuitemcheckbox')` throughout the test suite (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:65,125,148,179,209,253,281,316,344`).
- `aria-checked` is set to `"true"` or `"false"` reflecting the checked state (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:104-128,151,156,225,319,351`).
- `data-checked` / `data-unchecked` attributes mirror the checked state on the DOM element (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:104-128,152,157,186,189,260,320,352`).

## DOM structure & portal behavior

- `Menu.CheckboxItem` must be rendered within `Menu.Popup`/`Menu.Positioner`/`Menu.Portal`/`Menu.Root` tree; conformance test renders it inside an open `Menu.Root` (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:16-21`).
- `Menu.CheckboxItemIndicator` requires a `MenuCheckboxItemContext`, supplied only when rendered inside `Menu.CheckboxItem`; rendering it standalone (even inside `render()`, without a `Menu.CheckboxItem` ancestor) throws the error "Base UI: MenuCheckboxItemContext is missing. MenuCheckboxItem parts must be placed within <Menu.CheckboxItem>." (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:31-41`).
- With `Menu.Portal keepMounted`, the popup subtree (including the checkbox item) stays mounted in the DOM across open/close/reopen cycles, preserving state (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:323-353`).
- When the menu closes with `closeOnClick`, the popup (role `menu`) is removed from the DOM entirely (`screen.queryByRole('menu')` returns `null`) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:357-378`).
- Without `closeOnClick`, the popup (role `menu`) remains present after clicking the item (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:380-401`).
- `Menu.CheckboxItemIndicator`, without `keepMounted` and with no exit animation defined, is removed from the DOM (queried by `data-testid`) once the item becomes unchecked, after pending animation frames are flushed (Chromium-only) (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:43-94`).
- `Menu.CheckboxItemIndicator` without `keepMounted`, when an exit animation is defined via CSS, remains in the DOM with `data-ending-style` attribute present until the animation completes, then is removed (Chromium-only) (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:155-209`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onCheckedChange(checked: boolean, eventDetails)` is called once per toggle interaction (click or keyboard), with `checked` reflecting the new state (`true` then `false` on subsequent clicks) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:263-291`).
- `eventDetails` (second argument to `onCheckedChange`) exposes a `cancel()` method; calling it prevents the checked state from changing (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:300-303,319-320`).
- When `disabled`, `onClick`, `onKeyDown`, `onKeyUp`, and `onCheckedChange` are never invoked in response to Enter keydown, Space keyup, or click events (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:435-449`).
- `Menu.CheckboxItemIndicator` supports an `onAnimationEnd` prop/handler that fires when the CSS exit animation completes (Chromium-only) (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:100-102,132,150-152`).
- During an active typeahead session, Space keyboard events do not trigger `onCheckedChange` (Chromium-only) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:221-224`).

## Edge cases (rapid interactions, unmount, nesting)

- Rapid double-toggle: clicking twice in succession correctly alternates `onCheckedChange` payloads `true` then `false`, with call count incrementing each time (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:282-291`).
- Rerender optimization: only items whose highlighted or selected/checked state actually changes rerender when navigating with ArrowDown among multiple `Menu.CheckboxItem`s; unrelated sibling items do not rerender (Chromium-only) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:23-99`).
- Closing the menu via a Space/typeahead-adjacent interaction does not cause a toggle if a typeahead session is active, even across two Space presses (Chromium-only) (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:220-226`).
- Closing the parent menu (clicking an external "Close" button that flips `checked` to `false` via React state, not a direct menu-close action) while an exit animation is in-flight keeps `Menu.CheckboxItemIndicator` mounted with `data-ending-style` until the animation ends, then removes it — demonstrating the indicator does not unmount immediately on unchecked state change if an exit animation is defined (Chromium-only) (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:155-209`).
- When no exit animation is defined and `requestAnimationFrame` is mocked/controlled, the indicator remains mounted until enqueued animation frame callbacks are flushed, after which a subsequent unchecked state change removes it from the DOM (Chromium-only) (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:43-94`).
- Rendering `Menu.CheckboxItemIndicator` outside of `Menu.CheckboxItem` (nesting violation) throws synchronously during render, verified via `await expect(render(...)).rejects.toThrow(...)` (`packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:31-41`).

## Shared harness dependencies

- `#test-utils` resolves to `packages/react/test/index.ts`, providing `createRenderer`, `describeConformance`, and `isJSDOM` used by both test files (`packages/react/src/menu/checkbox-item/MenuCheckboxItem.test.tsx:5`, `packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.test.tsx:4`).
