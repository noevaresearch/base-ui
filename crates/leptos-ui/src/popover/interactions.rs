//! `PopoverInteractions` (`packages/react/src/popover/root/PopoverRoot.tsx:227-260`)
//! — the renderless dismissal machinery the root mounts while `open || mounted`
//! (`:83-88`), over the shared `useDismiss` machinery.
//!
//! Popover's policy delta from dialog (`:236-243`): the outside-press event is
//! `mouse: modal === 'trap-focus' ? 'sloppy' : 'intentional'`, `touch: 'sloppy'` —
//! the comment explains it removes outside `aria-hidden` immediately on outside
//! press when trapping focus. The bags publish through `usePopupInteractionProps`
//! (`:253-257`); `dismiss.trigger` is always the same object as `dismiss.reference`
//! (`:250`), so both trigger bags carry the reference bag.

use std::rc::Rc;

use reactive_graph::traits::{GetUntracked as _, Update as _};

use crate::popover::store::SharedPopoverRootContext;

use leptos_ui_internals::floating_ui::popup_store::selectors;
use leptos_ui_internals::floating_ui::use_dismiss::{
    OutsidePressEvent, PressType, UseDismissProps, use_dismiss,
};
use leptos_ui_internals::popup_store_utils::{PopupStore, use_popup_interaction_props};

/// Builds and owns the interactions machinery. Called from the root view while
/// `open || mounted`; registers its listeners in the reactive owner. The context
/// must already be provided (the root's view wrapper provides it).
pub fn popover_interactions(shared: SharedPopoverRootContext) -> impl leptos::IntoView {
    let context = shared.take();
    let store: PopupStore<()> = Rc::clone(&context.store);
    let extra = context.extra;

    // `const floatingRootContext = store.useState('floatingRootContext')` (`:232`).
    let floating_root_context = store.get_snapshot().floating_root_context.clone();

    // `useDismiss(floatingRootContext, { outsidePressEvent })` (`:235-243`).
    let trap_focus = extra.get_untracked().trap_focus;
    let outside_press_event = OutsidePressEvent::PerPointer {
        // `mouse: modal === 'trap-focus' ? 'sloppy' : 'intentional'` (`:238`).
        mouse: if trap_focus {
            PressType::Sloppy
        } else {
            PressType::Intentional
        },
        // `touch: 'sloppy'` (`:240`).
        touch: PressType::Sloppy,
    };

    let dismiss = use_dismiss(
        leptos_ui_internals::floating_ui::element_props::FloatingContextSource::Store(
            floating_root_context,
        ),
        UseDismissProps {
            outside_press_event,
            ..UseDismissProps::default()
        },
    );

    // `usePopupInteractionProps(store, { activeTriggerProps: dismiss.reference,
    // inactiveTriggerProps: dismiss.reference, popupProps: dismiss.floating })`
    // (`:250-257`): the port's store bags are `HTMLProps` (the module-docs
    // adaptation note — the handler bags ride the popup/trigger parts' direct
    // merge), so the write-through carries the default empty bags exactly as the
    // dialog interactions precedent does.
    use_popup_interaction_props(
        &store,
        Default::default(),
        Default::default(),
        Default::default(),
    );

    // The dismissal close routes through the full `popover_set_open` pipeline: the
    // machinery's `onOpenChange` (the floating-root store's open-change dispatcher)
    // was seeded with `sync_only: true` (`popup_store.rs`), so the port binds the
    // dismiss events' close path through the root writer here — every dismiss
    // (escape, outside press) funnels into `popover_set_open` with the machinery's
    // reason (`escapeKey` / `outsidePress`).
    {
        let store_for_events = Rc::clone(&store);
        let extra_for_events = extra.clone();
        let events = store_for_events
            .get_snapshot()
            .floating_root_context
            .context
            .events
            .clone();
        let unsubscribe = events.on(
            "openchange",
            Rc::new(
                move |details: &leptos_ui_internals::floating_ui::types::FloatingUIOpenChangeDetails| {
                    let next_open = details.open;
                    let mut change =
                        leptos_ui_internals::floating_ui::types::RootOpenChangeEventDetails::new(
                            details.reason.clone(),
                            details.native_event.clone(),
                            None,
                            String::new(),
                        );
                    if !next_open {
                        extra_for_events.update(|state| {
                            state.open_change_reason = Some(details.reason.clone())
                        });
                    }
                    crate::popover::store::popover_set_open(
                        &store_for_events,
                        next_open,
                        &mut change,
                    );
                },
            ),
        );
        let unsubscribe = send_wrapper::SendWrapper::new(unsubscribe);
        reactive_graph::owner::on_cleanup(move || unsubscribe());
    }

    // Renderless (`:260`).
    ()
}
