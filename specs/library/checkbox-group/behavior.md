# checkbox-group — behavior spec

Mined from this unit's own test files only:

- `packages/react/src/checkbox-group/CheckboxGroup.test.tsx` (2211 lines)
- `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx` (537 lines)

The unit's `TODO.md` entry (`library: checkbox-group`, TODO.md:339-345) has no `wraps-external:` field, so behavior is derived entirely from these tests; no third-party delegation applies.

## Public API surface (props, parts, subcomponents)

- `CheckboxGroup` renders a `div` as its root element and forwards the ref to it (`refInstanceof: window.HTMLDivElement`): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:15-19`.
- The root element is exposed as `role="group"` (retrieved via `screen.getByRole('group')` in multiple tests): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:25`, `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:227`.
- Observed props proven by tests: `id` (forwarded to root): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:21-27`; `value` / `onValueChange` (controlled): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:30-69`; `defaultValue` (uncontrolled): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:230-256`; `disabled` (group-level): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:305-325`; `allValues` (declares the full set of child values for parent-checkbox toggling): `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:17-22`. Arbitrary HTML attributes pass through too — `aria-describedby` set on the group ends up on the group element: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1167`, `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1192-1195`.
- `onValueChange(nextValue, eventDetails)` receives the proposed next array as its first argument, and `eventDetails.cancel()` rejects the change: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:196-217`.
- Children are `Checkbox.Root` items (used throughout both suites), with `Checkbox.Root parent` marking a parent/tri-state checkbox: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:18-22`. The `parent` checkbox is distinguishable in the DOM by a `data-parent` attribute: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:28-30`.
- Integrates with `Field.Root` / `Field.Item` / `Field.Label` / `Field.Description` / `Field.Error` and `Form` (tests under the `Field`, `Field.Label`, `Field.Description`, and `Form` describes): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:372-875`, `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:877-1161`, `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1199-1463`, `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1465-2210`.
- No subcomponents are exported by the group itself in these tests; the group works purely through context with `Checkbox.Root` children. UNVERIFIED — inferred from `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1-10` (imports), no test asserts a subcomponent export list.

## State model (controlled/uncontrolled, defaults, transitions)

- Value is a `string[]` of checked child values. Controlled via `value` + `onValueChange`; `aria-checked` on each child reflects membership (`'true'`/`'false'`): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:30-69`.
- The empty string is a valid item value: `value={['']}` checks a `value=""` child and unchecking it works: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:71-93`.
- A controlled value that becomes `undefined` is treated as an empty array (children uncheck, no crash): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:95-123`.
- With no `defaultValue`, the uncontrolled initial value is an empty array; clicks accumulate/removing values in click order: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:169-194`.
- `defaultValue={null}` is tolerated and treated as an empty array (no crash; group still renders): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:220-228`.
- `defaultValue` sets the initial checked state: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:230-256`.
- Cancellation: if `onValueChange` calls `eventDetails.cancel()`, the handler still receives the proposed value but the group does not update (children keep `aria-checked="false"`): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:196-217`.
- Field dirty/filled state on the group root:
  - `data-dirty` is absent initially, appears after the value changes, and is removed when the value returns to the initial value: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:373-399`.
  - `data-filled` is present when the group value is non-empty, even if no rendered checkbox matches that value (a value with no corresponding checkbox still counts), and remains while any value is selected: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:401-421`.
- Parent-checkbox tri-state (requires `allValues`):
  - All children checked → parent `aria-checked="true"`; some → `'mixed'`; none → `'false'`: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:58-85`, `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:131-149`.
  - Parent click when none are checked checks all children; parent click when all are checked unchecks all: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:11-56`.
  - Parent click from a mixed state checks all; clicking again unchecks all; clicking once more restores the pre-"all" snapshot (the exact set that was checked before the mixed→all transition): `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:340-384`.
  - Disabled children are not toggled by the parent: an unchecked disabled child stays unchecked (parent shows `mixed` because the enabled ones checked): `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:488-508`; a checked disabled child stays checked through parent off-toggles while enabled children uncheck: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:510-536`.
  - A child without an identifying value (no `value`/`name`) is not selected by the parent toggle: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:319-338`.
  - The parent snapshot is not polluted by a canceled child change: if a child uncheck is canceled at group level, a subsequent parent click still proposes unchecking everything (snapshot intact): `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:462-486`.
  - The parent toggle cycle does not advance when the group cancels a parent change: from mixed, a canceled attempt stays mixed and the next click retries the mixed→all transition (proposing `allValues` again rather than an empty array): `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:434-460`.
