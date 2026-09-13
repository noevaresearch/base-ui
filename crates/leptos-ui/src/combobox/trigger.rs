//! Combobox Trigger - Button that opens/closes the combobox popup
//!
//! Port of Base UI's ComboboxTrigger component to Leptos.

use leptos::*;
use leptos::ev::*;
use leptos::html::*;
use leptos::wasm_bindgen::JsValue;
use leptos::wasm_bindgen::__rt::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, Event as WebEvent};

use leptos_ui_utils::{
    use_controlled, use_stable_callback, use_merged_refs, use_value_as_ref,
    visually_hidden, use_on_mount, use_iso_layout_effect, use_on_first_render,
};

use leptos_ui_internals::{
    use_render_element, merge_props, dispatch_click_with_modifiers,
    use_floating_root_context, use_floating, use_click, useDismiss,
    use_focus, use_list_navigation, ElementProps, FloatingRootContext,
    FloatingContext, FloatingFocusManager, FloatingPortal, FloatingList,
    use_floating_root_context,
};

/// Combobox trigger component props
#[derive(Clone, PartialEq)]
pub struct ComboboxTriggerProps {
    /// Whether the trigger is disabled
    pub disabled: bool,
    
    /// Whether the trigger is read-only
    pub read_only: bool,
    
    /// Trigger element ref
    pub trigger_ref: NodeRef<HtmlElement>,
    
    /// Additional HTML attributes
    pub extra_attrs: Option<Vec<(&'static str, String)>>,
    
    /// Callback when trigger is clicked
    pub on_click: Option<Callback<MouseEvent, ()>>,
    
    /// Callback when trigger is pressed (keyboard)
    pub on_press: Option<Callback<KeyboardEvent, ()>>,
    
    /// Callback when trigger is released (keyboard)
    pub on_release: Option<Callback<KeyboardEvent, ()>>,
}

/// Combobox trigger component
pub fn ComboboxTrigger(props: ComboboxTriggerProps) -> impl IntoView {
    let ComboboxTriggerProps {
        disabled,
        read_only,
        trigger_ref,
        extra_attrs,
        on_click,
        on_press,
        on_release,
    } = props;
    
    // Handle click
    let handle_click = use_stable_callback(
        on_click,
        move |ev: MouseEvent| {
            if let Some(callback) = on_click {
                callback.call(ev);
            }
        }
    );
    
    // Handle key press
    let handle_press = use_stable_callback(
        on_press,
        move |ev: KeyboardEvent| {
            if let Some(callback) = on_press {
                callback.call(ev);
            }
        }
    );
    
    // Handle key release
    let handle_release = use_stable_callback(
        on_release,
        move |ev: KeyboardEvent| {
            if let Some(callback) = on_release {
                callback.call(ev);
            }
        }
    );
    
    // Build class names
    let class_name = if disabled {
        "leptos-combobox-trigger leptos-combobox-trigger--disabled"
    } else if read_only {
        "leptos-combobox-trigger leptos-combobox-trigger--readonly"
    } else {
        "leptos-combobox-trigger"
    };
    
    // Build extra attributes
    let extra_attrs_vec = extra_attrs.unwrap_or_default();
    
    view! {
        <button
            type="button"
            class=class_name
            prop:disabled=disabled
            prop:readonly=read_only
            on:click=handle_click
            on:keydown=handle_press
            on:keyup=handle_release
            ..extra_attrs_vec
            ref=trigger_ref
        >
            <ComboboxTriggerIcon />
        </button>
    }
}

/// Combobox trigger icon component
pub fn ComboboxTriggerIcon() -> impl IntoView {
    view! {
        <svg
            class="leptos-combobox-trigger__icon"
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M4 6L8 10L12 6"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            />
        </svg>
    }
}

/// Combobox trigger with arrow down icon
pub fn ComboboxTriggerArrowDown() -> impl IntoView {
    view! {
        <svg
            class="leptos-combobox-trigger__arrow-down"
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M4 6L8 10L12 6"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            />
        </svg>
    }
}

/// Combobox trigger with arrow up icon
pub fn ComboboxTriggerArrowUp() -> impl IntoView {
    view! {
        <svg
            class="leptos-combobox-trigger__arrow-up"
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M4 10L8 6L12 10"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            />
        </svg>
    }
}

/// Combobox trigger with chevron icon
pub fn ComboboxTriggerChevron() -> impl IntoView {
    view! {
        <svg
            class="leptos-combobox-trigger__chevron"
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M4 6L8 10L12 6"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            />
        </svg>
    }
}

/// Combobox trigger with menu icon
pub fn ComboboxTriggerMenu() -> impl IntoView {
    view! {
        <svg
            class="leptos-combobox-trigger__menu"
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M2 4H14M2 8H14M2 12H14"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
            />
        </svg>
    }
}

impl Default for ComboboxTriggerProps {
    fn default() -> Self {
        Self {
            disabled: false,
            read_only: false,
            trigger_ref: NodeRef::new(),
            extra_attrs: None,
            on_click: None,
            on_press: None,
            on_release: None,
        }
    }
}