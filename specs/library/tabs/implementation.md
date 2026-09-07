# Tabs implementation spec

Companion to `behavior.md` (same directory), which documents WHAT tabs does; this file documents
WHY/HOW. The `TODO.md` entry (`TODO.md:527-533`) has no `wraps-external:` field, so there is no
external package to delegate to — everything below is derived from the component's own source.
All five parts are mined from the non-test files under `packages/react/src/tabs/`; the two barrel
files (`packages/react/src/tabs/index.ts:1-7`, `packages/react/src/tabs/index.parts.ts:1-5`) are
plain re-exports of the `Tabs` namespace and contribute no behavior.

The one-line structural summary: **the root owns selection; the internal composite subsystem owns
focus; the panel owns its own mount/transition lifecycle; the indicator is a stateless render-time
measuring machine.** Keyboard navigation, roving tabindex, and list registration are not
implemented in this unit at all — they are delegated to `internals/composite`.

## State machine / hooks used

### Root: `useControlled` + an uncontrolled-only "automatic consistency" effect

- `useControlled` (`packages/react/src/tabs/root/TabsRoot.tsx:50-55`) holds `value`, with
  `defaultValue` defaulting to `0` (`packages/react/src/tabs/root/TabsRoot.tsx:32`). The setter it
  returns is a no-op when controlled (`packages/utils/src/useControlled.ts:82-91`), which is why
  the root can call `setValue` unconditionally on user changes and still satisfy the "controlled
  roots keep the exact value supplied by the parent" guarantee (State model section of
  behavior.md). `value={null}` is a *controlled* empty selection because the root's local
  `isControlled` is `valueProp !== undefined` (`packages/react/src/tabs/root/TabsRoot.tsx:57`);
  note `useControlled` itself decides control by the first-render prop
  (`packages/utils/src/useControlled.ts:41`), so the root's per-render `isControlled` gates the
  automatic-fallback effect (below) while `useControlled` gates the writes.
- Dev-mode warnings for default-value mutation come from `useControlled`
  (`packages/utils/src/useControlled.ts:74-77`) — this is the source of the
  `'Base UI: A component is changing the default value state…'` warning in the State model section
  of behavior.md. Because that hook warns but does not stop honoring the *initial* default, the
  root snapshots `defaultValueProp` into `initialDefaultValueRef`
  (`packages/react/src/tabs/root/TabsRoot.tsx:233`) and keeps a separate
  `hasExplicitDefaultValueProp` boolean computed from the raw props object
  (`packages/react/src/tabs/root/TabsRoot.tsx:43`).
- **Activation direction is computed during render, not in the commit** so children see the
  correct direction on their very first render after a selection change (the motivating bug is
  referenced at `packages/react/src/tabs/root/TabsRoot.tsx:83`). The render-phase block compares
  the previous value snapshot in `activationDirectionState`
  (`packages/react/src/tabs/root/TabsRoot.tsx:70-75`) against the current `value` and calls
  `computeActivationDirection` (`packages/react/src/tabs/root/TabsRoot.tsx:84-93`). When the newly
  selected tab element is not yet registered in `tabMap` (a tab added and selected in the same
  controlled update), `directionComputationIncomplete` keeps the previous-value snapshot stale so
  the direction is re-derived from DOM positions once the map catches up
  (`packages/react/src/tabs/root/TabsRoot.tsx:88-98`); the commit is then synced via
  `useIsoLayoutEffect` (`packages/react/src/tabs/root/TabsRoot.tsx:100-109`).
- `computeActivationDirection` (`packages/react/src/tabs/root/TabsRoot.tsx:365-408`) is
  geometry-first: it looks up both tab elements via `findTabElement`
  (`packages/react/src/tabs/root/TabsRoot.tsx:352-363`) and compares
  `getBoundingClientRect().left` (horizontal) or `.top` (vertical)
  (`packages/react/src/tabs/root/TabsRoot.tsx:375-378`, `:397-407`). Only when an element is
  missing does it fall back to comparing the raw values — and only for same-typed numbers or
  strings (`packages/react/src/tabs/root/TabsRoot.tsx:383-394`); anything else (objects,
  mixed types, missing old value) yields `'none'`. `null` on either side is `'none'` outright
  (`packages/react/src/tabs/root/TabsRoot.tsx:371-373`) — this is the mechanism behind
  `value={null}` resetting `data-activation-direction` (State model section of behavior.md).
- The user-initiated path is a single stable `onValueChange` wrapper
  (`packages/react/src/tabs/root/TabsRoot.tsx:111-125`): it *recomputes* the direction here
  (overwriting the `'none'` placeholder that `TabsTab.activate` puts into the details
  (`packages/react/src/tabs/tab/TabsTab.tsx:123-130`)), invokes the consumer prop, and only then
  calls `setValue` if `eventDetails.isCanceled` is false — the cancel-before-commit ordering that
  the Events section of behavior.md documents.
