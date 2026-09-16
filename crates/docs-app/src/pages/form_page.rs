//! The docs page for `Form`, mirroring
//! `docs/src/app/(docs)/react/components/form/page.mdx`
//! (`specs/docs-content/form/page.md`) — the `docs-content: components/form`
//! TODO item, and the Phase D half of the `library: form` pair.
//!
//! Page structure per the spec's "Page structure" section: `# Form` h1
//! (`page.mdx:1`), `<Subtitle>` ("A native form element with consolidated error
//! handling.", `:3`), the hero demo **before the first heading** (`:10-12`),
//! `## Anatomy` (`:14`) with its single fenced snippet (`:18-29`), `## Examples`
//! (`:31`) over the three subsections — "Submit with a Server Function"
//! (`:33`, external link to the React docs at `:35`), "Submit form values as a
//! JavaScript object" (`:41`, the `onFormSubmit` snippet at `:45-60` plus the
//! `preventDefault` claim at `:62`), and "Using with Zod" (`:64`) — then
//! `## API reference` (`:72`) over the single generated `<TypesForm />`
//! reference (`:74`), echoed as static prose per the
//! fieldset/field/button/checkbox page precedent: the port has no docs
//! generator, so `docs/src/app/(docs)/react/components/form/types.md`'s one
//! part table (Form), its `actionsRef` example, its seven type sections and its
//! Canonical Types list are rendered as text, never fabricated as executable
//! machinery.
//!
//! Page furniture mirrored in module docs (the field/fieldset page precedent):
//! the `<Meta name="description">` content — "A high-quality, unstyled React
//! form component with consolidated error handling." (`page.mdx:5-8`) — and the
//! trailing `export const metadata` SEO keywords block (10 keywords,
//! `page.mdx:78-91`: 'React Form Component', 'Form Submission Handler', 'Form
//! Validation', 'Form State', 'Server Function Form', 'JavaScript Form Values',
//! 'Accessible Form', 'Headless React Components', 'Form Error Handling',
//! 'Base UI').
//!
//! ## Two heading-text facts the live upstream comparison forced out
//!
//! This page is the first in the loop to be diffed against the REAL React docs page
//! (`DIFF_UPSTREAM=1 node ralph/scripts/playwright-diff.mjs --todo-id "docs-content:
//! components/form"`, with the Next dev server on `:3005`): the heading-text subset was
//! 5/14, and both gap classes are mirrored here deliberately.
//!
//! 1. **The `## Examples` subsections' heading text carries a non-breaking space.**
//!    `page.mdx:33` is plain `### Submit with a Server Function`, but the docs pipeline
//!    (`docs/next.config.mjs:41`'s `remark-typography`) renders it as
//!    `Submit with a Server\u{a0}Function` — the widow-prevention transform that also
//!    gives `Rendering as a native\u{a0}button` on the already-ported checkbox page and
//!    `Rendering links as\u{a0}buttons` on the button page. The port mirrors the RENDERED
//!    text, because that is what a visitor (and the differential's `textContent`
//!    snapshot) sees; the source-level plain space is in the spec, which is
//!    unchanged. Checked against the oracle: `grep -c $'\u00a0'` on the served upstream
//!    HTML finds it, and the `page.mdx` source has no `\u00a0` in those headings.
//! 2. **The generated type sections carry an upstream back-link, not a bare heading.**
//!    Upstream renders the `TypesForm` additional types through `AdditionalTypes`
//!    (`docs/src/components/ReferenceTable/AdditionalTypes.tsx`), whose `<h3>` is
//!    `{name}` immediately followed by `<a href="#" class="AdditionalTypeBackLink">Hide</a>`
//!    (`:44-55`), so its `textContent` is `Form.PropsHide`, not `Form.Props`. See
//!    [`api_part`] — the wrapper div, the two class names and the label are upstream's
//!    verbatim.
//!
//! ## The three live demos and how they ride the real port
//!
//! Every demo is the upstream Tailwind demo (`docs/src/app/(docs)/react/components/form/demos/`,
//! the three `specs/docs-content/form/demos.json` entries) ported onto the REAL
//! `leptos_ui` parts — `Form`, `FieldRoot`/`FieldLabel`/`FieldControl`/`FieldError`
//! and the `button_element` engine — with every upstream `className` string
//! carried verbatim, so the DOM the Leptos port produces matches the React
//! demo's element-for-element.
//!
//! All three demos are *server-error* demos: a submit reaches a fake async
//! server and the response comes back as an external `errors` record keyed by
//! `Field.Root name`, which `Field.Error` then surfaces. Two adaptations of the
//! port's architecture are visible in them, and they are the same two in all
//! three:
//!
//! 1. **The errors record is delivered as the `errors` prop, and the `Form`
//!    subtree rebuilds when it changes.** Upstream owns the record in demo state
//!    and passes it as the prop; the re-render it triggers is what makes
//!    `Field.Error` see it. The port's props are plain values (`form.rs`'s
//!    `errors`-seeding note: "a prop change is a subtree rebuild"), so the demo
//!    renders its `<Form>` inside a tracked child closure keyed on the record —
//!    the button/merge-props pages' React-re-render analog. The port's other
//!    documented channel, [`leptos_ui::FormErrorsHandle`] (`form.rs:180-236`),
//!    writes the component's mirror without a rebuild, but a `Field.Error` built
//!    *after* the `Form`'s own build window has closed cannot resolve the form
//!    context it reads its render gate from: the context is registered on the
//!    reactive-graph (rg-0.2) owner the `Form` provides it on, while a view
//!    closure that re-runs later for the leptos (rg-0.1) tree — the internals
//!    crate's dual-runtime law, which this iteration confirmed empirically (the
//!    field's own `data-invalid` flipped from a handle write while the error
//!    slot stayed hidden, because the slot's gate read the inert default
//!    context). The demos therefore take the prop path, which is the one that
//!    puts the whole subtree — and so every context read — inside the window.
//!    Consequence, stated plainly: the handle is not used, so the "errors that
//!    arrive after a submit still focus the first invalid control" half of
//!    behavior.md's Focus-management contract is not *demonstrated* by this page
//!    (it is covered by the `leptos-ui` suite: `form_tests.rs:900-973`).
//! 2. **The uncontrolled controls are re-seeded from the values the demo
//!    submitted.** A rebuild creates new DOM nodes, so an uncontrolled
//!    `Field.Control` would come back at its initial `defaultValue` — upstream's
//!    uncontrolled input keeps whatever the user typed across its re-render. The
//!    demos mirror the submitted values in their own signals and pass them back
//!    as `default_value`, so the control the visitor sees keeps the typed text
//!    exactly as upstream's does. (The control still has no `value` prop — it is
//!    uncontrolled in both; only the demo's computed `defaultValue` differs from
//!    the literal upstream passes.)
//!
//! One demo needs a third adaptation, recorded here rather than hidden:
//! upstream's "Submit with a Server Function" demo submits through React's
//! `action` prop with `useActionState` (`form-action/tailwind/index.tsx:12-18`).
//! React's action/Server-Function flow has no counterpart in the ported `Form`
//! (its prop set is `errors`/`validationMode`/`noValidate`/`onSubmit`/
//! `onFormSubmit`/`actionsRef`/`errorsHandle`), so that demo reproduces the
//! *observable* contract — a native submit, a 1s fake server action, server
//! errors keyed by `Field.Root name` landing on the matching field, and a submit
//! button disabled-but-focusable while pending — through the port's own seams:
//! the native `onSubmit`, with the response delivered as the same `errors`
//! prop. The DOM the visitor sees is upstream's, including the button's
//! `data-disabled` surface.

