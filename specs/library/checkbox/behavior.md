# Checkbox — behavior spec (Stage 1: mined from tests)

Unit: `checkbox` (`packages/react/src/checkbox`). Mined exclusively from this unit's own test files:

- `packages/react/src/checkbox/root/CheckboxRoot.test.tsx`
- `packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx`
- `packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx`

The `TODO.md` entry for `library: checkbox` declares no `wraps-external:` field, so nothing here is delegated to a third-party npm package; all behavior below is derived from this unit's own tests. Every claim is test-proven unless marked UNVERIFIED.

## Public API surface (props, parts, subcomponents)

- `Checkbox.Root` is the primary component; `Checkbox.Indicator` is a subcomponent. Tests import both from the namespaced export `@base-ui/react/checkbox` (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:4`, `packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:3`).
- `Checkbox.Root` renders a `span` by default (conformance asserts refs are `instanceof HTMLSpanElement`) and supports the `render` prop; it is registered with the button conformance suite (`button: true`) (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:16-21`).
- `Checkbox.Indicator` also renders a `span` by default (conformance with `refInstanceof: window.HTMLSpanElement`) (`packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:28-35`).
- Props on Root proven by tests: `checked` / `defaultChecked` (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:240-267`, `:546-575`), `onCheckedChange` (`:269-298`), `disabled` (`:367-385`), `readOnly` (`:388-429`), `required` (`:24-31`, `:688-709`), `indeterminate` (`:432-529`), `name` (`:577-585`), `nativeButton` (`:162-193`, `:1773-1792`), `render` (conformance + `role="switch"` override, `:35-38`; native button, `:1773-1792`), `id` (`:68-130`), `value` (`:1077-1116`), `uncheckedValue` (`:1055-1075`, `:1176-1226`, `:1248-1287`), `form` (external form id, `:1002-1053`), `parent` (inside `CheckboxGroup`, `:506-529`; `aria-controls` wiring, `packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:115-150`), and plain DOM props such as `onClick` (`:133-193`) plus a forwarded `ref` (`:197-211`).
- Props on Indicator proven by tests: `keepMounted` (`packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:79-109`), `className` / `data-testid` / extra `data-*` props (`:69-77`, `:174-179`), `onAnimationEnd` (`:174-179`), `onTransitionEnd` (`:241-246`), and `ref` (conformance, `:28-35`).
- `Checkbox.Root` composes with `CheckboxGroup` (value/allValues/parent coordination, `:506-529`, `:665-684`, `:1410-1435`), `Field.Root` (disabled/name/validation/data-* attributes, `:1291-1701`), and `Form` (errors + submission, `:688-1287`).

## State model (controlled/uncontrolled, defaults, transitions)

- The checked state is a boolean: uncontrolled default is unchecked (`aria-checked="false"`, hidden input `checked === false`) and each click toggles false → true → false (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:213-238`).
- Uncontrolled usage via `defaultChecked` renders checked and can be uncontrolled-toggled afterwards (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:546-575`).
- Controlled usage via `checked`: external state changes update the rendered state; `onCheckedChange` receives the new boolean value as its first argument (`:240-267`, `:269-280`).
- `onCheckedChange(checked, eventDetails)` is vetoable: calling `eventDetails.cancel()` inside the handler leaves both `aria-checked` and the hidden input unchanged (`:282-298`).
- `eventDetails` exposes the triggering DOM event as `.event`, including keyboard modifiers such as `event.shiftKey` (`:300-311`).
- `indeterminate` is an independent, render-time flag: it renders `aria-checked="mixed"`, wins over `checked` for the `aria-checked` value (`:456-459`), is never consumed by clicking (click while `indeterminate` keeps `aria-checked="mixed"`, `:438-449`), and is mirrored to the hidden native input's `indeterminate` property (`:461-468`). Clicking while `indeterminate` still advances the native input's `checked` to `true` while the component re-applies `input.indeterminate = true` (native clicks would otherwise clear it) (`:470-493`).
- Root state is shared downward through `CheckboxRootContext`; the context value shape exercised by tests is `{ checked, disabled, readOnly, required, indeterminate, dirty, touched, valid, filled, focused }` (`packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:8-19`).
- Style-hook data attributes track state on both Root and Indicator: `data-checked` / `data-unchecked` (mutually exclusive), `data-disabled`, `data-readonly`, `data-required` (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:546-575`), and `data-indeterminate` (`:495-504`).
- Inside `Field.Root`, additional lifecycle data attributes are tracked on the Root element: `data-touched` (set after focus then blur, `:1315-1328`), `data-dirty` (set after first user click, `:1330-1344`), `data-filled` (present while checked; added when checked after being unchecked, removed when unchecked again, `:1346-1381`), `data-focused` (`:1438-1456`), `data-valid` / `data-invalid` (`:1472-1503`). In a group, `data-filled` is derived from "any sibling filled" (`:1410-1435`).
- Validation: `validate` is called once per user change (`:1563-1576`), on blur with `validationMode="onBlur"` (`:1611-1633`), on submit with `validationMode="onSubmit"` (`:1505-1539`), immediately with `validationMode="onChange"` (`:1541-1561`), and is re-run when a controlled `checked` value changes externally (`:1578-1609`).

