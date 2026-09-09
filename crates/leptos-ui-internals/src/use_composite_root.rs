//! Port of `packages/react/src/internals/composite/root/useCompositeRoot.ts` — the
//! composite highlight state machine and keydown filter pipeline
//! (`TODO.md`, item `infra: internals`; the checkpoint sequence recorded in that entry's
//! note). The `CompositeRoot`/`CompositeItem` *view components* stay with the
//! `useRenderElement` checkpoint (they are `useRenderElement` call sites), so this module
//! ports the hook exactly and leaves element rendering to the view layer.
//!
//! Upstream (`useCompositeRoot.ts:74-340`) owns: the `externalHighlightedIndex ??
//! internalHighlightedIndex` highlight state (`:90`, `:100`); the one stable
//! `onHighlightedIndexChange` that snapshots the highlighted element into
//! `highlightedElementRef` and scrolls it into view on request (`:101-108`) — that ref is
//! the identity anchor of the whole reconciliation model (indexes are positions in a
//! shifting array, elements are stable); the `onMapChange` reconciliation pump
//! (`:110-164`: first population adopts an active `data-composite-item-active` item or
//! moves off a disabled index 0; subsequent populations follow the highlighted *element*
//! through index shifts, keep a same-index replacement when eligible, else fall back via
//! `getFallbackIndex`); the late-`disabledIndices` re-validation layout effect
//! (`:166-193`); and the keydown filter pipeline (`:206-317`): key whitelist → modifier
//! gate → native-input yield → target computation (grid delegation or list mode with the
//! RTL forward/backward mapping, loop handling, and the `onLoop` override) → guarded
//! commit with optional `stopPropagation`/`preventDefault`, the scroll-carrying highlight
//! change, and the `queueMicrotask`-deferred `focus()` (deliberately deferred so Floating
//! UI FocusManager's `returnFocus` runs first). The root's own `onFocus` selects the
//! entire value of a focused native input (`:321-328`) — the full-selection state is what
//! the keydown yield-check reads, which is what makes "first arrow key returns control to
//! the textbox" work.
//!
//! Spec: `specs/library/internals/implementation.md` ("Highlight state machine
//! (`useCompositeRoot`)" — the mechanisms above, verified against the source) and
//! `specs/library/internals/behavior.md` ("State model", "Keyboard interactions", "Focus
//! management" — the observable matrix).
//!
//! ## Rust adaptations
//!
//! - `React.useState(0)` / `externalHighlightedIndex ?? internal` (`:90`, `:100`) port to
//!   an internal `RwSignal<i32>` behind a [`Memo`] reading the optional external source —
//!   the `use_composite_list_item.rs` controlled/uncontrolled mapping, so the view layer
//!   binds `tabindex` reactively and consumers track the highlight.
//! - `useStableCallback` wrappers (`:101`, `:110`, `:195`, `:206`) exist upstream so
//!   handler identity never invalidates identity-sensitive consumers. The port keeps the
//!   stable handle for `onHighlightedIndexChange` (the context's callable) and builds
//!   `onKeyDown`/`onMapChange` as single `Rc` closures — one closure per hook call has
//!   exactly the stable identity upstream's trampolines simulate, and `Rc` clones share it.
//! - `highlightedIndex` (`:248`) is read at dispatch time with `get_untracked` — the
//!   render-scope closure read upstream, without tracking a signal inside an event
//!   handler.
//! - The late-`disabledIndices` layout effect (`:166-193`) runs its check once at hook
//!   time with untracked reads; its *reactive* re-runs (upstream's
//!   `[disabledIndices, externalHighlightedIndex, …]` deps) are discharged to
//!   [`UseCompositeRoot::set_disabled_indices`] (the `use_composite_list_item.rs`
//!   `set_params` precedent: Rust closures are not recreated per render, so the prop
//!   change is an explicit store+re-run), plus the view-layer wiring that forwards prop
//!   changes into it. The check body is upstream's verbatim.
//! - The `queueMicrotask` focus deferral (`:313-315`) runs through
//!   `window.queueMicrotask` on wasm (the `floating_focus_manager.rs` precedent); host
//!   builds have no microtask queue and no DOM, so the deferred arm is a no-op there —
//!   focus is only observable in the browser suite.
//! - `direction` arrives as a reactive source ([`Get`]`<Value = TextDirection>`) — the
//!   `useDirection()` memo the `CompositeRoot` view feeds in (`CompositeRoot.tsx:42`);
//!   reads at dispatch/scroll time are untracked (the render-value read upstream).
//! - `target.value.length` (`:232`, `:327`) is JS's UTF-16 code-unit count — the port
//!   uses `encode_utf16().count()`.
//! - The native-input yield's two reads (`target.selectionStart`, `target.selectionEnd`,
//!   `:230-231`) can throw on selection-unsupported input types in some browsers; the
//!   port maps a failed read to the `selectionStart == null` yield arm — the fail-soft
//!   adaptation `crate::composite` documents for `isNativeInput` (which has already
//!   guaranteed a non-null selectionStart for `<input>` targets by the time this reads).
//! - `elementsRef` (`:96`) is the [`CompositeListElementsRef`] the `CompositeList` wiring
//!   shares (upstream: the hook owns it and `CompositeRoot.tsx:86` hands it to
//!   `CompositeList`); `labelsRef` is not part of the root's surface (`CompositeRoot.tsx`
//!   passes none).

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::computed::Memo;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set, WithValue};
use send_wrapper::SendWrapper;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, EventTarget, FocusEvent, HtmlElement, KeyboardEvent};

use leptos_ui_utils::is_element_disabled;
use leptos_ui_utils::shadow_dom::get_target;
use leptos_ui_utils::use_iso_layout_effect::use_iso_layout_effect;
use leptos_ui_utils::use_merged_refs::{InputRef, MergedRefCallback, use_merged_refs};
use leptos_ui_utils::use_ref_with_init::use_ref_with_init;
use leptos_ui_utils::use_stable_callback::{StableCallback, use_stable_callback};

use crate::composite::{
    ACTIVE_COMPOSITE_ITEM, ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, END, HOME, MODIFIER_KEYS,
    ModifierKey, find_non_disabled_list_index, get_max_list_index, get_min_list_index,
    is_composite_key, is_index_out_of_list_bounds, is_list_index_disabled, is_native_input,
    scroll_into_view_if_needed,
};
use crate::composite_grid_navigation::{CompositeGridNavigationState, CompositeGridNavigator};
use crate::composite_list::{CompositeListElementsRef, CompositeListMap};
use crate::direction_context::TextDirection;
use crate::floating_ui::composite::{DisabledIndices, FindNonDisabledListIndexOptions};
use crate::floating_ui::element_props::ElementEventHandler;
use crate::floating_ui::types::Orientation;

/// The user-facing `onLoop` shape (`useCompositeRoot.ts:37-44`): `(event, prevIndex,
/// nextIndex, elementsRef)`, returning the index navigation continues from. A `None`
/// event is the host-target arm of the grid navigator's event split (module docs).
pub type CompositeOnLoop =
    Rc<dyn Fn(Option<&KeyboardEvent>, i32, i32, &CompositeListElementsRef) -> i32>;

/// `UseCompositeRootParameters` (`useCompositeRoot.ts:33-72`). Upstream's destructuring
/// defaults are documented per field; the `CompositeRoot` view layer applies them
/// (`CompositeRoot.tsx:33` defaults `stopEventPropagation = true`;
/// `useCompositeRoot.ts:76-77` default `loopFocus = true` / `orientation = 'both'`).
pub struct UseCompositeRootParams<I, D> {
    /// `orientation` (`:34`) — upstream default `'both'`; [`None`] is the `undefined`
    /// pass-through from the component layer.
    pub orientation: Option<Orientation>,
    /// `grid` (`:35`) — the [`crate::composite_grid_navigation::grid_navigation`] builder
    /// output; [`None`] is list mode.
    pub grid: Option<CompositeGridNavigator>,
    /// `loopFocus` (`:36`) — upstream default `true`.
    pub loop_focus: bool,
    /// `onLoop` (`:37-44`).
    pub on_loop: Option<CompositeOnLoop>,
    /// `highlightedIndex` (`:45`) — the controlled value; [`None`] is uncontrolled.
    pub highlighted_index: Option<I>,
    /// `onHighlightedIndexChange` (`:46`) — the consumer's change callback; [`None`]
    /// writes the internal signal (upstream's `?? internalSetHighlightedIndex`).
    pub on_highlighted_index_change: Option<Rc<dyn Fn(i32)>>,
    /// `direction` (`:47`) — a reactive `TextDirection` source.
    pub direction: D,
    /// `rootRef` (`:48`) — merged with the internal root slot.
    pub root_ref: InputRef<HtmlElement>,
    /// `enableHomeAndEndKeys` (`:49-54`) — upstream default `false`.
    pub enable_home_and_end_keys: bool,
    /// `stopEventPropagation` (`:55-60`) — the component layer defaults it to `true`
    /// (`CompositeRoot.tsx:33`).
    pub stop_event_propagation: bool,
    /// `disabledIndices` (`:61-65`).
    pub disabled_indices: Option<DisabledIndices>,
    /// `modifierKeys` (`:66-71`) — upstream default `[]` (`:87`).
    pub modifier_keys: Vec<ModifierKey>,
}

/// Upstream's returned `props` bag (`useCompositeRoot.ts:319-330`) — the ref, the
/// input-selecting `onFocus`, and the navigation `onKeyDown`. The view layer spreads
/// these onto the rendered root element.
pub struct CompositeRootProps {
    /// `ref` (`:320`) — the internal root slot merged with the external `rootRef`.
    pub ref_callback: Option<MergedRefCallback<HtmlElement>>,
    /// `onFocus` (`:321-328`) — selects the whole value of a focused native input.
    pub on_focus: Rc<dyn Fn(&FocusEvent)>,
    /// `onKeyDown` (`:329`) — the navigation pipeline.
    pub on_key_down: ElementEventHandler<KeyboardEvent>,
}

