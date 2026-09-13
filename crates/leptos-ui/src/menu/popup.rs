//! Menu popup component
//! 
//! Ported from packages/react/src/menu/popup/MenuPopup.tsx

use leptos::*;
use leptos_ui_utils::*;
use leptos_ui_internals::*;
use crate::menu::types::*;
use crate::menu::store::*;
use crate::menu::root::*;
use super::primitive::Primitive;

/// Menu popup component
/// 
/// The container that wraps the menu content and handles positioning.
pub fn MenuPopup(
    props: menu::MenuPopupProps,
) -> impl IntoView {
    let menu::MenuPopupProps { 
        as_child, 
        children,
        .. 
    } = props;
    
    // Get menu root context
    let root_context = use_menu_root_context();
    let store = root_context.store;
    
    // Get popup ref from store
    let popup_ref = store.popup_ref();
    
    // Check if menu is open and mounted
    let is_open = Memo::new(move |_| store.open().get() && store.mounted().get());
    
    // Positioning classes
    let positioning_classes = move || {
        let transition_status = store.transition_status().get();
        let mut classes = Vec::new();
        
        // Add transition classes
        match transition_status {
            TransitionStatus::Entering => {
                classes.push("transition-all");
                classes.push("duration-200");
                classes.push("ease-out");
            }
            TransitionStatus::Exiting => {
                classes.push("transition-all");
                classes.push("duration-200");
                classes.push("ease-in");
            }
            _ => {}
        }
        
        classes.join(" ")
    };
    
    // Render the popup
    if as_child.get() {
        // Render children directly when as_child is true
        view! {
            @if let Some(children) = children {
                <div 
                    node_ref=popup_ref
                    class=positioning_classes()
                    data-state=move || if is_open() { "open" } else { "closed" }
                    aria-hidden=move || !is_open().to_string()
                    style="display: none;" // Hidden when closed
                >
                    {children()}
                </div>
            }
        }
    } else {
        // Render as a div when as_child is false
        view! {
            @if let Some(children) = children {
                <Primitive
                    element="div"
                    node_ref=popup_ref
                    as_child=as_child
                    class=positioning_classes()
                    data-state=move || if is_open() { "open" } else { "closed" }
                    aria-hidden=move || !is_open().to_string()
                    style="display: none;" // Hidden when closed
                >
                    {children()}
                </Primitive>
            }
        }
    }
}

/// Hook to get popup state
pub fn use_popup_state() -> (ReadSignal<bool>, ReadSignal<bool>) {
    let root_context = use_menu_root_context();
    let store = root_context.store;
    
    let is_open = store.open();
    let is_mounted = store.mounted();
    
    (is_open.read_only(), is_mounted.read_only())
}

/// Hook to get popup ref
pub fn use_popup_ref() -> NodeRef {
    let root_context = use_menu_root_context();
    root_context.store.popup_ref()
}

/// Menu popup context for child components
#[derive(Clone)]
pub struct MenuPopupContext {
    pub open: ReadSignal<bool>,
    pub mounted: ReadSignal<bool>,
    pub popup_ref: NodeRef,
}

impl Copy for MenuPopupContext {}

/// Provide popup context
pub fn provide_menu_popup_context(context: MenuPopupContext) {
    provide_context(context);
}

/// Use popup context
pub fn use_menu_popup_context() -> MenuPopupContext {
    use_context().expect("MenuPopupContext not found")
}

/// Menu popup portal component
/// 
/// Renders the popup in a portal for positioning
pub fn MenuPopupPortal(
    props: menu::MenuPopupPortalProps,
) -> impl IntoView {
    let menu::MenuPopupPortalProps { 
        as_child, 
        children,
        portal_ref,
        .. 
    } = props;
    
    let root_context = use_menu_root_context();
    let store = root_context.store;
    
    let is_open = Memo::new(move |_| store.open().get() && store.mounted().get());
    
    // Portal context
    provide_menu_portal_context(true); // Keep mounted
    
    // Render the portal
    if as_child.get() {
        view! {
            @if let Some(children) = children {
                @if is_open() {
                    <div node_ref=portal_ref>
                        {children()}
                    </div>
                }
            }
        }
    } else {
        view! {
            @if let Some(children) = children {
                @if is_open() {
                    <Primitive
                        element="div"
                        node_ref=portal_ref
                        as_child=as_child
                    >
                        {children()}
                    </Primitive>
                }
            }
        }
    }
}

