# Drawer docs page content spec

Mined from `docs/src/app/(docs)/react/components/drawer/page.mdx` only. The component's own
behavior is covered by `specs/library/drawer/behavior.md` (with part files under
`specs/library/drawer/parts/`) and is referenced here by section name instead of being restated.
Demo source files under `docs/src/app/(docs)/react/components/drawer/demos/` are out of scope for
this file (Stage 2 mines them into `specs/docs-content/drawer/demos.json`).

## Page structure (headings, in order)

- `# Drawer` (h1) — `docs/src/app/(docs)/react/components/drawer/page.mdx:1`
- `## Usage guidelines` — `docs/src/app/(docs)/react/components/drawer/page.mdx:13`
- `## Anatomy` — `docs/src/app/(docs)/react/components/drawer/page.mdx:17`
- `## Examples` — `docs/src/app/(docs)/react/components/drawer/page.mdx:51`
  - `### State` — `docs/src/app/(docs)/react/components/drawer/page.mdx:53`
  - `### Position` — `docs/src/app/(docs)/react/components/drawer/page.mdx:94`
  - `### Nested drawers` — `docs/src/app/(docs)/react/components/drawer/page.mdx:106`
  - `### Snap points` — `docs/src/app/(docs)/react/components/drawer/page.mdx:116`
  - `### Virtual keyboard aware` — `docs/src/app/(docs)/react/components/drawer/page.mdx:143`
  - `### Indent effect` — `docs/src/app/(docs)/react/components/drawer/page.mdx:161`
  - `### Non-modal` — `docs/src/app/(docs)/react/components/drawer/page.mdx:169`
  - `### Mobile navigation` — `docs/src/app/(docs)/react/components/drawer/page.mdx:177`
  - `### Swipe to open` — `docs/src/app/(docs)/react/components/drawer/page.mdx:185`
  - `### Close confirmation` — `docs/src/app/(docs)/react/components/drawer/page.mdx:193`
  - `### Action sheet with separate destructive action` — `docs/src/app/(docs)/react/components/drawer/page.mdx:205`
  - `### Detached triggers` — `docs/src/app/(docs)/react/components/drawer/page.mdx:213`
  - `### Stacking and animations` — `docs/src/app/(docs)/react/components/drawer/page.mdx:267`
- `## API reference` — `docs/src/app/(docs)/react/components/drawer/page.mdx:366`
  - `### Provider` — `docs/src/app/(docs)/react/components/drawer/page.mdx:370`
  - `### IndentBackground` — `docs/src/app/(docs)/react/components/drawer/page.mdx:374`
  - `### Indent` — `docs/src/app/(docs)/react/components/drawer/page.mdx:378`
  - `### Root` — `docs/src/app/(docs)/react/components/drawer/page.mdx:382`
  - `### Trigger` — `docs/src/app/(docs)/react/components/drawer/page.mdx:386`
  - `### SwipeArea` — `docs/src/app/(docs)/react/components/drawer/page.mdx:390`
  - `### VirtualKeyboardProvider` — `docs/src/app/(docs)/react/components/drawer/page.mdx:394`
  - `### Portal` — `docs/src/app/(docs)/react/components/drawer/page.mdx:398`
  - `### Backdrop` — `docs/src/app/(docs)/react/components/drawer/page.mdx:402`
  - `### Viewport` — `docs/src/app/(docs)/react/components/drawer/page.mdx:406`
  - `### Popup` — `docs/src/app/(docs)/react/components/drawer/page.mdx:410`
  - `### Content` — `docs/src/app/(docs)/react/components/drawer/page.mdx:414`
  - `### Title` — `docs/src/app/(docs)/react/components/drawer/page.mdx:418`
  - `### Description` — `docs/src/app/(docs)/react/components/drawer/page.mdx:422`
  - `### Close` — `docs/src/app/(docs)/react/components/drawer/page.mdx:426`
- `## createHandle` — `docs/src/app/(docs)/react/components/drawer/page.mdx:430`
  - `### Handle` — `docs/src/app/(docs)/react/components/drawer/page.mdx:436` (placed after the
    `@exclude-table-of-contents` comment so it stays with the createHandle docs entry)

Non-heading page furniture, in document order:

- `<Subtitle>` — "A panel that slides in from the edge of the screen." — `docs/src/app/(docs)/react/components/drawer/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React drawer component with swipe-to-dismiss gestures." — `docs/src/app/(docs)/react/components/drawer/page.mdx:4-7`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/drawer/page.mdx:9-11`
- Demo imports interleaved with the Examples subsections: `./demos/position` at `docs/src/app/(docs)/react/components/drawer/page.mdx:102-104`, `./demos/nested` at `docs/src/app/(docs)/react/components/drawer/page.mdx:112-114`, `./demos/snap-points` at `docs/src/app/(docs)/react/components/drawer/page.mdx:137-139`, `./demos/virtual-keyboard-aware` at `docs/src/app/(docs)/react/components/drawer/page.mdx:157-159`, `./demos/indent-provider` at `docs/src/app/(docs)/react/components/drawer/page.mdx:165-167`, `./demos/non-modal` at `docs/src/app/(docs)/react/components/drawer/page.mdx:173-175`, `./demos/mobile-nav` at `docs/src/app/(docs)/react/components/drawer/page.mdx:181-183`, `./demos/swipe-area` at `docs/src/app/(docs)/react/components/drawer/page.mdx:189-191`, `./demos/close-confirmation` at `docs/src/app/(docs)/react/components/drawer/page.mdx:201-203`, `./demos/uncontained` at `docs/src/app/(docs)/react/components/drawer/page.mdx:209-211`
- `TypesDrawer` import for the API reference — `docs/src/app/(docs)/react/components/drawer/page.mdx:368`
- `[//]: # '@exclude-table-of-contents'` comment before `### Handle` — `docs/src/app/(docs)/react/components/drawer/page.mdx:434`
- Trailing `export const metadata` SEO keywords block (12 keywords, e.g. 'React Drawer', 'Bottom Sheet', 'Swipe Dismiss Drawer') — `docs/src/app/(docs)/react/components/drawer/page.mdx:440-455`

## Prose claims about component behavior

- Page describes the component as "A panel that slides in from the edge of the screen." and, in
  the meta description, as "unstyled" with "swipe-to-dismiss gestures". Naming-level claim only;
  consistent with the part set and gesture engine in behavior.md "Part index" (`viewport` is "The
  gesture engine") and "Uniform DOM shell".
  `docs/src/app/(docs)/react/components/drawer/page.mdx:3`, `docs/src/app/(docs)/react/components/drawer/page.mdx:4-7`
- Usage guideline: "**Drawer extends [Dialog](/react/components/dialog):** It adds gesture support,
  snap points, and indent effects. If you don't need these, use Dialog instead. A panel that
  slides in from the edge of the screen and doesn't need gesture support is a positioned Dialog."
  The added capabilities match behavior.md "Part index" (viewport gesture engine, root snap-point
  state machine, Provider/Indent parts). The "extends Dialog" architectural relationship itself is
  not asserted in behavior.md; see Discrepancies.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:15`
- Anatomy usage guidance: "Import the component and assemble its parts", with the snippet (see
  Code snippets) assembling `Drawer.Provider` > `IndentBackground` > `Indent` > `Root` (with
  `Trigger`, `SwipeArea`, and `Portal` > `Backdrop` > `Viewport` > `Popup` > `Content` containing
  `Title`, `Description`, `Close`), imported from the `@base-ui/react/drawer` namespace. The part
  set and namespace import match behavior.md "Part index" and "Cross-cutting behavior" ("Uniform
  DOM shell"); `Title` and `Description` have no recorded behavior in behavior.md (see
  Discrepancies).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:19-45`
- "Drawer supports swipe gestures to dismiss. Set `swipeDirection` to control which direction
  dismisses the drawer." Matches behavior.md "Part index" (`root` `swipeDirection` values
  right/down/up; `viewport` swipe-to-dismiss from all four orientations).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:47`
- "`<Drawer.Content>` allows text selection of its children without swipe interference when using
  a mouse pointer." Matches behavior.md "Part index" (`viewport`: non-touch pointer drag support
  with the Content-boundary rule — mouse/pen swipes never start from `Drawer.Content` or its
  descendants — and active text selections exempt from swipe start).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:47`