/// Upstream's return object (`useCompositeRoot.ts:332-339`) plus the
/// `set_disabled_indices` re-run method (module docs).
pub struct UseCompositeRoot<M> {
    /// `props` (`:333`).
    pub props: CompositeRootProps,
    /// `highlightedIndex` (`:334`) — the resolved external ?? internal index.
    pub highlighted_index: Memo<i32>,
    /// `onHighlightedIndexChange` (`:335`) — the stable change handle; the second tuple
    /// element is `shouldScrollIntoView` (upstream's optional second parameter).
    pub on_highlighted_index_change: StableCallback<(i32, bool), ()>,
    /// `elementsRef` (`:336`) — shared with the `CompositeList` wiring.
    pub elements_ref: CompositeListElementsRef,
    /// `onMapChange` (`:337`) — the reconciliation pump; hand this to
    /// `provide_composite_list`'s `on_map_change` (upstream `CompositeRoot.tsx:87-90`
    /// fans each publication here and to the consumer prop).
    pub on_map_change: Rc<dyn Fn(&CompositeListMap<M>)>,
    /// `relayKeyboardEvent` (`:338`) — the same handler object as
    /// `props.on_key_down`, exposed for the context's detached-trigger forwarding.
    pub relay_keyboard_event: ElementEventHandler<KeyboardEvent>,
    /// The late-`disabledIndices` re-render analog (`useCompositeRoot.ts:166-193`): a
    /// late `disabledIndices` resolution stores it and re-runs the default-tab-stop
    /// re-validation — see the module docs.
    pub set_disabled_indices: Rc<dyn Fn(Option<DisabledIndices>)>,
}

