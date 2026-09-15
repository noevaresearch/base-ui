//! The Fieldset legend — port of `FieldsetLegend`
//! (`packages/react/src/fieldset/legend/FieldsetLegend.tsx`).
//!
//! Upstream is one component:
//!
//! 1. the destructuring `{ render, className, style, id: idProp, ...elementProps }` (`:18`),
//! 2. `const { disabled, setLegendId } = useFieldsetRootContext()` (`:20`) — the REQUIRED
//!    overload, which throws when there is no root ancestor
//!    (`root/FieldsetRootContext.ts:14-21`; behavior.md *Public API surface* pins the message),
//! 3. `const id = useRegisteredLabelId(idProp, setLegendId)` (`:22`) — the id resolution
//!    plus the registration whose lifecycle IS behavior.md's *Association lifecycle*,
//! 4. the one-member state record `{ disabled }` (`:24-26`),
//! 5. `useRenderElement('div', componentProps, { state, ref, props: [{ id }, elementProps] })`
//!    (`:28-32`).
//!
//! ## Why this is a `<div>` and not a `<legend>`
//!
//! Deliberately the element implementation.md's "DOM/portal strategy" section explains:
//! a native `<legend>` is laid out into the fieldset border rather than normal flow,
//! so Base UI keeps a plain block element and establishes the association
//! programmatically through the ROOT's `aria-labelledby` (`FieldsetRoot.tsx:39`). The
//! cost is the JS-managed lifecycle this module ports, and it is why the conformance
//! suite's `refInstanceof` is `HTMLDivElement` (`FieldsetLegend.test.tsx:10-15`).
//!
//! ## The registration and its runtime
//!
//! [`fieldset_legend_view`] calls the REAL ported hook
//! (`leptos-ui-internals/src/use_registered_label_id.rs`) rather than re-deriving it,
//! because the hook owns three behaviors a re-derivation silently loses: the
//! `base-ui-`-prefixed generated id (`useBaseUiId`), the stale-cleanup guard
//! (`setLabelId((currentId) => (currentId === id ? undefined : currentId))`,
//! `useRegisteredLabelId.ts:14-16` — the *multiple legends* case behavior.md marks
//! UNVERIFIED), and the re-registration cycle on an id change.
//!
//! That hook is an rg-0.2 citizen: its browser binding is `RenderEffect` and its
//! unmount cleanup is rg-0.2 `on_cleanup`, which is a silent no-op outside an rg-0.2
//! owner (its module docs, "Rust adaptations"). So the view opens a dedicated rg-0.2
//! [`Owner`] around the hook call and disposes it from this view's LEPTOS cleanup —
//! attaching and disposing in the same runtime, which is the rule the checkbox
//! iteration paid for. Disposal is what fires the hook's `ClearIfCurrent` dispatch, so
//! unmounting the legend withdraws the association from the root (behavior.md
//! *Association lifecycle*).
//!
//! The resulting id is read once and rendered: the port's props are static per body
//! run (the meter convention), so an id CHANGE is the caller rebuilding the legend —
//! the fresh call re-runs the hook's cycle in a fresh owner.

use std::rc::Rc;

use leptos::children::Children;
use leptos::html::{custom, Custom};
use leptos::prelude::*;
use leptos::tachys::html::element::ElementChild;
use leptos::tachys::html::node_ref::NodeRefAttribute;
use send_wrapper::SendWrapper;

use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::use_registered_label_id::{
    use_registered_label_id, LabelIdSetter, LabelIdUpdate,
};
use leptos_ui_internals::use_render_element::{
    static_attr, use_render_element, ClassNameSource, RenderElementProps, RenderProp,
    RenderedElement, StyleSource, UseRenderElementComponentProps, UseRenderElementParams,
};
use leptos_ui_utils::use_merged_refs::{InputRef, RefCallback};
use reactive_graph::owner::Owner;
use reactive_graph::wrappers::read::Signal as RgSignal;
use web_sys::Element;

use super::root::{write_element_bag, FieldsetRootContext};

/// The `FieldsetLegendState` record (`FieldsetLegend.tsx:37-42`): one member, taken
/// from the context, converted to `data-disabled=""` by the engine's default walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldsetLegendState {
    /// `disabled` (`:39`) — the root's effective value.
    pub disabled: bool,
}

impl FieldsetLegendState {
    /// The `serde_json` state map the engine's default walk consumes.
    pub fn to_state_map(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "disabled".to_string(),
            serde_json::Value::Bool(self.disabled),
        );
        map
    }
}

