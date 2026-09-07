# Preview Card docs page content spec

Mined from `docs/src/app/(docs)/react/components/preview-card/page.mdx` only. The component's own
behavior is covered by `specs/library/preview-card/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/preview-card/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/preview-card/demos.json`).

## Page structure (headings, in order)

- `# Preview Card` (h1) — `docs/src/app/(docs)/react/components/preview-card/page.mdx:1`
- `## Usage guidelines` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:20`
- `## Anatomy` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:25`
- `## Examples` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:46`
  - `### Detached triggers` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:48`
  - `### Multiple triggers` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:79`
  - `### Controlled mode with multiple triggers` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:146`
  - `### Animating the Preview Card` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:159`
    - `#### Position and Size` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:164`
    - `#### Content` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:169`
- `## API reference` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:189`
  - `### Root` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:193`
  - `### Trigger` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:197`
  - `### Portal` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:201`
  - `### Backdrop` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:205`
  - `### Positioner` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:209`
  - `### Popup` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:213`
  - `### Viewport` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:217`
  - `### Arrow` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:223`
- `## createHandle` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:227`
  - `### Handle` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:233`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A link that shows a destination preview without interrupting keyboard or screen
  reader navigation." — `docs/src/app/(docs)/react/components/preview-card/page.mdx:3-5`
- `<Meta name="description">` — "A high-quality, unstyled React preview card component for a link
  that shows a destination preview without interrupting keyboard or screen reader navigation." —
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:6-9`
- Hero demo import (`./demos/hero`) and render before the first heading —
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:11-13`
- `<link as="image" rel="preload">` for an `images.unsplash.com` URL immediately after the hero —
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:14-18`
- Demo imports interleaved with the Examples subsections (`./demos/detached-triggers-simple` at
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:75`,
  `./demos/detached-triggers-controlled` at
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:155`,
  `./demos/detached-triggers-full` at
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:185`)
- `TypesPreviewCard` import for the API reference —
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:191`
- `[//]: # '@exclude-table-of-contents'` comment between the `createHandle` heading and
  `### Handle` — `docs/src/app/(docs)/react/components/preview-card/page.mdx:231`
- Trailing `export const metadata` SEO keywords block (16 keywords, e.g. 'React Preview Card',
  'Detached Trigger Preview Card') —
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:237-256`

## Prose claims about component behavior

- Positioning claim: "A link that shows a destination preview without interrupting keyboard or
  screen reader navigation" (subtitle, repeated verbatim in the meta description). The link-based
  trigger matches behavior.md "Accessibility (roles, aria-*, id linking)"; keyboard open/close on
  trigger focus/blur and Escape match behavior.md "Keyboard interactions". The "without
  interrupting screen reader navigation" framing is a docs-only assertion with no test counterpart
  in behavior.md (its screen-reader coverage is limited to absence-of-assertion notes). See
  Discrepancies.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:3-5`, `docs/src/app/(docs)/react/components/preview-card/page.mdx:6-9`
- Usage guideline "Protect screen reader users' current context": exposing each preview to screen
  readers "would force users through its contents before they could continue through the page,
  repeatedly disrupting their current context"; keep the link as the only accessible interface and
  include all previewed information at its destination. Not covered by behavior.md (no test
  asserts screen reader presentation); see Discrepancies.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:22`
- Usage guideline "Keep popup content supplementary": avoid placing unique or essential information
  in the popup unless it is also available on the linked page, because "preview card content is not
  touch, keyboard or screen reader navigable"; the popup "acts as a visual progressive enhancement
  for sighted mouse and keyboard users only". The non-navigability claim is not asserted by any
  test recorded in behavior.md (its "Focus management" section marks focus-not-moved-into-popup as
  UNVERIFIED); see Discrepancies.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:23`
- Anatomy guidance: "Import the component and assemble its parts", with the snippet assembling
  `PreviewCard.Root` containing `PreviewCard.Trigger` and `PreviewCard.Portal` (containing
  `PreviewCard.Backdrop` and `PreviewCard.Positioner` > `PreviewCard.Popup` >
  `PreviewCard.Arrow` + `PreviewCard.Viewport`), imported from the `@base-ui/react/preview-card`
  namespace. The part set, namespace, and nesting match behavior.md "Public API surface (props,
  parts, subcomponents)" and "DOM structure & portal behavior" (Arrow inside Popup; Backdrop a
  sibling of Positioner inside Portal; Viewport an inner container inside Popup).
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:27-44`
- Detached triggers: a trigger can be located "either inside or outside the `<PreviewCard.Root>`
  component"; for simple, one-off interactions place the Trigger inside Root "as shown in the
  example at the top of this page". Consistent with behavior.md "Public API surface (props, parts,
  subcomponents)" (Trigger accepts `handle`; Trigger outside Root without a handle throws a
  context error).
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:50-51`
- Detached-trigger mechanics: link Trigger and Root "with a `handle` created by the
  `PreviewCard.createHandle()` function", placing the Trigger outside Root. Matches behavior.md
  "Public API surface (props, parts, subcomponents)" (`PreviewCard.createHandle()` factory for a
  handle linking detached triggers to a root).
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:53-54`
- Handle lifecycle requirement: "The imperative methods on the handle, such as `open()` and
  `close()`, require a `<PreviewCard.Root>` using the same handle to be mounted. Calls made while
  no root is attached to the handle — before one mounts, or after it unmounts — are ignored."
  Matches behavior.md "State model (controlled/uncontrolled, defaults, transitions)" (imperative
  calls on a handle with no mounted root are ignored, both before attachment and after
  detachment). The page omits the console warning behavior.md records for ignored calls; see
  Discrepancies.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:56-57`
