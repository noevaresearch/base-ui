# Dialog docs page content spec

Mined from `docs/src/app/(docs)/react/components/dialog/page.mdx` only. The component's own
behavior is covered by `specs/library/dialog/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/dialog/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/dialog/demos.json`).

## Page structure (headings, in order)

- `# Dialog` (h1) — `docs/src/app/(docs)/react/components/dialog/page.mdx:1`
- `## Usage guidelines` — `docs/src/app/(docs)/react/components/dialog/page.mdx:13`
- `## Anatomy` — `docs/src/app/(docs)/react/components/dialog/page.mdx:17`
- `## Examples` — `docs/src/app/(docs)/react/components/dialog/page.mdx:39`
  - `### State` — `docs/src/app/(docs)/react/components/dialog/page.mdx:41`
  - `### Open from a menu` — `docs/src/app/(docs)/react/components/dialog/page.mdx:98`
  - `### Nested dialogs` — `docs/src/app/(docs)/react/components/dialog/page.mdx:106`
  - `### Close confirmation` — `docs/src/app/(docs)/react/components/dialog/page.mdx:116`
  - `### Custom focus management` — `docs/src/app/(docs)/react/components/dialog/page.mdx:126`
  - `### Outside scroll dialog` — `docs/src/app/(docs)/react/components/dialog/page.mdx:136`
  - `### Inside scroll dialog` — `docs/src/app/(docs)/react/components/dialog/page.mdx:144`
  - `### Placing elements outside the popup` — `docs/src/app/(docs)/react/components/dialog/page.mdx:152`
  - `### Detached triggers` — `docs/src/app/(docs)/react/components/dialog/page.mdx:162`
  - `### Multiple triggers` — `docs/src/app/(docs)/react/components/dialog/page.mdx:191`
  - `### Controlled mode with multiple triggers` — `docs/src/app/(docs)/react/components/dialog/page.mdx:251`
- `## API reference` — `docs/src/app/(docs)/react/components/dialog/page.mdx:264`
  - `### Root` — `docs/src/app/(docs)/react/components/dialog/page.mdx:268`
  - `### Trigger` — `docs/src/app/(docs)/react/components/dialog/page.mdx:272`
  - `### Portal` — `docs/src/app/(docs)/react/components/dialog/page.mdx:276`
  - `### Backdrop` — `docs/src/app/(docs)/react/components/dialog/page.mdx:280`
  - `### Viewport` — `docs/src/app/(docs)/react/components/dialog/page.mdx:284`
  - `### Popup` — `docs/src/app/(docs)/react/components/dialog/page.mdx:288`
  - `### Title` — `docs/src/app/(docs)/react/components/dialog/page.mdx:292`
  - `### Description` — `docs/src/app/(docs)/react/components/dialog/page.mdx:296`
  - `### Close` — `docs/src/app/(docs)/react/components/dialog/page.mdx:300`
- `## createHandle` — `docs/src/app/(docs)/react/components/dialog/page.mdx:304`
  - `### Handle` — `docs/src/app/(docs)/react/components/dialog/page.mdx:310`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A popup that opens on top of the entire page." — `docs/src/app/(docs)/react/components/dialog/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React dialog component that opens on top of the entire page." — `docs/src/app/(docs)/react/components/dialog/page.mdx:4-7`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/dialog/page.mdx:9-11`
- Demo imports interleaved with the Examples subsections (`./demos/open-from-menu` at `docs/src/app/(docs)/react/components/dialog/page.mdx:102`, `./demos/nested` at `docs/src/app/(docs)/react/components/dialog/page.mdx:112`, `./demos/close-confirmation` at `docs/src/app/(docs)/react/components/dialog/page.mdx:122`, `./demos/focus-management` at `docs/src/app/(docs)/react/components/dialog/page.mdx:132`, `./demos/outside-scroll` at `docs/src/app/(docs)/react/components/dialog/page.mdx:140`, `./demos/inside-scroll` at `docs/src/app/(docs)/react/components/dialog/page.mdx:148`, `./demos/uncontained` at `docs/src/app/(docs)/react/components/dialog/page.mdx:158`, `./demos/detached-triggers-simple` at `docs/src/app/(docs)/react/components/dialog/page.mdx:187`, `./demos/detached-triggers-controlled` at `docs/src/app/(docs)/react/components/dialog/page.mdx:260`)
- `TypesDialog` import for the API reference — `docs/src/app/(docs)/react/components/dialog/page.mdx:266`
- `[//]: # '@exclude-table-of-contents'` marker placed after the `createHandle` reference and
  before the `### Handle` heading — `docs/src/app/(docs)/react/components/dialog/page.mdx:308`
- Trailing `export const metadata` SEO keywords block (15 keywords, e.g. 'React Dialog',
  'Nested Dialog React', 'Accessible Dialog') — `docs/src/app/(docs)/react/components/dialog/page.mdx:314-332`

## Prose claims about component behavior

- Page describes the component as "A popup that opens on top of the entire page." and, in the
  meta description, as "unstyled". Naming-level claim only; consistent with the popup-as-dialog
  role and portal-based overlay parts in behavior.md "Public API surface (props, parts,
  subcomponents)" and "DOM structure & portal behavior".
  `docs/src/app/(docs)/react/components/dialog/page.mdx:3`, `docs/src/app/(docs)/react/components/dialog/page.mdx:4-7`
