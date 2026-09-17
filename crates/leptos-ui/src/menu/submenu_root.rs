//! Menu submenu root — `Menu.SubmenuRoot`, the port of
//! `packages/react/src/menu/submenu-root/MenuSubmenuRoot.tsx` (64 lines) plus the one module it
//! owns, `MenuSubmenuRootContext.ts` (15 lines).
//!
//! WHAT THIS REPLACES. The previous file (spelled `submenu-root.rs`) was a facade, not a port, and
//! it had never been type-checked: a hyphen is not a legal Rust identifier, so `menu/mod.rs` could
//! never declare the module and nothing in the crate ever referenced it. Its 156 lines
//! (1) invented a props surface that is not upstream's at all — `modal`, `orientation`, `id`,
//! `defaultOpen` and `highlightItemOnHover` were ROOT-level props, while upstream's
//! `MenuSubmenuRootProps` explicitly OMITS `modal` and `openOnHover` (`MenuSubmenuRoot.tsx:27-36`)
//! precisely because a nested menu ignores them (`MenuRoot.tsx:190`; `:178-180` warns when one is
//! passed anyway); (2) rendered a hardcoded `<div class="menu-submenu-root" role="none">` — upstream
//! renders NO element at all ("Doesn't render its own HTML element", `:11`), and the hardcoded class
//! is what `specs/library/menu/behavior.md` → "Uniform DOM shell" forbids; (3) invented seven
//! helpers (`use_submenu_store`, `use_submenu_open`, `use_submenu_orientation`, `use_submenu_modal`,
//! `use_submenu_highlight_item_on_hover`, `use_submenu_root_id`, `use_submenu_parent`) with no
//! upstream counterpart; (4) constructed a `MenuStore::new()`/`MenuStoreContext` pair that does not
//! exist in this crate; and (5) set `MenuParent::Submenu { id }` — a variant that does not exist,
//! and the wrong SHAPE: upstream's submenu parent is `{ type: 'menu', store }`, i.e. the parent
//! menu's store by reference (`MenuRoot.tsx:80-85`), not an id string.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from:
//! - the required parent read `useMenuRootContext().store` (`:16`) — the parent menu's store, which
//!   is what makes this a *sub*menu rather than an independently rooted one.
//! - the `MenuSubmenuRootContext { parentMenu }` value (`MenuSubmenuRootContext.ts:9-11`), provided
//!   around the children (`:21-23`). Two readers depend on it: the inner `MenuRoot`, which resolves
//!   its `parent` from it (`MenuRoot.tsx:77,80-85`), and `Menu.SubmenuTrigger`, which throws
//!   without it (`MenuSubmenuTrigger.tsx:49-52`).
//! - the delegation `return <MenuRoot {...props} />` (`:22`), as
//!   [`MenuSubmenuRootProps::to_root_props`] — the one place the Omit list is applied.
//! - the prop omissions (`:27-36`): no `modal`, no `openOnHover`, no `onOpenChange`-minus-details,
//!   no `handle`, no `triggerId`/`defaultTriggerId`; `onOpenChange` re-declared with the submenu's
//!   own details type (`:40-41`, `:57` — whose definition IS `MenuRoot.ChangeEventDetails`).
//! - `closeParentOnEsc` (`:44,47`; `MenuRoot.tsx:67,469,729`): carried as a prop and seeded into the
//!   store's own slot, see DEFERRED below for why its consumer is not wired here.
//!
//! The context crosses leptos's `Send + Sync` bound through the `SendWrapper` bridge (the
//! `SharedMenuPositionerContext` / `SharedMenuGroupContext` precedent) rather than the thread-local
//! the ROOT context uses. That distinction is deliberate and load-bearing: a thread-local slot is
//! last-writer-wins across the whole wasm thread, so with two sibling submenus the second one's
//! Root would overwrite the slot before the first one's trigger read it. `provide_context` is
//! owner-scoped, so each submenu subtree resolves its own parent — which is the nesting behaviour
//! upstream gets from `React.createContext` (`MenuSubmenuRootContext.ts:5`).
//!
//! DEFERRED, each with the reason it is not this checkpoint's, and none of them silently dropped:
//! - `closeParentOnEsc`'s single upstream consumer is the dismissal wiring
//!   (`MenuRoot.tsx:469` — `bubbles: { escapeKey: closeParentOnEsc && parent.type === 'menu' }`),
//!   which this unit has not reached: it belongs to the `FloatingFocusManager`/`useDismiss`
//!   checkpoint named in this item's remaining-work list. The prop therefore seeds REAL store state
//!   (`MenuExtraState::close_parent_on_esc`) that the dismissal checkpoint reads, instead of being
//!   dropped or turned into an inert helper that returns a constant.
//! - The `FloatingTree` wrap that upstream applies at a floating-tree top (`MenuRoot.tsx:643-648`)
//!   is the Root's own deferral, recorded in [`crate::menu::root`]'s module docs; this module
//!   delegates to that view and adds nothing of its own.

