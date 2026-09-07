# Scroll Area implementation spec

Mined from the non-test source under `packages/react/src/scroll-area/` (all 22 files). Companion
to `specs/library/scroll-area/behavior.md` (the WHAT); this document is the WHY/HOW. Behavior
already documented there is cited by section name, not restated. The unit's `TODO.md` entry
(`TODO.md:491-497`) has no `wraps-external:` field, so there is no external-package delegation —
everything below is in-repo.

## State machine / hooks used

There is no reducer and no `useControlled` — consistent with behavior.md's "State model" section
(no controlled/uncontrolled API). Exactly one component owns React state: `ScrollAreaRoot`. Every
other part is stateless; `Scrollbar`, `Thumb`, `Content`, and `Corner` exist to attach refs,
mount events, and surface attributes. The interesting state lives in three places:

### 1. Root: React state + the ref-based gesture latch

`ScrollAreaRoot` holds ten `useState` slices (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:55-63`):
`hovering`, `scrollingX`/`scrollingY` (deliberately split per axis — the mechanism behind the
per-axis isolation and independent timers in behavior.md's "State model" section), `touchModality`,
`hasMeasuredScrollbar`, `cornerSize`, `thumbSize`, `overflowEdges`, and `hiddenState`
(initialized `{ x: true, y: true, corner: true }`, `packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:21` —
nothing is visible until the first measurement pass).

The drag gesture itself is a state machine built entirely on refs
(`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:73-80`): `activePointerIdRef` (which
pointer owns the latch), `startYRef`/`startXRef` + `startScrollTopRef`/`startScrollLeftRef`
(drag origin), `currentOrientationRef`, `scrollPositionRef` (last scroll coords for delta
computation), and `savedSnapTypeRef` (scroll-snap suppression latch). Refs, not state, because
`pointermove` fires at gesture frequency and must not re-render; React state is touched only at
latch/unlatch boundaries. Transitions:

- **Latch** — `handlePointerDown` (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:121-155`):
  primary button only (`.tsx:122-124`); multi-pointer arbitration at `.tsx:126-136` — if a pointer
  is already latched *and* the active thumb still `hasPointerCapture`, the second pointer is
  ignored (behavior.md's "Multi-pointer" bullet); if capture was silently lost, the new pointer
  takes over the latch (behavior.md's "Lost capture" bullet). Orientation is read from the
  `data-orientation` DOM attribute on `event.currentTarget` (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:141-143`),
  not from React state — this is how one shared handler serves both scrollbars and the thumb of
  either (the attribute is rendered by the default state→attribute path, see the data-attribute
  engine below). It records start coords/scroll offsets, calls `disableViewportSnap`, and
  `setPointerCapture` on the *thumb* element (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:152-154`)
  — capture on the thumb is what keeps drags alive when the pointer leaves the track, and what
  makes the track-press handoff (jump-to-click then keep dragging) work with no extra code.
- **Move** — `handlePointerMove` (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:184-236`):
  filtered by `pointerId`; the missed-release fallback at `.tsx:189-197` treats a move with the
  primary `buttons` bit unset as an implicit release (behavior.md's "Missed release" bullet —
  the scroll freeze/snap restore/`data-scrolling` clear all happen at once precisely because the
  fallback routes through the same `handlePointerUp`); the degenerate-track guard at `.tsx:216-221`
  maps `maxThumbOffset <= 0` to `scrollRatio = 0` instead of dividing into `Infinity`/`NaN`
  (behavior.md's "Degenerate geometry" edge case); then proportional assignment, `preventDefault`,
  and `startScrolling(vertical)`.
- **Unlatch** — `handlePointerUp` (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:157-182`):
  filtered by `pointerId`; clears the drag axis's scrolling state immediately (`.tsx:163-166`)
  rather than waiting out `SCROLL_TIMEOUT` (the uniform-release-path guarantee in behavior.md's
  "Missed release" bullet); restores the saved `scrollSnapType` (`.tsx:168-173`); and only calls
  `releasePointerCapture` if the thumb still holds it (`.tsx:179-181`) — the guard that makes
  `pointercancel` (which releases capture implicitly) safe (behavior.md's "Pointer cancel" bullet).

Scroll-snap suppression is its own mini-latch: `disableViewportSnap`
(`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:107-119`) saves the inline
`scrollSnapType` only once (`savedSnapTypeRef.current === null` guard), so the second
`disableViewportSnap` call in the track-press path (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:172-176`)
is a no-op and a second pointer can't clobber the saved value with `'none'`. This produces
behavior.md's scroll-snap bullets (drag between snap points, re-snap on release).

Per-axis "scrolling" timers: `startScrolling`
(`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:82-90`) arms one of two `useTimeout`
instances (`scrollYTimeout`/`scrollXTimeout`, `.tsx:50-51`) with `SCROLL_TIMEOUT`
(`packages/react/src/scroll-area/constants.ts:1`); re-arming restarts the countdown, which is the
"A scroll within `SCROLL_TIMEOUT - 1` keeps the attribute" behavior in behavior.md's "State
model" section. `handleScroll` (`.tsx:92-105`) is delta-based: it compares the incoming coords
against `scrollPositionRef` and only arms an axis whose offset actually changed — an additional
gate beyond the user-interaction gate in the Viewport.

Touch/modality and hover: `handleTouchModalityChange` (`.tsx:238-240`) derives `touchModality`
from `pointerType` on root `onPointerDown` (`.tsx:269`); `handlePointerEnterOrMove` (`.tsx:242-249`)
updates both modality and `hovering`, where hovering is computed with shadow-DOM-safe
`contains(rootRef.current, event.target)` — note this makes hovering a *root-scope* notion (any
root child counts), while behavior.md's "State model" section describes it from the
viewport-targeted tests. `onPointerLeave` clears it (`.tsx:270-272`).

The `state` object (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:251-263`) and the
context value (`.tsx:287-341`, memoized on the full dep list) are both `React.useMemo`'d — see
the context section for why that matters for behavior.md's "Scroll-triggered re-renders" edge case.

### 2. Viewport: the measurement engine + the user-intent classifier

`computeThumbPosition` (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:116-279`,
a `useStableCallback`) is the single recomputation routine everything funnels into. Its pipeline:

1. Read viewport metrics (`viewportEl.scrollHeight/clientHeight/scrollTop/scrollLeft`, `.tsx:128-133`),
   stash them into `lastMeasuredViewportMetricsRef` (`.tsx:106-111`, NaN-seeded), and mark
   `setHasMeasuredScrollbar(true)` on the first pass (`.tsx:142-144`) — this flag drives the
   scrollbar/thumb visibility sequencing in behavior.md's "State model" section ("Scrollbar
   visibility is computed at mount before the first ResizeObserver measurement").
2. Bail when either scrollable dimension is 0 (`.tsx:146-148`).
3. Derive `hiddenState` via `getHiddenState` (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:440-449`):
   `clientHeight >= scrollHeight` per axis, `corner = x || y` — overflow is the *absence* of
   hidden, so no-overflow means every attribute clears (behavior.md's overflow bullets).
4. Compute direction-aware start/end distances with `normalizeScrollOffset`
   (`packages/react/src/utils/scrollEdges.ts:9-32`), which clamps and applies a ±1px edge
   tolerance with a tie-break when both edges are within tolerance — the mechanism behind
   "Near-edge offsets are treated as fully scrolled" in behavior.md's "State model" section.
   RTL is handled by negating `scrollLeft` before normalizing
   (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:160-167`) so all downstream
   math is direction-agnostic.
5. Thumb sizing (`.tsx:189-205`): subtract scrollbar *padding* and thumb *margin* (via `getOffset`)
   from the viewport dimension, cap by the track size minus the corner only when the corner
   hasn't been sized yet (`.tsx:183-187` — the bootstrap handoff between the corner and scrollbar
   layout), scale by the content ratio, and floor at `MIN_THUMB_SIZE = 16`
   (`packages/react/src/scroll-area/constants.ts:2`). Track *margin* is never subtracted — the
   math reads `offsetHeight/offsetWidth` of the scrollbar, which excludes margins — the exact
   asymmetry behavior.md's thumb-geometry bullets document.
6. Position the thumb imperatively: `translate3d` transforms per axis (`.tsx:212-246`) with
   `applyOverscrollThumb` (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:470-490`)
   handling Safari rubber-band feedback: it clamps the scroll, computes the damped size
   `size * content / (content + |overscroll|)`, writes it into the thumb-size CSS variable only
   while overscrolled (empty string restores the resting `var(...)`, `.tsx:484`), and shifts the
   offset so the thumb stays pinned to the overscrolled edge. All of this is direct DOM writes —
   no React state — which is why overscroll feedback in behavior.md's "Edge cases" section costs
   zero re-renders.
7. Write the four overflow CSS variables on the viewport (`.tsx:248-257`), size the corner
   (`.tsx:259-267`), and publish `hiddenState`/`overflowEdges` through the root setters — every
   setState wrapped in `pickState` (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:455-463`).

`pickState` is the key trick: it returns the *previous* object when the next is shallow-equal, so
`setState` bails out and the memoized root context (see below) keeps its identity. Combined with
the fact that per-scroll updates otherwise go straight to the DOM (CSS vars, transforms), this is
the mechanism behind "Scroll-triggered re-renders" in behavior.md's "Edge cases" section.

Recompute scheduling — four `useIsoLayoutEffect`s:

- Register the overflow CSS variables as non-inheriting custom properties, once per app via a
  module-level flag (`.tsx:281-283` → `removeCSSVariableInheritance`, `.tsx:29-67`), skipped on
  WebKit where `inherits: false` breaks child `inherit` opt-in (`.tsx:44-48`). A rendering-
  performance optimization (the comment links motion.dev's animation-performance tier list), not
  behavior.
- `queueMicrotask(computeThumbPosition)` keyed on `[hiddenState, direction, ...thresholds]`
  (`.tsx:285-297`) — a microtask (not a sync call) so hidden-state toggles that mount/unmount
  scrollbars get to attach their refs before measurement; re-running on `direction` is the
  mechanism behind behavior.md's runtime-RTL-flip recompute, and on each threshold field is the
  mechanism behind "raising the threshold clears the edge attribute without a new scroll event".
- A mount-time `matches(':hover')` check (`.tsx:299-305`) — the "viewport already hovered at
  mount" case in behavior.md's "State model" section (`onMouseEnter` doesn't fire on load).
- A `ResizeObserver` on the viewport (`.tsx:307-356`) with a first-delivery dedupe: on the first
  observer callback, if the metrics match what the mount-scheduling pass already measured, skip
  (`.tsx:313-329`). Plus `waitForAnimationsTimeout.start(0)` → `getAnimations({ subtree: true })`
  → `Promise.allSettled(finished)` → recompute, with `.catch(() => {})` so a post-unmount settle
  is swallowed (`.tsx:338-350`) — the "subtree animation finishing" pickup and "unmount during
  async work" safety in behavior.md's "Edge cases" section.

The user-intent classifier: `programmaticScrollRef` (`.tsx:105`) starts `true` and is flipped
`false` by `handleUserInteraction` (`.tsx:358-360`) wired to `onWheel`/`onPointerMove`/
`onPointerEnter`/`onKeyDown` (`.tsx:401-404`). `onScroll` (`.tsx:372-400`) always recomputes the
thumb, but only credits the user — calling root `handleScroll` — when `touchModality ||
!programmaticScrollRef` (`.tsx:384-389`). That single condition is the mechanism behind two
behavior.md "State model" behaviors: "a scroll with no prior pointer interaction does not set
`data-scrolling`" (programmatic flag still true) and the WebKit touch gate ("Treat every scroll in
touch modality as user-driven", `.tsx:379-383` — a touch that catches momentum dispatches no
events at all). A 100 ms `scrollEndTimeout` debounce restores the programmatic flag
(`.tsx:395-399`) so momentum scrolling (no further interaction events) stays user-driven.

### 3. Scrollbar: conditional render + wheel/track interaction host

- `shouldRender = keepMounted || !isHidden` (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:67`)
  with an early `return null` (`.tsx:228-230`) — the `keepMounted` semantics in behavior.md's
  "Public API surface" section, and the reason every no-overflow test needs `keepMounted`.
- `hideTrackUntilMeasured = !hasMeasuredScrollbar && !keepMounted` (`.tsx:65`) → `visibility:
  hidden` inline style (`.tsx:204`); the thumb applies the same gate
  (`packages/react/src/scroll-area/thumb/ScrollAreaThumb.tsx:49`). This is the mount-sequencing
  mechanism behind behavior.md's "Scrollbar visibility" bullet: the track is invisible until the
  first `computeThumbPosition` marks `hasMeasuredScrollbar`.
- Wheel interception is a native, non-passive listener (`addEventListener` from
  `@base-ui/utils/addEventListener`, `.tsx:117`) registered in a `React.useEffect`
  (`.tsx:69-118`): guards viewport presence, `ctrlKey` (browser zoom passthrough), and zero delta
  (`.tsx:81-91`); computes a direction-aware scroll range — RTL horizontal is `[-maxScroll, 0]`
  (`.tsx:93-98`) — matching behavior.md's RTL wheel bullets; **edge-chains by early-returning
  without `preventDefault`** (`.tsx:101-105`) so an unconsumed wheel propagates to the page (the
  cancel-only-when-consumed semantics in behavior.md's "Events" section); otherwise clamps the
  assignment and calls `handleScroll` (`.tsx:109-114`), which is why consumed wheels mark
  `data-scrolling` and edge-chained ones don't. The effect re-registers when `shouldRender` or
  `direction` changes — the "Wheel registration is live" behavior.
- Track pointerdown (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:123-189`)
  implements jump-to-click: `getTarget` (shadow retargeting) + `contains(thumbEl, target)` exclude
  thumb hits even when React retargets the synthetic event across a shadow boundary
  (`.tsx:128-135` — the composedPath test in behavior.md's "Events" section); centers the thumb on
  the click (`.tsx:153-155`); bails on `maxThumbOffset <= 0` (`.tsx:161-167`); disables snapping
  *before* the assignment so the jump isn't quantized (`.tsx:172-176`); writes the position with
  an RTL split — `-(1 - scrollRatio) * maxScrollDistance` maps the click position onto the
  negative range (`.tsx:178-184`); then calls `handleScroll` and *delegates to the root's
  `handlePointerDown`* (`.tsx:186-188`) so a track press chains directly into the thumb-drag latch.

### The data-attribute engine

All parts share `scrollAreaStateAttributesMapping`
(`packages/react/src/scroll-area/root/stateAttributes.ts:7-15`) passed through `useRenderElement`
(`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:284`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:411`,
`packages/react/src/scroll-area/content/ScrollAreaContent.tsx:63`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:225`).
`getStateAttributesProps` (`packages/react/src/internals/getStateAttributesProps.ts:24-28`) has a
default path — boolean `true` → bare `data-<key-lowercased>`, other truthy → `data-<key>="value"`
— which is why single-word keys (`scrolling`, `hovering`, `orientation`) need no mapping entry.
The custom mapping exists for multi-word keys whose attribute names differ (`hasOverflowX` →
`data-has-overflow-x`, etc.) and returns `null` for `cornerHidden`
(`packages/react/src/scroll-area/root/stateAttributes.ts:14`) because the corner's visibility is
expressed by *not rendering* (`packages/react/src/scroll-area/corner/ScrollAreaCorner.tsx:38-40`),
never by an attribute. States per part: Root/Viewport/Content share the root state shape;
Scrollbar spreads `viewportState` and adds `hovering`/its-axis `scrolling`/`orientation`
(`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:57-62`); Thumb carries only
`scrolling` + `orientation` (`packages/react/src/scroll-area/thumb/ScrollAreaThumb.tsx:34-37`).
See the untested-gap section for the cross-axis consequence of the Scrollbar spread.

### Hooks by call site

- `ScrollAreaRoot`: `React.forwardRef` (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:34`),
  `useBaseUiId` (`.tsx:48`), `useTimeout` ×2 (`.tsx:50-51`), `useCSPContext` (`.tsx:53`),
  `React.useState` ×10 (`.tsx:55-63`), refs ×15 (`.tsx:65-80`), `useStableCallback` ×5 for all
  gesture/scroll handlers (`.tsx:92,113,121,157,184`), `React.useMemo` for `state` (`.tsx:251`)
  and the context value (`.tsx:287`), `useRenderElement` (`.tsx:280`).
- `ScrollAreaViewport`: root-context read (`.tsx:81-101`), `useDirection` (`.tsx:103`),
  `React.useRef` ×2 (`.tsx:105-111`), `useTimeout` ×2 (`.tsx:113-114`), `useStableCallback` for
  `computeThumbPosition` (`.tsx:116`), `useIsoLayoutEffect` ×4 (`.tsx:281,285,299,307`),
  `useRenderElement` (`.tsx:407`), `React.useMemo` context (`.tsx:414`).
- `ScrollAreaScrollbar`: root-context read (`.tsx:35-53`), `useDirection` (`.tsx:64`),
  `React.useEffect` for the wheel listener (`.tsx:69`), `useRenderElement` (`.tsx:221`).
- `ScrollAreaThumb`: root- and scrollbar-context reads (`.tsx:20-31`), `useRenderElement` (`.tsx:39`).
- `ScrollAreaContent`: viewport- and root-context reads (`.tsx:23-24`), refs (`.tsx:26-27`),
  `useIsoLayoutEffect` for its `ResizeObserver` (`.tsx:29-58`), `useRenderElement` (`.tsx:60`).
- `ScrollAreaCorner`: root-context read (`.tsx:19`), `useRenderElement` (`.tsx:21`).
- Notably absent unit-wide: `useControlled`, `useAnimationFrame`, reducers, and any state in
  Thumb/Content/Corner.

`ScrollAreaContent`'s observer (`.tsx:29-58`) mirrors the viewport's: skip the initial
ResizeObserver fire unless `computeOnInitialResizeRef` (initialized from `hasMeasuredScrollbar`,
`.tsx:27`) says the content mounted after the viewport's first measurement — the "content mounted
later" pickup without double-computing on the normal path.

## Context providers/consumers

Three nested contexts enforce the part ancestry that behavior.md's "Edge cases" section records
as error contracts:

1. **`ScrollAreaRootContext`** (`packages/react/src/scroll-area/root/ScrollAreaRootContext.ts:48-50`),
   provided by Root wrapping the injected `<style>` element and the root div
   (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:343-348`). Accessor
   `useScrollAreaRootContext` throws the "ScrollAreaRootContext is missing" error
   (`packages/react/src/scroll-area/root/ScrollAreaRootContext.ts:52-60`). Consumed by Viewport
   (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:81-101`), Scrollbar
   (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:35-53`), Thumb
   (`packages/react/src/scroll-area/thumb/ScrollAreaThumb.tsx:20-29`), Content
   (`packages/react/src/scroll-area/content/ScrollAreaContent.tsx:24`), Corner
   (`packages/react/src/scroll-area/corner/ScrollAreaCorner.tsx:19`). The boundary crosses:
   - *Down*: all six refs (`viewportRef`, `scrollbarXRef`/`YRef`, `thumbXRef`/`YRef`,
     `cornerRef`), the four gesture handlers (`handlePointerDown/Move/Up`, `handleScroll`) plus
     `disableViewportSnap`, measurement outputs (`thumbSize`, `cornerSize`, `hiddenState`,
     `overflowEdges`), flags (`scrollingX/Y`, `hovering`, `touchModality`,
     `hasMeasuredScrollbar`), the normalized `overflowEdgeThreshold`, `rootId`, and
     `viewportState` (the root state object, consumed for child data attributes).
   - *Up*: five setters (`setCornerSize`, `setThumbSize`, `setHiddenState`, `setOverflowEdges`
     called by Viewport's `computeThumbPosition`; `setHasMeasuredScrollbar` also Viewport;
     `setHovering` called by both Root and Viewport's mount-hover effect) and ref attachment
     (Scrollbar/Thumb/Corner merge their forwarded refs with the root-owned refs —
     `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:222`,
     `packages/react/src/scroll-area/thumb/ScrollAreaThumb.tsx:40`,
     `packages/react/src/scroll-area/corner/ScrollAreaCorner.tsx:22`).
   The design inversion worth noting: **Root owns all gesture math and state; Scrollbar and Thumb
   are thin event-mounting surfaces that delegate** (Thumb's handlers are pure context
   pass-through, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.tsx:43-47`). That is the
   repo's "share handlers through context" guideline in its strongest form, and it is why the
   multi-pointer/missed-release/cancel semantics live in exactly one place.
   Stability: the context value is memoized on a large dep list
   (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:287-341`) where every entry is either
   a `useStableCallback` product or state wrapped in `pickState` — so scroll frames that change
   nothing bail out of every setState and never rebuild the context (the `ContextProbe`
   commit-count guarantee in behavior.md's "Edge cases" section).

2. **`ScrollAreaViewportContext`** (`packages/react/src/scroll-area/viewport/ScrollAreaViewportContext.ts:8-10`),
   provided by Viewport with the single member `{ computeThumbPosition }`
   (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:414-425`), stable because
   `computeThumbPosition` is a `useStableCallback`. Consumed only by Content
   (`packages/react/src/scroll-area/content/ScrollAreaContent.tsx:23`) — it is the channel that
   lets a content resize trigger recomputation without exposing measurement plumbing. Its
   accessor throws the "ScrollAreaViewportContext missing" error
   (`packages/react/src/scroll-area/viewport/ScrollAreaViewportContext.ts:12-20`).

3. **`ScrollAreaScrollbarContext`** (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbarContext.ts:6-8`),
   provided by Scrollbar with just the orientation string
   (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:232-236`). Consumed only by
   Thumb (`packages/react/src/scroll-area/thumb/ScrollAreaThumb.tsx:31`) to pick its axis's ref,
   CSS variable, and scrolling flag. Accessor throws the
   "ScrollAreaScrollbarContext is missing" error
   (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbarContext.ts:10-18`).

The provider placements are conditional in one case: the Scrollbar provider (and its children,
including Thumb) only render when `shouldRender`
(`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:228-236`), so an unmounted
scrollbar's children can never observe the missing context.

## DOM/portal strategy and why

No portals anywhere — every part renders in place (consistent with behavior.md's "DOM structure
& portal behavior" section). Nothing needs overlay positioning math: the scrollbars and corner
are absolutely positioned *within the root* (`position: 'relative'` on Root,
`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:274`; `position: 'absolute'` on
Scrollbar/Corner, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:200`,
`packages/react/src/scroll-area/corner/ScrollAreaCorner.tsx:27`), so the custom scrollbars
overlay the viewport instead of participating in layout — which is exactly why behavior.md's
"DOM structure" section finds no scrollbar-compensation padding anywhere.

Fixed DOM shape (all six parts render `div` via `useRenderElement`'s first argument):

- Root: `role: 'presentation'`, pointer handlers, corner CSS vars
  (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:265-278`).
- Viewport: inline `overflow: 'scroll'` (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:370`)
  plus the `styleDisableScrollbar` class (`.tsx:368`). The pairing is the whole trick: forced
  native scrollability guarantees real `scrollTop/scrollLeft/scroll*` geometry and native
  gesture/animation behavior, while the class (backed by a `<style>` element Root injects once,
  CSP-gated via `useCSPContext` — `packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:53,345`,
  CSS at `packages/react/src/utils/styles.tsx:7-9`) hides the native scrollbars so only the
  custom ones are visible.
- Viewport `tabIndex: hiddenState.x && hiddenState.y ? -1 : 0`
  (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:367`) — the tabbability-
  tracks-overflow behavior in behavior.md's "Focus management" section, with the a11y rationale
  linked in-source.
- Content **is** the "contentWrapper" `viewport.firstElementChild` that behavior.md's DOM section
  describes — there is no built-in intermediate wrapper; the viewport renders arbitrary children
  directly, and `ScrollAreaContent` merely adds `minWidth: 'fit-content'`
  (`packages/react/src/scroll-area/content/ScrollAreaContent.tsx:68`) so the wrapper can grow
  wider than the viewport (horizontal overflow must be measurable) plus a `ResizeObserver`.
- Scrollbar geometry is logical-property CSS anchored to the root: `top/bottom` +
  `insetInlineEnd` for vertical, `insetInlineStart/End` + `bottom` for horizontal, with the
  corner CSS variables as the counter-insets
  (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:205-217`) — logical
  properties mirror RTL with zero JS, and defining the corner vars on the Root
  (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:275-276`) makes them available to both
  scrollbars and the corner.
- Thumb sizing flows through CSS variables rather than inline px: the Scrollbar writes
  `--scroll-area-thumb-height`/`--scroll-area-thumb-width` into its own style
  (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:210,216`), the Thumb consumes
  them via `var(...)` (`packages/react/src/scroll-area/thumb/ScrollAreaThumb.tsx:50-52`), and the
  Viewport temporarily overrides the variable during overscroll (empty string restores the
  resting value — `packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:484`). This
  three-layer indirection is why thumb geometry updates and overscroll feedback need no React
  re-render of the thumb.
- `data-id` linking (`${rootId}-viewport`, `${rootId}-scrollbar` —
  `packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:364`,
  `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:121`) and `aria-hidden: true`
  on Scrollbar/Corner (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:122`,
  `packages/react/src/scroll-area/corner/ScrollAreaCorner.tsx:25`) — the accessibility posture in
  behavior.md's "Accessibility" section (custom scrollbar chrome hidden, real scrolling left to
  the viewport).

## Dependencies on other Base UI internals

The unit's `TODO.md` entry (`TODO.md:491-497`) has **no `wraps-external:` field** (crate:
`leptos-ui`), so there is no external-package delegation to name — all dependencies below are
in-repo and must be ported/reimplemented by the Rust crate work.

In-package (`packages/react/src`):

- `internals/useRenderElement` — all six parts; it *is* the render pipeline: props-array merging,
  state→data attributes, class/style resolution, `render` prop handling, and ref-array merging
  (every part passes `ref: [forwardedRef, <root-owned ref>]`). Pulls
  `@base-ui/utils/useMergedRefs`, `getReactElementRef`, `mergeObjects`, `warn`,
  `internals/getStateAttributesProps`, and `merge-props` transitively.
- `internals/getStateAttributesProps` — the state→attribute engine behind the shared mapping
  (type import at `packages/react/src/scroll-area/root/stateAttributes.ts:1`; default path at
  `packages/react/src/internals/getStateAttributesProps.ts:24-28`).
- `internals/types` — `BaseUIComponentProps`/`HTMLProps` that every part's props interface
  extends (e.g. `packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:5,386`).
- `internals/useBaseUiId` → `@base-ui/utils/useId` — `base-ui-`-prefixed `rootId` for `data-id`
  linking (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:11,48`).
- `internals/csp-context/CSPContext` — `nonce`/`disableStyleElements` for the injected
  scrollbar-hiding stylesheet (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:14,53,345`).
- `internals/direction-context/DirectionContext` (`useDirection`) — RTL scroll math in the
  wheel/track/drag/overscroll paths and recompute-on-direction-change
  (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:12,103`,
  `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:10,64`). This is the unit's
  only RTL input (behavior.md's "Public API surface" notes there is no `dir` prop).
- `utils/styles` (`styleDisableScrollbar`) — element injected by Root, className on Viewport
  (`packages/react/src/utils/styles.tsx:3-11`).
- `utils/scrollEdges` (`normalizeScrollOffset` + `SCROLL_EDGE_TOLERANCE_PX = 1`) — edge-distance
  normalization (`packages/react/src/utils/scrollEdges.ts:3-32`).
- `floating-ui-react/utils` — `contains` (Root hover check; Scrollbar thumb-hit exclusion) and
  `getTarget` (shadow-retarget-safe track presses): `packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:13,246`,
  `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:5,128-133`. This is the
  unit's entire floating-ui-react footprint — no positioning.

Cross-package (`@base-ui/utils`):

- `useStableCallback` — every handler that crosses an effect/event boundary (Root ×5, Viewport's
  `computeThumbPosition`).
- `useTimeout` — per-axis `SCROLL_TIMEOUT` timers (Root), scroll-end debounce + animation-settle
  (Viewport).
- `useIsoLayoutEffect` — Viewport ×4, Content ×1.
- `platform` — WebKit engine detection gating the `registerProperty` optimization
  (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:5,46`).
- `clamp` — direct in Viewport's overscroll math; indirectly via `scrollEdges`.
- `addEventListener` — the non-passive native wheel listener
  (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:117`).

Explicitly **not** used by this unit: `useControlled`, `use-render` (directly), portals/anchor
positioning, `useAnimationFrame`, stores, and imports from any other component. Aside from
`internals/` + `utils/` + the two tiny `floating-ui-react/utils` helpers, scroll-area is fully
self-contained.

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and the backward-looking audit; none of these are
pinned by the tests mined for behavior.md:

1. **Scrollbars carry the full root-state attribute set, contradicting behavior.md's wording.**
   `ScrollAreaScrollbar` spreads the entire `viewportState` into its state
   (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:57-62`) and the shared
   mapping emits every key (`packages/react/src/scroll-area/root/stateAttributes.ts:7-15`), so a
   vertical scrollbar *also* renders `data-has-overflow-x` and the `data-overflow-x-*` edges
   whenever horizontal overflow/edges exist. The tests only assert own-axis presence and
   own-axis-start absence — never cross-axis absence
   (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:953-961` — cited here
   only to show the assertion gap). behavior.md's "State model" phrase "each Scrollbar gets the
   attribute for its own axis only" is therefore stronger than the evidence; fixtures must follow
   the source (full state on scrollbars) unless the audit decides otherwise.
2. **`role="presentation"`** hardcoded on Root, Viewport, and Content
   (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:266`,
   `packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:363`,
   `packages/react/src/scroll-area/content/ScrollAreaContent.tsx:66`). behavior.md's
   "Accessibility" section records that no role is asserted anywhere (UNVERIFIED). Fixtures
   should expect it from source.
3. **`data-id` linking** (`${rootId}-viewport`, `${rootId}-scrollbar`; nothing on Root/Thumb/
   Content/Corner) — behavior.md's "Accessibility" section notes no id-linking is asserted. The
   `rootId &&` guards exist because `useBaseUiId` can return `undefined`
   (`packages/react/src/internals/useBaseUiId.ts:9-10`).
4. **Native-scrollbar suppression machinery.** The injected stylesheet
   (`.base-ui-disable-scrollbar{scrollbar-width:none} … ::-webkit-scrollbar{display:none}`,
   `packages/react/src/utils/styles.tsx:7-9`), its CSP-gated injection
   (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:345`), and the class on the Viewport
   (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:368`) are never directly
   asserted (behavior.md only asserts the *absence of compensation padding*). A port must still
   hide native scrollbars for the visual contract to hold.
5. **CSS.registerProperty `inherits: false` optimization**
   (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:29-67`): module-level
   once-flag, `<length>`/`0px` registration of the four overflow variables, and a WebKit skip
   because non-inheriting properties break child `inherit` opt-in in Safari. Pure performance
   plumbing invisible to behavior tests; a port must decide whether to replicate it and its
   Safari caveat.
6. **`MIN_THUMB_SIZE = 16` floor** (`packages/react/src/scroll-area/constants.ts:2`; applied at
   `packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:204-205,481`). behavior.md's
   "Degenerate geometry" tests exercise the `maxThumbOffset <= 0` guard but never pin the 16px
   floor value itself.
7. **The 100 ms scroll-end debounce constant**
   (`packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:395-399`) that restores
   `programmaticScrollRef`. Tests advance `SCROLL_TIMEOUT` (500) but never assert this
   restoration timing; momentum-scroll attribution in a port needs an equivalent rest heuristic.
8. **`getOffset`'s Safari RTL margin workaround**
   (`packages/react/src/scroll-area/utils/getOffset.ts:14-18`): x-axis margins are assumed
   symmetrical (`start * 2`) because Safari misreports `marginInlineEnd` in RTL. The thumb-size
   tests exercise `marginBlock` only; the RTL margin path is untested.
9. **`normalizeScrollOffset` tolerance details** (`packages/react/src/utils/scrollEdges.ts:3,17-30`):
   the ±1px `SCROLL_EDGE_TOLERANCE_PX` and the both-edges-within-tolerance tie-break. behavior.md
   pins only a 0.5px-from-edge case ("Near-edge offsets"), not the full tolerance band.
10. **`overflowEdgeThreshold` normalization** (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:408-421`):
    `|| 0` coercion (so `0`/falsy fields default cleanly) and `Math.max(0, …)` — negative
    thresholds clamp to 0; behavior.md exercises only positive numeric/object forms.
11. **`handleScroll`'s offset ≠ 0 gate** (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:92-105`):
    a scroll event that doesn't change the position arms no timer even after prior interaction —
    only exercised indirectly via the zero-delta wheel case.
12. **Hover is root-scoped, not viewport-scoped.** `handlePointerEnterOrMove` sets `hovering`
    whenever the target is any root descendant (`contains(rootRef.current, event.target)`,
    `packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:242-249`) — hovering over the track
    or content also sets `data-hovering` on scrollbars. behavior.md describes it as "pointer over
    the Viewport" because the tests target the viewport; the root-scope semantics are unpinned.
13. **Track-press chains into a drag.** After jump-to-click, the Scrollbar delegates to the
    root's `handlePointerDown` (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:186-188`),
    so holding and moving after a track press drags from the track. behavior.md documents track
    press and thumb drag as separate behaviors; the continuation itself is unasserted.
14. **Gesture latch depends on rendered `data-orientation`.** The root reads orientation from the
    DOM attribute of the pressed element
    (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:141-143`) rather than from context —
    a coupling to the state-attribute output surviving on custom `render` elements (both
    Scrollbar and Thumb render it via their state). Untested with custom renderers.
15. **`touchAction: 'none'` + `WebkitUserSelect`/`userSelect: 'none'` on the track**
    (`packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.tsx:201-203`): untested inline
    styles that are load-bearing for gesture fidelity (per the repo's styling guidelines).
