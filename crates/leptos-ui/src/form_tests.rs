//! Tests for the Form port — mirrors of `packages/react/src/form/Form.test.tsx`
//! (the suite behavior.md mines) over the registry-driven pipeline Form owns.
//!
//! Form's own surface is the *registry* contract (implementation.md header: "Form.tsx is
//! deliberately thin (251 lines): it owns only the submit orchestration, the error mirror,
//! and the field registry handle"). Upstream's suite exercises it through real
//! `Field.*`/`Checkbox.*` children; the port pins the same machine, but registers its
//! entries through the very same context the field package uses — synthetic
//! [`FormFieldEntry`] values written into `use_form_context().form_ref`, the way
//! `FormContext.ts`'s own unit tests build entries. That keeps this suite about Form's
//! orchestration (gating, focus targeting, value projection, the actions handle, the
//! error mirror) rather than re-testing field's validity machine.
//!
//! Host suite: the registry semantics that need no DOM — the gate's validate-all plus
//! submit-count bump, the `valid !== false` invalid test, value projection, the
//! `validate(name)` first-match rule, and the error mirror + `clearErrors` pruning through
//! a probe child reading the real provided context.
//!
//! Wasm suite: the mounted-DOM contracts — the native `<form>` with the `novalidate`
//! default (and its `no_validate: false` opt-out), the user `elementProps` bag, the
//! injected submit pipeline over a real dispatched event (`preventDefault` observable on
//! the event object, `onFormSubmit`'s values/details), first-invalid focus by document
//! order, `actionsRef`, and the post-submit focus for errors that land after the submit
//! (behavior.md "Focus management", the async-server-error path).
//!
//! The dual-target split is the crate's convention: wasm-only harness items are dead code
//! on the host target (the separator/progress precedent).

#![allow(unused_imports, dead_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::prelude::*;
use leptos_ui_internals::field_constants::{
    DEFAULT_VALIDITY_STATE, FieldValidityData, FieldValidityState,
};
use leptos_ui_internals::form_context::{ClearErrorsFn, FormFieldEntry, FormRef, FormState};
use serde_json::Value;

use super::*;

// The shared fixture vocabulary both suites build on: one registry entry's description.
#[derive(Clone)]
pub(crate) struct EntrySpec {
    /// The registry key (`fields.set(id, entry)` — the field package's registration id).
    pub id: String,
    /// `name` (`FormContext.ts:17`) — what `validate(name)` matches and the values
    /// projection keys on.
    pub name: Option<String>,
    /// `validityData.state.valid` — `None` is unvalidated, `Some(false)` invalid.
    pub valid: Option<bool>,
    /// `getValue()`'s return (`FormContext.ts:25`).
    pub value: Value,
    /// `controlRef.current` (`FormContext.ts:24`) — the focus target.
    pub control: Option<web_sys::Element>,
    /// Counts this entry's `validate()` invocations (the suite's observation slot).
    pub calls: Rc<Cell<u32>>,
    /// Flips the entry's validity when `validate()` runs — the port's stand-in for a
    /// validator settling a verdict (the field machinery writes the merged validity back
    /// through `form_ref` the same way; `field/validation.rs:409-413`).
    pub verdict: Option<bool>,
}

impl EntrySpec {
    /// A valid, name-carrying entry with no control — the minimal registry citizen.
    pub(crate) fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: Some(id.to_string()),
            valid: Some(true),
            value: Value::Null,
            control: None,
            calls: Rc::new(Cell::new(0)),
            verdict: None,
        }
    }

    pub(crate) fn named(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub(crate) fn unnamed(mut self) -> Self {
        self.name = None;
        self
    }

    pub(crate) fn invalid(mut self) -> Self {
        self.valid = Some(false);
        self
    }

    pub(crate) fn unvalidated(mut self) -> Self {
        self.valid = None;
        self
    }

    pub(crate) fn valued(mut self, value: Value) -> Self {
        self.value = value;
        self
    }

    pub(crate) fn with_control(mut self, control: web_sys::Element) -> Self {
        self.control = Some(control);
        self
    }

    pub(crate) fn settling(mut self, verdict: bool) -> Self {
        self.verdict = Some(verdict);
        self
    }

    /// The registry entry this spec describes (`FormContext.ts:16-27`).
    pub(crate) fn entry(&self) -> FormFieldEntry {
        let calls = Rc::clone(&self.calls);
        let verdict = self.verdict;
        let control = self.control.clone();
        let value = self.value.clone();
        FormFieldEntry {
            name: self.name.clone(),
            // The live validator: counts, then writes its verdict into the registry entry
            // through the shared slot (the field machinery's write-back shape — the
            // validity the gate reads is the one the last `validate()` settled).
            validate: Rc::new(move || {
                calls.set(calls.get() + 1);
                if let Some(verdict) = verdict {
                    if let Some(entry) = VALIDITY_WRITEBACK.with(|slot| slot.borrow().clone()) {
                        entry(verdict);
                    }
                }
            }),
            validity_data: FieldValidityData {
                state: FieldValidityState {
                    valid: self.valid,
                    ..DEFAULT_VALIDITY_STATE
                },
                value: value.clone(),
                ..FieldValidityData::default()
            },
            control_ref: Rc::new(Cell::new(control)),
            get_value: Rc::new(move || Some(value.clone())),
        }
    }
}

