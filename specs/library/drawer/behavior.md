# Drawer — Behavior Spec (whole-unit index)

Mining method: `library: drawer` is flagged `needs-batched-mining: true` (`TODO.md:387-387`), so this unit was mined in six
parallel batches, one per clustered subdirectory of `packages/react/src/drawer/` (the four smallest subdirectories were
grouped together). Each batch produced a part file under `specs/library/drawer/parts/` using the full 9-section behavior
template (Public API surface; State model; Keyboard interactions; Focus management; Accessibility; DOM structure & portal
behavior; Events; Edge cases; Shared harness dependencies). This file is the index: one paragraph per part pointing into
the part file for depth, followed by behavior that is genuinely cross-cutting and only makes sense at the whole-unit
level. The unit has no `wraps-external:` entry in `TODO.md`, so no third-party delegation applies — all behavior below is
derived from the drawer test suites themselves.

## Part index

- **`root`** — The open/snap-point state machine plus the detached-handle API. Closed by default; controlled
  `open`/`onOpenChange` or uncontrolled `defaultOpen`; details objects carry `reason` (`swipe`, `closeWatcher`,
  `closePress`, `none`) and a selective `cancel()` that keeps the drawer open and rolls back all transient swipe
  styles. Snap points (`string 'px'/'rem' | number | null`) resolve to `{value, height, offset}` with
  `offset = popupHeight − height`, invalid strings filtered, numbers clamped to `[0, popupHeight]`, duplicates keeping
  the last; controlled `snapPoint`/`onSnapPointChange(next, { reason, cancel })` with `defaultSnapPoint` reset on close.
  `Drawer.createHandle<T>()` (`open(triggerId)`, `close()`, `openWithPayload(payload)`, `isOpen`) drives the same state
  as triggers, with a render-prop child receiving `{ payload }`; unmounted-handle calls warn and no-op. Geometry/velocity
  tests (dismiss threshold 45–60% of size, release-velocity reversal, sqrt-damped overshoot, `snapToSequentialPoints`
  flick/skip/nearest semantics, `swipeDirection` right/down/up) are Chromium-only; CloseWatcher integration produces
  exactly one close per burst of `close` events; missing context throws a `Base UI:` guard error.
  `specs/library/drawer/parts/root.md:5-15`, `specs/library/drawer/parts/root.md:19-34`, `specs/library/drawer/parts/root.md:64-76`,
  `specs/library/drawer/parts/root.md:80-97`

- **`popup`** — The sheet surface: renders an `HTMLDivElement` (full conformance), must sit inside `Drawer.Viewport`
  (dev-only `console.error` with an exact message otherwise), takes `initialFocus` (default focuses the popup element
  itself, `false` keeps the trigger focused), and stops composite `ArrowDown` keydowns from escaping while letting
  character keys through. It measures its own height via ResizeObserver into root context (`popupHeight`, retained
  under nested stretch), renders `--drawer-snap-point-offset`, `data-swipe-direction`, and — as the parent of a nested
  drawer — `--nested-drawers`, `data-nested-drawer-open`, and `--drawer-frontmost-height` (border-inclusive) with the
  Chromium-only `--drawer-height` lock surviving close/reopen; Dialog/AlertDialog popups are never counted as nested
  drawers. `specs/library/drawer/parts/popup.md:17-29`, `specs/library/drawer/parts/popup.md:31-47`,
  `specs/library/drawer/parts/popup.md:49-62`, `specs/library/drawer/parts/popup.md:74-87`

- **`viewport`** — The gesture engine. A plain `HTMLDivElement` (full conformance) inside `Drawer.Portal` that
  arbitrates touch scrolling and drives swipe-to-dismiss/snap from all four orientations: axis-attribution slop
  (claimed between 3–12px, never re-arbitrated), scroll-edge blocking (in-axis touchmove prevented and stopped from
  reaching React handlers at the dismiss edge, allowed for native scrolling until the edge), exemptions
  (`data-base-ui-swipe-ignore`, native range/Slider inputs, active text selections, pinch-zoom, portaled
  Combobox/ descendants), non-touch pointer drag support (mouse/pen, with popup-anchored selection clearing and the
  Content-boundary rule), velocity-sampled release resolution (nearest snap point vs dismissal vs
  `snapToSequentialPoints`), unattributed-gesture fallback, and shadow-root-aware `elementFromPoint` hit testing.
  It publishes every transient visual (backdrop `data-swiping`, popup movement vars, nested progress, provider
  `--drawer-swipe-progress`/`--drawer-height`) and clears them at unmount/reopen. `specs/library/drawer/parts/viewport.md:17-43`,
  `specs/library/drawer/parts/viewport.md:57-66`, `specs/library/drawer/parts/viewport.md:92-114`, `specs/library/drawer/parts/viewport.md:126-142`

