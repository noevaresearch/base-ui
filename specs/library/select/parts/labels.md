# Select — Group, GroupLabel, Label: behavior mined from tests

Scope: exactly three test files —
`packages/react/src/select/group/SelectGroup.test.tsx`, `packages/react/src/select/group-label/SelectGroupLabel.test.tsx`, `packages/react/src/select/label/SelectLabel.test.tsx`.
Only behavior asserted by these files (plus the shared harness in `packages/react/test/` they invoke) is described. Everything else is marked UNVERIFIED.

## Public API surface (props, parts, subcomponents)

Parts exercised via the `Select` namespace import (`import { Select } from '@base-ui/react/select'`):

- `Select.Group` — used as `<Select.Group>` containing a `Select.GroupLabel` and `Select.Item` children (`packages/react/src/select/group/SelectGroup.test.tsx:20-24`). It is also rendered standalone under `Select.Root` with no `Positioner` in the Group conformance scaffold (`packages/react/src/select/group/SelectGroup.test.tsx:11-13`) and in every GroupLabel test scaffold (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:14-17`, `:23-27`).
- `Select.GroupLabel` — used as `<Select.GroupLabel>` with text children (`packages/react/src/select/group/SelectGroup.test.tsx:21`), with an explicit `id` prop (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:68`, `:91-93`, `:96-98`, `:162-164`), with a `ref` prop (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:131`), with an `aria-hidden` prop (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:37`), and with no children (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:52`).
- `Select.Label` — used only inside the conformance scaffold, with no props other than those the conformance suite clones onto it (`packages/react/src/select/label/SelectLabel.test.tsx:8-21`). No behavioral (non-conformance) test exists for `Select.Label` in this batch; only conformance assertions apply to it.
- `Select.Root` — used with the boolean `open` prop (`packages/react/src/select/group/SelectGroup.test.tsx:18`, `packages/react/src/select/group-label/SelectGroupLabel.test.tsx:14`, `:23`, `:36`) and without it (closed) in the Label conformance scaffold (`packages/react/src/select/label/SelectLabel.test.tsx:12`).
- Scaffolding parts appearing in these tests: `Select.Positioner` (`packages/react/src/select/group/SelectGroup.test.tsx:19`, `:25`, `:36`, `:43`), `Select.Trigger` and `Select.Portal` (`packages/react/src/select/label/SelectLabel.test.tsx:14-17`), `Select.Item` (`packages/react/src/select/group/SelectGroup.test.tsx:22-23`, `:39-40`).

Props proven per part (all via the conformance suite, which runs its full default suite — `propsSpread`, `refForwarding`, `renderProp`, `className` — because neither test file passes `only` or `skip`; see `packages/react/test/describeConformance.tsx:44-68`):

- `ref` — forwarded to a DOM element that is `instanceof window.HTMLDivElement` for all three parts (`packages/react/src/select/group/SelectGroup.test.tsx:10`, `packages/react/src/select/group-label/SelectGroupLabel.test.tsx:11`, `packages/react/src/select/label/SelectLabel.test.tsx:9`; assertion logic in `packages/react/test/conformanceTests/refForwarding.tsx:32-38`).
- Arbitrary HTML props spread onto the rendered element (`lang`, `data-foobar`, `data-testid`) — proven for Group, GroupLabel, and Label through the shared suite (`packages/react/test/conformanceTests/propForwarding.tsx:23-36`, `:38-59`, `:61-80`).
- `style` object merged onto the rendered element (`packages/react/test/conformanceTests/propForwarding.tsx:82-127`).
- `className` string applied (`packages/react/test/conformanceTests/className.tsx:20-23`).
- `render` prop (function or element) replacing the rendered root, with ref and className merging (`packages/react/test/conformanceTests/renderProp.tsx:41-76`, `:93-144`, `:146-178`).

The `render` prop wrapper used by the GroupLabel conformance is `<Select.Root open><Select.Group>{node}</Select.Group></Select.Root>` (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:12-18`); for Label it is a closed `Select.Root` with `Select.Trigger` and `Select.Portal > Select.Positioner` siblings (`packages/react/src/select/label/SelectLabel.test.tsx:10-20`).

