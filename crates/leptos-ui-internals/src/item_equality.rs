//! Port of `packages/react/src/internals/itemEquality.ts:1-134` — the selection-index
//! semantics shared by the Select-family components (`TODO.md`, item `infra: internals`).
//!
//! Upstream is a pure utility module: the `Object.is`-semantics default comparer
//! ([`default_item_equality`], `itemEquality.ts:8-10`), the null-safe comparison wrapper
//! ([`compare_item_equality`], `:12-21`), the array-aware dirty check ([`is_selected_value_dirty`]
//! / [`is_selected_values_dirty`], `:23-35`), the membership/lookup helpers
//! ([`selected_value_includes`], `:37-51`; [`find_item_index`], `:53-67`), the selection
//! anchor ([`find_selection_index`], `:86-101`), the per-item registration hook
//! ([`resolve_selected_index`], `:104-124`), and the removal filter
//! ([`remove_item`], `:126-134`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - Upstream's two type parameters (`Item`, `Value`) are one runtime type in JS (`any`);
//!   the port collapses them onto a single [`ObjectIs`]-generic `T`, which carries the
//!   SameValue semantics the module is built on (`NaN` matches `NaN`, `+0`/`-0` differ —
//!   the `f64` impl, `crates/leptos-ui-utils/src/are_arrays_equal.rs:66-76`). The upstream
//!   tests instantiate the port with `f64`/`String`/`Value` accordingly.
//! - JS `undefined` items and array holes (sparse slots) both port to `None` in the
//!   `Option`-wrapped slices; a JS `null` is a *value*, not an absence, so it stays inside
//!   `T` (the dynamic-value instantiation carries it as `Value::Null`). This preserves the
//!   load-bearing distinction upstream relies on: `undefined` items are skipped
//!   (`itemEquality.ts:62-65`, `:46-48`) while `null` items match `null` selections through
//!   the comparison (`:17-20`).
//! - The nullish bypass in [`compare_item_equality`] covers `None` only: upstream
//!   `itemValue == null` (loose, `:17-20`) also short-circuits a JS `null` *value* pair to
//!   `Object.is`; in the port a `T`-encoded null reaches the comparer, which for
//!   [`default_item_equality`] is the same SameValue comparison. The custom-comparer tests
//!   never mix `null` items with a custom comparer, so the observable contract holds.
//! - The `Set`-indexed fast path for the default comparer
//!   (`itemEquality.ts:71-84`) does not port: JS hashes untyped values with SameValueZero
//!   semantics, which a generic `T` without a `Hash` bound (and across the item/selection
//!   pair) cannot replicate, and the `+0`/`-0` re-check exists precisely because `Set`
//!   collapses the zeros the comparer must distinguish. Both paths therefore run the linear
//!   scan; every return value is identical (the scan *is* the `Object.is` semantics), and
//!   only the default path's O(n + m) asymptotics degrade to O(n · m) — a performance
//!   property whose upstream pin instruments JS property reads through a `Proxy`
//!   (`itemEquality.test.ts:40-59`), a mechanism with no Rust analog. The anchoring contract
//!   that test shares with the others — first selected item in *rendered* order,
//!   independent of selection order — is pinned by the order-swap tests below.
//! - Upstream's `multiple && Array.isArray(selectedValue)` runtime branch
//!   (`:94-99`) ports to the [`SelectedValues`] enum: the caller states statically whether
//!   the selection is a list. Passing an array *value* as a single selection — the
//!   "array can itself be a valid single-select value" case (`:92-93`) — is
//!   [`SelectedValues::Single`] wrapping that array-valued `T`.
//! - `-1` sentinels (`itemEquality.ts:60`, `:100`) port to `Option<usize>`: `None` is the
//!   "no index" result (`null` in `findSelectionIndex`'s return).
//! - [`is_selected_value_dirty`]'s runtime array branch (`:28-31`) ports to the separate
//!   [`is_selected_values_dirty`]: the caller knows statically whether its value is a list.
//!   The scalar branch keeps JS strict-`!==` semantics (`:34`) — `NaN` is always dirty,
//!   which Rust's `PartialEq` reproduces — while the list branch wraps the caller's comparer
//!   in the null-safe comparison, exactly as `:29-31` does.
//! - `RequestQueue.pickEntries`-style extension points are N/A here; nothing in this module
//!   is overridden upstream.

