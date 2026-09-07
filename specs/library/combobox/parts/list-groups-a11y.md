# Combobox parts — List, Group, GroupLabel, Label, Empty, Status (batch: list-groups-a11y)

Behavior-only spec mined from test files. Every non-trivial claim cites the test lines that prove it. Claims without a covering test are marked `UNVERIFIED`.

## Public API surface (props, parts, subcomponents)

- `Combobox.List` is a composable part used inside `Combobox.Root` (conformance suite wraps it in `Combobox.Root`); it forwards its ref to an `HTMLDivElement`. `packages/react/src/combobox/list/ComboboxList.test.tsx:10-15`
- `Combobox.List` accepts a `grid` prop; when set, the list element is exposed with role `grid` and items are wrapped in `Combobox.Row` parts. `packages/react/src/combobox/list/ComboboxList.test.tsx:54-75`
- `Combobox.List` supports a render-function child of the shape `(item) => <Combobox.Item value={item} />` when `Combobox.Root` receives an `items` array. `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:36-42`
- `Combobox.Group` forwards its ref to an `HTMLDivElement` and is rendered inside `Combobox.Root` in an open state for conformance. `packages/react/src/combobox/group/ComboboxGroup.test.tsx:9-14`
- `Combobox.GroupLabel` forwards its ref to an `HTMLDivElement` and is rendered inside `Combobox.Root` + `Combobox.Group` for conformance. `packages/react/src/combobox/group-label/ComboboxGroupLabel.test.tsx:10-19`
- `Combobox.GroupLabel` accepts an `id` prop, and the provided id is used verbatim. `packages/react/src/combobox/group-label/ComboboxGroupLabel.test.tsx:78-95`
- `Combobox.GroupLabel` accepts `aria-hidden` as a passthrough prop: `aria-hidden={undefined}` results in no `aria-hidden` attribute on the element. `packages/react/src/combobox/group-label/ComboboxGroupLabel.test.tsx:60-76`
- `Combobox.Label` forwards its ref to an `HTMLDivElement` and composes with `Combobox.Trigger` and a portaled `Combobox.Input` in the conformance setup. `packages/react/src/combobox/label/ComboboxLabel.test.tsx:9-26`
- `Combobox.Empty` forwards its ref to an `HTMLDivElement`, is rendered inside `Combobox.Popup` next to `Combobox.List`, accepts children, and supports the `render` prop (e.g. `render={<p data-testid="custom-empty" />}` renders a `<p>`). `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:10-26` and `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:188-212`
- `Combobox.Status` forwards its ref to an `HTMLDivElement`, can be rendered directly under `Combobox.Root` (outside any portal), accepts children, and supports the `render` prop (e.g. `render={<p data-testid="custom-status" />}` renders a `<p>`). `packages/react/src/combobox/status/ComboboxStatus.test.tsx:11-16` and `packages/react/src/combobox/status/ComboboxStatus.test.tsx:85-111`
- An internal `useComboboxGroupContext` hook exists for Group parts and throws when used outside `<Combobox.Group>`. `packages/react/src/combobox/group/ComboboxGroupContext.test.tsx:8-23`
- `Combobox.List` behavior is shared with the sibling `Autocomplete` component: `Autocomplete.List` with a readOnly root also gets `aria-readonly="true"` on its listbox. `packages/react/src/combobox/list/ComboboxList.test.tsx:95-111`

## State model (controlled/uncontrolled, defaults, transitions)

- `Combobox.Root` drives rendering state via props used in these tests: `multiple`, `readOnly`, `disabled`, `grid`, `open`/`defaultOpen`, `items`, `defaultInputValue`, and `onValueChange`. `packages/react/src/combobox/list/ComboboxList.test.tsx:17-19`, `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:30` and `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:79`
- `Combobox.Status` visibility tracks popup open state: it is absent from the DOM while the popup is closed and appears once the popup opens (via `defaultOpen` or clicking the input). `packages/react/src/combobox/status/ComboboxStatus.test.tsx:18-38`
- Live-region text-mutation state machine (Empty and Status): on initial mount with visible text, a WORD JOINER marker (`\u2060`) is appended to the text content; after `INITIAL_LIVE_REGION_TEXT_MUTATION_RESET_DELAY` (fake timer tick) the marker is removed and the original text remains. `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:145-151` and `packages/react/src/combobox/status/ComboboxStatus.test.tsx:66-72`
- When the live region mounts with no content and content is added later (via rerender), the text appears immediately without waiting for the reset delay. `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:153-186` and `packages/react/src/combobox/status/ComboboxStatus.test.tsx:74-83`
- If the component unmounts while still inside the reset-delay window, the marker is removed before unmount so the text ends up clean. `packages/react/src/combobox/status/ComboboxStatus.test.tsx:113-122`
- On iOS platforms, the initial text mutation is skipped entirely: text stays clean at mount and remains clean after the reset delay elapses. `packages/react/src/combobox/status/ComboboxStatus.iOS.test.tsx:25-48`

