# Radio — behavior spec (Stage 1: mined from tests)

Unit: `radio` (`packages/react/src/radio`). Mined exclusively from this unit's own test files:

- `packages/react/src/radio/root/RadioRoot.test.tsx`
- `packages/react/src/radio/indicator/RadioIndicator.test.tsx`

The `TODO.md` entry for `library: radio` (TODO.md:476-482) declares no `wraps-external:` field, so nothing here is delegated to a third-party npm package; all behavior below is derived from this unit's own tests. The unit is small (two test files), so no batched mining was needed. Every claim is test-proven unless marked UNVERIFIED.

## Public API surface (props, parts, subcomponents)

- `Radio.Root` and `Radio.Indicator` are exported from the namespaced entry `@base-ui/react/radio`; `RadioGroup` is a separate entry point `@base-ui/react/radio-group` that every test composes Root into (`packages/react/src/radio/root/RadioRoot.test.tsx:3-4`, `packages/react/src/radio/indicator/RadioIndicator.test.tsx:5-6`). No other parts of the `Radio` namespace are exercised by this unit's tests.
- `Radio.Root` renders a `span` by default (conformance asserts the ref is `instanceof HTMLSpanElement`), supports the `render` prop, and is registered with the button conformance suite (`button: true`) (`packages/react/src/radio/root/RadioRoot.test.tsx:11-16`).
- `Radio.Indicator` renders a `span` by default (conformance with `refInstanceof: window.HTMLSpanElement`), rendered inside a `Radio.Root` for the conformance run (`packages/react/src/radio/indicator/RadioIndicator.test.tsx:15-20`).
- Props on Root proven by tests: `value` (option identity; accepts `null`, `packages/react/src/radio/root/RadioRoot.test.tsx:42-56`), `id` (`:58-97`), `nativeButton` (`:58-82`), `disabled` (`:233-243`), `onClick` (`:136-209`), and the standard customization props exercised by conformance: `render` with a `<button />` element (`:66`), forwarded `ref` (`packages/react/test/conformanceTests/refForwarding.tsx:32-38`), `render` as function or element with ref/className merging (`packages/react/test/conformanceTests/renderProp.tsx:41-113`, `:146-178`), string `className` (`packages/react/test/conformanceTests/className.tsx:20-23`), and arbitrary DOM props such as `lang`, `data-*`, and `style` (`packages/react/test/conformanceTests/propForwarding.tsx:23-36`, `:82-95`).
- Props on Indicator proven by tests: `keepMounted` (`packages/react/src/radio/indicator/RadioIndicator.test.tsx:93`, `:99`), `onAnimationEnd` (`:94`), `onTransitionEnd` (`:159`), and `className` / `data-testid` (`:34`, `:157-158`).

## State model (controlled/uncontrolled, defaults, transitions)

- `Radio.Root` has no checked state of its own in this unit's tests: its checked state is derived from the surrounding `RadioGroup`, exercised both uncontrolled (`defaultValue`, `packages/react/src/radio/root/RadioRoot.test.tsx:20`) and controlled (`value` bound to React state, `packages/react/src/radio/indicator/RadioIndicator.test.tsx:28`, `:82`). Per-Root `checked`/`onCheckedChange` props are not asserted anywhere in this unit — UNVERIFIED, no test asserts them.
- A Root whose `value` does not match the group value renders unchecked: `aria-checked="false"` (`packages/react/src/radio/root/RadioRoot.test.tsx:79`) and `data-unchecked` without `data-checked` (`:28-29`).
- When the group starts with `defaultValue`, the matching Root renders checked at initial render: `data-checked` present, `data-unchecked` absent (`packages/react/src/radio/root/RadioRoot.test.tsx:20-30`).
- Transitions: selecting a Root sets `aria-checked="true"` (`packages/react/src/radio/root/RadioRoot.test.tsx:53`, `:150`); selecting a sibling sets the previously selected Root back to `aria-checked="false"` (`:54-55`).
- `data-checked` / `data-unchecked` are mutually exclusive style hooks on the Root element (`packages/react/src/radio/root/RadioRoot.test.tsx:26-29`).
- The Root's `value` is an identity tag only: it is never forwarded to the DOM as a `value` attribute (`packages/react/src/radio/root/RadioRoot.test.tsx:32-40`), and `null` is a legal value that can be selected and later deselected by choosing a sibling (`:42-56`).
- Indicator mount semantics: the indicator is absent while its Root is unchecked (`packages/react/src/radio/indicator/RadioIndicator.test.tsx:171`) and present while checked (`:46`, `:108`, `:222`); once the Root becomes unchecked it is removed (`:52-54`, `:228-230`).

