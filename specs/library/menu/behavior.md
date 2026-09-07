# Menu — Behavior Spec (whole-unit index)

Mining method: `library: menu` is flagged `needs-batched-mining: true` (`TODO.md:418`), so this unit was mined in
nine parallel batches, one per clustered subdirectory of `packages/react/src/menu/`. Each batch produced a part file
under `specs/library/menu/parts/` using the full 8-section behavior template (Public API surface; State model;
Keyboard interactions; Focus management; Accessibility; DOM structure & portal behavior; Events; Edge cases) plus a
Shared harness dependencies section. This file is the index: one paragraph per part pointing into the part file for
depth, followed by the behavior that is genuinely cross-cutting and only makes sense at the whole-unit level.
The unit has no `wraps-external:` entry, so no third-party delegation applies.

## Part index

- **`arrow-backdrop-portal-viewport`** — `Menu.Arrow`, `Menu.Backdrop`, `Menu.Portal`, and `Menu.Viewport`.
  Arrow is a `div` ref nested under Popup>Positioner>Portal; Backdrop is a `div` whose inline `pointerEvents` is
  `'none'` only when the menu was opened on hover; Portal supports `keepMounted`; Viewport wraps its children in an
  internal `data-current` container that is remounted (new DOM node) whenever the active trigger/payload changes,
  and during Chromium-only morph transitions renders an `inert` `data-previous` container plus a
  `data-activation-direction` token set on itself.
  `specs/library/menu/parts/arrow-backdrop-portal-viewport.md:1-61`

- **`checkbox-item`** — `Menu.CheckboxItem` renders role `menuitemcheckbox` and mirrors checked state in
  `aria-checked` plus `data-checked`/`data-unchecked`; it is controlled/uncontrolled via `checked`/`defaultChecked`,
  toggles on click/Enter/Space, reports `onCheckedChange(newChecked, { cancel })` (cancelable), does not close the
  menu by default (`closeOnClick` opt-in), and is `disabled`-safe while still focusable. `Menu.CheckboxItemIndicator`
  unmounts when unchecked unless `keepMounted` or a CSS exit animation (`data-ending-style`/`onAnimationEnd`) keeps it
  mounted through the transition. `specs/library/menu/parts/checkbox-item.md:1-81`

- **`group-item-link`** — `Menu.Group` (role `group`) and `Menu.GroupLabel` (default `aria-hidden="true"`, registers
  its `id` with the group's `aria-labelledby`, correct under remount ordering); `Menu.Item` (role `menuitem`, closes
  the menu on click by default, `preventBaseUIHandler()` vetoes that close, disabled items are focusable but skipped in
  navigation); and `Menu.LinkItem` (role `menuitem` on an anchor, Enter/Space trigger router navigation except during an
  active typeahead session). `specs/library/menu/parts/group-item-link.md:1-63`

- **`popup`** — `Menu.Popup` must be rendered inside `Menu.Positioner` (else throws), has `finalFocus`
  (ref / function / `false` to disable / `true`/`null` to use the default trigger return), stops toolbar arrow keys from
  bubbling inside a Toolbar context, and — per the Chromium-only `MenuPopupEnterTransition` suite — plays enter
  transitions (`data-starting-style`, `data-instant="click"` on instant keyboard open) for itself and nested submenus
  under a matrix of `defaultOpen`/controlled `open`/`keepMounted` combinations, with the inherited `instantType` seed
  cleared across interruptions and late-mounted popup subtrees.
  `specs/library/menu/parts/popup.md:1-68`

- **`positioner`** — `Menu.Positioner` requires a `Menu.Portal` ancestor (else throws); accepts `anchor` as ref,
  element, function, virtual element (`getBoundingClientRect` only), or `undefined` (falls back to trigger); accepts
  `side`/`align`/`sideOffset`/`alignOffset` (number or resolver function) and `collisionAvoidance`; defaults to
  `"bottom"`/`"inline-end"` for horizontal/vertical menubars; positions with inline `transform` unless a `Menu.Viewport`
  is present (then top/left); supports `keepMounted` (content stays in DOM but `toBeInaccessible()` when closed); and a
  nested submenu closes with reason `'sibling-open'` when its controlled parent closes.
  `specs/library/menu/parts/positioner.md:1-72`

