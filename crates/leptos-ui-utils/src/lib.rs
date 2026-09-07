//! Shared utilities for the Leptos port of Base UI (`packages/utils/src`).

pub mod add_event_listener;

pub use add_event_listener::{
    EventListenerOptions, EventListenerUnsubscribe, add_event_listener,
    add_event_listener_with_options,
};
