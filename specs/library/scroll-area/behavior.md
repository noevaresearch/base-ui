# Scroll Area behavior spec

Mined from the six scroll-area test files listed below. The `TODO.md` entry
(`TODO.md:495-501`) has no `wraps-external:` field, so the behavior below is derived entirely
from the component's own tests; there is no third-party package to delegate to. The unit is not
on the `needs-batched-mining: true` list, so this single file covers the whole scroll-area.

Files mined:
- `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx`
- `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx`
- `packages/react/src/scroll-area/content/ScrollAreaContent.test.tsx`
- `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx`
- `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx`
- `packages/react/src/scroll-area/corner/ScrollAreaCorner.test.tsx`

## Public API surface (props, parts, subcomponents)

- Parts exercised by the tests, all imported from the `@base-ui/react/scroll-area` namespace:
  `ScrollArea.Root`, `ScrollArea.Viewport`, `ScrollArea.Content`, `ScrollArea.Scrollbar`,
  `ScrollArea.Thumb`, `ScrollArea.Corner`. `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:3`, `packages/react/src/scroll-area/corner/ScrollAreaCorner.test.tsx:2`
- Every part renders a `div` (conformance `refInstanceof: window.HTMLDivElement` for all six).
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:57-60`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:13-18`, `packages/react/src/scroll-area/content/ScrollAreaContent.test.tsx:11-19`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:12-17`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:14-23`, `packages/react/src/scroll-area/corner/ScrollAreaCorner.test.tsx:30-48`
- `ScrollArea.Root`:
  - `overflowEdgeThreshold` — object form `{ xStart: 20, yStart: 5 }` (per-edge pixel
    thresholds). `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:728-747`
  - `overflowEdgeThreshold` — numeric form (`20`) applied to every edge.
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:773-792`
  - `style` (layout sizing drives all measured behavior) and `data-testid` forwarded.
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:69-74`
- `ScrollArea.Viewport`:
  - `onScroll` — user scroll callback, invoked on viewport scroll.
    `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:27-32`
  - `style` — consumer-set `scrollSnapType` participates in thumb-drag behavior (see Events).
    `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:477-487`
  - Arbitrary children (plain `div` or `ScrollArea.Content`) are the scrollable content.
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:70-72`, `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:583-587`
- `ScrollArea.Content`:
  - `render={<ContentWithoutRef />}` — element-form custom renderer whose component does not
    forward its ref is supported. `packages/react/src/scroll-area/content/ScrollAreaContent.test.tsx:59-76`
- `ScrollArea.Scrollbar`:
  - `orientation="vertical" | "horizontal"`. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:39-47`
  - `keepMounted` — keeps the scrollbar element mounted/in the DOM even with no Viewport or no
    overflow; every no-viewport/no-overflow scenario in the tests relies on it.
    `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:22-27`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:245-258`
  - `render={<ScrollbarWithoutRef />}` — element-form custom renderer that does not forward its
    ref is supported. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:49-69`
  - `aria-hidden={undefined}` overrides the default `aria-hidden="true"`.
    `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:29-37`
- `ScrollArea.Thumb`: renders inside `ScrollArea.Scrollbar`; no scroll-area-specific props beyond
  standard element props (`style`, `data-testid`, `onPointerMove`, `onPointerUp`) are exercised.
  `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:53-64`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:84-89`
- `ScrollArea.Corner`: no scroll-area-specific props exercised; `aria-hidden={undefined}`
  override supported. `packages/react/src/scroll-area/corner/ScrollAreaCorner.test.tsx:65-78`
- No `dir` prop on any part: RTL behavior comes from `DirectionProvider` plus the container's
  CSS `direction` style. `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:484-498`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:158-175`
- Error contracts (see Edge cases): `Thumb` requires `Scrollbar` context, `Viewport` requires
  `Root` context, `Content` requires `Viewport` context.
  `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:25-41`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:488-498`, `packages/react/src/scroll-area/content/ScrollAreaContent.test.tsx:22-38`

## State model (controlled/uncontrolled, defaults, transitions)

- No controlled/uncontrolled value API exists: there is no prop or callback through which the
  consumer sets scroll position or overflow state (tests mutate `scrollTop`/`scrollLeft` directly
  or via gestures). All state below is internal. UNVERIFIED — inferred from
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:509-526`, no test shows any
  controlled-value prop.
