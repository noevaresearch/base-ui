# Tooltip docs page content spec

Mined from `docs/src/app/(docs)/react/components/tooltip/page.mdx` only. The component's own
behavior is covered by `specs/library/tooltip/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/tooltip/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/tooltip/demos.json`).

## Page structure (headings, in order)

- `# Tooltip` (h1) — `docs/src/app/(docs)/react/components/tooltip/page.mdx:1`
- `## Usage guidelines` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:15`
- `## Anatomy` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:20`
- `## Alternatives to tooltips` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:42`
  - `### Infotips` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:50`
  - `### Description text` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:57`
  - `### Contextual feedback messages` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:65`
- `## Examples` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:69`
  - `### Detached triggers` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:71`
  - `### Multiple triggers` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:100`
  - `### Controlled mode with multiple triggers` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:170`
  - `### Animating the Tooltip` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:183`
    - `#### Position and Size` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:188`
    - `#### Content` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:193`
- `## API reference` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:213`
  - `### Provider` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:217`
  - `### Root` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:221`
  - `### Trigger` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:225`
  - `### Portal` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:229`
  - `### Positioner` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:233`
  - `### Popup` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:237`
  - `### Arrow` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:241`
  - `### Viewport` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:245`
- `## createHandle` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:251`
  - `### Handle` — `docs/src/app/(docs)/react/components/tooltip/page.mdx:257`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A popup that appears when an element is hovered or focused, showing a hint for
  sighted users." — `docs/src/app/(docs)/react/components/tooltip/page.mdx:3-5`