- Automatic changes are emitted by a separate `notifyAutomaticValueChange` wrapper that hardcodes
  `activationDirection: 'none'` and creates synthetic details
  (`packages/react/src/tabs/root/TabsRoot.tsx:127-136`). The details factory defaults
  `event` to `new Event('base-ui')` and `trigger` to `undefined`
  (`packages/react/src/internals/createBaseUIEventDetails.ts:130-147`) — exactly the payload
  shape asserted for the initial auto-selection in the Events section of behavior.md. Automatic
  reasons are not cancelable *by construction*: `notifyAutomaticValueChange` never routes through
  the cancelable wrapper, and the consumer's `cancel()` mutates a details object the root never
  re-reads for these paths.
- **The uncontrolled-only automatic consistency effect**
  (`packages/react/src/tabs/root/TabsRoot.tsx:241-331`) is the reason for the whole
  `initial`/`disabled`/`missing` reason taxonomy in the State model section of behavior.md. It
  returns immediately for controlled roots (`packages/react/src/tabs/root/TabsRoot.tsx:242-244`).
  Its decision inputs are `selectedTabMetadata` and `firstEnabledTabValue` — two memos that scan
  `tabMap` in registration order (`packages/react/src/tabs/root/TabsRoot.tsx:208-215`,
  `:217-226`) — plus four refs:
  - `shouldNotifyInitialValueChangeRef` — initialized true only when `defaultValue` was omitted,
    so explicit defaults are treated as user-owned and never notify `initial`
    (`packages/react/src/tabs/root/TabsRoot.tsx:230`, comment at `:228-229`);
  - `initialDefaultValueRef` (`:233`) and `shouldHonorDisabledDefaultValueRef` (`:236`) —
    implement the "explicit default may point at a disabled tab, and stays honored until the
    selection has once been valid" policy (`:283-293`);
  - `didRegisterTabsRef` + `lastKnownTabElementRef` (`:237`, `:62`) — distinguish "the list was
    swapped to empty" from "an outer Suspense boundary tore down layout effects while the tabs
    are still connected", so the latter does not emit a spurious `missing` change
    (`:264-275`, comment at `:265-266`).
  The reason resolution order is: implicit-initial transition → `REASONS.initial`; else disabled
  selection → `REASONS.disabled`; else missing selection → `REASONS.missing`
  (`:295-317`). The fallback value is `firstEnabledTabValue ?? null` (`:298`), and the
  all-disabled case (`null` with no enabled tab) plus the "already at the fallback value" early
  return that still resolves the initial notification live at `:300-305`. When the selection is
  valid and it *is* the implicit initial transition, the root notifies once with the current value
  and marks the notification delivered (`:319-322`).
- Panel registration: `registerMountedTabPanel` maintains a `Map<value, id>`
  (`packages/react/src/tabs/root/TabsRoot.tsx:45-48`, `:138-160`) whose cleanup is
  ownership-aware — the returned disposer refuses to delete a newer registration for the same
  value (`:150-152`), which is the last-panel-wins semantics asserted in the Accessibility section
  of behavior.md. `getTabPanelIdByValue` (`:163-168`) and `getTabIdByPanelValue`
  (`:171-181`) are the two id-lookup accessors that implement `aria-controls` (tab → panel id,
  only while the panel is registered) and `aria-labelledby` (panel → tab id).
- The context value is a memo of all of the above
  (`packages/react/src/tabs/root/TabsRootContext.ts:7-38` is its shape; construction at
  `packages/react/src/tabs/root/TabsRoot.tsx:183-206`), and the root renders through
  `useRenderElement` with `tabsStateAttributesMapping`, which maps only
  `tabActivationDirection` → `data-activation-direction`
  (`packages/react/src/tabs/root/stateAttributesMapping.ts:5-9`); `orientation` falls through the
  generic truthy-string mapping (`packages/react/src/internals/getStateAttributesProps.ts:24-28`)
  and becomes `data-orientation`.

### List: owns the roving-highlight state and the resize fan-out

- `TabsList` keeps two pieces of state: `highlightedTabIndex`
  (`packages/react/src/tabs/list/TabsList.tsx:34`) — passed *into* `CompositeRoot` as its
  controlled highlight with `setHighlightedTabIndex` as the change callback
  (`:119`, `:123`) — and `tabsListElement` (`:35`), captured by merging the forwarded ref into
  `CompositeRoot`'s refs array (`:116`). Owning the highlight state in the list (rather than
  letting composite hold it internally) is what lets `TabsTab` reconcile the highlight with the
  *selection* (next section).
- The ResizeObserver machinery (`:37-39`, `:41-66`) observes the list element plus every
  registered tab element and fans out to a listener set on any resize (`:46-50`); two stable
  registration callbacks are exposed through context: `registerIndicatorUpdateListener`
  (`:68-73`) and `registerTabResizeObserverElement` (`:75-82`, returning an unobserve
  disposer). A `typeof ResizeObserver === 'undefined'` guard makes the whole thing a no-op in
  environments without it (`:42-44`). This is the transport that makes indicator re-measurement
  work for the resize/transform cases in the DOM structure section of behavior.md without the
  indicator owning any observer of its own.
