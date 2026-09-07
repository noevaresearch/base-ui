# floating-ui-react components — behavior spec (mined from unit tests)

Scope: `packages/react/src/floating-ui-react/components/` — `FloatingDelayGroup` (+ `useDelayGroup`), `FloatingFocusManager`, `FloatingPortal` (+ `FloatingPortalLite`).

**Delegation note (applies to every section below):** this package is an in-repo adaptation that wraps `@floating-ui/react-dom` and `@floating-ui/utils`. Any underlying algorithm not directly asserted at wrapper level (positioning/middleware math, platform/DOM internals, tree bookkeeping internals) is delegated to those packages and must not be reimplemented. The Rust equivalent crate for Stage 3 to bind against is **`floating-ui-leptos`** (https://floating-ui.rustforweb.org/frameworks/leptos.html).

## Public API surface (props, parts, subcomponents)

### FloatingDelayGroup
- `<FloatingDelayGroup delay={{ open: number, close: number }} timeoutMs={number}>` wraps multiple floating consumers (tooltips) and coordinates their open/close delays as a group. Test usage: `packages/react/src/floating-ui-react/components/FloatingDelayGroup.test.tsx:67`, `:139`.
- Props observed: `delay` (`{ open, close }` in ms; `:67`), `timeoutMs` (ms; `:139`).
- `useDelayGroup(context, { open })` hook is the consumer side; returns `{ delayRef, isInstantPhase }` — `delayRef.current` is intended to be passed as the per-hook `delay` (e.g. to `useHover`), and `isInstantPhase` is a boolean flag. Test fixture wiring: `packages/react/src/floating-ui-react/components/FloatingDelayGroup.test.tsx:23-24`.
- Consumers are plain components calling `useFloating`/`useHover`; no special child component or render-prop is required (`packages/react/src/floating-ui-react/components/FloatingDelayGroup.test.tsx:15-63`).
- UNVERIFIED — inferred from `packages/react/src/floating-ui-react/components/FloatingDelayGroup.test.tsx:23`, no test asserts this: the exact JSDoc/prop types of `FloatingDelayGroup` and `useDelayGroup` beyond those exercised here.

### FloatingFocusManager
- `<FloatingFocusManager context={...}>` — requires the `context` from `useFloating` (first prop in every usage, e.g. `packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:99`, `:131`).
- Props exercised by tests (citations in this list are `packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx`):
  - `initialFocus`: `true`/default → first tabbable (`packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:188-195`); a `RefObject<HTMLElement>` → that element (`:206-212`); `false` (or `-1`) → do not move focus in (`:2165`, `:2335`, `:2376`, `:2505`).
  - `returnFocus`: default `true`; `false` (`:238`, `:247`); a `RefObject` → alternate return target (`:267`); a function → callback form, receives an interaction type and its (falsy) return suppresses fallback insertion (`:676`, `:1491`).
  - `modal`: boolean; `true` traps Tab and hides outside content; default `true` (tests explicitly pass `modal={false}` for non-modal, `:425`, `:880`, etc.).
  - `closeOnFocusOut`: boolean; `false` prevents close when focus leaves a non-modal floating element (`:903-926`).
  - `disabled`: boolean; disables all focus management while mounted (`:1360`, `:1399`, `:1431`, `:1491`, `:1606`, `:1666`).
  - `restoreFocus`: boolean; restore focus to nearest tabbable when focused element is removed/hidden (`:1989`, `:2003-2065`).
  - `getInsideElements`: `() => Element[]`; marks additional elements as "inside" so they are not hidden from AT (`:1207-1210`).
- Children: a single element (the floating element) or a fragment containing it (`:1211-1214`); the manager reads the element that carries the floating props (`refs.setFloating` ref target or `getFloatingProps()` host) to place initial focus (`:2296-2310`).
- Co-exports used with it: `FloatingNode`/`FloatingTree`/`useFloatingNodeId`/`useFloatingParentNodeId` for nesting (`:22-33`, `:165`, `:288-295`).

### FloatingPortal
- `<FloatingPortal>` portals children into the DOM. Props exercised (citations in this list are `packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx` unless noted):
  - `container`: `HTMLElement` (`packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx:35`), `RefObject<HTMLElement>` (`:49-50`), or `null`/`undefined` initially (resolves later; `:59-84`, `:92-99`). Default container is `document.body` (`:108-110`).
  - `id`: sets the rendered portal element's id, used for the `aria-owns` relationship (`packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx:146`, `:157`).
  - `portalOwnerRole`: sets the `role` of the rendered aria-owns owner element (`packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:1852`, `:1874`).
  - Any additional HTML props (`data-testid`, `className`, …) are forwarded to the portal element (`packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx:126-139`).
- Rendered portal element carries the attribute `data-base-ui-portal` (`:41`, `:54`, `:138`).
- `FloatingPortalLite` (internal util, `packages/react/src/utils/FloatingPortalLite`) also forwards HTML props to its portal element (`packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx:160-171`).

## State model (controlled/uncontrolled, defaults, transitions)

### FloatingDelayGroup
- Open/close state lives in each consumer (`useFloating` + `onOpenChange`), not in the group; the group only mediates delay values and an "instant phase" flag (`packages/react/src/floating-ui-react/components/FloatingDelayGroup.test.tsx:18-24`).
- Defaults (no group / first interaction): a member opens only after the group's `delay.open` elapses — with `delay.open: 1000`, floating is absent at +1ms and present at +1000ms (`:89-101`).
- Transition "hover next member while one is open": previous floating closes and the new one opens **instantly** (within 1ms), skipping the open delay (`:103-110`, `:112-119`).
- Transition "close": on `mouseLeave`, the floating stays open for the group `delay.close` (absent at +1ms, gone by +200ms with `close: 200`) (`:121-133`).
- `timeoutMs` window: after a member closes, hovering another member within `timeoutMs` still opens instantly (`:171-177`); the close `delay.close` (100ms) governs actual closing (`:188-200`).
- `isInstantPhase` → surfaced by the fixture as `data-instant-phase` on the reference: present while the member is in the instant-open phase, removed after the group's timeout window elapses (50ms in test) (`:217-235`).
- Unmount transitions: unmounting an *inactive* member preserves the active member's context and instant-open behavior for the next member (`:238-283`); unmounting the *just-closed* member preserves the group timeout so the next open is still instant (`:285-332`).
- Render isolation: members that never receive interaction render a small bounded number of times (3) regardless of other members' open/close activity; interacted members render more (fixture counts 11/7/3) (`:334-379`, counts at `:376-378`).

### FloatingFocusManager
- All props are controlled inputs to imperative focus/attribute behavior; the manager holds no public open state — open/close comes from the `useFloating` context (`packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:87-90`, `:95-112`).
- Default transitions on open: focus moves into the floating element (first tabbable, or `initialFocus` target) unless `disabled` or `initialFocus={false}` (`:188-195`, `:1372-1377`).
- Default transitions on close (`returnFocus` default): focus returns to the reference (see Focus management); with `returnFocus={false}` focus is left where it is (`:236-243`, `:246-257`).
- `disabled` transition `true → false` while mounted: the floating element acquires focus at the moment `disabled` flips false (`:1341-1378`) — enabling keepMounted patterns (`disabled={!isOpen}`, `:1431`).
- Non-modal `modal={false}` transition: Tab past the last inside element triggers close (via `onOpenChange(false)` wrapped in a setTimeout) and focus lands on the next outside element (`:880-901`); `closeOnFocusOut={false}` removes that transition (`:903-926`).
- Close modality (keyboard vs pointer) is tracked per open session and reset between sessions of a keep-mounted manager: Escape close → `focusVisible: true` on restore and `returnFocus` callback receives `'keyboard'`; pointer/programmatic close → no `focusVisible`, callback receives `''`; a `click` with `detail: 0` is classified as keyboard (`'keyboard'`) (`:1470-1577`, assertions `:1519`, `:1536`, `:1555`, `:1573`).
- Delegated: none of the above involves positioning; all focus/timer logic is wrapper-level. (Underlying platform focus checks are delegated to `@floating-ui/utils`; Rust equivalent `floating-ui-leptos`.)

### FloatingPortal
- No state of its own beyond container resolution: `container` may start `null`/`undefined` and resolve later; the portal mounts into `document.body` by default and re-parents when `container` changes (`packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx:59-84`, `:86-124`).
- Portal element persists per mounted `FloatingPortal`; the conditionally-rendered child (`{open && ...}`) appears/disappears inside it (`packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx:22-25`, `packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:1706-1748`).

## Keyboard interactions

### FloatingDelayGroup
- N/A (mouse-delay component; tests only use `fireEvent.mouseEnter`/`mouseLeave`).

### FloatingFocusManager
- **Tab (modal=true):** focus is trapped inside the floating element and wraps: one→two→three→one; Shift+Tab wraps backwards three→two→one→three (`packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:849-878`).
- **Tab (modal=false):** Tab from the last inside element moves focus out; the floating element then closes and focus is on the following outside element (`:880-901`). Repeated Tab cycles keep working after reopen in a combobox flow (`:2355-2410`).
- **Shift+Tab (non-modal, floating between siblings):** first Shift+Tab keeps the floating open (`:1912-1914`); second Shift+Tab closes it (`:1916-1918`). Standard back-and-forth returns focus between reference and inner element (`:2683-2686`).
- **Escape:** dismisses (with `useDismiss`); the manager then returns focus to the reference with `{ preventScroll: true, focusVisible: true }` (`:595-602`, `:1512-1518`). Escape on a hover-opened popup also returns focus to the reference (`:1930-1936`) and does not re-open on subsequent hover (`:1949-1954`).
- **Enter on reference:** opening via keyboard (Enter on the reference's focusable descendant) works; closing via Escape returns focus to that descendant when the reference itself is not focusable (`:330-348`).
- **Tab in a trapped combobox (modal):** with a modal manager, focus moves from the input into the floating listbox and stays trapped cycling its buttons (`:2069-2126`).
- **Tab with `initialFocus={false}`:** focus stays on the reference/input on open (`:2179-2181`, `:2349`); a subsequent Tab exits to the outside element (`:2350-2352`, `:2394-2398`).
- **Tab/Shift+Tab with no tabbable content (modal):** focus rests on the floating element itself (tabIndex=-1) and both Tab and Shift+Tab keep it there (`:1008-1031`).

### FloatingPortal
- N/A (no keyboard behavior asserted).

## Focus management

### FloatingDelayGroup
- N/A.

### FloatingFocusManager
- **Initial focus:** (all citations in this list are `packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx`)
  - Default: first tabbable element inside the floating element (`packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:188-195`).
  - Named radio group: the *checked* radio receives focus instead of the first tabbable (`:197-204`).
  - `initialFocus` as ref: that element (`:206-212`).
  - `autoFocus` on a child wins over the default first-tabbable choice (`:214-223`).
  - `initialFocus={false}`: focus is not moved into the floating element (`:2165`, `:2335`, `:2505`).
  - Wrapper elements: focus is placed on the element carrying the floating props (`getFloatingProps()` host), even when it is nested inside the manager's direct child (`:2260-2311`).
- **Return focus on close:**
  - Default (`returnFocus` true): focus returns to the reference after close, with options `{ preventScroll: true }` plus `focusVisible: true` when closed via keyboard, and without `focusVisible` when closed via pointer (`:556-606`, `:608-660`).
  - `returnFocus={ref}`: focus goes to the referenced element (`:259-280`).
  - `returnFocus={fn}`: called with the close interaction type (`'keyboard'` or `''`); a falsy return suppresses insertion of any fallback element (`:662-702`, `:1491-1577`).
  - Non-focusable reference: focus returns to the first focusable descendant of the reference (`:330-348`).
  - Nested floats: focus return targets the correct (nearest) reference per level; outside presses dismiss one level at a time (`:282-328`).
  - Replacing elements: when the reference itself is removed on close, a fallback tabbable is inserted next to its former position so the next Tab lands on the element after it — both modal (`:350-402`) and non-modal (`:404-456`).
  - Outside-press close: focus is restored to the reference only when the environment's `focus()` supports options (preventScroll detection); otherwise it is not (`:458-500` vs `:502-554`).
  - Hover-opened floats: focus does **not** return to the reference on unhover close (`:2412-2451`) but does return on Escape or explicit Close-button close (`:1930-1947`).
  - Replacement chain: when one popup is closed by opening another, focus returns to the *last connected* element in the chain (parent's reference after drawer Escape) (`:2188-2258`).
- **Focus restoration (`restoreFocus`):**
  - `true` (browser-only test): if the focused element is removed, focus moves to the nearest tabbable ("three") (`:2003-2023`).
  - `false`: focus falls back to `document.body` (`:2025-2045`).
  - Focused element becoming hidden (`visibility: hidden`) also triggers restoration to the nearest tabbable (`:2047-2065`).
- **Tab-out close and focus-out:** non-modal closes when focus leaves (`:894-901`); `closeOnFocusOut: false` keeps it open (`:903-926`); clicking a nested `data-base-ui-click-trigger` element does not permanently suppress the next focus-out close (`:928-968`).
- **disabled interplay:** with `disabled`, no focus management happens; flipping to enabled grabs focus (`:1341-1378`); keep-mounted Escape close returns focus to reference (`:1465-1467`).
- **Outside-pointer state reset:** between keep-mounted open sessions, stale outside-pointer state is cleared: after reopening, a `focusOut` from the child with an outside `relatedTarget` leaves `context.dataRef.current.insideReactTree === true` (`:1642-1703`).
- **Reopen-before-restore:** reopening synchronously during close preserves the keyboard close modality (focus restored with `focusVisible: true`) and open stays true (`:1579-1640`).
- Delegated: tabbable-element computation and focus-order heuristics on real DOM are provided by `@floating-ui/utils` internals; Rust equivalent crate `floating-ui-leptos`.

## Accessibility (roles, aria-*, id linking)

### FloatingDelayGroup
- N/A (no aria attributes asserted; only the `data-instant-phase` marker, `packages/react/src/floating-ui-react/components/FloatingDelayGroup.test.tsx:43`, `:228`, `:235`).

### FloatingFocusManager
- **modal=true hides outside content from AT:** on open, outside elements get `aria-hidden="true"`; the floating element itself does not; a sibling `aria-live` region is kept exposed; on close all `aria-hidden` are removed (`packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:1139-1185`).
  - The reference gets `aria-hidden="true"` too (`:1173`).
  - `getInsideElements` opt-out: elements returned by it (even outside the floating subtree) are not aria-hidden; the enclosing outside wrapper still is (`:1187-1229`).
- **modal=true + hover-opened:** outside nodes get `aria-hidden` instead of `inert` (`:2453-2486`).
- **modal=false does not hide content:** no `inert` attribute on any node (`:970-1006`, `:1231-1280`); instead a `data-base-ui-inert` marker attribute is applied to top-level outside ancestors — the reference itself when it has no wrapper (`:1269`), or the wrapper branch containing the reference and separate top-level siblings (`:1282-1337`), never to the reference's siblings or nested descendants (`:1271-1272`, `:1322-1326`); removed on close (`:1274-1279`, `:1330-1336`). Non-modal focus guards do not mark reference siblings (`:1750-1791`).
- **tabindex management of the floating element:**
  - Floating element with no tabbable content and a non-listbox role gets `tabindex="0"` (after a focus-out, with `initialFocus={false}`, non-modal) (`:2488-2522`).
  - Managed tabindex is mirrored in a `data-tabindex` attribute and downgraded `0 → -1` once content becomes tabbable (`:2524-2560`).
  - `role="listbox"` floating elements are never upgraded (stay `tabindex="-1"`) (`:2562-2605`).
  - Non-modal dialog without tabbable content gets `tabindex="0"` and is focusable in Tab order (`:2607-2642`); with tabbable content it stays `tabindex="-1"` (`:2681`).
- **aria-owns linking (with FloatingPortal):** the portal node gets a generated id; an owner `span[aria-owns]` is rendered into the document with `aria-owns=<portal node id>`; by default the owner has no `role`, and the reference itself does not receive `aria-owns` (`:1793-1835`); `FloatingPortal portalOwnerRole="group"` sets `role="group"` on that owner (`:1837-1875`); `FloatingPortal id="custom-portal"` makes the owner point at the custom id (`packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx:141-158`).
- Role attributes (`role="dialog"`, `role="listbox"`, `aria-expanded`, `aria-controls`, `aria-haspopup`) in these tests are supplied by fixtures/user props, not by the manager itself (`:101`, `:2079-2088`, `:2269-2281`).

### FloatingPortal
- Portal element exposes `data-base-ui-portal` (`packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx:41`, `:54`, `:138`) and forwards user HTML attributes (`:126-139`); its id is consumed by the aria-owns owner (`:141-158`).

## DOM structure & portal behavior

### FloatingDelayGroup
- N/A (no DOM structure asserted beyond fixture-rendered floating divs).

### FloatingFocusManager
- Renders no wrapper DOM of its own: children render in place; the manager attaches behavior to the floating element and manipulates attributes (`aria-hidden`, `inert`-marker, `tabindex`, `data-tabindex`, `data-base-ui-inert`) on existing nodes (`packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:1139-1337`, `:2488-2687`).
- May insert a temporary fallback tabbable element next to the reference's former position when the return target is removed; not inserted when the return element is falsy (`:350-402`, `:404-456`, `:662-702`).
- Renders one `span[aria-owns]` owner element into the owner document when used with a portal; its `role` is configurable via `portalOwnerRole` (`:1793-1875`).
- Works inside nested documents: a non-modal manager portaled into an iframe integrates with that document's Tab order (`:704-845`).

### FloatingPortal
- Creates a portal host element (with `data-base-ui-portal`) as the direct parent of the children; default parent of the host is `document.body` (`packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx:40-42`, `:108-110`).
- `container` accepts an element, a ref object, or a lazily-resolved (initially null) element; children re-parent into it (`:31-84`).
- Container changes cause re-attachment: body → customRoot → body; after switching away, the floating content is no longer contained by the previous container (`:86-124`).
- HTML props (`data-testid`, `className`, `id`) land on the portal host element (`:126-139`, `:141-158`).
- `FloatingPortalLite` (internal) renders a portal host receiving forwarded HTML props (`:160-171`).

## Events (names, payload shape, bubbling, preventDefault semantics)

### FloatingDelayGroup
- N/A (only synthetic `mouseEnter`/`mouseLeave` triggers on references; no emitted events).

### FloatingFocusManager
- No custom DOM events are emitted. Observable "event-like" surface:
  - `onOpenChange` from `useFloating` is invoked on focus-out close, wrapped in a `setTimeout` (tests wait a macrotask before asserting removal) (`packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:894-898`).
  - `returnFocus` callback receives a close-interaction-type payload: `'keyboard'` for Escape / `detail: 0` click, `''` for pointer or programmatic close (`:1519`, `:1536`, `:1555`, `:1573`).
  - Programmatic focus restore calls `reference.focus({ preventScroll: true })`, adding `focusVisible: true` only for keyboard closes (`:597-602`, `:651-656`, `:1513-1518`, `:1530-1535`).
  - The manager listens to native `focusout` (dispatched directly in tests with `relatedTarget`) to drive close/restore/tabindex behavior (`:1698-1702`, `:2518`, `:2555`).
- The `data-base-ui-click-trigger` attribute on a nested element marks a click trigger; clicking it does not suppress the subsequent focus-out close (`:928-968`).
- Delegated: pointer-outside-press classification (`context.dataRef.current.insideReactTree`) is maintained by the interaction hooks/`@floating-ui` internals; Rust equivalent crate `floating-ui-leptos`.

### FloatingPortal
- N/A (no event payloads asserted).

## Edge cases (rapid interactions, unmount, nesting)

### FloatingDelayGroup
- Strict Mode double-invocation of lifecycle effects does not leave the instant phase stuck: `data-instant-phase` is present during the instant window and removed after the group timeout (`packages/react/src/floating-ui-react/components/FloatingDelayGroup.test.tsx:203-236`).
- Unmount of an inactive member while another is open keeps the group working (instant switch to a third member) (`:238-283`).
- Unmount of the member that just closed keeps the group timeout active (next open is still instant) (`:285-332`).
- Rapid member-to-member hovering: each switch closes the previous floating instantly within 1ms (`:103-119`).

### FloatingFocusManager
- Rapid open/close within the same tick (reopen before focus restoration) preserves close modality and does not leave stale state (`packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:1579-1640`).
- Keep-mounted sessions: outside pointer state is cleared between sessions so a later focus-out is not misclassified (`:1642-1703`).
- Close modality resets between keep-mounted sessions (keyboard vs pointer classification per session) (`:1470-1577`).
- Reference removal: fallback tabbable insertion keeps Tab order sane when the reference is unmounted together with the popup (modal and non-modal) (`:350-456`).
- Focused-element removal/hidden inside the popup: `restoreFocus` moves focus to the nearest tabbable; disabled restore leaves body focused (`:2003-2065`).
- Nested floats (FloatingTree): outside press dismisses one level per press, from innermost outward (`:282-328`); mixed modal/non-modal nesting coexists (`:1033-1137`); focus return walks to the last connected element across replaced popups (`:2188-2258`).
- Floating element as a wrapper: focus targets the inner element carrying floating props, not the manager's direct child (`:2260-2311`).
- Empty floating element (no tabbable content): focus falls back to the floating element and Tab/Shift+Tab are trapped on it (`:1008-1031`).
- iframe: Tab/Shift+Tab traverse between the host document, the portaled popover inside the iframe, and back, without losing focus (`:704-845`).
- Environment guards: tests polyfill `HTMLElement.prototype.inert` and mock rAF to synchronous to make behavior deterministic (`:62-76`); `preventScroll`-support detection changes outside-press focus return (`:458-554`).

### FloatingPortal
- Initially null container: portal waits for the container to exist and mounts into it (`packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx:59-84`).
- Container switching fully re-parents (no duplicates/leftovers in the old container) (`:86-124`).
- Portal content unmounts with the conditional child while the portal host persists (`:22-25`, `:126-139`).

## Shared harness dependencies

- `packages/react/test/index.ts` — the `#test-utils` entry point re-exporting `isJSDOM` (via `@base-ui/utils/testUtils`), `useTestInteractions`, `flushMicrotasks`/`waitFor` (via `./wait`), plus render/act/screen/fireEvent from `@mui/internal-test-utils` (`packages/react/test/index.ts:1-11`).
- `packages/utils/src/testUtils.ts` — provides `isJSDOM` (userAgent sniff) used by `describe.skipIf(!isJSDOM)` gates in all three batch tests (`packages/utils/src/testUtils.ts:4`; used at `packages/react/src/floating-ui-react/components/FloatingDelayGroup.test.tsx:7,81`, `packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:21,186`, `packages/react/src/floating-ui-react/components/FloatingPortal.test.tsx:4,30`).
- `packages/react/test/useTestInteractions.ts` — `useTestInteractions(propsList)` merges the `reference`/`floating`/`item`/`trigger` element-props of interaction hooks into `getReferenceProps`/`getFloatingProps`/`getItemProps`/`getTriggerProps`; for floating elements it adds `tabIndex: -1` and the `FOCUSABLE_ATTRIBUTE`, and composes event handlers so all hooks' handlers run (`packages/react/test/useTestInteractions.ts:30-105`). Used by DelayGroup fixture (`packages/react/src/floating-ui-react/components/FloatingDelayGroup.test.tsx:25`), FocusManager fixtures (`packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:159`, `:362`, `:745`, `:1059`, `:1392`, `:1483`, `:1591`, `:1658`, `:1980`, `:2094`, `:2153`, `:2221`, `:2284`, `:2324`, `:2365`, `:2423`, `:2464`, `:2572`, `:2654`), and the Navigation fixture.
- `packages/react/test/floating-ui-tests/Navigation.tsx` — fixture used by the FocusManager `Navigation` describe: a nav bar of `NavigationItem`s using `useHover` (+`safePolygon`), `useFocus`, `useDismiss`, `FloatingNode`/`FloatingPortal`, and a non-modal `FloatingFocusManager` with `initialFocus={false}`; exposes `Product` reference, `subnavigation` floating panel, `Close` button, `Link 1/2/3` sub-items (`packages/react/test/floating-ui-tests/Navigation.tsx:46-110`, `:126-143`; consumed at `packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:35`, `:1922-1965`).
- External test runner helpers (`@mui/internal-test-utils`, `@testing-library/user-event`, Vitest) — `render`/`screen`/`fireEvent`/`act`/`flushMicrotasks`/`waitFor`/`within` and fake timers; not repo files (`packages/react/src/floating-ui-react/components/FloatingFocusManager.test.tsx:7-18`, `packages/react/src/floating-ui-react/components/FloatingDelayGroup.test.tsx:5`).
