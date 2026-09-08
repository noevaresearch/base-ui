//! Port of the *type-shape* half of `packages/react/src/internals/createBaseUIEventDetails.ts`
//! — the two event-details types the `infra: types` unit re-exports as its public surface
//! (`packages/react/src/types/index.ts:3-6`). The runtime-factories half of the upstream file
//! (`createChangeEventDetails`/`createGenericEventDetails`, `:118-166`) is deliberately NOT
//! ported here: it belongs to the `infra: internals` unit, and the types unit re-exports only
//! the types ("this module is the type surface only" —
//! `specs/library/types/implementation.md`, "Public API surface"). The constructors below are
//! plain struct-literal sugar for the type shape (the flag fields are private), not the
//! factories — in particular they do not reproduce the factories' omitted-argument defaults
//! (`event ?? new Event('base-ui')`, `customProperties ?? EMPTY_OBJECT`, `:129-132`,
//! `:159-162`).
//!
//! ## Upstream shape being ported
//!
//! - [`BaseUIChangeEventDetails`] ports `BaseUIChangeEventDetails` and the internal
//!   `BaseUIChangeEventDetail` it unwraps to (`:56-93`): `reason` (`:60`), the native `event`
//!   (`:64`), `cancel()` (`:68`, closure `:133-135`), `allowPropagation()` (`:72`, closure
//!   `:136-138`), the `isCanceled`/`isPropagationAllowed` indicators (`:76-80`, getters
//!   `:139-144`), `trigger` (`:84`, `:145`), intersected with caller-supplied
//!   `CustomProperties` (`:85`, spread `:146`). The exported type's
//!   `Reason extends string ? … : never` guard (`:93`) and the internal/exported type split
//!   are TypeScript soundness devices with no Rust counterpart — a Rust `reason` field is
//!   always a string, so the non-`string` → `never` collapse is moot (behavior.md,
//!   "Edge cases", documents the guard as type-level only).
//! - [`BaseUIGenericEventDetails`] ports `BaseUIGenericEventDetail`/`BaseUIGenericEventDetails`
//!   (`:98-112`): `reason` (`:101`), `event` (`:106`), intersected with `CustomProperties`
//!   (`:107`, spread `:163`).
//! - The per-reason native-event typing (`ReasonToEventMap`, `:4-47`, and the
//!   `ReasonToEvent<Reason>` conditional, `:52-54`) is a compile-time table; at runtime one
//!   event value crosses the boundary. The port flattens it into the `E` type parameter,
//!   defaulting to [`web_sys::Event`] — the map's own fallback for unlisted reasons
//!   (`:52-54`). The `REASONS` string registry the map is keyed by (`:2`) belongs to the
//!   internals unit and is not ported here, so `reason` is an owned `String`.
//!
//! ## Rust adaptations
//!
//! - The `& CustomProperties` intersections (`:85`, `:107`) become the typed `custom` field:
//!   JS spreads `...custom` into the object (`:146`, `:163`); Rust composes it as a field
//!   carrying the caller's own value.
//! - The `cancel`/`allowPropagation` closures mutate factory-local variables captured once
//!   (`:127-128`); the port stores the two flags as shared cells ([`Rc`] of [`Cell`]`<bool>`),
//!   so every clone of a details value shares one flag set — the JS object-identity semantics
//!   (the same details object handed to two handlers observes one `cancel()` call).
//! - The `isCanceled`/`isPropagationAllowed` getter properties (`:139-144`) port to the
//!   [`BaseUIChangeEventDetails::is_canceled`] / [`BaseUIChangeEventDetails::
//!   is_propagation_allowed`] accessor methods; `cancel`/`allowPropagation` keep their names
//!   as methods.
//! - The `event`/`trigger` fields are generic (`E`, `Tr`) rather than fixed DOM types: like
//!   the per-reason conditional they replace, the concrete event type is a compile-time
//!   choice, and the genericity is what lets non-DOM stubs instantiate the type in host tests.
//!   The defaults (`Event`, `Element`) are the DOM shapes upstream names (`:5`, `:52-54`,
//!   `:84`).

use std::cell::Cell;
use std::rc::Rc;

use web_sys::Event;

