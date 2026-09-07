# Collapsible implementation spec

WHY/HOW companion to the behavior spec (`specs/library/collapsible/behavior.md` — referenced
below by section name). Mined from the non-test source files under
`packages/react/src/collapsible/`. The unit's `TODO.md` entry (`TODO.md:346-352`) has no
`wraps-external:` field, so nothing here is delegated to an external package; every mechanism
below is derived from this unit's own source.

Architecture summary: Collapsible is a three-part unit (Root / Trigger / Panel) with a
deliberately thin Root and a heavy Panel. The Root owns the single source of truth — the
controlled/uncontrolled `open` boolean — and the shared `transitionStatus` machine
(`useTransitionStatus`). The Panel owns everything animation-related: size measurement, motion
detection, motion suppression, and its own mount/unmount lifecycle, writing mount state back to
the Root through context setters. The Trigger is a stateless adapter that wires the shared open
state into button semantics.

## State machine / hooks used

### Root: `open` truth and the transition status machine

- `useControlled` (`packages/utils/src/useControlled.ts:28-92`) at
  `packages/react/src/collapsible/root/useCollapsibleRoot.ts:16-21` implements the controlled /
  uncontrolled split. It returns the prop value when `controlled !== undefined` (mode is frozen
  on first render via a ref, `packages/utils/src/useControlled.ts:41-45`) and, critically, the
  setter it returns is `setValueIfUncontrolled`
  (`packages/utils/src/useControlled.ts:82-91`) — a write is a no-op in controlled mode. This
  single fact produces the behavior spec's "State model" contract: in controlled mode a trigger
  press calls `onOpenChange` and the UI only moves when the prop is updated externally; there is
  no separate "ignore click in controlled mode" branch anywhere in the component.
- `useTransitionStatus(open, true, true)` (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:23`)
  — the second boolean enables the `'idle'` phase, the third defers `'ending'` by a frame. The
  machine lives in `packages/react/src/internals/useTransitionStatus.ts`: `open && !mounted`
  transitions to `mounted=true` + `'starting'` synchronously during render
  (`packages/react/src/internals/useTransitionStatus.ts:31-34`), an rAF then settles `'starting'`
  → `'idle'` (`packages/react/src/internals/useTransitionStatus.ts:74-90`), and close defers
  `'ending'` by one animation frame (`packages/react/src/internals/useTransitionStatus.ts:44-56`).
  The deferred `ending` is what gives the panel's layout effect (below) a window to measure and
  commit the pixel height *before* `[data-ending-style]` exists — the "Measured height is
  restored before closing transition/keyframe styles apply" edge case in the behavior spec.
  `mounted` is deliberately not "in the DOM"; it means "mounted for transition purposes" and can
  stay `true` while the element is hidden (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:103-107`).
- `useBaseUiId()` (`packages/react/src/internals/useBaseUiId.ts:9-11`, a `useId` wrapper prefixing
  `base-ui-`) at `packages/react/src/collapsible/root/useCollapsibleRoot.ts:25` generates the
  fallback panel id.
- Panel-id registry: a tri-state `React.useState<string | null | undefined>` at
  `packages/react/src/collapsible/root/useCollapsibleRoot.ts:27` — `undefined` = use the
  generated fallback, `string` = a panel registered its own id, `null` = the panel unmounted.
  The derived `panelId` (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:28`) maps
  `null` → `undefined`, which is how the trigger's `aria-controls` disappears when the panel is
  gone (behavior spec, "Accessibility").
- `handleTrigger` (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:30-41`) is the only
  mutation entry point for user-driven toggling: compute `nextOpen = !open`, build
  `createChangeEventDetails(REASONS.triggerPress, event.nativeEvent)`, call `onOpenChange`
  *first*, and only `setOpen(nextOpen)` if `eventDetails.isCanceled` is still false
  (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:36-40`). This is the mechanism
  behind the behavior spec's "Cancellation" bullets: `cancel()` inside `onOpenChange` blocks the
  uncontrolled write; in controlled mode it is irrelevant because the write is a no-op anyway.
  The details object itself (mutual `cancel`/`isCanceled` getter pair) is created by
  `createChangeEventDetails` (`packages/react/src/internals/createBaseUIEventDetails.ts:118-149`).
- The root callback identity is stabilized before it enters the context:
  `useStableCallback(onOpenChangeProp)` (`packages/react/src/collapsible/root/CollapsibleRoot.tsx:33`),
  so consumers holding effects on `onOpenChange` don't re-subscribe per render.
- The whole hook return value is memoized (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:43-68`)
  and becomes the context payload.

