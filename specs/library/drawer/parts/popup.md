# Popup — Behavior Spec (part)

Covers behavior proven by `packages/react/src/drawer/popup/DrawerPopup.test.tsx` (the only test file mined for this part).

## Public API surface (props, parts, subcomponents)

- `<Drawer.Popup>` is a named part of the `Drawer` namespace object imported from `@base-ui/react/drawer`; it is always used as `<Drawer.Popup>` inside `<Drawer.Root>` → `<Drawer.Portal>` → `<Drawer.Viewport>`. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:3-5`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:33-44`
- The popup renders as an HTML `div` element (ref forwarding conformance asserts `refInstanceof: window.HTMLDivElement`). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:33-44`
- Conformance suite runs for the popup: prop spread/forwarding, ref forwarding, render prop, and className handling (the full Base UI component API describe suite). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:33-44`, `packages/react/test/describeConformance.tsx:44-49`
- Proven props:
  - `initialFocus`: accepts `false` to opt out of initial focus stealing (see Focus management). Any other shape is not exercised. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:124-143`
  - `render`: accepts a render function; the popup is resilient when the render function returns `null` (no throw during mount). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:242-255`
  - Standard pass-through: `data-testid` (used throughout, e.g. `data-testid="popup"`), `style` (e.g. borders applied to child popup), `className` (used for exit-animation targeting), and `ref` (callback refs used to stub `offsetHeight`). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:105`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:313-318`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:653-656`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:413-421`
- No subcomponents are defined or exercised by the popup tests.
- The drawer root context is consumable from anywhere under `<Drawer.Root>`: a probe component reads `popupHeight` (number) and calls `onNestedFrontmostHeightChange(height)` obtained from the drawer root context. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:19-28`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:230`

## State model (controlled/uncontrolled, defaults, transitions)

- Open/close state is owned by `<Drawer.Root>` (`open` / `defaultOpen` props); the popup tests never pass open-state props to the popup itself. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:37`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:172-178`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:403`
- **Popup height measurement**: the popup measures its own height (via `ResizeObserver`) and exposes it as `popupHeight` in the drawer root context; a probe renders it as text and it equals the stubbed `offsetHeight` (100). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:19-28`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:213-228`
- The last measured popup height is retained while nested content stretches the popup: after a nested-frontmost-height change is reported (200) and ResizeObserver callbacks fire with the stubbed real height now 150, `popupHeight` still reads 100 (no re-measure overwrite). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:230-236`
- **Nested-drawer presence state** lives on the parent popup's inline style/attributes:
  - `--nested-drawers` CSS custom property holds the count as a string: `'0'` when no nested drawer is open (even if the nested `<Drawer.Portal keepMounted>` exists), `'1'` when one is open. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:603-604`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:610-612`
  - `data-nested-drawer-open` attribute (empty-string value) is present exactly while a nested drawer is open, and removed as soon as the nested drawer closes. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:445`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:492`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:613`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:620-621`
- **Frontmost height state**: when a nested drawer is open, the parent popup gets `--drawer-frontmost-height` (px string) equal to the nested popup's measured height; in the StrictMode timing test it is `'100px'` for a child `offsetHeight` stubbed to 100. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:414-421`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:443-446`
- Nested presence and frontmost height are reported to the parent **before passive effects flush** (during the layout/sync phase): wrappers around root-context methods `notifyParentHasNestedDrawer` and `notifyParentFrontmostHeight` observe both being invoked with truthy values before any passive effect runs, under `React.StrictMode`. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:359-382`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:437-442`
- **`--drawer-height` lifecycle (Chromium only)**: when a nested drawer opens, the parent popup's inline style gains a non-empty `--drawer-height` (its own measured height); it persists (stays non-empty) after the nested drawer starts closing and the `data-nested-drawer-open` attribute is removed; on reopen it is (re)applied by the time `data-nested-drawer-open` appears. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:766-771`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:793-803`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:907-912`
- **Snap point offset**: with `defaultSnapPoint="100px"`, `snapPoints={['100px','300px']}`, `swipeDirection="up"`, a 400px-tall viewport and a 300px-tall popup, the popup style gets `--drawer-snap-point-offset: -200px` (negative offset for an upward drawer whose snap point is smaller than the popup height; 100px − 300px = −200px). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:170-195`
  - UNVERIFIED — inferred from `packages/react/src/drawer/popup/DrawerPopup.test.tsx:189-193`, no test asserts the general formula for other snap points/directions.

## Keyboard interactions

- Composite navigation keys are stopped from escaping the popup: a `keydown` for `ArrowDown` fired on an `<input>` inside the popup does NOT reach an `onKeyDown` handler on an ancestor `<div>` outside the drawer. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:145-164`
- Regular character keys are not stopped: a `keydown` for `a` on the same input DOES propagate to the ancestor handler. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:145-148`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:166-167`
- No other keyboard behavior (Escape, Tab trapping, etc.) is proven by this file for the popup.

## Focus management

