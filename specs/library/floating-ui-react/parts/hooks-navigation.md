# floating-ui-react — list-navigation & typeahead hooks (behavior spec)

Behavior-only spec mined from the unit tests of `useListNavigation` (incl. its WebKit-specific variant and grid navigation) and `useTypeahead` in `packages/react/src/floating-ui-react/hooks/`. Everything below is what the tests prove; delegation notes: the underlying positioning/behavior engine is the wrapped external library (`@floating-ui/react-dom`, `@floating-ui/utils`) — Rust equivalent crate: `floating-ui-leptos` (https://floating-ui.rustforweb.org/frameworks/leptos.html), to be bound in Stage 3 rather than reimplemented.

## Public API surface (props, parts, subcomponents)

### useListNavigation

- Hook wiring: `useListNavigation(context, options)` is composed with interaction hooks (e.g. `useClick`) and surfaced through a props-merging helper that yields `getReferenceProps`, `getFloatingProps`, and `getItemProps` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:34-45`).
- `listRef: Array<HTMLElement | null>` — consumer-owned mutable array; items register nodes via ref callbacks passed to `getItemProps` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:74-78`). For virtualized grids the array length can exceed the rendered DOM (set to the logical total via `listRef.current.length = totalItems`, `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:129-131`).
- `activeIndex: number | null` — consumer state passed in; the fixture keeps it in `useState` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:29`).
- `onNavigate(index)` — callback invoked with the new active index (see State model for `null` cases); test wiring wraps it as `onNavigate(index) { setActiveIndex(index); props.onNavigate?.(index, undefined); }` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:40-43`).
- `loopFocus` — enables wrap-around at both ends (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:440-492`).
- `orientation` — `'horizontal'` maps navigation to ArrowRight/ArrowLeft (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:494-546`); `'both'` is exercised by the grid fixtures (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1003-1026`).
- `rtl` — flips horizontal arrow mapping when combined with `orientation="horizontal"` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:548-600`).
- `focusItemOnOpen` — `true` focuses the first item when opened by click; `false` opts out (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:602-618`).
- `selectedIndex` — separately from `activeIndex`; on open the selected item is scrolled into view (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:620-642`).
- `allowEscape` — with `virtual`, moving past the first/last item navigates "out of the list" to `null`, overriding `loopFocus` at boundaries (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:644-680`).
- `virtual` — active item is tracked by index only; no DOM focus is moved (fixtures expose it via state attributes, `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:117-127`, `:174`).
- `openOnArrowKeyDown` — `true` (default) makes ArrowDown/ArrowUp on the closed reference open the popup; `false` disables opening via arrows (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:682-708`).
- `disabledIndices: number[]` — array form is exercised: listed indices are skipped in navigation (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:710-722`, `:1136-1154`). A function form is typed/rendered by the fixture (`aria-disabled` computed from `typeof props.disabledIndices === 'function'`, `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:70-72`) but no test asserts the hook skipping function-provided indices — UNVERIFIED — inferred from `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:70-72`, no test asserts this.
- `focusItemOnHover` — hover moves the active item and DOM focus; `false` disables (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:724-874`).
- `grid` — accepts a grid-navigation module: tests pass `gridNavigation` imported from `./gridNavigation` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:8`, `:125`); fixtures pass column-parameterized factories `gridNavigationWithColumns(5|7|3)` (`packages/react/test/floating-ui-tests/Grid.tsx:12-14`, `packages/react/test/floating-ui-tests/ComplexGrid.tsx:12-14`, `packages/react/test/floating-ui-tests/EmojiPicker.tsx:21-23`).
- Disabled/hidden item inputs the hook reacts to: `aria-disabled` attribute on the item element (fixture-rendered, `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:68-73`) and CSS hiding via `display: none` or `visibility: hidden` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:288-316`).

#### WebKit variant

