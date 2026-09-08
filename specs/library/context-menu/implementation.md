# Context Menu implementation spec

Companion to behavior.md (the WHAT). This file explains the HOW/WHY: where state lives, which
hooks compose the gesture handling, what crosses the context boundary, and which Base UI internals
the unit leans on. The unit's own source is small — two real components (`Root`, `Trigger`) and a
re-exported `Positioner` — because Context Menu is Menu plus pointer-gesture plumbing.

The `TODO.md` entry (`TODO.md:365-371`) has no `wraps-external:` field, so there is no external
package whose internals this spec must delegate to; every dependency below is in-repo.

## State machine / hooks used

The open/closed state machine is not owned by this unit. It lives in the shared Menu store
(`MenuStore`, a `ReactStore` with selectors); ContextMenu contributes exactly one piece of React
state — the anchor position — plus the gesture plumbing that calls `setOpen` on that store.

Root (`packages/react/src/context-menu/root/ContextMenuRoot.tsx`):

- `React.useState` for the anchor virtual element — the only React state in the unit
  (`packages/react/src/context-menu/root/ContextMenuRoot.tsx:17-21`). The initial value is a
  zero-size `DOMRect` at the origin, so the positioner always has a well-formed
  `getBoundingClientRect` before the first open.
- Six `React.useRef` coordination refs: `backdropRef`, `internalBackdropRef`, `actionsRef`,
  `positionerRef`, `allowMouseUpTriggerRef`, `initialCursorPointRef`
  (`packages/react/src/context-menu/root/ContextMenuRoot.tsx:23-28`). Note
  `allowMouseUpTriggerRef` starts `true` (`packages/react/src/context-menu/root/ContextMenuRoot.tsx:27`).
- `useId` from `@base-ui/utils/useId` produces `rootId`
  (`packages/react/src/context-menu/root/ContextMenuRoot.tsx:3`, `packages/react/src/context-menu/root/ContextMenuRoot.tsx:29`)
  — the tree identity used for popup-ownership checks (below).
- `React.useMemo` for the context value, keyed on `[anchor, id]`; refs are stable
  (`packages/react/src/context-menu/root/ContextMenuRoot.tsx:31-44`).
- Props composition: `Menu.Root.Props` minus the props that make no sense here (`handle`,
  `triggerId`, `defaultTriggerId`, `modal`, `openOnHover`, `delay`, `closeDelay`,
  `closeParentOnEsc`, `onOpenChange`, render-function `children`)
  (`packages/react/src/context-menu/root/ContextMenuRoot.tsx:57-73`). `onOpenChange` is
  re-declared to narrow `eventDetails` to the Context Menu detail type
  (`packages/react/src/context-menu/root/ContextMenuRoot.tsx:77-78`); `closeParentOnEsc` is
  re-declared as a deprecated no-op (`packages/react/src/context-menu/root/ContextMenuRoot.tsx:80-84`).
  Re-exported types alias Menu's (`packages/react/src/context-menu/root/ContextMenuRoot.tsx:87-90`).

Trigger (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx`):

- Two `useTimeout` instances (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:47-48`):
  `longPressTimeout` (hold-to-open timer) and `allowMouseUpTimeout` (grace window before a
  document `mouseup` may cancel the menu).
- Store reads: `useMenuRootContext(false).store`, then `store.useState('open')` and
  `store.useState('disabled')` (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:41-43`).
- Local refs: `triggerRef`, `touchPositionRef`, `allowMouseUpRef`, `mouseUpAbortControllerRef`
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:45-50`).
- `React.useEffect` cleanup aborts a pending document `mouseup` listener on unmount
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:165-171`).
- `React.useEffect` registers a document-level (non-React) `contextmenu` listener via
  `addEventListener` + `ownerDocument`
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:173-192`).
- `useRenderElement('div', ...)` renders the trigger, with
  `stateAttributesMapping: pressableTriggerOpenStateMapping`
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:198-215`), which maps state
  `open` to `data-popup-open`
  (`packages/react/src/utils/popupStateMapping.ts:39-46`,
  `packages/react/src/utils/CommonTriggerDataAttributes.ts:4`).

One shared open path: both gestures funnel into `handleLongPress(x, y, event)`
(`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:52-74`), which
(1) records the spawn point in `initialCursorPointRef`, (2) swaps the anchor to a virtual element
at those coordinates, (3) sets `allowMouseUpRef.current = false`, (4) calls
`actionsRef.current?.setOpen(true, reason: triggerPress)`, and (5) arms the 500ms
`allowMouseUpTimeout` that later re-enables mouseup-driven cancellation
(`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:55-73`). Right-click reaches it
from `handleContextMenu` (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:76-82`),
long-press from the `touchstart` timer
(`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:141-143`).