/// The builder parameters — upstream's destructuring plus the RESOLVED id.
pub struct FieldsetLegendElementProps {
    /// `id` (`:31`) — the resolved id from `useRegisteredLabelId` (the override
    /// verbatim, else a `base-ui-`-prefixed generated one). It is upstream's first bag
    /// (`[{ id }, elementProps]`), so a user-supplied `id` in the rest still wins on
    /// key conflict — the same value `useRegisteredLabelId` already resolved.
    pub id: String,
    /// `disabled` (`:24-26`) — from the context.
    pub disabled: bool,
    /// `className`/`style`/`render` (`:18`) through the engine's vocabulary.
    pub render_class_style: UseRenderElementComponentProps,
    /// The `...elementProps` rest (`:18`) — the last bag.
    pub element_attributes: Vec<(String, String)>,
    /// The forwarded `ref` (`:15,:30`) — attaches to the `<div>`
    /// (`refInstanceof: window.HTMLDivElement`).
    pub ref_callback: Option<RefCallback<Element>>,
}

impl Default for FieldsetLegendElementProps {
    fn default() -> Self {
        Self {
            id: String::new(),
            disabled: false,
            render_class_style: UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            ref_callback: None,
        }
    }
}

/// Builds the legend's `<div>` element description — upstream's `FieldsetLegend` body up
/// to and including `useRenderElement` (`FieldsetLegend.tsx:28-32`). Must be called
/// inside a reactive owner (the ref fork registers there).
pub fn fieldset_legend_element(props: FieldsetLegendElementProps) -> Option<RenderedElement> {
    let FieldsetLegendElementProps {
        id,
        disabled,
        render_class_style,
        element_attributes,
        ref_callback,
    } = props;

    let state_map = FieldsetLegendState { disabled }.to_state_map();

    // The intrinsic bag (`:31`'s first element): `{ id }` — one plain attribute.
    let mut intrinsic = RenderElementProps::default();
    intrinsic
        .handlers
        .attributes
        .push(("id".to_string(), static_attr(id)));

    // The `...elementProps` rest (`:18`) — the last bag, later-wins.
    let mut element_bag = RenderElementProps::default();
    for (name, value) in &element_attributes {
        element_bag
            .handlers
            .attributes
            .push((name.clone(), static_attr(value.clone())));
    }

    let refs: Vec<InputRef<Element>> = match ref_callback {
        Some(callback) => vec![InputRef::Callback(callback)],
        None => Vec::new(),
    };

    // `useRenderElement('div', componentProps, …)` (`:28`) — a plain `<div>`
    // (module docs; behavior.md "DOM structure").
    use_render_element(
        "div",
        render_class_style,
        UseRenderElementParams {
            enabled: true,
            state: &state_map,
            refs,
            props: vec![
                PropsSource::Static(intrinsic),
                PropsSource::Static(element_bag),
            ],
            state_attributes_mapping: None,
        },
    )
}

/// The view-layer props (`FieldsetLegendViewProps` — the house naming: a
/// `#[component]`-generated `FieldsetLegendProps` exists for the public component, and
/// the element-level struct above is spelled `FieldsetLegendElementProps` so neither
/// name collides — the `FieldRootViewProps`/`FieldControlViewProps` convention).
pub struct FieldsetLegendViewProps {
    /// `id` (`FieldsetLegend.tsx:18`) — the `idProp` override; `None` generates one.
    pub id: Option<String>,
    /// The user's `className` (`:18`).
    pub class: Option<String>,
    /// The user's `style` (`:18`).
    pub style: Vec<(String, String)>,
    /// `render` (`:18`).
    pub render: Option<RenderProp>,
    /// The `...elementProps` rest (`:18`).
    pub element_attributes: Vec<(String, String)>,
    /// The forwarded `ref` (`:15`).
    pub ref_callback: Option<RefCallback<Element>>,
    /// The legend's content — upstream's `elementProps.children`.
    pub children: Option<Children>,
}

impl Default for FieldsetLegendViewProps {
    fn default() -> Self {
        Self {
            id: None,
            class: None,
            style: Vec::new(),
            render: None,
            element_attributes: Vec::new(),
            ref_callback: None,
            children: None,
        }
    }
}

