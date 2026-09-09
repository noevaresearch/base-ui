//! Port of the `CompositeRoot`/`CompositeItem` view components —
//! `packages/react/src/internals/composite/root/CompositeRoot.tsx` and
//! `packages/react/src/internals/composite/item/CompositeItem.tsx` — the two
//! `useRenderElement` call sites the [`crate::use_composite_root`] and
//! [`crate::use_composite_item`] checkpoints deferred here ("the CompositeRoot/
//! CompositeItem view components stay with the useRenderElement checkpoint because
//! they are useRenderElement call sites", the item note).
//!
//! Upstream `CompositeRoot` (`:14-96`) is thin wiring: `useDirection()` (`:42`) →
//! `useCompositeRoot` (`:44-64`) → `useRenderElement(tag, componentProps, { state,
//! ref: refs, props: [defaultProps, ...props, elementProps], stateAttributesMapping })`
//! (`:66-71`), wrapped in `CompositeRootContext.Provider` (`:84`) →
//! `<CompositeList elementsRef onMapChange>` (`:85-93`) whose callback fans each
//! publication to the consumer's `onMapChange` prop *and* the hook's internal pump
//! (`:87-90`). `CompositeItem` (`:9-34`) is thinner still: `useCompositeItem` →
//! `useRenderElement(tag, componentProps, { state, ref: [compositeRef, ...refs],
//! props: [compositeProps, ...props, elementProps], stateAttributesMapping })` — the
//! composite ref attaches *first* so an outer item wins when nested items share a DOM
//! node (`:29-30`).
//!
//! ## Rust adaptations
//!
//! - The `#[component]` wrapper that receives JSX props and renders the returned
//!   element into the tree lands with the crate's first view-layer consumer (Phase C)
//!   — the [`crate::direction_provider`] convention. Everything observable about both
//!   components — the hook composition, the prop-bag order, the context/list
//!   provision, the ref ordering — is complete here and pinned by the wasm suite,
//!   with [`RenderedElement::create_element`] materializing the description the way
//!   React renders the upstream components' return value.
//! - The two context providers dissolve into owner-scoped `provide_context` calls
//!   ([`provide_composite_root_context`], [`provide_composite_list`]) made in the
//!   same order upstream nests them — the root context, then the list registry — so
//!   items rendered under the owner see both (`specs/architecture.md`, "Context
//!   passing").
//! - The prop-bag order reproduces the spread arrays verbatim:
//!   `[defaultProps, ...props, elementProps]` for the root (later bags win per the
//!   merge rules — `elementProps` is the consumer's escape hatch),
//!   `[compositeProps, ...props, elementProps]` for the item.
//! - The root's `defaultProps.ref` is a `MergedRefCallback<HtmlElement>` (the hook's
//!   `HtmlElement`-typed root slot); the bag's ref slot is
//!   `MergedRefCallback<Element>`, so the wiring adapts with a downcast that drops
//!   non-HTML elements (upstream `rootRef: React.RefObject<HTMLElement | null>`
//!   would equally only see HTML elements — the root's `tag` defaults to `'div'`).
//! - The item's roving `tabIndex` (`CompositeItemProps.tab_index`, a `Memo<i32>`)
//!   rides a lazy attribute closure that re-reads the memo at read time — the
//!   lazy-attribute convention standing in for the per-render prop value.
//! - The hook's `onHighlightedIndexChange` (`StableCallback<(i32, bool), ()>`) adapts
//!   to the context's `Rc<dyn Fn(i32, bool)>` slot with a closure over
//!   [`StableCallback::call`] — the same handler object downstream, per upstream's
//!   single-`useStableCallback` instance (`useCompositeRoot.ts:111-127`).
//! - `'use client'` is N/A — no React Server Components boundary in Rust.

use std::rc::Rc;

use reactive_graph::traits::{Get, GetUntracked};
use serde_json::Value;
use web_sys::Element;
use web_sys::wasm_bindgen::JsCast;

use leptos_ui_utils::use_merged_refs::{InputRef, MergedRefCallback, RefCallback};
use leptos_ui_utils::use_stable_callback::StableCallback;

