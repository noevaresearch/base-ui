# Toast — implementation index

Index over the toast unit's five batch specs, mined per `specs/library/toast/parts/PLAN.json`.
The parts carry the depth; this file is a map to them plus the whole-unit coordination that no
single batch can state on its own. Ground truth for WHAT remains `specs/library/toast/behavior.md`
and the per-batch behavior specs.

Batches are listed in dependency order (store → shell → per-toast components → leaves), not PLAN
order.

## manager — the store and its two frontends

`ToastStore` is the single source of truth: per-toast lifecycle transitions, the wall-clock timer
map, toast metadata, and interaction flags. `createToastManager` is a stateless, framework-
agnostic event bus whose `' subscribe'` channel is consumed only by the Provider, and
`useToastManager` is the selector-keyed React subscription surface. See
`parts/manager.implementation.md` for the transition funnel, the timer machine, the promise
chain, and the type-check-only `.spec.tsx` suites.

## shell — Provider, Portal, Positioner

`Toast.Provider` owns the store instance, one-way-bridges an optional external `toastManager`
into it, and syncs `timeout`/`limit` through a dedicated first-child synchronizer component whose
effect ordering is load-bearing. `Toast.Portal` is a two-level portal
(`container → portal div → children`); `Toast.Positioner` runs anchor positioning permanently
"open" and renders the positioned div. See `parts/shell.implementation.md` for the bridge
mapping, the synchronizer mechanism, and the positioning pipeline.

## root — Toast.Root

Stateless with respect to toast existence — enter/exit are the store's `transitionStatus`
transitions. Root adds the swipe gesture FSM, height measurement that publishes
`{ rootRef, height }` into the store, aria id registration, and per-toast context provision. See
`parts/root.implementation.md` for the gesture machine, lifecycle effects, and a11y defaults.

## viewport — Toast.Viewport

Zero local React state: everything reactive is a store selector, and the component writes only
the pause inputs (`hovering`, `focused`, `isWindowFocused`, `prevFocusElement`) while driving the
global key/blur/focus/pointerdown listeners, the deferred-collapse latch, focus guards, and the
high-priority alert mirror. See `parts/viewport.implementation.md` for the listener lifecycle and
guard choreography.

## leaf-parts — Action, Arrow, Close, Content, Description, Title, utils

Thin `useRenderElement` consumers with almost no local state (Close's focus flip, Content's
Resize/MutationObservers). Render gating is centralized in `isRenderableNode` /
`useToastLabelElement`, and label parts push ids upward instead of rendering aria attributes
themselves. See `parts/leaf-parts.implementation.md` for per-part prop-merge orders and the
context-consumption map.

## Cross-cutting coordination

### One store, three contexts, one optional bridge

`ToastProviderContext`'s value IS the `ToastStore` (see `parts/manager.implementation.md`,
`parts/shell.implementation.md`); `ToastRootContext` adds per-toast state for the leaves
(see `parts/root.implementation.md`); `ToastPositionerContext` carries positioning outputs to the
Arrow only (see `parts/shell.implementation.md`, `parts/leaf-parts.implementation.md`).
Everything else — expansion, focus, limited flags, metadata — is coordinated through store
selectors rather than extra contexts (see `parts/viewport.implementation.md`). An external
`toastManager` reaches the store only through the Provider's one-way bridge and never replaces
the store (see `parts/shell.implementation.md`). `Toast.Close` is the only leaf that consumes the
provider store context directly; all other leaves read Root's context
(see `parts/leaf-parts.implementation.md`).

### Canonical composition

`Provider → Portal → Viewport → Root → leaves`, with `Positioner` inserted between Viewport and
Root only when anchoring or an Arrow is needed
(`docs/src/app/(docs)/react/components/toast/demos/hero/css-modules/index.tsx:8-15`,
`docs/src/app/(docs)/react/components/toast/demos/hero/css-modules/index.tsx:38-51`; the
Positioner requirement for Arrow: `parts/leaf-parts.implementation.md`,
`parts/shell.implementation.md`).

### Upward flows — the store's imperative logic is fed by other batches

Three hand-offs publish into shared state, and they are why store-side logic can act without
rendering anything itself:

1. Title/Description register their DOM ids into Root state, which renders the aria attributes
   (see `parts/leaf-parts.implementation.md`, `parts/root.implementation.md`).
2. Root measures height and writes `{ rootRef, height }` into the store; the store consumes
   `height` for offsetY accumulation and uses the published refs as focus targets
   (see `parts/root.implementation.md`, `parts/manager.implementation.md`).
3. The Viewport publishes its node via `store.setViewport` and the pre-F6 element via
   `prevFocusElement`; store-side focus management and the touch pointerdown predicate read those
   handles (see `parts/viewport.implementation.md`, `parts/manager.implementation.md`).

### Downward flows — precomputed selectors and per-toast context

The store precomputes `domIndex`/`visibleIndex`/`offsetY` per id
(see `parts/manager.implementation.md`); the Positioner and Root each write `--toast-index` from
those selectors, both switching to `domIndex` while a toast is ending
(see `parts/shell.implementation.md`, `parts/root.implementation.md`); `expanded`,
`visibleIndex`, and `recalculateHeight` reach the leaves only through `ToastRootContext`
(see `parts/leaf-parts.implementation.md`).

### The exit lifecycle is a multi-batch relay

`store.closeToast` stamps `ending` and `height: 0` (see `parts/manager.implementation.md`) → the
animating toast keeps its stack slot because both index writers switch to `domIndex` while ending
(see `parts/shell.implementation.md`, `parts/root.implementation.md`) → the viewport's
deferred-collapse latch blocks mouseleave collapse while ending toasts exist
(see `parts/viewport.implementation.md`) → `useOpenChangeComplete` removes the toast from the
store, triggering the metadata rebuild that re-indexes survivors
(see `parts/root.implementation.md`, `parts/manager.implementation.md`).

### Timers pause from three directions

The viewport writes the window/hover/focus inputs (see `parts/viewport.implementation.md`), Root
sets `hovering` and pauses timers on touch pointerdown while arming a swipe
(see `parts/root.implementation.md`), and the store owns the timer map, the
`expandedOrOutOfFocus` scheduling gate, and the derived paused-state reset
(see `parts/manager.implementation.md`). No batch owns timers alone.

### DOM contract — one portal, everything below renders inline

Provider renders no DOM; Portal is the only component that portals
(see `parts/shell.implementation.md`); Viewport, Positioner, Root, and all leaves render their
children inline beneath it (each batch's "DOM/portal strategy" section in `parts/`). Two
behaviors depend on this chain: Root gates Escape on DOM containment of the active element, so
React-portaled content stays a React child but exits the DOM subtree
(see `parts/root.implementation.md`), and the viewport derives
`ownerWindow`/`ownerDocument` from its own node so its listeners follow the realm it lands in
(see `parts/viewport.implementation.md`).

## Verification gaps

Each batch spec ends with "Anything in source not explained by any test"; the unit's
known-unverified surface is the union of those five lists (final section of each
`parts/<batchName>.implementation.md`). The recurring themes are attribute-level a11y defaults
(Root roles/aria, Viewport landmark semantics, Close's `aria-hidden` flip) and portal/container
mechanics (Portal container resolution, Positioner constant modules) — the areas a golden fixture
or future test pass would have to decide on first.
