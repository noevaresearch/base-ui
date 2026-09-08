# Menubar — implementation spec

Unit: `menubar` (`packages/react/src/menubar/`, non-test files: `Menubar.tsx`, `MenubarContext.ts`, `MenubarDataAttributes.ts`, `index.ts`). Companion to `specs/library/menubar/behavior.md` (the WHAT). This file is the WHY/HOW. No `wraps-external:` field exists in the unit's `TODO.md` entry (`TODO.md:423-429`), so there is no external-package delegation to document — `floating-ui-react` is vendored in-repo, not an external dependency.

The one-sentence architecture: `<Menubar />` owns almost nothing. It is a thin coordinator that (a) wraps its children in a floating tree node, (b) exposes a context that lets each `Menu.Root`/`Menu.Trigger` discover it is "in a menubar", and (c) derives a single boolean (`hasSubmenuOpen`) from the floating-tree event bus. Roving focus, portals, positioning, scroll lock, and all menu semantics live in `CompositeRoot` and the `menu` package.

## State machine / hooks used

Hooks, by call site in `packages/react/src/menubar/Menubar.tsx`:

- `React.forwardRef` — `packages/react/src/menubar/Menubar.tsx:30-32`; ref is composed into the root element via the refs array (`packages/react/src/menubar/Menubar.tsx:85`).
- Prop defaults applied by destructuring (`orientation='horizontal'`, `loopFocus=true`, `modal=true`, `disabled=false`) — `packages/react/src/menubar/Menubar.tsx:34-44`.
- `React.useState` for `contentElement` — `packages/react/src/menubar/Menubar.tsx:46`. Not set directly; it is the ref-callback member of the composite refs array (`packages/react/src/menubar/Menubar.tsx:85`), so the menubar captures its own root DOM node once mounted and hands it to menus through context.
- `React.useState` for `hasSubmenuOpen` — `packages/react/src/menubar/Menubar.tsx:47`. This is the **only** state the menubar owns. Individual menu open/close is owned by each `Menu.Root` (behavior.md, "State model"); the menubar merely tracks whether any of its direct-child menus is currently open.
- `useBaseUiId(idProp)` — `packages/react/src/menubar/Menubar.tsx:49`; wraps `@base-ui/utils/useId` with a `base-ui-` prefix (`packages/react/src/internals/useBaseUiId.ts:9-11`). The id lands on the root element (`packages/react/src/menubar/Menubar.tsx:86`) and in context as `rootId` (`packages/react/src/menubar/Menubar.tsx:70`).
- `React.useRef` for `contentRef` (`packages/react/src/menubar/Menubar.tsx:57`) and `allowMouseUpTriggerRef` (`packages/react/src/menubar/Menubar.tsx:58`) — the latter is a mutable coordination flag (deliberately not state; see Context section).
- `React.useMemo` for the context value — `packages/react/src/menubar/Menubar.tsx:60-73`, memoized on every context field so consumers re-render only on real changes.

Inner component `MenubarContent` (`packages/react/src/menubar/Menubar.tsx:98-129`):

- `useFloatingNodeId()` — `packages/react/src/menubar/Menubar.tsx:99`; allocates the menubar's floating-tree node id.
- `useFloatingTree()` — `packages/react/src/menubar/Menubar.tsx:100`; non-null because `Menubar` renders `<FloatingTree>` above it (`packages/react/src/menubar/Menubar.tsx:77`); provides the `events` emitter.
- `useMenubarContext()` — `packages/react/src/menubar/Menubar.tsx:101` (non-optional form; the provider is guaranteed here).
- `React.useEffect` subscribing to the tree event bus — `packages/react/src/menubar/Menubar.tsx:103-126`.

### The `hasSubmenuOpen` state machine (`onSubmenuOpenChange`, `packages/react/src/menubar/Menubar.tsx:104-119`)

The menu package emits `menuopenchange` on the shared floating-tree event bus every time any menu in the tree transitions (`packages/react/src/menu/positioner/MenuPositioner.tsx:230`, details typed as `MenuOpenEventDetails`, imported at `packages/react/src/menubar/Menubar.tsx:14`). The menubar's reducer over that stream:

1. **Scope filter**: events whose `details.parentNodeId !== nodeId` are ignored (`packages/react/src/menubar/Menubar.tsx:105-107`). Only direct floating-tree children of the menubar node — i.e. contained top-level `Menu.Root`s — count. Nested submenus are grandchildren and can never flip the flag themselves.
2. **Open branch**: `details.open === true` → `setHasSubmenuOpen(true)`, guarded by a `!rootContext.hasSubmenuOpen` read to skip redundant renders (`packages/react/src/menubar/Menubar.tsx:109-112`).
3. **Close branch**: `details.open === false` → `setHasSubmenuOpen(false)` **only if** the close reason is neither `REASONS.siblingOpen` nor `REASONS.listNavigation` (`packages/react/src/menubar/Menubar.tsx:113-118`; `REASONS` imported from `internals/reasons`, `packages/react/src/menubar/Menubar.tsx:17`).

Branch 3 is the mechanism behind behavior.md's menu-switching suites: during a handoff from menu A to menu B (hover or arrow navigation), A's close event carries a "menubar-internal transition" reason, so the flag is not cleared mid-transition; B's open event then re-asserts `true`. Only closes caused by "real" exits (outside press, Escape, trigger click, item activation) clear the flag, which is exactly when `data-has-submenu-open` must disappear (behavior.md, "State model" and the #2222 regression suite).

### Derived state → attributes

`MenubarState` is `{ orientation, modal, hasSubmenuOpen }` (`packages/react/src/menubar/Menubar.tsx:131-144`). A custom `menubarStateAttributesMapping` maps `hasSubmenuOpen` to the attribute only when truthy and to `null` otherwise (`packages/react/src/menubar/Menubar.tsx:19-23`) — producing presence/absence semantics for `data-has-submenu-open` (`packages/react/src/menubar/MenubarDataAttributes.ts:13`) rather than a `="false"` value. `modal` and `orientation` flow through the default state→`data-*` pipeline as `data-modal`/`data-orientation` (`packages/react/src/menubar/MenubarDataAttributes.ts:4-9`).

### Hooks conspicuously absent

