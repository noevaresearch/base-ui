# Combobox overlay parts — behavior mined from tests (batch: overlay-parts)

Scope: `Combobox.Popup`, `Combobox.Positioner`, `Combobox.Portal`, `Combobox.Arrow`, `Combobox.Backdrop`, `Combobox.Icon` plus their context modules. The unit's TODO.md entry has no `wraps-external:` field, so there is no external-package delegation to note.

## Public API surface (props, parts, subcomponents)

- Six overlay parts are exercised as first-class subcomponents of `Combobox`: `Combobox.Popup`, `Combobox.Positioner`, `Combobox.Portal`, `Combobox.Arrow`, `Combobox.Backdrop`, `Combobox.Icon`. Each has a conformance suite proving ref forwarding, prop spreading, `render` prop, and `className`: Popup `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:10-21`, Positioner `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:66-75`, Portal `packages/react/src/combobox/portal/ComboboxPortal.test.tsx:9-14`, Arrow `packages/react/src/combobox/arrow/ComboboxArrow.test.tsx:8-19`, Backdrop `packages/react/src/combobox/backdrop/ComboboxBackdrop.test.tsx:8-17`, Icon `packages/react/src/combobox/icon/ComboboxIcon.test.tsx:8-13`.
- Forwarded refs resolve to real DOM elements: Popup, Positioner, Portal, Arrow, and Backdrop render a `HTMLDivElement` (`refInstanceof: window.HTMLDivElement` in each conformance setup; e.g. `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:11`, `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:67`, `packages/react/src/combobox/portal/ComboboxPortal.test.tsx:10`, `packages/react/src/combobox/arrow/ComboboxArrow.test.tsx:9`, `packages/react/src/combobox/backdrop/ComboboxBackdrop.test.tsx:9`), while `Combobox.Icon` renders a `HTMLSpanElement` `packages/react/src/combobox/icon/ComboboxIcon.test.tsx:9`. The conformance runner asserts the attached ref is an instance of that element type `packages/react/test/conformanceTests/refForwarding.tsx:32-38`.
- Every part forwards arbitrary DOM props (`lang`, `data-*`), `style`, and `className` to its rendered element, both on the default element and through a function- or element-valued `render` prop `packages/react/test/conformanceTests/propForwarding.tsx:23-95`, `packages/react/test/conformanceTests/className.tsx:20-23`.
- The `render` prop accepts a function or a JSX element; refs from both the component and the custom element are merged, and classNames from component and render element are merged `packages/react/test/conformanceTests/renderProp.tsx:41-178`.
- `Combobox.Popup` props proven: `initialFocus` (passing `false` disables moving focus into the popup on open) `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:100-120`; `finalFocus` (a ref to an element that receives focus when the popup closes) `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:122-152`.
- `Combobox.Positioner` props proven: `side` (e.g. `side="bottom"`) `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:90`; `sideOffset`, which accepts either a number or a function receiving a positioning-data object that includes `anchor` with a `width` property `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:130-134`, `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:164-168`.
- Structural context rules: `Combobox.Popup` and `Combobox.Arrow` must be used within `Combobox.Positioner`; a consumer of the positioner context outside the positioner rejects rendering with `Base UI: <Combobox.Popup> and <Combobox.Arrow> must be used within the <Combobox.Positioner> component` `packages/react/src/combobox/positioner/ComboboxPositionerContext.test.tsx:17-19`. A consumer of the portal context outside `Combobox.Portal` rejects with `Base UI: <Combobox.Portal> is missing.` `packages/react/src/combobox/portal/ComboboxPortalContext.test.tsx:17`.

## State model (controlled/uncontrolled, defaults, transitions)

