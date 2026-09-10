//! Shared internal infrastructure for the Leptos port of Base UI — the Phase A infra units of
//! `packages/react/src` (`use-render`, `merge-props`, `internals`, `floating-ui-react`,
//! `csp-provider`, `direction-provider`, `types`, `unstable-use-media-query`, `utils`), all
//! consolidated into this one crate by the crate-workspace decision in `specs/architecture.md`.

pub mod abort_signal;
pub mod adaptive_origin_constants;
pub mod adaptive_origin_middleware;
pub mod close_part;
pub mod collapsible_open_state_mapping;
pub mod common_popup_css_vars;
pub mod common_popup_data_attributes;
pub mod common_positioner_css_vars;
pub mod common_trigger_data_attributes;
pub mod common_viewport_data_attributes;
pub mod composite;
pub mod composite_grid_navigation;
pub mod composite_list;
pub mod composite_root_context;
pub mod composite_view;
pub mod constants;
pub mod create_base_ui_event_details;
pub mod csp_context;
pub mod csp_provider;
pub mod date_fns_calendar;
pub mod date_fns_format;
pub mod date_fns_locale;
pub mod date_fns_parse;
pub mod direction_context;
pub mod direction_provider;
pub mod dispatch_click_with_modifiers;
pub mod field_constants;
pub mod field_register_control;
pub mod field_root_context;
pub mod filter;
pub mod floating_ui;
pub mod form_context;
pub mod get_combined_field_validity_data;
pub mod get_css_dimensions;
pub mod get_disabled_mount_transition_styles;
pub mod get_element_at_point;
pub mod get_element_transform;
pub mod hide_middleware;
pub mod item_equality;
pub mod labelable_provider;
pub mod merge_props;
pub mod null_store;
pub mod popup_state_mapping;
pub mod popup_store_utils;
pub mod prehydration_script;
pub mod request_queue;
pub mod resolve_aria_labelled_by;
pub mod resolve_value_label;
pub mod scroll_edges;
pub mod scrollable;
pub mod serialize_value;
pub mod state_attributes;
pub mod styles;
pub mod temporal;
pub mod temporal_adapter_date_fns;
pub mod timeout_manager;
pub mod types;
pub mod use_anchor_positioning;
pub mod use_animations_finished;
pub mod use_base_ui_id;
pub mod use_button;
pub mod use_composite_item;
pub mod use_composite_list_item;
pub mod use_composite_root;
pub mod use_focusable_when_disabled;
pub mod use_is_hydrating;
pub mod use_media_query;
pub mod use_open_change_complete;
pub mod use_press_and_hold;
pub mod use_registered_label_id;
pub mod use_render_element;
pub mod use_transition_status;
pub mod use_value_changed;
pub mod value_to_percent;

pub use constants::{
    BASE_UI_SWIPE_IGNORE_ATTRIBUTE, BASE_UI_SWIPE_IGNORE_SELECTOR, CLICK_TRIGGER_IDENTIFIER,
    CollisionAvoidancePreset, DISABLED_TRANSITIONS_STYLE, DROPDOWN_COLLISION_AVOIDANCE,
    LEGACY_SWIPE_IGNORE_ATTRIBUTE, LEGACY_SWIPE_IGNORE_SELECTOR, OWNER_VISUALLY_HIDDEN,
    PATIENT_CLICK_THRESHOLD, POPUP_COLLISION_AVOIDANCE, TYPEAHEAD_RESET_MS,
};

