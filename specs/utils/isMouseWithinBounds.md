# `isMouseWithinBounds` — behavior spec

Unit: `packages/utils/src/isMouseWithinBounds` (Phase A util → crate `leptos-ui-utils`, per
`TODO.md:84-88`; the TODO entry has no `wraps-external:` field and no `needs-batched-mining` flag).

Unit registration and test-file reality: the registry lists exactly one src file,
`packages/utils/src/isMouseWithinBounds.ts`, and an empty `testFiles` array
(`ralph/generated/utils.json:141-148`). That registered file is a deprecated single-argument
variant with no dedicated test file. The only test file in the repo exercising an
`isMouseWithinBounds` implementation is `packages/react/src/utils/getPseudoElementBounds.test.ts`,
which targets the live two-arg implementation of the same name in
`packages/react/src/utils/getPseudoElementBounds.ts` (imported at
`packages/react/src/utils/getPseudoElementBounds.test.ts:3`). This spec therefore grounds all
proven claims in that test file, and marks every claim about the deprecated registered variant as
UNVERIFIED against its source.

## Public API surface (props, parts, subcomponents)

Two same-named function variants exist; no components, parts, or subcomponents in either.

Live, tested variant — `packages/react/src/utils/getPseudoElementBounds.ts`:

- Named export `isMouseWithinBounds(event, element)`: two positional arguments — a `MouseEvent`
  and an `HTMLElement`. The argument roles are pinned by the test helper, which constructs
  `new MouseEvent('mouseup', { clientX, clientY })` and passes the element separately, so the
  element is explicit and not derived from the event
  (`packages/react/src/utils/getPseudoElementBounds.test.ts:64-65`).
- Return value: boolean, asserted with strict `toBe(true)`/`toBe(false)`
  (`packages/react/src/utils/getPseudoElementBounds.test.ts:68-78`).
- Companion named export `getPseudoElementBounds(element)` returning an `{ left, right, top,
  bottom }` bounds object, asserted with `toEqual`/`toMatchObject`
  (`packages/react/src/utils/getPseudoElementBounds.test.ts:14-19`,
  `packages/react/src/utils/getPseudoElementBounds.test.ts:49-54`).

Deprecated registered variant — `packages/utils/src/isMouseWithinBounds.ts`:

- Named export `isMouseWithinBounds(event)` taking a single `React.MouseEvent<HTMLElement> |
  React.PointerEvent<HTMLElement>` argument and reading `event.currentTarget`
  (`packages/utils/src/isMouseWithinBounds.ts:4-7`). UNVERIFIED — inferred from
  `packages/utils/src/isMouseWithinBounds.ts:4-7`, no test asserts this.
- Marked `@deprecated` ("no longer used internally and will be removed in the next version") per
  its JSDoc (`packages/utils/src/isMouseWithinBounds.ts:1-3`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — stateless pure predicates. Every call recomputes from the event coordinates and the
element's current rect; the same element yields different answers as only the event coordinates
vary across assertions (`packages/react/src/utils/getPseudoElementBounds.test.ts:68-78`), and no
module-level state exists in the implementation
(`packages/react/src/utils/getPseudoElementBounds.ts:20-29`). No defaults, no controlled/
uncontrolled duality, no transitions.

## Keyboard interactions

N/A — no keyboard behavior is implemented or asserted by any test. The only event constructor the
tests use is a mouse event (`packages/react/src/utils/getPseudoElementBounds.test.ts:65`), and the
implementation reads no keyboard-related state
(`packages/react/src/utils/getPseudoElementBounds.ts:20-29`).

## Focus management

N/A — no focus behavior is implemented
(`packages/react/src/utils/getPseudoElementBounds.ts:20-29`) or asserted by any test
(`packages/react/src/utils/getPseudoElementBounds.test.ts:1-86`).

## Accessibility (roles, aria-*, id linking)

N/A — the function renders nothing and touches no attributes; it is a boolean predicate over
coordinates and a rect (`packages/react/src/utils/getPseudoElementBounds.ts:20-29`), and no test
asserts any role, aria, or id-linking behavior
(`packages/react/src/utils/getPseudoElementBounds.test.ts:1-86`).

## DOM structure & portal behavior

N/A for DOM creation — the function creates no DOM and no portals. It only *reads* DOM geometry:

- Reads `element.getBoundingClientRect()` (stubbed per-element in tests via
  `DOMRect.fromRect(rect)`; `packages/react/src/utils/getPseudoElementBounds.test.ts:82-86`).
- In jsdom, pseudo-element styles are never read: `window.getComputedStyle` is spied and asserted
  `not.toHaveBeenCalled()`, and the bounds equal the element's own rect (element at (100,50),
  20×10 → `{left:100, right:120, top:50, bottom:60}`); this test is `it.skipIf(!isJSDOM)`, so it
  only runs in jsdom (`packages/react/src/utils/getPseudoElementBounds.test.ts:10-21`).
- In real browsers, bounds expand symmetrically to include pseudo-elements: an element 20×10 at
  (100,50) with a 40×30 `::before` yields `{left:90, right:130, top:40, bottom:70}` (±10px on both
  axes); this test is `it.skipIf(isJSDOM)`, so it only runs in a browser
  (`packages/react/src/utils/getPseudoElementBounds.test.ts:23-59`).
- Cross-realm/iframe resolution via `ownerWindow(element)`
  (`packages/react/src/utils/getPseudoElementBounds.ts:33`): UNVERIFIED — inferred from
  `packages/react/src/utils/getPseudoElementBounds.ts:33`, no test asserts owner-window behavior.

## Events (names, payload shape, bubbling, preventDefault semantics)

