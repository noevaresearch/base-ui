//! Menu submenu trigger — `Menu.SubmenuTrigger`, the port of
//! `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.tsx` (273 lines) plus the one
//! module it owns, `MenuSubmenuTriggerDataAttributes.ts` (`:3,10,14`).
//!
//! WHAT THIS REPLACES. The previous file (spelled `submenu-trigger.rs`) was a facade, not a port,
//! and it had never been type-checked: a hyphen is not a legal Rust identifier, so `menu/mod.rs`
//! could never declare the module. Its 233 lines (1) invented the prop surface — `hover_open_delay`
//! and `hover_close_delay` (both defaulting to `200`) where upstream has `delay` (default **`100`**)
//! and `closeDelay` (default **`0`**), plus a `render: Option<fn() -> HtmlElement>` that is not a
//! render prop at all; (2) invented six helpers (`use_menu_submenu_trigger_props`, `_disabled`,
//! `_open_on_hover`, `_hover_open_delay`, `_hover_close_delay`, `_label`) returning constants, four
//! of them carrying the fabricated-body marker; (3) wrote the open state DIRECTLY
//! (`open.set(true)` / `open.set(false)`), bypassing the unit's one mutation gate that
//! `specs/library/menu/implementation.md` → "One mutation gate" mandates; (4) rendered a hardcoded
//! `<div class="menu-submenu-trigger">` shell with a hardcoded `▼` child — upstream renders no
//! fallback content and no class, and the hardcoded class is what behavior.md → "Uniform DOM shell"
//! forbids; and (5) invented ARIA: an `aria-label` defaulting to the literal `"Submenu"` (upstream
//! has no `aria-label` on this part at all — `label` is the keyboard-navigation override,
//! `useCompositeListItem({ guess: true, label })` at `:54`), and `aria-haspopup=true` as a BOOLEAN
//! where the DOM contract is the token `"menu"`.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from:
//! - the requirement (`:49-52`): the trigger must be inside `Menu.SubmenuRoot`, and throws
//!   upstream's own message otherwise — and the `parentMenu` it reads from that context (`:99`) is
//!   the PARENT menu's store, not this submenu's.
//! - the props (`:35-47`) with upstream's documented defaults: `id`, `label`, `nativeButton`
//!   (`false`), `openOnHover` (**`true`**), `delay` (**`100`**), `closeDelay` (**`0`**),
//!   `disabled` (`false`), `className`/`style`, and the `...elementProps` rest (`:46`).
//! - the composite-list-item registration (`:54`: `useCompositeListItem({ guess: true, label })`) —
//!   the trigger's index in the submenu's own list, which `highlighted` compares against (`:118`).
//! - `id = useBaseUiId(idProp)` (`:59`).
//! - the store reads (`:60-63,100-101,117-118,144,177-178`): `open`, this submenu's `disabled`, the
//!   PARENT menu's `disabled`, the parent's `highlightItemOnHover`, the popup id, `openMethod` and
//!   `lastOpenChangeReason`.
//! - the trigger registration (`:65,68-78,92-95`): `useTriggerRegistration(thisTriggerId, store)`
//!   plus the active-trigger claim that runs when the element attaches while the menu is already
//!   open and nothing owns the trigger.
//! - `store.useSyncedValue('closeDelay', closeDelay)` (`:97`).
//! - the disabled fold (`:102`): `disabledProp || rootDisabled || parentDisabled` — the PARENT term
//!   is why this part reads through the SubmenuRoot bridge at all.
//! - the item metadata (`:120-130`): `type: 'submenu-trigger'` whose `setActive()` writes the
//!   PARENT's `activeIndex` **only while the parent's `highlightItemOnHover` is set** — the
//!   sibling-open behaviour that distinguishes a submenu trigger from a plain item.
//! - the element contract (`:185-212`): `'div'` (`:185`), `role="menuitem"`, the
//!   `tabIndex = open || highlighted ? 0 : -1` (`:201` — note `||`, deliberately unlike the plain
//!   item's `&&` at `useMenuItemCommonProps.ts:63`), `aria-controls: popupId` (`:200`),
//!   `data-popup-open`/`data-disabled`/`data-highlighted` through `triggerOpenStateMapping`
//!   (`:187`) over `state = { disabled, highlighted, open }` (`:175`), and the `onBlur` that clears
//!   the PARENT's `activeIndex` when the trigger was highlighted (`:202-206`).
//! - the VoiceOver `aria-expanded` omission (`:23,177-183,198`): when the submenu is open, was
//!   opened by the keyboard, and the platform is VoiceOver, the attribute is REMOVED rather than set
//!   to `"true"` — upstream's fix for VoiceOver announcing the state change instead of the submenu
//!   item that focus moves to.
//!
//! DEFERRED, each with the reason it is not this checkpoint's, and none of them silently dropped:
//! - `useButton`'s layer and the `getItemProps` merge that spreads it (`:132-142,209`) — the same
//!   deferral [`crate::menu::item::Item`]'s module docs record for every item part. One consequence
//!   is specific to THIS part and is stated rather than hidden: upstream's `getItemProps` is what
//!   invokes `itemMetadata.setActive()` on `mouseEnter` (`useMenuItem.ts:53-58`), so
//!   [`menu_submenu_trigger_set_active`] is implemented and tested but has no in-tree caller until
//!   that merge lands. It is a real function with real semantics, not an inert helper.
//! - `useHoverReferenceInteraction` and `useClick` (`:146-170`) — the hover/click interaction layer,
//!   which [`crate::menu::trigger::Trigger`] also defers with the same reason (its docs, `:33-34`:
//!   "the delay timers are the hover checkpoint's"). `delay`/`closeDelay` are therefore synced into
//!   the store rather than silently ignored: the store's `closeDelay` slot is what the interaction
//!   layer reads when it lands.
//! - `onMouseMove`'s `'itemhover'` emission via the positioner's `nodeId` (`:141`,
//!   `useMenuItemCommonProps.ts:69-80`) — the positioner's own `itemhover`/`menuopenchange`
//!   coordination checkpoint named in this item's remaining-work list.
//! - `render`, the forwarded refs and the dev-only disabled-element warning (`:36,104-115,211`) —
//!   the crate-wide `library: the view paths drop render's element form`, plus a
//!   `process.env.NODE_ENV`-only diagnostic this crate has no equivalent of.
//! - The rest of `rootTriggerProps` (`:172-173,191`): upstream reads the ROOT's published
//!   trigger-props bag. `aria-haspopup` is emitted from the documented token
//!   ([`MENU_SUBMENU_TRIGGER_HASPOPUP`], pinned by `MenuSubmenuTrigger.voiceOver.test.tsx:70`), and
//!   the bag's full publication is the Root-publishes-`popupProps` change this item names.

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use leptos_ui_internals::common_trigger_data_attributes::POPUP_OPEN;
use leptos_ui_internals::popup_state_mapping::trigger_open_state_mapping;
use leptos_ui_internals::popup_store_utils::use_trigger_registration;
use leptos_ui_internals::state_attributes::get_state_attributes_props;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_composite_list_item::{
    UseCompositeListItemParams, use_composite_list_item,
};
// The reactive plumbing spelling the crate's parts use (`popover/parts.rs:9-11`): the
// `reactive_graph` aliases plus the traits the signal reads need.
use reactive_graph::computed::Memo as RgMemo;
use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked};

