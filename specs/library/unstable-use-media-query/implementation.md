# useMediaQuery — implementation spec

Source-only mining (the unit has no tests; see the header note in `specs/library/unstable-use-media-query/behavior.md`). Ground truth for WHAT is that behavior spec, referred to below by section name only. Sources mined: the unit's only file, `packages/react/src/unstable-use-media-query/index.ts` (90 lines), plus the one Base UI utility it imports (`packages/utils/src/addEventListener.ts`) and the export/consumer edges recorded in `packages/react/package.json`, `TODO.md`, and docs.

## State machine / hooks used

No state machine and no `useState`/`useReducer`: the boolean result lives entirely in an external store — the browser's `MediaQueryList` — read through `useSyncExternalStore` (`packages/react/src/unstable-use-media-query/index.ts:49`), imported from the SSR-safe shim (`packages/react/src/unstable-use-media-query/index.ts:3`). The hook's own hooks, in call order:

1. `React.useCallback` builds `getDefaultSnapshot`, a closure returning `defaultMatches`, memoized on `[defaultMatches]` (`packages/react/src/unstable-use-media-query/index.ts:22`). Its identity feeds both downstream memos (`packages/react/src/unstable-use-media-query/index.ts:34`, `packages/react/src/unstable-use-media-query/index.ts:47`), so changing `defaultMatches` re-creates the snapshot machinery — the mechanism behind behavior.md's "State model" re-creation bullet.
2. `React.useMemo` (server snapshot; deps `[getDefaultSnapshot, query, ssrMatchMedia, noSsr, matchMedia]`, `packages/react/src/unstable-use-media-query/index.ts:24-34`) selects among three tiers:
   - `noSsr && matchMedia` → `() => matchMedia(query).matches`, a live read evaluated per call (`packages/react/src/unstable-use-media-query/index.ts:25-27`);
   - `ssrMatchMedia` → `const { matches } = ssrMatchMedia(query)` evaluated once at memo time, with the getter closing over the captured boolean (`packages/react/src/unstable-use-media-query/index.ts:29-32`);
   - fallback → `getDefaultSnapshot` (`packages/react/src/unstable-use-media-query/index.ts:33`).
3. `React.useMemo` (client snapshot + subscribe pair; deps `[getDefaultSnapshot, matchMedia, query]`, `packages/react/src/unstable-use-media-query/index.ts:36-47`):
   - `matchMedia === null` → `[getDefaultSnapshot, () => () => {}]`: a static `defaultMatches` snapshot and a subscribe that ignores `notify` and returns an empty unsubscribe (`packages/react/src/unstable-use-media-query/index.ts:37-39`);
   - otherwise → one `mediaQueryList = matchMedia(query)` per memo run (`packages/react/src/unstable-use-media-query/index.ts:41`), snapshot `() => mediaQueryList.matches` (`packages/react/src/unstable-use-media-query/index.ts:44`), subscribe wiring `notify` to the list's `change` event via the `addEventListener` util (`packages/react/src/unstable-use-media-query/index.ts:45`).
4. `useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot)` (`packages/react/src/unstable-use-media-query/index.ts:49`) — the point where the external store becomes render-observable state.
5. Dev-only `React.useDebugValue({ query, match })` (`packages/react/src/unstable-use-media-query/index.ts:51-54`), guarded by `process.env.NODE_ENV !== 'production'` with an eslint-disable for `react-hooks/rules-of-hooks` (`packages/react/src/unstable-use-media-query/index.ts:52`) because the hook call is textually conditional; the guard is constant per bundle, so hook order is stable in any given environment.

The `@media` prefix strip (`packages/react/src/unstable-use-media-query/index.ts:13`) is plain string preprocessing before any list is created: `/^@media( ?)/m` removes the first `@media` plus an optional single space found at a line start.

## Context providers/consumers