use crate::code_block::{Lang, code_block};
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use leptos_ui::button_element;
use leptos_ui::field_control::FieldControl;
use leptos_ui::field_parts::{FieldError, FieldLabel};
use leptos_ui::field_root::FieldRoot;
use leptos_ui::{ButtonProps, Form, FormErrorValue, FormErrors};
use leptos_ui_internals::timeout_manager::TimeoutManager;
use leptos_ui_internals::use_render_element::{ClassNameSource, UseRenderElementComponentProps};

use crate::pages::use_render_page::RawElementView;

// ---------------------------------------------------------------------------
// The upstream class strings, carried verbatim
// ---------------------------------------------------------------------------

/// The upstream `Form` `className` (`hero/tailwind/index.tsx:13`,
/// `form-action/tailwind/index.tsx:18`, `zod/tailwind/index.tsx:32` — all three
/// demos share it).
const DEMO_FORM_CLASS: &str = "flex w-full max-w-64 flex-col gap-4";

/// The upstream `Field.Root` `className` (`hero/tailwind/index.tsx:30`,
/// `form-action/tailwind/index.tsx:20`, `zod/tailwind/index.tsx:39,49`).
const DEMO_FIELD_CLASS: &str = "flex flex-col items-start gap-1";

/// The upstream `Field.Label` `className` (`hero/tailwind/index.tsx:31`).
const DEMO_LABEL_CLASS: &str = "text-sm font-bold text-neutral-950 dark:text-white";

/// The upstream `Field.Control` `className`
/// (`hero/tailwind/index.tsx:40`) — carried verbatim, colons and all
/// (`any-pointer-coarse:`, `placeholder:`, `focus:-outline-offset-*` are
/// Tailwind variant syntax consumed by the stylesheet).
const DEMO_CONTROL_CLASS: &str = "h-8 w-full border border-neutral-950 bg-white dark:bg-neutral-950 px-2 text-sm any-pointer-coarse:text-base font-normal text-neutral-950 placeholder:text-neutral-500 focus:outline-2 focus:-outline-offset-1 focus:outline-neutral-950 dark:focus:outline-white dark:border-white dark:text-white dark:placeholder:text-neutral-400";

/// The upstream `Field.Error` `className` (`hero/tailwind/index.tsx:42`) — the
/// red rule every demo's error slot carries. All three demos render
/// `<Field.Error className=…/>` with **no** children upstream and let the field's
/// own error message fill the slot.
const DEMO_ERROR_CLASS: &str = "text-sm text-red-700 dark:text-red-400";

/// The upstream `Button` `className` (`hero/tailwind/index.tsx:48`,
/// `form-action/tailwind/index.tsx:38`, `zod/tailwind/index.tsx:61` — all three
/// carry the identical string).
const DEMO_BUTTON_CLASS: &str = "flex h-8 items-center justify-center gap-2 rounded-none border border-neutral-950 bg-white px-3 text-sm leading-none whitespace-nowrap font-normal text-neutral-950 select-none hover:not-data-disabled:bg-neutral-100 active:not-data-disabled:bg-neutral-200 focus-visible:outline-2 focus-visible:-outline-offset-1 focus-visible:outline-neutral-950 dark:focus-visible:outline-white data-disabled:border-neutral-500 data-disabled:text-neutral-500 disabled:border-neutral-500 disabled:text-neutral-500 dark:border-white dark:bg-neutral-950 dark:text-white dark:hover:not-data-disabled:bg-neutral-800 dark:active:not-data-disabled:bg-neutral-700 dark:data-disabled:border-neutral-400 dark:data-disabled:text-neutral-400";

/// The `## Anatomy` snippet (`page.mdx:18-29`) — Form composed together with Field. Translated to
/// the port's namespaced surface: `leptos_ui::Form` is a plain component (upstream's `<Form>` has no
/// subcomponents — `specs/library/form/behavior.md` § Public API surface), and the nested parts are
/// the same `Field::*` items the field page teaches
/// (`crates/leptos-ui/tests/part_surface.rs:150-190`), with `Field.Label`/`Field.Error` taking
/// their content as children.
const ANATOMY_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{Field, Form};

