//! The docs page for `Input`, mirroring
//! `docs/src/app/(docs)/react/components/input/page.mdx`
//! (`specs/docs-content/input/page.md`) — the `docs-content: components/input` TODO item.
//!
//! Page structure per the spec's "Page structure" section: `# Input` h1, the `<Subtitle>`
//! ("A native input element that automatically works with Field."), the hero demo before the
//! first heading, `## Usage guidelines`, `## Anatomy` with the single fenced snippet, and
//! `## API reference` over the generated `TypesInput` reference (echoed as static prose per the
//! meter/separator/toggle page precedent — the port has no docs generator, so the tables'
//! documented props, data attributes and state type are rendered as text, never fabricated as
//! executable machinery).
//!
//! Page furniture mirrored in module docs (the accordion/meter page precedent): the
//! `<Meta name="description">` content — "A high-quality, unstyled React input component." — and
//! the trailing `export const metadata` SEO keywords block (15 keywords: 'React Input',
//! 'Input Component', 'Input Field', 'Form Input Field', 'Text Field', 'Text Input', 'Text Box',
//! 'Form Input', 'Textarea Alternative', 'Controlled Input', 'Form Element', 'Accessible Input',
//! 'Headless React Components', 'Unstyled Input', 'Base UI').
//!
//! The live demo is the upstream Tailwind hero
//! (`docs/src/app/(docs)/react/components/input/demos/hero/tailwind/index.tsx`,
//! the single `specs/docs-content/input/demos.json` entry) ported onto the REAL
//! `leptos_ui::Input` component: a wrapping `<label>` ("Name") around one `Input` carrying the
//! demo's two exercised props — `placeholder` (`propsExercised.Input`) and `className`, which is
//! the port's `class` — with every upstream `className` string carried verbatim so the DOM the
//! Leptos port produces matches the React demo's element-for-element. The demo is fully
//! uncontrolled with no state and no handlers (demos.json: `stateManaged: "none"` — "the input is
//! fully uncontrolled with no value/defaultValue, no useState, and no event handlers; the browser
//! owns the value"), so the port adds no signals and no effects: `Input` is upstream's one-line
//! delegation to `Field.Control` (`crates/leptos-ui/src/input.rs:121-161`) and the browser owns
//! the value in the port exactly as it does in React.

use crate::code_block::{Lang, code_block};
use crate::reference::{
    AdditionalType, DataAttributeRow, ReferenceProp, Segment, additional_types, code, data_attributes_table,
    link, part_section_heading, part_summary, props_section, segments, text,
};
use leptos::prelude::*;

use leptos_ui::Input;

/// The upstream `<label>` className (`hero/tailwind/index.tsx:5`).
const LABEL_CLASS: &str =
    "flex flex-col items-start gap-1 text-sm font-bold text-neutral-950 dark:text-white";

/// The upstream `Input` className (`hero/tailwind/index.tsx:7-10`) — the demo's
/// `className` exercise, which is this port's `class`.
///
/// `pub(crate)` so the page's render test asserts the attribute against this same constant
/// (`render_test.rs`), rather than a second copy of a 400-character literal that could drift from
/// the demo unnoticed; the demo-styles guard is what keeps every token here defined.
pub(crate) const DEMO_INPUT_CLASS: &str = "h-8 w-40 border border-neutral-950 dark:border-white bg-white dark:bg-neutral-950 px-2 text-sm any-pointer-coarse:text-base font-normal text-neutral-950 dark:text-white placeholder:text-neutral-500 dark:placeholder:text-neutral-400 focus:outline-2 focus:-outline-offset-1 focus:outline-neutral-950 dark:focus:outline-white";

/// The upstream `placeholder` value (`hero/tailwind/index.tsx:8`) — the demo's other exercised
/// prop, which rides this port's `element_attributes` rest bag (upstream's `...props` spread,
/// `packages/react/src/input/Input.tsx:16`).
const DEMO_PLACEHOLDER: &str = "e.g. Colm Tuite";