Timing guards that produce behavior.md's gesture semantics:

- `LONG_PRESS_DELAY = 500` is the single hold-to-open constant
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:16`).
- Grace window 1 (trigger-local): a document `mouseup` within 500ms of open cannot cancel —
  `allowMouseUpTimeout` flips `allowMouseUpRef` after the delay
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:71-73`) and the mouseup handler
  bails while it is false
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:95-97`). This is the
  "menu appears under the held button" guard.
- Grace window 2 (Menu-side): `allowOutsidePressDismissalRef` starts `false` for context-menu
  parents and is flipped by a 500ms timeout after open
  (`packages/react/src/menu/root/MenuRoot.tsx:170`,
  `packages/react/src/menu/root/MenuRoot.tsx:279-300`); `useDismiss`'s `outsidePress` callback
  honors it, but bypasses the grace entirely when the open event was a `contextmenu`
  (`packages/react/src/menu/root/MenuRoot.tsx:467-478`). Net effect: right-click opens allow
  outside-press dismissal immediately, long-press opens wait out the grace.
- The 10px touch-move threshold cancels a pending long press
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:152-162`).
- The 1px spawn-point tolerance in item `onMouseUp` (below) suppresses item activation at the
  spawn cursor (`packages/react/src/menu/item/useMenuItemCommonProps.ts:88-96`).

`allowMouseUpTriggerRef` is the tree-wide item-activation gate, distinct from the trigger-local
`allowMouseUpRef`: set `true` on right-click open
(`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:80`), cleared by the first
document `mouseup` (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:93`) and on
touch start (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:131`). `MenuStore`
swaps this ref into its own context when the parent is a context menu
(`packages/react/src/menu/store/MenuStore.ts:157-159`) and inherits the parent menu's ref for
submenus (`packages/react/src/menu/store/MenuStore.ts:153`), so all items in the tree share one
gate; `useMenuItemCommonProps` reads it before dispatching a synthetic activation click
(`packages/react/src/menu/item/useMenuItemCommonProps.ts:106-110`).

Close paths converge on MenuRoot's `setOpen` closure: the trigger's gesture-end cancel calls
`actionsRef.current.setOpen(false, reason: cancelOpen)`
(`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:112-115`); item activation
dispatches a synthetic `click` with `detail: 1`
(`packages/react/src/menu/item/useMenuItemCommonProps.ts:113-117`,
`packages/react/src/utils/dispatchClickWithModifiers.ts:19-35`) whose handler emits
`close` with `reason: itemPress`
(`packages/react/src/menu/item/useMenuItemCommonProps.ts:81-85`); Escape/outside-press come from
`useDismiss` inside MenuRoot. `setOpen` runs the shared popup pipeline — cancelable via
`eventDetails.cancel`, reason bookkeeping, and a single `store.update` that mounts/unmounts the
popup with its `instantType`/`openChangeReason` (`packages/react/src/menu/root/MenuRoot.tsx:308-413`,
`packages/react/src/menu/root/MenuRoot.tsx:380-411`). `MenuStore.setOpen` (used by nested stores)
relays through the floating root's `setOpen` event
(`packages/react/src/menu/store/MenuStore.ts:165-167`,
`packages/react/src/menu/root/MenuRoot.tsx:428-442`).