use crate::menu::store::{
    MenuItemMetadata, MenuStore, claim_menu_active_trigger, menu_store_is_open,
    menu_store_trigger_popup_id, menu_submenu_trigger_highlight_item_on_hover,
    menu_submenu_trigger_set_active_index, use_menu_active_index_signal, use_menu_disabled_signal,
    use_menu_last_open_change_reason_signal, use_menu_open_method_signal, use_menu_open_signal,
    use_menu_store, use_menu_store_active_trigger_id,
};
use crate::menu::submenu_root::use_menu_submenu_root_context;

/// Upstream renders `'div'` (`MenuSubmenuTrigger.tsx:185`, `useRenderElement('div', …)`), which the
/// mined suite pins as a `div` (`MenuSubmenuTrigger.test.tsx:32`).
pub const MENU_SUBMENU_TRIGGER_TAG: &str = "div";

/// `role: 'menuitem'` (`MenuSubmenuTrigger.tsx:191` through `useMenuItemCommonProps.ts:62`) — the
/// trigger is an item of its PARENT menu (`MenuSubmenuTrigger.test.tsx:238`'s
/// `getByRole('menuitem')`).
pub const MENU_SUBMENU_TRIGGER_ROLE: &str = "menuitem";

/// `aria-haspopup`'s value — the token `"menu"`, not a boolean
/// (`MenuSubmenuTrigger.voiceOver.test.tsx:70` asserts the trigger "always has"
/// `aria-haspopup="menu"`).
pub const MENU_SUBMENU_TRIGGER_HASPOPUP: &str = "menu";

