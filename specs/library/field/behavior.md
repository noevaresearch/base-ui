# Field — behavior spec (Stage 1: behavior mining)

Mined from the unit's own test suite only:

- `packages/react/src/field/control/FieldControl.test.tsx`
- `packages/react/src/field/description/FieldDescription.test.tsx`
- `packages/react/src/field/error/FieldError.test.tsx`
- `packages/react/src/field/item/FieldItem.test.tsx`
- `packages/react/src/field/label/FieldLabel.test.tsx`
- `packages/react/src/field/root/FieldRoot.react17.test.tsx`
- `packages/react/src/field/root/FieldRoot.test.tsx`
- `packages/react/src/field/validity/FieldValidity.test.tsx`

The unit has no `wraps-external:` field in its TODO entry (no third-party package is delegated to).

## Public API surface (props, parts, subcomponents)

- All parts are imported from a single `Field` barrel (`@base-ui/react/field`): `Field.Root`, `Field.Control`, `Field.Label`, `Field.Description`, `Field.Error`, `Field.Item`, `Field.Validity`. Types `Field.Root.Actions`, `Field.Item.State`, `Field.Validity.State`, and `Form.Values` are exported from the barrel namespace. `packages/react/src/field/root/FieldRoot.test.tsx:12`, `packages/react/src/field/root/FieldRoot.test.tsx:1232`, `packages/react/src/field/root/FieldRoot.test.tsx:1482`, `packages/react/src/field/item/FieldItem.test.tsx:3`
- `Field.Root` props proven by tests:
  - `disabled` (boolean) — disables the field and propagates to all parts. `packages/react/src/field/root/FieldRoot.test.tsx:513-531`
  - `invalid` (boolean) — explicitly marks the field invalid. `packages/react/src/field/root/FieldRoot.test.tsx:535-554`
  - `name` (string | undefined) — logical field name for form submission; removable at runtime. `packages/react/src/field/root/FieldRoot.test.tsx:558-570`, `packages/react/src/field/root/FieldRoot.test.tsx:1190-1228`
  - `validate(value, formValues)` — sync or async validator; returns `null`/`undefined`/`''`/empty array (valid), a string, or an array of strings (errors). `packages/react/src/field/root/FieldRoot.test.tsx:1014-1099`, `packages/react/src/field/root/FieldRoot.test.tsx:631-665`, `packages/react/src/field/root/FieldRoot.test.tsx:217-232`
  - `validationMode` — `'onSubmit'` | `'onChange'` | `'onBlur'`. `packages/react/src/field/root/FieldRoot.test.tsx:1363-1433`
  - `validationDebounceTime` (ms). `packages/react/src/field/root/FieldRoot.test.tsx:1836-1872`
  - `dirty` / `touched` (boolean) — controlled override of the dirty/touched state. `packages/react/src/field/root/FieldRoot.test.tsx:2947-2990`, `packages/react/src/field/root/FieldRoot.test.tsx:2993-3020`
  - `actionsRef` — imperative handle typed `Field.Root.Actions` exposing a `validate()` method. `packages/react/src/field/root/FieldRoot.test.tsx:3023-3047`
- `Field.Control` props proven by tests:
  - `value` / `defaultValue` / `onValueChange(value, details)` — controlled and uncontrolled value models. `packages/react/src/field/control/FieldControl.test.tsx:104-121`, `packages/react/src/field/control/FieldControl.test.tsx:277-285`, `packages/react/src/field/control/FieldControl.test.tsx:357-369`
  - `name` — takes precedence over `Field.Root`'s `name` and acts as a dynamic fallback when the root name is absent. `packages/react/src/field/root/FieldRoot.test.tsx:1230-1256`, `packages/react/src/field/root/FieldRoot.test.tsx:1296-1322`
  - `id` — explicit id is honored; removal falls back to a generated id. `packages/react/src/field/root/FieldRoot.test.tsx:182-217`
  - Native input props (`required`, `type`, `minLength`, `pattern`, `readOnly`, `disabled`, `autoFocus`) pass through to the rendered `<input>`. `packages/react/src/field/error/FieldError.test.tsx:110-116`, `packages/react/src/field/root/FieldRoot.test.tsx:2445-2473`, `packages/react/src/field/control/FieldControl.test.tsx:530-558`
  - `aria-describedby` — user value is preserved and appended to. `packages/react/src/field/description/FieldDescription.test.tsx:30-41`
  - `render` — replacement element (`render={<div />}`) or render function. `packages/react/src/field/control/FieldControl.test.tsx:347-355`, `packages/react/src/field/control/FieldControl.test.tsx:26-52`
