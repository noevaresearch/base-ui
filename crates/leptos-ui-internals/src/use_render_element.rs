//! Port of `packages/react/src/internals/useRenderElement.tsx` — the props-merge +
//! element-evaluation primitive every Base UI view component renders through.
//!
//! Upstream `useRenderElement(element, componentProps, params)` computes the merged
//! `outProps` (state `data-*` attributes + the intrinsic prop bags + the consumer's
//! `className`/`style`, with refs forked) and then *evaluates the render prop*
//! (`evaluateRenderProp`, `:158-206`): a render function is called with
//! `(props, state)`; a render element is cloned with the merged props; otherwise the
//! default tag renders — with `button` forced to `type="button"` and `img` to `alt=""`
//! at the JSX level (`renderTag`, `:232-240`). `enabled: false` returns `null` and
//! short-circuits prop computation (`:40-47`).
//!
//! ## The port's element vocabulary
//!
//! React's `ReactElement` return value becomes [`RenderedElement`] — a resolved
//! *element description* (`tag` + merged [`RenderElementProps`]). Materializing it
//! into a real DOM node is [`RenderedElement::create_element`] — the seam standing in
//! for "React renders the returned element" (upstream the React reconciler's job). The
//! description is the unit of composition the crate's view-free layer passes around;
//! the Phase C `#[component]` wrappers turn it into views, and the wasm tests
//! materialize it to pin the end-to-end behavior the upstream suite asserts against
//! real DOM.
//!
//! The merged-props bag ([`RenderElementProps`]) specializes the generic
//! `Record<string, any>` `outProps` to the crate's established vocabulary: typed
//! handler slots ([`RenderElementHandlers`]), lazy `(name, value-fn)` attribute pairs
//! (the `useButton`/`ElementHandlers` convention), and merged `class`/`style`.
//!
//! ## Rust adaptations
//!
//! - **The merge machinery** ([`merge_props_n`], [`merge_class_names`],
//!   [`merge_styles`], [`merge_event_handlers`]) reproduces
//!   `packages/react/src/merge-props/mergeProps.ts` semantics over the typed bag
//!   vocabulary: later bags win for plain attributes (rightmost-overwrites,
//!   `mergeProps.ts:14-15`), handlers compose right-to-left with the leftmost (most
//!   internal) handler skipped once [`BaseUIEvent::prevent_base_ui_handler`] marks the
//!   dispatch (`:17-20`, `:229-249`), `class` concatenates with the later bag's
//!   classes first (`mergeClassNames`, `:276-290`), `style` merges per-key with the
//!   later bag winning (`:166-172`), and a props-getter bag (the function form of
//!   `InputProps`, `:7-8`) is resolved *wholesale* against the merged-so-far props —
//!   the getter owns handler chaining and its handlers are not automatically
//!   prevention-gated (`:25-31`, `:210-219`). `ref` is not merged by the fold — a
//!   later bag only replaces an earlier bag's ref when it actually carries one
//!   (upstream's `for...in` iterates the later bag's own keys), and the fork happens
//!   once in [`use_render_element`] over [bag ref, render-element ref, `params.refs`]
//!   exactly like `useMergedRefs(outProps.ref, getReactElementRef(renderProp), ref)`
//!   (`:99-103`). The standalone public `mergeProps`/`mergePropsN` utility (with
//!   `resolveAriaLabelledBy` and its own suite) remains the `infra: merge-props` TODO
//!   item's scope; this module carries only what evaluation needs, exposed so that
//!   item can re-home rather than duplicate.
//! - **Handler slots are [`BaseUIEvent`]-typed.** Upstream wraps every synthetic-event
//!   handler so `preventBaseUIHandler` exists on each dispatch (`wrapEventHandler`,
//!   `:252-266`); the port bakes that into the slot type — the attach seam constructs
//!   one [`BaseUIEvent`] per DOM dispatch (the shared-mark cell standing in for the JS
//!   object's identity) and the composed slot handler gates the earlier handlers on
//!   it. Internal hooks return native-event handlers ([`ElementEventHandler`] over
//!   `web_sys` types — e.g. `CompositeRootProps.on_key_down`); [`native_to_base_ui`]
//!   adapts them at bag-build time the way `wrapEventHandler` wraps, with the handler
//!   reading [`BaseUIEvent::inner`]. `onFocus`/`onBlur` attach under the bubbling
//!   `focusin`/`focusout` (React 17+ delegation, the `floating_ui::element_props`
//!   module-docs precedent); every other slot maps 1:1. The slot set is narrowed to
//!   the events this unit's call sites fill (`CompositeRoot`'s
//!   `onFocus`/`onKeyDown`, `CompositeItem`'s `onFocus`/`onMouseMove`) plus the
//!   preventability matrix the upstream suite pins (`onMouseDown`, `onContextMenu`,
//!   `onClick`) — the same narrowing rule the `floating_ui::element_props` vocabulary
//!   documents.
//! - **`enabled: false`** returns [`None`] and skips bag resolution entirely — getters
//!   are not called (`useRenderElement.test.tsx:200-209`). Upstream still calls
//!   `useMergedRefs(null, null)` to keep React's hook order stable (`:96-98`); the
//!   port runs its hooks once per component instance, so there is no order to keep
//!   and the call is dropped.
//! - **State** is the crate's concrete state vocabulary — a `serde_json` object map,
//!   the shape [`get_state_attributes_props`] consumes — standing in for the generic
//!   `State extends Record<string, any>`; the class/style function props take it by
//!   reference, matching `(state) => ...` (`useRenderElement.tsx:73-74`).
//! - **N/A adaptations** (React-runtime specifics with no Rust shape): the
//!   lazy/Flight-shaped render-prop unwrap (`unwrapLazyRenderProp`, `:139-156` — no
//!   Flight client exists); the dev-only uppercase-function-name warning
//!   (`warnIfRenderPropLooksLikeComponent`, `:208-230` — `render={Component}` misuse
//!   is a compile-time type error here, and Rust closures carry no JavaScript
//!   `name`); the invalid-render-element throw (`:182-194` — the type system rejects
//!   non-elements); and the rerender-driven behaviors (`useRenderElement.test.tsx:211-268`
//!   — enabled toggles and ref-shape changes across rerenders — which dissolve, since
//!   the port runs its hooks once per instance and the fork is built there).
//! - `'use client'` is N/A — no React Server Components boundary in Rust.

use std::rc::Rc;

use serde_json::Value;
use web_sys::Element;

use leptos_ui_utils::add_event_listener::EventListenerUnsubscribe;
use leptos_ui_utils::merge_cleanups::CleanupFn;
use leptos_ui_utils::use_merged_refs::{
    InputRef, MergedRefCallback, RefCallback, use_merged_refs_n,
};

use crate::floating_ui::element_props::{ElementAttributeFn, ElementEventHandler};
use crate::state_attributes::{
    StateAttributeProps, StateAttributesMapping, get_state_attributes_props,
};
use crate::types::BaseUIEvent;