- **`swipe-area`** — `Drawer.SwipeArea`: an in-place `div` (full conformance, no portal) that opens the drawer by
  swiping from its own footprint, even with no popup rendered. Direction-locked gestures open mid-drag; release
  commits by distance threshold (35–60% of height, Chromium-only) with velocity consulted only within an 80ms
  window; `disabled` (with `data-disabled` and layout-phase progress reset), part-level `swipeDirection` override,
  sqrt-damped overshoot, `pointercancel`/`contextmenu`/buttons-change interruption, exactly-once quick-flick
  handling, and a release-click guard that suppresses the gesture's own trailing click from dismissing the
  just-opened drawer while allowing fresh presses, keyboard Enter, and virtual clicks. It registers by generated `id`
  in the dialog store's `triggerElements` registry. `specs/library/drawer/parts/swipe-area.md:7-15`,
  `specs/library/drawer/parts/swipe-area.md:17-27`, `specs/library/drawer/parts/swipe-area.md:46-58`,
  `specs/library/drawer/parts/swipe-area.md:60-72`, `specs/library/drawer/parts/swipe-area.md:74-86`

- **`virtual-keyboard-provider`** — Opt-in (`Drawer.VirtualKeyboardProvider`, zero props) mobile-keyboard
  coordination, entirely Chromium-only in the tests. Computes `--drawer-keyboard-inset` on the viewport
  (`innerHeight − min(innerHeight, offsetTop + height)`, <60px reductions ignored as browser chrome, disabled while
  pinch-zoomed), intercepts taps on keyboard-eligible fields (≤10px move, touch lifecycle, hit-slop probing,
  `focus({ preventScroll: true })` + `touchend` preventDefault + synthetic click re-dispatch, blur/refocus to
  re-summon a closed keyboard), adds scroll slack (`paddingBottom`, `scrollPaddingBottom`, `overflowAnchor: 'none'`)
  to the field's scroll ancestor, schedules bounded alignment scrolling that centers the field in the visible band
  (reduced-motion uses `behavior: 'auto'`), reconciles consumer focus redirects, and restores modal page scroll
  (`window.scrollTo({left:0, top:0, behavior:'instant'})` on window scroll while the keyboard is open, modal only).
  Graceful degradation when `visualViewport`/`PointerEvent` are missing; inert while closed-but-mounted or when a
  nested drawer is open. `specs/library/drawer/parts/virtual-keyboard-provider.md:5-12`,
  `specs/library/drawer/parts/virtual-keyboard-provider.md:14-30`, `specs/library/drawer/parts/virtual-keyboard-provider.md:32-44`,
  `specs/library/drawer/parts/virtual-keyboard-provider.md:64-88`, `specs/library/drawer/parts/virtual-keyboard-provider.md:89-103`

- **`content-indent-provider`** — `Drawer.Content` (plain `div`, full conformance inside
  `Root > Portal > Viewport > Popup`, deliberately exposing no public swipe-ignore markers), `Drawer.Provider` (a
  registry of open drawers spanning sibling roots — active while ≥1 is open, removal on close or unmount, idempotent
  registration, zero re-renders for closed registration — plus the `visualStateStore` partial-merge publisher with
  NaN/Infinity sanitization), `Drawer.Indent` (maps `swipeProgress` → `--drawer-swipe-progress` unitless,
  `frontmostHeight` → `--drawer-height` px, cleared at 0, inline styles reset on unmount, placement-flexible), and
  `Drawer.IndentBackground` (`data-active`/`data-inactive` mirroring the registry). No keyboard, focus, ARIA, or
  event surface is asserted for any of these four parts. `specs/library/drawer/parts/content-indent-provider.md:7-29`,
  `specs/library/drawer/parts/content-indent-provider.md:33-50`, `specs/library/drawer/parts/content-indent-provider.md:64-83`,
  `specs/library/drawer/parts/content-indent-provider.md:89-101`

## Cross-cutting behavior

- **One open/snap state machine at the Root.** `Drawer.Root` owns `open` and the active snap point; Viewport and
  SwipeArea only *request* transitions through `onOpenChange`/`onSnapPointChange`, whose details carry a `reason`
  and `cancel()` — a cancel keeps state intact and rolls back transient DOM (movement vars, `data-swipe-dismiss`,
  snap point). Controlled parents learn of dismiss/open intent even when they ignore it.
  `specs/library/drawer/parts/root.md:19-24`, `specs/library/drawer/parts/viewport.md:27-28`,
  `specs/library/drawer/parts/swipe-area.md:19-27`

- **Uniform DOM shell.** `Drawer.Root > (Trigger) + Portal > (Backdrop) + Viewport > Popup (> Content, Close)`; the
  popup is `role="dialog"` with an `id`, the trigger a `button` with `aria-expanded`/`aria-controls` linked to that
  id while open (no `aria-controls` after remount closed). Popup without a Viewport warns in dev (disabling swipe
  handling and touch scroll locking); without Root context, parts throw.
  `specs/library/drawer/parts/root.md:47-62`, `specs/library/drawer/parts/popup.md:44-54`, `specs/library/drawer/parts/root.md:96-96`

