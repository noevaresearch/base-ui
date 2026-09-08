//! Port of the positioning-engine delegation of
//! `packages/react/src/floating-ui-react/hooks/useFloating.ts:71-77` — upstream delegates
//! all positioning to `@floating-ui/react-dom`'s `useFloating` (aliased `usePosition`),
//! and the port delegates the same concerns to the `floating-ui-dom` Rust crate per the
//! unit's `wraps-external` TODO field (`specs/library/floating-ui-react/implementation.
//! md`, "Dependencies on other Base UI internals").
//!
//! Upstream engine semantics ported (`node_modules/@floating-ui/react-dom`,
//! `floating-ui.react-dom.esm.js`, `useFloating`):
//! - `update()` computes the position when both elements exist
//!   (`:130-133` guard), passing placement/strategy/middleware, and writes
//!   `isPositioned: openRef.current !== false` — "The floating element's position may be
//!   recomputed while it's closed but still mounted ... avoid setting it to `true` when
//!   `open === false`".
//! - Elements drive computation through an effect on both elements, invoking
//!   `whileElementsMounted(reference, floating, update)` when provided (its return value
//!   is the per-run cleanup, `:265-281`) and `update()` otherwise.
//! - A separate effect resets `isPositioned` to `false` when `open` goes false
//!   (`:243-251`).
//! - `floatingStyles` resolves `transform` positioning with DPR rounding
//!   (`:285-309`), including the `willChange: 'transform'` hint at DPR ≥ 1.5.
//!
//! ## Rust adaptations
//!
//! - Upstream reads elements from its own ref objects, which the Base UI layer feeds
//!   both directly (setters) and through the elements option. The port reads the
//!   root store's coalesced selectors (`positionReference ?? referenceElement`,
//!   `components/FloatingRootStore.ts:37` / `floatingElement`) — the same values
//!   upstream's `elements` option carries into the engine (`hooks/useFloating.ts:71-77`
//!   passes `...storeElements` plus the `positionReference` override). The store's
//!   no-op-skip update rule keeps the effect from re-firing on identical writes.
//! - `data` state ports to `RwSignal`s; upstream's `deepEqual` skip before `setData`
//!   (`:154-156`) is reactive_graph's own equality check on `set`.
//! - The middleware option is a reactive `Signal<WrappedMiddleware>`; the
//!   `deepEqual(latestMiddleware, middleware)` render-phase sync (`:102-106`) is
//!   expressed by the signal's tracking (an identity change re-runs the engine's
//!   effect).
//! - `floatingStyles` is a derived signal instead of a `useMemo`d style object, and the
//!   style members are plain strings (Leptos's native style bindings consume them;
//!   `specs/architecture.md`, "Prop / class / style merging (mergeProps)").

use std::rc::Rc;

use floating_ui_dom::{
    ComputePositionConfig, ElementOrVirtual, MiddlewareData, Placement, Strategy, compute_position,
    dom,
};
use reactive_graph::effect::Effect;
use reactive_graph::owner::LocalStorage;
use reactive_graph::owner::on_cleanup;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use web_sys::Element;

use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::selectors;
use crate::floating_ui::types::{
    PositioningStyles, ReferenceType, WhileElementsMountedFn, WrappedMiddleware,
};

/// Options for [`use_position`] — upstream's `UseFloatingOptions` positioning members
/// (`placement`, `strategy`, `middleware`, `transform`, `whileElementsMounted`; `open`
/// and `elements` come from the root store).
pub struct UsePositionOptions {
    /// Where to place the floating element (`placement = 'bottom'` default upstream).
    pub placement: Signal<Placement>,
    /// The positioning strategy (`strategy = 'absolute'` default upstream).
    pub strategy: Signal<Strategy>,
    /// The middleware stack, shared behind a `SendWrapper` (see
    /// [`crate::floating_ui::types::WrappedMiddleware`]).
    pub middleware: Signal<WrappedMiddleware>,
    /// `transform` positioning instead of `top`/`left` (default `true` upstream).
    pub transform: Signal<bool>,
    /// `whileElementsMounted` — the `autoUpdate` wiring point.
    pub while_elements_mounted: Option<Rc<WhileElementsMountedFn>>,
}