/// Port of `useCompositeRoot` (`useCompositeRoot.ts:74-340`). Must be called inside a
/// reactive owner (a component) — the refs and the re-validation effect register there.
pub fn use_composite_root<M, I, D>(params: UseCompositeRootParams<I, D>) -> UseCompositeRoot<M>
where
    M: Clone + PartialEq + 'static,
    I: Clone + Get<Value = Option<i32>> + GetUntracked<Value = Option<i32>> + 'static,
    D: Clone + Get<Value = TextDirection> + GetUntracked<Value = TextDirection> + 'static,
{
    let UseCompositeRootParams {
        orientation,
        grid,
        loop_focus,
        on_loop,
        highlighted_index: external_highlighted_index,
        on_highlighted_index_change: external_set_highlighted_index,
        direction,
        root_ref: external_root_ref,
        enable_home_and_end_keys,
        stop_event_propagation,
        disabled_indices,
        modifier_keys,
    } = params;

    // `orientation = 'both'` (`:77`).
    let orientation = orientation.unwrap_or(Orientation::Both);

    // `React.useState(0)` (`:90`) + `isGrid` (`:91`).
    let internal_highlighted_index = RwSignal::new(0);
    let is_grid = grid.is_some();

    // `React.useRef<HTMLElement | null>(null)` (`:93`) merged with the external `rootRef`
    // (`:94`).
    let root_ref = use_ref_with_init(|| None::<HtmlElement>);
    let merged_root_ref = use_merged_refs(
        InputRef::Object(Rc::new(root_ref.clone())),
        external_root_ref,
    );

    // `elementsRef`/`hasSetDefaultIndexRef`/`highlightedElementRef` (`:96-98`).
    let elements_ref: CompositeListElementsRef = Rc::new(RefCell::new(Vec::new()));
    let has_set_default_index = Rc::new(std::cell::Cell::new(false));
    let highlighted_element_ref: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));

    // The disabled indices ride a shared slot so `set_disabled_indices` updates what the
    // already-built handlers see (module docs).
    let disabled_indices_slot: Rc<RefCell<Option<DisabledIndices>>> =
        Rc::new(RefCell::new(disabled_indices));

    // `highlightedIndex = externalHighlightedIndex ?? internalHighlightedIndex` (`:100`).
    // The external source rides a `SendWrapper` clone into the memo (reactive_graph's
    // `Send + Sync` compute contract) so the original stays available for the
    // re-validation closure below.
    let highlighted_index: Memo<i32> = {
        let external = external_highlighted_index.clone().map(SendWrapper::new);
        Memo::new(move |_| {
            external
                .as_ref()
                .map(|source| source.get())
                .unwrap_or(None)
                .unwrap_or_else(|| internal_highlighted_index.get())
        })
    };

    // The stable change handler (`:101-108`): snapshot the highlighted element, write
    // the external-or-internal index, and scroll on request.
    let on_highlighted_index_change = {
        let elements_ref = Rc::clone(&elements_ref);
        let highlighted_element_ref = Rc::clone(&highlighted_element_ref);
        let root_ref = root_ref.clone();
        let direction = direction.clone();
        use_stable_callback(Some(
            move |(index, should_scroll_into_view): (i32, bool)| {
                *highlighted_element_ref.borrow_mut() =
                    elements_ref.borrow().get(index as usize).cloned().flatten();
                match &external_set_highlighted_index {
                    Some(external_set) => external_set(index),
                    None => internal_highlighted_index.set(index),
                }
                if should_scroll_into_view {
                    let new_active_item = elements_ref
                        .borrow()
                        .get(index as usize)
                        .cloned()
                        .flatten()
                        .and_then(|element| element.dyn_into::<HtmlElement>().ok());
                    let scroll_container = root_ref.with_value(|slot| slot.clone());
                    scroll_into_view_if_needed(
                        scroll_container.as_ref(),
                        new_active_item.as_ref(),
                        direction.get_untracked(),
                        orientation,
                    );
                }
            },
        ))
    };

    // The late-`disabledIndices` re-validation body (`:166-193`) — factored so
    // `set_disabled_indices` re-runs it (module docs).
    let revalidate_default_tab_stop = {
        let disabled_indices_slot = Rc::clone(&disabled_indices_slot);
        let external_highlighted_index = external_highlighted_index.clone();
        let has_set_default_index = Rc::clone(&has_set_default_index);
        let elements_ref = Rc::clone(&elements_ref);
        let highlighted_index = highlighted_index.clone();
        let on_highlighted_index_change = on_highlighted_index_change.clone();
        Rc::new(move || {
            let disabled_indices = disabled_indices_slot.borrow().clone();
            if disabled_indices.is_none()
                || external_highlighted_index
                    .as_ref()
                    .map(|source| source.get_untracked())
                    .unwrap_or(None)
                    .is_some()
                || !has_set_default_index.get()
            {
                return;
            }
            let elements = elements_ref.borrow();
            let current = highlighted_index.get_untracked();
            if is_list_index_disabled(&elements, current, disabled_indices.as_ref()) {
                let first_enabled_index = find_non_disabled_list_index(
                    &elements,
                    FindNonDisabledListIndexOptions::new(disabled_indices.as_ref()),
                );
                if !is_index_out_of_list_bounds(&elements, first_enabled_index) {
                    on_highlighted_index_change.call((first_enabled_index, false));
                }
            }
        })
    };

    {
        let revalidate = Rc::clone(&revalidate_default_tab_stop);
        use_iso_layout_effect(move || revalidate());
    }

    // `onMapChange` (`:110-164`) — the reconciliation pump.
    let on_map_change: Rc<dyn Fn(&CompositeListMap<M>)> = {
        let has_set_default_index = Rc::clone(&has_set_default_index);
        let highlighted_element_ref = Rc::clone(&highlighted_element_ref);
        let elements_ref = Rc::clone(&elements_ref);
        let highlighted_index = highlighted_index.clone();
        let disabled_indices_slot = Rc::clone(&disabled_indices_slot);
        let on_highlighted_index_change = on_highlighted_index_change.clone();
        let root_ref = root_ref.clone();
        let direction = direction.clone();
        Rc::new(move |map: &CompositeListMap<M>| {
            if map.is_empty() {
                return;
            }

            let highlighted = highlighted_index.get_untracked();
            let disabled_indices = disabled_indices_slot.borrow().clone();

            if has_set_default_index.get() {
                let elements = elements_ref.borrow();
                // Items added or removed around the highlighted one shift its index, so
                // the tab stop would otherwise move to a different item and navigation
                // would resume from the wrong position (`:115-119`).
                let next_index = highlighted_element_ref
                    .borrow()
                    .clone()
                    .and_then(|element| {
                        elements
                            .iter()
                            .position(|entry| entry.as_ref() == Some(&element))
                    })
                    .map_or(-1, |position| position as i32);

                if next_index == -1 {
                    // A replacement at the same index can keep the tab stop. Otherwise
                    // move it to an eligible item so a missing, hidden, or disabled
                    // replacement does not take the composite out of the tab order
                    // (`:121-130`).
                    let replacement = elements.get(highlighted as usize).cloned().flatten();
                    if replacement.is_none()
                        || is_list_index_disabled(&elements, highlighted, disabled_indices.as_ref())
                    {
                        on_highlighted_index_change.call((
                            get_fallback_index(&elements, disabled_indices.as_ref()),
                            false,
                        ));
                    } else {
                        *highlighted_element_ref.borrow_mut() = replacement;
                    }
                } else if next_index != highlighted {
                    on_highlighted_index_change.call((next_index, false));
                }
                return;
            }

            has_set_default_index.set(true);

            // `Array.from(map.keys())` (`:139`) — the published map is index order.
            let sorted_elements: Vec<Option<Element>> = map.elements().cloned().map(Some).collect();
            let active_item = sorted_elements
                .iter()
                .flatten()
                .find(|element| element.has_attribute(ACTIVE_COMPOSITE_ITEM))
                .cloned();
            // The map value carries the item's own index, which is not its position
            // among the keys once a list mixes explicit and automatic indexes and leaves
            // gaps (`:144-147`).
            let active_index = active_item
                .as_ref()
                .and_then(|item| map.get(item))
                .map_or(-1, |metadata| metadata.index);

            if active_index != -1 {
                on_highlighted_index_change.call((active_index, false));
            } else if is_list_index_disabled(
                &sorted_elements,
                highlighted,
                disabled_indices.as_ref(),
            ) {
                // The default highlighted item is disabled, so it should not hold the
                // single roving tab stop. Move the tab stop to the first enabled item.
                // If every item is disabled, keep the current highlighted index
                // (`:151-161`).
                let first_enabled_index = find_non_disabled_list_index(
                    &sorted_elements,
                    FindNonDisabledListIndexOptions::new(disabled_indices.as_ref()),
                );
                if !is_index_out_of_list_bounds(&sorted_elements, first_enabled_index) {
                    on_highlighted_index_change.call((first_enabled_index, false));
                }
            }

            let scroll_container = root_ref.with_value(|slot| slot.clone());
            let active_html =
                active_item.and_then(|element| element.dyn_into::<HtmlElement>().ok());
            scroll_into_view_if_needed(
                scroll_container.as_ref(),
                active_html.as_ref(),
                direction.get_untracked(),
                orientation,
            );
        })
    };

    // `wrappedOnLoop` (`:195-202`): the grid navigator's 3-argument cell-level shape
    // over the user's 4-argument callback.
    let wrapped_on_loop: Rc<dyn Fn(Option<&KeyboardEvent>, i32, i32) -> i32> = {
        let on_loop = on_loop.clone();
        let elements_ref = Rc::clone(&elements_ref);
        Rc::new(move |event, prev_index, next_index| match &on_loop {
            None => next_index,
            Some(on_loop) => on_loop(event, prev_index, next_index, &elements_ref),
        })
    };

    // The keydown pipeline (`:206-317`).
    let on_key_down: ElementEventHandler<KeyboardEvent> = {
        let modifier_keys = modifier_keys.clone();
        let root_ref = root_ref.clone();
        let direction = direction.clone();
        let disabled_indices_slot = Rc::clone(&disabled_indices_slot);
        let elements_ref = Rc::clone(&elements_ref);
        let highlighted_index = highlighted_index.clone();
        let on_highlighted_index_change = on_highlighted_index_change.clone();
        let wrapped_on_loop = Rc::clone(&wrapped_on_loop);
        Rc::new(move |event: &KeyboardEvent| {
            let key = event.key();
            let key = key.as_str();

            // The key whitelist with the Home/End opt-in (`:207-210`).
            let is_home_or_end = key == HOME || key == END;
            if !is_composite_key(key) || (!enable_home_and_end_keys && is_home_or_end) {
                return;
            }

            // The modifier gate (`:212-214`).
            if is_modifier_key_set(event, &modifier_keys) {
                return;
            }

            let element = root_ref.with_value(|slot| slot.clone());
            if element.is_none() {
                return;
            }

            let is_rtl = direction.get_untracked() == TextDirection::Rtl;

            // The RTL forward/backward mapping (`:221-226`).
            let horizontal_forward_key = if is_rtl { ARROW_LEFT } else { ARROW_RIGHT };
            let horizontal_backward_key = if is_rtl { ARROW_RIGHT } else { ARROW_LEFT };
            let forward_key = if orientation == Orientation::Vertical {
                ARROW_DOWN
            } else {
                horizontal_forward_key
            };
            let backward_key = if orientation == Orientation::Vertical {
                ARROW_UP
            } else {
                horizontal_backward_key
            };

            // The native-input yield (`:228-246`): return to native textbox behavior when
            // 1 - Shift is held to make a text selection, or if there already is a text
            // selection; 2 - arrow-ing forward and not in the last position of the text;
            // 3 - arrow-ing backward and not in the first position of the text.
            if let Some(native) = native_input_read(event) {
                let NativeInputRead { start, end, len } = native;
                if event.shift_key() || start != end {
                    return;
                }
                if key != backward_key && f64::from(start) < len {
                    return;
                }
                if key != forward_key && start > 0 {
                    return;
                }
            }

            let highlighted = highlighted_index.get_untracked();
            let disabled_indices = disabled_indices_slot.borrow().clone();
            let mut next_index = highlighted;
            let (min_index, max_index) = {
                let elements = elements_ref.borrow();
                (
                    get_min_list_index(&elements, disabled_indices.as_ref()),
                    get_max_list_index(&elements, disabled_indices.as_ref()),
                )
            };

            // Grid mode delegates entirely to the injected navigator (`:252-265`).
            if let Some(grid) = &grid {
                let state = CompositeGridNavigationState {
                    key,
                    event: Some(event),
                    elements_ref: &elements_ref,
                    highlighted_index: highlighted,
                    min_index,
                    max_index,
                    orientation,
                    loop_focus,
                    on_loop: Some(wrapped_on_loop.as_ref()),
                    disabled_indices: disabled_indices.as_ref(),
                    rtl: is_rtl,
                };
                next_index = grid(&state);
            }

            let is_forward_key = (orientation != Orientation::Vertical
                && key == horizontal_forward_key)
                || (orientation != Orientation::Horizontal && key == ARROW_DOWN);
            let is_backward_key = (orientation != Orientation::Vertical
                && key == horizontal_backward_key)
                || (orientation != Orientation::Horizontal && key == ARROW_UP);

            // The Home/End targets (`:274-280`).
            if enable_home_and_end_keys {
                if key == HOME {
                    next_index = min_index;
                } else if key == END {
                    next_index = max_index;
                }
            }

            // The loop/search branch for a no-op linear step (`:282-300`).
            if next_index == highlighted && (is_forward_key || is_backward_key) {
                if loop_focus && next_index == max_index && is_forward_key {
                    next_index = min_index;
                    if let Some(on_loop) = &on_loop {
                        next_index = on_loop(Some(event), highlighted, next_index, &elements_ref);
                    }
                } else if loop_focus && next_index == min_index && is_backward_key {
                    next_index = max_index;
                    if let Some(on_loop) = &on_loop {
                        next_index = on_loop(Some(event), highlighted, next_index, &elements_ref);
                    }
                } else {
                    let elements = elements_ref.borrow();
                    next_index = find_non_disabled_list_index(
                        &elements,
                        FindNonDisabledListIndexOptions {
                            starting_index: next_index,
                            decrement: is_backward_key,
                            disabled_indices: disabled_indices.as_ref(),
                            ..FindNonDisabledListIndexOptions::new(None)
                        },
                    );
                }
            }

            // The guarded commit (`:302-316`).
            let in_bounds = {
                let elements = elements_ref.borrow();
                next_index != highlighted && !is_index_out_of_list_bounds(&elements, next_index)
            };
            if in_bounds {
                if stop_event_propagation {
                    event.stop_propagation();
                }

                if is_grid || is_home_or_end || is_forward_key || is_backward_key {
                    event.prevent_default();
                }
                on_highlighted_index_change.call((next_index, true));

                // Wait for FocusManager `returnFocus` to execute (`:312-315`).
                queue_microtask_focus(&elements_ref, next_index);
            }
        })
    };

    // `relayKeyboardEvent` (`:338`) — the same handler object.
    let relay_keyboard_event = Rc::clone(&on_key_down);

    // The props bag (`:319-330`).
    let props = CompositeRootProps {
        ref_callback: merged_root_ref,
        on_focus: {
            let root_ref = root_ref.clone();
            Rc::new(move |event: &FocusEvent| {
                let element = root_ref.with_value(|slot| slot.clone());
                let target = get_target(event);
                if element.is_none() || target.is_none() {
                    return;
                }
                let Some(target) = target else { return };
                if !is_native_input(&target) {
                    return;
                }
                // `target.setSelectionRange(0, target.value.length)` (`:327`).
                let value_length = native_input_value_length(&target);
                set_full_selection(&target, value_length);
            })
        },
        on_key_down,
    };

    UseCompositeRoot {
        props,
        highlighted_index,
        on_highlighted_index_change,
        elements_ref,
        on_map_change,
        relay_keyboard_event,
        set_disabled_indices: {
            let disabled_indices_slot = Rc::clone(&disabled_indices_slot);
            let revalidate = Rc::clone(&revalidate_default_tab_stop);
            Rc::new(move |disabled_indices: Option<DisabledIndices>| {
                *disabled_indices_slot.borrow_mut() = disabled_indices;
                revalidate();
            })
        },
    }
}

/// The native-input yield's read (`useCompositeRoot.ts:229-246`), taken only when the
/// event target is a native input that is not element-disabled. `None` when the target is
/// not such an input — the port's spelling of the `if` guard. A failed selection read
/// also surfaces as `None` (navigation proceeds instead of yielding); upstream's
/// `selectionStart == null` arm is unreachable for this read because `isNativeInput` has
/// already verified the selection getter works for `<input>` targets and `<textarea>`
/// never throws it — the fail-soft note `crate::composite` documents.
struct NativeInputRead {
    start: u32,
    end: u32,
    /// `target.value.length` — JS's UTF-16 code-unit count.
    len: f64,
}

fn native_input_read(event: &KeyboardEvent) -> Option<NativeInputRead> {
    let target = get_target(event)?;
    if !is_native_input(&target) {
        return None;
    }
    let html: HtmlElement = target.dyn_into().ok()?;
    if is_element_disabled(Some(&html)) {
        return None;
    }

    let (start, end, len) = if let Some(input) = html.dyn_ref::<web_sys::HtmlInputElement>() {
        let start = input.selection_start().ok().flatten()?;
        let end = input.selection_end().ok().flatten()?;
        (start, end, input.value().encode_utf16().count() as f64)
    } else {
        let textarea = html.dyn_ref::<web_sys::HtmlTextAreaElement>()?;
        let start = textarea.selection_start().ok().flatten()?;
        let end = textarea.selection_end().ok().flatten()?;
        (start, end, textarea.value().encode_utf16().count() as f64)
    };

    Some(NativeInputRead { start, end, len })
}