- `<Meta name="description">` — "A high-quality, unstyled React tooltip component that appears
  when an element is hovered or focused, showing a hint for sighted users." —
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:6-9`
- Hero demo import and render (`./demos/hero`) before the first heading —
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:11-13`
- Demo imports interleaved with the Examples subsections (`./demos/detached-triggers-simple` at
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:96`, `./demos/detached-triggers-controlled`
  at `docs/src/app/(docs)/react/components/tooltip/page.mdx:179`, `./demos/detached-triggers-full`
  at `docs/src/app/(docs)/react/components/tooltip/page.mdx:209`)
- `TypesTooltip` import for the API reference — `docs/src/app/(docs)/react/components/tooltip/page.mdx:215`
- `[//]: # '@exclude-table-of-contents'` marker between the `createHandle` table and the
  `### Handle` heading — `docs/src/app/(docs)/react/components/tooltip/page.mdx:255`
- Trailing `export const metadata` SEO keywords block (15 keywords, e.g. 'React Tooltip',
  'Detached Trigger Tooltip', 'Aria Describedby Tooltip', 'Animated Tooltip') —
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:261-279`

## Prose claims about component behavior

- Page describes the component as "A popup that appears when an element is hovered or focused,
  showing a hint for sighted users." and, in the meta description, as "unstyled". Consistent with
  the hover-open and focus-open behavior in behavior.md "State model (controlled/uncontrolled,
  defaults, transitions)" and "Focus management".
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:3-5`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:6-9`
- Usage guideline: "Prefer using tooltips as visual labels only" — tooltips should act as
  supplementary visual labels for sighted mouse and keyboard users, and "Tooltips alone are not
  accessible to touch or screen reader users." Docs-only usage guidance; behavior.md does not
  cover accessibility of the tooltip content to touch/screen-reader users (its "Accessibility"
  section notes the popup's ARIA role is untested and no aria-describedby link is asserted).
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:17`
- Usage guideline: "Provide an accessible name for the trigger" — tooltips are visual-only and
  not a replacement for labeling the trigger; "The tooltip's trigger must have an `aria-label`
  attribute that closely matches the tooltip's content to ensure consistency for screen reader
  users." Docs-only claim; no test in behavior.md ("Accessibility") asserts any aria-label or
  describedby linking; see Discrepancies.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:18`
- Anatomy usage guidance: "Import the component and assemble its parts", showing
  `Tooltip.Provider` wrapping `Tooltip.Root`, which contains `Tooltip.Trigger` and
  `Tooltip.Portal` > `Tooltip.Positioner` > `Tooltip.Popup` with `Tooltip.Arrow` and
  `Tooltip.Viewport` inside the Popup, imported from the `@base-ui/react/tooltip` namespace.
  The part set and namespace match behavior.md "Public API surface (props, parts, subcomponents)"
  and its "DOM structure & portal behavior" composition (Root → Trigger → Portal → Positioner →
  Popup → Arrow/Viewport); the Provider wrapper is additionally shown (behavior.md covers
  Provider behavior in its "State model" but not in the composition under test).
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:22-40`
- "Tooltips should be supplementary popups that provide non-essential clarity in high-density
  UIs. A user should not miss critical information if they never see a tooltip." Docs-only usage
  philosophy; not covered by behavior.md.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:44`
- "Tooltips don't work well with touch input. Unlike mouse pointers with hover capability,
  there's no easily discoverable way to reveal a tooltip before tapping its trigger on a touch
  device." Docs-only rationale; behavior.md only records touch-pointer edge cases (local reopen
  path not triggered by touch; mouse hover works after touch) in "Edge cases (rapid
  interactions, unmount, nesting)".
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:46`
- "iOS doesn't provide a system-standard, touch-friendly tooltip affordance, while Android may
  show a tooltip on long press. However, on the web, long press is often used to trigger
  contextual menus in the browser, which can lead to potential conflicts. For this reason,
  tooltips are disabled on touch devices." The blanket "disabled on touch devices" claim is not
  verified by behavior.md (no test asserts general touch disablement); see Discrepancies.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:48`
- Infotips guidance: "Popups that open when hovering an info icon should use
  [Popover](/react/components/popover) with the `openOnHover` prop on the trigger instead of a
  tooltip. This way, touch users and screen reader users can access the content." Cross-component
  guidance (Popover's `openOnHover` prop); outside behavior.md's scope; see Discrepancies.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:52`
- Trigger-purpose heuristic: "If the trigger's purpose is to open the popup itself, it's a
  popover. If the trigger's purpose is unrelated to opening the popup, it's a tooltip."
  Docs-only decision guidance; not covered by behavior.md.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:54-55`
- Description-text guidance: "Tooltips are designed for sighted users and are not a reliable way
  to deliver important information to touch users or assistive technologies. If the description
  is important to understanding the element, don't hide it behind a tooltip — use inline text or
  [Popover](/react/components/popover) if space is limited, so the information is accessible to
  everyone." Docs-only usage guidance; not covered by behavior.md.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:59`
- "Since tooltips serve sighted mouse and keyboard users, iconography should clearly communicate
  the purpose of icon-only triggers, especially on mobile where the text label may not be
  visible." Docs-only usage guidance; not covered by behavior.md.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:61`
- "If the description is not critical, a tooltip can still be used to provide extra clarity for
  sighted mouse or keyboard users." Docs-only usage guidance; not covered by behavior.md.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:63`
- Contextual feedback guidance: "Use the Toast component's [anchoring
  ability](/react/components/toast#anchored-toasts) for more ergonomic DX, to ensure the message
  is announced to screen readers, and to support complex content." Cross-component guidance
  (Toast); outside behavior.md's scope; see Discrepancies.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:67`
- "A tooltip can be controlled by a trigger located either inside or outside the
  `<Tooltip.Root>` component." and "For simple, one-off interactions, place the
  `<Tooltip.Trigger>` inside `<Tooltip.Root>`, as shown in the example at the top of this page."
  Matches behavior.md's structural note that behaviors hold for contained, detached, and
  multiple-detached trigger wirings.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:73-74`
- "if defining the tooltip's content next to its trigger is not practical, you can use a
  detached trigger. This involves placing the `<Tooltip.Trigger>` outside of `<Tooltip.Root>`
  and linking them with a `handle` created by the `Tooltip.createHandle()` function." Matches
  behavior.md "Public API surface (props, parts, subcomponents)" (`handle` prop on Trigger and
  Root; static `Tooltip.createHandle()`).
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:76-77`
- Handle lifecycle: "The imperative methods on the handle, such as `open()` and `close()`,
  require a `<Tooltip.Root>` using the same handle to be mounted. Calls made while no root is
  attached to the handle — before one mounts, or after it unmounts — are ignored. Each time a
  root mounts, it starts from fresh state: a call made while no root was attached is not
  replayed, and no open state carries over from a previous mount." Matches behavior.md "State
  model (controlled/uncontrolled, defaults, transitions)" (handle methods, calls ignored with no
  mounted root, state resets to closed/no payload on re-attach); the docs omit the
  accompanying console warning behavior.md records; see Discrepancies.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:79-80`
- "A single tooltip can be opened by multiple trigger elements. You can achieve this by using
  the same `handle` for several detached triggers, or by placing multiple `<Tooltip.Trigger>`
  components inside a single `<Tooltip.Root>`." Matches behavior.md's wirings ("multiple
  detached triggers" and multiple triggers inside one root) in its structural note and
  "Focus management".
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:102-103`
- "The tooltip can render different content depending on which trigger opened it. This is
  achieved by passing a `payload` to the `<Tooltip.Trigger>` and using the function-as-a-child
  pattern in `<Tooltip.Root>`." Matches behavior.md "State model" (popup content reflects the
  active trigger's `payload`; render-prop children form receiving `{ payload }`).
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:129-130`
- "The payload can be strongly typed by providing a type argument to the `createHandle()`
  function" — matches behavior.md's `createHandle<T>()` signature in "Public API surface
  (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:132`
- Controlled mode: "You can control the tooltip's open state externally using the `open` and
  `onOpenChange` props on `<Tooltip.Root>`." Matches behavior.md "State model" (`open`,
  `onOpenChange(open, details)`).
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:172`
- "When using multiple triggers, you have to manage which trigger is active with the
  `triggerId` prop on `<Tooltip.Root>` and the `id` prop on each `<Tooltip.Trigger>`." Matches
  behavior.md "Public API surface" (`triggerId` on Root, `id` on Trigger) and "State model"
  (active-trigger control via `triggerId`).
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:174`
- "Note that there is no separate `onTriggerIdChange` prop. Instead, the `onOpenChange`
  callback receives an additional argument, `eventDetails`, which contains the trigger element
  that initiated the state change." Consistent with behavior.md "Events (names, payload shape,
  bubbling, preventDefault semantics)" (`details.trigger` exposes the active trigger object with
  an `id`); the docs name the second argument `eventDetails` while behavior.md calls it
  `details` — same object, different local name; see Discrepancies.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:176-177`
- "You can animate a tooltip as it moves between different trigger elements. This includes
  animating its position, size, and content." Matches behavior.md's Viewport trigger-switch
  coverage in "DOM structure & portal behavior" and "Edge cases (rapid interactions, unmount,
  nesting)".
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:185-186`
- "To animate the tooltip's position, apply CSS transitions to the `left`, `right`, `top`, and
  `bottom` properties of the **Positioner** part. To animate its size, transition the `width`
  and `height` of the **Popup** part." Docs-only styling instruction; behavior.md records that
  positioning is applied via inline `transform` without a Viewport and switches to top/left
  with a Viewport ("DOM structure & portal behavior"), which the page does not mention; see
  Discrepancies.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:190-191`
- "The tooltip also supports content transitions. This is useful when different triggers display
  different content within the same tooltip." Matches behavior.md's Viewport content-switch
  behavior in "DOM structure & portal behavior".
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:195-196`
- "To enable content animations, wrap the content in the `<Tooltip.Viewport>` part. This part
  provides features to create direction-aware animations." Consistent with behavior.md's
  Viewport coverage in "DOM structure & portal behavior".
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:198-199`
- "It renders a `div` with a `data-activation-direction` attribute that indicates the new
  trigger's position relative to the previous one. The value is a space-separated set of up to
  two tokens (one per axis) — `left` or `right` for the horizontal axis and `up` or `down` for
  the vertical axis (for example, `right down`). Match a single token with the `~=` attribute
  selector, such as `[data-activation-direction~='right']`." Matches behavior.md "DOM structure
  & portal behavior" (space-separated `right`/`left`/`down`/`up` tokens, single-axis movement
  reduced to one token); behavior.md additionally records that small (≈5px) deltas are dropped,
  which the page omits; see Discrepancies.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:200`
