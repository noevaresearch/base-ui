//! Port of `packages/react/src/checkbox-group/CheckboxGroup.tsx` +
//! `useCheckboxGroupParent.ts` + `CheckboxGroupContext.ts` — the `library: checkbox-group`
//! TODO item (`specs/library/checkbox-group/behavior.md`,
//! `specs/library/checkbox-group/implementation.md`).
//!
//! Upstream structure (implementation.md):
//!
//! - **One root element, one context.** `useRenderElement('div', …)` with `state` =
//!   `{ ...fieldState, disabled }` (`CheckboxGroup.tsx:153-155`), the props array
//!   `[baseProps, elementProps, getDescriptionProps]` (`:161-167`), and
//!   `stateAttributesMapping: fieldValidityMapping` — wrapped in the
//!   `CheckboxGroupContext.Provider` (`:173`). The context value is
//!   `{ allValues, value, setValue, parent, disabled, validation, registerControlId }`
//!   (`:157-165`), and `CheckboxRoot.tsx:89` is its sole consumer (the "Cross-component
//!   contract" in implementation.md): the port reproduces the provider/consumer
//!   contract, not just the rendering.
//! - **useCheckboxGroupParent** (`useCheckboxGroupParent.ts:8-175`) is the parent-checkbox
//!   engine: `checked`/`indeterminate` derived from `value.length` vs
//!   `allValues.length` (`:19-20`), a `Map`-backed child-id registry
//!   (`:22-26`, the comment's prototype-pollution note) driving `getParentProps`'s
//!   space-joined `aria-controls` (`:62-65`), an `uncontrolledStateRef` snapshot
//!   (`:14`), a `disabledStatesRef` map (`:15`), and a
//!   `'mixed' → 'on' → 'off' → mixed…` status cycle where a parent click from any
//!   other state proposes `all` / `none` filtered by the disabled bookkeeping
//!   (`:70-116`) — disabled children that are checked are held (the `none`/`all`
//!   filter arms, `:75-84`), and `setStatus` is deferred behind the
//!   `!eventDetails.isCanceled` gate (`:117-120`).
//! - **The group is the Field's control** (`CheckboxGroup.tsx:88-118`):
//!   `useLabelableId({ id: null })` (`:86-87`) suppresses per-child `htmlFor` (the
//!   JSDoc comment at `:84-86`), `useRegisterFieldControl(controlRef, id, value,
//!   getFormValue, !!fieldName && !disabled, fieldName)` registers the group with the
//!   projected `getFormValue` filter (`:76-118`), the `useIsoLayoutEffect`
//!   `setFilled(value.length > 0)` (`:120-122`), and the `useValueChanged` block —
//!   `clearErrors(fieldName)`, `setDirty(!areArraysEqual(value, initialValue))`,
//!   `validation.change(value)` — runs only when a real change lands
//!   (`:124-141`).
//! - **setValue is the stable veto wrapper** (`:60-72`): `onValueChange` first, the
//!   `isCanceled` short-circuit, `setValueUnwrapped` last.
//!
//! ## Rust adaptations
//!
//! - React context ports to the reactive-owner `provide_context` bridge (the
//!   `SendWrapper` convention — `field/field_root.rs`, `form_context.rs`). The port
//!   exports [`SharedCheckboxGroupContext`] and the
//!   [`use_checkbox_group_context`] accessor; absence (`None`) is upstream's
//!   `createContext<CheckboxGroupContext | undefined>` default
//!   (`CheckboxGroupContext.ts:26-28`), so a child checkbox outside a group renders
//!   standalone.
//! - `useCheckboxGroupParent`'s refs become `Rc<RefCell<…>>` interiors behind an
//!   owned context value (the interior-mutability convention of the store ports);
//!   the React `useState` pairs (`status`, `childIdsState`) become `RwSignal`s so
//!   the `aria-controls`/`checked`/`indeterminate` consumers re-derive reactively.
//! - `useStableCallback` has no Rust aliasing problem: the closures capture clones
//!   and are `Rc`-stable by construction.
//! - The value array rides through [`leptos_ui_utils::use_controlled`], whose read
//!   signal the context exposes to children (they only read membership; writes go
//!   through the veto-wrapped `setValue`).
//! - Upstream's `controlRef` getter object (`:93-104`, a `get current()`) has no
//!   Rust property-getter form: the port re-resolves `getInputControl()` in a
//!   layout effect keyed on the value signal — the same moments upstream's dep
//!   array re-memos the getter (`:100-104`) — and stores the result in the
//!   registration `Cell`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::{LocalStorage, use_context};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use serde_json::Value;
use web_sys::wasm_bindgen::JsCast;

