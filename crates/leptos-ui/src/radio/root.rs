//! `Radio.Root` — port of `packages/react/src/radio/root/RadioRoot.tsx`
//! (the `library: radio` TODO item; `specs/library/radio/behavior.md`,
//! `specs/library/radio/implementation.md`).
//!
//! ## What upstream does
//!
//! Root is a **stateless selection proxy** (implementation.md's own heading for it):
//! `checked` is a pure per-render derivation (`RadioRoot.tsx:83`), never local state, so
//! selection truth lives entirely in the surrounding group's context state and the
//! group's re-render flows back down as a new derivation. State mutation is delegated
//! end-to-end: the hidden input's `onChange` builds
//! `createChangeEventDetails(REASONS.none, event.nativeEvent)` and calls
//! `setCheckedValue(value, details)` (`:184-203`), honouring the cancelable-decision
//! protocol (`details.isCanceled`) before marking the Field touched (`:202`).
//!
//! The element is a `span` by default and a real `<button>` under `nativeButton`
//! (`:218`, `:240`), and — the structural fact that shapes this port — the render path
//! **branches on whether a group is above**: inside a group the element is a
//! `CompositeItem` (`:250-260`, so the group's roving focus and arrow navigation see
//! it), standalone it is a plain `useRenderElement('span', …)` with
//! `enabled: !isRadioGroup` (`:240-246`). Both branches receive the same state, refs,
//! props chain and state mapping, and both are wrapped by the same
//! `RadioRootContext.Provider` with the hidden `<input>` as their sibling (`:248-266`).
//!
//! ## The port's split
//!
//! - The pure folds, the state records and the state→attribute walk live in
//!   [`crate::radio::state`] (host-testable, citing the same upstream lines).
//! - [`crate::radio::context`] carries the state to the parts.
//! - This module is the DOM/event boundary: the two render branches, the context
//!   provider, the Field/Labelable integration, the hidden input and its listeners.
//!
//! Runtime law (the crate's dual-runtime split): leptos 0.7 tracks reactive-graph 0.1
//! while the internals crate's hooks are typed over 0.2, so the machines are re-homed on
//! leptos signals (the checkbox/switch precedent) and the rg-0.2 handles the internals
//! hand back (`useBaseUiId`, `useLabelableId`, `useAriaLabelledBy`, `useButton`) are
//! bridged with [`mirror_rg_to_leptos`] / a written rg-0.2 mirror.
//!
//! ## Documented adaptations (never silent)
//!
//! 1. **`view!` has no attribute spread.** The consumer's `...elementProps` rest (`:50`)
//!    and the Field validation overlay (`:235-237`) are dynamic maps, so one writer
//!    effect applies them to the control node with stale-name pruning (the checkbox
//!    writer precedent). `state_attributes_mapping` is deliberately NOT handed to the
//!    render layer: that layer folds a mapping *once* at description time from a static
//!    state map, whereas this unit's `data-*` must track live signals — so the walk runs
//!    inside the effect instead, through the same
//!    [`crate::radio::state::radio_state_attributes`] the description layer would call.
//! 2. **The change details' payload type.** Upstream hands the *same* details object to
//!    `setCheckedValue` and then reads `details.isCanceled` (`:194-200`), with
//!    `event.nativeEvent` as its payload. The sibling unit `library: radio-group` already
//!    published its setter as taking `&RadioGroupChangeEventDetails`
//!    (`crates/leptos-ui/src/radio_group.rs:108-114`, payload `()` there), so radio
//!    builds *that* type, passes it, and reads `is_canceled()` off it. The veto semantics
//!    implementation.md calls the "cancelable-decision protocol" are preserved exactly —
//!    one object, cancelled by the group, read back here (`:196-200`) — and only the
//!    payload's static type differs, because the consumer of the value is the group.
//! 3. **`render`** is honoured for its element form's tag (`<button />`) — the
//!    description-layer render *function* is the crate-wide gap already recorded as
//!    `library: the view paths drop render's element form`.
//! 4. **`suppressHydrationWarning`** (`:264`) is N/A (CSR-only app), and so is the
//!    React-17 id fallback implementation.md mentions for `useBaseUiId`.
//! 5. **`inputRef`** (`:94`, `:329-331`) is fired at mount in upstream's merged-ref order
//!    (`inputRefProp`, the internal slot, the group's `registerInputRef`, the Field
//!    registration), so the consumer's handle observes the real hidden input — the
//!    observable implementation.md's untested item 7 leaves open.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::either::Either;
use leptos::prelude::*;
use reactive_graph::signal::RwSignal as RgRwSignal;
use wasm_bindgen::JsCast;