/// The engine's return — upstream `UseFloatingData` + `update` + `floatingStyles`.
pub struct UsePositionReturn {
    pub x: RwSignal<f64>,
    pub y: RwSignal<f64>,
    pub placement: RwSignal<Placement>,
    pub strategy: RwSignal<Strategy>,
    pub middleware_data: RwSignal<MiddlewareData>,
    pub is_positioned: RwSignal<bool>,
    pub floating_styles: Signal<PositioningStyles, LocalStorage>,
    pub update: Rc<dyn Fn()>,
}

/// `getDPR` — `floating-ui-dom`'s `dom.get_window(Some(element)).devicePixelRatio()`
/// (upstream `utils/getDPR`).
fn get_dpr(element: &Element) -> f64 {
    dom::get_window(Some(element)).device_pixel_ratio()
}

/// `roundByDPR` (upstream `utils/roundByDPR`): rounds the coordinate to the device
/// pixel grid so the floating element renders on whole physical pixels.
fn round_by_dpr(element: &Element, value: f64) -> f64 {
    let dpr = get_dpr(element);
    (value * dpr).round() / dpr
}

/// Runs the positioning engine against the root store — the port of the delegation at
/// `hooks/useFloating.ts:71-77`. Must be called inside a reactive owner.
pub fn use_position(
    store: &Rc<FloatingRootStore>,
    options: UsePositionOptions,
) -> UsePositionReturn {
    // The render-phase hooks take the shared `ReactStore` handle (`&Rc<Self>`
    // receivers).
    let inner = store.rc();
    let reference_element = inner.use_state(selectors::reference_element);
    let floating_element = inner.use_state(selectors::floating_element);
    let open = inner.use_state(selectors::open);

    let x = RwSignal::new(0.0);
    let y = RwSignal::new(0.0);
    let placement = RwSignal::new(options.placement.get_untracked());
    let strategy = RwSignal::new(options.strategy.get_untracked());
    let middleware_data = RwSignal::new(MiddlewareData::default());
    let is_positioned = RwSignal::new(false);

    let update: Rc<dyn Fn()> = {
        // Upstream's `update` reads `referenceRef.current`/`floatingRef.current` and
        // `openRef.current` — always-fresh values, not render-mirrored state
        // (`node_modules/@floating-ui/react-dom`, `useFloating`). The port reads the
        // store snapshot directly, which is that same freshness (the mirrors only feed
        // the tracking effect below).
        let snapshot_store = Rc::clone(store);
        let options_placement = options.placement.clone();
        let options_strategy = options.strategy.clone();
        let options_middleware = options.middleware.clone();
        Rc::new(move || {
            let snapshot = snapshot_store.get_snapshot();
            let (Some(reference), Some(floating)) = (
                selectors::reference_element(&snapshot),
                selectors::floating_element(&snapshot),
            ) else {
                return;
            };
            let config = ComputePositionConfig {
                placement: Some(options_placement.get_untracked()),
                strategy: Some(options_strategy.get_untracked()),
                middleware: Some(
                    std::ops::Deref::deref(&options_middleware.get_untracked()).clone(),
                ),
            };
            let engine_reference = match &reference {
                ReferenceType::Element(element) => ElementOrVirtual::Element(element),
                ReferenceType::Virtual(virtual_reference) => {
                    ElementOrVirtual::VirtualElement(virtual_reference.element.clone())
                }
            };
            let position = compute_position(engine_reference, &floating, config);
            x.set(position.x);
            y.set(position.y);
            placement.set(position.placement);
            strategy.set(position.strategy);
            middleware_data.set(position.middleware_data);
            // "avoid setting it to `true` when `open === false`" — the closed-but-mounted
            // recomputation case (see the module docs).
            is_positioned.set(snapshot.open);
        })
    };

    // The elements effect (`:265-281`): compute (or wire whileElementsMounted) whenever
    // either element changes.
    {
        let update = Rc::clone(&update);
        let while_elements_mounted = options.while_elements_mounted.clone();
        Effect::new(move |_| {
            let reference = reference_element.get();
            let floating = floating_element.get();
            if let (Some(reference), Some(floating)) = (reference, floating) {
                if let Some(while_elements_mounted) = while_elements_mounted.as_ref() {
                    let cleanup = while_elements_mounted(&reference, &floating, Rc::clone(&update));
                    // The returned cleanup is per-run (`:273-280` — the effect's return
                    // value): disposed before the next run and on owner disposal. The
                    // handle rides behind a `SendWrapper` for `on_cleanup`'s Send+Sync
                    // bound.
                    let cleanup = SendWrapper::new(cleanup);
                    on_cleanup(move || cleanup());
                } else {
                    update();
                }
            }
        });
    }

    // The `open === false` reset effect (`:243-251`).
    {
        Effect::new(move |_| {
            if !open.get() && is_positioned.get_untracked() {
                is_positioned.set(false);
            }
        });
    }

    let floating_styles = Signal::derive_local({
        let floating_element = floating_element.clone();
        let options_strategy = options.strategy.clone();
        let options_transform = options.transform.clone();
        move || {
            let initial = PositioningStyles {
                position: options_strategy.get_untracked(),
                top: "0".to_owned(),
                left: "0".to_owned(),
                transform: None,
                will_change: None,
            };
            let Some(floating) = floating_element.get() else {
                return initial;
            };
            let x_value = round_by_dpr(&floating, x.get_untracked());
            let y_value = round_by_dpr(&floating, y.get_untracked());
            if options_transform.get() {
                PositioningStyles {
                    transform: Some(format!("translate({x_value}px, {y_value}px)")),
                    will_change: (get_dpr(&floating) >= 1.5).then(|| "transform".to_owned()),
                    ..initial
                }
            } else {
                PositioningStyles {
                    top: format!("{y_value}px"),
                    left: format!("{x_value}px"),
                    ..initial
                }
            }
        }
    });

    UsePositionReturn {
        x,
        y,
        placement,
        strategy,
        middleware_data,
        is_positioned,
        floating_styles,
        update,
    }
}

