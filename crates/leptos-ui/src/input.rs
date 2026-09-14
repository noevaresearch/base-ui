//! Input — port of `packages/react/src/input/Input.tsx`
//!
//! A pure delegation wrapper around `Field.Control` that provides the standard
//! input element interface with Base UI's field state management.

use leptos::prelude::*;

use crate::field::field_control::FieldControl;

/// The Input component - a pure delegation wrapper around Field.Control
///
/// This component forwards all props to Field.Control, providing the standard
/// input element interface with Base UI's field state management.
#[component]
pub fn Input(
    /// CSS class name(s) to apply to the root element
    #[prop(default = None, optional)]
    class: Option<String>,
    /// The explicit id
    #[prop(default = None, optional)]
    id: Option<String>,
    /// The control's name
    #[prop(default = None, optional)]
    name: Option<String>,
    /// The controlled value
    #[prop(default = None, optional)]
    value: Option<String>,
    /// The default value for uncontrolled mode
    #[prop(default = None, optional)]
    default_value: Option<String>,
    /// Whether the input is disabled
    #[prop(default = false, optional)]
    disabled: bool,
) -> impl IntoView {
    view! {
        <FieldControl
            id=id.clone().unwrap_or_default()
            name=name.clone().unwrap_or_default()
            value=value.clone().unwrap_or_default()
            default_value=default_value.clone().unwrap_or_default()
            disabled=disabled
            class=class.clone().unwrap_or_default()
        />
    }
}
