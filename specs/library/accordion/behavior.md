# Accordion behavior spec

Mined from the five accordion test files listed below. The `TODO.md` entry
(`TODO.md:301-307`) has no `wraps-external:` field, so the behavior below is derived entirely
from the component's own tests; there is no third-party package to delegate to. The unit is not
on the `needs-batched-mining: true` list, so this single file covers the whole accordion.

Files mined:
- `packages/react/src/accordion/root/AccordionRoot.test.tsx`
- `packages/react/src/accordion/header/AccordionHeader.test.tsx`
- `packages/react/src/accordion/item/AccordionItem.test.tsx`
- `packages/react/src/accordion/panel/AccordionPanel.test.tsx`
- `packages/react/src/accordion/trigger/AccordionTrigger.test.tsx`

## Public API surface (props, parts, subcomponents)

- Parts exercised by the tests, all imported from the `@base-ui/react/accordion` namespace:
  `Accordion.Root`, `Accordion.Item`, `Accordion.Header`, `Accordion.Trigger`,
  `Accordion.Panel`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:4`, `packages/react/src/accordion/header/AccordionHeader.test.tsx:2`, `packages/react/src/accordion/panel/AccordionPanel.test.tsx:4`
- `Accordion.Root`:
  - `defaultValue` (uncontrolled initial open items, `Value[]`). `packages/react/src/accordion/root/AccordionRoot.test.tsx:43`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:284`
  - `value` (controlled open items). `packages/react/src/accordion/root/AccordionRoot.test.tsx:312`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:344`
  - `onValueChange(nextValue, eventDetails)`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:438`
  - `multiple` / `multiple={false}`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:796`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:837`
  - `disabled`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:372`
  - `hiddenUntilFound`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:24`
  - `keepMounted`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:24`, `packages/react/src/accordion/panel/AccordionPanel.test.tsx:99`
- `Accordion.Item`:
  - `value` (any). `packages/react/src/accordion/root/AccordionRoot.test.tsx:44`
  - `disabled`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:402`
  - `onOpenChange(nextOpen, eventDetails)` — item-level open callback. `packages/react/src/accordion/root/AccordionRoot.test.tsx:435`
  - `render={(props, state) => ...}` — render prop receives a state object with `open` and
    `hidden` booleans. `packages/react/src/accordion/item/AccordionItem.test.tsx:36-38`
- `Accordion.Header`: no props exercised directly; used solely as a structural wrapper in
  conformance tests rendered inside `Root > Item`. `packages/react/src/accordion/header/AccordionHeader.test.tsx:20-28`
- `Accordion.Trigger`:
  - `nativeButton` (boolean). `packages/react/src/accordion/root/AccordionRoot.test.tsx:503`, `packages/react/src/accordion/trigger/AccordionTrigger.test.tsx:26`
  - `render` (element form, e.g. `<span />`). `packages/react/src/accordion/root/AccordionRoot.test.tsx:503`, `packages/react/src/accordion/trigger/AccordionTrigger.test.tsx:26`
  - `id` (hooks into the panel-linking ARIA mechanism). `packages/react/src/accordion/root/AccordionRoot.test.tsx:86`
  - `disabled`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:445`
  - `onMouseUp` (may call `event.preventBaseUIHandler()`). `packages/react/src/accordion/root/AccordionRoot.test.tsx:478`
- `Accordion.Panel`:
  - `id` (manual id referenced by `aria-controls`). `packages/react/src/accordion/root/AccordionRoot.test.tsx:69`
  - `keepMounted` / `keepMounted={false}`. `packages/react/src/accordion/panel/AccordionPanel.test.tsx:21`, `packages/react/src/accordion/panel/AccordionPanel.test.tsx:38`
  - `hiddenUntilFound` / `hiddenUntilFound={false}`. `packages/react/src/accordion/panel/AccordionPanel.test.tsx:38`, `packages/react/src/accordion/panel/AccordionPanel.test.tsx:125`
  - `style` (inline animation styles participate in SSR/Activity suppression). `packages/react/src/accordion/panel/AccordionPanel.test.tsx:75-81`, `packages/react/src/accordion/panel/AccordionPanel.test.tsx:231-239`
  - `className` and `data-testid` forwarded to the panel element. `packages/react/src/accordion/panel/AccordionPanel.test.tsx:158`, `packages/react/test/conformanceTests/className.tsx:20-23`
- Conformance suite defaults/behavior: every part forwards unknown props (`lang`, `data-*`),
  `style`, and `className` to its root element, and refs attach to the default root element
  (Root/Panel/Item -> `HTMLDivElement`, Header -> `HTMLHeadingElement`, Trigger ->
  `HTMLButtonElement`). `packages/react/test/conformanceTests/propForwarding.tsx:23-36`, `packages/react/test/conformanceTests/refForwarding.tsx:31-37`, `packages/react/test/conformanceTests/className.tsx:20-23`, `packages/react/test/conformanceTests/renderProp.tsx:41-75`

## State model (controlled/uncontrolled, defaults, transitions)

- Root open-state is a list of item values, controlled via `value` or uncontrolled via
  `defaultValue`; both accept custom item values as well as indices.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:43`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:284`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:312`
