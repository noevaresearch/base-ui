//! The Fieldset root — port of `FieldsetRoot` and its context
//! (`packages/react/src/fieldset/root/FieldsetRoot.tsx`,
//! `packages/react/src/fieldset/root/FieldsetRootContext.ts`).
//!
//! Upstream is one component plus one context:
//!
//! 1. the destructuring with the `disabled = false` default (`FieldsetRoot.tsx:17-23`),
//! 2. the optional ancestor read + the one-line effective-disabled OR
//!    (`:27-28` — the whole of behavior.md's *Effective-disabled rule*),
//! 3. the one-member state record `{ disabled }` (`:30-32`),
//! 4. `useRenderElement('fieldset', componentProps, { ref, state, props: [{ 'aria-labelledby':
//!    legendId, disabled }, elementProps] })` (`:34-44`),
//! 5. the memoized context value `{ legendId, setLegendId, disabled }` provided around
//!    that element (`:46-57`).
//!
//! ## The port's structural decisions
//!
//! - **Two layers, as everywhere else in the crate.** [`fieldset_element`] is the pure
//!   builder (upstream's body up to and including `useRenderElement`, exactly the
//!   `separator_element`/`button_element` facade convention) and [`fieldset_root_view`]
//!   is the composition root: it provides the context, builds the consumer's subtree
//!   inside it, and materializes the merged bag onto a real `<fieldset>` node.
//!   The view layer exists because upstream's root is a PROVIDER AROUND its children
//!   (`:55-57`) and `view!` has no attribute spread — the
//!   `checkbox_group_view`/`avatar_root_view` precedent, which this module follows.
//! - **The context rides the leptos runtime** (`FieldsetRootContext` carries a leptos
//!   `Signal`/`WriteSignal`, both `Send + Sync`, so `provide_context` takes it bare —
//!   no `SendWrapper`). That is what lets `Field.Root` keep reading it with a plain
//!   `use_context` (`field/field_root.rs:202-215`), and it is why the ROOT's
//!   `aria-labelledby` can be a live view attribute: the registration the legend
//!   performs is a leptos write.
//! - **`setLegendId` is the value half only.** Upstream's parameter is
//!   `React.Dispatch<React.SetStateAction<string | undefined>>` — a union whose
//!   second arm (`ClearIfCurrent`) exists purely for the unmount guard inside
//!   `useRegisteredLabelId` (`packages/react/src/utils/useRegisteredLabelId.ts:14-16`).
//!   The context carries the plain [`WriteSignal`]; the guard is built by the legend,
//!   which is the only caller of the union's updater arm (the ported
//!   `LabelIdUpdate` collapse, `leptos-ui-internals/src/use_registered_label_id.rs:43-53`).
//! - **`disabled` is a plain `bool` in the context**, mirroring upstream's memoized
//!   value (it carries the already-OR-ed effective state, `FieldsetRoot.tsx:28,50`).
//!   Documented scope limit: the port's `disabled` is a build-time prop, so a runtime
//!   flip of the ROOT's own prop is the caller's re-invocation (the meter/`Props are
//!   static per body run` precedent); the association (`legendId`) IS live.

use std::rc::Rc;

use leptos::children::Children;
use leptos::html::{Custom, custom};
use leptos::prelude::*;
use leptos::tachys::html::element::ElementChild;
use leptos::tachys::html::node_ref::NodeRefAttribute;
use send_wrapper::SendWrapper;

use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderAttributeFn, RenderElementProps, RenderProp, RenderedElement,
    StyleSource, UseRenderElementComponentProps, UseRenderElementParams, static_attr,
    use_render_element,
};
use leptos_ui_utils::use_merged_refs::{InputRef, RefCallback};
use web_sys::{Element, HtmlElement};

