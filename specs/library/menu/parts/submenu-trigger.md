# Menu.SubmenuTrigger — Behavior Spec

## Public API surface (props, parts, subcomponents)

- `Menu.SubmenuTrigger` renders a `div` with role `menuitem` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:32` and must be placed inside `Menu.SubmenuRoot`; it throws if rendered outside it `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:93-103`.
- Accepts `disabled` prop: when set, renders `data-disabled` and `aria-disabled="true"` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:240-241`.
- Accepts `openOnHover` prop (default: `true`); when `false`, hover does not open the submenu `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.talkBack.test.tsx:95-110`.
- Accepts `delay` prop; a disabled trigger with `delay={0}` still does not open on hover `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:253-275`.
- Accepts `label` prop used for type-ahead text navigation `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:187-209`.
- Accepts `nativeButton` and `render` props; when a native disabled button is provided via `render`, a dev warning is emitted `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:278-310`.
- The `id` prop is tracked by the menu store; changing it updates the store's trigger element registry `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:49-91`.

## State model (controlled/uncontrolled, defaults, transitions)

- The component is controlled by the parent `Menu.Root` open state; the submenu trigger itself does not own open/close state — UNVERIFIED, inferred from every test rendering `<Menu.Root open>` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:59`; no test asserts the state owner.
- `openOnHover` defaults to `true` (implicit from TalkBack tests using default props) `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.talkBack.test.tsx:83-93`.
- A `disabled` submenu trigger never opens regardless of hover or delay settings `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:244-276`.

## Keyboard interactions

- **ArrowRight** (LTR) / **ArrowLeft** (RTL) opens the submenu and highlights the first item `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:133-165`.
- **ArrowLeft** (LTR) / **ArrowRight** (RTL) closes the submenu — UNVERIFIED, inferred from the `closeKey` values in the test-case table `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:134-136`; no test ever exercises `closeKey`.
- On VoiceOver, **Enter** also opens the submenu `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:73-92`.
- After opening via keydown, the trigger's `tabIndex` is set to `0` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:167-177`.
- When the submenu opens via keyboard, the first submenu item receives focus and is highlighted; parent menu items lose their `data-highlighted` attribute `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:139-164`.

## Focus management

- Keyboard open (ArrowRight/ArrowLeft) moves focus into the submenu, landing on the first `menuitem` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:150-151`.
- Pointer open keeps focus on the trigger itself — UNVERIFIED, inferred from the test comment `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:104`; no test asserts focus location after a pointer open (the test asserts `aria-expanded="true"` on the trigger instead `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:105`).
- After keyboard opening, the trigger retains `tabIndex="0"` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:174-176`.

## Accessibility (roles, aria-*, id linking)

- The trigger renders as a `menuitem` (inferred from `screen.getByRole('menuitem')` queries) `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:238`.
- When `disabled`, the trigger gets `aria-disabled="true"` and `data-disabled` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:240-241`.
- On VoiceOver, before the submenu opens, the trigger has `aria-expanded="false"` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:58`.
- On VoiceOver, after keyboard opening, the `aria-expanded` attribute is **removed** (not set to `"true"`) to avoid VoiceOver announcing the expanded state and talking over the submenu item announcement `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:68`.
- On VoiceOver, the trigger always has `aria-haspopup="menu"` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:70`.
- On VoiceOver, after pointer opening, `aria-expanded="true"` is **kept** because focus stays on the trigger and there is no item announcement to conflict with `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:105`.

## DOM structure & portal behavior

- The submenu trigger is a `div` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:32`.
- The submenu content is rendered in a separate `Menu.Portal` nested inside the `Menu.SubmenuRoot` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:3-46`.
- Disabled state produces `data-disabled` on the trigger element `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:240`.
- Highlighted state is indicated via `data-highlighted` on menu items `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:154`.

## Events (names, payload shape, bubbling, preventDefault semantics)

- N/A — no custom events are dispatched or tested by `Menu.SubmenuTrigger`.

## Edge cases (rapid interactions, unmount, nesting)

- Dev-mode warning when a native disabled button is used via `render` prop: warns to use the `disabled` prop instead `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:278-310`.
- The warning includes the owner stack when `React.captureOwnerStack` is available, and omits `"undefined"` from the message when it is not `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:313-348`.
- In production (`NODE_ENV=production`), the disabled-element inspection is skipped and no warning is emitted `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:350-379`.
- The TalkBack press detection relies on zero-pressure pointer events (virtual press shape); a real mouse press with non-zero pressure does not trigger the submenu open `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.talkBack.test.tsx:112-132`.
- TalkBack press with `openOnHover={false}` opens via mousedown path, and the subsequent trailing click does not toggle it closed `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.talkBack.test.tsx:95-110`.
- On VoiceOver, opening the submenu with Enter removes `aria-expanded` to prevent announcement overlap, same as ArrowRight `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:73-92`.

## Shared harness dependencies

- `#test-utils` → resolves to `packages/react/test/index.ts`, which re-exports `createRenderer`, `describeConformance`, `isJSDOM`, and utilities from `@base-ui/utils/testUtils` `packages/react/test/index.ts:1-14`.
- The TalkBack test mocks `@base-ui/utils/platform` to set `platform.os.android = true` for virtual-press detection `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.talkBack.test.tsx:10-20`.
- The VoiceOver test mocks `@base-ui/utils/platform` to set `platform.screenReader.voiceOver = true` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:8-18`.
- The main test sets `globalThis.BASE_UI_ANIMATIONS_DISABLED = true` in `beforeEach` and drains a requestAnimationFrame in `afterEach` `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:16-29`.