/// `MenuSubmenuTriggerDataAttributes.highlighted` (`:10`).
pub const MENU_SUBMENU_TRIGGER_HIGHLIGHTED_ATTRIBUTE: &str = "data-highlighted";

/// `MenuSubmenuTriggerDataAttributes.disabled` (`:14`).
pub const MENU_SUBMENU_TRIGGER_DISABLED_ATTRIBUTE: &str = "data-disabled";

/// `MenuSubmenuTriggerDataAttributes.popupOpen` (`:6`) — upstream's own re-export of
/// `CommonTriggerDataAttributes.popupOpen`, which the shared registry already carries.
pub const MENU_SUBMENU_TRIGGER_POPUP_OPEN_ATTRIBUTE: &str = POPUP_OPEN;

/// The `openOnHover` default (`MenuSubmenuTrigger.tsx:42`): **`true`**.
pub const MENU_SUBMENU_TRIGGER_OPEN_ON_HOVER_DEFAULT: bool = true;

/// The `delay` default (`:43`): **`100`** ms.
pub const MENU_SUBMENU_TRIGGER_DELAY_DEFAULT: u32 = 100;

/// The `closeDelay` default (`:44`): **`0`** ms.
pub const MENU_SUBMENU_TRIGGER_CLOSE_DELAY_DEFAULT: u32 = 0;

/// `throw new Error('Base UI: <Menu.SubmenuTrigger> must be placed in <Menu.SubmenuRoot>.')`
/// (`:51`) — the message a trigger rendered outside its SubmenuRoot carries.
pub const MENU_SUBMENU_TRIGGER_OUTSIDE_ROOT_MESSAGE: &str =
    "Base UI: <Menu.SubmenuTrigger> must be placed in <Menu.SubmenuRoot>.";

/// The `openMethod` value that marks a keyboard activation (`MenuStore.ts:22`, read at
/// `MenuSubmenuTrigger.tsx:182`).
pub const MENU_SUBMENU_TRIGGER_OPEN_METHOD_KEYBOARD: &str = "keyboard";