- Same hook/API; the WebKit-specific behavior is activated by mocking `@base-ui/utils/platform` so `platform.engine.webkit === true` (`packages/react/src/floating-ui-react/hooks/useListNavigation.webkit.test.tsx:7-18`). No separate API surface is exercised.

### useTypeahead

- Hook wiring: `useTypeahead(context, options)` composed with optional `useClick` (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:30-43`).
- `listRef` — here populated with plain strings (labels), not elements: `React.useRef(props.list ?? ['one','two','three'])` (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:29`).
- `activeIndex: number | null` — consumer state (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:24`).
- `onMatch(index)` — invoked with the matched list index; consumer syncs `activeIndex` through it (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:33-36`).
- `onTyping(typing: boolean)` — invoked with typing-state changes (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:37`, `:292-305`).
- `elementsRef` — optional; when provided, the hook can consult real item elements to skip CSS-hidden items (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:89-98`, `:307-348`).

## State model (controlled/uncontrolled, defaults, transitions)

### useListNavigation

- `activeIndex` is fully consumer-controlled: the hook never holds it; it calls `onNavigate` and the consumer sets state (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:29`, `:40-43`). Initial state in fixtures is `null` (`:29`) but non-null starting indices are supported and act as the navigation origin (virtual grid starts at `initialActiveIndex` 4/93 etc., `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1064-1098`).
- Transitions to `null` (observed via `onNavigate`/state):
  - `allowEscape` + `virtual`: navigating up from the first or down from the last item sets the index to `null` — items deselect and `onNavigate` is called with `null` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:645-660`, `:671-679`).
  - Pointer leaving the active item (default hover behavior) clears it: `aria-selected` becomes `false` and `onNavigate` is last called with `null` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:769-772`, `:869-872`).
- Internal index reset on close: with `virtual`, navigating to index 2, closing via Escape, then reopening and pressing ArrowDown once lands on index `0` (not 3) — the pointer into the list is reset on close, and the displayed `activeIndex` state is `null` after close (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:318-438`, key assertions `:425-437`).
- Defaults proven with no-props `<App />`: ArrowDown/ArrowUp open the popup (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:191-209`, so `openOnArrowKeyDown` defaults to true); navigation clamps at the ends without `loopFocus` (`:230-234`, `:256-260`); with `loopFocus`, wrap-around happens instead (`:441-491`), so `allowEscape` defaults to off (loop wins at boundaries without it, `:441-491` vs `:645-660`).
- `focusItemOnOpen` default value is not directly asserted: tests pass it explicitly `true` (`:603-609`) or `false` (`:611-617`) — UNVERIFIED — inferred from `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:602-618`, no test asserts this for the implicit default.
- `focusItemOnHover` defaults to on: with default props a mouse-move over an item focuses it and fires `onNavigate` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:780-790`); explicit `focusItemOnHover={false}` disables it (`:804-812`).
- `orientation` default is vertical (ArrowUp/Down navigate with no props, `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:211-261`); whether ArrowLeft/Right are no-ops in the default vertical mode is not asserted — UNVERIFIED — inferred from `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:211-261`, no test asserts this.
- `selectedIndex` is independent consumer state; it drives a scroll-into-view on open, not `activeIndex` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:621-641`), and a hover can sync `activeIndex` from a matching `selectedIndex` when `activeIndex` is `null` (`:792-802`).
- `virtual` mode transitions are observable purely through state (fixtures mirror `activeIndex` to `data-active-index` / `aria-selected` / `data-active`), e.g. `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:174`, `:1056-1061`.

### useTypeahead

- `activeIndex` is consumer-controlled, updated via `onMatch` (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:33-36`, `:94-97`).
- Typing state: `onTyping(true)` fires on the first keystroke and `onTyping(false)` after 750 ms of inactivity (tests use fake timers advanced by exactly 750 ms; `packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:10-12`, `:292-305`).
- The 750 ms constant is also the match-buffer reset window: re-typing the same full string while still "typing" does not re-match; after `advanceTimersByTime(750)` the same string matches the next list item, starting from the current `activeIndex` and looping (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:158-202`).

## Keyboard interactions

### useListNavigation — list mode

- Reference, closed: ArrowDown opens and focuses the first item; ArrowUp opens and focuses the last item (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:191-209`).
- Reference, `orientation="horizontal"`: ArrowRight opens + first item; ArrowLeft opens + last item (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:495-545`).
- Reference, `rtl` + horizontal: mapping flips — ArrowLeft opens + first item, ArrowRight opens + last item (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:549-599`).
- `openOnArrowKeyDown={false}`: ArrowDown/ArrowUp on the reference do not open the popup (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:697-707`).
- Floating element (open, default vertical): ArrowDown moves to the next item, clamping at the last without `loopFocus` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:211-235`); ArrowUp moves to the previous item, clamping at the first (`:237-261`). With `loopFocus`, ArrowDown past the last wraps to the first and ArrowUp before the first wraps to the last (`:441-491`).
- Horizontal orientation: ArrowRight/ArrowLeft move forward/backward, clamping at the ends (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:504-518`, `:530-544`).
- `allowEscape` + `virtual` + `loopFocus`: ArrowUp from the first item and ArrowDown from the last item move the selection "off-list" to `null`; subsequent ArrowDown re-enters at the first item (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:645-660`).
- Home and End do not change the active item (no navigation occurs when they are pressed; the test frames this as required so a typeable combobox reference keeps them for caret movement) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1468-1538`, assertions `:1528-1537`).

### useListNavigation — disabled/hidden skipping (list mode)

- Initial navigation (opening focus) skips an `aria-disabled` item: ArrowDown lands on item 1 when item 0 is `aria-disabled` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:263-286`). However, subsequent ArrowUp from item 1 does land DOM focus on the `aria-disabled` item 0 in that same test (`:282-285`).
- Items hidden with `display: none` are excluded from navigation entirely, including during loop wrap (ArrowUp from item 1 loops to item 2, skipping hidden item 0) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:288-301`); the same holds for `visibility: hidden` (`:303-316`).
- `disabledIndices={[0]}`: ArrowDown from the start skips index 0; ArrowUp from item 1 stays on item 1 (disabled index skipped upward, clamped without loop) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:711-721`).

