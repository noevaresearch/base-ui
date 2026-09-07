# floating-ui-react — safePolygon + utils (behavior spec)

Scope: wrapper-level behavior only, mined from the unit's own tests. Per the unit's TODO (`wraps-external: @floating-ui/react-dom, @floating-ui/utils`; Rust-equivalent crate `floating-ui-leptos`), algorithms owned by those third-party packages are NOT specified here and are to be bound in Stage 3 instead of reimplemented. None of the tests in this batch directly exercise an `@floating-ui/*` public API — everything below is Base UI wrapper-level behavior. UNVERIFIED — inferred from `packages/react/src/floating-ui-react/TODO.md`, no test asserts the delegation split.

## Public API surface (props, parts, subcomponents)

### safePolygon

- `safePolygon()` is called with no arguments in every test and returns a handler factory. The factory is invoked with a "handle close context" and returns a mouse-move handler function that is then called with a `MouseEvent`. Factory invocation: `safePolygon()(context)` at `packages/react/src/floating-ui-react/safePolygon.test.ts:186`, `:243`, `:275`, `:304`, and re-invocation of one factory at `:331-336`. No option object is ever passed, so no option semantics are proven by tests.
- The context shape exercised by tests is `{ x, y, placement, elements: { domReference, floating }, nodeId, onClose, tree }` — built by the test helper at `packages/react/src/floating-ui-react/safePolygon.test.ts:56-64`. `x`/`y` default to `2`/`0` and `placement` defaults to `'right'` in the helper (`:44-46`).
- `tree` is a `FloatingTreeStore`; nodes are registered with `tree.addNode({ id, parentId, context })` where `context` is optional (`packages/react/src/floating-ui-react/safePolygon.test.ts:184`, `:212-213`, `:240-241` — note `inline-root` added with no `context` at `:212`).
- The returned handler takes a single `MouseEvent`-shaped argument. Tests construct it as `{ type: 'mousemove', clientX, clientY, relatedTarget: null, composedPath: () => [target] }` (`packages/react/src/floating-ui-react/safePolygon.test.ts:25-37`), i.e. the handler reads `clientX`/`clientY` for geometry and uses `composedPath()`/`relatedTarget` to identify the event target (a specific target is passed at `:333`).
- `onClose` is the observable side effect: the handler calls it (via the context) when it decides the popup should close.

### utils/composite

- `isHiddenByStyles(style: CSSStyleDeclaration): boolean` — imported from `./composite` at `packages/react/src/floating-ui-react/utils/composite.test.ts:2`; takes a computed style and returns a boolean (`:18-20`).
- `isElementVisible(element): boolean` — takes an element, returns a boolean (`packages/react/src/floating-ui-react/utils/composite.test.ts:42-45`).
- `isListIndexDisabled(list, index, disabledIndices?): boolean` — `list` is an array of elements, `index` a number; the optional `disabledIndices` third argument accepts either an array (`isListIndexDisabled(list, 0, [])`, `:62`) or a predicate function (`isListIndexDisabled(list, 0, () => false)`, `:65`). Called without the third argument at `:57-58`.

### utils/markOthers

- `markOthers(targets, options?)` — `targets` is an array of "keep" elements that must NOT be marked; all other elements in the document get marked. Returns a cleanup function `() => void` (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:14-20`).
- Options object keys exercised: `ariaHidden?: boolean` (`:14`, `:183`), `inert?: boolean` (`:138`, `:182`), `mark?: boolean` (`:215-216` with `mark: false`, `:247` with `mark: true`). Called with no options at all at `:102` and `:144`.

### utils/nodes

- `getNodeChildren(nodes, id, onlyOpenChildren)` — takes a flat node array, a root id string, and a boolean; returns a flat array of node objects (`packages/react/src/floating-ui-react/utils/nodes.test.ts:8-26`).
- `getNodeAncestors(nodes, id)` — takes a flat node array and an id; returns a flat array of ancestor nodes (`packages/react/src/floating-ui-react/utils/nodes.test.ts:101-115`).
- Node shape exercised: `{ id: string, parentId: string | null, context?: FloatingContext }`; `context` is optional (contextless nodes at `packages/react/src/floating-ui-react/utils/nodes.test.ts:71` and `:105-107`).

### utils/tabbable

- `isTabbable(element): boolean` — predicate (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:119`, `:129`).
- `tabbable(container): Element[]` — returns the container's tabbable elements in DOM order (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:18`, ordering asserted at `:104-109`).

