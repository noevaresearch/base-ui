//! Port of `packages/react/src/utils/usePopupViewport.tsx` — the shared morphing-viewport
//! engine the four `Viewport` parts compose (Popover, Menu, Tooltip, PreviewCard;
//! `specs/library/utils/implementation.md`, "Viewport / auto-resize
//! (`usePopupViewport.tsx` + `usePopupAutoResize.ts`)").
//!
//! Upstream is one 396-line module with four separable parts, all carried here:
//!
//! 1. the state-attribute mapping `popupViewportStateMapping` (`:21-30`) — the one data
//!    attribute the four Viewport parts expose, [`ACTIVATION_DIRECTION`];
//! 2. the pure geometry — `getActivationDirection` (`:307-313`),
//!    `getValueWithTolerance` (`:325-340`) with upstream's 5px tolerance, and
//!    `calculateRelativePosition` (`:345-362`) over two element rects;
//! 3. the remount-key walk `usePopupContentKey` (`:367-396`) — the trigger-id/payload
//!    comparison whose second bump handles a payload that arrives a render late;
//! 4. the engine `usePopupViewport` (`:75-294`) — the trigger-change walk onto the captured
//!    DOM clone (`:156-176`), the `data-previous`/`data-current` container choreography with
//!    `data-starting-style`/`data-ending-style` (`:227-265`), the frozen
//!    `--popup-width`/`--popup-height` on the exiting container (`:243-248`), the clone
//!    capture (`:204-224`) and its move into the previous container (`:268-275`), and the
//!    cleanup re-arming after animations finish (`:137-146`, `:184-202`).
//!
//! The observable contract of that render output is [`PopupViewportContainers`] — which
//! containers exist and which attributes/style declarations each carries — so the state
//! walk is host-testable even though the DOM it describes is not (see below).
//!
//! ## Rust adaptations
//!
//! - **`side`/`direction` are absent from the hook's parameters.** Upstream declares `side`
//!   at `:53`, destructures it from the parameters at `:76`, and passes it (with
//!   `useDirection()`, `:78`) into the `usePopupAutoResize` composition at `:284` — those are
//!   its every occurrence in the file. That util is a separate unported module, so it is its
//!   own ledger item and the port carries no dead parameter. The measure callbacks that util
//!   drives (`handleMeasureLayout`/`handleMeasureLayoutComplete`, `:119-135`) ARE part of this
//!   engine's result, because they are the engine's own ref writes.
//! - **Two of the seven store reads are not taken here.** `popupElement` and
//!   `positionerElement` (`:85-86`) appear upstream only at those lines and in the
//!   auto-resize call (`:278-279`); the five the engine itself consumes are read.
//! - **`previousContentDimensions` is `Option<Dimensions>`** — upstream's
//!   `{ width, height } | null` (`:105-108`) is the floating-ui `Dimensions` record the port
//!   already carries for this exact value (`crate::floating_ui::types::Dimensions`, the type
//!   the measure-complete callback receives).
//! - **`usePreviousValue` is re-derived here.** The Phase A hook requires `ObjectIs`
//!   (`use_previous_value.rs:101-103`), whose impls are primitives/`String`/`&str`
//!   (`are_arrays_equal.rs:33-52`) — `web_sys::Element` has none — so the
//!   `open ? activeTrigger : null` channel (`:88`) carries [`PreviousValue`] with the same
//!   read-time-transition contract and `Node::is_same_node` as the `Object.is` for nodes.
//! - **The clone-capture effect tracks instead of running unkeyed.** Upstream's capture
//!   (`:206-224`) has no dep array, so it re-clones after every commit; a Leptos effect runs
//!   once plus per tracked change ([`leptos_ui_utils::use_iso_layout_effect`]), so the port
//!   tracks the content key and the transition flag — the two things that replace the subtree
//!   the clone is taken from.
//! - **`store.set('adaptiveOrigin', adaptiveOrigin)` (`:112-117`) is re-homed, not dropped:**
//!   the port's `adaptive_origin` middleware is an option of
//!   [`crate::use_anchor_positioning`] (`use_anchor_positioning.rs:502,1057-1058`) rather than
//!   a store field this crate's `PopupStoreState` has none of, so the registration is the
//!   positioner's own wiring.
//! - **Attributes are returned as pairs, not a React props object:** the container contract is
//!   [`PopupViewportContainers::current_attributes`] /
//!   [`PopupViewportContainers::previous_attributes`] /
//!   [`PopupViewportContainers::previous_style_declarations`], which is what the view layer
//!   spreads onto the elements it builds.
//! - **The DOM half is compile-verified only in this environment.** The crate's wasm suite runs
//!   in no CI job and this box refuses browsers by design
//!   (`ralph/generated/env-health.json`: `browser: DEGRADED`), so the real mount, the clone
//!   move, the animation watcher and the layout writes are pinned here by
//!   `cargo check --target wasm32-unknown-unknown` plus the host suite over the state walk and
//!   the direction math — never claimed as executed.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::owner::{LocalStorage, on_cleanup};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, GetValue, Set};
use reactive_graph::wrappers::read::Signal;
use serde_json::{Map, Value};
use send_wrapper::SendWrapper;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement};

use leptos_ui_utils::inert_value::inert_value;
use leptos_ui_utils::owner::owner_document;
use leptos_ui_utils::use_animation_frame::use_animation_frame;
use leptos_ui_utils::use_iso_layout_effect::use_iso_layout_effect;
use leptos_ui_utils::use_ref_with_init::use_ref_with_init;
use leptos_ui_utils::use_stable_callback::{StableCallback, use_stable_callback};

use crate::abort_signal::AbortSignal;
use crate::common_popup_css_vars::{POPUP_HEIGHT, POPUP_WIDTH};
use crate::common_viewport_data_attributes::ACTIVATION_DIRECTION;
use crate::floating_ui::popup_store::{self};
use crate::floating_ui::types::Dimensions;
use crate::popup_store_utils::PopupStore;
use crate::state_attributes::{
    StateAttributeProps, get_state_attributes_props, is_truthy, js_to_string,
};
use crate::use_animations_finished::use_animations_finished;