### Panel: measurement + motion subsystem (`useCollapsiblePanel.ts`)

The panel hook is where all DOM measurement lives. Its one-shot refs:

- `shouldSkipNextOpenRef` (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:54`) —
  set by the `beforematch` handler so the next open skips author motion once (behavior spec,
  "Events": beforematch opens are instant).
- `shouldPreventMountAnimationRef` initialized from `open`
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:58`) — suppresses the *first*
  open lifecycle's keyframe animation for initially-open panels (SSR/initial-mount layout shift;
  behavior spec, "DOM structure": `animationName: 'none'` on SSR'd open panels). Cleared on the
  first close request (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:227`), so
  later opens animate (behavior spec, "Initial mount" edge case).
- `shouldPreventActivityResumeAnimationRef`
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:61`) — set from the effect
  cleanup path via `markActivityResumeAnimationSuppressed`
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:124-128`) when React.Activity
  tears down effects while an open css-animation panel is visible, so the re-reveal doesn't
  replay the open keyframes (behavior spec, "React.Activity" edge case).
- `forcePanelIdle` state (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:65`) —
  panel-local override for opens that intentionally skip motion: the shared root machine still
  advances `'starting' → 'idle'` asynchronously, so the panel forces its effective status to
  `'idle'` immediately and drops the override once the root catches up
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:130-139`, drops when
  `transitionStatus !== 'starting'`).
- `pendingTemporaryStyleRestoreRef` with chained set/restore helpers
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:108-119`) — tracks a temporary
  inline style (a `0s` duration forced for instant opens) that must be restored before the next
  animation-type detection, otherwise the first close after a beforematch open would be misread
  as motionless. Restored (a) whenever the panel goes closed
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:158-160`) and (b) on unmount
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:141-146`).

The central `useIsoLayoutEffect` (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:148-277`)
is a fork per lifecycle phase; it re-runs on every `open`/`mounted`/`transitionStatus` change:

1. First it detects the motion type via `getAnimationType(panel, shouldPreventOpenAnimation)`
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:162-163`) — computed styles from
   `ownerWindow(element).getComputedStyle(element)`
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:414`), classifying
   `css-transition` / `css-animation` / `none` by which longhands have non-zero durations
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:410-452`). Both present → dev
   warning, transitions win (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:424-434`;
   the warning text appears in the behavior spec's "Edge cases").
2. Open + `'starting'`: measure `scrollHeight`/`scrollWidth`
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:403-408`) into state, then per
   type: `none` → clear dims and `setForcePanelIdle(true)`
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:186-190`);
   `css-transition` → first `resetLayoutStyles` — a one-frame `initial !important` override of
   `justify-*`/`align-*` inline styles that would distort scroll-based measurement
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:485-514`, restored on the next
   animation frame or effect cleanup; the behavior spec's "Before measuring an opening panel"
   edge case) — and, if the open should skip motion, a temporary
   `transition-duration: 0s` + `setForcePanelIdle(true)`
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:196-203`);
   `css-animation` → for a normal open, temporarily set `animation-name: 'none'` and restore
   immediately so the keyframe never runs for this transition
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:206-211`); for a skip-motion
   open, force `animation-duration: 0s` instead
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:212-218`).
3. Close requested (`!open && mounted`, status still `idle`/`starting`): clear both suppression
   refs; if motion is `none`, unmount immediately (`setMounted(false)`) and clear dims
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:226-234`); otherwise capture the
   expanded size *now*, before the deferred ending phase applies closed styles, so the close
   transition starts from pixels — this includes interrupted opens
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:236-238`).
4. `'ending'` (only reachable when the effect skipped branch 3): if motion is `none` → unmount
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:247-250`); if the measured size
   is zero → unmount without waiting for animations (the behavior spec's zero-size panel edge
   case, `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:255-258`); otherwise
   re-measure and, for css-animation closes, set `animation-name: 'none'` and restore it
   synchronously (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:260-265`).
5. Initially-open css-animation panels (`open && idle && suppression active`) skip the motion
   branches entirely and only cache the expanded size for the *first close*
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:168-176`).

