# Avatar — implementation spec (Stage 2: implementation mining)

Ground truth for WHAT happens is `specs/library/avatar/behavior.md`; this document explains the
state machine, hook composition, context usage, and DOM decisions that produce it. Source files:

- `packages/react/src/avatar/root/AvatarRoot.tsx` (plus `AvatarRootContext.ts`, `stateAttributesMapping.ts`)
- `packages/react/src/avatar/image/AvatarImage.tsx` (plus `useImageLoadingStatus.ts`, `AvatarImageDataAttributes.ts`)
- `packages/react/src/avatar/fallback/AvatarFallback.tsx`
- `packages/react/src/avatar/index.ts` / `index.parts.ts` (barrels) and `Avatar.spec.tsx` (type-only spec)

The unit's TODO entry (`TODO.md:383-388`) has no `wraps-external:` field — no external package is
delegated to; everything below is derived from first-party source.

## State machine / hooks used

The unit runs a **two-tier status machine with three satellite sub-machines** (delay, transition,
keepMounted sync). All status plumbing is plain React state; there are no reducers or external
stores.

### Tier 1 — image-local status (source of truth)

`useImageLoadingStatus` (`packages/react/src/avatar/image/useImageLoadingStatus.ts:14-71`), called
from `packages/react/src/avatar/image/AvatarImage.tsx:47-51`, owns the canonical status:

- `React.useState<ImageLoadingStatus>('idle')` (`packages/react/src/avatar/image/useImageLoadingStatus.ts:19-20`).
- One `useIsoLayoutEffect` keyed on `[enabled, src, srcSet, sizes, crossOrigin, referrerPolicy, setLoadingStatus]`
  (`packages/react/src/avatar/image/useImageLoadingStatus.ts:68`) implements the probe mode:
  - `enabled === false` (i.e. `keepMounted`) bails out before touching `window.Image`
    (`packages/react/src/avatar/image/useImageLoadingStatus.ts:22-25`) — no probe is ever
    constructed in keepMounted mode.
  - No `src` and no `srcSet` short-circuits to `'error'` without constructing a probe
    (`packages/react/src/avatar/image/useImageLoadingStatus.ts:27-30`).
  - Otherwise it constructs a detached `new window.Image()` (`packages/react/src/avatar/image/useImageLoadingStatus.ts:33`),
    sets `'loading'` synchronously (`:43`), wires `onload`/`onerror` through an `isMounted`-guarded
    updater (`:32`, `:35-41`, `:44-45`), applies the request-configuration props to the probe
    (`:46-58`), and then takes the cached fast-path: if `image.complete`, resolve immediately from
    `naturalWidth` to `'loaded'`/`'error'` in the same layout effect
    (`packages/react/src/avatar/image/useImageLoadingStatus.ts:61-63`). This is why a cached image
    never reports `'loading'` and a cached error never reports `'idle'` (behavior.md, *State model*
    and *Events*).
  - Cleanup only flips the `isMounted` flag (`packages/react/src/avatar/image/useImageLoadingStatus.ts:65-67`);
    it does not reset status. Late probe events become no-ops instead of React state updates after
    unmount.
- The reset-on-source-change transition (behavior.md, *Transitions*) falls out of the effect's
  dependency array: any change to `src`/`srcSet`/`sizes`/`crossOrigin`/`referrerPolicy` tears the
  effect down and re-runs it, which re-sets `'loading'` and re-probes.

### Tier 2 — root-mirrored status (what Fallback and Root render props see)

`AvatarRoot` holds `React.useState<ImageLoadingStatus>('idle')` (`packages/react/src/avatar/root/AvatarRoot.tsx:20`)
and publishes it through context. `AvatarImage` is the only writer:

- `handleLoadingStatusChange = useStableCallback(...)` (`packages/react/src/avatar/image/AvatarImage.tsx:120-123`)
  fans each status out to both the user's `onLoadingStatusChange` prop and the context setter —
  one call site, two consumers.
- A `useIsoLayoutEffect` (`packages/react/src/avatar/image/AvatarImage.tsx:125-129`) reports every
  status *except* `'idle'`; `'idle'` is treated as "nothing to say" while mounted, which is the
  mechanism behind the no-spurious-`'idle'` guarantee (behavior.md, *Events*).
