# Menubar docs-content page spec

Mined from the docs page only: `docs/src/app/(docs)/react/components/menubar/page.mdx` (79 lines).
Component behavior itself is covered by `specs/library/menubar/behavior.md` and is referenced
below by section name (§ ...) rather than restated. Demo source files (`./demos/*`) are intentionally
not opened here — Stage 2 (docs) mines them into `specs/docs-content/menubar/demos.json`.

## Page structure (headings, in order)

- `# Menubar` — `docs/src/app/(docs)/react/components/menubar/page.mdx:1`
- Subtitle: "A menu bar providing commands and options for your application." — `docs/src/app/(docs)/react/components/menubar/page.mdx:3`
- `<Meta name="description">` block — `docs/src/app/(docs)/react/components/menubar/page.mdx:4-7`
- Hero demo: imports `DemoMenubarHero` from `./demos/hero` and renders it — `docs/src/app/(docs)/react/components/menubar/page.mdx:9-11`
- `## Anatomy` — `docs/src/app/(docs)/react/components/menubar/page.mdx:13` (assembly sentence at `docs/src/app/(docs)/react/components/menubar/page.mdx:15`, fenced "Anatomy" snippet at `docs/src/app/(docs)/react/components/menubar/page.mdx:17-57`)
- `## API reference` — `docs/src/app/(docs)/react/components/menubar/page.mdx:59` (imports `TypesMenubar` from `./types` at `docs/src/app/(docs)/react/components/menubar/page.mdx:61` and renders `<TypesMenubar />` at `docs/src/app/(docs)/react/components/menubar/page.mdx:63`)
- `export const metadata` keywords block (SEO keywords, e.g. 'React Menubar', 'Application Menu Bar', 'Command Bar') — `docs/src/app/(docs)/react/components/menubar/page.mdx:65-79`

The page has no `## Usage guidelines`, no `## Examples`, and no `###` per-part headings — only the
two `##` sections above follow the hero demo.

## Prose claims about component behavior

- The page defines the component as "A menu bar providing commands and options for your application." (subtitle, restated verbatim as the meta description). — `docs/src/app/(docs)/react/components/menubar/page.mdx:3`, `docs/src/app/(docs)/react/components/menubar/page.mdx:6`
  Cross-check: behavior.md has no positioning/usage section to compare against; the framing is consistent with (and less specific than) behavior.md § Public API surface, where the menubar coordinates `Menu` parts whose items include commands (`Menu.Item`, `Menu.LinkItem`) and options (`Menu.RadioItem`, `Menu.CheckboxItem`). Not contradicted.
- Assembly instruction: "Import the component and assemble its parts:" — `docs/src/app/(docs)/react/components/menubar/page.mdx:15`
  Cross-check: consistent with behavior.md § Public API surface, which states the children API is composition of `Menu` parts.
- Implicit cross-package claim: the snippet imports `Menubar` from `@base-ui/react/menubar` and `Menu` from `@base-ui/react/menu` as two separate namespaces, and composes `<Menubar><Menu.Root>...` — `docs/src/app/(docs)/react/components/menubar/page.mdx:18-22`
  Cross-check: matches behavior.md's framing that `<Menubar />` is "a coordinator over `Menu.Root`/`Menu.Trigger`/`Menu.Portal` components imported from `@base-ui/react/menu`" (intro) and behavior.md § Public API surface.
