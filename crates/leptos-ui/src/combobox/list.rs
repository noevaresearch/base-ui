//! Combobox List - Container for selectable items
//!
//! Port of Base UI's ComboboxList component to Leptos.

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

/// Combobox list component props
#[derive(Clone, PartialEq)]
pub struct ComboboxListProps<T = String> {
    /// List items
    pub items: Vec<T>,
    
    /// Currently active item index
    pub active_index: Option<usize>,
    
    /// Currently selected item index
    pub selected_index: Option<usize>,
    
    /// Whether to highlight items on hover
    pub highlight_on_hover: bool,
    
    /// Whether the list is disabled
    pub disabled: bool,
    
    /// List element ref
    pub list_ref: NodeRef<HtmlElement>,
    
    /// Callback when item is activated
    pub on_activate: Option<Callback<(T, usize), ()>>,
    
    /// Callback when item is selected
    pub on_select: Option<Callback<(T, usize), ()>>,
    
    /// Additional HTML attributes
    pub extra_attrs: Option<Vec<(&'static str, String)>>,
}

/// Combobox list component
pub fn ComboboxList<T>(props: ComboboxListProps<T>) -> impl IntoView
where
    T: Clone + PartialEq + 'static,
{
    let ComboboxListProps {
        items,
        active_index,
        selected_index,
        highlight_on_hover,
        disabled,
        list_ref,
        on_activate,
        on_select,
        extra_attrs,
    } = props;
    
    // Handle item click
    let handle_item_click = use_stable_callback(on_select);
    
    // Handle item hover
    let handle_item_hover = use_stable_callback(on_activate);
    
    // Build class names
    let class_name = if disabled {
        "leptos-combobox-list leptos-combobox-list--disabled"
    } else {
        "leptos-combobox-list"
    };
    
    // Build extra attributes
    let extra_attrs_vec = extra_attrs.unwrap_or_default();
    
    view! {
        <div
            class=class_name
            role="listbox"
            aria-activedescendant=active_index.map(|idx| format!("item-{}", idx))
            aria-multiselectable=highlight_on_hover
            prop:disabled=disabled
            ..extra_attrs_vec
            ref=list_ref
        >
            {items.into_iter().enumerate().map(|(index, item)| {
                view! {
                    <ComboboxItem<T>
                        item=item
                        index=index
                        is_active=active_index == Some(index)
                        is_selected=selected_index == Some(index)
                        on_click=handle_item_click
                        on_hover=handle_item_hover
                        disabled=disabled
                    />
                }
            }).collect_view()}
        </div>
    }
}

/// Combobox item component props
#[derive(Clone, PartialEq)]
pub struct ComboboxItemProps<T = String> {
    /// Item data
    pub item: T,
    
    /// Item index
    pub index: usize,
    
    /// Whether the item is active (highlighted)
    pub is_active: bool,
    
    /// Whether the item is selected
    pub is_selected: bool,
    
    /// Whether the item is disabled
    pub disabled: bool,
    
    /// Callback when item is clicked
    pub on_click: Option<Callback<(T, usize), ()>>,
    
    /// Callback when item is hovered
    pub on_hover: Option<Callback<(T, usize), ()>>,
    
    /// Additional HTML attributes
    pub extra_attrs: Option<Vec<(&'static str, String)>>,
}

/// Combobox item component
pub fn ComboboxItem<T>(props: ComboboxItemProps<T>) -> impl IntoView
where
    T: Clone + PartialEq + 'static,
{
    let ComboboxItemProps {
        item,
        index,
        is_active,
        is_selected,
        disabled,
        on_click,
        on_hover,
        extra_attrs,
    } = props;
    
    // Handle click
    let handle_click = use_stable_callback(
        on_click,
        move |_| {
            if let Some(callback) = on_click {
                callback.call((item.clone(), index));
            }
        }
    );
    
    // Handle hover
    let handle_hover = use_stable_callback(
        on_hover,
        move |_| {
            if let Some(callback) = on_hover {
                callback.call((item.clone(), index));
            }
        }
    );
    
    // Build class names
    let class_name = if disabled {
        "leptos-combobox-item leptos-combobox-item--disabled"
    } else if is_selected {
        "leptos-combobox-item leptos-combobox-item--selected"
    } else if is_active {
        "leptos-combobox-item leptos-combobox-item--active"
    } else {
        "leptos-combobox-item"
    };
    
    // Build extra attributes
    let extra_attrs_vec = extra_attrs.unwrap_or_default();
    
    view! {
        <div
            id=format!("item-{}", index)
            class=class_name
            role="option"
            aria-selected=is_selected
            aria-disabled=disabled
            prop:tabindex=if disabled { -1 } else { 0 }
            on:click=handle_click
            on:mouseenter=handle_hover
            ..extra_attrs_vec
        >
            // Render item content
            <ComboboxItemContent item=item />
        </div>
    }
}

/// Combobox item content component
pub fn ComboboxItemContent<T>(props: ComboboxItemProps<T>) -> impl IntoView
where
    T: Clone + PartialEq + 'static,
{
    let ComboboxItemProps { item, .. } = props;
    
    // Convert item to string for display
    let item_display = format!("{:?}", item); // TODO: Use item_to_string_label
    
    view! {
        <span class="leptos-combobox-item__content">
            {item_display}
        </span>
    }
}

/// Combobox item with custom renderer
#[derive(Clone, PartialEq)]
pub struct ComboboxItemWithRendererProps<T = String> {
    /// Item data
    pub item: T,
    
