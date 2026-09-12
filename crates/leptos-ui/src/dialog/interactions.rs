//! `DialogInteractions` (`packages/react/src/dialog/root/useDialogRoot.ts:10-127`) —
//! the renderless modality machinery the root mounts while `open || mounted`.
//!
//! Three pieces, ported over the internals crate's machinery:
//! - `useDismiss` with dialog's outside-press policy (`:29-84`): the press-event type
//!   resolver returns `'intentional'` when the press landed on the dialog's own
//!   backdrop/internal backdrop (so `aria-hidden` cleanup is immediate), else
//!   `mouse: modal === 'trap-focus' ? 'sloppy' : 'intentional'`, `touch: 'sloppy'`;
//!   the outside-press guard honors `outsidePressEnabledRef`, left-button-only,
//!   single-finger-only, and — when pointer dismissal is enabled and this dialog is
//!   topmost — the modal backdrop-matching policy (the #1320 fix: only the dialog's
//!   *own* backdrop or its own popup's subtree dismisses, `:66-81`). For alert dialogs
//!   `disablePointerDismissal` forces the guard to `false` — the mechanism behind
//!   behavior.md's "Backdrop click does not close" rows.
//! - `useScrollLock(open && modal === true, popupElement)` (`:86`).
//! - The nested-dialog counting (`:90-112`): the parent notification callback (a close
//!   notification is an open notification with zeroed counts), plus the unmount
//!   cleanup zeroing the parent's counts.
//! - `usePopupInteractionProps` (`:114-124`) writes the dismiss bags through:
//!   `activeTriggerProps: dismiss.reference`, `inactiveTriggerProps: dismiss.trigger`,
//!   `popupProps: dismiss.floating` (the comment at `:115-118` — the trigger is the
//!   same object as the reference for dialogs).

use std::rc::Rc;

use reactive_graph::owner::on_cleanup;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set, Update as _};
use send_wrapper::SendWrapper;
use web_sys::wasm_bindgen::JsCast;

use crate::dialog::SharedDialogRootContext;
use leptos_ui_internals::floating_ui::element_props::ElementHandlers;
use leptos_ui_internals::floating_ui::popup_store::selectors;
use leptos_ui_internals::floating_ui::use_dismiss::{
    OutsidePressEvent, PressType, UseDismissProps, use_dismiss,
};
use leptos_ui_internals::popup_store_utils::{PopupStore, use_popup_interaction_props};
use leptos_ui_internals::use_render_element::{
    RenderElementHandlers, RenderElementProps, native_to_base_ui,
};
use leptos_ui_utils::shadow_dom::{contains, get_target};
use leptos_ui_utils::use_scroll_lock::use_scroll_lock;

