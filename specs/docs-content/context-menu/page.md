# Context Menu docs-content page spec

Mined from the docs page only: `docs/src/app/(docs)/react/components/context-menu/page.mdx` (172 lines).
Component behavior itself is covered by `specs/library/context-menu/behavior.md` and is referenced
below by section name (§ ...) rather than restated. Demo source files (`./demos/*`) are intentionally
not opened here — Stage 2 (docs) mines them into `specs/docs-content/context-menu/demos.json`.

## Page structure (headings, in order)

- `# Context Menu` — `docs/src/app/(docs)/react/components/context-menu/page.mdx:1`
- Subtitle: "A menu that appears at the pointer on right click or long press." — `docs/src/app/(docs)/react/components/context-menu/page.mdx:3`
- `<Meta name="description">` block — `docs/src/app/(docs)/react/components/context-menu/page.mdx:4-7`
- Hero demo: imports `DemoContextMenuHero` from `./demos/hero` and renders it — `docs/src/app/(docs)/react/components/context-menu/page.mdx:9-11`
- `## Usage guidelines` — `docs/src/app/(docs)/react/components/context-menu/page.mdx:13`
- `## Anatomy` — `docs/src/app/(docs)/react/components/context-menu/page.mdx:17` (fenced "Anatomy" snippet at `docs/src/app/(docs)/react/components/context-menu/page.mdx:21-56`)
- `## Examples` — `docs/src/app/(docs)/react/components/context-menu/page.mdx:58`
  - `### Using with Menu` — `docs/src/app/(docs)/react/components/context-menu/page.mdx:62` (renders `DemoContextMenuWithMenu` from `./demos/with-menu`, `docs/src/app/(docs)/react/components/context-menu/page.mdx:66-68`)
  - `### Nested menu` — `docs/src/app/(docs)/react/components/context-menu/page.mdx:70` (renders `DemoContextMenuSubmenu` from `./demos/submenu`, `docs/src/app/(docs)/react/components/context-menu/page.mdx:74-76`)
- `## API reference` — `docs/src/app/(docs)/react/components/context-menu/page.mdx:78`
  - Nineteen `###` part sections, in order: Root (`docs/src/app/(docs)/react/components/context-menu/page.mdx:82`), Trigger (`docs/src/app/(docs)/react/components/context-menu/page.mdx:86`), Portal (`docs/src/app/(docs)/react/components/context-menu/page.mdx:90`), Backdrop (`docs/src/app/(docs)/react/components/context-menu/page.mdx:94`), Positioner (`docs/src/app/(docs)/react/components/context-menu/page.mdx:98`), Popup (`docs/src/app/(docs)/react/components/context-menu/page.mdx:102`), Arrow (`docs/src/app/(docs)/react/components/context-menu/page.mdx:106`), Item (`docs/src/app/(docs)/react/components/context-menu/page.mdx:110`), LinkItem (`docs/src/app/(docs)/react/components/context-menu/page.mdx:114`), SubmenuRoot (`docs/src/app/(docs)/react/components/context-menu/page.mdx:118`), SubmenuTrigger (`docs/src/app/(docs)/react/components/context-menu/page.mdx:122`), Group (`docs/src/app/(docs)/react/components/context-menu/page.mdx:126`), GroupLabel (`docs/src/app/(docs)/react/components/context-menu/page.mdx:130`), RadioGroup (`docs/src/app/(docs)/react/components/context-menu/page.mdx:134`), RadioItem (`docs/src/app/(docs)/react/components/context-menu/page.mdx:138`), RadioItemIndicator (`docs/src/app/(docs)/react/components/context-menu/page.mdx:142`), CheckboxItem (`docs/src/app/(docs)/react/components/context-menu/page.mdx:146`), CheckboxItemIndicator (`docs/src/app/(docs)/react/components/context-menu/page.mdx:150`), Separator (`docs/src/app/(docs)/react/components/context-menu/page.mdx:154`)
- `export const metadata` keywords block (SEO keywords, e.g. 'React Context Menu', 'Long Press Menu') — `docs/src/app/(docs)/react/components/context-menu/page.mdx:158-172`

## Prose claims about component behavior

- The page defines the component as "A menu that appears at the pointer on right click or long press." — `docs/src/app/(docs)/react/components/context-menu/page.mdx:3`
  Cross-check: matches behavior.md § Events (right-click `contextmenu` opens; touch long-press opens after a 500ms hold).
- The meta description calls it "A high-quality, unstyled React context menu component that appears at the pointer on right click or long press." — `docs/src/app/(docs)/react/components/context-menu/page.mdx:4-7`
  Cross-check: the open gestures match behavior.md § Events; the "unstyled" characterization has no counterpart in behavior.md (uncovered, not contradicted — behavior.md mines tests only).