/// The activation-direction tolerance (`usePopupViewport.tsx:312`) — the ~5px band inside
/// which an axis contributes no token.
pub const ACTIVATION_DIRECTION_TOLERANCE_PX: f64 = 5.0;

/// The center-to-center offset between two triggers (`usePopupViewport.tsx:296-299`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Offset {
    /// `offset.horizontal`.
    pub horizontal: f64,
    /// `offset.vertical`.
    pub vertical: f64,
}

/// A `getBoundingClientRect()` reading — the four fields `calculateRelativePosition`
/// (`:346-356`) consumes, split out so the math is host-testable without a layout engine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElementRect {
    /// `rect.left`.
    pub left: f64,
    /// `rect.top`.
    pub top: f64,
    /// `rect.width`.
    pub width: f64,
    /// `rect.height`.
    pub height: f64,
}

/// `getValueWithTolerance` (`usePopupViewport.tsx:325-340`): `positiveLabel` above the
/// tolerance, `negativeLabel` below its negation, and the empty string inside the band
/// (`:331-339`).
fn get_value_with_tolerance(
    value: f64,
    tolerance: f64,
    positive_label: &str,
    negative_label: &str,
) -> String {
    if value > tolerance {
        return positive_label.to_owned();
    }

    if value < -tolerance {
        return negative_label.to_owned();
    }

    String::new()
}

/// `getActivationDirection` (`usePopupViewport.tsx:307-313`): the space-joined
/// `"<horizontal> <vertical>"` pair, or `None` for a missing offset (`:308-310`). Both axes
/// inside the band yield `" "` — the separator with no tokens — exactly as upstream's
/// template literal does, which is why the mapping's truthiness check still emits the
/// attribute (JS `" "` is truthy).
pub fn get_activation_direction(offset: Option<Offset>) -> Option<String> {
    let offset = offset?;

    Some(format!(
        "{} {}",
        get_value_with_tolerance(
            offset.horizontal,
            ACTIVATION_DIRECTION_TOLERANCE_PX,
            "right",
            "left",
        ),
        get_value_with_tolerance(
            offset.vertical,
            ACTIVATION_DIRECTION_TOLERANCE_PX,
            "down",
            "up",
        ),
    ))
}

/// `calculateRelativePosition` (`usePopupViewport.tsx:345-362`): the new trigger's center
/// minus the previous trigger's center, per axis.
pub fn calculate_relative_position(from: ElementRect, to: ElementRect) -> Offset {
    let from_center = (
        from.left + from.width / 2.0,
        from.top + from.height / 2.0,
    );
    let to_center = (to.left + to.width / 2.0, to.top + to.height / 2.0);

    Offset {
        horizontal: to_center.0 - from_center.0,
        vertical: to_center.1 - from_center.1,
    }
}

/// Reads one element's rect into [`ElementRect`] (`:346-347`).
pub fn element_rect(element: &Element) -> ElementRect {
    let rect = element.get_bounding_client_rect();

    ElementRect {
        left: rect.left(),
        top: rect.top(),
        width: rect.width(),
        height: rect.height(),
    }
}

/// The DOM arm of `calculateRelativePosition` (`:345-347`) — the two
/// `getBoundingClientRect()` reads feeding the pure math.
pub fn calculate_relative_position_between(from: &Element, to: &Element) -> Offset {
    calculate_relative_position(element_rect(from), element_rect(to))
}

/// `popupViewportStateMapping` (`usePopupViewport.tsx:21-30`) — the viewport's activation
/// direction as [`ACTIVATION_DIRECTION`]. The `Some(None)` arm mirrors upstream's
/// `: value ? {…} : null` (`:23-29`): a falsy value emits nothing, and every other key
/// falls through to the default handling (the `None` return).
pub fn popup_viewport_state_mapping(
    key: &str,
    value: &Value,
) -> Option<Option<StateAttributeProps>> {
    if key != "activationDirection" {
        return None;
    }

    if is_truthy(value) {
        Some(Some(
            [(ACTIVATION_DIRECTION.to_string(), js_to_string(value))]
                .into_iter()
                .collect(),
        ))
    } else {
        Some(None)
    }
}

/// The `PopupViewportState` upstream builds at `:288-291` — the viewport parts' own state,
/// distinct from the popup store's.
#[derive(Clone, Debug, PartialEq)]
pub struct PopupViewportState {
    /// `activationDirection` (`:289`) — `None` while no trigger change has happened.
    pub activation_direction: Option<String>,
    /// `transitioning` (`:290`) — whether a `data-previous` container is present.
    pub transitioning: bool,
}

/// The `{ activationDirection, transitioning }` record handed to
/// [`get_state_attributes_props`] (`:288-291`), in the port's state-record shape.
pub fn popup_viewport_state_record(state: &PopupViewportState) -> Map<String, Value> {
    let mut record = Map::new();
    record.insert(
        "activationDirection".to_owned(),
        match &state.activation_direction {
            Some(direction) => Value::String(direction.clone()),
            None => Value::Null,
        },
    );
    record.insert(
        "transitioning".to_owned(),
        Value::Bool(state.transitioning),
    );
    record
}

/// The viewport element's `data-*` set:
/// `getStateAttributesProps({ activationDirection, transitioning }, popupViewportStateMapping)`.
/// The mapped key produces `data-activation-direction` with the direction as its value; the
/// `transitioning` key is default-handled into the bare `data-transitioning` marker the
/// morphing CSS selects on (`:290`; the mined spec records that attribute as
/// `MenuViewport.test.tsx:110-124`-referenced and never directly asserted).
pub fn popup_viewport_attributes(state: &PopupViewportState) -> Vec<(String, String)> {
    get_state_attributes_props(
        &popup_viewport_state_record(state),
        Some(&popup_viewport_state_mapping),
    )
    .into_iter()
    .collect()
}

