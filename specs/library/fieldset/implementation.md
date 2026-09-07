# Fieldset — implementation spec (Stage 2: implementation mining)

Unit: `packages/react/src/fieldset/` (parts: `root`, `legend`).

Ground truth for WHAT happens: `specs/library/fieldset/behavior.md` (referenced below by section name only).

External delegation: none. `TODO.md`'s `library: fieldset` entry has no `wraps-external:` field (`TODO.md:390-396`), consistent with the statement in behavior.md's front matter — there is no third-party npm package whose internals this spec would otherwise need to derive, and no Rust crate stands in for one.

## State machine / hooks used

The unit has exactly one piece of internal state and no controlled/uncontrolled machinery (`useControlled` is not used anywhere here).

### Root (`packages/react/src/fieldset/root/FieldsetRoot.tsx`)

- `React.useState<string | undefined>(undefined)` holds `legendId` (`packages/react/src/fieldset/root/FieldsetRoot.tsx:25`). It starts `undefined` and is only populated by the legend's registration effect, which is why `aria-labelledby` is omitted (React drops `undefined` attribute values) both pre-hydration and when no legend exists (behavior.md, *Accessibility* SSR bullets).
- `useFieldsetRootContext(true)` — the optional overload — reads a possible ancestor fieldset's context (`packages/react/src/fieldset/root/FieldsetRoot.tsx:27`). Effective disabled is a one-line OR: `disabled = parentDisabled || disabledProp` (`packages/react/src/fieldset/root/FieldsetRoot.tsx:28`). Because the context value carries the *already-OR-ed* effective value rather than the raw prop, nesting composes to arbitrary depth with no extra logic, and the *Effective-disabled rule* in behavior.md (enabling an inner fieldset cannot override a disabled outer one) falls out structurally.
- `React.useMemo` builds the context value with deps `[legendId, setLegendId, disabled]` (`packages/react/src/fieldset/root/FieldsetRoot.tsx:46-53`); `setLegendId` is a stable `useState` setter, so context identity changes only when `legendId` or `disabled` actually change.
- `useRenderElement('fieldset', componentProps, { ref, state, props })` renders the element (`packages/react/src/fieldset/root/FieldsetRoot.tsx:34-44`). Two details worth preserving in a port:
  - `disabled` is destructured out of `componentProps` with a `false` default (`packages/react/src/fieldset/root/FieldsetRoot.tsx:21`), so the raw prop never leaks into `elementProps`; only the effective value is spread (`packages/react/src/fieldset/root/FieldsetRoot.tsx:40`).
  - The props array is `[internalProps, elementProps]` (`packages/react/src/fieldset/root/FieldsetRoot.tsx:37-43`), and `mergePropsN` resolves conflicts rightmost-wins for plain values (`packages/react/src/merge-props/mergeProps.ts:14-15`, `packages/react/src/merge-props/mergeProps.ts:99-115`), i.e. user-supplied props win over the internally managed `aria-labelledby`/`disabled`.
- `state: { disabled }` (`packages/react/src/fieldset/root/FieldsetRoot.tsx:30-32`) is converted to a `data-disabled=""` attribute by `useRenderElement`'s state-attributes machinery (`packages/react/src/internals/getStateAttributesProps.ts:24-28` — a `true` state value becomes `data-<key>=""`; `false` produces nothing). No custom `stateAttributesMapping` is passed.

### Legend (`packages/react/src/fieldset/legend/FieldsetLegend.tsx`)

