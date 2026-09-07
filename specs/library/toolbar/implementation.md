# Toolbar — implementation spec (Stage 2: implementation mining)

Unit: `toolbar` (packages/react/src/toolbar). Companion to `behavior.md` (same directory), which
is ground truth for WHAT happens; this document explains the state machine, hook composition,
context wiring, and DOM decisions that produce it. The TODO.md entry for `library: toolbar`
(`TODO.md:555-561`) has no `wraps-external:` field — first-party implementation, no external
delegation to state.

The defining architectural fact: Toolbar owns almost no logic. It is a thin configuration layer
over the shared `internals/composite` roving-focus machine plus `useButton`. Everything in
`behavior.md` under Keyboard interactions, Focus management, and most of State model is produced
by that delegated machinery, parameterized by three toolbar-specific inputs: `orientation`,
`loopFocus`, and a derived `disabledIndices` array.

## State machine / hooks used

Toolbar has no controlled state anywhere. `orientation`, `disabled`, and `loopFocus` are plain
props — `useControlled` is not used in this unit and there is nothing to control. The only state
is the composite layer's roving tab stop plus one feedback map:

- `React.useState` — `ToolbarRoot` holds `itemMap: Map<Node, CompositeMetadata<ToolbarRoot.ItemMetadata>>`
  (`packages/react/src/toolbar/root/ToolbarRoot.tsx:32-34`). This is the "items report upward"
  channel: `onMapChange={setItemMap}` is handed to `CompositeRoot`
  (`packages/react/src/toolbar/root/ToolbarRoot.tsx:74`), which forwards it to `CompositeList`'s
  registration flush. The map values carry each item's `{ disabled, focusableWhenDisabled }`
  metadata, declared as `ToolbarRootItemMetadata`
  (`packages/react/src/toolbar/root/ToolbarRoot.tsx:81-84`).
- `React.useMemo` — three sites: the derived `disabledIndices` array
  (`packages/react/src/toolbar/root/ToolbarRoot.tsx:36-46`), the memoized `ToolbarRootContext`
  value (`packages/react/src/toolbar/root/ToolbarRoot.tsx:48-54`), and per-item `itemMetadata`
  objects in Button/Input (`packages/react/src/toolbar/button/ToolbarButton.tsx:37-40`,
  `packages/react/src/toolbar/input/ToolbarInput.tsx:35-38`; memoization keeps the callback-ref
  registration stable per `packages/react/src/internals/composite/list/useCompositeListItem.ts:59-61`).
- The full state cycle: item metadata flows UP through `CompositeList` → root's `itemMap` →
  `disabledIndices` = indices where `disabled && !focusableWhenDisabled`
  (`packages/react/src/toolbar/root/ToolbarRoot.tsx:41`) → passed back DOWN into `CompositeRoot`
  (`packages/react/src/toolbar/root/ToolbarRoot.tsx:72`) → composite navigation skips those
  indices. Because metadata changes arrive as a state update *after* the map is populated, the
  composite re-validates the default tab stop in a layout effect whenever `disabledIndices`
  changes — its comment names Toolbar as the motivating case
  (`packages/react/src/internals/composite/root/useCompositeRoot.ts:166-193`).
- Roving tabindex state itself lives in `useCompositeRoot`
  (`packages/react/src/internals/composite/root/useCompositeRoot.ts:90-108`): uncontrolled
  `highlightedIndex` starting at 0 (Toolbar passes neither `highlightedIndex` nor
  `onHighlightedIndexChange` — `packages/react/src/toolbar/root/ToolbarRoot.tsx:65-76`), with the
  root's `onMapChange` handler picking the initial tab stop and moving it off disabled items
  (`packages/react/src/internals/composite/root/useCompositeRoot.ts:110-164`). Items read
  `isHighlighted = highlightedIndex === index` and render `tabIndex: isHighlighted ? 0 : -1`
  (`packages/react/src/internals/composite/item/useCompositeItem.ts:21-27`) — that is the entire
  roving mechanism, and the `onFocus` handler on each item syncs the index when focus arrives by
  any other means (`packages/react/src/internals/composite/item/useCompositeItem.ts:28-30`).
