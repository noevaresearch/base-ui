//! The combobox root's mutator layer — the state-machine functions of
//! `packages/react/src/combobox/root/AriaCombobox.tsx` (`library: combobox`'s
//! headless core), ported over the store spine ([`crate::combobox::store`]) as a
//! host-testable runtime struct.
//!
//! Upstream defines each mutator as a `useStableCallback` closing over the root's
//! props, state hooks, and refs (`AriaCombobox.tsx:600-930`). The port's equivalent
//! of React's closure-per-render is one runtime struct holding the shared handles
//! (the store, the context refs, the controlled-state writers, and the local
//! `React.useState` flags — `queryChangedAfterOpen`, `closeQuery` — as cells). The
//! reactive wiring (which reactive writer backs each "state hook") is injected
//! through [`RootRuntimeCallbacks`], so the same logic runs host-side against cells
//! and browser-side against signals.
//!
//! Ported, by function:
//!
//! - [`emit_highlight`] (`AriaCombobox.tsx:600-613`) — the `lastHighlightRef` dedup:
//!   index `-1` clears to the sentinel and bails when already there; otherwise the
//!   `{value, index}` pair is stashed and `onItemHighlighted` fires.
//! - [`set_indices`] (`AriaCombobox.tsx:615-646`) — writes the provided indices
//!   atomically in one `store.update`, then emits the highlight: `null` →
//!   `emit_highlight(None, -1)`, else the value at `valuesRef[activeIndex]`.
//! - [`set_input_value`] (`AriaCombobox.tsx:648-726`) — the user callback first,
//!   bail on cancel, the `hadInputClearRef` note, the typed-input classification
//!   (`compositionend`, or a non-empty `inputType` that isn't
//!   `insertReplacementText`), the frozen-query release when typing proves the popup
//!   stays open, the `pendingQueryHighlightRef` scheduling, the non-virtualized
//!   list scroll reset over the overflow ancestors, the auto-highlight arm
//!   (index 0 when `activeIndex == null` and `(open || inline)`), the
//!   `inputClear`-on-inside-popup pending-restore flag, and the commit.
//! - [`handle_interrupted_reopen`] (`AriaCombobox.tsx:728-748`) — the reopen path
//!   that releases the frozen query, resetting `queryChangedAfterOpen` when the
//!   input is empty or matches the selection, and clearing the pending input with
//!   an `inputClear` details when the interrupted close owned it.
//! - [`set_open`] (`AriaCombobox.tsx:750-829`) — the same-value bail, the
//!   Escape-with-no-`Empty` propagation allowance, the user callback + cancel gate,
//!   the reopen handling, the close-side query freeze (single: freeze + empty-query
//!   flag reset; multiple: freeze + activeIndex clear inside the popup + the
//!   outside/inline immediate clear with the `isItemPress` custom flag), the commit,
//!   and the Field touched/focused/validation block on focus-out/outside-press
//!   closes with the input inside the popup.
//! - [`set_selected_value`] (`AriaCombobox.tsx:831-854`) — the user callback +
//!   cancel gate, the commit, and the input-fill rule (single + input outside the
//!   popup, or `selectionMode: 'none'` with `fillInputOnItemPress` and a mounted
//!   popup).
//! - [`handle_selection`] (`AriaCombobox.tsx:856-920`) — the `selectionEventRef`
//!   override swap, the link-target short-circuit (`#`-anchors close instead of
//!   selecting), the multiple-mode toggle over `selectedValueIncludes`/
//!   `removeItem` with the `wasFiltering` clear branch (inside-popup: clear with
//!   `isItemPress` + the toggled-value highlight hold; outside: close), and the
//!   single-mode select-then-close.
//! - [`handle_unmount`] (`AriaCombobox.tsx:972-992`) — the `onOpenChangeComplete`
//!   close notification, the flag/query resets, the mode-split index clearing, and
//!   the input reconciliation: multiple + non-empty + not-already-cleared → clear;
//!   single inside-popup non-empty → clear; single outside → sync to the selected
//!   label (or clear, reason `inputClear`, when the selection label is empty).
//!
//! Rust adaptations:
//!
//! - Upstream's `React.useState` flags (`queryChangedAfterOpen`, `closeQuery`) and
//!   refs (`lastHighlightRef`, `hadInputClearRef`, `pendingQueryHighlightRef`) port
//!   to cells on the runtime — the root's closure-lifetime handles, not reactive
//!   state; nothing reads them reactively except the derived `query`, which reads
//!   them directly (upstream reads them at render scope through the same closure).
//! - The runtime is generic over the event type `E` (default `web_sys::Event`):
//!   `ChangeCommandDetails` pins `E = Event` and a `web_sys::Event` cannot be
//!   constructed off-wasm, so the host tests instantiate the runtime over a fake
//!   event — exactly the genericity the details type's module docs describe ("the
//!   genericity is what lets non-DOM stubs instantiate the type in host tests").
//!   The two DOM reads the mutators make on the event — `event.type` +
//!   `event.inputType` (the typed-input classification, `AriaCombobox.tsx:668-672`)
//!   and `getTarget(event)` (`:858`) — port to the injected accessors
//!   [`RootRuntimeCallbacks::event_input_info`] and
//!   [`RootRuntimeCallbacks::event_target`]; the wasm wiring supplies the real
//!   reads, the host tests supply fakes. The synthetic placeholder event
//!   (`createChangeEventDetails` with no event — type `'base-ui'`, behavior.md
//!   "Events": the reopen-cleanup clear's details) is likewise the injected
//!   [`RootRuntimeCallbacks::synthetic_event`] factory.
//! - The injected callbacks are plain `Rc<dyn Fn>` — the NOOP-seeded command-slot
//!   convention ([`crate::combobox::store::ComboboxStoreContext`]). React's
//!   `useStableCallback` identity guarantee is structural here: a method on one
//!   struct cannot change identity.
//! - `stringifyValueLabel` ports to a closure over
//!   [`leptos_ui_internals::resolve_value_label::stringify_as_label`] with the
//!   store-carried `item_to_string_label`.
//! - The DOM-touching arm (the query-change scroll reset, `AriaCombobox.tsx:696-712`)
//!   reads the elements from the store/context and runs wasm-side; on host the
//!   elements are `None` and the arm no-ops, matching upstream's `if (list)` guard.
//!   `getOverflowAncestors` re-exports through the internals crate (the crate does
//!   not depend on `floating-ui-utils` directly — the arrow.rs re-export precedent).
//! - The Field `validationMode === 'onBlur'` read and the internal
//!   `fillInputOnItemPress` prop (an Autocomplete-driven prop the combobox tests
//!   never set, implementation.md gap 2) are carried on [`RootMode`] as plain
//!   booleans the wiring layer fills.
//! - The highlight/change reason strings are the upstream `reason-parts.ts` values;
//!   the combobox-emitted reasons (`item-press`, `input-change`, `input-clear`) are
//!   defined here — following the floating-ui reasons registry's placement note —
//!   until the `infra: internals` unit ports `reason-parts.ts` behind a re-export.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use serde_json::Value;
use wasm_bindgen::JsCast;
use web_sys::Element;