use crate::composite_list::{CompositeListMap, provide_composite_list};
use crate::composite_root_context::{CompositeRootContextValue, provide_composite_root_context};
use crate::direction_context::TextDirection;
use crate::floating_ui::element_props::ElementAttributeFn;
use crate::state_attributes::StateAttributesMapping;
use crate::use_composite_item::{UseCompositeItem, UseCompositeItemParams, use_composite_item};
use crate::use_composite_root::{
    CompositeRootProps, UseCompositeRoot, UseCompositeRootParams, use_composite_root,
};
use crate::use_render_element::{
    PropsSource, RenderElementHandlers, RenderElementProps, RenderedElement,
    UseRenderElementComponentProps, UseRenderElementParams, native_to_base_ui, use_render_element,
};

/// The mapping type as the component props carry it — an owned trait object
/// (`Rc<dyn Fn(&str, &Value) -> ...>`, implicitly `'static`), dereferenced to the
/// `&StateAttributesMapping` slice shape [`UseRenderElementParams`] consumes.
pub type OwnedStateAttributesMapping = Rc<StateAttributesMapping<'static>>;

/// The component props of upstream `CompositeRoot.Props` (`:100-130`), narrowed to
/// what the wiring consumes; the keyboard-navigation props travel through
/// [`UseCompositeRootParams`] unchanged.
pub struct CompositeRootComponentProps<M> {
    /// `render`/`className`/`style` (`:101-103`).
    pub render_class_style: UseRenderElementComponentProps,
    /// `tag` (`:38`) — the default element; upstream default `'div'`.
    pub tag: String,
    /// `state` (`:23`).
    pub state: serde_json::Map<String, Value>,
    /// `stateAttributesMapping` (`:24`).
    pub state_attributes_mapping: Option<OwnedStateAttributesMapping>,
    /// `refs` (`:21`).
    pub refs: Vec<InputRef<Element>>,
    /// `props` (`:22`) — the consumer's extra bags, spread between the hook's
    /// defaults and the rest props.
    pub props: Vec<PropsSource>,
    /// The `...elementProps` rest (`:39`) — the consumer's plain attribute/handler
    /// bag, the last (highest-precedence) bag of the spread.
    pub element_props: RenderElementProps,
    /// `onMapChange` (`:32`) — the consumer's publication callback; each map
    /// publication fans out here *and* into the hook's reconciliation pump
    /// (`CompositeRoot.tsx:87-90`).
    pub on_map_change: Option<Rc<dyn Fn(&CompositeListMap<M>)>>,
    /// `highlightItemOnHover` (`:37`) — forwarded into the root context value
    /// (`:77`).
    pub highlight_item_on_hover: bool,
}

/// Port of `CompositeRoot` (`CompositeRoot.tsx:14-96`). Must be called inside a
/// reactive owner (a component). Returns the rendered root element description —
/// upstream's `useRenderElement` return value. Generic over the list's metadata type,
/// upstream `CompositeRoot<Metadata, State>` (`:14`).
pub fn composite_root<M, I, D>(
    component_props: CompositeRootComponentProps<M>,
    hook_params: UseCompositeRootParams<I, D>,
) -> Option<RenderedElement>
where
    M: Clone + PartialEq + 'static,
    I: Clone + Get<Value = Option<i32>> + GetUntracked<Value = Option<i32>> + 'static,
    D: Clone + Get<Value = TextDirection> + GetUntracked<Value = TextDirection> + 'static,
{
    let CompositeRootComponentProps {
        render_class_style,
        tag,
        state,
        state_attributes_mapping,
        refs,
        props,
        element_props,
        on_map_change,
        highlight_item_on_hover,
    } = component_props;

    // `const direction = useDirection()` (`:42`) → `useCompositeRoot({ ..., direction,
    // ... })` (`:61`): in the port the ambient direction rides `hook_params.direction`
    // — callers compose [`use_direction`] into the params, keeping the wiring a pure
    // pass-through.
    // `useCompositeRoot({...})` (`:44-64`) — the return object's consumed members.
    let UseCompositeRoot {
        props: default_props,
        highlighted_index,
        on_highlighted_index_change,
        elements_ref,
        on_map_change: on_map_change_unwrapped,
        relay_keyboard_event,
        ..
    } = use_composite_root::<M, I, D>(hook_params);

    // `useRenderElement(tag, componentProps, { state, ref: refs,
    // props: [defaultProps, ...props, elementProps], stateAttributesMapping })`
    // (`:66-71`).
    let mut bags = Vec::with_capacity(props.len() + 2);
    bags.push(PropsSource::Static(root_default_props_bag(&default_props)));
    bags.extend(props);
    bags.push(PropsSource::Static(element_props));

    let element = use_render_element(
        &tag,
        render_class_style,
        UseRenderElementParams {
            state: &state,
            refs,
            props: bags,
            state_attributes_mapping: state_attributes_mapping.as_deref(),
            ..UseRenderElementParams::default()
        },
    );

    // `CompositeRootContext.Provider value={contextValue}` (`:73-84`).
    provide_composite_root_context(CompositeRootContextValue {
        highlighted_index,
        on_highlighted_index_change: stable_to_context_callback(on_highlighted_index_change),
        highlight_item_on_hover,
        relay_keyboard_event,
    });

    // `<CompositeList elementsRef onMapChange={(newMap) => { onMapChangeProp?.(newMap);
    // onMapChangeUnwrapped(newMap); }}>` (`:85-93`).
    provide_composite_list::<M>(elements_ref, None, move |map: &CompositeListMap<M>| {
        if let Some(on_map_change) = &on_map_change {
            on_map_change(map);
        }
        on_map_change_unwrapped(map);
    });

    element
}

