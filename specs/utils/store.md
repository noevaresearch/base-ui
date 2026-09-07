# `store` — behavior spec

Unit: `packages/utils/src/store` (Phase A util → crate `leptos-ui-utils`).
Source of truth: the unit's own test suite — `packages/utils/src/store/ReactStore.test.tsx`,
`packages/utils/src/store/Store.test.ts`, `packages/utils/src/store/createSelector.test.ts`,
`packages/utils/src/store/createSelectorMemoized.test.ts`. The `TODO.md` entry has no
`wraps-external:` field (`TODO.md:124-128`), so there is no runtime delegation to a
third-party package; the Rust port implements the behavior below. The unit is a headless
state container: no DOM, no portal, no keyboard/focus/a11y surface of its own.

## Public API surface (props, parts, subcomponents)

No components, props, parts, or subcomponents. Four modules of class/factory exports are
exercised by the suite:

- **`Store`** — plain observable state container.
  - `Store.create(initialState)` static factory: returns a `Store` instance with `state`
    seeded to the argument (`packages/utils/src/store/Store.test.ts:8-13`); each call
    produces an independent instance — mutating one does not affect another
    (`packages/utils/src/store/Store.test.ts:15-23`); when invoked on a subclass it
    constructs an instance of that subclass (`packages/utils/src/store/Store.test.ts:25-37`).
  - Direct construction via `new Store(state)` also works
    (`packages/utils/src/store/Store.test.ts:40-45`).
  - Instance API exercised by tests: read `state` (`packages/utils/src/store/Store.test.ts:12`),
    `setState(next)` (`packages/utils/src/store/Store.test.ts:45`),
    `set(key, value)` (`packages/utils/src/store/Store.test.ts:19`),
    `update(partial)` (`packages/utils/src/store/Store.test.ts:91`),
    `subscribe(listener)` returning an unsubscribe function
    (`packages/utils/src/store/Store.test.ts:65`), and `notifyAll()`
    (`packages/utils/src/store/Store.test.ts:105`).

- **`ReactStore`** — the React-facing store.
  - `ReactStore.create(initialState)`: returns an instance of `ReactStore`
    (`packages/utils/src/store/ReactStore.test.tsx:21`) with state seeded
    (`packages/utils/src/store/ReactStore.test.tsx:22`) and a `context` field equal to
    `{}` (`packages/utils/src/store/ReactStore.test.tsx:24`).
  - `new ReactStore(initialState, context?, selectors?)`: tests always pass `undefined`
    as the second argument and optionally a named-selector map as the third
    (`packages/utils/src/store/ReactStore.test.tsx:287-291`,
    `packages/utils/src/store/ReactStore.test.tsx:369-373`).
  - Named-selector map: selectors registered under keys in the third constructor
    argument are addressable by key from `observe`
    (`packages/utils/src/store/ReactStore.test.tsx:437`), `useState`
    (`packages/utils/src/store/ReactStore.test.tsx:376`), and `select`
    (`packages/utils/src/store/ReactStore.test.tsx:344`).
  - Mutation API (inherited/effective): `set(key, value)`
    (`packages/utils/src/store/ReactStore.test.tsx:335`), `update(partial)`
    (`packages/utils/src/store/ReactStore.test.tsx:40`), `notifyAll()`
    (`packages/utils/src/store/ReactStore.test.tsx:311`).
  - React hook integrations called in tests: `useControlledProp(key, controlled)`
    (`packages/utils/src/store/ReactStore.test.tsx:32`),
    `useSyncedValue(key, value)` (`packages/utils/src/store/ReactStore.test.tsx:108`),
    `useSyncedValues(props)` (`packages/utils/src/store/ReactStore.test.tsx:141`),
    `useSyncedValueWithCleanup(key, value)`
    (`packages/utils/src/store/ReactStore.test.tsx:217`),
    `useStateSetter(key)` (`packages/utils/src/store/ReactStore.test.tsx:245`), and
    `useState(selectorKey, ...args)` (`packages/utils/src/store/ReactStore.test.tsx:327`,
    `packages/utils/src/store/ReactStore.test.tsx:376`).
  - Observation API: `observe(keyOrSelector, listener)` accepts a state key string, a
    named-selector key, or an inline selector function
    (`packages/utils/src/store/ReactStore.test.tsx:323-324`,
    `packages/utils/src/store/ReactStore.test.tsx:405-410`) and returns an unsubscribe
    function (`packages/utils/src/store/ReactStore.test.tsx:423`).
  - `select(key)` reads state through a named selector
    (`packages/utils/src/store/ReactStore.test.tsx:344`).
  - Files visible in the unit directory but not exercised by any test in this suite:
    `useStore.ts` and `StoreInspector.tsx` (directory listing only; nothing asserted
    about them here).