- Usage guidelines: "Dialog doesn't support gestures: Use [Drawer] when you need gesture support
  or snap points. A panel that slides in from the edge of the screen and doesn't need gesture
  support is a positioned Dialog." Not covered by behavior.md (no test addresses gestures or
  Drawer); see Discrepancies. `docs/src/app/(docs)/react/components/dialog/page.mdx:15`
- Anatomy usage guidance: "Import the component and assemble its parts", showing the assembly
  `Dialog.Root` > (`Dialog.Trigger`, `Dialog.Portal` > (`Dialog.Backdrop`, `Dialog.Viewport` >
  `Dialog.Popup` > (`Dialog.Title`, `Dialog.Description`, `Dialog.Close`))), imported from the
  `@base-ui/react/dialog` namespace. The part set matches behavior.md "Public API surface
  (props, parts, subcomponents)"; the Backdrop-inside-Portal placement is consistent with
  behavior.md "DOM structure & portal behavior" (backdrop sits as a sibling before the popup),
  though behavior.md additionally records an automatically rendered internal backdrop under
  `modal={true}` that the page prose never mentions (see Discrepancies).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:19`, `docs/src/app/(docs)/react/components/dialog/page.mdx:21-37`
- "By default, Dialog is an uncontrolled component that manages its own state." Matches
  behavior.md "State model (controlled/uncontrolled, defaults, transitions)" (uncontrolled
  toggle via trigger/close).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:43`
- The uncontrolled example snippet renders `Dialog.Root` > `Dialog.Trigger` ("Open") >
  `Dialog.Portal` > `Dialog.Popup` containing `Dialog.Title` ("Example dialog") and
  `Dialog.Close` ("Close"), with no open-state props — an assembly-level illustration, not a
  new behavioral claim. `docs/src/app/(docs)/react/components/dialog/page.mdx:45-55`
- "Use `open` and `onOpenChange` props if you need to access or control the state of the dialog.
  For example, you can control the dialog state in order to open it imperatively from another
  place in your app." Matches behavior.md "State model (controlled/uncontrolled, defaults,
  transitions)" (controlled `open`/`onOpenChange` supported).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:57-58`
- The controlled example snippet closes the dialog from a form's async submit handler via
  `setOpen(false)` after `submitData()` — usage guidance consistent with behavior.md "Events"
  (`onOpenChange` is the state-setting callback); the async-submit flow itself is not exercised
  by any test recorded in behavior.md. `docs/src/app/(docs)/react/components/dialog/page.mdx:60-80`
- "It's also common to use `onOpenChange` if your app needs to do something when the dialog is
  closed or opened. This is recommended over `React.useEffect` when reacting to state changes."
  The `onOpenChange`-for-side-effects part matches behavior.md "Events"; the
  prefer-over-`React.useEffect` recommendation is guidance with no test counterpart; see
  Discrepancies. `docs/src/app/(docs)/react/components/dialog/page.mdx:82`
- The side-effects snippet shows an `onOpenChange` handler running `doStuff()` when `!open` and
  then calling `setOpen(open)` — a controlled-component illustration of the previous claim.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:84-96`
- "In order to open a dialog using a menu, control the dialog state and open it imperatively
  using the `onClick` handler on the menu item." Docs-only usage guidance; behavior.md has no
  menu-open test (its only Menu interaction is the Chromium-only focus-return edge case in
  "Focus management"). Not contradictory; see Discrepancies.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:100`