/// The remount-key walk of `usePopupContentKey` (`usePopupViewport.tsx:367-396`) as a plain
/// state machine, so its two-bump behaviour is host-testable
/// (`MenuViewport.test.tsx:47-94`, `:210-273`).
pub struct PopupContentKeyState<P> {
    content_key: u32,
    previous_active_trigger_id: Option<String>,
    previous_payload: Option<P>,
    pending_payload_update: bool,
}

impl<P: PartialEq + Clone + 'static> PopupContentKeyState<P> {
    /// Seeds the walk from the hook's first render (`:368-371`).
    pub fn new(active_trigger_id: Option<String>, payload: Option<P>) -> Self {
        Self {
            content_key: 0,
            previous_active_trigger_id: active_trigger_id,
            previous_payload: payload,
            pending_payload_update: false,
        }
    }

    /// The layout-effect body (`:373-393`): a trigger-id change bumps the key immediately and
    /// remembers whether the payload lagged; a payload that then arrives bumps it once more.
    /// The comparisons are against the last committed values (`:375-378`).
    pub fn step(&mut self, active_trigger_id: Option<String>, payload: Option<P>) {
        let trigger_id_changed = active_trigger_id != self.previous_active_trigger_id;
        let payload_changed = payload != self.previous_payload;

        if trigger_id_changed {
            // `setContentKey((value) => value + 1)` + `pendingPayloadUpdateRef.current =
            // !payloadChanged` (`:380-383`).
            self.content_key = self.content_key.saturating_add(1);
            self.pending_payload_update = !payload_changed;
        } else if self.pending_payload_update && payload_changed {
            // `:384-388`.
            self.content_key = self.content_key.saturating_add(1);
            self.pending_payload_update = false;
        }

        // `:391-392`.
        self.previous_active_trigger_id = active_trigger_id;
        self.previous_payload = payload;
    }

    /// The current counter (`contentKey`).
    pub fn content_key(&self) -> u32 {
        self.content_key
    }

    /// `:395` — `${activeTriggerId ?? 'current'}-${contentKey}`.
    pub fn key(&self, active_trigger_id: Option<&str>) -> String {
        popup_content_key(active_trigger_id, self.content_key)
    }
}

/// The key string itself (`usePopupViewport.tsx:395`).
pub fn popup_content_key(active_trigger_id: Option<&str>, content_key: u32) -> String {
    format!("{}-{}", active_trigger_id.unwrap_or("current"), content_key)
}

/// `usePreviousValue(open ? activeTrigger : null)` (`usePopupViewport.tsx:88`) — the
/// last-seen value and the one before it. Upstream's Phase A hook needs `ObjectIs`, which
/// `web_sys::Element` does not implement, so the element channel carries this walk instead
/// with a caller-supplied identity (the port passes `Node::is_same_node`, JS identity for
/// nodes). `previous` is `None` until the first change — upstream's `null` sentinel
/// (`packages/utils/src/usePreviousValue.ts:10-13`).
pub struct PreviousValue<T> {
    started: bool,
    current: Option<T>,
    previous: Option<Option<T>>,
    equal: Rc<dyn Fn(&T, &T) -> bool>,
}

impl<T: Clone + 'static> PreviousValue<T> {
    /// The hook-time seed (`packages/utils/src/usePreviousValue.ts:10-13`): `current` is the
    /// first value and `previous` is still the sentinel.
    pub fn new(initial: Option<T>, equal: impl Fn(&T, &T) -> bool + 'static) -> Self {
        Self {
            started: true,
            current: initial,
            previous: None,
            equal: Rc::new(equal),
        }
    }

    /// One read (`:15-19`): equal values never advance the previous value, so the transition
    /// is idempotent; a change returns the value seen before it. The outer [`Option`] is the
    /// null sentinel ("no previous yet"); the inner one is the previous value itself, which
    /// may be the channel's own null (`PreviousValue.ts:10-13`).
    pub fn read(&mut self, value: Option<T>) -> Option<Option<T>> {
        if !self.started {
            self.started = true;
            self.current = value;
            return None;
        }

        let changed = match (&self.current, &value) {
            (Some(current), Some(value)) => !(self.equal)(current, value),
            (None, None) => false,
            _ => true,
        };

        if changed {
            let last_seen = std::mem::replace(&mut self.current, value);
            self.previous = Some(last_seen);
        }

        self.previous.clone()
    }
}

/// The trigger-change branch condition (`usePopupViewport.tsx:159-165`) as a predicate over
/// its five conjuncts, so the state walk is host-testable:
/// `activeTrigger && previousActiveTrigger && activeTrigger !== previousActiveTrigger &&
/// lastHandledTriggerRef.current !== activeTrigger && capturedNodeRef.current`.
pub fn should_begin_transition(
    has_active_trigger: bool,
    has_previous_active_trigger: bool,
    trigger_changed: bool,
    not_already_handled: bool,
    has_captured_node: bool,
) -> bool {
    has_active_trigger
        && has_previous_active_trigger
        && trigger_changed
        && not_already_handled
        && has_captured_node
}

/// The container set the engine's render output describes (`usePopupViewport.tsx:226-265`).
/// Upstream returns React nodes; the port returns the resolvable contract the view layer
/// builds from — which containers exist and what each carries. `PartialEq` is deliberately
/// not derived: [`Dimensions`] does not implement it (`floating-ui-utils-0.6.0/src/lib.rs:235`),
/// so the contract is compared field by field in the tests that need it.
#[derive(Clone, Debug)]
pub struct PopupViewportContainers {
    /// The `currentContentKey` the `data-current` container is keyed on (`:230`, `:258`) —
    /// a changed key is a remount, which is how a trigger change replaces the subtree
    /// (`MenuViewport.test.tsx:88-93`).
    pub content_key: String,
    /// `isTransitioning` (`:226`): a `data-previous` container accompanies the current one.
    pub transitioning: bool,
    /// `showStartingStyleAttribute` (`:110`): the starting-style choreography is armed.
    pub show_starting_style: bool,
    /// `previousContentDimensions` (`:105-108`) — freezes the exiting container's size.
    pub previous_dimensions: Option<Dimensions>,
}

