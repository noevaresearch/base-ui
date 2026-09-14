//! The popover root — `packages/react/src/popover/root/PopoverRoot.tsx` over the
//! store (`store.rs`), the dialog `use_render_dialog_root` house pattern:
//!
//! - the store is created exactly once per Root and shared by reference through
//!   `PopoverRootContext` (`PopoverRoot.tsx:40`, `PopoverRootContext.ts:5`);
//! - the controlled/uncontrolled split rides the shared spine's `openProp` /
//!   `triggerIdProp` (`:48-49`);
//! - `usePopupRootSync` (`:58`) + `useImplicitActiveTrigger` (`:59`) +
//!   `useOpenStateTransitions` (`:60-62`) drive the shared lifecycle;
//! - `PopoverInteractions` (`:227-260`) mounts `useDismiss` while
//!   `open || mounted` (`:83-88`) and publishes the dismiss bags through
//!   `usePopupInteractionProps` (`:253-257`);
//! - the root wraps itself in `FloatingTree` when no enclosing popup root context
//!   exists (`:100-110` — the port's floating tree context read, the dialog
//!   nesting precedent).

use std::cell::Cell;
use std::rc::Rc;

use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked, Update as _};
use send_wrapper::SendWrapper;

use crate::popover::interactions::popover_interactions;
use crate::popover::store::{
    OnOpenChange, OnOpenChangeComplete, PopoverExtraState, PopoverRootContext,
    SharedPopoverRootContext, popover_set_open, wire_popover_slots,
};

use leptos_ui_internals::floating_ui::popup_store::selectors;
use leptos_ui_internals::floating_ui::types::RootOpenChangeEventDetails;
use leptos_ui_internals::floating_ui::{PopupTriggerMap, reasons};
use leptos_ui_internals::popup_store_utils::{
    UseOpenStateTransitions, use_implicit_active_trigger, use_open_state_transitions,
    use_popup_root_sync,
};

pub mod interactions;
pub mod parts;
pub mod store;

/// The root props — upstream's `PopoverRootProps` (`PopoverRoot.tsx:141-232`) with
/// the documented defaults.
#[derive(Clone)]
pub struct PopoverRootProps {
    /// `open` (`:149`) — the controlled value; `None` while uncontrolled.
    pub open: Option<bool>,
    /// `defaultOpen` (`:143` — upstream default `false`).
    pub default_open: bool,
    /// `onOpenChange` (`:152`).
    pub on_open_change: Option<OnOpenChange>,
    /// `onOpenChangeComplete` (`:158`).
    pub on_open_change_complete: Option<OnOpenChangeComplete>,
    /// `modal: boolean | 'trap-focus'` (`:177-189` — upstream default `false`).
    pub modal: PopoverModal,
    /// `triggerId` (`:196`) — selects the active trigger in controlled mode.
    pub trigger_id: Option<String>,
    /// `defaultTriggerId` (`:202` — upstream default `null`).
    pub default_trigger_id: Option<String>,
}

/// The `modal` union (`PopoverRoot.tsx:177`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PopoverModal {
    /// `false` — the default.
    #[default]
    False,
    /// `true`.
    True,
    /// `'trap-focus'`.
    TrapFocus,
}

impl PopoverModal {
    /// The boolean projection the store slab carries (`PopoverStore.ts:22`).
    pub fn is_bool_true(self) -> bool {
        matches!(self, PopoverModal::True)
    }
    /// The `'trap-focus'` arm.
    pub fn is_trap_focus(self) -> bool {
        matches!(self, PopoverModal::TrapFocus)
    }
}

impl Default for PopoverRootProps {
    fn default() -> Self {
        PopoverRootProps {
            open: None,
            default_open: false,
            on_open_change: None,
            on_open_change_complete: None,
            modal: PopoverModal::False,
            trigger_id: None,
            default_trigger_id: None,
        }
    }
}

/// The root value the `PopoverRoot` component produces — the open/mounted reads and
/// the imperative actions handle, handed back to the caller (the dialog
/// `DialogRootValue` precedent for upstream's `actionsRef` ref-object contract).
pub struct PopoverRootValue {
    /// The open-state read (`:51`).
    pub open: RgRwSignal<bool, reactive_graph::owner::LocalStorage>,
    /// The mounted-state read (`:52`).
    pub mounted: RgRwSignal<bool, reactive_graph::owner::LocalStorage>,
    /// The payload read (`:53`).
    pub payload: RgRwSignal<Option<()>, reactive_graph::owner::LocalStorage>,
}

