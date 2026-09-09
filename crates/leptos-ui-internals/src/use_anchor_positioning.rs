//! Port of `packages/react/src/internals/useAnchorPositioning.ts` — the standardized
//! anchor-positioning hook every floating Base UI component positions its popup through
//! (`specs/library/internals/behavior.md`, "DOM structure & portal behavior": "the
//! positioner element receives `positionerStyles` ... the anchor is registered via
//! `refs.setPositionReference`"; `specs/library/internals/implementation.md`,
//! "`useAnchorPositioning`": "A middleware *builder* around the vendored floating-ui
//! `useFloating`").
//!
//! ## Rust adaptations
//!
//! - **Function-based middleware options** (`offset`'s data function, `limitShift`'s
//!   offset function, `size`'s `apply`, the `arrow` element function) require
//!   `&'static` closures: `floating_ui_core::Derivable::Fn` holds a *borrowed*
//!   `&dyn Fn` while the engine's middleware vector (`MiddlewareVec`) boxes every
//!   middleware to `'static`. The port `Box::leak`s one small closure per live-read
//!   option, per hook instance. The leaks are bounded (a few boxed closures capturing
//!   `Rc` cells and signal handles — no store graphs) and only ever run while the
//!   instance's engine is live (the engine bails before middleware run once either
//!   element is gone). Upstream rebuilds the whole stack per render and lets the JS GC
//!   reclaim it; the leak is the no-per-render-reallocation counterpart.
//! - **`open` gating**: upstream forwards `open: keepMounted ? mounted : undefined`
//!   into `useFloating`, and the engine's `openRef` — not the store's open — decides
//!   `isPositioned` (`hooks/useFloating.ts:71-77` spreads `...options` into the engine,
//!   so the option reaches it on the root-store path too). The port's engine
//!   (`use_position`) gates on the *store's* open, which flips false while the popup is
//!   still mounted during an exit transition. To reproduce upstream's observable
//!   behavior (the positioner keeps its computed styles through the ending phase,
//!   upstream `:499-501` reading a still-true `isPositioned`), the
//!   [`PositionerStyles`] derivation treats the popup as positioned while
//!   `is_positioned || (!store_open && mounted)`. The raw engine signal is returned
//!   unchanged as [`UseAnchorPositioningReturn::is_positioned`]. The `lazyFlip` lock
//!   effect keeps the raw gate (upstream's `isPositioned`), so a lock can only be
//!   taken while the engine reports positioned — during an ending phase it cannot;
//!   the sticky value is cleared on close anyway.
//! - **`shift.rootBoundary`**: upstream forwards `'layoutViewport'` (or the default
//!   visual viewport) into floating-ui's root-boundary option. `floating-ui-dom` 0.6.0
//!   has no layout-viewport distinction (`RootBoundary` is
//!   `Viewport | Document | Rect`, and its viewport rect always reads the visual
//!   viewport), so both configurations map to `RootBoundary::Viewport` — the
//!   pinch-zoom/iOS-keyboard difference the option exists for is not expressible in the
//!   external crate. The [`ShiftRootBoundary`] param is kept so the caller-facing API
//!   stays faithful.
//! - **`collisionBoundary`**: upstream's `Rect` arm has no counterpart in
//!   `floating-ui-dom` 0.6.0's `Boundary` enum (`ClippingAncestors | Element |
//!   Elements`), so [`CollisionBoundary`] omits it. No upstream component passes a
//!   `Rect` boundary (the positioners default to `'clipping-ancestors'`); if one ever
//!   does, the external crate needs the variant first.
//! - **Anchor shapes**: upstream accepts an element, a virtual element, a
//!   `RefObject`, or a function returning one, and unwraps refs (`isRef`,
//!   `useAnchorPositioning.ts:637-641`). The port's [`Anchor`] collapses the
//!   ref-object form into [`Anchor::Fn`] — the caller resolves its `NodeRef` inside
//!   the closure — because this crate is reactive-graph-only and has no `NodeRef`
//!   type. Upstream's two-effect anchor registration (`:537-571` — a layout effect
//!   plus a passive effect re-checking refs populated after layout effects) collapses
//!   into one tracked layout effect plus a passive re-check that also covers `Fn`
//!   anchors: Leptos `NodeRef`s bind when the view builds, before effects run, so the
//!   React parent-ref timing gap does not exist; the passive pass is kept
//!   (dedupe-guarded by the same registration reference) so a late-populating anchor
//!   still resolves.
//! - **Per-render rebuilds dissolve**: upstream recreates the middleware array every
//!   render (with dep arrays for `offset`/`shift`/`arrow`), so param changes (e.g.
//!   `sideOffset` number values) swap the stack. The port builds the stack once: live
//!   values (offset numbers and functions, arrow presence, mount state, RTL) are read
//!   through captured handles on every engine pass, while static params (`side`) are
//!   fixed per instance — the established "hooks run once" convention, discharged to
//!   the view layer the same way `useTransitionStatus`'s render-phase trio was.
//! - **`useStableCallback(anchorFn)`** (`:177-178`) is unnecessary here: the
//!   `Anchor::Fn` handle is an `Rc` whose identity is stable for the component's
//!   lifetime by construction.
//! - The render-phase `mountSide` reset (`:166-168`) becomes a plain effect watching
//!   `mounted` — the sticky-side lock must only be cleared while closed, and the next
//!   open observes the cleared value either way.
//! - `nodeId`/`externalTree` (`:158,161`) are accepted for API fidelity but not yet
//!   wired: the port's `use_base_ui_floating` seam hardcodes them (nodes wiring is the
//!   floating-ui item's store concern).

use std::cell::RefCell;
use std::rc::Rc;

use floating_ui_core::middleware::{
    ApplyFn, ApplyState, CrossAxis, LimitShift, LimitShiftOffset, LimitShiftOffsetValues,
    LimitShiftOptions, Limiter, ShiftData,
};
use floating_ui_core::{
    Boundary as CoreBoundary, Derivable, DerivableFn, DetectOverflowOptions, Middleware,
    MiddlewareReturn, MiddlewareState, RootBoundary,
};
use floating_ui_dom::{
    Alignment, ArrowData, AutoUpdateOptions, Axis, ElementOrVirtual, Flip, FlipOptions, Offset,
    OffsetOptions, OffsetOptionsValues, Padding, PartialSideObject, Placement, Shift, ShiftOptions,
    Size, SizeOptions, Strategy, auto_update, dom,
};
use reactive_graph::computed::Memo;
use reactive_graph::owner::LocalStorage;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use web_sys::{Element, Node, Window};

use leptos_ui_utils::owner_document;
use leptos_ui_utils::use_iso_layout_effect;
use leptos_ui_utils::use_value_as_ref::{ValueAsRef, use_value_as_ref};

use crate::adaptive_origin_constants::AdaptiveOriginData;
use crate::common_positioner_css_vars;
use crate::constants::CollisionAvoidancePreset;
use crate::direction_context::{TextDirection, use_direction};
use crate::floating_ui::arrow::{ArrowOptions, BaseArrow, OffsetParent};
use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::types::{ExtendedRefs, FloatingContext, ReferenceType, WrappedMiddleware};
use crate::floating_ui::use_floating::use_base_ui_floating;
use crate::floating_ui::use_position::UsePositionOptions;
use crate::hide_middleware::{HIDE_NAME, hide};

/// Re-export of the positioning engine's physical side — upstream's
/// `type Side as PhysicalSide` import (`useAnchorPositioning.ts:22`).
pub use floating_ui_dom::Side as PhysicalSide;

/// Upstream `Side` (`useAnchorPositioning.ts:63`) — the logical side: the physical
/// sides plus the two writing-mode-relative ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
    InlineEnd,
    InlineStart,
}

/// Upstream `Align` (`useAnchorPositioning.ts:64`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
}

impl From<Option<Alignment>> for Align {
    /// `getAlignment(placement) || 'center'` (`useAnchorPositioning.ts:56,582`).
    fn from(alignment: Option<Alignment>) -> Self {
        match alignment {
            Some(Alignment::Start) => Align::Start,
            Some(Alignment::End) => Align::End,
            None => Align::Center,
        }
    }
}

/// Upstream `Boundary` (`useAnchorPositioning.ts:65`) minus the `Rect` arm — see the
/// module docs for why the external crate cannot express it.
#[derive(Clone, Debug, PartialEq)]
pub enum CollisionBoundary {
    /// `'clipping-ancestors'` (the default).
    ClippingAncestors,
    /// A single boundary element.
    Element(Element),
    /// Multiple boundary elements.
    Elements(Vec<Element>),
}

impl CollisionBoundary {
    /// The `commonCollisionProps` boundary mapping (`useAnchorPositioning.ts:235`):
    /// `'clipping-ancestors'` becomes the engine's `'clippingAncestors'`, everything
    /// else passes through.
    fn to_engine_boundary(&self) -> CoreBoundary<Element> {
        match self {
            CollisionBoundary::ClippingAncestors => CoreBoundary::ClippingAncestors,
            CollisionBoundary::Element(element) => CoreBoundary::Element(element.clone()),
            CollisionBoundary::Elements(elements) => CoreBoundary::Elements(elements.clone()),
        }
    }
}

/// Upstream `OffsetFunction`'s data object (`useAnchorPositioning.ts:66-71`) — what
/// the offset callbacks receive.
#[derive(Clone, Debug)]
pub struct OffsetData {
    /// The logical side the positioner is aligned against.
    pub side: Side,
    /// How the positioner is aligned relative to the side.
    pub align: Align,
    /// The anchor element's dimensions.
    pub anchor: floating_ui_utils::Dimensions,
    /// The positioner element's dimensions.
    pub positioner: floating_ui_utils::Dimensions,
}

pub type OffsetFunction = Rc<dyn Fn(&OffsetData) -> f64>;

/// Upstream `sideOffset`/`alignOffset` (`useAnchorPositioning.ts:690,720`) — a number
/// or a function reading the live geometry.
#[derive(Clone)]
pub enum SideOffset {
    Number(f64),
    Function(OffsetFunction),
}

impl Default for SideOffset {
    /// `0` (`useAnchorPositioning.ts:143,145`).
    fn default() -> Self {
        SideOffset::Number(0.0)
    }
}

impl SideOffset {
    fn resolve(&self, data: &OffsetData) -> f64 {
        match self {
            SideOffset::Number(value) => *value,
            SideOffset::Function(function) => function(data),
        }
    }
}

impl PartialEq for SideOffset {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SideOffset::Number(a), SideOffset::Number(b)) => a == b,
            // Function identity — upstream compares nothing (it re-reads the
            // render-time param); the pointer check only serves the dedupe paths.
            (SideOffset::Function(a), SideOffset::Function(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// Upstream `SideFlipMode`/`SideShiftMode`'s `side` field
/// (`useAnchorPositioning.ts:79,103`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CollisionAvoidanceSide {
    /// `'flip'` (the default) — try the opposite side when there is not enough space.
    #[default]
    Flip,
    /// `'shift'` — keep the preferred side and shift within the boundary.
    Shift,
    /// `'none'` — keep the preferred side even if it overflows.
    None,
}

/// Upstream `SideFlipMode`/`SideShiftMode`'s `align` field
/// (`useAnchorPositioning.ts:86,109`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CollisionAvoidanceAlign {
    /// `'flip'` (the default) — swap start/end alignment when it overflows.
    #[default]
    Flip,
    /// `'shift'` — keep the alignment and shift within the boundary.
    Shift,
    /// `'none'` — keep the preferred alignment even if it overflows.
    None,
}

