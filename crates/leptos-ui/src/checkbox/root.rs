//! Port of the Base UI Checkbox — the `library: checkbox` TODO item
//! (`specs/library/checkbox/behavior.md`, `specs/library/checkbox/implementation.md`).
//!
//! Upstream's structural facts this port follows (implementation.md):
//!
//! - **The component contains a state machine** — unlike Button, Checkbox has controlled/uncontrolled
//!   state via `useControlled` (`packages/react/src/checkbox/root/CheckboxRoot.tsx:137-145`), plus
//!   the `indeterminate` flag and Field lifecycle state. The port is therefore a stateful
//!   composition of already-ported utilities: [`leptos_ui_utils::use_controlled`] for the main
//!   boolean state, plus Field/Form/Group context integration.
//! - **State model**: `checked` is the primary boolean state (uncontrolled default: false), with
//!   `indeterminate` as an independent render-time flag that wins for `aria-checked` but doesn't
//!   affect the toggle logic. Field lifecycle state (touched/dirty/filled/focused/valid) comes
//!   from `FieldRootContext` and is not owned here.
//! - **DOM structure**: Root renders a visible control (span or button) + hidden native checkbox.
//!   The Indicator is conditionally rendered based on state + `keepMounted` prop.
//! - **Event handling**: Click re-dispatches to the hidden input via `dispatchClickWithModifiers`,
//!   with vetoable `onCheckedChange` handlers and Enter-based form submission handling.
//! - **Dependencies**: Requires `Field`, `Form`, `Labelable`, and `CheckboxGroup` contexts from
//!   the internals crate.

use leptos_ui_internals::floating_ui::element_props::ElementEventHandler;
use leptos_ui_internals::types::BaseUIEvent;
use leptos_ui_utils::use_controlled::use_controlled;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::GetUntracked;

/// Checkbox state — the primary boolean `checked` plus `indeterminate` flag
/// (`packages/react/src/checkbox/root/CheckboxRoot.tsx:147-150`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckboxState {
    /// The checked state (boolean, not ternary — `indeterminate` wins for aria-checked)
    pub checked: bool,
    /// The indeterminate flag (independent of checked, wins for aria-checked)
    pub indeterminate: bool,
}

impl CheckboxState {
    /// Convert state to a JSON map for `use_render_element`'s state attributes
    pub fn to_state_map(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        // `data-checked`/`data-unchecked` are mutually exclusive, both absent when indeterminate
        if !self.indeterminate {
            map.insert("checked".to_string(), serde_json::Value::Bool(self.checked));
            if !self.checked {
                map.insert("unchecked".to_string(), serde_json::Value::Bool(true));
            }
        }
        if self.indeterminate {
            map.insert("indeterminate".to_string(), serde_json::Value::Bool(true));
        }
        map
    }
}

/// Checkbox component props — upstream's destructured `Checkbox.Root.Props`
/// (`packages/react/src/checkbox/root/CheckboxRoot.tsx:58-91`)
pub struct CheckboxRootProps {
    /// `checked` — controlled mode
    pub checked: Option<bool>,
    /// `defaultChecked` — uncontrolled mode
    pub default_checked: Option<bool>,
    /// `onCheckedChange` — vetoable change handler
    pub on_checked_change: Option<fn(bool, BaseUIEvent<web_sys::Event>)>,
    /// `disabled`
    pub disabled: bool,
    /// `readOnly`
    pub read_only: bool,
    /// `required`
    pub required: bool,
    /// `indeterminate` — render-time flag
    pub indeterminate: bool,
    /// `name` — for form submission
    pub name: Option<String>,
    /// `nativeButton` — render as button element
    pub native_button: bool,
    /// `render` — custom render prop
    pub render: Option<leptos_ui_internals::use_render_element::RenderProp>,
    /// `id` — control id
    pub id: Option<String>,
    /// `value` — form value when checked
    pub value: Option<String>,
    /// `uncheckedValue` — form value when unchecked
    pub unchecked_value: Option<String>,
    /// `form` — external form id
    pub form: Option<String>,
    /// `parent` — inside CheckboxGroup
    pub parent: bool,
    /// `className`/`style`/`children` — through render props
    pub render_class_style: leptos_ui_internals::use_render_element::UseRenderElementComponentProps,
    /// Additional element attributes
    pub element_attributes: Vec<(String, String)>,
    /// Event handlers
    pub handlers: CheckboxRootHandlers,
}