- **`radio-group-item`** — `Menu.RadioGroup` (role `group`) is uncontrolled via `defaultValue` or controlled via `value`
  with `onValueChange(newValue, { cancel })`, and has an optional group-level `disabled`. `Menu.RadioItem` renders role
  `menuitemradio` with `aria-checked`/`data-checked`, does not close the menu by default (`closeOnClick`), is skipped for
  keyboard activation when disabled but remains focusable, and is driven by typeahead. `Menu.RadioItemIndicator` follows
  the same unmount/`keepMounted`/exit-animation model as the checkbox indicator.
  `specs/library/menu/parts/radio-group-item.md:1-87`

- **`root`** — The whole-menu state machine: controlled `open`/uncontrolled `defaultOpen` (default closed),
  `onOpenChange(nextOpen, { reason, event, trigger })` with reasons `itemPress`/`cancelOpen`/`outsidePress`/`siblingOpen`/
  `triggerHover` and keyboard-vs-mouse disambiguation via `event.detail`, cancelable via `details.cancel()` and
  unmount-overridable via `details.preventUnmountOnClose()`, plus `onOpenChangeComplete`; `modal` and its `role="presentation"`
  backdrop; `orientation`; a root-level `disabled`; `highlightItemOnHover`; keyboard navigation via arrows/Home/End across
  disabled and CSS-hidden items, typeahead over labels/diacritics, Enter/ArrowDown/ArrowUp open focus targeting, Escape
  level-by-level with `closeParentOnEsc`; tab/shift-tab guards including nested dialogs; scroll locking that depends on
  mouse-vs-touch open; PATIENT_CLICK_THRESHOLD click-drag/release semantics; submenu hover open/close delays and safe-polygon
  pointer-events; `keepMounted`/`preventUnmountOnClose`/`actionsRef`; multi-trigger `payload` dispatch; and the
  `Menu.createHandle()` detached-trigger imperative API (`open(triggerId)`/`close()`/`isOpen`) with multi-root handoff.
  `specs/library/menu/parts/root.md:1-129`

- **`submenu-trigger`** — `Menu.SubmenuTrigger` renders a `div` with role `menuitem` and throws outside `Menu.SubmenuRoot`;
  `openOnHover` defaults to `true`; `delay`/`label`/`disabled` props (disabled never opens); orientation-aware arrow keys
  open the submenu and focus its first item with `tabIndex=0`; on VoiceOver keyboard open removes `aria-expanded` (to avoid
  announcement overlap) while pointer open keeps `"true"` with `aria-haspopup="menu"`; TalkBack virtual (zero-pressure)
  presses open the submenu while ordinary mouse presses do not. `specs/library/menu/parts/submenu-trigger.md:1-67`

- **`trigger`** — `Menu.Trigger` renders a native button by default (`nativeButton`/`render` to override), opens on click
  and on ArrowDown/ArrowUp/Enter/Space (Enter/Space only tested for non-native triggers); exposes
  `aria-haspopup`/`aria-expanded`/`data-popup-open`/`data-pressed`; honors `preventBaseUIHandler()` in `onMouseDown`
  (blocks mouse open) and `onClick` (blocks keyboard open); supports `openOnHover`/`delay` with
  `PATIENT_CLICK_THRESHOLD` impatient/patient click stickiness; works as a handle-backed detached trigger with controlled
  close-veto; and throws without a `Menu.Root` ancestor or `handle`. `specs/library/menu/parts/trigger.md:1-69`

## Cross-cutting behavior

- **One shared state machine.** `Menu.Root` owns visibility and hosts the store; every other part is presentational or an
  input to it. Popup content is present in the DOM only while open unless `keepMounted` (or `preventUnmountOnClose`), in
  which case it stays mounted but is inaccessible.
  `specs/library/menu/parts/root.md:15-29`, `specs/library/menu/parts/positioner.md:29-29`

- **Uniform DOM shell.** All interactive content hangs off `Menu.Root > Menu.Portal > Menu.Positioner > Menu.Popup`, with
  `Menu.Trigger` for the root opener and per-level `Menu.Portal > Menu.Positioner > Menu.Popup` for each `Menu.SubmenuRoot`.
  `specs/library/menu/parts/root.md:81-89`, `specs/library/menu/parts/popup.md:44-51`

