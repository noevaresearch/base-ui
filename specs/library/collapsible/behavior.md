# Collapsible behavior spec

Mined from the three collapsible test files listed below. The `TODO.md` entry (`TODO.md:346-352`)
has no `wraps-external:` field, so the behavior below is derived entirely from the component's own
tests; there is no third-party package to delegate to. The unit is not on the
`needs-batched-mining: true` list (only combobox, drawer, floating-ui-react, menu, number-field,
select are), so this single file covers the whole collapsible unit.

Files mined:
- `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx`
- `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx`
- `packages/react/src/collapsible/trigger/CollapsibleTrigger.test.tsx`

## Public API surface (props, parts, subcomponents)

- Parts exercised by the tests, all imported from the `@base-ui/react/collapsible` namespace as
  `Collapsible.Root`, `Collapsible.Trigger`, `Collapsible.Panel`.
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:11`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:4`, `packages/react/src/collapsible/trigger/CollapsibleTrigger.test.tsx:3`
- `Collapsible.Root`:
  - `defaultOpen` (uncontrolled initial state). `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:21`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:162`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:269`
  - `open` (controlled) + `onOpenChange`. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:182`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:211`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:231`
  - `onOpenChange(nextOpen, eventDetails)`. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:108-130`
  - `disabled`. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:76`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:91`
  - `className` and `style` as functions of state. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:300-343`
  - Arbitrary DOM props (`data-testid`) land on the root element. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:303`
- `Collapsible.Trigger`:
  - `id` is forwarded to the rendered element. `packages/react/src/collapsible/trigger/CollapsibleTrigger.test.tsx:32-43`
  - Renders as a native button (conformance `refInstanceof: window.HTMLButtonElement`,
    `testComponentPropWith: 'button'`, `button: true`). `packages/react/src/collapsible/trigger/CollapsibleTrigger.test.tsx:21-30`
- `Collapsible.Panel`:
  - `keepMounted` / `keepMounted={false}`. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:83`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:61`
  - `hiddenUntilFound`. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:61`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:1571-1573`
  - `id` (manual id referenced by `aria-controls`). `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:39`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:46-47`
  - `render={(props, state) => ReactNode}` — render prop receives a state object with `open` and
    `transitionStatus`; the exported type `Collapsible.Panel.State['transitionStatus']` includes at
    least the `'ending'` value. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:165-171`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:186-196`
  - `style` (object form, including inline `animation*` and `transition*` longhands that the panel
    may temporarily override). `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:849-853`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:1313`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:392`
  - `ref` forwarded to the panel element. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:491-502`
- The panel exposes a `--collapsible-panel-height` CSS variable (see DOM structure section).
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:275`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:282`
- Conformance suite (via `describeConformance`) additionally proves every part forwards unknown
  props, `style`, `className`, refs, and the render prop; Root and Panel refs attach to
  `HTMLDivElement`, Trigger's to `HTMLButtonElement`.
  `packages/react/test/describeConformance.tsx:44-49`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:13-16`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:48-53`, `packages/react/src/collapsible/trigger/CollapsibleTrigger.test.tsx:21-30`
- Rendering any part outside `Collapsible.Root` throws
  `'Base UI: CollapsibleRootContext is missing. Collapsible parts must be placed within <Collapsible.Root>.'`
  (asserted for the Trigger). `packages/react/src/collapsible/trigger/CollapsibleTrigger.test.tsx:9-19`

## State model (controlled/uncontrolled, defaults, transitions)

- Default state is closed: with no props, the trigger has `aria-expanded="false"` and the panel is
  absent from the DOM. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:102-103`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:150-151`
- Uncontrolled mode (`defaultOpen`): pointer clicks on the trigger toggle closed -> open -> closed.
  `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:267-296`
