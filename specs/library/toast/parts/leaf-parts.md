# Toast leaf parts — behavior spec (Action, Arrow, Close, Content, Description, Title, isRenderableNode)

Evidence: only the unit test files listed below plus the shared harness under `packages/react/test/`. Nothing here is asserted beyond what those tests prove.

## Public API surface (props, parts, subcomponents)

- Parts are consumed off the `Toast` namespace import: `Toast.Action`, `Toast.Arrow`, `Toast.Close`, `Toast.Content`, `Toast.Description`, `Toast.Title` (each test imports `{ Toast } from '@base-ui/react/toast'`). `packages/react/src/toast/action/ToastAction.test.tsx:2`, `packages/react/src/toast/arrow/ToastArrow.test.tsx:3`, `packages/react/src/toast/close/ToastClose.test.tsx:2`, `packages/react/src/toast/content/ToastContent.test.tsx:3`, `packages/react/src/toast/description/ToastDescription.test.tsx:2`, `packages/react/src/toast/title/ToastTitle.test.tsx:3`
- `Toast.Action` default element is a native `<button>`: conformance is declared with `refInstanceof: window.HTMLButtonElement` and `testComponentPropWith: 'button'`, `button: true`. `packages/react/src/toast/action/ToastAction.test.tsx:15-18`
- `Toast.Arrow` ref target is a generic DOM `Element` (`refInstanceof: window.Element`), and it is mounted inside `Toast.Positioner` for conformance. `packages/react/src/toast/arrow/ToastArrow.test.tsx:15-24`
- `Toast.Close` default element is a native `<button>` (same conformance options as Action). `packages/react/src/toast/close/ToastClose.test.tsx:15-18`
- `Toast.Content` default element is a `<div>` (`refInstanceof: window.HTMLDivElement`). `packages/react/src/toast/content/ToastContent.test.tsx:15-16`
- `Toast.Description` default element is a `<p>` (`refInstanceof: window.HTMLParagraphElement`). `packages/react/src/toast/description/ToastDescription.test.tsx:15-16`
- `Toast.Title` default element is a heading element (`refInstanceof: window.HTMLHeadingElement`). `packages/react/src/toast/title/ToastTitle.test.tsx:16-17`
- `render` prop accepts a React element: Action, Description, and Title all render content passed as `render={<button .../>}` / `render={<div>...</div>}`. `packages/react/src/toast/action/ToastAction.test.tsx:82-94`, `packages/react/src/toast/description/ToastDescription.test.tsx:93-105`, `packages/react/src/toast/title/ToastTitle.test.tsx:114-126`
- `render` prop also accepts a render function receiving props to spread: `<Toast.Title render={(props) => <div {...props}>render fn title</div>} />` renders the custom element. `packages/react/src/toast/title/ToastTitle.test.tsx:128-140`
- Default content is sourced from the toast object: `Toast.Title` with no children renders `toast.title` as its text. `packages/react/src/toast/title/ToastTitle.test.tsx:96-112`, `packages/react/src/toast/title/ToastTitle.test.tsx:263-275`
- `Toast.Description` with no children renders the toast's `description` field as its text. `packages/react/src/toast/description/ToastDescription.test.tsx:75-91`, `packages/react/src/toast/description/ToastDescription.test.tsx:123-135`
- `Toast.Action` renders `toast.actionProps.children` as its content. `packages/react/src/toast/action/ToastAction.test.tsx:110-122`
- A childless `render` element receives the toast's field content as its children: Action renders `actionProps.children` (`'Undo'`) into the childless render element. `packages/react/src/toast/action/ToastAction.test.tsx:110-122`; Description renders `toast.description` into it. `packages/react/src/toast/description/ToastDescription.test.tsx:123-135`; Title renders `toast.title` into it. `packages/react/src/toast/title/ToastTitle.test.tsx:263-275`
- `actionProps.id` from the toast object is applied as the Action element's DOM `id` (after adding a toast with `actionProps: { id: 'action', children: 'action' }`, the rendered action element's `id` is `'action'`). `packages/react/src/toast/action/ToastAction.test.tsx:30-45`
- `positionerProps` on the added toast accepts `anchor` (an element) and `side` (`'bottom'`) — used to drive Arrow positioning in tests. `packages/react/src/toast/arrow/ToastArrow.test.tsx:36-41`
- `Toast.Close` accepts arbitrary props like `aria-label="close-press"` in the shared fixture. `packages/react/src/toast/utils/test-utils.tsx:35`
- For every part listed here, the `describeConformance` harness runs its full suite (`propsSpread`, `refForwarding`, `renderProp`, `className`) unless skipped, proving: refs attach to the default element, custom props (`lang`, `data-*`) forward to the default element and to custom elements via both render-function and render-element forms, `style` forwards from the component and both render forms, function/element `render` render a custom root with the ref threaded through and refs/classNames merged, and string `className` applies. `packages/react/src/toast/action/ToastAction.test.tsx:15-28`, `packages/react/src/toast/arrow/ToastArrow.test.tsx:15-24`, `packages/react/src/toast/close/ToastClose.test.tsx:15-28`, `packages/react/src/toast/content/ToastContent.test.tsx:15-26`, `packages/react/src/toast/description/ToastDescription.test.tsx:15-26`, `packages/react/src/toast/title/ToastTitle.test.tsx:16-27`, `packages/react/test/describeConformance.tsx:44-49`, `packages/react/test/conformanceTests/refForwarding.tsx:31-39`, `packages/react/test/conformanceTests/propForwarding.tsx:22-128`, `packages/react/test/conformanceTests/renderProp.tsx:40-179`, `packages/react/test/conformanceTests/className.tsx:20-23`

