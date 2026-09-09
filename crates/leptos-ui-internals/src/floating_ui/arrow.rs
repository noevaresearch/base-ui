//! Port of `packages/react/src/floating-ui-react/middleware/arrow.ts` — the unit's one
//! vendored fork over the external `@floating-ui/react-dom`/`@floating-ui/utils`
//! delegation (`specs/library/floating-ui-react/implementation.md`, "External
//! delegation": "`middleware/arrow.ts` is the only vendored fork (adds
//! `offsetParent: 'real' | 'floating'`, `arrow.ts:30-34`)"). Everything else the unit
//! needs from `@floating-ui/*` binds to the `floating-ui-dom` crate directly (see the
//! `floating_ui` module docs); this file is forked instead because it changes `arrow`'s
//! behavior, not just its call surface.
//!
//! - `ArrowOptions` (`arrow.ts:11-28`) → [`ArrowOptions`]
//! - `baseArrow` (`arrow.ts:34-109`) → [`BaseArrow`], [`base_arrow`]
//! - `arrow` (`arrow.ts:117-127`) → [`arrow`] — upstream's wrapper only reattaches a
//!   `deps: React.DependencyList` for `@floating-ui/react`'s `useMemo`-based option
//!   memoization (`arrow.ts:119,125`); `reactive_graph`'s fine-grained reactivity has no
//!   equivalent manual dependency list, so `arrow` here is [`base_arrow`] under the name
//!   real callers use (`packages/react/src/floating-ui-react/index.ts:39`,
//!   `packages/react/src/internals/useAnchorPositioning.ts:372-382`).
//!
//! **Spec-discrepancy note** (logged to `ralph/logs/spec-discrepancies.md`): tracing
//! `offsetParent`'s only use (`arrow.ts:60-67`) line by line shows `clientSize` resolves
//! to the identical expression (`elements.floating[clientProp] || rects.floating[length]`)
//! on both branches — `arrowOffsetParent` (including the `'real'`-only
//! `platform.getOffsetParent` call) is computed but never feeds `clientSize`'s *value*,
//! only an `isElement` check whose result is likewise discarded by the redundant
//! reassignment on line 66. So `offsetParent: 'floating'` — which
//! `useAnchorPositioning.ts:379` sets explicitly, expecting different behavior from the
//! `'real'` default — currently has no effect on output; both modes compute identically.
//! Ported byte-for-byte below rather than "fixed", per this loop's faithful-port mandate;
//! [`OffsetParent`] and the `'real'`-only platform call are kept so a future upstream fix
//! has a matching seam to land in.

use floating_ui_core::Reset;
use floating_ui_dom::{
    Axis, Coords, Derivable, DerivableFn, Middleware, MiddlewareReturn, MiddlewareState,
    MiddlewareWithOptions, Padding, Side,
};
use floating_ui_utils::{
    clamp, get_alignment, get_alignment_axis, get_axis_length, get_padding_object,
};

pub use floating_ui_dom::{ARROW_NAME, ArrowData};

/// Which element's client size the arrow centers within (`arrow.ts:23-27`).
///
/// See the module's spec-discrepancy note: as upstream currently reads, this option has no
/// effect on the middleware's output — kept for API fidelity with `ArrowOptions` and so a
/// future upstream fix has a matching seam to land in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OffsetParent {
    /// `platform.getOffsetParent(element)` — upstream's default (`arrow.ts:27`).
    #[default]
    Real,
    /// The floating element itself (`arrow.ts:61`).
    Floating,
}

/// Options for [`BaseArrow`]/[`arrow`] (`arrow.ts:11-28`).
#[derive(Clone, Debug, PartialEq)]
pub struct ArrowOptions<Element: Clone> {
    /// The arrow element to be positioned. `None` short-circuits [`BaseArrow::compute`] to
    /// an empty [`MiddlewareReturn`] (`arrow.ts:16,42-44`) — upstream types this as
    /// required but still null-checks it at runtime, so the port keeps both: a
    /// required-`element` constructor below, plus the checked `Option` path `compute`
    /// actually takes.
    pub element: Option<Element>,
    /// Padding between the arrow and the floating element's edges (`arrow.ts:18-22`).
    /// Defaults to `0` on every side.
    pub padding: Option<Padding>,
    /// Which element to use as the offset parent (`arrow.ts:23-27`). Defaults to
    /// [`OffsetParent::Real`].
    pub offset_parent: OffsetParent,
}