- Group-level `disabled` forces all children disabled (`aria-disabled="true"`), overriding an individual child's `disabled={false}`: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:305-369`. `disabled={false}` on the group does not itself disable children: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:327-347`.

## Keyboard interactions

N/A — no test in this unit's suite exercises keyboard interaction (Space/Enter/arrow keys) against the group root or its checkboxes. Keyboard behavior of individual checkboxes belongs to the `checkbox` unit's tests.

## Focus management

Focus behavior in this unit is exercised through `Form`/`Field` integration (Chromium-only tests, skipped in jsdom via `isJSDOM`):

- When the field receives an error from `Form` (via the `errors` prop on submit), the first checkbox in the group is focused and marked `aria-invalid="true"`: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1528-1564`.
- When a later checkbox in the group fails native validation, that invalid checkbox (not the first) is focused: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1566-1592`.
- A disabled representative checkbox is skipped on a later focus attempt (focus moves to the next enabled checkbox): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:2020-2057`.
- Checkboxes disabled by an ancestor `<fieldset disabled>` are skipped when focusing Form errors: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1710-1742`.
- The parent checkbox is never used as the focus target for group errors; the first child checkbox is focused instead: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:2059-2090`.
- If the first checkbox unmounts as part of the failing submit, a remaining checkbox in the group is focused: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1744-1776`.
- If the group is invalid but has no focusable control (no mounted inputs, or all children disabled), focus skips to the next invalid field in the form: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:2092-2109`, `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:2111-2129`.
- Required portaled checkboxes are validated and focused as part of the form: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1656-1686`.

## Accessibility (roles, aria-*, id linking)

- Group root: `role="group"`: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:25`.
- A `Field.Label` that is a sibling of the group (inside the same `Field.Root`) labels the group itself rather than a checkbox inside it: after hydration the label drops its `for` attribute and the group gets `aria-labelledby` pointing at the label id (server markup still carries the stale `for`): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:946-958`.
- Individual checkboxes get unique ids even when many share one `Field.Root`, in both client render and SSR: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:902-923`, `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:925-942`.
- Implicit label association: `Field.Label` wrapping a checkbox gets `for` = the hidden input's id, and the exposed checkbox's `aria-labelledby` points at the label's id; clicking the label toggles the checkbox: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1071-1114`.
- Explicit label association (label sibling of the checkbox) behaves the same, plus `Field.Description` ids are linked to the checkbox's `aria-describedby`: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1116-1160`.
- Each label points at its item's labelable element: the button itself with `nativeButton`, otherwise the hidden input rendered next to the exposed element: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1050-1056`.
- Checkboxes wrapped in plain `<label>` elements each get their own accessible name via `aria-labelledby` referencing their own label: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:960-985`.
- Group description linking: a `Field.Description` next to the group is appended to the group's `aria-describedby` (after user-supplied values) and also to each checkbox's `aria-describedby`: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1163-1196`.
- Error linking: the `Field.Error` id is appended to individual checkboxes' `aria-describedby` when the form has an error: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:2183-2209`.
- Validation state is exposed as `aria-invalid="true"` on all checkbox elements of the group when the group value is invalid: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:589-619`, `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:660-710`.
- Group `disabled` is exposed as `aria-disabled="true"` on every child: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:305-325`.
- Parent checkbox `aria-controls`:
  - Space-separated list of child element ids: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:184-203`.
  - A child's custom `id` is preserved and used (with `nativeButton` the id stays on the exposed element): `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:205-215`; a `render`-prop element's id is used for both `nativeButton` modes: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:217-238`.
  - Without `nativeButton`, a custom `id` lands on the hidden input, and `aria-controls` references the exposed child element instead: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:240-256`.
  - Ids are looked up only among registered children — prototype-key names in `allValues` (e.g. `'constructor'`) do not leak into `aria-controls`: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:258-270`.
  - Unmounted children are dropped: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:272-290`.
  - Checkboxes sharing the same value both appear in `aria-controls`; the survivor is retained when one unmounts: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:292-317`.
  - During SSR the parent renders no `aria-controls` (children have not registered); it appears after hydration: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1019-1069`.
- `aria-checked` semantics: `'true'`/`'false'` for children; `'mixed'` on the parent checkbox when partially checked: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:58-85`.