- "Add `data-base-ui-swipe-ignore` to a descendant when you need to opt that element out of swipe
  dismissal for all input types." Matches behavior.md "Part index" (`viewport` exemptions list
  `data-base-ui-swipe-ignore`; `content-indent-provider` notes Content deliberately exposes no
  public swipe-ignore markers, so the attribute is consumer-added to a descendant).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:47`
- "Use `<Drawer.VirtualKeyboardProvider>` when a bottom sheet contains form fields and you want
  Base UI to manage keyboard-aware focus and scroll handling for software keyboards. Drawers
  without this provider are unaffected." Matches behavior.md "Part index"
  (`virtual-keyboard-provider`: opt-in, zero-prop mobile-keyboard coordination).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:49`
- "By default, Drawer is an uncontrolled component that manages its own state." Matches behavior.md
  "Part index" (`root`: closed by default; uncontrolled `defaultOpen` or controlled
  `open`/`onOpenChange`) and "Cross-cutting behavior" ("One open/snap state machine at the Root").
  `docs/src/app/(docs)/react/components/drawer/page.mdx:55`
- "Use `open` and `onOpenChange` props if you need to access or control the state of the drawer."
  Matches behavior.md "Cross-cutting behavior" ("One open/snap state machine at the Root").
  `docs/src/app/(docs)/react/components/drawer/page.mdx:73`
- "Positioning is handled by your styles. `swipeDirection` defaults to `"down"` for bottom sheets.
  Use `"up"`, `"left"`, or `"right"` for other drawer positions." The proven values are consistent
  with behavior.md "Part index" (`root`), but the default value `"down"` is not asserted in
  behavior.md (behavior.md records for `SwipeArea` only that the default behaves as
  upward-swipe-opens on a bottom drawer); see Discrepancies.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:96`
- "Use the `[data-nested-drawer-open]` selector and the `--nested-drawers` CSS variable to style
  drawers when a nested drawer is open." Matches behavior.md "Part index" (`popup`: parent popups
  render `data-nested-drawer-open` and `--nested-drawers`) and "Cross-cutting behavior"
  ("Nested-drawer coordination").
  `docs/src/app/(docs)/react/components/drawer/page.mdx:108`
- "This demo stacks nested drawers using a constant peek so the frontmost drawer stays anchored to
  the bottom while the ones behind it are scaled down and lifted. It also uses the
  `--drawer-height` and `--drawer-frontmost-height` CSS variables to handle varying drawer
  heights." Demo-implementation description (Stage 2 mines the demo itself); the named variables
  match behavior.md "Part index" (`popup`: border-inclusive `--drawer-frontmost-height`,
  Chromium-only `--drawer-height` lock).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:110`
- "Use `snapPoints` to snap a bottom sheet drawer to preset heights. Numbers between 0 and 1
  represent fractions of the viewport height, and numbers greater than 1 are treated as pixel
  values. String values support `px` and `rem` units (for example, `'148px'` or `'30rem'`)." The
  px/rem string support matches behavior.md "Part index" (`root` snap resolution); >1-as-pixels is
  consistent with numbers resolving as raw pixel heights clamped to `[0, popupHeight]`; the 0–1
  fraction-of-viewport-height semantics are not covered by behavior.md, which explicitly marks the
  fractional resolution base as never asserted (see Discrepancies).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:118`
- "Apply the snap point offset in your styles when using vertical drawers:" — introduces the CSS
  snippet; `--drawer-snap-point-offset` matches behavior.md "Part index" (`popup` renders it,
  `viewport` publishes movement vars).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:129`
- "By default, the drawer can skip snap points when swiping quickly. Specify the
  `snapToSequentialPoints` prop to disable velocity-based skipping so the snap target is
  determined by drag distance (you can still drag past multiple points)." Matches behavior.md
  "Part index" (`root`: `snapToSequentialPoints` flick/skip/nearest semantics; `viewport`:
  velocity-sampled release resolution) and the recorded slow drag past the last snap point
  settling on the nearest snap point in both modes.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:141`
- "Wrap a bottom sheet in `<Drawer.VirtualKeyboardProvider>` to make it react to software keyboards
  when it contains form controls. When the keyboard opens, the provider scrolls the body to keep
  the focused field visible." Matches behavior.md "Part index" (`virtual-keyboard-provider`:
  alignment scrolling centers the field in the visible band); the "scrolls the body" phrasing is
  looser than the recorded scroll-ancestor targeting (note, not a contradiction).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:145`
- "**Keep the popup frame stable:** place header and footer content outside a plain scrollable
  body." Layout usage guidance, docs-only (no counterpart in behavior.md).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:147`