- The function registers no event listeners; it is a pure predicate evaluated against an
  already-constructed event object. UNVERIFIED as a strict absence claim — inferred from
  `packages/react/src/utils/getPseudoElementBounds.ts:20-29`, no test asserts listener
  (non-)registration.
- Event type exercised: a plain (untrusted) `MouseEvent('mouseup')` carrying only `clientX`/
  `clientY`, with no target or `currentTarget` set
  (`packages/react/src/utils/getPseudoElementBounds.test.ts:64-65`).
- The result is driven solely by the event's `clientX`/`clientY` against the passed element's
  bounds: coordinates are the only varying input across the edge matrix, and flipping each
  coordinate by 1px flips the verdict (`packages/react/src/utils/getPseudoElementBounds.test.ts:68-78`).
- Bubbling and preventDefault semantics: N/A — nothing is asserted; the implementation never calls
  `preventDefault` or `stopPropagation`
  (`packages/react/src/utils/getPseudoElementBounds.ts:20-29`), but no test asserts this
  mutation-freedom.

## Edge cases (rapid interactions, unmount, nesting)

Proven behaviors:

- Inclusive ±5px tolerance on every edge (`BOUNDARY_OFFSET`); all four edges are pinned. For an
  element with left=100, right=120, top=50, bottom=60: left edge — x=95 in, x=94 out; right edge —
  x=125 in, x=126 out; top edge — y=45 in, y=44 out; bottom edge — y=65 in, y=66 out
  (`packages/react/src/utils/getPseudoElementBounds.test.ts:61-78`). The tolerance exists so a
  fast click with slight pointer drift is not mistaken for a drag-off-and-release cancellation
  (`packages/react/src/utils/getPseudoElementBounds.ts:11-14`).
- The drift/tolerance test is not environment-gated (no `skipIf`), unlike the jsdom-only and
  browser-only bounds tests (`packages/react/src/utils/getPseudoElementBounds.test.ts:61` vs
  `packages/react/src/utils/getPseudoElementBounds.test.ts:10-23`).
- The element does not need to be attached to the document: every test uses a detached `div` whose
  `getBoundingClientRect` is stubbed (`packages/react/src/utils/getPseudoElementBounds.test.ts:82-86`,
  used at `packages/react/src/utils/getPseudoElementBounds.test.ts:11` and
  `packages/react/src/utils/getPseudoElementBounds.test.ts:62`).

Unproven behaviors (must not be relied on as specified):

- `::after` pseudo-element expansion: only `::before` is exercised in the browser test
  (`packages/react/src/utils/getPseudoElementBounds.test.ts:34-39`). UNVERIFIED — inferred from
  `packages/react/src/utils/getPseudoElementBounds.ts:41` and
  `packages/react/src/utils/getPseudoElementBounds.ts:52-53`, no test asserts `::after` handling.
- The no-pseudo-element browser path (`content: 'none'` short-circuit returning the plain rect):
  UNVERIFIED — inferred from `packages/react/src/utils/getPseudoElementBounds.ts:43-47`; only the
  jsdom bypass test proves rect passthrough, and it proves it via the environment check, not the
  content check.
- Non-numeric pseudo-element `width`/`height` falling back to 0 via `parseFloat(...) || 0`:
  UNVERIFIED — inferred from `packages/react/src/utils/getPseudoElementBounds.ts:50-53`, no test
  asserts this.
- Zero-size or negative-size rects, and pseudo-elements smaller than the host element (expansion
  diff of 0): UNVERIFIED — inferred from `packages/react/src/utils/getPseudoElementBounds.ts:56-67`,
  no test asserts this.
- Rapid repeated calls, nesting, or unmount interaction: N/A — pure function with no lifecycle; no
  test covers call-frequency or teardown (`packages/react/src/utils/getPseudoElementBounds.test.ts:1-86`).

Deprecated registered variant (`packages/utils/src/isMouseWithinBounds.ts`) — all UNVERIFIED, as
the registry lists no test files for it (`ralph/generated/utils.json:144`):

- 1px inset on each side (`top + 1 <= clientY <= bottom - 1`, same for x): UNVERIFIED — inferred
  from `packages/utils/src/isMouseWithinBounds.ts:13-17`, no test asserts this.
- Depends on `event.currentTarget` being the element of interest: UNVERIFIED — inferred from
  `packages/utils/src/isMouseWithinBounds.ts:7`, no test asserts this.
- Exists as a workaround for Safari misfiring `mouseleave` on trigger-aligned items (issue #869):
  UNVERIFIED — inferred from `packages/utils/src/isMouseWithinBounds.ts:9-12`, no test asserts
  this.

## Shared harness dependencies

- The unit's only test file imports `isJSDOM` from `#test-utils`
  (`packages/react/src/utils/getPseudoElementBounds.test.ts:2`), which maps to
  `packages/react/test/index.ts` (`packages/react/package.json:110`).
- `isJSDOM` reaches the test through the barrel's `export * from '@base-ui/utils/testUtils'`
  (`packages/react/test/index.ts:1`) and is defined as a jsdom user-agent check
  (`packages/utils/src/testUtils.ts:4`).
- The harness is used solely for environment gating: `it.skipIf(!isJSDOM)` for the jsdom-only test
  and `it.skipIf(isJSDOM)` for the browser-only test
  (`packages/react/src/utils/getPseudoElementBounds.test.ts:10-23`).
- No other harness APIs (`createRenderer`, `firePointer`, `wait`, etc.) are used by this test file
  (`packages/react/src/utils/getPseudoElementBounds.test.ts:1-3`). The `createElementWithRect`
  helper is local to the test file, not shared harness code
  (`packages/react/src/utils/getPseudoElementBounds.test.ts:82-86`).
