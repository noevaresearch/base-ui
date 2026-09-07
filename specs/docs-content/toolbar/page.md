# Toolbar docs-content page spec

Mined from the docs page only: `docs/src/app/(docs)/react/components/toolbar/page.mdx` (139 lines).
Component behavior itself is covered by `specs/library/toolbar/behavior.md` and is referenced
below by section name (§ ...) rather than restated. Demo source files (`./demos/*`) are intentionally
not opened here — Stage 2 (docs) mines them into `specs/docs-content/toolbar/demos.json`.

## Page structure (headings, in order)

- `# Toolbar` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:1`
- Subtitle: "A container for grouping a set of buttons and controls." — `docs/src/app/(docs)/react/components/toolbar/page.mdx:3`
- `<Meta name="description">` block — `docs/src/app/(docs)/react/components/toolbar/page.mdx:4-7`
- Hero demo: imports `DemoToolbarHero` from `./demos/hero` and renders it — `docs/src/app/(docs)/react/components/toolbar/page.mdx:9-11`
- `## Usage guidelines` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:13` (intro sentence at `docs/src/app/(docs)/react/components/toolbar/page.mdx:15`, single guideline bullet at `docs/src/app/(docs)/react/components/toolbar/page.mdx:17`)
- `## Anatomy` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:19` (assembly sentence at `docs/src/app/(docs)/react/components/toolbar/page.mdx:21`, fenced "Anatomy" snippet at `docs/src/app/(docs)/react/components/toolbar/page.mdx:23-36`)
- `## Examples` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:38`, with three `###` subsections:
  - `### Using with Menu` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:40` (prose at `docs/src/app/(docs)/react/components/toolbar/page.mdx:42`, snippet at `docs/src/app/(docs)/react/components/toolbar/page.mdx:44-57`, applicability sentence at `docs/src/app/(docs)/react/components/toolbar/page.mdx:59`)
  - `### Using with Tooltip` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:61` (prose at `docs/src/app/(docs)/react/components/toolbar/page.mdx:63`, snippet at `docs/src/app/(docs)/react/components/toolbar/page.mdx:65-78`)
  - `### Using with NumberField` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:80` (prose at `docs/src/app/(docs)/react/components/toolbar/page.mdx:82`, snippet at `docs/src/app/(docs)/react/components/toolbar/page.mdx:84-97`)
- `## API reference` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:99` (imports `TypesToolbar` from `./types` at `docs/src/app/(docs)/react/components/toolbar/page.mdx:101`), with six `###` part headings: `### Root` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:103`, `### Button` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:107`, `### Link` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:111`, `### Input` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:115`, `### Group` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:119`, `### Separator` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:123`
- `export const metadata` keywords block (SEO keywords, e.g. 'React Toolbar', 'Action Bar', 'Command Strip', 'Accessible Toolbar') — `docs/src/app/(docs)/react/components/toolbar/page.mdx:127-139`

## Prose claims about component behavior

- Definition: the subtitle calls Toolbar "A container for grouping a set of buttons and controls." (`docs/src/app/(docs)/react/components/toolbar/page.mdx:3`), restated as the meta description "A high-quality, unstyled React toolbar component that groups a set of buttons and controls." (`docs/src/app/(docs)/react/components/toolbar/page.mdx:6`)
  Cross-check: consistent with (and less specific than) behavior.md § Public API surface, which exercises six parts grouped under one root. Not contradicted.
- Usage guideline, "Use inputs sparingly" (`docs/src/app/(docs)/react/components/toolbar/page.mdx:17`), makes two claims: (a) left/right arrow keys do double duty in horizontal toolbars — moving the text insertion cursor inside an input and navigating among controls; (b) an input in a horizontal toolbar should be used alone and placed as the last element.
  Cross-check for (a): matches behavior.md § Keyboard interactions, where arrow keys first move the caret/selection inside the input and focus only leaves at the text boundary. Cross-check for (b): usage guidance with no tested counterpart — behavior.md § Keyboard interactions covers single-input caret handling only; no test exercises multiple inputs or input placement. Guidance, not contradiction (see Discrepancies).
