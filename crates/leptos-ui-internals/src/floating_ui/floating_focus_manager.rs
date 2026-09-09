//! Port of `packages/react/src/floating-ui-react/components/FloatingFocusManager.tsx` —
//! provides focus management for the floating element: initial focus on open, focus
//! return on close, focus restoration, modal trapping, and outside-content hiding
//! (`specs/library/floating-ui-react/behavior.md`, "Focus management" + "Accessibility":
//! the FocusManager ⇄ utils pipeline — `utils/tabbable` classification feeds the
//! initial-focus/tab-trap/tabindex decisions while `utils/markOthers` implements the
//! modal AT-hiding).
//!
//! ## Upstream shape being ported
//!
//! - `getEventType` (`FloatingFocusManager.tsx:49-70`) — close-modality inference
//!   (keyboard vs pointer) from the native event, preferring the last tracked
//!   pointer/keyboard interaction for focus/click events that cannot self-classify.
//! - The module-level `WeakRef` LRU of previously focused elements (`:72-94`) — backs the
//!   "return to last focused before body" fallback.
//! - `getFirstTabbableElement` (`:96-106`) and `handleTabIndex` (`:108-145`) — the
//!   managed `tabindex` (`0`↔`-1` with the `data-tabindex` write mirror) for
//!   role="dialog" floating elements.
//! - The component body (`:251-1005`) — nine effects: modal Tab prevention over an
//!   empty-content floating element (`:328-348`), pointer/keyboard interaction tracking
//!   to disambiguate focus and outside presses (`:351-406`), close-on-focus-out with
//!   in-tree focus restoration (`:409-602`), outside-content marking via `markOthers`
//!   (`:605-662`), initial focus on open (`:665-746`), return-focus bookkeeping through
//!   the `openchange` bus with the cleanup-time restore (`:749-902`), the WebKit
//!   typeable-blur on close (`:907-920`), the focus-manager-state push to the portal
//!   context (`:924-940`), and the floating-element tabindex sync (`:943-951`) — plus
//!   the two inside `FocusGuard` spans (`:953-1003`).
//!
//! ## Rust adaptations
//!
//! - The crate is view-free (the `floating_portal`/`floating_delay_group` convention), so
//!   upstream's `<FloatingFocusManager>{children}</FloatingFocusManager>` render ports to
//!   [`provide_floating_focus_manager`], a provider-body function returning a
//!   [`FloatingFocusManagerHandle`] whose two inside-guard spans the consumer places
//!   flanking the floating element — upstream renders them around `children` at the call
//!   site (`:958-1002`), a position the view-free port cannot know. The
//!   create-when-active/remove-when-inactive lifecycle mirrors the portal's realized
//!   outside guards (the `RealizedElement` machinery, shared from `floating_portal`).
//! - React state ports to store-backed signals: `open`, `domReference`, and `floating`
//!   are `store.useState` selectors (`:271-273`), and `floatingFocusElement` derives
//!   from them through `getFloatingFocusElement` (`:315`). The boolean props port to
//!   signals too — `disabled`/`modal`/`closeOnFocusOut`/`restoreFocus` are reactive
//!   inputs upstream (the keep-mounted `disabled` flip re-runs every effect through the
//!   dependency arrays), and the port's effects track the signal reads the same way.
//! - The `useValueAsRef` mirrors (`:285-288`) collapse: `initialFocus`/`returnFocus`/
//!   `openInteractionType` are read fresh inside the effect/microtask closures from the
//!   captured option values (upstream's refs exist so effects see the latest render's
//!   prop without resubscribing — the port's options are construction-time values, and
//!   function forms own their state), while `open` reads the store signal untracked
//!   (upstream's `openRef.current`, `:721`).
//! - The prop unions port to enums: `initialFocus`/`returnFocus`'s
//!   `boolean | RefObject | function` become [`InitialFocus`]/[`ReturnFocus`] with a
//!   shared [`ResolvedFocusTarget`] decision type (`true`/`null` → default behavior,
//!   `false`/`undefined`/`void` → nothing, element → focus it — `:170-172`, `:186-188`),
//!   and `restoreFocus`'s `boolean | 'popup'` becomes [`RestoreFocus`].
//!   `nextFocusableElement`/`previousFocusableElement` resolve at construction (their
//!   upstream ref form is re-read per event through `resolveRef`; a stable element is
//!   the port's equivalent input). Upstream's `''` empty `InteractionType` member is the
//!   port's [`InteractionType::Unknown`], and the `null`-vs-`undefined`
//!   `openInteractionType` distinction (only an explicit `null` is a programmatic open,
//!   `:756-759`) ports to [`FloatingFocusManagerOptions::open_interaction_type`] being
//!   `None` vs `Some(InteractionType::Unknown)`.
//! - The `WeakRef` LRU ports to `js_sys::WeakRef` — the same JS semantics (entries free
//!   themselves when the element is collected; `deref()?.isConnected` prunes, `:76-78`).
//! - `queueMicrotask` runs through `window.queueMicrotask` with a once-closure (the
//!   `use_list_navigation` precedent), and the `focus({ preventScroll, focusVisible })`
//!   restore call builds its options object through `Reflect` — `focusVisible` is a
//!   Base UI extension property web-sys's `FocusOptions` dictionary does not model. The
//!   `preventScroll` support probe (`:789-797`) reads the getter the same way.
//! - The `getNodeChildren(nodes, id)` walk in the focus-out handler uses upstream's
//!   default `onlyOpenChildren = true` (`:471`); the return-focus cleanup walk passes
//!   `false` explicitly (`:855`).
//! - **Store-mirror seeding:** the store's `useState` mirror effects run their first
//!   executor-timed pass after mount and re-write the seeded signal values (the mirror
//!   seed write cannot be skipped — see `leptos_ui_utils::react_store`), which re-runs
//!   every store-reading effect once right after construction with no observable state
//!   change. The return-focus effect's cleanup therefore performs one extra
//!   never-opened-session bookkeeping pass right after mount: the `returnFocus`
//!   function form receives one call with the empty interaction type before any real
//!   session ends (upstream calls it once per session end). The restore itself is
//!   suppressed on that pass by upstream's own "focus moved elsewhere" guard
//!   (`:876-879`) — focus has not entered the floating tree, so the default return
//!   target is not the active element and not the body, making `isFocusInsideFloatingTree`
//!   gate the restore off. The wasm tests assert the session-end call
//!   (`close_types.last()`).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use floating_ui_dom::dom::{DomNodeOrWindow, get_node_name};
use reactive_graph::owner::{LocalStorage, on_cleanup};
use reactive_graph::traits::{Get, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{
    Document, Element, Event, FocusEvent, HtmlElement, KeyboardEvent, MouseEvent, PointerEvent,
    TouchEvent,
};

use js_sys::Reflect;

use leptos_ui_utils::merge_cleanups::{CleanupFn, merge_cleanups};
use leptos_ui_utils::owner::owner_document;
use leptos_ui_utils::platform;
use leptos_ui_utils::shadow_dom::{active_element, contains, get_target};
use leptos_ui_utils::use_animation_frame;
use leptos_ui_utils::use_enhanced_click_handler::InteractionType;
use leptos_ui_utils::use_iso_layout_effect;
use leptos_ui_utils::use_timeout;

use crate::floating_ui::composite::is_element_visible;
use crate::floating_ui::constants::CLICK_TRIGGER_IDENTIFIER;
use crate::floating_ui::create_attribute::create_attribute;
use crate::floating_ui::element::{
    focus_element, get_floating_focus_element, is_typeable_combobox, is_typeable_element,
};
use crate::floating_ui::element_props::FloatingContextSource;
use crate::floating_ui::enqueue_focus::{EnqueueFocusOptions, enqueue_focus};
use crate::floating_ui::event::{is_virtual_click, is_virtual_pointer_event, stop_event};
use crate::floating_ui::floating_portal::{
    FocusManagerState, RealizedElement, SharedFloatingPortalContext, SharedGuardRef, SharedNodeRef,
    use_portal_context,
};
use crate::floating_ui::focus_guard::{FocusGuardProps, create_focus_guard};
use crate::floating_ui::mark_others::{MarkOthersOptions, mark_others};
use crate::floating_ui::nodes::{get_node_ancestors, get_node_children};
use crate::floating_ui::reasons;
use crate::floating_ui::tabbable::{
    focusable, get_next_tabbable, get_previous_tabbable, is_outside_event, is_tabbable, tabbable,
};
use crate::floating_ui::tree::{SharedFloatingTreeStore, use_floating_tree};
use crate::floating_ui::types::{
    FloatingUIOpenChangeDetails, OnOpenChangeFn, RootOpenChangeEventDetails,
};

// ---------------------------------------------------------------------------
// Close-modality inference (`getEventType`, `FloatingFocusManager.tsx:49-70`)
// ---------------------------------------------------------------------------

/// `getEventType(event, lastInteractionType)` (`FloatingFocusManager.tsx:49-70`): the
/// close-interaction type inferred from the event driving an open-state change. Focus
/// and click events cannot self-classify (a focusout can trail a pointer press; a click
/// may carry no pointer events), so the last tracked interaction wins when known
/// (`lastInteractionType || ...`), and a `detail: 0` click is keyboard. The terminal
/// fallthrough (`:69`) is `''` — [`InteractionType::Unknown`].
pub fn get_event_type(event: &Event, last_interaction_type: InteractionType) -> InteractionType {
    // `lastInteractionType || fallback` — the empty member defers to the fallback.
    let last_or = |fallback: InteractionType| {
        if last_interaction_type == InteractionType::Unknown {
            fallback
        } else {
            last_interaction_type
        }
    };

    if event.dyn_ref::<KeyboardEvent>().is_some() {
        return InteractionType::Keyboard;
    }
    if event.dyn_ref::<FocusEvent>().is_some() {
        // Focus events can be caused by a preceding pointer interaction; prefer the last
        // known pointer type if provided, else treat as keyboard (`:55-57`).
        return last_or(InteractionType::Keyboard);
    }
    if let Some(pointer_event) = event.dyn_ref::<PointerEvent>() {
        // `'pointerType' in event` (`:59`) — the runtime discriminator is the
        // PointerEvent interface; an empty pointerType falls back to keyboard (`:60`).
        return match pointer_event.pointer_type().as_str() {
            "mouse" => InteractionType::Mouse,
            "touch" => InteractionType::Touch,
            "pen" => InteractionType::Pen,
            _ => InteractionType::Keyboard,
        };
    }
    if event.dyn_ref::<TouchEvent>().is_some() {
        return InteractionType::Touch;
    }
    if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
        // `lastInteractionType || (event.detail === 0 ? 'keyboard' : 'mouse')` (`:67`).
        return if last_interaction_type != InteractionType::Unknown {
            last_interaction_type
        } else if mouse_event.detail() == 0 {
            InteractionType::Keyboard
        } else {
            InteractionType::Mouse
        };
    }
    InteractionType::Unknown
}

// ---------------------------------------------------------------------------
// The previously-focused-element LRU (`FloatingFocusManager.tsx:72-94`)
// ---------------------------------------------------------------------------

/// `LIST_LIMIT` (`:72`).
const LIST_LIMIT: usize = 20;

thread_local! {
    /// `previouslyFocusedElements` (`:73`) — the module-level `WeakRef<Element>[]`; the
    /// JS `WeakRef` semantics are kept (`js_sys::WeakRef`) so entries free themselves
    /// when the element is collected.
    static PREVIOUSLY_FOCUSED_ELEMENTS: RefCell<Vec<js_sys::WeakRef>> =
        const { RefCell::new(Vec::new()) };
}

/// `clearDisconnectedPreviouslyFocusedElements` (`:75-79`).
fn clear_disconnected_previously_focused_elements() {
    PREVIOUSLY_FOCUSED_ELEMENTS.with_borrow_mut(|entries| {
        entries.retain(|entry| {
            entry
                .deref()
                .and_then(|value| value.dyn_ref::<Element>().cloned())
                .is_some_and(|element| element.is_connected())
        });
    });
}

/// `addPreviouslyFocusedElement` (`:81-89`): body-focused elements are not recorded, and
/// the list is trimmed to the last [`LIST_LIMIT`] entries.
fn add_previously_focused_element(element: Option<&Element>) {
    clear_disconnected_previously_focused_elements();
    let Some(element) = element else {
        return;
    };
    if get_node_name(DomNodeOrWindow::Node(element)) == "body" {
        return;
    }
    PREVIOUSLY_FOCUSED_ELEMENTS.with_borrow_mut(|entries| {
        entries.push(js_sys::WeakRef::new(element.as_ref()));
        if entries.len() > LIST_LIMIT {
            *entries = entries.split_off(entries.len() - LIST_LIMIT);
        }
    });
}

/// `getPreviouslyFocusedElement` (`:91-94`).
fn get_previously_focused_element() -> Option<Element> {
    clear_disconnected_previously_focused_elements();
    PREVIOUSLY_FOCUSED_ELEMENTS.with_borrow(|entries| {
        entries
            .last()
            .and_then(|entry| entry.deref())
            .and_then(|value| value.dyn_ref::<Element>().cloned())
    })
}

// ---------------------------------------------------------------------------
// Tabbable-content helpers (`FloatingFocusManager.tsx:96-145`)
// ---------------------------------------------------------------------------

/// `getFirstTabbableElement(container)` (`:96-106`): the container when it is itself
/// tabbable, else its first tabbable descendant, else the container itself.
fn get_first_tabbable_element(container: Option<&Element>) -> Option<Element> {
    let container = container?;
    if is_tabbable(Some(container)) {
        return Some(container.clone());
    }
    tabbable(container)
        .into_iter()
        .next()
        .or_else(|| Some(container.clone()))
}

/// `handleTabIndex(floatingFocusElement)` (`:108-145`): keep the floating element's
/// managed `tabindex` in sync — `0` when the role="dialog" floating element has no
/// tabbable content (so it can hold focus), `-1` once it does, with every managed write
/// mirrored into `data-tabindex` so an externally-authored `tabindex` freezes
/// management (`:109-114`).
fn handle_tab_index(floating_focus_element: &HtmlElement) {
    if floating_focus_element.has_attribute("tabindex")
        && !floating_focus_element.has_attribute("data-tabindex")
    {
        return;
    }

    if !floating_focus_element
        .get_attribute("role")
        .is_some_and(|role| role.contains("dialog"))
    {
        return;
    }

    let tabbable_content: Vec<Element> = focusable(floating_focus_element)
        .into_iter()
        .filter(|element| {
            let data_tab_index = element.get_attribute("data-tabindex").unwrap_or_default();
            is_tabbable(Some(element))
                || (element.has_attribute("data-tabindex") && !data_tab_index.starts_with('-'))
        })
        .collect();
    let tab_index = floating_focus_element.get_attribute("tabindex");

    if tabbable_content.is_empty() {
        if tab_index.as_deref() != Some("0") {
            floating_focus_element
                .set_attribute("tabindex", "0")
                .expect("attribute name is valid");
            // Mark our own write so the externally-managed early-return above doesn't
            // mistake it for a user-authored `tabindex` and freeze management
            // (`:133-135`).
            floating_focus_element
                .set_attribute("data-tabindex", "0")
                .expect("attribute name is valid");
        }
    } else if tab_index.as_deref() != Some("-1")
        || (floating_focus_element.has_attribute("data-tabindex")
            && floating_focus_element
                .get_attribute("data-tabindex")
                .as_deref()
                != Some("-1"))
    {
        floating_focus_element
            .set_attribute("tabindex", "-1")
            .expect("attribute name is valid");
        floating_focus_element
            .set_attribute("data-tabindex", "-1")
            .expect("attribute name is valid");
    }
}

