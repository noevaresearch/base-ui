//! Menu store — `packages/react/src/menu/store/MenuStore.ts` over the shared popup
//! machinery (`popup_store_utils::PopupStore`), plus the root-owned open/close
//! transition gate (`MenuRoot.tsx:308-413`).
//!
//! Why a rewrite of the previous facade: the prior `MenuStoreContext` minted a fresh
//! store per `use_menu_store()` call (parts could never share state — `MenuRoot`'s
//! provision was inert), `MenuRoot` rendered a `<div>` where upstream renders no
//! element, and every part toggled the open signal directly, bypassing the one
//! mutation gate the upstream spec mandates ("Every open/close request … funnels
//! into `store.setOpen`", implementation.md "One mutation gate"). This module
//! replaces that with the dialog/collapsible house pattern:
//!
//! - the store is a [`MenuStore`] — the shared `PopupStoreState` spine (with the
//!   menu's extra state riding the payload slot) plus the menu-specific members
//!   (`MenuStore.ts:19-38`);
//! - [`menu_set_open`] ports `MenuRoot`'s `setOpen` transition: stale-close guard,
//!   same-state dedupe, the veto point, the triggerless-close trigger backfill, the
//!   floating-root dispatch, and the `instantType` derivation — all cited to
//!   `MenuRoot.tsx` line ranges;
//! - parts obtain the store from the provided context ([`use_menu_root_context`])
//!   and route every open/close request through [`store_set_open`] (the
//!   `store.setOpen` emission, `MenuStore.ts:165-167`).
//!
//! Deliberately NOT in this iteration (recorded in the item's TODO note): the
//! `observe('parent')` store inheritance, the hover/typeahead/list-navigation
//! wiring, dismissal hooks, focus management, and the portals — each is its own
//! checkpoint. The gate below is the contract everything else composes through.

use std::cell::Cell;
use std::rc::Rc;

use leptos::prelude::*;
use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked};
use wasm_bindgen::JsCast;

use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::floating_ui::popup_store::{self, InstantType, PopupStoreState};
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_internals::floating_ui::types::RootOpenChangeEventDetails;
use leptos_ui_internals::popup_store_utils::{
    PopupStore, attach_prevent_unmount_on_close, create_popup_open_state,
};

/// The event details type — `MenuRoot.ChangeEventDetails`. The shared machinery
/// (`attach_prevent_unmount_on_close`, `dispatch_open_change`) is typed over the
/// floating layer's `BaseUIChangeEventDetails<String>`, so the menu's details are
/// that type (the custom payload stays empty at the floating layer,
/// `types.rs:645-648`).
pub type MenuChangeEventDetails = RootOpenChangeEventDetails;

/// The menu-family reason strings (`reason-parts.ts:8,35`) — reasons belong to the
/// component that emits them (the floating_ui::reasons module-docs rule); the shared
/// registry carries only the floating-ui emitters.
pub mod reasons_menu {
    /// `REASONS.itemPress` (`reason-parts.ts:8`).
    pub const ITEM_PRESS: &str = "item-press";
    /// `REASONS.siblingOpen` (`reason-parts.ts:35`).
    pub const SIBLING_OPEN: &str = "sibling-open";
}

// ---------------------------------------------------------------------------
// Menu-specific store state (`MenuStore.ts:19-38`)
// ---------------------------------------------------------------------------

