//! Port of `packages/react/src/floating-ui-react/hooks/gridNavigation.ts` — the
//! positional-argument shim grid-capable consumers inject into
//! [`crate::floating_ui::use_list_navigation`]'s `grid` option
//! (`specs/library/floating-ui-react/implementation.md`, "Notable per-hook state
//! machines": "Grid navigation is opt-in via the injected `grid` function so non-grid
//! consumers tree-shake the helpers").
//!
//! The injected grid navigator only ever operates on a uniform 1x1 grid (sizes are
//! always `1x1` and packing is never dense), so the cell-map machinery that supports
//! multi-cell items collapses to an identity transform over the item list. Calling
//! [`crate::floating_ui::composite::get_grid_navigated_index`] directly keeps the
//! cell-map helpers out of grid-combobox bundles (`gridNavigation.ts:8-19`).
//!
//! ## Rust adaptations
//!
//! - The positional-argument signature is deliberate upstream (`gridNavigation.ts:8-12`
//!   — property names don't minify, and the signature is locked to the caller via
//!   `typeof`); the port locks it the same way through the [`GridNavigationFn`] alias
//!   the hook's `grid` option accepts.
//! - The `cols = 2` default parameter (`gridNavigation.ts:30`) has no Rust analog on a
//!   positional fn; it lives in [`GRID_NAVIGATION_DEFAULT_COLS`], which
//!   [`crate::floating_ui::use_list_navigation`]'s call site passes (upstream's call
//!   site omits the argument, `hooks/useListNavigation.ts:592-602`). The free
//!   [`grid_navigation`] keeps the explicit `cols` parameter so injected custom
//!   navigators can differ.
//! - The `event` parameter rides as `Option<&KeyboardEvent>` through the composite
//!   core (see [`crate::floating_ui::composite`] module docs); the host-testable
//!   [`grid_navigation_with_key`] takes the key separately.

use std::cell::RefCell;
use std::rc::Rc;

use web_sys::KeyboardEvent;

use crate::floating_ui::composite::{
    DisabledIndices, GridNavigatedIndexOptions, get_grid_navigated_index,
    is_index_out_of_list_bounds,
};
use crate::floating_ui::types::Orientation;

/// The consumer-owned item-element list — upstream's
/// `React.RefObject<Array<HTMLElement | null>>` (`gridNavigation.ts:23`): the same
/// shape [`crate::floating_ui::use_list_navigation`] owns. A `None` slot is upstream's
/// `null` entry.
pub type ListRef = Rc<RefCell<Vec<Option<web_sys::Element>>>>;

/// The default `cols` (`gridNavigation.ts:30`) — the call site in
/// `useListNavigation` omits the argument, so the shim always runs a 2-column grid.
pub const GRID_NAVIGATION_DEFAULT_COLS: i32 = 2;

/// The `typeof gridNavigation` shape `useListNavigation`'s `grid` option accepts
/// (`hooks/useListNavigation.ts:221`, call site `:592-602`) — positional arguments,
/// in upstream's order.
pub type GridNavigationFn = Rc<
    dyn Fn(
        &KeyboardEvent,
        i32,
        &ListRef,
        Orientation,
        bool,
        bool,
        Option<DisabledIndices>,
        i32,
        i32,
    ) -> Option<i32>,
>;

/// Port of `gridNavigation` (`gridNavigation.ts:20-49`) with the key/event split (see
/// the module docs): computes the next index on the uniform 1x1 grid.
///
/// - An out-of-range previous index falls back to the first enabled item
///   (`gridNavigation.ts:42`).
/// - `getGridNavigatedIndex` can return an out-of-bounds sentinel (e.g. `-1` when
///   there is no previous item to move from); that surfaces as `None` so the caller
///   treats it as "no navigation" rather than highlighting index `-1`
///   (`gridNavigation.ts:46-49`).
pub fn grid_navigation_with_key(
    key: &str,
    event: Option<&KeyboardEvent>,
    prev_index: i32,
    list_ref: &ListRef,
    orientation: Orientation,
    loop_focus: bool,
    rtl: bool,
    disabled_indices: Option<DisabledIndices>,
    min_index: i32,
    max_index: i32,
    cols: i32,
) -> Option<i32> {
    let list = list_ref.borrow();
    let next_index = get_grid_navigated_index(
        &list,
        GridNavigatedIndexOptions {
            key,
            event,
            orientation,
            loop_focus,
            on_loop: None,
            rtl,
            cols,
            disabled_indices: disabled_indices.as_ref(),
            min_index,
            max_index,
            // An out-of-range previous index falls back to the first enabled item
            // (`gridNavigation.ts:42`).
            prev_index: if prev_index > max_index {
                min_index
            } else {
                prev_index
            },
            stop: true,
        },
    );

    // `getGridNavigatedIndex` can return an out-of-bounds sentinel (e.g. `-1` when
    // there is no previous item to move from); surface that as `None` so the caller
    // treats it as "no navigation" rather than highlighting index `-1`
    // (`gridNavigation.ts:46-49`).
    (!is_index_out_of_list_bounds(&list, next_index)).then_some(next_index)
}

