# Alert Dialog docs-content page spec (Stage 1 docs)

Mined from the docs page `.mdx` only. Component behavior is NOT re-derived here — cross-checks
reference `specs/library/alert-dialog/behavior.md` by section name, and cite it by path/lines
only where a specific discrepancy hinges on an exact recorded claim. Demo internals are out of
scope (Stage 2 mines `specs/docs-content/alert-dialog/demos.json`).

## Page structure (headings, in order)

- `# Alert Dialog` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:13`
- `## Examples` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:35`
  - `### Open from a menu` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:37`
  - `### Close confirmation` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:45`
  - `### Detached triggers` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:57`
  - `### Multiple triggers` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:86`
  - `### Controlled mode with multiple triggers` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:146`
- `## API reference` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:159`
  - `### Root` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:163`
  - `### Trigger` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:167`
  - `### Portal` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:171`
  - `### Backdrop` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:175`
  - `### Viewport` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:179`
  - `### Popup` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:183`
  - `### Title` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:187`
  - `### Description` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:191`
  - `### Close` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:195`
- `## createHandle` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:199`
  - `### Handle` — `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:205` (kept out of the
    table of contents by an `@exclude-table-of-contents` marker directly before it —
    `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:203`)

Non-heading page furniture, in document order: `<Subtitle>` tagline plus `<Meta>` description
(`docs/src/app/(docs)/react/components/alert-dialog/page.mdx:3-7`); hero demo import/render
(`docs/src/app/(docs)/react/components/alert-dialog/page.mdx:9-11`); demo imports/renders inside
each Examples subsection (`docs/src/app/(docs)/react/components/alert-dialog/page.mdx:41-43`,
`docs/src/app/(docs)/react/components/alert-dialog/page.mdx:53-55`,
`docs/src/app/(docs)/react/components/alert-dialog/page.mdx:82-84`,
`docs/src/app/(docs)/react/components/alert-dialog/page.mdx:155-157`); generated API table
renders (`docs/src/app/(docs)/react/components/alert-dialog/page.mdx:161-207`); and an exported
`metadata` keywords block (`docs/src/app/(docs)/react/components/alert-dialog/page.mdx:209-223`).

## Prose claims about component behavior

- Tagline: "A dialog that requires a user response to proceed."
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:3`; meta description repeats it as
  "a high-quality, unstyled React alert dialog component that requires a user response to
  proceed" `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:4-7`.
  Cross-check: supported in spirit by behavior.md "State model" (pointer dismissal disabled) and
  "Accessibility" (`role="alertdialog"`), though behavior.md never uses the "requires a user
  response" phrasing. No contradiction.
- Anatomy: "Import the component and assemble its parts"
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:15`; the snippet imports from the
  `@base-ui/react/alert-dialog` namespace and nests Root → Trigger → Portal → Backdrop →
  Viewport → Popup → Title/Description/Close
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:17-33`.
  Cross-check: consistent with behavior.md "Public API surface", which lists the same nine parts
  via namespace import. No contradiction.
- Open from a menu: "control the dialog state and open it imperatively using the `onClick`
  handler on the menu item" `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:39`.
  Cross-check: no counterpart in behavior.md (Menu integration is not exercised by the behavior
  suite). Not a contradiction, just untested territory.
- Close confirmation: "a nested confirmation dialog that opens if the text entered in the parent
  dialog is going to be discarded" `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:47`;
  "both dialogs should be controlled. The confirmation dialog may be opened when `onOpenChange`
  callback of the parent dialog receives a request to close. This way, the confirmation is
  automatically shown when the user clicks the backdrop, presses the Esc key, or clicks a close
  button" `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:49`.
  Cross-check: MISMATCH with behavior.md "State model" — see Discrepancies #1.
- Close confirmation styling: use the `[data-nested-dialog-open]` selector and the
  `var(--nested-dialogs)` CSS variable to customize the parent dialog; "Backdrops of the child
  dialogs won't be rendered" `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:51`.
  Cross-check: no counterpart in behavior.md, which explicitly marks nested dialogs untested —
  see Discrepancies #3.
- Detached triggers: a trigger may sit "either inside or outside the `<AlertDialog.Root>`"
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:59`; for simple one-off
  interactions place the Trigger inside the Root, "as shown in the example at the top of this
  page" `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:60`; otherwise place the
  Trigger outside the Root "and linking them with a `handle` created by the
  `AlertDialog.createHandle()` function"
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:62-63`.
  Cross-check: consistent with behavior.md "Public API surface" (handle wiring for detached
  triggers) and "State model" (detached-trigger suites). No contradiction.
- Handle imperative methods "such as `open()` and `openWithPayload()`" "require an
  `<AlertDialog.Root>` using the same handle to be mounted"
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:65`.
  Cross-check: behavior.md "State model" documents `handle.open`/`openWithPayload`/`close` but
  never asserts the mount requirement itself. Not contradicted, just uncovered.