/// The menu-specific members layered over the shared [`PopupStoreState`] —
/// `MenuStore.ts:19-38` (the `PopupStoreState<Payload> & {...}` intersection; the
/// shared fields live on the spine, the menu fields ride the payload slot).
#[derive(Clone, Debug)]
pub struct MenuExtraState {
    /// `disabled` (`MenuStore.ts:20`).
    pub disabled: bool,
    /// `modal` (`MenuStore.ts:21`) — only meaningful for root/context-menu parents,
    /// default `true` (`MenuStore.ts:57-59`).
    pub modal: bool,
    /// `allowMouseEnter` (`MenuStore.ts:23`).
    pub allow_mouse_enter: bool,
    /// `highlightItemOnHover` (`MenuStore.ts:24`).
    pub highlight_item_on_hover: bool,
    /// `parent` (`MenuStore.ts:25`) — the resolved parent descriptor.
    pub parent: MenuParent,
    /// `rootId` (`MenuStore.ts:26`).
    pub root_id: Option<String>,
    /// `activeIndex` (`MenuStore.ts:27`).
    pub active_index: Option<usize>,
    /// `hoverEnabled` (`MenuStore.ts:28`).
    pub hover_enabled: bool,
    /// `instantType` (`MenuStore.ts:29`) — the menu union includes `'group'`
    /// (menubar focus/hover/navigation/sibling reasons, `MenuRoot.tsx:386-391`),
    /// which the shared `InstantType` enum lacks.
    pub instant_type: Option<MenuInstantType>,
    /// `openChangeReason` (`MenuStore.ts:30`).
    pub open_change_reason: Option<String>,
    /// `openMethod` (`MenuStore.ts:22`) — how the menu was opened, `None` until it is opened.
    /// `MenuSubmenuTrigger.tsx:182` reads it (with `lastOpenChangeReason`) to decide whether the
    /// open came from the keyboard; the WRITER is the Root's open transition, deferred with the
    /// listener checkpoint this unit's item names, so the slot exists and is read but is not
    /// populated yet — recorded rather than dressed up.
    pub open_method: Option<String>,
    /// `closeDelay` (`MenuStore.ts:35`).
    pub close_delay: u32,
    /// `closeParentOnEsc` (`MenuRoot.tsx:67,729`) — whether Escape in a submenu closes the whole
    /// menu instead of the child. Seeded by the Root from the prop
    /// ([`crate::menu::submenu_root::MenuSubmenuRootProps::close_parent_on_esc`]); its single reader
    /// upstream is the dismissal wiring (`MenuRoot.tsx:469`), which this port has not reached.
    pub close_parent_on_esc: bool,
}

/// The menu `instantType` union (`MenuStore.ts:29`): the shared popup values plus
/// the menu-only `'group'` (`MenuRoot.tsx:386-391`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuInstantType {
    /// `'dismiss'`.
    Dismiss,
    /// `'click'`.
    Click,
    /// `'group'` — menubar-family opens (focus/hover/navigation/sibling).
    Group,
    /// `'trigger-change'`.
    TriggerChange,
}

impl From<InstantType> for MenuInstantType {
    fn from(value: InstantType) -> Self {
        match value {
            InstantType::Dismiss => MenuInstantType::Dismiss,
            InstantType::Click => MenuInstantType::Click,
            InstantType::TriggerChange => MenuInstantType::TriggerChange,
            // The shared enum's Delay/Focus values are not in the menu union
            // (`MenuStore.ts:29`); the menu gate never emits them — the mapping
            // exists only for exhaustiveness.
            InstantType::Delay | InstantType::Focus => MenuInstantType::Group,
        }
    }
}

/// The resolved `MenuParent` descriptor (`MenuRoot.tsx:786-806`). The submenu arm
/// carries the parent menu's store (the `observe('parent')` inheritance source);
/// the menubar and context-menu context handles arrive with their Phase B units.
#[derive(Clone)]
pub enum MenuParent {
    /// `{ type: undefined }` — a top-level menu (`MenuRoot.tsx:104-106`).
    None,
    /// `{ type: 'menu', store }` — a submenu whose parent menu's store is shared
    /// (`MenuRoot.tsx:80-85`).
    Menu { store: MenuStore },
    /// `{ type: 'menubar', context }` — menubar children (context arrives with the
    /// menubar unit; the discriminant is what the gate and selectors need).
    Menubar,
    /// `{ type: 'context-menu', context }`.
    ContextMenu,
}

impl PartialEq for MenuParent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MenuParent::None, MenuParent::None) => true,
            (MenuParent::Menubar, MenuParent::Menubar) => true,
            (MenuParent::ContextMenu, MenuParent::ContextMenu) => true,
            // Store identity, not state equality (`store: parentMenuRootContext.store`
            // is shared by reference, `MenuRoot.tsx:80-85`).
            (MenuParent::Menu { store: a }, MenuParent::Menu { store: b }) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Default for MenuExtraState {
    fn default() -> Self {
        Self {
            disabled: false,
            modal: true,
            allow_mouse_enter: true,
            highlight_item_on_hover: true,
            parent: MenuParent::None,
            root_id: None,
            active_index: None,
            hover_enabled: true,
            instant_type: None,
            open_change_reason: None,
            // `openMethod: null` (`MenuStore.ts:214`).
            open_method: None,
            close_delay: 0,
            // `closeParentOnEsc = false` (`MenuRoot.tsx:67`).
            close_parent_on_esc: false,
        }
    }
}

/// The menu store handle — upstream's `MenuStore<Payload>` class instance
/// (`MenuStore.ts:114`), realized as the shared [`PopupStore`] with the menu extra
/// state riding the payload slot (the dialog port's store-is-the-Rc convention,
/// `dialog/mod.rs:49-63`).
pub type MenuStore = PopupStore<MenuExtraState>;

