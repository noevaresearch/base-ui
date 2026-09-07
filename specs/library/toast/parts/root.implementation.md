# Toast.Root — implementation spec (batch: `root`)

Ground truth for WHAT is `specs/library/toast/parts/root.md` (the batch behavior spec); this file
explains WHY/HOW for the four files of this batch only:

- `packages/react/src/toast/root/ToastRoot.tsx`
- `packages/react/src/toast/root/ToastRootContext.ts`
- `packages/react/src/toast/root/ToastRootCssVars.ts`
- `packages/react/src/toast/root/ToastRootDataAttributes.ts`

## State machine / hooks used

ToastRoot is stateless with respect to toast *existence* — all lifecycle authority lives in the
provider store. The component's local state is the *swipe gesture machine* plus *aria id
registration* and a *height-measurement side effect*.

### Local state (React `useState`)

- `currentSwipeDirection` — the armed swipe direction; renders as `data-swipe-direction` through
  the state mapping (`packages/react/src/toast/root/ToastRoot.tsx:71-73`,
  `packages/react/src/toast/root/ToastRoot.tsx:502`).
- `isSwiping` — a pointer drag is active; renders as `data-swiping`
  (`packages/react/src/toast/root/ToastRoot.tsx:74`, `packages/react/src/toast/root/ToastRoot.tsx:501`).
- `isRealSwipe` — the gesture has moved at least `MIN_DRAG_THRESHOLD` px
  (`packages/react/src/toast/root/ToastRoot.tsx:75`); only meaningful as a gate for axis locking
  and direction latching (`packages/react/src/toast/root/ToastRoot.tsx:331-347`). It is internal
  state that is never rendered directly.
- `dragOffset` and `initialTransform` — the live visual position and the transform snapshot taken
  at gesture start (`packages/react/src/toast/root/ToastRoot.tsx:76-77`). Each has a mirrored ref
  (`dragOffsetRef`, `initialTransformRef`,
  `packages/react/src/toast/root/ToastRoot.tsx:87,95`) so the document-level `pointerup` handler
  (a stable callback, not re-created per render) always reads fresh values; `setResolvedDragOffset`
  keeps both in sync (`packages/react/src/toast/root/ToastRoot.tsx:169-172`).
- `titleId` / `descriptionId` — DOM ids registered by `Toast.Title`/`Toast.Description` (leaf
  parts, outside this batch) and rendered as `aria-labelledby`/`aria-describedby`
  (`packages/react/src/toast/root/ToastRoot.tsx:78-79`,
  `packages/react/src/toast/root/ToastRoot.tsx:466-467`). This is the implementation behind the
  spec's "Accessibility" section.
- `lockedDirection` — axis lock (`'horizontal' | 'vertical' | null`) chosen on the first real
  move of a gesture when both axes are swipeable
  (`packages/react/src/toast/root/ToastRoot.tsx:80-82`,
  `packages/react/src/toast/root/ToastRoot.tsx:338-345`).

### Per-gesture refs (not state, on purpose)

`dragStartPosRef`, `intendedSwipeDirectionRef`, `maxSwipeDisplacementRef`, `cancelledSwipeRef`,
`swipeCancelBaselineRef`, `isFirstPointerMoveRef`, `activePointerIdRef`,
`dragAbortControllerRef` (`packages/react/src/toast/root/ToastRoot.tsx:86-97`) hold the gesture's
bookkeeping. They are refs because pointermove fires far more often than React should re-render —
only the *rendered* outcomes (armed direction, swiping flag, visual offset) go through `setState`.

### Store subscriptions

`store.useState` selectors (`packages/react/src/toast/root/ToastRoot.tsx:99-103`) subscribe to
`'toastIndex'`, `'toastVisibleIndex'`, `'toastOffsetY'` **keyed by `toast.id`**, plus the global
`'focused'` and `'expanded'`. The id-keyed selectors are what let a retained Root instance follow
the toast it currently renders (see the index-keyed-list edge cases in the spec's "Edge cases"
section): identity is the toast id, never the object reference (the object-identity regression in
the spec).

### Swipe gesture FSM (idle → armed → real → resolved)

