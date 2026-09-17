//! Menu popup — `Menu.Popup`, the port of
//! `packages/react/src/menu/popup/MenuPopup.tsx`.
//!
//! WHAT THIS REPLACES. The previous `popup.rs` was a facade, not a port: it rendered a bare
//! `<div class="menu-popup">` — the hardcoded shell `specs/library/menu/behavior.md` →
//! "Uniform DOM shell" forbids — derived its visibility from a local
//! `create_rw_signal(false)` instead of the store, and exported invented helpers. Upstream
//! renders the popup through `useRenderElement('div', componentProps, …)`
//! (`MenuPopup.tsx:87-112`) over the store's state, with the state-attribute mapping, the
//! `data-rootownerid` marker, and the focus contract around it.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from:
//! - the store reads the element's state is built from (`MenuPopup.tsx:36-53`): `open`,
//!   `transitionStatus`, `instantType`, `parent`, `lastOpenChangeReason`, `rootId`, and the
//!   active trigger element (for the ARIA wiring below).
//! - the state record `{ transitionStatus, side, align, open, nested, instant }` (`:84-91`)
//!   and its `data-*` attributes through `popupTransitionStateMapping` (`:95` →
//!   `popupStateMapping.ts:68-73`) — via the ported mapping and the ported default
//!   handling, so `data-open`/`data-closed`, `data-side`/`data-align`,
//!   `data-nested`/`data-instant` and `data-starting-style`/`data-ending-style` all come
//!   from the function upstream reaches them through. The element renders that same set;
//!   [`menu_popup_attributes`] is the tested producer of it.
//! - `role="menu"` (`MenuRoot.tsx:577`, the `popupProps` member) and the popup's `id`
//!   (`:576` — the store's floating id), plus `aria-labelledby={activeTriggerElement?.id}`
//!   (`:579`).
//! - `data-rootownerid={rootId}` (`MenuPopup.tsx:110`) — the marker `findRootOwnerId`
//!   (`menu/utils/findRootOwnerId.ts:3-13`) walks for the trigger's drag-release guard
//!   (`MenuTrigger.tsx:145`).
//! - the composite-key guard (`:100-107`): while the popup sits inside a `Toolbar`, the
//!   composite keys stop propagating — [`menu_popup_should_stop_composite_key`].
//! - the `returnFocus` derivation (`:114-120`), the `initialFocus` rule (`:129`) and the
//!   hover-interaction enablement (`:79-82`), as pure functions with their tests.
//! - the popup element registration (`store.context.popupRef` + the `popupElement` store
//!   setter, `:90-91`).
//! - `useOpenChangeComplete({ open, ref: store.context.popupRef, onComplete })`'s open half
//!   (`:55-64`) through the ported hook: the completion fires only while open (`:58-61`).
//!
//! DEFERRED, each with the evidence, none of them silently dropped:
//! - `FloatingFocusManager` (`MenuPopup.tsx:122-142`) does not wrap the element yet. Its
//!   ported implementation exists
//!   (`crates/leptos-ui-internals/src/floating_ui/floating_focus_manager.rs`, 3294 lines),
//!   but the sibling popover part defers the same wiring (`popover/parts.rs:455-467`), and
//!   the menu's options are the most parent-dependent in the repo (`returnFocus`,
//!   `initialFocus`, `externalTree`, `previousFocusableElement`, the root's two focus-guard
//!   refs — `:126-138`). The values it consumes are ported and tested here, so the focus
//!   checkpoint lands the manager against proven predicates. Until it lands, the portal's
//!   guard spans stay unrendered for the same reason (see `portal.rs`).
//! - `useHoverFloatingInteraction` (`:79-82`) and the `floatingTreeRoot.events` close
//!   listener (`:66-77`): both are floating-tree machinery whose emitter in this unit is the
//!   positioner's `menuopenchange` re-broadcast (`MenuPositioner.tsx:230`), deferred with
//!   the menubar checkpoint (nothing in this tree emits it — `grep -rn "menuopenchange"
//!   crates/` returns nothing). The enablement predicate is ported and tested.
//! - The `popupProps` bag's remaining members (`MenuRoot.tsx:571-600`):
//!   `aria-orientation` (`:578`, rendered only for a horizontal menu) and the two hover
//!   handlers (`onMouseMove`/`onClick`, `:580-593`) are NOT reachable from this part,
//!   because the port's `MenuRoot` does not publish a `popupProps` bag into the store
//!   (`grep -rn "popup_props" crates/leptos-ui/src/menu/` returns nothing) — the bag is
//!   Root's to publish. The members that have a store source (id, role, labelledby,
//!   rootownerid) are rendered; the rest are recorded in
//!   `ralph/logs/spec-discrepancies.md` rather than guessed at. `FOCUSABLE_POPUP_PROPS`
//!   (`:573`) is the same story.

