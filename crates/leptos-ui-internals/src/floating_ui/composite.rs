//! Port of `packages/react/src/floating-ui-react/utils/composite.ts` — the
//! list-navigation helpers: the [`DisabledIndices`] vocabulary,
//! [`is_list_index_disabled`], the visibility pair
//! ([`is_element_visible`]/[`is_hidden_by_styles`]), and the grid-navigation matrix
//! ([`find_non_disabled_list_index`], [`get_grid_navigated_index`], the cell-map
//! trio) consumed by [`crate::floating_ui::use_typeahead`],
//! [`crate::floating_ui::use_list_navigation`], and
//! [`crate::floating_ui::grid_navigation`]
//! (`specs/library/floating-ui-react/implementation.md`, "Notable per-hook state
//! machines": grid navigation "is a positional-arg shim … with the row-structure
//! inference and virtualized-gap logic living in `utils/composite`").
//!
//! ## Rust adaptations
//!
//! - `DisabledIndices` (`composite.ts:7` — `ReadonlyArray<number> |
//!   ((index: number) => boolean)`) becomes [`DisabledIndices`]: a list arm and a
//!   predicate arm. Indices are `i32` so the `-1` "no index" sentinel the navigation
//!   hooks use flows through unchanged.
//! - `isElementVisible(element, styles?)` (`composite.ts:511-527`) folds its optional
//!   computed-styles parameter into the function body — every current caller resolves
//!   the element's own styles, and a Rust caller holding styles for a *different*
//!   element would be a bug the JS signature allows silently. `checkVisibility` keeps
//!   its upstream feature-detect (`:520`) — resolved through `Reflect` so a browser
//!   without the API (or a polyfill-less runtime) takes the manual
//!   `display !== 'none' && display !== 'contents'` fallback exactly as upstream.
//! - `text[0]`/`text[1]` indexing in the typeahead's doubled-letter bail-out is
//!   UTF-16-code-unit indexing upstream; the port compares the first two `char`s —
//!   identical for every realistic list label, differing only for astral-plane
//!   leading characters (see [`crate::floating_ui::use_typeahead`]).
//! - `element.matches(':disabled')` (`composite.ts:496`) can only throw for an
//!   invalid selector, and `:disabled` is valid — the port maps a hypothetical
//!   rejection to `false` rather than propagating a JsValue error.
//! - `Math.floor(index / cols)` (`:10-12,202,287,362`) is floor division — Rust's
//!   `i32::div_euclid` for a positive divisor, not the truncating `/` operator.
//! - `getGridNavigatedIndex`'s `event` parameter (`:77`) is only *read* — for
//!   `event.key` and to hand the same event to `stopEvent`/`onLoop`. The port splits
//!   those two uses: the key is a plain `&str` parameter and the event is an
//!   `Option<&KeyboardEvent>` (`None` on the host target, where no DOM event exists —
//!   the index math is event-independent). The two row-structure probes
//!   (`el.closest('[role="row"]')`, `:120`) need elements only on the vertical path.
//! - `createGridCellMap`'s dev-only `width > cols` throw (`:391-398`,
//!   `process.env.NODE_ENV !== 'production'`) ports to a panic: production's
//!   silent continuation is an infinite `while (!itemPlaced)` hang (`:403`), which
//!   the port refuses to reproduce — failing fast is the only safe translation of a
//!   guard whose release-mode absence is a hang.

use std::rc::Rc;

use web_sys::wasm_bindgen::JsCast;
use web_sys::{CssStyleDeclaration, Element, KeyboardEvent};

use crate::floating_ui::constants::{ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP};
use crate::floating_ui::event::stop_event;
use crate::floating_ui::types::Orientation;

/// Port of the `DisabledIndices` type (`composite.ts:7`): the explicit disabled set,
/// either as an array of indices or a per-index predicate — the same shape as
/// `useListNavigation`'s `disabledIndices`
/// (`packages/react/src/floating-ui-react/hooks/useTypeahead.ts:36-43`).
#[derive(Clone)]
pub enum DisabledIndices {
    /// The array arm — `ReadonlyArray<number>`.
    List(Vec<i32>),
    /// The predicate arm — `(index: number) => boolean`.
    Predicate(Rc<dyn Fn(i32) -> bool>),
}

/// Port of `isListIndexDisabled` (`composite.ts:471-505`): whether list entry
/// `index` must be skipped — explicitly disabled through `disabledIndices`, hidden,
/// natively disabled, or (only when no explicit set is given) carrying a
/// `disabled`/`aria-disabled="true"` attribute.
pub fn is_list_index_disabled(
    list: &[Option<Element>],
    index: i32,
    disabled_indices: Option<&DisabledIndices>,
) -> bool {
    let is_explicitly_disabled = match disabled_indices {
        Some(DisabledIndices::Predicate(predicate)) => predicate(index),
        Some(DisabledIndices::List(indices)) => indices.contains(&index),
        None => false,
    };

    if is_explicitly_disabled {
        return true;
    }

    // JS `list[index]` on an out-of-bounds (or negative) index reads `undefined` —
    // the element-less arm (`composite.ts:480-482`).
    let element = if index < 0 {
        None
    } else {
        list.get(index as usize).cloned().flatten()
    };
    let Some(element) = element else {
        return false;
    };

    if !is_element_visible(Some(&element)) {
        return true;
    }

    // A natively disabled element can never receive focus, so it must always be
    // skipped, even when `disabledIndices` marks it as enabled. Only
    // `aria-disabled` items can be focusable-while-disabled (`composite.ts:488-493`).
    if element.matches(":disabled").unwrap_or(false) {
        return true;
    }

    disabled_indices.is_none()
        && (element.has_attribute("disabled")
            || element.get_attribute("aria-disabled").as_deref() == Some("true"))
}

/// Port of `isHiddenByStyles` (`composite.ts:507-509`): the styles-level hidden
/// check — `visibility: hidden|collapse`.
pub fn is_hidden_by_styles(styles: &CssStyleDeclaration) -> bool {
    let visibility = styles.get_property_value("visibility").unwrap_or_default();
    visibility == "hidden" || visibility == "collapse"
}

