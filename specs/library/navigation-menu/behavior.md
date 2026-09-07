# Navigation Menu behavior spec

Mined from the fifteen navigation-menu test files listed below. The `TODO.md` entry
(`TODO.md:433-439`) has no `wraps-external:` field, so the behavior below is derived entirely from
the component's own tests; there is no third-party package to delegate to. The unit is not on the
`needs-batched-mining: true` list (only combobox, drawer, floating-ui-react, menu, number-field,
select are — `TODO.md:447`), and the suite is ~5.6k lines across 15 files, so this single file
covers the whole navigation-menu unit.

Files mined:
- `packages/react/src/navigation-menu/arrow/NavigationMenuArrow.test.tsx`
- `packages/react/src/navigation-menu/backdrop/NavigationMenuBackdrop.test.tsx`
- `packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx`
- `packages/react/src/navigation-menu/icon/NavigationMenuIcon.test.tsx`
- `packages/react/src/navigation-menu/item/NavigationMenuItem.test.tsx`
- `packages/react/src/navigation-menu/link/NavigationMenuLink.test.tsx`
- `packages/react/src/navigation-menu/list/NavigationMenuList.test.tsx`
- `packages/react/src/navigation-menu/popup/NavigationMenuPopup.test.tsx`
- `packages/react/src/navigation-menu/portal/NavigationMenuPortal.test.tsx`
- `packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.test.tsx`
- `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx`
- `packages/react/src/navigation-menu/root/NavigationMenuRoot.webkit.test.tsx`
- `packages/react/src/navigation-menu/root/NavigationMenuRootContext.test.ts`
- `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx`
- `packages/react/src/navigation-menu/viewport/NavigationMenuViewport.test.tsx`

The root tests run under fake timers (`clock.withFakeTimers()`,
`packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1019-1025`), and hover timing
constants used by the tests are `OPEN_DELAY = 50` and `CLOSE_DELAY = 50`
(`packages/react/src/navigation-menu/utils/constants.ts:1-2`, imported at
`packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:10`,
`packages/react/src/navigation-menu/root/NavigationMenuRoot.webkit.test.tsx:6`) and
`PATIENT_CLICK_THRESHOLD = 500`
(`packages/react/src/internals/constants.ts:4`, imported at
`packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:9`).

## Public API surface (props, parts, subcomponents)

- All parts are imported from the `@base-ui/react/navigation-menu` namespace as `NavigationMenu.X`:
  `Root`, `List`, `Item`, `Trigger`, `Content`, `Link`, `Popup`, `Positioner`, `Viewport`,
  `Portal`, `Arrow`, `Icon`, `Backdrop`.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:4`,
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:3`,
  `packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx:2`
- Rendered element types (conformance `refInstanceof`): Root → `HTMLElement`
  (`packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1027-1032`); List →
  `HTMLUListElement` (`packages/react/src/navigation-menu/list/NavigationMenuList.test.tsx:10-15`);
  Item → `HTMLLIElement`
  (`packages/react/src/navigation-menu/item/NavigationMenuItem.test.tsx:8-13`); Trigger →
  `HTMLButtonElement` rendered as a native button (`testComponentPropWith: 'button'`,
  `button: true`)
  (`packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:123-136`); Content →
  `HTMLDivElement` (`packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx:9-25`,
  conformance suite skipped there); Link → `HTMLAnchorElement`
  (`packages/react/src/navigation-menu/link/NavigationMenuLink.test.tsx:9-18`); Popup → `HTMLElement`
  (`packages/react/src/navigation-menu/popup/NavigationMenuPopup.test.tsx:8-19`); Positioner and
  Portal → `HTMLDivElement`
  (`packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.test.tsx:30-39`,
  `packages/react/src/navigation-menu/portal/NavigationMenuPortal.test.tsx:9-14`); Viewport,
  Backdrop, Arrow → `HTMLDivElement`
  (`packages/react/src/navigation-menu/viewport/NavigationMenuViewport.test.tsx:8-13`,
  `packages/react/src/navigation-menu/backdrop/NavigationMenuBackdrop.test.tsx:8-13`,
  `packages/react/src/navigation-menu/arrow/NavigationMenuArrow.test.tsx:8-19`); Icon →
  `HTMLSpanElement` (`packages/react/src/navigation-menu/icon/NavigationMenuIcon.test.tsx:8-17`).
