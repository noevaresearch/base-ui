# Menu Root — Behavior Spec

Scope: `Menu.Root` only, as proven by `packages/react/src/menu/root/MenuRoot.test.tsx` and `packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx`. Related component families in these files (`Menu.Portal`, `Menu.Positioner`, `Menu.Popup`, `Menu.Item`, `Menu.SubmenuRoot`, `Menu.SubmenuTrigger`, `Menu.Trigger`, `Dialog.Root`, `AlertDialog.Root`) are included only insofar as these root test files exercise `Menu.Root` behavior directly.

## Public API surface (props, parts, subcomponents)

- `Menu.Root` is the root stateful component; its trigger/popup parts are `Menu.Trigger`, `Menu.Portal`, `Menu.Positioner`, `Menu.Popup`, `Menu.Item`, `Menu.SubmenuRoot`, `Menu.SubmenuTrigger`, `Menu.Group`, `Menu.Item`, `Menu.CheckboxItem`, `Menu.RadioGroup`, `Menu.RadioItem`, and `Menu.Viewport`, all exercised in the root tests (`packages/react/src/menu/root/MenuRoot.test.tsx:2900-3006`, `:3008-3032`; `packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:1036`).
- Accepts `open` (controlled), `defaultOpen` (uncontrolled), `onOpenChange`, `onOpenChangeComplete`, `modal`, `orientation`, `disabled`, `highlightItemOnHover`, `actionsRef`, `closeParentOnEsc` (on nested submenus), `triggerId`, `defaultTriggerId`, `handle`, and a render-prop `children` receiving `{ payload }` (`packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:494`, `:496-512`, `:1013`; `packages/react/src/menu/root/MenuRoot.test.tsx:1077-1116`, `:1917-2074`, `:304-513`).
- The trigger/root supports a `handle` (created via `Menu.createHandle<number>()`) that exposes imperative methods `open(triggerId)`, `close()`, and a read-only `isOpen` property; triggers register to the handle via `handle={...}` or `id` (`packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:17-60`, `:1318-1408`).
- `Menu.Root` children can be a render-prop function receiving `{ payload }`; a `Menu.Trigger` with a `payload` prop supplies the value to that render prop when it is the active trigger (`packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:345-382`, `:814-856`).
- A constrained `modal` prop on nested `Menu.SubmenuRoot` (a `modal` boolean on a submenu root) is not supported; passing it emits a development-only console warning `'Base UI: The `modal` prop is not supported on nested menus. It will be ignored.'` and the warning is suppressed in production (`packages/react/src/menu/root/MenuRoot.test.tsx:83-104`).
- `actionsRef` accepts an object with `unmount` and `close` methods; calling `unmount` unmounts (removes) the popup from the DOM (`packages/react/src/menu/root/MenuRoot.test.tsx:1866-1914`, `:2751-2796`).
- Menu items support a `nativeButton` prop and `closeOnClick` (false, e.g. on a `Menu.Item` rendering a `Dialog.Trigger`), plus `label`, `disabled`, `onClick`, and `data-testid` (`packages/react/src/menu/root/MenuRoot.test.tsx:449-501`, `:1120-1148`, `:503-537`, `:2590-2642`).

## State model (controlled/uncontrolled, defaults, transitions)

