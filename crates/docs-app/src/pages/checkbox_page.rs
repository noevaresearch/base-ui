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
//!
//! Snippet language (`specs/docs-content/CONTRACT.md`; this page's spec carries the contract table):
//! all five embedded snippets show the PORT's API — `use leptos::prelude::*` and `view!` over the
//! `leptos_ui::checkbox` view functions — instead of upstream's JSX, which they carried verbatim
//! until this pass (the gap report's probe read `{total: 5, leptos: 0, react: 5}`: every structural
//! gate passed while the page taught React). One example — the render *callback* — cannot be
//! reproduced by the port as upstream writes it; its prose states the port's real behaviour instead
//! of repeating upstream's rationale, and the finding is recorded in
//! `ralph/logs/spec-discrepancies.md`.

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

// ---------------------------------------------------------------------------
// The page's embedded snippets, TRANSLATED to the port.
//
// `specs/docs-content/CONTRACT.md` requirement 1: a mirrored page's code blocks must demonstrate
// THIS port's API, never upstream's runtime source. These five constants used to carry
// `docs/src/app/(docs)/react/components/checkbox/page.mdx`'s JSX verbatim (gap-report probe:
// `{total: 5, leptos: 0, react: 5}`); each is now the port's own composition, per the page spec's
// § Snippet & behaviour contract table.
//
// Two conventions, applied uniformly so the snippets stay both truthful and comparable with
// upstream's:
//   * upstream's `@highlight` / `@highlight-text` / `@highlight-start`-`@highlight-end` directives
//     are kept as RUST line comments — the information a highlighter needs (which line/identifier
//     the example is about), expressed in the snippet's own comment syntax. Upstream's `{/* … */}`
//     JSX-comment spelling is not valid in Rust/RSX. `docs-chrome: code blocks` consumes them when
//     it ports the highlighter.
//   * the port exposes `Checkbox.Root`/`Checkbox.Indicator` as VIEW FUNCTIONS (no `#[component]`
//     wrappers, `crates/leptos-ui/src/checkbox/mod.rs:34-41`), so every snippet calls them and
//     passes props structs — the same shape the crate's own wasm suite uses
//     (`checkbox_tests.rs:508-518`).
// ---------------------------------------------------------------------------

/// The `## Anatomy` snippet (`page.mdx:19-27`) — "import the component and assemble its parts".
/// Translated: the import is the port's module surface, and the assembly is `view!` syntax nesting
/// `checkbox_indicator_view` inside `checkbox_root_view` (contract row "Anatomy").
const ANATOMY_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{CheckboxIndicatorViewProps, CheckboxRootViewProps};
use leptos_ui::{checkbox_indicator_view, checkbox_root_view};

view! {
    {checkbox_root_view(CheckboxRootViewProps {
        children: Some(Box::new(|| {
            checkbox_indicator_view(CheckboxIndicatorViewProps::default()).into_any()
        })),
        ..CheckboxRootViewProps::default()
    })}
}"#;

/// The "Labeling a checkbox" snippet (`page.mdx:31-41`) — the enclosing `<label>` pattern.
/// Translated: the root and its indicator nest inside a real `<label>` element, and NO `htmlFor`/`id`
/// is passed — the implicit association is the point of the example (contract row "Labeling a
/// checkbox — wrapping label"). Upstream's bare `<Checkbox.Root />` renders the port's default
/// parts, so the indicator is composed explicitly.
const LABELING_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{CheckboxIndicatorViewProps, CheckboxRootViewProps};
use leptos_ui::{checkbox_indicator_view, checkbox_root_view};

view! {
    // @highlight-start
    <label>
        {checkbox_root_view(CheckboxRootViewProps {
            children: Some(Box::new(|| {
                checkbox_indicator_view(CheckboxIndicatorViewProps::default()).into_any()
            })),
            ..CheckboxRootViewProps::default()
        })}
        "Accept terms and conditions"
    </label>
    // @highlight-end
}"#;

/// The "Rendering as a native button" snippet (`page.mdx:44-56`) — the sibling-label pattern.
/// Translated: `id` and `native_button` are the port's prop spellings (snake_case), and upstream's
/// `render={<button />}` is the port's element form of the `render` prop —
/// `RenderProp::Element { tag: "button", .. }` (`crates/leptos-ui-internals/src/use_render_element.rs:294-306`),
/// the same shape the crate's own tests use (`separator_tests.rs:330`). The sibling label keeps a
/// `for` attribute pointing at that id, which is what makes the fallback `aria-labelledby` resolve
/// (contract row "Rendering as a native button — sibling label").
const NATIVE_BUTTON_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{CheckboxIndicatorViewProps, CheckboxRootViewProps};
use leptos_ui::{checkbox_indicator_view, checkbox_root_view};
use leptos_ui_internals::use_render_element::{RenderElementProps, RenderProp};