- The list's only DOM opinions are `role: 'tablist'` and `aria-orientation` emitted only when
  vertical (`:89-92`) — the aria-attribute behavior in the Accessibility section of behavior.md.
- Everything else about the list (keyboard handling, wrapping, RTL, Home/End, scroll-into-view)
  is `CompositeRoot` configuration (`:109-128`): `loopFocus` (`:121`), `enableHomeAndEndKeys`
  (`:120`), `orientation` (`:122`), and crucially `disabledIndices={EMPTY_ARRAY}` (`:125`) —
  tabs deliberately do *not* feed the composite a disabled-index list, because disabled tabs
  must remain focusable (Focus management section of behavior.md). Tab skipping is instead
  emergent: the composite's disabled check treats *natively* disabled elements
  (`:matches(':disabled')`) and invisible elements as disabled regardless of the
  `disabledIndices` parameter, but `aria-disabled` elements only when `disabledIndices` is
  `undefined` — and tabs pass an array, so prop-disabled (aria-disabled) tabs are never skipped
  by keyboard navigation (`packages/react/src/floating-ui-react/utils/composite.ts:471-505`).

### Tab: reconciles composite highlight with root selection, and gates activation

- The tab reads three contexts — root, list, and composite
  (`packages/react/src/tabs/tab/TabsTab.tsx:42-53`) — and registers itself as a composite item
  via `useCompositeItem` (the hook, not the `CompositeItem` component, "because the index is
  needed for Tab internals" — `:63-64`). Its metadata `{disabled, id, value}`
  (`:57`) is what the root's `tabMap` scans (via `CompositeList`'s metadata plumbing) for all
  the value→element/id lookups described above.
- `active` is a pure derivation `value === activeTabValue` (`:69`).
- **Highlight↔selection reconciliation** (`useIsoLayoutEffect`, `:83-109`) keeps the composite's
  roving highlight pointed at the active tab when the value changes externally (controlled mode).
  Three gates give the behavior its documented subtleties:
  1. `isNavigatingRef` — set by `onKeyDownCapture` on the tab (`:200-202`) and consumed once
     here (`:84-88`) — suppresses the sync for the very next commit after a keyboard-driven
     composite move, so the composite's own highlight update is not immediately overwritten;
  2. focus-within guard — if the active element is inside the list (checked with the shadow-DOM
     safe `activeElement` + `contains` helpers, `:96-101`), the highlight stays on the focused
     tab while `aria-selected` moves elsewhere (Focus management section of behavior.md,
     "external value change while focus is inside the tablist");
  3. disabled guard — an active-but-disabled tab never takes the highlight (`:104-108`), so the
     previously highlighted tab keeps `tabindex="0"` (State model section of behavior.md).
- Button semantics are `useButton({disabled, native: nativeButton, focusableWhenDisabled: true})`
  (`:111-115`). `focusableWhenDisabled` is what keeps disabled tabs focusable and aria-disabled
  rather than natively disabled (Focus management section of behavior.md); inside `useButton`,
  that path also deletes the native `disabled` attribute from composite items
  (`packages/react/src/internals/use-button/useButton.ts:28-34`, `:72-89`). Enter/Space
  activation of the focused tab is `useButton`'s synthetic-click dispatch
  (`packages/react/src/internals/use-button/useButton.ts:116-176`) — the tab itself only ever
  activates through `onClick` (`:132-138`).
- **Activation gating**: both `onClick` and `onFocus` bail when `active || disabled`
  (`:133-135`, `:141-143`), which is why re-clicking/re-focusing the active tab never re-emits
  `onValueChange` (Events and Edge cases sections of behavior.md). `onFocus` additionally
  requires `activateOnFocus` from the list context and consults two press refs:
  `isPressingRef`/`isMainButtonRef` (`:145-151`). `onPointerDown` sets them and registers
  document-level `pointerup`/`pointercancel` listeners *for every press* — the comment at
  `:164-166` explains that a secondary-button press must still clear the pressing state so a
  later focus can activate. This is the entire mechanism behind the secondary-button suppression
  behavior (Focus management section of behavior.md); the refs are cleared on the document, not
  the tab, so the release can happen anywhere.
- Resize observation of the tab element is registered from the *ref callback*, not an effect, so
  the observer follows the rendered element when the `render` prop swaps the host element type
  (`:74-79`, merged into the ref array at `:188`) — the mechanism behind the "element type
  changes, keeps being observed" case in the DOM structure section of behavior.md.
