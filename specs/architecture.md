# Architecture

Cross-cutting React → Leptos design decisions, resolved via interview (2026-09-07) before any
Stage 3 forward-loop work begins, per the build-order note in `TODO.md`. Every decision below
either cites verified evidence (a docs.rs page, an empirical browser test, real source code read
directly) or is explicitly marked provisional/unresolved — nothing here is guessed.

This file is deliberately not exhaustive. Topics not yet covered by an interview round are absent,
not implicitly resolved by omission — see "Not yet covered" at the end.

## State: controlled/uncontrolled (`useControlled`)

A dedicated `leptos-ui-utils::use_controlled` utility, ported from `useControlled`'s actual
semantics rather than reinvented: standard controlled/uncontrolled duality — an internal signal
for uncontrolled mode, external signal reads take over when the caller passes a controlled value.
Every leptos-ui component with a controllable value (open state, selected value, etc.) uses this
one utility rather than each hand-rolling its own controlled/uncontrolled branch.

## Context passing

Direct 1:1 mapping to React Context: Leptos's `provide_context` / `use_context`, scoped to the
component subtree the same way `ToastProviderContext`/`TooltipRootContext`/etc. work today.

## Refs / DOM access

Every leptos-ui component that forwards a ref in React accepts a `node_ref: NodeRef<T>` prop,
mirroring React's `forwardRef` shape directly — not internal-only, callers can bind it.

## Layout effect (`useIsoLayoutEffect`)

**Resolved via empirical test**, not docs alone — Leptos's own documentation doesn't specify this
precisely enough to trust.

- Leptos's plain `Effect` (0.7+) runs batched on the next microtask — does **not** match
  `useIsoLayoutEffect`.
- `RenderEffect`'s *first* run is documented as synchronous ("during rendering, not on the next
  tick"), but subsequent re-runs are undocumented.
- Built a minimal Leptos CSR app (`RenderEffect` + a click handler that logs before/after a signal
  `set` call, plus a separately-queued microtask marker), served it, and drove a real Chrome tab
  via `claude-in-chrome` to capture actual console ordering:
  ```
  [click] before set_count.set(1)
  [click] after set_count.set(1)
  [RenderEffect] run #1 saw count=1     ← the re-run
  [microtask marker] fired
  ```
- **Finding**: `RenderEffect`'s re-run is NOT synchronous within the same call stack as the signal
  update (it ran after "after set_count.set(1)" logged) — this differs architecturally from
  React's `useLayoutEffect`, which is same-call-stack synchronous. But it resolves as an
  earlier-queued microtask, before paint, so it should still avoid visible flicker for
  measurement/positioning use cases in practice.

**Decision**: use `RenderEffect` as the `useIsoLayoutEffect` equivalent. Record this
microtask-vs-synchronous distinction explicitly — if a specific Base UI behavior ever turns out to
depend on true same-call-stack synchronicity (not just pre-paint timing), this decision needs
revisiting for that case specifically.

## Portals

