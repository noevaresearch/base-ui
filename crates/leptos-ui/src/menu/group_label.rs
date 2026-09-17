//! Menu group label — `Menu.GroupLabel`, port of
//! `packages/react/src/menu/group-label/MenuGroupLabel.tsx`.
//!
//! WHAT THIS REPLACES. The previous `group-label.rs` was a facade, not a port: it rendered a
//! hardcoded `<div class="menu-group-label">`, fell back to the literal text `"Group Label"`
//! when it had no children (upstream renders what it is given and nothing else), accepted a
//! `render: Option<fn() -> HtmlElement>` prop that upstream spells `render` as an ELEMENT
//! (`BaseUIComponentProps`), typed `aria-hidden` as a plain `bool` defaulting to `true` (so
//! the documented `aria-hidden={undefined}` override — `MenuGroupLabel.test.tsx:63-80` —
//! could not be expressed at all), and exported two "hooks" that returned constants.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from
//! (`MenuGroupLabel.tsx`, 41 lines total):
//! - `id = useBaseUiId(idProp)` (`:23`) and the registration cycle (`:25-30`): the mount
//!   write `setLabelId(id)` and the returned cleanup
//!   `setLabelId((currentId) => currentId === id ? undefined : currentId)` — the ported
//!   [`use_registered_label_id`], whose two `LabelIdUpdate` arms ARE those two call shapes
//!   (`use_registered_label_id.rs`'s module docs). This is what makes an older label's
//!   unmount unable to clear a newer label's id, the ordering
//!   `MenuGroupLabel.test.tsx:183-219` asserts.
//! - the required group read (`:24`, `useMenuGroupRootContext`), which throws upstream's
//!   message outside `<Menu.Group>` / `<Menu.RadioGroup>`
//!   (`MenuGroupLabel.test.tsx:31-41`).
//! - the element props (`:33-37`): `id` and `'aria-hidden': true`, with the consumer's own
//!   bag landing after them so an explicit `aria-hidden={undefined}` removes the attribute
//!   (`:63-80`) and an explicit value wins verbatim.
//!
//! DEFERRED, with the reason: `render` (`:21`) and the forwarded ref (`:22`) — the
//! crate-wide items named in [`crate::menu::arrow`]'s docs.

use leptos::prelude::*;
use leptos_ui_internals::use_registered_label_id::use_registered_label_id;
// The 0.2 `reactive_graph` surface the internals crate's signatures use — `leptos::prelude`
// re-exports reactive_graph 0.1, so a `Signal` built from the prelude would not satisfy
// `use_registered_label_id`'s `Get` bound (`use_registered_label_id.rs:72`).
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::Signal as RgSignal;

use crate::menu::group::menu_group_context;

/// `MenuGroupLabel.tsx:33-37` — the label element's attribute plan after the consumer's
/// `elementProps` bag has had its say.
///
/// `aria_hidden` models upstream's three states exactly (the combobox group label's
/// convention, `combobox/group_wiring.rs:117-131`): `None` = the consumer said nothing, so
/// the part's own `aria-hidden: true` stands; `Some(None)` = the consumer passed
/// `aria-hidden={undefined}`, which removes the attribute; `Some(Some(value))` = the
/// consumer's value wins verbatim.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuGroupLabelAttrs {
    /// The resolved `id` (`useBaseUiId`'s result).
    pub id: String,
    /// The effective `aria-hidden` after the override resolution.
    pub aria_hidden: Option<bool>,
}

/// Resolves the label element's `aria-hidden` (`MenuGroupLabel.tsx:36` with the override
/// rule at `:63-80`).
pub fn menu_group_label_aria_hidden(aria_hidden_override: Option<Option<bool>>) -> Option<bool> {
    match aria_hidden_override {
        // Not specified: the part's own `aria-hidden: true` stands.
        None => Some(true),
        // Consumer value wins verbatim — including the explicit `aria-hidden={undefined}`
        // that removes the attribute.
        Some(explicit) => explicit,
    }
}

/// The label element's attribute plan (`MenuGroupLabel.tsx:33-37`). The `id` arrives already
/// resolved by [`use_registered_label_id`] (the `useBaseUiId` override rule: a provided id
/// wins verbatim, otherwise the `base-ui-` prefix).
pub fn menu_group_label_attrs(
    id: String,
    aria_hidden_override: Option<Option<bool>>,
) -> MenuGroupLabelAttrs {
    MenuGroupLabelAttrs {
        id,
        aria_hidden: menu_group_label_aria_hidden(aria_hidden_override),
    }
}

/// `Menu.GroupLabel` — upstream's `MenuGroupLabel` (`MenuGroupLabel.tsx:20-38`).
///
/// An accessible label that registers its id with its enclosing group's `aria-labelledby`
/// for as long as it is mounted, and renders `aria-hidden` by default.
#[component]
pub fn GroupLabel(
    /// `id` (`:21`) — the `useBaseUiId` override.
    #[prop(optional)]
    id: Option<String>,
    /// `aria-hidden` (`:36`, default `true`; `Some(None)` is upstream's
    /// `aria-hidden={undefined}`, which removes the attribute).
    #[prop(optional)]
    aria_hidden: Option<Option<bool>>,
    /// `className` (`:21`).
    #[prop(optional, into)]
    class: Option<String>,
    /// `style` (`:21`).
    #[prop(default = Vec::new())]
    style: Vec<(String, String)>,
    /// The label's content.
    children: Children,
) -> impl IntoView {
    // `const setLabelId = useMenuGroupRootContext()` (`:24`) — the required read, which
    // throws outside both group kinds (`MenuGroupContext.ts:8-17`).
    let group = menu_group_context();

    // `const id = useBaseUiId(idProp)` (`:23`) plus the registration effect (`:25-30`).
    let id_signal = use_registered_label_id(
        RgSignal::derive_local(move || id.clone()),
        group.set_label_id.clone(),
    );

    let attrs_aria_hidden = menu_group_label_aria_hidden(aria_hidden);

    let style_attribute = style
        .into_iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<String>();

    view! {
        <div
            id=move || id_signal.get()
            aria-hidden=move || attrs_aria_hidden.map(|value| value.to_string())
            class=class
            style=style_attribute
        >
            {children()}
        </div>
    }
}