view! {
    <div>
        <label for="notifications-checkbox">"Enable notifications"</label>
        // @highlight-text "native_button" "render"
        {checkbox_root_view(CheckboxRootViewProps {
            id: Some("notifications-checkbox".into()),
            native_button: true,
            render: Some(RenderProp::Element {
                tag: "button".into(),
                props: RenderElementProps::default(),
            }),
            children: Some(Box::new(|| {
                checkbox_indicator_view(CheckboxIndicatorViewProps::default()).into_any()
            })),
            ..CheckboxRootViewProps::default()
        })}
    </div>
}"#;

/// The "Render callback" snippet (`page.mdx:58-72`) — HONESTLY rendered, per
/// `specs/docs-content/CONTRACT.md` requirement 3 ("if an obligation cannot be proved by an
/// observable in this port yet, say so explicitly … do not write a weaker claim to make the row
/// look filled").
///
/// Upstream's example hands `render` a *callback* that owns the returned element — `<label><button
/// {...buttonProps} /></label>` — which is how it keeps the hidden input OUTSIDE the wrapping label.
/// The port honors the element form of `render` (the tag-replacement shown here,
/// `crates/leptos-ui/src/checkbox/root.rs:580-584`, `:1161-1163`) but not the callback form: the
/// Function arm is explicitly not half-ported and is recorded in `ralph/logs/spec-discrepancies.md`
/// (`root.rs:46-50`). So this snippet shows the port's real composition — the tag replaced with a
/// native `<button>` inside the wrapping label — and the prose below states the limitation instead
/// of repeating upstream's invalid-HTML rationale, which this port does not reproduce (the root
/// renders its hidden input as a sibling of the control inside its own fragment, so a wrapping
/// label encloses it).
const RENDER_CALLBACK_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{CheckboxIndicatorViewProps, CheckboxRootViewProps};
use leptos_ui::{checkbox_indicator_view, checkbox_root_view};
use leptos_ui_internals::use_render_element::{RenderElementProps, RenderProp};

view! {
    // @highlight-start
    <label>
        {checkbox_root_view(CheckboxRootViewProps {
            native_button: true,
            render: Some(RenderProp::Element {
                tag: "button".into(),
                props: RenderElementProps::default(),
            }),
            children: Some(Box::new(|| {
                checkbox_indicator_view(CheckboxIndicatorViewProps::default()).into_any()
            })),
            ..CheckboxRootViewProps::default()
        })}
        "Enable notifications"
    </label>
    // @highlight-end
}"#;

/// The "Using Checkbox in a form" snippet (`page.mdx:74-88`) — the Field integration.
/// Translated: the port's `Field` parts are view functions too
/// (`field_root_view`/`field_label_view`, `crates/leptos-ui/src/field/field_root.rs:103`,
/// `field_parts.rs:126`), so the label association upstream gets from `Field.Label` is shown as the
/// nested `view!` composition the Field provides (contract row "Form integration"). The `name`
/// rides `Field.Root`, which owns the form-value submission, and the label's text is the port's
/// explicit `children` slot (upstream takes it from the `elementProps` spread).
const FORM_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{CheckboxIndicatorViewProps, CheckboxRootViewProps};
use leptos_ui::{checkbox_indicator_view, checkbox_root_view};
use leptos_ui::field_parts::{FieldLabelViewProps, field_label_view};
use leptos_ui::field_root::{FieldRootViewProps, field_root_view};

view! {
    {field_root_view(FieldRootViewProps {
        name: Some("stayLoggedIn".into()),
        children: Some(Box::new(|| {
            view! {
                {field_label_view(FieldLabelViewProps {
                    children: Some(Box::new(|| {
                        view! {
                            {checkbox_root_view(CheckboxRootViewProps {
                                children: Some(Box::new(|| {
                                    checkbox_indicator_view(CheckboxIndicatorViewProps::default())
                                        .into_any()
                                })),
                                ..CheckboxRootViewProps::default()
                            })}
                            "Stay logged in for 7 days"
                        }
                        .into_any()
                    })),
                    ..FieldLabelViewProps::default()
                })}
            }
            .into_any()
        })),
        ..FieldRootViewProps::default()
    })}
}"#;

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
                "The port honors the `render` prop's element form, which replaces the visible "
                "element with a native `<button>`:"
            </p>
            <pre><code>{RENDER_CALLBACK_SNIPPET}</code></pre>
            <p>
                "Upstream's example passes a `render` callback that owns the returned element, which "
                "is what keeps the hidden input outside the wrapping label. That callback form is not "
                "ported yet — the port renders its hidden input as a sibling of the control inside the "
                "root's own fragment — so the composition above is what the port supports today."
            </p>

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