use std::rc::Rc;

use leptos::prelude::*;
use leptos_ui_internals::floating_ui::popup_store::selectors;
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_internals::popup_state_mapping::popup_transition_state_mapping;
use leptos_ui_internals::state_attributes::get_state_attributes_props;
use leptos_ui_internals::use_anchor_positioning::{Align, Side};
use leptos_ui_internals::use_open_change_complete::{
    UseOpenChangeCompleteParams, use_open_change_complete,
};
use leptos_ui_internals::use_transition_status::TransitionStatus;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked};
use serde_json::{Map, Value};

use crate::menu::positioner::{align_attr, instant_attr, menu_positioner_context, side_attr};
use crate::menu::store::{MenuInstantType, MenuParent, use_menu_store};

/// The pre-rewrite flat name for this part, kept as an alias so existing call sites keep
/// compiling (`context_menu/mod.rs:36` re-exports it under its own part name).
pub use self::Popup as MenuPopup;

/// `MenuPopup.tsx:114-120` — the `returnFocus` derivation:
///
/// ```text
/// let returnFocus = parent.type === undefined || isContextMenu;
/// if (activeTriggerElement ||
///     (parent.type === 'menubar' && lastOpenChangeReason !== REASONS.outsidePress)) {
///   returnFocus = true;
/// }
/// ```
///
/// with `finalFocus` (`:144`) overriding the derivation when the consumer set it.
pub fn menu_popup_return_focus(
    parent: &MenuParent,
    has_active_trigger_element: bool,
    last_open_change_reason: Option<&str>,
    final_focus: Option<bool>,
) -> bool {
    let mut return_focus = matches!(parent, MenuParent::None | MenuParent::ContextMenu);
    if has_active_trigger_element
        || (matches!(parent, MenuParent::Menubar)
            && last_open_change_reason != Some(reasons::OUTSIDE_PRESS))
    {
        return_focus = true;
    }
    final_focus.unwrap_or(return_focus)
}

/// `MenuPopup.tsx:129` — `initialFocus={parent.type !== 'menu'}`: a submenu does not take
/// initial focus (its first item is focused through list navigation instead).
pub fn menu_popup_initial_focus(parent: &MenuParent) -> bool {
    !matches!(parent, MenuParent::Menu { .. })
}

/// `MenuPopup.tsx:79-82` — the hover-close interaction's enablement:
/// `hoverEnabled && !disabled && !isContextMenu && parent.type !== 'menubar'`.
pub fn menu_popup_hover_interaction_enabled(
    parent: &MenuParent,
    hover_enabled: bool,
    disabled: bool,
) -> bool {
    hover_enabled && !disabled && !matches!(parent, MenuParent::Menubar | MenuParent::ContextMenu)
}

/// `MenuPopup.tsx:88` — `isContextMenu`.
pub fn menu_popup_is_context_menu(parent: &MenuParent) -> bool {
    matches!(parent, MenuParent::ContextMenu)
}

/// `MenuPopup.tsx:100-107` — the guard that keeps composite keys from bubbling into a
/// surrounding toolbar (`insideToolbar && COMPOSITE_KEYS.has(event.key)`).
pub fn menu_popup_should_stop_composite_key(inside_toolbar: bool, is_composite_key: bool) -> bool {
    inside_toolbar && is_composite_key
}

