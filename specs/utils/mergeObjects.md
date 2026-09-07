# `mergeObjects` — behavior spec

Unit: `packages/utils/src/mergeObjects` (Phase A util → crate `leptos-ui-utils`).

There is no dedicated `mergeObjects.test.ts`. The unit is a tiny pure helper whose observable
behavior is proven entirely through its two consumer suites: `packages/react/src/merge-props/mergeProps.test.ts`
(which delegates `style` merging to `mergeObjects`, per `packages/react/src/merge-props/mergeProps.ts:167-170`)
and `packages/react/src/internals/useRenderElement.test.tsx` (which delegates state merging and
`style` merging to it, per `packages/react/src/internals/useRenderElement.tsx:86` and
`packages/react/src/internals/useRenderElement.tsx:115`). The `TODO.md` entry for this unit has no
`wraps-external:` field, so no external-package delegation applies; it is not flagged for batched
mining.

## Public API surface (props, parts, subcomponents)

- Single named export `mergeObjects`, a pure two-argument function; no components, no props object,
  no parts, no subcomponents (`packages/utils/src/mergeObjects.ts:1-4`).
- Both parameters are typed as `object | undefined`, and the return value is either an object or
  `undefined` (`packages/utils/src/mergeObjects.ts:1-4`). No test imports the unit directly, so the
  generic signature itself is UNVERIFIED — inferred from `packages/utils/src/mergeObjects.ts:1-2`,
  no test asserts the type-level contract.
- As exercised, the function receives possibly-`undefined` style objects from both sides of a merge
  and returns a style object (`packages/react/src/merge-props/mergeProps.test.ts:154-167`), and it
  is also applied once to whole prop bags during state/props resolution
  (`packages/react/src/internals/useRenderElement.tsx:86`, exercised end-to-end by
  `packages/react/src/internals/useRenderElement.test.tsx:462-475`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — stateless pure combinator with no state, defaults, or transitions. Every call's output is a
direct function of its two inputs, as shown by the four input combinations proven in Edge cases
below (`packages/react/src/merge-props/mergeProps.test.ts:154-188`,
`packages/react/src/internals/useRenderElement.test.tsx:126-134`).

## Keyboard interactions

N/A — non-visual utility. No test asserts any keyboard behavior.

## Focus management

N/A — no focus behavior is implemented or asserted by any test.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test.

## DOM structure & portal behavior

N/A — the utility creates no DOM and renders nothing. Its output does end up applied to real DOM
`style` attributes through its consumers: merged three-source styles are all visible on the rendered
element (`padding`, `color`, `fontSize`) in
`packages/react/src/internals/useRenderElement.test.tsx:462-475` and
`packages/react/src/internals/useRenderElement.test.tsx:477-490`.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no DOM events are emitted, dispatched, or asserted by any test. Event-handler merging is
deliberately NOT routed through `mergeObjects`: the style key is the only key `mergeProps` sends to
it (`packages/react/src/merge-props/mergeProps.ts:165-172`), and handler merging has separate
rightmost-first/prevention semantics proven independently in
`packages/react/src/merge-props/mergeProps.test.ts:6-28`.

## Edge cases (rapid interactions, unmount, nesting)

All four input branches are proven by tests:

- Both inputs defined → shallow right-wins merge: the right object's value overwrites the conflicting
  key (`color: 'red'` beats `color: 'blue'`) while left-only keys survive
  (`backgroundColor: 'blue'`), i.e. `{ ...a, ...b }` spread semantics
  (`packages/react/src/merge-props/mergeProps.test.ts:154-167`).
- Left input `undefined`, right defined → result equals the right object's content
  (`packages/react/src/merge-props/mergeProps.test.ts:169-180`).
- Left input defined, right `undefined` → result equals the left object's content unchanged, with no
  crash (internal `padding: 10px` survives when a style function returns `undefined`)
  (`packages/react/src/internals/useRenderElement.test.tsx:126-134`).
- Both inputs `undefined` → result is exactly `undefined` (`toBe(undefined)`)
  (`packages/react/src/merge-props/mergeProps.test.ts:182-188`).
- End-to-end in a real render, styles from three sources (internal props, component `style`, and
  render-element `style`) all coexist after chaining through the unit, including when the component
  style is a function of state
  (`packages/react/src/internals/useRenderElement.test.tsx:462-475`,
  `packages/react/src/internals/useRenderElement.test.tsx:477-490`).
- Frozen-object safety: with `EMPTY_OBJECT` as the state input the merge path does not throw in
  strict mode when a `style` object is provided, and the style still lands on the element
  (`packages/react/src/internals/useRenderElement.test.tsx:652-681`, specifically the style case at
  `packages/react/src/internals/useRenderElement.test.tsx:676-681`; the caller supplements the
  `undefined` result with `?? {}` per the comment at
  `packages/react/src/internals/useRenderElement.test.tsx:659-660`).
- Rapid interactions / unmount / nesting lifecycle: N/A — the utility has no lifecycle, component
  tree, or timing of its own, and no test covers call-pattern edge cases.

Unproven behaviors (must not be relied on as specified):

- Reference identity: whether the sole defined input is returned by reference without copying when
  the other side is `undefined`. UNVERIFIED — inferred from `packages/utils/src/mergeObjects.ts:5-10`,
  no test asserts identity (consumer assertions use value equality, e.g.
  `packages/react/src/merge-props/mergeProps.test.ts:177-179`).
- Deep merging: nested objects are not deep-merged (a top-level spread only). UNVERIFIED — inferred
  from `packages/utils/src/mergeObjects.ts:12`, no test covers nested style objects.
- Input non-mutation: no test asserts the input objects remain unmutated after the call. UNVERIFIED —
  inferred from `packages/react/src/merge-props/mergeProps.test.ts:154-167` (only the output is
  asserted).
- Other falsy inputs: only `undefined` is exercised as a "missing" side; behavior for `null` or other
  falsy values is UNVERIFIED — inferred from the truthiness checks at
  `packages/utils/src/mergeObjects.ts:5-9`, no test asserts this.

## Shared harness dependencies

- `packages/react/src/internals/useRenderElement.test.tsx` imports `createRenderer` from the shared
  `#test-utils` harness (resolved to `packages/react/test/index.ts`, which re-exports it from
  `packages/react/test/index.ts:3`), `reactMajor` from `@mui/internal-test-utils`
  (`packages/react/src/internals/useRenderElement.test.tsx:5`), and `EMPTY_OBJECT` from
  `@base-ui/utils/empty` (`packages/react/src/internals/useRenderElement.test.tsx:6`).
- `packages/react/src/merge-props/mergeProps.test.ts` has no harness dependency — it imports only
  `vitest` APIs and the public `@base-ui/react/merge-props` entry
  (`packages/react/src/merge-props/mergeProps.test.ts:1-2`).
