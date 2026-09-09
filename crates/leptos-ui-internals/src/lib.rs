//! Shared internal infrastructure for the Leptos port of Base UI — the Phase A infra units of
//! `packages/react/src` (`use-render`, `merge-props`, `internals`, `floating-ui-react`,
//! `csp-provider`, `direction-provider`, `types`, `unstable-use-media-query`, `utils`), all
//! consolidated into this one crate by the crate-workspace decision in `specs/architecture.md`.

pub mod abort_signal;
pub mod composite;
pub mod composite_grid_navigation;
pub mod composite_list;
pub mod composite_root_context;
pub mod constants;
pub mod create_base_ui_event_details;
pub mod csp_context;
pub mod csp_provider;
pub mod direction_context;
pub mod direction_provider;
pub mod dispatch_click_with_modifiers;
pub mod filter;
pub mod floating_ui;
pub mod get_disabled_mount_transition_styles;
pub mod item_equality;
pub mod request_queue;
pub mod resolve_value_label;
pub mod serialize_value;
pub mod state_attributes;
pub mod timeout_manager;
pub mod types;
pub mod use_animations_finished;
pub mod use_base_ui_id;
pub mod use_button;
pub mod use_composite_item;
pub mod use_composite_list_item;
pub mod use_composite_root;
pub mod use_focusable_when_disabled;
pub mod use_media_query;
pub mod use_open_change_complete;
pub mod use_press_and_hold;
pub mod use_transition_status;
pub mod use_value_changed;

pub use constants::{
    BASE_UI_SWIPE_IGNORE_ATTRIBUTE, BASE_UI_SWIPE_IGNORE_SELECTOR, CLICK_TRIGGER_IDENTIFIER,
    CollisionAvoidance, DISABLED_TRANSITIONS_STYLE, DROPDOWN_COLLISION_AVOIDANCE,
    LEGACY_SWIPE_IGNORE_ATTRIBUTE, LEGACY_SWIPE_IGNORE_SELECTOR, OWNER_VISUALLY_HIDDEN,
    PATIENT_CLICK_THRESHOLD, POPUP_COLLISION_AVOIDANCE, TYPEAHEAD_RESET_MS,
};

pub use composite::{
    ACTIVE_COMPOSITE_ITEM, ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, COMPOSITE_KEYS, END,
    HOME, MODIFIER_KEYS, ModifierKey, PAGE_DOWN, PAGE_UP, SHIFT, is_composite_key, is_native_input,
    scroll_into_view_if_needed,
};
pub use composite_grid_navigation::{
    CompositeGridConfig, CompositeGridItemSize, CompositeGridNavigationState,
    CompositeGridNavigator, grid_navigation,
};
pub use composite_list::{
    CompositeItemRefCallback, CompositeList, CompositeListContextValue, CompositeListElementsRef,
    CompositeListLabelsRef, CompositeListMap, CompositeListRegistration, CompositeListUnsubscribe,
    CompositeMetadata, RegistrationLabel, SharedCompositeListContext, TextRef,
    compare_document_position_following, provide_composite_list, use_composite_list_context,
};
pub use composite_root_context::{
    CompositeRootContextValue, SharedCompositeRootContext, provide_composite_root_context,
    use_composite_root_context, use_composite_root_context_required,
};
pub use create_base_ui_event_details::{BaseUIChangeEventDetails, BaseUIGenericEventDetails};
pub use csp_context::{CSPContextValue, use_csp_context};
pub use csp_provider::provide_csp_context;
pub use direction_context::{DirectionContextValue, TextDirection, use_direction};
pub use direction_provider::provide_direction_context;
pub use dispatch_click_with_modifiers::dispatch_click_with_modifiers;
pub use get_disabled_mount_transition_styles::get_disabled_mount_transition_styles;
pub use types::{BaseUIEvent, ComponentRenderFn, HTMLProps};
pub use use_animations_finished::{
    ElementSource, RunOnceAnimationsFinish, use_animations_finished,
};
pub use use_base_ui_id::use_base_ui_id;
pub use use_button::{
    ButtonExternalHandlers, ButtonHandlers, ButtonProps, UseButtonParams, UseButtonReturnValue,
    use_button,
};
pub use use_composite_item::{
    CompositeItemProps, UseCompositeItem, UseCompositeItemParams, use_composite_item,
};
pub use use_composite_list_item::{
    UseCompositeListItem, UseCompositeListItemParams, use_composite_list_item,
};
pub use use_composite_root::{
    CompositeOnLoop, CompositeRootProps, UseCompositeRoot, UseCompositeRootParams,
    use_composite_root,
};
pub use use_focusable_when_disabled::{
    FocusableWhenDisabledProps, UseFocusableWhenDisabledParams, use_focusable_when_disabled,
};
pub use use_media_query::{
    MatchMediaFn, MatchMediaSource, SsrMatchMediaFn, UseMediaQueryOptions, use_media_query,
};
pub use use_open_change_complete::{UseOpenChangeCompleteParams, use_open_change_complete};
pub use use_press_and_hold::{
    PressAndHoldOnStop, PressAndHoldPointerHandlers, PressAndHoldTick, UsePressAndHoldParams,
    UsePressAndHoldReturnValue, is_touch_like_pointer_type, use_press_and_hold,
};
pub use use_transition_status::{TransitionStatus, UseTransitionStatus, use_transition_status};