- Handle fresh-state semantics: "Each time a root mounts, it starts from fresh state: a call made
  while no root was attached is not replayed, and no open state carries over from a previous
  mount." Not covered by behavior.md (no test recorded for replay suppression or open-state
  carry-over across mounts); see Discrepancies.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:57`
- Multiple triggers: "A single preview card can be opened by multiple trigger elements", either
  "by using the same `handle` for several detached triggers, or by placing multiple
  `<PreviewCard.Trigger>` components inside a single `<PreviewCard.Root>`". Consistent with
  behavior.md "Public API surface (props, parts, subcomponents)" (root suite runs popup
  conformance across contained-trigger, detached-trigger, and multiple-detached-trigger layouts).
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:81-82`
- Per-trigger payload: "The preview card can render different content depending on which trigger
  opened it", achieved by "passing a `payload` to the `<PreviewCard.Trigger>` and using the
  function-as-a-child pattern in `<PreviewCard.Root>`". Matches behavior.md "State model
  (controlled/uncontrolled, defaults, transitions)" (active-trigger tracking: the root's function
  children receive `{ payload }` taken from the active trigger's payload prop).
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:108-109`
- Payload typing: "The payload can be strongly typed by providing a type argument to the
  `createHandle()` function". Matches behavior.md "Public API surface (props, parts,
  subcomponents)" (createHandle is generic over the payload type).
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:111`
- Controlled mode: control the open state externally "using the `open` and `onOpenChange` props on
  `<PreviewCard.Root>`" to manage visibility "based on your application's state". Matches
  behavior.md "State model (controlled/uncontrolled, defaults, transitions)" (controlled `open` +
  `onOpenChange(nextOpen, details)`).
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:148-149`
- Active-trigger management while controlled: with multiple triggers "you have to manage which
  trigger is active with the `triggerId` prop on `<PreviewCard.Root>` and the `id` prop on each
  `<PreviewCard.Trigger>`". Matches behavior.md "State model (controlled/uncontrolled, defaults,
  transitions)" (`triggerId` / `defaultTriggerId` control the anchored trigger) and
  "Accessibility (roles, aria-*, id linking)" (these reference the trigger's DOM `id`).
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:150`
- No `onTriggerIdChange`: "Note that there is no separate `onTriggerIdChange` prop. Instead, the
  `onOpenChange` callback receives an additional argument, `eventDetails`, which contains the
  trigger element that initiated the state change." Matches behavior.md "Accessibility (roles,
  aria-*, id linking)" (`onOpenChange` details expose `details.trigger` with `.id`); behavior.md's
  root prop list contains no `onTriggerIdChange`, consistent with the absence claim.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:152-153`
- Animation overview: "You can animate a preview card as it moves between different trigger
  elements", including "its position, size, and content". Consistent with behavior.md "Edge cases
  (rapid interactions, unmount, nesting)" (viewport morphing on trigger switches) and "DOM
  structure & portal behavior" (positioner placement switches from inline `transform` to
  `top`/`left` when a Viewport is present).
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:161-162`
- Position/size animation targets: "apply CSS transitions to the `left`, `right`, `top`, and
  `bottom` properties of the **Positioner** part" for position; "transition the `width` and
  `height` of the **Popup** part" for size. Corroborated indirectly by behavior.md "DOM structure
  & portal behavior" (with a Viewport inside the popup the positioner uses `top`/`left`
  positioning and inline `transform` is empty, so left/top transitions are effective); the
  CSS-transition guidance itself is docs-only; see Discrepancies.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:166-167`
- Content animation enablement: "wrap the content in the `<PreviewCard.Viewport>` part", which
  "provides features to create direction-aware animations"; useful "when different triggers
  display different content within the same preview card". Matches behavior.md "Public API surface
  (props, parts, subcomponents)" (Viewport hosts morphing content containers) and "Edge cases
  (rapid interactions, unmount, nesting)" (morph transitions during trigger switches).
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:171-175`
- `data-activation-direction`: Viewport "renders a `div` with a `data-activation-direction`
  attribute that indicates the new trigger's position relative to the previous one. The value is a
  space-separated set of up to two tokens (one per axis) — `left` or `right` for the horizontal
  axis and `up` or `down` for the vertical axis (for example, `right down`). Match a single token
  with the `~=` attribute selector, such as `[data-activation-direction~='right']`." Matches
  behavior.md "Edge cases (rapid interactions, unmount, nesting)" (space-separated attribute,
  `right`/`left` horizontal, `up`/`down` vertical, both for diagonal); behavior.md additionally
  records an empty value when both deltas are within ~5px tolerance, which the page does not
  mention.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:176`
- Viewport content wrappers: inside the Viewport, content is "further wrapped in `div`s with data
  attributes": `data-current` is "The currently visible content when no transitions are present or
  the incoming content"; `data-previous` is "The outgoing content during a transition"; "You can
  use these attributes to style the enter and exit animations." Matches behavior.md "Edge cases
  (rapid interactions, unmount, nesting)" (during a trigger switch a `[data-previous]` container
  with old content coexists with a `[data-current]` container with new content; the viewport marks
  the active container `data-current`); behavior.md additionally records that the previous
  container is `inert` under a `[data-transitioning]` ancestor, which the page does not mention.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:178-183`
