# Toast manager — implementation spec (createToastManager, store, useToastManager, resolvePromiseOptions)

Scope: implementation mining for the `toast` unit, batch `manager`. Evidence is the four batch source files — `packages/react/src/toast/createToastManager.ts`, `packages/react/src/toast/store.ts`, `packages/react/src/toast/useToastManager.ts`, `packages/react/src/toast/utils/resolvePromiseOptions.ts` — plus the shared utils their imports pull in. This spec explains WHY/HOW behind the behavior documented in `specs/library/toast/parts/manager.md` (referenced below as "behavior spec"); it does not restate it. Files from other batches (Provider, Root, Viewport) are cited by path only.

## State machine / hooks used

### Two-layer architecture: event emitter + store

The unit is split into two objects with deliberately different natures:

1. `createToastManager()` returns a **stateless event emitter**. It holds only a `Set` of listeners (`packages/react/src/toast/createToastManager.ts:13`) and an `emit` helper that fans an event out to all of them (`createToastManager.ts:15-17`). Every manager method (`add` at `createToastManager.ts:29-43`, `close` at `createToastManager.ts:45-50`, `update` at `createToastManager.ts:52-62`, `promise` at `createToastManager.ts:64-82`) does no state work at all — it just emits a `ToastManagerEvent` of shape `{ action: 'add' | 'close' | 'update' | 'promise', options: any }` (`createToastManager.ts:101-104`). If no one is subscribed, the event is dropped and `add` still returns a freshly generated id — the toast simply exists nowhere.
2. `ToastStore` (extends `ReactStore<State, {}, typeof selectors>`, `packages/react/src/toast/store.ts:99`) is the **actual state machine**, instantiated once per `Toast.Provider` (path only: `packages/react/src/toast/provider/ToastProvider.tsx`, which constructs it with the initial `{ timeout, limit, viewport: null, toasts: [], hovering: false, focused: false, isWindowFocused: true, prevFocusElement: null }`). The bridge between the two layers is the Provider's `toastManager[' subscribe'](...)` handler, which translates each emitted event into the corresponding store call (`store.promiseToast` / `store.updateToast` / `store.closeToast` / `store.addToast`) (path only: `packages/react/src/toast/provider/ToastProvider.tsx`). The `' subscribe'` key is intentionally package-private — the quoted name with a leading space keeps it off the public surface while still being reachable by the Provider (`createToastManager.ts:20-27`, interface at `createToastManager.ts:87`).

This split is why `createToastManager()` works outside React (behavior spec, "Public API surface"): the emitter is plain-JS, and all React-visible state lives in the per-provider store.

### ToastStore class shape

State fields (`store.ts:25-35`):

- `toasts: StoredToast[]` — newest-first array. `StoredToast = ToastObject & { updateKey: number }` (`store.ts:23`): inside the store `updateKey` is never missing, because `addToast` is the only entry point and always stamps it.
- `toastMetadata: Map<string, ToastMetadata>` — derived index rebuilt on every toasts write (see below).
- `timeout: number`, `limit: number` — provider config mirrored into state so store methods can read them synchronously without prop drilling.
- `hovering: boolean`, `focused: boolean`, `isWindowFocused: boolean` — the three "pause timers" inputs.
- `viewport: HTMLElement | null`, `prevFocusElement: HTMLElement | null` — viewport-owned DOM hooks (written only by the viewport batch; `store.ts` initializes/reads them).

`ToastMetadata` is `{ value: StoredToast; domIndex: number; visibleIndex: number; offsetY: number }` (`store.ts:37-42`), computed by `createToastMetadata` (`store.ts:46-68`): `domIndex` is the raw array index, `visibleIndex` is `-1` for `transitionStatus: 'ending'` toasts and otherwise the running count of non-ending toasts seen so far (`store.ts:52-64`), and `offsetY` accumulates `toast.height || 0` of every preceding toast (`store.ts:60`). The map is rebuilt in the constructor (`store.ts:104-113`) and on every `setToasts` (`store.ts:466-478`), so selectors over it are pure O(1) reads.

Mutation methods (public API of the class):

