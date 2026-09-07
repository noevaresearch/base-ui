# useMediaQuery — behavior spec

This unit has no test files (`hasTests: false` in `ralph/generated/components.json:1483`, entry `ralph/generated/components.json:1481-1487`) — every claim below is derived directly from source, not confirmed by a test.

Mined from the unit's single source file: `packages/react/src/unstable-use-media-query/index.ts` (90 lines). The unit is a hook that reports whether a CSS media query currently matches; it renders no UI of its own.

## Public API surface (props, parts, subcomponents)

- `useMediaQuery(query, options)` is a hook returning a `boolean` — whether `query` currently matches (`packages/react/src/unstable-use-media-query/index.ts:5`).
- `query` is a media-query string; a leading `@media` prefix (with an optional single following space) is stripped before evaluation, first occurrence only (`packages/react/src/unstable-use-media-query/index.ts:13`), so both `"(min-width: 600px)"` and `"@media (min-width: 600px)"` are accepted forms.
- Options (`useMediaQuery.Options`, `packages/react/src/unstable-use-media-query/index.ts:59-83`):
  - `defaultMatches?: boolean` — value used when no real match information is available (see State model); defaults to `false` (`packages/react/src/unstable-use-media-query/index.ts:60-65`, `packages/react/src/unstable-use-media-query/index.ts:16`).
  - `matchMedia?: typeof window.matchMedia` — a custom `matchMedia` implementation, documented for handling an iframe content window; overrides the ambient `window.matchMedia` when provided (`packages/react/src/unstable-use-media-query/index.ts:66-70`, `packages/react/src/unstable-use-media-query/index.ts:17`).
  - `noSsr?: boolean` — when `true` (and a `matchMedia` implementation is available), the server snapshot is computed by calling `matchMedia(query).matches` directly, avoiding the double render that hydration otherwise requires; defaults to `false` (`packages/react/src/unstable-use-media-query/index.ts:71-78`, `packages/react/src/unstable-use-media-query/index.ts:25-27`).
  - `ssrMatchMedia?: (query: string) => { matches: boolean }` — a server-side stand-in for `matchMedia`, consulted when `noSsr` is not in effect (`packages/react/src/unstable-use-media-query/index.ts:79-82`, `packages/react/src/unstable-use-media-query/index.ts:29-32`).
- Type namespace: `useMediaQuery.Options` and `useMediaQuery.State` alias `UseMediaQueryOptions` and `UseMediaQueryState` (`packages/react/src/unstable-use-media-query/index.ts:87-90`); `UseMediaQueryState` is an empty interface (`packages/react/src/unstable-use-media-query/index.ts:85`).
- No subcomponents, parts, or context — the entire public surface is the single hook plus its option types, published as `@base-ui/react/unstable-use-media-query` (`packages/react/package.json:72`).

## State model (controlled/uncontrolled, defaults, transitions)

- No controlled/uncontrolled dichotomy: the hook holds no React state of its own; the single observable value is the boolean match result read from an external store (`packages/react/src/unstable-use-media-query/index.ts:49`).
- Effective default of `matchMedia`: `window.matchMedia` when both `window` and `window.matchMedia` exist, otherwise `null` (`packages/react/src/unstable-use-media-query/index.ts:10-11`, `packages/react/src/unstable-use-media-query/index.ts:17`).
- With a working `matchMedia`, the result tracks the live `MediaQueryList.matches` value and updates whenever that list fires `change` (`packages/react/src/unstable-use-media-query/index.ts:41-46`).
- Without an effective `matchMedia` (no `window`, missing `window.matchMedia`, or an explicit `null`), the result is pinned to `defaultMatches` and never changes — no subscription is made (`packages/react/src/unstable-use-media-query/index.ts:37-39`).
- Transitions: the only transition is the boolean flipping when the browser re-evaluates the media query, delivered through the `change` subscription (`packages/react/src/unstable-use-media-query/index.ts:45`).
- Re-creating the machinery: a change to `query`, `matchMedia`, or `defaultMatches` re-runs the client memo, producing a fresh `MediaQueryList` and snapshot/subscribe pair (`packages/react/src/unstable-use-media-query/index.ts:36-47`; `defaultMatches` participates via the identity of `getDefaultSnapshot`, `packages/react/src/unstable-use-media-query/index.ts:22`).
- SSR/hydration model: three possible server snapshots — a live `matchMedia` call under `noSsr`, a captured `ssrMatchMedia` result, or `defaultMatches` (`packages/react/src/unstable-use-media-query/index.ts:24-34`); details under Edge cases.