No `useControlled` (the menubar has no controlled open/value state — see "Anything in source not explained by any test" for the resolution of behavior.md's UNVERIFIED note), no `useIsoLayoutEffect`, no `useTimeout`, no `useStableCallback`, no direct DOM traversal utilities, no portal logic. All timing (stick-if-open, mouse-up trigger windows) lives in `MenuTrigger` (`packages/react/src/menu/trigger/MenuTrigger.tsx:125`, `packages/react/src/menu/trigger/MenuTrigger.tsx:235-236`), not here.

## Context providers/consumers

`MenubarContext` is created once with a nullable default: `React.createContext<MenubarContext | null>(null)` (`packages/react/src/menubar/MenubarContext.ts:17`); the interface is `{ modal, disabled, contentElement, setContentElement, hasSubmenuOpen, setHasSubmenuOpen, orientation, allowMouseUpTriggerRef, rootId }` (`packages/react/src/menubar/MenubarContext.ts:5-15`). The provider is mounted at `packages/react/src/menubar/Menubar.tsx:76` with the memoized value from `packages/react/src/menubar/Menubar.tsx:60-73`.

`useMenubarContext(optional?)` (`packages/react/src/menubar/MenubarContext.ts:21-30`) has two overloads (`packages/react/src/menubar/MenubarContext.ts:19-20`): without arguments it throws `` `Base UI: MenubarContext is missing. Menubar parts must be placed within <Menubar>.` `` when the context is null (the throw asserted in behavior.md, "Accessibility"); with `optional: true` it returns `null` instead. Notably, **every production consumer uses the optional form** — being inside a menubar is a mode a `Menu.Root` detects, never a hard requirement. The throw path is exercised only by test consumers.

Production consumers (both in the menu package, both optional):

- `MenuRoot` (`packages/react/src/menu/root/MenuRoot.tsx:76`) folds the context into a `MenuParent` descriptor `{ type: 'menubar', context }` (`packages/react/src/menu/root/MenuRoot.tsx:87-92`), with precedence submenu > menubar > context-menu > standalone (`packages/react/src/menu/root/MenuRoot.tsx:79-107`). The whole menu package dispatches on this parent type.
- `MenuTrigger` via local hook `useMenuParent` (`packages/react/src/menu/trigger/MenuTrigger.tsx:378-395`) builds the same descriptor (`packages/react/src/menu/trigger/MenuTrigger.tsx:382-387`).

Field-by-field crossing of the boundary:

- `modal` → read by `MenuPositioner` as `menubarModal` (`packages/react/src/menu/positioner/MenuPositioner.tsx:267`, `packages/react/src/menu/positioner/MenuPositioner.tsx:290`) for the scroll-lock/backdrop decisions documented in behavior.md, "DOM structure & portal behavior".
- `disabled` → OR-combined into the menu store's `disabled` selector (`packages/react/src/menu/store/MenuStore.ts:53-56`) and into the trigger's native `disabled` (`packages/react/src/menu/trigger/MenuTrigger.tsx:111`) — the propagation asserted in behavior.md, "State model".
- `hasSubmenuOpen` → read as `parentMenubarHasSubmenuOpen` in `MenuTrigger` (`packages/react/src/menu/trigger/MenuTrigger.tsx:163`); this is the menu-side gate that makes hover open another trigger's menu only while some submenu is already open (behavior.md, "Events").
- `contentElement` → used as `backdropCutout` in `MenuPositioner` (`packages/react/src/menu/positioner/MenuPositioner.tsx:295`): the menubar bar itself is cut out of the modal backdrop so triggers stay interactive while a menu is open.
- `setContentElement` → invoked through the composite ref composition (`packages/react/src/menubar/Menubar.tsx:85`).
- `rootId` → returned by the store's `rootId` selector for menubar parents (`packages/react/src/menu/store/MenuStore.ts:65-71`).
- `orientation` → consumed within the unit by `CompositeRoot` for trigger-to-trigger navigation (`packages/react/src/menubar/Menubar.tsx:87`); any menu-side consumption of the menubar's orientation is UNVERIFIED (not traced).
- `allowMouseUpTriggerRef` → copied into each `MenuStore`'s context when the parent is a menubar (`packages/react/src/menu/store/MenuStore.ts:153`, `packages/react/src/menu/store/MenuStore.ts:158`), set/cleared by `MenuTrigger` timers (`packages/react/src/menu/trigger/MenuTrigger.tsx:120-133`, `packages/react/src/menu/trigger/MenuTrigger.tsx:235-236`), and read during item click handling (`packages/react/src/menu/item/useMenuItemCommonProps.ts:108`). It implements "release the mouse over an item to activate it" for menus opened via trigger press-drag — a shared-mutable-ref (not state) because it must be read synchronously in event handlers without re-rendering.

## DOM/portal strategy and why

**Root element.** `CompositeRoot` renders the root `div` (its default tag, `packages/react/src/internals/composite/root/CompositeRoot.tsx:38`) and merges the menubar's built-in props — `role="menubar"`, `id`, `aria-orientation` — ahead of consumer `elementProps` so consumers can override (`packages/react/src/menubar/Menubar.tsx:86`). `render`/`className`/`style`/state-attribute mapping are forwarded (`packages/react/src/menubar/Menubar.tsx:80-84`), which is what satisfies the conformance suite (behavior.md, "Public API surface").

**Refs.** The refs array `[forwardedRef, setContentElement, contentRef]` (`packages/react/src/menubar/Menubar.tsx:85`) attaches three consumers to one DOM node: the user's ref, the context's `setContentElement` (backdrop cutout, above), and an internal ref.

**No portals are rendered by the menubar.** All popup content is portaled by each menu's own `Menu.Portal`. The menubar's entire DOM contribution beyond the root element is two invisible structures:

1. **Floating-tree membership.** `Menubar` wraps everything in `<FloatingTree>` (`packages/react/src/menubar/Menubar.tsx:77`), and `MenubarContent` wraps the children in `<FloatingNode id={nodeId}>` (`packages/react/src/menubar/Menubar.tsx:128`). This makes the menubar element the floating-tree parent of every contained top-level menu, which simultaneously:
   - scopes the `menuopenchange` listener to direct children via the `parentNodeId` check (`packages/react/src/menubar/Menubar.tsx:105-107`), and
   - lets `Menu.Root` resolve its floating parent node so popups/positioners join the menubar's tree (dismissal cascades, `floatingTreeRoot` selector in `packages/react/src/menu/store/MenuStore.ts:77-83`).
2. **`contentElement` through context** (see Context section) — the only DOM value the menubar exports.

**The `aria-owns` ownership span is produced by the portal stack, not the menubar.** This explains behavior.md's contained/detached asymmetry without any menubar-side code:

- `MenuPortal` decides `portalOwnerRole = 'group'` iff the portal's React-tree parent is a menu or a menubar (`packages/react/src/menu/portal/MenuPortal.tsx:29-32`). The comment there is the design rationale: the role is chosen from **where `Menu.Root` sits in the React tree** (context), deliberately not from the store's `parent`, which a detached trigger overwrites with its own (`packages/react/src/menu/portal/MenuPortal.tsx:29-31`).
- `FloatingPortal` then renders the visually hidden `span[role][aria-owns=portalNodeId]` **inline at the React-tree position** (`packages/react/src/floating-ui-react/components/FloatingPortal.tsx:270`) while the actual portal `<div id data-base-ui-portal>` is `createPortal`-ed into the container (defaulting to `document.body`, `packages/react/src/floating-ui-react/components/FloatingPortal.tsx:106-109`, `packages/react/src/floating-ui-react/components/FloatingPortal.tsx:140-143`) and the popup children are portaled into that div (`packages/react/src/floating-ui-react/components/FloatingPortal.tsx:272`).
- Consequence: with contained triggers, `Menu.Root` is a JSX descendant of `<Menubar>`, so the inline owner span mounts **inside** the menubar div, where `role="group"` is meaningful — it makes the portaled menus count as menubar children for `aria-required-children`. With detached triggers, the `Menu.Root handle` lives outside the menubar element, so `useMenubarContext(true)` there returns `null`, the parent type is `undefined`, and the span lands outside the menubar **with no role** — exactly behavior.md, "Accessibility" and "DOM structure & portal behavior".

**Hover-highlight gating.** `highlightItemOnHover={hasSubmenuOpen}` (`packages/react/src/menubar/Menubar.tsx:90`) passes the derived flag into the composite: triggers highlight on hover only while some menu is open. This is the composite-layer half of behavior.md's hover-chaining; the menu-layer half is the `parentMenubarHasSubmenuOpen` read in `MenuTrigger` (`packages/react/src/menu/trigger/MenuTrigger.tsx:163`).

## Dependencies on other Base UI internals

Everything `Menubar.tsx` imports (`packages/react/src/menubar/Menubar.tsx:3-17`), grouped:

- **`../floating-ui-react`** (vendored fork of Floating UI React at `packages/react/src/floating-ui-react/`): `FloatingTree` (`packages/react/src/menubar/Menubar.tsx:77`), `FloatingNode` (`packages/react/src/menubar/Menubar.tsx:128`), `useFloatingNodeId` (`packages/react/src/menubar/Menubar.tsx:99`), `useFloatingTree` (`packages/react/src/menubar/Menubar.tsx:100`, for the event bus). The `menuopenchange` event this unit listens to is emitted by menu internals (`packages/react/src/menu/positioner/MenuPositioner.tsx:230`) and consumed here; the `aria-owns` span is a `FloatingPortal` feature (`packages/react/src/floating-ui-react/components/FloatingPortal.tsx:270`).
- **`../internals/`**: `BaseUIComponentProps` (`packages/react/src/menubar/Menubar.tsx:10`), `useBaseUiId` (`packages/react/src/menubar/Menubar.tsx:13`; `packages/react/src/internals/useBaseUiId.ts:9-11`), `StateAttributesMapping` from `getStateAttributesProps` (`packages/react/src/menubar/Menubar.tsx:15`), `REASONS` (`packages/react/src/menubar/Menubar.tsx:17`).
- **`../internals/composite/root/CompositeRoot`** (`packages/react/src/menubar/Menubar.tsx:12`) — the entire roving-focus engine, composing `useCompositeRoot` and `useRenderElement` (`packages/react/src/internals/composite/root/CompositeRoot.tsx:44-71`). Every trigger-level keyboard behavior in behavior.md's "Keyboard interactions" — arrow navigation, `Home`/`End` (`enableHomeAndEndKeys`, `packages/react/src/menubar/Menubar.tsx:89`), `loopFocus` wrap/clamp (`packages/react/src/menubar/Menubar.tsx:88`), orientation awareness, skip-disabled — is implemented there, not in the menubar. The composite exposes a context of `{ highlightedIndex, highlightItemOnHover, relayKeyboardEvent, ... }` to items (`packages/react/src/internals/composite/root/CompositeRoot.tsx:73-80`).
- **`../menu/root/MenuRoot`** — type-only import of `MenuRoot.Orientation` reused for the context and state types (`packages/react/src/menubar/Menubar.tsx:9`, `packages/react/src/menubar/Menubar.tsx:135`, `packages/react/src/menubar/Menubar.tsx:161`; `packages/react/src/menubar/MenubarContext.ts:3`, `packages/react/src/menubar/MenubarContext.ts:12`).
- **`./MenubarContext`** — the menubar↔menu contract itself (`MenubarContext.ts`), consumed cross-package by `packages/react/src/menu/root/MenuRoot.tsx:20` and `packages/react/src/menu/trigger/MenuTrigger.tsx:35`.
- **Transitive, via context**: `MenuStore` selectors (`packages/react/src/menu/store/MenuStore.ts:53-71`), `MenuPositioner` (`packages/react/src/menu/positioner/MenuPositioner.tsx:267-295`), `MenuTrigger` (`packages/react/src/menu/trigger/MenuTrigger.tsx:111-236`), `MenuPortal`/`FloatingPortal` (above), `useMenuItemCommonProps` (`packages/react/src/menu/item/useMenuItemCommonProps.ts:108`).

Dependency summary for downstream synthesis (precise per-component edges): `menubar` depends on `internals/composite` (roving focus), `floating-ui-react` (tree + portal + event bus), `internals/useBaseUiId`, `internals/reasons`, and defines a context that the `menu` package consumes (soft dependency edge pointing *into* `menubar` from `menu`). Public surface is a two-line re-export of the component and its types (`packages/react/src/menubar/index.ts:1-3`).

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture stage and backward-looking audit:

1. **`contentRef` is attached but never read** (`packages/react/src/menubar/Menubar.tsx:57`, sole other occurrence `packages/react/src/menubar/Menubar.tsx:85`). Within the unit it is write-only; no test observes it. Possibly vestigial or reserved for external composition.
2. **Mouse-up-release item activation** — `allowMouseUpTriggerRef` is set 200 ms after a menubar menu opens (`packages/react/src/menu/trigger/MenuTrigger.tsx:235-236`) and gates a fast-path in item click handling (`packages/react/src/menu/item/useMenuItemCommonProps.ts:108`). No test in behavior.md asserts press-drag-release activation of items or the 200 ms window; the ref's plumbing is only observable through its absence (nothing breaks in the tested paths).
3. **`data-modal` on the menubar root** (`packages/react/src/menubar/MenubarDataAttributes.ts:4`): the scroll-lock suites assert `<html>`/`<body>` effects, never this attribute.
4. **`data-orientation` on the menubar root** (`packages/react/src/menubar/MenubarDataAttributes.ts:9`): tests assert `aria-orientation` only (behavior.md, "Accessibility"). The attribute is emitted via the default state mapping but is unobserved.
5. **`rootId`'s final consumer** — `MenuStore`'s selector resolves it for menubar parents (`packages/react/src/menu/store/MenuStore.ts:65-71`), but the downstream reader of that selector (aria wiring vs. portal id derivation) was not traced within this unit; no menubar test pins its effect.
6. **Effect resubscription churn** — `MenubarContent`'s effect lists `rootContext` in its deps (`packages/react/src/menubar/Menubar.tsx:126`), so it unsubscribes/resubscribes on every context value change (each `hasSubmenuOpen` flip, prop change, or element mount). Benign (off/on within the same commit), untested, and worth preserving in a port only if the port's event bus has re-subscription side effects.
7. **Resolution of a Stage 1 UNVERIFIED marker**: behavior.md's "State model" flags as unverified that the menubar exposes no `value`/`open` props. Confirmed from source: the destructured props (`packages/react/src/menubar/Menubar.tsx:34-44`) and `MenubarProps` (`packages/react/src/menubar/Menubar.tsx:146-168`) contain only `modal`, `disabled`, `orientation`, `loopFocus`, plus the standard `BaseUIComponentProps` surface. The menubar has no controlled state of any kind; each `Menu.Root` keeps its own.
