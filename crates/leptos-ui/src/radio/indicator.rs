//! `Radio.Indicator` — port of `packages/react/src/radio/indicator/RadioIndicator.tsx`
//! (the `library: radio` TODO item; `specs/library/radio/behavior.md`,
//! `specs/library/radio/implementation.md`).
//!
//! ## What upstream does
//!
//! The Indicator is "a pure function of `RadioRootState.checked` plus a small animation
//! status machine" (implementation.md's own words): `rendered` *is* the Root's `checked`
//! (`RadioIndicator.tsx:25`), with no local boolean — `mounted` and `transitionStatus` are
//! the only state, owned by `useTransitionStatus(rendered)` (`:27`).
//!
//! Two obligations follow from the spec, and both are asserted:
//!
//! - **Mount semantics** (behavior.md:26) — absent while its Root is unchecked, present
//!   while checked, removed once the Root becomes unchecked. `shouldRender = keepMounted ||
//!   mounted` (`:36`) with an early `return null` otherwise (`:57-59`) is the mechanism.
//! - **Unmount timing** (behavior.md:67) — removed immediately when no exit animation is
//!   defined, and only after `data-ending-style` finishes when one is.
//!   `useOpenChangeComplete({ batch: true, enabled: !rendered, open: rendered, … })`
//!   (`:45-55`) waits for the exit animation to finish (or fires immediately when none is
//!   defined) and then `setMounted(false)`.
//!
//! ## Documented adaptations (never silent)
//!
//! 1. **`view!` has no attribute spread** — the same writer effect as `radio::root`, and the
//!    state walk runs inside it so the transition hooks and the Root's state both track live
//!    signals. `state_attributes_mapping` is deliberately not handed to the render layer for
//!    the same reason: that layer folds a mapping once at description time.
//! 2. **The two hooks run in a dedicated reactive-graph 0.2 owner** and their outputs are
//!    mirrored into leptos signals, because `useTransitionStatus`/`useOpenChangeComplete` are
//!    typed over rg 0.2 while this crate's view tree tracks leptos 0.7's runtime — the
//!    `checkbox::indicator` bridge verbatim, one crossing at a time.
//! 3. **`ref`** (`:34`, `:39`) has no consumer-forwarding slot on the description path — the
//!    crate-wide `library: the view paths drop render's element form`. The internal
//!    `indicatorRef` slot *is* implemented: it is the `NodeRef` the view owns and the
//!    animation runner reads through (`:49`).
//! 4. **`onAnimationEnd` / `onTransitionEnd`** (behavior.md:60) ride the view's own event
//!    bindings rather than the internals' handler vocabulary, which has no animation slots.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use leptos_ui_internals::floating_ui::element_props::{ElementAttributeFn, ElementEventHandler};
use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::state_attributes::StateAttributeProps;
use leptos_ui_internals::use_open_change_complete::{
    UseOpenChangeCompleteParams, use_open_change_complete,
};
use leptos_ui_internals::use_render_element::{
    RenderElementHandlers, RenderElementProps, UseRenderElementComponentProps,
    UseRenderElementParams, use_render_element,
};
use leptos_ui_internals::use_transition_status::{TransitionStatus, use_transition_status};

use crate::radio::context::use_radio_root_context;
use crate::radio::state::{
    MANAGED_STATE_ATTRIBUTES, RadioIndicatorState, indicator_should_render,
    radio_indicator_state_attributes,
};

/// The Indicator props — upstream's `RadioIndicatorProps` (`RadioIndicator.tsx:64-70`) plus
/// the element-level members `useRenderElement` resolves from `componentProps` (`:21`).
#[derive(Default)]
pub struct RadioIndicatorViewProps {
    /// `keepMounted` (`:69`, default `false`) — keep the element in the DOM when the radio is
    /// inactive.
    pub keep_mounted: bool,
    /// `render`/`className`/`style` (`:21`).
    pub render_class_style: UseRenderElementComponentProps,
    /// The consumer's `...elementProps` rest (`:21`) as plain attributes.
    pub element_attributes: Vec<(String, String)>,
    /// The consumer's handlers out of the same rest.
    pub handlers: RadioIndicatorHandlers,
}

