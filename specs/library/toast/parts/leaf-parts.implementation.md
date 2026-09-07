# Toast leaf parts — implementation spec (Action, Arrow, Close, Content, Description, Title, isRenderableNode, useToastLabelPart)

Ground truth for WHAT is `specs/library/toast/behavior.md` and the batch behavior spec
`specs/library/toast/parts/leaf-parts.md`; those sections are cited by name below and their
claims are not restated. The toast unit's TODO entry carries `needs-batched-mining: true` but
no `wraps-external:` field (`TODO.md:534-543`), so nothing in this batch delegates to a
third-party algorithm — all logic is first-party React.

One structural fact drives most of this batch: every part delegates element creation to the
shared `useRenderElement` factory, which merges an ordered array of props
(`packages/react/src/internals/useRenderElement.tsx:125-172`), composes event handlers and
merges `className`/`style` instead of overwriting them
(`packages/react/src/merge-props/mergeProps.ts:17-23`), merges ref arrays, and converts the
`state` object into `data-*` attributes (`packages/react/src/internals/getStateAttributesProps.ts:24-28`:
boolean `true` → attribute present with an empty value, any other truthy value → stringified,
falsy → attribute omitted). The per-part `*DataAttributes.ts` files are pure string-constant
metadata documenting those generated names — they contain no runtime logic
(`packages/react/src/toast/action/ToastActionDataAttributes.ts:5`,
`packages/react/src/toast/content/ToastContentDataAttributes.ts:5-10`); the Arrow's constants
reuse the shared popup mapping so `data-side`/`data-align` names match every other anchored
component (`packages/react/src/toast/arrow/ToastArrowDataAttributes.ts:1-12`).

## State machine / hooks used

None of these parts owns a lifecycle state machine. All persistent state lives in the
Root/Positioner contexts and the store (see "Context providers/consumers"); the parts add only
two pieces of local state: `ToastClose`'s `hasFocus` boolean and `ToastContent`'s observer
subscriptions (refs, not React state).

### `ToastAction`

- Resolves content first: `toast.actionProps?.children ?? elementProps.children`
  (`packages/react/src/toast/action/ToastAction.tsx:30`) — the toast object's `actionProps.children`
  wins when defined, JSX children are the fallback. This single expression produces both tested
  outcomes: the toast's `'Undo'` content flowing into a childless render element (leaf-parts.md
  "Public API surface", `packages/react/src/toast/action/ToastAction.test.tsx:110-122`) and
  nothing rendering when `actionProps.children` is undefined and no JSX children were passed
  (leaf-parts.md "Edge cases", `packages/react/src/toast/action/ToastAction.test.tsx:47-80`).
  The conformance suite incidentally exercises the fallback half by rendering
  `<Toast.Action>action</Toast.Action>` on a toast without `actionProps`
  (`packages/react/src/toast/action/ToastAction.test.tsx:15-28`).
- `useButton({ disabled, native: nativeButton })`
  (`packages/react/src/toast/action/ToastAction.tsx:32-35`) delegates native-button semantics to
  the shared hook; `nativeButton` defaults to `true` (`packages/react/src/toast/action/ToastAction.tsx:24`).
- State is a single field: `{ type: toast.type }` (`packages/react/src/toast/action/ToastAction.tsx:37-39`),
  which the generic mapper turns into `data-type` (`packages/react/src/toast/action/ToastActionDataAttributes.ts:5`).
- Element assembly order matters: `props: [elementProps, toast.actionProps, getButtonProps,
  { children: computedChildren }]` (`packages/react/src/toast/action/ToastAction.tsx:41-52`).
  Because plain props are last-wins while handlers compose
  (`packages/react/src/merge-props/mergeProps.ts:14-23`), `toast.actionProps` (e.g. the tested
  `id: 'action'`, leaf-parts.md "Public API surface") overrides user element props; `getButtonProps`
  then layers button semantics on top; and the final `{ children: computedChildren }` entry
  re-asserts the resolved children as the highest-priority value so the merge order above can
  never leak a different `children` into the element.