/// `getTabbableContent(container = floatingFocusElement)` (`:317-321`): the container's
/// tabbable descendants, empty when there is no container.
fn get_tabbable_content(floating_focus_element: Option<&Element>) -> Vec<Element> {
    floating_focus_element.map(tabbable).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Prop vocabulary (`FloatingFocusManagerProps`, `FloatingFocusManager.tsx:147-244`)
// ---------------------------------------------------------------------------

/// A fillable element-ref slot — upstream's `React.RefObject<HTMLElement | null>` in the
/// `initialFocus`/`returnFocus` prop unions (`:169`, `:186`). The same shape as the
/// portal's [`SharedNodeRef`].
pub type SharedFocusRef = SharedNodeRef;

/// The decision an `initialFocus`/`returnFocus` function form reports (`:170-172`,
/// `:186-188`): an element to focus, `true`/`null` for the default behavior, or
/// `false`/`undefined`/`void` to do nothing.
pub enum ResolvedFocusTarget {
    /// `false` / `undefined` / `void` — do not move focus.
    Nothing,
    /// `true` / `null` — use the default behavior.
    Default,
    /// An explicit element to focus.
    Element(Element),
}

/// Port of `FloatingFocusManagerProps['initialFocus']` (`:175-179`).
#[derive(Clone)]
pub enum InitialFocus {
    /// The boolean form — `false` does not move focus, `true` is the default behavior.
    Bool(bool),
    /// The ref form — the referenced element (an empty ref falls back to the default,
    /// `:708-710`).
    Ref(SharedFocusRef),
    /// The function form — called with the open interaction type.
    Fn(Rc<dyn Fn(InteractionType) -> ResolvedFocusTarget>),
}

/// Port of `FloatingFocusManagerProps['returnFocus']` (`:191-195`): the same union over
/// the close interaction type.
#[derive(Clone)]
pub enum ReturnFocus {
    /// The boolean form — `false` leaves focus where it is, `true` is the default
    /// behavior.
    Bool(bool),
    /// The ref form — the referenced element (an empty ref falls back to the default,
    /// `:843`).
    Ref(SharedFocusRef),
    /// The function form — called with the close interaction type.
    Fn(Rc<dyn Fn(InteractionType) -> ResolvedFocusTarget>),
}

/// Port of `FloatingFocusManagerProps['restoreFocus']` (`:206` — `boolean | 'popup'`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RestoreFocus {
    /// `false` (the default) — do not restore focus.
    #[default]
    False,
    /// `'popup'` — restore directly to the floating element (container) itself.
    Popup,
    /// `true` — restore to the nearest tabbable element inside the floating tree.
    True,
}

/// `getInsideElements` (`:243`) — additional elements treated as part of the floating
/// subtree even when rendered outside the floating element. Upstream's
/// `Array<Element | null | undefined>` return ports to `Vec<Element>` with nulls
/// dropped at the resolution site (`:323-325`).
pub type InsideElementsFn = Rc<dyn Fn() -> Vec<Option<Element>>>;

/// Port of `FloatingFocusManagerProps` (`:147-244`).
pub struct FloatingFocusManagerOptions {
    /// `context` (`:152`) — the floating context returned from
    /// `useFloatingRootContext`/`useFloating`, or the store itself.
    pub context: FloatingContextSource,
    /// `openInteractionType` (`:156`) — the interaction type used to open the floating
    /// element. `None` is upstream's explicit `null` (a programmatic open — the
    /// return-focus target prefers the element focused before open, `:756-759`);
    /// `Some(InteractionType::Unknown)` is the `''` default.
    pub open_interaction_type: Option<InteractionType>,
    /// `disabled` (`:163`, default `false`) — whether focus management is disabled.
    pub disabled: Signal<bool, LocalStorage>,
    /// `initialFocus` (`:175`, default `true`).
    pub initial_focus: InitialFocus,
    /// `returnFocus` (`:191`, default `true`).
    pub return_focus: ReturnFocus,
    /// `restoreFocus` (`:206`, default `false`).
    pub restore_focus: Signal<RestoreFocus, LocalStorage>,
    /// `modal` (`:213`, default `true`) — whether focus is fully trapped.
    pub modal: Signal<bool, LocalStorage>,
    /// `closeOnFocusOut` (`:221`, default `true`).
    pub close_on_focus_out: Signal<bool, LocalStorage>,
    /// `nextFocusableElement` (`:225`) — the Tab-forward focus override.
    pub next_focusable_element: Option<Element>,
    /// `previousFocusableElement` (`:229`) — the Tab-backward focus override.
    pub previous_focusable_element: Option<Element>,
    /// `beforeContentFocusGuardRef` (`:234`) — a caller handle the before-content guard
    /// registers into (useful to focus the popup programmatically).
    pub before_content_focus_guard_ref: Option<SharedGuardRef>,
    /// `externalTree` (`:238`) — an external `FloatingTree` for the nested-focus walks.
    pub external_tree: Option<SharedFloatingTreeStore>,
    /// `getInsideElements` (`:243`).
    pub get_inside_elements: Option<InsideElementsFn>,
}

impl Default for FloatingFocusManagerOptions {
    fn default() -> Self {
        Self {
            // `context` is required upstream (`:152`); the throwaway store from
            // `getEmptyRootContext` stands in for the Default convenience.
            context: FloatingContextSource::Store(
                crate::floating_ui::get_empty_root_context::get_empty_root_context(),
            ),
            open_interaction_type: Some(InteractionType::Unknown),
            disabled: Signal::derive_local(|| false),
            initial_focus: InitialFocus::Bool(true),
            return_focus: ReturnFocus::Bool(true),
            restore_focus: Signal::derive_local(|| RestoreFocus::False),
            modal: Signal::derive_local(|| true),
            close_on_focus_out: Signal::derive_local(|| true),
            next_focusable_element: None,
            previous_focusable_element: None,
            before_content_focus_guard_ref: None,
            external_tree: None,
            get_inside_elements: None,
        }
    }
}

// ---------------------------------------------------------------------------
// The component body (`FloatingFocusManager.tsx:251-1005`)
// ---------------------------------------------------------------------------

/// The handle returned by [`provide_floating_focus_manager`] — the two inside
/// `FocusGuard` spans (`:958-1002`), realized while [`should_render_guards`] holds and
/// removed when it stops; the consumer places them flanking the floating element
/// (upstream renders them around `children` at the call site).
pub struct FloatingFocusManagerHandle {
    /// The `<FocusGuard data-type="inside">` before the floating element (`:959-978`).
    pub before_inside_guard: RealizedElement,
    /// The `<FocusGuard data-type="inside">` after the floating element (`:981-1001`).
    pub after_inside_guard: RealizedElement,
}

/// `queueMicrotask` through `window.queueMicrotask` with a once-closure — the
/// `use_list_navigation` precedent.
fn queue_microtask(callback: impl FnOnce() + 'static) {
    if let Some(window) = web_sys::window() {
        let function =
            js_sys::Function::from(wasm_bindgen::closure::Closure::once_into_js(callback));
        window.queue_microtask(&function);
    }
}

/// `element.focus({ preventScroll, focusVisible })` (`:881-885`): the restore call.
/// `focusVisible` is a Base UI extension property web-sys's `FocusOptions` dictionary
/// does not model, so the options object is built through `Reflect`.
fn focus_with_options(element: &Element, prevent_scroll: bool, focus_visible: bool) {
    let options = js_sys::Object::new();
    let _ = Reflect::set(&options, &"preventScroll".into(), &prevent_scroll.into());
    if focus_visible {
        let _ = Reflect::set(&options, &"focusVisible".into(), &true.into());
    }
    let focus_function: Option<js_sys::Function> = Reflect::get(element.as_ref(), &"focus".into())
        .ok()
        .and_then(|value| value.dyn_into::<js_sys::Function>().ok());
    if let Some(focus_function) = focus_function {
        let _ = focus_function.call1(element, &options);
    }
}

/// The `preventScroll` support probe (`:789-797`): focus a throwaway element with a
/// getter-backed `preventScroll` option — the getter runs when the browser reads the
/// option, so a raised flag means the runtime supports it.
fn probe_prevent_scroll_support(document: &Document) -> bool {
    let supported = Rc::new(Cell::new(false));
    if let Ok(div) = document.create_element("div") {
        let options = js_sys::Object::new();
        let flag = Rc::clone(&supported);
        let getter = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            flag.set(true);
            false
        }) as Box<dyn Fn() -> bool>);
        if Reflect::set(
            &options,
            &"preventScroll".into(),
            getter.as_ref().unchecked_ref(),
        )
        .is_ok()
        {
            let focus_function: Option<js_sys::Function> =
                Reflect::get(div.as_ref(), &"focus".into())
                    .ok()
                    .and_then(|value| value.dyn_into::<js_sys::Function>().ok());
            if let Some(focus_function) = focus_function {
                let _ = focus_function.call1(&div, &options);
            }
        }
        getter.forget();
    }
    supported.get()
}

/// `shouldRenderGuards` (`:953-954`): `!disabled && (modal ? !isUntrappedTypeableCombobox
/// : true) && (isInsidePortal || modal)` — extracted for the host-testable pin.
fn should_render_guards(
    disabled: bool,
    modal: bool,
    is_untrapped_typeable_combobox: bool,
    is_inside_portal: bool,
) -> bool {
    !disabled && (!modal || !is_untrapped_typeable_combobox) && (is_inside_portal || modal)
}

/// The before-inside guard's focus handler (`:962-976`): modal wraps focus to the last
/// tabbable inside; non-modal routes Tab to the next document tabbable (outside event)
/// or to the previous focusable override / the portal's before-outside guard. The
/// signals are read fresh at event time (the floating_portal outside-guard pattern).
fn before_inside_guard_focus_handler(
    modal: Signal<bool, LocalStorage>,
    floating_focus_element_signal: Signal<Option<Element>, LocalStorage>,
    portal_context: Option<SharedFloatingPortalContext>,
    prevent_return_focus: Rc<Cell<bool>>,
    dom_reference_signal: Signal<Option<Element>, LocalStorage>,
    previous_focusable_element: Option<Element>,
) -> impl FnMut(&Event) + 'static {
    move |event: &Event| {
        if modal.get() {
            let floating_focus_element = floating_focus_element_signal.get_untracked();
            let tabbable_content = get_tabbable_content(floating_focus_element.as_ref());
            // `enqueueFocus(els[els.length - 1])` (`:966`) — the rAF-cancel return is
            // intentionally not kept (`:965`).
            let _ = enqueue_focus(
                tabbable_content
                    .last()
                    .and_then(|element| element.dyn_ref::<HtmlElement>()),
                EnqueueFocusOptions::default(),
            );
        } else if portal_context
            .as_ref()
            .and_then(|context| context.portal_node.get_untracked())
            .is_some()
        {
            let portal_node: Option<HtmlElement> = portal_context
                .as_ref()
                .and_then(|context| context.portal_node.get_untracked());
            prevent_return_focus.set(false);
            let focus_event: &FocusEvent = match event.dyn_ref() {
                Some(focus_event) => focus_event,
                None => return,
            };
            if is_outside_event(focus_event, portal_node.as_deref()) {
                // `getNextTabbable(domReference)?.focus()` (`:970-971`).
                let dom_reference = dom_reference_signal.get_untracked();
                if let Some(next) = get_next_tabbable(dom_reference.as_ref()) {
                    focus_element(&next);
                }
            } else {
                // `resolveRef(previousFocusableElement ??
                // portalContext.beforeOutsideRef)?.focus()` (`:973`).
                let fallback: Option<HtmlElement> = portal_context
                    .as_ref()
                    .and_then(|context| context.before_outside_ref.borrow().clone());
                if let Some(previous) = previous_focusable_element.as_ref().or(fallback.as_deref())
                {
                    focus_element(previous);
                }
            }
        }
    }
}

/// The after-inside guard's focus handler (`:984-999`): modal wraps focus to the first
/// tabbable inside; non-modal suppresses the next return focus when closing on focus
/// out, routing Tab to the previous document tabbable (outside event) or to the next
/// focusable override / the portal's after-outside guard.
fn after_inside_guard_focus_handler(
    modal: Signal<bool, LocalStorage>,
    close_on_focus_out: Signal<bool, LocalStorage>,
    floating_focus_element_signal: Signal<Option<Element>, LocalStorage>,
    portal_context: Option<SharedFloatingPortalContext>,
    prevent_return_focus: Rc<Cell<bool>>,
    dom_reference_signal: Signal<Option<Element>, LocalStorage>,
    next_focusable_element: Option<Element>,
) -> impl FnMut(&Event) + 'static {
    move |event: &Event| {
        if modal.get() {
            let floating_focus_element = floating_focus_element_signal.get_untracked();
            let tabbable_content = get_tabbable_content(floating_focus_element.as_ref());
            // `enqueueFocus(getTabbableContent()[0])` (`:987`).
            let _ = enqueue_focus(
                tabbable_content
                    .first()
                    .and_then(|element| element.dyn_ref::<HtmlElement>()),
                EnqueueFocusOptions::default(),
            );
        } else if portal_context
            .as_ref()
            .and_then(|context| context.portal_node.get_untracked())
            .is_some()
        {
            let portal_node: Option<HtmlElement> = portal_context
                .as_ref()
                .and_then(|context| context.portal_node.get_untracked());
            if close_on_focus_out.get() {
                prevent_return_focus.set(true);
            }

            let focus_event: &FocusEvent = match event.dyn_ref() {
                Some(focus_event) => focus_event,
                None => return,
            };
            if is_outside_event(focus_event, portal_node.as_deref()) {
                // `getPreviousTabbable(domReference)?.focus()` (`:994-995`).
                let dom_reference = dom_reference_signal.get_untracked();
                if let Some(previous) = get_previous_tabbable(dom_reference.as_ref()) {
                    focus_element(&previous);
                }
            } else {
                // `resolveRef(nextFocusableElement ?? portalContext.afterOutsideRef)?
                // .focus()` (`:997`).
                let fallback: Option<HtmlElement> = portal_context
                    .as_ref()
                    .and_then(|context| context.after_outside_ref.borrow().clone());
                if let Some(next) = next_focusable_element.as_ref().or(fallback.as_deref()) {
                    focus_element(next);
                }
            }
        }
    }
}

