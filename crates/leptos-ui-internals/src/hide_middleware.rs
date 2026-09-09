//! Port of `packages/react/src/utils/hideMiddleware.ts` — the custom `hide` middleware
//! Base UI layers over the positioning engine (`useAnchorPositioning.ts:31,447`).
//!
//! It mirrors Floating UI's own `hide()` *referenceHidden* strategy, but is kept as a
//! separate middleware so its data lands under the `"hide"` key with exactly the shape
//! `useAnchorPositioning`'s `anchorHidden` read expects
//! (`useAnchorPositioning.ts:583`).
//!
//! Rust adaptations:
//! - Upstream's `async fn` exists only because the JS `Middleware` contract is
//!   promise-based; the Rust `Middleware::compute` is synchronous and
//!   `floating_ui_core::detect_overflow` is a synchronous call, so the port drops the
//!   async wrapper with no behavioral change.
//! - The returned data is a `serde_json` object built with `json!` — the engine's
//!   `MiddlewareData` is a JSON map and the crate has no direct `serde` derive
//!   dependency.

use floating_ui_core::{
    DetectOverflowOptions, ElementContext, Middleware as MiddlewareTrait, MiddlewareReturn,
    MiddlewareState,
};
use serde_json::json;
use web_sys::{Element, Window};

/// The middleware name — upstream's `name: 'hide'` (`hideMiddleware.ts:5`).
pub const HIDE_NAME: &str = "hide";

/// The custom `hide` middleware (`hideMiddleware.ts:4-25`). Reports
/// `referenceHidden: true` when the reference element is fully clipped by its clipping
/// ancestors (or has collapsed to zero size at the origin), for consumers to hide the
/// popup while its anchor is out of view.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HideMiddleware;

impl HideMiddleware {
    /// Constructs the middleware (upstream is a bare object literal).
    pub fn new() -> Self {
        HideMiddleware
    }
}

impl MiddlewareTrait<Element, Window> for HideMiddleware {
    fn name(&self) -> &'static str {
        HIDE_NAME
    }

    fn compute(&self, state: MiddlewareState<Element, Window>) -> MiddlewareReturn {
        let MiddlewareState { rects, .. } = &state;
        let reference = &rects.reference;
        // The zero-size-at-origin check (`hideMiddleware.ts:8`).
        let anchor_hidden = reference.width == 0.0
            && reference.height == 0.0
            && reference.x == 0.0
            && reference.y == 0.0;

        // "Mirrors Floating UI's `hide()` referenceHidden strategy" (`:9-11`) — overflow
        // measured for the *reference* element context, every other option at its
        // documented default (`floating-ui`'s `hide()` options shape).
        let overflow = floating_ui_core::detect_overflow(
            state,
            DetectOverflowOptions {
                boundary: None,
                root_boundary: None,
                element_context: Some(ElementContext::Reference),
                alt_boundary: None,
                padding: None,
            },
        );

        let reference_hidden = overflow.top - reference.height >= 0.0
            || overflow.right - reference.width >= 0.0
            || overflow.bottom - reference.height >= 0.0
            || overflow.left - reference.width >= 0.0;

        MiddlewareReturn {
            x: None,
            y: None,
            data: Some(json!({
                "referenceHidden": reference_hidden || anchor_hidden,
            })),
            reset: None,
        }
    }
}

/// Constructs the custom `hide` middleware — the `hide` import site
/// (`useAnchorPositioning.ts:31`).
pub fn hide() -> HideMiddleware {
    HideMiddleware::new()
}
