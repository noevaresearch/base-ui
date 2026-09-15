//! `Checkbox.Indicator` — port of
//! `packages/react/src/checkbox/indicator/CheckboxIndicator.tsx`
//! (`specs/library/checkbox/behavior.md`, `specs/library/checkbox/implementation.md`).
//!
//! A pure leaf (implementation.md, "DOM/portal strategy"): the root's state plus its
//! own `transitionStatus`, rendered as a `span` with no DOM structure of its own
//! beyond the consumer's `render`/children (`CheckboxIndicator.tsx:59-68`). The mount
//! machine is the shared pair — `useTransitionStatus(rendered)` where `rendered =
//! rootState.checked || rootState.indeterminate` (`:27-29`) and
//! `useOpenChangeComplete({ batch: true, enabled: !rendered, open: rendered, ref:
//! indicatorRef, onComplete: if (!rendered) setMounted(false) })` (`:38-48`), whose
//! `batch: true` is precisely the mechanism behind behavior.md's "many simultaneously
//! unchecked indicators are removed in one React commit" (`:315-379`).
//!
//! Runtime law: the two hooks are the internals crate's rg-0.2 ports, so the port runs
//! them inside a dedicated rg-0.2 owner and mirrors every crossing value (the
//! `field::validation_helpers::transition_status_signal` bridge shape — one effect per
//! runtime, each created inside its own owner).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::children::ChildrenFn;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use leptos_ui_internals::floating_ui::element_props::ElementEventHandler;
use leptos_ui_internals::state_attributes::{StateAttributeProps, transition_status_mapping};
use leptos_ui_internals::use_open_change_complete::{
    UseOpenChangeCompleteParams, use_open_change_complete,
};
use leptos_ui_internals::use_transition_status::{TransitionStatus, use_transition_status};

use crate::checkbox::root::{CheckboxRootContextValue, use_checkbox_root_context};
use crate::checkbox::state::{
    MANAGED_STATE_ATTRIBUTES, checkbox_state_attributes, indicator_rendered,
    indicator_should_render,
};

/// The Indicator's event handlers (`CheckboxIndicatorProps`, the `...elementProps`
/// rest's handler members this unit's tests exercise): the exit-animation hooks
/// (`CheckboxIndicator.test.tsx:141-194`, `:201-263`).
#[derive(Clone, Default)]
pub struct CheckboxIndicatorHandlers {
    /// `onAnimationEnd`.
    pub on_animation_end: Option<ElementEventHandler<web_sys::AnimationEvent>>,
    /// `onTransitionEnd`.
    pub on_transition_end: Option<ElementEventHandler<web_sys::TransitionEvent>>,
}

/// The Indicator props — upstream's destructured set (`CheckboxIndicator.tsx:23`).
pub struct CheckboxIndicatorViewProps {
    /// `keepMounted` (`:23`, default `false`).
    pub keep_mounted: bool,
    /// `className` (`:23`).
    pub class: Option<String>,
    /// `style` — ordered declarations.
    pub style: Vec<(String, String)>,
    /// The `...elementProps` rest (`:23`) — plain attributes.
    pub element_attributes: Vec<(String, String)>,
    /// The rest's handlers.
    pub handlers: CheckboxIndicatorHandlers,
    /// The consumer's children — `ChildrenFn` (not the boxed `FnOnce` form) so the
    /// `shouldRender` gate can re-emit them on each mount transition (the accordion
    /// panel / `Field.Label` convention).
    pub children: Option<ChildrenFn>,
}

impl Default for CheckboxIndicatorViewProps {
    fn default() -> Self {
        Self {
            keep_mounted: false,
            class: None,
            style: Vec::new(),
            element_attributes: Vec::new(),
            handlers: CheckboxIndicatorHandlers::default(),
            children: None,
        }
    }
}

/// The bridged transition machine: the leptos-tracked mirrors of the rg-0.2 hooks'
/// outputs plus the two input mirrors.
struct IndicatorTransition {
    /// The tracked `transitionStatus` (`CheckboxIndicator.tsx:29`).
    status: RwSignal<Option<TransitionStatus>>,
    /// The tracked `mounted` (`:29`) — the render gate's input.
    mounted: RwSignal<bool>,
    /// The rg-0.2 `mounted` writable (`setMounted`).
    mounted_rg: reactive_graph::signal::RwSignal<bool>,
}