/// The trigger's state object (`MenuSubmenuTrigger.tsx:175`, `MenuSubmenuTriggerState` at
/// `:217-230`) — exactly the `{ disabled, highlighted, open }` map upstream hands
/// `useRenderElement`, so the shared engine's own mapping produces the attribute set.
///
/// The ORDER is upstream's literal property order, and it is load-bearing for the engine's default
/// handling: `triggerOpenStateMapping` claims `open` only (`popupStateMapping.ts:33-41`), so
/// `disabled` and `highlighted` fall through to the `data-<key>` default and emit
/// [`MENU_SUBMENU_TRIGGER_DISABLED_ATTRIBUTE`] / [`MENU_SUBMENU_TRIGGER_HIGHLIGHTED_ATTRIBUTE`] —
/// which is where those two attributes actually come from, rather than from a hand-rolled list.
pub fn menu_submenu_trigger_state_map(
    disabled: bool,
    highlighted: bool,
    open: bool,
) -> serde_json::Map<String, serde_json::Value> {
    let mut state = serde_json::Map::new();
    state.insert("disabled".to_owned(), serde_json::Value::Bool(disabled));
    state.insert("highlighted".to_owned(), serde_json::Value::Bool(highlighted));
    state.insert("open".to_owned(), serde_json::Value::Bool(open));
    state
}

/// The trigger's resolved `data-*` set (`MenuSubmenuTrigger.tsx:187` over `:175`) — the ported
/// Phase A engine rather than a hand-rolled list, the [`crate::menu::utils::menu_item_attributes`]
/// precedent.
pub fn menu_submenu_trigger_attributes(
    disabled: bool,
    highlighted: bool,
    open: bool,
) -> Vec<(String, String)> {
    get_state_attributes_props(
        &menu_submenu_trigger_state_map(disabled, highlighted, open),
        Some(&trigger_open_state_mapping),
    )
    .into_iter()
    .collect()
}

/// The disabled fold (`MenuSubmenuTrigger.tsx:102`):
/// `disabledProp || rootDisabled || parentDisabled`. The parent term is what disables a trigger
/// inside a disabled parent menu even though its own submenu is not disabled.
pub fn menu_submenu_trigger_disabled(
    disabled_prop: bool,
    root_disabled: bool,
    parent_disabled: bool,
) -> bool {
    disabled_prop || root_disabled || parent_disabled
}

/// The trigger's `tabIndex` (`MenuSubmenuTrigger.tsx:201`): `open || highlighted ? 0 : -1`.
///
/// Deliberately NOT [`crate::menu::store::menu_item_tab_index`], which is the plain item's
/// `open && highlighted ? 0 : -1` (`useMenuItemCommonProps.ts:63`). An open submenu's trigger keeps
/// focus while its own list holds the highlight, so the two disagree exactly when the submenu is
/// open but the trigger is not the highlighted item — which is why the trigger gets its own function
/// instead of reusing the item's.
pub fn menu_submenu_trigger_tab_index(open: bool, highlighted: bool) -> i32 {
    if open || highlighted { 0 } else { -1 }
}

/// `aria-controls: popupId` (`MenuSubmenuTrigger.tsx:200`).
///
/// `popupId` is `store.useState('triggerPopupId', thisTriggerId)` (`:63`): the popup's id while this
/// trigger owns the open popup, otherwise the selector's default — the trigger's OWN id
/// (`popups/store.ts:199-200`; a `triggerPopupId` that resolves to `undefined` lets the `useState`
/// default argument stand, and the port's [`menu_store_trigger_popup_id`] returns that `None`).
pub fn menu_submenu_trigger_aria_controls(
    trigger_popup_id: Option<String>,
    trigger_id: &str,
) -> String {
    trigger_popup_id.unwrap_or_else(|| trigger_id.to_owned())
}

/// `openedByKeyboard` (`MenuSubmenuTrigger.tsx:181-182`):
/// `lastOpenChangeReason === REASONS.listNavigation || openMethod === 'keyboard'`.
///
/// Arrow keys open a submenu through list navigation without dispatching a click, so `openMethod`
/// stays null on that path while Enter and Space do dispatch one and report `keyboard` — which is
/// why upstream tests both.
pub fn menu_submenu_trigger_opened_by_keyboard(
    last_open_change_reason: Option<&str>,
    open_method: Option<&str>,
) -> bool {
    last_open_change_reason == Some(leptos_ui_internals::floating_ui::reasons::LIST_NAVIGATION)
        || open_method == Some(MENU_SUBMENU_TRIGGER_OPEN_METHOD_KEYBOARD)
}

