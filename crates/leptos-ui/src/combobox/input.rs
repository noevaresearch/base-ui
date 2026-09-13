//! Combobox Input - Text input field for filtering and displaying values
//!
//! Port of Base UI's ComboboxInput component to Leptos.

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

/// Combobox input component props
#[derive(Clone, PartialEq)]
pub struct ComboboxInputProps {
    /// Input value
    pub value: String,
    
    /// Callback when input value changes
    pub on_change: Option<Callback<Event, ()>>,
    
    /// Callback when input is blurred
    pub on_blur: Option<Callback<FocusEvent, ()>>,
    
    /// Callback when input is focused
    pub on_focus: Option<Callback<FocusEvent, ()>>,
    
    /// Whether the input is disabled
    pub disabled: bool,
    
    /// Whether the input is read-only
    pub read_only: bool,
    
    /// Whether the input is required
    pub required: bool,
    
    /// AutoComplete attribute
    pub auto_complete: Option<String>,
    
    /// Form attribute
    pub form: Option<String>,
    
    /// Input element ref
    pub input_ref: NodeRef<HtmlInputElement>,
    
    /// Additional HTML attributes
    pub extra_attrs: Option<Vec<(&'static str, String)>>,
}

/// Combobox input component
pub fn ComboboxInput(props: ComboboxInputProps) -> impl IntoView {
    let ComboboxInputProps {
        value,
        on_change,
        on_blur,
        on_focus,
        disabled,
        read_only,
        required,
        auto_complete,
        form,
        input_ref,
        extra_attrs,
    } = props;
    
    // Handle input change
    let handle_change = use_stable_callback(
        on_change,
        move |ev: Event| {
            let new_value = event_target_value(&ev);
            if let Some(callback) = on_change {
                callback.call(ev);
            }
        }
    );
    
    // Handle blur
    let handle_blur = use_stable_callback(
        on_blur,
        move |ev: FocusEvent| {
            if let Some(callback) = on_blur {
                callback.call(ev);
            }
        }
    );
    
    // Handle focus
    let handle_focus = use_stable_callback(
        on_focus,
        move |ev: FocusEvent| {
            if let Some(callback) = on_focus {
                callback.call(ev);
            }
        }
    );
    
    // Build class names
    let class_name = if disabled {
        "leptos-combobox-input leptos-combobox-input--disabled"
    } else if read_only {
        "leptos-combobox-input leptos-combobox-input--readonly"
    } else {
        "leptos-combobox-input"
    };
    
    // Build extra attributes
    let extra_attrs_vec = extra_attrs.unwrap_or_default();
    
    view! {
        <input
            type="text"
            class=class_name
            value=value
            prop:disabled=disabled
            prop:readonly=read_only
            prop:required=required
            prop:autocomplete=auto_complete
            prop:form=form
            on:change=handle_change
            on:blur=handle_blur
            on:focus=handle_focus
            ..extra_attrs_vec
            ref=input_ref
        />
    }
}

impl Default for ComboboxInputProps {
    fn default() -> Self {
        Self {
            value: String::new(),
            on_change: None,
            on_blur: None,
            on_focus: None,
            disabled: false,
            read_only: false,
            required: false,
            auto_complete: None,
            form: None,
            input_ref: NodeRef::new(),
            extra_attrs: None,
        }
    }
}

/// Accessible combobox input with label
#[derive(Clone, PartialEq)]
pub struct ComboboxLabelProps {
    /// Label text
    pub children: Children,
    
    /// Whether the label is visually hidden (for screen readers)
    pub visually_hidden: bool,
    
    /// HTML element type for label
    pub r#as: Option<HtmlElement>,
    
    /// Additional HTML attributes
    pub extra_attrs: Option<Vec<(&'static str, String)>>,
}

/// Combobox label component
pub fn ComboboxLabel(props: ComboboxLabelProps) -> impl IntoView {
    let ComboboxLabelProps {
        children,
        visually_hidden,
        r#as,
        extra_attrs,
    } = props;
    
    let element_type = r#as.unwrap_or(html::label());
    let class_name = if visually_hidden {
        "leptos-combobox-label leptos-combobox-label--visually-hidden"
    } else {
        "leptos-combobox-label"
    };
    
    let extra_attrs_vec = extra_attrs.unwrap_or_default();
    
    view! {
        @match element_type {
            html::label() => {
                <label
                    class=class_name
                    ..extra_attrs_vec
                >
                    {children()}
                </label>
            }
            html::span() => {
                <span
                    class=class_name
                    ..extra_attrs_vec
                >
                    {children()}
                </span>
            }
            _ => {
                <element
                    class=class_name
                    ..extra_attrs_vec
                >
                    {children()}
                </element>
            }
        }
    }
}

impl Default for ComboboxLabelProps {
    fn default() -> Self {
        Self {
            children: Children::new(|_| view! { "" }),
            visually_hidden: false,
            r#as: None,
            extra_attrs: None,
        }
    }
}