view! {
    <Form>
        <Field::Root>
            <Field::Label>"Quantity"</Field::Label>
            <Field::Control />
            <Field::Error>"Required"</Field::Error>
        </Field::Root>
    </Form>
}"#;

/// The "Submission using `onFormSubmit`" snippet (`page.mdx:45-60`), translated: the port's
/// `on_form_submit` receives the values record as `FormValues = Vec<(String, serde_json::Value)>`
/// (`crates/leptos-ui/src/form.rs:141`) plus upstream's event details, and the handler builds the
/// same `{ product_id, order_quantity }` payload upstream's `async` handler builds. The port calls
/// no service itself, so the request is shown as the payload the handler produces (the shape the
/// page's own `FormZodDemo` uses). `Form`'s children are not optional in the port (the parts are the
/// form), so the snippet keeps the Field it submits, where upstream's fragment is self-closing.
const ON_FORM_SUBMIT_SNIPPET: &str = r#"use std::rc::Rc;
use leptos::prelude::*;
use leptos_ui::{Field, Form, FormSubmitEventDetails, FormValues};

let on_form_submit: Rc<dyn Fn(FormValues, FormSubmitEventDetails)> =
    Rc::new(move |form_values: FormValues, _details: FormSubmitEventDetails| {
        let value = |name: &str| {
            form_values
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.clone())
        };

        // POST this payload to https://api.example.com as JSON.
        let _payload = serde_json::json!({
            "product_id": value("id"),
            "order_quantity": value("quantity"),
        });
    });

view! {
    <Form on_form_submit=on_form_submit>
        <Field::Root>
            <Field::Label>"Quantity"</Field::Label>
            <Field::Control />
        </Field::Root>
    </Form>
}"#;

/// The `actionsRef` usage example the generated `TypesForm` reference carries (`types.md`), in the
/// port's spelling: `FormActionsRef` is `Rc<Cell<Option<Rc<FormActions>>>>`
/// (`crates/leptos-ui/src/form.rs:178`), so the handle the mounted Form wrote is read with
/// `take()` — the shape the crate's own suite uses (`crates/leptos-ui/src/form_tests.rs:878-884`) —
/// and `FormActions::validate` takes `Option<&str>` (`form.rs:170-176`): `None` re-validates every
/// registered field, `Some(name)` the first field with that name.
const ACTIONS_REF_SNIPPET: &str = r#"use leptos_ui::FormActionsRef;

// the Form wrote its imperative handle into the slot when it mounted
let actions = actions_ref.take().expect("the Form mounted");

// validate all fields
actions.validate(None);

// validate one field
actions.validate(Some("email"));"#;

// ---------------------------------------------------------------------------
// Shared demo machinery
// ---------------------------------------------------------------------------

/// The demo's submit button, built by the REAL port: `button_element` with the
/// upstream class string, `type="submit"` through the `...elementProps` rest
/// (`Button.spec.tsx`'s override — the earlier-bag-wins rule), the loading
/// demos' `focusableWhenDisabled`, and the label text as the button's content.
fn demo_submit_button(disabled: bool, label: &str) -> RawElementView {
    let rendered = button_element(ButtonProps {
        disabled,
        // hero (`hero/tailwind/index.tsx:46`) and form-action
        // (`form-action/tailwind/index.tsx:37`) both pass
        // `focusableWhenDisabled`, so the pending button stays keyboard
        // focusable; the zod demo's button is never disabled, so the flag has
        // nothing to gate there (behavior.md "Events": the guard only gates user
        // interaction while `disabled`).
        focusable_when_disabled: true,
        native_button: true,
        render_class_style: UseRenderElementComponentProps {
            class_name: Some(ClassNameSource::Static(DEMO_BUTTON_CLASS.to_string())),
            render: None,
            style: None,
        },
        element_attributes: vec![("type".to_string(), "submit".to_string())],
        handlers: Default::default(),
    })
    .expect("the demo Button renders (a leaf with no enabled gate)");

    let mut rendered = rendered;
    rendered.props.inner_html = Some(label.to_string());

    RawElementView::new(rendered)
}

/// `formData.get(name) as string` from the native submit event
/// (`hero/tailwind/index.tsx:17-18`, `form-action/tailwind/index.tsx:54`) — the
/// upstream `new FormData(event.currentTarget)` read, over the real form node
/// the injected submit listener is attached to.
fn form_value(event: &web_sys::Event, name: &str) -> Option<String> {
    let target = event.current_target()?;
    let form: web_sys::HtmlFormElement = target.dyn_into().ok()?;
    let data = web_sys::FormData::new_with_form(&form).ok()?;
    data.get(name).as_string()
}

/// The string one `onFormSubmit` values entry carries (the port's
/// `Form.Values` is an insertion-ordered record of JSON values; a
/// `Field.Control` reports its DOM value as a string,
/// `field_control.rs:229-233`).
fn value_of(values: &[(String, serde_json::Value)], name: &str) -> String {
    values
        .iter()
        .find(|(key, _)| key == name)
        .and_then(|(_, value)| value.as_str())
        .unwrap_or_default()
        .to_string()
}

/// The message the demo's error record carries for one field name — upstream's
/// `errors[name]` lookup (`FieldError.tsx:39-40`), with a single-element array
/// collapsed to its string exactly as upstream's `errorMessage` does
/// (`FieldError.tsx:82-93`). This is the same text upstream renders inside the
/// error element; the port's `FieldError` takes it as children rather than
/// deriving it from the record (`field_parts.rs:436-453`'s form-error arm is the
/// sentinel, and the `FieldError` component always passes children — the shape
/// the field docs page's hero uses for "Please enter your name").
fn message_for(errors: &FormErrors, name: &str) -> String {
    match errors.iter().find(|(key, _)| key == name) {
        Some((_, FormErrorValue::Single(message))) => message.clone(),
        Some((_, FormErrorValue::Multiple(messages))) => {
            messages.first().cloned().unwrap_or_default()
        }
        None => String::new(),
    }
}

