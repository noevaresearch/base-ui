# Scroll Area docs page content spec

Mined from `docs/src/app/(docs)/react/components/scroll-area/page.mdx` only. The component's own
behavior is covered by `specs/library/scroll-area/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/scroll-area/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/scroll-area/demos.json`).

## Page structure (headings, in order)

- `# Scroll Area` (h1) — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:14`
- `## Examples` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:32`
  - `### Both scrollbars` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:34`
  - `### Gradient scroll fade` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:42`
  - `### Combining with Tabs` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:79`
- `## API reference` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:96`
  - `### Root` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:100`
  - `### Viewport` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:104`
  - `### Content` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:108`
  - `### Scrollbar` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:112`
  - `### Thumb` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:116`
  - `### Corner` — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:120`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A native scroll container with custom scrollbars." — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React scroll area that provides a native scroll container with custom scrollbars." — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:5-8`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:10-12`
- Demo imports interleaved with the Examples subsections (`./demos/both` at `docs/src/app/(docs)/react/components/scroll-area/page.mdx:38`, `./demos/scroll-fade` at `docs/src/app/(docs)/react/components/scroll-area/page.mdx:44`; note the ScrollFade demo is imported early but rendered late, at `docs/src/app/(docs)/react/components/scroll-area/page.mdx:77`, after the gradient-fade CSS snippets)
- `TypesScrollArea` import for the API reference — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:98`
- Trailing `export const metadata` SEO keywords block (15 keywords, e.g. 'React Scroll Area', 'Custom Scrollbars', 'Gradient Scroll Fade') — `docs/src/app/(docs)/react/components/scroll-area/page.mdx:124-142`

## Prose claims about component behavior

- Page describes the component as "A native scroll container with custom scrollbars." and, in the
  meta description, as "high-quality" and "unstyled". Naming-level claims; consistent with the
  six-part API (Root/Viewport/Content/Scrollbar/Thumb/Corner) in behavior.md "Public API surface
  (props, parts, subcomponents)", and the "native scroll container" framing is consistent with the
  native-scroll behavior recorded in behavior.md "Events" (viewport `scrollTop`/`scrollLeft`
  driven by wheel/track/thumb gestures rather than a synthetic scroll model).
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:3`, `docs/src/app/(docs)/react/components/scroll-area/page.mdx:5-8`
- Anatomy usage guidance: "Import the component and assemble its parts", showing the assembly
  `ScrollArea.Root` > `ScrollArea.Viewport` > `ScrollArea.Content`, with `ScrollArea.Scrollbar`
  (containing `ScrollArea.Thumb`) and `ScrollArea.Corner` as siblings of the Viewport, imported
  from the `@base-ui/react/scroll-area` namespace. The part set and namespace import match
  behavior.md "Public API surface (props, parts, subcomponents)"; the nesting matches that spec's
  context requirements (Content requires Viewport context, Thumb requires Scrollbar context) and
  its note that arbitrary children, including `ScrollArea.Content`, are the scrollable content.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:16`, `docs/src/app/(docs)/react/components/scroll-area/page.mdx:18-30`
- "Use `<ScrollArea.Corner>` to prevent the scrollbars from intersecting." The corner's existence
  and sizing-to-the-scrollbars match behavior.md "State model (controlled/uncontrolled, defaults,
  transitions)" (corner geometry equals the scrollbars' cross-axis sizes) and "DOM structure &
  portal behavior" (corner presence tracks overflow); the anti-intersection purpose phrasing is
  docs-only.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:36`
- "Use the viewport overflow CSS variables to drive a CSS mask, which gradually increases the fade
  as the user scrolls away from the edges." Matches behavior.md "State model (controlled/
  uncontrolled, defaults, transitions)" — scroll metrics are exposed as CSS variables on the
  Viewport reflecting the live scroll offset and remaining overflow, reset to `0px` when content
  stops overflowing. The page's mask uses the y-axis pair `--scroll-area-overflow-y-start` /
  `--scroll-area-overflow-y-end` (behavior.md's examples are the x-axis pair; the naming scheme
  and semantics are the same family).
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:46`
- "For SSR, a fallback can be used as part of the end-side `var()` call so the mask is visible
  before the overflow CSS variables hydrate." Not covered by behavior.md (no test addresses
  server rendering or hydration); see Discrepancies.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:61`