- Popup integration claim: "All Base UI popup components that provide a `Trigger` component can be integrated with a toolbar by passing the trigger to `<Toolbar.Button>` with the `render` prop" (`docs/src/app/(docs)/react/components/toolbar/page.mdx:42`), scoped by the following sentence to `<AlertDialog>`, `<Dialog>`, `<Menu>`, `<Popover>`, and `<Select>` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:59`).
  Cross-check: matches behavior.md § Events and § Focus management, where `Menu.Trigger`, `Select.Trigger`, `Dialog`/`AlertDialog`/`Popover` triggers are rendered via `Toolbar.Button`'s `render` prop and focus returns to the trigger on close. The enumerated set is exactly the popup set behavior.md tests. Consistent.
- Tooltip inversion claim: "Unlike other popups, the toolbar item should be passed to the `render` prop of `<Tooltip.Trigger>`" (`docs/src/app/(docs)/react/components/toolbar/page.mdx:63`).
  Cross-check: docs-only — behavior.md contains no Tooltip coverage anywhere (the tested popup set is Menu, Select, Dialog, AlertDialog, Popover per § Focus management). Coverage gap, flagged under Discrepancies; not contradicted.
- NumberField integration claim: "To use a NumberField in the toolbar, pass `<NumberField.Input>` to `<Toolbar.Input>` using the `render` prop" (`docs/src/app/(docs)/react/components/toolbar/page.mdx:82`).
  Cross-check: matches behavior.md § Public API surface (`Toolbar.Input` exercised with `render` of `NumberField.Input`) and behavior.md § Keyboard interactions (`ArrowUp`/`ArrowDown` on a rendered `NumberField.Input` increment/decrement the value rather than navigating the toolbar). Consistent.
- Implicit composition claims from the snippets: a single `Toolbar` namespace import from `@base-ui/react/toolbar` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:24`); `Toolbar.Root` directly contains `Toolbar.Button`, `Toolbar.Link`, `Toolbar.Separator`, `Toolbar.Group` (wrapping two more `Toolbar.Button`s), and `Toolbar.Input` as siblings (`docs/src/app/(docs)/react/components/toolbar/page.mdx:26-35`); `Toolbar.Button` accepts `render={<Menu.Trigger />}` with the popup remainder inside `Menu.Portal` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:46-53`); `Tooltip.Trigger` accepts `render={<Toolbar.Button />}` with the tooltip remainder inside `Tooltip.Portal` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:67-74`); `Toolbar.Input` with `render={<NumberField.Input />}` sits between `NumberField.Decrement` and `NumberField.Increment` inside `NumberField.Group` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:88-92`).
  Cross-check: composition matches behavior.md § Public API surface (all six parts, `render` on Button and Input, `NumberField.Input` composition); the Anatomy placing `Toolbar.Input` last mirrors the guideline at `docs/src/app/(docs)/react/components/toolbar/page.mdx:17`. Consistent.
- No other prose claims: the page text makes no statements about roving tabindex, `loopFocus`, wrapping, RTL, disabled-state ARIA conventions, `focusableWhenDisabled`, or separator orientation — all of which behavior.md § Keyboard interactions / § Focus management / § Accessibility document. The page's silence is a coverage gap, not a mismatch.

## API tables referenced (props/parts documented on this page)

The `## API reference` section has six `###` part headings — Root, Button, Link, Input, Group, Separator — each rendering a generated `<TypesToolbar.{Part} />` component imported from `./types` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:101-125`). The .mdx text itself inlines zero prop tables and documents zero named props; there is no page-stated prop text to cross-check against behavior.md § Public API surface (which exercises `orientation`, `dir`, `loopFocus`, `disabled`, `nativeButton`, `focusableWhenDisabled`, `render`, `defaultValue`, `type`, `href`).

- Parts named in the page text (Anatomy snippet + API reference headings): `Root`, `Button`, `Link`, `Separator`, `Group`, `Input` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:26-35`, `docs/src/app/(docs)/react/components/toolbar/page.mdx:103-125`).
- Ordering quirk: the Anatomy lists parts as Button, Link, Separator, Group, Input (`docs/src/app/(docs)/react/components/toolbar/page.mdx:27-34`) while the API reference orders them Root, Button, Link, Input, Group, Separator (`docs/src/app/(docs)/react/components/toolbar/page.mdx:103-123`). Factual note only — no behavioral claim attaches to either order.
- Cross-package parts named on the page (snippet-level, not API-documented): `Menu.Root`/`Menu.Trigger`/`Menu.Portal` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:47-53`), `Tooltip.Root`/`Tooltip.Trigger`/`Tooltip.Portal` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:68-74`), and `NumberField.Root`/`NumberField.Group`/`NumberField.Decrement`/`NumberField.Input`/`NumberField.Increment` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:87-93`).

## Code snippets embedded directly in the .mdx (not pulled from demos/)

Four fenced snippets:

1. Anatomy (```` ```jsx title="Anatomy" ````, `docs/src/app/(docs)/react/components/toolbar/page.mdx:23`):
   - Single namespace import: `import { Toolbar } from '@base-ui/react/toolbar';` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:24`
   - Composition skeleton: `Toolbar.Root` > `Toolbar.Button` + `Toolbar.Link` + `Toolbar.Separator` + `Toolbar.Group` (two `Toolbar.Button`s) + `Toolbar.Input` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:26-35`
   - Factual quirk: closing tag carries a trailing semicolon, `</Toolbar.Root>;` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:35`
   - No props are passed to any element (bare self-closing parts throughout) — `docs/src/app/(docs)/react/components/toolbar/page.mdx:26-35`
2. Menu integration (```` ```tsx title="Using popups with toolbar" ````, `docs/src/app/(docs)/react/components/toolbar/page.mdx:44`):
   - `return (`-wrapped JSX: `Toolbar.Root` > `Menu.Root` > `Toolbar.Button render={<Menu.Trigger />}` (the highlighted line) — `docs/src/app/(docs)/react/components/toolbar/page.mdx:45-49`
   - `Menu.Portal` containing only placeholder comments `{/* prettier-ignore */}` / `{/* Compose the rest of the menu */}` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:50-53`
   - Factual quirks: trailing semicolon inside the JSX close `</Toolbar.Root>;` before the `return`'s closing paren — `docs/src/app/(docs)/react/components/toolbar/page.mdx:55-56`
3. Tooltip integration (```` ```tsx title="Using popups with toolbar" ````, `docs/src/app/(docs)/react/components/toolbar/page.mdx:65` — the title duplicates snippet 2's):
   - Inverted composition: `Toolbar.Root` > `Tooltip.Root` > `Tooltip.Trigger render={<Toolbar.Button />}` (the highlighted line) — `docs/src/app/(docs)/react/components/toolbar/page.mdx:66-70`
   - `Tooltip.Portal` with the same placeholder-comment pattern — `docs/src/app/(docs)/react/components/toolbar/page.mdx:71-74`
   - Same trailing-semicolon quirk — `docs/src/app/(docs)/react/components/toolbar/page.mdx:76-77`
4. NumberField integration (```` ```tsx title="Using NumberField with toolbar" ````, `docs/src/app/(docs)/react/components/toolbar/page.mdx:84`):
   - `return (`-wrapped JSX: `Toolbar.Root` > `NumberField.Root` > `NumberField.Group` > `NumberField.Decrement` + `Toolbar.Input render={<NumberField.Input />}` (the highlighted line) + `NumberField.Increment` — `docs/src/app/(docs)/react/components/toolbar/page.mdx:85-95`
   - Same trailing-semicolon quirk — `docs/src/app/(docs)/react/components/toolbar/page.mdx:95-96`

Non-snippet MDX constructs embedded in the page (not component documentation): the hero demo
import/render call (`docs/src/app/(docs)/react/components/toolbar/page.mdx:9-11` — demo internals
are Stage 2 scope), the `TypesToolbar` import and its six per-part renders
(`docs/src/app/(docs)/react/components/toolbar/page.mdx:101-125`), and the `export const metadata`
keywords block (`docs/src/app/(docs)/react/components/toolbar/page.mdx:127-139`).

## Discrepancies (docs page vs. behavior.md)

No contradictions found. Cross-check notes:

1. Tooltip coverage gap: the page dedicates a full example to Tooltip integration with an
   inverted render direction — toolbar item inside `Tooltip.Trigger`'s `render` rather than the
   reverse (`docs/src/app/(docs)/react/components/toolbar/page.mdx:63`,
   `docs/src/app/(docs)/react/components/toolbar/page.mdx:67-74`) — but behavior.md has no
   Tooltip content at all (its popup set is Menu, Select, Dialog, AlertDialog, Popover per
   § Focus management / § Events). The inversion claim is untested; it conflicts with nothing.
2. "Use only one and place it as the last element" input guidance
   (`docs/src/app/(docs)/react/components/toolbar/page.mdx:17`) has no tested counterpart:
   behavior.md § Keyboard interactions covers single-input caret handling only. Guidance, not a
   tested claim; nothing contradicts it. The Anatomy snippet itself conforms (Input last,
   `docs/src/app/(docs)/react/components/toolbar/page.mdx:34`).
3. "All Base UI popup components that provide a `Trigger` component"
   (`docs/src/app/(docs)/react/components/toolbar/page.mdx:42`) is broader phrasing than the
   tested set, but the page immediately scopes it by enumeration to exactly the five popups
   behavior.md tests (`docs/src/app/(docs)/react/components/toolbar/page.mdx:59`), with Tooltip
   called out separately (`docs/src/app/(docs)/react/components/toolbar/page.mdx:61-63`). No conflict.
4. Zero prop prose: the page names no props in its text (API documentation is fully delegated to
   the generated `./types` renders, `docs/src/app/(docs)/react/components/toolbar/page.mdx:101-125`),
   so nothing on the page can contradict behavior.md § Public API surface's exercised props
   (`orientation`, `dir`, `loopFocus`, `disabled`, `nativeButton`, `focusableWhenDisabled`,
   `render`, `defaultValue`, `type`, `href`) or behavior.md's keyboard/focus/ARIA sections.

## Cross-links to other docs pages

N/A — the page contains no markdown links to other docs pages. Other components are referenced
only as plain code spans without links: `<AlertDialog>`, `<Dialog>`, `<Menu>`, `<Popover>`,
`<Select>` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:59`), `<Tooltip.Trigger>`
(`docs/src/app/(docs)/react/components/toolbar/page.mdx:63`), and `<NumberField.Input>` /
`<Toolbar.Input>` (`docs/src/app/(docs)/react/components/toolbar/page.mdx:82`). All module
imports are local (`./demos/hero` at `docs/src/app/(docs)/react/components/toolbar/page.mdx:9`
and `./types` at `docs/src/app/(docs)/react/components/toolbar/page.mdx:101`).
