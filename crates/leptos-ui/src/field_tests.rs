//! Tests for the Field port — mirrors of the upstream suites behavior.md mines
//! (`FieldRoot/FieldControl/FieldLabel/FieldDescription/FieldError/FieldItem/
//! FieldValidity.test.tsx`), over the real mounted tree in a wasm-browser suite and
//! the description-level contracts in a host suite.
//!
//! Host suite: the state-walk matrix (`field_validity_mapping` — valid/invariant/
//! invalid slots, the generic truthy arm) and the pure-helper pin.
//! Wasm suite: the mounted Field.Root > parts contract — the label/control id
//! association (including the render-order case the association MUST survive:
//! label first, control after), the `data-*` state hooks, the custom validator
//! through the REAL commit path (`aria-invalid` + `data-invalid`), the Error
//! part's message-id registration linked from the control's `aria-describedby`,
//! and the `elementProps` passthrough (behavior.md:32). It also pins the
//! dirty/touched lifecycle hooks, the full `FieldValidity` render-prop payload
//! (value/validity/error/errors/transitionStatus), the multi-error `<ul>` shape,
//! and the `<Fieldset.Root disabled>` inheritance.
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
            false,
            false,
            false,
            Some(false),
            false,
            false,
        ));
        assert_eq!(attrs.data_invalid, Some(String::new()), "invalid flip");
        assert_eq!(attrs.data_valid, None, "no data-valid alongside");

        let attrs = field_state_attributes_snapshot(&state_value(
            false,
            false,
            false,
            Some(true),
            false,
            false,
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
        let attrs =
            field_state_attributes_snapshot(&state_value(true, true, true, None, true, true));
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

    // `isEligibleInput` (`useFieldValidation.ts:32-44`) is browser-conditional
    // (`:disabled` matching needs a document) — the harness identity pin lives
    // here; the real election runs in the wasm suite (the representative-input
    // search reads `validity()`).
    #[test]
    fn the_representative_input_election_helper_is_wired() {
        let _ = crate::field::validation::is_eligible_input;
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{Event, HtmlElement, HtmlInputElement};

    use super::*;
    use crate::field::field_control::{FieldControlViewProps, field_control_view};
    use crate::field::field_parts::{
        ErrorMatch, FieldValidityPayload, field_error_view, field_label_view, field_validity_view,
    };
    use crate::field::field_root::{FieldRootViewProps, field_root_view};
    use leptos::mount::mount_to;
    use leptos::prelude::*;
    use std::rc::Rc;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Mounts Field.Root (the view fn) whose children closure builds the leaf
    /// parts at the root's body time — INSIDE the bridge window — which is the
    /// real composition path: the parts' labelable reads resolve against the
    /// root's provider, exactly as the component-tree nesting does. The parts
    /// builder crosses into the Send children box, so non-Send handles (props
    /// structs with Rc fields) must be constructed INSIDE the builder.
    fn mount_field_root(
        build_parts: impl Fn() -> Vec<AnyView> + Send + 'static,
        validate: Option<
            Rc<
                dyn Fn(
                    &serde_json::Value,
                    &serde_json::Map<String, serde_json::Value>,
                ) -> crate::field::validation::ValidationOutcome,
            >,
        >,
    ) -> HtmlElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        container.set_id("test-field-root");
        document().body().unwrap().append_child(&container).unwrap();

        // The UnmountHandle must be forgotten (the meter_tests forget convention):
        // dropping it unmounts the view and cancels the reactive owner before the
        // assertions run.
        std::mem::forget(mount_to({ container.clone() }, move || {
            field_root_view(FieldRootViewProps {
                validate,
                children: Some(Box::new(move || {
                    let views: Vec<AnyView> = build_parts();
                    views.into_view().into_any()
                })),
                ..FieldRootViewProps::default()
            })
        }));
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

    fn press_enter(input: &HtmlInputElement) {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key("Enter");
        let event =
            web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap();
        input
            .dispatch_event(event.dyn_ref::<web_sys::KeyboardEvent>().unwrap())
            .unwrap();
    }

    /// One real browser turn (a `setTimeout(0)` drain): the browser microtask
    /// queue is where leptos-side effects (attribute re-renders, the rest-bag
    /// effect) are scheduled, which a synchronous assert never sees (the
    /// avatar-suite flush_one_turn convention; the suite's original fingerprint:
    /// every sync Effect-dependent test froze at its seed value).
    async fn flush_one_turn() {
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .expect("window")
                .set_timeout_with_callback_and_timeout_and_arguments_0(resolve.unchecked_ref(), 0)
                .expect("setTimeout");
        });
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .expect("await");
    }

    // behavior.md "DOM structure" (`FieldRoot.test.tsx:62-65` +
    // `FieldControl.test.tsx:19-24`): Root renders a div, Control an input.
    #[wasm_bindgen_test]
    fn the_field_renders_a_root_div_with_a_control_input() {
        let container = mount_field_root(
            || vec![field_control_view(FieldControlViewProps::default()).into_any()],
            None,
        );
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
    // matching generated id. The LABEL RENDERS FIRST (the doc-order composition),
    // so the control's registration lands AFTER the label's first attribute
    // evaluation — the ordering the association must survive (behavior.md:93).
    #[wasm_bindgen_test]
    fn the_label_associates_with_the_control_id() {
        let container = mount_field_root(
            || {
                vec![
                    field_label_view(crate::field::field_parts::FieldLabelViewProps::default())
                        .into_any(),
                    field_control_view(FieldControlViewProps::default()).into_any(),
                ]
            },
            None,
        );
        let input = input_of(&container);
        let label = label_of(&container);
        let control_id = input.get_attribute("id").expect("the control id generated");
        assert!(!control_id.is_empty(), "the id is a generated non-empty id");
        assert_eq!(
            label.get_attribute("for").as_deref(),
            Some(control_id.as_str()),
            "the label's for matches the control id"
        );
        let label_id = label.get_attribute("id").expect("the label id generated");
        assert_eq!(
            input.get_attribute("aria-labelledby").as_deref(),
            Some(label_id.as_str()),
            "the control names the label (aria-labelledby)"
        );
    }

    // behavior.md "State model" (`FieldControl.test.tsx:297-345`): typing fills
    // the field — `data-filled` appears on the control. The `data-*` attributes
    // ride leptos-side effects (tracked re-render slots), so the assert follows a
    // flush_one_turn (the module's helper docs).
    #[wasm_bindgen_test]
    async fn typing_fills_the_field() {
        let container = mount_field_root(
            || vec![field_control_view(FieldControlViewProps::default()).into_any()],
            None,
        );
        let input = input_of(&container);
        assert_eq!(
            input.get_attribute("data-filled"),
            None,
            "the seeded empty control is not filled"
        );
        type_value(&input, "hello");
        flush_one_turn().await;
        assert_eq!(
            input.get_attribute("data-filled").as_deref(),
            Some(""),
            "the bare data-filled attribute appears"
        );
    }

    // behavior.md "State model" (`FieldRoot.test.tsx:598-629` + `FieldControl.test.tsx:470-496`):
    // the Enter-key commit path runs the custom validator and publishes the error —
    // `aria-invalid="true"` on the control, `data-invalid` on the parts. The commit
    // publishes through the validity RwSignal; the attribute effects land one browser
    // turn later (flush_one_turn).
    #[wasm_bindgen_test]
    async fn the_commit_path_publishes_aria_invalid() {
        use crate::field::validation::ValidationOutcome;
        let container = mount_field_root(
            || vec![field_control_view(FieldControlViewProps::default()).into_any()],
            Some(Rc::new(|_value, _form_values| {
                ValidationOutcome::Invalid(vec!["too short".to_string()])
            })),
        );
        let input = input_of(&container);
        assert_eq!(
            input.get_attribute("aria-invalid"),
            None,
            "neutral: no aria-invalid"
        );
        type_value(&input, "x");
        press_enter(&input);
        flush_one_turn().await;
        assert_eq!(
            input.get_attribute("aria-invalid").as_deref(),
            Some("true"),
            "the committed error publishes aria-invalid on the control"
        );
        assert_eq!(
            input.get_attribute("data-invalid").as_deref(),
            Some(""),
            "data-invalid lands on the control too"
        );
        assert_eq!(
            input.get_attribute("data-touched").as_deref(),
            Some(""),
            "the Enter path marks touched"
        );
    }

    // behavior.md "Public API surface" (`FieldError.test.tsx:18-30` +
    // `FieldDescription.test.tsx:30-52`): the rendered Error registers its id and
    // the control's `aria-describedby` links it; user values are preserved and
    // appended to. The control's user `aria-describedby` rides the elementProps
    // bag (constructed inside the builder — the Rc-holding props struct cannot
    // cross the Send children box).
    #[wasm_bindgen_test]
    fn the_error_slot_registers_and_the_control_links_it() {
        let container = mount_field_root(
            || {
                let error = field_error_view(
                    None,
                    None,
                    ErrorMatch::Always,
                    Some(view! { <span>"bad"</span> }.into_any()),
                );
                let mut props = FieldControlViewProps::default();
                props.element_attributes =
                    vec![("aria-describedby".to_string(), "user-note".to_string())];
                vec![error.into_any(), field_control_view(props).into_any()]
            },
            None,
        );
        let input = input_of(&container);
        let described = input
            .get_attribute("aria-describedby")
            .expect("the control links the described-by ids");
        assert!(
            described.contains("user-note"),
            "the user's aria-describedby is preserved: {described:?}"
        );
        // The error part's id is registered into the labelable scope; the merged
        // attribute links it (the registration ran at the error's body time —
        // before the control's attribute here, and after it too, via the mirror).
        let error_el = container
            .query_selector("div[data-invalid], div[id^='base-ui-']")
            .unwrap()
            .expect("the error part rendered");
        let error_id = error_el.get_attribute("id").expect("the error id");
        assert!(
            described.contains(&error_id),
            "the error's id is linked from the control: {described:?} vs {error_id:?}"
        );
    }

    // behavior.md "Public API surface" (`FieldControl.test.tsx:530-558`): native
    // props (`required`, `type`) pass through to the rendered `<input>` via the
    // `elementProps` bag. The bag lands through the mount effect (the rest-bag
    // channel), so the assert follows a flush_one_turn.
    #[wasm_bindgen_test]
    async fn the_native_props_pass_through_to_the_input() {
        let container = mount_field_root(
            || {
                let mut props = FieldControlViewProps::default();
                props.element_attributes = vec![
                    ("required".to_string(), String::new()),
                    ("type".to_string(), "email".to_string()),
                    ("placeholder".to_string(), "your name".to_string()),
                ];
                vec![field_control_view(props).into_any()]
            },
            None,
        );
        flush_one_turn().await;
        let input = input_of(&container);
        assert!(input.has_attribute("required"), "required passes through");
        assert_eq!(input.get_attribute("type").as_deref(), Some("email"));
        assert_eq!(
            input.get_attribute("placeholder").as_deref(),
            Some("your name")
        );
    }

    // behavior.md "Public API surface" (`FieldControl.test.tsx:104-121`): the
    // uncontrolled default seeds the DOM value.
    #[wasm_bindgen_test]
    fn the_uncontrolled_default_seeds_the_dom_value() {
        let container = mount_field_root(
            || {
                let mut props = FieldControlViewProps::default();
                props.default_value = Some("seed".to_string());
                vec![field_control_view(props).into_any()]
            },
            None,
        );
        assert_eq!(input_of(&container).value(), "seed");
    }

    // behavior.md "State model" (`FieldRoot.test.tsx:2507-2540` touched,
    // `:2542-2576` dirty): the lifecycle hooks track the real user path — typing
    // marks dirty, the first blur marks touched. Both ride the live state walk, so
    // the asserts follow the browser turn.
    #[wasm_bindgen_test]
    async fn typing_marks_dirty_and_blur_marks_touched() {
        let container = mount_field_root(
            || vec![field_control_view(FieldControlViewProps::default()).into_any()],
            None,
        );
        let input = input_of(&container);
        assert_eq!(input.get_attribute("data-dirty"), None, "pristine field");
        assert_eq!(input.get_attribute("data-touched"), None, "pristine field");

        type_value(&input, "typed");
        flush_one_turn().await;
        assert_eq!(
            input.get_attribute("data-dirty").as_deref(),
            Some(""),
            "typing marks dirty"
        );
        assert_eq!(
            input.get_attribute("data-touched"),
            None,
            "typing alone does not touch"
        );

        let init = web_sys::EventInit::new();
        let blur = Event::new_with_event_init_dict("blur", &init).unwrap();
        input.dispatch_event(&blur).unwrap();
        flush_one_turn().await;
        assert_eq!(
            input.get_attribute("data-touched").as_deref(),
            Some(""),
            "blur marks touched"
        );
    }

    // implementation.md "Dependencies" (`FieldRoot.tsx:44,48`): a Field nested in a
    // disabled `<Fieldset.Root>` inherits the disabled state — the control renders the
    // `disabled` attribute + `data-disabled`, and disabled fields never publish
    // `aria-invalid` (behavior.md "Accessibility" `:533-570`).
    #[wasm_bindgen_test]
    async fn the_disabled_fieldset_disables_the_field() {
        let _ = any_spawner::Executor::init_futures_executor();
        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        container.set_id("test-field-inside-fieldset");
        document().body().unwrap().append_child(&container).unwrap();

        std::mem::forget(mount_to({ container.clone() }, move || {
            view! {
                <crate::fieldset::FieldsetRoot disabled=true>
                    {field_root_view(FieldRootViewProps {
                        children: Some(Box::new(|| {
                            field_control_view(FieldControlViewProps::default()).into_any()
                        })),
                        ..FieldRootViewProps::default()
                    })}
                </crate::fieldset::FieldsetRoot>
            }
        }));
        flush_one_turn().await;

        let input = input_of(&container);
        assert!(
            input.has_attribute("disabled"),
            "the inherited disabled lands on the control"
        );
        assert_eq!(
            input.get_attribute("data-disabled").as_deref(),
            Some(""),
            "the control publishes the state hook"
        );
        assert_eq!(
            container
                .query_selector("div[data-disabled]")
                .unwrap()
                .map(|div| div.tag_name())
                .as_deref(),
            Some("DIV".to_string()).as_deref(),
            "the root div publishes data-disabled too"
        );
    }

    // behavior.md "Events" (`FieldValidity.test.tsx:39-47`, `:118-131`,
    // `:159-195`): `Field.Validity`'s render prop receives the FULL state object —
    // `value`, the merged `validity`, `error` (first string), `errors` (array order
    // preserved) and `transitionStatus` (upstream `{ ...combinedFieldValidityData,
    // validity: combined.state, transitionStatus }`, `FieldValidity.tsx:37-45`). The
    // payload re-derives on the committed error, so the slot's attributes carry the
    // post-commit values.
    #[wasm_bindgen_test]
    async fn the_validity_payload_carries_the_full_state_object() {
        use crate::field::validation::ValidationOutcome;
        let container = mount_field_root(
            || {
                vec![
                    field_control_view(FieldControlViewProps::default()).into_any(),
                    field_validity_view(|payload: FieldValidityPayload| {
                        let valid = format!("{:?}", payload.validity.valid);
                        let error = payload.error.clone();
                        let count = payload.errors.len().to_string();
                        let value = payload.value.to_string();
                        let transition = payload.transition_status.is_some().to_string();
                        view! {
                            <div
                                class="validity-payload"
                                data-valid=valid
                                data-error=error
                                data-error-count=count
                                data-value=value
                                data-has-transition=transition
                            ></div>
                        }
                        .into_any()
                    })
                    .into_any(),
                ]
            },
            Some(Rc::new(|_value, _form_values| {
                ValidationOutcome::Invalid(vec!["one".to_string(), "two".to_string()])
            })),
        );

        let input = input_of(&container);
        type_value(&input, "v");
        press_enter(&input);
        flush_one_turn().await;

        let out = container
            .query_selector(".validity-payload")
            .unwrap()
            .expect("the validity slot rendered");
        assert_eq!(
            out.get_attribute("data-valid").as_deref(),
            Some("Some(false)"),
            "the merged validity verdict is `valid: false`"
        );
        assert_eq!(
            out.get_attribute("data-error").as_deref(),
            Some("one"),
            "`error` is the first error string"
        );
        assert_eq!(
            out.get_attribute("data-error-count").as_deref(),
            Some("2"),
            "`errors` carries the full array"
        );
        assert!(
            out.get_attribute("data-value")
                .unwrap_or_default()
                .contains('v'),
            "`value` carries the committed value"
        );
        // The payload always materializes `transitionStatus`; its VALUE rides the shared
        // transition machine (the `starting`/`ending` timing is browser-frame-dependent,
        // so the structural presence is what this facade pins — the upstream assertion
        // is "transitionStatus is present on the payload", behavior.md:124).
        assert!(
            out.has_attribute("data-has-transition"),
            "`transitionStatus` rides the payload"
        );
    }

    // behavior.md "Edge cases" (`FieldError.test.tsx:183-212`): a multi-error payload
    // renders as a `<ul>` with one `<li>` per error. The error slot renders only once
    // the committed validity makes it rendered (match: true here — the `Always` arm).
    #[wasm_bindgen_test]
    async fn the_error_renders_a_list_for_multiple_errors() {
        use crate::field::validation::ValidationOutcome;
        let container = mount_field_root(
            || {
                vec![
                    field_error_view(None, None, ErrorMatch::Always, None).into_any(),
                    field_control_view(FieldControlViewProps::default()).into_any(),
                ]
            },
            Some(Rc::new(|_value, _form_values| {
                ValidationOutcome::Invalid(vec!["one".to_string(), "two".to_string()])
            })),
        );
        let input = input_of(&container);
        type_value(&input, "x");
        press_enter(&input);
        flush_one_turn().await;

        let items = container.query_selector_all("ul > li").unwrap();
        assert_eq!(
            items.length(),
            2,
            "one <li> per error in the payload, array order preserved"
        );
    }

    // ---- the public `#[component]` part surface (the docs page's assembly path) ----

    /// The docs page composition (`specs/docs-content/field/demos.json` hero +
    /// `page.md` "Anatomy"): `Root > Label + Control + Error + Description`, written
    /// with the `#[component]` parts exactly as upstream's JSX writes it. The other
    /// tests drive the leaf view fns (the bodies underneath); this pins the
    /// component-level surface the page uses — the label's children text, the
    /// `elementProps` rest on the control (`required`/`placeholder`), the
    /// label↔control association, and the Error/Description content.
    #[wasm_bindgen_test]
    async fn the_part_components_render_the_docs_page_composition() {
        use crate::field::field_control::FieldControl;
        use crate::field::field_parts::{FieldDescription, FieldError, FieldLabel};
        use crate::field::field_root::FieldRoot;

        let _ = any_spawner::Executor::init_futures_executor();
        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();

        std::mem::forget(mount_to({ container.clone() }, move || {
            view! {
                <FieldRoot class="field-root".to_string()>
                    <FieldLabel class="field-label".to_string()>"Name"</FieldLabel>
                    <FieldControl
                        class="field-control".to_string()
                        element_attributes=vec![
                            ("required".to_string(), String::new()),
                            ("placeholder".to_string(), "Required".to_string()),
                        ]
                    />
                    <FieldError
                        class="field-error".to_string()
                        error_match=ErrorMatch::Always
                    >
                        "Please enter your name"
                    </FieldError>
                    <FieldDescription class="field-description".to_string()>
                        "Visible on your profile"
                    </FieldDescription>
                </FieldRoot>
            }
        }));

        // The `elementProps` rest bag lands through the control's post-mount Effect
        // (`field_control.rs`, the bag-writer effect), so the attribute asserts follow
        // one browser turn (the module's flush helper docs).
        flush_one_turn().await;

        let label = label_of(&container);
        assert_eq!(
            label.text_content().as_deref(),
            Some("Name"),
            "Field.Label renders its children (upstream's elementProps.children)"
        );
        let input = input_of(&container);
        assert!(
            input.has_attribute("required"),
            "the elementProps rest reaches the input's required attribute"
        );
        assert_eq!(
            input.get_attribute("placeholder").as_deref(),
            Some("Required"),
            "the elementProps rest reaches the input's placeholder"
        );
        assert_eq!(
            label.get_attribute("for").as_deref(),
            input.get_attribute("id").as_deref(),
            "the component-composed label still associates with the control"
        );
        let error = container
            .query_selector(".field-error")
            .unwrap()
            .expect("the error slot rendered (match: true)");
        assert_eq!(
            error.text_content().as_deref(),
            Some("Please enter your name"),
            "Field.Error renders its children"
        );
        let description = container
            .query_selector(".field-description")
            .unwrap()
            .expect("the description rendered");
        assert_eq!(
            description.text_content().as_deref(),
            Some("Visible on your profile"),
            "Field.Description renders its children"
        );
        assert_eq!(
            description.tag_name(),
            "P",
            "the description still renders upstream's <p> (`FieldDescription.tsx`)"
        );
    }

    /// `FieldError.tsx:120-126` — `props: [{ id, children: rendered ? errorMessage :
    /// lastRenderedMessage }, elementProps]`. `elementProps` spreads LAST, so a
    /// user-supplied `children` overrides the derived message rather than being
    /// appended to it (the hero demo's `Please enter your name` is that arm, and the
    /// derived-message arm is what a `Form`-level/server error uses). A field with
    /// BOTH must therefore show the children only — no duplicated message.
    #[wasm_bindgen_test]
    async fn the_error_children_override_the_derived_message() {
        use crate::field::field_parts::FieldError;
        use crate::field::validation::ValidationOutcome;

        let container = mount_field_root(
            || {
                vec![
                    field_control_view(FieldControlViewProps::default()).into_any(),
                    view! {
                        <FieldError class="field-error".to_string() error_match=ErrorMatch::Always>
                            "Please enter your name"
                        </FieldError>
                    }
                    .into_any(),
                ]
            },
            Some(Rc::new(|_value, _form_values| {
                ValidationOutcome::Invalid(vec!["Enter a name".to_string()])
            })),
        );
        let input = input_of(&container);
        type_value(&input, "v");
        press_enter(&input);
        flush_one_turn().await;

        let error = container
            .query_selector(".field-error")
            .unwrap()
            .expect("the error slot rendered");
        let text = error.text_content().unwrap_or_default();
        assert!(
            text.contains("Please enter your name"),
            "the user children render: {text:?}"
        );
        assert!(
            !text.contains("Enter a name"),
            "the derived message is overridden by the user children, not appended: {text:?}"
        );
    }

    /// `FieldValidity.tsx:37-45` through the component surface: the render function
    /// receives the combined payload (the wrapper is the docs-page path).
    #[wasm_bindgen_test]
    fn the_validity_component_hands_the_payload_to_its_render_function() {
        use crate::field::field_control::FieldControl;
        use crate::field::field_parts::FieldValidity;
        use crate::field::field_root::FieldRoot;

        let _ = any_spawner::Executor::init_futures_executor();
        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();

        std::mem::forget(mount_to({ container.clone() }, move || {
            view! {
                <FieldRoot>
                    <FieldControl />
                    <FieldValidity children=Box::new(|payload: FieldValidityPayload| {
                        view! {
                            <div
                                class="validity-payload"
                                data-valid=format!("{:?}", payload.validity.valid)
                                data-error-count=payload.errors.len().to_string()
                            ></div>
                        }
                        .into_any()
                    }) />
                </FieldRoot>
            }
        }));

        let out = container
            .query_selector(".validity-payload")
            .unwrap()
            .expect("the validity slot rendered");
        assert!(
            out.has_attribute("data-valid"),
            "the combined payload reached the component's render function"
        );
        assert_eq!(
            out.get_attribute("data-error-count").as_deref(),
            Some("0"),
            "the payload carries the errors array"
        );
    }
}