- Default (no `value`/`defaultValue`): all items are closed. A closed, non-`keepMounted` panel is
  fully absent from the DOM (`queryByText(...).toBe(null)`); its trigger has
  `aria-expanded="false"`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:264-265`
- Controlled transitions: `setProps({ value: [0] })` opens the item; `setProps({ value: [] })`
  closes it — the DOM follows the prop. `packages/react/src/accordion/root/AccordionRoot.test.tsx:327-338`
- Pointer/keyboard activation toggles each item: closed -> open -> closed on successive
  activations. `packages/react/src/accordion/root/AccordionRoot.test.tsx:267-278`
- `multiple={false}` (default): opening item 2 closes item 1, so at most one item is open.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:865-870`
- `multiple`: each item opens/closes independently; clicking the open item's trigger closes only
  that item while the other stays open. `packages/react/src/accordion/root/AccordionRoot.test.tsx:819-832`
- State reported to consumers:
  - `onValueChange` receives the new full value array. `packages/react/src/accordion/root/AccordionRoot.test.tsx:902`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:910`
  - Item-level `onOpenChange` receives `(nextOpen: boolean, eventDetails)`.
    `packages/react/src/accordion/root/AccordionRoot.test.tsx:576`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:587`
  - `Accordion.Item`'s `render` prop receives `state` with `open`/`hidden` fields; starting to open
    never produces `{ open: true, hidden: true }`. `packages/react/src/accordion/item/AccordionItem.test.tsx:49-53`
- Cancellation: `eventDetails.cancel()` on either `onOpenChange` or `onValueChange` blocks the
  state change (see Events section). `packages/react/src/accordion/root/AccordionRoot.test.tsx:593-621`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:623-646`
- Disabled (root or item) prevents all state transitions; see Events / Edge cases.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:460-468`

## Keyboard interactions

- Both native-button triggers and non-interactive (`render={<span />}`, `nativeButton={false}`)
  triggers toggle open/closed with Enter and Space. `packages/react/src/accordion/root/AccordionRoot.test.tsx:496-533`
- Space-specific timing: pressing Space (keydown, `[Space>]`) does NOT toggle — `aria-expanded`
  stays `false`, the panel stays out of the DOM, and `onOpenChange` is not called; releasing Space
  (keyup, `[/Space]`) toggles and fires `onOpenChange` exactly once. The same applies when
  closing. `packages/react/src/accordion/root/AccordionRoot.test.tsx:567-587`
- Keyboard activation fires `onValueChange` with the same `reason` (`REASONS.triggerPress`) as
  pointer activation. `packages/react/src/accordion/root/AccordionRoot.test.tsx:907-912`