/// One merged event-handler slot set — upstream's handler members of the `outProps`
/// record, narrowed to the events this unit's call sites and upstream test matrix use
/// (see the module docs). Every slot receives the dispatch wrapped in
/// [`BaseUIEvent`], so any handler can call
/// [`BaseUIEvent::prevent_base_ui_handler`] to skip the earlier (more internal)
/// handlers in the composed chain.
#[derive(Clone, Default)]
pub struct RenderElementHandlers {
    /// `onFocus` — attached as `focusin` (the React 17+ delegation mapping).
    pub on_focus: Option<ElementEventHandler<BaseUIEvent<web_sys::FocusEvent>>>,
    /// `onBlur` — attached as `focusout`.
    pub on_blur: Option<ElementEventHandler<BaseUIEvent<web_sys::FocusEvent>>>,
    /// `onClick`.
    pub on_click: Option<ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>>>,
    /// `onMouseDown`.
    pub on_mouse_down: Option<ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>>>,
    /// `onContextMenu` — the upstream suite's "obscure event" preventability probe.
    pub on_context_menu: Option<ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>>>,
    /// `onMouseMove`.
    pub on_mouse_move: Option<ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>>>,
    /// `onKeyDown`.
    pub on_key_down: Option<ElementEventHandler<BaseUIEvent<web_sys::KeyboardEvent>>>,
    /// `onKeyUp`.
    pub on_key_up: Option<ElementEventHandler<BaseUIEvent<web_sys::KeyboardEvent>>>,
    /// `onPointerDown`.
    pub on_pointer_down: Option<ElementEventHandler<BaseUIEvent<web_sys::PointerEvent>>>,
    /// The bag's non-handler attribute members (the `aria-activedescendant` pattern) —
    /// one `(name, value-fn)` entry per attribute, resolved lazily at read time.
    pub attributes: Vec<(String, ElementAttributeFn)>,
}

/// Adapts a native-event handler into the bag's [`BaseUIEvent`]-typed slot shape —
/// `wrapEventHandler`'s wrapping (`mergeProps.ts:252-266`) as a bag-build-time
/// conversion: the handler observes the wrapped dispatch's underlying event and
/// ignores the augmentation.
pub fn native_to_base_ui<E: Clone + 'static>(
    handler: ElementEventHandler<E>,
) -> ElementEventHandler<BaseUIEvent<E>> {
    Rc::new(move |event: &BaseUIEvent<E>| handler(event.inner()))
}

/// The lazy attribute-value closure of one bag entry; `None` is the omitted prop.
pub type RenderAttributeFn = ElementAttributeFn;

/// The merged-props bag — upstream's `outProps`
/// (`React.HTMLAttributes<any> & React.RefAttributes<any>`, `:62`) specialized to the
/// crate's attribute vocabulary. `ref_callback` is the bag's own `ref` member: the
/// fold does not merge refs (mergeProps.ts:33's `@important`), so this is the last
/// ref-carrying bag's slot; the full fork (with the render-element ref and the
/// caller's refs) is built once in [`use_render_element`].
#[derive(Clone, Default)]
pub struct RenderElementProps {
    /// The handler slots.
    pub handlers: RenderElementHandlers,
    /// `className` — the merged class string.
    pub class: Option<String>,
    /// `style` — the merged declarations, ordered `(property, value)` pairs.
    pub style: Vec<(String, String)>,
    /// The bag's `ref` member (last ref-carrying bag wins — see the module docs).
    pub ref_callback: Option<MergedRefCallback<Element>>,
}

impl RenderElementProps {
    /// True when the bag carries nothing — upstream's `EMPTY_OBJECT` bags.
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
            && self.class.is_none()
            && self.style.is_empty()
            && self.ref_callback.is_none()
    }
}

impl RenderElementHandlers {
    /// Whether no slot is filled.
    pub fn is_empty(&self) -> bool {
        self.on_focus.is_none()
            && self.on_blur.is_none()
            && self.on_click.is_none()
            && self.on_mouse_down.is_none()
            && self.on_context_menu.is_none()
            && self.on_mouse_move.is_none()
            && self.on_key_down.is_none()
            && self.on_key_up.is_none()
            && self.on_pointer_down.is_none()
            && self.attributes.is_empty()
    }

    /// Attaches every filled slot to `target` as a bubble-phase native listener,
    /// wrapping each dispatch in one shared [`BaseUIEvent`] (`makeEventPreventable`
    /// before the fan-out, `mergeProps.ts:236-238`). `onFocus`/`onBlur` attach under
    /// `focusin`/`focusout` (see the module docs). Returns the merged unsubscribe —
    /// the composition work upstream's JSX spread does; `None` when the bag is empty.
    pub fn attach_to(&self, target: &web_sys::EventTarget) -> Option<CleanupFn> {
        use web_sys::wasm_bindgen::JsCast;

        let mut cleanups: Vec<Option<CleanupFn>> = Vec::new();

        fn wrap_unsubscribe(unsubscribe: EventListenerUnsubscribe) -> Option<CleanupFn> {
            Some(Box::new(move || unsubscribe.unsubscribe()))
        }

        macro_rules! attach {
            ($slot:expr, $event_name:literal, $event_type:ty) => {
                if let Some(handler) = &$slot {
                    let handler = Rc::clone(handler);
                    let unsubscribe = leptos_ui_utils::add_event_listener(
                        target,
                        $event_name,
                        move |event: &web_sys::Event| {
                            if let Some(typed) = event.dyn_ref::<$event_type>() {
                                handler(&BaseUIEvent::new(typed.clone()));
                            }
                        },
                    );
                    cleanups.push(wrap_unsubscribe(unsubscribe));
                }
            };
        }

        attach!(self.on_focus, "focusin", web_sys::FocusEvent);
        attach!(self.on_blur, "focusout", web_sys::FocusEvent);
        attach!(self.on_click, "click", web_sys::MouseEvent);
        attach!(self.on_mouse_down, "mousedown", web_sys::MouseEvent);
        attach!(self.on_context_menu, "contextmenu", web_sys::MouseEvent);
        attach!(self.on_mouse_move, "mousemove", web_sys::MouseEvent);
        attach!(self.on_key_down, "keydown", web_sys::KeyboardEvent);
        attach!(self.on_key_up, "keyup", web_sys::KeyboardEvent);
        attach!(self.on_pointer_down, "pointerdown", web_sys::PointerEvent);

        if self.is_empty() {
            return None;
        }
        let merged = leptos_ui_utils::merge_cleanups(cleanups);
        Some(Box::new(merged))
    }
}

/// The `className` component prop — a string or a state function
/// (`useRenderElement.tsx:292`).
pub enum ClassNameSource {
    Static(String),
    Function(Rc<dyn Fn(&serde_json::Map<String, Value>) -> Option<String>>),
}

/// The `style` component prop — a style record or a state function
/// (`useRenderElement.tsx:301`). The record is ordered `(property, value)` pairs.
pub enum StyleSource {
    Static(Vec<(String, String)>),
    Function(Rc<dyn Fn(&serde_json::Map<String, Value>) -> Option<Vec<(String, String)>>>),
}