/// Port of `isElementVisible` (`composite.ts:511-527`): the effective-visibility
/// check — connected, not visibility-hidden, and (through the `checkVisibility`
/// feature-detect or the display fallback) not display-hidden.
pub fn is_element_visible(element: Option<&Element>) -> bool {
    let Some(element) = element else {
        return false;
    };
    if !element.is_connected() {
        return false;
    }
    let styles = web_sys::window()
        .and_then(|window| window.get_computed_style(element).ok())
        .flatten();
    let Some(styles) = styles else {
        return false;
    };
    if is_hidden_by_styles(&styles) {
        return false;
    }

    // `typeof element.checkVisibility === 'function'` (`composite.ts:520`) — the
    // feature-detect resolves through Reflect so the binding needs no web-sys feature.
    let check_visibility = js_sys::Reflect::get(element.as_ref(), &"checkVisibility".into())
        .ok()
        .filter(|value| value.is_function());
    if let Some(function) =
        check_visibility.and_then(|value| value.dyn_into::<js_sys::Function>().ok())
    {
        return js_sys::Reflect::apply(&function, element.as_ref(), &js_sys::Array::new())
            .map(|result| result.is_truthy())
            .unwrap_or(false);
    }

    let display = styles.get_property_value("display").unwrap_or_default();
    display != "none" && display != "contents"
}

// ---------------------------------------------------------------------------
// The grid half (`composite.ts:10-60,62-383,385-469`) — the row-structure
// inference, the vertical/horizontal navigation matrix, and the cell-map
// helpers the multi-cell grid consumers ride.
// ---------------------------------------------------------------------------

/// Port of `isDifferentGridRow` (`composite.ts:10-12`): whether `index` sits on a
/// different `cols`-wide grid row than `prev_row` (`Math.floor(index / cols)`).
pub fn is_different_grid_row(index: i32, cols: i32, prev_row: i32) -> bool {
    index.div_euclid(cols) != prev_row
}

/// Port of `isIndexOutOfListBounds` (`composite.ts:14-16`): a negative index or one
/// at/past the list length.
pub fn is_index_out_of_list_bounds(list: &[Option<Element>], index: i32) -> bool {
    index < 0 || index as usize >= list.len()
}

/// The options object of `findNonDisabledListIndex` (`composite.ts:36-49`), with the
/// upstream defaults documented per field.
pub struct FindNonDisabledListIndexOptions<'a> {
    /// `startingIndex` (`:39` — default `-1`).
    pub starting_index: i32,
    /// `decrement` (`:40` — default `false`).
    pub decrement: bool,
    /// `disabledIndices` (`:41`).
    pub disabled_indices: Option<&'a DisabledIndices>,
    /// `amount` (`:42` — default `1`).
    pub amount: i32,
}

impl<'a> FindNonDisabledListIndexOptions<'a> {
    /// The upstream defaults — `startingIndex: -1`, `decrement: false`, `amount: 1`.
    pub fn new(disabled_indices: Option<&'a DisabledIndices>) -> Self {
        Self {
            starting_index: -1,
            decrement: false,
            disabled_indices,
            amount: 1,
        }
    }
}

/// Port of `getMinListIndex` (`composite.ts:18-23`): the first non-disabled index
/// from the list head.
pub fn get_min_list_index(
    list: &[Option<Element>],
    disabled_indices: Option<&DisabledIndices>,
) -> i32 {
    find_non_disabled_list_index(list, FindNonDisabledListIndexOptions::new(disabled_indices))
}

/// Port of `getMaxListIndex` (`composite.ts:25-34`): the last non-disabled index,
/// searched downward from `list.length` (one past the end, mirroring upstream's
/// `startingIndex: listRef.current.length`).
pub fn get_max_list_index(
    list: &[Option<Element>],
    disabled_indices: Option<&DisabledIndices>,
) -> i32 {
    find_non_disabled_list_index(
        list,
        FindNonDisabledListIndexOptions {
            starting_index: list.len() as i32,
            decrement: true,
            ..FindNonDisabledListIndexOptions::new(disabled_indices)
        },
    )
}

/// Port of `findNonDisabledListIndex` (`composite.ts:36-60`): steps `index` by
/// `amount` (downward when `decrement`) until it lands on a non-disabled entry or
/// leaves the list — the do-while's exit condition is the returned sentinel.
pub fn find_non_disabled_list_index(
    list: &[Option<Element>],
    options: FindNonDisabledListIndexOptions,
) -> i32 {
    let FindNonDisabledListIndexOptions {
        starting_index,
        decrement,
        disabled_indices,
        amount,
    } = options;

    let mut index = starting_index;
    loop {
        index += if decrement { -amount } else { amount };
        if !(index >= 0
            && index <= list.len() as i32 - 1
            && is_list_index_disabled(list, index, disabled_indices))
        {
            break;
        }
    }

    index
}

/// The options object of `getGridNavigatedIndex` (`composite.ts:62-89`), split per
/// the module docs: `event.key` is [`Self::key`], the event itself is
/// [`Self::event`] (only handed to `stopEvent`/`onLoop`).
pub struct GridNavigatedIndexOptions<'a> {
    /// `event.key` (`:77`).
    pub key: &'a str,
    /// The `event` (`:77`) — `None` on the host target, where the index math is
    /// event-independent (see the module docs).
    pub event: Option<&'a KeyboardEvent>,
    /// `orientation` (`:78`).
    pub orientation: Orientation,
    /// `loopFocus` (`:79`).
    pub loop_focus: bool,
    /// `onLoop` (`:80-81`) — receives `(event, prevIndex, nextIndex)` and returns the
    /// index to continue with; the port's event arm is `Option<&KeyboardEvent>`.
    pub on_loop: Option<&'a dyn Fn(Option<&KeyboardEvent>, i32, i32) -> i32>,
    /// `rtl` (`:82`).
    pub rtl: bool,
    /// `cols` (`:83`).
    pub cols: i32,
    /// `disabledIndices` (`:84`).
    pub disabled_indices: Option<&'a DisabledIndices>,
    /// `minIndex` (`:85`).
    pub min_index: i32,
    /// `maxIndex` (`:86`).
    pub max_index: i32,
    /// `prevIndex` (`:87`).
    pub prev_index: i32,
    /// `stopEvent` (`:88` — default `false`).
    pub stop: bool,
}