- Internal state is exposed only through DOM attributes, CSS variables, and computed styles:
  - `data-scrolling` (empty value) on Root, Viewport, Scrollbar, and Thumb while a user scroll is
    active; removed after `SCROLL_TIMEOUT` (imported from `../constants` and used as the fake
    clock tick). `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:8`, `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:84-88`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:155-159`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:270-296`
  - Per-axis isolation: a vertical scroll marks only the vertical scrollbar/thumb
    (`data-scrolling` on vertical, absent on horizontal) and vice versa.
    `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:96-103`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:602-609`
  - A scroll within `SCROLL_TIMEOUT - 1` keeps the attribute; each new scroll event in the other
    axis starts its own timer, and the attributes clear independently.
    `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:288-295`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:105-131`
  - `data-scrolling` requires recent user interaction: a scroll with no prior pointer
    interaction (programmatic scroll, as with `scrollTo()`) does not set it.
    `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:172-191`
  - Modality gate: after a `pointerdown` with `pointerType: 'touch'`, scroll events DO set
    `data-scrolling` even when the gesture delivered no events (WebKit momentum/rubber-band
    case); with `pointerType: 'mouse'` they do not; a mouse `pointermove` on the root flips
    modality back and restores suppression.
    `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:193-218`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:220-239`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:241-268`
  - `data-hovering` on Scrollbar while a mouse pointer is over the Viewport — including when the
    viewport is already hovered at mount (`:hover` matches) — and not for touch pointers;
    `pointerleave` clears it. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:135-161`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:163-177`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:194-219`
- Overflow state (content larger than viewport), recomputed on measurements:
  - `data-has-overflow-x` / `data-has-overflow-y` on Root, Viewport, and Content; each Scrollbar
    gets the attribute for its own axis only.
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:605-631`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:953-961`
  - With no overflow, none of these attributes are set anywhere (even on `keepMounted`
    scrollbars). `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:877-896`
- Edge state (`data-overflow-x-start/end`, `data-overflow-y-start/end`):
  - At the initial scroll position (at start), only `*-end` is present on Root, Viewport,
    Content, and the matching-axis Scrollbars. `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:605-632`
  - Scrolled to the middle: all four present. Scrolled fully to the end: only `*-start`.
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:643-660`, `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:671-688`
  - Near-edge offsets are treated as fully scrolled: `maxScroll - 0.5` already removes the
    `*-end` attribute. `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:691-726`
  - RTL inverts the horizontal axis: `scrollLeft` 0 is the start (`data-overflow-x-end` present,
    no `data-overflow-x-start`), scrolling toward the negative range eventually removes
    `data-overflow-x-end`; mid-way has both.
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:899-947`
  - Changing text direction at runtime recomputes the horizontal edges.
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:483-532`
- `overflowEdgeThreshold` gates the start/end attributes: with `{ xStart: 20, yStart: 5 }`,
  `scrollLeft: 15` does not set `data-overflow-x-start` while `scrollTop: 7` sets
  `data-overflow-y-start`; raising the threshold above the current offset clears the edge
  attribute without a new scroll event.
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:751-762`, `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:820-848`
- Scroll metrics as CSS variables on the Viewport: `--scroll-area-overflow-x-start` reflects the
  live scroll offset in px (e.g. `35px`); `--scroll-area-overflow-x-end` is a non-zero px value
  when overflow remains at the end; when content stops overflowing all four
  (`-overflow-x-start/end`, `-overflow-y-start/end`) are reset to `0px`.
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:764-770`, `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:303-317`
- Thumb geometry state (CSS variables on the thumb):
  - `--scroll-area-thumb-height` (vertical) / `--scroll-area-thumb-width` (horizontal) =
    `viewportSize / scrollableSize * viewportSize` (e.g. 200/1000*200 = 40px).
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:339-346`
  - Scrollbar padding reduces the thumb size (paddingBlock 8 → (200-16)*ratio).
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:402-409`
  - Thumb margin reduces the thumb size (marginBlock 8 → (200-16)*ratio).
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:471-478`
  - Scrollbar margin does NOT change the thumb size.
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:441-448`
  - Thumb size is recomputed when the area becomes visible without any scroll (hidden → shown).
    `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:103-142`