- A separate unmount-only `useIsoLayoutEffect` (`packages/react/src/avatar/image/AvatarImage.tsx:131-133`)
  resets the root to `'idle'` on cleanup. That reset is the whole mechanism behind "unmounting a
  loaded image makes the fallback reappear" (behavior.md, *Edge cases*).

Layout-effect timing on both reporting effects is load-bearing: status reaches the root (and the
DOM attributes derived from it) before paint, so the fallback never flashes for a frame around
hydration or cached resolution.

### Sub-machine: keepMounted status sync from the real element

With `keepMounted`, the probe is disabled and the status is instead read off the rendered `<img>`:

- A `useIsoLayoutEffect` on `imageRef` (`packages/react/src/avatar/image/AvatarImage.tsx:61-99`),
  re-keyed on every source/config prop plus `render`, syncs from the element: not `complete` →
  `'loading'`; `complete` → `'loaded'`/`'error'` from `naturalWidth` (`:76-82`).
- If the user's `render` element dropped the ref, the effect bails without setting anything
  (`packages/react/src/avatar/image/AvatarImage.tsx:69-74`) — the element's own `onLoad`/`onError`
  handlers (`:111-116`) remain the only source of truth and their already-reported status is never
  overwritten. This produces the "dropped ref never spuriously reports `'loading'`" edge case
  (behavior.md, *Edge cases*).
- `initialCommitRef` (`packages/react/src/avatar/image/AvatarImage.tsx:57`, `:66-67`) marks the
  first commit; only then, if the image was already loaded (SSR paint, cache), does the effect call
  `setMounted(true)` directly (`:84-88`). Pre-seeding `mounted` means `useTransitionStatus`'s
  render-phase `open && !mounted → 'starting'` branch (`packages/react/src/internals/useTransitionStatus.ts:31-34`)
  never fires, so the enter animation is not replayed after hydration (behavior.md, *Edge cases*).

### Sub-machine: fallback delay

`AvatarFallback` adds one boolean of local state, `delayPassed`, initialized to `delay === 0`
(`packages/react/src/avatar/fallback/AvatarFallback.tsx:23`) so `delay={0}` renders synchronously
on mount. A plain `React.useEffect` (`packages/react/src/avatar/fallback/AvatarFallback.tsx:26-35`)
— not a layout effect, because delay timing is user-perceived, not pre-paint-critical — starts a
`useTimeout` when `delay > 0` (`:27-29`) and otherwise force-shows (`:30-33`). The "once visible,
never re-hidden" semantics (behavior.md, *delay lifecycle*) fall out of the state only ever moving
`false → true`; nothing ever writes `setDelayPassed(false)`. The enabled gate also ORs in
`delay === 0` directly (`packages/react/src/avatar/fallback/AvatarFallback.tsx:46`), so changing a
pending delay to `0` shows the fallback on that same render rather than waiting an effect cycle.

### Sub-machine: presence + transition status

`useTransitionStatus(isVisible)` (`packages/react/src/avatar/image/AvatarImage.tsx:54`), where
`isVisible = imageLoadingStatus === 'loaded'` (`:53`), drives the enter/exit phases that surface as
`data-starting-style`/`data-ending-style` (behavior.md, *Animation hooks*). Two deliberate wrinkles:

- Presence gate: `shouldRender = keepMounted || mounted` (`packages/react/src/avatar/image/AvatarImage.tsx:153`),
  fed to `useRenderElement`'s `enabled` (`:171`) with a redundant-looking `if (!shouldRender) return null`
  (`:174-176`) for type narrowing. In default mode the `<img>` exists only while `mounted` — i.e.
  only from the moment it is loaded — which is what makes the not-loaded data attributes
  unnecessary in that mode.
- `useOpenChangeComplete({ enabled: !isVisible, open: isVisible, ref: imageRef })`
  (`packages/react/src/avatar/image/AvatarImage.tsx:135-144`) defers `setMounted(false)` until exit
  animations finish (the hook waits on `useAnimationsFinished`, `packages/react/src/internals/useOpenChangeComplete.tsx:15-27`).
  `enabled: !isVisible` scopes it to the closing direction only. This is the mechanism behind the
  "exactly one of image or fallback mounted, even during the fallback's 2s exit animation"
  regression guarantee (behavior.md, *Edge cases*).
- With `keepMounted` the element never unmounts, so an `'ending'` transition would play and
  reverse; the state object masks it (`transitionStatus: keepMounted && transitionStatus === 'ending' ? undefined : transitionStatus`,
  `packages/react/src/avatar/image/AvatarImage.tsx:150`) and `data-loading`/`data-error` carry the
  state instead (`:106-107`), scoped to keepMounted mode precisely so the default mode's
  exiting-but-loaded element can't pick them up.