/// One entry of a demo's error record (`{ url: response.error }`).
fn single_error(name: &str, message: &str) -> FormErrors {
    vec![(
        name.to_string(),
        FormErrorValue::Single(message.to_string()),
    )]
}

/// The fake server behind the hero demo (`hero/tailwind/index.tsx:56-73`):
/// after the 1s delay, `new URL(value)` decides between "The example domain is
/// not allowed" (a resolvable URL under `example.com`) and "This is not a valid
/// URL" (the constructor throwing), else success. `web_sys::Url` is the
/// browser's own URL parser, so the `endsWith('example.com')` hostname check is
/// the same one upstream performs.
fn hero_fake_server(value: &str) -> Option<String> {
    match web_sys::Url::new(value) {
        Ok(url) => {
            if url.hostname().ends_with("example.com") {
                Some("The example domain is not allowed".to_string())
            } else {
                None
            }
        }
        Err(_) => Some("This is not a valid URL".to_string()),
    }
}

/// The fake server function behind the form-action demo
/// (`form-action/tailwind/index.tsx:47-73`): `'admin'` is reserved,
/// otherwise a 50% chance the username is taken, else success.
fn action_fake_server(username: &str) -> FormErrors {
    if username == "admin" {
        return single_error("username", "'admin' is reserved for system use");
    }
    // `Math.random() > 0.5` — upstream's coin flip for "username is unavailable".
    if js_sys::Math::random() > 0.5 {
        FormErrors::new()
    } else {
        single_error("username", &format!("{username} is unavailable"))
    }
}

/// `Number(value)` — the coercion `z.coerce.number()` performs
/// (`zod/tailwind/index.tsx:10`): the empty string is `0`, an unparseable value
/// is `NaN`.
fn coerce_number(value: Option<&serde_json::Value>) -> f64 {
    match value {
        Some(serde_json::Value::Number(number)) => number.as_f64().unwrap_or(f64::NAN),
        Some(serde_json::Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                0.0
            } else {
                trimmed.parse::<f64>().unwrap_or(f64::NAN)
            }
        }
        Some(serde_json::Value::Null) | None => 0.0,
        _ => f64::NAN,
    }
}

/// The Zod schema of the zod demo (`zod/tailwind/index.tsx:8-11`) applied to
/// `onFormSubmit`'s values record, with `z.flattenError(...).fieldErrors`'s
/// shape: one entry per failing field, keyed by the `Field.Root name`. Zod
/// itself is a JavaScript schema library with no part in Base UI (the page
/// spec's "Discrepancies" records the same: the Zod API specifics have no
/// counterpart in behavior.md), so the schema's two rules are ported directly:
/// `z.string().min(1, 'Name is required')` and
/// `z.coerce.number('Age must be a number').positive('Age must be a positive
/// number')`.
fn zod_field_errors(values: &[(String, serde_json::Value)]) -> FormErrors {
    let lookup = |name: &str| {
        values
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    };

    let mut errors = FormErrors::new();

    let name = lookup("name")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    if name.is_empty() {
        errors.extend(single_error("name", "Name is required"));
    }

    let age = coerce_number(lookup("age"));
    if age.is_nan() {
        errors.extend(single_error("age", "Age must be a number"));
    } else if age <= 0.0 {
        errors.extend(single_error("age", "Age must be a positive number"));
    }

    errors
}

// ---------------------------------------------------------------------------
// Demo 1 — hero (`demos/hero/tailwind/index.tsx`)
// ---------------------------------------------------------------------------

