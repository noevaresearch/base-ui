//! `RadioGroup` — the radio group unit (`specs/library/radio-group/behavior.md`,
//! `specs/library/radio-group/implementation.md`), ported from
//! `packages/react/src/radio-group/RadioGroup.tsx` (`:31-278`) plus
//! `RadioGroupContext.ts` (`:7-22`), `RadioGroupDataAttributes.ts` (`:4`) and
//! `index.ts` (`:1-3`).
//!
//! WHY THE GROUP LANDS BEFORE `library: radio`
//! ------------------------------------------
//! The group is the PROVIDER: `Radio.Root` consumes this context and its selection
//! semantics cannot be specified without it (`specs/library/radio-group/implementation.md`,
//! "Cross-component contract" — `radio/root/RadioRoot.tsx:24` is the sole consumer). The
//! oracle says the same thing independently: EVERY case in `RadioRoot.test.tsx` and
//! `RadioIndicator.test.tsx` wraps its items in `<RadioGroup>`, so `library: radio` is not
//! verifiable before this unit exists — the `library: checkbox-group` → `library: checkbox`
//! ordering this port repeats (TODO.md:555).
//!
//! WHAT THE GROUP IS
//! -----------------
//! `RadioGroup` renders one `<div role="radiogroup">` through the ported composite root
//! ([`composite_root`]) and owns every piece of state a radio needs: the controlled/
//! uncontrolled `value` duality, the single veto-able commit gate (`setCheckedValue`,
//! `:80-90`), the group-local "arrow keys armed an auto-select" flag (`touched`, `:78`),
//! the `inputRef` representative machinery (`:92-157`), the form-value projection
//! (`:159-172`), and the Field/Form/Labelable/Fieldset integration (`:52-70`, `:174-189`).
//! Its context value is `RadioGroupContext.ts:7-22` in the crate's spelling.
//!
//! RUST ADAPTATIONS (stated, not hidden)
//! -------------------------------------
//! - **One value type.** Upstream is generic over `Value` (`:31`); the port's is
//!   `Option<String>` (`RadioGroupValue`). The mined behavior spec only ever proves string
//!   values (`behavior.md` "State model": "Value is a single `string` … or `null` when
//!   nothing is selected"), and upstream's own generic coverage is type-only
//!   (`RadioGroup.spec.tsx:1-57`, implementation.md untested item 9).
//! - **`inputRef` is a callback only.** Upstream accepts an object ref OR a function ref
//!   (`:108-122`); Rust has no property-getter ref, so the port exposes the function form
//!   ([`RadioGroupInputRef`]) and its returned cleanup, which is the branch `:112-114`
//!   takes. A consumer that needs the *node* keeps it from the callback.
//! - **`onKeyDownCapture` (`:249-254`) rides the bag's `onKeyDown`.** The ported bag
//!   vocabulary (`use_render_element.rs:117-139`) carries no capture slot; the arm is
//!   attached as the FIRST bag entry of `defaultProps`, so it still runs before the
//!   composite root's own navigation handler for the same event. The deviation is
//!   recorded in `ralph/logs/spec-discrepancies.md` rather than silently accepted.
//! - **`controlRef` (`:92-100`)** is upstream a `useMemo`'d object whose `current` getter
//!   resolves live. The port has no property getter; it re-resolves in an
//!   `use_iso_layout_effect` keyed on the value signal and stores the result in the
//!   registration `Cell` — the `checkbox_group/mod.rs:616-627` precedent.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::prelude::*;
// `NodeRef`'s read in the view path is the leptos-side trait; the reactive handles this
// unit stores are the workspace `reactive_graph` ones the internals APIs speak, so the
// two are imported explicitly — this workspace carries two `reactive_graph` versions
// (`reactive_graph` 0.1.8 and 0.2.14 are both in `Cargo.lock`).
use leptos::prelude::Get as LeptosGet;
use leptos::prelude::GetUntracked as LeptosGetUntracked;
use reactive_graph::computed::Memo;
use reactive_graph::owner::LocalStorage;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use serde_json::{Map, Value, json};
use wasm_bindgen::JsCast;

use leptos_ui_internals::composite::{ModifierKey, SHIFT};
use leptos_ui_internals::composite_view::{CompositeRootComponentProps, composite_root};
use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::direction_context::{TextDirection, use_direction};
use leptos_ui_internals::field_register_control::{
    UseRegisterFieldControlParams, use_register_field_control,
};
use leptos_ui_internals::field_root_context::{FieldValidationBag, use_field_root_context};
use leptos_ui_internals::floating_ui::element_props::{ElementAttributeFn, ElementEventHandler};
use leptos_ui_internals::form_context::use_form_context;
use leptos_ui_internals::labelable_provider::use_labelable_context;
use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::state_attributes::{StateAttributeProps, field_validity_mapping};
use leptos_ui_internals::types::BaseUIEvent;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_composite_root::UseCompositeRootParams;
use leptos_ui_internals::use_render_element::{
    RenderElementHandlers, RenderElementProps, RenderedElement, UseRenderElementComponentProps,
};
use leptos_ui_internals::use_value_changed::use_value_changed;
use leptos_ui_utils::use_controlled::{SetValueAction, UseControlledProps, use_controlled};
use leptos_ui_utils::use_iso_layout_effect::use_iso_layout_effect;
use leptos_ui_utils::use_merged_refs::InputRef;

use crate::field::validation::{cell_peek, is_eligible_input};
use crate::fieldset::root::FieldsetRootContext;

// ---------------------------------------------------------------------------
// Vocabulary
// ---------------------------------------------------------------------------

/// The group's value: the selected radio's string value, or `None` for "nothing
/// selected" — upstream's `Value | null` (`RadioGroup.test.tsx:853-866`,
/// `:584-598`; `RadioGroup.tsx:72-77`).
pub type RadioGroupValue = Option<String>;

/// Upstream's `RadioGroup.ChangeEventDetails`
/// (`RadioGroup.tsx:343`, `BaseUIChangeEventDetails<typeof REASONS.none>`): the change
/// carries no custom properties and no per-reason event payload — the same shape
/// `CheckboxGroup` uses (`checkbox_group/mod.rs:101`).
pub type RadioGroupChangeEventDetails = BaseUIChangeEventDetails<(), ()>;

