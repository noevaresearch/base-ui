//! `Switch.Root` — port of `packages/react/src/switch/root/SwitchRoot.tsx`
//! (the `library: switch` TODO item; `specs/library/switch/behavior.md`,
//! `specs/library/switch/implementation.md`).
//!
//! ## What upstream does
//!
//! Root owns one controlled boolean (`useControlled`, `SwitchRoot.tsx:86-91`) and the
//! Field lifecycle state inherited through `FieldRootContext` (`:59-70`). Every
//! interaction funnels through the single native `change` event of the hidden
//! `<input type="checkbox">` (`:172-193`): the visible element re-dispatches its own
//! click onto that input instead of mutating state (`:143-156`). The element is a
//! `span` by default and a real `<button>` under `nativeButton` (`:218`), and the
//! change veto protocol runs `onCheckedChange` and then checks
//! `eventDetails.isCanceled` (`:186-190`).
//!
//! ## The port's split
//!
//! - The pure folds, the state record and the state→attribute walk live in
//!   [`crate::switch::state`] (host-testable, citing the same upstream lines).
//! - This module is the DOM/event boundary: the three siblings upstream returns inside
//!   the provider (`:230-238`), the context provider (`:231`), the Field/Form/Labelable
//!   integration, and the listeners.
//!
//! Runtime law (the crate's dual-runtime split): leptos 0.7 tracks reactive-graph 0.1
//! while the internals crate's hooks are typed over 0.2, so the state machine is
//! re-homed on leptos signals (the checkbox/`Field.Control` precedent) and the rg-0.2
//! handles the internals hand back (`useLabelableId`, `useAriaLabelledBy`, `useButton`)
//! are bridged with [`mirror_rg_to_leptos`] / a written rg-0.2 mirror.
//!
//! ## Documented adaptations (never silent)
//!
//! 1. **`view!` has no attribute spread.** The consumer's `...elementProps` rest and the
//!    Field validation overlay (`validation.getValidationProps`, `:225`) are plain
//!    dynamic maps, so they are applied by one writer effect on the control node with
//!    stale-name pruning (the checkbox writer precedent, `checkbox/root.rs`).
//! 2. **The `checked` *property*** is re-asserted after every change-funnel run.
//!    React's controlled-input commit for `checked` (`:161`, `:204`) is what keeps a
//!    cancelled or `readOnly` change from leaving the browser-toggled property out of
//!    sync with the state; leptos' `prop:checked` binding only re-runs when the state
//!    signal changes, which a cancelled change never does. Measured by the
//!    cancelled-change assertions (`SwitchRoot.test.tsx:203-243`).
//! 3. **`suppressHydrationWarning`** (`:236`) has no Leptos analog (CSR-only app).
//! 4. **`render`** is honored for its element form's tag (`<button />`, `:1290-1309`) —
//!    the description-layer render *function* is the crate-wide gap already recorded as
//!    `library: the view paths drop render's element form`.
//! 5. **The hidden input's `validation.inputRef` merge** (`:77`) is represented by the
//!    validation overlay (adaptation 1) plus the native `required` attribute; the
//!    native-validity registration slot is the recorded `Field.Control` ref gap
//!    (`library: Field.Control's port surface carries no ref slot`).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::either::Either;
use leptos::prelude::*;
use serde_json::Value;
use wasm_bindgen::JsCast;

use leptos_ui_internals::floating_ui::element_props::ElementEventHandler;
use leptos_ui_internals::labelable_provider::{
    UseLabelableIdParams, use_aria_labelled_by, use_labelable_context, use_labelable_id,
};
use leptos_ui_internals::types::BaseUIEvent;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_button::{ButtonExternalHandlers, UseButtonParams, use_button};
use leptos_ui_internals::use_render_element::RenderProp;

