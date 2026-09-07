# Toast — behavior spec (Stage 1 index)

Mined from the unit's 15 test files under `packages/react/src/toast/` only. The unit is flagged
`needs-batched-mining: true` in `TODO.md`, so depth lives in the per-batch part specs under
`parts/` (mined per subdirectory batch); this file is the index plus whole-unit cross-cutting
behavior. No `wraps-external:` field exists on this unit's TODO entry — behavior is derived
directly from the tests.

Cross-cutting data flow: manager methods (called imperatively on a `Toast.createToastManager()`
instance, or via `useToastManager()` inside `Toast.Provider`) mutate a single store; the Provider
syncs its `timeout`/`limit` props into that store; the app maps `toasts` to one `Toast.Root` per
toast inside `Toast.Viewport` (newest-first in the DOM); Root renders the leaf parts, which source
content from the toast object. `specs/library/toast/parts/manager.md:30-31`, `specs/library/toast/parts/viewport.md:59-66`

## Public API surface (props, parts, subcomponents)

- One namespace (`Toast` from `@base-ui/react/toast`) exposes `Provider`, `Portal`, `Viewport`,
  `Positioner`, `Root`, `Content`, `Title`, `Description`, `Action`, `Close`, `Arrow`, plus the
  `useToastManager` hook and `createToastManager` factory.
  `specs/library/toast/parts/leaf-parts.md:5-23`, `specs/library/toast/parts/shell.md:5-17`
- Shell parts: `Provider` takes `timeout` (ms), `limit`, and `toastManager` (external manager);
  `Portal` renders a `<div>` (test coverage is conformance-only); `Positioner` requires a `toast`
  prop and takes positioning props (`side`, `align`, anchoring via toast `positionerProps`).
  `specs/library/toast/parts/shell.md:5-17`, `specs/library/toast/parts/shell.md:47-57`
- `Root` takes a required `toast` (`Toast.Root.ToastObject`) plus `swipeDirection` (single value or
  array); toast-object fields include `id`, `title`, `description`, `actionProps`, `timeout`,
  `positionerProps`, `type`, `onClose`, `onRemove`, `priority`, `data`.
  `specs/library/toast/parts/root.md:5-15`, `specs/library/toast/parts/manager.md:24`
- Manager/hook surface: `Toast.createToastManager()` works outside React with `add`/`update`/
  `close(id?)`/`promise`; `useToastManager()` returns `{ toasts, add, close, update, promise }`;
  the `ToastStore` class exposes state (`toasts`, `timeout`, `limit`, `hovering`, `focused`,
  `isWindowFocused`, `viewport`, `prevFocusElement`), mutation methods, and
  `selectors.toast/toastIndex/toastOffsetY/toastVisibleIndex`.
  `specs/library/toast/parts/manager.md:5-26`
- Leaf parts' default elements (proven via conformance `refInstanceof`): Action/Close = `<button>`,
  Arrow = generic `Element`, Content = `<div>`, Description = `<p>`, Title = heading; all support
  ref forwarding, prop spreading, `render` prop (element and function forms), and `className`.
  `specs/library/toast/parts/leaf-parts.md:5-23`
- Viewport renders a `<div>` and must sit inside `Toast.Provider` (context-missing error otherwise).
  `specs/library/toast/parts/viewport.md:5-12`

## State model (controlled/uncontrolled, defaults, transitions)

- Two modes: uncontrolled (Provider's internal manager) or controlled (external
  `Toast.createToastManager()` passed via `toastManager`, driven imperatively).
  `specs/library/toast/parts/manager.md:30-31`, `specs/library/toast/parts/shell.md:30`
- Toast lifecycle: a newly added toast has `transitionStatus: 'starting'` and `updateKey: 0`;
  settled roots carry no `data-starting-style`; `close`/timeout moves the toast to
  `transitionStatus: 'ending'` (root renders `data-ending-style`, still mounted for the exit
  animation) and then `removeToast` removes it.
  `specs/library/toast/parts/manager.md:38`, `specs/library/toast/parts/manager.md:51`, `specs/library/toast/parts/root.md:21`