- "You can nest dialogs within one another normally." Matches behavior.md "Edge cases (rapid
  interactions, unmount, nesting)" (opening a new modal dialog does not dismiss a previous modal
  dialog underneath it). `docs/src/app/(docs)/react/components/dialog/page.mdx:108`
- "Use the `[data-nested-dialog-open]` selector and the `var(--nested-dialogs)` CSS variable to
  customize the styling of the parent dialog." Matches behavior.md "State model (controlled/
  uncontrolled, defaults, transitions)" (`data-nested-dialog-open` applied to the parent popup;
  `--nested-dialogs` counts currently-open descendant dialogs).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:110`
- "Backdrops of the child dialogs won't be rendered so that you can present the parent dialog in
  a clean way behind the one on top of it." Matches behavior.md "DOM structure & portal
  behavior" (a nested backdrop does not render when an ancestor dialog's backdrop is rendered;
  only the outermost backdrop renders).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:110`
- Close confirmation guidance: "both dialogs should be controlled. The confirmation dialog may be
  opened when `onOpenChange` callback of the parent dialog receives a request to close. This way,
  the confirmation is automatically shown when the user clicks the backdrop, presses the Esc key,
  or clicks a close button." Consistent with behavior.md "Events" (`onOpenChange` fires with
  reason `closePress` for backdrop/close-button dismissals and `escapeKey` for Esc).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:118-120`
- "You can control where the focus goes when the dialog opens and closes using the `initialFocus`
  and `finalFocus` props on the `<Dialog.Popup>` component." Matches behavior.md "Public API
  surface (props, parts, subcomponents)" (Dialog.Popup props include `initialFocus`, `finalFocus`)
  and "Focus management". `docs/src/app/(docs)/react/components/dialog/page.mdx:128`
- "You can also set these props to `false` to prevent focus from moving when the dialog opens or
  closes, or to a function that returns the element to focus based on the interaction type."
  Matches behavior.md "Focus management" (`initialFocus`/`finalFocus` accept ref | function |
  `false`; the function receives the interaction type — `'keyboard'`/`'touch'` for
  `initialFocus`, close type for `finalFocus`).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:130`
- Outside scroll pattern: "The dialog can be made scrollable by using `<Dialog.Viewport>` as an
  outer scrollable container for `<Dialog.Popup>` while the popup can extend past the bottom
  edge. The scrollable area uses the [Scroll Area component] to provide custom scrollbars."
  Docs-only layout pattern; behavior.md records no viewport-scrollability test (its ScrollArea
  coverage is the Chromium-only focus-trap case in "Focus management"). Not contradictory; see
  Discrepancies. `docs/src/app/(docs)/react/components/dialog/page.mdx:138`
- Inside scroll pattern: "The dialog can be made scrollable by making an inner container
  scrollable while the popup stays fully on screen. `<Dialog.Viewport>` is used as a positioning
  container for `<Dialog.Popup>`, while an inner scrollable area is created using the [Scroll
  Area component]." Same status as the outside-scroll pattern — docs-only layout guidance not
  covered by behavior.md. `docs/src/app/(docs)/react/components/dialog/page.mdx:146`
- "When adding elements that should appear 'outside' the colored popup area, continue to place
  them inside `<Dialog.Popup>`, but create a child element that has the popup styles. This
  ensures they are kept in the tab order and announced correctly by screen readers."
  Accessibility rationale not covered by behavior.md (no test exercises this layout pattern);
  see Discrepancies. `docs/src/app/(docs)/react/components/dialog/page.mdx:154`
- "`<Dialog.Popup>` has `pointer-events: none`, while inner content (the colored popup and close
  button) has `pointer-events: auto` so clicks on the backdrop continue to be registered."
  Internal-styling claim not covered by behavior.md (its outside-press tests verify that
  mousedown+click on the backdrop closes, but never assert pointer-events styling); see
  Discrepancies. `docs/src/app/(docs)/react/components/dialog/page.mdx:156`
