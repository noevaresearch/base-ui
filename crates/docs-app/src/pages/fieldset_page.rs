//! The docs page for `Fieldset`, mirroring
//! `docs/src/app/(docs)/react/components/fieldset/page.mdx`
//! (`specs/docs-content/fieldset/page.md`) — the `docs-content: components/fieldset`
//! TODO item, and the Phase D half of the `library: fieldset` pair.
//!
//! Page structure per the spec's "Page structure" section: `# Fieldset` h1
//! (`page.mdx:1`), `<Subtitle>` ("A native fieldset element with an easily
//! stylable legend.", `:3`), the hero demo **before the first heading**
//! (`:10-12`), `## Anatomy` (`:14`) with its single fenced snippet
//! (`:18-24`), then `## API reference` (`:26`) over the two generated
//! `TypesFieldset` reference tables — Root (`:30`) and Legend (`:34`) — echoed
//! as static prose per the accordion/button/field/meter page precedent: the
//! port has no docs generator, so the tables' documented props and State types
//! (`docs/src/app/(docs)/react/components/fieldset/types.md`) are rendered as
//! text, never fabricated as executable machinery.
//!
//! Page furniture mirrored in module docs (the field/accordion page
//! precedent): the `<Meta name="description">` content — "A high-quality,
//! unstyled React fieldset component with an easily stylable legend."
//! (`page.mdx:5-8`) — and the trailing `export const metadata` SEO keywords
//! block (12 keywords, `page.mdx:38-53`: 'React Fieldset', 'Fieldset
//! Component', 'Form Field Grouping', 'Form Fieldset', 'Form Group', 'Input
//! Grouping', 'Custom Legend Styling', 'Fieldset Legend', 'Accessible
//! Fieldset', 'Headless React Components', 'Form Section Labeling',
//! 'Base UI'). The page is prose-light: outside the Subtitle/meta description
//! and the Anatomy usage line (`:16`), it carries no narrative text.
//!
//! The live demo is the upstream Tailwind hero
//! (`docs/src/app/(docs)/react/components/fieldset/demos/hero/tailwind/index.tsx`,
//! cited by the single `specs/docs-content/fieldset/demos.json` entry) ported
//! onto the REAL `leptos_ui` parts — `FieldsetRoot`/`FieldsetLegend` plus
//! `FieldRoot`/`FieldLabel`/`FieldControl` — with every upstream `className`
//! string carried verbatim so the DOM the Leptos port produces matches the
//! React demo's element-for-element. The demo is uncontrolled (demos.json
//! `stateManaged: "none"` — "both Field.Controls are uncontrolled inputs with
//! no value state, no form submission, and no callbacks"): the port's own
//! machines own the rendered surface exactly as upstream's do — the
//! fieldset's `aria-labelledby` derivation from the legend's registration, and
//! each field's label↔control association. Upstream's prop exercises
//! (demos.json `propsExercised`) ride the real port vocabulary: the two
//! `Field.Control` `placeholder` values are upstream's `...elementProps` rest,
//! which the port spells `element_attributes` (`FieldControl.tsx:33-58`).
//!
//! demos.json's one `nonTrivialInteractions` entry is the reason the second
//! test below exists rather than only a structural assertion: rendering
//! `Fieldset.Legend` inside `Fieldset.Root` "exercises the automatic
//! aria-labelledby linking between the fieldset root and its legend", i.e. the
//! generated id must land on BOTH elements — a live consequence of the ported
//! `use_registered_label_id` hook, not a static attribute.

use leptos::prelude::*;

use leptos_ui::field_control::FieldControl;
use leptos_ui::field_parts::FieldLabel;
use leptos_ui::field_root::FieldRoot;
use leptos_ui::{FieldsetLegend, FieldsetRoot};

/// The upstream Fieldset.Root `className`
/// (`hero/tailwind/index.tsx:6`) — the one-column stack the legend and the two
/// fields sit in.
const DEMO_FIELDSET_CLASS: &str = "flex w-full max-w-64 flex-col gap-4";

/// The upstream Fieldset.Legend `className` (`hero/tailwind/index.tsx:7`) —
/// including the `dark:` variants upstream writes for the bottom rule.
const DEMO_LEGEND_CLASS: &str = "border-b border-neutral-950 text-base font-bold text-neutral-950 dark:border-white dark:text-white";

/// The upstream Field.Root `className` (`hero/tailwind/index.tsx:11,21`).
const DEMO_FIELD_CLASS: &str = "flex flex-col items-start gap-1";

/// The upstream Field.Label `className` (`hero/tailwind/index.tsx:12,22`).
const DEMO_LABEL_CLASS: &str = "text-sm font-bold text-neutral-950 dark:text-white";

/// The upstream Field.Control `className` (`hero/tailwind/index.tsx:17,27`) —
/// carried verbatim, colons and all (`any-pointer-coarse:`, `placeholder:`,
/// `focus:-outline-offset-*` are Tailwind variant syntax consumed by the
/// stylesheet). Note this is upstream's `w-full` control (the fieldset hero),
/// not the field page hero's `self-stretch` one; the two demos differ.
const DEMO_CONTROL_CLASS: &str = "h-8 w-full border border-neutral-950 bg-white dark:bg-neutral-950 px-2 text-sm any-pointer-coarse:text-base font-normal text-neutral-950 placeholder:text-neutral-500 focus:outline-2 focus:-outline-offset-1 focus:outline-neutral-950 dark:focus:outline-white dark:border-white dark:text-white dark:placeholder:text-neutral-400";

