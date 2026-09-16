//! Part-surface tests — ONE per module, each exercising that module's namespaced path.
//!
//! THE CLAIM
//! ---------
//! Upstream's docs teach `<Checkbox.Root><Checkbox.Indicator /></Checkbox.Root>`; the port's
//! spelling is the same tree with Rust's path separator, `<Checkbox::Root><Checkbox::Indicator />`.
//! `specs/docs-content/CONTRACT.md` (requirement 1) makes that the surface every mirrored page must
//! teach, `crates/leptos-ui/tests/ns_component_path.rs` pins that the `view!` macro accepts a
//! path-form component name at all, and `ralph/scripts/check-part-surface.mjs` counts, per mined
//! spec, whether each `Component.Part` exists as a capitalised public item inside the component's
//! module.
//!
//! This file is the per-module half of that contract, and it is deliberately a COMPILE pin rather
//! than a render assertion: what the docs examples need is not that `Root` renders but that a
//! reader can write `<Field::Label>` and get the port's Field.Label — the same shape upstream
//! teaches. Each test therefore (a) names every documented part of its module through the
//! capitalised path, and (b) refers to a `#[component]` whose body assembles the parts in `view!`
//! markup, so a part that loses its namespaced spelling, its props, or its attribute surface fails
//! to compile here, at the line the docs would write.
//!
//! Modules whose documented surface is a TYPE rather than a view part (Form: `Form.Props`,
//! `Form.Values`, `Form.Actions` — upstream's `Form` has no subcomponents) are pinned by name in
//! the same way.
//!
//! COVERAGE, EXACTLY (the batch's 14 components):
//!
//! * 9 modules pin namespaced view parts here: accordion (5), avatar (3), checkbox (2),
//!   collapsible (3), field (7), fieldset (2), meter (5), progress (5) — 32 view parts — plus form's
//!   3 dotted type names.
//! * 4 components document NO part of their own and so have no namespaced path to pin: `button`,
//!   `checkbox-group` (upstream's `<CheckboxGroup>` is a single component — `page.mdx:23-27` — whose
//!   only dotted references are to `Checkbox.*`/`Field.*`, other units), `separator` and `toggle`
//!   (`grep -oE '`[A-Z][A-Za-z0-9]*\.[A-Z][A-Za-z0-9]*`'` over each `behavior.md` returns nothing).
//! * `otp-field` is the one component with documented parts that are NOT here: its spec requires
//!   `OTPField.Root`/`Input`/`Separator` (`specs/library/otp-field/behavior.md:14-20`) but the crate
//!   exposes that unit as `RenderedElement` builders with no view layer, so the parts cannot be
//!   written in `view!` markup at all. That is a pending ledger item
//!   (`library: otp-field — the namespaced view surface`), not a gap this file can paper over.
//!
//! Runtime behaviour is NOT this file's job: it lives in the per-unit suites
//! (`crates/leptos-ui/src/<component>_tests.rs`) which mount the real parts in Chrome.

use leptos::prelude::*;
use leptos_ui::field::field_parts::FieldValidityPayload;
use leptos_ui::{
    Accordion, Avatar, Checkbox, Collapsible, Field, Fieldset, Form, Meter, Progress,
};

// ---------------------------------------------------------------------------
// accordion
// ---------------------------------------------------------------------------

#[component]
fn AccordionPin() -> impl IntoView {
    view! {
        <Accordion::Root>
            <Accordion::Item>
                <Accordion::Header>
                    <Accordion::Trigger>"Toggle"</Accordion::Trigger>
                </Accordion::Header>
                <Accordion::Panel>"Panel content"</Accordion::Panel>
            </Accordion::Item>
        </Accordion::Root>
    }
}

#[test]
fn accordion_exposes_its_five_documented_parts_namespaced() {
    let _ = (
        Accordion::Root,
        Accordion::Item,
        Accordion::Header,
        Accordion::Trigger,
        Accordion::Panel,
    );
    let _ = AccordionPin;
}

// ---------------------------------------------------------------------------
// avatar
// ---------------------------------------------------------------------------

#[component]
fn AvatarPin() -> impl IntoView {
    view! {
        <Avatar::Root>
            <Avatar::Image src="/avatar.png".to_string() />
            <Avatar::Fallback inner_html="AP".to_string() />
        </Avatar::Root>
    }
}

#[test]
fn avatar_exposes_its_three_documented_parts_namespaced() {
    let _ = (Avatar::Root, Avatar::Image, Avatar::Fallback);
    let _ = AvatarPin;
}

// ---------------------------------------------------------------------------
// checkbox
// ---------------------------------------------------------------------------

#[component]
fn CheckboxPin() -> impl IntoView {
    view! {
        <Checkbox::Root checked=true native_button=true>
            <Checkbox::Indicator />
        </Checkbox::Root>
    }
}