/// Adapts the hook's stable change handle to the context's plain callback slot (see
/// the module docs).
fn stable_to_context_callback(stable: StableCallback<(i32, bool), ()>) -> Rc<dyn Fn(i32, bool)> {
    Rc::new(move |index: i32, should_scroll: bool| {
        stable.call((index, should_scroll));
    })
}

/// Builds the root's `defaultProps` bag (`CompositeRoot.tsx:69` — the first bag of
/// the spread): the hook's `onFocus`/`onKeyDown` handlers and its merged root ref.
fn root_default_props_bag(props: &CompositeRootProps) -> RenderElementProps {
    RenderElementProps {
        handlers: RenderElementHandlers {
            on_focus: Some(native_to_base_ui(Rc::clone(&props.on_focus))),
            on_key_down: Some(native_to_base_ui(Rc::clone(&props.on_key_down))),
            ..RenderElementHandlers::default()
        },
        ref_callback: props.ref_callback.clone().map(html_ref_to_element_ref),
        ..RenderElementProps::default()
    }
}

/// Adapts the hook's `HtmlElement`-typed merged ref to the bag's `Element`-typed
/// slot (see the module docs — the downcast drops non-HTML elements, matching
/// upstream's `HTMLElement`-typed `rootRef`).
fn html_ref_to_element_ref(
    callback: MergedRefCallback<web_sys::HtmlElement>,
) -> MergedRefCallback<Element> {
    Rc::new(move |instance: Option<&Element>| {
        callback(instance.and_then(|element| element.dyn_ref::<web_sys::HtmlElement>()));
    })
}

/// The component props of upstream `CompositeItem.Props` (`:38-49`).
pub struct CompositeItemComponentProps<M> {
    /// `metadata` (`:19`).
    pub metadata: Option<M>,
    /// `render`/`className`/`style` (`:40`).
    pub render_class_style: UseRenderElementComponentProps,
    /// `tag` (`:21`) — the default element; upstream default `'div'`.
    pub tag: String,
    /// `state` (`:16`).
    pub state: serde_json::Map<String, Value>,
    /// `stateAttributesMapping` (`:20`).
    pub state_attributes_mapping: Option<OwnedStateAttributesMapping>,
    /// `refs` (`:18`).
    pub refs: Vec<InputRef<Element>>,
    /// `props` (`:17`) — the consumer's extra bags.
    pub props: Vec<PropsSource>,
    /// The `...elementProps` rest (`:23`).
    pub element_props: RenderElementProps,
}

