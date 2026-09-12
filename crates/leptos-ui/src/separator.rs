//! Port of the Base UI Separator — the `library: separator` TODO item
//! (`specs/library/separator/behavior.md`, `specs/library/separator/implementation.md`).
//!
//! Upstream's structural facts this port follows (implementation.md):
//!
//! - **The entire component is a synchronous prop-to-DOM projection** (implementation.md
//!   header): no state machine, no effects, no context, no portal
//!   (`packages/react/src/separator/Separator.tsx:12-27` — the body is one destructuring,
//!   one state literal, and the single `useRenderElement` call). The port is a
//!   configuration of the already-ported
//!   [`leptos_ui_internals::use_render_element`] engine — the button facade
//!   precedent (the unit's own ~40 lines contribute prop narrowing and bag order,
//!   everything else is the shared machinery; unlike Button there is no `useButton`
//!   and no internal bag beyond the intrinsic attributes).
//! - **`orientation` defaults to `'horizontal'` at the destructuring**
//!   (`packages/react/src/separator/Separator.tsx:16`) — the source-level answer to
//!   behavior.md's UNVERIFIED default: `aria-orientation` and `data-orientation` are
//!   always fully rendered, never omitted.
//! - **The only state is `{ orientation }`** (`Separator.tsx:18`), and Separator passes
//!   **no** `stateAttributesMapping`, so the DEFAULT state walk applies
//!   (implementation.md:43-47): the truthy non-boolean string becomes
//!   `data-orientation = String(value)` through the ported
//!   [`get_state_attributes_props`] default arm — unlike progress's custom
//!   status mapping, nothing here is hand-mapped.
//! - **Bag order is the precedence contract** (implementation.md:32-34, 97-109): the
//!   intrinsic `[{ role: 'separator', 'aria-orientation': orientation }, elementProps]`
//!   bags merge left-to-right with later-wins scalars (`mergeProps.ts`), so the
//!   library attributes are always present but a user-supplied `role`,
//!   `aria-orientation`, or `data-orientation` in `elementProps` (the rightmost bag)
//!   silently overrides them — implementation.md untested item 4, pinned by the
//!   wasm override test.
//! - **`className` and `style` merge rather than overwrite** (implementation.md:104-106)
//!   across the component prop, the render element's own, and the render function's
//!   output — all handled inside the shared engine (the `mergeClassNames` /
//!   `mergeStyles` ports), which the render-prop merging test pins end-to-end.
//! - **The `render` prop** (`implementation.md:65-73`): a function is called as
//!   `render(props, state)` and owns the element wholesale; a JSX element is cloned
//!   with the merged props (its plain attributes win, its class prepends). The port's
//!   [`RenderProp`] union is the same shape.
//! - **The ref attaches to the root** (behavior.md "DOM structure": `refInstanceof:
//!   window.HTMLDivElement`), forked through the engine's `useMergedRefsN` over
//!   [bag ref, render-element ref, forwarded ref] — the forwarded ref rides the
//!   same fork slot button's does.
//! - The unit contributes **no event handlers** (behavior.md "Events": N/A), so the
//!   composed handler set is exactly the user's — none, in this port's vocabulary;
//!   user handlers ride `elementProps` upstream and have no separate slot here.
//!
//! ## Rust adaptations
//!
//! - `orientation` is an owned [`SeparatorOrientation`] string-newtype rather than the
//!   `Orientation` union (that internals type is not ported — nothing else consumes
//!   it yet): `'horizontal'` / `'vertical'`, stringified into the state map and the
//!   attributes exactly as the JS union's members serialize.
//! - The reforked-ref model: upstream forwards `ref` through props; the port carries
//!   it as an explicit [`RefCallback`] prop (the button facade's forked-ref
//!   convention) fired at materialization.
//! - `useRenderElement`'s dev-only guards (uppercase render-fn warning, invalid-element
//!   throw) belong to the hook, not this unit (implementation.md untested item 5);
//!   the engine's own suite covers them.

use std::rc::Rc;

use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::use_render_element::{
    use_render_element, static_attr, RenderElementProps, RenderProp, RenderedElement,
    UseRenderElementComponentProps, UseRenderElementParams,
};
use leptos_ui_utils::use_merged_refs::{InputRef, RefCallback};
use web_sys::Element;

/// The `Orientation` prop union (`packages/react/src/internals/types.ts:97`) as the
/// unit consumes it — `SeparatorProps.orientation` (`Separator.tsx:34`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeparatorOrientation(pub &'static str);

/// `'horizontal'` — the destructuring default (`Separator.tsx:16`).
pub const SEPARATOR_ORIENTATION_HORIZONTAL: SeparatorOrientation =
    SeparatorOrientation("horizontal");

/// `'vertical'`.
pub const SEPARATOR_ORIENTATION_VERTICAL: SeparatorOrientation = SeparatorOrientation("vertical");

impl SeparatorOrientation {
    /// The JSON string the state walk consumes (`Separator.tsx:18`'s state member;
    /// the default mapping stringifies it onto `data-orientation`).
    pub fn as_json_value(self) -> serde_json::Value {
        serde_json::Value::String(self.0.to_string())
    }
}

/// `SeparatorState` (`packages/react/src/separator/Separator.tsx:37-42`): the only
/// member is `orientation`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeparatorState {
    /// The `orientation` member (`:41`).
    pub orientation: &'static str,
}