- "A dialog can be controlled by a trigger located either inside or outside the `<Dialog.Root>`
  component. For simple, one-off interactions, place the `<Dialog.Trigger>` inside
  `<Dialog.Root>`" — matches behavior.md "DOM structure & portal behavior" (contained vs.
  detached trigger configurations).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:164-165`
- "However, if defining the dialog's content next to its trigger is not practical, you can use a
  detached trigger. This involves placing the `<Dialog.Trigger>` outside of `<Dialog.Root>` and
  linking them with a `handle` created by the `Dialog.createHandle()` function." Matches
  behavior.md "DOM structure & portal behavior" (detached triggers via a shared
  `Dialog.createHandle()` handle) and "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/dialog/page.mdx:167-168`
- "The imperative methods on the handle, such as `open()` and `openWithPayload()`, require a
  `<Dialog.Root>` using the same handle to be mounted. Calls made while no root is attached to
  the handle — before one mounts, or after it unmounts — are ignored. Each time a root mounts,
  it starts from fresh state: a call made while no root was attached is not replayed, and no
  open state carries over from a previous mount." Matches behavior.md "State model (controlled/
  uncontrolled, defaults, transitions)" (handle methods are no-ops before attach/after detach;
  remounting a handle-backed root resets stale payload/open state) and "Edge cases (rapid
  interactions, unmount, nesting)" (a root that unmounts while open and remounts starts fresh).
  The docs omit the dev-only console warning behavior.md records; see Discrepancies.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:170-171`
- "A single dialog can be opened by multiple trigger elements. You can achieve this by using the
  same `handle` for several detached triggers, or by placing multiple `<Dialog.Trigger>`
  components inside a single `<Dialog.Root>`." Matches behavior.md "State model (controlled/
  uncontrolled, defaults, transitions)" (multiple Triggers, contained or detached, can each open
  the shared dialog).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:193-194`
- "The dialog can render different content depending on which trigger opened it. This is
  achieved by passing a `payload` to the `<Dialog.Trigger>` and using the function-as-a-child
  pattern in `<Dialog.Root>`." Matches behavior.md "Public API surface (props, parts,
  subcomponents)" (Trigger `payload` prop; render-prop children receiving `{ payload }`).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:214-215`
- "The payload can be strongly typed by providing a type argument to the `createHandle()`
  function" (snippet uses `Dialog.createHandle<{ text: string }>()`). Matches behavior.md
  "Public API surface (props, parts, subcomponents)" (`Dialog.createHandle<TPayload>()`).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:217`, `docs/src/app/(docs)/react/components/dialog/page.mdx:219-249`
- "You can control the dialog's open state externally using the `open` and `onOpenChange` props
  on `<Dialog.Root>`. This allows you to manage the dialog's visibility based on your
  application's state. When using multiple triggers, you have to manage which trigger is active
  with the `triggerId` prop on `<Dialog.Root>` and the `id` prop on each `<Dialog.Trigger>`."
  Matches behavior.md "Public API surface (props, parts, subcomponents)" (`triggerId`,
  `defaultTriggerId`, Trigger `id`) and "State model (controlled/uncontrolled, defaults,
  transitions)" (a controlled `triggerId` sets the associated trigger's ARIA state and payload).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:253-255`
- "Note that there is no separate `onTriggerIdChange` prop. Instead, the `onOpenChange` callback
  receives an additional argument, `eventDetails`, which contains the trigger element that
  initiated the state change." Matches behavior.md "Events (names, payload shape, bubbling,
  preventDefault semantics)" (`eventDetails.trigger`, undefined when no trigger is associated).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:257-258`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the nine parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesDialog.Root />`,
  `### Trigger` → `<TypesDialog.Trigger />`, `### Portal` → `<TypesDialog.Portal />`,
  `### Backdrop` → `<TypesDialog.Backdrop />`, `### Viewport` → `<TypesDialog.Viewport />`,
  `### Popup` → `<TypesDialog.Popup />`, `### Title` → `<TypesDialog.Title />`,
  `### Description` → `<TypesDialog.Description />`, `### Close` → `<TypesDialog.Close />`.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:264-302`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesDialog } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:266`
- A separate `## createHandle` section renders `<TypesDialog.createHandle />` and a trailing
  `### Handle` subsection (after the `'@exclude-table-of-contents'` marker) renders
  `<TypesDialog.Handle />`. `docs/src/app/(docs)/react/components/dialog/page.mdx:304-312`
