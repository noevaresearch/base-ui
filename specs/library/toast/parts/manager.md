# Toast manager — behavior spec (unit: `toast`, batch: `manager`)

Scope: behavior mined from `createToastManager.test.tsx`, `store.test.ts`, and `useToastManager.test.tsx` only. Every claim is limited to what those tests actually assert; anything else is marked UNVERIFIED.

## Public API surface (props, parts, subcomponents)

- `Toast.createToastManager()` is a standalone factory that creates a toast manager outside React; it is instantiated at plain test scope with no component mounted. `packages/react/src/toast/createToastManager.test.tsx:16`
- The manager is fully drivable outside React: `toastManager.add({ title: 'title' })` is called with no provider rendered and returns a value of type `string` (a toast id). `packages/react/src/toast/createToastManager.test.tsx:53-61`
- Manager method `add(options)` exists. `packages/react/src/toast/createToastManager.test.tsx:19-21`
- Manager method `update(id, patch)` exists. `packages/react/src/toast/createToastManager.test.tsx:336-338`
- Manager method `close(id?)` exists: called with an id it closes one toast, called with no argument it closes all toasts. `packages/react/src/toast/createToastManager.test.tsx:711-713` and `packages/react/src/toast/createToastManager.test.tsx:753-755`
- Manager method `promise(inputPromise, { loading, success, error })` exists. `packages/react/src/toast/createToastManager.test.tsx:135-146`
- `createToastManager` accepts a generic type parameter for custom `data` (`Toast.createToastManager<{ count: number }>()`). `packages/react/src/toast/createToastManager.test.tsx:373`
- `Toast.Provider` accepts a `toastManager` prop to adopt an externally created manager. `packages/react/src/toast/createToastManager.test.tsx:33`
- `Toast.Provider` also works with no `toastManager` prop (default internal manager). `packages/react/src/toast/useToastManager.test.tsx:38-44`
- `Toast.Provider` accepts a `timeout` prop (number, ms). `packages/react/src/toast/useToastManager.test.tsx:1303` and `packages/react/src/toast/useToastManager.test.tsx:1797`
- `Toast.Provider` accepts a `limit` prop (number). `packages/react/src/toast/useToastManager.test.tsx:1855` and `packages/react/src/toast/useToastManager.test.tsx:1944`
- `useToastManager()` is exported and callable inside components under a `Toast.Provider`. `packages/react/src/toast/useToastManager.test.tsx:7` and `packages/react/src/toast/useToastManager.test.tsx:23`
- `useToastManager()` returns `{ toasts, add, close, update, promise }`: `toasts` is an array mapped to `Toast.Root` elements, `add`/`update`/`close`/`promise` mirror the manager methods. `packages/react/src/toast/useToastManager.test.tsx:56-92` and `packages/react/src/toast/useToastManager.test.tsx:745-755`
- `ToastStore` is an exported class; it is constructed with a state object of shape `{ toasts, timeout, limit, hovering, focused, isWindowFocused, viewport, prevFocusElement }`. `packages/react/src/toast/store.test.ts:2` and `packages/react/src/toast/store.test.ts:5-17`
- Store state is publicly readable (`store.state`, `store.state.toasts`). `packages/react/src/toast/store.test.ts:23` and `packages/react/src/toast/store.test.ts:88`
- Store methods proven: `addToast` (`packages/react/src/toast/store.test.ts:62`), `updateToast` (`packages/react/src/toast/store.test.ts:102`), `updateToastInternal` (`packages/react/src/toast/store.test.ts:49`), `closeToast` (`packages/react/src/toast/store.test.ts:53`), `removeToast(id, flag?)` (`packages/react/src/toast/store.test.ts:58`), `promiseToast` (`packages/react/src/toast/store.test.ts:206-210`), `syncProviderProps(timeout, limit)` (`packages/react/src/toast/store.test.ts:234`), `pauseTimers` / `resumeTimers` (`packages/react/src/toast/store.test.ts:257` and `packages/react/src/toast/store.test.ts:345`), and `set(key, value)` (`packages/react/src/toast/store.test.ts:333` and `packages/react/src/toast/store.test.ts:344`).
- Selectors proven: `selectors.toast(state, id)` returns the toast object itself (identity), `selectors.toastIndex(state, id)` returns its array index, `selectors.toastOffsetY(state, id)` returns the accumulated height of preceding toasts, `selectors.toastVisibleIndex(state, id)` returns `-1` for ending toasts and the running count of non-ending toasts otherwise. `packages/react/src/toast/store.test.ts:26-35`
- `add` options proven by tests: `title` (`packages/react/src/toast/createToastManager.test.tsx:19-21`), `description` (`packages/react/src/toast/useToastManager.test.tsx:486-491`), `id` (`packages/react/src/toast/createToastManager.test.tsx:74-78`), `timeout` (`packages/react/src/toast/createToastManager.test.tsx:77`), `type` (`packages/react/src/toast/useToastManager.test.tsx:527`), `onClose` (`packages/react/src/toast/useToastManager.test.tsx:568-571`), `onRemove` (`packages/react/src/toast/useToastManager.test.tsx:193-198`), `priority` (`packages/react/src/toast/useToastManager.test.tsx:707`), `data` (store level: `packages/react/src/toast/store.test.ts:99`), and `actionProps` (passed by the shared `Button` helper: `packages/react/src/toast/utils/test-utils.tsx:15-19`).
- `Toast.Root`, `Toast.Title`, `Toast.Description`, `Toast.Close`, and `Toast.Action` are the subcomponents exercised; `Toast.Root` receives the toast object via a `toast` prop and is keyed by `toast.id`. `packages/react/src/toast/utils/test-utils.tsx:30-39`
- UNVERIFIED — inferred from `packages/react/src/toast/utils/test-utils.tsx:15-19`, no test asserts this: `Toast.Action` rendering driven by `actionProps` (no assertion in the three test files checks the action element).