/// The store context `MenuRootContext { store, parent }` provides
/// (`MenuRootContext.ts:6-11`; the port's `parent` lives in the extra state).
#[derive(Clone)]
pub struct MenuRootContextValue {
    /// `store` (`MenuRootContext.ts:7`).
    pub store: MenuStore,
}

// ---------------------------------------------------------------------------
// Store construction (`useMenuRootStore`, MenuRoot.tsx:651-666)
// ---------------------------------------------------------------------------

/// The initial state seed (`MenuStore.ts:211` seeds from
/// `createInitialPopupStoreState`): the shared spine plus the menu defaults on the
/// payload slot (`MenuStore.ts:19-38`; `modal`/`hoverEnabled`/`highlightItemOnHover`
/// mirror upstream's defaults).
pub fn create_initial_menu_store_state() -> PopupStoreState<MenuExtraState> {
    let trigger_elements =
        leptos_ui_internals::floating_ui::popup_trigger_map::PopupTriggerMap::new();
    let mut state = popup_store::create_initial_popup_store_state(&trigger_elements, None, false);
    state.payload = Some(MenuExtraState::default());
    state
}

/// Creates the store exactly once for a Root (`useRefWithInit` at
/// `MenuRoot.tsx:651-666`; the port's Root body runs once, the same convention the
/// dialog port documents).
pub fn create_menu_store() -> MenuStore {
    Rc::new(leptos_ui_utils::react_store::ReactStore::with_context(
        create_initial_menu_store_state(),
        leptos_ui_internals::floating_ui::popup_store::PopupStoreContext {
            trigger_elements:
                leptos_ui_internals::floating_ui::popup_trigger_map::PopupTriggerMap::new(),
            popup_ref: Rc::new(Cell::new(None)),
            on_open_change: None,
            on_open_change_complete: None,
        },
    ))
}

// ---------------------------------------------------------------------------
// The one mutation gate
// ---------------------------------------------------------------------------

/// `store.setOpen` (`MenuStore.ts:165-167`): every open/close request routes here;
/// the Root-owned [`menu_set_open`] is the single consumer. The port routes the call
/// directly — the emission is synchronous, so nothing is dropped (upstream's
/// layout-effect-vs-passive subscription concern is React commit timing, which has
/// no counterpart in the port's synchronous call chain).
pub fn store_set_open(store: &MenuStore, next_open: bool, event_details: MenuChangeEventDetails) {
    menu_set_open(store, next_open, event_details);
}

