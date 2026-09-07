## Public API surface (props, parts, subcomponents)

- `Menu.Popup` accepts a `finalFocus` prop that controls where focus goes when the menu closes. It can be a React ref object (`finalFocus={inputRef}`) `packages/react/src/menu/popup/MenuPopup.test.tsx:115`, a function returning an element (`finalFocus={getRef}`) `packages/react/src/menu/popup/MenuPopup.test.tsx:153`, a function returning `true` to force default (trigger) focus `packages/react/src/menu/popup/MenuPopup.test.tsx:214`, a function returning `null` to fall back to default behavior `packages/react/src/menu/popup/MenuPopup.test.tsx:243`, or literal `false` to disable focus restoration entirely `packages/react/src/menu/popup/MenuPopup.test.tsx:185`.
- `Menu.Popup` accepts standard DOM passthrough props such as `data-testid` `packages/react/src/menu/popup/MenuPopup.test.tsx:51`.
- `Menu.Popup` must be rendered within `Menu.Positioner`; rendering it directly inside `Menu.Root` without a positioner throws `Base UI: MenuPositionerContext is missing. MenuPositioner parts must be placed within <Menu.Positioner>.` `packages/react/src/menu/popup/MenuPopup.test.tsx:24-40`.
- `Menu.Popup` passes `describeConformance` checks when rendered inside `Menu.Root open` > `Menu.Portal` > `Menu.Positioner`, with its root DOM node being an `HTMLDivElement` `packages/react/src/menu/popup/MenuPopup.test.tsx:11-22`.
- `Menu.Popup` renders `data-instant` attribute reflecting an inherited "instant" open type (e.g. `data-instant="click"`) during specific transition scenarios, described further below `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:381`.
- `Menu.Popup` supports `data-starting-style` attribute presence during enter transitions (see State model / DOM structure sections) `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:46`.

## State model (controlled/uncontrolled, defaults, transitions)

- A submenu that is uncontrolled and rendered with `defaultOpen` while its parent menu opens plays an enter transition (`[data-starting-style]` observed at least once) `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:74-110`.
- A submenu that is controlled (`open`) and already open when its parent menu opens also plays the enter transition `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:112-148`.
- A top-level menu popup that is open on first render (`Menu.Root defaultOpen`) does NOT play an enter transition (`[data-starting-style]` never observed within 100ms) `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:150-175`.
- A submenu inside a menu that is both open on first render (`defaultOpen` on both root and submenu) does not play an enter transition, for either the menu popup or the submenu popup `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:177-214`.
- When the parent `Menu.Portal` uses `keepMounted`, a submenu with `defaultOpen` does NOT play its enter transition when the parent opens via trigger click — this is called out as a known limitation because the submenu subtree exists from page load and its mount cannot be tied to the parent's reveal `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:216-257`.
- A submenu opened directly by the user (clicking the submenu trigger item while the parent menu is already open via `defaultOpen`) plays the enter transition `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:259-294`.
- A submenu that is initially open (`defaultOpen`) while its parent menu opens via keyboard (`Enter` on the focused trigger, i.e. an "instant" open) still plays the enter transition (goes through the `'starting'` phase) alongside the parent's own instant-opened transition; suppressing the transition via `[data-instant]` is left as a styling choice for the consumer, not built-in framework behavior `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:296-340`.
- When the parent menu opens instantly (keyboard `Enter`), the parent popup carries `data-instant="click"` `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:381`, and an initially-open submenu popup also transiently carries `[data-instant="click"]` (the inherited value) during its enter frames only; this attribute is cleared once the initial reveal settles, so it cannot suppress later transitions `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:342-390`.
- The inherited `instantType` seed does not outlive the initial reveal: when the submenu is closed and reopened later via a controlled `open` prop flip (bypassing `setOpen`), the reopened submenu popup does NOT carry `[data-instant]` `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:392-446`.
- If a controlled close interrupts the submenu's enter transition before it completes (e.g. closing before a 300ms CSS transition finishes) and it is then reopened, the reopened submenu popup does not carry `[data-instant]` `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:448-512`.
- If the submenu's popup subtree is not initially rendered (e.g. conditionally rendered later) while the parent instantly opens, then rendered afterward once the parent's reveal has settled, the newly rendered submenu popup does not carry `[data-instant]` — the inherited "instant" seed is still cleared even though there was no popup element present to run the animations-finished cleanup against `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:514-565`.
- A submenu that mounts closed during the parent's instant-open transition (`open={false}` while `defaultOpen` is also set, with `open` winning over `defaultOpen`) does not inherit the parent's `instantType`; when it is later opened programmatically via a controlled `open` prop flip, the submenu popup does not carry `[data-instant]` `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:567-617`.

## Keyboard interactions

- When `Menu.Popup` is used within a `ToolbarRootContext.Provider`, pressing `ArrowRight` (a toolbar navigation key) on the popup is stopped from bubbling to an ancestor `onKeyDown` handler (`onParentKeyDown` is not called) `packages/react/src/menu/popup/MenuPopup.test.tsx:42-65`.
- Ordinary key events (e.g. `F1`, not a toolbar navigation key) are NOT blocked and do bubble to the ancestor `onKeyDown` handler `packages/react/src/menu/popup/MenuPopup.test.tsx:66-68`.
- Pressing `Enter` while the trigger is focused opens the menu "instantly" (as opposed to a pointer/click open), which is distinguished for the purposes of the enter-transition and `data-instant` behavior described above `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:328-332`.

