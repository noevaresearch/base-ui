//! Port of `packages/react/src/utils/useSwipeDismiss.ts` — the shared pointer/touch
//! swipe-to-dismiss gesture engine behind the Drawer's two surfaces (the popup's close
//! engine and the SwipeArea's open engine) and Toast's per-toast dismissal
//! (`specs/library/utils/implementation.md`, "Swipe-to-dismiss (`useSwipeDismiss.ts`)";
//! behavior.md, State model → `useSwipeDismiss`, and Keyboard interactions → DOM structure
//! & portal behavior). This is the subsystem the `infra: utils` unit's own spec documents
//! and the reason `library: drawer` / `library: toast` name
//! `utils/useSwipeDismiss` as their gesture dependency
//! (`specs/library/drawer/implementation.md`, "Dependencies on other Base UI internals").
//!
//! Upstream React state is deliberately minimal — `currentSwipeDirection`, `isSwiping`,
//! `dragDismissed` (`useSwipeDismiss.ts:131-135`) — and everything else (start position,
//! offsets, baselines, thresholds, velocity samples, pointer-capture bookkeeping) lives in
//! ~25 refs (`:137-162`) so gesture math never triggers renders.
//!
//! ## Rust adaptations
//!
//! - The ~25 refs become fields of one DOM-free [`GestureState`], and the three `useState`
//!   slots become the returned signals ([`UseSwipeDismissReturn::swiping`],
//!   [`UseSwipeDismissReturn::swipe_direction`],
//!   [`UseSwipeDismissReturn::drag_dismissed`]). Upstream mirrors `isSwiping` into
//!   `isSwipingRef` precisely so the imperative paths never read the lagging state; the port
//!   keeps the same split (`GestureState::is_swiping` for the machine, the signal for the
//!   consumer), the `getDragStyles` note at `:1002-1005` being the reason.
//! - Every handler upstream wraps in `useStableCallback` (`:164`, `:190`, `:220`, `:527`,
//!   `:748`, `:862`, `:992`) is a plain `Rc` closure here: the hook body runs once, so
//!   stable identity is structural rather than memoized — the `usePressAndHold` adaptation.
//! - React's synthetic events are the DOM events themselves in the port, so
//!   `event.nativeEvent` *is* the event; [`SwipeNativeEvent`] is the `PointerEvent |
//!   TouchEvent` union the callbacks receive, and the internal duck-typed
//!   `SwipeDismissNativeTouchMove` shape (`:13-19`) is [`SwipeInput`] with
//!   [`SwipeInputKind::NativeTouchMove`].
//! - `event.currentTarget` (`:576`, used only as the boundary element for the
//!   scrollable-target walk on touches) is the element the returned props are spread on,
//!   which is `elementRef.current` in every consumer — resolved through the element source
//!   (`:570-580`), the `usePressAndHold` convention.
//! - `elementRef` (`:90`) is the element-source closure, read at each use (`:179`, `:223`,
//!   `:324`, `:366`, `:382`, `:455`, `:774`, `:921`).
//! - The DOM-free part of the machine (all of the geometry — damping, latching, the
//!   reverse-cancel rule, progress and velocity math, the dismissal decision) lives in
//!   [`GestureState`] so it is exercised on the host target; the DOM effects
//!   (`getElementTransform`, hit-testing, scrollable walks, pointer capture, imperative
//!   style writes) stay beside their upstream call sites and are covered by the in-browser
//!   `wasm_bindgen_test` suite below, the `usePressAndHold` precedent.
//! - `getPointerProps`/`getTouchProps` return `{}` when disabled upstream (`:1029-1041`,
//!   `:1042-1053`); the port always returns the bags and makes each handler inert instead
//!   (`enabled` is read untracked per invocation, the `usePressAndHold` `disabled` note).
//! - `'use client'` (`:1`) is N/A — there is no React Server Components boundary in Rust.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{GetUntracked, Set};
use web_sys::wasm_bindgen::JsCast;
use web_sys::{CssStyleDeclaration, Element, EventTarget, HtmlElement, PointerEvent, TouchEvent};

use leptos_ui_utils::clamp::clamp;
use leptos_ui_utils::owner::owner_document;
use leptos_ui_utils::shadow_dom::{contains, get_target};

use crate::floating_ui::element_props::ElementEventHandler;
use crate::get_element_at_point::get_element_at_point;
use crate::get_element_transform::{ElementTransform, get_element_transform};
use crate::scrollable::{ScrollAxis, find_scrollable_touch_target, has_scrollable_ancestor};

/// `DEFAULT_SWIPE_THRESHOLD` (`useSwipeDismiss.ts:31`).
const DEFAULT_SWIPE_THRESHOLD: f64 = 40.0;
/// `REVERSE_CANCEL_THRESHOLD` (`:32`).
const REVERSE_CANCEL_THRESHOLD: f64 = 10.0;
/// `MIN_DRAG_THRESHOLD` (`:33`).
const MIN_DRAG_THRESHOLD: f64 = 1.0;
/// `MIN_VELOCITY_DURATION_MS` (`:34`).
const MIN_VELOCITY_DURATION_MS: f64 = 50.0;
/// `MIN_RELEASE_VELOCITY_DURATION_MS` (`:35`).
const MIN_RELEASE_VELOCITY_DURATION_MS: f64 = 16.0;
/// `MAX_RELEASE_VELOCITY_AGE_MS` (`:36`).
const MAX_RELEASE_VELOCITY_AGE_MS: f64 = 80.0;
/// `DEFAULT_IGNORE_SELECTOR` (`:37`).
const DEFAULT_IGNORE_SELECTOR: &str = "button,a,input,select,textarea,label,[role=\"button\"]";

/// `SwipeDirection` (`:11`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

impl SwipeDirection {
    /// The lowercase union member spelling (`:11`), as used for data attributes.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// A pointer position in CSS pixels — upstream's `{ x, y }` literals (`:137-141`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct SwipePoint {
    pub x: f64,
    pub y: f64,
}

impl SwipePoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// `SwipeProgressDetailsInternal` (`:25-29`) — the optional second argument of
/// `onProgress` (`:216`) and the payload deduped in `updateSwipeProgress` (`:196-204`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwipeProgressDetails {
    pub delta_x: f64,
    pub delta_y: f64,
    pub direction: Option<SwipeDirection>,
}

/// The `onRelease` payload (`:1124-1133`), minus the event (passed alongside, since the
/// event is a reference-counted handle rather than a field of a JS object literal).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwipeReleaseDetails {
    pub direction: Option<SwipeDirection>,
    pub delta_x: f64,
    pub delta_y: f64,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub release_velocity_x: f64,
    pub release_velocity_y: f64,
}

/// `UseSwipeDismissDetails` (`:1069-1072`) — the second argument of `canStart`
/// (`:551-554`, `:910-913`).
#[derive(Clone)]
pub struct UseSwipeDismissDetails {
    pub native_event: SwipeNativeEvent,
    pub direction: Option<SwipeDirection>,
}

/// The `PointerEvent | TouchEvent` union the public callbacks receive
/// (`SwipeDismissNativeEvent`, `:21`).
#[derive(Clone)]
pub enum SwipeNativeEvent {
    Pointer(PointerEvent),
    Touch(TouchEvent),
}

impl SwipeNativeEvent {
    /// `event.preventDefault()` on the native event (`:584`).
    pub fn prevent_default(&self) {
        match self {
            Self::Pointer(event) => {
                event.prevent_default();
            }
            Self::Touch(event) => {
                event.prevent_default();
            }
        }
    }

    /// `event.timeStamp` (`:395`, `:598`, `:722`, `:784`) — for the touch-move wrapper the
    /// native `TouchEvent` carries it (`:998`).
    pub fn time_stamp(&self) -> f64 {
        match self {
            Self::Pointer(event) => event.time_stamp(),
            Self::Touch(event) => event.time_stamp(),
        }
    }

    /// `event.defaultPrevented` (`:532`, `:903`) — checked on both the synthetic event and
    /// its `nativeEvent`, which coincide in the port.
    pub fn default_prevented(&self) -> bool {
        match self {
            Self::Pointer(event) => event.default_prevented(),
            Self::Touch(event) => event.default_prevented(),
        }
    }

    /// `getTarget(nativeEvent)` (`:326`, `:574`).
    fn target(&self) -> Option<EventTarget> {
        match self {
            Self::Pointer(event) => get_target(event.as_ref() as &web_sys::Event),
            Self::Touch(event) => get_target(event.as_ref() as &web_sys::Event),
        }
    }
}

/// `movementCssVars` (`:91`, `:1080`) — the two custom properties the hook writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementCssVars {
    pub x: String,
    pub y: String,
}

/// `swipeThreshold` (`:1086-1087`): a pixel count, or a resolver called with the element
/// and the latched direction.
#[derive(Clone)]
pub enum SwipeThreshold {
    Pixels(f64),
    Function(Rc<dyn Fn(&HtmlElement, SwipeDirection) -> f64>),
}

/// `getDisplacement` (`:39-52`): the signed displacement of a delta pair along
/// `direction`.
pub fn get_displacement(direction: SwipeDirection, delta_x: f64, delta_y: f64) -> f64 {
    match direction {
        SwipeDirection::Up => -delta_y,
        SwipeDirection::Down => delta_y,
        SwipeDirection::Left => -delta_x,
        SwipeDirection::Right => delta_x,
    }
}

/// `getValidTimeStamp` (`:54-56`).
fn get_valid_time_stamp(time_stamp: f64) -> Option<f64> {
    if time_stamp.is_finite() && time_stamp > 0.0 {
        Some(time_stamp)
    } else {
        None
    }
}

/// `getDragTransform` (`:58-60`).
fn get_drag_transform(drag_offset: SwipePoint, scale: f64) -> String {
    format!(
        "translate3d({}px,{}px,0) scale({scale})",
        format_number(drag_offset.x),
        format_number(drag_offset.y)
    )
}

/// JS number-to-string for the style values interpolated above (and the `${delta}px`
/// movement vars at `:251-252`): `40` not `40.0`. Rust's `Display` for `f64` already
/// prints the shortest round-trippable form, which matches `String(n)` for gesture
/// magnitudes — this helper only exists so the intent is explicit at both sites.
fn format_number(value: f64) -> String {
    format!("{value}")
}

/// JS `Math.sign` (`:470`): unlike Rust's `f64::signum`, `sign(0) == 0` and `sign(NaN)`
/// is `NaN`.
fn js_sign(value: f64) -> f64 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        value
    }
}

/// `hasPrimaryMouseButton` (`:62-64`).
fn has_primary_mouse_button(buttons: u16) -> bool {
    buttons % 2 == 1
}

/// The duck-typed move-event kinds (`:21-24`): a plain pointer event, a React touch
/// event, or the `SwipeDismissNativeTouchMove` shape fed by [`moveNative`] (`:992-1000`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SwipeInputKind {
    Pointer,
    Touch,
    NativeTouchMove,
}

/// The internal view of a start/move/end event — upstream's `'touches' in event`,
/// `event.clientX/Y`, `event.buttons`, `event.pointerId`, `event.timeStamp`,
/// `event.defaultPrevented` and `event.nativeEvent` reads, bundled.
#[derive(Clone)]
struct SwipeInput {
    kind: SwipeInputKind,
    pointer_type_touch: bool,
    button: i16,
    buttons: u16,
    pointer_id: i32,
    position: Option<SwipePoint>,
    time_stamp: f64,
    default_prevented: bool,
    native: SwipeNativeEvent,
}

impl SwipeInput {
    /// `isTouchLikeEvent` (`:314-321`).
    fn is_touch_like(&self) -> bool {
        self.kind != SwipeInputKind::Pointer || self.pointer_type_touch
    }

    /// `'touches' in event` (`:306`, `:411`, `:536`, `:548`, `:582`, `:776`, `:870`, …).
    fn has_touches(&self) -> bool {
        self.kind != SwipeInputKind::Pointer
    }

    fn from_pointer(event: &PointerEvent) -> Self {
        Self {
            kind: SwipeInputKind::Pointer,
            pointer_type_touch: event.pointer_type() == "touch",
            button: event.button(),
            buttons: event.buttons(),
            pointer_id: event.pointer_id(),
            position: Some(SwipePoint::new(
                event.client_x() as f64,
                event.client_y() as f64,
            )),
            time_stamp: event.time_stamp(),
            default_prevented: event.default_prevented(),
            native: SwipeNativeEvent::Pointer(event.clone()),
        }
    }

    /// `getPrimaryPointerPosition` for a touch event (`:306-308`): `touches[0]`, `null`
    /// when the list is empty (which is what `touchend` provides).
    fn from_touch(event: &TouchEvent) -> Self {
        let touch = event.touches().get(0);
        Self {
            kind: SwipeInputKind::Touch,
            pointer_type_touch: false,
            button: 0,
            buttons: 0,
            pointer_id: 0,
            position: touch
                .as_ref()
                .map(|touch| SwipePoint::new(touch.client_x() as f64, touch.client_y() as f64)),
            time_stamp: event.time_stamp(),
            default_prevented: event.default_prevented(),
            native: SwipeNativeEvent::Touch(event.clone()),
        }
    }

    /// The `moveNative` wrapper (`:992-1000`): a native `TouchEvent` re-shaped as the
    /// `'touches' in event` branch, with `currentTarget` supplied by the caller.
    fn from_native_touch_move(event: &TouchEvent, _current_target: &HtmlElement) -> Self {
        Self {
            kind: SwipeInputKind::NativeTouchMove,
            ..Self::from_touch(event)
        }
    }
}

/// `lockedDirectionRef` (`:145`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LockedAxis {
    Horizontal,
    Vertical,
}

/// The static per-invocation configuration derived from the props (`:105-129`).
#[derive(Clone)]
struct GestureConfig {
    directions: Vec<SwipeDirection>,
    primary_direction: Option<SwipeDirection>,
    allow_left: bool,
    allow_right: bool,
    allow_up: bool,
    allow_down: bool,
    has_horizontal: bool,
    has_vertical: bool,
    scroll_axes: Vec<ScrollAxis>,
    swipe_threshold_default: f64,
    ignore_selector_when_touch: bool,
    ignore_scrollable_ancestors: bool,
    track_drag: bool,
}