- Handle calls with no root attached — before mount or after unmount — are ignored and not
  replayed; "no open state carries over from a previous mount"
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:66`.
  Cross-check: not asserted anywhere in behavior.md, and its "Edge cases" entry about
  unmount-while-open records `handle.isOpen` staying `true` across remount — see
  Discrepancies #2.
- Multiple triggers: "A single alert dialog can be opened by multiple trigger elements" via "the
  same `handle` for several detached triggers, or by placing multiple `<AlertDialog.Trigger>`
  components inside a single `<AlertDialog.Root>`"
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:88-89`.
  Cross-check: consistent with behavior.md "State model" (multiple triggers within one Root or
  detached via a handle; every cycle works with every trigger). No contradiction.
- Per-trigger content: "The alert dialog can render different content depending on which trigger
  opened it", achieved "by passing a `payload` to the `<AlertDialog.Trigger>` and using the
  function-as-a-child pattern in `<AlertDialog.Root>`"
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:109-110`; "The payload can be
  strongly typed by providing a type argument to the `createHandle()` function"
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:112`.
  Cross-check: payload/render-prop consistent with behavior.md "Public API surface"
  (children-as-function receiving `{ payload }`); the typing claim has no counterpart there.
  No contradiction.
- Controlled mode: open state controlled externally via `open` and `onOpenChange` on
  `<AlertDialog.Root>` `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:148-149`;
  with multiple triggers, "you have to manage which trigger is active with the `triggerId` prop
  on `<AlertDialog.Root>` and the `id` prop on each `<AlertDialog.Trigger>`"
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:150`.
  Cross-check: consistent with behavior.md "Public API surface" (`triggerId`/`defaultTriggerId`
  select the active trigger; Trigger `id` exercised) and "Accessibility" (open-state ARIA sync
  to the active trigger). No contradiction.
- "Note that there is no separate `onTriggerIdChange` prop"
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:152`.
  Cross-check: behavior.md documents no such prop either — consistent absence.