use crate::combobox::store::{ComboboxStore, SetIndicesInput};
use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::item_equality::{remove_item, selected_value_includes};

/// `REASONS.itemPress` (`reason-parts.ts:8`).
pub const REASON_ITEM_PRESS: &str = "item-press";
/// `REASONS.inputChange` (`reason-parts.ts:14`).
pub const REASON_INPUT_CHANGE: &str = "input-change";
/// `REASONS.inputClear` (`reason-parts.ts:15`).
pub const REASON_INPUT_CLEAR: &str = "input-clear";

/// The `pendingQueryHighlightRef` payload (`AriaCombobox.tsx:267-274`): the
/// scheduled post-filter highlight.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PendingQueryHighlight {
    pub has_query: bool,
    pub selection: bool,
    /// The value a selection-driven clear just added, so the restore can keep it
    /// highlighted instead of returning to the open anchor.
    pub toggled_value: Option<Value>,
}

/// The `setIndices` options shape as the mutator receives it
/// (`AriaCombobox.tsx:617-621`). `None` means "leave unchanged"; `Some(None)` is
/// upstream `null` (clear the index).
#[derive(Clone, Debug, Default)]
pub struct IndicesOptions {
    pub active_index: Option<Option<usize>>,
    pub selected_index: Option<Option<usize>>,
    pub reason: Option<String>,
}