/// Port of `getGridNavigatedIndex` (`composite.ts:62-383`): the two-dimensional
/// navigation matrix. Vertical keys first infer the row structure
/// (`[role="row"]` DOM rows vs. the uniform `cols` grid with virtualized gaps),
/// then fall back to a `cols`-wide column walk; horizontal keys stay on the
/// current row, looping or clamping per `loopFocus`.
pub fn get_grid_navigated_index(
    list: &[Option<Element>],
    options: GridNavigatedIndexOptions,
) -> i32 {
    let GridNavigatedIndexOptions {
        key,
        event,
        orientation,
        loop_focus,
        on_loop,
        rtl,
        cols,
        disabled_indices,
        min_index,
        max_index,
        prev_index,
        stop,
    } = options;

    let mut next_index = prev_index;

    let vertical_direction = if key == ARROW_UP {
        Some(false)
    } else if key == ARROW_DOWN {
        Some(true)
    } else {
        None
    };

    if let Some(is_down) = vertical_direction {
        let direction = if is_down { "down" } else { "up" };

        // -------------------------------------------------------------------------
        // Detect row structure only when handling vertical navigation. This keeps
        // the non-vertical key paths free from row inference work.
        // (`composite.ts:101-104`)
        // -------------------------------------------------------------------------
        let mut rows: Vec<Vec<usize>> = Vec::new();
        let mut row_index_map: Vec<Option<i32>> = vec![None; list.len()];
        let mut has_role_row = false;
        let mut visible_item_count = 0usize;
        {
            let mut current_row_el: Option<Element> = None;
            let mut current_row_index: i32 = -1;

            for (idx, el) in list.iter().enumerate() {
                let Some(el) = el else {
                    continue;
                };

                visible_item_count += 1;

                let row_el = el.closest("[role=\"row\"]").ok().flatten();
                if row_el.is_some() {
                    has_role_row = true;
                }

                // `rowEl !== currentRowEl || currentRowIndex === -1`
                // (`composite.ts:125`): a different row element (or the first item)
                // opens a new row. The `Option<&Element>` equality is the JsValue
                // `===` (two wrappers of the same DOM object compare equal).
                let row_el_matches_current = match (&row_el, &current_row_el) {
                    (Some(row_el), Some(current)) => row_el == current,
                    _ => false,
                };
                if !row_el_matches_current || current_row_index == -1 {
                    current_row_el = row_el;
                    current_row_index += 1;
                    rows.push(Vec::new());
                }
                rows[current_row_index as usize].push(idx);
                row_index_map[idx] = Some(current_row_index);
            }
        }

        let mut has_dom_rows = false;
        let mut inferred_dom_cols: usize = 0;

        if has_role_row {
            for row in &rows {
                let row_length = row.len();

                if row_length > inferred_dom_cols {
                    inferred_dom_cols = row_length;
                }

                if row_length != cols as usize {
                    has_dom_rows = true;
                }
            }
        }

        let has_virtualized_gaps = has_dom_rows && visible_item_count < list.len();
        let vertical_cols = if inferred_dom_cols != 0 {
            inferred_dom_cols as i32
        } else {
            cols
        };

        // `navigateVertically` (`composite.ts:155-192`): the real-DOM-row walk.
        let navigate_vertically = |direction: &str| -> Option<i32> {
            if !has_dom_rows || prev_index == -1 {
                return None;
            }

            let current_row = row_index_map[prev_index as usize]? as usize;
            let row = rows.get(current_row)?;
            let col_in_row = row.iter().position(|&idx| idx == prev_index as usize)?;
            let step: i64 = if direction == "up" { -1 } else { 1 };

            let mut next_row = current_row as i64 + step;
            let mut i = 0usize;
            while i < rows.len() {
                if next_row < 0 || next_row >= rows.len() as i64 {
                    if !loop_focus || has_virtualized_gaps {
                        return None;
                    }
                    next_row = if next_row < 0 {
                        rows.len() as i64 - 1
                    } else {
                        0
                    };
                    if let Some(on_loop) = on_loop {
                        let target_row = &rows[next_row as usize];
                        let clamped_col = (col_in_row).min(target_row.len() - 1);
                        let target_item_index = target_row
                            .get(clamped_col)
                            .or_else(|| target_row.first())
                            .copied()
                            .unwrap_or(0) as i32;
                        let returned_item_index = on_loop(event, prev_index, target_item_index);
                        next_row = row_index_map
                            .get(returned_item_index as usize)
                            .cloned()
                            .flatten()
                            .map(|row| row as i64)
                            .unwrap_or(next_row);
                    }
                }

                let target_row = &rows[next_row as usize];
                let mut col = (col_in_row).min(target_row.len() - 1) as i64;
                while col >= 0 {
                    let candidate = target_row[col as usize];
                    if !is_list_index_disabled(list, candidate as i32, disabled_indices) {
                        return Some(candidate as i32);
                    }
                    col -= 1;
                }

                next_row += step;
                i += 1;
            }

            None
        };

        // `navigateVerticallyWithInferredRows` (`composite.ts:194-229`): the
        // uniform-grid walk used when the list has virtualized gaps.
        let navigate_vertically_with_inferred_rows = |direction: &str| -> Option<i32> {
            if !has_virtualized_gaps || prev_index == -1 {
                return None;
            }

            let col_in_row = prev_index % vertical_cols;
            let row_step: i64 = if direction == "up" {
                -(vertical_cols as i64)
            } else {
                vertical_cols as i64
            };
            let last_row_start = max_index - (max_index % vertical_cols);
            let row_count = max_index.div_euclid(vertical_cols) + 1;

            let mut row_start: i64 = prev_index as i64 - col_in_row as i64 + row_step;
            let mut i: i32 = 0;
            while i < row_count {
                if row_start < 0 || row_start > max_index as i64 {
                    if !loop_focus {
                        return None;
                    }
                    row_start = if row_start < 0 {
                        last_row_start as i64
                    } else {
                        0
                    };
                }

                let row_end = (row_start + vertical_cols as i64 - 1).min(max_index as i64);
                let mut candidate = (row_start + col_in_row as i64).min(row_end);
                while candidate >= row_start {
                    if !is_list_index_disabled(list, candidate as i32, disabled_indices) {
                        return Some(candidate as i32);
                    }
                    candidate -= 1;
                }

                row_start += row_step;
                i += 1;
            }

            None
        };

        if stop {
            if let Some(event) = event {
                stop_event(event);
            }
        }

        let vertical_candidate = navigate_vertically(direction)
            .or_else(|| navigate_vertically_with_inferred_rows(direction));

        if let Some(candidate) = vertical_candidate {
            next_index = candidate;
        } else if prev_index == -1 {
            next_index = if direction == "up" {
                max_index
            } else {
                min_index
            };
        } else {
            next_index = find_non_disabled_list_index(
                list,
                FindNonDisabledListIndexOptions {
                    starting_index: prev_index,
                    amount: vertical_cols,
                    decrement: direction == "up",
                    disabled_indices,
                },
            );

            if loop_focus {
                if direction == "up" && (prev_index - vertical_cols < min_index || next_index < 0) {
                    let col = prev_index % vertical_cols;
                    let max_col = max_index % vertical_cols;
                    let offset = max_index - (max_col - col);

                    if max_col == col {
                        next_index = max_index;
                    } else {
                        next_index = if max_col > col {
                            offset
                        } else {
                            offset - vertical_cols
                        };
                    }
                    if let Some(on_loop) = on_loop {
                        next_index = on_loop(event, prev_index, next_index);
                    }
                }

                if direction == "down" && prev_index + vertical_cols > max_index {
                    next_index = find_non_disabled_list_index(
                        list,
                        FindNonDisabledListIndexOptions {
                            starting_index: (prev_index % vertical_cols) - vertical_cols,
                            amount: vertical_cols,
                            disabled_indices,
                            ..FindNonDisabledListIndexOptions::new(None)
                        },
                    );
                    if let Some(on_loop) = on_loop {
                        next_index = on_loop(event, prev_index, next_index);
                    }
                }
            }
        }

        if is_index_out_of_list_bounds(list, next_index) {
            next_index = prev_index;
        }
    }

    // Remains on the same row/column (`composite.ts:285-380`).
    if orientation == Orientation::Both {
        let prev_row = prev_index.div_euclid(cols);

        let forward_key = if rtl { ARROW_LEFT } else { ARROW_RIGHT };
        let backward_key = if rtl { ARROW_RIGHT } else { ARROW_LEFT };

        if key == forward_key {
            if stop {
                if let Some(event) = event {
                    stop_event(event);
                }
            }

            if prev_index % cols != cols - 1 {
                next_index = find_non_disabled_list_index(
                    list,
                    FindNonDisabledListIndexOptions {
                        starting_index: prev_index,
                        disabled_indices,
                        ..FindNonDisabledListIndexOptions::new(None)
                    },
                );

                if loop_focus && is_different_grid_row(next_index, cols, prev_row) {
                    next_index = find_non_disabled_list_index(
                        list,
                        FindNonDisabledListIndexOptions {
                            starting_index: prev_index - (prev_index % cols) - 1,
                            disabled_indices,
                            ..FindNonDisabledListIndexOptions::new(None)
                        },
                    );
                    if let Some(on_loop) = on_loop {
                        next_index = on_loop(event, prev_index, next_index);
                    }
                }
            } else if loop_focus {
                next_index = find_non_disabled_list_index(
                    list,
                    FindNonDisabledListIndexOptions {
                        starting_index: prev_index - (prev_index % cols) - 1,
                        disabled_indices,
                        ..FindNonDisabledListIndexOptions::new(None)
                    },
                );
                if let Some(on_loop) = on_loop {
                    next_index = on_loop(event, prev_index, next_index);
                }
            }

            if is_different_grid_row(next_index, cols, prev_row) {
                next_index = prev_index;
            }
        }

        if key == backward_key {
            if stop {
                if let Some(event) = event {
                    stop_event(event);
                }
            }

            if prev_index % cols != 0 {
                next_index = find_non_disabled_list_index(
                    list,
                    FindNonDisabledListIndexOptions {
                        starting_index: prev_index,
                        decrement: true,
                        disabled_indices,
                        ..FindNonDisabledListIndexOptions::new(None)
                    },
                );

                if loop_focus && is_different_grid_row(next_index, cols, prev_row) {
                    next_index = find_non_disabled_list_index(
                        list,
                        FindNonDisabledListIndexOptions {
                            starting_index: prev_index + (cols - (prev_index % cols)),
                            decrement: true,
                            disabled_indices,
                            ..FindNonDisabledListIndexOptions::new(None)
                        },
                    );
                    if let Some(on_loop) = on_loop {
                        next_index = on_loop(event, prev_index, next_index);
                    }
                }
            } else if loop_focus {
                next_index = find_non_disabled_list_index(
                    list,
                    FindNonDisabledListIndexOptions {
                        starting_index: prev_index + (cols - (prev_index % cols)),
                        decrement: true,
                        disabled_indices,
                        ..FindNonDisabledListIndexOptions::new(None)
                    },
                );
                if let Some(on_loop) = on_loop {
                    next_index = on_loop(event, prev_index, next_index);
                }
            }

            if is_different_grid_row(next_index, cols, prev_row) {
                next_index = prev_index;
            }
        }

        let last_row = max_index.div_euclid(cols) == prev_row;

        if is_index_out_of_list_bounds(list, next_index) {
            if loop_focus && last_row {
                next_index = if key == backward_key {
                    max_index
                } else {
                    find_non_disabled_list_index(
                        list,
                        FindNonDisabledListIndexOptions {
                            starting_index: prev_index - (prev_index % cols) - 1,
                            disabled_indices,
                            ..FindNonDisabledListIndexOptions::new(None)
                        },
                    )
                };
                if let Some(on_loop) = on_loop {
                    next_index = on_loop(event, prev_index, next_index);
                }
            } else {
                next_index = prev_index;
            }
        }
    }

    next_index
}

