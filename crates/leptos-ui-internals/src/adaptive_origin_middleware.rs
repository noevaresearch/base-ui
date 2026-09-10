//! Port of `packages/react/src/utils/adaptiveOriginMiddleware.ts` — the custom
//! `adaptiveOrigin` Floating UI middleware a popup family registers into its store for
//! the viewport's lifetime, re-deriving the popup's x/y so a size transition keeps its
//! transform-origin visually anchored (`usePopupViewport.tsx:112-117` registers it;
//! `internals/useAnchorPositioning.ts:497` reads its data).
//!
//! The middleware only acts when the popup has a CSS transition
//! (`adaptiveOriginMiddleware.ts:20-29`); it re-derives x/y from the offset parent
//! (the visual viewport for `fixed` strategy, the document element or the offset
//! parent element otherwise) so that for `left`/`top` sides the coordinate is
//! expressed from the opposite edge (`:31-63`), and reports the CSS properties the
//! coordinates apply to through `sideX`/`sideY` (the [`AdaptiveOriginData`] shape the
//! positioner-styles split reads).
//!
//! Rust adaptations:
//! - The `async fn` exists only because the JS `Middleware` contract is promise-based;
//!   the Rust `Middleware::compute` is synchronous and the platform calls it forwards
//!   are synchronous, so the port drops the async wrapper with no behavioral change
//!   (the `hide_middleware` precedent).
//! - `await platform.isElement?.(offsetParent)` (`:47`) discriminated the offset
//!   parent's type at runtime; the port's [`OwnedElementOrWindow`] enum carries the
//!   discrimination in its variants, so the `Element` arm replaces the call.

use floating_ui_core::{Middleware as MiddlewareTrait, MiddlewareReturn, MiddlewareState};
use floating_ui_utils::{OwnedElementOrWindow, Side, Strategy, get_side};
use serde_json::json;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Node, Window};

use leptos_ui_utils::owner::{owner_document, owner_window};

use crate::adaptive_origin_constants::{AdaptiveOriginData, default_sides};

/// The middleware name — upstream's `name: 'adaptiveOrigin'`
/// (`adaptiveOriginMiddleware.ts:7`).
pub const ADAPTIVE_ORIGIN_NAME: &str = "adaptiveOrigin";

/// The `visualViewport` dimensions for `fixed` strategy when the window exposes the
/// API (`adaptiveOriginMiddleware.ts:36-40`).
fn visual_viewport_dimensions(win: &Window) -> Option<(f64, f64)> {
    let viewport = win.visual_viewport()?;
    Some((f64::from(viewport.width()), f64::from(viewport.height())))
}

/// The offset-parent dimensions the coordinates re-derive against
/// (`adaptiveOriginMiddleware.ts:31-49`): the visual viewport for `fixed` strategy,
/// the document element's client size when the offset parent is the window, the
/// platform-measured size when it is an element, and `{0, 0}` when the platform
/// declines to answer.
fn offset_dimensions(
    state: &MiddlewareState<'_, web_sys::Element, Window>,
    win: &Window,
    floating: &web_sys::Element,
) -> (f64, f64) {
    if state.strategy == Strategy::Fixed {
        if let Some(dimensions) = visual_viewport_dimensions(win) {
            return dimensions;
        }
    }

    let Some(offset_parent) = state.platform.get_offset_parent(floating) else {
        return (0.0, 0.0);
    };

    match offset_parent {
        // `offsetParent === win` (`:41`): the document element's client size.
        OwnedElementOrWindow::Window(_) => {
            let doc = owner_document(Some(floating.as_ref() as &Node));
            let document_element = doc.document_element();
            let dimensions = document_element.and_then(|element| {
                element.dyn_into::<HtmlElement>().ok().map(|html| {
                    (
                        f64::from(html.client_width()),
                        f64::from(html.client_height()),
                    )
                })
            });
            dimensions.unwrap_or((0.0, 0.0))
        }
        // `await platform.isElement?.(offsetParent)` + `getDimensions` (`:47-48`).
        OwnedElementOrWindow::Element(element) => {
            let dimensions = state.platform.get_dimensions(&element);
            (dimensions.width, dimensions.height)
        }
    }
}

/// The custom `adaptiveOrigin` middleware (`adaptiveOriginMiddleware.ts:6-73`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AdaptiveOriginMiddleware;

impl AdaptiveOriginMiddleware {
    /// Constructs the middleware (upstream is a bare object literal).
    pub fn new() -> Self {
        AdaptiveOriginMiddleware
    }
}