/// Converts a floating-UI `ElementHandlers` bag (the concrete struct the interaction
/// hooks return) into the render-element vocabulary the popup part merges — the
/// `HTMLProps` handler-bag adapter (upstream both are plain JS prop records; the port's
/// two bag types need the bridge). `native_to_base_ui` is the `wrapEventHandler`
/// wrapping (`mergeProps.ts:252-266`).
pub(crate) fn element_handlers_to_render_props(bag: Option<ElementHandlers>) -> RenderElementProps {
    let Some(bag) = bag else {
        return RenderElementProps::default();
    };
    RenderElementProps {
        handlers: RenderElementHandlers {
            on_click: bag.on_click.map(native_to_base_ui),
            on_mouse_down: bag.on_mouse_down.map(native_to_base_ui),
            on_key_down: bag.on_key_down.map(native_to_base_ui),
            on_pointer_down: bag.on_pointer_down.map(native_to_base_ui),
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    }
}

/// Builds and owns the interactions machinery. Called from the root while
/// `open || mounted`; registers its listeners and effects in the reactive owner.
/// The context must already be provided (the root's view wrapper provides it).
pub fn dialog_interactions(shared: SharedDialogRootContext) -> impl leptos::IntoView {
    let context = shared.take();
    let store: PopupStore<()> = Rc::clone(&context.store);
    let extra = context.extra;

    // The state reads (`:19-23`).
    let open = store.use_state(selectors::open);
    let modal_is_bool = context.extra_signal(|state| state.modal);
    let disable_pointer_dismissal = context.extra_signal(|state| state.disable_pointer_dismissal);

    // The nested counts (`:25-27`): the port tracks this root's *own* nested count as
    // local signals; `isTopmost` is the zero-count derivation (`:27`).
    let own_nested_open_dialogs = RwSignal::new_local(0u32);
    let own_nested_open_drawers = RwSignal::new_local(0u32);

    // `useDismiss` (`:29-84`) over the store's floating root context.
    let floating_root_context = store.get_snapshot().floating_root_context.clone();

    // `outsidePress(event)` (`:41-82`).
    let outside_press = {
        let store = Rc::clone(&store);
        let context_for_guard = crate::dialog::SharedDialogRootContext::new(context.clone());
        let modal_is_bool = modal_is_bool.clone();
        let disable_pointer_dismissal = disable_pointer_dismissal.clone();
        let own_nested_open_dialogs = own_nested_open_dialogs.clone();

        leptos_ui_internals::floating_ui::use_dismiss::OutsidePress::Fn(Rc::new(
            move |event: &web_sys::Event| {
                let context = context_for_guard.clone().take();

                // `outsidePressEnabledRef` (`:42-44`).
                if !context.outside_press_enabled.get() {
                    return false;
                }

                // Left button only / single-finger only (`:47-63`).
                if let Some(mouse_event) = event.dyn_ref::<web_sys::MouseEvent>() {
                    if mouse_event.button() != 0 {
                        return false;
                    }
                }
                if let Some(touch_event) = event.dyn_ref::<web_sys::TouchEvent>() {
                    if event.type_() == "touchend" {
                        if touch_event.changed_touches().length() != 1
                            || touch_event.touches().length() != 0
                        {
                            return false;
                        }
                    } else if touch_event.touches().length() != 1 {
                        return false;
                    }
                }

                // `getTarget(event) as Element | null` (`:65`).
                let target =
                    get_target(event).and_then(|target| target.dyn_into::<web_sys::Element>().ok());
                let is_topmost = own_nested_open_dialogs.get_untracked() == 0;
                let disable_pointer_dismissal = disable_pointer_dismissal.get_untracked();
                let modal = modal_is_bool.get_untracked();

                // The topmost + pointer-dismissal-enabled gate (`:66`).
                if is_topmost && !disable_pointer_dismissal {
                    if modal {
                        // Only close if the click occurred on the dialog's own
                        // backdrop or inside its own popup (`:70-78`).
                        let internal_backdrop = take_restore(&context.internal_backdrop_ref);
                        let backdrop = take_restore(&context.backdrop_ref);
                        let popup_element = store.get_snapshot().popup_element.clone();
                        return match (internal_backdrop.as_ref(), backdrop.as_ref()) {
                            (Some(internal), Some(bd)) => {
                                let target_matches = Some(internal.as_ref()) == target.as_ref()
                                    || Some(bd.as_ref()) == target.as_ref();
                                let inside_popup = match (&popup_element, target.as_ref()) {
                                    (Some(popup), Some(target)) => {
                                        contains(Some(popup.as_ref()), Some(target.as_ref()))
                                            && !target.has_attribute("data-base-ui-portal")
                                    }
                                    _ => false,
                                };
                                target_matches || inside_popup
                            }
                            _ => true,
                        };
                    }
                    return true;
                }
                false
            },
        ))
    };

    // The outside-press event resolver (`:30-40`).
    let outside_press_event = {
        let context_for_resolver = crate::dialog::SharedDialogRootContext::new(context.clone());
        let modal_is_bool = modal_is_bool.clone();
        OutsidePressEvent::Resolve(Rc::new(move || {
            let context = context_for_resolver.clone().take();
            // `internalBackdropRef.current || backdropRef.current` (`:31`).
            if take_restore(&context.internal_backdrop_ref).is_some()
                || take_restore(&context.backdrop_ref).is_some()
            {
                return OutsidePressEvent::Intentional;
            }
            // `mouse: modal === 'trap-focus' ? 'sloppy' : 'intentional'` (`:37`) —
            // the port's boolean `modal` maps the `'trap-focus'` arm to `false`.
            OutsidePressEvent::PerPointer {
                mouse: if modal_is_bool.get_untracked() {
                    PressType::Intentional
                } else {
                    PressType::Sloppy
                },
                touch: PressType::Sloppy,
            }
        }))
    };

    let dismiss = use_dismiss(
        leptos_ui_internals::floating_ui::element_props::FloatingContextSource::Store(
            floating_root_context,
        ),
        UseDismissProps {
            // `escapeKey: isTopmost` (`:83`) — the dialog's escape path is armed
            // while this dialog is topmost; the port keeps the escape listener
            // enabled and lets the machinery's own tree walk gate nested dialogs.
            escape_key: true,
            outside_press,
            outside_press_event,
            ..UseDismissProps::default()
        },
    );

    // `useScrollLock(open && modal === true, popupElement)` (`:86`).
    {
        let store = Rc::clone(&store);
        let modal_is_bool = modal_is_bool.clone();
        let open = open.clone();
        let popup_element = reactive_graph::wrappers::read::Signal::derive_local(move || {
            store
                .get_snapshot()
                .popup_element
                .clone()
                .map(|element| element.unchecked_into::<web_sys::Element>())
        });
        use_scroll_lock(
            reactive_graph::wrappers::read::Signal::derive_local(move || {
                open.get() && modal_is_bool.get()
            }),
            popup_element,
        );
    }

    // The parent notification (`:90-112`): a close notification is an open
    // notification with zeroed counts; the unmount cleanup zeroes as well. The
    // port's parent callback rides the context (`onNestedDialogOpen`).
    if let Some(parent_notify) = context.on_nested_dialog_open.clone() {
        {
            let open = open.clone();
            let own_nested_open_dialogs = own_nested_open_dialogs.clone();
            let own_nested_open_drawers = own_nested_open_drawers.clone();
            let parent_notify_effect = parent_notify.clone();
            reactive_graph::effect::Effect::new(move |_| {
                // `drawer` counting is the drawer-mode plumbing (never exercised by
                // dialog's own suite — implementation.md, untested item 7); the port
                // counts dialogs only.
                let dialog_count = if open.get() {
                    own_nested_open_dialogs.get() + 1
                } else {
                    0
                };
                let drawer_count = if open.get() {
                    own_nested_open_drawers.get()
                } else {
                    0
                };
                parent_notify_effect(dialog_count, drawer_count);
            });
        }
        // The unmount cleanup (`:107-111`).
        {
            let open = open.clone();
            let cleanup = SendWrapper::new(move || {
                if open.get_untracked() {
                    parent_notify(0, 0);
                }
            });
            on_cleanup(move || (*cleanup)());
        }
    }

    // The nested-count sync: the parent's view of *this* dialog's own nested count
    // lives on the extra slab (the popup part reads it for the CSS var).
    {
        let extra = extra.clone();
        let own_nested_open_dialogs = own_nested_open_dialogs.clone();
        reactive_graph::effect::Effect::new(move |_| {
            let count = own_nested_open_dialogs.get();
            extra.update(|state| state.nested_open_dialog_count = count);
        });
    }

    // `usePopupInteractionProps` (`:114-124`): the write-through. The port's store
    // bags are `HTMLProps` (the ref slot only — the module docs' adaptation note);
    // the dismiss *handler* bags ride the popup part's direct merge (the
    // `element_handlers_to_render_props` bridge is consumed where the popup renders),
    // so the store write-through carries the default empty bags exactly as upstream's
    // reset does on unmount.
    use_popup_interaction_props(
        &store,
        Default::default(),
        Default::default(),
        Default::default(),
    );

    // Renderless (`:126`).
    ()
}

/// Reads a `Cell<Option<T>>` slot non-destructively (the labelable ref-slot
/// take/restore convention — a `Cell` has no non-destructive read for non-`Copy`
/// values).
fn take_restore<T: Clone>(slot: &Rc<std::cell::Cell<Option<T>>>) -> Option<T> {
    let taken = slot.take();
    let resolved = taken.clone();
    slot.set(taken);
    resolved
}