/// The item sizes `createGridCellMap` packs — upstream `Dimensions`
/// (`composite.ts:386`, the `types.ts` re-export of `@floating-ui/utils`' shape).
pub use crate::floating_ui::types::Dimensions as GridDimensions;

/// Port of `createGridCellMap` (`composite.ts:385-426`): for each cell index, the
/// item index that occupies that cell — `None` cells are unoccupied. `width`/`height`
/// arrive as `f64` (the upstream `Dimensions` shape) and are used as integral loop
/// bounds, the way every real consumer sizes items.
pub fn create_grid_cell_map(
    sizes: &[GridDimensions],
    cols: i32,
    dense: bool,
) -> Vec<Option<usize>> {
    let mut cell_map: Vec<Option<usize>> = Vec::new();
    let mut start_index: i32 = 0;
    for (index, size) in sizes.iter().enumerate() {
        let width = size.width as i32;
        let height = size.height as i32;
        if width > cols {
            // `createGridCellMap`'s dev-only throw (`composite.ts:391-398`); see the
            // module docs for the panic translation.
            panic!(
                "[Floating UI]: Invalid grid - item width at index {index} is greater than grid columns"
            );
        }
        let mut item_placed = false;
        if dense {
            start_index = 0;
        }
        while !item_placed {
            let mut target_cells: Vec<i32> = Vec::new();
            for i in 0..width {
                for j in 0..height {
                    target_cells.push(start_index + i + j * cols);
                }
            }
            let fits_row = (start_index % cols) + width <= cols;
            let all_free = target_cells
                .iter()
                .all(|&cell| cell_map.get(cell as usize).cloned().flatten().is_none());
            if fits_row && all_free {
                for &cell in &target_cells {
                    let cell = cell as usize;
                    if cell_map.len() <= cell {
                        cell_map.resize(cell + 1, None);
                    }
                    cell_map[cell] = Some(index);
                }
                item_placed = true;
            } else {
                start_index += 1;
            }
        }
    }

    // `[...cellMap]` (`composite.ts:425`) — dense, in cell order.
    cell_map
}