- `Field.Label` props proven by tests:
  - `nativeLabel` (boolean) — controls whether the rendered element must be a `<label>`; dev warnings enforce agreement with the actual element. `packages/react/src/field/label/FieldLabel.test.tsx:132-159`, `packages/react/src/field/label/FieldLabel.test.tsx:161-187`
  - `render` — e.g. `render={<div />}` for a non-native label. `packages/react/src/field/label/FieldLabel.test.tsx:30-47`
- `Field.Description` — renders its children; no field-specific props beyond the shared render/ref API are exercised. `packages/react/src/field/description/FieldDescription.test.tsx:16-28`
- `Field.Error` props proven by tests:
  - `match` — a boolean (`match` shorthand / `match={true}` always renders; `match={false}` is the default slot for Form/server errors), a `ValidityState` key (`'valueMissing'`, `'tooShort'`, `'patternMismatch'`, `'customError'`, `'badInput'`), or omitted (same as `false`). `packages/react/src/field/error/FieldError.test.tsx:57-104`, `packages/react/src/field/error/FieldError.test.tsx:106-146`, `packages/react/src/field/error/FieldError.test.tsx:299-308`
  - `children` — message content. `packages/react/src/field/error/FieldError.test.tsx:22-29`
- `Field.Item` props proven by tests: `disabled`, `render` (render function receives `Field.Item.State` with `disabled`). `packages/react/src/field/item/FieldItem.test.tsx:21-40`
- `Field.Validity` — render-prop/children-function component receiving the validity state; renders nothing itself in tests. `packages/react/src/field/validity/FieldValidity.test.tsx:104-132`, `packages/react/src/field/validity/FieldValidity.test.tsx:159-195`
- All parts forward refs: Root → `HTMLDivElement` (conformance). `packages/react/src/field/root/FieldRoot.test.tsx:62-65`
- Control → `HTMLInputElement`. `packages/react/src/field/control/FieldControl.test.tsx:19-24`
- Label → `HTMLLabelElement`. `packages/react/src/field/label/FieldLabel.test.tsx:11-17`
- Description → `HTMLParagraphElement`. `packages/react/src/field/description/FieldDescription.test.tsx:9-14`
- Error → `HTMLDivElement`. `packages/react/src/field/error/FieldError.test.tsx:11-16`
- Item → `HTMLDivElement`. `packages/react/src/field/item/FieldItem.test.tsx:14-19`
- `Field.Root` reads/writes state shared with form-aware controls (Checkbox, CheckboxGroup, Radio, RadioGroup, Select, NumberField, Slider, Switch) — those controls register into the field when nested. `packages/react/src/field/root/FieldRoot.test.tsx:1014-1099`

## State model (controlled/uncontrolled, defaults, transitions)