## Keyboard interactions

| Key | Behavior |
| --- | --- |
| ArrowDown | Selects the next radio in the group (`aria-checked` flips to `"true"` on the following Root) without dispatching a click event to ancestors (`packages/react/src/radio/root/RadioRoot.test.tsx:211-229`) |

- No test in this unit asserts Space, Enter, Tab, or any other arrow key — UNVERIFIED, no test asserts them in `packages/react/src/radio/root/RadioRoot.test.tsx` or `packages/react/src/radio/indicator/RadioIndicator.test.tsx`.

## Focus management

- No test in this unit makes an explicit focus assertion. The arrow-key test implicitly relies on the clicked Root holding focus before `{ArrowDown}` is sent, but never asserts where focus lands after the selection — UNVERIFIED — inferred from `packages/react/src/radio/root/RadioRoot.test.tsx:211-229`, no test asserts this.
- Focus restoration, focus trapping, and autofocus behavior: N/A — no test asserts any of these in this unit.

## Accessibility (roles, aria-*, id linking)

- The visible control exposes `role="radio"` — tests retrieve it with `getByRole('radio')` (`packages/react/src/radio/root/RadioRoot.test.tsx:96`, `:119`).
- `aria-checked` reflects selection state: `"true"` when selected, `"false"` when a sibling is selected (`packages/react/src/radio/root/RadioRoot.test.tsx:53-55`, `:79-81`).
- `disabled` renders as `aria-disabled="true"` and never as the HTML `disabled` attribute (`packages/react/src/radio/root/RadioRoot.test.tsx:233-243`).
- ID linking in `nativeButton` mode with `render={<button />}`: the consumer `id` lands on the visible control (`packages/react/src/radio/root/RadioRoot.test.tsx:73`) and the hidden native input sibling does not steal it (`:75-77`); clicking a `<label htmlFor={id}>` then activates the radio (`:80-81`).
- Without `nativeButton`, a sibling `<label htmlFor>` associated with the hidden input produces a fallback `aria-labelledby` on the visible control pointing at the label's auto-generated id (`packages/react/src/radio/root/RadioRoot.test.tsx:84-97`); that fallback updates when the `id` prop changes to another label (`:99-134`). Which exact element carries the consumer `id` in this mode is not directly asserted — UNVERIFIED — inferred from the test name at `packages/react/src/radio/root/RadioRoot.test.tsx:84-97`, no test asserts this.

## DOM structure & portal behavior

- Default DOM: a `span` root (`packages/react/src/radio/root/RadioRoot.test.tsx:11-16`). In `nativeButton` mode with `render={<button />}`, a hidden native `<input>` exists as the control's `nextElementSibling` and carries no consumer `id` (`packages/react/src/radio/root/RadioRoot.test.tsx:75-77`).
- `Radio.Indicator` renders a `span` inside its `Radio.Root` (`packages/react/src/radio/indicator/RadioIndicator.test.tsx:15-20`).
- Portal behavior: N/A — no test asserts portal usage; all asserted DOM renders inline.

## Events (names, payload shape, bubbling, preventDefault semantics)