## Keyboard interactions

- A `keydown` for `Enter` dispatched on the list is NOT default-prevented when no item is highlighted. `packages/react/src/combobox/list/ComboboxList.test.tsx:113-133`
- After `ArrowDown` highlights an item, an `Enter` keydown dispatched on the list selects the highlighted item: `onValueChange` is called with the item value (`'a'`) plus an event argument. `packages/react/src/combobox/list/ComboboxList.test.tsx:135-160`
- When the root is `disabled` or `readOnly`, `Enter` on the list does not select: `onValueChange` is never called. `packages/react/src/combobox/list/ComboboxList.test.tsx:162-190`

## Focus management

- N/A — no test in this batch asserts focus placement, focus movement, or focus return. The Enter-key tests dispatch synthetic `KeyboardEvent`s directly on the list element (`bubbles: true`) rather than moving focus. `packages/react/src/combobox/list/ComboboxList.test.tsx:129-132` and `packages/react/src/combobox/list/ComboboxList.test.tsx:155-157`
- UNVERIFIED — inferred from `packages/react/src/combobox/label/ComboboxLabel.test.tsx:9-26`, no test asserts this: whether `Combobox.Label` sets `htmlFor`/`id` linking to the popup `Combobox.Input`; the only labeling behavior proven is a development warning (see Accessibility).

## Accessibility (roles, aria-*, id linking)

- The list element gets `role="listbox"`, and in `multiple` mode it sets `aria-multiselectable="true"`. `packages/react/src/combobox/list/ComboboxList.test.tsx:17-34`
- With a `readOnly` root, the listbox gets `aria-readonly="true"`; by default (non-readOnly) the attribute is absent. `packages/react/src/combobox/list/ComboboxList.test.tsx:36-52` and `packages/react/src/combobox/list/ComboboxList.test.tsx:77-93`
- In `grid` mode, `aria-readonly` is deliberately NOT set on the grid (because it would describe cell editability); instead the `readOnly` state is conveyed by `aria-readonly="true"` on the `Combobox.Input` (which has role combobox). `packages/react/src/combobox/list/ComboboxList.test.tsx:54-75`
- `Combobox.Group` renders with `role="group"` and an `aria-labelledby` attribute; its value is exactly the `Combobox.GroupLabel` element's id, and the label text remains visible. `packages/react/src/combobox/group/ComboboxGroup.test.tsx:16-33` and `packages/react/src/combobox/group/ComboboxGroup.test.tsx:35-53`
- `Combobox.GroupLabel` is `aria-hidden="true"` by default (hidden from the accessibility tree) while still wired into the group's `aria-labelledby`. `packages/react/src/combobox/group-label/ComboboxGroupLabel.test.tsx:22-58`
- When two GroupLabels are rendered, `aria-labelledby` reflects the newest label; when the older label unmounts, its cleanup does not clear the newer label's id from `aria-labelledby` (`old` → `old-label`, `both` → `new-label`, `new` → `new-label`). `packages/react/src/combobox/group-label/ComboboxGroupLabel.test.tsx:97-127`
- `Combobox.Empty` renders with `role="status"` (it is the element matched by `getByRole('status')`) whenever its text is shown. `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:49-51` and `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:145`
- `Combobox.Status` renders with `role="status"` and is the element matched by `getByRole('status')`. `packages/react/src/combobox/status/ComboboxStatus.test.tsx:66-67` and `packages/react/src/combobox/status/ComboboxStatus.test.tsx:77-78`
- `Combobox.Label` emits a development-mode `console.error` containing `<Combobox.Label> labels <Combobox.Trigger> only.` when labeling alongside a `Combobox.Input`; the warning works without relying on `React.captureOwnerStack`. `packages/react/src/combobox/label/ComboboxLabel.test.tsx:28-54`
- The `Combobox.Label` development warning is suppressed in production (`NODE_ENV=production`): neither the error nor `captureOwnerStack` runs. `packages/react/src/combobox/label/ComboboxLabel.test.tsx:56-85`
- UNVERIFIED — inferred from `packages/react/src/combobox/label/ComboboxLabel.test.tsx:44-46`, no test asserts this: any aria/id wiring from `Combobox.Label` to the input itself (e.g. `htmlFor`).

## DOM structure & portal behavior