/// Runs `useTransitionStatus(rendered)` + `useOpenChangeComplete({ batch: true, … })`
/// inside a dedicated rg-0.2 owner and mirrors their outputs into leptos signals.
fn use_indicator_transition(
    rendered: Signal<bool>,
    indicator_element: Rc<Cell<Option<web_sys::Element>>>,
) -> IndicatorTransition {
    let open_rg = reactive_graph::signal::RwSignal::new(rendered.get_untracked());
    let enabled_rg = reactive_graph::signal::RwSignal::new(!rendered.get_untracked());
    let batch_rg = reactive_graph::signal::RwSignal::new(true);

    let status = RwSignal::new(None::<TransitionStatus>);
    let mounted = RwSignal::new(rendered.get_untracked());

    let hook_owner = reactive_graph::owner::Owner::new();
    let (mounted_rg, transition_rg) = hook_owner.with(|| {
        let hook = use_transition_status(
            open_rg.clone(),
            // `enableIdleState` / `deferEndingState` default to `false` upstream
            // (`useTransitionStatus.ts:92-96`'s parameter defaults).
            reactive_graph::signal::RwSignal::new(false),
            reactive_graph::signal::RwSignal::new(false),
            false,
        );

        // `useOpenChangeComplete` (`CheckboxIndicator.tsx:38-48`): `enabled: !rendered`,
        // `open: rendered`, `batch: true`, and the completion that unmounts a closed
        // indicator.
        let open_for_complete = open_rg.clone();
        let mounted_for_complete = hook.mounted.clone();
        use_open_change_complete(UseOpenChangeCompleteParams {
            enabled: enabled_rg.clone(),
            open: open_rg.clone(),
            reference: {
                let indicator_element = Rc::clone(&indicator_element);
                move || {
                    let element = indicator_element.replace(None);
                    indicator_element.set(element.clone());
                    element
                }
            },
            batch: batch_rg.clone(),
            on_complete: Rc::new(move || {
                if !reactive_graph::traits::GetUntracked::get_untracked(&open_for_complete) {
                    reactive_graph::traits::Set::set(&mounted_for_complete, false);
                }
            }),
        });

        (hook.mounted, hook.transition_status)
    });
    std::mem::forget(hook_owner);

    // Seed the leptos mirrors from the hooks' initializers (read untracked at hook time).
    status.set(reactive_graph::traits::GetUntracked::get_untracked(
        &transition_rg,
    ));
    mounted.set(reactive_graph::traits::GetUntracked::get_untracked(
        &mounted_rg,
    ));

    // leptos → rg-0.2: the hook's `open`/`enabled` inputs follow the tracked state.
    Effect::new({
        let open_rg = open_rg.clone();
        let enabled_rg = enabled_rg.clone();
        move |_| {
            let rendered = rendered.get();
            reactive_graph::traits::Set::set(&open_rg, rendered);
            reactive_graph::traits::Set::set(&enabled_rg, !rendered);
        }
    });

    // rg-0.2 → leptos: the two outputs the view reads.
    reactive_graph::effect::Effect::new({
        let status = status.clone();
        let transition_rg = transition_rg.clone();
        move |_| {
            status.set(reactive_graph::traits::Get::get(&transition_rg));
        }
    });
    reactive_graph::effect::Effect::new({
        let mounted = mounted.clone();
        let mounted_rg = mounted_rg.clone();
        move |_| {
            mounted.set(reactive_graph::traits::Get::get(&mounted_rg));
        }
    });

    IndicatorTransition {
        status,
        mounted,
        mounted_rg,
    }
}

/// The Indicator view — upstream's `CheckboxIndicator` body
/// (`CheckboxIndicator.tsx:19-71`).
pub fn checkbox_indicator_view(props: CheckboxIndicatorViewProps) -> impl IntoView {
    let bridge_owner = reactive_graph::owner::Owner::new();
    let view = bridge_owner.with(move || checkbox_indicator_body(props));
    std::mem::forget(bridge_owner);
    view
}

