# useMergedRefs — behavior spec

Unit: `useMergedRefs` (a React hook in `packages/utils/src`). The `TODO.md` entry for this unit
(TODO.md:179-183) has no `wraps-external:` field and no `needs-batched-mining:` field, so no
third-party delegation applies and the whole unit is mined from its single test file:
`packages/utils/src/useMergedRefs.test.tsx` (174 lines: one top-level `describe('useMergedRefs')`
with three tests plus one nested `describe('changing refs')` with two tests, and one standalone
`test(...)` for cleanup semantics).

## Public API surface (props, parts, subcomponents)

- Not a component — a React hook. It has no parts, no subcomponents, and no rendered output of
  its own. The only export exercised is the named `useMergedRefs`
  `packages/utils/src/useMergedRefs.test.tsx:5`.
- Only the two-argument form is exercised: `useMergedRefs(a, b)`
  `packages/utils/src/useMergedRefs.test.tsx:19`,
  `packages/utils/src/useMergedRefs.test.tsx:86`,
  `packages/utils/src/useMergedRefs.test.tsx:151`. Any additional arities or exports are
  UNVERIFIED — inferred from `packages/utils/src/useMergedRefs.test.tsx:5`, no test asserts them.
- Each argument is a ref "branch" and all of these flavors are proven to be accepted:
  - Ref objects from `React.createRef`
    `packages/utils/src/useMergedRefs.test.tsx:24`,
    `packages/utils/src/useMergedRefs.test.tsx:98`,
    `packages/utils/src/useMergedRefs.test.tsx:106-108`.
  - Callback refs (plain functions, including a `useState` setter used directly as a ref)
    `packages/utils/src/useMergedRefs.test.tsx:17-19`,
    `packages/utils/src/useMergedRefs.test.tsx:35`.
  - Nullish/absent branches: a forwarded `ref` never populated by the caller
    `packages/utils/src/useMergedRefs.test.tsx:33-36`, and the result of `getReactElementRef`
    on a child element that carries no ref
    `packages/utils/src/useMergedRefs.test.tsx:59-66`.
- The return value is a single ref-setter function that consumers assign directly to a host
  element's JSX `ref` prop `packages/utils/src/useMergedRefs.test.tsx:21`,
  `packages/utils/src/useMergedRefs.test.tsx:39`,
  `packages/utils/src/useMergedRefs.test.tsx:88`,
  `packages/utils/src/useMergedRefs.test.tsx:152`.
- The canonical usage shape is merging an externally supplied ref with an internal one so both
  observe the same node `packages/utils/src/useMergedRefs.test.tsx:15-21`,
  `packages/utils/src/useMergedRefs.test.tsx:33-36`.

## State model (controlled/uncontrolled, defaults, transitions)

N/A for controlled/uncontrolled — this is a stateless utility hook with no options object, no
defaults, and no internal observable state. The proven behavior is an input-driven lifecycle:

- Mount: the node is assigned to every provided branch — a `createRef` branch ends up populated
  `packages/utils/src/useMergedRefs.test.tsx:29`, a state-setter branch is invoked (the rendered
  text flips to "has a ref") `packages/utils/src/useMergedRefs.test.tsx:21` +
  `packages/utils/src/useMergedRefs.test.tsx:29`, and each callback branch is invoked exactly
  once `packages/utils/src/useMergedRefs.test.tsx:157-162`.
- Mount does not invoke cleanup functions returned by callback refs — cleanup call count is 0
  after attach `packages/utils/src/useMergedRefs.test.tsx:159`.
- Ref-input change between renders (via re-render): a branch ref added after initial mount
  receives the node `packages/utils/src/useMergedRefs.test.tsx:98-102`; a detached branch ref is
  cleaned up (its `.current` becomes `null`)
  `packages/utils/src/useMergedRefs.test.tsx:119-123`; branch refs whose identity did not change
  keep holding the node `packages/utils/src/useMergedRefs.test.tsx:121`.
- Unmount: a branch whose callback returned a cleanup function has that cleanup invoked exactly
  once `packages/utils/src/useMergedRefs.test.tsx:164-167`, and its callback is not invoked again
  (its null path never fires) `packages/utils/src/useMergedRefs.test.tsx:166`; a branch whose
  callback returned no cleanup is invoked once with `null` on unmount (the shared null-handler
  mock fires exactly once) `packages/utils/src/useMergedRefs.test.tsx:169-172` and is not called
  again after that `packages/utils/src/useMergedRefs.test.tsx:170`.

## Keyboard interactions

N/A — no keyboard events are exercised anywhere in this unit's tests.

## Focus management

N/A — no test exercises focus; the hook has no focus behavior of its own.

## Accessibility (roles, aria-*, id linking)