- Usage guideline, "Use context menus as an enhancement": the context menu must not be the only way to perform its actions, because users "may not discover or be able to open a context menu, especially on touch devices or with assistive technology," and visible controls for those actions must "Always" be provided. — `docs/src/app/(docs)/react/components/context-menu/page.mdx:15`
  Cross-check: advisory guidance; behavior.md § Events confirms touch long-press support exists, so the touch caveat is a discoverability concern, not a capability claim. No behavior.md section covers usage guidance; no mismatch.
- Anatomy instruction: "Import the components and place them together," with a single namespace import from `@base-ui/react/context-menu`. — `docs/src/app/(docs)/react/components/context-menu/page.mdx:19-22`
  Cross-check: consistent with behavior.md § Public API surface (parts imported from the `@base-ui/react/context-menu` namespace).
- Anatomy composition (implicit claim): Root wraps Trigger and Portal; Portal wraps Backdrop and Positioner; Positioner wraps Popup; Popup contains Arrow, Item, LinkItem, Separator, a SubmenuRoot > SubmenuTrigger pair, a Group > GroupLabel pair, a RadioGroup > RadioItem > RadioItemIndicator chain, and a CheckboxItem > CheckboxItemIndicator pair. — `docs/src/app/(docs)/react/components/context-menu/page.mdx:24-55`
  Cross-check: consistent with behavior.md § DOM structure & portal behavior (canonical `Root > (Trigger + Portal > Positioner > Popup > items)`; a user-rendered Backdrop inside the portal is a sibling of the positioner). Parts beyond Root/Trigger/Portal/Backdrop/Positioner/Popup/Item/SubmenuRoot/SubmenuTrigger are not exercised by the mined tests (behavior.md § Public API surface) — see Discrepancies.
- Examples pointer: "[Menu](/react/components/menu#examples) displays additional demos, many of which apply to the context menu as well." — `docs/src/app/(docs)/react/components/context-menu/page.mdx:60`
- "Using with Menu" rationale: "A context menu should supplement a primary way to perform the same actions"; the example's image card exposes actions through a visible menu button and reuses them "in the context menu for right-click and long-press users." — `docs/src/app/(docs)/react/components/context-menu/page.mdx:64`
  Cross-check: consistent with behavior.md § Events (both open gestures) and restates the § Usage-guideline theme from `docs/src/app/(docs)/react/components/context-menu/page.mdx:15`.
- Nested menu guidance: "To create a submenu, create a `<ContextMenu.SubmenuRoot>` inside the parent context menu. Use the `<ContextMenu.SubmenuTrigger>` part for the menu item that opens the nested menu." — `docs/src/app/(docs)/react/components/context-menu/page.mdx:72`
  Cross-check: consistent with behavior.md § DOM structure & portal behavior (submenus nest their own Portal > Positioner > Popup chain inside the parent popup) and behavior.md § Public API surface (SubmenuRoot and SubmenuTrigger parts with their props).

## API tables referenced (props/parts documented on this page)

The page documents 19 parts, each as a `###` heading followed by a table rendered by a
`TypesContextMenu.<Part>` component (imported from `./types`, `docs/src/app/(docs)/react/components/context-menu/page.mdx:80`).
The .mdx itself contains no inline prop tables — all prop documentation is delegated to those
generated components, so no page-stated prop text exists to cross-check against behavior.md
§ Public API surface.

- Root — heading `docs/src/app/(docs)/react/components/context-menu/page.mdx:82`, table `docs/src/app/(docs)/react/components/context-menu/page.mdx:84`
- Trigger — `docs/src/app/(docs)/react/components/context-menu/page.mdx:86-88`
- Portal — `docs/src/app/(docs)/react/components/context-menu/page.mdx:90-92`
- Backdrop — `docs/src/app/(docs)/react/components/context-menu/page.mdx:94-96`
- Positioner — `docs/src/app/(docs)/react/components/context-menu/page.mdx:98-100`
- Popup — `docs/src/app/(docs)/react/components/context-menu/page.mdx:102-104`
- Arrow — `docs/src/app/(docs)/react/components/context-menu/page.mdx:106-108`
- Item — `docs/src/app/(docs)/react/components/context-menu/page.mdx:110-112`
- LinkItem — `docs/src/app/(docs)/react/components/context-menu/page.mdx:114-116`
- SubmenuRoot — `docs/src/app/(docs)/react/components/context-menu/page.mdx:118-120`
- SubmenuTrigger — `docs/src/app/(docs)/react/components/context-menu/page.mdx:122-124`
- Group — `docs/src/app/(docs)/react/components/context-menu/page.mdx:126-128`
- GroupLabel — `docs/src/app/(docs)/react/components/context-menu/page.mdx:130-132`
- RadioGroup — `docs/src/app/(docs)/react/components/context-menu/page.mdx:134-136`
- RadioItem — `docs/src/app/(docs)/react/components/context-menu/page.mdx:138-140`
- RadioItemIndicator — `docs/src/app/(docs)/react/components/context-menu/page.mdx:142-144`
- CheckboxItem — `docs/src/app/(docs)/react/components/context-menu/page.mdx:146-148`
- CheckboxItemIndicator — `docs/src/app/(docs)/react/components/context-menu/page.mdx:150-152`
- Separator — `docs/src/app/(docs)/react/components/context-menu/page.mdx:154-156`