## Keyboard interactions

| Key | Behavior |
| --- | --- |
| Tab | Moves focus to the checkbox control (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:340-351`, `:353-364`) |
| Space | Toggles the checkbox (uncontrolled state flips to checked) (`:340-351`); also toggles when rendered as a native button (`:1787-1788`) |
| Enter | Does NOT toggle the checkbox, in either default or native-button mode (`:353-364`, `:1784-1785`) |

- While inside a `Form`, pressing Enter on a focused checkbox submits the form instead of toggling: the submit handler runs with the submit `<button>` as `event.submitter`, that button's `click` fires once, and the checkbox state does not change (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:777-813`). Submission via Enter still occurs when `readOnly` (`:877-904`) and submits exactly once in `nativeButton` mode (`:906-940`).
- Enter-based submission is suppressed when the keydown is default-prevented by the consumer's own `onKeyDown` (`:815-841`) or by an ancestor (`Form`'s `onKeyDown`) (`:843-875`); at the time the ancestor handler runs, `event.defaultPrevented` is still `false`, proving Root does not itself `preventDefault()` on Enter (`:849-852`, `:871`).
- Enter does not submit when the form has no submit button (`:942-967`) or when the resolved default submit button is disabled — the disabled button is intentionally targeted and clicking it is a no-op rather than falling through to a later submitter (`:969-1000`).
- Keyboard modifier state at click time is observable on `eventDetails.event` (e.g. `shiftKey === true`) (`:300-311`).

## Focus management