/// The hero demo (`demos/hero/tailwind/index.tsx:4-32`, demos.json entry 1): a
/// "Billing details" fieldset whose legend groups two labeled text fields
/// (Company / Tax ID), upstream's exact JSX shape in upstream's element order.
///
/// Each control's `placeholder` is upstream's bare JSX attribute, i.e. the
/// `...elementProps` rest the port spells `element_attributes` — the same
/// mapping the field docs page's hero uses.
#[component]
pub fn FieldsetHeroDemo() -> impl IntoView {
    view! {
        <FieldsetRoot class=DEMO_FIELDSET_CLASS.to_string()>
            <FieldsetLegend class=DEMO_LEGEND_CLASS.to_string()>"Billing details"</FieldsetLegend>

            <FieldRoot class=DEMO_FIELD_CLASS.to_string()>
                <FieldLabel class=DEMO_LABEL_CLASS.to_string()>"Company"</FieldLabel>
                <FieldControl
                    class=DEMO_CONTROL_CLASS.to_string()
                    element_attributes=vec![(
                        "placeholder".to_string(),
                        "Enter company name".to_string(),
                    )]
                />
            </FieldRoot>

            <FieldRoot class=DEMO_FIELD_CLASS.to_string()>
                <FieldLabel class=DEMO_LABEL_CLASS.to_string()>"Tax ID"</FieldLabel>
                <FieldControl
                    class=DEMO_CONTROL_CLASS.to_string()
                    element_attributes=vec![(
                        "placeholder".to_string(),
                        "Enter fiscal number".to_string(),
                    )]
                />
            </FieldRoot>
        </FieldsetRoot>
    }
}

/// The `## API reference` section, shaped like the rendered upstream page: each
/// part's `### Root` / `### Legend` heading, then the additional-type headings
/// the generated `TypesFieldset` component emits for that part —
/// `Fieldset.Root.Props`, `Fieldset.Root.State`, `Fieldset.Legend.Props`,
/// `Fieldset.Legend.State` — each rendered as an `<h3>` exactly as upstream
/// renders them (`.ReferenceSectionHeading.AdditionalTypeHeading`, verified
/// against the deployed page's HTML: `<h3 ...>Fieldset.Root.Props<a href="#"
/// class="AdditionalTypeBackLink">Hide</a></h3>`).
///
/// The upstream `Hide` disclosure link is deliberately NOT reproduced — it is
/// docs-site chrome around the heading text, so the port renders the heading
/// alone. `specs/docs-content/fieldset/page.md`'s "Page structure (headings, in
/// order)" list predates this check and records only Root/Legend; the omission
/// is written up in `ralph/logs/spec-discrepancies.md` rather than silently
/// editing the spec.
#[component]
fn FieldsetApiReference() -> impl IntoView {
    view! {
        <h2>"API reference"</h2>

        <h3>"Root"</h3>
        <p class="api-summary">
            "Groups a shared legend with related controls. Renders a <fieldset> element."
        </p>
        <h3>"Fieldset.Root.Props"</h3>
        <p class="api-summary">"Re-export of Root props."</p>
        <p class="api-props">
            "Props: className (string | ((state: Fieldset.Root.State) => string | undefined) — CSS class applied to the element, or a function that returns a class based on the component's state), style (React.CSSProperties | ((state: Fieldset.Root.State) => React.CSSProperties | undefined) — style applied to the element, or a function that returns a style object based on the component's state), render (ReactElement | ((props: HTMLProps, state: Fieldset.Root.State) => ReactElement) — allows you to replace the component's HTML element with a different tag, or compose it with another component; accepts a ReactElement or a function that returns the element to render)."
        </p>
        <h3>"Fieldset.Root.State"</h3>
        <p class="api-state">
            "State: Fieldset.Root.State — { disabled: boolean } — whether the component should ignore user interaction. Canonical alias: FieldsetRootState (also exported as FieldsetRootProps for the props)."
        </p>

        <h3>"Legend"</h3>
        <p class="api-summary">
            "An accessible label that is automatically associated with the fieldset. Renders a <div> element."
        </p>
        <h3>"Fieldset.Legend.Props"</h3>
        <p class="api-summary">"Re-export of Legend props."</p>
        <p class="api-props">
            "Props: className (string | ((state: Fieldset.Legend.State) => string | undefined) — CSS class applied to the element, or a function that returns a class based on the component's state), style (React.CSSProperties | ((state: Fieldset.Legend.State) => React.CSSProperties | undefined) — style applied to the element, or a function that returns a style object based on the component's state), render (ReactElement | ((props: HTMLProps, state: Fieldset.Legend.State) => ReactElement) — allows you to replace the component's HTML element with a different tag, or compose it with another component; accepts a ReactElement or a function that returns the element to render)."
        </p>
        <h3>"Fieldset.Legend.State"</h3>
        <p class="api-state">
            "State: Fieldset.Legend.State — { disabled: boolean } — whether the component should ignore user interaction. Canonical alias: FieldsetLegendState (also exported as FieldsetLegendProps for the props)."
        </p>
    }
}

/// The `docs/src/app/(docs)/react/components/fieldset/page.mdx` page.
#[component]
pub fn FieldsetPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Fieldset"</h1>
            <p class="subtitle">"A native fieldset element with an easily stylable legend."</p>

            <div class="docs-demo" data-demo="hero"><FieldsetHeroDemo /></div>

            <h2>"Anatomy"</h2>
            <p>"Import the component and assemble its parts:"</p>
            <pre><code>
"import { Fieldset } from '@base-ui/react/fieldset';

<Fieldset.Root>
  <Fieldset.Legend />
</Fieldset.Root>;"
            </code></pre>

            <FieldsetApiReference />
        </article>
    }
}