- `addToast(toast): string` (`store.ts:165-203`) — upsert-or-insert; detailed below.
- `updateToast(id, updates | updater)` (`store.ts:205-225`) — public guarded update.
- `updateToastInternal(id, updates, resetTimer = false, markUpdated = false)` (`store.ts:227-286`) — the raw write used by both public paths and by the root/viewport batches (e.g. height writes pass `markUpdated: false` so `updateKey` is not bumped; path only: `packages/react/src/toast/root/ToastRoot.tsx`).
- `closeToast(toastId?)` (`store.ts:288-320`) — ending transition for one or all.
- `removeToast(toastId, skipOnRemove = false)` (`store.ts:149-163`) — final removal; fires `onRemove` unless suppressed; called by the root batch after the exit animation (path only: `packages/react/src/toast/root/ToastRoot.tsx`) and internally by the upsert-over-ending path.
- `promiseToast(promise, options)` (`store.ts:322-362`) — promise lifecycle; below.
- `syncProviderProps(timeout, limit)` (`store.ts:119-138`) — provider prop sync; below.
- `pauseTimers()` / `resumeTimers()` (`store.ts:364-380`, `store.ts:382-396`) — timer pause machinery; below.
- `setViewport(viewport)` (`store.ts:115-117`) — ref callback handed to the viewport (path only: `packages/react/src/toast/viewport/ToastViewport.tsx`).
- `disposeEffect` (`store.ts:140-147`) — returns a cleanup that clears every `Timeout` and empties the map; mounted once by the Provider via `useOnMount` (path only: `packages/react/src/toast/provider/ToastProvider.tsx`) so unmount leaks no timers.
- `handleDocumentPointerDown` (`store.ts:402-416`) — document-level touch handler, subscribed (capture phase) by the viewport batch (path only: `packages/react/src/toast/viewport/ToastViewport.tsx`).

Selectors (module-level `selectors` object passed to the `ReactStore` constructor, `store.ts:86-97`, wired at `store.ts:104-113`):

- `toasts` — returns `state.toasts` (the array reference itself).
- `isEmpty` — `toasts.length === 0`.
- `toast(state, id)`, `toastIndex(state, id)`, `toastOffsetY(state, id)`, `toastVisibleIndex(state, id)` — all read the precomputed `toastMetadata` map, with fallbacks `undefined / -1 / 0 / -1` for unknown ids (`store.ts:89-92`).
- `focused`, `expanded` (`hovering || focused`), `expandedOrOutOfFocus` (`hovering || focused || !isWindowFocused`), `prevFocusElement` (`store.ts:93-96`). `expandedOrOutOfFocus` is the single predicate every timer-scheduling decision consults (`store.ts:198`, `store.ts:282`, `store.ts:420`).

### The toast state machine (transitionStatus / updateKey)

`transitionStatus` has exactly two values ever written by this unit: `'starting'`, stamped on every insert (`store.ts:187`, and again by the emitter for cosmetic parity at `createToastManager.ts:34`), and `'ending'`, stamped by `closeToast` (`store.ts:305-309`). There is no third value: a toast that has finished its enter animation simply has whatever status the last write left (root/viewport batches may write `transitionStatus: undefined` through `updateToastInternal`, which accepts it because `ToastInternalUpdateOptions` only omits `id` and `updateKey`, `store.ts:15-17`). `'ending'` is a terminal sink: every mutation path refuses to touch an ending toast — `updateToastInternal` returns early (`store.ts:242-244`), `updateToast` refuses to even run the user's updater function so callbacks don't fire for dying toasts (`store.ts:212-215`), `applyLimited` leaves ending toasts untouched (`store.ts:77-79`), and `onClose`/`onRemove` are suppressed for already-ending toasts (`store.ts:313-317`, `store.ts:149-158` with `skipOnRemove`). `closeToast` also zeroes `height` on the ending toast (`store.ts:307`) so collapse animations start from zero and the toast stops contributing `offsetY` (it would anyway, because the height is now 0).

`updateKey` is a pure re-render counter: `0` stamped by `addToast` (`store.ts:186`), incremented only when `updateToastInternal` is called with `markUpdated: true` (`store.ts:249-251`). Two callers use `markUpdated: true`: the public `updateToast` (`store.ts:219-224`) and the upsert branch of `addToast` (`store.ts:177`), which is why re-adding under an existing id bumps it (behavior spec, "State model"). Height/transition writes from the root batch use the default `markUpdated: false`.

### Id generation