/// `shouldOmitExpanded` (`MenuSubmenuTrigger.tsx:183`):
/// `open && openedByKeyboard && platform.screenReader.voiceOver` — the condition under which
/// `aria-expanded` is DROPPED (`:23,196-198`) rather than set, so VoiceOver announces the item focus
/// moves to instead of the trigger's state change.
///
/// Takes `voice_over` as a parameter rather than reading the platform itself, so the predicate is
/// host-testable: `platform()`'s real value on a non-Apple host is `false`, which would make the
/// positive case unreachable in a host test.
pub fn menu_submenu_trigger_should_omit_expanded(
    open: bool,
    opened_by_keyboard: bool,
    voice_over: bool,
) -> bool {
    open && opened_by_keyboard && voice_over
}

/// The resolved `aria-expanded` (`MenuSubmenuTrigger.tsx:198,201` over `:23`): `Some(open)`, except
/// that [`menu_submenu_trigger_should_omit_expanded`] makes it `None` — dropped entirely — which is
/// a different thing from `Some(false)` and is what the VoiceOver tests assert.
pub fn menu_submenu_trigger_aria_expanded(
    open: bool,
    opened_by_keyboard: bool,
    voice_over: bool,
) -> Option<bool> {
    if menu_submenu_trigger_should_omit_expanded(open, opened_by_keyboard, voice_over) {
        return None;
    }
    Some(open)
}

/// `itemMetadata.setActive()` (`MenuSubmenuTrigger.tsx:122-127`): the trigger claims the PARENT
/// menu's highlight index — but only while the parent's `highlightItemOnHover` is set
/// (`:124-126`), which is why a hover-highlighting-disabled parent leaves the highlight where it was.
///
/// Returns whether a write happened, so the behaviour is observable in a host test without a DOM
/// (this box refuses a browser — `ralph/generated/env-health.json` → `browser`). Its upstream caller
/// is the deferred `getItemProps` `mouseEnter` merge (`useMenuItem.ts:53-58`); see the module docs.
pub fn menu_submenu_trigger_set_active(parent_store: &MenuStore, item_index: i32) -> bool {
    if !menu_submenu_trigger_highlight_item_on_hover(parent_store) || item_index < 0 {
        return false;
    }

    menu_submenu_trigger_set_active_index(parent_store, Some(item_index as usize));
    true
}

/// `onBlur` (`MenuSubmenuTrigger.tsx:202-206`): when the trigger that lost focus WAS the highlighted
/// item, the PARENT's `activeIndex` is cleared so the parent stops reporting a highlight.
pub fn menu_submenu_trigger_on_blur(parent_store: &MenuStore, highlighted: bool) {
    if highlighted {
        menu_submenu_trigger_set_active_index(parent_store, None);
    }
}

/// The resolved description of the trigger's root element — the members
/// `useRenderElement('div', componentProps, …)` (`MenuSubmenuTrigger.tsx:185-212`) puts on the
/// element, in the crate's attribute vocabulary. Host-testable: the resolution carries no DOM
/// access, so the trigger's contract can be asserted without a browser.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuSubmenuTriggerResolved {
    /// The resolved tag (`:185`).
    pub tag: &'static str,
    /// The trigger's id (`:40,59`).
    pub id: String,
    /// `role` (`:191`).
    pub role: &'static str,
    /// `tabIndex` (`:201`) — the `open || highlighted` variant.
    pub tab_index: i32,
    /// `aria-controls` (`:200`).
    pub aria_controls: String,
    /// `aria-expanded` (`:198`) — `None` when VoiceOver is meant to drop it.
    pub aria_expanded: Option<bool>,
    /// `aria-haspopup` (`:191` over the root's trigger-props bag) — always the `"menu"` token.
    pub aria_haspopup: &'static str,
    /// The folded disabled state (`:102`).
    pub disabled: bool,
    /// `highlighted` (`:118`).
    pub highlighted: bool,
    /// `open` (`:60`).
    pub open: bool,
    /// The resolved `data-*` set (`:187` over `:175`).
    pub attributes: Vec<(String, String)>,
}