- Render gate: `hasRenderableChildren(element) ? element : null`
  (`packages/react/src/toast/action/ToastAction.tsx:54`) — the suppression mechanism documented
  in leaf-parts.md "Edge cases". Note Action does *not* destructure `children` out of props, so
  JSX children travel through `elementProps` into the merge and are also readable at line 30.

### `ToastClose`

- Dual-context consumer: the provider context (which *is* the `ToastStore` instance —
  `packages/react/src/toast/provider/ToastProviderContext.ts:5-7`) for the imperative close, and
  the root context for `toast` + `expanded`
  (`packages/react/src/toast/close/ToastClose.tsx:28-29`).
- The click handler is a one-line transition trigger: `store.closeToast(toast.id)`
  (`packages/react/src/toast/close/ToastClose.tsx:48-50`). Everything the close test observes —
  ending state, removal, `onClose`/`onRemove` timing — is store/Root behavior (behavior.md
  "State model", "Events"); this leaf is only the call site.
- `aria-hidden: !expanded && !hasFocus` (`packages/react/src/toast/close/ToastClose.tsx:47`):
  the close button is hidden from assistive technology while the viewport is collapsed, but
  flips to visible-when-focused — local `hasFocus` state is set by `onFocus`/`onBlur`
  (`packages/react/src/toast/close/ToastClose.tsx:31,51-56`) — so a keyboard user who tabs into
  a collapsed toast still gets the button announced. No test covers this (see the last section).
- User handlers compose rather than replace: `elementProps` sits after the built-in props object
  in the merge array (`packages/react/src/toast/close/ToastClose.tsx:45-60`), and `mergeProps`
  composes handlers with the user's running first, able to call `event.preventBaseUIHandler()`
  to suppress the internal close
  (`packages/react/src/merge-props/mergeProps.ts:17-23,221-250`).
- State `{ type: toast.type }` → `data-type`
  (`packages/react/src/toast/close/ToastClose.tsx:38-40`,
  `packages/react/src/toast/close/ToastCloseDataAttributes.ts:5`).

### `ToastContent`

- Consumes `{ visibleIndex, expanded, recalculateHeight }` from the root context
  (`packages/react/src/toast/content/ToastContent.tsx:20`).
- `behind = visibleIndex > 0` (`packages/react/src/toast/content/ToastContent.tsx:44`) is derived
  per render from Root-provided state (Root computes it from the store's stack order, behavior.md
  "State model"); no local state is needed. State `{ expanded, behind }`
  (`packages/react/src/toast/content/ToastContent.tsx:46-49`) maps to `data-expanded`/`data-behind`
  (`packages/react/src/toast/content/ToastContentDataAttributes.ts:5-10`) — presence/absence via
  the boolean mapping rule (`packages/react/src/internals/getStateAttributesProps.ts:24-28`).
- Height synchronization (`packages/react/src/toast/content/ToastContent.tsx:24-42`), in a
  `useIsoLayoutEffect` so the measure request lands before paint:
  1. calls `recalculateHeight()` on every (re)mount of the effect — asking Root to re-measure
     `--toast-height` (behavior.md "State model");
  2. if `ResizeObserver` and `MutationObserver` exist (guarded at
     `packages/react/src/toast/content/ToastContent.tsx:28` for environments without them), observes
     its own DOM node for size changes and for childList/subtree/characterData mutations, calling
     `recalculateHeight(true)` — the boolean is the `flushSync` parameter of the callback type
     (`packages/react/src/toast/root/ToastRootContext.ts:11`), upgrading async DOM mutations to
     synchronous re-measures;
  3. disconnects both observers on cleanup
     (`packages/react/src/toast/content/ToastContent.tsx:38-41`).
- The observed node is the real rendered element even under a render prop: a local `contentRef`
  is merged with the forwarded ref (`packages/react/src/toast/content/ToastContent.tsx:22,51-52`).

### `ToastTitle` / `ToastDescription`

These two are the same component modulo the tag (`'h2'` vs `'p'`) and the `part` argument —
stated explicitly in the shared hook's doc
(`packages/react/src/toast/utils/useToastLabelPart.ts:8-12`). Composition:

- `id` and `children` are destructured out of props
  (`packages/react/src/toast/title/ToastTitle.tsx:21-22`,
  `packages/react/src/toast/description/ToastDescription.tsx:22-23`) so they can be intercepted.
- `useToastLabelPart(idProp, childrenProp, part)`
  (`packages/react/src/toast/utils/useToastLabelPart.ts:13-26`):
  - pulls `{ toast, setTitleId, setDescriptionId }` from the root context and picks the matching
    setter by `part` (`packages/react/src/toast/utils/useToastLabelPart.ts:18-20`);
  - `children = childrenProp ?? (part === 'title' ? toast.title : toast.description)`
    (`packages/react/src/toast/utils/useToastLabelPart.ts:21`) — JSX children win, the toast
    object's field is the fallback (leaf-parts.md "Public API surface");
  - `id = useId(idProp)` (`packages/react/src/toast/utils/useToastLabelPart.ts:23`) — the
    auto-generated id the Root's `aria-labelledby`/`aria-describedby` point at; the hook honors an
    explicit `id` prop override and prefers `React.useId`
    (`packages/utils/src/useId.ts:32-41`);
  - returns `{ id, children, type: toast.type, setId }` — `type` feeding the `data-type` state
    (`packages/react/src/toast/title/ToastTitle.tsx:28`,
    `packages/react/src/toast/title/ToastTitleDataAttributes.ts:5`).
- Element is built with `props: { ...elementProps, id, children }`
  (`packages/react/src/toast/title/ToastTitle.tsx:30-34`,
  `packages/react/src/toast/description/ToastDescription.tsx:31-35`); since `id`/`children` were
  destructured out of `elementProps`, these are the authoritative values.
- `useToastLabelElement(element, id, setId)`
  (`packages/react/src/toast/utils/useToastLabelPart.ts:33-52`) is the conditional-render +
  registration wrapper:
  - `shouldRender = hasRenderableChildren(element)`
    (`packages/react/src/toast/utils/useToastLabelPart.ts:38`). This is where the documented
    suppression cases come from: a render fn returning `null` yields a null element (not a valid
    element → false, leaf-parts.md "Edge cases"); a childless styling-only render element with no
    toast field yields `children: undefined` → false; the toast field or the render element's own
    children yield true — for a childless render element, `useRenderElement` merges the resolved
    `children` in and the render element's own props merge last
    (`packages/react/src/internals/useRenderElement.tsx:172`), so a render element's own children
    count while a childless one inherits the toast field (the distinction the hook's doc states at
    `packages/react/src/toast/utils/useToastLabelPart.ts:29-31`).
  - A `useIsoLayoutEffect` registers `setId(id)` while the part renders renderable content
    (`packages/react/src/toast/utils/useToastLabelPart.ts:40-45`), so the id is in Root state
    before the browser paints and before a synchronous `act()` flush completes.
  - Cleanup uses a functional update that only clears its own id —
    `setId((currentId) => (currentId === id ? undefined : currentId))`
    (`packages/react/src/toast/utils/useToastLabelPart.ts:46-48`) — which is the mechanism behind
    the documented unmount-ordering guarantee that an older Title's cleanup must not clear a newer
    Title's registration (leaf-parts.md "Edge cases",
    `packages/react/src/toast/title/ToastTitle.test.tsx:229-261`).
  - Returns `shouldRender ? element : null`
    (`packages/react/src/toast/utils/useToastLabelPart.ts:51`).

### `ToastArrow`

- Consumes the positioner context: `{ arrowRef, side, align, arrowUncentered, arrowStyles }`
  (`packages/react/src/toast/arrow/ToastArrow.tsx:20`); the context type is a `Pick` of the shared
  anchor-positioning return value (`packages/react/src/toast/positioner/ToastPositionerContext.ts:5-8`),
  so the arrow's rendering inputs are exactly the positioning engine's outputs.