use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::floating_ui::element_props::ElementAttributeFn;
use leptos_ui_internals::labelable_provider::RegisterControlIdFn;
use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::state_attributes::field_validity_mapping;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_render_element::{
    RenderElementHandlers, RenderElementProps, RenderedElement, UseRenderElementComponentProps,
    UseRenderElementParams, use_render_element,
};
use leptos_ui_internals::use_value_changed::use_value_changed;
use leptos_ui_utils::are_arrays_equal::are_arrays_equal;
use leptos_ui_utils::use_controlled::{UseControlledProps, use_controlled};

use crate::field::context::use_field_root_context;
use crate::field::validation::FieldValidation;

// ---------------------------------------------------------------------------
// The context (CheckboxGroupContext.ts)
// ---------------------------------------------------------------------------

/// The change-event details type — upstream's
/// `BaseUIChangeEventDetails<CheckboxGroup.ChangeEventReason>` with
/// `ChangeEventReason = REASONS.none` (`CheckboxGroup.tsx:218-220`). The payload
/// element type is a placeholder (the `REASONS.none` reason carries no payload),
/// mirroring the toggle port's `BaseUIChangeEventDetails<(), MouseEvent>`.
pub type CheckboxGroupChangeEventDetails = BaseUIChangeEventDetails<(), ()>;

/// The `setValue`/`onValueChange` callback shape (`CheckboxGroup.tsx:62`):
/// `(value: string[], eventDetails)`.
pub type OnGroupValueChange = Rc<dyn Fn(Vec<String>, &CheckboxGroupChangeEventDetails)>;

/// The parent-checkbox engine's return — upstream's
/// `UseCheckboxGroupParentReturnValue` (`useCheckboxGroupParent.ts:158-175`), exposed
/// on the context exactly as upstream stores the whole object
/// (`CheckboxGroupContext.ts:11`).
#[derive(Clone)]
pub struct CheckboxGroupParent {
    /// `disabledStatesRef` (`:159`) — child value → disabled, published by each
    /// checkbox (`CheckboxRoot.tsx:265-276`); the parent's toggle filter reads it.
    pub disabled_states: Rc<RefCell<std::collections::HashMap<String, bool>>>,
    /// `childIdsState` (`:22-26`) — the `Map`-backed child-id registry as a *tracked*
    /// seam. `get_parent_props` reads it untracked (the engine's per-call snapshot);
    /// the parent checkbox's live `aria-controls` binding reads this handle so the
    /// attribute lands once a child registers (`specs/library/checkbox/implementation.md`
    /// , "Accessibility": the parent aggregates the child ids).
    pub child_ids:
        RwSignal<Rc<RefCell<std::collections::HashMap<String, Vec<String>>>>, LocalStorage>,
    /// `registerChildId` (`:161-163`): `(childValue, childId)` registers; the
    /// returned closure unregisters (the React cleanup return).
    pub register_child_id: Rc<dyn Fn(&str, &str) -> Rc<dyn Fn()>>,
    /// `getParentProps` (`:165-174`) — the parent checkbox's props factory,
    /// snapshotted per call (the React render-time memo analog).
    pub get_parent_props: Rc<dyn Fn() -> ParentPropsSnapshot>,
    /// `getChildProps` (`:176-182`) — the per-child factory.
    pub get_child_props: Rc<dyn Fn(&str) -> ChildPropsSnapshot>,
}

/// `getParentProps()`'s return shape (`useCheckboxGroupParent.ts:166-173`) at the
/// call instant — the React render-scope object as a plain snapshot.
#[derive(Clone)]
pub struct ParentPropsSnapshot {
    /// `indeterminate` (`:167`).
    pub indeterminate: bool,
    /// `checked` (`:168`).
    pub checked: bool,
    /// `aria-controls` (`:169-171`) — the space-joined registered child ids;
    /// `None` is the JS `undefined` (empty join).
    pub aria_controls: Option<String>,
    /// `onCheckedChange` (`:172`) — `(checked, eventDetails)`.
    pub on_checked_change: Rc<dyn Fn(bool, &CheckboxGroupChangeEventDetails)>,
}

/// `getChildProps(value)`'s return shape (`useCheckboxGroupParent.ts:177-181`).
#[derive(Clone)]
pub struct ChildPropsSnapshot {
    /// `checked` (`:178`) — the child's membership in the group value.
    pub checked: bool,
    /// `onCheckedChange` (`:179`) — `(nextChecked, eventDetails)`; the engine
    /// splices the child's value in/out of the group value.
    pub on_checked_change: Rc<dyn Fn(bool, &CheckboxGroupChangeEventDetails)>,
}

