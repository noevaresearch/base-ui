## Public API surface (props, parts, subcomponents)

- `Menu.Trigger` passes `describeConformance` checks, rendering as an `HTMLButtonElement` when given `refInstanceof: window.HTMLButtonElement` and supporting a `button: true` conformance option `packages/react/src/menu/trigger/MenuTrigger.test.tsx:14-21`.
- `Menu.Trigger` supports a `disabled` prop that renders a disabled button `packages/react/src/menu/trigger/MenuTrigger.test.tsx:36-45`.
- `Menu.Trigger` supports a `render` prop to override the root element, e.g. `render={<span />}` with `nativeButton={false}` `packages/react/src/menu/trigger/MenuTrigger.test.tsx:89-91`.
- `Menu.Trigger` supports `openOnHover` to open the menu on hover `packages/react/src/menu/trigger/MenuTrigger.test.tsx:248`.
- `Menu.Trigger` supports a `delay` prop controlling the hover-open delay (tested with values `0`, `100`, `300`) `packages/react/src/menu/trigger/MenuTrigger.test.tsx:248,336,367`.
- `Menu.Trigger` supports `onMouseDown` and `onClick` event handler props `packages/react/src/menu/trigger/MenuTrigger.test.tsx:433,451`.
- `Menu.Trigger` can be created outside of `Menu.Root` via `Menu.createHandle()` and passed as a `handle` prop to `Menu.Root` `packages/react/src/menu/trigger/MenuTrigger.test.tsx:193`.

## State model (controlled/uncontrolled, defaults, transitions)

- In the default (uncontrolled) mode, clicking the trigger toggles the menu open/closed: a closed menu becomes open after a click `packages/react/src/menu/trigger/MenuTrigger.test.tsx:66-84`.
- In controlled mode (`open` state + `onOpenChange`), the `handle.isOpen` property reflects the controlled state `packages/react/src/menu/trigger/MenuTrigger.test.tsx:193-237`.
- When a controlled close is "vetoed" (the `onOpenChange` handler does not call `setOpen(false)` for a close event), `data-popup-open` remains on the trigger and `handle.isOpen` remains `true` `packages/react/src/menu/trigger/MenuTrigger.test.tsx:192-237`.
- When the trigger is opened via click, the popup carries `data-open` `packages/react/src/menu/trigger/MenuTrigger.test.tsx:83`.

## Keyboard interactions

- Pressing `ArrowDown` on a native `Menu.Trigger` (HTMLButtonElement) opens the menu `packages/react/src/menu/trigger/MenuTrigger.test.tsx:87-123`.
- Pressing `ArrowUp` on a native `Menu.Trigger` opens the menu `packages/react/src/menu/trigger/MenuTrigger.test.tsx:87-123`.
- Pressing `Enter` on a native `Menu.Trigger` does NOT open the menu (skipped for native buttons) `packages/react/src/menu/trigger/MenuTrigger.test.tsx:95-97`.
- Pressing `Space` (key `' '`) on a native `Menu.Trigger` does NOT open the menu (skipped for native buttons) `packages/react/src/menu/trigger/MenuTrigger.test.tsx:95-97`.
- For a non-native button (`render={<span />}`, `nativeButton={false}`), pressing `ArrowDown`, `ArrowUp`, `Enter`, or `Space` all open the menu `packages/react/src/menu/trigger/MenuTrigger.test.tsx:89-123`.
- Keyboard-triggered open is blocked when `preventBaseUIHandler()` is called in the `onClick` handler `packages/react/src/menu/trigger/MenuTrigger.test.tsx:448-468`.

## Focus management

- No focus-management behavior (e.g. where focus moves after open/close) is tested in this file.

## Accessibility (roles, aria-*, id linking)

- The trigger renders `aria-haspopup` when inside a `Menu.Root` `packages/react/src/menu/trigger/MenuTrigger.test.tsx:128-137`.
- The trigger renders `aria-expanded="false"` when the menu is closed `packages/react/src/menu/trigger/MenuTrigger.test.tsx:139-148`.
- After the menu opens, the trigger renders `aria-expanded="true"` `packages/react/src/menu/trigger/MenuTrigger.test.tsx:150-171`.
- The trigger renders `data-popup-open` when the menu is open `packages/react/src/menu/trigger/MenuTrigger.test.tsx:169`.
- The trigger renders `data-pressed` when the menu is open (after a click) `packages/react/src/menu/trigger/MenuTrigger.test.tsx:189`.
- When `Menu.Trigger` is nested inside a `Popover` and renders as a native button, it does NOT get an explicit `role` attribute `packages/react/src/menu/trigger/MenuTrigger.test.tsx:471-489`.
- When `Menu.Trigger` is nested inside a `Popover` and renders as a non-native element (`render={<span />}`, `nativeButton={false}`), it gets `role="button"` `packages/react/src/menu/trigger/MenuTrigger.test.tsx:491-509`.