Coverage note: of these 19, the parts exercised by the mined tests (behavior.md § Public API
surface) are Root, Trigger, Portal, Backdrop, Positioner, Popup, Item, SubmenuRoot, SubmenuTrigger.
Arrow, LinkItem, Group, GroupLabel, RadioGroup, RadioItem, RadioItemIndicator, CheckboxItem,
CheckboxItemIndicator, and Separator are documented here but untested (see Discrepancies).

## Code snippets embedded directly in the .mdx (not pulled from demos/)

Exactly one fenced snippet, titled `Anatomy` (```` ```jsx title="Anatomy" ````, `docs/src/app/(docs)/react/components/context-menu/page.mdx:21`):

- Single namespace import: `import { ContextMenu } from '@base-ui/react/context-menu';` — `docs/src/app/(docs)/react/components/context-menu/page.mdx:22`
- Full composition skeleton: Root > Trigger alongside Portal > Backdrop + Positioner > Popup containing Arrow, Item, LinkItem, Separator, SubmenuRoot > SubmenuTrigger, Group > GroupLabel, RadioGroup > RadioItem > RadioItemIndicator, and CheckboxItem > CheckboxItemIndicator — `docs/src/app/(docs)/react/components/context-menu/page.mdx:24-55`
- Factual quirk: the snippet's closing tag carries a trailing semicolon, `</ContextMenu.Root>;` — `docs/src/app/(docs)/react/components/context-menu/page.mdx:55`

Non-snippet MDX constructs embedded in the page (not component documentation): the three demo
imports/render calls (`docs/src/app/(docs)/react/components/context-menu/page.mdx:9-11`,
`docs/src/app/(docs)/react/components/context-menu/page.mdx:66-68`,
`docs/src/app/(docs)/react/components/context-menu/page.mdx:74-76` — demo internals are Stage 2
scope), the `TypesContextMenu` import (`docs/src/app/(docs)/react/components/context-menu/page.mdx:80`),
and the `export const metadata` keywords block (`docs/src/app/(docs)/react/components/context-menu/page.mdx:158-172`).

## Discrepancies (docs page vs. behavior.md)

No contradictions found. Cross-check notes:

1. Open gestures: the subtitle, meta description, and "Using with Menu" example all describe opening
   via "right click or long press" (`docs/src/app/(docs)/react/components/context-menu/page.mdx:3`,
   `docs/src/app/(docs)/react/components/context-menu/page.mdx:6`,
   `docs/src/app/(docs)/react/components/context-menu/page.mdx:64`); behavior.md § Events documents
   both gestures. Consistent.
2. Composition: the page's Anatomy (`docs/src/app/(docs)/react/components/context-menu/page.mdx:24-55`)
   is a strict superset of behavior.md § DOM structure & portal behavior's canonical composition
   (`Root > (Trigger + Portal > Positioner > Popup > items)`); the extra parts conflict with no
   tested claim.
3. Coverage gaps (not mismatches): ten parts documented on the page (Arrow, LinkItem, Group,
   GroupLabel, RadioGroup, RadioItem, RadioItemIndicator, CheckboxItem, CheckboxItemIndicator,
   Separator — `docs/src/app/(docs)/react/components/context-menu/page.mdx:30-51`) are not exercised
   by the mined tests; behavior.md § Public API surface explicitly states no props beyond its listed
   set are asserted. Likewise the page's "unstyled" descriptor
   (`docs/src/app/(docs)/react/components/context-menu/page.mdx:6`) has no styling counterpart in
   behavior.md.
4. Prop documentation: the page states zero props in prose (all delegated to `TypesContextMenu`
   tables, `docs/src/app/(docs)/react/components/context-menu/page.mdx:80-156`), so nothing on the
   page can contradict behavior.md § Public API surface's asserted prop list (`open`,
   `defaultOpen`, `onOpenChange`, `disabled`, `container`, `anchor`, `collisionAvoidance`,
   `alignOffset`, `delay`, `openOnHover`, `onContextMenu`).

## Cross-links to other docs pages

- One outbound link: `[Menu](/react/components/menu#examples)`, with the note that Menu "displays
  additional demos, many of which apply to the context menu as well." —
  `docs/src/app/(docs)/react/components/context-menu/page.mdx:60`
  (The three `./demos/*` imports are local module imports, not docs-page links.)