- Arrow-key navigation is the root-level `onKeyDown` in `useCompositeRoot`
  (`packages/react/src/internals/composite/root/useCompositeRoot.ts:206-317`):
  orientation/RTL key mapping at `packages/react/src/internals/composite/root/useCompositeRoot.ts:221-226`
  (vertical is not direction-flipped), loop vs. hard-stop at
  `packages/react/src/internals/composite/root/useCompositeRoot.ts:282-300` driven by `loopFocus`
  (default `true` from `packages/react/src/internals/composite/root/useCompositeRoot.ts:77`;
  `ToolbarRoot` only forwards the prop, `packages/react/src/toolbar/root/ToolbarRoot.tsx:73`), and
  the text-input caret guards at
  `packages/react/src/internals/composite/root/useCompositeRoot.ts:228-246` plus
  select-all-on-focus at `packages/react/src/internals/composite/root/useCompositeRoot.ts:319-330`
  — both explain the input-specific clauses of `behavior.md` (Keyboard interactions, Focus
  management).
- `React.useContext` via two tiny accessor hooks. `useToolbarRootContext` is overload-typed for
  optional consumption and throws the context-missing error otherwise
  (`packages/react/src/toolbar/root/ToolbarRootContext.ts:12-23`, message at
  `packages/react/src/toolbar/root/ToolbarRootContext.ts:18`); `useToolbarGroupContext` never
  throws — it returns `undefined` outside a group
  (`packages/react/src/toolbar/group/ToolbarGroupContext.ts:10-12`).
- `useButton` (Button only, `packages/react/src/toolbar/button/ToolbarButton.tsx:42-46`) — a
  hook-composed behavior bundle, not a state machine: it internally re-runs
  `useFocusableWhenDisabled` (`packages/react/src/internals/use-button/useButton.ts:28-34`),
  detects composite membership from context (`packages/react/src/internals/use-button/useButton.ts:25-26`),
  installs the dev-only `nativeButton` mismatch warnings
  (`packages/react/src/internals/use-button/useButton.ts:36-66` — the two errors quoted in
  `behavior.md` Accessibility), and supplies disabled event suppression plus Enter/Space click
  synthesis (`packages/react/src/internals/use-button/useButton.ts:104-176`).
- `useFocusableWhenDisabled` (Input directly,
  `packages/react/src/toolbar/input/ToolbarInput.tsx:40-45`; Button indirectly through
  `useButton`) — the disabled-attribute strategy selector. With `composite: true,
  isNativeButton: false` (the Input configuration) it yields `aria-disabled` but never a native
  `disabled` attribute, and an `onKeyDown` that prevents all non-Tab keys while disabled
  (`packages/react/src/utils/useFocusableWhenDisabled.ts:20-47`). The native-`disabled` branch is
  `isNativeButton && !focusableWhenDisabled`
  (`packages/react/src/utils/useFocusableWhenDisabled.ts:45-47`) — which is what
  `focusableWhenDisabled={false}` on a Button flips (per `behavior.md` State model).
- `useRenderElement` (Group directly, `packages/react/src/toolbar/group/ToolbarGroup.tsx:43-47`;
  also the renderer inside `CompositeRoot`/`CompositeItem`) — resolves `render`, merges the props
  arrays, and maps `state` to `data-*` attributes via `getStateAttributesProps`
  (`packages/react/src/internals/useRenderElement.tsx:22-48`;
  `packages/react/src/internals/getStateAttributesProps.ts:24-29`). The `*DataAttributes.ts`
  files in this unit (`packages/react/src/toolbar/button/ToolbarButtonDataAttributes.ts:4-13` and
  siblings) are constant-documentation of that mapping: `state.disabled` → `data-disabled`,
  `state.orientation` → `data-orientation`, `state.focusable` → `data-focusable`.
- `React.forwardRef` wraps all six parts; note `useButton`'s `buttonRef` is merged ahead of the
  forwarded ref (`packages/react/src/toolbar/button/ToolbarButton.tsx:62`) because its callback
  ref also runs `updateDisabled` (`packages/react/src/internals/use-button/useButton.ts:234-237`),
  the layout-effect shim that strips a native `disabled` attribute from a disabled-but-focusable
  composite button that rendered another button (e.g.
  `<Toolbar.Button disabled render={<Menu.Trigger/>}/>` —
  `packages/react/src/internals/use-button/useButton.ts:68-89`).
