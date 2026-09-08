# Separator behavior spec

Mined from `packages/react/src/separator/Separator.test.tsx` — the only test file in the unit. No
`wraps-external:` field exists in the separator `TODO.md` entry (`TODO.md:510-516`), so the
behavior below is derived entirely from the component's own tests plus the shared conformance
harness they invoke.

## Public API surface (props, parts, subcomponents)

- `Separator` is a single root component imported from `@base-ui/react/separator`; no
  subcomponents, parts, or context hooks are exercised by the tests.
  `packages/react/src/separator/Separator.test.tsx:3`
- Default root element is an HTML `div`, proven by the conformance option
  `refInstanceof: window.HTMLDivElement` combined with the ref-forwarding suite.
  `packages/react/src/separator/Separator.test.tsx:9-12`, `packages/react/test/conformanceTests/refForwarding.tsx:32-38`
- Props directly exercised by the tests:
  - `orientation` — accepts `'horizontal'` and `'vertical'`; the value is reflected to the
    `aria-orientation` attribute on the root. `packages/react/src/separator/Separator.test.tsx:19-27`
  - `render` — accepts a function and a JSX element (conformance suite uses a `div` custom
    element, wrapping allowed by default); the custom element becomes the root.
    `packages/react/test/conformanceTests/renderProp.tsx:41-59`, `packages/react/test/conformanceTests/renderProp.tsx:61-76`
  - `className` — string form applied to the root (`packages/react/test/conformanceTests/className.tsx:20-23`);
    function form resolves to a class name that merges with the render element's own class name
    (`packages/react/test/conformanceTests/renderProp.tsx:163-178`).
  - `style` — forwards to the default root and to function/JSX render elements.
    `packages/react/test/conformanceTests/propForwarding.tsx:82-95`, `packages/react/test/conformanceTests/propForwarding.tsx:97-112`, `packages/react/test/conformanceTests/propForwarding.tsx:114-127`
  - Arbitrary DOM props (`lang`, `data-*`, `data-testid`) and `ref` — forwarded to both the
    default and customized roots. `packages/react/test/conformanceTests/propForwarding.tsx:23-36`, `packages/react/test/conformanceTests/propForwarding.tsx:38-59`, `packages/react/test/conformanceTests/propForwarding.tsx:61-80`
- No test exercises any callback/event-handler prop; no controlled props exist for this unit's
  tests.

## State model (controlled/uncontrolled, defaults, transitions)

- No internal state machine is observable: the only prop-driven output change tested is
  `orientation` → `aria-orientation`; there is no controlled/uncontrolled prop pair, no default
  transitions, and no toggle-like behavior in the tests. `packages/react/src/separator/Separator.test.tsx:19-27`
- Default value of `orientation` (and whether `aria-orientation` is present at all when the prop
  is omitted): UNVERIFIED — the no-props test only asserts the role and visibility
  (`packages/react/src/separator/Separator.test.tsx:14-17`); no test asserts a default
  `aria-orientation`.

## Keyboard interactions

- N/A — no test asserts any keyboard behavior on `Separator`.

## Focus management

- N/A — no test asserts focusability, tab order, or focus movement; `Separator` is rendered and
  queried via role only. `packages/react/src/separator/Separator.test.tsx:14-17`

## Accessibility (roles, aria-*, id linking)

- Renders with `role="separator"` and is visible (passes `toBeVisible()`, so no hidden/removed
  semantics). `packages/react/src/separator/Separator.test.tsx:14-17`
- `aria-orientation` mirrors the `orientation` prop exactly for both supported values:
  `'horizontal'` → `aria-orientation="horizontal"`, `'vertical'` →
  `aria-orientation="vertical"`. `packages/react/src/separator/Separator.test.tsx:24`, `packages/react/src/separator/Separator.test.tsx:20-26`
- No `aria-*` id-linking (e.g. `aria-labelledby`, `aria-describedby`, `id`) is asserted anywhere
  in the tests. N/A for id linking; no test covers it.

## DOM structure & portal behavior

- Renders a single element in place (an HTML `div` by default). No wrapper element is added and
  no portal is used in any test. `packages/react/src/separator/Separator.test.tsx:9-12`, `packages/react/src/separator/Separator.test.tsx:14-17`
- With `render` as a function or JSX element, the custom element fully replaces the root and
  receives the forwarded props; the conformance wrapper (`<div data-testid="base-ui-wrapper">`)
  around it remains the outer node, proving `Separator` itself adds no extra wrapper.
  `packages/react/test/conformanceTests/renderProp.tsx:41-59`, `packages/react/test/conformanceTests/renderProp.tsx:61-76`, `packages/react/test/conformanceTests/renderProp.tsx:27-38`
- The ref attaches to the default root (`instanceof window.HTMLDivElement`) and merges with a
  render-element ref so both callers observe the same custom node.
  `packages/react/test/conformanceTests/refForwarding.tsx:32-38`, `packages/react/test/conformanceTests/renderProp.tsx:115-144`
- N/A for portal behavior — no test covers portaling.

## Events (names, payload shape, bubbling, preventDefault semantics)

- N/A — no test asserts event handler props, event dispatch, bubbling, or `preventDefault`
  behavior for `Separator`.

## Edge cases (rapid interactions, unmount, nesting)

- N/A — rapid interactions, unmount mid-interaction, and nesting are not covered by any test in
  this unit. The only repeat-render behavior exercised is the conformance suite's repeated
  renders of the same minimal element across independent test cases
  (`packages/react/src/separator/Separator.test.tsx:9-12`).

## Shared harness dependencies

- `#test-utils` (module alias `./test/index.ts`, defined at `packages/react/package.json:110`)
  provides `createRenderer` and `describeConformance` used by this test.
  `packages/react/src/separator/Separator.test.tsx:4`, `packages/react/test/index.ts:3-4`
- `describeConformance` (`packages/react/test/describeConformance.tsx:51-68`, suite named
  'Base UI component API' at `packages/react/test/describeConformance.tsx:70`) runs the full
  four-suite set for Separator — `propsSpread`, `refForwarding`, `renderProp`, `className` —
  because no `only`/`skip` options are passed (`packages/react/test/describeConformance.tsx:44-49`, `packages/react/test/describeConformance.tsx:55-60`).
- `createRenderer` (`packages/react/test/createRenderer.ts:27-49`) wraps
  `@mui/internal-test-utils`'s renderer in `act` and exposes promise-based `render` plus
  `rerender`/`setProps` helpers; the Separator test destructures `render` from it
  (`packages/react/src/separator/Separator.test.tsx:7`).
- Conformance sub-suites invoked for this unit: prop forwarding
  (`packages/react/test/conformanceTests/propForwarding.tsx:22-128`), ref forwarding
  (`packages/react/test/conformanceTests/refForwarding.tsx:31-39`), render prop
  (`packages/react/test/conformanceTests/renderProp.tsx:40-179`, custom element `div`,
  `wrappingAllowed` default true per `packages/react/test/conformanceTests/renderProp.tsx:14-19`),
  and className (`packages/react/test/conformanceTests/className.tsx:13-24`).
- `screen` comes from `@mui/internal-test-utils`; no project-local pointer/interaction helpers
  (`firePointer`, `useTestInteractions`, `isJSDOM` gates) are used by this unit's tests.
  `packages/react/src/separator/Separator.test.tsx:2`