- "Inside the `<Tooltip.Viewport>`, the content is further wrapped in `div`s with data
  attributes to help with styling: `data-current`: The currently visible content when no
  transitions are present or the incoming content. `data-previous`: The outgoing content during
  a transition." Matches behavior.md "DOM structure & portal behavior" (children render inside a
  `[data-current]` container; a `[data-previous]` container holding the old content coexists
  during animations); behavior.md adds that the previous container is marked `inert`, which the
  page omits; see Discrepancies.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:202-206`
- Viewport usage note in the API reference: "The `Viewport` is optional — reach for it only
  when a single popup is opened by multiple triggers, its content differs per trigger, and the
  switch between them is animated. When used, set `width: var(--positioner-width)` and
  `height: var(--positioner-height)` on the `Positioner` so its box is frozen to the measured
  size during the transition; otherwise content-driven resizing can make the popup thrash or
  flip to another side." Docs-only guidance; the CSS-variable freezing technique is not covered
  by behavior.md; see Discrepancies.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:249`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents eight parts, each as its own heading rendering a
  generated reference component: `### Provider` → `<TypesTooltip.Provider />`,
  `### Root` → `<TypesTooltip.Root />`, `### Trigger` → `<TypesTooltip.Trigger />`,
  `### Portal` → `<TypesTooltip.Portal />`, `### Positioner` → `<TypesTooltip.Positioner />`,
  `### Popup` → `<TypesTooltip.Popup />`, `### Arrow` → `<TypesTooltip.Arrow />`,
  `### Viewport` → `<TypesTooltip.Viewport />`.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:213-247`
- A separate `## createHandle` section documents the static factory
  (`<TypesTooltip.createHandle />`) and, after the `@exclude-table-of-contents` marker, a
  `### Handle` section documents the handle object type (`<TypesTooltip.Handle />`).
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:251-259`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesTooltip } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:215`
- Parts documented on this page: Provider, Root, Trigger, Portal, Positioner, Popup, Arrow,
  Viewport, plus `createHandle()`/`Handle`. behavior.md "Public API surface (props, parts,
  subcomponents)" enumerates Root, Trigger, Portal, Positioner, Popup, Arrow, Viewport plus
  `createHandle()` and the handle object — the same set except that Provider is documented on
  the page but not listed in behavior.md's part enumeration (Provider behavior is nonetheless
  covered in behavior.md's "State model"); see Discrepancies.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:217-259`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- Five fenced code blocks exist in the page:
  1. Anatomy snippet (` ```jsx title="Anatomy" `): imports `{ Tooltip }` from
     `@base-ui/react/tooltip` and assembles `<Tooltip.Provider>` > `<Tooltip.Root>` containing
     `<Tooltip.Trigger />` and `<Tooltip.Portal>` > `<Tooltip.Positioner>` > `<Tooltip.Popup>`
     with `<Tooltip.Arrow />` and `<Tooltip.Viewport />` inside the Popup.
     `docs/src/app/(docs)/react/components/tooltip/page.mdx:24-40`
  2. "Detached triggers" snippet: `const demoTooltip = Tooltip.createHandle();` with
     `<Tooltip.Trigger handle={demoTooltip}>Button</Tooltip.Trigger>` and
     `<Tooltip.Root handle={demoTooltip}>` (with `@highlight`/`@highlight-text` markers).
     `docs/src/app/(docs)/react/components/tooltip/page.mdx:82-94`
  3. "Multiple triggers within the Root part" snippet: one `<Tooltip.Root>` containing two
     `<Tooltip.Trigger>` children. `docs/src/app/(docs)/react/components/tooltip/page.mdx:105-111`
  4. "Multiple detached triggers" snippet: two detached `<Tooltip.Trigger handle={demoTooltip}>`
     elements sharing one handle plus `<Tooltip.Root handle={demoTooltip}>`.
     `docs/src/app/(docs)/react/components/tooltip/page.mdx:113-127`
  5. "Detached triggers with payload" snippet: `Tooltip.createHandle<{ text: string }>()`,
     two triggers each passing `payload={{ text: ... }}`, and `<Tooltip.Root>` using the
     function-as-a-child pattern rendering Portal > Positioner (with `sideOffset={8}`) > Popup
     (with `className={styles.Popup}`) > Arrow (with `className={styles.Arrow}` wrapping an
     `ArrowSvg`), conditionally showing `Tooltip opened by {payload.text}` (with
     `@highlight`/`@highlight-text` markers). `docs/src/app/(docs)/react/components/tooltip/page.mdx:134-168`
- No other code blocks exist in the page. The three Examples subsections that render demos
  (`<DemoTooltipDetachedTriggersSimple />`, `<DemoTooltipDetachedTriggersControlled />`,
  `<DemoTooltipDetachedTriggersFull />`, plus `<DemoTooltipHero />` at the top) import them
  from `./demos/*`; their code lives in demo files and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:11-13`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:96-98`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:179-181`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:209-211`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/tooltip/behavior.md`.
Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: the trigger "must have an `aria-label` attribute
  that closely matches the tooltip's content" (accessibility guidance). behavior.md
  "Accessibility" explicitly records that no test asserts `aria-describedby` or any id/aria link
  between trigger and popup, and the popup's ARIA role is untested; the aria-label requirement
  is likewise untested.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:18`
- Docs-only claim, not covered by behavior.md: "tooltips are disabled on touch devices"
  (presented as a consequence of iOS/Android long-press/context-menu conflicts). behavior.md
  "Edge cases (rapid interactions, unmount, nesting)" only verifies that the local reopen path
  is not triggered by touch and that mouse hover works after a touch interaction; no test
  asserts general disablement on touch devices.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:48`
- Docs-only cross-component guidance, out of behavior.md's scope: use Popover with
  `openOnHover` for infotips (and for important description text if space is limited), the
  popover-vs-tooltip trigger-purpose heuristic, and Toast's anchoring ability for contextual
  feedback messages.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:52`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:54-55`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:59`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:67`
- Docs-only claim, not covered by behavior.md: the Viewport usage note — reach for Viewport only
  when a single popup is opened by multiple triggers with differing content and an animated
  switch, and freeze the Positioner box with `width: var(--positioner-width)` /
  `height: var(--positioner-height)` during transitions to avoid thrashing or side flipping.
  behavior.md covers Viewport structure/animation attributes but not this CSS-variable technique.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:249`
- Unstated caveat (docs page vs. behavior.md): the page instructs animating the tooltip's
  position by transitioning `left`/`right`/`top`/`bottom` on the Positioner, but behavior.md
  "DOM structure & portal behavior" records that without a Viewport the positioner is placed
  via inline `transform` (top/left positioning only applies with a Viewport). The page's
  animation flow assumes the Viewport setup and never mentions the transform mode. Omission of
  context, not a contradiction.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:190`
- Omission (docs page vs. behavior.md): behavior.md "State model" records that ignored handle
  calls with no mounted root come with a "no root using this handle is mounted" console warning,
  and "Edge cases" records a deferred "more than one mounted root" warning when two roots share
  a handle. The page only says calls "are ignored" and never mentions these warnings. Omission
  of developer-facing caveats, not a contradiction of the ignored-call outcome.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:79-80`
- Omission (docs page vs. behavior.md): behavior.md "DOM structure & portal behavior" records
  that small (≈5px) activation-direction deltas are dropped; the page describes the
  `data-activation-direction` token format without the delta threshold.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:200`
- Omission (docs page vs. behavior.md): the page describes `data-previous` as "The outgoing
  content during a transition"; behavior.md adds that the previous container is marked `inert`.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:205`
- Note (not a mismatch): the page names the `onOpenChange` second argument `eventDetails`
  containing the trigger element; behavior.md "Events" refers to the same object as `details`
  with `details.trigger`. Different local name for the same callback argument.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:176-177`
- Note (not a mismatch): the page's API reference documents a `Provider` part and the page's
  Anatomy shows `Tooltip.Provider` wrapping the tree; behavior.md's "Public API surface (props,
  parts, subcomponents)" part enumeration omits Provider even though Provider behavior (delay,
  closeDelay, timeout) is covered in behavior.md's "State model". A gap in behavior.md's
  enumeration, not a docs error.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:27-39`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:217-219`

## Cross-links to other docs pages

- `[Popover](/react/components/popover)` — recommended for infotips (hover-opened info-icon
  popups) instead of a tooltip. `docs/src/app/(docs)/react/components/tooltip/page.mdx:52`
- `[Popover](/react/components/popover)` — recommended for important description text when
  space is limited. `docs/src/app/(docs)/react/components/tooltip/page.mdx:59`
- `[Toast](/react/components/toast#anchored-toasts)` — recommended (with its anchoring ability)
  for contextual feedback messages.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:67`
- One same-page anchor link: `[Alternatives to tooltips](#alternatives-to-tooltips)` inside the
  first Usage guidelines bullet — an in-page jump, not a cross-page link.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:17`
- Demo components (`./demos/hero`, `./demos/detached-triggers-simple`,
  `./demos/detached-triggers-controlled`, `./demos/detached-triggers-full`) and the `./types`
  API-reference module are imported and rendered on this page itself; they are same-page
  imports, not cross-page links.
  `docs/src/app/(docs)/react/components/tooltip/page.mdx:11`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:96`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:179`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:209`, `docs/src/app/(docs)/react/components/tooltip/page.mdx:215`