impl PopupViewportContainers {
    /// The `data-current` container's attributes (`:230`, `:255-260`): always `data-current`,
    /// plus `data-starting-style` while the starting-style choreography is armed
    /// (`:259` — `showStartingStyleAttribute ? '' : undefined`).
    pub fn current_attributes(&self) -> Vec<(String, String)> {
        let mut attributes = vec![("data-current".to_owned(), String::new())];

        if self.show_starting_style {
            attributes.push(("data-starting-style".to_owned(), String::new()));
        }

        attributes
    }

    /// The `data-previous` container's attributes (`:237-253`), or `None` when no transition
    /// is running. The exiting container is `inert` (`:239`, upstream's
    /// `inert={inertValue(true)}`, which the port materializes as the bare attribute —
    /// `crates/leptos-ui-internals/src/floating_ui/mark_others.rs:322`), and carries
    /// `data-ending-style` exactly while the starting-style attribute is NOT armed
    /// (`:253` — the inverted condition).
    pub fn previous_attributes(&self) -> Option<Vec<(String, String)>> {
        if !self.transitioning {
            return None;
        }

        let mut attributes = vec![
            ("data-previous".to_owned(), String::new()),
            ("inert".to_owned(), String::new()),
        ];

        if inert_value(Some(true)).is_none() {
            // Unreachable with the passthrough `inertValue` port; kept as the guard the
            // omitted-attribute case would need.
            attributes.retain(|(name, _)| name != "inert");
        }

        if !self.show_starting_style {
            attributes.push(("data-ending-style".to_owned(), String::new()));
        }

        Some(attributes)
    }

    /// The `data-previous` container's inline style (`:241-251`): the frozen
    /// `--popup-width`/`--popup-height` custom properties in `"<number>px"` form when the
    /// previous content was measured, then `position: absolute`. Returned as ordered
    /// declarations (the crate's style shape).
    pub fn previous_style_declarations(&self) -> Option<Vec<(String, String)>> {
        if !self.transitioning {
            return None;
        }

        let mut declarations = Vec::new();

        if let Some(dimensions) = &self.previous_dimensions {
            declarations.push((POPUP_WIDTH.to_owned(), format!("{}px", dimensions.width)));
            declarations.push((POPUP_HEIGHT.to_owned(), format!("{}px", dimensions.height)));
        }

        declarations.push(("position".to_owned(), "absolute".to_owned()));

        Some(declarations)
    }
}

/// Whether two optional elements are the same node — upstream's `Object.is` /
/// `!==` identity for nodes (`:161-163`), which is `Node::is_same_node`.
fn same_element(left: Option<&Element>, right: Option<&Element>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.is_same_node(Some(right)),
        (None, None) => true,
        _ => false,
    }
}

/// The engine's result (`usePopupViewport.tsx:293` returns `{ children, state }`). Upstream
/// hands back rendered children; the port hands back the state and the container contract the
/// view layer renders, plus the engine's own imperative hooks (the measure pair the
/// auto-resize util drives, and the cleanup re-arm).
pub struct UsePopupViewportReturnValue {
    /// `state` (`:288-291`) — feeds [`popup_viewport_attributes`].
    pub state: Signal<PopupViewportState, LocalStorage>,
    /// The containers to render (`:226-265`).
    pub containers: Signal<PopupViewportContainers, LocalStorage>,
    /// `previousContentDimensions` (`:105-108`), written by the measure-complete callback.
    pub previous_content_dimensions: RwSignal<Option<Dimensions>>,
    /// `handleMeasureLayout` (`:119-124`) — freezes animation/transition on the current
    /// container and hides the previous one while a measurement is taken.
    pub handle_measure_layout: StableCallback<(), ()>,
    /// `handleMeasureLayoutComplete` (`:126-135`) — restores the styles and commits the
    /// measured previous dimensions.
    pub handle_measure_layout_complete: StableCallback<Option<Dimensions>, ()>,
    /// `armViewportCleanup` (`:137-146`) — watches the current container's animations and,
    /// when they finish, drops the previous content.
    pub arm_viewport_cleanup: StableCallback<(), ()>,
}