/// `target.value.length` (`useCompositeRoot.ts:327`) — JS's UTF-16 code-unit count.
fn native_input_value_length(target: &EventTarget) -> u32 {
    if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
        return input.value().encode_utf16().count() as u32;
    }
    target
        .dyn_ref::<web_sys::HtmlTextAreaElement>()
        .map(|textarea| textarea.value().encode_utf16().count() as u32)
        .unwrap_or(0)
}

/// `target.setSelectionRange(0, target.value.length)` (`useCompositeRoot.ts:327`) —
/// input or textarea.
fn set_full_selection(target: &EventTarget, value_length: u32) {
    if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
        let _ = input.set_selection_range(0, value_length);
        return;
    }
    if let Some(textarea) = target.dyn_ref::<web_sys::HtmlTextAreaElement>() {
        let _ = textarea.set_selection_range(0, value_length);
    }
}

/// `getFallbackIndex` (`useCompositeRoot.ts:345-365`): the active item when it can take
/// focus, otherwise the first item that can; falls back to index 0 so an all-disabled
/// composite keeps the index in range and regains a tab stop as soon as one of its items
/// becomes focusable.
fn get_fallback_index(
    elements: &[Option<Element>],
    disabled_indices: Option<&DisabledIndices>,
) -> i32 {
    let mut fallback_index = -1;

    for (index, element) in elements.iter().enumerate() {
        let Some(element) = element else {
            continue;
        };

        if is_list_index_disabled(elements, index as i32, disabled_indices) {
            continue;
        }

        if element.has_attribute(ACTIVE_COMPOSITE_ITEM) {
            return index as i32;
        }

        if fallback_index == -1 {
            fallback_index = index as i32;
        }
    }

    std::cmp::max(fallback_index, 0)
}

/// `isModifierKeySet` (`useCompositeRoot.ts:367-377`): any of [`MODIFIER_KEYS`] not in
/// the allow-list being held blocks navigation.
fn is_modifier_key_set(event: &KeyboardEvent, ignored_modifier_keys: &[ModifierKey]) -> bool {
    MODIFIER_KEYS
        .iter()
        .any(|key| !ignored_modifier_keys.contains(key) && event.get_modifier_state(key))
}

