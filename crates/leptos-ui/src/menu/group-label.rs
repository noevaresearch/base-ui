//! Menu group label - labels for menu groups
//! 
//! This is a port of Base UI's MenuGroupLabel from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;

/// Props for the menu group label component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuGroupLabelProps {
    /// Whether the label is hidden
    #[prop(default = true)]
    aria_hidden: bool,
    /// Custom content for the label
    #[prop(optional)]
    children: Option<Children>,
    /// Custom render function for the label
    #[prop(optional)]
    render: Option<fn() -> HtmlElement>,
}

/// Group label component for the menu
/// 
/// Labels menu groups and provides accessibility.
#[component]
pub fn MenuGroupLabel(
    /// Props for the menu group label
    #[prop(optional)]
    props: MenuGroupLabelProps,
) -> impl IntoView {
    let MenuGroupLabelProps {
        aria_hidden,
        children,
        render,
    } = props;
    
    // Get the group context to set the label ID
    let group_context = use_context::<crate::menu::group::MenuGroupContext>();
    
    // Generate a unique ID for the label
    let label_id = use_base_ui_id();
    
    // Set the label ID in the group context
    if let Some(ctx) = &group_context {
        if let Some(set_label_id) = &ctx.1 {
            set_label_id.call(label_id.clone());
        }
    }
    
    // Render the label
    let label_content = if let Some(render) = render {
        render()
    } else if let Some(children) = children {
        children().into_view()
    } else {
        view! { { "Group Label" } }.into_view()
    };
    
    view! {
        <div
            class="menu-group-label"
            id=label_id
            aria-hidden=aria_hidden
        >
            {label_content}
        </div>
    }
}

impl Default for MenuGroupLabelProps {
    fn default() -> Self {
        Self {
            aria_hidden: true,
            children: None,
            render: None,
        }
    }
}

/// Hook to get menu group label props
pub fn use_menu_group_label_props() -> MenuGroupLabelProps {
    MenuGroupLabelProps::default()
}

/// Hook to check if the menu group label is hidden
pub fn use_menu_group_label_aria_hidden() -> bool {
    // In a real implementation, this would read from props or context
    true
}

/// Hook to get the menu group label ID
pub fn use_menu_group_label_id() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}