- Disabled items: Space and Enter on a (manually focused) trigger produce no toggle and no
  `onValueChange`/`onOpenChange` calls. `packages/react/src/accordion/root/AccordionRoot.test.tsx:460-468`
- Enter/Scroll keyup-vs-keydown timing beyond Space is not asserted. UNVERIFIED — inferred from
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:496-533`, no test distinguishes Enter
  keydown from keyup.

## Focus management

- Both native and non-native triggers are focused via `[Tab]` before keyboard activation.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:519-520`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:564-565`
- A non-native trigger (`nativeButton={false}`, `render={<span />}`) is kept tabbable via
  `tabindex="0"`. `packages/react/src/accordion/trigger/AccordionTrigger.test.tsx:21-37`
- Focus retention is not managed by the accordion: no test asserts where focus goes after a panel
  opens/closes or after a trigger is removed. UNVERIFIED — inferred, no test asserts focus
  movement on open/close.
- Whether disabled triggers are skipped by Tab is not asserted (the disabled test focuses the
  trigger programmatically). UNVERIFIED — inferred from
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:461`, no test asserts Tab behavior on a
  disabled trigger.

## Accessibility (roles, aria-*, id linking)

- Trigger/panel pairing uses generated ids: trigger gets `aria-controls` pointing at the panel's
  `id`; the panel gets `role="region"` and `aria-labelledby` pointing at the trigger's `id`.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:53-59`
- Manual ids are honored: a panel `id="custom-panel-id"` is referenced by the trigger's
  `aria-controls`; a trigger `id="custom-trigger-id"` is referenced by the panel's
  `aria-labelledby`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:62-79`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:81-96`
- Dynamic id changes propagate: assigning a trigger id after mount updates the panel's
  `aria-labelledby`; removing a manual trigger id reverts the panel to the (new) generated
  trigger id. `packages/react/src/accordion/root/AccordionRoot.test.tsx:98-143`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:145-180`
- Generated id associations unregister on unmount: unmounting the trigger removes the panel's
  `aria-labelledby`; unmounting the panel removes the trigger's `aria-controls`; both restore on
  remount. `packages/react/src/accordion/root/AccordionRoot.test.tsx:182-217`
- Associations survive hydration: after `hydrate()`, trigger `aria-controls` and panel
  `aria-labelledby` still reference each other. `packages/react/src/accordion/root/AccordionRoot.test.tsx:219-246`
- Open state on the trigger is exposed as `aria-expanded` (`true`/`false`).
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:264`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:269`
- Disabled propagation exposes `data-disabled` on the item, header, trigger, and panel (root- or
  item-level). `packages/react/src/accordion/root/AccordionRoot.test.tsx:394-396`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:423-428`
- No `aria-disabled`, `role` on items/headers, or `aria-*` on the root are asserted anywhere.
  UNVERIFIED — inferred, no test covers them.
- `data-orientation` and `data-index` are never asserted in these tests. UNVERIFIED — inferred, no
  test covers them.

## DOM structure & portal behavior