- "the `onOpenChange` callback receives an additional argument, `eventDetails`, which contains
  the trigger element that initiated the state change"
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:153`.
  Cross-check: consistent with behavior.md "Events" (`details.trigger` is the initiating trigger
  element, kept across close presses). No contradiction.

## API tables referenced (props/parts documented on this page)

- The API reference section renders generated table components imported from `./types`
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:161`; the `.mdx` itself contains
  no inline prop tables. Parts documented via heading + `<TypesAlertDialog.* />`: Root, Trigger,
  Portal, Backdrop, Viewport, Popup, Title, Description, Close
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:163-197`.
- The `## createHandle` section documents the factory
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:199-201` and a separate `Handle`
  type section `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:205-207`.
- Props named in page prose (outside the generated tables): Root `handle`
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:77`; Trigger `handle`
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:73`; Trigger `payload`
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:120-126`; Root
  `open`/`onOpenChange` `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:148`; Root
  `triggerId` and Trigger `id` `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:150`;
  `AlertDialog.createHandle()` with optional type argument
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:63-116`.
- Behavior-documented API with no prose mention on this page: `defaultOpen`, `defaultTriggerId`,
  `onOpenChangeComplete`, `actionsRef` (+ `Actions.unmount()`/`close()`), `details.
  preventUnmountOnClose()`, and handle `close()` — all recorded in behavior.md "Public API
  surface", "State model", and "Events"
  `specs/library/alert-dialog/behavior.md:22-38`,
  `specs/library/alert-dialog/behavior.md:59-72`. Whether the generated tables cover them is out
  of scope here (the `./types` module was not opened).

## Code snippets embedded directly in the .mdx (not pulled from demos/)

1. "Anatomy" `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:17-33` — namespace
   import plus full part assembly (Root, Trigger, Portal, Backdrop, Viewport, Popup, Title,
   Description, Close).
2. "Detached triggers" `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:68-80` —
   `AlertDialog.createHandle()`, `<AlertDialog.Trigger handle={...}>`,
   `<AlertDialog.Root handle={...}>`; annotated with `@highlight`/`@highlight-text` markers
   `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:71-76`.
3. "Multiple triggers within the Root part"
   `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:91-97` — two Triggers inside one
   Root.
4. "Multiple detached triggers"
   `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:99-107` — one shared handle, two
   detached Triggers plus the Root.
5. "Detached triggers with payload"
   `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:114-144` — typed handle
   `AlertDialog.createHandle<{ message: string }>()`, `payload` on each Trigger,
   function-as-a-child Root reading `{ payload }`, conditional `AlertDialog.Description`
   rendering `payload.message`; `@highlight`/`@highlight-text` annotations throughout
   `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:115-137`.

## Discrepancies (docs page vs. behavior.md)

1. Backdrop-click close request: the page says the confirmation dialog is "automatically shown
   when the user clicks the backdrop, presses the Esc key, or clicks a close button"
   `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:49`. behavior.md "State model"
   records the opposite for this component: "Backdrop click does not close the dialog and does
   not call `onOpenChange` — pointer dismissal is disabled for alert dialogs"
   `specs/library/alert-dialog/behavior.md:52-54`. The page's claim holds for the demo it
   reuses — the Dialog component's `close-confirmation` demo
   `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:53` — but not for an alert
   dialog as tested. Trust behavior.md for alert-dialog semantics here.
2. Open-state carryover: the page says "no open state carries over from a previous mount"
   `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:66`, while behavior.md "Edge
   cases" records that after unmounting the root while open and remounting it, the popup stays
   gone but `handle.isOpen` stays `true`
   `specs/library/alert-dialog/behavior.md:180-183`. The two may be reconcilable (the root
   starts fresh while the handle retains its own open flag), but the page wording does not make
   that distinction, so a reader could take it as contradicting the recorded handle behavior.
   Neither source should be silently trusted over the other without a test that pins the
   intended semantics.
3. Nested-dialog styling: `[data-nested-dialog-open]`, `var(--nested-dialogs)`, and "Backdrops
   of the child dialogs won't be rendered"
   `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:51` have no tested counterpart —
   behavior.md "DOM structure & portal behavior" explicitly marks nested alert dialogs untested
   `specs/library/alert-dialog/behavior.md:139-140`. Unverified rather than contradicted.
4. Coverage gap (not a contradiction): page prose omits `defaultOpen`, `defaultTriggerId`,
   `onOpenChangeComplete`, `actionsRef`/`preventUnmountOnClose()`, and handle `close()`, all of
   which are documented behavior in behavior.md "Public API surface", "State model", and
   "Events" `specs/library/alert-dialog/behavior.md:22-38`,
   `specs/library/alert-dialog/behavior.md:59-72`.

## Cross-links to other docs pages

- N/A for literal cross-links: the page contains no markdown hyperlinks to other docs pages.
- The only content dependency on another component's docs tree is the reused Dialog demo import
  (`../dialog/demos/close-confirmation`)
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:53` — recorded as an import line
  only; demo internals belong to Stage 2.
- Demo imports from this page's own demos directory — hero
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:9`, open-from-menu
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:41`, detached-triggers-simple
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:82`,
  detached-triggers-controlled `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:155`
  — are page-structure facts only; their code is mined separately (Stage 2).
- The API reference imports the generated types module (`./types`)
  `docs/src/app/(docs)/react/components/alert-dialog/page.mdx:161`.