/// Port of `usePopupViewport` (`usePopupViewport.tsx:75-294`). The two element sources stand
/// in for upstream's `currentContainerRef`/`previousContainerRef` (`:98-99`): the view layer
/// owns those elements, so it supplies their readers. Must be called inside a reactive owner.
pub fn use_popup_viewport<P>(
    store: &PopupStore<P>,
    current_container: impl Fn() -> Option<HtmlElement> + 'static,
    previous_container: impl Fn() -> Option<HtmlElement> + 'static,
) -> UsePopupViewportReturnValue
where
    P: PartialEq + Clone + 'static,
{
    // The store reads the engine itself consumes (`:80-84`). `popupElement`/`positionerElement`
    // (`:85-86`) are deliberately absent — see the module docs.
    let active_trigger_element = store.use_state(popup_store::selectors::active_trigger_element);
    let active_trigger_id = store.use_state(popup_store::selectors::active_trigger_id);
    let open = store.use_state(popup_store::selectors::open);
    let payload = store.use_state(popup_store::selectors::payload);
    let mounted = store.use_state(popup_store::selectors::mounted);

    // `usePreviousValue(open ? activeTrigger : null)` (`:88`).
    let open_trigger = {
        let active_trigger_element = active_trigger_element.clone();
        let open = open.clone();
        Signal::derive_local(move || {
            if open.get() {
                active_trigger_element.get()
            } else {
                None
            }
        })
    };
    let previous_trigger = use_ref_with_init({
        let open_trigger = open_trigger.clone();
        move || {
            Rc::new(RefCell::new(PreviousValue::new(
                open_trigger.get_untracked(),
                |left: &Element, right: &Element| left.is_same_node(Some(right)),
            )))
        }
    });

    // `usePopupContentKey(activeTriggerId, payload)` (`:91`).
    let content_key_state = use_ref_with_init({
        let active_trigger_id = active_trigger_id.clone();
        let payload = payload.clone();
        move || {
            Rc::new(RefCell::new(PopupContentKeyState::new(
                active_trigger_id.get_untracked(),
                payload.get_untracked(),
            )))
        }
    });
    let content_key: RwSignal<u32> = RwSignal::new(0);
    {
        let active_trigger_id = active_trigger_id.clone();
        let payload = payload.clone();
        use_iso_layout_effect(move || {
            let trigger_id = active_trigger_id.get();
            let payload = payload.get();

            let walk = content_key_state.get_value();
            walk.borrow_mut().step(trigger_id, payload);
            let next = walk.borrow().content_key();
            if content_key.get_untracked() != next {
                content_key.set(next);
            }
        });
    }

    // `capturedNodeRef` (`:93`), `previousContentNode` (`:94`), `newTriggerOffset` (`:96`).
    let captured_node: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));
    let previous_content: RwSignal<bool> = RwSignal::new(false);
    let new_trigger_offset: RwSignal<Option<Offset>> = RwSignal::new(None);
    let show_starting_style_attribute: RwSignal<bool> = RwSignal::new(false);
    let previous_content_dimensions: RwSignal<Option<Dimensions>> = RwSignal::new(None);

    // `useAnimationsFinished(currentContainerRef, true)` (`:101`) — fed by the current
    // container source; upstream's second argument (`true`) is the
    // wait-for-starting-style-removed flag.
    let current_element_source: Rc<dyn Fn() -> Option<HtmlElement>> = Rc::new(current_container);
    let previous_element_source: Rc<dyn Fn() -> Option<HtmlElement>> =
        Rc::new(previous_container);
    let on_animations_finished = {
        // `useAnimationsFinished(currentContainerRef, true)` (`:101`): the second argument is
        // upstream's `waitForStartingStyleRemoved` default-argument slot, passed as the
        // literal `true`; the third (`batch`) is left at its `false` default.
        let wait = Signal::derive_local(|| true);
        let batch = Signal::derive_local(|| false);
        let element = {
            let current_element_source = Rc::clone(&current_element_source);
            move || {
                current_element_source().map(|element| element.unchecked_into::<Element>())
            }
        };
        use_animations_finished(element, wait, batch)
    };

    // `cleanupControllerRef` (`:103`).
    let cleanup_controller: Rc<RefCell<Option<AbortSignal>>> = Rc::new(RefCell::new(None));
    on_cleanup({
        // `on_cleanup` requires `Send + Sync`; the controller is single-threaded state, so it
        // rides the crate's `SendWrapper` (`send_wrapper` is a direct dependency and is the
        // same mechanism `ReactStore::use_state` uses for its unsubscribe handle).
        let cleanup_controller = SendWrapper::new(Rc::clone(&cleanup_controller));
        move || {
            if let Some(controller) = cleanup_controller.borrow_mut().take() {
                controller.abort();
            }
        }
    });

    // `handleMeasureLayout` (`:119-124`).
    let handle_measure_layout = use_stable_callback::<(), (), _>(Some({
        let current_element_source = Rc::clone(&current_element_source);
        let previous_element_source = Rc::clone(&previous_element_source);
        move |_| {
            if let Some(current) = (current_element_source)() {
                let style = current.style();
                let _ = style.set_property("animation", "none");
                let _ = style.set_property("transition", "none");
            }

            if let Some(previous) = (previous_element_source)() {
                let _ = previous.style().set_property("display", "none");
            }
        }
    }));

    // `handleMeasureLayoutComplete` (`:126-135`).
    let handle_measure_layout_complete =
        use_stable_callback::<Option<Dimensions>, (), _>(Some({
        let current_element_source = Rc::clone(&current_element_source);
        let previous_element_source = Rc::clone(&previous_element_source);
        move |previous_dimensions: Option<Dimensions>| {
            if let Some(current) = (current_element_source)() {
                let style = current.style();
                let _ = style.remove_property("animation");
                let _ = style.remove_property("transition");
            }

            if let Some(previous) = (previous_element_source)() {
                let _ = previous.style().remove_property("display");
            }

            if let Some(previous_dimensions) = previous_dimensions {
                previous_content_dimensions.set(Some(previous_dimensions));
            }
        }
    }));

    // `armViewportCleanup` (`:137-146`).
    let arm_viewport_cleanup = use_stable_callback::<(), (), _>(Some({
        let cleanup_controller = Rc::clone(&cleanup_controller);
        let captured_node = Rc::clone(&captured_node);
        move |_| {
            if let Some(controller) = cleanup_controller.borrow_mut().take() {
                controller.abort();
            }

            let controller = AbortSignal::new();
            let signal = controller.clone();
            *cleanup_controller.borrow_mut() = Some(controller);

            // The clone is taken per call so this closure stays `Fn` (the stable-callback
            // trampoline may invoke it repeatedly).
            let captured_node = Rc::clone(&captured_node);
            on_animations_finished.run(
                move || {
                    previous_content.set(false);
                    previous_content_dimensions.set(None);
                    *captured_node.borrow_mut() = None;
                },
                Some(&signal),
            );
        }
    }));

    // `lastHandledTriggerRef` (`:148`) and its reset (`:150-154`).
    let last_handled_trigger: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));
    {
        let last_handled_trigger = Rc::clone(&last_handled_trigger);
        let open = open.clone();
        let mounted = mounted.clone();
        use_iso_layout_effect(move || {
            if !open.get() || !mounted.get() {
                *last_handled_trigger.borrow_mut() = None;
            }
        });
    }

    // The trigger-change walk (`:156-176`).
    {
        let last_handled_trigger = Rc::clone(&last_handled_trigger);
        let captured_node = Rc::clone(&captured_node);
        let active_trigger_element = active_trigger_element.clone();
        use_iso_layout_effect(move || {
            let active = active_trigger_element.get();
            let previous = previous_trigger
                .get_value()
                .borrow_mut()
                .read(if open.get() { active.clone() } else { None })
                .flatten();

            let has_captured_node = captured_node.borrow().is_some();
            let not_already_handled = !same_element(
                last_handled_trigger.borrow().as_ref(),
                active.as_ref(),
            );

            if !should_begin_transition(
                active.is_some(),
                previous.is_some(),
                !same_element(active.as_ref(), previous.as_ref()),
                not_already_handled,
                has_captured_node,
            ) {
                return;
            }

            // `setPreviousContentNode(capturedNodeRef.current)` (`:166`).
            if let Some(captured) = &*captured_node.borrow() {
                *captured_node.borrow_mut() = Some(captured.clone());
            }
            previous_content.set(true);
            show_starting_style_attribute.set(true);

            // `calculateRelativePosition(previousActiveTrigger, activeTrigger)` (`:171`).
            if let (Some(from), Some(to)) = (previous.as_ref(), active.as_ref()) {
                new_trigger_offset.set(Some(calculate_relative_position_between(from, to)));
            }

            *last_handled_trigger.borrow_mut() = active;
        });
    }

    // The cleanup re-arm after a trigger change (`:184-202`).
    let cleanup_frame = use_animation_frame();
    {
        let cleanup_frame = cleanup_frame.clone();
        let arm_viewport_cleanup = arm_viewport_cleanup.clone();
        let cleanup_controller = Rc::clone(&cleanup_controller);
        use_iso_layout_effect(move || {
            let _ = content_key.get();

            if !previous_content.get() {
                return;
            }

            // The stale watcher is aborted synchronously (`:192`) so its microtask cannot run
            // the cleanup before the re-armed watcher below is in place.
            if let Some(controller) = cleanup_controller.borrow_mut().take() {
                controller.abort();
            }

            show_starting_style_attribute.set(true);

            // The clone keeps this effect `Fn`-callable across re-runs.
            let arm_viewport_cleanup = arm_viewport_cleanup.clone();
            cleanup_frame.request(move || {
                // `ReactDOM.flushSync(() => setShowStartingStyleAttribute(false))`
                // (`:196-199`) collapses to the synchronous signal write the crate's
                // flushSync note documents.
                show_starting_style_attribute.set(false);
                arm_viewport_cleanup.call(());
            });
        });
    }

    // The clone capture (`:206-224`) — see the module docs for the tracked adaptation.
    {
        let current_element_source = Rc::clone(&current_element_source);
        let captured_node = Rc::clone(&captured_node);
        let content_key = content_key;
        use_iso_layout_effect(move || {
            let _ = content_key.get();
            let _ = previous_content.get();

            let Some(source) = (current_element_source)() else {
                return;
            };

            let document = owner_document(Some(source.as_ref()));
            let Ok(wrapper) = document.create_element("div") else {
                return;
            };

            let children = source.child_nodes();
            for index in 0..children.length() {
                let Some(child) = children.item(index) else {
                    continue;
                };
                if let Ok(clone) = child.clone_node_with_deep(true) {
                    let _ = wrapper.append_child(&clone);
                }
            }

            *captured_node.borrow_mut() = Some(wrapper);
        });
    }

    // The previous container's imperative population (`:268-275`).
    {
        let previous_element_source = Rc::clone(&previous_element_source);
        let captured_node = Rc::clone(&captured_node);
        use_iso_layout_effect(move || {
            if !previous_content.get() {
                return;
            }

            let Some(container) = (previous_element_source)() else {
                return;
            };
            let Some(clone) = captured_node.borrow().clone() else {
                return;
            };

            let children = clone.child_nodes();
            let moved = js_sys::Array::new();
            for index in 0..children.length() {
                if let Some(child) = children.item(index) {
                    let value: &JsValue = child.as_ref();
                    moved.push(value);
                }
            }
            container.replace_children_with_node(&moved);
        });
    }

    // `state` (`:288-291`).
    let state = Signal::derive_local(move || PopupViewportState {
        activation_direction: get_activation_direction(new_trigger_offset.get()),
        transitioning: previous_content.get(),
    });

    // The render output (`:226-265`) as the container contract.
    let containers = Signal::derive_local(move || PopupViewportContainers {
        content_key: popup_content_key(
            active_trigger_id.get().as_deref(),
            content_key.get(),
        ),
        transitioning: previous_content.get(),
        show_starting_style: show_starting_style_attribute.get(),
        previous_dimensions: previous_content_dimensions.get(),
    });

    UsePopupViewportReturnValue {
        state,
        containers,
        previous_content_dimensions,
        handle_measure_layout,
        handle_measure_layout_complete,
        arm_viewport_cleanup,
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use serde_json::json;

    use super::*;

    fn rect(left: f64, top: f64, width: f64, height: f64) -> ElementRect {
        ElementRect {
            left,
            top,
            width,
            height,
        }
    }

    // `getValueWithTolerance` (`:325-340`): strictly beyond the tolerance picks the label,
    // the boundary itself and everything inside it yields the empty token (`:331-339`).
    #[test]
    fn the_tolerance_band_yields_an_empty_token() {
        assert_eq!(get_value_with_tolerance(6.0, 5.0, "right", "left"), "right");
        assert_eq!(get_value_with_tolerance(-6.0, 5.0, "right", "left"), "left");
        assert_eq!(get_value_with_tolerance(5.0, 5.0, "right", "left"), "");
        assert_eq!(get_value_with_tolerance(-5.0, 5.0, "right", "left"), "");
        assert_eq!(get_value_with_tolerance(0.0, 5.0, "right", "left"), "");
        assert_eq!(get_value_with_tolerance(2.0, 5.0, "right", "left"), "");
    }

    // The full direction matrix: each axis independently contributes its token, both axes
    // compose in `"<horizontal> <vertical>"` order (`:312`), and a missing offset has no
    // direction at all (`:308-310`).
    #[test]
    fn the_activation_direction_composes_both_axes() {
        assert_eq!(get_activation_direction(None), None);
        assert_eq!(
            get_activation_direction(Some(Offset {
                horizontal: 40.0,
                vertical: 40.0,
            })),
            Some("right down".to_owned())
        );
        assert_eq!(
            get_activation_direction(Some(Offset {
                horizontal: -40.0,
                vertical: -40.0,
            })),
            Some("left up".to_owned())
        );
        assert_eq!(
            get_activation_direction(Some(Offset {
                horizontal: 40.0,
                vertical: -40.0,
            })),
            Some("right up".to_owned())
        );
        // One axis inside the band: only the other axis carries a token (`:312`).
        assert_eq!(
            get_activation_direction(Some(Offset {
                horizontal: 0.0,
                vertical: 40.0,
            })),
            Some(" down".to_owned())
        );
        assert_eq!(
            get_activation_direction(Some(Offset {
                horizontal: 40.0,
                vertical: 0.0,
            })),
            Some("right ".to_owned())
        );
    }

    // `MenuViewport.test.tsx:289-305`'s edge case, at the port level: a 2px offset on an axis
    // omits that axis's token, and 2px on both axes leaves the separator with no tokens. The
    // string is not empty, so the mapping still emits the attribute (JS truthiness).
    #[test]
    fn offsets_inside_the_tolerance_omit_their_axis_token() {
        let two_px_vertical = get_activation_direction(Some(Offset {
            horizontal: 40.0,
            vertical: 2.0,
        }));
        assert_eq!(two_px_vertical, Some("right ".to_owned()));

        let both_inside = get_activation_direction(Some(Offset {
            horizontal: 2.0,
            vertical: 2.0,
        }));
        assert_eq!(both_inside, Some(" ".to_owned()));
        assert_eq!(
            popup_viewport_attributes(&PopupViewportState {
                activation_direction: both_inside,
                transitioning: false,
            }),
            vec![("data-activation-direction".to_owned(), " ".to_owned())]
        );
    }

    // `calculateRelativePosition` (`:345-362`): the new trigger's center minus the previous
    // trigger's center — a pure translation, so identical rects give a zero offset.
    #[test]
    fn the_relative_position_is_center_to_center() {
        assert_eq!(
            calculate_relative_position(rect(0.0, 0.0, 10.0, 10.0), rect(0.0, 0.0, 10.0, 10.0)),
            Offset {
                horizontal: 0.0,
                vertical: 0.0,
            }
        );
        assert_eq!(
            calculate_relative_position(
                rect(0.0, 0.0, 100.0, 20.0),
                rect(200.0, 50.0, 20.0, 100.0)
            ),
            Offset {
                horizontal: 160.0,
                vertical: 90.0,
            }
        );
    }

    // The mapping (`:21-30`): the attribute carries the direction string as its value, a
    // falsy value emits nothing, and any other key falls through (`None`).
    #[test]
    fn the_viewport_mapping_emits_only_the_activation_direction() {
        assert_eq!(
            popup_viewport_state_mapping("activationDirection", &json!("right down"))
                .expect("mapped")
                .expect("props"),
            [("data-activation-direction".to_owned(), "right down".to_owned())]
                .into_iter()
                .collect()
        );
        assert_eq!(
            popup_viewport_state_mapping("activationDirection", &Value::Null),
            Some(None)
        );
        assert_eq!(
            popup_viewport_state_mapping("activationDirection", &json!("")),
            Some(None)
        );
        assert_eq!(popup_viewport_state_mapping("transitioning", &json!(true)), None);
    }

    // The state record's default handling: `transitioning` is not mapped, so it becomes the
    // bare `data-transitioning` marker the morphing CSS selects on; the direction rides the
    // mapping. An absent direction contributes nothing.
    #[test]
    fn the_state_record_emits_the_transitioning_marker_and_the_direction() {
        assert_eq!(
            popup_viewport_attributes(&PopupViewportState {
                activation_direction: Some("left up".to_owned()),
                transitioning: true,
            }),
            vec![
                (
                    "data-activation-direction".to_owned(),
                    "left up".to_owned()
                ),
                ("data-transitioning".to_owned(), String::new()),
            ]
        );

        assert_eq!(
            popup_viewport_attributes(&PopupViewportState {
                activation_direction: None,
                transitioning: false,
            }),
            Vec::new()
        );
    }

    // The content-key walk (`:367-396`): a trigger change bumps the key, and a payload that
    // lags one render bumps it once more (the documented `<img>` reuse guard, `:381-388`).
    #[test]
    fn a_lagging_payload_bumps_the_content_key_twice() {
        let mut walk = PopupContentKeyState::new(Some("t1".to_owned()), Some("p1".to_owned()));
        assert_eq!(walk.key(Some("t1")), "t1-0");

        // Trigger changes, payload has not caught up yet: one bump, pending remembered.
        walk.step(Some("t2".to_owned()), Some("p1".to_owned()));
        assert_eq!(walk.content_key(), 1);
        assert_eq!(walk.key(Some("t2")), "t2-1");

        // The payload arrives a render later: the second bump.
        walk.step(Some("t2".to_owned()), Some("p2".to_owned()));
        assert_eq!(walk.content_key(), 2);
        assert_eq!(walk.key(Some("t2")), "t2-2");

        // Nothing further changes: the key is stable (no spurious remounts).
        walk.step(Some("t2".to_owned()), Some("p2".to_owned()));
        assert_eq!(walk.content_key(), 2);
    }

    // A payload arriving in the same step as the trigger change needs no second bump, and a
    // payload-only change does not remount at all (`:380-388`).
    #[test]
    fn only_a_trigger_change_or_a_lagging_payload_remounts() {
        let mut together = PopupContentKeyState::new(Some("t1".to_owned()), Some("p1".to_owned()));
        together.step(Some("t2".to_owned()), Some("p2".to_owned()));
        assert_eq!(together.content_key(), 1);

        let mut payload_only =
            PopupContentKeyState::new(Some("t1".to_owned()), Some("p1".to_owned()));
        payload_only.step(Some("t1".to_owned()), Some("p2".to_owned()));
        assert_eq!(payload_only.content_key(), 0);
    }

    // The key string (`:395`): the null trigger id falls back to `current`.
    #[test]
    fn the_content_key_falls_back_to_current() {
        assert_eq!(popup_content_key(None, 0), "current-0");
        assert_eq!(popup_content_key(Some("t3"), 4), "t3-4");
    }

    // The trigger-change predicate (`:159-165`): all five conjuncts are required, so a mount
    // (no previous trigger), an unrelated re-render (already handled), and a change with no
    // captured clone each decline to start a transition.
    #[test]
    fn the_transition_branch_requires_all_five_conjuncts() {
        assert!(should_begin_transition(true, true, true, true, true));
        assert!(!should_begin_transition(false, true, true, true, true));
        assert!(!should_begin_transition(true, false, true, true, true));
        assert!(!should_begin_transition(true, true, false, true, true));
        assert!(!should_begin_transition(true, true, true, false, true));
        assert!(!should_begin_transition(true, true, true, true, false));
    }

    // `PreviousValue` (`:88`): the first read is the hook-time seed and returns the null
    // sentinel — which is why a mount never starts a transition; equal reads never advance;
    // a change returns the value seen before it.
    #[test]
    fn the_previous_value_walk_returns_the_last_seen_value_on_change() {
        let mut walk = PreviousValue::new(
            Some("t1".to_owned()),
            |left: &String, right: &String| left == right,
        );

        assert_eq!(walk.read(Some("t1".to_owned())), None);
        assert_eq!(walk.read(Some("t1".to_owned())), None);
        assert_eq!(
            walk.read(Some("t2".to_owned())),
            Some(Some("t1".to_owned()))
        );
        assert_eq!(
            walk.read(Some("t2".to_owned())),
            Some(Some("t1".to_owned()))
        );
    }

    // The closing arm: `open ? activeTrigger : null` (`:88`) feeds `null` on close, so the
    // reopen after a close sees the null sentinel as its previous trigger — a reopen is not a
    // trigger-to-trigger morph.
    #[test]
    fn the_closed_channel_reports_the_null_sentinel() {
        let mut walk = PreviousValue::new(
            Some("t1".to_owned()),
            |left: &String, right: &String| left == right,
        );

        assert_eq!(walk.read(None), Some(Some("t1".to_owned())));
        assert_eq!(walk.read(None), Some(Some("t1".to_owned())));
        assert_eq!(
            walk.read(Some("t2".to_owned())),
            Some(None),
            "the closed channel's null is the previous value"
        );
    }

    // The container contract (`:226-265`). Not transitioning: a `data-current` container only,
    // keyed, with no starting-style choreography and no previous container at all.
    #[test]
    fn the_quiet_container_carries_only_data_current() {
        let containers = PopupViewportContainers {
            content_key: "t1-0".to_owned(),
            transitioning: false,
            show_starting_style: false,
            previous_dimensions: None,
        };

        assert_eq!(
            containers.current_attributes(),
            vec![("data-current".to_owned(), String::new())]
        );
        assert_eq!(containers.previous_attributes(), None);
        assert_eq!(containers.previous_style_declarations(), None);
    }

    // Mid-transition (`:235-265`): the exiting container is `data-previous` + `inert`,
    // carries the frozen size vars in `"<number>px"` form and `position: absolute`, and the
    // two style hooks are exact inverses — `data-starting-style` on the current container
    // while the previous one omits `data-ending-style`, and the reverse once disarmed.
    #[test]
    fn the_transitioning_containers_carry_the_frozen_size_and_the_style_hooks() {
        let armed = PopupViewportContainers {
            content_key: "t2-1".to_owned(),
            transitioning: true,
            show_starting_style: true,
            previous_dimensions: Some(Dimensions {
                width: 240.0,
                height: 96.0,
            }),
        };

        assert_eq!(
            armed.current_attributes(),
            vec![
                ("data-current".to_owned(), String::new()),
                ("data-starting-style".to_owned(), String::new()),
            ]
        );
        assert_eq!(
            armed.previous_attributes(),
            Some(vec![
                ("data-previous".to_owned(), String::new()),
                ("inert".to_owned(), String::new()),
            ]),
            "data-ending-style is omitted while the starting style is armed (:253)"
        );
        assert_eq!(
            armed.previous_style_declarations(),
            Some(vec![
                ("--popup-width".to_owned(), "240px".to_owned()),
                ("--popup-height".to_owned(), "96px".to_owned()),
                ("position".to_owned(), "absolute".to_owned()),
            ])
        );

        let disarmed = PopupViewportContainers {
            show_starting_style: false,
            ..armed
        };
        assert_eq!(
            disarmed.current_attributes(),
            vec![("data-current".to_owned(), String::new())]
        );
        assert_eq!(
            disarmed.previous_attributes(),
            Some(vec![
                ("data-previous".to_owned(), String::new()),
                ("inert".to_owned(), String::new()),
                ("data-ending-style".to_owned(), String::new()),
            ])
        );
    }

    // An unmeasured previous content still gets the absolute exit position but no size vars
    // (`:243-249` — the conditional spread).
    #[test]
    fn an_unmeasured_previous_content_carries_no_size_vars() {
        let containers = PopupViewportContainers {
            content_key: "t2-1".to_owned(),
            transitioning: true,
            show_starting_style: true,
            previous_dimensions: None,
        };

        assert_eq!(
            containers.previous_style_declarations(),
            Some(vec![("position".to_owned(), "absolute".to_owned())])
        );
    }
}