- `useFieldsetRootContext()` — the required overload — throws the error quoted in behavior.md's *Public API surface* when there is no root ancestor (`packages/react/src/fieldset/root/FieldsetRootContext.ts:14-21`); the overloads that make `true`/absent pick optional-vs-required typing are at `packages/react/src/fieldset/root/FieldsetRootContext.ts:12-14`.
- `useRegisteredLabelId(idProp, setLegendId)` (`packages/react/src/fieldset/legend/FieldsetLegend.tsx:22`) does two jobs in one hook:
  - **Id resolution**: `useBaseUiId(idProp)` (`packages/react/src/utils/useRegisteredLabelId.ts:10`) wraps `useId(idOverride, 'base-ui')` (`packages/react/src/internals/useBaseUiId.ts:9-11`), which on modern React returns a server-stable `React.useId` value with the `base-ui-` prefix (`packages/utils/src/useId.ts:32-37`). This is why the legend's own `id` attribute is present in SSR markup (behavior.md, *Accessibility* SSR bullet) — it comes straight from render, not from an effect.
  - **Registration**: a `useIsoLayoutEffect` pushes the resolved id into the root's state and its cleanup withdraws it only if it is still current (`setLabelId((currentId) => (currentId === id ? undefined : currentId))`) (`packages/react/src/utils/useRegisteredLabelId.ts:12-17`, guard at `packages/react/src/utils/useRegisteredLabelId.ts:14-16`). This single effect explains behavior.md's *Association lifecycle* end to end: id changes re-run the effect (the `id` dep), unmount runs the cleanup, and SSR/hydration ordering exists because effects do not run on the server, so `legendId` stays `undefined` until after hydration.
- `state: { disabled }` taken from context (`packages/react/src/fieldset/legend/FieldsetLegend.tsx:24-26`) surfaces as `data-disabled` on the legend element through the same `getStateAttributesProps` mechanism as Root.
- `useRenderElement('div', componentProps, { state, ref, props: [{ id }, elementProps] })` (`packages/react/src/fieldset/legend/FieldsetLegend.tsx:28-32`); the resolved id is spread as a plain attribute (`packages/react/src/fieldset/legend/FieldsetLegend.tsx:31`), again with `elementProps` last (user id wins, though `useRegisteredLabelId` already resolves the override so both agree).

## Context providers/consumers

`FieldsetRootContext` (`packages/react/src/fieldset/root/FieldsetRootContext.ts:4-10`) carries `{ legendId, setLegendId, disabled }`. The provider wraps the rendered element (`packages/react/src/fieldset/root/FieldsetRoot.tsx:55-57`), so everything in the fieldset's React subtree sees the context while the fieldset element itself does not need to.

In-unit:

- `FieldsetLegend` consumes `disabled` + `setLegendId` (`packages/react/src/fieldset/legend/FieldsetLegend.tsx:20`). Flow is bidirectional: `setLegendId` flows upward (legend → root state → `aria-labelledby`), `disabled` flows downward (root effective state → legend state attribute).
- Nested `FieldsetRoot` consumes only `disabled`, via the optional overload (`packages/react/src/fieldset/root/FieldsetRoot.tsx:27`).

Cross-unit consumers (context crosses the unit boundary — load-bearing for dependency computation):

- `Field.Root` imports the hook (`packages/react/src/field/root/FieldRoot.tsx:10`) and reads `disabled` with the optional overload (`packages/react/src/field/root/FieldRoot.tsx:44`), OR-ing it into its own disabled (`packages/react/src/field/root/FieldRoot.tsx:48`). This is the entire mechanism behind behavior.md's *Accessibility* assertions about nested `Field.Root` gaining/losing `data-disabled` — fieldset-side code does nothing Field-specific.
- `RadioGroup` imports the hook (`packages/react/src/radio-group/RadioGroup.tsx:15`), reads the context optionally (`packages/react/src/radio-group/RadioGroup.tsx:66`) and consumes `legendId` as the `aria-labelledby` fallback (`packages/react/src/radio-group/RadioGroup.tsx:191`). Notably, `legendId` has **no in-unit consumer** — Legend uses only `disabled`/`setLegendId` — so the field's only reader lives in radio-group.

## DOM/portal strategy and why