/// The consumer's handler members out of the Indicator's `...elementProps` rest (`:21`).
/// behavior.md:60 proves both: `onAnimationEnd` fires when the exit (`data-ending-style`)
/// animation finishes (`RadioIndicator.test.tsx:94`, `:113-115`) and `onTransitionEnd` fires
/// when the enter transition driven by `data-starting-style` completes (`:159`, `:175-177`).
#[derive(Clone, Default)]
pub struct RadioIndicatorHandlers {
    /// `onAnimationEnd` (`:94`).
    pub on_animation_end: Option<ElementEventHandler<web_sys::AnimationEvent>>,
    /// `onTransitionEnd` (`:159`).
    pub on_transition_end: Option<ElementEventHandler<web_sys::TransitionEvent>>,
}

/// The mirrored transition machine.
struct IndicatorTransition {
    /// `transitionStatus` as a leptos signal.
    status: RwSignal<Option<TransitionStatus>>,
    /// `mounted` as a leptos signal.
    mounted: RwSignal<bool>,
}

/// Runs `useTransitionStatus(rendered)` + `useOpenChangeComplete({ batch: true, … })` inside a
/// dedicated rg-0.2 owner and mirrors their outputs into leptos signals (the
/// `checkbox::indicator` bridge).
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

        // `useOpenChangeComplete` (`RadioIndicator.tsx:45-55`): `enabled: !rendered`,
        // `open: rendered`, `batch: true`, and the completion that unmounts a closed
        // indicator (`:50-54`).
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
        let status = status;
        let transition_rg = transition_rg.clone();
        move |_| {
            status.set(reactive_graph::traits::Get::get(&transition_rg));
        }
    });

    IndicatorTransition { status, mounted }
}