- The unit contains **no effects of its own** — no `useIsoLayoutEffect`, `useStableCallback`, or
  timeout/animation usage in `packages/react/src/toolbar/**`; all timing-sensitive work (layout
  effects in `CompositeList`, the `queueMicrotask` focus move at
  `packages/react/src/internals/composite/root/useCompositeRoot.ts:313-315`) belongs to the
  internals layer.

Disabled-state resolution is a pure OR-composition evaluated per item: root context ∥ group
context ∥ own prop (`packages/react/src/toolbar/button/ToolbarButton.tsx:35`,
`packages/react/src/toolbar/input/ToolbarInput.tsx:33`,
`packages/react/src/toolbar/group/ToolbarGroup.tsx:29`). The group context is optional so the
same item code works inside or outside a group; the link and the group itself are the only parts
that read root context without merging group state
(`packages/react/src/toolbar/link/ToolbarLink.tsx:26` ignores `disabled` entirely — constant
never-disabled metadata at `packages/react/src/toolbar/link/ToolbarLink.tsx:8-12` — and
`packages/react/src/toolbar/group/ToolbarGroup.tsx:27-29` merges root into its own disabled
value).

## Context providers/consumers

Two toolbar-owned contexts, both narrow:

- `ToolbarRootContext` — `{ disabled, orientation }`
  (`packages/react/src/toolbar/root/ToolbarRootContext.ts:5-8`), provided by `ToolbarRoot` around
  the `CompositeRoot` element (`packages/react/src/toolbar/root/ToolbarRoot.tsx:64`). Consumers:
  `Group` (`packages/react/src/toolbar/group/ToolbarGroup.tsx:27`, non-optional), `Button`
  (`packages/react/src/toolbar/button/ToolbarButton.tsx:31`), `Input`
  (`packages/react/src/toolbar/input/ToolbarInput.tsx:29`), `Link`
  (`packages/react/src/toolbar/link/ToolbarLink.tsx:26`, reads `orientation` only), `Separator`
  (`packages/react/src/toolbar/separator/ToolbarSeparator.tsx:17`, non-optional — hence the
  throw outside a root recorded in `behavior.md` Edge cases). It crosses *downward only*, and it
  is the entire reason parts must be nested under a root.
- `ToolbarGroupContext` — `{ disabled }`
  (`packages/react/src/toolbar/group/ToolbarGroupContext.ts:4-8`), provided by `Group`
  (`packages/react/src/toolbar/group/ToolbarGroup.tsx:49-51`). Consumers: `Button` and `Input`
  only, always via the nullable accessor
  (`packages/react/src/toolbar/button/ToolbarButton.tsx:33`,
  `packages/react/src/toolbar/input/ToolbarInput.tsx:31`). `Group` is a context repeater, not an
  owner: it folds root-disabled into group-disabled
  (`packages/react/src/toolbar/group/ToolbarGroup.tsx:29`) so a nested item sees one merged
  boolean instead of needing both contexts.

Crossing the boundary in the *reverse* direction (items → root) is the composite metadata channel
described above: `CompositeItem metadata` (`packages/react/src/toolbar/button/ToolbarButton.tsx:60`,
`packages/react/src/toolbar/input/ToolbarInput.tsx:70`,
`packages/react/src/toolbar/link/ToolbarLink.tsx:38`) is registered into `CompositeList`'s node
map (`packages/react/src/internals/composite/list/useCompositeListItem.ts:62-82`), flushed to a
DOM-order-sorted map
(`packages/react/src/internals/composite/list/CompositeList.tsx:142-170`), and surfaced to
`ToolbarRoot` as `itemMap`. `ToolbarLink` exploits this channel to participate in navigation
without any disable logic: its module-constant metadata
(`packages/react/src/toolbar/link/ToolbarLink.tsx:8-12`) is never reactive.

The composite layer contributes its own provider pair below the toolbar's:
`CompositeRootContext` (`packages/react/src/internals/composite/root/CompositeRoot.tsx:73-84`) —
consumed by every item through `useCompositeItem`
(`packages/react/src/internals/composite/item/useCompositeItem.ts:17-19`) and by `useButton`'s
composite detection (`packages/react/src/internals/use-button/useButton.ts:25-26`) — and
`CompositeListContext` (`packages/react/src/internals/composite/list/CompositeList.tsx:209-215`).
Toolbar code never imports these directly; it reaches them only through
`CompositeRoot`/`CompositeItem`.

## DOM/portal strategy and why