### Hook inventory (per call site)

| Hook | Call site | Role |
| --- | --- | --- |
| `React.useState` | `packages/react/src/avatar/root/AvatarRoot.tsx:20` | root-mirrored status |
| `React.useState` | `packages/react/src/avatar/image/useImageLoadingStatus.ts:19` | image-local status |
| `React.useState` | `packages/react/src/avatar/fallback/AvatarFallback.tsx:23` | `delayPassed` latch |
| `React.useMemo` | `packages/react/src/avatar/root/AvatarRoot.tsx:26-32` | stable context value |
| `React.useRef` | `packages/react/src/avatar/image/AvatarImage.tsx:56-57` | element ref + first-commit marker |
| `React.useEffect` | `packages/react/src/avatar/fallback/AvatarFallback.tsx:26-35` | delay timer |
| `useIsoLayoutEffect` | `packages/react/src/avatar/image/useImageLoadingStatus.ts:22` | probe lifecycle |
| `useIsoLayoutEffect` | `packages/react/src/avatar/image/AvatarImage.tsx:61`, `:125`, `:131` | keepMounted sync, status reporting, unmount reset |
| `useStableCallback` | `packages/react/src/avatar/image/AvatarImage.tsx:120` | stable fan-out callback |
| `useTimeout` | `packages/react/src/avatar/fallback/AvatarFallback.tsx:24` | delay (per repo convention, not `window.setTimeout`) |
| `useTransitionStatus` | `packages/react/src/avatar/image/AvatarImage.tsx:54` | presence + starting/ending phases |
| `useOpenChangeComplete` | `packages/react/src/avatar/image/AvatarImage.tsx:135-144` | wait for exit animation before unmount |
| `useRenderElement` | `packages/react/src/avatar/root/AvatarRoot.tsx:34`, `packages/react/src/avatar/image/AvatarImage.tsx:166`, `packages/react/src/avatar/fallback/AvatarFallback.tsx:41` | shared render pipeline |
| `useAvatarRootContext` | `packages/react/src/avatar/image/AvatarImage.tsx:46`, `packages/react/src/avatar/fallback/AvatarFallback.tsx:22` | context read |

## Context providers/consumers

`AvatarRootContext` (`packages/react/src/avatar/root/AvatarRootContext.ts:10`) carries exactly two
members (`packages/react/src/avatar/root/AvatarRootContext.ts:5-8`):

- `imageLoadingStatus` — consumed by `AvatarFallback` (`packages/react/src/avatar/fallback/AvatarFallback.tsx:22`)
  for its `enabled` gate, and mirrored into Root's and Fallback's `state` objects for render props.
- `setImageLoadingStatus` — consumed by `AvatarImage` only (`packages/react/src/avatar/image/AvatarImage.tsx:46`),
  called from the fan-out callback (`:122`) and the unmount cleanup (`:133`).

`AvatarRoot` renders its own element *inside* the provider it creates
(`packages/react/src/avatar/root/AvatarRoot.tsx:41`), so the boundary is strictly one-directional:
Image writes → Root state updates → context value flows back to Fallback and to Root's own render
callbacks. Neither child ever reads the value the other wrote in the same pass through context
alone; Fallback re-renders because Root re-renders (context value identity changes via the
`useMemo` on `imageLoadingStatus`, `packages/react/src/avatar/root/AvatarRoot.tsx:26-32`).

`useAvatarRootContext` throws a `Base UI:`-prefixed error when a part is rendered outside
`<Avatar.Root>` (`packages/react/src/avatar/root/AvatarRootContext.ts:12-19`) — see the gaps
section: no test exercises this.

## DOM/portal strategy and why

- **No portal.** No file in the unit imports any portal utility; all three parts render in place
  (`packages/react/src/avatar/root/AvatarRoot.tsx:34-39`, `packages/react/src/avatar/image/AvatarImage.tsx:166-172`,
  `packages/react/src/avatar/fallback/AvatarFallback.tsx:41-47`). Element mapping is Root → `<span>`,
  Image → `<img>`, Fallback → `<span>` (behavior.md, *DOM structure & portal behavior*).