impl GestureConfig {
    fn new(
        directions: Vec<SwipeDirection>,
        threshold: Option<&SwipeThreshold>,
        ignore_selector_when_touch: bool,
        ignore_scrollable_ancestors: bool,
        track_drag: bool,
    ) -> Self {
        let primary_direction = if directions.len() == 1 {
            directions.first().copied()
        } else {
            None
        };
        let allow_left = directions.contains(&SwipeDirection::Left);
        let allow_right = directions.contains(&SwipeDirection::Right);
        let allow_up = directions.contains(&SwipeDirection::Up);
        let allow_down = directions.contains(&SwipeDirection::Down);
        let has_horizontal = allow_left || allow_right;
        let has_vertical = allow_up || allow_down;

        // `scrollAxes` (`:120-129`): vertical first, then horizontal.
        let mut scroll_axes = Vec::new();
        if has_vertical {
            scroll_axes.push(ScrollAxis::Vertical);
        }
        if has_horizontal {
            scroll_axes.push(ScrollAxis::Horizontal);
        }

        // `swipeThresholdDefault` (`:108-111`): a non-numeric prop falls back to 40.
        let swipe_threshold_default = match threshold {
            Some(SwipeThreshold::Pixels(value)) => value.max(0.0),
            _ => DEFAULT_SWIPE_THRESHOLD,
        };

        Self {
            directions,
            primary_direction,
            allow_left,
            allow_right,
            allow_up,
            allow_down,
            has_horizontal,
            has_vertical,
            scroll_axes,
            swipe_threshold_default,
            ignore_selector_when_touch,
            ignore_scrollable_ancestors,
            track_drag,
        }
    }
}

/// The DOM-free gesture machine: the ~25 refs at `:137-162` plus the imperative paths
/// that read them. Every method here follows its cited upstream counterpart; the caller
/// (the hook body below) performs the DOM reads/writes and the consumer callbacks in the
/// same order upstream does.
#[derive(Clone, Debug)]
struct GestureState {
    drag_start_pos: SwipePoint,
    drag_offset: SwipePoint,
    last_move_pos: Option<SwipePoint>,
    initial_transform: ElementTransform,
    intended_swipe_direction: Option<SwipeDirection>,
    max_swipe_displacement: f64,
    cancelled_swipe: bool,
    swipe_cancel_baseline: SwipePoint,
    locked_direction: Option<LockedAxis>,
    is_first_pointer_move: bool,
    pending_swipe: bool,
    pending_swipe_start_pos: Option<SwipePoint>,
    swipe_from_scrollable: bool,
    saw_primary_buttons_on_move: bool,
    element_size: SwipePoint,
    swipe_progress: f64,
    swipe_threshold: f64,
    swipe_start_time: Option<f64>,
    last_drag_sample: Option<(SwipePoint, f64)>,
    last_drag_velocity: SwipePoint,
    last_progress_details: Option<SwipeProgressDetails>,
    is_swiping: bool,
    drag_style_snapshot: Option<(String, String)>,
}

impl Default for GestureState {
    fn default() -> Self {
        Self {
            drag_start_pos: SwipePoint::default(),
            drag_offset: SwipePoint::default(),
            last_move_pos: None,
            // `initialTransformRef = { x: 0, y: 0, scale: 1 }` (`:140`, `:283`).
            initial_transform: ElementTransform {
                x: 0.0,
                y: 0.0,
                scale: 1.0,
            },
            intended_swipe_direction: None,
            max_swipe_displacement: 0.0,
            cancelled_swipe: false,
            swipe_cancel_baseline: SwipePoint::default(),
            locked_direction: None,
            is_first_pointer_move: false,
            pending_swipe: false,
            pending_swipe_start_pos: None,
            swipe_from_scrollable: false,
            saw_primary_buttons_on_move: false,
            element_size: SwipePoint::default(),
            swipe_progress: 0.0,
            swipe_threshold: DEFAULT_SWIPE_THRESHOLD,
            swipe_start_time: None,
            last_drag_sample: None,
            last_drag_velocity: SwipePoint::default(),
            last_progress_details: None,
            is_swiping: false,
            drag_style_snapshot: None,
        }
    }
}

/// What a move must report back to the hook so it can run the DOM/consumer effects in
/// upstream's order.
#[derive(Debug, Default)]
struct MoveOutcome {
    /// `setCurrentSwipeDirection(...)` calls (`:667`, `:676`), in order.
    direction_updates: Vec<Option<SwipeDirection>>,
    /// `resolveSwipeThreshold(candidate)` (`:668`) — the latched direction to resolve.
    resolve_threshold_for: Option<SwipeDirection>,
    /// `if (offsetChanged) syncDragStyles(true)` (`:719-721`).
    offset_changed: bool,
    /// `updateSwipeProgress(progress, details)` (`:745-745`), already deduped.
    progress_report: Option<(f64, Option<SwipeProgressDetails>)>,
}

/// The release measurements (`:753-811`), computed before `onRelease` runs.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ReleaseMeasurements {
    progress_details: SwipeProgressDetails,
    velocity_x: f64,
    velocity_y: f64,
    release_velocity_x: f64,
    release_velocity_y: f64,
}

/// What the release path must do after `onRelease` has had its say (`:825-859`).
#[derive(Clone, Copy, Debug, PartialEq)]
enum ReleaseAction {
    /// `!isSwipingRef.current` (`:763-767`).
    NotSwiping {
        progress_report: Option<(f64, Option<SwipeProgressDetails>)>,
    },
    /// The latched change-of-mind, unoverridden (`:825-831`).
    Cancelled {
        progress_report: Option<(f64, Option<SwipeProgressDetails>)>,
    },
    /// `shouldClose && dismissDirection` (`:849-853`).
    Dismiss { direction: SwipeDirection },
    /// The snap-back else branch (`:854-859`).
    SnapBack {
        progress_report: Option<(f64, Option<SwipeProgressDetails>)>,
    },
}

impl GestureState {
    fn new(config: &GestureConfig) -> Self {
        Self {
            swipe_threshold: config.swipe_threshold_default,
            ..Default::default()
        }
    }

    /// `setSwiping` (`:164-172`): returns `true` when the value actually flipped, so the
    /// hook mirrors it into the signal and fires `onSwipingChange` only then.
    fn set_swiping(&mut self, next: bool) -> bool {
        if self.is_swiping == next {
            return false;
        }
        self.is_swiping = next;
        true
    }

    /// `updateSwipeProgress` (`:190-218`) including its dedupe: `Some((progress, details))`
    /// when `onProgress` must fire, `None` when the write was a no-op. A non-finite
    /// `progress` clamps to `0` (`:192`).
    fn update_swipe_progress(
        &mut self,
        progress: f64,
        details: Option<SwipeProgressDetails>,
    ) -> Option<(f64, Option<SwipeProgressDetails>)> {
        let next_progress = if progress.is_finite() {
            clamp(progress, 0.0, 1.0)
        } else {
            0.0
        };
        let progress_changed = next_progress != self.swipe_progress;
        let mut details_changed = false;

        if let Some(details) = details {
            let last_details = self.last_progress_details;
            details_changed = match last_details {
                None => true,
                Some(last) => {
                    last.delta_x != details.delta_x
                        || last.delta_y != details.delta_y
                        || last.direction != details.direction
                }
            };
        }

        if !progress_changed && !details_changed {
            return None;
        }

        self.swipe_progress = next_progress;
        if let Some(details) = details {
            self.last_progress_details = Some(details);
        } else if progress_changed {
            self.last_progress_details = None;
        }
        Some((next_progress, details))
    }

    /// `syncDragStyles` (`:220-253`) — the state half: whether this call owns the style
    /// snapshot, and the transform/var values to write. The hook performs the writes.
    fn sync_drag_styles_plan(
        &mut self,
        config: &GestureConfig,
        swiping: bool,
        element_present: bool,
        current_transition: &str,
        current_transform: &str,
    ) -> Option<DragStylePlan> {
        if !config.track_drag || !element_present {
            if !swiping {
                self.drag_style_snapshot = None;
            }
            return None;
        }

        let mut restore = None;
        if swiping {
            if self.drag_style_snapshot.is_none() {
                self.drag_style_snapshot = Some((
                    current_transition.to_string(),
                    current_transform.to_string(),
                ));
            }
        } else if let Some(snapshot) = self.drag_style_snapshot.take() {
            restore = Some(snapshot);
        }

        let delta_x = self.drag_offset.x - self.initial_transform.x;
        let delta_y = self.drag_offset.y - self.initial_transform.y;

        Some(DragStylePlan {
            transition_none: swiping,
            transform: if swiping {
                Some(get_drag_transform(
                    self.drag_offset,
                    self.initial_transform.scale,
                ))
            } else {
                None
            },
            restore,
            movement_x: format!("{}px", format_number(delta_x)),
            movement_y: format!("{}px", format_number(delta_y)),
        })
    }

    /// `recordDragSample` (`:255-271`).
    fn record_drag_sample(&mut self, offset: SwipePoint, time_stamp: Option<f64>) {
        let Some(time_stamp) = time_stamp else {
            return;
        };

        if let Some((sample, sample_time)) = self.last_drag_sample {
            if time_stamp > sample_time {
                let duration_ms = (time_stamp - sample_time).max(MIN_RELEASE_VELOCITY_DURATION_MS);
                self.last_drag_velocity = SwipePoint::new(
                    (offset.x - sample.x) / duration_ms,
                    (offset.y - sample.y) / duration_ms,
                );
            }
        }

        self.last_drag_sample = Some((offset, time_stamp));
    }

    /// `reset` (`:273-301`) — state only. `setSwiping(false)` (`:275`) and
    /// `syncDragStyles(false)` (`:300`) belong to the hook, which is the single writer of
    /// the `swiping` flag (so its dedupe cannot swallow a real transition); this returns
    /// whether a gesture was live so the hook knows the flip is coming.
    fn reset(&mut self, config: &GestureConfig) -> bool {
        self.intended_swipe_direction = None;
        let was_swiping = self.is_swiping;
        // `setDragDismissed(false)` (`:276`).
        self.swipe_threshold = config.swipe_threshold_default;
        self.drag_start_pos = SwipePoint::default();
        self.drag_offset = SwipePoint::default();
        self.initial_transform = ElementTransform {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        };
        self.max_swipe_displacement = 0.0;
        self.cancelled_swipe = false;
        self.swipe_cancel_baseline = SwipePoint::default();
        self.locked_direction = None;
        self.is_first_pointer_move = false;
        self.last_move_pos = None;
        self.pending_swipe = false;
        self.pending_swipe_start_pos = None;
        self.swipe_from_scrollable = false;
        self.saw_primary_buttons_on_move = false;
        self.element_size = SwipePoint::default();
        self.swipe_start_time = None;
        self.last_drag_sample = None;
        self.last_drag_velocity = SwipePoint::default();
        self.last_progress_details = None;
        was_swiping
    }

    /// The threshold-function ref is cleared by `reset` (`:280`) and set at gesture start
    /// (`:399-400`); the hook owns the resolver itself, so only the "is resolver active"
    /// fact lives here.
    fn clear_pending_swipe_start_state(&mut self) {
        self.pending_swipe = false;
        self.pending_swipe_start_pos = None;
    }

    /// `resetPendingSwipeState` (`:427-431`).
    fn reset_pending_swipe_state(&mut self) {
        self.clear_pending_swipe_start_state();
        self.swipe_from_scrollable = false;
        self.last_move_pos = None;
    }

    /// The state half of `startSwipeAtPosition` (`:390-424`): the resets, the drag origin,
    /// and the per-gesture threshold snapshot. The hook resolves the threshold function
    /// and element transform before calling this (it owns the DOM reads).
    fn begin_swipe(
        &mut self,
        config: &GestureConfig,
        position: SwipePoint,
        start_time: Option<f64>,
        transform: ElementTransform,
        element_size: SwipePoint,
    ) {
        self.cancelled_swipe = false;
        self.intended_swipe_direction = None;
        self.max_swipe_displacement = 0.0;

        self.drag_start_pos = position;
        self.swipe_start_time = start_time;
        self.swipe_cancel_baseline = position;
        self.last_move_pos = Some(position);
        // `swipeThresholdRef.current = swipeThresholdDefault` (`:398`).
        self.swipe_threshold = config.swipe_threshold_default;

        self.element_size = element_size;
        self.initial_transform = transform;
        self.drag_offset = SwipePoint::new(transform.x, transform.y);
        self.record_drag_sample(SwipePoint::new(transform.x, transform.y), start_time);

        // `setSwiping(true)` (`:418`) and `updateSwipeProgress(0)` (`:421`) are the hook's
        // single-writer path; the state must not flip `is_swiping` itself or the hook's
        // dedupe would swallow the signal write (the reason the browser suite pins this).
        self.locked_direction = None;
        self.is_first_pointer_move = true;
    }

    /// `applyDirectionalDamping` (`:469-482`): displacement in an unsupported direction is
    /// damped with `sign(x)·√|x|` rather than zeroed; an axis with no allowed direction at
    /// all is damped on both sides.
    fn apply_directional_damping(
        &self,
        config: &GestureConfig,
        delta_x: f64,
        delta_y: f64,
    ) -> SwipePoint {
        let exponent = |value: f64| js_sign(value) * value.abs().powf(0.5);
        let damp_axis = |delta: f64, allow_negative: bool, allow_positive: bool| {
            if (!allow_negative && delta < 0.0) || (!allow_positive && delta > 0.0) {
                exponent(delta)
            } else {
                delta
            }
        };

        let new_delta_x = if config.has_horizontal {
            damp_axis(delta_x, config.allow_left, config.allow_right)
        } else {
            exponent(delta_x)
        };
        let new_delta_y = if config.has_vertical {
            damp_axis(delta_y, config.allow_up, config.allow_down)
        } else {
            exponent(delta_y)
        };

        SwipePoint::new(new_delta_x, new_delta_y)
    }