pub use adaptive_origin_middleware::{
    ADAPTIVE_ORIGIN_NAME, AdaptiveOriginMiddleware, adaptive_origin,
};
pub use close_part::{
    ClosePartContextValue, SharedClosePartContext, provide_close_part_context,
    use_close_part_count, use_close_part_registration,
};
pub use common_popup_css_vars::{POPUP_HEIGHT, POPUP_WIDTH};
pub use common_popup_data_attributes::{
    ALIGN, ANCHOR_HIDDEN, CLOSED, ENDING_STYLE, OPEN, SIDE, STARTING_STYLE,
};
pub use common_trigger_data_attributes::{POPUP_OPEN, PRESSED};
pub use common_viewport_data_attributes::ACTIVATION_DIRECTION;
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
pub use composite_view::{
    CompositeItemComponentProps, CompositeRootComponentProps, OwnedStateAttributesMapping,
    composite_item, composite_root,
};
pub use create_base_ui_event_details::{BaseUIChangeEventDetails, BaseUIGenericEventDetails};
pub use csp_context::{CSPContextValue, use_csp_context};
pub use csp_provider::provide_csp_context;
pub use direction_context::{DirectionContextValue, TextDirection, use_direction};
pub use direction_provider::provide_direction_context;
pub use dispatch_click_with_modifiers::dispatch_click_with_modifiers;
pub use field_constants::{
    DEFAULT_FIELD_ROOT_STATE, DEFAULT_VALIDITY_STATE, FieldRootState, FieldValidityData,
    FieldValidityState,
};
pub use field_register_control::{
    FieldControlRegistration, GetControlValueFn, UseFieldControlRegistrationParams,
    UseFieldControlRegistrationReturn, UseRegisterFieldControlParams,
    use_field_control_registration, use_register_field_control,
};
pub use field_root_context::{
    FieldChangeFn, FieldCommitFn, FieldRootContextValue, FieldValidationBag, GetInputControlFn,
    RegisterFieldControlFn, RegisterInputFn, RegisteredInput, RegisteredInputs,
    SharedFieldRootContext, ValidationPropsFn, use_field_root_context,
    use_field_root_context_required,
};
pub use form_context::{
    ClearErrorsFn, FormContextValue, FormErrorValue, FormErrors, FormFieldEntry, FormFields,
    FormRef, FormState, FormValidationMode, GetFieldValueFn, SharedFormContext, use_form_context,
};
pub use get_combined_field_validity_data::get_combined_field_validity_data;
pub use get_css_dimensions::get_css_dimensions;
pub use get_disabled_mount_transition_styles::get_disabled_mount_transition_styles;
pub use get_element_at_point::get_element_at_point;
pub use get_element_transform::{ElementTransform, get_element_transform};
pub use labelable_provider::{
    ControlIdRegistration, ControlIdSource, DescriptionPropsFn, LabelProps, LabelableContextValue,
    RegisterControlIdFn, SharedLabelableContext, UseLabelParams, UseLabelableIdParams,
    focus_element_with_visible, provide_labelable_context, use_aria_labelled_by, use_label,
    use_labelable_context, use_labelable_id,
};
pub use merge_props::{
    PropsSource, RenderPropsGetter, merge_class_names, merge_event_handlers, merge_props_n,
    merge_styles,
};
pub use null_store::NullStore;
pub use popup_state_mapping::{
    popup_state_mapping, popup_transition_state_mapping, pressable_trigger_open_state_mapping,
    trigger_open_state_mapping,
};
pub use popup_store_utils::{
    PopupOpenState, PopupStore, apply_popup_open_change, attach_prevent_unmount_on_close,
    create_default_initial_focus, create_popup_open_state, focusable_popup_props,
    sync_trigger_count, use_trigger_registration,
};
pub use prehydration_script::{PrehydrationScriptProps, prehydration_script};
pub use resolve_aria_labelled_by::{get_default_label_id, resolve_aria_labelled_by};
pub use scroll_edges::{SCROLL_EDGE_TOLERANCE_PX, get_max_scroll_offset, normalize_scroll_offset};
pub use scrollable::{
    ScrollAxis, find_scrollable_touch_target, has_scrollable_ancestor, is_scrollable,
    is_scrollable_x, is_scrollable_y,
};
pub use styles::{DISABLE_SCROLLBAR_CLASS_NAME, DISABLE_SCROLLBAR_CSS, StyleDisableScrollbar};
pub use types::{BaseUIEvent, ComponentRenderFn, HTMLProps};
pub use use_anchor_positioning::{
    Align, Anchor, AnchorFn, ArrowStyles, CollisionAvoidance, CollisionAvoidanceAlign,
    CollisionAvoidanceSide, CollisionBoundary, FallbackAxisSide, OffsetData, OffsetFunction,
    PaddingRect, PhysicalSide, PositionerStyles, ShiftConfig, ShiftRootBoundary, Side, SideOffset,
    UseAnchorPositioningParams, UseAnchorPositioningReturn, get_logical_side,
    physical_side_for_param, placement_for, use_anchor_positioning,
    use_anchor_positioning_with_hook,
};
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
pub use use_is_hydrating::{set_is_hydrating, use_is_hydrating};
pub use use_media_query::{
    MatchMediaFn, MatchMediaSource, SsrMatchMediaFn, UseMediaQueryOptions, use_media_query,
};
pub use use_open_change_complete::{UseOpenChangeCompleteParams, use_open_change_complete};
pub use use_press_and_hold::{
    PressAndHoldOnStop, PressAndHoldPointerHandlers, PressAndHoldTick, UsePressAndHoldParams,
    UsePressAndHoldReturnValue, is_touch_like_pointer_type, use_press_and_hold,
};
pub use use_registered_label_id::{LabelIdSetter, LabelIdUpdate, use_registered_label_id};
pub use use_render_element::{
    ClassNameSource, RenderElementHandlers, RenderElementProps, RenderFn, RenderProp,
    RenderedElement, StyleSource, UseRenderElementComponentProps, UseRenderElementParams,
    native_to_base_ui, static_attr, use_render_element,
};
pub use use_transition_status::{TransitionStatus, UseTransitionStatus, use_transition_status};
pub use use_value_changed::use_value_changed;
pub use value_to_percent::value_to_percent;
