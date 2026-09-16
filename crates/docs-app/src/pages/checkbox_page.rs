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
//! `TypesCheckbox` reference sections (`page.mdx:90-100`): each part's summary, its
//! generated props table (one `<details>` row per prop, `Name`/`Description`/`Type`/
//! `Default`) and its generated data-attributes table, over the content
//! `docs/src/app/(docs)/react/components/checkbox/types.md` carries. The port has no
//! docs generator, so that content is carried as data in this file and rendered through
//! the ported reference primitives (`crate::reference`), which reproduce the element
//! shape and class names upstream's own renderer produces — never fabricated as
//! executable machinery.
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

use crate::code_block::{Lang, code_block};
use leptos::prelude::*;

use crate::reference::{self, DataAttributeRow, ReferenceProp, Segment};

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

// ---------------------------------------------------------------------------
// The generated `## API reference` content.
//
// Source: `docs/src/app/(docs)/react/components/checkbox/types.md` (17 kB, generated by
// `pnpm docs:api` from the component's type definitions), which is what upstream's page renders
// through `<TypesCheckbox.Root />` / `<TypesCheckbox.Indicator />` (`page.mdx:92-100`). The port
// has no docs generator, so the generated content is carried here as data — the same approach the
// page's snippets take — and rendered by `crate::reference`, whose element shape and class names
// are upstream's.
//
// The descriptions' inline code spans and links are transcribed where the generated table has
// them (upstream keeps them in the DOM, and the fidelity gate's `codeBlocks`/`links` recall terms
// count them), and the type strings keep upstream's rendered multi-line union form.
// ---------------------------------------------------------------------------

/// `types.md:7-10` — the `### Root` summary line.
const ROOT_SUMMARY: &[Segment] = &[
    reference::text("Represents the checkbox itself.\nRenders a "),
    reference::code("span"),
    reference::text(" element and a hidden "),
    reference::code("input"),
    reference::text(" beside."),
];

