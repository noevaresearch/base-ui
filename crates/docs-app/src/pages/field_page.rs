//! The docs page for `Field`, mirroring
//! `docs/src/app/(docs)/react/components/field/page.mdx`
//! (`specs/docs-content/field/page.md`) — the `docs-content: components/field`
//! TODO item.
//!
//! Page structure per the spec's "Page structure" section: `# Field` h1,
//! `<Subtitle>` ("A component that provides labeling and validation for form
//! controls."), the hero demo before the first heading, `## Anatomy` with the
//! single fenced snippet, then `## API reference` over the seven generated
//! `TypesField` reference tables (Root, Label, Control, Description, Item,
//! Error, Validity — echoed as static prose per the accordion/button/meter page
//! precedent: the port has no docs generator, so the tables' documented props
//! and data attributes are rendered as text, never fabricated as executable
//! machinery). The page has no `## Examples` section (`page.md`): the hero is
//! the only demo, and the API reference follows the Anatomy directly.
//!
//! Page furniture mirrored in module docs (the accordion/separator page
//! precedent): the `<Meta name="description">` content — "A high-quality,
//! unstyled React field component that provides labeling and validation for
//! form controls." — and the trailing `export const metadata` SEO keywords
//! block (11 keywords: 'React Field Component', 'Form Field Labeling',
//! 'Form Label', 'Input Wrapper', 'Form Validation', 'Accessible Form Field',
//! 'Field Validation UI', 'Headless React Components', 'Custom Form Control
//! Wrapper', 'Form Description Error State', 'Base UI').
//!
//! The live demo is the upstream Tailwind hero
//! (`docs/src/app/(docs)/react/components/field/demos/hero/tailwind/index.tsx`,
//! the single `specs/docs-content/field/demos.json` entry) ported onto the REAL
//! `leptos_ui` field parts — `FieldRoot`/`FieldLabel`/`FieldControl`/
//! `FieldError`/`FieldDescription` — with every upstream `className` string
//! carried verbatim so the DOM the Leptos port produces matches the React
//! demo's element-for-element. The demo is uncontrolled (demos.json
//! `stateManaged: "uncontrolled"` — "the demo holds no React state;
//! Field.Control is a plain uncontrolled input ... and Field's internal
//! validity state drives Field.Error visibility via match=\"valueMissing\""):
//! the port's own validation machine owns the rendered surface exactly as
//! upstream's does — the label↔control association, the `required`/`placeholder`
//! elementProps rest, and the Error slot's `valueMissing` gate all derive from
//! the props with zero demo-side machinery. Upstream's two prop exercises on the
//! control (`page.md` "API tables referenced", demos.json
//! `propsExercised["Field.Control"]`: `["required", "placeholder"]`) ride the
//! port's real `element_attributes` vocabulary — the `...elementProps` rest
//! (`FieldControl.tsx:33-58`), applied post-mount by the control's bag-writer
//! effect.

use leptos::prelude::*;

use leptos_ui::field_control::FieldControl;
use leptos_ui::field_parts::{ErrorMatch, FieldDescription, FieldError, FieldLabel};
use leptos_ui::field_root::FieldRoot;

/// The upstream Root `className` (`hero/tailwind/index.tsx:5`): the narrow
/// one-column stack the field's parts sit in.
const DEMO_ROOT_CLASS: &str = "flex w-full max-w-64 flex-col items-start gap-1";

/// The upstream Label `className` (`hero/tailwind/index.tsx:6`).
const DEMO_LABEL_CLASS: &str = "text-sm font-bold text-neutral-950 dark:text-white";

/// The upstream Control `className` (`hero/tailwind/index.tsx:10`) — carried
/// verbatim, colons and all (`any-pointer-coarse:`, `placeholder:`,
/// `focus:outline-*` are Tailwind variant syntax consumed by the stylesheet).
const DEMO_CONTROL_CLASS: &str = "h-8 self-stretch border border-neutral-950 bg-white dark:bg-neutral-950 px-2 text-sm any-pointer-coarse:text-base font-normal text-neutral-950 placeholder:text-neutral-500 focus:outline-2 focus:-outline-offset-1 focus:outline-neutral-950 dark:focus:outline-white dark:border-white dark:text-white dark:placeholder:text-neutral-400";