/// `getReturnElement(closeType)` (`:809-844`): the return-focus target resolution —
/// the explicit prop value (boolean/ref/function) over the default chain
/// (`preferPreviousFocus ? previous || reference : reference || previous`, then the
/// previously-focused LRU fallback, `:831-837`).
fn get_return_element(
    return_focus: &ReturnFocus,
    close_type: InteractionType,
    dom_reference: Option<&Element>,
    element_focused_before_open: Option<&Element>,
    prefer_previous_focus: bool,
) -> Option<Element> {
    // The explicit value (`:810-823`): `undefined || false` → no return; `null` → the
    // default behavior; a ref resolves at the `resolveRef(resolved) || default || null`
    // point (`:843`).
    let resolved = match return_focus {
        ReturnFocus::Bool(false) => return None,
        ReturnFocus::Bool(true) => ResolvedFocusTarget::Default,
        ReturnFocus::Ref(handle) => match handle.borrow().clone() {
            Some(element) => ResolvedFocusTarget::Element(element.into()),
            None => ResolvedFocusTarget::Default,
        },
        ReturnFocus::Fn(resolve) => resolve(close_type),
    };
    if matches!(resolved, ResolvedFocusTarget::Nothing) {
        return None;
    }

    let reference_return_element: Option<Element> = dom_reference
        .filter(|element| element.is_connected())
        .cloned();
    let previous_return_element: Option<Element> = element_focused_before_open
        .filter(|element| {
            element.is_connected() && get_node_name(DomNodeOrWindow::Node(element)) != "body"
        })
        .cloned();

    let mut default_return_element = if prefer_previous_focus {
        previous_return_element.or(reference_return_element)
    } else {
        reference_return_element.or(previous_return_element)
    };

    if default_return_element.is_none() {
        default_return_element = get_previously_focused_element();
    }

    match resolved {
        ResolvedFocusTarget::Default => default_return_element,
        ResolvedFocusTarget::Element(element) => Some(element).or(default_return_element),
        ResolvedFocusTarget::Nothing => None,
    }
}