- **Detached probe image (default mode).** The defining DOM decision: in `!keepMounted` mode the
  real `<img>` is absent from the document until `mounted` (i.e. loaded), and loading is carried by
  `new window.Image()` configured with the same `referrerPolicy`/`crossOrigin`/`sizes`/`srcset`/`src`
  (`packages/react/src/avatar/image/useImageLoadingStatus.ts:33`, `:46-58`). Why: preloading
  outside the layout keeps a broken or half-loaded image from ever painting, keeps the fallback as
  the sole accessible name until the image is displayable, and gives clean mount/unmount animation
  boundaries because the real element appears exactly at `'loaded'` and animates out before
  `setMounted(false)` (via `useOpenChangeComplete`).
- **In-place loading (`keepMounted`).** The probe is skipped and the real `<img>` loads in the
  document (`packages/react/src/avatar/image/useImageLoadingStatus.ts:22-25`), which is the only way
  `loading="lazy"` and framework image components (`next/image`) can work — their machinery only
  activates for in-DOM elements (documented on the prop, `packages/react/src/avatar/image/AvatarImage.tsx:198-201`).
  The costs are handled explicitly: `aria-hidden` while not loaded (`:110`), `data-loading`/`data-error`
  (`:106-107`), and the `transitionStatus` mask (`:150`).
- **Attribute ordering as a compatibility shim.** `props: [renderedStatusProps, elementProps, sourceProps]`
  (`packages/react/src/avatar/image/AvatarImage.tsx:169`) is merged right-to-left by `mergePropsN`
  (plain props: rightmost wins; `packages/react/src/merge-props/mergeProps.ts:14-17`), so `src`
  lands on the element *after* `loading`/`sizes`/`srcSet`. React ≤18 sets attributes in props order
  and Safari/Firefox begin fetching at `src`, ignoring a `loading` that arrives later — hence the
  comment and the deliberate destructure-order (`packages/react/src/avatar/image/AvatarImage.tsx:37-44`)
  and the conditional `sourceProps` construction (`:154-164`). User `elementProps` sit in the
  middle: they override the internal status attributes (which is why an explicit `aria-hidden`
  survives, behavior.md, *Accessibility*), while event handlers are chained rather than replaced —
  the user's handler runs first and can call `event.preventBaseUIHandler()` to cancel the internal
  status update (`packages/react/src/merge-props/mergeProps.ts:221-250`, `:268-274`). That chaining
  is the entire implementation of the cancellation semantics in behavior.md, *Events*.
- **State → attribute mapping.** `imageLoadingStatus` is intentionally mapped to `null` on every
  part (`packages/react/src/avatar/root/stateAttributesMapping.ts:1-3`), so no
  `data-image-loading-status`-style attribute ever leaks to the DOM; the public attribute surface is
  only the hand-picked `data-loading`/`data-error` (keepMounted image) plus the transition attributes
  re-exported for docs (`packages/react/src/avatar/image/AvatarImageDataAttributes.ts:6-18`), which
  come from `transitionStatusMapping` merged into the image's mapping
  (`packages/react/src/avatar/image/AvatarImage.tsx:16-19`).

## Dependencies on other Base UI internals

Direct imports of the unit, by consumer:

- **Render kernel:** `internals/useRenderElement` — `packages/react/src/avatar/root/AvatarRoot.tsx:4`,
  `packages/react/src/avatar/image/AvatarImage.tsx:7`, `packages/react/src/avatar/fallback/AvatarFallback.tsx:5`.
  All three parts lean on it for: `render` prop evaluation (element or callback), state→data-attribute
  derivation via `stateAttributesMapping`, ref merging (array form for the image's dual refs,
  `packages/react/src/avatar/image/AvatarImage.tsx:168`), and the `enabled` flag that returns `null`
  instead of an element (`packages/react/src/internals/useRenderElement.tsx:40-42`) — `enabled` is
  the presence gate for both Image and Fallback.
- **Animation kernel:** `internals/useTransitionStatus` (`packages/react/src/avatar/image/AvatarImage.tsx:13`)
  and `internals/useOpenChangeComplete` (`packages/react/src/avatar/image/AvatarImage.tsx:11`),
  plus `internals/stateAttributesMapping`'s `transitionStatusMapping` for the
  `data-starting-style`/`data-ending-style` hooks (`packages/react/src/avatar/image/AvatarImage.tsx:12`).