/// `MenuRoot`'s `setOpen` transition (`MenuRoot.tsx:308-413`) — the one mutation
/// gate. Order is load-bearing:
///
/// 1. stale-close guard (`:317-319`) — a close is dropped when the store is
///    already closed (read directly, not the render-captured value);
/// 2. same-state/same-reason/same-trigger dedupe (`:321-327`);
/// 3. `attachPreventUnmountOnClose` (`:329-331`);
/// 4. triggerless close backfills the active trigger (`:335-337`);
/// 5. the veto point: `onOpenChange` runs before any state change and
///    `details.cancel()` aborts (`:339-343`);
/// 6. the floating-root dispatch commits the flip (`:345`);
/// 7. the touch shields (`:347-368`) — deferred with the hover/touch checkpoints
///    (the timeout-machinery wiring is its own checkpoint; recorded in the item's
///    TODO note);
/// 8. `instantType` derivation (`:370-405`) landing in the same `store.update`
///    (`:411`).
pub fn menu_set_open(
    store: &MenuStore,
    next_open: bool,
    mut event_details: MenuChangeEventDetails,
) {
    // 1. The stale-close guard (`:317-319`) — the shared selector reads the
    //    `openProp ?? open` pair directly off the store.
    if !next_open && !popup_store::selectors::open(&store.get_snapshot()) {
        return;
    }

    // 2. The same-state dedupe (`:321-327`): upstream compares the render-captured
    //    `open` against `nextOpen`, the active trigger element against the details'
    //    trigger, and the last reason against this request's reason. The port reads
    //    all three off the store (the render-capture analog is the snapshot).
    {
        let snapshot = store.get_snapshot();
        let extra = snapshot.payload.clone().unwrap_or_default();
        let current_open = popup_store::selectors::open(&snapshot);
        let same_trigger = match (&snapshot.active_trigger_element, &event_details.trigger) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        };
        if current_open == next_open
            && same_trigger
            && extra.open_change_reason.as_deref() == Some(event_details.reason.as_str())
        {
            return;
        }
    }

    // 3. `attachPreventUnmountOnClose` (`:329-331`).
    let prevent_unmount_requested = attach_prevent_unmount_on_close(&mut event_details);

    // 4. The triggerless close backfill (`:335-337`).
    if !next_open && event_details.trigger.is_none() {
        let snapshot = store.get_snapshot();
        event_details.trigger = snapshot.active_trigger_element.clone();
    }

    // 5. The veto point (`:339-343`).
    if let Some(on_open_change) = &store.context.on_open_change {
        on_open_change(next_open, &event_details);
    }
    if event_details.is_canceled() {
        return;
    }

    // 6. The floating-root dispatch commits the flip (`:345`).
    store
        .get_snapshot()
        .floating_root_context
        .dispatch_open_change(next_open, &event_details);

    // 7. The touch shields (`:347-368`) — deferred (see the doc header).

    // 8. The `instantType` derivation (`:370-405`) + the single commit (`:411`).
    let reason = event_details.reason.clone();
    // The instanceof-based keyboard-click heuristic needs a JS runtime: cloning
    // or `dyn_ref`-ing the native event panics on the host target (no JS
    // runtime, so no event can be a MouseEvent — matches the host/wasm split in
    // dialog_tests.rs and create_base_ui_event_details.rs).
    #[cfg(target_arch = "wasm32")]
    let native_event: Option<web_sys::Event> = Some(event_details.event.clone());
    #[cfg(target_arch = "wasm32")]
    let is_mouse_event = native_event
        .as_ref()
        .and_then(|e| e.dyn_ref::<web_sys::MouseEvent>())
        .is_some_and(|m| m.detail() == 0);
    #[cfg(not(target_arch = "wasm32"))]
    let is_mouse_event = false;
    let is_keyboard_click =
        (reason == reasons::TRIGGER_PRESS || reason == reasons_menu::ITEM_PRESS) && is_mouse_event;
    let is_dismiss_close = !next_open && (reason == reasons::ESCAPE_KEY || reason == reasons::NONE);

    let parent_is_menubar = matches!(
        store.get_snapshot().payload,
        Some(MenuExtraState {
            parent: MenuParent::Menubar,
            ..
        })
    );
    let instant_type = if parent_is_menubar
        && matches!(
            reason.as_str(),
            x if x == reasons::TRIGGER_FOCUS
                || x == reasons::FOCUS_OUT
                || x == reasons::TRIGGER_HOVER
                || x == reasons::LIST_NAVIGATION
                || x == reasons_menu::SIBLING_OPEN
        ) {
        Some(MenuInstantType::Group)
    } else if is_keyboard_click {
        Some(MenuInstantType::Click)
    } else if is_dismiss_close {
        Some(MenuInstantType::Dismiss)
    } else {
        None
    };

    // The single `store.update` (`:411`): `createPopupOpenState` derives the shared
    // fields; the menu fields ride the same update (the React-17 same-flush rule —
    // `instantType` must land with the mount that observes it).
    let popup_open_state = {
        let snapshot = store.get_snapshot();
        create_popup_open_state(
            &snapshot,
            next_open,
            event_details.trigger.as_ref(),
            prevent_unmount_requested.get(),
        )
    };
    store.update(move |state, _| {
        state.open = popup_open_state.open;
        state.prevent_unmounting_on_close = popup_open_state.prevent_unmounting_on_close;
        state.active_trigger_id = popup_open_state.active_trigger_id.clone();
        state.active_trigger_element = popup_open_state.active_trigger_element.clone();
        if let Some(extra) = state.payload.as_mut() {
            extra.instant_type = instant_type;
            extra.open_change_reason = Some(reason);
        }
        true
    });
}

// ---------------------------------------------------------------------------
// Context plumbing
// ---------------------------------------------------------------------------

thread_local! {
    static MENU_ROOT_CONTEXT: std::cell::RefCell<Option<MenuRootContextValue>> =
        const { std::cell::RefCell::new(None) };
}

/// Provides the menu root context for the subtree (`MenuRoot.tsx:636-641`). The
/// thread-local slot is the port's stand-in for the React provider (the
/// single-owner assumption the wasm single-thread reactive graph makes — the
/// dialog's `STORE_ACTIONS_SLOT` precedent).
pub fn provide_menu_root_context(context: MenuRootContextValue) {
    MENU_ROOT_CONTEXT.with(|slot| *slot.borrow_mut() = Some(context));
}

/// The required context read — panics with upstream's message when a part renders
/// outside a Root (the composite_root_context precedent; `MenuRootContext.ts`'
/// required accessor).
pub fn use_menu_root_context() -> MenuRootContextValue {
    MENU_ROOT_CONTEXT
        .with(|slot| slot.borrow().clone())
        .expect("Base UI: MenuRootContext is missing. Menu parts must be used within <Menu.Root>.")
}