Completion plumbing is split in two directions:

- Open completion: `useOpenChangeComplete` (`packages/react/src/internals/useOpenChangeComplete.tsx:9-28`,
  itself a thin wrapper over `useAnimationsFinished`) at
  `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:279-295` clears the rendered
  dimensions back to `auto` once the open animation finishes. The `if (!open) return` guard
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:289-291`) closes the documented
  post-paint race where `animation.finished` resolves after the render that set `open=false` but
  before the effect cleanup — clearing then would start the close transition from `height: 0`.
  The underlying watcher is `useAnimationsFinished`
  (`packages/react/src/internals/useAnimationsFinished.ts:44-158`): awaits
  `getAnimations()` `finished` promises with an aborted-replacement re-check loop
  (`packages/react/src/internals/useAnimationsFinished.ts:99-127`), and honors
  `globalThis.BASE_UI_ANIMATIONS_DISABLED` by completing immediately
  (`packages/react/src/internals/useAnimationsFinished.ts:91-97`) — the runtime flag the tests
  toggle.
- Close completion: a dedicated passive effect
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:303-344`) that cannot reuse the
  open path. It waits one animation frame *after* `data-ending-style` is on the element before
  watching animations, because Chrome can register the exit transition a frame late when an
  Accordion closes one item while opening another
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:297-302`, mui/base-ui#3099), then
  calls `runOnceCloseAnimationsFinish(handleComplete)` inside that frame
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:328-330`). `handleComplete`
  re-checks the *latest* `open` via `useValueAsRef` before unmounting
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:320-322`) — the reopen-mid-close
  guard — then `setMounted(false)` and clears dimensions
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:324-325`).

Derived render values (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:73-93`):

- `hidden = !open && !mounted` — the `hidden` attribute is the conjunction of "user wants it
  closed" and "transition finished"; this is why a kept-mounted closed panel is `hidden` but a
  closing one is not.
- `panelTransitionStatus = forcePanelIdle ? 'idle' : transitionStatus`.
- `shouldPreventOpenAnimation` reads the two suppression refs during render
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:75-80`; the comment documents why
  render-time ref reads are intentional here — one-shot motion suppression gated on the last
  committed write).
- `renderedDimensions` falls back to `lastMeasuredDimensionsRef` for a closed-but-mounted
  css-animation panel whose live dims were already reset
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:81-91`) — keeps a pixel size
  rendered while hidden so reopening keyframes start from the expanded size.
- `shouldPersistHiddenTransitionStyles` (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:92-93`)
  — for a `hiddenUntilFound` panel closed with a css *transition*, the panel renders an empty
  `data-starting-style` attribute (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:389-391`)
  so the starting-style CSS contract keeps matching while hidden and no leftover transition runs
  (behavior spec, "hiddenUntilFound closes cleanly" edge case).
- `shouldRender = keepMounted || hiddenUntilFound || mounted || open`
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:384`) — `hiddenUntilFound`
  implies keepMounted, which is why combining it with `keepMounted={false}` only warns
  (`packages/react/src/collapsible/panel/CollapsiblePanel.tsx:36-45`, dev-only `warn`).
