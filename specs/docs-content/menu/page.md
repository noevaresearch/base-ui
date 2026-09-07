# Menu — docs-content page spec

Scope: this file mines the docs page only — `docs/src/app/(docs)/react/components/menu/page.mdx`
(page prose, embedded snippets, inline references). Demo sources under `demos/` and the
API-table definitions in `./types` are NOT opened here (demos are Stage 2's scope:
`specs/docs-content/menu/demos.json`). Behavior claims are cross-checked against
`specs/library/menu/behavior.md` and cited below by section name, not re-derived.

## Page structure (headings, in order)

- `# Menu` (page title) `docs/src/app/(docs)/react/components/menu/page.mdx:1`
- `<Subtitle>` — "A list of actions in a dropdown, enhanced with keyboard navigation." `docs/src/app/(docs)/react/components/menu/page.mdx:3`
- `<Meta>` SEO description (not a heading) `docs/src/app/(docs)/react/components/menu/page.mdx:4-7`
- Hero demo rendered immediately after the intro, before any heading (import + render point; demo source out of scope) `docs/src/app/(docs)/react/components/menu/page.mdx:9-11`
- `## Anatomy` `docs/src/app/(docs)/react/components/menu/page.mdx:13`
- `## Examples` `docs/src/app/(docs)/react/components/menu/page.mdx:57`
  - `### Open on hover` `docs/src/app/(docs)/react/components/menu/page.mdx:59`
  - `### Checkbox items` `docs/src/app/(docs)/react/components/menu/page.mdx:67`
  - `### Radio items` `docs/src/app/(docs)/react/components/menu/page.mdx:75`
  - `### Close on click` `docs/src/app/(docs)/react/components/menu/page.mdx:83`
  - `### Group labels` `docs/src/app/(docs)/react/components/menu/page.mdx:95`
  - `### Nested menu` `docs/src/app/(docs)/react/components/menu/page.mdx:103`
  - `### Navigate to another page` `docs/src/app/(docs)/react/components/menu/page.mdx:140`
  - `### Open a dialog` `docs/src/app/(docs)/react/components/menu/page.mdx:148`
  - `### Detached triggers` `docs/src/app/(docs)/react/components/menu/page.mdx:193`
  - `### Multiple triggers` `docs/src/app/(docs)/react/components/menu/page.mdx:232`
  - `### Controlled mode with multiple triggers` `docs/src/app/(docs)/react/components/menu/page.mdx:298`
  - `### Arrow` `docs/src/app/(docs)/react/components/menu/page.mdx:308`
  - `### Animating the Menu` `docs/src/app/(docs)/react/components/menu/page.mdx:316`
    - `#### Position and Size` `docs/src/app/(docs)/react/components/menu/page.mdx:321`
    - `#### Content` `docs/src/app/(docs)/react/components/menu/page.mdx:326`
- `## API reference` `docs/src/app/(docs)/react/components/menu/page.mdx:343`
  - `### Root` `docs/src/app/(docs)/react/components/menu/page.mdx:347`
  - `### Trigger` `docs/src/app/(docs)/react/components/menu/page.mdx:351`
  - `### Portal` `docs/src/app/(docs)/react/components/menu/page.mdx:355`
  - `### Backdrop` `docs/src/app/(docs)/react/components/menu/page.mdx:359`
  - `### Positioner` `docs/src/app/(docs)/react/components/menu/page.mdx:363`
  - `### Popup` `docs/src/app/(docs)/react/components/menu/page.mdx:367`
  - `### Viewport` `docs/src/app/(docs)/react/components/menu/page.mdx:371`
  - `### Arrow` `docs/src/app/(docs)/react/components/menu/page.mdx:377`
  - `### Item` `docs/src/app/(docs)/react/components/menu/page.mdx:381`
  - `### LinkItem` `docs/src/app/(docs)/react/components/menu/page.mdx:385`
  - `### SubmenuRoot` `docs/src/app/(docs)/react/components/menu/page.mdx:389`
  - `### SubmenuTrigger` `docs/src/app/(docs)/react/components/menu/page.mdx:393`
  - `### Group` `docs/src/app/(docs)/react/components/menu/page.mdx:397`
  - `### GroupLabel` `docs/src/app/(docs)/react/components/menu/page.mdx:401`
  - `### RadioGroup` `docs/src/app/(docs)/react/components/menu/page.mdx:405`
  - `### RadioItem` `docs/src/app/(docs)/react/components/menu/page.mdx:409`
  - `### RadioItemIndicator` `docs/src/app/(docs)/react/components/menu/page.mdx:413`
  - `### CheckboxItem` `docs/src/app/(docs)/react/components/menu/page.mdx:417`
  - `### CheckboxItemIndicator` `docs/src/app/(docs)/react/components/menu/page.mdx:421`
  - `### Separator` `docs/src/app/(docs)/react/components/menu/page.mdx:425`
