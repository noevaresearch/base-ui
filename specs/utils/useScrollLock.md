# `useScrollLock` — behavior spec

Unit: `packages/utils/src/useScrollLock` (Phase A util → crate `leptos-ui-utils`).

Corpus note: this unit has no dedicated test file (`testFiles: []` for `useScrollLock` in
`ralph/generated/utils.json:376-382`). Per the stage prompt's "detect from imports" instruction,
behavior was mined from consumer suites that exercise the unit through its production callers:
modal Dialog calls it directly (`packages/react/src/dialog/root/useDialogRoot.ts:86`), and the
anchored popups (Menu, Select, Popover, Combobox, Menubar) route through
`useAnchoredPopupScrollLock`, which forwards a boolean to `useScrollLock`
(`packages/react/src/utils/useAnchoredPopupScrollLock.ts:42`). Claims about unit internals that no
test isolates are marked UNVERIFIED with a source citation.

## Public API surface (props, parts, subcomponents)

- Single named export, a React hook: `useScrollLock(enabled?, referenceElement?)`. No components,
  no props object, no parts, no subcomponents. UNVERIFIED — inferred from
  `packages/utils/src/useScrollLock.ts:312-319`, no test asserts the signature directly.
- `enabled: boolean = true` — the lock is applied only while enabled. Proven in both directions
  through consumers: a mouse-opened modal menu locks the page
  (`packages/react/src/menu/root/MenuRoot.test.tsx:1777-1792`) while a narrow touch-opened popup
  under the same modal root does not (`packages/react/src/menu/root/MenuRoot.test.tsx:1757-1775`),
  and a re-render that mounts a Positioner with a falsy enabled expression leaves body/html
  overflow untouched (`packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:45-63`).
- Default-parameter hazard: passing `undefined` for `enabled` would fall through to the
  `enabled = true` default — documented by the regression test's own explanation
  (`packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:45-48`).
- `referenceElement: Element | null = null` — element used as reference for lock calculations
  (owner document/window resolution and scrollbar measurements). UNVERIFIED — inferred from
  `packages/utils/src/useScrollLock.ts:261-264,290`, no test varies the argument.
- No public imperative API and no return value consumed by callers; acquiring/releasing is tied to
  the effect lifecycle of `enabled`. UNVERIFIED — inferred from
  `packages/utils/src/useScrollLock.ts:312-319`, no test asserts a return value.

## State model (controlled/uncontrolled, defaults, transitions)

- No controlled/uncontrolled state and no per-instance state machine: all lock bookkeeping lives in
  a module-level singleton (`SCROLL_LOCKER`), shared by every consumer on the page. UNVERIFIED —
  inferred from `packages/utils/src/useScrollLock.ts:228-304`, no test asserts refcounting in
  isolation.