### useListNavigation — grid navigation (`grid` option)

Fixture context: 49 `role="option"` buttons in 5 columns, `disabledIndices [0..7, 10, 15, 45, 48]`, `openOnArrowKeyDown: false`, wrapped in `FloatingFocusManager` (`packages/react/test/floating-ui-tests/Grid.tsx:14-49`, `:59-90`).

- Opening focuses the first non-disabled item (index 8, since 0–7 are disabled) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:877-895`).
- ArrowRight/ArrowLeft move to the next/previous item, skipping disabled cells: 8→9→11 (10 disabled)→14→16 (15 disabled) forward (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:897-912`); 47→46→44 (45 disabled)→41 backward (`:914-930`).
- ArrowDown/ArrowUp move a full row while staying in the same column: 8→13→18→23→28 down (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:932-945`); 47→42→37→32→27 up (`:947-963`).
- `loopFocus`: ArrowDown cycles within the same column back to the start (8 downs from 8 → 8) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:965-981`); ArrowUp cycles within the column upward (8 ups from 43 → 43) (`:983-1001`).
- `orientation="both"` + `loopFocus`: horizontal arrows wrap within the current row and never leave it (14→right→10 is disabled→11; 11→left→14), while vertical arrows move rows (8→13) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1003-1026`); on the last row the same row-locked wrap applies (46↔47 with 48 disabled, `:1028-1044`).

Virtualized partial rows (`VirtualizedGridRows`: 100-item logical list, only 15 cells rendered, `virtual`, `gridNavigation`, horizontal orientation, `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:91-177`):

- ArrowUp from index 0 wraps to the first item of the last row of the *full* list (0→95 with 100 items) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1046-1062`).
- With a partial last row (98 items: row 19 = 95–97): ArrowUp clamps to the last existing item (4→97) and ArrowDown clamps into the partial row (93→97) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1064-1098`).
- `loopFocus={false}`: ArrowUp does not wrap (stays 4) but ArrowDown still clamps into the partial last row (93→97) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1100-1134`).
- Preferred-candidate fallback (leftward): when the clamped target is disabled (97 in `disabledIndices`) ArrowDown lands on 96 (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1136-1154`); when it is CSS-hidden, ArrowDown lands on 13 instead of hidden 14 (`:1156-1172`).

Complex multi-column grid (37 items, 7 columns, `disabledIndices [0..6, 9, 14, 23, 35]`, `packages/react/test/floating-ui-tests/ComplexGrid.tsx:14-56`):

- Opens on the first non-disabled item (index 7) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1176-1183`).
- RTL-parameterized horizontal movement (`describe.each` over rtl): the "forward" arrow (ArrowRight in ltr, ArrowLeft in rtl) skips disabled cells across rows (7→8→10→13→15→20→24→34→36), and the "backward" arrow mirrors it (36→34→28→20→7) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1185-1273`).
- `orientation="both"` + `loopFocus` on the last row: the forward arrow does not leave the row (from 36 it stays 36, since 35 is disabled and 36 is the row end) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1275-1285`).