/// `onValueChange` (`RadioGroup.tsx:334`) — invoked before the state write, with the
/// details whose `cancel()` vetoes it (`:80-90`).
pub type OnRadioGroupValueChange =
    Rc<dyn Fn(RadioGroupValue, &RadioGroupChangeEventDetails)>;

/// Upstream's `inputRef` (`RadioGroup.tsx:338`), function-ref form, including the
/// cleanup a function ref may return (`:112-114`).
pub type RadioGroupInputRef =
    Rc<dyn Fn(Option<web_sys::HtmlInputElement>) -> Option<Rc<dyn Fn()>>>;

/// The composite list's metadata type. This unit's items are `Radio.Root`s, which
/// register themselves in the list (`radio/root/RadioRoot.tsx:251`'s `CompositeItem`);
/// the type is the seam `library: radio` will name when it calls
/// `use_composite_list_item::<RadioGroupItemMetadata, _>`, the
/// `toggle::ToggleItemMetadata` precedent (`toggle/mod.rs`).
#[derive(Clone, PartialEq)]
pub struct RadioGroupItemMetadata;

/// The reason string upstream's change details carry — `REASONS.none`
/// (`RadioGroup.tsx:341`: "The change-event reason type admits only `'none'`").
pub const REASON_NONE: &str = "none";

/// The root's role (`RadioGroup.tsx:231`) — overridable by a consumer's
/// `role="switch"` through the later element bag (`:22-27` of the behavior spec).
pub(crate) const ROLE_RADIOGROUP: &str = "radiogroup";

/// `enableHomeAndEndKeys={false}` (`RadioGroup.tsx:271`) — Home/End are deliberately
/// inert (implementation.md untested item 1). Named so the host suite can pin it.
pub(crate) const ENABLE_HOME_AND_END_KEYS: bool = false;

/// `MODIFIER_KEYS = [SHIFT]` (`RadioGroup.tsx:23`): Shift is exempted from the composite
/// modifier guard, which is behavior.md's "Modifier keys do not block navigation".
pub(crate) const MODIFIER_KEYS: [ModifierKey; 1] = [SHIFT];

/// The evaluated `RadioGroupState` members this unit contributes on top of the field
/// state (`RadioGroup.tsx:280-289`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RadioGroupState {
    /// `readOnly` (`:284-288`).
    pub read_only: bool,
    /// `required` (`:284-289`).
    pub required: bool,
}

/// The group's context value — `RadioGroupContext.ts:7-22` in the crate's spelling.
///
/// Consumers (`Radio.Root`) read the fields with their own per-field fallbacks
/// (`radio/root/RadioRoot.tsx:53-67`), which is why every member is a plain value or a
/// reactive handle rather than an `Option` wrapper.
#[derive(Clone)]
pub struct RadioGroupContextValue {
    /// `checkedValue` (`RadioGroupContext.ts:7`).
    pub checked_value: Signal<RadioGroupValue, LocalStorage>,
    /// `setCheckedValue` (`:7`) — the veto-able commit gate (`RadioGroup.tsx:80-90`).
    pub set_checked_value: OnRadioGroupValueChange,
    /// `disabled` (`:7`).
    pub disabled: bool,
    /// `readOnly`.
    pub read_only: bool,
    /// `required`.
    pub required: bool,
    /// `form` (`:8`) — the external form association for the hidden inputs.
    pub form: Option<String>,
    /// `name` (`:9`) — resolved through the Field (`RadioGroup.tsx:69`).
    pub name: Option<String>,
    /// `touched`/`setTouched` (`:12`) — the group-local auto-select ARM flag, not the
    /// Field's touched (`RadioGroup.tsx:78`, implementation.md `:47-56`).
    pub touched: Signal<bool, LocalStorage>,
    /// The write half of `touched`.
    pub set_touched: RwSignal<bool, LocalStorage>,
    /// `validation` (`:12`) — the surrounding Field's validation handle, shared so the
    /// children can register their hidden inputs and read their validation props.
    pub validation: FieldValidationBag,
    /// `registerInputRef` (`RadioGroupContext.ts:20-21`) — a child attaches the hidden
    /// input here; the group forwards the representative to the public `inputRef`.
    pub register_input_ref: Rc<dyn Fn(Option<web_sys::HtmlInputElement>) -> Option<Rc<dyn Fn()>>>,
}

/// The context type provided through the reactive owner — the `SendWrapper` bridge
/// `provide_context`'s `Send + Sync` contract requires for an `Rc`-heavy value
/// (`SharedCheckboxGroupContext`/`SharedFormContext` precedent).
pub type SharedRadioGroupContext = SendWrapper<RadioGroupContextValue>;

/// Upstream's `useRadioGroupContext` — `None` is upstream's default (`context ?? {}`,
/// `radio/root/RadioRoot.tsx:53`).
pub fn use_radio_group_context() -> Option<RadioGroupContextValue> {
    reactive_graph::owner::use_context::<SharedRadioGroupContext>().map(|shared| (*shared).clone())
}

/// Upstream's `RadioGroupContext.Provider` (`RadioGroup.tsx:257-274`).
///
/// Qualified deliberately (the `toggle_group.rs:500-512` note): this workspace carries two
/// `reactive_graph` versions, so the prelude's `provide_context` would write into a context
/// map the consumer's `reactive_graph::owner::use_context` never reads.
pub fn provide_radio_group_context(value: RadioGroupContextValue) {
    reactive_graph::owner::provide_context(SharedRadioGroupContext::new(value));
}

// ---------------------------------------------------------------------------
// Pure decision helpers — the half behavior.md's sections can be proven on the host
// ---------------------------------------------------------------------------

/// `confirmed = fieldDisabled || disabledProp` (`RadioGroup.tsx:68`): an ancestor
/// `Field.Root disabled` disables the group regardless of its own prop
/// (behavior.md:29, `RadioGroup.test.tsx:947-965`).
pub(crate) fn effective_disabled(field_disabled: bool, disabled_prop: bool) -> bool {
    field_disabled || disabled_prop
}

/// `name = fieldName ?? nameProp` (`RadioGroup.tsx:69`): the Field's name takes
/// precedence over the group's own (behavior.md:81, "Field name takes precedence",
/// `RadioGroup.test.tsx:929-944`).
pub(crate) fn effective_name(
    field_name: Option<String>,
    name_prop: Option<String>,
) -> Option<String> {
    field_name.or(name_prop)
}

