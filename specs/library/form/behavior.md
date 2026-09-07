# `Form` — behavior spec (mined from tests)

Unit: `form` (library). Source of truth: `packages/react/src/form/Form.test.tsx` (1182 lines, single file).
This spec records ONLY what tests prove. Claims without test coverage are marked `UNVERIFIED`.

## Public API surface (props, parts, subcomponents)

- `Form` renders a native `<form>` element: the conformance suite asserts the forwarded ref instance is `window.HTMLFormElement` `packages/react/src/form/Form.test.tsx:23-26` (ref attachment proven by the shared harness at `packages/react/test/conformanceTests/refForwarding.tsx:32-38`).
- Props exercised by tests:
  - `onSubmit` — called with the native form submit event; handlers call `event.preventDefault()` inside it `packages/react/src/form/Form.test.tsx:191-193`, `packages/react/src/form/Form.test.tsx:326-328`, and read `event.currentTarget` as a `FormData` source `packages/react/src/form/Form.test.tsx:708-711`.
  - `onFormSubmit` — called as `(formValues, eventDetails)`; `formValues` is a plain record keyed by field name `packages/react/src/form/Form.test.tsx:954-983`.
  - `errors` — controlled, externally-owned error record keyed by field name `packages/react/src/form/Form.test.tsx:633-651`, `packages/react/src/form/Form.test.tsx:702-731`.
  - `noValidate` — defaults to `true` (renders `novalidate` attribute); settable to `false` `packages/react/src/form/Form.test.tsx:1033-1043`.
  - `actionsRef` — imperative handle of type `Form.Actions` exposing `validate()` (whole form) and `validate(name)` (single field by name) `packages/react/src/form/Form.test.tsx:1045-1112`.