## State model (controlled/uncontrolled, defaults, transitions)

- Controlled/external-manager mode: a manager created via `Toast.createToastManager()` and passed to `Toast.Provider toastManager={...}` drives the toasts rendered under that provider. `packages/react/src/toast/createToastManager.test.tsx:32-39`
- Uncontrolled/default mode: `<Toast.Provider>` without `toastManager` works; `useToastManager()` operates on the provider's internal manager. `packages/react/src/toast/useToastManager.test.tsx:37-44`
- Default auto-dismiss timeout is 5000 ms: a toast added with only `{ title }` is gone after `clock.tickAsync(5000)`. `packages/react/src/toast/createToastManager.test.tsx:48-50` and `packages/react/src/toast/useToastManager.test.tsx:51-53`
- Per-toast `timeout` overrides the default: `timeout: 1000` dismisses after 1000 ms. `packages/react/src/toast/useToastManager.test.tsx:411-434`
- `timeout: 0` means never auto-dismiss: the toast is still present after 10000 ms. `packages/react/src/toast/useToastManager.test.tsx:1369-1415`
- Changing the provider `timeout` prop applies to toasts added afterwards (a toast added after `setProps({ timeout: 1000 })` dismisses after 1000 ms). `packages/react/src/toast/useToastManager.test.tsx:1794-1822`
- `add` returns the toast id; when a user-supplied `id` is given, the returned id equals it (`'save'`). `packages/react/src/toast/createToastManager.test.tsx:117-118`
- Adding again with an existing id upserts in place: the title changes, the number of rendered roots stays 1, and the returned ids are equal. `packages/react/src/toast/createToastManager.test.tsx:108-120` and `packages/react/src/toast/useToastManager.test.tsx:170-179`
- A newly added toast has `transitionStatus: 'starting'`. `packages/react/src/toast/useToastManager.test.tsx:362-364`
- `transitionStatus` supplied by the caller in `add` options is ignored when upserting an existing toast (toast stays `'starting'`). `packages/react/src/toast/useToastManager.test.tsx:336-368`
- `updateKey` starts at `0` on the first add. `packages/react/src/toast/useToastManager.test.tsx:403-404` (the store helper stamps `updateKey: 0` on the way in, mirroring `addToast`: `packages/react/src/toast/store.test.ts:7-8`)
- Re-adding with the same id increments `updateKey` to `1`. `packages/react/src/toast/useToastManager.test.tsx:406-407`
- `update` increments `updateKey` to `1`. `packages/react/src/toast/useToastManager.test.tsx:1585-1586` and `packages/react/src/toast/store.test.ts:112-123`
- `update(id, patch)` applies a partial patch (a patch containing only `title` changes only the title rendering). `packages/react/src/toast/createToastManager.test.tsx:336-338` and `packages/react/src/toast/createToastManager.test.tsx:369`
- `update(id, fn)` derives the patch from the current toast: the updater is called exactly once and receives the previous toast object (its `title` and `data` are readable). `packages/react/src/toast/createToastManager.test.tsx:372-417`
- Updating with the same `timeout` value resets the auto-dismiss timer (dismissal happens a full timeout after the update, not at the original deadline). `packages/react/src/toast/createToastManager.test.tsx:419-472` and `packages/react/src/toast/createToastManager.test.tsx:474-538`
- Updating `timeout` from `0` to a positive value starts auto-dismissal. `packages/react/src/toast/createToastManager.test.tsx:540-587` and `packages/react/src/toast/useToastManager.test.tsx:1589-1633`
- Updating a `type: 'loading'` toast to a non-loading type with a `timeout` schedules the dismissal timer. `packages/react/src/toast/createToastManager.test.tsx:589-640` and `packages/react/src/toast/useToastManager.test.tsx:1635-1680`
- Two `update` calls issued before a re-render do not clear the scheduled auto-dismiss timer. `packages/react/src/toast/createToastManager.test.tsx:642-696`
- `close(id)` removes the toast from the rendered output. `packages/react/src/toast/createToastManager.test.tsx:737-743`
- `close()` with no argument closes all toasts (5 added, 0 rendered afterwards). `packages/react/src/toast/createToastManager.test.tsx:780-787` and `packages/react/src/toast/useToastManager.test.tsx:1776-1785`
- Store transition: `closeToast` sets `transitionStatus: 'ending'`, then `removeToast` removes the toast from state. `packages/react/src/toast/store.test.ts:53-58`
- Store `data` semantics: `updateToast` with `data` replaces the previous `data` object wholesale (identity is preserved, not merged); a patch without `data` keeps the previous value; `{ data: undefined }` clears it. `packages/react/src/toast/store.test.ts:98-110`
- A function passed as `data` is stored as a value, not invoked as an updater. `packages/react/src/toast/store.test.ts:137-143`
- Re-adding a toast under an existing id replaces `data` wholesale; re-adding without `data` preserves the previous `data`. `packages/react/src/toast/store.test.ts:180-199`
- Promise toasts: `promise()` first renders the `loading` state (a string option becomes the rendered description), then updates to `success` on resolve or `error` on reject. `packages/react/src/toast/createToastManager.test.tsx:166-174` and `packages/react/src/toast/createToastManager.test.tsx:268-272`
- Promise state options accept a string, an object (e.g. `{ title, description }`), or a function: `success: (data) => string` receives the resolved value, `error: (error) => string` receives the rejection error, and `success: (data) => ({ title, description, timeout })` returns full toast options that are applied (including the returned `timeout`). `packages/react/src/toast/useToastManager.test.tsx:847-887`, `packages/react/src/toast/useToastManager.test.tsx:889-938`, and `packages/react/src/toast/useToastManager.test.tsx:940-982`
- Promise toasts auto-dismiss after the default 5000 ms following resolution or rejection. `packages/react/src/toast/useToastManager.test.tsx:1115-1121` and `packages/react/src/toast/useToastManager.test.tsx:1165-1170`
- Promise `success`/`error` options may specify their own `timeout` (2000 ms and 3000 ms respectively are honored). `packages/react/src/toast/useToastManager.test.tsx:1173-1222` and `packages/react/src/toast/useToastManager.test.tsx:1224-1275`
- When no promise-state timeout is specified, the provider `timeout` is used (`<Toast.Provider timeout={1000}>` → dismissed 1000 ms after resolve). `packages/react/src/toast/useToastManager.test.tsx:1277-1320`
- A `loading` toast's `timeout` is not inherited by the success state: `loading: { timeout: 0 }` with a timeout-less `success` still auto-dismisses after the default 5000 ms. `packages/react/src/toast/createToastManager.test.tsx:177-229` and `packages/react/src/toast/useToastManager.test.tsx:1322-1367`
- `timeout: 0` on a promise state disables auto-dismissal for that state. `packages/react/src/toast/useToastManager.test.tsx:1369-1415`
- Hovering the toast pauses its timer; the toast survives 5000 ms while hovered, and dismisses 2000 ms after mouse leave (3 s timeout, 1 s already elapsed before hover — remaining time is preserved). `packages/react/src/toast/useToastManager.test.tsx:1417-1472`
- Store `limited` flag: with `syncProviderProps(0, limit)` and toasts ordered newest-first, only toasts beyond the limit (i.e. older ones) get `limited: true`; raising the limit clears all flags. `packages/react/src/toast/store.test.ts:229-243`
- Provider `limit` in React: with `limit={2}`, adding a 3rd toast marks the oldest with `data-limited`. `packages/react/src/toast/useToastManager.test.tsx:1853-1876`
- Closing a toast un-marks previously limited toasts once under the limit. `packages/react/src/toast/useToastManager.test.tsx:1878-1905`
- Upserting a limited toast (same id) preserves its limited state. `packages/react/src/toast/useToastManager.test.tsx:1907-1963`
- Changing the provider `limit` prop recomputes limited flags in both directions. `packages/react/src/toast/useToastManager.test.tsx:1965-1995`
- Timer pausing is exposed at store level: `pauseTimers()`/`resumeTimers()` plus `set('hovering', true|false)`. `packages/react/src/toast/store.test.ts:333-345`
- Remaining timeout is preserved across pause/resume cycles (2000 ms consumed of 5000 ms → still 3000 ms remain). `packages/react/src/toast/store.test.ts:350-370`
- If the wall clock jumped past the timeout before pausing (background-tab simulation), resume restarts the full delay. `packages/react/src/toast/store.test.ts:372-389`
- Active time accumulates across hover cycles (3 × 40 ms of running time reaches a 100 ms timeout). `packages/react/src/toast/store.test.ts:407-426`
- The store clears its internal paused state when the last toast closes, so a toast added afterwards can be paused again (proven across four scenarios: last toast closed, all closed via `closeToast()`, last active toast closed while ending toasts remain, last timed toast closed while an untimed `type: 'loading'` toast remains). `packages/react/src/toast/store.test.ts:251-271`, `packages/react/src/toast/store.test.ts:273-288`, `packages/react/src/toast/store.test.ts:290-306`, and `packages/react/src/toast/store.test.ts:308-323`
- The paused state is also re-clearable after the last timed toast becomes untimed via `updateToastInternal('a', { timeout: 0 })`. `packages/react/src/toast/store.test.ts:391-405`
- A rescheduled timer (via `updateToast` with `timeout`) does not start while the viewport is expanded (hovering) and runs once collapsed. `packages/react/src/toast/store.test.ts:325-348`