/// The view seam (the `checkbox_indicator_view` convention).
pub fn radio_indicator_view(props: RadioIndicatorViewProps) -> impl IntoView {
    let RadioIndicatorViewProps {
        keep_mounted,
        render_class_style,
        element_attributes,
        handlers: consumer_handlers,
    } = props;

    // `const rootState = useRadioRootContext()` (`:23`) — throws outside a Root, which is the
    // source-side answer behavior.md:71 records for the nesting question.
    let root = use_radio_root_context();

    // `const rendered = rootState.checked` (`:25`).
    let rendered: Signal<bool> = root.checked;

    // `const indicatorRef = React.useRef(null)` (`:34`).
    let indicator_element_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));

    // `const { mounted, transitionStatus, setMounted } = useTransitionStatus(rendered)` (`:27`)
    // plus the `useOpenChangeComplete` completion (`:45-55`).
    let transition = use_indicator_transition(rendered, Rc::clone(&indicator_element_ref));
    let mounted = transition.mounted;
    let transition_status = transition.status;

    // `const shouldRender = keepMounted || mounted` (`:36`).
    let should_render: Signal<bool> =
        Signal::derive(move || indicator_should_render(keep_mounted, mounted.get()));

    // `const state: RadioIndicatorState = { ...rootState, transitionStatus }` (`:29-32`).
    let indicator_state = {
        let root = root.clone();
        move || RadioIndicatorState {
            root: root.snapshot(),
            transition_status: transition_status.get(),
        }
    };

    // The element (`:38-43`): `useRenderElement('span', componentProps, { ref, state,
    // props: elementProps, stateAttributesMapping })`. The consumer's rest members here are the
    // plain attributes plus the two animation handlers, which ride the view's own event
    // bindings below (adaptation 4).
    let element_bag = RenderElementProps {
        handlers: RenderElementHandlers {
            attributes: element_attributes
                .iter()
                .map(|(name, value)| {
                    let value = value.clone();
                    (
                        name.clone(),
                        Rc::new(move || Some(value.clone())) as ElementAttributeFn,
                    )
                })
                .collect(),
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    };
    let on_animation_end = consumer_handlers.on_animation_end.clone();
    let on_transition_end = consumer_handlers.on_transition_end.clone();

    let state_map = indicator_state().to_state_map();
    let rendered_element = use_render_element(
        "span",
        render_class_style,
        UseRenderElementParams {
            enabled: true,
            state: &state_map,
            refs: Vec::new(),
            props: vec![PropsSource::Static(element_bag)],
            state_attributes_mapping: None,
        },
    )
    .expect("the indicator element always renders while shouldRender");

    let class_value = rendered_element.props.class.clone();
    let style_value = rendered_element.props.style.clone();

    // The live state walk (`:41`) — recomputed in the writer so the transition hooks and the
    // Root's state both track (adaptation 1).
    let state_attributes: Signal<StateAttributeProps> =
        Signal::derive(move || radio_indicator_state_attributes(&indicator_state()));

    let indicator_node: NodeRef<leptos::html::Span> = NodeRef::new();
    let managed_names: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    {
        let managed_names = Rc::clone(&managed_names);
        let bag_attributes = rendered_element.props.handlers.attributes.clone();
        let class_value = class_value.clone();
        let style_value = style_value.clone();
        let indicator_element_ref = Rc::clone(&indicator_element_ref);
        Effect::new(move |_| {
            // Track the mount gate: the node only exists once `shouldRender` flips, so the
            // writer must re-run then and not just at the first pass.
            let _ = should_render.get();

            let Some(element) = indicator_node
                .get()
                .map(|span| span.unchecked_into::<web_sys::Element>())
            else {
                indicator_element_ref.set(None);
                return;
            };
            indicator_element_ref.set(Some(element.clone()));

            let walk = state_attributes.get();

            let mut attributes: Vec<(String, Option<String>)> = Vec::new();
            if let Some(class) = &class_value {
                attributes.push(("class".to_string(), Some(class.clone())));
            }
            if !style_value.is_empty() {
                attributes.push((
                    "style".to_string(),
                    Some(
                        style_value
                            .iter()
                            .map(|(property, value)| format!("{property}: {value};"))
                            .collect::<Vec<_>>()
                            .join(" "),
                    ),
                ));
            }
            for (name, value) in &bag_attributes {
                attributes.push((name.clone(), value()));
            }
            for name in MANAGED_STATE_ATTRIBUTES {
                attributes.push((name.to_string(), walk.get(name).cloned()));
            }

            let previous = managed_names.borrow().clone();
            let mut written: Vec<String> = Vec::new();
            for (name, value) in attributes {
                if let Some(value) = value {
                    let _ = element.set_attribute(&name, &value);
                    written.push(name);
                }
            }
            for name in previous {
                if !written.contains(&name) {
                    let _ = element.remove_attribute(&name);
                }
            }
            *managed_names.borrow_mut() = written;
        });
    }

    // The handler attach, alongside the writer (the `radio::root` idiom).
    {
        let rendered_element = rendered_element.clone();
        Effect::new(move |_| {
            let _ = should_render.get();
            let Some(element) = indicator_node
                .get()
                .map(|span| span.unchecked_into::<web_sys::Element>())
            else {
                return;
            };
            let target: &web_sys::EventTarget = element.unchecked_ref();
            let cleanup = rendered_element.props.handlers.attach_to(target);
            let cleanup = send_wrapper::SendWrapper::new(RefCell::new(cleanup));
            leptos::prelude::on_cleanup(move || {
                if let Some(cleanup) = cleanup.borrow_mut().take() {
                    cleanup();
                }
            });
        });
    }

    // The rest's handlers (`onAnimationEnd`/`onTransitionEnd`) attach at mount — the event
    // names' DOM spelling, and the mechanism that keeps the `Rc` handlers off the view's
    // `Send + Sync` closure boundary (the `checkbox::indicator` precedent).
    {
        let on_animation_end = on_animation_end.clone();
        let on_transition_end = on_transition_end.clone();
        Effect::new(move |_| {
            let _ = should_render.get();
            let Some(span) = indicator_node.get() else {
                return;
            };
            let mut cleanups: Vec<Option<Box<dyn FnOnce()>>> = Vec::new();
            if let Some(handler) = on_animation_end.clone() {
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
            if let Some(handler) = on_transition_end.clone() {
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
            let cleanups = send_wrapper::SendWrapper::new(RefCell::new(cleanups));
            leptos::prelude::on_cleanup(move || {
                for cleanup in cleanups.borrow_mut().drain(..) {
                    if let Some(cleanup) = cleanup {
                        cleanup();
                    }
                }
            });
        });
    }

    // `if (!shouldRender) return null` (`:57-59`).
    view! {
        <Show when=move || should_render.get() fallback=|| ()>
            <span node_ref=indicator_node></span>
        </Show>
    }
}