/// Upstream's `fallbackAxisSide` field (`useAnchorPositioning.ts:94,117`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FallbackAxisSide {
    /// `'start'` — prefer the logical start side of the perpendicular axis.
    Start,
    /// `'end'` (the default) — prefer the logical end side of the perpendicular axis.
    #[default]
    End,
    /// `'none'` — never fall back to the perpendicular axis.
    None,
}

/// Upstream `CollisionAvoidance` (`useAnchorPositioning.ts:120`): the
/// `SideFlipMode | SideShiftMode` union, flattened into one struct with the same
/// per-field defaults the hook applies (`collisionAvoidance.side || 'flip'` etc.,
/// `:170-172`). The TS union's "when `side` is `'shift'`, `align` only supports
/// `'shift'`/`'none'`" restriction is compile-time-only; the runtime code reads the
/// fields uniformly, and so does the port.
///
/// The two constants-file presets (`DROPDOWN_COLLISION_AVOIDANCE` /
/// `POPUP_COLLISION_AVOIDANCE`) convert with [`CollisionAvoidance::from_preset`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CollisionAvoidance {
    /// `side` — defaults to [`CollisionAvoidanceSide::Flip`].
    pub side: Option<CollisionAvoidanceSide>,
    /// `align` — defaults to [`CollisionAvoidanceAlign::Flip`].
    pub align: Option<CollisionAvoidanceAlign>,
    /// `fallbackAxisSide` — defaults to [`FallbackAxisSide::End`].
    pub fallback_axis_side: Option<FallbackAxisSide>,
}

impl CollisionAvoidance {
    fn side(&self) -> CollisionAvoidanceSide {
        self.side.unwrap_or_default()
    }

    fn align(&self) -> CollisionAvoidanceAlign {
        self.align.unwrap_or_default()
    }

    fn fallback_axis_side(&self) -> FallbackAxisSide {
        self.fallback_axis_side.unwrap_or_default()
    }

    /// The constants-file presets (`{ fallbackAxisSide: 'none' | 'end' }`) — the
    /// `&'static str` literal mapped onto the enum. Every other field is absent, so
    /// the hook's defaults apply, exactly as upstream's partial literal does.
    pub fn from_preset(preset: &CollisionAvoidancePreset) -> Self {
        CollisionAvoidance {
            side: None,
            align: None,
            fallback_axis_side: Some(match preset.fallback_axis_side {
                "none" => FallbackAxisSide::None,
                "start" => FallbackAxisSide::Start,
                _ => FallbackAxisSide::End,
            }),
        }
    }
}

/// Upstream's `shift` parameter (`useAnchorPositioning.ts:799-804`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShiftConfig {
    /// `crossAxis` — defaults to `false` (`:173`).
    pub cross_axis: bool,
    /// `rootBoundary` — see the module docs for the external-crate limitation.
    pub root_boundary: ShiftRootBoundary,
}

/// Upstream `shift.rootBoundary` (`useAnchorPositioning.ts:802`): `undefined` (the
/// visual viewport) or `'layoutViewport'`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShiftRootBoundary {
    /// `undefined` — the visual viewport (floating-ui's default root boundary).
    #[default]
    VisualViewport,
    /// `'layoutViewport'` — mapped to the same engine boundary (module docs).
    LayoutViewport,
}

/// Upstream `collisionPadding` (`useAnchorPositioning.ts:730` — floating-ui's
/// `Padding`: a number or a partial side object). Defaults to `5` (`:147`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CollisionPadding {
    All(f64),
    Sides {
        top: f64,
        right: f64,
        bottom: f64,
        left: f64,
    },
}

impl Default for CollisionPadding {
    fn default() -> Self {
        CollisionPadding::All(5.0)
    }
}

impl From<f64> for CollisionPadding {
    fn from(value: f64) -> Self {
        CollisionPadding::All(value)
    }
}

/// The normalized four-side padding — upstream's destructured
/// `{ top, right, bottom, left }` object (`useAnchorPositioning.ts:200-221`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaddingRect {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl PaddingRect {
    /// The engine's `Padding` option shape (a full side object).
    fn to_padding(self) -> Padding {
        Padding::PerSide(PartialSideObject {
            top: Some(self.top),
            right: Some(self.right),
            bottom: Some(self.bottom),
            left: Some(self.left),
        })
    }
}

/// Normalizes the parameter (`useAnchorPositioning.ts:200-221`): a number spreads to
/// every side as-is; the object path applies the `|| 0` falsy normalization per side.
fn normalize_collision_padding(padding: &CollisionPadding) -> PaddingRect {
    match padding {
        CollisionPadding::All(value) => PaddingRect {
            top: *value,
            right: *value,
            bottom: *value,
            left: *value,
        },
        CollisionPadding::Sides {
            top,
            right,
            bottom,
            left,
        } => {
            let or_zero = |value: f64| {
                if value.is_finite() && value != 0.0 {
                    value
                } else {
                    0.0
                }
            };
            PaddingRect {
                top: or_zero(*top),
                right: or_zero(*right),
                bottom: or_zero(*bottom),
                left: or_zero(*left),
            }
        }
    }
}

/// Upstream `anchor` (`useAnchorPositioning.ts:648-654`): an element or virtual
/// element, statically or through a function (which also covers the ref-object form —
/// see the module docs). Defaults to [`Anchor::None`]; the popup then positions
/// against whatever the shared root context already holds.
#[derive(Clone, Default)]
pub enum Anchor {
    /// `anchor` omitted or `null`.
    #[default]
    None,
    /// A static element or virtual element.
    Static(ReferenceType),
    /// A function anchor — re-resolved on every registration pass; the ref-object
    /// form is expressed by reading the `NodeRef` inside the closure.
    Fn(AnchorFn),
}

/// The function-anchor handle — an `Rc` for cheap cloning into the effects.
pub type AnchorFn = Rc<dyn Fn() -> Option<ReferenceType>>;

/// Resolves the anchor to a positioning reference (`useAnchorPositioning.ts:542-546`
/// — the function call plus the ref unwrap; the ref unwrap lives in the caller's
/// closure in the port).
fn resolve_anchor(anchor: &Anchor) -> Option<ReferenceType> {
    match anchor {
        Anchor::None => None,
        Anchor::Static(reference) => Some(reference.clone()),
        Anchor::Fn(function) => function(),
    }
}

/// The `!==` identity check for the registration dedupe
/// (`useAnchorPositioning.ts:548,567`): elements compare by DOM identity (the
/// `web_sys::Element` `PartialEq`), virtual references by their identity token.
fn reference_matches(a: &Option<ReferenceType>, b: &Option<ReferenceType>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(ReferenceType::Element(x)), Some(ReferenceType::Element(y))) => x == y,
        (Some(ReferenceType::Virtual(x)), Some(ReferenceType::Virtual(y))) => x.id == y.id,
        _ => false,
    }
}

/// The upstream hook's parameter object (`UseAnchorPositioningParameters`,
/// `useAnchorPositioning.ts:791-812`, with the required members bundled into
/// [`UseAnchorPositioningParams::new`]).
pub struct UseAnchorPositioningParams {
    /// `anchor` — the element to position against. Defaults to [`Anchor::None`].
    pub anchor: Anchor,
    /// `positionMethod` — defaults to [`Strategy::Absolute`] (`:141`).
    pub position_method: Strategy,
    /// `side` — the preferred logical side. Defaults to [`Side::Bottom`] (`:142`).
    pub side: Side,
    /// `sideOffset` — defaults to `0` (`:143`).
    pub side_offset: SideOffset,
    /// `align` — defaults to [`Align::Center`] (`:144`).
    pub align: Align,
    /// `alignOffset` — defaults to `0` (`:145`).
    pub align_offset: SideOffset,
    /// `collisionBoundary` — defaults to [`CollisionBoundary::ClippingAncestors`]
    /// (`:146`).
    pub collision_boundary: CollisionBoundary,
    /// `collisionPadding` — defaults to `5` (`:147`).
    pub collision_padding: CollisionPadding,
    /// `sticky` — defaults to `false` (`:148`).
    pub sticky: bool,
    /// `arrowPadding` — defaults to `5` (`:149`).
    pub arrow_padding: f64,
    /// `disableAnchorTracking` — defaults to `false` (`:150`).
    pub disable_anchor_tracking: bool,
    /// `inline` — caller middleware pushed before everything else (`:151`, used by
    /// Preview Card's line-box override).
    pub inline: Option<Box<dyn Middleware<Element, Window>>>,
    // -- Private parameters -------------------------------------------------------------
    /// `keepMounted` (`:153`).
    pub keep_mounted: bool,
    /// `floatingRootContext` — required (upstream's public entry requires it too,
    /// `:129`); the popup tree's shared root store.
    pub floating_root_context: Rc<FloatingRootStore>,
    /// `mounted` (`:155`).
    pub mounted: Signal<bool, LocalStorage>,
    /// `collisionAvoidance` (`:156`).
    pub collision_avoidance: CollisionAvoidance,
    /// `shift` (`:157`).
    pub shift: Option<ShiftConfig>,
    /// `nodeId` (`:158`) — accepted for API fidelity; see the module docs.
    pub node_id: Option<String>,
    /// `adaptiveOrigin` (`:159`) — the caller middleware whose data drives the
    /// adaptive-branch `positionerStyles` split.
    pub adaptive_origin: Option<Box<dyn Middleware<Element, Window>>>,
    /// `lazyFlip` (`:160`).
    pub lazy_flip: bool,
    /// `externalTree` (`:161`) — accepted for API fidelity; see the module docs.
    pub external_tree: Option<crate::floating_ui::tree::SharedFloatingTreeStore>,
}

impl UseAnchorPositioningParams {
    /// Defaults with the two required members (`floatingRootContext`, `mounted`);
    /// every other field matches upstream's default-parameter values.
    pub fn new(
        floating_root_context: Rc<FloatingRootStore>,
        mounted: Signal<bool, LocalStorage>,
    ) -> Self {
        UseAnchorPositioningParams {
            anchor: Anchor::None,
            position_method: Strategy::Absolute,
            side: Side::Bottom,
            side_offset: SideOffset::default(),
            align: Align::Center,
            align_offset: SideOffset::default(),
            collision_boundary: CollisionBoundary::ClippingAncestors,
            collision_padding: CollisionPadding::default(),
            sticky: false,
            arrow_padding: 5.0,
            disable_anchor_tracking: false,
            inline: None,
            keep_mounted: false,
            floating_root_context,
            mounted,
            collision_avoidance: CollisionAvoidance::default(),
            shift: None,
            node_id: None,
            adaptive_origin: None,
            lazy_flip: false,
            external_tree: None,
        }
    }
}

/// Upstream `arrowStyles` (`useAnchorPositioning.ts:594-601`).
#[derive(Clone, Debug, PartialEq)]
pub struct ArrowStyles {
    /// Always `'absolute'`.
    pub position: &'static str,
    /// `middlewareData.arrow?.y` — `None` when the arrow middleware has not run.
    pub top: Option<String>,
    /// `middlewareData.arrow?.x`.
    pub left: Option<String>,
}

/// Upstream `positionerStyles` (`useAnchorPositioning.ts:503-533`) — the styles the
/// positioner element binds. The `--available-*` seeds are always present (see the
/// upstream comment: the keys must stay present with a constant value so React's
/// per-property style diff — here, the consumer's style bindings — never rewrites the
/// values `size()` sets imperatively).
#[derive(Clone, Debug, PartialEq)]
pub struct PositionerStyles {
    /// `position` — `fixed` until positioned (prevents `autoFocus` scroll jumps),
    /// then the configured `positionMethod`.
    pub position: Strategy,
    /// `top` — set on every branch except the adaptive split's opposite side.
    pub top: Option<String>,
    /// `left`.
    pub left: Option<String>,
    /// `right` — only the adaptive split can target it (`sideX`).
    pub right: Option<String>,
    /// `bottom` — only the adaptive split can target it (`sideY`).
    pub bottom: Option<String>,
    /// `transform` — the engine's transform positioning.
    pub transform: Option<String>,
    /// `willChange` — the engine's DPR hint.
    pub will_change: Option<String>,
    /// `opacity` — `0` only while not positioned.
    pub opacity: Option<String>,
    /// `--available-width` — seeded `'100vw'`; `size()` writes the real value
    /// imperatively.
    pub available_width: String,
    /// `--available-height` — seeded `'100vh'`.
    pub available_height: String,
}