/// The optional read (`MenuRootContext.ts:13-24` — the `optional` parameter
/// pattern the Root and Trigger use).
pub fn use_menu_root_context_optional() -> Option<MenuRootContextValue> {
    MENU_ROOT_CONTEXT.with(|slot| slot.borrow().clone())
}

/// Clears the provided context — the `MenuRootContext.Provider value={undefined}`
/// severing (`ContextMenuRoot.tsx:50-52`) that keeps a Context Menu mounted inside
/// another menu's subtree a standalone root (`MenuRoot.tsx:94-102`).
pub fn clear_menu_root_context() {
    MENU_ROOT_CONTEXT.with(|slot| *slot.borrow_mut() = None);
}

/// The store's open-state read as a reactive signal (the coalescing
/// `openProp ?? open` selector — `store.ts:142` via the shared table).
pub fn use_menu_open_signal(
    store: &MenuStore,
) -> RgRwSignal<bool, reactive_graph::owner::LocalStorage> {
    store.use_state(popup_store::selectors::open)
}

/// Convenience: reads the store's current open state non-reactively.
pub fn menu_store_is_open(store: &MenuStore) -> bool {
    popup_store::selectors::open(&store.get_snapshot())
}

/// Convenience: the store's active trigger element.
pub fn menu_store_active_trigger(store: &MenuStore) -> Option<web_sys::Element> {
    store.get_snapshot().active_trigger_element.clone()
}

// ---------------------------------------------------------------------------
// The item-facing reads (`MenuItem`'s store surface)
//
// Upstream's `MenuItem` reads four things off the store (`MenuItem.tsx:38-41`,
// `useMenuItemCommonProps.ts:54-55`): the root's `disabled`, the item's `highlighted`
// (`store.useState('isActive', index)`), the store-level `itemProps` bag, and the open
// state. The open state already has its signal ([`use_menu_open_signal`]); the rest are
// resolved here so the item part reads the store the way upstream does instead of
// re-deriving them.
//
// The store-level `itemProps` bag (`MenuStore.ts:34,86`) is deliberately NOT read: its
// only writer upstream is the constructor's `EMPTY_OBJECT` seed (`MenuStore.ts:228`) —
// nothing in the menu unit ever sets it — so `store.useState('itemProps')` is
// upstream's empty bag and merging it is a no-op.
// ---------------------------------------------------------------------------

/// The item metadata descriptor `useMenuItem` switches on (`useMenuItem.ts:121-126`).
///
/// `RegularItem` is `REGULAR_ITEM` (`:10-12`). `SubmenuTrigger` is the `'submenu-trigger'` arm
/// (`MenuSubmenuTrigger.tsx:120-130`), whose `setActive()` drives the sibling-open behaviour
/// (`:124-126`); it is spelled as a real variant now that `Menu.SubmenuTrigger` is ported, and its
/// behaviour lives in [`crate::menu::submenu_trigger::menu_submenu_trigger_set_active`] rather than
/// in an inert marker — the reason this arm was previously omitted outright (an inert variant would
/// have silently changed `useMenuItem`'s branch) no longer applies.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuItemMetadata {
    /// `REGULAR_ITEM` (`useMenuItem.ts:10-12`) — the metadata every other item type
    /// extends.
    RegularItem,
    /// `{ type: 'submenu-trigger', setActive() }` (`MenuSubmenuTrigger.tsx:120-130`) — the one arm
    /// whose `setActive` writes the PARENT menu's highlight index.
    SubmenuTrigger,
}

/// `store.useState('activeIndex')` (`MenuStore.ts:27,72`): the index the menu's
/// highlight currently sits on, `None` while nothing is highlighted.
pub fn use_menu_active_index_signal(
    store: &MenuStore,
) -> RgRwSignal<Option<usize>, reactive_graph::owner::LocalStorage> {
    store.use_state(|state| {
        state
            .payload
            .as_ref()
            .and_then(|payload| payload.active_index)
    })
}

/// `store.useState('disabled')` (`MenuStore.ts:20`, plus the menubar fold at `:53-56`).
/// With no menubar parent — the only parent shape this port has ([`MenuParent::None`]) —
/// upstream's fold reduces to the store's own field, which [`MenuRootProps`]'s `disabled`
/// prop seeds (`root.rs:87-90`).
pub fn use_menu_disabled_signal(
    store: &MenuStore,
) -> RgRwSignal<bool, reactive_graph::owner::LocalStorage> {
    store.use_state(|state| {
        state
            .payload
            .as_ref()
            .map(|payload| payload.disabled)
            .unwrap_or(false)
    })
}

