//! Menu radio group - groups radio items together
//! 
//! This is a port of Base UI's MenuRadioGroup from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use std::rc::Rc;

/// Context for menu radio group
#[derive(Clone, PartialEq)]
pub struct MenuRadioGroupContext(Rc<(String, RwSignal<String>, Option<Callback<String>>, bool)>);

/// Props for the menu radio group component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuRadioGroupProps {
    /// Selected value (controlled)
    #[prop(into, optional)]
    value: Option<String>,
    /// Default selected value (uncontrolled)
    #[prop(into, optional)]
    default_value: Option<String>,
    /// Callback when the selected value changes
    #[prop(into, optional)]
    on_value_change: Option<Callback<String>>,
    /// Whether the group is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Custom ID for the group
    #[prop(into, optional)]
    id: Option<String>,
    /// ARIA label for the group
    #[prop(into, optional)]
    aria_label: Option<String>,
    /// ARIA describedby for the group
    #[prop(into, optional)]
    aria_describedby: Option<String>,
}

/// Radio group component for the menu
/// 
/// Groups radio items together and manages selection.
#[component]
pub fn MenuRadioGroup(
    /// Props for the menu radio group
    #[prop(optional)]
    props: MenuRadioGroupProps,
) -> impl IntoView {
    let MenuRadioGroupProps {
        value,
        default_value,
        on_value_change,
        disabled,
        id,
        aria_label,
        aria_describedby,
    } = props;
    
    // Generate a unique ID for the group
    let group_id = id.unwrap_or_else(|| use_base_ui_id());
    
    // State for the selected value
    let value_signal = value.unwrap_or_else(|| {
        let default = default_value.unwrap_or_else(|| "".to_string());
        create_rw_signal(default)
    });
    
    // Handle value changes
    let on_value_change = on_value_change.unwrap_or_else(|| Callback::new(|_| {}));
    
    // Set up the group context
    let group_context = MenuRadioGroupContext(Rc::new((
        group_id.clone(),
        value_signal.clone(),
        Some(on_value_change.clone()),
        disabled,
    )));
    
    // Provide the group context
    provide_context(group_context);
    
    view! {
        <div
            class="menu-radio-group"
            role="group"
            aria-disabled=disabled
            aria-label=aria_label
            aria-describedby=aria_describedby
            data-group-id=group_id
            data-disabled=disabled
        >
            {children()}
        </div>
    }
}

impl Default for MenuRadioGroupProps {
    fn default() -> Self {
        Self {
            value: None,
            default_value: None,
            on_value_change: None,
            disabled: false,
            id: None,
            aria_label: None,
            aria_describedby: None,
        }
    }
}

/// Hook to get menu radio group props
pub fn use_menu_radio_group_props() -> MenuRadioGroupProps {
    MenuRadioGroupProps::default()
}

/// Hook to get the menu radio group value
pub fn use_menu_radio_group_value() -> RwSignal<String> {
    // In a real implementation, this would read from context
    create_rw_signal("".to_string())
}

/// Hook to get the menu radio group default value
pub fn use_menu_radio_group_default_value() -> Option<String> {
    // In a real implementation, this would read from context
    None
}

/// Hook to get the menu radio group change callback
pub fn use_menu_radio_group_on_value_change() -> Option<Callback<String>> {
    // In a real implementation, this would read from context
    None
}

/// Hook to check if the menu radio group is disabled
pub fn use_menu_radio_group_disabled() -> bool {
    // In a real implementation, this would read from context
    false
}

/// Hook to get the menu radio group ID
pub fn use_menu_radio_group_id() -> Option<String> {
    // In a real implementation, this would read from context
    None
}

/// Hook to get the menu radio group ARIA label
pub fn use_menu_radio_group_aria_label() -> Option<String> {
    // In a real implementation, this would read from context
    None
}

/// Hook to get the menu radio group ARIA describedby
pub fn use_menu_radio_group_aria_describedby() -> Option<String> {
    // In a real implementation, this would read from context
    None
}

/// Hook to get the menu radio group context
pub fn use_menu_radio_group_context() -> Option<MenuRadioGroupContext> {
    use_context::<MenuRadioGroupContext>()
}

/// Hook to get the group ID from context
pub fn use_menu_radio_group_context_id() -> Option<String> {
    use_menu_radio_group_context().map(|ctx| ctx.0.clone())
}

/// Hook to get the value signal from context
pub fn use_menu_radio_group_context_value() -> Option<RwSignal<String>> {
    use_menu_radio_group_context().map(|ctx| ctx.1.clone())
}

/// Hook to get the change callback from context
pub fn use_menu_radio_group_context_on_value_change() -> Option<Callback<String>> {
    use_menu_radio_group_context().and_then(|ctx| ctx.2.clone())
}

/// Hook to get the disabled flag from context
pub fn use_menu_radio_group_context_disabled() -> bool {
    use_menu_radio_group_context().map(|ctx| ctx.3).unwrap_or(false)
}