/// Menu popup arrow component
/// 
/// Renders an arrow for the popup
pub fn MenuPopupArrow(
    props: menu::MenuPopupArrowProps,
) -> impl IntoView {
    let menu::MenuPopupArrowProps { 
        as_child, 
        children,
        arrow_ref,
        arrow_uncentered,
        arrow_styles,
        .. 
    } = props;
    
    // Positioner context for arrow positioning
    let positioner_context = use_menu_positioner_context();
    
    let arrow_classes = move || {
        let mut classes = Vec::new();
        
        if arrow_uncentered.get() {
            classes.push("data-[uncentered]:translate-x-1/2");
        }
        
        if let Some(styles) = arrow_styles.get() {
            classes.push(styles);
        }
        
        classes.join(" ")
    };
    
    if as_child.get() {
        view! {
            @if let Some(children) = children {
                <div 
                    node_ref=arrow_ref
                    class=arrow_classes()
                    data-arrow=""
                >
                    {children()}
                </div>
            }
        }
    } else {
        view! {
            @if let Some(children) = children {
                <Primitive
                    element="div"
                    node_ref=arrow_ref
                    as_child=as_child
                    class=arrow_classes()
                    data-arrow=""
                >
                    {children()}
                </Primitive>
            }
        }
    }
}

/// Menu popup backdrop component
/// 
/// Renders a backdrop for modal menus
pub fn MenuPopupBackdrop(
    props: menu::MenuPopupBackdropProps,
) -> impl IntoView {
    let menu::MenuPopupBackdropProps { 
        as_child, 
        children,
        backdrop_ref,
        .. 
    } = props;
    
    let root_context = use_menu_root_context();
    let store = root_context.store;
    
    let is_open = Memo::new(move |_| store.open().get() && store.mounted().get());
    
    if as_child.get() {
        view! {
            @if let Some(children) = children {
                @if is_open() {
                    <div 
                        node_ref=backdrop_ref
                        class="fixed inset-0 bg-black/50 backdrop-blur-sm"
                        aria-hidden="true"
                    >
                        {children()}
                    </div>
                }
            }
        }
    } else {
        view! {
            @if let Some(children) = children {
                @if is_open() {
                    <Primitive
                        element="div"
                        node_ref=backdrop_ref
                        as_child=as_child
                        class="fixed inset-0 bg-black/50 backdrop-blur-sm"
                        aria-hidden="true"
                    >
                        {children()}
                    </Primitive>
                }
            }
        }
    }
}

/// Menu popup viewport component
/// 
/// The scrollable viewport for the menu
pub fn MenuPopupViewport(
    props: menu::MenuPopupViewportProps,
) -> impl IntoView {
    let menu::MenuPopupViewportProps { 
        as_child, 
        children,
        viewport_ref,
        .. 
    } = props;
    
    if as_child.get() {
        view! {
            @if let Some(children) = children {
                <div 
                    node_ref=viewport_ref
                    class="max-h-60 w-48 overflow-auto rounded-md border bg-popover p-1 text-popover-foreground shadow-md"
                >
                    {children()}
                </div>
            }
        }
    } else {
        view! {
            @if let Some(children) = children {
                <Primitive
                    element="div"
                    node_ref=viewport_ref
                    as_child=as_child
                    class="max-h-60 w-48 overflow-auto rounded-md border bg-popover p-1 text-popover-foreground shadow-md"
                >
                    {children()}
                </Primitive>
            }
        }
    }
}