/// `types.md:14-31` — the generated `**Root Props:**` table, in its own order, with each prop's
/// `#CheckboxRoot-<name>` anchor as upstream renders it.
const ROOT_PROPS: &[ReferenceProp] = &[
    ReferenceProp {
        name: "name",
        anchor: "CheckboxRoot-name",
        short_ty: "string",
        ty: "string | undefined",
        default_value: Some("undefined"),
        description: &[reference::text(
            "Identifies the field when a form is submitted.",
        )],
    },
    ReferenceProp {
        name: "defaultChecked",
        anchor: "CheckboxRoot-defaultChecked",
        short_ty: "boolean",
        ty: "boolean | undefined",
        default_value: Some("false"),
        description: &[
            reference::text(
                "Whether the checkbox is initially ticked. To render a controlled checkbox, use the ",
            ),
            reference::code("checked"),
            reference::text(" prop instead."),
        ],
    },
    ReferenceProp {
        name: "checked",
        anchor: "CheckboxRoot-checked",
        short_ty: "boolean",
        ty: "boolean | undefined",
        default_value: Some("undefined"),
        description: &[
            reference::text(
                "Whether the checkbox is currently ticked. To render an uncontrolled checkbox, use the ",
            ),
            reference::code("defaultChecked"),
            reference::text(" prop instead."),
        ],
    },
    ReferenceProp {
        name: "onCheckedChange",
        anchor: "CheckboxRoot-onCheckedChange",
        short_ty: "function",
        ty: "| ((\n    checked: boolean,\n    eventDetails: Checkbox.Root.ChangeEventDetails,\n  ) => void)\n| undefined",
        default_value: None,
        description: &[reference::text(
            "Event handler called when the checkbox is ticked or unticked.",
        )],
    },
    ReferenceProp {
        name: "indeterminate",
        anchor: "CheckboxRoot-indeterminate",
        short_ty: "boolean",
        ty: "boolean | undefined",
        default_value: Some("false"),
        description: &[reference::text(
            "Whether the checkbox is in a mixed state: neither ticked, nor unticked.",
        )],
    },
    ReferenceProp {
        name: "value",
        anchor: "CheckboxRoot-value",
        short_ty: "string",
        ty: "string | undefined",
        default_value: None,
        description: &[
            reference::text("The checkbox\u{2019}s value. Identifies it within a "),
            reference::link(
                "https://base-ui.com/react/components/checkbox-group",
                "Checkbox Group",
            ),
            reference::text(", falling back to "),
            reference::code("name"),
            reference::text(" when omitted.\nWhen submitting a form, a checked box submits "),
            reference::code("value"),
            reference::text("; with no "),
            reference::code("value"),
            reference::text(", it submits the native \u{201c}on\u{201d}."),
        ],
    },
    ReferenceProp {
        name: "form",
        anchor: "CheckboxRoot-form",
        short_ty: "string",
        ty: "string | undefined",
        default_value: None,
        description: &[reference::text(
            "Identifies the form that owns the hidden input.\nUseful when the checkbox is rendered outside the form.",
        )],
    },
    ReferenceProp {
        name: "nativeButton",
        anchor: "CheckboxRoot-nativeButton",
        short_ty: "boolean",
        ty: "boolean | undefined",
        default_value: Some("false"),
        description: &[
            reference::text("Whether the component renders a native "),
            reference::code("button"),
            reference::text(" element when replacing it via the "),
            reference::code("render"),
            reference::text(" prop.\nSet to "),
            reference::code("true"),
            reference::text(" if the rendered element is a native button."),
        ],
    },
    ReferenceProp {
        name: "parent",
        anchor: "CheckboxRoot-parent",
        short_ty: "boolean",
        ty: "boolean | undefined",
        default_value: Some("false"),
        description: &[
            reference::text(
                "Whether the checkbox controls a group of child checkboxes. Must be used in a ",
            ),
            reference::link(
                "https://base-ui.com/react/components/checkbox-group",
                "Checkbox Group",
            ),
            reference::text("."),
        ],
    },
    ReferenceProp {
        name: "uncheckedValue",
        anchor: "CheckboxRoot-uncheckedValue",
        short_ty: "string",
        ty: "string | undefined",
        default_value: None,
        description: &[reference::text(
            "The value submitted with the form when the checkbox is unchecked.\nBy default, unchecked checkboxes do not submit any value, matching native checkbox behavior.",
        )],
    },
    ReferenceProp {
        name: "disabled",
        anchor: "CheckboxRoot-disabled",
        short_ty: "boolean",
        ty: "boolean | undefined",
        default_value: Some("false"),
        description: &[reference::text(
            "Whether the component should ignore user interaction.",
        )],
    },
    ReferenceProp {
        name: "readOnly",
        anchor: "CheckboxRoot-readOnly",
        short_ty: "boolean",
        ty: "boolean | undefined",
        default_value: Some("false"),
        description: &[reference::text(
            "Whether the user should be unable to tick or untick the checkbox.",
        )],
    },
    ReferenceProp {
        name: "required",
        anchor: "CheckboxRoot-required",
        short_ty: "boolean",
        ty: "boolean | undefined",
        default_value: Some("false"),
        description: &[reference::text(
            "Whether the user must tick the checkbox before submitting a form.",
        )],
    },
    ReferenceProp {
        name: "inputRef",
        anchor: "CheckboxRoot-inputRef",
        short_ty: "React.Ref<HTMLInputElement>",
        ty: "React.Ref<HTMLInputElement> | undefined",
        default_value: None,
        description: &[
            reference::text("A ref to access the hidden "),
            reference::code("input"),
            reference::text(" element."),
        ],
    },
    ReferenceProp {
        name: "id",
        anchor: "CheckboxRoot-id",
        short_ty: "string",
        ty: "string | undefined",
        default_value: None,
        description: &[reference::text("The id of the input element.")],
    },
    ReferenceProp {
        name: "className",
        anchor: "CheckboxRoot-className",
        short_ty: "string | function",
        ty: "| string\n| ((state: Checkbox.Root.State) => string | undefined)\n| undefined",
        default_value: None,
        description: &[reference::text(
            "CSS class applied to the element, or a function that\nreturns a class based on the component\u{2019}s state.",
        )],
    },
    ReferenceProp {
        name: "style",
        anchor: "CheckboxRoot-style",
        short_ty: "React.CSSProperties | function",
        ty: "| React.CSSProperties\n| ((\n    state: Checkbox.Root.State,\n  ) => React.CSSProperties | undefined)\n| undefined",
        default_value: None,
        description: &[reference::text(
            "Style applied to the element, or a function that\nreturns a style object based on the component\u{2019}s state.",
        )],
    },
    ReferenceProp {
        name: "render",
        anchor: "CheckboxRoot-render",
        short_ty: "ReactElement | function",
        ty: "| ReactElement\n| ((\n    props: HTMLProps,\n    state: Checkbox.Root.State,\n  ) => ReactElement)\n| undefined",
        default_value: None,
        description: &[
            reference::text(
                "Allows you to replace the component\u{2019}s HTML element with a different tag, or compose it with another component. Accepts a ",
            ),
            reference::code("ReactElement"),
            reference::text(" or a function that returns the element to render."),
        ],
    },
];