- Value: `Field.Control` supports uncontrolled (`defaultValue`) and controlled (`value` + `onValueChange`) operation; controlled value changes made programmatically (external state update) still sync filled/dirty state and trigger validation. `packages/react/src/field/control/FieldControl.test.tsx:149-174`
- Default `validationMode` is `'onSubmit'`: tests that validate on submit render `Field.Root` without `validationMode`. `packages/react/src/field/root/FieldRoot.test.tsx:1364-1383`
- Outside a `<Form>`, `validate` does not run at all (not on focus, change, or blur) unless invoked through `actionsRef.validate()`. `packages/react/src/field/root/FieldRoot.test.tsx:574-596`, `packages/react/src/field/root/FieldRoot.test.tsx:3023-3047`
- Per-field validity state observed through `Field.Validity`/style hooks has a three-phase shape: neutral (no `data-valid`/`data-invalid`, `validity.valid === null`) → valid → invalid. `packages/react/src/field/validity/FieldValidity.test.tsx:118-123`, `packages/react/src/field/root/FieldRoot.test.tsx:667-718`
- Async validation pending rules:
  - While a validator is in flight, the field publishes neutral validity (no `data-valid`, no `data-invalid`, no `aria-invalid`, no rendered error). `packages/react/src/field/root/FieldRoot.test.tsx:667-718`
  - A previously valid result retires to neutral while revalidating. `packages/react/src/field/root/FieldRoot.test.tsx:720-771`
  - A previously resolved error stays published mid-flight and keeps blocking submission until the new result resolves. `packages/react/src/field/root/FieldRoot.test.tsx:773-840`
  - A stale native error retires to neutral once the constraint passes again during the pending window. `packages/react/src/field/root/FieldRoot.test.tsx:842-897`
  - In `'onSubmit'` mode a resolved async error retires to neutral so a second submit goes through. `packages/react/src/field/root/FieldRoot.test.tsx:932-981`
  - A native constraint failure is published immediately; in `'onBlur'` mode it short-circuits the custom validator entirely (validate is never called). `packages/react/src/field/root/FieldRoot.test.tsx:899-930`
- Touched: set after the control loses focus once (`focus` → `blur`); controlled via the `touched` prop (user input never updates a controlled `touched={false}`). `packages/react/src/field/root/FieldRoot.test.tsx:2507-2540`, `packages/react/src/field/root/FieldRoot.test.tsx:3009-3020`
- Dirty: set when the value differs from the baseline; cleared when the value returns to the original baseline value (including `null`-valued controls like NumberField/Select returning to empty, and numeric values returning to their initial number). `packages/react/src/field/root/FieldRoot.test.tsx:2542-2576`, `packages/react/src/field/root/FieldRoot.test.tsx:2578-2601`, `packages/react/src/field/root/FieldRoot.test.tsx:2603-2644`, `packages/react/src/field/control/FieldControl.test.tsx:123-147`
- Dirty baseline rules on control swap/remount: the baseline is the field's original value, not a swapped-in control's default; a controlled control's remount keeps the original baseline; the baseline is captured only once in StrictMode. `packages/react/src/field/root/FieldRoot.test.tsx:2712-2747`, `packages/react/src/field/root/FieldRoot.test.tsx:2749-2772`, `packages/react/src/field/root/FieldRoot.test.tsx:2774-2794`
- Filled: true while the value is non-empty; flips with user input, external controlled value changes, and control remounts with a new default. `packages/react/src/field/root/FieldRoot.test.tsx:2797-2856`, `packages/react/src/field/control/FieldControl.test.tsx:297-345`
- Native constraint validation (`required`, `typeMismatch`, `badInput`, …) runs before custom `validate`; custom validators "run after native validations". `packages/react/src/field/root/FieldRoot.test.tsx:598-629`
- `valueMissing` suppression in `'onBlur'` mode: a required field that was never dirtied is not marked invalid on blur, and `Field.Validity` reports `validity.valid === true`, `errors: []`, `error: ''`; once dirtied (changed then cleared), the same blur marks it invalid. `packages/react/src/field/root/FieldRoot.test.tsx:1465-1522`
- Other native errors (e.g. `typeMismatch`) mark the field invalid even when it is not dirty. `packages/react/src/field/root/FieldRoot.test.tsx:1810-1826`
- Revalidation on change (once invalid): only `valueMissing` is cleared immediately on change; other native errors like `typeMismatch` are deferred to the next blur/submit. `packages/react/src/field/root/FieldRoot.test.tsx:1694-1770`
- Disabled fields: keep `data-disabled`, keep an explicit `invalid` mark and Form-error-driven `data-invalid`, but do not participate in native constraint validation (no `aria-invalid`). `packages/react/src/field/root/FieldRoot.test.tsx:512-570`