- No portals anywhere in the unit. `Toolbar.Root`, `Group` render `div`s; items render
  `button`/`input`/`a` via `CompositeItem`'s `tag`
  (`packages/react/src/toolbar/button/ToolbarButton.tsx:56`,
  `packages/react/src/toolbar/input/ToolbarInput.tsx:66`,
  `packages/react/src/toolbar/link/ToolbarLink.tsx:34`; `CompositeRoot`'s default tag `div` for
  the root). The Group renders through `useRenderElement('div', …)` with `role: 'group'`
  (`packages/react/src/toolbar/group/ToolbarGroup.tsx:43-47`).
- The Group is deliberately **DOM-transparent for navigation**: it is not a `CompositeItem`, so
  it never occupies a tab-stop slot. Its children register individually with the root's list, and
  `CompositeList` orders registrations by `compareDocumentPosition`
  (`packages/react/src/internals/composite/list/CompositeList.tsx:227-272`, sort at
  `packages/react/src/internals/composite/list/CompositeList.tsx:255`), so "flat DOM order" in
  `behavior.md` (Keyboard interactions) falls out of document order regardless of how deeply
  items are nested in groups. A `MutationObserver` over common ancestors of adjacent items
  repairs the index map if wrappers are physically moved
  (`packages/react/src/internals/composite/list/CompositeList.tsx:89-140`).
- Root decorations: `role="toolbar"` and `aria-orientation` are injected as `defaultProps` ahead
  of consumer props (`packages/react/src/toolbar/root/ToolbarRoot.tsx:58-61`); `state { disabled,
  orientation }` becomes `data-disabled`/`data-orientation` on the root element through the same
  state-attributes path the items use.
- Props ordering matters and is load-bearing: in `CompositeItem`, `compositeProps` (the roving
  `tabIndex`, `onFocus`) are merged first, then toolbar's array (element props, conditional
  forward, `getButtonProps`), then consumer `elementProps` last, so user handlers win over
  internal ones except where a hook deliberately chains them
  (`packages/react/src/internals/composite/item/CompositeItem.tsx:27-33`,
  `packages/react/src/toolbar/button/ToolbarButton.tsx:63-73`). `ToolbarSeparator` relies on the
  same rule in reverse: it computes the perpendicular orientation and then spreads `{...props}`
  *after* it, so an explicit `orientation` prop overrides the default
  (`packages/react/src/toolbar/separator/ToolbarSeparator.tsx:19-21`).
- `ToolbarSeparator` is the only part that delegates to another public component wholesale — it
  renders `<Separator>` (`packages/react/src/toolbar/separator/ToolbarSeparator.tsx:21`),
  inheriting that component's DOM and attribute behavior, contributing only the
  perpendicular-orientation default.
- Popup portals opened from toolbar buttons belong entirely to the rendered components
  (`behavior.md` DOM structure & portal behavior); the toolbar neither hosts nor observes them.
  Its only interaction with that world is indirect: the composite `onKeyDown`'s
  `stopEventPropagation` default (`packages/react/src/internals/composite/root/CompositeRoot.tsx:33`,
  honored at `packages/react/src/internals/composite/root/useCompositeRoot.ts:303-305`) keeps
  toolbar navigation from re-handling keys that a nested composite already consumed.

## Dependencies on other Base UI internals

Direct imports by toolbar source files:

- `internals/composite` — the load-bearing dependency:
  - `CompositeRoot` (`packages/react/src/toolbar/root/ToolbarRoot.tsx:8`, rendered at
    `packages/react/src/toolbar/root/ToolbarRoot.tsx:65-76`) and its subtree: `useCompositeRoot`
    (`packages/react/src/internals/composite/root/useCompositeRoot.ts:74-340`),
    `CompositeRootContext` (`packages/react/src/internals/composite/root/CompositeRoot.tsx:6`),
    `CompositeList` + `CompositeMetadata` type
    (`packages/react/src/toolbar/root/ToolbarRoot.tsx:9`,
    `packages/react/src/internals/composite/list/CompositeList.tsx:22`), `useCompositeItem`/
    `useCompositeListItem` (`packages/react/src/internals/composite/item/useCompositeItem.ts:16`,
    `packages/react/src/internals/composite/list/useCompositeListItem.ts:32`), and the
    `composite` key/util module (`COMPOSITE_KEYS`, `findNonDisabledListIndex`, `isNativeInput`,
    etc. — `packages/react/src/internals/composite/root/useCompositeRoot.ts:9-26`).
  - `CompositeItem` (`packages/react/src/toolbar/button/ToolbarButton.tsx:9`,
    `packages/react/src/toolbar/input/ToolbarInput.tsx:8`,
    `packages/react/src/toolbar/link/ToolbarLink.tsx:6`) — every focusable part is a thin
    props/metadata shell around it.
