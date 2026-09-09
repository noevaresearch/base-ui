//! Port of `packages/react/src/internals/composite/root/gridNavigation.ts` — the grid
//! navigator builder the composite root consumes through its `grid` parameter
//! (`TODO.md`, item `infra: internals`; the checkpoint sequence recorded in that entry's
//! note). This is the *internals* grid navigation — distinct from
//! [`crate::floating_ui::grid_navigation`], the floating-ui positional-argument shim over
//! the same underlying cell-map math.
//!
//! Upstream (`packages/react/src/internals/composite/root/gridNavigation.ts:50-126`)
//! builds the grid navigation handler passed to `CompositeRoot`/`useCompositeRoot` via
//! the `grid` prop; importing and calling `gridNavigation` is the opt-in for grid
//! support ("composites that don't pass `grid` never reference the algorithm, so
//! bundlers tree-shake the grid helpers out" — its doc comment). The returned navigator
//! works in *hypothetical 1×1 cell indices*: `createGridCellMap` packs the (possibly
//! multi-cell) item sizes into a flat cell map, gaps are treated as disabled so
//! navigation cannot land in them, `getGridCellIndexOfCorner` picks the corner of the
//! spanning item closest to the movement direction so spanning items do not immediately
//! resolve back to themselves, and floating-ui's `getGridNavigatedIndex` does the actual
//! step. The final `cellMap[cellIndex]` converts the navigated cell back to an item
//! index.
//!
//! Spec: `specs/library/internals/implementation.md` ("Highlight state machine" — "Grid
//! math lives in a closure built by `gridNavigation(config)` … lets floating-ui's
//! `getGridNavigatedIndex` do the actual step") and `specs/library/internals/behavior.md`
//! ("Keyboard interactions" — the grid navigation matrix asserted through
//! `CompositeRoot`). Every claim was verified against the source before porting.
//!
//! ## Rust adaptations
//!
//! - `CompositeGridItemSize` is upstream's `{ width, height }` shape — the same
//!   `Dimensions` record the floating-ui cell-map helpers already pack, so the port
//!   aliases [`crate::floating_ui::types::Dimensions`] (upstream's two types are
//!   structurally identical).
//! - The navigator's state object (`CompositeGridNavigationState`, `:29-40`) ports to a
//!   borrowed struct; the `event` field splits into `key` + `event: Option<&KeyboardEvent>`
//!   — the [`crate::floating_ui::composite::GridNavigatedIndexOptions`] host/wasm split
//!   (the host target runs the index math event-independently).
//! - `onLoop` (`:37`) keeps the upstream 3-argument cell-level shape; the *user-facing*
//!   4-argument form (with `elementsRef`) is adapted by the composite root's
//!   `wrappedOnLoop` (`useCompositeRoot.ts:195-202`), exactly as upstream adapts it at
//!   its call site (`:261`).
//! - The cell-level `disabledIndices` derivation (`:94-103`) —
//!   `getGridCellIndices([...(disabledIndices || perIndexDerived), undefined], cellMap)` —
//!   ports with one documented quirk: when a *predicate* `disabledIndices` is provided,
//!   upstream spreads a function into the index array (`[...fn]` yields `[fn]`), and
//!   `getGridCellIndices`' `includes` never matches a function against a cell entry, so
//!   the predicate contributes no disabled cells (only the trailing `undefined` gap
//!   marker does). The port reproduces that: the predicate arm contributes no indices.
//! - `cellMap[cellIndex] as number` (`:125`, "Navigated cell will never be nullish") is
//!   an upstream non-null assertion; the port returns `-1` for the impossible unoccupied
//!   arm, which routes through the composite root's `isIndexOutOfListBounds` commit gate
//!   (a `-1` is out of bounds, so nothing commits) instead of JS's `undefined`-comparison
//!   behavior.

use std::rc::Rc;

use web_sys::KeyboardEvent;

use crate::composite_list::CompositeListElementsRef;
use crate::floating_ui::composite::{
    DisabledIndices, GridCorner, create_grid_cell_map, get_grid_cell_index_of_corner,
    get_grid_cell_indices, get_grid_navigated_index, is_list_index_disabled,
};
use crate::floating_ui::types::{Dimensions, Orientation};

/// `CompositeGridItemSize` (`packages/react/src/internals/composite/root/
/// gridNavigation.ts:18-21`) — the per-item cell span. The same `{ width, height }`
/// record the floating-ui cell-map helpers pack.
pub type CompositeGridItemSize = Dimensions;