- The rendered props (`:186-208`) compose, in order: composite props (which include the roving
  `tabIndex` and a focus handler that moves the highlight —
  `packages/react/src/internals/composite/item/useCompositeItem.ts:26-31`), the tab's own ARIA
  and handlers (`role: 'tab'`, `aria-controls` set only when a panel id is registered (`:117`),
  `aria-selected`, `id`), the `ACTIVE_COMPOSITE_ITEM` marker rendered as
  `data-composite-item-active` when active (`:199`; constant at
  `packages/react/src/internals/composite/constants.ts:1`), and `getButtonProps` last. The
  state object (`:179-184`) separately produces `data-active`/`data-disabled` through the
  generic boolean mapping (`packages/react/src/internals/getStateAttributesProps.ts:24-28`) —
  so an active tab carries **both** `data-active` (state attribute, consumed by the prehydration
  script) and `data-composite-item-active` (composite marker, consumed by the composite root to
  seed the default tab stop —
  `packages/react/src/internals/composite/root/useCompositeRoot.ts:139-150`).

### Panel: self-owned mount/transition lifecycle

- `open` is `value === selectedValue` (`packages/react/src/tabs/panel/TabsPanel.tsx:48`); the
  panel then runs the shared transition machine `useTransitionStatus(open)` (`:49`), whose
  `mounted` starts `false` for initially-closed panels and flips synchronously during the opening
  render while `transitionStatus` walks `starting → idle → ending`
  (`packages/react/src/internals/useTransitionStatus.ts:17-42`). `hidden = !mounted` (`:50`).
- Unmount-on-close is delegated to `useOpenChangeComplete` (`:82-90`), which waits for the
  element's CSS animations/transitions to finish (via `useAnimationsFinished`) before calling
  `setMounted(false)` (`packages/react/src/internals/useOpenChangeComplete.tsx:9-28`) — the
  mechanism behind "receives `data-ending-style` and is removed after the exit animation" (DOM
  structure section of behavior.md). The `starting`/`ending` data attributes come from
  `transitionStatusMapping` merged into the panel's mapping
  (`packages/react/src/tabs/panel/TabsPanel.tsx:19-22`;
  `packages/react/src/internals/stateAttributesMapping.ts:10-19`).
- The render gate is `keepMounted || mounted`, returning `null` otherwise
  (`:103-106`) — the unit's single conditional mount. Rendered props (`:63-80`) wire the ARIA
  pair (`aria-labelledby` from `getTabIdByPanelValue` — undefined while no matching tab is
  registered — plus `role: 'tabpanel'`, `id`, the `hidden` attribute, `tabIndex: open ? 0 : -1`,
  `inert` when closed via `inertValue`, and `data-index` from the composite-list item index).
- Panel registration into the root's id map happens in a layout effect gated on having an id and
  being visible-or-keepMounted (`:92-101`), with a React 17 `useId` caveat at `:93-95`. The
  registration is what flips `aria-controls` on/off on the tab as panels mount and unmount
  (Accessibility section of behavior.md). The panel also registers its DOM element into the
  root's own `CompositeList` (via `useCompositeListItem`, `:46`) purely to get a stable
  `index` for `data-index`; that list holds panel metadata but is otherwise inert.

### Indicator: a stateless machine measured during render

- The indicator owns no measurement state. It subscribes a `useForcedRerendering` callback into
  the list's resize fan-out (`packages/react/src/tabs/indicator/TabsIndicator.tsx:52-56`) and
  otherwise re-measures **during render** whenever anything makes it re-render (selection change,
  any list/tab resize). All measurement is a pure function of
  `(value, tabsListElement, DOM geometry)`.
- The measurement algorithm (`:58-120`) exists to produce six CSS variables
  (`packages/react/src/tabs/indicator/TabsIndicatorCssVars.ts:5-30`) that are transform- and
  scroll-correct:
  1. CSS dimensions via `getCssDimensions` for both tab and list
     (`packages/react/src/utils/getCssDimensions.ts:4-24`), and list scale inferred as
     rect-size / CSS-size (`:77-78`);
  2. a *layout offset* from the cumulative `offsetLeft/offsetTop` chain, which is immune to
     transforms but rounded (`:80-83`, helpers `getCumulativeOffset` `:230-249` and
     `getLayoutOffset` `:204-228`);
  3. a *rect offset* from `getBoundingClientRect`, which is sub-pixel-precise but warped by any
     ancestor transform; it is divided by the inferred scale and corrected by the list's scroll
     and `clientLeft/Top` (`:85-92`);
  4. an agreement check: the rect offset is adopted only when it matches the layout offset (after
     subtracting the tab's own translation, `getActiveTabTranslation` `:251-275`, which reads the
     computed `transform` matrix via `getElementTransform`
     (`packages/react/src/utils/getElementTransform.ts:10-33`) and the `translate` longhand with
     percentage resolution `:281-290`) within `MAX_LAYOUT_ROUNDING_ERROR` = 2px (`:27`,
     check at `:106-113`). A degenerate zero-scale list produces `NaN`/`Infinity`, which fails
     the comparison and leaves the layout offset in place — no separate guard needed
     (comment at `:94-100`). This dual-offset strategy is what makes all the ancestor-transform
     and scrolled-ancestor cases in the DOM structure section of behavior.md work with one code
     path.
  5. `getLayoutOffset` subtracts the scroll of every container between tab and list, using
     `getParentNode` so shadow roots are crossed (`:211-225`);
