# Separator implementation spec

Unit source files: `packages/react/src/separator/index.ts`, `packages/react/src/separator/Separator.tsx`,
`packages/react/src/separator/SeparatorDataAttributes.ts`. Companion to `behavior.md` (WHAT); this
document covers WHY/HOW. The entire component is a synchronous prop-to-DOM projection: no state
machine, no effects, no context, no portal. All mechanics live in one shared hook,
`useRenderElement`, so most of this spec is about what that hook does on Separator's behalf.

## State machine / hooks used

- `React.forwardRef(function SeparatorComponent(...))` — `packages/react/src/separator/Separator.tsx:12-15`.
  The body is a single synchronous render function. There is no `useState`, `useReducer`,
  `useEffect`, `useMemo`, or `useCallback` anywhere in the unit, which is exactly why
  behavior.md's "State model" observes no internal state machine and no transitions.
- Prop destructure with default — `packages/react/src/separator/Separator.tsx:16`:
  `const { className, render, orientation = 'horizontal', style, ...elementProps } = componentProps`.
  This line is the source-level answer to the default that behavior.md's "State model" flagged
  UNVERIFIED: `orientation` is always defined, so `aria-orientation` and `data-orientation` are
  always fully rendered (never omitted).
- The only "state": `const state: SeparatorState = { orientation }` —
  `packages/react/src/separator/Separator.tsx:18`. A plain object recreated every render, no
  memoization (cheap leaf). It is a projection of props, not a store, and it feeds four
  downstream consumers inside the pipeline:
  1. `data-orientation` emission (via state-attributes mapping, below),
  2. function-form `className` (`(state) => string`),
  3. function-form `style` (`(state) => CSSProperties`),
  4. the second argument of a function-form `render` (`render(props, state)`).
- The single real hook: `useRenderElement('div', componentProps, {...})` —
  `packages/react/src/separator/Separator.tsx:20-24`. Separator passes:
  - `state` (the object above),
  - `ref: forwardedRef` (the forwardRef payload),
  - `props: [{ role: 'separator', 'aria-orientation': orientation }, elementProps]`
    (`packages/react/src/separator/Separator.tsx:23`) — library-intrinsic attributes first,
    user DOM props last (precedence implications under "DOM/portal strategy").