`generateId('toast')` returns `toast-<random base36 4 chars>-<module-level counter>` (`packages/utils/src/generateId.ts:2-4`) — a monotonically increasing counter shared per JS realm, so ids are unique but not sortable by recency (ordering comes from array position, newest-first at `store.ts:190`). The id is resolved in two places: `createToastManager.add` pre-generates one if the caller omitted it (`createToastManager.ts:30`) so it can return a usable id before any store exists, and `store.addToast` resolves it again (`store.ts:167`) — if a caller-supplied `options.id` is present, the store uses it verbatim (`store.ts:169`), which is what makes the emitter's pre-generated id and user-supplied ids both land as the same key.

### Upsert semantics (`addToast`, `store.ts:165-203`)

- Caller-supplied `id` that already exists: if the existing toast is `'ending'`, the store first removes it with `skipOnRemove = true` (`store.ts:173-174`) — suppressing the old toast's `onRemove` (behavior spec, "Edge cases") — and then falls through to insert a fresh toast. Otherwise it strips `id` and `transitionStatus` from the incoming options (`store.ts:176`; the destructure-and-drop idiom — this is why a caller-supplied `transitionStatus` is ignored on upsert, behavior spec "State model") and routes through `updateToastInternal(id, updates, /* resetTimer */ true, /* markUpdated */ true)` (`store.ts:177`): the timer is fully refreshed and `updateKey` bumped, and the toast keeps its array position (no move to front).
- New id: build the `StoredToast` (`store.ts:183-188`), prepend to the array (`store.ts:190`), run `applyLimited` over the result (`store.ts:191`), then conditionally schedule the auto-dismiss timer — skipped for `type: 'loading'` and for `duration <= 0` where `duration = toast.timeout ?? state.timeout` (`store.ts:193-196`).
- Finally, if the viewport is currently expanded or out of focus, the new toast's freshly scheduled timer is paused immediately (`store.ts:198-200`), so a toast born under hover never runs while covered.

### Auto-dismiss timer machinery

Timer state lives in a private `Map<string, TimerInfo>` (`store.ts:100`). `TimerInfo` (`store.ts:518-527`) carries: `timeout` (a `Timeout` instance from `packages/utils/src/useTimeout.ts:9-41` — the class, not the `useTimeout` hook, because the store is not a component — or `undefined` when the timer was created while paused), `start` (timestamp of the last time the timeout actually began running), `delay` (the full configured duration), `remaining` (time left excluding paused time), and `callback` (the close closure).

`scheduleTimer(id, delay, callback)` (`store.ts:418-435`):

1. Stamps `start = Date.now()` (`store.ts:419`).
2. Consults `expandedOrOutOfFocus` (`store.ts:420`): if the viewport is hovered/focused/window-blurred, it does **not** create a running `Timeout` at all — the `TimerInfo` is stored with `timeout: undefined` and `remaining: delay` (`store.ts:428-434`). This is the mechanism behind "a rescheduled timer does not start while the viewport is expanded" (behavior spec, "State model"): resume later starts it with the full delay (`store.ts:388`).
3. Otherwise creates `Timeout.create()` and starts it with the delay; the fired callback first deletes the map entry (`handleTimerFired`, `store.ts:453-456`) **then** invokes `callback` (i.e. `closeToast`) (`store.ts:423-426`) — delete-before-fire means the subsequent `clearTimer` inside `closeToast` is a no-op and, if this was the last timer, the paused flag is auto-reset (see below).

`updateToastInternal`'s reschedule policy (`store.ts:256-285`) is the heart of auto-dismiss timing:

- Compute `nextTimeout = nextToast.timeout ?? state.timeout` and `prevTimeout = prevToast.timeout ?? state.timeout` (`store.ts:256-257`) — per-toast `timeout: 0` disables, `undefined` inherits the provider default.
- `timeoutUpdated = Object.hasOwn(updates, 'timeout')` (`store.ts:259`) — **key presence**, distinct from `timeoutChanged = prevTimeout !== nextTimeout` (`store.ts:265`). This distinction implements "updating with the same timeout value resets the timer" (behavior spec "State model"): the key being present forces a reschedule even when the value is identical.
- `shouldHaveTimer = status !== 'ending' && type !== 'loading' && nextTimeout > 0` (`store.ts:261-262`).
- If the toast should *not* have a timer but has one, clear it and stop (`store.ts:268-271`) — this is how `timeout: 0` updates and `type: 'loading'` toasts cancel dismissal.
- Otherwise reschedule (clear old + schedule new) only when: no timer exists, the value changed, the `timeout` key was present, the toast *was* loading, or the caller forced `resetTimer` (`store.ts:274-277`). The `wasLoading` term (`store.ts:266`) covers the loading→success/error transition where the loading toast legitimately had no timer. Each reschedule is followed by an immediate `pauseTimers()` if the viewport is still expanded/out-of-focus (`store.ts:282-284`).

