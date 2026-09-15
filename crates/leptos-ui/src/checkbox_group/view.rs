//! The `CheckboxGroup` composition root — upstream's
//! `<CheckboxGroup>{children}</CheckboxGroup>` (`CheckboxGroup.tsx:29-175`, the
//! `docs-content: components/checkbox-group` TODO item's PAIR-PORTABILITY GAP).
//!
//! `checkbox_group_element` (`super`) builds the group's *element description* — the
//! merged props bag, the field-validity state walk, and the provider seam — but it
//! returns a [`leptos_ui_internals::use_render_element::RenderedElement`] and takes no
//! children, while upstream's group is a PROVIDER wrapped around its subtree: every
//! demo on `docs/src/app/(docs)/react/components/checkbox-group/page.mdx` nests its
//! `Checkbox.Root` items inside the group element, and `CheckboxRoot.tsx:89` is the
//! context's sole consumer (`specs/library/checkbox-group/implementation.md`,
//! "Cross-component contract"). This module adds that missing surface without touching
//! the element path: the view builder below calls [`CheckboxGroupProps`]'s existing
//! builder and keeps every behavior (the value duality, the veto-wrapped `setValue`,
//! the parent engine, the Field/Form/Labelable wiring, the `fieldValidityMapping`
//! state record) exactly where it is.
//!
//! ## The two mechanics that make the composition real
//!
//! - **One owner chain.** Both `provide_checkbox_group_context` (inside
//!   `checkbox_group_element`) and the children's `use_checkbox_group_context()` reads
//!   are reactive-graph 0.2 context operations, so they must share an owner chain.
//!   `reactive_graph::owner::Owner::new()` "registers [the owner] as a child of the
//!   current `Owner`, if there is one" (`reactive_graph-0.2.14/src/owner.rs:147-190`),
//!   and `Owner::with` swaps only the rg thread-local (`:271-291`) — the leptos owner
//!   (and every leptos Effect children create) stays live. So the children are BUILT
//!   inside the window, exactly as `field_root_view`'s bridge body does
//!   (`crates/leptos-ui/src/field/field_root.rs:437-443`), and every part the subtree
//!   materializes (`checkbox_root_view`, whose own `Owner::new()` chains to this one)
//!   sees the group.
//! - **No attribute spread in `view!`.** The element's merged bag is replayed onto the
//!   real node by a mount effect — the writer the checkbox Root, the Indicator and the
//!   field Root all use. The bag's lazy attribute closures are resolved in one pass at
//!   build time (the group's own state record is already a build-time snapshot, and the
//!   element path materializes its node once too), so the effect carries plain data.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::children::Children;
use leptos::html;
use leptos::prelude::{
    Effect, ElementChild, Get, IntoView, NodeRef, NodeRefAttribute, view,
};
use reactive_graph::owner::LocalStorage;
use reactive_graph::wrappers::read::Signal;
use wasm_bindgen::JsCast;

use leptos_ui_internals::use_render_element::{
    ClassNameSource, StyleSource, UseRenderElementComponentProps,
};

use super::{CheckboxGroupProps, OnGroupValueChange, checkbox_group_element};

/// The view-layer props for the group root: upstream's `CheckboxGroupProps`
/// (`CheckboxGroup.tsx:31-43`) with the consumer's `children` — the two props the
/// element-only builder cannot take. `className`/`style` are the plain forms the other
/// view-layer props structs use (`CheckboxRootViewProps.class`/`style`), so a caller
/// does not have to reach for the internals crate's source unions.
pub struct CheckboxGroupViewProps {
    /// `value` (`:56-59`) — controlled; `None` while uncontrolled. The one-shot form;
    /// prefer [`CheckboxGroupViewProps::value_source`] when the page owns the state and
    /// the display must follow it (upstream's controlled parent/nested recipes).
    pub value: Option<Vec<String>>,
    /// The controlled value as a reactive read — a page's `useState` analog
    /// (`value={value}` + `onValueChange={setValue}` in the demos). A
    /// `Signal::derive(move || Some(page_value.get()))` over the page's own signal keeps
    /// the group's display, its children and the parent checkbox's tri-state live.
    pub value_source: Option<Signal<Option<Vec<String>>, LocalStorage>>,
    /// `defaultValue` (`:64-67`) — the uncontrolled seed.
    pub default_value: Option<Vec<String>>,
    /// `onValueChange` (`:68-71`).
    pub on_value_change: Option<OnGroupValueChange>,
    /// `allValues` (`:72-74`) — the parent-checkbox contract's full set.
    pub all_values: Option<Vec<String>>,
    /// `disabled` (`:75-78`, default `false`).
    pub disabled: bool,
    /// `id` (`:36`).
    pub id: Option<String>,
    /// `className` (`:37`).
    pub class: Option<String>,
    /// `style` (`:38`) — ordered declarations.
    pub style: Vec<(String, String)>,
    /// The `...elementProps` rest (`:43`) — static attributes.
    pub element_attributes: Vec<(String, String)>,
    /// The parts subtree — upstream's `<CheckboxGroup>{children}</CheckboxGroup>`.
    pub children: Option<Children>,
}