    /// `canSwipeFromScrollEdgeOnPendingMove` (`:484-525`): `Some(true)` when the swipe may
    /// start from the reached scroll edge (ignoring the scrollable), `Some(false)` when it
    /// must not, `None` when neither axis applies.
    fn can_swipe_from_scroll_edge_on_pending_move(
        &self,
        config: &GestureConfig,
        scroll_target: &HtmlElement,
        delta_x: f64,
        delta_y: f64,
    ) -> Option<bool> {
        let can_swipe_on_axis = |delta: f64,
                                 scroll_offset: f64,
                                 max_scroll_offset: f64,
                                 allow_toward_start: bool,
                                 allow_toward_end: bool| {
            (delta > 0.0 && scroll_offset <= 0.0 && allow_toward_start)
                || (delta < 0.0 && scroll_offset >= max_scroll_offset.max(0.0) && allow_toward_end)
        };

        let abs_delta_x = delta_x.abs();
        let abs_delta_y = delta_y.abs();

        if config.has_vertical
            && delta_y != 0.0
            && (!config.has_horizontal || abs_delta_y >= abs_delta_x)
        {
            return Some(can_swipe_on_axis(
                delta_y,
                scroll_target.scroll_top() as f64,
                (scroll_target.scroll_height() - scroll_target.client_height()) as f64,
                config.allow_down,
                config.allow_up,
            ));
        }

        if config.has_horizontal
            && delta_x != 0.0
            && (!config.has_vertical || abs_delta_x > abs_delta_y)
        {
            return Some(can_swipe_on_axis(
                delta_x,
                scroll_target.scroll_left() as f64,
                (scroll_target.scroll_width() - scroll_target.client_width()) as f64,
                config.allow_right,
                config.allow_left,
            ));
        }

        None
    }

    /// `handleMoveCore`'s geometry (`:587-745`) — everything except the scrollable-target
    /// early return, `preventDefault`, and the two imperative effects, which the hook runs
    /// around this call in upstream's order.
    fn move_core(
        &mut self,
        config: &GestureConfig,
        time_stamp: f64,
        position: SwipePoint,
        movement: SwipePoint,
    ) -> MoveOutcome {
        let mut outcome = MoveOutcome::default();

        if self.is_first_pointer_move {
            self.is_first_pointer_move = false;
            // `:587-603`: re-baseline to the first move so the iOS touchstart/touchmove gap
            // doesn't jump the element — only when an element follows the pointer.
            if config.track_drag {
                self.drag_start_pos = position;
                if let Some(move_time) = get_valid_time_stamp(time_stamp) {
                    self.swipe_start_time = Some(move_time);
                }
            }
        }

        let client_x = position.x;
        let client_y = position.y;
        let movement_x = movement.x;
        let movement_y = movement.y;

        if (movement_y < 0.0 && client_y > self.swipe_cancel_baseline.y)
            || (movement_y > 0.0 && client_y < self.swipe_cancel_baseline.y)
        {
            self.swipe_cancel_baseline.y = client_y;
        }
        if (movement_x < 0.0 && client_x > self.swipe_cancel_baseline.x)
            || (movement_x > 0.0 && client_x < self.swipe_cancel_baseline.x)
        {
            self.swipe_cancel_baseline.x = client_x;
        }

        let delta_x = client_x - self.drag_start_pos.x;
        let delta_y = client_y - self.drag_start_pos.y;
        let cancel_delta_y = client_y - self.swipe_cancel_baseline.y;
        let cancel_delta_x = client_x - self.swipe_cancel_baseline.x;

        // `:629-636`: with both axes allowed the first move past 1px locks one of them.
        if self.locked_direction.is_none() && config.has_horizontal && config.has_vertical {
            let movement_distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
            if movement_distance >= MIN_DRAG_THRESHOLD {
                let locked = if delta_x.abs() > delta_y.abs() {
                    LockedAxis::Horizontal
                } else {
                    LockedAxis::Vertical
                };
                self.locked_direction = Some(locked);
            }
        }

        if self.intended_swipe_direction.is_none() {
            let candidate = match self.locked_direction {
                Some(LockedAxis::Vertical) => {
                    if delta_y > 0.0 {
                        Some(SwipeDirection::Down)
                    } else if delta_y < 0.0 {
                        Some(SwipeDirection::Up)
                    } else {
                        None
                    }
                }
                Some(LockedAxis::Horizontal) => {
                    if delta_x > 0.0 {
                        Some(SwipeDirection::Right)
                    } else if delta_x < 0.0 {
                        Some(SwipeDirection::Left)
                    } else {
                        None
                    }
                }
                None => {
                    if delta_x.abs() >= delta_y.abs() {
                        Some(if delta_x > 0.0 {
                            SwipeDirection::Right
                        } else {
                            SwipeDirection::Left
                        })
                    } else {
                        Some(if delta_y > 0.0 {
                            SwipeDirection::Down
                        } else {
                            SwipeDirection::Up
                        })
                    }
                }
            };

            if let Some(candidate) = candidate {
                let is_allowed = match candidate {
                    SwipeDirection::Left => config.allow_left,
                    SwipeDirection::Right => config.allow_right,
                    SwipeDirection::Up => config.allow_up,
                    SwipeDirection::Down => config.allow_down,
                };
                if is_allowed {
                    self.intended_swipe_direction = Some(candidate);
                    self.max_swipe_displacement = get_displacement(candidate, delta_x, delta_y);
                    outcome.direction_updates.push(Some(candidate));
                    outcome.resolve_threshold_for = Some(candidate);
                }
            }
        } else if let Some(direction) = self.intended_swipe_direction {
            let current_displacement = get_displacement(direction, cancel_delta_x, cancel_delta_y);
            if current_displacement > self.swipe_threshold {
                self.cancelled_swipe = false;
                outcome.direction_updates.push(Some(direction));
            } else if !(config.allow_left && config.allow_right)
                && !(config.allow_up && config.allow_down)
                && self.max_swipe_displacement - current_displacement >= REVERSE_CANCEL_THRESHOLD
            {
                // `:677-685`: a change of mind, latched until release.
                self.cancelled_swipe = true;
            }
        }

        let damped_delta = self.apply_directional_damping(config, delta_x, delta_y);
        let mut new_offset_x = self.initial_transform.x;
        let mut new_offset_y = self.initial_transform.y;

        match self.locked_direction {
            Some(LockedAxis::Horizontal) => {
                if config.has_horizontal {
                    new_offset_x += damped_delta.x;
                }
            }
            Some(LockedAxis::Vertical) => {
                if config.has_vertical {
                    new_offset_y += damped_delta.y;
                }
            }
            None => {
                if config.has_horizontal {
                    new_offset_x += damped_delta.x;
                }
                if config.has_vertical {
                    new_offset_y += damped_delta.y;
                }
            }
        }

        // `:708-721`: only rewrite drag styles when the offset actually changed, otherwise a
        // move jittering on the ignored axis would reinstate the raw undamped transform.
        let previous_offset = self.drag_offset;
        let offset_changed = new_offset_x != previous_offset.x || new_offset_y != previous_offset.y;
        self.drag_offset = SwipePoint::new(new_offset_x, new_offset_y);
        outcome.offset_changed = offset_changed;

        self.record_drag_sample(self.drag_offset, get_valid_time_stamp(time_stamp));

        let drag_delta_x = new_offset_x - self.initial_transform.x;
        let drag_delta_y = new_offset_y - self.initial_transform.y;
        let progress_details = SwipeProgressDetails {
            delta_x: drag_delta_x,
            delta_y: drag_delta_y,
            direction: self.intended_swipe_direction,
        };

        // `:731-743`: progress is the displacement along the progressing direction over the
        // element's scaled size.
        let mut progress = 0.0;
        let progress_direction = config.primary_direction.or(self.intended_swipe_direction);
        if let Some(direction) = progress_direction {
            let size = if direction.is_horizontal() {
                self.element_size.x
            } else {
                self.element_size.y
            };
            let scale = if self.initial_transform.scale != 0.0 {
                self.initial_transform.scale
            } else {
                1.0
            };
            let progress_displacement = get_displacement(direction, drag_delta_x, drag_delta_y);
            if size > 0.0 && scale > 0.0 && progress_displacement > 0.0 {
                progress = progress_displacement / (size * scale);
            }
        }

        outcome.progress_report = self.update_swipe_progress(progress, Some(progress_details));
        outcome
    }

    /// `cancelSwipeInteraction` (`:438-467`) — state only; returns whether a live gesture
    /// was cancelled (the hook then releases pointer capture, fires `onCancel` and resets
    /// progress).
    fn cancel_interaction(&mut self) -> bool {
        self.reset_pending_swipe_state();

        if !self.is_swiping {
            return false;
        }

        // `setSwiping(false)` (`:445`) is the hook's writer.
        self.locked_direction = None;
        self.drag_offset = SwipePoint::new(self.initial_transform.x, self.initial_transform.y);
        let _ = None::<SwipeDirection>; // `setCurrentSwipeDirection(undefined)` — hook-side.
        self.saw_primary_buttons_on_move = false;
        true
    }

    /// The measurement half of `handleEnd` (`:753-811`), run before `onRelease`.
    fn release_measurements(&self, end_time: Option<f64>) -> ReleaseMeasurements {
        let release_delta_x = self.drag_offset.x - self.initial_transform.x;
        let release_delta_y = self.drag_offset.y - self.initial_transform.y;
        let progress_details = SwipeProgressDetails {
            delta_x: release_delta_x,
            delta_y: release_delta_y,
            direction: self.intended_swipe_direction,
        };

        let delta_x = release_delta_x;
        let delta_y = release_delta_y;
        let duration_ms = match (self.swipe_start_time, end_time) {
            (Some(start), Some(end)) if end > start => end - start,
            _ => 0.0,
        };
        let velocity_duration_ms = if duration_ms > 0.0 {
            duration_ms.max(MIN_VELOCITY_DURATION_MS)
        } else {
            0.0
        };
        let velocity_x = if velocity_duration_ms > 0.0 {
            delta_x / velocity_duration_ms
        } else {
            0.0
        };
        let velocity_y = if velocity_duration_ms > 0.0 {
            delta_y / velocity_duration_ms
        } else {
            0.0
        };

        let mut release_velocity_x = self.last_drag_velocity.x;
        let mut release_velocity_y = self.last_drag_velocity.y;
        if let (Some((sample, sample_time)), Some(end)) = (self.last_drag_sample, end_time) {
            if end >= sample_time {
                let age_ms = end - sample_time;
                if age_ms <= MAX_RELEASE_VELOCITY_AGE_MS {
                    let sample_duration_ms = age_ms.max(MIN_RELEASE_VELOCITY_DURATION_MS);
                    let sample_velocity_x = (self.drag_offset.x - sample.x) / sample_duration_ms;
                    let sample_velocity_y = (self.drag_offset.y - sample.y) / sample_duration_ms;
                    if sample_velocity_x != 0.0 {
                        release_velocity_x = sample_velocity_x;
                    }
                    if sample_velocity_y != 0.0 {
                        release_velocity_y = sample_velocity_y;
                    }
                } else {
                    release_velocity_x = 0.0;
                    release_velocity_y = 0.0;
                }
            }
        }

        ReleaseMeasurements {
            progress_details,
            velocity_x,
            velocity_y,
            release_velocity_x,
            release_velocity_y,
        }
    }

    /// The decision half of `handleEnd` (`:763-859`), after `onRelease` returned
    /// `decision` (`:813-823`: `Some(true|false)` is an override, `None` means no decision).
    fn apply_release(
        &mut self,
        config: &GestureConfig,
        measurements: &ReleaseMeasurements,
        decision: Option<bool>,
    ) -> ReleaseAction {
        if !self.is_swiping {
            self.reset_pending_swipe_state();
            let progress_report =
                self.update_swipe_progress(0.0, Some(measurements.progress_details));
            return ReleaseAction::NotSwiping { progress_report };
        }

        // `setSwiping(false)` (`:769`) is the hook's writer: every `ReleaseAction` arm
        // mirrors it into the signal.
        self.locked_direction = None;
        self.reset_pending_swipe_state();
        self.saw_primary_buttons_on_move = false;

        let delta_x = measurements.progress_details.delta_x;
        let delta_y = measurements.progress_details.delta_y;

        if self.cancelled_swipe && decision.is_none() {
            self.drag_offset = SwipePoint::new(self.initial_transform.x, self.initial_transform.y);
            let progress_report =
                self.update_swipe_progress(0.0, Some(measurements.progress_details));
            return ReleaseAction::Cancelled { progress_report };
        }

        let mut should_close = false;
        let mut dismiss_direction = None;

        match decision {
            Some(override_decision) => {
                should_close = override_decision;
                dismiss_direction = self.intended_swipe_direction.or(config.primary_direction);
            }
            None => {
                // `:840-846`: the first allowed direction whose displacement beats the
                // per-gesture threshold decides.
                for direction in &config.directions {
                    if get_displacement(*direction, delta_x, delta_y) > self.swipe_threshold {
                        should_close = true;
                        dismiss_direction = Some(*direction);
                        break;
                    }
                }
            }
        }

        if should_close {
            if let Some(direction) = dismiss_direction {
                return ReleaseAction::Dismiss { direction };
            }
        }

        self.drag_offset = SwipePoint::new(self.initial_transform.x, self.initial_transform.y);
        let progress_report = self.update_swipe_progress(0.0, Some(measurements.progress_details));
        ReleaseAction::SnapBack { progress_report }
    }
}

/// The imperative style writes `syncDragStyles` performs (`:229-253`).
#[derive(Clone, Debug, PartialEq)]
struct DragStylePlan {
    /// `style.transition = 'none'` while swiping (`:236`).
    transition_none: bool,
    /// `style.transform = getDragTransform(...)` while swiping (`:247-249`).
    transform: Option<String>,
    /// `[style.transition, style.transform] = dragStyleSnapshot` (`:237-240`): the element's
    /// own inline pair to put back once the gesture ends.
    restore: Option<(String, String)>,
    /// `style.setProperty(movementCssVars.x/y, ...)` — always written (`:251-252`).
    movement_x: String,
    movement_y: String,
}