## State model (controlled/uncontrolled, defaults, transitions)

### safePolygon

- The factory holds per-invocation traversal state: creating a second handler from the same `safePolygon()` factory resets traversal state, so the second handler does not inherit the first handler's movement history (first handler moved onto the floating element at (130, 50); second handler moved straight to the trough point and `onClose` was not called) — `packages/react/src/floating-ui-react/safePolygon.test.ts:311-339`, assertion `:338`.
- There is an "intent timeout" under fake timers: after a mouse move off the reference, advancing timers by 50ms is sufficient for `onClose` to have been called exactly once when no nested child is open (`packages/react/src/floating-ui-react/safePolygon.test.ts:246-250`). The trough/away-from-corridor checks are synchronous: no timer advancement occurs before the assertions at `:276-278` and `:305-307`.
- No other state model applies; there are no controlled/uncontrolled props in this batch.

### utils/markOthers

- Cleanup bookkeeping is stateful across concurrent `markOthers` calls: each call tracks which attributes it owns and cleanup is reference-counted per control attribute (see Accessibility and Edge cases). Demonstrated by the mixed-attribute test where three concurrent calls each own a different attribute subset and cleanups only remove what their call is responsible for (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:116-167`).

### composite / nodes / tabbable

- Stateless pure functions: given the same DOM/array input they return the same result; no state, defaults, or transitions are exercised (`packages/react/src/floating-ui-react/utils/composite.test.ts:8-67`, `packages/react/src/floating-ui-react/utils/nodes.test.ts:8-115`, `packages/react/src/floating-ui-react/utils/tabbable.test.ts:10-379`).

## Keyboard interactions

N/A — no keyboard behavior is exercised by any test in this batch. The closest related surface is focusability classification (see Focus management), which is about what *can* receive focus, not key handling.

## Focus management

### utils/tabbable — focusability rules (what `tabbable()` includes / `isTabbable()` returns true for)

Included in the tab order:

- `button` and default `input` elements (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:10-19`).
- `iframe` (embedded focusable element) — `packages/react/src/floating-ui-react/utils/tabbable.test.ts:21-27`.
- Light-DOM elements slotted into a shadow root's `<slot>` (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:39-50`).
- The `<summary>` of a closed `<details>` (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:68-80`) and a `<details>` without any `<summary>` (the details element itself is tabbable) (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:82-110`, `summarylessDetails` in the expected array at `:104-109`).
- Elements with `aria-disabled="true"` remain tabbable (given `tabIndex = 0`) — `packages/react/src/floating-ui-react/utils/tabbable.test.ts:112-121`.
- Descendants of `display: contents` ancestors stay tabbable, even when the ancestor's `checkVisibility()` reports false (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:133-150`).
- Descendants that override an ancestor's `visibility: hidden` with `visibility: visible` stay tabbable (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:183-194`).
- Zero-size elements (0 width/height/padding/border, `tabIndex = 0`) are tabbable — Chromium-only test (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:293-305`).
- Elements styled with the `visuallyHidden` util style object remain tabbable (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:307-315`), as do checkbox inputs styled with `visuallyHiddenInput` (`:317-326`).
- In a named radio group, only the checked radio is tabbable (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:328-346`); when no radio in the group is checked, only the first is tabbable (`:348-363`).
- Chromium-only nuances: a `display: contents` ancestor with `tabIndex = 0` is itself NOT tabbable while its descendants are (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:152-168`); descendants of `display: contents` + `content-visibility: hidden` ancestors stay tabbable (`:251-265`); descendants of `display: inline` + `content-visibility: hidden` ancestors stay tabbable (`:267-281`); `content-visibility: hidden` candidates themselves stay tabbable (`:283-291`).

