# Menu Positioner — Behavior Spec

Scope: `Menu.Positioner` only, as proven by `packages/react/src/menu/positioner/MenuPositioner.test.tsx`. Related component families (`ContextMenu.Positioner`, `Menubar` + `Menu`) are included only insofar as this test file exercises `Menu.Positioner`/`ContextMenu.Positioner` behavior directly.

## Public API surface (props, parts, subcomponents)

- `Menu.Positioner` is a component that must be rendered inside `Menu.Portal`; rendering it directly inside `Menu.Root` without a portal throws an error with message `'Base UI: <Menu.Portal> is missing.'` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:48-62`).
- `Menu.Positioner` renders as (or forwards a ref to) an `HTMLDivElement`, verified via `describeConformance` with `refInstanceof: window.HTMLDivElement` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:64-73`).
- Accepts an `anchor` prop that can be: a React ref object pointing to an element (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:190-229`), a plain DOM element via state (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:231-273`), a function returning an element (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:275-319`), a "virtual element" object exposing `getBoundingClientRect` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:321-357`), a non-memoized inline function (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:359-385`), and `undefined` (falls back to the trigger element) (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:387-458`).
- Accepts `side` prop with physical values (`"bottom"`, `"left"`, `"right"`) and logical values (`"inline-start"`, `"inline-end"`) that are resolved/flipped based on writing direction and available space (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:639-661`, `:688-710`, `:749-771`, `:798-820`).
- Accepts `align` prop (e.g. `"start"`, `"center"`, `"end"`) which can be flipped/resolved by the positioning engine (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:663-686`, `:773-796`).
- Accepts `sideOffset` prop as either a number (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:600-617`) or a function receiving positioning data (`{ side, align, positioner, anchor }`) and returning a number (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:619-637`, `:639-710`).
- Accepts `alignOffset` prop as either a number (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:713-730`) or a function receiving the same positioning data shape (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:732-820`).
- Accepts `arrowPadding` prop (used alongside `anchor` in several tests, e.g. `arrowPadding={0}`) (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:202`).
- Accepts `collisionAvoidance` prop, e.g. `{ side: 'flip' }`, which affects shift/cross-axis behavior passed to the underlying anchor-positioning hook (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:93-108`).
- `ContextMenu.Positioner` (a related part exercised in this file) accepts `side`, `align`, `sideOffset`, `alignOffset`, and `collisionAvoidance` props with the same shapes as `Menu.Positioner` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:110-127`).
- Composes with `Menu.Popup`, `Menu.Item`, `Menu.Viewport`, `Menu.SubmenuRoot`, `Menu.SubmenuTrigger`, `Menu.Portal`, `Menu.Root`, `Menu.Trigger`, and (for context menus) `ContextMenu.Root`/`ContextMenu.Portal`/`ContextMenu.Popup`/`ContextMenu.SubmenuRoot` (throughout the file, e.g. `packages/react/src/menu/positioner/MenuPositioner.test.tsx:64-73`, `:147-187`, `:75-145`).

## State model (controlled/uncontrolled, defaults, transitions)

- The positioner itself does not own open/closed state in this test file; state is driven by the enclosing `Menu.Root`/`ContextMenu.Root` via `open` (controlled) or `defaultOpen` (uncontrolled) props (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:54-57`, `:132`, `:152-176`).
- When a controlled parent menu's `open` is set to `false`, an open submenu nested under that parent's positioner/popup closes and reports `onOpenChange(false, { reason: 'sibling-open', ... })` — i.e. the submenu's close is attributed to the parent closing, not to independent user interaction (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:147-187`).
- Default `side` for a positioner whose menu is opened from a `Menubar` is `"bottom"` when the menubar is horizontal (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:551-573`) and `"inline-end"` when the menubar is `orientation="vertical"` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:575-597`).
- For a root `ContextMenu`, the underlying anchor-positioning hook is invoked with `shift: { crossAxis: true, rootBoundary: 'layoutViewport' }` by default (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:76-91`).
- Setting `collisionAvoidance={{ side: 'flip' }}` on a `ContextMenu.Positioner` changes the hook's `shift` value to `{ crossAxis: false, rootBoundary: 'layoutViewport' }` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:93-108`).
- For a `ContextMenu` submenu (nested `ContextMenu.SubmenuRoot`), the hook is called with `shift: undefined`, implying the submenu positioner uses the visual viewport rather than the layout viewport used by the root context menu (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:129-144`).
- Explicit `side`, `align`, `sideOffset`, and `alignOffset` props passed to a `ContextMenu.Positioner` are forwarded unchanged to the underlying anchor-positioning hook (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:110-127`).
- When `anchor` transitions from a ref to `undefined` and back to the ref, the positioner's computed transform updates accordingly each time — falling back to the trigger element's position when `anchor` is `undefined`, and returning to the referenced anchor's position when the ref is restored (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:387-458`).
- `Menu.Portal`'s `keepMounted` prop affects whether the positioner's popup content stays in the DOM when the menu closes: with `keepMounted` (default true in the parent-closes-submenu test), content stays mounted (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:158`); with `keepMounted={true}` explicitly, the popup element remains present but inaccessible when closed (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:467-503`); with `keepMounted={false}`, the popup element is unmounted (not found by `queryByRole`) when closed (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:505-538`).

## Keyboard interactions

N/A — this test file does not exercise any keyboard interactions against the positioner (only pointer clicks via `userEvent.click` and programmatic state changes are used).

## Focus management

N/A — this test file does not assert any focus-management behavior for the positioner.

## Accessibility (roles, aria-*, id linking)

- The popup rendered inside the positioner has role `"menu"`, queried via `screen.getByRole('menu', { hidden: true | false })` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:487-502`, `:525-536`).
- When the menu is closed but `keepMounted` is true, the menu element is present in the DOM but is asserted to be inaccessible via the `toBeInaccessible()` matcher (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:488`, `:501-502`).
- When the menu is open, the menu element is asserted to not be inaccessible (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:494`, `:532`).