/// Upstream's `FieldsetRootContext` value (`FieldsetRootContext.ts:4-8`):
/// `{ legendId, setLegendId, disabled }`.
///
/// `Copy` like the crate's other signal-bag contexts (both signal handles are arena
/// handles); every field is `Send + Sync`, which is what `provide_context` requires
/// and what keeps `field`'s optional read a plain `use_context`.
#[derive(Clone, Copy)]
pub struct FieldsetRootContext {
    /// `legendId` (`:5`) — the legend's registered id, or `None` (React's `undefined`,
    /// which drops the `aria-labelledby` attribute entirely; the comment at
    /// `FieldsetRoot.tsx:39` and behavior.md's SSR bullets).
    pub legend_id: Signal<Option<String>>,
    /// `setLegendId` (`:6`) — the plain-value dispatch. The `SetStateAction` updater
    /// arm is built by the legend (module docs).
    pub set_legend_id: WriteSignal<Option<String>>,
    /// `disabled` (`:7`) — the EFFECTIVE value (parent OR prop, `:28`).
    pub disabled: bool,
}

/// The `FieldsetRootState` record (`FieldsetRoot.tsx:60-65`): one member, converted
/// to `data-disabled=""` by the engine's default state walk
/// (`getStateAttributesProps.ts:24-28`; no custom `stateAttributesMapping` is passed).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldsetRootState {
    /// `disabled` (`:63`).
    pub disabled: bool,
}

impl FieldsetRootState {
    /// The `serde_json` state map the engine's default walk consumes.
    pub fn to_state_map(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert("disabled".to_string(), serde_json::Value::Bool(self.disabled));
        map
    }
}

/// The builder parameters — upstream's destructured props (`FieldsetRoot.tsx:17-23`)
/// plus the resolved association source.
pub struct FieldsetRootElementProps {
    /// `disabled` (`:21`) — the EFFECTIVE value the caller resolved (parent OR prop).
    /// It is spread onto the element (`:40`) and is the state record's only member.
    pub disabled: bool,
    /// `legendId` (`:39`) — the root's derived association (`aria-labelledby`). A
    /// reactive source, because the legend registers it after the root is built.
    pub legend_id: Signal<Option<String>>,
    /// `className`/`style`/`render` (`:18-20`) through the engine's vocabulary.
    pub render_class_style: UseRenderElementComponentProps,
    /// The `...elementProps` rest (`:22`) — the LAST bag, so a user `aria-labelledby`,
    /// `disabled`, or `data-disabled` overrides the internally managed value
    /// (`:37-43`, `mergeProps.ts:14-15`; implementation.md untested item 2).
    pub element_attributes: Vec<(String, String)>,
    /// The forwarded `ref` (`:14,:36`) — attaches to the `<fieldset>` (behavior.md
    /// "DOM structure": `refInstanceof: window.HTMLFieldSetElement`).
    pub ref_callback: Option<RefCallback<Element>>,
}

impl Default for FieldsetRootElementProps {
    fn default() -> Self {
        Self {
            disabled: false,
            legend_id: Signal::derive(|| None),
            render_class_style: UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            ref_callback: None,
        }
    }
}