Leptos ships a real, built-in `leptos::portal::Portal` component ("renders components somewhere
else in the DOM") — confirmed via docs.rs, not assumed. Use it directly; no custom portal
mechanism needed.

## Positioning (floating-ui)

**Provisional** — adopted pending a follow-up audit, not fully verified.

`floating-ui-leptos` (crates.io, `RustForWeb/floating-ui` org): real, actively maintained Rust
port of floating-ui — v0.6.0, published 2025-11-04, ~44.8k downloads, targets Leptos/Dioxus/Yew.
Exact middleware coverage (`flip`, `shift`, `offset`, `arrow`, `size`, `autoUpdate`, `hide`) is
**not yet confirmed** against what `packages/react/src/internals/useAnchorPositioning.ts` actually
uses — do this audit when the first positioning-dependent unit reaches Stage 3, not before.

`RustForWeb` has not ported Radix UI or Base UI itself — no shortcut exists for component logic,
only for the positioning primitive.

## Render composition (`useRender` / `asChild`)

The hardest translation problem in this port — no mechanical 1:1 exists, and initial research into
existing "prior art" (`radix-leptos`, `leptix-ui`) found one dead end and one real answer:

- `cloud-shuttle/radix-leptos`'s `Slot` primitive (the file most directly named for this): doc
  comment example doesn't compile (mismatched braces, wrong param name), references an undefined
  type (`SlotProps`), and its tests use the pre-0.5 Leptos `Scope`-as-argument API — stale/broken,
  not usable as a reference.
- `leptix-ui`'s `Primitive` component (`crates/core/src/primitive.rs`): real, current Leptos API
  (`tachys`, `ElementType`, `TypedChildrenFn`, `add_any_attr`). Small project (3 stars, pushed
  April 2026) but the code itself is legitimate and working.

**Decision**: build leptos-ui's own render-composition primitive using leptix-ui's design as the
model (not a dependency on the crate itself):
- `element: fn() -> HtmlElement<E, ...>` — constructor for the component's default element
- `as_child: MaybeProp<bool>` — the switch
- When `as_child` is true: render `children()` directly; when false: render
  `element().child(children())`
- Both branches merge the same `node_ref`/attrs via `add_any_attr`
- Event handler composition via a `compose_callbacks`-style utility (chains handlers, respects
  `defaultPrevented` — matches Base UI's own `composeEventHandlers` convention exactly)

## State → `data-*` attributes (`getStateAttributesProps`)

A separate `use_state_attributes` utility, composed alongside the render-composition primitive
above rather than built into it — mirrors React's own separation of `useRender`'s `state` param
from its render logic. Supports per-key override mappings the same way
`stateAttributesMapping` does.

## Prop / class / style merging (`mergeProps`)

No custom merge utility. Leptos's native `class:`/`style:`/attribute-binding system composes
class names and styles without needing a `mergeProps`-equivalent — rely on it directly rather than
building one preemptively. (Event-handler composition specifically is still its own thing — see
`compose_callbacks` under Render composition above; this decision covers class/style/generic
attrs only.)

## Shared store (`ReactStore`/`Store`, e.g. toast's manager)

Port `ReactStore`'s actual API shape as its own `leptos-ui-utils::Store` type — not just a bare
`RwSignal` in a struct. Keeps the same subscribe/notify abstraction shape as
`packages/utils/src/store/` for state that outlives a single component's render (toast's list,
etc.), for API-shape consistency across the port rather than ad hoc per-component state
containers.

## Timers (`useTimeout`)

A `leptos-ui-utils::use_timeout` utility wrapping Leptos's `set_timeout_with_handle`, registering
`on_cleanup` for automatic clearing — ports `useTimeout`'s own start/clear API shape.

## Composite keyboard navigation (`internals/composite/composite.ts`)

One shared `leptos-ui-internals` composite-navigation utility, matching how `internals/` is
already treated as shared Phase A infrastructure in `TODO.md` — not reimplemented per-component
across menu/select/tabs/radio-group/toolbar/etc.

## CSS custom properties (`ToastPositionerCssVars`, etc.)

**Not** `stylance` — that solves a different problem (compile-time CSS Modules-style class-name
scoping). These files expose *runtime-computed* values (measured offsets, heights) as `--`-prefixed
custom properties for consumer styling, which is a reactive-inline-style concern, not a
class-scoping one.

Confirmed via docs.rs: Leptos's `style=` binding supports a tuple form for arbitrary property
names including custom properties:
```rust
style=("--columns", move || count.get().to_string())
```
Use this directly — no extra abstraction needed.

## Form / validity integration (`Field`, native validation)

Base UI's Field/Form never use `ElementInternals` (confirmed: zero mentions across
`specs/library/field/*`, `specs/library/form/*`) — `Field.Control` renders a real native
`<input>`/`<textarea>`/`<select>`, and native form participation and validity just work because
these are genuine form elements, not custom/shadow-DOM ones needing form-association tricks.
`web_sys::ElementInternals` doesn't even exist as a binding (confirmed: 404 on its docs.rs page) —
moot, since it was never needed.

What Base UI actually calls — `element.setCustomValidity()`, reading `validity`/`validationMessage`
— maps 1:1 to real `web-sys` methods, confirmed present on all three relevant element types:

| Method | `HtmlInputElement` | `HtmlTextAreaElement` | `HtmlSelectElement` |
|---|---|---|---|
| `set_custom_validity(&str)` | ✓ | ✓ | ✓ |
| `validity() -> ValidityState` | ✓ | ✓ | ✓ |
| `check_validity() -> bool` | ✓ | ✓ | ✓ |
| `report_validity() -> bool` | ✓ | ✓ | ✓ |
| `validation_message()` | `-> String` | `-> Result<String, JsValue>` | `-> Result<String, JsValue>` |

Note the return-type inconsistency on `validation_message()` — a real `web-sys` API wrinkle to
handle per-element-type, not a port design decision.

`ValidityState` itself exposes every flag Base UI reads: `value_missing`, `type_mismatch`,
`pattern_mismatch`, `too_long`, `too_short`, `range_underflow`, `range_overflow`, `step_mismatch`,
`bad_input`, `custom_error`, `valid`.

`getDefaultFormSubmitter` is a plain DOM query (find the default submit button/input within a
form) — direct web-sys port, no special primitive.

## React ↔ Leptos state-management quick reference

Consolidated table of every state-management primitive mapping resolved in this file, plus a few
general React↔Leptos primitive equivalences confirmed via research (docs.rs/Leptos book) rather
than assumed, for quick lookup while porting a component. Entries already covered in detail above
just link back to their section; new entries have their own evidence noted inline.