- Overlay mount is driven by the Root's open state: `Combobox.Root defaultOpen` mounts the popup `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:23-37`, and controlled `open` mounts the overlay in the conformance setups `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:14-18`, `packages/react/src/combobox/portal/ComboboxPortal.test.tsx:12`, `packages/react/src/combobox/icon/ComboboxIcon.test.tsx:11`.
- The popup element carries a `data-open` attribute when open `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:35-36`.
- Pressing Escape closes an open popup (proven indirectly: after `{Escape}` the popup is gone and focus moved to the final-focus element) `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:146-151`.
- The Positioner participates in a mount path even while the Root is closed: with `Combobox.Root multiple value={[]}` (neither `open` nor `defaultOpen`), a post-mount re-render still runs the positioner's mount logic `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:28-34`, `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:45-48`; the proven requirement from that path is only that scroll locking must not engage (see Edge cases).
- `keepMounted` behavior (popup kept mounted but hidden when closed) is UNVERIFIED — inferred from `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:45-48`, no test asserts this.
- Default initial-focus target when opened by a non-touch pointer is UNVERIFIED — inferred from `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:100-120`, no test asserts this (this batch only proves the touch case and the `initialFocus={false}` opt-out).

## Keyboard interactions

- Escape while the popup is open closes it and triggers final-focus restoration to the element provided via `finalFocus` `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:147-151`.
- No other keyboard interactions are asserted in this batch (arrow-key navigation, Home/End, typing-to-filter, etc. are not covered here).

## Focus management

- When the combobox is opened via a touch pointerdown on the Trigger, focus moves to the popup element itself and explicitly NOT to the `Combobox.Input` inside it `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:90-98`.
- `initialFocus={false}` on the Popup keeps focus on the trigger after a click-open; the input renders but never receives focus `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:114-119`.
- `finalFocus` accepts a React ref; on close (Escape), the referenced element receives focus `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:147-151`.
- Backdrop unmount-on-close behavior is UNVERIFIED — inferred from `packages/react/src/combobox/backdrop/ComboboxBackdrop.test.tsx:12-16`, no test asserts this (its conformance suite only renders with `defaultOpen`).

## Accessibility (roles, aria-*, id linking)

- The popup's role depends on where the Input renders: when the `Combobox.Input` is OUTSIDE the popup, the popup gets `role="presentation"` `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:51-54`; when the `Combobox.Input` is rendered INSIDE the popup, the popup gets `role="dialog"` `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:70-73`.
- The positioner exposes the resolved placement as a `data-side` attribute (`data-side="bottom"` when `side="bottom"` is honored) `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:112-114`.
- The popup exposes open state via `data-open` (state-attribute mapping) `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:23-37`.
- aria-* id linking between input, popup, and listbox is not asserted in this batch.

## DOM structure & portal behavior

- `Combobox.Portal` renders a real portal container element: its forwarded ref is a `HTMLDivElement` and arbitrary props/`render`/`className` apply to it `packages/react/src/combobox/portal/ComboboxPortal.test.tsx:9-14`.
- Popup, Positioner, Arrow, and Backdrop each render a `div` (see element-type citations in Public API surface: `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:11`, `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:67`, `packages/react/src/combobox/arrow/ComboboxArrow.test.tsx:9`, `packages/react/src/combobox/backdrop/ComboboxBackdrop.test.tsx:9`); `Combobox.Icon` renders a `span` `packages/react/src/combobox/icon/ComboboxIcon.test.tsx:9`.
- The popup is structured as Portal → Positioner → Popup in every overlay test setup `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:14-18`, and the Positioner context enforces that Popup and Arrow live inside the Positioner `packages/react/src/combobox/positioner/ComboboxPositionerContext.test.tsx:17-19`.
- The Positioner sets `data-side` on its element to the resolved side `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:112-114`.
- The Positioner exposes an `--available-height` CSS variable for consumers; the Chromium test caps a List with `maxHeight: min(80px, var(--available-height))` `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:93`, and the popup stays on the preferred side only because the variable is seeded before sizing runs (otherwise the first flip pass would measure the list at full height and flip it) `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:108-114`.
- Default anchor resolution: with no InputGroup, the popup anchors to the `Combobox.Input` — the `sideOffset` callback observes `anchor.width` equal to the input's width (120) and not the trigger's width (240) `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:146-149`. With an `InputGroup` wrapping Input and Trigger, the popup anchors to the group — `anchor.width` equals the group width (240) `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:180-182`.