/// Browser-free guard for `specs/docs-content/CONTRACT.md` requirement 1: every snippet embedded in
/// this page demonstrates the PORT's API.
///
/// Why it exists: the obligation was unenforced in the host gate. `run-regression.sh` runs
/// `cargo test --workspace`; the snippet-language probe lives in `ralph/scripts/visual-gap-report.mjs`
/// and its purity term in `check-visual-budget.mjs`, both of which need BOTH dev servers up (they
/// report a NOTE and pass when the upstream reference is down). The five blocks above sat on the
/// page as upstream's JSX through every green gate, and because transcribed text counted toward
/// content recall, keeping them there *raised* the fidelity score. This module is the cheap half of
/// that obligation: it runs in the ordinary host suite, and it fails the moment a snippet teaches
/// React again.
///
/// It is deliberately two-part:
///   * `the_pages_snippets_all_teach_the_port` classifies each constant with the same rules as the
///     probe (`ralph/scripts/visual-gap-report.mjs:233-242` — the browser-side classifier, mirrored
///     here in plain string operations so no new dependency is needed), so the numbers the probe
///     would report are asserted in CI: `{total: 5, leptos: 5, react: 0}`;
///   * the `_shape` functions below compile the composition each snippet teaches, so a snippet
///     cannot name a prop, field or path the port does not actually have — the failure mode that
///     transcription made invisible. They are never called (the page's real compositions are
///     exercised by `render_test.rs`); the compiler is the assertion.
#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use leptos_ui::field_parts::{FieldLabelViewProps, field_label_view};
    use leptos_ui::field_root::{FieldRootViewProps, field_root_view};
    use leptos_ui_internals::use_render_element::{RenderElementProps, RenderProp};

    /// The upstream Anatomy block (`page.mdx:19-27`), kept here as the classifier's positive
    /// control: if `looks_react` ever stops recognising upstream's source, the assertions below
    /// would pass vacuously, and this test would say so instead.
    const UPSTREAM_ANATOMY: &str = "import { Checkbox } from '@base-ui/react/checkbox';\n\n<Checkbox.Root>\n  <Checkbox.Indicator />\n</Checkbox.Root>;";

    /// `looksReact` from the probe (`visual-gap-report.mjs:233-238`), mirrored.
    fn looks_react(text: &str) -> bool {
        let has = |needle: &str| text.contains(needle);
        // `/import\s+[\s\S]{0,120}?\sfrom\s+['"]/`
        let import_from = text.match_indices("import").any(|(i, _)| {
            let window = &text[i..text.len().min(i + 140)];
            window.contains("from '") || window.contains("from \"")
        });
        // `/<\/?[A-Z][A-Za-z]*(\.[A-Z][A-Za-z]*)?[\s/>]/` — a JSX-style tag.
        let jsx_tag = {
            let b = text.as_bytes();
            (0..b.len()).any(|i| {
                if b[i] != b'<' {
                    return false;
                }
                let mut j = i + 1;
                if b.get(j) == Some(&b'/') {
                    j += 1;
                }
                if !b.get(j).is_some_and(u8::is_ascii_uppercase) {
                    return false;
                }
                while b.get(j).is_some_and(u8::is_ascii_alphabetic) {
                    j += 1;
                }
                if b.get(j) == Some(&b'.') && b.get(j + 1).is_some_and(u8::is_ascii_uppercase) {
                    j += 2;
                    while b.get(j).is_some_and(u8::is_ascii_alphabetic) {
                        j += 1;
                    }
                }
                b.get(j)
                    .is_some_and(|c| c.is_ascii_whitespace() || *c == b'/' || *c == b'>')
            })
        };
        // `/=>\s*\(|=>\s*\{/`
        let arrow_block = {
            let b = text.as_bytes();
            (0..b.len().saturating_sub(2)).any(|i| {
                if !(b[i] == b'=' && b[i + 1] == b'>') {
                    return false;
                }
                let mut j = i + 2;
                while b.get(j).is_some_and(|c| c.is_ascii_whitespace()) {
                    j += 1;
                }
                matches!(b.get(j), Some(b'(') | Some(b'{'))
            })
        };
        has("@base-ui/react")
            || has("@mui/")
            || import_from
            || has("useState")
            || has("useRef")
            || has("useEffect")
            || has("useCallback")
            || has("className=")
            || has("onClick={")
            || has("{props")
            || arrow_block
            || jsx_tag
    }

    /// `looksLeptos` from the probe (`visual-gap-report.mjs:239-242`), mirrored.
    fn looks_leptos(text: &str) -> bool {
        let has = |needle: &str| text.contains(needle);
        has("use leptos")
            || has("leptos_ui")
            || has("leptos-ui")
            || has("view!")
            || has("#[component]")
            || has("-> impl IntoView")
            || has("cx(")
            || has("Signal<")
            || has("RwSignal")
            || has("ReadSignal")
            || has("Memo<")
            || has("on:click")
            || has("prop:")
            || has("attr:")
    }

    #[test]
    fn the_classifier_recognises_upstream_source() {
        assert!(
            looks_react(UPSTREAM_ANATOMY) && !looks_leptos(UPSTREAM_ANATOMY),
            "the classifier no longer recognises upstream's React source — the assertions below \
             would be vacuous"
        );
    }

    #[test]
    fn the_pages_snippets_all_teach_the_port() {
        let snippets = [
            ("Anatomy", ANATOMY_SNIPPET),
            ("Labeling a checkbox", LABELING_SNIPPET),
            ("Rendering as a native button", NATIVE_BUTTON_SNIPPET),
            ("Render callback", RENDER_CALLBACK_SNIPPET),
            ("Form integration", FORM_SNIPPET),
        ];
        let (mut leptos, mut react, mut other) = (0, 0, 0);
        for (name, text) in snippets {
            match (looks_leptos(text), looks_react(text)) {
                (true, false) => leptos += 1,
                (_, true) => {
                    react += 1;
                    panic!("the '{name}' snippet still carries React source");
                }
                _ => {
                    other += 1;
                    panic!("the '{name}' snippet identifies as neither port nor React source");
                }
            }
        }
        assert_eq!(
            (leptos, react, other),
            (5, 0, 0),
            "the probe must read {{total: 5, leptos: 5, react: 0}} for this page"
        );
    }
    // --- the snippets' shapes, compiled ------------------------------------------------------
    // Each mirrors its snippet's composition verbatim (imports included, at the top of this
    // module). Never called: the compiler checks the props, fields and paths the page teaches.

    #[allow(dead_code)]
    fn anatomy_snippet_shape() -> impl IntoView {
        view! {
            {checkbox_root_view(CheckboxRootViewProps {
                children: Some(Box::new(|| {
                    checkbox_indicator_view(CheckboxIndicatorViewProps::default()).into_any()
                })),
                ..CheckboxRootViewProps::default()
            })}
        }
    }

    #[allow(dead_code)]
    fn labeling_snippet_shape() -> impl IntoView {
        view! {
            <label>
                {checkbox_root_view(CheckboxRootViewProps {
                    children: Some(Box::new(|| {
                        checkbox_indicator_view(CheckboxIndicatorViewProps::default()).into_any()
                    })),
                    ..CheckboxRootViewProps::default()
                })}
                "Accept terms and conditions"
            </label>
        }
    }

    #[allow(dead_code)]
    fn native_button_snippet_shape() -> impl IntoView {
        view! {
            <div>
                <label for="notifications-checkbox">"Enable notifications"</label>
                {checkbox_root_view(CheckboxRootViewProps {
                    id: Some("notifications-checkbox".into()),
                    native_button: true,
                    render: Some(RenderProp::Element {
                        tag: "button".into(),
                        props: RenderElementProps::default(),
                    }),
                    children: Some(Box::new(|| {
                        checkbox_indicator_view(CheckboxIndicatorViewProps::default()).into_any()
                    })),
                    ..CheckboxRootViewProps::default()
                })}
            </div>
        }
    }

    #[allow(dead_code)]
    fn render_callback_snippet_shape() -> impl IntoView {
        view! {
            <label>
                {checkbox_root_view(CheckboxRootViewProps {
                    native_button: true,
                    render: Some(RenderProp::Element {
                        tag: "button".into(),
                        props: RenderElementProps::default(),
                    }),
                    children: Some(Box::new(|| {
                        checkbox_indicator_view(CheckboxIndicatorViewProps::default()).into_any()
                    })),
                    ..CheckboxRootViewProps::default()
                })}
                "Enable notifications"
            </label>
        }
    }

    #[allow(dead_code)]
    fn form_snippet_shape() -> impl IntoView {
        view! {
            {field_root_view(FieldRootViewProps {
                name: Some("stayLoggedIn".into()),
                children: Some(Box::new(|| {
                    view! {
                        {field_label_view(FieldLabelViewProps {
                            children: Some(Box::new(|| {
                                view! {
                                    {checkbox_root_view(CheckboxRootViewProps {
                                        children: Some(Box::new(|| {
                                            checkbox_indicator_view(CheckboxIndicatorViewProps::default())
                                                .into_any()
                                        })),
                                        ..CheckboxRootViewProps::default()
                                    })}
                                    "Stay logged in for 7 days"
                                }
                                .into_any()
                            })),
                            ..FieldLabelViewProps::default()
                        })}
                    }
                    .into_any()
                })),
                ..FieldRootViewProps::default()
            })}
        }
    }
}
