# Toggle — behavior spec (Stage 1: behavior mining)

Unit: `toggle` (packages/react/src/toggle). Mined from the unit's own test suite only:
`packages/react/src/toggle/Toggle.test.tsx` (single file, no batching; the TODO.md entry for `library: toggle` has no `wraps-external:` field and no `needs-batched-mining: true`).

## Public API surface (props, parts, subcomponents)

- `Toggle` renders a native `<button>` element: the conformance suite asserts the forwarded ref is an `instanceof window.HTMLButtonElement` and that a plain `'button'` can substitute for the component via `testComponentPropWith` (`packages/react/src/toggle/Toggle.test.tsx:11-16`).
- Props exercised by the suite: `pressed` (controlled state, `packages/react/src/toggle/Toggle.test.tsx:25`), `defaultPressed` (uncontrolled initial state, `packages/react/src/toggle/Toggle.test.tsx:49`), `onPressedChange` (state-change callback, `packages/react/src/toggle/Toggle.test.tsx:72`), `disabled` (`packages/react/src/toggle/Toggle.test.tsx:132`), `value` (group membership identifier, `packages/react/src/toggle/Toggle.test.tsx:109`), and `render` (custom rendering, `packages/react/src/toggle/Toggle.test.tsx:164`).
- The conformance harness (`describeConformance`) additionally proves ref forwarding, props spreading, `className` handling, and the `render` prop as function/element — these are run for Toggle via the `button: true` configuration (`packages/react/src/toggle/Toggle.test.tsx:11-16`); the assertions live in the shared harness (see "Shared harness dependencies").
- No subcomponents of Toggle itself are tested. The suite uses `ToggleGroup` (imported from `../toggle-group/ToggleGroup`, `packages/react/src/toggle/Toggle.test.tsx:6`) purely as a parent fixture to exercise grouped behavior; ToggleGroup's own behavior is out of scope for this unit.

## State model (controlled/uncontrolled, defaults, transitions)

- Controlled: the `pressed` prop fully determines the rendered state — when external state flips, `aria-pressed` flips from `'false'` to `'true'` and back without any Toggle interaction (`packages/react/src/toggle/Toggle.test.tsx:34-45`).
- Uncontrolled: `defaultPressed={false}` seeds the internal state; the rendered `aria-pressed` starts as `'false'` (`packages/react/src/toggle/Toggle.test.tsx:49-53`).
- Transition: each click on the Toggle toggles the pressed state — `'false'` → `'true'` → `'false'` over two clicks in the uncontrolled test (`packages/react/src/toggle/Toggle.test.tsx:54-64`).
- Cancellation: calling `eventDetails.cancel()` inside `onPressedChange` aborts the state transition; `aria-pressed` stays `'false'` after the click (`packages/react/src/toggle/Toggle.test.tsx:88-100`).
- Grouped cancellation: when a grouped Toggle (`value="one"` inside `ToggleGroup`) cancels in `onPressedChange`, neither its own `aria-pressed` changes (`packages/react/src/toggle/Toggle.test.tsx:124`) nor does the group's `onValueChange` fire at all (`packages/react/src/toggle/Toggle.test.tsx:125`) — i.e. the cancel propagates through the Toggle into the group's value update.

## Keyboard interactions

N/A — no test in this suite dispatches keyboard events (no `fireEvent.key*`/`userEvent.keyboard` anywhere in `packages/react/src/toggle/Toggle.test.tsx`). Space/Enter activation is at most the native `<button>` default; UNVERIFIED — inferred from `packages/react/src/toggle/Toggle.test.tsx:11-16` (renders a native button), no test asserts keyboard behavior.

## Focus management

- No focus-management behavior is asserted: no test checks focus movement, focusing on mount, or focus trapping.
- The only focus-adjacent evidence is roving-tabindex-style composite state received in a group: with `ToggleGroup defaultValue={['left']}`, a Toggle's `render` prop receives `tabIndex: 0` (`packages/react/src/toggle/Toggle.test.tsx:162-168`).
- Any other focus behavior: UNVERIFIED — inferred from `packages/react/src/toggle/Toggle.test.tsx:11-16` (native button is natively focusable), no test asserts it.

## Accessibility (roles, aria-*, id linking)