/// `ariaLabelledby = labelId ?? fieldsetContext?.legendId` (`RadioGroup.tsx:191`):
/// `Field.Label` beats `Fieldset.Legend` by `??` short-circuit (behavior.md:71-72).
/// A user-supplied `aria-labelledby` beats both through the props merge, not here.
pub(crate) fn resolve_aria_labelledby(
    label_id: Option<String>,
    legend_id: Option<String>,
) -> Option<String> {
    label_id.or(legend_id)
}

/// What one run of the commit gate did — the observable shape of `setCheckedValue`
/// (`RadioGroup.tsx:80-90`) for the host suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ValueChangeFunnelOutcome {
    /// Whether the consumer's `onValueChange` was invoked (`:83`).
    pub callback_ran: bool,
    /// Whether the unwrapped setter ran (`:88`) — `false` when the change was vetoed.
    pub committed: bool,
}

/// `setCheckedValue` (`RadioGroup.tsx:80-90`): `onValueChange` FIRST, then the
/// `isCanceled` gate, then the unwrapped write. Every cancel semantic in behavior.md
/// ("State model" → Cancellation, "Events") funnels through this one check
/// (`RadioGroup.test.tsx:117-137`, `:139-164`, `:166-195`).
pub(crate) fn run_value_change_funnel<C, K>(
    next: RadioGroupValue,
    event_details: &RadioGroupChangeEventDetails,
    on_value_change: Option<&C>,
    commit_unwrapped: K,
) -> ValueChangeFunnelOutcome
where
    C: Fn(RadioGroupValue, &RadioGroupChangeEventDetails) + ?Sized,
    K: FnOnce(RadioGroupValue),
{
    if let Some(callback) = on_value_change {
        callback(next.clone(), event_details);
    }

    if event_details.is_canceled() {
        return ValueChangeFunnelOutcome {
            callback_ran: on_value_change.is_some(),
            committed: false,
        };
    }

    commit_unwrapped(next);
    ValueChangeFunnelOutcome {
        callback_ran: on_value_change.is_some(),
        committed: true,
    }
}

/// `getFormValue` (`RadioGroup.tsx:159-172`) as a pure projection: the registration
/// facts are the registry scan the component performs.
///
/// With no form element the logical `checkedValue ?? null` is returned unfiltered
/// (`:160-163`, implementation.md untested item 5); inside a Form the value projects only
/// if some registered input is BOTH checked and eligible (`:165-169`) — the single
/// mechanism behind the split behavior.md documents between a disabled checked radio, a
/// radio bound to another `form`, and a fully unmounted group (behavior.md:82-98).
pub(crate) fn project_form_value(
    checked_value: &RadioGroupValue,
    registered: &[(bool, bool)],
    has_form_element: bool,
) -> Value {
    let projected = || serde_json::to_value(checked_value).unwrap_or(Value::Null);

    if !has_form_element {
        return projected();
    }

    for (checked, eligible) in registered {
        if *checked && *eligible {
            return projected();
        }
    }

    Value::Null
}

/// `registerInputRef`'s re-point decision (`RadioGroup.tsx:133-137`): the public ref
/// moves to this input when it is checked, when no representative exists yet, or when
/// the current representative is disabled. That last arm is behavior.md:55's "Disabled
/// radios are skipped when assigning the ref"; `:125-127` skips null/disabled inputs
/// entirely (the guard is the caller's, [`register_input_ref_should_skip`]).
pub(crate) fn register_input_ref_should_repoint(
    input_checked: bool,
    has_current: bool,
    current_disabled: bool,
) -> bool {
    input_checked || !has_current || current_disabled
}

/// `if (!input || input.disabled) return undefined` (`RadioGroup.tsx:125-127`).
pub(crate) fn register_input_ref_should_skip(input_disabled: bool) -> bool {
    input_disabled
}

/// The imperative re-point after the value became `null` (`RadioGroup.tsx:184-188`):
/// "clearing to null keeps the ref on the first radio's input" (behavior.md:57,
/// `RadioGroup.test.tsx:468-497`) — but only while that fallback is itself enabled.
pub(crate) fn should_point_ref_at_fallback(
    checked_value_is_none: bool,
    fallback_present: bool,
    fallback_disabled: bool,
) -> bool {
    checked_value_is_none && fallback_present && !fallback_disabled
}

/// The state record the `fieldValidityMapping` walk consumes
/// (`RadioGroup.tsx:193-198`: `{ ...fieldState, disabled, required, readOnly }`).
///
/// The walk lowers each truthy key to `data-{key}` (`getStateAttributesProps.ts:24-28`),
/// which is where behavior.md:67's `data-disabled`/`data-readonly`/`data-required` come
/// from; `fieldValidityMapping` overrides `valid` into `data-valid`/`data-invalid`
/// (`field-constants/constants.ts:34-43`).
pub(crate) fn radio_group_state_map(
    field_state: &RadioGroupFieldStateSnapshot,
    disabled: bool,
    required: bool,
    read_only: bool,
) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("disabled".to_string(), json!(disabled));
    map.insert("touched".to_string(), json!(field_state.touched));
    map.insert("dirty".to_string(), json!(field_state.dirty));
    map.insert(
        "valid".to_string(),
        match field_state.valid {
            Some(valid) => json!(valid),
            None => Value::Null,
        },
    );
    map.insert("filled".to_string(), json!(field_state.filled));
    map.insert("focused".to_string(), json!(field_state.focused));
    map.insert("required".to_string(), json!(required));
    map.insert("readOnly".to_string(), json!(read_only));
    map
}

/// The field-state members the group's state record inherits (`RadioGroup.tsx:194`,
/// "`...fieldState`") — the `FieldRootState` projection this unit reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RadioGroupFieldStateSnapshot {
    /// `touched`.
    pub touched: bool,
    /// `dirty`.
    pub dirty: bool,
    /// `valid` — `None` while validation has not run.
    pub valid: Option<bool>,
    /// `filled`.
    pub filled: bool,
    /// `focused`.
    pub focused: bool,
}

/// `onKeyDownCapture` (`RadioGroup.tsx:249-254`): ANY `Arrow*` keydown arms the
/// group-local `touched` flag (and marks the field focused), which is half of
/// behavior.md:26's "arrow keys move focus AND select" — the child's `onFocus` consumes
/// the flag and clicks its hidden input (`radio/root/RadioRoot.tsx:153-161`).
///
/// Note the implementation.md observation (`:313-320`): this arm runs before the
/// composite modifier guard, so `Ctrl+Arrow` arms the flag without navigating. That is
/// preserved deliberately — it is upstream's behavior.
pub(crate) fn keydown_arms_touched(key: &str) -> bool {
    key.starts_with("Arrow")
}