/// Upstream `UseAnchorPositioningReturnValue` (`useAnchorPositioning.ts:814-827`).
pub struct UseAnchorPositioningReturn {
    /// `positionerStyles`.
    pub positioner_styles: Signal<PositionerStyles, LocalStorage>,
    /// `arrowStyles`.
    pub arrow_styles: Signal<ArrowStyles, LocalStorage>,
    /// `arrowRef` — the fillable arrow-element slot; the consumer sets it from its
    /// arrow part's `NodeRef`. Read by the `arrow`/`limitShift`/`transformOrigin`
    /// middleware on every engine pass.
    pub arrow_ref: Rc<RefCell<Option<Element>>>,
    /// `arrowUncentered` — `middlewareData.arrow?.centerOffset !== 0`.
    pub arrow_uncentered: Memo<bool>,
    /// `side` — the logical rendered side.
    pub side: Memo<Side>,
    /// `align` — the rendered alignment.
    pub align: Memo<Align>,
    /// `physicalSide` — the engine's rendered side.
    pub physical_side: Memo<PhysicalSide>,
    /// `anchorHidden` — the custom `hide` middleware's report.
    pub anchor_hidden: Memo<bool>,
    /// `refs` — the extended refs (the setters live on the context).
    pub refs: ExtendedRefs,
    /// `context` — the shared floating context.
    pub context: Rc<FloatingContext>,
    /// `isPositioned` — the raw engine signal (the positioner styles apply the
    /// ending-phase compensation; see the module docs).
    pub is_positioned: RwSignal<bool>,
    /// `update` — the engine's recompute handle.
    pub update: Rc<dyn Fn()>,
}

/// `getLogicalSide` (`useAnchorPositioning.ts:38-50`): maps the engine's physical
/// rendered side back to the logical vocabulary, honoring the RTL swap of the logical
/// sides when the requested side was a logical one.
pub fn get_logical_side(side_param: Side, rendered_side: PhysicalSide, is_rtl: bool) -> Side {
    let logical_right = if is_rtl {
        Side::InlineStart
    } else {
        Side::InlineEnd
    };
    let logical_left = if is_rtl {
        Side::InlineEnd
    } else {
        Side::InlineStart
    };
    let is_logical_side_param = matches!(side_param, Side::InlineStart | Side::InlineEnd);
    match rendered_side {
        PhysicalSide::Top => Side::Top,
        PhysicalSide::Right => {
            if is_logical_side_param {
                logical_right
            } else {
                Side::Right
            }
        }
        PhysicalSide::Bottom => Side::Bottom,
        PhysicalSide::Left => {
            if is_logical_side_param {
                logical_left
            } else {
                Side::Left
            }
        }
    }
}

/// The physical side the requested logical side maps to before any flip
/// (`useAnchorPositioning.ts:185-196`).
pub fn physical_side_for_param(side_param: Side, is_rtl: bool) -> PhysicalSide {
    match side_param {
        Side::Top => PhysicalSide::Top,
        Side::Right => PhysicalSide::Right,
        Side::Bottom => PhysicalSide::Bottom,
        Side::Left => PhysicalSide::Left,
        Side::InlineEnd => {
            if is_rtl {
                PhysicalSide::Left
            } else {
                PhysicalSide::Right
            }
        }
        Side::InlineStart => {
            if is_rtl {
                PhysicalSide::Right
            } else {
                PhysicalSide::Left
            }
        }
    }
}

/// The initial `placement` (`useAnchorPositioning.ts:198`):
/// `align === 'center' ? side : `${side}-${align}``.
pub fn placement_for(side: PhysicalSide, align: Align) -> Placement {
    match (side, align) {
        (PhysicalSide::Top, Align::Start) => Placement::TopStart,
        (PhysicalSide::Top, Align::Center) => Placement::Top,
        (PhysicalSide::Top, Align::End) => Placement::TopEnd,
        (PhysicalSide::Right, Align::Start) => Placement::RightStart,
        (PhysicalSide::Right, Align::Center) => Placement::Right,
        (PhysicalSide::Right, Align::End) => Placement::RightEnd,
        (PhysicalSide::Bottom, Align::Start) => Placement::BottomStart,
        (PhysicalSide::Bottom, Align::Center) => Placement::Bottom,
        (PhysicalSide::Bottom, Align::End) => Placement::BottomEnd,
        (PhysicalSide::Left, Align::Start) => Placement::LeftStart,
        (PhysicalSide::Left, Align::Center) => Placement::Left,
        (PhysicalSide::Left, Align::End) => Placement::LeftEnd,
    }
}

/// Whether `shift` comes before `flip` in the middleware stack
/// (`useAnchorPositioning.ts:339-348` — the floating-ui combining note: when shift is
/// preferred or the alignment is centered, shift must run first or it undoes the
/// flip's alignment changes).
fn shift_precedes_flip(
    avoidance_side: CollisionAvoidanceSide,
    avoidance_align: CollisionAvoidanceAlign,
    align: Align,
) -> bool {
    avoidance_side == CollisionAvoidanceSide::Shift
        || avoidance_align == CollisionAvoidanceAlign::Shift
        || align == Align::Center
}

/// `getOffsetData` (`useAnchorPositioning.ts:52-61`).
fn get_offset_data(
    state: &MiddlewareState<Element, Window>,
    side_param: Side,
    is_rtl: bool,
) -> OffsetData {
    OffsetData {
        side: get_logical_side(
            side_param,
            floating_ui_utils::get_side(state.placement),
            is_rtl,
        ),
        align: Align::from(floating_ui_utils::get_alignment(state.placement)),
        anchor: floating_ui_utils::Dimensions {
            width: state.rects.reference.width,
            height: state.rects.reference.height,
        },
        positioner: floating_ui_utils::Dimensions {
            width: state.rects.floating.width,
            height: state.rects.floating.height,
        },
    }
}

/// The public hook (`useAnchorPositioning.ts:128-132`): upstream injects
/// `useBaseUIFloating`; the port binds [`use_base_ui_floating`] directly.
pub fn use_anchor_positioning(params: UseAnchorPositioningParams) -> UseAnchorPositioningReturn {
    use_anchor_positioning_with_hook(params)
}