- "When the fade is applied to `<ScrollArea.Viewport>` itself, the variables can be used directly.
  However, inheritance to children is disabled, so they must explicitly opt-in using the `inherit`
  keyword." Not covered by behavior.md (no test addresses CSS-variable inheritance); see
  Discrepancies.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:68`
- "Use `<Tabs.List>`'s `render` prop to render `<ScrollArea.Viewport>` directly when the tab list
  itself needs the viewport overflow values for a mask fade. This keeps the mask logic on the same
  element that receives the scroll state." Cross-component integration guidance (Tabs is outside
  behavior.md's scope); consistent with that spec's placement of the overflow CSS variables on the
  Viewport element ("State model (controlled/uncontrolled, defaults, transitions)"). See
  Discrepancies for the coverage gap.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:81`
- The page prose documents no props (no mention of `overflowEdgeThreshold`, `keepMounted`,
  `orientation`, `onScroll`, or any other prop) and no `data-*` attributes (no mention of
  `data-scrolling`, `data-has-overflow-x/y`, `data-overflow-*-start/end`, or `data-hovering`);
  styling state is conveyed only through the overflow CSS variables. See Discrepancies for the
  omission relative to behavior.md "State model (controlled/uncontrolled, defaults, transitions)".

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the six parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesScrollArea.Root />`,
  `### Viewport` → `<TypesScrollArea.Viewport />`, `### Content` → `<TypesScrollArea.Content />`,
  `### Scrollbar` → `<TypesScrollArea.Scrollbar />`, `### Thumb` → `<TypesScrollArea.Thumb />`,
  `### Corner` → `<TypesScrollArea.Corner />`.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:96-122`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesScrollArea } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:98`