- Viewport optional guidance (prose inside the API reference): "The `Viewport` is optional — reach
  for it only when a single popup is opened by multiple triggers, its content differs per trigger,
  and the switch between them is animated. When used, set `width: var(--positioner-width)` and
  `height: var(--positioner-height)` on the `Positioner` so its box is frozen to the measured size
  during the transition; otherwise content-driven resizing can make the popup thrash or flip to
  another side." The optional/multi-trigger framing is consistent with behavior.md "DOM structure
  & portal behavior" and "Edge cases (rapid interactions, unmount, nesting)"; the CSS-variable
  freezing technique and the thrash/flip failure mode are docs-only with no test counterpart; see
  Discrepancies.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:221`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the eight parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesPreviewCard.Root />`,
  `### Trigger` → `<TypesPreviewCard.Trigger />`, `### Portal` → `<TypesPreviewCard.Portal />`,
  `### Backdrop` → `<TypesPreviewCard.Backdrop />`,
  `### Positioner` → `<TypesPreviewCard.Positioner />`, `### Popup` → `<TypesPreviewCard.Popup />`,
  `### Viewport` → `<TypesPreviewCard.Viewport />`, `### Arrow` → `<TypesPreviewCard.Arrow />`.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:189-225`
- The `## createHandle` section (outside the API reference) documents
  `<TypesPreviewCard.createHandle />` and, after the `@exclude-table-of-contents` comment,
  `<TypesPreviewCard.Handle />` under `### Handle`.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:227-235`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesPreviewCard } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:191`
