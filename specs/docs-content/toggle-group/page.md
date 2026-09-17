# toggle-group — docs page spec

Mined from the docs page file only:

- `docs/src/app/(docs)/react/components/toggle-group/page.mdx` (52 lines)

Cross-checked against `specs/library/toggle-group/behavior.md` (the component's own behavior spec, mined from its tests). That spec is cited by section name; its claims are NOT re-derived here.

## Page structure (headings, in order)

- H1 `# Toggle Group` — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:1`
- `<Subtitle>` one-liner (not a heading, but part of the page header block): "Provides a shared state to a series of toggle buttons." — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:3`
- `<Meta name="description">` for the page: "A high-quality, unstyled React toggle group component that provides shared state to a series of toggle buttons." — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:4-7`
- Hero demo render: imports `DemoToggleGroupHero` from `./demos/hero` and renders it immediately after the header block — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:9-11`
- H2 `## Anatomy` — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:13`
- H2 `## Examples` — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:23`
  - H3 `### Multiple` — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:25`
- H2 `## API reference` — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:33`
- `export const metadata` block with SEO keywords (React Toggle Group, Toggle Button Group, Segmented Control, Button Set, Radio Group Alternative, Exclusive Selection Toggle, Multi Select Toggle Buttons, Accessible Toggle Group, Headless React Components, Base UI) — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:39-52`

Additional imports rendered inline mid-page (demo source files themselves are out of scope here — Stage 2):

- `import { DemoToggleGroupMultiple } from './demos/multiple'` + render — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:29-31`
- `import { TypesToggleGroup } from './types'` + render — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:35-37`

## Prose claims about component behavior

Only claims made by the page text itself, each cross-checked against the behavior spec (`specs/library/toggle-group/behavior.md`). The page is very short; there are exactly four prose claims:

- "Provides a shared state to a series of toggle buttons." (Subtitle) — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:3`. Consistent with the behavior spec's "State model" section (one shared value array drives every child's pressed state; items accumulate under `multiple`, otherwise single selection) and its "Public API surface" section (children are `Toggle` components from the separate `toggle` unit, identified by their `value` prop).
- Meta description repeats the shared-state framing and adds "high-quality, unstyled React toggle group component" — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:6`. Marketing framing; no behavioral claim to check beyond shared state.
- Anatomy intro: "Import the component and use it as a single part:" — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:15`. Consistent with the behavior spec's "Public API surface" section (the group renders a single `div` root and exports no subcomponents of its own — that no-subcomponents point is marked UNVERIFIED in the spec — while the `Toggle` children come from the separate `toggle` unit).
- "Add the `multiple` prop to allow pressing more than one toggle at a time." — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:27`. Consistent with the behavior spec's "State model" section: `multiple={true}` lets items accumulate (pressing a second item keeps the first pressed: `packages/react/src/toggle-group/ToggleGroup.test.tsx:244-261`), whereas with `multiple` false (the default) only one item stays pressed (`packages/react/src/toggle-group/ToggleGroup.test.tsx:263-280`).

## API tables referenced (props/parts documented on this page)

- The page contains NO literal Markdown API tables. The `## API reference` section consists solely of a rendered React component: `import { TypesToggleGroup } from './types'` + `<TypesToggleGroup />` — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:33-37`.
- Props/parts named in the page's prose and inline snippets (no tables, just usage): `multiple` (prop on the group) — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:27`; the `ToggleGroup` part imported from `@base-ui/react/toggle-group` — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:18`.
- N/A otherwise: no props table, no parts table, no return-value table exists in the .mdx itself.

## Code snippets embedded directly in the .mdx (not pulled from demos/)

Exactly one fenced snippet is inline in the page file; the two `<Demo... />` renders and `<TypesToggleGroup />` are component imports, not snippets, and their sources are Stage 2's scope.

- **"Anatomy"** (jsx, `title="Anatomy"`): imports `ToggleGroup` from `@base-ui/react/toggle-group` and renders a bare `<ToggleGroup />` with no children — `docs/src/app/(docs)/react/components/toggle-group/page.mdx:17-21`. The bare render matches the "single part" framing of the intro line; composing the group with `Toggle` children is not shown in any inline snippet (that lives in demo sources, Stage 2 scope).

## Discrepancies (docs page vs. behavior.md)