/// `CompositeGridConfig` (`gridNavigation.ts:23-27`).
pub struct CompositeGridConfig {
    /// `cols` (`:24`).
    pub cols: i32,
    /// `dense` (`:25`) — upstream default `false`.
    pub dense: bool,
    /// `itemSizes` (`:26`) — per-item spans; [`None`] is upstream's `undefined`, which
    /// packs every item as a uniform 1×1 cell.
    pub item_sizes: Option<Vec<CompositeGridItemSize>>,
}

/// `CompositeGridNavigationState` (`gridNavigation.ts:29-40`) — the per-keystroke state
/// the composite root hands the navigator. Borrowed; the navigator never outlives the
/// dispatch. `event` splits into [`Self::key`] + [`Self::event`] per the module docs.
pub struct CompositeGridNavigationState<'a> {
    /// `event.key` — read directly by the corner selection (`:114-118`).
    pub key: &'a str,
    /// `event` (`:30`) — handed through to `getGridNavigatedIndex` (only read by its
    /// `stopEvent`/`onLoop` paths); `None` on the host target.
    pub event: Option<&'a KeyboardEvent>,
    /// `elementsRef` (`:31`).
    pub elements_ref: &'a CompositeListElementsRef,
    /// `highlightedIndex` (`:32`).
    pub highlighted_index: i32,
    /// `minIndex` (`:33`).
    pub min_index: i32,
    /// `maxIndex` (`:34`).
    pub max_index: i32,
    /// `orientation` (`:35`).
    pub orientation: Orientation,
    /// `loopFocus` (`:36`).
    pub loop_focus: bool,
    /// `onLoop` (`:37`) — the cell-level (3-argument) shape; see the module docs.
    pub on_loop: Option<&'a dyn Fn(Option<&KeyboardEvent>, i32, i32) -> i32>,
    /// `disabledIndices` (`:38`) — the item-level set.
    pub disabled_indices: Option<&'a DisabledIndices>,
    /// `rtl` (`:39`).
    pub rtl: bool,
}

/// `CompositeGridNavigator` (`gridNavigation.ts:42`) — the `grid` parameter type the
/// composite root invokes per keystroke.
pub type CompositeGridNavigator = Rc<dyn Fn(&CompositeGridNavigationState) -> i32>;