- `## createHandle` `docs/src/app/(docs)/react/components/menu/page.mdx:429`
  - `### Handle` `docs/src/app/(docs)/react/components/menu/page.mdx:435` — placed after a `[//]: # '@exclude-table-of-contents'` comment, so it is excluded from the table of contents `docs/src/app/(docs)/react/components/menu/page.mdx:433`
- Trailing non-heading content: `export const metadata` SEO keyword list (13 keywords including "React Menu", "Dropdown Menu", "Keyboard Navigation Menu") `docs/src/app/(docs)/react/components/menu/page.mdx:439-455`

## Prose claims about component behavior

Intro:

- The component is described as "A high-quality, unstyled React menu component that displays list of actions in a dropdown, enhanced with keyboard navigation." `docs/src/app/(docs)/react/components/menu/page.mdx:4-7`
- Cross-check: consistent with `specs/library/menu/behavior.md` § Part index (headless/unstyled is the library premise; keyboard navigation is covered by the shared keyboard model). No mismatch.

Anatomy:

- "Import the component and assemble its parts" — the anatomy shows `Menu.Root` containing `Menu.Trigger`, and `Menu.Portal` containing `Menu.Backdrop` and `Menu.Positioner` containing `Menu.Popup`; inside the popup: `Menu.Arrow`, `Menu.Item`, `Menu.LinkItem`, `Menu.Separator`, `Menu.SubmenuRoot` (containing `Menu.SubmenuTrigger`), `Menu.Group` (containing `Menu.GroupLabel`), `Menu.RadioGroup` (containing `Menu.GroupLabel` and `Menu.RadioItem` with `Menu.RadioItemIndicator`), `Menu.CheckboxItem` (with `Menu.CheckboxItemIndicator`), and `Menu.Viewport` `docs/src/app/(docs)/react/components/menu/page.mdx:17-55`
- Cross-check: the shell `Root > Portal > Positioner > Popup` matches behavior.md § Cross-cutting → "Uniform DOM shell"; no mismatch.

Open on hover:

- Add the `openOnHover` prop to `<Menu.Trigger>` to make a menu that opens on hover. `docs/src/app/(docs)/react/components/menu/page.mdx:61`
- The `delay` prop additionally configures how quickly the menu opens on hover. `docs/src/app/(docs)/react/components/menu/page.mdx:61`
- Cross-check: consistent with behavior.md § Part index → `trigger` ("supports `openOnHover`/`delay`"). No mismatch.

Checkbox items:

- `<Menu.CheckboxItem>` creates a menu item that can toggle a setting on or off. `docs/src/app/(docs)/react/components/menu/page.mdx:69`
- Cross-check: consistent with behavior.md § Part index → `checkbox-item` (toggles on click/Enter/Space, `aria-checked`/`data-checked`). No mismatch.

Radio items:

- `<Menu.RadioGroup>` and `<Menu.RadioItem>` create menu items that work like radio buttons. `docs/src/app/(docs)/react/components/menu/page.mdx:77`
- Cross-check: consistent with behavior.md § Part index → `radio-group-item` (role `menuitemradio`, `value`/`onValueChange`). No mismatch.

Close on click:

- The `closeOnClick` prop changes whether the menu closes when an item is clicked. `docs/src/app/(docs)/react/components/menu/page.mdx:85`
- The snippet states `<Menu.CheckboxItem closeOnClick />` closes the menu when a checkbox item is clicked, and `<Menu.Item closeOnClick={false} />` keeps the menu open when an item is clicked. `docs/src/app/(docs)/react/components/menu/page.mdx:87-93`
- Cross-check: consistent with behavior.md § Part index → `checkbox-item` (does not close by default, `closeOnClick` opt-in) and `group-item-link` (`Menu.Item` closes the menu on click by default). No mismatch.