/// Port of `CompositeItem` (`CompositeItem.tsx:9-34`). Must be called inside a
/// reactive owner and within a `composite_root`'s subtree. Returns the rendered item
/// element description.
pub fn composite_item<M>(component_props: CompositeItemComponentProps<M>) -> Option<RenderedElement>
where
    M: Clone + PartialEq + 'static,
{
    let CompositeItemComponentProps {
        metadata,
        render_class_style,
        tag,
        state,
        state_attributes_mapping,
        refs,
        props,
        element_props,
    } = component_props;

    // `const { compositeProps, compositeRef } = useCompositeItem({ metadata })` (`:25`).
    let item = use_composite_item(UseCompositeItemParams { metadata });

    // `useRenderElement(tag, componentProps, { state, ref: [compositeRef, ...refs],
    // props: [compositeProps, ...props, elementProps] })` (`:27-33`) — the composite
    // ref attaches first so an outer item wins when nested items share a DOM node
    // (`:29-30`).
    let mut refs = refs;
    if let Some(composite_ref) = &item.composite_ref {
        refs.insert(
            0,
            InputRef::Callback(merged_to_ref_callback(composite_ref.clone())),
        );
    }

    let mut bags = Vec::with_capacity(props.len() + 2);
    bags.push(PropsSource::Static(item_default_props_bag(&item)));
    bags.extend(props);
    bags.push(PropsSource::Static(element_props));

    use_render_element(
        &tag,
        render_class_style,
        UseRenderElementParams {
            state: &state,
            refs,
            props: bags,
            state_attributes_mapping: state_attributes_mapping.as_deref(),
            ..UseRenderElementParams::default()
        },
    )
}