- **Context-ancestry contract.** Every part except the trigger throws a `Base UI:`-prefixed, context-naming error when
  rendered without its required ancestor — Item needs Root, Popup needs Positioner, Positioner needs Portal, GroupLabel
  needs Group/RadioGroup, CheckboxItemIndicator needs CheckboxItem, RadioItem needs RadioGroup, RadioItemIndicator needs
  RadioItem, SubmenuTrigger needs SubmenuRoot, Trigger needs Root or a handle.
  `specs/library/menu/parts/group-item-link.md:56-59`, `specs/library/menu/parts/radio-group-item.md:77-78`,
  `specs/library/menu/parts/popup.md:5-5`, `specs/library/menu/parts/positioner.md:7-7`,
  `specs/library/menu/parts/submenu-trigger.md:5-5`, `specs/library/menu/parts/trigger.md:61-61`

- **Shared keyboard model.** Arrows/Home/End move highlight along the composite list skipping disabled (`aria-disabled`)
  and CSS-hidden items; typeahead matches visible text and the `label` prop (diacritic-tolerant) and consumes Space while a
  session is active so it never activates items or toggles selection; orientation/writing-direction-aware arrow keys enter
  and leave submenus; Escape closes level by level and returns focus to each trigger, with `closeParentOnEsc` opting the
  whole tree in. `specs/library/menu/parts/root.md:31-49`, `specs/library/menu/parts/submenu-trigger.md:19-25`

- **Shared `onOpenChange` reason taxonomy.** The same reason vocabulary crosses nesting levels — `itemPress`, `cancelOpen`,
  `outsidePress`, `siblingOpen` (a child closes because its controlled parent closed), `triggerHover` — and the events carry
  a `detail` distinguishing keyboard/instant activation (`0`) from mouse drag-release activation.
  `specs/library/menu/parts/root.md:91-99`, `specs/library/menu/parts/positioner.md:55-58`

- **Focus returns to the opener.** Closing a menu returns focus to its trigger by default; `Menu.Popup finalFocus`
  overrides the target; nested submenus hand focus back level by level to each submenu trigger.
  `specs/library/menu/parts/popup.md:31-39`, `specs/library/menu/parts/root.md:51-65`

- **ARIA model across the tree.** Popup is role `menu` (with `aria-orientation` only when horizontal); items are
  `menuitem`/`menuitemcheckbox`/`menuitemradio`; the trigger links the popup via `aria-controls` and `aria-expanded`
  (with a VoiceOver keyboard-open caveat on submenu triggers) plus `aria-haspopup`; root menu owns the popup through an
  `aria-owns` span while submenus own theirs through a `role="group"` child.
  `specs/library/menu/parts/root.md:67-79`, `specs/library/menu/parts/trigger.md:31-39`,
  `specs/library/menu/parts/submenu-trigger.md:33-40`

- **Chromium-only surface.** Enter transitions (`data-starting-style`/`data-ending-style`/`data-instant`), the Viewport
  morph, pointer-timing/inertia (`PATIENT_CLICK_THRESHOLD` clicks, firePointer-velocity), TalkBack/VoiceOver, and
  browser-scroll-lock coverage are all gated behind `describe.skipIf(isJSDOM)` and rely on the suites toggling
  `BASE_UI_ANIMATIONS_DISABLED`; jsdom tests cover the pure logic.
  `specs/library/menu/parts/arrow-backdrop-portal-viewport.md:55-55`, `specs/library/menu/parts/popup.md:64-64`,
  `specs/library/menu/parts/trigger.md:54-60`, `specs/library/menu/parts/submenu-trigger.md:64-67`

- **Shared harness across all 22 test files.** Every part file's tests import `#test-utils`
  (`packages/react/test/index.ts`), which re-exports `createRenderer`, `describeConformance`, `isJSDOM`, `firePointer`,
  `resetBrowserPointer`, `wait`, and pointer/time helpers from `@base-ui/utils/testUtils`; `describeConformance` supplies
  the shared prop-spreading/ref-forwarding/render-prop suite for each part.
  `specs/library/menu/parts/root.md:119-129`, `specs/library/menu/parts/trigger.md:64-69`,
  `specs/library/menu/parts/submenu-trigger.md:62-67`