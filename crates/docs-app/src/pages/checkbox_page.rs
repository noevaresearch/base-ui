//! The docs page for `Checkbox`, mirroring
//! `docs/src/app/(docs)/react/components/checkbox/page.mdx`
//! (`specs/docs-content/checkbox/page.md`) — the `docs-content: components/checkbox`
//! TODO item.
//!
//! Page structure per the spec's "Page structure (headings, in order)" section:
//! `# Checkbox` h1, the `<Subtitle>` ("An easily stylable checkbox component."),
//! the hero demo before the first heading, `## Usage guidelines`, `## Anatomy`
//! with its fenced snippet, `## Examples` over "Labeling a checkbox"
//! (`page.mdx:31-42`), "Rendering as a native button" (`:44-72`) and "Form
//! integration" (`:74-88`), and `## API reference` over the two generated
//! `TypesCheckbox` reference tables (`page.mdx:90-100`), echoed as static prose
//! per the accordion/button/meter/field page precedent: the port has no docs
//! generator, so the tables' documented props and data attributes are rendered as
//! text, never fabricated as executable machinery.
//!
//! Page furniture mirrored in module docs (the accordion/separator/field page
//! precedent): the `<Meta name="description">` content — "A high-quality, unstyled
//! React checkbox component that is easy to customize." (`page.mdx:4-7`) — and the
//! trailing `export const metadata` SEO keywords block (16 keywords, `page.mdx:102-121`:
//! 'React Checkbox', 'Checkbox Component', 'Accessible Checkbox', 'Customizable
//! Checkbox', 'Form Control Checkbox', 'Checkmark', 'Tick Box', 'Selection Control',
//! 'Checkbox Input', 'Binary Input', 'Checked State', 'Indeterminate Checkbox',
//! 'Tri-State Checkbox', 'Headless React Components', 'Input Indicator', 'Base UI').
//!
//! The live demo is the upstream Tailwind hero
//! (`docs/src/app/(docs)/react/components/checkbox/demos/hero/tailwind/index.tsx`,
//! the single `specs/docs-content/checkbox/demos.json` entry, whose two recorded
//! citation windows cover `:6-16` — the label + Root — and `:20-34` — the
//! `CheckIcon` svg) ported onto the REAL `leptos_ui::checkbox` parts. Unlike the
//! accordion/field pages, the checkbox crate exposes the unit as view functions
//! rather than `#[component]` wrappers (`checkbox_root_view` /
//! `checkbox_indicator_view`, `crates/leptos-ui/src/checkbox/mod.rs:34-41`), so the
//! demo composes the props structs directly — the shape the crate's own wasm suite
//! uses (`checkbox_tests.rs:508-518`). Every upstream `className` string is carried
//! verbatim so the DOM the Leptos port produces matches the React demo's
//! element-for-element.
//!
//! The demo is uncontrolled (`defaultChecked`, `hero/tailwind/index.tsx:8`):
//! `specs/docs-content/checkbox/behavior.md`'s sibling — the library unit's
//! `behavior.md` § State model — makes the port's own controlled boolean and
//! Indicator render gate own the ticked state, exactly as upstream's does; the
//! label's enclosing `<label>` is upstream's "simplest labeling pattern"
//! (`page.mdx:33`), which behavior.md § Accessibility proves produces the implicit
//! association.

use leptos::prelude::*;

use leptos_ui::checkbox_indicator_view;
use leptos_ui::checkbox_root_view;
use leptos_ui::{CheckboxIndicatorViewProps, CheckboxRootViewProps};

/// The upstream label `className` (`hero/tailwind/index.tsx:6`) — the enclosing
/// `<label>`, carried verbatim through the Tailwind variant colons.
const DEMO_LABEL_CLASS: &str =
    "flex items-center gap-2 text-sm font-normal text-neutral-950 dark:text-white";

/// The upstream Root `className` (`hero/tailwind/index.tsx:9`) — including the
/// `data-checked:`/`focus-visible:` variants the port's real state attributes
/// feed through the stylesheet.
const DEMO_CHECKBOX_CLASS: &str = "flex size-4 shrink-0 items-center justify-center border rounded-none p-0 border-neutral-950 bg-white text-white dark:border-white dark:bg-neutral-950 dark:text-neutral-950 data-checked:bg-neutral-950 data-checked:text-white dark:data-checked:bg-white dark:data-checked:text-neutral-950 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-950 dark:focus-visible:outline-white";