impl<Element: Clone> ArrowOptions<Element> {
    /// Constructs options for a present `element`, matching upstream's `{element}` shape
    /// with every other field at its documented default.
    pub fn new(element: Element) -> Self {
        ArrowOptions {
            element: Some(element),
            padding: None,
            offset_parent: OffsetParent::default(),
        }
    }

    /// Set `element` option.
    pub fn element(mut self, value: Element) -> Self {
        self.element = Some(value);
        self
    }

    /// Set `padding` option.
    pub fn padding(mut self, value: Padding) -> Self {
        self.padding = Some(value);
        self
    }

    /// Set `offset_parent` option (`arrow.ts:23-27`, the fork's addition over vanilla
    /// `arrow`).
    pub fn offset_parent(mut self, value: OffsetParent) -> Self {
        self.offset_parent = value;
        self
    }
}

/// Fork of the core `arrow` middleware allowing the offset parent to be configured
/// (`arrow.ts:30-109`, upstream's `baseArrow`).
#[derive(PartialEq)]
pub struct BaseArrow<'a, Element: Clone + 'static, Window: Clone> {
    options: Derivable<'a, Element, Window, ArrowOptions<Element>>,
}

impl<'a, Element: Clone + 'static, Window: Clone> BaseArrow<'a, Element, Window> {
    /// Constructs a new instance of this middleware.
    pub fn new(options: ArrowOptions<Element>) -> Self {
        BaseArrow {
            options: options.into(),
        }
    }

    /// Constructs a new instance of this middleware with derivable options
    /// (`arrow.ts:34`'s `Derivable<ArrowOptions>` branch — the shape
    /// `useAnchorPositioning.ts:372-382` actually uses).
    pub fn new_derivable(options: Derivable<'a, Element, Window, ArrowOptions<Element>>) -> Self {
        BaseArrow { options }
    }

    /// Constructs a new instance of this middleware with a derivable-options function.
    pub fn new_derivable_fn(
        options: DerivableFn<'a, Element, Window, ArrowOptions<Element>>,
    ) -> Self {
        BaseArrow {
            options: options.into(),
        }
    }
}

impl<Element: Clone + 'static, Window: Clone> Clone for BaseArrow<'_, Element, Window> {
    fn clone(&self) -> Self {
        Self {
            options: self.options.clone(),
        }
    }
}

impl<Element: Clone + PartialEq, Window: Clone + PartialEq> Middleware<Element, Window>
    for BaseArrow<'static, Element, Window>
{
    fn name(&self) -> &'static str {
        ARROW_NAME
    }

    fn compute(&self, state: MiddlewareState<Element, Window>) -> MiddlewareReturn {
        let options = self.options.evaluate(state.clone());

        // `arrow.ts:42-44`: `element == null` short-circuits to an empty return before
        // anything else runs.
        let Some(element) = options.element else {
            return MiddlewareReturn {
                x: None,
                y: None,
                data: None,
                reset: None,
            };
        };

        let MiddlewareState {
            x,
            y,
            placement,
            middleware_data,
            elements,
            rects,
            platform,
            ..
        } = state;

        let data: Option<ArrowData> = middleware_data.get_as(self.name());

        let padding_object = get_padding_object(options.padding.unwrap_or(Padding::All(0.0)));
        let coords = Coords { x, y };
        let axis = get_alignment_axis(placement);
        let length = get_axis_length(axis);
        let arrow_dimensions = platform.get_dimensions(&element);
        let min_prop = match axis {
            Axis::X => Side::Left,
            Axis::Y => Side::Top,
        };
        let max_prop = match axis {
            Axis::X => Side::Right,
            Axis::Y => Side::Bottom,
        };

        let start_diff = coords.axis(axis) - rects.reference.axis(axis);
        let end_diff = rects.reference.length(length) + rects.reference.axis(axis)
            - coords.axis(axis)
            - rects.floating.length(length);

        // `arrow.ts:60-67` (see the module's spec-discrepancy note): upstream computes
        // `arrowOffsetParent` — calling `platform.getOffsetParent` only in `'real'` mode —
        // purely to feed an `isElement` check whose result never reaches `clientSize`
        // either way (lines 62 and 66 are the same expression). The `'real'`-only platform
        // call is kept here for call-site parity with upstream; its result is
        // intentionally unused.
        if matches!(options.offset_parent, OffsetParent::Real) {
            let _ = platform.get_offset_parent(&element);
        }
        let client_size = platform
            .get_client_length(elements.floating, length)
            .filter(|size| *size != 0.0)
            .unwrap_or(rects.floating.length(length));

        let center_to_reference = end_diff / 2.0 - start_diff / 2.0;

        // If the padding is large enough that it causes the arrow to no longer be
        // centered, modify the padding so that it is centered (`arrow.ts:71-75`).
        let largest_possible_padding =
            client_size / 2.0 - arrow_dimensions.length(length) / 2.0 - 1.0;
        let min_padding = padding_object.side(min_prop).min(largest_possible_padding);
        let max_padding = padding_object.side(max_prop).min(largest_possible_padding);

        // Make sure the arrow doesn't overflow the floating element if the center point is
        // outside the floating element's bounds (`arrow.ts:77-82`).
        let min = min_padding;
        let max = client_size - arrow_dimensions.length(length) - max_padding;
        let center =
            client_size / 2.0 - arrow_dimensions.length(length) / 2.0 + center_to_reference;
        let offset = clamp(min, center, max);

        // If the reference is small enough that the arrow's padding causes it to point to
        // nothing for an aligned placement, adjust the offset of the floating element
        // itself. A single reset is performed when this is true so `shift()` keeps taking
        // action (`arrow.ts:84-97`).
        let should_add_offset = data.is_none()
            && get_alignment(placement).is_some()
            && center != offset
            && rects.reference.length(length) / 2.0
                - (if center < min {
                    min_padding
                } else {
                    max_padding
                })
                - arrow_dimensions.length(length) / 2.0
                < 0.0;
        let alignment_offset = if should_add_offset {
            if center < min {
                center - min
            } else {
                center - max
            }
        } else {
            0.0
        };

        MiddlewareReturn {
            x: match axis {
                Axis::X => Some(coords.axis(axis) + alignment_offset),
                Axis::Y => None,
            },
            y: match axis {
                Axis::X => None,
                Axis::Y => Some(coords.axis(axis) + alignment_offset),
            },
            data: Some(
                serde_json::to_value(ArrowData {
                    x: match axis {
                        Axis::X => Some(offset),
                        Axis::Y => None,
                    },
                    y: match axis {
                        Axis::X => None,
                        Axis::Y => Some(offset),
                    },
                    center_offset: center - offset - alignment_offset,
                    alignment_offset: should_add_offset.then_some(alignment_offset),
                })
                .expect("Data should be valid JSON."),
            ),
            reset: should_add_offset.then_some(Reset::True),
        }
    }
}

