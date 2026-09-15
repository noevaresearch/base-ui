//! Context Menu Root — port of `packages/react/src/context-menu/root/ContextMenuRoot.tsx`.
//!
//! Upstream renders NO element (`ContextMenuRoot.tsx:46-52`): the Root returns a
//! two-layer provider sandwich — `ContextMenuRootContext.Provider` →
//! `MenuRootContext.Provider value={undefined}` → `Menu.Root` — and nothing else.
//! The `undefined` MenuRootContext severs any enclosing Menu/Menubar context so
//! MenuRoot's parent detection lands on `{ type: 'context-menu', context }`
//! (`MenuRoot.tsx:94-102`), keeping a Context Menu mounted inside another menu's
//! subtree a standalone root.
//!
//! The only React state in the unit is the anchor virtual element
//! (`ContextMenuRoot.tsx:17-21`), seeded with a zero-size rect at the origin so the
//! positioner always has a well-formed `getBoundingClientRect` before the first open.
//! Everything else is coordination refs (`:23-28`), crossed through the
//! `ContextMenuRootContext` (`ContextMenuRootContext.ts:5-17`).

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;

use crate::menu::store::{
    MenuChangeEventDetails, MenuStore, MenuStoreContext, create_menu_store_with_on_open_change,
};

/// The virtual-element anchor (`ContextMenuRoot.tsx:17-21`, `:58-64`): a
/// `getBoundingClientRect` over a cursor point that corresponds to no DOM node —
/// zero-size for mouse, 10×10 for touch (`ContextMenuTrigger.tsx:57-66`). The whole
/// floating-ui positioning pipeline runs unchanged against this rect.
#[derive(Clone, Debug)]
pub struct VirtualAnchor {
    /// The spawn point, in viewport (`clientX`/`clientY`) coordinates.
    pub x: f64,
    /// The spawn point's y.
    pub y: f64,
    /// The synthetic rect's size: 0 for mouse opens, 10 for touch (`:59-65`).
    pub size: f64,
}

impl VirtualAnchor {
    /// The mouse seed / the Root's initial value — a zero-size rect at the origin
    /// (`ContextMenuRoot.tsx:19-21`) so the positioner always has a well-formed rect.
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0, size: 0.0 }
    }

    /// The rect the positioner reads (`ContextMenuRoot.tsx:58-64` —
    /// `DOMRect.fromRect({ width, height, x, y })`).
    pub fn rect(&self) -> (f64, f64, f64, f64) {
        (self.x, self.y, self.size, self.size)
    }
}

/// `ContextMenuRootContext` (`ContextMenuRootContext.ts:5-17`) — the coordination
/// surface the trigger and the shared Menu parts read and write. Refs become
/// clone-shared `Rc` cells (the dialog port's context-ref convention, `dialog/mod.rs:169-181`).
#[derive(Clone)]
pub struct ContextMenuRootContext {
    /// `anchor` / `setAnchor` (`:8-9`) — written by the trigger on open
    /// (`ContextMenuTrigger.tsx:57-66`); read by the positioner as the default
    /// anchor for context-menu parents (`MenuPositioner.tsx:88-95`).
    pub anchor: Rc<RefCell<VirtualAnchor>>,
    /// `actionsRef` (`:10`) — filled by MenuRoot with `{ setOpen }` via
    /// `useImperativeHandle` (`MenuRoot.tsx:465`); called by the trigger to open and
    /// cancel (`ContextMenuTrigger.tsx:69`, `:112-115`). The port's MenuRoot fills
    /// it with the store directly (the `setOpen` emission, `MenuStore.ts:165-167`,
    /// routes through the Root-owned gate synchronously — upstream's
    /// imperative-handle indirection is React commit timing, which has no
    /// counterpart in the port's synchronous call chain).
    pub actions: RefCell<Option<MenuStore>>,
    /// `positionerRef` (`:11`) — filled with the store's `positionerElement`
    /// (`MenuRoot.tsx:459-463`); read by the trigger's mouseup handler to skip
    /// cancellation for targets inside the positioner (`ContextMenuTrigger.tsx:104-106`).
    pub positioner_element: Rc<RefCell<Option<web_sys::Element>>>,
    /// `backdropRef` (`:12`) — filled by a user-rendered `MenuBackdrop`
    /// (`MenuBackdrop.tsx:36-39`); read by the trigger's document `contextmenu`
    /// prevention (`ContextMenuTrigger.tsx:182-185`).
    pub backdrop_element: Rc<RefCell<Option<web_sys::HtmlElement>>>,
    /// `internalBackdropRef` (`:13`) — filled by the `InternalBackdrop` the
    /// positioner renders for context-menu parents (`MenuPositioner.tsx:302-312`);
    /// same prevention check as `backdropRef`.
    pub internal_backdrop_element: Rc<RefCell<Option<web_sys::HtmlElement>>>,
    /// `allowMouseUpTriggerRef` (`:14`) — the tree-wide item-activation gate, shared
    /// with every item through the store (`MenuStore.ts:157-159`, `:153`); starts
    /// `true` (`ContextMenuRoot.tsx:27`).
    pub allow_mouse_up_trigger: Rc<std::cell::Cell<bool>>,
    /// `initialCursorPointRef` (`:15`) — written on open
    /// (`ContextMenuTrigger.tsx:55`); consumed-and-cleared by item activation
    /// (`useMenuItemCommonProps.ts:88-89`).
    pub initial_cursor_point: Rc<RefCell<Option<(f64, f64)>>>,
    /// `rootId` (`:16`) — the tree identity stamped on every popup in the tree as
    /// `data-rootownerid` (`MenuPopup.tsx:110`) and matched by `findRootOwnerId`'s
    /// ancestor walk in the trigger's mouseup handler (`ContextMenuTrigger.tsx:108-110`)
    /// — so a mouseup inside any portaled popup of this tree, including submenus,
    /// never cancels the menu.
    pub root_id: String,
}