- Exported types exercised: `Form.Values` (cross-field validator's second arg) `packages/react/src/form/Form.test.tsx:213-215`, `Form.Props['errors']` `packages/react/src/form/Form.test.tsx:664`, `packages/react/src/form/Form.test.tsx:703`, `Form.Actions` `packages/react/src/form/Form.test.tsx:1048`.
- Custom DOM props (`data-testid`, `lang`, `data-foobar`, `style`) and the `render` prop (function and JSX forms) are forwarded to the rendered element per the standard conformance suite, which Form runs in full (propsSpread, refForwarding, renderProp, className) `packages/react/src/form/Form.test.tsx:23-26`, suite composition at `packages/react/test/describeConformance.tsx:44-49`, custom-prop forwarding proof at `packages/react/test/conformanceTests/propForwarding.tsx:23-36`.
- Parts/subcomponents: Form has no subcomponents of its own; it composes with `Field.Root`, `Field.Control`, `Field.Error`, `Fieldset.Root`, `Checkbox.Root`, `Switch.Root`, and `NumberField.Root/Input` as registered value/validation participants `packages/react/src/form/Form.test.tsx:31-38`, `packages/react/src/form/Form.test.tsx:957-970`.

## State model (controlled/uncontrolled, defaults, transitions)

- Submission gating: submit is blocked when any registered field control is invalid, whether the error is native (`required`) `packages/react/src/form/Form.test.tsx:28-47` or from a custom `validate` function `packages/react/src/form/Form.test.tsx:49-74`.
- A field's invalid state can come from three sources observed here: native constraint validation (`required`) `packages/react/src/form/Form.test.tsx:28-47`, per-field `validate` `packages/react/src/form/Form.test.tsx:56-58`, and the form-level `errors` prop `packages/react/src/form/Form.test.tsx:634-646`. The field-level `invalid` prop also independently blocks submission even when `validate` returns null `packages/react/src/form/Form.test.tsx:1007-1031`.
- When `errors` is provided but contains no entry for a field, that field is not marked invalid `packages/react/src/form/Form.test.tsx:648-660`.
- External `errors` transitions:
  - Errors are removed when the corresponding field's value changes `packages/react/src/form/Form.test.tsx:777-792`; within a single commit, every field that changed has its error removed and untouched fields keep theirs `packages/react/src/form/Form.test.tsx:904-950`.
  - Error objects created with `Object.create(null)` (no `Object.prototype`) are supported; typing into one errored field clears only that field's own error property `packages/react/src/form/Form.test.tsx:662-700`.
  - After a Form-level error is set, the first subsequent change runs the field's own validation, replacing the server error with the field error `packages/react/src/form/Form.test.tsx:794-846`.
  - With `invalid` prop true: in `validationMode="onChange"` the field validator runs on change and replaces the external error `packages/react/src/form/Form.test.tsx:848-873`; in `validationMode="onBlur"` it does not run on change — only on blur `packages/react/src/form/Form.test.tsx:875-902`.
- Async validation:
  - Submission proceeds immediately while an async validator is still pending `packages/react/src/form/Form.test.tsx:190-209`.
  - A resolved async error is stored and blocks the next submit attempt in both `onBlur` and `onChange` modes `packages/react/src/form/Form.test.tsx:289-323`.
  - A previously stored async error is retired (cleared) by the next submit when the value has become valid `packages/react/src/form/Form.test.tsx:250-287`.
- Registration lifecycle:
  - Unmounted fields are removed from the form: they no longer block submission and no longer appear in submitted values `packages/react/src/form/Form.test.tsx:435-470`.
  - Fields inside a disabled `<Fieldset.Root>` are excluded from validation and from `onFormSubmit` values, and get no `aria-invalid` `packages/react/src/form/Form.test.tsx:472-494`.
  - When a field (via fieldset or its own `disabled` prop) becomes disabled, its invalid UI is cleared `packages/react/src/form/Form.test.tsx:496-546`, `packages/react/src/form/Form.test.tsx:548-582`; when it becomes enabled again it re-registers and participates in validation and values `packages/react/src/form/Form.test.tsx:584-631`.
  - Unnamed registered field controls (no `name` on `Field.Root`) still block submission when invalid and clear their invalid state on change `packages/react/src/form/Form.test.tsx:325-379`.
  - When two controls live in one `Field.Root`, the later-registered control takes over and the previous registration is removed — only one registration remains valid for submission `packages/react/src/form/Form.test.tsx:409-433`.
  - Toggling a control's value (e.g. a checkbox) updates its field-control registration; focus targeting stays stable across these re-registrations `packages/react/src/form/Form.test.tsx:76-101`, `packages/react/src/form/Form.test.tsx:144-148`.
- Values: `onFormSubmit` receives form values aligned with native submission semantics for the tested controls — string input values and a NumberField numeric value (`quantity: 5`, not a formatted string) `packages/react/src/form/Form.test.tsx:977-981`; disabled fields contribute no value `packages/react/src/form/Form.test.tsx:492`, `packages/react/src/form/Form.test.tsx:538`, `packages/react/src/form/Form.test.tsx:618`.

## Keyboard interactions

N/A — no Form-level keyboard behavior (Enter-to-submit, etc.) is asserted in this file. Keyboard input appears only as a means to mutate field values before asserting validation/focus outcomes `packages/react/src/form/Form.test.tsx:833-834`, `packages/react/src/form/Form.test.tsx:895-896`, `packages/react/src/form/Form.test.tsx:1022-1023`.

## Focus management

- On an invalid submit attempt, submit is blocked and the first invalid field is focused `packages/react/src/form/Form.test.tsx:49-74`; focusing invokes `select()` exactly once on the text input `packages/react/src/form/Form.test.tsx:51`, `packages/react/src/form/Form.test.tsx:68-70`.
- First-invalid selection order: fields are focused in document order; after a keyed reorder flips DOM order (without remount), focus follows the new document order (the reordered first field) `packages/react/src/form/Form.test.tsx:158-188`. A registration-order fallback is used for fields portaled into disconnected shadow-root trees, and it stays stable across control value changes and re-submits `packages/react/src/form/Form.test.tsx:103-156`.
- Focus targeting remains on the first invalid field after that control's value changed and re-registered (double toggle between submits) `packages/react/src/form/Form.test.tsx:76-101`.
- External (`errors` prop) validation focuses the first invalid field only on submit, never on value change `packages/react/src/form/Form.test.tsx:733-757`; after a second submission the no-focus-swap-on-change behavior holds `packages/react/src/form/Form.test.tsx:759-775`. Errors set asynchronously after submit (microtask) still trigger the focus `packages/react/src/form/Form.test.tsx:662-692`.
- `actionsRef.validate(name)` re-validates a single field by name and `validate()` validates all fields; targeting follows the current registration even across rename/unmount/replacement under Strict Mode `packages/react/src/form/Form.test.tsx:1046-1112`, `packages/react/src/form/Form.test.tsx:1114-1180`.
- UNVERIFIED — no test asserts scroll-into-view or any focus behavior for fields beyond the first invalid one; inferred from `packages/react/src/form/Form.test.tsx:49-74`, no test asserts this.

## Accessibility (roles, aria-*, id linking)

- Invalid fields expose `aria-invalid="true"`: proven for `Field.Control` text inputs `packages/react/src/form/Form.test.tsx:465`, `packages/react/src/form/Form.test.tsx:526`, `packages/react/src/form/Form.test.tsx:570-571`, `packages/react/src/form/Form.test.tsx:612`, `packages/react/src/form/Form.test.tsx:645`, `packages/react/src/form/Form.test.tsx:872`, `packages/react/src/form/Form.test.tsx:1030`, and for `Switch.Root` (`role="switch"`) `packages/react/src/form/Form.test.tsx:343`, `packages/react/src/form/Form.test.tsx:368`, `packages/react/src/form/Form.test.tsx:403`.
- `aria-invalid` is cleared when the value changes `packages/react/src/form/Form.test.tsx:371-374`, when the field or an ancestor fieldset becomes disabled `packages/react/src/form/Form.test.tsx:531-533`, `packages/react/src/form/Form.test.tsx:575-576`, and when a stale async error is retired `packages/react/src/form/Form.test.tsx:284`, `packages/react/src/form/Form.test.tsx:431-432`.
- Per-field scoping with duplicate names: two `Field.Root`s sharing `name="shared"` get independent validity — only the invalid one gets `aria-invalid` and only its `Field.Error` renders `packages/react/src/form/Form.test.tsx:381-407`.
- `Field.Error` content mirrors the error: it renders the error message text (custom validator message `packages/react/src/form/Form.test.tsx:238`, `packages/react/src/form/Form.test.tsx:845`; external `errors` value `packages/react/src/form/Form.test.tsx:644`, `packages/react/src/form/Form.test.tsx:693-694`) and is absent when there is no error `packages/react/src/form/Form.test.tsx:658`, `packages/react/src/form/Form.test.tsx:1000`.
- UNVERIFIED — `aria-describedby`/id linking between `Field.Control` and `Field.Error` is not asserted anywhere in this file; inferred from `packages/react/src/form/Form.test.tsx:31-38`, no test asserts this.

## DOM structure & portal behavior

- Root element is a native `<form>` with the `novalidate` attribute by default; `noValidate={false}` removes it `packages/react/src/form/Form.test.tsx:1034-1042`.
- Fields may be portaled outside the `<form>`'s DOM subtree (including into shadow roots via `ReactDOM.createPortal`); they still register, block submission, and receive focus (tracked via the shadow root's `activeElement`) `packages/react/src/form/Form.test.tsx:103-156`.
- Keyed field lists can reorder DOM nodes without remounting; the form still resolves first-invalid focus against the resulting document order `packages/react/src/form/Form.test.tsx:158-188`.

