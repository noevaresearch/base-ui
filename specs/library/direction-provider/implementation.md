# DirectionProvider — implementation spec

Stage 2 mining of `packages/react/src/direction-provider/` (non-test sources: `DirectionProvider.tsx`, `index.parts.ts`, `index.ts`, plus the type-contract file `DirectionProvider.spec.tsx`). WHAT-level behavior is ground truth in [behavior.md](./behavior.md); this doc explains the WHY/HOW behind it. The unit is a stateless context provider plus a consumer hook it does not even own — the entire context mechanism lives in `internals/direction-context`, and this unit is mostly a public-surface wrapper around it.

## State machine / hooks used

There is no state machine, no state, and no effects. The component body (`packages/react/src/direction-provider/DirectionProvider.tsx:13-21`) is a single render expression with exactly one hook:

- `React.useMemo` (`packages/react/src/direction-provider/DirectionProvider.tsx:17`) memoizes `{ direction }` against `[direction]`. Purpose: referential stability of the context value. Since the memo dep is the `direction` string itself, any provider re-render with an unchanged value reuses the same context object, and React's context identity bail-out skips re-rendering every consumer below. When the prop does change, a fresh object fans out to all consumers — this is the entire mechanism behind the live `rtl → ltr` transition verified in behavior.md's "State model" section: context propagation is a re-render of subscribers, never a remount, so descendant DOM/state survives the switch.
- The provider applies a **writer-side default**: `const { direction = 'ltr' } = props` (`packages/react/src/direction-provider/DirectionProvider.tsx:16`). Unlike CSPProvider, which forwards props verbatim including `undefined`, this unit never lets `undefined` into the context — under any provider the value is always a fully-formed `{ direction }`.
- No `useControlled`, no `useIsoLayoutEffect`/`useStableCallback`, no lifecycle code anywhere in the unit. The provider is a pure function of one prop → context value.

`DirectionProviderState` is an empty interface (`packages/react/src/direction-provider/DirectionProvider.tsx:23`), re-exported through the `DirectionProvider` namespace (`packages/react/src/direction-provider/DirectionProvider.tsx:34-37`) purely to conform to the part-API shape other Base UI components follow; it carries no runtime meaning. The component is typed `React.FC<DirectionProvider.Props>` (`packages/react/src/direction-provider/DirectionProvider.tsx:13`) with the prop JSDoc documenting `@default 'ltr'` (`packages/react/src/direction-provider/DirectionProvider.tsx:27-30`).

## Context providers/consumers

The unit is the write half of `internals/direction-context/DirectionContext`:

- The context value shape is `{ direction: TextDirection }` (`packages/react/src/internals/direction-context/DirectionContext.tsx:6-8`), where `TextDirection = 'ltr' | 'rtl'` (`packages/react/src/internals/direction-context/DirectionContext.tsx:4`). The only field crossing the boundary is `direction`.
- The context is created with an `undefined` default (`packages/react/src/internals/direction-context/DirectionContext.tsx:10`), and the consumer hook `useDirection` applies the fallback `context?.direction ?? 'ltr'` (`packages/react/src/internals/direction-context/DirectionContext.tsx:12-15`). So the no-provider default proven in behavior.md's "State model" section is implemented **here**, in the hook — not in the unit.
- Double-default note: because of the writer-side default (`packages/react/src/direction-provider/DirectionProvider.tsx:16`), the hook's `?? 'ltr'` fallback is unreachable under any provider — it only fires with no provider in scope. Belt-and-suspenders; observably identical either way.
- The unit is the codebase's only writer: no other module imports the raw `DirectionContext` object; every reader goes through `useDirection`.
- Readers — production call sites of `useDirection`, grouped by what they consume it for:
  - Anchored-popup positioning: `packages/react/src/internals/useAnchorPositioning.ts:182` (feeds floating-ui side/alignment for every positionable popup), `packages/react/src/select/popup/SelectPopup.tsx:68`, `packages/react/src/menu/root/MenuRoot.tsx:480`, `packages/react/src/navigation-menu/popup/NavigationMenuPopup.tsx:28`, `packages/react/src/navigation-menu/trigger/NavigationMenuTrigger.tsx:110`, `packages/react/src/combobox/root/AriaCombobox.tsx:157`, `packages/react/src/combobox/input/ComboboxInput.tsx:69`, `packages/react/src/combobox/chip/ComboboxChip.tsx:30`.
  - Composite/roving-focus keyboard geometry: `packages/react/src/internals/composite/root/CompositeRoot.tsx:42` (with type-only `TextDirection` imports at `packages/react/src/internals/composite/root/useCompositeRoot.ts:8` and `packages/react/src/internals/composite/composite.ts:2`).
  - Slider geometry: `packages/react/src/slider/control/SliderControl.tsx:116`, `packages/react/src/slider/thumb/SliderThumb.tsx:145`.
  - ScrollArea: `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:64`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:103`.
  - OTP field: `packages/react/src/otp-field/input/OTPFieldInput.tsx:67`.
  - Shared popup-viewport util: `packages/react/src/utils/usePopupViewport.tsx:78` — notably imported via the unit's public path (`packages/react/src/utils/usePopupViewport.tsx:16`, `from '../direction-provider'`), whereas every other consumer imports the internals module directly.
- Public re-export chain: `packages/react/src/direction-provider/index.parts.ts:3` re-exports the internals hook, `packages/react/src/direction-provider/index.parts.ts:5` the `TextDirection` type, and `packages/react/src/direction-provider/index.parts.ts:1` aliases the component as `Provider` (the parts-file convention); the public entry renames it back and adds the props type (`packages/react/src/direction-provider/index.ts:1`, `packages/react/src/direction-provider/index.ts:2`). The subpath itself is wired in `packages/react/package.json:42` and the barrel in `packages/react/src/index.ts:13`.

## DOM/portal strategy and why

The provider renders no DOM node at all — it returns the bare `DirectionContext.Provider` wrapping `children` (`packages/react/src/direction-provider/DirectionProvider.tsx:18-20`). This resolves behavior.md's UNVERIFIED "Accessibility" and "DOM structure & portal behavior" sections: there is no wrapper element and no `dir` attribute is ever set — DOM directionality is entirely the application's concern.

There is no portal logic, and none is needed: React context follows the React element tree, so it reaches consumers whose DOM is portaled elsewhere (menus, selects, popovers). The heaviest consumers need the value at the React-tree level anyway — e.g. anchor positioning computes floating-ui placement from it before any DOM exists to inspect. `'use client'` directives at `packages/react/src/direction-provider/DirectionProvider.tsx:1` and `packages/react/src/internals/direction-context/DirectionContext.tsx:1` mark the RSC client boundary.

## Dependencies on other Base UI internals

Exactly one internal import: `DirectionContext` and `TextDirection` from `internals/direction-context/DirectionContext` (`packages/react/src/direction-provider/DirectionProvider.tsx:3-6`). That one file supplies the unit's entire mechanism — context object, consumer hook, union type. The consumer half (`useDirection`) is imported by the reader components listed above, not re-imported by this unit.

Nothing else: no `floating-ui-react`, no `use-render`, no `@base_ui/utils` imports anywhere in the unit's four files. (The dependency arrow points the other way: `internals/useAnchorPositioning.ts` — the floating-ui integration layer — depends on this unit's context, not vice versa.)

Per the TODO.md entry (`TODO.md:247-252`), there is no `wraps-external:` field — no external package is delegated to, and no Rust crate substitution applies beyond the unit's own `leptos-direction-provider` target. The porting dependency graph is: one internals file (context + hook), the props/state types, and the read-side consumers above.

## Anything in source not explained by any test

- **Writer-side default** (`packages/react/src/direction-provider/DirectionProvider.tsx:16`): no test renders `<DirectionProvider>` without a `direction` prop — behavior.md's "State model" flags exactly this gap as UNVERIFIED. Consequently no test could distinguish the writer-side default from the hook-side `?? 'ltr'` fallback (`packages/react/src/internals/direction-context/DirectionContext.tsx:15`), nor observe that `{ direction: undefined }` never enters context under a provider.
- **`useMemo` identity-stability invariant** (`packages/react/src/direction-provider/DirectionProvider.tsx:17`): no test mounts multiple consumers or counts renders, so dropping the memo would pass the whole suite. Performance-only invariant.
- **Type-only contract** (`packages/react/src/direction-provider/DirectionProvider.spec.tsx:10-12`, `packages/react/src/direction-provider/DirectionProvider.spec.tsx:19`, `packages/react/src/direction-provider/DirectionProvider.spec.tsx:25-27`): the compile-time assertions that `useDirection()` returns exactly `TextDirection` (never `undefined` — guaranteed by the hook fallback), that the prop is `TextDirection | undefined`, and that the union is closed (`@ts-expect-error` on `direction="vertical"`) have no runtime counterpart in any test file.
- **`DirectionProviderState` and the namespace** (`packages/react/src/direction-provider/DirectionProvider.tsx:23`, `packages/react/src/direction-provider/DirectionProvider.tsx:34-37`): nothing references the State type — and `packages/react/src/direction-provider/index.ts:2` exports only `DirectionProviderProps`, so State/namespace never even reach the public entry. Dead part-API surface.
- **Nesting** (behavior.md "Edge cases" flags UNVERIFIED): the source contains no merge/override logic, so by React context scoping the innermost provider's value wins wholesale (`packages/react/src/direction-provider/DirectionProvider.tsx:18-20`) — source-explainable, untested.
- **Unmount** (behavior.md "Edge cases" flags UNVERIFIED): zero lifecycle code, so there is nothing to clean up by construction.
- **Export shape** (`packages/react/src/direction-provider/index.parts.ts:1`, `packages/react/src/direction-provider/index.ts:1-2`, `packages/react/package.json:42`, `packages/react/src/index.ts:13`): the `Provider` alias / rename chain, subpath export, and barrel re-export are exercised by no test — the suite imports only the public entry.
- **`'use client'` directives** (`packages/react/src/direction-provider/DirectionProvider.tsx:1`, `packages/react/src/internals/direction-context/DirectionContext.tsx:1`): RSC behavior is untestable in the jsdom/chromium suites.