- Controlled mode (`open`): trigger presses request changes through `onOpenChange` but the state
  only changes when the prop is updated externally; without an external update the UI stays as the
  prop says. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:178-224`
- External state updates drive the DOM in controlled mode (open -> `aria-expanded="true"`,
  panel visible, `data-open`; close reverses it). `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:226-265`
- Cancellation: `eventDetails.cancel()` inside `onOpenChange` blocks both the uncontrolled open and
  the uncontrolled close. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:132-152`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:154-174`
- `disabled` blocks all transitions: clicks and Enter/Space produce no toggle and no
  `onOpenChange` call. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:87-104`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:346-369`
- Panel mount states (see DOM structure for attribute detail):
  - Default: a closed panel is unmounted. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:279`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:295`
  - `keepMounted`: a closed panel stays mounted but hidden. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:92-95`
  - `hiddenUntilFound`: a closed panel is forced to stay mounted with `hidden="until-found"`;
    combining it with `keepMounted={false}` warns that `keepMounted={false}` is ignored.
    `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:55-74`
- Close completion semantics: after a close finishes (no transition configured), a kept-mounted
  panel is `hidden` and has `data-closed` with no `data-ending-style`.
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:154-159`
- Transition phases surface as `transitionStatus` on the panel render-prop state; a panel
  intentionally mounted by the consumer only during the `'ending'` phase is unmounted by the
  component once the ending phase has passed. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:164-208`
- Open state is reported to `className`/`style` callbacks on Root, Trigger, and Panel
  simultaneously (`state.open` flips together). `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:300-343`

## Keyboard interactions

- Enter and Space each toggle the collapsible: Tab to the trigger, press the key to open, press
  again to close; `aria-expanded`, `aria-controls`, `data-panel-open`, and panel visibility follow.
  (Browser-only test.) `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:346-405`
- Disabled: Enter and Space on the (focused) trigger produce no toggle and no `onOpenChange` call.
  (Browser-only test.) `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:347-369`
- Keydown-vs-keyup timing for Enter/Space is not distinguished by any test. UNVERIFIED — inferred
  from `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:371-404`, no test asserts
  keydown/keyup split.

## Focus management

- The trigger is tabbable: `user.keyboard('[Tab]')` focuses it (native button focus behavior).
  `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:360-361`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:386-387`
- No other focus behavior is asserted: no test covers focus transfer to the panel on open, focus
  return on close, or focus trapping. UNVERIFIED — inferred from the full test files, no test
  asserts focus movement beyond the trigger.

## Accessibility (roles, aria-*, id linking)

- Trigger/panel pairing uses ids: the trigger has `aria-expanded` (`'true'`/`'false'`) and
  `aria-controls` pointing at the panel's `id`. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:19-33`
- `aria-controls` is present only while open: it is removed when the panel is unmounted
  (default) and also when a `keepMounted` panel is closed, and restored on open.
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:100-106`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:111-114`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:246`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:253`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:262`
- Manual panel ids are honored: `aria-controls` references `<Collapsible.Panel id="custom-panel-id">`.
  `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:35-48`
- Generated panel-id association unregisters when the panel unmounts and restores on remount (even
  under `React.StrictMode`). `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:50-70`
- Disabled state is exposed as `data-disabled` on the trigger. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:74-85`
- Open state is additionally exposed as `data-panel-open` on the trigger (present when open,
  removed when closed). `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:258`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:288`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:294`
- The panel is hidden with the `hidden` attribute while closed-and-mounted; with
  `hiddenUntilFound` the value is `hidden="until-found"`. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:144`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:70`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:1523`
- A `role` on the panel (e.g. `region`), `aria-labelledby` linking, or any `aria-*` on the root are
  never asserted. UNVERIFIED — inferred, no test covers them.

## DOM structure & portal behavior

- Root renders a `div`, Panel renders a `div`, Trigger renders a `button`.
  `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:15`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:49`, `packages/react/src/collapsible/trigger/CollapsibleTrigger.test.tsx:22`
- Closed, non-`keepMounted` panels are removed from the DOM entirely
  (`queryByText(...).toBe(null)`). `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:103`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:279`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:295`
