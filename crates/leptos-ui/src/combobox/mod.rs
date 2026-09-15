//! Combobox — the port of `packages/react/src/combobox/**` (TODO item `library:
//! combobox`).
//!
//! This iteration lands the unit's non-DOM core, the layers every part subscribes to:
//!
//! - [`store`] — the two-tier store over the shared [`ReactStore`] engine: the
//!   reactive [`ComboboxState`] shape, the non-reactive [`ComboboxStoreContext`]
//!   (refs + NOOP-seeded command slots), and the selector table including the
//!   derived predicates (`has_selected_value`'s `[]`-reads-as-empty rule,
//!   `is_selected`'s comparer fan).
//! - [`items`] — the derived-items pipeline: `findCollectionItem`, the lazy
//!   `valueToItem` index with the first-occurrence-wins dedup and dev warning, the
//!   `data` passthrough, and the label resolution with the fallback chain.
//! - [`root_utils`] — the popup-id convention, the highlight sentinels, and the two
//!   collator filter factories (wasm-bound, over the ported core filter) plus the
//!   `useComboboxFilter` composition.
//! - [`parts_util`] — `usePopupSide`, the chip-removal index walk,
//!   `clickHighlightedItem` (the `selectionEventRef` tagging), and
//!   `handleInputPress` (the press funnel).
//!
//! The remaining surface — the root's three `useControlled` states, the mutators
//! (`setOpen`/`setInputValue`/`setSelectedValue`/`handleSelection`), the derived
//! filtered-items memo, the prop bags, the floating-hook composition, the parts, and
//! the hidden form control — is downstream work gated on this spine.

use crate::combobox::store::ComboboxStore;

pub mod items;
pub mod parts_util;
pub mod root_runtime;
#[cfg(test)]
mod root_runtime_tests;
pub mod root_utils;
pub mod store;
pub mod value_chips;
#[cfg(test)]
mod value_chips_tests;

// The parts' DOM-wiring layer (the value-chips batch's part components).
pub mod chip_remove_wiring;
pub mod chip_wiring;
pub mod chips_wiring;
pub mod clear;
pub mod group_wiring;
pub mod input_runtime;
#[cfg(test)]
mod input_runtime_tests;
pub mod label_wiring;
pub mod list_wiring;
pub mod popup_wiring;
pub mod portal_wiring;
pub mod positioner_wiring;
pub mod trigger_runtime;
pub mod value;

pub use clear::{ComboboxClearProps, ComboboxClearState, execute_clear_click};
pub use value::{ComboboxValueProps, combobox_value_display, display_to_text};

pub use items::{ItemCollection, create_combobox_items, find_collection_item};
pub use parts_util::{
    InputPressEvent, click_highlighted_item, get_chip_navigation_keys,
    get_index_after_chip_removal, handle_input_press, use_list_empty, use_popup_side,
};
pub use root_utils::{
    ComboboxFilter, FilterItemToString, INITIAL_LAST_HIGHLIGHT_INDEX, NO_ACTIVE_VALUE,
    UseComboboxFilterOptions, create_collator_item_filter, create_single_selection_collator_filter,
    get_combobox_popup_id, use_combobox_filter,
};
pub use store::{
    AutoHighlight, ChangeCommandDetails, ComboboxState, ComboboxStoreContext, InteractionType,
    SelectionMode, SetIndicesInput, Side, TransitionStatus, selectors,
};

/// The `useComboboxRootContext` accessor (upstream
/// `ComboboxRootContext.tsx:30-38`): the store every part consumes. The port's
/// components pass the store handle explicitly (the reactive-owner plumbing replaces
/// React's context provider for the store layer); the throwing-outside-Root contract
/// is enforced at the part-call sites.
pub fn use_combobox_root_context(store: &ComboboxStore) -> &ComboboxStore {
    store
}
