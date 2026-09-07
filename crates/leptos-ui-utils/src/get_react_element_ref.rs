//! Port of `packages/utils/src/getReactElementRef.ts` (Base UI Phase A util).
//!
//! Upstream extracts the `ref` from a React element
//! (`packages/utils/src/getReactElementRef.ts:7-16`): a non-element input returns `null`
//! (`packages/utils/src/getReactElementRef.test.tsx:6-12`), a `React.Fragment` element returns
//! `null` even though it has element children (`packages/utils/src/getReactElementRef.test.tsx:22-31`),
//! a host element created without a `ref` prop returns `null`
//! (`packages/utils/src/getReactElementRef.test.tsx:33-37`), and a ref-carrying element returns
//! its exact ref object by strict identity (`packages/utils/src/getReactElementRef.test.tsx:15-19`).
//! Its consumers merge that ref into the rendered element's ref chain so a `render` element's
//! own ref survives instead of being clobbered
//! (`packages/react/src/internals/useRenderElement.tsx:100`,
//! `packages/react/src/internals/useRenderElement.tsx:102`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The dynamic "is this a React element at all?" guard (`React.isValidElement`,
//!   `packages/utils/src/getReactElementRef.ts:8`) has no Rust counterpart: a value that is not
//!   an element is unrepresentable as a [`ReactElement`], so the guard collapses into the type
//!   system. The [`Option`] parameter models upstream's `unknown` input, where `None` stands for
//!   "no element" (upstream's `undefined`, `packages/utils/src/getReactElementRef.test.tsx:8`);
//!   the other non-element inputs upstream passes (`false`, `1`, an array of elements,
//!   `packages/utils/src/getReactElementRef.test.tsx:6-12`) simply cannot be constructed as a
//!   [`ReactElement`], so their `null` result is enforced by the type system instead of a
//!   runtime branch.
//! - The React-version-conditional ref location — `props.ref` on React 19+, `element.ref` before
//!   (`packages/utils/src/getReactElementRef.ts:15`) — is a React-runtime implementation detail
//!   with no Leptos counterpart: N/A. [`get_react_element_ref`] reads the ref from wherever the
//!   port's element value carries it, the same value both upstream branches produce.
//! - The `?? null` undefined normalization (`packages/utils/src/getReactElementRef.ts:15`) maps
//!   to `Option`'s `None`. Upstream returns the ref by object identity (asserted with `toBe`,
//!   `packages/utils/src/getReactElementRef.test.tsx:19`); the port returns it by reference, so
//!   the caller can compare pointers for the same strictness.
//! - The ref handle is generic ([`R`]) rather than `React.Ref<unknown>`: the port's concrete ref
//!   type is chosen by callers (per `specs/architecture.md`, "Refs / DOM access", components use
//!   `NodeRef<T>`), and this crate stays framework-agnostic like the rest of the workspace. The
//!   generic parameter also covers upstream's untested ref shapes (callback refs, `ref={null}`,
//!   non-host element kinds — all UNVERIFIED upstream, see
//!   `specs/utils/getReactElementRef.md` "Unproven behaviors"), which downstream items pin per
//!   use site as the render-composition primitive is built.

/// The port's model of a React element, restricted to the distinction the ref-extraction
/// contract observes: fragments are never ref-carrying
/// (`packages/utils/src/getReactElementRef.test.tsx:22-31`), while every other element kind may
/// carry a ref (`packages/utils/src/getReactElementRef.test.tsx:15-19`).
///
/// Values that are not elements at all (`packages/utils/src/getReactElementRef.test.tsx:6-12`)
/// are not representable here — they are modeled by the [`Option`] in
/// [`get_react_element_ref`]'s parameter instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReactElement<R> {
    /// A `React.Fragment` element: not a ref-carrying element, even with element children
    /// (`packages/utils/src/getReactElementRef.test.tsx:22-31`). Upstream's fragment input in
    /// that test carries children; the port's variant carries no payload at all because the
    /// extraction contract observes nothing else about it.
    Fragment,
    /// Any element kind that may carry a ref (upstream exercises a host element,
    /// `packages/utils/src/getReactElementRef.test.tsx:17`; forwardRef components, class
    /// components, and other kinds are UNVERIFIED upstream but indistinguishable to the
    /// extraction contract, which only reads the ref slot
    /// `packages/utils/src/getReactElementRef.ts:12-15`). The field is `None` for elements
    /// created without a `ref` (`packages/utils/src/getReactElementRef.test.tsx:33-37`) and for
    /// upstream's `ref={null}`/`ref={undefined}` props (UNVERIFIED — inferred, no test asserts
    /// them).
    Element {
        /// The element's own ref, when it carries one
        /// (`packages/utils/src/getReactElementRef.test.tsx:17`).
        react_ref: Option<R>,
    },
}

