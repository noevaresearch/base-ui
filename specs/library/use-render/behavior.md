# useRender — behavior spec

Mined from the unit's full test suite: `packages/react/src/use-render/useRender.test.tsx` (single file, 314 lines). This is a headless rendering utility hook; it owns no UI of its own beyond the element it returns.

## Public API surface (props, parts, subcomponents)

- `useRender` is a hook that returns a single React element, intended to be returned directly from a component's body (`packages/react/src/use-render/useRender.test.tsx:16-22`).
- Parameters, as exercised via the `useRender.Parameters<{}, Element, undefined>` type (`packages/react/src/use-render/useRender.test.tsx:12`):
  - `render`: either a React element (`packages/react/src/use-render/useRender.test.tsx:112`) or a render function `(props, state) => ReactElement` (`packages/react/src/use-render/useRender.test.tsx:27-29`). Render functions receive `props` as their first argument and spread it onto their output element (`packages/react/src/use-render/useRender.test.tsx:27-29`). The contents of the second `state` argument are never asserted in this suite.
  - `props`: a props object forwarded onto the rendered element (`packages/react/src/use-render/useRender.test.tsx:18-20`, `packages/react/src/use-render/useRender.test.tsx:167-171`).
  - `ref`: accepts an array of refs; every ref in the array is attached to the rendered DOM element after mount (`packages/react/src/use-render/useRender.test.tsx:56-60`, `packages/react/src/use-render/useRender.test.tsx:69-71`). Only the array form is tested; the single-ref form is UNVERIFIED — inferred from `packages/react/src/use-render/useRender.test.tsx:56-60`, no test asserts this.
  - `defaultTagName`: a `keyof React.JSX.IntrinsicElements` fallback element type (`packages/react/src/use-render/useRender.test.tsx:85-91`).
  - `state`: a record of primitive values converted into `data-*` attributes on the rendered element (`packages/react/src/use-render/useRender.test.tsx:124-130`).
  - `stateAttributesMapping`: a per-key map of functions `(value) => attribute-record | null` that replace the default state→attribute conversion for those keys (`packages/react/src/use-render/useRender.test.tsx:297-301`).
- No subcomponents or parts: the entire public surface is the single hook plus its parameter types.

## State model (controlled/uncontrolled, defaults, transitions)

- `useRender` is stateless — there is no controlled/uncontrolled dichotomy and no internal state; the `state` parameter is caller-supplied data rendered as attributes, not state the hook manages (`packages/react/src/use-render/useRender.test.tsx:126-129`).
- Defaults: renders a `DIV` when neither `defaultTagName` nor `render` is provided (`packages/react/src/use-render/useRender.test.tsx:75-82`).
- Transition on re-render: changing the `defaultTagName` prop swaps the rendered element type (DIV → SPAN via `setProps`) (`packages/react/src/use-render/useRender.test.tsx:93-98`).
- Precedence: a `render` element overrides `defaultTagName`; once `render` is provided, subsequent `defaultTagName` changes have no effect on the output (SPAN remains SPAN while `defaultTagName` flips to `a`) (`packages/react/src/use-render/useRender.test.tsx:110-118`).
- Attribute-level precedence: a value in `props` wins over the same key's state-derived `data-*` attribute (`data-active="false"` from props beats `state.active = true`) (`packages/react/src/use-render/useRender.test.tsx:187-205`).

## Keyboard interactions

N/A — the hook only renders elements; no keyboard behavior is asserted anywhere in the suite.

## Focus management

N/A — no focus behavior is asserted anywhere in the suite.

## Accessibility (roles, aria-*, id linking)

N/A — no role or `aria-*` assertions exist in the suite. The only related proven behavior is generic attribute forwarding: `id` passed via `props` lands on the rendered element (`packages/react/src/use-render/useRender.test.tsx:167-183`).

## DOM structure & portal behavior

- Output is exactly one element, the container's `firstElementChild`, in every scenario (`packages/react/src/use-render/useRender.test.tsx:33-35`, `packages/react/src/use-render/useRender.test.tsx:80-82`).
- Default element is `DIV` (`packages/react/src/use-render/useRender.test.tsx:75-82`); `defaultTagName` selects the element otherwise (`packages/react/src/use-render/useRender.test.tsx:84-98`); a `render` element replaces it entirely (`packages/react/src/use-render/useRender.test.tsx:100-118`).
- Class composition: when `className` is unspecified in `props`, the render function receives it as undefined and its own composed class survives unmodified (`my-span ` with trailing space from the user's own template) — the hook does not inject or overwrite `className` for render functions (`packages/react/src/use-render/useRender.test.tsx:10-36`).
- Portal behavior: N/A — no portal usage is asserted in the suite.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no event handlers are registered or asserted in the suite. (Whether event handlers passed via `props` reach the rendered element is UNVERIFIED — inferred from `packages/react/src/use-render/useRender.test.tsx:167-183`, no test asserts this.)

## Edge cases (rapid interactions, unmount, nesting)

- Falsy state values produce no attribute: `false` boolean → `data-disabled` absent while `true` → `data-active=""` (present with empty value) (`packages/react/src/use-render/useRender.test.tsx:248-265`); number `0` → `data-count` absent (`packages/react/src/use-render/useRender.test.tsx:267-286`); `undefined` → `data-notdefined` absent (`packages/react/src/use-render/useRender.test.tsx:141-158`).
- Truthy numbers are stringified into attribute values, including decimals (`data-index="42"`, `data-percentage="99.9"`) (`packages/react/src/use-render/useRender.test.tsx:267-286`).
- State-derived `data-*` attributes coexist with `props` attributes (`className`, `id`, `data-existing` all preserved alongside `data-form` from state) (`packages/react/src/use-render/useRender.test.tsx:160-185`).
- Empty `state: {}` renders no `data-*` attributes while `props` still apply (`packages/react/src/use-render/useRender.test.tsx:207-226`).
- `state: undefined` is safe; `props` apply normally (`packages/react/src/use-render/useRender.test.tsx:228-246`).
- `stateAttributesMapping` lets each state key produce custom attribute names and values (kebab-case `data-is-active=""`, `data-item-count="5"`, `data-user-name="John"`) (`packages/react/src/use-render/useRender.test.tsx:288-312`). The mapping's falsy/`null` branch (omitting an attribute) is UNVERIFIED — inferred from `packages/react/src/use-render/useRender.test.tsx:298`, no test asserts this.
- Multi-ref attachment: an array of refs all end up pointing at the same rendered element (`packages/react/src/use-render/useRender.test.tsx:38-72`).
- Rapid interactions, unmount, and nesting are not exercised in this suite (no such tests exist).

## Shared harness dependencies

- The suite imports only `createRenderer` from `#test-utils` (`packages/react/src/use-render/useRender.test.tsx:5`).
- `#test-utils` resolves to `packages/react/test/index.ts`, which re-exports `createRenderer` from `packages/react/test/createRenderer.ts` (`packages/react/test/index.ts:3`).
- `createRenderer` (`packages/react/test/createRenderer.ts:27-49`) wraps `@mui/internal-test-utils`'s `createRenderer`; its `render` awaits React `act` around the underlying render and returns augmented `rerender`/`setProps` helpers, where `setProps` re-renders via `React.cloneElement` (`packages/react/test/createRenderer.ts:31-43`).
- No other harness files (pointer utils, `mergeRefs`, conformance suites, etc.) are used by this suite.