## DOM structure & portal behavior

- `Menu.Positioner` requires a `Menu.Portal` ancestor; without one, rendering throws synchronously during render with the message `'Base UI: <Menu.Portal> is missing.'` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:48-62`).
- The positioner element itself is a plain `HTMLDivElement` per conformance testing (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:64-73`).
- The positioner is positioned using inline CSS `transform: translate(Xpx, Ypx)` when there is no `Menu.Viewport` inside `Menu.Popup` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:823-840`), reflected in assertions like `positioner.style.getPropertyValue('transform')` / `positioner.style.transform` throughout the `anchor`, `sideOffset`, and `alignOffset` describe blocks (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:226-228`, `:614-616`, `:727-729`).
- When `Menu.Popup` contains a `Menu.Viewport`, the positioner instead uses top/left positioning and its `transform` style is empty string (`''`) once positioned (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:842-860`).
- With a virtual element anchor (an object with only `getBoundingClientRect`), the positioner's transform reflects the virtual element's returned bounding rect coordinates directly (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:321-357`).
- `Menu.Popup` and `Menu.Item` are expected to be nested inside `Menu.Positioner`, which is nested inside `Menu.Portal` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:196-211` and throughout).
- A nested submenu structure is `Menu.Popup > Menu.SubmenuRoot > Menu.SubmenuTrigger + Menu.Portal > Menu.Positioner > Menu.Popup` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:159-171`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onOpenChange` (passed to `Menu.SubmenuRoot`, which controls the nested positioner's visibility) is called with signature `(open: boolean, eventDetails)` where `eventDetails.reason` can be `'sibling-open'` when a submenu closes because its controlled parent menu closed (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:147-187`, specifically `:184-186`).
- No other custom events are asserted on the positioner itself in this file.

## Edge cases (rapid interactions, unmount, nesting)

- Nesting: an open submenu (`Menu.SubmenuRoot defaultOpen`) whose positioner/popup is rendered inside a parent's positioner/popup automatically closes with reason `'sibling-open'` when the parent's controlled `open` state flips to `false` (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:147-187`).
- Nesting: `ContextMenu.SubmenuRoot defaultOpen` inside a root `ContextMenu.Root` causes the submenu's positioner to receive `shift: undefined` from the anchor-positioning hook, differing from the root context menu's `shift` configuration (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:129-144`).
- Anchor prop changes at runtime (ref → undefined → ref) are handled without error, with the positioner's transform recalculating to track the trigger element when the anchor is absent, then the anchor element once restored (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:387-458`).
- A non-memoized (freshly created on every render) function anchor is accepted without error or infinite update loops, and the positioner still renders (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:359-385`).
- An anchor function that resolves to `null` is tolerated — the positioner still renders (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:359-385`).
- Unmount: after asserting transform-based positioning is applied (no `Menu.Viewport`), the component is explicitly unmounted via `unmount()` without asserted errors (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:823-840`); similarly for the top/left positioning (with `Menu.Viewport`) case (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:842-860`).
- Toggling the menu open/closed repeatedly via trigger clicks with `keepMounted` true vs. false produces consistent mount/accessibility state each time (`packages/react/src/menu/positioner/MenuPositioner.test.tsx:467-538`).

## Shared harness dependencies

- `#test-utils` resolves to `packages/react/test/index.ts`, which re-exports (among others) `createRenderer`, `describeConformance`, `isJSDOM`, `resetBrowserPointer`, and `waitForPositioned` — all used directly by this test file.