/// The upstream Indicator `className` (`hero/tailwind/index.tsx:11`) — the
/// `data-unchecked:hidden` variant is why the demo's checkmark is visible only
/// while the box is ticked.
const DEMO_INDICATOR_CLASS: &str = "flex data-unchecked:hidden";

/// The `## Anatomy` snippet (`page.mdx:21-27`), carried verbatim.
const ANATOMY_SNIPPET: &str = "import { Checkbox } from '@base-ui/react/checkbox';\n\n<Checkbox.Root>\n  <Checkbox.Indicator />\n</Checkbox.Root>;";

/// The "Labeling a checkbox" snippet (`page.mdx:35-42`), carried verbatim —
/// including its `// @highlight` directives, which are comments in the source.
const LABELING_SNIPPET: &str = "// @highlight\n<label>\n  <Checkbox.Root />\n  Accept terms and conditions\n  {/* @highlight */}\n</label>";

/// The "Sibling label pattern with a native button" snippet (`page.mdx:48-56`).
const NATIVE_BUTTON_SNIPPET: &str = "<div>\n  <label htmlFor=\"notifications-checkbox\">Enable notifications</label>\n  {/* @highlight-text \"nativeButton\" \"render={<button />}\" */}\n  <Checkbox.Root id=\"notifications-checkbox\" nativeButton render={<button />}>\n    <Checkbox.Indicator />\n  </Checkbox.Root>\n</div>";

/// The "Render callback" snippet (`page.mdx:60-72`) — the invalid-HTML rationale
/// the page spec flags under Discrepancies as unproven by behavior.md.
const RENDER_CALLBACK_SNIPPET: &str = "<Checkbox.Root\n  nativeButton\n  // @highlight-start\n  render={(buttonProps) => (\n    <label>\n      <button {...buttonProps} />\n      Enable notifications\n    </label>\n  )}\n  {/* @highlight-end */}\n/>";

/// The "Using Checkbox in a form" snippet (`page.mdx:78-88`).
const FORM_SNIPPET: &str = "<Form>\n  {/* @highlight */}\n  <Field.Root name=\"stayLoggedIn\">\n    <Field.Label>\n      <Checkbox.Root />\n      Stay logged in for 7 days\n    </Field.Label>\n  </Field.Root>\n</Form>";

/// The demo's checkmark (`hero/tailwind/index.tsx:20-34`) — a 16×16 stroke svg
/// with `display: block` inline, exactly as upstream renders it inside the
/// Indicator.
fn check_icon_view() -> AnyView {
    view! {
        <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            style="display:block"
        >
            <path d="m2.5 8.5 4 4 7-9" />
        </svg>
    }
    .into_any()
}

/// The hero demo (`demos/hero/tailwind/index.tsx`, the single demos.json entry):
/// upstream's exact JSX shape, in upstream's element order — the enclosing
/// `<label>` with the checkbox Root (ticked by `defaultChecked`) wrapping the
/// Indicator (the checkmark svg), then the label text.
#[component]
pub fn CheckboxHeroDemo() -> impl IntoView {
    let root_props = CheckboxRootViewProps {
        default_checked: Some(true),
        class: Some(DEMO_CHECKBOX_CLASS.to_string()),
        children: Some(Box::new(move || {
            checkbox_indicator_view(CheckboxIndicatorViewProps {
                class: Some(DEMO_INDICATOR_CLASS.to_string()),
                children: Some(std::sync::Arc::new(check_icon_view)),
                ..CheckboxIndicatorViewProps::default()
            })
            .into_any()
        })),
        ..CheckboxRootViewProps::default()
    };

    view! {
        <label class=DEMO_LABEL_CLASS>
            {checkbox_root_view(root_props)}
            "Enable notifications"
        </label>
    }
}

/// One API-reference block: the generated `TypesCheckbox` tables
/// (`docs/src/app/(docs)/react/components/checkbox/types.md`) echoed as static
/// prose — the summary line, the props list, and the data-attributes list.
fn api_part(summary: &'static str, props: &'static str, data_attrs: &'static str) -> impl IntoView {
    view! {
        <p class="api-summary">{summary}</p>
        <p class="api-props">{props}</p>
        <p class="api-data-attrs">{data_attrs}</p>
    }
}