UNVERIFIED — inferred from `packages/react/src/select/label/SelectLabel.test.tsx:8-21`, no test asserts this: `Select.Label` renders anything with specific role, text behavior, or event/focus behavior; the Label file contains only conformance tests.

## State model (controlled/uncontrolled, defaults, transitions)

- The only stateful-looking prop used in this batch is `Select.Root open`, always passed as a literal `true` in the Group/GroupLabel tests (`packages/react/src/select/group/SelectGroup.test.tsx:18`, `packages/react/src/select/group-label/SelectGroupLabel.test.tsx:14`) — whether `open` is controlled/uncontrolled or its transitions is UNVERIFIED — inferred from `packages/react/src/select/group/SelectGroup.test.tsx:18`, no test asserts this.
- Group ↔ GroupLabel registration state (the one transition model these tests actually pin down):
  - When a `GroupLabel` with explicit `id="group-label"` is mounted inside a `Group`, the group element carries `aria-labelledby="group-label"`; after the label unmounts (rerender with `labelMounted={false}`), the group element has no `aria-labelledby` attribute at all (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:74-82`).
  - When two labels are mounted simultaneously (`key="old"` id `old-label`, `key="new"` id `new-label`), the group's `aria-labelledby` is `new-label` — the newest label wins; removing the old label afterwards keeps `aria-labelledby="new-label"` (the older label's cleanup does not clobber the newer registration) (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:105-114`).
  - With no explicit `id`, the group's `aria-labelledby` equals the label's generated `id` (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:139-144`); passing `id="custom-label"` switches `aria-labelledby` to the explicit id (`:146-147`); removing the explicit `id` switches it back to the same generated id value captured before (`:151-152`).
  - Under `React.StrictMode` with a keyed label (`key={labelId}` `id={labelId}`), swapping the rendered label from `first-label` to `second-label` updates `aria-labelledby` to `second-label`, and unmounting the label removes the attribute (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:172-181`).
- No open/close, value, or item-selection state is exercised in this batch: UNVERIFIED — inferred from `packages/react/src/select/group/SelectGroup.test.tsx:16-49`, no test asserts this (the `Select.Item` children are never interacted with).

## Keyboard interactions

N/A — no keyboard interactions are simulated or asserted anywhere in this batch (`packages/react/src/select/group/SelectGroup.test.tsx:1-50`, `packages/react/src/select/group-label/SelectGroupLabel.test.tsx:1-183`, `packages/react/src/select/label/SelectLabel.test.tsx:1-22`).

## Focus management

N/A — no focus is queried, moved, or asserted in this batch (`packages/react/src/select/group/SelectGroup.test.tsx:1-50`, `packages/react/src/select/group-label/SelectGroupLabel.test.tsx:1-183`, `packages/react/src/select/label/SelectLabel.test.tsx:1-22`). The only ref-related lifecycle assertion is that when the label's `ref` function is replaced, the old ref is last called with `null` and the new ref is last called with the label's DOM element (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:146-149`).

## Accessibility (roles, aria-*, id linking)

- `Select.Group` renders an element with role `group` (queried via `screen.getByRole('group')`) that carries an `aria-labelledby` attribute (`packages/react/src/select/group/SelectGroup.test.tsx:29`, `:46`).
- The group's `aria-labelledby` value is exactly the `id` of the rendered `GroupLabel` element, linking group → label (`packages/react/src/select/group/SelectGroup.test.tsx:46-48`).
- `Select.GroupLabel` is exposed with `aria-hidden="true"` by default (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:21-31`). Passing `aria-hidden={undefined}` explicitly results in the rendered label having no `aria-hidden` attribute at all — the default does not win over an explicit `undefined` (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:33-43`).
- The `aria-labelledby` attribute is present on the group only while a label is registered: it tracks the current label's id through mount, replacement (explicit vs generated ids), and unmount, including under `React.StrictMode` (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:74-82`, `:105-114`, `:139-152`, `:172-181`).
- Rendering `Select.GroupLabel` outside a `Select.Group` (but inside `Select.Root open`) rejects rendering with the error `Base UI: SelectGroupContext is missing. SelectGroup parts must be placed within <Select.Group>.` (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:49-57`; `console.error` is spied/silenced and restored around it, `:46`, `:59`).
- UNVERIFIED — inferred from `packages/react/src/select/group/SelectGroup.test.tsx:30`, no test asserts this: the visible/hidden visibility semantics of the label beyond the jest-dom `toBeVisible()` check; the test asserts `screen.getByText('Fruits')` is visible in an open Select, and separately that the label element carries `aria-hidden="true"` (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:30`).

