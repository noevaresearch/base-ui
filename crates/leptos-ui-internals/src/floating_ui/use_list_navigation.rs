//! Port of `packages/react/src/floating-ui-react/hooks/useListNavigation.ts` — the
//! arrow-key navigation hook that moves real or virtual focus through a list of items
//! (`specs/library/floating-ui-react/implementation.md`, "Notable per-hook state
//! machines", "List navigation").
//!
//! ## Rust adaptations
//!
//! - `indexRef` is the internal truth mirror exactly as upstream: the consumer's
//!   `activeIndex` is reconciled into it by the reactive re-run of the active-index
//!   sync effect, and `onNavigate` always reports `indexRef`
//!   (`hooks/useListNavigation.ts:292-298`).
//! - The upstream dependency arrays become reactive reads: `open`,
//!   `floatingElement`, `activeIndex`, and `selectedIndex` are read through the
//!   store's `useState` signals / reactive prop sources inside each effect, so a
//!   change re-runs the effect the way a re-render re-runs a layout effect. The
//!   `useValueAsRef` mirrors that exist to keep effect deps stable
//!   (`disabledIndicesRef`/`selectedIndexRef`/`resetOnPointerLeaveRef`,
//!   `hooks/useListNavigation.ts:306-309`) collapse to the values themselves — the
//!   port's hook runs once, so there is no re-subscription to suppress (the
//!   `use_typeahead` snapshot convention); `latestOpenRef.current` reads land on
//!   `store.select('open')`, the same event-time-fresh read every other ported hook
//!   uses — the ref's one-render staleness is unobservable at event time because no
//!   consumer handler runs between `setOpen` and the re-render that would refresh it.
//! - `previousOpenRef`/`previousMountedRef` are written by a **separate, later**
//!   effect upstream (the no-deps effect at `hooks/useListNavigation.ts:494-497`),
//!   so every consumer reads the *previous* render's value during its own run. The
//!   port gives each consuming effect its own previous-value snapshot (updated at
//!   the end of each of its runs, seeded with the hook-time values) instead of
//!   shared refs, because reactive effect run order is not the declaration order
//!   React guarantees — the observable semantics (each effect sees the pre-change
//!   value on the run where the value changed) are preserved per effect.
//! - `focusItemOnOpen` (`boolean | 'auto'`, `:134`) becomes [`FocusItemOnOpen`]; its
//!   truthiness (`'auto'` and `true` are truthy, `false` is not) and the strict
//!   `=== true` check port to [`FocusItemOnOpen::truthy`]/[`FocusItemOnOpen::is_true`].
//! - `getParentOrientation()` (`:520-526`) can return `undefined` upstream (no
//!   `parentOrientation` prop, no parent node, or the parent's dataRef has no
//!   orientation yet), and the orientation-key classifiers' `switch` then hits the
//!   `default` arm (`vertical || horizontal`) — the key helpers take
//!   `Option<Orientation>` so `None` reproduces the default arm.
//! - `event.which === 229` (`:540`) is the IME composition keyCode — the port reads
//!   `KeyboardEvent::key_code()`.
//! - `isStationaryWebKitPointer` (`:38-40`) gates on `platform.engine.webkit` and the
//!   zero-delta `movementX/movementY` pair, the WebKit scroll-under-pointer bug
//!   workaround (mui/base-ui#4002).
//! - `enqueueFocus`'s cancel closure rides [`EventUnsubscribe`]
//!   (`cancelQueuedFocusRef`, `:304,319-322,728-729`).
//! - The wait-for-list-populated retry (bounded `queueMicrotask` → rAF,
//!   `:421-450`) recurses through a free function so the scheduler choice survives
//!   the recursion; the microtask lands on `window.queueMicrotask` and the frames on
//!   the `useAnimationFrame` port (`waitForListPopulatedFrame`, `:312`).
//! - `scrollIntoView?.({ block: 'nearest', inline: 'nearest' })` (`:353-356`) keeps
//!   its options — `Element.scrollIntoView` with `ScrollIntoViewOptions` (the
//!   optional-chaining guard is jsdom-only upstream; the browser always has the
//!   method).
//! - The returned `aria-activedescendant` (`:756-764`) is a **non-handler bag
//!   member** — it ports to [`ElementHandlers::attributes`] with a lazily-resolved
//!   value (see the `element_props` module docs), not a frozen string, because the
//!   port's hook runs once and the value depends on the reactive
//!   `open`/`activeIndex`. An absent `id` yields no attribute (upstream would
//!   stringify `undefined`; every real consumer passes the id).
//! - The `item`-bag `shouldScrollIntoView` reads `item && (…)` (`:348-350`) where
//!   `item` is the always-truthy memoized props object — the port drops the vacuous
//!   operand.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::traits::{Get, GetUntracked};
use reactive_graph::wrappers::read::Signal;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{
    Element, Event, FocusEvent, HtmlElement, KeyboardEvent, MouseEvent, PointerEvent,
    ScrollIntoViewOptions, ScrollLogicalPosition,
};

use leptos_ui_utils::owner::owner_document;
use leptos_ui_utils::platform::platform;
use leptos_ui_utils::shadow_dom::{active_element, contains, get_target};
use leptos_ui_utils::use_animation_frame::use_animation_frame;
use leptos_ui_utils::use_iso_layout_effect::use_iso_layout_effect;
use leptos_ui_utils::use_value_as_ref::{ValueAsRef, use_value_as_ref};
use leptos_ui_utils::warn;

use crate::floating_ui::composite::{
    DisabledIndices, FindNonDisabledListIndexOptions, find_non_disabled_list_index,
    get_max_list_index, get_min_list_index, is_index_out_of_list_bounds,
};
use crate::floating_ui::constants::{ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP};
use crate::floating_ui::element;
use crate::floating_ui::element_props::{
    ElementAttributeFn, ElementEventHandler, ElementHandlers, ElementProps, FloatingContextSource,
};
use crate::floating_ui::enqueue_focus::{EnqueueFocusOptions, enqueue_focus};
use crate::floating_ui::event::{is_virtual_click, is_virtual_pointer_event, stop_event};
use crate::floating_ui::floating_root_store::{FloatingRootStore, selectors};
use crate::floating_ui::grid_navigation::{GridNavigationFn, ListRef};
use crate::floating_ui::reasons;
use crate::floating_ui::tree::{
    SharedFloatingTreeStore, use_floating_parent_node_id, use_floating_tree,
};
use crate::floating_ui::types::{
    ContextData, EventUnsubscribe, FloatingTreeEvent, Orientation, RootOpenChangeEventDetails,
};

/// `ESCAPE` (`useListNavigation.ts:33`).
pub const ESCAPE: &str = "Escape";

/// The consumer-owned item-element list — upstream's
/// `React.RefObject<Array<HTMLElement | null>>` (`useListNavigation.ts:104`); the
/// same shape [`crate::floating_ui::grid_navigation`] takes. A `None` slot is
/// upstream's `null` entry.
pub use crate::floating_ui::grid_navigation::ListRef as ItemListRef;

/// `onNavigate` (`useListNavigation.ts:115-116`): the reported active index (`None`
/// is upstream's `null`) and the originating event when there is one (upstream's
/// optional `React.SyntheticEvent`).
pub type OnNavigateFn = Rc<dyn Fn(Option<i32>, Option<&Event>)>;

/// Port of `UseListNavigationProps['focusItemOnOpen']` (`useListNavigation.ts:134`):
/// `'auto'` infers from the input type, a boolean forces it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FocusItemOnOpen {
    /// `'auto'` (the default).
    #[default]
    Auto,
    /// `true`.
    True,
    /// `false`.
    False,
}

impl FocusItemOnOpen {
    /// The JS truthiness of the value — `'auto'` and `true` are truthy
    /// (`focusItemOnOpenRef.current` gates, `useListNavigation.ts:373,418`).
    pub fn truthy(self) -> bool {
        !matches!(self, FocusItemOnOpen::False)
    }

    /// The strict `=== true` check (`useListNavigation.ts:419`).
    pub fn is_true(self) -> bool {
        matches!(self, FocusItemOnOpen::True)
    }
}

/// Port of `UseListNavigationProps` (`useListNavigation.ts:99-222`) with the
/// documented defaults. No `Default`: `list_ref` and `active_index` are upstream's
/// required props; [`UseListNavigationProps::new`] seeds the defaulted fields.
pub struct UseListNavigationProps<A, S> {
    /// `listRef` (`:104`).
    pub list_ref: ListRef,
    /// `activeIndex` (`:110`) — the consumer-controlled active index.
    pub active_index: A,
    /// `onNavigate` (`:115-116`).
    pub on_navigate: Option<OnNavigateFn>,
    /// `enabled` (`:122` — default `true`).
    pub enabled: bool,
    /// `selectedIndex` (`:127` — default `null`).
    pub selected_index: S,
    /// `focusItemOnOpen` (`:134` — default `'auto'`).
    pub focus_item_on_open: FocusItemOnOpen,
    /// `focusItemOnHover` (`:139` — default `true`).
    pub focus_item_on_hover: bool,
    /// `openOnArrowKeyDown` (`:145` — default `true`).
    pub open_on_arrow_key_down: bool,
    /// `disabledIndices` (`:156`).
    pub disabled_indices: Option<DisabledIndices>,
    /// `allowEscape` (`:165` — default `false`).
    pub allow_escape: bool,
    /// `loopFocus` (`:171` — default `false`).
    pub loop_focus: bool,
    /// `nested` (`:177` — default `false`).
    pub nested: bool,
    /// `parentOrientation` (`:184`).
    pub parent_orientation: Option<Orientation>,
    /// `rtl` (`:190` — default `false`).
    pub rtl: bool,
    /// `virtual` (`:199` — default `false`).
    pub is_virtual: bool,
    /// `orientation` (`:204` — default `'vertical'`).
    pub orientation: Orientation,
    /// `id` (`:208`) — the `aria-activedescendant` prefix in virtual mode.
    pub id: Option<String>,
    /// `resetOnPointerLeave` (`:213` — default `true`).
    pub reset_on_pointer_leave: bool,
    /// `externalTree` (`:217`).
    pub external_tree: Option<SharedFloatingTreeStore>,
    /// `grid` (`:221`) — the injected two-dimensional navigator.
    pub grid: Option<GridNavigationFn>,
}