/// Extracts the `ref` from an element — the port of upstream `getReactElementRef`
/// (`packages/utils/src/getReactElementRef.ts:7-16`), keeping its JSDoc contract: handle the
/// different React versions by reading the ref wherever the port's element value carries it.
///
/// Returns `None` for a missing element (upstream's non-element inputs,
/// `packages/utils/src/getReactElementRef.test.tsx:6-12`), for a fragment
/// (`packages/utils/src/getReactElementRef.test.tsx:22-31`), and for an element without a ref
/// (`packages/utils/src/getReactElementRef.test.tsx:33-37`); otherwise returns the element's own
/// ref by reference, preserving the strict identity upstream asserts with `toBe`
/// (`packages/utils/src/getReactElementRef.test.tsx:19`).
pub fn get_react_element_ref<R>(element: Option<&ReactElement<R>>) -> Option<&R> {
    match element {
        None => None,
        Some(ReactElement::Fragment) => None,
        Some(ReactElement::Element { react_ref }) => react_ref.as_ref(),
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    // Mirrors `packages/utils/src/getReactElementRef.test.tsx:6-9`: no element → `null`.
    // Upstream's `undefined` input is the port's `None`; its `false`, `1`, and
    // `packages/utils/src/getReactElementRef.test.tsx:11-12` array-of-elements inputs are not
    // `ReactElement` values in Rust, so the same `null` result is enforced by the type system
    // instead of a runtime branch (see module docs).
    #[test]
    fn returns_none_when_not_provided_a_react_element() {
        assert!(get_react_element_ref::<Rc<()>>(None).is_none());
    }

    // Mirrors `packages/utils/src/getReactElementRef.test.tsx:15-19`: the returned value is the
    // exact ref object supplied via the `ref` prop, compared by strict identity (`toBe`).
    #[test]
    fn returns_the_ref_of_a_react_element() {
        let handle: Rc<()> = Rc::new(());
        let element = ReactElement::Element {
            react_ref: Some(handle.clone()),
        };

        let extracted = get_react_element_ref(Some(&element)).unwrap();

        assert!(
            Rc::ptr_eq(extracted, &handle),
            "the ref is returned by strict identity"
        );
    }

    // Mirrors `packages/utils/src/getReactElementRef.test.tsx:22-31`: a fragment returns `null`
    // even though it has element children — fragments are not ref-carrying elements.
    #[test]
    fn returns_none_for_a_fragment() {
        let element: ReactElement<Rc<()>> = ReactElement::Fragment;

        assert!(get_react_element_ref(Some(&element)).is_none());
    }

    // Mirrors `packages/utils/src/getReactElementRef.test.tsx:33-37`: an element created
    // without a `ref` prop returns `null`.
    #[test]
    fn returns_none_for_an_element_without_a_ref() {
        let element: ReactElement<Rc<()>> = ReactElement::Element { react_ref: None };

        assert!(get_react_element_ref(Some(&element)).is_none());
    }

    // Upstream's return type is `React.Ref<unknown>`
    // (`packages/utils/src/getReactElementRef.ts:7`) — any ref shape flows through; only an
    // object ref is exercised upstream (callback refs are UNVERIFIED,
    // `specs/utils/getReactElementRef.md` "Unproven behaviors"). The generic parameter keeps
    // every ref shape representable; pinned with a second handle kind.
    #[test]
    fn works_with_any_ref_handle_shape() {
        #[derive(Debug, PartialEq)]
        enum RefHandle {
            Object(u8),
            Callback,
        }

        let object_element = ReactElement::Element {
            react_ref: Some(RefHandle::Object(1)),
        };
        let callback_element = ReactElement::Element {
            react_ref: Some(RefHandle::Callback),
        };

        assert_eq!(
            get_react_element_ref(Some(&object_element)),
            Some(&RefHandle::Object(1))
        );
        assert_eq!(
            get_react_element_ref(Some(&callback_element)),
            Some(&RefHandle::Callback)
        );
    }
}