Because a reschedule always clears-then-creates with the full delay from *now*, consecutive updates converge on the last one's deadline — two updates before a re-render leave exactly one live timer, never zero (behavior spec, "Edge cases").

### Pause / resume semantics

The pause flag `areTimersPaused` is a plain private boolean (`store.ts:102`), deliberately outside React state — pausing is a timing concern, not a render concern.

- `pauseTimers()` (`store.ts:364-380`): idempotent guard; for each timer that currently has a running `Timeout`, clears it and subtracts elapsed active time: `remaining = Math.max(remaining - (Date.now() - start), 0)` (`store.ts:377`). Because `start` is re-stamped on every resume (`store.ts:394`), the subtraction only ever consumes the most recent running window, so active time accumulates correctly across arbitrary pause/resume cycles and no cycle grants extra time (comments at `store.ts:374-376`; behavior spec "State model" items on remaining-time preservation and accumulation). Timers created while already paused have `timeout: undefined` and are skipped — their `remaining` is still the untouched full delay (`store.ts:370-372`).
- `resumeTimers()` (`store.ts:382-396`): idempotent guard; for each timer, if `remaining` has hit `0` it is reset to the full `delay` (`store.ts:388`) — this is the wall-clock-jump case (background tab): the elapsed-time subtraction already clamped `remaining` to 0 at pause time, and resume treats "no time left but not fired" as "restart the full delay" (behavior spec, "State model"). It lazily creates the `Timeout` for previously-paused entries (`store.ts:389`), starts it with `remaining`, and re-stamps `start` (`store.ts:390-394`).
- Flag lifecycle: `clearTimer` and `handleTimerFired` delete the map entry and then call `resetPausedStateIfNoTimersRemain` (`store.ts:445-456`), which flips `areTimersPaused` back to `false` once the map is empty (`store.ts:458-464`) — without this, a toast added after the last one closed could never be paused again (behavior spec, four paused-state-reset scenarios). `clearTimers()` (the close-all path, `store.ts:437-443`) resets the flag in the same breath.
- Who pauses: nothing in this batch subscribes to DOM events. The inputs are state fields written elsewhere: `hovering`/`focused` are set by the viewport batch's pointer/focus handlers (path only: `packages/react/src/toast/viewport/ToastViewport.tsx`, which sets `focused` only when the active element is focus-visible), `isWindowFocused` by its window blur/focus handlers, and the store's own touch-outside escape hatch `handleDocumentPointerDown` (`store.ts:402-416`) resumes timers and clears `hovering`/`focused` on any touch pointerdown outside the viewport (shadow-DOM-safe `getTarget`/`contains` from `packages/react/src/floating-ui-react/utils.ts`, imported at `store.ts:12`). `isFocusVisible` (`packages/react/src/toast/utils/focusVisible.ts`, imported at `store.ts:13`) is consulted by this batch only in focus management, not timer pausing.

### Limit recompute

`applyLimited(toasts, limit)` (`store.ts:74-84`) is the single source of truth for the `limited` flag. Callers always pass newest-first; `activeIndex` counts only non-ending toasts; a toast is `limited` when its active index is `>= limit` (`store.ts:80-81`) — so the *oldest* toasts past the limit are flagged. Ending toasts are passed through untouched so their flags can't resurrect. Crucially, when a toast's flag doesn't change, the original object reference is returned (`store.ts:82`), keeping object identity stable for subscribers (see snapshot rules below). `applyLimited` is re-run on every structural mutation — insert (`store.ts:191`) and close (`store.ts:310`, which is why closing a toast un-marks previously limited ones) — and `syncProviderProps` re-runs it when the provider `limit` prop changes (`store.ts:131-135`), also rebuilding the metadata map. `syncProviderProps` early-returns when neither prop changed (`store.ts:122-124`) so unrelated provider re-renders don't churn state.

### Promise lifecycle (`promiseToast`, `store.ts:322-362`)

