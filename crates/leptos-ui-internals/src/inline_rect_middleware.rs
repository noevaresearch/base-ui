//! Port of `packages/react/src/utils/popups/inlineRect.ts` — the ENGINE side: the
//! local `inline` middleware (`createInlineMiddleware`, `inlineRect.ts:239-292`)
//! standing in for Floating UI's own `inline()`. The pure line-rect helpers, the
//! trigger-side capture surface, and the coords vocabulary live in
//! [`super::inline_rect`].
//!
//! Rust adaptations (the hide_middleware.rs precedent):
//! - the `async fn` wrapper is dropped (the engine's `Middleware::compute` is
//!   synchronous) and `state.platform.getElementRects` is the direct
//!   `Platform::get_element_rects` call; the `typeof ... !== 'function'` guards
//!   dissolve into the type system;
//! - upstream's inline virtual element (`{ contextElement,
//!   getBoundingClientRect() { return rect; } }`, `:273-278`) is
//!   [`RectOverrideVirtualElement`], a minimal `floating_ui_utils::VirtualElement`
//!   whose `get_client_rects()` is `None` — the middleware's own
//!   `reference_client_rects` treats a rect-less virtual reference as the
//!   `typeof reference.getClientRects !== 'function'` early return (`:250-254`).

use std::cell::Cell;
use std::rc::Rc;

use floating_ui_core::{
    GetElementRectsArgs, Middleware as MiddlewareTrait, MiddlewareReturn, MiddlewareState, Reset,
    ResetRects, ResetValue,
};
use floating_ui_dom::{ElementOrVirtual, Rect as DomRect, VirtualElement as DomVirtualElement};
use floating_ui_utils::{ElementRects, VirtualElement};
use web_sys::Element;

use super::inline_rect::{
    InlineRectCoords, InlineRectCoordsRef, element_client_rects, get_inline_reference_rect,
};

/// A minimal [`VirtualElement`] implementation carrying a fixed client rect —
/// upstream's inline object literal (`inlineRect.ts:273-278`).
#[derive(Clone, Debug)]
pub struct RectOverrideVirtualElement {
    rect: DomRect,
    context_element: Option<Element>,
}

impl PartialEq for RectOverrideVirtualElement {
    fn eq(&self, other: &Self) -> bool {
        self.rect == other.rect
            && match (&self.context_element, &other.context_element) {
                (Some(a), Some(b)) => a == b,
                (None, None) => true,
                _ => false,
            }
    }
}

impl RectOverrideVirtualElement {
    /// The override over the reference's context element (`getContextElement`,
    /// `inlineRect.ts:192-199`).
    pub fn new(rect: DomRect, context_element: Option<Element>) -> Self {
        RectOverrideVirtualElement {
            rect,
            context_element,
        }
    }
}

impl DomVirtualElement<Element> for RectOverrideVirtualElement {
    fn get_bounding_client_rect(&self) -> floating_ui_dom::ClientRectObject {
        floating_ui_dom::ClientRectObject {
            x: self.rect.x,
            y: self.rect.y,
            width: self.rect.width,
            height: self.rect.height,
            top: self.rect.y,
            right: self.rect.x + self.rect.width,
            bottom: self.rect.y + self.rect.height,
            left: self.rect.x,
        }
    }

    fn get_client_rects(&self) -> Option<Vec<floating_ui_utils::ClientRectObject>> {
        None
    }

    fn context_element(&self) -> Option<Element> {
        self.context_element.clone()
    }
}

/// Reads the reference's client rects from the engine state — the
/// `reference.getClientRects()` call (`inlineRect.ts:255-257`) over both arms of
/// `ElementOrVirtual`. Returns `None` when the reference cannot produce rects
/// (upstream's `{}` early return, `:250-254`).
pub fn reference_client_rects(
    reference: &ElementOrVirtual,
) -> Option<Vec<floating_ui_dom::ClientRectObject>> {
    match reference {
        ElementOrVirtual::Element(element) => Some(
            floating_ui_dom::ClientRectObject::from_dom_rect_list(element.get_client_rects()),
        ),
        ElementOrVirtual::VirtualElement(virtual_element) => virtual_element.get_client_rects(),
    }
}

fn no_return() -> MiddlewareReturn {
    MiddlewareReturn {
        x: None,
        y: None,
        data: None,
        reset: None,
    }
}

/// The `inline` engine middleware (`inlineRect.ts:239-292`) — re-derives the
/// hovered-line reference rect and asks the platform to reset with the
/// line-scoped rects. Handwritten `PartialEq` (the coords cell compares by
/// pointer, matching the shared-handle identity the positioner passes).
#[derive(Clone)]
pub struct InlineRectMiddleware {
    coords_ref: InlineRectCoordsRef,
}