/// The `queueMicrotask(() => elementsRef.current[nextIndex]?.focus())` deferral
/// (`useCompositeRoot.ts:313-315`). wasm runs it through `window.queueMicrotask` (the
/// `floating_focus_manager.rs` precedent); host builds have no microtask queue and no
/// DOM, so the arm is a no-op there — focus is only observable in the browser suite.
fn queue_microtask_focus(elements_ref: &CompositeListElementsRef, index: i32) {
    #[cfg(target_arch = "wasm32")]
    {
        let elements_ref = Rc::clone(elements_ref);
        if let Some(window) = web_sys::window() {
            let function =
                js_sys::Function::from(wasm_bindgen::closure::Closure::once_into_js(move || {
                    if let Some(element) =
                        elements_ref.borrow().get(index as usize).cloned().flatten()
                    {
                        if let Ok(html) = element.dyn_into::<HtmlElement>() {
                            let _ = html.focus();
                        }
                    }
                }));
            let _ = window.queue_microtask(&function);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (elements_ref, index);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};

    use super::*;
    use crate::composite_root_context::{
        CompositeRootContextValue, use_composite_root_context, use_composite_root_context_required,
    };

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn root_params() -> UseCompositeRootParams<RwSignal<Option<i32>>, Memo<TextDirection>> {
        UseCompositeRootParams {
            orientation: None,
            grid: None,
            loop_focus: true,
            on_loop: None,
            highlighted_index: None,
            on_highlighted_index_change: None,
            direction: Memo::new(|_| TextDirection::Ltr),
            root_ref: InputRef::Empty,
            enable_home_and_end_keys: false,
            stop_event_propagation: false,
            disabled_indices: None,
            modifier_keys: Vec::new(),
        }
    }

    // The hook builds without a DOM on host: the refs/slot plumbing is environment-free
    // and the default index is 0 (`useCompositeRoot.ts:90`).
    #[test]
    fn builds_with_default_state() {
        let _owner = in_owner();
        let root = use_composite_root::<String, _, _>(root_params());
        assert_eq!(root.highlighted_index.get_untracked(), 0);
        assert!(root.elements_ref.borrow().is_empty());
        assert!(
            root.props.ref_callback.is_some(),
            "the merged ref exists even with an empty external branch"
        );
    }

    // The controlled/uncontrolled split (`useCompositeRoot.ts:100`): the external source
    // wins while present, the internal signal answers otherwise.
    #[test]
    fn external_index_wins_while_present() {
        let _owner = in_owner();
        let external: RwSignal<Option<i32>> = RwSignal::new(Some(3));
        let params = UseCompositeRootParams {
            highlighted_index: Some(external),
            ..root_params()
        };
        let root = use_composite_root::<String, _, _>(params);
        assert_eq!(root.highlighted_index.get_untracked(), 3);

        external.set(None);
        assert_eq!(root.highlighted_index.get_untracked(), 0);
    }

    // The context accessors (`CompositeRootContext.ts:21-32`): `None` outside a root,
    // the provided value inside one, and the required accessor's upstream error message.
    #[test]
    fn root_context_is_optional_outside_a_root_and_required_panics() {
        let _owner = in_owner();
        assert!(
            use_composite_root_context().is_none(),
            "no context outside a root"
        );

        let result = std::panic::catch_unwind(|| {
            let _owner = in_owner();
            use_composite_root_context_required();
        });
        let message = result
            .err()
            .and_then(|payload| {
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            })
            .unwrap_or_default();
        assert!(
            message.starts_with("Base UI: CompositeRootContext is missing."),
            "the missing-provider error reads {message:?}"
        );

        let _owner = in_owner();
        let root = use_composite_root::<String, _, _>(root_params());
        let on_hic = root.on_highlighted_index_change.clone();
        crate::composite_root_context::provide_composite_root_context(CompositeRootContextValue {
            highlighted_index: root.highlighted_index.clone(),
            on_highlighted_index_change: Rc::new(move |index, scroll| {
                on_hic.call((index, scroll));
            }),
            highlight_item_on_hover: false,
            relay_keyboard_event: root.relay_keyboard_event.clone(),
        });
        let provided = use_composite_root_context().expect("provided inside a root");
        assert_eq!(provided.highlighted_index.get_untracked(), 0);
    }

    // `getFallbackIndex` (`useCompositeRoot.ts:345-365`) needs no DOM for the
    // element-less arms: an empty list and element-less slots both fall back to index 0
    // ("an all-disabled composite keeps the index in range"). The element-driven arms
    // (active marker, disabled skip) run in the browser suite.
    #[test]
    fn fallback_index_falls_back_to_zero_for_an_empty_list() {
        assert_eq!(get_fallback_index(&[], None), 0);
        assert_eq!(
            get_fallback_index(&[None, None], None),
            0,
            "element-less slots cannot hold the tab stop"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{Element, FocusEvent, HtmlElement, KeyboardEvent};

    use super::*;
    use crate::composite_grid_navigation::{CompositeGridConfig, grid_navigation};
    use crate::composite_list::provide_composite_list;
    use crate::composite_root_context::{
        CompositeRootContextValue, provide_composite_root_context,
    };
    use crate::use_composite_item::{UseCompositeItem, UseCompositeItemParams, use_composite_item};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    type Metadata = String;

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn key_event(key: &str, modifiers: &[&str]) -> KeyboardEvent {
        let init = web_sys::KeyboardEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_key(key);
        for modifier in modifiers {
            match *modifier {
                "Shift" => init.set_shift_key(true),
                "Control" => init.set_ctrl_key(true),
                "Alt" => init.set_alt_key(true),
                "Meta" => init.set_meta_key(true),
                _ => {}
            }
        }
        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap()
    }

    fn key_event_simple(key: &str) -> KeyboardEvent {
        key_event(key, &[])
    }

    /// Lets the registry's coalesced flush and the keydown's deferred focus run: one
    /// awaited resolved promise per microtask tick.
    async fn run_microtasks(ticks: usize) {
        for _ in 0..ticks {
            JsFuture::from(js_sys::Promise::resolve(&JsValue::undefined()))
                .await
                .unwrap();
        }
    }

    struct Harness {
        owner: Owner,
        root: UseCompositeRoot<Metadata>,
        root_element: HtmlElement,
        items: Vec<UseCompositeItem>,
        nodes: Vec<HtmlElement>,
        external_index: Option<RwSignal<Option<i32>>>,
        changes: Rc<RefCell<Vec<i32>>>,
    }

    /// The composite wiring the `CompositeRoot` view builds (`CompositeRoot.tsx:42-95`),
    /// minus the `useRenderElement` call: the list registry provided with the root's
    /// elements ref and reconciliation pump (`:86-90`), the root context provided
    /// (`:73-81`), the root element attached through the merged ref, and `count` items
    /// registered through `use_composite_item`.
    ///
    /// `controlled` mirrors upstream's `highlightedIndex` + `onHighlightedIndexChange`
    /// pair: the external signal is the state, the change callback writes it.
    async fn build_harness(
        count: usize,
        controlled: bool,
        configure: impl FnOnce(&mut UseCompositeRootParams<RwSignal<Option<i32>>, Memo<TextDirection>>),
    ) -> Harness {
        let owner = Owner::new();
        owner.set();

        let (external_index, changes) = if controlled {
            let external: RwSignal<Option<i32>> = RwSignal::new(Some(0));
            let changes: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
            (Some(external), Rc::clone(&changes))
        } else {
            (None, Rc::new(RefCell::new(Vec::new())))
        };

        let mut params = UseCompositeRootParams {
            orientation: None,
            grid: None,
            loop_focus: true,
            on_loop: None,
            highlighted_index: external_index,
            on_highlighted_index_change: external_index.map(|signal| {
                let changes = Rc::clone(&changes);
                Rc::new(move |index: i32| {
                    changes.borrow_mut().push(index);
                    signal.set(Some(index));
                }) as Rc<dyn Fn(i32)>
            }),
            direction: Memo::new(|_| TextDirection::Ltr),
            root_ref: InputRef::Empty,
            enable_home_and_end_keys: false,
            stop_event_propagation: false,
            disabled_indices: None,
            modifier_keys: Vec::new(),
        };
        configure(&mut params);

        let root = use_composite_root::<Metadata, _, _>(params);

        // The `CompositeRoot.tsx:87-90` fan-out: the registry's onMapChange is the
        // reconciliation pump (the consumer-prop arm is not asserted here).
        let pump = root.on_map_change.clone();
        let _list =
            provide_composite_list::<Metadata>(Rc::clone(&root.elements_ref), None, move |map| {
                pump(map);
            });

        // The `CompositeRoot.tsx:73-81` context value.
        let on_hic = root.on_highlighted_index_change.clone();
        provide_composite_root_context(CompositeRootContextValue {
            highlighted_index: root.highlighted_index.clone(),
            on_highlighted_index_change: Rc::new(move |index, should_scroll| {
                on_hic.call((index, should_scroll));
            }),
            highlight_item_on_hover: false,
            relay_keyboard_event: root.relay_keyboard_event.clone(),
        });

        // The root element attached through `props.ref` (`useCompositeRoot.ts:320`).
        let root_element = document()
            .create_element("div")
            .unwrap()
            .unchecked_into::<HtmlElement>();
        document()
            .body()
            .unwrap()
            .append_child(&root_element)
            .unwrap();
        if let Some(set_ref) = &root.props.ref_callback {
            set_ref(Some(&root_element));
        }

        // The items (`TestGridItems`/inline `CompositeItem`s, `CompositeRoot.test.tsx
        // :36-46`): label, register, and roving-tabindex the view layer's analog.
        let mut items = Vec::new();
        let mut nodes = Vec::new();
        for index in 0..count {
            let node = document()
                .create_element("div")
                .unwrap()
                .unchecked_into::<HtmlElement>();
            node.set_text_content(Some(&format!("{}", index + 1)));
            node.set_attribute("data-testid", &format!("{}", index + 1))
                .unwrap();
            root_element.append_child(&node).unwrap();
            let item = use_composite_item::<Metadata>(UseCompositeItemParams { metadata: None });
            if let Some(set_ref) = &item.composite_ref {
                set_ref(Some(&node));
            }
            items.push(item);
            nodes.push(node);
        }
        run_microtasks(1).await;

        // The mount-time roving-tabindex binding (the view layer's first render).
        harness_sync_tabindex(&root, &items, &nodes);

        Harness {
            owner,
            root,
            root_element,
            items,
            nodes,
            external_index,
            changes,
        }
    }

    /// The roving tabindex binding the view layer performs per render
    /// (`useCompositeItem.ts:27`) — run at "mount" by [`build_harness`] and after every
    /// highlight change, mirroring the React render cycle.
    fn harness_sync_tabindex(
        root: &UseCompositeRoot<Metadata>,
        items: &[UseCompositeItem],
        nodes: &[HtmlElement],
    ) {
        for (item, node) in items.iter().zip(nodes) {
            node.set_attribute(
                "tabindex",
                &item.composite_props.tab_index.get_untracked().to_string(),
            )
            .unwrap();
        }
        let _ = root;
    }

    impl Harness {
        fn sync_tabindex(&self) {
            harness_sync_tabindex(&self.root, &self.items, &self.nodes);
        }

        /// `act(() => item.focus())` (`CompositeRoot.test.tsx:81`): real DOM focus plus
        /// the item's onFocus moving the highlight.
        fn focus_item(&self, index: usize) {
            self.nodes[index].focus().unwrap();
            let event = web_sys::FocusEvent::new("focusin").unwrap();
            self.nodes[index].dispatch_event(event.as_ref()).unwrap();
            (self.items[index].composite_props.on_focus)(event.unchecked_ref::<FocusEvent>());
            self.sync_tabindex();
        }

        /// `fireEvent.keyDown(item, { key })` (`CompositeRoot.test.tsx:85`): the keydown
        /// is dispatched on the item so `getTarget` resolves it, then handed to the
        /// root's `onKeyDown` (the bubbling-listener analog), followed by the upstream
        /// `await flushMicrotasks()` (the commit's deferred focus runs in a microtask).
        async fn key_down(&self, index: usize, key: &str, modifiers: &[&str]) -> bool {
            let event = key_event(key, modifiers);
            self.nodes[index].dispatch_event(event.as_ref()).unwrap();
            (self.root.props.on_key_down)(&event);
            self.sync_tabindex();
            let prevented = event.default_prevented();
            run_microtasks(1).await;
            prevented
        }

        fn focus_label(&self) -> Option<String> {
            let active = document().active_element()?;
            let inside = self.root_element.contains(Some(&active));
            inside.then(|| {
                active
                    .unchecked_ref::<Element>()
                    .get_attribute("data-testid")
            })?
        }

        fn tabindex(&self, index: usize) -> String {
            self.nodes[index]
                .get_attribute("tabindex")
                .unwrap_or_default()
        }
    }

    /// The default uncontrolled params for the hand-built fixtures.
    fn uncontrolled_params() -> UseCompositeRootParams<RwSignal<Option<i32>>, Memo<TextDirection>> {
        UseCompositeRootParams {
            orientation: None,
            grid: None,
            loop_focus: true,
            on_loop: None,
            highlighted_index: None,
            on_highlighted_index_change: None,
            direction: Memo::new(|_| TextDirection::Ltr),
            root_ref: InputRef::Empty,
            enable_home_and_end_keys: false,
            stop_event_propagation: false,
            disabled_indices: None,
            modifier_keys: Vec::new(),
        }
    }

    /// The shared context wiring every hand-built fixture provides (`CompositeRoot.tsx
    /// :73-95`): the registry fan-out plus the root context.
    fn provide_wiring(root: &UseCompositeRoot<Metadata>) {
        let pump = root.on_map_change.clone();
        let _list =
            provide_composite_list::<Metadata>(Rc::clone(&root.elements_ref), None, move |map| {
                pump(map);
            });
        let on_hic = root.on_highlighted_index_change.clone();
        provide_composite_root_context(CompositeRootContextValue {
            highlighted_index: root.highlighted_index.clone(),
            on_highlighted_index_change: Rc::new(move |index, should_scroll| {
                on_hic.call((index, should_scroll));
            }),
            highlight_item_on_hover: false,
            relay_keyboard_event: root.relay_keyboard_event.clone(),
        });

        // The root element through `props.ref` (`useCompositeRoot.ts:320`) — the keydown
        // pipeline's `rootRef.current` guard reads it.
        let root_element = document()
            .create_element("div")
            .unwrap()
            .unchecked_into::<HtmlElement>();
        document()
            .body()
            .unwrap()
            .append_child(&root_element)
            .unwrap();
        if let Some(set_ref) = &root.props.ref_callback {
            set_ref(Some(&root_element));
        }
    }

    /// Dispatches a keydown on `node` and hands it to the root's `onKeyDown` — the
    /// bubbling-listener analog of `fireEvent.keyDown`, with the flush-microtasks await.
    async fn key_down_on(root: &UseCompositeRoot<Metadata>, node: &HtmlElement, key: &str) -> bool {
        let event = key_event_simple(key);
        node.dispatch_event(event.as_ref()).unwrap();
        (root.props.on_key_down)(&event);
        let prevented = event.default_prevented();
        run_microtasks(1).await;
        prevented
    }

    fn tabindex_of(node: &HtmlElement) -> String {
        node.get_attribute("tabindex").unwrap_or_default()
    }

    /// Whether the item currently holds the roving tab stop — the port's analog of the
    /// upstream `toHaveAttribute('tabindex', '0')` assertions (the attribute binding is
    /// the view layer's; the value lives on the item's memos).
    fn is_highlighted(item: &crate::use_composite_item::UseCompositeItem) -> bool {
        item.is_highlighted.get_untracked()
    }

    /// The focused element's `data-testid` when it is the given node (`toHaveFocus` +
    /// `getByTestId`).
    fn focus_label_of(node: &HtmlElement) -> Option<String> {
        let active = document().active_element()?;
        (active.unchecked_ref::<Element>() == node.unchecked_ref::<Element>())
            .then(|| node.get_attribute("data-testid"))?
    }

    // Mirrors `CompositeRoot.test.tsx:60-108` — "controlled mode": arrows move highlight
    // and focus together across the full down/down/up/up walk.
    #[wasm_bindgen_test]
    async fn controlled_mode_moves_highlight_and_focus() {
        init_executor();
        let harness = build_harness(3, true, |_| {}).await;
        harness.focus_item(0);

        assert_eq!(harness.tabindex(0), "0");

        harness.key_down(0, "ArrowDown", &[]).await;
        assert_eq!(harness.tabindex(1), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("2"));

        harness.key_down(1, "ArrowDown", &[]).await;
        assert_eq!(harness.tabindex(2), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("3"));

        harness.key_down(2, "ArrowUp", &[]).await;
        assert_eq!(harness.tabindex(1), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("2"));

        harness.key_down(1, "ArrowUp", &[]).await;
        assert_eq!(harness.tabindex(0), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("1"));
        assert_eq!(
            harness.changes.borrow().clone(),
            // The leading 0 is the `act(() => item1.focus())` highlight; the rest is the
            // arrow walk (`:85-107`).
            vec![0, 1, 2, 1, 0],
            "onHighlightedIndexChange received each new index"
        );
    }

    // Mirrors `CompositeRoot.test.tsx:110-148` — "uncontrolled mode": the same walk with
    // the internal signal.
    #[wasm_bindgen_test]
    async fn uncontrolled_mode_moves_highlight_and_focus() {
        init_executor();
        let harness = build_harness(3, false, |_| {}).await;
        harness.focus_item(0);

        harness.key_down(0, "ArrowDown", &[]).await;
        assert_eq!(harness.tabindex(1), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("2"));

        harness.key_down(1, "ArrowDown", &[]).await;
        assert_eq!(harness.tabindex(2), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("3"));

        harness.key_down(2, "ArrowUp", &[]).await;
        assert_eq!(harness.tabindex(1), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("2"));

        harness.key_down(1, "ArrowUp", &[]).await;
        assert_eq!(harness.tabindex(0), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("1"));
        assert_eq!(harness.root.highlighted_index.get_untracked(), 0);
    }

    // Mirrors `CompositeRoot.test.tsx:150-163` — "uses an active item explicit index as
    // the initial highlighted index": the controlled root starts at 0, the index-2 item
    // carries the active marker, and the first publication adopts 2.
    #[wasm_bindgen_test]
    async fn active_item_explicit_index_becomes_the_initial_highlight() {
        init_executor();
        let harness = build_harness(0, true, |_| {}).await;

        // The `IndexedItem` fixture (`CompositeRoot.test.tsx:25-34`): explicit indexes
        // 2/0/1 with the active marker on the index-2 item.
        let two = document()
            .create_element("div")
            .unwrap()
            .unchecked_into::<HtmlElement>();
        two.set_attribute("data-testid", "two").unwrap();
        two.set_attribute("data-composite-item-active", "").unwrap();
        document().body().unwrap().append_child(&two).unwrap();
        let zero = document()
            .create_element("div")
            .unwrap()
            .unchecked_into::<HtmlElement>();
        zero.set_attribute("data-testid", "zero").unwrap();
        document().body().unwrap().append_child(&zero).unwrap();
        let one = document()
            .create_element("div")
            .unwrap()
            .unchecked_into::<HtmlElement>();
        one.set_attribute("data-testid", "one").unwrap();
        document().body().unwrap().append_child(&one).unwrap();

        use crate::use_composite_list_item::{UseCompositeListItemParams, use_composite_list_item};
        let item_two = use_composite_list_item::<Metadata, _>(UseCompositeListItemParams {
            guess: false,
            index: RwSignal::new(Some(2)),
            label: None,
            metadata: None,
            text_ref: None,
        });
        let item_zero = use_composite_list_item::<Metadata, _>(UseCompositeListItemParams {
            guess: false,
            index: RwSignal::new(Some(0)),
            label: None,
            metadata: None,
            text_ref: None,
        });
        let item_one = use_composite_list_item::<Metadata, _>(UseCompositeListItemParams {
            guess: false,
            index: RwSignal::new(Some(1)),
            label: None,
            metadata: None,
            text_ref: None,
        });
        (item_two.ref_callback)(Some(&two));
        (item_zero.ref_callback)(Some(&zero));
        (item_one.ref_callback)(Some(&one));
        run_microtasks(1).await;

        assert_eq!(
            harness.changes.borrow().clone(),
            vec![2],
            "onHighlightedIndexChange was called with the active item's index"
        );
        assert_eq!(harness.root.highlighted_index.get_untracked(), 2);
    }

    // Mirrors `CompositeRoot.test.tsx:302-344` — Home/End move focus to the first/last
    // item, only with `enableHomeAndEndKeys`.
    #[wasm_bindgen_test]
    async fn home_and_end_are_opt_in() {
        init_executor();
        let harness = build_harness(3, false, |params| {
            params.enable_home_and_end_keys = true;
        })
        .await;
        harness.focus_item(0);

        harness.key_down(0, "End", &[]).await;
        assert_eq!(harness.tabindex(2), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("3"));

        harness.key_down(2, "Home", &[]).await;
        assert_eq!(harness.tabindex(0), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("1"));

        // Without the flag the keys are outside the whitelist gate.
        let harness = build_harness(3, false, |_| {}).await;
        harness.focus_item(0);
        harness.key_down(0, "End", &[]).await;
        assert_eq!(
            harness.tabindex(0),
            "0",
            "End is inert without enableHomeAndEndKeys"
        );
    }

    // Mirrors `CompositeRoot.test.tsx:346-380` — "calls onLoop and uses its return
    // value" (nine items, wrap from the last), and `:382-406` — no loop with
    // `loopFocus: false`.
    #[wasm_bindgen_test]
    async fn on_loop_receives_the_wrap_and_its_return_wins() {
        init_executor();
        let loops: Rc<RefCell<Vec<(String, i32, i32)>>> = Rc::new(RefCell::new(Vec::new()));
        let harness = build_harness(9, false, |params| {
            let loops = Rc::clone(&loops);
            params.on_loop = Some(Rc::new(move |event, prev, next, elements| {
                loops.borrow_mut().push((
                    event.map(|event| event.key()).unwrap_or_default(),
                    prev,
                    next,
                ));
                assert!(
                    !elements.borrow().is_empty(),
                    "elementsRef is provided to onLoop"
                );
                1 // the upstream mock's return value
            }));
        })
        .await;
        harness.focus_item(8);

        harness.key_down(8, "ArrowDown", &[]).await;

        assert_eq!(
            loops.borrow().clone(),
            vec![("ArrowDown".into(), 8, 0)],
            "onLoop saw the wrap from the last item to the first"
        );
        assert_eq!(
            harness.tabindex(1),
            "0",
            "the return value overrides the target"
        );
        assert_eq!(harness.focus_label().as_deref(), Some("2"));

        // loopFocus: false — no wrap, no onLoop, the tab stop stays.
        let harness = build_harness(9, false, |params| {
            params.loop_focus = false;
            let loops = Rc::clone(&loops);
            params.on_loop = Some(Rc::new(move |_event, _prev, _next, _elements| {
                loops.borrow_mut().push(("sentinel".into(), -1, -1));
                0
            }));
        })
        .await;
        harness.focus_item(8);
        harness.key_down(8, "ArrowDown", &[]).await;
        assert!(
            loops.borrow().iter().all(|(key, _, _)| key != "sentinel"),
            "onLoop is never called without loopFocus"
        );
        assert_eq!(
            harness.tabindex(8),
            "0",
            "the tab stop stayed on the last item"
        );
    }

    // Mirrors `CompositeRoot.test.tsx:217-242` — the default prevention matrix.
    #[wasm_bindgen_test]
    async fn default_prevention_follows_the_orientation() {
        init_executor();
        // Each case builds its own harness (the upstream `it.each` matrix).
        async fn check(orientation: Orientation, key: &str) -> bool {
            let harness = build_harness(2, false, |params| {
                params.orientation = Some(orientation);
            })
            .await;
            harness.focus_item(0);
            harness.key_down(0, key, &[]).await
        }

        assert!(check(Orientation::Horizontal, "ArrowRight").await);
        assert!(!check(Orientation::Horizontal, "ArrowDown").await);
        assert!(check(Orientation::Vertical, "ArrowDown").await);
        assert!(!check(Orientation::Vertical, "ArrowRight").await);
        assert!(check(Orientation::Both, "ArrowRight").await);
        assert!(check(Orientation::Both, "ArrowDown").await);
    }

    // Mirrors `CompositeRoot.test.tsx:1218-1283` — the modifier gate: all four modifiers
    // block by default; the allow-list re-enables the listed ones and unlisted ones
    // still block.
    #[wasm_bindgen_test]
    async fn modifier_keys_gate_navigation() {
        init_executor();
        let harness = build_harness(2, false, |_| {}).await;
        harness.focus_item(0);
        for modifier in ["Shift", "Control", "Alt", "Meta"] {
            harness.key_down(0, "ArrowDown", &[modifier]);
            assert_eq!(
                harness.tabindex(0),
                "0",
                "{modifier} blocks navigation by default"
            );
        }

        // `modifierKeys: ['Alt', 'Meta']` (`CompositeRoot.test.tsx:1250-1283`).
        let harness = build_harness(3, false, |params| {
            params.modifier_keys = vec!["Alt", "Meta"];
        })
        .await;
        harness.focus_item(0);
        harness.key_down(0, "ArrowDown", &["Shift"]).await;
        assert_eq!(harness.tabindex(0), "0", "unlisted Shift still blocks");
        harness.key_down(0, "ArrowDown", &["Control"]).await;
        assert_eq!(harness.tabindex(0), "0", "unlisted Ctrl still blocks");
        harness.key_down(0, "ArrowDown", &["Alt"]).await;
        assert_eq!(harness.tabindex(1), "0", "listed Alt navigates");
        harness.key_down(1, "ArrowDown", &["Meta"]).await;
        assert_eq!(harness.tabindex(2), "0", "listed Meta navigates");
    }

    // Mirrors `CompositeRoot.test.tsx:408-461` — RTL horizontal: ArrowLeft moves
    // forward, ArrowRight moves backward, and backward wrap goes to the last item.
    #[wasm_bindgen_test]
    async fn rtl_horizontal_inverts_the_arrow_mapping() {
        init_executor();
        let harness = build_harness(3, false, |params| {
            params.orientation = Some(Orientation::Horizontal);
            params.direction = Memo::new(|_| TextDirection::Rtl);
        })
        .await;
        harness.focus_item(0);

        harness.key_down(0, "ArrowLeft", &[]).await;
        assert_eq!(
            harness.tabindex(1),
            "0",
            "ArrowLeft is the forward key in RTL"
        );
        harness.key_down(1, "ArrowLeft", &[]).await;
        assert_eq!(harness.tabindex(2), "0");
        harness.key_down(2, "ArrowRight", &[]).await;
        assert_eq!(
            harness.tabindex(1),
            "0",
            "ArrowRight is the backward key in RTL"
        );
        harness.key_down(1, "ArrowRight", &[]).await;
        assert_eq!(harness.tabindex(0), "0");

        // Loop backward: ArrowRight from the first item wraps to the last
        // (`CompositeRoot.test.tsx:455-460`).
        harness.key_down(0, "ArrowRight", &[]).await;
        assert_eq!(harness.tabindex(2), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("3"));
    }

    // Mirrors `CompositeRoot.test.tsx:165-215` — "keeps native input behavior when the
    // native target differs from the synthetic target": focus selects the whole value so
    // the first arrow key returns control to the textbox.
    #[wasm_bindgen_test]
    async fn native_input_yield_returns_control_to_the_textbox() {
        init_executor();
        let harness = build_harness(2, false, |params| {
            params.orientation = Some(Orientation::Horizontal);
        })
        .await;

        let input = document()
            .create_element("input")
            .unwrap()
            .unchecked_into::<web_sys::HtmlInputElement>();
        input.set_value("abcd");
        document().body().unwrap().append_child(&input).unwrap();

        // `fireEvent(host, focusEvent)` with the composed path starting at the input
        // (`:183-189`) — the port dispatches on the input itself, so `getTarget` resolves
        // the same native target.
        let focus_init = web_sys::FocusEventInit::new();
        focus_init.set_bubbles(true);
        let focus_event =
            web_sys::FocusEvent::new_with_focus_event_init_dict("focusin", &focus_init).unwrap();
        input.dispatch_event(focus_event.as_ref()).unwrap();
        (harness.root.props.on_focus)(focus_event.unchecked_ref::<FocusEvent>());

        assert_eq!(
            input.selection_start().unwrap(),
            Some(0),
            "focus selects the whole value (start)"
        );
        assert_eq!(
            input.selection_end().unwrap(),
            Some(4),
            "focus selects the whole value (end)"
        );

        // A mid-text forward arrow yields to the textbox: no composite move, no composite
        // focus (`:196-215`).
        harness.focus_item(0);
        input.set_selection_range(1, 1);

        let key_init = web_sys::KeyboardEventInit::new();
        key_init.set_bubbles(true);
        key_init.set_cancelable(true);
        key_init.set_key("ArrowRight");
        let key_event =
            KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &key_init).unwrap();
        input.dispatch_event(key_event.as_ref()).unwrap();
        (harness.root.props.on_key_down)(&key_event);
        run_microtasks(1).await;

        assert_eq!(
            harness.focus_label().as_deref(),
            Some("1"),
            "focus stayed on the composite item, not moved by the arrow"
        );
        assert_ne!(
            harness.root.highlighted_index.get_untracked(),
            1,
            "the composite highlight did not advance"
        );
    }

    // Mirrors `CompositeRoot.test.tsx:938-963` — the initial tab stop moves to the first
    // enabled item (and stays when everything is disabled), and `:965-999` — navigation
    // skips the listed indexes.
    #[wasm_bindgen_test]
    async fn disabled_indices_shape_the_tab_stop_and_navigation() {
        // disabledIndices: [0] — the tab stop moves to the first enabled item.
        let harness = build_harness(3, false, |params| {
            params.disabled_indices = Some(DisabledIndices::List(vec![0]));
        })
        .await;
        assert_eq!(harness.tabindex(0), "-1");
        assert_eq!(harness.tabindex(1), "0");
        assert_eq!(harness.tabindex(2), "-1");

        // All disabled: the tab stop stays on the first item.
        let harness = build_harness(2, false, |params| {
            params.disabled_indices = Some(DisabledIndices::List(vec![0, 1]));
        })
        .await;
        assert_eq!(harness.tabindex(0), "0");
        assert_eq!(harness.tabindex(1), "-1");

        // disabledIndices: [1], controlled — ArrowDown from 0 skips 1 and lands on 2;
        // ArrowUp from 2 skips 1 and lands back on 0 (`:965-999`).
        let harness = build_harness(3, true, |params| {
            params.disabled_indices = Some(DisabledIndices::List(vec![1]));
        })
        .await;
        harness.focus_item(0);
        harness.key_down(0, "ArrowDown", &[]).await;
        assert_eq!(harness.tabindex(2), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("3"));
        harness.key_down(2, "ArrowUp", &[]).await;
        assert_eq!(harness.tabindex(0), "0");
        assert_eq!(harness.focus_label().as_deref(), Some("1"));
    }

    // Mirrors `CompositeRoot.test.tsx:1065-1215` — the removal reconciliation matrix.
    #[wasm_bindgen_test]
    async fn item_removal_reconciles_the_tab_stop() {
        init_executor();

        /// Builds one plain item under the harness's owner and registry (the hand-built
        /// fixtures of the removal scenarios): element + hook + attach.
        fn make_item(label: &str) -> (HtmlElement, UseCompositeItem) {
            let node = document()
                .create_element("div")
                .unwrap()
                .unchecked_into::<HtmlElement>();
            node.set_attribute("data-testid", label).unwrap();
            // The roving tabindex the view layer binds (`useCompositeItem.ts:27`); any
            // tabindex value makes the div focusable for the focus assertions below.
            node.set_attribute("tabindex", "-1").unwrap();
            document().body().unwrap().append_child(&node).unwrap();
            let item = use_composite_item::<Metadata>(UseCompositeItemParams { metadata: None });
            if let Some(set_ref) = &item.composite_ref {
                set_ref(Some(&node));
            }
            (node, item)
        }

        /// Focuses an item: DOM focus plus the item hook's onFocus moving the highlight.
        fn focus_arbitrary(node: &HtmlElement, item: &UseCompositeItem) {
            node.focus().unwrap();
            let event = web_sys::FocusEvent::new("focusin").unwrap();
            node.dispatch_event(event.as_ref()).unwrap();
            (item.composite_props.on_focus)(event.unchecked_ref::<FocusEvent>());
        }

        /// Detaches an item (the `setProps({ show...: false })` unmount): a `None` cycle
        /// through the merged ref unregisters the node.
        fn detach(item: &UseCompositeItem) {
            if let Some(set_ref) = &item.composite_ref {
                set_ref(None);
            }
        }

        // Removing an earlier item keeps the tab stop on the same element and navigation
        // continues from it (`:1066-1103`).
        let owner = Owner::new();
        owner.set();
        let direction = Memo::new(|_| TextDirection::Ltr);
        let root =
            use_composite_root::<Metadata, RwSignal<Option<i32>>, _>(UseCompositeRootParams {
                highlighted_index: None,
                direction,
                ..uncontrolled_params()
            });
        provide_wiring(&root);

        let (one, item_one) = make_item("1");
        let (two, item_two) = make_item("2");
        let (three, item_three) = make_item("3");
        let (four, item_four) = make_item("4");
        run_microtasks(1).await;

        focus_arbitrary(&one, &item_one);
        key_down_on(&root, &one, "ArrowDown").await;
        key_down_on(&root, &two, "ArrowDown").await;
        assert_eq!(root.highlighted_index.get_untracked(), 2);
        assert!(is_highlighted(&item_three));

        detach(&item_one);
        run_microtasks(1).await;

        assert_eq!(
            root.highlighted_index.get_untracked(),
            1,
            "c followed the shift"
        );
        assert!(
            is_highlighted(&item_three),
            "the same element holds the tab stop"
        );
        assert!(!is_highlighted(&item_two));
        assert!(!is_highlighted(&item_four));
        key_down_on(&root, &three, "ArrowDown").await;
        run_microtasks(1).await;
        assert_eq!(
            focus_label_of(&four),
            Some("4".to_string()),
            "navigation continues from the item that holds the tab stop"
        );
        let _ = (&item_two, &item_three, &item_four);

        // Removing the highlighted item moves the tab stop back into range (first item)
        // (`:1105-1133`).
        let owner = Owner::new();
        owner.set();
        let direction = Memo::new(|_| TextDirection::Ltr);
        let root =
            use_composite_root::<Metadata, RwSignal<Option<i32>>, _>(UseCompositeRootParams {
                highlighted_index: None,
                direction,
                ..uncontrolled_params()
            });
        provide_wiring(&root);

        let (one, item_one) = make_item("1");
        let (two, item_two) = make_item("2");
        let (three, item_three) = make_item("3");
        run_microtasks(1).await;

        focus_arbitrary(&one, &item_one);
        key_down_on(&root, &one, "ArrowDown").await;
        key_down_on(&root, &two, "ArrowDown").await;
        assert!(is_highlighted(&item_three));

        detach(&item_three);
        run_microtasks(1).await;

        assert_eq!(
            root.highlighted_index.get_untracked(),
            0,
            "the tab stop moved back into range"
        );
        assert!(is_highlighted(&item_one));
        assert!(!is_highlighted(&item_two));
        let _ = (&item_one, &item_two, three);

        // The active item retains the tab stop over the first item when the highlighted
        // item is removed (`:1135-1163`).
        let owner = Owner::new();
        owner.set();
        let direction = Memo::new(|_| TextDirection::Ltr);
        let root =
            use_composite_root::<Metadata, RwSignal<Option<i32>>, _>(UseCompositeRootParams {
                highlighted_index: None,
                direction,
                ..uncontrolled_params()
            });
        provide_wiring(&root);

        let (one, item_one) = make_item("1");
        let (two, item_two) = make_item("2");
        let (three, item_three) = make_item("3");
        three
            .set_attribute("data-composite-item-active", "")
            .unwrap();
        let (four, item_four) = make_item("4");
        run_microtasks(1).await;

        assert_eq!(
            root.highlighted_index.get_untracked(),
            2,
            "the active item's index was adopted"
        );
        focus_arbitrary(&three, &item_three);
        key_down_on(&root, &three, "ArrowDown").await;
        assert!(is_highlighted(&item_four));

        detach(&item_four);
        run_microtasks(1).await;

        assert_eq!(
            root.highlighted_index.get_untracked(),
            2,
            "the active item regained the tab stop"
        );
        assert!(!is_highlighted(&item_one));
        assert!(is_highlighted(&item_three));
        let _ = (&item_one, &item_two, &item_three);

        // Items that cannot hold the tab stop are skipped (`:1165-1194`).
        let owner = Owner::new();
        owner.set();
        let direction = Memo::new(|_| TextDirection::Ltr);
        let root =
            use_composite_root::<Metadata, RwSignal<Option<i32>>, _>(UseCompositeRootParams {
                highlighted_index: None,
                direction,
                ..uncontrolled_params()
            });
        provide_wiring(&root);

        let (hidden, item_hidden) = make_item("1");
        hidden.style().set_property("display", "none").unwrap();
        let (aria_disabled, item_aria) = make_item("2");
        aria_disabled
            .set_attribute("aria-disabled", "true")
            .unwrap();
        let (regular, item_regular) = make_item("3");
        let (last, item_last) = make_item("4");
        run_microtasks(1).await;

        assert_eq!(
            root.highlighted_index.get_untracked(),
            2,
            "the initial tab stop skipped the hidden and aria-disabled items"
        );
        focus_arbitrary(&regular, &item_regular);
        key_down_on(&root, &regular, "ArrowDown").await;
        assert!(is_highlighted(&item_last));

        detach(&item_last);
        run_microtasks(1).await;

        assert_eq!(
            root.highlighted_index.get_untracked(),
            2,
            "the tab stop skipped the ineligible replacements"
        );
        assert!(!is_highlighted(&item_hidden));
        assert!(!is_highlighted(&item_aria));
        assert!(is_highlighted(&item_regular));
        let _ = (&item_hidden, &item_aria, &item_regular);

        // Nothing can hold the tab stop: the fallback keeps it in range (index 0)
        // (`:1196-1215`).
        let owner = Owner::new();
        owner.set();
        let direction = Memo::new(|_| TextDirection::Ltr);
        let root =
            use_composite_root::<Metadata, RwSignal<Option<i32>>, _>(UseCompositeRootParams {
                highlighted_index: None,
                direction,
                ..uncontrolled_params()
            });
        provide_wiring(&root);

        let (a, item_a) = make_item("1");
        a.set_attribute("aria-disabled", "true").unwrap();
        let (b, item_b) = make_item("2");
        b.set_attribute("aria-disabled", "true").unwrap();
        let (c, item_c) = make_item("3");
        run_microtasks(1).await;

        assert_eq!(
            root.highlighted_index.get_untracked(),
            2,
            "the only eligible item took the tab stop"
        );

        detach(&item_c);
        run_microtasks(1).await;

        assert_eq!(
            root.highlighted_index.get_untracked(),
            0,
            "the tab stop stayed in range when no item can hold it"
        );
        assert!(is_highlighted(&item_a));
        assert!(!is_highlighted(&item_b));
        let _ = (&item_a, &item_b);
    }

    // Mirrors `CompositeRoot.test.tsx:538-601` — grid navigation: arrows move by row and
    // column with wrap-around, driven through the root's keydown with the
    // `gridNavigation({ cols: 3 })` navigator.
    #[wasm_bindgen_test]
    async fn grid_navigation_moves_by_row_and_wraps() {
        init_executor();
        let harness = build_harness(9, false, |params| {
            params.grid = Some(grid_navigation(CompositeGridConfig {
                cols: 3,
                dense: false,
                item_sizes: None,
            }));
        })
        .await;
        harness.focus_item(0);

        harness.key_down(0, "ArrowDown", &[]).await;
        assert_eq!(harness.tabindex(3), "0", "ArrowDown moved a full row");
        harness.key_down(3, "ArrowDown", &[]).await;
        assert_eq!(harness.tabindex(6), "0");
        harness.key_down(6, "ArrowDown", &[]).await;
        assert_eq!(
            harness.tabindex(0),
            "0",
            "ArrowDown from the last row wrapped to the same column in the first row"
        );

        harness.key_down(0, "ArrowRight", &[]).await;
        assert_eq!(harness.tabindex(1), "0");
        harness.key_down(1, "ArrowRight", &[]).await;
        assert_eq!(harness.tabindex(2), "0");
        harness.key_down(2, "ArrowRight", &[]).await;
        assert_eq!(
            harness.tabindex(0),
            "0",
            "ArrowRight wrapped to the row start"
        );

        // ArrowDown from the last cell wraps to the same column in the first row
        // (`CompositeRoot.test.tsx:594-600` region).
        harness.focus_item(8);
        harness.key_down(8, "ArrowDown", &[]).await;
        assert_eq!(harness.tabindex(2), "0");
    }

    // Pins the item hook's roving tabindex + focus-follow + hover focus
    // (`useCompositeItem.ts:26-42`; the roving matrix asserted through the `CompositeRoot`
    // suite, the hover arm documented untested upstream — implementation spec item 6).
    #[wasm_bindgen_test]
    async fn composite_item_roving_tabindex_and_hover_focus() {
        init_executor();
        // The hover arm needs `highlightItemOnHover` in the context, so this test builds
        // its wiring directly instead of through the default harness.
        let owner = Owner::new();
        owner.set();

        let direction = Memo::new(|_| TextDirection::Ltr);
        let root =
            use_composite_root::<Metadata, RwSignal<Option<i32>>, _>(UseCompositeRootParams {
                orientation: None,
                grid: None,
                loop_focus: true,
                on_loop: None,
                highlighted_index: None,
                on_highlighted_index_change: None,
                direction,
                root_ref: InputRef::Empty,
                enable_home_and_end_keys: false,
                stop_event_propagation: false,
                disabled_indices: None,
                modifier_keys: Vec::new(),
            });
        let pump = root.on_map_change.clone();
        let _list =
            provide_composite_list::<Metadata>(Rc::clone(&root.elements_ref), None, move |map| {
                pump(map);
            });
        let on_hic = root.on_highlighted_index_change.clone();
        provide_composite_root_context(CompositeRootContextValue {
            highlighted_index: root.highlighted_index.clone(),
            on_highlighted_index_change: Rc::new(move |index, should_scroll| {
                on_hic.call((index, should_scroll));
            }),
            highlight_item_on_hover: true,
            relay_keyboard_event: root.relay_keyboard_event.clone(),
        });

        let a = document()
            .create_element("div")
            .unwrap()
            .unchecked_into::<HtmlElement>();
        document().body().unwrap().append_child(&a).unwrap();
        let b = document()
            .create_element("div")
            .unwrap()
            .unchecked_into::<HtmlElement>();
        document().body().unwrap().append_child(&b).unwrap();
        let item_a = use_composite_item::<Metadata>(UseCompositeItemParams { metadata: None });
        let item_b = use_composite_item::<Metadata>(UseCompositeItemParams { metadata: None });
        if let Some(set_ref) = &item_a.composite_ref {
            set_ref(Some(&a));
        }
        if let Some(set_ref) = &item_b.composite_ref {
            set_ref(Some(&b));
        }
        // The roving tabindex binding the view layer performs per render
        // (`useCompositeItem.ts:27`) — and what makes the divs focusable for the hover
        // arm below.
        a.set_attribute(
            "tabindex",
            &item_a.composite_props.tab_index.get_untracked().to_string(),
        )
        .unwrap();
        b.set_attribute(
            "tabindex",
            &item_b.composite_props.tab_index.get_untracked().to_string(),
        )
        .unwrap();
        run_microtasks(1).await;

        // Roving values follow the highlight.
        assert_eq!(item_a.composite_props.tab_index.get_untracked(), 0);
        assert_eq!(item_b.composite_props.tab_index.get_untracked(), -1);

        // The item's onFocus moves the highlight (`useCompositeItem.ts:28-30`).
        let focus_init = web_sys::FocusEventInit::new();
        focus_init.set_bubbles(true);
        let focus_event =
            web_sys::FocusEvent::new_with_focus_event_init_dict("focusin", &focus_init).unwrap();
        b.dispatch_event(focus_event.as_ref()).unwrap();
        (item_b.composite_props.on_focus)(focus_event.unchecked_ref::<FocusEvent>());
        assert_eq!(root.highlighted_index.get_untracked(), 1);
        assert_eq!(item_b.is_highlighted.get_untracked(), true);
        assert_eq!(item_a.composite_props.tab_index.get_untracked(), -1);

        // The hover arm focuses the hovered item: mouse move on the not-highlighted,
        // not-disabled item (`useCompositeItem.ts:31-41`).
        let mouse = web_sys::MouseEvent::new("mousemove").unwrap();
        a.dispatch_event(mouse.as_ref()).unwrap();
        (item_a.composite_props.on_mouse_move)(&mouse);
        let active = document().active_element().unwrap();
        assert_eq!(
            active.unchecked_ref::<Element>() == a.unchecked_ref::<Element>(),
            true,
            "mousemove focused the hovered item"
        );

        // An aria-disabled hovered item is not focused.
        b.set_attribute("aria-disabled", "true").unwrap();
        let mouse = web_sys::MouseEvent::new("mousemove").unwrap();
        b.dispatch_event(mouse.as_ref()).unwrap();
        (item_b.composite_props.on_mouse_move)(&mouse);
        assert_eq!(
            document()
                .active_element()
                .map(|active| active.unchecked_ref::<Element>() == a.unchecked_ref::<Element>()),
            Some(true),
            "the aria-disabled item was not focused"
        );
    }
}
