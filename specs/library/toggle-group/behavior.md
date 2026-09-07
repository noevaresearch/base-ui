# toggle-group — behavior spec

Mined from this unit's own test files only:

- `packages/react/src/toggle-group/ToggleGroup.test.tsx` (605 lines)

The unit's `TODO.md` entry (`library: toggle-group`, `TODO.md:548-553`) has no `wraps-external:` field and no `needs-batched-mining` flag, so behavior is derived entirely from this single test file; no third-party delegation applies.

## Public API surface (props, parts, subcomponents)

- `ToggleGroup` renders a `div` as its root element and forwards the ref to it (`refInstanceof: window.HTMLDivElement`): `packages/react/src/toggle-group/ToggleGroup.test.tsx:13-16`.
- The root is exposed as `role="group"` and named via `aria-label` (`screen.queryByRole('group', { name: 'My Toggle Group' })` resolves): `packages/react/src/toggle-group/ToggleGroup.test.tsx:18-22`.
- Observed props proven by tests: `value` (controlled, array): `packages/react/src/toggle-group/ToggleGroup.test.tsx:116-163`; `defaultValue` (uncontrolled): `packages/react/src/toggle-group/ToggleGroup.test.tsx:55-74`; `disabled` (group-level): `packages/react/src/toggle-group/ToggleGroup.test.tsx:166-181`; `orientation` (`'horizontal' | 'vertical'`): `packages/react/src/toggle-group/ToggleGroup.test.tsx:200-223`; `multiple`: `packages/react/src/toggle-group/ToggleGroup.test.tsx:226-306`; `onValueChange`: `packages/react/src/toggle-group/ToggleGroup.test.tsx:520-603`.
- Arbitrary HTML attributes pass through to the root: conformance asserts `lang` and a custom `data-foobar` land on the root element: `packages/react/test/conformanceTests/propForwarding.tsx:23-36` (enabled for ToggleGroup via `packages/react/src/toggle-group/ToggleGroup.test.tsx:13-16`, which passes no `skip` list, so the full conformance suite runs per `packages/react/test/describeConformance.tsx:44-55`).
- `render` prop customization is supported in both function and element forms: `packages/react/test/conformanceTests/renderProp.tsx:41-59`, `packages/react/test/conformanceTests/renderProp.tsx:61-76`. The custom element receives the merged ref: `packages/react/test/conformanceTests/renderProp.tsx:93-113`, `packages/react/test/conformanceTests/renderProp.tsx:115-144`. A `className` string is applied to the root: `packages/react/test/conformanceTests/className.tsx:20-23`; a function `className` is resolved and merged with the render-prop element's `className`: `packages/react/test/conformanceTests/renderProp.tsx:163-178`.
- Children are `Toggle` components (imported from `@base-ui/react/toggle`) identified by their `value` prop: `packages/react/src/toggle-group/ToggleGroup.test.tsx:5`, `packages/react/src/toggle-group/ToggleGroup.test.tsx:31-34`. A `Toggle` rendered with no explicit `value` (or `value=""`) still participates as its own item: `packages/react/src/toggle-group/ToggleGroup.test.tsx:76-96`.
- The group can be nested inside `Toolbar.Root` / `Toolbar.Group` and keeps working there: `packages/react/src/toggle-group/ToggleGroup.test.tsx:313-329`.
- The tests also exercise `DirectionProvider` for RTL keyboard mapping: `packages/react/src/toggle-group/ToggleGroup.test.tsx:381-388`.
- No subcomponents are exported by the group itself in these tests (children come from the separate `toggle` unit). UNVERIFIED — inferred from `packages/react/src/toggle-group/ToggleGroup.test.tsx:1-8` (imports), no test asserts a subcomponent export list.

## State model (controlled/uncontrolled, defaults, transitions)