/// The `## Anatomy` snippet (`specs/docs-content/input/page.md`, mirroring
/// `docs/src/app/(docs)/react/components/input/page.mdx:21-25`): upstream's
/// `import { Input } from '@base-ui/react/input'` followed by a bare `<Input />;`.
///
/// Translated to THIS port (`specs/docs-content/CONTRACT.md` requirement 1 — a snippet embedded in
/// a mirrored page must show the port's own API, never upstream's import line, so this item's
/// `check-react-mentions.mjs --source` count is zero on this page). `Input` is a single-part
/// component with no subcomponents upstream and none here
/// (`specs/library/input/behavior.md` § Public API surface; `crates/leptos-ui/src/input.rs:178-183`:
/// "there is no `Input::Root` to add"), so there is no `Input::Part` tree to teach and the port's
/// `#[component] Input` is used in `view!` markup — the same call site upstream teaches, spelled
/// the way every other part in this crate is spelled.
const ANATOMY_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::Input;

view! { <Input /> }"#;

// ---------------------------------------------------------------------------
// The generated `## API reference` content
// ---------------------------------------------------------------------------
//
// Source: `docs/src/app/(docs)/react/components/input/types.md` (generated by `pnpm docs:api` from
// the component's type definitions), which is what upstream's page renders through `<TypesInput />`
// (`docs/src/app/(docs)/react/components/input/page.mdx:29-31`). The port has no docs generator, so
// the generated content is carried here as data and rendered through `crate::reference`, whose
// element shape and class names are upstream's own (the checkbox/button/accordion page precedent) —
// never fabricated as executable machinery.
//
// The rows keep upstream's NAMES and ORDER. The type columns state the Rust types THIS port accepts,
// which `specs/docs-content/CONTRACT.md` requirement 6 requires of every API table ("a type column
// that says `ReactElement` must say the Rust type the port accepts"), and the one row whose
// behaviour the port does not have says so in its own description and names the ledger item that
// owns the gap.

/// `types.md:7-9` — the part's generated summary line. Its `Field` link points at this port's own
/// Field page rather than upstream's site URL (the checkbox page's `summary` precedent).
const INPUT_SUMMARY: &[Segment] = &[
    text("A native input element that automatically works with "),
    link("/react/components/field", "Field"),
    text(". Renders an "),
    code("<input>"),
    text(" element."),
];

/// `types.md:11-21` — the generated `**Input Props:**` rows, in upstream's order. `short_ty` is what
/// the row's summary shows beside the name and `ty` is the full form the `Type` item carries; both
/// are this port's Rust types (`crates/leptos-ui/src/input.rs`'s `InputViewProps`, and
/// `crates/leptos-ui/src/field/field_control.rs:75-92`'s `ValueChangeEventDetails` for the change
/// handler's second argument).
const INPUT_PROPS: &[ReferenceProp] = &[
    ReferenceProp {
        name: "defaultValue",
        anchor: "Input-defaultValue",
        short_ty: "Option<String>",
        ty: "Option<String>",
        default_value: None,
        description: &[text("The default value of the input. Use when uncontrolled.")],
    },
    ReferenceProp {
        name: "value",
        anchor: "Input-value",
        short_ty: "Option<String>",
        ty: "Option<String>",
        default_value: None,
        description: &[text("The value of the input. Use when controlled.")],
    },
    ReferenceProp {
        name: "onValueChange",
        anchor: "Input-onValueChange",
        short_ty: "Option<InputChangeHandler>",
        ty: "Option<Rc<dyn Fn(String, &ValueChangeEventDetails)>>",
        default_value: None,
        description: &[
            text("Callback fired when the "),
            code("value"),
            text(" changes. Use when controlled."),
        ],
    },
    ReferenceProp {
        name: "className",
        anchor: "Input-className",
        short_ty: "Option<String>",
        ty: "Option<String>",
        default_value: None,
        description: &[text(
            "CSS class applied to the element, or a function that returns a class based on the \
             component's state.",
        )],
    },
    ReferenceProp {
        name: "style",
        anchor: "Input-style",
        short_ty: "Vec<(String, String)>",
        ty: "Vec<(String, String)>",
        default_value: None,
        description: &[text(
            "Style applied to the element, or a function that returns a style object based on the \
             component's state.",
        )],
    },
    ReferenceProp {
        name: "render",
        anchor: "Input-render",
        short_ty: "\u{2014}",
        ty: "not exposed by this port",
        default_value: None,
        // The one row whose obligation this port cannot meet, so it says so instead of transcribing
        // upstream's rationale for behaviour the port does not have (`CONTRACT.md` requirement 1's
        // "never transcribe upstream's rationale"; the `snippet_language.rs` header's rule).
        description: &[text(
            "Upstream replaces the component's HTML element with a different tag, or composes it \
             with another component. This port does not expose ",
        ), code("render"), text(
            ": the delegation target builds a fixed ",
        ), code("<input>"), text(
            " so a component-level substitution could not be kept. Upstream's arbitrary DOM props \
             are this port's ",
        ), code("element_attributes"), text(" rest bag; the gap is the ledger item "), code(
            "library: the view paths drop render's element form",
        ), text(".")],
    },
];