## Keyboard interactions

N/A — no test in `createToastManager.test.tsx`, `store.test.ts`, or `useToastManager.test.tsx` exercises keyboard input against the manager, store, or hook.

## Focus management

N/A — no test asserts focus movement or restoration. The only evidence is state-shape: the store state carries `focused: false` and `prevFocusElement: null` fields. `packages/react/src/toast/store.test.ts:12` and `packages/react/src/toast/store.test.ts:15`

## Accessibility (roles, aria-*, id linking)

- A toast added with `priority: 'high'` renders its root with `role="alertdialog"`. `packages/react/src/toast/useToastManager.test.tsx:725-728`
- The high-priority root also has `aria-modal="false"`. `packages/react/src/toast/useToastManager.test.tsx:728`
- A `role="alert"` element is present for the high-priority toast and has `aria-atomic="true"`. `packages/react/src/toast/useToastManager.test.tsx:729-730`
- Closing the high-priority toast removes the `alert` role from the document. `packages/react/src/toast/useToastManager.test.tsx:732-735`
- The shared `List` helper renders `Toast.Close` with `aria-label="close-press"`; tests locate it via `getByLabelText('close-press')` and click it to dismiss. `packages/react/src/toast/utils/test-utils.tsx:35`, `packages/react/src/toast/createToastManager.test.tsx:314`, and `packages/react/src/toast/useToastManager.test.tsx:1067`
- A high-priority toast added from inside an open dialog renders with `aria-hidden="true"` on its root while a `role="alert"` remains present. `packages/react/src/toast/useToastManager.test.tsx:2103-2108`
- A regular toast added from inside an open dialog renders into the viewport and its title/description content is queryable. `packages/react/src/toast/useToastManager.test.tsx:2053-2066`
- UNVERIFIED — inferred from `packages/react/src/toast/useToastManager.test.tsx:727-730`, no test asserts this: id linking between the toast root and title/description (e.g. `aria-labelledby`/`aria-describedby`) — no such attribute is asserted anywhere in these files.

