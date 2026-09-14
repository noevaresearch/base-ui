//! Port of the Base UI Checkbox Indicator — the `Checkbox.Indicator` subcomponent
//! (`specs/library/checkbox/behavior.md`, `specs/library/checkbox/implementation.md`).

use crate::checkbox::root::CheckboxState;
use leptos_ui_internals::use_render_element::RenderedElement;
use leptos_ui_internals::use_transition_status::TransitionStatus;

/// Checkbox Indicator component props
pub struct CheckboxIndicatorProps {
    /// `keepMounted` — keep mounted even when unchecked
    pub keep_mounted: bool,
    /// `className`/`style`/`children` — through render props
    pub render_class_style: leptos_ui_internals::use_render_element::UseRenderElementComponentProps,
    /// Additional element attributes
    pub element_attributes: Vec<(String, String)>,
    /// Event handlers
    pub handlers: CheckboxIndicatorHandlers,
}

impl Default for CheckboxIndicatorProps {
    fn default() -> Self {
        Self {
            keep_mounted: false,
            render_class_style:
                leptos_ui_internals::use_render_element::UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            handlers: CheckboxIndicatorHandlers::default(),
        }
    }
}

/// Checkbox Indicator event handlers
#[derive(Clone, Default)]
pub struct CheckboxIndicatorHandlers {
    pub on_animation_end: Option<
        leptos_ui_internals::floating_ui::element_props::ElementEventHandler<
            web_sys::AnimationEvent,
        >,
    >,
    pub on_transition_end: Option<
        leptos_ui_internals::floating_ui::element_props::ElementEventHandler<
            web_sys::TransitionEvent,
        >,
    >,
}

/// Builds the Checkbox Indicator element description
pub fn checkbox_indicator_element(
    props: CheckboxIndicatorProps,
    _state: &CheckboxState,
    _transition_status: TransitionStatus,
) -> Option<RenderedElement> {
    unimplemented!("Checkbox Indicator implementation to be completed")
}