- **Default initial focus is the popup element itself**: opening the drawer (trigger click, `modal={false}`) moves focus to the popup `<div>`, not to the focusable `<input>` inside it. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:97-122`
- **`initialFocus={false}`** leaves focus on the trigger: after clicking the trigger, the dialog becomes visible and the trigger still has focus. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:124-143`

## Accessibility (roles, aria-*, id linking)

- The popup is exposed with the `dialog` role (retrieved via `screen.getByRole('dialog')`). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:139-141`
- **Development warning when rendered without a viewport**: rendering `<Drawer.Popup>` inside `<Drawer.Portal>` but NOT inside `<Drawer.Viewport>` logs a `console.error` in development containing `Base UI: <Drawer.Popup> expected to be rendered within <Drawer.Viewport>.`. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:46-66`
- The full, exact warning message is: `Base UI: <Drawer.Popup> expected to be rendered within <Drawer.Viewport>. Omitting the viewport disables drawer swipe handling and touch scroll locking. Wrap <Drawer.Popup> in <Drawer.Viewport>.` — asserted with exact string equality (not a substring), and it is emitted without relying on React owner-stack support (`SafeReact.captureOwnerStack` mocked to return `null` still yields the identical message, with no owner stack appended). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:68-95`
- No `aria-*` attribute wiring or id linking is asserted by this file.

## DOM structure & portal behavior

- The popup mounts inside `<Drawer.Portal>` (portalled to outside the React tree of the trigger), normally wrapped by `<Drawer.Viewport>` which contains it as a direct child. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:33-44`
- Rendering the popup inside a portal without a viewport is supported at runtime but triggers the development warning above (viewport omission disables drawer swipe handling and touch scroll locking, per the warning text). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:49-63`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:84-90`
- The popup element carries a `data-swipe-direction` attribute reflecting the root's `swipeDirection` (e.g. `data-swipe-direction="up"`). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:194`
- The popup element carries the `data-nested-drawer-open` attribute (empty value) while a nested drawer is open; it is absent otherwise, including on child popups that merely contain dialogs. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:445`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:491-492`
- Inline CSS custom properties set on the popup element (proven):
  - `--drawer-snap-point-offset` (see State model). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:189-193`
  - `--nested-drawers` (count string). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:444`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:603-612`
  - `--drawer-frontmost-height` (nested popup height incl. borders, px string, set on the parent popup while a nested drawer is open). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:334-339`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:446`
  - `--drawer-height` (popup's own measured height; Chromium-only assertions — see State model). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:769-771`
- The popup's children may include an entire nested `<Drawer.Root>` tree (nested drawer rendered inside the parent popup's DOM). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:306-325`
- A closed nested drawer with `<Drawer.Portal keepMounted>` does not mark the parent as having a nested drawer open (count stays `'0'`, no attribute). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:586-604`
- `data-ending-style` is applied to the popup while its exit animation runs (used by tests to detect close-in-progress; real animations enabled in Chromium). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:680-682`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:790-792`

## Events (names, payload shape, bubbling, preventDefault semantics)

