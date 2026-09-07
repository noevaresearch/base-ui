# Menubar — behavior spec

Unit: `menubar` (`packages/react/src/menubar/`). Source of truth for this spec: `packages/react/src/menubar/Menubar.test.tsx` (single test file, 1484 lines). No `wraps-external:` field exists in this unit's `TODO.md` entry, so no third-party delegation applies.

The tests exercise `<Menubar />` as a coordinator over `Menu.Root`/`Menu.Trigger`/`Menu.Portal` components imported from `@base-ui/react/menu` (`packages/react/src/menubar/Menubar.test.tsx:11-12`). Three structural fixture variants are parameterized across most suites: contained triggers (`Menu.Root` per trigger inside the menubar, `packages/react/src/menubar/Menubar.test.tsx:1198-1264`), detached triggers (`Menu.Trigger handle`/`payload` pairs inside the menubar with a single `Menu.Root handle` rendered outside it, `packages/react/src/menubar/Menubar.test.tsx:1450-1470`), and multiple contained triggers (a render-prop wrapper holding several `Menu.Trigger payload` elements, `packages/react/src/menubar/Menubar.test.tsx:1472-1484`). Behavior is asserted to be identical across all three variants via `describe.for` (`packages/react/src/menubar/Menubar.test.tsx:126-130`).

## Public API surface (props, parts, subcomponents)

- `<Menubar />` forwards a DOM ref to an `HTMLDivElement` (conformance suite runs with `refInstanceof: window.HTMLDivElement`, `packages/react/src/menubar/Menubar.test.tsx:24-29`).
- Props exercised by tests:
  - `loopFocus: boolean` — enables/clamps roving focus wrap-around at the ends of the trigger list (`packages/react/src/menubar/Menubar.test.tsx:920-998`).
  - `disabled: boolean` — disables all child menus and triggers (`packages/react/src/menubar/Menubar.test.tsx:1001-1018`, `packages/react/src/menubar/Menubar.test.tsx:1170-1194`).
  - `orientation: 'vertical'` (at minimum) — reflected as `aria-orientation` on the root (`packages/react/src/menubar/Menubar.test.tsx:1082-1085`).
  - `modal: boolean` — combined with touch-opened menus, drives scroll-lock behavior (`packages/react/src/menubar/Menubar.test.tsx:790-917`).
  - `style` (and by conformance, `className`, ref, and prop spreading) pass through to the root element (`packages/react/src/menubar/Menubar.test.tsx:24-29`, `packages/react/src/menubar/Menubar.test.tsx:1200`).
- The children API is composition of `Menu` parts: `Menu.Root`, `Menu.Trigger`, `Menu.Portal`, `Menu.Positioner`, `Menu.Popup`, `Menu.Item`, `Menu.SubmenuRoot`, `Menu.SubmenuTrigger`, `Menu.RadioGroup`, `Menu.RadioItem` (`packages/react/src/menubar/Menubar.test.tsx:1198-1264`).
- Detached mode is part of the tested surface: `Menu.Trigger` accepts a `handle` plus a `payload`, and a matching `Menu.Root handle` elsewhere renders the menu via a render prop whose argument exposes the payload (`packages/react/src/menubar/Menubar.test.tsx:1266-1289`, `packages/react/src/menubar/Menubar.test.tsx:1450-1470`); `Menu.Handle` is instantiated with `useRefWithInit` (`packages/react/src/menubar/Menubar.test.tsx:1451`).
- `Menubar.Props` is the exported props type used by consumer fixtures (`packages/react/src/menubar/Menubar.test.tsx:1198`, `packages/react/src/menubar/Menubar.test.tsx:1450`, `packages/react/src/menubar/Menubar.test.tsx:1472`).
- Conformance: the root passes the standard `propsSpread`, `refForwarding`, `renderProp`, and `className` tests (`packages/react/src/menubar/Menubar.test.tsx:24-29`; suite membership defined in `packages/react/test/describeConformance.tsx:44-49`).

## State model (controlled/uncontrolled, defaults, transitions)