- Root renders a `div` (`refInstanceof: window.HTMLDivElement`); Item and Panel also render `div`s;
  Header renders a heading (`HTMLHeadingElement`); Trigger renders a `button`.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:14-17`, `packages/react/src/accordion/item/AccordionItem.test.tsx:21-26`, `packages/react/src/accordion/header/AccordionHeader.test.tsx:20-28`, `packages/react/src/accordion/trigger/AccordionTrigger.test.tsx:9-12`, `packages/react/src/accordion/panel/AccordionPanel.test.tsx:21-29`
- Closed panel unmounting: with default `keepMounted`, a closed panel is removed from the DOM
  entirely. `packages/react/src/accordion/root/AccordionRoot.test.tsx:265`
- Kept-mounted closed panels: `keepMounted` on the root applies to its closed panels, which render
  with the `hidden` attribute. `packages/react/src/accordion/panel/AccordionPanel.test.tsx:97-110`
- `hiddenUntilFound` (root or panel) renders the closed panel with `hidden="until-found"`; a
  panel can override the root's `hiddenUntilFound={false}` plus `keepMounted={false}` to unmount
  entirely instead. `packages/react/src/accordion/root/AccordionRoot.test.tsx:34`, `packages/react/src/accordion/panel/AccordionPanel.test.tsx:112-134`
- Open panel attributes: `data-open` on the panel and `data-panel-open` on its trigger.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:270-273`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:814-815`
- Transition attributes/CSS vars on the panel: when open, `--accordion-panel-height` is `auto`;
  when another item opens while this one closes, the closing panel gets `data-ending-style`, still
  lacks `hidden`, and `--accordion-panel-height` resolves to a `px` value until its exit
  transition ends, after which it becomes `hidden`. `packages/react/src/accordion/panel/AccordionPanel.test.tsx:179-196`
- `data-starting-style` is referenced by test CSS but never asserted. UNVERIFIED — inferred from
  `packages/react/src/accordion/panel/AccordionPanel.test.tsx:147-150`, no test asserts the
  attribute itself.
- No portal usage: nothing is rendered outside the root's DOM tree; no portal behavior is tested.
  N/A for portal behavior.
- SSR: an open panel rendered to string has inline `animationName` overridden to `none` while
  `animationDuration` is preserved (initial open animation suppressed). `packages/react/src/accordion/panel/AccordionPanel.test.tsx:54-95`

## Events (names, payload shape, bubbling, preventDefault semantics)

- Activation events that toggle state: pointer click (testing-library `[MouseLeft]`) on the
  trigger, `fireEvent.click`, Enter, and Space keyup all toggle. `packages/react/src/accordion/root/AccordionRoot.test.tsx:267`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:616`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:499-533`
- `onValueChange(value: Value[], eventDetails)` payload: `eventDetails.reason` is
  `REASONS.triggerPress` for both pointer and keyboard activation, and `eventDetails.event.type`
  is not `'base-ui'` (a real user-event-derived detail). `packages/react/src/accordion/root/AccordionRoot.test.tsx:901-912`, and the `REASONS` import at `packages/react/src/accordion/root/AccordionRoot.test.tsx:6`
- In `multiple` mode the new value array appends the toggled item's value; in
  non-`multiple` mode it replaces the array with just the newly opened item.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:910`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:976-982`
- `onOpenChange(nextOpen: boolean, eventDetails)` fires exactly once per activation (on Space it
  fires on keyup, not keydown). `packages/react/src/accordion/root/AccordionRoot.test.tsx:570-576`
- `eventDetails.cancel()` semantics:
  - `onOpenChange` cancel prevents an uncontrolled open and prevents `onValueChange` from firing.
    `packages/react/src/accordion/root/AccordionRoot.test.tsx:593-621`
  - `onValueChange` cancel prevents an uncontrolled open, an uncontrolled close, a controlled
    open (state stays `[]`), and both directions in `multiple` mode — cancelling is the consumer's
    job when controlled. `packages/react/src/accordion/root/AccordionRoot.test.tsx:623-646`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:648-672`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:704-740`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:742-790`
  - The attempted value is still passed to `onValueChange` even when cancelled (e.g. closing with
    `defaultValue={[0]}` yields `[]`). `packages/react/src/accordion/root/AccordionRoot.test.tsx:670-671`
  - A controlled consumer checks `eventDetails.isCanceled` before applying the new value.
    `packages/react/src/accordion/root/AccordionRoot.test.tsx:713-718`
