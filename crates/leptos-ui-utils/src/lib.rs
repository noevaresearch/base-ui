//! Shared utilities for the Leptos port of Base UI (`packages/utils/src`).

pub mod add_event_listener;
pub mod are_arrays_equal;
pub mod clamp;
pub mod create_log_once;

pub use add_event_listener::{
    EventListenerOptions, EventListenerUnsubscribe, add_event_listener,
    add_event_listener_with_options,
};
pub use are_arrays_equal::{ObjectIs, are_arrays_equal, are_arrays_equal_by};
pub use clamp::{MAX_SAFE_INTEGER, MIN_SAFE_INTEGER, clamp};
pub use create_log_once::{LogOnce, Severity, create_log_once, create_log_once_with_prefix, reset};