/// The render prop — upstream `render?: ReactElement | ComponentRenderFn`
/// (`useRenderElement.tsx:296`).
pub enum RenderProp {
    /// A render element: cloned with the merged props (`:172-196`). Its own props are
    /// the *later* argument of the merge (its plain attributes win, its handlers run
    /// first, its class prepends) — `mergeProps(props, render.props)`, `:172`, the
    /// precedence the lazy-element test pins (`useRenderElement.test.tsx:531-548`).
    Element {
        tag: String,
        props: RenderElementProps,
    },
    /// A render function: called with the merged props and the state (`:169`),
    /// owning the returned element wholesale.
    Function(RenderFn),
}

/// The render-function shape — `(props, state) => ReactElement` (`:165-170`).
/// Receives the merged bag *including* the forked ref and returns the final element
/// description.
pub type RenderFn =
    Rc<dyn Fn(RenderElementProps, &serde_json::Map<String, Value>) -> RenderedElement>;

/// One intrinsic props bag of `params.props` — upstream's
/// `RenderFunctionProps<TagName> | (props) => RenderFunctionProps<TagName>` array
/// members (`:273-280`).
pub enum PropsSource {
    /// A props record.
    Static(RenderElementProps),
    /// A props getter: resolved wholesale against the merged-so-far props and
    /// *replacing* them — the getter owns handler chaining and prevention gating
    /// (`mergeProps.ts:25-31`, `:210-219`).
    Getter(RenderPropsGetter),
}

/// The props-getter callable — receives the merged props up to that point
/// (`mergeProps.ts:27`).
pub type RenderPropsGetter = Rc<dyn Fn(&RenderElementProps) -> RenderElementProps>;

/// `useRenderElement`'s third parameter (`UseRenderElementParameters`, `:246-285`).
pub struct UseRenderElementParams<'a> {
    /// `enabled` (`:257`) — `false` skips everything and yields [`None`].
    pub enabled: bool,
    /// `state` (`:269`).
    pub state: &'a serde_json::Map<String, Value>,
    /// `ref` (`:265`) — the caller's ref(s); the single form is a one-element vec
    /// (upstream's single-or-array union, `:265`).
    pub refs: Vec<InputRef<Element>>,
    /// `props` (`:273`) — the intrinsic bags, left-to-right (later wins).
    pub props: Vec<PropsSource>,
    /// `stateAttributesMapping` (`:284`).
    pub state_attributes_mapping: Option<&'a StateAttributesMapping<'a>>,
}

/// Upstream's `state ?? EMPTY_OBJECT` default (`:44`) — the shared empty map. The
/// static borrow keeps the default reference valid for any `params` lifetime.
static EMPTY_STATE: std::sync::OnceLock<serde_json::Map<String, Value>> =
    std::sync::OnceLock::new();

impl<'a> Default for UseRenderElementParams<'a> {
    fn default() -> Self {
        Self {
            enabled: true,
            state: EMPTY_STATE.get_or_init(serde_json::Map::new),
            refs: Vec::new(),
            props: Vec::new(),
            state_attributes_mapping: None,
        }
    }
}

/// The component props — upstream `UseRenderElementComponentProps` (`:287-302`),
/// narrowed to the three members evaluation reads (`render`, `className`, `style`;
/// upstream's doc note at `:19` — "Other props are ignored").
#[derive(Default)]
pub struct UseRenderElementComponentProps {
    /// `className` (`:292`).
    pub class_name: Option<ClassNameSource>,
    /// `render` (`:296`).
    pub render: Option<RenderProp>,
    /// `style` (`:301`).
    pub style: Option<StyleSource>,
}

/// The resolved element — upstream's returned `React.ReactElement` as a description:
/// the tag that receives the merged props.
#[derive(Clone)]
pub struct RenderedElement {
    /// The tag — the default element, or the render prop's choice.
    pub tag: String,
    /// The merged props to spread onto the element (including the forked
    /// `ref_callback`).
    pub props: RenderElementProps,
}

/// A concrete attribute value as a lazy closure.
pub fn static_attr(value: String) -> ElementAttributeFn {
    Rc::new(move || Some(value.clone()))
}

/// `mergeClassNames` (`mergeProps.ts:276-290`): the later (rightmost) class string
/// comes first in the concatenation.
pub fn merge_class_names(our_class: Option<String>, their_class: Option<String>) -> Option<String> {
    match (their_class, our_class) {
        (Some(theirs), Some(ours)) => Some(format!("{theirs} {ours}")),
        (Some(theirs), None) => Some(theirs),
        (None, ours) => ours,
    }
}

/// `mergeObjects` over the style pairs (`mergeProps.ts:166-172`): the later
/// (rightmost) declarations win per property, both sides retained otherwise.
pub fn merge_styles(
    our_style: Vec<(String, String)>,
    their_style: Vec<(String, String)>,
) -> Vec<(String, String)> {
    let mut merged = our_style;
    for (property, value) in their_style {
        match merged
            .iter_mut()
            .find(|(existing, _)| *existing == property)
        {
            Some(slot) => slot.1 = value,
            None => merged.push((property, value)),
        }
    }
    merged
}

/// `mergeEventHandlers` (`mergeProps.ts:221-250`) over the [`BaseUIEvent`]-typed
/// slots: the later (more external) handler runs first; the earlier handler runs
/// unless the dispatch was marked with [`BaseUIEvent::prevent_base_ui_handler`] — the
/// shared-mark cell standing in for the JS augmented object's identity across the
/// whole composed chain.
pub fn merge_event_handlers<E: Clone + 'static>(
    our_handler: Option<ElementEventHandler<BaseUIEvent<E>>>,
    their_handler: Option<ElementEventHandler<BaseUIEvent<E>>>,
) -> Option<ElementEventHandler<BaseUIEvent<E>>> {
    match (our_handler, their_handler) {
        (ours, None) => ours,
        (None, theirs) => theirs,
        (Some(ours), Some(theirs)) => Some(Rc::new(move |event: &BaseUIEvent<E>| {
            theirs(event);
            if !event.base_ui_handler_prevented() {
                ours(event);
            }
        })),
    }
}

/// Folds one later bag into the accumulated merged props — `mutablyMergeInto`
/// (`mergeProps.ts:153-188`) over the typed vocabulary: handlers compose (later runs
/// first, mark-gated), `class` concatenates later-first, `style` merges per-key,
/// plain attributes are replaced per key by the later bag, and `ref` is only replaced
/// when the later bag actually carries one (upstream's `for...in` iterates the later
/// bag's own keys; absent keys never overwrite).
fn merge_into(merged: &mut RenderElementProps, later: RenderElementProps) {
    let RenderElementProps {
        handlers,
        class,
        style,
        ref_callback,
    } = later;

    let RenderElementHandlers {
        on_focus,
        on_blur,
        on_click,
        on_mouse_down,
        on_context_menu,
        on_mouse_move,
        on_key_down,
        on_key_up,
        on_pointer_down,
        attributes: incoming_attributes,
    } = handlers;

    let RenderElementHandlers {
        on_focus: our_focus,
        on_blur: our_blur,
        on_click: our_click,
        on_mouse_down: our_mouse_down,
        on_context_menu: our_context_menu,
        on_mouse_move: our_mouse_move,
        on_key_down: our_key_down,
        on_key_up: our_key_up,
        on_pointer_down: our_pointer_down,
        attributes: our_attributes,
    } = std::mem::take(&mut merged.handlers);

    let mut attributes = our_attributes;
    for (name, value) in incoming_attributes {
        match attributes
            .iter_mut()
            .find(|(existing, _)| *existing == name)
        {
            Some(slot) => slot.1 = value,
            None => attributes.push((name, value)),
        }
    }

    merged.handlers = RenderElementHandlers {
        on_focus: merge_event_handlers(our_focus, on_focus),
        on_blur: merge_event_handlers(our_blur, on_blur),
        on_click: merge_event_handlers(our_click, on_click),
        on_mouse_down: merge_event_handlers(our_mouse_down, on_mouse_down),
        on_context_menu: merge_event_handlers(our_context_menu, on_context_menu),
        on_mouse_move: merge_event_handlers(our_mouse_move, on_mouse_move),
        on_key_down: merge_event_handlers(our_key_down, on_key_down),
        on_key_up: merge_event_handlers(our_key_up, on_key_up),
        on_pointer_down: merge_event_handlers(our_pointer_down, on_pointer_down),
        attributes,
    };

    merged.class = merge_class_names(merged.class.take(), class);
    merged.style = merge_styles(std::mem::take(&mut merged.style), style);
    if ref_callback.is_some() {
        merged.ref_callback = ref_callback;
    }
}