/// Builds the `<fieldset>` element description — upstream's `FieldsetRoot` body up to
/// and including `useRenderElement` (`FieldsetRoot.tsx:34-44`), without materializing a
/// DOM node. Must be called inside a reactive owner (the ref fork registers there).
///
/// The intrinsic bag is upstream's `{ 'aria-labelledby': legendId, disabled }` (`:37-43`):
/// `aria-labelledby` omits the attribute when no legend is registered (`None`), and the
/// native `disabled` attribute is the empty string when true — React's
/// `disabled={boolean}` projection, not the port's string-attribute spelling.
pub fn fieldset_element(props: FieldsetRootElementProps) -> Option<RenderedElement> {
    let FieldsetRootElementProps {
        disabled,
        legend_id,
        render_class_style,
        element_attributes,
        ref_callback,
    } = props;

    let state_map = FieldsetRootState { disabled }.to_state_map();

    // The intrinsic bag (`:37-43`'s first element). Both members are LIVE closures:
    // the association changes when the legend registers, and the effect that
    // materializes the bag re-reads them (module docs).
    let legend_id_attr: RenderAttributeFn = Rc::new(move || legend_id.get());
    let disabled_attr: RenderAttributeFn = Rc::new(move || disabled.then(String::new));
    let mut intrinsic = RenderElementProps::default();
    intrinsic.handlers.attributes = vec![
        ("aria-labelledby".to_string(), legend_id_attr),
        ("disabled".to_string(), disabled_attr),
    ];

    // The `...elementProps` rest (`:22`) — the LAST bag, later-wins.
    let mut element_bag = RenderElementProps::default();
    for (name, value) in &element_attributes {
        element_bag
            .handlers
            .attributes
            .push((name.clone(), static_attr(value.clone())));
    }

    let props_bags = vec![
        PropsSource::Static(intrinsic),
        PropsSource::Static(element_bag),
    ];

    let refs: Vec<InputRef<Element>> = match ref_callback {
        Some(callback) => vec![InputRef::Callback(callback)],
        None => Vec::new(),
    };

    // `useRenderElement('fieldset', componentProps, …)` (`:34`) — a native `<fieldset>`
    // (behavior.md "DOM structure"; the implicit `group` role comes free, `:48` of the
    // implementation spec), no `stateAttributesMapping` (the DEFAULT walk).
    use_render_element(
        "fieldset",
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

/// The view-layer props (`FieldsetRootViewProps`, the house naming: a
/// `#[component]`-generated `FieldsetRootProps` would collide with this struct).
/// Upstream's destructured set plus the consumer's subtree.
pub struct FieldsetRootViewProps {
    /// `disabled` (`:21`) — default `false`.
    pub disabled: bool,
    /// The user's `className` (`:19`).
    pub class: Option<String>,
    /// The user's `style` (`:20`) — ordered declarations.
    pub style: Vec<(String, String)>,
    /// `render` (`:18`).
    pub render: Option<RenderProp>,
    /// The `...elementProps` rest (`:22`).
    pub element_attributes: Vec<(String, String)>,
    /// The forwarded `ref` (`:14`).
    pub ref_callback: Option<RefCallback<Element>>,
    /// The grouped controls — upstream's `{element}` children (`:56`).
    pub children: Option<Children>,
}

impl Default for FieldsetRootViewProps {
    fn default() -> Self {
        Self {
            disabled: false,
            class: None,
            style: Vec::new(),
            render: None,
            element_attributes: Vec::new(),
            ref_callback: None,
            children: None,
        }
    }
}

/// The root view — upstream's `FieldsetRoot` body (`FieldsetRoot.tsx:17-57`). Must be
/// called inside a reactive owner (a component body).
///
/// Order matters and follows upstream: the ancestor read resolves the effective
/// disabled BEFORE the context is provided, the provider is installed BEFORE the
/// children are built (so every nested part — including another `<Fieldset.Root>` —
/// resolves this root), and the element description is materialized after.
pub fn fieldset_root_view(props: FieldsetRootViewProps) -> impl IntoView {
    let FieldsetRootViewProps {
        disabled: disabled_prop,
        class,
        style,
        render,
        element_attributes,
        ref_callback,
        children,
    } = props;

    // `useFieldsetRootContext(true)?.disabled` (`:27`) — the OPTIONAL read. The same
    // leptos-context read `Field.Root` performs (`field/field_root.rs:210-213`), so an
    // inner fieldset and a Field nested in this root agree on the effective value.
    let parent_disabled = use_context::<FieldsetRootContext>()
        .map(|context| context.disabled)
        .unwrap_or(false);
    // `const disabled = parentDisabled || disabledProp` (`:28`).
    let disabled = parent_disabled || disabled_prop;

    // `React.useState<string | undefined>(undefined)` (`:25`) — the legendId store,
    // leptos-side so the view's `aria-labelledby` tracks it (module docs).
    let (legend_id_read, set_legend_id) = signal(None::<String>);
    let legend_id: Signal<Option<String>> = legend_id_read.into();
    provide_context(FieldsetRootContext {
        legend_id,
        set_legend_id,
        disabled,
    });

    // `useRenderElement('fieldset', componentProps, …)` (`:34-44`) — the same builder
    // the element path uses, so class/style/render/attribute-merge semantics are the
    // engine's, not a view-side re-derivation (the `avatar_root_view` note). The ref
    // rides `None` here: this view fires it in its own commit (React's ref-after-children
    // order), not through `create_element`'s fork.
    let rendered = fieldset_element(FieldsetRootElementProps {
        disabled,
        legend_id,
        render_class_style: UseRenderElementComponentProps {
            class_name: class.map(ClassNameSource::Static),
            render,
            style: (!style.is_empty()).then(|| StyleSource::Static(style)),
        },
        element_attributes,
        ref_callback: None,
    });

    let Some(rendered) = rendered else {
        // `enabled: false` is not reachable from this unit (the gate is a literal
        // `true` above); the empty view keeps the return type total.
        return ().into_any();
    };

    let tag = rendered.tag.clone();
    // The consumer's subtree (`:56`), built here — after the provider, so the parts
    // resolve this root (module docs).
    let children_view: Option<leptos::prelude::AnyView> = children.map(|children| children());
    let node_ref = NodeRef::<Custom<String>>::new();

    let rendered = SendWrapper::new(rendered);
    let ref_callback = SendWrapper::new(ref_callback);
    // The commit: the merged bag onto the retained node. The body calls every lazy
    // attribute closure, which is where the `legend_id` read is TRACKED — so a legend
    // registering (or withdrawing) its id re-runs this effect and the root's
    // `aria-labelledby` follows (`write_element_bag`, and the checkbox-group
    // mount-writer precedent for the timing).
    Effect::new(move |_| {
        let Some(node) = node_ref.get() else {
            return;
        };
        write_element_bag(&node, &rendered);
        if let Some(callback) = ref_callback.as_ref().as_ref() {
            let element: Element = node.clone().into();
            if let Some(cleanup) = callback(Some(&element)) {
                // The node outlives the mount; the cleanup is leaked with it (the
                // parts' held-or-forgotten listener convention).
                std::mem::forget(cleanup);
            }
        }
    });

    custom(tag).node_ref(node_ref).child(children_view).into_any()
}

/// The React commit — the merged bag written onto the materialized node, with the
/// defined → `undefined` diff (attributes this description no longer carries are
/// removed). `view!` has no attribute spread, so the engine's output is replayed here;
/// the same writer shape the checkbox-group view and the avatar commit use.
pub(crate) fn write_element_bag(node: &HtmlElement, rendered: &RenderedElement) {
    if let Some(class) = &rendered.props.class {
        let _ = node.set_attribute("class", class);
    }
    if !rendered.props.style.is_empty() {
        let style = rendered
            .props
            .style
            .iter()
            .map(|(property, value)| format!("{property}: {value};"))
            .collect::<Vec<_>>()
            .join(" ");
        let _ = node.set_attribute("style", &style);
    }

    let mut seen: Vec<String> = Vec::new();
    for (name, value) in &rendered.props.handlers.attributes {
        match value() {
            Some(value) => {
                let _ = node.set_attribute(name, &value);
            }
            // A bag member that resolved to `None` this run is removed — React's
            // `undefined` projection (the `aria-labelledby` withdrawal when a legend
            // unmounts is exactly this arm).
            None => {
                let _ = node.remove_attribute(name);
            }
        }
        seen.push(name.clone());
    }

    if let Some(inner_html) = &rendered.props.inner_html {
        node.set_inner_html(inner_html);
    }

    // Walk the node's own attribute set for names this description does not manage
    // (a `data-*` that disappeared between commits), the `update_element` contract.
    let dom_attrs = node.get_attribute_names();
    for entry in dom_attrs.iter() {
        let Some(name) = entry.as_string() else {
            continue;
        };
        let managed = seen.iter().any(|seen_name| seen_name == &name)
            || matches!(name.as_str(), "class" | "style");
        if !managed {
            let _ = node.remove_attribute(&name);
        }
    }
}

/// `Fieldset.Root` — the `#[component]` wrapper over [`fieldset_root_view`]. The
/// house convention for a part a docs page instantiates (the field parts precedent).
#[component]
pub fn FieldsetRoot(
    /// `disabled` (`FieldsetRoot.tsx:21`) — default `false`.
    #[prop(default = false, optional)]
    disabled: bool,
    /// The user's `className` (`:19`).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// Upstream's `...elementProps` rest (`:22`) — the port's explicit attribute list.
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The grouped controls.
    children: Children,
) -> impl IntoView {
    fieldset_root_view(FieldsetRootViewProps {
        disabled,
        class,
        element_attributes,
        children: Some(children),
        ..Default::default()
    })
}
