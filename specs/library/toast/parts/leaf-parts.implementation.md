# Toast leaf parts — implementation spec (Action, Arrow, Close, Content, Description, Title, isRenderableNode, useToastLabelPart)

Scope: this spec explains WHY/HOW the documented behavior of `Toast.Action`, `Toast.Arrow`, `Toast.Close`, `Toast.Content`, `Toast.Description`, `Toast.Title`, `isRenderableNode`, and `useToastLabelPart` is produced. Evidence is the batch source files: `packages/react/src/toast/action/ToastAction.tsx`, `packages/react/src/toast/action/ToastActionDataAttributes.ts`, `packages/react/src/toast/arrow/ToastArrow.tsx`, `packages/react/src/toast/arrow/ToastArrowDataAttributes.ts`, `packages/react/src/toast/close/ToastClose.tsx`, `packages/react/src/toast/close/ToastCloseDataAttributes.ts`, `packages/react/src/toast/content/ToastContent.tsx`, `packages/react/src/toast/content/ToastContentDataAttributes.ts`, `packages/react/src/toast/description/ToastDescription.tsx`, `packages/react/src/toast/description/ToastDescriptionDataAttributes.ts`, `packages/react/src/toast/title/ToastTitle.tsx`, `packages/react/src/toast/title/ToastTitleDataAttributes.ts`, `packages/react/src/toast/utils/isRenderableNode.ts`, `packages/react/src/toast/utils/useToastLabelPart.ts` — plus the imports they pull in. Behavior ground truth is `specs/library/toast/parts/leaf-parts.md` (referenced below by section name, not re-described).

## State machine / hooks used

There is no controlled/uncontrolled state machine inside any leaf part — all leaf state is derived from context, except one local bit in Close. The composition per part:

### Context hooks

- `useToastRootContext()` — consumed by `ToastAction.tsx:28`, `ToastClose.tsx:29`, `ToastContent.tsx:20`, and `useToastLabelPart.ts:18` (which is invoked by Title at `ToastTitle.tsx:26` and Description at `ToastDescription.tsx:27`). The context value shape is `{ toast, setTitleId, setDescriptionId, visibleIndex, expanded, recalculateHeight }` (`packages/react/src/toast/root/ToastRootContext.ts:5-12`); the hook throws when absent (`ToastRootContext.ts:16-23`).
- `useToastProviderContext()` — consumed only by Close (`ToastClose.tsx:28`). It returns the `ToastStore` itself (`packages/react/src/toast/provider/ToastProviderContext.ts:3-7`), which is why Close can call `store.closeToast(toast.id)` directly (`ToastClose.tsx:49`).
- `useToastPositionerContext()` — consumed only by Arrow (`ToastArrow.tsx:20`); returns the positioner's slice `{ side, align, arrowRef, arrowUncentered, arrowStyles }` picked from the anchor-positioning return type (`packages/react/src/toast/positioner/ToastPositionerContext.ts:5-8`), and throws when missing (`ToastPositionerContext.ts:14-21`).

### Primitive and internal hooks

