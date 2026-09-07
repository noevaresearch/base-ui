## Public API surface (props, parts, subcomponents)

- `Menu.Group` — renders a `<div>` by default (`refInstanceof: window.HTMLDivElement`) (`packages/react/src/menu/group/MenuGroup.test.tsx:11`). It supports the standard `render` prop to render as another component, e.g. `render={<Menu.RadioGroup />}` (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:168`).
- `Menu.GroupLabel` — renders a `<div>` by default (`refInstanceof: window.HTMLDivElement`) (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:28`). Accepts a standard `id` prop (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:110`) and an `aria-hidden` prop that can be overridden by passing `aria-hidden={undefined}` (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:70,79`). Must be used within `<Menu.Group>` or `<Menu.RadioGroup>` (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:31-41`).
- `Menu.Item` — renders as a button-like element (`button: true` in conformance config) with an `HTMLDivElement` ref type by default (`packages/react/src/menu/item/MenuItem.test.tsx:16-22`). Must be rendered within `<Menu.Root>` (`packages/react/src/menu/item/MenuItem.test.tsx:24-34`). Props observed in tests: `onClick`, `id`, `closeOnClick` (boolean, default true) (`packages/react/src/menu/item/MenuItem.test.tsx:178-224`), `disabled` (`packages/react/src/menu/item/MenuItem.test.tsx:226-267`), `nativeButton` (used together with `render={<button type="button" disabled />}`) (`packages/react/src/menu/item/MenuItem.test.tsx:276-278`), `onMouseDown`, `onKeyDown`, `onKeyUp`, and the standard `render` prop.
- `Menu.LinkItem` — renders as an `HTMLAnchorElement` by default (`refInstanceof: window.HTMLAnchorElement`) (`packages/react/src/menu/link-item/MenuLinkItem.test.tsx:12`). Supports the `render` prop to compose with routing links, e.g. `render={<Link to="/two" />}` from `react-router` (`packages/react/src/menu/link-item/MenuLinkItem.test.tsx:44-45`).

## State model (controlled/uncontrolled, defaults, transitions)

- `Menu.Item`'s `closeOnClick` prop defaults to `true`: clicking an item closes the menu unless `closeOnClick={false}` is explicitly passed (`packages/react/src/menu/item/MenuItem.test.tsx:179-200,202-223`).
- `Menu.Item`'s `disabled` state (native/prop-driven) prevents the item's `onClick`, `onKeyDown`, and `onKeyUp` handlers from firing in response to Enter/Space keys or a click, while still allowing the item to receive focus (`packages/react/src/menu/item/MenuItem.test.tsx:227-267`).
- `Menu.GroupLabel`'s `aria-hidden` attribute defaults to `"true"` and can be removed by explicitly passing `aria-hidden={undefined}` (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:44-61,63-80`).
- `Menu.GroupLabel` registers/unregisters its `id` with the enclosing group's `aria-labelledby`; when multiple labels mount/unmount in sequence (old removed, new added), the group's `aria-labelledby` tracks the newest label and an older label's cleanup does not clobber a newer label's id (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:183-219`).

## Keyboard interactions

- Pressing `ArrowDown`/`ArrowUp` while a menu item has focus moves focus to the next/previous focusable, non-disabled item, skipping a natively-disabled item (`<button disabled>` via `render`) (`packages/react/src/menu/item/MenuItem.test.tsx:269-300`).
- Pressing `ArrowDown` while an item is highlighted moves the highlight to the next item and triggers rerenders only of the previously- and newly-highlighted items (`packages/react/src/menu/item/MenuItem.test.tsx:101-176`).
- When a `Menu.Item` is `disabled`, `Enter` (`keydown`) and `Space` (`keyup`) do not invoke `onKeyDown`/`onKeyUp`/`onClick` (`packages/react/src/menu/item/MenuItem.test.tsx:255-266`).
- `Menu.LinkItem` rendered with a router `<Link>` activates navigation on `Enter` and on `Space` when it has focus (`packages/react/src/menu/link-item/MenuLinkItem.test.tsx:69-71,97-99,83-84,111-113`).
- During an active typeahead session (triggered by typing printable characters, e.g. `"Item T"`, to move focus/highlight to a matching item), a subsequent `Space` keypress is consumed by typeahead and does not activate/navigate the focused `Menu.LinkItem` (`packages/react/src/menu/link-item/MenuLinkItem.test.tsx:118-165`).

## Focus management

- A `Menu.Item` can receive programmatic focus (`item.focus()`) even while `disabled`, but the disabled item does not respond to click/keyboard activation while focused (`packages/react/src/menu/item/MenuItem.test.tsx:251-267`).
- Keyboard arrow navigation moves focus between sibling `Menu.Item`s, automatically skipping a disabled one, and wraps focus from the last non-disabled item back appropriately (verified via `ArrowDown` then `ArrowUp` returning focus to the first item) (`packages/react/src/menu/item/MenuItem.test.tsx:286-300`).
- `Menu.LinkItem` elements can be given focus programmatically (`link.focus()`) and keyboard activation (`Enter`/`Space`) is scoped to whichever link currently has focus (`packages/react/src/menu/link-item/MenuLinkItem.test.tsx:61-115`).
- Typeahead-driven focus movement shifts focus from one `Menu.LinkItem` to another matching link based on typed text (`packages/react/src/menu/link-item/MenuLinkItem.test.tsx:146-158`).

## Accessibility (roles, aria-*, id linking)

- `Menu.Group` exposes the `group` ARIA role (`packages/react/src/menu/group/MenuGroup.test.tsx:14-17`).
- `Menu.GroupLabel` is hidden from the accessibility tree by default via `aria-hidden="true"` (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:44-61`), and this can be overridden by explicitly passing `aria-hidden={undefined}`, removing the attribute (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:63-80`).
- The enclosing `Menu.Group`'s `aria-labelledby` is set to the `Menu.GroupLabel`'s (auto-generated or user-provided) `id` (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:82-101,103-120`).
- The same `id` linking behavior for `aria-labelledby` applies when the group label is inside `Menu.RadioGroup` instead of `Menu.Group` (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:122-141,143-160`), and also when `Menu.RadioGroup` is rendered in place of `Menu.Group` via `render={<Menu.RadioGroup />}` (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:162-181`).
- `Menu.Item` exposes the `menuitem` ARIA role (used throughout via `screen.getByRole('menuitem')`) (`packages/react/src/menu/item/MenuItem.test.tsx:52,76,96`).
- `Menu.LinkItem` also exposes the `menuitem` role even when rendered as an anchor (`screen.getAllByRole('menuitem')`) (`packages/react/src/menu/link-item/MenuLinkItem.test.tsx:53`).
- `Menu.GroupLabel` throws a descriptive error if rendered outside `Menu.Group`/`Menu.RadioGroup`: `"Base UI: MenuGroupContext is missing. Menu group parts must be used within <Menu.Group> or <Menu.RadioGroup>."` (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:31-41`).
- `Menu.Item` throws a descriptive error if rendered outside `Menu.Root`: `"Base UI: MenuRootContext is missing. Menu parts must be placed within <Menu.Root>."` (`packages/react/src/menu/item/MenuItem.test.tsx:24-34`).

## DOM structure & portal behavior

- `Menu.Group`, `Menu.GroupLabel`, `Menu.Item`, and `Menu.LinkItem` are all exercised inside the standard `Menu.Root > Menu.Portal > Menu.Positioner > Menu.Popup` structure in the a11y and interaction tests, implying the popup content (including groups, group labels, and items) is rendered through `Menu.Portal` (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:45-57`, `packages/react/src/menu/item/MenuItem.test.tsx:39-50`, `packages/react/src/menu/link-item/MenuLinkItem.test.tsx:40-49`).
- UNVERIFIED — inferred from `packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:45-57`, no test asserts that the portal renders content to `document.body` or another specific portal target for these parts specifically.