- Timers: default auto-dismiss is 5000 ms; per-toast `timeout` overrides it; `timeout: 0` means
  never; updates/upserts reset the timer; hover (viewport `hovering`) pauses with remaining time
  preserved, and a wall-clock jump past the deadline restarts the full delay on resume.
  `specs/library/toast/parts/manager.md:32-34`, `specs/library/toast/parts/manager.md:45-46`, `specs/library/toast/parts/manager.md:62-70`, `packages/react/src/toast/store.test.ts:350-370`
- Promise toasts: `promise()` renders the `loading` state, then applies `success`/`error` options
  (string, object, or function receiving the resolved value/rejection error) on settle, with
  per-state timeouts falling back to the provider timeout; the loading state's timeout is not
  inherited by success.
  `specs/library/toast/parts/manager.md:55-61`
- Stack coordination: store keeps toasts newest-first (frontmost `--toast-index: '0'`, older
  ascending); a closing toast keeps a real index (0, not -1) while animating out and the rest
  re-index; `limit` flags older toasts `limited: true` (rendered as `data-limited`), recomputed on
  limit changes; `Content` of non-frontmost toasts carries `data-behind`.
  `specs/library/toast/parts/shell.md:23`, `specs/library/toast/parts/shell.md:28-29`, `specs/library/toast/parts/manager.md:63-67`, `specs/library/toast/parts/root.md:25-26`, `packages/react/src/toast/viewport/ToastViewport.test.tsx:1077-1078`
- Root visual state: `--toast-height` (measured, recalculated on content update, cleared while
  ending, restored on re-add), `--toast-offset-y`, `--toast-swipe-movement-x/y`, and
  `data-swiping`/`data-swipe-direction` during swipe gestures.
  `specs/library/toast/parts/root.md:22-27`, `specs/library/toast/parts/root.md:54-55`
- Viewport expansion state (`data-expanded`) is internal: driven by hover/focus-in/touch-swipe
  rules, never by props.
  `specs/library/toast/parts/viewport.md:14-26`

## Keyboard interactions

- F6 (window-level keydown listener, works cross-realm via iframe portals) focuses the viewport
  when toasts exist (and even with no rendered toast); pressing it also pauses dismiss timers.
  `specs/library/toast/parts/viewport.md:28-39`, `packages/react/src/toast/viewport/ToastViewport.test.tsx:163-168`
- Tab from the focused viewport enters the first toast root; Tab order walks toast root → close
  button → action button per toast; shift+Tab on the first toast (or on the viewport itself)
  restores focus to the element focused before F6 and resumes timers only if that element is
  outside the viewport.
  `specs/library/toast/parts/viewport.md:34-38`
- Escape closes the toast that currently has keyboard focus (browser-only, :focus-visible-gated)
  and hands focus to the next active toast; Escape is ignored when focus is inside React-portaled
  (DOM-external) children of a toast.
  `specs/library/toast/parts/root.md:30-34`, `packages/react/src/toast/root/ToastRoot.test.tsx:715-735`, `packages/react/src/toast/root/ToastRoot.test.tsx:737-769`
- Leaf parts, shell parts, and the manager/store/hook have no keyboard behavior asserted.
  `specs/library/toast/parts/leaf-parts.md:34-36`, `specs/library/toast/parts/manager.md:76-78`

## Focus management

- F6 renders focus-guard elements (`data-base-ui-focus-guard`); guard focus targets the first
  toast that is not animating out, falling back to the pre-viewport element when none can take it.
  `specs/library/toast/parts/viewport.md:41-50`
- Closing toasts reorients focus (skipping toasts with `data-ending-style`), but never steals focus
  when focus has never entered the viewport.
  `specs/library/toast/parts/viewport.md:46-48`
- A remounted `Toast.Root` (list hidden then shown) re-registers as active/focusable, proving
  registration is re-established for new DOM nodes; focus moves to the next active toast when the
  focused one closes in an index-keyed list.
  `specs/library/toast/parts/root.md:36-40`
- Store state carries the focus coordinates used above (`focused`, `prevFocusElement`), set by the
  viewport; manager-level focus behavior is otherwise N/A.
  `specs/library/toast/parts/manager.md:80-82`

## Accessibility (roles, aria-*, id linking)