impl<Element: Clone, Window: Clone> MiddlewareWithOptions<Element, Window, ArrowOptions<Element>>
    for BaseArrow<'_, Element, Window>
{
    fn options(&self) -> &Derivable<'_, Element, Window, ArrowOptions<Element>> {
        &self.options
    }
}

/// `baseArrow` (`arrow.ts:34-109`) — constructs the middleware from a fixed
/// [`ArrowOptions`] value. Use [`BaseArrow::new_derivable`] / [`BaseArrow::new_derivable_fn`]
/// directly for the `Derivable<ArrowOptions>` branch upstream's parameter type also allows
/// (`arrow.ts:34`).
pub fn base_arrow<Element: Clone + 'static, Window: Clone>(
    options: ArrowOptions<Element>,
) -> BaseArrow<'static, Element, Window> {
    BaseArrow::new(options)
}

/// `arrow` (`arrow.ts:117-127`) — the name real callers use
/// (`packages/react/src/floating-ui-react/index.ts:39`,
/// `packages/react/src/internals/useAnchorPositioning.ts:372`). See the module docs for
/// why this is [`base_arrow`] under upstream's public name rather than a distinct wrapper.
pub fn arrow<Element: Clone + 'static, Window: Clone>(
    options: ArrowOptions<Element>,
) -> BaseArrow<'static, Element, Window> {
    base_arrow(options)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use floating_ui_core::{Elements, GetClippingRectArgs, GetElementRectsArgs, Platform};
    use floating_ui_dom::{
        Dimensions, ElementRects, Length, MiddlewareData, Placement, Rect, Strategy,
    };
    // `floating_ui_dom::ElementOrVirtual` is that crate's own DOM-specialized alias
    // (narrowed to `web_sys::Element`, shadowing the generic re-export from
    // `floating_ui_utils`); these fixtures need the actual generic type to plug in
    // `FakeElement`.
    use floating_ui_utils::ElementOrVirtual;

    #[derive(Clone, Debug, PartialEq)]
    struct FakeElement;
    #[derive(Clone, Debug, PartialEq)]
    struct FakeWindow;

    /// A fixture platform exercising only what [`BaseArrow::compute`] actually calls —
    /// `get_dimensions` (required) and `get_client_length` (configurable per test).
    /// `get_element_rects`/`get_clipping_rect` are required by the trait but never invoked
    /// by `compute` directly (it reads `rects`/`elements` off the `MiddlewareState` the
    /// test constructs, the way `compute_position` would have supplied them).
    #[derive(Debug)]
    struct FixturePlatform {
        arrow_dimensions: Dimensions,
        floating_client_length: Option<f64>,
    }

    impl Platform<FakeElement, FakeWindow> for FixturePlatform {
        fn get_element_rects(&self, _args: GetElementRectsArgs<FakeElement>) -> ElementRects {
            unreachable!("not exercised by BaseArrow::compute")
        }

        fn get_clipping_rect(&self, _args: GetClippingRectArgs<FakeElement>) -> Rect {
            unreachable!("not exercised by BaseArrow::compute")
        }

        fn get_dimensions(&self, _element: &FakeElement) -> Dimensions {
            self.arrow_dimensions.clone()
        }

        fn get_client_length(&self, _element: &FakeElement, _length: Length) -> Option<f64> {
            self.floating_client_length
        }
    }

    /// Builds a `MiddlewareState` for a floating element horizontally centered under a
    /// reference of the given width, both `100x100`-ish rects, for a `Placement::Bottom`
    /// compute pass (bottom placement puts the alignment axis on `x`, matching upstream's
    /// own `arrow` fixtures).
    fn state<'a>(
        middleware_data: &'a MiddlewareData,
        reference: &'a FakeElement,
        floating: &'a FakeElement,
        rects: &'a ElementRects,
        placement: Placement,
        platform: &'a FixturePlatform,
    ) -> MiddlewareState<'a, FakeElement, FakeWindow> {
        MiddlewareState {
            x: rects.floating.x,
            y: rects.floating.y,
            initial_placement: placement,
            placement,
            strategy: Strategy::Absolute,
            middleware_data,
            elements: Elements {
                reference: ElementOrVirtual::Element(reference),
                floating,
            },
            rects,
            platform,
        }
    }

    // Pins the basic centering math shared with vanilla `arrow` (`arrow.ts:46-83`): a
    // 20-wide arrow inside a 50-wide floating element, reference and floating both
    // starting at the same x, centers with no reset.
    #[test]
    fn centers_the_arrow_within_the_floating_element_with_no_reset() {
        let reference = FakeElement;
        let floating = FakeElement;
        let rects = ElementRects {
            reference: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 30.0,
            },
            floating: Rect {
                x: 0.0,
                y: 30.0,
                width: 50.0,
                height: 20.0,
            },
        };
        let platform = FixturePlatform {
            arrow_dimensions: Dimensions {
                width: 20.0,
                height: 10.0,
            },
            floating_client_length: Some(50.0),
        };
        let middleware_data = MiddlewareData::default();
        let s = state(
            &middleware_data,
            &reference,
            &floating,
            &rects,
            Placement::Bottom,
            &platform,
        );

        let middleware = base_arrow::<FakeElement, FakeWindow>(ArrowOptions::new(FakeElement));
        let result = middleware.compute(s);

        // Bare `Bottom` has no alignment (`getAlignment` is `None`), so `shouldAddOffset`
        // never applies and the floating element's own position is untouched: `result.x`
        // stays at the input `coords.x` and `result.y` is `None` (the alignment axis is
        // `x` for a top/bottom placement).
        assert_eq!(result.x, Some(0.0));
        assert_eq!(result.y, None);
        assert_eq!(result.reset, None);

        // The arrow's own centering position is carried in the returned `data`, not
        // `result.x`: reference (width 100) and floating (width 50) share the same left
        // edge, giving `centerToReference` = 25; the arrow (width 20) would then center at
        // `50/2 - 20/2 + 25 = 40`, clamped into `[0, client_size - arrow_width] = [0, 30]`.
        let arrow_data: ArrowData = result
            .data
            .as_ref()
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .expect("a successful compute records arrow data");
        assert_eq!(arrow_data.x, Some(30.0));
        assert_eq!(arrow_data.alignment_offset, None);
    }

    // Pins the fork's one documented delta (`arrow.ts:23-27,60-67`): per the module's
    // spec-discrepancy note, `offsetParent` does not currently change `compute`'s output,
    // so `'real'` and `'floating'` must resolve identically for the same fixture.
    #[test]
    fn offset_parent_real_and_floating_resolve_identically() {
        let reference = FakeElement;
        let floating = FakeElement;
        let rects = ElementRects {
            reference: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 30.0,
            },
            floating: Rect {
                x: 0.0,
                y: 30.0,
                width: 50.0,
                height: 20.0,
            },
        };
        let platform = FixturePlatform {
            arrow_dimensions: Dimensions {
                width: 20.0,
                height: 10.0,
            },
            floating_client_length: Some(50.0),
        };

        let real_data = MiddlewareData::default();
        let real_result = base_arrow::<FakeElement, FakeWindow>(
            ArrowOptions::new(FakeElement).offset_parent(OffsetParent::Real),
        )
        .compute(state(
            &real_data,
            &reference,
            &floating,
            &rects,
            Placement::Bottom,
            &platform,
        ));

        let floating_data = MiddlewareData::default();
        let floating_result = base_arrow::<FakeElement, FakeWindow>(
            ArrowOptions::new(FakeElement).offset_parent(OffsetParent::Floating),
        )
        .compute(state(
            &floating_data,
            &reference,
            &floating,
            &rects,
            Placement::Bottom,
            &platform,
        ));

        assert_eq!(real_result, floating_result);
    }

    // Pins `arrow.ts:42-44`: a `None` element short-circuits to an empty return without
    // touching the platform (a panic from `FixturePlatform::get_dimensions`'s use in a
    // non-fixture arrangement would fail this test).
    #[test]
    fn a_missing_element_returns_an_empty_middleware_return() {
        let reference = FakeElement;
        let floating = FakeElement;
        let rects = ElementRects {
            reference: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 30.0,
            },
            floating: Rect {
                x: 0.0,
                y: 30.0,
                width: 50.0,
                height: 20.0,
            },
        };
        let platform = FixturePlatform {
            arrow_dimensions: Dimensions {
                width: 20.0,
                height: 10.0,
            },
            floating_client_length: Some(50.0),
        };
        let middleware_data = MiddlewareData::default();
        let s = state(
            &middleware_data,
            &reference,
            &floating,
            &rects,
            Placement::Bottom,
            &platform,
        );

        let middleware = BaseArrow::<FakeElement, FakeWindow>::new(ArrowOptions {
            element: None,
            padding: None,
            offset_parent: OffsetParent::default(),
        });
        let result = middleware.compute(s);

        assert_eq!(
            result,
            MiddlewareReturn {
                x: None,
                y: None,
                data: None,
                reset: None,
            }
        );
    }

    // Pins the `shouldAddOffset`/`reset` interaction (`arrow.ts:87-107`, the
    // implementation spec's other named-untested branch): a reference much narrower than
    // the arrow's padding for an aligned placement forces a reset with a non-zero
    // `alignmentOffset`, and only fires once (`data.is_none()` / `!middlewareData.arrow`,
    // `arrow.ts:88-89`) — a second pass with prior `arrow` data present does not reset
    // again even though the geometry is unchanged.
    #[test]
    fn a_too_small_aligned_reference_resets_once_with_an_alignment_offset() {
        let reference = FakeElement;
        let floating = FakeElement;
        let rects = ElementRects {
            reference: Rect {
                x: 0.0,
                y: 0.0,
                width: 4.0,
                height: 30.0,
            },
            floating: Rect {
                x: 0.0,
                y: 30.0,
                width: 50.0,
                height: 20.0,
            },
        };
        let platform = FixturePlatform {
            arrow_dimensions: Dimensions {
                width: 20.0,
                height: 10.0,
            },
            floating_client_length: Some(50.0),
        };
        let first_pass_data = MiddlewareData::default();
        let first_pass = base_arrow::<FakeElement, FakeWindow>(
            ArrowOptions::new(FakeElement).padding(Padding::All(10.0)),
        )
        .compute(state(
            &first_pass_data,
            &reference,
            &floating,
            &rects,
            Placement::BottomStart,
            &platform,
        ));

        assert_eq!(first_pass.reset, Some(Reset::True));
        let first_pass_data: ArrowData = first_pass
            .data
            .as_ref()
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .expect("the reset pass records arrow data");
        assert_ne!(first_pass_data.alignment_offset, None);

        let mut second_pass_middleware_data = MiddlewareData::default();
        second_pass_middleware_data.set(
            ARROW_NAME,
            first_pass.data.clone().expect("first pass recorded data"),
        );
        let second_pass = base_arrow::<FakeElement, FakeWindow>(
            ArrowOptions::new(FakeElement).padding(Padding::All(10.0)),
        )
        .compute(state(
            &second_pass_middleware_data,
            &reference,
            &floating,
            &rects,
            Placement::BottomStart,
            &platform,
        ));

        assert_eq!(second_pass.reset, None);
    }
}