thread_local! {
    /// The write-back slot a spec's validator uses to settle its verdict — the tests
    /// install a closure over the live registry before invoking the gate (the
    /// `field/validation.rs` write-back shape).
    static VALIDITY_WRITEBACK: RefCell<Option<Rc<dyn Fn(bool)>>> = const { RefCell::new(None) };
}

thread_local! {
    /// The `clearErrors` the last mounted probe read out of the real provided context —
    /// the wasm fixture fills it (the `FormErrorsHandle`'s companion observation).
    static OBSERVED_CLEAR_ERRORS: RefCell<Option<ClearErrorsFn>> = const { RefCell::new(None) };
}

/// Builds a registry around the given specs, in order (registration order is submit
/// order — `useRegisterFieldControl.ts:18-20`).
fn registry_of(specs: &[EntrySpec]) -> FormRef {
    let registry: FormRef = Rc::new(RefCell::new(FormState::default()));
    for spec in specs {
        registry
            .borrow_mut()
            .fields
            .set(spec.id.clone(), spec.entry());
    }
    registry
}

/// Installs a write-back closure that flips the named entry's `valid` flag in the
/// registry — the mechanism a settled validator uses (`field/validation.rs:409-413`).
fn install_verdict_writer(registry: &FormRef, id: &str) {
    let registry = Rc::clone(registry);
    let id = id.to_string();
    VALIDITY_WRITEBACK.with(|slot| {
        *slot.borrow_mut() = Some(Rc::new(move |verdict: bool| {
            let mut state = registry.borrow_mut();
            if let Some(entry) = state.fields.get(&id) {
                let mut entry = entry.clone();
                entry.validity_data.state.valid = Some(verdict);
                state.fields.set(id.clone(), entry);
            }
        }))
    });
}

fn observed_clear_errors() -> leptos_ui_internals::form_context::ClearErrorsFn {
    OBSERVED_CLEAR_ERRORS.with(|slot| {
        slot.borrow_mut()
            .take()
            .expect("the probe ran inside a Form's children slot")
    })
}