/// `isActive(state, itemIndex)` (`MenuStore.ts:73`): `state.activeIndex === itemIndex`.
/// The item's `highlighted` read, and the same comparison [`crate::menu::item::Item`]'s
/// view makes against the composite list's index.
pub fn menu_item_is_active(store: &MenuStore, item_index: i32) -> bool {
    item_index >= 0
        && store
            .get_snapshot()
            .payload
            .as_ref()
            .and_then(|payload| payload.active_index)
            == Some(item_index as usize)
}

/// `onClick`'s close request (`useMenuItemCommonProps.ts:81-85`): with `closeOnClick` the
/// item asks for `store.setOpen(false, createChangeEventDetails('item-press'))`.
///
/// Upstream emits this on the floating tree's event bus and the POPUP listens
/// (`MenuPopup.tsx:69-79`) — the hop exists so that a nested submenu closes through the
/// popup that owns it. The port calls the unit's one mutation gate directly with the same
/// reason and the same end state, the deviation `trigger.rs:10-13` already documents for
/// its own `'close'` emission (`MenuTrigger.tsx:153`): the bus's only consumer in this
/// unit is the popup part, which is still a placeholder.
pub fn menu_item_on_click(store: &MenuStore, close_on_click: bool, event: Option<web_sys::Event>) {
    if close_on_click {
        menu_store_set_open(store, false, reasons_menu::ITEM_PRESS, event);
    }
}

/// The item's `tabIndex` (`useMenuItemCommonProps.ts:63`): `open && highlighted ? 0 : -1`.
pub fn menu_item_tab_index(open: bool, highlighted: bool) -> i32 {
    if open && highlighted { 0 } else { -1 }
}

/// The item's resolved state attributes (`MenuItemDataAttributes.ts:3,6` — the two
/// `data-*` hooks the item's own props carry). The crate's presence spelling
/// (`then_some("true")`, `accordion/mod.rs:398`) is used so a state attribute renders
/// exactly like its siblings' do.
pub fn menu_item_state_attributes(
    disabled: bool,
    highlighted: bool,
) -> Vec<(&'static str, &'static str)> {
    let mut attributes = Vec::new();
    if disabled {
        attributes.push(("data-disabled", "true"));
    }
    if highlighted {
        attributes.push(("data-highlighted", "true"));
    }
    attributes
}

// ---------------------------------------------------------------------------
// Part-facing compat surface
//
// The part files (`trigger.rs`, `popup.rs`, `item.rs`, …) predate this rewrite and
// call `use_menu_store()` + `store.open()`/`set_open()`. Those entry points now
// resolve the Root's store from the provided context (the upstream contract) —
// the previous implementation minted a fresh store per call, which made the Root's
// provision inert.
// ---------------------------------------------------------------------------

/// The store handle the parts consume. Resolves the Root's store from the provided
/// context — a fresh store is never minted at a part (`MenuTrigger.tsx:66-73` reads
/// the root context; throwing without one is behavior.md's "Public API surface").
pub fn use_menu_store() -> MenuRootContextValue {
    use_menu_root_context()
}

/// The details constructor for the parts' open/close requests —
/// `createChangeEventDetails(REASONS.…)` (`MenuRoot.tsx:24-28`), with the native
/// event and no trigger (the gate backfills the active trigger on close).
pub fn menu_change_event_details(
    reason: &str,
    event: Option<web_sys::Event>,
) -> MenuChangeEventDetails {
    BaseUIChangeEventDetails::new(
        reason,
        event.unwrap_or_else(|| web_sys::Event::new("").unwrap()),
        None,
        String::new(),
    )
}

/// Routes a part's open/close request through the one mutation gate
/// (`store.setOpen`, `MenuStore.ts:165-167`).
pub fn menu_store_set_open(
    store: &MenuStore,
    next_open: bool,
    reason: &str,
    event: Option<web_sys::Event>,
) {
    store_set_open(store, next_open, menu_change_event_details(reason, event));
}

/// The pre-rewrite parts' import name — an alias over the shared store handle while
/// the parts are migrated to the gate (the alias is the migration seam, not a second
/// type).
pub type MenuStoreContext = MenuRootContextValue;

/// Menu orientation — upstream's Root has no orientation prop (the menubar owns
/// it); the enum lives here for the menubar-family callers pending their Phase B
/// port (the old facade also carried it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuOrientation {
    /// Horizontal (menubar).
    Horizontal,
    /// Vertical (root menu).
    Vertical,
}

impl Default for MenuOrientation {
    fn default() -> Self {
        MenuOrientation::Vertical
    }
}