- Corner geometry: `--scroll-area-corner-width` / `--scroll-area-corner-height` equal the
  horizontal scrollbar's height and the vertical scrollbar's width respectively (11px/13px; 10px/10px).
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:248-252`, `packages/react/src/scroll-area/corner/ScrollAreaCorner.test.tsx:95-99`
- Scrollbar visibility is computed at mount before the first ResizeObserver measurement
  (scrollbar and thumb are `visible` after mount compute, and stay visible after the mocked
  ResizeObserver fires). `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:144-207`

## Keyboard interactions

N/A — no scroll-area test asserts any keyboard interaction (no `fireEvent.key`, `user.keyboard`,
or key constants appear in any of the six test files). The only keyboard-adjacent behavior is
focus-related tabbability of the viewport (see Focus management).
`packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:566-577`

## Focus management

- Viewport tabbability tracks overflow: an empty (non-overflowing) viewport has
  `tabindex="-1"`; once overflowing content mounts, it becomes `tabindex="0"`.
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:566-577`
- Track and thumb presses are default-prevented (`mousedown` with `cancelable: true` dispatches
  with `defaultPrevented === true`) so focus stays on the active element — for primary, middle,
  and secondary buttons on the track, and for the thumb.
  `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:493-516`
- Thumb drags latch via pointer capture: `setPointerCapture` is called once on drag start and
  `releasePointerCapture` on release.
  `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:187-210`
- Pointer cancel does not release a stale capture: after `pointercancel`,
  `releasePointerCapture` must not be called (a test fails it if it is).
  `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:288-303`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:426-431`
- Where focus goes after unmounting parts, or whether the viewport is focusable by other means,
  is not asserted. UNVERIFIED — inferred, no test asserts focus movement on unmount.

## Accessibility (roles, aria-*, id linking)

- Scrollbar is hidden from the accessibility tree by default: `aria-hidden="true"`; passing
  `aria-hidden={undefined}` removes the attribute.
  `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:19-37`
- Corner is hidden from the accessibility tree by default: `aria-hidden="true"`; overridable the
  same way. `packages/react/src/scroll-area/corner/ScrollAreaCorner.test.tsx:50-78`
- `data-orientation="horizontal"|"vertical"` is set on the scrollbar.
  `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:39-47`
- No `role` attribute is asserted on any part, and no `aria-*` id-linking (e.g.
  `aria-controls`/`aria-labelledby`) is asserted anywhere. UNVERIFIED — inferred, no test covers
  roles or id linking.

## DOM structure & portal behavior

- All six parts render `div` elements (see Public API surface).
- The Viewport renders a content wrapper as its first element child — tests select
  `viewport.firstElementChild` as "contentWrapper" and assert its computed padding.
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:366-372`
- That content wrapper gets no scrollbar-compensation padding for overlay scrollbars:
  `paddingLeft/Right/Bottom` are all `0px` even with both scrollbars rendered.
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:349-372`
- Corner presence tracks overflow: with `ResizeObserver` notified, no overflow → corner absent
  (`queryByTestId` null); content grows past the viewport → corner appears sized to the
  scrollbars; content shrinks back → corner is removed, overflow attributes cleared, metrics
  zeroed. `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:234-253`, `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:256-319`
- Scrollbars render their `Thumb` as a child; a scrollbar without a thumb never starts a track
  gesture. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:260-277`
- No portal behavior: nothing is rendered outside the Root's DOM tree and no test uses or asserts
  a portal. N/A for portal behavior.
- Custom element renderers that do not forward refs work for `Scrollbar` and `Content` (element
  form of the `render` prop). `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:49-69`, `packages/react/src/scroll-area/content/ScrollAreaContent.test.tsx:59-76`

## Events (names, payload shape, bubbling, preventDefault semantics)

- Viewport `onScroll`: user callback invoked on viewport scroll events; unmounting the viewport
  from within the callback is safe (no throw, viewport removed).
  `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:20-42`