- Parts documented on this page: Root, Trigger, Portal, Backdrop, Positioner, Popup, Viewport,
  Arrow, plus `createHandle` and its `Handle` type — the same part set recorded in behavior.md
  "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:189-235`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- Five fenced code blocks total:
  1. Anatomy (` ```jsx title="Anatomy" `): namespace import from `@base-ui/react/preview-card` and
     full part assembly (Root > Trigger; Portal > Backdrop + Positioner > Popup > Arrow + Viewport).
     `docs/src/app/(docs)/react/components/preview-card/page.mdx:29-44`
  2. Detached triggers (` ```jsx title="Detached triggers" `): `PreviewCard.createHandle()`
     assigned to a const, `<PreviewCard.Trigger handle={demoPreviewCard} href="#">`, and a detached
     `<PreviewCard.Root handle={demoPreviewCard}>`, with `@highlight` /
     `@highlight-text "handle={demoPreviewCard}"` markers.
     `docs/src/app/(docs)/react/components/preview-card/page.mdx:59-73`
  3. Multiple triggers within the Root part (` ```jsx title="Multiple triggers within the Root
     part" `): two `<PreviewCard.Trigger href="#">` inside one `<PreviewCard.Root>`.
     `docs/src/app/(docs)/react/components/preview-card/page.mdx:84-90`
  4. Multiple detached triggers (` ```jsx title="Multiple detached triggers" `): one handle shared
     by two `<PreviewCard.Trigger handle={demoPreviewCard} href="#">` elements plus a detached
     `<PreviewCard.Root handle={demoPreviewCard}>`.
     `docs/src/app/(docs)/react/components/preview-card/page.mdx:92-106`
  5. Detached triggers with payload (` ```jsx title="Detached triggers with payload" `): typed
     `PreviewCard.createHandle<{ title: string }>()`, two payload-carrying detached triggers, and a
     function-children `<PreviewCard.Root handle={demoPreviewCard}>` destructuring `{ payload }`
     to render content conditionally on `payload !== undefined`, with `@highlight` /
     `@highlight-text "payload"` markers.
     `docs/src/app/(docs)/react/components/preview-card/page.mdx:113-144`
- No other code blocks exist in the page. The controlled-mode and animation examples render demo
  components (`<DemoPreviewCardHero />`, `<DemoPreviewCardDetachedTriggersSimple />`,
  `<DemoPreviewCardDetachedTriggersControlled />`, `<DemoPreviewCardDetachedTriggersFull />`)
  imported from `./demos/*`; their code lives in demo files and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:11-13`, `docs/src/app/(docs)/react/components/preview-card/page.mdx:75-77`, `docs/src/app/(docs)/react/components/preview-card/page.mdx:155-157`, `docs/src/app/(docs)/react/components/preview-card/page.mdx:185-187`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and
`specs/library/preview-card/behavior.md`. Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: popup content "is not touch, keyboard or screen
  reader navigable" and "acts as a visual progressive enhancement for sighted mouse and keyboard
  users only". behavior.md's "Focus management" section marks "focus is not moved into the popup on
  open" as UNVERIFIED and no test asserts popup non-navigability. Not contradicted (behavior.md's
  keyboard coverage is trigger-level: focus opens, blur/Escape close), but unverified by tests.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:23`
- Docs-only claim, not covered by behavior.md: the screen-reader framing in the subtitle and meta
  description ("without interrupting keyboard or screen reader navigation") and the usage-guideline
  rationale that exposing each preview to screen readers "would force users through its contents
  before they could continue through the page".
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:3-5`, `docs/src/app/(docs)/react/components/preview-card/page.mdx:6-9`, `docs/src/app/(docs)/react/components/preview-card/page.mdx:22`
- Docs-only claim, not covered by behavior.md: per-mount fresh state on the handle — "a call made
  while no root was attached is not replayed, and no open state carries over from a previous
  mount". behavior.md's "State model (controlled/uncontrolled, defaults, transitions)" records
  that calls with no mounted root are ignored, but records neither replay suppression nor
  state-carry-over semantics.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:57`
- Docs-only claim, not covered by behavior.md: the `--positioner-width` / `--positioner-height`
  freezing technique and the "thrash or flip to another side" failure mode. behavior.md's nearest
  recorded facts are the Viewport-driven switch from inline `transform` to `top`/`left`
  positioning ("DOM structure & portal behavior"); no test covers the CSS variables or the resize
  behavior during content transitions.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:221`
- Omission (docs page vs. behavior.md): behavior.md's "State model (controlled/uncontrolled,
  defaults, transitions)" records a console warning ("no root using this handle is mounted") when
  imperative handle calls are made with no mounted root; the page says such calls "are ignored"
  without mentioning the warning. Omission of a developer-facing caveat, not a contradiction of
  the ignored-call outcome.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:56-57`
- Omission (docs page vs. behavior.md): behavior.md's "State model (controlled/uncontrolled,
  defaults, transitions)" records that `handle.open('missing')` throws synchronously when the
  trigger id is not registered; the page's handle discussion does not mention this failure mode.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:56`