/// The blur guard (`RadioGroup.tsx:239-248`): intra-group focus moves (radio → radio)
/// must never touch or validate. `contains(currentTarget, relatedTarget)` decides —
/// behavior.md:75's `onBlur` rule depends on it.
pub(crate) fn blur_leaves_the_group(related_target_inside: bool) -> bool {
    !related_target_inside
}

/// One `(attribute, value)` pair of the group's `defaultProps` bag
/// (`RadioGroup.tsx:229-255`). `String::is_empty` is React's `|| undefined` for the
/// `aria-*` members (`:232-234`).
pub(crate) fn aria_presence(value: bool) -> Option<String> {
    value.then(|| "true".to_string())
}

/// The `defaultProps` bag (`RadioGroup.tsx:229-255`): `id: idProp` (the raw prop — the
/// generated id feeds only field registration, implementation.md untested item 7),
/// `role: 'radiogroup'`, the three `aria-*` presence attributes, `aria-labelledby`, and
/// the three handlers.
fn default_props_bag(
    id_prop: Option<String>,
    required: bool,
    disabled: bool,
    read_only: bool,
    aria_labelledby: Option<String>,
    runtime: &RadioGroupRuntime,
) -> RenderElementProps {
    let mut attributes: Vec<(String, ElementAttributeFn)> = Vec::new();

    if let Some(id_prop) = id_prop {
        attributes.push((
            "id".to_string(),
            Rc::new(move || Some(id_prop.clone())) as ElementAttributeFn,
        ));
    }
    attributes.push((
        "role".to_string(),
        Rc::new(|| Some(ROLE_RADIOGROUP.to_string())) as ElementAttributeFn,
    ));
    attributes.push((
        "aria-required".to_string(),
        Rc::new(move || aria_presence(required)) as ElementAttributeFn,
    ));
    attributes.push((
        "aria-disabled".to_string(),
        Rc::new(move || aria_presence(disabled)) as ElementAttributeFn,
    ));
    attributes.push((
        "aria-readonly".to_string(),
        Rc::new(move || aria_presence(read_only)) as ElementAttributeFn,
    ));
    attributes.push((
        "aria-labelledby".to_string(),
        Rc::new(move || aria_labelledby.clone()) as ElementAttributeFn,
    ));

    RenderElementProps {
        handlers: RenderElementHandlers {
            attributes,
            on_focus: Some({
                let runtime = runtime.clone();
                Rc::new(move |_event: &BaseUIEvent<web_sys::FocusEvent>| {
                    runtime.set_field_focused(true);
                })
            }),
            on_blur: Some({
                let runtime = runtime.clone();
                Rc::new(move |event: &BaseUIEvent<web_sys::FocusEvent>| {
                    // `contains(event.currentTarget, event.relatedTarget)` (`:240`).
                    let inside = related_target_inside_group(event);
                    if blur_leaves_the_group(inside) {
                        runtime.set_field_touched(true);
                        runtime.set_field_focused(false);

                        if runtime.validation_mode_is_on_blur() {
                            runtime.commit_validation();
                        }
                    }
                })
            }),
            // `onKeyDownCapture` (`:249-254`) — see the module docs' deviation note.
            on_key_down: Some({
                let runtime = runtime.clone();
                Rc::new(move |event: &BaseUIEvent<web_sys::KeyboardEvent>| {
                    if keydown_arms_touched(&event.inner().key()) {
                        runtime.set_group_touched(true);
                        runtime.set_field_focused(true);
                    }
                })
            }),
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    }
}

/// `contains(event.currentTarget, event.relatedTarget)` (`RadioGroup.tsx:240`) — true
/// when focus moved to a node inside the group (the radio → radio case).
fn related_target_inside_group(event: &BaseUIEvent<web_sys::FocusEvent>) -> bool {
    // `currentTarget`/`relatedTarget` arrive as `EventTarget`; only a `Node` can be
    // asked about containment.
    let current = event
        .inner()
        .current_target()
        .and_then(|target| target.dyn_into::<web_sys::Node>().ok());
    let related = event
        .inner()
        .related_target()
        .and_then(|target| target.dyn_into::<web_sys::Node>().ok());
    match (current, related) {
        (Some(current), Some(related)) => current.contains(Some(&related)),
        // A blur with no related target (focus left the document) leaves the group.
        _ => false,
    }
}

/// The `...elementProps` rest bag (`RadioGroup.tsx:265`) — the consumer's plain
/// attributes, the second bag of the spread.
fn element_props_bag(attributes: &[(String, String)]) -> RenderElementProps {
    RenderElementProps {
        handlers: RenderElementHandlers {
            attributes: attributes
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
    }
}

/// The third bag of upstream's spread (`RadioGroup.tsx:267`):
/// `(props) => validation.getValidationProps(disabled ?? false, props)`. The ported
/// `get_validation_props` mutates a bag in place, so the getter replays the consumer's
/// merged attributes into it and folds the result back — the later-bag-wins order of
/// `mergeProps` is preserved by the engine.
fn validation_props_getter(
    validation: FieldValidationBag,
    disabled: bool,
) -> PropsSource {
    PropsSource::Getter(Rc::new(move |merged: &RenderElementProps| {
        let mut overlay: Vec<(String, ElementAttributeFn)> = merged
            .handlers
            .attributes
            .iter()
            .map(|(name, value)| (name.clone(), Rc::clone(value)))
            .collect();
        (validation.get_validation_props)(disabled, &mut overlay);

        let mut next = merged.clone();
        next.handlers.attributes = overlay;
        next
    }))
}

// ---------------------------------------------------------------------------
// The runtime — every piece of the group's state in one cloneable handle
// ---------------------------------------------------------------------------

/// The group's machinery, shared between the element description, the context and the
/// handlers. Mirrors the `ToggleGroupRuntime` shape (`toggle_group.rs:256`).
#[derive(Clone)]
struct RadioGroupRuntime {
    /// The resolved value (`:72-77`).
    value: Signal<RadioGroupValue, LocalStorage>,
    /// The unwrapped writer (`useControlled`'s setter, `:72`).
    set_value_unwrapped: Rc<dyn Fn(SetValueAction<RadioGroupValue>)>,
    /// The gated committer children call (`:80-90`).
    set_checked_value: OnRadioGroupValueChange,
    /// The group-local auto-select arm flag (`:78`).
    touched: RwSignal<bool, LocalStorage>,
    /// `getInputControl`-backed registration ref (`:92-100`).
    control_ref: Rc<Cell<Option<web_sys::Element>>>,
    /// The current public representative (`:101`). A `RefCell` rather than a `Cell`,
    /// because reading a `Cell` needs `Copy` and a DOM handle is not `Copy`.
    group_input_ref: Rc<RefCell<Option<web_sys::HtmlInputElement>>>,
    /// The first enabled registered input — the null-value fallback (`:102`).
    first_enabled_input_ref: Rc<RefCell<Option<web_sys::HtmlInputElement>>>,
    /// The consumer's `inputRef` (`:108-122`).
    input_ref: Option<RadioGroupInputRef>,
    /// The surrounding Field's handles (`:52-63`).
    field: leptos_ui_internals::field_root_context::FieldRootContextValue,
    /// `validationMode` (`:244`) — the group validates on blur only in that mode.
    validation_mode: leptos_ui_internals::form_context::FormValidationMode,
    /// `elementRef` (`RadioGroup.tsx:65`) — the `<form>` the Form context attaches; the
    /// `getFormValue` eligibility scan needs it (`:160-169`).
    form_element_ref: Rc<Cell<Option<web_sys::HtmlFormElement>>>,
    /// `clearErrors` (`:177`).
    clear_errors: leptos_ui_internals::form_context::ClearErrorsFn,
}

impl RadioGroupRuntime {
    /// `setFocused(true|false)` (`:237`, `:242`).
    fn set_field_focused(&self, focused: bool) {
        self.field.set_focused.set(focused);
    }

    /// `setFieldTouched(true)` (`:241`).
    fn set_field_touched(&self, touched: bool) {
        self.field.set_touched.set(touched);
    }

    /// `setTouched(true)` — the group-local arm flag (`:251`).
    fn set_group_touched(&self, touched: bool) {
        self.touched.set(touched);
    }

    /// `validationMode === 'onBlur'` (`:244`).
    fn validation_mode_is_on_blur(&self) -> bool {
        matches!(
            self.validation_mode,
            leptos_ui_internals::form_context::FormValidationMode::OnBlur
        )
    }

    /// `validation.commit(checkedValue)` (`:245`).
    fn commit_validation(&self) {
        let value = serde_json::to_value(self.value.get_untracked()).unwrap_or(Value::Null);
        (self.field.validation.commit)(value);
    }

    /// `setInputRef` (`:108-122`): forwards to the public ref and tracks the
    /// representative. Upstream's object-ref arm (`:115`) has no Rust analog; the
    /// callback form and its cleanup are the whole path here.
    fn set_input_ref(
        &self,
        hidden_input: Option<web_sys::HtmlInputElement>,
    ) -> Option<Rc<dyn Fn()>> {
        let cleanup = self
            .input_ref
            .as_ref()
            .and_then(|input_ref| input_ref(hidden_input.clone()));
        *self.group_input_ref.borrow_mut() = hidden_input;
        cleanup
    }

    /// `registerInputRef` (`:124-157`): children attach their hidden input here. The
    /// returned closure is the unmount cleanup, which re-checks live state rather than
    /// trusting captured state (`:139-141`).
    fn register_input_ref(
        &self,
        input: Option<web_sys::HtmlInputElement>,
    ) -> Option<Rc<dyn Fn()>> {
        let input = input?;

        if register_input_ref_should_skip(input.disabled()) {
            return None;
        }

        if self.first_enabled_input_ref.borrow().is_none() {
            *self.first_enabled_input_ref.borrow_mut() = Some(input.clone());
        }

        let current = self.group_input_ref.borrow().clone();
        let cleanup = if register_input_ref_should_repoint(
            input.checked(),
            current.is_some(),
            current
                .as_ref()
                .is_some_and(web_sys::HtmlInputElement::disabled),
        ) {
            self.set_input_ref(Some(input.clone()))
        } else {
            None
        };

        let this = self.clone();
        Some(Rc::new(move || {
            if same_input(this.first_enabled_input_ref.borrow().as_ref(), Some(&input)) {
                *this.first_enabled_input_ref.borrow_mut() = None;
            }

            if same_input(this.group_input_ref.borrow().as_ref(), Some(&input)) {
                if let Some(cleanup) = cleanup.clone() {
                    cleanup();
                    *this.group_input_ref.borrow_mut() = None;
                } else {
                    let _ = this.set_input_ref(None);
                }
            } else if let Some(cleanup) = cleanup.clone() {
                cleanup();
            }
        }))
    }

    /// `getFormValue` (`:159-172`) over the live registry.
    fn get_form_value(&self) -> Value {
        let form_element = cell_peek(&self.form_element_ref);
        let has_form_element = form_element.is_some();
        let registered: Vec<(bool, bool)> = {
            let registry = self.field.validation.registered_inputs.borrow();
            registry
                .iter()
                .map(|(input, _registration)| {
                    (input.checked(), is_eligible_input(input, form_element.as_ref()))
                })
                .collect()
        };

        project_form_value(&self.value.get_untracked(), &registered, has_form_element)
    }

    /// `useValueChanged(checkedValue, …)` (`:176-189`).
    fn value_changed(&self, previous: RadioGroupValue) {
        let _ = previous;
        let current = self.value.get_untracked();

        // `clearErrors(name)` (`:177`).
        if let Some(name) = self.field.name.get_untracked() {
            if !name.is_empty() {
                (self.clear_errors)(Some(name.as_str()));
            }
        }

        // `setDirty(checkedValue !== validityData.initialValue)` (`:179`).
        let initial_value = self.field.validity_data.get_untracked().initial_value;
        let current_json = serde_json::to_value(&current).unwrap_or(Value::Null);
        self.field.set_dirty.set(current_json != initial_value);

        // `setFilled(checkedValue != null)` (`:180`).
        self.field.set_filled.set(current.is_some());

        // `validation.change(checkedValue)` (`:182`).
        (self.field.validation.change)(Some(current_json), false);

        // `if (checkedValue == null && fallbackInput && !fallbackInput.disabled)
        //    setInputRef(fallbackInput)` (`:184-188`).
        let fallback = self.first_enabled_input_ref.borrow().clone();
        if should_point_ref_at_fallback(
            current.is_none(),
            fallback.is_some(),
            fallback
                .as_ref()
                .is_some_and(web_sys::HtmlInputElement::disabled),
        ) {
            let _ = self.set_input_ref(fallback);
        }
    }
}

/// `firstEnabledInputRef.current === input` / `groupInputRef.current === input`
/// (`RadioGroup.tsx:143,146`) — DOM node identity.
fn same_input(
    a: Option<&web_sys::HtmlInputElement>,
    b: Option<&web_sys::HtmlInputElement>,
) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => js_sys::Object::is(a.as_ref(), b.as_ref()),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Element path
// ---------------------------------------------------------------------------

/// The element-level props: upstream's `RadioGroupProps` (`RadioGroup.tsx:291-339`)
/// plus the container-level wiring the view layer also needs.
pub struct RadioGroupElementProps {
    /// `value` (`:320-324`) — the controlled value. `None` is upstream's `undefined`
    /// (uncontrolled); use [`RadioGroupElementProps::value_source`] when the page owns the
    /// state and the display must follow it, including the controlled-`null` case.
    pub value: Option<String>,
    /// The controlled value as a reactive read — a page's `useState` analog
    /// (`value={value}` + `onValueChange={setValue}`). The outer `Option` is the
    /// controlled/uncontrolled distinction and the inner one is upstream's `Value | null`,
    /// so `Signal::derive(move || Some(page_value.get()))` is a controlled group that can
    /// legitimately hold `null` (`RadioGroup.test.tsx:468-497`).
    pub value_source: Option<Signal<Option<Option<String>>, LocalStorage>>,
    /// `defaultValue` (`:325-330`) — the uncontrolled seed.
    pub default_value: Option<String>,
    /// `onValueChange` (`:331-334`).
    pub on_value_change: Option<OnRadioGroupValueChange>,
    /// `disabled` (`:295-299`, `@default false`).
    pub disabled: bool,
    /// `readOnly` (`:300-304`, `@default false`).
    pub read_only: bool,
    /// `required` (`:305-309`, `@default false`).
    pub required: bool,
    /// `name` (`:310-313`).
    pub name: Option<String>,
    /// `form` (`:314-318`).
    pub form: Option<String>,
    /// `inputRef` (`:335-338`).
    pub input_ref: Option<RadioGroupInputRef>,
    /// `id` (`:322`, through `BaseUIComponentProps`).
    pub id: Option<String>,
    /// `className`/`style`/`render` (`:35-37`).
    pub render_class_style: UseRenderElementComponentProps,
    /// The `...elementProps` rest (`:49`).
    pub element_attributes: Vec<(String, String)>,
    /// The forwarded root ref (`refs={[forwardedRef]}`, `:269`).
    pub root_ref: Option<InputRef<web_sys::Element>>,
}

impl Default for RadioGroupElementProps {
    fn default() -> Self {
        Self {
            value: None,
            value_source: None,
            default_value: None,
            on_value_change: None,
            disabled: false,
            read_only: false,
            required: false,
            name: None,
            form: None,
            input_ref: None,
            id: None,
            render_class_style: UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            root_ref: None,
        }
    }
}

/// Builds the group's element description — upstream's `RadioGroup` body
/// (`RadioGroup.tsx:31-278`) up to `useRenderElement`, and provisions the group context
/// for the subtree (`:257-274`). Must be called inside a reactive owner.
pub fn radio_group_element(props: RadioGroupElementProps) -> RenderedElement {
    let RadioGroupElementProps {
        value: external_value,
        value_source: external_source,
        default_value,
        on_value_change,
        disabled: disabled_prop,
        read_only,
        required,
        name: name_prop,
        form,
        input_ref,
        id: id_prop,
        render_class_style,
        element_attributes,
        root_ref,
    } = props;

    // The Field/Form/Labelable/Fieldset reads (`:52-66`).
    let field = use_field_root_context();
    let form_context = use_form_context();
    let labelable = use_labelable_context();
    // `useFieldsetRootContext(true)` (`:66`) — the OPTIONAL read (`:252-255`).
    let fieldset_legend_id = use_context::<FieldsetRootContext>()
        .and_then(|context| LeptosGetUntracked::get_untracked(&context.legend_id));

    // `const disabled = fieldDisabled || disabledProp` (`:68`).
    let disabled = effective_disabled(field.disabled.get_untracked().unwrap_or(false), disabled_prop);
    // `const name = fieldName ?? nameProp` (`:69`).
    let name = effective_name(field.name.get_untracked(), name_prop.clone());
    // `const id = useBaseUiId(idProp)` (`:70`).
    let id = use_base_ui_id(RwSignal::new(id_prop.clone()));

    // The value duality (`:72-77`). `useControlled`'s `controlled` is upstream's
    // `externalValue` — `undefined` (uncontrolled) vs a value that may itself be `null`
    // (controlled-and-empty), so the port's encoding is the double `Option`: the OUTER
    // `None` is "no controlled prop", the inner one is the value. `use_controlled`
    // (`use_controlled.rs:144-151`) reads exactly that pair (`C: Get<Value = Option<T>>`,
    // `D: Get<Value = T>`) with `T = Option<String>`.
    let controlled_source: Signal<Option<Option<String>>, LocalStorage> = match external_source {
        Some(source) => source,
        None => match external_value {
            Some(value) => Signal::derive_local(move || Some(Some(value.clone()))),
            None => Signal::derive_local(|| None::<Option<String>>),
        },
    };
    let (value, set_value_unwrapped) = use_controlled(UseControlledProps::new(
        controlled_source,
        RwSignal::new(default_value),
        "RadioGroup",
    ));
    let set_value_unwrapped: Rc<dyn Fn(SetValueAction<RadioGroupValue>)> =
        Rc::new(move |action| set_value_unwrapped(action));

    // `const [touched, setTouched] = React.useState(false)` (`:78`).
    let touched: RwSignal<bool, LocalStorage> = RwSignal::new_local(false);

    // The registration ref (`:92-104`), re-resolved in a layout effect (module docs).
    let control_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    {
        let control_ref = Rc::clone(&control_ref);
        let get_input_control = Rc::clone(&field.validation.get_input_control);
        let value_for_ref = value;
        use_iso_layout_effect(move || {
            let _ = value_for_ref.get_untracked();
            let resolved = get_input_control().map(|element| element.unchecked_into());
            control_ref.set(resolved);
        });
    }

    let mut runtime = RadioGroupRuntime {
        value,
        set_value_unwrapped: Rc::clone(&set_value_unwrapped),
        set_checked_value: Rc::new(|_, _| {}),
        touched,
        control_ref: Rc::clone(&control_ref),
        group_input_ref: Rc::new(RefCell::new(None)),
        first_enabled_input_ref: Rc::new(RefCell::new(None)),
        input_ref,
        field: field.clone(),
        validation_mode: field.validation_mode,
        form_element_ref: Rc::clone(&form_context.element_ref),
        clear_errors: Rc::clone(&form_context.clear_errors),
    };

    // `setCheckedValue` (`:80-90`) — the gated committer, shared through the context.
    let set_checked_value: OnRadioGroupValueChange = {
        let runtime = runtime.clone();
        let on_value_change = on_value_change.clone();
        Rc::new(
            move |next: RadioGroupValue, event_details: &RadioGroupChangeEventDetails| {
                let unwrapped = Rc::clone(&runtime.set_value_unwrapped);
                let _ = run_value_change_funnel(
                    next,
                    event_details,
                    on_value_change.as_deref(),
                    move |committed| unwrapped(SetValueAction::Value(committed)),
                );
            },
        )
    };
    runtime.set_checked_value = Rc::clone(&set_checked_value);

    // `useRegisterFieldControl(controlRef, id, checkedValue ?? null, getFormValue,
    // !disabled, nameProp)` (`:174`) — note the RAW `nameProp`, not the Field-resolved
    // name (implementation.md untested item 6).
    let name_prop_for_registration = name_prop.clone();
    let runtime_for_form_value = runtime.clone();
    let runtime_for_value = runtime.clone();
    use_register_field_control(UseRegisterFieldControlParams {
        control_ref: Rc::clone(&control_ref),
        id: {
            let id = id;
            Memo::new(move |_| Some(id.get()))
        },
        value: {
            let value = runtime.value;
            Memo::new(move |_| Some(serde_json::to_value(value.get()).unwrap_or(Value::Null)))
        },
        get_form_value_override: Some(Rc::new(move || Some(runtime_for_form_value.get_form_value()))),
        enabled: {
            let disabled = disabled;
            Memo::new(move |_| !disabled)
        },
        name: Memo::new(move |_| name_prop_for_registration.clone()),
    });

    // `useValueChanged(checkedValue, …)` (`:176-189`).
    {
        let runtime = runtime_for_value;
        use_value_changed(runtime.value, move |previous: RadioGroupValue| {
            runtime.value_changed(previous);
        });
    }

    // `const state = { ...fieldState, disabled, required, readOnly }` (`:193-198`).
    let field_state = field.state.get_untracked();
    let state_map = radio_group_state_map(
        &RadioGroupFieldStateSnapshot {
            touched: field_state.touched,
            dirty: field_state.dirty,
            valid: field_state.valid,
            filled: field_state.filled,
            focused: field_state.focused,
        },
        disabled,
        required,
        read_only,
    );

    // The provided context (`:200-227`, `:257-274`).
    let context_value = RadioGroupContextValue {
        checked_value: runtime.value,
        set_checked_value: Rc::clone(&set_checked_value),
        disabled,
        read_only,
        required,
        form,
        name,
        touched: runtime.touched.read_only().into(),
        set_touched: runtime.touched,
        validation: field.validation.clone(),
        register_input_ref: {
            let runtime = runtime.clone();
            Rc::new(move |input| runtime.register_input_ref(input))
        },
    };
    provide_radio_group_context(context_value);

    let aria_labelledby = resolve_aria_labelledby(labelable.label_id.get_untracked(), fieldset_legend_id);
    let default_props = default_props_bag(
        id_prop,
        required,
        disabled,
        read_only,
        aria_labelledby,
        &runtime,
    );

    let direction = use_direction();
    let refs: Vec<InputRef<web_sys::Element>> = root_ref.into_iter().collect();

    composite_root::<RadioGroupItemMetadata, Memo<Option<i32>>, Memo<TextDirection>>(
        CompositeRootComponentProps {
            render_class_style,
            tag: "div".to_string(),
            state: state_map,
            // `stateAttributesMapping={fieldValidityMapping}` (`:270`).
            state_attributes_mapping: Some(Rc::new(
                |key: &str, value: &Value| -> Option<Option<StateAttributeProps>> {
                    field_validity_mapping(key, value)
                },
            )),
            refs,
            // `props={[defaultProps, elementProps, (props) =>
            //   validation.getValidationProps(disabled ?? false, props)]}` (`:264-268`).
            props: vec![
                PropsSource::Static(default_props),
                PropsSource::Static(element_props_bag(&element_attributes)),
                validation_props_getter(field.validation.clone(), disabled),
            ],
            element_props: RenderElementProps::default(),
            on_map_change: None,
            highlight_item_on_hover: false,
        },
        UseCompositeRootParams {
            orientation: None,
            grid: None,
            // Upstream's `CompositeRoot` default (`:87`).
            loop_focus: true,
            on_loop: None,
            highlighted_index: None,
            on_highlighted_index_change: None,
            direction,
            root_ref: InputRef::Empty,
            // `enableHomeAndEndKeys={false}` (`:271`) — Home/End are deliberately inert
            // (implementation.md untested item 1).
            enable_home_and_end_keys: ENABLE_HOME_AND_END_KEYS,
            stop_event_propagation: true,
            disabled_indices: None,
            // `modifierKeys={MODIFIER_KEYS}` where `MODIFIER_KEYS = [SHIFT]` (`:23,:272`).
            modifier_keys: MODIFIER_KEYS.to_vec(),
        },
    )
    .expect("the composite root always renders while enabled")
}

// ---------------------------------------------------------------------------
// View path
// ---------------------------------------------------------------------------

/// The view-layer props: the element props plus the consumer's subtree
/// (`<RadioGroup>{children}</RadioGroup>`).
pub struct RadioGroupViewProps {
    /// The element-level props.
    pub element: RadioGroupElementProps,
    /// The radios the group's context is provided to.
    pub children: Option<ChildrenFn>,
}

impl Default for RadioGroupViewProps {
    fn default() -> Self {
        Self {
            element: RadioGroupElementProps::default(),
            children: None,
        }
    }
}

/// Adapts a bag handler back to the native event a `view!` listener receives — the
/// inverse of `native_to_base_ui` (`toggle_group.rs:532-536`).
fn base_ui_to_native<E: Clone + 'static>(
    handler: ElementEventHandler<BaseUIEvent<E>>,
) -> impl Fn(&E) {
    move |event: &E| handler(&BaseUIEvent::new(event.clone()))
}

/// The group root view — the context provider wrapping the composite root element with
/// the consumer's subtree inside it (`RadioGroup.tsx:257-274`).
///
/// Must be called inside a reactive owner. The subtree is built inside a bridge owner so
/// the provided `RadioGroupContextValue` is visible to every `Radio.Root` built in it —
/// the `checkbox_group/view.rs` mechanism.
pub fn radio_group_view(props: RadioGroupViewProps) -> impl IntoView {
    let RadioGroupViewProps { element, children } = props;

    let bridge_owner = reactive_graph::owner::Owner::new();
    let view = bridge_owner.with(move || {
        let rendered = radio_group_element(element);

        let class = rendered.props.class.clone();
        let style = {
            let declarations = &rendered.props.style;
            (!declarations.is_empty()).then(|| {
                declarations
                    .iter()
                    .map(|(property, value)| format!("{property}: {value};"))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
        };
        let resolved_attributes: Vec<(String, Option<String>)> = {
            let mut resolved: Vec<(String, Option<String>)> = Vec::new();
            if let Some(class) = class {
                resolved.push(("class".to_string(), Some(class)));
            }
            if let Some(style) = style {
                resolved.push(("style".to_string(), Some(style)));
            }
            resolved.extend(
                rendered
                    .props
                    .handlers
                    .attributes
                    .iter()
                    .map(|(name, value)| (name.clone(), value())),
            );
            resolved
        };
        let on_blur = rendered.props.handlers.on_blur.clone().map(base_ui_to_native);
        let on_focus = rendered.props.handlers.on_focus.clone().map(base_ui_to_native);
        let on_key_down = rendered
            .props
            .handlers
            .on_key_down
            .clone()
            .map(base_ui_to_native);
        let ref_callback = rendered.props.ref_callback.clone();

        let root_node: NodeRef<leptos::html::Div> = NodeRef::new();

        // The attribute writer — the element bag on a real node, with removals so a
        // stale `data-*` never misreports state (the toggle-group/checkbox-group
        // convention).
        {
            let resolved_attributes = resolved_attributes.clone();
            Effect::new(move |_| {
                let Some(node) = root_node.get() else {
                    return;
                };
                let node: &web_sys::Element = node.as_ref();
                for (name, value) in &resolved_attributes {
                    match value {
                        Some(value) => {
                            let _ = node.set_attribute(name, value);
                        }
                        None => {
                            let _ = node.remove_attribute(name);
                        }
                    }
                }
            });
        }

        if let Some(ref_callback) = ref_callback {
            Effect::new(move |_| {
                if let Some(node) = root_node.get() {
                    let node: &web_sys::Element = node.as_ref();
                    ref_callback(Some(node));
                }
            });
        }

        // The subtree crosses a `Send + Sync` boundary because a `view!` child closure is
        // stored that way; `SendWrapper` is the crate's bridge for local handles.
        let children = SendWrapper::new(children);
        let children_view = move || children.as_ref().map(|children| children());

        view! {
            <div
                node_ref=root_node
                on:keydown=move |event: web_sys::KeyboardEvent| {
                    if let Some(handler) = &on_key_down {
                        handler(&event);
                    }
                }
                on:focusin=move |event: web_sys::FocusEvent| {
                    if let Some(handler) = &on_focus {
                        handler(&event);
                    }
                }
                on:focusout=move |event: web_sys::FocusEvent| {
                    if let Some(handler) = &on_blur {
                        handler(&event);
                    }
                }
            >
                {children_view}
            </div>
        }
    });
    // The bridge owner outlives the subtree (the `field_root_view`/`toggle_group_view`
    // precedent).
    std::mem::forget(bridge_owner);
    view
}

/// The `RadioGroup` component — upstream's exported component (`RadioGroup.tsx:31`) in
/// the port's `view!`-usable form.
///
/// `render` is deliberately NOT exposed: the element form substitutes the tag
/// (`useRenderElement.tsx:164-196`) and this unit's view path builds a fixed `<div>`, so a
/// component-level `render` would promise a substitution the view cannot keep — the
/// crate-wide `library: the view paths drop render's element form` item.
#[leptos::component]
pub fn RadioGroup(
    /// `value` (`:320-324`) — controlled.
    #[prop(default = None, optional)]
    value: Option<String>,
    /// `defaultValue` (`:325-330`) — the uncontrolled seed.
    #[prop(default = None, optional)]
    default_value: Option<String>,
    /// `onValueChange` (`:331-334`).
    #[prop(default = None, optional)]
    on_value_change: Option<OnRadioGroupValueChange>,
    /// `disabled` (`:295-299`, `@default false`).
    #[prop(default = false, optional)]
    disabled: bool,
    /// `readOnly` (`:300-304`, `@default false`).
    #[prop(default = false, optional)]
    read_only: bool,
    /// `required` (`:305-309`, `@default false`).
    #[prop(default = false, optional)]
    required: bool,
    /// `name` (`:310-313`).
    #[prop(default = None, optional)]
    name: Option<String>,
    /// `form` (`:314-318`).
    #[prop(default = None, optional)]
    form: Option<String>,
    /// `inputRef` (`:335-338`).
    #[prop(default = None, optional)]
    input_ref: Option<RadioGroupInputRef>,
    /// `id` (`:322`).
    #[prop(default = None, optional)]
    id: Option<String>,
    /// `className` (`:35`).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style` (`:37`) — upstream's style record as ordered `(property, value)`
    /// declarations (the avatar/otp-field spelling).
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// The consumer's `...elementProps` attributes (`:49`).
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The radios.
    children: ChildrenFn,
) -> impl IntoView {
    radio_group_view(RadioGroupViewProps {
        element: RadioGroupElementProps {
            value,
            value_source: None,
            default_value,
            on_value_change,
            disabled,
            read_only,
            required,
            name,
            form,
            input_ref,
            id,
            render_class_style: crate::toggle_group::class_style_bag(class, style),
            element_attributes,
            root_ref: None,
        },
        children: Some(children),
    })
}