- Click: a single user click on `Radio.Root` propagates exactly one click event to ancestors (`packages/react/src/radio/root/RadioRoot.test.tsx:137-151`), identically in `nativeButton` mode (`:173-187`).
- A consumer `onClick` calling `event.stopPropagation()` blocks ancestor click handlers while the selection still proceeds (`packages/react/src/radio/root/RadioRoot.test.tsx:153-171`, `:189-209`).
- Selecting via ArrowDown dispatches no click event to ancestors (`packages/react/src/radio/root/RadioRoot.test.tsx:211-229`).
- Indicator animation events: `onAnimationEnd` fires when the exit (`data-ending-style`) animation finishes (`packages/react/src/radio/indicator/RadioIndicator.test.tsx:94`, `:113-115`); `onTransitionEnd` fires when the enter transition driven by `data-starting-style` completes (`:159`, `:175-177`).
- No change/checked-change callback is asserted at the Root level in this unit, and no test asserts `preventDefault` semantics for any event — UNVERIFIED, no test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

- `value={null}` is selectable and later deselectable via a sibling (`packages/react/src/radio/root/RadioRoot.test.tsx:42-56`).
- Two Roots sharing the same `value` in one group are tolerated; when the group value moves elsewhere both become unchecked and the indicator unmounts (`packages/react/src/radio/indicator/RadioIndicator.test.tsx:32-41`, `:50-54`).
- Indicator unmount timing: without a defined exit animation the indicator is removed once the radio becomes unchecked (browser-only test, `packages/react/src/radio/indicator/RadioIndicator.test.tsx:22-55`); with a defined exit animation, `data-ending-style` is applied before removal and the element is removed after the animation finishes (`:182-231`).
- `keepMounted` indicator with an exit animation: `onAnimationEnd` fires when the group value moves away (`packages/react/src/radio/indicator/RadioIndicator.test.tsx:57-116`); whether the element is then unmounted is not asserted — UNVERIFIED, no test asserts this.
- Enter animation on mount: when a radio becomes checked, its indicator mounts through a `data-starting-style` transition and remains mounted after `onTransitionEnd` (`packages/react/src/radio/indicator/RadioIndicator.test.tsx:123-180`).
- All animation behavior is browser-only: the two top-level indicator tests `skip()` in JSDOM (`packages/react/src/radio/indicator/RadioIndicator.test.tsx:23-25`, `:58-60`) and the `animations` suite is `describe.skipIf(isJSDOM)` (`:118`).
- Nesting: every proven Root behavior requires placement inside a `RadioGroup`; whether `Radio.Root` or `Radio.Indicator` outside their required parents throws or degrades is not tested in this unit — UNVERIFIED, no test asserts this.

## Shared harness dependencies

- Both test files import `createRenderer`, `describeConformance`, and `isJSDOM` from `#test-utils`, which resolves to the in-repo harness at `packages/react/test/index.ts:1-11`; `isJSDOM` is re-exported from `@base-ui/utils/testUtils` (`packages/utils/src/testUtils.ts:4`).
- `createRenderer` wraps `@mui/internal-test-utils`' renderer so `render` is act-wrapped and returns promise-based `rerender`/`setProps` helpers (`packages/react/test/createRenderer.ts:27-49`).
- `describeConformance` runs the shared prop-forwarding / ref-forwarding / render-prop / className suites over each part (`packages/react/test/describeConformance.tsx:44-49`, `:51-68`).
- Tests also use `@mui/internal-test-utils` directly for `screen`, `fireEvent`, `waitFor`, and the `user` interaction API (`packages/react/src/radio/root/RadioRoot.test.tsx:5`, `packages/react/src/radio/indicator/RadioIndicator.test.tsx:3`).
- Animation tests toggle the global `BASE_UI_ANIMATIONS_DISABLED` flag — disabled in `beforeEach` (`packages/react/src/radio/indicator/RadioIndicator.test.tsx:9-11`) and re-enabled inside animation tests (`:62`, `:124`, `:183`).