- Open/close state of each menu is owned by the individual `Menu.Root` (e.g. controlled `open`/`onOpenChange` passed straight through, `packages/react/src/menubar/Menubar.test.tsx:89-110`); the menubar coordinates which single menu is active rather than owning the state. UNVERIFIED — inferred from `packages/react/src/menubar/Menubar.test.tsx:99`, no test asserts menubar itself exposes `value`/`open` props.
- The menubar root exposes derived state via a `data-has-submenu-open` attribute that is present while any submenu is open and absent once all menus close (`packages/react/src/menubar/Menubar.test.tsx:187-189`, `packages/react/src/menubar/Menubar.test.tsx:1052-1054`).
- Trigger click toggles the attached menu: click opens, clicking the same trigger again closes (`packages/react/src/menubar/Menubar.test.tsx:137-153`).
- Clicking the trigger of a menu that was opened by hover closes it and clears `data-has-submenu-open`; a subsequent click re-opens and re-sets the attribute (regression #2222, `packages/react/src/menubar/Menubar.test.tsx:1021-1074`).
- Hover-driven transitions: hovering another top-level trigger while a submenu is open switches the open menu (previous closes, new opens, `packages/react/src/menubar/Menubar.test.tsx:176-214`); this also applies when a nested submenu is the currently open one (`packages/react/src/menubar/Menubar.test.tsx:245-284`).
- `disabled` on the menubar propagates down: triggers render the native `disabled` attribute and their menus cannot be opened by click (`packages/react/src/menubar/Menubar.test.tsx:1002-1018`); items inside an open menu render `aria-disabled="true"` and their `onClick` handlers are not invoked (`packages/react/src/menubar/Menubar.test.tsx:1189-1193`).

## Keyboard interactions

- `Tab` moves focus to the first trigger without opening its menu (`packages/react/src/menubar/Menubar.test.tsx:288-305`).
- `Enter` opens the focused trigger's menu (`packages/react/src/menubar/Menubar.test.tsx:301-304`, `packages/react/src/menubar/Menubar.test.tsx:443-448`).
- `Space` opens the focused trigger's menu (`packages/react/src/menubar/Menubar.test.tsx:417-433`).
- `ArrowRight`/`ArrowLeft` move focus between triggers when no menu is open (`packages/react/src/menubar/Menubar.test.tsx:371-394`; leftward wrap tested under `loopFocus`, `packages/react/src/menubar/Menubar.test.tsx:940-957`).
- `Home`/`End` move focus to the first/last trigger respectively (`packages/react/src/menubar/Menubar.test.tsx:396-415`).
- Opening a menu automatically focuses its first item (`packages/react/src/menubar/Menubar.test.tsx:450-454`, `packages/react/src/menubar/Menubar.test.tsx:484-486`).
- `ArrowDown` moves focus through items of the open menu, including submenu triggers (`packages/react/src/menubar/Menubar.test.tsx:456-469`).
- `ArrowRight` on a focused submenu trigger opens the submenu and focuses its first item (`packages/react/src/menubar/Menubar.test.tsx:498-511`).
- `ArrowLeft` inside an open submenu closes it and returns focus to the submenu trigger (`packages/react/src/menubar/Menubar.test.tsx:570-579`).
- `Escape` closes the open menu (browser-run only, `packages/react/src/menubar/Menubar.test.tsx:513-536`).
- `ArrowRight` while a submenu is open (no menu-level navigation ambiguity): closes the submenu and its root menu, opens the next menubar menu, and fires `onOpenChange(false)` on the submenu root, `onOpenChange(false)` on the closing root, and `onOpenChange(true)` on the next root (`packages/react/src/menubar/Menubar.test.tsx:582-670`).
- `ArrowRight`/`ArrowLeft` while a top-level menu is open navigates between menubar menus: the open menu closes and the adjacent one opens (`packages/react/src/menubar/Menubar.test.tsx:672-701`); this works identically after mouse-opened menus (`packages/react/src/menubar/Menubar.test.tsx:732-753`).
- `loopFocus=true`: `ArrowRight` past the last trigger wraps to the first, and `ArrowLeft` before the first wraps to the last (`packages/react/src/menubar/Menubar.test.tsx:920-957`).
- `loopFocus=false`: focus clamps — `ArrowRight` past the last trigger keeps focus on the last, `ArrowLeft` before the first keeps focus on the first (`packages/react/src/menubar/Menubar.test.tsx:960-998`).

## Focus management

- Initial `Tab` lands on the first trigger; the menu does not auto-open, even after a delay (`packages/react/src/menubar/Menubar.test.tsx:288-305`).
- Opening a menu moves focus into it (first item), whether opened by keyboard (`packages/react/src/menubar/Menubar.test.tsx:450-454`) or by mouse click (`packages/react/src/menubar/Menubar.test.tsx:717-722`).
- Closing a submenu via `ArrowLeft` restores focus to its trigger (`packages/react/src/menubar/Menubar.test.tsx:578-579`).
- A menubar whose first trigger is `disabled` stays keyboard-reachable: the disabled trigger gets `tabindex="-1"`, the next enabled trigger gets `tabindex="0"` and receives `Tab` focus (`packages/react/src/menubar/Menubar.test.tsx:1130-1168`).
- A fully `disabled` menubar is unreachable via `Tab` (focus lands on `document.body`) (`packages/react/src/menubar/Menubar.test.tsx:1010-1013`).
- Closing a triggerless menu (a `Menu.Root` with no `Menu.Trigger`, opened via controlled state) returns focus to the outside element that was clicked, rather than trapping or resetting it (`packages/react/src/menubar/Menubar.test.tsx:89-122`).

## Accessibility (roles, aria-*, id linking)

- Root element has `role="menubar"` (`packages/react/src/menubar/Menubar.test.tsx:1076-1080`).
- Root reflects `aria-orientation` from the `orientation` prop (e.g. `"vertical"`) (`packages/react/src/menubar/Menubar.test.tsx:1082-1085`).
- Menu triggers are exposed with `role="menuitem"` — a 3-trigger menubar yields exactly 3 `menuitem` roles (`packages/react/src/menubar/Menubar.test.tsx:1087-1091`).
- Contained (menubar-owned) top-level portals are linked for accessibility: the menubar renders a `span[aria-owns]` with `role="group"` inside itself that is not `aria-hidden` and whose `aria-owns` equals the portal element's `id`; the portal element carries `data-base-ui-portal` (`packages/react/src/menubar/Menubar.test.tsx:1095-1109`).
- Detached portals (rendered outside the menubar element) get an `aria-owns` span too, but outside the menubar, with no `role` attribute — adding `role="group"` there would create a stray accessible group (`packages/react/src/menubar/Menubar.test.tsx:1111-1127`).
- Items disable accessibly via `aria-disabled="true"` when the menubar is disabled (native `disabled` is not used on items) (`packages/react/src/menubar/Menubar.test.tsx:1189-1190`).
- Consuming `useMenubarContext` outside a `<Menubar>` throws: `Base UI: MenubarContext is missing. Menubar parts must be placed within <Menubar>.` (`packages/react/src/menubar/Menubar.test.tsx:31-46`).

## DOM structure & portal behavior

- The root renders as a `div` (`packages/react/src/menubar/Menubar.test.tsx:28`).
- Menu content renders into portals marked with `data-base-ui-portal` and an `id`, positioned by `Menu.Positioner` (`packages/react/src/menubar/Menubar.test.tsx:1100-1108`).
- For contained triggers, the menubar injects an ownership `span[aria-owns]` (role `group`) as a child of the menubar pointing at the portal id, so assistive tech can associate the portal with the menubar (`packages/react/src/menubar/Menubar.test.tsx:1103-1108`).
- For detached triggers (portal rendered as a sibling of the menubar), the ownership span lives outside the menubar and has no role (`packages/react/src/menubar/Menubar.test.tsx:1119-1126`).
- `keepMounted` portals remain in the DOM when closed; closed state is detectable via absence of `data-open` on the popup (`packages/react/src/menubar/Menubar.test.tsx:100-107`, `packages/react/src/menubar/Menubar.test.tsx:118-120`).
- Touch-opened menus participate in scroll locking: a `modal` menubar whose touch-opened menu spans (nearly) the full viewport width locks document scroll (`overflow: hidden` on `<html>` or `data-base-ui-scroll-locked` attribute, or `overflow: hidden` on body), while a narrower menu (240px) does not (`packages/react/src/menubar/Menubar.test.tsx:789-854`). Handing off touch-open between two top-level menus updates the lock to match the newly opened menu's geometry (lock applied for the wide menu, removed after switching to the narrow one) (`packages/react/src/menubar/Menubar.test.tsx:856-917`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- No menubar-specific custom events are asserted. All event behavior is expressed through props and native interactions:
  - `onOpenChange` (on `Menu.Root`) is invoked with `false` when a menu closes and `true` when it opens; during `ArrowRight` navigation with an open submenu, the closing submenu root and closing top-level root both receive `false` as their last call and the destination root receives `true` (`packages/react/src/menubar/Menubar.test.tsx:585-588`, `packages/react/src/menubar/Menubar.test.tsx:666-668`).
  - `onClick` on `Menu.Item` is suppressed while the menubar is `disabled` (`packages/react/src/menubar/Menubar.test.tsx:1171-1193`).
  - Touch `pointerdown` (with `pointerType: 'touch'`) opens menus, and a single outside touch press closes the entire open tree (top-level menu and nested submenu) (`packages/react/src/menubar/Menubar.test.tsx:756-787`).
  - Touch clicks are ignored for a wall-clock cooldown (~300ms, test advances 310ms with fake timers) immediately after a menu is opened via focus — a delayed touch click within the cooldown does not close the newly focused menu; after the cooldown expires, the touch click closes it (`packages/react/src/menubar/Menubar.test.tsx:48-87`).
  - Hover events open submenus only when another submenu is already open (hover-then-open chaining); hovering with no menu open does nothing (`packages/react/src/menubar/Menubar.test.tsx:162-174`, `packages/react/src/menubar/Menubar.test.tsx:176-214`).
- No test asserts `preventDefault`/`stopPropagation` semantics for menubar events. UNVERIFIED — no coverage found in `packages/react/src/menubar/Menubar.test.tsx`.

## Edge cases (rapid interactions, unmount, nesting)

- Nested submenus (2 levels: menubar → menu → submenu) are exercised throughout; hover opens nested submenus while the parent menu is open (`packages/react/src/menubar/Menubar.test.tsx:216-243`), and hovering from an open nested submenu to another top-level trigger closes the whole previous chain (file menu and share submenu both close) (`packages/react/src/menubar/Menubar.test.tsx:245-284`).
- `closeOnClick`/non-closing nested items: clicking a radio item (`Menu.RadioItem` inside `Menu.RadioGroup`) in a nested submenu does not close the menu, both when the parent menu was opened by click (`packages/react/src/menubar/Menubar.test.tsx:314-337`) and when it was opened by hover (regression #2092, `packages/react/src/menubar/Menubar.test.tsx:339-367`).
- Delayed touch click after a focus-open is ignored during the cooldown window and honored after it (see Events) (`packages/react/src/menubar/Menubar.test.tsx:48-87`).
- Clicking the trigger of a hover-opened menu closes it instead of re-opening/toggling unexpectedly, and the menubar remains fully usable afterward (open → hover-open → click-close → click-open → hover-open cycle) (`packages/react/src/menubar/Menubar.test.tsx:1021-1074`).
- Outside interactions: a single outside touch press closes the entire open tree (`packages/react/src/menubar/Menubar.test.tsx:757-786`); a click on an outside element closes a triggerless menu and keeps focus on that element (`packages/react/src/menubar/Menubar.test.tsx:89-122`).
- Mixed input sequences: mouse-open followed by keyboard navigation works (ArrowDown focuses first item; ArrowRight switches menus) (`packages/react/src/menubar/Menubar.test.tsx:704-753`).
- Environment partitioning: click-toggle, hover, Escape, #2222, and scroll-lock suites are browser-only (`describe.skipIf(isJSDOM)`, `packages/react/src/menubar/Menubar.test.tsx:131`, `packages/react/src/menubar/Menubar.test.tsx:156`, `packages/react/src/menubar/Menubar.test.tsx:513`, `packages/react/src/menubar/Menubar.test.tsx:1021`, `packages/react/src/menubar/Menubar.test.tsx:789`); the ArrowRight-closes-submenus spy test, left/right menu navigation, mixed mouse/keyboard, touch tree-close, and loopFocus suites are jsdom-only (`describe.skipIf(!isJSDOM)`, `packages/react/src/menubar/Menubar.test.tsx:582`, `packages/react/src/menubar/Menubar.test.tsx:673`, `packages/react/src/menubar/Menubar.test.tsx:704`, `packages/react/src/menubar/Menubar.test.tsx:756`, `packages/react/src/menubar/Menubar.test.tsx:920`).
- Tests disable Base UI animations globally via `globalThis.BASE_UI_ANIMATIONS_DISABLED = true` in `beforeEach` (`packages/react/src/menubar/Menubar.test.tsx:17-20`).

## Shared harness dependencies

- `#test-utils` resolves to `packages/react/test/index.ts` (`packages/react/package.json:110`) and re-exports: `@base-ui/utils/testUtils` (providing `isJSDOM`, detected from the user agent, `packages/utils/src/testUtils.ts:4`), `createRenderer` (`packages/react/test/createRenderer.ts:27-49` — wraps the shared renderer's `render` in `act` and adds `rerender`/`setProps`), `describeConformance` (`packages/react/test/describeConformance.tsx:51-70` — runs the prop-forwarding/ref/render-prop/className suite), `resetBrowserPointer` (`packages/react/test/resetBrowserPointer.ts:6-10` — unhovers `document.body` via `vitest/browser` in real browsers), and `wait` (`packages/react/test/wait.ts:4-8`).
- `@mui/internal-test-utils` (external workspace package, not read) provides `act`, `fireEvent`, `screen`, `waitFor` used directly by the test file (`packages/react/src/menubar/Menubar.test.tsx:3`).
- Browser-mode tests additionally import `vitest/browser` (`userEvent`) and `vitest-browser-react` (`render`, `cleanup`) at runtime (`packages/react/src/menubar/Menubar.test.tsx:133-139`, `packages/react/src/menubar/Menubar.test.tsx:158-164`).
- `useRefWithInit` from `@base-ui/utils/useRefWithInit` (production util, `packages/utils/src/useRefWithInit.ts:11-15`) is used in the detached fixture to create the stable `Menu.Handle` (`packages/react/src/menubar/Menubar.test.tsx:13`, `packages/react/src/menubar/Menubar.test.tsx:1451`).
- `useMenubarContext` from the unit's own `MenubarContext.ts` is asserted to throw when used outside the provider (`packages/react/src/menubar/Menubar.test.tsx:14`, `packages/react/src/menubar/Menubar.test.tsx:31-46`).