use crate::field::context::use_field_root_context;
use crate::field::validation_helpers::mirror_rg_to_leptos;
use crate::switch::context::{SwitchRootContextValue, provide_switch_root_context};
use crate::switch::state::{
    ChangeFunnel, DATA_DIRTY, DATA_DISABLED, DATA_FILLED, DATA_FOCUSED, DATA_INVALID, DATA_READONLY,
    DATA_REQUIRED, DATA_TOUCHED, DATA_UNCHECKED, DATA_VALID, DATA_CHECKED, SwitchChangeEventDetails,
    SwitchRootState, aria_bool_attr, checked_is_dirty, effective_disabled, effective_name,
    hidden_input_id, input_style, root_id, run_change_funnel, switch_state_attributes,
};

/// The Root props — upstream's `SwitchRoot.Props` (`SwitchRoot.tsx:37-56`, `:260-324`)
/// with its documented defaults.
#[derive(Clone, Default)]
pub struct SwitchRootViewProps {
    /// `checked` (`:273`) — the controlled value; `None` is upstream's `undefined`.
    pub checked: Option<bool>,
    /// `defaultChecked` (`:280`, default `false`).
    pub default_checked: bool,
    /// `disabled` (`:285`, default `false`).
    pub disabled: bool,
    /// `readOnly` (`:308`, default `false`).
    pub read_only: bool,
    /// `required` (`:313`, default `false`).
    pub required: bool,
    /// `name` (`:293`).
    pub name: Option<String>,
    /// `form` (`:298`) — the form that owns the hidden input.
    pub form: Option<String>,
    /// `id` (`:267`) — the hidden input's id, or the root's under `nativeButton`.
    pub id: Option<String>,
    /// `value` (`:318`) — the value submitted while checked.
    pub value: Option<String>,
    /// `uncheckedValue` (`:323`) — the value submitted while unchecked.
    pub unchecked_value: Option<String>,
    /// `nativeButton` (`:46`, default `false`) — render a real `<button>`.
    pub native_button: bool,
    /// `'aria-labelledby'` (`:41`).
    pub aria_labelledby: Option<String>,
    /// `inputRef` (`:44`) — direct access to the hidden input.
    pub input_ref: Option<Rc<dyn Fn(Option<web_sys::HtmlInputElement>)>>,
    /// `onCheckedChange` (`:47`).
    pub on_checked_change: Option<Rc<dyn Fn(bool, &SwitchChangeEventDetails)>>,
    /// The consumer's `onClick` (`:55`, the `...elementProps` rest's click member).
    pub on_click: Option<ElementEventHandler<web_sys::MouseEvent>>,
    /// `render` (`:51`).
    pub render: Option<RenderProp>,
    /// `className` (`:38`).
    pub class: Option<String>,
    /// `style` (`:54`).
    pub style: Option<String>,
    /// The remaining `...elementProps` rest (`:55`) as plain attributes.
    pub element_attributes: Vec<(String, String)>,
}

/// The `Switch.Root` component (`SwitchRoot.tsx:33-239`).
#[component]
pub fn Root(
    /// The Root's props; `#[prop(optional)]` because every member has an upstream default.
    #[prop(optional)]
    view_props: SwitchRootViewProps,
    children: ChildrenFn,
) -> impl IntoView {
    switch_root_view(view_props, children)
}

/// The view seam (the `checkbox_root_view` convention). Establishes its own rg-0.2
/// bridge owner so an enclosing `Field.Root`/`Form`'s rg-0.2 contexts stay visible, and
/// returns the three-sibling fragment upstream returns inside the provider.
pub fn switch_root_view(props: SwitchRootViewProps, children: ChildrenFn) -> impl IntoView {
    let bridge_owner = reactive_graph::owner::Owner::new();
    let view = bridge_owner.with(move || switch_root_body(props, children));
    // The bridge owner outlives the subtree (the checkbox/direction-provider precedent).
    std::mem::forget(bridge_owner);
    view
}

