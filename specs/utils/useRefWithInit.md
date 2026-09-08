# `useRefWithInit` — behavior spec

Unit: `packages/utils/src/useRefWithInit` (Phase A util → crate `leptos-ui-utils`).
Spec target per `TODO.md`: `specs/utils/useRefWithInit.md` (`TODO.md:234-238`). The `TODO.md`
entry has no `wraps-external:` field — this is original Base UI code, not a wrapper around a
third-party npm package — and no `needs-batched-mining:` field (the unit is a single 23-line
file).

Source of truth: **almost none.** The unit inventory lists `testFiles: []` for `useRefWithInit`
— there is no dedicated `useRefWithInit.test.*` anywhere in the repo
(`ralph/generated/utils.json:368-374`). Per the template's import-detection rule, a repo-wide
import search found exactly one test file that imports and runtime-exercises the unit:
`packages/utils/src/store/ReactStore.test.tsx` (`packages/utils/src/store/ReactStore.test.tsx:5`),
which uses it through a local `useStableStore` fixture
(`packages/utils/src/store/ReactStore.test.tsx:11-13`). That suite is testing `ReactStore`, not
this hook, so it only proves `useRefWithInit`'s most basic contract (usable as a hook,
synchronous first-render initialization, identity stability across re-renders). Every other
claim below is UNVERIFIED — inferred from the unit's own source
(`packages/utils/src/useRefWithInit.ts`) with no test asserting it. Stage 2/3 should treat
source-derived claims as implementation behavior, not test-proven behavior, and should consider
adding a `packages/utils/src/useRefWithInit.test.ts` suite before porting.

## Public API surface (props, parts, subcomponents)

- Single named export: the hook `useRefWithInit`. No default export, no components, no props
  object, no parts, no subcomponents (`packages/utils/src/useRefWithInit.ts:13-15`).
- Module is marked `'use client'` (`packages/utils/src/useRefWithInit.ts:1`).
- Two overload signatures (`packages/utils/src/useRefWithInit.ts:13-14`):
  - `useRefWithInit<T>(init: () => T): React.RefObject<T>` — zero-argument factory.
  - `useRefWithInit<T, U>(init: (arg: U) => T, initArg: U): React.RefObject<T>` — factory
    taking one initialization argument, so the init function does not need to be an inline
    closure (JSDoc usage example: `useRefWithInit(sortColumns, columns)`,
    `packages/utils/src/useRefWithInit.ts:7-11`).
- Returns the `React.useRef` box itself (callers read `.current`
  (`packages/utils/src/useRefWithInit.ts:16-22`)); proven usage shape is
  `useRefWithInit(() => new ReactStore<State>(initial)).current`
  (`packages/utils/src/store/ReactStore.test.tsx:12`).
- UNVERIFIED — inferred from `packages/utils/src/useRefWithInit.ts:14`, no test asserts this:
  the two-argument `initArg` overload. The only detected test usage is single-argument
  (`packages/utils/src/store/ReactStore.test.tsx:12`).

## State model (controlled/uncontrolled, defaults, transitions)

- Not a form/state primitive: no controlled/uncontrolled semantics, no props, no user-facing
  state. N/A beyond the internal latch described below.
- Internal state is the ref slot itself, with two phases separated by a module-private
  `UNINITIALIZED` sentinel (`const UNINITIALIZED = {}`, `packages/utils/src/useRefWithInit.ts:4`):
  - Uninitialized: `ref.current === UNINITIALIZED` — the state on `React.useRef` creation
    (`packages/utils/src/useRefWithInit.ts:16`, guard at `packages/utils/src/useRefWithInit.ts:18`).
  - Initialized: `ref.current = init(initArg)` — the factory is invoked with the optional init
    argument and its return value replaces the sentinel
    (`packages/utils/src/useRefWithInit.ts:19`).
- Transition is one-way per component-instance lifetime: once initialized, the sentinel check
  never matches again (the sentinel object is module-private and cannot be produced by caller
  code), so `init` cannot re-run on later renders. UNVERIFIED — inferred from
  `packages/utils/src/useRefWithInit.ts:4,18-19`, no test asserts this.
- Per-component instance: each component calling the hook gets its own ref/instance; there is
  no shared or module-level state (`packages/utils/src/useRefWithInit.ts:16`). UNVERIFIED —
  inferred from source; consistent with the detected tests, where each rendered fixture gets an
  independent store (`packages/utils/src/store/ReactStore.test.tsx:31,107,140,159,189`).

Test-proven parts of this model (via the `ReactStore` suite):

- Initialization happens synchronously during the first render, before the component body
  finishes: the fixture calls `useStableStore(...)` and immediately uses the returned store in
  the same render pass (`packages/utils/src/store/ReactStore.test.tsx:30-32`), and state on the
  created instance is readable right after `render` completes
  (`packages/utils/src/store/ReactStore.test.tsx:36-37`).