## Events (names, payload shape, bubbling, preventDefault semantics)

- `submit` (native, received via `onSubmit`): fired by clicking a submit button `packages/react/src/form/Form.test.tsx:205`, by `fireEvent.submit(form)` `packages/react/src/form/Form.test.tsx:141-146`, and implicitly by `user.click` on a submit button `packages/react/src/form/Form.test.tsx:43`. Payload is the native `FormEvent<HTMLFormElement>`; consumers call `preventDefault()` themselves and can build `FormData` from `event.currentTarget` `packages/react/src/form/Form.test.tsx:191-193`, `packages/react/src/form/Form.test.tsx:708-711`.
- `onFormSubmit` (Base UI-level): receives `(formValues, eventDetails)` where `formValues` is a record of field values `packages/react/src/form/Form.test.tsx:978-981` and `eventDetails.event.defaultPrevented` is `true` (the Form handles/prevents the native default itself) `packages/react/src/form/Form.test.tsx:982`. It fires once per successful submit `packages/react/src/form/Form.test.tsx:977` and not at all when the form is invalid `packages/react/src/form/Form.test.tsx:1001-1003`.
- Validation-triggering events: `blur` triggers field validation in `onBlur` mode, including cross-field validators that read other fields' values, and submit re-runs an onBlur validator that previously failed `packages/react/src/form/Form.test.tsx:233-247`; `change` clears externally-set errors `packages/react/src/form/Form.test.tsx:788-791` and triggers `onChange`-mode validation `packages/react/src/form/Form.test.tsx:868-871`.
- No custom events with bubbling semantics beyond the native submit event are asserted in this file.

