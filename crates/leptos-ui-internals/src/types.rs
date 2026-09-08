//! Port of `packages/react/src/types/index.ts` — the `infra: types` TODO item
//! (`specs/library/types/behavior.md`, `specs/library/types/implementation.md`).
//!
//! Upstream is a single 26-line, type-only module with no runtime code — no components, no
//! hooks, no DOM (`specs/library/types/behavior.md`, "Public API surface"): two re-exports of
//! the event-details types from the internals module (`:3-6` — ported in
//! [`crate::create_base_ui_event_details`], preserving the layering where the public type
//! barrel reaches into `internals/` rather than duplicating the definitions), and three type
//! declarations — [`HTMLProps`] (`:8-10`), [`ComponentRenderFn`] (`:12-21`, JSDoc `:13-17`),
//! and [`BaseUIEvent`] (`:23-26`). The port is likewise pure type vocabulary with no reactive
//! or view dependency.
//!
//! ## Rust adaptations
//!
//! - TypeScript types have no runtime to port; the spec's own porting note names the Rust
//!   equivalent as "a set of trait/enum type definitions"
//!   (`specs/library/types/implementation.md`, "State machine / hooks used"). These are
//!   concrete Rust types other units build on.
//! - [`HTMLProps`]: React's `HTMLAttributes<T>` surface is deliberately not reproduced as a
//!   Rust struct — attributes/classes/styles travel through Leptos's native attribute system
//!   rather than a custom merge utility (`specs/architecture.md`, "Prop / class / style
//!   merging (mergeProps)"). The one member `HTMLProps` *adds* — the explicit, always-allowed
//!   `ref` that lets the prop-getter pipeline carry and merge refs (`:8-10`;
//!   `specs/library/types/implementation.md`, "DOM/portal strategy and why") — is what the
//!   port defines: a fillable [`Rc`] of [`Cell`]`<Option<E>>` slot, the
//!   `{ current: T | null }` ref-object shape. The view-layer `node_ref: NodeRef<E>` prop
//!   convention wraps the same slot (`specs/architecture.md`, "Refs / DOM access"); the
//!   crate is view-free, so the slot type stays generic-friendly at this layer.
//! - [`ComponentRenderFn`]: the render-prop shape `(props, state) => React.ReactElement`
//!   (`:18-21`). The third parameter is the port's stand-in for `React.ReactElement` — the
//!   element/view type, instantiated at the view layer (this crate is view-free, per
//!   `csp_provider`'s realm convention). `Rc<dyn Fn>` is the shared-ownership callable
//!   convention for callback-typed values. Upstream gives neither type parameter a default
//!   (`:13-17`: "Props to be spread on the rendered element" / "Component's internal
//!   state"), and Rust type aliases cannot default parameters either, so all three are
//!   explicit.
//! - [`BaseUIEvent`]: the `E & { preventBaseUIHandler; baseUIHandlerPrevented? }`
//!   augmentation (`:23-26`) ports to a wrapper carrying the underlying event plus the
//!   shared mark — [`BaseUIEvent::prevent_base_ui_handler`] sets it (`:24`),
//!   [`BaseUIEvent::base_ui_handler_prevented`] exposes it (`:25`). This is the unit's only
//!   cross-boundary behavior, consumed by `mergeProps` (which runs Base UI's internal
//!   handler only when the flag is NOT set,
//!   `packages/react/src/merge-props/mergeProps.ts:239`) and `useButton` (which bails out
//!   when it is, `packages/react/src/internals/use-button/useButton.ts:123`) — the porting
//!   note in `specs/library/types/implementation.md`, "Dependencies on other Base UI
//!   internals". The mark is a shared cell so clones observe one prevention (JS object
//!   identity: the augmented event handed to two handlers is one object). The upstream
//!   optional/readonly shape (`baseUIHandlerPrevented?: boolean | undefined`, `:25`) — which
//!   lets unaugmented events satisfy `MaybeBaseUIEvent`
//!   (`packages/react/src/internals/types.ts:6-7`) — is a TypeScript assignability device:
//!   in Rust, code handling a plain event simply holds an `E`, and the flag is only read off
//!   values that were actually wrapped.

use std::cell::Cell;
use std::rc::Rc;

pub use crate::create_base_ui_event_details::{BaseUIChangeEventDetails, BaseUIGenericEventDetails};

