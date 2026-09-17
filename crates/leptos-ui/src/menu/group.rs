//! Menu group — `Menu.Group`, port of `packages/react/src/menu/group/MenuGroup.tsx` and its
//! context module (`MenuGroupContext.ts`).
//!
//! WHAT THIS REPLACES. The previous `group.rs` was a facade, not a port: it rendered a
//! hardcoded `<div class="menu-group">` carrying invented `aria-disabled`,
//! `data-group-id` and `data-disabled` attributes (upstream's group element carries exactly
//! two props: `role` and `aria-labelledby`, `MenuGroup.tsx:24-30`), and its context was an
//! invented tuple `Rc<(String, Option<Callback<String>>)>` — the group id plus a setter whose
//! "getter" (`use_menu_group_label_id`) returned `None` with the comment "in a real
//! implementation this would be stored in a signal". A label id that is never stored is
//! precisely the association `specs/library/menu/parts/group-item-link.md` →
//! "Accessibility" requires the group to expose.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from
//! (`MenuGroup.tsx`, 45 lines total, plus `MenuGroupContext.ts`, 17):
//! - the `labelId` state and its setter (`MenuGroup.tsx:17`,
//!   `React.useState<string | undefined>(undefined)`) — held as a signal so the group's
//!   `aria-labelledby` re-renders when a label registers or unmounts (the registration
//!   order test, `MenuGroupLabel.test.tsx:183-219`).
//! - the element props (`:24-30`): `role: 'group'` (the accessibility claim,
//!   `MenuGroup.test.tsx:14-17`) and `aria-labelledby: labelId`, with the consumer's own
//!   bag landing after them (the merge order `input.rs` documents).
//! - the context value and its provider (`:32`, `<MenuGroupContext.Provider
//!   value={setLabelId}>`) — the raw setter, shared by `Menu.RadioGroup`
//!   (`implementation.md` → "Group/label wiring": both group kinds provide the same
//!   setter).
//! - the required read's error (`MenuGroupContext.ts:8-17`): rendered outside
//!   `<Menu.Group>` / `<Menu.RadioGroup>` a group label throws upstream's own message.
//!
//! The setter's two call shapes are upstream's `React.SetStateAction<string | undefined>`
//! union, collapsed by the already-ported [`LabelIdUpdate`] (`use_registered_label_id.rs`'s
//! own module docs): the mount write is [`LabelIdUpdate::Set`] and the cleanup's
//! `(currentId) => currentId === id ? undefined : currentId` is
//! [`LabelIdUpdate::ClearIfCurrent`] — which is what makes an older label's unmount unable
//! to clobber a newer label's id.
//!
//! DEFERRED, with the reason: `render` (`MenuGroup.tsx:19`) and the forwarded ref (`:18`)
//! — the crate-wide items named in [`crate::menu::arrow`]'s docs. `Menu.RadioGroup` is the
//! other provider of this context (`radio-group/MenuRadioGroup.tsx:34,77`); it belongs to
//! its own part checkpoint, so this module exposes the context and its setter for it rather
//! than porting it here.

use std::rc::Rc;

use leptos::prelude::*;
use leptos_ui_internals::use_registered_label_id::{LabelIdSetter, LabelIdUpdate};
use reactive_graph::owner::LocalStorage;
use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};

/// `MenuGroup.tsx:26` — the group element's role. `MenuGroup.test.tsx:14-17` asserts it.
pub const MENU_GROUP_ROLE: &str = "group";

/// The group context value (`MenuGroupContext.ts:3-5`): upstream's `React.Dispatch<…>`
/// setter plus the state it dispatches into. Upstream's value IS the setter, whose target
/// state lives in the `MenuGroup` component; the port carries the state alongside it so the
/// read (`aria-labelledby`) and the write (a label's registration) share one handle.
#[derive(Clone)]
pub struct MenuGroupContextValue {
    /// `labelId` (`MenuGroup.tsx:17`) — the id of the currently mounted group label, `None`
    /// before one registers (upstream's `undefined`).
    pub label_id: RgRwSignal<Option<String>, LocalStorage>,
    /// `setLabelId` (`MenuGroupContext.ts:3`) — dispatches [`LabelIdUpdate`] over
    /// [`Self::label_id`].
    pub set_label_id: LabelIdSetter,
}

impl Default for MenuGroupContextValue {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for MenuGroupContextValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuGroupContextValue")
            .field("label_id", &self.label_id.get_untracked())
            .finish()
    }
}