impl Default for CheckboxRootProps {
    fn default() -> Self {
        Self {
            checked: None,
            default_checked: None,
            on_checked_change: None,
            disabled: false,
            read_only: false,
            required: false,
            indeterminate: false,
            name: None,
            native_button: false,
            render: None,
            id: None,
            value: None,
            unchecked_value: None,
            form: None,
            parent: false,
            render_class_style:
                leptos_ui_internals::use_render_element::UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            handlers: CheckboxRootHandlers::default(),
        }
    }
}

/// Checkbox event handlers (`packages/react/src/checkbox/root/CheckboxRoot.tsx:93-107`)
#[derive(Clone, Default)]
pub struct CheckboxRootHandlers {
    pub on_click: Option<ElementEventHandler<web_sys::MouseEvent>>,
    pub on_animation_end: Option<ElementEventHandler<web_sys::AnimationEvent>>,
    pub on_transition_end: Option<ElementEventHandler<web_sys::TransitionEvent>>,
}

/// Builds the Checkbox element description
///
/// Upstream's `Checkbox` body (`packages/react/src/checkbox/root/CheckboxRoot.tsx:14-406`)
/// ported to Leptos utilities.
pub fn checkbox_root_element(
    props: CheckboxRootProps,
) -> Option<leptos_ui_internals::use_render_element::RenderedElement> {
    let CheckboxRootProps {
        checked,
        default_checked,
        on_checked_change,
        disabled,
        read_only,
        required,
        indeterminate,
        name,
        native_button,
        render,
        id,
        value,
        unchecked_value,
        form,
        parent,
        render_class_style,
        element_attributes,
        handlers,
    } = props;

    // Use controlled state management - convert Option<bool> to RwSignal<Option<bool>>
    let checked_rw_signal = RwSignal::new(checked.map(|c| Some(c)));
    let default_checked_rw_signal = RwSignal::new(Some(default_checked.unwrap_or(false)));

    let (checked_signal, _) = leptos_ui_utils::use_controlled::use_controlled(
        leptos_ui_utils::use_controlled::UseControlledProps::new(
            checked_rw_signal,
            default_checked_rw_signal,
            "Checkbox",
        ),
    );

    // Computed state that respects indeterminate flag
    let computed_checked = if indeterminate {
        false // indeterminate wins for aria-checked but doesn't affect toggle logic
    } else {
        checked_signal.get_untracked().unwrap_or(false) // Unwrap the Option<bool>
    };

    let computed_indeterminate = indeterminate;

    // Create state record for attribute mapping
    let state = CheckboxState {
        checked: computed_checked,
        indeterminate: computed_indeterminate,
    };
    let state_map = state.to_state_map();

    // TODO: Implement the full checkbox logic including:
    // 1. Field/Form/Group context integration
    // 2. Hidden input creation and management
    // 3. Event handling with vetoable onCheckedChange
    // 4. Keyboard interactions (Space toggle, Enter submission)
    // 5. Indicator conditional rendering
    // 6. ID management and accessibility

    // For now, return a basic placeholder
    leptos_ui_internals::use_render_element::use_render_element(
        if native_button { "button" } else { "span" },
        render_class_style,
        leptos_ui_internals::use_render_element::UseRenderElementParams {
            enabled: !disabled,
            state: &state_map,
            refs: vec![],
            props: vec![],
            state_attributes_mapping: None,
        },
    )
}