/// Details of custom change events emitted by Base UI components — upstream's
/// `BaseUIChangeEventDetails`
/// (`packages/react/src/internals/createBaseUIEventDetails.ts:90-93`, shape `:56-85`),
/// re-exported through the types unit (`packages/react/src/types/index.ts:3-6`).
///
/// [`Clone`] shares the cancel/propagation flag cells (see the module docs) and clones the
/// `event`/`trigger`/`custom` values.
#[derive(Clone, Debug)]
pub struct BaseUIChangeEventDetails<Custom = (), E = Event, Tr = web_sys::Element> {
    /// The reason for the event (`:60`).
    pub reason: String,
    /// The native event associated with the custom event (`:64`) — the per-reason
    /// `ReasonToEvent` typing flattened into the `E` parameter (see the module docs).
    pub event: E,
    /// The element that triggered the event, if applicable (`:84`).
    pub trigger: Option<Tr>,
    /// The caller-supplied `CustomProperties` intersection (`:85`, spread `:146`).
    pub custom: Custom,
    canceled: Rc<Cell<bool>>,
    propagation_allowed: Rc<Cell<bool>>,
}

impl<Custom, E, Tr> BaseUIChangeEventDetails<Custom, E, Tr> {
    /// Constructs the details value from its full shape — struct-literal sugar for the type,
    /// not the upstream factory (the omitted-argument defaults are not reproduced; see the
    /// module docs).
    pub fn new(reason: impl Into<String>, event: E, trigger: Option<Tr>, custom: Custom) -> Self {
        Self {
            reason: reason.into(),
            event,
            trigger,
            custom,
            canceled: Rc::new(Cell::new(false)),
            propagation_allowed: Rc::new(Cell::new(false)),
        }
    }

    /// Cancels Base UI from handling the event (`:68`; the `cancel` closure `:133-135`).
    pub fn cancel(&self) {
        self.canceled.set(true);
    }

    /// Allows the event to propagate in cases where Base UI will stop the propagation
    /// (`:72`; the `allowPropagation` closure `:136-138`).
    pub fn allow_propagation(&self) {
        self.propagation_allowed.set(true);
    }

    /// Indicates whether the event has been canceled (`:76`; the `isCanceled` getter
    /// `:139-141`).
    pub fn is_canceled(&self) -> bool {
        self.canceled.get()
    }

    /// Indicates whether the event is allowed to propagate (`:80`; the
    /// `isPropagationAllowed` getter `:142-144`).
    pub fn is_propagation_allowed(&self) -> bool {
        self.propagation_allowed.get()
    }
}

/// Details of custom generic events emitted by Base UI components — upstream's
/// `BaseUIGenericEventDetails`
/// (`packages/react/src/internals/createBaseUIEventDetails.ts:109-112`, shape `:98-107`),
/// re-exported through the types unit (`packages/react/src/types/index.ts:3-6`).
///
/// Unlike the change-details type, no flags cross this boundary — the shape is only `reason`,
/// `event`, and the caller's `CustomProperties` — so it is a plain value type, including
/// [`PartialEq`] (the change-details type holds interior-mutable shared flags and therefore
/// deliberately has no value-equality impl).
#[derive(Clone, Debug, PartialEq)]
pub struct BaseUIGenericEventDetails<Custom = (), E = Event> {
    /// The reason for the event (`:101`).
    pub reason: String,
    /// The native event associated with the custom event (`:106`).
    pub event: E,
    /// The caller-supplied `CustomProperties` intersection (`:107`, spread `:163`).
    pub custom: Custom,
}

