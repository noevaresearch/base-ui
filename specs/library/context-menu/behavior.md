# Context Menu behavior spec

Mined from the three context-menu test files listed below. The `TODO.md` entry
(`TODO.md:361-367`) has no `wraps-external:` field, so all behavior below is derived from the
component's own tests; there is no third-party package to delegate to. The unit is not on the
`needs-batched-mining: true` list, so this single file covers the whole unit.

Files mined:
- `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx`
- `packages/react/src/context-menu/root/ContextMenuRoot.non-mac.test.tsx`
- `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx`

Platform note: `ContextMenuRoot.test.tsx` mocks `@base-ui/utils/platform` to report a Mac
(`os.mac: true, os.apple: true`) (`packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:14-25`),
while `ContextMenuRoot.non-mac.test.tsx` mocks it to report non-Apple platforms
(`packages/react/src/context-menu/root/ContextMenuRoot.non-mac.test.tsx:12-23`). Where the two
files disagree about gesture-end handling, the difference is deliberate platform behavior.

## Public API surface (props, parts, subcomponents)

- Parts exercised by the tests, all imported from the `@base-ui/react/context-menu` namespace:
  `ContextMenu.Root`, `ContextMenu.Trigger`, `ContextMenu.Portal`, `ContextMenu.Positioner`,
  `ContextMenu.Popup`, `ContextMenu.Backdrop`, `ContextMenu.Item`, `ContextMenu.SubmenuRoot`,
  `ContextMenu.SubmenuTrigger`. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:10`, `packages/react/src/context-menu/root/ContextMenuRoot.non-mac.test.tsx:9`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:4`
- `ContextMenu.Root`:
  - `open` (controlled open state; renders open with no interaction). `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:296`
  - `defaultOpen` (uncontrolled initial state). `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:60`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:185`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:298`
  - `onOpenChange(nextOpen, eventDetails)` — details carry a `reason`. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:95-98`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:95`
  - `disabled` — hard gate on every open path. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:264`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:248`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:564`
- `ContextMenu.Trigger`:
  - Renders a `div` (conformance resolves its ref to `window.HTMLDivElement`) and must be
    rendered inside `ContextMenu.Root` (the conformance wrapper mounts it within a Root).
    `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:20-25`
  - `onContextMenu` — receives a wrapped event exposing `preventBaseUIHandler()`.
    `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:354-357`
  - `data-testid` and other DOM props pass through to the rendered element.
    `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:42`
- `ContextMenu.Portal`:
  - `container` (ref to an arbitrary element to portal into). `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:337`
- `ContextMenu.Positioner`:
  - `anchor` accepting a virtual element (`{ getBoundingClientRect }`).
    `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:303-311`
  - `collisionAvoidance` (e.g. `{ side: 'flip' }`). `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:299-301`
  - `alignOffset` (positive and negative numbers). `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:162`, `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:198`, `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:234`, `packages/react/src/context-menu/root/ContextMenuRoot.non-mac.test.tsx:50`
- `ContextMenu.Popup` / `ContextMenu.Item` / `ContextMenu.Backdrop` — no props asserted beyond
  DOM passthrough (`data-testid`, `style`). `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:313`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:301`
- `ContextMenu.SubmenuRoot`:
  - `defaultOpen`. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:51`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:185`
  - `onOpenChange`. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:51`, `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:114`
- `ContextMenu.SubmenuTrigger`:
  - `delay`. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:52`
  - `openOnHover`. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:115-120`
- No props beyond those listed above are asserted by these tests.

## State model (controlled/uncontrolled, defaults, transitions)

- The root owns a single open/closed state; the default is closed — without `defaultOpen`/`open`
  the popup is absent until its trigger is right-clicked (the outer menu in the nesting test
  stays `null` until right-clicked). `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:622-623`, `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:281`
- Uncontrolled: `defaultOpen` starts the menu open (trigger carries `data-popup-open`).
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:60-71`
- Controlled: `open` forces the state (menu rendered open with no interaction).
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:294-296`
- Transitions asserted:
  - `contextmenu` (right-click) on the trigger → open, `onOpenChange(true)`.
    `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:77-96`
  - Escape key → close (`data-popup-open` removed). `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:73-74`
  - Secondary-button `mouseup` over an item after the pointer left the spawn point → close with
    `reason: REASONS.itemPress`. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:223-258`, `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:95-98`
  - `mouseup` on `document.body` more than 500ms after open → close (`onOpenChange(false)`).
    `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:128-152`
  - `pointerdown` on `document.body` → closes a nested inner menu.
    `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:625-628`