impl MiddlewareTrait<web_sys::Element, Window> for AdaptiveOriginMiddleware {
    fn name(&self) -> &'static str {
        ADAPTIVE_ORIGIN_NAME
    }

    fn compute(&self, state: MiddlewareState<web_sys::Element, Window>) -> MiddlewareReturn {
        let MiddlewareState {
            x: raw_x,
            y: raw_y,
            rects,
            elements,
            placement,
            ..
        } = &state;
        let floating = &elements.floating;
        let float_rect = &rects.floating;

        let win: Window = owner_window(Some(floating.as_ref() as &Node));
        let styles = win.get_computed_style(floating).ok().flatten();
        // Upstream reads the `transitionDuration` camelCase IDL accessor
        // (`adaptiveOriginMiddleware.ts:21`); kebab-case `getPropertyValue` is the
        // equivalent read (camelCase lookups return "" in modern Chrome — the
        // `composite.rs` scroll-margin precedent).
        let transition_duration = styles
            .and_then(|style| style.get_property_value("transition-duration").ok())
            .unwrap_or_default();
        let has_transition = !transition_duration.is_empty() && transition_duration != "0s";

        if !has_transition {
            let default = default_sides();
            return MiddlewareReturn {
                x: Some(*raw_x),
                y: Some(*raw_y),
                data: Some(json!({
                    "sideX": default.side_x,
                    "sideY": default.side_y,
                })),
                reset: None,
            };
        }

        let (offset_width, offset_height) = offset_dimensions(&state, &win, floating);

        let current_side = get_side(*placement);
        let mut x = *raw_x;
        let mut y = *raw_y;

        // For `left`/`top` the coordinate is expressed from the opposite edge
        // (`adaptiveOriginMiddleware.ts:55-60`).
        if current_side == Side::Left {
            x = offset_width - (raw_x + float_rect.width);
        }
        if current_side == Side::Top {
            y = offset_height - (raw_y + float_rect.height);
        }

        let AdaptiveOriginData { side_x, side_y } = default_sides();
        let side_x = if current_side == Side::Left {
            "right".to_owned()
        } else {
            side_x
        };
        let side_y = if current_side == Side::Top {
            "bottom".to_owned()
        } else {
            side_y
        };

        MiddlewareReturn {
            x: Some(x),
            y: Some(y),
            data: Some(json!({
                "sideX": side_x,
                "sideY": side_y,
            })),
            reset: None,
        }
    }
}