/// `types.md:14-21` — the generated `**Root Data Attributes:**` table.
const ROOT_DATA_ATTRIBUTES: &[DataAttributeRow] = &[
    DataAttributeRow {
        name: "data-checked",
        description: &[reference::text("Present when the checkbox is checked.")],
    },
    DataAttributeRow {
        name: "data-unchecked",
        description: &[reference::text("Present when the checkbox is not checked.")],
    },
    DataAttributeRow {
        name: "data-disabled",
        description: &[reference::text("Present when the checkbox is disabled.")],
    },
    DataAttributeRow {
        name: "data-readonly",
        description: &[reference::text("Present when the checkbox is readonly.")],
    },
    DataAttributeRow {
        name: "data-required",
        description: &[reference::text("Present when the checkbox is required.")],
    },
    DataAttributeRow {
        name: "data-valid",
        description: &[reference::text(
            "Present when the checkbox is in a valid state (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-invalid",
        description: &[reference::text(
            "Present when the checkbox is in an invalid state (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-dirty",
        description: &[reference::text(
            "Present when the checkbox\u{2019}s value has changed (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-touched",
        description: &[reference::text(
            "Present when the checkbox has been touched (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-filled",
        description: &[reference::text(
            "Present when the checkbox is checked (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-focused",
        description: &[reference::text(
            "Present when the checkbox is focused (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-indeterminate",
        description: &[reference::text(
            "Present when the checkbox is in an indeterminate state.",
        )],
    },
];

/// `types.md:110-113` — the `### Indicator` summary line.
const INDICATOR_SUMMARY: &[Segment] = &[
    reference::text("Indicates whether the checkbox is ticked.\nRenders a "),
    reference::code("span"),
    reference::text(" element."),
];

/// `types.md:117-122` — the generated `**Indicator Props:**` table.
const INDICATOR_PROPS: &[ReferenceProp] = &[
    ReferenceProp {
        name: "className",
        anchor: "CheckboxIndicator-className",
        short_ty: "string | function",
        ty: "| string\n| ((\n    state: Checkbox.Indicator.State,\n  ) => string | undefined)\n| undefined",
        default_value: None,
        description: &[reference::text(
            "CSS class applied to the element, or a function that\nreturns a class based on the component\u{2019}s state.",
        )],
    },
    ReferenceProp {
        name: "style",
        anchor: "CheckboxIndicator-style",
        short_ty: "React.CSSProperties | function",
        ty: "| React.CSSProperties\n| ((\n    state: Checkbox.Indicator.State,\n  ) => React.CSSProperties | undefined)\n| undefined",
        default_value: None,
        description: &[reference::text(
            "Style applied to the element, or a function that\nreturns a style object based on the component\u{2019}s state.",
        )],
    },
    ReferenceProp {
        name: "keepMounted",
        anchor: "CheckboxIndicator-keepMounted",
        short_ty: "boolean",
        ty: "boolean | undefined",
        default_value: Some("false"),
        description: &[reference::text(
            "Whether to keep the element in the DOM when the checkbox is not checked.",
        )],
    },
    ReferenceProp {
        name: "render",
        anchor: "CheckboxIndicator-render",
        short_ty: "ReactElement | function",
        ty: "| ReactElement\n| ((\n    props: HTMLProps,\n    state: Checkbox.Indicator.State,\n  ) => ReactElement)\n| undefined",
        default_value: None,
        description: &[
            reference::text(
                "Allows you to replace the component\u{2019}s HTML element with a different tag, or compose it with another component. Accepts a ",
            ),
            reference::code("ReactElement"),
            reference::text(" or a function that returns the element to render."),
        ],
    },
];

/// `types.md:126-141` — the generated `**Indicator Data Attributes:**` table.
const INDICATOR_DATA_ATTRIBUTES: &[DataAttributeRow] = &[
    DataAttributeRow {
        name: "data-checked",
        description: &[reference::text("Present when the checkbox is checked.")],
    },
    DataAttributeRow {
        name: "data-unchecked",
        description: &[reference::text("Present when the checkbox is not checked.")],
    },
    DataAttributeRow {
        name: "data-disabled",
        description: &[reference::text("Present when the checkbox is disabled.")],
    },
    DataAttributeRow {
        name: "data-readonly",
        description: &[reference::text("Present when the checkbox is readonly.")],
    },
    DataAttributeRow {
        name: "data-required",
        description: &[reference::text("Present when the checkbox is required.")],
    },
    DataAttributeRow {
        name: "data-valid",
        description: &[reference::text(
            "Present when the checkbox is in a valid state (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-invalid",
        description: &[reference::text(
            "Present when the checkbox is in an invalid state (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-dirty",
        description: &[reference::text(
            "Present when the checkbox\u{2019}s value has changed (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-touched",
        description: &[reference::text(
            "Present when the checkbox has been touched (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-filled",
        description: &[reference::text(
            "Present when the checkbox is checked (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-focused",
        description: &[reference::text(
            "Present when the checkbox is focused (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-indeterminate",
        description: &[reference::text(
            "Present when the checkbox is in an indeterminate state.",
        )],
    },
    DataAttributeRow {
        name: "data-starting-style",
        description: &[reference::text(
            "Present when the checkbox indicator begins animating in.",
        )],
    },
    DataAttributeRow {
        name: "data-ending-style",
        description: &[reference::text(
            "Present when the checkbox indicator is animating out.",
        )],
    },
];

/// One generated part section: its summary line, its props section and its data-attributes table —
/// the order `types.md` documents them in and upstream renders them.
fn api_part(
    summary: &'static [Segment],
    props: &'static [ReferenceProp],
    data_attributes: &'static [DataAttributeRow],
    table_id: &'static str,
) -> impl IntoView {
    view! {
        {reference::part_summary(summary)}
        {reference::props_section(props, table_id)}
        {reference::data_attributes_table(data_attributes)}
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
            {code_block(Lang::Rust, "Anatomy", ANATOMY_SNIPPET)}

            <h2>"Examples"</h2>

            <h3>"Labeling a checkbox"</h3>
            <p>"An enclosing `<label>` is the simplest labeling pattern:"</p>
            {code_block(Lang::Rust, "Wrapping a label around a checkbox", LABELING_SNIPPET)}

            <h3>"Rendering as a native button"</h3>
            <p>
                "By default, `<Checkbox.Root>` renders a `<span>` element to support enclosing "
                "labels. Prefer rendering the checkbox as a native button when using sibling "
                "labels (`htmlFor`/`id`)."
            </p>
            {code_block(Lang::Rust, "Sibling label pattern with a native button", NATIVE_BUTTON_SNIPPET)}
            <p>
                "The port honors the `render` prop's element form, which replaces the visible "
                "element with a native `<button>`:"
            </p>
            {code_block(Lang::Rust, "Render callback", RENDER_CALLBACK_SNIPPET)}
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
            {code_block(Lang::Rust, "Using Checkbox in a form", FORM_SNIPPET)}

            <h2>"API reference"</h2>
            <h3>"Root"</h3>
            {api_part(
                ROOT_SUMMARY,
                ROOT_PROPS,
                ROOT_DATA_ATTRIBUTES,
                "checkbox-root-props-table",
            )}
            <h3>"Indicator"</h3>
            {api_part(
                INDICATOR_SUMMARY,
                INDICATOR_PROPS,
                INDICATOR_DATA_ATTRIBUTES,
                "checkbox-indicator-props-table",
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
///     in `crate::snippet_language` so one copy of the rules serves every mirrored page), so the
///     numbers the probe would report are asserted in CI: `{total: 5, leptos: 5, react: 0}`;
///   * the `_shape` functions below compile the composition each snippet teaches, so a snippet
///     cannot name a prop, field or path the port does not actually have — the failure mode that
///     transcription made invisible. They are never called (the page's real compositions are
///     exercised by `render_test.rs`); the compiler is the assertion.
#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use crate::snippet_language::{looks_leptos, looks_react};
    use leptos_ui::field_parts::{FieldLabelViewProps, field_label_view};
    use leptos_ui::field_root::{FieldRootViewProps, field_root_view};
    use leptos_ui_internals::use_render_element::{RenderElementProps, RenderProp};

    /// The upstream Anatomy block (`page.mdx:19-27`), kept here as the classifier's positive
    /// control: if `looks_react` ever stops recognising upstream's source, the assertions below
    /// would pass vacuously, and this test would say so instead.
    const UPSTREAM_ANATOMY: &str = "import { Checkbox } from '@base-ui/react/checkbox';\n\n<Checkbox.Root>\n  <Checkbox.Indicator />\n</Checkbox.Root>;";

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

/// Browser-free drift guard for the generated `## API reference` content this page carries as data.
///
/// Why it exists: the two generated tables are the only part of the page whose content comes from a
/// file outside this repo's Rust test path — `docs/src/app/(docs)/react/components/checkbox/types.md`,
/// which is regenerated by `pnpm docs:api` — and every structural check passes for any content
/// (`render_test.rs` asserts the shape of the tables: two tables, 12 and 14 rows, 22 prop rows;
/// none of that notices a mistyped prop name, a dropped row or a reordered list). The row sets and
/// their order are asserted here instead, so the hand transcription is checked without a browser
/// and a future regeneration of the documented surface fails loudly rather than silently.
#[cfg(test)]
mod reference_content_guard {
    use super::*;

    /// `types.md:14-31` — the generated `**Root Props:**` rows, in order.
    const ROOT_PROP_NAMES: [&str; 18] = [
        "name",
        "defaultChecked",
        "checked",
        "onCheckedChange",
        "indeterminate",
        "value",
        "form",
        "nativeButton",
        "parent",
        "uncheckedValue",
        "disabled",
        "readOnly",
        "required",
        "inputRef",
        "id",
        "className",
        "style",
        "render",
    ];

    /// `types.md:117-122` — the generated `**Indicator Props:**` rows, in order.
    const INDICATOR_PROP_NAMES: [&str; 4] = ["className", "style", "keepMounted", "render"];

    /// `types.md:22-33` — the generated `**Root Data Attributes:**` rows, in order.
    const ROOT_DATA_ATTRIBUTE_NAMES: [&str; 12] = [
        "data-checked",
        "data-unchecked",
        "data-disabled",
        "data-readonly",
        "data-required",
        "data-valid",
        "data-invalid",
        "data-dirty",
        "data-touched",
        "data-filled",
        "data-focused",
        "data-indeterminate",
    ];

    /// `types.md:126-141` — the generated `**Indicator Data Attributes:**` rows: the Root set plus
    /// the two transition-status attributes.
    const INDICATOR_DATA_ATTRIBUTE_NAMES: [&str; 14] = [
        "data-checked",
        "data-unchecked",
        "data-disabled",
        "data-readonly",
        "data-required",
        "data-valid",
        "data-invalid",
        "data-dirty",
        "data-touched",
        "data-filled",
        "data-focused",
        "data-indeterminate",
        "data-starting-style",
        "data-ending-style",
    ];

    #[test]
    fn the_transcribed_rows_match_the_generated_types_content() {
        assert_eq!(
            ROOT_PROPS.iter().map(|prop| prop.name).collect::<Vec<_>>(),
            ROOT_PROP_NAMES.to_vec(),
            "the Root prop rows drifted from types.md"
        );
        assert_eq!(
            INDICATOR_PROPS
                .iter()
                .map(|prop| prop.name)
                .collect::<Vec<_>>(),
            INDICATOR_PROP_NAMES.to_vec(),
            "the Indicator prop rows drifted from types.md"
        );
        assert_eq!(
            ROOT_DATA_ATTRIBUTES
                .iter()
                .map(|row| row.name)
                .collect::<Vec<_>>(),
            ROOT_DATA_ATTRIBUTE_NAMES.to_vec(),
            "the Root data-attribute rows drifted from types.md"
        );
        assert_eq!(
            INDICATOR_DATA_ATTRIBUTES
                .iter()
                .map(|row| row.name)
                .collect::<Vec<_>>(),
            INDICATOR_DATA_ATTRIBUTE_NAMES.to_vec(),
            "the Indicator data-attribute rows drifted from types.md"
        );
    }

    /// Every prop row must carry a type and a description, and be addressed by upstream's own
    /// anchor (`CheckboxRoot-<name>`), which is also what its `Name` cell links to. A missing
    /// anchor or a duplicate would silently break the section's in-page links — and upstream's
    /// per-prop links are a recall term the fidelity gate scores.
    #[test]
    fn every_prop_row_is_addressed_and_carries_its_generated_content() {
        for (props, prefix) in [
            (ROOT_PROPS, "CheckboxRoot"),
            (INDICATOR_PROPS, "CheckboxIndicator"),
        ] {
            let mut anchors = std::collections::BTreeSet::new();
            for prop in props {
                assert_eq!(
                    prop.anchor,
                    format!("{prefix}-{}", prop.name),
                    "prop '{}' does not carry its upstream anchor",
                    prop.name
                );
                assert!(
                    anchors.insert(prop.anchor),
                    "duplicate prop anchor '{}'",
                    prop.anchor
                );
                assert!(!prop.ty.is_empty(), "prop '{}' has no type", prop.name);
                assert!(
                    !prop.description.is_empty(),
                    "prop '{}' has no description",
                    prop.name
                );
            }
        }
    }

    /// Upstream renders a `Default` item only where the generated table documents one; the port
    /// renders `None` as no row at all, so the two sets have to stay complementary.
    #[test]
    fn only_props_with_a_documented_default_carry_a_default_row() {
        assert_eq!(
            ROOT_PROPS
                .iter()
                .filter(|prop| prop.default_value.is_some())
                .map(|prop| prop.name)
                .collect::<Vec<_>>(),
            vec![
                "name",
                "defaultChecked",
                "checked",
                "indeterminate",
                "nativeButton",
                "parent",
                "disabled",
                "readOnly",
                "required",
            ],
            "the Root props carrying a documented default drifted from types.md"
        );
        assert_eq!(
            INDICATOR_PROPS
                .iter()
                .filter(|prop| prop.default_value.is_some())
                .map(|prop| prop.name)
                .collect::<Vec<_>>(),
            vec!["keepMounted"],
            "the Indicator props carrying a documented default drifted from types.md"
        );
    }

    /// Upstream's SHORT summary type per row, in the generated tables' order — the label each
    /// `<details>` summary shows beside the prop name (measured off upstream's own render of this
    /// route at 1280px; the generated markdown's `Type` column carries the full union instead, so
    /// this half of the row has no file in the repo to be diffed against).
    #[test]
    fn the_rows_short_summary_types_match_upstreams_render() {
        assert_eq!(
            ROOT_PROPS
                .iter()
                .map(|prop| prop.short_ty)
                .collect::<Vec<_>>(),
            vec![
                "string",
                "boolean",
                "boolean",
                "function",
                "boolean",
                "string",
                "string",
                "boolean",
                "boolean",
                "string",
                "boolean",
                "boolean",
                "boolean",
                "React.Ref<HTMLInputElement>",
                "string",
                "string | function",
                "React.CSSProperties | function",
                "ReactElement | function",
            ],
            "the Root rows' short summary types drifted from upstream's render"
        );
        assert_eq!(
            INDICATOR_PROPS
                .iter()
                .map(|prop| prop.short_ty)
                .collect::<Vec<_>>(),
            vec![
                "string | function",
                "React.CSSProperties | function",
                "boolean",
                "ReactElement | function",
            ],
            "the Indicator rows' short summary types drifted from upstream's render"
        );
    }
}