## DOM structure & portal behavior

- All three parts render real DOM elements whose refs are `instanceof window.HTMLDivElement` — Group (`packages/react/src/select/group/SelectGroup.test.tsx:10`), GroupLabel (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:11`), Label (`packages/react/src/select/label/SelectLabel.test.tsx:9`) — asserted by the ref conformance test (`packages/react/test/conformanceTests/refForwarding.tsx:32-38`).
- `Select.Group` with a `GroupLabel` is rendered inside `Select.Positioner` inside `Select.Root open` in the behavioral Group tests (`packages/react/src/select/group/SelectGroup.test.tsx:18-26`, `:35-43`).
- `Select.Group` also renders and is queryable (`getByRole('group')`) when placed directly under `Select.Root open` with no `Positioner` — this is the scaffold of every GroupLabel test and of the Group/GroupLabel conformance suites (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:14-17`, `:23-27`, `packages/react/src/select/group/SelectGroup.test.tsx:11-13`).
- The GroupLabel unmount tests find the group element once (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:76`, `:107`, `:141`, `:174`) and keep asserting on that same element across `rerender` calls, implying the group DOM node is stable across label mount/unmount/swap in those scenarios (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:79-82`, `:110-114`).
- Portal behavior itself is not asserted anywhere in this batch; `Select.Portal > Select.Positioner` appears only as conformance scaffolding for `Select.Label` (`packages/react/src/select/label/SelectLabel.test.tsx:15-17`). Any portal semantics are UNVERIFIED — inferred from `packages/react/src/select/label/SelectLabel.test.tsx:15-17`, no test asserts this.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are fired, dispatched, or asserted in this batch; no `on*` handler props appear in any of the three files (`packages/react/src/select/group/SelectGroup.test.tsx:1-50`, `packages/react/src/select/group-label/SelectGroupLabel.test.tsx:1-183`, `packages/react/src/select/label/SelectLabel.test.tsx:1-22`).

## Edge cases (rapid interactions, unmount, nesting)