- State `{ side, align, uncentered: arrowUncentered }`
  (`packages/react/src/toast/arrow/ToastArrow.tsx:22-26`) → `data-side`/`data-align`/`data-uncentered`
  (`packages/react/src/toast/arrow/ToastArrowDataAttributes.ts:7-16`); the browser-only test
  asserts the side mirroring (leaf-parts.md "Shared harness dependencies",
  `packages/react/src/toast/arrow/ToastArrow.test.tsx:26-68`).
- Ref array `[forwardedRef, arrowRef]` (`packages/react/src/toast/arrow/ToastArrow.tsx:30`): the
  arrow's DOM node is registered *into* the Positioner's `arrowRef` so the anchor-positioning
  engine can measure it and offset/translate the positioned element around it — the arrow is an
  input to positioning, not just a styled shape. This is why Arrow must live inside
  `<Toast.Positioner>` (the documented missing-context error,
  `packages/react/src/toast/positioner/ToastPositionerContext.ts:16-20`).
- Props `[ { style: arrowStyles, 'aria-hidden': true }, elementProps ]`
  (`packages/react/src/toast/arrow/ToastArrow.tsx:31`): the engine's computed inline styles come
  first so user `style` still merges on top (`packages/react/src/merge-props/mergeProps.ts:166-172`),
  and `aria-hidden="true"` is unconditional (leaf-parts.md "Accessibility",
  `packages/react/src/toast/arrow/ToastArrow.test.tsx:69`) — the arrow is purely decorative.

### `isRenderableNode`

- `isRenderableNode` (`packages/react/src/toast/utils/isRenderableNode.ts:3-11`) rejects
  `null`/`undefined` (via `== null`), booleans, and the empty string
  (`packages/react/src/toast/utils/isRenderableNode.ts:4`); arrays recurse through
  `.some(isRenderableNode)` (`packages/react/src/toast/utils/isRenderableNode.ts:7-9`), which is
  inherently arbitrary-depth because nested arrays re-enter the predicate; everything else is
  renderable — deliberately including falsy-but-visible values `0`, `0n`, `Number.NaN`. This is
  why it exists instead of plain truthiness: `0` is valid title content (leaf-parts.md
  "Edge cases"), which naive `children ? … : null` would suppress.
- `hasRenderableChildren` (`packages/react/src/toast/utils/isRenderableNode.ts:13-18`) requires a
  valid React element whose `.props.children` are renderable
  (`packages/react/src/toast/utils/isRenderableNode.ts:15-16`); non-element inputs (null, raw
  strings, bare arrays at the top level) are false because there are no props to inspect. It is
  the shared gate used by `ToastAction` (`packages/react/src/toast/action/ToastAction.tsx:54`)
  and `useToastLabelElement` (`packages/react/src/toast/utils/useToastLabelPart.ts:38`).

## Context providers/consumers

All three contexts are defined outside this batch and imported by it; none are created here.

- `ToastRootContext` (`packages/react/src/toast/root/ToastRootContext.ts:5-14`) — provided by
  `Toast.Root`, consumed by Action (`toast`, `packages/react/src/toast/action/ToastAction.tsx:28`),
  Close (`toast`, `expanded`, `packages/react/src/toast/close/ToastClose.tsx:29`), Content
  (`visibleIndex`, `expanded`, `recalculateHeight`,
  `packages/react/src/toast/content/ToastContent.tsx:20`), and both label parts via
  `useToastLabelPart` (`toast`, `setTitleId`, `setDescriptionId`,
  `packages/react/src/toast/utils/useToastLabelPart.ts:18`). The hook throws the documented
  missing-context error (`packages/react/src/toast/root/ToastRootContext.ts:16-23`).
- `ToastPositionerContext` (`packages/react/src/toast/positioner/ToastPositionerContext.ts:10-12`)
  — provided by `Toast.Positioner`, consumed by Arrow only
  (`packages/react/src/toast/arrow/ToastArrow.tsx:20`); throws when missing
  (`packages/react/src/toast/positioner/ToastPositionerContext.ts:14-21`).