- "**Lift a pinned footer input:** if a footer contains its own input, reserve a footer slot below
  the scroll area and offset it by `var(--drawer-keyboard-inset, 0px)`. The demo switches the
  focused footer to `position: fixed` — positioned against the popup, since its `transform`
  contains fixed descendants — and adds the inset to its bottom padding." Usage guidance; the
  `--drawer-keyboard-inset` variable matches behavior.md "Part index" (`virtual-keyboard-provider`
  computes it on the viewport); the demo's `position: fixed` technique is demo-implementation
  description (Stage 2 scope).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:148`
- "**Always include the `0px` fallback:** the provider only sets `--drawer-keyboard-inset` while
  the keyboard is aligned, so a bare `var(--drawer-keyboard-inset)` is invalid before the first
  alignment and after cleanup." The set-only-while-aligned claim is in tension with the states
  behavior.md covers (see Discrepancies).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:149`
- "Scale the background down when any drawer opens by wrapping your app in `<Drawer.Provider>` and
  use `<Drawer.IndentBackground>` + `<Drawer.Indent>` at the top of your tree. Any `<Drawer.Root>`
  within the provider notifies it when it mounts, which activates the indent parts (they receive
  `[data-active]` state attributes)." Matches behavior.md "Part index" (`content-indent-provider`:
  Provider is a registry of open drawers active while ≥1 is open, `IndentBackground` mirrors it
  with `data-active`/`data-inactive`, registration lands before passive effects flush).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:163`
- "Set `modal={false}` to opt out of focus trapping and `disablePointerDismissal` to keep the
  drawer open on outside clicks." `modal={false}` is a proven Root prop in behavior.md "Part
  index" (`root` Public API surface), but focus-trap opt-out behavior is not asserted anywhere in
  behavior.md, and `disablePointerDismissal` does not appear in behavior.md at all; see
  Discrepancies.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:171`
- "You can build a full-screen mobile navigation sheet using Drawer parts, including a
  flick-to-dismiss from the top gesture." Usage guidance describing the mobile-nav demo;
  consistent with the four-orientation gesture support in behavior.md "Part index" (`viewport`),
  though the demo's specific gesture is not separately covered.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:179`
- "Place `<Drawer.SwipeArea>` along the edge of the viewport to enable swipe-to-open gestures."
  Matches behavior.md "Part index" (`swipe-area`: opens the drawer by swiping from its own
  footprint, even with no popup rendered).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:187`
- "This example shows a nested confirmation dialog that opens if the text entered in the drawer is
  going to be discarded." Demo description.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:195`
- "To implement this, both the drawer and the confirmation dialog should be controlled. The
  confirmation dialog may be opened when the `onOpenChange` callback of the drawer receives a
  request to close while there is text in the textarea. This way, the confirmation is
  automatically shown when the user clicks the backdrop, presses the Esc key, clicks a close
  button, or dismisses the drawer with a swipe gesture." Usage guidance consistent with behavior.md
  "Cross-cutting behavior" ("One open/snap state machine at the Root": gesture surfaces request
  transitions through `onOpenChange` with a `reason`) and "Part index" (`root`: reasons include
  `swipe`, `closeWatcher`, `closePress`, `none` — covering swipe, Esc/CloseWatcher, and
  close-press dismissals).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:197`
- "Use `eventDetails.cancel()` in `onOpenChange` to prevent the drawer from closing while the
  confirmation prompt is shown." Matches behavior.md "Cross-cutting behavior" ("One open/snap state
  machine at the Root": a `cancel()` keeps state intact and rolls back transient swipe styles).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:199`
- "This demo builds an action sheet with a grouped list of actions plus a separate destructive
  action button." Demo description only.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:207`
- "A drawer can be controlled by a trigger located either inside or outside the `<Drawer.Root>`
  component. For simple, one-off interactions, place the `<Drawer.Trigger>` inside
  `<Drawer.Root>`."
  `docs/src/app/(docs)/react/components/drawer/page.mdx:215`