- Thumb user pointer handlers: `onPointerMove`/`onPointerUp` receive the native event, and
  unmounting the Scrollbar from `onPointerMove` or the Viewport from `onPointerUp` (via
  `ReactDOM.flushSync`) during an active gesture does not throw; state stays consistent.
  `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:75-112`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:114-155`
- Wheel over the Scrollbar (no `onWheel` prop is asserted — behavior is built-in interception):
  - The viewport is scrolled by the wheel delta along the scrollbar's axis; LTR horizontal uses
    the positive `scrollLeft` range, RTL the negative range; scrolling is clamped at both edges.
    `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:780-836`
  - `preventDefault` semantics: the event is cancelled only when the wheel scroll is actually
    consumed (mid-range `fireEvent` returns `false`); at either edge with further scroll in that
    direction the event is NOT cancelled (returns `true`) so it chains to the parent/page.
    `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:838-853`
  - Zero-delta events are ignored (no scroll, no `data-scrolling`), and `ctrlKey` wheel (browser
    zoom) is not intercepted. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:855-875`
  - A consumed wheel marks the area `data-scrolling`; an edge-chained (unconsumed) wheel does
    not. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:877-902`
  - Wheel registration is live: a horizontal scrollbar that becomes visible after initial render
    (RTL overflow) intercepts wheel events. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:904-926`
- Track (`Scrollbar` element) `pointerdown`:
  - Non-primary buttons are ignored (viewport untouched, consumer `scrollSnapType` style
    preserved). `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:224-243`
  - With a thumb mounted and a primary press on the track, a jump-to-click positions the
    viewport and marks `data-scrolling`. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:345-405`
  - Clicking below the thumb on a vertical track scrolls down; clicking the end of a horizontal
    LTR track scrolls right; an RTL horizontal track scrolls into the negative range.
    `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:550-590`
  - `pointercancel` clears the track drag state: subsequent `pointermove` on the thumb no longer
    scrolls. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:407-475`
  - Thumb hits (native path contains the thumb) are excluded from track-jump handling: a
    pointerdown whose `composedPath` differs from the synthetic target is ignored.
    `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:279-343`
  - With `scroll-snap`, the jump-to-click position is not snapped (lands within 1px of the aimed
    offset); on `pointerup` snapping is restored and the position re-snaps to the nearest snap
    point. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:597-646`
- Thumb drag (`Thumb` element):
  - `pointerdown` with the primary button disables the viewport's scroll snapping
    (`style.scrollSnapType === 'none'`); `pointerup` and `pointercancel` restore the original
    value. `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:490-516`
  - Dragging moves the viewport proportionally: LTR horizontal → positive `scrollLeft`, RTL
    horizontal → negative `scrollLeft`; vertical drags scroll positively.
    `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:185-211`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:213-233`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:343-356`
  - During a drag the scrollbar/thumb carry `data-scrolling`; `pointercancel` clears it.
    `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:206`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:296-303`
  - Multi-pointer: a second `pointerdown` while a drag is active is ignored, and releasing the
    non-dragging pointer does not end the drag; only the drag-active pointer's release ends it.
    `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:518-534`
  - Lost capture: if pointer capture was silently dropped (no `pointerup`/`pointercancel`), a new
    pointer can take over the latched drag. `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:536-554`
  - Missed release: if the release never arrived, the first buttonless (`buttons: 0`) move from
    the drag-active pointer ends the drag immediately — scroll position frozen, snap restored,
    `data-scrolling` cleared at once (not deferred to the scroll timeout); other pointers merely
    hovering (`buttons: 0`, different `pointerId`) never end the active drag or scroll.
    `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:306-372`
- `mousedown` on track or thumb is always default-prevented (all buttons) — see Focus management.
  `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:493-516`
- Event bubbling beyond the part under test is not asserted. UNVERIFIED — no test covers bubbling
  beyond gesture handling.

## Edge cases (rapid interactions, unmount, nesting)

- Missing context throws with actionable messages:
  - Thumb outside `Scrollbar`: "Base UI: ScrollAreaScrollbarContext is missing.
    ScrollAreaScrollbar parts must be placed within <ScrollArea.Scrollbar>."
    `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:25-41`
  - Viewport outside `Root`: "Base UI: ScrollAreaRootContext is missing. ScrollArea parts must be
    placed within <ScrollArea.Root>." `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:488-498`
  - Content outside `Viewport`: "Base UI: ScrollAreaViewportContext missing. ScrollAreaViewport
    parts must be placed within <ScrollArea.Viewport>."
    `packages/react/src/scroll-area/content/ScrollAreaContent.test.tsx:22-38`
- Degenerate mounts: a track press with no Viewport mounted is a safe no-op (no
  `data-scrolling`); a thumb gesture with no viewport neither scrolls nor transforms nor latches
  `data-scrolling`. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:245-258`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:43-73`