No hard contradictions found. Scope notes:

- **Unproven-by-this-spec claims the page makes**: none — the page makes no claim the behavior spec fails to cover or contradicts.
- **Silent omissions** (docs page says nothing; behavior spec covers it — informational only, not errors): controlled `value` / uncontrolled `defaultValue`, `onValueChange` including `eventDetails.cancel()` cancellation semantics, group-level `disabled`, `orientation` plus the entire roving-tabindex keyboard model (arrow wrap-around, `Home`/`End`, `Enter`/`Space` activation, `DirectionProvider` RTL mapping), `role="group"` / `aria-label` naming, `aria-pressed`/`aria-disabled` exposure, data attributes (`data-pressed`, `data-disabled`, `data-orientation`, `data-multiple`), dev warning for value-less `Toggle` children, and `Toolbar` nesting — all in the behavior spec's "Public API surface", "State model", "Keyboard interactions", "Focus management", "Accessibility", "DOM structure & portal behavior", and "Events" sections; none are claimed on the page, so nothing to contradict.

## Cross-links to other docs pages

N/A — the page body contains no markdown links at all (no `[...](/react/...)` page links and no same-page anchor links). The only non-body references are the relative imports (`./demos/hero`, `./demos/multiple`, `./types`) listed under Page structure, and the `@base-ui/react/toggle-group` package specifier inside the Anatomy snippet (`docs/src/app/(docs)/react/components/toggle-group/page.mdx:18`), which is a package import, not a docs link.

## Snippet & behaviour contract

Per `specs/docs-content/CONTRACT.md` requirement 5, this table is part of this page's done. Authored
2026-09-17 by the `docs-content: components/toggle-group` iteration, which is the pick the contract's
own exception covers ("or the page's own item"); the spec carried no such section when the page was
mirrored, which is why `node ralph/scripts/check-docs-contract.mjs` listed this page among the pages
lacking a contract.

This page teaches three examples: the hero demo (`docs/src/app/(docs)/react/components/toggle-group/page.mdx:9-11`),
the single inline Anatomy fence (`docs/src/app/(docs)/react/components/toggle-group/page.mdx:17-21`) and the
`### Multiple` demo (`docs/src/app/(docs)/react/components/toggle-group/page.mdx:29-31`). The port's real surface
is the `#[component] ToggleGroup` in `crates/leptos-ui/src/toggle_group.rs:694-745` over `toggle_group_view`,
whose children are the `toggle` unit's element-description builder `leptos_ui::toggle_element` — the group
renders a single `div` and exports no subcomponents of its own (`specs/library/toggle-group/behavior.md`
§ Public API surface), so there is no `ToggleGroup::Part` tree to teach, and the snippets do not show a
flattened `*_view(..)` call for the group either.

