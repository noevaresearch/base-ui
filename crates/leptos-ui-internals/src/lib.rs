//! Shared internal infrastructure for the Leptos port of Base UI — the Phase A infra units of
//! `packages/react/src` (`use-render`, `merge-props`, `internals`, `floating-ui-react`,
//! `csp-provider`, `direction-provider`, `types`, `unstable-use-media-query`, `utils`), all
//! consolidated into this one crate by the crate-workspace decision in `specs/architecture.md`.

pub mod constants;
pub mod create_base_ui_event_details;
pub mod csp_context;
pub mod csp_provider;
pub mod direction_context;
pub mod direction_provider;
pub mod floating_ui;
pub mod item_equality;
pub mod resolve_value_label;
pub mod serialize_value;
pub mod state_attributes;
pub mod types;
pub mod use_media_query;

pub use constants::{
    BASE_UI_SWIPE_IGNORE_ATTRIBUTE, BASE_UI_SWIPE_IGNORE_SELECTOR, CLICK_TRIGGER_IDENTIFIER,
    CollisionAvoidance, DISABLED_TRANSITIONS_STYLE, DROPDOWN_COLLISION_AVOIDANCE,
    LEGACY_SWIPE_IGNORE_ATTRIBUTE, LEGACY_SWIPE_IGNORE_SELECTOR, OWNER_VISUALLY_HIDDEN,
    PATIENT_CLICK_THRESHOLD, POPUP_COLLISION_AVOIDANCE, TYPEAHEAD_RESET_MS,
};

pub use create_base_ui_event_details::{BaseUIChangeEventDetails, BaseUIGenericEventDetails};
pub use csp_context::{CSPContextValue, use_csp_context};
pub use csp_provider::provide_csp_context;
pub use direction_context::{DirectionContextValue, TextDirection, use_direction};
pub use direction_provider::provide_direction_context;
pub use types::{BaseUIEvent, ComponentRenderFn, HTMLProps};
pub use use_media_query::{
    MatchMediaFn, MatchMediaSource, SsrMatchMediaFn, UseMediaQueryOptions, use_media_query,
};