1. `resolvePromiseOptions(options.loading)` resolves the loading options (see below) and the toast is inserted with `type: 'loading'` (`store.ts:326-331`) — which `addToast` deliberately never schedules a timer for (`store.ts:194`).
2. The input promise is chained (not consumed): `.then` resolves `options.success` with the resolved value and `updateToast`s the same id to `type: 'success'`; `.catch` resolves `options.error` with the error, updates to `type: 'error'`, and **re-rejects** (`return Promise.reject(error)`, `store.ts:352`) so the caller's own `.catch` still sees the failure (`store.ts:333-353`).
3. The success/error update payloads explicitly include `timeout: successOptions.timeout` (`store.ts:339`, `store.ts:349`) even when that is `undefined`. Two effects fall out of the reschedule policy above: `Object.hasOwn(updates, 'timeout')` is always true, forcing the reschedule branch; and because the spread overwrites the loading toast's own `timeout` with `undefined`, `nextTimeout` falls back to the **provider default** (`store.ts:256`) rather than the loading toast's timeout — this is exactly the "loading `timeout: 0` is not inherited by the success state" behavior (behavior spec, "State model"). An explicit `timeout: 0` in the success/error options still disables auto-dismiss because `shouldHaveTimer` requires `nextTimeout > 0` (`store.ts:261-262`).
4. `handledPromise` handoff: the store calls `options.setPromise(handledPromise)` only when the options object has the key at all (`{}.hasOwnProperty.call`, `store.ts:357-359`) — a duck-typed private channel. `createToastManager.promise` seeds `handledPromise` with the raw input promise and passes a `setPromise` closure that reassigns it (`createToastManager.ts:68`, `createToastManager.ts:75-77`); since `emit` is synchronous, by the time `promise()` returns (`createToastManager.ts:81`) the variable holds the handled chain *if a provider was subscribed at call time* — otherwise the raw input promise is returned and the toast never materializes. `promiseToast` returns the handled chain either way, so Provider-internal callers always get the handled chain.

`resolvePromiseOptions` (`packages/react/src/toast/utils/resolvePromiseOptions.ts:3-22`) normalizes the three accepted shapes of each promise-state option: a string becomes `{ description: options }` (`resolvePromiseOptions.ts:10-14`); a function is invoked with the settled value (resolved result or rejection error) and its string return is likewise wrapped as `{ description }` (`resolvePromiseOptions.ts:16-19`); an object passes through as-is (`resolvePromiseOptions.ts:21`).

### Observable / subscription model (how state reaches React)

Base layer: `Store<State>` (`packages/utils/src/store/Store.ts`) keeps `state` public and synchronous (`Store.ts:27`), a listener `Set` (`Store.ts:36`), and three mutators that all funnel into `setState` (`Store.ts:65-82`): `update(changes)` diffs each key with `Object.is` and no-ops if nothing differs (`Store.ts:91-98`), `set(key, value)` does the same for one key (`Store.ts:106-110`), and both write a fresh object `{ ...this.state, ...changes }` — so any real change produces a **new state reference**, and no-op writes preserve the old one. `setState` itself short-circuits identical references (`Store.ts:66-68`) and has a re-entrancy guard (`updateTick`, `Store.ts:71-80`) so a listener that calls back into the store doesn't double-notify — this is what makes "the updater may have called back into the store, so the internal update reads the current state again" (`store.ts:217-224`) safe.

React layer: `ReactStore.useState(key, ...args)` (`packages/utils/src/store/ReactStore.ts:171-179`) delegates to `useStore(store, selector, ...)` (`packages/utils/src/store/useStore.ts`), which has three implementations behind a React-version switch (`useStore.ts:12-13`):

- React 19 standalone: `useSyncExternalStore(store.subscribe, getSelection, getSelection)` where `getSelection` is `React.useCallback(() => selector(store.getSnapshot(), ...))` (`useStore.ts:47-60`).
- React 19 "fast" path (registered via `fastHooks` when components opt in): a shared `getSnapshot` returns a monotonically increasing tick that is bumped only when *any* registered hook's selected value changed by `Object.is` (`useStore.ts:78-127`), plus per-hook value caching in `useStoreFast` (`useStore.ts:129-178`).
- Legacy (React <19): `useSyncExternalStoreWithSelector(store.subscribe, store.getSnapshot, store.getSnapshot, wrappedSelector)` (`useStore.ts:180-193`).

Snapshot identity rules that decide re-renders:

- The store-level snapshot is the whole `state` object; selector results are compared with `Object.is` in all three paths.
- `useToastManager` subscribes only to `toasts` (`packages/react/src/toast/useToastManager.ts:12`) — the array reference. Every toasts write goes through `setToasts`, which always passes a new array plus a rebuilt metadata map (`store.ts:466-478`), so **every** add/update/close re-renders every `useToastManager()` consumer, regardless of which toast changed.
- Re-render *within* that is minimized by object identity: untouched toast objects keep their references (map-based writes replace only the matching id, `store.ts:254`; `applyLimited` returns the same reference when a flag is unchanged, `store.ts:82`; `removeToast` splices, `store.ts:160-162`), so memoized `Toast.Root` children keyed by `toast.id` skip re-render when only a sibling changed.
- `expanded`-style boolean selectors (`store.ts:93-95`) let the viewport/root subscribe to cheap primitives instead of the array.

### Hooks used, by call site

In-batch direct hook usage is minimal because the heavy lifting is class-based:

- `React.useMemo` — `packages/react/src/toast/useToastManager.ts:14-23`: memoizes the `{ toasts, add, close, update, promise }` return on `[toasts, store]`. The methods are the store's own arrow-function-bound methods (`store.addToast`, `store.closeToast`, `store.updateToast`, `store.promiseToast` — class-property arrow functions at `store.ts:165`, `store.ts:288`, `store.ts:205`, `store.ts:322`), so they are referentially stable across renders and safe to pass as event-handler props. `useToastManager` therefore has no logic of its own: it is a thin context read + one subscription.
- `React.useContext` — via `useToastProviderContext()` (`useToastManager.ts:10` → `packages/react/src/toast/provider/ToastProviderContext.ts:9-15`, context read at `ToastProviderContext.ts:10`).
- `useSyncExternalStore` (from `use-sync-external-store/shim` and `/with-selector`) — not called in toast files; reached through `ReactStore.useState` (`packages/utils/src/store/ReactStore.ts:178`) at `useToastManager.ts:12` and by the viewport/root subscribers; implementation at `packages/utils/src/store/useStore.ts:59`, `useStore.ts:124`, `useStore.ts:187`.
- `useIsoLayoutEffect` — used by the `ReactStore` helpers `useSyncedValue` / `useSyncedValueWithCleanup` / `useSyncedValues` (`packages/utils/src/store/ReactStore.ts:49`, `ReactStore.ts:69`, `ReactStore.ts:110`); no toast batch file calls those helpers or the hook directly (the Provider uses it directly for `syncProviderProps`; path only: `packages/react/src/toast/provider/ToastProvider.tsx`).
- `useStableCallback` — **not used anywhere in this unit.** It appears only inside `ReactStore.useContextCallback` (`packages/utils/src/store/ReactStore.ts:188-195`), and the toast store constructs its `ReactStore` base with an empty context type (`store.ts:99`), so no context callbacks are registered. The store instead uses class-property arrow functions for stability (see above).
- `useRefWithInit` / `useOnMount` — Provider-side only (store creation and `disposeEffect` mounting; path only: `packages/react/src/toast/provider/ToastProvider.tsx`).

## Context providers/consumers

- The React context is `ToastContext = React.createContext<ToastContext | undefined>(undefined)` where the context value **is the `ToastStore` instance itself** (`packages/react/src/toast/provider/ToastProviderContext.ts:5-7`). There is no separate "manager context": the store is the single source of truth, and `ToastStore extends ReactStore` with an empty context parameter (`store.ts:99`) means the store's non-reactive `context` bag (`packages/utils/src/store/ReactStore.ts:35`) is unused.
- Producer: `Toast.Provider` provides the store and renders a null child that syncs `timeout`/`limit` into it via `syncProviderProps` (path only: `packages/react/src/toast/provider/ToastProvider.tsx`).
- Consumer: `useToastManager()` is the only sanctioned read path for app code (`useToastManager.ts:9-24`); it throws a namespaced error when used outside a Provider (`ToastProviderContext.ts:11-13`).
- The `createToastManager` instance crosses **no** context boundary — it reaches the store only as a prop on `Toast.Provider` (behavior spec, "Public API surface"), where the Provider subscribes to it in an effect and forwards events into the store (path only: `packages/react/src/toast/provider/ToastProvider.tsx`). Multiple providers are isolated for free because each owns its own store instance and its own subscription (behavior spec, "Edge cases").
- The one state field consumed across batches through this same context: viewport/root components subscribe via `store.useState(...)` and read `store.state` directly in event handlers (the base-class doc encourages reading `state` without subscribing in effects/handlers, `packages/utils/src/store/Store.ts:22-23`).