- **`createSelector`** — variadic selector combinator.
  - Called with a single function it returns that function itself (identity, no
    wrapping) (`packages/utils/src/store/createSelector.test.ts:5-11`).
  - Called with N input selectors plus a trailing combiner it returns a derived
    selector; proven for 1 input (`packages/utils/src/store/createSelector.test.ts:13-23`),
    6 inputs (`packages/utils/src/store/createSelector.test.ts:25-40`), and 7 inputs
    (`packages/utils/src/store/createSelector.test.ts:42-66`).
  - Throws `'Unsupported number of selectors'` when given eight input selectors plus a
    combiner (`packages/utils/src/store/createSelector.test.ts:68-75`).

- **`createSelectorMemoized`** — memoized variant.
  - `createSelectorMemoized(...inputs, combiner)` with input selectors
    (`packages/utils/src/store/createSelectorMemoized.test.ts:50-54`); the
    single-function form treats its only argument as the combiner and wires an identity
    input selector (`packages/utils/src/store/createSelectorMemoized.test.ts:9-21`).
  - **`createSelectorMemoizedWithOptions(options)`** returns a selector factory;
    calling it with no options yields a working memoized selector
    (`packages/utils/src/store/createSelectorMemoized.test.ts:154-165`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A in the React-form sense (the store is not a rendered component), but the unit
implements controlled/uncontrolled prop-syncing for state keys and has precise update
semantics:

- **`useControlledProp(key, controlled)`**: when `controlled` is defined, the store's
  state key is synced from it on render — initial render with `controlled={1}` sets
  `state.value` to 1 (`packages/utils/src/store/ReactStore.test.tsx:36-37`); store
  updates to *other* keys still apply while a key is controlled
  (`packages/utils/src/store/ReactStore.test.tsx:39-43`); a later prop change syncs into
  state (`packages/utils/src/store/ReactStore.test.tsx:46-49`).
- **Controlled/uncontrolled switching warns in dev**: rendering uncontrolled then
  passing a controlled value produces the dev error
  `'A component is changing the controlled state of value to be uncontrolled. Elements should not switch from uncontrolled to controlled (or vice versa).'`
  twice (`packages/utils/src/store/ReactStore.test.tsx:78-83`); the reverse switch
  produces
  `'A component is changing the uncontrolled state of value to be controlled. Elements should not switch from uncontrolled to controlled (or vice versa).'`
  twice (`packages/utils/src/store/ReactStore.test.tsx:97-100`). (Note the asserted
  message wording is crossed relative to the switch direction; the spec records the
  strings as asserted, not as they "should" read.)
- **Store-instance swap**: when the store instance passed to the component changes, the
  new store receives the synced value — `useControlledProp` on the second store sets its
  `value` to 1 after the swap
  (`packages/utils/src/store/ReactStore.test.tsx:61-66`), and the same holds for
  `useSyncedValue` (`packages/utils/src/store/ReactStore.test.tsx:128-133`).
- **`useSyncedValue(key, value)`**: writes a single key whenever the passed value
  changes (`packages/utils/src/store/ReactStore.test.tsx:112-117`).
- **`useSyncedValues(props)`**: applies multiple keys from a props object, on mount and
  on prop change (`packages/utils/src/store/ReactStore.test.tsx:145-151`); it depends on
  *entries*, not object identity — a spy on `store.update` is called exactly once on
  mount, zero additional times for a rerender with an identical-but-new props object,
  and a second time once a value actually changes
  (`packages/utils/src/store/ReactStore.test.tsx:171-184`).
- **`useSyncedValues` key stability contract**: changing the set of keys between renders
  triggers the dev error
  `'ReactStore.useSyncedValues expects the same prop keys on every render. Keys should be stable.'`
  twice (`packages/utils/src/store/ReactStore.test.tsx:198-205`).
- **`useStateSetter(key)`**: returns a stable callback (identity preserved across a
  forced re-render) that writes the value into the store state when called
  (`packages/utils/src/store/ReactStore.test.tsx:258-271`).
- **Update semantics (`Store`)**: `setState` notifies every subscriber once with the new
  state (`packages/utils/src/store/Store.test.ts:45-48`); passing the *current state
  reference* notifies no one (`packages/utils/src/store/Store.test.ts:57-59`);
  `set(key, value)` writes one key and notifies once, and a second `set` with the same
  value is skipped (`packages/utils/src/store/Store.test.ts:78-83`); `update(partial)`
  merges changed keys, notifies once, and skips no-op updates
  (`packages/utils/src/store/Store.test.ts:91-96`).
- **`notifyAll()`**: renews the state reference — `state` becomes a new object
  (`not.toBe`) with equal content — and notifies subscribers once
  (`packages/utils/src/store/Store.test.ts:103-109`). The nested-store pattern uses it
  to force re-notification of the child store after subscribing to the parent
  (`packages/utils/src/store/ReactStore.test.tsx:310-312`).
- **Derived state via named selectors**: `select(key)` reads through the named selector
  rather than raw state — after a parent store is attached, `childStore.select('count')`
  returns the parent's count (0) while `childStore.state.count` stays 5
  (`packages/utils/src/store/ReactStore.test.tsx:343-345`); see Edge cases for the full
  nested-store transitions.

## Keyboard interactions

N/A — headless state utility; no keyboard surface is rendered or asserted anywhere in
the suite.

## Focus management

N/A — no focusable output; test components render `null` or plain `<output>` elements
(`packages/utils/src/store/ReactStore.test.tsx:326-329`).

## Accessibility (roles, aria-*, id linking)

N/A — no roles, aria attributes, or id linking exist in this unit; nothing in the suite
asserts any.

## DOM structure & portal behavior

N/A — the store renders nothing and no portal is used. React rendering appears in tests
only as a host for the hook integrations: components render `null` for the prop-sync
hooks (`packages/utils/src/store/ReactStore.test.tsx:33`) and `<output>` elements to
observe `useState` re-renders (`packages/utils/src/store/ReactStore.test.tsx:328`,
`packages/utils/src/store/ReactStore.test.tsx:377`).

## Events (names, payload shape, bubbling, preventDefault semantics)

No DOM events. The unit's "event" system is store notification:

- **`subscribe(listener)`**: the listener is called with the new state object on every
  notifying update (`packages/utils/src/store/Store.test.ts:47-48`).
- **`observe(selector, listener)`** payload is `(newValue, oldValue, store)`:
  - the listener fires once immediately on subscription with the current selector
    result in both slots — for an inline boolean selector the first call is
    `{ newValue: false, oldValue: false }`
    (`packages/utils/src/store/ReactStore.test.tsx:412-413`), and for a named selector
    the initial call is `{ newValue: 10, oldValue: 10 }`
    (`packages/utils/src/store/ReactStore.test.tsx:440-442`);
  - subsequent calls fire only when the selector result changes, with the previous
    result as `oldValue` (`packages/utils/src/store/ReactStore.test.tsx:457-462`); a
    state change that leaves the selector result unchanged fires nothing
    (`packages/utils/src/store/ReactStore.test.tsx:477-479`);
  - a change to *any* dependency of the selector fires the listener — the `multiplied`
    selector fires for a `count` change and again for a `multiplier` change
    (`packages/utils/src/store/ReactStore.test.tsx:494-500`);
  - the third argument is the store instance itself
    (`packages/utils/src/store/ReactStore.test.tsx:511-515`);
  - multiple observers on the same selector each receive the sequence of values
    (`packages/utils/src/store/ReactStore.test.tsx:556-559`), and observers on different
    selectors fire independently of each other
    (`packages/utils/src/store/ReactStore.test.tsx:579-583`);
  - the returned unsubscribe function stops all further calls
    (`packages/utils/src/store/ReactStore.test.tsx:530-536`,
    `packages/utils/src/store/ReactStore.test.tsx:423-427`).
- **Selector-argument reactivity**: `store.useState('valueByKey', valueKey)` inside a
  `fastComponent`-wrapped component re-renders the output when the selector *argument*
  changes (`'first'` → `'second'` flips the rendered value `'one'` → `'two'`)
  (`packages/utils/src/store/ReactStore.test.tsx:384-390`).
- Bubbling and `preventDefault` semantics: N/A — nothing propagates or is cancelable;
  there is no `details.cancel()`-style API in this unit's tests.

## Edge cases (rapid interactions, unmount, nesting)

- **Re-entrant/nested update during notification**: a subscriber that calls
  `store.set(...)` while being notified does not deadlock or double-notify — with
  listener `first` writing `value: 2` upon seeing `value: 1`, the final state is 2,
  `first` is called twice, and `second` is called exactly once with the *final* state
  `{ value: 2, label: 'a' }` (the outer pass detects the nested update and skips
  delivering the stale state) (`packages/utils/src/store/Store.test.ts:112-131`).
- **Rapid sequential updates**: each notifying `set` delivers exactly one
  `observe` callback — two back-to-back sets produce calls with correct old→new pairs
  (`packages/utils/src/store/ReactStore.test.tsx:457-462`), while no-op writes
  (`set`/`update`/`setState` with unchanged values or identical reference) produce none
  (`packages/utils/src/store/Store.test.ts:82-83`,
  `packages/utils/src/store/Store.test.ts:95-96`,
  `packages/utils/src/store/Store.test.ts:57-59`).
- **Unmount**: `useSyncedValueWithCleanup` resets its state key to `undefined` when the
  component unmounts (after syncing on mount and on prop change)
  (`packages/utils/src/store/ReactStore.test.tsx:221-232`).
- **Nesting (store-in-store)**: a child store may hold a parent store as a state value,
  with cross-store coordination wired through `observe`:
  - observing the `parent` key lets the child subscribe/unsubscribe to the parent on
    attach/removal — the handler subscribes to the parent with a `store.notifyAll()`
    callback when a parent arrives and unsubscribes when it becomes `undefined`
    (`packages/utils/src/store/ReactStore.test.tsx:299-313`);
  - an observer on the local `count` selector propagates local writes into the parent
    via `store.state.parent?.set('count', newCount)`
    (`packages/utils/src/store/ReactStore.test.tsx:315-324`):
    after `childStore.set('count', 20)` the parent's count is also 20
    (`packages/utils/src/store/ReactStore.test.tsx:347-352`);
  - attaching a parent flips the derived view immediately: rendered output goes from
    `'5'` to the parent's `0` while raw child state stays 5
    (`packages/utils/src/store/ReactStore.test.tsx:340-345`);
  - parent updates flow back into the rendered output through the parent subscription —
    `parentStore.set('count', 15)` leaves raw child state at 20 but
    `childStore.select('count')` and the rendered output become 15
    (`packages/utils/src/store/ReactStore.test.tsx:355-361`).
- **Strict-mode notes**: the spy and fast-component tests render with
  `{ strict: false }` (`packages/utils/src/store/ReactStore.test.tsx:169`,
  `packages/utils/src/store/ReactStore.test.tsx:381`); the dev-warning tests assert the
  warning text twice per switch (`packages/utils/src/store/ReactStore.test.tsx:80-83`,
  `packages/utils/src/store/ReactStore.test.tsx:97-100`,
  `packages/utils/src/store/ReactStore.test.tsx:202-205`).
- **Arity overflow errors**: `createSelector` throws `'Unsupported number of selectors'`
  beyond seven inputs (`packages/utils/src/store/createSelector.test.ts:71-74`);
  `createSelectorMemoized` throws `'Unsupported number of arguments'` when the combiner
  takes more than three extra arguments
  (`packages/utils/src/store/createSelectorMemoized.test.ts:133-140`) and also when
  `Function.length` under-reports the combiner's parameters (rest-parameter combiner)
  (`packages/utils/src/store/createSelectorMemoized.test.ts:144-150`).
- **Memoization corner cases**:
  - caching is per *state identity*: two distinct state objects with equal content each
    run the combiner (`packages/utils/src/store/createSelectorMemoized.test.ts:74-80`);
  - the single-function form passes the state to the combiner first, followed by extra
    args, and memoizes on all of them — changing an extra argument re-runs the combiner
    (`packages/utils/src/store/createSelectorMemoized.test.ts:38-43`);
  - extra-argument memoization holds up to three additional arguments
    (`packages/utils/src/store/createSelectorMemoized.test.ts:100-129`);
  - `NaN` in state still memoizes via `Object.is` equality when object
    `memoizeOptions` are merged (a spread state with `NaN` value does not re-run the
    combiner) (`packages/utils/src/store/createSelectorMemoized.test.ts:199-208`);
  - a `resultEqualityCheck` option preserves the *result reference* across calls with
    different extra arguments (`packages/utils/src/store/createSelectorMemoized.test.ts:302-305`).
- **`createSelectorMemoizedWithOptions` option pass-through**:
  - `argsMemoize`/`argsMemoizeOptions`/`devModeChecks` are forwarded to the underlying
    reselect creator — an always-unequal custom `argsMemoize` re-runs input selectors on
    every call while the combiner stays memoized
    (`packages/utils/src/store/createSelectorMemoized.test.ts:172-187`);
  - the module's `lruMemoize` defaults are *not* leaked to a custom `memoize` (the
    memoizer observes zero options) (`packages/utils/src/store/createSelectorMemoized.test.ts:228-231`)
    and a custom memoizer that defaults its options parameter still works
    (`packages/utils/src/store/createSelectorMemoized.test.ts:256-262`);
  - custom `memoizeOptions` are forwarded verbatim as a single argument
    (`packages/utils/src/store/createSelectorMemoized.test.ts:283-285`).