/// Port of the `FloatingFocusManager` component (`:251-1005`), view-free (see the module
/// docs). Must be called inside a reactive owner (a component).
pub fn provide_floating_focus_manager(
    options: FloatingFocusManagerOptions,
) -> FloatingFocusManagerHandle {
    let FloatingFocusManagerOptions {
        context,
        open_interaction_type,
        disabled,
        initial_focus,
        return_focus,
        restore_focus,
        modal,
        close_on_focus_out,
        next_focusable_element,
        previous_focusable_element,
        before_content_focus_guard_ref,
        external_tree,
        get_inside_elements,
    } = options;

    // `const store = 'rootStore' in context ? context.rootStore : context` (`:269`) — the
    // `FloatingContextSource` normalization.
    let store = context.root_store();
    let inner = store.rc();

    // `const open = store.useState('open')` etc. (`:271-273`).
    let open_signal = inner.use_state(crate::floating_ui::floating_root_store::selectors::open);
    let dom_reference_signal =
        inner.use_state(crate::floating_ui::floating_root_store::selectors::dom_reference_element);
    let floating_signal =
        inner.use_state(crate::floating_ui::floating_root_store::selectors::floating_element);
    let events = store.context.events.clone();
    let data_ref = store.context.data_ref.clone();

    // `const getNodeId = useStableCallback(() => dataRef.current.floatingContext?.nodeId)`
    // (`:276`) — the port reads the stashed node id (`ContextData::floating_node_id`).
    let get_node_id: Rc<dyn Fn() -> Option<String>> = {
        let data_ref = Rc::clone(&data_ref);
        Rc::new(move || data_ref.borrow().floating_node_id.clone())
    };

    // `const ignoreInitialFocus = initialFocus === false` (`:278`).
    let ignore_initial_focus = matches!(initial_focus, InitialFocus::Bool(false));

    // `const isUntrappedTypeableCombobox` (`:279-283`) — reactive over the DOM reference.
    let is_untrapped_typeable_combobox: Signal<bool, LocalStorage> = Signal::derive_local({
        let dom_reference_signal = dom_reference_signal.clone();
        move || {
            is_typeable_combobox(dom_reference_signal.get_untracked().as_ref())
                && ignore_initial_focus
        }
    });

    // `const tree = useFloatingTree(externalTree)` (`:290`) and
    // `const portalContext = usePortalContext()` (`:291`).
    let tree = use_floating_tree(external_tree);
    let portal_context: Option<SharedFloatingPortalContext> = use_portal_context();
    let is_inside_portal = portal_context.is_some();

    // The per-session mutable state (`:293-301`).
    let prevent_return_focus: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let is_pointer_down: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let pointer_down_outside: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let last_focused_tabbable: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));
    let close_type: Rc<Cell<InteractionType>> = Rc::new(Cell::new(InteractionType::Unknown));
    let last_interaction_type: Rc<Cell<InteractionType>> =
        Rc::new(Cell::new(InteractionType::Unknown));

    // `const beforeGuardRef/afterGuardRef` (`:300-301`).
    let before_guard_ref: SharedGuardRef = Rc::new(RefCell::new(None));
    let after_guard_ref: SharedGuardRef = Rc::new(RefCell::new(None));

    // `const blurTimeout = useTimeout()` etc. (`:310-312`).
    let blur_timeout = use_timeout();
    let pointer_down_timeout = use_timeout();
    let restore_focus_frame = use_animation_frame();

    // `const floatingFocusElement = getFloatingFocusElement(floating)` (`:315`) —
    // reactive over the store's floating element.
    let floating_focus_element_signal: Signal<Option<Element>, LocalStorage> =
        Signal::derive_local({
            let floating_signal = floating_signal.clone();
            move || get_floating_focus_element(floating_signal.get().as_ref())
        });

    // `const getResolvedInsideElements` (`:323-325`).
    let get_resolved_inside_elements: Rc<dyn Fn() -> Vec<Element>> = match &get_inside_elements {
        Some(resolve) => {
            let resolve = Rc::clone(resolve);
            Rc::new(move || resolve().into_iter().flatten().collect())
        }
        None => Rc::new(|| Vec::new()),
    };

    // -----------------------------------------------------------------------
    // Prevent Tab from escaping the modal when there are no tabbable elements
    // (`:328-348`).
    // -----------------------------------------------------------------------
    {
        let disabled = disabled.clone();
        let modal = modal.clone();
        let floating_focus_element_signal = floating_focus_element_signal.clone();
        let is_untrapped_typeable_combobox = is_untrapped_typeable_combobox.clone();
        use_iso_layout_effect(move || {
            let disabled_value = disabled.get();
            let modal_value = modal.get();
            let floating_focus_element = floating_focus_element_signal.get();
            let is_untrapped = is_untrapped_typeable_combobox.get();
            // `if (disabled || !modal) return undefined` (`:329-331`).
            if disabled_value || !modal_value {
                return;
            }
            let Some(floating_focus_element) = floating_focus_element else {
                return;
            };
            let doc = owner_document(Some(floating_focus_element.as_ref()));
            let keydown = leptos_ui_utils::add_event_listener(&doc, "keydown", {
                let floating_focus_element = floating_focus_element.clone();
                move |event: &Event| {
                    let keyboard_event: &KeyboardEvent = match event.dyn_ref() {
                        Some(keyboard_event) => keyboard_event,
                        None => return,
                    };
                    if keyboard_event.key() == "Tab" {
                        // The focus guards have nothing to focus, so we need to stop the
                        // event (`:335-342`).
                        let active =
                            active_element(&owner_document(Some(floating_focus_element.as_ref())));
                        if contains(Some(&floating_focus_element), active.as_ref())
                            && get_tabbable_content(Some(&floating_focus_element)).is_empty()
                            && !is_untrapped
                        {
                            stop_event(event);
                        }
                    }
                }
            });
            let keydown = SendWrapper::new(keydown);
            on_cleanup(move || keydown.take().unsubscribe());
        });
    }

    // -----------------------------------------------------------------------
    // Track pointer/keyboard interactions to disambiguate focus and outside
    // presses (`:351-406`).
    // -----------------------------------------------------------------------
    {
        let disabled = disabled.clone();
        let open_signal = open_signal.clone();
        let floating_signal = floating_signal.clone();
        let dom_reference_signal = dom_reference_signal.clone();
        let floating_focus_element_signal = floating_focus_element_signal.clone();
        let portal_context = portal_context.clone();
        let is_pointer_down = Rc::clone(&is_pointer_down);
        let pointer_down_outside = Rc::clone(&pointer_down_outside);
        let last_interaction_type = Rc::clone(&last_interaction_type);
        let pointer_down_timeout = pointer_down_timeout.clone();
        let get_resolved_inside_elements = Rc::clone(&get_resolved_inside_elements);
        use_iso_layout_effect(move || {
            // `if (disabled || !open) return undefined` (`:352-354`).
            let disabled_value = disabled.get();
            let open_value = open_signal.get();
            if disabled_value || !open_value {
                return;
            }

            let floating = floating_signal.get();
            let dom_reference = dom_reference_signal.get();
            let floating_focus_element = floating_focus_element_signal.get();
            // The portal node is tracked so a host mount/unmount re-runs the
            // registration (upstream's `portalContext` dependency changes with it,
            // `:403`), and read fresh in the handler.
            let portal_node = portal_context
                .as_ref()
                .map(|context| context.portal_node.get())
                .unwrap_or(None);

            let doc = owner_document(
                floating_focus_element
                    .as_ref()
                    .map(|element| element.as_ref()),
            );

            // `clearPointerDownOutside` (`:358-360`).
            let clear_pointer_down_outside = {
                let pointer_down_outside = Rc::clone(&pointer_down_outside);
                move || pointer_down_outside.set(false)
            };

            // `onPointerDown` (`:362-382`).
            let on_pointer_down = {
                let floating = floating.clone();
                let dom_reference = dom_reference.clone();
                let portal_node = portal_node.clone();
                let portal_context = portal_context.clone();
                let pointer_down_outside = Rc::clone(&pointer_down_outside);
                let last_interaction_type = Rc::clone(&last_interaction_type);
                let is_pointer_down = Rc::clone(&is_pointer_down);
                let pointer_down_timeout = pointer_down_timeout.clone();
                let get_resolved_inside_elements = Rc::clone(&get_resolved_inside_elements);
                move |event: &Event| {
                    let target = get_target(event);
                    let target_element: Option<Element> = target
                        .as_ref()
                        .and_then(|target| target.dyn_ref::<Element>().cloned());
                    let inside_elements = get_resolved_inside_elements();
                    let pointer_target_inside =
                        contains(floating.as_ref(), target_element.as_ref())
                            || contains(dom_reference.as_ref(), target_element.as_ref())
                            || contains(portal_node.as_deref(), target_element.as_ref())
                            || inside_elements.iter().any(|element| {
                                target_element
                                    .as_ref()
                                    .is_some_and(|target| target == element)
                                    || contains(Some(element), target_element.as_ref())
                            })
                            || portal_context.as_ref().is_some_and(|context| {
                                contains(
                                    context.portal_node.get_untracked().as_deref(),
                                    target_element.as_ref(),
                                )
                            });
                    pointer_down_outside.set(!pointer_target_inside);
                    // `lastInteractionTypeRef.current = event.pointerType || 'keyboard'`
                    // (`:371-372`).
                    let pointer_event: Option<&PointerEvent> = event.dyn_ref();
                    last_interaction_type.set(match pointer_event {
                        Some(pointer_event) => match pointer_event.pointer_type().as_str() {
                            "mouse" => InteractionType::Mouse,
                            "touch" => InteractionType::Touch,
                            "pen" => InteractionType::Pen,
                            _ => InteractionType::Keyboard,
                        },
                        None => InteractionType::Keyboard,
                    });

                    // `target?.closest(`[${CLICK_TRIGGER_IDENTIFIER}]`)` (`:374`).
                    if target_element
                        .as_ref()
                        .and_then(|element| {
                            element
                                .closest(&format!("[{CLICK_TRIGGER_IDENTIFIER}]"))
                                .ok()
                                .flatten()
                        })
                        .is_some()
                    {
                        is_pointer_down.set(true);
                        // Reset on the next tick so a single click on a click-trigger
                        // doesn't permanently suppress focus-out closing (`:376-381`).
                        let is_pointer_down = Rc::clone(&is_pointer_down);
                        pointer_down_timeout.start(0, move || {
                            is_pointer_down.set(false);
                        });
                    }
                }
            };

            // `onKeyDown` (`:384-386`).
            let on_key_down = {
                let last_interaction_type = Rc::clone(&last_interaction_type);
                move |_event: &Event| {
                    last_interaction_type.set(InteractionType::Keyboard);
                }
            };

            // The capture-phase `pointerup`/`pointercancel` clearers — the listener
            // shape wraps the zero-arg clearer.
            let clear_pointer_up = {
                let clear = clear_pointer_down_outside.clone();
                move |_event: &Event| clear()
            };
            let clear_pointer_cancel = {
                let clear = clear_pointer_down_outside.clone();
                move |_event: &Event| clear()
            };

            let pointerdown_listener =
                leptos_ui_utils::add_event_listener(&doc, "pointerdown", on_pointer_down);
            let pointerup_listener = leptos_ui_utils::add_event_listener_with_options(
                &doc,
                "pointerup",
                clear_pointer_up,
                true,
            );
            let pointercancel_listener = leptos_ui_utils::add_event_listener_with_options(
                &doc,
                "pointercancel",
                clear_pointer_cancel,
                true,
            );
            let keydown_listener = leptos_ui_utils::add_event_listener_with_options(
                &doc,
                "keydown",
                on_key_down,
                true,
            );

            let cleanups: Vec<Option<CleanupFn>> = vec![
                Some(Box::new(move || pointerdown_listener.unsubscribe()) as CleanupFn),
                Some(Box::new(move || pointerup_listener.unsubscribe()) as CleanupFn),
                Some(Box::new(move || pointercancel_listener.unsubscribe()) as CleanupFn),
                Some(Box::new(move || keydown_listener.unsubscribe()) as CleanupFn),
            ];
            let merged: CleanupFn = Box::new(merge_cleanups(cleanups));
            let merged = SendWrapper::new(merged);
            let clear_for_cleanup = SendWrapper::new(Rc::clone(&pointer_down_outside));
            on_cleanup(move || {
                merged.take()();
                // Avoid a stale `true` leaking into the next open (e.g. keep-mounted
                // popups) if the popup dismissed between pointerdown and pointerup
                // (`:394-395`).
                clear_for_cleanup.set(false);
            });
        });
    }

    // -----------------------------------------------------------------------
    // Close on focus out and restore focus within the floating tree when needed
    // (`:409-602`).
    // -----------------------------------------------------------------------
    {
        let disabled = disabled.clone();
        let close_on_focus_out = close_on_focus_out.clone();
        let restore_focus = restore_focus.clone();
        let modal = modal.clone();
        let floating_signal = floating_signal.clone();
        let dom_reference_signal = dom_reference_signal.clone();
        let floating_focus_element_signal = floating_focus_element_signal.clone();
        let is_untrapped_typeable_combobox = is_untrapped_typeable_combobox.clone();
        let portal_context = portal_context.clone();
        let tree = tree.clone();
        let data_ref = Rc::clone(&data_ref);
        let store = Rc::clone(&store);
        let get_node_id = Rc::clone(&get_node_id);
        let get_resolved_inside_elements = Rc::clone(&get_resolved_inside_elements);
        let is_pointer_down = Rc::clone(&is_pointer_down);
        let pointer_down_outside = Rc::clone(&pointer_down_outside);
        let last_focused_tabbable = Rc::clone(&last_focused_tabbable);
        let prevent_return_focus = Rc::clone(&prevent_return_focus);
        let blur_timeout = blur_timeout.clone();
        let pointer_down_timeout = pointer_down_timeout.clone();
        let restore_focus_frame = restore_focus_frame.clone();
        let next_focusable_element = next_focusable_element.clone();
        let previous_focusable_element = previous_focusable_element.clone();
        let before_guard_ref = Rc::clone(&before_guard_ref);
        let after_guard_ref = Rc::clone(&after_guard_ref);
        use_iso_layout_effect(move || {
            // `if (disabled || !closeOnFocusOut) return undefined` (`:410-412`).
            let disabled_value = disabled.get();
            let close_on_focus_out_value = close_on_focus_out.get();
            if disabled_value || !close_on_focus_out_value {
                return;
            }
            let restore_focus_value = restore_focus.get();
            let modal_value = modal.get();
            let is_untrapped = is_untrapped_typeable_combobox.get();
            let floating = floating_signal.get();
            let dom_reference = dom_reference_signal.get();
            let floating_focus_element = floating_focus_element_signal.get();
            let portal_node = portal_context
                .as_ref()
                .map(|context| context.portal_node.get())
                .unwrap_or(None);

            let doc = owner_document(
                floating_focus_element
                    .as_ref()
                    .map(|element| element.as_ref()),
            );

            // In Safari, buttons lose focus when pressing them (`:416-422`).
            let on_pointer_down = {
                let is_pointer_down = Rc::clone(&is_pointer_down);
                let pointer_down_timeout = pointer_down_timeout.clone();
                move |_event: &Event| {
                    is_pointer_down.set(true);
                    let is_pointer_down = Rc::clone(&is_pointer_down);
                    pointer_down_timeout.start(0, move || {
                        is_pointer_down.set(false);
                    });
                }
            };

            // `handleFocusIn` (`:424-429`).
            let on_focus_in = {
                let last_focused_tabbable = Rc::clone(&last_focused_tabbable);
                move |event: &Event| {
                    let target = get_target(event);
                    let target_element: Option<Element> = target
                        .as_ref()
                        .and_then(|target| target.dyn_ref::<Element>().cloned());
                    if is_tabbable(target_element.as_ref()) {
                        *last_focused_tabbable.borrow_mut() = target_element;
                    }
                }
            };

            // `handleFocusOutside` (`:431-554`).
            let on_focus_out = {
                let floating = floating.clone();
                let dom_reference = dom_reference.clone();
                let floating_focus_element = floating_focus_element.clone();
                let portal_node = portal_node.clone();
                let portal_context = portal_context.clone();
                let tree = tree.clone();
                let data_ref = Rc::clone(&data_ref);
                let store = Rc::clone(&store);
                let get_node_id = Rc::clone(&get_node_id);
                let get_resolved_inside_elements = Rc::clone(&get_resolved_inside_elements);
                let is_pointer_down = Rc::clone(&is_pointer_down);
                let prevent_return_focus = Rc::clone(&prevent_return_focus);
                let last_focused_tabbable = Rc::clone(&last_focused_tabbable);
                let restore_focus_frame = restore_focus_frame.clone();
                let next_focusable_element = next_focusable_element.clone();
                let previous_focusable_element = previous_focusable_element.clone();
                let before_guard_ref = Rc::clone(&before_guard_ref);
                let after_guard_ref = Rc::clone(&after_guard_ref);
                move |event: &Event| {
                    let focus_event: &FocusEvent = match event.dyn_ref() {
                        Some(focus_event) => focus_event,
                        None => return,
                    };
                    let related_target: Option<Element> = focus_event
                        .related_target()
                        .and_then(|target| target.dyn_into::<Element>().ok());
                    let current_target: Option<Element> = event
                        .current_target()
                        .and_then(|target| target.dyn_into::<Element>().ok());
                    let target =
                        get_target(event).and_then(|target| target.dyn_ref::<Element>().cloned());

                    // When focus is lost to the body (e.g. on a backdrop press), record
                    // the element that had focus so a confirmation dialog opened while
                    // the body is focused can return focus to it. Scoped to `modal`
                    // (`:436-441`).
                    if modal_value
                        && related_target.is_none()
                        && target.is_some()
                        && contains(floating.as_ref(), target.as_ref())
                    {
                        add_previously_focused_element(target.as_ref());
                    }

                    // The microtask body re-derives every shared handle per invocation —
                    // the event handler must stay `FnMut` (the listener contract), so
                    // each dispatch clones the handles it hands to the queued task.
                    let event = event.clone();
                    let floating = floating.clone();
                    let dom_reference = dom_reference.clone();
                    let floating_focus_element = floating_focus_element.clone();
                    let portal_node = portal_node.clone();
                    let portal_context = portal_context.clone();
                    let tree = tree.clone();
                    let data_ref = Rc::clone(&data_ref);
                    let store = Rc::clone(&store);
                    let get_node_id = Rc::clone(&get_node_id);
                    let get_resolved_inside_elements = Rc::clone(&get_resolved_inside_elements);
                    let is_pointer_down = Rc::clone(&is_pointer_down);
                    let prevent_return_focus = Rc::clone(&prevent_return_focus);
                    let last_focused_tabbable = Rc::clone(&last_focused_tabbable);
                    let restore_focus_frame = restore_focus_frame.clone();
                    let next_focusable_element = next_focusable_element.clone();
                    let previous_focusable_element = previous_focusable_element.clone();
                    let before_guard_ref = Rc::clone(&before_guard_ref);
                    let after_guard_ref = Rc::clone(&after_guard_ref);
                    let doc = doc.clone();
                    queue_microtask(move || {
                        let node_id = get_node_id();
                        let inside_elements = get_resolved_inside_elements();

                        // `isRelatedFocusGuard` (`:447-458`).
                        let guard_candidates: Vec<Element> = [
                            before_guard_ref.borrow().clone().map(Element::from),
                            after_guard_ref.borrow().clone().map(Element::from),
                            portal_context
                                .as_ref()
                                .and_then(|context| context.before_inside_ref.borrow().clone())
                                .map(Element::from),
                            portal_context
                                .as_ref()
                                .and_then(|context| context.after_inside_ref.borrow().clone())
                                .map(Element::from),
                            portal_context
                                .as_ref()
                                .and_then(|context| context.before_outside_ref.borrow().clone())
                                .map(Element::from),
                            portal_context
                                .as_ref()
                                .and_then(|context| context.after_outside_ref.borrow().clone())
                                .map(Element::from),
                            previous_focusable_element.clone(),
                            next_focusable_element.clone(),
                        ]
                        .into_iter()
                        .flatten()
                        .collect();
                        let is_related_focus_guard =
                            related_target.as_ref().is_some_and(|related_target| {
                                related_target.has_attribute(&create_attribute("focus-guard"))
                                    && guard_candidates
                                        .iter()
                                        .any(|candidate| candidate == related_target)
                            });

                        // `movedToUnrelatedNode` (`:460-484`).
                        let tree_related = tree.as_ref().is_some_and(|tree| {
                            let nodes = tree.nodes.borrow();
                            // `getNodeChildren(tree.nodesRef.current, nodeId).find(...)`
                            // (`:471-475`) — the default `onlyOpenChildren = true`.
                            let children_hit = get_node_children(&nodes, node_id.as_deref(), true)
                                .iter()
                                .any(|node| {
                                    let context = node.context.borrow();
                                    let floating_of_node = context
                                        .as_ref()
                                        .map(|context| context.elements.floating.get_untracked())
                                        .unwrap_or(None);
                                    let dom_reference_of_node = context
                                        .as_ref()
                                        .map(|context| {
                                            context.elements.dom_reference.get_untracked()
                                        })
                                        .unwrap_or(None);
                                    contains(floating_of_node.as_ref(), related_target.as_ref())
                                        || contains(
                                            dom_reference_of_node.as_ref(),
                                            related_target.as_ref(),
                                        )
                                });
                            // `getNodeAncestors(tree.nodesRef.current, nodeId).find(...)`
                            // (`:476-483`).
                            let ancestors_hit = get_node_ancestors(&nodes, node_id.as_deref())
                                .iter()
                                .any(|node| {
                                    let context = node.context.borrow();
                                    let floating_of_node = context
                                        .as_ref()
                                        .map(|context| context.elements.floating.get_untracked())
                                        .unwrap_or(None);
                                    let focus_element_of_node =
                                        floating_of_node.as_ref().and_then(|element| {
                                            get_floating_focus_element(Some(element))
                                        });
                                    let dom_reference_of_node = context
                                        .as_ref()
                                        .map(|context| {
                                            context.elements.dom_reference.get_untracked()
                                        })
                                        .unwrap_or(None);
                                    related_target.as_ref().is_some_and(|related| {
                                        Some(related) == floating_of_node.as_ref()
                                            || Some(related) == focus_element_of_node.as_ref()
                                            || dom_reference_of_node.as_ref() == Some(related)
                                    })
                                });
                            children_hit || ancestors_hit
                        });

                        let triggers_hit =
                            store
                                .context
                                .trigger_elements
                                .has_matching_element(|trigger| {
                                    contains(Some(trigger), related_target.as_ref())
                                });

                        let moved_to_unrelated_node =
                            !(contains(dom_reference.as_ref(), related_target.as_ref())
                                || contains(floating.as_ref(), related_target.as_ref())
                                || contains(related_target.as_ref(), floating.as_ref())
                                || contains(portal_node.as_deref(), related_target.as_ref())
                                || inside_elements.iter().any(|element| {
                                    related_target
                                        .as_ref()
                                        .is_some_and(|related| related == element)
                                        || contains(Some(element), related_target.as_ref())
                                })
                                || triggers_hit
                                || is_related_focus_guard
                                || tree_related);

                        // `if (currentTarget === domReference && floatingFocusElement)`
                        // (`:486-488`).
                        if current_target.is_some() && current_target == dom_reference {
                            if let Some(floating_focus_html) = floating_focus_element
                                .as_ref()
                                .and_then(|element| element.dyn_ref::<HtmlElement>())
                            {
                                handle_tab_index(floating_focus_html);
                            }
                        }

                        // Restore focus to the previously focused tabbable element to
                        // prevent focus from being lost outside the floating tree
                        // (`:492-526`).
                        let body_element: Option<Element> =
                            doc.body().map(|body| body.unchecked_into());
                        if restore_focus_value != RestoreFocus::False
                            && current_target != dom_reference
                            && !is_element_visible(target.as_ref())
                            && active_element(&doc) == body_element
                        {
                            // Let `FloatingPortal` know that focus is still inside the
                            // floating tree (`:498-514`).
                            if let Some(floating_focus_html) = floating_focus_element
                                .as_ref()
                                .and_then(|element| element.dyn_ref::<HtmlElement>())
                            {
                                focus_element(floating_focus_html);
                                // If explicitly requested to restore focus to the popup
                                // container, do not search for the next/previous tabbable
                                // element; re-focusing asynchronously (next frame) wins
                                // the removal race (`:504-513`).
                                if restore_focus_value == RestoreFocus::Popup {
                                    let floating_focus_html = floating_focus_html.clone();
                                    restore_focus_frame.request(move || {
                                        focus_element(&floating_focus_html);
                                    });
                                    return;
                                }
                            }

                            let tabbable_content =
                                get_tabbable_content(floating_focus_element.as_ref());
                            let previous_tabbable = last_focused_tabbable.borrow().clone();
                            let node_to_focus = previous_tabbable
                                .filter(|previous| tabbable_content.contains(previous))
                                .or_else(|| tabbable_content.last().cloned())
                                .or_else(|| floating_focus_element.clone());

                            if let Some(node) = node_to_focus
                                .as_ref()
                                .and_then(|node| node.dyn_ref::<HtmlElement>())
                            {
                                focus_element(node);
                            }
                        }

                        // https://github.com/floating-ui/floating-ui/issues/3060
                        // (`:529-532`).
                        if data_ref.borrow().inside_react_tree {
                            data_ref.borrow_mut().inside_react_tree = false;
                            return;
                        }

                        // Focus did not move inside the floating tree, and there are no
                        // tabbable portal guards to handle closing (`:536-552`).
                        if (is_untrapped || !modal_value)
                            && related_target.is_some()
                            && moved_to_unrelated_node
                            && !is_pointer_down.get()
                            && (is_untrapped || related_target != get_previously_focused_element())
                        {
                            prevent_return_focus.set(true);
                            store.set_open(
                                false,
                                &RootOpenChangeEventDetails::new(
                                    reasons::FOCUS_OUT,
                                    event.clone(),
                                    None,
                                    String::new(),
                                ),
                            );
                        }
                    });
                }
            };

            // `markInsideReactTree` (`:556-564`).
            let on_focus_out_capture = {
                let pointer_down_outside = Rc::clone(&pointer_down_outside);
                let data_ref = Rc::clone(&data_ref);
                let blur_timeout = blur_timeout.clone();
                move |_event: &Event| {
                    if pointer_down_outside.get() {
                        return;
                    }
                    data_ref.borrow_mut().inside_react_tree = true;
                    let data_ref = Rc::clone(&data_ref);
                    blur_timeout.start(0, move || {
                        data_ref.borrow_mut().inside_react_tree = false;
                    });
                }
            };

            // `const domReferenceElement = isHTMLElement(domReference) ? domReference :
            // null` (`:566`).
            let dom_reference_element = dom_reference
                .as_ref()
                .and_then(|element| element.dyn_ref::<HtmlElement>().cloned());
            // `if (!floating && !domReferenceElement) return undefined` (`:567-569`).
            if floating.is_none() && dom_reference_element.is_none() {
                return;
            }

            // The listeners attach eagerly; the cleanup boxes hold the unsubscribe
            // handles (the floating_portal pattern).
            let reference_focus_out_listener = dom_reference_element.as_ref().map(|element| {
                leptos_ui_utils::add_event_listener(element, "focusout", on_focus_out.clone())
            });
            let reference_pointer_down_listener = dom_reference_element.as_ref().map(|element| {
                leptos_ui_utils::add_event_listener(element, "pointerdown", on_pointer_down.clone())
            });
            let floating_focus_in_listener = floating.as_ref().map(|element| {
                leptos_ui_utils::add_event_listener(element, "focusin", on_focus_in.clone())
            });
            let floating_focus_out_listener = floating.as_ref().map(|element| {
                leptos_ui_utils::add_event_listener(element, "focusout", on_focus_out.clone())
            });
            // `floating && portalContext && addEventListener(floating, 'focusout',
            // markInsideReactTree, true)` (`:577-579`).
            let floating_focus_out_capture_listener =
                match (floating.as_ref(), portal_context.as_ref()) {
                    (Some(floating), Some(_)) => {
                        Some(leptos_ui_utils::add_event_listener_with_options(
                            floating,
                            "focusout",
                            on_focus_out_capture.clone(),
                            true,
                        ))
                    }
                    _ => None,
                };

            let cleanups: Vec<Option<CleanupFn>> = vec![
                reference_focus_out_listener
                    .map(|listener| Box::new(move || listener.unsubscribe()) as CleanupFn),
                reference_pointer_down_listener
                    .map(|listener| Box::new(move || listener.unsubscribe()) as CleanupFn),
                floating_focus_in_listener
                    .map(|listener| Box::new(move || listener.unsubscribe()) as CleanupFn),
                floating_focus_out_listener
                    .map(|listener| Box::new(move || listener.unsubscribe()) as CleanupFn),
                floating_focus_out_capture_listener
                    .map(|listener| Box::new(move || listener.unsubscribe()) as CleanupFn),
            ];
            let merged: CleanupFn = Box::new(merge_cleanups(cleanups));
            let merged = SendWrapper::new(merged);
            on_cleanup(move || merged.take()());
        });
    }

    // -----------------------------------------------------------------------
    // Hide everything outside the floating tree from assistive tech while open
    // (`:605-662`).
    // -----------------------------------------------------------------------
    {
        let disabled = disabled.clone();
        let open_signal = open_signal.clone();
        let floating_signal = floating_signal.clone();
        let dom_reference_signal = dom_reference_signal.clone();
        let modal = modal.clone();
        let is_untrapped_typeable_combobox = is_untrapped_typeable_combobox.clone();
        let portal_context = portal_context.clone();
        let tree = tree.clone();
        let get_node_id = Rc::clone(&get_node_id);
        let get_resolved_inside_elements = Rc::clone(&get_resolved_inside_elements);
        let before_guard_ref = Rc::clone(&before_guard_ref);
        let after_guard_ref = Rc::clone(&after_guard_ref);
        let next_focusable_element = next_focusable_element.clone();
        let previous_focusable_element = previous_focusable_element.clone();
        use_iso_layout_effect(move || {
            let disabled_value = disabled.get();
            let open_value = open_signal.get();
            let floating = floating_signal.get();
            let dom_reference = dom_reference_signal.get();
            let modal_value = modal.get();
            let is_untrapped = is_untrapped_typeable_combobox.get();
            let portal_node = portal_context
                .as_ref()
                .map(|context| context.portal_node.get())
                .unwrap_or(None);
            // `if (disabled || !floating || !open) return undefined` (`:606-608`).
            if disabled_value || floating.is_none() || !open_value {
                return;
            }
            let floating = floating.expect("floating checked above");

            // Don't hide portals nested within the parent portal (`:611-613`).
            let portal_nodes: Vec<Element> = portal_node
                .as_ref()
                .map(|node| {
                    node.query_selector_all(&format!("[{}]", create_attribute("portal")))
                        .map(|list| {
                            let mut elements = Vec::new();
                            for index in 0..list.length() {
                                if let Some(node) = list.get(index) {
                                    if let Ok(element) = node.dyn_into::<Element>() {
                                        elements.push(element);
                                    }
                                }
                            }
                            elements
                        })
                        .unwrap_or_default()
                })
                .unwrap_or_default();

            let ancestors = tree
                .as_ref()
                .map(|tree| {
                    let nodes = tree.nodes.borrow();
                    get_node_ancestors(&nodes, get_node_id().as_deref())
                })
                .unwrap_or_default();
            // `rootAncestorComboboxDomReference` (`:616-618`).
            let root_ancestor_combobox_dom_reference = ancestors.iter().find_map(|node| {
                let context = node.context.borrow();
                let dom_reference_of_node = context
                    .as_ref()
                    .map(|context| context.elements.dom_reference.get_untracked())
                    .unwrap_or(None);
                is_typeable_combobox(dom_reference_of_node.as_ref())
                    .then_some(dom_reference_of_node)
                    .flatten()
            });

            // `controlInsideElements` (`:620-628`).
            let mut control_inside_elements: Vec<Element> =
                Vec::with_capacity(8 + portal_nodes.len());
            control_inside_elements.push(floating.clone());
            control_inside_elements.extend(portal_nodes.iter().cloned());
            if let Some(guard) = before_guard_ref.borrow().as_ref() {
                control_inside_elements.push(guard.clone().into());
            }
            if let Some(guard) = after_guard_ref.borrow().as_ref() {
                control_inside_elements.push(guard.clone().into());
            }
            if let Some(context) = portal_context.as_ref() {
                if let Some(guard) = context.before_outside_ref.borrow().as_ref() {
                    control_inside_elements.push(guard.clone().into());
                }
                if let Some(guard) = context.after_outside_ref.borrow().as_ref() {
                    control_inside_elements.push(guard.clone().into());
                }
            }
            control_inside_elements.extend(get_resolved_inside_elements());

            // `insideElements` (`:629-635`).
            let mut inside_elements = control_inside_elements.clone();
            if let Some(reference) = root_ancestor_combobox_dom_reference.as_ref() {
                inside_elements.push(reference.clone());
            }
            if let Some(previous) = previous_focusable_element.as_ref() {
                inside_elements.push(previous.clone());
            }
            if let Some(next) = next_focusable_element.as_ref() {
                inside_elements.push(next.clone());
            }
            if is_untrapped {
                if let Some(reference) = dom_reference.as_ref() {
                    inside_elements.push(reference.clone());
                }
            }

            // `markOthers(insideElements, { ariaHidden: modal ||
            // isUntrappedTypeableCombobox, mark: false })` (`:637-640`).
            let aria_hidden_cleanup = mark_others(
                &inside_elements,
                MarkOthersOptions {
                    aria_hidden: modal_value || is_untrapped,
                    inert: false,
                    mark: false,
                },
            );

            // `markOthers(markerInsideElements)` (`:642-643`) — the default options
            // (the marker attribute only).
            let marker_inside_elements: Vec<Element> = [floating]
                .into_iter()
                .chain(portal_nodes.iter().cloned())
                .collect();
            let marker_cleanup = mark_others(&marker_inside_elements, MarkOthersOptions::default());

            let merged: CleanupFn = Box::new(merge_cleanups(vec![
                // `return () => { markerCleanup(); ariaHiddenCleanup(); }` (`:645-648`).
                Some(Box::new(move || marker_cleanup()) as CleanupFn),
                Some(Box::new(move || aria_hidden_cleanup()) as CleanupFn),
            ]));
            let merged = SendWrapper::new(merged);
            on_cleanup(move || merged.take()());
        });
    }

    // -----------------------------------------------------------------------
    // Focus the initial element when the floating element opens (`:665-746`).
    // -----------------------------------------------------------------------
    {
        let disabled = disabled.clone();
        let open_signal = open_signal.clone();
        let floating_focus_element_signal = floating_focus_element_signal.clone();
        let initial_focus = initial_focus.clone();
        let close_type = Rc::clone(&close_type);
        let last_interaction_type = Rc::clone(&last_interaction_type);
        use_iso_layout_effect(move || {
            let open_value = open_signal.get();
            let disabled_value = disabled.get();
            let floating_focus_element = floating_focus_element_signal.get();
            // `if (!open || disabled || !isHTMLElement(floatingFocusElement)) return`
            // (`:666-668`).
            if !open_value || disabled_value {
                return;
            }
            let Some(floating_focus_element) =
                floating_focus_element.filter(|element| element.dyn_ref::<HtmlElement>().is_some())
            else {
                return;
            };

            close_type.set(InteractionType::Unknown);
            last_interaction_type.set(InteractionType::Unknown);

            let doc = owner_document(Some(floating_focus_element.as_ref()));
            let previously_focused_element = active_element(&doc);

            // The per-open-session resolution of `initialFocus`
            // (`resolvedInitialFocus`, `:679-687`): the function form is invoked once
            // with the open interaction type; `undefined`/`false` (and the fn's
            // `Nothing`) do nothing; `true`/`null` use the default element; the ref form
            // resolves at the `elToFocus` step.
            enum ResolvedInitial {
                Nothing,
                Default,
                ResolveRef,
                Element(Element),
            }
            let resolved = match &initial_focus {
                InitialFocus::Bool(true) => ResolvedInitial::Default,
                InitialFocus::Bool(false) => ResolvedInitial::Nothing,
                InitialFocus::Ref(_) => ResolvedInitial::ResolveRef,
                InitialFocus::Fn(resolve) => {
                    match resolve(open_interaction_type.unwrap_or(InteractionType::Unknown)) {
                        ResolvedFocusTarget::Nothing => ResolvedInitial::Nothing,
                        ResolvedFocusTarget::Default => ResolvedInitial::Default,
                        ResolvedFocusTarget::Element(element) => ResolvedInitial::Element(element),
                    }
                }
            };
            // `null` should fallback to default behavior in case of an empty ref;
            // `undefined`/`false` do nothing (`:685-687`).
            if matches!(resolved, ResolvedInitial::Nothing) {
                return;
            }

            let focus_already_inside_floating_el = contains(
                Some(&floating_focus_element),
                previously_focused_element.as_ref(),
            );
            if focus_already_inside_floating_el {
                return;
            }

            // `getDefaultFocusElement` (`:696-702`) — the memoized first tabbable.
            let mut focusable_elements: Option<Vec<Element>> = None;
            let get_default_focus_element = |focusable_elements: &mut Option<Vec<Element>>| {
                if focusable_elements.is_none() {
                    *focusable_elements = Some(get_tabbable_content(Some(&floating_focus_element)));
                }
                focusable_elements
                    .as_ref()
                    .and_then(|elements| elements.first().cloned())
                    .unwrap_or_else(|| floating_focus_element.clone())
            };

            // `elToFocus` (`:705-710`).
            let el_to_focus = match resolved {
                ResolvedInitial::Default => None,
                ResolvedInitial::Element(element) => Some(element),
                ResolvedInitial::ResolveRef => match &initial_focus {
                    InitialFocus::Ref(handle) => handle
                        .borrow()
                        .clone()
                        .map(|element| Element::from(element)),
                    _ => None,
                },
                ResolvedInitial::Nothing => None,
            }
            .unwrap_or_else(|| get_default_focus_element(&mut focusable_elements));

            let had_focus_inside =
                contains(Some(&floating_focus_element), active_element(&doc).as_ref());

            // enqueueFocus returns a rAF-cancel function; we intentionally don't cancel
            // this focus (`:715`).
            let _ = enqueue_focus(
                el_to_focus.dyn_ref::<HtmlElement>(),
                EnqueueFocusOptions {
                    prevent_scroll: Some(el_to_focus == floating_focus_element),
                    should_focus: Some(Box::new({
                        let open_signal = open_signal.clone();
                        let doc = doc.clone();
                        let floating_focus_element = floating_focus_element.clone();
                        let el_to_focus = el_to_focus.clone();
                        move || {
                            // This focus is queued on the next animation frame. If the
                            // floating element has closed before it runs — e.g. tabbing
                            // out of a kept-mounted popup — don't pull focus back onto
                            // the initial element after it has legitimately moved
                            // elsewhere (`:718-723`).
                            if !open_signal.get_untracked() {
                                return false;
                            }

                            if had_focus_inside {
                                return true;
                            }

                            let current_active_element = active_element(&doc);
                            let focus_moved_inside =
                                current_active_element.as_ref().is_some_and(|current| {
                                    *current != el_to_focus
                                        && contains(Some(&floating_focus_element), Some(current))
                                });

                            !focus_moved_inside
                        }
                    })),
                    ..EnqueueFocusOptions::default()
                },
            );
        });
    }

    // -----------------------------------------------------------------------
    // Track return focus targets and restore focus on unmount/close
    // (`:749-902`).
    // -----------------------------------------------------------------------
    {
        let disabled = disabled.clone();
        let floating_signal = floating_signal.clone();
        let floating_focus_element_signal = floating_focus_element_signal.clone();
        let dom_reference_signal = dom_reference_signal.clone();
        let return_focus = return_focus.clone();
        let tree = tree.clone();
        let events = events.clone();
        let get_node_id = Rc::clone(&get_node_id);
        let get_resolved_inside_elements = Rc::clone(&get_resolved_inside_elements);
        let close_type = Rc::clone(&close_type);
        let last_interaction_type = Rc::clone(&last_interaction_type);
        let prevent_return_focus = Rc::clone(&prevent_return_focus);
        use_iso_layout_effect(move || {
            // `if (disabled || !floatingFocusElement) return undefined` (`:750-752`).
            let disabled_value = disabled.get();
            if disabled_value {
                return;
            }
            let Some(floating_focus_element) = floating_focus_element_signal.get() else {
                return;
            };
            let floating = floating_signal.get();

            let doc = owner_document(Some(floating_focus_element.as_ref()));
            let element_focused_before_open = active_element(&doc);
            // Only an explicit `null` interaction type represents a programmatic open
            // (`:756-759`).
            let prefer_previous_focus = open_interaction_type.is_none();

            add_previously_focused_element(element_focused_before_open.as_ref());

            // The listener takes its own clones — the effect callback must stay `FnMut`
            // (the listener is re-registered per effect run).
            let close_type_for_listener = Rc::clone(&close_type);
            let last_interaction_type_for_listener = Rc::clone(&last_interaction_type);
            let prevent_return_focus_for_listener = Rc::clone(&prevent_return_focus);
            let floating_focus_element_for_listener = floating_focus_element.clone();

            // `onOpenChangeLocal` (`:763-805`).
            let unsubscribe = events.on(
                "openchange",
                Rc::new(move |details: &FloatingUIOpenChangeDetails| {
                    if !details.open {
                        close_type_for_listener.set(get_event_type(
                            &details.native_event,
                            last_interaction_type_for_listener.get(),
                        ));
                    }

                    if details.reason == reasons::TRIGGER_HOVER
                        && details.native_event.type_() == "mouseleave"
                    {
                        prevent_return_focus_for_listener.set(true);
                    }

                    if details.reason != reasons::OUTSIDE_PRESS {
                        return;
                    }

                    if details.nested {
                        prevent_return_focus_for_listener.set(false);
                    } else {
                        let virtual_event = details
                            .native_event
                            .dyn_ref::<MouseEvent>()
                            .map(is_virtual_click)
                            .unwrap_or(false)
                            || details
                                .native_event
                                .dyn_ref::<PointerEvent>()
                                .map(is_virtual_pointer_event)
                                .unwrap_or(false);
                        if virtual_event {
                            prevent_return_focus_for_listener.set(false);
                        } else {
                            // On outside press, only return focus to the reference when
                            // the browser supports the `focus({ preventScroll })` option;
                            // without it, restoring focus scrolls the page (`:784-803`).
                            let supported = probe_prevent_scroll_support(&owner_document(Some(
                                floating_focus_element_for_listener.as_ref(),
                            )));
                            prevent_return_focus_for_listener.set(!supported);
                        }
                    }
                }),
            );

            let dom_reference = dom_reference_signal.get();

            let unsubscribe = SendWrapper::new(unsubscribe);
            let doc_for_cleanup = SendWrapper::new(doc);
            let floating_for_cleanup = SendWrapper::new(floating);
            let dom_reference_for_cleanup = SendWrapper::new(dom_reference);
            let element_focused_before_open_for_cleanup =
                SendWrapper::new(element_focused_before_open);
            let return_focus_for_cleanup = SendWrapper::new(return_focus.clone());
            let tree_for_cleanup = tree.clone();
            let get_node_id_for_cleanup = SendWrapper::new(Rc::clone(&get_node_id));
            let get_resolved_inside_elements_for_cleanup =
                SendWrapper::new(Rc::clone(&get_resolved_inside_elements));
            let close_type_for_cleanup = SendWrapper::new(Rc::clone(&close_type));
            let prevent_return_focus_for_cleanup =
                SendWrapper::new(Rc::clone(&prevent_return_focus));
            on_cleanup(move || {
                // `events.off('openchange', onOpenChangeLocal)` (`:847`).
                unsubscribe.take()();

                let doc = doc_for_cleanup;
                let active_el = active_element(&doc);
                let inside_elements = get_resolved_inside_elements_for_cleanup.take()();
                let floating = floating_for_cleanup.take();
                let dom_reference = dom_reference_for_cleanup.take();
                let element_focused_before_open = element_focused_before_open_for_cleanup.take();

                // `isFocusInsideFloatingTree` (`:851-857`).
                let is_focus_inside_floating_tree = contains(floating.as_ref(), active_el.as_ref())
                    || inside_elements.iter().any(|element| {
                        active_el.as_ref().is_some_and(|active| active == element)
                            || contains(Some(element), active_el.as_ref())
                    })
                    || tree_for_cleanup.as_ref().is_some_and(|tree| {
                        let nodes = tree.nodes.borrow();
                        get_node_children(&nodes, get_node_id_for_cleanup().as_deref(), false)
                            .iter()
                            .any(|node| {
                                let floating_of_node = node
                                    .context
                                    .borrow()
                                    .as_ref()
                                    .map(|context| context.elements.floating.get_untracked())
                                    .unwrap_or(None);
                                contains(floating_of_node.as_ref(), active_el.as_ref())
                            })
                    });

                let return_element = get_return_element(
                    &return_focus_for_cleanup,
                    close_type_for_cleanup.get(),
                    dom_reference.as_ref(),
                    element_focused_before_open.as_ref(),
                    prefer_previous_focus,
                );

                let return_focus_for_microtask = (*return_focus_for_cleanup).clone();
                let close_type_for_microtask = Rc::clone(&close_type_for_cleanup);
                let prevent_return_focus_for_microtask =
                    Rc::clone(&prevent_return_focus_for_cleanup);
                queue_microtask(move || {
                    // `returnElement` if it is tabbable, otherwise its first tabbable
                    // child, otherwise `returnElement` itself (`:867`).
                    let tabbable_return_element =
                        get_first_tabbable_element(return_element.as_ref());
                    let has_explicit_return_focus =
                        !matches!(return_focus_for_microtask, ReturnFocus::Bool(_));
                    let return_focus_enabled =
                        !matches!(return_focus_for_microtask, ReturnFocus::Bool(false));

                    if return_focus_enabled
                        && !prevent_return_focus_for_microtask.get()
                        && tabbable_return_element.is_some()
                        && tabbable_return_element
                            .as_ref()
                            .is_some_and(|element| element.dyn_ref::<HtmlElement>().is_some())
                    {
                        let tabbable_return_element =
                            tabbable_return_element.expect("checked above");
                        // If the focus moved somewhere else after mount, avoid returning
                        // focus since it likely entered a different element which should
                        // be respected (`:876-879`).
                        let body_element: Option<Element> =
                            doc.body().map(|body| body.unchecked_into());
                        let condition = if !has_explicit_return_focus
                            && Some(&tabbable_return_element) != active_el.as_ref()
                            && active_el != body_element
                        {
                            is_focus_inside_floating_tree
                        } else {
                            true
                        };

                        if condition {
                            focus_with_options(
                                &tabbable_return_element,
                                true,
                                close_type_for_microtask.get() == InteractionType::Keyboard,
                            );
                        }
                    }

                    prevent_return_focus_for_microtask.set(false);
                });
            });
        });
    }

    // -----------------------------------------------------------------------
    // Safari may randomly scroll to the bottom of the page if an input inside a
    // popup has focus when the popup unmounts (`:907-920`).
    // -----------------------------------------------------------------------
    {
        let open_signal = open_signal.clone();
        let floating_signal = floating_signal.clone();
        use_iso_layout_effect(move || {
            let open_value = open_signal.get();
            let floating = floating_signal.get();
            // `if (!platform.engine.webkit || open || !floating) return` (`:908-910`).
            if !platform().engine.webkit || open_value {
                return;
            }
            let Some(floating) = floating else {
                return;
            };

            let doc = owner_document(Some(floating.as_ref()));
            let Some(active_html) =
                active_element(&doc).and_then(|element| element.dyn_into::<HtmlElement>().ok())
            else {
                return;
            };
            if !is_typeable_element(active_html.as_ref()) {
                return;
            }

            if contains(Some(&floating), Some(active_html.as_ref())) {
                let _ = active_html.blur();
            }
        });
    }

    // -----------------------------------------------------------------------
    // Synchronize the focus manager state to the FloatingPortal context, which
    // uses it to decide whether to render its own guards (`:924-940`).
    // -----------------------------------------------------------------------
    {
        let disabled = disabled.clone();
        let modal = modal.clone();
        let open_signal = open_signal.clone();
        let close_on_focus_out = close_on_focus_out.clone();
        let dom_reference_signal = dom_reference_signal.clone();
        let portal_context = portal_context.clone();
        let store_for_state = Rc::clone(&store);
        let on_open_change: OnOpenChangeFn =
            Rc::new(move |open: bool, details: &RootOpenChangeEventDetails| {
                store_for_state.set_open(open, details);
            });
        use_iso_layout_effect(move || {
            // `if (disabled || !portalContext) return undefined` (`:925-927`).
            let disabled_value = disabled.get();
            let Some(portal) = portal_context.as_ref() else {
                return;
            };
            if disabled_value {
                return;
            }

            portal.focus_manager_state.set(Some(FocusManagerState {
                modal: modal.get(),
                open: open_signal.get(),
                on_open_change: on_open_change.clone(),
                dom_reference: dom_reference_signal.get(),
                close_on_focus_out: close_on_focus_out.get(),
            }));

            let portal = portal.clone();
            on_cleanup(move || {
                portal.focus_manager_state.set(None);
            });
        });
    }

    // -----------------------------------------------------------------------
    // Keep the floating element tabIndex in sync and clear stale focus records
    // (`:943-951`).
    // -----------------------------------------------------------------------
    {
        let disabled = disabled.clone();
        let floating_focus_element_signal = floating_focus_element_signal.clone();
        use_iso_layout_effect(move || {
            let disabled_value = disabled.get();
            if disabled_value {
                return;
            }
            let Some(floating_focus_html) = floating_focus_element_signal
                .get()
                .and_then(|element| element.dyn_into::<HtmlElement>().ok())
            else {
                return;
            };
            handle_tab_index(&floating_focus_html);
            on_cleanup(move || {
                queue_microtask(clear_disconnected_previously_focused_elements);
            });
        });
    }

    // -----------------------------------------------------------------------
    // The inside guards (`:953-1003`), view-free — realized while
    // `shouldRenderGuards` holds, removed when it stops; the consumer places them
    // flanking the floating element (see the module docs).
    // -----------------------------------------------------------------------
    let should_render_guards_signal: Signal<bool, LocalStorage> = Signal::derive_local({
        let disabled = disabled.clone();
        let modal = modal.clone();
        let is_untrapped_typeable_combobox = is_untrapped_typeable_combobox.clone();
        move || {
            should_render_guards(
                disabled.get(),
                modal.get(),
                is_untrapped_typeable_combobox.get(),
                is_inside_portal,
            )
        }
    });

    let handle = FloatingFocusManagerHandle {
        before_inside_guard: RealizedElement::default(),
        after_inside_guard: RealizedElement::default(),
    };

    {
        let should_render_guards_signal = should_render_guards_signal.clone();
        let portal_context = portal_context.clone();
        let modal = modal.clone();
        let close_on_focus_out = close_on_focus_out.clone();
        let dom_reference_signal = dom_reference_signal.clone();
        let floating_focus_element_signal = floating_focus_element_signal.clone();
        let prevent_return_focus = Rc::clone(&prevent_return_focus);
        let next_focusable_element = next_focusable_element.clone();
        let previous_focusable_element = previous_focusable_element.clone();
        let before_content_focus_guard_ref = before_content_focus_guard_ref.clone();
        let before_inside_guard = handle.before_inside_guard.clone();
        let after_inside_guard = handle.after_inside_guard.clone();
        use_iso_layout_effect(move || {
            let should = should_render_guards_signal.get();
            if !should {
                before_inside_guard.clear();
                after_inside_guard.clear();
                *before_guard_ref.borrow_mut() = None;
                *after_guard_ref.borrow_mut() = None;
                if let Some(portal) = portal_context.as_ref() {
                    *portal.before_inside_ref.borrow_mut() = None;
                    *portal.after_inside_ref.borrow_mut() = None;
                }
                if let Some(caller_ref) = before_content_focus_guard_ref.as_ref() {
                    *caller_ref.borrow_mut() = None;
                }
                return;
            }

            if before_inside_guard.get().is_none() {
                // `<FocusGuard data-type="inside" ref={mergedBeforeGuardRef}
                // onFocus={...}>` (`:959-977`).
                let guard = create_focus_guard(FocusGuardProps {
                    attributes: vec![("data-type".to_owned(), "inside".to_owned())],
                });
                let focus_handler = before_inside_guard_focus_handler(
                    modal.clone(),
                    floating_focus_element_signal.clone(),
                    portal_context.clone(),
                    Rc::clone(&prevent_return_focus),
                    Signal::from(dom_reference_signal.clone()),
                    previous_focusable_element.clone(),
                );
                let listener = leptos_ui_utils::add_event_listener(&guard, "focus", focus_handler);
                *before_guard_ref.borrow_mut() = Some(guard.clone());
                if let Some(portal) = portal_context.as_ref() {
                    *portal.before_inside_ref.borrow_mut() = Some(guard.clone());
                }
                if let Some(caller_ref) = before_content_focus_guard_ref.as_ref() {
                    *caller_ref.borrow_mut() = Some(guard.clone());
                }
                before_inside_guard.set(guard, Some(listener));
            }

            if after_inside_guard.get().is_none() {
                // `<FocusGuard data-type="inside" ref={mergedAfterGuardRef}
                // onFocus={...}>` (`:981-1001`).
                let guard = create_focus_guard(FocusGuardProps {
                    attributes: vec![("data-type".to_owned(), "inside".to_owned())],
                });
                let focus_handler = after_inside_guard_focus_handler(
                    modal.clone(),
                    close_on_focus_out.clone(),
                    floating_focus_element_signal.clone(),
                    portal_context.clone(),
                    Rc::clone(&prevent_return_focus),
                    Signal::from(dom_reference_signal.clone()),
                    next_focusable_element.clone(),
                );
                let listener = leptos_ui_utils::add_event_listener(&guard, "focus", focus_handler);
                *after_guard_ref.borrow_mut() = Some(guard.clone());
                if let Some(portal) = portal_context.as_ref() {
                    *portal.after_inside_ref.borrow_mut() = Some(guard.clone());
                }
                after_inside_guard.set(guard, Some(listener));
            }
        });
    }

    handle
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // `shouldRenderGuards` (`FloatingFocusManager.tsx:953-954`): not disabled, modal
    // managers require a trapped (non-untrapped-combobox) reference, and the guards
    // render inside a portal or when modal.
    #[test]
    fn should_render_guards_follows_the_disabled_modal_and_portal_matrix() {
        // A plain modal manager renders its guards outside a portal.
        assert!(should_render_guards(false, true, false, false));
        // Disabled managers render nothing.
        assert!(!should_render_guards(true, true, false, false));
        // The untrapped typeable combobox suppresses modal guards (`:280-283`).
        assert!(!should_render_guards(false, true, true, false));
        // Non-modal managers only render guards inside a portal (`:954`).
        assert!(!should_render_guards(false, false, false, false));
        assert!(should_render_guards(false, false, false, true));
        // Non-modal managers are unaffected by the combobox arm.
        assert!(should_render_guards(false, false, true, true));
    }

    // The default options (`FloatingFocusManagerProps` defaults, `:163-261`).
    #[test]
    fn default_options_match_upstream() {
        let defaults = FloatingFocusManagerOptions::default();
        assert!(!defaults.disabled.get_untracked());
        assert!(defaults.modal.get_untracked());
        assert!(defaults.close_on_focus_out.get_untracked());
        assert_eq!(defaults.restore_focus.get_untracked(), RestoreFocus::False);
        assert_eq!(
            defaults.open_interaction_type,
            Some(InteractionType::Unknown),
            "the `''` default interaction type"
        );
        assert!(matches!(defaults.initial_focus, InitialFocus::Bool(true)));
        assert!(matches!(defaults.return_focus, ReturnFocus::Bool(true)));
    }
}