/// The `corner` argument of `getGridCellIndexOfCorner` (`composite.ts:434`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridCorner {
    /// `'tl'`.
    TopLeft,
    /// `'tr'`.
    TopRight,
    /// `'bl'`.
    BottomLeft,
    /// `'br'`.
    BottomRight,
}

/// Port of `getGridCellIndexOfCorner` (`composite.ts:429-461`): the cell index of the
/// item's chosen corner, or `-1` when the item index is the `-1` sentinel.
pub fn get_grid_cell_index_of_corner(
    index: i32,
    sizes: &[GridDimensions],
    cell_map: &[Option<usize>],
    cols: i32,
    corner: GridCorner,
) -> i32 {
    if index == -1 {
        return -1;
    }

    let first_cell_index = cell_map
        .iter()
        .position(|&cell| cell == Some(index as usize))
        .map_or(-1, |position| position as i32);
    let size_item = sizes.get(index as usize);

    match corner {
        GridCorner::TopLeft => first_cell_index,
        GridCorner::TopRight => match size_item {
            None => first_cell_index,
            Some(size_item) => first_cell_index + size_item.width as i32 - 1,
        },
        GridCorner::BottomLeft => match size_item {
            None => first_cell_index,
            Some(size_item) => first_cell_index + (size_item.height as i32 - 1) * cols,
        },
        GridCorner::BottomRight => cell_map
            .iter()
            .rposition(|&cell| cell == Some(index as usize))
            .map_or(-1, |position| position as i32),
    }
}