impl<Custom, E> BaseUIGenericEventDetails<Custom, E> {
    /// Constructs the details value from its full shape — struct-literal sugar for the type,
    /// not the upstream factory (see the module docs).
    pub fn new(reason: impl Into<String>, event: E, custom: Custom) -> Self {
        Self {
            reason: reason.into(),
            event,
            custom,
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct FakeEvent(&'static str);

    #[derive(Clone, Debug, PartialEq)]
    struct FakeElement(&'static str);

    // Pins the flags' initial state (`packages/react/src/internals/createBaseUIEventDetails.
    // ts:127-128` — `let canceled = false; let allowPropagation = false`): a fresh details
    // value is neither canceled nor propagation-allowed (behavior.md, "Events": the
    // cancel()/allowPropagation() calls "mutate internal flags surfaced via the
    // isCanceled/isPropagationAllowed getters").
    #[test]
    fn fresh_details_are_neither_canceled_nor_propagation_allowed() {
        let details: BaseUIChangeEventDetails<(), FakeEvent, FakeElement> =
            BaseUIChangeEventDetails::new("none", FakeEvent("e"), None, ());
        assert!(!details.is_canceled(), "a fresh value is not canceled");
        assert!(
            !details.is_propagation_allowed(),
            "a fresh value does not allow propagation"
        );
    }

    // Pins cancel() → isCanceled (`:133-135` writing the flag, `:139-141` reading it) —
    // behavior.md, "Events": cancel() "stops Base UI's internal handling".
    #[test]
    fn cancel_marks_the_details_canceled() {
        let details: BaseUIChangeEventDetails<(), FakeEvent, FakeElement> =
            BaseUIChangeEventDetails::new("itemPress", FakeEvent("e"), None, ());
        details.cancel();
        assert!(details.is_canceled(), "cancel() sets the canceled flag");
    }

    // Pins allowPropagation() → isPropagationAllowed (`:136-138`, `:142-144`) — behavior.md,
    // "Events": allowPropagation() "lets the event propagate where Base UI would stop it".
    #[test]
    fn allow_propagation_marks_the_details_propagation_allowed() {
        let details: BaseUIChangeEventDetails<(), FakeEvent, FakeElement> =
            BaseUIChangeEventDetails::new("outsidePress", FakeEvent("e"), None, ());
        details.allow_propagation();
        assert!(
            details.is_propagation_allowed(),
            "allowPropagation() sets the propagation flag"
        );
    }

    // Pins that the two flags are independent — upstream they are two separate locals
    // (`:127-128`), so canceling says nothing about propagation and vice versa.
    #[test]
    fn the_two_flags_are_independent() {
        let canceled: BaseUIChangeEventDetails<(), FakeEvent, FakeElement> =
            BaseUIChangeEventDetails::new("escapeKey", FakeEvent("e"), None, ());
        canceled.cancel();
        assert!(
            !canceled.is_propagation_allowed(),
            "cancel() does not touch the propagation flag"
        );

        let allowed: BaseUIChangeEventDetails<(), FakeEvent, FakeElement> =
            BaseUIChangeEventDetails::new("escapeKey", FakeEvent("e"), None, ());
        allowed.allow_propagation();
        assert!(
            !allowed.is_canceled(),
            "allowPropagation() does not touch the canceled flag"
        );
    }

    // Pins the field surface (`:130-147`): reason, event, trigger, and the custom
    // intersection cross verbatim.
    #[test]
    fn the_fields_carry_the_constructor_arguments() {
        let details = BaseUIChangeEventDetails::new(
            "triggerPress",
            FakeEvent("pointer"),
            Some(FakeElement("button")),
            ("key", 7u32),
        );
        assert_eq!(details.reason, "triggerPress");
        assert_eq!(details.event, FakeEvent("pointer"));
        assert_eq!(details.trigger, Some(FakeElement("button")));
        assert_eq!(details.custom, ("key", 7u32));
    }

    // Pins the shared-flag clone semantics (module docs "Rust adaptations"): the JS details
    // object handed to two handlers is one object, so a clone's cancel() is observable
    // through the original — the closure-capture semantics of `:127-135`.
    #[test]
    fn clones_share_the_flag_cells() {
        let details: BaseUIChangeEventDetails<(), FakeEvent, FakeElement> =
            BaseUIChangeEventDetails::new("closePress", FakeEvent("e"), None, ());
        let clone = details.clone();
        clone.cancel();
        clone.allow_propagation();
        assert!(
            details.is_canceled() && details.is_propagation_allowed(),
            "a clone's flag mutations are observable through the original"
        );
    }

    // Pins the generic-details shape (`:98-107`): only reason + event + custom cross the
    // boundary — no flags, no trigger.
    #[test]
    fn generic_details_carry_reason_event_and_custom() {
        let details = BaseUIGenericEventDetails::new("listNavigation", FakeEvent("keydown"), 42u8);
        assert_eq!(details.reason, "listNavigation");
        assert_eq!(details.event, FakeEvent("keydown"));
        assert_eq!(details.custom, 42u8);

        let clone = details.clone();
        assert_eq!(clone, details, "the generic details are plain value clones");
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // Pins the default instantiation against real DOM values: `E = Event` is the fallback
    // type the ReasonToEvent conditional resolves to for unlisted reasons (`:52-54`) and
    // `Tr = Element` is the `trigger` field's type (`:84`).
    #[wasm_bindgen_test]
    fn the_default_instantiation_carries_real_dom_values() {
        let event = web_sys::Event::new("base-ui").unwrap();
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();
        let details = BaseUIChangeEventDetails::<(), Event, web_sys::Element>::new(
            "triggerPress",
            event,
            Some(element.clone()),
            (),
        );
        assert_eq!(details.reason, "triggerPress");
        assert_eq!(details.event.type_(), "base-ui");
        details.cancel();
        assert!(details.is_canceled());
        assert_eq!(details.trigger, Some(element));
    }

    // Pins the generic-details default instantiation (`E = Event`) with a real event.
    #[wasm_bindgen_test]
    fn generic_details_default_to_the_fallback_event_type() {
        let details = BaseUIGenericEventDetails::<(), Event>::new(
            "none",
            web_sys::Event::new("base-ui").unwrap(),
            (),
        );
        assert_eq!(details.reason, "none");
        assert_eq!(details.event.type_(), "base-ui");
    }
}