N/A — no roles, aria attributes, or id linkages are asserted; the hook renders nothing itself.

## DOM structure & portal behavior

N/A — no portal and no wrapper DOM. The merged ref-setter is only ever attached to plain host
`<div>` elements owned by the test components
`packages/utils/src/useMergedRefs.test.tsx:21`,
`packages/utils/src/useMergedRefs.test.tsx:39`,
`packages/utils/src/useMergedRefs.test.tsx:88`,
`packages/utils/src/useMergedRefs.test.tsx:152` (or forwarded through
`React.cloneElement` `packages/utils/src/useMergedRefs.test.tsx:61`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — the hook emits and consumes no DOM events; there are no event names, bubbling, or
preventDefault semantics. Its only outward channel is the ref-invocation protocol: RefObject
assignment, callback-ref invocation, and cleanup invocation. Proven payload shape: a callback-ref
branch is invoked with the mounted DOM node on attach (asserted via the received value's
`id === 'test'`, matching the rendered `<div id="test">`)
`packages/utils/src/useMergedRefs.test.tsx:152` +
`packages/utils/src/useMergedRefs.test.tsx:157`, and with `null` on the no-cleanup unmount path
`packages/utils/src/useMergedRefs.test.tsx:169-172`. Attach-time invocation happens exactly once
per branch `packages/utils/src/useMergedRefs.test.tsx:158` +
`packages/utils/src/useMergedRefs.test.tsx:161-162`.

## Edge cases (rapid interactions, unmount, nesting)

- Nullish branches are tolerated silently: merging a branch that resolves to nothing (a child
  element with no ref) renders without error and without dev warnings
  `packages/utils/src/useMergedRefs.test.tsx:68-74`; a forwarded ref never populated by the
  caller also renders cleanly while the remaining callback branch still fires
  `packages/utils/src/useMergedRefs.test.tsx:45-49`.
- Every covered render asserts `not.toErrorDev()` — mounting with refs, forking to a single
  live branch, forking to no live branches, and ref changes all emit no dev warnings
  `packages/utils/src/useMergedRefs.test.tsx:26-28`,
  `packages/utils/src/useMergedRefs.test.tsx:45-47`,
  `packages/utils/src/useMergedRefs.test.tsx:68-74`,
  `packages/utils/src/useMergedRefs.test.tsx:94-101`,
  `packages/utils/src/useMergedRefs.test.tsx:111-113`.
- Ref churn mid-lifecycle: adding a branch after mount picks the node up
  `packages/utils/src/useMergedRefs.test.tsx:98-102`; swapping a branch detaches (nulls) only
  the swapped branch while untouched branches are unaffected
  `packages/utils/src/useMergedRefs.test.tsx:119-123`.
- Unmount is exercised only through the cleanup-protocol test, rendered with the harness option
  `strict: false` `packages/utils/src/useMergedRefs.test.tsx:155`, so no StrictMode
  double-invocation is in play for those assertions.
- Not covered — no behavior asserted for: rapid successive ref changes (each test performs at
  most one change), nesting merged refs inside each other, a callback ref throwing, or
  interacting instances. Any such behavior is UNVERIFIED — inferred from
  `packages/utils/src/useMergedRefs.test.tsx:91-124`, no test asserts it.

## Shared harness dependencies

- `@mui/internal-test-utils` (npm package), imported for `createRenderer`, `MuiRenderResult`,
  and `screen` `packages/utils/src/useMergedRefs.test.tsx:3`; `createRenderer()` provides the
  environment-aware `render` used throughout `packages/utils/src/useMergedRefs.test.tsx:8`.
  Re-renders are driven by `view.setProps`
  `packages/utils/src/useMergedRefs.test.tsx:100`,
  `packages/utils/src/useMergedRefs.test.tsx:119`; unmount by the `unmount` returned from
  `render` `packages/utils/src/useMergedRefs.test.tsx:155` +
  `packages/utils/src/useMergedRefs.test.tsx:164`; DOM queries via `screen.getByTestId`
  `packages/utils/src/useMergedRefs.test.tsx:49`. The `toErrorDev` / `not.toErrorDev` matchers
  used for the dev-warning assertions are provided by this harness's chai plugin (asserting on
  captured `console.error` output; `node_modules` files are not cited since they are not stable
  across installs).
- `./getReactElementRef` (sibling util in `packages/utils/src`), imported by the test
  `packages/utils/src/useMergedRefs.test.tsx:4` and used at
  `packages/utils/src/useMergedRefs.test.tsx:59` to derive a ref branch from the test's child
  element. Its own behavior is specified separately in `specs/utils/getReactElementRef.md`; only
  its role as a ref-branch source is relevant here.
- No `#test-utils` alias or `packages/react/test/` harness files are imported by this unit's
  tests.