- Parts documented on this page: Root, Trigger, Portal, Backdrop, Viewport, Popup, Title,
  Description, Close — the same nine-part set recorded in behavior.md "Public API surface
  (props, parts, subcomponents)"; the additional `createHandle`/`Handle` entries match
  behavior.md's `Dialog.createHandle<TPayload>()` and handle-object coverage.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:268-312`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- Eight fenced code blocks:
  1. ` ```jsx title="Anatomy" ` — imports `{ Dialog }` from `@base-ui/react/dialog` and
     assembles Root > Trigger + Portal > (Backdrop, Viewport > Popup > (Title, Description,
     Close)). `docs/src/app/(docs)/react/components/dialog/page.mdx:21-37`
  2. ` ```tsx title="Uncontrolled dialog" ` — uncontrolled Root with Trigger/Portal/Popup/
     Title/Close and no open-state props. `docs/src/app/(docs)/react/components/dialog/page.mdx:45-55`
  3. ` ```tsx title="Controlled dialog" ` — `React.useState`-backed `open`/`onOpenChange` with a
     form whose async `onSubmit` closes the dialog via `setOpen(false)`.
     `docs/src/app/(docs)/react/components/dialog/page.mdx:60-80`
  4. ` ```tsx title="Running code when dialog state changes" ` — `onOpenChange` handler doing
     side effects on close and setting the new state.
     `docs/src/app/(docs)/react/components/dialog/page.mdx:84-96`
  5. ` ```jsx title="Detached triggers" ` — `Dialog.createHandle()` shared by a detached
     `<Dialog.Trigger handle={demoDialog}>` and `<Dialog.Root handle={demoDialog}>`, with
     `// @highlight` / `// @highlight-text "handle={demoDialog}"` directives.
     `docs/src/app/(docs)/react/components/dialog/page.mdx:173-185`
  6. ` ```jsx title="Multiple triggers within the Root part" ` — two `<Dialog.Trigger>` elements
     inside one `<Dialog.Root>`. `docs/src/app/(docs)/react/components/dialog/page.mdx:196-202`
  7. ` ```jsx title="Multiple detached triggers" ` — two detached triggers sharing one handle
     with one `<Dialog.Root handle={demoDialog}>`.
     `docs/src/app/(docs)/react/components/dialog/page.mdx:204-212`
  8. ` ```jsx title="Detached triggers with payload" ` — typed handle
     `Dialog.createHandle<{ text: string }>()`, two triggers passing distinct `payload` values,
     and `Dialog.Root` using the function-as-a-child pattern reading `{ payload }` to render
     trigger-specific content, with `// @highlight` / `// @highlight-text "payload"` directives.
     `docs/src/app/(docs)/react/components/dialog/page.mdx:219-249`
- No other code blocks exist in the page. The remaining example sections render demo components
  (`<DemoDialogOpenFromMenu />`, `<DemoDialogNested />`, `<DemoDialogCloseConfirmation />`,
  `<DemoDialogFocusManagement />`, `<DemoDialogOutsideScroll />`, `<DemoDialogInsideScroll />`,
  `<DemoDialogUncontained />`, `<DemoDialogDetachedTriggersSimple />`,
  `<DemoDialogDetachedTriggersControlled />`, plus `<DemoDialogHero />` at the top) imported
  from `./demos/*`; their code lives in demo files and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:9-11`, `docs/src/app/(docs)/react/components/dialog/page.mdx:102-104`, `docs/src/app/(docs)/react/components/dialog/page.mdx:112-114`, `docs/src/app/(docs)/react/components/dialog/page.mdx:122-124`, `docs/src/app/(docs)/react/components/dialog/page.mdx:132-134`, `docs/src/app/(docs)/react/components/dialog/page.mdx:140-142`, `docs/src/app/(docs)/react/components/dialog/page.mdx:148-150`, `docs/src/app/(docs)/react/components/dialog/page.mdx:158-160`, `docs/src/app/(docs)/react/components/dialog/page.mdx:187-189`, `docs/src/app/(docs)/react/components/dialog/page.mdx:260-262`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/dialog/behavior.md`.
Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: "Dialog doesn't support gestures", with the
  referral to Drawer for "gesture support or snap points" and the "positioned Dialog"
  characterization of non-gestured sliding panels. No test recorded in behavior.md addresses
  gestures or the Drawer component.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:15`
- Docs-only guidance, not covered by behavior.md: preferring `onOpenChange` over
  `React.useEffect` when reacting to dialog state changes.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:82`
