# Popover docs-content page spec (Stage 1 docs)

Mined from the docs page `.mdx` only. Component behavior is NOT re-derived here — cross-checks
reference `specs/library/popover/behavior.md` by section name, and cite it by path/lines only
where a specific discrepancy hinges on an exact recorded claim. Demo internals are out of scope
(Stage 2 mines `specs/docs-content/popover/demos.json`).

## Page structure (headings, in order)

- `# Popover` — `docs/src/app/(docs)/react/components/popover/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/popover/page.mdx:13`
- `## Examples` — `docs/src/app/(docs)/react/components/popover/page.mdx:38`
  - `### Opening on hover` — `docs/src/app/(docs)/react/components/popover/page.mdx:40`
  - `### Detached triggers` — `docs/src/app/(docs)/react/components/popover/page.mdx:50`
  - `### Multiple triggers` — `docs/src/app/(docs)/react/components/popover/page.mdx:81`
  - `### Controlled mode with multiple triggers` — `docs/src/app/(docs)/react/components/popover/page.mdx:152`
  - `### Animating the Popover` — `docs/src/app/(docs)/react/components/popover/page.mdx:165`
    - `#### Position and Size` — `docs/src/app/(docs)/react/components/popover/page.mdx:170`
    - `#### Content` — `docs/src/app/(docs)/react/components/popover/page.mdx:175`
- `## API reference` — `docs/src/app/(docs)/react/components/popover/page.mdx:195`
  - `### Root` — `docs/src/app/(docs)/react/components/popover/page.mdx:199`
  - `### Trigger` — `docs/src/app/(docs)/react/components/popover/page.mdx:203`
  - `### Backdrop` — `docs/src/app/(docs)/react/components/popover/page.mdx:207`
  - `### Portal` — `docs/src/app/(docs)/react/components/popover/page.mdx:211`
  - `### Positioner` — `docs/src/app/(docs)/react/components/popover/page.mdx:215`
  - `### Popup` — `docs/src/app/(docs)/react/components/popover/page.mdx:219`
  - `### Arrow` — `docs/src/app/(docs)/react/components/popover/page.mdx:223`
  - `### Title` — `docs/src/app/(docs)/react/components/popover/page.mdx:227`
  - `### Description` — `docs/src/app/(docs)/react/components/popover/page.mdx:231`
  - `### Close` — `docs/src/app/(docs)/react/components/popover/page.mdx:235`
  - `### Viewport` — `docs/src/app/(docs)/react/components/popover/page.mdx:239`
- `## createHandle` — `docs/src/app/(docs)/react/components/popover/page.mdx:245`
  - `### Handle` — `docs/src/app/(docs)/react/components/popover/page.mdx:251` (kept out of the
    table of contents by an `@exclude-table-of-contents` marker directly before it —
    `docs/src/app/(docs)/react/components/popover/page.mdx:249`)

Non-heading page furniture, in document order: `<Subtitle>` tagline plus `<Meta>` description
(`docs/src/app/(docs)/react/components/popover/page.mdx:3-7`); hero demo import/render
(`docs/src/app/(docs)/react/components/popover/page.mdx:9-11`); demo imports/renders inside the
Examples subsections (`docs/src/app/(docs)/react/components/popover/page.mdx:46-48`,
`docs/src/app/(docs)/react/components/popover/page.mdx:77-79`,
`docs/src/app/(docs)/react/components/popover/page.mdx:161-163`,
`docs/src/app/(docs)/react/components/popover/page.mdx:191-193` — the Multiple triggers
subsection has no demo); generated API table renders across the API reference and createHandle
sections (`docs/src/app/(docs)/react/components/popover/page.mdx:199-253`); and an exported
`metadata` keywords block (`docs/src/app/(docs)/react/components/popover/page.mdx:255-278`).

## Prose claims about component behavior