/// Resolves one [`PropsSource`] against the merged-so-far props —
/// `createInitialMergedProps`/`mergeInto` (`mergeProps.ts:117-131`). A getter replaces
/// the accumulated props wholesale (`resolvePropsGetter`, `:210-219`).
fn resolve_source(source: PropsSource, previous: RenderElementProps) -> RenderElementProps {
    match source {
        PropsSource::Static(props) => {
            let mut merged = previous;
            merge_into(&mut merged, props);
            merged
        }
        PropsSource::Getter(getter) => getter(&previous),
    }
}

/// `mergePropsN` over the bag vocabulary (`mergeProps.ts:99-115`): left-to-right
/// fold, later bags winning per the [`merge_into`] rules.
pub fn merge_props_n(props: Vec<PropsSource>) -> RenderElementProps {
    let mut bags = props;
    let Some(first) = (!bags.is_empty()).then(|| bags.remove(0)) else {
        return RenderElementProps::default();
    };
    let mut merged = match first {
        PropsSource::Static(props) => props,
        PropsSource::Getter(getter) => getter(&RenderElementProps::default()),
    };
    for bag in bags {
        merged = resolve_source(bag, merged);
    }
    merged
}

/// Port of `useRenderElement` (`useRenderElement.tsx:22-48`). Must be called inside a
/// reactive owner (a component) — the ref fork registers there.
pub fn use_render_element(
    element: &str,
    component_props: UseRenderElementComponentProps,
    params: UseRenderElementParams<'_>,
) -> Option<RenderedElement> {
    let UseRenderElementComponentProps {
        class_name,
        render,
        style,
    } = component_props;

    // `if (params.enabled !== false) { renderProp = unwrapLazyRenderProp(...) }`
    // (`:33-36`) — the lazy/Flight unwrap has no Rust shape (module docs).
    if !params.enabled {
        // Upstream still calls `useMergedRefs(null, null)` for hook-order stability
        // (`:96-98`); the port runs its hooks once — nothing to keep stable.
        return None;
    }

    // `useRenderElementProps` (`:53-119`): state attributes, then the intrinsic
    // bags, then className/style, with the ref fork over
    // [bag ref, render-element ref, params refs] (`:99-103`).
    let state_attrs: StateAttributeProps =
        get_state_attributes_props(params.state, params.state_attributes_mapping);

    let mut out_props = RenderElementProps {
        handlers: RenderElementHandlers {
            attributes: state_attrs
                .into_iter()
                .map(|(name, value)| (name, static_attr(value)))
                .collect(),
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    };

    for source in params.props {
        out_props = resolve_source(source, out_props);
    }

    // `className`/`style` resolution (`:73-74`) applied after the bag merge
    // (`:110-116`); the component prop is the *later* argument of both merges.
    let resolved_class = class_name.map(|source| match source {
        ClassNameSource::Static(class) => Some(class),
        ClassNameSource::Function(function) => function(params.state),
    });
    if let Some(class) = resolved_class {
        out_props.class = merge_class_names(out_props.class.take(), class);
    }

    let resolved_style = style.map(|source| match source {
        StyleSource::Static(style) => Some(style),
        StyleSource::Function(function) => function(params.state),
    });
    if let Some(style) = resolved_style {
        out_props.style = merge_styles(
            std::mem::take(&mut out_props.style),
            style.unwrap_or_default(),
        );
    }

    // The ref fork (`:99-103`): [bag ref, render-element ref, ...params refs]. The
    // render element's own ref participates exactly where upstream's
    // `getReactElementRef(renderProp)` slot sits — inside the fork, before the
    // caller's refs.
    let render_element_ref = match &render {
        Some(RenderProp::Element { props, .. }) => props.ref_callback.clone(),
        _ => None,
    };

    let mut fork_inputs: Vec<InputRef<Element>> = Vec::new();
    if let Some(bag_ref) = &out_props.ref_callback {
        fork_inputs.push(InputRef::Callback(merged_callback_to_ref(Rc::clone(
            bag_ref,
        ))));
    }
    if let Some(render_ref) = &render_element_ref {
        fork_inputs.push(InputRef::Callback(merged_callback_to_ref(Rc::clone(
            render_ref,
        ))));
    }
    fork_inputs.extend(params.refs.iter().cloned());
    let forked_ref = use_merged_refs_n(fork_inputs);

    // `evaluateRenderProp` (`:158-206`).
    match render {
        // `return render(props, state)` (`:169`).
        Some(RenderProp::Function(function)) => {
            out_props.ref_callback = forked_ref;
            Some(function(out_props, params.state))
        }
        // `mergeProps(props, render.props)` + `cloneElement` (`:172-196`). The
        // invalid-element throw has no Rust shape (module docs).
        Some(RenderProp::Element { tag, props }) => {
            let mut merged = out_props;
            merge_into(&mut merged, props);
            // `mergedProps.ref = props.ref` (`:174`) — the fork already includes the
            // render element's own ref, and the forked callback replaces whatever the
            // fold carried.
            merged.ref_callback = forked_ref;
            Some(RenderedElement { tag, props: merged })
        }
        // `renderTag` (`:232-240`): the default tag with the JSX-level defaults —
        // `button` forced to `type="button"`, `img` to `alt=""` — overridden by any
        // bag that carried the attribute.
        None => {
            out_props.ref_callback = forked_ref;
            match element {
                "button"
                    if !out_props
                        .handlers
                        .attributes
                        .iter()
                        .any(|(name, _)| name == "type") =>
                {
                    out_props
                        .handlers
                        .attributes
                        .push(("type".to_string(), static_attr("button".to_string())));
                }
                "img"
                    if !out_props
                        .handlers
                        .attributes
                        .iter()
                        .any(|(name, _)| name == "alt") =>
                {
                    out_props
                        .handlers
                        .attributes
                        .push(("alt".to_string(), static_attr(String::new())));
                }
                _ => {}
            }
            Some(RenderedElement {
                tag: element.to_string(),
                props: out_props,
            })
        }
    }
}

fn merged_callback_to_ref(callback: MergedRefCallback<Element>) -> RefCallback<Element> {
    Rc::new(move |instance: Option<&Element>| {
        callback(instance);
        None
    })
}

impl RenderedElement {
    /// Materializes the description into a real DOM element — the seam standing in
    /// for React rendering the returned `ReactElement` upstream. Applies `class`,
    /// `style`, the lazy attributes, attaches the handler slots, and fires the
    /// forked ref with the element. Returns the listener cleanup for the view
    /// layer's owner-scoped teardown (upstream React owns unmount cleanup for its
    /// rendered elements); the ref's detach call rides the same teardown. Tests
    /// that keep the element alive hold or forget the cleanup.
    pub fn create_element(&self) -> (Element, Option<CleanupFn>) {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        let dom_element = document
            .create_element(self.tag.as_str())
            .expect("create_element");

        if let Some(class) = &self.props.class {
            dom_element
                .set_attribute("class", class)
                .expect("set class");
        }
        if !self.props.style.is_empty() {
            let style = self
                .props
                .style
                .iter()
                .map(|(property, value)| format!("{property}: {value};"))
                .collect::<Vec<_>>()
                .join(" ");
            dom_element
                .set_attribute("style", &style)
                .expect("set style");
        }
        for (name, value) in &self.props.handlers.attributes {
            if let Some(value) = value() {
                dom_element
                    .set_attribute(name, &value)
                    .expect("set attribute");
            }
        }
        let cleanup = self.props.handlers.attach_to(&dom_element);
        if let Some(ref_callback) = &self.props.ref_callback {
            ref_callback(Some(&dom_element));
        }
        (dom_element, cleanup)
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use reactive_graph::owner::Owner;

    use super::*;

    // `mergeClassNames` (`mergeProps.ts:276-290`): the later (rightmost) class comes
    // first in the string, single sides pass through, and empty/absent classes keep
    // the other side.
    #[test]
    fn merge_class_names_concatenates_with_the_later_class_first() {
        assert_eq!(
            merge_class_names(Some("internal".to_string()), Some("external".to_string())),
            Some("external internal".to_string()),
            "the later class is prepended"
        );
        assert_eq!(
            merge_class_names(None, Some("external".to_string())),
            Some("external".to_string()),
            "an absent earlier class passes the later through"
        );
        assert_eq!(
            merge_class_names(Some("internal".to_string()), None),
            Some("internal".to_string()),
            "an absent later class keeps the earlier"
        );
        assert_eq!(
            merge_class_names(None, None),
            None,
            "both absent stays absent"
        );
    }

    // `mergeObjects` over style (`mergeProps.ts:166-172`): the later bag's
    // declarations win per property; non-conflicting ones are retained.
    #[test]
    fn merge_styles_resolves_conflicts_in_favor_of_the_later_bag() {
        let merged = merge_styles(
            vec![
                ("padding".to_string(), "10px".to_string()),
                ("color".to_string(), "red".to_string()),
            ],
            vec![("color".to_string(), "blue".to_string())],
        );
        assert_eq!(
            merged,
            vec![
                ("padding".to_string(), "10px".to_string()),
                ("color".to_string(), "blue".to_string()),
            ],
            "the later declaration wins the conflicted property, the other survives"
        );
    }

    // `mergeEventHandlers` (`mergeProps.ts:229-249`): the later handler runs first;
    // the earlier handler is skipped once the dispatch is marked — and runs when it
    // is not.
    #[test]
    fn merge_event_handlers_runs_the_later_handler_first_and_gates_the_earlier_on_the_mark() {
        let order: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

        let ours: ElementEventHandler<BaseUIEvent<String>> = {
            let order = Rc::clone(&order);
            Rc::new(move |_| order.borrow_mut().push("ours"))
        };
        let theirs: ElementEventHandler<BaseUIEvent<String>> = {
            let order = Rc::clone(&order);
            Rc::new(move |event| {
                order.borrow_mut().push("theirs");
                if event.inner() == "prevent" {
                    event.prevent_base_ui_handler();
                }
            })
        };

        let merged = merge_event_handlers(Some(ours), Some(theirs));
        let event = BaseUIEvent::new("plain".to_string());
        merged.as_ref().unwrap()(&event);
        assert_eq!(
            *order.borrow(),
            vec!["theirs", "ours"],
            "right-to-left execution without the mark"
        );

        order.borrow_mut().clear();
        let prevented = BaseUIEvent::new("prevent".to_string());
        merged.as_ref().unwrap()(&prevented);
        assert_eq!(
            *order.borrow(),
            vec!["theirs"],
            "the mark set by the later handler skips the earlier one"
        );
    }

    // `mergeEventHandlers` with one side absent (`mergeProps.ts:222-227`): the
    // surviving handler passes through unwrapped.
    #[test]
    fn merge_event_handlers_passes_single_sided_handlers_through() {
        let calls: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let ours: ElementEventHandler<BaseUIEvent<String>> = {
            let calls = Rc::clone(&calls);
            Rc::new(move |_| *calls.borrow_mut() += 1)
        };
        let merged = merge_event_handlers(Some(ours), None);
        merged.as_ref().unwrap()(&BaseUIEvent::new(String::new()));
        assert_eq!(*calls.borrow(), 1, "the earlier handler alone still runs");
    }

    // `mergePropsN`'s plain-attribute precedence (`mergeProps.ts:14-15`) and the
    // ref rule (`:33` — "ref is not merged"; a later bag replaces only when it
    // carries one).
    #[test]
    fn merge_props_n_later_bags_win_attributes_and_only_replace_carried_refs() {
        let _owner = Owner::new();
        _owner.set();

        let early_ref: MergedRefCallback<Element> = Rc::new(|_: Option<&Element>| {});
        let late_ref: MergedRefCallback<Element> = Rc::new(|_: Option<&Element>| {});
        let late = RenderElementProps {
            handlers: RenderElementHandlers {
                attributes: vec![("data-late".to_string(), static_attr("late".to_string()))],
                ..RenderElementHandlers::default()
            },
            class: Some("late".to_string()),
            style: vec![("color".to_string(), "blue".to_string())],
            ref_callback: Some(Rc::clone(&late_ref)),
        };
        let early = RenderElementProps {
            handlers: RenderElementHandlers {
                attributes: vec![
                    ("data-early".to_string(), static_attr("early".to_string())),
                    (
                        "data-late".to_string(),
                        static_attr("superseded".to_string()),
                    ),
                ],
                ..RenderElementHandlers::default()
            },
            class: Some("early".to_string()),
            style: vec![("color".to_string(), "red".to_string())],
            ref_callback: Some(early_ref),
        };

        let merged = merge_props_n(vec![PropsSource::Static(early), PropsSource::Static(late)]);
        assert_eq!(
            merged.class,
            Some("late early".to_string()),
            "class concatenates later-first"
        );
        assert_eq!(
            merged.style,
            vec![("color".to_string(), "blue".to_string())],
            "the later style wins per key"
        );
        let late_value = merged
            .handlers
            .attributes
            .iter()
            .find(|(name, _)| name == "data-late")
            .and_then(|(_, value)| value())
            .expect("the later bag's attribute wins");
        assert_eq!(
            late_value, "late",
            "the later bag's attribute replaces the earlier's"
        );
        assert!(
            merged
                .handlers
                .attributes
                .iter()
                .any(|(name, _)| name == "data-early"),
            "the earlier bag's non-conflicted attribute survives"
        );
        assert!(
            Rc::ptr_eq(merged.ref_callback.as_ref().unwrap(), &late_ref),
            "the last ref-carrying bag's ref wins"
        );
    }

    // The getter form (`mergeProps.ts:210-219`): resolved wholesale against the
    // merged-so-far props and replacing them — and the `enabled: false`
    // short-circuit never resolves it (`useRenderElement.test.tsx:200-209`).
    #[test]
    fn getter_bags_replace_wholesale_and_are_skipped_when_disabled() {
        let _owner = Owner::new();
        _owner.set();

        let getter_calls: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let getter: RenderPropsGetter = {
            let calls = Rc::clone(&getter_calls);
            Rc::new(move |previous: &RenderElementProps| {
                *calls.borrow_mut() += 1;
                assert_eq!(
                    previous.class.as_deref(),
                    Some("base"),
                    "the getter receives the merged-so-far props"
                );
                RenderElementProps {
                    class: Some("from-getter".to_string()),
                    ..RenderElementProps::default()
                }
            })
        };

        let base = RenderElementProps {
            class: Some("base".to_string()),
            ..RenderElementProps::default()
        };
        let merged = merge_props_n(vec![PropsSource::Static(base), PropsSource::Getter(getter)]);
        assert_eq!(
            merged.class,
            Some("from-getter".to_string()),
            "the getter's props replace the accumulated ones wholesale"
        );
        assert_eq!(
            *getter_calls.borrow(),
            1,
            "the getter resolved exactly once"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use reactive_graph::owner::Owner;
    use reactive_graph::traits::{SetValue, WithValue};
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::MouseEvent;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn state(pairs: &[(&str, serde_json::Value)]) -> serde_json::Map<String, serde_json::Value> {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.clone()))
            .collect()
    }

    fn static_props(attributes: &[(&str, &str)]) -> RenderElementProps {
        RenderElementProps {
            handlers: RenderElementHandlers {
                attributes: attributes
                    .iter()
                    .map(|(name, value)| (name.to_string(), static_attr(value.to_string())))
                    .collect(),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        }
    }

    /// An owner-scoped [`use_render_element`] call materialized into the document.
    fn render_case(
        tag: &str,
        component_props: UseRenderElementComponentProps,
        params: UseRenderElementParams<'_>,
    ) -> Option<Element> {
        let owner = Owner::new();
        owner.set();
        let element = use_render_element(tag, component_props, params);
        owner.with(|| {
            element.map(|rendered| {
                let (dom, cleanup) = rendered.create_element();
                document().body().unwrap().append_child(&dom).unwrap();
                // The element lives in the document for the whole test run, so the
                // listener cleanup is held for the same lifetime (never dropped).
                std::mem::forget(cleanup);
                dom
            })
        })
    }

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn attribute(element: &Element, name: &str) -> Option<String> {
        element.get_attribute(name)
    }

    // The className function resolves against the state and merges with the
    // intrinsic bag's class — the component prop prepends
    // (`useRenderElement.test.tsx:90-101`, where the result is
    // 'active-class test-component').
    #[wasm_bindgen_test]
    fn class_function_resolves_against_state_and_merges_with_the_bag_class() {
        let component_props = UseRenderElementComponentProps {
            class_name: Some(ClassNameSource::Function(Rc::new(
                |state: &serde_json::Map<String, serde_json::Value>| {
                    state
                        .get("active")
                        .and_then(|value| value.as_bool())
                        .map(|active| {
                            if active {
                                "active-class"
                            } else {
                                "inactive-class"
                            }
                            .to_string()
                        })
                },
            ))),
            ..UseRenderElementComponentProps::default()
        };
        let params = UseRenderElementParams {
            state: &state(&[("active", serde_json::Value::Bool(true))]),
            props: vec![PropsSource::Static(static_props(&[("id", "target")]))],
            ..UseRenderElementParams::default()
        };
        // The intrinsic bag's class rides `class`, the bag-merge analog of the
        // upstream `className: 'test-component'` member of `props[0]`.
        let mut bag = static_props(&[("id", "target")]);
        bag.class = Some("test-component".to_string());
        let params = UseRenderElementParams {
            props: vec![PropsSource::Static(bag)],
            ..params
        };

        let element = render_case("div", component_props, params).unwrap();
        assert_eq!(
            attribute(&element, "class"),
            Some("active-class test-component".to_string()),
            "the component class resolves from the state and prepends the bag's"
        );
    }

    // A class function returning undefined falls back to the internal value
    // (`useRenderElement.test.tsx:103-111`).
    #[wasm_bindgen_test]
    fn class_function_returning_none_falls_back_to_the_bag_class() {
        let component_props = UseRenderElementComponentProps {
            class_name: Some(ClassNameSource::Function(Rc::new(
                |state: &serde_json::Map<String, serde_json::Value>| {
                    state
                        .get("active")
                        .and_then(|value| value.as_bool())
                        .filter(|active| *active)
                        .map(|_| "active-class".to_string())
                },
            ))),
            ..UseRenderElementComponentProps::default()
        };
        let mut bag = static_props(&[]);
        bag.class = Some("test-component".to_string());
        let params = UseRenderElementParams {
            state: &state(&[("active", serde_json::Value::Bool(false))]),
            props: vec![PropsSource::Static(bag)],
            ..UseRenderElementParams::default()
        };

        let element = render_case("div", component_props, params).unwrap();
        assert_eq!(
            attribute(&element, "class"),
            Some("test-component".to_string())
        );
    }

    // The style function merges with the bag's style per property
    // (`useRenderElement.test.tsx:113-124`).
    #[wasm_bindgen_test]
    fn style_function_merges_with_the_bag_style() {
        let component_props = UseRenderElementComponentProps {
            style: Some(StyleSource::Function(Rc::new(
                |state: &serde_json::Map<String, serde_json::Value>| {
                    state
                        .get("active")
                        .and_then(|value| value.as_bool())
                        .map(|active| {
                            vec![(
                                "color".to_string(),
                                if active {
                                    "rgb(255,0,0)"
                                } else {
                                    "rgb(0,255,0)"
                                }
                                .to_string(),
                            )]
                        })
                },
            ))),
            ..UseRenderElementComponentProps::default()
        };
        let mut bag = static_props(&[]);
        bag.style = vec![("padding".to_string(), "10px".to_string())];
        let params = UseRenderElementParams {
            state: &state(&[("active", serde_json::Value::Bool(true))]),
            props: vec![PropsSource::Static(bag)],
            ..UseRenderElementParams::default()
        };

        let element = render_case("div", component_props, params).unwrap();
        let style = attribute(&element, "style").unwrap();
        assert!(
            style.contains("padding: 10px"),
            "the bag's declaration survives: {style}"
        );
        assert!(
            style.contains("color: rgb(255,0,0)"),
            "the function's declaration merges: {style}"
        );
    }

    // State renders as `data-*` attributes: a bare `true` emits the empty value, a
    // truthy non-boolean stringifies (`getStateAttributesProps.ts:24-28` via the
    // upstream suite's `state={{ active }}` rendering).
    #[wasm_bindgen_test]
    fn state_renders_data_attributes() {
        let params = UseRenderElementParams {
            state: &state(&[
                ("active", serde_json::Value::Bool(true)),
                (
                    "orientation",
                    serde_json::Value::String("horizontal".to_string()),
                ),
                ("inactive", serde_json::Value::Bool(false)),
            ]),
            ..UseRenderElementParams::default()
        };

        let element =
            render_case("div", UseRenderElementComponentProps::default(), params).unwrap();
        assert_eq!(
            attribute(&element, "data-active"),
            Some(String::new()),
            "true emits the empty value"
        );
        assert_eq!(
            attribute(&element, "data-orientation"),
            Some("horizontal".to_string()),
            "a truthy string stringifies"
        );
        assert_eq!(
            attribute(&element, "data-inactive"),
            None,
            "falsy emits nothing"
        );
    }

    // `enabled: false` renders nothing and does not resolve prop bags
    // (`useRenderElement.test.tsx:200-209`).
    #[wasm_bindgen_test]
    fn disabled_renders_nothing_and_skips_prop_resolution() {
        let getter_calls: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let getter: RenderPropsGetter = {
            let calls = Rc::clone(&getter_calls);
            Rc::new(move |_| {
                *calls.borrow_mut() += 1;
                RenderElementProps::default()
            })
        };
        let params = UseRenderElementParams {
            enabled: false,
            props: vec![PropsSource::Getter(getter)],
            ..UseRenderElementParams::default()
        };

        let element = render_case("div", UseRenderElementComponentProps::default(), params);
        assert!(element.is_none(), "the hook yields null");
        assert_eq!(*getter_calls.borrow(), 0, "prop bags are not resolved");
    }

    // The merged ref fires with the materialized element for every branch — the
    // bag ref, the render-element ref, and the caller's array refs
    // (`useRenderElement.test.tsx:237-268` single-call form and `:417-428`).
    #[wasm_bindgen_test]
    fn merged_refs_all_observe_the_materialized_element() {
        use reactive_graph::owner::{LocalStorage, StoredValue};

        let bag_slot: StoredValue<Option<Element>, LocalStorage> = StoredValue::new_local(None);
        let caller_slot: StoredValue<Option<Element>, LocalStorage> = StoredValue::new_local(None);

        let bag_write = bag_slot.clone();
        let bag = RenderElementProps {
            ref_callback: Some(Rc::new(move |instance: Option<&Element>| {
                let _ = bag_write.try_set_value(instance.cloned());
            })),
            ..RenderElementProps::default()
        };
        let params = UseRenderElementParams {
            refs: vec![InputRef::Object(Rc::new(caller_slot.clone()))],
            props: vec![PropsSource::Static(bag)],
            ..UseRenderElementParams::default()
        };

        let element =
            render_case("div", UseRenderElementComponentProps::default(), params).unwrap();
        assert_eq!(
            bag_slot.with_value(|slot| slot.clone()),
            Some(element.clone()),
            "the bag's ref observes the element"
        );
        assert_eq!(
            caller_slot.with_value(|slot| slot.clone()),
            Some(element),
            "the caller's ref observes the element"
        );
    }

    // A single external `onMouseDown` is preventable — the handler can call
    // `preventBaseUIHandler()` and the internal-bag handler is skipped
    // (`useRenderElement.test.tsx:136-149`).
    #[wasm_bindgen_test]
    fn external_handler_can_prevent_the_internal_handler() {
        let internal_calls: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let external_calls: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let prevent: Rc<std::cell::Cell<bool>> = Rc::new(std::cell::Cell::new(true));

        let mut internal = static_props(&[("id", "target")]);
        internal.handlers.on_mouse_down = Some({
            let calls = Rc::clone(&internal_calls);
            Rc::new(move |_: &BaseUIEvent<MouseEvent>| *calls.borrow_mut() += 1)
        });
        let external = RenderElementProps {
            handlers: RenderElementHandlers {
                on_mouse_down: Some({
                    let calls = Rc::clone(&external_calls);
                    let prevent = Rc::clone(&prevent);
                    Rc::new(move |event: &BaseUIEvent<MouseEvent>| {
                        *calls.borrow_mut() += 1;
                        if prevent.get() {
                            event.prevent_base_ui_handler();
                        }
                    })
                }),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };
        let params = UseRenderElementParams {
            props: vec![PropsSource::Static(internal), PropsSource::Static(external)],
            ..UseRenderElementParams::default()
        };

        let element =
            render_case("div", UseRenderElementComponentProps::default(), params).unwrap();
        element
            .dispatch_event(&MouseEvent::new("mousedown").unwrap())
            .unwrap();
        assert_eq!(*external_calls.borrow(), 1, "the external handler ran");
        assert_eq!(
            *internal_calls.borrow(),
            0,
            "the internal handler was prevented"
        );

        prevent.set(false);
        element
            .dispatch_event(&MouseEvent::new("mousedown").unwrap())
            .unwrap();
        assert_eq!(
            *external_calls.borrow(),
            2,
            "the external handler ran again"
        );
        assert_eq!(
            *internal_calls.borrow(),
            1,
            "unmarked dispatches reach the internal handler"
        );
    }

    // An "obscure" event (`onContextMenu`) is preventable through the same
    // composition (`useRenderElement.test.tsx:166-181`).
    #[wasm_bindgen_test]
    fn obscure_events_are_preventable() {
        let internal_calls: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let mut internal = static_props(&[]);
        internal.handlers.on_context_menu = Some({
            let calls = Rc::clone(&internal_calls);
            Rc::new(move |_: &BaseUIEvent<MouseEvent>| *calls.borrow_mut() += 1)
        });
        let external = RenderElementProps {
            handlers: RenderElementHandlers {
                on_context_menu: Some(Rc::new(|event: &BaseUIEvent<MouseEvent>| {
                    event.prevent_base_ui_handler();
                })),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };
        let params = UseRenderElementParams {
            props: vec![PropsSource::Static(internal), PropsSource::Static(external)],
            ..UseRenderElementParams::default()
        };

        let element =
            render_case("div", UseRenderElementComponentProps::default(), params).unwrap();
        element
            .dispatch_event(&MouseEvent::new("contextmenu").unwrap())
            .unwrap();
        assert_eq!(
            *internal_calls.borrow(),
            0,
            "the prevented internal handler never ran"
        );
    }

    // A render function receives the merged props and the state and its returned
    // element replaces the default tag (`useRenderElement.test.tsx:271-295`).
    #[wasm_bindgen_test]
    fn render_function_receives_merged_props_and_state_and_overrides_the_tag() {
        let seen_props: Rc<RefCell<Option<RenderElementProps>>> = Rc::new(RefCell::new(None));
        let seen_state: Rc<RefCell<Option<serde_json::Map<String, serde_json::Value>>>> =
            Rc::new(RefCell::new(None));
        let render: RenderFn = {
            let seen_props = Rc::clone(&seen_props);
            let seen_state = Rc::clone(&seen_state);
            Rc::new(
                move |props: RenderElementProps,
                      state: &serde_json::Map<String, serde_json::Value>| {
                    *seen_props.borrow_mut() = Some(props.clone());
                    *seen_state.borrow_mut() = Some(state.clone());
                    RenderedElement {
                        tag: "span".to_string(),
                        props,
                    }
                },
            )
        };
        let mut bag = static_props(&[("data-testid", "custom")]);
        bag.class = Some("test-component".to_string());
        let params = UseRenderElementParams {
            state: &state(&[("active", serde_json::Value::Bool(true))]),
            props: vec![PropsSource::Static(bag)],
            ..UseRenderElementParams::default()
        };

        let element = render_case(
            "div",
            UseRenderElementComponentProps {
                render: Some(RenderProp::Function(render)),
                ..UseRenderElementComponentProps::default()
            },
            params,
        )
        .unwrap();

        let props = seen_props.borrow().clone().unwrap();
        assert_eq!(
            props.class,
            Some("test-component".to_string()),
            "the render fn receives the merged bag (class)"
        );
        let testid = props
            .handlers
            .attributes
            .iter()
            .find(|(name, _)| name == "data-testid")
            .and_then(|(_, value)| value())
            .expect("the merged bag carries data-testid");
        assert_eq!(
            testid, "custom",
            "the render fn receives the merged bag (attributes)"
        );
        assert_eq!(
            seen_state.borrow().as_ref().unwrap().get("active"),
            Some(&serde_json::Value::Bool(true)),
            "the render fn receives the state"
        );
        assert_eq!(
            element.tag_name(),
            "SPAN",
            "the render fn's element replaces the default tag"
        );
        assert_eq!(
            attribute(&element, "data-testid"),
            Some("custom".to_string())
        );
    }

    // A render element: its tag governs, its own plain attributes win over the
    // merged bags, its class merges, and its own ref joins the fork
    // (`useRenderElement.test.tsx:399-428`, `:531-548`, `:634-649`).
    #[wasm_bindgen_test]
    fn render_element_merges_with_lower_handler_precedence_and_its_own_attribute_precedence() {
        use reactive_graph::owner::{LocalStorage, StoredValue};

        let render_ref_slot: StoredValue<Option<Element>, LocalStorage> =
            StoredValue::new_local(None);
        let render_element = RenderProp::Element {
            tag: "span".to_string(),
            props: RenderElementProps {
                handlers: RenderElementHandlers {
                    attributes: vec![
                        ("data-slot".to_string(), static_attr("render".to_string())),
                        ("data-shared".to_string(), static_attr("render".to_string())),
                    ],
                    ..RenderElementHandlers::default()
                },
                class: Some("render-class".to_string()),
                ref_callback: Some({
                    let render_ref_slot = render_ref_slot.clone();
                    Rc::new(move |instance: Option<&Element>| {
                        let _ = render_ref_slot.try_set_value(instance.cloned());
                    })
                }),
                ..RenderElementProps::default()
            },
        };
        let bag = static_props(&[("data-shared", "bag")]);
        let caller_slot: StoredValue<Option<Element>, LocalStorage> = StoredValue::new_local(None);
        let params = UseRenderElementParams {
            refs: vec![InputRef::Object(Rc::new(caller_slot.clone()))],
            props: vec![PropsSource::Static(bag)],
            ..UseRenderElementParams::default()
        };

        let element = render_case(
            "div",
            UseRenderElementComponentProps {
                render: Some(render_element),
                ..UseRenderElementComponentProps::default()
            },
            params,
        )
        .unwrap();

        assert_eq!(
            element.tag_name(),
            "SPAN",
            "the render element's tag governs"
        );
        assert_eq!(
            attribute(&element, "data-slot"),
            Some("render".to_string()),
            "the render element's own attributes win"
        );
        assert_eq!(
            attribute(&element, "data-shared"),
            Some("render".to_string()),
            "mergeProps(props, render.props): the render element is the later argument"
        );
        assert_eq!(
            render_ref_slot.with_value(|slot| slot.clone()),
            Some(element.clone()),
            "the render element's own ref observes the element"
        );
        assert_eq!(
            caller_slot.with_value(|slot| slot.clone()),
            Some(element),
            "the caller's ref observes the same element"
        );
    }

    // The default-tag path forces `type="button"` on buttons, overridable by a bag
    // (`renderTag`, `useRenderElement.tsx:232-240`).
    #[wasm_bindgen_test]
    fn button_defaults_to_type_button_and_img_to_alt_empty() {
        let button = render_case(
            "button",
            UseRenderElementComponentProps::default(),
            UseRenderElementParams::default(),
        )
        .unwrap();
        assert_eq!(attribute(&button, "type"), Some("button".to_string()));

        let overridden = static_props(&[("type", "submit")]);
        let submit = render_case(
            "button",
            UseRenderElementComponentProps::default(),
            UseRenderElementParams {
                props: vec![PropsSource::Static(overridden)],
                ..UseRenderElementParams::default()
            },
        )
        .unwrap();
        assert_eq!(
            attribute(&submit, "type"),
            Some("submit".to_string()),
            "a bag's type wins"
        );

        let img = render_case(
            "img",
            UseRenderElementComponentProps::default(),
            UseRenderElementParams::default(),
        )
        .unwrap();
        assert_eq!(attribute(&img, "alt"), Some(String::new()));
    }

    // Later bags win per key — the `[defaultProps, ...props, elementProps]` spread
    // order the component layer relies on (`useRenderElement.tsx:69`).
    #[wasm_bindgen_test]
    fn later_bags_win_per_attribute_key() {
        let params = UseRenderElementParams {
            props: vec![
                PropsSource::Static(static_props(&[("data-which", "first"), ("id", "first")])),
                PropsSource::Static(static_props(&[("data-which", "last")])),
            ],
            ..UseRenderElementParams::default()
        };

        let element =
            render_case("div", UseRenderElementComponentProps::default(), params).unwrap();
        assert_eq!(attribute(&element, "data-which"), Some("last".to_string()));
        assert_eq!(
            attribute(&element, "id"),
            Some("first".to_string()),
            "non-conflicted keys survive"
        );
    }

    // Handlers compose right-to-left across the whole bag list: the last bag's
    // handler runs first, the first bag's last — and the fork fires every ref
    // branch once the element materializes.
    #[wasm_bindgen_test]
    fn handlers_compose_right_to_left_across_bags() {
        let order: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
        let first = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some({
                    let order = Rc::clone(&order);
                    Rc::new(move |_: &BaseUIEvent<MouseEvent>| order.borrow_mut().push("first"))
                }),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };
        let second = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some({
                    let order = Rc::clone(&order);
                    Rc::new(move |_: &BaseUIEvent<MouseEvent>| order.borrow_mut().push("second"))
                }),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };
        let params = UseRenderElementParams {
            props: vec![PropsSource::Static(first), PropsSource::Static(second)],
            ..UseRenderElementParams::default()
        };

        let element =
            render_case("div", UseRenderElementComponentProps::default(), params).unwrap();
        element
            .dispatch_event(&MouseEvent::new("click").unwrap())
            .unwrap();
        assert_eq!(
            *order.borrow(),
            vec!["second", "first"],
            "right-to-left execution"
        );
    }
}