Excluded from the tab order:

- `input[type=hidden]` (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:10-19`).
- Natively disabled controls (`disabledButton.disabled = true`) (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:29-37`).
- Unslotted light-DOM children of a shadow host (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:52-66`).
- Non-summary content inside a closed `<details>` (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:68-80`); in an open `<details>`, only the first `<summary>` is tabbable — a second `<summary>` is excluded while later visible content is included (`:82-110`, `ignoredSummary` absent from the expected array `:104-109`).
- Elements hidden with CSS `visibility: hidden` (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:123-131`).
- Descendants of hidden `display: contents` ancestors (ancestor also `visibility: hidden`) (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:170-181`).
- `display: contents` candidates themselves, when `checkVisibility()` reports them hidden (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:196-208`) or when `checkVisibility` is unavailable (`:210-222`).
- Descendants of `display: none` ancestors (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:224-234`).
- Descendants of block-level `content-visibility: hidden` ancestors — Chromium-only (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:236-249`).
- Slotted light-DOM elements that fall inside `inert` shadow content (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:365-379`).

### utils/composite — list-index disabled semantics (focus routing helper)

- An empty `disabledIndices` array (or a `() => false` predicate) marks every item as enabled — except natively disabled elements, which stay disabled "because natively disabled elements can never receive focus" (in-test comment) — `packages/react/src/floating-ui-react/utils/composite.test.ts:60-66`.
- `aria-disabled="true"` elements are reported disabled only when no `disabledIndices` argument is provided; they are reported enabled when an empty array or false-returning predicate is passed (`packages/react/src/floating-ui-react/utils/composite.test.ts:57-58`, `:62-63`, `:65-66`).

## Accessibility (roles, aria-*, id linking)

### utils/markOthers — aria-hidden / inert marking of "other" elements

- With `{ ariaHidden: true }`, every element outside the keep-list gets `aria-hidden="true"`; `cleanup()` removes it (attribute becomes `null`) — `packages/react/src/floating-ui-react/utils/markOthers.test.ts:14-21`.
- By default (no options, or with a control attribute), the marker attribute `data-base-ui-inert` (empty value) is also applied: `{ ariaHidden: true }` sets both `aria-hidden` and `data-base-ui-inert` on others (`:132-136`); `{ inert: true }` sets both `inert` and `data-base-ui-inert` (`:138-142`); no options sets only `data-base-ui-inert` (`:102-105`, `:144-148`).
- `mark: false` suppresses the `data-base-ui-inert` marker for that call: `{ mark: true }` alone yields `data-base-ui-inert` on others, while `{ ariaHidden: true, mark: false }` yields only `aria-hidden` (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:247-251`).
- Cleanup is scoped per control attribute and reference-counted: when `ariaHidden`, `inert`, and no-option calls overlap, cleaning one call removes only its own attribute (`inert` removal at `:156-160`) and the last cleanup removes all remaining attributes (`:162-166`).
- Externally owned attributes are preserved: an element that already has `inert` keeps it after an inert-marking call's cleanup, and an already-`aria-hidden` element keeps `aria-hidden="true"` even after the aria-hidden call's cleanup (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:169-203`, `:205-236`).
- No `role`, `aria-*` id-linking, or live-region behavior is exercised in this batch.

### utils/composite — aria-disabled

- `aria-disabled="true"` is honored as "disabled" by `isListIndexDisabled` only in the absence of an explicit `disabledIndices` argument, unlike native `disabled` (`packages/react/src/floating-ui-react/utils/composite.test.ts:48-66`).

### utils/tabbable — aria-disabled

