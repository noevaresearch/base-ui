# Navigation Menu docs page content spec

Mined from `docs/src/app/(docs)/react/components/navigation-menu/page.mdx` only. The component's own
behavior is covered by `specs/library/navigation-menu/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/navigation-menu/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/navigation-menu/demos.json`).

## Page structure (headings, in order)

- `# Navigation Menu` (h1) — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:13`
- `## Examples` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:44`
  - `### Nested submenus` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:46`
  - `### Nested inline submenus` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:54`
  - `### Custom links` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:62`
  - `### Large menus` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:82`
- `## API reference` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:113`
  - `### Root` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:117`
  - `### List` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:121`
  - `### Item` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:125`
  - `### Trigger` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:129`
  - `### Icon` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:133`
  - `### Content` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:137`
  - `### Link` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:141`
  - `### Backdrop` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:145`
  - `### Portal` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:149`
  - `### Positioner` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:153`
  - `### Popup` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:157`
  - `### Viewport` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:161`
  - `### Arrow` — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:165`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A collection of links and menus for website navigation." — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React navigation menu component that displays a collection of links and menus for website navigation." — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:4-7`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:9-11`
- Demo imports interleaved with the Examples subsections (`./demos/nested` at `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:50`, `./demos/nested-inline` at `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:58`)
- `TypesNavigationMenu` import for the API reference — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:115`
- Trailing `export const metadata` SEO keywords block (13 keywords, e.g. 'React Navigation Menu', 'Mega Menu React', 'Flyout Menu') — `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:169-185`

## Prose claims about component behavior

- Page describes the component as "A collection of links and menus for website navigation." and, in
  the meta description, as "high-quality" and "unstyled". Naming-level claim only; consistent with
  the thirteen-part namespace API in behavior.md "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:3`, `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:4-7`
- Anatomy usage guidance: "Import the component and assemble its parts", showing the assembly
  `NavigationMenu.Root` > `NavigationMenu.List` > `NavigationMenu.Item` > `NavigationMenu.Trigger`
  (wrapping `NavigationMenu.Icon`) with `NavigationMenu.Content` (containing
  `NavigationMenu.Link`) inside the Item, plus a separate
  `NavigationMenu.Portal` > `NavigationMenu.Backdrop` + `NavigationMenu.Positioner` >
  `NavigationMenu.Popup` (containing `NavigationMenu.Arrow` and `NavigationMenu.Viewport`),
  imported from the `@base-ui/react/navigation-menu` namespace. The part set and namespace import
  match behavior.md "Public API surface (props, parts, subcomponents)"; the typical structure there
  records Root → List → Item → Trigger + Content plus Portal → Positioner → Popup → Viewport, and
  does not itself assert the Icon-in-Trigger, Link-in-Content, Backdrop, or Arrow placement shown
  here (all those parts exist in the recorded part set, so this is consistent, just finer-grained).
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:15-42`
- "`<NavigationMenu.Root>` component can be nested within a higher-level `<NavigationMenu.Content>`
  part to create a multi-level navigation menu." Matches behavior.md "DOM structure & portal
  behavior" (inline nested roots: a `Root` inside a parent's `Content` with its own `List` +
  `Viewport`, no Portal, works without a positioner/popup) and "Keyboard interactions" (cycling
  works across 3 levels of nested Roots).
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:48`
- "For second-level navigation that should stay in the same panel, omit the nested
  `<NavigationMenu.Portal>` and render only `List` + `Viewport` with a `defaultValue`." Matches
  behavior.md "DOM structure & portal behavior" (inline nested roots need no Portal/Positioner/Popup)
  and "State model (controlled/uncontrolled, defaults, transitions)" (`defaultValue` opens that item
  on mount).
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:56`
- "The `<NavigationMenu.Link>` part can be customized to render the link from your framework using
  the `render` prop to enable client-side routing." Consistent with behavior.md "Public API surface
  (props, parts, subcomponents)" (every part passes the conformance suite's render-prop test) and
  "Accessibility" (links keep their native `link` role). behavior.md does not itself exercise
  framework-router integration; the routing claim is docs guidance.
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:64`
- Large menus: "When you have large menu content that doesn't fit in the viewport in some cases, you
  usually have two choices" — compress the content, or make the menu scrollable. For compressing:
  "change the layout ... to render less content or be more compact" and, if the content is flexible,
  "use the `max-height` property on `.Popup` to limit the height of the navigation menu to let it
  compress itself while preventing overflow." Not covered by behavior.md (no recorded counterpart);
  see Discrepancies.
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:84-89`
- For the scrollable option, the page prescribes the same `max-height` via CSS plus
  `overflow-y: auto` on `.Content` (see Code snippets for the blocks). Not covered by behavior.md;
  see Discrepancies. `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:98-109`
- "Native scrollbars are visible while transitioning content, so we recommend using the
  [Scroll Area](/react/components/scroll-area) component instead of native scrollbars to keep them
  hidden, which also allows the `Arrow` to be centered correctly." Not covered by behavior.md; see
  Discrepancies. `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:111`
- Omission note (not a claim): the page's prose documents only `defaultValue` and the `render` prop;
  it contains no interaction or timing guidance (nothing about hover open/close delays, controlled
  `value`/`onValueChange`, `orientation`, keyboard, or focus behavior), all of which behavior.md
  records under "State model (controlled/uncontrolled, defaults, transitions)", "Keyboard
  interactions", and "Focus management".
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:44-111`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the thirteen parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesNavigationMenu.Root />`,
  `### List` → `<TypesNavigationMenu.List />`, `### Item` → `<TypesNavigationMenu.Item />`,
  `### Trigger` → `<TypesNavigationMenu.Trigger />`, `### Icon` → `<TypesNavigationMenu.Icon />`,
  `### Content` → `<TypesNavigationMenu.Content />`, `### Link` → `<TypesNavigationMenu.Link />`,
  `### Backdrop` → `<TypesNavigationMenu.Backdrop />`, `### Portal` → `<TypesNavigationMenu.Portal />`,
  `### Positioner` → `<TypesNavigationMenu.Positioner />`, `### Popup` → `<TypesNavigationMenu.Popup />`,
  `### Viewport` → `<TypesNavigationMenu.Viewport />`, `### Arrow` → `<TypesNavigationMenu.Arrow />`.
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:113-167`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesNavigationMenu } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:115`
- Parts documented on this page: Root, List, Item, Trigger, Icon, Content, Link, Backdrop, Portal,
  Positioner, Popup, Viewport, Arrow — the same thirteen-part set recorded in behavior.md "Public
  API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:117-167`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- Anatomy snippet (` ```jsx title="Anatomy" `): imports `{ NavigationMenu }` from
  `@base-ui/react/navigation-menu` and assembles the full part tree —
  `<NavigationMenu.Root>` > `<NavigationMenu.List>` > `<NavigationMenu.Item>` >
  `<NavigationMenu.Trigger>` (containing `<NavigationMenu.Icon />`) with `<NavigationMenu.Content>`
  (containing `<NavigationMenu.Link />`) inside the Item, plus `<NavigationMenu.Portal>` containing
  `<NavigationMenu.Backdrop />` and `<NavigationMenu.Positioner>` >
  `<NavigationMenu.Popup>` (containing `<NavigationMenu.Arrow />` and `<NavigationMenu.Viewport />`).
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:17-42`
- Next.js example snippet (` ```jsx title="Next.js example" `): defines a custom `Link` component
  typed `NavigationMenu.Link.Props` that wraps `next/link` via
  `render={<NextLink href={props.href} />}` and spreads the remaining props onto
  `<NavigationMenu.Link>`. Two `// @highlight` markers (before the import block and before the
  `render` line) flag the relevant lines. `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:66-80`