- Root renders a native `<fieldset>` (`packages/react/src/fieldset/root/FieldsetRoot.tsx:34`) and spreads the effective `disabled` onto it (`packages/react/src/fieldset/root/FieldsetRoot.tsx:40`). This is the deliberate core decision: the browser's native fieldset semantics do the work behind behavior.md's *Keyboard interactions* note and *Public API surface* `disabled` bullet — descendant form controls are disabled natively (no JS walking of descendants), and the implicit `group` role (behavior.md, *Accessibility*) comes free. No portal is used anywhere; nothing escapes the tree.
- Legend renders a `<div>`, not a native `<legend>` (`packages/react/src/fieldset/legend/FieldsetLegend.tsx:28`; the JSDoc says so explicitly at `packages/react/src/fieldset/legend/FieldsetLegend.tsx:8-12`). Native `<legend>` is laid out into the fieldset border rather than normal block flow, which is hostile to styling; Base UI instead keeps a plain block element and establishes the association programmatically via `aria-labelledby` on the root (`packages/react/src/fieldset/root/FieldsetRoot.tsx:39`). The cost of that trade is exactly the JS-managed lifecycle behavior.md documents (registration, id-change tracking, unmount removal, hydration ordering), and it is why Legend's conformance `refInstanceof` is `HTMLDivElement`.
- Render-prop replacement is delegated entirely to `useRenderElement`: function renders are invoked (`packages/react/src/internals/useRenderElement.tsx:169`) and element renders are cloned with the merged props (`packages/react/src/internals/useRenderElement.tsx:196`) inside `evaluateRenderProp` (`packages/react/src/internals/useRenderElement.tsx:158-206`). This is also why the render-composition tests with Base UI group roots pass (behavior.md, *DOM structure & portal behavior*): the cloned element receives `disabled` as a plain prop (`packages/react/src/fieldset/root/FieldsetRoot.tsx:40`), and because Base UI group roots name their prop `disabled` too, the value lands in the group's own disabled logic (with the group's own state machinery producing the `aria-disabled`/`data-disabled` surfacing those tests assert). RadioGroup additionally reads the context directly for legend labeling — see *Context providers/consumers*.
- Module shape: `index.parts.ts` aliases the two parts as `Root`/`Legend` (`packages/react/src/fieldset/index.parts.ts:1-2`) and `index.ts` exposes the `Fieldset` namespace plus type re-exports (`packages/react/src/fieldset/index.ts:1-4`) — the import style behavior.md's *Public API surface* records.

## Dependencies on other Base UI internals

Direct imports:

- `internals/useRenderElement` (`packages/react/src/fieldset/root/FieldsetRoot.tsx:5`, `packages/react/src/fieldset/legend/FieldsetLegend.tsx:3`) — the render/className/style/state-attributes/ref-merging engine. Both parts delegate every conformance-tested behavior (prop forwarding, ref merging, render replacement, className/style merging, `data-*` state attributes) to it; a port must reproduce `useRenderElement` semantics before fieldset can match.
- `internals/types` — `BaseUIComponentProps` type only (`packages/react/src/fieldset/root/FieldsetRoot.tsx:4`, `packages/react/src/fieldset/legend/FieldsetLegend.tsx:5`; defined at `packages/react/src/internals/types.ts:36`).
- `utils/useRegisteredLabelId` (`packages/react/src/fieldset/legend/FieldsetLegend.tsx:6`; `packages/react/src/utils/useRegisteredLabelId.ts`).
- Local: `root/FieldsetRootContext`, shared by both parts and imported across unit boundaries by Field/RadioGroup (`packages/react/src/fieldset/root/FieldsetRoot.tsx:3`, `packages/react/src/fieldset/legend/FieldsetLegend.tsx:4`).

Transitive (via `useRenderElement`, from its import block at `packages/react/src/internals/useRenderElement.tsx:2-11`): `@base-ui/utils/useMergedRefs`, `@base-ui/utils/getReactElementRef`, `@base-ui/utils/mergeObjects`, `@base-ui/utils/warn`, `@base-ui/utils/empty`, `internals/getStateAttributesProps`, `utils/resolveClassName`, `utils/resolveStyle`, `merge-props` (`mergeProps`/`mergePropsN`/`mergeClassNames`).

