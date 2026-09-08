# `types` — behavior spec

This unit has no test files (`hasTests: false` in `ralph/generated/components.json`) — every claim below is derived directly from source, not confirmed by a test.

Unit: `infra: types` (`TODO.md:274-279`, target crate `leptos-types`). The TODO entry has no `wraps-external:` field, so no third-party package's internals are treated as given and no external delegation applies. The unit's entire source is a single 26-line, type-only module (`packages/react/src/types/index.ts`), publicly surfaced via the package barrel (`packages/react/src/index.ts:44`, `export type * from './types'`). It declares no runtime code: no components, no hooks, no DOM. Everything below is therefore the *type-level* contract other units consume; runtime sections that have no type-level counterpart are N/A.

## Public API surface (props, parts, subcomponents)

Four exports, all types (no values are exported anywhere in the file):

- `BaseUIChangeEventDetails` and `BaseUIGenericEventDetails` are re-exported verbatim from the internals module `'../internals/createBaseUIEventDetails'` (`packages/react/src/types/index.ts:3-6`). The change-details type is the "details" object Base UI change events carry: `reason` (the reason string), `event` (the native event, typed per-reason via `ReasonToEvent`), `cancel()` to stop Base UI's internal handling, `allowPropagation()` to let the event propagate where Base UI would stop it, `isCanceled`/`isPropagationAllowed` indicators, and `trigger` (the element that triggered the event, if any), intersected with caller-supplied `CustomProperties` (`packages/react/src/internals/createBaseUIEventDetails.ts:56-85`, exported at `packages/react/src/internals/createBaseUIEventDetails.ts:90-93`). The generic-details type carries only `reason` + `event` plus `CustomProperties` (`packages/react/src/internals/createBaseUIEventDetails.ts:98-112`). Both are distributive conditionals: a non-`string` `Reason` resolves the whole type to `never` (`packages/react/src/internals/createBaseUIEventDetails.ts:93`, `packages/react/src/internals/createBaseUIEventDetails.ts:112`). The companion runtime factories (`createChangeEventDetails`/`createGenericEventDetails`) live in the internals unit and are deliberately NOT re-exported here — this module is the type surface only (`packages/react/src/internals/createBaseUIEventDetails.ts:118-166`).
- `HTMLProps<T = any>` is `React.HTMLAttributes<T> & { ref?: React.Ref<T> | undefined }` (`packages/react/src/types/index.ts:8-10`) — i.e. React's event/attribute prop set plus an explicit, always-allowed `ref`, with the element type defaulting to `any`.
- `ComponentRenderFn<Props, State>` is the render-prop shape: a function from (props to spread on the rendered element, component state) to a `React.ReactElement<unknown>` (`packages/react/src/types/index.ts:12-21`, JSDoc at `packages/react/src/types/index.ts:13-17`). There is no default for either type parameter.
- `BaseUIEvent<E extends React.SyntheticEvent<Element, Event>>` augments any synthetic event with `preventBaseUIHandler: () => void` and a `readonly baseUIHandlerPrevented?: boolean | undefined` flag (`packages/react/src/types/index.ts:23-26`). This is Base UI's counterpart to React's `preventDefault` (see Events below).