use std::rc::Rc;

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::menu::root::{MenuRootProps, menu_root_view};
use crate::menu::store::{MenuChangeEventDetails, MenuStore, use_menu_store};

/// `MenuSubmenuRootContext` (`MenuSubmenuRootContext.ts:9-11`) — upstream's
/// `{ parentMenu: MenuStore<unknown> }`. The parent menu's store by reference, which is exactly
/// what `MenuRoot` reads to build `parent: { type: 'menu', store }` (`MenuRoot.tsx:80-85`).
#[derive(Clone)]
pub struct MenuSubmenuRootContextValue {
    /// `parentMenu` (`MenuSubmenuRootContext.ts:10`).
    pub parent_menu: MenuStore,
}

/// The shared context bridge: the value crosses `provide_context`'s `Send + Sync` bound through the
/// `SendWrapper` bridge — the [`crate::menu::positioner::SharedMenuPositionerContext`] precedent.
pub type SharedMenuSubmenuRootContext = SendWrapper<MenuSubmenuRootContextValue>;

/// Provides the submenu-root context for the subtree (`MenuSubmenuRoot.tsx:21-23`).
pub fn provide_menu_submenu_root_context(context: MenuSubmenuRootContextValue) {
    provide_context(SendWrapper::new(context));
}

/// `useMenuSubmenuRootContext()` (`MenuSubmenuRootContext.ts:13-15`) — the OPTIONAL read, whose
/// `undefined` return is what `MenuRoot` tests to decide `isSubmenu` (`MenuRoot.tsx:77`) and what
/// `Menu.SubmenuTrigger` turns into a throw (`MenuSubmenuTrigger.tsx:49-52`).
pub fn use_menu_submenu_root_context() -> Option<MenuSubmenuRootContextValue> {
    use_context::<SharedMenuSubmenuRootContext>().map(|value| value.take())
}

/// `MenuSubmenuRoot.Props` (`MenuSubmenuRoot.tsx:27-52`) in the crate's spelling: upstream's
/// `Omit<MenuRoot.Props, 'modal' | 'openOnHover' | 'onOpenChange' | 'handle' | 'triggerId' |
/// 'defaultTriggerId' | 'children'>` plus `onOpenChange` (re-declared, `:40-41`),
/// `closeParentOnEsc` (`:47`) and `children` (`:51`).
///
/// The Omit list is expressed by ABSENCE here, which is the crate's convention for a props struct
/// whose removed fields have no legal value: there is no `modal` field to pass, so the nested-menu
/// rule (`MenuRoot.tsx:190`) cannot be violated from this surface — and accordingly there is no
/// `modal` seeding in [`Self::to_root_props`]. Documented defaults live on [`Self::default`].
#[derive(Clone)]
pub struct MenuSubmenuRootProps {
    /// `open` (`MenuRoot.Props`, kept) — the controlled value; `None` while uncontrolled.
    pub open: Option<bool>,
    /// `defaultOpen` (`MenuRoot.Props`, kept) — upstream default `false`.
    pub default_open: bool,
    /// `onOpenChange` (`MenuSubmenuRoot.tsx:40-41`) — the veto point's user callback, typed with the
    /// submenu's own details (`:57`).
    pub on_open_change: Option<Rc<dyn Fn(bool, &MenuChangeEventDetails)>>,
    /// `disabled` (`MenuRoot.Props`, kept).
    pub disabled: bool,
    /// `closeParentOnEsc` (`:44`, default `false`) — when in a submenu, whether Escape closes the
    /// entire menu or only this child menu (`:42-46`).
    pub close_parent_on_esc: bool,
    /// `highlightItemOnHover` (`MenuRoot.Props`, kept) — upstream default `true`.
    pub highlight_item_on_hover: bool,
    /// `rootId` (`MenuRoot.Props`, kept).
    pub root_id: Option<String>,
}

