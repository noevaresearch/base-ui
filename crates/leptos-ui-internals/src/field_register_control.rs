//! Port of `packages/react/src/internals/field-register-control/` — the two hooks wiring a
//! field's control into the surrounding Field/Form registration machinery
//! (`specs/library/internals/implementation.md`, "Context providers/consumers" — the
//! `FieldRootContext`/`FormContext` rows and the cross-boundary state notes; the whole
//! directory is untested upstream per "Anything in source not explained by any test" item
//! 4, so the suites below pin the written mechanics per the labelable-provider
//! precedent).
//!
//! Upstream is two modules, consolidated here in upstream's own order:
//! `useRegisterFieldControl.ts` ([`use_register_field_control`], the control-side hook
//! that pushes its registration into the field) and `useFieldControlRegistration.ts`
//! ([`use_field_control_registration`], the field-side hook owning the Form registry
//! entry, plus the [`FieldControlRegistration`] payload type). The supporting context
//! vocabulary lives in [`crate::field_root_context`] and [`crate::form_context`]; the
//! validity combiner is [`crate::get_combined_field_validity_data`].
//!
//! ## Behavior carried over
//!
//! - Re-registration never unregisters first: re-registering with the same id updates the
//!   form's fields entry **in place**, because a delete + re-add would move the field to
//!   the end of the registry and reorder form submission
//!   (`useRegisterFieldControl.ts:18-20`).
//! - The registration is keyed by a per-control `Symbol` source
//!   (`useRegisterFieldControl.ts:16`, the `useRefWithInit(() => Symbol())` the
//!   implementation spec's cross-note records): a replaced control's pending work is
//!   dropped (`change(undefined, true)`) but only when a previous registration existed
//!   (`useFieldControlRegistration.ts:150-153`), and an unregister from a source that is
//!   not the active one is ignored (`:136-144`).
//! - The field-level `initialValue` baseline is captured exactly once, not per control
//!   instance: registration re-runs on every value change, and a swapped/remounted control
//!   would otherwise turn whichever value it happens to hold into the initial value
//!   (`useFieldControlRegistration.ts:86-103`'s comment).
//! - The disabled arm registers `undefined` (`useRegisterFieldControl.ts:24-27`) — which
//!   the field-side hook answers by dropping the registration and cancelling pending
//!   work — and unmount unregisters through the dedicated cleanup effect
//!   (`:40-45`, `:122-131`).
//!
//! ## Rust adaptations
//!
//! - The per-control `Symbol()` ports to [`ControlIdSource`], the labelable-provider
//!   token (the implementation spec's cross-note records the same shape for both units;
//!   `labelable_provider.rs` module docs).
//! - The `useIsoLayoutEffect` dep arrays port to tracked reactive reads: `value`, `id`,
//!   `name`, and `enabled` are [`Get`] sources whose changes re-run the registration
//!   effect (the `use_press_and_hold.rs` reactive-source convention). `controlRef` and
//!   `getFormValueOverride` are stable per instance — the ref slot is the view layer's
//!   node-ref handle and the override closure is handed in once (the
//!   `use_labelable_id.rs` static-params precedent).
//! - `setValidityData`'s `Dispatch<SetStateAction<FieldValidityData>>` param collapses
//!   onto the `validity_data: RwSignal<FieldValidityData>` handle the caller already
//!   holds (the `field_root_context.rs` setter convention): the functional update in
//!   `captureInitialValue` composes `get_untracked` + inequality-guarded `set` (React's
//!   same-value bail-out, the `use_transition_status.rs` precedent).
//! - `unknown` values port to [`serde_json::Value`] with `undefined` collapsing onto
//!   [`Value::Null`] at the `commit`/`initialValue` boundary (the `field_constants.rs`
//!   precedent); [`FieldControlRegistration::value`] and the `getValue` return keep
//!   [`Option`] so the `registration.value === undefined ? getValueForForm() :
//!   registration.value` fallback (`useFieldControlRegistration.ts:49-51`) is exact.
//! - `change(undefined, true)`'s literal `undefined` value ports to [`None`] in the
//!   [`FieldChangeFn`] call — the receiving side (Phase B validation) decides what the
//!   empty value means, exactly as upstream's `change` does.
//! - `commit`'s async wrapper (`useFieldValidation.ts:414`) collapses to a synchronous
//!   call: in-unit callers never await it and the port's state writes are synchronous
//!   (the `use_animations_finished.rs` flushSync note).
//! - `'use client'` is N/A — no React Server Components boundary in Rust.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::LocalStorage;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use send_wrapper::SendWrapper;
use serde_json::Value;
use web_sys::Element;

use leptos_ui_utils::use_iso_layout_effect;

use crate::field_constants::FieldValidityData;
use crate::field_root_context::{FieldChangeFn, FieldCommitFn, use_field_root_context};
use crate::form_context::{FormFieldEntry, FormRef, use_form_context};
use crate::get_combined_field_validity_data::get_combined_field_validity_data;
use crate::labelable_provider::ControlIdSource;

// ---------------------------------------------------------------------------
// The registration payload (useFieldControlRegistration.ts:9-15)
// ---------------------------------------------------------------------------

/// The imperative value getter — upstream `getValue?: () => unknown`
/// (`useFieldControlRegistration.ts:13`); `None` is the JS `undefined` return.
pub type GetControlValueFn = Rc<dyn Fn() -> Option<Value>>;

/// One control registration — upstream's `FieldControlRegistration`
/// (`useFieldControlRegistration.ts:9-15`).
#[derive(Clone)]
pub struct FieldControlRegistration {
    /// `controlRef` (`:10` — `React.RefObject<any>`; the crate's ref slot).
    pub control_ref: Rc<Cell<Option<Element>>>,
    /// `id` (`:11` — `string | undefined`).
    pub id: Option<String>,
    /// `name` (`:12`).
    pub name: Option<String>,
    /// `getValue` (`:13`).
    pub get_value: Option<GetControlValueFn>,
    /// `value` (`:14` — `unknown`; `None` is the JS `undefined`).
    pub value: Option<Value>,
}