- List, Group, GroupLabel, Empty, and Status are all exercised inside the `Combobox.Portal` → `Combobox.Positioner` → `Combobox.Popup` tree in behavioral tests. `packages/react/src/combobox/list/ComboboxList.test.tsx:20-28`, `packages/react/src/combobox/group/ComboboxGroup.test.tsx:19-27`, `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:32-45`, `packages/react/src/combobox/status/ComboboxStatus.test.tsx:22-31`
- `Combobox.Status` may also be rendered directly under `Combobox.Root` (no portal/popup), as its conformance setup does. `packages/react/src/combobox/status/ComboboxStatus.test.tsx:11-15`
- `Combobox.Status` is completely absent from the DOM while the popup is closed (query returns null) and is mounted once open. `packages/react/src/combobox/status/ComboboxStatus.test.tsx:35-37`
- When items exist, the `Combobox.Empty` element's visible text is not in the document (`queryByText(/No results/)` is null). `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:53-75` and `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:101-123`
- However, in the live-region mechanics tests the Empty element itself is mounted with empty text content while items exist, and the text appears after the items become empty — i.e. content is suppressed, not the element, in that flow. `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:153-186`
- The `render` prop replaces the host element while keeping the `role="status"` live-region semantics on the custom element (a `<p>` becomes the status node). `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:195-207` and `packages/react/src/combobox/status/ComboboxStatus.test.tsx:92-106`

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onValueChange` is invoked with the selected value as the first argument and an event-like object as the second (`toHaveBeenCalledWith('a', expect.anything())`) when Enter selects the highlighted item. `packages/react/src/combobox/list/ComboboxList.test.tsx:135-160`
- Keyboard events are handled on the list element with bubbling semantics: tests dispatch `new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true })` directly on the listbox node. `packages/react/src/combobox/list/ComboboxList.test.tsx:130-131` and `packages/react/src/combobox/list/ComboboxList.test.tsx:156-157` and `packages/react/src/combobox/list/ComboboxList.test.tsx:181-186`
- `Enter` handling respects `cancelable`/preventDefault semantics in the sense that the handler leaves `event.defaultPrevented === false` when there is no highlighted item to select. `packages/react/src/combobox/list/ComboboxList.test.tsx:113-133`
- No pointer or change events on these parts are asserted in this batch; no custom event names beyond value-change callbacks are proven. `packages/react/src/combobox/list/ComboboxList.test.tsx:135-190`

## Edge cases (rapid interactions, unmount, nesting)

- Label-swap race under `Combobox.Group`: rendering an old GroupLabel, adding a new one, then removing the old one must end with `aria-labelledby="new-label"` — an older label's cleanup must not clear a newer label's id. `packages/react/src/combobox/group-label/ComboboxGroupLabel.test.tsx:97-127`
- Unmount during the live-region reset-delay window: the WORD JOINER marker is restored/removed so the detached element's text ends up as the clean string (`'Searching…'`). `packages/react/src/combobox/status/ComboboxStatus.test.tsx:113-122`
- Context misuse: calling `useComboboxGroupContext` outside `<Combobox.Group>` rejects rendering with the exact error `Base UI: ComboboxGroupContext is missing. ComboboxGroup parts must be placed within <Combobox.Group>.` `packages/react/src/combobox/group/ComboboxGroupContext.test.tsx:8-23`
- Filtering × empty interplay: with `items={['a','b','c']}` and `defaultInputValue="d"` (no matches) the Empty content shows; with `defaultInputValue="c"` (a match) it does not — i.e. Empty reacts to filtered results, not just the raw items array. `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:77-123`
- Platform-dependent behavior: the live-region initial text mutation is gated on platform detection; tests simulate iOS by mocking `@base-ui/utils/platform` (`os.ios: true, os.apple: true`) and confirm the marker is skipped. `packages/react/src/combobox/status/ComboboxStatus.iOS.test.tsx:7-18` and `packages/react/src/combobox/status/ComboboxStatus.iOS.test.tsx:25-48`
- Delay-dependent behavior is tested under fake timers (`clock.withFakeTimers()`) with `clock.tick(INITIAL_LIVE_REGION_TEXT_MUTATION_RESET_DELAY)` driving the transition. `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:125-128`, `packages/react/src/combobox/status/ComboboxStatus.test.tsx:40-43`, `packages/react/src/combobox/status/ComboboxStatus.iOS.test.tsx:21-23`
- The marker is exactly the WORD JOINER character `\u2060` appended to the existing text (`'No results\u2060'`, `'Searching…\u2060'`). `packages/react/src/combobox/empty/ComboboxEmpty.test.tsx:146` and `packages/react/src/combobox/status/ComboboxStatus.test.tsx:67`

## Shared harness dependencies

- `packages/react/test/index.ts` — barrel re-exporting `createRenderer`, `describeConformance`, `isJSDOM` (from `@base-ui/utils/testUtils`), and other helpers used by these tests. `packages/react/test/index.ts:1-11`
- `packages/react/test/createRenderer.ts` — wraps `@mui/internal-test-utils`'s render in `act`, and provides act-wrapped `rerender`/`setProps` on the result. `packages/react/test/createRenderer.ts:27-49`
- `packages/react/test/describeConformance.tsx` — runs the standard conformance suite (props spread, ref forwarding, render prop, className) for each part, driven by the `refInstanceof`/`render` options in each test file. `packages/react/test/describeConformance.tsx:44-68`