/// Port of `gridNavigation` (`gridNavigation.ts:50-126`): builds the grid navigator over
/// `config`. See the module docs for the algorithm and the two documented adaptations.
pub fn grid_navigation(config: CompositeGridConfig) -> CompositeGridNavigator {
    let CompositeGridConfig {
        cols,
        dense,
        item_sizes,
    } = config;
    // The optional explicit sizes are shared into the navigator (`None` packs uniform
    // 1x1 cells per dispatch, from the live elements length).
    let item_sizes = item_sizes.map(Rc::new);

    Rc::new(move |state: &CompositeGridNavigationState| {
        // `*state` destructure — every field is `Copy` or a shared reference, and the
        // non-reference bindings keep the arithmetic below honest about ownership.
        let CompositeGridNavigationState {
            key,
            event,
            elements_ref,
            highlighted_index,
            min_index,
            max_index,
            orientation,
            loop_focus,
            on_loop,
            disabled_indices,
            rtl,
        } = *state;

        let elements = elements_ref.borrow();

        // `const sizes = itemSizes || Array.from({length: elementsRef.current.length}, () =>
        // ({width: 1, height: 1}))` (`:67-72`).
        let sizes: Vec<CompositeGridItemSize> = match &item_sizes {
            Some(sizes) => (**sizes).clone(),
            None => vec![
                Dimensions {
                    width: 1.0,
                    height: 1.0,
                };
                elements.len()
            ],
        };

        // Work in hypothetical 1x1 cell indices, then convert back to item indices
        // (`:73-74`).
        let cell_map = create_grid_cell_map(&sizes, cols, dense);

        // `minGridIndex` (`:75-77`) — the first cell whose item exists and is not
        // disabled (item-level check; `findIndex` → `-1` when none).
        let min_grid_index = cell_map
            .iter()
            .position(|index| {
                index
                    .map(|item| !is_list_index_disabled(&elements, item as i32, disabled_indices))
                    .unwrap_or(false)
            })
            .map_or(-1, |position| position as i32);

        // `maxGridIndex` (`:78-84`) — the reduce keeps the *last* matching cell.
        let max_grid_index = cell_map
            .iter()
            .rposition(|index| {
                index
                    .map(|item| !is_list_index_disabled(&elements, item as i32, disabled_indices))
                    .unwrap_or(false)
            })
            .map_or(-1, |position| position as i32);

        // The cell-ordered element list `getGridNavigatedIndex` navigates over
        // (`:86`): `cellMap.map((itemIndex) => itemIndex != null ?
        // elementsRef.current[itemIndex] : null)`.
        let cell_elements: Vec<Option<web_sys::Element>> = cell_map
            .iter()
            .map(|item_index| {
                item_index.and_then(|item_index| elements.get(item_index).cloned().flatten())
            })
            .collect();

        // The cell-level disabled set (`:93-103`): the item-level disabled indices (or
        // the per-index DOM-derived fallback when none were given) mapped through
        // `getGridCellIndices` into cell indices, plus the trailing `undefined` gap
        // marker that disables every unoccupied cell. See the module docs for the
        // predicate-arm quirk.
        let mut item_markers: Vec<Option<usize>> = match disabled_indices {
            Some(DisabledIndices::List(indices)) => indices
                .iter()
                .filter_map(|&index| usize::try_from(index).ok())
                .map(Some)
                .collect(),
            Some(DisabledIndices::Predicate(_)) => Vec::new(),
            None => elements
                .iter()
                .enumerate()
                .filter(|(index, _)| is_list_index_disabled(&elements, *index as i32, None))
                .map(|(index, _)| Some(index))
                .collect(),
        };
        item_markers.push(None);
        let disabled_cells: Vec<i32> = get_grid_cell_indices(&item_markers, &cell_map)
            .into_iter()
            .map(|cell| cell as i32)
            .collect();

        // The corner of the spanning item closest to the movement direction
        // (`:106-119`) — so spanning items do not immediately resolve back to
        // themselves.
        let corner = if key == crate::composite::ARROW_DOWN {
            GridCorner::BottomLeft
        } else if key
            == if rtl {
                crate::composite::ARROW_LEFT
            } else {
                crate::composite::ARROW_RIGHT
            }
        {
            GridCorner::TopRight
        } else {
            GridCorner::TopLeft
        };
        let cell_index = get_grid_navigated_index(
            &cell_elements,
            crate::floating_ui::composite::GridNavigatedIndexOptions {
                key,
                event,
                orientation,
                loop_focus,
                on_loop,
                rtl,
                cols,
                // Treat undefined gaps as disabled so navigation cannot land in them
                // (`:93`).
                disabled_indices: Some(&DisabledIndices::List(disabled_cells)),
                min_index: min_grid_index,
                max_index: max_grid_index,
                // `highlightedIndex > maxIndex ? minIndex : highlightedIndex`
                // (`:107`) — an out-of-range highlight re-anchors to the grid's first
                // item.
                prev_index: get_grid_cell_index_of_corner(
                    if highlighted_index > max_index {
                        min_index
                    } else {
                        highlighted_index
                    },
                    &sizes,
                    &cell_map,
                    cols,
                    corner,
                ),
                stop: false,
            },
        );

        // `cellMap[cellIndex] as number` (`:124-125`) — see the module docs for the
        // unoccupied-arm adaptation.
        cell_map
            .get(cell_index as usize)
            .cloned()
            .flatten()
            .map_or(-1, |item_index| item_index as i32)
    })
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::KeyboardEvent;

    use super::*;
    use crate::composite_list::CompositeListElementsRef;

    fn list(len: usize) -> CompositeListElementsRef {
        Rc::new(RefCell::new((0..len).map(|_| None).collect()))
    }

    fn key_event(key: &str) -> KeyboardEvent {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(key);
        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap()
    }

    fn state<'a>(
        key: &'a str,
        event: Option<&'a KeyboardEvent>,
        elements_ref: &'a CompositeListElementsRef,
        highlighted_index: i32,
        orientation: Orientation,
    ) -> CompositeGridNavigationState<'a> {
        CompositeGridNavigationState {
            key,
            event,
            elements_ref,
            highlighted_index,
            min_index: 0,
            max_index: elements_ref.borrow().len() as i32 - 1,
            orientation,
            loop_focus: true,
            on_loop: None,
            disabled_indices: None,
            rtl: false,
        }
    }

    // Pins the uniform 1x1 packing (`gridNavigation.ts:67-74` with `itemSizes` absent):
    // every item occupies exactly one cell, so cell indices equal item indices and a
    // `cols: 3` navigator steps by row.
    #[wasm_bindgen_test]
    fn uniform_items_navigate_by_row() {
        let elements = list(6);
        let navigator = grid_navigation(CompositeGridConfig {
            cols: 3,
            dense: false,
            item_sizes: None,
        });

        let event = key_event("ArrowDown");
        let down = navigator(&state(
            "ArrowDown",
            Some(&event),
            &elements,
            0,
            Orientation::Both,
        ));
        assert_eq!(down, 3, "ArrowDown from cell 0 lands a full row below");
        let up = navigator(&state(
            "ArrowUp",
            Some(&event),
            &elements,
            3,
            Orientation::Both,
        ));
        assert_eq!(up, 0, "ArrowUp returns to the row above");
        let right = navigator(&state(
            "ArrowRight",
            Some(&event),
            &elements,
            0,
            Orientation::Both,
        ));
        assert_eq!(right, 1, "ArrowRight steps one cell");
    }

    // Pins the gap disabling (`:93-103`): a spanning item leaves unoccupied cells, and
    // navigation may not land in them — the 2x2 item at index 0 in a 3-column grid
    // occupies cells 0/1/3/4 and the 1x1 item fills cell 2, so ArrowRight from the
    // spanning item's top-right corner (cell 1) steps into cell 2 and lands item 1
    // rather than a gap.
    #[wasm_bindgen_test]
    fn spanning_items_leave_disabled_gaps() {
        let elements = list(2);
        let navigator = grid_navigation(CompositeGridConfig {
            cols: 3,
            dense: false,
            item_sizes: Some(vec![
                Dimensions {
                    width: 2.0,
                    height: 2.0,
                },
                Dimensions {
                    width: 1.0,
                    height: 1.0,
                },
            ]),
        });

        let event = key_event("ArrowRight");
        let next = navigator(&state(
            "ArrowRight",
            Some(&event),
            &elements,
            0,
            Orientation::Both,
        ));
        assert_eq!(next, 1, "ArrowRight steps into the occupied cell next door");
    }

    // Pins the dense packing (`CompositeRoot.test.tsx:764-802` motivates it): two
    // 2x1 items in a 3-column grid leave a gap at cell 2 in sparse mode (the persisted
    // `startIndex` walks past it), and dense mode resets the scan to 0 so the trailing
    // 1x1 item back-fills it — which flips ArrowRight from the first item from "wrap
    // back to itself" (the gap is disabled, the next row's re-find lands cell 0) to
    // "step into the filled gap".
    #[wasm_bindgen_test]
    fn dense_packing_fills_earlier_gaps() {
        let elements = list(3);
        let sizes = vec![
            Dimensions {
                width: 2.0,
                height: 1.0,
            },
            Dimensions {
                width: 2.0,
                height: 1.0,
            },
            Dimensions {
                width: 1.0,
                height: 1.0,
            },
        ];
        let sparse = grid_navigation(CompositeGridConfig {
            cols: 3,
            dense: false,
            item_sizes: Some(sizes.clone()),
        });
        let dense = grid_navigation(CompositeGridConfig {
            cols: 3,
            dense: true,
            item_sizes: Some(sizes),
        });

        let event = key_event("ArrowRight");
        // Sparse cell map: [0,0,_,1,1,2] — cell 2 is a disabled gap, so ArrowRight from
        // item 0 (top-right corner, cell 1) cannot land there and wraps to the row
        // start (item 0 itself).
        assert_eq!(
            sparse(&state(
                "ArrowRight",
                Some(&event),
                &elements,
                0,
                Orientation::Both
            )),
            0,
            "sparse mode: the gap breaks the rightward step and the move re-anchors"
        );
        // Dense cell map: [0,0,2,1,1,_] — item 2 fills cell 2, so the same keystroke
        // lands it.
        assert_eq!(
            dense(&state(
                "ArrowRight",
                Some(&event),
                &elements,
                0,
                Orientation::Both
            )),
            2,
            "dense mode: the trailing item back-fills cell 2 and receives the focus"
        );
    }

    // Pins the corner selection (`:106-119`): moving down from a spanning item anchors
    // at its bottom-left corner, so a second spanning row does not resolve back to the
    // start item.
    #[wasm_bindgen_test]
    fn corner_selection_uses_the_movement_direction() {
        let elements = list(2);
        let navigator = grid_navigation(CompositeGridConfig {
            cols: 2,
            dense: false,
            item_sizes: Some(vec![
                Dimensions {
                    width: 2.0,
                    height: 2.0,
                },
                Dimensions {
                    width: 2.0,
                    height: 2.0,
                },
            ]),
        });

        let down = key_event("ArrowDown");
        assert_eq!(
            navigator(&state(
                "ArrowDown",
                Some(&down),
                &elements,
                0,
                Orientation::Both
            )),
            1,
            "ArrowDown anchors the bottom-left corner and lands the next row's item"
        );
    }
}