## Edge cases (rapid interactions, unmount, nesting)

- Rapid double toggle of a control between submits keeps both registration and focus targeting stable `packages/react/src/form/Form.test.tsx:96-97`, `packages/react/src/form/Form.test.tsx:144-145`.
- React Strict Mode: keyed-subtree moves re-run effects, so registration order can diverge from DOM order — tests pin both document-order focus (`strict: false` to expose the divergence) `packages/react/src/form/Form.test.tsx:178-187` and current-registration targeting of `actionsRef.validate` under `<React.StrictMode>` across rename/unmount/replace `packages/react/src/form/Form.test.tsx:1159-1179`.
- Same-name fields: validity is scoped per field instance, not per name `packages/react/src/form/Form.test.tsx:381-407`.
- Multiple controls per field: last registration wins, previous one is dropped `packages/react/src/form/Form.test.tsx:409-433`.
- Conditional unmount: removing an invalid field lets submission succeed `packages/react/src/form/Form.test.tsx:435-470`.
- Disabled fieldset nesting: fields nested in a disabled `Fieldset.Root` are skipped entirely (validation, values, `aria-invalid`) and their invalid UI clears reactively when disabled flips `packages/react/src/form/Form.test.tsx:472-546`; per-control `disabled` behaves the same `packages/react/src/form/Form.test.tsx:548-582`, with re-registration on re-enable `packages/react/src/form/Form.test.tsx:584-631`.
- Async races: submit is not deferred by a pending async validator `packages/react/src/form/Form.test.tsx:190-209`; a stale failing async result does not block the next submit after the value was fixed `packages/react/src/form/Form.test.tsx:250-287`; a settled async error does block the following submit `packages/react/src/form/Form.test.tsx:289-323`.

## Shared harness dependencies

- `@mui/internal-test-utils` (npm workspace package): provides `createRenderer`, `fireEvent`, `flushMicrotasks`, `screen`, `waitFor`, `within` used throughout `packages/react/src/form/Form.test.tsx:10-17`.
- `packages/react/test/describeConformance.tsx`: shared conformance runner invoked by this unit `packages/react/src/form/Form.test.tsx:18`; it composes the `propsSpread`, `refForwarding`, `renderProp`, and `className` sub-tests `packages/react/test/describeConformance.tsx:44-49` and delegates to `packages/react/test/conformanceTests/*` (e.g. ref assertion `packages/react/test/conformanceTests/refForwarding.tsx:32-38`, custom-prop forwarding `packages/react/test/conformanceTests/propForwarding.tsx:23-36`). It also pulls `packages/react/test/createRenderer` (`BaseUIRenderResult`) `packages/react/test/describeConformance.tsx:12`.
- No other component's test files were read.
