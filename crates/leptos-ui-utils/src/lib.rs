//! Shared utilities for the Leptos port of Base UI (`packages/utils/src`).

pub mod add_event_listener;
pub mod are_arrays_equal;
pub mod clamp;
pub mod create_log_once;
pub mod empty;
pub mod error;
pub mod fast_hooks;
pub mod fast_object_shallow_compare;
pub mod format_error_message;
pub mod format_number;
pub mod generate_id;
pub mod get_default_form_submitter;
pub mod get_react_element_ref;
pub mod inert_value;
pub mod is_element_disabled;
pub mod is_mouse_within_bounds;
pub mod merge_cleanups;
pub mod merge_objects;
pub mod owner;
pub mod stringify_locale;

#[cfg(test)]
pub(crate) mod test_support;

pub use add_event_listener::{
    EventListenerOptions, EventListenerUnsubscribe, add_event_listener,
    add_event_listener_with_options,
};
pub use are_arrays_equal::{ObjectIs, are_arrays_equal, are_arrays_equal_by};
pub use clamp::{MAX_SAFE_INTEGER, MIN_SAFE_INTEGER, clamp};
pub use create_log_once::{LogOnce, Severity, create_log_once, create_log_once_with_prefix, reset};
pub use empty::{EMPTY_OBJECT, NOOP, empty_array};
pub use error::error;
pub use fast_hooks::{
    Hook, HookCallback, Instance, InstanceHandle, fast_component, get_instance, register,
    set_instance,
};
pub use fast_object_shallow_compare::fast_object_shallow_compare;
pub use format_error_message::{
    DEFAULT_BASE_URL, DEFAULT_PREFIX, create_format_error_message, format_error_message,
};
pub use format_number::{NumberFormat, format_number, get_formatter};
pub use generate_id::generate_id;
pub use get_default_form_submitter::{DefaultFormSubmitter, get_default_form_submitter};
pub use get_react_element_ref::{ReactElement, get_react_element_ref};
pub use inert_value::inert_value;
pub use is_element_disabled::is_element_disabled;
#[allow(deprecated)]
pub use is_mouse_within_bounds::is_mouse_within_bounds_registered;
pub use is_mouse_within_bounds::{
    ElementBounds, get_pseudo_element_bounds, is_mouse_within_bounds,
};
pub use merge_cleanups::{CleanupFn, merge_cleanups};
pub use merge_objects::merge_objects;
pub use owner::{owner_document, owner_window};
pub use stringify_locale::stringify_locale;