- Instance identity is stable across prop-driven re-renders: a spy installed on the
  first-render instance's `update` method (guarded to install once,
  `packages/utils/src/store/ReactStore.test.tsx:161-163`) still intercepts calls after two
  subsequent `setProps` re-renders — the call count stays at 1 across a no-change re-render
  (`packages/utils/src/store/ReactStore.test.tsx:171-177`) and increments to 2 after a
  value-changing re-render, with the state update landing on the same instance
  (`packages/utils/src/store/ReactStore.test.tsx:179-184`). If the hook re-initialized on
  re-render, the spied first-render instance would be orphaned and the count would stay at 1.
- UNVERIFIED — inferred from `packages/utils/src/useRefWithInit.ts:18-19`, no test asserts
  this: that `init` is invoked exactly once in total (no test spies on the init factory
  itself).

## Keyboard interactions

N/A — non-visual utility. No keyboard behavior is implemented, and no test asserts any.

## Focus management

N/A — no focus behavior is implemented, and no test asserts any.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented, and no test asserts any.

## DOM structure & portal behavior

N/A — the utility creates no DOM, renders nothing, and performs no portal behavior. It is a
pure ref-box wrapper over `React.useRef` (`packages/utils/src/useRefWithInit.ts:16-22`).
UNVERIFIED — no test asserts this (the detected test renders components returning `null` or a
plain `<output>` and never inspects DOM for the hook's sake,
`packages/utils/src/store/ReactStore.test.tsx:33,328`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no DOM events are emitted, dispatched, or observed; no bubbling or `preventDefault`
semantics exist. The only user callback is the `init` factory, which is invoked during render
(not in response to an event) with at most one argument: `init(initArg)`
(`packages/utils/src/useRefWithInit.ts:19`); its return value becomes the ref's `.current`.
UNVERIFIED — no test asserts the argument forwarding; the detected tests only use zero-argument
factories (`packages/utils/src/store/ReactStore.test.tsx:12`).

## Edge cases (rapid interactions, unmount, nesting)

- Rapid re-renders / prop churn: no re-initialization — the sentinel guard makes later renders
  no-ops with respect to `init`, so rapidly changing props, parents, or context cannot recreate
  the instance (`packages/utils/src/useRefWithInit.ts:18-20`). Test-proven for the
  identity-stability part across `setProps` re-renders
  (`packages/utils/src/store/ReactStore.test.tsx:171-184`).
- Sentinel collision is impossible from caller code: `UNINITIALIZED` is a module-private fresh
  object (`packages/utils/src/useRefWithInit.ts:4`) and is not exported, so no `init` return
  value can equal it; the guard is therefore only true before the first initialization.
  UNVERIFIED — inferred from source, no test asserts this.
- Throwing `init`: if the factory throws during the initializing render, the ref remains
  `UNINITIALIZED`, so the next render would re-invoke `init` (there is no error latch). There
  is no try/catch in the hook (`packages/utils/src/useRefWithInit.ts:15-22`). UNVERIFIED —
  inferred from source, no test asserts this.
- Unmount/remount: the hook registers no effects and no cleanup
  (`packages/utils/src/useRefWithInit.ts:15-22`); on unmount the ref and its instance are
  simply garbage-collected, and a remount creates a fresh ref with a fresh `init` call (plain
  React ref semantics). UNVERIFIED — inferred from source, no test asserts this.
- StrictMode double render: no detected test renders the hook under StrictMode (the
  identity-stability test explicitly renders with `{ strict: false }`,
  `packages/utils/src/store/ReactStore.test.tsx:169`); StrictMode double-invocation behavior of
  `init` is therefore UNVERIFIED — no test asserts this.
- Nesting / multiple consumers: independent per call site — each component instance holds its
  own ref, so nested or sibling consumers do not share the initialized value
  (`packages/utils/src/useRefWithInit.ts:16`). UNVERIFIED — inferred from source; consistent
  with the detected tests where separately rendered fixtures hold separate stores
  (`packages/utils/src/store/ReactStore.test.tsx:53-66,120-133`).
- No memoization dependencies: unlike `useMemo`, there is no dependency array — the optional
  `initArg` is passed only on the first initialization and later changes to it are ignored
  (the guard skips `init` entirely, `packages/utils/src/useRefWithInit.ts:18-20`).
  UNVERIFIED — inferred from source, no test asserts this.

## Shared harness dependencies

- The detected test file imports the external npm harness `@mui/internal-test-utils` for
  `act`, `createRenderer`, and `screen`
  (`packages/utils/src/store/ReactStore.test.tsx:3`) plus `vitest` for
  `expect`/`vi`/`describe`/`it` (`packages/utils/src/store/ReactStore.test.tsx:1`). This is an
  npm dependency resolved from `node_modules`, not a repo-local `#test-utils` or
  `packages/react/test/` file, so there is no in-repo harness file to read for this unit.
- No repo-local shared harness (`#test-utils`, `packages/react/test/*`) is imported by the
  detected test file; its remaining imports are the unit under test and sibling source modules
  (`./ReactStore`, `../useRefWithInit`, `../fastHooks`, `./createSelector`,
  `packages/utils/src/store/ReactStore.test.tsx:4-7`).