#[test]
fn checkbox_exposes_its_two_documented_parts_namespaced() {
    let _ = (Checkbox::Root, Checkbox::Indicator);
    let _ = CheckboxPin;
}

// ---------------------------------------------------------------------------
// collapsible
// ---------------------------------------------------------------------------

#[component]
fn CollapsiblePin() -> impl IntoView {
    view! {
        <Collapsible::Root>
            <Collapsible::Trigger>"Toggle"</Collapsible::Trigger>
            <Collapsible::Panel>"Panel content"</Collapsible::Panel>
        </Collapsible::Root>
    }
}

#[test]
fn collapsible_exposes_its_three_documented_parts_namespaced() {
    let _ = (Collapsible::Root, Collapsible::Trigger, Collapsible::Panel);
    let _ = CollapsiblePin;
}

// ---------------------------------------------------------------------------
// field
// ---------------------------------------------------------------------------

#[component]
fn FieldPin() -> impl IntoView {
    view! {
        <Field::Root>
            <Field::Label>"Name"</Field::Label>
            <Field::Control />
            <Field::Description>"Your name"</Field::Description>
            <Field::Item>"item"</Field::Item>
            <Field::Error>"Required"</Field::Error>
        </Field::Root>
    }
}

#[test]
fn field_exposes_its_seven_documented_parts_namespaced() {
    let _ = (
        Field::Root,
        Field::Control,
        Field::Label,
        Field::Description,
        Field::Item,
        Field::Error,
        Field::Validity,
    );
    let _ = FieldPin;
}

// `Field.Validity`'s children are upstream's RENDER FUNCTION (`FieldValidity.tsx:20-23`): the
// payload it receives is the combined validity record, not a view body, so the namespaced part is
// written with `children=` rather than as an element subtree — the one part in this batch whose
// `view!` spelling differs from the JSX shape for a documented reason.
#[component]
fn FieldValidityPin() -> impl IntoView {
    view! {
        <Field::Root>
            <Field::Validity children=Box::new(|_payload: FieldValidityPayload| ().into_any()) />
        </Field::Root>
    }
}

#[test]
fn field_validity_takes_a_render_function() {
    let _ = FieldValidityPin;
}

// ---------------------------------------------------------------------------
// fieldset
// ---------------------------------------------------------------------------

#[component]
fn FieldsetPin() -> impl IntoView {
    view! {
        <Fieldset::Root>
            <Fieldset::Legend>"Shipping"</Fieldset::Legend>
        </Fieldset::Root>
    }
}

#[test]
fn fieldset_exposes_its_two_documented_parts_namespaced() {
    let _ = (Fieldset::Root, Fieldset::Legend);
    let _ = FieldsetPin;
}

// ---------------------------------------------------------------------------
// form — a single component; its dotted names are TYPES, not view parts
// ---------------------------------------------------------------------------

#[test]
fn form_exposes_its_dotted_type_names() {
    // Upstream: `Form.Props['errors']` (`Form.test.tsx:664`), `Form.Values`
    // (`Form.test.tsx:213-215`), `Form.Actions` (`Form.test.tsx:1048`). The port spells the same
    // three names with `::`; `Form` itself has no subcomponents (behavior.md "Public API surface").
    fn _takes_props(_: Form::Props) {}
    let _values = core::any::type_name::<Form::Values>();
    let _actions = core::any::type_name::<Form::Actions>();
}

// ---------------------------------------------------------------------------
// meter
// ---------------------------------------------------------------------------

#[component]
fn MeterPin() -> impl IntoView {
    view! {
        <Meter::Root value=50.0>
            <Meter::Label>"Storage"</Meter::Label>
            <Meter::Track>
                <Meter::Indicator />
            </Meter::Track>
            <Meter::Value />
        </Meter::Root>
    }
}

#[test]
fn meter_exposes_its_five_documented_parts_namespaced() {
    let _ = (
        Meter::Root,
        Meter::Label,
        Meter::Track,
        Meter::Indicator,
        Meter::Value,
    );
    let _ = MeterPin;
}

// ---------------------------------------------------------------------------
// progress
// ---------------------------------------------------------------------------

#[component]
fn ProgressPin() -> impl IntoView {
    view! {
        <Progress::Root value=Some(50.0)>
            <Progress::Label>"Loading"</Progress::Label>
            <Progress::Track>
                <Progress::Indicator />
            </Progress::Track>
            <Progress::Value />
        </Progress::Root>
    }
}

#[test]
fn progress_exposes_its_five_documented_parts_namespaced() {
    let _ = (
        Progress::Root,
        Progress::Label,
        Progress::Track,
        Progress::Indicator,
        Progress::Value,
    );
    let _ = ProgressPin;
}