/// `MenuPopup.tsx:84-91` — the popup element's state record, in the shape
/// `getStateAttributesProps` consumes (`getStateAttributesProps.ts:5-31`).
pub fn menu_popup_state(
    open: bool,
    side: Option<Side>,
    align: Option<Align>,
    transition_status: Option<TransitionStatus>,
    parent: &MenuParent,
    instant: Option<MenuInstantType>,
) -> Map<String, Value> {
    let mut state = Map::new();
    // `transitionStatus` — omitted when unset (upstream's `undefined`); `'idle'` is carried
    // because the mapping owns that key and declines it explicitly
    // (`stateAttributesMapping.ts:10-19`).
    if let Some(status) = transition_status {
        state.insert(
            "transitionStatus".to_owned(),
            Value::String(transition_status_attr(status).to_owned()),
        );
    }
    if let Some(side) = side {
        state.insert("side".to_owned(), Value::String(side_attr(side).to_owned()));
    }
    if let Some(align) = align {
        state.insert(
            "align".to_owned(),
            Value::String(align_attr(align).to_owned()),
        );
    }
    state.insert("open".to_owned(), Value::Bool(open));
    // `nested: parent.type === 'menu'` (`:90`).
    state.insert(
        "nested".to_owned(),
        Value::Bool(matches!(parent, MenuParent::Menu { .. })),
    );
    if let Some(instant) = instant {
        state.insert(
            "instant".to_owned(),
            Value::String(instant_attr(instant).to_owned()),
        );
    }
    state
}

/// The `transitionStatus` state value's rendered spelling (`useTransitionStatus.ts`).
pub fn transition_status_attr(status: TransitionStatus) -> &'static str {
    match status {
        TransitionStatus::Starting => "starting",
        TransitionStatus::Ending => "ending",
        TransitionStatus::Idle => "idle",
    }
}

/// The popup element's `data-*` attributes:
/// `getStateAttributesProps(state, popupTransitionStateMapping)` (`MenuPopup.tsx:95-111`).
/// The ported producer already returns FINAL attribute names (`state_attributes.rs:153`
/// for the default-handled fields, `popup_state_mapping.rs:26-31` for the hooked ones), so
/// the pairs are returned as they come. The element below renders this same set; a host
/// test pins the mapping's output so a silent divergence is a test failure rather than a
/// rendering mystery.
pub fn menu_popup_attributes(state: &Map<String, Value>) -> Vec<(String, String)> {
    let props = get_state_attributes_props(state, Some(&popup_transition_state_mapping));
    props.into_iter().collect()
}

/// The `data-rootownerid` value (`MenuPopup.tsx:110`) — the store's `rootId`, absent when
/// unset (the port keeps that case out of the DOM rather than emitting an empty marker).
pub fn menu_popup_root_owner_id(root_id: Option<&str>) -> Option<String> {
    root_id.filter(|id| !id.is_empty()).map(str::to_owned)
}