- `setDimensions` wraps `useState` with a measurement cache
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:98-106`); `shouldCacheMeasurement`
  is `false` only when clearing back to `auto`, so `lastMeasuredDimensionsRef` always holds the
  last expanded size.

### Trigger: delegation to button semantics

- `useButton` (`packages/react/src/internals/use-button/useButton.ts:14-243`) at
  `packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:44-48` with
  `focusableWhenDisabled: true` and `native: nativeButton` (default `true`,
  `packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:39`). This answers the behavior
  spec's UNVERIFIED keyboard note: the collapsible never listens for keydown/keyup itself. For a
  native `<button>` the browser synthesizes the click for Enter/Space, and `useButton`'s extra
  keyboard handling only kicks in for non-native renders and composite items
  (`packages/react/src/internals/use-button/useButton.ts:154-176`); its `onClick` wrapper is also
  what suppresses activation while disabled
  (`packages/react/src/internals/use-button/useButton.ts:104-110`). `handleTrigger` is wired as
  `onClick` (`packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:57`), so click is the
  single activation funnel.
- ARIA wiring lives directly in the trigger's props slice:
  `aria-controls: open ? panelId : undefined`
  (`packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:55` — open-gated, matching the
  behavior spec) and `aria-expanded: open`
  (`packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:56`).
- Trigger-local `disabled` defaults to the root's but is overridable per trigger
  (`packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:37`).

## Context providers/consumers

One context: `CollapsibleRootContext`
(`packages/react/src/collapsible/root/CollapsibleRootContext.ts:11-13`), created `undefined` and
read exclusively through `useCollapsibleRootContext`
(`packages/react/src/collapsible/root/CollapsibleRootContext.ts:15-24`), which throws the
"CollapsibleRootContext is missing" error the behavior spec documents — that throw is the entire
outside-a-Root enforcement; there is no fallback rendering.

The provider wraps the root's rendered element so descendants (including portaled trees, though
this unit never portals) see the same state
(`packages/react/src/collapsible/root/CollapsibleRoot.tsx:67-71`). The payload is the memoized
hook return plus the stabilized `onOpenChange` and a `state` object
(`{open, disabled, transitionStatus}`, `packages/react/src/collapsible/root/CollapsibleRoot.tsx:42-58`).

What crosses the boundary, per consumer:

- Trigger reads `panelId`, `open`, `handleTrigger`, `state`, `disabled`
  (`packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:27-33`) — read-only.
- Panel reads `defaultPanelId`, `mounted`, `onOpenChange`, `open`, `setMounted`,
  `setPanelIdState`, `setOpen`, `state`, `transitionStatus`
  (`packages/react/src/collapsible/panel/CollapsiblePanel.tsx:47-57`) — and is the only writer:
  it drives the mount lifecycle (`setMounted`), performs beforematch opens (`setOpen`), and
  registers/unregisters its id (`setPanelIdState`). The asymmetry is intentional: all
  animation-sensitive state lives below the root, keeping the root cheap for nesting.
- Panel-id registration is how a *child* informs the *parent*:
  `useIsoLayoutEffect` in `packages/react/src/collapsible/panel/CollapsiblePanel.tsx:64-69`
  pushes `registeredId` (or resets `null` → `undefined` to return to the generated fallback) and
  the cleanup sets `null` when the deregistering id is the one currently registered. This is
  StrictMode-safe by construction — the double-invoked effect re-registers — which is what the
  behavior spec's id-association round-trip test exercises.
- The shared `state` object is passed to `useRenderElement` by all three parts, so
  `className`/`style` callbacks and data attributes flip from one state object everywhere
  (behavior spec, "State model": state callbacks fire simultaneously). Trigger and panel widen
  the mapping: trigger uses `triggerOpenStateMapping` (`data-panel-open`, only when open) +
  `transitionStatusMapping`
  (`packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:12-15`), root and panel use
  `collapsibleStateAttributesMapping`
  (`packages/react/src/collapsible/root/stateAttributesMapping.ts:6-9`) which maps `open` →
  `data-open`/`data-closed`
  (`packages/react/src/utils/collapsibleOpenStateMapping.ts:26-32`). The attribute string
  constants themselves are re-exported from `TransitionStatusDataAttributes`
  (`packages/react/src/collapsible/panel/CollapsiblePanelDataAttributes.ts:14-16`,
  `packages/react/src/collapsible/root/CollapsibleRootDataAttributes.ts:14-16`).

## DOM/portal strategy and why

- No portal in the entire unit (behavior spec, "DOM structure": N/A). All three parts render
  inline: Root `div` (`packages/react/src/collapsible/root/CollapsibleRoot.tsx:60`), Panel `div`
  (`packages/react/src/collapsible/panel/CollapsiblePanel.tsx:100`), Trigger `button`
  (`packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:50`). This is consistent with
  the component's role — a disclosure pattern needs no positioning layer, unlike popup-style
  components.
- Unmounting is conditional rendering at the component boundary, not CSS: the panel returns
  `null` when `!shouldRender`
  (`packages/react/src/collapsible/panel/CollapsiblePanel.tsx:128-130`), while `hidden` remains a
  normal boolean attribute for the kept-mounted case
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:392`).
- `hidden="until-found"` cannot be expressed through React's boolean `hidden` prop (React
  coerces the enum value), so it is forced imperatively post-commit:
  `panel.setAttribute('hidden', 'until-found')` in a layout effect
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:346-357`).
- All parts render through the shared `useRenderElement` pipeline
  (`packages/react/src/internals/useRenderElement.tsx:22-48`): state → data attributes via
  `getStateAttributesProps` (`packages/react/src/internals/useRenderElement.tsx:76-78`), merged
  refs including the `render` prop element's own ref
  (`packages/react/src/internals/useRenderElement.tsx:99-103`), resolved `className`/`style`
  functions of state, and `render` prop evaluation. That single pipeline is what makes the
  render-prop conformance contract uniform across parts.
- The panel stacks its props in a deliberate order
  (`packages/react/src/collapsible/panel/CollapsiblePanel.tsx:108-123`): data-attr props → CSS
  variable style → user `elementProps` → user `style` resolved against panel state →
  `animationName: 'none'` last. The last entry wins over user inline animation longhands, which
  is precisely the SSR contract in the behavior spec (suppressed `animationName`, preserved
  `animationDuration`).
- Panel exposes `--collapsible-panel-height` and `--collapsible-panel-width` as inline CSS
  variables (`packages/react/src/collapsible/panel/CollapsiblePanelCssVars.ts:5-10`), bound from
  measured dimensions with `'auto'` for `undefined`
  (`packages/react/src/collapsible/panel/CollapsiblePanel.tsx:111-116`). The px/auto flip is the
  mechanism behind the behavior spec's height-var observations (`auto` when settled, px during
  ending); authors target these vars for height-based transitions/keyframes.

## Dependencies on other Base UI internals

Everything the unit reaches into, grouped. Call-site citations are to collapsible source;
the callee paths are given for dependency mapping.

`@base-ui/utils` (packages/utils, public shared utils):

| Utility | Call site | Used for |
| --- | --- | --- |
| `useControlled` | `packages/react/src/collapsible/root/useCollapsibleRoot.ts:3,16` | controlled/uncontrolled `open` |
| `useStableCallback` | `packages/react/src/collapsible/root/CollapsibleRoot.tsx:3,33`, `packages/react/src/collapsible/root/useCollapsibleRoot.ts:4,30`, `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:7,98,108,113,124` | stable handler identities across context/effects |
| `useIsoLayoutEffect` | `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:4,130,148,346`, `packages/react/src/collapsible/panel/CollapsiblePanel.tsx:3,64` | pre-paint measurement/suppression and id registration |
| `useMergedRefs` | `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:5,68` | external ref + internal panel ref |
| `useValueAsRef` | `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:8,69` | reopen race guard in close completion |
| `addEventListener` | `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:3,379` | shadow-DOM-safe `beforematch` listener |
| `warn` | `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:9,427`, `packages/react/src/collapsible/panel/CollapsiblePanel.tsx:4,40` | dual-motion + `keepMounted`/`hiddenUntilFound` warnings |
| `ownerWindow` | `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:10,414` | realm-safe computed styles |
| `AnimationFrame` | `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:6,508,511,328,333` | one-frame waits (layout-style restore, ending-style commit) |

`packages/react/src/internals` (private):

- `useTransitionStatus` (`packages/react/src/internals/useTransitionStatus.ts:17-97`) — root's
  mounted/status machine (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:8,23`).