impl Default for MenuSubmenuRootProps {
    fn default() -> Self {
        Self {
            open: None,
            default_open: false,
            on_open_change: None,
            // Upstream's own defaults for the kept props.
            disabled: false,
            // `closeParentOnEsc = false` (`MenuRoot.tsx:67`).
            close_parent_on_esc: false,
            highlight_item_on_hover: true,
            root_id: None,
        }
    }
}

impl MenuSubmenuRootProps {
    /// The `{...props}` spread of `return <MenuRoot {...props} />` (`MenuSubmenuRoot.tsx:22`) — the
    /// submenu surface mapped onto the root's, and the ONE place the two differ.
    ///
    /// `modal` is deliberately NOT mapped: upstream omits it from the submenu prop surface
    /// (`:27-36`), so a nested menu leaves it at the store's own rule (`MenuStore.ts:57-59`, whose
    /// selector makes `parent.type === 'menu'` never modal) rather than having a value seeded here.
    /// Every other member is a straight pass-through, which is why this is a mapping and not a
    /// decision.
    pub fn to_root_props(self) -> MenuRootProps {
        let Self {
            open,
            default_open,
            on_open_change,
            disabled,
            close_parent_on_esc,
            highlight_item_on_hover,
            root_id,
        } = self;

        MenuRootProps {
            open,
            default_open,
            on_open_change,
            disabled,
            // Upstream passes `modal: undefined` for a nested menu (`MenuRoot.tsx:190` is guarded on
            // `parent.type === undefined`) and omits the prop entirely from this surface
            // (`MenuSubmenuRoot.tsx:27-36`). The store's `modal` field is irrelevant for a nested
            // menu because the SELECTOR folds the parent in (`MenuStore.ts:57-59`, ported as
            // [`crate::menu::store::menu_modal`]) and answers `false` for any `parent.type ===
            // 'menu'`; so the port seeds the store's documented default and lets the fold supply the
            // nested-menu answer, rather than inventing a tri-state the field does not have.
            modal: true,
            close_parent_on_esc,
            highlight_item_on_hover,
            root_id,
        }
    }
}

/// `Menu.SubmenuRoot` — upstream's `MenuSubmenuRoot` (`MenuSubmenuRoot.tsx:15-25`).
///
/// Reads the parent menu's store from the surrounding `Menu.Root` (`:16`), provides the submenu
/// bridge the inner Root and `Menu.SubmenuTrigger` consume (`:18-23`), and renders no element of its
/// own (`:11`). See the module docs for what is ported and what is deferred.
#[component]
pub fn SubmenuRoot(
    /// The submenu root's props (`MenuSubmenuRoot.tsx:15`).
    #[prop(default = MenuSubmenuRootProps::default(), optional)]
    submenu_props: MenuSubmenuRootProps,
    /// The submenu's content — the trigger plus its own `Menu.Portal > Menu.Positioner >
    /// Menu.Popup` (`:51`).
    children: leptos::children::ChildrenFn,
) -> impl IntoView {
    // `const parentMenu = useMenuRootContext().store;` (`MenuSubmenuRoot.tsx:16`) — the REQUIRED
    // read, so a SubmenuRoot outside a parent Root panics with the crate's own message
    // (`store.rs:396-402`), which is upstream's contract for a part used outside its Root.
    let parent_menu = use_menu_store().store;

    // `const contextValue = React.useMemo(() => ({ parentMenu }), [parentMenu]);` (`:18`) and the
    // provider around `<MenuRoot />` (`:21-23`). Provided BEFORE the delegation below, so the inner
    // Root's own `useMenuSubmenuRootContext()` read (`MenuRoot.tsx:77`) sees it.
    provide_menu_submenu_root_context(MenuSubmenuRootContextValue { parent_menu });

    // `<MenuRoot {...props} />` (`:22`) — the port's provider-only root view.
    menu_root_view(submenu_props.to_root_props(), children)
}