/// JS truthiness over the optional field `name` — `!name` at
/// `useFieldControlRegistration.ts:157` and `name ? undefined : registration.name` at
/// `:111` are truthiness checks, so an empty string is "no name" while still being
/// nullish-coalesced over the registration name (`??`, `:73`/`:115`).
fn is_truthy_name(name: Option<&str>) -> bool {
    name.is_some_and(|name| !name.is_empty())
}

// ---------------------------------------------------------------------------
// useRegisterFieldControl.ts
// ---------------------------------------------------------------------------

/// The parameters — upstream's `useRegisterFieldControl(controlRef, id, value,
/// getFormValueOverride?, enabled = true, name?)` positional signature
/// (`useRegisterFieldControl.ts:7-14`). The reactive sources stand in for the dep array
/// at `:38` (module docs).
pub struct UseRegisterFieldControlParams<I, V, E, N> {
    /// `controlRef` (`:8`) — the control element ref slot.
    pub control_ref: Rc<Cell<Option<Element>>>,
    /// `id` (`:9`).
    pub id: I,
    /// `value` (`:10`).
    pub value: V,
    /// `getFormValueOverride` (`:11` — `getFormValueOverride?: getValue`).
    pub get_form_value_override: Option<GetControlValueFn>,
    /// `enabled` (`:12` — upstream default `true`): `false` registers `undefined`,
    /// dropping the control from the field (`:24-27`).
    pub enabled: E,
    /// `name` (`:13`).
    pub name: N,
}

/// Port of `useRegisterFieldControl` (`useRegisterFieldControl.ts:7-46`): pushes the
/// control's registration into the surrounding field — re-registering on `value`/`id`/
/// `name` changes without unregistering first (the in-place update rule), registering
/// `undefined` while disabled, and unregistering on unmount. Must be called inside a
/// reactive owner (a component), like the other hook ports.
pub fn use_register_field_control<I, V, E, N>(params: UseRegisterFieldControlParams<I, V, E, N>)
where
    I: Get<Value = Option<String>> + Clone + 'static,
    V: Get<Value = Option<Value>> + Clone + 'static,
    E: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    N: Get<Value = Option<String>> + GetUntracked<Value = Option<String>> + Clone + 'static,
{
    let UseRegisterFieldControlParams {
        control_ref,
        id,
        value,
        get_form_value_override,
        enabled,
        name,
    } = params;

    let context = use_field_root_context();

    // `const sourceRef = useRefWithInit(() => Symbol())` (`:16`).
    let source = ControlIdSource::new();

    // The registration effect (`:21-38`): re-register without unregistering first — the
    // same-id update keeps the form's fields entry in place (module docs).
    {
        let register_field_control = context.register_field_control.clone();
        let control_ref = Rc::clone(&control_ref);
        let get_form_value_override = get_form_value_override.clone();
        use_iso_layout_effect(move || {
            if !enabled.get() {
                // `registerFieldControl(source, undefined)` (`:24-27`).
                register_field_control(source, None);
                return;
            }

            let registration = FieldControlRegistration {
                control_ref: Rc::clone(&control_ref),
                get_value: get_form_value_override.clone(),
                id: id.get(),
                name: name.get(),
                value: value.get(),
            };
            register_field_control(source, Some(registration));
        });
    }

    // The unmount unregistration (`:40-45`) — registered through the layout effect's
    // cleanup so it fires in the layout phase (the `labelable_provider.rs`
    // on_cleanup contract).
    {
        let register_field_control = SendWrapper::new(context.register_field_control.clone());
        use_iso_layout_effect(move || {
            let register_field_control = register_field_control.clone();
            reactive_graph::owner::on_cleanup(move || {
                register_field_control(source, None);
            });
        });
    }
}

// ---------------------------------------------------------------------------
// useFieldControlRegistration.ts
// ---------------------------------------------------------------------------

/// The parameters — upstream's `UseFieldControlRegistrationParameters`
/// (`useFieldControlRegistration.ts:174-184`). `validityData` + `setValidityData`
/// collapse onto one reactive handle, and `invalid`/`name` are reactive sources standing
/// in for the dep array at `:120` (module docs).
pub struct UseFieldControlRegistrationParams<I, N> {
    /// `change` (`:175`) — `(value, cancelPending?)`; `None` is the literal `undefined`
    /// value, `true` cancels pending work.
    pub change: FieldChangeFn,
    /// `commit` (`:176`).
    pub commit: FieldCommitFn,
    /// `invalid` (`:177`).
    pub invalid: I,
    /// `markedDirtyRef` (`:178`).
    pub marked_dirty_ref: Rc<Cell<bool>>,
    /// `name` (`:179`) — the field-level name.
    pub name: N,
    /// `setRegisteredFieldName` (`:180`).
    pub set_registered_field_name: Rc<dyn Fn(Option<String>)>,
    /// `registeredFieldIdRef` (`:181`).
    pub registered_field_id_ref: Rc<RefCell<Option<String>>>,
    /// `validityData` + `setValidityData` (`:182-183`) — one reactive handle (module
    /// docs).
    pub validity_data: RwSignal<FieldValidityData, LocalStorage>,
}

/// The return — upstream's `[validate, register] as const`
/// (`useFieldControlRegistration.ts:171`).
pub struct UseFieldControlRegistrationReturn {
    /// `validate` — the field-level validate action (imperatively exposed through the
    /// field's `actionsRef`, `FieldRoot.tsx:154-156`).
    pub validate: Rc<dyn Fn()>,
    /// `register` — the `FieldRootContext.registerFieldControl` implementation.
    pub register: crate::field_root_context::RegisterFieldControlFn,
}