- Tagline: "An accessible popup anchored to a button."
  `docs/src/app/(docs)/react/components/popover/page.mdx:3`; meta description repeats it as "a
  high-quality, unstyled React popover component that displays an accessible popup anchored to a
  button" `docs/src/app/(docs)/react/components/popover/page.mdx:4-7`.
  Cross-check: supported by behavior.md "Accessibility" (`role="dialog"` popup) and "Public API
  surface" (Trigger renders a native button). No contradiction.
- Anatomy: "Import the component and assemble its parts"
  `docs/src/app/(docs)/react/components/popover/page.mdx:15`; the snippet imports from the
  `@base-ui/react/popover` namespace and nests Root → Trigger, Portal → Backdrop + Positioner →
  Popup → Arrow + Viewport → Title/Description/Close
  `docs/src/app/(docs)/react/components/popover/page.mdx:17-36`.
  Cross-check: consistent with behavior.md "Public API surface", which lists the same eleven
  parts plus `createHandle()`. No contradiction. (The anatomy shows Title/Description/Close
  inside the Viewport; behavior.md does not prescribe part placement, so nothing to contradict.)
- Opening on hover: configure the popover "to open on hover using the `openOnHover` prop"
  `docs/src/app/(docs)/react/components/popover/page.mdx:42`; "the `delay` prop to specify how
  long to wait (in milliseconds) before the popover opens on hover"
  `docs/src/app/(docs)/react/components/popover/page.mdx:44`.
  Cross-check: consistent with behavior.md "Public API surface" (`openOnHover` with `delay` and
  `closeDelay`) and "State model" (opens after the rest-type `delay`, closes after
  `closeDelay`). No contradiction — the page never mentions `closeDelay` (coverage gap only,
  see Discrepancies #4).
- Detached triggers: a trigger may sit "either inside or outside the `<Popover.Root>`"
  `docs/src/app/(docs)/react/components/popover/page.mdx:52`; for simple one-off interactions
  place the Trigger inside the Root, "as shown in the example at the top of this page"
  `docs/src/app/(docs)/react/components/popover/page.mdx:53`; otherwise place the Trigger
  outside the Root "and linking them with a `handle` created by the `Popover.createHandle()`
  function" `docs/src/app/(docs)/react/components/popover/page.mdx:55-56`.
  Cross-check: consistent with behavior.md "Public API surface" (static `createHandle()`,
  `handle` prop on both Trigger and Root). No contradiction.
- Handle imperative methods "such as `open()` and `close()`" "require a `<Popover.Root>` using
  the same handle to be mounted" `docs/src/app/(docs)/react/components/popover/page.mdx:58`.
  Cross-check: behavior.md "State model" documents the imperative `open(triggerId)`/`close()`
  pair; the mount requirement is implied by its ignored-calls record there. No contradiction.
- Handle calls with no root attached — "before one mounts, or after it unmounts — are ignored",
  and "each time a root mounts, it starts from fresh state: a call made while no root was
  attached is not replayed, and no open state carries over from a previous mount"
  `docs/src/app/(docs)/react/components/popover/page.mdx:59`.
  Cross-check: consistent with behavior.md "State model" (ignored calls with a "no root using
  this handle is mounted" warning) and "Edge cases" (unmount resets `handle.isOpen`; after
  remount the popover starts closed with no payload). No contradiction.
- Multiple triggers: "A single popover can be opened by multiple trigger elements" via "the same
  `handle` for several detached triggers, or by placing multiple `<Popover.Trigger>` components
  inside a single `<Popover.Root>`" `docs/src/app/(docs)/react/components/popover/page.mdx:83-84`.
  Cross-check: consistent with behavior.md "State model" (active-trigger model) and
  "Accessibility" (only the active trigger gets `aria-expanded="true"`/`aria-controls`). No
  contradiction.
- Per-trigger content: "The popover can render different content depending on which trigger
  opened it", achieved "by passing a `payload` to the `<Popover.Trigger>` and using the
  function-as-a-child pattern in `<Popover.Root>`"
  `docs/src/app/(docs)/react/components/popover/page.mdx:110-111`; "The payload can be strongly
  typed by providing a type argument to the `createHandle()` function"
  `docs/src/app/(docs)/react/components/popover/page.mdx:113`.
  Cross-check: payload consistent with behavior.md "Public API surface" (children-as-function
  receiving `{ payload }`; Trigger `payload` prop; handle `open(id)` sets the payload from that
  trigger). The generic-typing claim has no tested counterpart there. Not contradicted, just
  uncovered.
- Controlled mode: open state controlled externally via `open` and `onOpenChange` on
  `<Popover.Root>`, managing visibility from application state
  `docs/src/app/(docs)/react/components/popover/page.mdx:154-155`; with multiple triggers, "you
  have to manage which trigger is active with the `triggerId` prop on `<Popover.Root>` and the
  `id` prop on each `<Popover.Trigger>`"
  `docs/src/app/(docs)/react/components/popover/page.mdx:156`.
  Cross-check: consistent with behavior.md "State model" (controlled `triggerId`, and
  `onOpenChange` receives `details.trigger?.id` which the app mirrors back) and "Public API
  surface" (Trigger `id`). No contradiction.
- "Note that there is no separate `onTriggerIdChange` prop"
  `docs/src/app/(docs)/react/components/popover/page.mdx:158`.
  Cross-check: behavior.md documents no such prop either — consistent absence.
- "the `onOpenChange` callback receives an additional argument, `eventDetails`, which contains
  the trigger element that initiated the state change"
  `docs/src/app/(docs)/react/components/popover/page.mdx:159`.
  Cross-check: broadly consistent with behavior.md "Events" (`eventDetails.trigger` is the
  active trigger element), though the tested contract is richer — see Discrepancies #3.
- Animation overview: "You can animate a popover as it moves between different trigger elements.
  This includes animating its position, size, and content"
  `docs/src/app/(docs)/react/components/popover/page.mdx:167-168`.
  Cross-check: behavior.md covers multi-trigger switching (reused popup/positioner DOM nodes, no
  inline `scale` after a switch) but makes no animation-technique claims of its own; nothing to
  contradict.
- Position/size technique: apply CSS transitions to the "`left`, `right`, `top`, and `bottom`
  properties of the **Positioner** part" for position, and "the `width` and `height` of the
  **Popup** part" for size `docs/src/app/(docs)/react/components/popover/page.mdx:172-173`.
  Cross-check: MISMATCH in framing with behavior.md "DOM structure & portal behavior" — see
  Discrepancies #1.
- Content transitions: "To enable content animations, wrap the content in the
  `<Popover.Viewport>` part", which "provides features to create direction-aware animations"
  `docs/src/app/(docs)/react/components/popover/page.mdx:180-181`; the Viewport "renders a
  `div` with a `data-activation-direction` attribute that indicates the new trigger's position
  relative to the previous one", whose value is "a space-separated set of up to two tokens (one
  per axis) — `left` or `right` for the horizontal axis and `up` or `down` for the vertical axis
  (for example, `right down`)", matched with the `~=` attribute selector
  `docs/src/app/(docs)/react/components/popover/page.mdx:182`.
  Cross-check: consistent with behavior.md "Accessibility" (`data-activation-direction`
  space-separated tokens, omitted within tolerance) and "Edge cases" (~5px direction tolerance).
  No contradiction.
- Viewport wrappers: inside the Viewport, content is "wrapped in `div`s with data attributes":
  "`data-current`: The currently visible content when no transitions are present or the
  incoming content" and "`data-previous`: The outgoing content during a transition", used to
  style enter and exit animations
  `docs/src/app/(docs)/react/components/popover/page.mdx:184-189`.
  Cross-check: consistent with behavior.md "Accessibility" (`data-previous` rendered inert with
  the outgoing content inside a `[data-transitioning]` wrapper, cleaned up after the animation)
  and "DOM structure & portal behavior" (content renders inside `[data-current]` by default).
  The page omits the wrapper/inert details — simplification, not contradiction.
- Viewport guidance: "The `Viewport` is optional — reach for it only when a single popup is
  opened by multiple triggers, its content differs per trigger, and the switch between them is
  animated"; when used, "set `width: var(--positioner-width)` and
  `height: var(--positioner-height)` on the `Positioner` so its box is frozen to the measured
  size during the transition; otherwise content-driven resizing can make the popup thrash or
  flip to another side" `docs/src/app/(docs)/react/components/popover/page.mdx:243`.
  Cross-check: the "optional / only when animated" framing matches the multi-trigger switching
  scenarios behavior.md describes; the CSS-variable recipe has no tested counterpart — see
  Discrepancies #2.

## API tables referenced (props/parts documented on this page)

- The API reference section renders generated table components imported from `./types`
  `docs/src/app/(docs)/react/components/popover/page.mdx:197`; the `.mdx` itself contains no
  inline prop tables. Parts documented via heading + `<TypesPopover.* />`: Root, Trigger,
  Backdrop, Portal, Positioner, Popup, Arrow, Title, Description, Close, Viewport
  `docs/src/app/(docs)/react/components/popover/page.mdx:199-241`.
- The `## createHandle` section documents the factory
  `docs/src/app/(docs)/react/components/popover/page.mdx:245-247` and a separate `Handle` type
  section `docs/src/app/(docs)/react/components/popover/page.mdx:251-253`.
- Props/parts named in page prose or embedded snippets (outside the generated tables): Trigger
  and Root `handle` `docs/src/app/(docs)/react/components/popover/page.mdx:56-58`,
  `docs/src/app/(docs)/react/components/popover/page.mdx:66`,
  `docs/src/app/(docs)/react/components/popover/page.mdx:72`; Trigger `openOnHover` and `delay`
  `docs/src/app/(docs)/react/components/popover/page.mdx:42-44`; Trigger `payload` and typed
  `createHandle()` `docs/src/app/(docs)/react/components/popover/page.mdx:111-117`; Root
  `open`/`onOpenChange` `docs/src/app/(docs)/react/components/popover/page.mdx:154`; Root
  `triggerId` and Trigger `id` `docs/src/app/(docs)/react/components/popover/page.mdx:156`;
  Positioner `sideOffset` `docs/src/app/(docs)/react/components/popover/page.mdx:134`; and the
  Viewport data attributes / Positioner CSS variables
  `docs/src/app/(docs)/react/components/popover/page.mdx:182-187`,
  `docs/src/app/(docs)/react/components/popover/page.mdx:243`.
- Behavior-documented API with no prose mention on this page: Root `modal`, `defaultOpen`,
  `defaultTriggerId`, `onOpenChangeComplete`, `actionsRef`; Trigger `disabled`; Portal
  `keepMounted`/`container`; Popup `initialFocus`/`finalFocus`; Positioner
  `side`/`align`/offsets/`collisionAvoidance`/`anchor`/`arrowPadding`; Trigger `closeDelay`;
  and handle `close()` — all recorded in behavior.md "Public API surface" and "State model"
  `specs/library/popover/behavior.md:16-21`,
  `specs/library/popover/behavior.md:33-36`. Whether the generated tables cover them is out of
  scope here (the `./types` module was not opened).

## Code snippets embedded directly in the .mdx (not pulled from demos/)

1. "Anatomy" `docs/src/app/(docs)/react/components/popover/page.mdx:17-36` — namespace import
   plus full part assembly (Root, Trigger, Portal, Backdrop, Positioner, Popup, Arrow, Viewport,
   Title, Description, Close).
2. "Detached triggers" `docs/src/app/(docs)/react/components/popover/page.mdx:61-75` —
   `Popover.createHandle()`, `<Popover.Trigger handle={...}>`, `<Popover.Root handle={...}>`;
   annotated with `@highlight`/`@highlight-text` markers
   `docs/src/app/(docs)/react/components/popover/page.mdx:64-71`.
3. "Multiple triggers within the Root part"
   `docs/src/app/(docs)/react/components/popover/page.mdx:86-92` — two Triggers inside one Root.
4. "Multiple detached triggers"
   `docs/src/app/(docs)/react/components/popover/page.mdx:94-108` — one shared handle, two
   detached Triggers plus the Root.
5. "Detached triggers with payload"
   `docs/src/app/(docs)/react/components/popover/page.mdx:115-150` — typed handle
   `Popover.createHandle<{ text: string }>()`, `payload` on each Trigger, function-as-a-child
   Root reading `{ payload }`, Positioner `sideOffset={8}`, an `<ArrowSvg />` child referenced
   but not defined in the snippet, and conditional `Popover.Description` rendering
   `payload.text`; `@highlight`/`@highlight-text` annotations throughout
   `docs/src/app/(docs)/react/components/popover/page.mdx:116-142`.

## Discrepancies (docs page vs. behavior.md)

1. Position-animation technique: the page instructs animating position by transitioning the
   Positioner's `left`/`right`/`top`/`bottom`
   `docs/src/app/(docs)/react/components/popover/page.mdx:172`, but behavior.md "DOM structure
   & portal behavior" records that positioning is transform-based on the positioner without a
   Viewport and only "switches to `top`/`left` positioning" when a `Popover.Viewport` is used
   `specs/library/popover/behavior.md:82-83`. Following the page advice without a Viewport
   would transition properties that are not used for positioning. Not a hard contradiction —
   the animated multi-trigger scenario the section describes implies Viewport usage per the
   page's own guidance `docs/src/app/(docs)/react/components/popover/page.mdx:243` — but the
   dependency is unstated. The Popup `width`/`height` transition advice
   `docs/src/app/(docs)/react/components/popover/page.mdx:173` has no tested counterpart at
   all.
2. Viewport sizing recipe: `var(--positioner-width)`/`var(--positioner-height)` on the
   Positioner, and the thrash-or-flip warning, are untested — behavior.md's only CSS-variable
   note is an UNVERIFIED record of the popup consuming `var(--available-height)`
   `specs/library/popover/behavior.md:91`. Unverified rather than contradicted.
3. `eventDetails.trigger` simplification: the page says the callback argument "contains the
   trigger element that initiated the state change"
   `docs/src/app/(docs)/react/components/popover/page.mdx:159`, while behavior.md "Events"
   records that `eventDetails.trigger` is the active trigger element but falls back to the
   popup's anchored element when the active trigger id is unregistered and is `undefined` when
   no trigger is mounted `specs/library/popover/behavior.md:95`. Simplification, not
   contradiction.
4. Coverage gap (not a contradiction): page prose omits the behavior-documented API listed in
   the API-tables section above (`modal`, `defaultOpen`, `defaultTriggerId`,
   `onOpenChangeComplete`, `actionsRef`, `initialFocus`/`finalFocus`, `keepMounted`/`container`,
   collision/anchor positioning props, `closeDelay`, handle `close()`)
   `specs/library/popover/behavior.md:16-21`,
   `specs/library/popover/behavior.md:33-36`.

## Cross-links to other docs pages

- N/A for literal cross-links: the page contains no markdown hyperlinks to other docs pages.
- No dependency on another component's docs tree: every content import resolves inside this
  page's own directory.
- Demo imports from this page's own demos directory — hero
  `docs/src/app/(docs)/react/components/popover/page.mdx:9`, open-on-hover
  `docs/src/app/(docs)/react/components/popover/page.mdx:46`, detached-triggers-simple
  `docs/src/app/(docs)/react/components/popover/page.mdx:77`,
  detached-triggers-controlled `docs/src/app/(docs)/react/components/popover/page.mdx:161`, and
  detached-triggers-full `docs/src/app/(docs)/react/components/popover/page.mdx:191` — are
  page-structure facts only; their code is mined separately (Stage 2).
- The API reference imports the generated types module (`./types`)
  `docs/src/app/(docs)/react/components/popover/page.mdx:197`.