- Uncontrolled default: no items pressed — every `Toggle` starts with `aria-pressed="false"`: `packages/react/src/toggle-group/ToggleGroup.test.tsx:37-40`.
- A pointer click presses an item (`aria-pressed="true"` plus a `data-pressed` attribute): `packages/react/src/toggle-group/ToggleGroup.test.tsx:42-46`. Default is single selection: pressing a second item unpresses the first: `packages/react/src/toggle-group/ToggleGroup.test.tsx:48-52`.
- `defaultValue={['two']}` marks that item pressed at mount: `packages/react/src/toggle-group/ToggleGroup.test.tsx:63-67`; a later click replaces it under single selection: `packages/react/src/toggle-group/ToggleGroup.test.tsx:69-73`.
- Controlled: `value={['two']}` mirrors the prop at mount: `packages/react/src/toggle-group/ToggleGroup.test.tsx:125-129`; external prop updates (`setProps({ value: [...] })`) drive the pressed state on every render: `packages/react/src/toggle-group/ToggleGroup.test.tsx:131-141`, `packages/react/src/toggle-group/ToggleGroup.test.tsx:144-163`. The controlled describe contains no click interactions, so whether clicks are ignored or only overridden in controlled mode is: UNVERIFIED — inferred from `packages/react/src/toggle-group/ToggleGroup.test.tsx:116-163`, no test asserts this.
- `multiple={true}` lets items accumulate: pressing a second item keeps the first pressed: `packages/react/src/toggle-group/ToggleGroup.test.tsx:244-261`. With `multiple` false (default) only one item stays pressed: `packages/react/src/toggle-group/ToggleGroup.test.tsx:263-280`. In multiple mode, clicking a pressed item unpresses only that item: `packages/react/src/toggle-group/ToggleGroup.test.tsx:303-305`.
- Switching `multiple` at runtime (`false → true → false`) preserves the current selection and continues from it: after the switch to multiple, both items end up pressed; after switching back, a click reverts to single selection: `packages/react/src/toggle-group/ToggleGroup.test.tsx:346-362`.
- `data-multiple` on the group reflects the prop exactly — absent when false/unset, present when true, and it updates live with `setProps`: `packages/react/src/toggle-group/ToggleGroup.test.tsx:227-242`.
- Group-level `disabled` marks every child with `aria-disabled="true"` and `data-disabled`: `packages/react/src/toggle-group/ToggleGroup.test.tsx:175-180`. Item-level `disabled` does the same for that item, while enabled items render an explicit `aria-disabled="false"` and no `data-disabled`: `packages/react/src/toggle-group/ToggleGroup.test.tsx:191-196`. No test asserts that clicks or keys on disabled items are ignored: UNVERIFIED — inferred from `packages/react/src/toggle-group/ToggleGroup.test.tsx:166-197`, no test asserts this.
- Dev warning: a `Toggle` without an explicit `value` rendered inside a group whose value is defined (`defaultValue={['one']}`) triggers exactly one `console.error` with the message starting `Base UI: A `<Toggle>` component rendered in a `<ToggleGroup>` has no explicit `value` prop...`: `packages/react/src/toggle-group/ToggleGroup.test.tsx:98-113`.
- Cancellation: if `onValueChange` calls `eventDetails.cancel()`, the handler still fires but the pressed state does not change (`aria-pressed` stays `'false'`): `packages/react/src/toggle-group/ToggleGroup.test.tsx:546-564`.

## Keyboard interactions

Chromium-only suite (`describe.skipIf(isJSDOM)`): `packages/react/src/toggle-group/ToggleGroup.test.tsx:369`.

- Roving-tabindex arrow navigation with wrap-around, parameterized by `DirectionProvider` direction × group `orientation` (matrix defined at `packages/react/src/toggle-group/ToggleGroup.test.tsx:370-375`):
  - `ltr` + `horizontal`: ArrowRight moves to the next item, ArrowLeft to the previous: `packages/react/src/toggle-group/ToggleGroup.test.tsx:371`, `packages/react/src/toggle-group/ToggleGroup.test.tsx:397-421`.
  - `ltr` + `vertical`: ArrowDown next, ArrowUp previous: `packages/react/src/toggle-group/ToggleGroup.test.tsx:372`.
  - `rtl` + `horizontal`: ArrowLeft next, ArrowRight previous: `packages/react/src/toggle-group/ToggleGroup.test.tsx:373`.
  - `rtl` + `vertical`: ArrowDown next, ArrowUp previous (horizontal arrows ignored): `packages/react/src/toggle-group/ToggleGroup.test.tsx:374`.
  - Arrow-forward from the last item wraps focus to the first: `packages/react/src/toggle-group/ToggleGroup.test.tsx:407-411`; arrow-back from the first wraps to the last: `packages/react/src/toggle-group/ToggleGroup.test.tsx:413-416`.
  - Keys from the other axis (e.g. ArrowDown in a horizontal group) do not move focus: `packages/react/src/toggle-group/ToggleGroup.test.tsx:424-432`.
- `Home` moves focus to the first item (which takes `tabindex="0"`) and navigation continues from there: `packages/react/src/toggle-group/ToggleGroup.test.tsx:437-464`. `End` moves focus to the last item: `packages/react/src/toggle-group/ToggleGroup.test.tsx:466-490`.
- `Enter` and `Space` toggle the pressed state of the focused item (`aria-pressed` flips `'false' → 'true' → 'false'`): `packages/react/src/toggle-group/ToggleGroup.test.tsx:492-517`.
- The arrow-navigation assertions only ever assert focus and `tabindex`; whether arrow keys also change the pressed value is: UNVERIFIED — inferred from `packages/react/src/toggle-group/ToggleGroup.test.tsx:379-433`, no test asserts this.