/// Port of `HTMLProps<T = any>`
/// (`packages/react/src/types/index.ts:8-10`): the library's attribute-prop vocabulary —
/// React's event/attribute prop set plus an explicit, always-allowed `ref`.
///
/// Only the added `ref` member is defined here (see the module docs for why the rest of the
/// attribute surface is delegated to Leptos's native attribute system): a fillable slot the
/// render pipeline can carry and merge, defaulting to absent — upstream's `ref?: … |
/// undefined` (`:9`). The `E` parameter carries the element type (`React.Ref<T>`), defaulting
/// to [`web_sys::Element`] where upstream defaulted `T` to `any` (`:8`).
// Clone only: `Cell<T>: Debug` is bounded on `T: Copy`, which `Option<E>` never is, so a
// derived `Debug` cannot exist for this shape.
#[derive(Clone)]
pub struct HTMLProps<E = web_sys::Element> {
    /// The explicit, always-allowed `ref` slot (`:9`) — the member that lets the prop-getter
    /// pipeline carry and merge refs instead of dropping them the way React's own attribute
    /// types do.
    pub node_ref: Option<Rc<Cell<Option<E>>>>,
}

// Handwritten rather than derived: the derived impl would bound `E: Default`, but an empty
// slot is element-independent — `ref?` defaults to absent for every element type (`:9`).
impl<E> Default for HTMLProps<E> {
    fn default() -> Self {
        Self { node_ref: None }
    }
}

/// Port of `ComponentRenderFn<Props, State>`
/// (`packages/react/src/types/index.ts:12-21`): the render-prop shape — a function taking the
/// props to be spread on the rendered element and the component's internal state, returning
/// the element to render (upstream `React.ReactElement<unknown>`, `:21`).
///
/// The third parameter is the port's stand-in for `React.ReactElement`, instantiated with the
/// view-layer element/view type (see the module docs). Neither upstream type parameter has a
/// default (`:13-17`), and neither does this alias.
pub type ComponentRenderFn<Props, State, Element> = Rc<dyn Fn(Props, State) -> Element>;

/// Port of `BaseUIEvent<E extends React.SyntheticEvent<Element, Event>>`
/// (`packages/react/src/types/index.ts:23-26`): an event augmented with Base UI's own
/// prevention mark, semantically distinct from the DOM's `preventDefault` —
/// `preventBaseUIHandler()` means "do not run Base UI's own handler", and
/// `baseUIHandlerPrevented` exposes that mark to other internal handlers (behavior.md,
/// "Events"; consumer contract at `packages/react/src/merge-props/mergeProps.ts:31`: handlers
/// "must check `event.baseUIHandlerPrevented` themselves and bail out if it's true").
///
/// The wrapper keeps every member of the underlying event reachable through
/// [`BaseUIEvent::inner`] / [`BaseUIEvent::into_inner`] (the upstream intersection `E & { … }`
/// preserves `E`'s members). [`Clone`] shares the mark between clones and clones the event
/// value.
#[derive(Debug)]
pub struct BaseUIEvent<E> {
    event: E,
    base_ui_handler_prevented: Rc<Cell<bool>>,
}

impl<E> BaseUIEvent<E> {
    /// Wraps a plain event with the augmentation — the port-side analog of
    /// `makeEventPreventable` attaching the members to the event object before fan-out
    /// (`packages/react/src/merge-props/mergeProps.ts:236-238` wraps, `:239` reads).
    pub fn new(event: E) -> Self {
        Self {
            event,
            base_ui_handler_prevented: Rc::new(Cell::new(false)),
        }
    }

    /// `preventBaseUIHandler` (`:24`): marks the event as "do not run Base UI's own handler".
    /// Producers call this — e.g. `useButton`
    /// (`packages/react/src/internals/use-button/useButton.ts:147`) — and downstream readers
    /// consult [`BaseUIEvent::base_ui_handler_prevented`].
    pub fn prevent_base_ui_handler(&self) {
        self.base_ui_handler_prevented.set(true);
    }

    /// `baseUIHandlerPrevented` (`:25`): whether
    /// [`BaseUIEvent::prevent_base_ui_handler`] was called on this value.
    pub fn base_ui_handler_prevented(&self) -> bool {
        self.base_ui_handler_prevented.get()
    }

    /// The underlying event (`E & { … }` keeps every member of `E`).
    pub fn inner(&self) -> &E {
        &self.event
    }

    /// Unwraps the augmentation, returning the underlying event.
    pub fn into_inner(self) -> E {
        self.event
    }
}

