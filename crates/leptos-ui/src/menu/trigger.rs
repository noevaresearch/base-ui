//! Menu trigger — the button that opens the menu.
//!
//! Port of `packages/react/src/menu/trigger/MenuTrigger.tsx` (the interaction layer
//! this iteration ports: the click/hover/keyboard open requests routed through the
//! one mutation gate with the reason taxonomy, and the active-trigger claim). The
//! `useButton`/`useClick`/hover-hook composition, the drag-release mouseup contract,
//! and the menubar specifics are their own checkpoints — each is recorded in the
//! item's TODO note.
//!
//! Why every request goes through `menu_store_set_open`: the upstream trigger never
//! writes state directly — `store.setOpen` emits and `MenuRoot`'s gate owns the
//! transition (implementation.md "One mutation gate"). The previous port set the
//! open signal directly, bypassing the stale-guard/dedupe/veto/instantType machine.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::menu::store::{menu_store_set_open, use_menu_store};

/// The `Menu.Trigger` part — upstream's `MenuTrigger`
/// (`packages/react/src/menu/trigger/MenuTrigger.tsx`), named the way upstream's
/// `index.parts.ts:17` names it (`export { MenuTrigger as Trigger }`).
///
/// Renders a button that opens the menu when clicked or hovered
/// (`MenuTrigger.tsx:221-247` interaction layers; the port's event handlers adapt
/// them to Leptos's native events).
#[component]
pub fn Trigger(
    /// Whether the trigger is disabled (`MenuTrigger.tsx` `disabled` — read via the
    /// store's selector once the store carries it; the prop seeds the request gate).
    #[prop(default = false)]
    disabled: bool,
    /// Whether to open on hover (`MenuTrigger.tsx` hover layer; the delay timers are
    /// the hover checkpoint's).
    #[prop(default = false)]
    open_on_hover: bool,
    /// Custom trigger content.
    children: Children,
) -> impl IntoView {
    let menu_store = use_menu_store();
    let open = menu_store.open();
    let click_store = menu_store.store.clone();
    let enter_store = menu_store.store.clone();
    let leave_store = menu_store.store.clone();
    let keydown_store = menu_store.store.clone();

    view! {
        <button
            disabled=disabled
            on:click=move |event: leptos::ev::MouseEvent| {
                if disabled {
                    return;
                }
                // The click toggle (`MenuTrigger.tsx` useClick layer →
                // `store.setOpen(!open, createChangeEventDetails('trigger-press'))`).
                let next = !open.get_untracked();
                menu_store_set_open(
                    &click_store,
                    next,
                    "trigger-press",
                    Some(event.unchecked_into()),
                );
            }
            on:mouseenter=move |event: leptos::ev::MouseEvent| {
                if disabled || !open_on_hover {
                    return;
                }
                // The hover open (`useHoverReferenceInteraction` →
                // `store.setOpen(true, …('trigger-hover'))`; the delay/rest timers are
                // the hover checkpoint's).
                menu_store_set_open(
                    &enter_store,
                    true,
                    "trigger-hover",
                    Some(event.unchecked_into()),
                );
            }
            on:mouseleave=move |event: leptos::ev::MouseEvent| {
                if disabled || !open_on_hover {
                    return;
                }
                // The hover close (`store.setOpen(false, …('trigger-hover'))`).
                menu_store_set_open(
                    &leave_store,
                    false,
                    "trigger-hover",
                    Some(event.unchecked_into()),
                );
            }
            on:keydown=move |event: leptos::ev::KeyboardEvent| {
                if disabled {
                    return;
                }
                // The keyboard open (ArrowDown/Enter/Space — `MenuTrigger.tsx`'s
                // keydown layer; keyboard activations carry `detail === 0`, which the
                // gate's `instantType: 'click'` heuristic reads, `MenuRoot.tsx:370-375`).
                match event.key().as_str() {
                    "ArrowDown" | "Enter" | " " => {
                        event.prevent_default();
                        let next = !open.get_untracked();
                        menu_store_set_open(
                            &keydown_store,
                            next,
                            "trigger-press",
                            Some(event.unchecked_into()),
                        );
                    }
                    _ => {}
                }
            }
            aria-haspopup="menu"
            aria-expanded=move || open.get()
            data-popup-open=move || open.get()
        >
            {children()}
        </button>
    }
}