- No custom DOM events (no `on*` callback props or dispatched CustomEvents) are proven for the popup by this file.
- Keydown interception semantics (see Keyboard interactions): composite navigation keys (`ArrowDown`) fired inside the popup are stopped from bubbling past the popup; printable character keys propagate normally. No `preventDefault` observable effect is asserted, only that the outer React handler is not invoked. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:161-168`
- Root-context notification callbacks (not DOM events) proven to be invoked by the popup/root machinery when a nested drawer exists:
  - `notifyParentHasNestedDrawer(present: boolean)` — called with `true` on nested open before passive effects. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:364-378`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:441-442`
  - `notifyParentFrontmostHeight(height: number)` — called with the nested popup's measured height (> 0) before passive effects; height includes borders (Chromium test equates it to `childPopup.offsetHeight`). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:364-372`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:334-339`
  - `onNestedFrontmostChange`-style public escape hatch: children can call `onNestedFrontmostHeightChange(200)` obtained from the drawer root context to report a frontmost height change. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:24-27`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:230`
- `ResizeObserver` is observed on the popup to track height changes (stubbed in tests; callbacks fire with the standard `(entries, observer)` shape). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:198-210`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:232-234`

## Edge cases (rapid interactions, unmount, nesting)

- **Render function returning `null`**: `<Drawer.Popup render={() => null}>` mounts without throwing (exercise of custom render resilience; `@ts-expect-error` used to bypass the type check). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:242-255`
- **Queued measurement after unmount**: if the popup unmounts and its queued `ResizeObserver` callbacks fire afterwards, invoking them resolves without throwing (the observer is disconnected/cleanup is safe). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:257-299`
- **Height pinned under nested stretch**: a nested frontmost-height report plus a subsequent ResizeObserver tick with a larger real popup height does NOT overwrite the popup's retained measured height (stays 100 after reporting 200 and stubbing 150). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:226-236`
- **StrictMode safety**: the before-passive-effects timing behavior holds under `<React.StrictMode>` (double render/invoke tolerated). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:402`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:437-447`
- **Dialogs are not nested drawers**: a `Dialog.Root` opened inside a nested drawer does not change the parent popup's `--nested-drawers` (stays `'1'`), and the child popup's count stays `'0'` with no `data-nested-drawer-open`; a MutationObserver over the parent popup's `style` attribute never observes a `'2'` count (no transient increment). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:449-510`
- **Alert dialogs are not nested drawers**: identical behavior for `AlertDialog.Root` opened inside a nested drawer (no `'2'` ever observed). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:512-576`
- **Parent state clears at close-start, not unmount**: clicking a close control in the nested drawer immediately resets the parent popup to `--nested-drawers: '0'` and removes `data-nested-drawer-open` while the child popup is still in the document (keepMounted portal). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:615-621`
- **(Chromium-only) Parent state clears during exit animation**: with real animations (`BASE_UI_ANIMATIONS_DISABLED = false`) and a non-keepMounted portal, the parent's nested state is already cleared the moment the child popup gains `data-ending-style`, and only later does the child popup unmount (`queryByTestId` → null). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:624-693`
- **(Chromium-only) Frontmost height includes borders**: for a child popup with 2px top/bottom borders, the parent's `--drawer-frontmost-height` equals the child's `offsetHeight` (border-inclusive box measurement, guarded by `offsetHeight > scrollHeight`). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:301-341`
- **(Chromium-only) Height lock across close**: the parent popup's `--drawer-height` remains non-empty in the same mutation batch in which `data-nested-drawer-open` is removed, so consumer CSS (`height: var(--drawer-height)` with a transition) can hold the fixed height during the nested close animation. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:773-803`
- **(Chromium-only) Height lock across reopen**: when the nested drawer reopens, `--drawer-height` is already non-empty in the mutation in which `data-nested-drawer-open` reappears (height restored before/at nested-state application). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:890-912`
- Drag/swipe-to-close gesture handling on the popup is NOT exercised by this test file; only the static `--drawer-snap-point-offset` variable and `data-swipe-direction` attribute are proven. N/A — no test in `packages/react/src/drawer/popup/DrawerPopup.test.tsx` covers pointer gestures.

## Shared harness dependencies

- `#test-utils` (resolves to `packages/react/test/index.ts`) provides `createRenderer`, `describeConformance`, and `isJSDOM`. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:9`
- `createRenderer` wraps `@mui/internal-test-utils`'s render in `act`, and augments the result with act-wrapped `rerender` / `setProps` helpers (`setProps` clones the rendered element with new props). `packages/react/test/createRenderer.ts:31-43`
- `describeConformance` runs the "Base UI component API" suite — `propsSpread` (prop forwarding), `refForwarding`, `renderProp`, `className` — against the provided minimal element and render wrapper. `packages/react/test/describeConformance.tsx:44-68`
- `isJSDOM` is a boolean derived from `/jsdom/.test(window.navigator.userAgent)`; four tests are gated `it.skipIf(isJSDOM)` (Chromium-only): border-inclusive frontmost height (line 301), parent state cleared before unmount (line 624), fixed height kept while nested closes (line 695), fixed height restored on reopen (line 815). `packages/utils/src/testUtils.ts:4`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:301`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:624`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:695`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:815`
- `act`, `fireEvent`, `screen`, `waitFor` come from `@mui/internal-test-utils` (external MUI harness; semantics assumed standard RTL-like). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:8`
- Test-local helpers defined in the test file:
  - `setHeight(element, getValue)` — defines an `offsetHeight` getter override on an element to stub layout measurement. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:13-17`
  - `PopupHeightProbe` — child of `Drawer.Root` that renders root-context `popupHeight` and a button calling `onNestedFrontmostHeightChange(200)`. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:19-28`
  - `ResizeObserver` stub class collecting constructor callbacks so tests can fire them manually. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:202-210`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:261-269`
  - `MutationObserver` on the popup's `style`/`data-nested-drawer-open` attributes to capture exact ordering of inline-style mutations. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:479-486`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:773-784`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:890-901`
  - Injected `<style dangerouslySetInnerHTML>` blocks defining keyframes/transitions for Chromium animation tests. `packages/react/src/drawer/popup/DrawerPopup.test.tsx:629-639`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:698-728`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:820-840`
- Global flag `globalThis.BASE_UI_ANIMATIONS_DISABLED` is set to `false` to enable real exit animations in Chromium tests and restored to `true` in `finally` (implying the harness default is animations-disabled). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:627`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:690`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:696`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:811`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:818`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:915`
- Root/drawer context hooks used by in-file probes: `useDrawerRootContext` and `useDialogRootContext` (imported from component modules solely to build probes/phase boundaries, not to assert internals). `packages/react/src/drawer/popup/DrawerPopup.test.tsx:10-11`, `packages/react/src/drawer/popup/DrawerPopup.test.tsx:359-398`