- Inside `useRenderElement` (`packages/react/src/internals/useRenderElement.tsx:22-48`):
  - `useRenderElementProps(...)` (`packages/react/src/internals/useRenderElement.tsx:38,53-119`)
    does the prop algebra:
    - `resolveClassName` / `resolveStyle` (`packages/react/src/internals/useRenderElement.tsx:73-74`;
      pure functions at `packages/react/src/utils/resolveClassName.ts:8-13` and
      `packages/react/src/utils/resolveStyle.ts:8-13`) — string forms pass through; function
      forms are invoked with the `{ orientation }` state. This is the mechanism behind
      behavior.md's "Public API surface" className-merging claims.
    - `getStateAttributesProps(state)` (`packages/react/src/internals/useRenderElement.tsx:76-78`)
      generically maps state to data attributes: a truthy non-boolean value becomes
      `data-<key> = String(value)` (`packages/react/src/internals/getStateAttributesProps.ts:24-28`).
      `{ orientation: 'horizontal' }` → `data-orientation="horizontal"`. Separator passes no
      `stateAttributesMapping`, so the default mapping applies.
    - The one actual React hook call in the whole pipeline: `useMergedRefs`/`useMergedRefsN`
      (`packages/react/src/internals/useRenderElement.tsx:95-104`) — merges (a) any ref already
      attached to the render element (read via `getReactElementRef`,
      `packages/react/src/internals/useRenderElement.tsx:100,102`),
      (b) the forwarded ref. Client-only, guarded by `typeof document !== 'undefined'` with a
      dummy call on the disabled/server branch to keep hook order stable
      (`packages/react/src/internals/useRenderElement.tsx:96-98`). This merged ref is what lets both the caller and the render
      element observe the same node (behavior.md "DOM structure & portal behavior").
    - `outProps = mergeObjects(stateProps, resolvedProps)` (`packages/react/src/internals/useRenderElement.tsx:85-87`)
      — later argument wins (`mergeObjects` is `{...a, ...b}`), so the intrinsic+user props layer
      overrides the state-derived data-attributes layer on conflict. For Separator there is no
      conflict in the default path (`data-orientation` vs `role`/`aria-orientation`/user props
      are disjoint keys), but a user-supplied `data-orientation` would win over the
      state-derived one.
  - `unwrapLazyRenderProp(renderProp)` (`packages/react/src/internals/useRenderElement.tsx:35,145-156`)
    — normalizes React Flight lazy wrappers so `.props`/`.ref` are readable. Inert for typical
    Separator usage.
  - `evaluateRenderProp` (`packages/react/src/internals/useRenderElement.tsx:44-47,158-206`):
    - function `render` → called as `render(props, state)` (`packages/react/src/internals/useRenderElement.tsx:169`);
    - JSX `render` → `mergeProps(props, render.props)`, then `mergedProps.ref = props.ref`
      (the merged ref replaces the element's own), then `React.cloneElement(render, mergedProps)`
      (`packages/react/src/internals/useRenderElement.tsx:172-196`);
    - no `render` → `renderTag('div', props)` (`packages/react/src/internals/useRenderElement.tsx:198-201,232-240`).
  - Dev-only guards in the hook: uppercase-named render-function warning
    (`packages/react/src/internals/useRenderElement.tsx:208-230`) and the invalid-element error
    thrown before `cloneElement` (`packages/react/src/internals/useRenderElement.tsx:182-194`).

## Context providers/consumers

- None. No React context is created or consumed anywhere in the unit:
  `packages/react/src/separator/Separator.tsx` imports nothing context-related, and
  `packages/react/src/separator/index.ts:1-3` exports only the component and its types
  (`export { Separator }` + `export type *`). There is no Root/Part/Context split — the unit is a
  single-file leaf, unlike composite components.
- State crosses no provider boundary. It reaches the DOM as `data-orientation` (state-attributes
  machinery above) and reaches custom render code only through the `render` function's
  `(props, state)` signature (`packages/react/src/internals/useRenderElement.tsx:169`).
- `SeparatorProps` / `SeparatorState` and the `Separator` namespace
  (`packages/react/src/separator/Separator.tsx:29-47`) are type-only exports so other units/docs
  can reference the state shape; they create no runtime coupling.

## DOM/portal strategy and why

- Default tag `'div'` is hardcoded as the first `useRenderElement` argument
  (`packages/react/src/separator/Separator.tsx:20`). The element renders in place — no portal,
  no wrapper, no fragment. A separator is a static leaf with no floating/overlay/positioning
  needs, so there is nothing to portal and no reason to add DOM; this is why behavior.md's
  "DOM structure & portal behavior" observes a single in-place element under every render
  customization.
- Prop precedence follows `mergePropsN` Object.assign semantics where the rightmost source wins
  for scalar props (`packages/react/src/merge-props/mergeProps.ts:14-15,99-115`), applied to
  Separator's props array by `resolveRenderFunctionProps`
  (`packages/react/src/internals/useRenderElement.tsx:80,121-129`):
  - `role="separator"` and `aria-orientation` are library-intrinsic (leftmost), so they are
    always present — matching behavior.md's "Accessibility" — but a user-supplied `role` or
    `aria-orientation` in `elementProps` (rightmost) overrides them.
  - `className` and `style` merge rather than overwrite
    (`packages/react/src/merge-props/mergeProps.ts:166-176`), which is what produces the
    function-className + render-element className coexistence in behavior.md.
  - Event handlers compose right-to-left with the `preventBaseUIHandler` protocol
    (`packages/react/src/merge-props/mergeProps.ts:17-20,177-184,221-250`), though Separator
    contributes no handlers itself, so the composed set is exactly the user's.
- `renderTag` special-cases `button` (`type="button"`) and `img` (`alt=""`)
  (`packages/react/src/internals/useRenderElement.tsx:232-240`) but is unreachable for
  Separator: the tag is fixed `'div'`, and a custom element goes through `cloneElement`, not
  `renderTag`.

## Dependencies on other Base UI internals

Direct imports of the unit:

- `../internals/useRenderElement` (`packages/react/src/separator/Separator.tsx:4`) — the entire
  render/merge/ref machinery; nothing else does any work.
- `../internals/types` (`packages/react/src/separator/Separator.tsx:3`) —
  `BaseUIComponentProps<'div', SeparatorState>` (`packages/react/src/internals/types.ts:36`)
  supplies the `className`/`render`/`style`/DOM-props/ref prop typing, and `Orientation`
  (`packages/react/src/internals/types.ts:97`) types the `orientation` prop.

Transitive dependencies (via `useRenderElement`) that a port must replicate to match this unit:

- `@base-ui/utils/useMergedRefs` (`useMergedRefs`, `useMergedRefsN`) —
  `packages/react/src/internals/useRenderElement.tsx:2`
- `@base-ui/utils/getReactElementRef` — `packages/react/src/internals/useRenderElement.tsx:3`
- `@base-ui/utils/mergeObjects` — `packages/react/src/internals/useRenderElement.tsx:4`
- `@base-ui/utils/warn` — `packages/react/src/internals/useRenderElement.tsx:5`
- `@base-ui/utils/empty` (`EMPTY_OBJECT`) — `packages/react/src/internals/useRenderElement.tsx:6`
- `internals/getStateAttributesProps` — `packages/react/src/internals/useRenderElement.tsx:8`
- `utils/resolveClassName`, `utils/resolveStyle` —
  `packages/react/src/internals/useRenderElement.tsx:9-10`
- `merge-props` (`mergeProps`, `mergePropsN`, `mergeClassNames`) —
  `packages/react/src/internals/useRenderElement.tsx:11`

Explicitly not used: `floating-ui-react`, `use-render`, portal/containment utilities,
`useControlled`, `useIsoLayoutEffect`, `useStableCallback`, `useTimeout`, `useAnimationFrame`.
The unit's `TODO.md` entry has no `wraps-external:` field (`TODO.md:506-512`), so there is no
external-package delegation to record — everything above is in-repo.

## Anything in source not explained by any test

1. Default `orientation = 'horizontal'` (`packages/react/src/separator/Separator.tsx:16`, JSDoc
   `@default` at `packages/react/src/separator/Separator.tsx:32`): no separator test asserts the
   resulting default `aria-orientation="horizontal"` — behavior.md's "State model" flags it
   UNVERIFIED. Source confirms the default exists; a fixture should pin it.
2. `data-orientation` is emitted on every render by the state-attributes machinery
   (`packages/react/src/internals/useRenderElement.tsx:76-78` +
   `packages/react/src/internals/getStateAttributesProps.ts:24-28`) and is the attribute
   documented by `packages/react/src/separator/SeparatorDataAttributes.ts:1-5`, but no separator
   test asserts it anywhere (behavior.md documents only `aria-orientation`). The golden-fixture
   stage should pin `data-orientation` alongside `aria-orientation` for both orientations.
3. `SeparatorDataAttributes.ts` is not imported by any runtime code under `packages/` (a repo
   grep for `SeparatorDataAttributes` matches only the file itself). It is a consumer-facing
   constant/type anchor for styling selectors, exercised by no test and no runtime path.
4. User-override semantics are untested: because `elementProps` sit last in the props array
   (`packages/react/src/separator/Separator.tsx:23`), a user-supplied `role` or
   `aria-orientation` silently replaces the library values, and a user-supplied
   `data-orientation` beats the state-derived one (precedence chains in
   `packages/react/src/internals/useRenderElement.tsx:85-87` and
   `packages/react/src/merge-props/mergeProps.ts:177-184`). No test covers any override case.
5. Dev-only render-prop guards belong to the hook, not this unit: the uppercase-function-name
   warning (`packages/react/src/internals/useRenderElement.tsx:208-230`) and the
   invalid-element error (`packages/react/src/internals/useRenderElement.tsx:182-194`) are never
   exercised by separator's tests (they are covered by `useRenderElement`'s own suite, which is
   out of this unit's scope). Listed so the backward-looking audit doesn't count them as
   separator gaps.
6. `'use client'` (`packages/react/src/separator/Separator.tsx:1`) is an RSC boundary marker with
   no runtime behavior; fixtures should not probe it.