## Focus management

- `Tab` into the group focuses the first item, which carries `tabindex="0"`: `packages/react/src/toggle-group/ToggleGroup.test.tsx:392-395`.
- Roving tabindex: as focus moves, `tabindex="0"` moves with it — each newly focused item is asserted to have `tabindex="0"` while remaining the focused element: `packages/react/src/toggle-group/ToggleGroup.test.tsx:397-432`, `packages/react/src/toggle-group/ToggleGroup.test.tsx:455-456`, `packages/react/src/toggle-group/ToggleGroup.test.tsx:481-482`.
- Focus wraps at both ends of the group (last → first on next, first → last on previous): `packages/react/src/toggle-group/ToggleGroup.test.tsx:407-416`.
- Programmatic `button.focus()` puts the group in a state where `Enter`/`Space` activate that item: `packages/react/src/toggle-group/ToggleGroup.test.tsx:505-515`, and the activation fires `onValueChange` for that item: `packages/react/src/toggle-group/ToggleGroup.test.tsx:585-601`.
- Roving focus survives `multiple` prop transitions: focus moved with ArrowRight stays put through `setProps({ multiple })` changes and subsequent arrow navigation continues correctly (`ArrowLeft` returns to the first item): `packages/react/src/toggle-group/ToggleGroup.test.tsx:339-365`.

## Accessibility (roles, aria-*, id linking)

- Group root: `role="group"`, nameable via `aria-label`: `packages/react/src/toggle-group/ToggleGroup.test.tsx:18-22`.
- `aria-orientation` is intentionally NOT rendered on the `role="group"` root, even when `orientation="horizontal"` is set: `packages/react/src/toggle-group/ToggleGroup.test.tsx:213-223`; orientation is exposed via `data-orientation` instead (see DOM section).
- Each `Toggle` exposes `aria-pressed` as `'true'`/`'false'` reflecting its membership in the group value: `packages/react/src/toggle-group/ToggleGroup.test.tsx:39-52`, `packages/react/src/toggle-group/ToggleGroup.test.tsx:125-141`.
- Disabled state is exposed as an explicit `aria-disabled` of `'true'` or `'false'` on each toggle, with `data-disabled` only when disabled: `packages/react/src/toggle-group/ToggleGroup.test.tsx:177-180`, `packages/react/src/toggle-group/ToggleGroup.test.tsx:193-196`.
- Id linking (`aria-labelledby`, `aria-controls`, `aria-describedby`): N/A — no id-linking behavior is asserted anywhere in this suite.

## DOM structure & portal behavior