impl<A, S> UseListNavigationProps<A, S> {
    /// The required props plus every upstream default (`useListNavigation.ts:233-254`).
    pub fn new(list_ref: ListRef, active_index: A, selected_index: S) -> Self {
        Self {
            list_ref,
            active_index,
            on_navigate: None,
            enabled: true,
            selected_index,
            focus_item_on_open: FocusItemOnOpen::Auto,
            focus_item_on_hover: true,
            open_on_arrow_key_down: true,
            disabled_indices: None,
            allow_escape: false,
            loop_focus: false,
            nested: false,
            parent_orientation: None,
            rtl: false,
            is_virtual: false,
            orientation: Orientation::Vertical,
            id: None,
            reset_on_pointer_leave: true,
            external_tree: None,
            grid: None,
        }
    }
}

/// `isStationaryWebKitPointer` (`useListNavigation.ts:38-40`): WebKit fires
/// zero-delta `mousemove`/`pointermove` events when the list scrolls beneath a
/// stationary pointer — moving the highlight during keyboard navigation
/// (mui/base-ui#4002).
fn is_stationary_webkit_pointer(event: &MouseEvent) -> bool {
    platform().engine.webkit && event.movement_x() == 0 && event.movement_y() == 0
}

/// `doSwitch` (`useListNavigation.ts:42-55`): `undefined` (a missing parent
/// orientation) hits the `default` arm (see the module docs).
fn do_switch(orientation: Option<Orientation>, vertical: bool, horizontal: bool) -> bool {
    match orientation {
        Some(Orientation::Vertical) => vertical,
        Some(Orientation::Horizontal) => horizontal,
        Some(Orientation::Both) | None => vertical || horizontal,
    }
}

/// `isMainOrientationKey` (`useListNavigation.ts:57-61`).
pub fn is_main_orientation_key(key: &str, orientation: Option<Orientation>) -> bool {
    let vertical = key == ARROW_UP || key == ARROW_DOWN;
    let horizontal = key == ARROW_LEFT || key == ARROW_RIGHT;
    do_switch(orientation, vertical, horizontal)
}

/// `isMainOrientationToEndKey` (`useListNavigation.ts:63-73`).
pub fn is_main_orientation_to_end_key(
    key: &str,
    orientation: Option<Orientation>,
    rtl: bool,
) -> bool {
    let vertical = key == ARROW_DOWN;
    let horizontal = if rtl {
        key == ARROW_LEFT
    } else {
        key == ARROW_RIGHT
    };
    do_switch(orientation, vertical, horizontal) || key == "Enter" || key == " " || key == ""
}

/// `isCrossOrientationOpenKey` (`useListNavigation.ts:75-83`).
pub fn is_cross_orientation_open_key(
    key: &str,
    orientation: Option<Orientation>,
    rtl: bool,
) -> bool {
    let vertical = if rtl {
        key == ARROW_LEFT
    } else {
        key == ARROW_RIGHT
    };
    let horizontal = key == ARROW_DOWN;
    do_switch(orientation, vertical, horizontal)
}

/// `isCrossOrientationCloseKey` (`useListNavigation.ts:85-97`).
pub fn is_cross_orientation_close_key(
    key: &str,
    orientation: Orientation,
    rtl: bool,
    is_grid: bool,
) -> bool {
    let vertical = if rtl {
        key == ARROW_RIGHT
    } else {
        key == ARROW_LEFT
    };
    let horizontal = key == ARROW_UP;
    if orientation == Orientation::Both || (orientation == Orientation::Horizontal && is_grid) {
        return key == ESCAPE;
    }
    do_switch(Some(orientation), vertical, horizontal)
}

/// The wait-for-list-populated retry (`useListNavigation.ts:421-450`): a free
/// function so the microtask-then-rAF scheduler choice survives the recursion (see
/// the module docs). On success, computes the initial index (`disabledIndices`
/// deliberately omitted so attribute-disabled items are skipped on open even when
/// the consumer passes an empty `disabledIndices` array — mui/base-ui#2604), clears
/// the key mirror, and reports the navigation.
#[allow(clippy::too_many_arguments)]
fn wait_for_list_populated(
    list_ref: &ListRef,
    runs: &Rc<Cell<u32>>,
    wait_frame: &leptos_ui_utils::use_animation_frame::AnimationFrame,
    key_ref: &Rc<RefCell<Option<String>>>,
    index_ref: &Rc<Cell<i32>>,
    on_navigate: &Rc<dyn Fn(Option<&Event>)>,
    orientation: Orientation,
    rtl: bool,
    nested: bool,
) {
    let list_is_empty = list_ref.borrow().first().is_none();
    if list_is_empty {
        // Avoid letting the browser paint if possible on the first try,
        // otherwise use rAF. Don't try more than twice, since something is
        // wrong otherwise (`useListNavigation.ts:424-433`).
        if runs.get() < 2 {
            let is_first_run = runs.get() == 0;
            let list_ref = Rc::clone(list_ref);
            let runs = Rc::clone(runs);
            let retry_frame = wait_frame.clone();
            let key_ref = Rc::clone(key_ref);
            let index_ref = Rc::clone(index_ref);
            let on_navigate = Rc::clone(on_navigate);
            let retry = move || {
                wait_for_list_populated(
                    &list_ref,
                    &runs,
                    &retry_frame,
                    &key_ref,
                    &index_ref,
                    &on_navigate,
                    orientation,
                    rtl,
                    nested,
                );
            };
            if is_first_run {
                // `queueMicrotask` (`useListNavigation.ts:430`).
                if let Some(window) = web_sys::window() {
                    let function =
                        js_sys::Function::from(wasm_bindgen::closure::Closure::once_into_js(retry));
                    window.queue_microtask(&function);
                }
            } else {
                wait_frame.request(retry);
            }
        }
        runs.set(runs.get() + 1);
    } else {
        // Initially focus the first non-disabled item. `disabledIndices` is
        // deliberately omitted so attribute-disabled items (`disabled`/
        // `aria-disabled`) are skipped on open even when the consumer passes an
        // empty `disabledIndices` array. Passing it would regress that behavior
        // (see mui/base-ui#2604) (`useListNavigation.ts:435-447`).
        let key = key_ref.borrow().clone();
        let to_end = key
            .as_deref()
            .map(|key| is_main_orientation_to_end_key(key, Some(orientation), rtl))
            .unwrap_or(false);
        let list = list_ref.borrow();
        index_ref.set(if key.is_none() || to_end || nested {
            get_min_list_index(&list, None)
        } else {
            get_max_list_index(&list, None)
        });
        drop(list);
        *key_ref.borrow_mut() = None;
        on_navigate(None);
    }
}