## Focus management

- By default, when the popup closes (e.g. clicking a `Menu.Item` that closes the menu), focus returns to the `Menu.Trigger` element `packages/react/src/menu/popup/MenuPopup.test.tsx:72-103`.
- If `finalFocus` is set to a ref object pointing to an arbitrary element, closing the menu moves focus to that element instead of the trigger `packages/react/src/menu/popup/MenuPopup.test.tsx:105-141`.
- If `finalFocus` is a function returning an element, closing the menu moves focus to the element returned by that function `packages/react/src/menu/popup/MenuPopup.test.tsx:143-175`.
- If `finalFocus` is `false`, closing the menu does not move focus to the trigger (focus restoration is disabled) `packages/react/src/menu/popup/MenuPopup.test.tsx:177-204`.
- If `finalFocus` is a function returning `true`, closing the menu moves focus to the trigger (the default target) `packages/react/src/menu/popup/MenuPopup.test.tsx:206-233`.
- If `finalFocus` is a function returning `null`, the default focus-restoration behavior is used (focus moves to the trigger) `packages/react/src/menu/popup/MenuPopup.test.tsx:235-260`.

## Accessibility (roles, aria-*, id linking)

N/A — neither test file asserts roles, aria attributes, or id linking for `Menu.Popup` itself. (The tests do query elements by role, e.g. `screen.getByRole('button', { name: 'Trigger' })` and `screen.getByRole('menuitem', { name: 'Submenu' })`, but these assert the existence/queryability of trigger and item roles, not a role or aria-* contract for the popup element.) `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:102,286`

## DOM structure & portal behavior

- `Menu.Popup` must be a descendant of `Menu.Positioner`, which itself is expected inside `Menu.Portal` inside `Menu.Root` `packages/react/src/menu/popup/MenuPopup.test.tsx:11-22`.
- Rendering `Menu.Popup` directly under `Menu.Root` without `Menu.Positioner` throws at render/mount time `packages/react/src/menu/popup/MenuPopup.test.tsx:24-40`.
- When `Menu.Portal keepMounted` is used, the popup subtree (including a nested submenu's subtree) exists in the DOM from initial render rather than only once opened `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:216-257`.
- Submenus are rendered via their own nested `Menu.Portal` > `Menu.Positioner` > `Menu.Popup` structure inside `Menu.SubmenuRoot`, independent of the parent popup's portal `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:79-100`.
- During the enter transition, the popup element carries a `[data-starting-style]` attribute for at least one animation frame, which is what CSS-driven enter transitions key off of `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:41-47`.
- The popup element carries a `data-instant` attribute (e.g. `data-instant="click"`) when opened via an instant interaction such as a keyboard `Enter`, and this attribute is absent otherwise `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:381,388`.

## Events (names, payload shape, bubbling, preventDefault semantics)

- Native `keydown` `KeyboardEvent`s are used to probe bubbling: a toolbar-navigation key (`ArrowRight`) dispatched on the popup with `bubbles: true, cancelable: true` does not reach a parent `onKeyDown` handler, while a `F1` keydown does reach it, unmodified (`event.key === 'F1'`) `packages/react/src/menu/popup/MenuPopup.test.tsx:60-68`.
- No custom Base UI events (e.g. `onOpenChange` payloads) are exercised in these two test files for `Menu.Popup` specifically.

## Edge cases (rapid interactions, unmount, nesting)

- Nesting: a submenu (`Menu.SubmenuRoot` containing its own `Menu.Popup`) nested inside a parent `Menu.Popup` is exercised extensively for enter-transition timing; see State model section for the full matrix of `defaultOpen`/controlled `open`/`keepMounted`/instant-open combinations `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:74-617`.
- Interrupting an in-progress enter transition: closing a submenu via a controlled `open` flip before its CSS enter transition (300ms) completes, then reopening it, results in no stale `[data-instant]` attribute — the transition's pending "animations finished" cleanup is interrupted correctly and does not leak state into the next open `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:448-512`.
- Popup element absent at the time the parent's transition seed is set: if the submenu's popup subtree is conditionally not rendered while the parent's instant-open reveal settles, then later mounted, the seeded `instantType` is still correctly cleared despite there being no element for a ref-based "animations finished" callback to observe at seed time `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:514-565`.
- Controlled `open` prop changes bypass `setOpen`, which the tests use specifically to verify inherited transition state (`instantType`) does not leak across programmatic (non-`setOpen`) open/close cycles `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:392-395,567-572`.
- `describe.skipIf(isJSDOM)` is applied to the entire enter-transition test suite, so these transition/animation-frame-dependent behaviors are only verified in the Chromium (browser) test environment, not in jsdom `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:66`.

## Shared harness dependencies

- `#test-utils` (resolves to `packages/react/test/index.ts`) — provides `createRenderer`, `describeConformance`, and `isJSDOM`, used by both `packages/react/src/menu/popup/MenuPopup.test.tsx:4` and `packages/react/src/menu/popup/MenuPopupEnterTransition.test.tsx:8`.