- The menu is closed by default; with `open={false}` (controlled) the popup is absent, and with `open={true}` it is present (`packages/react/test/popupConformanceTests.tsx:35-45`).
- In uncontrolled mode, clicking the trigger toggles open/closed (`packages/react/test/popupConformanceTests.tsx:48-66`).
- `open` (controlled) and `defaultOpen` (uncontrolled) both drive visibility; with `defaultOpen` the menu is initially open (`packages/react/src/menu/root/MenuRoot.test.tsx:113`, `:119`, `:1679`, `:3009`; `packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:492-516`).
- Controlled `open` combined with `onOpenChange` allows fully programmatic open/close (e.g. an external button opening, item click closing), with focus returned to the opener (`packages/react/src/menu/root/MenuRoot.test.tsx:1077-1116`; `packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:419-491`, `:898-988`).
- `defaultTriggerId` establishes the initially active trigger in a multi-trigger menu; with `defaultOpen` + `defaultTriggerId="trigger-2"` the popup content reflects trigger-2's payload (`packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:492-516`, `:989-1014`).
- `onOpenChange` is called with `(nextOpen: boolean, details)`; `details.reason` carries the close/open reason (e.g. `REASONS.itemPress`, `REASONS.cancelOpen`, `REASONS.triggerHover`, `REASONS.siblingOpen`, `REASONS.outsidePress`) and `details.event` carries the triggering event (e.g. a `MouseEvent`) (`packages/react/src/menu/root/MenuRoot.test.tsx:210-237`, `:2633-2641`, `:2684`, `:891-895`, `:2923`).
- `onOpenChange` callbacks can call `details.cancel()` to prevent the state change (e.g. preventing opening while uncontrolled: menu stays closed) (`packages/react/src/menu/root/MenuRoot.test.tsx:2688-2708`), and `details.preventUnmountOnClose()` to keep the popup mounted after close (`packages/react/src/menu/root/MenuRoot.test.tsx:2710-2747`, `:1866-1914`).
- `onOpenChangeComplete` is called with the new open boolean once the enter/exit animation finishes (or immediately when no animation is defined); it is called on close-with-no-exit-animation, on close-after-exit-animation-finish, on open-with-no-enter-animation, on open-after-enter-animation-finish, and is NOT called on mount when not open (`packages/react/src/menu/root/MenuRoot.test.tsx:1917-2074`).
- Popup visibility transitions: opening/closing apply `data-open`, `data-starting-style`/`data-ending-style` transition hooks, and the popup is unmounted by default on close (unless `keepMounted`/`preventUnmountOnClose`) (`packages/react/src/menu/root/MenuRoot.test.tsx:2792-2795`, `:1416-1464`, `:3009`; `packages/react/test/popupConformanceTests.tsx:134-153`).
- When a modal menu is opened via a hover-open trigger opened with `openOnHover`, an impatient click (before `PATIENT_CLICK_THRESHOLD`) that closes via item press still allows re-open on subsequent hover (`packages/react/src/menu/root/MenuRoot.test.tsx:2320-2354`).
- `defaultOpen: true` with no trigger hover: hovering out of the popup does not close it (no trigger-hover open context) (`packages/react/src/menu/root/MenuRoot.test.tsx:1677-1692`).
- A menu opened externally (controlled `open`) does not close when the pointer hovers out of the popup, but a menu opened by its trigger with `openOnHover` does close on hover-out (`packages/react/src/menu/root/MenuRoot.test.tsx:1694-1752`).
- A hover-opened menu is treated as modal after an impatient click completes: a backdrop (`role="presentation"`) appears in front of the positioner after the click (`packages/react/src/menu/root/MenuRoot.test.tsx:2356-2388`).

## Keyboard interactions

