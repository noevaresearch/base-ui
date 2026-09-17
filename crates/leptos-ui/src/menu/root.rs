//! Menu root — the state owner. Port of `packages/react/src/menu/root/MenuRoot.tsx`
//! (the open/close spine this iteration ports; the hover/dismissal/navigation
//! wiring is deferred to later checkpoints — see the item's TODO note).
//!
//! Upstream renders NO element: the Root returns the provider, the optional handle
//! attachment, and the children — optionally wrapped in a `FloatingTree` when it is
//! the top of a floating tree (`MenuRoot.tsx:636-648`). The previous port rendered a
//! `<div class="menu-root">`, which behavior.md → "Uniform DOM shell" forbids.

use std::rc::Rc;

use leptos::prelude::*;

use crate::menu::store::{
    MenuChangeEventDetails, MenuParent, MenuStore, create_menu_store_with_on_open_change,
    provide_menu_root_context,
};
use crate::menu::submenu_root::use_menu_submenu_root_context;

/// The root props — upstream's `MenuRootProps` subset whose behavior this
/// iteration ports. The `orientation` prop stays for the menubar-family callers
/// (upstream's Root itself has none — the menubar owns orientation); the store
/// seeds it.
#[derive(Clone)]
pub struct MenuRootProps {
    /// `open` (`MenuRoot.tsx`) — the controlled value; `None` while uncontrolled.
    pub open: Option<bool>,
    /// `defaultOpen` — upstream default `false`.
    pub default_open: bool,
    /// `onOpenChange` — the veto point's user callback.
    pub on_open_change: Option<Rc<dyn Fn(bool, &MenuChangeEventDetails)>>,
    /// `disabled` (`MenuStore.ts:20`).
    pub disabled: bool,
    /// `modal` (`MenuStore.ts:21`, default `true`).
    pub modal: bool,
    /// `highlightItemOnHover` (`MenuStore.ts:24`, default `true`).
    pub highlight_item_on_hover: bool,
    /// `closeParentOnEsc` (`MenuRoot.tsx:729`, default `false`) — whether Escape in a submenu closes
    /// the whole menu instead of only the child (`:67,469`). Carried on the Root because upstream
    /// declares it on `MenuRootProps` and `Menu.SubmenuRoot` re-declares it
    /// (`MenuSubmenuRoot.tsx:47`) rather than omitting it.
    pub close_parent_on_esc: bool,
    /// `rootId` (`MenuStore.ts:26`).
    pub root_id: Option<String>,
}

impl Default for MenuRootProps {
    fn default() -> Self {
        Self {
            open: None,
            default_open: false,
            on_open_change: None,
            disabled: false,
            modal: true,
            highlight_item_on_hover: true,
            // `closeParentOnEsc = false` (`MenuRoot.tsx:67`).
            close_parent_on_esc: false,
            root_id: None,
        }
    }
}

