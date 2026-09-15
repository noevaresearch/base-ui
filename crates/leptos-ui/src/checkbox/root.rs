//! `Checkbox.Root` — port of `packages/react/src/checkbox/root/CheckboxRoot.tsx` +
//! `CheckboxRootContext.ts` (the `library: checkbox` TODO item;
//! `specs/library/checkbox/behavior.md`, `specs/library/checkbox/implementation.md`).
//!
//! ## What upstream does (implementation.md's structural reading)
//!
//! Root is a composition, not a machine: one controlled boolean (`useControlled`,
//! `CheckboxRoot.tsx:137-145`), one render-only `indeterminate` flag, and the Field
//! lifecycle state inherited wholesale from `FieldRootContext` (`:74-85`). Every
//! user interaction funnels through the single native `change` event of the hidden
//! `<input type="checkbox">` (`:211-246`) — the visible control re-dispatches its
//! click onto that input instead of mutating state (`:365-378`) — and the veto
//! protocol runs `onCheckedChange` then the group's change callback, checking
//! `details.isCanceled` after each (`:223-235`).
//!
//! ## The port's split
//!
//! - The pure folds, the state record and the state→attribute walk live in
//!   [`crate::checkbox::state`] (host-testable, citing the same upstream lines).
//! - This module is the DOM/event boundary: the three sibling nodes upstream returns
//!   inside the provider (`:399-406`), the context provider (`:400`), the Field /
//!   Form / Labelable / CheckboxGroup integration, and the listeners.
//!
//! Runtime law (the crate's dual-runtime split): leptos 0.7 tracks reactive-graph
//! 0.1 while the internals crate's hooks are typed over 0.2. The port therefore
//! re-homes the state machine on leptos signals (the accordion/`Field.Control`
//! precedent — `useControlled`'s mode fix and guarded setter are re-derived at
//! `:137-145`'s semantics) and bridges the internals' rg-0.2 handles it must read
//! (`CheckboxGroupContext.value`, `useLabelableId`, `useAriaLabelledBy`) through
//! [`mirror_rg_to_leptos`]. The rg-0.2-only engines (`useButton`) are driven with
//! rg-0.2 mirrors of the equal sources.
//!
//! ## Documented adaptations (never silent)
//!
//! 1. **Attributes are applied at mount + on state change** by one writer effect —
//!    the React commit analog — because `view!` has no attribute spread and the
//!    walk's output is dynamic. A rebuild is therefore not needed for a state flip
//!    to reach the DOM.
//! 2. **`suppressHydrationWarning`** (`:405`) has no Leptos analog (CSR-only app);
//!    the concrete `checked` property is re-asserted instead (see 3).
//! 3. **The `checked` *property*** is written on the input at commit and re-asserted
//!    after every change-funnel run — React's controlled-input commit for
//!    `checked` (`:197`), which the layout effect at `:173-182` performs for
//!    `indeterminate`. Without it a cancelled/`readOnly` change would leave the
//!    browser-toggled property out of sync with the state.
//! 4. **The `render` prop** is honored for its element form's tag (`<button />`,
//!    the only form this unit's tests use, `CheckboxRoot.test.tsx:52-66`,
//!    `:1773-1792`) plus the element's own props; a render *function*
//!    (`useRenderElement.tsx:165-170`) is a description-layer feature and is
//!    recorded in `ralph/logs/spec-discrepancies.md` rather than half-ported.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::children::Children;
use leptos::either::Either;
use leptos::prelude::*;
use serde_json::Value;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};

use leptos_ui_internals::floating_ui::element_props::ElementEventHandler;
use leptos_ui_internals::labelable_provider::{
    UseLabelableIdParams, use_aria_labelled_by, use_labelable_context, use_labelable_id,
};
use leptos_ui_internals::merge_props::{PropsSource, merge_props_n};
use leptos_ui_internals::types::BaseUIEvent;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_button::{ButtonExternalHandlers, UseButtonParams, use_button};
use leptos_ui_internals::use_render_element::{
    RenderElementHandlers, RenderElementProps, RenderProp,
};

use crate::checkbox::state::{
    ChangeFunnel, CheckboxChangeEventDetails, CheckboxRootState, PARENT_CHECKBOX,
    checkbox_state_attributes, computed_checked, computed_indeterminate, controlled_checked,
    effective_disabled, effective_name, effective_value, input_id, input_name, input_style,
    input_value, root_id, run_change_funnel, should_render_unchecked_value_input,
    splice_group_value,
};
use crate::checkbox_group::{
    CheckboxGroupChangeEventDetails, ChildPropsSnapshot, ParentPropsSnapshot,
    use_checkbox_group_context,
};
use crate::field::context::{use_field_item_context, use_field_root_context};

/// `useCheckboxRootContext`'s throw (`CheckboxRootContext.ts:9-18`) — the exact
/// upstream message (behavior.md → "Edge cases").
pub const MISSING_ROOT_CONTEXT_MESSAGE: &str = "Base UI: CheckboxRootContext is missing. Checkbox parts must be placed within <Checkbox.Root>.";

/// The context value — upstream's `CheckboxRootContext` is nothing more than the
/// state snapshot (`CheckboxRootContext.ts:5-7`): read-only derived state, no setters
/// or refs, so consumers can react to state but cannot mutate it. The port carries it
/// as live leptos signals (the context bag the parts track); [`Self::snapshot`] is
/// the render-scoped object upstream hands over.
#[derive(Clone)]
pub struct CheckboxRootContextValue {
    /// The computed `checked` (`CheckboxRoot.tsx:281`).
    pub checked: Signal<bool>,
    /// The resolved `disabled` (`:282`).
    pub disabled: Signal<bool>,
    /// `readOnly` (`:283`).
    pub read_only: Signal<bool>,
    /// `required` (`:284`).
    pub required: Signal<bool>,
    /// The computed `indeterminate` (`:285`).
    pub indeterminate: Signal<bool>,
    /// The Field's `touched`.
    pub touched: Signal<bool>,
    /// The Field's `dirty`.
    pub dirty: Signal<bool>,
    /// The Field's `valid` — `None` is upstream's `null`.
    pub valid: Signal<Option<bool>>,
    /// The Field's `filled`.
    pub filled: Signal<bool>,
    /// The Field's `focused`.
    pub focused: Signal<bool>,
}

impl CheckboxRootContextValue {
    /// The untracked snapshot (`CheckboxRoot.tsx:278-288`'s memoized record).
    pub fn snapshot(&self) -> CheckboxRootState {
        CheckboxRootState {
            checked: self.checked.get_untracked(),
            disabled: self.disabled.get_untracked(),
            read_only: self.read_only.get_untracked(),
            required: self.required.get_untracked(),
            indeterminate: self.indeterminate.get_untracked(),
            touched: self.touched.get_untracked(),
            dirty: self.dirty.get_untracked(),
            valid: self.valid.get_untracked(),
            filled: self.filled.get_untracked(),
            focused: self.focused.get_untracked(),
        }
    }

    /// The *tracked* snapshot — the writer effects' read, so a state flip re-runs
    /// the state→attribute walk.
    pub fn tracked_snapshot(&self) -> CheckboxRootState {
        CheckboxRootState {
            checked: self.checked.get(),
            disabled: self.disabled.get(),
            read_only: self.read_only.get(),
            required: self.required.get(),
            indeterminate: self.indeterminate.get(),
            touched: self.touched.get(),
            dirty: self.dirty.get(),
            valid: self.valid.get(),
            filled: self.filled.get(),
            focused: self.focused.get(),
        }
    }
}

