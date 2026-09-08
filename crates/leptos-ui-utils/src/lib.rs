//! Shared utilities for the Leptos port of Base UI (`packages/utils/src`).

pub mod add_event_listener;
pub mod are_arrays_equal;
pub mod clamp;
pub mod create_log_once;
pub mod create_selector;
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
pub mod platform;
pub mod react_store;
pub mod react_version;
pub mod safe_react;
pub mod shadow_dom;
pub mod store;
pub mod stringify_locale;
pub mod test_utils;
pub mod use_animation_frame;
pub mod use_controlled;
pub mod use_enhanced_click_handler;

#[cfg(test)]
pub(crate) mod test_support;

pub use add_event_listener::{
    EventListenerOptions, EventListenerUnsubscribe, add_event_listener,
    add_event_listener_with_options,
};
pub use are_arrays_equal::{ObjectIs, are_arrays_equal, are_arrays_equal_by};
pub use clamp::{MAX_SAFE_INTEGER, MIN_SAFE_INTEGER, clamp};
pub use create_log_once::{LogOnce, Severity, create_log_once, create_log_once_with_prefix, reset};
pub use create_selector::{
    MemoizedSelector, SelectorArgs, create_selector_1, create_selector_2, create_selector_3,
    create_selector_4, create_selector_5, create_selector_6, create_selector_7,
    create_selector_memoized, create_selector_memoized_1, create_selector_memoized_1_with_args,
    create_selector_memoized_2, create_selector_memoized_with_args,
};
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
pub use platform::{Engine, Env, MediaQuery, Os, Platform, ScreenReader, platform};
pub use react_store::{ReactStore, controlled_state_switch_message};
pub use react_version::{REACT_MAJOR_VERSION, SupportedVersion, is_react_version_at_least};
pub use safe_react::capture_owner_stack;
pub use shadow_dom::{active_element, contains, get_target};
pub use store::{Store, StoreListener, StoreUnsubscribe};
pub use stringify_locale::stringify_locale;
pub use test_utils::{TypeEq, expect_type, is_jsdom};
pub use use_animation_frame::{
    AnimationFrame, AnimationFrameId, cancel_animation_frame, request_animation_frame,
    reset_animation_frame_scheduler, use_animation_frame,
};
pub use use_controlled::{SetValueAction, UseControlledProps, use_controlled};
pub use use_enhanced_click_handler::{
    EnhancedClickHandlers, InteractionType, use_enhanced_click_handler,
};