- Right/bottom offsets are derived from `scrollWidth/scrollHeight` (`:117-118`); the style object
  with the six vars is only produced when a tab is selected (`:126-135`); the element is
  `hidden` until the measured size is non-zero (`:137`, `:153`) — the
  `activeTabPosition: null / activeTabSize: null` state asserted in the Accessibility section of
  behavior.md — and `value == null` short-circuits to no render at all (`:163-165`), the
  "not rendered with `value={null}`" case. `role="presentation"` (`:151`) and
  `suppressHydrationWarning` (`:157`) round out the element. The indicator's state mapping
  suppresses `activeTabPosition`/`activeTabSize` from becoming data attributes
  (`:19-23`).

### Pre-hydration script (SSR)

- `renderBeforeHydration` renders a sibling `PrehydrationScript`
  (`packages/react/src/tabs/indicator/TabsIndicator.tsx:167-172`) whose body is imported through
  the `#prehydration/tabs/indicator` subpath (`:6`). That subpath resolves to the minified
  autogenerated script for SSR bundles and to an empty-string stub under the `browser` condition
  (`packages/react/package.json:112-115`; stub at
  `packages/react/src/internals/prehydrationScript.stub.ts:1-6`) — the script only ever executes
  from server HTML, so client bundles drop it. `PrehydrationScript` renders the inline
  `<script nonce={…}>` only while hydrating and uses `suppressHydrationWarning` so React keeps
  the already-executed server script element
  (`packages/react/src/internals/PrehydrationScript.tsx:32-49`) — the CSP-nonce and
  keep-then-remove behaviors in the DOM structure section of behavior.md.
- The script source is authored at
  `packages/react/src/tabs/indicator/prehydrationScript.template.js` and minified into
  `prehydrationScript.min.ts` by `pnpm inline-scripts`
  (`packages/react/src/tabs/indicator/prehydrationScript.min.ts:1-2`,
  `scripts/inlineScripts.mts:15-27`). It locates the indicator via
  `document.currentScript.previousElementSibling`, the list via `closest('[role="tablist"]')`,
  and the active tab via `[data-active]`
  (`packages/react/src/tabs/indicator/prehydrationScript.template.js:2-15`) — note it bails out
  entirely if the active tab carries any transform (`:17-29`), intentionally deferring to the
  hydrated component rather than painting at a wrong spot; it then computes the same
  layout-offset variables (clamped to the list's scroll extents, `:99-105`), un-hides the
  indicator when measurable, and keeps a `ResizeObserver` + 10s-timeout watchdog running only
  until the indicator is revealed, hydration wins, or the selection moves
  (`:121-159`). The template comments mark two "keep in sync with `TabsIndicator.tsx`"
  constraints (`:20`, `:60`) — the layout-offset math is deliberately duplicated.

## Context providers/consumers

Three Base-UI-owned contexts cross boundaries (plus the composite's own), each created with an
`undefined` default and guarded by an accessor that throws the `Base UI: …` errors asserted in
the Public API surface section of behavior.md:

1. **`TabsRootContext`** — Root → List, Tab, Panel, Indicator
   (`packages/react/src/tabs/root/TabsRootContext.ts:43-54`; provider at
   `packages/react/src/tabs/root/TabsRoot.tsx:345-349`). Crosses the boundary: `value`,
   `onValueChange`, `orientation`, `tabActivationDirection`, the three lookup/registration
   accessors, and `setTabMap`. Consumers use disjoint slices: List only
   `orientation`/`setTabMap`/`tabActivationDirection`
   (`packages/react/src/tabs/list/TabsList.tsx:32`); Tab `value`, `getTabPanelIdByValue`,
   `onValueChange`, `orientation`, `tabActivationDirection`
   (`packages/react/src/tabs/tab/TabsTab.tsx:42-48`); Panel `value`, `getTabIdByPanelValue`,
   `orientation`, `tabActivationDirection`, `registerMountedTabPanel`
   (`packages/react/src/tabs/panel/TabsPanel.tsx:36-42`); Indicator `value`,
   `getTabElementBySelectedValue`, `orientation`, `tabActivationDirection`
   (`packages/react/src/tabs/indicator/TabsIndicator.tsx:47-48`). Nested roots are isolated for
   free because each root provides a fresh memoized value (nesting independence in the Edge
   cases section of behavior.md).
2. **`TabsListContext`** — List → Tab, Indicator
   (`packages/react/src/tabs/list/TabsListContext.ts:11-21`; provider at
   `packages/react/src/tabs/list/TabsList.tsx:109-128`). Crosses the boundary:
   `activateOnFocus`, `tabsListElement` (the DOM node the Tab uses for its focus-within check
   and the Indicator uses as its measurement frame), and the two resize-registration functions.
   The tab placement error (`TabsTab` outside a List) is thrown by its accessor
   (`packages/react/src/tabs/list/TabsListContext.ts:13-21`).
3. **`CompositeRootContext`** (internal, provided by `CompositeRoot` inside the List) —
   → Tab. Carries `highlightedIndex`, `onHighlightedIndexChange`, `highlightItemOnHover`,
   `relayKeyboardEvent`
   (`packages/react/src/internals/composite/root/CompositeRoot.tsx:73-94`). The Tab is the only
   tabs part that consumes it (`packages/react/src/tabs/tab/TabsTab.tsx:53`) — the roving
   tabindex is entirely composite-owned.

Plus two `CompositeList` registration scopes with no tabs-specific code: tabs register into the
List's `CompositeRoot` (which forwards map changes to `TabsRoot.setTabMap` via `onMapChange`,
`packages/react/src/tabs/list/TabsList.tsx:124`), and panels register into a second, root-owned
`CompositeList` that exists only to number panels (`packages/react/src/tabs/root/TabsRoot.tsx:345-349`,
`packages/react/src/tabs/panel/TabsPanel.tsx:46`). Registration is by callback ref, indexes are
assigned by document position, and map changes flush synchronously before paint
(`packages/react/src/internals/composite/list/CompositeList.tsx:39-58`, `:142-170`,
`packages/react/src/internals/composite/list/useCompositeListItem.ts:62-96`) — this is what
makes StrictMode re-registration and transient duplicate values settle (Accessibility and Edge
cases sections of behavior.md).

## DOM/portal strategy and why

- **No portals anywhere in the unit.** Every part renders inline through `useRenderElement`
  (Root `div` `packages/react/src/tabs/root/TabsRoot.tsx:338-343`; List `div[role=tablist]` via
  `CompositeRoot` `packages/react/src/tabs/list/TabsList.tsx:109-128`; Tab `button[role=tab]`
  `packages/react/src/tabs/tab/TabsTab.tsx:186-208`; Panel `div[role=tabpanel]`
  `packages/react/src/tabs/panel/TabsPanel.tsx:63-80`; Indicator `span[role=presentation]`
  `packages/react/src/tabs/indicator/TabsIndicator.tsx:146-161`), matching the default-DOM
  assertions in the DOM structure section of behavior.md. Rationale: tabs are document-flow
  widgets whose parts must keep their authored relative positions — the indicator measures
  against the list's geometry (its offset math assumes it sits inside the list and scrolls with
  it, `packages/react/src/tabs/indicator/TabsIndicator.tsx:211-216`), the panel must sit at its
  authored position for `aria-controls` linking and CSS transitions, and composition with popup
  portals works precisely because tabs themselves bring no portal machinery (DOM structure
  section of behavior.md, popover/dialog composition).
