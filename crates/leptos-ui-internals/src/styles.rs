//! Port of `packages/react/src/utils/styles.tsx` — the `styleDisableScrollbar` helper
//! ScrollArea and Select scrolling use to hide scrollbars on elements they scroll
//! programmatically (implementation.md, "Downstream consumers": "ScrollArea and Select
//! scrolling (`styleDisableScrollbar`, ...)").
//!
//! Upstream's `getElement(nonce)` returns a React `<style>` element whose `href`/
//! `precedence` props are the React-runtime's stylesheet dedup keys
//! (`styles.tsx:7-10`); the element body is the two rule sets that hide the scrollbar.
//! The port exposes the class name, the CSS body, and a `RenderedElement` description
//! factory (the `prehydration_script` convention — `tag: "style"` + `inner_html`).
//! The React `href`/`precedence` dedup has no runtime counterpart in the crate:
//! injecting the description twice is the caller's responsibility to avoid, and the
//! `nonce` parameter flows into the element's `nonce` attribute for the CSP contract
//! (the `CSPContextValue` docs cover inline `<style>` tags).
//!
//! No test anywhere targets this helper (implementation.md, "Submodules with no test
//! anywhere").

use std::rc::Rc;

use crate::floating_ui::element_props::ElementAttributeFn;
use crate::use_render_element::{RenderElementProps, RenderedElement};

/// The injected class name (`styles.tsx:1`).
pub const DISABLE_SCROLLBAR_CLASS_NAME: &str = "base-ui-disable-scrollbar";

/// The stylesheet body (`styles.tsx:8`): `scrollbar-width: none` plus the
/// `::-webkit-scrollbar` display rule, class-scoped.
pub const DISABLE_SCROLLBAR_CSS: &str = ".base-ui-disable-scrollbar{scrollbar-width:none}\
.base-ui-disable-scrollbar::-webkit-scrollbar{display:none}";

/// Port of `styleDisableScrollbar` (`styles.tsx:3-12`): the class name plus the
/// element factory. Upstream is a plain object with a `className` string and a
/// `getElement` method.
pub struct StyleDisableScrollbar;

impl StyleDisableScrollbar {
    /// Upstream's `className` member (`styles.tsx:4`).
    pub const CLASS_NAME: &'static str = DISABLE_SCROLLBAR_CLASS_NAME;

    /// Port of `getElement(nonce?)` (`styles.tsx:5-11`): the `<style>` element
    /// description carrying the CSS body and the optional CSP nonce.
    pub fn get_element(nonce: Option<String>) -> RenderedElement {
        let nonce_attribute: ElementAttributeFn = Rc::new(move || nonce.clone());

        RenderedElement {
            tag: "style".to_string(),
            props: RenderElementProps {
                inner_html: Some(DISABLE_SCROLLBAR_CSS.to_string()),
                handlers: crate::use_render_element::RenderElementHandlers {
                    attributes: vec![("nonce".to_string(), nonce_attribute)],
                    ..Default::default()
                },
                ..RenderElementProps::default()
            },
        }
    }
}