/// The group context value — upstream's `CheckboxGroupContext`
/// (`CheckboxGroupContext.ts:9-24`), the provider/consumer contract with
/// `CheckboxRoot.tsx:89` (implementation.md, "Cross-component contract").
#[derive(Clone)]
pub struct CheckboxGroupContextValue {
    /// `value` (`:10`) — the current checked-children array (read side; children
    /// only read membership).
    pub value: Signal<Vec<String>, LocalStorage>,
    /// `setValue` (`:11-14`) — the veto wrapper: `onValueChange` first, the
    /// `isCanceled` short-circuit, the unwrapped write last
    /// (`CheckboxGroup.tsx:60-72`).
    pub set_value: Rc<dyn Fn(Vec<String>, &CheckboxGroupChangeEventDetails)>,
    /// `allValues` (`:15`) — the parent-checkbox contract's full set; `None` is
    /// the JS `undefined`.
    pub all_values: Option<Vec<String>>,
    /// `parent` (`:16`) — `Some` only when `allValues` was provided (upstream
    /// always runs the hook but the parent machinery is inert without the full
    /// set — `CheckboxGroup.tsx:71-74` + the `allValues = EMPTY_ARRAY` default).
    pub parent: Option<CheckboxGroupParent>,
    /// `disabled` (`:17`) — the group-level flag.
    pub disabled: bool,
    /// `validation` (`:18`) — the field validation machine shared to children.
    pub validation: FieldValidation,
    /// `registerControlId` (`:19-23`) — the labelable scope the group renders
    /// in; a checkbox seeing the same function shares that scope, so the group —
    /// not the checkbox — is the field's control (the JSDoc, `:20-23`).
    pub register_control_id: RegisterControlIdFn,
}

/// The context type provided through the reactive owner — the `SendWrapper`
/// bridge (`SharedFormContext`/`SharedLabelableContext` precedent: the Rc-heavy
/// bag cannot satisfy `provide_context`'s `Send + Sync` directly).
pub type SharedCheckboxGroupContext = SendWrapper<CheckboxGroupContextValue>;

/// `useCheckboxGroupContext()` (`CheckboxGroupContext.ts:30-32`): the child
/// accessor; `None` outside a provider is upstream's `undefined` default.
pub fn use_checkbox_group_context() -> Option<CheckboxGroupContextValue> {
    use_context::<SharedCheckboxGroupContext>().map(|shared| (*shared).clone())
}

/// The provider seam (`CheckboxGroup.tsx:173`): wraps the rendered element in the
/// `CheckboxGroupContext.Provider`. The caller owns the scope; the port exposes
/// the same bridge the other Rc-heavy contexts use.
pub fn provide_checkbox_group_context(value: CheckboxGroupContextValue) {
    reactive_graph::owner::provide_context(SharedCheckboxGroupContext::new(value));
}

// ---------------------------------------------------------------------------
// useCheckboxGroupParent (useCheckboxGroupParent.ts:8-175)
// ---------------------------------------------------------------------------

