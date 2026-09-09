//! Provisional port of `packages/react/src/utils/FocusGuard.tsx` — the visually-hidden,
//! tabbable `span` the FloatingPortal/FloatingFocusManager pair renders flanking floating
//! content so Tab can be routed back inside (or to the next/previous document tabbable)
//! (`specs/library/floating-ui-react/parts/components.md`, "Non-modal tabbability guards":
//! "Outside `FocusGuard` spans route Tab back inside or to the next/previous document
//! tabbable").
//!
//! The source file belongs to the `infra: utils` TODO item (not yet ported); the
//! floating-ui unit cannot realize its guards without it, so it ports here first — the
//! `popup_trigger_map` precedent — and moves behind a re-export of the react-utils port
//! when that item lands.
//!
//! ## Rust adaptations
//!
//! - Upstream's `FocusGuard` is a React component receiving `React.ComponentPropsWithoutRef
//!   <'span'>` plus a forwarded ref; the port is a factory returning the element, with the
//!   caller's attributes in a [`FocusGuardProps`] bag and the caller writing the returned
//!   element into its own ref handle (React's forwardRef plumbing is view-layer).
//! - The VoiceOver role effect (`FocusGuard.tsx:16-24`) runs once per mount with an empty
//!   dependency array, so the port computes the role once at element-creation time (a
//!   component body runs once in the port's model) instead of in an effect.
//! - Attribute application order follows the JSX spread order (`FocusGuard.tsx:32-40`): the
//!   caller's props first, then the visually-hidden style, the computed `aria-hidden`, the
//!   `tabIndex: 0`/`role` rest-props (overriding any caller value), and finally the
//!   `data-base-ui-focus-guard` marker.

use web_sys::HtmlElement;
use web_sys::wasm_bindgen::JsCast;

use leptos_ui_utils::platform::platform;
use leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN;

/// The caller's attribute spread (`FocusGuard.tsx:11` —
/// `React.ComponentPropsWithoutRef<'span'>`): the attribute name/value pairs applied to the
/// span before the component's own values (e.g. `data-type="inside"` from the focus
/// managers, `FloatingPortal.tsx:256`).
#[derive(Default)]
pub struct FocusGuardProps {
    pub attributes: Vec<(String, String)>,
}

/// Port of `FocusGuard` (`FocusGuard.tsx:10-42`): a visually-hidden, tabbable `span`
/// (`tabIndex: 0`) that is `aria-hidden` unless the VoiceOver/WebKit workaround applies
/// (`role="button"` lets VoiceOver's virtual cursor trigger the focus handlers,
/// `FocusGuard.tsx:17-20`), marked with `data-base-ui-focus-guard`.
pub fn create_focus_guard(props: FocusGuardProps) -> HtmlElement {
    let document = web_sys::window()
        .expect("FocusGuard renders only in a DOM realm")
        .document()
        .expect("document");
    let span: HtmlElement = document
        .create_element("span")
        .expect("span creation")
        .dyn_into()
        .expect("span is an HtmlElement");

    // `{...props}` (`:34`).
    for (name, value) in &props.attributes {
        span.set_attribute(name, value)
            .expect("attribute name is valid");
    }

    // `style={visuallyHidden}` (`:36`) — applied after the spread, overriding any caller
    // style, the way the JSX attribute order does.
    for (name, value) in VISUALLY_HIDDEN {
        span.style().set_property(name, value).expect("style set");
    }

    // The VoiceOver role (`:16-24`, computed once — see the module docs).
    let is_voice_over_cursor = platform().screen_reader.voice_over && platform().engine.webkit;
    let role = is_voice_over_cursor.then_some("button");

    // `aria-hidden={role ? undefined : true}` (`:37`).
    if role.is_none() {
        span.set_attribute("aria-hidden", "true")
            .expect("attribute name is valid");
    }

    // `{...restProps}` (`:26-30`): `tabIndex: 0` plus the role ("only for VoiceOver").
    span.set_attribute("tabindex", "0")
        .expect("attribute name is valid");
    if let Some(role) = role {
        span.set_attribute("role", role)
            .expect("attribute name is valid");
    }

    // `data-base-ui-focus-guard=""` (`:39`).
    span.set_attribute("data-base-ui-focus-guard", "")
        .expect("attribute name is valid");

    span
}