- Omission (docs page vs. behavior.md): behavior.md's "Edge cases (rapid interactions, unmount,
  nesting)" records that during viewport transitions the previous container is `inert` under a
  `[data-transitioning]` ancestor and that both transition containers are cleaned up after the
  animation finishes; the page documents only `data-activation-direction`, `data-current`, and
  `data-previous`.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:176-183`
- Note (not a mismatch): the page's Anatomy snippet shows both `PreviewCard.Arrow` and
  `PreviewCard.Viewport` inside the same Popup; behavior.md's composition summary ("DOM structure
  & portal behavior") writes the optional inner content as Arrow-or-Viewport. The page is
  illustrating the full part assembly rather than a required nesting, and behavior.md's per-part
  coverage ("Public API surface (props, parts, subcomponents)") treats Arrow and Viewport as
  independent optional parts.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:29-44`

## Cross-links to other docs pages

- N/A — the page contains no internal links to other Base UI documentation pages.
- One external link: MDN glossary entry for progressive enhancement
  (`https://developer.mozilla.org/en-US/docs/Glossary/Progressive_Enhancement`), used in the "Keep
  popup content supplementary" usage guideline.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:23`
- One external resource (not a docs link): `<link as="image" rel="preload">` pointing at an
  `images.unsplash.com` photo URL, preloading the hero demo's preview image.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:14-18`
- Demo components (`./demos/hero`, `./demos/detached-triggers-simple`,
  `./demos/detached-triggers-controlled`, `./demos/detached-triggers-full`) and the `./types`
  reference module are imported and rendered on this page itself; they are same-page imports, not
  cross-page links.
  `docs/src/app/(docs)/react/components/preview-card/page.mdx:11`, `docs/src/app/(docs)/react/components/preview-card/page.mdx:75`, `docs/src/app/(docs)/react/components/preview-card/page.mdx:155`, `docs/src/app/(docs)/react/components/preview-card/page.mdx:185`, `docs/src/app/(docs)/react/components/preview-card/page.mdx:191`