/// Port of `useCheckboxGroupParent` (`useCheckboxGroupParent.ts:8-175`). Must be
/// called inside a reactive owner. `value` is the group's reactive value source;
/// `on_value_change` is the group's veto-wrapped `setValue`.
pub fn use_checkbox_group_parent(
    all_values: &[String],
    value: Signal<Vec<String>, LocalStorage>,
    on_value_change: OnGroupValueChange,
) -> CheckboxGroupParent {
    // `uncontrolledStateRef = useRef(value)` (`:14`): the last *committed*
    // snapshot, seeded from the initial value. Child commits update it behind the
    // `!isCanceled` gate (`:140-143`); the parent arm reads it at click time
    // (`:72`).
    let uncontrolled_state = Rc::new(RefCell::new(value.get_untracked()));

    // `disabledStatesRef = useRef(new Map())` (`:15`).
    let disabled_states = Rc::new(RefCell::new(
        std::collections::HashMap::<String, bool>::new(),
    ));

    // `const [status, setStatus] = useState('mixed')` (`:16`).
    let status: RwSignal<&'static str, LocalStorage> = RwSignal::new_local("mixed");

    // `const [childIdsState, setChildIdsState] = useState(…Map…)` (`:22-26`) —
    // the `Map` (not an object) so a value like `constructor` reads as data, not
    // off `Object.prototype` (the `:21-22` comment; behavior.md, parent
    // `aria-controls` "prototype-key names … do not leak").
    let child_ids: RwSignal<
        Rc<RefCell<std::collections::HashMap<String, Vec<String>>>>,
        LocalStorage,
    > = RwSignal::new_local(Rc::new(RefCell::new(std::collections::HashMap::new())));

    let all_values = all_values.to_vec();

    // `getParentProps` (`:57-122`), snapshotted per call.
    let get_parent_props = {
        let value_for_parent = value.clone();
        let uncontrolled_for_parent = Rc::clone(&uncontrolled_state);
        let disabled_for_parent = Rc::clone(&disabled_states);
        let child_ids_for_parent = child_ids.clone();
        let status_for_parent = status.clone();
        let all_for_parent = all_values.clone();
        let on_value_change_for_parent = Rc::clone(&on_value_change);

        Rc::new(move || {
            let current_value = value_for_parent.get_untracked();
            let uncontrolled = uncontrolled_for_parent.borrow().clone();

            // `checked = value.length === allValues.length`,
            // `indeterminate = value.length !== allValues.length && value.length > 0`
            // (`:19-20`) — plus upstream's `allValues = EMPTY_ARRAY` default
            // (`:10`), so an empty full-set is "all checked" (0 === 0).
            let checked = current_value.len() == all_for_parent.len();
            let indeterminate =
                current_value.len() != all_for_parent.len() && !current_value.is_empty();

            // `aria-controls` (`:62-65`): every registered id per value in
            // `allValues` order, space-joined; an empty join is `undefined`.
            let registry = child_ids_for_parent.get_untracked();
            let ids: Vec<String> = all_for_parent
                .iter()
                .filter_map(|v| registry.borrow().get(v).cloned())
                .flatten()
                .collect();
            let aria_controls = if ids.is_empty() {
                None
            } else {
                Some(ids.join(" "))
            };

            let on_checked_change = {
                let uncontrolled_for_change = Rc::clone(&uncontrolled_for_parent);
                let disabled_for_change = Rc::clone(&disabled_for_parent);
                let status_for_change = status_for_parent.clone();
                let all_for_change = all_for_parent.clone();
                let value_for_change = value_for_parent.clone();
                let on_value_change = Rc::clone(&on_value_change_for_parent);
                Rc::new(
                    move |_checked: bool, event_details: &CheckboxGroupChangeEventDetails| {
                        let uncontrolled_state = uncontrolled_for_change.borrow().clone();

                        // `none` (`:75-77`): the disabled ones that are checked — they
                        // can't be changed, so they are held when unchecking all.
                        let none: Vec<String> = all_for_change
                            .iter()
                            .filter(|v| {
                                disabled_for_change
                                    .borrow()
                                    .get(*v)
                                    .copied()
                                    .unwrap_or(false)
                                    && uncontrolled_state.contains(*v)
                            })
                            .cloned()
                            .collect();

                        // `all` (`:80-83`): everything not disabled, plus the checked
                        // disabled ones held above.
                        let all: Vec<String> = all_for_change
                            .iter()
                            .filter(|v| {
                                !disabled_for_change
                                    .borrow()
                                    .get(*v)
                                    .copied()
                                    .unwrap_or(false)
                                    || uncontrolled_state.contains(*v)
                            })
                            .cloned()
                            .collect();

                        // `allOnOrOff` (`:85-86`): the snapshot is already at a pole —
                        // propose the opposite pole directly.
                        let all_on_or_off =
                            uncontrolled_state.len() == all.len() || uncontrolled_state.is_empty();

                        if all_on_or_off {
                            if value_for_change.get_untracked().len() == all.len() {
                                on_value_change(none, event_details);
                            } else {
                                on_value_change(all, event_details);
                            }
                            return;
                        }

                        // The status cycle (`:88-105`): mixed proposes `all`, `on`
                        // proposes `none`, `off` keeps the value mixed (the pre-`all`
                        // snapshot restores through the child commits that updated
                        // `uncontrolledStateRef`).
                        let mut next_status = "mixed";
                        let mut next_value = uncontrolled_state;

                        if status_for_change.get_untracked() == "mixed" {
                            next_status = "on";
                            next_value = all;
                        } else if status_for_change.get_untracked() == "on" {
                            next_status = "off";
                            next_value = none;
                        }

                        on_value_change(next_value, event_details);

                        // `if (!eventDetails.isCanceled) setStatus(nextStatus)`
                        // (`:117-120`).
                        if !event_details.is_canceled() {
                            status_for_change.set(next_status);
                        }
                    },
                ) as Rc<dyn Fn(bool, &CheckboxGroupChangeEventDetails)>
            };

            ParentPropsSnapshot {
                indeterminate,
                checked,
                aria_controls,
                on_checked_change,
            }
        })
    };

    // `getChildProps` (`:124-148`), snapshotted per child per call.
    let get_child_props = {
        let value_for_child = value.clone();
        let uncontrolled_for_child = Rc::clone(&uncontrolled_state);
        let status_for_child = status.clone();
        let on_value_change_for_child = Rc::clone(&on_value_change);
        Rc::new(move |child_value: &str| {
            let checked = value_for_child
                .get_untracked()
                .iter()
                .any(|v| v == child_value);
            let child_value = child_value.to_string();
            let on_checked_change = {
                let value_for_change = value_for_child.clone();
                let uncontrolled_for_change = Rc::clone(&uncontrolled_for_child);
                let status_for_change = status_for_child.clone();
                let on_value_change = Rc::clone(&on_value_change_for_child);
                Rc::new(
                    move |next_checked: bool, event_details: &CheckboxGroupChangeEventDetails| {
                        // The splice (`:131-137`): push on check, remove on uncheck.
                        let mut new_value = value_for_change.get_untracked();
                        if next_checked {
                            if !new_value.contains(&child_value) {
                                new_value.push(child_value.clone());
                            }
                        } else if let Some(position) =
                            new_value.iter().position(|v| v == &child_value)
                        {
                            new_value.remove(position);
                        }

                        on_value_change(new_value.clone(), event_details);

                        // `if (!eventDetails.isCanceled)` (`:139-143`): the snapshot
                        // and the status advance only on a committed change — a
                        // canceled child uncheck leaves the pre-"all" snapshot
                        // intact (behavior.md, parent snapshot not polluted).
                        if !event_details.is_canceled() {
                            *uncontrolled_for_change.borrow_mut() = new_value;
                            status_for_change.set("mixed");
                        }
                    },
                ) as Rc<dyn Fn(bool, &CheckboxGroupChangeEventDetails)>
            };
            ChildPropsSnapshot {
                checked,
                on_checked_change,
            }
        })
    };

    // `registerChildId` (`:27-54`): append-on-new (deduped), cleanup filters the
    // id back out (and drops the value entry when empty), both re-publishing the
    // registry wrapper (`setChildIdsState`) so `aria-controls` re-derives.
    let register_child_id: Rc<dyn Fn(&str, &str) -> Rc<dyn Fn()>> = {
        let child_ids = child_ids.clone();
        Rc::new(move |child_value: &str, child_id: &str| {
            let registry = child_ids.get_untracked();
            let needs_update = {
                let registry = registry.borrow();
                match registry.get(child_value) {
                    Some(ids) => !ids.iter().any(|id| id == child_id),
                    None => true,
                }
            };
            if needs_update {
                registry
                    .borrow_mut()
                    .entry(child_value.to_string())
                    .or_default()
                    .push(child_id.to_string());
                // Replace only the wrapper (`:25-26`): re-publish without
                // cloning the growing registry.
                child_ids.set(registry.clone());
            }

            let child_ids = child_ids.clone();
            let child_value = child_value.to_string();
            let child_id = child_id.to_string();
            Rc::new(move || {
                let registry = child_ids.get_untracked();
                let mut registry = registry.borrow_mut();
                let Some(ids) = registry.get_mut(&child_value) else {
                    return;
                };
                if !ids.iter().any(|id| id == &child_id) {
                    return;
                }
                ids.retain(|id| id != &child_id);
                if ids.is_empty() {
                    registry.remove(&child_value);
                }
                drop(registry);
                let republished = child_ids.get_untracked();
                child_ids.set(republished);
            })
        })
    };

    CheckboxGroupParent {
        disabled_states,
        child_ids,
        register_child_id,
        get_parent_props,
        get_child_props,
    }
}