/// The event-taking shape [`GridNavigationFn`] locks to — the shim itself, ready for
/// injection.
pub fn grid_navigation(
    event: &KeyboardEvent,
    prev_index: i32,
    list_ref: &ListRef,
    orientation: Orientation,
    loop_focus: bool,
    rtl: bool,
    disabled_indices: Option<DisabledIndices>,
    min_index: i32,
    max_index: i32,
) -> Option<i32> {
    grid_navigation_with_key(
        &event.key(),
        Some(event),
        prev_index,
        list_ref,
        orientation,
        loop_focus,
        rtl,
        disabled_indices,
        min_index,
        max_index,
        GRID_NAVIGATION_DEFAULT_COLS,
    )
}

/// The [`GridNavigationFn`] handle wrapping [`grid_navigation`] — the value Menu- and
/// Combobox-shaped consumers inject (`hooks/useListNavigation.ts:221`).
pub fn grid_navigation_fn() -> GridNavigationFn {
    Rc::new(grid_navigation)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    use crate::floating_ui::composite::GridNavigatedIndexOptions;

    fn none_list(len: usize) -> Vec<Option<web_sys::Element>> {
        vec![None; len]
    }

    // Mirrors the `useListNavigation` call site (`hooks/useListNavigation.ts:592-602`):
    // the shim locks the positional signature and the default 2-column grid, and the
    // out-of-bounds sentinel surfaces as `None` ("no navigation").
    #[test]
    fn the_shim_uses_the_default_two_column_grid_and_reports_no_navigation_out_of_bounds() {
        // 2 columns: ArrowDown from 0 strides to 2.
        let list_ref: ListRef = Rc::new(RefCell::new(none_list(4)));
        assert_eq!(
            grid_navigation_with_key(
                "ArrowDown",
                None,
                0,
                &list_ref,
                Orientation::Vertical,
                false,
                false,
                None,
                0,
                3,
                GRID_NAVIGATION_DEFAULT_COLS
            ),
            Some(2)
        );

        // From -1 (no previous item): ArrowDown resolves to the min, ArrowUp to the
        // max — both in bounds.
        assert_eq!(
            grid_navigation_with_key(
                "ArrowDown",
                None,
                -1,
                &list_ref,
                Orientation::Vertical,
                false,
                false,
                None,
                0,
                3,
                GRID_NAVIGATION_DEFAULT_COLS
            ),
            Some(0)
        );
        assert_eq!(
            grid_navigation_with_key(
                "ArrowUp",
                None,
                -1,
                &list_ref,
                Orientation::Vertical,
                false,
                false,
                None,
                0,
                3,
                GRID_NAVIGATION_DEFAULT_COLS
            ),
            Some(3)
        );

        // A non-arrow key leaves the index — in bounds, so `Some(prev)`.
        assert_eq!(
            grid_navigation_with_key(
                "Home",
                None,
                1,
                &list_ref,
                Orientation::Vertical,
                false,
                false,
                None,
                0,
                3,
                GRID_NAVIGATION_DEFAULT_COLS
            ),
            Some(1)
        );
    }

    // Pins the out-of-range previous-index fallback (`gridNavigation.ts:42`): a
    // `prevIndex` past `maxIndex` restarts from `minIndex`.
    #[test]
    fn an_out_of_range_previous_index_falls_back_to_the_min() {
        let list_ref: ListRef = Rc::new(RefCell::new(none_list(4)));
        assert_eq!(
            grid_navigation_with_key(
                "Home",
                None,
                9,
                &list_ref,
                Orientation::Vertical,
                false,
                false,
                None,
                0,
                3,
                GRID_NAVIGATION_DEFAULT_COLS
            ),
            Some(0),
            "prevIndex 9 > maxIndex 3 restarts at minIndex, and the non-arrow key keeps it"
        );
    }

    // Pins the `disabledIndices` pass-through: the shim forwards the explicit set to
    // `getGridNavigatedIndex`, so a disabled stride target cancels the move (the
    // walk exits past the list end and the bounds reset restores the previous
    // index), while the same move goes through with no set.
    #[test]
    fn the_shim_forwards_disabled_indices() {
        let list_ref: ListRef = Rc::new(RefCell::new(none_list(4)));
        assert_eq!(
            grid_navigation_with_key(
                "ArrowDown",
                None,
                0,
                &list_ref,
                Orientation::Vertical,
                false,
                false,
                None,
                0,
                3,
                GRID_NAVIGATION_DEFAULT_COLS
            ),
            Some(2),
            "down from 0 strides to 2 with no explicit set"
        );
        let disabled = DisabledIndices::List(vec![2]);
        assert_eq!(
            grid_navigation_with_key(
                "ArrowDown",
                None,
                0,
                &list_ref,
                Orientation::Vertical,
                false,
                false,
                Some(disabled),
                0,
                3,
                GRID_NAVIGATION_DEFAULT_COLS
            ),
            Some(0),
            "the disabled stride target pushes the walk past the list end, so the move cancels"
        );
    }
}
