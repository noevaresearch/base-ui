# Content, Indent, IndentBackground, Provider — Behavior Spec (part)

Covers behavior mined from `packages/react/src/drawer/content/DrawerContent.test.tsx`, `packages/react/src/drawer/indent/DrawerIndent.test.tsx`, `packages/react/src/drawer/indent-background/DrawerIndentBackground.test.tsx`, and `packages/react/src/drawer/provider/DrawerProvider.test.tsx`.

## Public API surface (props, parts, subcomponents)

**Drawer.Content**
- Exported as a subcomponent of the `Drawer` namespace (`Drawer.Content`) and rendered as a plain `div` element: the conformance suite mounts `<Drawer.Content />` with `refInstanceof: window.HTMLDivElement`, proving the default rendered element is an `HTMLDivElement`. `packages/react/src/drawer/content/DrawerContent.test.tsx:9-22`
- Supports the standard Base UI component API, exercised through the conformance suite with no tests skipped: props spread/forwarding, ref forwarding, `render` prop (function and element forms), and `className`. `packages/react/src/drawer/content/DrawerContent.test.tsx:9-22`
  - Ref forwarding: attaching a `ref` yields an instance of `refInstanceof` on the rendered element. `packages/react/test/conformanceTests/refForwarding.tsx:32-38`
  - Custom props (`lang`, arbitrary `data-*`) are forwarded onto the default rendered element, onto an element produced by a function `render` prop, and onto an element passed as the `render` prop. `packages/react/test/conformanceTests/propForwarding.tsx:23-36`, `packages/react/test/conformanceTests/propForwarding.tsx:38-59`, `packages/react/test/conformanceTests/propForwarding.tsx:61-80`
  - A custom `style` object is forwarded to the rendered root in all three usages (direct, function `render`, element `render`). `packages/react/test/conformanceTests/propForwarding.tsx:82-127`
  - A string `className` is applied to the rendered element. `packages/react/test/conformanceTests/className.tsx:20-23`
  - The `render` prop accepts a function or a JSX element; custom wrappers may wrap the rendered element (wrapping allowed); the component ref is passed through to the custom element and merged with the custom element's own ref; component `className` (string or function form) is merged with the `render` element's `className`. `packages/react/test/conformanceTests/renderProp.tsx:41-59`, `packages/react/test/conformanceTests/renderProp.tsx:61-76`, `packages/react/test/conformanceTests/renderProp.tsx:93-113`, `packages/react/test/conformanceTests/renderProp.tsx:115-144`, `packages/react/test/conformanceTests/renderProp.tsx:146-161`, `packages/react/test/conformanceTests/renderProp.tsx:163-178`

**Drawer.Indent**
- Exported as `Drawer.Indent`; accepts standard attributes such as `data-testid`. No props, ref, or render-prop behavior is asserted beyond the data-attribute and inline-style behavior below (no conformance suite runs for it in this batch). `packages/react/src/drawer/indent/DrawerIndent.test.tsx:16`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:61`
- Can be placed anywhere inside (or outside, with a fallback) a `Drawer.Provider`: proven both as a wrapper element that contains a `Drawer.Root` in its subtree and as a plain sibling of `Drawer.Root` / provider consumers. `packages/react/src/drawer/indent/DrawerIndent.test.tsx:14-22`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:58-65`

**Drawer.IndentBackground**
- Exported as `Drawer.IndentBackground`; accepts standard attributes such as `data-testid`. No props/ref behavior beyond data-attribute state is asserted in this batch. `packages/react/src/drawer/indent-background/DrawerIndentBackground.test.tsx:15`, `packages/react/src/drawer/indent-background/DrawerIndentBackground.test.tsx:29`

**Drawer.Provider**
- Exported as `Drawer.Provider`; takes only children in all tested usages (no props asserted). `packages/react/src/drawer/provider/DrawerProvider.test.tsx:50-54`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:60-64`
- Exposes an internal context consumed by `useDrawerProviderContext` (imported from `./DrawerProviderContext`, i.e. an internal hook, not part of the public namespace surface exercised here). The context value used by tests has this shape:
  - `setDrawerOpen(handle: object, open: boolean)` — registers/unregisters a drawer's open state against the provider. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:20-21`
  - `removeDrawer(handle: object)` — removes a drawer registration. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:22-23`
  - `visualStateStore.set(partial: { swipeProgress?: number; frontmostHeight?: number })` — publishes shared visual state consumed by indent parts. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:24-41`