impl<E: Clone> Clone for BaseUIEvent<E> {
    fn clone(&self) -> Self {
        Self {
            event: self.event.clone(),
            base_ui_handler_prevented: Rc::clone(&self.base_ui_handler_prevented),
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct FakeEvent(&'static str);

    // Pins the BaseUIEvent flag contract (`packages/react/src/types/index.ts:23-26`):
    // `baseUIHandlerPrevented` starts falsy, `preventBaseUIHandler()` marks it, and readers
    // observe the mark — the producer (`useButton.ts:147`) / reader (`mergeProps.ts:239`)
    // pair behavior.md, "Events" documents.
    #[test]
    fn a_fresh_event_is_not_marked_and_preventing_marks_it() {
        let event = BaseUIEvent::new(FakeEvent("pointerdown"));
        assert!(
            !event.base_ui_handler_prevented(),
            "a fresh augmented event is not marked"
        );
        event.prevent_base_ui_handler();
        assert!(
            event.base_ui_handler_prevented(),
            "preventBaseUIHandler() sets the mark"
        );
    }

    // Pins the shared-mark clone semantics (module docs "Rust adaptations"): the JS
    // augmented event handed to two handlers is one object, so a reader holding a clone
    // observes the producer's mark.
    #[test]
    fn clones_share_the_mark() {
        let event = BaseUIEvent::new(FakeEvent("keydown"));
        let reader = event.clone();
        event.prevent_base_ui_handler();
        assert!(
            reader.base_ui_handler_prevented(),
            "the mark set through one handle is visible through a clone"
        );
    }

    // Pins that the underlying event stays reachable — the augmentation is an intersection
    // (`E & { … }`, `:23`), so `E`'s own members are never lost.
    #[test]
    fn the_underlying_event_stays_reachable() {
        let event = BaseUIEvent::new(FakeEvent("click"));
        assert_eq!(event.inner(), &FakeEvent("click"), "inner() exposes E");
        assert_eq!(
            event.into_inner(),
            FakeEvent("click"),
            "into_inner() returns E"
        );
    }

    // Pins the always-allowed ref slot (`:8-10`): `ref?` defaults to absent, and the slot is
    // the fillable `{ current: T | null }` shape the pipeline carries.
    #[test]
    fn the_ref_slot_starts_empty_and_is_fillable() {
        let props = HTMLProps::<FakeElement>::default();
        assert!(props.node_ref.is_none(), "ref? defaults to absent");

        let slot: Rc<Cell<Option<FakeElement>>> = Rc::new(Cell::new(None));
        let props = HTMLProps {
            node_ref: Some(Rc::clone(&slot)),
        };
        assert!(
            props.node_ref.is_some(),
            "the ref member is always allowed on the props vocabulary"
        );
        slot.set(Some(FakeElement("host")));
        assert_eq!(
            props.node_ref.as_ref().unwrap().take(),
            Some(FakeElement("host")),
            "the slot is fillable through the shared cell"
        );
    }

    // Pins the render-prop shape (`:18-21`): the callable receives (props, state) and
    // returns the element.
    #[test]
    fn the_render_fn_is_called_with_props_and_state() {
        let calls: Rc<Cell<Option<(u8, bool)>>> = Rc::new(Cell::new(None));
        let render: ComponentRenderFn<u8, bool, &'static str> = Rc::new({
            let calls = Rc::clone(&calls);
            move |props: u8, state: bool| {
                calls.set(Some((props, state)));
                "element"
            }
        });
        let element = render(3u8, true);
        assert_eq!(element, "element", "the callable returns the element");
        assert_eq!(
            calls.get(),
            Some((3u8, true)),
            "the callable receives (props, state)"
        );
    }

    #[derive(Clone, Debug, PartialEq)]
    struct FakeElement(&'static str);
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // Pins the flag contract under the browser target and that the augmentation composes
    // with a real DOM event: `E`'s own members (here `preventDefault`) stay reachable — the
    // intersection semantics of `:23`.
    #[wasm_bindgen_test]
    fn the_event_wrapper_composes_with_a_real_dom_event() {
        let event = web_sys::Event::new_with_event_init_dict(
            "base-ui",
            web_sys::EventInit::new().cancelable(true),
        )
        .unwrap();
        let wrapped = BaseUIEvent::new(event);
        assert!(!wrapped.base_ui_handler_prevented());
        assert_eq!(wrapped.inner().type_(), "base-ui");
        wrapped.prevent_base_ui_handler();
        assert!(wrapped.base_ui_handler_prevented());
        wrapped.inner().prevent_default();
        assert!(
            wrapped.into_inner().default_prevented(),
            "the underlying event's own members remain usable"
        );
    }

    // Pins the always-allowed ref slot against a real element (`:8-10`).
    #[wasm_bindgen_test]
    fn html_props_ref_slot_accepts_a_real_element() {
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("button")
            .unwrap();
        let props = HTMLProps::<web_sys::Element>::default();
        assert!(props.node_ref.is_none());
        let slot: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
        let props = HTMLProps {
            node_ref: Some(Rc::clone(&slot)),
        };
        slot.set(Some(element.clone()));
        assert_eq!(
            props.node_ref.as_ref().unwrap().take(),
            Some(element),
            "the slot carries the real element"
        );
    }
}