/// Port of `useListNavigation(context, props)` (`useListNavigation.ts:229-941`).
/// Must be called inside a reactive owner (the effects and the animation-frame
/// handles register cleanups). Returns the four handler bags, or the empty props
/// when disabled.
pub fn use_list_navigation<A, S>(
    context: impl Into<FloatingContextSource>,
    props: UseListNavigationProps<A, S>,
) -> ElementProps
where
    A: Get<Value = Option<i32>> + GetUntracked<Value = Option<i32>> + Clone + 'static,
    S: Get<Value = Option<i32>> + GetUntracked<Value = Option<i32>> + Clone + 'static,
{
    let UseListNavigationProps {
        list_ref,
        active_index,
        on_navigate: on_navigate_prop,
        enabled,
        selected_index,
        focus_item_on_open,
        focus_item_on_hover,
        open_on_arrow_key_down,
        disabled_indices,
        allow_escape,
        loop_focus,
        nested,
        parent_orientation,
        rtl,
        is_virtual: virtual_,
        orientation,
        id,
        reset_on_pointer_leave,
        external_tree,
        grid: navigate_grid,
    } = props;

    let is_grid = navigate_grid.is_some();

    if allow_escape {
        if !loop_focus {
            warn().log(&["`useListNavigation` looping must be enabled to allow escaping."]);
        }

        if !virtual_ {
            warn().log(&["`useListNavigation` must be virtual to allow escaping."]);
        }
    }

    if orientation == Orientation::Vertical && is_grid {
        warn().log(&[
            "In grid list navigation mode, the `orientation` should",
            "be either \"horizontal\" or \"both\".",
        ]);
    }

    let store: Rc<FloatingRootStore> = context.into().root_store();
    let inner = store.rc();

    // `store.useState('open' | 'floatingElement' | 'domReferenceElement')`
    // (`useListNavigation.ts:278-280`).
    let open = inner.use_state(selectors::open);
    let floating_element = inner.use_state(selectors::floating_element);
    let dom_reference_element = inner.use_state(selectors::dom_reference_element);

    // `const dataRef = store.context.dataRef` (`useListNavigation.ts:282`).
    let data_ref: Rc<RefCell<ContextData>> = Rc::clone(&store.context.data_ref);

    // `const parentId = useFloatingParentNodeId(); const tree =
    // useFloatingTree(externalTree)` (`useListNavigation.ts:288-289`).
    let parent_id = use_floating_parent_node_id();
    let tree = use_floating_tree(external_tree);

    // The mirrors (`useListNavigation.ts:291-304`), seeded per upstream.
    let focus_item_on_open_ref: Rc<RefCell<FocusItemOnOpen>> =
        Rc::new(RefCell::new(focus_item_on_open));
    let index_ref: Rc<Cell<i32>> = Rc::new(Cell::new(selected_index.get_untracked().unwrap_or(-1)));
    let key_ref: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let is_pointer_modality_ref: Rc<Cell<bool>> = Rc::new(Cell::new(true));

    let on_navigate: Rc<dyn Fn(Option<&Event>)> = {
        let on_navigate_prop = on_navigate_prop.clone();
        let index_ref = Rc::clone(&index_ref);
        // `useStableCallback` wrapper that always reports `indexRef`
        // (`useListNavigation.ts:296-298`).
        Rc::new(move |event: Option<&Event>| {
            let index = index_ref.get();
            if let Some(on_navigate_prop) = &on_navigate_prop {
                on_navigate_prop(if index == -1 { None } else { Some(index) }, event);
            }
        })
    };

    // The hook-time seeds of the per-effect previous-value snapshots (see the
    // module docs): upstream seeds `previousMountedRef` with `!!floatingElement`
    // (`useListNavigation.ts:300`) and `previousOpenRef` with `open` (`:301`).
    let floating_element_at_hook = floating_element.get_untracked();
    let open_at_hook = open.get_untracked();

    let force_sync_focus_ref: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let force_scroll_into_view_ref: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let cancel_queued_focus_ref: Rc<RefCell<Option<EventUnsubscribe>>> =
        Rc::new(RefCell::new(None));

    // `focusFrame`/`waitForListPopulatedFrame` (`useListNavigation.ts:311-312`).
    let focus_frame = use_animation_frame();
    let wait_frame = use_animation_frame();

    // `floatingFocusElementRef = useValueAsRef(getFloatingFocusElement(floatingElement))`
    // (`useListNavigation.ts:284-286`).
    let floating_focus_element_signal = Signal::derive({
        let floating_element = floating_element.clone();
        move || element::get_floating_focus_element(floating_element.get_untracked().as_ref())
    });
    let floating_focus_element_ref: ValueAsRef<Option<Element>> =
        use_value_as_ref(floating_focus_element_signal);

    // `typeableComboboxReference` (`useListNavigation.ts:285`) is re-derived at
    // event time where the handlers read it; the floating bag's attribute merge
    // uses the hook-time value (the bag is built once, per the port's
    // construction-once model).
    let typeable_combobox_reference_at_hook =
        element::is_typeable_combobox(dom_reference_element.get_untracked().as_ref());

    // `dataRef.current.orientation = orientation` (`useListNavigation.ts:360-362`).
    {
        let data_ref = Rc::clone(&data_ref);
        use_iso_layout_effect(move || {
            data_ref.borrow_mut().orientation = Some(orientation);
        });
    }

    // Sync `selectedIndex` to be the `activeIndex` upon opening the floating
    // element. Also, reset `activeIndex` upon closing the floating element
    // (`useListNavigation.ts:366-386`).
    {
        let open = open.clone();
        let floating_element = floating_element.clone();
        let selected_index = selected_index.clone();
        let index_ref = Rc::clone(&index_ref);
        let focus_item_on_open_ref = Rc::clone(&focus_item_on_open_ref);
        let force_scroll_into_view_ref = Rc::clone(&force_scroll_into_view_ref);
        let on_navigate = Rc::clone(&on_navigate);
        let previous_mounted = Rc::new(Cell::new(floating_element_at_hook.is_some()));
        use_iso_layout_effect(move || {
            let open_value = open.get();
            let floating_value = floating_element.get();
            let selected_value = selected_index.get();
            if open_value && floating_value.is_some() {
                index_ref.set(selected_value.unwrap_or(-1));
                if focus_item_on_open_ref.borrow().truthy() && selected_value.is_some() {
                    // Regardless of the pointer modality, we want to ensure the
                    // selected item comes into view when the floating element is
                    // opened (`useListNavigation.ts:374-377`).
                    force_scroll_into_view_ref.set(true);
                    on_navigate(None);
                }
            } else if previous_mounted.get() {
                // Reset the active index when the list is no longer open and
                // mounted (closing or unmounting) (`useListNavigation.ts:379-385`).
                index_ref.set(-1);
                on_navigate(None);
            }
            previous_mounted.set(floating_value.is_some());
        });
    }

    // `focusItem` (`useListNavigation.ts:314-358`).
    let focus_item: Rc<dyn Fn()> = {
        let list_ref = Rc::clone(&list_ref);
        let index_ref = Rc::clone(&index_ref);
        let tree = tree.clone();
        let force_sync_focus_ref = Rc::clone(&force_sync_focus_ref);
        let force_scroll_into_view_ref = Rc::clone(&force_scroll_into_view_ref);
        let is_pointer_modality_ref = Rc::clone(&is_pointer_modality_ref);
        let cancel_queued_focus_ref = Rc::clone(&cancel_queued_focus_ref);
        let focus_frame = focus_frame.clone();
        Rc::new(move || {
            let run_focus = {
                let tree = tree.clone();
                let force_sync_focus_ref = Rc::clone(&force_sync_focus_ref);
                let cancel_queued_focus_ref = Rc::clone(&cancel_queued_focus_ref);
                Rc::new(move |item: &Element| {
                    if virtual_ {
                        if let Some(tree) = tree.as_ref() {
                            tree.events.emit(
                                "virtualfocus",
                                &FloatingTreeEvent::VirtualFocus(item.clone()),
                            );
                        }
                    } else {
                        let item: HtmlElement = item.clone().dyn_into().unwrap();
                        let cancel = enqueue_focus(
                            Some(&item),
                            EnqueueFocusOptions {
                                sync: Some(force_sync_focus_ref.get()),
                                prevent_scroll: Some(true),
                                ..Default::default()
                            },
                        );
                        *cancel_queued_focus_ref.borrow_mut() = Some(cancel);
                    }
                })
            };

            let initial_item = list_ref
                .borrow()
                .get(index_ref.get() as usize)
                .cloned()
                .flatten();

            if let Some(initial_item) = &initial_item {
                run_focus(initial_item);
            }

            let force_sync = force_sync_focus_ref.get();
            let scheduler_frame = focus_frame.clone();
            let run_list_ref = Rc::clone(&list_ref);
            let run_index_ref = Rc::clone(&index_ref);
            let run_initial_item = initial_item.clone();
            let run_run_focus = Rc::clone(&run_focus);
            let run_force_scroll_into_view_ref = Rc::clone(&force_scroll_into_view_ref);
            let run_is_pointer_modality_ref = Rc::clone(&is_pointer_modality_ref);
            let run_after = move || {
                let waited_item = run_list_ref
                    .borrow()
                    .get(run_index_ref.get() as usize)
                    .cloned()
                    .flatten()
                    .or_else(|| run_initial_item.clone());

                let Some(waited_item) = waited_item else {
                    return;
                };

                if run_initial_item.is_none() {
                    run_run_focus(&waited_item);
                }

                // `item && (forceScrollIntoView || !isPointerModalityRef.current)`
                // (`useListNavigation.ts:348-350`) — `item` is the always-truthy
                // memoized props object (see the module docs).
                let should_scroll_into_view =
                    run_force_scroll_into_view_ref.get() || !run_is_pointer_modality_ref.get();

                if should_scroll_into_view {
                    // JSDOM doesn't support `.scrollIntoView()` but it's widely
                    // supported by all browsers (`useListNavigation.ts:353-356`).
                    let options = ScrollIntoViewOptions::new();
                    options.set_block(ScrollLogicalPosition::Nearest);
                    options.set_inline(ScrollLogicalPosition::Nearest);
                    let _: Element = waited_item.clone().dyn_into().unwrap();
                    waited_item.scroll_into_view_with_scroll_into_view_options(&options);
                }
            };

            if force_sync {
                run_after();
            } else {
                scheduler_frame.request(run_after);
            }
        })
    };

    // Sync `activeIndex` to be the focused item while the floating element is open
    // (`useListNavigation.ts:390-470`).
    {
        let open = open.clone();
        let floating_element = floating_element.clone();
        let active_index = active_index.clone();
        let selected_index = selected_index.clone();
        let list_ref = Rc::clone(&list_ref);
        let index_ref = Rc::clone(&index_ref);
        let key_ref = Rc::clone(&key_ref);
        let focus_item_on_open_ref = Rc::clone(&focus_item_on_open_ref);
        let force_sync_focus_ref = Rc::clone(&force_sync_focus_ref);
        let force_scroll_into_view_ref = Rc::clone(&force_scroll_into_view_ref);
        let on_navigate = Rc::clone(&on_navigate);
        let focus_item = Rc::clone(&focus_item);
        let wait_frame = wait_frame.clone();
        let previous_mounted = Rc::new(Cell::new(floating_element_at_hook.is_some()));
        let previous_open = Rc::new(Cell::new(open_at_hook));
        use_iso_layout_effect(move || {
            let open_value = open.get();
            let floating_value = floating_element.get();
            let active_value = active_index.get();
            'body: {
                if !enabled {
                    break 'body;
                }
                if !open_value {
                    force_sync_focus_ref.set(false);
                    break 'body;
                }
                if floating_value.is_none() {
                    break 'body;
                }

                match active_value {
                    None => {
                        force_sync_focus_ref.set(false);

                        if selected_index.get_untracked().is_some() {
                            break 'body;
                        }

                        // Reset while the floating element was open (e.g. the list
                        // changed) (`useListNavigation.ts:409-413`).
                        if previous_mounted.get() {
                            index_ref.set(-1);
                            focus_item();
                        }

                        // Initial sync (`useListNavigation.ts:415-451`).
                        if (!previous_open.get() || !previous_mounted.get())
                            && focus_item_on_open_ref.borrow().truthy()
                            && (key_ref.borrow().is_some()
                                || (focus_item_on_open_ref.borrow().is_true()
                                    && key_ref.borrow().is_none()))
                        {
                            let runs = Rc::new(Cell::new(0));
                            wait_for_list_populated(
                                &list_ref,
                                &runs,
                                &wait_frame,
                                &key_ref,
                                &index_ref,
                                &on_navigate,
                                orientation,
                                rtl,
                                nested,
                            );
                        }
                    }
                    Some(active_value) => {
                        let in_bounds =
                            !is_index_out_of_list_bounds(&list_ref.borrow(), active_value);
                        if in_bounds {
                            index_ref.set(active_value);
                            focus_item();
                            force_scroll_into_view_ref.set(false);
                        }
                    }
                }
            }
            previous_mounted.set(floating_value.is_some());
            previous_open.set(open_value);
        });
    }

    // Ensure the parent floating element has focus when a nested child closes to
    // allow arrow key navigation to work after the pointer leaves the child
    // (`useListNavigation.ts:474-492`).
    {
        let tree = tree.clone();
        let parent_id = parent_id.clone();
        let floating_element = floating_element.clone();
        let dom_reference_element = dom_reference_element.clone();
        let previous_mounted = Rc::new(Cell::new(floating_element_at_hook.is_some()));
        let is_pointer_modality_ref = Rc::clone(&is_pointer_modality_ref);
        use_iso_layout_effect(move || {
            let floating_value = floating_element.get();
            let dom_reference_value = dom_reference_element.get();
            'body: {
                if !enabled
                    || floating_value.is_some()
                    || tree.is_none()
                    || virtual_
                    || !previous_mounted.get()
                {
                    break 'body;
                }

                let tree = tree.as_ref().unwrap();
                let nodes = tree.nodes.borrow();
                let parent = nodes
                    .iter()
                    .find(|node| node.id.as_deref() == parent_id.as_deref())
                    .and_then(|node| node.context.borrow().clone())
                    .and_then(|context| context.elements.floating.get_untracked());
                // `floatingElement` is null here (see the guard above), so resolve
                // the owner document from an in-DOM element for realm-safety
                // (shadow DOM/iframes): the reference element, falling back to the
                // parent floating element when the reference is virtual
                // (`domReferenceElement` is null) (`useListNavigation.ts:481-484`).
                let active_el = active_element(&owner_document(
                    dom_reference_value.as_deref().or(parent.as_deref()),
                ));
                let tree_contains_active_el = nodes.iter().any(|node| {
                    node.context
                        .borrow()
                        .as_ref()
                        .and_then(|context| context.elements.floating.get_untracked())
                        .map(|floating| contains(Some(&floating), active_el.as_ref()))
                        .unwrap_or(false)
                });

                if let Some(parent) =
                    parent.filter(|_| !tree_contains_active_el && is_pointer_modality_ref.get())
                {
                    if let Ok(parent) = parent.dyn_into::<HtmlElement>() {
                        let options = web_sys::FocusOptions::new();
                        options.set_prevent_scroll(true);
                        let _ = parent.focus_with_options(&options);
                    }
                }
            }
            previous_mounted.set(floating_value.is_some());
        });
    }

    // The closed-state resets (`useListNavigation.ts:499-504`).
    {
        let open = open.clone();
        let key_ref = Rc::clone(&key_ref);
        let focus_item_on_open_ref = Rc::clone(&focus_item_on_open_ref);
        use_iso_layout_effect(move || {
            if !open.get() {
                *key_ref.borrow_mut() = None;
                *focus_item_on_open_ref.borrow_mut() = focus_item_on_open;
            }
        });
    }

    // `syncCurrentTarget` (`useListNavigation.ts:508-518`).
    let sync_current_target: Rc<dyn Fn(Option<&Event>)> = {
        let store = Rc::clone(&store);
        let list_ref = Rc::clone(&list_ref);
        let index_ref = Rc::clone(&index_ref);
        let active_index = active_index.clone();
        let on_navigate = Rc::clone(&on_navigate);
        Rc::new(move |event: Option<&Event>| {
            if !store.select(selectors::open) {
                return;
            }

            let current_target = event
                .and_then(|event| event.current_target())
                .and_then(|target| target.dyn_into::<Element>().ok());
            let Some(current_target) = current_target else {
                return;
            };
            let index = list_ref
                .borrow()
                .iter()
                .position(|element| element.as_ref() == Some(&current_target))
                .map(|index| index as i32);
            if let Some(index) = index {
                if index_ref.get() != index || active_index.get_untracked() != Some(index) {
                    index_ref.set(index);
                    on_navigate(event);
                }
            }
        })
    };

    // `getParentOrientation` (`useListNavigation.ts:520-526`).
    let get_parent_orientation: Rc<dyn Fn() -> Option<Orientation>> = {
        let tree = tree.clone();
        Rc::new(move || {
            parent_orientation.or_else(|| {
                tree.as_ref().and_then(|tree| {
                    tree.nodes
                        .borrow()
                        .iter()
                        .find(|node| node.id.as_deref() == parent_id.as_deref())
                        .and_then(|node| node.context.borrow().clone())
                        .and_then(|context| context.data_ref.borrow().orientation)
                })
            })
        })
    };

    // `getMinEnabledIndex` (`useListNavigation.ts:528-530`).
    let get_min_enabled_index: Rc<dyn Fn() -> i32> = {
        let list_ref = Rc::clone(&list_ref);
        let disabled_indices = disabled_indices.clone();
        Rc::new(move || {
            let list = list_ref.borrow();
            get_min_list_index(&list, disabled_indices.as_ref())
        })
    };

    // `commonOnKeyDown` (`useListNavigation.ts:532-688`).
    let common_on_key_down: ElementEventHandler<KeyboardEvent> = {
        let store = Rc::clone(&store);
        let list_ref = Rc::clone(&list_ref);
        let index_ref = Rc::clone(&index_ref);
        let is_pointer_modality_ref = Rc::clone(&is_pointer_modality_ref);
        let force_sync_focus_ref = Rc::clone(&force_sync_focus_ref);
        let on_navigate = Rc::clone(&on_navigate);
        let floating_focus_element_ref = floating_focus_element_ref.clone();
        let tree = tree.clone();
        let get_parent_orientation = Rc::clone(&get_parent_orientation);
        Rc::new(move |event: &KeyboardEvent| {
            is_pointer_modality_ref.set(false);
            force_sync_focus_ref.set(true);

            let key = event.key();

            // When composing a character, Chrome fires ArrowDown twice.
            // Firefox/Safari don't appear to suffer from this. `event.isComposing`
            // is avoided due to Safari not supporting it properly (although it's
            // not needed in the first place for Safari, just avoiding any possible
            // issues) (`useListNavigation.ts:536-542`).
            if event.key_code() == 229 {
                return;
            }

            // If the floating element is animating out, ignore navigation.
            // Otherwise, the `activeIndex` gets set to 0 despite not being open so
            // the next time the user ArrowDowns, the first item won't be focused
            // (`useListNavigation.ts:544-549`).
            if !store.select(selectors::open) {
                let current_target = event
                    .current_target()
                    .and_then(|target| target.dyn_into::<Element>().ok());
                if current_target.as_ref() == floating_focus_element_ref.current().as_ref() {
                    return;
                }
            }

            if nested && is_cross_orientation_close_key(&key, orientation, rtl, is_grid) {
                // If the nested list's close key is also the parent navigation key,
                // let the parent navigate. Otherwise, stop propagating the event
                // (`useListNavigation.ts:552-556`).
                if !is_main_orientation_key(&key, get_parent_orientation()) {
                    stop_event(event);
                }

                store.set_open(
                    false,
                    &RootOpenChangeEventDetails::new(
                        reasons::LIST_NAVIGATION,
                        event.clone().into(),
                        None,
                        String::new(),
                    ),
                );

                let dom_reference = store.select(selectors::dom_reference_element);
                if let Some(dom_reference) = dom_reference
                    .as_ref()
                    .and_then(|element| element.dyn_ref::<HtmlElement>().map(HtmlElement::clone))
                {
                    if virtual_ {
                        if let Some(tree) = tree.as_ref() {
                            tree.events.emit(
                                "virtualfocus",
                                &FloatingTreeEvent::VirtualFocus(dom_reference.into()),
                            );
                        }
                    } else {
                        let _ = dom_reference.focus();
                    }
                }

                return;
            }

            let current_index = index_ref.get();
            let (min_index, max_index) = {
                let list = list_ref.borrow();
                (
                    get_min_list_index(&list, disabled_indices.as_ref()),
                    get_max_list_index(&list, disabled_indices.as_ref()),
                )
            };

            let dom_reference = store.select(selectors::dom_reference_element);
            let typeable_combobox_reference = element::is_typeable_combobox(dom_reference.as_ref());

            if !typeable_combobox_reference {
                if key == "Home" {
                    stop_event(event);
                    index_ref.set(min_index);
                    on_navigate(Some(event));
                }

                if key == "End" {
                    stop_event(event);
                    index_ref.set(max_index);
                    on_navigate(Some(event));
                }
            }

            // Grid navigation is injected by grid-capable consumers so non-grid
            // consumers (menu, select) tree-shake the grid helpers
            // (`useListNavigation.ts:589-612`).
            if let Some(navigate_grid) = &navigate_grid {
                let index = navigate_grid(
                    event,
                    index_ref.get(),
                    &list_ref,
                    orientation,
                    loop_focus,
                    rtl,
                    disabled_indices.clone(),
                    min_index,
                    max_index,
                );

                if let Some(index) = index {
                    index_ref.set(index);
                    on_navigate(Some(event));
                }

                if orientation == Orientation::Both {
                    return;
                }
            }

            if is_main_orientation_key(&key, Some(orientation)) {
                stop_event(event);

                // Reset the index if no item is focused
                // (`useListNavigation.ts:617-628`).
                let current_target = event
                    .current_target()
                    .and_then(|target| target.dyn_into::<Element>().ok());
                let active_el = active_element(&owner_document(current_target.as_deref()));
                if store.select(selectors::open)
                    && !virtual_
                    && current_target.as_ref() == active_el.as_ref()
                {
                    index_ref.set(
                        if is_main_orientation_to_end_key(&key, Some(orientation), rtl) {
                            min_index
                        } else {
                            max_index
                        },
                    );
                    on_navigate(Some(event));
                    return;
                }

                if is_main_orientation_to_end_key(&key, Some(orientation), rtl) {
                    if loop_focus {
                        if current_index >= max_index {
                            if allow_escape && current_index != list_ref.borrow().len() as i32 {
                                index_ref.set(-1);
                            } else {
                                // Give time for virtualizers to update the listRef
                                // (`useListNavigation.ts:637-639`).
                                force_sync_focus_ref.set(false);
                                index_ref.set(min_index);
                            }
                        } else {
                            let next = {
                                let list = list_ref.borrow();
                                find_non_disabled_list_index(
                                    &list,
                                    FindNonDisabledListIndexOptions {
                                        starting_index: current_index,
                                        disabled_indices: disabled_indices.as_ref(),
                                        ..FindNonDisabledListIndexOptions::new(None)
                                    },
                                )
                            };
                            index_ref.set(next);
                        }
                    } else {
                        let next = {
                            let list = list_ref.borrow();
                            find_non_disabled_list_index(
                                &list,
                                FindNonDisabledListIndexOptions {
                                    starting_index: current_index,
                                    disabled_indices: disabled_indices.as_ref(),
                                    ..FindNonDisabledListIndexOptions::new(None)
                                },
                            )
                        };
                        index_ref.set(i32::min(max_index, next));
                    }
                } else if loop_focus {
                    if current_index <= min_index {
                        if allow_escape && current_index != -1 {
                            index_ref.set(list_ref.borrow().len() as i32);
                        } else {
                            // Give time for virtualizers to update the listRef
                            // (`useListNavigation.ts:660-662`).
                            force_sync_focus_ref.set(false);
                            index_ref.set(max_index);
                        }
                    } else {
                        let next = {
                            let list = list_ref.borrow();
                            find_non_disabled_list_index(
                                &list,
                                FindNonDisabledListIndexOptions {
                                    starting_index: current_index,
                                    decrement: true,
                                    disabled_indices: disabled_indices.as_ref(),
                                    ..FindNonDisabledListIndexOptions::new(None)
                                },
                            )
                        };
                        index_ref.set(next);
                    }
                } else {
                    let next = {
                        let list = list_ref.borrow();
                        find_non_disabled_list_index(
                            &list,
                            FindNonDisabledListIndexOptions {
                                starting_index: current_index,
                                decrement: true,
                                disabled_indices: disabled_indices.as_ref(),
                                ..FindNonDisabledListIndexOptions::new(None)
                            },
                        )
                    };
                    index_ref.set(i32::max(min_index, next));
                }

                if is_index_out_of_list_bounds(&list_ref.borrow(), index_ref.get()) {
                    index_ref.set(-1);
                }

                on_navigate(Some(event));
            }
        })
    };

    // The `item` bag (`useListNavigation.ts:690-754`).
    let item_handlers = ElementHandlers {
        on_focus: {
            let force_sync_focus_ref = Rc::clone(&force_sync_focus_ref);
            let sync_current_target = Rc::clone(&sync_current_target);
            Some(Rc::new(move |event: &FocusEvent| {
                force_sync_focus_ref.set(true);
                sync_current_target(Some(event));
            }) as ElementEventHandler<FocusEvent>)
        },
        on_click: {
            // Safari (`useListNavigation.ts:696`).
            Some(Rc::new(move |event: &MouseEvent| {
                if let Some(current_target) = event
                    .current_target()
                    .and_then(|target| target.dyn_into::<HtmlElement>().ok())
                {
                    let options = web_sys::FocusOptions::new();
                    options.set_prevent_scroll(true);
                    let _ = current_target.focus_with_options(&options);
                }
            }) as ElementEventHandler<MouseEvent>)
        },
        on_mouse_move: {
            let force_sync_focus_ref = Rc::clone(&force_sync_focus_ref);
            let force_scroll_into_view_ref = Rc::clone(&force_scroll_into_view_ref);
            let sync_current_target = Rc::clone(&sync_current_target);
            Some(Rc::new(move |event: &MouseEvent| {
                if is_stationary_webkit_pointer(event) {
                    return;
                }
                force_sync_focus_ref.set(true);
                force_scroll_into_view_ref.set(false);
                if focus_item_on_hover {
                    sync_current_target(Some(event));
                }
            }) as ElementEventHandler<MouseEvent>)
        },
        on_pointer_leave: {
            let store = Rc::clone(&store);
            let list_ref = Rc::clone(&list_ref);
            let index_ref = Rc::clone(&index_ref);
            let is_pointer_modality_ref = Rc::clone(&is_pointer_modality_ref);
            let force_sync_focus_ref = Rc::clone(&force_sync_focus_ref);
            let cancel_queued_focus_ref = Rc::clone(&cancel_queued_focus_ref);
            let floating_focus_element_ref = floating_focus_element_ref.clone();
            let on_navigate = Rc::clone(&on_navigate);
            Some(Rc::new(move |event: &PointerEvent| {
                if !store.select(selectors::open)
                    || !is_pointer_modality_ref.get()
                    || event.pointer_type() == "touch"
                {
                    return;
                }

                force_sync_focus_ref.set(true);

                let related_target = event
                    .related_target()
                    .and_then(|target| target.dyn_into::<Element>().ok());

                if !focus_item_on_hover
                    || related_target.as_ref().is_some_and(|related_target| {
                        list_ref
                            .borrow()
                            .iter()
                            .any(|element| element.as_ref() == Some(related_target))
                    })
                {
                    return;
                }

                if !reset_on_pointer_leave {
                    return;
                }

                if let Some(cancel) = cancel_queued_focus_ref.borrow_mut().take() {
                    cancel();
                }

                index_ref.set(-1);
                on_navigate(Some(event));

                if !virtual_ {
                    let floating_focus_el = floating_focus_element_ref.current();
                    let active_el = active_element(&owner_document(floating_focus_el.as_deref()));
                    if let Some(floating_focus_el) = &floating_focus_el {
                        if contains(Some(floating_focus_el), active_el.as_ref()) {
                            if let Ok(floating_focus_el) =
                                floating_focus_el.clone().dyn_into::<HtmlElement>()
                            {
                                let options = web_sys::FocusOptions::new();
                                options.set_prevent_scroll(true);
                                let _ = floating_focus_el.focus_with_options(&options);
                            }
                        }
                    }
                }
            }) as ElementEventHandler<PointerEvent>)
        },
        ..ElementHandlers::default()
    };

    // The `ariaActiveDescendantProp` (`useListNavigation.ts:756-764`) — the
    // lazily-resolved attribute member (see the module docs).
    let aria_active_descendant: ElementAttributeFn = {
        let open = open.clone();
        let active_index = active_index.clone();
        Rc::new(move || {
            let has_active_index = active_index.get_untracked().is_some();
            if !(virtual_ && open.get_untracked() && has_active_index) {
                return None;
            }
            let Some(id) = &id else {
                return None;
            };
            let active_index = active_index.get_untracked().unwrap();
            Some(format!("{id}-{active_index}"))
        })
    };

    // The `floating` bag (`useListNavigation.ts:766-808`).
    let floating_handlers = {
        let mut attributes: Vec<(String, ElementAttributeFn)> = Vec::new();
        if !typeable_combobox_reference_at_hook {
            attributes.push((
                "aria-activedescendant".to_owned(),
                Rc::clone(&aria_active_descendant),
            ));
        }
        ElementHandlers {
            attributes,
            on_key_down: {
                let store = Rc::clone(&store);
                let common_on_key_down = Rc::clone(&common_on_key_down);
                let floating_focus_element_ref = floating_focus_element_ref.clone();
                let dom_reference_element = dom_reference_element.clone();
                Some(Rc::new(move |event: &KeyboardEvent| {
                    let key = event.key();
                    // Close submenu on Shift+Tab (`useListNavigation.ts:769-788`).
                    if key == "Tab"
                        && event.shift_key()
                        && store.select(selectors::open)
                        && !virtual_
                    {
                        // If the event originated from within a nested element
                        // (e.g., a Dialog opened from within the menu), don't close
                        // the menu. The nested element has its own focus management
                        // and should handle the Tab key.
                        let target = get_target(event);
                        let target: Option<Element> =
                            target.and_then(|target| target.dyn_into::<Element>().ok());
                        let floating_focus_element = floating_focus_element_ref.current();
                        let within_floating_focus = match (&floating_focus_element, &target) {
                            (Some(floating), Some(target)) => {
                                contains(Some(floating), Some(target))
                            }
                            _ => false,
                        };
                        if target.is_some() && !within_floating_focus {
                            return;
                        }

                        stop_event(event);
                        store.set_open(
                            false,
                            &RootOpenChangeEventDetails::new(
                                reasons::FOCUS_OUT,
                                event.clone().into(),
                                None,
                                String::new(),
                            ),
                        );

                        if let Some(dom_reference) = dom_reference_element
                            .get_untracked()
                            .as_ref()
                            .and_then(|element| {
                                element.dyn_ref::<HtmlElement>().map(HtmlElement::clone)
                            })
                        {
                            let _ = dom_reference.focus();
                        }

                        return;
                    }

                    common_on_key_down(event);
                }) as ElementEventHandler<KeyboardEvent>)
            },
            on_pointer_move: {
                let is_pointer_modality_ref = Rc::clone(&is_pointer_modality_ref);
                Some(Rc::new(move |event: &PointerEvent| {
                    if is_stationary_webkit_pointer(event) {
                        return;
                    }
                    is_pointer_modality_ref.set(true);
                }) as ElementEventHandler<PointerEvent>)
            },
            ..ElementHandlers::default()
        }
    };

    // The `trigger` bag (`useListNavigation.ts:810-928`): the keyboard path plus
    // the four virtual-input modality checks.
    let make_check_virtual_pointer = |focus_item_on_open_ref: Rc<RefCell<FocusItemOnOpen>>| {
        Some(Rc::new(move |event: &PointerEvent| {
            // `pointerdown` fires first, reset the state then perform the checks
            // (`useListNavigation.ts:828-834`).
            *focus_item_on_open_ref.borrow_mut() = focus_item_on_open;
            if focus_item_on_open == FocusItemOnOpen::Auto && is_virtual_pointer_event(event) {
                *focus_item_on_open_ref.borrow_mut() = FocusItemOnOpen::True;
            }
        }) as ElementEventHandler<PointerEvent>)
    };
    let make_check_virtual_mouse = |focus_item_on_open_ref: Rc<RefCell<FocusItemOnOpen>>| {
        Some(Rc::new(move |event: &MouseEvent| {
            // `focusItemOnOpen === 'auto' && isVirtualClick(event.nativeEvent)`
            // (`useListNavigation.ts:822-826`).
            if focus_item_on_open == FocusItemOnOpen::Auto && is_virtual_click(event) {
                *focus_item_on_open_ref.borrow_mut() = if virtual_ {
                    FocusItemOnOpen::False
                } else {
                    FocusItemOnOpen::True
                };
            }
        }) as ElementEventHandler<MouseEvent>)
    };
    let trigger_handlers = ElementHandlers {
        on_key_down: {
            let store = Rc::clone(&store);
            let common_on_key_down = Rc::clone(&common_on_key_down);
            let index_ref = Rc::clone(&index_ref);
            let key_ref = Rc::clone(&key_ref);
            let is_pointer_modality_ref = Rc::clone(&is_pointer_modality_ref);
            let selected_index = selected_index.clone();
            let on_navigate = Rc::clone(&on_navigate);
            let get_parent_orientation = Rc::clone(&get_parent_orientation);
            let get_min_enabled_index = Rc::clone(&get_min_enabled_index);
            Some(Rc::new(move |event: &KeyboardEvent| {
                let open_on_navigation_key_down = {
                    let store = Rc::clone(&store);
                    let event = event.clone();
                    move || {
                        let current_target = event
                            .current_target()
                            .and_then(|target| target.dyn_into::<Element>().ok());
                        store.set_open(
                            true,
                            &RootOpenChangeEventDetails::new(
                                reasons::LIST_NAVIGATION,
                                event.into(),
                                current_target,
                                String::new(),
                            ),
                        );
                    }
                };

                // `useListNavigation.ts:837-903`.
                // non-reactive open state (to prevent re-creation of the handler)
                let current_open = store.select(selectors::open);
                is_pointer_modality_ref.set(false);

                let key = event.key();
                let is_arrow_key = key.starts_with("Arrow");
                let is_parent_cross_open_key =
                    is_cross_orientation_open_key(&key, get_parent_orientation(), rtl);
                let is_main_key = is_main_orientation_key(&key, Some(orientation));
                let is_navigation_key = (if nested {
                    is_parent_cross_open_key
                } else {
                    is_main_key
                }) || key == "Enter"
                    || key.trim().is_empty();

                if virtual_ && current_open {
                    common_on_key_down(event);
                    return;
                }

                // If a floating element should not open on arrow key down, avoid
                // setting `activeIndex` while it's closed
                // (`useListNavigation.ts:858-862`).
                if !current_open && !open_on_arrow_key_down && is_arrow_key {
                    return;
                }

                if is_navigation_key {
                    let is_parent_main_key =
                        is_main_orientation_key(&key, get_parent_orientation());
                    *key_ref.borrow_mut() = if nested && is_parent_main_key {
                        None
                    } else {
                        Some(key.clone())
                    };
                }

                if nested {
                    if is_parent_cross_open_key {
                        stop_event(event);

                        if current_open {
                            index_ref.set(get_min_enabled_index());
                            on_navigate(Some(event));
                        } else {
                            open_on_navigation_key_down();
                        }
                    }

                    return;
                }

                if is_main_key {
                    if let Some(selected_index) = selected_index.get_untracked() {
                        index_ref.set(selected_index);
                    }

                    stop_event(event);

                    if !current_open && open_on_arrow_key_down {
                        open_on_navigation_key_down();
                    } else {
                        common_on_key_down(event);
                    }

                    if current_open {
                        on_navigate(Some(event));
                    }
                }
            }) as ElementEventHandler<KeyboardEvent>)
        },
        on_focus: {
            let store = Rc::clone(&store);
            let index_ref = Rc::clone(&index_ref);
            let on_navigate = Rc::clone(&on_navigate);
            Some(Rc::new(move |event: &FocusEvent| {
                if store.select(selectors::open) && !virtual_ {
                    index_ref.set(-1);
                    on_navigate(Some(event));
                }
            }) as ElementEventHandler<FocusEvent>)
        },
        on_pointer_down: make_check_virtual_pointer(Rc::clone(&focus_item_on_open_ref)),
        on_pointer_enter: make_check_virtual_pointer(Rc::clone(&focus_item_on_open_ref)),
        on_mouse_down: make_check_virtual_mouse(Rc::clone(&focus_item_on_open_ref)),
        on_click: make_check_virtual_mouse(Rc::clone(&focus_item_on_open_ref)),
        ..ElementHandlers::default()
    };

    // The `reference` bag = `ariaActiveDescendantProp` + `trigger`
    // (`useListNavigation.ts:930-935`).
    let mut reference_handlers = trigger_handlers.clone();
    let mut reference_attributes: Vec<(String, ElementAttributeFn)> = vec![(
        "aria-activedescendant".to_owned(),
        Rc::clone(&aria_active_descendant),
    )];
    reference_attributes.extend(
        trigger_handlers
            .attributes
            .iter()
            .map(|(name, value)| (name.clone(), Rc::clone(value))),
    );
    reference_handlers.attributes = reference_attributes;

    // `enabled ? { reference, floating, item, trigger } : {}`
    // (`useListNavigation.ts:937-940`).
    if enabled {
        ElementProps {
            reference: Some(reference_handlers),
            floating: Some(floating_handlers),
            item: Some(item_handlers),
            trigger: Some(trigger_handlers),
        }
    } else {
        ElementProps::default()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Pins the orientation-key classification (`useListNavigation.ts:57-97`): the
    // main-axis keys, the to-end keys, and the cross-axis open/close keys —
    // including the rtl swaps, the `undefined`-parent-orientation default arm, and
    // the grid Escape override.
    #[test]
    fn orientation_key_classification_matches_upstream() {
        assert!(is_main_orientation_key(
            "ArrowDown",
            Some(Orientation::Vertical)
        ));
        assert!(!is_main_orientation_key(
            "ArrowLeft",
            Some(Orientation::Vertical)
        ));
        assert!(is_main_orientation_key(
            "ArrowRight",
            Some(Orientation::Horizontal)
        ));
        assert!(!is_main_orientation_key(
            "ArrowUp",
            Some(Orientation::Horizontal)
        ));
        assert!(is_main_orientation_key("ArrowUp", Some(Orientation::Both)));
        // `getParentOrientation()` returning `undefined` hits the switch default:
        // every arrow key is a main key.
        assert!(is_main_orientation_key("ArrowLeft", None));

        assert!(is_main_orientation_to_end_key(
            "ArrowDown",
            Some(Orientation::Vertical),
            false
        ));
        assert!(!is_main_orientation_to_end_key(
            "ArrowUp",
            Some(Orientation::Vertical),
            false
        ));
        assert!(is_main_orientation_to_end_key(
            "Enter",
            Some(Orientation::Vertical),
            false
        ));
        assert!(is_main_orientation_to_end_key(
            " ",
            Some(Orientation::Vertical),
            false
        ));
        assert!(is_main_orientation_to_end_key(
            "",
            Some(Orientation::Vertical),
            false
        ));
        assert!(is_main_orientation_to_end_key(
            "ArrowRight",
            Some(Orientation::Horizontal),
            false
        ));
        assert!(!is_main_orientation_to_end_key(
            "ArrowRight",
            Some(Orientation::Horizontal),
            true
        ));

        assert!(is_cross_orientation_open_key(
            "ArrowRight",
            Some(Orientation::Vertical),
            false
        ));
        assert!(!is_cross_orientation_open_key(
            "ArrowRight",
            Some(Orientation::Vertical),
            true
        ));
        assert!(is_cross_orientation_open_key(
            "ArrowDown",
            Some(Orientation::Horizontal),
            false
        ));
        // `undefined` parent orientation: the default arm takes any arrow through
        // each key class's own arm — ArrowRight is the vertical cross-open key.
        assert!(is_cross_orientation_open_key("ArrowRight", None, false));
        assert!(!is_cross_orientation_open_key("ArrowUp", None, false));

        assert!(is_cross_orientation_close_key(
            "ArrowLeft",
            Orientation::Vertical,
            false,
            false
        ));
        assert!(!is_cross_orientation_close_key(
            "ArrowLeft",
            Orientation::Vertical,
            true,
            false
        ));
        assert!(is_cross_orientation_close_key(
            "ArrowUp",
            Orientation::Horizontal,
            false,
            false
        ));
        assert!(is_cross_orientation_close_key(
            "Escape",
            Orientation::Both,
            false,
            false
        ));
        assert!(is_cross_orientation_close_key(
            "Escape",
            Orientation::Horizontal,
            false,
            true
        ));
        assert!(!is_cross_orientation_close_key(
            "Escape",
            Orientation::Horizontal,
            false,
            false
        ));
    }

    // Pins the stationary-pointer gate (`useListNavigation.ts:38-40`) at the
    // platform level: off WebKit (the host and Chromium probe) a zero-delta move
    // is never stationary — the WebKit-only branch cannot engage.
    #[test]
    fn zero_delta_moves_are_not_stationary_off_webkit() {
        assert!(!platform().engine.webkit);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use std::collections::VecDeque;

    use reactive_graph::owner::LocalStorage;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::Set;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::grid_navigation::grid_navigation_fn;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::tree::FloatingTreeStore;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    fn sleep(ms: i32) -> wasm_bindgen_futures::js_sys::Promise {
        wasm_bindgen_futures::js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap();
        })
    }

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn active_target() -> Option<Element> {
        active_element(&document())
    }

    /// The store's notify → use_state mirror → RenderEffect re-run chain is
    /// executor-timed; drain it (the `use_client_point` test wiring).
    fn flush() {
        for _ in 0..4 {
            any_spawner::Executor::poll_local();
        }
    }

    fn same_element(a: Option<&Element>, b: Option<&HtmlElement>) -> bool {
        match (a, b) {
            (Some(a), Some(b)) => JsValue::from(a) == JsValue::from(b),
            (None, None) => true,
            _ => false,
        }
    }

    /// The upstream `App` harness (`useListNavigation.test.tsx:19-89`): a store whose
    /// `onOpenChange` syncs back into the store state (the `setOpen` consumer), a
    /// reference button, an always-mounted floating menu holding three `li` items,
    /// and the consumer's `activeIndex` state fed by `onNavigate`.
    struct Harness {
        store: Rc<FloatingRootStore>,
        list: ListRef,
        items: Vec<HtmlElement>,
        floating: HtmlElement,
        reference: HtmlElement,
        active_index: RwSignal<Option<i32>, LocalStorage>,
        navigate_log: Rc<RefCell<Vec<Option<i32>>>>,
        open_log: Rc<RefCell<Vec<(bool, String)>>>,
        /// The floating bag's attribute members, cloned out of the returned props —
        /// the view layer's spread source (see the `element_props` module docs).
        floating_attributes: Vec<(String, ElementAttributeFn)>,
    }

    impl Harness {
        fn new() -> Harness {
            Self::with(|_| {})
        }

        /// `with` gives the test a mutation point on the props before the hook runs.
        fn with(
            configure: impl FnOnce(
                &mut UseListNavigationProps<
                    RwSignal<Option<i32>, LocalStorage>,
                    RwSignal<Option<i32>, LocalStorage>,
                >,
            ),
        ) -> Harness {
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

            // The consumer wiring the upstream harness has
            // (`useListNavigation.test.tsx:30-44` — `onOpenChange: setOpen` +
            // `onNavigate: setActiveIndex`).
            let open_log: Rc<RefCell<Vec<(bool, String)>>> = Rc::new(RefCell::new(Vec::new()));
            let weak = Rc::downgrade(&store);
            let log_handle = Rc::clone(&open_log);
            store.context.set_on_open_change(Some(Rc::new(
                move |open: bool, details: &RootOpenChangeEventDetails| {
                    if let Some(store) = weak.upgrade() {
                        store.update(|state, _| {
                            state.open = open;
                            true
                        });
                    }
                    log_handle.borrow_mut().push((open, details.reason.clone()));
                },
            )));

            let reference: HtmlElement = document()
                .create_element("button")
                .unwrap()
                .dyn_into()
                .unwrap();
            document().body().unwrap().append_child(&reference).unwrap();
            store.set_field(
                |state| &mut state.dom_reference_element,
                Some(reference.clone().into()),
            );

            let floating: HtmlElement = document()
                .create_element("div")
                .unwrap()
                .dyn_into()
                .unwrap();
            // Real popup elements are focusable (tabindex=-1) — the pointer-leave
            // reset moves focus back to this element.
            floating.set_tab_index(-1);
            document().body().unwrap().append_child(&floating).unwrap();
            store.set_field(
                |state| &mut state.floating_element,
                Some(floating.clone().into()),
            );

            let list: ListRef = Rc::new(RefCell::new(Vec::new()));
            let items: Vec<HtmlElement> = (0..3)
                .map(|index| {
                    let item: HtmlElement =
                        document().create_element("li").unwrap().dyn_into().unwrap();
                    item.set_tab_index(-1);
                    item.set_text_content(Some(match index {
                        0 => "one",
                        1 => "two",
                        _ => "three",
                    }));
                    floating.append_child(&item).unwrap();
                    list.borrow_mut().push(Some(item.clone().into()));
                    item
                })
                .collect();

            let active_index: RwSignal<Option<i32>, LocalStorage> = RwSignal::new_local(None);
            let navigate_log: Rc<RefCell<Vec<Option<i32>>>> = Rc::new(RefCell::new(Vec::new()));
            let navigate_handle = Rc::clone(&navigate_log);
            let active_handle = active_index.clone();

            let mut props = UseListNavigationProps::new(
                Rc::clone(&list),
                active_index.clone(),
                RwSignal::new_local(None),
            );
            props.on_navigate = Some(Rc::new(
                move |index: Option<i32>, _event: Option<&Event>| {
                    navigate_handle.borrow_mut().push(index);
                    active_handle.set(index);
                },
            ));
            configure(&mut props);

            let element_props = use_list_navigation(Rc::clone(&store), props);

            let floating_attributes = element_props
                .floating
                .as_ref()
                .map(|bag| bag.attributes.clone())
                .unwrap_or_default();

            let reference_cleanup = element_props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(reference.as_ref());
            let floating_cleanup = element_props
                .floating
                .as_ref()
                .unwrap()
                .attach_to(floating.as_ref());
            let item_cleanups: Vec<_> = items
                .iter()
                .map(|item| {
                    element_props
                        .item
                        .as_ref()
                        .unwrap()
                        .attach_to(item.as_ref())
                })
                .collect();
            // Keep the cleanups alive for the harness lifetime — the dispatches
            // below rely on the listeners staying attached.
            std::mem::forget(reference_cleanup);
            std::mem::forget(floating_cleanup);
            std::mem::forget(item_cleanups);

            // Settle the mount-time executor queue (the store mirror effects'
            // first runs re-notify their signals, which re-runs the mount effects
            // once — upstream's effects settle during the mount commit).
            flush();

            Harness {
                store,
                list,
                items,
                floating,
                reference,
                active_index,
                navigate_log,
                open_log,
                floating_attributes,
            }
        }

        fn open_calls(&self) -> Vec<(bool, String)> {
            self.open_log.borrow().clone()
        }

        fn navigations(&self) -> Vec<Option<i32>> {
            self.navigate_log.borrow().clone()
        }

        fn last_navigation(&self) -> Option<Option<i32>> {
            self.navigate_log.borrow().last().copied()
        }

        fn key_down(&self, target: &HtmlElement, key: &str) {
            self.key_down_full(target, key, false);
        }

        fn key_down_full(&self, target: &HtmlElement, key: &str, shift: bool) {
            let init = web_sys::KeyboardEventInit::new();
            init.set_bubbles(true);
            init.set_cancelable(true);
            init.set_key(key);
            init.set_shift_key(shift);
            let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap();
            target.dispatch_event(&event).unwrap();
            flush();
        }

        fn mouse_move(&self, target: &HtmlElement) {
            let init = web_sys::MouseEventInit::new();
            init.set_bubbles(true);
            init.set_movement_x(10);
            init.set_movement_y(10);
            let event = MouseEvent::new_with_mouse_event_init_dict("mousemove", &init).unwrap();
            target.dispatch_event(&event).unwrap();
            flush();
        }

        fn pointer_move(&self, target: &HtmlElement) {
            let init = web_sys::PointerEventInit::new();
            init.set_bubbles(true);
            init.set_pointer_type("mouse");
            init.set_movement_x(10);
            init.set_movement_y(10);
            let event = PointerEvent::new_with_event_init_dict("pointermove", &init).unwrap();
            target.dispatch_event(&event).unwrap();
            flush();
        }

        fn pointer_leave(&self, target: &HtmlElement, related: Option<&web_sys::EventTarget>) {
            let init = web_sys::PointerEventInit::new();
            init.set_pointer_type("mouse");
            if let Some(related) = related {
                init.set_related_target(Some(related));
            }
            let event = PointerEvent::new_with_event_init_dict("pointerleave", &init).unwrap();
            target.dispatch_event(&event).unwrap();
            flush();
        }

        fn focused_tag(&self) -> String {
            active_target()
                .map(|element| element.tag_name().to_lowercase())
                .unwrap_or_default()
        }

        fn focus_index(&self) -> Option<usize> {
            let focused = active_target()?;
            self.items
                .iter()
                .position(|item| JsValue::from(item) == JsValue::from(&focused))
        }

        fn resolved_attribute(&self, name: &str) -> Option<String> {
            self.floating_attributes
                .iter()
                .find(|(attribute_name, _)| attribute_name == name)
                .and_then(|(_, value)| value())
        }
    }

    // Pins the open-and-focus contract (`useListNavigation.test.tsx:191-199`):
    // ArrowDown on the closed reference opens with the list-navigation reason and
    // lands DOM focus on the first item.
    #[wasm_bindgen_test(async)]
    async fn arrow_down_opens_and_focuses_the_first_item() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let harness = Harness::new();

            harness.key_down(&harness.reference, "ArrowDown");
            sleep(60).await;

            assert_eq!(
                harness.open_calls(),
                vec![(true, "list-navigation".to_owned())],
                "the arrow key opened the popup"
            );
            assert_eq!(
                harness.focus_index(),
                Some(0),
                "the first item received focus"
            );
            assert_eq!(
                harness.last_navigation(),
                Some(Some(0)),
                "the initial sync reported index 0"
            );
        };
        __owner.cleanup();
    }

    // Pins the to-start open (`useListNavigation.test.tsx:201-209`): ArrowUp opens
    // and focuses the LAST item.
    #[wasm_bindgen_test(async)]
    async fn arrow_up_opens_and_focuses_the_last_item() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let harness = Harness::new();

            harness.key_down(&harness.reference, "ArrowUp");
            sleep(60).await;

            assert_eq!(
                harness.focus_index(),
                Some(2),
                "the last item received focus"
            );
        };
        __owner.cleanup();
    }

    // Pins the floating-element navigation (`useListNavigation.test.tsx:211-261`):
    // ArrowDown walks item to item and stops at the end; ArrowUp walks back and
    // stops at the start.
    #[wasm_bindgen_test(async)]
    async fn arrow_keys_navigate_the_open_list_and_stop_at_the_ends() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let harness = Harness::new();
            harness.key_down(&harness.reference, "ArrowDown");
            sleep(60).await;
            assert_eq!(harness.focus_index(), Some(0));

            harness.key_down(&harness.floating, "ArrowDown");
            sleep(40).await;
            assert_eq!(harness.focus_index(), Some(1), "ArrowDown moved to item 1");

            harness.key_down(&harness.floating, "ArrowDown");
            sleep(40).await;
            assert_eq!(harness.focus_index(), Some(2), "ArrowDown moved to item 2");

            harness.key_down(&harness.floating, "ArrowDown");
            sleep(40).await;
            assert_eq!(
                harness.focus_index(),
                Some(2),
                "ArrowDown at the end of the list stays"
            );

            harness.key_down(&harness.floating, "ArrowUp");
            sleep(40).await;
            assert_eq!(
                harness.focus_index(),
                Some(1),
                "ArrowUp moved back to item 1"
            );

            harness.key_down(&harness.floating, "ArrowUp");
            harness.key_down(&harness.floating, "ArrowUp");
            sleep(40).await;
            assert_eq!(
                harness.focus_index(),
                Some(0),
                "ArrowUp at the start of the list stays"
            );
        };
        __owner.cleanup();
    }

    // Pins the wrap-around (`useListNavigation.test.tsx:441-491`): with
    // `loopFocus`, ArrowUp from the first item re-enters at the bottom and ArrowDown
    // from the last re-enters at the top.
    #[wasm_bindgen_test(async)]
    async fn loop_focus_wraps_around_the_ends() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let harness = Harness::with(|props| {
                props.loop_focus = true;
            });

            harness.key_down(&harness.reference, "ArrowDown");
            sleep(60).await;
            assert_eq!(harness.focus_index(), Some(0));

            harness.key_down(&harness.floating, "ArrowUp");
            sleep(40).await;
            assert_eq!(
                harness.focus_index(),
                Some(2),
                "ArrowUp from the first item wraps to the last"
            );

            harness.key_down(&harness.floating, "ArrowDown");
            sleep(40).await;
            assert_eq!(
                harness.focus_index(),
                Some(0),
                "ArrowDown from the last item wraps to the first"
            );
        };
        __owner.cleanup();
    }

    // Pins the disabled skipping (`useListNavigation.test.tsx:263-286,710-722`): a
    // disabled first item is skipped on the initial navigation and cannot be walked
    // into afterwards — both through the explicit set and the aria-disabled
    // attribute fallback (the item mirrors the upstream `disableFirstItem` arm).
    #[wasm_bindgen_test(async)]
    async fn disabled_first_item_is_skipped() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let harness = Harness::with(|props| {
                props.loop_focus = true;
                // The upstream harness mirrors the set on the item's
                // aria-disabled attribute (`useListNavigation.test.tsx:68-73`).
                props.disabled_indices = Some(DisabledIndices::List(vec![0]));
            });
            harness.items[0]
                .set_attribute("aria-disabled", "true")
                .unwrap();

            harness.key_down(&harness.reference, "ArrowDown");
            sleep(60).await;
            assert_eq!(
                harness.focus_index(),
                Some(1),
                "the initial sync skipped the aria-disabled first item"
            );

            harness.key_down(&harness.floating, "ArrowUp");
            sleep(40).await;
            assert_eq!(
                harness.focus_index(),
                Some(2),
                "ArrowUp from item 1 is at the min (item 0 disabled), so loopFocus wraps to the end"
            );
        };
        __owner.cleanup();
    }

    // Pins the hover sync (`useListNavigation.test.tsx:780-790`) and the
    // pointer-leave reset (`:786-788`): mousemove focuses and reports the hovered
    // item; a pointer leave that exits the list resets the index to null and moves
    // focus back to the floating element.
    #[wasm_bindgen_test(async)]
    async fn hover_syncs_the_index_and_pointer_leave_resets_it() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let harness = Harness::new();
            harness.key_down(&harness.reference, "ArrowDown");
            sleep(60).await;

            // The keyboard open set keyboard modality; a real pointer entering the
            // popup restores pointer modality through the floating bag's
            // onPointerMove before the item is hovered.
            harness.pointer_move(&harness.floating);

            harness.mouse_move(&harness.items[1]);
            sleep(40).await;
            assert_eq!(
                harness.focus_index(),
                Some(1),
                "the hovered item received focus"
            );
            assert!(
                harness.navigations().contains(&Some(1)),
                "the hover reported index 1: {:?}",
                harness.navigations()
            );

            harness.pointer_leave(&harness.items[1], Some(&document().body().unwrap().into()));
            sleep(40).await;
            assert_eq!(
                harness.last_navigation(),
                Some(None),
                "the pointer leave reported the null index"
            );
            assert_eq!(
                harness.focused_tag(),
                "div",
                "focus moved back to the floating element"
            );
        };
        __owner.cleanup();
    }

    // Pins the virtual mode (`useListNavigation.test.tsx:644-679`): no DOM focus
    // moves — the tree's 'virtualfocus' bus carries the item element instead.
    #[wasm_bindgen_test(async)]
    async fn virtual_mode_emits_virtualfocus_instead_of_moving_focus() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let tree = SharedFloatingTreeStore::new(Rc::new(FloatingTreeStore::new()));
            let virtual_focus_log: Rc<RefCell<VecDeque<Element>>> =
                Rc::new(RefCell::new(VecDeque::new()));
            {
                let log = Rc::clone(&virtual_focus_log);
                tree.events.on(
                    "virtualfocus",
                    Rc::new(move |payload: &FloatingTreeEvent| {
                        if let FloatingTreeEvent::VirtualFocus(element) = payload {
                            log.borrow_mut().push_back(element.clone());
                        }
                    }),
                );
            }

            let harness = Harness::with(|props| {
                props.is_virtual = true;
                props.external_tree = Some(tree.clone());
                props.id = Some("menu".to_owned());
            });

            harness.key_down(&harness.reference, "ArrowDown");
            sleep(60).await;

            assert_eq!(harness.active_index.get_untracked(), Some(0));
            assert_eq!(
                harness.focus_index(),
                None,
                "virtual focus never moves DOM focus"
            );
            assert_eq!(
                virtual_focus_log.borrow().len(),
                1,
                "the virtualfocus event was emitted once"
            );
            assert!(
                same_element(virtual_focus_log.borrow().front(), harness.items.first()),
                "the event carried the first item element"
            );
        };
        __owner.cleanup();
    }

    // Pins the aria-activedescendant attribute (`useListNavigation.ts:756-764`):
    // while open with an active index, the floating bag resolves
    // `aria-activedescendant` to `<id>-<index>`; before any navigation it resolves
    // to nothing (the `&&`-gated spread arm).
    #[wasm_bindgen_test(async)]
    async fn the_floating_bag_resolves_aria_activedescendant() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let tree = SharedFloatingTreeStore::new(Rc::new(FloatingTreeStore::new()));
            let harness = Harness::with(|props| {
                props.is_virtual = true;
                props.external_tree = Some(tree.clone());
                props.id = Some("menu".to_owned());
            });

            assert_eq!(
                harness.resolved_attribute("aria-activedescendant"),
                None,
                "closed with no active index, the attribute is omitted"
            );

            harness.key_down(&harness.reference, "ArrowDown");
            sleep(60).await;
            assert_eq!(harness.active_index.get_untracked(), Some(0));

            assert_eq!(
                harness.resolved_attribute("aria-activedescendant"),
                Some("menu-0".to_owned()),
                "open with active index 0, the attribute points at the item id"
            );
        };
        __owner.cleanup();
    }

    // Pins the Shift+Tab close path (`useListNavigation.ts:769-788`): the floating
    // element closes with the focus-out reason and focus returns to the reference.
    #[wasm_bindgen_test(async)]
    async fn shift_tab_closes_with_the_focus_out_reason() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let harness = Harness::new();
            harness.key_down(&harness.reference, "ArrowDown");
            sleep(60).await;
            assert_eq!(harness.open_calls().len(), 1);

            harness.key_down_full(&harness.floating, "Tab", true);
            sleep(40).await;

            assert_eq!(
                harness.open_calls(),
                vec![
                    (true, "list-navigation".to_owned()),
                    (false, "focus-out".to_owned())
                ],
                "Shift+Tab closed with the focus-out reason"
            );
            assert_eq!(
                harness.focused_tag(),
                "button",
                "focus returned to the reference"
            );
        };
        __owner.cleanup();
    }

    // Pins the nested close key (upstream's NestedMenu behavior,
    // `useListNavigation.ts:551-569`): with `nested`, the parent's main axis key
    // closes the child with the list-navigation reason and refocuses the reference
    // element.
    #[wasm_bindgen_test(async)]
    async fn the_nested_close_key_closes_and_refocuses_the_reference() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let harness = Harness::with(|props| {
                props.nested = true;
            });
            harness.key_down(&harness.reference, "ArrowDown");
            sleep(60).await;
            assert_eq!(harness.open_calls().len(), 1, "the nested popup opened");

            harness.key_down(&harness.floating, "ArrowLeft");
            sleep(40).await;

            assert_eq!(
                harness.open_calls(),
                vec![
                    (true, "list-navigation".to_owned()),
                    (false, "list-navigation".to_owned())
                ],
                "the parent's main axis key closed the nested popup"
            );
            assert_eq!(
                harness.focused_tag(),
                "button",
                "focus returned to the reference"
            );
        };
        __owner.cleanup();
    }

    // Pins `openOnArrowKeyDown: false` (`useListNavigation.test.tsx:697-707`): an
    // arrow key on the closed reference neither opens nor moves the index.
    #[wasm_bindgen_test(async)]
    async fn open_on_arrow_key_down_false_does_not_open() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let harness = Harness::with(|props| {
                props.open_on_arrow_key_down = false;
            });

            // The pre-mounted floating element makes the mount-time reset report
            // one null navigation (upstream's `previousMountedRef` seed behaves the
            // same with an already-mounted floating element); the keydown must not
            // add anything.
            let navigations_before = harness.navigations().len();
            harness.key_down(&harness.reference, "ArrowDown");
            sleep(40).await;

            assert!(
                harness.open_calls().is_empty(),
                "no open was requested: {:?}",
                harness.open_calls()
            );
            assert_eq!(
                harness.navigations().len(),
                navigations_before,
                "no navigation was reported by the keydown while closed: {:?}",
                harness.navigations()
            );
        };
        __owner.cleanup();
    }

    // Pins the injected grid navigator end-to-end (`useListNavigation.test.tsx:876-1044`,
    // the Grid fixture): ArrowDown strides a column, ArrowRight moves within the
    // row and wraps at the row end, and ArrowUp strides back.
    #[wasm_bindgen_test(async)]
    async fn the_injected_grid_navigates_two_dimensionally() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            // Six items on the shim's default 2-column grid.
            let mut harness = Harness::with(|props| {
                props.grid = Some(grid_navigation_fn());
                props.orientation = Orientation::Both;
                props.loop_focus = true;
            });
            for _index in 3..6 {
                let item: HtmlElement =
                    document().create_element("li").unwrap().dyn_into().unwrap();
                item.set_tab_index(-1);
                harness.floating.append_child(&item).unwrap();
                harness.list.borrow_mut().push(Some(item.clone().into()));
                harness.items.push(item);
            }

            harness.key_down(&harness.reference, "ArrowDown");
            sleep(60).await;
            assert_eq!(harness.active_index.get_untracked(), Some(0));

            harness.key_down(&harness.floating, "ArrowDown");
            sleep(40).await;
            assert_eq!(
                harness.active_index.get_untracked(),
                Some(2),
                "ArrowDown strides the 2-column grid"
            );

            harness.key_down(&harness.floating, "ArrowRight");
            sleep(40).await;
            assert_eq!(
                harness.active_index.get_untracked(),
                Some(3),
                "ArrowRight moves within the row"
            );

            harness.key_down(&harness.floating, "ArrowRight");
            sleep(40).await;
            assert_eq!(
                harness.active_index.get_untracked(),
                Some(2),
                "ArrowRight at the row end wraps within the row"
            );

            harness.key_down(&harness.floating, "ArrowUp");
            sleep(40).await;
            assert_eq!(
                harness.active_index.get_untracked(),
                Some(0),
                "ArrowUp strides back up the column"
            );
        };
        __owner.cleanup();
    }

    // Pins the close reset (`useListNavigation.ts:379-385`): when the popup closes,
    // the index resets to -1 and the consumer receives `null`.
    #[wasm_bindgen_test(async)]
    async fn closing_resets_the_active_index() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let harness = Harness::new();
            harness.key_down(&harness.reference, "ArrowDown");
            sleep(60).await;
            assert_eq!(harness.active_index.get_untracked(), Some(0));

            harness.store.set_open(
                false,
                &RootOpenChangeEventDetails::new(
                    crate::floating_ui::reasons::ESCAPE_KEY,
                    web_sys::Event::new("escape").unwrap(),
                    None,
                    String::new(),
                ),
            );
            flush();
            sleep(40).await;

            assert_eq!(
                harness.last_navigation(),
                Some(None),
                "the close reset reported the null index"
            );
        };
        __owner.cleanup();
    }
}