- The consumer hook is written to handle a missing context (returns `null` from a consumer when context is falsy), which implies the hook can be called outside a provider without throwing; however, no test in this batch actually renders a context consumer outside a provider. UNVERIFIED — inferred from `packages/react/src/drawer/provider/DrawerProvider.test.tsx:11-16`, no test asserts this.

## State model (controlled/uncontrolled, defaults, transitions)

**Drawer.Provider (open-drawer registry)**
- The provider maintains a registry of open drawers spanning multiple `Drawer.Root` instances. Its derived "active" state is true while at least one registered drawer is open, and false only once every open drawer has been closed or removed (unmounted). `packages/react/src/drawer/provider/DrawerProvider.test.tsx:70-86`
- Observed transitions with two `Drawer.Root`s controlled by `open` props: both closed → inactive; first open → active; first closed while second still open → still active; second drawer unmounted (via `showSecond={false}`, its `open` prop still `true`) → inactive. Unmounting an open drawer therefore removes its registration and can deactivate the provider. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:78-85`
- Registry updates are idempotent: a redundant `setDrawerOpen(handle, true)` while already open, and `removeDrawer` of a handle that was never registered, leave the active state undisturbed. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:97-102`
- Registering a drawer as closed (`setDrawerOpen(handle, false)`) transitions the provider to inactive and does not retain any registration: a subsequent double `removeDrawer(handle)` is a no-op that keeps the state inactive. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:104-109`
- Registering a closed drawer does not cause any re-render of the provider subtree — verified with a `React.Profiler` `onRender` spy that records zero renders after clicking "Register closed". Closed drawers are therefore not stored in provider state. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:112-126`

**Drawer.Provider (visual state store)**
- `visualStateStore.set` performs partial merges: setting `{ swipeProgress: 0 }` alone updates only the progress value and leaves a previously set `frontmostHeight` intact; setting `{ frontmostHeight: 0 }` alone updates only the height. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:29-34`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:136-141`
- Invalid values are sanitized when applied to `Drawer.Indent`: `swipeProgress: NaN` results in `--drawer-swipe-progress: 0` on the indent element, and `frontmostHeight: Infinity` results in `--drawer-height` being cleared (empty), not set to a bogus value. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:35-41`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:143-145`

**Drawer.Indent / Drawer.IndentBackground (active/inactive flag)**
- Both parts carry a binary active/inactive state with a default of inactive: initially (no open drawers) they render with `data-inactive=""` present and `data-active` absent. `packages/react/src/drawer/indent/DrawerIndent.test.tsx:31-32`, `packages/react/src/drawer/indent-background/DrawerIndentBackground.test.tsx:31-32`
- The flag transitions to active as soon as any drawer inside the same provider becomes open (`data-active=""` present, `data-inactive` removed), and it is shared across sibling parts: when the drawer opens, both `Drawer.Indent` and `Drawer.IndentBackground` in the same provider show `data-active`. `packages/react/src/drawer/indent/DrawerIndent.test.tsx:34-38`, `packages/react/src/drawer/indent-background/DrawerIndentBackground.test.tsx:34-37`
- Rendering `Drawer.Indent` and `Drawer.IndentBackground` without any `Drawer.Provider` does not crash and falls back to the inactive state (`data-inactive=""` on both). `packages/react/src/drawer/provider/DrawerProvider.test.tsx:153-163`

**Drawer.Content**
- No controlled/uncontrolled state is asserted for `Drawer.Content` in this batch. N/A.

## Keyboard interactions

N/A — none of the four test files in this batch assert any keyboard behavior (no `user.keyboard`, no key events) for `Drawer.Content`, `Drawer.Indent`, `Drawer.IndentBackground`, or `Drawer.Provider`. The only interactions exercised are mouse clicks on test-only buttons and prop-driven re-renders. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:97-150`

## Focus management

N/A — no test in this batch asserts any focus behavior (no focus/blur assertions, no `activeElement` checks) for any of the four parts.

## Accessibility (roles, aria-*, id linking)

N/A — no test asserts roles, `aria-*` attributes, or id linking for any of the four parts. The only observable attributes are `data-*` state attributes (`data-active`, `data-inactive`) and inline CSS custom properties (`--drawer-swipe-progress`, `--drawer-height`). `packages/react/src/drawer/indent/DrawerIndent.test.tsx:31-38`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:133-150`