impl Default for CheckboxGroupViewProps {
    fn default() -> Self {
        Self {
            value: None,
            value_source: None,
            default_value: None,
            on_value_change: None,
            all_values: None,
            disabled: false,
            id: None,
            class: None,
            style: Vec::new(),
            element_attributes: Vec::new(),
            children: None,
        }
    }
}

/// The `style` record as a CSS declaration string (`Meter`'s/`Field.Root`'s writer
/// spelling).
fn style_declarations(style: &[(String, String)]) -> String {
    style
        .iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The group root view — upstream's `<CheckboxGroup>` element with its provider around
/// the consumer's subtree. Must be called inside a reactive owner (the leptos one a
/// component render establishes); the rg-0.2 window for the provider/children chain is
/// opened here.
pub fn checkbox_group_view(props: CheckboxGroupViewProps) -> impl IntoView {
    let CheckboxGroupViewProps {
        value,
        value_source,
        default_value,
        on_value_change,
        all_values,
        disabled,
        id,
        class,
        style,
        element_attributes,
        children,
    } = props;

    let bridge_owner = reactive_graph::owner::Owner::new();
    let view = bridge_owner.with(move || {
        // The element description — the same builder the element path uses, so the
        // provider seam, the veto-wrapped `setValue`, the parent engine and the
        // field-validity state record all stay in one place. `render` rides `None`:
        // the tag-replacing form is the description layer's (see
        // `ralph/logs/spec-discrepancies.md`); this view is the default `div`.
        let rendered = checkbox_group_element(CheckboxGroupProps {
            value,
            value_source,
            default_value,
            on_value_change,
            all_values,
            disabled,
            id,
            render_class_style: UseRenderElementComponentProps {
                class_name: class.map(ClassNameSource::Static),
                render: None,
                style: (!style.is_empty()).then(|| StyleSource::Static(style)),
            },
            element_attributes,
        });

        // The whole merged bag — the state walk's `data-*` attributes, the base bag
        // (`id`, `role="group"`, `aria-labelledby`), the consumer's `...elementProps`
        // and the labelable `aria-describedby` — resolved once, in merge order.
        let mut resolved: Vec<(String, Option<String>)> = Vec::new();
        if let Some(class) = &rendered.props.class {
            resolved.push(("class".to_string(), Some(class.clone())));
        }
        if !rendered.props.style.is_empty() {
            resolved.push((
                "style".to_string(),
                Some(style_declarations(&rendered.props.style)),
            ));
        }
        for (name, attribute) in &rendered.props.handlers.attributes {
            resolved.push((name.clone(), attribute()));
        }

        // The parts subtree: built HERE, inside the window, so the group context is in
        // scope for every part's reads (module docs, "One owner chain").
        let children_view = children.map(|children| children());

        let root_node: NodeRef<html::Div> = NodeRef::new();
        let managed: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        {
            let resolved = resolved.clone();
            Effect::new(move |_| {
                let Some(div) = root_node.get() else {
                    return;
                };
                let element: &web_sys::Element = div.unchecked_ref();

                let previous = managed.borrow().clone();
                let mut written: Vec<String> = Vec::new();
                for (name, value) in &resolved {
                    if let Some(value) = value {
                        let _ = element.set_attribute(name, value);
                        written.push(name.clone());
                    }
                }
                // A bag member that disappeared between runs is removed — the input
                // tolerates the attributes but a stale `data-*`/`aria-*` hook would
                // misreport the state to the stylesheet and to AT.
                for name in previous {
                    if !written.contains(&name) {
                        let _ = element.remove_attribute(&name);
                    }
                }
                *managed.borrow_mut() = written;
            });
        }

        view! {
            <div node_ref=root_node>{children_view}</div>
        }
    });
    // The bridge owner outlives the subtree (the `field_root_view` precedent).
    std::mem::forget(bridge_owner);
    view
}