- Opening with `[Enter]` on a focused trigger opens the menu and focuses the first item; subsequent `ArrowDown`/`ArrowUp` move the highlighted item (`packages/react/src/menu/root/MenuRoot.test.tsx:126-158`).
- `Home`/`End` move highlight to the first/last item (`packages/react/src/menu/root/MenuRoot.test.tsx:239-264`).
- `ArrowDown` opens the menu focusing the first item; `ArrowUp` opens focusing the last item; `[Enter]` opens focusing the first item (`packages/react/src/menu/root/MenuRoot.test.tsx:1339-1396`).
- Arrow-key navigation operates across `Menu.Group`s (Apple/Banana/Cherry) (`packages/react/src/menu/root/MenuRoot.test.tsx:160-208`).
- Disabled items are included during arrow-key navigation (they receive focus with `aria-disabled="true"`) (`packages/react/src/menu/root/MenuRoot.test.tsx:266-297`).
- Items hidden with CSS (`display: none`) are skipped during arrow navigation and get `tabindex="-1"` (`packages/react/src/menu/root/MenuRoot.test.tsx:299-341`).
- Text (typeahead) navigation: typing characters moves highlight to the matching item, which then gets `tabindex="0"` (`packages/react/src/menu/root/MenuRoot.test.tsx:344-377`).
- Text navigation skips CSS-hidden items (`packages/react/src/menu/root/MenuRoot.test.tsx:379-410`), natively-disabled items (no `data-highlighted`, `tabindex="-1"`) (`packages/react/src/menu/root/MenuRoot.test.tsx:412-447`), and non-stringifiable items (empty children, nested divs, `undefined`, `null` children) (`packages/react/src/menu/root/MenuRoot.test.tsx:503-537`).
- Text navigation matches on the `label` prop (`packages/react/src/menu/root/MenuRoot.test.tsx:449-501`) and matches diacritic characters both when typed and as the first character of targets (`packages/react/src/menu/root/MenuRoot.test.tsx:539-596`).
- During a typeahead session, pressing `Space` appends to the typed text and does NOT activate the item's `onClick` nor open a submenu; a `Space` on a focused submenu trigger with no active typeahead does open the submenu (`packages/react/src/menu/root/MenuRoot.test.tsx:598-678`, `:680-691`).
- Typeahead continues across `space + numeric` suffixes and matches submenu trigger labels (`packages/react/src/menu/root/MenuRoot.test.tsx:693-763`).
- Nested menus open with the orientation-appropriate arrow key (`ArrowRight` opening/`ArrowLeft` closing in LTR vertical, reversed in RTL vertical, `ArrowDown`/`ArrowUp` in horizontal) (`packages/react/src/menu/root/MenuRoot.test.tsx:767-815`).
- `[Escape]` closes a nested submenu, returning focus to its trigger; pressing Escape again closes each parent level upward (`packages/react/src/menu/root/MenuRoot.test.tsx:1027-1074`, `:1583-1590`).
- By default, pressing `Escape` inside a nested submenu closes only that submenu (leaving the root menu open); with `closeParentOnEsc=true` on the submenu, `Escape` closes the entire tree including the root (`packages/react/src/menu/root/MenuRoot.test.tsx:1549-1630`).
- Keyboard item activation (Enter on a focused item) closes the menu with `reason === REASONS.itemPress` and the activation click carries `detail === 0` (classified as keyboard/instant activation) (`packages/react/src/menu/root/MenuRoot.test.tsx:210-237`).
- Keyboard navigation works identically regardless of which trigger (in a multi-trigger menu) opened the popup; submenu ArrowRight/ArrowLeft/Escape work from any opener (`packages/react/src/menu/root/MenuRoot.test.tsx:519-579`, `:1068-1128`).
- Keyboard navigation skips `null`/filtered-out dynamic items and loops from last to first (`packages/react/src/menu/root/MenuRoot.test.tsx:3127-3186`).

## Focus management

- After the menu is opened by keyboard, the first item has `tabIndex = 0` and all other items `tabIndex = -1`; `ArrowDown`-open focuses first item, `ArrowUp`-open focuses last item (`packages/react/src/menu/root/MenuRoot.test.tsx:1339-1396`).
- After the menu is closed (own item click), focus returns to the trigger (`packages/react/src/menu/root/MenuRoot.test.tsx:1398-1414`).
- After close via `Escape` when the closing menu receives a `mouseleave` during an exit transition, focus returns to the trigger (`packages/react/src/menu/root/MenuRoot.test.tsx:1416-1464`).
- After close of a `keepMounted` menu (portal closed but not unmounted), focus still returns to the trigger (`packages/react/src/menu/root/MenuRoot.test.tsx:1466-1487`).
- Opening a menu programmatically via a separate opener button, then clicking an item to close, returns focus to the opener (`packages/react/src/menu/root/MenuRoot.test.tsx:1077-1116`).
- Tabbing forward out of an open non-modal (`modal: false`) menu moves focus to the next tabbable element and closes the menu; shift-tabbing out moves focus back to the trigger and closes the menu (`packages/react/src/menu/root/MenuRoot.test.tsx:1491-1546`).
- Tabbing out of a `keepMounted` menu before it closes leaves no tabbable item behind: after tab-out the previously-focused item's `tabindex` returns to `-1` (`packages/react/src/menu/root/MenuRoot.test.tsx:2751-2796`).
- Closing a nested submenu returns focus to its submenu trigger; closing each level upward returns focus level by level (`packages/react/src/menu/root/MenuRoot.test.tsx:1027-1074`, `:897-930`, `:923-929`).
- `Shift+Tab` from a nested menu (submenu item focused) closes the submenu and returns focus to the submenu trigger (`packages/react/src/menu/root/MenuRoot.test.tsx:897-930`).
- `highlightItemOnHover` (default true): moving the mouse over an item focuses it; with `highlightItemOnHover={false}`, mouse movement neither focuses items nor marks submenu triggers with `data-highlighted` (`packages/react/src/menu/root/MenuRoot.test.tsx:2930-3006`).
- When a nested dialog is open, `Shift+Tab` inside the dialog does NOT close the menu or the dialog (`packages/react/src/menu/root/MenuRoot.test.tsx:1119-1187`).
- When a nested `AlertDialog` popup takes focus, the pointer leaving the triggering item keeps focus inside the alert dialog popup (not in the menu popup) (`packages/react/src/menu/root/MenuRoot.test.tsx:1189-1250`).
- When a nested dialog has pending deferred focus (via `requestAnimationFrame`), the pointer leaving the triggering menu item keeps the pending focus in the dialog popup rather than the menu popup (`packages/react/src/menu/root/MenuRoot.test.tsx:1252-1335`).