/// The legend view — upstream's `FieldsetLegend` body (`FieldsetLegend.tsx:18-34`).
/// Must be called inside a reactive owner (a component body).
///
/// # Panics
///
/// When there is no [`FieldsetRootContext`] in scope — upstream's thrown error,
/// message verbatim (`root/FieldsetRootContext.ts:16-20`; behavior.md *Public API surface*).
pub fn fieldset_legend_view(props: FieldsetLegendViewProps) -> impl IntoView {
    let FieldsetLegendViewProps {
        id: id_prop,
        class,
        style,
        render,
        element_attributes,
        ref_callback,
        children,
    } = props;

    // `useFieldsetRootContext()` (`:20`) — the required overload (module docs).
    let context = use_context::<FieldsetRootContext>().expect(
        "Base UI: FieldsetRootContext is missing. Fieldset parts must be placed within \
         <Fieldset.Root>.",
    );

    // `setLegendId` as `useRegisteredLabelId` consumes it: the `SetStateAction` union
    // collapsed into the ported hook's `LabelIdUpdate` (its module docs) — `Set` writes
    // through, and the cleanup's guarded `ClearIfCurrent` clears the registration only
    // while this legend is still the registered one. The guard's read is a leptos read
    // (the context signal is leptos-side), which is the single-writer rule that keeps
    // the root's `aria-labelledby` live.
    let set_legend_id: LabelIdSetter = Rc::new(move |update| match update {
        LabelIdUpdate::Set(next) => context.set_legend_id.set(next),
        LabelIdUpdate::ClearIfCurrent(id) => {
            if leptos::prelude::GetUntracked::get_untracked(&context.legend_id) == Some(id) {
                context.set_legend_id.set(None);
            }
        }
    });

    // `const id = useRegisteredLabelId(idProp, setLegendId)` (`:22`) — the real ported
    // hook, inside its own rg-0.2 owner (module docs: its layout effect and cleanup are
    // rg-0.2, and an rg-0.2 `on_cleanup` outside an rg-0.2 owner is a silent no-op).
    let render_owner = Owner::new();
    let id_signal = render_owner.with(|| {
        use_registered_label_id(
            RgSignal::derive_local(move || id_prop.clone()),
            set_legend_id,
        )
    });
    let resolved_id: String = reactive_graph::traits::GetUntracked::get_untracked(&id_signal);

    // The registration's lifetime is this view's: leptos disposes the view, the rg-0.2
    // owner is cleaned up, the hook's unmount cleanup dispatches `ClearIfCurrent`, and
    // the root's `aria-labelledby` disappears (`FieldsetLegend.test.tsx:59-67`).
    let render_owner = SendWrapper::new(render_owner);
    on_cleanup(move || {
        render_owner.cleanup();
    });

    // `useRenderElement('div', componentProps, { state, ref, props: [{ id }, elementProps] })`
    // (`:28-32`).
    let rendered = fieldset_legend_element(FieldsetLegendElementProps {
        id: resolved_id,
        disabled: context.disabled,
        render_class_style: UseRenderElementComponentProps {
            class_name: class.map(ClassNameSource::Static),
            render,
            style: (!style.is_empty()).then(|| StyleSource::Static(style)),
        },
        element_attributes,
        ref_callback: None,
    });

    let Some(rendered) = rendered else {
        return ().into_any();
    };

    let tag = rendered.tag.clone();
    let children_view: Option<AnyView> = children.map(|children| children());
    let node_ref = NodeRef::<Custom<String>>::new();

    let rendered = SendWrapper::new(rendered);
    let ref_callback = SendWrapper::new(ref_callback);
    // The commit — the merged bag (`id`, the state walk's `data-disabled`, class/style,
    // the rest) onto the retained node, then the forwarded ref (React's
    // ref-after-children order). An id is static per build, so this effect's first run
    // is the whole story; it stays an effect because a node ref is only resolved after
    // mount (the checkbox-group mount-writer timing).
    Effect::new(move |_| {
        let Some(node) = node_ref.get() else {
            return;
        };
        write_element_bag(&node, &rendered);
        if let Some(callback) = ref_callback.as_ref().as_ref() {
            let element: Element = node.clone().into();
            if let Some(cleanup) = callback(Some(&element)) {
                std::mem::forget(cleanup);
            }
        }
    });

    custom(tag)
        .node_ref(node_ref)
        .child(children_view)
        .into_any()
}

/// `Fieldset.Legend` — the `#[component]` wrapper over [`fieldset_legend_view`].
#[component]
pub fn FieldsetLegend(
    /// `id` (`FieldsetLegend.tsx:18`).
    #[prop(default = None, optional)]
    id: Option<String>,
    /// The user's `className` (`:18`).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// Upstream's `...elementProps` rest (`:18`) — the port's explicit attribute list.
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The legend's content.
    children: Children,
) -> impl IntoView {
    fieldset_legend_view(FieldsetLegendViewProps {
        id,
        class,
        element_attributes,
        children: Some(children),
        ..Default::default()
    })
}