## Events (names, payload shape, bubbling, preventDefault semantics)

- `Menu.Item`'s `onClick` handler fires once when the item is clicked via user interaction (`packages/react/src/menu/item/MenuItem.test.tsx:36-56`).
- Calling `event.preventBaseUIHandler()` inside `Menu.Item`'s `onClick` handler prevents Base UI's internal close-on-click behavior; the menu remains open (`screen.queryByRole('menu')` is not `null`) even though the user's `onClick` still fires (`packages/react/src/menu/item/MenuItem.test.tsx:58-81`).
- Calling `event.preventBaseUIHandler()` inside `Menu.Item`'s `onMouseDown` handler does not throw when the mouse-down event fires (`packages/react/src/menu/item/MenuItem.test.tsx:83-99`).
- When `Menu.Item` is `disabled`, `onClick`, `onKeyDown`, and `onKeyUp` handlers are not called in response to `keydown` (Enter), `keyup` (Space), or `click` events (`packages/react/src/menu/item/MenuItem.test.tsx:255-266`).
- Clicking a `Menu.Item` closes the menu by default (`closeOnClick` default `true`) and does not close it when `closeOnClick={false}` (`packages/react/src/menu/item/MenuItem.test.tsx:179-200,202-223`).

## Edge cases (rapid interactions, unmount, nesting)

- Rendering two `Menu.GroupLabel`s in sequence — first only "old", then both "old" and "new" simultaneously, then only "new" — verifies that when the "old" label unmounts after "new" has mounted, its cleanup effect does not incorrectly clear the group's `aria-labelledby`, which continues to correctly reference the newest ("new") label throughout (`packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:183-219`).
- Keyboard navigation (`ArrowDown`) correctly skips a natively-disabled `Menu.Item` (rendered via `render={<button type="button" disabled />}` with `nativeButton`) positioned between two enabled items, landing focus on the last item, and `ArrowUp` returns focus to the first item, confirming disabled items are excluded from the navigable set even amid mixed enabled/disabled siblings (`packages/react/src/menu/item/MenuItem.test.tsx:269-300`).
- A performance-oriented test confirms that with four `Menu.Item`s, only the previously-highlighted and newly-highlighted items rerender when highlight moves via `ArrowDown`; unrelated sibling items do not rerender (skipped in JSDOM environment) (`packages/react/src/menu/item/MenuItem.test.tsx:101-176`).
- For `Menu.LinkItem`, typing text that matches another item's label while one link is focused moves focus via typeahead to the matching link, and a `Space` keypress immediately after does not trigger navigation (the typeahead session intercepts it), demonstrating interaction between typeahead and link activation (`packages/react/src/menu/link-item/MenuLinkItem.test.tsx:118-165`).

## Shared harness dependencies

- `#test-utils` (resolves to `packages/react/test/index.ts`), providing `createRenderer`, `describeConformance`, and `isJSDOM`, imported by all four test files (`packages/react/src/menu/group/MenuGroup.test.tsx:4`, `packages/react/src/menu/group-label/MenuGroupLabel.test.tsx:5`, `packages/react/src/menu/item/MenuItem.test.tsx:5`, `packages/react/src/menu/link-item/MenuLinkItem.test.tsx:6`).