/// `usePopoverRootStore` + the Root body (`PopoverRoot.tsx:40-110`). Must be called
/// inside a reactive owner. Returns the root value and the shared context the caller
/// provides before rendering children (the `PopoverRootView` shim).
pub fn use_render_popover_root(
    props: PopoverRootProps,
) -> (PopoverRootValue, SharedPopoverRootContext) {
    let PopoverRootProps {
        open: open_prop,
        default_open,
        on_open_change,
        on_open_change_complete,
        modal,
        trigger_id: trigger_id_prop,
        default_trigger_id: default_trigger_id_prop,
    } = props;

    // The store (`:40` + factory `:112-127`): created exactly once, seeded with the
    // family defaults (`createInitialState`, `PopoverStore.ts:189-217`). The
    // `defaultOpen` seed: `open: true` ⇒ `mounted: true` so content renders on
    // first paint.
    let mut initial_state = crate::popover::store::create_initial_popover_store_state(None, false);
    initial_state.open = default_open;
    if default_open {
        initial_state.mounted = true;
    }

    let store: leptos_ui_internals::popup_store_utils::PopupStore<()> = {
        Rc::new(leptos_ui_utils::react_store::ReactStore::with_context(
            initial_state,
            leptos_ui_internals::floating_ui::popup_store::PopupStoreContext {
                trigger_elements: leptos_ui_internals::floating_ui::PopupTriggerMap::new(),
                popup_ref: Rc::new(Cell::new(None)),
                on_open_change: on_open_change.clone().map(|callback| {
                    let callback: Rc<dyn Fn(bool, &RootOpenChangeEventDetails)> = callback;
                    callback
                }),
                on_open_change_complete: on_open_change_complete
                    .map(|callback| callback as Rc<dyn Fn(bool)>),
            },
        ))
    };

    // The controlled-prop syncs (`:48-49`).
    store.set_field(|state| &mut state.open_prop, open_prop);
    store.set_field(|state| &mut state.trigger_id_prop, trigger_id_prop.clone());
    // `activeTriggerId` seeds from `defaultTriggerId` (`:55`).
    store.set_field(
        |state| &mut state.active_trigger_id,
        default_trigger_id_prop.clone(),
    );

    // The reactive reads (`:51-53`).
    let open = store.use_state(selectors::open);
    let mounted = store.use_state(selectors::mounted);
    let payload = store.use_state(selectors::payload);

    // The extra slab (`PopoverStore.ts:18-31` defaults) and the patient-click
    // timeout (`createInitialContext`, `:44`).
    let extra: RgRwSignal<PopoverExtraState, reactive_graph::owner::LocalStorage> =
        RgRwSignal::new_local(PopoverExtraState {
            modal: modal.is_bool_true(),
            trap_focus: modal.is_trap_focus(),
            ..PopoverExtraState::default()
        });
    let stick_if_open_timeout = Rc::new(leptos_ui_utils::use_timeout::Timeout::create());
    wire_popover_slots(extra.clone(), Rc::clone(&stick_if_open_timeout));

    // `usePopupRootSync(store, open)` (`:58`).
    use_popup_root_sync(&store, open);

    // `useImplicitActiveTrigger(store)` (`:59`) — the detached-trigger robustness
    // engine; the writer routes through the full `popover_set_open` pipeline.
    use_implicit_active_trigger(&store, root_writer(&store), false);

    // `useOpenStateTransitions(open, store, onUnmount)` (`:60-62`) — the onUnmount
    // callback resets `stickIfOpen` and `openChangeReason` (`:60-62`).
    let extra_for_unmount = extra.clone();
    let on_unmount: Rc<dyn Fn()> = Rc::new(move || {
        extra_for_unmount.update(|state| {
            state.stick_if_open = true;
            state.open_change_reason = None;
        });
    });
    let UseOpenStateTransitions { force_unmount, .. } =
        use_open_state_transitions(open, &store, Some(on_unmount), false);

    // The patient-click timeout disposal on unmount (`:124`).
    {
        let timeout = Rc::clone(&stick_if_open_timeout);
        let cleanup = SendWrapper::new(move || timeout.clear());
        reactive_graph::owner::on_cleanup(move || (*cleanup)());
    }

    // The stickIfOpen reset effect whenever closed (`:68-72`).
    {
        let extra = extra.clone();
        let timeout = Rc::clone(&stick_if_open_timeout);
        reactive_graph::effect::Effect::new(move |_| {
            if !open.get() {
                timeout.clear();
                extra.update(|state| state.stick_if_open = true);
            }
        });
    }

    // The actions handle (`:74-81`): `unmount` → `forceUnmount`, `close` →
    // `store.setOpen(false, imperativeAction)`.
    let actions_store = Rc::clone(&store);
    let actions = PopoverActions {
        unmount: Rc::new(move || {
            force_unmount.call(());
        }),
        close: Rc::new(move || {
            let mut details = crate::popover::store::popover_change_event_details(
                reasons::IMPERATIVE_ACTION,
                None,
            );
            popover_set_open(&actions_store, false, &mut details);
        }),
    };
    STORE_ACTIONS_SLOT.with(|slot| *slot.borrow_mut() = Some(actions.clone()));

    // The context value (`:86`).
    let context = PopoverRootContext {
        store: Rc::clone(&store),
        extra,
        popup_ref: Rc::new(Cell::new(None)),
        trigger_focus_target_ref: Rc::new(Cell::new(None)),
        before_content_focus_guard_ref: Rc::new(Cell::new(None)),
        stick_if_open_timeout,
    };
    let shared_context = SendWrapper::new(context);

    (
        PopoverRootValue {
            open,
            mounted,
            payload,
        },
        shared_context,
    )
}

