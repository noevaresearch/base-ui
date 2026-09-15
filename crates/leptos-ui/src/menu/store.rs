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
    /// `closeDelay` (`MenuStore.ts:35`).
    pub close_delay: u32,
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
            close_delay: 0,
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