- `internals/use-button` — `useButton` (`packages/react/src/toolbar/button/ToolbarButton.tsx:5`,
  called at `packages/react/src/toolbar/button/ToolbarButton.tsx:42`).
- `internals/useRenderElement` — `packages/react/src/toolbar/group/ToolbarGroup.tsx:3`; also the
  shared renderer inside `CompositeRoot`
  (`packages/react/src/internals/composite/root/CompositeRoot.tsx:7`) and `CompositeItem`
  (`packages/react/src/internals/composite/item/CompositeItem.tsx:4`), which is where the
  `render` prop and state→`data-*` mapping actually resolve.
- `internals/types` — `BaseUIComponentProps`, `HTMLProps`, `Orientation`, `NativeButtonProps`
  (`packages/react/src/toolbar/root/ToolbarRoot.tsx:3-7`,
  `packages/react/src/toolbar/button/ToolbarButton.tsx:4`,
  `packages/react/src/toolbar/input/ToolbarInput.tsx:3`,
  `packages/react/src/toolbar/link/ToolbarLink.tsx:3`).
- `utils/useFocusableWhenDisabled` — `packages/react/src/toolbar/input/ToolbarInput.tsx:4`
  (called at `packages/react/src/toolbar/input/ToolbarInput.tsx:40`); also pulled in inside
  `useButton` (`packages/react/src/internals/use-button/useButton.ts:11`).
- `separator` — the public `Separator` component, the entirety of `ToolbarSeparator`'s rendering
  (`packages/react/src/toolbar/separator/ToolbarSeparator.tsx:4`, rendered at
  `packages/react/src/toolbar/separator/ToolbarSeparator.tsx:21`).
- `@base-ui/utils/empty` — `EMPTY_OBJECT` for the conditional disabled-forwarding branch
  (`packages/react/src/toolbar/button/ToolbarButton.tsx:3`, used at
  `packages/react/src/toolbar/button/ToolbarButton.tsx:71`).

Indirect dependencies the porting unit inherits through the composite layer (relevant when
replacing `blocked-by: [Phase A complete]` with precise per-component edges):

- `DirectionContext`/`useDirection` — RTL arrow mapping
  (`packages/react/src/internals/composite/root/CompositeRoot.tsx:11` and
  `packages/react/src/internals/composite/root/CompositeRoot.tsx:42` →
  `packages/react/src/internals/composite/root/useCompositeRoot.ts:221-226`); toolbar tests
  exercise it via `DirectionProvider` (per `behavior.md` Edge cases).
- `floating-ui-react` utils — only `getTarget`, the shadow-DOM-safe event-target resolver used in
  the composite keydown/focus paths
  (`packages/react/src/internals/composite/root/useCompositeRoot.ts:30`, used at
  `packages/react/src/internals/composite/root/useCompositeRoot.ts:228` and
  `packages/react/src/internals/composite/root/useCompositeRoot.ts:323`).
- `@floating-ui/utils/dom` — `isHTMLElement` tag checks inside `useButton`
  (`packages/react/src/internals/use-button/useButton.ts:3`).
- `@base-ui/utils/*` plumbing used by the internals: `useStableCallback`, `useIsoLayoutEffect`,
  `useMergedRefs`, `useRefWithInit`, `isElementDisabled`, `error`, `SafeReact` (imports across
  `packages/react/src/internals/composite/root/useCompositeRoot.ts:3-7`,
  `packages/react/src/internals/composite/list/CompositeList.tsx:4-6`,
  `packages/react/src/internals/use-button/useButton.ts:4-7`).

No `use-render` package import appears in this unit; render-prop composition is resolved entirely
by `internals/useRenderElement`.

## Anything in source not explained by any test

Flagged explicitly — none of these are covered by the toolbar test suite (per `behavior.md`,
which never mentions them) and some are explicitly called out there as unasserted:

1. **`aria-orientation` on the root** (`packages/react/src/toolbar/root/ToolbarRoot.tsx:59`) —
   `behavior.md` Accessibility states no `aria-orientation` on the root is asserted anywhere, yet
   the source always sets it.
2. **Root-level `data-disabled` / `data-orientation`** — root `state`
   (`packages/react/src/toolbar/root/ToolbarRoot.tsx:56`) maps to these attributes on the root
   element, but tests only assert them on groups/items.
3. **`data-focusable` on Button/Input** — `state.focusable`
   (`packages/react/src/toolbar/button/ToolbarButton.tsx:51`,
   `packages/react/src/toolbar/input/ToolbarInput.tsx:50`; constants at
   `packages/react/src/toolbar/button/ToolbarButtonDataAttributes.ts:13`,
   `packages/react/src/toolbar/input/ToolbarInputDataAttributes.ts:13`) is rendered for every
   item but never asserted; its semantics ("remains focusable when disabled") are only documented
   by the constant's comment.
4. **Home/End keys are silently unsupported** — the composite supports `enableHomeAndEndKeys`
   (`packages/react/src/internals/composite/root/useCompositeRoot.ts:274-280`) but `ToolbarRoot`
   never passes it (`packages/react/src/toolbar/root/ToolbarRoot.tsx:65-76`), and no test pins
   this absence. A port that "helpfully" enables Home/End would deviate from the React
   implementation without any fixture catching it.
5. **Toolbar is always an uncontrolled composite** — `highlightedIndex`,
   `onHighlightedIndexChange`, `onLoop`, `grid`, `modifierKeys`, and `enableHomeAndEndKeys` are
   all absent from the `CompositeRoot` call
   (`packages/react/src/toolbar/root/ToolbarRoot.tsx:65-76`); no test would notice if any leaked
   through or if the tab stop became controllable.
6. **`stopEventPropagation` reliance** — toolbar navigation depends on the composite default of
   stopping propagation of handled navigation keys
   (`packages/react/src/internals/composite/root/CompositeRoot.tsx:33`) but no toolbar test
   observes propagation; `behavior.md` only tests the related-looking (but mechanistically
   different) popup-focus case.
7. **Native `type="button"` injection** — `useButton` adds `type: 'button'` to native buttons
   (`packages/react/src/internals/use-button/useButton.ts:226`); the toolbar suite never asserts
   it.
8. **Shared-DOM-node item ownership** — when `Toolbar.Button render={<Menu.Trigger/>}` puts two
   composite items on one DOM node, the outer item's ref attaches first and wins registration
   (`packages/react/src/internals/composite/item/CompositeItem.tsx:29-30`,
   `packages/react/src/internals/composite/list/useCompositeListItem.ts:59-61`); every
   render-prop popup test in `behavior.md` depends on this rule, but the rule itself is never
   asserted.
9. **CompositeList unmount/StrictMode/reorder machinery** — the MutationObserver moved-node
   repair (`packages/react/src/internals/composite/list/CompositeList.tsx:89-140`), Strict Mode
   dirty-map replay (`packages/react/src/internals/composite/list/CompositeList.tsx:193-200`),
   and disconnected-node filtering
   (`packages/react/src/internals/composite/list/CompositeList.tsx:235-237`) are all invisible to
   the toolbar suite; `behavior.md` Edge cases marks unmount behavior UNVERIFIED.
   `getFallbackIndex` (`packages/react/src/internals/composite/root/useCompositeRoot.ts:345-365`)
   — the tab-stop fallback when a highlighted item is replaced — is likewise untested from
   Toolbar.
10. **Disabled-forwarding asymmetry** — `ToolbarButton` forwards `disabled` to rendered
    components only when `render` is provided
    (`packages/react/src/toolbar/button/ToolbarButton.tsx:64-71`, with a TODO referencing
    mui/base-ui#1976), while `ToolbarInput` never forwards `disabled` at all (its `props` array
    is `[defaultProps, elementProps, focusableWhenDisabledProps]`,
    `packages/react/src/toolbar/input/ToolbarInput.tsx:73`); rendered inputs get their disabled
    behavior from the toolbar's own `preventWhenDisabled` handlers
    (`packages/react/src/toolbar/input/ToolbarInput.tsx:53-62`) and
    `useFocusableWhenDisabled`'s keydown guard. The observable outcomes are tested, but the
    asymmetry between the two parts' forwarding strategies is a source-only decision.