// The focus machinery runs through the ported `use_iso_layout_effect`, whose browser
// binding only exists under the wasm/browser target — every behavioral pin therefore runs
// in the browser, like the crate's other effect-driven suites.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use std::ops::Deref;

    use wasm_bindgen_test::wasm_bindgen_test;

    use reactive_graph::owner::Owner;

    use reactive_graph::signal::RwSignal;

    use crate::floating_ui::floating_root_store::FloatingRootStore;
    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::types::ReferenceType;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window()
            .expect("realm")
            .document()
            .expect("document")
    }

    fn body() -> Element {
        document()
            .body()
            .expect("body should exist")
            .unchecked_into()
    }

    fn create_element(tag: &str) -> Element {
        document()
            .create_element(tag)
            .unwrap_or_else(|_| panic!("{tag} should be creatable"))
    }

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    /// Drains the executor so the tracked effects re-run after their inputs changed —
    /// the first `RenderEffect` run is synchronous, re-runs execute on an executor poll
    /// (the `floating_delay_group` harness note).
    fn flush() {
        for _ in 0..8 {
            any_spawner::Executor::poll_local();
        }
    }

    async fn sleep(ms: i32) {
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap();
        });
        wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
    }

    /// Waits out the open/close focus round-trips: the openchange dispatch + microtask +
    /// the `enqueueFocus` animation frame.
    async fn settle() {
        flush();
        sleep(60).await;
        flush();
    }

    fn current_active() -> Option<Element> {
        let doc = document();
        active_element(&doc)
    }

    fn assert_focused(test_id: &str) {
        let active = current_active().expect("something is focused");
        assert_eq!(
            active.get_attribute("data-testid").as_deref(),
            Some(test_id),
            "the focused element should be {test_id}"
        );
    }

    type OpenLog = Rc<RefCell<Vec<(bool, String)>>>;

    /// The upstream `App` fixture (`FloatingFocusManager.test.tsx:55-101`): a reference
    /// button, a role="dialog" floating element carrying buttons `one`/`two`/`three`,
    /// and an outside tabbable `last` — wired to a real store whose `onOpenChange`
    /// flips the state, exactly the consumer loop.
    struct Harness {
        owner: Option<Owner>,
        store: Rc<FloatingRootStore>,
        reference: HtmlElement,
        floating: HtmlElement,
        one: HtmlElement,
        two: HtmlElement,
        three: HtmlElement,
        last: HtmlElement,
        open_log: OpenLog,
        handle: FloatingFocusManagerHandle,
        _elements: Vec<Element>,
    }

    impl Harness {
        fn new(modal: bool) -> Self {
            Self::with(|options| FloatingFocusManagerOptions {
                modal: Signal::derive_local(move || modal),
                ..options
            })
        }

        fn with(
            tweak: impl FnOnce(FloatingFocusManagerOptions) -> FloatingFocusManagerOptions,
        ) -> Self {
            init_executor();
            let mut elements: Vec<Element> = Vec::new();

            let reference: HtmlElement = create_element("button").unchecked_into();
            reference
                .set_attribute("data-testid", "reference")
                .expect("attribute name is valid");
            body()
                .append_child(&reference)
                .expect("append should succeed");
            elements.push(reference.clone().unchecked_into());

            let floating: HtmlElement = create_element("div").unchecked_into();
            floating
                .set_attribute("role", "dialog")
                .expect("attribute name is valid");
            floating
                .set_attribute("data-testid", "floating")
                .expect("attribute name is valid");
            body()
                .append_child(&floating)
                .expect("append should succeed");
            elements.push(floating.clone().unchecked_into());

            let one: HtmlElement = create_element("button").unchecked_into();
            one.set_attribute("data-testid", "one")
                .expect("attribute name is valid");
            floating.append_child(&one).expect("append should succeed");

            let two: HtmlElement = create_element("button").unchecked_into();
            two.set_attribute("data-testid", "two")
                .expect("attribute name is valid");
            floating.append_child(&two).expect("append should succeed");

            let three: HtmlElement = create_element("button").unchecked_into();
            three
                .set_attribute("data-testid", "three")
                .expect("attribute name is valid");
            floating
                .append_child(&three)
                .expect("append should succeed");

            let last: HtmlElement = create_element("div").unchecked_into();
            last.set_attribute("data-testid", "last")
                .expect("attribute name is valid");
            last.set_attribute("tabindex", "0")
                .expect("attribute name is valid");
            body().append_child(&last).expect("append should succeed");
            elements.push(last.clone().unchecked_into());

            let open_log: OpenLog = Rc::new(RefCell::new(Vec::new()));

            let owner = Owner::new();
            let (store, handle) = owner.with(|| {
                let store = FloatingRootStore::new(FloatingRootStoreOptions {
                    open: false,
                    transition_status: None,
                    reference_element: None,
                    floating_element: None,
                    trigger_elements: PopupTriggerMap::new(),
                    floating_id: None,
                    sync_only: false,
                    nested: false,
                    on_open_change: None,
                });

                {
                    let store = Rc::clone(&store);
                    let reference_for_store = reference.clone();
                    let floating_for_store = floating.clone();
                    store.update(|state, _| {
                        state.reference_element =
                            Some(ReferenceType::Element(reference_for_store.clone().into()));
                        state.dom_reference_element = Some(reference_for_store.into());
                        state.floating_element = Some(floating_for_store.into());
                        true
                    });
                }

                let open_log_for_callback = Rc::clone(&open_log);
                let store_for_callback = Rc::clone(&store);
                store.context.set_on_open_change(Some(Rc::new(
                    move |open: bool, details: &RootOpenChangeEventDetails| {
                        open_log_for_callback
                            .borrow_mut()
                            .push((open, details.reason.clone()));
                        // The consumer loop: flip the controlled open state.
                        store_for_callback.update(|state, _| {
                            state.open = open;
                            true
                        });
                    },
                )));

                let handle = provide_floating_focus_manager(tweak(FloatingFocusManagerOptions {
                    context: FloatingContextSource::Store(Rc::clone(&store)),
                    ..FloatingFocusManagerOptions::default()
                }));

                (store, handle)
            });
            flush();

            Self {
                owner: Some(owner),
                store,
                reference,
                floating,
                one,
                two,
                three,
                last,
                open_log,
                handle,
                _elements: elements,
            }
        }

        /// `{open && <FloatingFocusManager>}`: closing unmounts the manager, running its
        /// cleanup — the return-focus path.
        fn unmount(&mut self) {
            self.owner.take();
        }

        fn open(&self) {
            self.store.update(|state, _| {
                state.open = true;
                true
            });
        }

        fn close(&self, reason: &str, event: &Event) {
            self.store.set_open(
                false,
                &RootOpenChangeEventDetails::new(reason, event.clone(), None, String::new()),
            );
        }

        fn focus(&self, element: &HtmlElement) {
            let _ = element.focus();
        }
    }

    impl Drop for Harness {
        fn drop(&mut self) {
            self.owner.take();
            for element in self._elements.drain(..) {
                let _ = element.remove();
            }
        }
    }

    // Default transitions on open (`:188-195`): focus moves into the floating element,
    // to the first tabbable.
    #[wasm_bindgen_test(async)]
    async fn focuses_the_first_tabbable_element_on_open_by_default() {
        let harness = Harness::new(true);

        harness.open();
        settle().await;

        assert_focused("one");
    }

    // Initial focus, named radio group (`:197-204`): the checked radio wins.
    #[wasm_bindgen_test(async)]
    async fn focuses_the_checked_radio_in_a_named_radio_group() {
        init_executor();
        let mut elements: Vec<Element> = Vec::new();

        let reference: HtmlElement = create_element("button").unchecked_into();
        reference
            .set_attribute("data-testid", "reference")
            .expect("attribute name is valid");
        body().append_child(&reference).expect("append");
        elements.push(reference.clone().unchecked_into());

        let floating: HtmlElement = create_element("div").unchecked_into();
        floating
            .set_attribute("role", "dialog")
            .expect("attribute name is valid");
        body().append_child(&floating).expect("append");
        elements.push(floating.clone().unchecked_into());

        let radio_one: HtmlElement = create_element("input").unchecked_into();
        radio_one
            .set_attribute("type", "radio")
            .expect("attribute name is valid");
        radio_one
            .set_attribute("name", "group")
            .expect("attribute name is valid");
        radio_one
            .set_attribute("data-testid", "radio-one")
            .expect("attribute name is valid");
        floating.append_child(&radio_one).expect("append");

        let radio_two: HtmlElement = create_element("input").unchecked_into();
        radio_two
            .set_attribute("type", "radio")
            .expect("attribute name is valid");
        radio_two
            .set_attribute("name", "group")
            .expect("attribute name is valid");
        radio_two
            .set_attribute("checked", "")
            .expect("attribute name is valid");
        radio_two
            .set_attribute("data-testid", "radio-two")
            .expect("attribute name is valid");
        floating.append_child(&radio_two).expect("append");

        let owner = Owner::new();
        owner.with(|| {
            let store = FloatingRootStore::new(FloatingRootStoreOptions {
                open: false,
                transition_status: None,
                reference_element: Some(ReferenceType::Element(reference.clone().into())),
                floating_element: Some(floating.clone().into()),
                trigger_elements: PopupTriggerMap::new(),
                floating_id: None,
                sync_only: false,
                nested: false,
                on_open_change: None,
            });
            {
                let reference = reference.clone();
                store.update(|state, _| {
                    state.dom_reference_element = Some(reference.into());
                    true
                });
            }
            let _handle = provide_floating_focus_manager(FloatingFocusManagerOptions {
                context: FloatingContextSource::Store(Rc::clone(&store)),
                ..FloatingFocusManagerOptions::default()
            });

            store.update(|state, _| {
                state.open = true;
                true
            });
        });

        settle().await;
        assert_focused("radio-two");
        drop(owner);

        for element in elements.drain(..) {
            let _ = element.remove();
        }
    }

    // `initialFocus` as ref (`:206-212`): that element receives focus.
    #[wasm_bindgen_test(async)]
    async fn initial_focus_ref_targets_that_element() {
        let two_ref: SharedFocusRef = Rc::new(RefCell::new(None));
        let harness = Harness::with(|options| FloatingFocusManagerOptions {
            initial_focus: InitialFocus::Ref(Rc::clone(&two_ref)),
            ..options
        });
        *two_ref.borrow_mut() = Some(harness.two.clone());

        harness.open();
        settle().await;

        assert_focused("two");
    }

    // `initialFocus={false}` (`:2165`): focus is not moved into the floating element.
    #[wasm_bindgen_test(async)]
    async fn initial_focus_false_leaves_focus_on_the_reference() {
        let harness = Harness::with(|options| FloatingFocusManagerOptions {
            initial_focus: InitialFocus::Bool(false),
            ..options
        });
        harness.focus(&harness.reference);

        harness.open();
        settle().await;

        assert_focused("reference");
    }

    // The `initialFocus` function form returning "do nothing" (`:170-172`).
    #[wasm_bindgen_test(async)]
    async fn initial_focus_function_do_nothing_leaves_focus() {
        let harness = Harness::with(|options| FloatingFocusManagerOptions {
            initial_focus: InitialFocus::Fn(Rc::new(|_open_type| ResolvedFocusTarget::Nothing)),
            ..options
        });
        harness.focus(&harness.reference);

        harness.open();
        settle().await;

        assert_focused("reference");
    }

    // Default transitions on close (`:236-243`, `:556-606`): focus returns to the
    // reference.
    #[wasm_bindgen_test(async)]
    async fn return_focus_returns_to_the_reference_on_close() {
        let mut harness = Harness::new(true);

        harness.open();
        settle().await;
        assert_focused("one");

        harness.close(reasons::TRIGGER_PRESS, &Event::new("keydown").unwrap());
        flush();
        harness.unmount();
        settle().await;

        assert_focused("reference");
        assert_eq!(
            harness
                .open_log
                .borrow()
                .deref()
                .last()
                .map(|(open, _)| *open),
            Some(false),
        );
    }

    // `returnFocus={false}` (`:246-257`): focus is left where it is.
    #[wasm_bindgen_test(async)]
    async fn return_focus_false_leaves_focus_where_it_is() {
        let mut harness = Harness::with(|options| FloatingFocusManagerOptions {
            return_focus: ReturnFocus::Bool(false),
            ..options
        });

        harness.open();
        settle().await;
        assert_focused("one");

        harness.close(reasons::TRIGGER_PRESS, &Event::new("keydown").unwrap());
        flush();
        harness.unmount();
        settle().await;

        assert_focused("one");
    }

    // `returnFocus={ref}` (`:259-280`): focus goes to the referenced element.
    #[wasm_bindgen_test(async)]
    async fn return_focus_ref_targets_that_element() {
        let return_ref: SharedFocusRef = Rc::new(RefCell::new(None));
        let mut harness = Harness::with(|options| FloatingFocusManagerOptions {
            return_focus: ReturnFocus::Ref(Rc::clone(&return_ref)),
            ..options
        });
        *return_ref.borrow_mut() = Some(harness.last.clone());

        harness.open();
        settle().await;

        harness.close(reasons::TRIGGER_PRESS, &Event::new("keydown").unwrap());
        flush();
        harness.unmount();
        settle().await;

        assert_focused("last");
    }

    // Non-focusable reference (`:330-348`): focus returns to the first focusable
    // descendant of the reference.
    #[wasm_bindgen_test(async)]
    async fn non_focusable_reference_returns_to_its_first_tabbable_descendant() {
        init_executor();
        let mut elements: Vec<Element> = Vec::new();

        let reference: HtmlElement = create_element("div").unchecked_into();
        reference
            .set_attribute("data-testid", "reference")
            .expect("attribute name is valid");
        body().append_child(&reference).expect("append");
        elements.push(reference.clone().unchecked_into());

        let child: HtmlElement = create_element("button").unchecked_into();
        child
            .set_attribute("data-testid", "reference-child")
            .expect("attribute name is valid");
        reference.append_child(&child).expect("append");

        let floating: HtmlElement = create_element("div").unchecked_into();
        floating
            .set_attribute("role", "dialog")
            .expect("attribute name is valid");
        let one: HtmlElement = create_element("button").unchecked_into();
        one.set_attribute("data-testid", "one")
            .expect("attribute name is valid");
        floating.append_child(&one).expect("append");
        body().append_child(&floating).expect("append");
        elements.push(floating.clone().unchecked_into());

        let owner = Owner::new();
        let store = owner.with(|| {
            let store = FloatingRootStore::new(FloatingRootStoreOptions {
                open: false,
                transition_status: None,
                reference_element: Some(ReferenceType::Element(reference.clone().into())),
                floating_element: Some(floating.clone().into()),
                trigger_elements: PopupTriggerMap::new(),
                floating_id: None,
                sync_only: false,
                nested: false,
                on_open_change: None,
            });
            {
                let reference = reference.clone();
                store.update(|state, _| {
                    state.dom_reference_element = Some(reference.into());
                    true
                });
            }
            let store_for_callback = Rc::clone(&store);
            store.context.set_on_open_change(Some(Rc::new(
                move |open: bool, _details: &RootOpenChangeEventDetails| {
                    store_for_callback.update(|state, _| {
                        state.open = open;
                        true
                    });
                },
            )));

            let _handle = provide_floating_focus_manager(FloatingFocusManagerOptions {
                context: FloatingContextSource::Store(Rc::clone(&store)),
                ..FloatingFocusManagerOptions::default()
            });

            store.update(|state, _| {
                state.open = true;
                true
            });
            store
        });

        settle().await;
        assert_focused("one");

        store.set_open(
            false,
            &RootOpenChangeEventDetails::new(
                reasons::TRIGGER_PRESS,
                Event::new("keydown").unwrap(),
                None,
                String::new(),
            ),
        );
        flush();
        // The manager unmounts with the closed popup.
        drop(owner);
        settle().await;

        assert_focused("reference-child");
        for element in elements.drain(..) {
            let _ = element.remove();
        }
    }

    // Close modality (`:1519`): an Escape-keyboard close reports `'keyboard'` to the
    // `returnFocus` function form.
    #[wasm_bindgen_test(async)]
    async fn return_focus_fn_receives_the_keyboard_close_type() {
        let close_types: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let close_types_for_fn = Rc::clone(&close_types);
        let mut harness = Harness::with(|options| FloatingFocusManagerOptions {
            return_focus: ReturnFocus::Fn(Rc::new(move |close_type| {
                close_types_for_fn
                    .borrow_mut()
                    .push(format!("{close_type:?}"));
                ResolvedFocusTarget::Default
            })),
            ..options
        });

        harness.open();
        settle().await;

        harness.close(reasons::ESCAPE_KEY, &KeyboardEvent::new("keydown").unwrap());
        flush();
        harness.unmount();
        settle().await;

        assert_eq!(
            close_types.borrow().deref().last(),
            Some(&"Keyboard".to_owned()),
            "an Escape close reports the keyboard modality (the mount pass's never-opened \
             session reports the empty member first — the store-mirror seeding note)"
        );
        assert_focused("reference");
    }

    // Close modality (`:1555`): a programmatic close carries no native event, so the
    // terminal fallthrough (`:69`) reports `''` (Unknown). The port's analog of
    // upstream's `setOpen(false)` with no event is a plain (non-UI) Event.
    #[wasm_bindgen_test(async)]
    async fn return_focus_fn_receives_the_empty_type_for_a_programmatic_close() {
        let close_types: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let close_types_for_fn = Rc::clone(&close_types);
        let mut harness = Harness::with(|options| FloatingFocusManagerOptions {
            return_focus: ReturnFocus::Fn(Rc::new(move |close_type| {
                close_types_for_fn
                    .borrow_mut()
                    .push(format!("{close_type:?}"));
                ResolvedFocusTarget::Default
            })),
            ..options
        });

        harness.open();
        settle().await;

        harness.close(reasons::OUTSIDE_PRESS, &Event::new("programmatic").unwrap());
        flush();
        harness.unmount();
        settle().await;

        assert_eq!(
            close_types.borrow().deref().last(),
            Some(&"Unknown".to_owned()),
            "a programmatic close reports the empty member"
        );
    }

    // Tab (modal=true, `:849-878`): the after-inside guard wraps focus to the first
    // tabbable inside.
    #[wasm_bindgen_test(async)]
    async fn modal_tab_trap_wraps_forward_through_the_after_guard() {
        let harness = Harness::new(true);

        // Place the realized guards flanking the floating element (the consumer's job in
        // the view-free port).
        let after_guard = harness
            .handle
            .after_inside_guard
            .get()
            .expect("guard realized");
        body()
            .insert_before(&after_guard, Some(&harness.last))
            .expect("guard inserted");

        harness.open();
        settle().await;
        assert_focused("one");

        // Tab from the last inside element: the browser moves focus to the after guard.
        harness.focus(&after_guard);
        settle().await;

        assert_focused("one");
    }

    // Tab (modal=false, `:880-901`): Tab out closes the floating element through the
    // focus-out path.
    #[wasm_bindgen_test(async)]
    async fn non_modal_tab_out_closes_with_the_focus_out_reason() {
        let harness = Harness::new(false);

        harness.open();
        settle().await;
        assert_focused("one");

        // Tab out: focus lands on the next outside element.
        harness.focus(&harness.last);
        settle().await;

        let log = harness.open_log.borrow().deref().clone();
        assert_eq!(
            log.last().map(|(open, reason)| (*open, reason.as_str())),
            Some((false, reasons::FOCUS_OUT)),
            "the focus-out close flows through onOpenChange"
        );
    }

    // modal=true hides outside content (`:1139-1185`): outside elements get
    // `aria-hidden="true"`, the floating element does not, and close removes them.
    #[wasm_bindgen_test(async)]
    async fn modal_hides_outside_content_with_aria_hidden_and_restores_on_close() {
        let mut harness = Harness::new(true);

        assert!(
            !harness.reference.has_attribute("aria-hidden"),
            "nothing is hidden while closed"
        );

        harness.open();
        settle().await;

        assert_eq!(
            harness.reference.get_attribute("aria-hidden").as_deref(),
            Some("true"),
            "the reference is hidden from AT"
        );
        assert_eq!(
            harness.last.get_attribute("aria-hidden").as_deref(),
            Some("true"),
            "outside elements are hidden from AT"
        );
        assert!(
            !harness.floating.has_attribute("aria-hidden"),
            "the floating element is not hidden"
        );

        harness.close(reasons::TRIGGER_PRESS, &Event::new("keydown").unwrap());
        flush();
        harness.unmount();
        flush();

        assert!(
            !harness.reference.has_attribute("aria-hidden"),
            "the aria-hidden marks are removed on close"
        );
        assert!(!harness.last.has_attribute("aria-hidden"));
    }

    // modal=false (`:1269-1279`): the `data-base-ui-inert` marker lands on the
    // top-level outside ancestor (the reference), not the floating element, and is
    // removed on close.
    #[wasm_bindgen_test(async)]
    async fn non_modal_marks_the_reference_with_the_inert_marker() {
        let mut harness = Harness::new(false);

        harness.open();
        settle().await;

        assert!(
            harness.reference.has_attribute("data-base-ui-inert"),
            "the reference gets the inert marker"
        );
        assert!(
            !harness.floating.has_attribute("data-base-ui-inert"),
            "the floating element is never marked"
        );

        harness.close(reasons::TRIGGER_PRESS, &Event::new("keydown").unwrap());
        flush();
        harness.unmount();
        flush();

        assert!(
            !harness.reference.has_attribute("data-base-ui-inert"),
            "the marker is removed on close"
        );
    }

    // The `disabled` interplay (`:1341-1378`): with `disabled` no focus management
    // happens; flipping to enabled grabs focus.
    #[wasm_bindgen_test(async)]
    async fn disabled_defers_focus_management_until_flipped() {
        init_executor();
        let mut elements: Vec<Element> = Vec::new();

        let reference: HtmlElement = create_element("button").unchecked_into();
        reference
            .set_attribute("data-testid", "reference")
            .expect("attribute name is valid");
        body().append_child(&reference).expect("append");
        elements.push(reference.clone().unchecked_into());

        let floating: HtmlElement = create_element("div").unchecked_into();
        floating
            .set_attribute("role", "dialog")
            .expect("attribute name is valid");
        let one: HtmlElement = create_element("button").unchecked_into();
        one.set_attribute("data-testid", "one")
            .expect("attribute name is valid");
        floating.append_child(&one).expect("append");
        body().append_child(&floating).expect("append");
        elements.push(floating.clone().unchecked_into());

        let disabled: RwSignal<bool, LocalStorage> = RwSignal::new_local(true);

        let owner = Owner::new();
        owner.with(|| {
            let store = FloatingRootStore::new(FloatingRootStoreOptions {
                open: false,
                transition_status: None,
                reference_element: Some(ReferenceType::Element(reference.clone().into())),
                floating_element: Some(floating.clone().into()),
                trigger_elements: PopupTriggerMap::new(),
                floating_id: None,
                sync_only: false,
                nested: false,
                on_open_change: None,
            });
            {
                let reference = reference.clone();
                store.update(|state, _| {
                    state.dom_reference_element = Some(reference.into());
                    true
                });
            }

            let _handle = provide_floating_focus_manager(FloatingFocusManagerOptions {
                context: FloatingContextSource::Store(Rc::clone(&store)),
                disabled: disabled.into(),
                ..FloatingFocusManagerOptions::default()
            });

            store.update(|state, _| {
                state.open = true;
                true
            });
        });

        settle().await;

        let active = current_active();
        assert_ne!(
            active.and_then(|element| element.get_attribute("data-testid")),
            Some("one".to_owned()),
            "a disabled manager does not move focus"
        );

        // The keep-mounted `disabled` flip (`:1341-1378`).
        disabled.set(false);
        settle().await;

        assert_focused("one");
        drop(owner);

        for element in elements.drain(..) {
            let _ = element.remove();
        }
    }

    // Managed tabindex (`:2488-2560`): a dialog without tabbable content gets
    // `tabindex="0"` (mirrored in `data-tabindex`), downgraded to `-1` once content
    // becomes tabbable.
    #[wasm_bindgen_test(async)]
    async fn managed_tabindex_upgrades_and_downgrades_with_content() {
        init_executor();
        let mut elements: Vec<Element> = Vec::new();

        let reference: HtmlElement = create_element("button").unchecked_into();
        reference
            .set_attribute("data-testid", "reference")
            .expect("attribute name is valid");
        body().append_child(&reference).expect("append");
        elements.push(reference.clone().unchecked_into());

        let floating: HtmlElement = create_element("div").unchecked_into();
        floating
            .set_attribute("role", "dialog")
            .expect("attribute name is valid");
        floating
            .set_attribute("data-testid", "floating")
            .expect("attribute name is valid");
        body().append_child(&floating).expect("append");
        elements.push(floating.clone().unchecked_into());

        let owner = Owner::new();
        owner.with(|| {
            let store = FloatingRootStore::new(FloatingRootStoreOptions {
                open: false,
                transition_status: None,
                reference_element: Some(ReferenceType::Element(reference.clone().into())),
                floating_element: Some(floating.clone().into()),
                trigger_elements: PopupTriggerMap::new(),
                floating_id: None,
                sync_only: false,
                nested: false,
                on_open_change: None,
            });
            {
                let reference = reference.clone();
                store.update(|state, _| {
                    state.dom_reference_element = Some(reference.into());
                    true
                });
            }

            let _handle = provide_floating_focus_manager(FloatingFocusManagerOptions {
                context: FloatingContextSource::Store(Rc::clone(&store)),
                ..FloatingFocusManagerOptions::default()
            });

            store.update(|state, _| {
                state.open = true;
                true
            });
        });

        settle().await;

        assert_eq!(
            floating.get_attribute("tabindex").as_deref(),
            Some("0"),
            "no tabbable content upgrades the dialog to tabindex=0"
        );
        assert_eq!(
            floating.get_attribute("data-tabindex").as_deref(),
            Some("0"),
            "the managed write is mirrored"
        );

        // Content becomes tabbable; the next focus-out on the reference re-runs
        // `handleTabIndex` and downgrades (`:2524-2560`).
        let button: HtmlElement = create_element("button").unchecked_into();
        button
            .set_attribute("data-testid", "late")
            .expect("attribute name is valid");
        floating.append_child(&button).expect("append");

        let focus_out = web_sys::FocusEvent::new("focusout").unwrap();
        reference
            .dispatch_event(focus_out.as_ref())
            .expect("dispatch should succeed");
        flush();
        sleep(10).await;
        flush();

        assert_eq!(
            floating.get_attribute("tabindex").as_deref(),
            Some("-1"),
            "tabbable content downgrades the dialog to tabindex=-1"
        );
        assert_eq!(
            floating.get_attribute("data-tabindex").as_deref(),
            Some("-1")
        );
        drop(owner);

        for element in elements.drain(..) {
            let _ = element.remove();
        }
    }

    // `role="listbox"` floating elements are never upgraded (`:2562-2605`).
    #[wasm_bindgen_test(async)]
    async fn listbox_role_is_never_upgraded() {
        init_executor();
        let mut elements: Vec<Element> = Vec::new();

        let reference: HtmlElement = create_element("button").unchecked_into();
        reference
            .set_attribute("data-testid", "reference")
            .expect("attribute name is valid");
        body().append_child(&reference).expect("append");
        elements.push(reference.clone().unchecked_into());

        let floating: HtmlElement = create_element("div").unchecked_into();
        floating
            .set_attribute("role", "listbox")
            .expect("attribute name is valid");
        floating
            .set_attribute("tabindex", "-1")
            .expect("attribute name is valid");
        body().append_child(&floating).expect("append");
        elements.push(floating.clone().unchecked_into());

        let owner = Owner::new();
        owner.with(|| {
            let store = FloatingRootStore::new(FloatingRootStoreOptions {
                open: false,
                transition_status: None,
                reference_element: Some(ReferenceType::Element(reference.clone().into())),
                floating_element: Some(floating.clone().into()),
                trigger_elements: PopupTriggerMap::new(),
                floating_id: None,
                sync_only: false,
                nested: false,
                on_open_change: None,
            });
            {
                let reference = reference.clone();
                store.update(|state, _| {
                    state.dom_reference_element = Some(reference.into());
                    true
                });
            }

            let _handle = provide_floating_focus_manager(FloatingFocusManagerOptions {
                context: FloatingContextSource::Store(Rc::clone(&store)),
                ..FloatingFocusManagerOptions::default()
            });

            store.update(|state, _| {
                state.open = true;
                true
            });
        });

        settle().await;

        assert_eq!(
            floating.get_attribute("tabindex").as_deref(),
            Some("-1"),
            "the listbox role is exempt from the tabindex=0 upgrade"
        );
        assert!(
            floating.get_attribute("data-tabindex").is_none(),
            "no managed write happened"
        );
        drop(owner);

        for element in elements.drain(..) {
            let _ = element.remove();
        }
    }

    // `restoreFocus: true` (`:2003-2023`): if the focused element is removed, focus
    // moves to the nearest (last) tabbable inside the floating element.
    #[wasm_bindgen_test(async)]
    async fn restore_focus_true_restores_to_the_last_tabbable_after_removal() {
        let harness = Harness::with(|options| FloatingFocusManagerOptions {
            restore_focus: Signal::derive_local(|| RestoreFocus::True),
            modal: Signal::derive_local(|| false),
            ..options
        });

        harness.open();
        settle().await;
        assert_focused("one");

        // Focus the last tabbable, then remove it — focus falls to the body.
        harness.focus(&harness.three);
        flush();
        assert_focused("three");

        let _ = harness.three.remove();
        settle().await;

        assert_focused("two");
    }

    // `getEventType` (`:49-70`): the classification matrix over constructed events.
    #[wasm_bindgen_test]
    fn get_event_type_classifies_the_event_matrix() {
        let keyboard = InteractionType::Keyboard;

        let keydown = KeyboardEvent::new("keydown").unwrap();
        assert_eq!(
            get_event_type(keydown.as_ref(), keyboard),
            InteractionType::Keyboard,
        );

        let focus_event = web_sys::FocusEvent::new("focusout").unwrap();
        assert_eq!(
            get_event_type(focus_event.as_ref(), keyboard),
            InteractionType::Keyboard,
            "a focus event defers to the last known interaction"
        );
        assert_eq!(
            get_event_type(focus_event.as_ref(), InteractionType::Unknown),
            InteractionType::Keyboard,
            "an empty last interaction falls back to keyboard"
        );

        let init = web_sys::PointerEventInit::new();
        init.set_pointer_type("mouse");
        let pointer = PointerEvent::new_with_event_init_dict("pointerdown", &init).unwrap();
        assert_eq!(
            get_event_type(pointer.as_ref(), InteractionType::Unknown),
            InteractionType::Mouse,
        );

        let empty_pointer_init = web_sys::PointerEventInit::new();
        empty_pointer_init.set_pointer_type("");
        let empty_pointer =
            PointerEvent::new_with_event_init_dict("pointerdown", &empty_pointer_init).unwrap();
        assert_eq!(
            get_event_type(empty_pointer.as_ref(), InteractionType::Unknown),
            InteractionType::Keyboard,
            "an empty pointerType falls back to keyboard"
        );

        let touch = web_sys::TouchEvent::new("touchend").unwrap();
        assert_eq!(
            get_event_type(touch.as_ref(), InteractionType::Unknown),
            InteractionType::Touch,
        );

        // A `detail: 0` click is keyboard; a real click is mouse.
        let detail_zero_init = web_sys::MouseEventInit::new();
        detail_zero_init.set_detail(0);
        let detail_zero_click =
            MouseEvent::new_with_mouse_event_init_dict("click", &detail_zero_init).unwrap();
        assert_eq!(
            get_event_type(detail_zero_click.as_ref(), InteractionType::Unknown),
            InteractionType::Keyboard,
        );
        let detail_one_init = web_sys::MouseEventInit::new();
        detail_one_init.set_detail(1);
        let detail_one_click =
            MouseEvent::new_with_mouse_event_init_dict("click", &detail_one_init).unwrap();
        assert_eq!(
            get_event_type(detail_one_click.as_ref(), InteractionType::Unknown),
            InteractionType::Mouse,
        );
        // The last tracked interaction wins over the detail check (`:67`).
        assert_eq!(
            get_event_type(detail_one_click.as_ref(), InteractionType::Touch),
            InteractionType::Touch,
        );
    }
}