- `aria-disabled="true"` does not remove an element from the tab order (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:112-121`).

## DOM structure & portal behavior

### utils/markOthers — document + shadow DOM scope

- Marking applies to elements elsewhere in `document.body` relative to the keep-list (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:8-21`).
- Shadow DOM: when the keep-target lives inside a shadow root (inside an `<a>` anchor), `markOthers` does not recurse infinitely and does not mark the shadow host, because the host contains the target (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:270-288`, assertion `:285`).
- The shadow host is treated as the avoid/keep element when the parent chain includes the anchor: elements outside the host get `aria-hidden="true"` while the host does not, and cleanup restores the outside element (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:290-313`, assertions `:307-308`, `:312`).
- No portal-specific behavior is tested in this batch.

### utils/tabbable — shadow DOM and details structure

- Slotted light-DOM content is reachable from `tabbable(document.body)`; unslotted light-DOM content of a shadow host is not (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:39-50`, `:52-66`).
- Elements inside `inert` subtrees in the shadow root make their slotted light-DOM content untabbable (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:365-379`).
- `<details>`/`<summary>` semantics follow native rendering rules (closed-content exclusion, first-summary-only) as listed under Focus management (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:68-110`).

### utils/nodes — logical (non-DOM) floating tree

- The tree is a flat array of `{ id, parentId, context? }` records, not a DOM structure; traversal is purely by `parentId` linkage (`packages/react/src/floating-ui-react/utils/nodes.test.ts:8-26`, `:101-115`).

## Events (names, payload shape, bubbling, preventDefault semantics)

### safePolygon — mousemove handling

- The only event consumed is a synthetic `mousemove`-shaped object with `clientX`/`clientY` coordinates, `relatedTarget: null`, and `composedPath()` returning `[target]` (`packages/react/src/floating-ui-react/safePolygon.test.ts:25-37`). The target can be the floating element (`:333`).
- No bubbling, cancellation, or `preventDefault` semantics are asserted anywhere in this batch — the handler's sole observable output is the `onClose` callback being called or not.

### Other units

- composite, markOthers, nodes, tabbable: N/A — no event handling is exercised.

## Edge cases (rapid interactions, unmount, nesting)

### safePolygon

- Nesting — open child prevents close: with an open (`open: true`) child node registered in the tree, a mousemove outside the corridor does not close; `onClose` is not called even after 50ms of timer advancement (`packages/react/src/floating-ui-react/safePolygon.test.ts:168-194`).
- Nesting through contextless intermediaries: an open child behind an intermediary node with no `context` still prevents close (`packages/react/src/floating-ui-react/safePolygon.test.ts:196-223`).
- Closed child does not prevent close: with the only child `open: false`, `onClose` fires once after the 50ms intent window (`packages/react/src/floating-ui-react/safePolygon.test.ts:225-251`).
- Rapid re-invocation: obtaining a fresh handler from the same factory resets traversal state, so stale movement from a previous handler cannot suppress or trigger a close on the new handler (`packages/react/src/floating-ui-react/safePolygon.test.ts:311-339`).
- Geometry semantics (per placement, `packages/react/src/floating-ui-react/safePolygon.test.ts:112-157`): with a 100×100 reference at (0,0) and a 100×100 floating element offset 120px along the placement axis, the "leave point" is the edge midpoint of the reference facing the floating element, the "trough point" is 10px beyond that edge toward the floating element, and the "outside point" is far away on the opposite side (e.g. for `'right'`: leave (100, 50), trough (110, 50), outside (-50, 50)). Moving through the trough keeps the popup open on all four placements (`:253-280`); moving to the outside point closes on all four placements (`:282-309`).

### utils/markOthers

- Unmount before cleanup: removing a marked/keep element from the DOM before calling its cleanup does not break cleanup — earlier keep targets get their marks restored/removed correctly (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:41-46`).
- Out-of-order cleanup: with two overlapping calls, cleaning the first call does not remove marks still owned by the second (`target` keeps `aria-hidden` and `other` stays marked after the first cleanup), and the final cleanup clears everything (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:55-85`).
- Concurrent differing control attributes: an aria-hidden call's target is not marked `aria-hidden` by a later call but is marked `data-base-ui-inert` by an optionless call; each cleanup removes only its own attribute (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:87-114`).
- Mixed attribute overlap (aria-hidden / inert / none): three concurrent calls stack attributes; cleanups peel off one attribute at a time in reverse order of ownership (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:116-167`).
- External ownership tracking per attribute: pre-existing `inert`/`aria-hidden` attributes survive cleanups of calls that would otherwise remove them (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:169-203`, `:205-236`).
- Mark-only overlap: a `{ mark: true }` call's cleanup does not disturb a concurrent control-attribute call's bookkeeping (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:238-268`).
- Shadow DOM recursion: a target nested inside a shadow root (within an anchor) does not cause infinite recursion (`packages/react/src/floating-ui-react/utils/markOthers.test.ts:270-288`).

### utils/nodes

- Deep trees: `getNodeChildren` returns flat, depth-first-ordered descendant arrays, not just direct children (`packages/react/src/floating-ui-react/utils/nodes.test.ts:49-66`, `:80-99`).
- Contextless intermediary nodes: skipped as results themselves, but their open descendants are still included when `onlyOpenChildren=true` (`packages/react/src/floating-ui-react/utils/nodes.test.ts:68-78`).
- `onlyOpenChildren=true` filters on each node's OWN `context.open` (closed nodes excluded, `:16` and `:21-25`), but does NOT prune descendants of closed intermediaries: an open node whose parent is closed is still returned (`packages/react/src/floating-ui-react/utils/nodes.test.ts:49-66`, node `'3'` with closed parent `'2'` included in the result `:60-65`).
- `getNodeAncestors` excludes the queried node itself and returns ancestors ordered nearest-parent-first up to the root (`packages/react/src/floating-ui-react/utils/nodes.test.ts:101-115`).

### utils/tabbable

- Visibility edge cases around `display: contents`, `content-visibility`, ancestor visibility overrides, and zero-size elements are enumerated under Focus management (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:133-305`).
- Radio-group edge cases (checked vs unchecked groups) are under Focus management (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:328-363`).

### utils/composite

- Missing `checkVisibility` API (undefined) forces CSS-style fallbacks: `display: none` → hidden, `display: contents` → hidden, `content-visibility: hidden` → treated as visible (`packages/react/src/floating-ui-react/utils/composite.test.ts:23-46`).
- `visibility: collapse` is treated the same as `visibility: hidden` (`packages/react/src/floating-ui-react/utils/composite.test.ts:13-19`).

## Shared harness dependencies

- `#test-utils` → `packages/react/test/index.ts` (re-exports `@base-ui/utils/testUtils`): imported by `tabbable.test.ts` solely for the `isJSDOM` boolean environment flag, used with `it.skipIf(isJSDOM)` to restrict layout-dependent tests (content-visibility, zero-size, display:contents) to the Chromium env (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:2`, skipIf usages at `:152`, `:236`, `:251`, `:267`, `:283`, `:293`).
- `@mui/internal-test-utils` (external test package): imported by `safePolygon.test.ts` for `act`, which wraps fake-timer advancement (`vi.advanceTimersByTime(50)`) so React-side effects flush (`packages/react/src/floating-ui-react/safePolygon.test.ts:2`, usages at `:189-191`, `:218-220`, `:246-248`).
- No other harness file is imported by this batch; composite, markOthers, and nodes tests import only Vitest plus their unit under test (`packages/react/src/floating-ui-react/utils/composite.test.ts:1-2`, `packages/react/src/floating-ui-react/utils/markOthers.test.ts:1-2`, `packages/react/src/floating-ui-react/utils/nodes.test.ts:1-3`).