/// The hero demo (`hero/tailwind/index.tsx:7-73`, demos.json entry "hero"): a
/// `Form` whose `onSubmit` validates a URL against a fake async server and feeds
/// the server error back into the `errors` record so it renders on the field.
///
/// Upstream's two `useState`s are the two signals here — `errors` (seeded `{}`)
/// and `loading` (seeded `false`, gating the submit button's `disabled`) — and
/// the control is uncontrolled with `defaultValue="https://example.com"`, its
/// `type="url"`/`required`/`placeholder`/`pattern="https?://.*"` riding the
/// `...elementProps` rest the port spells `element_attributes`. The third signal
/// mirrors the value the fake server received so the rebuild re-seeds
/// `defaultValue` with it (this module's header, adaptation 2).
///
/// `delay_ms` is the fake server's 1s wait (`:58-60`); the page passes 1000 and
/// the tests pass a value short enough to observe the pending state.
#[component]
pub fn FormHeroDemo(delay_ms: i32) -> impl IntoView {
    let errors = RwSignal::new(FormErrors::new());
    let loading = RwSignal::new(false);
    let submitted_value = RwSignal::new("https://example.com".to_string());
    let timeouts = TimeoutManager::default();

    let on_submit: Rc<dyn Fn(&web_sys::Event)> = {
        let timeouts = timeouts.clone();
        Rc::new(move |event: &web_sys::Event| {
            // `event.preventDefault(); const formData = new FormData(event.currentTarget);`
            // (`:16-17`) — upstream's handler prevents the native submission itself.
            event.prevent_default();
            let value = form_value(event, "url").unwrap_or_default();
            submitted_value.set(value.clone());

            loading.set(true);
            // `await submitForm(value)` — the ported dual-target timer stands in for
            // the demo's `setTimeout` wait (the AGENTS.md rule).
            timeouts.start("form-hero-demo", delay_ms, move || {
                let record = match hero_fake_server(&value) {
                    Some(message) => single_error("url", &message),
                    // `{ success: true }` — `errors` becomes `{ url: undefined }`,
                    // i.e. no form error for the field.
                    None => FormErrors::new(),
                };
                errors.set(record);
                loading.set(false);
            });
        })
    };
    // The rebuild closure below is a VIEW node, so leptos requires it to be
    // `Send + Sync` — and the handler is an `Rc`. `SendWrapper` is the honest
    // adapter on the single-threaded wasm target (the button page's own note for
    // the same wall), unwrapped with `take()` when the prop is built.
    let on_submit = send_wrapper::SendWrapper::new(on_submit);

    view! {
        <div class="docs-demo-form" data-demo-part="hero">
            {move || {
                let record = errors.get();
                let is_loading = loading.get();
                let value = submitted_value.get();
                view! {
                    <Form
                        class=DEMO_FORM_CLASS.to_string()
                        errors=record.clone()
                        on_submit=on_submit.clone().take()
                    >
                        <FieldRoot name="url".to_string() class=DEMO_FIELD_CLASS.to_string()>
                            <FieldLabel class=DEMO_LABEL_CLASS.to_string()>"Homepage"</FieldLabel>
                            <FieldControl
                                class=DEMO_CONTROL_CLASS.to_string()
                                default_value=value
                                element_attributes=vec![
                                    ("type".to_string(), "url".to_string()),
                                    ("required".to_string(), String::new()),
                                    ("placeholder".to_string(), "https://example.com".to_string()),
                                    ("pattern".to_string(), "https?://.*".to_string()),
                                ]
                            />
                            <FieldError class=DEMO_ERROR_CLASS.to_string()>
                                {message_for(&record, "url")}
                            </FieldError>
                        </FieldRoot>
                        {demo_submit_button(is_loading, "Submit")}
                    </Form>
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Demo 2 — form-action (`demos/form-action/tailwind/index.tsx`)
// ---------------------------------------------------------------------------

/// The form-action demo (`form-action/tailwind/index.tsx:11-73`, demos.json
/// entry "form-action"): upstream submits through React's `action` prop with
/// `useActionState` and renders the action's `state.serverErrors` on the
/// matching field while `loading` gates the submit button.
///
/// See this module's header for the adaptation: the port's `Form` has no
/// `action` prop, so the demo drives the native `onSubmit` and delivers the
/// server errors through the same `errors` prop the other demos use. The
/// username control is uncontrolled with `defaultValue="admin"` (`:28`), the
/// value that always hits the reserved-name branch of the fake server.
#[component]
pub fn FormActionDemo(delay_ms: i32) -> impl IntoView {
    let errors = RwSignal::new(FormErrors::new());
    let loading = RwSignal::new(false);
    let submitted_value = RwSignal::new("admin".to_string());
    let timeouts = TimeoutManager::default();

    let on_submit: Rc<dyn Fn(&web_sys::Event)> = {
        let timeouts = timeouts.clone();
        Rc::new(move |event: &web_sys::Event| {
            // The action flow's native submission: upstream's `action={formAction}`
            // hands the server function the FormData and never calls `onSubmit`
            // (`:16`); the port's equivalent seam is the native submit handler, which
            // must prevent the default itself.
            event.prevent_default();
            let username = form_value(event, "username").unwrap_or_default();
            submitted_value.set(username.clone());

            loading.set(true);
            timeouts.start("form-action-demo", delay_ms, move || {
                errors.set(action_fake_server(&username));
                loading.set(false);
            });
        })
    };
    // The view-closure `Send + Sync` requirement, as in the hero demo.
    let on_submit = send_wrapper::SendWrapper::new(on_submit);

    view! {
        <div class="docs-demo-form" data-demo-part="form-action">
            {move || {
                let record = errors.get();
                let is_loading = loading.get();
                let value = submitted_value.get();
                view! {
                    <Form
                        class=DEMO_FORM_CLASS.to_string()
                        errors=record.clone()
                        on_submit=on_submit.clone().take()
                    >
                        <FieldRoot name="username".to_string() class=DEMO_FIELD_CLASS.to_string()>
                            <FieldLabel class=DEMO_LABEL_CLASS.to_string()>"Username"</FieldLabel>
                            <FieldControl
                                class=DEMO_CONTROL_CLASS.to_string()
                                default_value=value
                                element_attributes=vec![
                                    ("type".to_string(), "text".to_string()),
                                    ("autocomplete".to_string(), "username".to_string()),
                                    ("required".to_string(), String::new()),
                                    ("placeholder".to_string(), "e.g. alice132".to_string()),
                                ]
                            />
                            <FieldError class=DEMO_ERROR_CLASS.to_string()>
                                {message_for(&record, "username")}
                            </FieldError>
                        </FieldRoot>
                        {demo_submit_button(is_loading, "Submit")}
                    </Form>
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Demo 3 — zod (`demos/zod/tailwind/index.tsx`)
// ---------------------------------------------------------------------------

/// The zod demo (`zod/tailwind/index.tsx:27-67`, demos.json entry "zod"):
/// schema-validating the values record inside `onFormSubmit` and mapping the
/// flattened field errors back through the `errors` record, so each
/// `Field.Error` shows its own message. Both controls are uncontrolled and
/// carry only `placeholder` (`:44,54`), so nothing gates submission natively —
/// the schema's verdict is what populates the errors. Their submitted values are
/// mirrored for the rebuild's `defaultValue` re-seed (this module's header,
/// adaptation 2).
#[component]
pub fn FormZodDemo() -> impl IntoView {
    let errors = RwSignal::new(FormErrors::new());
    let name_value = RwSignal::new(String::new());
    let age_value = RwSignal::new(String::new());

    let on_form_submit: Rc<dyn Fn(leptos_ui::FormValues, leptos_ui::FormSubmitEventDetails)> = {
        Rc::new(
            move |form_values: leptos_ui::FormValues,
                  _details: leptos_ui::FormSubmitEventDetails| {
                // `const result = schema.safeParse(formValues); … setErrors(response.errors)`
                // (`:14-24`) — a successful parse resets `errors` to `{}`, clearing
                // every field error (the page's own claim at `:63`).
                name_value.set(value_of(&form_values, "name"));
                age_value.set(value_of(&form_values, "age"));
                errors.set(zod_field_errors(&form_values));
            },
        )
    };
    // The view-closure `Send + Sync` requirement, as in the hero demo.
    let on_form_submit = send_wrapper::SendWrapper::new(on_form_submit);

    view! {
        <div class="docs-demo-form" data-demo-part="zod">
            {move || {
                let record = errors.get();
                let name = name_value.get();
                let age = age_value.get();
                // The two slots' messages are read before the view so the closure the
                // macro generates for each child does not move `record` twice
                // (E0382: `record` is stored in the `Form` prop *and* read by both
                // `FieldError` children, each of which the macro wraps in its own
                // `FnOnce`).
                let name_message = message_for(&record, "name");
                let age_message = message_for(&record, "age");
                view! {
                    <Form
                        class=DEMO_FORM_CLASS.to_string()
                        errors=record.clone()
                        on_form_submit=on_form_submit.clone().take()
                    >
                        <FieldRoot name="name".to_string() class=DEMO_FIELD_CLASS.to_string()>
                            <FieldLabel class=DEMO_LABEL_CLASS.to_string()>"Name"</FieldLabel>
                            <FieldControl
                                class=DEMO_CONTROL_CLASS.to_string()
                                default_value=name
                                element_attributes=vec![(
                                    "placeholder".to_string(),
                                    "Enter name".to_string(),
                                )]
                            />
                            <FieldError class=DEMO_ERROR_CLASS.to_string()>
                                {name_message.clone()}
                            </FieldError>
                        </FieldRoot>
                        <FieldRoot name="age".to_string() class=DEMO_FIELD_CLASS.to_string()>
                            <FieldLabel class=DEMO_LABEL_CLASS.to_string()>"Age"</FieldLabel>
                            <FieldControl
                                class=DEMO_CONTROL_CLASS.to_string()
                                default_value=age
                                element_attributes=vec![(
                                    "placeholder".to_string(),
                                    "Enter age".to_string(),
                                )]
                            />
                            <FieldError class=DEMO_ERROR_CLASS.to_string()>
                                {age_message.clone()}
                            </FieldError>
                        </FieldRoot>
                        {demo_submit_button(false, "Submit")}
                    </Form>
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// The page
// ---------------------------------------------------------------------------

/// One generated type section of the `## API reference`: upstream's `AdditionalTypes`
/// markup — the `AdditionalTypeWrapper` div keyed by the type's slug
/// (`AdditionalTypes.tsx:36-43`, `id="form.state"`-style: the lowercased name, dots
/// kept), the `<h3 class="ReferenceSectionHeading AdditionalTypeHeading">` carrying the
/// type name **plus** the `AdditionalTypeBackLink` whose default label is the literal
/// `Hide` (`:44-55`; `hydrated && canGoBack ? 'Back' : 'Hide'` at `:54`, which resolves to
/// `Hide` in the page's initial DOM — the state the Playwright differential snapshots),
/// and then the section's content: upstream renders the `Re-Export of … as …` line
/// (`:57-67`) or a code block holding the declaration (`:69-71`). The port echoes that
/// content as static prose (the page spec's "API tables referenced" section, the
/// fieldset/field/button/checkbox precedent) — the generated-type machinery itself is
/// not fabricated.
///
/// The back-link is mirrored as markup, not as behavior: upstream's `onClick` (a
/// `history.back()` on the hydrated page, `:28-31`) has no counterpart in a static
/// mirror, so the anchor keeps upstream's `href="#"` and label and is inert.
fn api_part(name: &'static str, summary: &'static str, props: &'static str) -> impl IntoView {
    view! {
        <div id=name.to_lowercase() class="AdditionalTypeWrapper">
            <h3 class="ReferenceSectionHeading AdditionalTypeHeading">
                {name}
                <a href="#" class="AdditionalTypeBackLink">"Hide"</a>
            </h3>
            <p class="api-summary">{summary}</p>
            <p class="api-props">{props}</p>
        </div>
    }
}

/// The `## API reference` section: the generated `TypesForm` reference
/// (`docs/src/app/(docs)/react/components/form/types.md`) — one part table
/// (`### Form`), its `actionsRef` usage example, the seven type sections
/// (`Form.Props`, `Form.State`, `Form.Actions`, `Form.SubmitEventDetails`,
/// `Form.SubmitEventReason`, `Form.ValidationMode`, `Form.Values`) and the
/// Canonical Types list, each rendered as the heading text upstream renders.
#[component]
fn FormApiReference() -> impl IntoView {
    view! {
        <h2>"API reference"</h2>

        <h3>"Form"</h3>
        <p class="api-summary">
            "A native form element with consolidated error handling. Renders a <form> element."
        </p>
        <p class="api-props">
            "Props: errors (Errors — validation errors returned externally, typically after submission by a server or a form action; this should be an object where keys correspond to the name attribute on <Field.Root>, and values correspond to error(s) related to that field), actionsRef (FormActionsRef — a ref to imperative actions; validate validates all fields when called, optionally passing a field name to validate a single field), onFormSubmit (((formValues: Record<string, any>, eventDetails: Form.SubmitEventDetails) => void) — event handler called when the form is submitted; preventDefault() is called on the native submit event when used), validationMode (Form.ValidationMode, 'onSubmit' — determines when the form should be validated; the validationMode prop on <Field.Root> takes precedence over this: 'onSubmit' validates the field when the form is submitted, afterwards fields will re-validate on change, 'onBlur' validates a field when it loses focus, 'onChange' validates the field on every change to its value), className (string | ((state: Form.State) => string | undefined)), this port's Form takes `class` and `children` only, so it has no `style` or `render` prop yet."
        </p>
        <p class="api-props">"actionsRef Prop Example:"</p>
        {code_block(Lang::Rust, "", ACTIONS_REF_SNIPPET)}

        {api_part(
            "Form.Props",
            "Re-export of Form props.",
            "Props: the same set as Form above.",
        )}

        {api_part(
            "Form.State",
            "State: Form.State",
            "type FormState = {}; — the component's own state object is empty: the form's state lives in the field registry it coordinates.",
        )}

        {api_part(
            "Form.Actions",
            "State: Form.Actions",
            "type FormActions = { validate: (fieldName?: string) => void };",
        )}

        {api_part(
            "Form.SubmitEventDetails",
            "State: Form.SubmitEventDetails",
            "type FormSubmitEventDetails = { reason: 'none' (the reason for the event); event: Event (the native event associated with the custom event) };",
        )}

        {api_part(
            "Form.SubmitEventReason",
            "State: Form.SubmitEventReason",
            "type FormSubmitEventReason = 'none';",
        )}

        {api_part(
            "Form.ValidationMode",
            "State: Form.ValidationMode",
            "type FormValidationMode = 'onSubmit' | 'onBlur' | 'onChange';",
        )}

        {api_part(
            "Form.Values",
            "State: Form.Values",
            "type FormValues = Record<string, any>;",
        )}

        <h2>"Canonical Types"</h2>
        <p>
            "Maps `Canonical`: `Alias` — Use Canonical when its namespace is already imported; otherwise use Alias."
        </p>
        <ul>
            <li>"Form.Props: FormProps"</li>
            <li>"Form.State: FormState"</li>
            <li>"Form.Actions: FormActions"</li>
            <li>"Form.ValidationMode: FormValidationMode"</li>
            <li>"Form.SubmitEventReason: FormSubmitEventReason"</li>
            <li>"Form.SubmitEventDetails: FormSubmitEventDetails"</li>
        </ul>
    }
}

/// The `docs/src/app/(docs)/react/components/form/page.mdx` page.
#[component]
pub fn FormPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Form"</h1>
            <p class="subtitle">"A native form element with consolidated error handling."</p>

            <div class="docs-demo" data-demo="hero"><FormHeroDemo delay_ms=1000 /></div>

            <h2>"Anatomy"</h2>
            <p>
                "Form is composed together with "
                <a href="/react/components/field">"Field"</a>
                ". Import the components and place them together:"
            </p>
            {code_block(Lang::Rust, "Anatomy", ANATOMY_SNIPPET)}

            <h2>"Examples"</h2>

            <h3>"Submit with a Server\u{a0}Function"</h3>
            <p>
                "Upstream's React docs submit this demo with a server function, through React DOM's "
                "`useActionState`, instead of `onSubmit`. Server functions are an upstream React DOM feature with "
                "no counterpart in this Rust/Leptos port, so this mirror keeps the demo's markup and "
                "interaction and credits the upstream-only path here rather than teaching it as this "
                "port's own API."
            </p>
            <div class="docs-demo" data-demo="form-action"><FormActionDemo delay_ms=1000 /></div>

            <h3>"Submit form values as a JavaScript\u{a0}object"</h3>
            <p>
                "You can use `onFormSubmit` instead of the native `onSubmit` to access form values as a JavaScript object. This is useful when you need to transform the values before submission, or integrate with 3rd party APIs."
            </p>
            {code_block(Lang::Rust, "Submission using onFormSubmit", ON_FORM_SUBMIT_SNIPPET)}
            <p>"When used, `preventDefault` is called on the native submit event."</p>

            <h3>"Using with Zod"</h3>
            <p>
                "When parsing the schema using `schema.safeParse()`, the `z.flattenError(result.error).fieldErrors` data can be used to map the errors to each field's `name`."
            </p>
            <div class="docs-demo" data-demo="zod"><FormZodDemo /></div>

            <FormApiReference />
        </article>
    }
}

// ---------------------------------------------------------------------------
// The page's snippets teach the PORT (`specs/docs-content/CONTRACT.md` req 1)
// ---------------------------------------------------------------------------
//
// Same guard as the checkbox/button/accordion/field/fieldset/meter pages: this page's three blocks
// were upstream's own fences (the mirrored page's `import { Field }` / `import { Form }` lines, the `async`
// `onFormSubmit` handler, `actionsRef.current?.validate(…)`) while every structural gate passed. The
// classifier assertion reads the same rules the gap report's browser probe injects; the `_shape`
// functions compile the compositions the snippets teach (never called — the compiler is the
// assertion; the page's real compositions are exercised by `render_test.rs`).
#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use crate::snippet_language::{SnippetLanguage, classify};
    use std::rc::Rc;

    use leptos_ui::field_control::FieldControl as FieldControlPart;
    use leptos_ui::field_parts::FieldLabel as FieldLabelPart;
    use leptos_ui::field_root::FieldRoot as FieldRootPart;
    use leptos_ui::{Field, FormActionsRef, FormSubmitEventDetails, FormValues};

    /// Upstream's blocks (`page.mdx:18-28`, `:45-60`) as the classifier's positive controls, so the
    /// assertions below cannot pass vacuously if `looks_react` ever stops recognising upstream's
    /// source. The package specifier is left out (page-source, not reader-facing). Upstream's third
    /// example — `types.md`'s bare `actionsRef.current?.validate(…)` lines — is deliberately NOT among
    /// them: it carries no lexical React marker at all, so the probe scores it `other`, and asserting
    /// `React` for it would be asserting something the classifier does not measure.
    const UPSTREAM_SHAPES: [&str; 2] = [
        "<Form>\n  <Field.Root>\n    <Field.Label />\n  </Field.Root>\n</Form>;",
        "<Form onFormSubmit={async (formValues) => {\n  const response = await fetch('https://api.example.com', {\n    method: 'POST',\n  });\n}} />;",
    ];

    #[test]
    fn the_classifier_recognises_upstream_source() {
        for (i, shape) in UPSTREAM_SHAPES.iter().enumerate() {
            assert_eq!(
                classify(shape),
                SnippetLanguage::React,
                "positive control {i} is no longer detected as upstream's source — the assertions \
                 below would be vacuous"
            );
        }
    }

    /// Every snippet this page embeds, in document order, with the language `CONTRACT.md`
    /// requirement 1 requires of it. All three are the port's own code; the `actionsRef` fragment is
    /// an imperative example with no JSX at all, and it still classifies as the port's because it
    /// reads the port's `FormActionsRef` slot.
    fn page_snippets() -> [(&'static str, &'static str, SnippetLanguage); 3] {
        [
            ("Anatomy", ANATOMY_SNIPPET, SnippetLanguage::Leptos),
            (
                "Submission using onFormSubmit",
                ON_FORM_SUBMIT_SNIPPET,
                SnippetLanguage::Leptos,
            ),
            (
                "actionsRef example",
                ACTIONS_REF_SNIPPET,
                SnippetLanguage::Leptos,
            ),
        ]
    }

    /// The page-level number this item's own done-when names: `{total: 3, leptos: 3, react: 0,
    /// other: 0}` — the same triple `visual-gap-report.mjs`'s in-browser probe reports. Pinned as one
    /// ordered comparison so a re-ordering or a re-classified block fails with both sides visible.
    #[test]
    fn the_pages_snippets_all_teach_the_port() {
        let languages: Vec<(&str, SnippetLanguage)> = page_snippets()
            .iter()
            .map(|(name, text, _)| (*name, classify(text)))
            .collect();
        assert_eq!(
            languages,
            vec![
                ("Anatomy", SnippetLanguage::Leptos),
                ("Submission using onFormSubmit", SnippetLanguage::Leptos),
                ("actionsRef example", SnippetLanguage::Leptos),
            ],
            "the probe must read {{total: 3, leptos: 3, react: 0, other: 0}} for this page, in \
             document order"
        );
    }

    /// `CONTRACT.md` requirement 1's mapping table: upstream composes `Form` with `Field.Root` /
    /// `Field.Label` / `Field.Control` / `Field.Error`, so this port's spelling is the same tree with
    /// `::` — never the flattened `<FieldRoot>`. The `actionsRef` fragment composes no parts at all,
    /// so only the snippets that name `Field` are asserted to use the namespaced form; every snippet
    /// is asserted not to carry the flattened one.
    #[test]
    fn every_snippet_uses_the_namespaced_spelling() {
        for (name, text, _) in page_snippets() {
            if text.contains("Field") {
                assert!(
                    text.contains("<Field::"),
                    "the '{name}' snippet composes Field but not through the namespaced <Field::…> \
                     spelling (CONTRACT.md requirement 1)"
                );
            }
            for flattened in ["<FieldRoot", "<FieldLabel", "<FieldControl", "<FieldError"] {
                assert!(
                    !text.contains(flattened),
                    "the '{name}' snippet still spells {flattened}> (the flattened form is not the \
                     teaching surface — CONTRACT.md requirement 1)"
                );
            }
        }
    }

    // --- the snippets' shapes, compiled -------------------------------------------------------

    /// The Anatomy snippet: Form composed with the port's `Field::*` parts.
    #[allow(dead_code)]
    fn anatomy_snippet_shape() -> impl IntoView {
        view! {
            <Form>
                <Field::Root>
                    <Field::Label>"Quantity"</Field::Label>
                    <Field::Control />
                    <Field::Error>"Required"</Field::Error>
                </Field::Root>
            </Form>
        }
    }

    /// The `onFormSubmit` snippet: the values record the port hands the handler is
    /// `FormValues = Vec<(String, serde_json::Value)>`, and the handler is an
    /// `Rc<dyn Fn(FormValues, FormSubmitEventDetails)>` the `Form` prop accepts.
    #[allow(dead_code)]
    fn on_form_submit_snippet_shape() -> impl IntoView {
        let on_form_submit: Rc<dyn Fn(FormValues, FormSubmitEventDetails)> = Rc::new(
            move |form_values: FormValues, _details: FormSubmitEventDetails| {
                let value = |name: &str| {
                    form_values
                        .iter()
                        .find(|(key, _)| key == name)
                        .map(|(_, value)| value.clone())
                };

                let _payload = serde_json::json!({
                    "product_id": value("id"),
                    "order_quantity": value("quantity"),
                });
            },
        );

        view! {
            <Form on_form_submit=on_form_submit>
                <Field::Root>
                    <Field::Label>"Quantity"</Field::Label>
                    <Field::Control />
                </Field::Root>
            </Form>
        }
    }

    /// The `actionsRef` snippet: the slot type the `Form` writes at materialization and the two
    /// `FormActions::validate` arms the example shows.
    #[allow(dead_code)]
    fn actions_ref_snippet_shape(actions_ref: FormActionsRef) -> impl IntoView {
        let actions = actions_ref.take().expect("the Form mounted");
        actions.validate(None);
        actions.validate(Some("email"));
        view! { <span /> }
    }

    #[test]
    fn the_snippets_compile_against_the_ports_surface() {
        let _ = (
            anatomy_snippet_shape,
            on_form_submit_snippet_shape,
            actions_ref_snippet_shape,
            FieldControlPart,
            FieldLabelPart,
            FieldRootPart,
        );
    }
}