## State model (controlled/uncontrolled, defaults, transitions)

- Leaf parts hold no controlled/uncontrolled props of their own in these tests; toasts are created through the toast manager: `Toast.useToastManager()` exposes `add` and `toasts`, and `add` accepts a toast object with fields `title`, `description`, `actionProps` (`{ id, children }`), and `positionerProps` (`{ anchor, side }`). `packages/react/src/toast/utils/test-utils.tsx:6-25`, `packages/react/src/toast/arrow/ToastArrow.test.tsx:29-41`, `packages/react/src/toast/content/ToastContent.test.tsx:28-42`
- `toasts` from `useToastManager` is the source list that the app maps into `Toast.Root` elements; the App re-renders as toasts are added. `packages/react/src/toast/content/ToastContent.test.tsx:44-51`, `packages/react/src/toast/utils/test-utils.tsx:30-38`
- Stack-order state: the newest toast is the frontmost and does not carry `data-behind`; older toasts in the stack do. `packages/react/src/toast/content/ToastContent.test.tsx:56-70`
- Expanded state: `Toast.Content` gains `data-expanded` when the viewport is hovered (mouseEnter) and lacks it before. `packages/react/src/toast/content/ToastContent.test.tsx:72-86`
- `aria-labelledby`/`aria-describedby` on `Toast.Root` are state that transitions with content: present while Title/Description render content, removed when the content goes away. `packages/react/src/toast/title/ToastTitle.test.tsx:201-227`, `packages/react/src/toast/description/ToastDescription.test.tsx:28-47`
- UNVERIFIED — inferred from `packages/react/src/toast/content/ToastContent.test.tsx:56-86`, no test asserts whether `data-behind`/`data-expanded` are removed on subsequent state changes (e.g. mouseLeave) or how the front toast is chosen beyond two toasts.

## Keyboard interactions

N/A — no test in this batch asserts keyboard behavior on any of these parts (interactions used are `user.click` and `fireEvent.mouseEnter`/`fireEvent.click` only). `packages/react/src/toast/action/ToastAction.test.tsx:42`, `packages/react/src/toast/close/ToastClose.test.tsx:53`, `packages/react/src/toast/content/ToastContent.test.tsx:64-65`, `packages/react/src/toast/content/ToastContent.test.tsx:84`

## Focus management

- The Close test focuses the viewport before clicking the close button (via `act(() => viewport.focus())`), proving the close flow works while the viewport holds focus. `packages/react/src/toast/close/ToastClose.test.tsx:47-53`
- UNVERIFIED — inferred from `packages/react/src/toast/close/ToastClose.test.tsx:47-55`, no test asserts where focus moves after the toast is removed.

## Accessibility (roles, aria-*, id linking)