- `useBaseUiId` (`packages/react/src/internals/useBaseUiId.ts:9-11`) — default panel id
  (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:5,25`).
- `createChangeEventDetails` + `REASONS`
  (`packages/react/src/internals/createBaseUIEventDetails.ts:118-149`) — the shared
  cancelable-details protocol for both `triggerPress` (root) and `none` (beforematch) reasons
  (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:6-7,32`,
  `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:12-13,367`).
- `useOpenChangeComplete` (`packages/react/src/internals/useOpenChangeComplete.tsx:9-28`) — open
  completion (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:14,279`).
- `useAnimationsFinished` (`packages/react/src/internals/useAnimationsFinished.ts:44-158`) —
  close completion (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:15,71`).
- `useRenderElement` (`packages/react/src/internals/useRenderElement.tsx:22-48`) — all three
  parts' rendering pipeline
  (`packages/react/src/collapsible/root/CollapsibleRoot.tsx:5,60`,
  `packages/react/src/collapsible/panel/CollapsiblePanel.tsx:7,99`,
  `packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:6,50`).
- `useButton` (`packages/react/src/internals/use-button/useButton.ts:14-243`) — trigger button
  semantics (`packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:8,44`).
- `getStateAttributesProps` + `transitionStatusMapping` (used via
  `packages/react/src/collapsible/root/stateAttributesMapping.ts:4,8` and
  `packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:4-5,13-14`) — state→attribute
  mapping, including `data-starting-style`/`data-ending-style`
  (`packages/react/src/internals/stateAttributesMapping.ts:7-10`).