## DOM/portal strategy and why

None — this batch is pure logic/headless. `createToastManager.ts` imports no React at all; `store.ts` never touches `document`/`window` globals (the only DOM access is `ownerDocument(this.state.viewport)` for focus management, `store.ts:481`, plus the shadow-DOM-safe `activeElement`/`contains`/`getTarget` helpers from `packages/react/src/floating-ui-react/utils.ts` imported at `store.ts:12`); `useToastManager.ts` renders nothing. This matters for three reasons:

1. The manager/store/hook layer is fully testable and drivable with no DOM — the behavior spec's store-level tests run the entire state machine including timers with no viewport mounted, and toasts are consumable (`toasts` array) even without a `Toast.Viewport` (behavior spec, "DOM structure & portal behavior").
2. All DOM concerns are opt-in and viewport-scoped: `viewport` starts as `null` (`store.ts:33`), `setViewport` is only called by the viewport's ref (path only: `packages/react/src/toast/viewport/ToastViewport.tsx`), and every DOM-touching code path (`handleDocumentPointerDown`, `handleFocusManagement`) guards on the viewport being set (`store.ts:407-408`, `store.ts:482-488`).
3. Where toasts render is entirely the consumer's decision (map `toasts` yourself, per behavior spec), keeping the portal/positioning logic in the portal/positioner/viewport batches rather than in state management.

## Dependencies on other Base UI internals

Imports made by the batch files:

- `@base_ui/utils/store` → `ReactStore` (`store.ts:1`): base class providing `state`/`subscribe`/`set`/`update`/`setState`/`notifyAll` and the `useState`/`select`/`observe` React bridge (`packages/utils/src/store/Store.ts`, `packages/utils/src/store/ReactStore.ts`, `packages/utils/src/store/useStore.ts`). Not inlined here; the subscription model is described above only as far as toast's snapshot semantics depend on it.
- `@base_ui/utils/generateId` (`store.ts:2`, `createToastManager.ts:1`): module-level counter id generator (`packages/utils/src/generateId.ts`).
- `@base_ui/utils/owner` → `ownerDocument` (`store.ts:3`): realm-safe document lookup used in focus management (`store.ts:481`).
- `@base_ui/utils/useTimeout` → the `Timeout` **class** (`store.ts:4`): `Timeout.create()`/`.start()`/`.clear()` wrap raw `setTimeout`/`clearTimeout` (`packages/utils/src/useTimeout.ts:9-41`). The store uses the class (it is not a component); the `useTimeout` hook itself is not used in this batch.
- `./utils/resolvePromiseOptions` (`store.ts:11`) — in-batch.
- `../floating-ui-react/utils` → `activeElement`, `contains`, `getTarget` (`store.ts:12`, used at `store.ts:407-408` and `store.ts:481-484`): shadow-DOM-safe traversal/targeting utilities (`packages/react/src/floating-ui-react/utils.ts`).
- `./utils/focusVisible` → `isFocusVisible` (`store.ts:13`, used at `store.ts:485`): gates whether closing a toast moves focus; internals owned by the viewport batch's spec.
- `./provider/ToastProviderContext` (`useToastManager.ts:3`) and type-only `./positioner/ToastPositioner` (`useToastManager.ts:4`, for `ToastManagerPositionerProps` extending `ToastPositionerProps` minus `anchor`/`toast`, `useToastManager.ts:99-107`) — same unit, other batches.
- Provider-side (encountered, not in batch): `@base_ui/utils/useOnMount`, `@base_ui/utils/useRefWithInit`, `@base_ui/utils/useIsoLayoutEffect` (path only: `packages/react/src/toast/provider/ToastProvider.tsx`).

Notable non-dependencies: no `useStableCallback`, no `useTimeout` hook, no floating-ui positioning code, and no CSS/styling imports in this batch.

## Anything in source not explained by any test

Cross-checked against the behavior spec (`specs/library/toast/parts/manager.md`), which is grounded only in `createToastManager.test.tsx`, `store.test.ts`, and `useToastManager.test.tsx`. The following source behavior has **no coverage in those tests** and is the gap list for the golden-fixture stage:

1. **Focus management machinery is entirely untested at this level.** `handleFocusManagement` (`store.ts:480-515`) — the active-element/focus-visible guard, the forward-then-backward scan for the nearest non-ending toast, focusing its `ref.current` (`store.ts:510-511`), and the `restoreFocusToPrevElement` fallback (`store.ts:398-400`, `store.ts:490-493`) — is asserted nowhere: the behavior spec's "Focus management" section is explicitly N/A with only state-shape evidence. The `prevFocusElement` state field is never even written inside this batch (the viewport batch writes it; path only: `packages/react/src/toast/viewport/ToastViewport.tsx`), so within this batch's evidence the field is write-only dead state.
2. **`handleDocumentPointerDown` (`store.ts:402-416`) — the touch-outside-viewport escape hatch — appears in no manager-level test.** The behavior spec's pause/resume evidence uses only `pauseTimers`/`resumeTimers`/`set('hovering', ...)` and `mouseEnter`/`mouseLeave`; touch-pointer resume and the "explicit touch activity outside the viewport ends paused interaction state even when window focus is unchanged" behavior (`store.ts:412-415`) are untested here. Its listener wiring lives in the viewport batch.
3. **The `isWindowFocused` pause input is untested at this level.** `expandedOrOutOfFocus` includes `!isWindowFocused` (`store.ts:95`), so window blur alone pauses timers — but the behavior spec's store tests exercise only `hovering` (and the direct `pauseTimers` API). Nothing in this batch writes the field (initialized `true` by the Provider; written by viewport handlers, path only), and its effect on scheduling (`store.ts:420`, `store.ts:198-200`, `store.ts:282-284`) is only implicitly covered via hovering.
4. **`disposeEffect` unmount cleanup (`store.ts:140-147`) is untested** — consistent with the behavior spec's UNVERIFIED flag on provider unmount behavior: no test unmounts a provider and asserts timers were cleared.
5. **`createToastManager.promise()` with no subscribed provider returns the raw input promise** (`createToastManager.ts:68`, `createToastManager.ts:81`): the `handledPromise` reassignment happens only if the synchronous `emit` reaches a Provider bridge. All behavior-spec promise tests run with a mounted provider, so the "unhandled" return path and the fact that `add`/`close`/`update`/`promise` events are silently dropped with no listener are unobserved. Relatedly, the exact resolved value of the handled chain is UNVERIFIED in the behavior spec (its "Events" section).
6. **`setViewport` (`store.ts:115-117`) and the `viewport` state field have no manager-level test** — store tests never set a viewport, so `handleFocusManagement`'s viewport guard and the pointer-down `contains` check are exercised only as early-returns. Owned by the viewport batch.
7. **Unused-here selectors have no manager-level assertions:** `selectors.isEmpty`, `selectors.expanded`, `selectors.focused`, and `selectors.prevFocusElement` (`store.ts:88`, `store.ts:93-96`) are consumed only by the viewport/root batches; the behavior spec's selector evidence covers `toast`/`toastIndex`/`toastOffsetY`/`toastVisibleIndex` only.
8. **`ToastObject.positionerProps` (`useToastManager.ts:92`) is never mentioned in the behavior spec** — no test in its three files adds a toast with `positionerProps` or asserts anchored-toast behavior; it exists in the type and flows through the store untouched.
9. **`ToastManagerEvent.options` is typed `any` (`createToastManager.ts:103`)** — the event payload contract between emitter and Provider bridge is untyped and untested for shape drift; the bridge's `action` dispatch order (promise → update → close → fallthrough add, path only: `packages/react/src/toast/provider/ToastProvider.tsx`) is also outside the behavior spec's evidence (the `close` fallthrough means a `close` event always reaches `store.closeToast(id)` even when `id` is `undefined`, which is how close-all works through the emitter).
10. **Minor untested niceties:** `selectors.toastOffsetY`'s `?? 0` fallback for unknown ids (`store.ts:91`) and the `ToastMetadata` invariant that ending toasts still contribute their (pre-zeroed, thus 0) height to `offsetY` accumulation order (`store.ts:60-64`) are covered only implicitly by the metadata-invariant tests the behavior spec cites; and `ReactStore.notifyAll` (`packages/utils/src/store/Store.ts:115-118`) exists on the base class but is never called by any toast file.