/// The mode/config slice the mutators read from the root's props — snapshotted at
/// runtime construction, exactly as upstream's closures capture the derived
/// booleans (`AriaCombobox.tsx:283-291`).
#[derive(Clone, Debug)]
pub struct RootMode {
    /// `selectionMode === 'single'`.
    pub single: bool,
    /// `selectionMode === 'multiple'`.
    pub multiple: bool,
    /// `selectionMode === 'none'`.
    pub none: bool,
    /// Whether the `items` prop exists (`AriaCombobox.tsx:283`).
    pub has_items: bool,
    /// The resolved auto-highlight mode (`AriaCombobox.tsx:285-291`): `false`
    /// (`None`), `'input-change'`, or `'always'`.
    pub auto_highlight_mode: Option<String>,
    /// Whether the combobox renders inline (`AriaCombobox.tsx:280`'s prop).
    pub inline: bool,
    /// The Field `validationMode === 'onBlur'` read (the validation commit gate in
    /// `setOpen`'s focus-out block, `AriaCombobox.tsx:819-825`).
    pub validate_on_blur: bool,
    /// The internal `fillInputOnItemPress` prop (`AriaCombobox.tsx:135`;
    /// `false` unless the Autocomplete consumer sets it, implementation.md gap 2).
    pub fill_input_on_item_press: bool,
}

impl RootMode {
    /// Derives the mode snapshot from the store state (`single`/`multiple`/
    /// `hasItems`/`inline`; the Autocomplete-driven and Field-driven members stay
    /// wiring-supplied).
    pub fn from_state(state: &crate::combobox::store::ComboboxState) -> Self {
        let selection_mode = state.selection_mode.as_str();
        Self {
            single: selection_mode == "single",
            multiple: selection_mode == "multiple",
            none: selection_mode == "none",
            has_items: state.items.is_some(),
            auto_highlight_mode: match state.auto_highlight.as_str() {
                "always" => Some("always".into()),
                "true" | "input-change" => Some("input-change".into()),
                _ => None,
            },
            inline: state.inline,
            validate_on_blur: true,
            fill_input_on_item_press: false,
        }
    }
}

/// The callbacks the runtime drives — the injected seam between the pure mutator
/// logic and the reactive layer backing upstream's `useControlled` states and the
/// Field/form integrations.
pub struct RootRuntimeCallbacks<E> {
    /// Commits the open state (upstream `setOpenUnwrapped`).
    pub set_open_unwrapped: Rc<dyn Fn(bool)>,
    /// Commits the input value (upstream `setInputValueUnwrapped`).
    pub set_input_value_unwrapped: Rc<dyn Fn(String)>,
    /// Commits the selected value (upstream `setSelectedValueUnwrapped`).
    pub set_selected_value_unwrapped: Rc<dyn Fn(Value)>,
    /// The user's `onOpenChange` (upstream `props.onOpenChange`).
    pub on_open_change: Rc<dyn Fn(bool, &RuntimeDetails<E>)>,
    /// The user's `onInputValueChange` (upstream `props.onInputValueChange`).
    pub on_input_value_change: Rc<dyn Fn(String, &RuntimeDetails<E>)>,
    /// The user's `onSelectedValueChange` (upstream `onSelectedValueChange`).
    pub on_selected_value_change: Rc<dyn Fn(Value, &RuntimeDetails<E>)>,
    /// The user's `onItemHighlighted` (upstream `onItemHighlighted(value, details)`
    /// — the details' `index` carried as the second argument).
    pub on_item_highlighted: Rc<dyn Fn(Option<Value>, usize, &str)>,
    /// Releases the frozen query (upstream `setCloseQuery`).
    pub set_close_query: Rc<dyn Fn(Option<String>)>,
    /// Sets the query-changed flag (upstream `setQueryChangedAfterOpen`).
    pub set_query_changed_after_open: Rc<dyn Fn(bool)>,
    /// Marks the Field touched (upstream `setTouched`).
    pub set_touched: Rc<dyn Fn(bool)>,
    /// Marks the Field focused (upstream `setFocused`).
    pub set_focused: Rc<dyn Fn(bool)>,
    /// Runs Field validation on close (upstream `validation.commit`).
    pub validation_commit: Rc<dyn Fn(&Value)>,
    /// Notifies open-change completion (upstream `onOpenChangeComplete`).
    pub on_open_change_complete: Rc<dyn Fn(bool)>,
    /// Reads the current input value (upstream reads `inputRef.current.value`).
    pub input_value: Rc<dyn Fn() -> String>,
    /// Whether the input is inside the popup (upstream
    /// `store.state.inputInsidePopup` — read live because the composition is
    /// decided at input-ref time).
    pub input_inside_popup: Rc<dyn Fn() -> bool>,
    /// `(event.type, event.inputType)` for the typed-input classification
    /// (`AriaCombobox.tsx:668-672`) — the DOM read the host tests fake.
    pub event_input_info: Rc<dyn Fn(&E) -> (String, Option<String>)>,
    /// `getTarget(event)` (`AriaCombobox.tsx:858`) — the DOM read the host tests
    /// fake.
    pub event_target: Rc<dyn Fn(&E) -> Option<Element>>,
    /// The synthetic placeholder event factory (`createChangeEventDetails` with no
    /// event — upstream constructs a `new Event('base-ui')`).
    pub synthetic_event: Rc<dyn Fn() -> E>,
}