/// The `docs/src/app/(docs)/react/components/checkbox/page.mdx` page.
#[component]
pub fn CheckboxPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Checkbox"</h1>
            <p class="subtitle">"An easily stylable checkbox component."</p>

            <div class="docs-demo" data-demo="hero"><CheckboxHeroDemo /></div>

            <h2>"Usage guidelines"</h2>
            <ul>
                <li>
                    <strong>"Form controls must have an accessible name"</strong>
                    ": It can be created using a `<label>` element or the `Field` component. See "
                    <a href="#labeling-a-checkbox">"Labeling a checkbox"</a>
                    " and the "
                    <a href="/react/handbook/forms">"forms guide"</a>
                    "."
                </li>
            </ul>

            <h2>"Anatomy"</h2>
            <p>"Import the component and assemble its parts:"</p>
            <pre><code>{ANATOMY_SNIPPET}</code></pre>

            <h2>"Examples"</h2>

            <h3>"Labeling a checkbox"</h3>
            <p>"An enclosing `<label>` is the simplest labeling pattern:"</p>
            <pre><code>{LABELING_SNIPPET}</code></pre>

            <h3>"Rendering as a native button"</h3>
            <p>
                "By default, `<Checkbox.Root>` renders a `<span>` element to support enclosing "
                "labels. Prefer rendering the checkbox as a native button when using sibling "
                "labels (`htmlFor`/`id`)."
            </p>
            <pre><code>{NATIVE_BUTTON_SNIPPET}</code></pre>
            <p>
                "Native buttons with wrapping labels are supported by using the `render` callback "
                "to avoid invalid HTML, so the hidden input is placed outside the label:"
            </p>
            <pre><code>{RENDER_CALLBACK_SNIPPET}</code></pre>

            <h3>"Form integration"</h3>
            <p>
                "Use "
                <a href="/react/components/field">"Field"</a>
                " to handle label associations and form integration:"
            </p>
            <pre><code>{FORM_SNIPPET}</code></pre>

            <h2>"API reference"</h2>
            <h3>"Root"</h3>
            {api_part(
                "Represents the checkbox itself. Renders a <span> element and a hidden <input> beside.",
                "Props: name (string — identifies the field when a form is submitted), defaultChecked (boolean, false — whether the checkbox is initially ticked; for a controlled checkbox use checked instead), checked (boolean — whether the checkbox is currently ticked; for an uncontrolled checkbox use defaultChecked instead), onCheckedChange ((checked: boolean, eventDetails: Checkbox.Root.ChangeEventDetails) => void — called when the checkbox is ticked or unticked), indeterminate (boolean, false — whether the checkbox is in a mixed state: neither ticked, nor unticked), value (string — the checkbox's value; identifies it within a Checkbox Group, falling back to name when omitted; when submitting a form, a checked box submits value, and with no value it submits the native \"on\"), form (string — identifies the form that owns the hidden input; useful when the checkbox is rendered outside the form), nativeButton (boolean, false — whether the component renders a native <button> element when replacing it via the render prop; set to true if the rendered element is a native button), parent (boolean, false — whether the checkbox controls a group of child checkboxes; must be used in a Checkbox Group), uncheckedValue (string — the value submitted with the form when the checkbox is unchecked; by default, unchecked checkboxes do not submit any value, matching native checkbox behavior), disabled (boolean, false — whether the component should ignore user interaction), readOnly (boolean, false — whether the user should be unable to tick or untick the checkbox), required (boolean, false — whether the user must tick the checkbox before submitting a form), inputRef (React.Ref<HTMLInputElement> — a ref to access the hidden <input> element), id (string — the id of the input element), className, style, render.",
                "Data attributes: data-checked (present when the checkbox is checked), data-unchecked (present when the checkbox is not checked), data-disabled, data-readonly, data-required, data-valid (present when the checkbox is in a valid state — when wrapped in Field.Root), data-invalid (present when the checkbox is in an invalid state — when wrapped in Field.Root), data-dirty (present when the checkbox's value has changed — when wrapped in Field.Root), data-touched (present when the checkbox has been touched — when wrapped in Field.Root), data-filled (present when the checkbox is checked — when wrapped in Field.Root), data-focused (present when the checkbox is focused — when wrapped in Field.Root), data-indeterminate (present when the checkbox is in an indeterminate state).",
            )}
            <h3>"Indicator"</h3>
            {api_part(
                "Indicates whether the checkbox is ticked. Renders a <span> element.",
                "Props: className, style, keepMounted (boolean, false — whether to keep the element in the DOM when the checkbox is not checked), render.",
                "Data attributes: the same state set as Root — data-checked, data-unchecked, data-disabled, data-readonly, data-required, data-valid, data-invalid, data-dirty, data-touched, data-filled, data-focused, data-indeterminate — plus data-starting-style (present when the checkbox indicator begins animating in) and data-ending-style (present when the checkbox indicator is animating out).",
            )}
        </article>
    }
}