- Closed `keepMounted` panels remain in the DOM, are not visible, carry `hidden` and `data-closed`.
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:92-95`
- Panel data attributes: `data-open` when open (plus transient `data-starting-style` at the start
  of an open transition and `data-ending-style` during a close transition), `data-closed` when
  closed. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:243-244`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:281`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:95`
- The panel sets `--collapsible-panel-height` inline: `'auto'` while open/settled, a resolved `px`
  value during the ending phase (used by height-based transitions/keyframes).
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:274-276`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:280-283`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:531-534`
- The `render` prop can replace the panel's rendered element with an arbitrary component while
  keeping behavior (state passed as second argument). `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:186-196`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:335-341`
- No portal usage: no test renders any part outside the surrounding tree. N/A for portal behavior.
- SSR: a panel rendered open via `renderToString` has its initial keyframe animation suppressed —
  inline `animationName` is overridden to `'none'` while other animation longhands (e.g.
  `animationDuration`) are preserved. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:798-865`

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onOpenChange(nextOpen: boolean, eventDetails)` fires exactly once per trigger activation, in
  both directions (open and close). `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:121`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:355`
- `eventDetails` payload shape: `reason` is `REASONS.triggerPress` for trigger presses,
  `event` is the underlying `MouseEvent`, `isCanceled` is a boolean (initially `false`), and
  `cancel`/`allowPropagation` are functions. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:121-129`
- `cancel()` semantics: calling it in `onOpenChange` prevents the uncontrolled open and the
  uncontrolled close. `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:132-174`
- `allowPropagation()` is part of the details object but its effect is never asserted.
  UNVERIFIED — inferred from `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:129`,
  no test asserts what it does.
- Native `beforematch` event on a `hiddenUntilFound` panel opens the panel: `onOpenChange` is
  called and the panel gets `data-open`. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:1599-1617`
- A `beforematch`-triggered open carries `reason === REASONS.none` in the event details, and
  cancelling it (with reason `none`) keeps the panel closed and does not suppress or alter the next
  trigger-driven open (inline `transitionDuration` stays intact).
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:1541-1597`
- No event-bubbling or `preventDefault` semantics are asserted anywhere (the `beforematch` events
  dispatched by the test helper are `bubbles: true, cancelable: false`, but that is the helper's
  choice). UNVERIFIED — inferred from
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:18-26`, no test asserts bubbling
  or preventDefault behavior of component events.

## Edge cases (rapid interactions, unmount, nesting)

- Close interrupted by reopening: the exit transition still works afterwards — `data-ending-style`
  -> reopen shows `data-open` with `data-starting-style` removed -> closing again reaches
  `data-ending-style` on the same DOM node. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:422-471`
- A close animation finishing after the panel has reopened does not restart the entrance
  transition (panel stays `data-open`, no `data-starting-style`, same DOM node).
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:542-600`
- Zero-size panels (0x0) unmount immediately on close without waiting for unrelated long-running
  transitions (e.g. a 10s opacity transition). `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:286-320`
- A consumer-rendered panel that removes itself while closing is handled: the component does not
  crash, `onOpenChange(false, ...)` still fires, and the content is gone within a frame.
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:322-356`
- A panel that first mounts after the close has already entered the ending phase is unmounted once
  the ending phase passes (no stuck `ending` state). `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:164-208`