// The engine's behaviors are DOM-bound (real rects): the wasm/browser suite pins them.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;

    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;

    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn empty_store() -> Rc<FloatingRootStore> {
        FloatingRootStore::new(FloatingRootStoreOptions {
            open: false,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only: false,
            nested: false,
            on_open_change: None,
        })
    }

    fn mount_element() -> Element {
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&element)
            .unwrap();
        element
    }

    fn position_options() -> UsePositionOptions {
        UsePositionOptions {
            placement: Signal::derive(|| Placement::Bottom),
            strategy: Signal::derive(|| Strategy::Absolute),
            middleware: Signal::derive(|| SendWrapper::new(Vec::new())),
            transform: Signal::derive(|| true),
            while_elements_mounted: None,
        }
    }

    // Pins the engine's core flow (`node_modules/@floating-ui/react-dom`,
    // `useFloating`): both elements present → the position computes; `isPositioned`
    // follows `open` ("avoid setting it to `true` when `open === false`"); a closed
    // store leaves `isPositioned` false even though the position computed.
    //
    // The store's `use_state` mirrors re-apply on the microtask after a notify (the
    // ReactStore port's effect wiring), so the test yields between the store writes and
    // the engine reads — the same ordering real consumers get from their reactive
    // effects.
    #[wasm_bindgen_test]
    fn the_engine_positions_elements_and_gates_is_positioned_on_open() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();

        let store = empty_store();
        let reference = mount_element();
        let floating = mount_element();

        store.set_field(
            |state| &mut state.reference_element,
            Some(ReferenceType::Element(reference.clone())),
        );
        store.set_field(|state| &mut state.floating_element, Some(floating.clone()));

        let result = use_position(&store, position_options());

        // Mark the popup open and compute — `update` reads the store snapshot, so no
        // settle time is needed (the upstream `openRef.current` semantics).
        store.update(|state, _| {
            state.open = true;
            true
        });
        (result.update)();
        assert!(
            result.is_positioned.get_untracked(),
            "isPositioned is true when open"
        );

        // Closing does not report positioned (the open gate), while the coordinates
        // still computed — the closed-but-mounted recomputation case.
        store.update(|state, _| {
            state.open = false;
            true
        });
        (result.update)();
        assert!(
            !result.is_positioned.get_untracked(),
            "isPositioned stays false when closed, even after a recomputation"
        );

        owner.cleanup();
    }
}