thread_local! {
    static CONTEXT_MENU_ROOT_CONTEXT: RefCell<Option<ContextMenuRootContext>> =
        const { RefCell::new(None) };
    /// The saved enclosing value — the `MenuRootContext.Provider value={undefined}`
    /// severing (`ContextMenuRoot.tsx:50-52`), realized as save/clear/restore around
    /// the children's mount: a nested ContextMenu Root clears the outer value for the
    /// extent of its own children, then restores it (the React context-scope
    /// semantics, `MenuRoot.tsx:94-102` reads the *enclosing* provider's value).
    static ENCLOSING_MENU_CONTEXT: RefCell<Option<MenuStoreContext>> =
        const { RefCell::new(None) };
}

/// Provides the context-menu root context for the subtree and severs the enclosing
/// menu context (`ContextMenuRoot.tsx:46-52`). Crate-visible so the test suite can
/// stage the provider without a full mount.
pub(crate) fn provide_context_menu_root_context(
    context: ContextMenuRootContext,
    enclosing: Option<MenuStoreContext>,
) {
    CONTEXT_MENU_ROOT_CONTEXT.with(|slot| *slot.borrow_mut() = Some(context));
    ENCLOSING_MENU_CONTEXT.with(|slot| *slot.borrow_mut() = enclosing);
}

/// Restores the severed enclosing menu context after the children have mounted —
/// the provider scope closes where the Root's element closes (`ContextMenuRoot.tsx:46-52`).
pub fn restore_enclosing_menu_context() {
    let saved = ENCLOSING_MENU_CONTEXT.with(|slot| slot.borrow_mut().take());
    // The severing slot is now None (this Root's children no longer read the outer
    // value); the outer value (if any) is re-provided for the siblings outside this
    // Root — the React context scope closing.
    if let Some(value) = saved {
        crate::menu::store::provide_menu_root_context(value);
    }
}

/// `useContextMenuRootContext(false)` (`ContextMenuRootContext.ts:23-32`) — the
/// required form: panics with the standard missing-provider error outside a Root.
pub fn use_context_menu_root_context() -> ContextMenuRootContext {
    CONTEXT_MENU_ROOT_CONTEXT
        .with(|slot| slot.borrow().clone())
        .expect(
            "Base UI: ContextMenu parts must be used within <ContextMenu.Root> (the \
             ContextMenuRootContext is missing).",
        )
}

/// `useContextMenuRootContext(true)` (`:9`) — the optional form the MenuRoot parent
/// detection uses.
pub fn use_context_menu_root_context_optional() -> Option<ContextMenuRootContext> {
    CONTEXT_MENU_ROOT_CONTEXT.with(|slot| slot.borrow().clone())
}

/// The Root props — upstream's `ContextMenuRootProps` (`ContextMenuRoot.tsx:73-102`):
/// `Menu.Root.Props` minus the props that make no sense here (`handle`, `triggerId`,
/// `defaultTriggerId`, `modal`, `openOnHover`, `delay`, `closeDelay`,
/// `closeParentOnEsc`, the render-function `children`), with `onOpenChange`
/// re-declared to narrow the details type. `modal` is forced on for context menus by
/// the store's `modal` selector (`MenuStore.ts:57-59`) — hence omitted.
#[derive(Clone, Default)]
pub struct ContextMenuRootProps {
    /// `open` — the controlled open state (renders open with no interaction,
    /// `ContextMenuRoot.test.tsx:296`); `None` while uncontrolled.
    pub open: Option<bool>,
    /// `defaultOpen` — the uncontrolled initial state (`:60-71`); upstream default
    /// `false` (`MenuRoot.tsx:65`).
    pub default_open: bool,
    /// `onOpenChange(nextOpen, eventDetails)` — the details carry a `reason`
    /// (`ContextMenuRoot.test.tsx:95-98`).
    pub on_open_change: Option<Rc<dyn Fn(bool, &MenuChangeEventDetails)>>,
    /// `disabled` — the hard gate on every open path
    /// (`ContextMenuRoot.test.tsx:264`); `MenuStore.ts:20`.
    pub disabled: bool,
    /// The deprecated `closeParentOnEsc` no-op (`ContextMenuRoot.tsx:80-84`) — kept
    /// for API parity, has no effect (behavior.md "Anything in source not explained
    /// by any test", item 7).
    #[allow(dead_code)]
    pub close_parent_on_esc: bool,
}