## DOM structure & portal behavior

**Drawer.Content**
- Canonical mounting context used by the tests: `Drawer.Root open` > `Drawer.Portal` > `Drawer.Viewport` > `Drawer.Popup` > `Drawer.Content`; Content renders successfully within this composition (conformance wrapper). `packages/react/src/drawer/content/DrawerContent.test.tsx:11-21`
- The default rendered element is a `div` (ref resolves to `window.HTMLDivElement`). `packages/react/src/drawer/content/DrawerContent.test.tsx:10`
- Content must not expose public swipe-ignore markers: rendered inside an open drawer, the Content element carries neither `data-swipe-ignore` nor `data-base-ui-swipe-ignore` attributes. `packages/react/src/drawer/content/DrawerContent.test.tsx:24-39`

**Drawer.Indent**
- Renders an element whose inline style is driven by the provider's visual state store:
  - `swipeProgress` maps to the CSS custom property `--drawer-swipe-progress` as a bare number string (e.g. `0.5`, `0`). `packages/react/src/drawer/provider/DrawerProvider.test.tsx:133`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:137`
  - `frontmostHeight` maps to `--drawer-height` with a `px` suffix (e.g. `120` → `120px`). `packages/react/src/drawer/provider/DrawerProvider.test.tsx:134`
  - `frontmostHeight: 0` removes the `--drawer-height` property entirely (empty string), rather than writing `0px`. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:140-141`
  - `data-active` / `data-inactive` reflect the provider registry state (see State model). `packages/react/src/drawer/indent/DrawerIndent.test.tsx:31-38`