- `Toast.Root` links `aria-labelledby` to the mounted `Toast.Title` id and `aria-describedby` to
  the mounted `Toast.Description` id; the links update on mode switches, are removed when the parts
  unmount, and are restored on remount; with duplicate titles the newest wins and an older title's
  cleanup must not clear a newer link.
  `specs/library/toast/parts/root.md:42-48`, `specs/library/toast/parts/leaf-parts.md:45-50`
- High-priority toasts (`priority: 'high'`) render the root as `role="alertdialog"` with
  `aria-modal="false"` plus a `role="alert"` element with `aria-atomic="true"` (removed on close);
  when added from inside an open dialog the root also gets `aria-hidden="true"`.
  `specs/library/toast/parts/manager.md:86-93`
- `Toast.Arrow` renders `aria-hidden="true"`; Viewport landmark role/aria (e.g. `aria-live`) is
  UNVERIFIED — no test asserts it.
  `specs/library/toast/parts/leaf-parts.md:51`, `specs/library/toast/parts/viewport.md:52-57`

## DOM structure & portal behavior

- Canonical nesting is `Toast.Provider` > `Portal` (optional) > `Viewport` > (`Positioner` >)
  `Root` > leaf parts; toasts render inside the viewport newest-first.
  `specs/library/toast/parts/leaf-parts.md:54-58`, `specs/library/toast/parts/viewport.md:59-66`
- Portal is covered only by conformance (renders/forwards a ref to `HTMLDivElement`, rendered
  standalone); container/reparenting behavior is UNVERIFIED in the test suite.
  `specs/library/toast/parts/shell.md:49-51`
- Positioner renders a `<div>` exposing `data-side`/`data-align` (element props override toast
  `positionerProps`, unset props fall back), anchors geometrically to `positionerProps.anchor`
  (browser-only), and writes `--toast-index` inline.
  `specs/library/toast/parts/shell.md:26-29`, `specs/library/toast/parts/shell.md:53-57`
- The viewport's window/document listeners attach to the owner realm: a viewport portaled into an
  iframe binds to the iframe window/document; viewport inline `--toast-frontmost-height` is set
  after a toast is added (browser-only).
  `specs/library/toast/parts/viewport.md:63-64`
- Context-missing errors are descriptive `Base UI:` messages rejecting the render (Arrow outside
  Positioner, Title outside Root, Viewport/Positioner outside Provider).
  `specs/library/toast/parts/leaf-parts.md:59-61`, `specs/library/toast/parts/shell.md:16`, `specs/library/toast/parts/viewport.md:9`
- React-portaled children of a toast root live outside it in the DOM (relevant to the Escape rule
  above); the specific portal target of Viewport itself is UNVERIFIED.
  `specs/library/toast/parts/root.md:53`, `specs/library/toast/parts/manager.md:104`

## Events (names, payload shape, bubbling, preventDefault semantics)

- Behavior is expressed through toast-object callbacks, not custom DOM events: `onClose` fires
  once per toast termination (programmatic close or timeout; an already-ending toast is not
  re-notified) and `onRemove` fires once on removal (not when a closing toast is replaced).
  `specs/library/toast/parts/manager.md:109-115`, `packages/react/src/toast/createToastManager.test.tsx:790-841`
- Swipe gestures are a pointer sequence on the root (`pointerdown`/`pointermove`/
  `pointerup`/`pointercancel`): non-primary buttons are ignored, `touchmove` is preventDefault-ed
  only mid-swipe, gestures on `data-base-ui-swipe-ignore` (and legacy `data-swipe-ignore`)
  descendants are ignored, and anchored toasts don't swipe; document-level release/cancel ends the
  gesture.
  `specs/library/toast/parts/root.md:60-67`, `specs/library/toast/parts/root.md:83-85`
- Viewport listener inventory: window `keydown` + `blur`/`focus` (capture) + document
  `pointerdown`, each added/removed exactly once per non-empty store cycle, bound to the owner
  realm; mouseenter/leave and touch pointer events drive expansion per the State model rules.
  `specs/library/toast/parts/viewport.md:70-76`