- Implicit composition claim (from the snippet, the page's only behavioral content): `Menubar` wraps `Menu.Root`, which contains `Menu.Trigger` alongside `Menu.Portal`; the portal contains `Menu.Backdrop` and `Menu.Positioner`; the positioner wraps `Menu.Popup`, which holds `Menu.Arrow`, `Menu.Item`, `Menu.LinkItem`, `Menu.Separator`, a `Menu.SubmenuRoot` > `Menu.SubmenuTrigger` pair, a `Menu.Group` > `Menu.GroupLabel` pair, a `Menu.RadioGroup` > `Menu.RadioItem` > `Menu.RadioItemIndicator` chain, a `Menu.CheckboxItem` > `Menu.CheckboxItemIndicator` pair, and `Menu.Viewport`. — `docs/src/app/(docs)/react/components/menubar/page.mdx:21-56`
  Cross-check: consistent with behavior.md § Public API surface (tested children composition: `Menu.Root`, `Menu.Trigger`, `Menu.Portal`, `Menu.Positioner`, `Menu.Popup`, `Menu.Item`, `Menu.SubmenuRoot`, `Menu.SubmenuTrigger`, `Menu.RadioGroup`, `Menu.RadioItem`) and behavior.md § DOM structure & portal behavior (portals via `Menu.Portal`/`Menu.Positioner`). The single contained `Menu.Root` inside `<Menubar>` matches behavior.md's "contained triggers" fixture variant; the page shows no detached-trigger usage (behavior.md also covers detached and multi-trigger variants — a coverage gap, see Discrepancies).
- No other prose claims exist: the page text makes no prop descriptions, no usage guidance, no keyboard/a11y statements, and no caveats beyond the snippet itself.

## API tables referenced (props/parts documented on this page)

The `## API reference` section contains no `###` part headings and no inline prop tables: all API
documentation is delegated to a single `<TypesMenubar />` component imported from `./types`
(`docs/src/app/(docs)/react/components/menubar/page.mdx:61-63`). The .mdx text itself therefore
documents zero named props, and there is no page-stated prop text to cross-check against
behavior.md § Public API surface (which asserts `loopFocus`, `disabled`, `orientation`, `modal`,
`style`).

- Parts named in the page text (Anatomy snippet only): `Menubar` (from `@base-ui/react/menubar`, `docs/src/app/(docs)/react/components/menubar/page.mdx:18`) and the `Menu` parts `Root`, `Trigger`, `Portal`, `Backdrop`, `Positioner`, `Popup`, `Arrow`, `Item`, `LinkItem`, `Separator`, `SubmenuRoot`, `SubmenuTrigger`, `Group`, `GroupLabel`, `RadioGroup`, `RadioItem`, `RadioItemIndicator`, `CheckboxItem`, `CheckboxItemIndicator`, `Viewport` (`docs/src/app/(docs)/react/components/menubar/page.mdx:21-56`).
- Coverage note: of these, the parts exercised by the mined tests (behavior.md § Public API surface) are Menubar, `Menu.Root`, `Menu.Trigger`, `Menu.Portal`, `Menu.Positioner`, `Menu.Popup`, `Menu.Item`, `Menu.SubmenuRoot`, `Menu.SubmenuTrigger`, `Menu.RadioGroup`, `Menu.RadioItem`. `Menu.Backdrop`, `Menu.Arrow`, `Menu.LinkItem`, `Menu.Separator`, `Menu.Group`, `Menu.GroupLabel`, `Menu.RadioItemIndicator`, `Menu.CheckboxItem`, `Menu.CheckboxItemIndicator`, and `Menu.Viewport` appear in the Anatomy but are untested (see Discrepancies).

## Code snippets embedded directly in the .mdx (not pulled from demos/)

Exactly one fenced snippet, titled `Anatomy` (```` ```jsx title="Anatomy" ````, `docs/src/app/(docs)/react/components/menubar/page.mdx:17`):

- Two namespace imports: `import { Menubar } from '@base-ui/react/menubar';` — `docs/src/app/(docs)/react/components/menubar/page.mdx:18` — and `import { Menu } from '@base-ui/react/menu';` — `docs/src/app/(docs)/react/components/menubar/page.mdx:19`
- Full composition skeleton: `Menubar` > `Menu.Root` > `Menu.Trigger`; `Menu.Portal` > `Menu.Backdrop` + `Menu.Positioner` > `Menu.Popup` containing `Menu.Arrow`, `Menu.Item`, `Menu.LinkItem`, `Menu.Separator`, `Menu.SubmenuRoot` > `Menu.SubmenuTrigger`, `Menu.Group` > `Menu.GroupLabel`, `Menu.RadioGroup` > `Menu.RadioItem` > `Menu.RadioItemIndicator`, `Menu.CheckboxItem` > `Menu.CheckboxItemIndicator`, and `Menu.Viewport` — `docs/src/app/(docs)/react/components/menubar/page.mdx:21-56`
- Factual quirk: the snippet's closing tag carries a trailing semicolon, `</Menubar>;` — `docs/src/app/(docs)/react/components/menubar/page.mdx:56`
- No props are passed to any element in the snippet (bare self-closing parts throughout, `docs/src/app/(docs)/react/components/menubar/page.mdx:21-56`).

Non-snippet MDX constructs embedded in the page (not component documentation): the hero demo
import/render call (`docs/src/app/(docs)/react/components/menubar/page.mdx:9-11` — demo internals
are Stage 2 scope), the `TypesMenubar` import and render
(`docs/src/app/(docs)/react/components/menubar/page.mdx:61-63`), and the `export const metadata`
keywords block (`docs/src/app/(docs)/react/components/menubar/page.mdx:65-79`).

## Discrepancies (docs page vs. behavior.md)

No contradictions found. Cross-check notes:

1. Composition superset: the Anatomy shows ten `Menu` parts not exercised by the mined tests —
   `Backdrop`, `Arrow`, `LinkItem`, `Separator`, `Group`, `GroupLabel`, `RadioItemIndicator`,
   `CheckboxItem`, `CheckboxItemIndicator`, `Viewport`
   (`docs/src/app/(docs)/react/components/menubar/page.mdx:25-51`); behavior.md § Public API surface
   lists only the tested children composition. Coverage gap, not mismatch — the extra parts conflict
   with no tested claim.
2. Single-trigger skeleton: the snippet shows one contained `Menu.Root` with one `Menu.Trigger`
   (`docs/src/app/(docs)/react/components/menubar/page.mdx:21-23`); behavior.md documents three
   structural variants (contained, detached, multiple contained triggers) plus `Menubar.Props`
   usage, but the page makes no claims about any variant, so nothing conflicts.
3. Zero prop prose: the page names no props anywhere in its text (single delegated
   `<TypesMenubar />`, `docs/src/app/(docs)/react/components/menubar/page.mdx:61-63`), so nothing on
   the page can contradict behavior.md § Public API surface's asserted props (`loopFocus`,
   `disabled`, `orientation`, `modal`, `style`) or behavior.md § Keyboard interactions /
   § Focus management / § Accessibility, none of which the page text touches.
4. "Providing commands and options for your application"
   (`docs/src/app/(docs)/react/components/menubar/page.mdx:3`,
   `docs/src/app/(docs)/react/components/menubar/page.mdx:6`) has no direct counterpart in
   behavior.md; it is consistent with the tested item surface (items, radio/checkbox options) and
   not contradicted by any tested behavior.

## Cross-links to other docs pages

N/A — the page contains no markdown links to other docs pages. All module imports are local
(`./demos/hero` at `docs/src/app/(docs)/react/components/menubar/page.mdx:9` and `./types` at
`docs/src/app/(docs)/react/components/menubar/page.mdx:61`).
