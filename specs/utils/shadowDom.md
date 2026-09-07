# `shadowDom` — behavior spec

Unit: `packages/utils/src/shadowDom` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/shadowDom.test.ts` (32 lines — a single `describe('getTarget')`
block with 2 tests). This is a small shadow-DOM-safe DOM/event utility module; the suite only
exercises `getTarget`.

## Public API surface (props, parts, subcomponents)

- Three named exports: `activeElement`, `contains`, `getTarget`. No components, no props, no parts.
- Only `getTarget` is test-proven: signature as exercised is `getTarget(event)` taking an `Event`
  and returning an `EventTarget` — called synchronously inside a click listener
  (`packages/utils/src/shadowDom.test.ts:12-14`) and after dispatch has completed
  (`packages/utils/src/shadowDom.test.ts:25-29`).
- `activeElement` and `contains` are exports of the module but are not imported or exercised by any
  test in this unit's suite (the test file imports only `getTarget`,
  `packages/utils/src/shadowDom.test.ts:2`) — UNVERIFIED — inferred from
  `packages/utils/src/shadowDom.ts:3-11` and `packages/utils/src/shadowDom.ts:13-37`, no test
  asserts their behavior.

## State model (controlled/uncontrolled, defaults, transitions)

N/A — stateless pure utilities. `getTarget` is a pure function of the event argument: tests pass
events in and assert only the return value, with no state or defaults involved
(`packages/utils/src/shadowDom.test.ts:12-17`, `packages/utils/src/shadowDom.test.ts:25-29`).

## Keyboard interactions

N/A — non-visual utility. No keyboard behavior is implemented or asserted by any test.

## Focus management

N/A for tested behavior — no test asserts any focus behavior. The module also exports an
`activeElement` helper intended to resolve the focused element through nested shadow roots
(walking down `shadowRoot.activeElement` until a light-DOM active element is reached) — UNVERIFIED
— inferred from `packages/utils/src/shadowDom.ts:3-11`, no test asserts this.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test.

## DOM structure & portal behavior

N/A — the utility renders nothing, creates no DOM, and performs no portal work. The tests build
plain light-DOM fixtures (`div` parent with a `span` child appended to `document.body`,
`packages/utils/src/shadowDom.test.ts:6-9`; a standalone `div` appended to `document.body`,
`packages/utils/src/shadowDom.test.ts:22-23`) only as event-dispatch targets. No test creates a
shadow root, so all proven behavior is in the light DOM.

## Events (names, payload shape, bubbling, preventDefault semantics)

- During dispatch (from inside an event listener), `getTarget` returns the composed-path target —
  the element the event was originally dispatched on — even when the handler currently running is
  attached to an ancestor and the event is bubbling: the listener is registered on a parent `div`,
  a bubbling `click` is dispatched on the child `span`, and `getTarget(event)` inside that listener
  returns the child (`packages/utils/src/shadowDom.test.ts:5-19`).
- Once dispatch has completed, `event.composedPath()` is empty (asserted with `toHaveLength(0)`)
  and `getTarget` falls back to `event.target`: the same element dispatched upon is returned
  post-dispatch (`packages/utils/src/shadowDom.test.ts:21-31`).
- Proven event types: only a plain bubbling `click` `Event` on light-DOM elements. Shadow-root
  retargeting specifics and behavior for event objects lacking `composedPath` (older browsers
  without shadow DOM support, where it falls back to `event.target`): UNVERIFIED — inferred from
  `packages/utils/src/shadowDom.ts:39-48`, no test asserts this.
- Payload shape: N/A — the return value is the target node itself; no structured payload is
  produced (`packages/utils/src/shadowDom.test.ts:17`, `packages/utils/src/shadowDom.test.ts:29`).
- preventDefault semantics: N/A — no test dispatches a cancelable event or asserts default-action
  behavior.

## Edge cases (rapid interactions, unmount, nesting)

- Post-dispatch usage is the one proven edge case: `composedPath()` becomes empty once dispatch is
  over, and `getTarget` still returns the original target via the `event.target` fallback
  (`packages/utils/src/shadowDom.test.ts:21-31`).
- Rapid interactions, unmount/cleanup, and re-entrancy: N/A — the utility holds no state, so no
  lifecycle cleanup is exercised by any test.
- Nesting across shadow boundaries: the `contains` export is designed to traverse out of shadow
  roots (native `contains` first, then a host-chain walk when the child's root node is a
  `ShadowRoot`, with the shadow-root predicate delegated to `isShadowRoot` from
  `@floating-ui/utils/dom`) — UNVERIFIED — inferred from `packages/utils/src/shadowDom.ts:1-37`,
  no test in this suite asserts cross-shadow containment.

## Shared harness dependencies

None — the test file's only imports are `vitest` and the unit under test itself
(`packages/utils/src/shadowDom.test.ts:1-2`). No `#test-utils` harness and no file under
`packages/react/test/` is involved.