- `NavigationMenu.Root` props exercised by tests:
  - `value` (controlled open-item value; `null` closes).
    `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1752-1773`,
    `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1815-1827`
  - `defaultValue` (initially open item).
    `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1602-1612`
  - `onValueChange(value, eventDetails)` — called with the next item value.
    `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1656-1672`
  - `orientation` (`'horizontal'` | `'vertical'`) affects activation-direction semantics and
    keyboard behavior (no `aria-orientation` attribute is rendered — see Accessibility).
    `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1264-1301`
  - `delay` (hover open delay in ms).
    `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1903-1923`
  - `closeDelay` (hover close delay in ms).
    `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1925-1950`
  - `actionsRef` exposing `unmount()` (`NavigationMenu.Root.Actions`).
    `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2015-2040`
  - `onOpenChangeComplete` (called with `false` when a close finishes).
    `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3856-3915`
- `NavigationMenu.Trigger` props: `disabled` (native button disable; renders `data-disabled`).
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:138-165`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1952-2013`
- `NavigationMenu.Content` props: `keepMounted`.
  `packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx:30-53`,
  `packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx:81-105`
- `NavigationMenu.Link` props: `closeOnClick` (default false per the `false` test rendering without
  close behavior; `true` closes the menu on click), `active`.
  `packages/react/src/navigation-menu/link/NavigationMenuLink.test.tsx:20-59`,
  `packages/react/src/navigation-menu/link/NavigationMenuLink.test.tsx:61-100`,
  `packages/react/src/navigation-menu/link/NavigationMenuLink.test.tsx:102-131`
- `NavigationMenu.Positioner` props: `side` (`'left'`, `'inline-start'`, `'inline-end'` etc.).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2081-2120`
- `NavigationMenu.List` supports a `render` prop with a custom element.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1175-1196`
- Composition rules enforced with descriptive errors (all tested via `rejects.toThrow`):
  - Any part outside `Root`:
    `'Base UI: NavigationMenuRootContext is missing. Navigation Menu parts must be placed within <NavigationMenu.Root>.'`
    `packages/react/src/navigation-menu/list/NavigationMenuList.test.tsx:17-27`
  - `Icon` outside `Item`:
    `'Base UI: NavigationMenuItem parts must be used within a <NavigationMenu.Item>.'`
    `packages/react/src/navigation-menu/icon/NavigationMenuIcon.test.tsx:19-35`
  - `Positioner` outside `Portal`: `'Base UI: <NavigationMenu.Portal> is missing.'`
    `packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.test.tsx:55-69`
  - `Arrow` outside `Positioner`:
    `'Base UI: NavigationMenuPositionerContext is missing. NavigationMenuPositioner parts must be placed within <NavigationMenu.Positioner>.'`
    `packages/react/src/navigation-menu/arrow/NavigationMenuArrow.test.tsx:21-37`
- Every part passes the shared conformance suite (prop spreading, ref forwarding, render prop,
  className handling) except Content, whose conformance run is skipped.
  `packages/react/test/describeConformance.tsx:44-49`,
  `packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx:9`
- `NavigationMenuRootContext` exposes `displayName === 'NavigationMenuRootContext'` in development
  and `undefined` in production.
  `packages/react/src/navigation-menu/root/NavigationMenuRootContext.test.ts:10-16`,
  `packages/react/src/navigation-menu/root/NavigationMenuRootContext.test.ts:18-24`

## State model (controlled/uncontrolled, defaults, transitions)

- The root's state is the currently open item's value (`string`), or closed. `defaultValue` opens
  that item on mount; with no props the menu starts closed (popup absent before any interaction).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1602-1612`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1915`
- Uncontrolled mode: hover (after `OPEN_DELAY`), click, touch click, and keyboard open set the
  value; `onValueChange` receives the new value.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1049-1060`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1239-1248`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1303-1314`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1656-1672`
- Falsy item values (`0`, `''`, `false`) are treated as valid open values, not as "closed".
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1716-1750`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3000-3025`
- Controlled mode (`value`): the UI follows the prop; an external change from `'item-1'` to
  `'item-2'` switches which trigger is `aria-expanded`. External close (`value={null}`) runs the
  exit animation while preserving the popup's measured size.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1752-1773`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1819-1838`
- Hover behavior: mouse enter + move opens after the open delay; leaving closes after the close
  delay; both delays are configurable via `delay`/`closeDelay`. Hover does not open for touch
  pointers.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1049-1060`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1903-1923`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1925-1950`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1250-1262`