Group labels:

- `<Menu.GroupLabel>` adds a label to a `<Menu.Group>` or `<Menu.RadioGroup>`. `docs/src/app/(docs)/react/components/menu/page.mdx:97`
- Cross-check: consistent with behavior.md § Part index → `group-item-link` (GroupLabel registers its `id` with the group's `aria-labelledby`). No mismatch.

Nested menu:

- To create a submenu, nest another menu inside the parent menu with `<Menu.SubmenuRoot>`; `<Menu.SubmenuTrigger>` is the menu item that opens the nested menu. `docs/src/app/(docs)/react/components/menu/page.mdx:105`
- Cross-check: consistent with behavior.md § Part index → `submenu-trigger` and the per-level portal shell in § Cross-cutting → "Uniform DOM shell". No mismatch.

Navigate to another page:

- `<Menu.LinkItem>` creates a link; the snippet shows `<Menu.LinkItem href="/projects">Go to Projects</Menu.LinkItem>`. `docs/src/app/(docs)/react/components/menu/page.mdx:142`, `docs/src/app/(docs)/react/components/menu/page.mdx:144-146`
- Cross-check: consistent with behavior.md § Part index → `group-item-link` (role `menuitem` on an anchor, Enter/Space trigger router navigation). No mismatch.

Open a dialog:

- To open a dialog from a menu: control the dialog state and open it imperatively using the `onClick` handler on the menu item. `docs/src/app/(docs)/react/components/menu/page.mdx:150`
- The snippet wires `<Menu.Item onClick={() => setDialogOpen(true)}>` to a controlled `<Dialog.Root open={dialogOpen} onOpenChange={setDialogOpen}>`. `docs/src/app/(docs)/react/components/menu/page.mdx:152-191`
- Cross-check: behavior.md § Part index → `group-item-link` notes `Menu.Item` closes the menu on click by default; the page does not mention or override that close here (the menu closing as the dialog opens is the implied behavior). Not contradicted; no mismatch.

Detached triggers:

- A menu can be opened by a trigger that lives either inside or outside the `<Menu.Root>`. `docs/src/app/(docs)/react/components/menu/page.mdx:195`
- Usage guidance: keep the trigger inside `<Menu.Root>` for simple, tightly coupled layouts (like the hero demo at the top of the page); when trigger and content must live in different parts of the tree (e.g. a card list controlling a menu rendered near the document root), create a handle with `Menu.createHandle()` and pass it to both the trigger and the root. `docs/src/app/(docs)/react/components/menu/page.mdx:196-197`
- Only top-level menus can have detached triggers; submenus must have their triggers defined within the `SubmenuRoot` part. `docs/src/app/(docs)/react/components/menu/page.mdx:199-200`
- Imperative methods on the handle, such as `open()` and `close()`, require a `<Menu.Root>` using the same handle to be mounted. `docs/src/app/(docs)/react/components/menu/page.mdx:202`
- Calls made while no root is attached to the handle — before one mounts, or after it unmounts — are ignored. Each root mount starts from fresh state: a call made while no root was attached is not replayed, and no open state carries over from a previous mount. `docs/src/app/(docs)/react/components/menu/page.mdx:203`
- Cross-check: consistent with behavior.md § Part index → `root` (`Menu.createHandle()` detached-trigger imperative API with multi-root handoff) and → `trigger` ("works as a handle-backed detached trigger"; trigger throws without a Root ancestor or handle, which is why detached triggers need a handle). The top-level-only restriction is not stated verbatim in the behavior index but is implied by → `submenu-trigger` (SubmenuTrigger throws outside `SubmenuRoot`). No mismatch.

Multiple triggers:

- One menu can be opened by several triggers, either by rendering multiple `<Menu.Trigger>` components inside the same `<Menu.Root>`, or by attaching several detached triggers to the same handle. `docs/src/app/(docs)/react/components/menu/page.mdx:234-235`
- Menus can render different content depending on which trigger opened them: pass a `payload` prop to each `<Menu.Trigger>` and read it via a function child on `<Menu.Root>`. `docs/src/app/(docs)/react/components/menu/page.mdx:256-257`
- Provide a type argument to `createHandle()` to strongly type the payload. `docs/src/app/(docs)/react/components/menu/page.mdx:258`
- Cross-check: consistent with behavior.md § Part index → `root` ("multi-trigger `payload` dispatch"). No mismatch.

Controlled mode with multiple triggers:

- Control a menu's open state externally with the `open` and `onOpenChange` props on `<Menu.Root>`. `docs/src/app/(docs)/react/components/menu/page.mdx:300`
- When more than one trigger can open the menu, track the active trigger with the `triggerId` prop on `<Menu.Root>` and matching `id` props on each `<Menu.Trigger>`. `docs/src/app/(docs)/react/components/menu/page.mdx:301`
- The `onOpenChange` callback receives `eventDetails`, which includes the DOM element that initiated the change, so you can update your `triggerId` state when the user activates a different trigger. `docs/src/app/(docs)/react/components/menu/page.mdx:302`
- Cross-check: mostly consistent with behavior.md § Part index → `root` (controlled `open`, `onOpenChange` with a details object carrying `trigger`). The behavior index names the fields `reason`/`event`/`trigger` and never uses the label `eventDetails` — flagged under Discrepancies as a terminology note.

Arrow:

- `<Menu.Arrow>` placed inside the popup visually connects the menu to its trigger. `docs/src/app/(docs)/react/components/menu/page.mdx:310`
- Cross-check: consistent with behavior.md § Part index → `arrow-backdrop-portal-viewport` (Arrow nested under Popup > Positioner > Portal). No mismatch.

Animating the Menu:

- When one menu is opened by multiple detached triggers, you can animate the menu as it moves between triggers, including position, size, and content. `docs/src/app/(docs)/react/components/menu/page.mdx:318-319`
- Position: apply CSS transitions to the `left`, `right`, `top`, and `bottom` properties of the Positioner part. `docs/src/app/(docs)/react/components/menu/page.mdx:323`
- Size: transition the `width` and `height` of the Popup part. `docs/src/app/(docs)/react/components/menu/page.mdx:324`
- The menu also supports content transitions, useful when different triggers display different content within the same menu. `docs/src/app/(docs)/react/components/menu/page.mdx:328-329`
- Content animations are enabled by wrapping the menu content in the `<Menu.Viewport>` part. `docs/src/app/(docs)/react/components/menu/page.mdx:331`
- The Viewport renders a `div` with `data-activation-direction`, containing up to two space-separated tokens — a horizontal (`left` or `right`) and a vertical (`up` or `down`) value — for direction-aware animations. `docs/src/app/(docs)/react/components/menu/page.mdx:332`
- Inside `<Menu.Viewport>`, content is wrapped in divs with transition data attributes: `data-current` (the currently visible content when no transitions are present, or the incoming content) and `data-previous` (the outgoing content during a transition). `docs/src/app/(docs)/react/components/menu/page.mdx:334-337`
- Cross-check: token names match behavior.md § Part index → `arrow-backdrop-portal-viewport`, but the behavior spec gates `data-previous` (and the morph transition) to Chromium-only transitions and marks it `inert`; the page states the attributes unconditionally — flagged under Discrepancies.

Viewport usage note (prose under the API reference):

- The Viewport is optional — reach for it only when a single popup is opened by multiple triggers, its content differs per trigger, and the switch between them is animated. `docs/src/app/(docs)/react/components/menu/page.mdx:375`
- When used, set `width: var(--positioner-width)` and `height: var(--positioner-height)` on the Positioner so its box is frozen to the measured size during the transition; otherwise content-driven resizing can make the popup thrash or flip to another side. `docs/src/app/(docs)/react/components/menu/page.mdx:375`
- Cross-check: compatible with behavior.md § Part index → `positioner` (positions with inline `transform` unless a `Menu.Viewport` is present, then top/left). The CSS-var freeze guidance is page-only detail with no contradicting statement in the behavior spec. No mismatch.

## API tables referenced (props/parts documented on this page)

- All prop tables are rendered dynamically by `<TypesMenu.* />` components imported from `./types` `docs/src/app/(docs)/react/components/menu/page.mdx:345` — the `.mdx` contains no static prop rows, so individual prop signatures are out of scope for this page-only pass (they live in the types file, not this page).
- Parts with an API-reference section (heading + table component), in page order:
  - Root `docs/src/app/(docs)/react/components/menu/page.mdx:347-349`
  - Trigger `docs/src/app/(docs)/react/components/menu/page.mdx:351-353`
  - Portal `docs/src/app/(docs)/react/components/menu/page.mdx:355-357`
  - Backdrop `docs/src/app/(docs)/react/components/menu/page.mdx:359-361`
  - Positioner `docs/src/app/(docs)/react/components/menu/page.mdx:363-365`
  - Popup `docs/src/app/(docs)/react/components/menu/page.mdx:367-369`
  - Viewport `docs/src/app/(docs)/react/components/menu/page.mdx:371-373` (followed by the usage-note prose cited above `docs/src/app/(docs)/react/components/menu/page.mdx:375`)
  - Arrow `docs/src/app/(docs)/react/components/menu/page.mdx:377-379`
  - Item `docs/src/app/(docs)/react/components/menu/page.mdx:381-383`
  - LinkItem `docs/src/app/(docs)/react/components/menu/page.mdx:385-387`
  - SubmenuRoot `docs/src/app/(docs)/react/components/menu/page.mdx:389-391`
  - SubmenuTrigger `docs/src/app/(docs)/react/components/menu/page.mdx:393-395`
  - Group `docs/src/app/(docs)/react/components/menu/page.mdx:397-399`
  - GroupLabel `docs/src/app/(docs)/react/components/menu/page.mdx:401-403`
  - RadioGroup `docs/src/app/(docs)/react/components/menu/page.mdx:405-407`
  - RadioItem `docs/src/app/(docs)/react/components/menu/page.mdx:409-411`
  - RadioItemIndicator `docs/src/app/(docs)/react/components/menu/page.mdx:413-415`
  - CheckboxItem `docs/src/app/(docs)/react/components/menu/page.mdx:417-419`
  - CheckboxItemIndicator `docs/src/app/(docs)/react/components/menu/page.mdx:421-423`
  - Separator `docs/src/app/(docs)/react/components/menu/page.mdx:425-427`
  - createHandle (`##`-level section) `docs/src/app/(docs)/react/components/menu/page.mdx:429-431`
  - Handle (`###`-level section, excluded from table of contents) `docs/src/app/(docs)/react/components/menu/page.mdx:435-437`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- `title="Anatomy"` (jsx) — full part assembly tree `docs/src/app/(docs)/react/components/menu/page.mdx:17-55`
- `title="Control whether the menu closes on click"` (jsx) — `closeOnClick` on CheckboxItem vs `closeOnClick={false}` on Item `docs/src/app/(docs)/react/components/menu/page.mdx:87-93`
- `title="Adding a submenu"` (jsx) — `SubmenuRoot` + `SubmenuTrigger` nesting with its own `Portal`/`Positioner`/`Popup`; uses `@highlight-start`/`@highlight`/`@highlight-end` and `prettier-ignore` markers `docs/src/app/(docs)/react/components/menu/page.mdx:107-134`
- `title="A menu item that opens a link"` (jsx) — `Menu.LinkItem` with `href` `docs/src/app/(docs)/react/components/menu/page.mdx:144-146`
- `title="Connecting a dialog to a menu"` (tsx) — controlled Dialog opened imperatively from `Menu.Item` `onClick`; imports `React`, `Dialog`, and `Menu` `docs/src/app/(docs)/react/components/menu/page.mdx:152-191`
- `title="Detached triggers"` (jsx) — `Menu.createHandle()` shared between `Menu.Trigger` and `Menu.Root` `docs/src/app/(docs)/react/components/menu/page.mdx:205-226`
- `title="Multiple triggers within the Root part"` (jsx) — two `Menu.Trigger`s in one `Menu.Root` `docs/src/app/(docs)/react/components/menu/page.mdx:237-243`
- `title="Multiple detached triggers"` (jsx) — two triggers attached to the same handle `docs/src/app/(docs)/react/components/menu/page.mdx:245-254`
- `title="Detached triggers with payload"` (jsx) — typed `createHandle<{ items: string[] }>()`, `payload` per trigger, function child on `Menu.Root` reading `payload`, rendering items through `Menu.Viewport` `docs/src/app/(docs)/react/components/menu/page.mdx:260-296`

Demo render points on the page (imports + JSX usage only; demo sources are NOT opened here — Stage 2's scope): hero `docs/src/app/(docs)/react/components/menu/page.mdx:9-11`, open-on-hover `docs/src/app/(docs)/react/components/menu/page.mdx:63-65`, checkbox-items `docs/src/app/(docs)/react/components/menu/page.mdx:71-73`, radio-items `docs/src/app/(docs)/react/components/menu/page.mdx:79-81`, group-labels `docs/src/app/(docs)/react/components/menu/page.mdx:99-101`, submenu `docs/src/app/(docs)/react/components/menu/page.mdx:136-138`, detached-triggers-simple `docs/src/app/(docs)/react/components/menu/page.mdx:228-230`, detached-triggers-controlled `docs/src/app/(docs)/react/components/menu/page.mdx:304-306`, arrow `docs/src/app/(docs)/react/components/menu/page.mdx:312-314`, detached-triggers-full `docs/src/app/(docs)/react/components/menu/page.mdx:339-341`.

## Discrepancies (docs page vs. behavior.md)

- Viewport transition attributes are stated unconditionally on the page: `data-current` is described as "the currently visible content when no transitions are present or the incoming content" and `data-previous` as "the outgoing content during a transition" `docs/src/app/(docs)/react/components/menu/page.mdx:334-337`. behavior.md § Part index → `arrow-backdrop-portal-viewport` says the Viewport's internal `data-current` container is remounted whenever the active trigger/payload changes, and the `inert` `data-previous` container is rendered only during the Chromium-only morph transitions. Mismatch nuance: a reader could expect `data-previous` to always exist during any CSS transition and to be interactive-safe, when per the behavior spec it appears only in the Chromium-only morph path and is `inert`.
- Same gating nuance for `data-activation-direction`: the page says the Viewport "renders a `div` with `data-activation-direction`" as a general statement `docs/src/app/(docs)/react/components/menu/page.mdx:332`, while behavior.md § Part index → `arrow-backdrop-portal-viewport` ties the token to the Chromium-only morph-transition path. The token composition detail on the page (up to two space-separated tokens, horizontal `left`/`right` + vertical `up`/`down`) is not contradicted, only the unconditional availability.
- `eventDetails` naming: the page says the `onOpenChange` callback "receives `eventDetails`, which includes the DOM element that initiated the change" `docs/src/app/(docs)/react/components/menu/page.mdx:302`; behavior.md § Part index → `root` describes `onOpenChange(nextOpen, { reason, event, trigger })` and never uses the label `eventDetails`. Substantively compatible (the details object carries the trigger/event), but the terminology differs — reconciled here as a naming note, not a behavioral contradiction.
- Detached-handle call semantics: the page states handle calls while no root is attached are ignored and every mount starts from fresh state (no replay, no carried-over open state) `docs/src/app/(docs)/react/components/menu/page.mdx:203`; behavior.md § Part index → `root` mentions the `createHandle()` API and multi-root handoff but its index does not state the unmounted-call/fresh-state semantics. No contradiction found; the docs are strictly more specific here.
- Top-level-only detached triggers: the page restricts detached triggers to top-level menus `docs/src/app/(docs)/react/components/menu/page.mdx:199-200`; behavior.md § Part index → `submenu-trigger` (SubmenuTrigger throws outside `SubmenuRoot`) implies the same constraint without stating it for detached handles. Consistent; no mismatch.

All other prose claims cross-check clean against `specs/library/menu/behavior.md` (see per-claim "Cross-check" notes under "Prose claims about component behavior"): openOnHover/delay (§ Part index → `trigger`), checkbox/radio semantics (→ `checkbox-item`, `radio-group-item`), closeOnClick (→ `checkbox-item`, `group-item-link`), GroupLabel (→ `group-item-link`), submenu parts (→ `submenu-trigger`), LinkItem (→ `group-item-link`), Arrow (→ `arrow-backdrop-portal-viewport`), controlled open/onOpenChange and payload dispatch (→ `root`), and the anatomy shell (§ Cross-cutting → "Uniform DOM shell").

## Cross-links to other docs pages

- N/A — the page contains no markdown links to other docs pages.
- Related-component reference (code-level, not a docs-page link): the "Connecting a dialog to a menu" snippet imports `Dialog` from `@base-ui/react/dialog` `docs/src/app/(docs)/react/components/menu/page.mdx:154`.
- Intra-page imports of sibling modules (not docs-page cross-links): demo modules under `./demos/` (render points listed above under "Code snippets") and the API-table module `./types` `docs/src/app/(docs)/react/components/menu/page.mdx:345`.
