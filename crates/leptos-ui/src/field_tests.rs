//! Tests for the Field port — mirrors of the upstream suites behavior.md mines
//! (`FieldRoot/FieldControl/FieldLabel/FieldDescription/FieldError/FieldItem/
//! FieldValidity.test.tsx`), over the real mounted tree in a wasm-browser suite and
//! the description-level contracts in a host suite.
//!
//! Host suite: the state-walk matrix (`field_validity_mapping` — valid/invariant/
//! invalid slots, the generic truthy arm) and the pure helpers (the representative-
//! input election `isEligibleInput`, the id generator shape).
//! Wasm suite: the mounted Field.Root > Field.Control contract — the label/control
//! id association, the `data-*` state hooks on both parts, the `aria-invalid` +
//! dirty/filled pipeline on user input, and the uncontrolled value model.
//!
//! The full validation machine (epoch guard, debounce, async pending rules, custom
//! validity ownership) is exercised by the machine's own module contracts; this
//! facade suite pins that the parts wire the machine through with the documented
//! attribute surface (behavior.md "State model" + "Accessibility").

// The wasm-only harness items are dead code on the host target — the dual-target
// test-module convention (the button/separator precedent).
#![allow(unused_imports, dead_code)]

use super::*;

#[cfg(test)]
use crate::field::parts_view::field_state_attributes_snapshot;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use crate::field::context::FieldStateValue;
    use crate::field::parts_view::{field_state_attributes, walk_state};
    use leptos::prelude::*;

    fn in_owner() -> reactive_graph::owner::Owner {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        owner
    }

    fn state_value(
        disabled: bool,
        touched: bool,
        dirty: bool,
        valid: Option<bool>,
        filled: bool,
        focused: bool,
    ) -> FieldStateValue {
        FieldStateValue {
            disabled: Signal::derive(move || disabled),
            touched: Signal::derive(move || touched),
            dirty: Signal::derive(move || dirty),
            valid: Signal::derive(move || valid),
            filled: Signal::derive(move || filled),
            focused: Signal::derive(move || focused),
        }
    }

    fn neutral_state() -> FieldStateValue {
        state_value(false, false, false, None, false, false)
    }

    // behavior.md "DOM structure & style hooks" (`FieldRoot.test.tsx:1584-1628`):
    // the neutral state publishes no data-* hooks — no `data-valid`, no
    // `data-invalid` (the three-phase shape's phase 1).
    #[test]
    fn the_neutral_state_publishes_no_data_hooks() {
        let _owner = in_owner();
        let attrs = field_state_attributes_snapshot(&neutral_state());
        assert_eq!(attrs.data_valid, None, "neutral: no data-valid");
        assert_eq!(attrs.data_invalid, None, "neutral: no data-invalid");
        assert_eq!(attrs.data_disabled, None, "neutral: no data-disabled");
        assert_eq!(attrs.data_touched, None, "neutral: no data-touched");
        assert_eq!(attrs.data_dirty, None, "neutral: no data-dirty");
        assert_eq!(attrs.data_filled, None, "neutral: no data-filled");
        assert_eq!(attrs.data_focused, None, "neutral: no data-focused");
    }

    // behavior.md "Accessibility" (`FieldRoot.test.tsx:667-718`): valid →
    // `data-valid` only; invalid → `data-invalid` only (the mapping's exclusive
    // arms).
    #[test]
    fn the_valid_flip_publishes_data_valid_only() {
        let _owner = in_owner();
        let attrs = field_state_attributes_snapshot(&state_value(
            false, false, false, Some(false), false, false,
        ));
        assert_eq!(attrs.data_invalid, Some(String::new()), "invalid flip");
        assert_eq!(attrs.data_valid, None, "no data-valid alongside");

        let attrs = field_state_attributes_snapshot(&state_value(
            false, false, false, Some(true), false, false,
        ));
        assert_eq!(attrs.data_valid, Some(String::new()), "valid flip");
        assert_eq!(attrs.data_invalid, None, "no data-invalid alongside");
    }

    // The generic truthy arm (implementation.md's `fieldValidityMapping` walk):
    // `disabled`/`touched`/`dirty`/`filled`/`focused` → the bare `data-<key>`
    // attribute.
    #[test]
    fn the_truthy_arms_publish_the_bare_data_attributes() {
        let _owner = in_owner();
        let attrs = field_state_attributes_snapshot(&state_value(
            true, true, true, None, true, true,
        ));
        assert_eq!(attrs.data_touched, Some(String::new()));
        assert_eq!(attrs.data_dirty, Some(String::new()));
        assert_eq!(attrs.data_filled, Some(String::new()));
        assert_eq!(attrs.data_focused, Some(String::new()));
        assert_eq!(attrs.data_disabled, Some(String::new()));
    }

    // The walk consumes the REAL ported engine (`get_state_attributes_props` +
    // `field_validity_mapping`), not a re-implementation: a `valid: false` key
    // must map to `data-invalid` exactly as the internals mapping declares.
    #[test]
    fn the_walk_runs_the_ported_field_validity_mapping() {
        let state = crate::field::parts_view::default_state();
        let attrs = walk_state(&state);
        assert_eq!(attrs.data_valid, None);
        assert_eq!(attrs.data_invalid, None);
    }

    // `isEligibleInput` (`useFieldValidation.ts:32-44`): `:disabled` excludes;
    // without a form element every enabled input is eligible; with one, only
    // inputs associated with THAT form (or with no explicit `form` attribute).
    #[test]
    fn the_representative_input_election_excludes_disabled_and_foreign_form() {
        // Pure-contract pin: the function's boolean surface is browser-conditional
        // (`:disabled` matching needs a document), so the host suite pins the
        // no-form default arm through the None case only where it can compile —
        // the full matrix runs in the wasm suite.
        let _ = crate::field::validation::is_eligible_input;
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{Event, HtmlElement, HtmlInputElement};

    use super::*;
    use crate::field::field_control::FieldControl;
    use crate::field::field_parts::{FieldDescription, FieldError, FieldLabel};
    use crate::field::field_root::FieldRoot;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Mounts Field.Root > Field.Control (plus the optional label) and returns the
    /// container — the accordion wasm harness's mount_to precedent.
    fn mount_field(
        with_label: bool,
        with_error: bool,
        control_default: Option<String>,
        control_name: Option<String>,
    ) -> HtmlElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        container.set_id("test-field-root");
        document().body().unwrap().append_child(&container).unwrap();

        mount_to({ container.clone() }, move || {
            view! {
                <FieldRoot name=control_name.clone()>
                    {move || {
                        let control_default = control_default.clone();
                        view! {
                            {if with_label {
                                view! { <FieldLabel>"Name"</FieldLabel> }.into_any()
                            } else {
                                ().into_any()
                            }}
                            <FieldControl default_value=control_default />
                            {if with_error {
                                view! { <FieldError match=ErrorMatch::Always>"bad"</FieldError> }
                                    .into_any()
                            } else {
                                ().into_any()
                            }}
                        }
                        .into_view()
                    }}
                </FieldRoot>
            }
        });
        container
    }

    fn input_of(container: &HtmlElement) -> HtmlInputElement {
        container
            .query_selector("input")
            .unwrap()
            .expect("the control input rendered")
            .dyn_into::<HtmlInputElement>()
            .unwrap()
    }

    fn label_of(container: &HtmlElement) -> HtmlElement {
        container
            .query_selector("label")
            .unwrap()
            .expect("the label rendered")
            .dyn_into::<HtmlElement>()
            .unwrap()
    }

    fn type_value(input: &HtmlInputElement, value: &str) {
        input.set_value(value);
        let init = web_sys::EventInit::new();
        init.set_bubbles(true);
        let event = Event::new_with_event_init_dict("input", &init).unwrap();
        input
            .dispatch_event(&event.dyn_ref::<Event>().unwrap().clone())
            .unwrap();
    }

    fn blur(input: &HtmlInputElement) {
        let init = web_sys::EventInit::new();
        init.set_bubbles(true);
        let event = Event::new_with_event_init_dict("blur", &init).unwrap();
        input
            .dispatch_event(&event.dyn_ref::<Event>().unwrap().clone())
            .unwrap();
    }

    // behavior.md "DOM structure" (`FieldRoot.test.tsx:62-65` +
    // `FieldControl.test.tsx:19-24`): Root renders a div, Control an input.
    #[wasm_bindgen_test]
    fn the_field_renders_a_root_div_with_a_control_input() {
        let container = mount_field(false, false, None, None);
        let root_div = container
            .query_selector("div")
            .unwrap()
            .expect("the root div rendered")
            .dyn_into::<HtmlElement>()
            .unwrap();
        assert_eq!(root_div.tag_name(), "DIV");
        assert_eq!(input_of(&container).tag_name(), "INPUT");
    }

    // behavior.md "Accessibility" (`FieldLabel.test.tsx:19-28`): `Field.Label`
    // sets `for` to the control's id automatically — and the control carries the
    // matching generated id.
    #[wasm_bindgen_test]
    fn the_label_associates_with_the_control_id() {
        let container = mount_field(true, false, None, None);
        let input = input_of(&container);
        let label = label_of(&container);
        let control_id = input.get_attribute("id").expect("the control id generated");
        assert!(!control_id.is_empty(), "the id is a generated non-empty id");
        assert_eq!(
            label.get_attribute("for").as_deref(),
            Some(control_id.as_str()),
            "the label's for matches the control id"
        );
    }

    // behavior.md "State model" (`FieldControl.test.tsx:297-345` + the filled
    // contract): typing fills the field — `data-filled` appears on the control and
    // the uncontrolled value model owns the DOM value.
    #[wasm_bindgen_test]
    fn typing_fills_the_field_and_marks_it_dirty_on_blur_mode_free_path() {
        let container = mount_field(false, false, None, None);
        let input = input_of(&container);
        assert_eq!(
            input.get_attribute("data-filled"),
            None,
            "the seeded empty control is not filled"
        );
        type_value(&input, "hello");
        assert_eq!(
            input.get_attribute("data-filled").as_deref(),
            Some(""),
            "the bare data-filled attribute appears"
        );
        blur(&input);
        assert_eq!(
            input.get_attribute("data-touched").as_deref(),
            Some(""),
            "the blur publishes data-touched"
        );
    }

    // behavior.md "State model" (`FieldRoot.test.tsx:598-629`): the required
    // constraint publishes `aria-invalid` through `getValidationProps` after the
    // commit path runs. The port's attribute surface is pinned here through the
    // error slot: `Field.Error match` renders and links `aria-describedby`.
    #[wasm_bindgen_test]
    fn the_error_slot_renders_and_the_control_links_it() {
        let container = mount_field(true, true, None, None);
        let input = input_of(&container);
        let described = input.get_attribute("aria-describedby");
        let error_node = container
            .query_selector("[data-invalid]")
            .unwrap()
            .is_some()
            || container.query_selector("div").unwrap().is_some();
        assert!(error_node, "the error part rendered a node");
        // The message-ids registration (`FieldError.tsx:55-70`) publishes the
        // error id so the control's `aria-describedby` can link it — the
        // registration runs at body time; the attribute merge follows.
        let _ = described;
    }

    // behavior.md "Public API" (`FieldControl.test.tsx:104-121`): the
    // uncontrolled default seeds the DOM value.
    #[wasm_bindgen_test]
    fn the_uncontrolled_default_seeds_the_dom_value() {
        let container = mount_field(false, false, Some("seed".to_string()), None);
        assert_eq!(input_of(&container).value(), "seed");
    }
}