## DOM structure & portal behavior

- The shared `List` helper renders one `Toast.Root key={toast.id} data-testid="root"` per toast containing `Toast.Title data-testid="title"`, `Toast.Description data-testid="description"`, `Toast.Close aria-label="close-press"`, and `Toast.Action data-testid="action"`. `packages/react/src/toast/utils/test-utils.tsx:30-39`
- `List` is rendered as a child of `Toast.Viewport`; toasts added through the manager appear under it. `packages/react/src/toast/createToastManager.test.tsx:32-39` and `packages/react/src/toast/createToastManager.test.tsx:41-46`
- Upserting with an existing id keeps exactly one rendered root (`queryAllByTestId('root')` length 1). `packages/react/src/toast/createToastManager.test.tsx:110-120` and `packages/react/src/toast/useToastManager.test.tsx:170-179`
- Adding 5 toasts renders 5 roots; closing all removes them. `packages/react/src/toast/useToastManager.test.tsx:1776-1785`
- Toasts exist in `useToastManager().toasts` (and render a count) even when no `Toast.Viewport` is mounted. `packages/react/src/toast/useToastManager.test.tsx:156-157` and `packages/react/src/toast/useToastManager.test.tsx:228-241`
- Components can also map `toasts` themselves into `Toast.Root` elements without the shared helper. `packages/react/src/toast/useToastManager.test.tsx:62-69` and `packages/react/src/toast/useToastManager.test.tsx:455-462`
- A toast added from inside a dialog (dialog portal open) is found in the document via `screen.getByTestId('toast-root')`. `packages/react/src/toast/useToastManager.test.tsx:2058-2062`
- UNVERIFIED — inferred from `packages/react/src/toast/useToastManager.test.tsx:2032-2041`, no test asserts this: the specific portal target/container of `Toast.Viewport` (tests only query via `screen`, never assert the mount node or `document.body` structure).

