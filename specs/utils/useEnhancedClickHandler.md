# `useEnhancedClickHandler` — behavior spec

Unit: `packages/utils/src/useEnhancedClickHandler` (Phase A util → crate `leptos-ui-utils`).

**Source of truth: none.** This unit has no test files. `ralph/generated/utils.json` records
`testFiles: []` for it (`ralph/generated/utils.json:279-283`), and no
`useEnhancedClickHandler.test.*` or `.spec.*` file exists anywhere in the repo. Per the mining
rules, every non-trivial claim below is therefore UNVERIFIED — inferred from the unit's own source,
no test asserts this. Downstream stages must treat this spec as a source-derived behavioral sketch,
not test-proven behavior; any indirect coverage through consumer component suites (drawer, select,
combobox, menu, popover, dialog) is out of scope for this unit's spec and was not read.

`TODO.md` has no `wraps-external:` field for this unit (`TODO.md:184-188`) — it is original Base UI
code, not a wrapper around a third-party npm package, so nothing is delegated to an external
package. It also has no `needs-batched-mining:` field, and the unit is a single 50-line source file
with zero test lines, so batched mining does not apply.

## Public API surface (props, parts, subcomponents)

- Named type export `InteractionType = 'mouse' | 'touch' | 'pen' | 'keyboard' | ''`. The empty
  string is a meaningful sentinel ("unknown / no pointerdown recorded yet"), not a filler member.
  UNVERIFIED — inferred from `packages/utils/src/useEnhancedClickHandler.ts:4`, no test asserts this.
- Single named hook export `useEnhancedClickHandler(handler)` taking one argument: a callback of
  type `(event: React.MouseEvent | React.PointerEvent, interactionType: InteractionType) => void`.
  UNVERIFIED — inferred from `packages/utils/src/useEnhancedClickHandler.ts:13-15`, no test asserts this.
- Returns exactly two handler props, `{ onClick, onPointerDown }`, shaped for direct spread onto a
  trigger element. UNVERIFIED — inferred from `packages/utils/src/useEnhancedClickHandler.ts:49`,
  no test asserts this.
- No components, no parts, no subcomponents, no props object — this is a headless behavior hook.
  UNVERIFIED — inferred from `packages/utils/src/useEnhancedClickHandler.ts:1-50`, no test asserts this.
- Documented intent: a cross-browser way to determine the pointer type used for a click, because
  Safari and Firefox deliver a `MouseEvent` (no `pointerType`) to click handlers instead of a
  `PointerEvent`, plus detection of keyboard-triggered clicks. UNVERIFIED — inferred from
  `packages/utils/src/useEnhancedClickHandler.ts:6-9`, no test asserts this.

## State model (controlled/uncontrolled, defaults, transitions)

N/A for controlled/uncontrolled — this is not a form/state primitive. The hook's only state is one
`useRef<InteractionType>('')` per hook instance (`lastClickInteractionTypeRef`), initialized to
`''`. UNVERIFIED — inferred from `packages/utils/src/useEnhancedClickHandler.ts:16`, no test
asserts this.

Inferred transition machine for that ref, all UNVERIFIED — inferred from
`packages/utils/src/useEnhancedClickHandler.ts:16-44`, no test asserts this:

- `''` → `event.pointerType` on a non-default-prevented `pointerdown` (recorded before the handler
  is invoked, `packages/utils/src/useEnhancedClickHandler.ts:24`).
- → `''` after any click that reaches the pointer branches
  (`packages/utils/src/useEnhancedClickHandler.ts:38-44`), i.e. the recorded value is single-shot:
  it is consumed and cleared by the next click.
- unchanged on a keyboard click: the `event.detail === 0` path early-returns before the reset at
  `packages/utils/src/useEnhancedClickHandler.ts:44`
  (`packages/utils/src/useEnhancedClickHandler.ts:33-36`).
- unchanged on a default-prevented `pointerdown`: the guard at
  `packages/utils/src/useEnhancedClickHandler.ts:20-22` skips both the record and the handler call.

## Keyboard interactions

The hook registers no keyboard listeners; keyboard activation is detected structurally at click
time via the DOM sentinel `event.detail === 0` (click count 0 means the click was triggered by the
keyboard), and the handler is then invoked with the literal `'keyboard'` interaction type.
UNVERIFIED — inferred from `packages/utils/src/useEnhancedClickHandler.ts:32-36`, no test asserts this.

## Focus management

N/A — the unit's source implements no focus behavior, and the unit has no test files to assert any.
UNVERIFIED — inferred from `packages/utils/src/useEnhancedClickHandler.ts:1-50`, no test asserts this.

## Accessibility (roles, aria-*, id linking)

N/A — the unit's source sets no roles, aria attributes, or id links; it only returns event-handler
props for the caller to attach. The unit has no test files to assert any.
UNVERIFIED — inferred from `packages/utils/src/useEnhancedClickHandler.ts:1-50`, no test asserts this.