use leptos_ui_utils::are_arrays_equal::{ObjectIs, are_arrays_equal_by};
use serde_json::Value;

/// The upstream `defaultItemEquality` (`packages/react/src/internals/itemEquality.ts:8-10`):
/// SameValue comparison (`Object.is`), carried by the [`ObjectIs`] port.
pub fn default_item_equality<T: ObjectIs>(item_value: &T, selected_value: &T) -> bool {
    item_value.object_is(selected_value)
}

/// The upstream `compareItemEquality` (`packages/react/src/internals/itemEquality.ts:12-21`):
/// when either side is `undefined` (the `None` arm), the pair compares by SameValue — so two
/// absences match and an absence never matches a present value; otherwise the caller's
/// comparer decides.
pub fn compare_item_equality<T>(
    item_value: Option<&T>,
    selected_value: Option<&T>,
    comparer: impl Fn(&T, &T) -> bool,
) -> bool {
    match (item_value, selected_value) {
        (Some(item_value), Some(selected_value)) => comparer(item_value, selected_value),
        (None, None) => true,
        _ => false,
    }
}

/// The upstream scalar `isSelectedValueDirty` branch
/// (`packages/react/src/internals/itemEquality.ts:34`): JS strict `!==`, so `NaN` is always
/// dirty and equal values are not. The list-aware branch lives in
/// [`is_selected_values_dirty`].
pub fn is_selected_value_dirty<T: PartialEq>(
    current_value: Option<&T>,
    initial_value: Option<&T>,
) -> bool {
    match (current_value, initial_value) {
        (Some(current), Some(initial)) => current != initial,
        (None, None) => false,
        _ => true,
    }
}

/// The upstream list branch of `isSelectedValueDirty`
/// (`packages/react/src/internals/itemEquality.ts:28-32`): the arrays must be equal under
/// the null-safe comparison for the value to count as unchanged.
pub fn is_selected_values_dirty<T>(
    current_values: &[T],
    initial_values: &[T],
    comparer: impl Fn(&T, &T) -> bool,
) -> bool {
    !are_arrays_equal_by(current_values, initial_values, |current, initial| {
        compare_item_equality(Some(current), Some(initial), &comparer)
    })
}

/// The upstream `selectedValueIncludes` (`packages/react/src/internals/itemEquality.ts:37-51`):
/// whether the selection contains the item — a missing selection is `false`, `undefined`
/// slots never match, and the comparison is the null-safe one.
pub fn selected_value_includes<T>(
    selected_values: Option<&[Option<T>]>,
    item_value: Option<&T>,
    comparer: impl Fn(&T, &T) -> bool,
) -> bool {
    let Some(selected_values) = selected_values else {
        return false;
    };
    selected_values
        .iter()
        .any(|selected_value| compare_item_equality(item_value, selected_value.as_ref(), &comparer))
}

/// The upstream `findItemIndex` (`packages/react/src/internals/itemEquality.ts:53-67`): the
/// first item matching the selected value, or `None` (upstream `-1`). A missing list is
/// `None` without comparing (`:59-61`), and `undefined` items are skipped (`:62-65`).
pub fn find_item_index<T>(
    item_values: Option<&[Option<T>]>,
    selected_value: Option<&T>,
    comparer: impl Fn(&T, &T) -> bool,
) -> Option<usize> {
    let item_values = item_values?;
    item_values.iter().position(|item_value| {
        compare_item_equality(item_value.as_ref(), selected_value, &comparer)
    })
}