use leptos_ui_internals::composite::ACTIVE_COMPOSITE_ITEM;
use leptos_ui_internals::composite_view::{CompositeItemComponentProps, composite_item};
use leptos_ui_internals::floating_ui::element_props::{ElementAttributeFn, ElementEventHandler};
use leptos_ui_internals::labelable_provider::{
    UseLabelableIdParams, use_aria_labelled_by, use_labelable_context, use_labelable_id,
};
use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::state_attributes::StateAttributeProps;
use leptos_ui_internals::types::BaseUIEvent;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_button::{ButtonExternalHandlers, UseButtonParams, use_button};
use leptos_ui_internals::use_render_element::{
    RenderElementHandlers, RenderElementProps, RenderedElement, UseRenderElementComponentProps,
    UseRenderElementParams, native_to_base_ui, use_render_element,
};

use crate::field::context::{use_field_item_context, use_field_root_context};
use crate::field::validation_helpers::mirror_rg_to_leptos;
use crate::radio::context::{RadioRootContextValue, provide_radio_root_context};
use crate::radio::state::{
    MANAGED_STATE_ATTRIBUTES, aria_bool_attr, effective_disabled, effective_read_only,
    effective_required, has_value, hidden_input_id, input_style, input_value_attr, is_checked,
    radio_state_attributes, root_id, style_string,
};
use crate::radio_group::{
    REASON_NONE, RadioGroupChangeEventDetails, RadioGroupItemMetadata, use_radio_group_context,
};

/// The Root props — upstream's `RadioRootProps` (`RadioRoot.tsx:310-332`) plus the
/// element-level members `useRenderElement` resolves from `componentProps` (`:39-54`).
#[derive(Default)]
pub struct RadioRootViewProps {
    /// `value` (`:315`) — the unique identifying value of the radio in a group. Double
    /// [`Option`]: outer = upstream's `undefined` (prop absent), inner = `Value | null`
    /// (see `crate::radio::state`'s module docs and [`is_checked`]).
    pub value: Option<Option<String>>,
    /// `disabled` (`:319`, default `false`).
    pub disabled: bool,
    /// `required` (`:323`, default `false`).
    pub required: bool,
    /// `readOnly` (`:327`, default `false`).
    pub read_only: bool,
    /// `'aria-labelledby'` (`:44`).
    pub aria_labelledby: Option<String>,
    /// `inputRef` (`:331`) — a handle on the hidden input.
    pub input_ref: Option<Rc<dyn Fn(Option<web_sys::HtmlInputElement>)>>,
    /// `nativeButton` (`:47`, default `false`) — render a real `<button>`.
    pub native_button: bool,
    /// `id` (`:48`) — the labelable id: the control's under `nativeButton`, the hidden
    /// input's otherwise (`:117`, `:131`).
    pub id: Option<String>,
    /// `render`/`className`/`style` (`:39-40`, `:49`) — the description layer's members.
    pub render_class_style: UseRenderElementComponentProps,
    /// The consumer's `...elementProps` rest (`:50`) as plain attributes.
    pub element_attributes: Vec<(String, String)>,
    /// The consumer's handlers out of the same rest (`:50`) — the `onClick` the behavior
    /// spec's `stopPropagation` case exercises (`RadioRoot.test.tsx:153-171`).
    pub handlers: RadioRootHandlers,
    /// The Root's subtree — the Indicator lives inside the visible control upstream
    /// (`:248-263`: the children ride the element's props bag and become its content).
    pub children: Option<ChildrenFn>,
}

/// The consumer's handler members out of the `...elementProps` rest (`:50`).
#[derive(Clone, Default)]
pub struct RadioRootHandlers {
    /// The consumer's `onClick` — runs before the internal click handler (the
    /// later-bag-first merge rule), which is what makes behavior.md:58's
    /// `stopPropagation` case observable while the selection still proceeds
    /// (`RadioRoot.test.tsx:153-171`).
    pub on_click: Option<ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>>>,
    /// The consumer's `onMouseDown`.
    pub on_mouse_down: Option<ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>>>,
    /// The consumer's `onMouseMove`.
    pub on_mouse_move: Option<ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>>>,
    /// The consumer's `onKeyDown`.
    pub on_key_down: Option<ElementEventHandler<BaseUIEvent<web_sys::KeyboardEvent>>>,
    /// The consumer's `onKeyUp`.
    pub on_key_up: Option<ElementEventHandler<BaseUIEvent<web_sys::KeyboardEvent>>>,
    /// The consumer's `onPointerDown`.
    pub on_pointer_down: Option<ElementEventHandler<BaseUIEvent<web_sys::PointerEvent>>>,
    /// The consumer's `onFocus`.
    pub on_focus: Option<ElementEventHandler<BaseUIEvent<web_sys::FocusEvent>>>,
    /// The consumer's `onBlur`.
    pub on_blur: Option<ElementEventHandler<BaseUIEvent<web_sys::FocusEvent>>>,
    /// The consumer's `onContextMenu`.
    pub on_context_menu: Option<ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>>>,
}

/// The view seam (the `checkbox_root_view`/`switch_root_view` convention). Establishes
/// its own rg-0.2 bridge owner so an enclosing `Field.Root`/`RadioGroup`'s rg-0.2 contexts
/// stay visible, and returns the two-sibling fragment upstream returns inside the
/// provider (`RadioRoot.tsx:248-266`: the control, then the hidden input).
pub fn radio_root_view(props: RadioRootViewProps) -> impl IntoView {
    let bridge_owner = reactive_graph::owner::Owner::new();
    let view = bridge_owner.with(move || radio_root_body(props));
    // The bridge owner outlives the subtree (the switch/checkbox/direction-provider precedent).
    std::mem::forget(bridge_owner);
    view
}