### useListNavigation — nested menus

Fixture context: nested menus with `FloatingTree`/`FloatingNode`, submenu triggers as items, `nested: isNested` passed to the hook, one grid submenu (`packages/react/test/floating-ui-tests/Menu.tsx:91-148`, `:397-416`).

- ArrowRight on a nested item opens that submenu and DOM-focuses its first item ('Text') (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1395-1404`, JSDOM-only `:1395`).
- Deep navigation: ArrowRight opens submenu after submenu (Edit → Copy as → Image); inside the grid submenu, ArrowRight/ArrowDown/ArrowLeft/ArrowUp move among `.png/.jpg/.svg/.gif` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1407-1441`).
- Escape closes the open submenu and returns focus to the parent item that opened it ('Image' trigger) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1439-1440`).
- Menubar-style horizontal root (`HorizontalMenu`, `packages/react/test/floating-ui-tests/MenuOrientation.tsx:386-460`): ArrowDown opens a submenu from a horizontal root item; after entering a nested submenu ('Mail' focused), ArrowLeft closes it and returns focus to the parent item ('Copy as') (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1444-1466`).

### useListNavigation — WebKit variant

- No keyboard-specific differences are asserted in the WebKit file (`packages/react/src/floating-ui-react/hooks/useListNavigation.webkit.test.tsx:66-110`); ArrowDown still opens and focuses the first item (`:70-73`).

### useTypeahead

