# button — docs page spec

Mined from the docs page file only:

- `docs/src/app/(docs)/react/components/button/page.mdx` (81 lines)

Cross-checked against `specs/library/button/behavior.md` (the component's own behavior spec, mined from its tests). That spec is cited by section name; its claims are NOT re-derived here.

## Page structure (headings, in order)

- H1 `# Button` — `docs/src/app/(docs)/react/components/button/page.mdx:1`
- `<Subtitle>` one-liner (not a heading, but part of the page header block): "A button component that can be rendered as another tag or focusable when disabled." — `docs/src/app/(docs)/react/components/button/page.mdx:3-5`
- `<Meta name="description">` for the page: "A high-quality, unstyled React button component that can be rendered as another tag or focusable when disabled." — `docs/src/app/(docs)/react/components/button/page.mdx:6-9`
- Hero demo render: imports `DemoButtonHero` from `./demos/hero` and renders it immediately after the header block — `docs/src/app/(docs)/react/components/button/page.mdx:11-13`
- H2 `## Usage guidelines` — `docs/src/app/(docs)/react/components/button/page.mdx:15` (exactly two bullets — `docs/src/app/(docs)/react/components/button/page.mdx:17-18`)
- H2 `## Anatomy` — `docs/src/app/(docs)/react/components/button/page.mdx:20` (intro "Import the component:" — `docs/src/app/(docs)/react/components/button/page.mdx:22`)
- H2 `## Examples` — `docs/src/app/(docs)/react/components/button/page.mdx:30`
  - H3 `### Rendering as another tag` — `docs/src/app/(docs)/react/components/button/page.mdx:32`
  - H3 `### Rendering links as buttons` — `docs/src/app/(docs)/react/components/button/page.mdx:45`
  - H3 `### Loading states` — `docs/src/app/(docs)/react/components/button/page.mdx:51`
- H2 `## API reference` — `docs/src/app/(docs)/react/components/button/page.mdx:59`
- `export const metadata` block with SEO keywords (React Button, Button Component, Focusable Disabled Button, Custom Element Button, Clickable Element, Action Button, Submit Button, Accessible Button, Loading State Button, Button as Link, Link Button, Headless React Components, Base UI) — `docs/src/app/(docs)/react/components/button/page.mdx:65-81`

Additional demo/type imports rendered inline mid-page (demo source files themselves are out of scope here — Stage 2):

- `import { DemoButtonLoading } from './demos/loading'` + render — `docs/src/app/(docs)/react/components/button/page.mdx:55-57`
- `import { TypesButton } from './types'` + render — `docs/src/app/(docs)/react/components/button/page.mdx:61-63`

## Prose claims about component behavior

Only claims made by the page text itself, each cross-checked against the behavior spec (`specs/library/button/behavior.md`).

- "A button component that can be rendered as another tag or focusable when disabled." (Subtitle; Meta description repeats it and adds "high-quality, unstyled React button component") — `docs/src/app/(docs)/react/components/button/page.mdx:3-5`, `docs/src/app/(docs)/react/components/button/page.mdx:6-9`. Consistent with the behavior spec's "Public API surface" section: `render` and `focusableWhenDisabled` are both directly exercised props, and the default root is a native `<button>` ("DOM structure & portal behavior").
- "Unlike the native button element, `type="submit"` must be specified on Button for it to act as a submit button." — `docs/src/app/(docs)/react/components/button/page.mdx:17`. Not covered by the behavior spec: no test exercises `type` or form submission anywhere in it (its "Public API surface" lists every prop the tests touch; `type` is not among them). Docs-only claim — see Discrepancies.
- "The Button component enforces button semantics (`role="button"`, keyboard interaction, disabled state). It should not be used for links." — `docs/src/app/(docs)/react/components/button/page.mdx:18`. Consistent with the behavior spec's "Accessibility" section (custom element gets `role="button"`), "Keyboard interactions" section (Enter/Space dispatch real clicks on custom elements; all activation suppressed while disabled), and "Focus management" section (disabled focus rules). The "should not be used for links" half is usage guidance elaborated by the page's own "Rendering links as buttons" section.
- Anatomy: "Import the component:" followed by a snippet importing `Button` from `@base-ui/react/button` and rendering a bare `<Button />;` — `docs/src/app/(docs)/react/components/button/page.mdx:22-28`. Consistent with the behavior spec's "Public API surface" and "DOM structure & portal behavior" sections (single root component, default native `<button>`, no sub-parts exercised).
- "The button can remain keyboard accessible while being rendered as another tag, such as a `<div>`, by specifying `nativeButton={false}`." — `docs/src/app/(docs)/react/components/button/page.mdx:34`. Consistent with the behavior spec's "Focus management" section (custom element, non-disabled: `tabindex="0"`, reachable via Tab), "Accessibility" section (`role="button"`, `tabindex="0"`), and "Keyboard interactions" section (Enter/Space activate). The docs' example tag is a `<div>` while the tests exercise `<a>` and `<span>` roots, but the mechanism is tag-agnostic per the behavior spec's "DOM structure & portal behavior" section (the render element replaces the root; no wrapper added).
- "The Button component enforces button semantics. `nativeButton={false}` signals that the rendered tag is not a `<button>`, but it must still be a tag that can receive button semantics (`role="button"`, keyboard interaction handlers). Links (`<a>`) have their own semantics and should not be rendered as buttons through the `render` prop." — `docs/src/app/(docs)/react/components/button/page.mdx:47`. The semantics-enforcement half matches the behavior spec's "Accessibility" section; the `<a>` prohibition is usage guidance, while the behavior spec's "DOM structure & portal behavior" section documents tests exercising exactly an `<a>` render target (tag preserved, button role applied) — see Discrepancies for the guidance-vs-tested-surface nuance.
- "If a link needs to look like a button visually, style the `<a>` element directly with CSS rather than using the Button component." — `docs/src/app/(docs)/react/components/button/page.mdx:49`. Pure usage guidance; no behavioral claim to check against the behavior spec.
- "For buttons that enter a loading state after activation, specify `focusableWhenDisabled` so focus remains on the button while it is disabled." — `docs/src/app/(docs)/react/components/button/page.mdx:53`. Consistent with the behavior spec's "Focus management" section (`focusableWhenDisabled` removes the disabled/focus-blocking behavior, sets `tabindex="0"`, gains focus via Tab) and "State model" section (a button that becomes `disabled` while focused retains focus).
- "Because some browser and screen reader combinations do not reliably announce changes to a focused button's descendant text, use [`aria-labelledby`] to make the changing text the button's explicit accessible name." — `docs/src/app/(docs)/react/components/button/page.mdx:53`. Untested guidance: the behavior spec's "Accessibility" section explicitly records that no `aria-*` id-linking (`aria-labelledby`/`id`) is asserted anywhere in the tests (marked N/A there). Docs-only recommendation — see Discrepancies.

## API tables referenced (props/parts documented on this page)

- The page contains NO literal Markdown API tables. The `## API reference` section consists solely of a rendered React component: `import { TypesButton } from './types'` + `<TypesButton />` — `docs/src/app/(docs)/react/components/button/page.mdx:59-63`.
- Props/attributes named in the page's prose and inline snippets (no tables, just usage): `type="submit"` — `docs/src/app/(docs)/react/components/button/page.mdx:17`; `nativeButton={false}` — `docs/src/app/(docs)/react/components/button/page.mdx:34`, `docs/src/app/(docs)/react/components/button/page.mdx:40`, `docs/src/app/(docs)/react/components/button/page.mdx:47`; `render` (element form, `render={<div />}`) — `docs/src/app/(docs)/react/components/button/page.mdx:40`, `docs/src/app/(docs)/react/components/button/page.mdx:47`; `focusableWhenDisabled` — `docs/src/app/(docs)/react/components/button/page.mdx:53`; `aria-labelledby` (recommended HTML attribute guidance, not framed as a Button prop) — `docs/src/app/(docs)/react/components/button/page.mdx:53`.
- N/A otherwise: no props table, no parts table, no return-value table exists in the .mdx itself. (The rendered `TypesButton` component's contents are generated reference material outside this page's .mdx.)

## Code snippets embedded directly in the .mdx (not pulled from demos/)

Both snippets below are inline in the page file (fenced code blocks); the two `<Demo... />` renders and `<TypesButton />` are component imports, not snippets, and their sources are Stage 2's scope.

- **"Anatomy"** (jsx): imports `Button` from `@base-ui/react/button` and shows a bare self-closing `<Button />;` — `docs/src/app/(docs)/react/components/button/page.mdx:24-28`.
- **"Custom tag button"** (jsx, with a `// @highlight-text "nativeButton={false}"` directive): imports `Button` from `@base-ui/react/button` and shows `<Button render={<div />} nativeButton={false}>Button that can contain complex children</Button>;` — `docs/src/app/(docs)/react/components/button/page.mdx:36-43` (directive on `docs/src/app/(docs)/react/components/button/page.mdx:39`).

## Discrepancies (docs page vs. behavior.md)

No hard contradictions found. Coverage and nuance notes:

- **`type="submit"` claim is docs-only**: the page states "`type="submit"` must be specified on Button for it to act as a submit button" (`docs/src/app/(docs)/react/components/button/page.mdx:17`). The behavior spec is silent on `type`/form submission — no test exercises it, and the prop is absent from its "Public API surface" enumeration. Unverified by the unit's tests; not contradicted.
- **`aria-labelledby` loading-state guidance is docs-only**: the page recommends `aria-labelledby` so loading-text changes become the button's explicit accessible name (`docs/src/app/(docs)/react/components/button/page.mdx:53`), while the behavior spec's "Accessibility" section explicitly records that no `aria-labelledby`/id-linking is asserted anywhere in the tests (marked N/A there). Recommendation without test backing; not contradicted.
- **Guidance vs. tested surface for `<a>` render targets**: the page says links "should not be rendered as buttons through the `render` prop" (`docs/src/app/(docs)/react/components/button/page.mdx:47`), but the behavior spec's "DOM structure & portal behavior" section documents tests exercising an `<a>` render target (tag stays 'A', button role applied), and its "Keyboard interactions" section proves Space/Enter activation on that `<a>` (including Space `preventDefault()` preventing link-scroll navigation). The tests prove the mechanism works on `<a>`; the docs position it as misuse. Not a factual mismatch, but the docs' guidance is stricter than the tested/documented surface.
- **Silent omissions** (behavior spec covers it; page says nothing — informational only, not errors): native `disabled` attribute + Tab-skipping, custom-element disabled pattern (`aria-disabled="true"` + `data-disabled` + `tabindex="-1"`), suppression of click/mousedown/pointerdown/keydown while disabled, `mousemove` still firing under `focusableWhenDisabled` while click stays blocked, modifier-key preservation on synthesized keyboard clicks, and capture/bubble event composition — all in the behavior spec's "Focus management", "Accessibility", "Events", and "Edge cases" sections; the page makes no claims contradicting them.

## Cross-links to other docs pages

Links in the page body (excluding the `./demos/*` and `./types` relative imports listed under Page structure):

- `[Rendering links as buttons](#rendering-links-as-buttons)` — same-page anchor link (not another page, listed for completeness) — `docs/src/app/(docs)/react/components/button/page.mdx:18`
- `[aria-labelledby](https://www.w3.org/TR/accname-1.2/#computation-steps)` — external W3C spec link, not a docs page — `docs/src/app/(docs)/react/components/button/page.mdx:53`
- N/A otherwise: the page contains no links to other docs pages.

## Snippet & behaviour contract

Per `specs/docs-content/CONTRACT.md`: every snippet on this page must show the **port's** API, and
each demo must reproduce upstream's behaviour. Authored 2026-09-15, after this page was found marked
`done` while carrying no contract and teaching upstream's React source in both of its code blocks
(gap report probe: `{total: 2, leptos: 0, react: 2}` — `import { Button } from '@base-ui/react/button'`
plus JSX, verbatim from the `.mdx`).

This page teaches two inline snippets plus two demos (hero + loading). The Leptos column names what
the snippet must show; the port's real surface for this component is an **element description**, not
a `#[component]`: `leptos_ui::button_element(leptos_ui::ButtonProps) -> Option<RenderedElement>`
(`crates/leptos-ui/src/button.rs:155`), with the props struct spelled in snake_case
(`disabled`, `focusable_when_disabled`, `native_button`, `render_class_style`, `element_attributes`,
`handlers`) and the resulting description materialized with `RenderedElement::create_element()`
(`crates/leptos-ui-internals/src/use_render_element.rs:536`). There is no `button_view` counterpart
to the checkbox crate's `checkbox_root_view`; materializing the description is the port's idiom, the
same one the crate's own button tests use.

| example (upstream citation) | Leptos snippet to show | behavioural obligations (cited) | observable that proves it |
| --- | --- | --- | --- |
| Anatomy — import and render the component (`docs/src/app/(docs)/react/components/button/page.mdx:24-28`) | import `leptos_ui::{ButtonProps, button_element}` and build the element with `ButtonProps::default()` | `specs/library/button/behavior.md` § Public API surface (Button is a single public root part — no sub-parts to assemble), § DOM structure & portal behavior (the default root is a native `<button>`; no wrapper element is added) | the materialized root is a native `<button>` element carrying button semantics, with no extra wrapper node |
| Rendering as another tag — `render={<div />} nativeButton={false}` (`docs/src/app/(docs)/react/components/button/page.mdx:36-43`, `@highlight-text "nativeButton={false}"` on `:39`) | `ButtonProps { native_button: false, render_class_style: UseRenderElementComponentProps { render: Some(RenderProp::Element { tag: "div".into(), .. }), .. }, .. }` — snake_case prop, the port's element-form `render` prop | `specs/library/button/behavior.md` § Focus management (a custom element that is not disabled is reachable by Tab and carries `tabindex="0"`), § Accessibility (the custom element gets `role="button"`), § Keyboard interactions (Enter and Space dispatch a real click on the custom element) | the rendered tag is a `<div>` carrying `role="button"` and `tabindex="0"`; Enter/Space on it dispatch a click |
| Hero demo (`docs/src/app/(docs)/react/components/button/page.mdx:11-13`; source cited in `specs/docs-content/button/demos.json`, `docs/src/app/(docs)/react/components/button/demos/hero/tailwind/index.tsx:1-9`) | `button_element` with the demo's `className` through `render_class_style.class_name` and `"Submit"` as the element's content | `specs/library/button/behavior.md` § Public API surface (`className`/`style`/`render` are evaluated through the `useRenderElement` component-props vocabulary), § DOM structure & portal behavior (native `<button>`, no wrapper) | the rendered native `<button>` carries the demo's class string and the text "Submit"; it manages no state (`specs/docs-content/button/demos.json`, `stateManaged: "none"`), and the page's `button_hero_demo` mount proves the composition |
| Loading states demo (`docs/src/app/(docs)/react/components/button/page.mdx:55-57`; source `docs/src/app/(docs)/react/components/button/demos/loading/tailwind/index.tsx:1-25`) with its interaction observable | `loading` as a port-side signal, `focusable_when_disabled: true`, `aria-labelledby` fed from the port's `use_base_ui_id` id generator through `element_attributes`, and the consumer's `on_click` in `ButtonHandlers` | `specs/library/button/behavior.md` § Focus management (`focusableWhenDisabled` drops the disabled/focus-blocking behaviour and keeps the button Tab-reachable; a button that becomes `disabled` while focused retains focus), § Accessibility (the disabled custom-element pattern is `aria-disabled="true"` + `data-disabled`), § Events (the internal disabled guard runs before the consumer's handler, so activation while disabled never reaches `onClick`), § State model (`disabled` is the demo's controlled mirror; Button itself stays uncontrolled) | a real click while enabled sets the disabled state and swaps the label to "Submitting"; a click during the disabled phase is swallowed — the consumer's `onClick` does not fire; after the demo's 4-second reset the button re-enables and the click reaches the consumer again (render test `button_loading_demo_runs_the_full_state_cycle_through_the_real_port`) |

Gaps carried open against this contract (do not mark this page's snippet work done over them):

* the file-selector tabs, copy control and syntax highlighting upstream attaches to every code block
  are `docs-chrome: code blocks` / `docs-chrome: demo panels` scope, not snippet-language scope; this
  page renders both of its blocks as bare `<pre><code>`.
* the `## API reference` section renders the generated `TypesButton` content as static prose rather
  than upstream's rendered table — that is `docs-chrome: API reference tables` scope, whose
  blocked-note names this page's contract as its unblock step.
* upstream's guidance that `<a>` should not be rendered as buttons, set against the behaviour spec's
  tests that exercise exactly an `<a>` render target, is already recorded in this spec's
  Discrepancies section; the contract does not restate it.