fn checkbox_indicator_body(props: CheckboxIndicatorViewProps) -> impl IntoView {
    let CheckboxIndicatorViewProps {
        keep_mounted,
        class,
        style,
        element_attributes,
        handlers,
        children,
    } = props;

    // `useCheckboxRootContext()` (`:25`) — throws the upstream message outside a Root
    // (`CheckboxIndicator.test.tsx:37-47`).
    let root: CheckboxRootContextValue = use_checkbox_root_context();

    // `rendered = rootState.checked || rootState.indeterminate` (`:27`).
    let rendered: Signal<bool> = {
        let root = root.clone();
        Signal::derive(move || indicator_rendered(root.checked.get(), root.indeterminate.get()))
    };

    // The state the indicator's own mapping walks (`:33-36`): the root state plus
    // `transitionStatus`.
    let indicator_node: NodeRef<leptos::html::Span> = NodeRef::new();
    let indicator_element: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    let transition = use_indicator_transition(rendered, Rc::clone(&indicator_element));

    // `shouldRender = keepMounted || mounted` (`:57`).
    let should_render: Signal<bool> = {
        let mounted = transition.mounted;
        Signal::derive(move || indicator_should_render(keep_mounted, mounted.get()))
    };

    // The ref-resync + the node's `ref` slot (`:31`'s `indicatorRef`).
    {
        let indicator_element = Rc::clone(&indicator_element);
        Effect::new(move |_| {
            let Some(span) = indicator_node.get() else {
                return;
            };
            indicator_element.set(Some(span.unchecked_into::<web_sys::Element>()));
        });
    }

    // The attribute writer (`:59-64`'s `useRenderElement` spread): the root state's
    // walk merged with `transitionStatusMapping` (`:50-55`).
    let managed_names: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let consumer_attribute_names: Vec<String> = element_attributes
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    {
        let root = root.clone();
        let status = transition.status;
        let managed_names = Rc::clone(&managed_names);
        let consumer_attribute_names = consumer_attribute_names.clone();
        let class = class.clone();
        let style = style.clone();
        Effect::new(move |_| {
            let Some(span) = indicator_node.get() else {
                return;
            };
            let span: &web_sys::Element = span.unchecked_ref();

            let state = root.tracked_snapshot();
            let walk = checkbox_state_attributes(&state);

            let mut attributes: Vec<(String, Option<String>)> = Vec::new();
            if let Some(class) = &class {
                attributes.push(("class".to_string(), Some(class.clone())));
            }
            if !style.is_empty() {
                attributes.push((
                    "style".to_string(),
                    Some(
                        style
                            .iter()
                            .map(|(property, value)| format!("{property}: {value};"))
                            .collect::<Vec<_>>()
                            .join(" "),
                    ),
                ));
            }
            for name in MANAGED_STATE_ATTRIBUTES {
                attributes.push((name.to_string(), walk.get(name).cloned()));
            }

            // `stateAttributesMapping = { ...baseStateAttributesMapping,
            // ...transitionStatusMapping }` (`:52-55`): the transition hooks ride the
            // same walk, over the `transitionStatus` member.
            let transition_attributes: StateAttributeProps = {
                let mut map = serde_json::Map::new();
                map.insert(
                    "transitionStatus".to_string(),
                    match status.get() {
                        Some(TransitionStatus::Starting) => serde_json::json!("starting"),
                        Some(TransitionStatus::Ending) => serde_json::json!("ending"),
                        Some(TransitionStatus::Idle) => serde_json::json!("idle"),
                        None => serde_json::Value::Null,
                    },
                );
                let mapping =
                    |key: &str, value: &serde_json::Value| transition_status_mapping(key, value);
                leptos_ui_internals::state_attributes::get_state_attributes_props(
                    &map,
                    Some(
                        &mapping
                            as &dyn Fn(
                                &str,
                                &serde_json::Value,
                            )
                                -> Option<Option<StateAttributeProps>>,
                    ),
                )
            };
            for (name, value) in transition_attributes {
                attributes.push((name, Some(value)));
            }

            for (name, value) in &element_attributes {
                attributes.push((name.clone(), Some(value.clone())));
            }

            let previous = managed_names.borrow().clone();
            let mut written: Vec<String> = Vec::new();
            for (name, value) in attributes {
                if let Some(value) = value {
                    let _ = span.set_attribute(&name, &value);
                    written.push(name);
                }
            }
            for name in previous {
                if !written.contains(&name) {
                    let _ = span.remove_attribute(&name);
                }
            }
            *managed_names.borrow_mut() = written;
            let _ = &consumer_attribute_names;
        });
    }

    // The rest's handlers (`onAnimationEnd`/`onTransitionEnd`) attach at mount — the
    // event names' DOM spelling (`CheckboxIndicator.test.tsx:174-179`, `:241-246`).
    {
        let handlers = handlers.clone();
        Effect::new(move |_| {
            let Some(span) = indicator_node.get() else {
                return;
            };
            let mut cleanups: Vec<Option<Box<dyn FnOnce()>>> = Vec::new();
            if let Some(handler) = handlers.on_animation_end.clone() {
                let unsubscribe = leptos_ui_utils::add_event_listener::add_event_listener(
                    &span,
                    "animationend",
                    move |event: &web_sys::Event| {
                        if let Some(typed) = event.dyn_ref::<web_sys::AnimationEvent>() {
                            handler(typed);
                        }
                    },
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
            }
            if let Some(handler) = handlers.on_transition_end.clone() {
                let unsubscribe = leptos_ui_utils::add_event_listener::add_event_listener(
                    &span,
                    "transitionend",
                    move |event: &web_sys::Event| {
                        if let Some(typed) = event.dyn_ref::<web_sys::TransitionEvent>() {
                            handler(typed);
                        }
                    },
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
            }
            // The merged unsubscribe lives for the element's lifetime (the listeners go
            // with the node); the `Option` dance makes the `FnOnce` callable from the
            // `FnMut` cleanup slot.
            let cleanup: Vec<Option<Box<dyn FnOnce()>>> = cleanups;
            let cleanup = leptos_ui_utils::merge_cleanups::merge_cleanups(cleanup);
            let cleanup: Box<dyn FnOnce()> = Box::new(cleanup);
            let cleanup = send_wrapper::SendWrapper::new(RefCell::new(Some(cleanup)));
            reactive_graph::owner::on_cleanup(move || {
                if let Some(cleanup) = cleanup.borrow_mut().take() {
                    cleanup();
                }
            });
        });
    }

    let children_view: Option<ChildrenFn> = children;

    // `if (!shouldRender) return null` (`:66-68`).
    let render_gate = {
        let children_view = children_view.clone();
        move || {
            should_render.get().then(|| {
                let children = children_view.as_ref().map(|children| children());
                view! {
                    <span node_ref=indicator_node>{children}</span>
                }
            })
        }
    };

    view! { {render_gate} }
}