/// Constructs the middleware for the positioner's middleware list.
pub fn adaptive_origin() -> AdaptiveOriginMiddleware {
    AdaptiveOriginMiddleware::new()
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;

    use floating_ui_dom::{
        ComputePositionConfig, ElementOrVirtual, Placement, Strategy, compute_position,
    };
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn mount_element(style: &str) -> web_sys::Element {
        let element = document().create_element("div").unwrap();
        element.set_attribute("style", style).unwrap();
        document().body().unwrap().append_child(&element).unwrap();
        element
    }

    fn run(
        reference: &web_sys::Element,
        floating: &web_sys::Element,
        placement: Placement,
        strategy: Strategy,
        with_middleware: bool,
    ) -> (f64, f64, Option<AdaptiveOriginData>) {
        let middleware: Vec<Box<dyn MiddlewareTrait<web_sys::Element, Window>>> = if with_middleware
        {
            vec![Box::new(adaptive_origin())]
        } else {
            Vec::new()
        };
        let config = ComputePositionConfig {
            placement: Some(placement),
            strategy: Some(strategy),
            middleware: Some(middleware),
            ..Default::default()
        };
        let result = compute_position(ElementOrVirtual::Element(reference), floating, config);
        let data = result
            .middleware_data
            .get(ADAPTIVE_ORIGIN_NAME)
            .map(|value| AdaptiveOriginData::from_middleware_data(Some(value)));
        (result.x, result.y, data)
    }

    /// Mounts a positioned container (the deterministic offset parent for the pair),
    /// a mid-container anchor, and a popup, returning all three.
    fn setup_pair(floating_style: &str) -> (web_sys::Element, web_sys::Element, web_sys::Element) {
        let container =
            mount_element("position: absolute; left: 0; top: 0; width: 400px; height: 300px;");
        let reference = mount_element(
            "position: absolute; left: 200px; top: 200px; width: 20px; height: 20px;",
        );
        let floating = mount_element(floating_style);
        container.append_child(&reference).unwrap();
        container.append_child(&floating).unwrap();
        (container, reference, floating)
    }

    /// The anchor and popup are re-parented into the container by `setup_pair`, so
    /// removing the container removes the whole fixture.
    fn teardown(container: &web_sys::Element) {
        document().body().unwrap().remove_child(container).unwrap();
    }

    // Without a CSS transition the middleware is a passthrough
    // (`adaptiveOriginMiddleware.ts:20-29`): the raw coordinates come back unchanged
    // and the data reports the default sides.
    #[wasm_bindgen_test]
    fn passes_through_without_a_transition() {
        let (container, reference, floating) =
            setup_pair("position: absolute; left: 0; top: 0; width: 50px; height: 50px;");

        let (raw_x, raw_y, _) = run(
            &reference,
            &floating,
            Placement::Left,
            Strategy::Absolute,
            false,
        );
        let (x, y, data) = run(
            &reference,
            &floating,
            Placement::Left,
            Strategy::Absolute,
            true,
        );

        assert_eq!(x, raw_x);
        assert_eq!(y, raw_y);
        assert_eq!(
            data,
            Some(AdaptiveOriginData {
                side_x: "left".into(),
                side_y: "top".into()
            })
        );

        teardown(&container);
    }

    // With a transition and a `left` placement, x is re-expressed from the opposite
    // (`right`) edge of the offset parent (`:55-57`, `:62`) and `sideX` reports it.
    #[wasm_bindgen_test]
    fn re_expresses_the_left_side_from_the_opposite_edge() {
        let (container, reference, floating) = setup_pair(
            "position: absolute; left: 0; top: 0; width: 50px; height: 50px; transition-duration: 0.2s;",
        );
        let offset_width = f64::from(
            container
                .clone()
                .dyn_into::<HtmlElement>()
                .unwrap()
                .offset_width(),
        );

        let (raw_x, raw_y, _) = run(
            &reference,
            &floating,
            Placement::Left,
            Strategy::Absolute,
            false,
        );
        let (x, y, data) = run(
            &reference,
            &floating,
            Placement::Left,
            Strategy::Absolute,
            true,
        );

        assert_eq!(y, raw_y);
        assert_eq!(x, offset_width - (raw_x + 50.0));
        assert_eq!(data.unwrap().side_x, "right");

        teardown(&container);
    }

    // The `top` placement twin (`:58-60`, `:63`): y re-expressed from the bottom edge,
    // `sideY` reporting `bottom`, `sideX` at its default.
    #[wasm_bindgen_test]
    fn re_expresses_the_top_side_from_the_opposite_edge() {
        let (container, reference, floating) = setup_pair(
            "position: absolute; left: 0; top: 0; width: 50px; height: 50px; transition-duration: 0.2s;",
        );
        let offset_height = f64::from(
            container
                .clone()
                .dyn_into::<HtmlElement>()
                .unwrap()
                .offset_height(),
        );

        let (raw_x, raw_y, _) = run(
            &reference,
            &floating,
            Placement::Top,
            Strategy::Absolute,
            false,
        );
        let (x, y, data) = run(
            &reference,
            &floating,
            Placement::Top,
            Strategy::Absolute,
            true,
        );

        assert_eq!(x, raw_x);
        assert_eq!(y, offset_height - (raw_y + 50.0));
        let data = data.unwrap();
        assert_eq!(data.side_x, "left");
        assert_eq!(data.side_y, "bottom");

        teardown(&container);
    }

    // `fixed` strategy reads the visual viewport instead of the offset parent
    // (`:36-40`).
    #[wasm_bindgen_test]
    fn uses_the_visual_viewport_for_fixed_strategy() {
        let reference =
            mount_element("position: fixed; left: 200px; top: 200px; width: 20px; height: 20px;");
        let floating = mount_element(
            "position: fixed; left: 0; top: 0; width: 50px; height: 50px; transition-duration: 0.2s;",
        );

        let (raw_x, _, _) = run(
            &reference,
            &floating,
            Placement::Left,
            Strategy::Fixed,
            false,
        );
        let (x, _, _) = run(
            &reference,
            &floating,
            Placement::Left,
            Strategy::Fixed,
            true,
        );

        let viewport_width = web_sys::window()
            .unwrap()
            .inner_width()
            .unwrap()
            .as_f64()
            .unwrap();
        assert_eq!(x, viewport_width - (raw_x + 50.0));

        document().body().unwrap().remove_child(&reference).unwrap();
        document().body().unwrap().remove_child(&floating).unwrap();
    }
}
