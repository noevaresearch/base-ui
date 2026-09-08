//! Port of `packages/react/src/floating-ui-react` — Base UI's floating-ui unit
//! (`specs/library/floating-ui-react/behavior.md`,
//! `specs/library/floating-ui-react/implementation.md`), the shared interaction,
//! positioning-adjacent, and popup-infrastructure layer all popup components consume.
//!
//! The unit wraps `@floating-ui/react-dom` and `@floating-ui/utils` (the TODO's
//! `wraps-external` field); positioning/middleware math, platform/DOM internals, and the
//! other third-party algorithms are deliberately NOT ported — they bind to the
//! `floating-ui-dom` Rust crate (RustForWeb's port, the same crate the `rust-equivalent`
//! `floating-ui-leptos` re-exports wholesale). See
//! `specs/library/floating-ui-react/implementation.md`, "Dependencies on other Base UI
//! internals" for the delegation mandate.
//!
//! Module map (upstream file → port):
//! - `types.ts` → [`types`]
//! - `utils/createEventEmitter.ts` → [`types`] (`EventEmitter`)
//! - `utils/constants.ts` → [`constants`]
//! - `utils/createAttribute.ts` → [`create_attribute`]
//! - `utils/event.ts` → [`event`]
//! - `utils/element.ts` → [`element`]
//! - `utils/nodes.ts` → [`nodes`]
//! - `utils/enqueueFocus.ts` → [`enqueue_focus`]
//! - `utils/getEmptyRootContext.ts` → [`get_empty_root_context`]
//! - `components/FloatingRootStore.ts` → [`floating_root_store`]
//! - `components/FloatingTreeStore.ts` + `components/FloatingTree.tsx` → [`tree`]
//! - `utils/popups/popupTriggerMap.ts` (provisional home, see its module docs) →
//!   [`popup_trigger_map`]
//!
//! Not yet ported (remaining checkpoints of the unit): the context hooks
//! (`useFloatingRootContext`, `useSyncedFloatingRootContext`, `useFloating`), the
//! interaction hooks (`useClick`, `useClientPoint`, `useDismiss`, `useFocus`,
//! `useHover`, `useHoverFloatingInteraction`, `useHoverReferenceInteraction`,
//! `useHoverShared`, `useHoverInteractionSharedState`), the navigation hooks
//! (`useListNavigation`, `gridNavigation`, `useTypeahead`) and `utils/composite.ts`,
//! `safePolygon`, `utils/markOthers.ts`, `utils/tabbable.ts`, and the components
//! (`FloatingDelayGroup`, `FloatingFocusManager`, `FloatingPortal`) plus the vendored
//! `middleware/arrow.ts`.

pub mod constants;
pub mod create_attribute;
pub mod element;
pub mod enqueue_focus;
pub mod event;
pub mod floating_root_store;
pub mod get_empty_root_context;
pub mod nodes;
pub mod popup_trigger_map;
pub mod tree;
pub mod types;
pub mod use_floating;
pub mod use_floating_root_context;
pub mod use_position;

pub use constants::{
    ACTIVE_KEY, ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, FOCUSABLE_ATTRIBUTE,
    SELECTED_KEY, TYPEABLE_SELECTOR,
};
pub use create_attribute::create_attribute;
pub use element::{
    get_floating_focus_element, is_event_target_within, is_interactive_element,
    is_root_element, is_target_inside_enabled_trigger, is_typeable_combobox, is_typeable_element,
    matches_focus_visible,
};
pub use event::{
    is_click_like_event, is_mouse_like_pointer_type, is_virtual_click, is_virtual_pointer_event,
    stop_event,
};
pub use floating_root_store::{
    FloatingRootStore, FloatingRootStoreContext, FloatingRootStoreOptions, FloatingRootState,
    selectors,
};
pub use get_empty_root_context::get_empty_root_context;
pub use nodes::{get_deepest_node, get_node_ancestors, get_node_children};
pub use popup_trigger_map::PopupTriggerMap;
pub use tree::{
    provide_floating_node, provide_floating_tree, use_floating_node_id,
    use_floating_parent_node_id, use_floating_tree, FloatingNodeContext, FloatingTreeContext,
    FloatingTreeStore, SharedFloatingTreeStore,
};
pub use types::{
    ContextData, Delay, EventEmitter, EventListener, EventUnsubscribe, ExtendedElements,
    ExtendedRefs, FloatingContext, FloatingEvents, FloatingNodeType, FloatingTreeEvent,
    FloatingTreeEvents, FloatingTreeType, FloatingUIOpenChangeDetails, InsideReactTree,
    OnOpenChangeFn, Orientation, PositioningStyles, ReferenceType, RootOpenChangeEventDetails,
    TransitionStatus, UseFloatingReturn, WhileElementsMountedCleanupFn, WhileElementsMountedFn,
    WrappedMiddleware,
};
pub use use_floating::{use_base_ui_floating, use_floating, UseFloatingOptions};
pub use use_floating_root_context::{
    use_floating_root_context, UseFloatingRootContextOptions,
};
pub use use_position::{use_position, UsePositionOptions, UsePositionReturn};
pub use enqueue_focus::{enqueue_focus, EnqueueFocusOptions};

/// The positioning-engine vocabulary the unit re-exports through `types.ts:28-85` —
/// bound to the external `floating-ui-dom` crate (see the module docs).
pub use types::{
    AlignedPlacement, Alignment, AutoUpdateOptions, Axis, Boundary, Coords,
    ComputePositionConfig, ComputePositionReturn, DetectOverflowOptions, Dimensions,
    ElementContext, ElementOrVirtual, ElementRects, Middleware, MiddlewareData, MiddlewareState,
    Padding, Platform, RootBoundary, Side, SideObject, auto_update, compute_position, dom,
};