/// The shared handles the mutators close over — the store, the mode snapshot, the
/// callbacks, and the local `React.useState`/ref state as cells.
pub struct RootRuntimeHandles<E = web_sys::Event> {
    pub store: Rc<ComboboxStore>,
    pub mode: RootMode,
    pub callbacks: RootRuntimeCallbacks<E>,
    /// `queryChangedAfterOpen` (`AriaCombobox.tsx:248`).
    pub query_changed_after_open: Cell<bool>,
    /// `closeQuery` (`AriaCombobox.tsx:249`).
    pub close_query: RefCell<Option<String>>,
    /// `lastHighlightRef` (`AriaCombobox.tsx:265`): the sentinel
    /// (`Err(())` — [`INITIAL_LAST_HIGHLIGHT`]) or the last `{value, index}` pair.
    pub last_highlight: RefCell<Result<(Value, usize), ()>>,
    /// `hadInputClearRef` (`AriaCombobox.tsx:261`).
    pub had_input_clear: Cell<bool>,
    /// `pendingQueryHighlightRef` (`AriaCombobox.tsx:267-274`).
    pub pending_query_highlight: RefCell<Option<PendingQueryHighlight>>,
}

/// The initial `lastHighlightRef` value — upstream `INITIAL_LAST_HIGHLIGHT`
/// (`root/utils/constants.ts:1-5`), the cleared-highlight sentinel. The port's
/// `Err(())` is the sentinel arm; `Ok((value, index))` is a stashed highlight.
pub const INITIAL_LAST_HIGHLIGHT: Result<(Value, usize), ()> = Err(());

/// The details type the mutators emit — upstream
/// `AriaCombobox.ChangeEventDetails` whose one custom flag the close-path clear
/// carries (`isItemPress`, `AriaCombobox.tsx:802-805`) rides the `(bool,)` tuple.
pub type RuntimeDetails<E> = BaseUIChangeEventDetails<(bool,), E>;

/// `stringifyValueLabel` (`AriaCombobox.tsx:239-241`): the label of a value under
/// the store-carried `item_to_string_label`.
pub fn stringify_value_label(
    value: &Value,
    item_to_string_label: Option<&Rc<dyn Fn(&Value) -> String>>,
) -> String {
    let callback = item_to_string_label
        .as_ref()
        .map(|callback| callback.as_ref() as &dyn Fn(&Value) -> String);
    leptos_ui_internals::resolve_value_label::stringify_as_label(value, callback)
}