- `ToastContext` is typed as the `ToastStore` itself
  (`packages/react/src/toast/provider/ToastProviderContext.ts:5-7`) — the provider shares its
  store instance directly rather than wrapping it, which is why Close can call
  `store.closeToast` imperatively; throws the documented provider error
  (`packages/react/src/toast/provider/ToastProviderContext.ts:9-15`).

Data flows *up* from Title/Description (id registration through `setTitleId`/`setDescriptionId`
setters) and *down* to everything else (`toast`, `expanded`, `visibleIndex`,
`recalculateHeight`). The Root owns the `aria-labelledby`/`aria-describedby` attributes; the
label parts never render aria attributes themselves — they only push ids
(leaf-parts.md "Accessibility").

## DOM/portal strategy and why

- No portals anywhere in this batch. Every part renders in place under its `Toast.Root` (or
  `Toast.Positioner` for Arrow) — the canonical nesting in leaf-parts.md "DOM structure &
  portal behavior". Portaling is a shell concern (`Toast.Portal`/`Viewport`), not a leaf concern.
- Default elements carry the semantics: native `<button>` for Action and Close (both feed
  `useButton` so a `nativeButton: false` non-button element keeps button behavior,
  `packages/react/src/toast/action/ToastAction.tsx:32-35`,
  `packages/react/src/toast/close/ToastClose.tsx:33-36`), `<div>` for Content,
  `<p>` for Description, `<h2>` for Title (heading level is fixed; there is no `level` prop —
  matching the conformance `refInstanceof: window.HTMLHeadingElement`, leaf-parts.md
  "Public API surface").
- Contentless parts return `null` instead of rendering empty elements, so no inert DOM nodes
  leak for toasts lacking a title/description/action (leaf-parts.md "Edge cases").
- Arrow's only DOM coupling is its measured box: `aria-hidden="true"` plus the engine-computed
  `arrowStyles` and the `arrowRef` registration described above
  (`packages/react/src/toast/arrow/ToastArrow.tsx:30-31`).
- Content is the only part that touches the DOM imperatively — the observer pair on its own
  rendered node (`packages/react/src/toast/content/ToastContent.tsx:27-36`) — keeping
  `--toast-height` in sync with real layout rather than React state.
- Close's `aria-hidden` focus flip is the only attribute-level decision that depends on runtime
  interaction state (`packages/react/src/toast/close/ToastClose.tsx:47-56`).

## Dependencies on other Base UI internals

Imports from outside this unit, cited by path (internals not re-derived here):

- `packages/react/src/toast/root/ToastRootContext.ts` — root context hook + missing-context error.
- `packages/react/src/toast/positioner/ToastPositionerContext.ts` — positioner context hook + error.
- `packages/react/src/toast/provider/ToastProviderContext.ts` — store-backed provider context hook.
- `packages/react/src/internals/useRenderElement.tsx` — element factory: prop-array merging,
  render-prop support (element and function forms), ref-array merging, state → `data-*` mapping
  via `packages/react/src/internals/getStateAttributesProps.ts:5-31`.
- `packages/react/src/merge-props/mergeProps.ts` — `mergePropsN` semantics
  (`packages/react/src/merge-props/mergeProps.ts:99`): last-wins plain props, composed event
  handlers with `preventBaseUIHandler` (`packages/react/src/merge-props/mergeProps.ts:221-250`),
  merged `className`/`style`.
- `packages/react/src/internals/use-button/useButton.ts` — `getButtonProps`/`buttonRef`
  (`packages/react/src/internals/use-button/useButton.ts:14`); `buttonRef` is a stable callback
  (`packages/react/src/internals/use-button/useButton.ts:234`).
- `packages/react/src/internals/types` — `BaseUIComponentProps`, `NativeButtonProps` prop-type
  building blocks (`packages/react/src/toast/action/ToastAction.tsx:3`).
- `packages/react/src/internals/useAnchorPositioning.ts` — `Side`/`Align` types only (Arrow,
  `packages/react/src/toast/arrow/ToastArrow.tsx:5`).
- `packages/react/src/utils/popupStateMapping.ts` — `CommonPopupDataAttributes` shared
  `data-side`/`data-align` names (`packages/react/src/toast/arrow/ToastArrowDataAttributes.ts:1`).
