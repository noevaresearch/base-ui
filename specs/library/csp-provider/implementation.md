# CSPProvider — implementation spec

Stage 2 mining of `packages/react/src/csp-provider/` (non-test sources: `CSPProvider.tsx`, `index.parts.ts`, `index.ts`). WHAT-level behavior is ground truth in [behavior.md](./behavior.md); this doc explains the WHY/HOW behind it. The unit is a ~20-line stateless context provider — most sections are short because the unit genuinely has no machinery.

## State machine / hooks used

There is no state machine and no state. The component body (packages/react/src/csp-provider/CSPProvider.tsx:11-23) is a single render expression with exactly one hook:

- `React.useMemo` (packages/react/src/csp-provider/CSPProvider.tsx:14-20) memoizes `{ nonce, disableStyleElements }` against `[nonce, disableStyleElements]`. Purpose: referential stability of the context value. Without it, every re-render of the provider (triggered by any parent re-render, even with unchanged props) would produce a fresh context object and re-render every context consumer in the subtree. The memo makes consumer re-renders fire only when the two config props actually change.
- No `useControlled`, no effects, no `useIsoLayoutEffect`/`useStableCallback` anywhere in the unit. The provider is a pure function of props → context value.

`CSPProviderState` is an empty interface (packages/react/src/csp-provider/CSPProvider.tsx:25), re-exported through the `CSPProvider` namespace (packages/react/src/csp-provider/CSPProvider.tsx:40-43) purely to conform to the part-API shape other Base UI components follow; it carries no runtime meaning.

## Context providers/consumers

The unit IS the provider half of `internals/csp-context/CSPContext`:

- It wraps `children` in `CSPContext.Provider` (packages/react/src/csp-provider/CSPProvider.tsx:22), passing a `CSPContextValue` of `{ nonce?: string; disableStyleElements?: boolean }` (packages/react/src/internals/csp-context/CSPContext.tsx:4-7).
- What crosses the boundary is exactly the two props, verbatim — including `undefined` when a prop is omitted (packages/react/src/csp-provider/CSPProvider.tsx:12,14-19). The provider applies no defaults itself.
- Defaults live on the consumer side: the context is created with an `undefined` default (packages/react/src/internals/csp-context/CSPContext.tsx:9), and `useCSPContext` falls back to `DEFAULT_CSP_CONTEXT_VALUE = { disableStyleElements: false }` (packages/react/src/internals/csp-context/CSPContext.tsx:11-17). So the behavior.md-verified default (`disableStyleElements` off with no provider) is implemented by the `??` fallback in the hook, not in this unit — and under that fallback `nonce` stays `undefined` as well, which is the same observable outcome.
- Known consumers, all reading via `useCSPContext` (packages/react/src/internals/csp-context/CSPContext.tsx:15-17):
  - `ScrollAreaRoot` — packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:53 destructures both values; it injects the `.base-ui-disable-scrollbar` stylesheet (the subject of behavior.md's "DOM structure & portal behavior" ScrollArea assertions).
  - `SelectPopup` — packages/react/src/select/popup/SelectPopup.tsx:70, same pair; this is how the `Select.Portal` suppression in behavior.md works.
  - `PrehydrationScript` — packages/react/src/internals/PrehydrationScript.tsx:34 reads only `nonce` and applies it to an inline `<script>` (packages/react/src/internals/PrehydrationScript.tsx:42-47). Notably it ignores `disableStyleElements`: the CSP contract here covers script nonces too, not just `<style>` suppression (matches the provider's JSDoc "inline `<style>` or `<script>` tags", packages/react/src/csp-provider/CSPProvider.tsx:6-7).

## DOM/portal strategy and why

The provider renders no DOM node at all — it returns the context provider directly (packages/react/src/csp-provider/CSPProvider.tsx:22). This resolves behavior.md's UNVERIFIED wrapper-element question: there is no wrapper; children mount in place with no host element added.

There is no portal logic in the unit, and none is needed. The design rationale for context (rather than, say, a data attribute on a wrapper element) is that everything being governed lives at document level: inline `<style>` elements are hoisted to `<head>` by React 19 (observed in behavior.md) and `Select.Portal` content is portaled elsewhere in the DOM. React context follows the React element tree, so it reaches consumers regardless of where their DOM output lands — a DOM-attribute strategy could not. The mechanics of actually rendering (or suppressing) `<style nonce=...>` belong to the consumers listed above; this unit holds no effects and no cleanup.

## Dependencies on other Base UI internals

Exactly one internal import: `CSPContext` and `CSPContextValue` from `internals/csp-context/CSPContext` (packages/react/src/csp-provider/CSPProvider.tsx:3). The consumer-facing half of that module (`useCSPContext`) is imported by the consumer components, not by this unit.

Nothing else: no `floating-ui-react`, no `use-render`, no `@base_ui/utils` imports anywhere in the unit's three files.

Per the TODO.md entry (TODO.md:237-241), there is no `wraps-external:` field — no external package is delegated to, and no Rust crate substitution applies beyond the unit's own `leptos-csp-provider` target. The unit's dependency graph for porting is: one context module (one file) plus the consumers listed above.

## Anything in source not explained by any test

- `CSPProviderState` (empty, packages/react/src/csp-provider/CSPProvider.tsx:25) and the `CSPProvider` namespace re-export (packages/react/src/csp-provider/CSPProvider.tsx:40-43): no test references the State type or namespace. Part-API surface only.
- Public export shape: `packages/react/src/csp-provider/index.parts.ts:1` re-exports just the component, and `packages/react/src/csp-provider/index.ts:2` does `export type * from './CSPProvider'`. No test exercises the package entry points.
- Nesting semantics (behavior.md flags this UNVERIFIED): the source contains no merge/override logic, so by React context scoping the innermost provider's props win wholesale — there is no per-prop inheritance between nested providers (packages/react/src/csp-provider/CSPProvider.tsx:22). Source-explainable, but untested.
- Provider unmount (behavior.md flags this UNVERIFIED): the unit has zero lifecycle code — no effects, no cleanup — so the provider implements no style-removal on unmount by construction. Whether injected styles disappear is entirely a consumer-side question; nothing in this source can explain style cleanup either way.
- The `useMemo` identity-stability guarantee (packages/react/src/csp-provider/CSPProvider.tsx:14-20) is invisible to the current suite: every test re-renders from scratch, so no test could detect a dropped memo (e.g. extra consumer re-renders). It is a performance invariant only.
- The default-application split (provider passes `undefined` through; `false` comes from `DEFAULT_CSP_CONTEXT_VALUE` in packages/react/src/internals/csp-context/CSPContext.tsx:11-17) is only observable indirectly through consumer behavior; no test asserts the hook's fallback shape itself.
