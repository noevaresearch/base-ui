# Fieldset — behavior spec (Stage 1: behavior mined from tests)

Unit: `packages/react/src/fieldset/` (parts: `root`, `legend`).

Mined from: `packages/react/src/fieldset/root/FieldsetRoot.test.tsx`, `packages/react/src/fieldset/legend/FieldsetLegend.test.tsx`, plus the shared conformance harness those tests invoke (listed under Shared harness dependencies).

`TODO.md` has no `wraps-external:` field for this unit — nothing here delegates to a third-party npm package.

## Public API surface (props, parts, subcomponents)

Two subcomponents, used via the `Fieldset` namespace import: `Fieldset.Root` and `Fieldset.Legend` (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:7`, `packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:4`).

### `Fieldset.Root`

- Renders a native `<fieldset>` element: conformance is configured with `inheritComponent: 'fieldset'` and `refInstanceof: window.HTMLFieldSetElement`, and the ref-forwarding conformance test asserts the forwarded ref is an `HTMLFieldSetElement` instance (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:15-19`, `packages/react/test/conformanceTests/refForwarding.tsx:32-38`).
- `disabled?: boolean` — renders the native `disabled` attribute on the fieldset element and disables nested controls (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:21-30`).
- `render` — accepts another component as the rendered root. Proven with Base UI group roots: a rendered `RadioGroup` root receives the disabled state and surfaces `aria-disabled="true"`; a rendered `CheckboxGroup` propagates it to a nested `Checkbox.Root` as `data-disabled`; a rendered `Slider.Root` propagates it to `Slider.Control` as `data-disabled` (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:86-106`).
- Arbitrary DOM props are forwarded to the rendered element: `lang` + `data-foobar` on the default element (`packages/react/test/conformanceTests/propForwarding.tsx:23-36`), on a function `render` element (`packages/react/test/conformanceTests/propForwarding.tsx:38-59`), and on a JSX `render` element (`packages/react/test/conformanceTests/propForwarding.tsx:61-80`); custom `style` is forwarded in all three forms (`packages/react/test/conformanceTests/propForwarding.tsx:82-127`).
- `className` as a string is applied to the rendered element (`packages/react/test/conformanceTests/className.tsx:20-23`); a string className from the component merges with one from the `render` element (`packages/react/test/conformanceTests/renderProp.tsx:146-161`), and a function (state-resolved) className also merges with the `render` element's className (`packages/react/test/conformanceTests/renderProp.tsx:163-178`).
- Render-prop conformance runs against Root: function and element forms render the customized root element, the ref reaches the custom component, and component/render-element refs are merged (`packages/react/test/conformanceTests/renderProp.tsx:41-91`, `packages/react/test/conformanceTests/renderProp.tsx:93-144`).

### `Fieldset.Legend`

- Renders an element whose ref instance is `window.HTMLDivElement` (`refInstanceof` in conformance options) (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:10-15`, `packages/react/test/conformanceTests/refForwarding.tsx:32-38`).
- `id?: string` — when omitted, the legend element receives a generated (non-empty) id and the root's `aria-labelledby` points at it (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:17-28`); when provided, the custom id is used verbatim in the root's `aria-labelledby` (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:30-38`).
- Accepts children rendered as legend content (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:17-28`).
- Arbitrary DOM props are forwarded (conformance suite runs with the legend wrapped inside `Fieldset.Root`) (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:10-15`, `packages/react/test/conformanceTests/propForwarding.tsx:23-36`).
- Throws when rendered outside `<Fieldset.Root>` with message: `Base UI: FieldsetRootContext is missing. Fieldset parts must be placed within <Fieldset.Root>.` (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:69-79`).

## State model (controlled/uncontrolled, defaults, transitions)

- No internal state and no controlled/uncontrolled props are asserted for either part. The only prop-driven state is `disabled` plus the derived `aria-labelledby` legend association.
- `disabled` default: unset/false means enabled — after both nested fieldsets are updated to `disabled={false}`, the nested control is not disabled and `Field.Root` has no `data-disabled` attribute (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:82-83`).
- Transitions are reactive in both directions: toggling `disabled` on ancestor or descendant fieldsets updates the nested control's disabled state and the `data-disabled` attribute on nested `Field.Root`, asserted immediately after the triggering `fireEvent.click` without any wait (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:46-84`).
- Effective-disabled rule: a nested control is disabled if ANY ancestor fieldset is disabled — enabling an inner fieldset does not override a disabled outer fieldset (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:77-80`).
- Legend association: the root's `aria-labelledby` is derived state — it tracks the legend's current `id` prop (updates when the id changes) and is removed entirely when the legend unmounts (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:40-67`).