- "However, if defining the drawer's content next to its trigger is not practical, you can use a
  detached trigger. This involves placing the `<Drawer.Trigger>` outside of `<Drawer.Root>` and
  linking them with a `handle` created by the `Drawer.createHandle()` function." Matches
  behavior.md "Part index" (`root`: `Drawer.createHandle()` drives the same state as triggers).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:217`
- "The imperative methods on the handle, such as `open()` and `openWithPayload()`, require a
  `<Drawer.Root>` using the same handle to be mounted. Calls made while no root is attached to the
  handle — before one mounts, or after it unmounts — are ignored. Each time a root mounts, it
  starts from fresh state: a call made while no root was attached is not replayed, and no open
  state carries over from a previous mount." The no-op and fresh-state outcomes match behavior.md
  "Part index" (`root`: unmounted-handle calls no-op, remount attaches fresh state with no
  retained payload), but the docs omit that each such call also warns in development; see
  Discrepancies.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:219-220`
- "The drawer can render different content depending on which trigger opened it. This is achieved
  by passing a `payload` to the `<Drawer.Trigger>` and using the function-as-a-child pattern in
  `<Drawer.Root>`." Matches behavior.md "Part index" (`root`: render-prop child receiving
  `{ payload }`).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:236`
- "Use CSS transitions or animations to animate drawer opening, closing, swipe interactions, and
  nested stacking. The `data-starting-style` attribute is applied when a drawer starts to open,
  and `data-ending-style` is applied when it starts to close." `data-ending-style` matches
  behavior.md "Cross-cutting behavior" ("A published visual-state graph"; real-animation coverage
  of `data-ending-style` timing); `data-starting-style` does not appear in behavior.md; see
  Discrepancies.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:269`
- "The `--nested-drawers` CSS variable can be used to determine stack depth. The frontmost drawer
  has index `0`." The variable matches behavior.md "Part index" (`popup`); the frontmost-is-0
  indexing convention is not asserted in behavior.md; see Discrepancies.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:271`
- "When stacked drawers have varying heights, use the `--drawer-height` and
  `--drawer-frontmost-height` variables to keep collapsed drawers aligned with the frontmost one."
  Matches behavior.md "Part index" (`popup`: `--drawer-frontmost-height` border-inclusive,
  `--drawer-height` lock surviving close/reopen) and "Cross-cutting behavior" ("Nested-drawer
  coordination").
  `docs/src/app/(docs)/react/components/drawer/page.mdx:281`
- "The `data-nested-drawer-open` attribute marks drawers behind the frontmost drawer. Use it with
  `data-nested-drawer-swiping` to dim or hide parent drawer content while keeping it visible
  during nested swipe interactions." Matches behavior.md "Part index" (`popup`:
  `data-nested-drawer-open`) and "Cross-cutting behavior" ("A published visual-state graph":
  parent popups' `data-nested-drawer-swiping`).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:299`
- "The `--drawer-swipe-movement-x`, `--drawer-swipe-movement-y`, and `--drawer-snap-point-offset`
  CSS variables can be used to create smooth drag and snap offsets" Matches behavior.md
  "Cross-cutting behavior" ("A published visual-state graph": popup movement vars +
  `--drawer-snap-point-offset`, all cleared deterministically).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:317`
- "The `data-swipe-direction` attribute can be used with `data-ending-style` to animate
  directional dismissal" Matches behavior.md "Part index" (`popup`: renders
  `data-swipe-direction`) combined with the recorded `data-ending-style` coverage.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:333`