- Printable characters match list entries case-insensitively: 't' matches 'two' (index 1) in `['one','two','three']` (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:129-143`).
- Rapid repeated first-letter presses cycle through all items starting with that letter, wrapping: 't','t','t' → 1, 2, 1 (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:135-142`).
- Bail-out: if any list string starts with two of the same letter ('aaron' in `['apple','aaron','apricot']`), a repeated 'a' does not advance — both presses match index 0 (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:145-156`).
- Full-string typing starts from the current `activeIndex` and loops: 'toy' → 0; immediate re-type of 'toy' produces no match call; after the 750 ms reset, 'toy' → 1, then → 2, then → 0 (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:158-202`). This behavior is identical under `React.StrictMode` (`:158-165`).
- CapsLock: `CapsLock` + 't' still matches (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:204-212`).
- Locale-independence: matching survives a Turkish-locale `toLocaleLowerCase` spy — 'i' matches 'Istanbul' (matching does not use locale-sensitive lowercasing) (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:214-233`).
- Active matching works when focus is anywhere inside the reference subtree (nested `input` inside the reference div, `packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:266-274`) and when focus is inside the floating element (focus an option, then type) (`:276-290`).
- With `elementsRef`, CSS-hidden items are excluded from matches: `display: none` item 0 is skipped so 'a' matches 'apricot' (index 1) (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:307-315`); a hidden double-letter item ('aaron') no longer blocks rapid cycling — 'a','a' → 1, then 2 (`:317-334`); `visibility: hidden` items are skipped the same way (`:336-348`).

## Focus management

### useListNavigation — non-virtual (real DOM focus)

- Opening via ArrowDown/ArrowUp moves real focus to the first/last item (items carry `tabIndex={-1}` in fixtures) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:191-209`); item focus tracks every navigation step (`:211-261`).
- `focusItemOnOpen`: `true` focuses the first item on click-open; `false` leaves focus where it was (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:602-618`).
- Item focus on open/navigation is scheduled via `requestAnimationFrame` and is cancelable: with rAF mocked, after click-open the item is `aria-selected` but not yet focused; a `pointerleave` on the item before the frame runs cancels the pending focus — the item never gets focus, the menu never gets focus, the active index clears to `null` (browser-only, `it.skipIf(isJSDOM)`) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:725-778`).
- `focusItemOnHover`: a mouse move with nonzero movement deltas focuses the hovered item and syncs `activeIndex` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:780-790`, hover-focus at `:784-785`); a hover can also sync focus when `activeIndex` is `null` but `selectedIndex` matches the hovered item (`:792-802`).
- On pointer leave of the active item, focus moves to the floating element itself (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:786-787`).
- Pointer-leave clearing is coordinate-aware for clipped containers: if the pointer coordinates (clientX/clientY) are outside the floating element's (clipped, scrollable) rect but still inside the item's rect, the active item is still cleared (`aria-selected` false, `onNavigate(null)`) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:814-873`).
- `selectedIndex` changes do not steal DOM focus: clicking the reference (which changes `selectedIndex` in the fixture) leaves focus on the reference rather than pulling it to the newly selected option (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1377-1392`).
- On open with `selectedIndex`, the selected item is scrolled into view through a `requestAnimationFrame` callback (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:621-641`).
- Nested menus: opening a submenu transfers focus to the submenu's first item (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1395-1404`); Escape returns focus to the parent item (`:1439-1440`); ArrowLeft in the horizontal menubar case returns focus to the parent item ('Copy as') (`:1463-1464`).

### useListNavigation — virtual mode

- No DOM focus is moved: navigation is observed through state attributes (`data-active-index` on a status span; `aria-selected`/`data-active` on items) while the reference (an input) keeps focus (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:117-177`, `:1046-1098`).
- Keys pressed on the reference while open are still processed by the hook (ArrowDown/ArrowUp navigate; `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:318-438`, `:645-679`).

### useListNavigation — WebKit variant

- Stationary `mousemove` (zero `movementX`/`movementY`, as WebKit fires when the list scrolls under a still pointer) never moves focus or the highlight (`packages/react/src/floating-ui-react/hooks/useListNavigation.webkit.test.tsx:67-87`).
- A stationary `pointermove` bubbling through the floating element does not switch the interaction out of keyboard modality; focus stays on the keyboard-highlighted item (`packages/react/src/floating-ui-react/hooks/useListNavigation.webkit.test.tsx:89-102`).
- A `pointerleave` on the active item whose `relatedTarget` is inside the floating element does not clear the keyboard highlight (`aria-selected` stays `true`) (`packages/react/src/floating-ui-react/hooks/useListNavigation.webkit.test.tsx:103-110`).

### useTypeahead

- The hook does not move DOM focus; matching is reported via `onMatch` and the consumer syncs state (fixtures mirror it to `aria-selected`/`tabIndex`) (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:33-36`, `:107-121`).
- Matching remains active with focus inside the reference subtree or inside the floating element (see Keyboard interactions, `packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:266-290`).

## Accessibility (roles, aria-*, id linking)

### useListNavigation