- `disabled` on the root blocks all transitions into open: right-click and long-press produce no
  popup and zero `onOpenChange` calls. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:260-283`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:244-264`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:560-591`
- No other state (selection/checked/highlight) is asserted by these tests.

## Keyboard interactions

- Escape closes an open context menu (the trigger's `data-popup-open` attribute clears).
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:73-74`
- No other keyboard interaction (arrows, Enter/Space, typeahead, Tab) is asserted in these three
  files. UNVERIFIED — no test covers them here.

## Focus management

- N/A — none of the three test files asserts focus movement (initial focus, focus containment,
  or focus return on close). UNVERIFIED — inferred from the absence of focus assertions in
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:1-636`,
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:1-333`,
  `packages/react/src/context-menu/root/ContextMenuRoot.non-mac.test.tsx:1-76`, no test asserts focus behavior.

## Accessibility (roles, aria-*, id linking)

- The open popup is exposed with role `menu` (`queryByRole('menu')` finds it after both
  right-click and long-press opens). `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:55`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:397`
- While open, an internal outside-press blocker element exists as a direct child of a
  `[data-base-ui-portal]` element with `data-base-ui-inert` and `role="presentation"`.
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:311-313`
- Open state is mirrored on the trigger as `data-popup-open=""`, removed when closed.
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:70-74`
- `aria-haspopup`, `aria-expanded`, `aria-controls`, or trigger↔popup id linking are not asserted
  in these tests. UNVERIFIED — no test covers them.

## DOM structure & portal behavior

- Canonical composition: `ContextMenu.Root > (ContextMenu.Trigger + ContextMenu.Portal >
  ContextMenu.Positioner > ContextMenu.Popup > items)`. All three files build this shape.
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:46-56`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:40-48`
- Portal content renders outside the trigger tree, and the `container` prop can target any
  element — including one mounted inside the trigger's own DOM subtree.
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:328-349`
- Closed menus are fully absent from the DOM (popup `queryByTestId` is `null` when closed or
  disabled). `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:281`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:262`
- The positioner resolves and exposes the final placed side as `data-side` (e.g. `'top'` after a
  flip when there is no space below the anchor). `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:324-330`
- While open, a user-rendered `ContextMenu.Backdrop` inside the portal is a normal sibling of the
  positioner and is distinct from the internal inert blocker element.
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:296-326`
- Submenus use their own `Portal > Positioner > Popup` chain nested inside the parent popup.
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:51-64`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:185-191`
- Entire context menus can be nested as separate roots (a `ContextMenu.Root` inside another
  root's trigger children). `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:594-613`

## Events (names, payload shape, bubbling, preventDefault semantics)

- `contextmenu` on the trigger opens the menu; the right-click coordinates are remembered as the
  cursor/spawn point — a later `mouseup` at those same coordinates is treated differently from
  one at other coordinates. `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:39-56`, `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:173-178`, `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:245-251`
- `onOpenChange(open, details)`: `details.reason` is `REASONS.itemPress` when a gesture-end
  mouseup over an item closes the menu (asserted for both a submenu and its root);
  `REASONS` is imported from `../../internals/reasons`. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:95-98`, `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:12`
- Secondary-button `mouseup` after open (the "context-menu gesture end"):
  - Over an item after the pointer moved away from the spawn point → closes the menu (and the
    whole nested tree), `onOpenChange(false)`. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:223-258`, `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:41-99`
  - Directly under the spawn cursor point → ignored regardless of `alignOffset` sign; the menu
    stays open and no extra `onOpenChange` fires. `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:151-185`, `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:187-221`
  - On non-Mac platforms → ignored entirely, even after the pointer moves (menu stays open,
    `onOpenChange` called only for the open). `packages/react/src/context-menu/root/ContextMenuRoot.non-mac.test.tsx:39-74`
  - Over a submenu trigger → does not activate/open the submenu.
    `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:101-149`
  - Inside the positioner, or inside a portaled submenu popup → keeps the menu(s) open, even
    after the 500ms grace window (`tick(501)` precedes the mouseup).
    `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:154-174`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:176-205`
- Document `mouseup` is gated by a ~500ms grace window after open: within 500ms it is ignored
  (menu stays open, no extra callback); after 500ms it dismisses (`onOpenChange(false)`).
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:98-126`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:128-152`
- Touch long-press opens after a 500ms hold; `touchmove` beyond ~10px cancels the pending press
  (movement within the threshold keeps it); `touchend` cancels; the gesture becoming multi-touch
  cancels — and starting a gesture with two touches never opens.
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:369-532`
- After opening via long press, outside-press dismissal is delayed ~500ms: an immediate
  `mousedown` on `document.body` is ignored, one after 500ms closes.
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:534-558`
- Native `contextmenu` default-action prevention:
  - When the root is disabled, the native context menu is NOT default-prevented.
    `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:266-293`
  - While open, `contextmenu` events are default-prevented on the internal inert blocker and on
    a user-rendered `ContextMenu.Backdrop`, but not on plain `document.body` outside.
    `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:296-326`
  - Prevention reaches popups portaled inside the trigger's DOM subtree (a bubbling `contextmenu`
    dispatched on the popup ends up default-prevented). `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:328-349`
  - A consumer `onContextMenu` calling `event.preventBaseUIHandler()` skips Base UI's own open
    handling, yet the native context menu is still default-prevented.
    `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:351-367`
- Whether the native context menu is prevented on the plain enabled-open path (no consumer
  `onContextMenu`) is not directly asserted. UNVERIFIED — inferred from
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:351-367`, no test asserts
  `defaultPrevented` for that path.

## Edge cases (rapid interactions, unmount, nesting)

- Same-point `mouseup` immediately after open (the common "menu appears under the held button"
  case) never closes the menu, for both `alignOffset={0}` and `alignOffset={-5}`.
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:151-221`
- Trigger unmount mid-gesture: unmounting the trigger after a right-click aborts the pending
  document `mouseup` listener — the later mouseup neither closes the menu nor fires an extra
  `onOpenChange`. `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:207-241`
- Nested roots: right-clicking an inner trigger opens only the inner menu (outer stays closed);
  a body `pointerdown` closes the inner menu; right-clicking the outer trigger then opens only
  the outer menu (inner stays closed). `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:594-635`
- Nested submenu gesture-end: `mouseup` over a deep submenu item closes the entire tree, and both
  the submenu's and the root's `onOpenChange` report `(false, { reason: REASONS.itemPress })`.
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:41-99`
- Multi-touch recovery: after a canceled long press, ending the gesture and starting a fresh
  two-touch gesture still never opens. `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:525-531`
- Long-press grace: a `mousedown` on the body right after a long-press open is ignored
  (dismissal delayed ~500ms). `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:534-558`
- Disabled short-circuits every open path (right-click and long press) with zero callbacks.
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:260-283`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:560-591`

## Shared harness dependencies

- `#test-utils` (alias defined at `packages/react/package.json:110` → `packages/react/test/index.ts`)
  provides `createRenderer`, `describeConformance`, and `isJSDOM`.
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:11`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:5`
- `createRenderer` (`packages/react/test/createRenderer.ts:27-48`) wraps `@mui/internal-test-utils`'s
  renderer in an awaited `act` and exposes awaited `render`/`rerender`/`setProps` plus the
  user-event `user` instance; all three files create it with
  `clockOptions: { shouldAdvanceTime: true }` and enable fake timers via `clock.withFakeTimers()`.
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:32-39`, `packages/react/src/context-menu/root/ContextMenuRoot.non-mac.test.tsx:30-37`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:12-18`
- `describeConformance` (`packages/react/test/describeConformance.tsx:44-49`) runs the shared
  prop-forwarding / ref-forwarding / render-prop / className suites; the trigger test registers
  with `refInstanceof: window.HTMLDivElement` inside a `ContextMenu.Root` wrapper.
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:20-25`
- `isJSDOM` gates the Chromium-only suites (layout-based `collisionAvoidance` and all
  touch/long-press tests). `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:286`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:369`
- `globalThis.BASE_UI_ANIMATIONS_DISABLED = true` is set in every file's `beforeEach` (a global
  animation-disabling flag). `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:28-30`, `packages/react/src/context-menu/root/ContextMenuRoot.non-mac.test.tsx:26-28`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:8-10`
- Platform-dependent behavior is controlled by mocking `@base-ui/utils/platform`
  (`os.mac` / `os.apple`). `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:14-25`, `packages/react/src/context-menu/root/ContextMenuRoot.non-mac.test.tsx:12-23`
- `fireEvent`, `flushMicrotasks`, `ignoreActWarnings`, `reactMajor`, `screen`, `waitFor`, and
  `act` come from the external `@mui/internal-test-utils` package.
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:2-9`, `packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx:3`
- `REASONS` is imported from the source tree (`../../internals/reasons`), not a harness; the tests
  use it only to assert `reason` values in `onOpenChange` details.
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:12`