- **The indicator positions itself with CSS variables, not JS-applied styles**: the component
  writes six `--active-tab-*` custom properties and leaves layout to the consumer's stylesheet,
  which is why the API is styleable (`render` prop, plain `span`) while remaining pixel-correct
  under transforms, scaling, and scrolling.
- **Selection state reaches the DOM as data attributes** through
  `stateAttributesMapping` on every part (`data-activation-direction`, `data-orientation`, and
  per-part `data-active`/`data-disabled`/`data-hidden`/`data-index`/transition attributes), with
  the root's mapping shared by all five parts
  (`packages/react/src/tabs/root/stateAttributesMapping.ts:5-9`). The `data-activation-direction`
  attribute asserted on the root *and* each tab (State model section of behavior.md) is one
  shared mapping consumed by both.
- **SSR is addressed only by the indicator's pre-hydration script** (section above); every other
  part is hydration-neutral (ids via `useBaseUiId` wrapping `useId`,
  `packages/react/src/internals/useBaseUiId.ts:9-11`).

## Dependencies on other Base UI internals

No `wraps-external:` field exists for this unit (`TODO.md:527-533`), so there is no external
package or replacement crate to name; the only third-party runtime dependency is
`@floating-ui/utils/dom` type guards. Direct dependencies, by origin:

`@base-ui/utils/*` (public shared utils package):