/// Resolves the trigger's element description (`MenuSubmenuTrigger.tsx:185-212`). Pure apart from the
/// ids and the store facts the caller resolved, so the trigger's observable contract is assertable
/// on the host target.
#[allow(clippy::too_many_arguments)]
pub fn resolve_menu_submenu_trigger(
    id: String,
    trigger_popup_id: Option<String>,
    open: bool,
    highlighted: bool,
    disabled: bool,
    opened_by_keyboard: bool,
    voice_over: bool,
) -> MenuSubmenuTriggerResolved {
    MenuSubmenuTriggerResolved {
        tag: MENU_SUBMENU_TRIGGER_TAG,
        role: MENU_SUBMENU_TRIGGER_ROLE,
        tab_index: menu_submenu_trigger_tab_index(open, highlighted),
        aria_controls: menu_submenu_trigger_aria_controls(trigger_popup_id, &id),
        aria_expanded: menu_submenu_trigger_aria_expanded(open, opened_by_keyboard, voice_over),
        aria_haspopup: MENU_SUBMENU_TRIGGER_HASPOPUP,
        disabled,
        highlighted,
        open,
        attributes: menu_submenu_trigger_attributes(disabled, highlighted, open),
        id,
    }
}

/// `Menu.SubmenuTrigger` — upstream's `MenuSubmenuTrigger` (`MenuSubmenuTrigger.tsx:31-215`).
///
/// Renders a `<div role="menuitem">` that opens its submenu. It reads the PARENT menu's store through
/// the `Menu.SubmenuRoot` bridge (`:49-52,99`) and this submenu's own store from the surrounding
/// `Menu.Root` (`:57`). See the module docs for what is ported and what is deferred.
#[component]
pub fn SubmenuTrigger(
    /// `id` (`MenuSubmenuTrigger.tsx:40`) — defaults to the generated `base-ui-<n>` id the hook
    /// produces (`useBaseUiId`, `:59`).
    #[prop(optional)]
    id: Option<String>,
    /// `label` (`:39`) — overrides the text used for keyboard text navigation. NOT an `aria-label`:
    /// the facade this file replaces invented that, and upstream has no such prop here.
    #[prop(optional)]
    label: Option<String>,
    /// `nativeButton` (`:41`, default `false`) — the root is a `div`, so the non-native path.
    #[prop(default = false)]
    native_button: bool,
    /// `openOnHover` (`:42`, default **`true`**).
    #[prop(default = MENU_SUBMENU_TRIGGER_OPEN_ON_HOVER_DEFAULT)]
    open_on_hover: bool,
    /// `delay` (`:43`, default **`100`**) — how long before the submenu may open on hover; requires
    /// `openOnHover`.
    #[prop(default = MENU_SUBMENU_TRIGGER_DELAY_DEFAULT)]
    delay: u32,
    /// `closeDelay` (`:44`, default **`0`**) — how long before a hover-opened submenu closes.
    #[prop(default = MENU_SUBMENU_TRIGGER_CLOSE_DELAY_DEFAULT)]
    close_delay: u32,
    /// `disabled` (`:45`, default `false`) — ORed with this submenu's and the PARENT menu's own
    /// `disabled` (`:102`).
    #[prop(default = false)]
    disabled: bool,
    /// `className` (`:37` via `BaseUIComponentProps`).
    #[prop(optional, into)]
    class: Option<String>,
    /// `style` (`:38`) — ordered declarations.
    #[prop(default = Vec::new())]
    style: Vec<(String, String)>,
    /// The `...elementProps` rest (`:46`) — the caller's own DOM props, applied after the part's, in
    /// upstream's bag order (`:208`).
    #[prop(default = Vec::new())]
    element_attributes: Vec<(String, String)>,
    /// The trigger's content.
    children: Children,
) -> impl IntoView {
    // `const submenuRootContext = useMenuSubmenuRootContext(); if (!submenuRootContext?.parentMenu)
    // throw` (`MenuSubmenuTrigger.tsx:49-52`) — the required bridge read, with upstream's message.
    let submenu_root_context =
        use_menu_submenu_root_context().expect(MENU_SUBMENU_TRIGGER_OUTSIDE_ROOT_MESSAGE);
    let parent_menu_store: MenuStore = submenu_root_context.parent_menu;

    // `const { store } = useMenuRootContext();` (`:57`) — this SUBMENU's own store.
    let context = use_menu_store();
    let store: MenuStore = context.store;

    // `useBaseUiId(idProp)` (`:59`).
    let id_signal = use_base_ui_id(RgRwSignal::new_local(id));

    // `useCompositeListItem({ guess: true, label })` (`:54`) — the trigger's index in the submenu's
    // own composite list, carrying upstream's `'submenu-trigger'` metadata descriptor (`:120-130`),
    // which is what distinguishes this item from a plain one in the shared list.
    let list_item = use_composite_list_item::<
        MenuItemMetadata,
        RgRwSignal<Option<i32>, reactive_graph::owner::LocalStorage>,
    >(UseCompositeListItemParams {
        guess: true,
        index: RgRwSignal::new_local(None::<i32>),
        label: Some(label),
        metadata: Some(MenuItemMetadata::SubmenuTrigger),
        text_ref: None,
    });

    // The store reads (`:60,100-101,117-118,177-178`).
    let open = use_menu_open_signal(&store);
    let root_disabled = use_menu_disabled_signal(&store);
    let parent_disabled = use_menu_disabled_signal(&parent_menu_store);
    let active_index = use_menu_active_index_signal(&store);
    let last_open_change_reason = use_menu_last_open_change_reason_signal(&store);
    let open_method = use_menu_open_method_signal(&store);

    // `store.useSyncedValue('closeDelay', closeDelay)` (`:97`) — the store's `closeDelay` slot is
    // what the deferred hover interaction layer reads, so the prop is synced rather than ignored.
    let close_delay_signal = RgRwSignal::new_local(close_delay);
    store.use_synced_value(
        |state| &mut state.payload.get_or_insert_with(Default::default).close_delay,
        close_delay_signal,
    );

    // `const disabled = disabledProp || rootDisabled || parentDisabled;` (`:102`).
    let disabled_signal = RgMemo::new(move |_| {
        menu_submenu_trigger_disabled(disabled, root_disabled.get(), parent_disabled.get())
    });

    // `const highlighted = parentMenuStore.useState('isActive', listItem.index);` (`:118`).
    let highlighted = RgMemo::new(move |_| {
        let index = list_item.index.get();
        index >= 0 && active_index.get() == Some(index as usize)
    });

    // `openedByKeyboard` (`:181-182`), resolved from the store's own slots.
    let opened_by_keyboard = RgMemo::new(move |_| {
        menu_submenu_trigger_opened_by_keyboard(
            last_open_change_reason.get().as_deref(),
            open_method.get().as_deref(),
        )
    });

    // `popupId = store.useState('triggerPopupId', thisTriggerId)` (`:63`), whose `useState` default
    // is the trigger's own id — [`menu_submenu_trigger_aria_controls`] applies that fallback.
    let trigger_id_for_popup = id_signal.get_untracked();
    let store_for_popup_id = store.clone();
    let popup_id = store.use_state(move |state| {
        menu_store_trigger_popup_id(state, Some(trigger_id_for_popup.as_str()))
    });

    // `const registerTrigger = useStableCallback((element) => {…})` (`:65,68-78`): the shared trigger
    // registration plus the active-trigger claim for an already-open menu with no owner.
    let register_trigger = use_trigger_registration(Some(id_signal.get_untracked()), &store);
    // `return () => registerTrigger(null)` (`:94`) — the unmount half, registered on this component's
    // own owner so it runs when the part is disposed. The callback holds an `Rc`, which cannot cross
    // `on_cleanup`'s `Send + Sync` bound directly, so it rides the `SendWrapper` bridge (the
    // `popover/parts.rs:577-590` idiom).
    {
        let register_for_cleanup = SendWrapper::new(register_trigger.clone());
        on_cleanup(move || {
            register_for_cleanup.call(None);
        });
    }
    let register_for_effect = SendWrapper::new(register_trigger);

    let store_for_registration = store.clone();
    let extra_attributes = std::rc::Rc::new(element_attributes);
    let node_ref = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| {
        // `NodeRef`'s untracked read is leptos's own `Get` (the 0.7 surface), not
        // `reactive_graph 0.2`'s — the version split `link_item.rs` documents.
        let Some(node) =
            <NodeRef<leptos::html::Div> as leptos::prelude::GetUntracked>::get_untracked(&node_ref)
        else {
            return;
        };
        let element: web_sys::Element = web_sys::wasm_bindgen::JsCast::unchecked_into(node);

        // The `...elementProps` rest (`:46`) replayed onto the real node — `view!` has no attribute
        // spread, so the caller's bag is written here (the [`crate::menu::link_item::LinkItem`]
        // mount-writer shape), AFTER the part's own members, which is upstream's bag order (`:208`).
        for (name, value) in extra_attributes.iter() {
            let _ = element.set_attribute(name, value);
        }

        // `registerTrigger(triggerElementRef.current)` (`:93`).
        register_for_effect.call(Some(element.clone()));

        // `if (element !== null && store.select('open') && store.select('activeTriggerId') == null)`
        // (`:71`) — the claim runs only while nothing owns the trigger.
        if menu_store_is_open(&store_for_registration)
            && use_menu_store_active_trigger_id(&store_for_registration).is_none()
        {
            claim_menu_active_trigger(
                &store_for_registration,
                Some(id_signal.get_untracked()),
                element,
                close_delay,
            );
        }
    });

    let class = class.unwrap_or_default();
    let style_attribute = style
        .into_iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<String>();

    // The parent store keeps the `onBlur` write (`:202-206`); cloned for the handler.
    let parent_store_for_blur = parent_menu_store.clone();

    view! {
        <div
            role=MENU_SUBMENU_TRIGGER_ROLE
            id=move || id_signal.get()
            tabindex=move || menu_submenu_trigger_tab_index(open.get(), highlighted.get())
            aria-controls=move || {
                menu_submenu_trigger_aria_controls(popup_id.get(), &id_signal.get())
            }
            // `shouldOmitExpanded ? VOICE_OVER_EXPANDED_PROPS : undefined` (`:198`) — `None` drops
            // the attribute rather than rendering `false`.
            aria-expanded=move || {
                menu_submenu_trigger_aria_expanded(
                    open.get(),
                    opened_by_keyboard.get(),
                    leptos_ui_utils::platform::platform().screen_reader.voice_over,
                )
            }
            aria-haspopup=MENU_SUBMENU_TRIGGER_HASPOPUP
            // `data-popup-open` / `data-disabled` / `data-highlighted` are the shared engine's output
            // over the trigger's own state map (`:175,187`) — the view mirrors what
            // [`menu_submenu_trigger_attributes`] resolves rather than re-deriving it, so the two
            // cannot drift.
            data-popup-open=move || open.get().then_some("")
            data-disabled=move || disabled_signal.get().then_some("")
            data-highlighted=move || highlighted.get().then_some("")
            class=class
            style=style_attribute
            node_ref=node_ref
            on:blur=move |_| {
                menu_submenu_trigger_on_blur(
                    &parent_store_for_blur,
                    highlighted.get_untracked(),
                );
            }
        >
            {children()}
        </div>
    }
}
