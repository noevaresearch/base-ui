# `types` — implementation spec

Companion to `specs/library/types/behavior.md` (the WHAT). This file is the WHY/HOW. Unit: `infra: types` (`TODO.md:274-279`, target crate `leptos-types`); the TODO entry has no `wraps-external:` field, so there is no external-package delegation — the unit is entirely first-party and its entire implementation is one type-only file, `packages/react/src/types/index.ts` (26 lines, no runtime statements beyond a type-only import).

## State machine / hooks used

N/A — no hooks, no state machines, no effects. The file contains exclusively `import type`/`export type` declarations (`packages/react/src/types/index.ts:1-26`), so there is no React runtime to port; the Rust-equivalent is a set of trait/enum type definitions (`TODO.md:274-279`).

## Context providers/consumers

N/A — the unit defines no React context, no providers, no consumers (`packages/react/src/types/index.ts:1-26`). It is, however, the shared *vocabulary* other units' contexts are typed with (see inbound dependencies below).

## DOM/portal strategy and why

N/A — nothing is rendered or portaled. The one deliberate type-level design decision with DOM implications: `HTMLProps` re-adds `ref` explicitly on top of `React.HTMLAttributes<T>` (`packages/react/src/types/index.ts:8-10`) because React's attribute types omit it. This is what lets Base UI's prop-getter pipeline carry and merge refs — e.g. `useRenderElement` types its `propGetter` as `(externalProps: HTMLProps) => HTMLProps` (`packages/react/src/internals/useRenderElement.tsx:261`) — and why the shared `BaseUIComponentProps` uses `HTMLProps` as its render-function props base rather than `React.HTMLAttributes` (`packages/react/src/internals/types.ts:39`).

## Dependencies on other Base UI internals

Outbound (what this unit imports):

- `react` types only: `import type * as React from 'react'` and references to `React.HTMLAttributes`, `React.Ref`, `React.ReactElement`, `React.SyntheticEvent` (`packages/react/src/types/index.ts:1`, `packages/react/src/types/index.ts:8-10`, `packages/react/src/types/index.ts:18-21`, `packages/react/src/types/index.ts:23-26`).
- `../internals/createBaseUIEventDetails`: the only intra-repo dependency, for the two event-details re-exports (`packages/react/src/types/index.ts:3-6`). Note this creates a layering exception — a public type barrel that reaches into the internals directory — which is why the details types are defined there rather than duplicated here.

Inbound (what depends on this unit — the reason the unit exists):

- The public barrel re-exports the whole module as types: `export type * from './types'` (`packages/react/src/index.ts:44`).
- `internals/types.ts` builds the library's shared component contract on top of it: it imports `BaseUIEvent`, `ComponentRenderFn`, `HTMLProps` and re-exports them (`packages/react/src/internals/types.ts:2-4`), derives `MaybeBaseUIEvent` as a `Partial<Pick<BaseUIEvent<E>, ...>>` (`packages/react/src/internals/types.ts:6-7`), maps every event-handler prop onto `BaseUIEvent` via `WithPreventBaseUIHandler`/`WithBaseUIEvent` (`packages/react/src/internals/types.ts:17-30`), and defines `BaseUIComponentProps` with `HTMLProps` as the render-props base and `ComponentRenderFn` as the `render` prop's function form (`packages/react/src/internals/types.ts:36-61`).
- `useRenderElement` — the shared element-rendering hook all parts go through — consumes all three: `BaseUIComponentProps`, `ComponentRenderFn`, `HTMLProps` (`packages/react/src/internals/useRenderElement.tsx:7`, render-prop typing at `packages/react/src/internals/useRenderElement.tsx:296`).
- Roughly forty source files import directly from `'../types'`/`'../../types'`, including the floating-ui-react integration layer (`packages/react/src/floating-ui-react/hooks/useFloatingRootContext.ts`, `packages/react/src/floating-ui-react/components/FloatingTree.tsx`), the composite system (`packages/react/src/internals/composite/root/CompositeRoot.tsx`, `packages/react/src/internals/composite/item/useCompositeItem.ts`), field/labelable contexts (`packages/react/src/internals/field-root-context/FieldRootContext.ts:9`, `packages/react/src/internals/labelable-provider/LabelableContext.ts:4`), `useButton` (`packages/react/src/internals/use-button/useButton.ts:10`), and `SwitchRoot` (`packages/react/src/switch/root/SwitchRoot.tsx`).

Porting note: per the delegation rule, no third-party algorithm needs deriving — the unit's only cross-boundary behavior is the `BaseUIEvent` flag contract consumed by `mergeProps` (`packages/react/src/merge-props/mergeProps.ts:239`) and `useButton` (`packages/react/src/internals/use-button/useButton.ts:123`), which is first-party code being ported alongside it.

## Anything in source not explained by any test

N/A — this unit has no tests at all (`testFiles: []` at `ralph/generated/components.json:1472-1475`), so there is no test to fail to explain any of it; every line of `packages/react/src/types/index.ts:1-26` is explained only by source reading and its consumers (documented above). The unit is additionally `exempt-from-docs-pairing` in its TODO entry (`TODO.md:274-279`), so no docs demo exists to cross-reference either.