    /// Item index
    pub index: usize,
    
    /// Whether the item is active (highlighted)
    pub is_active: bool,
    
    /// Whether the item is selected
    pub is_selected: bool,
    
    /// Whether the item is disabled
    pub disabled: bool,
    
    /// Callback when item is clicked
    pub on_click: Option<Callback<(T, usize), ()>>,
    
    /// Callback when item is hovered
    pub on_hover: Option<Callback<(T, usize), ()>>,
    
    /// Custom renderer for item content
    pub children: Box<dyn Fn(T) -> View>,
    
    /// Additional HTML attributes
    pub extra_attrs: Option<Vec<(&'static str, String)>>,
}

/// Combobox item with custom renderer component
pub fn ComboboxItemWithRenderer<T>(props: ComboboxItemWithRendererProps<T>) -> impl IntoView
where
    T: Clone + PartialEq + 'static,
{
    let ComboboxItemWithRendererProps {
        item,
        index,
        is_active,
        is_selected,
        disabled,
        on_click,
        on_hover,
        children,
        extra_attrs,
    } = props;
    
    // Handle click
    let handle_click = use_stable_callback(
        on_click,
        move |_| {
            if let Some(callback) = on_click {
                callback.call((item.clone(), index));
            }
        }
    );
    
    // Handle hover
    let handle_hover = use_stable_callback(
        on_hover,
        move |_| {
            if let Some(callback) = on_hover {
                callback.call((item.clone(), index));
            }
        }
    );
    
    // Build class names
    let class_name = if disabled {
        "leptos-combobox-item leptos-combobox-item--disabled"
    } else if is_selected {
        "leptos-combobox-item leptos-combobox-item--selected"
    } else if is_active {
        "leptos-combobox-item leptos-combobox-item--active"
    } else {
        "leptos-combobox-item"
    };
    
    // Build extra attributes
    let extra_attrs_vec = extra_attrs.unwrap_or_default();
    
    view! {
        <div
            id=format!("item-{}", index)
            class=class_name
            role="option"
            aria-selected=is_selected
            aria-disabled=disabled
            prop:tabindex=if disabled { -1 } else { 0 }
            on:click=handle_click
            on:mouseenter=handle_hover
            ..extra_attrs_vec
        >
            // Render custom content
            <div class="leptos-combobox-item__content">
                {children(item.clone())}
            </div>
        </div>
    }
}

impl<T> Default for ComboboxListProps<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            active_index: None,
            selected_index: None,
            highlight_on_hover: true,
            disabled: false,
            list_ref: NodeRef::new(),
            on_activate: None,
            on_select: None,
            extra_attrs: None,
        }
    }
}

impl<T> Default for ComboboxItemProps<T> {
    fn default() -> Self {
        Self {
            item: panic!("Item is required"),
            index: 0,
            is_active: false,
            is_selected: false,
            disabled: false,
            on_click: None,
            on_hover: None,
            extra_attrs: None,
        }
    }
}

impl<T> Default for ComboboxItemWithRendererProps<T> {
    fn default() -> Self {
        Self {
            item: panic!("Item is required"),
            index: 0,
            is_active: false,
            is_selected: false,
            disabled: false,
            on_click: None,
            on_hover: None,
            children: Box::new(|_| view! { "" }),
            extra_attrs: None,
        }
    }
}