impl MenuRootContextValue {
    /// The store handle.
    pub fn store(&self) -> &MenuStore {
        &self.store
    }

    /// The reactive open-state read as a leptos-native signal — the store's
    /// coalescing `openProp ?? open` selector bridged across the reactive-graph
    /// storage boundary (the parts' view code reads leptos signals).
    pub fn open(&self) -> leptos::prelude::RwSignal<bool> {
        let source = use_menu_open_signal(&self.store);
        let bridged = leptos::prelude::RwSignal::new(source.get_untracked());
        leptos::prelude::Effect::new(move || {
            bridged.set(source.get());
        });
        bridged
    }

    /// Routes an open/close request through the one mutation gate with the
    /// trigger-press reason (the `store.setOpen` emission).
    pub fn set_open(&self, next_open: bool) {
        menu_store_set_open(
            &self.store,
            next_open,
            crate::menu::store::reasons::TRIGGER_PRESS,
            None,
        );
    }

    /// Claims the active trigger element (the `registerTrigger` data-forwarding
    /// write, `useTriggerDataForwarding`).
    pub fn set_active_trigger(&self, element: Option<web_sys::Element>) {
        self.store
            .set_field(|state| &mut state.active_trigger_element, element);
    }

    /// The store's active trigger element.
    pub fn active_trigger(&self) -> Option<web_sys::Element> {
        menu_store_active_trigger(&self.store)
    }
}

impl std::fmt::Debug for MenuParent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuParent::None => write!(f, "MenuParent::None"),
            MenuParent::Menu { .. } => write!(f, "MenuParent::Menu"),
            MenuParent::Menubar => write!(f, "MenuParent::Menubar"),
            MenuParent::ContextMenu => write!(f, "MenuParent::ContextMenu"),
        }
    }
}

/// The Root's constructor with the user's `onOpenChange` on the context slot (the
/// dialog port's construction-time write, `dialog/mod.rs:49-63`).
pub fn create_menu_store_with_on_open_change(
    on_open_change: Option<Rc<dyn Fn(bool, &MenuChangeEventDetails)>>,
) -> MenuStore {
    let trigger_elements =
        leptos_ui_internals::floating_ui::popup_trigger_map::PopupTriggerMap::new();
    Rc::new(leptos_ui_utils::react_store::ReactStore::with_context(
        create_initial_menu_store_state(),
        leptos_ui_internals::floating_ui::popup_store::PopupStoreContext {
            trigger_elements,
            popup_ref: Rc::new(Cell::new(None)),
            on_open_change,
            on_open_change_complete: None,
        },
    ))
}

// ---------------------------------------------------------------------------
// The submenu-trigger-facing reads (`MenuSubmenuTrigger`'s store surface)
//
// `MenuSubmenuTrigger` reads more of the store than the plain items do
// (`MenuSubmenuTrigger.tsx:60-63,100-101,117-118,144,177-178`): the last open-change
// reason and the open method (its `openedByKeyboard` test, `:181-182`), the popup id
// for `aria-controls` (`:63,200`), and — through the SubmenuRoot bridge — the PARENT
// menu's `disabled` and `highlightItemOnHover`. The parent reads are the shared
// `use_menu_disabled_signal`/`menu_submenu_trigger_highlight_item_on_hover` pair applied
// to the other store; the rest are here.
// ---------------------------------------------------------------------------

/// `store.useState('lastOpenChangeReason')` (`MenuStore.ts:30` as read at
/// `MenuSubmenuTrigger.tsx:178`): the reason of the last accepted open-state change.
pub fn use_menu_last_open_change_reason_signal(
    store: &MenuStore,
) -> RgRwSignal<Option<String>, reactive_graph::owner::LocalStorage> {
    store.use_state(|state| {
        state
            .payload
            .as_ref()
            .and_then(|payload| payload.open_change_reason.clone())
    })
}

/// `store.useState('openMethod')` (`MenuStore.ts:22,60`), read at
/// `MenuSubmenuTrigger.tsx:177` to tell a keyboard open from a pointer one.
pub fn use_menu_open_method_signal(
    store: &MenuStore,
) -> RgRwSignal<Option<String>, reactive_graph::owner::LocalStorage> {
    store.use_state(|state| {
        state
            .payload
            .as_ref()
            .and_then(|payload| payload.open_method.clone())
    })
}

/// `store.select('activeTriggerId')` (`popupStoreSelectors.activeTriggerId`) — the trigger
/// the floating root currently considers active, read non-reactively for the
/// `MenuSubmenuTrigger.tsx:71` "nothing owns the trigger" test.
pub fn use_menu_store_active_trigger_id(store: &MenuStore) -> Option<String> {
    popup_store::selectors::active_trigger_id(&store.get_snapshot())
}