impl<E: Clone + 'static> RootRuntimeHandles<E> {
    /// Builds the runtime over a store + mode + callbacks, seeding the cells to
    /// their upstream initial values.
    pub fn new(
        store: Rc<ComboboxStore>,
        mode: RootMode,
        callbacks: RootRuntimeCallbacks<E>,
    ) -> Rc<Self> {
        Rc::new(Self {
            store,
            mode,
            callbacks,
            query_changed_after_open: Cell::new(false),
            close_query: RefCell::new(None),
            last_highlight: RefCell::new(INITIAL_LAST_HIGHLIGHT),
            had_input_clear: Cell::new(false),
            pending_query_highlight: RefCell::new(None),
        })
    }

    /// The derived query (`AriaCombobox.tsx:339`): the frozen close query while
    /// closed, else the trimmed live input value.
    pub fn query(&self) -> String {
        let close_query = self.close_query.borrow().clone();
        if !self.store.select(|state| state.open) {
            if let Some(close_query) = close_query {
                return close_query;
            }
        }
        self.input_value().trim().to_string()
    }

    /// Reads the live input value through the injected reader.
    pub fn input_value(&self) -> String {
        (self.callbacks.input_value)()
    }

    /// Whether the input is inside the popup, read live.
    pub fn input_inside_popup(&self) -> bool {
        (self.callbacks.input_inside_popup)()
    }

    /// `emitHighlight` (`AriaCombobox.tsx:600-613`).
    pub fn emit_highlight(&self, value: Option<Value>, index: isize, reason: &str) {
        if index == -1 {
            let already_initial = matches!(&*self.last_highlight.borrow(), Err(()));
            if already_initial {
                return;
            }
            *self.last_highlight.borrow_mut() = INITIAL_LAST_HIGHLIGHT;
        } else {
            *self.last_highlight.borrow_mut() =
                Ok((value.clone().unwrap_or(Value::Null), index as usize));
        }

        (self.callbacks.on_item_highlighted)(value, index as usize, reason);
    }

    /// `setIndices` (`AriaCombobox.tsx:615-646`).
    pub fn set_indices(&self, options: IndicesOptions) {
        let IndicesOptions {
            active_index,
            selected_index,
            reason,
        } = options;

        let store = self.store.clone();
        store.update(|next, _| {
            if let Some(active_index) = active_index {
                next.active_index = active_index;
            }
            if let Some(selected_index) = selected_index {
                next.selected_index = selected_index;
            }
            // Upstream's per-key Object.is skip is Store.update's change report; the
            // indices are plain options — report changed whenever any arm wrote.
            true
        });

        let Some(active_index) = active_index else {
            return;
        };

        let reason = reason
            .as_deref()
            .unwrap_or(leptos_ui_internals::floating_ui::reasons::NONE);

        if let Some(active_index) = active_index {
            let value = self
                .store
                .context
                .values_ref
                .borrow()
                .get(active_index)
                .cloned();
            self.emit_highlight(value, active_index as isize, reason);
        } else {
            self.emit_highlight(None, -1, reason);
        }
    }

    /// Converts [`IndicesOptions`] into the command-slot shape
    /// ([`SetIndicesInput`]) — the two shapes are the same vocabulary; the
    /// conversion exists so parts calling through the context command hit the same
    /// path as direct calls.
    pub fn indices_to_command(options: IndicesOptions) -> SetIndicesInput {
        SetIndicesInput {
            active_index: options.active_index,
            selected_index: options.selected_index,
            reason: options.reason,
        }
    }

    /// The close-query release (`setCloseQuery(null)`).
    pub fn release_close_query(&self) {
        (self.callbacks.set_close_query)(None);
        *self.close_query.borrow_mut() = None;
    }

    /// The non-virtualized list scroll reset (`AriaCombobox.tsx:696-712`): walks
    /// the list's overflow ancestors inside the popup and scrolls the first
    /// scrollable one to the top. Host-side (no elements) this no-ops, matching
    /// upstream's `if (list)` guard.
    fn reset_list_scroll(&self) {
        let Some(list) = self.store.select(|state| state.list_element.clone()) else {
            return;
        };
        let popup = self.store.context.popup_ref.borrow().clone();

        let first_child = list.first_element_child().unwrap_or(list);
        use leptos_ui_internals::floating_ui::{OverflowAncestor, get_overflow_ancestors};
        for ancestor in get_overflow_ancestors(&first_child, Vec::new(), false) {
            let OverflowAncestor::Element(ancestor) = ancestor else {
                break;
            };
            let inside = match &popup {
                Some(popup) => popup.contains(Some(&ancestor)),
                None => ancestor.get_attribute("role").as_deref() != Some("dialog"),
            };
            if !inside {
                break;
            }

            if let Some(html) = ancestor.dyn_ref::<web_sys::HtmlElement>() {
                if leptos_ui_internals::scrollable::is_scrollable_y(html, false) {
                    html.set_scroll_top(0);
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// `setInputValue` (`AriaCombobox.tsx:648-726`).
    pub fn set_input_value(&self, next: String, event_details: &RuntimeDetails<E>) {
        (self.callbacks.on_input_value_change)(next.clone(), event_details);

        if event_details.is_canceled() {
            return;
        }

        // A canceled selection clear must not suppress close-completion cleanup.
        self.had_input_clear
            .set(event_details.reason == REASON_INPUT_CLEAR);

        if event_details.reason == REASON_INPUT_CHANGE {
            // A controlled popup may ignore a close request. Resuming input proves the
            // popup is remaining open, so release the query captured for an exit
            // animation.
            if self.store.select(|state| state.open) && self.close_query.borrow().is_some() {
                self.release_close_query();
            }

            let (event_type, input_type) = (self.callbacks.event_input_info)(&event_details.event);
            let input_type = input_type.as_deref().unwrap_or("");
            // Treat composition commits as typed input; autofill may omit `inputType`
            // or report `insertReplacementText`.
            let is_typed_input = event_type == "compositionend"
                || (!input_type.is_empty() && input_type != "insertReplacementText");

            if is_typed_input {
                let has_query = !next.trim().is_empty();
                if has_query {
                    (self.callbacks.set_query_changed_after_open)(true);
                    self.query_changed_after_open.set(true);
                }
                // Defer index updates until after the filtered items have been derived
                // to ensure `onItemHighlighted` receives the latest item.
                *self.pending_query_highlight.borrow_mut() = Some(PendingQueryHighlight {
                    has_query,
                    ..Default::default()
                });

                // Virtualized lists own their scroller. Reset regular lists directly so
                // a stale composite registry cannot select a reordered item and
                // scrolling cannot escape the popup.
                if !self.store.select(|state| state.virtualized) {
                    self.reset_list_scroll();
                }

                if has_query
                    && self.mode.auto_highlight_mode.is_some()
                    && self.store.select(|state| state.active_index).is_none()
                    && (self.store.select(|state| state.open) || self.mode.inline)
                {
                    self.store
                        .set_field(|state| &mut state.active_index, Some(0));
                }
            }
        } else if event_details.reason == REASON_INPUT_CLEAR
            && next.is_empty()
            && self.input_inside_popup()
        {
            // A programmatic clear of an active query (e.g. after selecting an item
            // with the input inside the popup): restore the highlight to the selected
            // item.
            *self.pending_query_highlight.borrow_mut() = Some(PendingQueryHighlight {
                has_query: false,
                selection: true,
                ..Default::default()
            });
        }

        (self.callbacks.set_input_value_unwrapped)(next);
    }

    /// `handleInterruptedReopen` (`AriaCombobox.tsx:728-748`).
    pub fn handle_interrupted_reopen(&self, is_input_change: bool) {
        let input_value = self.input_value();
        // Preserve values supplied with the reopen rather than owned by the
        // interrupted close.
        let clears_pending_input = !is_input_change
            && self.input_inside_popup()
            && !self.mode.inline
            && !input_value.is_empty()
            && (Some(input_value.trim().to_string()) == *self.close_query.borrow()
                || input_value == self.selected_label_string());

        // Keep the flag while a visible filter survives so the `items` sync cannot
        // overwrite it.
        if !is_input_change
            && (clears_pending_input
                || input_value.is_empty()
                || self.input_matches_selected_value())
        {
            (self.callbacks.set_query_changed_after_open)(false);
            self.query_changed_after_open.set(false);
        }

        self.release_close_query();

        if clears_pending_input {
            // Cleanup clears omit the selection flag and reopening gesture.
            self.set_input_value(
                String::new(),
                &self.details(REASON_INPUT_CLEAR, None, false),
            );
        }
    }

    /// `inputMatchesSelectedValue` (`AriaCombobox.tsx:568-569`).
    pub fn input_matches_selected_value(&self) -> bool {
        self.mode.single
            && !self.input_inside_popup()
            && self.input_value() == self.selected_label_string()
    }

    /// `selectedLabelString` (`AriaCombobox.tsx:341`).
    pub fn selected_label_string(&self) -> String {
        if self.mode.single {
            stringify_value_label(
                &self.store.select(|state| state.selected_value.clone()),
                self.store
                    .select(|state| state.item_to_string_label.clone())
                    .as_ref(),
            )
        } else {
            String::new()
        }
    }

    /// The details constructor (`createChangeEventDetails(reason, event, …)`): a
    /// `None` event falls back to the injected synthetic placeholder.
    fn details(&self, reason: &str, event: Option<E>, is_item_press: bool) -> RuntimeDetails<E> {
        BaseUIChangeEventDetails::new(
            reason,
            event.unwrap_or_else(|| (self.callbacks.synthetic_event)()),
            None,
            (is_item_press,),
        )
    }

    /// `setOpen` (`AriaCombobox.tsx:750-829`).
    pub fn set_open(&self, next_open: bool, event_details: &RuntimeDetails<E>) {
        if self.store.select(|state| state.open) == next_open {
            return;
        }

        // If the `Empty` component is not used, the positioner or popup should be
        // hidden with CSS. In this case, allow the Escape key to bubble to close a
        // parent popup if there are no items to show.
        if event_details.reason == leptos_ui_internals::floating_ui::reasons::ESCAPE_KEY
            && self.mode.has_items
            && self.flat_filtered_values().is_empty()
            && self.store.context.empty_ref.borrow().is_none()
        {
            event_details.allow_propagation();
        }

        (self.callbacks.on_open_change)(next_open, event_details);

        if event_details.is_canceled() {
            return;
        }

        if next_open && self.close_query.borrow().is_some() {
            // `ComboboxInput` calls `setInputValue` before `setOpen`, so on an
            // input-change reopen `inputValue` is still the pre-keystroke value and
            // the typed filter always survives.
            self.handle_interrupted_reopen(event_details.reason == REASON_INPUT_CHANGE);
        }

        if !next_open && self.query_changed_after_open.get() {
            if self.mode.single {
                if !self.mode.inline {
                    let query = self.query();
                    *self.close_query.borrow_mut() = Some(query.clone());
                    (self.callbacks.set_close_query)(Some(query));
                }
                // Avoid a flicker when closing the popup with an empty query.
                if self.query().is_empty() {
                    (self.callbacks.set_query_changed_after_open)(false);
                    self.query_changed_after_open.set(false);
                }
            } else if self.mode.multiple {
                if !self.mode.inline {
                    // Freeze the current query so filtering remains stable while
                    // exiting.
                    let query = self.query();
                    *self.close_query.borrow_mut() = Some(query.clone());
                    (self.callbacks.set_close_query)(Some(query));
                }

                if self.input_inside_popup() {
                    self.set_indices(IndicesOptions {
                        active_index: Some(None),
                        ..Default::default()
                    });
                }

                // Clear the input immediately on close while retaining filtering via
                // closeQuery for exit animations if the input is outside the popup.
                // When the input is inside the popup, defer the clear until unmount so
                // the filtered list doesn't flash to unfiltered during the exit
                // animation.
                if !self.input_inside_popup() || self.mode.inline {
                    self.set_input_value(
                        String::new(),
                        &self.details(
                            REASON_INPUT_CLEAR,
                            Some(event_details.event.clone()),
                            event_details.reason == REASON_ITEM_PRESS,
                        ),
                    );
                }
            }
        }

        (self.callbacks.set_open_unwrapped)(next_open);

        if !next_open
            && self.input_inside_popup()
            && (event_details.reason == leptos_ui_internals::floating_ui::reasons::FOCUS_OUT
                || event_details.reason == leptos_ui_internals::floating_ui::reasons::OUTSIDE_PRESS)
        {
            (self.callbacks.set_touched)(true);
            (self.callbacks.set_focused)(false);

            if self.mode.validate_on_blur {
                let value_to_validate = if self.mode.none {
                    Value::String(self.input_value())
                } else {
                    self.store.select(|state| state.selected_value.clone())
                };
                (self.callbacks.validation_commit)(&value_to_validate);
            }
        }
    }

    /// `setSelectedValue` (`AriaCombobox.tsx:831-854`).
    pub fn set_selected_value(&self, next_value: Value, event_details: &RuntimeDetails<E>) {
        (self.callbacks.on_selected_value_change)(next_value.clone(), event_details);

        if event_details.is_canceled() {
            return;
        }

        (self.callbacks.set_selected_value_unwrapped)(next_value.clone());

        let should_fill_input = (self.mode.none
            && self.store.context.popup_ref.borrow().is_some()
            && self.mode.fill_input_on_item_press)
            || (self.mode.single && !self.input_inside_popup());

        if should_fill_input {
            self.set_input_value(
                stringify_value_label(
                    &next_value,
                    self.store
                        .select(|state| state.item_to_string_label.clone())
                        .as_ref(),
                ),
                &self.details(&event_details.reason.clone(), None, false),
            );
        }
    }

    /// `handleSelection` (`AriaCombobox.tsx:856-920`). The `selectionEventRef`
    /// override is passed in by the caller: upstream reads the ref inside the
    /// closure (`AriaCombobox.tsx:858-860`), but the port's context ref is
    /// concretely a `web_sys::Event` while the runtime is generic over `E` — so
    /// the wasm wiring performs the take-and-clear and passes the result (or
    /// `None`, for the event-as-override fallback) here, keeping the funnel logic
    /// host-testable.
    pub fn handle_selection(
        &self,
        override_event: Option<E>,
        fallback_event: &E,
        item_value: Value,
    ) {
        let target_el = (self.callbacks.event_target)(fallback_event);
        let event_details = BaseUIChangeEventDetails::new(
            REASON_ITEM_PRESS,
            override_event.unwrap_or_else(|| fallback_event.clone()),
            None,
            (false,),
        );

        // Let the link handle the click.
        if let Some(href) = target_el
            .as_ref()
            .and_then(|target| target.closest("a").ok().flatten())
            .and_then(|anchor| anchor.get_attribute("href"))
        {
            if href.starts_with('#') {
                self.set_open(false, &event_details);
            }
            return;
        }

        if self.mode.multiple {
            let current_selected_value = self
                .store
                .select(|state| state.selected_value.clone())
                .as_array()
                .cloned()
                .unwrap_or_default();
            let comparer = self
                .store
                .select(|state| state.is_item_equal_to_value.clone());
            let item_values: Vec<Option<Value>> = current_selected_value
                .iter()
                .map(|value| Some(value.clone()))
                .collect();
            let is_currently_selected = selected_value_includes(
                Some(&item_values),
                Some(&item_value),
                |a: &Value, b: &Value| comparer(a, b),
            );
            let next_values = if is_currently_selected {
                remove_item(&item_values, Some(&item_value), |a: &Value, b: &Value| {
                    comparer(a, b)
                })
            } else {
                let mut next = item_values;
                next.push(Some(item_value.clone()));
                next
            };
            let next_value = Value::Array(
                next_values
                    .into_iter()
                    .map(|slot| slot.unwrap_or(Value::Null))
                    .collect(),
            );

            self.set_selected_value(next_value, &event_details);

            if event_details.is_canceled() {
                return;
            }

            let was_filtering = !self.input_value().trim().is_empty();
            if !was_filtering {
                return;
            }

            if self.input_inside_popup() {
                self.set_input_value(
                    String::new(),
                    &self.details(REASON_INPUT_CLEAR, Some(event_details.event.clone()), true),
                );
                // A newly selected item stays highlighted through the clear; a
                // deselection falls back to the standard selection anchor.
                let mut pending = self.pending_query_highlight.borrow_mut();
                if let Some(pending) = pending.as_mut() {
                    if !is_currently_selected {
                        pending.toggled_value = Some(item_value);
                    }
                }
            } else {
                self.set_open(false, &event_details);
            }
        } else {
            self.set_selected_value(item_value, &event_details);

            if event_details.is_canceled() {
                return;
            }

            self.set_open(false, &event_details);
        }
    }

    /// `handleUnmount` (`AriaCombobox.tsx:972-992`).
    pub fn handle_unmount(&self) {
        self.store.set_field(|state| &mut state.mounted, false);
        (self.callbacks.on_open_change_complete)(false);
        (self.callbacks.set_query_changed_after_open)(false);
        self.query_changed_after_open.set(false);
        self.release_close_query();

        if self.mode.none {
            self.set_indices(IndicesOptions {
                active_index: Some(None),
                selected_index: Some(None),
                ..Default::default()
            });
        } else {
            self.set_indices(IndicesOptions {
                active_index: Some(None),
                ..Default::default()
            });
        }

        // Multiple selection mode: if the user typed a filter and didn't select in
        // multiple mode, clear the input after close completes to avoid mid-exit
        // flicker and start fresh on next open.
        if self.mode.multiple && !self.input_value().is_empty() && !self.had_input_clear.get() {
            self.set_input_value(
                String::new(),
                &self.details(REASON_INPUT_CLEAR, None, false),
            );
        }

        // Single selection mode: clear the inside-popup input so the next open is
        // blank; sync the outside input to the selected value.
        if self.mode.single {
            if self.input_inside_popup() {
                if !self.input_value().is_empty() {
                    self.set_input_value(
                        String::new(),
                        &self.details(REASON_INPUT_CLEAR, None, false),
                    );
                }
            } else {
                let string_val = self.selected_label_string();
                if self.input_value() != string_val {
                    // If no selection was made, treat this as clearing the typed
                    // filter.
                    let reason = if string_val.is_empty() {
                        REASON_INPUT_CLEAR
                    } else {
                        leptos_ui_internals::floating_ui::reasons::NONE
                    };
                    self.set_input_value(string_val, &self.details(reason, None, false));
                }
            }
        }
    }

    /// The `flatFilteredValues` projection (`AriaCombobox.tsx:443-451`) — the
    /// selection values of the currently visible items, read from the
    /// `values_ref` registry the items write (the derived-memo wiring is at the
    /// composition layer; the runtime reads the live registry).
    pub fn flat_filtered_values(&self) -> Vec<Value> {
        self.store.context.values_ref.borrow().clone()
    }
}