impl SeparatorState {
    /// The `serde_json` state map `useRenderElement`'s DEFAULT state walk consumes —
    /// no `stateAttributesMapping` is passed (`Separator.tsx:20-24`), so the truthy
    /// string becomes `data-orientation = "horizontal"|"vertical"` through the
    /// generic arm (implementation.md:43-47).
    pub fn to_state_map(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "orientation".to_string(),
            serde_json::Value::String(self.orientation.to_string()),
        );
        map
    }
}

/// The Separator component props — upstream's destructured `Separator.Props`
/// (`packages/react/src/separator/Separator.tsx:16`, interface `:29-35`).
pub struct SeparatorProps {
    /// `orientation` (`:34` — upstream default `'horizontal'` via the destructuring).
    pub orientation: SeparatorOrientation,
    /// `className`/`style`/`render` (`:16`) — through the
    /// [`UseRenderElementComponentProps`] vocabulary.
    pub render_class_style: UseRenderElementComponentProps,
    /// The `...elementProps` rest (`:16`) — arbitrary DOM props spread onto the
    /// element, `(name, static value)`. Because `elementProps` is the LAST bag of
    /// the merge (`Separator.tsx:23`), these override the intrinsic
    /// `role`/`aria-orientation`/state-derived `data-orientation` on key conflict
    /// (implementation.md:101-103, untested item 4).
    pub element_attributes: Vec<(String, String)>,
    /// The forwarded `ref` (`:14`/`:22`) — attaches to the root (the default div,
    /// `refInstanceof: window.HTMLDivElement`, `Separator.test.tsx:11`).
    pub ref_callback: Option<RefCallback<Element>>,
}

impl Default for SeparatorProps {
    fn default() -> Self {
        Self {
            orientation: SEPARATOR_ORIENTATION_HORIZONTAL,
            render_class_style: UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            ref_callback: None,
        }
    }
}

/// Builds the Separator element description — upstream's `SeparatorComponent` body
/// (`packages/react/src/separator/Separator.tsx:12-27`) up to and including
/// `useRenderElement`, without materializing a DOM node (the description/element
/// split the view bridge materializes). Must be called inside a reactive owner
/// (the ref fork registers there).
///
/// Always returns `Some` — the unit's `useRenderElement` call has no `enabled` gate
/// (a static leaf with no conditional rendering).
pub fn separator_element(props: SeparatorProps) -> Option<RenderedElement> {
    let SeparatorProps {
        orientation,
        render_class_style,
        element_attributes,
        ref_callback,
    } = props;

    // `const state: SeparatorState = { orientation }` (`:18`) — the record the
    // default state walk turns into `data-orientation`.
    let state_map = SeparatorState {
        orientation: orientation.0,
    }
    .to_state_map();

    // The intrinsic bag (`:23`'s first element) — `role` and `aria-orientation`
    // as plain attributes. A separator has no behavior, so there are no handler
    // slots on this side of the merge.
    let mut intrinsic = RenderElementProps::default();
    intrinsic.handlers.attributes = vec![
        ("role".to_string(), static_attr("separator".to_string())),
        (
            "aria-orientation".to_string(),
            static_attr(orientation.0.to_string()),
        ),
    ];

    // The `...elementProps` rest (`:16`'s `...elementProps`) — the LAST bag, so its
    // plain attributes win the scalar merge (implementation.md:97-103).
    let mut element_bag = RenderElementProps::default();
    for (name, value) in &element_attributes {
        element_bag
            .handlers
            .attributes
            .push((name.clone(), static_attr(value.clone())));
    }

    // The props array `[intrinsic, elementProps]` (`:23`), merged left-to-right,
    // later-wins — the entire precedence contract (implementation.md:32-34).
    let props_bags = vec![
        PropsSource::Static(intrinsic),
        PropsSource::Static(element_bag),
    ];

    // The forwarded ref (`:22`'s `ref: forwardedRef`) — the engine forks it after
    // the bag/render-element slots (the button facade's convention; the wasm suite
    // observes the fork fire with the root div).
    let refs: Vec<InputRef<Element>> = match ref_callback {
        Some(callback) => vec![InputRef::Callback(callback)],
        None => Vec::new(),
    };

    // `useRenderElement('div', componentProps, …)` (`:20-24`) — the default tag is
    // a `div` (behavior.md "DOM structure": `refInstanceof: window.HTMLDivElement`);
    // no `stateAttributesMapping` (the DEFAULT walk, implementation.md:47).
    use_render_element(
        "div",
        render_class_style,
        UseRenderElementParams {
            enabled: true,
            state: &state_map,
            refs,
            props: props_bags,
            state_attributes_mapping: None,
        },
    )
}

// The render-prop vocabulary the port accepts — a convenience re-derivation so a
// caller can build a render function without reaching into the internals crate's
// closure types (the doc example in the tests uses this shape).
/// Builds a [`RenderProp::Function`] from a `(props, state) -> RenderedElement`
/// closure — upstream's `render: (props, state) => ReactElement` arm
/// (`useRenderElement.tsx:165-170`).
pub fn separator_render_fn(
    function: impl Fn(RenderElementProps, &serde_json::Map<String, serde_json::Value>) -> RenderedElement
        + 'static,
) -> RenderProp {
    RenderProp::Function(Rc::new(function))
}