/// Builds the item's `compositeProps` bag (`CompositeItem.tsx:31` — the first bag of
/// the spread): the roving `tabIndex`, the focus-move `onFocus`, and the
/// hover-focus `onMouseMove`.
fn item_default_props_bag(item: &UseCompositeItem) -> RenderElementProps {
    let tab_index = item.composite_props.tab_index.clone();
    RenderElementProps {
        handlers: RenderElementHandlers {
            on_focus: Some(native_to_base_ui(Rc::clone(&item.composite_props.on_focus))),
            on_mouse_move: Some(native_to_base_ui(Rc::clone(
                &item.composite_props.on_mouse_move,
            ))),
            attributes: vec![(
                "tabindex".to_string(),
                Rc::new(move || Some(tab_index.get().to_string())) as ElementAttributeFn,
            )],
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    }
}

fn merged_to_ref_callback(callback: MergedRefCallback<Element>) -> RefCallback<Element> {
    Rc::new(move |instance: Option<&Element>| {
        callback(instance);
        None
    })
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::GetUntracked;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Element, KeyboardEvent};

    use super::*;
    use crate::composite_list::CompositeListMap;
    use crate::use_render_element::{
        RenderElementHandlers, RenderElementProps, UseRenderElementComponentProps,
    };

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn key_event(key: &str) -> KeyboardEvent {
        let init = web_sys::KeyboardEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_key(key);
        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap()
    }

    async fn run_microtasks(ticks: usize) {
        for _ in 0..ticks {
            JsFuture::from(js_sys::Promise::resolve(&JsValue::undefined()))
                .await
                .unwrap();
        }
    }

    /// The `CompositeRoot` + `CompositeItem` wiring materialized end to end: the root
    /// description is built and rendered (`CompositeRoot.tsx:66-95`), then `count`
    /// item descriptions are built within the root's subtree context and rendered.
    /// The item *descriptions* are kept so the tests can re-evaluate the roving
    /// `tabindex`'s lazy closure — the per-render value the view layer's reactivity
    /// re-reads (the DOM attribute is one-shot at materialization).
    #[allow(dead_code)] // item_elements pins the materialized nodes for future assertions
    struct Harness {
        _owner: Owner,
        root_element: Element,
        item_elements: Vec<Element>,
        item_descriptions: Vec<RenderedElement>,
        publications: Rc<RefCell<u32>>,
        last_map: Rc<RefCell<Option<CompositeListMap<()>>>>,
    }

    async fn build_harness(count: usize, prevent_arrow_keys: bool) -> Harness {
        // The hooks' effects re-run through the ambient executor (the
        // `use_button`/`use_composite_root` wasm-suite note); re-initializing
        // returns `Err`.
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let publications: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let last_map: Rc<RefCell<Option<CompositeListMap<()>>>> = Rc::new(RefCell::new(None));

        let element_props = RenderElementProps {
            handlers: RenderElementHandlers {
                // The consumer's onKeyDown: upstream's `onKeyDown` rest prop
                // (`CompositeRootProps`, `:124`) — runs before the internal
                // navigation handler and can prevent it.
                on_key_down: Some(Rc::new(
                    move |event: &crate::types::BaseUIEvent<KeyboardEvent>| {
                        if prevent_arrow_keys {
                            event.prevent_base_ui_handler();
                        }
                    },
                )),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };

        // The `useCompositeRoot` params with the component layer's defaults
        // (`CompositeRoot.tsx:33,38`: `stopEventPropagation = true`, `tag = 'div'`;
        // `useCompositeRoot.ts:76-77`: `loopFocus = true`, `orientation = 'both'`).
        let hook_params = UseCompositeRootParams {
            orientation: None,
            grid: None,
            loop_focus: true,
            on_loop: None,
            highlighted_index: None::<RwSignal<Option<i32>>>,
            on_highlighted_index_change: None,
            direction: reactive_graph::computed::Memo::new(|_| TextDirection::Ltr),
            root_ref: InputRef::Empty,
            enable_home_and_end_keys: false,
            stop_event_propagation: true,
            disabled_indices: None,
            modifier_keys: Vec::new(),
        };

        let root_element = {
            let component_props = CompositeRootComponentProps::<()> {
                render_class_style: UseRenderElementComponentProps::default(),
                tag: "div".to_string(),
                state: serde_json::Map::new(),
                state_attributes_mapping: None,
                refs: Vec::new(),
                props: Vec::new(),
                element_props,
                on_map_change: {
                    let publications = Rc::clone(&publications);
                    let last_map = Rc::clone(&last_map);
                    Some(Rc::new(move |map: &CompositeListMap<()>| {
                        *publications.borrow_mut() += 1;
                        *last_map.borrow_mut() = Some(map.clone());
                    }))
                },
                highlight_item_on_hover: false,
            };
            let rendered = composite_root::<
                (),
                RwSignal<Option<i32>>,
                reactive_graph::computed::Memo<TextDirection>,
            >(component_props, hook_params)
            .expect("the root renders");
            let (dom, cleanup) = rendered.create_element();
            document().body().unwrap().append_child(&dom).unwrap();
            // The root lives in the document for the whole test run; the listener
            // cleanup is held for the same lifetime (never dropped).
            std::mem::forget(cleanup);
            dom
        };

        let mut item_elements = Vec::new();
        let mut item_descriptions = Vec::new();
        for index in 0..count {
            let rendered = composite_item::<()>(CompositeItemComponentProps::<()> {
                metadata: None,
                render_class_style: UseRenderElementComponentProps::default(),
                tag: "div".to_string(),
                state: serde_json::Map::new(),
                state_attributes_mapping: None,
                refs: Vec::new(),
                props: Vec::new(),
                element_props: RenderElementProps::default(),
            })
            .expect("the item renders");
            let (dom, cleanup) = rendered.create_element();
            std::mem::forget(cleanup);
            dom.set_text_content(Some(&format!("{}", index + 1)));
            root_element.append_child(&dom).unwrap();
            item_elements.push(dom);
            item_descriptions.push(rendered);
        }

        // The registry's coalesced flush (the mount publication).
        run_microtasks(2).await;

        Harness {
            _owner: owner,
            root_element,
            item_elements,
            item_descriptions,
            publications,
            last_map,
        }
    }

    /// Re-evaluates the roving `tabindex` lazy closure of one item description — the
    /// per-render read the view layer's reactivity performs on every render.
    fn rendered_tabindex(description: &RenderedElement) -> Option<String> {
        description
            .props
            .handlers
            .attributes
            .iter()
            .find(|(name, _)| name == "tabindex")
            .and_then(|(_, value)| value())
    }

    // The full wiring end to end: items register into the list (the consumer's
    // `onMapChange` fires through the `:87-90` fan-out with the map), the roving
    // `tabIndex` resolves highlighted→0 / others→-1 through the item descriptions'
    // lazy closures (`useCompositeItem.ts:27`), and an ArrowDown on the root moves
    // the highlight through the composed `onKeyDown` (`useCompositeRoot.ts:329`).
    #[wasm_bindgen_test]
    async fn root_and_items_wire_the_registry_context_and_navigation_end_to_end() {
        let harness = build_harness(3, false).await;

        assert!(
            *harness.publications.borrow() >= 1,
            "the item registrations published the map to the consumer callback"
        );
        let map = harness
            .last_map
            .borrow()
            .clone()
            .expect("the map publication reached the consumer");
        assert_eq!(map.len(), 3, "all three items registered");
        let mut indexes: Vec<i32> = map.iter().map(|(_, metadata)| metadata.index).collect();
        indexes.sort_unstable();
        assert_eq!(
            indexes,
            vec![0, 1, 2],
            "the items hold indexes 0..2 in document order"
        );

        assert_eq!(
            rendered_tabindex(&harness.item_descriptions[0]),
            Some("0".to_string()),
            "the initially highlighted item's roving tabindex is 0"
        );
        assert_eq!(
            rendered_tabindex(&harness.item_descriptions[1]),
            Some("-1".to_string()),
            "the other items' roving tabindex is -1"
        );

        harness
            .root_element
            .dispatch_event(&key_event("ArrowDown"))
            .unwrap();

        // The highlight moved to index 1 — the roving tabindex re-reads the
        // reactive value (the view layer re-renders the attribute from it), and
        // the item hooks' reactive `tab_index` memo observes the same source.
        let context = crate::composite_root_context::use_composite_root_context()
            .expect("the root context is provided inside the owner");
        assert_eq!(
            context.highlighted_index.get_untracked(),
            1,
            "ArrowDown moved the highlight through the composed keydown handler"
        );
        assert_eq!(
            rendered_tabindex(&harness.item_descriptions[0]),
            Some("-1".to_string()),
            "the previously highlighted item's roving tabindex follows the move"
        );
        assert_eq!(
            rendered_tabindex(&harness.item_descriptions[1]),
            Some("0".to_string()),
            "the newly highlighted item's roving tabindex follows the move"
        );
    }

    // The consumer's `onKeyDown` runs before the internal navigation handler and
    // `preventBaseUIHandler()` stops it — the mergeProps prevention contract over
    // the `[defaultProps, ...props, elementProps]` spread (`CompositeRoot.tsx:69`).
    #[wasm_bindgen_test]
    async fn the_consumers_keydown_can_prevent_the_navigation() {
        let harness = build_harness(3, true).await;

        let before = {
            let context = crate::composite_root_context::use_composite_root_context().unwrap();
            context.highlighted_index.get_untracked()
        };
        harness
            .root_element
            .dispatch_event(&key_event("ArrowDown"))
            .unwrap();
        let context = crate::composite_root_context::use_composite_root_context().unwrap();
        assert_eq!(
            context.highlighted_index.get_untracked(),
            before,
            "the prevented dispatch never reached the navigation pipeline"
        );
    }

    // The item's props spread order: the consumer's `elementProps` is the last bag,
    // so its plain attributes win over the composite props' — including `tabindex`
    // (`CompositeItem.tsx:31`).
    #[wasm_bindgen_test]
    async fn the_items_element_props_are_the_highest_precedence_bag() {
        let owner = Owner::new();
        owner.set();

        // Provide a root context so the item hook finds its required accessor.
        let root_owner_context = crate::composite_root_context::CompositeRootContextValue {
            highlighted_index: reactive_graph::computed::Memo::new(|_| 0),
            on_highlighted_index_change: Rc::new(|_, _| {}),
            highlight_item_on_hover: false,
            relay_keyboard_event: Rc::new(|_: &KeyboardEvent| {}),
        };
        crate::composite_root_context::provide_composite_root_context(root_owner_context);
        let _list = crate::composite_list::CompositeList::<()>::new(
            Rc::new(RefCell::new(Vec::new())),
            None,
            |_: &CompositeListMap<()>| {},
        );

        let rendered = composite_item::<()>(CompositeItemComponentProps::<()> {
            metadata: None,
            render_class_style: UseRenderElementComponentProps::default(),
            tag: "div".to_string(),
            state: serde_json::Map::new(),
            state_attributes_mapping: None,
            refs: Vec::new(),
            props: Vec::new(),
            element_props: RenderElementProps {
                handlers: RenderElementHandlers {
                    attributes: vec![(
                        "tabindex".to_string(),
                        crate::use_render_element::static_attr("5".to_string()),
                    )],
                    ..RenderElementHandlers::default()
                },
                ..RenderElementProps::default()
            },
        })
        .unwrap();
        let (dom, _cleanup) = rendered.create_element();

        assert_eq!(
            dom.get_attribute("tabindex"),
            Some("5".to_string()),
            "elementProps' tabindex wins over the composite props' roving value"
        );
    }
}