| example (upstream citation) | Leptos snippet to show | behavioural obligations (cited) | observable that proves it |
| --- | --- | --- | --- |
| Anatomy — import and use it as a single part (`docs/src/app/(docs)/react/components/toggle-group/page.mdx:17-21`) | `use leptos_ui::{ToggleGroup, ToggleProps, toggle_element};` then `view! { <ToggleGroup …>{…}</ToggleGroup> }` — the port's component in `view!` markup with one composed child, each child an element description materialized into a view | `specs/library/toggle-group/behavior.md` § Public API surface (a single root part, `div`, no subcomponents) and § DOM structure (`packages/react/src/toggle-group/ToggleGroup.test.tsx:13-16`, "Root is a single `div`") | the block classifies as Leptos rather than upstream's JSX, and the page reads `{total: 1, leptos: 1, react: 0, other: 0}` (`crates/docs-app/src/pages/toggle_group_page.rs`'s `snippet_language_guard`, the browser-free copy of the probe's rules); the snippet's shape also COMPILES (`anatomy_snippet_shape`), so a snippet naming an API the port lacks fails the build |
| Hero demo (`docs/src/app/(docs)/react/components/toggle-group/page.mdx:9-11`; source `docs/src/app/(docs)/react/components/toggle-group/demos/hero/tailwind/index.tsx:6-35`) | the port's `ToggleGroup` with `default_value=vec!["left".to_string()]`, upstream's panel class verbatim, its `aria-label` through the port's `element_attributes` rest bag (that IS upstream's `...elementProps` spread), and three grouped `toggle_element` children whose `value`, `aria-label` and `className` are the demo's | `specs/library/toggle-group/behavior.md` § State model (`packages/react/src/toggle-group/ToggleGroup.test.tsx:63-67` — `defaultValue={['two']}` marks that item pressed at mount; `:48-52` — pressing a second item unpresses the first, the single-selection default), § Accessibility (`:18-22` — `role="group"`, nameable via `aria-label`; `:39-52` — each child's `aria-pressed` reflects its membership in the group value), § DOM structure (`:44-45,50-51` — `data-pressed` on pressed items); `specs/docs-content/toggle-group/demos.json` entry 1 (`stateManaged: "uncontrolled … initialized with defaultValue={['left']} (single selection)"`) | `crates/docs-app/src/render_test.rs`'s `toggle_group_page_renders_its_demos_through_the_real_group`: the root is one `div[role="group"]` carrying `aria-label="Text alignment"`, `data-orientation="horizontal"` and no `data-multiple`; the first child carries `aria-pressed="true"` plus `data-pressed` and the other two `"false"`; and a dispatched click on the second child leaves exactly that one pressed — the compiled shape of this row is verified at this tree, the DOM assertions are compile-only here (see the gaps below) |
| Multiple demo (`docs/src/app/(docs)/react/components/toggle-group/page.mdx:29-31`; source `docs/src/app/(docs)/react/components/toggle-group/demos/multiple/tailwind/index.tsx:4-35`) | the same composition with `multiple=true` and `default_value=vec!["bold".to_string(), "italic".to_string()]`, its `aria-label="Text formatting options"` and its three labelled children | `specs/library/toggle-group/behavior.md` § State model (`packages/react/src/toggle-group/ToggleGroup.test.tsx:244-261` — under `multiple` pressing a second item keeps the first pressed; `:235-241` — `data-multiple` present only when the prop is set, absent otherwise, and it reflects the prop exactly), § DOM structure (`:235-241`); `specs/docs-content/toggle-group/demos.json` entry 2 (`stateManaged: "uncontrolled … with multiple enabled"`) | the same render test: the second root carries the bare `data-multiple` marker and its three children read `aria-pressed` `true, true, false` — two pressed at once, which single selection cannot produce |

Gaps carried open against this contract (do not mark more of this page done over them):

* **The composed-child ergonomics gap.** Upstream teaches `<ToggleGroup><Toggle value="left" /></ToggleGroup>`;
  the port has no `Toggle` **component** — the toggle unit ships the element description `leptos_ui::toggle_element`
  plus `RenderedElement::create_element` — so a grouped child is materialized through a view bridge (this page uses
  the docs app's own `crate::pages::use_render_page::RawElementView`; the crate-side twin is the avatar-named
  `leptos_ui::AvatarDocView`). Owned by `docs-ergonomics: mirrored snippets must read like upstream's (namespaced
  components, size parity)`, not by this item.
* **`render` and the forwarded `ref` are not exposed on `ToggleGroup`.** Its view path builds a fixed `<div>`, so
  the element form of `render` (which replaces the tag, `packages/react/src/internals/useRenderElement.tsx:164-196`)
  would be silently dropped — the unit's own recorded decision (`crates/leptos-ui/src/toggle_group.rs` module docs,
  "Rust adaptations"). Owned by `library: the view paths drop render's element form`.
* **The `Toolbar`-nesting branch is structurally absent, not emulated** (`specs/library/toggle-group/behavior.md`
  § DOM structure, `packages/react/src/toggle-group/ToggleGroup.test.tsx:322-325,339-365`): `library: toolbar` has no ledger item
  (`tooling: library: toolbar has no ledger item — the unit cannot be scheduled`), so the blocked behaviour is
  scoped there rather than dropped here.
* **The rendered axes are UNMEASURED from this box.** `ralph/generated/env-health.json` reports
  `browser: DEGRADED — browser gates REFUSED here by lib/browser-budget.mjs (4 GB cgroup)`, so this route's
  structure/page-parity/widget-parity/copy axes come from CI: the route is registered in
  `ralph/generated/routes.json`, which is the index `.github/workflows/measure-port.yml` shards over. An
  UNMEASURED axis is not a pass — the item's own note records the same.