- "Use `--drawer-swipe-progress` to fade the backdrop as the drawer is swiped, and
  `--drawer-swipe-strength` to scale release transition durations based on swipe velocity."
  `--drawer-swipe-progress` matches behavior.md "Cross-cutting behavior" ("A published
  visual-state graph": backdrop `--drawer-swipe-progress`); `--drawer-swipe-strength` does not
  appear in behavior.md; see Discrepancies.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:347`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the fifteen parts, each as its own heading rendering a
  generated reference component: `### Provider` → `<TypesDrawer.Provider />`,
  `### IndentBackground` → `<TypesDrawer.IndentBackground />`, `### Indent` →
  `<TypesDrawer.Indent />`, `### Root` → `<TypesDrawer.Root />`, `### Trigger` →
  `<TypesDrawer.Trigger />`, `### SwipeArea` → `<TypesDrawer.SwipeArea />`,
  `### VirtualKeyboardProvider` → `<TypesDrawer.VirtualKeyboardProvider />`, `### Portal` →
  `<TypesDrawer.Portal />`, `### Backdrop` → `<TypesDrawer.Backdrop />`, `### Viewport` →
  `<TypesDrawer.Viewport />`, `### Popup` → `<TypesDrawer.Popup />`, `### Content` →
  `<TypesDrawer.Content />`, `### Title` → `<TypesDrawer.Title />`, `### Description` →
  `<TypesDrawer.Description />`, `### Close` → `<TypesDrawer.Close />`.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:366-428`
- A separate `## createHandle` section documents the factory via `<TypesDrawer.createHandle />`,
  followed by the `@exclude-table-of-contents` comment and a `### Handle` section rendering
  `<TypesDrawer.Handle />` (documenting the handle object returned by `Drawer.createHandle()`).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:430-438`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesDrawer } from './types';`); no props, prop types, defaults, or prop descriptions
  are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:368`
- Parts documented on this page: Provider, IndentBackground, Indent, Root, Trigger, SwipeArea,
  VirtualKeyboardProvider, Portal, Backdrop, Viewport, Popup, Content, Title, Description, Close,
  plus the createHandle/Handle API. The Provider/Indent/IndentBackground/Content/SwipeArea/
  VirtualKeyboardProvider/Root/Popup/Viewport set matches behavior.md "Part index"; Trigger,
  Portal, Backdrop, Title, Description, and Close appear in behavior.md "Cross-cutting behavior"
  ("Uniform DOM shell") without dedicated part specs.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:366-438`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- Anatomy (` ```jsx title="Anatomy" `): imports `{ Drawer }` from `@base-ui/react/drawer` and
  assembles `Drawer.Provider` > `Drawer.IndentBackground` > `Drawer.Indent` > `Drawer.Root` (with
  `Drawer.Trigger` and `Drawer.SwipeArea` as Root children, before `Drawer.Portal` containing
  `Drawer.Backdrop` > `Drawer.Viewport` > `Drawer.Popup` > `Drawer.Content` > `Drawer.Title`,
  `Drawer.Description`, `Drawer.Close`).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:21-45`
- Uncontrolled drawer (` ```tsx title="Uncontrolled drawer" `): bare `Drawer.Root` with Trigger,
  Portal > Viewport > Popup > Content > Title + Close; no state props.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:57-71`
- Controlled drawer (` ```tsx title="Controlled drawer" `): `React.useState(false)` passed as
  `open={open} onOpenChange={setOpen}`, matching behavior.md "Cross-cutting behavior" ("One
  open/snap state machine at the Root") controlled pattern.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:75-92`
- Swipe directions (` ```tsx title="Swipe directions" `): one-line `<Drawer.Root
  swipeDirection="right">` fragment.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:98-100`
- Snap points (` ```tsx title="Snap points" `): `const snapPoints = ['148px', 1]`, state typed
  `Drawer.Root.SnapPoint | null`, and `<Drawer.Root snapPoints={snapPoints} snapPoint={snapPoint}
  onSnapPointChange={setSnapPoint}>` — matches behavior.md "Part index" (`root` controlled
  snap-point API and the `Drawer.Root.SnapPoint` state type).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:120-127`
- Snap point offset (` ```css title="Snap point offset" `): popup `transform: translateY(calc(var(--drawer-snap-point-offset) + var(--drawer-swipe-movement-y)))`.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:131-135`
- Virtual keyboard aware drawer (` ```tsx title="Virtual keyboard aware drawer" `):
  `<Drawer.Root>` wrapping `<Drawer.VirtualKeyboardProvider>`.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:151-155`
- Detached triggers (` ```jsx title="Detached triggers" `): `const demoDrawer =
  Drawer.createHandle();` shared by `<Drawer.Trigger handle={demoDrawer}>` and `<Drawer.Root
  handle={demoDrawer}>`; contains `@highlight`/`@highlight-text` directives.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:222-234`
- Detached triggers with payload (` ```jsx title="Detached triggers with payload" `): typed
  `Drawer.createHandle<{ title: string }>()`, two `Drawer.Trigger`s with distinct `payload`
  objects, and a function-as-a-child `Drawer.Root` rendering `payload?.title`; contains
  `@highlight`/`@highlight-text` directives.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:238-265`
- Stack depth (` ```css title="Stack depth" `): `--stack-scale: calc(1 - (var(--nested-drawers) *
  var(--stack-step)))` composed with `--drawer-swipe-movement-y` in the popup transform.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:273-279`
- Variable-height stacking (` ```css title="Variable-height stacking" `): popup height from
  `--drawer-height` / `--drawer-frontmost-height` with a `[data-nested-drawer-open]` variant
  clipping overflow.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:283-297`
- Nested content visibility (` ```css title="Nested content visibility" `): opacity transitions on
  `.DrawerContent` under `[data-nested-drawer-open]` and
  `[data-nested-drawer-open][data-nested-drawer-swiping]`; contains `@highlight-text` directives.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:301-315`
- Swipe and snap offset (` ```css title="Swipe and snap offset" `): `[data-swipe-direction='right']`
  uses `--drawer-swipe-movement-x`; `[data-swipe-direction='down']` combines
  `--drawer-snap-point-offset` + `--drawer-swipe-movement-y`; contains `@highlight-text`
  directives.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:319-331`
- Swipe dismissal direction (` ```css title="Swipe dismissal direction" `):
  `[data-ending-style][data-swipe-direction='right'/'down']` translating the popup 100% off; contains
  `@highlight-text` directives.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:335-345`
- Backdrop and release timing (` ```css title="Backdrop and release timing" `): backdrop opacity
  scaled by `1 - var(--drawer-swipe-progress)`, `[data-ending-style]` transition duration scaled
  by `var(--drawer-swipe-strength) * 400ms`, and `[data-swiping]` duration forced to `0ms`.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:349-364`
- No other code blocks exist in the page; all other examples render demo components
  (`<DemoDrawerHero />`, `<DemoDrawerPosition />`, `<DemoDrawerNested />`,
  `<DemoDrawerSnapPoints />`, `<DemoDrawerVirtualKeyboardAware />`,
  `<DemoDrawerIndentProvider />`, `<DemoDrawerNonModal />`, `<DemoDrawerMobileNav />`,
  `<DemoDrawerSwipeArea />`, `<DemoDrawerCloseConfirmation />`, `<DemoDrawerUncontained />`)
  imported from `./demos/*`; their code lives in demo files and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:9-11`, `docs/src/app/(docs)/react/components/drawer/page.mdx:102-104`, `docs/src/app/(docs)/react/components/drawer/page.mdx:112-114`, `docs/src/app/(docs)/react/components/drawer/page.mdx:137-139`, `docs/src/app/(docs)/react/components/drawer/page.mdx:157-159`, `docs/src/app/(docs)/react/components/drawer/page.mdx:165-167`, `docs/src/app/(docs)/react/components/drawer/page.mdx:173-175`, `docs/src/app/(docs)/react/components/drawer/page.mdx:181-183`, `docs/src/app/(docs)/react/components/drawer/page.mdx:189-191`, `docs/src/app/(docs)/react/components/drawer/page.mdx:201-203`, `docs/src/app/(docs)/react/components/drawer/page.mdx:209-211`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions of recorded outcomes found between the page prose and
`specs/library/drawer/behavior.md`. Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: the `disablePointerDismissal` prop ("keep the
  drawer open on outside clicks"). The prop name does not appear anywhere in behavior.md, and
  outside-click dismissal itself is not a recorded behavior there.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:171`
- Docs-only claim, not covered by behavior.md: `modal={false}` "opt[ing] out of focus trapping".
  The prop is exercised in behavior.md "Part index" (`root` Public API surface lists
  `modal={false}`; `viewport` consumes `modal`), but no drawer test recorded there asserts focus
  trapping or its opt-out (the popup part explicitly notes no Escape/Tab trapping proven, and the
  swipe-area part notes no focus trap asserted).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:171`
- Docs-only claim, not covered by behavior.md: snap-point fraction semantics — "Numbers between 0
  and 1 represent fractions of the viewport height". behavior.md "Part index" (`root` snap
  resolution) records numeric values resolving as raw pixel heights clamped to
  `[0, popupHeight]` and explicitly marks the fractional (`0 < n <= 1`) resolution base as
  UNVERIFIED — never asserted.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:118`
- Docs-only claim, not covered by behavior.md: "`swipeDirection` defaults to `"down"` for bottom
  sheets". behavior.md proves the `'right'`/`'down'`/`'up'` values but never asserts the Root
  default; the closest record is for `SwipeArea`, described as "the default (upward swipe opens —
  bottom drawer)" without naming the default value.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:96`
- Docs-only claim, not covered by behavior.md: `data-starting-style` "applied when a drawer starts
  to open". Only `data-ending-style` appears in behavior.md ("Cross-cutting behavior" and the
  real-animation gating note); `data-starting-style` has no recorded counterpart.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:269`
- Docs-only claim, not covered by behavior.md: `--drawer-swipe-strength` "to scale release
  transition durations based on swipe velocity". The variable does not appear anywhere in
  behavior.md.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:347`
- Docs-only claim, not covered by behavior.md: "The frontmost drawer has index `0`" for
  `--nested-drawers`. behavior.md records that parent popups render `--nested-drawers` but never
  asserts the indexing convention.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:271`
- Omission (docs page vs. behavior.md): detached-handle calls with no mounted root are described
  only as "ignored", while behavior.md "Part index" (`root` detached-handle API) records that each
  such call also emits a console warning ("no root using this handle is mounted") in addition to
  no-op'ing. Omission of a developer-facing warning, not a contradiction of the no-op outcome.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:220`
- Tension (docs page vs. behavior.md): the page claims the provider "only sets
  `--drawer-keyboard-inset` while the keyboard is aligned, so a bare `var(--drawer-keyboard-inset)`
  is invalid before the first alignment and after cleanup". behavior.md ("Part index",
  `virtual-keyboard-provider`) records the variable present as `'0px'` when the keyboard is not
  open and returning to `'0px'` on blur — i.e. still set to a valid value in the cleanup states it
  covers — so "invalid after cleanup" is at odds with the recorded closed/blur states; the
  "before the first alignment" absent-state itself is not covered by behavior.md.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:149`
- Docs-level claim, not derivable from behavior.md: "Drawer extends Dialog". behavior.md never
  states an inheritance/composition relationship with Dialog; it only records that popup is
  `role="dialog"` ("Uniform DOM shell") and that Dialog/AlertDialog popups are excluded from the
  nested-drawer count ("Part index" `popup`, "Nested-drawer coordination"). Consistent with, but
  not proven by, behavior.md.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:15`
- Omission (docs page vs. behavior.md): the Anatomy and snippets use `Drawer.Title` and
  `Drawer.Description`, and the API reference documents both, but behavior.md's "Uniform DOM
  shell" records only `Popup (> Content, Close)` and no part file covers Title/Description —
  they have no recorded behavior (keyboard, focus, ARIA, or otherwise).
  `docs/src/app/(docs)/react/components/drawer/page.mdx:21-45`, `docs/src/app/(docs)/react/components/drawer/page.mdx:418-424`
- Note (not a mismatch): "the provider scrolls the body to keep the focused field visible" is
  looser than behavior.md's "Part index" (`virtual-keyboard-provider`) alignment model — bounded
  alignment scrolling that centers the field in the visible band, with scroll slack added to the
  field's scroll ancestor; no contradiction, but the docs phrasing should not be read as a
  `document.body` scroll guarantee.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:145`

## Cross-links to other docs pages

- One internal link: the Usage guidelines bullet links to the Dialog component page,
  `[Dialog](/react/components/dialog)`.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:15`
- N/A otherwise — the page contains no other internal links and no external links.
- Demo components (`./demos/hero`, `./demos/position`, `./demos/nested`, `./demos/snap-points`,
  `./demos/virtual-keyboard-aware`, `./demos/indent-provider`, `./demos/non-modal`,
  `./demos/mobile-nav`, `./demos/swipe-area`, `./demos/close-confirmation`,
  `./demos/uncontained`) are imported and rendered on this page itself; they are same-page
  imports, not cross-page links.
  `docs/src/app/(docs)/react/components/drawer/page.mdx:9`, `docs/src/app/(docs)/react/components/drawer/page.mdx:102`, `docs/src/app/(docs)/react/components/drawer/page.mdx:112`, `docs/src/app/(docs)/react/components/drawer/page.mdx:137`, `docs/src/app/(docs)/react/components/drawer/page.mdx:157`, `docs/src/app/(docs)/react/components/drawer/page.mdx:165`, `docs/src/app/(docs)/react/components/drawer/page.mdx:173`, `docs/src/app/(docs)/react/components/drawer/page.mdx:181`, `docs/src/app/(docs)/react/components/drawer/page.mdx:189`, `docs/src/app/(docs)/react/components/drawer/page.mdx:201`, `docs/src/app/(docs)/react/components/drawer/page.mdx:209`