## Events (names, payload shape, bubbling, preventDefault semantics)

- No DOM or lifecycle events (e.g. `onOpenChange`, open/close change events, bubbling, or `preventDefault` semantics) are asserted anywhere in this batch — N/A beyond the following.
- `sideOffset` may be a function receiving a positioning-data object that includes `anchor` (with `width`); it is a positioning callback, not an event, and its return value is used as the offset `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:130-134`.
- The touch-open path is driven by a `pointerdown` with `pointerType: 'touch'` followed by `mousedown` on the Trigger, which is enough to open the overlay and trigger the popup-focus behavior `packages/react/src/combobox/popup/ComboboxPopup.test.tsx:91-92`.

## Edge cases (rapid interactions, unmount, nesting)

- Scroll-lock regression guard: a closed, controlled `Combobox.Root multiple value={[]}` plus a post-mount re-render (rendered outside `act()` so the initial render and the effect re-render are separate commits, matching real browser behavior `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:12-14`) must never set `overflow: hidden` on `body` or `documentElement` (both axes) `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:60-63`. The historical bug: the re-render force-mounts the Positioner whose `open` is `undefined` rather than `false`, making `open && modal` evaluate to `undefined` and defaulting the scroll lock to enabled `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:45-48`.
- Flip-suppression edge case (Chromium only, skipped in jsdom): with the anchor fixed near the bottom of the viewport (less room below than above) and a 40-item list capped to `min(80px, var(--available-height))`, the popup must stay on the preferred `side="bottom"` instead of flipping up `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:78-116`.
- Misuse/nesting errors are descriptive runtime errors, asserted as render rejections for both the positioner context (`...must be used within the <Combobox.Positioner> component`) and the portal context (`<Combobox.Portal> is missing.`) `packages/react/src/combobox/positioner/ComboboxPositionerContext.test.tsx:17-19`, `packages/react/src/combobox/portal/ComboboxPortalContext.test.tsx:17`.
- Scroll-lock cleanup on unmount is not asserted: the positioner test captures the overflow styles before `root.unmount()` and only cleans the DOM afterwards, so UNVERIFIED — inferred from `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:54-57`, no test asserts this.
- Rapid interactions (double-open, fast toggle) are not covered in this batch.

## Shared harness dependencies

- `packages/react/test/index.ts` — the `#test-utils` barrel; exports `createRenderer`, `describeConformance`, and re-exports `isJSDOM` (from `@base-ui/utils/testUtils`) used for the `skipIf(isJSDOM)` Chromium-only tests `packages/react/test/index.ts:1-11`.
- `packages/react/test/createRenderer.ts` — wraps rendering in `act` and adds `rerender`/`setProps` helpers on the render result `packages/react/test/createRenderer.ts:27-49`.
- `packages/react/test/describeConformance.tsx` — runs the conformance suite composed of `propsSpread`, `refForwarding`, `renderProp`, and `className` tests, filtered by `only`/`skip` `packages/react/test/describeConformance.tsx:44-67`.
- `packages/react/test/conformanceTests/refForwarding.tsx` — asserts the attached ref is an instance of `refInstanceof` `packages/react/test/conformanceTests/refForwarding.tsx:27-40`.
- `packages/react/test/conformanceTests/propForwarding.tsx` — asserts `lang`/`data-*`/`style` forwarding to the default element and through function- and element-valued `render` props `packages/react/test/conformanceTests/propForwarding.tsx:22-128`.
- `packages/react/test/conformanceTests/renderProp.tsx` — asserts function/element `render` customization, ref merging, and className merging `packages/react/test/conformanceTests/renderProp.tsx:40-179`.
- `packages/react/test/conformanceTests/className.tsx` — asserts string `className` is applied `packages/react/test/conformanceTests/className.tsx:13-24`.
- The tests also use `@mui/internal-test-utils` (`fireEvent`, `screen`, `waitFor`, `flushMicrotasks`, `randomStringValue`); this is an external package dependency, not a repo harness file, and was not read.