## DOM structure & portal behavior

- The trigger renders as an `HTMLButtonElement` by default `packages/react/src/menu/trigger/MenuTrigger.test.tsx:14-21`.
- The trigger can be overridden to render a different element via `render` prop (e.g. `<span />`) `packages/react/src/menu/trigger/MenuTrigger.test.tsx:89-91`.
- No portal behavior is tested for the trigger itself (portal tests are scoped to `Menu.Portal`/`Menu.Positioner`/`Menu.Popup`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- Calling `event.preventBaseUIHandler()` in `onMouseDown` prevents the menu from opening on a mouse click `packages/react/src/menu/trigger/MenuTrigger.test.tsx:430-446`.
- Calling `event.preventBaseUIHandler()` in `onClick` prevents the menu from opening on a keyboard `Enter` press `packages/react/src/menu/trigger/MenuTrigger.test.tsx:448-468`.

## Edge cases (rapid interactions, unmount, nesting)

- With `openOnHover` and `delay={0}`, an impatient click (within `PATIENT_CLICK_THRESHOLD` of 500ms after hover) does NOT close the menu; the menu stays open `packages/react/src/menu/trigger/MenuTrigger.test.tsx:245-261`.
- With `openOnHover` and `delay={0}`, a patient click (after `PATIENT_CLICK_THRESHOLD` has elapsed since hover) DOES close the menu `packages/react/src/menu/trigger/MenuTrigger.test.tsx:263-284`.
- With `openOnHover` and `delay={0}`, an impatient click "sticks" the menu open: after `mouseLeave`, the menu remains open even after 1ms more passes `packages/react/src/menu/trigger/MenuTrigger.test.tsx:286-307`.
- With `openOnHover` and `delay={0}`, a patient click does NOT stick: after `mouseLeave`, the menu closes `packages/react/src/menu/trigger/MenuTrigger.test.tsx:309-331`.
- With `openOnHover` and `delay={300}`, clicking before the hover delay completes (at 100ms) opens the menu and sticks it open through `mouseLeave` `packages/react/src/menu/trigger/MenuTrigger.test.tsx:333-362`.
- With `openOnHover` and `delay={100}`, re-hovering and clicking within the patient threshold after the delay keeps the menu open `packages/react/src/menu/trigger/MenuTrigger.test.tsx:364-396`.
- In Chromium (skipped in jsdom), a hover-opened menu stays open when `mouseUp` lands outside the trigger DOM element but within its bounding rect `packages/react/src/menu/trigger/MenuTrigger.test.tsx:399-427`.
- Throwing: rendering `Menu.Trigger` without a `Menu.Root` ancestor or a `handle` prop throws `"Base UI: <Menu.Trigger> must be either used within a <Menu.Root> component or provided with a handle."` `packages/react/src/menu/trigger/MenuTrigger.test.tsx:23-33`.
- Nesting: `Menu.Trigger` inside a `Popover.Popup` (inside a `Popover.Root open`) renders correctly and has no `role` attribute when native, or `role="button"` when non-native `packages/react/src/menu/trigger/MenuTrigger.test.tsx:471-509`.

## Shared harness dependencies

- `#test-utils` (resolves to `packages/react/test/index.ts`) — provides `createRenderer`, `describeConformance`, and `isJSDOM` `packages/react/src/menu/trigger/MenuTrigger.test.tsx:7`.
- `PATIENT_CLICK_THRESHOLD` (imported from `../../internals/constants`, value `500`) — defines the threshold between impatient and patient clicks in `openOnHover` tests `packages/react/src/menu/trigger/MenuTrigger.test.tsx:8`.
- `@testing-library/user-event` (`userEvent.setup()`) — used for `user.click()` and `user.keyboard()` interactions `packages/react/src/menu/trigger/MenuTrigger.test.tsx:3,12`.
- `@mui/internal-test-utils` (`act`, `fireEvent`, `flushMicrotasks`, `screen`) — used for rendering, querying, and low-level event dispatch `packages/react/src/menu/trigger/MenuTrigger.test.tsx:4`.