## DOM structure & portal behavior

- Root is a single `div` (conformance `inheritComponent: 'div'`): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:15-19`.
- Without `nativeButton`, each `Checkbox.Root` renders an exposed element plus a hidden `input[type="checkbox"]` as its next sibling; the hidden input carries a custom `id` while the exposed element gets the id referenced by labels/`aria-controls`: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:240-256`, `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:319-338`, `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1050-1056`.
- With `nativeButton` (rendering a `<button>`), the exposed element itself is the labelable control carrying the id: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1011-1016`.
- Portals (React `createPortal` outside the group's DOM subtree):
  - A checkbox portaled outside the form DOM still projects its value into `onFormSubmit` via context-driven Field registration (no native form association needed): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1362-1384`.
  - An entire `Field.Root` + group portaled outside the form still participates in submission (disabled children excluded): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1386-1410`.
  - A portaled checkbox explicitly associated via `form="current-form"` is included: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1412-1435`.
  - A checkbox portaled into another `<form>` element without a `form` attribute is ignored by the current form: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1688-1708`.
  - Required portaled checkboxes are validated and focused within the form: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1656-1686`.

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onValueChange(nextValue: string[], eventDetails)`: called once per committed interaction with the full next array in click order: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:127-167`. No duplicate callbacks occur for child clicks in parent-enabled groups: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:87-129`.
- Cancellation semantics: `eventDetails.cancel()` inside `onValueChange` leaves the group state unchanged (the handler is still invoked with the proposed value): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:196-217`.
- Parent-checkbox integration events:
  - Clicking the parent calls the parent's own `onCheckedChange` once and does NOT call any child's `onCheckedChange`: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:11-56`.
  - Clicking a child calls that child's `onCheckedChange` (once): `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:58-85`.
  - The parent's `onCheckedChange` can `cancel()` the group change (nothing updates): `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:386-409`.
  - A child's `onCheckedChange` can likewise `cancel()` the group change: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:411-432`.
- Bubbling of native DOM events through the group root is not asserted by any test in this unit: UNVERIFIED — inferred from `packages/react/src/checkbox-group/CheckboxGroup.test.tsx` (absence of such a test), no test asserts this.
- Native form submission (Chromium-only): the group's selected enabled child values are submitted as repeated entries under the field name (`formData.getAll('apple')` → `['fuji-apple', 'gala-apple']`): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1466-1494`.

### Field/Form value projection semantics

- Validation receives the full logical group value while form values only project selected, enabled, mounted checkboxes: `validateGroup` is called with `['apple', 'banana']` (the group value) while `formValues.fruits` and `onFormSubmit` values are `['apple']` (disabled checkbox excluded): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1200-1247`.
- Selected but unmounted checkboxes are omitted from submission while the group retains their state across remounts: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1249-1282`.
- A `Checkbox.Root` without a `value` prop contributes the field name itself as its value when selected (`defaultValue={['fruits']}` → `{ fruits: ['fruits'] }`): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1284-1301`.
- Parent checkboxes are excluded from form submission: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:2131-2181`.
- Checkboxes disabled by a `<fieldset disabled>` are omitted: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1437-1462`.
- Checkboxes associated with another form (via `form="external-form"`) are omitted: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1339-1360`, `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1594-1615`.

