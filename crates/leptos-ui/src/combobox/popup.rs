//! Combobox Popup - Overlay container for the option list
//!
//! Port of Base UI's ComboboxPopup component to Leptos.

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

/// Combobox popup component props
#[derive(Clone, PartialEq)]
pub struct ComboboxPopupProps {
    /// Whether the popup is open
    pub open: bool,
    
    /// Whether the popup is modal
    pub modal: bool,
    
    /// Whether to render inline (no portal)
    pub inline: bool,
    
    /// Popup element ref
    pub popup_ref: NodeRef<HtmlElement>,
    
    /// Additional HTML attributes
    pub extra_attrs: Option<Vec<(&'static str, String)>>,
    
    /// Callback when popup is opened
    pub on_open: Option<Callback<(), ()>>,
    
    /// Callback when popup is closed
    pub on_close: Option<Callback<(), ()>>,
    
    /// Callback when popup is mounted
    pub on_mount: Option<Callback<(), ()>>,
    
    /// Callback when popup is unmounted
    pub on_unmount: Option<Callback<(), ()>>,
}

/// Combobox popup component
pub fn ComboboxPopup(props: ComboboxPopupProps) -> impl IntoView {
    let ComboboxPopupProps {
        open,
        modal,
        inline,
        popup_ref,
        extra_attrs,
        on_open,
        on_close,
        on_mount,
        on_unmount,
    } = props;
    
    // Handle open/close
    let handle_open = use_stable_callback(
        on_open,
        move || {
            if let Some(callback) = on_open {
                callback.call(());
            }
        }
    );
    
    let handle_close = use_stable_callback(
        on_close,
        move || {
            if let Some(callback) = on_close {
                callback.call(());
            }
        }
    );
    
    let handle_mount = use_stable_callback(
        on_mount,
        move || {
            if let Some(callback) = on_mount {
                callback.call(());
            }
        }
    );
    
    let handle_unmount = use_stable_callback(
        on_unmount,
        move || {
            if let Some(callback) = on_unmount {
                callback.call(());
            }
        }
    );
    
    // Build class names
    let class_name = if modal {
        "leptos-combobox-popup leptos-combobox-popup--modal"
    } else {
        "leptos-combobox-popup"
    };
    
    // Build extra attributes
    let extra_attrs_vec = extra_attrs.unwrap_or_default();
    
    // Handle mount/unmount effects
    Effect::new(move |_| {
        if open() {
            handle_open.call(());
        } else {
            handle_close.call(());
        }
    });
    
    // Effect for mount/unmount
    Effect::new(move |_| {
        on_mount(())
    });
    
    view! {
        @if inline {
            <div
                class=class_name
                prop:data-open=open
                ..extra_attrs_vec
                ref=popup_ref
            >
                <ComboboxPopupContent />
            </div>
        } @else {
            <FloatingPortal>
                <FloatingFocusManager modal>
                    <ComboboxPopupContent />
                </FloatingFocusManager>
            </FloatingPortal>
        }
    }
}

/// Combobox popup content component
pub fn ComboboxPopupContent() -> impl IntoView {
    view! {
        <div
            class="leptos-combobox-popup__content"
            role="listbox"
            aria-expanded="true"
        >
            // Popup content will be rendered here by parent components
            <ComboboxList />
        </div>
    }
}

/// Combobox popup backdrop (for modal mode)
pub fn ComboboxPopupBackdrop() -> impl IntoView {
    view! {
        <div
            class="leptos-combobox-popup__backdrop"
            on:click=|_| {
                // Handle backdrop click to close
            }
        >
        </div>
    }
}

/// Combobox popup wrapper with positioning
#[derive(Clone, PartialEq)]
pub struct ComboboxPopupWrapperProps {
    /// Whether the popup is open
    pub open: bool,
    
    /// Whether the popup is modal
    pub modal: bool,
    
    /// Whether to render inline (no portal)
    pub inline: bool,
    
    /// Reference to the trigger element
    pub trigger_ref: NodeRef<HtmlElement>,
    
    /// Reference to the input element
    pub input_ref: NodeRef<HtmlInputElement>,
    
    /// Reference to the list element
    pub list_ref: NodeRef<HtmlElement>,
    
    /// Additional HTML attributes
    pub extra_attrs: Option<Vec<(&'static str, String)>>,
    
    /// Callback when popup is opened
    pub on_open: Option<Callback<(), ()>>,
    
    /// Callback when popup is closed
    pub on_close: Option<Callback<(), ()>>,
}

/// Combobox popup wrapper component with positioning
pub fn ComboboxPopupWrapper(props: ComboboxPopupWrapperProps) -> impl IntoView {
    let ComboboxPopupWrapperProps {
        open,
        modal,
        inline,
        trigger_ref,
        input_ref,
        list_ref,
        extra_attrs,
        on_open,
        on_close,
    } = props;
    
    // Use floating-ui for positioning
    let floating_data = use_floating(
        trigger_ref,
        floating,
        Some(FloatingOptions {
            placement: Placement::Bottom,
            strategy: Strategy::Absolute,
            middleware: vec![
                Middleware::Flip,
                Middleware::Shift,
                Middleware::Offset(Offset::Pixels(4)),
            ],
        })
    );
    
    let open = use_read_signal(open);
    let on_open = use_read_signal(on_open);
    let on_close = use_read_signal(on_close);
    
    view! {
        @if inline {
            <ComboboxPopup
                open=open
                modal=modal
                inline=true
                ..extra_attrs
            >
                <ComboboxPopupContent />
            </ComboboxPopup>
        } @else {
            <ComboboxPopup
                open=open
                modal=modal
                inline=false
                ..extra_attrs
            >
                <ComboboxPopupContent />
            </ComboboxPopup>
        }
    }
}

/// Combobox popup animation styles
pub fn ComboboxPopupStyles() -> impl IntoView {
    view! {
        <style>
            .leptos-combobox-popup {
                position: absolute;
                z-index: 1000;
                display: none;
            }
            
            .leptos-combobox-popup[data-open="true"] {
                display: block;
            }
            
            .leptos-combobox-popup--modal {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                z-index: 1000;
            }
            
            .leptos-combobox-popup__content {
                position: absolute;
                background: white;
                border: 1px solid #e5e7eb;
                border-radius: 6px;
                box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
                max-height: 300px;
                overflow-y: auto;
                min-width: 200px;
            }
            
            .leptos-combobox-popup__backdrop {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0, 0, 0, 0.5);
                z-index: 999;
            }
            
            @keyframes leptos-combobox-popup-enter {
                from {
                    opacity: 0;
                    transform: translateY(-10px);
                }
                to {
                    opacity: 1;
                    transform: translateY(0);
                }
            }
            
            .leptos-combobox-popup__content {
                animation: leptos-combobox-popup-enter 0.2s ease-out;
            }
        </style>
    }
}

impl Default for ComboboxPopupProps {
    fn default() -> Self {
        Self {
            open: false,
            modal: false,
            inline: false,
            popup_ref: NodeRef::new(),
            extra_attrs: None,
            on_open: None,
            on_close: None,
            on_mount: None,
            on_unmount: None,
        }
    }
}

impl Default for ComboboxPopupWrapperProps {
    fn default() -> Self {
        Self {
            open: false,
            modal: false,
            inline: false,
            trigger_ref: NodeRef::new(),
            input_ref: NodeRef::new(),
            list_ref: NodeRef::new(),
            extra_attrs: None,
            on_open: None,
            on_close: None,
        }
    }
}