- The hook does not set `aria-orientation` on the floating element, even with `orientation="horizontal"` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:180-189`).
- The hook does not inject ARIA attributes itself; fixtures surface the active index via `aria-selected={activeIndex === index}` on items (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:64`, WebKit fixture `packages/react/src/floating-ui-react/hooks/useListNavigation.webkit.test.tsx:47-49`) and via `data-active` in virtual grid fixtures (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:159`).
- Disabled semantics: the hook skips items carrying `aria-disabled="true"` on initial navigation (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:263-286`); `disabled`/hidden state for grid fixtures is rendered by the fixtures themselves (`packages/react/test/floating-ui-tests/Grid.tsx:34`, `:79`).
- Id linking is not performed by the hook; fixtures wire `aria-haspopup`/`aria-expanded`/`aria-controls` to `context.floatingId` and `id`/`role="menu"`/`aria-labelledby` manually (e.g. `packages/react/test/floating-ui-tests/EmojiPicker.tsx:137-150`, `packages/react/test/floating-ui-tests/Menu.tsx:212-218`, `:274-278`). No test asserts `aria-activedescendant` anywhere in this batch — UNVERIFIED — inferred from absence across `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1-1539`, no test asserts this.
- Home/End being ignored preserves combobox text-editing semantics for a typeable reference (`role="combobox"` input) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1468-1538`).

### useTypeahead

- No ARIA attributes are applied by the hook; the fixtures set `role="combobox"`/`role="listbox"`/`role="option"` and `aria-selected` themselves (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:48-59`, `:104-121`).

## DOM structure & portal behavior

### useListNavigation

- The hook renders nothing; it operates on consumer-rendered DOM: items must register into `listRef` via ref callbacks (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:74-78`), and handlers come from `getItemProps`/`getFloatingProps`/`getReferenceProps`.
- Fixtures place the floating element inside `FloatingPortal` and `FloatingFocusManager` (non-modal for menus): grid tests use `FloatingFocusManager` around a `role="menu"` grid (`packages/react/test/floating-ui-tests/Grid.tsx:59-90`); EmojiPicker uses `FloatingPortal` + `FloatingFocusManager modal={false}` (`packages/react/test/floating-ui-tests/EmojiPicker.tsx:238-241`); nested menus use `FloatingTree`/`FloatingNode`/`FloatingPortal`/`FloatingFocusManager` (`packages/react/test/floating-ui-tests/Menu.tsx:211-305`). Tests query these through normal roles, so portal/nested rendering is fully supported.
- Grid layout is consumer CSS (e.g. 5 fixed columns); the `grid` module defines the logical column count (`packages/react/test/floating-ui-tests/Grid.tsx:14`, `:67`).
- Virtualized grids: only a subset of rows is rendered while `listRef.current.length` is extended to the logical total; navigation wraps/clamps against the full logical list (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:129-131`, `:1046-1098`). Grid semantics are also exercised over `role="grid"`/`row`/`gridcell` markup (`:140-172`).
- Keydowns are handled whether dispatched on the reference, the floating element, or `document` (grid test dispatches on `document` while `FloatingFocusManager` is active) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:882-885`).

### useTypeahead

- Also headless; the basic fixture renders no item elements at all (string-only `listRef`), and matching works without any DOM items (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:29-74`). When item elements exist, they are optional and consulted through `elementsRef` for visibility filtering (`:89-98`, `:307-348`).

## Events (names, payload shape, bubbling, preventDefault semantics)

### useListNavigation