/// The Root body — must be called inside a reactive owner (`use_button`'s
/// update-disabled effect registers there).
fn switch_root_body(props: SwitchRootViewProps, children: ChildrenFn) -> impl IntoView {
    let SwitchRootViewProps {
        checked: checked_prop,
        default_checked,
        disabled: disabled_prop,
        read_only,
        required,
        name: name_prop,
        form,
        id: id_prop,
        value: value_prop,
        unchecked_value,
        native_button,
        aria_labelledby,
        input_ref,
        on_checked_change,
        on_click: consumer_on_click,
        render,
        class,
        style,
        element_attributes,
    } = props;

    // `useFormContext()` / `useFieldRootContext()` / `useLabelableContext()` (`:58-71`) —
    // every read falls back to its inert shell outside the matching provider.
    let form_context = leptos_ui_internals::form_context::use_form_context();
    let field = use_field_root_context();
    let labelable = use_labelable_context();

    // `disabled = fieldDisabled || disabledProp`, `name = fieldName ?? nameProp`
    // (`:73-74`).
    let field_disabled = field.disabled.clone();
    let disabled: Signal<bool> =
        Signal::derive(move || effective_disabled(field_disabled.get(), disabled_prop));
    let field_name = field.name.clone();
    let name: Signal<Option<String>> = {
        let name_prop = name_prop.clone();
        Signal::derive(move || effective_name(field_name.get(), name_prop.clone()))
    };

    // The element slots the machines read (`:77-79`): leptos has no ref fork, so the nodes
    // arrive through `NodeRef`s and a mount effect resyncs the `Rc<Cell>` slots (the
    // `Field.Control` precedent).
    let input_element_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    let switch_element_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    let input_node: NodeRef<leptos::html::Input> = NodeRef::new();
    let control_span_node: NodeRef<leptos::html::Span> = NodeRef::new();
    let control_button_node: NodeRef<leptos::html::Button> = NodeRef::new();

    // `const id = useBaseUiId()` (`:81`) — no override: the generated instance id. The
    // labelable id (which *can* come from the `id` prop) is `controlId` below. The hook is
    // typed over reactive-graph 0.2, so the (constant) override rides an rg-0.2 signal.
    let generated_id =
        use_base_ui_id(reactive_graph::signal::RwSignal::new(None::<String>));
    let generated_id_leptos = mirror_rg_to_leptos(&generated_id);

    // `const controlId = useLabelableId({ id: idProp })` (`:83`).
    let control_id_rg = use_labelable_id(UseLabelableIdParams {
        id: id_prop.filter(|id| !id.is_empty()),
        enabled: true,
    });
    let control_id = mirror_rg_to_leptos(&control_id_rg);
    let control_id_for_labels = control_id.get_untracked();

    // `hiddenInputId = nativeButton ? undefined : controlId` (`:84`).
    let hidden_input_id_value = hidden_input_id(native_button, &control_id_for_labels);

    // `useControlled({ controlled: checkedProp, default: Boolean(defaultChecked) })`
    // (`:86-91`): the port re-homes it on a leptos signal, with the controlled prop
    // winning whenever it is defined (the `controlled_checked` re-derivation precedent,
    // `checkbox/state.rs`).
    let uncontrolled_checked: RwSignal<bool> = RwSignal::new(default_checked);
    let checked: Signal<bool> = Signal::derive(move || {
        checked_prop.unwrap_or_else(|| uncontrolled_checked.get())
    });
    let set_checked = move |next: bool| uncontrolled_checked.set(next);

    // `useRegisterFieldControl(switchRef, id, checked, undefined, !disabled, nameProp)`
    // (`:93`) — re-homed on the leptos field context (one source token per instance,
    // in-place re-registration, unregistration at unmount: the `Field.Control`
    // registration mechanics the checkbox port reuses).
    {
        let register = field.register_field_control.clone();
        let source = leptos_ui_internals::labelable_provider::ControlIdSource::new();
        let control_ref = Rc::clone(&switch_element_ref);
        let id_for_registration = generated_id_leptos.clone();
        let name_prop = name_prop.clone();
        let checked_for_registration = checked;
        Effect::new(move |_| {
            if disabled.get() {
                register(source.clone(), None);
                return;
            }
            let registration =
                leptos_ui_internals::field_register_control::FieldControlRegistration {
                    control_ref: Rc::clone(&control_ref),
                    id: Some(id_for_registration.get()),
                    name: name_prop.clone(),
                    get_value: None,
                    value: Some(Value::Bool(checked_for_registration.get())),
                };
            register(source.clone(), Some(registration));
        });
        let register = field.register_field_control.clone();
        let source_for_cleanup = source;
        let cleanup = send_wrapper::SendWrapper::new(move || {
            register(source_for_cleanup, None);
        });
        reactive_graph::owner::on_cleanup(move || (*cleanup)());
    }

    // `useIsoLayoutEffect(() => setFilled(checked), [checked, setFilled])` (`:95-97`).
    {
        let set_filled = field.set_filled.clone();
        Effect::new(move |_| {
            set_filled(checked.get());
        });
    }

    // `useValueChanged(checked, …)` (`:99-104`): runs only on an actual change (upstream's
    // hook skips the first render), clearing the field's errors, marking dirty against
    // the validity baseline and running the `change` validation.
    {
        let set_dirty = field.set_dirty.clone();
        let validity_data = field.validity_data;
        let validation = field.validation.clone();
        let clear_errors = form_context.clear_errors.clone();
        let name_for_hook = name.clone();
        let first = Rc::new(Cell::new(true));
        Effect::new(move |_| {
            let current = checked.get();
            if first.get() {
                first.set(false);
                return;
            }
            (clear_errors)(name_for_hook.get().as_deref());
            let initial = validity_data.get_untracked().initial_value;
            set_dirty(checked_is_dirty(current, &initial));
            (validation.change)(Some(Value::Bool(current)), false);
        });
    }

    // `useButton({ disabled, native: nativeButton })` (`:106-109`). The hook reads its
    // `disabled` source through rg-0.2, so the computed flag rides a written mirror.
    let disabled_rg = reactive_graph::signal::RwSignal::new(disabled.get_untracked());
    {
        let disabled_rg = disabled_rg.clone();
        Effect::new(move |_| {
            reactive_graph::traits::Set::set(&disabled_rg, disabled.get());
        });
    }

    // The root's click member (`:143-156`) and the consumer's click member ride the button
    // layer's external slots, which is where upstream's `mergeProps` chain puts them: the
    // hook's internal click runs first, then the external slots in bag order
    // (`SwitchRoot.tsx:221-226`, `mergeProps.ts:229-244`).
    let input_ref_for_click = Rc::clone(&input_element_ref);
    let read_only_for_click = read_only;
    let disabled_for_click = disabled;
    let root_on_click: ElementEventHandler<web_sys::MouseEvent> = {
        let input_ref_for_click = Rc::clone(&input_ref_for_click);
        Rc::new(move |event: &web_sys::MouseEvent| {
            if read_only_for_click || disabled_for_click.get_untracked() {
                return;
            }

            event.prevent_default();

            let Some(element) = cell_read(&input_ref_for_click) else {
                return;
            };

            leptos_ui_internals::dispatch_click_with_modifiers::dispatch_click_with_modifiers(
                &element, event, 0,
            );
        })
    };
    let composed_on_click: ElementEventHandler<web_sys::MouseEvent> = {
        let consumer_on_click = consumer_on_click.clone();
        Rc::new(move |event: &web_sys::MouseEvent| {
            root_on_click(event);
            if let Some(consumer) = &consumer_on_click {
                consumer(event);
            }
        })
    };

    let button = use_button(UseButtonParams {
        disabled: disabled_rg,
        focusable_when_disabled: None,
        tab_index: 0,
        native: native_button,
        composite: None,
    });
    let button_props = (button.get_button_props)(ButtonExternalHandlers {
        on_click: Some(composed_on_click),
        on_mouse_down: None,
        on_key_down: None,
        on_key_up: None,
        on_pointer_down: None,
    });

    // `useAriaLabelledBy(ariaLabelledByProp, labelId, inputRef, !nativeButton, hiddenInputId)`
    // (`:110-116`).
    let aria_labelledby_rg = use_aria_labelled_by(
        aria_labelledby.clone(),
        labelable.label_id.clone(),
        Rc::clone(&input_element_ref),
        !native_button,
        hidden_input_id_value.clone(),
    );
    let aria_labelledby_value = mirror_rg_to_leptos(&aria_labelledby_rg);

    // -----------------------------------------------------------------------
    // The hidden input's listeners (`:159-205`)
    // -----------------------------------------------------------------------

    // The change funnel (`:172-193`).
    let on_input_change: Rc<dyn Fn(&web_sys::Event)> = {
        let on_checked_change = on_checked_change.clone();
        let checked_signal = checked;
        let input_element_ref = Rc::clone(&input_element_ref);
        Rc::new(move |event: &web_sys::Event| {
            // `event.nativeEvent.defaultPrevented` (`:174`).
            if event.default_prevented() {
                return;
            }

            if read_only {
                event.prevent_default();
                return;
            }

            let Some(target) = event.target() else {
                return;
            };
            let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
                return;
            };
            let next_checked = input.checked();

            let details = SwitchChangeEventDetails::new(
                leptos_ui_internals::floating_ui::reasons::NONE,
                event.clone(),
                None,
                (),
            );

            let outcome = run_change_funnel(
                read_only,
                false,
                on_checked_change.as_deref(),
                next_checked,
                &details,
            );

            if outcome == ChangeFunnel::Commit {
                set_checked(next_checked);
            }

            // Adaptation 2: re-assert the property so a cancelled/`readOnly` change does
            // not leave the browser-toggled input out of sync with the state.
            let _ = input.set_checked(checked_signal.get_untracked());
            let _ = &input_element_ref;
        })
    };

    // `onClick` on the hidden input (`:194-198`): the re-dispatched click is an
    // implementation detail and must not reach ancestors, which already receive the
    // original click.
    let on_input_click: ElementEventHandler<web_sys::MouseEvent> = Rc::new(|event: &web_sys::MouseEvent| {
        event.stop_propagation();
    });

    // `onFocus` on the hidden input (`:199-201`): focus the visible switch.
    let on_input_focus: Rc<dyn Fn(&web_sys::FocusEvent)> = {
        Rc::new(move |_event: &web_sys::FocusEvent| {
            let control = control_span_node
                .get()
                .map(|span| span.unchecked_into::<web_sys::Element>())
                .or_else(|| {
                    control_button_node
                        .get()
                        .map(|button| button.unchecked_into::<web_sys::Element>())
                });
            if let Some(control) = control {
                if let Ok(control) = control.dyn_into::<web_sys::HtmlElement>() {
                    let _ = control.focus();
                }
            }
        })
    };

    // -----------------------------------------------------------------------
    // The control element (`:118-157`, `:207-228`)
    // -----------------------------------------------------------------------

    // The root's own `onFocus`/`onBlur` (`:125-142`).
    let set_focused_for_focus = field.set_focused.clone();
    let on_control_focus: Rc<dyn Fn(&web_sys::FocusEvent)> = Rc::new(move |_event| {
        if !disabled.get_untracked() {
            set_focused_for_focus(true);
        }
    });
    let on_control_blur: Rc<dyn Fn(&web_sys::FocusEvent)> = {
        let set_touched = field.set_touched.clone();
        let set_focused = field.set_focused.clone();
        let validation = field.validation.clone();
        let validation_mode = field.validation_mode;
        let input_element_ref = Rc::clone(&input_element_ref);
        Rc::new(move |_event| {
            if disabled.get_untracked() {
                return;
            }
            let element = cell_read(&input_element_ref);
            let Some(element) = element else {
                return;
            };
            let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() else {
                return;
            };

            set_touched(true);
            set_focused(false);

            if validation_mode == leptos_ui_internals::form_context::FormValidationMode::OnBlur {
                (validation.commit)(Value::Bool(input.checked()));
            }
        })
    };

    // The state record handed to the parts (`:207-216`) and its walk (`:227`).
    let context_value = SwitchRootContextValue {
        checked,
        disabled,
        read_only: Signal::derive(move || read_only),
        required: Signal::derive(move || required),
        touched: field.state.touched.clone(),
        dirty: field.state.dirty.clone(),
        valid: field.state.valid.clone(),
        filled: field.state.filled.clone(),
        focused: field.state.focused.clone(),
    };

    // The consumer's `...elementProps` rest and the Field validation overlay
    // (`:225`) — adaptation 1: no attribute spread in `view!`, so one writer applies
    // them and prunes names it wrote before.
    let managed_names: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let overlay_attributes: Vec<(
        String,
        leptos_ui_internals::floating_ui::element_props::ElementAttributeFn,
    )> = {
        let mut attributes = Vec::new();
        (labelable.get_description_props)(&mut attributes);
        let mut overlay = Vec::new();
        (field.validation.get_validation_props)(disabled.get_untracked(), &mut overlay);
        attributes.extend(overlay);
        attributes
    };
    {
        let managed_names = Rc::clone(&managed_names);
        let element_attributes = element_attributes.clone();
        let overlay_attributes = overlay_attributes.clone();
        let invalid_signal = field.invalid;
        let disabled_signal = disabled;
        let control_span_node = control_span_node;
        let control_button_node = control_button_node;
        Effect::new(move |_| {
            // Track the two sources that decide the overlay's contents.
            let _ = invalid_signal.get();
            let _ = disabled_signal.get();

            let control = control_span_node
                .get()
                .map(|span| span.unchecked_into::<web_sys::Element>())
                .or_else(|| {
                    control_button_node
                        .get()
                        .map(|button| button.unchecked_into::<web_sys::Element>())
                });
            let Some(control) = control else {
                return;
            };

            let mut written: Vec<String> = Vec::new();
            for (name, value) in &element_attributes {
                let _ = control.set_attribute(name, value);
                written.push(name.clone());
            }
            for (name, value) in &overlay_attributes {
                if let Some(value) = value() {
                    let _ = control.set_attribute(name, &value);
                    written.push(name.clone());
                }
            }

            let previous = managed_names.borrow().clone();
            for name in previous {
                if !written.contains(&name) {
                    let _ = control.remove_attribute(&name);
                }
            }
            *managed_names.borrow_mut() = written;
        });
    }

    // The tag (`:218`): `span` by default, a real `<button>` under `nativeButton` — or
    // when the render element says so (adaptation 4).
    let render_tag = match &render {
        Some(RenderProp::Element { tag, .. }) => Some(tag.clone()),
        _ => None,
    };
    let renders_button = native_button || render_tag.as_deref() == Some("button");

    // The labelable/FIELD ids (`:81-84`, `:119`).
    let root_id_signal: Signal<String> = Signal::derive(move || {
        root_id(native_button, &control_id.get(), &generated_id_leptos.get())
    });

    // The state attributes' live bindings (`:227`).
    let context_for_attributes = context_value.clone();
    let state_attributes: Signal<leptos_ui_internals::state_attributes::StateAttributeProps> =
        Signal::derive(move || switch_state_attributes(&context_for_attributes.snapshot()));

    // The hidden input's `style` (`:167`).
    let input_style_signal: Signal<String> =
        Signal::derive(move || style_string(input_style(name.get().as_deref())));

    // The hidden input's props (`:159-205`).
    let input_disabled = Signal::derive(move || disabled.get());
    let input_name_signal = name.clone();
    let form_attr = form.clone();
    let required_attr = required.then(|| "true".to_string());
    let value_attr = value_prop.clone();

    // The button layer's composed click handler, native-typed back to the DOM.
    let button_click = button_props.handlers.on_click.clone();
    let button_key_down = button_props.handlers.on_key_down.clone();
    let button_key_up = button_props.handlers.on_key_up.clone();

    // The `inputRef` (`:161`, `:171`) — the consumer's handle sees the hidden input.
    {
        let input_ref = input_ref.clone();
        Effect::new(move |_| {
            let element = input_node
                .get()
                .map(|input| input.unchecked_into::<web_sys::Element>());
            input_element_ref.set(element.clone());
            let as_input = element.and_then(|element| {
                element
                    .dyn_into::<web_sys::HtmlInputElement>()
                    .ok()
            });
            if let Some(callback) = &input_ref {
                callback(as_input);
            }
        });
        // The switch element slot (`switchRef`, `:79`/`:93`).
        Effect::new(move |_| {
            let element = control_span_node
                .get()
                .map(|span| span.unchecked_into::<web_sys::Element>())
                .or_else(|| {
                    control_button_node
                        .get()
                        .map(|button| button.unchecked_into::<web_sys::Element>())
                });
            switch_element_ref.set(element);
        });
    }

    // The unchecked-value sibling (`:233-235`): only while unchecked, with a name and a
    // defined `uncheckedValue`.
    let unchecked_value_input = {
        let checked_signal = checked;
        let name_signal = name.clone();
        let unchecked_value = unchecked_value.clone();
        let form_attr = form_attr.clone();
        let disabled_signal = disabled;
        move || {
            let name_now = name_signal.get();
            let should_render = !checked_signal.get()
                && name_now.as_deref().is_some_and(|name| !name.is_empty())
                && unchecked_value.is_some();
            should_render.then(|| {
                let name_now = name_now.clone().unwrap_or_default();
                let unchecked_value = unchecked_value.clone().unwrap_or_default();
                let form_attr = form_attr.clone();
                let disabled_now = disabled_signal.get();
                view! {
                    <input
                        type="hidden"
                        form={form_attr}
                        name={name_now}
                        value={unchecked_value}
                        disabled={disabled_now}
                    />
                }
            })
        }
    };

    // The three siblings upstream returns inside the provider (`:230-238`), with the
    // visible element's tag chosen above.
    let children_view = children;
    let control = if renders_button {
        Either::Left(view! {
            <button
                node_ref=control_button_node
                type="button"
                id={move || root_id_signal.get()}
                class=class
                style=style
                role="switch"
                aria-checked={move || checked.get().to_string()}
                aria-readonly={move || aria_bool_attr(read_only)}
                aria-required={move || aria_bool_attr(required)}
                aria-labelledby={move || aria_labelledby_value.get()}
                data-checked={move || state_attributes.get().get(DATA_CHECKED).cloned()}
                data-unchecked={move || state_attributes.get().get(DATA_UNCHECKED).cloned()}
                data-disabled={move || state_attributes.get().get(DATA_DISABLED).cloned()}
                data-readonly={move || state_attributes.get().get(DATA_READONLY).cloned()}
                data-required={move || state_attributes.get().get(DATA_REQUIRED).cloned()}
                data-valid={move || state_attributes.get().get(DATA_VALID).cloned()}
                data-invalid={move || state_attributes.get().get(DATA_INVALID).cloned()}
                data-touched={move || state_attributes.get().get(DATA_TOUCHED).cloned()}
                data-dirty={move || state_attributes.get().get(DATA_DIRTY).cloned()}
                data-filled={move || state_attributes.get().get(DATA_FILLED).cloned()}
                data-focused={move || state_attributes.get().get(DATA_FOCUSED).cloned()}
                aria-disabled={move || disabled.get().then(|| "true".to_string())}
                on:click={move |event: web_sys::MouseEvent| {
                    let wrapped = BaseUIEvent::new(event);
                    if let Some(handler) = &button_click {
                        handler(wrapped.inner());
                    }
                }}
                on:keydown={move |event: web_sys::KeyboardEvent| {
                    if let Some(handler) = &button_key_down {
                        handler(&BaseUIEvent::new(event));
                    }
                }}
                on:keyup={move |event: web_sys::KeyboardEvent| {
                    if let Some(handler) = &button_key_up {
                        handler(&BaseUIEvent::new(event));
                    }
                }}
                on:focus={move |event: web_sys::FocusEvent| on_control_focus(&event)}
                on:blur={move |event: web_sys::FocusEvent| on_control_blur(&event)}
            >
                {children_view()}
            </button>
        })
    } else {
        Either::Right(view! {
            <span
                node_ref=control_span_node
                id={move || root_id_signal.get()}
                class=class
                style=style
                role="switch"
                aria-checked={move || checked.get().to_string()}
                aria-readonly={move || aria_bool_attr(read_only)}
                aria-required={move || aria_bool_attr(required)}
                aria-labelledby={move || aria_labelledby_value.get()}
                data-checked={move || state_attributes.get().get(DATA_CHECKED).cloned()}
                data-unchecked={move || state_attributes.get().get(DATA_UNCHECKED).cloned()}
                data-disabled={move || state_attributes.get().get(DATA_DISABLED).cloned()}
                data-readonly={move || state_attributes.get().get(DATA_READONLY).cloned()}
                data-required={move || state_attributes.get().get(DATA_REQUIRED).cloned()}
                data-valid={move || state_attributes.get().get(DATA_VALID).cloned()}
                data-invalid={move || state_attributes.get().get(DATA_INVALID).cloned()}
                data-touched={move || state_attributes.get().get(DATA_TOUCHED).cloned()}
                data-dirty={move || state_attributes.get().get(DATA_DIRTY).cloned()}
                data-filled={move || state_attributes.get().get(DATA_FILLED).cloned()}
                data-focused={move || state_attributes.get().get(DATA_FOCUSED).cloned()}
                aria-disabled={move || disabled.get().then(|| "true".to_string())}
                on:click={move |event: web_sys::MouseEvent| {
                    let wrapped = BaseUIEvent::new(event);
                    if let Some(handler) = &button_click {
                        handler(wrapped.inner());
                    }
                }}
                on:keydown={move |event: web_sys::KeyboardEvent| {
                    if let Some(handler) = &button_key_down {
                        handler(&BaseUIEvent::new(event));
                    }
                }}
                on:keyup={move |event: web_sys::KeyboardEvent| {
                    if let Some(handler) = &button_key_up {
                        handler(&BaseUIEvent::new(event));
                    }
                }}
                on:focus={move |event: web_sys::FocusEvent| on_control_focus(&event)}
                on:blur={move |event: web_sys::FocusEvent| on_control_blur(&event)}
            >
                {children_view()}
            </span>
        })
    };

    // The context provider (`:231`) — upstream wraps the fragment, and the parts read it.
    provide_switch_root_context(context_value);

    let on_input_click_handler = on_input_click.clone();
    let on_input_change_handler = on_input_change.clone();

    view! {
        <>
            {control}
            {unchecked_value_input}
            <input
                node_ref=input_node
                type="checkbox"
                id={move || hidden_input_id_value.clone()}
                name={move || input_name_signal.get()}
                form={form_attr}
                style={move || input_style_signal.get()}
                tabindex="-1"
                aria-hidden="true"
                disabled={move || input_disabled.get()}
                required={required_attr}
                prop:checked={move || checked.get()}
                value={value_attr}
                on:change={move |event: web_sys::Event| on_input_change_handler(&event)}
                on:click={move |event: web_sys::MouseEvent| on_input_click_handler(&event)}
                on:focus={move |event: web_sys::FocusEvent| on_input_focus(&event)}
            />
        </>
    }
}

/// Reads an `Rc<Cell<Option<T>>>` slot when `T` is not `Copy` — upstream's ref objects are
/// mutable slots, and `Cell::get` needs `Copy`, so the port swaps the value out and back
/// (the checkbox port's `Rc<Cell<..>>` slots are read the same way).
fn cell_read<T: Clone>(cell: &Rc<Cell<Option<T>>>) -> Option<T> {
    let value = cell.replace(None);
    if let Some(inner) = value.clone() {
        cell.set(Some(inner));
    }
    value
}

/// The `visuallyHidden` recipes as a `style` string (`@base-ui/utils/visuallyHidden`).
fn style_string(declarations: &[(&str, &str)]) -> String {
    declarations
        .iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<Vec<_>>()
        .join(" ")
}