// ---------------------------------------------------------------------------
// CheckboxGroup (CheckboxGroup.tsx:29-175)
// ---------------------------------------------------------------------------

/// The group props — upstream's `CheckboxGroupProps` destructure
/// (`CheckboxGroup.tsx:31-43`) with the documented defaults.
pub struct CheckboxGroupProps {
    /// `value` (`:56-59`) — controlled; `None` while uncontrolled.
    pub value: Option<Vec<String>>,
    /// `defaultValue` (`:64-67`) — the uncontrolled seed; `None` is upstream's
    /// `?? EMPTY_ARRAY` (the `defaultValue={null}` tolerance, behavior.md
    /// "State model").
    pub default_value: Option<Vec<String>>,
    /// `onValueChange` (`:68-71`).
    pub on_value_change: Option<OnGroupValueChange>,
    /// `allValues` (`:72-74`) — the parent-checkbox contract's full set.
    pub all_values: Option<Vec<String>>,
    /// `disabled` (`:75-78`, default `false`).
    pub disabled: bool,
    /// `id` (`:36`) — forwarded to the root element.
    pub id: Option<String>,
    /// `className`/`style`/`render` — the [`UseRenderElementComponentProps`]
    /// vocabulary.
    pub render_class_style: UseRenderElementComponentProps,
    /// The `...elementProps` rest (`:43`) — static attributes spread onto the
    /// root.
    pub element_attributes: Vec<(String, String)>,
}