- `HTMLProps` / `BaseUIComponentProps` / `NativeButtonProps` types
  (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:11`,
  `packages/react/src/collapsible/root/CollapsibleRoot.tsx:4`,
  `packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:7`).

`packages/react/src/utils`:

- `collapsibleOpenStateMapping` (`packages/react/src/utils/collapsibleOpenStateMapping.ts:13-32`) —
  shared with Accordion-family consumers; note the reverse dependency: it imports this unit's
  data attributes (`packages/react/src/utils/collapsibleOpenStateMapping.ts:2-3`), so the
  attribute contract of collapsible is a public seam for other units.
- `resolveStyle` — panel resolves its own `style` function against panel state (not root state)
  before handing it to `useRenderElement`
  (`packages/react/src/collapsible/panel/CollapsiblePanel.tsx:6,97`), because the panel's
  effective `transitionStatus` can diverge from the root's via `forcePanelIdle`.

External packages: none beyond `@floating-ui/utils/dom` used transitively *inside* `useButton`
for an `isHTMLElement` type guard
(`packages/react/src/internals/use-button/useButton.ts:3`) — no `floating-ui-react`, no
positioning, no `use-render` package import in this unit.

Public surface files: `packages/react/src/collapsible/index.parts.ts:1-3` maps the three
components to `Root`/`Trigger`/`Panel`; `packages/react/src/collapsible/index.ts:1-5` re-exports
the namespace plus all public types. `packages/react/src/collapsible/root/CollapsibleRoot.spec.tsx:21-40`
pins the type-level surface (props/state namespaces, `ChangeEventReason`/`ChangeEventDetails`),
which is compile-time only.

## Anything in source not explained by any test

Behavior spec sections marked UNVERIFIED are already on record there; the items below are
source-level facts with no behavioral evidence at all (in either the behavior spec's mined tests
or elsewhere in its citations):

1. `--collapsible-panel-width` has no behavioral coverage: it is defined, documented, and bound
   exactly like the height var
   (`packages/react/src/collapsible/panel/CollapsiblePanelCssVars.ts:10`,
   `packages/react/src/collapsible/panel/CollapsiblePanel.tsx:114-115`), and it *is* fed by the
   `scrollWidth` measurement (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:406`),
   but the behavior spec documents only the height var. Its px/auto lifecycle is unexercised.
