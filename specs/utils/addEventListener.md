# `addEventListener` — behavior spec

Unit: `packages/utils/src/addEventListener` (Phase A util → crate `leptos-ui-utils`).
Sources of truth: `packages/utils/src/addEventListener.test.ts` (runtime behavior) and
`packages/utils/src/addEventListener.spec.ts` (type-level behavior only — it is a compile-time
spec, never executed assertions).

## Public API surface (props, parts, subcomponents)

- Single named export `addEventListener`. No components, no props object, no parts.
- Signature as exercised: `addEventListener(target, type, listener, options?)`.
  - `target` is a generic `EventTarget`. Proven supported target kinds:
    - `window` (`packages/utils/src/addEventListener.spec.ts:4-7`)
    - a DOM element (`document.createElement('div')`, typed as element) (`packages/utils/src/addEventListener.spec.ts:9-13`)
    - a bare `Element` (non-element-specific DOM node) (`packages/utils/src/addEventListener.spec.ts:15-19`)
    - a `MediaQueryList` (`packages/utils/src/addEventListener.spec.ts:21-25`)
    - a generic `new EventTarget()` (`packages/utils/src/addEventListener.spec.ts:27-29`)
    - any object exposing `addEventListener`/`removeEventListener` methods (the runtime test uses
      such a mock) (`packages/utils/src/addEventListener.test.ts:5-9`)
  - `type` is the event name, discriminated per target kind at the type level (see Events below).
  - `listener` receives the event; its parameter type is enforced to match the event name
    (see Events below).
  - `options` may be omitted: every call in the type spec passes only 3 arguments and the file
    must typecheck (`packages/utils/src/addEventListener.spec.ts:4-29`).
- Return value: an unsubscribe function typed `() => void`
  (`packages/utils/src/addEventListener.spec.ts:7`, `packages/utils/src/addEventListener.spec.ts:13`,
  `packages/utils/src/addEventListener.spec.ts:25`, `packages/utils/src/addEventListener.spec.ts:29`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — stateless utility with no state, defaults, or transitions. No test asserts any state
behavior.

## Keyboard interactions

N/A — non-visual utility. `'keydown'` appears only as a typed example event name
(`packages/utils/src/addEventListener.spec.ts:10-19`); the utility itself implements no keyboard
behavior. UNVERIFIED — inferred from `packages/utils/src/addEventListener.spec.ts:10-19`, no test
asserts keyboard handling beyond type checks.

## Focus management

N/A — no focus behavior is implemented or asserted by any test.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test.

## DOM structure & portal behavior

N/A — the utility renders nothing and creates no DOM. It only subscribes listeners on targets
that already exist (window, element, MediaQueryList, plain EventTarget —
`packages/utils/src/addEventListener.spec.ts:4-29`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- Subscription: calling `addEventListener(target, type, listener, options)` forwards the exact
  `(type, listener, options)` triple to `target.addEventListener` — the mock target records
  `('click', listener, { capture: true, passive: false })` unchanged
  (`packages/utils/src/addEventListener.test.ts:11-15`).
- Unsubscription: the returned function forwards the *identical* `(type, listener, options)` triple
  to `target.removeEventListener` — same listener reference and same options object, so capture /
  passive matching works for removal (`packages/utils/src/addEventListener.test.ts:17-19`).
- Event-name typing is discriminated per target kind at compile time:
  - `window` + `'pointermove'` → listener param is `PointerEvent`
    (`packages/utils/src/addEventListener.spec.ts:4-6`)
  - element + `'keydown'` → listener param is `KeyboardEvent`
    (`packages/utils/src/addEventListener.spec.ts:10-12`, same for a generic `Element` target at
    `packages/utils/src/addEventListener.spec.ts:16-18`)
  - `MediaQueryList` + `'change'` → listener param is `MediaQueryListEvent`
    (`packages/utils/src/addEventListener.spec.ts:22-24`)
  - generic `EventTarget` + arbitrary name `'custom'` → accepted
    (`packages/utils/src/addEventListener.spec.ts:27-29`)
  - a mismatched handler type is a compile error: a `KeyboardEvent` handler on
    `window`/`'pointermove'` (`packages/utils/src/addEventListener.spec.ts:35-36`) and a
    `PointerEvent` handler on `'keydown'` (`packages/utils/src/addEventListener.spec.ts:38-41`).
- Runtime payload delivery: UNVERIFIED — inferred from `packages/utils/src/addEventListener.test.ts:10-15`,
  no test asserts this (the listener is a `vi.fn()` mock and no event is ever dispatched).
- Bubbling / `preventDefault` / capture-phase delivery semantics: N/A — no test dispatches events
  or asserts propagation behavior.

## Edge cases (rapid interactions, unmount, nesting)

- Cleanup/unmount: the returned unsubscribe function is the cleanup path; it removes the listener
  with the identical arguments it was registered with
  (`packages/utils/src/addEventListener.test.ts:17-19`).
- Repeated unsubscribe calls, subscribing the same listener twice, nested/overlapping subscriptions,
  and rapid subscribe/unsubscribe cycles: UNVERIFIED — inferred from
  `packages/utils/src/addEventListener.test.ts:5-20`, no test asserts this (the single runtime test
  subscribes and unsubscribes exactly once).

## Shared harness dependencies

- `packages/utils/src/testUtils.ts` — imported by the type spec
  (`packages/utils/src/addEventListener.spec.ts:1`). Provides `expectType<Expected, Actual>`, a
  no-op function that fails to compile unless the two types are exactly identical (via the
  `IfEquals` helper), and `isJSDOM` (`packages/utils/src/testUtils.ts:1-21`). No dependency on the
  `packages/react/test` `#test-utils` harness.