- Parts documented on this page: Root, Viewport, Content, Scrollbar, Thumb, Corner — the same
  six-part set recorded in behavior.md "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:100-122`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- Five fenced code blocks total:
  1. Anatomy snippet (` ```jsx title="Anatomy" `): imports `{ ScrollArea }` from
     `@base-ui/react/scroll-area` and assembles `<ScrollArea.Root>` > `<ScrollArea.Viewport>`
     containing `<ScrollArea.Content />`, with `<ScrollArea.Scrollbar>` containing
     `<ScrollArea.Thumb />` and `<ScrollArea.Corner />` as siblings of the Viewport.
     `docs/src/app/(docs)/react/components/scroll-area/page.mdx:18-30`
  2. CSS mask snippet (` ```css title="scroll-area.module.css" `): a `.Viewport` class with
     `mask-image: linear-gradient(to bottom, transparent 0, black min(40px,
     var(--scroll-area-overflow-y-start)), black calc(100% - min(40px,
     var(--scroll-area-overflow-y-end, 40px))), transparent 100%)` and `mask-repeat: no-repeat`,
     i.e. the fade width at each edge is capped at 40px and driven by the overflow variables.
     `docs/src/app/(docs)/react/components/scroll-area/page.mdx:48-59`
  3. SSR fallback snippet (` ```css title="SSR fallback" `): `var(--scroll-area-overflow-y-end,
     40px);` showing the end-side fallback value, with a `@highlight-text ", 40px"` directive
     comment inside the block. `docs/src/app/(docs)/react/components/scroll-area/page.mdx:63-66`
  4. Child element opt-in snippet (` ```css title="Child element opt-in" `): a `.Child` class
     setting `--scroll-area-overflow-y-start: inherit;` and `--scroll-area-overflow-y-end:
     inherit;` — the opt-in mechanism for the disabled inheritance described in the prose.
     `docs/src/app/(docs)/react/components/scroll-area/page.mdx:70-75`
  5. Tabs integration snippet (` ```jsx title="Tabs with ScrollArea" `): `<Tabs.Root
     defaultValue="overview">` wrapping a `<ScrollArea.Root>` whose `<Tabs.List
     render={<ScrollArea.Viewport />}>` contains `<Tabs.Tab value="overview">` and
     `<Tabs.Indicator />`, with a sibling `<Tabs.Panel value="overview">`; a `{/* @highlight */}`
     comment marks the `render` prop line.
     `docs/src/app/(docs)/react/components/scroll-area/page.mdx:83-94`
- No other code blocks exist in the page. The "Both scrollbars" example renders
  `<DemoScrollAreaBoth />` and the "Gradient scroll fade" example renders
  `<DemoScrollAreaScrollFade />` (plus `<DemoScrollAreaHero />` at the top), imported from
  `./demos/*`; their code lives in demo files and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:10-12`, `docs/src/app/(docs)/react/components/scroll-area/page.mdx:38-40`, `docs/src/app/(docs)/react/components/scroll-area/page.mdx:44`, `docs/src/app/(docs)/react/components/scroll-area/page.mdx:77`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/scroll-area/behavior.md`.
Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: the overflow CSS variables are unset server-side
  and only available after hydration, so an SSR-visible fallback value is needed for the end-side
  `var()` call. behavior.md's "State model (controlled/uncontrolled, defaults, transitions)"
  records the variables' client-side values and reset behavior but has no SSR/hydration coverage.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:61`
- Docs-only claim, not covered by behavior.md: the overflow CSS variables do not inherit to
  children of the Viewport (inheritance is disabled), and children must opt in with the `inherit`
  keyword. No test recorded in behavior.md covers CSS-variable inheritance either way.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:68`
- Docs-only claim, not covered by behavior.md: composing `<ScrollArea.Viewport>` onto
  `<Tabs.List>` via the Tabs `render` prop so the tab list itself receives the overflow values.
  behavior.md does not cover Tabs integration (it mines scroll-area tests only); the claim is
  consistent with that spec's placement of the variables on the Viewport ("State model").
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:81`
- Note (not a mismatch): the page states the Corner's purpose is to "prevent the scrollbars from
  intersecting"; behavior.md "State model (controlled/uncontrolled, defaults, transitions)" and
  "DOM structure & portal behavior" cover the corner's geometry (sized to the scrollbars) and its
  presence tracking overflow, but never state the anti-intersection purpose.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:36`
- Omission (docs page vs. behavior.md): behavior.md "State model (controlled/uncontrolled,
  defaults, transitions)" records a rich `data-*` state surface — `data-scrolling`,
  `data-has-overflow-x`/`-y`, `data-overflow-x/y-start/end`, `data-hovering` — that the page never
  mentions, even though the overflow CSS variables it does document are the CSS-variable half of
  that same state. Omission, not contradiction.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:46`
- Omission (docs page vs. behavior.md): no props are described anywhere in the page prose
  (behavior.md "Public API surface (props, parts, subcomponents)" records `overflowEdgeThreshold`,
  `keepMounted`, `orientation`, `onScroll` as part of the tested API); prop documentation is
  delegated entirely to the generated API tables. Omission from prose, not contradiction.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:16`, `docs/src/app/(docs)/react/components/scroll-area/page.mdx:96-122`

## Cross-links to other docs pages

- N/A — the page contains no internal links to other Base UI documentation pages and no external
  links.
- Same-page imports only: demo components (`./demos/hero`, `./demos/both`, `./demos/scroll-fade`)
  and the generated API reference (`./types`) are imported and rendered on this page itself; they
  are same-page imports, not cross-page links.
  `docs/src/app/(docs)/react/components/scroll-area/page.mdx:10`, `docs/src/app/(docs)/react/components/scroll-area/page.mdx:38`, `docs/src/app/(docs)/react/components/scroll-area/page.mdx:44`, `docs/src/app/(docs)/react/components/scroll-area/page.mdx:98`