/// `useCheckboxRootContext()` (`CheckboxRootContext.ts:30-32`): the part accessor,
/// throwing the upstream message outside a Root.
pub fn use_checkbox_root_context() -> CheckboxRootContextValue {
    try_use_checkbox_root_context().unwrap_or_else(|| panic!("{MISSING_ROOT_CONTEXT_MESSAGE}"))
}

/// The non-panicking probe (the port's host-testable half of the missing-context
/// contract — a wasm panic is an uncatchable trap, the meter/`field` precedent).
pub fn try_use_checkbox_root_context() -> Option<CheckboxRootContextValue> {
    use_context::<CheckboxRootContextValue>()
}

/// The provider seam (`CheckboxRoot.tsx:400`): the port publishes the live bag under
/// the leptos owner so the parts mounted inside the Root body read it.
pub fn provide_checkbox_root_context(value: CheckboxRootContextValue) {
    provide_context(value);
}

/// The consumer's handler members of the `...elementProps` rest
/// (`CheckboxRoot.tsx:70`) plus the two events the element's own machinery owns.
/// Every slot is wrapped-dispatch typed, so a consumer can call
/// `preventBaseUIHandler()` to mute the internal layer (`:329`).
#[derive(Clone, Default)]
pub struct CheckboxRootHandlers {
    /// The consumer's `onClick` (`CheckboxRoot.test.tsx:133-193`; runs before the
    /// internal click handler — the later-bag-first merge rule).
    pub on_click: Option<ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>>>,
    /// The consumer's `onMouseDown`.
    pub on_mouse_down: Option<ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>>>,
    /// The consumer's `onMouseMove`.
    pub on_mouse_move: Option<ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>>>,
    /// The consumer's `onKeyDown` — the Enter-funnel's opt-out (`:815-841`).
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

/// The Root props — upstream's destructured `CheckboxRoot.Props`
/// (`CheckboxRoot.tsx:50-71`) with the documented defaults.
pub struct CheckboxRootViewProps {
    /// `checked` (`:51`) — the controlled prop; `None` while uncontrolled. The
    /// `useControlled` mode fixes from this prop's initial defined-ness (`:137-145`
    /// → `packages/utils/src/useControlled.ts:41`).
    pub checked: Option<bool>,
    /// `defaultChecked` (`:53`, default `false`).
    pub default_checked: Option<bool>,
    /// `onCheckedChange` (`:61`) — `(checked, eventDetails)`, vetoable via
    /// `details.cancel()`.
    pub on_checked_change: Option<Rc<dyn Fn(bool, &CheckboxChangeEventDetails)>>,
    /// `disabled` (`:55`, default `false`).
    pub disabled: bool,
    /// `readOnly` (`:63`, default `false`).
    pub read_only: bool,
    /// `required` (`:65`, default `false`).
    pub required: bool,
    /// `indeterminate` (`:58`, default `false`) — the render-only flag.
    pub indeterminate: bool,
    /// The reactive `indeterminate` source: upstream re-reads the prop on every render,
    /// so a page computing it from its own state (the docs' NESTED parent recipe —
    /// `docs/src/app/(docs)/react/components/checkbox-group/demos/nested/css-modules/index.tsx:35-38`,
    /// where the outer parent mirrors the inner group's partial state) gets a live read.
    /// `None` keeps the one-shot [`CheckboxRootViewProps::indeterminate`] flag, so the
    /// existing callers are unchanged.
    pub indeterminate_source: Option<Signal<bool>>,
    /// `name` (`:60`) — the form field name.
    pub name: Option<String>,
    /// `form` (`:56`) — an external form id.
    pub form: Option<String>,
    /// `id` (`:57`) — the labelable control's id; an empty string falls back to the
    /// scope's control id (`:105-106`).
    pub id: Option<String>,
    /// `value` (`:67`) — falls back to `name` (`:96`).
    pub value: Option<String>,
    /// `uncheckedValue` (`:66`).
    pub unchecked_value: Option<String>,
    /// `parent` (`:62`, default `false`) — a group parent.
    pub parent: bool,
    /// `nativeButton` (`:68`, default `false`).
    pub native_button: bool,
    /// `aria-labelledby` (`:54`) — the explicit override.
    pub aria_labelledby: Option<String>,
    /// `inputRef` (`:59`) — the hidden input's ref callback.
    pub input_ref: Option<Rc<dyn Fn(Option<web_sys::HtmlInputElement>)>>,
    /// `render` (`:64`) — its element form's tag selects the visible element
    /// (`<button />` for `nativeButton`); its props merge after the internal bags.
    pub render: Option<RenderProp>,
    /// `className` (`:52`).
    pub class: Option<String>,
    /// `style` (`:69`) — ordered declarations.
    pub style: Vec<(String, String)>,
    /// The `...elementProps` rest's plain attributes (`:70`), applied last-but-one
    /// (the consumer wins over the internal values).
    pub element_attributes: Vec<(String, String)>,
    /// The rest's handler members.
    pub handlers: CheckboxRootHandlers,
    /// The parts subtree (the Indicator lives here upstream, `:401`).
    pub children: Option<Children>,
}

impl Default for CheckboxRootViewProps {
    fn default() -> Self {
        Self {
            checked: None,
            default_checked: None,
            on_checked_change: None,
            disabled: false,
            read_only: false,
            required: false,
            indeterminate: false,
            indeterminate_source: None,
            name: None,
            form: None,
            id: None,
            value: None,
            unchecked_value: None,
            parent: false,
            native_button: false,
            aria_labelledby: None,
            input_ref: None,
            render: None,
            class: None,
            style: Vec::new(),
            element_attributes: Vec::new(),
            handlers: CheckboxRootHandlers::default(),
            children: None,
        }
    }
}

/// The Root view — upstream's `CheckboxRoot` body (`CheckboxRoot.tsx:46-408`).
///
/// Establishes its own rg-0.2 bridge owner (the `field_root_view` precedent: the
/// parent chain is preserved, so an enclosing `Field.Root`/`CheckboxGroup`
/// provider's rg-0.2 contexts stay visible) and returns the three-sibling fragment
/// upstream returns inside the provider.
pub fn checkbox_root_view(props: CheckboxRootViewProps) -> impl IntoView {
    let bridge_owner = reactive_graph::owner::Owner::new();
    let view = bridge_owner.with(move || checkbox_root_body(props));
    // The bridge owner outlives the subtree (the direction-provider precedent).
    std::mem::forget(bridge_owner);
    view
}

/// The Root body — must be called inside a reactive owner (the `use_button`
/// update-disabled effect registers there).
fn checkbox_root_body(props: CheckboxRootViewProps) -> impl IntoView {
    let CheckboxRootViewProps {
        checked: checked_prop,
        default_checked,
        on_checked_change,
        disabled: disabled_prop,
        read_only,
        required,
        indeterminate: indeterminate_prop,
        indeterminate_source,
        name: name_prop,
        form,
        id: id_prop,
        value: value_prop,
        unchecked_value,
        parent,
        native_button,
        aria_labelledby,
        input_ref,
        render,
        class,
        style,
        element_attributes,
        handlers: consumer_handlers,
        children,
    } = props;

    // `useFormContext()` / `useFieldRootContext()` / `useFieldItemContext()` /
    // `useLabelableContext()` (`:73-87`) — every read falls back to its inert shell
    // outside the matching provider.
    let form_context = leptos_ui_internals::form_context::use_form_context();
    let field = use_field_root_context();
    let field_item = use_field_item_context();
    let labelable = use_labelable_context();

    // `useCheckboxGroupContext()` (`:89`).
    let group_context = use_checkbox_group_context();
    // Whether the checkbox participates in a `CheckboxGroup` at all (`groupContext !==
    // undefined` — the gate every Field-facing branch reads, `:152`, `:179`, `:239`).
    let in_group = group_context.is_some();

    // `disabled = rootDisabled || fieldItemContext.disabled || groupContext?.disabled
    // || disabledProp`, `name = fieldName ?? nameProp`, `value = valueProp ?? name`
    // (`:93-96`). The Field reads are leptos signals, so the composition is live.
    let field_disabled = field.disabled.clone();
    let field_name = field.name.clone();
    let group_disabled = group_context
        .as_ref()
        .map(|group| group.disabled)
        .unwrap_or(false);
    let disabled: Signal<bool> = Signal::derive(move || {
        effective_disabled(
            field_disabled.get(),
            field_item.disabled,
            group_disabled,
            disabled_prop,
        )
    });
    let name: Signal<Option<String>> = {
        let name_prop = name_prop.clone();
        Signal::derive(move || effective_name(field_name.get(), name_prop.clone()))
    };
    let value: Signal<Option<String>> = {
        let value_prop = value_prop.clone();
        let name = name.clone();
        Signal::derive(move || effective_value(value_prop.clone(), name.get()))
    };

    // The group's value, mirrored from the group context's rg-0.2 signal into a
    // leptos one (the tracked read the membership derivation and the parent
    // checkbox's own `checked`/`indeterminate` need — upstream re-renders on every
    // group value change, `CheckboxGroup.tsx:157-165`).
    let group_value: Option<RwSignal<Vec<String>>> = group_context
        .as_ref()
        .map(|group| crate::field::validation_helpers::mirror_rg_to_leptos(&group.value));

    // `parentContext = groupContext?.allValues === undefined ? undefined :
    // groupContext.parent` (`:90-91`).
    let is_grouped_with_parent = group_context
        .as_ref()
        .map(|group| group.all_values.is_some() && group.parent.is_some())
        .unwrap_or(false);
    let all_values: Option<Vec<String>> = group_context
        .as_ref()
        .and_then(|group| group.all_values.clone());
    let parent_engine = if is_grouped_with_parent {
        group_context
            .as_ref()
            .and_then(|group| group.parent.clone())
    } else {
        None
    };
    let _ = &parent_engine;

    // `groupProps` (`:110-124`): the parent factory's `checked`/`indeterminate`, or the
    // child factory's `checked`. Upstream destructures them with the local props as the
    // defaults; the port derives both from the tracked group value with the engine's own
    // formulas (`useCheckboxGroupParent.ts:19-20`) so the attributes stay live.
    let group_checked: Signal<Option<bool>> = {
        let all_values = all_values.clone();
        Signal::derive(move || {
            if !is_grouped_with_parent {
                return None;
            }
            let current = group_value.map(|value| value.get()).unwrap_or_default();
            let all = all_values.clone().unwrap_or_default();
            Some(current.len() == all.len())
        })
    };
    let group_indeterminate: Signal<Option<bool>> = {
        let all_values = all_values.clone();
        Signal::derive(move || {
            if !is_grouped_with_parent {
                return None;
            }
            let current = group_value.map(|value| value.get()).unwrap_or_default();
            let all = all_values.clone().unwrap_or_default();
            Some(current.len() != all.len() && !current.is_empty())
        })
    };

    // The `useControlled` controlled arm (`:137-141`) — captured at body time for the
    // mode fix, then re-derived reactively for the exposed value.
    let controlled_at_body_time = {
        let group_value = group_value.map(|value| value.get_untracked());
        controlled_checked(
            value.get_untracked().as_deref(),
            group_value.as_deref(),
            parent,
            group_checked.get_untracked().or(checked_prop),
        )
    };
    let is_controlled = controlled_at_body_time.is_some();

    // The uncontrolled store (`packages/utils/src/useControlled.ts:42`'s
    // `useState(defaultProp)`).
    let checked_state: RwSignal<bool> = RwSignal::new(default_checked.unwrap_or(false));

    // The exposed value (`useControlled.ts:45`): the controlled value wins while
    // controlled, else the internal state (which is also the fallback when a
    // controlled value later becomes absent).
    let controlled_source: Signal<Option<bool>> = {
        let value = value.clone();
        let group_checked = group_checked.clone();
        let group_value = group_value;
        Signal::derive(move || {
            let current_value = value.get();
            let group_current = group_value.map(|group| group.get());
            controlled_checked(
                current_value.as_deref(),
                group_current.as_deref(),
                parent,
                group_checked.get().or(checked_prop),
            )
        })
    };
    let checked: Signal<bool> = Signal::derive(move || {
        if is_controlled {
            controlled_source
                .get()
                .unwrap_or_else(|| checked_state.get())
        } else {
            checked_state.get()
        }
    });

    // `setCheckedState` (`useControlled.ts:82-89`): the write only reaches the internal
    // state while the hook is uncontrolled.
    let set_checked_state: Rc<dyn Fn(bool)> = Rc::new(move |next: bool| {
        if !is_controlled {
            checked_state.set(next);
        }
    });

    // `computedChecked` / `computedIndeterminate` (`:147-150`).
    let computed_checked_signal: Signal<bool> = Signal::derive(move || {
        computed_checked(
            is_grouped_with_parent,
            group_checked.get().unwrap_or(false),
            checked.get(),
        )
    });
    let computed_indeterminate_signal: Signal<bool> = {
        let indeterminate_source = indeterminate_source;
        Signal::derive(move || {
            computed_indeterminate(
                is_grouped_with_parent,
                group_indeterminate.get().unwrap_or(false),
                indeterminate_source
                    .map(|source| source.get())
                    .unwrap_or(indeterminate_prop),
            )
        })
    };

    // `validation = groupContext?.validation ?? localValidation` (`:135`).
    let validation = group_context
        .as_ref()
        .map(|group| group.validation.clone())
        .unwrap_or_else(|| field.validation.clone());
    let validation_mode = field.validation_mode;

    // The state bag (`:278-288`): the Field lifecycle spread plus the checkbox's own
    // five members — the context value AND the mapping's input.
    let context_value = CheckboxRootContextValue {
        checked: computed_checked_signal,
        disabled: disabled.clone(),
        read_only: Signal::derive(move || read_only),
        required: Signal::derive(move || required),
        indeterminate: computed_indeterminate_signal,
        touched: field.state.touched.clone(),
        dirty: field.state.dirty.clone(),
        valid: field.state.valid.clone(),
        filled: field.state.filled.clone(),
        focused: field.state.focused.clone(),
    };
    provide_checkbox_root_context(context_value.clone());

    // The parts subtree (`:401`) — built inside the bridge window so its own internals
    // reads resolve against this Root's scope.
    let children_view = children.map(|children| children());

    // `const id = useBaseUiId()` (`:98`) — instance-local and never re-derived, so it is
    // read once (a hydration-time id handoff is out of scope for the CSR-only app).
    let generated_id = reactive_graph::traits::GetUntracked::get_untracked(&use_base_ui_id(
        reactive_graph::signal::RwSignal::new_local(None::<String>),
    ));

    // `ownsControlId = groupContext?.registerControlId !== registerControlId` (`:103`):
    // the group is the field's control, so checkboxes sharing its labelable scope must
    // not claim the field's control id.
    let owns_control_id = match &group_context {
        Some(group) => !Rc::ptr_eq(&group.register_control_id, &labelable.register_control_id),
        None => true,
    };
    // `useLabelableId({ id: idProp || undefined, enabled: ownsControlId })` (`:106`) —
    // the `|| undefined` makes an empty `id=""` fall back to the scope's control id.
    let control_id_rg = use_labelable_id(UseLabelableIdParams {
        id: id_prop.filter(|id| !id.is_empty()),
        enabled: owns_control_id,
    });
    let control_id = crate::field::validation_helpers::mirror_rg_to_leptos(&control_id_rg);

    // `rootId = nativeButton ? controlId : id` (`:108`) — in the default mode the
    // labelable id lives on the hidden input and the visible element carries the
    // internal instance id.
    let root_id_value = {
        let control = control_id.get_untracked();
        root_id(native_button, &control, &generated_id)
    };
    // The attribute closure takes its own copy so the binding stays live for the
    // child-id registration below (`:391-397`, the id the parent aggregates).
    let root_id_for_attr = root_id_value.clone();
    let control_id_attr = move || {
        if native_button {
            control_id.get()
        } else {
            root_id_for_attr.clone()
        }
    };

    // The element slots the machines read (`:128`, `:156`): leptos has no ref fork, so
    // the nodes arrive through `NodeRef`s and a mount effect resyncs the `Rc<Cell>`
    // slots (the `Field.Control` precedent).
    let control_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    let input_element_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    let control_span_node: NodeRef<leptos::html::Span> = NodeRef::new();
    let control_button_node: NodeRef<leptos::html::Button> = NodeRef::new();
    let input_node: NodeRef<leptos::html::Input> = NodeRef::new();

    // The visible element's tag: the default `span`, or a real `<button>` when
    // `nativeButton` (or the render element says so) — `:197`, `:292`.
    let render_tag = match &render {
        Some(RenderProp::Element { tag, .. }) => Some(tag.clone()),
        _ => None,
    };
    let renders_button = native_button || render_tag.as_deref() == Some("button");

    // `useButton({ disabled, native: nativeButton })` (`:130-133`). The hook reads its
    // `disabled` source through rg-0.2, so the (mostly static) computed flag rides a
    // mirror written by a leptos effect (the direction-provider bridge pattern).
    let disabled_rg = reactive_graph::signal::RwSignal::new(disabled.get_untracked());
    {
        let disabled_rg = disabled_rg.clone();
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

    // `useAriaLabelledBy(ariaLabelledByProp, labelId, inputRef, !nativeButton,
    // controlId)` (`:165-171`).
    let aria_labelledby_rg = use_aria_labelled_by(
        aria_labelledby.clone(),
        labelable.label_id.clone(),
        Rc::clone(&input_element_ref),
        !native_button,
        Some(control_id.get_untracked()),
    );
    let aria_labelledby_value =
        crate::field::validation_helpers::mirror_rg_to_leptos(&aria_labelledby_rg);

    // `useRegisterFieldControl(controlRef, id, checked, undefined, !groupContext &&
    // !disabled, nameProp)` (`:152`) — re-homed on the leptos field context (the
    // `Field.Control` registration effect's mechanics: one source token per instance,
    // in-place re-registration, unregistration on unmount).
    {
        let register = field.register_field_control.clone();
        let source = leptos_ui_internals::labelable_provider::ControlIdSource::new();
        let control_ref = Rc::clone(&control_ref);
        let id_for_registration = generated_id.clone();
        let name_prop = name_prop.clone();
        let in_group = in_group;
        Effect::new(move |_| {
            if in_group || disabled.get() {
                register(source.clone(), None);
                return;
            }
            let registration =
                leptos_ui_internals::field_register_control::FieldControlRegistration {
                    control_ref: Rc::clone(&control_ref),
                    id: Some(id_for_registration.clone()),
                    name: name_prop.clone(),
                    get_value: None,
                    value: Some(Value::Bool(checked.get())),
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

    // `useIsoLayoutEffect #1` (`:173-182`): re-assert the input's `indeterminate` (a
    // native click resets it) and push `setFilled(checked)` into the Field when not
    // grouped. The `checked` property re-assert (adaptation 3) rides the same slot.
    {
        let set_filled = field.set_filled.clone();
        let in_group = in_group;
        Effect::new(move |_| {
            let input = input_node.get();
            let _ = input_element_ref;
            if let Some(input) = &input {
                let input: &web_sys::HtmlInputElement = input.unchecked_ref();
                let _ = input.set_indeterminate(computed_indeterminate_signal.get());
            }
            if !in_group {
                set_filled(checked.get());
            }
        });
    }

    // `useValueChanged(checked, …)` (`:184-193`): only on an actual checked change (not
    // mount) — `clearErrors(name)`, `setDirty(checked !== initialValue)`, and one
    // validation run. Both effects early-return inside groups (`:185-187`).
    {
        let in_group = in_group;
        if !in_group {
            let clear_errors = form_context.clear_errors.clone();
            let set_dirty = field.set_dirty.clone();
            let validation = validation.clone();
            let validity_data = field.validity_data.clone();
            let name = name.clone();
            // Upstream's `useValueChanged` captures the previous value in a ref; the port
            // rides an explicit signal (the leptos effect's own argument cannot carry a
            // non-`Send` previous value through `EffectFunction`).
            let previous_checked: RwSignal<Option<bool>> = RwSignal::new(None);
            Effect::new(move |_| {
                let current = checked.get();
                let previous = previous_checked.get_untracked();
                if previous == Some(current) {
                    return;
                }
                previous_checked.set(Some(current));

                // The mount run only seeds the ref (`:184` runs on change only).
                if previous.is_none() {
                    return;
                }

                if let Some(field_name) = name.get_untracked() {
                    if !field_name.is_empty() {
                        clear_errors(Some(&field_name));
                    }
                }

                set_dirty(crate::checkbox::state::checked_is_dirty(
                    current,
                    &validity_data.get_untracked().initial_value,
                ));

                (validation.change)(Some(Value::Bool(current)), false);
            });
        }
    }

    // `React.useEffect` (`:265-276`): publish each child's `disabled` into the parent's
    // `disabledStatesRef`, keyed by `value`, with the unmount cleanup.
    if let (Some(engine), true) = (parent_engine.as_ref(), is_grouped_with_parent) {
        let engine = engine.clone();
        let value_now = value.get_untracked();
        if let Some(child_value) = value_now {
            let disabled_states = Rc::clone(&engine.disabled_states);
            Effect::new(move |_| {
                let current = disabled.get();
                disabled_states
                    .borrow_mut()
                    .insert(child_value.clone(), current);
            });
            let disabled_states = Rc::clone(&engine.disabled_states);
            let child_value = value.get_untracked().unwrap_or_default();
            let cleanup = send_wrapper::SendWrapper::new(move || {
                disabled_states.borrow_mut().remove(&child_value);
            });
            reactive_graph::owner::on_cleanup(move || (*cleanup)());
        }
    }

    // `useIsoLayoutEffect #2` (`:391-397`): register a child checkbox's rendered id with
    // a parent checkbox — the parent aggregates them into `aria-controls`.
    let rendered_id = root_id_value.clone();
    if let (Some(engine), true, Some(child_value)) =
        (parent_engine.as_ref(), !parent, value.get_untracked())
    {
        if !is_grouped_with_parent {
            let register_child_id = Rc::clone(&engine.register_child_id);
            let source = register_child_id(&child_value, &rendered_id);
            let cleanup = send_wrapper::SendWrapper::new(move || source());
            reactive_graph::owner::on_cleanup(move || (*cleanup)());
        }
    }

    // The parent's `aria-controls` (`useCheckboxGroupParent.ts:62-65`): every registered
    // child id per value in `allValues` order, space-joined. Derived from the tracked
    // registry so the attribute lands once the children register.
    let aria_controls = {
        let all_values = all_values.clone().unwrap_or_default();
        let registry = group_context
            .as_ref()
            .and_then(|group| group.parent.as_ref())
            .map(|engine| engine.child_ids.clone());
        // The derivation runs in rg-0.2 (the registry's own runtime — a leptos closure
        // reading an rg-0.2 signal is the dual-runtime mismatch), and its `String`
        // result crosses into leptos through the mirror.
        let derived_rg = reactive_graph::wrappers::read::Signal::derive_local(move || {
            let Some(registry) = registry else {
                return None;
            };
            let current = reactive_graph::traits::Get::get(&registry);
            let children = current.borrow();
            let ids: Vec<String> = all_values
                .iter()
                .filter_map(|value| children.get(value).cloned())
                .flatten()
                .collect();
            if ids.is_empty() {
                None
            } else {
                Some(ids.join(" "))
            }
        });
        crate::field::validation_helpers::mirror_rg_to_leptos(&derived_rg)
    };

    // The group callbacks, resolved at event time (upstream resolves them per render; the
    // port's factories snapshot per call, so the lazy fetch is the same object).
    let is_parent = parent;
    let value_now = value.get_untracked();
    let fetch_group_on_change: Rc<
        dyn Fn() -> Option<Rc<dyn Fn(bool, &CheckboxGroupChangeEventDetails)>>,
    > = {
        let parent_engine = parent_engine.clone();
        let is_grouped_with_parent = is_grouped_with_parent;
        let value_now = value_now.clone();
        Rc::new(move || {
            if !is_grouped_with_parent {
                return None;
            }
            let engine = parent_engine.as_ref()?;
            if is_parent {
                let snapshot: ParentPropsSnapshot = (engine.get_parent_props)();
                Some(snapshot.on_checked_change)
            } else {
                let child_value = value_now.clone()?;
                let snapshot: ChildPropsSnapshot = (engine.get_child_props)(&child_value);
                Some(snapshot.on_checked_change)
            }
        })
    };

    // The `change` funnel (`:211-246`) — the single transition point.
    let on_input_change: Rc<dyn Fn(&web_sys::Event)> = {
        let on_checked_change = on_checked_change.clone();
        let fetch_group_on_change = Rc::clone(&fetch_group_on_change);
        let set_checked_state = set_checked_state.clone();
        let group_context = group_context.clone();
        let value_now = value_now.clone();
        let is_grouped_with_parent = is_grouped_with_parent;
        let computed_checked_for_sync = computed_checked_signal;
        let computed_indeterminate_for_sync = computed_indeterminate_signal;
        Rc::new(move |event: &web_sys::Event| {
            let Some(target) = event
                .current_target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
            else {
                return;
            };

            let details = CheckboxChangeEventDetails::new(
                leptos_ui_internals::floating_ui::reasons::NONE,
                event.clone(),
                None,
                (),
            );
            let group_details = CheckboxGroupChangeEventDetails::new(
                leptos_ui_internals::floating_ui::reasons::NONE,
                (),
                None,
                (),
            );

            let next_checked = target.checked();
            let group_on_change = fetch_group_on_change();
            let funnel = run_change_funnel(
                next_checked,
                read_only,
                event.default_prevented(),
                on_checked_change.as_deref(),
                group_on_change.as_deref(),
                &details,
                &group_details,
            );

            if funnel == ChangeFunnel::ReadOnly {
                event.prevent_default();
            }

            if funnel == ChangeFunnel::Commit {
                set_checked_state(next_checked);

                // The group-value splice (`:239-245`): only a grouped child that is not a
                // group parent and not grouped under a parent checkbox.
                if let (Some(value), Some(group)) = (&value_now, &group_context) {
                    if !is_parent && !is_grouped_with_parent {
                        let next_group_value = splice_group_value(
                            &reactive_graph::traits::GetUntracked::get_untracked(&group.value),
                            value,
                            next_checked,
                        );
                        (group.set_value)(next_group_value, &group_details);
                    }
                }
            }

            // The React controlled-input commit for the `checked` property (adaptation
            // 3): the browser already flipped it; the state is the source of truth.
            let _ = target.set_checked(computed_checked_for_sync.get_untracked());
            let _ = target.set_indeterminate(computed_indeterminate_for_sync.get_untracked());
        })
    };

    // The element's own handler bag (`:296-379`) — the base bag of the props array.
    let base_bag = {
        let set_focused = field.set_focused.clone();
        let set_touched = field.set_touched.clone();
        let validation = validation.clone();
        let input_node_for_blur = input_node;
        let group_value_for_blur = group_value;
        let input_for_click = input_node;
        let on_input_change = Rc::clone(&on_input_change);
        let aria_checked_for_mixed = computed_indeterminate_signal;
        let _ = aria_checked_for_mixed;

        // `onFocus` (`:304-308`): report focus to the Field unless disabled.
        let on_focus: ElementEventHandler<BaseUIEvent<web_sys::FocusEvent>> = {
            let set_focused = set_focused.clone();
            Rc::new(move |_event: &BaseUIEvent<web_sys::FocusEvent>| {
                if !disabled.get_untracked() {
                    set_focused(true);
                }
            })
        };

        // `onBlur` (`:309-321`): touched + focused, then the `onBlur` validation commit.
        let on_blur: ElementEventHandler<BaseUIEvent<web_sys::FocusEvent>> = {
            let validation = validation.clone();
            // Read at event time only to branch the validation value's source; the copy
            // keeps the enclosing binding live for the hidden input's registration below
            // (`:164`) — the `on_input_change` funnel's convention.
            let group_context = group_context.clone();
            Rc::new(move |_event: &BaseUIEvent<web_sys::FocusEvent>| {
                let Some(input) = input_node_for_blur.get() else {
                    return;
                };
                let input: &web_sys::HtmlInputElement = input.unchecked_ref();
                set_touched(true);
                set_focused(false);
                if validation_mode == leptos_ui_internals::form_context::FormValidationMode::OnBlur
                {
                    let value = match (group_value_for_blur, group_context.as_ref()) {
                        (Some(group), Some(_)) => {
                            serde_json::to_value(group.get_untracked()).unwrap_or(Value::Null)
                        }
                        _ => Value::Bool(input.checked()),
                    };
                    (validation.commit)(value);
                }
            })
        };

        // `onKeyDown` (`:322-364`): Enter never toggles; it submits the owning form via
        // the default submitter unless somebody prevented the default during
        // propagation. `event.preventBaseUIHandler()` (`:329`) mutes the shared button
        // layer's Enter activation — the reason this handler is a `BaseUIEvent`.
        //
        // The two-phase dance (`:338-363`): upstream swaps `preventDefault` on BOTH the
        // synthetic and the native event for recorders, calls the ORIGINAL native
        // `preventDefault` (cancelling native activation while leaving the observable
        // `defaultPrevented` false during propagation), then restores the originals one
        // microtask later and clicks the form's default submitter unless somebody called
        // `preventDefault()` while the event was propagating.
        //
        // Rust adaptation: there is no synthetic event layer, so the port shadows BOTH
        // members on the native event object for the propagation window —
        // `preventDefault` (the recorder) and the `defaultPrevented` getter (the
        // recorder's flag) — which is what makes behavior.md's "at the time the ancestor
        // handler runs, `event.defaultPrevented` is still `false`" hold
        // (`CheckboxRoot.test.tsx:849-852`, `:871`).
        let on_key_down: ElementEventHandler<BaseUIEvent<web_sys::KeyboardEvent>> =
            Rc::new(move |event: &BaseUIEvent<web_sys::KeyboardEvent>| {
                if event.inner().key() != "Enter" {
                    return;
                }

                event.prevent_base_ui_handler();

                if event.inner().default_prevented() {
                    return;
                }

                let form_to_submit = input_node.get().map(|input| input.form()).unwrap_or(None);
                let native_event = event.inner().clone();
                let event_object: &js_sys::Object = native_event.as_ref();
                let called_after_propagation = Rc::new(Cell::new(false));

                // The original (prototype-resolved) `preventDefault`.
                let original_prevent_default =
                    js_sys::Reflect::get(event_object, &JsValue::from_str("preventDefault"))
                        .ok()
                        .and_then(|value| value.dyn_into::<js_sys::Function>().ok());
                let original_default_prevented =
                    js_sys::Reflect::get(event_object, &JsValue::from_str("defaultPrevented"))
                        .ok()
                        .and_then(|value| value.dyn_into::<js_sys::Function>().ok());

                // Failure to install a piece of the dance still leaves the native
                // cancel + the one-shot submitter click (the load-bearing half).
                let mut installed: Vec<(
                    &'static str,
                    Option<js_sys::Function>,
                    Closure<dyn Fn() -> bool>,
                )> = Vec::new();

                if let Some(original) = original_prevent_default.clone() {
                    let called = Rc::clone(&called_after_propagation);
                    let this = native_event.clone();
                    let closure = Closure::<dyn Fn() -> bool>::new(move || {
                        called.set(true);
                        let _ = original.call0(&this);
                        true
                    });
                    let recorder: js_sys::Function = closure
                        .as_ref()
                        .clone()
                        .unchecked_into::<js_sys::Function>();
                    let _ = js_sys::Reflect::set(
                        event_object,
                        &JsValue::from_str("preventDefault"),
                        &recorder,
                    );
                    installed.push(("preventDefault", None, closure));
                }

                if let Some(original) = original_default_prevented.clone() {
                    let called = Rc::clone(&called_after_propagation);
                    let closure = Closure::<dyn Fn() -> bool>::new(move || called.get());
                    let recorder: js_sys::Function = closure
                        .as_ref()
                        .clone()
                        .unchecked_into::<js_sys::Function>();
                    let attributes = js_sys::Object::new();
                    let _ = js_sys::Reflect::set(&attributes, &JsValue::from_str("get"), &recorder);
                    let _ = js_sys::Reflect::set(
                        &attributes,
                        &JsValue::from_str("configurable"),
                        &JsValue::TRUE,
                    );
                    let _ = js_sys::Object::define_property(
                        event_object,
                        &JsValue::from_str("defaultPrevented"),
                        &attributes,
                    );
                    installed.push(("defaultPrevented", Some(original), closure));
                }

                // Cancel native activation now (`:354`), deliberately through the
                // ORIGINAL function so the observable flag stays false for ancestors.
                if let Some(original) = &original_prevent_default {
                    let _ = original.call0(&native_event);
                } else {
                    native_event.prevent_default();
                }

                // Restore the originals and click the form's default submitter unless an
                // ancestor opted out (`:356-363`) — one microtask later. The `installed`
                // vector owns the recorder closures until then.
                let installed = Rc::new(RefCell::new(installed));
                let installed_for_restore = Rc::clone(&installed);
                let event_object = event_object.clone();
                let called = Rc::clone(&called_after_propagation);
                queue_microtask(move || {
                    for (name, restore, closure) in installed_for_restore.borrow_mut().drain(..) {
                        match restore {
                            Some(original) => {
                                let _ = js_sys::Reflect::set(
                                    &event_object,
                                    &JsValue::from_str(name),
                                    &original,
                                );
                            }
                            None => {
                                let _ = js_sys::Reflect::set(
                                    &event_object,
                                    &JsValue::from_str(name),
                                    JsValue::UNDEFINED.as_ref(),
                                );
                            }
                        }
                        drop(closure);
                    }
                    if !called.get() {
                        if let Some(submitter) =
                            leptos_ui_utils::get_default_form_submitter::get_default_form_submitter(
                                form_to_submit.as_ref(),
                            )
                        {
                            let _ = submitter.as_html_element().click();
                        }
                    }
                });
            });

        // `onClick` (`:365-378`): bail for readOnly/disabled, cancel the original click
        // (so no second native activation), and re-dispatch it onto the hidden input.
        let on_click: ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>> =
            Rc::new(move |event: &BaseUIEvent<web_sys::MouseEvent>| {
                if read_only || disabled.get_untracked() {
                    return;
                }

                event.inner().prevent_default();

                let Some(input) = input_node.get() else {
                    return;
                };
                let input: &web_sys::Element = input.unchecked_ref();
                leptos_ui_internals::dispatch_click_with_modifiers::dispatch_click_with_modifiers(
                    input,
                    event.inner(),
                    0,
                );
            });

        let _ = &on_input_change;

        RenderElementProps {
            handlers: RenderElementHandlers {
                on_focus: Some(on_focus),
                on_blur: Some(on_blur),
                on_key_down: Some(on_key_down),
                on_click: Some(on_click),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        }
    };

    // The `...elementProps` rest bag (`:380`) — the consumer's handlers and attributes.
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
                        Rc::new(move || Some(value.clone()))
                            as leptos_ui_internals::floating_ui::element_props::ElementAttributeFn,
                    )
                })
                .collect(),
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    };

    // `otherGroupProps` (`:381`) — the parent factory's `aria-controls` member (the
    // factory's `onCheckedChange` is an internal callback, not a DOM prop: upstream's
    // spread drops it onto the span where nothing reads it).
    let group_bag = RenderElementProps {
        handlers: RenderElementHandlers {
            attributes: if is_grouped_with_parent && parent {
                vec![(
                    "aria-controls".to_string(),
                    Rc::new(move || aria_controls.get())
                        as leptos_ui_internals::floating_ui::element_props::ElementAttributeFn,
                )]
            } else {
                Vec::new()
            },
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    };

    // `getButtonProps` (`:382`) — the props-getter semantics: resolved against the
    // merged-so-far props, whose five handler members it consumes (upstream destructures
    // them out of `otherExternalProps`), and whose remaining members are merged LAST (so
    // the earlier bags' plain attributes win — the `role="checkbox"` over the button
    // layer's `role="button"`).
    let mut merge_bags = vec![
        PropsSource::Static(base_bag),
        PropsSource::Static(element_bag),
        PropsSource::Static(group_bag),
    ];
    if let Some(RenderProp::Element { props, .. }) = render.clone() {
        merge_bags.push(PropsSource::Static(props));
    }
    let merged = merge_props_n(merge_bags);
    let is_parent_for_attrs = parent;
    let _ = is_parent_for_attrs;

    let button_props = (button.get_button_props)(ButtonExternalHandlers {
        // The click-like external slots are native-typed in the hook's contract (the
        // hook wraps them into the prevention protocol itself); the key slots keep the
        // `BaseUIEvent` wrapper, which is what lets the checkbox's Enter handler mute
        // the button layer via `preventBaseUIHandler()`.
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

    // The final bags: the button layer's five composed handlers (each already wrapping
    // the merged-so-far handler), the pass-through slots from the merged bag, the button
    // layer's attributes with the merged-so-far attributes winning, and the merged
    // class/style.
    let final_handlers = RenderElementHandlers {
        on_click: button_props
            .handlers
            .on_click
            .clone()
            .map(leptos_ui_internals::use_render_element::native_to_base_ui),
        on_mouse_down: button_props
            .handlers
            .on_mouse_down
            .clone()
            .map(leptos_ui_internals::use_render_element::native_to_base_ui),
        on_key_down: button_props.handlers.on_key_down.clone(),
        on_key_up: button_props.handlers.on_key_up.clone(),
        on_pointer_down: button_props
            .handlers
            .on_pointer_down
            .clone()
            .map(leptos_ui_internals::use_render_element::native_to_base_ui),
        on_focus: merged.handlers.on_focus.clone(),
        on_blur: merged.handlers.on_blur.clone(),
        on_context_menu: merged.handlers.on_context_menu.clone(),
        on_mouse_move: merged.handlers.on_mouse_move.clone(),
        attributes: {
            let mut attributes = button_props.attributes.clone();
            for (name, value) in &merged.handlers.attributes {
                match attributes.iter_mut().find(|(existing, _)| existing == name) {
                    Some(slot) => slot.1 = Rc::clone(value),
                    None => attributes.push((name.clone(), Rc::clone(value))),
                }
            }
            attributes
        },
    };
    let final_bag = RenderElementProps {
        handlers: final_handlers,
        class: merged.class.clone(),
        style: merged.style.clone(),
        inner_html: merged.inner_html.clone(),
        ref_callback: merged.ref_callback.clone(),
    };

    // The whole element's attributes are the bags' merged output plus the state walk's
    // `data-*` members, applied by one writer effect (adaptation 1).
    let consumer_attribute_names: Vec<String> = element_attributes
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    let managed_names: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let class_value = class.clone().or(final_bag.class.clone());
    let style_value = style.clone();
    let bag_attributes: Vec<(
        String,
        leptos_ui_internals::floating_ui::element_props::ElementAttributeFn,
    )> = final_bag.handlers.attributes.clone();
    let state_for_writer = context_value.clone();
    {
        let managed_names = Rc::clone(&managed_names);
        let consumer_attribute_names = consumer_attribute_names.clone();
        let bag_attributes = bag_attributes.clone();
        let class_value = class_value.clone();
        let style_value = style_value.clone();
        let state_for_writer = state_for_writer.clone();
        let control_ref = Rc::clone(&control_ref);
        let button_ref = button.button_ref.clone();
        let control_element = move || -> Option<web_sys::Element> {
            control_span_node
                .get()
                .map(|span| span.unchecked_into::<web_sys::Element>())
                .or_else(|| {
                    control_button_node
                        .get()
                        .map(|button| button.unchecked_into::<web_sys::Element>())
                })
        };
        Effect::new(move |_| {
            let Some(element) = control_element() else {
                return;
            };
            let state = state_for_writer.tracked_snapshot();
            let walk = checkbox_state_attributes(&state);

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
            attributes.push(("id".to_string(), Some(control_id_attr())));
            attributes.push((
                "role".to_string(),
                Some(consumer_role(
                    &consumer_attribute_names,
                    &element_attributes,
                )),
            ));
            attributes.push(("aria-checked".to_string(), Some(state.aria_checked())));
            attributes.push((
                "aria-readonly".to_string(),
                state.read_only.then(|| "true".to_string()),
            ));
            attributes.push((
                "aria-required".to_string(),
                state.required.then(|| "true".to_string()),
            ));
            attributes.push(("aria-labelledby".to_string(), aria_labelledby_value.get()));
            attributes.push((PARENT_CHECKBOX.to_string(), parent.then(String::new)));

            for name in crate::checkbox::state::MANAGED_STATE_ATTRIBUTES {
                attributes.push((name.to_string(), walk.get(name).cloned()));
            }

            // The rest bag (the consumer's plain attributes) wins over the internal
            // members, then the button layer's attributes fill whatever neither wrote.
            for (name, value) in &element_attributes {
                attributes.push((name.clone(), Some(value.clone())));
            }
            for (name, value) in &bag_attributes {
                if name == "role" || consumer_attribute_names.iter().any(|seen| seen == name) {
                    continue;
                }
                attributes.push((name.clone(), value()));
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

    // The ref fork (`:294`): the button ref, the control ref and the hidden input's
    // validation registration, all resynced at mount (the `Field.Control` precedent).
    {
        let button_ref = button.button_ref.clone();
        let control_ref = Rc::clone(&control_ref);
        let control_element = move || -> Option<web_sys::Element> {
            control_span_node
                .get()
                .map(|span| span.unchecked_into::<web_sys::Element>())
                .or_else(|| {
                    control_button_node
                        .get()
                        .map(|button| button.unchecked_into::<web_sys::Element>())
                })
        };
        Effect::new(move |_| {
            let Some(element) = control_element() else {
                return;
            };
            control_ref.set(Some(element.clone()));
            let as_html: Option<web_sys::HtmlElement> =
                element.clone().dyn_into::<web_sys::HtmlElement>().ok();
            button_ref(as_html);
        });
    }

    // The hidden input's refs (`:164`): the consumer's `inputRef`, the internal slot, and
    // — for non-parent checkboxes — the validation-input registration with the group
    // value as its registration value (`:158`).
    {
        let input_element_ref = Rc::clone(&input_element_ref);
        let registered_input_value = group_context.as_ref().and_then(|_| value.get_untracked());
        let register_input = validation.register_input.clone();
        let control_ref = Rc::clone(&control_ref);
        let skip_registration = parent;
        Effect::new(move |_| {
            let Some(input) = input_node.get() else {
                return;
            };
            let input: &web_sys::HtmlInputElement = input.unchecked_ref();
            input_element_ref.set(Some(input.clone().unchecked_into::<web_sys::Element>()));
            if let Some(callback) = &input_ref {
                callback(Some(input.clone()));
            }
            if skip_registration {
                return;
            }
            let registration = crate::field::validation::RegisteredInput {
                control_ref: Rc::clone(&control_ref),
                value: registered_input_value.clone(),
            };
            let unsubscribe = register_input(input, registration);
            let cleanup = send_wrapper::SendWrapper::new(move || unsubscribe());
            // Leptos-side for the same reason as the handler-attach effect above: this is
            // a leptos (rg-0.1) effect body, where rg-0.2's `owner::on_cleanup` no-ops and
            // drops its closure — which silently discarded the field-input unregistration
            // (it would never run, on any re-run or unmount).
            leptos::prelude::on_cleanup(move || (*cleanup)());
        });
    }

    // The handlers' attach (`:292-387`'s JSX spread): one registration point, in the
    // merge's own order (later bags' handlers first — the `mergeProps` rule).
    {
        let final_bag = final_bag.clone();
        let control_element = move || -> Option<web_sys::Element> {
            control_span_node
                .get()
                .map(|span| span.unchecked_into::<web_sys::Element>())
                .or_else(|| {
                    control_button_node
                        .get()
                        .map(|button| button.unchecked_into::<web_sys::Element>())
                })
        };
        Effect::new(move |_| {
            let Some(element) = control_element() else {
                return;
            };
            let target: &web_sys::EventTarget = element.unchecked_ref();
            let cleanup = final_bag.handlers.attach_to(target);
            // The element's listeners live for its lifetime (they go with the node);
            // the `FnOnce` is taken out of the slot by the cleanup run.
            let cleanup: Box<dyn FnOnce()> = Box::new(move || {
                if let Some(cleanup) = cleanup {
                    cleanup();
                }
            });
            let cleanup = send_wrapper::SendWrapper::new(RefCell::new(Some(cleanup)));
            // LEPTOS-side teardown, deliberately: this effect body runs on the leptos
            // (rg-0.1) runtime, where no rg-0.2 owner is current — `reactive_graph::owner
            // ::on_cleanup` (rg-0.2) no-ops with no current owner and SILENTLY DROPS its
            // closure, and dropping the slot above drops the `EventListenerUnsubscribe`
            // handles, whose own `Drop` unsubscribes — so every listener this effect
            // attaches was torn back off before the first click could reach it. A leptos
            // `on_cleanup` inside an `Effect` registers on the per-run owner
            // (`with_cleanup` runs the owner's cleanups at the start of each run), which
            // is exactly the per-run re-attach contract described above. Same law as
            // `avatar/mod.rs:69-71` and the `context_menu/trigger.rs:309` teardown.
            leptos::prelude::on_cleanup(move || {
                if let Some(cleanup) = cleanup.borrow_mut().take() {
                    cleanup();
                }
            });
        });
    }

    // The hidden inputs' `aria-describedby` + validation overlay (`:261-262`): the
    // labelable description merge and `validation.getValidationProps(disabled)`.
    let description_attributes: Vec<(
        String,
        leptos_ui_internals::floating_ui::element_props::ElementAttributeFn,
    )> = {
        let mut attributes = Vec::new();
        (labelable.get_description_props)(&mut attributes);
        let mut overlay = Vec::new();
        (validation.get_validation_props)(disabled.get_untracked(), &mut overlay);
        attributes.extend(overlay);
        attributes
    };

    // The hidden native input's props (`:195-263`).
    let input_name_value = input_name(parent, name.get_untracked().as_deref());
    let input_id_signal = {
        let control_id = control_id;
        Signal::derive(move || input_id(native_button, &control_id.get()))
    };
    let input_value_signal = {
        let value_prop = value_prop.clone();
        let in_group = in_group;
        Signal::derive(move || input_value(in_group, checked.get(), value_prop.as_deref()))
    };
    let input_style_value = style_string(input_style(name.get_untracked().as_deref()));
    let description_attributes_for_input = description_attributes.clone();
    let input_managed_names: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    {
        let managed_names = Rc::clone(&input_managed_names);
        let computed_indeterminate = computed_indeterminate_signal;
        let checked_signal = computed_checked_signal;
        let attrs = description_attributes_for_input.clone();
        Effect::new(move |_| {
            let Some(input) = input_node.get() else {
                return;
            };
            let input: &web_sys::HtmlInputElement = input.unchecked_ref();
            // Adaptation 3: the `checked`/`indeterminate` properties.
            let _ = input.set_checked(checked_signal.get());
            let _ = input.set_indeterminate(computed_indeterminate.get());

            let mut attributes: Vec<(String, Option<String>)> = Vec::new();
            for (name, value) in &attrs {
                attributes.push((name.clone(), value()));
            }
            let previous = managed_names.borrow().clone();
            let mut written: Vec<String> = Vec::new();
            for (name, value) in attributes {
                if let Some(value) = value {
                    let _ = input.set_attribute(&name, &value);
                    written.push(name);
                }
            }
            for name in previous {
                if !written.contains(&name) {
                    let _ = input.remove_attribute(&name);
                }
            }
            *managed_names.borrow_mut() = written;
        });
    }

    // The `form`/`name`/`id`/`disabled`/`required`/`type`/`tabindex`/`aria-hidden`/
    // `style` members ride the view bindings; the dynamic members ride signals.
    let disabled_attr = {
        let disabled = disabled.clone();
        Signal::derive(move || disabled.get().then(|| "true".to_string()))
    };
    let required_attr = required.then(|| "true".to_string());
    let name_attr = input_name_value.clone();
    let form_attr = form.clone();
    let on_input_change_handler = {
        let on_input_change = Rc::clone(&on_input_change);
        move |event: web_sys::Event| on_input_change(&event)
    };
    let on_input_click = move |event: BaseUIEvent<web_sys::MouseEvent>| {
        // `:247-251`: the re-dispatched click is an implementation detail and must not
        // reach ancestors, which already receive the original click.
        event.inner().stop_propagation();
    };
    let on_input_focus = {
        let control_element = move || -> Option<web_sys::Element> {
            control_span_node
                .get()
                .map(|span| span.unchecked_into::<web_sys::Element>())
                .or_else(|| {
                    control_button_node
                        .get()
                        .map(|button| button.unchecked_into::<web_sys::Element>())
                })
        };
        move |_event: web_sys::FocusEvent| {
            if let Some(control) = control_element() {
                if let Ok(control) = control.dyn_into::<web_sys::HtmlElement>() {
                    let _ = control.focus();
                }
            }
        }
    };

    let unchecked_value_input = {
        let checked_signal = computed_checked_signal;
        let in_group = in_group;
        let name_for_gate = name.clone();
        let unchecked_value = unchecked_value.clone();
        let form_attr = form.clone();
        let disabled_attr = disabled_attr.clone();
        move || {
            let name_now = name_for_gate.get();
            should_render_unchecked_value_input(
                checked_signal.get(),
                in_group,
                name_now.as_deref(),
                parent,
                unchecked_value.as_deref(),
            )
            .then(|| {
                let name_now = name_now.clone().unwrap_or_default();
                let unchecked_value = unchecked_value.clone().unwrap_or_default();
                let form_attr = form_attr.clone();
                let disabled = disabled_attr.get().is_some();
                view! {
                    <input
                        type="hidden"
                        form={form_attr}
                        name={name_now}
                        value={unchecked_value}
                        disabled={disabled}
                    />
                }
            })
        }
    };

    // The three siblings upstream returns inside the provider (`:399-406`).
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

    view! {
        <>
            {control}
            {unchecked_value_input}
            <input
                node_ref=input_node
                type="checkbox"
                id={move || input_id_signal.get()}
                name={name_attr}
                form={form_attr}
                style={input_style_value}
                tabindex="-1"
                aria-hidden="true"
                disabled={move || disabled_attr.get().is_some()}
                required={required_attr}
                prop:value={move || input_value_signal.get().unwrap_or_default()}
                on:change=on_input_change_handler
                on:click={move |event: web_sys::MouseEvent| {
                    let wrapped = BaseUIEvent::new(event);
                    on_input_click(wrapped);
                }}
                on:focus=on_input_focus
            />
        </>
    }
}

/// The `role` member: the internal `"checkbox"` unless the consumer's rest bag
/// overrides it (the later-bag-wins rule — `CheckboxRoot.test.tsx:35-38`).
fn consumer_role(
    consumer_attribute_names: &[String],
    element_attributes: &[(String, String)],
) -> String {
    let _ = consumer_attribute_names;
    element_attributes
        .iter()
        .find(|(name, _)| name == "role")
        .map(|(_, value)| value.clone())
        .unwrap_or_else(|| "checkbox".to_string())
}

/// `ElementEventHandler<BaseUIEvent<E>>` → `ElementEventHandler<E>`: the inverse of
/// the internals' `native_to_base_ui`, needed because `useButton`'s external
/// click-like slots are native-typed while the element bags' slots are wrapped (the
/// crate's `ButtonExternalHandlers` contract).
fn base_ui_to_native<E: Clone + 'static>(
    handler: ElementEventHandler<BaseUIEvent<E>>,
) -> ElementEventHandler<E> {
    Rc::new(move |event: &E| handler(&BaseUIEvent::new(event.clone())))
}

/// The `visuallyHidden` recipes as a `style` string (`@base-ui/utils/visuallyHidden`).
fn style_string(declarations: &[(&str, &str)]) -> String {
    declarations
        .iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<Vec<_>>()
        .join(" ")
}