## Accessibility (roles, aria-*, id linking)

- The popup has role `menu` (`packages/react/test/popupConformanceTests.tsx:71-76`; `packages/react/src/menu/root/MenuRoot.test.tsx:115`, `:122`).
- The trigger has `aria-controls` pointing to the popup's id, `aria-expanded="false"` when closed and `"true"` when open, and `aria-haspopup` equal to the expected value (`packages/react/test/popupConformanceTests.tsx:79-113`).
- A custom popup `id` prop is honored: the trigger's `aria-controls` equals the popup's `id` (`packages/react/test/popupConformanceTests.tsx:115-120`).
- `aria-orientation` is set to `"horizontal"` on a horizontal popup and is NOT rendered on a vertical popup (implicitly vertical) (`packages/react/src/menu/root/MenuRoot.test.tsx:112-123`).
- Disabled items render `aria-disabled="true"` and `data-disabled` on items/checkbox/radio when the root is `disabled` (`packages/react/src/menu/root/MenuRoot.test.tsx:266-297`, `:3008-3032`).
- A detached trigger gets `aria-expanded="true"` when its menu is open (`packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:157`, `:1348`, `:1396-1397`).
- Root menu portal ownership: an `aria-owns` element inside the document (outside the popup) points at the root portal's id and has no `role`; a submenu's portal ownership is an element with `role="group"` (an allowed menu child) inside the parent menu, and the submenu trigger itself has no `aria-owns` (`packages/react/src/menu/root/MenuRoot.test.tsx:833-868`).
- The internal modal backdrop renders as a sibling immediately before the positioner with `role="presentation"` when `modal` is true, and is absent when `modal` is false (`packages/react/src/menu/root/MenuRoot.test.tsx:1633-1675`).
- The trigger carries `data-popup-open` when a menu is open (asserted toggling between open/closed states) (`packages/react/src/menu/root/MenuRoot.test.tsx:2734`, `:2740`).
- `data-open` is present on the popup/menu when open and removed when closed (`packages/react/test/popupConformanceTests.tsx:101`; `packages/react/src/menu/root/MenuRoot.test.tsx:2793`).
- The active/highlighted item is marked with a `data-highlighted` attribute (asserted absent for disabled/hidden items and with `highlightItemOnHover={false}`) (`packages/react/src/menu/root/MenuRoot.test.tsx:446`, `:3003-3004`, `:3059`).

## DOM structure & portal behavior