- `onMouseUp` receives the event and may call `event.preventBaseUIHandler()` without throwing.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:473-490`
- Disabled (root or item): pointer, Space, and Enter activation produce no toggle and no
  `onValueChange`/`onOpenChange` calls. `packages/react/src/accordion/root/AccordionRoot.test.tsx:460-468`
- Event bubbling past the trigger (e.g. to an ancestor handler) is not asserted. UNVERIFIED — no
  test covers bubbling.

## Edge cases (rapid interactions, unmount, nesting)

- Consecutive activations toggle reliably (open then close), and rapid Space down/up sequences
  only commit on keyup. `packages/react/src/accordion/root/AccordionRoot.test.tsx:267-278`, `packages/react/src/accordion/root/AccordionRoot.test.tsx:567-587`
- Disabled wins over a trigger's own `disabled={false}`: when the root or item is disabled, an
  explicitly non-disabled trigger still does not toggle or fire callbacks.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:431-468`
- `hiddenUntilFound` forces panels to stay mounted: combining it with `keepMounted={false}` on the
  root or a panel produces a `console.warn` ("Base UI: The `keepMounted={false}` prop ... is
  ignored when `hiddenUntilFound` is enabled ...") and the panel is rendered with
  `hidden="until-found"`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:19-38`, `packages/react/src/accordion/panel/AccordionPanel.test.tsx:31-52`
- Unmount cleanup: generated part ids are unregistered when the trigger or panel unmounts (in
  `React.StrictMode`), and re-associated on remount. `packages/react/src/accordion/root/AccordionRoot.test.tsx:182-217`
- Closed non-`keepMounted` panels are unmounted (see DOM structure), so open/close is a full
  mount/unmount cycle. `packages/react/src/accordion/root/AccordionRoot.test.tsx:265-278`
- Hydration: generated trigger/panel associations survive `hydrate()`. `packages/react/src/accordion/root/AccordionRoot.test.tsx:219-246`
- SSR animation suppression and `React.Activity` reveal: opening/ending animations defined via
  inline styles do not replay when a user-opened panel is revealed after being hidden by
  `React.Activity`, nor on initial SSR render. `packages/react/src/accordion/panel/AccordionPanel.test.tsx:54-95`, `packages/react/src/accordion/panel/AccordionPanel.test.tsx:200-275`
- Nested accordions (an accordion inside an accordion item) are not tested. UNVERIFIED — no test
  covers nesting.

## Shared harness dependencies

- `#test-utils` (module alias `./test/index.ts`, defined at `packages/react/package.json:110`)
  provides `describeConformance`, `createRenderer`, and `isJSDOM`, used by all five test files.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:5`
- `describeConformance` (`packages/react/test/describeConformance.tsx:44-49`) runs four suites —
  prop forwarding, ref forwarding, render prop, and className — for every accordion part with the
  options each test file passes (`refInstanceof` per part; `Accordion.Trigger` additionally passes
  `testComponentPropWith: 'button'` and `button: true`). `packages/react/test/describeConformance.tsx:10-42`, `packages/react/src/accordion/trigger/AccordionTrigger.test.tsx:9-19`, `packages/react/src/accordion/panel/AccordionPanel.test.tsx:21-29`
- `createRenderer` (`packages/react/test/createRenderer.ts:27-48`) wraps
  `@mui/internal-test-utils`'s renderer inside an awaited `act`; its `render` is a promise, and
  also exposes awaited `rerender`, `setProps`, and the user-event `user` instance.
  `packages/react/test/createRenderer.ts:31-43`
- `isJSDOM` gates Chrome-only tests (re-exported from `@base-ui/utils/testUtils` via
  `#test-utils`). `packages/react/test/index.ts:1`
- `fireEvent`, `screen`, `waitFor`, and `reactMajor` come from `@mui/internal-test-utils`.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:3`, `packages/react/src/accordion/panel/AccordionPanel.test.tsx:2`
- `REASONS` is imported from the source tree (`../../internals/reasons`), not a harness; the tests
  only use it to assert the `reason` value in `onValueChange` details.
  `packages/react/src/accordion/root/AccordionRoot.test.tsx:6`