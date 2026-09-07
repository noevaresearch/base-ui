# radio-group — behavior spec

Mined from this unit's own test files only:

- `packages/react/src/radio-group/RadioGroup.test.tsx` (1990 lines, the unit's entire suite)

The unit's `TODO.md` entry (`library: radio-group`, TODO.md:483-489) has no `wraps-external:` field, so behavior is derived entirely from this suite; no third-party delegation applies. The suite is a single file, so no batched mining was needed.

## Public API surface (props, parts, subcomponents)

- `RadioGroup` renders a `div` as its root element and forwards the ref to it (`refInstanceof: window.HTMLDivElement`): `packages/react/src/radio-group/RadioGroup.test.tsx:17-20`.
- Arbitrary extra props are applied to the root and can override built-in attributes (e.g. `role="switch"` replaces the default `radiogroup` role): `packages/react/src/radio-group/RadioGroup.test.tsx:22-27`.
- Observed props proven by tests: `id` (forwarded to root): `packages/react/src/radio-group/RadioGroup.test.tsx:29-35`; `onValueChange`: `packages/react/src/radio-group/RadioGroup.test.tsx:38-52`; `defaultValue` (uncontrolled): `packages/react/src/radio-group/RadioGroup.test.tsx:853-866`; `value` (controlled): `packages/react/src/radio-group/RadioGroup.test.tsx:468-497`; `disabled` (group-level): `packages/react/src/radio-group/RadioGroup.test.tsx:199-210`; `readOnly`: `packages/react/src/radio-group/RadioGroup.test.tsx:235-239`; `required`: `packages/react/src/radio-group/RadioGroup.test.tsx:294-296`; `name` (propagated to each hidden radio input): `packages/react/src/radio-group/RadioGroup.test.tsx:309-320`; `inputRef` (ref/function to the checked radio's hidden input): `packages/react/src/radio-group/RadioGroup.test.tsx:322-342`; `form` (native form association): `packages/react/src/radio-group/RadioGroup.test.tsx:1370-1393`.
- The `value` prop is NOT forwarded to the root DOM element as an attribute: `packages/react/src/radio-group/RadioGroup.test.tsx:843-851`.
- Children are `Radio.Root` items (each with a string `value`), optionally containing a `Radio.Indicator`: `packages/react/src/radio-group/RadioGroup.test.tsx:281-307`. No subcomponents are exported by the group itself; the group works purely through context with `Radio.Root`/`Radio.Indicator` children. UNVERIFIED — inferred from `packages/react/src/radio-group/RadioGroup.test.tsx:1-13` (imports), no test asserts a subcomponent export list.
- Integrates with `Field.Root`/`Field.Item`/`Field.Label`/`Field.Description`/`Field.Error`, `Fieldset.Root`/`Fieldset.Legend`, `Form`, and `DirectionProvider` (dedicated describes for each): `packages/react/src/radio-group/RadioGroup.test.tsx:928-1220`, `packages/react/src/radio-group/RadioGroup.test.tsx:1222-1359`, `packages/react/src/radio-group/RadioGroup.test.tsx:1361-1989`, `packages/react/src/radio-group/RadioGroup.test.tsx:652-762`.

## State model (controlled/uncontrolled, defaults, transitions)

- Value is a single `string` (the selected radio's value) or `null` when nothing is selected. Controlled via `value` + `onValueChange`; uncontrolled via `defaultValue`:
  - `defaultValue` sets the initial selection (a `value="b"` child renders checked and owns the tab stop): `packages/react/src/radio-group/RadioGroup.test.tsx:853-866`.
  - Controlled `value` drives `aria-checked` on children and can be changed externally (via a button setting state): `packages/react/src/radio-group/RadioGroup.test.tsx:984-1025`.
- Default with no `defaultValue`: no radio is checked initially (`aria-checked="false"` on all children): `packages/react/src/radio-group/RadioGroup.test.tsx:640-641`.
- Clicking a radio selects it and calls `onValueChange` with that value: `packages/react/src/radio-group/RadioGroup.test.tsx:38-52`. Selection is exclusive: selecting `b` unchecks `a` (`data-checked`/`data-unchecked` swap on root and indicator): `packages/react/src/radio-group/RadioGroup.test.tsx:794-840`. Clicking the already-selected radio keeps it selected (no unselect): `packages/react/src/radio-group/RadioGroup.test.tsx:833-839`.
- Clicking a radio's hidden `<input type="radio">` directly toggles the underlying state (group state updates from input change): `packages/react/src/radio-group/RadioGroup.test.tsx:264-279`.
- Arrow-key navigation moves focus AND automatically selects the newly focused radio: `packages/react/src/radio-group/RadioGroup.test.tsx:622-650`.
- Cancellation: `onValueChange` receives `eventDetails` whose `cancel()` rejects the change. Verified for three interaction paths — a radio click: `packages/react/src/radio-group/RadioGroup.test.tsx:117-137`; a hidden-input click: `packages/react/src/radio-group/RadioGroup.test.tsx:139-164`; arrow-key navigation (focus still moves, but neither radio becomes checked): `packages/react/src/radio-group/RadioGroup.test.tsx:166-195`. A canceled interaction also leaves the Field state hooks untouched (no `data-touched`/`data-dirty`/`data-filled`): `packages/react/src/radio-group/RadioGroup.test.tsx:134-136`, `packages/react/src/radio-group/RadioGroup.test.tsx:192-194`.
- A successful arrow-key interaction marks the group touched (`data-touched` appears on the root): `packages/react/src/radio-group/RadioGroup.test.tsx:649`.
- Group-level `disabled`: `aria-disabled="true"` on the group, `aria-disabled="true"`/`data-disabled` on each radio, and `disabled` on each hidden input; clicks do not change state: `packages/react/src/radio-group/RadioGroup.test.tsx:199-210`, `packages/react/src/radio-group/RadioGroup.test.tsx:217-231`. When `disabled` is unset the `aria-disabled` attribute is absent: `packages/react/src/radio-group/RadioGroup.test.tsx:212-215`. `Field.Root disabled` propagates the same way: `packages/react/src/radio-group/RadioGroup.test.tsx:947-965`.
- Group-level `readOnly`: `aria-readonly="true"` on the group (absent when unset): `packages/react/src/radio-group/RadioGroup.test.tsx:235-245`; clicks do not change state: `packages/react/src/radio-group/RadioGroup.test.tsx:247-261`.
- Field revalidation: when the controlled value changes externally (not via user interaction), the owning Field re-runs its validator with the new value: `packages/react/src/radio-group/RadioGroup.test.tsx:984-1025`.

## Keyboard interactions

- Space selects the focused radio on keydown-release boundary — the change fires only on keyUP, not while the key is held: `packages/react/src/radio-group/RadioGroup.test.tsx:73-95`.
- Enter does NOT select: no `onValueChange` call and the item keeps `aria-checked="false"`: `packages/react/src/radio-group/RadioGroup.test.tsx:97-115`.
- Arrow keys move focus within the group and select the newly focused radio: `packages/react/src/radio-group/RadioGroup.test.tsx:622-650`.
- `ArrowDown`/`ArrowUp` cycle through items with wrap-around (a → b → c → a → …): `packages/react/src/radio-group/RadioGroup.test.tsx:684-706`.
- Horizontal arrows are direction-aware: in LTR `ArrowRight` = next and `ArrowLeft` = previous; in RTL these are swapped (`ArrowLeft` = next): `packages/react/src/radio-group/RadioGroup.test.tsx:653-655`, `packages/react/src/radio-group/RadioGroup.test.tsx:708-714`.
- Modifier keys do not block navigation: with Shift held, arrow keys still move focus normally: `packages/react/src/radio-group/RadioGroup.test.tsx:734-759`.
- Clicking a radio while Shift is held still selects it; the modifier state is reported on the change event details (`eventDetails.event.shiftKey === true`): `packages/react/src/radio-group/RadioGroup.test.tsx:54-71`.

## Focus management

- Roving tabindex: the checked radio owns `tabindex="0"` initially (`defaultValue="b"` → radio-b tabindex 0, radio-a tabindex -1): `packages/react/src/radio-group/RadioGroup.test.tsx:853-866`.
- Arrow navigation moves the tab stop to the newly focused radio: after arrowing from `b` to `c`, `c` has `tabindex="0"` and `a` has `tabindex="-1"`: `packages/react/src/radio-group/RadioGroup.test.tsx:782-789`.
- Tab into the group lands on the tab stop; Tab from a radio exits the group to the next element; Shift+Tab returns to the current tab stop (not necessarily the last-focused item): `packages/react/src/radio-group/RadioGroup.test.tsx:716-731`.
- Item removal: when the highlighted (tab-stop) radio unmounts, the tab stop moves to the checked radio (`value="b"` group, `c` highlighted and removed → `b` tabindex 0, `a` tabindex -1): `packages/react/src/radio-group/RadioGroup.test.tsx:764-790`.
- Form-error focus: submitting a form with an empty required group focuses the first enabled radio: `packages/react/src/radio-group/RadioGroup.test.tsx:1865-1894` (all radios start disabled, get enabled, submit → first radio focused); also after an ancestor `<fieldset disabled>` is enabled: `packages/react/src/radio-group/RadioGroup.test.tsx:1896-1929` (Chromium-only).
- `inputRef` semantics (the group exposes the selected radio's hidden input):
  - Points to the checked radio's hidden input and updates when selection moves: `packages/react/src/radio-group/RadioGroup.test.tsx:322-342`.
  - Usable in a layout effect during the same commit (observes the initial value): `packages/react/src/radio-group/RadioGroup.test.tsx:344-365`.
  - Supports function refs (called with each input as selection moves, last call = current): `packages/react/src/radio-group/RadioGroup.test.tsx:367-387`.
  - A stable callback ref is not detached/re-attached on unrelated parent re-renders: `packages/react/src/radio-group/RadioGroup.test.tsx:389-417`.
  - Disabled radios are skipped when assigning the ref (ref points to the first enabled radio's input): `packages/react/src/radio-group/RadioGroup.test.tsx:419-433`.
  - With `nativeButton` rendering a `<button>` inside a `<label>`, the ref points to the first radio input: `packages/react/src/radio-group/RadioGroup.test.tsx:435-466`.
  - Clearing the controlled value to `null` keeps the ref on the first radio's input (no re-pointing): `packages/react/src/radio-group/RadioGroup.test.tsx:468-497`.
  - The ref is detached (set to `null`) when the radio it currently points to unmounts, whether it was initially checked: `packages/react/src/radio-group/RadioGroup.test.tsx:499-526`, or selected after mount: `packages/react/src/radio-group/RadioGroup.test.tsx:528-558`.
  - The ref becomes available again once an ancestor `<fieldset disabled>` is enabled: `packages/react/src/radio-group/RadioGroup.test.tsx:1223-1250`.

## Accessibility (roles, aria-*, id linking)

- Root: `role="radiogroup"` (default; overridable via extra props): `packages/react/src/radio-group/RadioGroup.test.tsx:33`, `packages/react/src/radio-group/RadioGroup.test.tsx:22-27`.
- Radio items expose `aria-checked` `'true'`/`'false'`: `packages/react/src/radio-group/RadioGroup.test.tsx:114`, `packages/react/src/radio-group/RadioGroup.test.tsx:645-648`.
- `disabled`: `aria-disabled="true"` + `data-disabled` on group and each radio, plus a real `disabled` attribute on the hidden input: `packages/react/src/radio-group/RadioGroup.test.tsx:199-210`. Absent when unset: `packages/react/src/radio-group/RadioGroup.test.tsx:212-215`.
- `readOnly`: `aria-readonly="true"` on the group, absent when unset: `packages/react/src/radio-group/RadioGroup.test.tsx:235-245`.
- Style-hook data attributes on root, item, and indicator: `data-disabled`, `data-readonly`, `data-required` on the group; `data-checked`/`data-unchecked` on items and indicators: `packages/react/src/radio-group/RadioGroup.test.tsx:281-307`, `packages/react/src/radio-group/RadioGroup.test.tsx:794-840`.
- Native `<label>` association: a `<label>` wrapping `Radio.Root` selects that radio when clicked; an explicit `htmlFor` label behaves the same: `packages/react/src/radio-group/RadioGroup.test.tsx:868-894`, `packages/react/src/radio-group/RadioGroup.test.tsx:896-925`.
- `Field.Label` association: implicit (label wrapping the radio inside `Field.Item`) renders a `for` attribute and clicking the label text selects the radio: `packages/react/src/radio-group/RadioGroup.test.tsx:1029-1059`; explicit association links `label[for]` to the input `id` and `Field.Description[id]` to the radio's `aria-describedby`: `packages/react/src/radio-group/RadioGroup.test.tsx:1061-1102`.
- `Field.Description` next to the group appends its id to BOTH the group's `aria-describedby` and each radio's `aria-describedby` (user-supplied values are preserved and come first, e.g. group = `external-description <group-desc-id>`, radio = `radio-description <group-desc-id>`): `packages/react/src/radio-group/RadioGroup.test.tsx:1106-1140`.
- `Fieldset.Legend` labels the group: the group's `aria-labelledby` points at the legend's id: `packages/react/src/radio-group/RadioGroup.test.tsx:1252-1268`.
- Label precedence and cleanup: explicit `aria-labelledby` beats `Field.Label`, which beats `Fieldset.Legend`; replaced or unmounted label ids are dropped from `aria-labelledby` (no stale references): `packages/react/src/radio-group/RadioGroup.test.tsx:1270-1358`.
- Validation state: on failed validation the group and all radios get `aria-invalid="true"`; selecting a value clears it from the group and every radio: `packages/react/src/radio-group/RadioGroup.test.tsx:1825-1837`; `validationMode="onSubmit"` flow: `packages/react/src/radio-group/RadioGroup.test.tsx:1144-1187`; external error from `Form errors` prop is cleared on change: `packages/react/src/radio-group/RadioGroup.test.tsx:1931-1961`.
- Error linking: the `Field.Error` id is appended to the radio's `aria-describedby` alongside its description id: `packages/react/src/radio-group/RadioGroup.test.tsx:1963-1988`.
- `validationMode="onBlur"` validates only when focus leaves the group: a blur whose `relatedTarget` is another radio inside the group does NOT validate; a blur to an element outside the group validates once with the current value: `packages/react/src/radio-group/RadioGroup.test.tsx:1189-1218`.

## DOM structure & portal behavior

- Root is a single `div` (conformance): `packages/react/src/radio-group/RadioGroup.test.tsx:17-20`.
- Each `Radio.Root` renders its exposed element plus a hidden `<input type="radio">` as its immediate next sibling; the hidden input carries `name` (from the group's `name` prop) and `value` (from the item): `packages/react/src/radio-group/RadioGroup.test.tsx:309-320`.
- Name resolution: `Field.Root name` takes precedence over the group's own `name` for the hidden inputs: `packages/react/src/radio-group/RadioGroup.test.tsx:929-944`, `packages/react/src/radio-group/RadioGroup.test.tsx:967-982`.
- Native form data (Chromium-only tests): with nothing selected `FormData.get(name)` is `null` (matching native radios): `packages/react/src/radio-group/RadioGroup.test.tsx:584-598`; with a selection the value is included: `packages/react/src/radio-group/RadioGroup.test.tsx:600-620`; native submission of an unselected group yields `null` too: `packages/react/src/radio-group/RadioGroup.test.tsx:560-582`.
- Form association via the `form` prop: the group's hidden inputs submit into the referenced external `<form>` on its submit: `packages/react/src/radio-group/RadioGroup.test.tsx:1370-1393`; conversely, a radio associated to another form (via `form`) is omitted from the current form's `FormData` and its `onFormSubmit` values: `packages/react/src/radio-group/RadioGroup.test.tsx:1687-1712`.
- Portals:
  - A `Radio.Root` portaled (React `createPortal`) outside the form's DOM subtree has no native form association (native `FormData` omits it), but is still included in `Form.onFormSubmit` via context-driven Field registration: `packages/react/src/radio-group/RadioGroup.test.tsx:1714-1743`.
  - A group fully portaled outside the form element (with its `Field.Root`) still participates in `onFormSubmit`: `packages/react/src/radio-group/RadioGroup.test.tsx:1745-1769`.
- The group itself never portals its own DOM; UNVERIFIED — inferred from `packages/react/src/radio-group/RadioGroup.test.tsx` (no portal-render test of the group root), no test asserts this.

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onValueChange(value, eventDetails)`: called once per committed interaction; first argument is the newly selected string value: `packages/react/src/radio-group/RadioGroup.test.tsx:38-52`. The second argument exposes the underlying event's properties (e.g. `eventDetails.event.shiftKey`): `packages/react/src/radio-group/RadioGroup.test.tsx:54-71`.
- Cancellation semantics: `eventDetails.cancel()` inside `onValueChange` prevents the state change (checked state, hidden input, and Field touched/dirty/filled hooks all stay unchanged) for clicks, hidden-input clicks, and arrow navigation: `packages/react/src/radio-group/RadioGroup.test.tsx:117-137`, `packages/react/src/radio-group/RadioGroup.test.tsx:139-164`, `packages/react/src/radio-group/RadioGroup.test.tsx:166-195`.
- No dedicated public event fires for Enter (Space-on-keyup is the only keyboard activation): `packages/react/src/radio-group/RadioGroup.test.tsx:97-115`. Bubbling behavior of the change is not asserted by any test: UNVERIFIED — inferred from `packages/react/src/radio-group/RadioGroup.test.tsx` (absence of such a test), no test asserts this.
- Form submission events:
  - `Form.onFormSubmit` receives `{ [fieldName]: value | null }` — `null` when nothing is selected: `packages/react/src/radio-group/RadioGroup.test.tsx:1422-1441`.
  - A selected radio projects its value (`{ choice: 'a' }`): `packages/react/src/radio-group/RadioGroup.test.tsx:1593-1617` (Chromium-only).
  - Disabled radios are excluded from both native `FormData` and `onFormSubmit` even when selected, matching native behavior; disabling after mount and initial-disabled cases are both covered: `packages/react/src/radio-group/RadioGroup.test.tsx:1501-1532`, `packages/react/src/radio-group/RadioGroup.test.tsx:1570-1591`; re-enabling before submit includes the value again: `packages/react/src/radio-group/RadioGroup.test.tsx:1534-1568`.
  - Radios disabled by an ancestor `<fieldset disabled>` are excluded natively (even though their `disabled` property is `false`), and are included once the fieldset is enabled: `packages/react/src/radio-group/RadioGroup.test.tsx:1619-1647`, `packages/react/src/radio-group/RadioGroup.test.tsx:1649-1685`.
  - Required validation: submitting an empty required group triggers native HTML validation (the `valueMissing` `Field.Error` renders): `packages/react/src/radio-group/RadioGroup.test.tsx:1395-1420`; selecting a value clears the error and `aria-invalid`: `packages/react/src/radio-group/RadioGroup.test.tsx:1803-1838`.
  - A disabled checked radio still satisfies the group's required (`valueMissing`) constraint — submission proceeds with `null` but no required error: `packages/react/src/radio-group/RadioGroup.test.tsx:1771-1801` (Chromium-only).
  - Custom `Field validate` receives the group value as the first argument; with a controlled group, changing the value externally triggers one revalidation: `packages/react/src/radio-group/RadioGroup.test.tsx:984-1025`.

### Hidden-input & inputRef event behavior

- A programmatic/native click on the hidden input updates the group state (`aria-checked` flips to `'true'`): `packages/react/src/radio-group/RadioGroup.test.tsx:264-279`.
- Validation works when `inputRef` is provided as a function ref: `packages/react/src/radio-group/RadioGroup.test.tsx:1840-1863`.

## Edge cases (rapid interactions, unmount, nesting)

- Rapid/sequential clicks: sequential clicks across items produce exactly one `onValueChange` per click with correct `data-checked`/`data-unchecked` swaps; re-clicking the selected item re-asserts it as checked: `packages/react/src/radio-group/RadioGroup.test.tsx:794-840`, `packages/react/src/radio-group/RadioGroup.test.tsx:868-894`. A dedicated rapid-fire stress test does not exist: UNVERIFIED — inferred from `packages/react/src/radio-group/RadioGroup.test.tsx`, no test asserts this.
- Item unmount: removing the highlighted radio relocates the tab stop to the checked radio: `packages/react/src/radio-group/RadioGroup.test.tsx:764-790`. Removing the radio currently referenced by `inputRef` detaches the ref (`null`): `packages/react/src/radio-group/RadioGroup.test.tsx:499-526`, `packages/react/src/radio-group/RadioGroup.test.tsx:528-558`.
- All radios unmounting:
  - Submission is unblocked (required constraint gone) and `onFormSubmit` receives `{ choice: null }`: `packages/react/src/radio-group/RadioGroup.test.tsx:1443-1470`.
  - A custom field `validate` still runs and can still block submission with a rendered error: `packages/react/src/radio-group/RadioGroup.test.tsx:1472-1499`.
- Ancestor `<fieldset disabled>`: inputRef becomes available after enabling: `packages/react/src/radio-group/RadioGroup.test.tsx:1223-1250`; validation and first-radio focus after enabling: `packages/react/src/radio-group/RadioGroup.test.tsx:1896-1929`.
- External value changes (controlled): revalidate on change: `packages/react/src/radio-group/RadioGroup.test.tsx:984-1025`; clearing value to `null` does not move `inputRef`: `packages/react/src/radio-group/RadioGroup.test.tsx:468-497`.
- `Form errors` external errors clear on the next value change: `packages/react/src/radio-group/RadioGroup.test.tsx:1931-1961`.
- Unselected group submits `null` (both native `FormData` and `onFormSubmit`), matching native radio semantics: `packages/react/src/radio-group/RadioGroup.test.tsx:584-598`, `packages/react/src/radio-group/RadioGroup.test.tsx:1422-1441`.
- Nesting groups inside each other: UNVERIFIED — inferred from `packages/react/src/radio-group/RadioGroup.test.tsx` (no nested-group test exists in this suite), no test asserts this.
- React Strict Mode / SSR: UNVERIFIED — inferred from `packages/react/src/radio-group/RadioGroup.test.tsx` (no Strict Mode or hydration tests in this suite), no test asserts this.

## Shared harness dependencies

- `#test-utils` maps to `packages/react/test/index.ts` (alias declared in `packages/react/package.json:110`). `RadioGroup.test.tsx` imports `isJSDOM` and `createRenderer` from it: `packages/react/src/radio-group/RadioGroup.test.tsx:10`.
  - `isJSDOM` — `packages/utils/src/testUtils.ts:4`: `const isJSDOM = /jsdom/.test(window.navigator.userAgent)`. Used to gate Chromium-only tests (`it.skipIf(isJSDOM)`) that rely on native form submission/layout: `packages/react/src/radio-group/RadioGroup.test.tsx:560`, `packages/react/src/radio-group/RadioGroup.test.tsx:584`.
  - `createRenderer` — `packages/react/test/createRenderer.ts:27-49`: wraps the `@mui/internal-test-utils` renderer's `render` in `act` and returns `rerender`/`setProps` helpers (used for the `setProps({ showLast: false })` unmount test): `packages/react/src/radio-group/RadioGroup.test.tsx:776`, `packages/react/src/radio-group/RadioGroup.test.tsx:786`. `createRenderer({ clockOptions: { shouldAdvanceTime: true } })` + `clock.withFakeTimers()` is used for the `Form` describe: `packages/react/src/radio-group/RadioGroup.test.tsx:1362-1368`.
- `describeConformance` — imported directly from `packages/react/test/describeConformance.tsx:51-70` (not via `#test-utils`): `packages/react/src/radio-group/RadioGroup.test.tsx:12`. It runs the "Base UI component API" suite composed of `propsSpread` (`packages/react/test/conformanceTests/propForwarding.tsx:10-14`), `refForwarding` (`packages/react/test/conformanceTests/refForwarding.tsx:27-40`), `renderProp`, and `className` tests; `RadioGroup.test.tsx` invokes it with only `refInstanceof: window.HTMLDivElement`, which is how the div-root/ref-forwarding claim above is proven: `packages/react/src/radio-group/RadioGroup.test.tsx:17-20`.
- `@mui/internal-test-utils` (external npm package, not a repo file): supplies `act`, `screen`, `fireEvent`, and the underlying `createRenderer`/`createDescribe`: `packages/react/src/radio-group/RadioGroup.test.tsx:11`, `packages/react/test/createRenderer.ts:2-9`.