- Degenerate geometry: a thumb that fills or overflows a short track (driving max thumb offset to
  zero or negative) must not teleport the scroll — dragging it or clicking such a track leaves
  `scrollTop` unchanged. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:654-718`
- Unmount during async work: a viewport animation finishing after its viewport unmounts is
  ignored (recompute is not attempted). `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:87-131`
- Overflow appearing after initial measurement is picked up in all tested paths: content mounted
  later, observed `ScrollArea.Content` resize, a subtree animation finishing,
  visibility-restored computation, and corner recomputation.
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:534-578`, `packages/react/src/scroll-area/content/ScrollAreaContent.test.tsx:40-57`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:44-85`, `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:103-142`, `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:209-254`
- Overscroll feedback (Safari rubber-band; `scrollTop`/`scrollLeft` read out of range): the thumb
  shrinks by a damped amount (smaller than resting but above 90% of it, not a 1:1 px
  subtraction), pins to the corresponding track edge (start pinned at top/left; end pinned at
  bottom/right; RTL horizontal mirrored), and restores its resting size when scrolling settles
  back into range. `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:330-486`
- Rapid/interleaved scrolling: successive scroll events in opposite axes maintain independent
  `data-scrolling` timers per axis across parts. `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:105-131`
- Scroll-triggered re-renders: scrolling does not re-render any scroll-area part while the corner
  size is unchanged (context value stability — a `ContextProbe` consumer's commit count is
  unchanged after three scroll events). `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:950-999`
- Hover modality edge: a viewport already hovered on mount is detected (scrollbar gets
  `data-hovering`); synthetic `pointerover` events whose `composedPath` disagrees with the event
  target are still honored when the path contains the viewport.
  `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:135-161`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:179-220`
- Nested scroll areas are not tested. UNVERIFIED — no test covers nesting.

## Shared harness dependencies

- `#test-utils` (module alias `./test/index.ts`, defined at `packages/react/package.json:110`)
  provides `describeConformance`, `createRenderer`, and `isJSDOM`, used by all six test files.
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:5`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:6`, `packages/react/src/scroll-area/content/ScrollAreaContent.test.tsx:4`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:6`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:4`, `packages/react/src/scroll-area/corner/ScrollAreaCorner.test.tsx:4`
- `describeConformance` (`packages/react/test/describeConformance.tsx`) runs the shared
  conformance suites for every part; each scroll-area test passes only `refInstanceof:
  window.HTMLDivElement` plus a `render` wrapper placing the part in its required context
  (Scrollbar/Thumb/Content/Corner wrapped in Root or Root+Viewport; Scrollbar conformance uses
  `keepMounted`). `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:57-60`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:12-17`
- `createRenderer` exposes `clock.withFakeTimers()`; the `data-scrolling` suites in all four
  scroll-timing test files use it to advance `SCROLL_TIMEOUT` deterministically.
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:63-65`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:72-74`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:575-577`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:135-137`
- `isJSDOM` gates Chromium-only tests (real layout required): sizing, overflow attributes,
  track-click-by-axis, scroll-snap-on-track-press, non-positive thumb offset, thumb dragging,
  overscroll feedback, and wheel-after-visible. `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:102`, `packages/react/src/scroll-area/scrollbar/ScrollAreaScrollbar.test.tsx:524`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:330`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:157`
- `fireEvent`, `screen`, `waitFor`, `act`, `flushMicrotasks`, and the `user` pointer instance come
  from `@mui/internal-test-utils`. `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:4`, `packages/react/src/scroll-area/viewport/ScrollAreaViewport.test.tsx:7`, `packages/react/src/scroll-area/thumb/ScrollAreaThumb.test.tsx:6`
- `SCROLL_TIMEOUT` is imported from the component source (`../constants`) — the tests assert
  timing relative to it rather than hardcoding a value.
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:8`
- `ScrollAreaRootContext` is imported from the component source only to build a context-stability
  probe (counting commits of a raw context consumer). `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:9`, `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:950-999`
- `withMockResizeObserver` is a locally-defined helper in the Root test (not shared harness): it
  swaps `window.ResizeObserver` for a mock whose `notifyResizeObserver()` fires all registered
  observers, used to drive measurement-dependent assertions.
  `packages/react/src/scroll-area/root/ScrollAreaRoot.test.tsx:16-52`