/// `getDragStyles` (`:1002-1027`) — the render-time style object. `movement_css_vars`
/// holds the configured pair so the consumer can spread it onto the element.
#[derive(Clone, Debug, PartialEq)]
pub struct SwipeDragStyles {
    pub transition: Option<&'static str>,
    pub transform: Option<String>,
    pub movement_css_vars: Vec<(String, String)>,
}

/// The `getPointerProps()` bag (`:1029-1040`).
#[derive(Clone)]
pub struct SwipePointerProps {
    pub on_pointer_down: ElementEventHandler<PointerEvent>,
    pub on_pointer_move: ElementEventHandler<PointerEvent>,
    pub on_pointer_up: ElementEventHandler<PointerEvent>,
    pub on_pointer_cancel: ElementEventHandler<PointerEvent>,
}

/// The `getTouchProps()` bag (`:1042-1053`).
#[derive(Clone)]
pub struct SwipeTouchProps {
    pub on_touch_start: ElementEventHandler<TouchEvent>,
    pub on_touch_move: ElementEventHandler<TouchEvent>,
    pub on_touch_end: ElementEventHandler<TouchEvent>,
    pub on_touch_cancel: ElementEventHandler<TouchEvent>,
}

/// `UseSwipeDismissOptions` (`:1076-1138`). `enabled` and `element_ref` are the two
/// sources upstream re-reads per handler invocation.
pub struct UseSwipeDismissOptions<D, E> {
    /// `enabled` (`:88`): a reactive boolean source, read untracked per invocation.
    pub enabled: D,
    /// `directions` (`:89`).
    pub directions: Vec<SwipeDirection>,
    /// `elementRef` (`:90`): the element the gesture is attached to.
    pub element_ref: E,
    /// `movementCssVars` (`:91`).
    pub movement_css_vars: MovementCssVars,
    /// `swipeThreshold` (`:95` — upstream default 40).
    pub swipe_threshold: Option<SwipeThreshold>,
    /// `canStart` (`:92`).
    pub can_start: Option<Rc<dyn Fn(SwipePoint, UseSwipeDismissDetails) -> bool>>,
    /// `ignoreSelectorWhenTouch` (`:93` — upstream default `true`).
    pub ignore_selector_when_touch: bool,
    /// `ignoreScrollableAncestors` (`:94` — upstream default `false`).
    pub ignore_scrollable_ancestors: bool,
    /// `trackDrag` (`:102` — upstream default `true`).
    pub track_drag: bool,
    /// `onSwipeStart` (`:99`).
    pub on_swipe_start: Option<Rc<dyn Fn(&SwipeNativeEvent)>>,
    /// `onProgress` (`:97`).
    pub on_progress: Option<Rc<dyn Fn(f64, Option<&SwipeProgressDetails>)>>,
    /// `onCancel` (`:98`).
    pub on_cancel: Option<Rc<dyn Fn(&SwipeNativeEvent)>>,
    /// `onSwipingChange` (`:101`).
    pub on_swiping_change: Option<Rc<dyn Fn(bool)>>,
    /// `onRelease` (`:100`) — `Some(true|false)` overrides the default decision.
    pub on_release: Option<Rc<dyn Fn(&SwipeNativeEvent, &SwipeReleaseDetails) -> Option<bool>>>,
    /// `onDismiss` (`:96`).
    pub on_dismiss: Option<Rc<dyn Fn(&SwipeNativeEvent, SwipeDirection)>>,
}

/// `UseSwipeDismissReturnValue` (`:1140-1159`).
pub struct UseSwipeDismissReturn {
    /// `swiping` (`:1056`).
    pub swiping: RwSignal<bool>,
    /// `swipeDirection` (`:1057`).
    pub swipe_direction: RwSignal<Option<SwipeDirection>>,
    /// `dragDismissed` (`:1058`).
    pub drag_dismissed: RwSignal<bool>,
    /// `getPointerProps()` (`:1059`).
    pub pointer_props: SwipePointerProps,
    /// `getTouchProps()` (`:1060`).
    pub touch_props: SwipeTouchProps,
    /// `moveNative` (`:1061`): feed a native `touchmove` captured by a consumer.
    pub move_native: Rc<dyn Fn(&TouchEvent, &HtmlElement)>,
    /// `getDragStyles()` (`:1062`).
    pub get_drag_styles: Rc<dyn Fn() -> SwipeDragStyles>,
    /// `reset()` (`:1063`).
    pub reset: Rc<dyn Fn()>,
}

