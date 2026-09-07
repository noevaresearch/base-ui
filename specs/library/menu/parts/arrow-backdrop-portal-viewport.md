## Public API surface (props, parts, subcomponents)

- `Menu.Arrow` is a renderable subcomponent that forwards a ref to an `HTMLDivElement` and must be rendered inside `Menu.Root > Menu.Portal > Menu.Positioner > Menu.Popup` (open) to be exercised by conformance tests `packages/react/src/menu/arrow/MenuArrow.test.tsx:8-21`.
- `Menu.Backdrop` is a renderable subcomponent that forwards a ref to an `HTMLDivElement`, accepts `data-testid`, and can be rendered directly as a child of `Menu.Root` (open) `packages/react/src/menu/backdrop/MenuBackdrop.test.tsx:9-14`. It is also composed inside `Menu.Portal` alongside `Menu.Positioner`/`Menu.Popup` `packages/react/src/menu/backdrop/MenuBackdrop.test.tsx:22-27`.
- `Menu.Portal` is a renderable subcomponent that forwards a ref to an `HTMLDivElement` and accepts a `keepMounted` prop used in its conformance test `packages/react/src/menu/portal/MenuPortal.test.tsx:9`.
- `Menu.Viewport` is a renderable subcomponent that forwards a ref to an `HTMLDivElement`, accepts `children`, and accepts `data-testid` `packages/react/src/menu/viewport/MenuViewport.test.tsx:10-24`, `packages/react/src/menu/viewport/MenuViewport.test.tsx:371`.
- `Menu.Trigger` accepts a `payload` prop (arbitrary value, e.g. string or number) that is exposed to `Menu.Root`'s render-prop children as `payload`, and accepts `delay` and `openOnHover` props `packages/react/src/menu/backdrop/MenuBackdrop.test.tsx:19`, `packages/react/src/menu/viewport/MenuViewport.test.tsx:52-57`.
- `Menu.Root` supports a function-as-children (render prop) pattern receiving an object with a `payload` field reflecting the most recently activated trigger's `payload` `packages/react/src/menu/viewport/MenuViewport.test.tsx:50-74`.

## State model (controlled/uncontrolled, defaults, transitions)