/// Creates the Root's store and context value — the hook-shaped entry the view
/// calls (`useRenderDialogRoot` precedent). The store is created exactly once per
/// Root body run (`useMenuRootStore`, `MenuRoot.tsx:651-666`).
pub fn use_menu_root(
    props: MenuRootProps,
) -> (MenuStore, crate::menu::store::MenuRootContextValue) {
    let MenuRootProps {
        open: open_prop,
        default_open,
        on_open_change,
        disabled,
        modal,
        highlight_item_on_hover,
        close_parent_on_esc,
        root_id,
    } = props;

    // The store (`:651-666`) with the user's `onOpenChange` on the context slot —
    // the veto point in [`crate::menu::store::menu_set_open`] invokes it. The
    // dialog port writes the slot at construction (`dialog/mod.rs:49-63`); the port
    // does the same via the context-taking constructor.
    let store: MenuStore = create_menu_store_with_on_open_change(on_open_change.map(|callback| {
        Rc::new(move |open: bool, details: &MenuChangeEventDetails| callback(open, details))
            as Rc<dyn Fn(bool, &MenuChangeEventDetails)>
    }));

    // The controlled-prop sync (`:149-150` — the store's controlled-prop
    // machinery; the port writes the prop into the raw field once, the coalescing
    // selector reads it with the prop winning).
    store.set_field(|state| &mut state.open_prop, open_prop);

    // The seeded extra state (`:187-193` — `useSyncedValues` mirrors the props into
    // the store; the port seeds them once here).
    store.set_field(
        |state| &mut state.payload.get_or_insert_with(Default::default).disabled,
        disabled,
    );
    store.set_field(
        |state| &mut state.payload.get_or_insert_with(Default::default).modal,
        modal,
    );
    store.set_field(
        |state| {
            &mut state
                .payload
                .get_or_insert_with(Default::default)
                .highlight_item_on_hover
        },
        highlight_item_on_hover,
    );
    store.set_field(
        |state| &mut state.payload.get_or_insert_with(Default::default).root_id,
        root_id,
    );

    // `closeParentOnEsc` (`:67,729`) into the store's own slot. Its reader is the dismissal wiring
    // (`:469`), deferred with this unit's listener checkpoint; the state exists so that checkpoint
    // reads a real value instead of a constant.
    store.set_field(
        |state| {
            &mut state
                .payload
                .get_or_insert_with(Default::default)
                .close_parent_on_esc
        },
        close_parent_on_esc,
    );

    // The parent descriptor (`MenuRoot.tsx:77-107`).
    //
    // Upstream resolves it from THREE optional contexts in a fixed order (`:79-107`): the submenu
    // bridge first, then menubar, then context-menu, else `{ type: undefined }`. The port resolves
    // the submenu arm — the one whose provider exists in this crate — from the bridge
    // `Menu.SubmenuRoot` provides (`submenu_root.rs`, `MenuSubmenuRoot.tsx:21-23`), and leaves the
    // menubar/context-menu arms to the units that own their contexts:
    //
    //     if (isSubmenu && parentMenuRootContext) return { type: 'menu', store: parentMenuRootContext.store };
    //
    // Note this reads `parentMenuRootContext` — the enclosing ROOT's store (`:78`) — which is the
    // same store `Menu.SubmenuRoot` captured before delegating here. `isSubmenu` being present is
    // what makes this Root a submenu at all, and it is what makes the `modal` SELECTOR reachable in
    // its false branch (`MenuStore.ts:57-59`).
    let parent = match use_menu_submenu_root_context() {
        Some(submenu_root) => MenuParent::Menu {
            store: submenu_root.parent_menu,
        },
        None => MenuParent::None,
    };
    store.set_field(
        |state| &mut state.payload.get_or_insert_with(Default::default).parent,
        parent,
    );

    // The uncontrolled seed (`defaultOpen`, `:132-147` — the store's initial
    // `open`).
    if default_open {
        store.set_field(|state| &mut state.open, true);
    }

    let _ = &store;
    let context = crate::menu::store::MenuRootContextValue {
        store: std::rc::Rc::clone(&store),
    };
    (store, context)
}

/// Renders the Root — the context provision plus the children, no element
/// (`MenuRoot.tsx:636-648`).
pub fn menu_root_view(
    props: MenuRootProps,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    let (_store, context) = use_menu_root(props);
    provide_menu_root_context(context);
    // The FloatingTree wrap for floating-tree-top parents (`:643-648`) arrives with
    // the dismissal checkpoint — the provider-only spine is the contract here.
    view! { <>{children()}</> }
}

/// The `Menu.Root` part — upstream's `MenuRoot` (`packages/react/src/menu/root/MenuRoot.tsx`),
/// named the way upstream's `index.parts.ts:16` names it (`export { MenuRoot as Root }`).
#[leptos::component]
pub fn Root(
    #[prop(default = MenuRootProps::default(), optional)] menu_props: MenuRootProps,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    menu_root_view(menu_props, children)
}

/// The orientation the menubar-family callers read (the store seeds it; upstream's
/// Root has no orientation prop — the menubar owns it).
pub use crate::menu::store::MenuOrientation;

/// Reads the provided root context (the optional-pattern accessor the Trigger
/// uses) — re-exported for the parts.
pub use crate::menu::store::use_menu_root_context_optional as use_optional_menu_root_context;