No props objects, parts, or subcomponents exist: the unit renders nothing and defines no component types of its own (the only component-shaped thing it contributes is the `ComponentRenderFn` contract used by every part's `render` prop).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — a type-only module has no state, controlled/uncontrolled props, or transitions (`packages/react/src/types/index.ts:1-26` contains only `export type` declarations and one type-only import). The only "defaults" are type-level: `HTMLProps`' element parameter defaults to `any` (`packages/react/src/types/index.ts:8`), and the details types' `CustomProperties` parameter defaults to `{}` (`packages/react/src/internals/createBaseUIEventDetails.ts:90-93`, `packages/react/src/internals/createBaseUIEventDetails.ts:109-112`).

## Keyboard interactions

N/A — the unit binds no event listeners and has no runtime interaction surface (`packages/react/src/types/index.ts:1-26`). `KeyboardEvent` appears only as a member of the native-event unions inside the re-exported types' `ReasonToEventMap` (e.g. `triggerPress`, `escapeKey`, `listNavigation`) (`packages/react/src/internals/createBaseUIEventDetails.ts:4-47`), which merely types the `event` field for keyboard-initiated reasons.

## Focus management

N/A — no focus behavior exists in a type-only module (`packages/react/src/types/index.ts:1-26`). `FocusEvent` likewise appears only in the reason→event map (`triggerHover`-adjacent reasons such as `triggerFocus`, `inputClear`, `inputBlur`, `focusOut`) (`packages/react/src/internals/createBaseUIEventDetails.ts:9-27`).

## Accessibility (roles, aria-*, id linking)

N/A — the unit declares no roles, `aria-*` attributes, or id-linking of its own. Indirectly, every component's prop surface inherits ARIA attribute typing because `HTMLProps` extends `React.HTMLAttributes<T>` (`packages/react/src/types/index.ts:8-10`).

## DOM structure & portal behavior

N/A — the unit renders no DOM and portals nothing (`packages/react/src/types/index.ts:1-26`). The closest DOM-adjacent contract is the `trigger?: Element | undefined` field on change-event details (`packages/react/src/internals/createBaseUIEventDetails.ts:82-84`), which types a reference to an existing element rather than creating one.

## Events (names, payload shape, bubbling, preventDefault semantics)

- The `BaseUIEvent` augmentation is the unit's core behavioral contract: `preventBaseUIHandler()` marks the event as "do not run Base UI's own handler", and `baseUIHandlerPrevented` exposes that mark to other internal handlers (`packages/react/src/types/index.ts:23-26`). It is semantically distinct from React's `preventDefault` — consumer source confirms the division of labor: `mergeProps` invokes Base UI's internal handlers only when the flag is NOT set (`packages/react/src/merge-props/mergeProps.ts:239`), documents that unrelated handlers must check `event.baseUIHandlerPrevented` themselves and bail out if true (`packages/react/src/merge-props/mergeProps.ts:31`), and `useButton` bails out of its press/click handling when the flag is set (`packages/react/src/internals/use-button/useButton.ts:123`, `packages/react/src/internals/use-button/useButton.ts:198`).
- Producers call `preventBaseUIHandler()` on the event object to set the mark — e.g. useButton (`packages/react/src/internals/use-button/useButton.ts:147`), CheckboxRoot (`packages/react/src/checkbox/root/CheckboxRoot.tsx:329`) — and readers that receive plain React events treat the flag as optional: `MaybeBaseUIEvent` picks both members as `Partial`, so an un-augmented handler's event still satisfies the type (`packages/react/src/internals/types.ts:6-7`); `handleInputPress` reads the flag off a `React.MouseEvent` intersection (`packages/react/src/combobox/utils/handleInputPress.ts:9-14`).
- Payload shape for the re-exported change-details type: `reason` is the machine-readable cause, `event` is the underlying native event whose type is keyed by reason through `ReasonToEvent` (falling back to plain `Event` for unlisted reasons) (`packages/react/src/internals/createBaseUIEventDetails.ts:52-54`, map at `packages/react/src/internals/createBaseUIEventDetails.ts:4-47`), `cancel()`/`allowPropagation()` mutate internal flags surfaced via the `isCanceled`/`isPropagationAllowed` getters, and `trigger` names the originating element (`packages/react/src/internals/createBaseUIEventDetails.ts:56-85`).
- Bubbling/preventDefault semantics per se are N/A: the unit never constructs an event or touches `Event.prototype`; runtime details factories (internal to the internals unit) default a missing native event to `new Event('base-ui')` (`packages/react/src/internals/createBaseUIEventDetails.ts:132`, `packages/react/src/internals/createBaseUIEventDetails.ts:162`).

## Edge cases (rapid interactions, unmount, nesting)

N/A for all runtime edge cases — no runtime exists to be rapid, unmounted, or nested (`packages/react/src/types/index.ts:1-26`). Type-level edge cases worth noting for porting:

- The `Reason extends string` guard makes `BaseUIChangeEventDetails`/`BaseUIGenericEventDetails` collapse to `never` under a non-string reason rather than silently widening (`packages/react/src/internals/createBaseUIEventDetails.ts:93`, `packages/react/src/internals/createBaseUIEventDetails.ts:112`).
- `BaseUIEvent`'s `baseUIHandlerPrevented` is optional and `readonly` (`packages/react/src/types/index.ts:25`), so the augmentation composes with plain React event types without breaking assignability in either direction.

## Shared harness dependencies

N/A — this unit has no test files (`testFiles: []` at `ralph/generated/components.json:1472-1475`), so no test harness (`createRenderer`, `#test-utils`, firePointer, etc.) applies. Its "harness" in the loosest sense is the type checker: the unit is exercised by `pnpm typescript` and the public-type validation pipeline rather than Vitest (`packages/react/src/types/index.ts:1-26`).