- Toggle exposes the implicit `button` role (found via `screen.getByRole('button')`, `packages/react/src/toggle/Toggle.test.tsx:32`, and confirmed as a native `<button>` via `refInstanceof: window.HTMLButtonElement`, `packages/react/src/toggle/Toggle.test.tsx:12`).
- `aria-pressed` mirrors the pressed state as the string `'true'`/`'false'` in every state: controlled initial `'false'` (`packages/react/src/toggle/Toggle.test.tsx:34`), controlled `'true'` (`packages/react/src/toggle/Toggle.test.tsx:39`) and back to `'false'` (`packages/react/src/toggle/Toggle.test.tsx:45`), uncontrolled toggling (`packages/react/src/toggle/Toggle.test.tsx:53-64`), and after a canceled interaction (`packages/react/src/toggle/Toggle.test.tsx:100`).
- When `disabled`, the button carries the native `disabled` attribute and a `data-disabled` attribute, and reports `aria-pressed="false"` (`packages/react/src/toggle/Toggle.test.tsx:136-138`).
- No `id` linking, `aria-controls`, or other ARIA relationships are asserted in this suite (no such assertions exist in `packages/react/src/toggle/Toggle.test.tsx`).

## DOM structure & portal behavior

- The rendered output is a single native `<button>` element — no wrapper element — proven by the ref being an `HTMLButtonElement` (`packages/react/src/toggle/Toggle.test.tsx:12`) and by the `render`-prop conformance config targeting `button` (`packages/react/src/toggle/Toggle.test.tsx:13`).
- Disabled state is expressed on the same root element via `disabled` + `data-disabled` attributes (`packages/react/src/toggle/Toggle.test.tsx:136-137`).
- N/A — portal: no test renders Toggle into a portal and none of the suite's assertions involve portals (nothing in `packages/react/src/toggle/Toggle.test.tsx` references containers/portals).

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onPressedChange` fires once per activating click, with the *next* pressed value as the first argument: after one click from `defaultPressed={false}` it is called exactly once with `true` (`packages/react/src/toggle/Toggle.test.tsx:80-81`).
- The second callback argument is an event-details object exposing `cancel()` (`packages/react/src/toggle/Toggle.test.tsx:88-91`); invoking `cancel()` suppresses the internal pressed-state change (`packages/react/src/toggle/Toggle.test.tsx:100`) and, when the Toggle is grouped, also suppresses the group's `onValueChange` entirely (zero calls, `packages/react/src/toggle/Toggle.test.tsx:125`).
- N/A — no custom DOM events (e.g. bubbling synthetic events) are named or asserted in this suite; the only trigger exercised is a plain `click` (`packages/react/src/toggle/Toggle.test.tsx:55-56`).

## Edge cases (rapid interactions, unmount, nesting)

- Rapid/double interactions: UNVERIFIED — inferred from `packages/react/src/toggle/Toggle.test.tsx:54-64` (two sequential clicks toggle back and forth correctly), no test asserts rapid/interleaved interaction handling.
- Unmount mid-interaction: N/A — no unmount-related test exists in `packages/react/src/toggle/Toggle.test.tsx`.
- Nesting in a group is the one composition case covered: inside `ToggleGroup`, a canceled `onPressedChange` blocks both the item's state and the group's `onValueChange` (`packages/react/src/toggle/Toggle.test.tsx:103-126`), and a grouped Toggle receives composite group props (e.g. `tabIndex`) through `render` (`packages/react/src/toggle/Toggle.test.tsx:162-168`).
- Disabled interaction: a click on a disabled Toggle invokes `onPressedChange` zero times and leaves `aria-pressed` at `'false'` (`packages/react/src/toggle/Toggle.test.tsx:140-145`).

## Shared harness dependencies

- `#test-utils` resolves to `packages/react/test/index.ts`, which re-exports the harness pieces used here (`createRenderer`, `describeConformance`) (`packages/react/test/index.ts:3-4`).
- `packages/react/test/createRenderer.ts` provides the `render` function the suite passes into conformance options (`packages/react/src/toggle/Toggle.test.tsx:9`).
- `packages/react/test/describeConformance.tsx` runs the conformance sub-suite (propsSpread / refForwarding / renderProp / className) and accepts the `button?: boolean` option Toggle sets (`packages/react/test/describeConformance.tsx:36`, `packages/react/test/describeConformance.tsx:44-49`).
- `packages/react/test/conformanceTests/*.tsx` contain the actual conformance assertions; the `button: true` flag feeds a `nativeButton` expectation into the prop-forwarding and render-prop tests (`packages/react/test/conformanceTests/renderProp.tsx:14-25`, `packages/react/test/conformanceTests/propForwarding.tsx:14-20`).
- The suite also uses `act`/`screen` from `@mui/internal-test-utils` (`packages/react/src/toggle/Toggle.test.tsx:3`) — an external npm package, not a repo harness file.