| Utility | Used by | Purpose |
|---|---|---|
| `useControlled` | Root (`packages/react/src/tabs/root/TabsRoot.tsx:3`) | controlled/uncontrolled `value` |
| `useStableCallback` | Root (`:5`), List (`packages/react/src/tabs/list/TabsList.tsx:3`), Tab (`packages/react/src/tabs/tab/TabsTab.tsx:5`) | stable callbacks in effects/handlers |
| `useIsoLayoutEffect` | Root (`:4`), List (`packages/react/src/tabs/list/TabsList.tsx:4`), Tab (`packages/react/src/tabs/tab/TabsTab.tsx:4`), Panel (`packages/react/src/tabs/panel/TabsPanel.tsx:4`) | pre-paint effects |
| `useForcedRerendering` | Indicator (`packages/react/src/tabs/indicator/TabsIndicator.tsx:4`) | re-measure trigger |
| `owner` (`ownerDocument`/`ownerWindow`) | Tab (`packages/react/src/tabs/tab/TabsTab.tsx:3`), Indicator (`packages/react/src/tabs/indicator/TabsIndicator.tsx:5`) | shadow/realm-safe document & window |
| `inertValue` | Panel (`packages/react/src/tabs/panel/TabsPanel.tsx:3`) | closed-panel inertness |
| `EMPTY_ARRAY` | List (`packages/react/src/tabs/list/TabsList.tsx:5`) | stable `disabledIndices` |

`internals/` (private to `packages/react`):

| Internal | Used by | Purpose |
|---|---|---|
| `useRenderElement` | all five parts | element rendering, ref/props merging, render prop, state→data-attribute mapping |
| `internals/composite` — `CompositeRoot`/`useCompositeRoot` | List | keyboard navigation (arrows/Home/End, `loopFocus`, RTL swap, modifier rejection), roving tab stop, scroll-into-view, default tab stop from `ACTIVE_COMPOSITE_ITEM` |
| `internals/composite` — `useCompositeItem`/`useCompositeRootContext` | Tab | per-tab registration, roving `tabIndex`, focus→highlight |
| `internals/composite` — `CompositeList`/`useCompositeListItem` | Root (panels), Panel | index assignment for panels; registration map for tabs (via `onMapChange` → `setTabMap`) |
| `internals/composite` — `composite.ts` helpers | List (via `CompositeRoot`) | key sets, disabled/visible index math (`isListIndexDisabled` etc. re-exported from `floating-ui-react/utils`) |
| `useButton` (+ `useFocusableWhenDisabled`, `dispatchClickWithModifiers`) | Tab | native/non-native button semantics, Enter/Space activation, focusable-when-disabled |
| `useTransitionStatus` | Panel | `mounted`/`starting`/`idle`/`ending` machine |
| `useOpenChangeComplete` (+ `useAnimationsFinished`) | Panel | unmount after exit animation |
| `createBaseUIEventDetails` / `REASONS` | Root (`packages/react/src/tabs/root/TabsRoot.tsx:14-18`), Tab (`packages/react/src/tabs/tab/TabsTab.tsx:17-18`) | `none`/`initial`/`disabled`/`missing` reasons, cancel protocol, synthetic event |
| `useBaseUiId` | Tab (`packages/react/src/tabs/tab/TabsTab.tsx:6,55`), Panel (`packages/react/src/tabs/panel/TabsPanel.tsx:5,44`) | `base-ui-` prefixed stable ids |
| `PrehydrationScript` + `csp-context` + `utils/useIsHydrating` | Indicator (`packages/react/src/tabs/indicator/TabsIndicator.tsx:7`) | inline SSR script with nonce, hydration-pass-only rendering |
| `getStateAttributesProps` / `stateAttributesMapping` (`transitionStatusMapping`) | all parts (mapping composition at `packages/react/src/tabs/panel/TabsPanel.tsx:19-22`) | boolean/string state → `data-*`, `data-starting-style`/`data-ending-style` |
| `DirectionContext` (`useDirection`) | List (via `CompositeRoot`) | RTL arrow-key swap |
| `types` (`BaseUIComponentProps`, `NativeButtonProps`, `Orientation`) | all parts (types) | shared prop contracts |

`floating-ui-react/utils` (in-package, not the npm package): `activeElement`, `contains`,
`getTarget` — Tab's focus-within guard (`packages/react/src/tabs/tab/TabsTab.tsx:19`) and
composite internals; `isListIndexDisabled`/`findNonDisabledListIndex`/`isElementVisible`
(`packages/react/src/floating-ui-react/utils/composite.ts:471-505`) govern which tabs the
keyboard can land on.

In-package shared utils (`packages/react/src/utils/`): `getCssDimensions`
(`packages/react/src/tabs/indicator/TabsIndicator.tsx:9`) and `getElementTransform`
(`:10`) — indicator measurement.

Package-private build inputs: the `#prehydration/tabs/indicator` import map
(`packages/react/package.json:112-115`) and the `pnpm inline-scripts` minifier
(`scripts/inlineScripts.mts:15-27`).

Notably **not** depended on: `floating-ui-react`'s positioning machinery (`useAnchorPositioning`,
portals), `use-render` (rendering goes through `internals/useRenderElement`), and any DOM-portal
infrastructure. The `@floating-ui/utils/dom` imports in the Indicator
(`getParentNode`, `isHTMLElement`, `isLastTraversableNode`,
`packages/react/src/tabs/indicator/TabsIndicator.tsx:3`) are pure DOM helpers, not positioning.

## Anything in source not explained by any test