## Keyboard interactions

N/A — no keyboard-driven behavior is asserted anywhere in this unit's tests (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:1-107`, `packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:1-109`). The only interaction-adjacent behavior proven is that controls inside a disabled fieldset carry the disabled state via native fieldset semantics (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:21-30`).

## Focus management

N/A — no focus management (trapping, movement, restoration) is asserted in this unit's tests (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:1-107`, `packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:1-109`).

## Accessibility (roles, aria-*, id linking)

- The rendered fieldset exposes the implicit `group` role — tests query it with `getByRole('group')` (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:24-27`).
- `aria-labelledby` id linking: set automatically on the root to the Legend's generated id (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:17-28`); honors a custom legend `id` (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:30-38`).
- Association lifecycle: follows legend `id` changes and is cleared (attribute removed) when the legend is removed (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:40-67`).
- SSR: no `aria-labelledby` when no legend is rendered (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:81-85`); with a legend present, SSR output has no `aria-labelledby` while the legend already has a non-empty generated id, and the root's `aria-labelledby` is set only after hydration (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:87-108`).
- Disabled state surfaces on nested Base UI components: nested `Field.Root` gets `data-disabled` (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:75-83`); a rendered `RadioGroup` root gets `aria-disabled="true"`, `Checkbox.Root` gets `data-disabled`, and `Slider.Control` gets `data-disabled` (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:86-106`).

## DOM structure & portal behavior

- Root renders a native `<fieldset>` element — the native `disabled` attribute lands on it (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:21-30`) and its ref instance is `HTMLFieldSetElement` (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:15-19`).
- Legend renders an `HTMLDivElement`-typed element (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:10-15`).
- The `render` prop fully replaces the rendered element; the custom element receives the forwarded props and merged refs (`packages/react/test/conformanceTests/renderProp.tsx:41-144`), including when the custom root is another Base UI component like `RadioGroup` (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:86-106`).
- Portal behavior: N/A — nothing is portaled; all assertions target in-tree elements.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no custom events or event-handler props are asserted for this unit. The only event usage in the tests is `fireEvent.click` on buttons external to the fieldset, used to drive prop changes (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:63-65`, `packages/react/src/fieldset/root/FieldsetRoot.test.tsx:77-81`).

## Edge cases

- Nested fieldsets: an ancestor's `disabled` keeps descendants disabled even when the inner fieldset is enabled; updates propagate in both directions and toggle `data-disabled` on nested `Field.Root` (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:32-84`).
- Fieldset as a render-prop wrapper for other Base UI group roots while disabled (RadioGroup / CheckboxGroup / Slider.Root) (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:86-106`).
- Legend `id` mutation and legend unmount both keep the root's `aria-labelledby` in sync (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:40-67`).
- Context requirement: `Fieldset.Legend` outside `Fieldset.Root` throws the descriptive error quoted in Public API surface (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:69-79`).
- SSR/hydration ordering: the `aria-labelledby` association is deliberately absent pre-hydration even when a legend is rendered (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:87-108`).
- UNVERIFIED — inferred from `packages/react/src/fieldset/root/FieldsetRoot.test.tsx:1-107`, no test asserts: rapid repeated toggling of `disabled`, unmounting `Fieldset.Root` itself, nesting deeper than two fieldset levels, or a legend nested more than one level below the root.
- UNVERIFIED — inferred from `packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:1-109`, no test asserts: multiple legends in one fieldset, or a legend with both children and a custom `id` combined.

## Shared harness dependencies

- The legend test imports `#test-utils` for `describeConformance` and `isJSDOM` (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:5`); `#test-utils` resolves to `packages/react/test/index.ts`, which re-exports the conformance harness (`packages/react/test/describeConformance.tsx`) and `isJSDOM` from `packages/utils/src/testUtils.ts` (a userAgent check) (`packages/react/test/index.ts:1-11`, `packages/utils/src/testUtils.ts:4`).
- The root test imports `describeConformance` directly from the same harness file (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:10`).
- `describeConformance` runs four sub-suites — prop forwarding, ref forwarding, render prop, className — unless skipped (`packages/react/test/describeConformance.tsx:44-68`).
- `createRenderer` / `screen` / `fireEvent` / `waitFor` come from `@mui/internal-test-utils` (external npm test harness, not repo-local) (`packages/react/src/fieldset/root/FieldsetRoot.test.tsx:3`, `packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:3`).
- SSR assertions (`renderToString`, `hydrate`) are provided by that external renderer harness and are skipped in JSDOM via `isJSDOM` (`packages/react/src/fieldset/legend/FieldsetLegend.test.tsx:81-108`).