1. **idle → armed** in `handlePointerDown`
   (`packages/react/src/toast/root/ToastRoot.tsx:240-291`): requires primary button
   (`packages/react/src/toast/root/ToastRoot.tsx:241-243`), a target that is not an interactive
   element — `button,a,input,textarea,[role="button"]` plus the current and legacy swipe-ignore
   selectors (`packages/react/src/toast/root/ToastRoot.tsx:249-257`) — and a non-anchored toast
   (swipe is disabled entirely when `positionerProps.anchor` is set, see
   `packages/react/src/toast/root/ToastRoot.tsx:60-67`, which the spec's "Events" section proves
   with the anchored-swipe test). Arming records the start position, snapshots the element's
   current transform via `getElementTransform`
   (`packages/react/src/toast/root/ToastRoot.tsx:268-274`), sets `hovering` in the store for touch
   timer pausing (`packages/react/src/toast/root/ToastRoot.tsx:245-247,276`), and registers
   document-level `pointerup`/`pointercancel` listeners through an `AbortController` plus
   `setPointerCapture` (`packages/react/src/toast/root/ToastRoot.tsx:282-290`).
2. **armed → real** in `handlePointerMove`
   (`packages/react/src/toast/root/ToastRoot.tsx:293-407`): the first move re-baselines the start
   position to compensate for the iOS pointerdown→pointermove delay
   (`packages/react/src/toast/root/ToastRoot.tsx:301-306`); movement ≥ `MIN_DRAG_THRESHOLD` px
   marks the gesture real and, only when both axes are swipeable, locks the dominant axis
   (`packages/react/src/toast/root/ToastRoot.tsx:331-347` — the comment explains the lock is
   redundant for single-axis configs). The intended direction latches once a candidate is in
   `swipeDirections` (`packages/react/src/toast/root/ToastRoot.tsx:349-373`), recording max
   displacement for reversal detection. The cancel baseline tracks direction turn-arounds
   (`packages/react/src/toast/root/ToastRoot.tsx:310-322`); once a direction is latched,
   displacement > `SWIPE_THRESHOLD` re-arms dismissal while dropping ≥ `REVERSE_CANCEL_THRESHOLD`
   px below max displacement marks a change of mind (`cancelledSwipeRef.current = true`,
   `packages/react/src/toast/root/ToastRoot.tsx:374-389`). The visual offset is the initial
   transform plus the damped delta, where movement in non-swipeable directions is damped with a
   0.5-power curve (`applyDirectionalDamping`,
   `packages/react/src/toast/root/ToastRoot.tsx:180-197`,
   `packages/react/src/toast/root/ToastRoot.tsx:391-406`) — this produces the spec's "damped,
   opposite-direction path" behavior.
3. **resolved** in `handleSwipeEnd`
   (`packages/react/src/toast/root/ToastRoot.tsx:199-238`): gated on the recorded `pointerId`,
   tears down the listeners via the abort controller and resets all gesture flags
   (`packages/react/src/toast/root/ToastRoot.tsx:200-209`). `pointercancel` or a change of mind
   restores the initial transform (`packages/react/src/toast/root/ToastRoot.tsx:213-217`);
   otherwise the first direction in `swipeDirections` whose displacement exceeds
   `SWIPE_THRESHOLD` wins — the direction is committed and `store.closeToast(toast.id)` starts the
   exit lifecycle (`packages/react/src/toast/root/ToastRoot.tsx:224-233`); below threshold the
   toast springs back (`packages/react/src/toast/root/ToastRoot.tsx:234-237`). Listening at the
   document level is why the spec's "release on document" and "document-level pointercancel" edge
   cases work.

### Toast lifecycle FSM

- **Enter**: a `useIsoLayoutEffect` keyed on `[toast.id, toast.transitionStatus]`
  (`packages/react/src/toast/root/ToastRoot.tsx:149-167`) initializes the toast on mount and
  re-initializes when a *retained root instance* begins a new lifecycle — either re-adding an
  ending toast (`key={toast.id}` reuse) or an index-keyed list handing the instance a different
  toast (spec "Edge cases"). `lastToastIdRef` guards re-entry: `recalculateHeight` itself clears
  the `starting` status, which would re-trigger the effect, so the guard bails when the id is
  unchanged and the status is not `starting`
  (`packages/react/src/toast/root/ToastRoot.tsx:153-155`). On reuse, component-local swipe state
  is explicitly cleared so the node doesn't stay offset or exit in the old swipe direction
  (`packages/react/src/toast/root/ToastRoot.tsx:157-163`).