impl MenuGroupContextValue {
    /// Builds the `MenuGroup` component's own state pair (`MenuGroup.tsx:17` + `:32`): one
    /// `labelId` state and the setter the provider publishes, answering both arms of
    /// upstream's `SetStateAction` union (`use_registered_label_id.rs`'s module docs).
    pub fn new() -> Self {
        let label_id: RgRwSignal<Option<String>, LocalStorage> = RgRwSignal::new_local(None);
        let set_label_id: LabelIdSetter = Rc::new(move |update| match update {
            // `setLabelId(id)` (`MenuGroupLabel.tsx:23`) — the plain value dispatch.
            LabelIdUpdate::Set(next) => label_id.set(next),
            // `setLabelId((currentId) => currentId === id ? undefined : currentId)`
            // (`:25`) — clear only while the setter's current value still equals the
            // carried id, so a later label's registration survives this hook's unmount.
            LabelIdUpdate::ClearIfCurrent(id) => {
                if label_id.get_untracked().as_deref() == Some(id.as_str()) {
                    label_id.set(None);
                }
            }
        });
        Self {
            label_id,
            set_label_id,
        }
    }
}

/// Executes the `setLabelId` dispatch — the free-function form the parts call.
pub fn set_group_label_id(context: &MenuGroupContextValue, update: LabelIdUpdate) {
    (context.set_label_id)(update);
}

/// `MenuGroup.tsx:27` — the group element's `aria-labelledby`: the registered label's id,
/// absent until a group label mounts (`MenuGroupLabel.test.tsx:82-101`). Read reactively so
/// the attribute tracks registrations.
///
/// Takes the state signal rather than the context value: a view closure must be `Send`, and
/// the context carries the non-`Send` [`LabelIdSetter`] handle. The view therefore captures
/// the `Copy` signal and calls this same function.
pub fn menu_group_aria_labelledby(
    label_id: RgRwSignal<Option<String>, LocalStorage>,
) -> Option<String> {
    label_id.get()
}

/// The shared context bridge: the group value crosses `provide_context`'s `Send + Sync`
/// bound through the `SendWrapper` bridge — the positioner's
/// [`crate::menu::positioner::SharedMenuPositionerContext`] precedent
/// (`positioner.rs:391-394`).
pub type SharedMenuGroupContext = send_wrapper::SendWrapper<MenuGroupContextValue>;

/// `useMenuGroupRootContext` (`MenuGroupContext.ts:8-17`): the required read, which throws
/// upstream's message outside both group kinds. `LabelIdUpdate`'s consumer (`::group-label`)
/// is the one part that reads it.
pub fn use_menu_group_context(value: Option<SharedMenuGroupContext>) -> MenuGroupContextValue {
    match value {
        Some(value) => value.take(),
        None => panic!(
            "Base UI: MenuGroupContext is missing. Menu group parts must be used within <Menu.Group> or <Menu.RadioGroup>."
        ),
    }
}

/// The `MenuGroupContext` read against the current reactive owner.
pub fn menu_group_context() -> MenuGroupContextValue {
    use_menu_group_context(use_context::<SharedMenuGroupContext>())
}

/// The optional read — for parts that are legal both inside and outside a group.
pub fn menu_group_context_optional() -> Option<MenuGroupContextValue> {
    use_context::<SharedMenuGroupContext>().map(|value| value.take())
}

/// `Menu.Group` — upstream's `MenuGroup` (`MenuGroup.tsx:16-33`).
///
/// Renders the `<div role="group">` that owns the `labelId` its `Menu.GroupLabel` children
/// register into, and provides it so `Menu.RadioGroup` can share the same contract.
#[component]
pub fn Group(
    /// `className` (`MenuGroup.tsx:17`).
    #[prop(optional, into)]
    class: Option<String>,
    /// `style` (`:17`).
    #[prop(default = Vec::new())]
    style: Vec<(String, String)>,
    /// The group's contents (`MenuGroupProps.children`, `:35-39`).
    children: Children,
) -> impl IntoView {
    // `const [labelId, setLabelId] = React.useState(undefined)` (`:17`).
    let context = MenuGroupContextValue::new();

    // `<MenuGroupContext.Provider value={setLabelId}>` (`:32`).
    provide_context(send_wrapper::SendWrapper::new(context.clone()));

    // The view captures the `Copy` state signal, not the context value: the context carries
    // the non-`Send` setter handle and a view closure must be `Send` (see
    // [`menu_group_aria_labelledby`]).
    let label_id = context.label_id;

    let style_attribute = style
        .into_iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<String>();

    view! {
        <div
            role=MENU_GROUP_ROLE
            aria-labelledby=move || menu_group_aria_labelledby(label_id)
            class=class
            style=style_attribute
        >
            {children()}
        </div>
    }
}