- `React.useState` — the batch's only local state: `hasFocus` in Close (`ToastClose.tsx:31`), toggled by `onFocus`/`onBlur` handlers (`ToastClose.tsx:51-56`). It exists solely to keep a focused Close button in the accessibility tree (see below).
- `React.useRef` — `contentRef` in Content (`ToastContent.tsx:22`), the observation target for the height-recalculation effect.
- `useButton({ disabled, native })` — Action (`ToastAction.tsx:32-35`) and Close (`ToastClose.tsx:33-36`); both default `nativeButton = true` (`ToastAction.tsx:24`, `ToastClose.tsx:24`, typed by `NativeButtonProps` at `packages/react/src/internals/types.ts:63-71`). In native mode `getButtonProps` injects `type: 'button'`; in non-native mode `role: 'button'` (`packages/react/src/internals/use-button/useButton.ts:226`), and it wraps `onClick` so a disabled button swallows clicks (`useButton.ts:102-108`). It returns `buttonRef` alongside the getter (`useButton.ts:234-242`).
- `useRenderElement(tag, componentProps, params)` — used by all six components: Action `ToastAction.tsx:41-52`, Arrow `ToastArrow.tsx:28-32`, Close `ToastClose.tsx:42-61`, Content `ToastContent.tsx:51-55`, Description `ToastDescription.tsx:31-35`, Title `ToastTitle.tsx:30-34`. It resolves `render`/`className`/`style` from `componentProps` (including lazy-unwrapping, `packages/react/src/internals/useRenderElement.tsx:145-156`), turns `params.state` into `data-*` attributes via `getStateAttributesProps` (`useRenderElement.tsx:76-78`), merges `params.ref` (array refs merge via `useMergedRefsN`, `useRenderElement.tsx:99-103`, which also picks up the render element's own ref), and merges `params.props` left-to-right through `mergePropsN` (`useRenderElement.tsx:121-129` → `packages/react/src/merge-props/mergeProps.ts:99-115`). For a render function the accumulated props are passed to it (`useRenderElement.tsx:158-170`); for a render element they are `cloneElement`-merged (`useRenderElement.tsx:172-196`); for the default tag, `<button>` is special-cased to also emit `type="button"` at the JSX level (`useRenderElement.tsx:232-235`), independent of `useButton`.
- `useId(idProp)` — `useToastLabelPart.ts:23`; user-supplied `id` wins over the generated one (`packages/utils/src/useId.ts:32-41`).
- `useIsoLayoutEffect` — two uses: Content's observer setup (`ToastContent.tsx:24-42`) and label-part id registration (`useToastLabelPart.ts:40-49`).

### State → data-attribute mapping

Each part defines a minimal state object that becomes its public data attributes:

- Action/Close: `{ type: toast.type }` (`ToastAction.tsx:37-39`, `ToastClose.tsx:38-40`) → `data-type` (`ToastActionDataAttributes.ts:5`, `ToastCloseDataAttributes.ts:5`, same constant in `ToastTitleDataAttributes.ts:5`, `ToastDescriptionDataAttributes.ts:5`).
- Arrow: `{ side, align, uncentered: arrowUncentered }` (`ToastArrow.tsx:22-26`) → `data-side`/`data-align` reuse `CommonPopupDataAttributes` (`ToastArrowDataAttributes.ts:7,12`, defined at `packages/react/src/utils/CommonPopupDataAttributes.ts:32,36`) and a toast-specific `data-uncentered` (`ToastArrowDataAttributes.ts:16`).
- Content: `{ expanded, behind }` (`ToastContent.tsx:46-49`) → `data-expanded`/`data-behind` (`ToastContentDataAttributes.ts:5,10`).

### Props-merge order (the "how" of precedence)

`mergePropsN` is rightmost-wins for plain props; event handlers chain rightmost-first with `preventBaseUIHandler` escape (`mergeProps.ts:153-188`, `mergeProps.ts:221-250`).

- Action merges `[elementProps, toast.actionProps, getButtonProps, { children: computedChildren }]` (`ToastAction.tsx:44-51`). `computedChildren = toast.actionProps?.children ?? elementProps.children` (`ToastAction.tsx:30`) — toast-object children deliberately beat JSX children. The trailing `{ children }` slot is load-bearing: `getButtonProps` is a props-getter that re-applies non-extracted external props last (`useButton.ts:91-232`, getter resolution at `mergeProps.ts:126-131`), so without the trailing slot the earlier children sources would be re-imposed after the button baseline.
- Close merges `[internal defaults, elementProps, getButtonProps]` (`ToastClose.tsx:45-60`): the internal `aria-hidden`/`onClick`/`onFocus`/`onBlur` are the leftmost layer, so user `elementProps` override plain props and chain handlers (user `onClick` runs first; the internal close still fires unless `event.preventBaseUIHandler()` is called). `toast.actionProps` is deliberately *not* spread on Close — action data belongs to Action only.
- Title/Description pass a single object `{ ...elementProps, id, children }` (`ToastTitle.tsx:33`, `ToastDescription.tsx:34`) — `id`/`children` are destructured out of props (`ToastTitle.tsx:17-24`, `ToastDescription.tsx:18-25`) and re-injected last, so the `useToastLabelPart` resolution always governs them.
- Arrow merges `[{ style: arrowStyles, 'aria-hidden': true }, elementProps]` (`ToastArrow.tsx:31`) — positioning styles are leftmost so consumer styles win, but `aria-hidden` is overridable by the user since `elementProps` is rightmost.
- Content spreads only `elementProps` (`ToastContent.tsx:54`).

### Content suppression (two shapes)

1. **Post-render gate**: Action computes the final element first, then returns `hasRenderableChildren(element) ? element : null` (`ToastAction.tsx:54`). Title/Description do the same via `useToastLabelElement` (`useToastLabelPart.ts:38,51`). Because the gate inspects the *merged* element's `props.children` (`isRenderableNode.ts:13-18`), a `render` element counts as renderable only if the merged props gave it children (a childless styling-only `render` stays suppressed), and a render function returning `null` fails `React.isValidElement` → renders nothing. The effect inside `useToastLabelElement` also skips registration when suppressed (`useToastLabelPart.ts:41-43`), which is what removes `aria-labelledby`/`aria-describedby` when content disappears (behavior spec, "Accessibility (roles, aria-*, id linking)").
2. **No gate**: Arrow, Close, and Content never self-suppress — they always render.

### Content height-recalculation loop

Content registers itself as the measurement node: `recalculateHeight()` runs once on mount (`ToastContent.tsx:25`), then a `ResizeObserver` and a `MutationObserver` (observing `childList`, `subtree`, `characterData` — `ToastContent.tsx:36`) call `recalculateHeight(true)` on any size or DOM-text change (`ToastContent.tsx:32-36`). The `true` argument is the `flushSync` flag per the context signature (`ToastRootContext.ts:11`) — mutations can happen mid-frame, so layout metrics are captured synchronously. Both observers disconnect on cleanup (`ToastContent.tsx:38-41`), and the whole effect depends only on `[recalculateHeight]` (a stable callback from Root), so the observers persist for the toast's life. There is an explicit feature guard: if `ResizeObserver`/`MutationObserver` don't exist, only the mount-time recalculation runs (`ToastContent.tsx:27-30`).

### `isRenderableNode` truthiness rules

`isRenderableNode` treats exactly `null`, `undefined`, `true`/`false`, and `''` as non-renderable; arrays recurse with `some` (so one renderable leaf makes the array renderable); everything else — including `0`, `0n`, `Number.NaN`, non-empty strings, and elements — is renderable (`isRenderableNode.ts:3-11`). This is the single predicate behind both the "no children → no render" suppression (above) and the id-registration gate.

### Label-part registration

`useToastLabelPart(idProp, childrenProp, part)` (`useToastLabelPart.ts:13-26`) is the shared hook making Title and Description structurally identical (its JSDoc says as much, `useToastLabelPart.ts:8-12`): it selects the id setter by part (`setTitleId` vs `setDescriptionId`, `useToastLabelPart.ts:20`), falls back to `toast.title`/`toast.description` when no children were passed (`useToastLabelPart.ts:21`), and generates the id with `useId` (`useToastLabelPart.ts:23`). `useToastLabelElement(element, id, setId)` (`useToastLabelPart.ts:33-37`) then runs in a layout effect: renderable → `setId(id)`, cleanup → `setId(currentId => currentId === id ? undefined : currentId)` (`useToastLabelPart.ts:40-49`). The functional-update guard is what makes an older title's unmount cleanup a no-op when a newer title already took over the registration (behavior spec, "Edge cases" unmount-ordering bullet). Using `useIsoLayoutEffect` means the registration/dispatch happens in the same commit, pre-paint, so Root's `aria-labelledby`/`aria-describedby` never paints stale.

## Context providers/consumers

Three context boundaries are crossed, and two leaves also *write back* through context:

- **ToastRootContext** (root batch) — field-by-field consumption:
  - Action reads only `toast` (`ToastAction.tsx:28`) — `toast.type` for state and `toast.actionProps` for default props/content (sourced from `ToastObject.actionProps`, `packages/react/src/toast/useToastManager.ts:88`).
  - Close reads `toast` and `expanded` (`ToastClose.tsx:29`) — `expanded` drives the `aria-hidden` computation (`ToastClose.tsx:47`).
  - Content reads `visibleIndex`, `expanded`, `recalculateHeight` (`ToastContent.tsx:20`) — `visibleIndex > 0` derives `behind` (`ToastContent.tsx:44`).
  - Title/Description (via `useToastLabelPart`) read `toast` and receive the `setTitleId`/`setDescriptionId` dispatchers (`useToastLabelPart.ts:18-20`). This is the write-back path: leaf → context → Root state → Root's `aria-labelledby`/`aria-describedby` (context shape at `ToastRootContext.ts:7-8`).
- **ToastPositionerContext** (positioner batch) — Arrow only. It both consumes positioning results (`side`, `align`, `arrowUncentered`, `arrowStyles`) and writes back: `arrowRef` is included in `ref: [forwardedRef, arrowRef]` (`ToastArrow.tsx:30`), so the positioner learns the arrow's DOM node to position it (`ToastPositionerContext.ts:5-8`).
- **ToastProviderContext** (provider/store batch) — Close only (`ToastClose.tsx:28`): a leaf deliberately skipping one level of context because closing is a store operation, not Root-owned state.

Missing-context behavior is a throw at render time in both the root hook (`ToastRootContext.ts:18-22`) and the positioner hook (`ToastPositionerContext.ts:16-20`) — matching the error messages recorded in the behavior spec's "DOM structure & portal behavior" section.

## DOM/portal strategy and why

- **No portaling in this batch.** None of the fourteen files import a portal mechanism; leaves render inline into whatever their parent (`Toast.Root`, or `Toast.Positioner` for Arrow) provides. Portaling to `document.body` is Provider/Viewport/Positioner responsibility (other batches). This keeps leaves pure element factories: `useRenderElement` output is either returned as-is or gated to `null`.
- **Default tags encode semantics**: `<button>` for Action and Close (native button semantics + `type="button"` from both `useButton` and the default-tag renderer, `useButton.ts:226` and `useRenderElement.tsx:232-235`), `<h2>` for Title and `<p>` for Description (a heading/description pair inside the region Root labels via `aria-labelledby`/`aria-describedby`), `<div>` for Content (layout/stack container) and Arrow.
- **Arrow is visually positioned and aurally hidden**: it receives `style: arrowStyles` computed by the positioner and hard-codes `aria-hidden: true` (`ToastArrow.tsx:31`) because it is purely decorative — the behavior spec's "Accessibility" section asserts this attribute. The `data-side`/`data-align`/`data-uncentered` attributes let CSS flip/adjust the arrow per side (`ToastArrowDataAttributes.ts:7-16`).
- **Close's conditional `aria-hidden`**: `'aria-hidden': !expanded && !hasFocus` (`ToastClose.tsx:47`) removes close buttons from the accessibility tree while the stack is collapsed (back toasts are visually hidden in the collapsed stack), but a keyboard-focused Close stays exposed via the `hasFocus` state handlers (`ToastClose.tsx:51-56`) — the source-level reason the close flow works while the viewport holds focus (behavior spec, "Focus management").
- **Content is the measurement boundary**: it is the node whose size feeds the stack layout. It attaches no positioning of its own; it just registers `contentRef` (`ToastContent.tsx:52`) and pushes height changes upward through `recalculateHeight`, making the leaf-to-root data flow inverted relative to typical context usage.

## Dependencies on other Base UI internals

Cross-batch toast imports (cite only; internals belong to their own specs):

- `packages/react/src/toast/root/ToastRootContext` — consumed by Action, Close, Content, and `useToastLabelPart` (`ToastAction.tsx:4`, `ToastClose.tsx:4`, `ToastContent.tsx:5`, `useToastLabelPart.ts:5`).
- `packages/react/src/toast/positioner/ToastPositionerContext` — Arrow (`ToastArrow.tsx:3`).
- `packages/react/src/toast/provider/ToastProviderContext` — Close (`ToastClose.tsx:5`).
- `packages/react/src/toast/useToastManager` — type-only, via the root context's `toast: ToastObject<any>` (`ToastRootContext.ts:3`); leaves read `id` (`useToastManager.ts:30`), `title` (`:38`), `type` (`:43`), `description` (`:47`), `actionProps` (`:88`).

Shared internals:

- `packages/react/src/internals/useRenderElement` — all six components (`ToastAction.tsx:6`, `ToastArrow.tsx:6`, `ToastClose.tsx:7`, `ToastContent.tsx:6`, `ToastDescription.tsx:4`, `ToastTitle.tsx:4`). Internally it uses `@base-ui/utils/useMergedRefs`, `@base-ui/utils/getReactElementRef`, `@base-ui/utils/mergeObjects`, and `packages/react/src/merge-props` (`useRenderElement.tsx:2-11`).
- `packages/react/src/internals/use-button/useButton` — Action and Close (`ToastAction.tsx:5`, `ToastClose.tsx:6`). It transitively pulls `@floating-ui/utils/dom` (`isHTMLElement`, `useButton.ts:3`), `@base-ui/utils/useStableCallback`, `@base-ui/utils/useIsoLayoutEffect`, `packages/react/src/internals/composite/root/CompositeRootContext`, and `packages/react/src/utils/useFocusableWhenDisabled` (`useButton.ts:4-12`).
- `packages/react/src/internals/types` — `BaseUIComponentProps`, `NativeButtonProps` (`ToastAction.tsx:3`, `ToastArrow.tsx:4`, `ToastClose.tsx:3`, `ToastContent.tsx:4`, `ToastDescription.tsx:3`, `ToastTitle.tsx:3`; `NativeButtonProps` defined at `types.ts:63-71`).
- `packages/react/src/toast/utils/isRenderableNode` — Action and `useToastLabelPart` (`ToastAction.tsx:7`, `useToastLabelPart.ts:6`).
- `@base-ui/utils/useIsoLayoutEffect` — Content and `useToastLabelPart` (`ToastContent.tsx:3`, `useToastLabelPart.ts:4`).
- `@base-ui/utils/useId` — `useToastLabelPart` (`useToastLabelPart.ts:3`).
- `packages/react/src/utils/popupStateMapping` → `CommonPopupDataAttributes` — Arrow's data-attribute constants re-export shared `data-side`/`data-align` tokens (`ToastArrowDataAttributes.ts:1,7,12`; re-export chain at `packages/react/src/utils/popupStateMapping.ts:4,7`).
- `packages/react/src/internals/useAnchorPositioning` — type-only: `Side`/`Align` for Arrow's state (`ToastArrow.tsx:5`) and the positioner context's picked type (`ToastPositionerContext.ts:3`).

No `floating-ui-react` import appears directly in any batch file — the positioning dependency reaches Arrow only through the positioner context's values and types.

## Anything in source not explained by any test

Cross-checked against every section of `specs/library/toast/parts/leaf-parts.md`. The following source behaviors have **no coverage** in that spec's tests; the golden-fixture stage must either encode them deliberately or document them as accepted gaps:

1. **Close's entire `aria-hidden` + focus-tracking mechanism** (`ToastClose.tsx:31,47,51-56`): no test in the behavior spec asserts `aria-hidden` on Close, that it flips with viewport hover/expand, or that it stays `false` while focused. The spec's Accessibility section covers only Arrow's static `aria-hidden` (its `Toast.Arrow renders aria-hidden="true"` bullet).
2. **Content's ResizeObserver/MutationObserver wiring and the `flushSync` recalculation path** (`ToastContent.tsx:24-42`): the behavior spec tests only `data-behind`/`data-expanded`. Nothing measures height recalcs, the synchronous flag, or the observer-disconnect cleanup. The feature-detection fallback (`ToastContent.tsx:27-30`) is also untested.
3. **`disabled` and `nativeButton` on Action and Close** (`ToastAction.tsx:23-24,32-35`, `ToastClose.tsx:23-24,33-36`): no leaf test renders either part disabled or with `nativeButton={false}`, so the `role="button"`/non-native paths and `useButton`'s disabled-click suppression are entirely unexercised here.
4. **All `data-type` attributes** (Action, Close, Title, Description): the state `{ type: toast.type }` and the exported `data-type` constants exist in four files (`ToastAction.tsx:37-39` etc.), but the behavior spec contains no `data-type` assertion for any leaf. Same for Arrow's `data-uncentered` (`ToastArrowDataAttributes.ts:16`); Arrow's `data-side`/`data-align` inherit untested status at this level too (the spec's Chromium-only Arrow test concerns style side-mirroring, not attributes).
5. **Action's content precedence when both sources exist** (`ToastAction.tsx:30`): `toast.actionProps.children` beats JSX `children` by construction. The behavior spec's fixtures only ever use childless parts (its shared-harness section says all parts are childless, sourcing content from the toast object), so the conflict case is untested.
6. **`isRenderableNode` on exotic truthy values**: symbols, functions, and objects pass the predicate (`isRenderableNode.ts:3-11` returns `true` for anything not null/undefined/boolean/`''`/array), but React renders symbols as nothing and throws on objects/functions as children. The behavior spec's truthiness table stops at primitives and arrays; this mismatch is untested and the golden fixture must decide whether `isRenderableNode`'s optimism for these values is normative.
7. **`behind` derivation for stacks deeper than two** (`ToastContent.tsx:44`): the behavior spec already flags the two-toast case as its UNVERIFIED boundary; at source level `behind` is purely `visibleIndex > 0`, so any fixture with ≥3 toasts is exercising untested code.
8. **Close's user-handler chaining semantics** (`ToastClose.tsx:45-60` with `mergeProps.ts:221-250`): a consumer `onClick` runs before the internal `store.closeToast` and can suppress it via `event.preventBaseUIHandler()`. No test covers a custom `onClick` on Close, nor user overrides of Arrow's `aria-hidden`/`style` precedence (`ToastArrow.tsx:31`).
9. **Label-part registration timing** (`useToastLabelPart.ts:40-49`): the *outcomes* (link present/removed, unmount ordering) are tested per the behavior spec's Accessibility and Edge cases sections, but the layout-effect/pre-paint timing and the `currentId === id` guard's exact functional-update form are observable only through implementation details, not the asserted DOM outcomes.