/// Port of `getGridCellIndices` (`composite.ts:463-469`): all cell indices that
/// correspond to the specified item indices.
pub fn get_grid_cell_indices(indices: &[Option<usize>], cell_map: &[Option<usize>]) -> Vec<usize> {
    cell_map
        .iter()
        .enumerate()
        .filter(|(_cell_index, cell)| indices.contains(cell))
        .map(|(cell_index, _)| cell_index)
        .collect()
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    fn none_list(len: usize) -> Vec<Option<Element>> {
        vec![None; len]
    }

    // Pins the explicit-disabled arms (`composite.ts:474-478`): the predicate arm is
    // consulted per index and the array arm is a membership check; an explicit hit
    // short-circuits before any element is read (so a null entry still reports
    // disabled).
    #[test]
    fn explicit_disabled_sets_decide_before_the_element_is_consulted() {
        let predicate = DisabledIndices::Predicate(Rc::new(|index: i32| index % 2 == 0));
        assert!(is_list_index_disabled(&[], 0, Some(&predicate)));
        assert!(!is_list_index_disabled(&[], 1, Some(&predicate)));

        let list = DisabledIndices::List(vec![2, 5]);
        assert!(is_list_index_disabled(&[], 2, Some(&list)));
        assert!(!is_list_index_disabled(&[], 3, Some(&list)));
    }

    // Pins the element-less arm (`composite.ts:480-482`): a missing entry (negative,
    // past-the-end, or a `null` slot) is not disabled by itself — only the explicit
    // set can disable it.
    #[test]
    fn a_missing_element_entry_is_not_disabled() {
        let list: Vec<Option<Element>> = vec![None];
        assert!(!is_list_index_disabled(&list, 0, None));
        assert!(!is_list_index_disabled(&list, -1, None));
        assert!(!is_list_index_disabled(&list, 1, None));
        assert!(is_list_index_disabled(
            &list,
            0,
            Some(&DisabledIndices::List(vec![0]))
        ));
    }

    // Pins the no-explicit-set attribute fallback (`composite.ts:495-497`): without
    // `disabledIndices` the function reports disabled for entries the element list
    // marks — untestable without DOM (the attribute checks need an element), so the
    // element-carrying arms are pinned in the wasm suite.
    #[test]
    fn no_explicit_set_leaves_the_decision_to_the_element_arms() {
        assert!(!is_list_index_disabled(&[], 0, None));
    }

    // Pins the step walk (`composite.ts:36-60`): the do-while advances by `amount`
    // (downward when `decrement`) while the landed entry is disabled, and the
    // out-of-list exit value is the returned sentinel.
    #[test]
    fn find_non_disabled_list_index_walks_and_stops_at_the_boundary() {
        // Every entry enabled: starting_index + amount.
        let list = none_list(5);
        assert_eq!(
            find_non_disabled_list_index(
                &list,
                FindNonDisabledListIndexOptions {
                    starting_index: 1,
                    ..FindNonDisabledListIndexOptions::new(None)
                }
            ),
            2
        );

        // Disabled entries are stepped over — an explicit set on 2,3.
        let disabled = DisabledIndices::List(vec![2, 3]);
        assert_eq!(
            find_non_disabled_list_index(
                &list,
                FindNonDisabledListIndexOptions {
                    starting_index: 1,
                    disabled_indices: Some(&disabled),
                    ..FindNonDisabledListIndexOptions::new(None)
                }
            ),
            4
        );

        // Decrement walks downward.
        assert_eq!(
            find_non_disabled_list_index(
                &list,
                FindNonDisabledListIndexOptions {
                    starting_index: 3,
                    decrement: true,
                    disabled_indices: Some(&disabled),
                    ..FindNonDisabledListIndexOptions::new(None)
                }
            ),
            1
        );

        // `amount` strides a full row (the vertical column walk).
        assert_eq!(
            find_non_disabled_list_index(
                &list,
                FindNonDisabledListIndexOptions {
                    starting_index: 0,
                    amount: 5,
                    ..FindNonDisabledListIndexOptions::new(None)
                }
            ),
            5,
            "stepping past the end returns the out-of-bounds sentinel"
        );

        // No non-disabled entry below: the walk exits at -1.
        let all_disabled = DisabledIndices::List(vec![0, 1]);
        assert_eq!(
            find_non_disabled_list_index(
                &list,
                FindNonDisabledListIndexOptions {
                    starting_index: 1,
                    decrement: true,
                    disabled_indices: Some(&all_disabled),
                    ..FindNonDisabledListIndexOptions::new(None)
                }
            ),
            -1
        );
    }

    // Pins the min/max helpers (`composite.ts:18-34`): the minimum from the list
    // head, the maximum searched downward from `list.length` (upstream's
    // `startingIndex: listRef.current.length`).
    #[test]
    fn get_min_and_max_list_index_resolve_the_enabled_ends() {
        let list = none_list(5);
        assert_eq!(get_min_list_index(&list, None), 0);
        assert_eq!(get_max_list_index(&list, None), 4);

        let disabled = DisabledIndices::List(vec![0, 4]);
        assert_eq!(get_min_list_index(&list, Some(&disabled)), 1);
        assert_eq!(get_max_list_index(&list, Some(&disabled)), 3);

        let empty: Vec<Option<Element>> = Vec::new();
        assert_eq!(get_min_list_index(&empty, None), 0);
        assert_eq!(get_max_list_index(&empty, None), -1);
    }

    // Pins the row-difference predicate (`composite.ts:10-12`) — floor division, so
    // the -1 sentinel lands on the virtual row above the first.
    #[test]
    fn grid_row_difference_uses_floor_division() {
        assert!(!is_different_grid_row(0, 5, 0));
        assert!(is_different_grid_row(5, 5, 0));
        assert!(!is_different_grid_row(9, 5, 1));
        assert!(is_different_grid_row(10, 5, 1));
        // `Math.floor(-1 / 5)` — the -1 sentinel sits on row -1.
        assert!(!is_different_grid_row(-1, 5, -1));
        assert!(is_different_grid_row(-1, 5, 0));
    }

    // Pins the bounds predicate (`composite.ts:14-16`).
    #[test]
    fn out_of_bounds_detection_matches_the_sentinels() {
        let list = none_list(3);
        assert!(is_index_out_of_list_bounds(&list, -1));
        assert!(!is_index_out_of_list_bounds(&list, 0));
        assert!(!is_index_out_of_list_bounds(&list, 2));
        assert!(is_index_out_of_list_bounds(&list, 3));
    }

    // Pins the vertical column walk of `getGridNavigatedIndex`
    // (`composite.ts:91-283`) on element-less lists (every entry enabled through the
    // element arms): from -1 the to-end keys land on min/max; otherwise movement is
    // a `cols`-wide stride clamped by the disabled walk.
    #[test]
    fn vertical_keys_walk_columns_and_fall_back_to_the_ends_from_the_sentinel() {
        let list = none_list(10);
        let options = |key: &'static str, prev_index: i32| GridNavigatedIndexOptions {
            key,
            event: None,
            orientation: Orientation::Vertical,
            loop_focus: false,
            on_loop: None,
            rtl: false,
            cols: 5,
            disabled_indices: None,
            min_index: 0,
            max_index: 9,
            prev_index,
            stop: false,
        };

        // No previous index: ArrowDown starts at the min, ArrowUp at the max.
        assert_eq!(get_grid_navigated_index(&list, options("ArrowDown", -1)), 0);
        assert_eq!(get_grid_navigated_index(&list, options("ArrowUp", -1)), 9);

        // The plain column walk (no DOM rows to infer).
        assert_eq!(get_grid_navigated_index(&list, options("ArrowDown", 0)), 5);
        assert_eq!(get_grid_navigated_index(&list, options("ArrowUp", 9)), 4);

        // Non-arrow keys return the previous index untouched.
        assert_eq!(get_grid_navigated_index(&list, options("Home", 3)), 3);
    }

    // Pins the vertical loop arithmetic (`composite.ts:251-277`): wrapping up past
    // the min lands on the last row's same column (the maxCol/col offset math), and
    // wrapping down past the max restarts from the same column on the first rows.
    #[test]
    fn vertical_loop_wraps_by_column() {
        let list = none_list(10);
        let options = |key: &'static str, prev_index: i32| GridNavigatedIndexOptions {
            key,
            event: None,
            orientation: Orientation::Vertical,
            loop_focus: true,
            on_loop: None,
            rtl: false,
            cols: 5,
            disabled_indices: None,
            min_index: 0,
            max_index: 9,
            prev_index,
            stop: false,
        };

        // Up from index 1 (row 0, col 1): the last row's col 1 is index 6.
        assert_eq!(get_grid_navigated_index(&list, options("ArrowUp", 1)), 6);

        // Down from index 6 (row 1, col 1): past the max, restart at col 1.
        assert_eq!(get_grid_navigated_index(&list, options("ArrowDown", 6)), 1);
    }

    // Pins the horizontal arms (`composite.ts:286-380`) — 'both' orientation keeps
    // the movement on the current grid row: forward/backward walks clamp at the row
    // boundary, loop restarts within the row, and the last row wraps back to the
    // maxIndex sentinel.
    #[test]
    fn horizontal_keys_stay_on_the_row() {
        let list = none_list(10);
        let options =
            |key: &'static str, prev_index: i32, loop_focus: bool| GridNavigatedIndexOptions {
                key,
                event: None,
                orientation: Orientation::Both,
                loop_focus,
                on_loop: None,
                rtl: false,
                cols: 5,
                disabled_indices: None,
                min_index: 0,
                max_index: 9,
                prev_index,
                stop: false,
            };

        // Same-row movement.
        assert_eq!(
            get_grid_navigated_index(&list, options("ArrowRight", 0, false)),
            1
        );
        assert_eq!(
            get_grid_navigated_index(&list, options("ArrowLeft", 6, false)),
            5
        );

        // Row boundary without loop: stay (the walk would leave the row).
        assert_eq!(
            get_grid_navigated_index(&list, options("ArrowRight", 4, false)),
            4
        );
        assert_eq!(
            get_grid_navigated_index(&list, options("ArrowLeft", 5, false)),
            5
        );

        // Row boundary with loop: wrap within the row.
        assert_eq!(
            get_grid_navigated_index(&list, options("ArrowRight", 4, true)),
            0
        );
        assert_eq!(
            get_grid_navigated_index(&list, options("ArrowLeft", 5, true)),
            9
        );

        // Walking off the list on the last row without loop: stay.
        assert_eq!(
            get_grid_navigated_index(&list, options("ArrowRight", 9, false)),
            9
        );
        // With loop on the last row: wrap to the row start.
        assert_eq!(
            get_grid_navigated_index(&list, options("ArrowRight", 9, true)),
            5
        );
    }

    // Pins the rtl swap (`composite.ts:289,324`): forward is ArrowLeft and backward
    // ArrowRight in RTL layouts, and the row-difference clamp still applies through
    // the mapping — a disabled block that would push the walk onto the next row
    // cancels the move.
    #[test]
    fn rtl_swaps_the_horizontal_directions() {
        let list = none_list(10);
        let options = |key: &'static str, rtl: bool| GridNavigatedIndexOptions {
            key,
            event: None,
            orientation: Orientation::Both,
            loop_focus: false,
            on_loop: None,
            rtl,
            cols: 5,
            disabled_indices: None,
            min_index: 0,
            max_index: 9,
            prev_index: 0,
            stop: false,
        };

        assert_eq!(
            get_grid_navigated_index(&list, options("ArrowLeft", true)),
            1
        );
        assert_eq!(
            get_grid_navigated_index(&list, options("ArrowRight", true)),
            0
        );
        assert_eq!(
            get_grid_navigated_index(
                &list,
                GridNavigatedIndexOptions {
                    key: "ArrowLeft",
                    prev_index: 3,
                    disabled_indices: Some(&DisabledIndices::List(vec![4])),
                    rtl: true,
                    ..options("ArrowLeft", true)
                }
            ),
            3,
            "the disabled 4 pushes the walk onto row 1, so the clamp cancels the move"
        );
    }

    // Pins the disabled-skipping on the horizontal walk
    // (`composite.ts:295-298,330-334`): the next/previous search is the disabled
    // walk, so an explicitly disabled neighbor is stepped over.
    #[test]
    fn horizontal_walk_skips_disabled_entries() {
        let list = none_list(10);
        let disabled = DisabledIndices::List(vec![1, 2]);
        assert_eq!(
            get_grid_navigated_index(
                &list,
                GridNavigatedIndexOptions {
                    key: "ArrowRight",
                    event: None,
                    orientation: Orientation::Both,
                    loop_focus: false,
                    on_loop: None,
                    rtl: false,
                    cols: 5,
                    disabled_indices: Some(&disabled),
                    min_index: 0,
                    max_index: 9,
                    prev_index: 0,
                    stop: false,
                }
            ),
            3,
            "1 and 2 are disabled, so right from 0 lands on 3"
        );
    }

    // Pins the cell-map packing (`composite.ts:385-426`): 1x1 items pack
    // sequentially (the identity transform the injected grid navigator relies on),
    // multi-cell items occupy their rectangle, and dense packing backfills holes a
    // non-dense pass would leave behind.
    #[test]
    fn create_grid_cell_map_places_items() {
        use crate::floating_ui::types::Dimensions;

        fn size(width: f64, height: f64) -> Dimensions {
            Dimensions { width, height }
        }

        // 1x1 identity over 6 cells, 3 cols.
        let unit: Vec<Dimensions> = (0..6).map(|_| size(1.0, 1.0)).collect();
        let cell_map = create_grid_cell_map(&unit, 3, false);
        assert_eq!(cell_map, (0..6).map(Some).collect::<Vec<_>>());

        // A 2x2 item after two 1x1 items (3 cols): it cannot fit in the remaining
        // row-0 cell, so it starts on row 1 and occupies cells 3, 4, 6, 7.
        let sizes = vec![size(1.0, 1.0), size(1.0, 1.0), size(2.0, 2.0)];
        let cell_map = create_grid_cell_map(&sizes, 3, false);
        assert_eq!(cell_map[0], Some(0));
        assert_eq!(cell_map[1], Some(1));
        assert_eq!(cell_map[2], None, "the lone row-0 cell stays unoccupied");
        assert_eq!(cell_map[3], Some(2));
        assert_eq!(cell_map[4], Some(2));
        assert_eq!(cell_map[5], None);
        assert_eq!(cell_map[6], Some(2));
        assert_eq!(cell_map[7], Some(2));

        // Two 2-wide items then a 1-wide one (3 cols): the second 2-wide cannot sit
        // on cell 2 (it would not fit the row) and jumps to row 1; non-dense leaves
        // the hole, dense backfills it with the trailing 1x1.
        let sizes = vec![size(2.0, 1.0), size(2.0, 1.0), size(1.0, 1.0)];
        let sparse = create_grid_cell_map(&sizes, 3, false);
        assert_eq!(sparse[2], None);
        assert_eq!(sparse[5], Some(2));
        let dense = create_grid_cell_map(&sizes, 3, true);
        assert_eq!(dense[2], Some(2), "the dense pass backfills the hole");

        // Oversized width panics (the dev-only throw; see the module docs).
        let result = std::panic::catch_unwind(|| {
            create_grid_cell_map(&[size(4.0, 1.0)], 3, false);
        });
        assert!(result.is_err(), "width > cols must fail fast, not hang");
    }

    // Pins the corner resolution (`composite.ts:429-461`): tl is the first cell,
    // tr/bl extend by the item size, br is the last occupied cell, and the -1
    // sentinel short-circuits.
    #[test]
    fn get_grid_cell_index_of_corner_resolves_the_corners() {
        use crate::floating_ui::types::Dimensions;

        let sizes = vec![Dimensions {
            width: 2.0,
            height: 2.0,
        }];
        let cell_map: Vec<Option<usize>> = vec![Some(0), Some(0), Some(0), Some(0)];

        assert_eq!(
            get_grid_cell_index_of_corner(0, &sizes, &cell_map, 2, GridCorner::TopLeft),
            0
        );
        assert_eq!(
            get_grid_cell_index_of_corner(0, &sizes, &cell_map, 2, GridCorner::TopRight),
            1
        );
        assert_eq!(
            get_grid_cell_index_of_corner(0, &sizes, &cell_map, 2, GridCorner::BottomLeft),
            2
        );
        assert_eq!(
            get_grid_cell_index_of_corner(0, &sizes, &cell_map, 2, GridCorner::BottomRight),
            3
        );
        assert_eq!(
            get_grid_cell_index_of_corner(-1, &sizes, &cell_map, 2, GridCorner::TopLeft),
            -1,
            "the -1 sentinel short-circuits"
        );
    }

    // Pins the cell lookup (`composite.ts:463-469`): every cell whose item is in
    // the requested set, in cell order.
    #[test]
    fn get_grid_cell_indices_collects_the_cells() {
        let cell_map: Vec<Option<usize>> = vec![Some(0), None, Some(1), Some(0)];
        assert_eq!(get_grid_cell_indices(&[Some(0)], &cell_map), vec![0, 3]);
        assert_eq!(get_grid_cell_indices(&[Some(1)], &cell_map), vec![2]);
        assert_eq!(get_grid_cell_indices(&[None], &cell_map), vec![1]);
        assert_eq!(
            get_grid_cell_indices(&[Some(2)], &cell_map),
            Vec::<usize>::new()
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// A connected element — visibility checks require `isConnected`
    /// (`composite.ts:515`), so every fixture is appended to the document body.
    fn attached_element(tag: &str) -> web_sys::HtmlElement {
        let document = web_sys::window().unwrap().document().unwrap();
        let element: web_sys::HtmlElement =
            document.create_element(tag).unwrap().dyn_into().unwrap();
        document.body().unwrap().append_child(&element).unwrap();
        element
    }

    // Pins the visibility arms through real computed styles
    // (`composite.ts:484-486` + `isElementVisible`): a connected element hidden with
    // `display: none` reports disabled; a visible one does not.
    #[wasm_bindgen_test]
    fn a_display_none_element_is_disabled_and_a_visible_one_is_not() {
        let visible = attached_element("button");
        let hidden = attached_element("button");
        hidden.style().set_property("display", "none").unwrap();
        let list = vec![Some(visible.clone().into()), Some(hidden.clone().into())];

        assert!(is_element_visible(Some(visible.as_ref())));
        assert!(!is_element_visible(Some(hidden.as_ref())));
        assert!(!is_list_index_disabled(&list, 0, None));
        assert!(is_list_index_disabled(&list, 1, None));
    }

    // Pins the visibility:hidden arm (`composite.ts:507-509`): `visibility: hidden`
    // hides through `isHiddenByStyles`, the same check the navigation hooks ride.
    #[wasm_bindgen_test]
    fn a_visibility_hidden_element_is_disabled() {
        let hidden = attached_element("button");
        hidden.style().set_property("visibility", "hidden").unwrap();
        assert!(!is_element_visible(Some(hidden.as_ref())));
        assert!(is_hidden_by_styles(
            &web_sys::window()
                .unwrap()
                .get_computed_style(hidden.as_ref())
                .unwrap()
                .unwrap()
        ));
    }

    // Pins the native-disabled arm (`composite.ts:488-493`): a `:disabled` match
    // always reports disabled, even when an explicit set marks the index enabled.
    #[wasm_bindgen_test]
    fn a_natively_disabled_element_is_disabled_even_when_explicitly_enabled() {
        let disabled = attached_element("button");
        disabled.set_attribute("disabled", "").unwrap();
        let list = vec![Some(disabled.clone().into())];

        assert!(
            is_list_index_disabled(&list, 0, None),
            "the no-explicit-set path reports the :disabled match"
        );
        assert!(
            is_list_index_disabled(&list, 0, Some(&DisabledIndices::List(vec![]))),
            "an explicit empty set still skips the natively disabled element"
        );
    }

    // Pins the `aria-disabled` fallback (`composite.ts:495-497`): without an explicit
    // set, `aria-disabled="true"` reports disabled — but only then, since
    // aria-disabled items are focusable-while-disabled.
    #[wasm_bindgen_test]
    fn an_aria_disabled_element_is_disabled_only_without_an_explicit_set() {
        let aria_disabled = attached_element("button");
        aria_disabled
            .set_attribute("aria-disabled", "true")
            .unwrap();
        let list = vec![Some(aria_disabled.clone().into())];

        assert!(is_list_index_disabled(&list, 0, None));
        assert!(
            !is_list_index_disabled(&list, 0, Some(&DisabledIndices::List(vec![]))),
            "the explicit set owns the decision when provided"
        );
    }

    // Pins the feature-detect fallback ordering (`composite.ts:520-526`): with
    // `checkVisibility` available (real Chrome), visibility flows through it — and a
    // detached element is never visible regardless of styling
    // (`!element.isConnected`, `composite.ts:515`).
    #[wasm_bindgen_test]
    fn a_detached_element_is_not_visible_even_when_styled() {
        let document = web_sys::window().unwrap().document().unwrap();
        let detached: web_sys::HtmlElement = document
            .create_element("button")
            .unwrap()
            .dyn_into()
            .unwrap();
        assert!(!detached.is_connected());
        assert!(!is_element_visible(Some(detached.as_ref())));
    }
}