- `onNavigate(index: number | null)` is the sole callback; tests assert only the first argument (the index or `null`), e.g. `onNavigate` spy calls checked for `null` and specific indices (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:671-679`, `:788`, `:800`, `:872`). The fixture forwards `(index, undefined)` as the second argument but no test asserts a second-argument payload — UNVERIFIED — inferred from `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:40-43`, no test asserts this.
- `onNavigate(null)` is delivered when: navigation escapes the list bounds (`allowEscape`, `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:671-679`), pointer-leave clears the active item (`:769-772`, `:869-872`).
- `onNavigate` fires synchronously with the triggering interaction: grid tests chain many `fireEvent.keyDown` calls with no flush between them and assert focus after each (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:897-912`); allowEscape assertions run without awaiting after keydowns (`:645-660`).
- Handled input events: `keydown` (reference and floating elements; see Keyboard interactions), `mousemove`/`pointermove` with movement deltas (hover focus, `:780-790`), `pointerleave` (`:786-787`, `:814-873`), WebKit-conditional stationary `mousemove`/`pointermove`/`pointerleave` handling (`packages/react/src/floating-ui-react/hooks/useListNavigation.webkit.test.tsx:67-110`).
- Bubbling and `preventDefault` semantics: no test in this batch asserts either — UNVERIFIED — inferred from absence across `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1-1539`, no test asserts this.

### useTypeahead

- `onMatch(index: number)` — payload is the matched list index (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:33-36`, asserted at `:136-142`, `:151-155`, `:172-200`).
- `onTyping(typing: boolean)` — payload is the typing-state flag: `true` on first keystroke, `false` after the 750 ms window (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:292-305`).
- Input events: printable-character `keydown`s (via `userEvent.keyboard`) are matched; no bubbling or `preventDefault` assertions exist in this batch — UNVERIFIED — inferred from absence across `packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:1-349`, no test asserts this.

## Edge cases (rapid interactions, unmount, nesting)

### useListNavigation

- Rapid consecutive keydowns are processed synchronously and correctly, including long chains that skip multiple disabled grid cells (9 forward presses, `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1189-1227`; 11 backward presses, `:1229-1273`; repeated column loops, `:965-1001`).
- Same-letter/adjacent-boundary sequences: escape → re-enter (ArrowUp off-list then ArrowDown back to item 0, `packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:645-660`); close → reopen resets the index pointer (`:318-438`).
- Disabled + hidden candidates in virtualized partial rows fall back leftward instead of failing (disabled 97 → 96; hidden 14 → 13) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1136-1172`).
- Changing the list while open (search filtering) keeps navigation consistent: after typing a filter, ArrowDown changes the active index and never leaves it `null` unintentionally (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1289-1317`).
- Disabled items in a filtered grid are never activated ('orange' never becomes active), and unmount + re-render restores clean navigation state — after remount, ArrowDown/ArrowRight/ArrowUp sequences land on the expected item ('cherry' active) (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1319-1375`).
- Clipped/scrollable containers: pointer-leave uses client coordinates against the floating element's rect, so leaving through a clip edge still clears the active item (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:814-873`).
- WebKit-specific: stationary pointer events generated by scrolling must be ignored, and pointer events must not break keyboard modality (`packages/react/src/floating-ui-react/hooks/useListNavigation.webkit.test.tsx:67-110`).
- Nested menus (incl. `keepMounted` grid submenus, `packages/react/test/floating-ui-tests/Menu.tsx:401-411`): multi-level open, grid navigation inside a nested grid submenu, Escape-to-parent, and menubar-style ArrowDown/ArrowLeft flows all behave (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1395-1466`).
- Known flakiness note recorded in-test: an animation-frame callback from `enqueueFocus` can land after a click, briefly focusing the wrong element (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:1380-1382`).

### useTypeahead

