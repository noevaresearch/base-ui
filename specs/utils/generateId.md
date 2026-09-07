# `generateId` — behavior spec

Unit: `packages/utils/src/generateId` (Phase A util → crate `leptos-ui-utils`).
Source of truth: none — this unit has no test file. The generated unit manifest records
`testFiles: []` for it (`ralph/generated/utils.json:97-104`), and no
`generateId.test.*` exists anywhere in the repo. The unit's `TODO.md` entry has no
`wraps-external:` field (`TODO.md:59-63`), so there is no runtime delegation to a
third-party npm package and no external crate for Stage 3 to bind against; the Rust port
must implement the algorithm itself.

Unlike some sibling utils, `generateId` does have **indirect** test coverage: it is the id
generator for the toast subsystem, and the toast suite exercises it through its two
consumer call sites (`packages/react/src/toast/store.ts:167` and
`packages/react/src/toast/createToastManager.ts:30`). Those consumer tests are cited below,
clearly framed as indirect evidence; they never assert the util's own output format, so all
format-level claims remain UNVERIFIED.

## Public API surface (props, parts, subcomponents)

- Single module with a single named export `generateId`; no components, no props objects,
  no parts, no subcomponents. UNVERIFIED — inferred from
  `packages/utils/src/generateId.ts:2`, no test asserts this.
- Signature: one positional string argument `prefix`, returns a `string`. UNVERIFIED —
  inferred from `packages/utils/src/generateId.ts:2-4`, no test asserts this directly. The
  only runtime evidence of the return type is indirect: a toast added without an explicit
  id resolves through `generateId('toast')` and the manager test asserts the returned
  value `toBeTypeOf('string')` (`packages/react/src/toast/createToastManager.test.tsx:53-61`).
- Module shape: a 5-line file with one module-level `let counter = 0` binding and one
  exported function; no other exports, no configuration. UNVERIFIED — inferred from
  `packages/utils/src/generateId.ts:1-5`.
- Import surface: consumed in-repo as `@base-ui/utils/generateId`, which resolves through
  the wildcard entry `./*": "./src/*.ts"` in the `@base-ui/utils` export map
  (`packages/utils/package.json:12-16`). The in-repo consumers import exactly this
  specifier (`packages/react/src/toast/store.ts:2`,
  `packages/react/src/toast/createToastManager.ts:1`). Not re-exported from
  `@base-ui/react`; there is no test importing the util by name.

## State model (controlled/uncontrolled, defaults, transitions)

N/A — not a stateful component (no controlled/uncontrolled concepts, no defaults, no
transitions). However, the util carries **module-level mutable state**: a counter that is
incremented on every call and embedded in the returned id
(`packages/utils/src/generateId.ts:1-4`). UNVERIFIED — inferred from those lines, no test
asserts this. Consequences inferred from the implementation (all UNVERIFIED):

- The counter is shared across every `generateId` call in the process for the module's
  lifetime; it never resets and cannot be reset or read from outside the module
  (`packages/utils/src/generateId.ts:1-4`).
- The counter suffix makes successive calls monotonically distinct regardless of the
  random segment, so uniqueness within a process does not actually depend on
  `Math.random` (`packages/utils/src/generateId.ts:3-4`).
- There is no SSR/isomorphic guard: the counter and `Math.random` both execute at call
  time, so server and client render passes would not agree on generated ids. Contrast with
  `useId` in the same package, whose fallback comment explicitly avoids random values
  server-side (`packages/utils/src/useId.ts:14-18`). No test covers `generateId` under
  SSR.

## Keyboard interactions

N/A — non-visual id-generation utility. No keyboard behavior exists to assert.

## Focus management

N/A — no focus behavior exists to assert. No test links a `generateId`-produced id to any
focusable element.

## Accessibility (roles, aria-*, id linking)