- **`createSelector` extra-argument forwarding**: every input selector receives the
  state plus *all* extra arguments positionally, and the combiner receives the input
  results followed by all extra args — `selector(state, 3, 7)` yields inputs
  `(state, 3)` / `(state, 3, 7)` and combiner args `(30, 17, 3, 7)`
  (`packages/utils/src/store/createSelector.test.ts:81-87`).

## Shared harness dependencies

- No repo-internal shared harness is used: none of the four test files imports
  `#test-utils` or anything under `packages/react/test/`.
- External npm test utilities used directly: `@mui/internal-test-utils` supplying `act`,
  `createRenderer`, and `screen` (`packages/utils/src/store/ReactStore.test.tsx:3`);
  Vitest's `expect`/`vi`/`describe`/`it`/`MockInstance` in all four files
  (`packages/utils/src/store/ReactStore.test.tsx:1`,
  `packages/utils/src/store/Store.test.ts:1`,
  `packages/utils/src/store/createSelector.test.ts:1`,
  `packages/utils/src/store/createSelectorMemoized.test.ts:1`); and `reselect`'s
  `lruMemoize` as an option value in the WithOptions tests
  (`packages/utils/src/store/createSelectorMemoized.test.ts:2`).
- In-unit sibling source imports appear in `ReactStore.test.tsx` — `useRefWithInit`
  (`packages/utils/src/store/ReactStore.test.tsx:5`) and `fastComponent` from
  `../fastHooks` (`packages/utils/src/store/ReactStore.test.tsx:6`) — these are source
  modules of adjacent utils (not harness); their behavior is specced separately in
  `specs/utils/fastHooks.md`.