- `Toast.Title` wires `aria-labelledby` on `Toast.Root` to the title element's auto-generated id: after adding a toast, the fixture asserts `root.getAttribute('aria-labelledby')` equals the title element's `id`. `packages/react/src/toast/title/ToastTitle.test.tsx:49-68`
- The `aria-labelledby` link also holds when the title is rendered through the render prop (render element receives the generated id, root points at it). `packages/react/src/toast/title/ToastTitle.test.tsx:142-156`
- When the title's content is removed (`null`), the root's `aria-labelledby` attribute is removed entirely. `packages/react/src/toast/title/ToastTitle.test.tsx:201-227`
- With two titles mounted, the root's `aria-labelledby` resolves to the newer title's id (`'new-title'` when both are present, and it stays `'new-title'` after the older one unmounts); an older title's cleanup must not clear a newer title's link. `packages/react/src/toast/title/ToastTitle.test.tsx:229-261`
- `Toast.Description` wires `aria-describedby` on `Toast.Root` to the description element's id. `packages/react/src/toast/description/ToastDescription.test.tsx:28-47`
- The `aria-describedby` link holds through the render prop as well (render element gets the generated id; root's `aria-describedby` equals it). `packages/react/src/toast/description/ToastDescription.test.tsx:107-121`
- `Toast.Arrow` renders `aria-hidden="true"`. `packages/react/src/toast/arrow/ToastArrow.test.tsx:69`
- UNVERIFIED — inferred from `packages/react/src/toast/utils/test-utils.tsx:33-36`, no test asserts explicit `role` attributes on Title/Description/Close/Action elements (Title resolves to a heading element per conformance, `packages/react/src/toast/title/ToastTitle.test.tsx:16-17`).

## DOM structure & portal behavior

- Canonical nesting used throughout: `Toast.Provider` > `Toast.Viewport` > `Toast.Root` > parts; Arrow mounts under `Toast.Provider` > `Toast.Positioner`. `packages/react/src/toast/action/ToastAction.test.tsx:19-27`, `packages/react/src/toast/arrow/ToastArrow.test.tsx:17-23`, `packages/react/src/toast/close/ToastClose.test.tsx:19-27`, `packages/react/src/toast/content/ToastContent.test.tsx:17-25`
- Toast content elements are rendered inside the viewport element (the fixture maps `toasts` to `Toast.Root` children of `Toast.Viewport data-testid="viewport"`). `packages/react/src/toast/content/ToastContent.test.tsx:43-51`
- `Toast.Content` can wrap other parts as children (fixture nests `Toast.Title` inside `Toast.Content`). `packages/react/src/toast/content/ToastContent.test.tsx:46-48`
- `Toast.Arrow` and `Toast.Title` render outside their required context (Provider-only, or Provider+Viewport without Root) cause the render to reject with descriptive errors:
  - Arrow: `'Base UI: ToastPositionerContext is missing. ToastPositioner parts must be placed within <Toast.Positioner>.'` `packages/react/src/toast/arrow/ToastArrow.test.tsx:72-88`
  - Title: `'Base UI: ToastRootContext is missing. Toast parts must be used within <Toast.Root>.'` `packages/react/src/toast/title/ToastTitle.test.tsx:29-47`
- UNVERIFIED — inferred from `packages/react/src/toast/content/ToastContent.test.tsx:43-51`, no test asserts toasts are portaled to `document.body` (viewport is rendered inline in the test tree).

## Events (names, payload shape, bubbling, preventDefault semantics)

- Click on `Toast.Close` removes the toast from the viewport (title element no longer present after click). `packages/react/src/toast/close/ToastClose.test.tsx:30-56`
- Clicking a toast's action area is exercised after adding a toast whose `actionProps` supply `id`/`children`; the observable assertion is the rendered action element carrying `id='action'`. No custom event name, payload, bubbling, or `preventDefault` semantics are asserted. `packages/react/src/toast/action/ToastAction.test.tsx:30-45`
- `mouseEnter` on the viewport toggles `data-expanded` on `Toast.Content` (no event-object semantics asserted). `packages/react/src/toast/content/ToastContent.test.tsx:84-85`
- UNVERIFIED — inferred from `packages/react/src/toast/action/ToastAction.test.tsx:30-45`, no test asserts what happens when the action element itself is clicked (the test clicks the "add" button, not the action button).

## Edge cases (rapid interactions, unmount, nesting)

- Rapid/sequential adds: two consecutive `fireEvent.click` calls on the add button create two toasts; the stack marks the older one `data-behind` and the newest front one without it. `packages/react/src/toast/content/ToastContent.test.tsx:63-70`
- Missing content suppresses rendering entirely: `Toast.Action` with `actionProps.children: undefined` renders nothing. `packages/react/src/toast/action/ToastAction.test.tsx:47-80`; `Toast.Description` with `description: undefined` renders nothing. `packages/react/src/toast/description/ToastDescription.test.tsx:49-73`; `Toast.Title` with `title: undefined` renders nothing. `packages/react/src/toast/title/ToastTitle.test.tsx:70-94`
- A childless render-prop element with no underlying content also renders nothing (Action and Title both covered). `packages/react/src/toast/action/ToastAction.test.tsx:96-108`, `packages/react/src/toast/title/ToastTitle.test.tsx:158-170`
- A render function returning `null` renders nothing while the parent `Toast.Root` still renders. `packages/react/src/toast/title/ToastTitle.test.tsx:186-199`
- Numeric `0` is valid title content and renders as text `'0'`. `packages/react/src/toast/title/ToastTitle.test.tsx:172-184`
- Context-missing throws (see DOM structure) occur at render time and reject the render promise; the tests silence `console.error` during them and restore it in `finally`. `packages/react/src/toast/arrow/ToastArrow.test.tsx:72-88`, `packages/react/src/toast/title/ToastTitle.test.tsx:29-47`
- Unmount ordering: when an older Title unmounts while a newer one stays, the root's `aria-labelledby` must remain the newer title's id — older cleanups must not clear newer registrations. `packages/react/src/toast/title/ToastTitle.test.tsx:229-261`
- `isRenderableNode` truthiness rules (drives the "no children → no render" behavior):
  - Renderable primitives: `0`, `0n` (BigInt), `Number.NaN`, and non-empty strings are renderable (truthy for rendering). `packages/react/src/toast/utils/isRenderableNode.test.ts:6-11`
  - Non-rendering values: `null`, `undefined`, `true`, `false`, and `''` are not renderable. `packages/react/src/toast/utils/isRenderableNode.test.ts:13-19`
  - Arrays recurse: `[]`, `[null, undefined, false]`, `[[null]]` are not renderable; `[null, 0]` and `[[0]]` (nested arrays) are. `packages/react/src/toast/utils/isRenderableNode.test.ts:21-27`
  - `hasRenderableChildren` requires an element whose children are renderable: a `<div>` with `'text'` or `0` is true; a bare `<div>`, `<div>` with `[]`, `null`, or a raw string are false. `packages/react/src/toast/utils/isRenderableNode.test.ts:30-38`
- UNVERIFIED — inferred from `packages/react/src/toast/utils/isRenderableNode.test.ts:21-27`, no test asserts arbitrarily deep array nesting beyond two levels.

## Shared harness dependencies

- `packages/react/test/index.ts` — barrel re-exporting `createRenderer`, `describeConformance`, `isJSDOM` (via `@base-ui/utils/testUtils`), and other helpers used by these tests. `packages/react/test/index.ts:1-11`
- `packages/react/test/createRenderer.ts` — wraps rendering in `act`, and the returned handle exposes `await`-able `rerender`/`setProps`; all toast part tests obtain `render` from it. `packages/react/test/createRenderer.ts:27-49`, `packages/react/src/toast/title/ToastTitle.test.tsx:14`
- `packages/react/test/describeConformance.tsx` — defines the conformance suite run for each part: `propsSpread` (testPropForwarding), `refForwarding` (testRefForwarding), `renderProp` (testRenderProp), `className` (testClassName), filtered by `only`/`skip`. `packages/react/test/describeConformance.tsx:44-68`
- `packages/react/test/conformanceTests/refForwarding.tsx` — clones the minimal element with a `React.createRef()` and asserts `ref.current` is an instance of the declared `refInstanceof` (e.g. `HTMLButtonElement` for Action/Close). `packages/react/test/conformanceTests/refForwarding.tsx:31-39`
- `packages/react/test/conformanceTests/propForwarding.tsx` — asserts custom props (`lang`, `data-foobar`) land on the default element and on custom elements rendered via function or JSX `render`, and that `style` forwards from the component, render function, and render element. `packages/react/test/conformanceTests/propForwarding.tsx:23-127`
- `packages/react/test/conformanceTests/renderProp.tsx` — asserts function and element `render` produce a custom root (including through a wrapper component), the component ref reaches the custom element, component/render-element refs merge, and component and render `className`s (string or function-resolved) merge. `packages/react/test/conformanceTests/renderProp.tsx:41-178`
- `packages/react/test/conformanceTests/className.tsx` — asserts a string `className` is applied to the rendered element. `packages/react/test/conformanceTests/className.tsx:20-23`
- `packages/react/src/toast/utils/test-utils.tsx` — toast-local fixtures: `Button` calls `Toast.useToastManager().add({ title: 'title', description: 'description', actionProps: { id: 'action', children: 'action' } })`; `List` maps toasts to `Toast.Root data-testid="root"` containing `Toast.Title data-testid="title"`, `Toast.Description data-testid="description"`, `Toast.Close aria-label="close-press"`, and `Toast.Action data-testid="action"` (all childless, so they source content from the toast object). `packages/react/src/toast/utils/test-utils.tsx:6-39`
- `isJSDOM` (used by the Arrow side-mirroring test via `it.skipIf(isJSDOM)`) comes from the harness barrel, restricting that layout-dependent test to the Chromium environment. `packages/react/src/toast/arrow/ToastArrow.test.tsx:5`, `packages/react/src/toast/arrow/ToastArrow.test.tsx:26`