Platform split: item activation on gesture-end is Mac-only — `useMenuItemCommonProps` returns
early for `button === 2` when `platform.os.mac` is false
(`packages/react/src/menu/item/useMenuItemCommonProps.ts:101-103`, imported at
`packages/react/src/menu/item/useMenuItemCommonProps.ts:3`) — which is where behavior.md's
platform-mocked test split comes from.

## Context providers/consumers

Root renders a two-layer provider sandwich and nothing else
(`packages/react/src/context-menu/root/ContextMenuRoot.tsx:46-52`):
`ContextMenuRootContext.Provider` → `MenuRootContext.Provider value={undefined}` → `Menu.Root`.
The explicit `undefined` severs any enclosing Menu/Menubar context so that MenuRoot's parent
detection lands on `{ type: 'context-menu', context }`
(`packages/react/src/menu/root/MenuRoot.tsx:94-102`), keeping a Context Menu mounted inside
another menu's subtree a standalone root. The layout effect at
`packages/react/src/menu/root/MenuRoot.tsx:253-277` re-syncs this parent type (plus floating node
ids) into the store after mount.

`ContextMenuRootContext` (`packages/react/src/context-menu/root/ContextMenuRootContext.ts:5-17`)
is created with an `undefined` default
(`packages/react/src/context-menu/root/ContextMenuRootContext.ts:19-21`) and consumed through
`useContextMenuRootContext(optional)`, which throws the standard missing-provider error when
`optional` is false (`packages/react/src/context-menu/root/ContextMenuRootContext.ts:23-32`).
What crosses the boundary, and who reads/writes it:

- `anchor` / `setAnchor` — written by the trigger on open
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:57-66`); read by
  `MenuPositioner` as the default anchor for context-menu parents
  (`packages/react/src/menu/positioner/MenuPositioner.tsx:88-95`).
- `actionsRef` — filled by MenuRoot with `{ setOpen }` via `useImperativeHandle`
  (`packages/react/src/menu/root/MenuRoot.tsx:465`); called by the trigger to open and cancel
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:69`,
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:112-115`).
- `positionerRef` — filled by MenuRoot with the store's `positionerElement`
  (`packages/react/src/menu/root/MenuRoot.tsx:459-463`); read by the trigger's mouseup handler to
  skip cancellation for targets inside the positioner
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:104-106`).
- `backdropRef` — filled by a user-rendered `MenuBackdrop` (the `Backdrop` part attaches it
  alongside the forwarded ref, `packages/react/src/menu/backdrop/MenuBackdrop.tsx:36-39`); read by
  the trigger's document `contextmenu` prevention
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:182-185`).
- `internalBackdropRef` — filled by the `InternalBackdrop` that `MenuPositioner` renders for
  context-menu parents (`packages/react/src/menu/positioner/MenuPositioner.tsx:302-312`); same
  prevention check as `backdropRef`.
- `allowMouseUpTriggerRef` — see state machine section; shared with items via the store.
- `initialCursorPointRef` — written on open
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:55`); consumed-and-cleared by
  item `onMouseUp` (`packages/react/src/menu/item/useMenuItemCommonProps.ts:88-89`).
- `rootId` — becomes the store's `rootId` through the selector
  (`packages/react/src/menu/store/MenuStore.ts:65-71`), is stamped on every popup in the tree as
  `data-rootownerid` (`packages/react/src/menu/popup/MenuPopup.tsx:110`), and is matched by
  `findRootOwnerId`'s ancestor walk in the trigger's mouseup handler
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:108-110`,
  `packages/react/src/menu/utils/findRootOwnerId.ts:3-13`) — so a mouseup inside any portaled
  popup of this tree, including submenus, never cancels the menu.

`MenuRootContext` (the store) is consumed by the trigger
(`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:41`) and by every shared part.
All other public parts are Menu components re-exported under the ContextMenu namespace —
`Portal`, `Popup`, `Backdrop`, `Arrow`, `Group`, `GroupLabel`, `Item`, `CheckboxItem`,
`CheckboxItemIndicator`, `LinkItem`, `RadioGroup`, `RadioItem`, `RadioItemIndicator`,
`SubmenuRoot`, `SubmenuTrigger`, `Separator` — with only `Root`, `Trigger`, and `Positioner`
being Context Menu-owned (`packages/react/src/context-menu/index.parts.ts:1-21`). They reach the
context-menu context because they render under the Root's providers; `index.ts` mirrors the same
split for types (`packages/react/src/context-menu/index.ts:1-6`).

## DOM/portal strategy and why

- Root renders no DOM; Trigger renders a plain `div` via `useRenderElement`
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:198`) — the right-click area
  is an ordinary stylable surface, not a button.