/// The statically-typed shape of upstream's `Value | readonly Value[] | null | undefined`
/// selection parameter (`packages/react/src/internals/itemEquality.ts:88`): see the module
/// docs for the runtime-branch-to-enum mapping.
pub enum SelectedValues<'a, T> {
    /// A single value (`undefined` included as the outer `None`).
    Single(Option<&'a T>),
    /// The multiple-selection list; `None` slots are `undefined` entries.
    Multiple(&'a [Option<T>]),
}

/// The upstream `findSelectionIndex` (`packages/react/src/internals/itemEquality.ts:86-101`):
/// the first rendered index anchored to the selection, or `None` when nothing is selected —
/// anchored to the *first selected item in rendered order* in multiple mode, so the index
/// does not depend on the order values were added to the selection.
pub fn find_selection_index<T: ObjectIs>(
    item_values: &[Option<T>],
    selected_value: SelectedValues<T>,
    comparer: impl Fn(&T, &T) -> bool,
) -> Option<usize> {
    let index = match selected_value {
        SelectedValues::Multiple(selected_values) => {
            item_values.iter().position(|item_value| match item_value {
                Some(item_value) => selected_values.iter().any(|selected_value| {
                    compare_item_equality(Some(item_value), selected_value.as_ref(), &comparer)
                }),
                None => false,
            })
        }
        SelectedValues::Single(selected_value) => {
            find_item_index(Some(item_values), selected_value, comparer)
        }
    };
    index
}

/// The upstream `resolveSelectedIndex` (`packages/react/src/internals/itemEquality.ts:104-124`):
/// the per-item hook that elects the single selection anchor. A selected item claims the
/// index unless an *earlier* selected holder still holds it; the holder re-elects a new
/// anchor (the first selected item in rendered order) once the index it holds stops being
/// selected.
pub fn resolve_selected_index<T: ObjectIs>(
    index: usize,
    item_value: Option<&T>,
    registry: &[Option<T>],
    selected_values: &[Option<T>],
    comparer: impl Fn(&T, &T) -> bool,
    current_index: Option<usize>,
) -> Option<usize> {
    if selected_value_includes(Some(selected_values), item_value, &comparer) {
        // A later item only takes over once the current anchor stops being selected
        // (`itemEquality.ts:113-118`).
        return match current_index {
            Some(current) if index > current => {
                let current_holds = registry
                    .get(current)
                    .and_then(|slot| slot.as_ref())
                    .filter(|current_value| {
                        selected_value_includes(
                            Some(selected_values),
                            Some(current_value),
                            &comparer,
                        )
                    })
                    .is_some();
                if current_holds {
                    Some(current)
                } else {
                    Some(index)
                }
            }
            _ => Some(index),
        };
    }
    // The holder re-elects the anchor once it stops being selected
    // (`itemEquality.ts:120-123`).
    match current_index {
        Some(current) if current == index => find_selection_index(
            registry,
            SelectedValues::Multiple(selected_values),
            comparer,
        ),
        other => other,
    }
}

/// The upstream `removeItem` (`packages/react/src/internals/itemEquality.ts:126-134`): the
/// selection with the item filtered out, under the null-safe comparison.
pub fn remove_item<T>(
    selected_values: &[Option<T>],
    item_value: Option<&T>,
    comparer: impl Fn(&T, &T) -> bool,
) -> Vec<Option<T>>
where
    T: Clone,
{
    selected_values
        .iter()
        .filter(|selected_value| {
            !compare_item_equality(item_value, selected_value.as_ref(), &comparer)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The dynamic-value instantiation for tests mixing strings and JS `null` — a newtype
    /// over `serde_json::Value` with `PartialEq`-based SameValue semantics (JS objects
    /// compare by reference; the JSON representation's structural equality is the
    /// port's stand-in for the literal-comparison tests, which only use primitives).
    #[derive(Clone, Debug)]
    struct Json(Value);

    impl ObjectIs for Json {
        fn object_is(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    fn json_items(values: &[Value]) -> Vec<Option<Json>> {
        values
            .iter()
            .map(|value| Some(Json(value.clone())))
            .collect()
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:7-10`.
    #[test]
    fn anchors_to_the_first_selected_item_in_rendered_order_not_value_order() {
        let items = ["a", "b", "c"];
        let items = items.map(Some);

        assert_eq!(
            find_selection_index(
                &items,
                SelectedValues::Multiple(&vec![Some("c"), Some("b")]),
                default_item_equality,
            ),
            Some(1)
        );
        assert_eq!(
            find_selection_index(
                &items,
                SelectedValues::Multiple(&vec![Some("b"), Some("c")]),
                default_item_equality,
            ),
            Some(1)
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:12-15`.
    #[test]
    fn returns_none_when_nothing_in_the_value_array_is_rendered() {
        let items = ["a", "b", "c"];
        let items = items.map(Some);

        assert_eq!(
            find_selection_index(
                &items,
                SelectedValues::Multiple(&vec![]),
                default_item_equality,
            ),
            None
        );
        assert_eq!(
            find_selection_index(
                &items,
                SelectedValues::Multiple(&vec![Some("d")]),
                default_item_equality,
            ),
            None
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:17-30`: an array value
    // outside multiple mode is one single value — instantiated with the dynamic `Json`
    // representation, as the JS test's mixed `string | string[]` items require.
    #[test]
    fn treats_an_array_as_a_single_value_outside_multiple_mode() {
        let array_value = json!(["x", "y"]);
        let comparer = |item_value: &Json, selected_value: &Json| {
            item_value.0.to_string() == selected_value.0.to_string()
        };

        let mut items = json_items(&[json!("a")]);
        items.push(Some(Json(array_value.clone())));

        assert_eq!(
            find_selection_index(
                &items,
                SelectedValues::Single(Some(&Json(array_value))),
                comparer,
            ),
            Some(1)
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:32-38`.
    #[test]
    fn anchors_to_the_first_selected_item_with_a_custom_comparer() {
        let items = ["a", "b", "c"];
        let items = items.map(Some);
        let comparer = |item_value: &&str, selected_value: &&str| {
            item_value.to_lowercase() == selected_value.to_lowercase()
        };

        assert_eq!(
            find_selection_index(
                &items,
                SelectedValues::Multiple(&vec![Some("C"), Some("B")]),
                comparer,
            ),
            Some(1)
        );
        assert_eq!(
            find_selection_index(&items, SelectedValues::Multiple(&vec![Some("D")]), comparer,),
            None
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:61-68` (the `Proxy`
    // read-counting companion at `:40-59` instruments JS property reads — see the module
    // docs; the anchoring contract it shares is pinned by the order-swap test above).
    #[test]
    fn keeps_positive_and_negative_zero_distinct_like_object_is() {
        assert_eq!(
            find_selection_index(
                &[Some(-0.0_f64)],
                SelectedValues::Multiple(&vec![Some(0.0)]),
                default_item_equality,
            ),
            None
        );
        assert_eq!(
            find_selection_index(
                &[Some(0.0_f64)],
                SelectedValues::Multiple(&vec![Some(-0.0)]),
                default_item_equality,
            ),
            None
        );
        assert_eq!(
            find_selection_index(
                &[Some(-0.0_f64)],
                SelectedValues::Multiple(&vec![Some(-0.0)]),
                default_item_equality,
            ),
            Some(0)
        );
        assert_eq!(
            find_selection_index(
                &[Some(1.0), Some(0.0)],
                SelectedValues::Multiple(&vec![Some(2.0), Some(0.0)]),
                default_item_equality,
            ),
            Some(1)
        );
        assert_eq!(
            find_selection_index(
                &[Some(1.0), Some(-0.0)],
                SelectedValues::Multiple(&vec![Some(2.0), Some(0.0)]),
                default_item_equality,
            ),
            None
        );
        assert_eq!(
            find_selection_index(
                &[Some(1.0), Some(-0.0)],
                SelectedValues::Multiple(&vec![Some(2.0), Some(-0.0)]),
                default_item_equality,
            ),
            Some(1)
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:70-73`.
    #[test]
    fn matches_nan_against_itself() {
        assert_eq!(
            find_selection_index(
                &[Some(1.0), Some(f64::NAN)],
                SelectedValues::Multiple(&vec![Some(f64::NAN)]),
                default_item_equality,
            ),
            Some(1)
        );
    }

    // Mirrors the `null` half of
    // `packages/react/src/internals/itemEquality.test.ts:70-73`: a JS `null` is a value
    // (inside the dynamic representation), so it matches itself.
    #[test]
    fn matches_null_against_itself() {
        let items = json_items(&[json!("a"), Value::Null]);
        let selection = json_items(&[Value::Null]);

        assert_eq!(
            find_selection_index(
                &items,
                SelectedValues::Multiple(&selection),
                default_item_equality,
            ),
            Some(1)
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:75-80`.
    #[test]
    fn never_matches_an_undefined_item_or_an_undefined_selected_value() {
        assert_eq!(
            find_selection_index(
                &[None, Some("b")],
                SelectedValues::Multiple(&vec![None, Some("b")]),
                default_item_equality,
            ),
            Some(1)
        );
        assert_eq!(
            find_selection_index(
                &[None],
                SelectedValues::<&str>::Multiple(&vec![None]),
                default_item_equality,
            ),
            None
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:82-90`: holes left by
    // unmounted items (the `None` slots) never match, and an empty selection is `None`.
    #[test]
    fn never_matches_a_hole_left_by_an_unmounted_item() {
        let mut sparse_items: Vec<Option<&str>> = vec![None, None];
        sparse_items.push(Some("c"));
        let mut sparse_selection: Vec<Option<&str>> = vec![None];
        sparse_selection.push(Some("c"));

        assert_eq!(
            find_selection_index(
                &sparse_items,
                SelectedValues::Multiple(&sparse_selection),
                default_item_equality,
            ),
            Some(2)
        );
        assert_eq!(
            find_selection_index(
                &sparse_items,
                SelectedValues::Multiple(&vec![]),
                default_item_equality,
            ),
            None
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:107-109`.
    #[test]
    fn resolve_does_not_claim_an_unselected_item() {
        let registry = ["a", "b", "c"].map(Some);

        assert_eq!(
            resolve_selected_index(
                1,
                Some(&"b"),
                &registry,
                &vec![Some("a"), Some("c")],
                default_item_equality,
                None,
            ),
            None
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:111-113`.
    #[test]
    fn resolve_claims_when_no_item_holds_the_index_yet() {
        let registry = ["a", "b", "c"].map(Some);

        assert_eq!(
            resolve_selected_index(
                2,
                Some(&"c"),
                &registry,
                &vec![Some("c")],
                default_item_equality,
                None,
            ),
            Some(2)
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:115-117`.
    #[test]
    fn resolve_claims_from_a_later_holder() {
        let registry = ["a", "b", "c"].map(Some);

        assert_eq!(
            resolve_selected_index(
                0,
                Some(&"a"),
                &registry,
                &vec![Some("a"), Some("c")],
                default_item_equality,
                Some(2),
            ),
            Some(0)
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:119-121`.
    #[test]
    fn resolve_leaves_the_index_with_an_earlier_selected_holder() {
        let registry = ["a", "b", "c"].map(Some);

        assert_eq!(
            resolve_selected_index(
                2,
                Some(&"c"),
                &registry,
                &vec![Some("a"), Some("c")],
                default_item_equality,
                Some(0),
            ),
            Some(0)
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:123-127`.
    #[test]
    fn resolve_takes_over_from_an_earlier_holder_that_is_no_longer_selected() {
        let registry = ["a", "b", "c"].map(Some);

        // `a` held the index but has been deselected, so `b` takes it and `c` then defers.
        assert_eq!(
            resolve_selected_index(
                0,
                Some(&"a"),
                &registry,
                &vec![Some("b"), Some("c")],
                default_item_equality,
                Some(0),
            ),
            Some(1)
        );
        assert_eq!(
            resolve_selected_index(
                2,
                Some(&"c"),
                &registry,
                &vec![Some("b"), Some("c")],
                default_item_equality,
                Some(1),
            ),
            Some(1)
        );
    }

    // Mirrors `packages/react/src/internals/itemEquality.test.ts:129-135`.
    #[test]
    fn resolve_takes_over_when_the_earlier_holder_has_left_the_registry() {
        let mut sparse_registry: Vec<Option<&str>> = vec![None];
        sparse_registry.push(Some("b"));
        sparse_registry.push(Some("c"));

        assert_eq!(
            resolve_selected_index(
                2,
                Some(&"c"),
                &sparse_registry,
                &vec![Some("c")],
                default_item_equality,
                Some(0),
            ),
            Some(2)
        );
    }

    // Port-owned pin for the helpers the selection anchor composes
    // (`packages/react/src/internals/itemEquality.ts:37-67` are cited by the implementation
    // spec's mechanism walkthrough but only exercised through `findSelectionIndex` upstream):
    // the membership test, the missing-selection short-circuit, and the null-safe comparison
    // wrapper's arms.
    #[test]
    fn the_membership_and_comparison_helpers_match_their_upstream_contract() {
        let selection = vec![Some("a"), None, Some("c")];

        // `undefined` slots never match; a missing selection is `false`
        // (`itemEquality.test.ts:75-80` semantics at the helper level).
        assert!(selected_value_includes(
            Some(&selection),
            Some(&"a"),
            default_item_equality
        ));
        assert!(!selected_value_includes(
            Some(&selection),
            Some(&"b"),
            default_item_equality
        ));
        assert!(!selected_value_includes(
            None,
            Some(&"a"),
            default_item_equality
        ));

        // First matching index, `undefined` items skipped, missing list is `None`
        // (`itemEquality.ts:53-67`).
        assert_eq!(
            find_item_index(Some(&selection), Some(&"c"), default_item_equality),
            Some(2)
        );
        assert_eq!(
            find_item_index(Some(&selection), Some(&"x"), default_item_equality),
            None
        );
        assert_eq!(
            find_item_index::<&str>(None, Some(&"x"), default_item_equality),
            None
        );

        // Two absences match; an absence never matches a presence
        // (`itemEquality.ts:17-20` at the wrapper level).
        assert!(compare_item_equality::<&str>(
            None,
            None,
            default_item_equality
        ));
        assert!(!compare_item_equality(
            None,
            Some(&"a"),
            default_item_equality
        ));
    }

    // Port-owned pin for the dirty check
    // (`packages/react/src/internals/itemEquality.ts:23-35`): the scalar branch is strict
    // `!==` (NaN always dirty), the list branch is the null-safe array equality.
    #[test]
    fn the_dirty_check_matches_its_upstream_contract() {
        assert!(is_selected_value_dirty(Some(&f64::NAN), Some(&f64::NAN)));
        assert!(!is_selected_value_dirty(Some(&"a"), Some(&"a")));
        assert!(is_selected_value_dirty(Some(&"a"), Some(&"b")));
        assert!(is_selected_value_dirty(Some(&"a"), None));

        assert!(!is_selected_values_dirty(
            &[1.0, 2.0],
            &[1.0, 2.0],
            default_item_equality
        ));
        assert!(is_selected_values_dirty(
            &[1.0, 2.0],
            &[1.0, 3.0],
            default_item_equality
        ));
    }

    // Port-owned pin for the removal filter
    // (`packages/react/src/internals/itemEquality.ts:126-134`): the selection is filtered
    // under the null-safe comparison.
    #[test]
    fn remove_item_filters_the_selection_under_the_comparison() {
        let selection = vec![Some("a"), Some("b"), Some("c")];

        assert_eq!(
            remove_item(&selection, Some(&"b"), default_item_equality),
            vec![Some("a"), Some("c")]
        );
    }
}