- Works as a wrapper element around drawer trees and as a standalone sibling; no specific DOM nesting relative to `Drawer.Root` is required. `packages/react/src/drawer/indent/DrawerIndent.test.tsx:16-20`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:61`

**Drawer.IndentBackground**
- Renders a single element carrying the same `data-active` / `data-inactive` state attributes as `Drawer.Indent` (no inline CSS custom properties are asserted for it). `packages/react/src/drawer/indent-background/DrawerIndentBackground.test.tsx:31-37`

**Drawer.Provider**
- Renders its children with no asserted wrapper element or DOM output of its own; all tested observables live on the children (registry state surfaced through `Drawer.IndentBackground` / `Drawer.Indent`). `packages/react/src/drawer/provider/DrawerProvider.test.tsx:50-54`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:88-94`

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no component-emitted events (callbacks, custom DOM events, bubbling, `preventDefault` semantics) are asserted for any of the four parts. All interactivity in the Provider tests goes through test-only `<button>` click handlers that call the internal context API directly (`setDrawerOpen`, `removeDrawer`, `visualStateStore.set`); these buttons are test plumbing, not part of the component API. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:18-43`

## Edge cases (rapid interactions, unmount, nesting)

- **Redundant registrations**: calling `setDrawerOpen(handle, true)` twice in a row does not disturb the active state. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:97-100`
- **Removing an unregistered handle**: `removeDrawer` with a handle that was never registered is a safe no-op. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:101-102`
- **Double removal**: calling `removeDrawer` twice on the same handle keeps the provider inactive without errors. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:107-109`
- **Unmount while open**: unmounting an open `Drawer.Root` (rather than closing it via props) deactivates the provider, proving removal happens on unmount as well as on close. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:84-85`
- **No render churn on closed registration**: registering a closed drawer triggers zero re-renders in the provider subtree (measured with `React.Profiler` `onRender`). `packages/react/src/drawer/provider/DrawerProvider.test.tsx:112-126`
- **Invalid numeric visual state**: `NaN` progress and `Infinity` height are sanitized to `--drawer-swipe-progress: 0` and a cleared `--drawer-height` respectively. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:143-145`
- **Indent unmount cleanup**: after the visual state is set (`--drawer-swipe-progress: 0.5`, `--drawer-height: 120px`) and the `Drawer.Indent` is unmounted, the (now-detached) indent element's inline style is reset to the defaults (`--drawer-swipe-progress` = `'0'`, `--drawer-height` = `''`), proving unmount-time cleanup of the synced CSS custom properties. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:147-150`
- **Nesting/placement flexibility**: `Drawer.Indent` may wrap an entire `Drawer.Root` subtree or sit as a sibling; both arrangements receive the same provider state. `packages/react/src/drawer/indent/DrawerIndent.test.tsx:14-22`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:58-65`
- **Missing provider**: indent parts render without a `Drawer.Provider`, defaulting to `data-inactive=""` on both. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:153-163`
- **Multiple sibling drawers**: the provider correctly tracks independent open/close of two sibling `Drawer.Root`s, deactivating only when all are gone. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:70-86`
- No rapid-interaction (e.g. fast swipe/pointer sequences) edge cases are asserted in this batch. N/A.

## Shared harness dependencies

- `#test-utils` resolves to the barrel `packages/react/test/index.ts`, which re-exports the shared render/conformance/pointer/wait helpers plus `@base-ui/utils/testUtils`. These four test files use only `createRenderer` and `describeConformance` from it. `packages/react/test/index.ts:1-14`
- **`createRenderer`** (`packages/react/test/createRenderer.ts`): wraps `@mui/internal-test-utils`'s renderer; `render` is awaited inside `act`, and the result is augmented with `setProps(newProps)` (shallow-merges props onto the originally rendered element via `React.cloneElement` + rerender, wrapped in `act`) and an async `rerender`. This is how all prop-transition assertions in the batch (open/close toggling, unmounting the second drawer, unmounting the indent) are performed. `packages/react/test/createRenderer.ts:31-43`
- **`describeConformance`** (`packages/react/test/describeConformance.tsx`): runs the "Base UI component API" suite against a minimal element with a required `render` wrapper. The full suite (used by `Drawer.Content` here, no `skip`/`only`) is `propsSpread` → `testPropForwarding`, `refForwarding` → `testRefForwarding`, `renderProp` → `testRenderProp`, `className` → `testClassName`. Options relevant to this batch: `refInstanceof: window.HTMLDivElement` and the `Drawer.Root open > Portal > Viewport > Popup` render wrapper. `packages/react/test/describeConformance.tsx:44-49`, `packages/react/test/describeConformance.tsx:55-68`
  - `testRefForwarding`: clones the minimal element with a `React.createRef` and asserts `ref.current` is an `instanceof refInstanceof`. `packages/react/test/conformanceTests/refForwarding.tsx:18-38`
  - `testPropForwarding`: asserts `lang`, random `data-foobar`, and `style={{ color: 'green' }}` land on the default element and on function/element `render` customizations, flushing microtasks after each render. `packages/react/test/conformanceTests/propForwarding.tsx:23-127`
  - `testClassName`: asserts a string `className` is present on the rendered element (queried via `document.querySelector`). `packages/react/test/conformanceTests/className.tsx:20-23`
  - `testRenderProp`: asserts function and element `render` forms render the custom element (wrapping allowed by default), refs are passed through and merged, and `className` (string and function forms) merges between the component and the `render` element. `packages/react/test/conformanceTests/renderProp.tsx:41-178`
- **`screen`** comes from the external `@mui/internal-test-utils` package (document-level queries, so portal-rendered content is found). `packages/react/src/drawer/content/DrawerContent.test.tsx:3`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:4`
- **`useDrawerProviderContext`** is imported from the component-local module `./DrawerProviderContext` (i.e. `packages/react/src/drawer/provider/DrawerProviderContext`) purely as a test accessor for the internal provider context. `packages/react/src/drawer/provider/DrawerProvider.test.tsx:6`
- **Environment gating**: none of the four test files use `describe.skipIf(isJSDOM)` / Chromium-only gating; all tests in this batch run in both jsdom and Chromium environments. `packages/react/src/drawer/content/DrawerContent.test.tsx:1-40`, `packages/react/src/drawer/indent/DrawerIndent.test.tsx:1-40`, `packages/react/src/drawer/indent-background/DrawerIndentBackground.test.tsx:1-39`, `packages/react/src/drawer/provider/DrawerProvider.test.tsx:1-164`