/// Creates the Root's store and context value — the hook-shaped entry the view
/// calls. The store is the Menu store with `modal` forced on
/// (`MenuStore.ts:57-59` — context menus are always modal) and the parent resolved
/// to `{ type: 'context-menu' }` (`MenuRoot.tsx:97-102` — the layout-effect sync at
/// `MenuRoot.tsx:253-277`).
pub fn use_context_menu_root(
    props: ContextMenuRootProps,
) -> (MenuStore, ContextMenuRootContext) {
    let ContextMenuRootProps {
        open: open_prop,
        default_open,
        on_open_change,
        disabled,
        close_parent_on_esc: _,
    } = props;

    // The Menu store (`useMenuRootStore`, `MenuRoot.tsx:651-666`) with the user's
    // `onOpenChange` on the context slot — the veto point.
    let store = create_menu_store_with_on_open_change(on_open_change.map(|callback| {
        Rc::new(move |open: bool, details: &MenuChangeEventDetails| callback(open, details))
            as Rc<dyn Fn(bool, &MenuChangeEventDetails)>
    }));

    // The controlled-prop sync + the uncontrolled seed (`MenuRoot.tsx:132-150`).
    store.set_field(|state| &mut state.open_prop, open_prop);
    if default_open {
        store.set_field(|state| &mut state.open, true);
    }

    // The seeded extra state: `modal` forced on (`MenuStore.ts:57-59`), `disabled`
    // mirrored (`MenuStore.ts:20`), and the parent resolved to the context-menu arm
    // (`MenuRoot.tsx:97-102` + the layout-effect sync at `:253-277`).
    store.set_field(
        |state| &mut state.payload.get_or_insert_with(Default::default).modal,
        true,
    );
    store.set_field(
        |state| &mut state.payload.get_or_insert_with(Default::default).disabled,
        disabled,
    );
    store.set_field(
        |state| &mut state.payload.get_or_insert_with(Default::default).parent,
        crate::menu::store::MenuParent::ContextMenu,
    );

    // The context value (`ContextMenuRoot.tsx:31-44`) — the anchor seeded with the
    // zero-size origin rect (`:19-21`), `allowMouseUpTriggerRef` starting `true`
    // (`:27`), and the `rootId` from `useId` (`:29`).
    let context = ContextMenuRootContext {
        anchor: Rc::new(RefCell::new(VirtualAnchor::origin())),
        actions: RefCell::new(Some(std::rc::Rc::clone(&store))),
        positioner_element: Rc::new(RefCell::new(None)),
        backdrop_element: Rc::new(RefCell::new(None)),
        internal_backdrop_element: Rc::new(RefCell::new(None)),
        allow_mouse_up_trigger: Rc::new(std::cell::Cell::new(true)),
        initial_cursor_point: Rc::new(RefCell::new(None)),
        root_id: {
            use reactive_graph::signal::RwSignal as RgRwSignal;
            let id_override = RgRwSignal::new(None::<String>);
            let id_signal = leptos_ui_utils::use_id(id_override, Some("ctx"));
            use reactive_graph::traits::GetUntracked;
            format!("context-menu-{}", id_signal.get_untracked())
        },
    };

    (store, context)
}

/// Renders the Root — the provider sandwich plus the children, no element
/// (`ContextMenuRoot.tsx:46-52`).
pub fn context_menu_root_view(
    props: ContextMenuRootProps,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    let (store, context) = use_context_menu_root(props);

    // The severing: read the enclosing menu context, clear it, provide the
    // context-menu context, then mount the children — the
    // `MenuRootContext.Provider value={undefined}` scope (`:50-52`).
    let enclosing = crate::menu::store::use_menu_root_context_optional();
    // Fill the actions slot (the `useImperativeHandle` wiring, `MenuRoot.tsx:465`)
    // before the context moves into the provider slot.
    context.actions.replace(Some(std::rc::Rc::clone(&store)));
    provide_context_menu_root_context(context, enclosing);

    // The children mount inside the severed scope; the restoration below closes it
    // (the port's synchronous-mount equivalent of the provider's element scope).
    let rendered = children();
    crate::menu::store::clear_menu_root_context();
    restore_enclosing_menu_context();

    view! { <>{rendered}</> }
}

/// The `ContextMenu.Root` component.
#[leptos::component]
pub fn ContextMenuRootComponent(
    #[prop(default = ContextMenuRootProps::default(), optional)] context_menu_props:
        ContextMenuRootProps,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    context_menu_root_view(context_menu_props, children)
}