/// `popupStoreSelectors.triggerPopupId(state, triggerId)`
/// (`packages/react/src/utils/popups/store.ts:199-200`): the popup id for the trigger that
/// currently owns the open popup, or `None`.
///
/// The predicate is `triggerOwnsOpenPopupOrIsOnlyTrigger` (`store.ts:155-166`) inlined here,
/// because the shared module keeps it private: ownership
/// (`open && activeTriggerId === triggerId`, `:149-153`) or the "only trigger" fallback
/// (`open` with no active trigger and exactly one registered trigger). `None` is what lets
/// [`crate::menu::submenu_trigger::menu_submenu_trigger_aria_controls`] fall back to the
/// trigger's own id — upstream's `useState('triggerPopupId', thisTriggerId)` default.
pub fn menu_store_trigger_popup_id(
    state: &PopupStoreState<MenuExtraState>,
    trigger_id: Option<&str>,
) -> Option<String> {
    let Some(trigger_id) = trigger_id else {
        return None;
    };

    let owns = popup_store::selectors::open(state)
        && popup_store::selectors::active_trigger_id(state).as_deref() == Some(trigger_id);

    let only_trigger = popup_store::selectors::open(state)
        && popup_store::selectors::active_trigger_id(state).is_none()
        && state.trigger_count == 1;

    if owns || only_trigger {
        popup_store::selectors::popup_id(state)
    } else {
        None
    }
}

/// The active-trigger claim a trigger makes when it attaches while the menu is already open
/// and nothing owns the trigger (`MenuSubmenuTrigger.tsx:71-77`):
/// `store.update({ activeTriggerId, activeTriggerElement, closeDelay })`.
pub fn claim_menu_active_trigger(
    store: &MenuStore,
    trigger_id: Option<String>,
    element: web_sys::Element,
    close_delay: u32,
) {
    store.set_field(|state| &mut state.active_trigger_id, trigger_id);
    store.set_field(
        |state| &mut state.active_trigger_element,
        Some(element),
    );
    store.set_field(
        |state| &mut state.payload.get_or_insert_with(Default::default).close_delay,
        close_delay,
    );
}

/// `parentMenuStore.select('highlightItemOnHover')` (`MenuSubmenuTrigger.tsx:124`) — the gate on
/// whether a submenu trigger's `setActive()` may move its parent's highlight.
pub fn menu_submenu_trigger_highlight_item_on_hover(store: &MenuStore) -> bool {
    store
        .get_snapshot()
        .payload
        .as_ref()
        .map(|payload| payload.highlight_item_on_hover)
        .unwrap_or(true)
}

/// `parentMenuStore.set('activeIndex', …)` (`MenuSubmenuTrigger.tsx:125,204`) — the PARENT menu's
/// highlight slot, which is the one piece of the parent's state this part writes.
pub fn menu_submenu_trigger_set_active_index(store: &MenuStore, index: Option<usize>) {
    store.set_field(
        |state| &mut state.payload.get_or_insert_with(Default::default).active_index,
        index,
    )
}

/// `MenuStore.ts:57-59` — the `modal` SELECTOR:
/// `(parent.type === undefined || parent.type === 'context-menu') && (state.modal ?? true)`.
///
/// A nested menu is never modal, whatever its own field holds — which is why upstream omits `modal`
/// from `Menu.SubmenuRoot`'s props entirely (`MenuSubmenuRoot.tsx:27-36`) and warns when it is passed
/// to a nested menu (`MenuRoot.tsx:178-180`). The parent term only becomes observable once a real
/// submenu parent exists, which is what the SubmenuRoot bridge now supplies.
pub fn menu_modal(parent: &MenuParent, modal: bool) -> bool {
    matches!(parent, MenuParent::None | MenuParent::ContextMenu) && modal
}

/// The parent-folded `modal` read (`MenuStore.ts:57-59`), for consumers that need the selector's
/// value rather than the raw field — the positioner's `popupModal` input
/// (`MenuPositioner.tsx:267-268`).
pub fn use_menu_modal_signal(
    store: &MenuStore,
) -> RgRwSignal<bool, reactive_graph::owner::LocalStorage> {
    store.use_state(|state| {
        let payload = state.payload.as_ref();
        let parent = payload
            .map(|payload| payload.parent.clone())
            .unwrap_or(MenuParent::None);
        // `state.modal ?? true` — the port's field is a plain `bool` already carrying that default.
        let modal = payload.map(|payload| payload.modal).unwrap_or(true);
        menu_modal(&parent, modal)
    })
}