- Clicking a different trigger switches the open item rather than closing (mouse and touch).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1440-1458`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1460-1480`
- Clicking an already-open inline nested trigger again does not close it (stays open).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2953-2959`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3019-3023`
- "Patient click": clicking the trigger after `PATIENT_CLICK_THRESHOLD` (500ms) following a
  hover-open closes the menu.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1581-1599`
- Cancellation: `eventDetails.cancel()` inside `onValueChange` blocks the open (value unchanged,
  popup not mounted, `aria-expanded` stays `false`).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1674-1688`
- `disabled` on the trigger blocks open via hover, click, touch, and keyboard, and `onValueChange`
  is not called.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1953-1966`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1968-1979`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1981-1994`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1996-2012`
- Close transitions carry `data-starting-style` / `data-ending-style` phases on popup and content;
  during an externally-triggered close the popup keeps `data-ending-style` plus its fixed
  `--popup-width`/`--popup-height` and the positioner keeps `--positioner-width`/`--positioner-height`.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1826-1833`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1888-1891`
- `data-activation-direction` (`left`/`right` for horizontal, `up`/`down` for vertical) is set on
  the popup when switching items, based on the relative geometry of the triggers, and is removed
  when the menu closes during an exit.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1264-1301`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1884-1891`
- `actionsRef.current.unmount()` opts out of the automatic unmount-on-close: closing leaves the
  popup mounted until `unmount()` is called; calling `unmount()` while an exit animation runs
  removes it immediately.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2015-2040`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2042-2078`
- After a trigger switch while an exit animation runs, an animation of a previous stage finishing
  does not reset the size to the older panel's dimensions ("initial open size reset" ignored once a
  switch started).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3642-3722`

## Keyboard interactions

- `ArrowRight`/`ArrowLeft` (horizontal default) move focus between top-level triggers; in a
  `vertical` root under RTL the mirrored `ArrowLeft` opens the focused trigger's menu.
  `packages/react/src/navigation-menu/list/NavigationMenuList.test.tsx:85-100`,
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:167-198`
- `ArrowDown` (and `Enter`, `Space`) open the focused trigger's menu; focus stays on the trigger
  (Chromium-only assertions).
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:200-236`
- Vertical arrow keys are intercepted inside the list and do not escape to outer handlers, while
  unrelated keys (`PageDown`) propagate.
  `packages/react/src/navigation-menu/list/NavigationMenuList.test.tsx:29-54`
- Moving focus to another trigger via `ArrowRight` does NOT emit `onValueChange` for that item
  (no open); pressing `ArrowDown` afterwards opens it and emits exactly one `onValueChange`.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1690-1714`
- `Escape` closes the open menu and returns focus to the trigger.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1482-1504`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3900-3911`
- Inside an open popup (viewport style), `ArrowDown`/`ArrowUp` cycle through the panel's links and
  the nested submenu triggers; this works across 3 levels of nested Roots.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3051-3083`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3085-3133`
- `Tab`/`Shift+Tab` move focus in DOM order through the open content and out (see Focus
  management); tabbing out closes the menu.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2122-2165`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2167-2190`

## Focus management

- Opening by click/keyboard leaves focus on the trigger; `Tab` then moves into the popup content
  (Link 1 → Link 2 → next trigger), and `Shift+Tab` returns through the same chain.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2122-2165`
- Hidden focus guards (`[data-base-ui-focus-guard]`) wrap the trigger; when no viewport focus guard
  is rendered, focusing the guard after the trigger returns focus to the trigger itself.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2230-2258`
- Tabbing forward past the content and out (including through arbitrary tabbable content such as a
  plain button inside the panel) closes the menu; tabbing backward out does the same.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2167-2190`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2192-2228`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2290-2313`
- `Shift+Tab` from the element after the open item returns focus to the last content link, not
  skipping over the panel; this holds for inline nested submenus too.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2260-2288`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:4004-4025`
- Tabbing from the last link of the last open nested panel moves focus to the next top-level
  trigger (keeping the nested panel open); tabbing between a nested trigger and its links does not
  open inactive panels.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3955-3971`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3973-4002`
- Hover-closed menus do not restore focus to the trigger; neither do closes triggered by focus
  moving outside the menu.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1531-1551`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1553-1578`
- Clicking outside the menu closes it and focus lands on the clicked outside element.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1506-1529`
- A link inside the content that blurs with `relatedTarget: null` does not close the menu.
  `packages/react/src/navigation-menu/link/NavigationMenuLink.test.tsx:134-165`

## Accessibility (roles, aria-*, id linking)

- Triggers expose `aria-expanded="true"/"false"` matching the open item.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1059`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1247`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1598`
- `aria-orientation` is NOT applied to the root element or to the list, for top-level and nested
  roots alike, even when `orientation="vertical"` is set.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1034-1046`