Explicit gaps, for the golden-fixture stage and the backward-looking audit:

1. **`data-orientation` is emitted by every part but never asserted.** `orientation` is a truthy
   string in every part's state, so the generic mapping stamps `data-orientation="horizontal"`
   (or `"vertical"`) on root/list/tab/panel/indicator
   (`packages/react/src/internals/getStateAttributesProps.ts:24-28`); behavior.md only covers
   `aria-orientation` on the list. The `TabsRootDataAttributes`/`TabsListDataAttributes`/
   `TabsTabDataAttributes`/`TabsPanelDataAttributes`/`TabsIndicatorDataAttributes` files are
   documentation constants only — actual emission is the generic state mapping, and
   `data-orientation` has no test coverage in any unit file.
2. **`data-hidden` on panels is never asserted.** `TabsPanelDataAttributes.hidden`
   (`packages/react/src/tabs/panel/TabsPanelDataAttributes.ts:20`) is emitted when the
   `hidden` state is true, but tests only assert the `hidden` *attribute*
   (Accessibility section of behavior.md).
3. **`data-active` and `data-composite-item-active` on tabs are never asserted at the component
   level.** `data-active` comes from the state mapping (`active: true`), and
   `data-composite-item-active` from `ACTIVE_COMPOSITE_ITEM`
   (`packages/react/src/tabs/tab/TabsTab.tsx:199`). The prehydration script tests execute the
   script against *hand-built* markup containing `data-active`
   (`packages/react/src/tabs/indicator/prehydrationScript.template.js:12`), so the contract "the
   real tab emits `data-active`" is load-bearing for SSR but untested end-to-end; the composite
   marker is only exercised indirectly via roving-tabindex tests.
4. **Panel `tabIndex` and `inert` are untested.** `tabIndex: open ? 0 : -1` and
   `inert: inertValue(!open)` (`packages/react/src/tabs/panel/TabsPanel.tsx:72-73`) are not
   asserted anywhere in behavior.md.
5. **The Suspense guard in the empty-map branch is untested for uncontrolled roots.** The
   `lastKnownTabElementRef.isConnected` check that prevents a spurious `missing` emission when an
   outer Suspense boundary tears down layout effects
   (`packages/react/src/tabs/root/TabsRoot.tsx:264-275`) is motivated by a Suspense scenario,
   but behavior.md's only Suspense test is a panel suspending when *opened*
   (Edge cases section) — the uncontrolled-root Suspense teardown path itself has no test.
6. **Keyboard navigation scroll-into-view is untested.** `CompositeRoot` passes
   `shouldScrollIntoView = true` on every highlight change and calls
   `scrollIntoViewIfNeeded` (`packages/react/src/internals/composite/root/useCompositeRoot.ts:101-108`,
   `:310`); no tabs test asserts scrolling. The only trace in the test suite is the Safari
   `scrollLeft` skip in the root test file (Edge cases section of behavior.md).
7. **`stopEventPropagation` (composite default `true`) is untested here.** Arrow-key handling
   stops propagation (`packages/react/src/internals/composite/root/useCompositeRoot.ts:302-306`);
   behavior.md marks bubbling as N/A/UNVERIFIED (Events section), so the fixture stage must not
   assume keyboard events bubble out of the list.
8. **Throwing `onValueChange` during the initial notification leaves the notify flag set.**
   `commitAutomaticValueChange` marks `shouldNotifyInitialValueChangeRef.current = false` only
   after the consumer callback returns (`packages/react/src/tabs/root/TabsRoot.tsx:258-261`); the
   comment frames this as intentional (queued first so consistency updates aren't cancelable
   through a throwing handler), but no test covers a throwing handler re-notifying on the next
   pass.
9. **Activation direction for incomparable value types without DOM registration is `'none'` by
   inference, untested.** The value-based fallback only applies to same-typed numbers/strings
   (`packages/react/src/tabs/root/TabsRoot.tsx:383-394`); behavior.md proves the numeric and
   string cases (State model section, `:2241-2337`) but an object-valued tab added and selected
   in one update is untested (consistent with its UNVERIFIED implicit-default note).
10. **The prehydration script's clamp is a deliberate divergence from the component.** The script
    clamps `left`/`top` to the list's scroll extents
    (`packages/react/src/tabs/indicator/prehydrationScript.template.js:99-105`) while
    `TabsIndicator.tsx` does not (`packages/react/src/tabs/indicator/TabsIndicator.tsx:82-83`);
    only template comments explain this, and no test pins the clamped behavior (the streaming
    tests in the DOM structure section cover reveal/stop conditions, not overflow clamping).
11. **The indicator's 2px agreement window is untested at its boundary.**
    `MAX_LAYOUT_ROUNDING_ERROR` (`packages/react/src/tabs/indicator/TabsIndicator.tsx:27`)
    decides layout-vs-rect adoption; the transform tests in behavior.md pin the outcomes on both
    sides of the boundary but no test targets the 1-2px band itself.