- Positioning is element-less by design: the anchor is a virtual element synthesized from pointer
  coordinates — zero-size for mouse, 10×10 for touch
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:57-66`). A virtual rect lets
  the entire floating-ui positioning pipeline run unchanged against a cursor point that
  corresponds to no DOM node. Because `clientX`/`clientY` are viewport coordinates, the
  positioner is forced to `positionMethod: 'fixed'` whenever a context-menu context is present
  (`packages/react/src/menu/positioner/MenuPositioner.tsx:114`).
- Context-menu positioning defaults are enforced inside the shared `MenuPositioner` when
  `parent.type === 'context-menu'`: `align: 'start'`, `alignOffset: 2`, `sideOffset: -5` (applied
  only when `side` is unspecified and `align` isn't `'center'`)
  (`packages/react/src/menu/positioner/MenuPositioner.tsx:88-95`), `arrowPadding: 0`
  (`packages/react/src/menu/positioner/MenuPositioner.tsx:120`), and a context-menu-specific
  `shift` with `rootBoundary: 'layoutViewport'` whose `crossAxis` shift is disabled when
  `collisionAvoidance.side === 'flip'`
  (`packages/react/src/menu/positioner/MenuPositioner.tsx:128-133`; the default collision
  avoidance preset is the dropdown one, `packages/react/src/menu/positioner/MenuPositioner.tsx:53`).
  The public `ContextMenuPositioner` is literally `MenuPositioner` re-exported with a narrowed
  prop interface — the positioning props are re-declared only to document the context-menu
  defaults (`packages/react/src/context-menu/positioner/ContextMenuPositioner.tsx:18-34`,
  `packages/react/src/context-menu/positioner/ContextMenuPositioner.tsx:66-67`,
  `packages/react/src/context-menu/positioner/ContextMenuPositioner.tsx:95-96`,
  `packages/react/src/context-menu/positioner/ContextMenuPositioner.tsx:111-113`); the CSS-vars
  and data-attributes modules are pure re-exports
  (`packages/react/src/context-menu/positioner/ContextMenuPositionerCssVars.ts:1`,
  `packages/react/src/context-menu/positioner/ContextMenuPositionerDataAttributes.ts:1`).
- Portal: `ContextMenu.Portal` is `MenuPortal`, which renders a `FloatingPortal` host (the
  element carrying `data-base-ui-portal`) only while the popup is mounted, with
  `keepMounted = false` by default — hence closed menus are fully absent from the DOM
  (`packages/react/src/menu/portal/MenuPortal.tsx:19-27`,
  `packages/react/src/menu/portal/MenuPortal.tsx:35-38`). The `container` prop passes through to
  `FloatingPortal`.
- The internal "inert blocker" from behavior.md is `InternalBackdrop`, rendered by
  `MenuPositioner` as a sibling of the floating node — i.e. a direct child of the portal host.
  It renders `role="presentation"`, `data-base-ui-inert`, `position: fixed; inset: 0`,
  `user-select: none` (`packages/react/src/utils/InternalBackdrop.tsx:19-33`), and is rendered
  when mounted + modal
  (`packages/react/src/menu/positioner/MenuPositioner.tsx:286-312`). The `data-base-ui-inert`
  attribute exists specifically so Floating UI's outside-press detection treats it as
  pre-existing content. Modal is forced on for context menus by the store's `modal` selector
  (`packages/react/src/menu/store/MenuStore.ts:57-59`) — consistent with `modal` being omitted
  from the public Root props (`packages/react/src/context-menu/root/ContextMenuRoot.tsx:64`).
  Scroll locking for the modal state is handled by `useAnchoredPopupScrollLock` inside
  `MenuPositioner` (`packages/react/src/menu/positioner/MenuPositioner.tsx:270-275`).
- Native `contextmenu` default-prevention is layered: the React-level open handler calls
  `stopEvent` (preventDefault + stopPropagation,
  `packages/react/src/floating-ui-react/utils/event.ts:3-6`) on the open path
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:81`), and the document-level
  listener default-prevents any `contextmenu` whose target is inside the trigger, internal
  backdrop, or user backdrop (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:173-192`).
  Why a document listener in addition: it is independent of React's synthetic-event layer, so it
  still prevents default when the consumer's `onContextMenu` called
  `event.preventBaseUIHandler()` — under `mergeProps`, consumer handlers run first and a
  prevented flag suppresses the internal handler
  (`packages/react/src/merge-props/mergeProps.ts:230-249`,
  `packages/react/src/merge-props/mergeProps.ts:268-274`) — and containment via `contains` covers
  portaled subtrees nested inside the trigger, matching behavior.md's prevention-through-portal
  case.
- Shadow-DOM-safe primitives are used for all traversal and targeting: `contains`, `getTarget`,
  `stopEvent` from floating-ui-react utils
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:6`,
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:102-106`,
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:179-184`) and `ownerDocument`
  for document lookups
  (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:83`,
  `packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:190`).

## Dependencies on other Base UI internals

No `wraps-external:` field exists on this unit's `TODO.md` entry (`TODO.md:365-371` lists only
`crate`, `specs`, `blocked-by`, `status`, `done-when`, `docs-pair`), so there is no external
package delegation to state: the pointer-gesture logic is owned by this unit, and everything else
is reused from in-repo internals.

- `menu/` — the dominant dependency; Context Menu is Menu plus gesture plumbing:
  - `Menu.Root` — the entire open/close state machine and floating integration
    (`packages/react/src/menu/root/MenuRoot.tsx:55-649`).
  - `MenuStore` — parent-linked per-menu stores; ref/flag inheritance across the tree
    (`packages/react/src/menu/store/MenuStore.ts:126-162`).
  - Re-exported parts (see context providers section): Portal, Popup, Backdrop, Arrow, Group*,
    Item variants, Submenu* (`packages/react/src/context-menu/index.parts.ts:4-19`).
  - `useMenuItemCommonProps` — consumes `initialCursorPointRef`, `allowMouseUpTriggerRef`, and
    `platform.os.mac` on behalf of this unit's gesture semantics
    (`packages/react/src/menu/item/useMenuItemCommonProps.ts:86-118`).
  - `findRootOwnerId` (`packages/react/src/menu/utils/findRootOwnerId.ts:3-13`),
    `MenuPopup`'s `data-rootownerid` stamping (`packages/react/src/menu/popup/MenuPopup.tsx:110`).
- `floating-ui-react/` — `FloatingPortal`, `FloatingNode`, `FloatingTree`, `useDismiss`,
  `useListNavigation`, `useTypeahead`, `useSyncedFloatingRootContext`, `FloatingTreeStore`, and
  the DOM-safe utils (`contains`, `getTarget`, `stopEvent`).
- `internals/` — `useRenderElement`, `createChangeEventDetails`
  (`packages/react/src/internals/createBaseUIEventDetails.ts:118-149`), `REASONS`,
  `useAnchorPositioning` (drives `MenuPositioner`), `useAnimationsFinished`, `DirectionContext`,
  collision-avoidance constants.
- `utils/` — `popupStateMapping` (`packages/react/src/utils/popupStateMapping.ts:39-46`),
  `InternalBackdrop` (`packages/react/src/utils/InternalBackdrop.tsx:6-35`), `usePositioner`,
  `useAnchoredPopupScrollLock`, `dispatchClickWithModifiers`
  (`packages/react/src/utils/dispatchClickWithModifiers.ts:19-35`), `useOpenInteractionType`, and
  the `popups/` open-state machinery (`useOpenStateTransitions`, `createPopupOpenState`,
  `usePopupInteractionProps`).
- `merge-props` — the `preventBaseUIHandler` contract
  (`packages/react/src/merge-props/mergeProps.ts:230-274`).
- `@base-ui/utils/` — `useId`, `useTimeout` (three call sites across the unit and MenuRoot),
  `useStableCallback`, `useIsoLayoutEffect`, `useRefWithInit`, `addEventListener`, `owner`
  (`ownerDocument`), `inertValue`, `platform`, `store` (`ReactStore`), `empty`, `fastHooks`.
- For the dependency-graph stage: this unit's coarse `blocked-by: [Phase A complete]` should
  eventually resolve to at minimum — `menu` (root/store/positioner/portal/popup/item machinery),
  `floating-ui-react`, `internals/useAnchorPositioning`, `utils/popups`, `merge-props`, and the
  `@base-ui/utils` modules listed above.

## Anything in source not explained by any test

Flagged gaps — none of the following is exercised by the three test files mined in behavior.md.
Listed for the golden-fixture stage and the backward-looking audit loop:

1. The `nested-context-menu` parent variant is vestigial: declared in `MenuParent`
   (`packages/react/src/menu/root/MenuRoot.tsx:799-803`) and special-cased in `MenuPositioner`'s
   backdrop ref wiring (`packages/react/src/menu/positioner/MenuPositioner.tsx:304-308`), but no
   code path ever assigns it — `MenuRoot` only produces `'menu'`, `'menubar'`, `'context-menu'`,
   or `undefined` (`packages/react/src/menu/root/MenuRoot.tsx:79-107`). Dead branch, untestable
   as written.
2. The default `sideOffset: -5` / `alignOffset: 2` for root context menus
   (`packages/react/src/menu/positioner/MenuPositioner.tsx:91-94`) is never asserted; the mined
   tests only pass explicit `alignOffset` values (behavior.md, "Public API surface" →
   `ContextMenu.Positioner`).
3. Re-open listener hygiene: aborting a previous pending `mouseup` listener before registering a
   fresh one (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:87-89`) is
   untested; only the unmount-abort path has coverage (behavior.md, "Edge cases" → trigger
   unmount mid-gesture).