N/A — the util itself has no accessibility surface. Indirectly, its output is used only as
toast identity (React keys and store lookup ids), not for aria linking: toast roots render
`<Toast.Root key={toast.id} ...>` (`packages/react/src/toast/useToastManager.test.tsx:65`),
and the toast suite's aria assertions (`aria-labelledby` / `aria-describedby`) all target
ids from `Toast.Title` / `Toast.Description` elements, never a generated toast id
(`packages/react/src/toast/root/ToastRoot.test.tsx:152-157`). No test ties `generateId`
output to any `aria-*` attribute.

## DOM structure & portal behavior

N/A — renders nothing and creates no DOM. Indirectly, generated ids become the React
`key` for rendered toast roots, so their distinctness matters to rendering (multiple
id-less toasts render fine — see Edge cases). UNVERIFIED for the keying claim — inferred
from `packages/react/src/toast/useToastManager.test.tsx:65`.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — emits no events. Consumer event flows that happen to carry generated ids (e.g.
`toastManager.add` returning the id, `onClose` callbacks firing once per toast) are
toast-manager behavior proven by the toast suite
(`packages/react/src/toast/createToastManager.test.tsx:790-841`), not util behavior.

## Edge cases (rapid interactions, unmount, nesting)

- Rapid repeated calls: five toasts added without explicit ids inside one event-handler
  burst are all rendered and all closed by `toastManager.close()`
  (`packages/react/src/toast/createToastManager.test.tsx:746-788`). This indirectly
  exercises `generateId` five times and proves consumer tracking stays intact; the test
  does NOT assert the five ids are distinct, so id distinctness under rapid calls is
  UNVERIFIED — inferred from `packages/utils/src/generateId.ts:3-4`, no test asserts this.
- Generated id as a stable handle: the id returned from an id-less `add` can later be
  used to `update` that exact toast (`packages/react/src/toast/createToastManager.test.tsx:324-370`)
  and to `close` it (`packages/react/src/toast/createToastManager.test.tsx:700-744`).
  Indirect proof that the same id value is returned to the caller and registered in
  consumer state.
- Id-less `ToastStore.addToast`: the store path also resolves through `generateId`; a
  toast added without an id is addressable later via the id read back from state, and its
  pause/resume timer behavior works under that generated id
  (`packages/react/src/toast/store.test.ts:251-271`).
- Explicit-id short-circuit: when `add` is given an explicit id, the manager upserts and
  returns that exact id (`packages/react/src/toast/createToastManager.test.tsx:63-127`).
  That `generateId` is skipped entirely on this path is UNVERIFIED — inferred from
  `packages/react/src/toast/createToastManager.ts:30` and
  `packages/react/src/toast/store.ts:167-179`; no test asserts the util is not called.
- Output format: the id embeds the caller's prefix, a random 4-character base36 segment,
  and the 1-based counter, joined with `-` (e.g. `toast-ab12-1`). UNVERIFIED — inferred
  from `packages/utils/src/generateId.ts:4`; no test asserts the format, prefix, segment
  length, or separator.
- Unmount/lifecycle: N/A — no React lifecycle; the util is a plain function. The module
  counter survives component unmounts (it outlives the React tree), which no test asserts.
  UNVERIFIED — inferred from `packages/utils/src/generateId.ts:1`.
- Nesting: N/A — no recursive or nested-call semantics beyond the shared counter.

## Shared harness dependencies

- `packages/react/src/toast/createToastManager.test.tsx` (an indirect consumer suite)
  imports the shared `#test-utils` harness (`createRenderer`, `isJSDOM`) — which resolves
  to `packages/react/test/index.ts`, exporting `createRenderer` from
  `packages/react/test/createRenderer.ts` among others — and the `@mui/internal-test-utils`
  package (`fireEvent`, `flushMicrotasks`, `screen`)
  (`packages/react/src/toast/createToastManager.test.tsx:4-5`).
- `packages/react/src/toast/store.test.ts` uses only `vitest` directly; no shared harness
  (`packages/react/src/toast/store.test.ts:1`).
- The unit itself has no test file, so it imposes no harness dependency of its own.