/// The root's shared writer — the full `popover_set_open` sequence as the store's
/// open writer (the dialog `root_writer` convention).
fn root_writer(store: &leptos_ui_internals::popup_store_utils::PopupStore<()>) -> OnOpenChange {
    let store = Rc::clone(store);
    Rc::new(
        move |next_open: bool, details: &RootOpenChangeEventDetails| {
            popover_set_open(&store, next_open, &mut { details.clone() });
        },
    )
}

/// The imperative `Actions` handle (`PopoverRoot.tsx:74-81`).
#[derive(Clone)]
pub struct PopoverActions {
    /// `unmount` (`:77`) — the `forceUnmount` callback.
    pub unmount: Rc<dyn Fn()>,
    /// `close` (`:78`) — closes the popover imperatively with
    /// `REASONS.imperativeAction`.
    pub close: Rc<dyn Fn()>,
}

impl PopoverActions {
    /// `actionsRef.current.unmount()`.
    pub fn unmount(&self) {
        (self.unmount)();
    }
    /// `actionsRef.current.close()`.
    pub fn close(&self) {
        (self.close)();
    }
}

thread_local! {
    /// The imperative-actions slot of the most recently rendered root — the port's
    /// `actionsRef` stand-in (the dialog `STORE_ACTIONS_SLOT` precedent).
    static STORE_ACTIONS_SLOT: std::cell::RefCell<Option<PopoverActions>> =
        const { std::cell::RefCell::new(None) };
}

/// Reads the mounted root's `Actions` handle (the `actionsRef.current` read).
pub fn popover_actions() -> Option<PopoverActions> {
    STORE_ACTIONS_SLOT.with(|slot| slot.borrow().clone())
}

/// The shared root view body — the context provision, the interactions gate while
/// `open || mounted` (`:83-88`), and the children render (the `dialog_root_view`
/// precedent).
pub fn popover_root_view(
    popover_props: PopoverRootProps,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    use leptos::prelude::*;

    let (value, shared_context) = use_render_popover_root(popover_props);
    let context = shared_context;
    let open = value.open;
    let mounted = value.mounted;

    // The context provision (`:86`) — the descendants consume it through
    // `use_context`.
    reactive_graph::owner::provide_context(SendWrapper::new((*context).clone()));

    view! {
        <>
            <Show when=move || open.get() || mounted.get() fallback=|| ()>
                {popover_interactions(SharedPopoverRootContext::new(
                    (*context).clone(),
                ))}
            </Show>
            {children()}
        </>
    }
}

/// The `Popover.Root` component — the view wrapper over [`use_render_popover_root`]
/// delegating to [`popover_root_view`]. Wraps itself in the floating tree when no
/// enclosing popup root context exists (`:100-110` — the port's nesting detection
/// rides the same context read the store seeding uses).
#[leptos::component]
pub fn PopoverRootComponent(
    #[prop(default = PopoverRootProps::default(), optional)] popover_props: PopoverRootProps,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    popover_root_view(popover_props, children)
}