- Before measuring an opening panel, inline styles (e.g. `justify-content`) are temporarily
  overridden to `initial !important` and restored afterwards; mixing CSS transitions and CSS
  animations on the panel emits the warning
  `'Base UI: CSS transitions and CSS animations both detected on Collapsible or Accordion panel. Only one of either animation type should be used.'`
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:358-420`
- Measured height is restored before closing transition/keyframe styles apply: the
  `--collapsible-panel-height` var flips from `auto` to a `px` value as `data-ending-style` is
  applied. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:247-284`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:704-747`
- An open animation finishing during a close commit keeps the measured size (height var stays `px`).
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:473-540`
- Initial mount: no open animation runs when rendered open (`animationName: 'none'`,
  zero `getAnimations()`), and later close/reopen still animate.
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:604-702`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:749-795`
- `React.Activity` (React 19): hiding then revealing the collapsible never replays the open
  transition or open keyframes — including inline-style animations and user-opened panels.
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:868-1199`
- `beforematch` opens use a temporary zero animation/transition duration so the reveal is instant;
  the real duration is restored before the next close, and a no-motion beforematch open does not
  suppress a later animated open. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:1205-1347`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:1349-1411`
- An instant (beforematch) open interrupted by an `React.Activity` toggle still restores the inline
  transition duration. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:1413-1472`
- After a `hiddenUntilFound` panel closes, no hidden transition is left running (zero running
  animations, computed opacity at the closed value). `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:1474-1535`
- Combining `hiddenUntilFound` with `keepMounted={false}` warns
  `'Base UI: The `keepMounted={false}` prop on `Collapsible.Panel` is ignored when `hiddenUntilFound` is enabled, since the panel must remain mounted while closed.'`
  and renders `hidden="until-found"`. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:55-74`
- Tests toggle the `globalThis.BASE_UI_ANIMATIONS_DISABLED` runtime flag to force animation
  detection on, restoring it afterwards. `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:474-475`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:536-538`
- Nested collapsibles are not tested. UNVERIFIED — no test covers nesting.

## Shared harness dependencies

- `#test-utils` (repo alias resolving to `packages/react/test/index.ts`) provides
  `describeConformance`, `createRenderer`, and `isJSDOM`; used by the panel and root test files
  (`packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:13`,
  `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:5`). The trigger test imports
  `describeConformance` directly via a relative path
  (`packages/react/src/collapsible/trigger/CollapsibleTrigger.test.tsx:4`).
- `describeConformance` (`packages/react/test/describeConformance.tsx:44-49`) runs four suites —
  prop forwarding, ref forwarding, render prop, and className — for each part with the options the
  test file passes; Trigger additionally passes `testComponentPropWith: 'button'` and
  `button: true` (`packages/react/src/collapsible/trigger/CollapsibleTrigger.test.tsx:23-24`).
- `createRenderer` (`packages/react/test/createRenderer.ts:27-48`) wraps `@mui/internal-test-utils`'s
  renderer inside an awaited `act`; its `render`/`rerender`/`setProps` are promises and it exposes
  the user-event `user` instance. `packages/react/test/createRenderer.ts:31-43`
- `isJSDOM` (re-exported from `@base-ui/utils/testUtils` via `packages/react/test/index.ts:1`) gates
  browser-only suites: the panel's transition/animation/Activity/beforematch suites and the root's
  open-state and keyboard suites are `skipIf(isJSDOM)`.
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:210`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:603`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:868`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:1202`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:1540`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:177`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:346`
- `@mui/internal-test-utils` (external package) provides `render`, `screen`, `user`, `fireEvent`,
  `waitFor`, `flushMicrotasks`, `act`, and `reactMajor`.
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:3-10`, `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:3`, `packages/react/src/collapsible/trigger/CollapsibleTrigger.test.tsx:2`
- `REASONS` is imported from the source tree (`../../internals/reasons`), not a harness; the tests
  only use it to assert `reason` values (`triggerPress`, `none`).
  `packages/react/src/collapsible/root/CollapsibleRoot.test.tsx:6`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:14`
- Panel-test-local helpers: `fireBeforeMatch` (dispatches a bubbling, non-cancelable `beforematch`
  event) and `waitForAnimationFrame`; `useIsoLayoutEffect` from `@base-ui/utils/useIsoLayoutEffect`
  is imported only by a test-fixture component.
  `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:18-34`, `packages/react/src/collapsible/panel/CollapsiblePanel.test.tsx:12`