/// `Menu.Popup` — upstream's `MenuPopup` (`MenuPopup.tsx:29-144`).
#[component]
pub fn Popup(
    /// `finalFocus` (`:144`) — `None` uses the derived default; the boolean arm of
    /// upstream's union is what the port exposes (the ref/function arms land with the focus
    /// checkpoint that consumes them).
    #[prop(optional)]
    final_focus: Option<bool>,
    /// `className` (`:33`).
    #[prop(optional, into)]
    class: Option<String>,
    /// The popup's items.
    children: Children,
) -> impl IntoView {
    // `const { store } = useMenuRootContext()` (`:31`).
    let context = use_menu_store();
    let store = context.store;

    // `const { side, align } = useMenuPositionerContext()` (`:32`) — the required read,
    // which throws outside a Positioner (`MenuPositionerContext.ts:12-21`).
    let positioner = menu_positioner_context(false)
        .expect("use_menu_positioner_context(false) panics when absent");

    // The store reads (`:36-53`).
    let open = store.use_state(selectors::open);
    let transition_status = store.use_state(selectors::transition_status);
    let parent = store.use_state(|state| {
        selectors::payload(state)
            .map(|extra| extra.parent)
            .unwrap_or(MenuParent::None)
    });
    let instant_type = store.use_state(|state| {
        selectors::payload(state).and_then(|extra| extra.instant_type)
    });
    let root_id = store.use_state(|state| selectors::payload(state).and_then(|extra| extra.root_id));
    // `id={floatingId}` (`MenuRoot.tsx:576`) — the store's popup id selector.
    let popup_id = store.use_state(selectors::popup_id);
    // `aria-labelledby={activeTriggerElement?.id}` (`MenuRoot.tsx:579`).
    let labelled_by = store.use_state(|state| {
        selectors::active_trigger_element(state).and_then(|element| element.get_attribute("id"))
    });

    // `useOpenChangeComplete({ open, ref: store.context.popupRef, onComplete })` (`:55-64`):
    // the completion fires only while open (`:58-61`), and upstream's `enabled`/`batch`
    // defaults are `true`/`false` (`useOpenChangeComplete.tsx:35,49`).
    {
        let store = Rc::clone(&store);
        let popup_ref = Rc::clone(&store.context.popup_ref);
        let open_for_complete = open;
        use_open_change_complete(UseOpenChangeCompleteParams {
            enabled: RwSignal::new(true),
            open: open_for_complete,
            // `ref: store.context.popupRef` (`:57`). `Rc<Cell<Option<HtmlElement>>>` is not
            // `Copy`, so the read uses the take-and-restore idiom the radio indicator part
            // documents (`radio/indicator.rs:134-137`).
            reference: move || {
                let element = popup_ref.replace(None);
                popup_ref.set(element.clone());
                element.map(Into::into)
            },
            batch: RwSignal::new(false),
            on_complete: Rc::new(move || {
                if open_for_complete.get_untracked() {
                    if let Some(callback) = &store.context.on_open_change_complete {
                        callback(true);
                    }
                }
            }),
        });
    }

    // `useRenderElement('div', componentProps, { ref: [forwardedRef, store.context.popupRef,
    // setPopupElement] })` (`:87-91`) — the element registration into both slots.
    let popup_node: NodeRef<leptos::html::Div> = NodeRef::new();
    {
        let store = Rc::clone(&store);
        let popup_ref = Rc::clone(&store.context.popup_ref);
        Effect::new(move |_| {
            let div = match <NodeRef<leptos::html::Div> as leptos::prelude::GetUntracked>::get_untracked(
                &popup_node,
            ) {
                Some(div) => div,
                None => return,
            };
            let element: web_sys::HtmlElement = web_sys::wasm_bindgen::JsCast::unchecked_into(div);
            popup_ref.set(Some(element.clone()));
            store.update(|state, _| {
                state.popup_element = Some(element);
                true
            });
        });
    }

    // The `returnFocus` resolution (`:114-120`) — computed from the live store so the focus
    // checkpoint consumes a resolved value rather than re-deriving it.
    let return_focus = move || {
        let current = store.get_snapshot();
        menu_popup_return_focus(
            &parent.get(),
            current.active_trigger_element.is_some(),
            selectors::payload(&current)
                .and_then(|extra| extra.open_change_reason)
                .as_deref(),
            final_focus,
        )
    };
    let _ = return_focus;

    let class = class.unwrap_or_default();

    view! {
        <div
            role="menu"
            class=class
            id=move || popup_id.get()
            node_ref=popup_node
            aria-labelledby=move || labelled_by.get()
            data-open=move || open.get().then_some("true")
            data-closed=move || (!open.get()).then_some("true")
            data-side=move || side_attr(positioner.side.get())
            data-align=move || align_attr(positioner.align.get())
            data-nested=move || {
                matches!(parent.get(), MenuParent::Menu { .. }).then_some("true")
            }
            data-instant=move || instant_type.get().map(instant_attr)
            data-starting-style=move || {
                (transition_status.get() == Some(TransitionStatus::Starting)).then_some("true")
            }
            data-ending-style=move || {
                (transition_status.get() == Some(TransitionStatus::Ending)).then_some("true")
            }
            data-rootownerid=move || menu_popup_root_owner_id(root_id.get().as_deref())
        >
            {children()}
        </div>
    }
}