impl Default for CheckboxGroupProps {
    fn default() -> Self {
        Self {
            value: None,
            default_value: None,
            on_value_change: None,
            all_values: None,
            disabled: false,
            id: None,
            render_class_style: UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
        }
    }
}

/// Builds the CheckboxGroup element description — upstream's `CheckboxGroup`
/// body (`CheckboxGroup.tsx:29-175`) up to the `useRenderElement` call, without
/// materializing a DOM node. Must be called inside a reactive owner; the caller
/// provides the context value (returned on the handle's scope) around the
/// rendered element (`CheckboxGroup.tsx:173`).
pub fn checkbox_group_element(props: CheckboxGroupProps) -> RenderedElement {
    let CheckboxGroupProps {
        value: external_value,
        default_value,
        on_value_change,
        all_values,
        disabled: disabled_prop,
        id: id_prop,
        render_class_style,
        element_attributes,
    } = props;

    // The Field/Form/Labelable context reads (`:45-53`).
    let field_context = use_field_root_context();
    let form_context = leptos_ui_internals::form_context::use_form_context();

    // `const disabled = fieldDisabled || disabledProp` (`:55`).
    let disabled =
        leptos::prelude::GetUntracked::get_untracked(&field_context.disabled) || disabled_prop;

    // `const defaultValue = defaultValueProp ?? EMPTY_ARRAY` (`:56`).
    let default_value = default_value.unwrap_or_default();

    // The value duality (`:58-63`).
    let (value, set_value_unwrapped) = use_controlled(UseControlledProps::new(
        RwSignal::new(external_value),
        RwSignal::new(default_value),
        "CheckboxGroup",
    ));

    // `setValue` — the stable veto wrapper (`:60-72`): `onValueChange` first,
    // the `isCanceled` short-circuit, `setValueUnwrapped` last.
    let set_value: OnGroupValueChange = {
        let on_value_change = on_value_change.clone();
        Rc::new(
            move |next_value: Vec<String>, event_details: &CheckboxGroupChangeEventDetails| {
                if let Some(callback) = &on_value_change {
                    callback(next_value.clone(), event_details);
                }
                if event_details.is_canceled() {
                    return;
                }
                set_value_unwrapped(leptos_ui_utils::use_controlled::SetValueAction::Value(
                    next_value,
                ));
            },
        )
    };

    // `const parent = useCheckboxGroupParent({ allValues, value, onValueChange:
    // setValue })` (`:71-75`).
    let parent = use_checkbox_group_parent(
        all_values.as_deref().unwrap_or(&[]),
        value.clone(),
        Rc::clone(&set_value),
    );

    // `useLabelableId({ id: null })` (`:86-87`): the group is the field's
    // control, so it does not own a per-child `htmlFor` — the JSDoc comment at
    // `:84-86`. The port's `enabled: false` is the `null` id's suppression arm.
    let _ = leptos_ui_internals::labelable_provider::use_labelable_id(
        leptos_ui_internals::labelable_provider::UseLabelableIdParams {
            id: None,
            enabled: false,
        },
    );

    // `const id = useBaseUiId(idProp)` (`:89`).
    let id = use_base_ui_id(RwSignal::new(id_prop.clone()));

    // `getInputControl` → `controlRef` (`:91-104`): the ref resolves through the
    // validation machine's representative-input getter. The getter-object form
    // has no Rust property-getter analog; the port re-resolves in a layout
    // effect keyed on the value signal (the dep array at `:100-104`'s
    // re-memo moments) and stores the result in the registration `Cell`.
    let control_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    {
        let control_ref = Rc::clone(&control_ref);
        let get_input_control = Rc::clone(&field_context.validation.get_input_control);
        let value_for_ref = value.clone();
        leptos_ui_utils::use_iso_layout_effect::use_iso_layout_effect(move || {
            let _ = value_for_ref.get_untracked();
            let resolved = get_input_control()
                .map(|element| element.clone().unchecked_into::<web_sys::Element>());
            control_ref.set(resolved);
        });
    }

    // `getFormValue` (`:106-119`): the projected submission values — only the
    // selected, enabled, eligible inputs' registration values, in group-value
    // order; the unmounted `elementRef` short-circuit returns the raw value.
    let get_form_value: Rc<dyn Fn() -> Option<Value>> = {
        let value = value.clone();
        let element_ref = Rc::clone(&form_context.element_ref);
        let validation = field_context.validation.clone();
        Rc::new(move || {
            let current_value = value.get_untracked();
            let Some(form_element) = crate::field::validation::cell_peek(&element_ref) else {
                return Some(serde_json::to_value(current_value).unwrap_or(Value::Null));
            };

            // `successfulValues` (`:110-118`): registration values of inputs
            // that are checked and eligible.
            let mut successful_values = std::collections::HashSet::new();
            for (input, registration) in validation.registered_inputs.borrow().iter() {
                if let Some(registration_value) = &registration.value {
                    if input.checked()
                        && crate::field::validation::is_eligible_input(input, Some(&form_element))
                    {
                        successful_values.insert(registration_value.clone());
                    }
                }
            }

            Some(
                serde_json::to_value(
                    current_value
                        .into_iter()
                        .filter(|input_value| successful_values.contains(input_value))
                        .collect::<Vec<_>>(),
                )
                .unwrap_or(Value::Null),
            )
        })
    };

    // `useRegisterFieldControl(controlRef, id, value, getFormValue, !!fieldName
    // && !disabled, fieldName)` (`:117`).
    let field_name = leptos::prelude::GetUntracked::get_untracked(&field_context.name);
    let field_name_for_registration = field_name.clone();
    leptos_ui_internals::field_register_control::use_register_field_control(
        leptos_ui_internals::field_register_control::UseRegisterFieldControlParams {
            control_ref: Rc::clone(&control_ref),
            id: {
                let id = id.clone();
                reactive_graph::computed::Memo::new(move |_| Some(id.get()))
            },
            value: {
                let value = value.clone();
                reactive_graph::computed::Memo::new(move |_| {
                    Some(serde_json::to_value(value.get()).unwrap_or(Value::Null))
                })
            },
            get_form_value_override: Some(get_form_value),
            enabled: {
                let field_name = field_name_for_registration.clone();
                reactive_graph::computed::Memo::new(move |_| {
                    field_name.as_ref().is_some_and(|name| !name.is_empty()) && !disabled
                })
            },
            name: {
                let field_name = field_name_for_registration.clone();
                reactive_graph::computed::Memo::new(move |_| field_name.clone())
            },
        },
    );

    // `useIsoLayoutEffect(() => { setFilled(value.length > 0) }, [value,
    // setFilled])` (`:120-122`).
    {
        let set_filled = Rc::clone(&field_context.set_filled);
        let value = value.clone();
        leptos_ui_utils::use_iso_layout_effect::use_iso_layout_effect(move || {
            let current = value.get_untracked();
            let _ = current;
            set_filled(!current.is_empty());
        });
    }

    // `useValueChanged(value, …)` (`:124-141`): `clearErrors(fieldName)` when
    // named, `setDirty(!areArraysEqual(value, initialValue))`, and
    // `validation.change(value)`.
    {
        let clear_errors = Rc::clone(&form_context.clear_errors);
        let set_dirty = Rc::clone(&field_context.set_dirty);
        let validity_data = field_context.validity_data.clone();
        let validation = field_context.validation.clone();
        let name = field_name.clone();
        let value_for_changed = value.clone();
        use_value_changed(value.clone(), move |_previous_value: Vec<String>| {
            let current_value = value_for_changed.get_untracked();

            if let Some(field_name) = &name {
                if !field_name.is_empty() {
                    clear_errors(Some(field_name));
                }
            }

            // `initialValue` (`:129-133`): the array form of
            // `validityData.initialValue`, else `EMPTY_ARRAY`.
            let initial_value: Vec<String> = {
                let data = leptos::prelude::GetUntracked::get_untracked(&validity_data);
                match &data.initial_value {
                    Value::Array(items) => items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_string))
                        .collect(),
                    _ => Vec::new(),
                }
            };

            set_dirty(!are_arrays_equal(&current_value, &initial_value));

            (validation.change)(
                Some(serde_json::to_value(current_value).unwrap_or(Value::Null)),
                false,
            );
        });
    }

    // `const state = { ...fieldState, disabled }` (`:153-155`) — the record the
    // `fieldValidityMapping` walk consumes (`data-valid`/`data-invalid` from
    // `valid`, plus the generic truthiness hooks `data-disabled`,
    // `data-touched`, `data-dirty`, `data-filled`, `data-focused`).
    let state = field_context.state.snapshot();
    let state_map = {
        let mut map = serde_json::Map::new();
        map.insert("disabled".to_string(), serde_json::json!(disabled));
        map.insert("touched".to_string(), serde_json::json!(state.touched));
        map.insert("dirty".to_string(), serde_json::json!(state.dirty));
        map.insert(
            "valid".to_string(),
            match state.valid {
                Some(valid) => serde_json::json!(valid),
                None => Value::Null,
            },
        );
        map.insert("filled".to_string(), serde_json::json!(state.filled));
        map.insert("focused".to_string(), serde_json::json!(state.focused));
        map
    };

    // The context value (`:157-165`), provided by the caller through
    // [`provide_checkbox_group_context`] around the rendered element.
    let labelable = leptos_ui_internals::labelable_provider::use_labelable_context();
    let context_value = CheckboxGroupContextValue {
        value: value.clone(),
        set_value: Rc::clone(&set_value),
        all_values: all_values.clone(),
        parent: Some(parent),
        disabled,
        validation: field_context.validation.clone(),
        register_control_id: Rc::clone(&labelable.register_control_id),
    };
    provide_checkbox_group_context(context_value);

    // The base props bag (`:161-166`): `id: idProp`, `role: 'group'`, and
    // `aria-labelledby: labelId`.
    let label_id = labelable.label_id;
    let base_bag = {
        let mut attributes: Vec<(String, ElementAttributeFn)> = Vec::new();
        match &id_prop {
            Some(id_prop) => {
                let id_prop = id_prop.clone();
                attributes.push((
                    "id".to_string(),
                    Rc::new(move || Some(id_prop.clone())) as ElementAttributeFn,
                ));
            }
            None => {
                let generated_id = id.get_untracked();
                attributes.push((
                    "id".to_string(),
                    Rc::new(move || Some(generated_id.clone())) as ElementAttributeFn,
                ));
            }
        }
        attributes.push((
            "role".to_string(),
            Rc::new(|| Some("group".to_string())) as ElementAttributeFn,
        ));
        attributes.push((
            "aria-labelledby".to_string(),
            Rc::new(move || label_id.get_untracked()) as ElementAttributeFn,
        ));
        RenderElementProps {
            handlers: RenderElementHandlers {
                attributes,
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        }
    };

    // The `...elementProps` rest bag (`:164`).
    let element_bag = RenderElementProps {
        handlers: RenderElementHandlers {
            attributes: element_attributes
                .iter()
                .map(|(name, value)| {
                    let value = value.clone();
                    (
                        name.clone(),
                        Rc::new(move || Some(value.clone())) as ElementAttributeFn,
                    )
                })
                .collect(),
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    };

    // The third bag: `getDescriptionProps` (`:165`) — the labelable
    // `aria-describedby` merger.
    let mut description_attributes: Vec<(String, ElementAttributeFn)> = Vec::new();
    (labelable.get_description_props)(&mut description_attributes);
    let description_bag = RenderElementProps {
        handlers: RenderElementHandlers {
            attributes: description_attributes,
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    };

    // The props array `[baseProps, elementProps, getDescriptionProps]`
    // (`:161-167`), merged left-to-right.
    let props_bags = vec![
        PropsSource::Static(base_bag),
        PropsSource::Static(element_bag),
        PropsSource::Static(description_bag),
    ];

    // `useRenderElement('div', …, { state, ref: forwardedRef, props,
    // stateAttributesMapping: fieldValidityMapping })` (`:166-172`). The
    // standalone path always renders (`enabled` has no upstream gate on this
    // unit), so the `Option` unwrap mirrors a non-null render.
    use_render_element(
        "div",
        render_class_style,
        UseRenderElementParams {
            enabled: true,
            state: &state_map,
            refs: vec![],
            props: props_bags,
            state_attributes_mapping: Some(
                &field_validity_mapping
                    as &dyn Fn(
                        &str,
                        &Value,
                    ) -> Option<
                        Option<leptos_ui_internals::state_attributes::StateAttributeProps>,
                    >,
            ),
        },
    )
    .expect("the CheckboxGroup root always renders")
}
