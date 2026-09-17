//! `Menu.Separator` — the SEPARATOR unit's component re-exported into the menu namespace
//! (`packages/react/src/menu/index.parts.ts:19`: `export { Separator } from
//! '../separator/Separator'`).
//!
//! Upstream's menu package defines no separator of its own, and
//! `specs/library/menu/implementation.md` records the consequence verbatim: "`Separator` is
//! re-exported as `Menu.Separator` (`packages/react/src/menu/index.parts.ts:19`), a docs-surface
//! dependency on `library: separator`". The part therefore owns NO menu behaviour: it renders no
//! menu context read, registers with no composite list, and takes no menu part's props — it is
//! the shared separator element, which is why it cannot appear in the mined menu suites except
//! as a plain child of a popup.
//!
//! The delegation target is [`crate::separator::separator_element`] — the same element builder
//! the separator unit's own tests exercise, rather than a re-derivation here (the otp_field
//! precedent, `crates/leptos-ui/src/otp_field.rs:1691-1697`, which carries the identical
//! upstream re-export at `packages/react/src/otp-field/index.parts.ts:3`). The `#[component]`
//! wrapper exists because the crate exposes components for `view!` markup and the separator unit
//! itself ships only the description/element split (implementation.md, "DOM/portal strategy and
//! why" — the unit's element path).
//!
//! DEFERRED, the crate-wide item rather than this part's own gap: the `render` prop reaches the
//! element description (below), but the view path materializes `custom(rendered.tag)` — a
//! replacement tag is honoured while a replacement *component* is not
//! (`library: the view paths drop render's element form`).

use leptos::children::Children;
use leptos::html::{Custom, custom};
use leptos::prelude::*;
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderProp, StyleSource, UseRenderElementComponentProps,
};
use leptos_ui_utils::use_merged_refs::RefCallback;
use send_wrapper::SendWrapper;
use web_sys::Element;

use crate::separator::{
    SEPARATOR_ORIENTATION_HORIZONTAL, SeparatorOrientation, SeparatorProps as SharedSeparatorProps,
};

/// The shared separator's `className`/`render`/`style` triple in the engine's vocabulary —
/// upstream's `componentProps` projection (`packages/react/src/separator/Separator.tsx:16`),
/// spelled the way the separator unit's own element path consumes it.
fn render_class_style(
    class: Option<String>,
    render: Option<RenderProp>,
    style: Vec<(String, String)>,
) -> UseRenderElementComponentProps {
    UseRenderElementComponentProps {
        class_name: class.map(ClassNameSource::Static),
        render,
        style: (!style.is_empty()).then(|| StyleSource::Static(style)),
    }
}

/// `Menu.Separator` — upstream's `<Menu.Separator>`, which IS the separator unit's component
/// (`index.parts.ts:19`). Renders a `<div role="separator">` with `aria-orientation` (or the
/// consumer's replacement tag/attributes); see the module docs for why it carries no menu
/// behaviour.
#[component]
pub fn Separator(
    /// `orientation` (`Separator.tsx:34`) — the shared separator's prop, default
    /// `'horizontal'` (`:16`).
    #[prop(optional)]
    orientation: Option<SeparatorOrientation>,
    /// `render` — the element-replacement union.
    #[prop(optional)]
    render: Option<RenderProp>,
    /// `className`.
    #[prop(optional, into)]
    class: Option<String>,
    /// `style` — ordered declarations.
    #[prop(default = Vec::new())]
    style: Vec<(String, String)>,
    /// The `...elementProps` rest — plain attributes (upstream's last bag, `Separator.tsx:23`,
    /// so these override the part's own members on key conflict).
    #[prop(default = Vec::new())]
    element_attributes: Vec<(String, String)>,
    /// The forwarded `ref`.
    #[prop(optional)]
    ref_callback: Option<RefCallback<Element>>,
    /// The separator's own content, if any.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let rendered = crate::separator::separator_element(SharedSeparatorProps {
        orientation: orientation.unwrap_or(SEPARATOR_ORIENTATION_HORIZONTAL),
        render_class_style: render_class_style(class, render, style),
        element_attributes,
        ref_callback,
    })
    // The shared separator's `useRenderElement` call has no `enabled` gate
    // (`separator_element`'s own contract), so this arm is unreachable; the empty view keeps the
    // return type total (the `fieldset_root_view` shape).
    .unwrap_or_else(|| unreachable!("the shared separator always renders"));

    let tag = rendered.tag.clone();
    let children_view: Option<AnyView> = children.map(|children| children());
    let node_ref = NodeRef::<Custom<String>>::new();
    let rendered = SendWrapper::new(rendered);
    // The React commit: the merged bag onto the retained node, with the defined → `undefined`
    // diff — the `fieldset::root::write_element_bag` / checkbox-group mount-writer shape.
    Effect::new(move |_| {
        // The prelude's own `Get` is spelled out because this crate imports the workspace's
        // `reactive_graph` traits elsewhere, which shadow it — and the prelude's is the one
        // `NodeRef` implements (tachys's reactive-graph).
        let Some(node) = ::leptos::prelude::Get::get(&node_ref) else {
            return;
        };
        crate::fieldset::root::write_element_bag(&node, &rendered);
    });

    custom(tag).node_ref(node_ref).child(children_view)
}