- Docs-only usage guidance, not covered by behavior.md: opening a dialog from a menu via
  controlled state and the menu item's `onClick` handler. behavior.md's only Menu-related
  coverage is the Chromium-only focus-return edge case ("Focus management"), which tests a
  different aspect.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:100`
- Docs-only layout patterns, not covered by behavior.md: Viewport as an outer scrollable
  container with the popup extending past the bottom edge (`:138`), Viewport as a positioning
  container with an inner Scroll Area (`:146`), and the uncontained-popup pattern of placing
  visually-outside elements inside `Dialog.Popup` for tab-order/screen-reader correctness
  (`:154`).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:138`, `docs/src/app/(docs)/react/components/dialog/page.mdx:146`, `docs/src/app/(docs)/react/components/dialog/page.mdx:154`
- Docs-only styling claim, not covered by behavior.md: `Dialog.Popup` has `pointer-events: none`
  and inner content (colored popup + close button) has `pointer-events: auto` so backdrop clicks
  register. behavior.md verifies mousedown+click dismissal behavior but asserts nothing about
  pointer-events styling.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:156`
- Omission (docs page vs. behavior.md): the page says calls to handle imperative methods made
  while no root is attached "are ignored", but behavior.md "State model (controlled/
  uncontrolled, defaults, transitions)" records that such calls are no-ops *with a dev-only
  console warning* ("no root using this handle is mounted") that is suppressed in production.
  The page never mentions this warning — omission of a developer-facing caveat, not a
  contradiction of the ignored-call outcome.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:171`
- Note (not a mismatch): the Anatomy snippet renders an explicit `<Dialog.Backdrop />` inside
  `<Dialog.Portal>`, while behavior.md "DOM structure & portal behavior" additionally records an
  automatically rendered internal backdrop when `modal={true}`; the page prose never mentions
  the `modal` prop or the automatic backdrop (it is presumably covered by the generated
  Backdrop/Root API tables, which are out of scope here).
  `docs/src/app/(docs)/react/components/dialog/page.mdx:27`
- Note (not a mismatch): behavior.md "State model (controlled/uncontrolled, defaults,
  transitions)" records that the `--nested-dialogs` count includes cross-type nesting with
  `AlertDialog` and that `data-nested` is also applied to nested popups; the page mentions only
  `[data-nested-dialog-open]` on the parent and the CSS variable. Simplified-but-consistent
  presentation, not a contradiction.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:110`

## Cross-links to other docs pages

- `[Drawer](/react/components/drawer)` — recommended in Usage guidelines for gesture support or
  snap points. `docs/src/app/(docs)/react/components/dialog/page.mdx:15`
- `[Scroll Area component](/react/components/scroll-area)` — referenced in both the outside
  scroll dialog (`:138`) and inside scroll dialog (`:146`) examples.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:138`, `docs/src/app/(docs)/react/components/dialog/page.mdx:146`
- No external (non-Base UI) links appear on the page.
- Same-page imports (`./demos/hero`, `./demos/open-from-menu`, `./demos/nested`,
  `./demos/close-confirmation`, `./demos/focus-management`, `./demos/outside-scroll`,
  `./demos/inside-scroll`, `./demos/uncontained`, `./demos/detached-triggers-simple`,
  `./demos/detached-triggers-controlled`, and `./types`) are rendered/imported on this page
  itself; they are same-page modules, not cross-page links.
  `docs/src/app/(docs)/react/components/dialog/page.mdx:9`, `docs/src/app/(docs)/react/components/dialog/page.mdx:102`, `docs/src/app/(docs)/react/components/dialog/page.mdx:112`, `docs/src/app/(docs)/react/components/dialog/page.mdx:122`, `docs/src/app/(docs)/react/components/dialog/page.mdx:132`, `docs/src/app/(docs)/react/components/dialog/page.mdx:140`, `docs/src/app/(docs)/react/components/dialog/page.mdx:148`, `docs/src/app/(docs)/react/components/dialog/page.mdx:158`, `docs/src/app/(docs)/react/components/dialog/page.mdx:187`, `docs/src/app/(docs)/react/components/dialog/page.mdx:260`, `docs/src/app/(docs)/react/components/dialog/page.mdx:266`