- Rapid same-letter cycling and its deliberate bail-out when a doubled-letter prefix exists (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:129-156`).
- The 750 ms typing window gates repeated full-string matches (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:158-202`); the suite runs under fake timers with `shouldAdvanceTime` (`:10-12`) and is StrictMode-safe (`:158-165`).
- Hidden items (both `display: none` and `visibility: hidden`) are excluded only when `elementsRef` is supplied, including the hidden-double-letter interaction with rapid cycling (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:307-348`).

## Shared harness dependencies

- `packages/react/test/index.ts` — the `#test-utils` entrypoint re-exported by the tests; provides `useTestInteractions`, `isJSDOM` (via `@base-ui/utils/testUtils`), `flushMicrotasks`-adjacent waits, and pointer helpers (`packages/react/test/index.ts:1-11`).
- `packages/react/test/useTestInteractions.ts` — merges each hook's `reference`/`floating`/`item`/`trigger` prop objects into `getReferenceProps`/`getFloatingProps`/`getItemProps`/`getTriggerProps`; forces `tabIndex={-1}` plus a focusable attribute on the floating element, chains event handlers (returning the first non-`undefined` result), and strips the internal `active`/`selected` item keys from user props (`packages/react/test/useTestInteractions.ts:30-67`, `:81-84`, `:107-151`).
- `packages/react/test/floating-ui-tests/Grid.tsx` — 5-column grid fixture: 49 `role="option"` buttons, `disabledIndices [0..7,10,15,45,48]`, `openOnArrowKeyDown: false`, `FloatingFocusManager`, `orientation`/`loopFocus` props (`packages/react/test/floating-ui-tests/Grid.tsx:14-49`, `:59-90`).
- `packages/react/test/floating-ui-tests/ComplexGrid.tsx` — 7-column, 37-item grid fixture with its own disabled set and an `rtl` prop, used by the `describe.each` rtl matrix (`packages/react/test/floating-ui-tests/ComplexGrid.tsx:14-56`, `:79-95`).
- `packages/react/test/floating-ui-tests/EmojiPicker.tsx` — searchable 3-column grid picker: `virtual`, `allowEscape`, `focusItemOnOpen: false`, `loopFocus`, horizontal orientation, input as the navigation reference, `FloatingPortal` + non-modal focus manager, disabled 'orange' item, `data-active-index` reporter (`packages/react/test/floating-ui-tests/EmojiPicker.tsx:21-23`, `:152-176`, `:238-257`, `:89`).
- `packages/react/test/floating-ui-tests/ListboxFocus.tsx` — listbox combining `useListNavigation` + `useTypeahead` with `selectedIndex`; the reference button sets `selectedIndex={1}` on click, used for the focus-steal test (`packages/react/test/floating-ui-tests/ListboxFocus.tsx:37-49`, `:64`).
- `packages/react/test/floating-ui-tests/Menu.tsx` — nested menu fixture (`FloatingTree`/`FloatingNode`/portal/non-modal focus manager, `nested: isNested`, `keepMounted`, grid 'Image' submenu, labels-driven typeahead) backing the nested-navigation tests (`packages/react/test/floating-ui-tests/Menu.tsx:91-148`, `:267-301`, `:397-416`).
- `packages/react/test/floating-ui-tests/MenuOrientation.tsx` — horizontal menubar variant (`HorizontalMenu`) used for the different-orientation nested test (`packages/react/test/floating-ui-tests/MenuOrientation.tsx:118-138`, `:386-460`).
- `packages/react/test/floating-ui-tests/gridNavigationWithColumns` — column-count-parameterized grid module factory consumed by the three grid fixtures (`packages/react/test/floating-ui-tests/Grid.tsx:12-14`, `packages/react/test/floating-ui-tests/ComplexGrid.tsx:12-14`, `packages/react/test/floating-ui-tests/EmojiPicker.tsx:21-23`).
- `packages/react/test/floating-ui-tests/renderGridRows` — grid-row rendering helper used by the menu fixtures (`packages/react/test/floating-ui-tests/Menu.tsx:32`).
- `useListNavigation.test.tsx` also imports the source-side `gridNavigation` module from `./gridNavigation` and passes it as the `grid` option (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:8`, `:125`).
- External test deps used directly by these tests: `flushMicrotasks` from `@mui/internal-test-utils` (`packages/react/src/floating-ui-react/hooks/useListNavigation.test.tsx:5`), Testing Library `render`/`fireEvent`/`userEvent` (`:3-4`), Vitest (`:1`), and Vitest fake timers for the typeahead timing assertions (`packages/react/src/floating-ui-react/hooks/useTypeahead.test.tsx:10-12`).