// ---------------------------------------------------------------------------
// Host suite — the registry semantics (no DOM)
// ---------------------------------------------------------------------------

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use leptos_ui_internals::form_context::FormErrorValue;

    fn in_owner() -> reactive_graph::owner::Owner {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        owner
    }

    // `Form.tsx:112-117` — every submit attempt bumps `submitCountRef` and runs every
    // registered field's `validate()` synchronously (async validators cannot block:
    // only the synchronous verdict is read).
    #[test]
    fn the_gate_bumps_the_submit_count_and_runs_every_validator() {
        let _owner = in_owner();
        let first = EntrySpec::new("a");
        let second = EntrySpec::new("b");
        let (first_calls, second_calls) = (Rc::clone(&first.calls), Rc::clone(&second.calls));
        let registry = registry_of(&[first, second]);
        let submit_count = Cell::new(0);

        assert_eq!(
            run_validation_gate(&registry, &submit_count),
            SubmitGate::Allowed,
            "two valid entries let the submit through"
        );
        assert_eq!(first_calls.get(), 1, "the first field validated");
        assert_eq!(second_calls.get(), 1, "the second field validated");
        assert_eq!(submit_count.get(), 1, "the submit count bumped once");
    }

    // `Form.tsx:54-56` + `:63-70` — only an explicit `valid: false` counts as invalid,
    // and an invalid field *without* a usable control still blocks the submit even
    // though there is nothing to focus (the checkbox-group case the source comments).
    #[test]
    fn an_invalid_entry_without_a_usable_control_still_blocks_the_submit() {
        let _owner = in_owner();
        clear_focus_record();
        let registry = registry_of(&[EntrySpec::new("a").invalid()]);
        let submit_count = Cell::new(0);

        assert_eq!(
            run_validation_gate(&registry, &submit_count),
            SubmitGate::Blocked,
            "an invalid field blocks the submit"
        );
        assert!(
            focus_record().is_empty(),
            "nothing was focusable, so nothing was focused"
        );
    }

    // `Form.tsx:54-56` — `validityData.state.valid !== false` is skipped, so an
    // unvalidated field (`null`) does not block.
    #[test]
    fn an_unvalidated_entry_does_not_block_the_submit() {
        let _owner = in_owner();
        let registry = registry_of(&[EntrySpec::new("a").unvalidated()]);
        let submit_count = Cell::new(0);

        assert_eq!(
            run_validation_gate(&registry, &submit_count),
            SubmitGate::Allowed,
            "an unvalidated field does not block"
        );
    }

    // `Form.tsx:115-121` — the validators run BEFORE the gate, so a validator that
    // settles its own verdict in this pass is what the gate observes (the registry entry
    // is written back through `form_ref`, `field/validation.rs:409-413`).
    #[test]
    fn a_validator_that_settles_invalid_in_this_pass_blocks_the_submit() {
        let _owner = in_owner();
        clear_focus_record();
        let spec = EntrySpec::new("a").settling(false);
        let registry = registry_of(&[spec]);
        install_verdict_writer(&registry, "a");
        let submit_count = Cell::new(0);

        assert_eq!(
            run_validation_gate(&registry, &submit_count),
            SubmitGate::Blocked,
            "the verdict the validator just settled is what the gate reads"
        );
    }

    // `Form.tsx:130-135` — the values projection keeps only named entries, in registry
    // (registration) order, and an absent `getValue()` result collapses onto `null`.
    #[test]
    fn only_named_entries_project_into_the_submitted_values() {
        let _owner = in_owner();
        let registry = registry_of(&[
            EntrySpec::new("first").valued(serde_json::json!("Jenny")),
            EntrySpec::new("unnamed")
                .unnamed()
                .valued(serde_json::json!(5)),
            EntrySpec::new("last").valued(serde_json::Value::Null),
        ]);

        assert_eq!(
            project_values(&registry),
            vec![
                ("first".to_string(), serde_json::json!("Jenny")),
                ("last".to_string(), serde_json::Value::Null),
            ],
            "named entries only, in registration order"
        );
    }

    // `Form.tsx:91-95` — `validate(name)` uses `Array.find`, so only the FIRST
    // registered entry with that name is re-validated (implementation.md untested
    // item 1: same-name fields are never exercised upstream).
    #[test]
    fn validate_by_name_targets_the_first_registered_match() {
        let _owner = in_owner();
        // Distinct registry ids, one shared `name` — the same-name case the source's
        // `Array.find` resolves by registration order (implementation.md untested item 1).
        let first = EntrySpec::new("shared-a").named("shared");
        let second = EntrySpec::new("shared-b").named("shared");
        let other = EntrySpec::new("elsewhere");
        let (first_calls, second_calls, other_calls) = (
            Rc::clone(&first.calls),
            Rc::clone(&second.calls),
            Rc::clone(&other.calls),
        );
        let registry = registry_of(&[first, second, other]);

        validate_registry(&registry, Some("shared"));
        assert_eq!(first_calls.get(), 1, "the first 'shared' ran");
        assert_eq!(second_calls.get(), 0, "the second 'shared' did not");
        assert_eq!(other_calls.get(), 0, "an unrelated name was untouched");

        validate_registry(&registry, None);
        assert_eq!(first_calls.get(), 2, "validate() re-ran the first");
        assert_eq!(second_calls.get(), 1, "validate() ran the second");
        assert_eq!(other_calls.get(), 1, "validate() ran the third");
    }

    // `Form.tsx:145-157` — `clearErrors` prunes exactly the named key; an absent key is
    // the `previousErrors` identity no-op (and `clearErrors(undefined)` returns before
    // touching anything, `:146-148`).
    #[test]
    fn clear_errors_prunes_only_the_named_key() {
        let seeded: FormErrors = vec![
            (
                "name".to_string(),
                FormErrorValue::Single("required".to_string()),
            ),
            (
                "age".to_string(),
                FormErrorValue::Multiple(vec!["too young".to_string()]),
            ),
        ];

        assert_eq!(
            prune_error(seeded.clone(), "missing"),
            None,
            "an absent key leaves the record unchanged (the identity return)"
        );

        assert_eq!(
            prune_error(seeded, "name"),
            Some(vec![(
                "age".to_string(),
                FormErrorValue::Multiple(vec!["too young".to_string()]),
            )]),
            "only the named key was pruned"
        );
    }

    // `Form.tsx:73,164` — the mirror is seeded from the `errors` prop at body time
    // (`errors ?? EMPTY_OBJECT`), and the handle binds to that same record: the two
    // observations a caller has of the provided context's `errors` member.
    #[test]
    fn the_errors_mirror_seeds_from_the_prop_and_binds_the_handle() {
        let _owner = in_owner();
        let handle = FormErrorsHandle::new();
        let seeded: FormErrors = vec![(
            "name".to_string(),
            FormErrorValue::Single("required".to_string()),
        )];

        // The rg-0.2 signal type the context's mirror is built from (the dual-runtime
        // law: leptos's prelude brings the rg-0.1 `RwSignal` into scope at this level).
        let mirror: reactive_graph::signal::RwSignal<FormErrors> =
            reactive_graph::signal::RwSignal::new(seeded.clone());
        handle.bind(mirror);
        assert!(handle.is_bound());
        assert_eq!(
            handle.get_untracked(),
            Some(seeded),
            "the bound handle reads the mirror the context publishes"
        );
    }

    // The port's late-error channel (the `errors`-prop-identity analog): before a Form
    // binds it, the handle is inert rather than silently writing nowhere.
    #[test]
    fn the_errors_handle_is_inert_until_a_form_binds_it() {
        let handle = FormErrorsHandle::new();
        assert!(!handle.is_bound());
        handle.set(vec![(
            "name".to_string(),
            FormErrorValue::Single("required".to_string()),
        )]);
        assert_eq!(
            handle.get_untracked(),
            None,
            "an unbound handle stores nothing"
        );
    }

    // `FormContext.ts:33-46` + `Form.tsx:159-171` — the accessor's members come from the
    // *inert default shell* until a Form provides its own bag. The wasm suite pins the
    // provided case at the mounted tree (`the_provided_context_is_the_forms_own`).
    #[test]
    fn the_default_context_is_the_inert_shell() {
        let _owner = in_owner();
        let context = leptos_ui_internals::form_context::use_form_context();

        assert!(
            reactive_graph::traits::GetUntracked::get_untracked(&context.errors).is_empty(),
            "the default shell carries no errors"
        );
        assert_eq!(context.submit_count_ref.get(), 0);
        assert!(context.element_ref.take().is_none());
    }
}