## Keyboard interactions

- Pressing Enter in a text control triggers field validation exactly once in every outcome: when it implicitly submits the form, when no implicit submission occurs, and when a disabled submit button blocks implicit submission (browser-only tests). `packages/react/src/field/control/FieldControl.test.tsx:396-417`, `packages/react/src/field/control/FieldControl.test.tsx:419-440`, `packages/react/src/field/control/FieldControl.test.tsx:442-468`
- Enter validates the *latest* controlled value even when a `keydown` handler changes the value before validation runs. `packages/react/src/field/control/FieldControl.test.tsx:470-496`
- Enter triggers validation outside any `<Form>` as well. `packages/react/src/field/control/FieldControl.test.tsx:498-512`
- No other keyboard behavior (arrow keys, Escape, etc.) is asserted by any test.

## Focus management

- A non-native label (`nativeLabel={false}`) focuses the associated control on click; it renders without a `for` attribute. `packages/react/src/field/label/FieldLabel.test.tsx:30-47`
- `data-focused` is applied to Root and all parts while the control is focused, removed on blur. `packages/react/src/field/root/FieldRoot.test.tsx:2859-2894`
- SSR + `autoFocus`: the control receives the `autofocus` attribute in server HTML; after hydration the focused state (`data-focused`) is synced to Root, control, and label. `packages/react/src/field/control/FieldControl.test.tsx:530-558`
- No focus trapping, roving tabindex, or invalid-field focus behavior is asserted by any Field test (focus-on-submit behavior is not tested at this unit level).

## Accessibility (roles, aria-*, id linking)

- Label↔control association:
  - `Field.Label` sets `for` to the control's id automatically. `packages/react/src/field/label/FieldLabel.test.tsx:19-28`
  - The association tracks control replacement, control id changes, and id removal (falls back to a generated id). `packages/react/src/field/root/FieldRoot.test.tsx:67-98`, `packages/react/src/field/root/FieldRoot.test.tsx:152-217`
  - A stale explicit id is dropped when an id-less control replaces the control that owned it. `packages/react/src/field/root/FieldRoot.test.tsx:100-124`
  - With two controls, the label keeps its selected id when another control unmounts and falls over to the remaining control when the selected one unmounts. `packages/react/src/field/label/FieldLabel.test.tsx:49-79`
  - Label association survives unmount/remount cycles (React 19 `<Activity>` hidden/visible) in both native and non-native modes without console errors. `packages/react/src/field/root/FieldRoot.test.tsx:329-436`
- Group naming: when the field's control is a group (e.g. `CheckboxGroup`), the label suppresses `htmlFor` and the group is named via `aria-labelledby`; the suppression is kept while the subtree is hidden and re-evaluated when the group is replaced by a plain control. `packages/react/src/field/root/FieldRoot.test.tsx:126-150`, `packages/react/src/field/root/FieldRoot.test.tsx:472-510`
- `aria-labelledby` is only applied after hydration, never during SSR (when `Field.Label` is absent it stays absent; when the label is removed after hydration it is removed from the control). `packages/react/src/field/root/FieldRoot.test.tsx:219-231`, `packages/react/src/field/root/FieldRoot.test.tsx:285-327`
- `aria-describedby` on the control:
  - `Field.Description` links its id. `packages/react/src/field/description/FieldDescription.test.tsx:16-28`
  - `Field.Error` links its id when rendered. `packages/react/src/field/error/FieldError.test.tsx:18-30`
  - User-provided `aria-describedby` values are preserved and appended to; empty ids are not registered. `packages/react/src/field/description/FieldDescription.test.tsx:30-52`, `packages/react/src/field/error/FieldError.test.tsx:234-246`
