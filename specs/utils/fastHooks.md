# `fastHooks` — behavior spec

Unit: `packages/utils/src/fastHooks` (single-file infra util → crate `leptos-ui-utils`).
Source of truth: **none of its own** — the unit has no dedicated test suite; the generated manifest
records `testFiles: []` for it (`ralph/generated/utils.json:62-66`). The only test in the repository
that exercises any of this unit's API is the store unit's suite, which imports `fastComponent` and
renders a wrapped component (`packages/utils/src/store/ReactStore.test.tsx:6`,
`packages/utils/src/store/ReactStore.test.tsx:364-391`).
Scope note: with no test files to mine, claims below are either (a) test-proven behavior cited to
that store test, or (b) marked `UNVERIFIED — inferred from ...` with citations into the unit's own
source file, per the template's UNVERIFIED rule. The unit's `TODO.md` entry has no
`wraps-external:` field, so no external-package delegation applies.

## Public API surface (props, parts, subcomponents)

- Single module `packages/utils/src/fastHooks.ts`; no components, props, parts, or subcomponents of
  its own — this is a render-plumbing infra util.
- Exports:
  - `type Instance = { didInitialize: boolean }` (`packages/utils/src/fastHooks.ts:4-6`).
  - `getInstance(): Instance | undefined` — reads the module-level current instance
    (`packages/utils/src/fastHooks.ts:17-19`).
  - `setInstance(instance: Instance | undefined): void` — overwrites the module-level current
    instance (`packages/utils/src/fastHooks.ts:21-23`). Intended for code rendering outside a
    `fastComponent` wrapper to install an instance context. UNVERIFIED — inferred from
    `packages/utils/src/fastHooks.ts:21-23`, no test asserts this.
  - `register(hook): void` — appends a `{ before, after }` hook pair to a module-level,
    append-only registry (`packages/utils/src/fastHooks.ts:25-27`,
    `packages/utils/src/fastHooks.ts:13`). The hook parameter type `HookType` is internal, not
    exported (`packages/utils/src/fastHooks.ts:8-11`).
  - `fastComponent<P, E, R>(fn: (props: P) => R): typeof fn` — wraps a component function and
    returns a component with the same signature as the input
    (`packages/utils/src/fastHooks.ts:63-65`, `packages/utils/src/fastHooks.ts:91`).
  - `fastComponentRef<P, E, R>(fn)` — convenience wrapper that combines `fastComponent` with
    `React.forwardRef` so the wrapped function receives a forwarded ref as its second argument
    (`packages/utils/src/fastHooks.ts:118-122`).
- Library usage pattern: root components are wrapped with `fastComponent` and trigger components
  with `fastComponentRef`, e.g. `packages/react/src/preview-card/root/PreviewCardRoot.tsx:3` and
  `packages/react/src/dialog/trigger/DialogTrigger.tsx:3`.
- The one specialized hook that consumes the protocol is `useStore`: it registers its before/after
  hooks at module load and reads the current instance during render
  (`packages/utils/src/store/useStore.ts:7`, `packages/utils/src/store/useStore.ts:78`,
  `packages/utils/src/store/useStore.ts:136`).
- Build-tool coupling: the shared babel display-name plugin is configured to recognize
  `fastComponent`/`fastComponentRef` calls as component definitions so display names are assigned
  at build time (`babel.config.mjs:29-32`).

## State model (controlled/uncontrolled, defaults, transitions)

- There is no controlled/uncontrolled concept. The unit's state is a module-level render-context
  singleton plus a per-component instance object.
- Module-level singleton `currentInstance` (`packages/utils/src/fastHooks.ts:15`): set to the
  component's instance immediately before calling the wrapped render function
  (`packages/utils/src/fastHooks.ts:70-71`) and always cleared afterwards in a `finally` block
  (`packages/utils/src/fastHooks.ts:84-86`). Specialized hooks read it via `getInstance()`
  (`packages/utils/src/fastHooks.ts:17-19`; consumer at
  `packages/utils/src/store/useStore.ts:136`).
- Per-component instance: created lazily exactly once per mounted wrapper component via
  `useRefWithInit(createInstance)` (`packages/utils/src/fastHooks.ts:67`; lazy-once semantics in
  `packages/utils/src/useRefWithInit.ts:16-22`), starting as `{ didInitialize: false }`
  (`packages/utils/src/fastHooks.ts:124-128`).
- Per-render transition: set `currentInstance` → run every registered hook's `before(instance)` →
  call the wrapped `fn(props, forwardedRef)` → run every registered hook's `after(instance)` → set
  `instance.didInitialize = true` → `finally` clear `currentInstance` → return the render result
  (`packages/utils/src/fastHooks.ts:70-88`).
- Second and later renders reuse the same instance object with `didInitialize` already `true`;
  specialized hooks branch on that flag for one-time initialization
  (`packages/utils/src/store/useStore.ts:82-85`).
- Test-proven behavior: a `fastComponent`-wrapped component using the store-backed
  `store.useState(...)` hook re-renders and displays the updated value when a selector argument
  changes, i.e. the wrapper + instance protocol keeps the store subscription update path working
  (`packages/utils/src/store/ReactStore.test.tsx:380`,
  `packages/utils/src/store/ReactStore.test.tsx:386-390`).
