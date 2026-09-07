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