| React | Leptos | Status | Notes |
|---|---|---|---|
| `useState` | `signal(initial) -> (ReadSignal<T>, WriteSignal<T>)` | Confirmed (Leptos book) | Direct equivalent |
| `useControlled` (Base UI's own) | `use_controlled` (custom `leptos-ui-utils` port) | **Resolved** — see [State: controlled/uncontrolled](#state-controlleduncontrolled-usecontrolled) | Not core Leptos — this is a Base UI-specific pattern we're porting, not a framework primitive |
| `useMemo` | Plain derived signal (a closure reading other signals) **or** `Memo::new` | Confirmed (Leptos book) | Leptos splits what `useMemo` covers into two: a derived signal by default (no caching, cheap), promoted to `Memo` only when the computation is genuinely expensive — this is a real branch to make per-use-site, not a 1:1 swap |
| `useCallback` | Usually nothing needed; `Callback<T>` type when a callback is passed as a component prop | Confirmed (Leptos book/docs) | `useCallback`'s core purpose (avoid recreating a function to prevent child re-renders) mostly doesn't apply — Leptos components run once, not on every reactive update, so identity-stability isn't the same concern. `Callback` exists for the prop-typing case (generic props can't express "optional function"), not for memoization |
| `useReducer` | No core-Leptos equivalent | **Unresolved / no direct primitive** | No built-in reducer primitive. Third-party `leptos-state` (XState/Zustand-inspired `MachineBuilder`/`use_machine`) exists but is an external dependency, not evaluated. Base UI's own internals mostly use plain signals + match-based transitions already (see toast's `Store`, below) rather than a reducer pattern, so this gap may not matter in practice — flagged, not decided |
| `useRef` (DOM) | `NodeRef<T>` | **Resolved** — see [Refs / DOM access](#refs--dom-access) | |
| `useRef` (mutable box, non-DOM) | `StoredValue<T>` / `Rc<Cell<T>>` | Confirmed (used directly in this session's own `RenderEffect` test) | For a plain mutable cell with no reactivity (e.g. the `run_number` counter in the empirical test), not `NodeRef` |
| `useContext` / Context.Provider | `use_context` / `provide_context` | **Resolved** — see [Context passing](#context-passing) | |
| `useEffect` | Leptos `Effect` (batched, microtask-timed) | Noted, not separately resolved | Only investigated `RenderEffect` (below) in depth since that's what `useIsoLayoutEffect` needed; plain `Effect` presumably suffices for ordinary `useEffect` post-paint side effects, unverified |
| `useLayoutEffect` / `useIsoLayoutEffect` | `RenderEffect` (first run sync, re-runs microtask-timed) | **Resolved, empirically verified** — see [Layout effect](#layout-effect-useisolayouteffect) | Real browser test, not docs-only |
| A shared subscribe/notify store outside component scope (Base UI's own `ReactStore`) | Ported `Store` type in `leptos-ui-utils` | **Resolved** — see [Shared store](#shared-store-reactstorestore-eg-toasts-manager) | |
| `setTimeout` wrapper (`useTimeout`) | `use_timeout` (`set_timeout_with_handle` + `on_cleanup`) | **Resolved** — see [Timers](#timers-usetimeout) | |
| `asChild` / `useRender`'s render-prop composition | Custom `Primitive` component (`as_child` + `add_any_attr`) | **Resolved** — see [Render composition](#render-composition-userender--aschild) | The one genuinely hard translation in this table |
| `getStateAttributesProps` (state → `data-*`) | `use_state_attributes` | **Resolved** — see [State → data-* attributes](#state--data--attributes-getstateattributesprops) | |

## ID generation (`useId`)

No Leptos framework primitive for this (confirmed via research — nothing found). Worth noting:
Leptos 0.7 removed ID-based hydration entirely (it walks the DOM directly instead of matching by
unique IDs), so there's less framework-level pressure toward providing one than in React, where
`useId` exists partly for the framework's own hydration-matching needs.

Base UI's own `useId` is simpler than React's built-in one regardless — just
`(external_id?: String, prefix?: String) -> String` for `aria-describedby`-style linking, nothing
hydration-specific.

**Decision**: a custom `leptos-ui-utils::use_id`, direct port of that same shape. The counter must
produce the same sequence on server and client for a given render — a plain implementation detail
(deterministic counter, scoped correctly), not a Leptos-hydration concern, since 0.7 doesn't
match elements by ID anyway.

## Animation/transition finish detection (`useAnimationsFinished`)

Confirmed via `web-sys` docs: `AnimationEvent` and `TransitionEvent` both exist with everything
needed —
`animation_name()`/`property_name()`, `elapsed_time()`, `pseudo_element()` — a direct 1:1 mapping
to the DOM Animation/Transition Events API Base UI's own hook wraps.

**Decision**: direct `web-sys` port (`animationend`/`transitionend` listeners), wrapped in a
`leptos-ui-internals` utility matching `useAnimationsFinished`'s own API shape
(`ref, disableCancelCheck?, batch?`).

## Log-once dedup (`createLogOnce`, used by `warn`/`error`)

`createLogOnce(severity, prefix?)` is a factory returning a logger that suppresses repeat
`(severity, message)` pairs until `reset()` clears the dedup registry — confirmed via the mined
spec, not assumed.

**Decision**: a `thread_local` `HashSet<(Severity, String)>` gating `web_sys::console::warn_1`/
`error_1` calls, with a `reset()` that clears the set — direct port of the React version's
mechanics. (Whether this should be dev-only/stripped from release builds, the way Base UI likely
gates on `NODE_ENV`, was raised but not resolved — the Rust/WASM equivalent gating, `cfg!(debug_assertions)`
vs. a dedicated cargo feature, needs its own decision later.)

## Crate workspace layout

**Fixed a real problem, not just confirmed a name.** `TODO.md`'s auto-generated `crate:` fields
had never been reviewed: `packages/utils/src/*` correctly shared one crate (`leptos-ui-utils`),
but each of the 8 `packages/react/src/{use-render, merge-props, internals, floating-ui-react,
csp-provider, direction-provider, types, unstable-use-media-query, utils}` Phase A units had been
mechanically given its own separate crate, named `leptos-<unit-dir-name>` verbatim — including
`leptos-floating-ui-react` (left "react" in a Leptos crate name) and `leptos-react-utils`
(confusingly named, actually `packages/react/src/utils`).

**Decision**: consolidate all 8 into one shared `leptos-ui-internals` crate. Fixed directly in
`TODO.md` — both the `crate:` field and every `done-when:` crate-path reference across all 8
`infra:` items — rather than just recording the decision here and leaving the generated file
stale.

Final crate list: `leptos-ui` (components), `leptos-ui-utils` (Phase A `packages/utils`),
`leptos-ui-internals` (Phase A `packages/react/src` shared infra), plus the external
`floating-ui-leptos` dependency and the `docs-app` crate (Phase C).

## Not yet covered

These came up during the interview as still-open but weren't pursued to resolution — do not treat
their absence above as a decision either way:

- Whether log-once dedup should be dev-only/stripped in release builds (see above)