- **Height measurement**: `recalculateHeight`
  (`packages/react/src/toast/root/ToastRoot.tsx:118-144`) measures the natural height by
  temporarily forcing `height: auto`, reading `offsetHeight`, and restoring the previous value;
  it then writes `{ ref: rootRef, height, transitionStatus: undefined }` into the store via
  `store.updateToastInternal` (`packages/react/src/toast/root/ToastRoot.tsx:131-137`). The
  optional `flushSync` argument exists because callers outside this batch (leaf parts' observers)
  invoke it from ResizeObserver callbacks; the store ignores the write while the toast is
  transitioning out (`packages/react/src/toast/root/ToastRoot.tsx:115-117`). Publishing `rootRef`
  into the store is how the viewport/stack logic can measure and position this toast without
  another DOM handle.
- **Exit**: `useOpenChangeComplete` derives `open` from the toast object —
  `open: toast.transitionStatus !== 'ending'` — and calls `store.removeToast(toast.id)` in
  `onComplete` (`packages/react/src/toast/root/ToastRoot.tsx:105-113`). There is no local `open`
  state: enter/exit are purely the store's `transitionStatus` transitions observed through the
  `toast` prop, which is what makes the spec's "rapid add→close→add" node-reuse behavior possible
  without remounting React state.
- **Unmount cleanup**: the active drag's `AbortController` is aborted on unmount
  (`packages/react/src/toast/root/ToastRoot.tsx:174-178`).

### Styling hooks

`getDragStyles` (`packages/react/src/toast/root/ToastRoot.tsx:444-458`) emits inline
`transition: 'none'` and a frozen `translateX/translateY/scale` only while swiping (so the element
tracks the finger instead of snapping to the end position); when idle both are `undefined` so CSS
transitions own the motion. The computed drag deltas are exported as CSS custom properties.

### Hooks used elsewhere in the file

- `useStableCallback` wraps `recalculateHeight` and `handleSwipeEnd`
  (`packages/react/src/toast/root/ToastRoot.tsx:118,199`) because both are consumed from effects
  and document-level listeners.
- The `touchmove` blocker is a plain `React.useEffect` returning `addEventListener`'s cleanup
  (`packages/react/src/toast/root/ToastRoot.tsx:422-442`).
- The context value object is memoized so consumers re-render only on meaningful changes
  (`packages/react/src/toast/root/ToastRoot.tsx:484-494`).

## Context providers/consumers

- **Provides** `ToastRootContext`
  (`packages/react/src/toast/root/ToastRoot.tsx:484-494`,
  `packages/react/src/toast/root/ToastRoot.tsx:512`; defined in
  `packages/react/src/toast/root/ToastRootContext.ts:14`): `{ toast, setTitleId,
  setDescriptionId, recalculateHeight, visibleIndex, expanded }`. Leaf parts are the consumers:
  Title/Description call `setTitleId`/`setDescriptionId` to register their DOM ids (the mechanism
  behind the spec's aria-linking tests), and content-side parts read `visibleIndex` (for
  `data-behind`-style stack styling) and `expanded`. Their internal usage is documented in the
  leaf-parts batch, not here.
- **Consumes** the provider store via `useToastProviderContext()`
  (`packages/react/src/toast/root/ToastRoot.tsx:69`; hook defined at
  `packages/react/src/toast/provider/ToastProviderContext.ts:9-15`). The context value *is* the
  `ToastStore` (`packages/react/src/toast/provider/ToastProviderContext.ts:5`), so Root talks
  directly to the store with:
  - selectors: `store.useState('toastIndex', toast.id)`, `('toastVisibleIndex', toast.id)`,
    `('toastOffsetY', toast.id)`, `'focused'`, `'expanded'`
    (`packages/react/src/toast/root/ToastRoot.tsx:99-103`);
  - mutators: `store.updateToastInternal`
    (`packages/react/src/toast/root/ToastRoot.tsx:132`), `store.removeToast`
    (`packages/react/src/toast/root/ToastRoot.tsx:110`), `store.closeToast`
    (`packages/react/src/toast/root/ToastRoot.tsx:233`), `store.pauseTimers`
    (`packages/react/src/toast/root/ToastRoot.tsx:246`), `store.set('hovering', true)`
    (`packages/react/src/toast/root/ToastRoot.tsx:276`).
  The store's internals are the manager batch's domain (`specs/library/toast/parts/manager.implementation.md`).
- `useToastRootContext`
  (`packages/react/src/toast/root/ToastRootContext.ts:16-24`) is the typed accessor leaf parts use
  and throws with the "Toast parts must be used within <Toast.Root>" error when missing — the
  composition boundary that makes Root the scope owner for all per-toast state.
- Note the deliberate split: Root *provides* per-toast context downward and *consumes* the
  provider store upward; it never re-exposes the store through its own context, so leaf parts that
  need store access go through props fed by Root or their own store subscription, not Root's
  context.

## DOM/portal strategy and why

- Root renders an inline `<div>` via `useRenderElement('div', ...)`
  (`packages/react/src/toast/root/ToastRoot.tsx:505-510`) and performs **no portal of its own** —
  placing the element in the document (inside the viewport container) is the Viewport batch's job.
  This has a direct behavioral consequence proven by the spec: a React child portaled to
  `document.body` is a React descendant of Root but not a DOM descendant, so keydown still bubbles
  through the React tree while `event.target` lives outside the toast. That is exactly why
  `handleKeyDown` gates Escape on DOM containment —
  `contains(rootRef.current, activeElement(ownerDocument(rootRef.current)))`
  (`packages/react/src/toast/root/ToastRoot.tsx:409-420`) — which is the source of the spec's
  "Escape ignored for portaled content" test.
- A11y decisions baked into `defaultProps`
  (`packages/react/src/toast/root/ToastRoot.tsx:462-482`):
  - `role: 'alertdialog'` when `toast.priority === 'high'`, else `'dialog'`
    (`packages/react/src/toast/root/ToastRoot.tsx:463`);
  - `aria-modal: false` (`packages/react/src/toast/root/ToastRoot.tsx:465`) — toasts never trap
    focus;
  - `tabIndex: 0` (`packages/react/src/toast/root/ToastRoot.tsx:464`) — each root is a
    keyboard-reachable tab stop (the `{F6}`+`{Tab}` recipe in the spec lands here);
  - `aria-hidden` only on high-priority toasts that are not focused
    (`packages/react/src/toast/root/ToastRoot.tsx:468`);
  - `inert` when `toast.limited` (`packages/react/src/toast/root/ToastRoot.tsx:474`) — toasts
    overflowed past the limit are rendered but non-interactive.
- **Pointer capture + document listeners**: `setPointerCapture` keeps pointermove events flowing
  to the root while the pointer is down, but `pointerup`/`pointercancel` are *also* listened for
  at the document level because browsers may deliver the final event outside the captured element
  (`packages/react/src/toast/root/ToastRoot.tsx:286-290`) — matching the spec's document-release
  tests. The `AbortController` makes teardown deterministic on gesture end and unmount.
- **Non-passive `touchmove` listener**
  (`packages/react/src/toast/root/ToastRoot.tsx:422-442`): React's `preventDefault` on pointermove
  (`packages/react/src/toast/root/ToastRoot.tsx:298-299`) is insufficient on iOS; a native
  non-passive listener blocks scrolling while a drag is active. It is gated on
  `activePointerIdRef` plus DOM containment so it only fires for the active gesture (matching the
  spec's touchmove-preventDefault test triple).
- **CSS variable emission** is the styling contract with the Viewport/Positioner and user CSS:
  `--toast-index`, `--toast-offset-y`, `--toast-height` are written in `defaultProps.style`
  (`packages/react/src/toast/root/ToastRoot.tsx:477-480`); `--toast-swipe-movement-x/y` come from
  `getDragStyles` (`packages/react/src/toast/root/ToastRoot.tsx:455-456`). Notably
  `--toast-index` switches source while ending — `domIndex` for a toast animating out (keeps its
  former stack slot) versus `visibleIndex` otherwise
  (`packages/react/src/toast/root/ToastRoot.tsx:477-478`).
- **State attributes** are rendered through `state` +
  `toastRootStateAttributesMapping`
  (`packages/react/src/toast/root/ToastRoot.tsx:28-33`,
  `packages/react/src/toast/root/ToastRoot.tsx:496-510`): `swipeDirection` maps to the
  `data-swipe-direction` attribute only when defined
  (`packages/react/src/toast/root/ToastRoot.tsx:30-32`); the remaining state fields rely on the
  default state mapping inside `useRenderElement`.

### The two const files

- `packages/react/src/toast/root/ToastRootCssVars.ts` declares the five custom properties
  (`--toast-index` at `packages/react/src/toast/root/ToastRootCssVars.ts:5`, `--toast-offset-y`
  at `packages/react/src/toast/root/ToastRootCssVars.ts:10`, `--toast-height` at
  `packages/react/src/toast/root/ToastRootCssVars.ts:15`, `--toast-swipe-movement-x/y` at
  `packages/react/src/toast/root/ToastRootCssVars.ts:20-24`). It is a pure string-constant module
  so CSS and TS stay in sync.
- `packages/react/src/toast/root/ToastRootDataAttributes.ts` declares `data-expanded`,
  `data-limited`, `data-type`, `data-swiping`, `data-swipe-direction`
  (`packages/react/src/toast/root/ToastRootDataAttributes.ts:7-27`) and re-exports the shared
  transition attributes `startingStyle`/`endingStyle` from
  `TransitionStatusDataAttributes`
  (`packages/react/src/toast/root/ToastRootDataAttributes.ts:31-35`), keeping Root's transition
  attributes identical to every other animated component.

## Dependencies on other Base UI internals

Imports from outside this batch (cited, not re-derived):

- `packages/react/src/internals/useRenderElement.tsx:22` — `useRenderElement`, the shared
  element renderer handling props merging, state→attribute mapping, and ref merging.
- `packages/react/src/internals/useOpenChangeComplete.tsx:9` — `useOpenChangeComplete`, fires
  `onComplete` after the exit transition finishes; Root's removal-from-store hook.
- `packages/react/src/internals/stateAttributesMapping.ts:10` — `transitionStatusMapping`,
  spread into Root's mapping; the same module exports `TransitionStatusDataAttributes` consumed
  by `packages/react/src/toast/root/ToastRootDataAttributes.ts:1`.
- `packages/react/src/internals/getStateAttributesProps.ts:1` — `StateAttributesMapping` type.
- `packages/react/src/internals/useTransitionStatus.ts:6` — `TransitionStatus` type used by
  `ToastRootState`.
- `packages/react/src/internals/constants.ts:11-12` — `BASE_UI_SWIPE_IGNORE_SELECTOR` and
  `LEGACY_SWIPE_IGNORE_SELECTOR`, combined into `TOAST_SWIPE_IGNORE_SELECTOR`
  (`packages/react/src/toast/root/ToastRoot.tsx:39`).
- `packages/react/src/internals/types` — `BaseUIComponentProps` and `HTMLProps` for the props
  interfaces.
- `packages/react/src/toast/provider/ToastProviderContext.ts:9` — `useToastProviderContext`
  (store access; see Context section).
- `packages/react/src/toast/useToastManager.ts:26` — `ToastObject` type re-exported through
  `ToastRoot.Props`.
- `packages/react/src/utils/useSwipeDismiss.ts:39` — `getDisplacement`, the signed-axis
  displacement math shared with other swipe-dismissable components.
- `packages/react/src/utils/getElementTransform.ts:10` — `getElementTransform`, reads the
  current translate/scale off computed style so drag offsets compose on top of the positioner's
  transform.
- `packages/react/src/floating-ui-react/utils.ts:1` — barrel re-exporting `contains`,
  `getTarget`, `activeElement` from `packages/react/src/floating-ui-react/utils/element.ts:3-8`,
  which re-exports the shadow-DOM-safe implementations at
  `packages/utils/src/shadowDom.ts:3`, `packages/utils/src/shadowDom.ts:13`,
  `packages/utils/src/shadowDom.ts:39`.
- `packages/utils/src/owner.ts:3` — `ownerDocument` (document/owner lookups tied to the node).
- `packages/utils/src/inertValue.ts:3` — `inertValue` (boolean → attribute value).
- `packages/utils/src/useIsoLayoutEffect.ts:6` — `useIsoLayoutEffect` for the mount/lifecycle
  effect.
- `packages/utils/src/useStableCallback.ts:35` — `useStableCallback` for listener-invoked
  handlers.
- `packages/utils/src/addEventListener.ts:37` — `addEventListener` returning a cleanup for the
  non-passive touchmove listener.
- `react-dom`'s `flushSync` is a direct dependency
  (`packages/react/src/toast/root/ToastRoot.tsx:3`,
  `packages/react/src/toast/root/ToastRoot.tsx:140`).

No `wraps-external:` field exists for this unit (there is no `specs/library/toast/TODO.md`), so
there is no third-party algorithm delegation to document.

## Anything in source not explained by any test

Derived from the spec's UNVERIFIED markers plus genuine gaps:

1. **Default `swipeDirection` value** — `['down', 'right']`
   (`packages/react/src/toast/root/ToastRoot.tsx:55`) is never asserted; the spec only proves a
   default-configured toast dismisses rightward.
2. **Exact threshold constants** — `SWIPE_THRESHOLD = 40`
   (`packages/react/src/toast/root/ToastRoot.tsx:35`) is only bracketed (10px fails, 45px passes
   per the spec's "Swipe thresholds"); `REVERSE_CANCEL_THRESHOLD = 10`
   (`packages/react/src/toast/root/ToastRoot.tsx:36`), `MIN_DRAG_THRESHOLD = 1`
   (`packages/react/src/toast/root/ToastRoot.tsx:38`), and the 0.5 damping exponent
   (`packages/react/src/toast/root/ToastRoot.tsx:37`) are unpinned — the reversal and
   damped-path tests observe outcomes, not the curve.
3. **During-drag inline styles** — `getDragStyles`' `transition: 'none'` and frozen transform
   (`packages/react/src/toast/root/ToastRoot.tsx:448-454`) are never asserted while active; the
   spec only verifies they are *cleared* after reset.
4. **Role and ARIA defaults** — the spec states no `role` assertion exists in the Root test file;
   `role: 'alertdialog'`/`'dialog'` (`packages/react/src/toast/root/ToastRoot.tsx:463`),
   `aria-modal: false` (`packages/react/src/toast/root/ToastRoot.tsx:465`), the `aria-hidden` on
   unfocused high-priority toasts (`packages/react/src/toast/root/ToastRoot.tsx:468`), and
   `inert` when limited (`packages/react/src/toast/root/ToastRoot.tsx:474`) are all untested
   here. The test comment tying Escape to `:focus-visible` is also broader than the source: the
   actual gate is DOM containment of the active element
   (`packages/react/src/toast/root/ToastRoot.tsx:411-416`), not focus modality.
5. **`store.pauseTimers()` on touch pointerdown**
   (`packages/react/src/toast/root/ToastRoot.tsx:245-247`) — timer semantics are explicitly out
   of scope for this test file (spec "Timers/auto-dismiss" section), so the touch-pause hook is
   untested here.
6. **`store.set('hovering', true)` on pointerdown**
   (`packages/react/src/toast/root/ToastRoot.tsx:276`) — no hovering assertion exists, and Root
   never resets `hovering` itself; the hover lifecycle belongs to the store/provider batches.
7. **Ending-phase `--toast-index` switch** — `domIndex` vs `visibleIndex` based on
   `transitionStatus === 'ending'`
   (`packages/react/src/toast/root/ToastRoot.tsx:477-478`) is not distinguished by tests, which
   only assert settled/stacked index values.
8. **Cancel-baseline mechanics** — the `swipeCancelBaselineRef` re-baselining
   (`packages/react/src/toast/root/ToastRoot.tsx:310-322`) is only observable through the
   reversal-cancel outcome, not the turning-point measurement itself.
9. **`isRealSwipe` / first-move re-baseline subtleties** — the test helper's 1px probe move
   implicitly exercises them, but no test distinguishes the armed-not-real state from real
   movement, and `isRealSwipe` is never rendered.
10. **`recalculateHeight`'s `flushSync` argument**
    (`packages/react/src/toast/root/ToastRoot.tsx:118,139-143`) — its call sites live outside
    this batch; Root's tests observe height updates but never the synchronous path.
11. **Store discarding writes while ending** — the comment at
    `packages/react/src/toast/root/ToastRoot.tsx:117` describes store behavior owned by the
    manager batch, untested in this file.
12. **Unexercised state fields** — `data-expanded`, `data-limited`, and `data-type` are rendered
    from state (`packages/react/src/toast/root/ToastRoot.tsx:496-503`) but the spec notes this
    test file exercises none of expanded/limited/type behavior; they are verified (if at all) in
    other batches.