## DOM structure & portal behavior

N/A — the hook renders nothing, creates no DOM, and performs no portal behavior; its only DOM
coupling is the event-handler props it hands back.
UNVERIFIED — inferred from `packages/utils/src/useEnhancedClickHandler.ts:1-50`, no test asserts this.

## Events (names, payload shape, bubbling, preventDefault semantics)

All UNVERIFIED — the unit has no test files; each claim is inferred from the cited source lines.

- Returned prop names are `onPointerDown` (bound to the internal `handlePointerDown`) and `onClick`
  (bound to the internal `handleClick`)
  (`packages/utils/src/useEnhancedClickHandler.ts:18-30`, `packages/utils/src/useEnhancedClickHandler.ts:49`).
- Dual invocation per pointer click: the handler is called once on `pointerdown` with the live
  `event.pointerType` (`packages/utils/src/useEnhancedClickHandler.ts:25`) and again on `click`
  (`packages/utils/src/useEnhancedClickHandler.ts:33-43`). A keyboard click invokes the handler only
  once (via click; `pointerdown` from keyboards does not carry `pointerType` input through this
  path) (`packages/utils/src/useEnhancedClickHandler.ts:33-36`).
- Payload shape: the original event object is passed through unmodified as the first argument, with
  the resolved `InteractionType` string as the second
  (`packages/utils/src/useEnhancedClickHandler.ts:25`, `packages/utils/src/useEnhancedClickHandler.ts:34`,
  `packages/utils/src/useEnhancedClickHandler.ts:40`, `packages/utils/src/useEnhancedClickHandler.ts:42`).
- Interaction-type resolution on click: `event.detail === 0` → `'keyboard'`; if the event carries
  `pointerType` (PointerEvent — Chrome/Edge) → live `event.pointerType`; otherwise (MouseEvent —
  Safari/Firefox) → the pointer type recorded at the last non-default-prevented `pointerdown`, which
  is `''` if none was recorded
  (`packages/utils/src/useEnhancedClickHandler.ts:33-43`).
- preventDefault semantics are asymmetric: the `pointerdown` handler no-ops entirely when
  `event.defaultPrevented` is true (no recording, no handler call,
  `packages/utils/src/useEnhancedClickHandler.ts:20-22`), while the `click` handler has no
  `defaultPrevented` guard and always invokes the handler
  (`packages/utils/src/useEnhancedClickHandler.ts:30-47`).
- Bubbling/propagation: the hook never calls `stopPropagation`/`preventDefault`, so bubbled events
  from descendants still reach and trigger the handlers.
  UNVERIFIED — inferred from `packages/utils/src/useEnhancedClickHandler.ts:18-47`, no test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

All UNVERIFIED — the unit has no test files; each claim is inferred from the cited source lines.

- Rapid interactions: each non-prevented `pointerdown` overwrites the recorded type
  (`packages/utils/src/useEnhancedClickHandler.ts:24`), and each completed click resets it to `''`
  (`packages/utils/src/useEnhancedClickHandler.ts:44`), so a click always consumes the most recent
  pointerdown; if two pointerdowns fire before one click, the last one wins.
- Click without a preceding pointerdown (e.g. a programmatic click delivering a `MouseEvent` with
  `detail > 0`): the handler receives `''` as the interaction type, since the ref's initial and
  reset value is `''`
  (`packages/utils/src/useEnhancedClickHandler.ts:16`, `packages/utils/src/useEnhancedClickHandler.ts:42`).
- A keyboard click does not clear a previously recorded pointer type (early return at
  `packages/utils/src/useEnhancedClickHandler.ts:35` precedes the reset at
  `packages/utils/src/useEnhancedClickHandler.ts:44`), so a stale recorded value can survive a
  keyboard click.
- Unmount: the hook registers no effects and has no cleanup; the ref is garbage-collected with the
  component instance. UNVERIFIED — inferred from `packages/utils/src/useEnhancedClickHandler.ts:1-50`,
  no test asserts this.
- Nesting / multiple instances: the ref is per-hook-instance (`React.useRef`), not module-level, so
  separate triggers keep independent interaction state
  (`packages/utils/src/useEnhancedClickHandler.ts:16`).
- Handler freshness: the returned handlers are `React.useCallback`-memoized on `[handler]`, so their
  identities change whenever the caller's handler changes; there is no ref-based staleness shim
  (`packages/utils/src/useEnhancedClickHandler.ts:27`, `packages/utils/src/useEnhancedClickHandler.ts:46`).

## Shared harness dependencies

None. The unit has no test files (`ralph/generated/utils.json:281`), so there is no test-utils
harness to import and none was read; nothing beyond the unit's own source, its manifest entry, and
its `TODO.md` entry was consulted.