- `aria-invalid="true"` is set on the control when validation finishes with an error (submit, change, or blur depending on mode). `packages/react/src/field/root/FieldRoot.test.tsx:996-1012`, `packages/react/src/field/root/FieldRoot.test.tsx:1408-1433`
- Disabled fields never get `aria-invalid` (they don't participate in native constraint validation) even when marked invalid. `packages/react/src/field/root/FieldRoot.test.tsx:533-570`
- `Field.Item` associates a label with a parent checkbox inside a group (`label.control` is the checkbox; clicking the label toggles all children). `packages/react/src/field/item/FieldItem.test.tsx:85-110`
- Dev-time warnings: `<Field.Label>` logs an error when `nativeLabel` disagrees with the rendered element, including on React 17 without the owner-stack API. `packages/react/src/field/label/FieldLabel.test.tsx:132-187`, `packages/react/src/field/root/FieldRoot.react17.test.tsx:79-113`

## DOM structure & portal behavior

- Element mapping: Root renders a `<div>`, Control an `<input>`, Label a `<label>` (by default), Description a `<p>`, Error a `<div>`, Item a `<div>` (ref conformance per part). `packages/react/src/field/root/FieldRoot.test.tsx:62-65`, `packages/react/src/field/control/FieldControl.test.tsx:19-24`, `packages/react/src/field/label/FieldLabel.test.tsx:11-17`, `packages/react/src/field/description/FieldDescription.test.tsx:9-14`, `packages/react/src/field/error/FieldError.test.tsx:11-16`, `packages/react/src/field/item/FieldItem.test.tsx:14-19`
- Every part supports the `render` prop for custom elements (Control renders a `<div>`, Label a non-`<label>` element). `packages/react/src/field/control/FieldControl.test.tsx:347-355`, `packages/react/src/field/label/FieldLabel.test.tsx:30-47`
- Style hooks: `data-disabled`, `data-invalid`, `data-valid`, `data-touched`, `data-dirty`, `data-filled`, `data-focused` are applied to Root and all rendered parts (Control, Label, Description, Error); `aria-invalid` is control-only. `packages/react/src/field/root/FieldRoot.test.tsx:512-531`, `packages/react/src/field/root/FieldRoot.test.tsx:1584-1628`, `packages/react/src/field/root/FieldRoot.test.tsx:2507-2540`, `packages/react/src/field/root/FieldRoot.test.tsx:2542-2576`, `packages/react/src/field/root/FieldRoot.test.tsx:2797-2831`, `packages/react/src/field/root/FieldRoot.test.tsx:2859-2894`
- `Field.Error` supports mount/unmount animation hooks: `data-starting-style` on enter and `data-ending-style` applied before unmount (browser-only tests). `packages/react/src/field/error/FieldError.test.tsx:316-372`, `packages/react/src/field/error/FieldError.test.tsx:374-425`
- Portal behavior: N/A — no Field part is portaled in any test; no test references portal behavior. UNVERIFIED — inferred from the whole suite, no test asserts portal behavior either way.

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onValueChange(value, details)` on `Field.Control`: receives the new value and a details object exposing `cancel()`; calling `details.cancel()` aborts the change so no validation runs. `packages/react/src/field/control/FieldControl.test.tsx:357-369`
- Native `input` event prevention: if the underlying (cancelable) `input` event is `preventDefault`-ed, `onValueChange` still fires once, but validation does not run and existing server errors are not cleared. `packages/react/src/field/control/FieldControl.test.tsx:371-394`
- `Field.Validity` passes a state object (via children/render function) with shape `{ value, validity, error, errors, transitionStatus }`:
  - `value` is the current control value. `packages/react/src/field/validity/FieldValidity.test.tsx:39-47`
  - `validity` is the native `ValidityState` merged with the custom-verdict (`customError` true for custom errors, `valid` true/false, or `null` before any validation). `packages/react/src/field/validity/FieldValidity.test.tsx:40-41`, `packages/react/src/field/validity/FieldValidity.test.tsx:118-131`
  - `error` is the first error string and `errors` the full array from `validate` (string return → `['error']`; array return → array order preserved). `packages/react/src/field/validity/FieldValidity.test.tsx:159-195`
  - `transitionStatus` is present on the payload. `packages/react/src/field/validity/FieldValidity.test.tsx:124`
- `validate(value, formValues)` receives all current form values (of every registered field kind: checkbox, checkbox-group, input, number-field, radio-group, select, slider, range-slider, switch) as the second argument; unmounted fields are excluded. `packages/react/src/field/root/FieldRoot.test.tsx:1014-1099`, `packages/react/src/field/root/FieldRoot.test.tsx:1101-1140`
- Bubbling: N/A — Field emits no custom DOM events; no test asserts event bubbling for Field behavior.

## Edge cases (rapid interactions, unmount, nesting)

- Async races:
  - Stale async results are ignored when superseded by a newer change (old resolver's error never publishes). `packages/react/src/field/root/FieldRoot.test.tsx:1547-1582`, `packages/react/src/field/control/FieldControl.test.tsx:205-248`
  - Async results superseded during a debounce window are dropped. `packages/react/src/field/root/FieldRoot.test.tsx:1975-2017`
  - A validator rejection keeps the already-published error and keeps blocking submission. `packages/react/src/field/root/FieldRoot.test.tsx:2060-2107`
- Unmount:
  - A pending debounced validation is dropped when the control unmounts. `packages/react/src/field/root/FieldRoot.test.tsx:1936-1973`
  - An in-flight async validation is dropped when the control unmounts (its result never publishes). `packages/react/src/field/root/FieldRoot.test.tsx:2019-2058`
  - A pending debounce is dropped when another field-aware control takes ownership, but stays armed across the first registration of a control. `packages/react/src/field/root/FieldRoot.test.tsx:2109-2147`, `packages/react/src/field/root/FieldRoot.test.tsx:2149-2170`
- Debounce semantics: with `validationDebounceTime`, repeated changes reset the timer and only the last value validates; applies to field-aware controls and radio groups alike. `packages/react/src/field/root/FieldRoot.test.tsx:1836-1934`
- Controlled/uncontrolled value on remount: prefilled (`defaultValue`) controls set `data-filled` on mount; remounting with an empty value clears it (both controlled and uncontrolled). `packages/react/src/field/control/FieldControl.test.tsx:277-285`, `packages/react/src/field/control/FieldControl.test.tsx:297-345`
- Programmatic DOM value changes are not clobbered by `defaultValue` on refocus. `packages/react/src/field/root/FieldRoot.test.tsx:2897-2944`
- Custom-validity ownership (`setCustomValidity` interplay):
  - A message set outside the field survives submit, change validation, and revalidation, and is surfaced by `Field.Error`. `packages/react/src/field/root/FieldRoot.test.tsx:2173-2237`
  - The field's own message normalizes `\r\n` to `\n` when written to the control. `packages/react/src/field/root/FieldRoot.test.tsx:2269-2290`
  - The field restores a displaced external message when its own message clears, but does not restore a message withdrawn while its own was installed, and does not adopt a native message as its displaced one. `packages/react/src/field/root/FieldRoot.test.tsx:2292-2401`
  - The field clears its own message on an input that became disabled. `packages/react/src/field/root/FieldRoot.test.tsx:2342-2380`
  - Per-input external messages within a group (radios): the external message moves with the input it was set on. `packages/react/src/field/root/FieldRoot.test.tsx:2403-2443`
  - Controls barred from validation (`readOnly` → `willValidate === false`): external messages are ignored and the field never writes its own message to them (browser-only tests). `packages/react/src/field/root/FieldRoot.test.tsx:2445-2504`
- Form integration:
  - Swapping one field-aware control for another submits the replacement's value. `packages/react/src/field/root/FieldRoot.test.tsx:1142-1188`
  - Removing `name` excludes a registration-gated control from `onFormSubmit` values. `packages/react/src/field/root/FieldRoot.test.tsx:1190-1228`
  - Name resolution order: `Field.Control`'s own `name` wins over `Field.Root`'s `name`; the fallback tracks rename/clear dynamically. `packages/react/src/field/root/FieldRoot.test.tsx:1230-1360`
- Nesting: `Field.Item` scopes `disabled` to wrapped checkboxes/radios inside groups without disabling siblings. `packages/react/src/field/item/FieldItem.test.tsx:42-82`
- React-version edges: a dedicated React 17 suite (with `SafeReact.useId`/`captureOwnerStack` stubbed out) proves generated-id fallback and mount-time imperative validation before the fallback id is assigned. `packages/react/src/field/root/FieldRoot.react17.test.tsx:8-18`, `packages/react/src/field/root/FieldRoot.react17.test.tsx:24-77`
- Render performance: uncontrolled input changes avoid re-rendering the control entirely; controlled changes render exactly once per keystroke. `packages/react/src/field/control/FieldControl.test.tsx:26-87`
- Error arrays: multi-error payloads render as a `<ul>` with one `<li>` per error; single-item arrays render as plain text; empty arrays render nothing and clear `aria-invalid`. `packages/react/src/field/error/FieldError.test.tsx:183-212`, `packages/react/src/field/error/FieldError.test.tsx:214-232`, `packages/react/src/field/error/FieldError.test.tsx:248-260`

## Shared harness dependencies

- `#test-utils` resolves to `packages/react/test/index.ts` (packages/react/package.json:110), which re-exports `@base-ui/utils/testUtils` plus local helpers. Field tests use `createRenderer` (providing `render`, `renderToString`/`hydrate`, `renderStrict`, fake-timer `clock` with `clock.withFakeTimers()`), `describeConformance`, and `isJSDOM` (used to split JSDOM-only from Chromium-only tests via `it.skipIf`). `packages/react/src/field/root/FieldRoot.test.tsx:21`, `packages/react/src/field/root/FieldRoot.test.tsx:33-37`, `packages/react/src/field/root/FieldRoot.test.tsx:1832-1834`
- `packages/react/test/describeConformance.tsx` runs the shared conformance suite (propsSpread, refForwarding, renderProp, className); FieldItem and FieldDescription import it directly by relative path. `packages/react/test/describeConformance.tsx:44-49`, `packages/react/src/field/item/FieldItem.test.tsx:9`, `packages/react/src/field/description/FieldDescription.test.tsx:4`
- `@mui/internal-test-utils` (npm package) provides `act`, `fireEvent`, `screen`, `waitFor`, `flushMicrotasks`, `reactMajor`, and the `user` event object. `packages/react/src/field/root/FieldRoot.test.tsx:13-20`, `packages/react/src/field/control/FieldControl.test.tsx:3-10`
- `FieldRoot.test.tsx` reaches into the unit-internal hook `useFieldRootContext` (`packages/react/src/internals/field-root-context/FieldRootContext`) only to grab `validation.commit` for the rejection test. `packages/react/src/field/root/FieldRoot.test.tsx:22`, `packages/react/src/field/root/FieldRoot.test.tsx:2062-2067`
- `FieldRoot.react17.test.tsx` mocks `@base-ui/utils/safeReact` to remove `useId`/`captureOwnerStack`, simulating React 17. `packages/react/src/field/root/FieldRoot.react17.test.tsx:8-18`
- Chromium-only tests toggle the global `BASE_UI_ANIMATIONS_DISABLED` flag for animation assertions. `packages/react/src/field/error/FieldError.test.tsx:311-314`