- Root is a single `div` (ref resolves to `window.HTMLDivElement`): `packages/react/src/toggle-group/ToggleGroup.test.tsx:13-16`.
- Group data attributes: `data-orientation="vertical"` when `orientation="vertical"` is set: `packages/react/src/toggle-group/ToggleGroup.test.tsx:209-211`; `data-multiple` present only when `multiple` is true: `packages/react/src/toggle-group/ToggleGroup.test.tsx:235-241`.
- Toggle data attributes: `data-pressed` on pressed items: `packages/react/src/toggle-group/ToggleGroup.test.tsx:44-45`, `packages/react/src/toggle-group/ToggleGroup.test.tsx:50-51`; `data-disabled` on disabled items (group-level or item-level): `packages/react/src/toggle-group/ToggleGroup.test.tsx:178-180`, `packages/react/src/toggle-group/ToggleGroup.test.tsx:195-196`.
- Custom props (`lang`, `data-foobar`, `style`, `className`) all land on the rendered root, including through `render` customization: `packages/react/test/conformanceTests/propForwarding.tsx:23-95`, `packages/react/test/conformanceTests/className.tsx:20-23`.
- The group renders and behaves correctly when wrapped in `Toolbar.Root` > `Toolbar.Group` (selection + roving focus verified there): `packages/react/src/toggle-group/ToggleGroup.test.tsx:322-325`, `packages/react/src/toggle-group/ToggleGroup.test.tsx:339-365`.
- Portal behavior: N/A — no test in this suite renders the group or its toggles through a portal. UNVERIFIED — inferred from `packages/react/src/toggle-group/ToggleGroup.test.tsx` (absence of any portal test), no test asserts this.

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onValueChange(nextValue, eventDetails)` is the only callback exercised. It fires once per committed interaction, and the first argument is the full next value array (single mode: `['one']` then `['two']` on successive clicks): `packages/react/src/toggle-group/ToggleGroup.test.tsx:533-543`. It is not called before any interaction: `packages/react/src/toggle-group/ToggleGroup.test.tsx:533`.
- Pointer clicks trigger it: `user.pointer({ keys: '[MouseLeft]' })` on a toggle: `packages/react/src/toggle-group/ToggleGroup.test.tsx:535-543`.
- `Enter` and `Space` on a focused toggle trigger it with that toggle's value (Chromium-only): `packages/react/src/toggle-group/ToggleGroup.test.tsx:566-603`.
- Cancellation semantics: calling `eventDetails.cancel()` inside `onValueChange` still invokes the handler exactly once, but the value does not change (`aria-pressed` stays `'false'`): `packages/react/src/toggle-group/ToggleGroup.test.tsx:546-564`. This is the tested cancellation mechanism; native `preventDefault` on a DOM event is never exercised: UNVERIFIED — inferred from `packages/react/src/toggle-group/ToggleGroup.test.tsx` (no preventDefault test), no test asserts this.
- Bubbling: N/A — no test asserts bubbling of any native event through the group root.

## Edge cases (rapid interactions, unmount, nesting)

- Toggles omitting `value` (one with no prop, one with `value=""`) remain independently toggleable; in single mode clicking one unpresses the other: `packages/react/src/toggle-group/ToggleGroup.test.tsx:76-96`. In multiple mode both can be pressed simultaneously: `packages/react/src/toggle-group/ToggleGroup.test.tsx:282-306`.
- A value-less `Toggle` combined with a defined group value produces a single dev warning (see State model): `packages/react/src/toggle-group/ToggleGroup.test.tsx:98-113`.
- Runtime `multiple` transitions (`false → true → false`) preserve the selection set and roving focus, in both standalone groups and groups nested in `Toolbar.Group`: `packages/react/src/toggle-group/ToggleGroup.test.tsx:309-367`.
- Rapid interactions: no dedicated rapid-fire/stress test exists; sequential clicks are the only covered cadence: UNVERIFIED — inferred from `packages/react/src/toggle-group/ToggleGroup.test.tsx` (absence of such a test), no test asserts this.
- Unmount: no test unmounts toggles or the group mid-interaction: UNVERIFIED — inferred from `packages/react/src/toggle-group/ToggleGroup.test.tsx` (absence of such a test), no test asserts this.
- Nesting: `Toolbar.Group` nesting is covered (see DOM structure); nesting a `ToggleGroup` inside another `ToggleGroup` and React StrictMode double-render are not covered: UNVERIFIED — inferred from `packages/react/src/toggle-group/ToggleGroup.test.tsx` (absence of such tests), no test asserts this.

## Shared harness dependencies

- `#test-utils` maps to `packages/react/test/index.ts`; `ToggleGroup.test.tsx` imports `createRenderer`, `describeConformance`, and `isJSDOM` from it: `packages/react/src/toggle-group/ToggleGroup.test.tsx:7`. The exports come from `packages/react/test/index.ts:3-4`, with `isJSDOM` re-exported through `packages/react/test/index.ts:1`.
  - `createRenderer` — `packages/react/test/createRenderer.ts:27-49`: wraps the shared renderer so `render` is act-wrapped (`packages/react/test/createRenderer.ts:31-43`) and adds a `setProps` helper that re-renders the same element with new props (`packages/react/test/createRenderer.ts:38-40`); this is how the controlled-state tests above drive prop updates.
  - `describeConformance` — `packages/react/test/describeConformance.tsx:51-68`: runs the "Base UI component API" suite composed of `propsSpread`, `refForwarding`, `renderProp`, and `className` tests (`packages/react/test/describeConformance.tsx:44-49`); `ToggleGroup.test.tsx` invokes it with only `refInstanceof: window.HTMLDivElement` and `render` (`packages/react/src/toggle-group/ToggleGroup.test.tsx:13-16`), so all four conformance suites run and prove the prop-forwarding/ref/render/className claims in the Public API section above.
  - `isJSDOM` — `packages/utils/src/testUtils.ts:4`: `const isJSDOM = /jsdom/.test(window.navigator.userAgent)`. It gates three suites in this file: the uncontrolled pointer-press test (`packages/react/src/toggle-group/ToggleGroup.test.tsx:25-28`), the `multiple transitions` describe (`packages/react/src/toggle-group/ToggleGroup.test.tsx:309`), the keyboard-interactions describe (`packages/react/src/toggle-group/ToggleGroup.test.tsx:369`), and the `Enter`/`Space` `onValueChange` tests (`packages/react/src/toggle-group/ToggleGroup.test.tsx:567-569`).
- `@mui/internal-test-utils` (external npm package, not a repo file): supplies `act` and `screen` directly (`packages/react/src/toggle-group/ToggleGroup.test.tsx:2`) and the underlying renderer/user-event plumbing used by `createRenderer` (`packages/react/test/createRenderer.ts:2-9`).