Transitive (via `useRegisteredLabelId`): `@base-ui/utils/useIsoLayoutEffect` (`packages/react/src/utils/useRegisteredLabelId.ts:3`) and `internals/useBaseUiId` (`packages/react/src/utils/useRegisteredLabelId.ts:4`), which itself wraps `@base-ui/utils/useId` (`packages/react/src/internals/useBaseUiId.ts:2`).

Not used: `floating-ui-react`, `use-render`, portal utilities, `useControlled`, `useStableCallback`, `useTimeout` — this unit is minimal.

Reverse dependencies (units importing fieldset internals; relevant for ordering):

- `packages/react/src/field/root/FieldRoot.tsx:10` (context hook, `disabled`).
- `packages/react/src/radio-group/RadioGroup.tsx:15` (context hook, `disabled` implied + `legendId`).

`FieldsetRootContext` is therefore a monorepo-internal seam: the optional/required overload contract (`packages/react/src/fieldset/root/FieldsetRootContext.ts:12-14`) and the `{ legendId, setLegendId, disabled }` shape must survive any reimplementation, or Field and RadioGroup break independently of fieldset's own tests.

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit; not papered over:

1. **The fieldset element's own `data-disabled` attribute is untested.** Root's `state: { disabled }` (`packages/react/src/fieldset/root/FieldsetRoot.tsx:30-32`) renders `data-disabled=""` on the `<fieldset>` itself via `getStateAttributesProps` (`packages/react/src/internals/getStateAttributesProps.ts:24-28`), but behavior.md's `data-disabled` assertions target nested `Field.Root`/`Checkbox.Root`/`Slider.Control` — never the fieldset element. Same for the **legend's own `data-disabled`** (`packages/react/src/fieldset/legend/FieldsetLegend.tsx:24-26`): no test asserts it. Fixtures must decide whether these attributes are contractual.
2. **User-prop override of internally managed props is untested.** `elementProps` sits last in both props arrays (`packages/react/src/fieldset/root/FieldsetRoot.tsx:37-43`, `packages/react/src/fieldset/legend/FieldsetLegend.tsx:31`) and `mergePropsN` is rightmost-wins (`packages/react/src/merge-props/mergeProps.ts:14-15`), so a user-supplied `aria-labelledby`, `disabled`, or `id` silently overrides the association/disabled plumbing. Behavior.md's prop-forwarding evidence uses only non-colliding props (`lang`, `data-foobar`, `style`). Whether the override wins is an undocumented contract a port must decide on.
3. **`legendId` in context has no fieldset-side consumer or test.** Its only reader is RadioGroup (`packages/react/src/radio-group/RadioGroup.tsx:191`), covered by radio-group's tests, not this unit's (behavior.md's *Edge cases* already notes legend-association tests are single-legend). The association is load-bearing beyond this unit.
4. **The stale-cleanup guard in `useRegisteredLabelId` is untested.** The cleanup withdraws the id only `if (currentId === id)` (`packages/react/src/utils/useRegisteredLabelId.ts:14-16`), which protects against an old legend's cleanup clobbering a new legend's registration when legends are swapped/replaced — precisely the *multiple legends in one fieldset* case behavior.md marks UNVERIFIED.
5. **The React-17 `useId` fallback path is untested.** `useId` falls back to a global-counter id assigned inside a client-only effect (`packages/utils/src/useId.ts:8-22`), meaning the SSR flow (legend id present pre-hydration) holds only on the `React.useId` branch (`packages/utils/src/useId.ts:32-37`) that behavior.md's SSR suite exercises. The fallback branch's SSR behavior differs (no id until after mount) and nothing in this unit's tests distinguishes the two.