None — the file contains no `createContext` or `useContext` calls anywhere (`packages/react/src/unstable-use-media-query/index.ts:1-90`). All communication happens via arguments and the boolean return (`packages/react/src/unstable-use-media-query/index.ts:5`). This is consistent with behavior.md's N/A sections for keyboard, focus, and accessibility: none of that plumbing exists in this unit.

## DOM/portal strategy and why

The hook renders nothing and portals nothing; its entire DOM strategy is indirect observation — it asks the browser for a `MediaQueryList` (`packages/react/src/unstable-use-media-query/index.ts:41`) and lets the browser push updates through `change` events rather than polling or listening to `resize` (`packages/react/src/unstable-use-media-query/index.ts:43-46`). The implementation choices with visible rationale:

- `useSyncExternalStore` is the React-sanctioned way to subscribe to an external store tear-free under concurrent rendering; the shim import (`packages/react/src/unstable-use-media-query/index.ts:3`) provides it for older React versions, and the package is a declared dependency (`packages/react/package.json:135`).
- The `addEventListener` util is used instead of raw `mediaQueryList.addEventListener` because it returns a cleanup function — exactly the unsubscribe shape the store subscription contract needs (`packages/react/src/unstable-use-media-query/index.ts:45`, `packages/utils/src/addEventListener.ts:52-61`, JSDoc at `packages/utils/src/addEventListener.ts:34-36`). The util's typed overloads enumerate `MediaQueryList` as a known target (`packages/utils/src/addEventListener.ts:8`) with `MediaQueryListEventMap` typing the `change` listener (`packages/utils/src/addEventListener.ts:17-18`, overload signature `packages/utils/src/addEventListener.ts:37-45`).
- The defensive `supportMatchMedia` check exists for environments without `matchMedia` (the source comments name jsdom); all browsers Base UI supports have it built in (`packages/react/src/unstable-use-media-query/index.ts:6-11`).

## Dependencies on other Base UI internals

The `TODO.md` entry `infra: unstable-use-media-query` (`TODO.md:334-341`, target crate `leptos-ui-internals` at `TODO.md:335-335` — consolidated from the mined per-unit `leptos-unstable-use-media-query` name by the crate-workspace decision in `specs/architecture.md`) carries **no `wraps-external:` field** (compare an entry that does, `TODO.md:316-316`), so there is no third-party delegation to document.

Inbound (what the unit imports):

- `react` (`packages/react/src/unstable-use-media-query/index.ts:1`).
- `@base-ui/utils/addEventListener` (`packages/react/src/unstable-use-media-query/index.ts:2`) — the only Base UI internal: a thin typed add/remove wrapper returning a cleanup (`packages/utils/src/addEventListener.ts:52-61`).
- `use-sync-external-store/shim` (`packages/react/src/unstable-use-media-query/index.ts:3`), declared in `packages/react/package.json:135`.

That is the complete import set — no other Base UI internals, no context, no shared component machinery.

Outbound (who depends on this unit): the package export map publishes it as `@base-ui/react/unstable-use-media-query` (`packages/react/package.json:72`). In-repo consumers are docs-only: `docs/src/blocks/GoogleAnalyticsProvider.tsx:3` and the navigation-menu nested-inline demos (`docs/src/app/(docs)/react/components/navigation-menu/demos/nested-inline/tailwind/index.tsx:4`, `docs/src/app/(docs)/react/components/navigation-menu/demos/nested-inline/css-modules/index.tsx:4`). Nothing under `packages/` imports the unit besides the export map, and the TODO entry is marked `exempt-from-docs-pairing: true` (`TODO.md:341-341`).

## Anything in source not explained by any test

N/A — this unit has no tests at all (`hasTests: false` in `ralph/generated/components.json:1483`, empty `testFiles` at `ralph/generated/components.json:1484`). Every behavioral claim in `behavior.md` is therefore source-derived rather than test-verified, and no part of the source can be distinguished as "untested" because none of it is tested.