- **Utils:** `@base-ui/utils/useIsoLayoutEffect` (`packages/react/src/avatar/image/AvatarImage.tsx:4`,
  `packages/react/src/avatar/image/useImageLoadingStatus.ts:3`), `@base-ui/utils/useStableCallback`
  (`packages/react/src/avatar/image/AvatarImage.tsx:3`), `@base-ui/utils/useTimeout`
  (`packages/react/src/avatar/fallback/AvatarFallback.tsx:3`), `internals/noop`'s `NOOP`
  (`packages/react/src/avatar/image/useImageLoadingStatus.ts:4`), and `internals/types`'
  `BaseUIComponentProps` on every part.
- **Transitive (via the kernels above, relevant for a port):** `useMergedRefs`/`useMergedRefsN`,
  `mergeProps`/`mergePropsN`/`mergeClassNames`, `mergeObjects`, `getReactElementRef`,
  `useAnimationsFinished`, `AnimationFrame` (`packages/react/src/internals/useTransitionStatus.ts:4`).
- **Explicitly not used:** `floating-ui-react` (no positioning anywhere in the unit) and
  `use-render` (the `render` prop is handled entirely by `useRenderElement`).

For `ralph/scripts/generate-todo.mjs` dependency computation, avatar's precise internal surface is:
`useRenderElement`, `useTransitionStatus`, `useOpenChangeComplete`, `stateAttributesMapping` +
`TransitionStatusDataAttributes`, `noop`, `types`, and the utils `useIsoLayoutEffect`,
`useStableCallback`, `useTimeout` — and nothing else. No focus, portal, or popover machinery.

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit:

1. **Missing-context error.** `useAvatarRootContext` throws `'Base UI: AvatarRootContext is missing. Avatar parts must be placed within <Avatar.Root>.'` (`packages/react/src/avatar/root/AvatarRootContext.ts:14-18`). No test renders a part outside Root; behavior.md has no assertion for it. The port must decide whether to reproduce the throw.
2. **Type-only API guarantees.** `packages/react/src/avatar/Avatar.spec.tsx:5-33` asserts at compile time that `state.imageLoadingStatus` is exposed to render callbacks on Root, Image, and Fallback, and that `props.src`/`props.alt` are typed on Image's render callback. No runtime test renders with a callback and reads `state`, so this public surface is guaranteed only by types; a fixture stage cannot discover it from runtime behavior.
3. **Status-attribute suppression.** `avatarStateAttributesMapping` maps `imageLoadingStatus` to `null` (`packages/react/src/avatar/root/stateAttributesMapping.ts:1-3`) on all parts. Tests assert `data-loading`/`data-error` *presence* on the keepMounted image, but nothing asserts the *absence* of a generic status attribute on Root/Fallback — the suppression is unobserved behavior.
4. **Probe re-runs on more than `src`.** The hook's dependency array includes `srcSet`, `sizes`, `crossOrigin`, and `referrerPolicy` (`packages/react/src/avatar/image/useImageLoadingStatus.ts:68`), so changing `sizes` alone re-creates the probe and resets status to `'loading'`. behavior.md documents resets only via `src`; config-prop-only resets are untested.
5. **Default-mode cached load plays a one-frame enter phase.** In default mode the first render is `'idle'`, so `useTransitionStatus` initializes `mounted = false` (`packages/react/src/internals/useTransitionStatus.ts:29`); when the probe's fast-path resolves `'loaded'` in a layout effect, the render-phase branch sets `'starting'` (`:31-34`) before an animation frame clears it. Only the keepMounted/hydration path pre-seeds `mounted` to skip this (`packages/react/src/avatar/image/AvatarImage.tsx:84-88`). behavior.md pins "no replay" only for SSR hydration; whether a client-side cached default-mode image shows `data-starting-style` for a frame is unpinned by tests.
6. **Unconditional `crossOrigin` assignment on the probe.** `image.crossOrigin = crossOrigin ?? null` runs even when the prop is absent (`packages/react/src/avatar/image/useImageLoadingStatus.ts:49`), unlike the guarded assignments for `referrerPolicy`/`sizes`/`srcSet`/`src` around it. Unobservable in tests; a porting detail that could silently differ.
7. **Barrel/type module shape.** `packages/react/src/avatar/index.ts:1-5` re-exports the namespace plus all part types;
`packages/react/src/avatar/index.parts.ts:1-3` aliases the class names to `Root`/`Image`/`Fallback`, and `packages/react/package.json:33` maps the `./avatar` subpath. No test constrains this export shape beyond imports resolving.