- `packages/utils/src/useId.ts` — id generation for Title/Description
  (`packages/react/src/toast/utils/useToastLabelPart.ts:3`).
- `packages/utils/src/useIsoLayoutEffect.ts` — the mandated layout effect used by Content and the
  label-part registration effect (`packages/react/src/toast/content/ToastContent.tsx:3`,
  `packages/react/src/toast/utils/useToastLabelPart.ts:4`).

## Anything in source not explained by any test

- `ToastClose`'s entire `aria-hidden` strategy — `!expanded && !hasFocus` plus the
  `onFocus`/`onBlur` state pair (`packages/react/src/toast/close/ToastClose.tsx:31,47,51-56`) —
  has no coverage: no test in this batch asserts `aria-hidden` on Close in any state. The close
  test proves only click-removal (leaf-parts.md "Events",
  `packages/react/src/toast/close/ToastClose.test.tsx:30-56`).
- Handler composition on Close: no test passes `onClick`/`onFocus`/`onBlur` to `Toast.Close` to
  pin that user handlers compose with (and can suppress) the internal `closeToast`
  (`packages/react/src/toast/close/ToastClose.tsx:45-60`).
- Content's height machinery — the ResizeObserver/MutationObserver pair, the `flushSync: true`
  argument, and the missing-API guard (`packages/react/src/toast/content/ToastContent.tsx:28-36`) —
  is untested at leaf level; the observable `--toast-height` outcome is documented from
  Root/viewport tests (behavior.md "State model"), not from this file.
- Falsy-state attribute omission: tests assert `data-behind`/`data-expanded` presence (leaf-parts.md
  "State model") but not their removal on state flip — that UNVERIFIED gap (leaf-parts.md line 32)
  is filled only by the generic mapping rule
  (`packages/react/src/internals/getStateAttributesProps.ts:24-28`).
- `ToastAction`'s `disabled` and `nativeButton: false` paths
  (`packages/react/src/toast/action/ToastAction.tsx:23-24,32-35`) are never exercised by any toast
  test (conformance renders a plain `<button>` with no `disabled`).
- Arrow's `uncentered` state → `data-uncentered`
  (`packages/react/src/toast/arrow/ToastArrow.tsx:22-26`,
  `packages/react/src/toast/arrow/ToastArrowDataAttributes.ts:16`): the Chromium-only test asserts
  `data-side` and `aria-hidden` (`packages/react/src/toast/arrow/ToastArrow.test.tsx:68-69`); no
  test drives the uncentered case, and `arrowStyles`/`arrowRef` mechanics are untested at this
  level.
- `data-type` on Action/Close/Title/Description (state `{ type: toast.type }`,
  `packages/react/src/toast/action/ToastAction.tsx:37-39`,
  `packages/react/src/toast/close/ToastClose.tsx:38-40`,
  `packages/react/src/toast/title/ToastTitle.tsx:28`) is documented by the per-part
  `*DataAttributes.ts` files but asserted by no test.
- Action's JSX-children fallback half (`toast.actionProps?.children ?? elementProps.children`,
  `packages/react/src/toast/action/ToastAction.tsx:30`): actionProps-children-wins is pinned
  (`packages/react/src/toast/action/ToastAction.test.tsx:110-122`) and the childless-render case is
  pinned (`packages/react/src/toast/action/ToastAction.test.tsx:96-108`), but no test renders
  `<Toast.Action>JSX children</Toast.Action>` on a toast whose `actionProps.children` is defined,
  so the precedence of toast content over explicit JSX children is unpinned.
- Explicit `id` prop override on Title/Description (`useId(idProp)`,
  `packages/react/src/toast/utils/useToastLabelPart.ts:23`) is untested — all tests rely on
  auto-generated ids (leaf-parts.md "Accessibility").
- Array recursion depth beyond two levels in `isRenderableNode` is UNVERIFIED (leaf-parts.md
  line 85); the implementation handles arbitrary depth via re-entrant `.some`
  (`packages/react/src/toast/utils/isRenderableNode.ts:7-9`).