- The popup is rendered inside `Menu.Portal > Menu.Positioner > Menu.Popup`, with `Menu.Item`(s) inside the popup (`packages/react/src/menu/root/MenuRoot.test.tsx:40-56`, `:3226-3277`).
- The popup is unmounted when closed by default (not found via `queryByRole('menu')`), and mounted when open (`packages/react/test/popupConformanceTests.tsx:37-45`, `:54-64`; `packages/react/src/menu/root/MenuRoot.test.tsx:54`, `:1344`).
- With `keepMounted` on the portal, the popup remains in the DOM but is inaccessible (`toBeInaccessible()`) when closed (`packages/react/test/popupConformanceTests.tsx:134-153`; `packages/react/src/menu/root/MenuRoot.test.tsx:2792-2795`).
- The portal wrapper carries a `data-base-ui-portal` attribute; the root menu's `aria-owns` element is a `span[aria-owns]` (`packages/react/src/menu/root/MenuRoot.test.tsx:840-846`).
- When switching between multiple triggers, the popup and positioner DOM nodes are reused (not recreated) across trigger changes (`packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:384-417`, `:858-896`).
- Switching triggers in an animation-enabled menu leaves no inline `scale` style on the popup (`packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:1016-1065`).
- Scroll locking: a modal menu opened via mouse applies scroll lock (constrains `overflow`/`data-base-ui-scroll-locked` on the document), but a modal menu opened via touch does NOT apply scroll lock unless the touch-opened popup covers the viewport width; a narrower touch-opened popup does not lock scroll (`packages/react/src/menu/root/MenuRoot.test.tsx:1755-1864`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onOpenChange(nextOpen, details)` — `details.reason` is one of the `REASONS` values; asserted reasons: `itemPress` (item activated by click or keyboard) (`packages/react/src/menu/root/MenuRoot.test.tsx:233`, `:2638`), `cancelOpen` (click-drag outside and release) (`packages/react/src/menu/root/MenuRoot.test.tsx:2684`), `triggerHover` (stale hover close must NOT fire after an item press close) (`packages/react/src/menu/root/MenuRoot.test.tsx:1018-1023`), `siblingOpen` (root must NOT report close when a sibling submenu opens) (`packages/react/src/menu/root/MenuRoot.test.tsx:891-895`), `outsidePress` (click outside a nested dialog must not close the dialog) (`packages/react/src/menu/root/MenuRoot.test.tsx:2923`).
- `onOpenChange` details include `details.trigger?.id` — the id of the trigger that caused the change (used to control which trigger is active) (`packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:429-431`, `:916-918`).
- `onOpenChange` is called exactly once per menu when a submenu item closes the tree: root reports one `false`, submenu reports one `false`, and a scheduled stale hover close is not dispatched (`packages/react/src/menu/root/MenuRoot.test.tsx:979-1025`).
- `onOpenChange` details carry the originating `event` (e.g. a `MouseEvent`) whose `detail` distinguishes keyboard (instant) activation (`detail === 0`) from mouse drag-release activation (`detail === 1`) (`packages/react/src/menu/root/MenuRoot.test.tsx:236`, `:2641`).
- Cancellable change: calling `details.cancel()` inside `onOpenChange` on open prevents the menu from opening while uncontrolled (`packages/react/src/menu/root/MenuRoot.test.tsx:2688-2708`).
- `onOpenChangeComplete(nextOpen)` is called with the resulting open boolean after open/close animations complete (or immediately without animations) (`packages/react/src/menu/root/MenuRoot.test.tsx:1917-2074`).
- Item `onClick` fires on click/drag/release activation (the item press closes the menu and calls `onClick` once) (`packages/react/src/menu/root/MenuRoot.test.tsx:2590-2642`), and does NOT fire during typeahead Space appends (`packages/react/src/menu/root/MenuRoot.test.tsx:598-633`). UNVERIFIED — inferred from `packages/react/src/menu/root/MenuRoot.test.tsx:604-612` that the `onClick` guard scope is global to the root tests; the individual item click-close-on-activation detail is asserted at `:2590-2642`.

## Edge cases (rapid interactions, unmount, nesting)

- Click-drag-release activation requires the pointer to rest past `PATIENT_CLICK_THRESHOLD`; mouseup on a menu item outside a quick click activates the item and closes the menu (`packages/react/src/menu/root/MenuRoot.test.tsx:2590-2642`; `packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:706-759`, `:1256-1311`).
- Click-drag OUTSIDE and release (drag from trigger to an outside element) closes the menu with `reason === REASONS.cancelOpen` (`packages/react/src/menu/root/MenuRoot.test.tsx:2644-2685`).
- A "patient" click that opens the menu (beyond threshold) does not close the submenu on release, and opening a dialog from an item keeps the dialog open after a press-drag-release (the browser's synthetic click on the common ancestor must not be treated as an outside press on the dialog) (`packages/react/src/menu/root/MenuRoot.test.tsx:2798-2928`).
- A scheduled delayed hover-close is cancelled when an item press closes the tree first (no `REASONS.triggerHover` refire) (`packages/react/src/menu/root/MenuRoot.test.tsx:979-1025`).
- Re-open after `preventUnmountOnClose`: after a cycle of prevent-unmount + reopen, a later normal close does unmount, and state toggles through `data-popup-open` accordingly (`packages/react/src/menu/root/MenuRoot.test.tsx:2710-2747`).
- Clicking outside the deepest submenu of a nested tree closes all levels (root and all submenus) (`packages/react/src/menu/root/MenuRoot.test.tsx:932-977`, `:1184-1254`, `:632-704`).
- Opening a sibling submenu does not close the root menu (no `reason === siblingOpen` for the root) (`packages/react/src/menu/root/MenuRoot.test.tsx:870-895`).
- Hovering a sibling item closes a second-level submenu after a delay (`closeDelay`), and repeated `mousemove` over the sibling does not restart the close timer (`packages/react/src/menu/root/MenuRoot.test.tsx:2422-2497`).
- A submenu opens after a plain delay even when the pointer does not rest (moves continuously) (`packages/react/src/menu/root/MenuRoot.test.tsx:2505-2529`).
- A pending delayed submenu hover-open is cancelled when the pointer leaves the trigger via `mouseout` with `relatedTarget` outside, but is preserved when `mouseout` stays within the trigger's subtree (`packages/react/src/menu/root/MenuRoot.test.tsx:2531-2581`).
- Safe-polygon logic: submenu safePolygon `pointer-events` mutation does not clear body `pointer-events` styles on close, and is scoped to the parent menu when the portal is `keepMounted`; hovering the submenu after the root is hovered keeps both open (`packages/react/src/menu/root/MenuRoot.test.tsx:2135-2208`, `:2210-2258`).
- A parent submenu stays open after a third-level submenu closes due to sibling hover (`packages/react/src/menu/root/MenuRoot.test.tsx:2260-2313`).
- Disabled root (`disabled` on root with controlled open): items/checkbox/radio get `data-disabled`, submenu triggers get `data-disabled` and do not open a submenu, text-navigation does not highlight, and clicking an item neither fires `onClick` nor `onOpenChange` (`packages/react/src/menu/root/MenuRoot.test.tsx:3008-3115`).
- Nested menus ignore the `modal` prop with a dev-only warning (see Public API) (`packages/react/src/menu/root/MenuRoot.test.tsx:83-104`).
- Multiple roots/multiple detached triggers, imperative handle semantics: see the Shared harness dependencies section (`packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:14-295`, `:763-1408`).

## Shared harness dependencies

- `#test-utils` (resolved to `packages/react/test/index.ts`, which re-exports `@base-ui/utils/testUtils` plus `createRenderer`, `popupConformanceTests`, `resetBrowserPointer`, `enterWithMouse`, `moveMouse`, `firePointer`, `wait`, and `describeConformance`) (`packages/react/test/index.ts:1-11`).
- `createRenderer()` returns `{ render, rerender, setProps, clock }`; `render` is async-act wrapped and returns `{ user, rerender, setProps }`; `clock.withFakeTimers()` enables fake-timer control (`packages/react/test/createRenderer.ts:27-48`).
- The popup conformance suite runs against `Menu.Root` with `expectedPopupRole: 'menu'` and `triggerMouseAction: 'click'`, asserting controlled/uncontrolled open, ARIA attributes, and animation removal behavior (`packages/react/src/menu/root/MenuRoot.test.tsx:40-56`; `packages/react/test/popupConformanceTests.tsx:6-213`).
- `enterWithMouse(element)` fires `pointerEnter`+`mouseEnter`+`mouseMove` (mouse pointerType); `moveMouse(from, to)` fires pointer/mouse leave on `from` and enter+move on `to` (`packages/react/test/pointer.ts:3-21`).
- `firePointer` (`down`/`move`/`up`) fires pointer events honoring an explicit `timeStamp` (velocity-sensitive use only) (`packages/react/test/pointer.ts:27-61`).
- `wait(ms)` awaits a real `setTimeout` of the given milliseconds (`packages/react/test/wait.ts:4-8`).
- `resetBrowserPointer` is run in `beforeEach` and `BASE_UI_ANIMATIONS_DISABLED` is set true before each test (`packages/react/src/menu/root/MenuRoot.test.tsx:32-36`; `packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx:8-10`).
- The `TestMenuContents`/`ContainedTriggerMenu`/`DetachedTriggerMenu` fixtures render a root menu with items, a disabled item, a three-level submenu tree, and identifiers (`data-testid`s) used throughout (`packages/react/src/menu/root/MenuRoot.test.tsx:3190-3277`); `DetachedTriggerMenu` uses `Menu.Handle` (`useRefWithInit(() => new Menu.Handle())`) and a detached `Menu.Trigger` (`packages/react/src/menu/root/MenuRoot.test.tsx:3199-3214`).
- `Menu.createHandle<T>()` creates an imperative handle exposing `open(triggerId)`, `close()`, and `isOpen`; the detached-triggers suites exercise handle-backed roots, multi-trigger payload dispatch, imperative open/close, and multi-root handle handoff (see citations throughout `packages/react/src/menu/root/MenuRoot.detached-triggers.test.tsx`).