## Keyboard interactions

N/A — the hook returns a boolean and renders nothing; no keyboard handling exists in the source (`packages/react/src/unstable-use-media-query/index.ts:5`, `packages/react/src/unstable-use-media-query/index.ts:56`).

## Focus management

N/A — the hook returns a boolean and renders nothing; it never touches focus (`packages/react/src/unstable-use-media-query/index.ts:5`, `packages/react/src/unstable-use-media-query/index.ts:56`).

## Accessibility (roles, aria-*, id linking)

N/A — no roles, `aria-*` attributes, or id linking are produced anywhere in the source; the hook's entire output is a boolean (`packages/react/src/unstable-use-media-query/index.ts:5`, `packages/react/src/unstable-use-media-query/index.ts:56`).

## DOM structure & portal behavior

N/A — the hook renders no DOM and creates no portals. Its only DOM interaction is indirect: it obtains a `MediaQueryList` from `matchMedia(query)` and reads/subscribes to it (`packages/react/src/unstable-use-media-query/index.ts:41-46`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- The hook emits no events of its own; it only consumes the browser's `change` event on a `MediaQueryList` (`packages/react/src/unstable-use-media-query/index.ts:45`).
- Payload shape: the `change` listener is the `notify` callback handed to the store subscription; the fresh value is obtained by re-reading `mediaQueryList.matches` through the snapshot on the following render (`packages/react/src/unstable-use-media-query/index.ts:43-46`, `packages/react/src/unstable-use-media-query/index.ts:49`).
- Bubbling and `preventDefault` semantics: N/A — the listener never inspects or cancels the event; it is a plain store-notify function (`packages/react/src/unstable-use-media-query/index.ts:45`).

## Edge cases (rapid interactions, unmount, nesting)

- No-`window` environments (server, jsdom): `supportMatchMedia` is `false`, so the effective `matchMedia` is `null` and the hook degenerates to a static read of `defaultMatches` with a no-op subscription; the source comments describe this check as defensive for jsdom, noting all supported browsers have `matchMedia` built in (`packages/react/src/unstable-use-media-query/index.ts:6-11`, `packages/react/src/unstable-use-media-query/index.ts:37-39`).
- Server snapshot precedence: `noSsr && matchMedia` → live `matchMedia(query).matches` per snapshot call (`packages/react/src/unstable-use-media-query/index.ts:25-27`); else `ssrMatchMedia` → its result captured once (`packages/react/src/unstable-use-media-query/index.ts:29-32`); else `defaultMatches` (`packages/react/src/unstable-use-media-query/index.ts:33`). With `noSsr: true` but no available `matchMedia`, the first branch is skipped because `matchMedia` is `null` (`packages/react/src/unstable-use-media-query/index.ts:25`).
- `ssrMatchMedia` is invoked once per memo factory run, not on every snapshot read; the getter closes over the captured `matches` boolean (`packages/react/src/unstable-use-media-query/index.ts:29-32`).
- Hydration: the `getServerSnapshot` argument makes React render with the server value during hydration and re-render with the client value — the double pass the `noSsr` JSDoc describes as the cost of correct hydration (`packages/react/src/unstable-use-media-query/index.ts:49`, `packages/react/src/unstable-use-media-query/index.ts:71-77`).
- Unmount: the subscription is torn down via the cleanup function `addEventListener` returns, which removes the `change` listener (`packages/react/src/unstable-use-media-query/index.ts:45`, `packages/utils/src/addEventListener.ts:58-61`).
- Rapid `change` events: each event notifies the store and triggers a re-render reading `matches`; the hook adds no throttling or coalescing (`packages/react/src/unstable-use-media-query/index.ts:43-49`).
- Query change mid-lifetime: a new `query` re-runs the client memo against a fresh `MediaQueryList`; the previous list is simply dropped, with React re-subscribing per the `useSyncExternalStore` contract (`packages/react/src/unstable-use-media-query/index.ts:36-47`, `packages/react/src/unstable-use-media-query/index.ts:3`).
- Nesting: N/A — multiple hook instances are independent; each creates its own `MediaQueryList` from its own memo (`packages/react/src/unstable-use-media-query/index.ts:41`).

## Shared harness dependencies

N/A — this unit has no test files (`hasTests: false`, `ralph/generated/components.json:1483`), so no test harness (`#test-utils`, renderers, matchers) is associated with it.