/// The Root body — must be called inside a reactive owner (`use_button`'s update-disabled
/// effect and the composite item's registration register there).
fn radio_root_body(props: RadioRootViewProps) -> impl IntoView {
    let RadioRootViewProps {
        value,
        disabled: disabled_prop,
        required: required_prop,
        read_only: read_only_prop,
        aria_labelledby,
        input_ref,
        native_button,
        id: id_prop,
        render_class_style,
        element_attributes,
        handlers: consumer_handlers,
        children,
    } = props;

    // `useRadioGroupContext()` / `useFieldRootContext()` / `useFieldItemContext()` /
    // `useLabelableContext()` (`:53`, `:69-76`) — every read falls back to its inert shell
    // outside the matching provider, which is what lets a standalone Root render.
    let group = use_radio_group_context();
    let field = use_field_root_context();
    let field_item = use_field_item_context();
    let labelable = use_labelable_context();

    let in_group = group.is_some();

    // The subtree crosses a `Send + Sync` boundary because a `view!` child closure is
    // stored that way; `SendWrapper` is the crate's bridge for local handles (the
    // `radio_group_view` mechanism).
    let children = send_wrapper::SendWrapper::new(children);
    let children_view = move || children.as_ref().map(|children| children());

    // The group's per-field fallbacks (`:55-67`): every member is optional, and
    // `setCheckedValue`/`setTouched`/`registerInputRef` fall back to `NOOP`.
    //
    // The group's context is typed over reactive-graph 0.2 (its own unit's runtime), while
    // this crate's signals are leptos 0.7's — so the two members the Root *reads* across
    // renders are mirrored into leptos signals here, and the two it *writes* keep the
    // group's own handle (the `checkbox::indicator` bridge, one crossing at a time).
    let group_checked_source = group.as_ref().map(|group| group.checked_value.clone());
    let group_checked_mirror: Option<RwSignal<Option<String>>> =
        group_checked_source.as_ref().map(|source| {
            let mirror = RwSignal::new(reactive_graph::traits::GetUntracked::get_untracked(
                source,
            ));
            let source = source.clone();
            reactive_graph::effect::Effect::new(move |_| {
                mirror.set(reactive_graph::traits::Get::get(&source));
            });
            mirror
        });

    let group_set_touched = group.as_ref().map(|group| group.set_touched);
    let group_disabled = group.as_ref().is_some_and(|group| group.disabled);
    let group_read_only = group.as_ref().is_some_and(|group| group.read_only);
    let group_required = group.as_ref().is_some_and(|group| group.required);
    let group_form = group.as_ref().and_then(|group| group.form.clone());
    let group_name = group.as_ref().and_then(|group| group.name.clone());

    // `const disabled = fieldDisabled || fieldItemContext.disabled || disabledGroup ||
    // disabledProp` (`:78`).
    let field_disabled = field.disabled.clone();
    let disabled: Signal<bool> = Signal::derive(move || {
        effective_disabled(
            field_disabled.get(),
            field_item.disabled,
            group_disabled,
            disabled_prop,
        )
    });

    // `readOnly = readOnlyGroup || readOnlyProp` (`:79`) / `required = requiredGroup ||
    // requiredProp` (`:80`). Neither is reactive in the group unit's published context
    // (plain values there), so these are plain flags.
    let read_only = effective_read_only(group_read_only, read_only_prop);
    let required = effective_required(group_required, required_prop);

    // `checked = groupContext ? checkedValue === value : value === ''` (`:83`).
    let value_for_checked = value.clone();
    let checked: Signal<bool> = Signal::derive(move || {
        let inner = value_for_checked.as_ref().map(|value| value.as_deref());
        match &group_checked_mirror {
            Some(group_value) => is_checked(true, group_value.get().as_deref(), inner),
            None => is_checked(false, None, inner),
        }
    });

    // The element slots the machines read (`:85-86`): leptos has no ref fork, so the
    // nodes arrive through `NodeRef`s and mount effects resync the `Rc<Cell>` slots.
    let radio_element_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    let input_element_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    let control_span_node: NodeRef<leptos::html::Span> = NodeRef::new();
    let control_button_node: NodeRef<leptos::html::Button> = NodeRef::new();
    let input_node: NodeRef<leptos::html::Input> = NodeRef::new();

    // `const id = useBaseUiId()` (`:115`) — no override: the generated instance id. The
    // labelable id (which *can* come from the `id` prop) is `inputId` below. The hook is
    // typed over reactive-graph 0.2, so the (constant) override rides an rg-0.2 signal.
    let generated_id = use_base_ui_id(RgRwSignal::new(None::<String>));
    let generated_id_leptos = mirror_rg_to_leptos(&generated_id);

    // `const inputId = useLabelableId({ id: idProp })` (`:116`).
    let control_id_rg = use_labelable_id(UseLabelableIdParams {
        id: id_prop.filter(|id| !id.is_empty()),
        enabled: true,
    });
    let control_id = mirror_rg_to_leptos(&control_id_rg);
    let control_id_for_labels = control_id.get_untracked();

    // `hiddenInputId = nativeButton ? undefined : inputId` (`:117`).
    let hidden_input_id_value = hidden_input_id(native_button, &control_id_for_labels);

    // `useAriaLabelledBy(ariaLabelledByProp, labelId, inputRef, !nativeButton, hiddenInputId)`
    // (`:118-124`) — the fallback label association behavior.md's Accessibility section
    // describes (`RadioRoot.test.tsx:84-134`).
    let aria_labelledby_rg = use_aria_labelled_by(
        aria_labelledby.clone(),
        labelable.label_id.clone(),
        Rc::clone(&input_element_ref),
        !native_button,
        hidden_input_id_value.clone(),
    );
    let aria_labelledby_value = mirror_rg_to_leptos(&aria_labelledby_rg);

    // `useIsoLayoutEffect(() => { … registerInputRef(inputRef.current) }, [checked,
    // disabled, registerInputRef])` (`:102-113`): the selected radio is the group's
    // representative input — except a checked-but-disabled one, which deregisters so the
    // group forwards a live representative.
    if let Some(register_input_ref) = group
        .as_ref()
        .map(|group| group.register_input_ref.clone())
    {
        let input_element_ref = Rc::clone(&input_element_ref);
        Effect::new(move |_| {
            let checked_now = checked.get();
            let disabled_now = disabled.get();

            let cell = cell_read(&input_element_ref)
                .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok());

            if checked_now && disabled_now {
                let _ = register_input_ref(None);
                return;
            }

            let _ = register_input_ref(cell);
        });
    }

    // -----------------------------------------------------------------------
    // The hidden input's listeners (`:184-211`)
    // -----------------------------------------------------------------------

    // The change funnel (`:184-203`). Adaptation 2 governs the details' payload type.
    let on_input_change: Rc<dyn Fn(&web_sys::Event)> = {
        let set_checked_value = group.as_ref().map(|group| group.set_checked_value.clone());
        let set_field_touched = field.set_touched;
        let value_for_change = value.clone();
        Rc::new(move |event: &web_sys::Event| {
            // `event.nativeEvent.defaultPrevented` (`:186`).
            if event.default_prevented() {
                return;
            }

            // `if (disabled || readOnly || value === undefined) return` (`:190`).
            if read_only || !has_value(value_for_change.as_ref()) {
                return;
            }

            let details = RadioGroupChangeEventDetails::new(REASON_NONE, (), None, ());

            if let Some(set_checked_value) = &set_checked_value {
                // `setCheckedValue(value, details)` (`:196`) — the inner encoding is the
                // group's own `RadioGroupValue`.
                (set_checked_value)(value_for_change.clone().flatten(), &details);
            }

            // `if (details.isCanceled) return` (`:198-200`).
            if details.is_canceled() {
                return;
            }

            // `setFieldTouched(true)` (`:202`).
            (set_field_touched)(true);
        })
    };

    // `onClick` on the hidden input (`:204-208`): the click the control re-dispatched is
    // an implementation detail and must not reach ancestors, which already receive the
    // original click (behavior.md:57).
    let on_input_click = Rc::new(|event: &web_sys::MouseEvent| {
        event.stop_propagation();
    });

    // `onFocus` on the hidden input (`:209-211`): focus the visible control.
    let on_input_focus = {
        let control_element = {
            let radio_element_ref = Rc::clone(&radio_element_ref);
            move || cell_read(&radio_element_ref)
        };
        Rc::new(move |_event: &web_sys::FocusEvent| {
            if let Some(control) = control_element() {
                if let Ok(control) = control.dyn_into::<web_sys::HtmlElement>() {
                    let _ = control.focus();
                }
            }
        })
    };

    // -----------------------------------------------------------------------
    // The control element (`:126-162`, `:230-263`)
    // -----------------------------------------------------------------------

    // `rootProps` (`:126-162`) in bag form, so the merge chain and the writer effect see
    // it (adaptation 1).
    let root_id_signal: Signal<String> = Signal::derive(move || {
        root_id(native_button, &control_id.get(), &generated_id_leptos.get())
    });
    let checked_attr = checked;
    let active_composite_attr = checked;
    let aria_labelledby_for_attr = aria_labelledby_value.clone();

    let on_control_key_down: ElementEventHandler<BaseUIEvent<web_sys::KeyboardEvent>> = {
        Rc::new(move |event: &BaseUIEvent<web_sys::KeyboardEvent>| {
            // `:132-138` — a radio activates with Space only; preventing the keydown's
            // default stops `useButton` from turning Enter into a click.
            if event.inner().key() == "Enter" {
                event.inner().prevent_default();
            }
        })
    };

    let on_control_click: ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>> = {
        let disabled_for_click = disabled;
        let input_element_ref_for_click = Rc::clone(&input_element_ref);
        Rc::new(move |event: &BaseUIEvent<web_sys::MouseEvent>| {
            // `:140-142`.
            if event.inner().default_prevented()
                || disabled_for_click.get_untracked()
                || read_only
            {
                return;
            }

            // `:144`, then the synthetic click on the hidden input (`:146-151`,
            // `dispatchClickWithModifiers` — preserving modifier state).
            event.inner().prevent_default();

            let Some(element) = cell_read(&input_element_ref_for_click) else {
                return;
            };

            leptos_ui_internals::dispatch_click_with_modifiers::dispatch_click_with_modifiers(
                &element,
                event.inner(),
                0,
            );
        })
    };

    // `onFocus` (`:153-161`): the Field-touched handoff for label-click activation —
    // when the group flagged `touched`, the control's focus clicks the input and resets
    // the flag.
    let on_control_focus: ElementEventHandler<BaseUIEvent<web_sys::FocusEvent>> = {
        let input_element_ref = Rc::clone(&input_element_ref);
        let disabled_for_focus = disabled;
        Rc::new(move |event: &BaseUIEvent<web_sys::FocusEvent>| {
            if event.inner().default_prevented()
                || disabled_for_focus.get_untracked()
                || read_only
            {
                return;
            }

            let Some(set_touched) = group_set_touched else {
                return;
            };

            // `if (!touched) return` (`:154`) — the group's own touched flag, read live
            // off the group's handle.
            if !reactive_graph::traits::GetUntracked::get_untracked(&set_touched) {
                return;
            }

            // `inputRef.current?.click()` (`:158`).
            if let Some(element) = cell_read(&input_element_ref) {
                if let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() {
                    let _ = input.click();
                }
            }

            // `setTouched(false)` (`:160`).
            reactive_graph::traits::Set::set(&set_touched, false);
        })
    };

    let base_bag = RenderElementProps {
        handlers: RenderElementHandlers {
            on_key_down: Some(on_control_key_down),
            on_click: Some(on_control_click),
            on_focus: Some(on_control_focus),
            attributes: vec![
                (
                    "role".to_string(),
                    Rc::new(|| Some("radio".to_string())) as ElementAttributeFn,
                ),
                (
                    "aria-checked".to_string(),
                    Rc::new(move || Some(checked_attr.get().to_string())) as ElementAttributeFn,
                ),
                (
                    "aria-labelledby".to_string(),
                    Rc::new(move || aria_labelledby_for_attr.get()) as ElementAttributeFn,
                ),
                (
                    "id".to_string(),
                    Rc::new(move || Some(root_id_signal.get())) as ElementAttributeFn,
                ),
                // `[ACTIVE_COMPOSITE_ITEM]: checked ? '' : undefined` (`:130`) — the
                // marker the composite internals scan for to adopt an active item.
                (
                    ACTIVE_COMPOSITE_ITEM.to_string(),
                    Rc::new(move || active_composite_attr.get().then(String::new))
                        as ElementAttributeFn,
                ),
            ],
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    };

    // The consumer's `...elementProps` rest (`:50`, `:232`) in bag form.
    let element_bag = RenderElementProps {
        handlers: RenderElementHandlers {
            on_focus: consumer_handlers.on_focus.clone(),
            on_blur: consumer_handlers.on_blur.clone(),
            on_click: consumer_handlers.on_click.clone(),
            on_mouse_down: consumer_handlers.on_mouse_down.clone(),
            on_mouse_move: consumer_handlers.on_mouse_move.clone(),
            on_context_menu: consumer_handlers.on_context_menu.clone(),
            on_key_down: consumer_handlers.on_key_down.clone(),
            on_key_up: consumer_handlers.on_key_up.clone(),
            on_pointer_down: consumer_handlers.on_pointer_down.clone(),
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

    // `useButton({ disabled, native: nativeButton, composite: false })` (`:164-168`).
    // The hook reads its `disabled` source through rg-0.2, so the computed flag rides a
    // written mirror.
    let disabled_rg = RgRwSignal::new(disabled.get_untracked());
    {
        let disabled_rg = disabled_rg;
        Effect::new(move |_| {
            reactive_graph::traits::Set::set(&disabled_rg, disabled.get());
        });
    }

    let button = use_button(UseButtonParams {
        disabled: disabled_rg,
        focusable_when_disabled: None,
        tab_index: 0,
        native: native_button,
        composite: None,
    });

    // `props = [rootProps, elementProps, getButtonProps, getDescriptionProps,
    // validation ? getter : EMPTY_OBJECT]` (`:230-238`). The bag order is upstream's; the
    // button layer arrives as a props-getter resolved against the merged-so-far handlers,
    // which is exactly the `Getter` semantics the internals' `PropsSource` models
    // (`mergeProps.ts:25-31`, later bag wins, later handler runs first).
    let button_getter: PropsSource = {
        let button_get_props = Rc::clone(&button.get_button_props);
        PropsSource::Getter(Rc::new(move |merged: &RenderElementProps| {
            let button_props = (button_get_props)(ButtonExternalHandlers {
                on_click: merged
                    .handlers
                    .on_click
                    .clone()
                    .map(base_ui_to_native::<web_sys::MouseEvent>),
                on_mouse_down: merged
                    .handlers
                    .on_mouse_down
                    .clone()
                    .map(base_ui_to_native::<web_sys::MouseEvent>),
                on_key_down: merged.handlers.on_key_down.clone(),
                on_key_up: merged.handlers.on_key_up.clone(),
                on_pointer_down: merged
                    .handlers
                    .on_pointer_down
                    .clone()
                    .map(base_ui_to_native::<web_sys::PointerEvent>),
            });

            // The button layer's five composed handlers win the five slots; the remaining
            // pass-through slots survive from the merged-so-far bag, and the button
            // layer's attributes are overridden by the merged-so-far attributes (the
            // checkbox precedent).
            let mut attributes = button_props.attributes.clone();
            for (name, value) in &merged.handlers.attributes {
                match attributes.iter_mut().find(|(existing, _)| existing == name) {
                    Some(slot) => slot.1 = Rc::clone(value),
                    None => attributes.push((name.clone(), Rc::clone(value))),
                }
            }

            RenderElementProps {
                handlers: RenderElementHandlers {
                    on_click: button_props
                        .handlers
                        .on_click
                        .clone()
                        .map(native_to_base_ui),
                    on_mouse_down: button_props
                        .handlers
                        .on_mouse_down
                        .clone()
                        .map(native_to_base_ui),
                    on_pointer_down: button_props
                        .handlers
                        .on_pointer_down
                        .clone()
                        .map(native_to_base_ui),
                    on_key_down: button_props.handlers.on_key_down.clone(),
                    on_key_up: button_props.handlers.on_key_up.clone(),
                    on_focus: merged.handlers.on_focus.clone(),
                    on_blur: merged.handlers.on_blur.clone(),
                    on_mouse_move: merged.handlers.on_mouse_move.clone(),
                    on_context_menu: merged.handlers.on_context_menu.clone(),
                    attributes,
                },
                class: merged.class.clone(),
                style: merged.style.clone(),
                inner_html: merged.inner_html.clone(),
                ref_callback: merged.ref_callback.clone(),
            }
        }))
    };

    // The description-layer props chain (`:230-238`), with the Field validation overlay
    // (`:235-237`, `validation ? (props) => validation.getValidationProps(disabled, props)
    // : EMPTY_OBJECT`) as its last member.
    let mut props_bags = vec![
        PropsSource::Static(base_bag),
        PropsSource::Static(element_bag),
        button_getter,
    ];
    if let Some(validation) = group.as_ref().map(|group| group.validation.clone()) {
        let disabled_for_validation = disabled;
        props_bags.push(PropsSource::Getter(Rc::new(
            move |merged: &RenderElementProps| {
                let mut merged = merged.clone();
                let mut overlay: Vec<(String, ElementAttributeFn)> = Vec::new();
                (validation.get_validation_props)(
                    disabled_for_validation.get_untracked(),
                    &mut overlay,
                );
                for (name, value) in overlay {
                    match merged
                        .handlers
                        .attributes
                        .iter_mut()
                        .find(|(existing, _)| existing == &name)
                    {
                        Some(slot) => slot.1 = Rc::clone(&value),
                        None => merged.handlers.attributes.push((name, value)),
                    }
                }
                merged
            },
        )));
    }

    // The labelable description merge (`:76`, `getDescriptionProps`).
    {
        let get_description_props = labelable.get_description_props.clone();
        props_bags.push(PropsSource::Getter(Rc::new(
            move |merged: &RenderElementProps| {
                let mut merged = merged.clone();
                let mut overlay: Vec<(String, ElementAttributeFn)> = Vec::new();
                (get_description_props)(&mut overlay);
                for (name, value) in overlay {
                    match merged
                        .handlers
                        .attributes
                        .iter_mut()
                        .find(|(existing, _)| existing == &name)
                    {
                        Some(slot) => slot.1 = Rc::clone(&value),
                        None => merged.handlers.attributes.push((name, value)),
                    }
                }
                merged
            },
        )));
    }

    // The refs (`:229`): `[forwardedRef, radioRef, buttonRef]`. `forwardedRef` and
    // `radioRef` are observed by the mount effect below; `buttonRef` rides here.
    let refs: Vec<leptos_ui_utils::use_merged_refs::InputRef<web_sys::Element>> = {
        let button_ref = Rc::clone(&button.button_ref);
        vec![leptos_ui_utils::use_merged_refs::InputRef::Callback(Rc::new(
            move |instance: Option<&web_sys::Element>| {
                button_ref(instance.map(|element| {
                    element.clone().unchecked_into::<web_sys::HtmlElement>()
                }));
                None
            },
        ))]
    };

    // The state record handed to the parts (`:214-225`).
    let context_value = RadioRootContextValue {
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

    let state_map = context_value.snapshot().to_state_map();

    // The render-path switch (`:240-263`). Each arm is exclusive, so the shared props move
    // into whichever branch runs (upstream builds both and renders one, `:240-263`).
    let rendered: Option<RenderedElement> = if in_group {
        composite_item(CompositeItemComponentProps {
            metadata: Some(RadioGroupItemMetadata),
            render_class_style,
            tag: "span".to_string(),
            state: state_map,
            state_attributes_mapping: None,
            refs,
            props: props_bags,
            element_props: RenderElementProps::default(),
        })
    } else {
        use_render_element(
            "span",
            render_class_style,
            UseRenderElementParams {
                enabled: !in_group,
                state: &state_map,
                refs,
                props: props_bags,
                state_attributes_mapping: None,
            },
        )
    };
    let rendered = rendered.expect("the radio control always renders");

    // The tag (`:218`, `:240`): a `span`, a real `<button>` under `nativeButton`, or the
    // render element's own choice (adaptation 3).
    let renders_button = native_button || rendered.tag == "button";

    let class_value = rendered.props.class.clone();
    let style_value = rendered.props.style.clone();

    // The state attributes' live bindings (`:245`) — recomputed inside the writer so a
    // selection change re-writes the pair (adaptation 1).
    let context_for_attributes = context_value.clone();
    let state_attributes: Signal<StateAttributeProps> =
        Signal::derive(move || radio_state_attributes(&context_for_attributes.snapshot()));
    let state_for_writer = context_value.clone();

    let control_element = {
        let control_span_node = control_span_node;
        let control_button_node = control_button_node;
        move || -> Option<web_sys::Element> {
            control_span_node
                .get()
                .map(|span| span.unchecked_into::<web_sys::Element>())
                .or_else(|| {
                    control_button_node
                        .get()
                        .map(|button| button.unchecked_into::<web_sys::Element>())
                })
        }
    };

    // The writer: the merged bag's attributes, then the state walk, with stale-name
    // pruning (the checkbox writer precedent).
    let managed_names: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    {
        let managed_names = Rc::clone(&managed_names);
        let bag_attributes = rendered.props.handlers.attributes.clone();
        let class_value = class_value.clone();
        let style_value = style_value.clone();
        let control_element = control_element.clone();
        let _ = state_for_writer;
        Effect::new(move |_| {
            let Some(element) = control_element() else {
                return;
            };

            let walk = state_attributes.get();

            let mut attributes: Vec<(String, Option<String>)> = Vec::new();
            if let Some(class) = &class_value {
                attributes.push(("class".to_string(), Some(class.clone())));
            }
            if !style_value.is_empty() {
                attributes.push((
                    "style".to_string(),
                    Some(
                        style_value
                            .iter()
                            .map(|(property, value)| format!("{property}: {value};"))
                            .collect::<Vec<_>>()
                            .join(" "),
                    ),
                ));
            }
            for (name, value) in &bag_attributes {
                attributes.push((name.clone(), value()));
            }
            for name in MANAGED_STATE_ATTRIBUTES {
                attributes.push((name.to_string(), walk.get(name).cloned()));
            }

            let previous = managed_names.borrow().clone();
            let mut written: Vec<String> = Vec::new();
            for (name, value) in attributes {
                if let Some(value) = value {
                    let _ = element.set_attribute(&name, &value);
                    written.push(name);
                }
            }
            for name in previous {
                if !written.contains(&name) {
                    let _ = element.remove_attribute(&name);
                }
            }
            *managed_names.borrow_mut() = written;
        });
    }

    // The handlers' attach (`:232`'s spread): one registration point, holding the five
    // composed button slots plus the pass-throughs.
    {
        let rendered = rendered.clone();
        let control_element = control_element.clone();
        Effect::new(move |_| {
            let Some(element) = control_element() else {
                return;
            };
            let target: &web_sys::EventTarget = element.unchecked_ref();
            let cleanup = rendered.props.handlers.attach_to(target);
            // Leptos-side teardown, deliberately: this effect body runs on the leptos
            // (rg-0.1) runtime, where no rg-0.2 owner is current and
            // `reactive_graph::owner::on_cleanup` silently drops its closure — so every
            // listener would be torn back off before the first click could reach it (the
            // checkbox/avatar/context-menu note).
            let cleanup = send_wrapper::SendWrapper::new(RefCell::new(cleanup));
            leptos::prelude::on_cleanup(move || {
                if let Some(cleanup) = cleanup.borrow_mut().take() {
                    cleanup();
                }
            });
        });
    }

    // The control's own refs: `radioRef` (`:85`) and the description layer's forked ref
    // (which carries the consumer's `forwardedRef`, `:229`).
    {
        let radio_element_ref = Rc::clone(&radio_element_ref);
        let rendered_ref = rendered.props.ref_callback.clone();
        let control_element = control_element.clone();
        Effect::new(move |_| {
            let Some(element) = control_element() else {
                return;
            };
            radio_element_ref.set(Some(element.clone()));
            if let Some(callback) = &rendered_ref {
                callback(Some(&element));
            }
        });
    }

    // The hidden input's refs (`:172`), in upstream's merged order: the consumer's
    // `inputRef`, the internal slot, the group's `registerInputRef`, and the Field
    // registration (`:88-94`). The `setFilled` hydration read (`:96-100`) rides here too:
    // it reads `inputRef.current?.checked`, so it can only be evaluated once the input
    // exists.
    {
        let input_element_ref = Rc::clone(&input_element_ref);
        let input_ref = input_ref.clone();
        let register_input_ref = group.as_ref().map(|group| group.register_input_ref.clone());
        let register_field_input = group
            .as_ref()
            .map(|group| group.validation.register_input.clone());
        let radio_element_ref_for_registration = Rc::clone(&radio_element_ref);
        let set_filled = field.set_filled.clone();
        Effect::new(move |_| {
            let Some(input) = input_node.get() else {
                return;
            };
            let html_input: web_sys::HtmlInputElement = input.clone().unchecked_into();
            input_element_ref.set(Some(html_input.clone().unchecked_into::<web_sys::Element>()));

            if let Some(callback) = &input_ref {
                callback(Some(html_input.clone()));
            }

            if let Some(register_input_ref) = &register_input_ref {
                let _ = (register_input_ref)(Some(html_input.clone()));
            }

            if let Some(register_field_input) = &register_field_input {
                let registration = leptos_ui_internals::field_root_context::RegisteredInput {
                    control_ref: Rc::clone(&radio_element_ref_for_registration),
                    // `{ controlRef: radioRef, value: undefined }` (`:91`).
                    value: None,
                };
                let unsubscribe = (register_field_input)(&html_input, registration);
                let cleanup = send_wrapper::SendWrapper::new(move || unsubscribe());
                leptos::prelude::on_cleanup(move || (*cleanup)());
            }

            // `useIsoLayoutEffect(() => { if (inputRef.current?.checked) setFilled(true) },
            // [setFilled])` (`:96-100`).
            if html_input.checked() {
                (set_filled)(true);
            }
        });
    }

    // The hidden input's props (`:170-212`). Every dynamic member rides a signal; the
    // static ones ride the view bindings.
    let input_style_value = style_string(input_style(group_name.is_some()));
    let input_value = input_value_attr(value.as_ref().map(|value| value.as_deref()));
    let input_required = required.then(|| "true".to_string());
    let on_input_change_handler = Rc::clone(&on_input_change);
    let on_input_click_handler = Rc::clone(&on_input_click);
    let on_input_focus_handler = Rc::clone(&on_input_focus);

    // The context provider (`:249`) — upstream wraps the fragment, and the parts read it.
    provide_radio_root_context(context_value);

    let control = if renders_button {
        Either::Left(view! {
            <button node_ref=control_button_node type="button">
                {children_view}
            </button>
        })
    } else {
        Either::Right(view! {
            <span node_ref=control_span_node>
                {children_view}
            </span>
        })
    };

    let input_id_attr = hidden_input_id_value.clone();

    view! {
        <>
            {control}
            <input
                node_ref=input_node
                type="radio"
                id={move || input_id_attr.clone()}
                name={group_name}
                form={group_form}
                tabindex="-1"
                aria-hidden="true"
                style={input_style_value}
                value={input_value}
                disabled={move || disabled.get()}
                required={input_required}
                prop:checked={move || checked.get()}
                readonly={read_only}
                on:change={move |event: web_sys::Event| on_input_change_handler(&event)}
                on:click={move |event: web_sys::MouseEvent| on_input_click_handler(&event)}
                on:focus={move |event: web_sys::FocusEvent| on_input_focus_handler(&event)}
            />
        </>
    }
}

/// Reads an `Rc<Cell<Option<T>>>` slot when `T` is not `Copy` — upstream's ref objects are
/// mutable slots, and `Cell::get` needs `Copy`, so the port swaps the value out and back
/// (the switch/checkbox precedent).
fn cell_read<T: Clone>(cell: &Rc<Cell<Option<T>>>) -> Option<T> {
    let value = cell.replace(None);
    if let Some(inner) = value.clone() {
        cell.set(Some(inner));
    }
    value
}

/// `ElementEventHandler<BaseUIEvent<E>>` → `ElementEventHandler<E>`: the inverse of the
/// internals' `native_to_base_ui`, needed because `useButton`'s external click-like slots
/// are native-typed while the element bags' slots are wrapped (the crate's
/// `ButtonExternalHandlers` contract — the checkbox precedent).
fn base_ui_to_native<E: Clone + 'static>(
    handler: ElementEventHandler<BaseUIEvent<E>>,
) -> ElementEventHandler<E> {
    Rc::new(move |event: &E| handler(&BaseUIEvent::new(event.clone())))
}

/// Keeps the `aria_bool_attr` helper referenced from this module's public surface: the
/// visible control never carries the HTML `disabled` attribute, only `aria-disabled`
/// (behavior.md:45, `RadioRoot.test.tsx:233-243`).
pub fn aria_disabled_attr(disabled: bool) -> Option<String> {
    aria_bool_attr(disabled)
}