/// The upstream Error `className` (`hero/tailwind/index.tsx:12`).
const DEMO_ERROR_CLASS: &str = "text-sm text-red-700 dark:text-red-400";

/// The upstream Description `className` (`hero/tailwind/index.tsx:16`).
const DEMO_DESCRIPTION_CLASS: &str = "text-sm text-neutral-600 dark:text-neutral-400";

/// The hero demo (`demos/hero/tailwind/index.tsx`, demos.json entry 1): a
/// required uncontrolled "Name" field wiring a label, an input, a valueMissing
/// error slot, and a helper description — upstream's exact JSX shape, in
/// upstream's element order.
///
/// The two control props upstream writes as bare JSX attributes (`required`,
/// `placeholder="Required"`, `:7-9`) are the `...elementProps` rest, which the
/// port spells `element_attributes` (`FieldControl` takes the same bag): an
/// empty value is the boolean-attribute form (`required=""`), exactly what the
/// DOM API's `setAttribute("required", "")` produces.
#[component]
pub fn FieldHeroDemo() -> impl IntoView {
    view! {
        <FieldRoot class=DEMO_ROOT_CLASS.to_string()>
            <FieldLabel class=DEMO_LABEL_CLASS.to_string()>"Name"</FieldLabel>
            <FieldControl
                class=DEMO_CONTROL_CLASS.to_string()
                element_attributes=vec![
                    ("required".to_string(), String::new()),
                    ("placeholder".to_string(), "Required".to_string()),
                ]
            />
            <FieldError
                class=DEMO_ERROR_CLASS.to_string()
                error_match=ErrorMatch::Key("valueMissing".to_string())
            >
                "Please enter your name"
            </FieldError>
            <FieldDescription class=DEMO_DESCRIPTION_CLASS.to_string()>
                "Visible on your profile"
            </FieldDescription>
        </FieldRoot>
    }
}

/// One API-reference block: the generated `TypesField` tables
/// (`docs/src/app/(docs)/react/components/field/types.md`) echoed as static
/// prose — the summary line, the props list, and the data-attributes list.
fn api_part(summary: &'static str, props: &'static str, data_attrs: &'static str) -> impl IntoView {
    view! {
        <p class="api-summary">{summary}</p>
        <p class="api-props">{props}</p>
        <p class="api-data-attrs">{data_attrs}</p>
    }
}

/// The `docs/src/app/(docs)/react/components/field/page.mdx` page.
#[component]
pub fn FieldPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Field"</h1>
            <p class="subtitle">
                "A component that provides labeling and validation for form controls."
            </p>

            <div class="docs-demo" data-demo="hero"><FieldHeroDemo /></div>

            <h2>"Anatomy"</h2>
            <p>"Import the component and assemble its parts:"</p>
            <pre><code>
"import { Field } from '@base-ui/react/field';

<Field.Root>
  <Field.Label />
  <Field.Control />
  <Field.Description />
  <Field.Item />
  <Field.Error />
  <Field.Validity />