impl std::fmt::Debug for InlineRectMiddleware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `Cell<Option<..>>` has no Debug without Copy; the middleware prints as
        // the coords cell's pointer identity.
        f.debug_struct("InlineRectMiddleware")
            .finish_non_exhaustive()
    }
}

impl PartialEq for InlineRectMiddleware {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.coords_ref, &other.coords_ref)
    }
}

impl InlineRectMiddleware {
    /// Constructs the middleware over the shared coords cell
    /// (`createInlineMiddleware(inlineRectCoordsRef)`,
    /// PreviewCardPositioner.tsx:77).
    pub fn new(coords_ref: InlineRectCoordsRef) -> Self {
        InlineRectMiddleware { coords_ref }
    }
}

impl MiddlewareTrait<Element, web_sys::Window> for InlineRectMiddleware {
    fn name(&self) -> &'static str {
        "inline"
    }

    fn compute(&self, state: MiddlewareState<Element, web_sys::Window>) -> MiddlewareReturn {
        // The coords the middleware consults (`:262-269`): only the ones captured
        // for THIS reference (`coords?.element === reference ||
        // coords?.element === contextElement`). The cell take/set keeps the
        // non-destructive read the Cell handle requires.
        let reference = &state.elements.reference;
        let context_element = reference.clone().resolve();
        let current_coords: Option<InlineRectCoords> = {
            let coords = self.coords_ref.take();
            let resolved = coords
                .as_ref()
                .filter(|coords| {
                    let reference_element = match reference {
                        ElementOrVirtual::Element(element) => Some(*element),
                        ElementOrVirtual::VirtualElement(_) => None,
                    };
                    reference_element
                        .map(|element| *element == coords.element)
                        .unwrap_or(false)
                        || context_element
                            .as_ref()
                            .map(|context| *context == coords.element)
                            .unwrap_or(false)
                })
                .cloned();
            self.coords_ref.set(coords);
            resolved
        };

        let Some(rects) = reference_client_rects(reference) else {
            return no_return();
        };

        // `placement[0]` (`:131`): the Debug spelling lowercased begins with the
        // physical side's initial (topstart → 't', leftend → 'l', …).
        let placement = format!("{:?}", state.placement).to_lowercase();
        let side = placement.chars().next().unwrap_or('b');
        let rect_like = get_inline_reference_rect(
            &client_rects_to_rect_likes(&rects),
            side,
            current_coords.as_ref(),
        );
        let Some(rect_like) = rect_like else {
            return no_return();
        };

        // `state.platform.getElementRects(...)` (`:270-282`): the engine re-reads
        // the reference rect through a virtual element carrying the override —
        // the same request shape, the rects come back in the engine's vocabulary.
        let override_element =
            RectOverrideVirtualElement::new(rect_like_to_rect(rect_like), context_element);
        let reset_rects: ElementRects = state.platform.get_element_rects(GetElementRectsArgs {
            reference: ElementOrVirtual::VirtualElement(Box::new(override_element)),
            floating: state.elements.floating,
            strategy: state.strategy,
        });

        // The unchanged-rect bail (`:284-291`).
        let reference_rects = &state.rects.reference;
        if reference_rects.x == reset_rects.reference.x
            && reference_rects.y == reset_rects.reference.y
            && reference_rects.width == reset_rects.reference.width
            && reference_rects.height == reset_rects.reference.height
        {
            return no_return();
        }

        MiddlewareReturn {
            x: None,
            y: None,
            data: None,
            reset: Some(Reset::Value(ResetValue {
                placement: None,
                rects: Some(ResetRects::Value(reset_rects)),
            })),
        }
    }
}

/// Adapts the engine's `ClientRectObject` list into the crate's [`super::inline_rect::RectLike`] boxes.
fn client_rects_to_rect_likes(
    rects: &[floating_ui_utils::ClientRectObject],
) -> Vec<super::inline_rect::RectLike> {
    rects
        .iter()
        .map(|rect| {
            super::inline_rect::RectLike::from_dom_rect(
                rect.left,
                rect.top,
                rect.right,
                rect.bottom,
                rect.width,
                rect.height,
            )
        })
        .collect()
}

/// The box back into the engine's [`Rect`].
fn rect_like_to_rect(rect: super::inline_rect::RectLike) -> DomRect {
    DomRect {
        x: rect.left,
        y: rect.top,
        width: rect.width,
        height: rect.height,
    }
}

// The `element_client_rects` helper is re-exported for the view layer's trigger
// wiring (the `getInlineRectTriggerProps` consumers) — kept referenced so the
// import stays live while the preview-card parts land in the next checkpoint.
#[allow(unused_imports)]
use element_client_rects as _element_client_rects_keepalive;
