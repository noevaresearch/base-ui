# `useId` behavior spec

Unit: `packages/utils/src/useId.ts` (hook). Mined from `packages/utils/src/useId.test.tsx` only.

## Public API surface (props, parts, subcomponents)

- Single hook: `useId(id?: string, prefix?: string): string`.
  - First argument is an externally supplied ID; when provided, the hook returns it verbatim — `useId('some-id')` yields `some-id` and a later prop update to `another-id` is returned as-is (`packages/utils/src/useId.test.tsx:13-26`).
  - Second argument is a prefix applied to the generated ID: `useId(undefined, 'base-ui')` returns an ID beginning with `base-ui-` (`packages/utils/src/useId.test.tsx:102-123`).
- Return value is a plain string usable in string composition: consumers build suffixed IDs via `` `${id}-label` `` (`packages/utils/src/useId.test.tsx:42-62`).
- No props, parts, or subcomponents — it is a non-rendering hook.
- Note: tests only exercise the hook through `renderToString(...).hydrate()` and `render(...)`, so behavior is proven for both SSR-hydrated and client-rendered usage (`packages/utils/src/useId.test.tsx:18-19`, `:33-34`, `:56`).

## State model (controlled/uncontrolled, defaults, transitions)

- Uncontrolled default: with no first argument, the hook generates a non-empty ID (`packages/utils/src/useId.test.tsx:28-40`).
- Controlled override: passing a first argument replaces the generated value; the element's `id` equals the prop after the update (`packages/utils/src/useId.test.tsx:36-39`).
- Transition generated → provided: a component that initially rendered with no ID picks up the newly supplied prop on the next render (`setProps({ id: 'another-id' })` → element id is `another-id`) (`packages/utils/src/useId.test.tsx:38-39`).
- UNVERIFIED — inferred from `packages/utils/src/useId.test.tsx:13-40`, no test asserts: switching back from a provided ID to `undefined` (reverting to a generated ID), or ID stability across re-renders that do not change props.

## Keyboard interactions

N/A — headless hook, no DOM element or event handling of its own (`packages/utils/src/useId.test.tsx:10-123`).

## Focus management

N/A — the hook returns a string only; no focus logic is exercised by any test (`packages/utils/src/useId.test.tsx:10-123`).

## Accessibility (roles, aria-*, id linking)

- The returned ID is intended for ARIA IDREF wiring: a test wires `aria-labelledby` on one element to another element's `id` produced by the same hook call, and the attributes match (`packages/utils/src/useId.test.tsx:42-62`).
- Multiple independent calls in one component produce distinct IDs that can be combined into a single space-separated IDREF list: `aria-labelledby={`${labelPartA} ${labelPartB}`}` matches the two label elements' ids (`packages/utils/src/useId.test.tsx:64-87`).
- Prefixed IDs remain valid for the same ARIA linking pattern (`aria-labelledby` ↔ `id`) (`packages/utils/src/useId.test.tsx:102-123`).
- No test asserts role assignment or any other ARIA attribute; the hook itself imposes none.

## DOM structure & portal behavior

N/A — the hook renders nothing; tests attach the returned string to plain `<span>` elements rendered by the test components (`packages/utils/src/useId.test.tsx:16`, `:31`, `:49-54`). No portal usage appears in tests.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are emitted, listened to, or tested (`packages/utils/src/useId.test.tsx:10-123`).

## Edge cases (rapid interactions, unmount, nesting)

- Server rendering: the hook provides a non-empty ID during `renderToString` (SSR), verified by a test explicitly named for React 18 (`packages/utils/src/useId.test.tsx:89-100`).
- The server test is skipped when `React.useId === undefined` (`packages/utils/src/useId.test.tsx:90-92`) — the suite gates server behavior on React's own `useId` availability. UNVERIFIED — inferred from `packages/utils/src/useId.test.tsx:90-92`, no test asserts the implementation delegates to React's `useId`, but the gate indicates server-side generation is tied to React's built-in capability.
- Concurrent instances: two `useId()` calls in the same component (analogous to nested/sibling usage) do not collide; each yields a distinct ID (`packages/utils/src/useId.test.tsx:64-87`).
- UNVERIFIED — inferred from the file's coverage, no test asserts: behavior on unmount, rapid prop churn (beyond one `setProps` call), or stable ID identity across a client-only re-render.

## Shared harness dependencies

- `@mui/internal-test-utils` (npm workspace dep, `node_modules/@mui/internal-test-utils`):
  - `createRenderer()` provides `render` (Testing Library render) and `renderToString` returning `{ hydrate }` with a `setProps` helper for SSR + hydration flows (`packages/utils/src/useId.test.tsx:3`, `:11`, `:18-19`).
  - `screen` is re-exported Testing Library query API (`packages/utils/src/useId.test.tsx:3`).
  - For Stage 3: a Rust/Leptos port needs equivalents for SSR string-render + hydrate + prop-update testing; the hook's own behavior assertions reduce to "element id attribute equals expected string".