- Click on `Toast.Close` removes the toast; clicking the action area is only exercised via the
  fixture (no custom event names, payloads, or bubbling asserted anywhere).
  `specs/library/toast/parts/leaf-parts.md:64-69`, `specs/library/toast/parts/shell.md:61-63`
- Close-reason payloads and `onClose`/`onRemove` argument shapes are UNVERIFIED — no test asserts
  them. `specs/library/toast/parts/manager.md:116`

## Edge cases (rapid interactions, unmount, nesting)

- Rapid add→close→re-add reuses the same DOM node, clears `data-starting-style`, restores
  `--toast-height` (and clears swipe state / movement vars for swipe-dismissed toasts); retained
  roots reused for different toasts (index-keyed lists) must have all per-toast state cleared.
  `specs/library/toast/parts/root.md:73-76`, `specs/library/toast/parts/root.md:87`
- Manager/store no-ops and re-entrancy: unknown-id mutations leave state reference-identical;
  updaters are not invoked for missing/ending toasts; adds/closes from inside updaters are handled;
  double updates before a re-render keep the auto-dismiss timer; upserting an ending toast avoids a
  spurious `onRemove`; a dismissed promise toast stays dismissed after resolution.
  `specs/library/toast/parts/manager.md:124-135`, `specs/library/toast/parts/manager.md:128`
- Abandoned renders: a suspending `startTransition` prop change never syncs into the store (toast
  closes on the previously committed timeout), while same-commit prop+toast sync uses the fresh
  values; providers are isolated from each other; toasts can be added from inside an open Dialog.
  `specs/library/toast/parts/shell.md:67-70`, `specs/library/toast/parts/manager.md:135-136`
- Viewport listener rebinding stays exact across add/close-all/re-add cycles; exit animations
  defer collapse (data-expanded removed only after the closing toast unmounts); pointercancel
  ordering relative to mouseleave determines collapse.
  `specs/library/toast/parts/viewport.md:81-85`
- Leaf-part conditional rendering: missing content (undefined field, childless render prop with no
  content, render fn returning null) suppresses the element entirely, while `0` renders; array
  recursion rules for renderability are pinned by the `isRenderableNode` util tests.
  `specs/library/toast/parts/leaf-parts.md:74-85`
- Provider unmount cleanup is UNVERIFIED — no test unmounts the provider.
  `specs/library/toast/parts/manager.md:138`

## Shared harness dependencies

- `#test-utils` (mapped to `packages/react/test/index.ts`) re-exports `createRenderer`,
  `describeConformance`, and `isJSDOM` (via `@base-ui/utils/testUtils`); `createRenderer` wraps
  rendering in `act` and returns `{ user, rerender, setProps, clock }`; several suites enable fake
  timers via `clock.withFakeTimers()`.
  `specs/library/toast/parts/leaf-parts.md:87-91`, `specs/library/toast/parts/manager.md:142-148`
- `describeConformance` runs the "Base UI component API" suite — prop spreading, ref forwarding,
  render prop (function/element forms with ref/className merging), and className — implemented in
  `packages/react/test/describeConformance.tsx` and `packages/react/test/conformanceTests/*`; it is
  used by every part test except the manager/store/hook files.
  `specs/library/toast/parts/leaf-parts.md:91-95`, `specs/library/toast/parts/viewport.md:89-97`, `specs/library/toast/parts/manager.md:150`
- Browser-only tests are gated with `it.skipIf(isJSDOM)` (anchoring geometry, :focus-visible
  keyboard flows, frontmost-height measurement); the store suite uses raw `vi.useFakeTimers()`.
  `specs/library/toast/parts/root.md:93`, `specs/library/toast/parts/manager.md:144`
- Toast-local fixture `packages/react/src/toast/utils/test-utils.tsx` (unit-internal, not shared
  across components): `Button` adds a toast with `title`/`description`/`actionProps`; `List` maps
  `toasts` to `Toast.Root data-testid="root"` containing Title/Description/Close
  (`aria-label="close-press"`)/Action with `data-testid`s.
  `specs/library/toast/parts/leaf-parts.md:96`
- `@mui/internal-test-utils` supplies `screen`, `fireEvent`, `act`, `waitFor`, `user` event drivers.
  `specs/library/toast/parts/root.md:96`