- **A published visual-state graph, not just gesture math.** The gesture surfaces write state consumed elsewhere:
  popup `--drawer-swipe-movement-x/y` + `--drawer-snap-point-offset` + `data-swipe-dismiss`/`data-ending-style`,
  backdrop `data-swiping` + `--drawer-swipe-progress` + `--drawer-height`, parent popups' nested progress
  (`data-nested-drawer-swiping`), and Provider/Indent `--drawer-swipe-progress`/`--drawer-height` via
  `visualStateStore`. All of these are cleared/reverted deterministically (release, cancel, reopen, teardown,
  snap change). `specs/library/drawer/parts/viewport.md:33-43`, `specs/library/drawer/parts/swipe-area.md:50-58`,
  `specs/library/drawer/parts/content-indent-provider.md:71-77`, `specs/library/drawer/parts/root.md:60-62`

- **Layout-phase timing contract.** Registration with the Provider, nested-drawer/frontmost notifications, reopen
  resets, teardown cleanup, and mid-gesture resets are all applied synchronously before passive effects flush —
  including under `React.StrictMode` and the React 17 `useId → undefined` fallback. A port must reproduce this
  before-passive-effect ordering, not just the end states.
  `specs/library/drawer/parts/root.md:97-97`, `specs/library/drawer/parts/popup.md:26-26`,
  `specs/library/drawer/parts/viewport.md:43-43`, `specs/library/drawer/parts/viewport.md:131-137`,
  `specs/library/drawer/parts/swipe-area.md:26-26`, `specs/library/drawer/parts/content-indent-provider.md:97-97`

- **Nested-drawer coordination.** A nested root propagates swipe progress to the parent's
  `nestedSwipeProgressStore` (deduped, NaN-normalized); the parent popup tracks presence/count/height
  (`data-nested-drawer-open`, `--nested-drawers`, `--drawer-frontmost-height`) and clears it at close-start of the
  child; Dialog/AlertDialog subtrees are excluded from the count; the keyboard provider suppresses parent tap-to-focus
  while a nested drawer is open. `specs/library/drawer/parts/root.md:25-25`, `specs/library/drawer/parts/root.md:95-95`,
  `specs/library/drawer/parts/popup.md:22-26`, `specs/library/drawer/parts/popup.md:80-83`,
  `specs/library/drawer/parts/viewport.md:132-135`, `specs/library/drawer/parts/virtual-keyboard-provider.md:95-95`

- **Geometry probing is intrinsic.** Snap resolution, dismiss thresholds, overshoot damping, hit testing, and
  keyboard alignment all depend on measured layout (`offsetHeight`, `elementFromPoint` incl. shadow roots,
  `getBoundingClientRect`, ResizeObserver/visualViewport streams), which is why the heavy suites are gated
  `it.skipIf(isJSDOM)` (Chromium) and the Root suite additionally runs under an `os.android = true` platform mock.
  `specs/library/drawer/parts/root.md:103-107`, `specs/library/drawer/parts/viewport.md:31-31`,
  `specs/library/drawer/parts/viewport.md:59-62`, `specs/library/drawer/parts/virtual-keyboard-provider.md:3-3`,
  `specs/library/drawer/parts/popup.md:84-84`

- **Platform/animation gating of proven behavior.** Real-animation coverage (`data-ending-style` timing, exit
  animations, re-grab during close) toggles `BASE_UI_ANIMATIONS_DISABLED` and is Chromium-only; jsdom suites cover
  pure logic (snap math, registry semantics, handle API, aria wiring). Behavior "proven" in the parts files is scoped
  to those environments. `specs/library/drawer/parts/popup.md:94-102`, `specs/library/drawer/parts/swipe-area.md:94-94`,
  `specs/library/drawer/parts/root.md:84-89`

- **Shared harness across all 11 test files.** Every part file's tests import `#test-utils`
  (`packages/react/test/index.ts`), which re-exports `createRenderer` (act-wrapped render + `setProps`/`rerender`),
  `describeConformance` (propsSpread/refForwarding/renderProp/className), `firePointer` (mandatory positive
  `timeStamp` so velocity logic is deterministic), `isJSDOM`, and `wait` helpers, plus `@mui/internal-test-utils`
  (`act`, `fireEvent`, `screen`, `waitFor`, `flushMicrotasks`). Gesture suites add file-local swipe scripts,
  `elementFromPoint`/`ResizeObserver`/`visualViewport` mocks, and `offsetHeight` stubs.
  `specs/library/drawer/parts/root.md:99-108`, `specs/library/drawer/parts/viewport.md:144-152`,
  `specs/library/drawer/parts/swipe-area.md:88-105`, `specs/library/drawer/parts/virtual-keyboard-provider.md:105-118`,
  `specs/library/drawer/parts/popup.md:89-103`, `specs/library/drawer/parts/content-indent-provider.md:103-114`