- `Menu.Root`'s `payload` render-prop value transitions from `undefined`/previous value to a new trigger's `payload` when that trigger is clicked/activated, causing `Menu.Viewport`'s children to re-render accordingly `packages/react/src/menu/viewport/MenuViewport.test.tsx:80-93`.
- When a menu is opened via hover (`Menu.Trigger openOnHover`), the `Menu.Backdrop` element's inline style sets `pointerEvents` to `'none'` `packages/react/src/menu/backdrop/MenuBackdrop.test.tsx:16-34`.
- When a menu is opened via click (no `openOnHover`), the `Menu.Backdrop` element's inline `pointerEvents` style is not `'none'` `packages/react/src/menu/backdrop/MenuBackdrop.test.tsx:36-54`.
- `Menu.Viewport` renders its children inside a container marked with a `data-current` attribute by default (i.e., in the "current" state) `packages/react/src/menu/viewport/MenuViewport.test.tsx:26-45`.
- On switching the active trigger (and thus `payload`), the previous `data-current` container is replaced/remounted: a new element carries `data-current` and is a different DOM node than the prior one `packages/react/src/menu/viewport/MenuViewport.test.tsx:47-94`.
- UNVERIFIED — inferred from `packages/react/src/menu/viewport/MenuViewport.test.tsx:96-103`, no test asserts this: the transition/morphing behavior is gated behind a global flag `BASE_UI_ANIMATIONS_DISABLED`; tests toggle it in `beforeEach`/`afterEach` but this is test setup, not an assertion about component defaults outside of Chromium-only environment.
- During a trigger-to-trigger transition (Chromium-only, animations enabled), `Menu.Viewport` creates a second container marked `data-previous` in addition to the `data-current` one; the `data-previous` container retains the prior content, has an `inert` attribute, and exposes `--popup-width`/`--popup-height` CSS custom properties formatted as `"<number>px"` on its inline style `packages/react/src/menu/viewport/MenuViewport.test.tsx:105-196`.
- After the transition/animation completes, the `data-previous` container is removed from the DOM, leaving only the visible `data-current` container `packages/react/src/menu/viewport/MenuViewport.test.tsx:201-207`.
- Rapid successive trigger activations (click trigger1, trigger2, trigger3, trigger1 in quick succession) eventually settle on content matching the last-clicked trigger's payload, and that content becomes visible `packages/react/src/menu/viewport/MenuViewport.test.tsx:210-273`.
- `Menu.Viewport` computes and exposes a `data-activation-direction` attribute on itself once a transition between two triggers occurs; its value is a space-separated set of direction tokens drawn from `right`/`left`/`up`/`down`, determined by the relative screen position of the previously-active trigger and the newly-active trigger, with tolerance for small (~5px) positional differences (differences at or below tolerance omit that axis's token) `packages/react/src/menu/viewport/MenuViewport.test.tsx:275-403`.

## Keyboard interactions

N/A — none of the four test files exercise keyboard interactions for Arrow, Backdrop, Portal, or Viewport.

## Focus management

N/A — none of the four test files assert focus-management behavior for Arrow, Backdrop, Portal, or Viewport.

## Accessibility (roles, aria-*, id linking)

N/A — none of the four test files assert ARIA roles, aria-* attributes, or id-linking behavior for Arrow, Backdrop, Portal, or Viewport.

## DOM structure & portal behavior

- `Menu.Arrow` renders as (or forwards ref to) an `HTMLDivElement`, and is expected to be nested under `Menu.Popup` which is nested under `Menu.Positioner`, which is nested under `Menu.Portal`, which is nested under an open `Menu.Root` `packages/react/src/menu/arrow/MenuArrow.test.tsx:8-21`.
- `Menu.Backdrop` renders as (or forwards ref to) an `HTMLDivElement` `packages/react/src/menu/backdrop/MenuBackdrop.test.tsx:9-14`.
- `Menu.Portal` renders as (or forwards ref to) an `HTMLDivElement` and supports a `keepMounted` prop that keeps it conformant when tested standalone under `Menu.Root` `packages/react/src/menu/portal/MenuPortal.test.tsx:9-14`.
- `Menu.Viewport` renders as (or forwards ref to) an `HTMLDivElement` and is expected to be nested under `Menu.Popup` `packages/react/src/menu/viewport/MenuViewport.test.tsx:10-24`.
- Content passed as children to `Menu.Viewport` is rendered inside a descendant element queryable via `closest('[data-current]')`, i.e. `Menu.Viewport` wraps its children in an internal container carrying `data-current` `packages/react/src/menu/viewport/MenuViewport.test.tsx:33-45`.
- During cross-trigger transitions, `Menu.Viewport`'s internal structure can contain two sibling/descendant containers simultaneously: one marked `data-previous` (outgoing content, `inert`) and one marked `data-current` (incoming content) `packages/react/src/menu/viewport/MenuViewport.test.tsx:182-199`.
- An ancestor element (of the `data-previous`/`data-current` containers) is marked `data-transitioning` during the morph, as referenced by the CSS selectors used to drive the animation in the test's injected stylesheet `packages/react/src/menu/viewport/MenuViewport.test.tsx:110-124`. UNVERIFIED — inferred from `packages/react/src/menu/viewport/MenuViewport.test.tsx:110-124`, no test asserts the presence of `data-transitioning` directly (only the CSS rules reference it; the test does not query for this attribute).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — none of the four test files assert custom DOM event names, event payload shapes, bubbling, or `preventDefault` semantics for Arrow, Backdrop, Portal, or Viewport. (Interactions are driven via `user.hover`/`user.click` from the shared test harness, not via direct event-object assertions.)

## Edge cases (rapid interactions, unmount, nesting)

- Rapid, successive trigger clicks (trigger1 -> trigger2 -> trigger3 -> trigger1, with no waiting between clicks) do not leave `Menu.Viewport` in an inconsistent state; the content for the last-clicked trigger's payload is eventually found and visible `packages/react/src/menu/viewport/MenuViewport.test.tsx:210-273`.
- After a morph transition finishes, the outgoing `data-previous` container is cleaned up (removed from the DOM) rather than persisting `packages/react/src/menu/viewport/MenuViewport.test.tsx:201-207`.
- `Menu.Viewport`'s "current" container is remounted (a new DOM node, not attribute-mutated in place) when the active trigger/payload changes, confirmed by identity comparison (`secondContainer` !== `firstContainer`) `packages/react/src/menu/viewport/MenuViewport.test.tsx:88-93`.
- Direction-calculation edge case: when the vertical (or horizontal) offset between two triggers is within a small tolerance (tested at 2px), that axis is omitted from `data-activation-direction`, and when both axes are within tolerance (tested at 2px both), no direction tokens are present at all `packages/react/src/menu/viewport/MenuViewport.test.tsx:289-305`.
- The morphing/direction-calculation tests are explicitly skipped in the JSDOM environment (`describe.skipIf(isJSDOM)`), i.e. they only run under a real browser (Chromium) environment `packages/react/src/menu/viewport/MenuViewport.test.tsx:96`.

## Shared harness dependencies

- `#test-utils` (resolves to `packages/react/test/index.ts`), providing `createRenderer`, `describeConformance`, and `isJSDOM`, imported by all four test files `packages/react/src/menu/arrow/MenuArrow.test.tsx:3`, `packages/react/src/menu/backdrop/MenuBackdrop.test.tsx:3`, `packages/react/src/menu/portal/MenuPortal.test.tsx:4`, `packages/react/src/menu/viewport/MenuViewport.test.tsx:5`.
- `describeConformance` itself is implemented in `packages/react/test/describeConformance.tsx`, which runs a shared suite (prop spreading, ref forwarding, render prop, className) against the minimal element/render function supplied by each test file.
- `@mui/internal-test-utils` supplies `screen` and `waitFor`, used directly by `MenuBackdrop.test.tsx` and `MenuViewport.test.tsx` `packages/react/src/menu/backdrop/MenuBackdrop.test.tsx:4`, `packages/react/src/menu/viewport/MenuViewport.test.tsx:4`.