// ---------------------------------------------------------------------------
// Wasm suite — the mounted DOM contracts
// ---------------------------------------------------------------------------

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use leptos::mount::{UnmountHandle, mount_to};
    use leptos::prelude::*;
    use leptos_ui_internals::form_context::FormErrorValue;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    thread_local! {
        /// The specs the fixture installs at body time — how the children closure stays
        /// `Send` while carrying the non-`Send` entry descriptions (the harness routes
        /// them through the single-threaded test page).
        static FIXTURE_SPECS: RefCell<Vec<EntrySpec>> = const { RefCell::new(Vec::new()) };

        /// The registry the fixture installed into (the mounted-context observation).
        static FIXTURE_REGISTRY: RefCell<Option<FormRef>> = const { RefCell::new(None) };

        /// The provided `elementRef` (`FormContext.ts:12`) — set by the mount effect.
        static FIXTURE_ELEMENT_REF: RefCell<Option<Rc<Cell<Option<HtmlFormElement>>>>> =
            const { RefCell::new(None) };

        /// The provided `submitCountRef` (`FormContext.ts:30`) — the "a submit happened"
        /// broadcast consumers gate on.
        static FIXTURE_SUBMIT_COUNT: RefCell<Option<Rc<Cell<u32>>>> = const { RefCell::new(None) };

        /// The submit observations: `(called, values, reason, default_prevented)`.
        static SUBMIT_RECORD: RefCell<Option<(bool, FormValues, String, bool)>> =
            const { RefCell::new(None) };

        /// How many times the user's native `onSubmit` ran.
        static NATIVE_SUBMIT_CALLS: RefCell<u32> = const { RefCell::new(0) };
    }

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Installs the fixture's specs and registers them into the real provided context —
    /// built inside [`Form`]'s children slot, so `use_form_context()` resolves against
    /// Form's provider and, for the field package, is exactly where real controls
    /// register.
    pub(crate) fn registry_fixture() -> impl IntoView {
        let context = leptos_ui_internals::form_context::use_form_context();
        let specs = FIXTURE_SPECS.with(|slot| std::mem::take(&mut *slot.borrow_mut()));
        for spec in &specs {
            context
                .form_ref
                .borrow_mut()
                .fields
                .set(spec.id.clone(), spec.entry());
        }
        FIXTURE_REGISTRY.with(|slot| *slot.borrow_mut() = Some(Rc::clone(&context.form_ref)));
        OBSERVED_CLEAR_ERRORS
            .with(|slot| *slot.borrow_mut() = Some(Rc::clone(&context.clear_errors)));
        FIXTURE_ELEMENT_REF.with(|slot| *slot.borrow_mut() = Some(Rc::clone(&context.element_ref)));
        FIXTURE_SUBMIT_COUNT
            .with(|slot| *slot.borrow_mut() = Some(Rc::clone(&context.submit_count_ref)));
        view! { <span data-testid="fixture"></span> }
    }

    fn fixture_element_ref() -> Rc<Cell<Option<HtmlFormElement>>> {
        FIXTURE_ELEMENT_REF.with(|slot| {
            slot.borrow()
                .clone()
                .expect("the fixture ran inside a Form's children slot")
        })
    }

    fn fixture_submit_count() -> Rc<Cell<u32>> {
        FIXTURE_SUBMIT_COUNT.with(|slot| {
            slot.borrow()
                .clone()
                .expect("the fixture ran inside a Form's children slot")
        })
    }

    /// One mounted Form plus everything a test needs to inspect and tear it down.
    struct Harness {
        container: web_sys::HtmlElement,
        form: web_sys::HtmlFormElement,
        controls: Vec<web_sys::Element>,
        handle: UnmountHandle,
    }

    impl Harness {
        fn registry(&self) -> FormRef {
            FIXTURE_REGISTRY.with(|slot| {
                slot.borrow()
                    .clone()
                    .expect("the fixture ran inside a Form's children slot")
            })
        }

        /// The injected pipeline's observable outcome for the last dispatch.
        fn submit_record(&self) -> Option<(bool, FormValues, String, bool)> {
            SUBMIT_RECORD.with(|slot| slot.borrow().clone())
        }

        /// Dispatches a cancelable `submit` on the mounted form — the synthetic-event
        /// channel, so the browser never performs a real submission/navigation and the
        /// test still observes `defaultPrevented` on the event object it dispatched.
        fn submit(&self) -> web_sys::Event {
            let init = *web_sys::EventInit::new().cancelable(true);
            let event = web_sys::Event::new_with_event_init_dict("submit", &init)
                .expect("a submit event constructs");
            self.form
                .dispatch_event(&event)
                .expect("the form dispatches the submit");
            event
        }

        fn active_id(&self) -> Option<String> {
            document()
                .active_element()
                .and_then(|element| element.get_attribute("id"))
        }

        /// The shared-page hygiene the crate's wasm suites need: drop the mounted tree and
        /// every node this harness appended, so a later test's document order and focus
        /// are its own.
        fn teardown(self) {
            for control in &self.controls {
                let _ = control.remove();
            }
            let _ = self.container.remove();
            // Dropping the handle unmounts the tree; the shared test page keeps every
            // node this test appended, so release the view and let the container removal
            // above take the rendered subtree with it.
            std::mem::forget(self.handle);
        }
    }

    /// Creates a detached-then-appended control element whose id the tests address
    /// (`controlRef.current`'s target).
    fn make_control(tag: &str, id: &str) -> web_sys::Element {
        let element = document().create_element(tag).unwrap();
        element.set_attribute("id", id).unwrap();
        document().body().unwrap().append_child(&element).unwrap();
        element
    }

    /// Mounts a Form with the given specs registered through the probe child, returning
    /// the harness. `no_validate` and the user `element_attributes` ride in as the props
    /// under test; the submit observations always capture the `onFormSubmit` pair so the
    /// pipeline's preventDefault path is exercised (and the browser cannot navigate).
    fn mount_form(
        specs: Vec<EntrySpec>,
        no_validate: Option<bool>,
        attributes: Vec<(String, String)>,
    ) -> Harness {
        FIXTURE_SPECS.with(|slot| *slot.borrow_mut() = specs);
        SUBMIT_RECORD.with(|slot| *slot.borrow_mut() = None);
        NATIVE_SUBMIT_CALLS.with(|slot| *slot.borrow_mut() = 0);

        let container: web_sys::HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();

        let handle = mount_to({ container.clone() }, move || {
            let on_form_submit: Rc<dyn Fn(FormValues, FormSubmitEventDetails)> =
                Rc::new(|values, details| {
                    SUBMIT_RECORD.with(|slot| {
                        *slot.borrow_mut() = Some((
                            true,
                            values,
                            details.reason.clone(),
                            details.event.default_prevented(),
                        ));
                    });
                });
            let on_submit: Rc<dyn Fn(&web_sys::Event)> = Rc::new(|_event| {
                NATIVE_SUBMIT_CALLS.with(|slot| *slot.borrow_mut() += 1);
            });
            view! {
                <Form
                    no_validate=no_validate
                    element_attributes=attributes
                    on_submit=on_submit
                    on_form_submit=on_form_submit
                >
                    {registry_fixture()}
                </Form>
            }
        });

        let form = container
            .query_selector("form")
            .unwrap()
            .expect("the form mounts")
            .dyn_into::<web_sys::HtmlFormElement>()
            .unwrap();
        Harness {
            container,
            form,
            controls: Vec::new(),
            handle,
        }
    }

    // behavior.md "DOM structure" (`Form.test.tsx:1034-1042`): the root is a native
    // `<form>` carrying `novalidate` by default — upstream injects `noValidate: true`
    // ahead of `elementProps` (`Form.tsx:110`), which is why Base UI re-implements the
    // gating itself instead of relying on native constraint UI.
    #[wasm_bindgen_test]
    fn the_root_is_a_native_form_with_novalidate_by_default() {
        let harness = mount_form(Vec::new(), None, Vec::new());
        assert_eq!(harness.form.tag_name(), "FORM", "the root is a native form");
        assert!(
            harness.form.has_attribute("novalidate"),
            "novalidate is present by default"
        );
        harness.teardown();
    }

    // `Form.tsx:110` + behavior.md "DOM structure": `noValidate={false}` removes the
    // attribute — the user's later bag wins over the injected default.
    #[wasm_bindgen_test]
    fn no_validate_false_removes_the_novalidate_attribute() {
        let harness = mount_form(Vec::new(), Some(false), Vec::new());
        assert!(
            !harness.form.has_attribute("novalidate"),
            "the explicit opt-out removes the attribute"
        );
        harness.teardown();
    }

    // behavior.md "Public API surface" (`propForwarding.tsx:23-36`): arbitrary DOM props
    // (`lang`, `data-*`) forward to the rendered element through the `elementProps` rest.
    #[wasm_bindgen_test]
    fn user_element_attributes_forward_to_the_form() {
        let harness = mount_form(
            Vec::new(),
            None,
            vec![
                ("lang".to_string(), "fr".to_string()),
                ("data-foobar".to_string(), "random-value".to_string()),
            ],
        );
        assert_eq!(harness.form.get_attribute("lang").as_deref(), Some("fr"));
        assert_eq!(
            harness.form.get_attribute("data-foobar").as_deref(),
            Some("random-value")
        );
        harness.teardown();
    }

    // behavior.md "Focus management" (`Form.test.tsx:49-74`) + "Events": an invalid
    // registered control blocks the submit (the event's default is prevented) and the
    // first invalid field is focused (`focusFirstInvalid`, `Form.tsx:43-71`);
    // `onFormSubmit` never runs for an invalid form (`:1001-1003`).
    #[wasm_bindgen_test]
    fn an_invalid_registered_control_blocks_the_submit_and_focuses_it() {
        let control = make_control("input", "field-a");
        let spec = EntrySpec::new("a").invalid().with_control(control.clone());
        let mut harness = mount_form(vec![spec], None, Vec::new());
        harness.controls = vec![control];

        let event = harness.submit();
        assert!(
            event.default_prevented(),
            "the invalid submit was prevented (`:119-121`)"
        );
        assert_eq!(
            harness.active_id().as_deref(),
            Some("field-a"),
            "the first invalid field was focused"
        );
        assert!(
            harness.submit_record().is_none(),
            "onFormSubmit does not fire for an invalid form"
        );
        harness.teardown();
    }

    // behavior.md "Events" (`Form.test.tsx:954-983`): a successful submit projects the
    // registered values, hands `(formValues, eventDetails)` to `onFormSubmit`, whose
    // `reason` is `REASONS.none` and whose `event.defaultPrevented` is `true` (Form
    // prevents the native default itself, `:127-128`).
    #[wasm_bindgen_test]
    fn a_valid_submit_projects_the_values_and_the_details() {
        let spec = EntrySpec::new("name").valued(serde_json::json!("Jenny"));
        let harness = mount_form(vec![spec], None, Vec::new());

        let _event = harness.submit();
        let (called, values, reason, default_prevented) = harness
            .submit_record()
            .expect("onFormSubmit ran for a valid form");
        assert!(called);
        assert_eq!(
            values,
            vec![("name".to_string(), serde_json::json!("Jenny"))],
            "the values are keyed by field name"
        );
        assert_eq!(reason, "none", "the only submit reason Form emits");
        assert!(
            default_prevented,
            "eventDetails.event.defaultPrevented is true (`:137`)"
        );
        assert_eq!(
            NATIVE_SUBMIT_CALLS.with(|slot| *slot.borrow()),
            1,
            "the user's native onSubmit ran once (`:125`)"
        );
        harness.teardown();
    }

    // behavior.md "Focus management" (`Form.test.tsx:158-188`): registration order can
    // diverge from DOM order, and first-invalid follows DOCUMENT order
    // (`comesBeforeInSameTree`, `Form.tsx:244-251`). The second-registered control is
    // first in the document, so it wins.
    #[wasm_bindgen_test]
    fn the_first_invalid_follows_document_order_over_registration_order() {
        let first_in_document = make_control("input", "early");
        let second_in_document = make_control("input", "late");
        let specs = vec![
            EntrySpec::new("late")
                .invalid()
                .with_control(second_in_document.clone()),
            EntrySpec::new("early")
                .invalid()
                .with_control(first_in_document.clone()),
        ];
        let mut harness = mount_form(specs, None, Vec::new());
        harness.controls = vec![first_in_document, second_in_document];

        let _event = harness.submit();
        assert_eq!(
            harness.active_id().as_deref(),
            Some("early"),
            "the control that comes first in the document wins, not the first registered"
        );
        harness.teardown();
    }

    // behavior.md "Focus management" (`Form.test.tsx:1046-1112`): `actionsRef.validate`
    // re-validates through the live registry — `validate(name)` targets the first entry
    // with that name (`Form.tsx:91-95`) and the handle is allocated once
    // (`:88-104`, the empty dep array).
    #[wasm_bindgen_test]
    fn actions_ref_validate_targets_the_first_registered_match() {
        let first = EntrySpec::new("shared");
        let second = EntrySpec::new("shared");
        let (first_calls, second_calls) = (Rc::clone(&first.calls), Rc::clone(&second.calls));
        let actions_slot: FormActionsRef = Rc::new(Cell::new(None));

        FIXTURE_SPECS.with(|slot| *slot.borrow_mut() = vec![first, second]);
        let container: web_sys::HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();
        let actions_for_view = Rc::clone(&actions_slot);
        std::mem::forget(mount_to({ container.clone() }, move || {
            view! { <Form actions_ref=Some(actions_for_view)>{registry_fixture()}</Form> }
        }));

        let handle = actions_slot
            .take()
            .expect("the mount effect wrote the actions handle");
        handle.validate(Some("shared"));
        assert_eq!(first_calls.get(), 1, "the first 'shared' re-validated");
        assert_eq!(second_calls.get(), 0, "only the first match ran");

        handle.validate(None);
        assert_eq!(first_calls.get(), 2, "validate() walked the registry");
        assert_eq!(
            second_calls.get(),
            1,
            "validate() re-validated the second too"
        );

        let _ = container.remove();
    }

    // behavior.md "Focus management" (`Form.test.tsx:662-692`): errors that arrive AFTER a
    // submit still trigger the focus — `submittedRef` is set on a successful submit
    // (`Form.tsx:124`) and the effect keyed on the mirror consumes it once
    // (`:79-86`). The port delivers those errors through [`FormErrorsHandle`] (the
    // `errors`-prop-identity analog), and the field machinery is what flips the entry's
    // validity on that commit — simulated here by writing the verdict back.
    #[wasm_bindgen_test]
    fn errors_that_land_after_a_submit_focus_the_first_invalid() {
        let control = make_control("input", "server-field");
        let spec = EntrySpec::new("name").with_control(control.clone());
        let calls = Rc::clone(&spec.calls);
        let errors_handle = FormErrorsHandle::new();

        FIXTURE_SPECS.with(|slot| *slot.borrow_mut() = vec![spec]);
        let container: web_sys::HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();
        let handle_for_view = errors_handle.clone();
        std::mem::forget(mount_to({ container.clone() }, move || {
            let on_form_submit: Rc<dyn Fn(FormValues, FormSubmitEventDetails)> = Rc::new(|_, _| {});
            view! {
                <Form errors_handle=handle_for_view on_form_submit=on_form_submit>
                    {registry_fixture()}
                </Form>
            }
        }));

        let registry = FIXTURE_REGISTRY.with(|slot| slot.borrow().clone().unwrap());
        let form = container
            .query_selector("form")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::HtmlFormElement>()
            .unwrap();

        // The valid submit: it passes the gate, so `submittedRef` is now set.
        let init = *web_sys::EventInit::new().cancelable(true);
        let event = web_sys::Event::new_with_event_init_dict("submit", &init).unwrap();
        form.dispatch_event(&event).unwrap();
        assert_eq!(calls.get(), 1, "the validator ran on submit");
        assert!(handle_for_view.is_bound(), "the mirror is bound");

        // The server error lands: the field becomes invalid and the mirror commits.
        {
            let mut state = registry.borrow_mut();
            let mut entry = state.fields.get("name").unwrap().clone();
            entry.validity_data.state.valid = Some(false);
            state.fields.set("name".to_string(), entry);
        }
        handle_for_view.set(vec![(
            "name".to_string(),
            leptos_ui_internals::form_context::FormErrorValue::Single("already taken".to_string()),
        )]);

        assert_eq!(
            document()
                .active_element()
                .and_then(|element| element.get_attribute("id"))
                .as_deref(),
            Some("server-field"),
            "the post-submit error commit focused the first invalid control"
        );

        let _ = control.remove();
        let _ = container.remove();
    }

    // implementation.md "Context providers/consumers" (`FormContext.ts:9-31`,
    // `Form.tsx:159-171`): every member crosses the boundary — the registry children
    // register into, the `elementRef` the mount effect writes (`:39,161`), and the
    // `submitCountRef` the submit bumps (`:112`). Reading them back out of the real
    // provided context is what makes the field package's registration work at all.
    #[wasm_bindgen_test]
    fn the_provided_context_carries_the_registry_element_and_submit_count() {
        let harness = mount_form(vec![EntrySpec::new("probe")], None, Vec::new());

        assert_eq!(
            harness.registry().borrow().fields.len(),
            1,
            "the fixture registered into the Form's own registry"
        );
        let element_ref = fixture_element_ref();
        let stored = crate::field::validation::cell_peek(&element_ref)
            .expect("the mount effect wrote elementRef");
        assert!(
            stored.is_same_node(Some(&harness.form)),
            "elementRef points at the mounted <form>"
        );

        let _event = harness.submit();
        assert_eq!(
            fixture_submit_count().get(),
            1,
            "submitCountRef bumped on the submit attempt"
        );
        harness.teardown();
    }

    // `Form.tsx:73,145-157,164` — the mirror seeds from the `errors` prop, and
    // `clearErrors` read out of the provided context prunes exactly the named key:
    // an absent key (and the `undefined` name) leaves the record untouched. The
    // mounted-tree counterpart of the host suite's pure-helper pin.
    #[wasm_bindgen_test]
    fn the_errors_mirror_seeds_from_the_prop_and_clear_errors_prunes_one_key() {
        let errors_handle = FormErrorsHandle::new();
        let seeded: FormErrors = vec![
            (
                "name".to_string(),
                FormErrorValue::Single("required".to_string()),
            ),
            (
                "age".to_string(),
                FormErrorValue::Multiple(vec!["too young".to_string()]),
            ),
        ];

        FIXTURE_SPECS.with(|slot| *slot.borrow_mut() = Vec::new());
        let container: web_sys::HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();
        let handle_for_view = errors_handle.clone();
        let seeded_for_view = seeded.clone();
        std::mem::forget(mount_to({ container.clone() }, move || {
            view! {
                <Form errors=seeded_for_view errors_handle=handle_for_view>
                    {registry_fixture()}
                </Form>
            }
        }));

        assert!(
            errors_handle.is_bound(),
            "the mounted Form bound its mirror"
        );
        assert_eq!(
            errors_handle.get_untracked(),
            Some(seeded.clone()),
            "the mirror seeded from the errors prop"
        );

        let clear_errors = observed_clear_errors();
        clear_errors(None);
        clear_errors(Some("missing"));
        assert_eq!(
            errors_handle.get_untracked(),
            Some(seeded),
            "a nameless or absent-key clear is a no-op"
        );

        clear_errors(Some("name"));
        assert_eq!(
            errors_handle.get_untracked(),
            Some(vec![(
                "age".to_string(),
                FormErrorValue::Multiple(vec!["too young".to_string()]),
            )]),
            "only the named key was pruned"
        );
        let _ = container.remove();
    }
}