/// `useAnchorPositioningWithHook` (`useAnchorPositioning.ts:134-635`): upstream's
/// test seam accepts any `useFloating`-shaped hook; the port's engine binding is the
/// only implementation, so the injection parameter is dropped (the seam remains
/// documented in the behavior spec's citation of the upstream test import).
pub fn use_anchor_positioning_with_hook(
    params: UseAnchorPositioningParams,
) -> UseAnchorPositioningReturn {
    let UseAnchorPositioningParams {
        anchor,
        position_method,
        side: side_param,
        side_offset,
        align_offset,
        align,
        collision_boundary,
        collision_padding: collision_padding_param,
        sticky,
        arrow_padding,
        disable_anchor_tracking,
        inline: inline_middleware,
        keep_mounted,
        floating_root_context,
        mounted,
        collision_avoidance,
        shift,
        node_id: _,
        adaptive_origin,
        lazy_flip,
        external_tree: _,
    } = params;

    // `mountSide` (`:164-168`) — the lazyFlip sticky-side lock. The render-phase
    // `!mounted` reset becomes a plain effect watching `mounted` (see the module docs).
    let mount_side: RwSignal<Option<PhysicalSide>, LocalStorage> = RwSignal::new_local(None);
    {
        let mount_side = mount_side.clone();
        let mounted = mounted.clone();
        reactive_graph::effect::Effect::new(move |_| {
            if !mounted.get() && mount_side.get_untracked().is_some() {
                mount_side.set(None);
            }
        });
    }

    // `:170-174`.
    let avoidance_side = collision_avoidance.side();
    let avoidance_align = collision_avoidance.align();
    let avoidance_fallback_axis_side = collision_avoidance.fallback_axis_side();
    let shift_cross_axis = shift.map(|config| config.cross_axis).unwrap_or(false);

    // `:176-180` — the stable-callback indirection dissolves (module docs);
    // `anchorDep` is the anchor itself.
    let anchor_value_ref: ValueAsRef<Anchor> = use_value_as_ref(RwSignal::new_local(anchor));
    let mounted_ref: ValueAsRef<bool> = use_value_as_ref(mounted);

    // `:182-183`.
    let direction = use_direction();
    let is_rtl = Signal::derive(move || direction.get() == TextDirection::Rtl);

    // `:185-198` — the requested physical side (sticky when `mountSide` is locked) and
    // the initial placement.
    let side: Signal<PhysicalSide, LocalStorage> = Signal::derive_local(move || {
        mount_side
            .get()
            .unwrap_or_else(|| physical_side_for_param(side_param, is_rtl.get_untracked()))
    });
    let placement: Signal<Placement> =
        Signal::derive(move || placement_for(side.get_untracked(), align));

    // `:200-221` + `:223-232` — the normalized padding and the iOS keyboard bias
    // (only applied to `flip()`, so the resting position shift/size compute stays at
    // the requested `collisionPadding`).
    let collision_padding = normalize_collision_padding(&collision_padding_param);
    let bias = 1.0;
    let bias_top = f64::from(side_param == Side::Bottom);
    let bias_bottom = f64::from(side_param == Side::Top);
    let bias_left = f64::from(side_param == Side::Right);
    let bias_right = f64::from(side_param == Side::Left);

    // `:234-237` — the shared collision boundary.
    let engine_boundary = collision_boundary.to_engine_boundary();

    // `:242` — the arrow slot ("a ref assumes that the arrow element is always present
    // in the DOM for the lifetime of the popup").
    let arrow_ref: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));

    // `:244-248` — the live offset reads (`sideOffsetDep`/`alignOffsetDep` dissolve:
    // the values flow through the refs on every engine pass).
    let side_offset_ref: ValueAsRef<SideOffset> =
        use_value_as_ref(RwSignal::new_local(side_offset));
    let align_offset_ref: ValueAsRef<SideOffset> =
        use_value_as_ref(RwSignal::new_local(align_offset));

    // The middleware stack (`:250-449`) — built once; live values are read through
    // the captured handles on every engine pass (module docs).
    let mut middleware: Vec<Box<dyn Middleware<Element, Window>>> = Vec::new();

    // `:252-254`.
    if let Some(inline) = inline_middleware {
        middleware.push(inline);
    }

    // `:256-278` — the offset middleware, function-based so the offset callbacks read
    // live rects.
    {
        let side_offset_ref = side_offset_ref.clone();
        let align_offset_ref = align_offset_ref.clone();
        let is_rtl = is_rtl.clone();
        let offset_fn: DerivableFn<'static, Element, Window, OffsetOptions> =
            Box::leak(Box::new(move |state: MiddlewareState<Element, Window>| {
                let data = get_offset_data(&state, side_param, is_rtl.get_untracked());
                OffsetOptions::Values(OffsetOptionsValues {
                    main_axis: Some(side_offset_ref.current().resolve(&data)),
                    cross_axis: Some(align_offset_ref.current().resolve(&data)),
                    alignment_axis: Some(align_offset_ref.current().resolve(&data)),
                })
            }));
        middleware.push(Box::new(Offset::new_derivable_fn(offset_fn)));
    }

    // `:280-282`.
    let shift_disabled = avoidance_align == CollisionAvoidanceAlign::None
        && avoidance_side != CollisionAvoidanceSide::Shift;
    let cross_axis_shift_enabled = !shift_disabled
        && (sticky || shift_cross_axis || avoidance_side == CollisionAvoidanceSide::Shift);

    // `:284-300` — the flip middleware (padding raised by the bias so the bias only
    // influences the flip decision, not the resting position).
    let flip_middleware = (avoidance_side != CollisionAvoidanceSide::None).then(|| {
        Flip::new(FlipOptions {
            detect_overflow: Some(DetectOverflowOptions {
                boundary: Some(engine_boundary.clone()),
                root_boundary: None,
                element_context: None,
                alt_boundary: None,
                padding: Some(Padding::PerSide(PartialSideObject {
                    top: Some(collision_padding.top + bias + bias_top),
                    right: Some(collision_padding.right + bias + bias_right),
                    bottom: Some(collision_padding.bottom + bias + bias_bottom),
                    left: Some(collision_padding.left + bias + bias_left),
                })),
            }),
            // `!shiftCrossAxis && collisionAvoidanceSide === 'flip'` (`:297`).
            main_axis: Some(!shift_cross_axis && avoidance_side == CollisionAvoidanceSide::Flip),
            cross_axis: Some(if avoidance_align == CollisionAvoidanceAlign::Flip {
                CrossAxis::Alignment
            } else {
                CrossAxis::False
            }),
            fallback_placements: None,
            fallback_strategy: None,
            // `'none'` disallows the perpendicular fallback — the Rust option's `None`.
            fallback_axis_side_direction: match avoidance_fallback_axis_side {
                FallbackAxisSide::None => None,
                FallbackAxisSide::Start => Some(Alignment::Start),
                FallbackAxisSide::End => Some(Alignment::End),
            },
            flip_alignment: None,
        })
    });

    // `:301-337` — the shift middleware, with the arrow-aware `limitShift` limiter
    // unless sticky/cross-axis shifting disabled it (`limiter: undefined`).
    let shift_middleware = (!shift_disabled).then(|| {
        let limiter = if sticky || shift_cross_axis {
            None
        } else {
            let arrow_ref = Rc::clone(&arrow_ref);
            let limit_fn: DerivableFn<'static, Element, Window, LimitShiftOffset> =
                Box::leak(Box::new(move |state: MiddlewareState<Element, Window>| {
                    let current = arrow_ref.borrow().clone();
                    let Some(arrow_element) = current.as_ref() else {
                        // `if (!arrowRef.current) { return {}; }` (`:314-316`) —
                        // an empty offset object, i.e. both axes absent.
                        return LimitShiftOffset::Values(LimitShiftOffsetValues {
                            main_axis: None,
                            cross_axis: None,
                        });
                    };
                    let rect = arrow_element.get_bounding_client_rect();
                    let side_axis = floating_ui_utils::get_side_axis(state.placement);
                    let arrow_size = if side_axis == Axis::Y {
                        rect.width()
                    } else {
                        rect.height()
                    };
                    let offset_amount = if side_axis == Axis::Y {
                        collision_padding.left + collision_padding.right
                    } else {
                        collision_padding.top + collision_padding.bottom
                    };
                    LimitShiftOffset::Value(arrow_size / 2.0 + offset_amount / 2.0)
                }));
            Some(Box::new(LimitShift::new(LimitShiftOptions {
                offset: Some(Derivable::Fn(limit_fn)),
                main_axis: None,
                cross_axis: None,
            })) as Box<dyn Limiter<Element, Window>>)
        };
        Shift::new(ShiftOptions {
            detect_overflow: Some(DetectOverflowOptions {
                boundary: Some(engine_boundary.clone()),
                // "Use the Layout Viewport to avoid shifting around when
                // pinch-zooming" (`:306`) — not expressible in the external crate
                // (module docs): both configurations land on the viewport boundary,
                // which is also the engine default for the undefined case.
                root_boundary: Some(RootBoundary::Viewport),
                element_context: None,
                alt_boundary: None,
                padding: Some(collision_padding.to_padding()),
            }),
            // `collisionAvoidanceAlign !== 'none'` (`:308`).
            main_axis: Some(avoidance_align != CollisionAvoidanceAlign::None),
            cross_axis: Some(cross_axis_shift_enabled),
            limiter,
        })
    });

    // `:339-348` — the collision middlewares, in the avoidance-dependent order.
    if shift_precedes_flip(avoidance_side, avoidance_align, align) {
        if let Some(shift) = shift_middleware {
            middleware.push(Box::new(shift));
        }
        if let Some(flip) = flip_middleware {
            middleware.push(Box::new(flip));
        }
    } else {
        if let Some(flip) = flip_middleware {
            middleware.push(Box::new(flip));
        }
        if let Some(shift) = shift_middleware {
            middleware.push(Box::new(shift));
        }
    }

    // `:350-371` — the size middleware: the `--available-*` vars plus the DPR-snapped
    // `--anchor-*` vars, gated on the mounted ref.
    {
        let mounted_ref = mounted_ref.clone();
        let apply: &'static ApplyFn<Element, Window> =
            Box::leak(Box::new(move |apply_state: ApplyState<Element, Window>| {
                if !mounted_ref.current() {
                    return;
                }
                let floating = apply_state.state.elements.floating;
                let floating_style = element_style(floating);
                let _ = floating_style.set_property(
                    common_positioner_css_vars::AVAILABLE_WIDTH,
                    &px(apply_state.available_width),
                );
                let _ = floating_style.set_property(
                    common_positioner_css_vars::AVAILABLE_HEIGHT,
                    &px(apply_state.available_height),
                );

                // Snap anchor dimensions to device pixels (`:362-366`) so the popup's
                // visual width matches the anchor's one.
                let dpr = dom::get_window(Some(floating)).device_pixel_ratio();
                let rect = &apply_state.state.rects.reference;
                let anchor_width =
                    (((rect.x + rect.width) * dpr).round() - (rect.x * dpr).round()) / dpr;
                let anchor_height =
                    (((rect.y + rect.height) * dpr).round() - (rect.y * dpr).round()) / dpr;
                let _ = floating_style
                    .set_property(common_positioner_css_vars::ANCHOR_WIDTH, &px(anchor_width));
                let _ = floating_style.set_property(
                    common_positioner_css_vars::ANCHOR_HEIGHT,
                    &px(anchor_height),
                );
            }));
        middleware.push(Box::new(Size::new(SizeOptions {
            detect_overflow: Some(DetectOverflowOptions {
                boundary: Some(engine_boundary.clone()),
                root_boundary: None,
                element_context: None,
                alt_boundary: None,
                padding: Some(collision_padding.to_padding()),
            }),
            apply: Some(apply),
        })));
    }

    // `:372-382` — the arrow middleware: the live arrow element, or a fake one so
    // `transform-origin` still has an anchor; no padding for the fake arrow ("it would
    // displace aligned popups on narrow anchors").
    {
        let arrow_ref = Rc::clone(&arrow_ref);
        let arrow_fn: DerivableFn<'static, Element, Window, ArrowOptions<Element>> =
            Box::leak(Box::new(move |state: MiddlewareState<Element, Window>| {
                let current = arrow_ref.borrow().clone();
                let padding = if current.is_some() {
                    arrow_padding
                } else {
                    0.0
                };
                ArrowOptions {
                    element: Some(current.unwrap_or_else(|| {
                        owner_document(Some(state.elements.floating.as_ref() as &Node))
                            .create_element("div")
                            .expect("createElement('div') failed")
                    })),
                    padding: Some(Padding::All(padding)),
                    offset_parent: OffsetParent::Floating,
                }
            }));
        middleware.push(Box::new(BaseArrow::new_derivable_fn(arrow_fn)));
    }

    // `:383-446` — the custom `transformOrigin` middleware.
    middleware.push(Box::new(TransformOriginMiddleware {
        arrow_ref: Rc::clone(&arrow_ref),
        side_offset: side_offset_ref.current(),
        side_param,
        is_rtl: is_rtl.clone(),
        cross_axis_shift_enabled,
    }));

    // `:447-448`.
    middleware.push(Box::new(hide()));
    if let Some(adaptive_origin) = adaptive_origin {
        middleware.push(adaptive_origin);
    }

    let middleware_signal: Signal<WrappedMiddleware> =
        RwSignal::new(SendWrapper::new(middleware)).into();

    // `:451-462` — the keepMounted nulling: a closed, keepMounted popup must not run
    // positioning on hidden elements.
    {
        let store = Rc::clone(&floating_root_context);
        let mounted = mounted.clone();
        use_iso_layout_effect(move || {
            if !mounted.get() {
                store.update(|state, _| {
                    let changed = state.reference_element.is_some()
                        || state.floating_element.is_some()
                        || state.dom_reference_element.is_some()
                        || state.position_reference.is_some();
                    state.reference_element = None;
                    state.floating_element = None;
                    state.dom_reference_element = None;
                    state.position_reference = None;
                    changed
                });
            }
        });
    }

    // `:464-471` — the autoUpdate options (`typeof ResizeObserver !== 'undefined'`
    // checks over the globals).
    let auto_update_options = AutoUpdateOptions {
        ancestor_scroll: Some(!disable_anchor_tracking),
        ancestor_resize: None,
        element_resize: Some(!disable_anchor_tracking && has_global("ResizeObserver")),
        layout_shift: Some(!disable_anchor_tracking && has_global("IntersectionObserver")),
        animation_frame: None,
    };

    // `:473-495` — the engine call over the required root store (`useBaseUIFloating`).
    // `open: keepMounted ? mounted : undefined` reaches the engine upstream as the
    // forwarded option; the port's engine gates on the store's open and the
    // positioner-styles derivation compensates (module docs).
    let result = use_base_ui_floating(
        UsePositionOptions {
            placement: placement.into(),
            strategy: Signal::derive(move || position_method),
            middleware: middleware_signal,
            transform: Signal::derive(|| true),
            while_elements_mounted: (!keep_mounted).then(|| {
                let auto_update_options = auto_update_options.clone();
                Rc::new(
                    move |reference: &ReferenceType,
                          floating: &Element,
                          update: Rc<dyn Fn()>|
                          -> Rc<dyn Fn()> {
                        let engine_reference = match reference {
                            ReferenceType::Element(element) => ElementOrVirtual::Element(element),
                            ReferenceType::Virtual(virtual_reference) => {
                                ElementOrVirtual::VirtualElement(virtual_reference.element.clone())
                            }
                        };
                        let cleanup = auto_update(
                            engine_reference,
                            floating,
                            update,
                            auto_update_options.clone(),
                        );
                        Rc::new(move || cleanup()) as Rc<dyn Fn()>
                    },
                )
                    as Rc<dyn Fn(&ReferenceType, &Element, Rc<dyn Fn()>) -> Rc<dyn Fn()>>
            }),
        },
        Rc::clone(&floating_root_context),
    );

    let is_positioned = result.is_positioned;
    let middleware_data = result.middleware_data.clone();
    let placement_rendered = result.placement.read_only();
    let x = result.x.read_only();
    let y = result.y.read_only();
    let original_floating_styles = result.floating_styles;
    let store_open = result.context.open.clone();

    // The ending-phase latch (module docs): upstream's engine receives
    // `open: mounted`/`undefined`, so its `isPositioned` survives the store-open reset
    // through an exit transition and a re-open gap; the port's engine gates on the
    // store's open. The latch records "has positioned for this mount" (cleared once
    // unmounted), the positioner styles use it to hold the real coordinates through an
    // ending phase, and the effect re-runs the engine when a re-open would otherwise
    // leave a surviving popup unpositioned (upstream recomputed under the same
    // conditions because its engine saw no open flip at all).
    let ever_positioned: RwSignal<bool, LocalStorage> = RwSignal::new_local(false);
    {
        let ever = ever_positioned.clone();
        let store_open = store_open.clone();
        let update = Rc::clone(&result.update);
        let mounted = mounted.clone();
        reactive_graph::effect::Effect::new(move |_| {
            if mounted.get() && is_positioned.get() {
                ever.set(true);
            }
            if !mounted.get() && ever.get_untracked() {
                ever.set(false);
            }
            if mounted.get() && store_open.get() && !is_positioned.get() {
                update();
            }
        });
    }

    // `:497` — the adaptive-origin side properties
    // (`middlewareData.adaptiveOrigin || DEFAULT_SIDES`).
    let adaptive_sides = Memo::new(move |_| {
        AdaptiveOriginData::from_middleware_data(middleware_data.get().get(
            // The data key is the custom middleware's `name` — upstream reads
            // `middlewareData.adaptiveOrigin`.
            "adaptiveOrigin",
        ))
    });
    let adaptive_origin_present =
        Memo::new(move |_| middleware_data.get().get("adaptiveOrigin").is_some());

    // `:503-533` — the positioner styles. `positioned` carries the ending-phase
    // compensation (module docs): `isPositioned || (!storeOpen && mounted)`.
    let positioner_styles: Signal<PositionerStyles, LocalStorage> = {
        let adaptive_sides = adaptive_sides.clone();
        let adaptive_origin_present = adaptive_origin_present.clone();
        Signal::derive_local(move || {
            // The ending-phase compensation (module docs): `isPositioned`, or a popup
            // that already positioned for this mount and is still mounted while the
            // store has closed (the exit transition window).
            let positioned = is_positioned.get()
                || (ever_positioned.get_untracked() && mounted.get() && !store_open.get());
            let resolved_position = if positioned {
                position_method
            } else {
                Strategy::Fixed
            };
            let x = x.get();
            let y = y.get();

            let mut styles = if !positioned {
                // Until a position for the current open is computed, ignore any
                // coordinates retained from a previous open (or from a pass that
                // measured the hidden popup as 0x0) (`:504-510`).
                PositionerStyles {
                    position: resolved_position,
                    top: Some("0px".to_owned()),
                    left: Some("0px".to_owned()),
                    right: None,
                    bottom: None,
                    transform: None,
                    will_change: None,
                    opacity: Some("0".to_owned()),
                    available_width: String::new(),
                    available_height: String::new(),
                }
            } else if adaptive_origin_present.get() {
                // `base = { position, [sideX]: x, [sideY]: y }` (`:511-512`).
                let sides = adaptive_sides.get();
                let mut styles = PositionerStyles {
                    position: resolved_position,
                    top: None,
                    left: None,
                    right: None,
                    bottom: None,
                    transform: None,
                    will_change: None,
                    opacity: None,
                    available_width: String::new(),
                    available_height: String::new(),
                };
                if sides.side_x == "right" {
                    styles.right = Some(px(x));
                } else {
                    styles.left = Some(px(x));
                }
                if sides.side_y == "bottom" {
                    styles.bottom = Some(px(y));
                } else {
                    styles.top = Some(px(y));
                }
                styles
            } else {
                // `{ ...originalFloatingStyles, position: resolvedPosition }` (`:514`).
                let original = original_floating_styles.get();
                PositionerStyles {
                    position: resolved_position,
                    top: Some(original.top),
                    left: Some(original.left),
                    right: None,
                    bottom: None,
                    transform: original.transform,
                    will_change: original.will_change,
                    opacity: None,
                    available_width: String::new(),
                    available_height: String::new(),
                }
            };

            // The unconditional `--available-*` seeds (`:517-527`).
            styles.available_width = "100vw".to_owned();
            styles.available_height = "100vh".to_owned();
            styles
        })
    };

    // `:535-552` — the anchor registration (layout phase), dedupe-guarded by the
    // registered reference.
    let registered_position_reference: RwSignal<Option<ReferenceType>, LocalStorage> =
        RwSignal::new_local(None);
    {
        let set_position_reference = Rc::clone(&result.context.set_position_reference);
        let registered = registered_position_reference.clone();
        let mounted = mounted.clone();
        let anchor_value_ref = anchor_value_ref.clone();
        use_iso_layout_effect(move || {
            if !mounted.get() {
                return;
            }
            let final_anchor = resolve_anchor(&anchor_value_ref.current());
            if !reference_matches(&final_anchor, &registered.get_untracked()) {
                set_position_reference(final_anchor.clone());
                registered.set(final_anchor);
            }
        });
    }

    // `:554-571` — the passive re-check (upstream: parent-populated refs; the port
    // re-resolves `Fn` anchors too — see the module docs).
    {
        let set_position_reference = Rc::clone(&result.context.set_position_reference);
        let registered = registered_position_reference.clone();
        let mounted = mounted.clone();
        let anchor_value_ref = anchor_value_ref.clone();
        reactive_graph::effect::Effect::new(move |_| {
            if !mounted.get() {
                return;
            }
            let final_anchor = resolve_anchor(&anchor_value_ref.current());
            if !reference_matches(&final_anchor, &registered.get_untracked()) {
                set_position_reference(final_anchor.clone());
                registered.set(final_anchor);
            }
        });
    }

    // `:573-578` — the keepMounted autoUpdate wiring (the engine's
    // `whileElementsMounted` is skipped in that mode upstream).
    if keep_mounted {
        let elements = result.elements.clone();
        let update = Rc::clone(&result.update);
        reactive_graph::effect::Effect::new(move |_| {
            if !mounted.get() {
                return;
            }
            let (Some(reference), Some(floating)) =
                (elements.reference.get(), elements.floating.get())
            else {
                return;
            };
            let engine_reference = match &reference {
                ReferenceType::Element(element) => ElementOrVirtual::Element(element),
                ReferenceType::Virtual(virtual_reference) => {
                    ElementOrVirtual::VirtualElement(virtual_reference.element.clone())
                }
            };
            let cleanup = auto_update(
                engine_reference,
                &floating,
                Rc::clone(&update),
                auto_update_options.clone(),
            );
            let cleanup = SendWrapper::new(cleanup);
            reactive_graph::owner::on_cleanup(move || cleanup());
        });
    }

    // `:580-583` — the rendered placement readback.
    let physical_side: Memo<PhysicalSide> =
        Memo::new(move |_| floating_ui_utils::get_side(placement_rendered.get()));
    let logical_side: Memo<Side> = Memo::new(move |_| {
        get_logical_side(side_param, physical_side.get(), is_rtl.get_untracked())
    });
    let rendered_align: Memo<Align> =
        Memo::new(move |_| Align::from(floating_ui_utils::get_alignment(placement_rendered.get())));
    let anchor_hidden: Memo<bool> = Memo::new(move |_| {
        middleware_data
            .get()
            .get(HIDE_NAME)
            .and_then(|data| data.get("referenceHidden"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    });

    // `:585-592` — the lazyFlip sticky-side lock: flips back lazily, not eagerly.
    {
        let mount_side = mount_side.clone();
        let side = side.clone();
        use_iso_layout_effect(move || {
            if lazy_flip
                && mounted.get()
                && is_positioned.get()
                && physical_side.get() != side.get()
            {
                mount_side.set(Some(physical_side.get_untracked()));
            }
        });
    }

    // `:594-603` — the arrow styles and centering report.
    let arrow_styles: Signal<ArrowStyles, LocalStorage> = {
        let middleware_data = middleware_data.clone();
        Signal::derive_local(move || {
            let arrow_data: Option<ArrowData> =
                middleware_data.get().get_as(floating_ui_dom::ARROW_NAME);
            ArrowStyles {
                position: "absolute",
                top: arrow_data.as_ref().and_then(|data| data.y).map(px),
                left: arrow_data.as_ref().and_then(|data| data.x).map(px),
            }
        })
    };
    let arrow_uncentered: Memo<bool> = {
        let middleware_data = middleware_data.clone();
        Memo::new(move |_| {
            middleware_data
                .get()
                .get_as::<ArrowData>(floating_ui_dom::ARROW_NAME)
                .map(|data| data.center_offset != 0.0)
                .unwrap_or(false)
        })
    };

    UseAnchorPositioningReturn {
        positioner_styles,
        arrow_styles,
        arrow_ref,
        arrow_uncentered,
        side: logical_side,
        align: rendered_align,
        physical_side,
        anchor_hidden,
        refs: result.refs.clone(),
        context: Rc::clone(&result.context),
        is_positioned,
        update: Rc::clone(&result.update),
    }
}

/// Formats an engine coordinate the way React formats numeric CSS values.
fn px(value: f64) -> String {
    format!("{}px", value)
}

/// The floating element's inline style — upstream reads `floating.style` off an
/// `HTMLElement` (`useAnchorPositioning.ts:358,439`); the engine's element is a plain
/// `Element` here (it may be SVG), so the property is read dynamically and cast.
fn element_style(element: &Element) -> web_sys::CssStyleDeclaration {
    use wasm_bindgen::JsCast;
    js_sys::Reflect::get(element, &wasm_bindgen::JsValue::from_str("style"))
        .expect("the element has an inline style")
        .unchecked_into()
}

/// The `typeof ResizeObserver !== 'undefined'` / `typeof IntersectionObserver !==
/// 'undefined'` checks (`useAnchorPositioning.ts:467-468`).
fn has_global(name: &str) -> bool {
    js_sys::Reflect::has(&js_sys::global(), &wasm_bindgen::JsValue::from_str(name)).unwrap_or(false)
}

/// The custom `transformOrigin` middleware (`useAnchorPositioning.ts:383-446`): writes
/// the `--transform-origin` var from the arrow/alignment/shift geometry so consumer
/// scale-in animations grow from the right point.
struct TransformOriginMiddleware {
    arrow_ref: Rc<RefCell<Option<Element>>>,
    side_offset: SideOffset,
    side_param: Side,
    is_rtl: Signal<bool>,
    cross_axis_shift_enabled: bool,
}

impl Clone for TransformOriginMiddleware {
    fn clone(&self) -> Self {
        TransformOriginMiddleware {
            arrow_ref: Rc::clone(&self.arrow_ref),
            side_offset: self.side_offset.clone(),
            side_param: self.side_param,
            is_rtl: self.is_rtl.clone(),
            cross_axis_shift_enabled: self.cross_axis_shift_enabled,
        }
    }
}

impl PartialEq for TransformOriginMiddleware {
    fn eq(&self, other: &Self) -> bool {
        self.side_param == other.side_param
            && self.is_rtl.get_untracked() == other.is_rtl.get_untracked()
            && self.cross_axis_shift_enabled == other.cross_axis_shift_enabled
            && self.side_offset == other.side_offset
            && Rc::ptr_eq(&self.arrow_ref, &other.arrow_ref)
    }
}

impl Middleware<Element, Window> for TransformOriginMiddleware {
    fn name(&self) -> &'static str {
        "transformOrigin"
    }

    fn compute(&self, state: MiddlewareState<Element, Window>) -> MiddlewareReturn {
        let floating = state.elements.floating;
        let rendered_side = floating_ui_utils::get_side(state.placement);
        let rendered_align = Align::from(floating_ui_utils::get_alignment(state.placement));
        let is_vertical = floating_ui_utils::get_side_axis(state.placement) == Axis::Y;
        let arrow_element = self.arrow_ref.borrow().clone();

        let side_offset_value = self.side_offset.resolve(&get_offset_data(
            &state,
            self.side_param,
            self.is_rtl.get_untracked(),
        ));

        let shift_data: Option<ShiftData> = state.middleware_data.get_as("shift");
        let arrow_data: Option<ArrowData> =
            state.middleware_data.get_as(floating_ui_dom::ARROW_NAME);
        let shift_x = shift_data.as_ref().map(|data| data.x).unwrap_or(0.0);
        let shift_y = shift_data.as_ref().map(|data| data.y).unwrap_or(0.0);

        // An aligned arrowless popup grows from its aligned edge, until a shift
        // (beyond subpixel) breaks its alignment with the anchor. Everything else
        // grows from the arrow, real or fake (`:405-424`).
        let cross_origin: String = if arrow_element.is_none()
            && rendered_align != Align::Center
            && (if is_vertical { shift_x } else { shift_y }).abs() <= 1.0
        {
            // The platform direction, not `isRtl`: it must match what Floating UI
            // placed with (`:413`).
            let platform_rtl = state.platform.is_rtl(floating).unwrap_or(false);
            if (rendered_align == Align::Start) == (is_vertical && platform_rtl) {
                "100%".to_owned()
            } else {
                "0%".to_owned()
            }
        } else {
            let arrow_offset = if is_vertical {
                arrow_data.as_ref().and_then(|data| data.x).unwrap_or(0.0)
            } else {
                arrow_data.as_ref().and_then(|data| data.y).unwrap_or(0.0)
            };
            let arrow_size = match arrow_element.as_ref() {
                Some(element) if is_vertical => element.client_width() as f64,
                Some(element) => element.client_height() as f64,
                None => 0.0,
            };
            format!("{}px", arrow_offset + arrow_size / 2.0)
        };

        // Side axis: the anchor-facing edge, or the anchor's center when the popup
        // overlaps it (`:426-437`).
        let mut side_origin: String =
            if rendered_side == PhysicalSide::Top || rendered_side == PhysicalSide::Left {
                format!("calc(100% + {}px)", side_offset_value)
            } else {
                format!("{}px", -side_offset_value)
            };
        if self.cross_axis_shift_enabled && is_vertical && shift_y.abs() > side_offset_value {
            side_origin = format!(
                "{}px",
                state.rects.reference.y + state.rects.reference.height / 2.0 - state.y
            );
        }

        let value = if is_vertical {
            format!("{} {}", cross_origin, side_origin)
        } else {
            format!("{} {}", side_origin, cross_origin)
        };
        let _ = element_style(floating)
            .set_property(common_positioner_css_vars::TRANSFORM_ORIGIN, &value);

        MiddlewareReturn {
            x: None,
            y: None,
            data: None,
            reset: None,
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use crate::floating_ui::types::VirtualReference;
    use floating_ui_dom::{ClientRectObject, VirtualElement};

    // `getLogicalSide` (`useAnchorPositioning.ts:38-50`): physical sides pass through
    // unless the requested side was logical, in which case the rendered side maps back
    // through the RTL swap.
    #[test]
    fn get_logical_side_passes_physical_sides_through() {
        assert_eq!(
            get_logical_side(Side::Top, PhysicalSide::Top, false),
            Side::Top
        );
        assert_eq!(
            get_logical_side(Side::Bottom, PhysicalSide::Bottom, true),
            Side::Bottom
        );
        assert_eq!(
            get_logical_side(Side::Left, PhysicalSide::Left, false),
            Side::Left
        );
        assert_eq!(
            get_logical_side(Side::Right, PhysicalSide::Right, true),
            Side::Right
        );
    }

    #[test]
    fn get_logical_side_maps_rendered_physical_sides_for_logical_params() {
        // The rendered side maps back onto the same logical side, honoring the RTL
        // swap of the logical sides.
        assert_eq!(
            get_logical_side(Side::InlineEnd, PhysicalSide::Right, false),
            Side::InlineEnd
        );
        assert_eq!(
            get_logical_side(Side::InlineEnd, PhysicalSide::Left, true),
            Side::InlineEnd
        );
        // A flip moved the rendered side to the opposite physical edge: the logical
        // mapping follows the *rendered* side, so an inline-start request whose flip
        // landed on the LTR left edge still reads inline-start (LTR: left IS the
        // logical start).
        assert_eq!(
            get_logical_side(Side::InlineStart, PhysicalSide::Left, false),
            Side::InlineStart
        );
        assert_eq!(
            get_logical_side(Side::InlineStart, PhysicalSide::Right, true),
            Side::InlineStart
        );
        // Whereas the rendered side on the non-matching physical edge maps to the
        // opposite logical side.
        assert_eq!(
            get_logical_side(Side::InlineStart, PhysicalSide::Right, false),
            Side::InlineEnd
        );
        assert_eq!(
            get_logical_side(Side::InlineStart, PhysicalSide::Left, true),
            Side::InlineEnd
        );
        // A physical param never maps onto a logical result, even in RTL.
        assert_eq!(
            get_logical_side(Side::Left, PhysicalSide::Left, true),
            Side::Left
        );
        assert_eq!(
            get_logical_side(Side::Right, PhysicalSide::Right, true),
            Side::Right
        );
    }

    // `:185-196` — the pre-flip physical mapping.
    #[test]
    fn physical_side_for_param_respects_rtl() {
        assert_eq!(
            physical_side_for_param(Side::InlineEnd, false),
            PhysicalSide::Right
        );
        assert_eq!(
            physical_side_for_param(Side::InlineEnd, true),
            PhysicalSide::Left
        );
        assert_eq!(
            physical_side_for_param(Side::InlineStart, false),
            PhysicalSide::Left
        );
        assert_eq!(
            physical_side_for_param(Side::InlineStart, true),
            PhysicalSide::Right
        );
        assert_eq!(physical_side_for_param(Side::Top, true), PhysicalSide::Top);
        assert_eq!(
            physical_side_for_param(Side::Bottom, false),
            PhysicalSide::Bottom
        );
    }

    // `:198` — the placement construction.
    #[test]
    fn placement_for_combines_side_and_align() {
        assert_eq!(
            placement_for(PhysicalSide::Bottom, Align::Center),
            Placement::Bottom
        );
        assert_eq!(
            placement_for(PhysicalSide::Bottom, Align::Start),
            Placement::BottomStart
        );
        assert_eq!(
            placement_for(PhysicalSide::Bottom, Align::End),
            Placement::BottomEnd
        );
        assert_eq!(
            placement_for(PhysicalSide::Left, Align::End),
            Placement::LeftEnd
        );
        assert_eq!(
            placement_for(PhysicalSide::Top, Align::Start),
            Placement::TopStart
        );
        assert_eq!(
            placement_for(PhysicalSide::Right, Align::End),
            Placement::RightEnd
        );
        assert_eq!(
            placement_for(PhysicalSide::Top, Align::Center),
            Placement::Top
        );
    }

    // `:200-221` — the padding normalization: numbers spread as-is, the object path
    // applies the `|| 0` falsy normalization.
    #[test]
    fn normalize_collision_padding_spreads_numbers_and_zeroes_falsy_sides() {
        assert_eq!(
            normalize_collision_padding(&CollisionPadding::All(5.0)),
            PaddingRect {
                top: 5.0,
                right: 5.0,
                bottom: 5.0,
                left: 5.0
            }
        );
        assert_eq!(
            normalize_collision_padding(&CollisionPadding::Sides {
                top: 1.0,
                right: 0.0,
                bottom: f64::NAN,
                left: 2.5
            }),
            PaddingRect {
                top: 1.0,
                right: 0.0,
                bottom: 0.0,
                left: 2.5
            }
        );
    }

    // `:223-232` — the iOS keyboard bias targets the preferred side's opposite edge
    // so flip() out-prioritizes size()'s smaller padding there and only there.
    #[test]
    fn flip_bias_targets_the_preferred_side() {
        let padding = PaddingRect {
            top: 5.0,
            right: 5.0,
            bottom: 5.0,
            left: 5.0,
        };
        let bias = 1.0;
        let side_param = Side::Bottom;
        let bias_top = f64::from(side_param == Side::Bottom);
        let bias_bottom = f64::from(side_param == Side::Top);
        let bias_left = f64::from(side_param == Side::Right);
        let bias_right = f64::from(side_param == Side::Left);
        assert_eq!(
            padding.top + bias + bias_top,
            7.0,
            "the flip side gets the extra headroom"
        );
        assert_eq!(padding.bottom + bias + bias_bottom, 6.0);
        assert_eq!(padding.left + bias + bias_left, 6.0);
        assert_eq!(padding.right + bias + bias_right, 6.0);
    }

    // `:339-348` — the stack-order decision.
    #[test]
    fn shift_precedes_flip_for_shift_preferences_and_centered_alignment() {
        assert!(shift_precedes_flip(
            CollisionAvoidanceSide::Shift,
            CollisionAvoidanceAlign::Flip,
            Align::Start
        ));
        assert!(shift_precedes_flip(
            CollisionAvoidanceSide::Flip,
            CollisionAvoidanceAlign::Shift,
            Align::Start
        ));
        assert!(shift_precedes_flip(
            CollisionAvoidanceSide::Flip,
            CollisionAvoidanceAlign::Flip,
            Align::Center
        ));
        assert!(!shift_precedes_flip(
            CollisionAvoidanceSide::Flip,
            CollisionAvoidanceAlign::Flip,
            Align::Start
        ));
        assert!(!shift_precedes_flip(
            CollisionAvoidanceSide::Flip,
            CollisionAvoidanceAlign::None,
            Align::End
        ));
    }

    // The constants-file presets map onto the full param type with the hook defaults
    // filling the absent fields.
    #[test]
    fn preset_collision_avoidance_converts_with_defaults_elsewhere() {
        let dropdown =
            CollisionAvoidance::from_preset(&crate::constants::DROPDOWN_COLLISION_AVOIDANCE);
        assert_eq!(dropdown.fallback_axis_side(), FallbackAxisSide::None);
        assert_eq!(dropdown.side(), CollisionAvoidanceSide::Flip);
        assert_eq!(dropdown.align(), CollisionAvoidanceAlign::Flip);

        let popup = CollisionAvoidance::from_preset(&crate::constants::POPUP_COLLISION_AVOIDANCE);
        assert_eq!(popup.fallback_axis_side(), FallbackAxisSide::End);
    }

    // `Anchor` default and resolution.
    #[test]
    fn resolve_anchor_handles_none_static_and_fn() {
        assert_eq!(resolve_anchor(&Anchor::None), None);

        #[derive(Clone, Debug, PartialEq)]
        struct FakeVirtualElement;
        impl VirtualElement<Element> for FakeVirtualElement {
            fn get_bounding_client_rect(&self) -> ClientRectObject {
                unimplemented!("not exercised by resolve_anchor")
            }
            fn get_client_rects(&self) -> Option<Vec<ClientRectObject>> {
                None
            }
            fn context_element(&self) -> Option<Element> {
                None
            }
        }

        let static_virtual = Anchor::Static(ReferenceType::Virtual(VirtualReference::new(
            FakeVirtualElement,
        )));
        let resolved = resolve_anchor(&static_virtual).expect("static anchors resolve");
        match resolved {
            ReferenceType::Virtual(virtual_reference) => {
                assert!(
                    virtual_reference.id > 0,
                    "the virtual identity token is set"
                );
            }
            ReferenceType::Element(_) => panic!("expected the virtual arm"),
        }

        let function = Anchor::Fn(Rc::new(|| {
            Some(ReferenceType::Virtual(VirtualReference::new(
                FakeVirtualElement,
            )))
        }));
        assert!(resolve_anchor(&function).is_some(), "fn anchors resolve");
    }

    // The registration dedupe (`useAnchorPositioning.ts:548`): elements compare by
    // DOM identity, virtual references by their identity token.
    #[test]
    fn reference_matches_compares_identity_not_content() {
        assert!(reference_matches(&None, &None));

        #[derive(Clone, Debug, PartialEq)]
        struct FakeVirtualElement;
        impl VirtualElement<Element> for FakeVirtualElement {
            fn get_bounding_client_rect(&self) -> ClientRectObject {
                unimplemented!("not exercised")
            }
            fn get_client_rects(&self) -> Option<Vec<ClientRectObject>> {
                None
            }
            fn context_element(&self) -> Option<Element> {
                None
            }
        }

        let first = Some(ReferenceType::Virtual(VirtualReference::new(
            FakeVirtualElement,
        )));
        let first_clone = first.clone();
        let second = Some(ReferenceType::Virtual(VirtualReference::new(
            FakeVirtualElement,
        )));
        assert!(
            reference_matches(&first, &first_clone),
            "a clone keeps the identity"
        );
        assert!(
            !reference_matches(&first, &second),
            "a fresh shim is a fresh identity"
        );
        assert!(!reference_matches(&first, &None));
        assert!(!reference_matches(&None, &second));
    }
}

// The hook's behaviors are DOM-bound (real rects, computed styles, viewport
// geometry): the wasm/browser suite pins them, the `use_position.rs` precedent.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::types::VirtualReference;

    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    use floating_ui_dom::VirtualElement;
    use wasm_bindgen_futures::JsFuture;

    fn init() -> reactive_graph::owner::Owner {
        let _ = any_spawner::Executor::init_futures_executor();
        // The suite's accumulated fixture DOM makes the page scrollable, and earlier
        // tests' scrolling leaves a nonzero offset — the engine's absolute-strategy
        // coordinates are document-relative (viewport + scroll), so the geometry
        // assertions need a reset page.
        web_sys::window().unwrap().scroll_with_x_and_y(0.0, 0.0);
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        owner
    }

    /// Disposes the test owner and unsets it so the next test's `Owner::new()` is a
    /// fresh root — a still-linked disposed parent would leak its contexts into the
    /// next test's `use_context` walk (the direction-provider context is global state
    /// per owner chain).
    fn finish(owner: reactive_graph::owner::Owner) {
        owner.cleanup();
        owner.unset();
    }

    /// One awaited resolved promise per microtask tick plus one executor poll — the
    /// `use_transition_status.rs` wasm-suite helper with the `use_press_and_hold.rs`
    /// `poll_local` addition: plain-effect re-runs land on the `futures_executor`
    /// `LocalPool`, which only advances when polled.
    async fn run_microtasks(ticks: usize) {
        for _ in 0..ticks {
            JsFuture::from(js_sys::Promise::resolve(&wasm_bindgen::JsValue::undefined()))
                .await
                .unwrap();
            any_spawner::Executor::poll_local();
        }
    }

    fn empty_store() -> Rc<FloatingRootStore> {
        FloatingRootStore::new(FloatingRootStoreOptions {
            open: false,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only: false,
            nested: false,
            on_open_change: None,
        })
    }

    fn body() -> web_sys::HtmlElement {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
    }

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn mount_element(style: &str) -> Element {
        let element = document().create_element("div").unwrap();
        element.set_attribute("style", style).unwrap();
        body().append_child(&element).unwrap();
        element
    }

    fn hook(
        store: &Rc<FloatingRootStore>,
        mounted: RwSignal<bool, LocalStorage>,
        adjust: impl FnOnce(&mut UseAnchorPositioningParams),
    ) -> UseAnchorPositioningReturn {
        let mut params = UseAnchorPositioningParams::new(Rc::clone(store), mounted.into());
        adjust(&mut params);
        use_anchor_positioning_with_hook(params)
    }

    /// Mounts a mid-viewport anchor and a popup, marks the store open, and runs one
    /// synchronous positioning pass.
    fn setup(
        mounted: RwSignal<bool, LocalStorage>,
        adjust: impl FnOnce(&mut UseAnchorPositioningParams),
    ) -> (
        Rc<FloatingRootStore>,
        Element,
        Element,
        UseAnchorPositioningReturn,
    ) {
        let store = empty_store();
        let anchor =
            mount_element("position: fixed; top: 100px; left: 100px; width: 60px; height: 40px;");
        let floating =
            mount_element("position: fixed; top: 0; left: 0; width: 100px; height: 50px;");
        let result = hook(&store, mounted, adjust);

        store.set_field(
            |state| &mut state.reference_element,
            Some(ReferenceType::Element(anchor.clone())),
        );
        store.set_field(|state| &mut state.floating_element, Some(floating.clone()));
        store.update(|state, _| {
            state.open = true;
            true
        });
        (result.update)();
        (store, anchor, floating, result)
    }

    // `useAnchorPositioning.ts:473-495,580-583` — the engine positions the elements,
    // the rendered readback reports bottom/center for the default params, and the
    // positioner styles carry the engine's transform positioning.
    #[wasm_bindgen_test]
    async fn positions_the_floating_element_and_reports_the_rendered_placement() {
        let owner = init();
        let mounted = RwSignal::new_local(true);
        let (store, _anchor, _floating, result) = setup(mounted, |_| {});

        assert!(
            result.is_positioned.get_untracked(),
            "positioned after the update"
        );
        assert_eq!(result.physical_side.get_untracked(), PhysicalSide::Bottom);
        assert_eq!(
            result.side.get_untracked(),
            Side::Bottom,
            "logical side is bottom in LTR"
        );
        assert_eq!(result.align.get_untracked(), Align::Center);
        assert!(
            !result.anchor_hidden.get_untracked(),
            "an in-view anchor is not hidden"
        );

        // The store's element mirrors re-apply on the microtask after a notify (the
        // ReactStore port's effect wiring) — the derived styles read the mirror.
        run_microtasks(2).await;
        let styles = result.positioner_styles.get_untracked();
        assert_eq!(
            styles.position,
            Strategy::Absolute,
            "the configured position method wins"
        );
        assert!(
            styles
                .transform
                .as_deref()
                .unwrap_or_default()
                .contains("translate("),
            "transform positioning: {:?}",
            styles.transform
        );
        assert_eq!(styles.opacity, None, "no opacity once positioned");
        assert_eq!(
            styles.available_width, "100vw",
            "the seed stays present while positioned"
        );
        assert_eq!(
            styles.available_height, "100vh",
            "the seed stays present while positioned"
        );

        finish(owner);
    }

    // `useAnchorPositioning.ts:504-510,529-531` — until positioned, the stale
    // coordinates are ignored: fixed at 0/0 and transparent, with the `--available-*`
    // seeds present so consumer `min()` rules resolve on the first pass.
    #[wasm_bindgen_test]
    async fn pre_positioning_styles_collapse_until_positioned() {
        let owner = init();
        let mounted = RwSignal::new_local(true);
        let store = empty_store();
        let _anchor =
            mount_element("position: fixed; top: 100px; left: 100px; width: 60px; height: 40px;");
        let floating =
            mount_element("position: fixed; top: 0; left: 0; width: 100px; height: 50px;");
        let result = hook(&store, mounted, |_| {});
        store.set_field(|state| &mut state.floating_element, Some(floating.clone()));

        let styles = result.positioner_styles.get_untracked();
        assert_eq!(styles.position, Strategy::Fixed, "fixed until positioned");
        assert_eq!(styles.top.as_deref(), Some("0px"));
        assert_eq!(styles.left.as_deref(), Some("0px"));
        assert_eq!(styles.opacity.as_deref(), Some("0"));
        assert_eq!(styles.available_width, "100vw");
        assert_eq!(styles.available_height, "100vh");

        finish(owner);
    }

    // `useAnchorPositioning.ts:256-278` — the side offset moves the popup off the
    // anchor by the configured distance; the function form reads the live rects.
    #[wasm_bindgen_test]
    async fn applies_the_side_offset_in_number_and_function_forms() {
        let owner = init();

        // Number form: bottom edge (100 + 40) + 10 = 150.
        let mounted = RwSignal::new_local(true);
        let (_, _anchor, _floating, result) = setup(mounted, |params| {
            params.side_offset = SideOffset::Number(10.0);
        });
        let y = result.context.y.get_untracked();
        assert!(
            (y - 150.0).abs() < 1.0,
            "the popup's y sits at the anchor bottom + side offset, got {y}"
        );

        // Function form: the callback receives the live anchor dimensions.
        let mounted = RwSignal::new_local(true);
        let (_, _anchor, _floating, result) = setup(mounted, |params| {
            params.side_offset = SideOffset::Function(Rc::new(|data| data.anchor.height));
        });
        let y = result.context.y.get_untracked();
        assert!(
            (y - 180.0).abs() < 1.0,
            "the fn offset reads anchor.height (40), got {y}"
        );

        finish(owner);
    }

    // `useAnchorPositioning.ts:182-196,580-582` — logical sides map through the RTL
    // swap: inline-end renders left in RTL and the logical readback follows the
    // rendered physical side.
    #[wasm_bindgen_test]
    async fn maps_logical_sides_through_rtl() {
        let owner = init();
        {
            let direction = RwSignal::new(Some(TextDirection::Rtl));
            crate::direction_provider::provide_direction_context(direction);
            let mounted = RwSignal::new_local(true);
            let (store, _anchor, _floating, result) = setup(mounted, |params| {
                params.side = Side::InlineEnd;
            });
            assert_eq!(result.physical_side.get_untracked(), PhysicalSide::Left);
            assert_eq!(result.side.get_untracked(), Side::InlineEnd);
            let _ = store;
        }
        finish(owner);

        // LTR (no provider): a fresh owner root so the disposed RTL context above is
        // not on this scope's context walk.
        let owner = init();
        {
            let mounted = RwSignal::new_local(true);
            let (_, _anchor, _floating, result) = setup(mounted, |params| {
                params.side = Side::InlineEnd;
            });
            assert_eq!(result.physical_side.get_untracked(), PhysicalSide::Right);
            assert_eq!(result.side.get_untracked(), Side::InlineEnd);
        }
        finish(owner);
    }

    // `useAnchorPositioning.ts:284-300` — flip moves the popup to the opposite side
    // when the preferred side overflows; `side: 'none'` keeps it.
    #[wasm_bindgen_test]
    async fn flip_avoids_collision_and_none_keeps_the_side() {
        let owner = init();

        // Default avoidance (flip): no room below the bottom-edge anchor → top.
        let mounted = RwSignal::new_local(true);
        let store = empty_store();
        let anchor =
            mount_element("position: fixed; top: 560px; left: 100px; width: 60px; height: 40px;");
        let floating =
            mount_element("position: fixed; top: 0; left: 0; width: 100px; height: 120px;");
        let result = hook(&store, mounted, |_| {});
        store.set_field(
            |state| &mut state.reference_element,
            Some(ReferenceType::Element(anchor.clone())),
        );
        store.set_field(|state| &mut state.floating_element, Some(floating.clone()));
        store.update(|state, _| {
            state.open = true;
            true
        });
        (result.update)();
        assert_eq!(
            result.physical_side.get_untracked(),
            PhysicalSide::Top,
            "the popup flips to the top side"
        );

        // `side: 'none'`: the same geometry keeps the preferred side.
        let mounted = RwSignal::new_local(true);
        let store = empty_store();
        let anchor =
            mount_element("position: fixed; top: 560px; left: 100px; width: 60px; height: 40px;");
        let floating =
            mount_element("position: fixed; top: 0; left: 0; width: 100px; height: 120px;");
        let result = hook(&store, mounted, |params| {
            params.collision_avoidance = CollisionAvoidance {
                side: Some(CollisionAvoidanceSide::None),
                ..CollisionAvoidance::default()
            };
        });
        store.set_field(
            |state| &mut state.reference_element,
            Some(ReferenceType::Element(anchor.clone())),
        );
        store.set_field(|state| &mut state.floating_element, Some(floating.clone()));
        store.update(|state, _| {
            state.open = true;
            true
        });
        (result.update)();
        assert_eq!(
            result.physical_side.get_untracked(),
            PhysicalSide::Bottom,
            "side: 'none' keeps the preferred side even though it overflows"
        );

        finish(owner);
    }

    // `useAnchorPositioning.ts:350-371` — the size middleware writes the
    // `--available-*` and DPR-snapped `--anchor-*` vars onto the floating element.
    #[wasm_bindgen_test]
    async fn size_writes_the_available_and_anchor_vars_on_the_element() {
        let owner = init();
        let mounted = RwSignal::new_local(true);
        let (store, _anchor, floating, result) = setup(mounted, |_| {});
        let _ = store;

        let style = element_style(&floating);
        let available_height = style
            .get_property_value(common_positioner_css_vars::AVAILABLE_HEIGHT)
            .unwrap();
        assert!(
            available_height.ends_with("px") && available_height != "0px",
            "the real available height is written imperatively, got {available_height:?}"
        );
        let anchor_width = style
            .get_property_value(common_positioner_css_vars::ANCHOR_WIDTH)
            .unwrap();
        assert_eq!(
            anchor_width, "60px",
            "the anchor width var is the anchor's width"
        );
        assert!(
            result.positioner_styles.get_untracked().available_width == "100vw",
            "the inline seed is unrelated to size()'s imperative values"
        );

        finish(owner);
    }

    // `useAnchorPositioning.ts:372-382,383-446` — the transform-origin var is written
    // on every pass, with or without a real arrow; with a real arrow the arrow
    // middleware data feeds arrowStyles.
    #[wasm_bindgen_test]
    async fn transform_origin_is_written_and_arrow_data_populates() {
        let owner = init();
        let mounted = RwSignal::new_local(true);
        let (store, _anchor, floating, result) = setup(mounted, |_| {});
        let _ = store;

        let origin = element_style(&floating)
            .get_property_value(common_positioner_css_vars::TRANSFORM_ORIGIN)
            .unwrap();
        assert!(
            !origin.is_empty(),
            "the fake-element path still anchors the transform origin"
        );
        assert!(
            result.arrow_styles.get_untracked().top.is_none(),
            "no arrow data before the arrow element is registered"
        );

        // Register a real arrow: the middleware data now flows into arrowStyles and
        // the origin derives from the arrow geometry.
        let arrow = document().create_element("div").unwrap();
        arrow.set_attribute("style", "position: absolute; width: 10px; height: 10px;");
        floating.append_child(&arrow).unwrap();
        result.arrow_ref.borrow_mut().replace(arrow);
        (result.update)();

        let styles = result.arrow_styles.get_untracked();
        assert!(
            styles.left.is_some() && styles.top.is_none(),
            "a bottom-placed popup's arrow offset rides the x axis, got {styles:?}"
        );
        assert!(
            !element_style(&floating)
                .get_property_value(common_positioner_css_vars::TRANSFORM_ORIGIN)
                .unwrap()
                .is_empty()
        );

        finish(owner);
    }

    // `useAnchorPositioning.ts:583` + the custom `hide` middleware — an offscreen
    // anchor reports `anchorHidden`.
    #[wasm_bindgen_test]
    async fn hide_reports_an_offscreen_anchor_as_hidden() {
        let owner = init();
        let mounted = RwSignal::new_local(true);
        let store = empty_store();
        let anchor =
            mount_element("position: fixed; top: -9999px; left: 100px; width: 60px; height: 40px;");
        let floating =
            mount_element("position: fixed; top: 0; left: 0; width: 100px; height: 50px;");
        let result = hook(&store, mounted, |_| {});
        store.set_field(
            |state| &mut state.reference_element,
            Some(ReferenceType::Element(anchor.clone())),
        );
        store.set_field(|state| &mut state.floating_element, Some(floating.clone()));
        store.update(|state, _| {
            state.open = true;
            true
        });
        (result.update)();
        assert!(
            result.anchor_hidden.get_untracked(),
            "the anchor scrolled out of the clipping ancestors is hidden"
        );

        finish(owner);
    }

    // `useAnchorPositioning.ts:537-552` — the anchor registers through
    // `refs.setPositionReference`, wrapping real elements into the virtual-rect shim.
    #[wasm_bindgen_test]
    async fn registers_the_anchor_through_the_position_reference_slot() {
        let owner = init();
        let mounted = RwSignal::new_local(true);
        let store = empty_store();
        let anchor =
            mount_element("position: fixed; top: 100px; left: 100px; width: 60px; height: 40px;");
        let _floating =
            mount_element("position: fixed; top: 0; left: 0; width: 100px; height: 50px;");
        let result = hook(&store, mounted, |params| {
            params.anchor = Anchor::Static(ReferenceType::Element(anchor.clone()));
        });

        let position_reference = store.get_snapshot().position_reference.clone();
        assert!(
            matches!(position_reference, Some(ReferenceType::Virtual(_))),
            "the registered anchor is stored as the virtual rect shim"
        );
        let _ = result.context.refs.reference.borrow().clone();

        finish(owner);
    }

    // `useAnchorPositioning.ts:451-462` — keepMounted + closed: the root context's
    // element fields are nulled so positioning cannot run on the hidden popup; on the
    // next open the anchor re-registers.
    #[wasm_bindgen_test]
    async fn keep_mounted_nulls_the_context_elements_while_closed() {
        let owner = init();
        let mounted = RwSignal::new_local(false);
        let store = empty_store();
        let anchor =
            mount_element("position: fixed; top: 100px; left: 100px; width: 60px; height: 40px;");
        let floating =
            mount_element("position: fixed; top: 0; left: 0; width: 100px; height: 50px;");
        store.set_field(
            |state| &mut state.reference_element,
            Some(ReferenceType::Element(anchor.clone())),
        );
        store.set_field(|state| &mut state.floating_element, Some(floating.clone()));
        let _result = hook(&store, mounted, |params| {
            params.keep_mounted = true;
            params.anchor = Anchor::Static(ReferenceType::Element(anchor.clone()));
        });

        let snapshot = store.get_snapshot();
        assert!(
            snapshot.reference_element.is_none(),
            "the reference clears while closed"
        );
        assert!(
            snapshot.floating_element.is_none(),
            "the floating element clears while closed"
        );
        assert!(
            snapshot.position_reference.is_none(),
            "no anchor registration while closed"
        );

        // Reopening re-registers the anchor (the layout effect tracks `mounted`).
        mounted.set(true);
        run_microtasks(3).await;
        assert!(
            store.get_snapshot().position_reference.is_some(),
            "the anchor registers on the next open"
        );

        finish(owner);
    }

    // The ending-phase compensation (module docs): upstream keeps `isPositioned` true
    // through the ending transition because the engine receives `open: mounted` /
    // `undefined`; the port's engine gates on the store's open, so the positioner
    // styles hold the real coordinates through `!store_open && mounted` once the
    // popup has positioned (the latch).
    #[wasm_bindgen_test]
    async fn positioner_styles_stay_real_while_ending() {
        let owner = init();
        let mounted = RwSignal::new_local(true);
        let (store, _anchor, _floating, result) = setup(mounted, |_| {});
        assert!(
            result.is_positioned.get_untracked(),
            "positioned while open"
        );

        // The latch effect must observe the positioned state before the close.
        run_microtasks(2).await;
        // The close flips the store's open while the popup stays mounted (the ending
        // transition): the engine's raw signal resets, the positioner styles must not
        // collapse.
        store.update(|state, _| {
            state.open = false;
            true
        });
        (result.update)();
        assert!(
            !result.is_positioned.get_untracked(),
            "the engine's raw isPositioned follows the store's open"
        );
        // The store's open mirror re-applies on the microtask after a notify — the
        // reactive derivation observes the close after the same settle real consumers
        // get.
        run_microtasks(2).await;
        let styles = result.positioner_styles.get_untracked();
        assert_eq!(
            styles.position,
            Strategy::Absolute,
            "still the configured position"
        );
        assert!(
            styles.transform.is_some(),
            "still transform-positioned during the ending phase"
        );
        assert_eq!(styles.opacity, None, "no opacity during the ending phase");

        // Once fully unmounted (mounted flips false), the collapsed pre-positioning
        // styles apply — the same styles a later open starts from.
        mounted.set(false);
        let styles = result.positioner_styles.get_untracked();
        assert_eq!(styles.position, Strategy::Fixed, "fixed once unmounted");
        assert_eq!(styles.opacity.as_deref(), Some("0"));

        finish(owner);
    }

    // `useAnchorPositioning.ts:585-592` — lazyFlip locks the flipped side: after the
    // flip, the lock value feeds the side derivation until the popup closes.
    #[wasm_bindgen_test]
    async fn lazy_flip_locks_the_rendered_side_until_close() {
        let owner = init();

        // A bottom-edge anchor with default params flips to top; with lazyFlip the
        // locked side is observed through the logical side readback after the flip
        // settles.
        let mounted = RwSignal::new_local(true);
        let store = empty_store();
        let anchor =
            mount_element("position: fixed; top: 560px; left: 100px; width: 60px; height: 40px;");
        let floating =
            mount_element("position: fixed; top: 0; left: 0; width: 100px; height: 120px;");
        let result = hook(&store, mounted, |params| {
            params.lazy_flip = true;
        });
        store.set_field(
            |state| &mut state.reference_element,
            Some(ReferenceType::Element(anchor.clone())),
        );
        store.set_field(|state| &mut state.floating_element, Some(floating.clone()));
        store.update(|state, _| {
            state.open = true;
            true
        });
        (result.update)();
        assert_eq!(result.physical_side.get_untracked(), PhysicalSide::Top);
        run_microtasks(4).await;
        assert_eq!(
            result.side.get_untracked(),
            Side::Top,
            "the sticky mount side equals the flipped side"
        );

        // Closing clears the lock (the render-phase reset analog), so the next open
        // re-derives the side from the param.
        mounted.set(false);
        run_microtasks(4).await;

        finish(owner);
    }

    // A virtual-element anchor positions from the shim's rect, not a DOM element —
    // the useClientPoint-style cursor anchoring.
    #[wasm_bindgen_test]
    async fn virtual_anchors_position_from_the_shim_rect() {
        let owner = init();
        let mounted = RwSignal::new_local(true);
        let store = empty_store();
        let _floating =
            mount_element("position: fixed; top: 0; left: 0; width: 100px; height: 50px;");
        #[derive(Clone, Debug, PartialEq)]
        struct FixedRect;
        impl VirtualElement<Element> for FixedRect {
            fn get_bounding_client_rect(&self) -> floating_ui_dom::ClientRectObject {
                floating_ui_dom::ClientRectObject {
                    x: 300.0,
                    y: 200.0,
                    width: 10.0,
                    height: 10.0,
                    top: 200.0,
                    right: 310.0,
                    bottom: 210.0,
                    left: 300.0,
                }
            }
            fn get_client_rects(&self) -> Option<Vec<floating_ui_dom::ClientRectObject>> {
                None
            }
            fn context_element(&self) -> Option<Element> {
                None
            }
        }
        let result = hook(&store, mounted, |params| {
            params.anchor =
                Anchor::Static(ReferenceType::Virtual(VirtualReference::new(FixedRect)));
        });
        store.set_field(
            |state| &mut state.floating_element,
            Some(mount_element(
                "position: fixed; top: 0; left: 0; width: 100px; height: 50px;",
            )),
        );
        store.update(|state, _| {
            state.open = true;
            true
        });
        (result.update)();
        let y = result.context.y.get_untracked();
        assert!(
            (y - 210.0).abs() < 1.0,
            "the popup sits under the virtual rect's bottom edge, got {y}"
        );

        finish(owner);
    }
}