/// Port of `useFieldControlRegistration` (`useFieldControlRegistration.ts:17-172`): the
/// field-side half of the registration — it owns the Form registry entry for this field
/// (`formRef.current.fields`), the initial-value baseline, and the source-keyed control
/// handover. Must be called inside a reactive owner (a component), like the other hook
/// ports.
pub fn use_field_control_registration<I, N>(
    params: UseFieldControlRegistrationParams<I, N>,
) -> UseFieldControlRegistrationReturn
where
    I: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    N: Get<Value = Option<String>> + GetUntracked<Value = Option<String>> + Clone + 'static,
{
    let UseFieldControlRegistrationParams {
        change,
        commit,
        invalid,
        marked_dirty_ref,
        name,
        set_registered_field_name,
        registered_field_id_ref,
        validity_data,
    } = params;

    // `const { formRef } = useFormContext()` (`:30`).
    let form: FormRef = use_form_context().form_ref;

    // The three guard refs (`:32-34`).
    let active_field_control_source: Rc<Cell<Option<ControlIdSource>>> = Rc::new(Cell::new(None));
    let registration_ref: Rc<RefCell<Option<FieldControlRegistration>>> =
        Rc::new(RefCell::new(None));
    let initial_value_captured: Rc<Cell<bool>> = Rc::new(Cell::new(false));

    // `getValueForForm` (`:36-47`): the override closure wins, else the stored value.
    let get_value_for_form: crate::form_context::GetFieldValueFn = {
        let registration_ref = Rc::clone(&registration_ref);
        Rc::new(move || {
            let registration = registration_ref.borrow().clone();
            let Some(registration) = registration else {
                return None;
            };
            if let Some(get_value) = &registration.get_value {
                return get_value();
            }
            registration.value.clone()
        })
    };

    // `getRegistrationValue` (`:49-51`): `value === undefined ? getValueForForm() : value`.
    let get_registration_value: Rc<dyn Fn(&FieldControlRegistration) -> Option<Value>> = {
        let get_value_for_form = Rc::clone(&get_value_for_form);
        Rc::new(move |registration: &FieldControlRegistration| {
            registration.value.clone().or_else(|| get_value_for_form())
        })
    };

    // `validate` (`:53-63`).
    let validate: Rc<dyn Fn()> = {
        let registration_ref = Rc::clone(&registration_ref);
        let marked_dirty_ref = Rc::clone(&marked_dirty_ref);
        let validity_data = validity_data.clone();
        let commit = Rc::clone(&commit);
        let get_registration_value = Rc::clone(&get_registration_value);
        Rc::new(move || {
            marked_dirty_ref.set(true);

            let registration = registration_ref.borrow().clone();
            match registration {
                None => commit(validity_data.get_untracked().value.clone()),
                Some(registration) => {
                    commit(get_registration_value(&registration).unwrap_or(Value::Null))
                }
            }
        })
    };

    // `refreshRegistration` (`:65-78`).
    let refresh_registration: Rc<dyn Fn()> = {
        let registration_ref = Rc::clone(&registration_ref);
        let form = Rc::clone(&form);
        let get_value_for_form = Rc::clone(&get_value_for_form);
        let validate = Rc::clone(&validate);
        let validity_data = validity_data.clone();
        let invalid = invalid.clone();
        let name = name.clone();
        Rc::new(move || {
            let registration = registration_ref.borrow().clone();
            let Some(registration) = registration else {
                return;
            };
            let Some(id) = registration.id.clone() else {
                return;
            };

            // The reads are untracked: this runs from `register` (a callback context),
            // never as a tracked effect — the registration effect below owns the tracked
            // freshness (module docs).
            let name_value = name.get_untracked();
            let validity_data = validity_data.get_untracked();
            let invalid = invalid.get_untracked();

            form.borrow_mut().fields.set(
                id,
                FormFieldEntry {
                    name: name_value.or(registration.name.clone()),
                    get_value: Rc::clone(&get_value_for_form),
                    control_ref: Rc::clone(&registration.control_ref),
                    validity_data: get_combined_field_validity_data(&validity_data, invalid),
                    validate: Rc::clone(&validate),
                },
            );
        })
    };

    // `deleteRegistration` (`:80-84`) — `None` defaults to the current registration's id.
    let delete_registration: Rc<dyn Fn(Option<String>)> = {
        let registration_ref = Rc::clone(&registration_ref);
        let form = Rc::clone(&form);
        Rc::new(move |id: Option<String>| {
            let id = id.or_else(|| {
                registration_ref
                    .borrow()
                    .as_ref()
                    .and_then(|registration| registration.id.clone())
            });
            if let Some(id) = id {
                form.borrow_mut().fields.delete(&id);
            }
        })
    };

    // `captureInitialValue` (`:92-103`).
    let capture_initial_value: Rc<dyn Fn(&FieldControlRegistration)> = {
        let initial_value_captured = Rc::clone(&initial_value_captured);
        let get_registration_value = Rc::clone(&get_registration_value);
        let validity_data = validity_data.clone();
        Rc::new(move |registration: &FieldControlRegistration| {
            if initial_value_captured.get() {
                return;
            }

            initial_value_captured.set(true);
            // The `undefined` baseline collapses onto `Value::Null` (module docs).
            let initial_value = get_registration_value(registration).unwrap_or(Value::Null);

            // `setValidityData((prev) => prev.initialValue === initialValue ? prev :
            // { ...prev, initialValue })` (`:100-102`) — the same-value bail-out as the
            // inequality guard.
            let prev = validity_data.get_untracked();
            if prev.initial_value != initial_value {
                validity_data.set(FieldValidityData {
                    initial_value,
                    ..prev
                });
            }
        })
    };

    // The registration effect (`:105-120`): keeps the form's registry entry fresh —
    // tracked reads of `validityData`/`invalid`/`name` reproduce the dep array (module
    // docs).
    {
        let registration_ref = Rc::clone(&registration_ref);
        let form = Rc::clone(&form);
        let get_value_for_form = Rc::clone(&get_value_for_form);
        let validate = Rc::clone(&validate);
        let validity_data = validity_data.clone();
        let invalid = invalid.clone();
        let name = name.clone();
        let set_registered_field_name = Rc::clone(&set_registered_field_name);
        use_iso_layout_effect(move || {
            // The tracked reads come FIRST, before the registration guard: upstream's dep
            // array lists `validityData`/`invalid`/`name` unconditionally, so the effect
            // re-runs on their changes even while the field has no registered control yet
            // (module docs).
            let validity_data = validity_data.get();
            let invalid = invalid.get();
            let name_value = name.get();

            let registration = registration_ref.borrow().clone();
            let Some(registration) = registration else {
                return;
            };
            let Some(id) = registration.id.clone() else {
                return;
            };

            // `setRegisteredFieldName(name ? undefined : registration.name)` (`:111`) —
            // truthiness, so an empty-string field name lets the control's name through.
            if is_truthy_name(name_value.as_deref()) {
                set_registered_field_name(None);
            } else {
                set_registered_field_name(registration.name.clone());
            }

            form.borrow_mut().fields.set(
                id,
                FormFieldEntry {
                    // `name ?? registration.name` (`:115`) — nullish, so an empty-string
                    // field name still wins over the registration name.
                    name: name_value.or(registration.name.clone()),
                    get_value: Rc::clone(&get_value_for_form),
                    control_ref: Rc::clone(&registration.control_ref),
                    validity_data: get_combined_field_validity_data(&validity_data, invalid),
                    validate: Rc::clone(&validate),
                },
            );
        });
    }

    // The unmount cleanup (`:122-131`): the registry handle is captured at mount and the
    // current registration's id deleted at unmount.
    {
        let registration_ref = SendWrapper::new(Rc::clone(&registration_ref));
        let form = SendWrapper::new(Rc::clone(&form));
        use_iso_layout_effect(move || {
            let registration_ref = registration_ref.clone();
            let form = form.clone();
            reactive_graph::owner::on_cleanup(move || {
                let id = registration_ref
                    .borrow()
                    .as_ref()
                    .and_then(|registration| registration.id.clone());
                if let Some(id) = id {
                    form.borrow_mut().fields.delete(&id);
                }
            });
        });
    }

    // `register` (`:133-169`).
    let register: crate::field_root_context::RegisterFieldControlFn = {
        let active_field_control_source = Rc::clone(&active_field_control_source);
        let registration_ref = Rc::clone(&registration_ref);
        let registered_field_id_ref = Rc::clone(&registered_field_id_ref);
        let change = Rc::clone(&change);
        let name = name.clone();
        let set_registered_field_name = Rc::clone(&set_registered_field_name);
        let delete_registration = Rc::clone(&delete_registration);
        let capture_initial_value = Rc::clone(&capture_initial_value);
        let refresh_registration = Rc::clone(&refresh_registration);
        Rc::new(move |source, registration| {
            let Some(registration) = registration else {
                // `if (!registration)` (`:135-145`): only the active source unregisters.
                if active_field_control_source.get() == Some(source) {
                    active_field_control_source.set(None);
                    change(None, true);
                    delete_registration(None);
                    *registration_ref.borrow_mut() = None;
                    set_registered_field_name(None);
                    *registered_field_id_ref.borrow_mut() = None;
                }
                return;
            };

            let previous_id = registration_ref
                .borrow()
                .as_ref()
                .and_then(|registration| registration.id.clone());
            let previous_source = active_field_control_source.get();

            // Drop work owned by a replaced control, but not on first registration
            // (`:150-153`).
            if previous_source.is_some_and(|previous_source| previous_source != source) {
                change(None, true);
            }

            active_field_control_source.set(Some(source));
            *registration_ref.borrow_mut() = Some(registration.clone());
            // `if (!name)` (`:157-159`) — truthiness (module docs).
            if !is_truthy_name(name.get_untracked().as_deref()) {
                set_registered_field_name(registration.name.clone());
            }
            *registered_field_id_ref.borrow_mut() = registration.id.clone();

            // `if (previousId && previousId !== registration.id)` (`:162-164`) — the
            // truthy guard, so an empty-string id never deletes.
            if previous_id
                .as_deref()
                .is_some_and(|previous_id| Some(previous_id) != registration.id.as_deref())
            {
                delete_registration(previous_id);
            }

            capture_initial_value(&registration);
            refresh_registration();
        })
    };

    UseFieldControlRegistrationReturn { validate, register }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::RefCell;

    use serde_json::json;

    use super::*;
    use crate::field_constants::{DEFAULT_VALIDITY_STATE, FieldValidityState};
    use reactive_graph::owner::Owner;
    use reactive_graph::traits::GetUntracked as _GetUntrackedUnused;
    use reactive_graph::wrappers::read::Signal;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn registration(id: &str, value: Value) -> FieldControlRegistration {
        FieldControlRegistration {
            control_ref: Rc::new(Cell::new(None)),
            id: Some(id.to_string()),
            name: Some("control-name".to_string()),
            get_value: None,
            value: Some(value),
        }
    }

    struct Harness {
        owner: Owner,
        form: FormRef,
        validity_data: RwSignal<FieldValidityData, LocalStorage>,
        change_calls: Rc<RefCell<Vec<(Option<Value>, bool)>>>,
        registered_field_name: Rc<RefCell<Option<String>>>,
        registered_field_id: Rc<RefCell<Option<String>>>,
        marked_dirty: Rc<Cell<bool>>,
        register: crate::field_root_context::RegisterFieldControlFn,
        validate: Rc<dyn Fn()>,
        commits: Rc<RefCell<Vec<Value>>>,
    }

    /// Builds the hook inside a fresh owner with fully wired params, returning the
    /// handles the host tests drive directly (the layout effects are no-ops on host —
    /// the `use_iso_layout_effect.rs` host contract — so the effect-driven freshness is
    /// pinned in the wasm suite).
    fn harness() -> Harness {
        let owner = in_owner();

        let form = use_form_context().form_ref;
        let validity_data: RwSignal<FieldValidityData, LocalStorage> =
            RwSignal::new_local(FieldValidityData::default());
        let change_calls: Rc<RefCell<Vec<(Option<Value>, bool)>>> =
            Rc::new(RefCell::new(Vec::new()));
        let registered_field_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let registered_field_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let commits: Rc<RefCell<Vec<Value>>> = Rc::new(RefCell::new(Vec::new()));

        let change: FieldChangeFn = {
            let change_calls = Rc::clone(&change_calls);
            Rc::new(move |value, cancel_pending| {
                change_calls.borrow_mut().push((value, cancel_pending));
            })
        };
        let commit: FieldCommitFn = {
            let commits = Rc::clone(&commits);
            Rc::new(move |value| commits.borrow_mut().push(value))
        };

        let field_name: Signal<Option<String>, LocalStorage> = Signal::derive_local(|| None);
        let invalid: Signal<bool, LocalStorage> = Signal::derive_local(|| false);

        let marked_dirty = Rc::new(Cell::new(false));
        let result = use_field_control_registration(UseFieldControlRegistrationParams {
            change,
            commit,
            invalid,
            marked_dirty_ref: Rc::clone(&marked_dirty),
            name: field_name,
            set_registered_field_name: {
                let registered_field_name = Rc::clone(&registered_field_name);
                Rc::new(move |name| *registered_field_name.borrow_mut() = name)
            },
            registered_field_id_ref: Rc::clone(&registered_field_id),
            validity_data: validity_data.clone(),
        });

        Harness {
            owner,
            form,
            validity_data,
            change_calls,
            registered_field_name,
            registered_field_id,
            marked_dirty,
            register: result.register,
            validate: result.validate,
            commits,
        }
    }

    // Pins the registration write (`useFieldControlRegistration.ts:133-169` over
    // `:65-78`): the entry lands in the form's registry with the field-name fallthrough,
    // the combined validity data, and the shared validate/getValue closures.
    #[test]
    fn register_writes_the_form_registry_entry() {
        let mut h = harness();

        (h.register)(
            ControlIdSource::new(),
            Some(registration("ctrl-1", json!("v1"))),
        );

        let entry = h.form.borrow().fields.get("ctrl-1").cloned();
        let Some(entry) = entry else {
            panic!("the registration landed in the form's fields map");
        };
        assert_eq!(
            entry.name.as_deref(),
            Some("control-name"),
            "the field-level name is unset, so the registration name fills in"
        );
        assert_eq!(
            (entry.get_value)(),
            Some(json!("v1")),
            "getValueForForm reads the stored value (no override)"
        );
        assert_eq!(
            entry.validity_data.state.valid, DEFAULT_VALIDITY_STATE.valid,
            "the combined validity data carries the current validity state"
        );
        assert_eq!(
            *h.registered_field_id.borrow(),
            Some("ctrl-1".to_string()),
            "registeredFieldIdRef tracks the registration"
        );

        h.owner.cleanup();
    }

    // Pins the in-place update (`useRegisterFieldControl.ts:18-20`'s comment via
    // `register` → `refreshRegistration`): a same-source, same-id re-registration — the
    // every-value-change re-run — refreshes the registration without dropping it. (A
    // *second source* instead replaces the control — see the handover tests; one field
    // owns one registration.)
    #[test]
    fn a_same_id_re_registration_keeps_the_registration_live() {
        let mut h = harness();
        let a = ControlIdSource::new();

        (h.register)(a, Some(registration("f1", json!("v1"))));
        (h.register)(a, Some(registration("f1", json!("v2"))));

        assert!(
            h.form.borrow().fields.get("f1").is_some(),
            "the re-registration kept the entry"
        );
        assert_eq!(
            (h.form.borrow().fields.get("f1").unwrap().get_value)(),
            Some(json!("v2")),
            "the refreshed value is live"
        );

        h.owner.cleanup();
    }

    // Pins the once-only baseline (`useFieldControlRegistration.ts:86-103`'s comment):
    // the initial value is captured on the first registration and never overwritten by a
    // later one — a swapped control's value must not read pristine.
    #[test]
    fn the_initial_value_baseline_is_captured_exactly_once() {
        let mut h = harness();

        (h.register)(
            ControlIdSource::new(),
            Some(registration("f1", json!("first"))),
        );
        assert_eq!(
            h.validity_data.get_untracked().initial_value,
            json!("first")
        );

        (h.register)(
            ControlIdSource::new(),
            Some(registration("f1", json!("second"))),
        );
        assert_eq!(
            h.validity_data.get_untracked().initial_value,
            json!("first"),
            "a re-registration does not re-baseline"
        );

        h.owner.cleanup();
    }

    // Pins the same-value bail-out (`:100-102`): capturing an identical baseline leaves
    // the validity-data record untouched (no extra write).
    #[test]
    fn an_identical_baseline_bails_out() {
        let mut h = harness();

        (h.register)(
            ControlIdSource::new(),
            Some(registration("f1", json!("same"))),
        );
        let before = h.validity_data.get_untracked();

        // A second registration with the same value: the functional update returns
        // `prev`, so the record is preserved verbatim (the port's guard is an
        // inequality check — module docs).
        (h.register)(
            ControlIdSource::new(),
            Some(registration("f1", json!("same"))),
        );
        assert_eq!(h.validity_data.get_untracked(), before);

        h.owner.cleanup();
    }

    // Pins the replaced-control handover (`:150-153`): a new source drops the previous
    // control's pending work (`change(undefined, true)`) and takes over the registration;
    // the first registration never fires the drop.
    #[test]
    fn a_replaced_control_drops_the_previous_work() {
        let mut h = harness();
        let first = ControlIdSource::new();
        let second = ControlIdSource::new();

        (h.register)(first, Some(registration("f1", json!("v1"))));
        assert!(
            h.change_calls.borrow().is_empty(),
            "the first registration drops nothing"
        );

        (h.register)(second, Some(registration("f1", json!("v2"))));
        assert_eq!(
            *h.change_calls.borrow(),
            vec![(None, true)],
            "the replaced control's pending work is cancelled"
        );
        assert_eq!(
            (h.form.borrow().fields.get("f1").unwrap().get_value)(),
            Some(json!("v2")),
            "the new source owns the registration"
        );

        h.owner.cleanup();
    }

    // Pins the id handover (`:162-164`): registering under a new id deletes the old
    // registry entry instead of leaking it.
    #[test]
    fn an_id_change_deletes_the_old_entry() {
        let mut h = harness();
        let source = ControlIdSource::new();

        (h.register)(source, Some(registration("old-id", json!("v1"))));
        (h.register)(source, Some(registration("new-id", json!("v2"))));

        assert!(
            h.form.borrow().fields.get("old-id").is_none(),
            "the previous id's entry is deleted"
        );
        assert!(h.form.borrow().fields.get("new-id").is_some());

        h.owner.cleanup();
    }

    // Pins the unregister branch (`:135-145`): only the active source's unregister takes
    // effect; it clears every handle and fires `change(undefined, true)`.
    #[test]
    fn unregistering_the_active_source_clears_the_registration() {
        let mut h = harness();
        let source = ControlIdSource::new();
        let other = ControlIdSource::new();

        (h.register)(source, Some(registration("f1", json!("v1"))));
        (h.register)(other, None);
        assert!(
            h.form.borrow().fields.get("f1").is_some(),
            "a foreign source's unregister is ignored"
        );
        assert!(h.change_calls.borrow().is_empty());

        (h.register)(source, None);
        assert!(
            h.form.borrow().fields.get("f1").is_none(),
            "the active source's unregister deletes the entry"
        );
        assert_eq!(*h.change_calls.borrow(), vec![(None, true)]);
        assert_eq!(*h.registered_field_name.borrow(), None);
        assert_eq!(*h.registered_field_id.borrow(), None);

        h.owner.cleanup();
    }

    // Pins `validate` (`:53-63` over `:49-51`): it marks the field dirty and commits
    // `getRegistrationValue` — the stored value when defined (the override is only
    // consulted when `value` is undefined), the current validity value when nothing is
    // registered. The override instead rides the registry entry's `getValue` — the
    // form-submit path (`:71-77`).
    #[test]
    fn validate_marks_dirty_and_commits_the_registration_value() {
        let mut h = harness();

        // Nothing registered: commit the validity data's value.
        h.validity_data.set(FieldValidityData {
            value: json!("current"),
            ..FieldValidityData::default()
        });
        (h.validate)();
        assert_eq!(*h.commits.borrow(), vec![json!("current")]);
        assert!(h.marked_dirty.get(), "validate marks the field dirty (:55)");

        // Registered with an override AND a defined value: the stored value commits, the
        // override serves the entry's submit-time getValue.
        let source = ControlIdSource::new();
        let with_override = FieldControlRegistration {
            control_ref: Rc::new(Cell::new(None)),
            id: Some("f1".to_string()),
            name: None,
            get_value: Some(Rc::new(|| Some(json!("override")))),
            value: Some(json!("stored")),
        };
        (h.register)(source, Some(with_override));
        (h.validate)();
        assert_eq!(*h.commits.borrow(), vec![json!("current"), json!("stored")]);
        assert_eq!(
            (h.form.borrow().fields.get("f1").unwrap().get_value)(),
            Some(json!("override")),
            "the registry entry's getValue consults the override"
        );

        // Registered without an override: the stored value.
        (h.register)(source, Some(registration("f1", json!("stored2"))));
        (h.validate)();
        assert_eq!(
            *h.commits.borrow(),
            vec![json!("current"), json!("stored"), json!("stored2")]
        );

        h.owner.cleanup();
    }

    // Pins the override-vs-stored fallback (`:49-51`): `value === undefined` delegates to
    // `getValueForForm`, an explicit value rides through — including an explicit null.
    #[test]
    fn the_registration_value_fallback_is_nullish_exact() {
        let mut h = harness();
        let source = ControlIdSource::new();

        // value: undefined → getValueForForm() → also undefined (no override) → the
        // baseline collapses to null (module docs).
        let undefined_value = FieldControlRegistration {
            control_ref: Rc::new(Cell::new(None)),
            id: Some("f1".to_string()),
            name: None,
            get_value: None,
            value: None,
        };
        (h.register)(source, Some(undefined_value));
        assert_eq!(
            h.validity_data.get_untracked().initial_value,
            Value::Null,
            "the undefined baseline collapses onto null"
        );

        h.owner.cleanup();
    }

    // Pins the name handover (`:157-159` over `:111`): with no field-level name, the
    // control's registration name becomes the registered field name; the registration
    // effect's `name ? undefined : registration.name` arm is pinned in the wasm suite.
    #[test]
    fn the_control_name_fills_the_registered_field_name() {
        let mut h = harness();

        (h.register)(ControlIdSource::new(), Some(registration("f1", json!("v"))));
        assert_eq!(
            *h.registered_field_name.borrow(),
            Some("control-name".to_string())
        );

        h.owner.cleanup();
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;

    use any_spawner::Executor;
    use reactive_graph::owner::{Owner, provide_context};
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{Get, Set};
    use reactive_graph::wrappers::read::Signal;
    use serde_json::json;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::wasm_bindgen::JsCast;

    use super::*;
    use crate::field_root_context::{SharedFieldRootContext, use_field_root_context};
    use crate::form_context::{FormContextValue, FormState, FormValidationMode, SharedFormContext};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn make_control(id: &str) -> Element {
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("input")
            .unwrap();
        element.set_id(id);
        element
    }

    struct WasmField {
        owner: Owner,
        form: FormRef,
        validity_data: RwSignal<FieldValidityData, LocalStorage>,
        invalid: RwSignal<bool, LocalStorage>,
        field_name: RwSignal<Option<String>, LocalStorage>,
        change_calls: Rc<RefCell<Vec<(Option<Value>, bool)>>>,
        registered_field_name: Rc<RefCell<Option<String>>>,
    }

    /// Wires the full stack inside one owner — the Phase B `Field.Root` stand-in: a
    /// private Form context carrying an isolated registry, the field-side registration
    /// hook, and the FieldRoot context publishing its `register`. The control-side hook
    /// runs in its own owner (the child control component).
    fn field() -> WasmField {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let form: FormRef = Rc::new(RefCell::new(FormState::default()));
        provide_context(SharedFormContext::new(FormContextValue {
            errors: Signal::derive_local(|| Vec::new()),
            clear_errors: Rc::new(|_| {}),
            element_ref: Rc::new(Cell::new(None)),
            form_ref: Rc::clone(&form),
            validation_mode: FormValidationMode::OnSubmit,
            submit_count_ref: Rc::new(Cell::new(0)),
        }));

        let validity_data: RwSignal<FieldValidityData, LocalStorage> =
            RwSignal::new_local(FieldValidityData::default());
        let invalid: RwSignal<bool, LocalStorage> = RwSignal::new_local(false);
        let field_name: RwSignal<Option<String>, LocalStorage> = RwSignal::new_local(None);
        let change_calls: Rc<RefCell<Vec<(Option<Value>, bool)>>> =
            Rc::new(RefCell::new(Vec::new()));
        let registered_field_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

        let change: FieldChangeFn = {
            let change_calls = Rc::clone(&change_calls);
            Rc::new(move |value, cancel_pending| {
                change_calls.borrow_mut().push((value, cancel_pending));
            })
        };

        let result = use_field_control_registration(UseFieldControlRegistrationParams {
            change,
            commit: Rc::new(|_| {}),
            invalid: invalid.clone(),
            marked_dirty_ref: Rc::new(Cell::new(false)),
            name: field_name.clone(),
            set_registered_field_name: {
                let registered_field_name = Rc::clone(&registered_field_name);
                Rc::new(move |name| *registered_field_name.borrow_mut() = name)
            },
            registered_field_id_ref: Rc::new(RefCell::new(None)),
            validity_data: validity_data.clone(),
        });

        // The Field.Root stand-in: the default shell with the real `register` swapped in
        // (what the provider value construction does).
        let mut context = use_field_root_context();
        context.register_field_control = result.register;
        provide_context(SharedFieldRootContext::new(context));

        WasmField {
            owner,
            form,
            validity_data,
            invalid,
            field_name,
            change_calls,
            registered_field_name,
        }
    }

    struct WasmControl {
        owner: Owner,
        value: RwSignal<Option<Value>, LocalStorage>,
        enabled: RwSignal<bool, LocalStorage>,
        id: RwSignal<Option<String>, LocalStorage>,
    }

    /// The Phase B control stand-in: runs `use_register_field_control` in a child owner
    /// with reactive id/value/enabled/name sources. The control owner is created under
    /// the *field* owner — re-entering it first — so sibling controls never nest inside
    /// each other and one control's cleanup cannot cascade into another's (React's
    /// component tree).
    fn control(
        field: &WasmField,
        field_name_hint: Option<String>,
        id: &str,
        value: Value,
    ) -> WasmControl {
        field.owner.set();
        let control_owner = Owner::new();
        control_owner.set();
        let element = make_control(id);

        let value: RwSignal<Option<Value>, LocalStorage> = RwSignal::new_local(Some(value));
        let enabled: RwSignal<bool, LocalStorage> = RwSignal::new_local(true);
        let id: RwSignal<Option<String>, LocalStorage> = RwSignal::new_local(Some(id.to_string()));

        use_register_field_control(UseRegisterFieldControlParams {
            control_ref: Rc::new(Cell::new(Some(element))),
            id: id.clone(),
            value: value.clone(),
            get_form_value_override: None,
            enabled: enabled.clone(),
            name: RwSignal::new_local(field_name_hint),
        });

        WasmControl {
            owner: control_owner,
            value,
            enabled,
            id,
        }
    }

    // Pins the end-to-end registration: the control's layout effect runs synchronously
    // during the hook call, so the form's registry carries the entry — with the
    // registration's name, the combined validity data, and the live value — by the time
    // the control hook returns (`useRegisterFieldControl.ts:21-38` over
    // `useFieldControlRegistration.ts:133-169`).
    #[wasm_bindgen_test]
    fn the_control_registers_end_to_end_into_the_form_registry() {
        let field = field();
        let control = control(
            &field,
            Some("control-name".to_string()),
            "ctrl-1",
            json!("v1"),
        );

        {
            let fields = field.form.borrow();
            let entry = fields.fields.get("ctrl-1").expect("the registered entry");
            assert_eq!(entry.name.as_deref(), Some("control-name"));
            assert_eq!((entry.get_value)(), Some(json!("v1")));
            assert_eq!(entry.validity_data.state.valid, None);
            let carried = entry.control_ref.replace(None);
            entry.control_ref.set(carried.clone());
            assert!(carried.is_some(), "the entry carries the control ref slot");
        }
        assert_eq!(
            *field.registered_field_name.borrow(),
            Some("control-name".to_string())
        );

        control.owner.cleanup();
        field.owner.cleanup();
    }

    // Pins the value freshness (the `:38` dep array's `value` member): a value-signal
    // change re-runs the registration effect, and the refreshed value is live through
    // the entry's `getValue` — the submit-path correctness rule.
    #[wasm_bindgen_test]
    fn a_value_change_re_registers_with_the_fresh_value() {
        let field = field();
        let control = control(
            &field,
            Some("control-name".to_string()),
            "ctrl-1",
            json!("v1"),
        );

        control.value.set(Some(json!("v2")));
        Executor::poll_local();

        assert_eq!(
            (field.form.borrow().fields.get("ctrl-1").unwrap().get_value)(),
            Some(json!("v2")),
            "the re-registration made the fresh value live"
        );

        control.owner.cleanup();
        field.owner.cleanup();
    }

    // Pins the disabled arm (`:24-27`): flipping `enabled` off registers `undefined`,
    // which the field answers by deleting the entry and cancelling pending work; flipping
    // back on re-registers.
    #[wasm_bindgen_test]
    fn disabling_the_control_unregisters_it() {
        let field = field();
        let control = control(
            &field,
            Some("control-name".to_string()),
            "ctrl-1",
            json!("v1"),
        );
        assert!(field.form.borrow().fields.get("ctrl-1").is_some());

        control.enabled.set(false);
        Executor::poll_local();
        assert!(
            field.form.borrow().fields.get("ctrl-1").is_none(),
            "the disabled control is unregistered"
        );
        assert_eq!(
            *field.change_calls.borrow(),
            vec![(None, true)],
            "the unregister branch cancels pending work"
        );

        control.enabled.set(true);
        Executor::poll_local();
        assert!(
            field.form.borrow().fields.get("ctrl-1").is_some(),
            "re-enabling re-registers"
        );

        control.owner.cleanup();
        field.owner.cleanup();
    }

    // Pins the unmount cleanups (`useRegisterFieldControl.ts:40-45`,
    // `useFieldControlRegistration.ts:122-131`): owner disposal unregisters the control —
    // the active-source branch deletes the registry entry.
    #[wasm_bindgen_test]
    fn disposal_unregisters_the_control_from_the_registry() {
        let field = field();
        let control = control(
            &field,
            Some("control-name".to_string()),
            "ctrl-1",
            json!("v1"),
        );
        assert!(field.form.borrow().fields.get("ctrl-1").is_some());

        control.owner.cleanup();
        Executor::poll_local();

        assert!(
            field.form.borrow().fields.get("ctrl-1").is_none(),
            "the unmount cleanup deleted the registry entry"
        );

        field.owner.cleanup();
    }

    // Pins the tracked freshness of the registry entry (`useFieldControlRegistration.ts:105-120`
    // dep array): a validity-data or invalid change re-runs the registration effect, and
    // the entry's combined validity data follows — including the external-invalidity
    // override.
    #[wasm_bindgen_test]
    fn validity_changes_refresh_the_registry_entry() {
        let field = field();
        let _control = control(
            &field,
            Some("control-name".to_string()),
            "ctrl-1",
            json!("v1"),
        );

        // A computed validity change rewrites the entry.
        field.validity_data.set(FieldValidityData {
            state: crate::field_constants::FieldValidityState {
                valid: Some(true),
                ..crate::field_constants::DEFAULT_VALIDITY_STATE
            },
            ..FieldValidityData::default()
        });
        Executor::poll_local();
        assert_eq!(
            field
                .form
                .borrow()
                .fields
                .get("ctrl-1")
                .unwrap()
                .validity_data
                .state
                .valid,
            Some(true),
            "the entry's validity data follows the field state"
        );

        // External invalidity forces `valid: false` in the combined entry.
        field.invalid.set(true);
        Executor::poll_local();
        assert_eq!(
            field
                .form
                .borrow()
                .fields
                .get("ctrl-1")
                .unwrap()
                .validity_data
                .state
                .valid,
            Some(false),
            "the combined validity reflects the external invalid flag"
        );

        field.owner.cleanup();
    }

    // Pins the name handover through the effect (`:111`): a field-level name clears the
    // registered field name (the field's own name wins) and takes over the registry
    // entry's name.
    #[wasm_bindgen_test]
    fn a_field_level_name_takes_over_the_registration_name() {
        let field = field();
        let _control = control(
            &field,
            Some("control-name".to_string()),
            "ctrl-1",
            json!("v1"),
        );
        assert_eq!(
            *field.registered_field_name.borrow(),
            Some("control-name".to_string())
        );

        field.field_name.set(Some("field-name".to_string()));
        Executor::poll_local();

        assert_eq!(
            *field.registered_field_name.borrow(),
            None,
            "the field-level name clears the registered control name"
        );
        assert_eq!(
            field
                .form
                .borrow()
                .fields
                .get("ctrl-1")
                .unwrap()
                .name
                .as_deref(),
            Some("field-name"),
            "the registry entry carries the field-level name"
        );

        field.owner.cleanup();
    }

    // Pins the id handover (`:162-164`) through the real wiring: a control whose id
    // changes deletes the old registry entry and registers under the new id.
    #[wasm_bindgen_test]
    fn an_id_change_moves_the_registry_entry() {
        let field = field();
        let control = control(
            &field,
            Some("control-name".to_string()),
            "old-id",
            json!("v1"),
        );
        assert!(field.form.borrow().fields.get("old-id").is_some());

        control.id.set(Some("new-id".to_string()));
        Executor::poll_local();

        assert!(field.form.borrow().fields.get("old-id").is_none());
        assert!(field.form.borrow().fields.get("new-id").is_some());

        control.owner.cleanup();
        field.owner.cleanup();
    }

    // Pins the replaced-control handover in the real wiring (`:150-153`): a second
    // control registering in the same field drops the first control's pending work and
    // takes over; the replaced control's later unregister is a no-op (the foreign-source
    // guard at `:136`).
    #[wasm_bindgen_test]
    fn a_second_control_takes_over_and_the_replaced_one_cannot_unregister() {
        let field = field();
        let first = control(&field, Some("first".to_string()), "ctrl-1", json!("v1"));
        let change_calls_before = field.change_calls.borrow().len();

        let second = control(&field, Some("second".to_string()), "ctrl-2", json!("v2"));
        assert_eq!(
            field.change_calls.borrow().last(),
            Some(&(None, true)),
            "the replaced control's pending work was cancelled"
        );
        let _ = change_calls_before;
        assert!(field.form.borrow().fields.get("ctrl-1").is_none());
        assert!(field.form.borrow().fields.get("ctrl-2").is_some());

        first.owner.cleanup();
        Executor::poll_local();
        assert!(
            field.form.borrow().fields.get("ctrl-2").is_some(),
            "the replaced control's unregister cannot touch the active registration"
        );

        second.owner.cleanup();
        field.owner.cleanup();
    }

    // Pins the stray-control arm (module docs): outside a provider the registration is
    // the shared no-op, so nothing lands anywhere.
    #[wasm_bindgen_test]
    fn a_stray_control_is_inert() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let shared_form = use_form_context();
        let before = shared_form.form_ref.borrow().fields.len();

        // The control hook under its own child owner (the component boundary).
        let control_owner = Owner::new();
        control_owner.set();
        use_register_field_control(UseRegisterFieldControlParams {
            control_ref: Rc::new(Cell::new(Some(make_control("stray")))),
            id: RwSignal::new_local(Some("stray".to_string())),
            value: RwSignal::new_local(Some(json!("v"))),
            get_form_value_override: None,
            enabled: RwSignal::new_local(true),
            name: RwSignal::new_local(None),
        });
        assert_eq!(
            shared_form.form_ref.borrow().fields.len(),
            before,
            "the no-op registration keeps the stray control inert"
        );

        control_owner.cleanup();
        owner.cleanup();
    }
}