- `NavigationMenu.Link` with `active` renders `aria-current="page"`; without it, no
  `aria-current` attribute.
  `packages/react/src/navigation-menu/link/NavigationMenuLink.test.tsx:102-116`,
  `packages/react/src/navigation-menu/link/NavigationMenuLink.test.tsx:118-131`
- The trigger renders a native `button` (focusable by `getByRole('button')`), and disabled triggers
  carry `data-disabled`.
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:123-136`,
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:161-164`
- `NavigationMenu.Icon` inside the active item's trigger gets `data-popup-open` when that item is
  open (and not on inactive items' icons).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2962-2998`
- id-linking attributes such as `aria-controls`/`aria-haspopup` between trigger and content are not
  asserted anywhere in these tests — UNVERIFIED — inferred from
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:1-665` (no test
  asserts this).
- Links keep their native `link` role and accessible name (`href` present, queried via
  `getByRole('link', { name })`).
  `packages/react/src/navigation-menu/link/NavigationMenuLink.test.tsx:54-55`,
  `packages/react/src/navigation-menu/link/NavigationMenuLink.test.tsx:115`

## DOM structure & portal behavior

- Typical structure: `Root` → `List` (ul) → `Item` (li) → `Trigger` + `Content`; plus
  `Portal` → `Positioner` → `Popup` → `Viewport`. All parts render their conformance element type
  (see Public API surface).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:28-53`
- `Content` renders inside the `Viewport` of the popup (moved out of the list's DOM subtree when
  its item activates) and remains mounted in the viewport while switching between items when
  `keepMounted`.
  `packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx:159-183`
- SSR: `Content` with `keepMounted` is present (hidden) in server-rendered HTML; without it, it is
  absent; the same holds after hydration.
  `packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx:30-53`,
  `packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx:55-78`,
  `packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx:81-105`,
  `packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx:107-130`
- With a `keepMounted` Portal, the content stays mounted inside the popup but gains `hidden` when
  the menu closes.
  `packages/react/src/navigation-menu/content/NavigationMenuContent.test.tsx:206-219`
- Opening with a `keepMounted` portal suppresses `transition` on the popup and arrow for the first
  frame (`transition: none`), released on a later tick.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1614-1653`
- Initially-open menus (`defaultValue`) mark the positioner `data-instant` for the first frame
  (transition suppressed), released on the next tick; the positioner also gets `data-instant`
  during resize-driven re-measuring for ~100ms.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:4038-4050`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3565-3574`
- Sizing is driven through CSS variables: `--popup-width`/`--popup-height` on the popup (fixed
  during size transitions, `auto` when settled) and `--positioner-width`/`--positioner-height` on
  the positioner (values track the active panel's measured box).
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:276-283`,
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:393-401`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3184-3189`
- Positioner placement uses `shift` with `rootBoundary: 'layoutViewport'` (spy on the anchor
  positioning hook).
  `packages/react/src/navigation-menu/positioner/NavigationMenuPositioner.test.tsx:41-53`
- `Positioner side='left'|'inline-start'` (LTR) or `side='inline-end'` (RTL) pins the popup to the
  right edge so it grows leftward: popup has `data-side="<side>"` and computed
  `position: absolute; top: 0px; right: 0px`.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2082-2119`
- Popup sizing reacts to: content insertion while open, switching to a keepMounted trigger's panel
  (immediate), `hidden` attribute changes on kept content (MutationObserver path), and window
  `resize` events.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3167-3189`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3618-3635`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3401-3411`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3565-3581`
- Inline nested roots (a `Root` inside a parent's `Content` with its own `List` + `Viewport`, no
  Portal) work without a positioner/popup.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2423-2466`
- When reopening after the pointer hovered a top-level link, the popup width is seeded from the
  exiting panel's size before the new panel's size is applied.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3818-3829`

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onValueChange(value, eventDetails)` fires on open-item change with the next value (a falsy value
  is delivered as-is, e.g. `0`, `''`, `false`); `eventDetails.cancel()` prevents the change.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1656-1672`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1745-1746`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1674-1688`
- A nested menu's close (via a `closeOnClick` link) propagates to the parent root's
  `onValueChange` with `null`, and closes every ancestor level for deeply nested roots.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2568-2595`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2597-2619`
- `onOpenChangeComplete(false)` fires after a close completes; after switching panels the exit uses
  the short path (complete observed in < 325ms, Chromium).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3903-3911`
- Pointer semantics: a `pointerdown` on the open trigger clears the safePolygon lock; a
  `pointerdown` on a link inside a hover-open popup followed by the pointer leaving closes the
  menu.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1170-1172`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1432-1437`
- `stopPropagation` inside nested Base UI popups (Dialog/Popover) is honored: interacting with a
  nested Dialog or Popover does not close the navigation menu (even hover-open ones).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2316-2337`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2339-2361`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2363-2385`
- No custom DOM CustomEvents are asserted in these tests; all callbacks are React props — UNVERIFIED
  for any DOM-event payload beyond the above — inferred from
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1656-1950`.

## Edge cases (rapid interactions, unmount, nesting)

- Rapid hover switching: a previously hovered trigger cannot reapply stale popup widths after a
  later switch (popup `--popup-width` never returns to the old value, settles to `auto`); the same
  protection holds for interrupted mutation-driven resizing of the positioner.
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:473-495`,
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:543-574`
- Hover-open then quick click, then moving to another trigger: hover-open behavior is restored
  (menu reopens on hovering the original trigger).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1385-1416`
- After a touch-click close via outside click, hover-open works again (also for nested submenu
  triggers touched).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1316-1344`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1346-1383`
- Unmount while open: if the open trigger unmounts (e.g. becomes an inline active link), the list's
  pointer-events lock is released.
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:580-617`
- Pointer sweeping across a trigger without waiting for the open delay releases the eager
  pointer-events lock.
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:619-664`
- Item removal: after an earlier item unmounts, arrow-key navigation continues from the focused
  trigger using the updated item list.
  `packages/react/src/navigation-menu/list/NavigationMenuList.test.tsx:77-101`
- Measurement races: a temporarily zero `offsetWidth`/`offsetHeight` during an interrupted mutation
  does not collapse the popup to zero (size preserved).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3489-3534`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3917-3953`
- Mutation-driven resizing is interruptible: a second content update mid-transition supersedes the
  first size (older values applied then overwritten; never reapplied after a later switch).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3418-3487`
- Nesting: nested Roots open on hover, keep the parent open while the nested popup or its
  triggers/viewport are hovered; a delayed hover-out of the nested submenu closes the parent after
  `closeDelay`; parent close closes the inline nested viewport.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2389-2421`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2468-2501`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2503-2536`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2869-2893`
- Inline nested safePolygon: pointer-events lock is scoped to the submenu list, follows the
  traversal path toward the viewport in all four directions, is cleared when the pointer leaves the
  path, and is re-applied when returning to the original trigger before traversing again.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2679-2717`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2719-2780`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2782-2813`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:2815-2867`
- WebKit-specific: the top-level safePolygon lock is applied to the top-level list (not
  `document.body`), and nested safePolygon locks stay scoped to their own submenu list.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.webkit.test.tsx:104-118`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.webkit.test.tsx:120-139`
- Custom (non-ul) list elements scope the lock to `document.body` instead of the list element.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1175-1219`
- Positioner repositions horizontally when switching triggers via hover (popup anchor follows the
  newly hovered trigger).
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:433-453`

## Shared harness dependencies

- Tests import `createRenderer`, `describeConformance`, and `isJSDOM` from `#test-utils`, which is
  `packages/react/test/index.ts` (re-exporting `@base-ui/utils/testUtils` plus local helpers).
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:8`,
  `packages/react/test/index.ts:1-11`
- `createRenderer` wraps render/rerender/setProps in `act` and exposes a fake-timer `clock`.
  `packages/react/test/createRenderer.ts:27-49`
- `describeConformance` runs the propsSpread/refForwarding/renderProp/className conformance tests
  against each part.
  `packages/react/test/describeConformance.tsx:44-49`
- Tests also use `@mui/internal-test-utils` directly (external package): `fireEvent`, `screen`,
  `flushMicrotasks`, `act`, `within`, `waitFor`, and `userEvent` from
  `@testing-library/user-event`.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:3`,
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:6-7`
- `isJSDOM` gates Chromium-only tests (`it.skipIf(isJSDOM)`) for real-hover and focus/positioning
  assertions.
  `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.test.tsx:200`,
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:1221`
- Constants imported by the tests: `OPEN_DELAY` (50) and `CLOSE_DELAY` (50) from the component's
  own utils, and `PATIENT_CLICK_THRESHOLD` (500) from internals.
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.test.tsx:9-10`,
  `packages/react/src/navigation-menu/utils/constants.ts:1-3`,
  `packages/react/src/internals/constants.ts:4`