- CSS snippet (` ```css title="Compact layout" `): `.Content, .Popup { max-height: var(--available-height); }`
  for the compress option. `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:91-96`
- CSS snippet (` ```css title="Scrollable layout" `): the same `max-height` rule on `.Content` and
  `.Popup` plus `overflow-y: auto` on `.Content` for the scrollable option.
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:100-109`
- The Nested submenus and Nested inline submenus examples render demo components
  (`<DemoNavigationMenuNested />`, `<DemoNavigationMenuNestedInline />`, plus
  `<DemoNavigationMenuHero />` at the top) imported from `./demos/*`; their code lives in demo
  files and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:9-11`, `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:50-52`, `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:58-60`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/navigation-menu/behavior.md`.
Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: the `--available-height` CSS variable used in both
  large-menu CSS prescriptions. behavior.md "DOM structure & portal behavior" records only
  `--popup-width`/`--popup-height` and `--positioner-width`/`--positioner-height`; `--available-height`
  is never mentioned there.
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:91-96`, `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:100-109`
- Docs-only claim, not covered by behavior.md: "Native scrollbars are visible while transitioning
  content", the recommendation to use the Scroll Area component to keep them hidden, and the claim
  that Scroll Area "also allows the `Arrow` to be centered correctly". No behavior.md section
  addresses scrollbar visibility or Arrow centering.
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:111`
- Docs-only framing, not covered by behavior.md: the `render` prop on `NavigationMenu.Link`
  enabling client-side routing with a framework router. behavior.md verifies render-prop support via
  conformance ("Public API surface (props, parts, subcomponents)") but exercises no router
  integration. `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:64-80`
- Omission (docs page vs. behavior.md): the page prose documents no interaction behavior at all —
  hover open/close delays, controlled `value`/`onValueChange`, `orientation`, keyboard navigation,
  focus management, and nesting hover semantics recorded in behavior.md ("State model (controlled/
  uncontrolled, defaults, transitions)", "Keyboard interactions", "Focus management", "Edge cases
  (rapid interactions, unmount, nesting)") appear nowhere in the page text; prop documentation is
  delegated entirely to the generated API tables. Omission, not contradiction.
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:44-111`
- Note (not a mismatch): the Anatomy snippet declares `<NavigationMenu.Content>` inside the
  `<NavigationMenu.Item>` and `<NavigationMenu.Viewport>` inside the Popup; behavior.md "DOM
  structure & portal behavior" records that Content is moved into the Viewport at runtime when its
  item activates. Declarative authoring structure vs. runtime relocation — consistent, not
  contradictory. `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:20-40`

## Cross-links to other docs pages

- One internal docs link: [Scroll Area](/react/components/scroll-area), recommended as a replacement
  for native scrollbars inside large menus.
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:111`
- No external (non-Base UI) links appear on the page.
- Same-page imports are not cross-page links: `./demos/hero`, `./demos/nested`,
  `./demos/nested-inline` (demo components) and `./types` (generated API reference).
  `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:9`, `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:50`, `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:58`, `docs/src/app/(docs)/react/components/navigation-menu/page.mdx:115`