## Edge cases (rapid interactions, unmount, nesting)

- React Strict Mode: two sibling groups each keep isolated default state (omitted defaults don't leak across groups); parent toggles in one group don't affect the other: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:258-301`.
- Duplicate registrations: when duplicate-value checkboxes mount/unmount right before a parent layout-effect-triggered `requestSubmit()`, form values reflect the final registration set (`items: ['two']`): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1303-1337`.
- Unmounting a checked sibling keeps the remaining required checkbox validated (the shared input ref may be nulled; the registry keeps validating): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:489-533`.
- All checkboxes unmounting:
  - Submission is unblocked (no required constraint left): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1778-1808`.
  - Custom `validate` still runs against the preserved logical value and can still block submission: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1810-1845`.
  - A previously rendered custom error clears when the inputless group's value becomes valid: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1847-1888`.
  - With `validationMode="onSubmit"`/`"onBlur"`, value changes do not trigger validation once no input is mounted; submit still validates exactly once with `(logicalValue, { group: [] })`: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1890-1929`.
  - A controlled group that starts without any inputs respects `validationMode` the same way (no change-time validation for onSubmit/onBlur; external value change after submit triggers one more validation for onSubmit, none for onBlur): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1931-1970`.
  - `validationMode="onChange"` validates an inputless controlled group on external value change: `validate(['one'], { group: ['one'] })`: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1972-1994`.
  - The imperative `Field.Root.Actions.validate()` validates an initially empty group: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1996-2018`.
- A checkbox changing form owner (via `form` prop) stops being validated by the original form: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1617-1654`.
- Custom validity cleanup: `validity.customError` set on a registered input is cleared when the group becomes valid even if that input has since been disabled: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:621-658`.
- Stale custom errors are not left behind when toggling checkboxes (re-validation reflects the real current error): `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:589-619`.
- Rapid/sequential interactions: sequential clicks in one test cycle (check/uncheck/check across several boxes) produce consistent per-click `onValueChange` calls and `aria-checked` states; no dedicated rapid-fire stress test exists: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:127-167`. Deeper rapid-interaction behavior: UNVERIFIED — inferred from `packages/react/src/checkbox-group/CheckboxGroup.test.tsx`, no test asserts this.
- Nesting groups inside each other: UNVERIFIED — inferred from `packages/react/src/checkbox-group/CheckboxGroup.test.tsx` (no nested-group test exists in this unit's suite), no test asserts this.

## Shared harness dependencies

- `#test-utils` maps to `packages/react/test/index.ts` (alias declared in `packages/react/package.json:110`). `CheckboxGroup.test.tsx` imports `describeConformance` and `isJSDOM` from it: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:10`.
  - `describeConformance` — `packages/react/test/describeConformance.tsx:51-70`: runs a "Base UI component API" suite composed of `propsSpread` (`packages/react/test/conformanceTests/propForwarding`), `refForwarding`, `renderProp`, and `className` tests; `CheckboxGroup.test.tsx` invokes it with `inheritComponent: 'div'` and `refInstanceof: window.HTMLDivElement` (`packages/react/src/checkbox-group/CheckboxGroup.test.tsx:15-19`), which is how the div-root/ref-forwarding claims above are proven.
  - `isJSDOM` — `packages/utils/src/testUtils.ts:4`: `const isJSDOM = /jsdom/.test(window.navigator.userAgent)`. Used to gate the Chromium-only `describe.skipIf(isJSDOM)('Form', ...)` suite: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:1465`.
- `@mui/internal-test-utils` (external npm package, not a repo file): supplies `createRenderer`, `screen`, `fireEvent`, `waitFor` used by both test files: `packages/react/src/checkbox-group/CheckboxGroup.test.tsx:5`, `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:3`.
- `useCheckboxGroupParent.test.tsx` imports no `#test-utils` harness members; it relies only on the external package plus React Testing Library-style queries: `packages/react/src/checkbox-group/useCheckboxGroupParent.test.tsx:1-5`.