- The performance benefit claimed in the JSDoc — multiple `useStore` calls in one wrapped component
  collapsing into a single `useSyncExternalStore` subscription per store, active only on React 19+
  — is implemented on the `useStore` side, not here: the implementation is selected by React
  version (`packages/utils/src/store/useStore.ts:12-13`), the fast path reads `getInstance()` and
  falls back to a per-call subscription when there is no instance
  (`packages/utils/src/store/useStore.ts:136-140`), and the `after` hook performs exactly one
  `useSyncExternalStore` call over the union of subscribed stores
  (`packages/utils/src/store/useStore.ts:103-126`,
  `packages/utils/src/store/useStore.ts:124`). UNVERIFIED as end-to-end behavior — inferred from
  `packages/utils/src/fastHooks.ts:37-40`, no test asserts the subscription-count reduction.

## Keyboard interactions

N/A — the unit renders no DOM of its own and contains no keyboard handling anywhere in its source
(`packages/utils/src/fastHooks.ts:1-128`).

## Focus management

N/A — no focus behavior exists in the unit (`packages/utils/src/fastHooks.ts:1-128`).

## Accessibility (roles, aria-*, id linking)

N/A — the unit produces no roles, aria attributes, or id linking; it only mediates render execution
(`packages/utils/src/fastHooks.ts:63-92`).

## DOM structure & portal behavior

- Renders nothing itself: the wrapper returns whatever the wrapped component function returns
  (`packages/utils/src/fastHooks.ts:88`). `fastComponentRef` additionally wraps the result in
  `React.forwardRef` so refs reach the wrapped function as its second argument, which
  `fastComponent` forwards through to `fn` (`packages/utils/src/fastHooks.ts:121`,
  `packages/utils/src/fastHooks.ts:77`). UNVERIFIED — inferred from
  `packages/utils/src/fastHooks.ts:66`, no test asserts the ref-forwarding plumbing end to end.
- No portal behavior — N/A beyond the above.

## Events (names, payload shape, bubbling, preventDefault semantics)

- No DOM events are emitted or observed by the unit; bubbling and preventDefault semantics do not
  apply.
- The unit's only event-like protocol is the registered-hook callback pair: `before(instance)`
  runs immediately before the wrapped render function and `after(instance)` runs immediately after
  it, each receiving the per-component `Instance` object as payload
  (`packages/utils/src/fastHooks.ts:73-75`, `packages/utils/src/fastHooks.ts:79-81`). The payload
  type is declared loosely (`any`) in `HookType` (`packages/utils/src/fastHooks.ts:8-11`) and is
  concretized by consumers, e.g. `StoreInstance` extends `Instance` with batching fields
  (`packages/utils/src/store/useStore.ts:62-76`).

## Edge cases (rapid interactions, unmount, nesting)

- Exception safety: if the wrapped render function throws, the `after` hooks and the
  `didInitialize = true` assignment are skipped and the error propagates, while `currentInstance`
  is still cleared because the cleanup sits in `finally`. UNVERIFIED — inferred from
  `packages/utils/src/fastHooks.ts:70-86`, no test asserts this.
- Hook matching by call index: specialized hooks must keep a stable call order and count across
  renders because batched calls are matched by index (`packages/utils/src/fastHooks.ts:44-45`);
  `useStore` implements that matching with a per-render `syncIndex` cursor and a first-render push
  into `syncHooks` (`packages/utils/src/store/useStore.ts:142-144`,
  `packages/utils/src/store/useStore.ts:146-157`). UNVERIFIED — inferred from
  `packages/utils/src/fastHooks.ts:44-45`, no test asserts the unstable-order failure mode.
- Registry is append-only: `register` has no unregister counterpart and the module-level `hooks`
  array can only grow (`packages/utils/src/fastHooks.ts:13`,
  `packages/utils/src/fastHooks.ts:25-27`); in practice it is populated once at module load by
  `useStore` (`packages/utils/src/store/useStore.ts:78`). UNVERIFIED — inferred from
  `packages/utils/src/fastHooks.ts:25-27`, no test asserts this.
- Unmount: the unit performs no unmount-time work and registers no cleanup of its own; the only
  subscription teardown path is the unsubscribe function returned by the consumer's `subscribe`
  implementation (`packages/utils/src/store/useStore.ts:116-120`). UNVERIFIED — inferred from
  `packages/utils/src/fastHooks.ts:63-92`, no test asserts this.
- StrictMode: the only test that renders a `fastComponent`-wrapped component renders with strict
  mode explicitly disabled (`packages/utils/src/store/ReactStore.test.tsx:381`). UNVERIFIED for
  strict-mode behavior — inferred from `packages/utils/src/store/ReactStore.test.tsx:381`, no test
  asserts strict-mode double-render behavior.
- Nesting/reentrancy: `currentInstance` is a single module-level slot set synchronously around the
  wrapped render call and cleared in `finally`; children are rendered by React in separate passes
  after the wrapper returns, so the slot is not reentered mid-render. UNVERIFIED — inferred from
  `packages/utils/src/fastHooks.ts:70-86`, no test asserts this.
- Rapid prop changes: the test-proven update path re-evaluates the selector when selector arguments
  change between renders and the rendered output switches to the new value
  (`packages/utils/src/store/ReactStore.test.tsx:386-390`).
- Display name: the wrapper copies `fn.displayName || fn.name` onto the wrapped component
  (`packages/utils/src/fastHooks.ts:90`), complemented at build time by the babel display-name
  plugin allowlisting these call wrappers (`babel.config.mjs:29-32`). UNVERIFIED — inferred from
  `packages/utils/src/fastHooks.ts:90`, no test asserts this.

## Shared harness dependencies

None of its own — the unit has no test files (`ralph/generated/utils.json:62-66`). For reference,
the one indirect consumer test imports vitest and the `@mui/internal-test-utils` harness
(`createRenderer`, `act`, `screen`) (`packages/utils/src/store/ReactStore.test.tsx:1-3`); that
harness belongs to the store unit's suite, not to this unit.
