//! Menu root component
//! 
//! Ported from packages/react/src/menu/root/MenuRoot.tsx

use leptos::*;
use leptos_ui_utils::*;
use leptos_ui_internals::*;
use crate::menu::types::*;
use crate::menu::store::*;
use crate::menu::constants::*;

/// Menu root component
/// 
/// The main container that manages the menu state machine and provides context
/// to all menu components.
pub fn MenuRoot(
    props: menu::MenuRootProps,
) -> impl IntoView {
    let MenuRootProps { 
        open, 
        on_open_change, 
        modal, 
        root_id, 
        children 
    } = props;
    
    // Create menu store
    let store = Memo::new(move |_| {
        MenuStore::new(MenuParent::Root)
    });
    
    // Set up controlled/unmanaged open state
    let (open_signal, set_open_signal) = create_signal(false);
    
    Effect::new(move |_| {
        let open_value = open.get();
        set_open_signal.set(open_value);
    });
    
    // Set up on_open_change handler
    Effect::new(move |_| {
        store.internal().update(|internal| {
            internal.on_open_change = Some(SendWrapper::new(on_open_change.clone()));
        });
    });
    
    // Provide context to children
    provide_context(MenuRootContext {
        store: store.clone(),
    });
    
    // Render the root component (no DOM element, just providers)
    view! {
        <></>
    }
}

/// Menu root context
#[derive(Clone)]
pub struct MenuRootContext {
    pub store: Memo<MenuStore>,
}

impl Copy for MenuRootContext {}

/// Menu parent descriptor
#[derive(Clone, Debug, PartialEq)]
pub struct MenuParent {
    pub r#type: MenuParentType,
    pub id: Option<String>,
    pub has_popup: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MenuParentType {
    Root,
    Submenu,
    Menubar,
    ContextMenu,
    Toolbar,
}

/// Menu context for submenu roots
#[derive(Clone)]
pub struct MenuSubmenuRootContext {
    pub parent_menu: MenuParent,
}

impl Copy for MenuSubmenuRootContext {}

/// Menu positioner context
#[derive(Clone)]
pub struct MenuPositionerContext {
    pub side: String,
    pub align: String,
    pub arrow_ref: NodeRef,
    pub arrow_uncentered: bool,
    pub arrow_styles: String,
    pub context: PositionerContext,
}

impl Copy for MenuPositionerContext {}

/// Positioner context
#[derive(Clone)]
pub struct PositionerContext {
    pub node_id: String,
}

impl Copy for PositionerContext {}

/// Menu portal context
#[derive(Clone)]
pub struct MenuPortalContext {
    pub keep_mounted: bool,
}

impl Copy for MenuPortalContext {}

/// Menu group context 
#[derive(Clone)]
pub struct MenuGroupContext {
    pub set_label_id: Callback<String>,
}

impl Copy for MenuGroupContext {}

/// Menu radio group context
#[derive(Clone)]
pub struct MenuRadioGroupContext {
    pub value: RwSignal<String>,
    pub set_value: Callback<String>,
    pub disabled: RwSignal<bool>,
}

impl Copy for MenuRadioGroupContext {}

/// Menu checkbox item context
#[derive(Clone)]
pub struct MenuCheckboxItemContext {
    pub checked: RwSignal<bool>,
    pub highlighted: RwSignal<bool>,
    pub disabled: RwSignal<bool>,
}

impl Copy for MenuCheckboxItemContext {}

/// Menu radio item context
#[derive(Clone)]
pub struct MenuRadioItemContext {
    pub checked: RwSignal<bool>,
    pub highlighted: RwSignal<bool>,
    pub disabled: RwSignal<bool>,
}

impl Copy for MenuRadioItemContext {}

fn use_menu_root_context() -> MenuRootContext {
    use_context().expect("MenuRootContext not found")
}

fn use_menu_parent() -> MenuParent {
    let root_context = use_menu_root_context();
    // For now, return a default - in real implementation this would traverse contexts
    MenuParent {
        r#type: MenuParentType::Root,
        id: None,
        has_popup: true,
    }
}

fn use_menu_submenu_root_context() -> Option<MenuSubmenuRootContext> {
    use_context::<Option<MenuSubmenuRootContext>>().flatten()
}

fn provide_menu_submenu_root_context(
    parent_menu: MenuParent,
) {
    provide_context(Some(MenuSubmenuRootContext { parent_menu }));
}

fn use_menu_positioner_context() -> MenuPositionerContext {
    use_context().expect("MenuPositionerContext not found")
}

fn provide_menu_positioner_context(
    context: MenuPositionerContext,
) {
    provide_context(context);
}

fn use_menu_portal_context() -> MenuPortalContext {
    use_context().expect("MenuPortalContext not found")
}

fn provide_menu_portal_context(
    keep_mounted: bool,
) {
    provide_context(MenuPortalContext { keep_mounted });
}

fn use_menu_group_context() -> MenuGroupContext {
    use_context().expect("MenuGroupContext not found")
}

fn provide_menu_group_context(
    set_label_id: Callback<String>,
) {
    provide_context(MenuGroupContext { set_label_id });
}

fn use_menu_radio_group_context() -> MenuRadioGroupContext {
    use_context().expect("MenuRadioGroupContext not found")
}

fn provide_menu_radio_group_context(
    value: RwSignal<String>,
    set_value: Callback<String>,
    disabled: RwSignal<bool>,
) {
    provide_context(MenuRadioGroupContext { value, set_value, disabled });
}

fn use_menu_checkbox_item_context() -> MenuCheckboxItemContext {
    use_context().expect("MenuCheckboxItemContext not found")
}

fn provide_menu_checkbox_item_context(
    checked: RwSignal<bool>,
    highlighted: RwSignal<bool>,
    disabled: RwSignal<bool>,
) {
    provide_context(MenuCheckboxItemContext { checked, highlighted, disabled });
}

fn use_menu_radio_item_context() -> MenuRadioItemContext {
    use_context().expect("MenuRadioItemContext not found")
}

fn provide_menu_radio_item_context(
    checked: RwSignal<bool>,
    highlighted: RwSignal<bool>,
    disabled: RwSignal<bool>,
) {
    provide_context(MenuRadioItemContext { checked, highlighted, disabled });
}