2. `allowPropagation`/`isPropagationAllowed` is set up in the shared details factory and exposed
   in the public `eventDetails` type
   (`packages/react/src/internals/createBaseUIEventDetails.ts:136-144`), but no code in this unit
   ever reads `isPropagationAllowed` — for collapsible it is inert plumbing. The behavior spec
   already flags `allowPropagation()`'s effect as untested; the source adds that nothing in the
   unit would consume it even if called.
3. The `animation-name: 'none'` set-and-immediately-restore nudge during the ending phase
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:262-265`) has no direct test;
   the behavior spec covers the *outcomes* (unmount timing, no stuck ending state) but never
   asserts this style manipulation exists or what it forces the browser to do with running
   keyframes.
4. `focusableWhenDisabled: true` on the trigger
   (`packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:46`) — the behavior spec's
   keyboard section covers that a disabled trigger doesn't toggle, but no test asserts the
   disabled trigger stays focusable/tabbable.
5. The non-native trigger path: `nativeButton` defaults to `true`
   (`packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:39`) and conformance only
   renders a native button (behavior spec, "Public API surface"). The `nativeButton={false}` /
   `role="button"` mode — including `useButton`'s Space-on-keyup activation and the dev warning
   when a `<button>` is rendered with `nativeButton={false}`
   (`packages/react/src/internals/use-button/useButton.ts:36-66`) — is untested for collapsible.
6. Per-trigger `disabled` override (`packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx:37`)
   lets a trigger be enabled/disabled independently of the root; tests exercise only root-level
   `disabled`.
7. `shouldPersistHiddenTransitionStyles` rendering an empty `data-starting-style` attribute on a
   hidden `hiddenUntilFound` panel
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:92-93,389-391`) — the behavior
   spec records the *outcome* (no leftover hidden transition) but never asserts the attribute
   mechanism or its CSS-authoring contract (`@starting-style` interplay).
8. The root exposes a wider mutation surface to context than any test uses: `setMounted`,
   `setOpen`, and `setPanelIdState` are public context members
   (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:52-53,63-64`), and
   `useCollapsibleRoot`'s state type explicitly documents `mounted` ≠ "in the DOM"
   (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:103-107`). Third-party consumers
   (and sibling units like Accordion, which share the attribute mapping via
   `packages/react/src/utils/collapsibleOpenStateMapping.ts`) may rely on this surface; the
   collapsible tests themselves never call the setters directly.
9. `useBaseUiId` accepts an `idOverride` parameter
   (`packages/react/src/internals/useBaseUiId.ts:9-11`) that the root never passes
   (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:25`) — dead capability from this
   unit's perspective, present in source but unreachable through collapsible's API.
10. The `deferEndingState`/`enableIdleState` tuple choice (`true, true`) at
    `packages/react/src/collapsible/root/useCollapsibleRoot.ts:23` means `'idle'` is a real,
    observable phase for authors (panel is open, no `data-starting-style`, before `ending`).
    The behavior spec's phase observations imply it but no test names or asserts the `'idle'`
    value of `transitionStatus` directly.