- The control is keyboard-focusable: `user.keyboard('[Tab]')` gives it focus (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:346-347`, `:359-360`, `:1781-1782`).
- Field integration reports focus visually: `data-focused` is added on focus and removed on blur (`:1438-1456`), but is suppressed while `disabled` (`:1458-1470`); `data-touched` is set after the first focus+blur cycle (`:1315-1328`).
- A `ref` callback may imperatively call `focus()`, `blur()`, and `click()` on the root element during mount (before the hidden input exists) without crashing or corrupting state — the checkbox stays unchecked (`:197-211`).
- No test in this unit asserts focus restoration, focus trapping, or autofocus behavior. UNVERIFIED — inferred from absence in `packages/react/src/checkbox/root/CheckboxRoot.test.tsx`, no test asserts this.

## Accessibility (roles, aria-*, id linking)

- The Root element carries `role="checkbox"` (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:27`) and exposes `aria-checked` with values `false` / `true` (`:213-238`) or `"mixed"` when `indeterminate` (`:432-436`).
- `role` can be overridden by consumer props (e.g. `role="switch"`) (`:35-38`).
- `required` renders as `aria-required="true"` on the control (`:24-31`); `disabled` renders as `aria-disabled="true"` and never the HTML `disabled` attribute (`:367-372`); `readOnly` renders `aria-readonly="true"` and omits the attribute when unset (`:388-397`).
- Field validation state surfaces as `aria-invalid` (`:1505-1539`, `:1541-1561`, `:1578-1609`), cleared on change when external `Form` errors resolve (`:711-734`).
- ID linking: in a `Field`, the `Field.Label`'s `for` matches the labelable control's `id` — the hidden input when `nativeButton={false}` and the rendered button when `nativeButton={true}` (`:62-66`, `:76-77`, `:622-644`); the checkbox additionally sets `aria-labelledby` to the label's id for both explicit (`:1635-1657`) and implicit (label-wrapped) associations (`:1660-1684`).
- A consumer-supplied `id` is honored and linked (`:76-77`); when the `id` prop is removed the control falls back to a generated (non-empty) id and the label `for` follows (`:68-86`); a keyed remount never reuses a previous instance's id (`:88-106`).
- An empty `id=""` falls back to the Field-provided control id (`:646-663`); valueless checkboxes inside a parent `CheckboxGroup` get generated ids on the hidden input (`:665-674`) or on the native button root (`:676-684`).
- SSR: server markup carries the generated id on both label and control (the explicit `id` is deferred until after hydration, and `Field.Label`'s `for` stays valid throughout) (`:108-130`); under the React 17 id fallback, no `[id]` attribute is server-rendered at all and the label association is wired once fallback ids arrive client-side (`packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:83-113`).
- A sibling `<label htmlFor="...">` pointing at the checkbox id produces a fallback `aria-labelledby` on the visible control (`:1725-1736`), which updates when the hidden input's id changes (`:1738-1771`).
- `aria-describedby` merges the consumer's value with `Field.Description`'s id (`:1687-1701`).
- A `parent` checkbox inside a `CheckboxGroup` gets `aria-controls` listing all child checkbox control ids, space-separated, once ids are assigned (`packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:115-150`).
- Native HTML validation is preserved: a required unchecked checkbox surfaces the `valueMissing` match through `Field.Error` on submit (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:688-709`).

## DOM structure & portal behavior

- Default DOM: a `span` root with `role="checkbox"` plus a sibling hidden native `<input type="checkbox">`; both match `role="checkbox"` when hidden elements are queried — index `[0]` is the visible control, `[1]` the hidden input (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:213-223`, `:577-585`).
- With `nativeButton` + `render={<button />}`, the visible control is an actual `<button>` element (`:1773-1792`, `:622-644`); the hidden input still exists and does not steal the consumer's `id` (`:636-639`).
- The `name` attribute is placed only on the hidden input, never on the visible control (`:577-585`); inside `Field.Root`, the Field's `name` reaches the hidden input the same way (`:1302-1313`).
- `Checkbox.Indicator` renders as a `span` child inside the Root (`:495-504`, `:546-575`) and only mounts when the checkbox state warrants it (see below). State data attributes (`data-checked`, `data-disabled`, `data-readonly`, `data-required`, `data-indeterminate`) are mirrored onto both Root and Indicator (`:546-575`, `:495-504`).
- Indicator mount conditions: unmounted while plain-unchecked (`packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:49-57`), mounted when `checked` (`:59-67`) or when `indeterminate` with `keepMounted` (`:100-108`), and `keepMounted` keeps it mounted in all states (`:79-109`). Without `keepMounted`, it is removed after unchecking — immediately when no exit animation exists (`:111-139`) or after the exit animation finishes (`:141-194`).
- No portal behavior is asserted anywhere in this unit's tests; all asserted DOM (root, indicator, hidden input) renders inline. N/A — no test asserts portal usage.

## Events (names, payload shape, bubbling, preventDefault semantics)

- Click: a single user click on the Root produces exactly one click event on ancestors — no duplicate click despite the hidden input also participating (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:133-146`, `:162-174`); a consumer `onClick` calling `event.stopPropagation()` blocks ancestor handlers while the toggle still proceeds (`:148-160`, `:176-193`).
- `onCheckedChange(checked, eventDetails)`: fires once per user click with the next boolean value (`:269-280`); `eventDetails` carries the source event (modifiers accessible, e.g. `event.shiftKey`, `:300-311`) and supports `eventDetails.cancel()` to veto the state update (`:282-298`).
- Clicking the hidden native input directly toggles the state (`:313-323`, `:532-544`); a click event on the hidden input that arrives already `preventDefault()`-ed is ignored (no `onCheckedChange`, no state change) (`:325-338`).
- Enter inside a Form dispatches a form submission whose `submitter` is the form's submit button, and that button receives a `click` event (`:777-813`); `preventDefault()` on the keydown (consumer or ancestor) suppresses submission (`:815-841`, `:843-875`).
- Form submission values follow native checkbox semantics: unchecked submits nothing (FormData `get` → `null`) unless `uncheckedValue` is set; checked submits the `value` prop, defaulting to `"on"` (`:736-775`, `:1077-1116`, `:1118-1174`, `:1176-1226`); `uncheckedValue` also applies to external `<form id>` targets (`:1055-1075`) and is withheld when `disabled` (`:1228-1246`).
- Indicator animation events: `onAnimationEnd` fires when the exit (`data-ending-style`) animation finishes (`packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:141-194`); `onTransitionEnd` fires when the enter transition driven by `data-starting-style` completes (`:201-263`); the Web Animations API (`element.getAnimations`) is not consulted for transition-based enter animations (`:261-262`).

## Edge cases (rapid interactions, unmount, nesting)

- Imperative interaction inside the `ref` callback before the hidden input mounts (focus/blur/click) is tolerated; the component ends up unchecked (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:197-211`).
- Rapid repeated clicks toggle state deterministically on each click (`:213-238`, `:1704-1723`) and re-validate exactly once per user change (`:1563-1576`).
- Controlled remount (same slot re-keyed) resets Field `data-filled` state on the `Field.Root` (`:1383-1408`).
- StrictMode / remount safety: a keyed id-less checkbox never reuses the previous unmounted instance's id (tested in a non-strict renderer to expose id-handoff timing, `:88-106`; also under the React 17 fallback, `packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:67-81`).
- SSR + hydration: explicit `id` is deferred until hydration while `Field.Label` keeps a working association (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:108-130`); the React 17 id fallback renders no ids on the server at all and wires label `for` only after hydration (`packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:83-113`).
- `indeterminate` interplay with the native input: clicking while indeterminate keeps `input.indeterminate === true` (re-applied after the native click would clear it) and sets `checked` to true (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:470-493`).
- Nesting: `Checkbox.Root` inside a parent `CheckboxGroup` participates in group state — `parent` checkbox renders `aria-checked="mixed"` and mirrors `input.indeterminate` when manually indeterminate (`:506-529`), and group membership drives `data-filled` (`:1410-1435`).
- Nesting: a `Checkbox.Root` inside a plain `<label>` (wrapping) or linked `<label htmlFor>` toggles on label clicks (`:589-620`), including in `nativeButton` mode (`:622-644`) and with `id=""` falling back to the Field control id (`:646-663`).
- Unmount/animation: without a defined exit animation the indicator is removed in the uncheck commit (`packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:111-139`); with one, `data-ending-style` is applied before removal (`:265-313`); when many checkboxes uncheck in the same state update, all indicators are removed in a single React commit (React.Profiler commit counts only ever observe 10 or 0 indicators) (`:315-379`).
- Disabled checkboxes: cannot be toggled (`:374-385`), show no `data-focused` (`:1458-1470`), and contribute no value (or `uncheckedValue`) on submit (`:1228-1246`).
- Animation behavior is globally disableable via `globalThis.BASE_UI_ANIMATIONS_DISABLED = true` (`packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:22-24`).
- Rendering `Checkbox.Indicator` outside `Checkbox.Root` throws with the message "Base UI: CheckboxRootContext is missing. Checkbox parts must be placed within <Checkbox.Root>." (`packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:37-47`).

## Shared harness dependencies

- All three test files import `createRenderer`, `describeConformance`, and `isJSDOM` from `#test-utils`, which resolves to `packages/react/test/index.ts` re-exporting the in-repo harness (`packages/react/test/index.ts:1-11`); `isJSDOM` is re-exported from `@base-ui/utils/testUtils` (`packages/react/test/index.ts:1`).
- `createRenderer` wraps `@mui/internal-test-utils`' renderer so `render` is act-wrapped and returns promise-based `rerender`/`setProps` helpers (`packages/react/test/createRenderer.ts:27-49`); it also provides `renderToString` + `hydrate` used for the SSR tests (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:114-130`).
- `describeConformance` runs the shared prop-forwarding / ref-forwarding / render-prop / className suites over each part (`packages/react/test/describeConformance.tsx:44-49`, `:51-68`); Root registers with `button: true` (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:16-21`) and Indicator with a manual `CheckboxRootContext.Provider` wrapper (`packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:28-35`).
- Tests also use `@mui/internal-test-utils` directly for `screen`, `fireEvent`, `waitFor`, `act`, and the `user` interaction API (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:3`, `packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:6`, `packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:5`).
- Browser-only assertions are gated with `it.skipIf(isJSDOM)` for form-submission tests (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:736-1287`) and `skip()` inside `it` for animation tests (`packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:111-113`, `:141-144`).
- The React 17 fallback file mocks `@base-ui/utils/safeReact` to disable `useId`/`captureOwnerStack` (`packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:9-19`); the indicator file toggles the global `BASE_UI_ANIMATIONS_DISABLED` flag (`packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx:22-24`).