/// `types.md:23-33` — the generated `**Input Data Attributes:**` rows, in upstream's order.
/// Upstream's own spelling is kept (`Field.Root`, not the port's `Field::Root`), exactly as the
/// checkbox page's generated table keeps it — these descriptions are transcribed, not re-authored.
const INPUT_DATA_ATTRIBUTES: &[DataAttributeRow] = &[
    DataAttributeRow {
        name: "data-disabled",
        description: &[text("Present when the input is disabled.")],
    },
    DataAttributeRow {
        name: "data-valid",
        description: &[text(
            "Present when the input is in a valid state (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-invalid",
        description: &[text(
            "Present when the input is in an invalid state (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-dirty",
        description: &[text(
            "Present when the input's value has changed (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-touched",
        description: &[text(
            "Present when the input has been touched (when wrapped in Field.Root).",
        )],
    },
    DataAttributeRow {
        name: "data-filled",
        description: &[text("Present when the input is filled (when wrapped in Field.Root).")],
    },
    DataAttributeRow {
        name: "data-focused",
        description: &[text("Present when the input is focused (when wrapped in Field.Root).")],
    },
];

/// `types.md`'s `Additional Types` panels, in upstream's order. The `.Props` panel carries
/// upstream's re-export line; the type definitions' bodies are generated TypeScript blocks this
/// loop assigns to `docs-chrome: code blocks` (the `reference.rs` module docs), so the panels
/// carry their headings and this page states the state/event vocabulary in prose below.
const INPUT_ADDITIONAL_TYPES: &[AdditionalType] = &[
    AdditionalType {
        name: "Input.Props",
        slug: "input.props",
        re_export_of: Some(("Input", "InputProps")),
    },
    AdditionalType {
        name: "Input.State",
        slug: "input.state",
        re_export_of: None,
    },
    AdditionalType {
        name: "Input.ChangeEventReason",
        slug: "input.changeeventreason",
        re_export_of: None,
    },
    AdditionalType {
        name: "Input.ChangeEventDetails",
        slug: "input.changeeventdetails",
        re_export_of: None,
    },
];

/// The API-reference intro sentence. Upstream's page has no such sentence (its reference is a
/// generated component); this states the one mapping a reader needs between the transcribed prop
/// names and the port's spelling, which is what keeps the rows above verbatim.
const API_SPELLING_NOTE: &[Segment] = &[
    text("The rows below keep upstream's prop names; this port spells them in Rust — "),
    code("default_value"),
    text(", "),
    code("on_value_change"),
    text(", and "),
    code("class"),
    text("/"),
    code("style"),
    text(" as the component's own declarations rather than through the "),
    code("element_attributes"),
    text(" rest bag."),
];

/// `types.md`'s `Input.State` definition, stated with this port's type and member signals
/// (`crates/leptos-ui/src/field/context.rs:37-58`).
const STATE_PROSE: &[Segment] = &[
    text("The state a state-aware "),
    code("className"),
    text(" or "),
    code("style"),
    text(" receives upstream — in this port the field control's own state, "),
    code("FieldStateValue"),
    text(", with the same members: "),
    code("disabled"),
    text(" (whether the component should ignore user interaction), "),
    code("touched"),
    text(" (whether the field has been touched), "),
    code("dirty"),
    text(" (whether the field value has changed from its initial value), "),
    code("valid"),
    text(" (whether the field is valid, the port's tri-state where the unvalidated state is "),
    code("None"),
    text("), "),
    code("filled"),
    text(" (whether the field has a value) and "),
    code("focused"),
    text(" (whether the field is focused). Each is a Leptos "),
    code("Signal"),
    text(", so a state-aware writer tracks them instead of being re-run by hand."),
];

/// `types.md`'s `Input.ChangeEventReason` definition.
const REASON_PROSE: &[Segment] = &[
    code("'none'"),
    text(" — an input's own change carries no Base UI reason; the port's change details carry "),
    text("the same single reason."),
];

/// `types.md`'s `Input.ChangeEventDetails` definition, stated with the port's own type
/// (`crates/leptos-ui/src/field/field_control.rs:75-92`).
const DETAILS_PROSE: &[Segment] = &[
    text("The second argument of "),
    code("onValueChange"),
    text(": the reason for the event ("),
    code("none"),
    text("), the native event associated with the change ("),
    code("event"),
    text("), "),
    code("cancel()"),
    text(" to stop Base UI from handling the event, with "),
    code("is_canceled()"),
    text(" reporting it, and "),
    code("allow_propagation()"),
    text(" for the cases where Base UI would stop propagation. The port's type is "),
    code("ValueChangeEventDetails"),
    text("."),
];

/// `types.md`'s `## Canonical Types` mapping, stated with this port's type names.
const CANONICAL_PROSE: &[Segment] = &[
    text("Upstream's canonical names map onto this port's types: "),
    code("Input.Props"),
    text(" is "),
    code("InputViewProps"),
    text(", "),
    code("Input.State"),
    text(" is the field control's "),
    code("FieldStateValue"),
    text(", "),
    code("Input.ChangeEventReason"),
    text(" is the single-reason change details, and "),
    code("Input.ChangeEventDetails"),
    text(" is "),
    code("ValueChangeEventDetails"),
    text("."),
];

/// The upstream hero demo (`hero/tailwind/index.tsx:3-12`) on the real port: a
/// `flex flex-col items-start gap-1 text-sm font-bold` `<label>` whose text is "Name", wrapping one
/// `Input` with the demo's `placeholder` and `className`.
///
/// The `placeholder` rides `element_attributes` because that is the port's spelling of upstream's
/// `...props` rest (`packages/react/src/input/Input.tsx:16`; the same bag the conformance suite
/// spreads `placeholder` through,
/// `packages/react/test/conformanceTests/propForwarding.tsx:23-36`). The bag is applied by the
/// control's own mount effect (`crates/leptos-ui/src/field/field_control.rs:584-601`), i.e. after
/// the first render turn — a browser assertion on `placeholder` has to await a turn, which the
/// page's render test does rather than asserting the seed.
///
/// Static per demos.json (`stateManaged: "none"`): the browser owns the value, so the demo keeps no
/// signal and no effect. The label association is native (`<label>` wrapping the input), which is
/// the access-name route the upstream page's "Usage guidelines" section teaches.
pub fn input_hero_demo() -> impl IntoView {
    view! {
        <label class=LABEL_CLASS>
            "Name"
            <Input
                class=DEMO_INPUT_CLASS.to_string()
                element_attributes=vec![
                    ("placeholder".to_string(), DEMO_PLACEHOLDER.to_string())
                ]
            />
        </label>
    }
}

/// The `docs/src/app/(docs)/react/components/input/page.mdx` page.
#[component]
pub fn InputPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Input"</h1>
            <p class="subtitle">
                "A native input element that automatically works with "
                <a href="/react/components/field">"Field"</a>
                "."
            </p>

            <div class="docs-demo" data-demo="hero">{input_hero_demo()}</div>

            <h2>"Usage guidelines"</h2>
            <ul>
                <li>
                    <strong>"Form controls must have an accessible name"</strong>
                    ": It can be created using a "<code>"<label>"</code>" element or the "<code>"Field"</code>
                    " component. See the "<a href="/react/handbook/forms">"forms guide"</a>"."
                </li>
            </ul>

            <h2>"Anatomy"</h2>
            <p>"Import the component and use it as a single part:"</p>
            {code_block(Lang::Rust, "Anatomy", ANATOMY_SNIPPET)}

            <h2>"API reference"</h2>
            <p class="MdP">{segments(API_SPELLING_NOTE)}</p>

            {part_section_heading("Input")}
            {part_summary(INPUT_SUMMARY)}

            <p class="MdP">"Input Props:"</p>
            {props_section(INPUT_PROPS, "input-props-table")}

            <p class="MdP">"Input Data Attributes:"</p>
            {data_attributes_table(INPUT_DATA_ATTRIBUTES)}

            {additional_types(INPUT_ADDITIONAL_TYPES)}

            <h3>"Input.State"</h3>
            <p class="MdP">{segments(STATE_PROSE)}</p>

            <h3>"Input.ChangeEventReason"</h3>
            <p class="MdP">{segments(REASON_PROSE)}</p>

            <h3>"Input.ChangeEventDetails"</h3>
            <p class="MdP">{segments(DETAILS_PROSE)}</p>

            <h3>"Canonical Types"</h3>
            <p class="MdP">{segments(CANONICAL_PROSE)}</p>
        </article>
    }
}

#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use crate::snippet_language::{SnippetLanguage, classify};
    use leptos_ui::Input;

    /// Upstream's Anatomy block (`docs/src/app/(docs)/react/components/input/page.mdx:21-25`),
    /// kept as the classifier's positive control so the assertions below cannot pass vacuously if
    /// `looks_react` ever stops recognising upstream's JSX shape. The package specifier upstream's
    /// import line carries is deliberately left out: this is page source, not reader-facing, and the
    /// sibling pages' controls omit it for the same reason (the `field_page.rs:224-227` precedent).
    const UPSTREAM_ANATOMY_SHAPE: &str = "<Input />;";

    #[test]
    fn the_classifier_recognises_upstream_source() {
        assert_eq!(
            classify(UPSTREAM_ANATOMY_SHAPE),
            SnippetLanguage::React,
            "the classifier no longer recognises upstream's source shape — the assertion below would \
             be vacuous"
        );
    }

    /// Every code block this page embeds, in document order. The page carries exactly one fence —
    /// the Anatomy listing; the demo's markup is Rust, so no other `<pre>` is rendered (upstream's
    /// page has no other inline fence either: its hero demo is an imported component, and its API
    /// reference is a generated table component,
    /// `specs/docs-content/input/page.md` § "Code snippets embedded directly in the .mdx").
    fn page_snippets() -> [(&'static str, &'static str); 1] {
        [("Anatomy", ANATOMY_SNIPPET)]
    }

    /// The page-level number the probe reads: `{total: 1, leptos: 1, react: 0, other: 0}` — the same
    /// triple `visual-gap-report.mjs`'s in-browser probe reports for `react/components/input`.
    #[test]
    fn the_pages_snippets_all_teach_the_port() {
        let languages: Vec<(&str, SnippetLanguage)> = page_snippets()
            .iter()
            .map(|(name, text)| (*name, classify(text)))
            .collect();
        assert_eq!(
            languages,
            vec![("Anatomy", SnippetLanguage::Leptos)],
            "the probe must read {{total: 1, leptos: 1, react: 0, other: 0}} for this page, in \
             document order"
        );
    }

    /// The snippet's own composition, compiled. Never called: the compiler checks the crate path,
    /// the component and the `view!` shape the page teaches, so a snippet naming an API the port
    /// does not have fails the build instead of shipping (the checkbox page's guard caught three
    /// such snippets — `snippet_language.rs`'s header).
    #[allow(dead_code)]
    fn anatomy_snippet_shape() {
        let _ = view! { <Input /> };
    }
}

/// The generated API-reference rows' transcription guard — the button page's `BUTTON_SHORT_TYPES`
/// precedent. The row NAMES and their ORDER come from
/// `docs/src/app/(docs)/react/components/input/types.md` (upstream's generated reference), so a
/// dropped or re-ordered row fails here instead of shipping; the type columns are this port's Rust
/// types (`specs/docs-content/CONTRACT.md` requirement 6 — a type column states the type the port
/// accepts, never upstream's React label), pinned the same way so re-spelling a row is a visible
/// decision rather than a silent drift.
#[cfg(test)]
mod reference_rows_guard {
    use super::*;

    /// `types.md:11-21` — the generated `**Input Props:**` rows, in upstream's order.
    const PROP_NAMES: [&str; 6] = [
        "defaultValue",
        "value",
        "onValueChange",
        "className",
        "style",
        "render",
    ];

    /// The type each row's summary shows. The first five are the port's Rust types; `render` is the
    /// one prop this port does not expose, and its cell says so rather than borrowing a type the
    /// component would not accept.
    const SHORT_TYPES: [&str; 6] = [
        "Option<String>",
        "Option<String>",
        "Option<InputChangeHandler>",
        "Option<String>",
        "Vec<(String, String)>",
        "\u{2014}",
    ];

    /// `types.md:23-33` — the generated `**Input Data Attributes:**` rows, in upstream's order.
    const DATA_ATTRIBUTE_NAMES: [&str; 7] = [
        "data-disabled",
        "data-valid",
        "data-invalid",
        "data-dirty",
        "data-touched",
        "data-filled",
        "data-focused",
    ];

    /// `types.md` — the generated `Additional Types` panels, in upstream's order.
    const ADDITIONAL_TYPE_NAMES: [&str; 4] = [
        "Input.Props",
        "Input.State",
        "Input.ChangeEventReason",
        "Input.ChangeEventDetails",
    ];

    #[test]
    fn the_transcribed_rows_match_the_generated_types_content() {
        assert_eq!(
            INPUT_PROPS.iter().map(|prop| prop.name).collect::<Vec<_>>(),
            PROP_NAMES.to_vec(),
            "the Input prop rows drifted from types.md"
        );
        assert_eq!(
            INPUT_PROPS
                .iter()
                .map(|prop| prop.short_ty)
                .collect::<Vec<_>>(),
            SHORT_TYPES.to_vec(),
            "the Input rows' short summary types drifted from the port's own types"
        );
        assert_eq!(
            INPUT_DATA_ATTRIBUTES
                .iter()
                .map(|row| row.name)
                .collect::<Vec<_>>(),
            DATA_ATTRIBUTE_NAMES.to_vec(),
            "the Input data-attribute rows drifted from types.md"
        );
        assert_eq!(
            INPUT_ADDITIONAL_TYPES
                .iter()
                .map(|additional| additional.name)
                .collect::<Vec<_>>(),
            ADDITIONAL_TYPE_NAMES.to_vec(),
            "the Input additional-type panels drifted from types.md"
        );
    }

    /// Every row must be addressed by upstream's own anchor (`Input-<name>`) — which is also what
    /// its `Name` cell links to — and carry the content the reference is for. A missing or duplicate
    /// anchor would silently break the section's in-page links, and upstream's per-prop links are a
    /// recall term the fidelity gate scores (the button page's guard, same reason).
    #[test]
    fn every_prop_row_is_addressed_and_carries_its_generated_content() {
        let mut anchors = std::collections::BTreeSet::new();
        for prop in INPUT_PROPS {
            assert_eq!(
                prop.anchor,
                format!("Input-{}", prop.name),
                "the row for {} does not carry upstream's anchor spelling",
                prop.name
            );
            assert!(
                anchors.insert(prop.anchor),
                "duplicate anchor {} — the page's in-page links would resolve arbitrarily",
                prop.anchor
            );
            assert!(
                !prop.description.is_empty(),
                "the row for {} carries no description",
                prop.name
            );
            assert!(
                !prop.ty.is_empty(),
                "the row for {} carries no type",
                prop.name
            );
            assert!(
                prop.default_value.is_none(),
                "types.md documents no default for {}, so the row must render the em dash",
                prop.name
            );
        }
        assert_eq!(anchors.len(), INPUT_PROPS.len());
    }

    /// No row may hand the reader a framework type: the whole point of carrying the generated table
    /// as data is that this port states its own types (`CONTRACT.md` requirement 6).
    #[test]
    fn no_row_states_a_react_type() {
        for prop in INPUT_PROPS {
            for text in [prop.short_ty, prop.ty] {
                assert!(
                    !text.contains("React"),
                    "the row for {} states a React type ({text}) — the port's own type belongs there",
                    prop.name
                );
            }
        }
        for row in INPUT_DATA_ATTRIBUTES {
            assert_eq!(
                row.description.iter().filter(|segment| matches!(segment, Segment::Text(_))).count()
                    > 0,
                true,
                "the data attribute {} carries prose",
                row.name
            );
        }
    }
}