/// The hook (`:86`).
pub fn use_swipe_dismiss<D, E>(options: UseSwipeDismissOptions<D, E>) -> UseSwipeDismissReturn
where
    D: GetUntracked<Value = bool> + 'static,
    E: Fn() -> Option<HtmlElement> + 'static,
{
    let UseSwipeDismissOptions {
        enabled,
        directions,
        element_ref,
        movement_css_vars,
        swipe_threshold,
        can_start,
        ignore_selector_when_touch,
        ignore_scrollable_ancestors,
        track_drag,
        on_swipe_start,
        on_progress,
        on_cancel,
        on_swiping_change,
        on_release,
        on_dismiss,
    } = options;

    let element_ref: Rc<dyn Fn() -> Option<HtmlElement>> = Rc::new(element_ref);
    let config = Rc::new(GestureConfig::new(
        directions,
        swipe_threshold.as_ref(),
        ignore_selector_when_touch,
        ignore_scrollable_ancestors,
        track_drag,
    ));

    let swiping = RwSignal::new(false);
    let swipe_direction = RwSignal::new(None);
    let drag_dismissed = RwSignal::new(false);

    let state = Rc::new(RefCell::new(GestureState::new(&config)));
    // `swipeThresholdFunctionRef` (`:154-156`): only a function-valued prop is stored, and
    // only at gesture start (`:399-400`).
    let threshold_function: Rc<RefCell<Option<Rc<dyn Fn(&HtmlElement, SwipeDirection) -> f64>>>> =
        Rc::new(RefCell::new(None));
    let threshold_prop = Rc::new(swipe_threshold);

    // `setSwiping` (`:164-172`): mirror into the signal and fire the callback on a flip.
    let set_swiping = {
        let state = Rc::clone(&state);
        let on_swiping_change = on_swiping_change.clone();
        Rc::new(move |next: bool| {
            let changed = state.borrow_mut().set_swiping(next);
            if changed {
                swiping.set(next);
                if let Some(callback) = on_swiping_change.as_ref() {
                    callback(next);
                }
            }
        })
    };

    let update_swipe_progress = {
        let state = Rc::clone(&state);
        let on_progress = on_progress.clone();
        Rc::new(
            move |progress: f64, details: Option<SwipeProgressDetails>| {
                if let Some((next_progress, details)) =
                    state.borrow_mut().update_swipe_progress(progress, details)
                {
                    if let Some(callback) = on_progress.as_ref() {
                        callback(next_progress, details.as_ref());
                    }
                }
            },
        )
    };

    // `resolveSwipeThreshold` (`:174-188`).
    let resolve_swipe_threshold = {
        let state = Rc::clone(&state);
        let element_ref = Rc::clone(&element_ref);
        let threshold_function = Rc::clone(&threshold_function);
        Rc::new(move |direction: Option<SwipeDirection>| {
            let Some(direction) = direction else {
                return;
            };
            let Some(element) = element_ref() else {
                return;
            };
            let resolver = threshold_function.borrow().clone();
            let Some(resolver) = resolver else {
                return;
            };
            let value = resolver(&element, direction);
            state.borrow_mut().swipe_threshold = value.max(0.0);
        })
    };

    // `syncDragStyles` (`:220-253`): the imperative writer.
    let sync_drag_styles = {
        let state = Rc::clone(&state);
        let config = Rc::clone(&config);
        let element_ref = Rc::clone(&element_ref);
        let movement_css_vars = movement_css_vars.clone();
        Rc::new(move |swiping: bool| {
            let element = element_ref();
            let plan = {
                let (transition, transform) = match element.as_ref() {
                    Some(element) => {
                        let style = inline_style(element);
                        (style_transition(&style), style_transform(&style))
                    }
                    None => (String::new(), String::new()),
                };
                state.borrow_mut().sync_drag_styles_plan(
                    &config,
                    swiping,
                    element.is_some(),
                    &transition,
                    &transform,
                )
            };

            let Some(element) = element else {
                return;
            };
            let style = inline_style(&element);

            let Some(plan) = plan else {
                // `if (!trackDrag || !element) return` (`:222-227`): no style writes at all
                // beyond the snapshot clearing done above.
                return;
            };

            if plan.transition_none {
                let _ = style.set_property("transition", "none");
            }
            if let Some(transform) = plan.transform {
                let _ = style.set_property("transform", &transform);
            }
            if let Some((transition, transform)) = plan.restore {
                let _ = style.set_property("transition", &transition);
                let _ = style.set_property("transform", &transform);
            }
            let _ = style.set_property(&movement_css_vars.x, &plan.movement_x);
            let _ = style.set_property(&movement_css_vars.y, &plan.movement_y);
        })
    };

    let start_swipe_at_position = {
        let state = Rc::clone(&state);
        let config = Rc::clone(&config);
        let element_ref = Rc::clone(&element_ref);
        let threshold_function = Rc::clone(&threshold_function);
        let threshold_prop = Rc::clone(&threshold_prop);
        let on_swipe_start = on_swipe_start.clone();
        let set_swiping = Rc::clone(&set_swiping);
        let update_swipe_progress = Rc::clone(&update_swipe_progress);
        let sync_drag_styles = Rc::clone(&sync_drag_styles);
        let resolve_swipe_threshold = Rc::clone(&resolve_swipe_threshold);
        Rc::new(
            move |event: &SwipeInput, position: SwipePoint, start_options: StartOptions| -> bool {
                state.borrow_mut().swipe_from_scrollable = false;
                let touch_like = event.is_touch_like();
                let element = element_ref();
                let native = event.native.clone();
                let target = get_target_at_point(element.as_ref(), position, &native);

                // `:366-375`: a touch starting inside a scroll container does not swipe,
                // unless the caller explicitly ignored scrollables.
                let scrollable_target = if touch_like {
                    match element.as_ref() {
                        Some(element) => {
                            let doc = owner_document(Some(element.as_ref() as &web_sys::Node));
                            doc.body().and_then(|body| {
                                find_gesture_scrollable_touch_target(
                                    target.as_ref(),
                                    &body,
                                    &config,
                                )
                            })
                        }
                        None => None,
                    }
                } else {
                    None
                };
                if scrollable_target.is_some() && !start_options.ignore_scrollable_target {
                    return false;
                }
                state.borrow_mut().swipe_from_scrollable =
                    scrollable_target.is_some() && start_options.ignore_scrollable_target;

                // `:377-380`: interactive elements are skipped — for pointer events always,
                // for touches per `ignoreSelectorWhenTouch`.
                let is_interactive_element = target
                    .as_ref()
                    .and_then(|target| target.dyn_ref::<Element>().cloned())
                    .map(|element| {
                        element
                            .closest(DEFAULT_IGNORE_SELECTOR)
                            .ok()
                            .flatten()
                            .is_some()
                    })
                    .unwrap_or(false);
                if is_interactive_element && (!touch_like || config.ignore_selector_when_touch) {
                    return false;
                }

                // `:382-388`.
                if config.ignore_scrollable_ancestors {
                    if let (Some(element), Some(target)) = (element.as_ref(), target.as_ref()) {
                        if !config.scroll_axes.is_empty() {
                            let ignore_ancestors = start_options.ignore_scrollable_ancestors;
                            let target_html = target.dyn_ref::<HtmlElement>();
                            if !ignore_ancestors {
                                if let Some(target_html) = target_html {
                                    if has_scrollable_ancestor(
                                        target_html,
                                        element,
                                        &config.scroll_axes,
                                    ) {
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                }

                // `:395-400`.
                let start_time = get_valid_time_stamp(event.time_stamp);
                let threshold_fn = match threshold_prop.as_ref() {
                    Some(SwipeThreshold::Function(resolver)) => Some(Rc::clone(resolver)),
                    _ => None,
                };

                if let Some(element) = element.as_ref() {
                    let element_size = SwipePoint::new(
                        element.offset_width() as f64,
                        element.offset_height() as f64,
                    );
                    *threshold_function.borrow_mut() = threshold_fn;
                    resolve_swipe_threshold(config.primary_direction);
                    let transform = get_element_transform(element, None);
                    state.borrow_mut().begin_swipe(
                        &config,
                        position,
                        start_time,
                        transform,
                        element_size,
                    );

                    // `:411-413`.
                    if !event.has_touches() {
                        safely_change_pointer_capture(
                            element,
                            event.pointer_id,
                            PointerCaptureMethod::Set,
                        );
                    }
                } else {
                    *threshold_function.borrow_mut() = threshold_fn;
                    state.borrow_mut().begin_swipe(
                        &config,
                        position,
                        start_time,
                        ElementTransform {
                            x: 0.0,
                            y: 0.0,
                            scale: 1.0,
                        },
                        SwipePoint::default(),
                    );
                }

                // `:416-422`.
                if let Some(callback) = on_swipe_start.as_ref() {
                    callback(&native);
                }
                set_swiping(true);
                update_swipe_progress(0.0, None);
                sync_drag_styles(true);
                true
            },
        )
    };

    let cancel_swipe_interaction = {
        let state = Rc::clone(&state);
        let element_ref = Rc::clone(&element_ref);
        let on_cancel = on_cancel.clone();
        let set_swiping = Rc::clone(&set_swiping);
        let update_swipe_progress = Rc::clone(&update_swipe_progress);
        let sync_drag_styles = Rc::clone(&sync_drag_styles);
        Rc::new(move |event: &SwipeInput| {
            let cancelled = state.borrow_mut().cancel_interaction();
            if !cancelled {
                return;
            }

            set_swiping(false);
            swipe_direction.set(None);
            sync_drag_styles(false);

            if let Some(element) = element_ref() {
                safely_change_pointer_capture(
                    &element,
                    event.pointer_id,
                    PointerCaptureMethod::Release,
                );
            }

            update_swipe_progress(
                0.0,
                Some(SwipeProgressDetails {
                    delta_x: 0.0,
                    delta_y: 0.0,
                    direction: None,
                }),
            );

            if let Some(callback) = on_cancel.as_ref() {
                callback(&event.native);
            }
        })
    };

    let handle_end = {
        let state = Rc::clone(&state);
        let config = Rc::clone(&config);
        let on_release = on_release.clone();
        let on_dismiss = on_dismiss.clone();
        let on_progress = on_progress.clone();
        let set_swiping = Rc::clone(&set_swiping);
        let sync_drag_styles = Rc::clone(&sync_drag_styles);
        Rc::new(move |event: &SwipeInput| {
            let end_time = get_valid_time_stamp(event.time_stamp);
            let measurements = state.borrow().release_measurements(end_time);

            // `:813-823`.
            let decision = on_release.as_ref().and_then(|callback| {
                callback(
                    &event.native,
                    &SwipeReleaseDetails {
                        direction: measurements.progress_details.direction,
                        delta_x: measurements.progress_details.delta_x,
                        delta_y: measurements.progress_details.delta_y,
                        velocity_x: measurements.velocity_x,
                        velocity_y: measurements.velocity_y,
                        release_velocity_x: measurements.release_velocity_x,
                        release_velocity_y: measurements.release_velocity_y,
                    },
                )
            });

            let action = state
                .borrow_mut()
                .apply_release(&config, &measurements, decision);

            match action {
                ReleaseAction::NotSwiping { progress_report } => {
                    if let Some((progress, details)) = progress_report {
                        if let Some(callback) = on_progress.as_ref() {
                            callback(progress, details.as_ref());
                        }
                    }
                }
                ReleaseAction::Cancelled { progress_report } => {
                    set_swiping(false);
                    swipe_direction.set(None);
                    sync_drag_styles(false);
                    if let Some((progress, details)) = progress_report {
                        if let Some(callback) = on_progress.as_ref() {
                            callback(progress, details.as_ref());
                        }
                    }
                }
                ReleaseAction::Dismiss { direction } => {
                    set_swiping(false);
                    swipe_direction.set(Some(direction));
                    drag_dismissed.set(true);
                    sync_drag_styles(false);
                    if let Some(callback) = on_dismiss.as_ref() {
                        callback(&event.native, direction);
                    }
                }
                ReleaseAction::SnapBack { progress_report } => {
                    set_swiping(false);
                    swipe_direction.set(None);
                    sync_drag_styles(false);
                    if let Some((progress, details)) = progress_report {
                        if let Some(callback) = on_progress.as_ref() {
                            callback(progress, details.as_ref());
                        }
                    }
                }
            }
        })
    };

    let handle_move = {
        let state = Rc::clone(&state);
        let config = Rc::clone(&config);
        let element_ref = Rc::clone(&element_ref);
        let can_start = can_start.clone();
        let start_swipe_at_position = Rc::clone(&start_swipe_at_position);
        let cancel_swipe_interaction = Rc::clone(&cancel_swipe_interaction);
        let handle_end = Rc::clone(&handle_end);
        let on_progress = on_progress.clone();
        let resolve_swipe_threshold = Rc::clone(&resolve_swipe_threshold);
        let sync_drag_styles = Rc::clone(&sync_drag_styles);
        Rc::new(move |event: &SwipeInput| {
            let Some(current_pos) = event.position else {
                return;
            };

            let mut end_after_move = false;

            if !event.has_touches() {
                let has_primary_button = has_primary_mouse_button(event.buttons);
                if has_primary_button {
                    state.borrow_mut().saw_primary_buttons_on_move = true;
                }

                // `:876-881`: a non-primary button taking over cancels outright.
                if event.buttons != 0 && !has_primary_button {
                    cancel_swipe_interaction(event);
                    return;
                }

                // `:883-897`: a trailing `buttons: 0` move *is* the release.
                if event.buttons == 0 && state.borrow().saw_primary_buttons_on_move {
                    if !state.borrow().is_swiping {
                        handle_end(event);
                        return;
                    }
                    end_after_move = true;
                }
            }

            if !state.borrow().is_swiping && state.borrow().pending_swipe {
                if !event.is_touch_like() && event.default_prevented {
                    state.borrow_mut().reset_pending_swipe_state();
                    return;
                }

                let allowed_to_start = match can_start.as_ref() {
                    Some(predicate) => predicate(
                        current_pos,
                        UseSwipeDismissDetails {
                            native_event: event.native.clone(),
                            direction: config.primary_direction,
                        },
                    ),
                    None => true,
                };

                if allowed_to_start {
                    let pending_start_pos = state.borrow().pending_swipe_start_pos;
                    let mut ignore_scrollable_on_start = false;

                    // `:919-948`.
                    if event.is_touch_like() {
                        if let (Some(pending_start_pos), Some(element)) =
                            (pending_start_pos, element_ref())
                        {
                            let native = event.native.clone();
                            let target = get_target_at_point(Some(&element), current_pos, &native);
                            let doc = owner_document(Some(element.as_ref() as &web_sys::Node));
                            let scroll_target = doc.body().and_then(|body| {
                                find_gesture_scrollable_touch_target(
                                    target.as_ref(),
                                    &body,
                                    &config,
                                )
                            });

                            if let Some(scroll_target) = scroll_target {
                                let contains_pair =
                                    contains(Some(element.as_ref()), Some(scroll_target.as_ref()))
                                        || contains(
                                            Some(scroll_target.as_ref()),
                                            Some(element.as_ref()),
                                        );
                                if contains_pair {
                                    let delta_x = current_pos.x - pending_start_pos.x;
                                    let delta_y = current_pos.y - pending_start_pos.y;
                                    let can_swipe_from_edge =
                                        state.borrow().can_swipe_from_scroll_edge_on_pending_move(
                                            &config,
                                            &scroll_target,
                                            delta_x,
                                            delta_y,
                                        );

                                    if can_swipe_from_edge == Some(false) {
                                        return;
                                    }
                                    if can_swipe_from_edge == Some(true) {
                                        ignore_scrollable_on_start = true;
                                    }
                                }
                            }
                        }
                    }

                    let started = start_swipe_at_position(
                        event,
                        current_pos,
                        StartOptions {
                            ignore_scrollable_target: ignore_scrollable_on_start,
                            ignore_scrollable_ancestors: ignore_scrollable_on_start,
                        },
                    );

                    // `:954-969`.
                    if started {
                        if pending_start_pos.is_some() && ignore_scrollable_on_start {
                            let pending_start_pos = pending_start_pos.unwrap();
                            let mut state_mut = state.borrow_mut();
                            state_mut.clear_pending_swipe_start_state();
                            state_mut.drag_start_pos = pending_start_pos;
                            state_mut.swipe_cancel_baseline = pending_start_pos;
                            state_mut.last_move_pos = Some(pending_start_pos);
                            state_mut.is_first_pointer_move = false;
                        } else {
                            let mut state_mut = state.borrow_mut();
                            state_mut.clear_pending_swipe_start_state();
                            state_mut.swipe_from_scrollable = false;
                        }
                    }
                }
            }

            // `:974-981`.
            let previous_pos = state.borrow().last_move_pos;
            let movement = match previous_pos {
                None => SwipePoint::default(),
                Some(previous) => {
                    SwipePoint::new(current_pos.x - previous.x, current_pos.y - previous.y)
                }
            };
            state.borrow_mut().last_move_pos = Some(current_pos);

            // `handleMoveCore` prelude (`:570-585`).
            if state.borrow().is_swiping {
                let native = event.native.clone();
                let target = native.target();
                if event.is_touch_like() && !state.borrow().swipe_from_scrollable {
                    let boundary = element_ref();
                    let in_scrollable = match boundary.as_ref() {
                        Some(boundary) => {
                            find_gesture_scrollable_touch_target(target.as_ref(), boundary, &config)
                                .is_some()
                        }
                        None => false,
                    };
                    if in_scrollable {
                        return;
                    }
                }

                if !event.has_touches() {
                    // Prevent text selection on Safari (`:582-585`).
                    event.native.prevent_default();
                }
            }

            let outcome =
                state
                    .borrow_mut()
                    .move_core(&config, event.time_stamp, current_pos, movement);

            for update in outcome.direction_updates {
                swipe_direction.set(update);
            }
            if let Some(direction) = outcome.resolve_threshold_for {
                resolve_swipe_threshold(Some(direction));
            }
            if outcome.offset_changed {
                sync_drag_styles(true);
            }
            if let Some((progress, details)) = outcome.progress_report {
                if let Some(callback) = on_progress.as_ref() {
                    callback(progress, details.as_ref());
                }
            }

            if end_after_move && !event.has_touches() {
                handle_end(event);
            }
        })
    };

    let handle_start = {
        let state = Rc::clone(&state);
        let config = Rc::clone(&config);
        let can_start = can_start.clone();
        let start_swipe_at_position = Rc::clone(&start_swipe_at_position);
        Rc::new(move |event: &SwipeInput| {
            if !enabled.get_untracked() {
                return;
            }
            if event.default_prevented || event.native.default_prevented() {
                return;
            }
            if !event.has_touches() && event.button != 0 {
                return;
            }
            let Some(start_pos) = event.position else {
                return;
            };

            {
                let mut state_mut = state.borrow_mut();
                state_mut.pending_swipe = true;
                state_mut.pending_swipe_start_pos = Some(start_pos);
                state_mut.swipe_from_scrollable = false;
                state_mut.saw_primary_buttons_on_move = !event.has_touches();
            }

            let allowed_to_start = match can_start.as_ref() {
                Some(predicate) => predicate(
                    start_pos,
                    UseSwipeDismissDetails {
                        native_event: event.native.clone(),
                        direction: config.primary_direction,
                    },
                ),
                None => true,
            };
            if !allowed_to_start {
                return;
            }

            if start_swipe_at_position(event, start_pos, StartOptions::default()) {
                state.borrow_mut().clear_pending_swipe_start_state();
            }
        })
    };

    // `reset` (`:273-301`) — `syncDragStyles(false)` after the state reset.
    let reset = {
        let state = Rc::clone(&state);
        let config = Rc::clone(&config);
        let set_swiping = Rc::clone(&set_swiping);
        let drag_dismissed_signal = drag_dismissed;
        let swipe_direction_signal = swipe_direction;
        let update_swipe_progress = Rc::clone(&update_swipe_progress);
        let sync_drag_styles = Rc::clone(&sync_drag_styles);
        let threshold_function = Rc::clone(&threshold_function);
        Rc::new(move || {
            let _was_swiping = state.borrow_mut().reset(&config);
            swipe_direction_signal.set(None);
            drag_dismissed_signal.set(false);
            set_swiping(false);
            *threshold_function.borrow_mut() = None;
            update_swipe_progress(0.0, None);
            sync_drag_styles(false);
        })
    };

    let get_drag_styles = {
        let state = Rc::clone(&state);
        let movement_css_vars = movement_css_vars.clone();
        Rc::new(move || {
            let state = state.borrow();
            // `:1002-1005`: read the imperative flag, not the lagging signal.
            let delta_x = state.drag_offset.x - state.initial_transform.x;
            let delta_y = state.drag_offset.y - state.initial_transform.y;
            let drag_dismissed_value = drag_dismissed.get_untracked();

            let movement = vec![
                (
                    movement_css_vars.x.clone(),
                    format!("{}px", format_number(delta_x)),
                ),
                (
                    movement_css_vars.y.clone(),
                    format!("{}px", format_number(delta_y)),
                ),
            ];

            if !state.is_swiping && delta_x == 0.0 && delta_y == 0.0 && !drag_dismissed_value {
                return SwipeDragStyles {
                    transition: None,
                    transform: None,
                    movement_css_vars: movement,
                };
            }

            SwipeDragStyles {
                transition: if state.is_swiping { Some("none") } else { None },
                transform: if state.is_swiping {
                    Some(get_drag_transform(
                        state.drag_offset,
                        state.initial_transform.scale,
                    ))
                } else {
                    None
                },
                movement_css_vars: movement,
            }
        })
    };

    let move_native = {
        let handle_move = Rc::clone(&handle_move);
        Rc::new(
            move |native_event: &TouchEvent, current_target: &HtmlElement| {
                let input = SwipeInput::from_native_touch_move(native_event, current_target);
                handle_move(&input);
            },
        )
    };

    let pointer_props = {
        let handle_start = Rc::clone(&handle_start);
        let handle_move = Rc::clone(&handle_move);
        let handle_end = Rc::clone(&handle_end);
        SwipePointerProps {
            on_pointer_down: Rc::new(move |event: &PointerEvent| {
                let input = SwipeInput::from_pointer(event);
                handle_start(&input);
            }),
            on_pointer_move: Rc::new(move |event: &PointerEvent| {
                let input = SwipeInput::from_pointer(event);
                handle_move(&input);
            }),
            on_pointer_up: {
                let handle_end = Rc::clone(&handle_end);
                Rc::new(move |event: &PointerEvent| {
                    let input = SwipeInput::from_pointer(event);
                    handle_end(&input);
                })
            },
            on_pointer_cancel: {
                let handle_end = Rc::clone(&handle_end);
                Rc::new(move |event: &PointerEvent| {
                    let input = SwipeInput::from_pointer(event);
                    handle_end(&input);
                })
            },
        }
    };

    let touch_props = {
        let handle_start = Rc::clone(&handle_start);
        let handle_move = Rc::clone(&handle_move);
        let handle_end = Rc::clone(&handle_end);
        SwipeTouchProps {
            on_touch_start: Rc::new(move |event: &TouchEvent| {
                let input = SwipeInput::from_touch(event);
                handle_start(&input);
            }),
            on_touch_move: Rc::new(move |event: &TouchEvent| {
                let input = SwipeInput::from_touch(event);
                handle_move(&input);
            }),
            on_touch_end: {
                let handle_end = Rc::clone(&handle_end);
                Rc::new(move |event: &TouchEvent| {
                    let input = SwipeInput::from_touch(event);
                    handle_end(&input);
                })
            },
            on_touch_cancel: {
                let handle_end = Rc::clone(&handle_end);
                Rc::new(move |event: &TouchEvent| {
                    let input = SwipeInput::from_touch(event);
                    handle_end(&input);
                })
            },
        }
    };

    UseSwipeDismissReturn {
        swiping,
        swipe_direction,
        drag_dismissed,
        pointer_props,
        touch_props,
        move_native,
        get_drag_styles,
        reset,
    }
}

/// `startSwipeAtPosition`'s `startOptions` (`:357-360`).
#[derive(Clone, Copy, Debug, Default)]
struct StartOptions {
    ignore_scrollable_target: bool,
    ignore_scrollable_ancestors: bool,
}

/// `safelyChangePointerCapture` (`:66-84`): pointer capture is optional (absent in some
/// environments) and a `NotFoundError` is swallowed; every other error propagates
/// upstream, which the port records by relying on the web-sys `Result` without unwrapping.
#[derive(Clone, Copy)]
enum PointerCaptureMethod {
    Set,
    Release,
}

fn safely_change_pointer_capture(
    element: &HtmlElement,
    pointer_id: i32,
    method: PointerCaptureMethod,
) {
    let result = match method {
        PointerCaptureMethod::Set => element.set_pointer_capture(pointer_id),
        PointerCaptureMethod::Release => element.release_pointer_capture(pointer_id),
    };
    // `:79-82`: only `NotFoundError` is swallowed. web-sys surfaces the DOM exception as a
    // `JsValue`; the name check mirrors upstream's duck-typed `error.name` test.
    if let Err(error) = result {
        let name = js_sys::Reflect::get(&error, &"name".into())
            .ok()
            .and_then(|name| name.as_string())
            .unwrap_or_default();
        let _swallowed = name == "NotFoundError";
    }
}

/// `element.style` — the inline style declaration the imperative writer touches
/// (`:229`).
fn inline_style(element: &HtmlElement) -> CssStyleDeclaration {
    element.style()
}

fn style_transition(style: &CssStyleDeclaration) -> String {
    style.get_property_value("transition").unwrap_or_default()
}

fn style_transform(style: &CssStyleDeclaration) -> String {
    style.get_property_value("transform").unwrap_or_default()
}

/// `getTargetAtPoint` (`:323-328`): the element under the point inside the element's root
/// node, falling back to the event target.
fn get_target_at_point(
    element: Option<&HtmlElement>,
    position: SwipePoint,
    native: &SwipeNativeEvent,
) -> Option<EventTarget> {
    let root = element.map(|element| element.get_root_node());
    let element_at_point =
        get_element_at_point(root.as_ref(), position.x, position.y).map(EventTarget::from);
    element_at_point.or_else(|| native.target())
}

/// `findGestureScrollableTouchTarget` (`:330-352`): the scrollable touch target for the
/// configured axes, with the page scroller (`body` / `documentElement`) excluded.
fn find_gesture_scrollable_touch_target(
    target: Option<&EventTarget>,
    root: &HtmlElement,
    config: &GestureConfig,
) -> Option<HtmlElement> {
    let find = |axis: ScrollAxis| -> Option<HtmlElement> {
        let scroll_target = find_scrollable_touch_target(target, root, axis, false)?;
        let doc = owner_document(Some(scroll_target.as_ref() as &web_sys::Node));
        let is_page_scroller = doc
            .body()
            .map(|body| is_same_node(body.as_ref(), scroll_target.as_ref()))
            .unwrap_or(false)
            || doc
                .document_element()
                .map(|element| is_same_node(element.as_ref(), scroll_target.as_ref()))
                .unwrap_or(false);
        if is_page_scroller {
            None
        } else {
            Some(scroll_target)
        }
    };

    if config.has_horizontal && !config.has_vertical {
        return find(ScrollAxis::Horizontal);
    }
    if config.has_vertical && !config.has_horizontal {
        return find(ScrollAxis::Vertical);
    }
    find(ScrollAxis::Vertical).or_else(|| find(ScrollAxis::Horizontal))
}

fn is_same_node(left: &Element, right: &HtmlElement) -> bool {
    left == right.as_ref()
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    /// `['down']`-only, the Drawer viewport's close engine shape (`:108-118`).
    fn down_config() -> GestureConfig {
        GestureConfig::new(vec![SwipeDirection::Down], None, true, false, true)
    }

    /// `['up','down','left','right']` — the SwipeArea's open engine shape.
    fn all_config() -> GestureConfig {
        GestureConfig::new(
            vec![
                SwipeDirection::Up,
                SwipeDirection::Down,
                SwipeDirection::Left,
                SwipeDirection::Right,
            ],
            None,
            true,
            false,
            true,
        )
    }

    /// A 200×200 element at rest (the behavior.md default-threshold scenario). The
    /// trailing `set_swiping(true)` mirrors the hook, which owns the `swiping` flag and
    /// its signal (`use_swipe_dismiss`'s `start_swipe_at_position` counterpart, `:418`).
    fn begin(state: &mut GestureState, config: &GestureConfig, time_stamp: f64) {
        state.begin_swipe(
            config,
            SwipePoint::new(0.0, 0.0),
            get_valid_time_stamp(time_stamp),
            ElementTransform {
                x: 0.0,
                y: 0.0,
                scale: 1.0,
            },
            SwipePoint::new(200.0, 200.0),
        );
        state.set_swiping(true);
    }

    /// Drive one `handleMoveCore` step from a raw position, deriving the movement delta the
    /// way `handleMove` does (`:974-981`).
    fn move_to(
        state: &mut GestureState,
        config: &GestureConfig,
        x: f64,
        y: f64,
        time_stamp: f64,
    ) -> MoveOutcome {
        let position = SwipePoint::new(x, y);
        let movement = match state.last_move_pos {
            None => SwipePoint::default(),
            Some(previous) => SwipePoint::new(x - previous.x, y - previous.y),
        };
        state.last_move_pos = Some(position);
        state.move_core(config, time_stamp, position, movement)
    }

    /// A real drag emits many moves; the first one only re-baselines the drag origin
    /// (`:587-603`), so a measured drag settles at the origin first.
    fn drag_to(
        state: &mut GestureState,
        config: &GestureConfig,
        x: f64,
        y: f64,
        time_stamp: f64,
    ) -> MoveOutcome {
        if state.is_first_pointer_move {
            let origin = state.drag_start_pos;
            move_to(state, config, origin.x, origin.y, time_stamp - 1.0);
        }
        move_to(state, config, x, y, time_stamp)
    }

    // `getDisplacement` (`:39-52`).
    #[test]
    fn displacement_maps_each_direction_to_its_signed_axis() {
        assert_eq!(get_displacement(SwipeDirection::Up, 10.0, 20.0), -20.0);
        assert_eq!(get_displacement(SwipeDirection::Down, 10.0, 20.0), 20.0);
        assert_eq!(get_displacement(SwipeDirection::Left, 10.0, 20.0), -10.0);
        assert_eq!(get_displacement(SwipeDirection::Right, 10.0, 20.0), 10.0);
    }

    // `exponent` inside `applyDirectionalDamping` (`:470`) is `Math.sign(v)·√|v|`.
    #[test]
    fn js_sign_matches_math_sign_including_zero_and_nan() {
        assert_eq!(js_sign(3.0), 1.0);
        assert_eq!(js_sign(-3.0), -1.0);
        assert_eq!(js_sign(0.0), 0.0);
        assert!(js_sign(f64::NAN).is_nan());
    }

    // `getValidTimeStamp` (`:54-56`).
    #[test]
    fn only_positive_finite_timestamps_are_valid() {
        assert_eq!(get_valid_time_stamp(120.0), Some(120.0));
        assert_eq!(get_valid_time_stamp(0.0), None);
        assert_eq!(get_valid_time_stamp(-5.0), None);
        assert_eq!(get_valid_time_stamp(f64::NAN), None);
        assert_eq!(get_valid_time_stamp(f64::INFINITY), None);
    }

    // `hasPrimaryMouseButton` (`:62-64`).
    #[test]
    fn primary_mouse_button_is_the_low_bit() {
        assert!(has_primary_mouse_button(1));
        assert!(!has_primary_mouse_button(0));
        assert!(!has_primary_mouse_button(2));
        assert!(has_primary_mouse_button(3));
    }

    // `getDragTransform` (`:58-60`).
    #[test]
    fn drag_transform_is_a_translate3d_with_the_initial_scale() {
        assert_eq!(
            get_drag_transform(SwipePoint::new(0.0, 40.0), 1.0),
            "translate3d(0px,40px,0) scale(1)"
        );
        assert_eq!(
            get_drag_transform(SwipePoint::new(-12.5, 0.0), 0.96),
            "translate3d(-12.5px,0px,0) scale(0.96)"
        );
    }

    // `swipeThresholdDefault` (`:108-111`), `primaryDirection` (`:106`) and `scrollAxes`
    // (`:120-129`).
    #[test]
    fn config_derives_the_default_threshold_primary_direction_and_scroll_axes() {
        let single = GestureConfig::new(vec![SwipeDirection::Down], None, true, false, true);
        assert_eq!(single.swipe_threshold_default, 40.0);
        assert_eq!(single.primary_direction, Some(SwipeDirection::Down));
        assert!(single.has_vertical);
        assert!(!single.has_horizontal);
        // Vertical axes come first (`:120-129`).
        assert_eq!(single.scroll_axes, vec![ScrollAxis::Vertical]);

        let both = all_config();
        assert_eq!(both.primary_direction, None);
        assert_eq!(
            both.scroll_axes,
            vec![ScrollAxis::Vertical, ScrollAxis::Horizontal]
        );

        // A negative numeric threshold clamps to 0 (`:108-111`).
        let clamped = GestureConfig::new(
            vec![SwipeDirection::Down],
            Some(&SwipeThreshold::Pixels(-5.0)),
            true,
            false,
            true,
        );
        assert_eq!(clamped.swipe_threshold_default, 0.0);
    }

    // `applyDirectionalDamping` (`:469-482`): unsupported directions are damped with
    // `sign(v)·√|v|`, an axis with no allowed direction is damped both ways.
    #[test]
    fn unsupported_directions_are_damped_exponentially_not_zeroed() {
        let down_only = down_config();
        let mut state = GestureState::new(&down_only);
        // Allowed: the raw delta passes through.
        let damped = state.apply_directional_damping(&down_only, 0.0, 100.0);
        assert_eq!(damped, SwipePoint::new(0.0, 100.0));
        // Backwards past the gesture direction: damped.
        let damped = state.apply_directional_damping(&down_only, 0.0, -9.0);
        assert_eq!(damped, SwipePoint::new(0.0, -3.0));
        // No horizontal direction is allowed at all: damped on both sides.
        let damped = state.apply_directional_damping(&down_only, 16.0, 0.0);
        assert_eq!(damped, SwipePoint::new(4.0, 0.0));

        let all = all_config();
        let damped = state.apply_directional_damping(&all, 16.0, -0.0);
        assert_eq!(damped, SwipePoint::new(16.0, -0.0));
    }

    // `handleMoveCore`'s latch (`:638-670`): the intended direction is chosen once and the
    // max displacement is recorded from that moment.
    #[test]
    fn the_first_allowed_direction_latches_and_records_max_displacement() {
        let config = down_config();
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);

        let outcome = drag_to(&mut state, &config, 0.0, 60.0, 116.0);
        assert_eq!(outcome.direction_updates, vec![Some(SwipeDirection::Down)]);
        assert_eq!(state.intended_swipe_direction, Some(SwipeDirection::Down));
        assert_eq!(state.max_swipe_displacement, 60.0);
        assert!(outcome.offset_changed);
    }

    // `:677-685`: backtracking 10px from the max displacement latches a change of mind
    // (only when neither axis has both directions allowed).
    #[test]
    fn backtracking_past_ten_pixels_latches_a_change_of_mind() {
        let config = down_config();
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);

        drag_to(&mut state, &config, 0.0, 60.0, 116.0);
        assert!(!state.cancelled_swipe);

        // 60 → 40 is a 20px reversal, past the 10px reverse-cancel threshold.
        drag_to(&mut state, &config, 0.0, 40.0, 132.0);
        assert!(state.cancelled_swipe);

        // The same reversal on an axis with both directions allowed does not cancel.
        let both = GestureConfig::new(
            vec![SwipeDirection::Up, SwipeDirection::Down],
            None,
            true,
            false,
            true,
        );
        let mut state = GestureState::new(&both);
        begin(&mut state, &both, 100.0);
        move_to(&mut state, &both, 0.0, 60.0, 116.0);
        move_to(&mut state, &both, 0.0, 40.0, 132.0);
        assert!(!state.cancelled_swipe);
    }

    // `:731-743`: progress is the progressing direction's displacement over the element's
    // scaled size, clamped to 0..1 by `updateSwipeProgress` (`:192`).
    #[test]
    fn progress_is_displacement_over_the_scaled_element_size() {
        let config = down_config();
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);

        // 100px of a 200px-tall element.
        let outcome = drag_to(&mut state, &config, 0.0, 100.0, 116.0);
        let (progress, details) = outcome.progress_report.expect("progress is reported");
        assert_eq!(progress, 0.5);
        assert_eq!(details.unwrap().direction, Some(SwipeDirection::Down));

        // Past the element's full travel: clamped to 1.
        let outcome = drag_to(&mut state, &config, 0.0, 300.0, 132.0);
        assert_eq!(outcome.progress_report.unwrap().0, 1.0);

        // A scaled element measures against `size · scale` (`:738-742`).
        let mut scaled = GestureState::new(&config);
        scaled.begin_swipe(
            &config,
            SwipePoint::new(0.0, 0.0),
            get_valid_time_stamp(100.0),
            ElementTransform {
                x: 0.0,
                y: 0.0,
                scale: 2.0,
            },
            SwipePoint::new(200.0, 200.0),
        );
        let outcome = drag_to(&mut scaled, &config, 0.0, 100.0, 116.0);
        assert_eq!(outcome.progress_report.unwrap().0, 0.25);
    }

    // `updateSwipeProgress`'s dedupe (`:193-208`) and its non-finite clamp (`:192`).
    #[test]
    fn progress_reports_only_when_something_changed() {
        let mut state = GestureState::default();
        assert!(state.update_swipe_progress(0.5, None).is_some());
        assert!(state.update_swipe_progress(0.5, None).is_none());
        assert!(state.update_swipe_progress(f64::NAN, None).is_some());
        assert_eq!(state.swipe_progress, 0.0);

        let details = SwipeProgressDetails {
            delta_x: 1.0,
            delta_y: 2.0,
            direction: None,
        };
        assert!(state.update_swipe_progress(0.0, Some(details)).is_some());
        // The same progress and details again: no report.
        assert!(state.update_swipe_progress(0.0, Some(details)).is_none());
        // Changed details alone still reports.
        let changed = SwipeProgressDetails {
            delta_x: 1.0,
            delta_y: 3.0,
            direction: None,
        };
        assert!(state.update_swipe_progress(0.0, Some(changed)).is_some());
    }

    // behavior.md, State model → `useSwipeDismiss`: the default threshold is 40px — a 35px
    // displacement does not dismiss, a 100px one does; a custom `swipeThreshold: 10`
    // dismisses at 20px (`useSwipeDismiss.test.tsx:1155-1229`, `:1079-1153`, `:743-820`).
    #[test]
    fn the_release_threshold_is_the_default_until_a_resolver_replaces_it() {
        // 35px: below the 40px default.
        let config = down_config();
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);
        drag_to(&mut state, &config, 0.0, 35.0, 116.0);
        let measurements = state.release_measurements(get_valid_time_stamp(132.0));
        assert_eq!(
            state.apply_release(&config, &measurements, None),
            ReleaseAction::SnapBack {
                progress_report: Some((
                    0.0,
                    Some(SwipeProgressDetails {
                        delta_x: 0.0,
                        delta_y: 35.0,
                        direction: Some(SwipeDirection::Down),
                    })
                ))
            }
        );

        // 100px: past the default.
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);
        drag_to(&mut state, &config, 0.0, 100.0, 116.0);
        let measurements = state.release_measurements(get_valid_time_stamp(132.0));
        assert_eq!(
            state.apply_release(&config, &measurements, None),
            ReleaseAction::Dismiss {
                direction: SwipeDirection::Down
            }
        );

        // A resolved function threshold of 10 dismisses the 20px drag.
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);
        drag_to(&mut state, &config, 0.0, 20.0, 116.0);
        state.swipe_threshold = 10.0;
        let measurements = state.release_measurements(get_valid_time_stamp(132.0));
        assert_eq!(
            state.apply_release(&config, &measurements, None),
            ReleaseAction::Dismiss {
                direction: SwipeDirection::Down
            }
        );
    }

    // behavior.md, State model → `useSwipeDismiss`: `swipeThreshold` is snapshotted at
    // gesture start, so a mid-gesture prop change does not affect the active gesture
    // (`useSwipeDismiss.test.tsx:822-918`).
    #[test]
    fn the_threshold_is_snapshotted_at_gesture_start() {
        let config = GestureConfig::new(
            vec![SwipeDirection::Down],
            Some(&SwipeThreshold::Pixels(50.0)),
            true,
            false,
            true,
        );
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);
        assert_eq!(state.swipe_threshold, 50.0);

        // The prop changes to 10 mid-gesture; only a fresh `begin_swipe` re-reads the
        // default (the resolver path is the hook's `resolveSwipeThreshold`).
        drag_to(&mut state, &config, 0.0, 20.0, 116.0);
        let measurements = state.release_measurements(get_valid_time_stamp(132.0));
        assert!(matches!(
            state.apply_release(&config, &measurements, None),
            ReleaseAction::SnapBack { .. }
        ));

        // The next gesture starts from the config's snapshot value again.
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 200.0);
        assert_eq!(state.swipe_threshold, 50.0);
    }

    // `:813-823` + `:836-847`: `onRelease` returning a boolean overrides the threshold
    // decision in both directions.
    #[test]
    fn a_release_override_beats_the_threshold_decision() {
        let config = down_config();

        // Override `true` on a drag that would otherwise snap back.
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);
        drag_to(&mut state, &config, 0.0, 10.0, 116.0);
        let measurements = state.release_measurements(get_valid_time_stamp(132.0));
        assert_eq!(
            state.apply_release(&config, &measurements, Some(true)),
            ReleaseAction::Dismiss {
                direction: SwipeDirection::Down
            }
        );

        // Override `false` on a drag that would otherwise dismiss.
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);
        drag_to(&mut state, &config, 0.0, 100.0, 116.0);
        let measurements = state.release_measurements(get_valid_time_stamp(132.0));
        assert!(matches!(
            state.apply_release(&config, &measurements, Some(false)),
            ReleaseAction::SnapBack { .. }
        ));
        // The snap-back returns the offset to the initial transform (`:855`).
        assert_eq!(state.drag_offset, SwipePoint::new(0.0, 0.0));
    }

    // `:825-831`: a latched change of mind cancels unless `onRelease` overrode it.
    #[test]
    fn a_latched_change_of_mind_snaps_back_unless_overridden() {
        let config = down_config();
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);
        drag_to(&mut state, &config, 0.0, 60.0, 116.0);
        drag_to(&mut state, &config, 0.0, 40.0, 132.0);
        assert!(state.cancelled_swipe);

        let measurements = state.release_measurements(get_valid_time_stamp(148.0));
        assert!(matches!(
            state.apply_release(&config, &measurements, None),
            ReleaseAction::Cancelled { .. }
        ));

        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);
        drag_to(&mut state, &config, 0.0, 60.0, 116.0);
        drag_to(&mut state, &config, 0.0, 40.0, 132.0);
        let measurements = state.release_measurements(get_valid_time_stamp(148.0));
        assert_eq!(
            state.apply_release(&config, &measurements, Some(true)),
            ReleaseAction::Dismiss {
                direction: SwipeDirection::Down
            }
        );
    }

    // `:783-811`: gesture velocity from the start/release timestamps (50ms floor) and the
    // release velocity from the last movement sample (16ms floor, 80ms staleness cutoff).
    #[test]
    fn release_velocities_use_the_gesture_duration_and_the_last_sample() {
        let config = down_config();
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 1000.0);

        // The absorbed first move re-baselines the gesture clock to 1000ms (`:598-601`),
        // then a slow 50px hop and a fast 50px hop.
        move_to(&mut state, &config, 0.0, 0.0, 1000.0);
        move_to(&mut state, &config, 0.0, 50.0, 1050.0);
        move_to(&mut state, &config, 0.0, 100.0, 1060.0);

        let measurements = state.release_measurements(get_valid_time_stamp(1065.0));
        assert_eq!(measurements.progress_details.delta_y, 100.0);
        // Gesture velocity: 100px over the 65ms gesture, floored at 50ms.
        assert!((measurements.velocity_y - 100.0 / 65.0).abs() < 1e-9);
        // Release velocity: the last 50px hop over the 16ms sample floor, not the gesture
        // average.
        assert!((measurements.release_velocity_y - 50.0 / 16.0).abs() < 1e-9);

        // A stale sample (>80ms old) zeroes the release velocity (`:807-810`).
        let measurements = state.release_measurements(get_valid_time_stamp(1200.0));
        assert_eq!(measurements.release_velocity_y, 0.0);

        // Without a usable start timestamp the gesture velocity is 0 (`:784-789`).
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 0.0);
        move_to(&mut state, &config, 0.0, 0.0, 0.0);
        move_to(&mut state, &config, 0.0, 100.0, 1100.0);
        let measurements = state.release_measurements(get_valid_time_stamp(1100.0));
        assert_eq!(measurements.velocity_y, 0.0);
    }

    // `syncDragStyles` (`:220-253`): the snapshot is taken on the first swiping write,
    // `transition: none` + the frozen transform are written while swiping, the movement
    // vars carry the offset delta (damped or not), and the snapshot is handed back on
    // release.
    #[test]
    fn drag_styles_snapshot_freeze_and_restore() {
        let config = down_config();
        let mut state = GestureState::new(&config);

        // `trackDrag: false`: no writes at all (`:222-227`).
        let no_track = GestureConfig::new(vec![SwipeDirection::Down], None, true, false, false);
        assert!(
            state
                .sync_drag_styles_plan(&no_track, true, true, "", "")
                .is_none()
        );

        begin(&mut state, &config, 100.0);
        drag_to(&mut state, &config, 0.0, 40.0, 116.0);

        let plan = state
            .sync_drag_styles_plan(&config, true, true, "transform 0.2s", "translate(1px)")
            .expect("swiping writes");
        assert!(plan.transition_none);
        assert_eq!(
            plan.transform.as_deref(),
            Some("translate3d(0px,40px,0) scale(1)")
        );
        assert_eq!(plan.movement_x, "0px");
        assert_eq!(plan.movement_y, "40px");
        assert_eq!(plan.restore, None);

        // The release restores the element's own inline pair and zeroes the movement vars.
        let plan = state
            .sync_drag_styles_plan(&config, false, true, "", "")
            .expect("release writes");
        assert!(!plan.transition_none);
        assert_eq!(plan.transform, None);
        assert_eq!(
            plan.restore,
            Some(("transform 0.2s".to_string(), "translate(1px)".to_string()))
        );
        assert_eq!(plan.movement_y, "40px");

        // The snapshot was consumed: a second release write has nothing to restore.
        let plan = state
            .sync_drag_styles_plan(&config, false, true, "", "")
            .expect("release writes");
        assert_eq!(plan.restore, None);
    }

    // `reset` (`:273-301`): every ref returns to its seed value and `swiping` flips off
    // exactly once.
    #[test]
    fn reset_returns_every_gesture_ref_to_its_seed() {
        let config = down_config();
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);
        drag_to(&mut state, &config, 0.0, 60.0, 116.0);
        assert!(state.is_swiping);
        assert!(state.swipe_progress > 0.0);

        let was_swiping = state.reset(&config);
        assert!(was_swiping);
        // The hook writes the flip (and its signal) right after the reset (`:275`).
        assert!(state.set_swiping(false));
        assert!(!state.is_swiping);
        assert!(!state.set_swiping(false));
        // `updateSwipeProgress(0)` (`:277`) zeroes the reported progress after the reset.
        assert!(state.update_swipe_progress(0.0, None).is_some());
        assert_eq!(state.drag_offset, SwipePoint::new(0.0, 0.0));
        assert_eq!(state.drag_start_pos, SwipePoint::new(0.0, 0.0));
        assert_eq!(state.intended_swipe_direction, None);
        assert_eq!(state.max_swipe_displacement, 0.0);
        assert!(!state.cancelled_swipe);
        assert_eq!(state.locked_direction, None);
        assert_eq!(state.last_move_pos, None);
        assert!(!state.pending_swipe);
        assert_eq!(state.swipe_start_time, None);
        assert_eq!(state.last_drag_sample, None);
        assert_eq!(state.swipe_progress, 0.0);
        assert_eq!(state.swipe_threshold, config.swipe_threshold_default);
        assert_eq!(state.drag_style_snapshot, None);

        // Already-off: a second reset reports no live gesture.
        assert!(!state.reset(&config));
    }

    // `handleStart` (`:527-563`) + the pending activation on the first move (`:900-971`).
    #[test]
    fn the_gesture_activates_on_the_first_move_not_the_press() {
        let config = down_config();
        let mut state = GestureState::new(&config);

        // The press: pending only, no live gesture.
        state.pending_swipe = true;
        state.pending_swipe_start_pos = Some(SwipePoint::new(0.0, 0.0));
        state.saw_primary_buttons_on_move = true;
        assert!(!state.is_swiping);

        // The activation happens in `start_swipe_at_position`, i.e. `begin_swipe`.
        begin(&mut state, &config, 100.0);
        assert!(state.is_swiping);

        // `handleStart` clears the pending refs once the start succeeded (`:560-562`).
        state.clear_pending_swipe_start_state();
        assert!(!state.pending_swipe);
        assert_eq!(state.pending_swipe_start_pos, None);

        // The first move is absorbed as the new drag origin and leaves the offset at rest.
        let outcome = move_to(&mut state, &config, 0.0, 40.0, 116.0);
        assert!(outcome.direction_updates.is_empty());
        assert_eq!(state.drag_offset, SwipePoint::new(0.0, 0.0));
        assert!(!state.is_first_pointer_move);

        // The next move is measured from there.
        let outcome = drag_to(&mut state, &config, 0.0, 60.0, 132.0);
        assert_eq!(outcome.direction_updates, vec![Some(SwipeDirection::Down)]);
    }

    // `cancelSwipeInteraction` (`:438-467`): the offset returns to the initial transform
    // and the callback only fires for a live gesture.
    #[test]
    fn cancel_returns_the_offset_and_only_fires_for_a_live_gesture() {
        let config = down_config();
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);
        drag_to(&mut state, &config, 0.0, 60.0, 116.0);

        assert!(state.cancel_interaction());
        // The hook's writer records the flip (`:445`).
        assert!(state.set_swiping(false));
        assert!(!state.is_swiping);
        assert_eq!(state.drag_offset, SwipePoint::new(0.0, 0.0));
        assert_eq!(state.locked_direction, None);
        assert!(!state.saw_primary_buttons_on_move);

        // No live gesture: no second cancel.
        assert!(!state.cancel_interaction());
    }

    // `handleEnd`'s early return (`:763-767`): a release with no live gesture still reports
    // the (zero) progress and never dismisses.
    #[test]
    fn a_release_without_a_live_gesture_reports_progress_only() {
        let config = down_config();
        let mut state = GestureState::new(&config);
        let measurements = state.release_measurements(get_valid_time_stamp(100.0));
        assert!(matches!(
            state.apply_release(&config, &measurements, None),
            ReleaseAction::NotSwiping { .. }
        ));
    }

    // `:876-897`: a `buttons: 0` trailing move is the release, and every direction's
    // displacement is compared against the snapshotted threshold in the release loop
    // (`:840-846`) — the first matching direction in `directions` order wins.
    #[test]
    fn the_release_picks_the_first_direction_whose_displacement_crosses_the_threshold() {
        let config = all_config();
        let mut state = GestureState::new(&config);
        begin(&mut state, &config, 100.0);
        // A 60px leftward drag also carries a positive displacement along `right`'s axis
        // only if it went right; here only `left` crosses.
        drag_to(&mut state, &config, -60.0, 0.0, 116.0);
        let measurements = state.release_measurements(get_valid_time_stamp(132.0));
        assert_eq!(
            state.apply_release(&config, &measurements, None),
            ReleaseAction::Dismiss {
                direction: SwipeDirection::Left
            }
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use reactive_graph::owner::Owner;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Document, HtmlElement, PointerEvent, PointerEventInit, Window};

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// A real, attached 200×200 element with the hook's `['down']`-only pointer props
    /// wired to it, plus the callback log the behavior spec's assertions read
    /// (`specs/library/utils/behavior.md`, State model → `useSwipeDismiss`).
    struct Harness {
        /// Keeps the reactive owner (the hook's signals) alive for the whole test.
        #[allow(dead_code)]
        owner: Owner,
        element: HtmlElement,
        return_value: UseSwipeDismissReturn,
        enabled: RwSignal<bool>,
        dismissed: Rc<RefCell<Vec<SwipeDirection>>>,
        swiping_changes: Rc<RefCell<Vec<bool>>>,
        cancels: Rc<Cell<u32>>,
    }

    impl Harness {
        fn new(seeded_enabled: bool, threshold: Option<SwipeThreshold>) -> Self {
            let owner = Owner::new();
            owner.set();

            let document: Document = window().document().unwrap();
            let element = document
                .create_element("div")
                .unwrap()
                .dyn_into::<HtmlElement>()
                .unwrap();
            let style = element.style();
            let _ = style.set_property("width", "200px");
            let _ = style.set_property("height", "200px");
            document
                .body()
                .unwrap()
                .append_child(element.as_ref() as &web_sys::Node)
                .unwrap();

            let dismissed: Rc<RefCell<Vec<SwipeDirection>>> = Rc::new(RefCell::new(Vec::new()));
            let swiping_changes: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
            let cancels: Rc<Cell<u32>> = Rc::new(Cell::new(0));
            let enabled = RwSignal::new(seeded_enabled);

            let element_for_source = element.clone();
            let dismissed_for_callback = Rc::clone(&dismissed);
            let swiping_changes_for_callback = Rc::clone(&swiping_changes);
            let cancels_for_callback = Rc::clone(&cancels);
            let return_value = use_swipe_dismiss(UseSwipeDismissOptions {
                enabled,
                directions: vec![SwipeDirection::Down],
                element_ref: move || Some(element_for_source.clone()),
                movement_css_vars: MovementCssVars {
                    x: "--x".to_string(),
                    y: "--y".to_string(),
                },
                swipe_threshold: threshold,
                can_start: None,
                ignore_selector_when_touch: true,
                ignore_scrollable_ancestors: false,
                track_drag: true,
                on_swipe_start: None,
                on_progress: None,
                on_cancel: Some(Rc::new(move |_| {
                    cancels_for_callback.set(cancels_for_callback.get() + 1)
                })),
                on_swiping_change: Some(Rc::new(move |swiping: bool| {
                    swiping_changes_for_callback.borrow_mut().push(swiping);
                })),
                on_release: None,
                on_dismiss: Some(Rc::new(move |_, direction: SwipeDirection| {
                    dismissed_for_callback.borrow_mut().push(direction);
                })),
            });

            Self {
                owner,
                element,
                return_value,
                enabled,
                dismissed,
                swiping_changes,
                cancels,
            }
        }

        fn down(&self, x: f64, y: f64) {
            (self.return_value.pointer_props.on_pointer_down)(&pointer_event(
                "pointerdown",
                x,
                y,
                1,
                0,
            ));
        }

        fn move_to(&self, x: f64, y: f64, buttons: u16) {
            (self.return_value.pointer_props.on_pointer_move)(&pointer_event(
                "pointermove",
                x,
                y,
                buttons,
                0,
            ));
        }

        fn up(&self, x: f64, y: f64) {
            (self.return_value.pointer_props.on_pointer_up)(&pointer_event(
                "pointerup",
                x,
                y,
                0,
                0,
            ));
        }

        /// pointerdown + the absorbed first move + the measured move (`:587-603`).
        fn drag(&self, distance: f64) {
            self.down(0.0, 0.0);
            self.move_to(0.0, 0.0, 1);
            self.move_to(0.0, distance, 1);
        }

        fn style(&self, property: &str) -> String {
            self.element
                .style()
                .get_property_value(property)
                .unwrap_or_default()
        }
    }

    impl Drop for Harness {
        fn drop(&mut self) {
            let _ = self.element.remove();
        }
    }

    fn window() -> Window {
        web_sys::window().unwrap()
    }

    fn pointer_event(type_: &str, x: f64, y: f64, buttons: u16, button: i16) -> PointerEvent {
        let init = PointerEventInit::new();
        init.set_client_x(x as i32);
        init.set_client_y(y as i32);
        init.set_buttons(buttons);
        init.set_button(button);
        init.set_pointer_id(1);
        init.set_pointer_type("mouse");
        init.set_bubbles(true);
        PointerEvent::new_with_event_init_dict(type_, &init).expect("PointerEvent failed")
    }

    // behavior.md, State model → `useSwipeDismiss`: a drag past the 40px default dismisses,
    // and the host element's inline `transform`/`transition`/movement vars are written
    // imperatively (`useSwipeDismiss.test.tsx:1155-1229`, `:605-653`).
    #[wasm_bindgen_test]
    fn a_drag_past_the_threshold_dismisses_and_writes_the_drag_styles() {
        let harness = Harness::new(true, None);

        harness.down(0.0, 0.0);
        let after_down = (harness.return_value.get_drag_styles)();
        assert!(
            harness.return_value.swiping.get_untracked(),
            "swiping signal is stale after pointerdown; internal flag: is_swiping(transition={:?})",
            after_down.transition
        );
        harness.move_to(0.0, 0.0, 1);
        harness.move_to(0.0, 60.0, 1);

        assert_eq!(harness.style("--x"), "0px");
        assert_eq!(harness.style("--y"), "60px");
        assert_eq!(harness.style("transition"), "none");
        // Chrome re-serializes the inline transform it stores (`0px, 60px, 0px`); the raw
        // `getDragTransform` string is pinned by the host suite's
        // `drag_transform_is_a_translate3d_with_the_initial_scale`.
        assert_eq!(
            harness.style("transform"),
            "translate3d(0px, 60px, 0px) scale(1)"
        );

        harness.up(0.0, 60.0);

        assert_eq!(*harness.dismissed.borrow(), vec![SwipeDirection::Down]);
        assert!(!harness.return_value.swiping.get_untracked());
        assert!(harness.return_value.drag_dismissed.get_untracked());
        assert_eq!(
            harness.return_value.swipe_direction.get_untracked(),
            Some(SwipeDirection::Down)
        );
        // `onSwipingChange` observed the flip on and off (`:164-172`).
        assert_eq!(*harness.swiping_changes.borrow(), vec![true, false]);
        // The element's own inline pair is restored on release (`:237-240`).
        assert_eq!(harness.style("transition"), "");
        assert_eq!(harness.style("transform"), "");
        assert_eq!(harness.style("--y"), "60px");
    }

    // behavior.md, State model → `useSwipeDismiss`: a 35px displacement does not dismiss
    // under the 40px default, and `--y` returns to `'0px'` on the snap-back
    // (`useSwipeDismiss.test.tsx:1079-1153`, `:1059`).
    #[wasm_bindgen_test]
    fn a_drag_below_the_threshold_snaps_back() {
        let harness = Harness::new(true, None);

        harness.drag(35.0);
        harness.up(0.0, 35.0);

        assert!(harness.dismissed.borrow().is_empty());
        assert!(!harness.return_value.drag_dismissed.get_untracked());
        assert!(!harness.return_value.swiping.get_untracked());
        assert_eq!(harness.style("--y"), "0px");
        assert_eq!(harness.style("transform"), "");
    }

    // behavior.md, State model → `useSwipeDismiss`: a custom `swipeThreshold: 10` dismisses
    // at 20px (`useSwipeDismiss.test.tsx:743-820`).
    #[wasm_bindgen_test]
    fn a_pixel_threshold_prop_dismisses_at_its_own_distance() {
        let harness = Harness::new(true, Some(SwipeThreshold::Pixels(10.0)));

        harness.drag(20.0);
        harness.up(0.0, 20.0);

        assert_eq!(*harness.dismissed.borrow(), vec![SwipeDirection::Down]);
    }

    // `swipeThreshold` as a resolver is called with the element and the latched direction
    // (`:174-188`), and the latched direction is what the gesture progresses by.
    #[wasm_bindgen_test]
    fn a_function_threshold_resolves_per_direction_at_latch_time() {
        let calls: Rc<RefCell<Vec<SwipeDirection>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_for_resolver = Rc::clone(&calls);
        let harness = Harness::new(
            true,
            Some(SwipeThreshold::Function(Rc::new(move |_, direction| {
                calls_for_resolver.borrow_mut().push(direction);
                10.0
            }))),
        );

        harness.drag(20.0);
        assert_eq!(
            *calls.borrow(),
            vec![SwipeDirection::Down, SwipeDirection::Down],
            "the resolver runs at gesture start ({}) and again at latch time ({})",
            404,
            668
        );

        harness.up(0.0, 20.0);
        assert_eq!(*harness.dismissed.borrow(), vec![SwipeDirection::Down]);
    }

    // `enabled: false` makes every handler inert (`:528-530`, `:570`, `:749`).
    #[wasm_bindgen_test]
    fn a_disabled_gesture_is_inert() {
        let harness = Harness::new(false, None);

        harness.drag(100.0);
        harness.up(0.0, 100.0);

        assert!(!harness.return_value.swiping.get_untracked());
        assert!(harness.dismissed.borrow().is_empty());
        assert!(harness.swiping_changes.borrow().is_empty());
    }

    // `:876-881`: a non-primary button taking over mid-drag cancels the gesture, reports
    // `onCancel`, and returns the offset to rest.
    #[wasm_bindgen_test]
    fn a_non_primary_button_takeover_cancels_the_gesture() {
        let harness = Harness::new(true, None);

        harness.drag(60.0);
        let after_drag = (harness.return_value.get_drag_styles)();
        assert!(
            harness.return_value.swiping.get_untracked(),
            "swiping signal is stale after the drag; internal flag: is_swiping(transition={:?})",
            after_drag.transition
        );

        harness.move_to(0.0, 70.0, 2);

        assert_eq!(harness.cancels.get(), 1);
        assert!(!harness.return_value.swiping.get_untracked());
        assert_eq!(harness.return_value.swipe_direction.get_untracked(), None);
        assert_eq!(harness.style("--y"), "0px");
        assert!(harness.dismissed.borrow().is_empty());
    }

    // `getDragStyles()` (`:1002-1027`): the resting shape carries the movement vars at zero,
    // and the swiping shape carries the frozen transform and `transition: none`.
    #[wasm_bindgen_test]
    fn drag_styles_report_the_resting_and_swiping_shapes() {
        let harness = Harness::new(true, None);

        let resting = (harness.return_value.get_drag_styles)();
        assert_eq!(resting.transition, None);
        assert_eq!(resting.transform, None);
        assert_eq!(
            resting.movement_css_vars,
            vec![
                ("--x".to_string(), "0px".to_string()),
                ("--y".to_string(), "0px".to_string())
            ]
        );

        harness.drag(60.0);
        let swiping = (harness.return_value.get_drag_styles)();
        assert_eq!(swiping.transition, Some("none"));
        assert_eq!(
            swiping.transform.as_deref(),
            Some("translate3d(0px,60px,0) scale(1)")
        );
        assert_eq!(swiping.movement_css_vars[1].1, "60px");
    }

    // `reset()` (`:273-301`): a live gesture is abandoned, the styles are restored, and the
    // machine forgets the gesture.
    #[wasm_bindgen_test]
    fn reset_abandons_a_live_gesture_and_restores_the_styles() {
        let harness = Harness::new(true, None);

        harness.drag(60.0);
        (harness.return_value.reset)();

        assert!(!harness.return_value.swiping.get_untracked());
        assert_eq!(harness.style("--y"), "0px");
        assert_eq!(harness.style("transform"), "");
        assert_eq!(harness.style("transition"), "");

        // The next release is inert: the gesture is gone.
        harness.up(0.0, 60.0);
        assert!(harness.dismissed.borrow().is_empty());
    }

    // The element-source bail: a resolvable-but-detached source leaves the gesture at rest
    // (no transform is readable, so no offset accumulates and nothing dismisses).
    #[wasm_bindgen_test]
    fn a_missing_element_never_dismisses() {
        let owner = Owner::new();
        owner.set();

        let dismissed: Rc<RefCell<Vec<SwipeDirection>>> = Rc::new(RefCell::new(Vec::new()));
        let dismissed_for_callback = Rc::clone(&dismissed);
        let return_value = use_swipe_dismiss(UseSwipeDismissOptions {
            enabled: RwSignal::new(true),
            directions: vec![SwipeDirection::Down],
            element_ref: || None,
            movement_css_vars: MovementCssVars {
                x: "--x".to_string(),
                y: "--y".to_string(),
            },
            swipe_threshold: None,
            can_start: None,
            ignore_selector_when_touch: true,
            ignore_scrollable_ancestors: false,
            track_drag: true,
            on_swipe_start: None,
            on_progress: None,
            on_cancel: None,
            on_swiping_change: None,
            on_release: None,
            on_dismiss: Some(Rc::new(move |_, direction: SwipeDirection| {
                dismissed_for_callback.borrow_mut().push(direction);
            })),
        });

        (return_value.pointer_props.on_pointer_down)(&pointer_event("pointerdown", 0.0, 0.0, 1, 0));
        (return_value.pointer_props.on_pointer_move)(&pointer_event(
            "pointermove",
            0.0,
            100.0,
            1,
            0,
        ));
        (return_value.pointer_props.on_pointer_up)(&pointer_event("pointerup", 0.0, 100.0, 0, 0));

        assert!(dismissed.borrow().is_empty());
        assert!(!return_value.swiping.get_untracked());
    }
}
