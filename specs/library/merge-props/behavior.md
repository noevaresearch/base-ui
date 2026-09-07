# mergeProps — behavior spec

Mined from `packages/react/src/merge-props/mergeProps.test.ts` (the unit's entire test suite: 599 lines). The unit is a headless props-merging utility — no component, no rendering — so the structural sections are N/A. The `TODO.md` entry for `infra: merge-props` has no `wraps-external:` field, so no external-package delegation applies.

Cross-cutting precedence rule proven across the suite: with `mergeProps(P1, P2, P3)`, the LAST argument has highest precedence — its scalar props win outright `packages/react/src/merge-props/mergeProps.test.ts:400-412`, its `style` values win per key `packages/react/src/merge-props/mergeProps.test.ts:154-167`, its `className` is placed first in the merged string `packages/react/src/merge-props/mergeProps.test.ts:190-200`, and its event handlers execute first and may veto the remaining (earlier-argument) handlers via `event.preventBaseUIHandler()` `packages/react/src/merge-props/mergeProps.test.ts:256-281`.

## Public API surface (props, parts, subcomponents)

- Exports `mergeProps` (variadic) and `mergePropsN` (array-taking), both imported from `@base-ui/react/merge-props` in the suite. `packages/react/src/merge-props/mergeProps.test.ts:2`
- `mergeProps<'button'>(propsA, propsB, ...)`: variadic; the generic parameter is an element type (tests use `'button'`, `'div'`, and `any`) used to type the merged result. Arguments are plain props objects and/or props-getter functions interleaved in any position. `packages/react/src/merge-props/mergeProps.test.ts:15`, `packages/react/src/merge-props/mergeProps.test.ts:33-49`, `packages/react/src/merge-props/mergeProps.test.ts:81-89`, `packages/react/src/merge-props/mergeProps.test.ts:442-452`
- `mergePropsN<'button'>([propsA, propsB, ...])`: same merging applied to a single array argument; proven equivalent for the lone-handler prevention behavior. `packages/react/src/merge-props/mergeProps.test.ts:119-129`
- Props getter: any function argument in the merge list. It is called exactly once per `mergeProps` invocation, receives the accumulated merge of all props-object arguments that PRECEDE it in the list, and its return value REPLACES that accumulated object before merging continues with the remaining arguments. `packages/react/src/merge-props/mergeProps.test.ts:435-456`, `packages/react/src/merge-props/mergeProps.test.ts:486-497`, `packages/react/src/merge-props/mergeProps.test.ts:514-530`
- Merged synthetic event handlers receive an event argument exposing `preventBaseUIHandler()` and `baseUIHandlerPrevented`; the `BaseUIEvent` type is imported (type-only) for typing these. `packages/react/src/merge-props/mergeProps.test.ts:3`, `packages/react/src/merge-props/mergeProps.test.ts:85-87`, `packages/react/src/merge-props/mergeProps.test.ts:422-424`
- Returns a fresh merged props object; source objects are not mutated (a getter-returned object reused across a merge keeps its original values). `packages/react/src/merge-props/mergeProps.test.ts:499-512`
- Every test passes at least two arguments; single-argument calls are not covered. UNVERIFIED — inferred from `packages/react/src/merge-props/mergeProps.test.ts:1-599`, no test asserts this.

## State model (controlled/uncontrolled, defaults, transitions)

N/A — stateless pure utility: no state, no defaults, no transitions. Each call is independent and deterministic; a props getter is invoked exactly once per call. `packages/react/src/merge-props/mergeProps.test.ts:454-455`

## Keyboard interactions

N/A — no keyboard usage or assertions exist anywhere in the suite. `packages/react/src/merge-props/mergeProps.test.ts:1-599`

## Focus management

N/A — no focus-related code or assertions exist in the suite. `packages/react/src/merge-props/mergeProps.test.ts:1-599`

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is asserted by any test. `role` and `id` appear only as ordinary scalar props flowing through the merge (e.g. a getter receives `{ role: 'tab', className: 'test-class' }` after `role: 'tab'` from a later argument overrides `role: 'button'`). `packages/react/src/merge-props/mergeProps.test.ts:458-484`

## DOM structure & portal behavior

N/A — the unit never renders and has no DOM/portal surface: tests invoke merged handlers and inspect returned prop values directly, mounting nothing. `packages/react/src/merge-props/mergeProps.test.ts:15-19`, `packages/react/src/merge-props/mergeProps.test.ts:161-166`

## Events (names, payload shape, bubbling, preventDefault semantics)

Handler chaining and order:

- All handlers supplied for the same key are chained into one function; every underlying handler is invoked exactly once per merged invocation. `packages/react/src/merge-props/mergeProps.test.ts:24-27`
- Execution order is last-argument-first: with `mergeProps(A, B, C)` the handlers run C, then B, then A (log `['1', '2', '3']`). `packages/react/src/merge-props/mergeProps.test.ts:30-53`
- `undefined` handler entries are skipped without breaking the chain: `mergeProps(A, { onClick: undefined }, C)` runs C then A. `packages/react/src/merge-props/mergeProps.test.ts:55-76`

Prevention semantics (synthetic event handlers):

- A merged handler's event argument exposes `preventBaseUIHandler()`; calling it synchronously sets `event.baseUIHandlerPrevented = true`, readable in the same handler immediately after the call. `packages/react/src/merge-props/mergeProps.test.ts:78-94`, `packages/react/src/merge-props/mergeProps.test.ts:414-432`
- Once called, every handler merged EARLIER in the argument list (i.e. scheduled to run after the caller) is skipped: when the last of three handlers calls it, earlier handlers never run (`ran` stays `false`) `packages/react/src/merge-props/mergeProps.test.ts:256-281`; with the call in the middle handler, execution is `['0', '1']` and the first argument's handler is skipped. `packages/react/src/merge-props/mergeProps.test.ts:283-308`
- Without `preventBaseUIHandler()`, all chained handlers run. `packages/react/src/merge-props/mergeProps.test.ts:237-254`
- Prevention applies regardless of handler position or chain length: a lone handler `packages/react/src/merge-props/mergeProps.test.ts:78-94`, a first-position handler whose other arguments only set non-handler props like `id` `packages/react/src/merge-props/mergeProps.test.ts:96-114`, and less common keys like `onContextMenu` `packages/react/src/merge-props/mergeProps.test.ts:136-152`. The `mergePropsN` array form behaves identically. `packages/react/src/merge-props/mergeProps.test.ts:116-134`
- Relationship to native `event.preventDefault()` / `nativeEvent` is not asserted anywhere. UNVERIFIED — inferred from `packages/react/src/merge-props/mergeProps.test.ts:1-599`, no test asserts this.

Non-standard (non-DOM) event handlers:

- Non-DOM callbacks (e.g. `onValueChange`, `onOpenChange`) are chained the same way (last argument first) and never error, even when the first invocation argument is `true`, `13`, `'newValue'`, an object, an array, or a function. `packages/react/src/merge-props/mergeProps.test.ts:310-331`
- All invocation arguments are forwarded verbatim to every chained handler: a lone handler receives exactly `(true, eventDetails)` `packages/react/src/merge-props/mergeProps.test.ts:333-347`; merged handlers each receive `(true, eventDetails)` in last-argument-first order. `packages/react/src/merge-props/mergeProps.test.ts:349-372`
- Synthetic-event handlers also support additional arguments after the event: `(event, details)` is forwarded to each chained handler. `packages/react/src/merge-props/mergeProps.test.ts:374-398`
- Whether `preventBaseUIHandler()` applies to non-standard handlers is not tested (no test calls it on one). UNVERIFIED — inferred from `packages/react/src/merge-props/mergeProps.test.ts:1-599`.
- The criterion that classifies a key/handler as "synthetic" (e.g. `onClick`, `onMouseDown`, `onContextMenu`) vs "non-standard" (e.g. `onValueChange`, `onOpenChange`) is not asserted by any test. UNVERIFIED — inferred from `packages/react/src/merge-props/mergeProps.test.ts:1-599`.

Payload shape:

- Tests pass `{ nativeEvent: new MouseEvent('click') }`-shaped payloads (cast `as any`) and assert only on the augmented members (`preventBaseUIHandler`, `baseUIHandlerPrevented`); nothing is asserted about other event fields. `packages/react/src/merge-props/mergeProps.test.ts:17`, `packages/react/src/merge-props/mergeProps.test.ts:91`, `packages/react/src/merge-props/mergeProps.test.ts:277`, `packages/react/src/merge-props/mergeProps.test.ts:429`
- Bubbling: N/A — no DOM events are dispatched; handlers are invoked directly. `packages/react/src/merge-props/mergeProps.test.ts:17-19`

## Edge cases (rapid interactions, unmount, nesting)

- Rapid interactions / unmount: N/A — no DOM, lifecycle, or timing behavior exists to exercise; the suite invokes merged functions synchronously. `packages/react/src/merge-props/mergeProps.test.ts:17-19`, `packages/react/src/merge-props/mergeProps.test.ts:91`
- Arbitrary non-event values as the first handler argument are tolerated without error (all chained handlers still run). `packages/react/src/merge-props/mergeProps.test.ts:310-331`
- `undefined` handler slots and `undefined` `style`/`className` values are handled by skipping or passthrough, never producing an empty object or stray whitespace: `style` and `className` stay `undefined` when both sides lack them. `packages/react/src/merge-props/mergeProps.test.ts:55-76`, `packages/react/src/merge-props/mergeProps.test.ts:182-188`, `packages/react/src/merge-props/mergeProps.test.ts:229-235`
- Getter nesting/wrapping: a getter can capture the props accumulated before it (including handlers) and return new handlers that call the captured ones. A getter handler that calls `preventBaseUIHandler()` and then MANUALLY invokes the captured handler with a fresh event object runs it anyway — manual re-invocation bypasses the automatic prevention of the merged chain (log `['last-handler', 'getter-handler', 'first-handler']`). `packages/react/src/merge-props/mergeProps.test.ts:532-563`
- The `baseUIHandlerPrevented` flag lets a getter handler opt into respecting prevention before manually re-invoking captured handlers (log `['last-handler', 'getter-handler']`). `packages/react/src/merge-props/mergeProps.test.ts:565-597`
- A getter's returned object is never mutated by subsequent merging (a reused shared object keeps `className: 'base'`). `packages/react/src/merge-props/mergeProps.test.ts:499-512`
- A getter positioned last REPLACES the accumulated props with its return value — earlier props (`id`) do not survive into the result. `packages/react/src/merge-props/mergeProps.test.ts:514-530`
- A getter with no preceding props-object arguments receives an empty object — not the props that follow it. `packages/react/src/merge-props/mergeProps.test.ts:486-497`

## Shared harness dependencies

- None. The suite imports only `vitest` primitives (`expect`, `vi`, `describe`, `it`), the unit under test (`@base-ui/react/merge-props`), and a type-only `BaseUIEvent` import. No `#test-utils` or `packages/react/test/` harness file is used. `packages/react/src/merge-props/mergeProps.test.ts:1-3`