## Events (names, payload shape, bubbling, preventDefault semantics)

- No DOM event names are proven at the manager/store/hook level; behavior is expressed through toast-object callbacks.
- `onClose` is invoked once when the toast is closed programmatically via `close(id)`, and is not called before that. `packages/react/src/toast/useToastManager.test.tsx:598-606`
- `onClose` is invoked once when the toast auto-dismisses by timeout. `packages/react/src/toast/useToastManager.test.tsx:638-645`
- `onClose` is not called again for toasts that are already ending: after `close(toastId1)` triggers toast 1's `onClose` (which itself calls `close()` for all), each of the two toasts' `onClose` has been called exactly once — the already-ending toast is not re-notified. `packages/react/src/toast/createToastManager.test.tsx:790-841`
- `onRemove` is invoked once when a closed toast is removed. `packages/react/src/toast/useToastManager.test.tsx:690-698`
- `onRemove` is not invoked when a toast is replaced (re-added with the same id) while it is ending. `packages/react/src/toast/useToastManager.test.tsx:234-241`
- The replacement toast's `onRemove` fires exactly once when that replacement is later closed. `packages/react/src/toast/useToastManager.test.tsx:304-314`
- Store level: `removeToast` invokes the removed toast's `onRemove` exactly once; removing an id no longer in the store is a no-op (callback not fired a second time). `packages/react/src/toast/store.test.ts:217-227`
- UNVERIFIED — inferred from `packages/react/src/toast/useToastManager.test.tsx:558-607`, no test asserts this: the arguments passed to `onClose`/`onRemove` (payload shape, close reason) are never asserted in these files.
- `promise()` returns a thenable that rejects when the input promise rejects: every rejection test attaches `.catch` to the returned value to swallow the error. `packages/react/src/toast/createToastManager.test.tsx:235-248` and `packages/react/src/toast/useToastManager.test.tsx:817-819`
- UNVERIFIED — inferred from `packages/react/src/toast/useToastManager.test.tsx:940-982`, no test asserts this: the resolved value of the promise returned by `promise()` (only the rejection path's `.catch` usage is exercised).
- Hover pause/resume is exercised through `fireEvent.mouseEnter`/`fireEvent.mouseLeave` on the toast root element. `packages/react/src/toast/useToastManager.test.tsx:1464-1471`
- preventDefault semantics: N/A — no test in these files asserts `preventDefault`/`stopPropagation` behavior.

## Edge cases (rapid interactions, unmount, nesting)

- Rapid double `update` in a single handler (before any re-render) keeps the auto-dismiss timer alive. `packages/react/src/toast/createToastManager.test.tsx:654-695`
- Upsert while the auto-dismiss timer is pending resets the timer (dismissal lands a full `timeout` after the upsert). `packages/react/src/toast/createToastManager.test.tsx:113-126`
- Re-adding a currently-closing toast under the same id replaces it with a single root and `toasts.length === 1`. `packages/react/src/toast/useToastManager.test.tsx:170-179`
- Replacing an ending toast does not fire the old toast's `onRemove`. `packages/react/src/toast/useToastManager.test.tsx:237-241`
- A promise toast that is dismissed (via its close button) before resolution stays dismissed after the promise resolves. `packages/react/src/toast/createToastManager.test.tsx:310-319` and `packages/react/src/toast/useToastManager.test.tsx:1063-1072`
- Store: a toast added from inside an updater function is kept and lands before the updated toast in the array (order becomes `['b', 'a']`); metadata selectors stay consistent. `packages/react/src/toast/store.test.ts:157-167`
- Store: closing a toast from inside its own updater keeps it ending and the updater's returned patch is not applied (data unchanged). `packages/react/src/toast/store.test.ts:169-178`
- Store: mutations targeting an unknown id (`removeToast`, `closeToast`, `updateToast`) are no-ops that leave `state.toasts` reference-identical. `packages/react/src/toast/store.test.ts:86-96`
- Store: the updater function is not invoked for a missing or already-ending toast. `packages/react/src/toast/store.test.ts:145-155`
- Store: height recalculation writes via `updateToastInternal` are ignored while a toast is transitioning out — after `closeToast`, a write of `{ height: 80, transitionStatus: undefined }` leaves `transitionStatus: 'ending'` and `height: 0`. `packages/react/src/toast/store.test.ts:67-84`
- Store: metadata invariants (selector identity, index, offsetY accumulation, visibleIndex with `-1` for ending) hold after every mutation sequence. `packages/react/src/toast/store.test.ts:40-65`
- Multiple `Toast.Provider`s are isolated: updating a toast in the first provider does not affect toasts in the second. `packages/react/src/toast/useToastManager.test.tsx:94-115`
- Nesting: toasts can be added from inside an open `Dialog` (both regular and high priority) and render in the viewport. `packages/react/src/toast/useToastManager.test.tsx:2043-2109`
- Provider prop changes (`timeout`, `limit`) are picked up for subsequent interactions without remounting. `packages/react/src/toast/useToastManager.test.tsx:1794-1822` and `packages/react/src/toast/useToastManager.test.tsx:1965-1995`
- UNVERIFIED — inferred from `packages/react/src/toast/useToastManager.test.tsx:182-242`, no test asserts this: provider unmount behavior (no test unmounts the provider and checks toast cleanup).

## Shared harness dependencies

- Suites are jsdom-only: `describe.skipIf(!isJSDOM)`. `packages/react/src/toast/createToastManager.test.tsx:9` and `packages/react/src/toast/useToastManager.test.tsx:15`
- `createRenderer()` from `#test-utils` provides `render` and `clock`; every React suite enables fake timers via `clock.withFakeTimers()`. `packages/react/src/toast/createToastManager.test.tsx:10-12` and `packages/react/src/toast/useToastManager.test.tsx:17-19`
- The store suite uses raw `vi.useFakeTimers()` / `vi.advanceTimersByTime` / `vi.setSystemTime` with `vi.useRealTimers()` in `afterEach`. `packages/react/src/toast/store.test.ts:247-249`, `packages/react/src/toast/store.test.ts:252`, and `packages/react/src/toast/store.test.ts:380`
- `fireEvent`, `flushMicrotasks`, and `screen` come from `@mui/internal-test-utils`. `packages/react/src/toast/createToastManager.test.tsx:4` and `packages/react/src/toast/useToastManager.test.tsx:5`
- A local `tick(clock, ms)` helper combines `clock.tick` with `flushMicrotasks`. `packages/react/src/toast/useToastManager.test.tsx:10-13`
- `createRenderer`'s `render` wraps rendering in `act` and exposes `setProps` (used to change provider `timeout`/`limit`). `packages/react/test/createRenderer.ts:31-43` and `packages/react/src/toast/useToastManager.test.tsx:1811-1813`
- `#test-utils` re-exports the shared harness (`createRenderer`, `describeConformance`, pointer helpers, wait helpers, etc.). `packages/react/test/index.ts:1-11`
- Toast-local helpers `Button` and `List` live in `packages/react/src/toast/utils/test-utils.tsx` and are imported by both React test files. `packages/react/src/toast/createToastManager.test.tsx:6` and `packages/react/src/toast/useToastManager.test.tsx:8`
- No `describeConformance` usage appears in any of the three test files (not a parts/conformance-tested surface here).