4. `stopPropagation()` in `handleTouchStart`
   (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:137`) keeps nested
   context-menu triggers from both arming long-press timers; the nested-roots tests cover
   right-click nesting only (behavior.md, "Edge cases" → nested roots).
5. `WebkitTouchCallout: 'none'` on the trigger element
   (`packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx:208-210`) suppresses the iOS
   Safari long-press callout — unobservable in jsdom/Chromium and untested.
6. A plain `Menu.Root` nested inside `ContextMenu.Trigger`'s children reads `contextMenuContext`
   (it is still under the provider) with no parent menu context, so
   `packages/react/src/menu/root/MenuRoot.tsx:97-102` classifies it as a context-menu root; the
   comment at `packages/react/src/menu/root/MenuRoot.tsx:94-96` frames the provider-clearing as
   preventing misclassification, and no test covers this composition either way. The only tested
   trigger-child nesting is a full `ContextMenu.Root` (behavior.md, "Edge cases" → nested roots).
7. The deprecated `closeParentOnEsc` no-op
   (`packages/react/src/context-menu/root/ContextMenuRoot.tsx:80-84`) and the deprecated
   `positionMethod` prop on `ContextMenuPositioner`
   (`packages/react/src/context-menu/positioner/ContextMenuPositioner.tsx:42-46`, always
   overridden to `'fixed'` by `packages/react/src/menu/positioner/MenuPositioner.tsx:114`) have
   no test asserting their inertness.
8. Arrow keys cannot open a closed context menu
   (`openOnArrowKeyDown: parent.type !== 'context-menu'`,
   `packages/react/src/menu/root/MenuRoot.tsx:503`); keyboard behavior is UNVERIFIED in
   behavior.md ("Keyboard interactions").
9. The 300ms `allowTouchToCloseRef` guard against delayed mobile click events in the shared
   `setOpen` pipeline (`packages/react/src/menu/root/MenuRoot.tsx:357-368`) is generic Menu
   machinery that no context-menu test exercises.