- Unmount: removing the `GroupLabel` from a mounted group removes the group's `aria-labelledby` attribute entirely and removes the label text from the document (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:79-82`).
- Replacement / stale-cleanup race: mounting a second label while the first is still mounted moves `aria-labelledby` to the second label's id; unmounting the first label afterwards must not clear the second label's registration (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:105-114`).
- Ref churn: replacing the label's `ref` function across rerenders does not disturb id tracking — the old ref receives `null` as its last call, the new ref receives the label element, and `aria-labelledby` keeps following the explicit/generated id transitions (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:146-152`).
- Strict Mode: under `React.StrictMode`, replacing a keyed label (new key/id) updates the group's `aria-labelledby` to the new id, and unmounting the label removes the attribute (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:155-181`).
- Mis-nesting: `GroupLabel` directly under `Select.Root` without a `Select.Group` ancestor throws/rejects with the `SelectGroupContext is missing` message (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:49-57`).
- Group containing items: a `Group` with `GroupLabel` plus two `Select.Item`s renders with the group role and visible label text (`packages/react/src/select/group/SelectGroup.test.tsx:16-31`).
- Rapid interactions: N/A — no rapid/pointer/timing interactions are tested in this batch (`packages/react/src/select/group/SelectGroup.test.tsx:1-50`, `packages/react/src/select/group-label/SelectGroupLabel.test.tsx:1-183`, `packages/react/src/select/label/SelectLabel.test.tsx:1-22`).

## Shared harness dependencies

These tests import `createRenderer` and `describeConformance` from `#test-utils`, which resolves to `packages/react/test/index.ts` (barrel) — the tests use only `createRenderer` and `describeConformance` from it; `screen` comes from the external `@mui/internal-test-utils` package (`packages/react/src/select/group/SelectGroup.test.tsx:3-4`, `packages/react/src/select/group-label/SelectGroupLabel.test.tsx:4-5`, `packages/react/src/select/label/SelectLabel.test.tsx:3`).

Harness files read:

- `packages/react/test/index.ts` — barrel re-exporting the harness: `advanceReactClock`, `createRenderer`, `describeConformance`, pointer helpers (`enterWithMouse`, `firePointer`, `moveMouse`), `mergeRefs`, `popupConformanceTests`, `resetBrowserPointer`, `useTestInteractions`, everything in `./wait`, `waitForPositioned`, and `describeGregorianAdapter` (`packages/react/test/index.ts:1-14`). Also re-exports `*` from `@base_ui/utils/testUtils` (`packages/react/test/index.ts:1`).
- `packages/react/test/createRenderer.ts` — wraps `@mui/internal-test-utils`'s `createRenderer`; its `render(element, options?)` awaits `act(async () => originalRender(...))` and returns the result with `rerender` and `setProps` also wrapped in `act` (async, promise-returning), plus a widened element type allowing arbitrary `data-*` attributes (`packages/react/test/createRenderer.ts:27-49`). This is why the tests can `await render(...)` and `await rerender(...)` synchronously-flushed (`packages/react/src/select/group-label/SelectGroupLabel.test.tsx:74-79`).
- `packages/react/test/describeConformance.tsx` — `describeConformance(minimalElement, getOptions)` registers a `describe('Base UI component API', ...)` block and runs the `fullSuite` keys `propsSpread → testPropForwarding`, `refForwarding → testRefForwarding`, `renderProp → testRenderProp`, `className → testClassName`, filtered by optional `only`/`skip` options and running an optional `after` hook via `afterAll` (`packages/react/test/describeConformance.tsx:44-70`). Options accepted include `refInstanceof`, `render`, `skip`, `testRenderPropWith`, `button`, `wrappingAllowed` (`packages/react/test/describeConformance.tsx:23-42`). None of the three test files pass `skip`/`only`, so all four suites run for Group, GroupLabel, and Label (`packages/react/src/select/group/SelectGroup.test.tsx:9-14`, `packages/react/src/select/group-label/SelectGroupLabel.test.tsx:10-19`, `packages/react/src/select/label/SelectLabel.test.tsx:8-21`).
  - `packages/react/test/conformanceTests/refForwarding.tsx` — clones the minimal element with a fresh `React.createRef()`, renders it via the provided `render`, and asserts `ref.current` is `instanceof` the options' `refInstanceof` (`packages/react/test/conformanceTests/refForwarding.tsx:9-38`).
  - `packages/react/test/conformanceTests/propForwarding.tsx` — asserts custom props (`lang`, `data-foobar`) land on the default element (`data-testid="root"`) (`:23-36`), on a function-`render` custom element (`:38-59`), and on a JSX-`render` custom element (`:61-80`); also asserts a `style` object is merged onto the rendered element in all three forms (`:82-127`). Each case runs `await flushMicrotasks()` after render (`:31`, `:54`, `:75`, `:90`, `:109`, `:122`).
  - `packages/react/test/conformanceTests/className.tsx` — asserts a `className` string passed to the component appears in the document (`packages/react/test/conformanceTests/className.tsx:20-23`).
  - `packages/react/test/conformanceTests/renderProp.tsx` — asserts the `render` prop accepts a function or element and renders the custom root (including an optional wrapper element when `wrappingAllowed`, default true) (`:41-76`, `:27-38`); that a component-level `ref` reaches the custom rendered element (`:93-113`); that component ref and render-element ref are both called with the rendered node (`:115-144`); and that `className` from both component and render element (string or function-resolved) merge onto one element (`:146-178`).
- `packages/react/test/wait.ts` — exports `wait(ms)` (setTimeout-based promise) and `waitSingleFrame()` (requestAnimationFrame-based promise) (`packages/react/test/wait.ts:4-18`). Exported by the barrel but not used by any of the three test files in this batch (`packages/react/test/index.ts:10`).