</Field.Root>;"
            </code></pre>

            <h2>"API reference"</h2>
            <h3>"Root"</h3>
            {api_part(
                "Groups all parts of the field. Renders a <div> element.",
                "Props: name (string — identifies the field when a form is submitted; takes precedence over the name prop on <Field.Control>), actionsRef (React.RefObject<Field.Root.Actions | null> — a ref to imperative actions; validate validates the field when called), dirty (boolean — whether the field's value has been changed from its initial value; useful when the field state is controlled by an external library), touched (boolean — whether the field has been touched; useful when the field state is controlled by an external library), disabled (boolean, false — whether the component should ignore user interaction; takes precedence over the disabled prop on <Field.Control>), invalid (boolean — whether the field is invalid; useful when the field state is controlled by an external library), validate ((value: unknown, formValues: Form.Values) => string | void | string[] | Promise<string | void | string[] | null> | null — custom validation; return a string or array of strings for the error message(s), and nothing/null/an empty string/an empty array for valid; asynchronous functions are supported but do not prevent form submission when using validationMode=\"onSubmit\"), validationMode (Form.ValidationMode, 'onSubmit' — determines when the field should be validated, taking precedence over the Form prop: 'onSubmit' validates on submit then re-validates on change after submission, 'onBlur' validates when the control loses focus, 'onChange' validates on every change), validationDebounceTime (number, 0 — how long to wait between validate callbacks under validationMode=\"onChange\", in milliseconds), className, style, render.",
                "Data attributes: data-disabled (present when the field is disabled), data-valid (present when the field is valid), data-invalid (present when the field is invalid), data-dirty (present when the field's value has changed), data-touched (present when the field has been touched), data-filled (present when the field is filled), data-focused (present when the field control is focused).",
            )}
            <h3>"Label"</h3>
            {api_part(
                "An accessible label that is automatically associated with the field control. Renders a <label> element.",
                "Props: nativeLabel (boolean, true — whether the component renders a native <label> element when replacing it via the render prop; set to false if the rendered element is not a label, for example <div>, which also avoids inheriting label behaviors on button controls such as <Select.Trigger> and <Combobox.Trigger>), className, style, render.",
                "Data attributes: data-disabled, data-valid, data-invalid, data-dirty, data-touched, data-filled, data-focused.",
            )}
            <h3>"Control"</h3>
            {api_part(
                "The form control to label and validate. Renders an <input> element. You can omit this part and use any Base UI input component instead — for example Input, Checkbox, or Select all work with Field out of the box.",
                "Props: defaultValue (string | number | string[]), onValueChange ((value: string, eventDetails: Field.Control.ChangeEventDetails) => void — fired when the value changes; use when controlled), className, style, render.",
                "Data attributes: data-disabled, data-valid, data-invalid, data-dirty, data-touched, data-filled, data-focused.",
            )}
            <h3>"Description"</h3>
            {api_part(
                "A paragraph with additional information about the field. Renders a <p> element.",
                "Props: className, style, render.",
                "Data attributes: data-disabled, data-valid, data-invalid, data-dirty, data-touched, data-filled, data-focused.",
            )}
            <h3>"Item"</h3>
            {api_part(
                "Groups individual items in a checkbox group or radio group with a label and description. Renders a <div> element.",
                "Props: disabled (boolean, false — whether the wrapped control should ignore user interaction; the disabled prop on <Field.Root> takes precedence), className, style, render.",
                "Data attributes: data-disabled, data-valid, data-invalid, data-dirty, data-touched, data-filled, data-focused.",
            )}
            <h3>"Error"</h3>
            {api_part(
                "An error message displayed if the field control fails validation. Renders a <div> element.",
                "Props: match (boolean | 'valid' | 'badInput' | 'customError' | 'patternMismatch' | 'rangeOverflow' | 'rangeUnderflow' | 'stepMismatch' | 'tooLong' | 'tooShort' | 'typeMismatch' | 'valueMissing' — determines whether to show the error message according to the field's ValidityState; true always shows the message and lets external libraries control visibility), className, style, render.",
                "Data attributes: data-disabled, data-valid, data-invalid, data-dirty, data-touched, data-filled, data-focused, data-starting-style (present when the error message begins animating in), data-ending-style (present when the error message is animating out).",
            )}
            <h3>"Validity"</h3>
            {api_part(
                "Used to display a custom message based on the field's validity. Requires children to be a function that accepts field validity state as an argument.",
                "Props: children ((state: Field.Validity.State) => React.ReactNode, required — a function that accepts the field validity state as an argument; the state carries validity, value, error, errors, initialValue, and transitionStatus).",
                "Data attributes: none — the part renders nothing itself; the render function's output is the caller's.",
            )}
        </article>
    }
}
