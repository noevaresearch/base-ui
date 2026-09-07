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