- Lock application is deferred, not synchronous: tests wait for a scheduled 0ms timeout before
  asserting lock state ("Flush the scroll locker's deferred lock, scheduled on a 0ms timeout" —
  `packages/react/src/dialog/root/DialogRoot.test.tsx:1985-1988`; "setTimeout(0) in
  ScrollLocker.acquire" — `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:40-43`).
  The deferral mechanism itself (0ms `Timeout` on first acquire) is UNVERIFIED — inferred from
  `packages/utils/src/useScrollLock.ts:234-240`.
- External-lock takeover: when the page is already scroll-locked at enable time, the unit does not
  stack a second lock; it stays in a waiting state and takes over the moment the external lock
  clears, so the page never becomes scrollable. Proven: with a dialog open under a simulated
  third-party locker that releases at 200ms, the page is locked at every sampled time
  t=0/120/260/400ms (`packages/react/src/dialog/root/DialogRoot.test.tsx:1942-1956`) and unlocks
  after the dialog closes (`packages/react/src/dialog/root/DialogRoot.test.tsx:1958-1964`).
  The detection/wait mechanism (attribute-watching `MutationObserver` on `<html>`/`<body>`) is
  UNVERIFIED — inferred from `packages/utils/src/useScrollLock.ts:269-288`.
- Transitions proven at consumer level: enable → locked (mouse-open modal menu,
  `packages/react/src/menu/root/MenuRoot.test.tsx:1777-1792`; modal dialog,
  `packages/react/src/dialog/root/DialogRoot.test.tsx:1990`); disable → unlocked (dialog close,
  `packages/react/src/dialog/root/DialogRoot.test.tsx:1958-1964`; menu handoff in a menubar,
  `packages/react/src/menubar/Menubar.test.tsx:909-916`).

## Keyboard interactions

N/A — non-visual utility. No keyboard behavior is implemented or asserted anywhere in the corpus.

## Focus management

N/A — the unit never moves, traps, or restores focus; no corpus test asserts focus behavior from
the scroll lock.

## Accessibility (roles, aria-*, id linking)

- No roles, no `aria-*` attributes, no id linking: N/A.
- The unit's page-visible footprint is the lock itself; consumer tests detect it as any of: inline
  `overflow: hidden` on `<html>`, inline `overflow: hidden` on `<body>`, or a
  `data-base-ui-scroll-locked` attribute on `<html>`
  (`packages/react/src/menu/root/MenuRoot.test.tsx:1769-1772`,
  `packages/react/src/select/root/SelectRoot.test.tsx:1808-1811`,
  `packages/react/src/popover/root/PopoverRoot.test.tsx:1613-1616`). That the unit itself writes
  the attribute and removes it on unlock is UNVERIFIED — inferred from
  `packages/utils/src/useScrollLock.ts:192` and `packages/utils/src/useScrollLock.ts:203`; no test
  isolates the attribute from the style-based signals.

## DOM structure & portal behavior

- Renders nothing; creates no elements and no portals. It only mutates existing document state:
  inline styles of `<html>`/`<body>`, the `data-base-ui-scroll-locked` attribute, scroll positions,
  and listeners. UNVERIFIED — inferred from `packages/utils/src/useScrollLock.ts:158-226`; tests
  only ever assert the resulting styles/attributes (e.g.
  `packages/react/src/dialog/root/DialogRoot.test.tsx:2255-2260`).
- Which element gets locked follows the viewport-scroller rule: `<html>` when it establishes its
  own scroll container, otherwise `<body>`. An external `<body>`-only `overflow: hidden` is
  correctly treated as NOT locking the page, and the unit still applies its own lock on `<html>`
  (`overflowX === 'hidden'`) in that situation
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1969-1996`, rule stated by the test helper
  at `packages/react/src/dialog/root/DialogRoot.test.tsx:2246-2253`). The selection logic
  (`isOverflowElement(html) ? html : body`) is UNVERIFIED — inferred from
  `packages/utils/src/useScrollLock.ts:16-18,69-82`.
- Test teardown wipes exactly the state the unit touches: `documentElement.style`, `body.style`,
  and `data-scroll-locked` are removed in `afterEach`
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1870-1874`), and the ComboboxPositioner
  regression test unmounts then strips `body`/`documentElement` styles
  (`packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:54-58`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- The unit emits no custom events, and no payload, bubbling, or `preventDefault` semantics exist:
  N/A.
- Listens internally: UNVERIFIED — a window `resize` listener re-applies the lock after a viewport
  resize (inferred from `packages/utils/src/useScrollLock.ts:208-214`), and an attributes-only
  `MutationObserver` on `<html>` + `<body>` drives the external-lock takeover (inferred from
  `packages/utils/src/useScrollLock.ts:270-286`). No test directly asserts listener registration;
  the takeover effect it produces is proven (see State model).

## Edge cases (rapid interactions, unmount, nesting)

- Rapid close/reopen during a pending close animation must not leave a stuck lock: a touch-opened
  select reopened while its previous close animation is still running ends up NOT scroll-locked
  (`packages/react/src/select/root/SelectRoot.test.tsx:1421-1537`, final assertion at 1534-1536).
- Consumer handoff through the shared singleton: in a modal menubar, a viewport-width touch-opened
  menu locks the page (`packages/react/src/menubar/Menubar.test.tsx:806-821`); when a second,
  narrower menu replaces the first (first menu unmounted,
  `packages/react/src/menubar/Menubar.test.tsx:905-907`), the page unlocks
  (`packages/react/src/menubar/Menubar.test.tsx:909-916`) — the lock follows the currently-enabled
  consumer instead of fighting across consumers.
- Touch-opened popups lock only when effectively viewport-sized: a `calc(100vw - 10px)` positioner
  locks and a `240px` positioner does not, in Menu
  (`packages/react/src/menu/root/MenuRoot.test.tsx:1796-1826`,
  `packages/react/src/menu/root/MenuRoot.test.tsx:1828-1862`), Select
  (`packages/react/src/select/root/SelectRoot.test.tsx:1802-1815`,
  `packages/react/src/select/root/SelectRoot.test.tsx:1817-1850`), Popover
  (`packages/react/src/popover/root/PopoverRoot.test.tsx:1587-1620`,
  `packages/react/src/popover/root/PopoverRoot.test.tsx:1622-1655`), Combobox
  (`packages/react/src/combobox/root/ComboboxRoot.test.tsx:4599-4631`,
  `packages/react/src/combobox/root/ComboboxRoot.test.tsx:4634-4670`), and Menubar
  (`packages/react/src/menubar/Menubar.test.tsx:806-821`,
  `packages/react/src/menubar/Menubar.test.tsx:824-854`). The 20px width tolerance and
  offsetWidth-vs-clientWidth measurement are UNVERIFIED — inferred from
  `packages/react/src/utils/useAnchoredPopupScrollLock.ts:7-11,32-39`.
- Mouse-opened modal popups always lock regardless of popup width
  (`packages/react/src/menu/root/MenuRoot.test.tsx:1777-1792`); the width rule only modulates
  touch opens.
- External third-party lockers: three real-world lock mechanisms are simulated (react-remove-scroll
  via body attribute + stylesheet, silk-hq via body overflow shorthand, Ariakit via html overflow
  longhands — `packages/react/src/dialog/root/DialogRoot.test.tsx:1879-1913`), and the page must
  remain continuously locked from before the external unlock through the dialog's lifetime, then
  fully unlock on close (`packages/react/src/dialog/root/DialogRoot.test.tsx:1915-1965`). This
  suite is browser-only (`describe.skipIf(isJSDOM)` —
  `packages/react/src/dialog/root/DialogRoot.test.tsx:1878`).
- An external lock that cannot affect the page (`<body>` hidden while `<html>` owns the viewport)
  does not defer the unit's own lock (`packages/react/src/dialog/root/DialogRoot.test.tsx:1969-1996`).
- Re-render-triggered mount (controlled `value={[]}` forcing a Positioner mount) must not lock body
  or html scroll (`packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:11-64`,
  assertions at 60-63). This test runs in jsdom as well (not `skipIf(isJSDOM)`).
- Unmount/close cleanup: disabling the last enabled consumer removes all inline style and attribute
  mutations (dialog close, `packages/react/src/dialog/root/DialogRoot.test.tsx:1958-1964`; menubar
  handoff, `packages/react/src/menubar/Menubar.test.tsx:909-916`). Restoring the exact pre-lock
  inline style values and scroll positions is UNVERIFIED — inferred from
  `packages/utils/src/useScrollLock.ts:114-122,196-206`.
- Nested/simultaneous independent locks (two consumers locking at once, page staying locked until
  the last releases): UNVERIFIED — inferred from `packages/utils/src/useScrollLock.ts:234-247`; no
  corpus test holds two independent locks concurrently (the menubar handoff is sequential
  replacement, not nesting).
- Environment branches are UNVERIFIED: inset vs overlay scrollbar path selection
  (`packages/utils/src/useScrollLock.ts:24-31,290-300`), iOS/WebKit pinch-zoom bail-out
  (`packages/utils/src/useScrollLock.ts:100-103`), and `scrollbar-gutter: stable` capability
  probing (`packages/utils/src/useScrollLock.ts:33-62,151,158-163`); no test forces those paths.
- Resize-driven re-lock while enabled: UNVERIFIED — inferred from
  `packages/utils/src/useScrollLock.ts:208-214`; no test resizes the viewport mid-lock.

## Shared harness dependencies

- All corpus suites import the shared `#test-utils` harness (resolved to
  `packages/react/test/index.ts`, which re-exports `createRenderer`, `isJSDOM`, `wait`,
  `popupConformanceTests`, `firePointer`, and more):
  `packages/react/src/dialog/root/DialogRoot.test.tsx:7`,
  `packages/react/src/menu/root/MenuRoot.test.tsx:27`,
  `packages/react/src/select/root/SelectRoot.test.tsx:15`,
  `packages/react/src/popover/root/PopoverRoot.test.tsx:8`,
  `packages/react/src/combobox/root/ComboboxRoot.test.tsx:13`,
  `packages/react/src/menubar/Menubar.test.tsx:10`,
  `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:6`.
- `isJSDOM` from that harness gates the environment: every scroll-lock suite that measures real
  layout is `describe.skipIf(isJSDOM)` (Chromium-only) —
  `packages/react/src/dialog/root/DialogRoot.test.tsx:1878`,
  `packages/react/src/menu/root/MenuRoot.test.tsx:1755`,
  `packages/react/src/select/root/SelectRoot.test.tsx:1419`,
  `packages/react/src/menubar/Menubar.test.tsx:789`,
  `packages/react/src/popover/root/PopoverRoot.test.tsx:1586`,
  `packages/react/src/combobox/root/ComboboxRoot.test.tsx:4598` — while the ComboboxPositioner
  regression test runs in both environments (no skip at
  `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:11`).
