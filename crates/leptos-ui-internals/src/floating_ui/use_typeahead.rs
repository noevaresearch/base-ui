//! Port of `packages/react/src/floating-ui-react/hooks/useTypeahead.ts` — the
//! printable-character matching hook that focuses/highlights a list item as the user
//! types (`specs/library/floating-ui-react/parts/hooks-navigation.md`, "useTypeahead":
//! matching is reported through `onMatch` and the consumer syncs state; the hook never
//! moves DOM focus).
//!
//! ## Rust adaptations
//!
//! - The `reference` and `floating` roles share ONE handler pair upstream
//!   (`sharedProps`, `useTypeahead.ts:241-246`) — the port clones the same
//!   [`ElementHandlers`] bag into both roles, so a keydown dispatched on either
//!   element (or a child of either — keydown bubbles) reaches the same session.
//! - `listRef`/`elementsRef` (`useTypeahead.ts:18,34` — `React.RefObject`s) are the
//!   consumer-owned [`ListRef`]/[`ElementsRef`] handles: mutable arrays read fresh at
//!   event time, the way upstream reads `.current` inside the handler.
//! - `activeIndex`/`selectedIndex` (`useTypeahead.ts:23,63`) are reactive sources
//!   ([`reactive_graph::traits::Get`]) rather than plain values: upstream re-renders
//!   supply the latest values to the stable `useStableCallback` handlers, while the
//!   port's hook runs once — so the props must be read fresh at event time (the
//!   `use_controlled` convention). `enabled`/`resetMs`/`disabledIndices` stay plain:
//!   upstream never re-reads them after a render in a way the test suite observes,
//!   matching the `use_click` port's snapshot treatment.
//! - The `listContent == null` guard (`useTypeahead.ts:148`) is unrepresentable: the
//!   port's `listRef` always holds a vector, so the guard's arms collapse (a
//!   missing entry is `None` inside the vector, handled by the matcher).
//! - `event.key.length !== 1` (`useTypeahead.ts:150`) counts UTF-16 code units; the
//!   port counts them the same way (`encode_utf16`) so astral-plane keys (emoji) are
//!   excluded exactly as upstream. The doubled-letter bail-out's `text[0]`/`text[1]`
//!   indexing (see [`allow_rapid_succession_of_first_letter`]) uses `char`s instead —
//!   identical for realistic list labels.
//! - `string.toLowerCase()` matching (`useTypeahead.ts:120,125`) is
//!   locale-independent in Rust (`str::to_lowercase` has no locale parameter), which
//!   is exactly the property the upstream Turkish-locale test pins
//!   (`useTypeahead.test.tsx:214-233`).
//! - The 750 ms buffer reset rides the `useTimeout` port ([`Timeout`]) and the
//!   open/selected reset rides the `useIsoLayoutEffect` port, per the implementation
//!   spec's per-call-site hook inventory
//!   (`specs/library/floating-ui-react/implementation.md`, "Base UI utility hooks (per
//!   call site)": `useTypeahead.ts:91`).
//! - `EMPTY_OBJECT` for the disabled case (`useTypeahead.ts:244`) is
//!   [`ElementProps::default`] — every role `None`.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::traits::{Get, GetUntracked};
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, FocusEvent, KeyboardEvent};

use leptos_ui_utils::use_iso_layout_effect;
use leptos_ui_utils::use_timeout;
use leptos_ui_utils::use_timeout::Timeout;

use crate::floating_ui::composite::{DisabledIndices, is_element_visible, is_list_index_disabled};
use crate::floating_ui::element;
use crate::floating_ui::element_props::{
    ElementEventHandler, ElementHandlers, ElementProps, FloatingContextSource,
};
use crate::floating_ui::event::stop_event;
use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::selectors;

/// The consumer-owned typed-string list — upstream's
/// `React.RefObject<Array<string | null>>` (`useTypeahead.ts:18`): indices match the
/// item elements', and a `None` slot is upstream's `null` entry (skipped while
/// matching).
pub type ListRef = Rc<RefCell<Vec<Option<String>>>>;

/// The optional item-element list — `React.RefObject<Array<HTMLElement | null>>`
/// (`useTypeahead.ts:34`): when an element exists for an index, matching skips it if
/// it is hidden or natively disabled (`useTypeahead.ts:30-33`).
pub type ElementsRef = Rc<RefCell<Vec<Option<Element>>>>;

/// `onMatch` (`useTypeahead.ts:27`) — invoked with the matching index, if found.
pub type OnMatchFn = Rc<dyn Fn(i32)>;

/// `onTyping` (`useTypeahead.ts:47`) — invoked with the typing activity as the user
/// types.
pub type OnTypingFn = Rc<dyn Fn(bool)>;

/// Port of `UseTypeaheadProps` (`useTypeahead.ts:12-64`) with the documented defaults
/// (`useTypeahead.ts:82-84`). No `Default`: `list_ref` and `active_index` are
/// upstream's required props.
pub struct UseTypeaheadProps<A, S> {
    /// `listRef` (`useTypeahead.ts:18`) — the typed-string list.
    pub list_ref: ListRef,
    /// `activeIndex` (`useTypeahead.ts:23`) — the consumer-controlled active index.
    pub active_index: A,
    /// `onMatch` (`useTypeahead.ts:27`).
    pub on_match: Option<OnMatchFn>,
    /// `elementsRef` (`useTypeahead.ts:34`).
    pub elements_ref: Option<ElementsRef>,
    /// `disabledIndices` (`useTypeahead.ts:43`).
    pub disabled_indices: Option<DisabledIndices>,
    /// `onTyping` (`useTypeahead.ts:47`).
    pub on_typing: Option<OnTypingFn>,
    /// `enabled` (`useTypeahead.ts:53` — default `true`).
    pub enabled: bool,
    /// `resetMs` (`useTypeahead.ts:58` — default `750`).
    pub reset_ms: u32,
    /// `selectedIndex` (`useTypeahead.ts:63`).
    pub selected_index: S,
}

/// Port of `getMatchingIndex` (`useTypeahead.ts:114-131`): the first index at or after
/// `start_index` (wrapping) whose entry starts with `string` case-insensitively and
/// passes `is_item_available`; `-1` when the list is empty or nothing matches.
///
/// Extracted verbatim so the matcher's table (start normalization, wrap, skipped
/// entries) is host-testable without a DOM.
pub fn get_matching_index(
    list: &[Option<String>],
    string: &str,
    start_index: i32,
    is_item_available: &dyn Fn(usize) -> bool,
) -> i32 {
    if list.is_empty() {
        return -1;
    }

    let length = list.len() as i32;
    let normalized_start_index = ((start_index % length) + length) % length;
    let lower_string = string.to_lowercase();

    for offset in 0..length {
        let index = ((normalized_start_index + offset) % length) as usize;
        // `!text?.toLowerCase().startsWith(lowerString)` (`useTypeahead.ts:125`): a
        // null entry has no `startsWith`, so the `!` of `undefined` skips it.
        let Some(text) = &list[index] else {
            continue;
        };
        if !text.to_lowercase().starts_with(&lower_string) || !is_item_available(index) {
            continue;
        }
        return index as i32;
    }
    -1
}

/// The doubled-letter bail-out predicate (`useTypeahead.ts:174-176`): rapid cycling
/// through same-letter items is allowed only when no *available* list entry starts
/// with two of the same letter (a word like "llama" or "aaron"). Unavailable entries
/// are vacuously allowed — they are skipped while matching, so a hidden or disabled
/// double-letter label must not block cycling (`useTypeahead.ts:170-173`).
pub fn allow_rapid_succession_of_first_letter(
    list: &[Option<String>],
    is_item_available: &dyn Fn(usize) -> bool,
) -> bool {
    list.iter().enumerate().all(|(index, text)| match text {
        // `text && isItemAvailable(index) ? ... : true` (`useTypeahead.ts:175`) —
        // the `text &&` is JS truthiness, so an *empty string* entry is vacuously
        // allowed too (it cannot double a first letter it doesn't have... and its
        // first and second characters are both `undefined` anyway).
        Some(text) if !text.is_empty() && is_item_available(index) => {
            // `text[0]?.toLowerCase() !== text[1]?.toLowerCase()`
            // (`useTypeahead.ts:175`) — first two characters, lowercased; a
            // single-character entry has no second character and therefore
            // cannot double its first.
            let first = text
                .chars()
                .next()
                .map(|c| c.to_lowercase().collect::<String>());
            let second = text
                .chars()
                .nth(1)
                .map(|c| c.to_lowercase().collect::<String>());
            first != second
        }
        _ => true,
    })
}

/// Port of `useTypeahead(context, props)` (`useTypeahead.ts:71-247`). Must be called
/// inside a reactive owner (the timer registers cleanups). Returns the shared
/// `reference` + `floating` handler bags, or the empty props when disabled.
pub fn use_typeahead<A, S>(
    context: impl Into<FloatingContextSource>,
    props: UseTypeaheadProps<A, S>,
) -> ElementProps
where
    A: Get<Value = Option<i32>> + GetUntracked<Value = Option<i32>> + Clone + 'static,
    S: Get<Value = Option<i32>> + GetUntracked<Value = Option<i32>> + Clone + 'static,
{
    let UseTypeaheadProps {
        list_ref,
        elements_ref,
        active_index,
        on_match: on_match_prop,
        disabled_indices,
        on_typing,
        enabled,
        reset_ms,
        selected_index,
    } = props;

    let store: Rc<FloatingRootStore> = context.into().root_store();
    let inner = store.rc();

    // `const open = store.useState('open')` (`useTypeahead.ts:89`).
    let open = inner.use_state(selectors::open);

    // `const timeout = useTimeout()` (`useTypeahead.ts:91`), the string buffer
    // (`:92`), and the index mirrors (`:93-94`).
    let timeout: Timeout = use_timeout();
    let string_ref: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let prev_index_ref: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(Some(
        selected_index
            .get_untracked()
            .or_else(|| active_index.get_untracked())
            .unwrap_or(-1),
    )));
    let match_index_ref: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

    // The per-keystroke availability predicate (`isItemAvailable`,
    // `useTypeahead.ts:101-112`), built per handler invocation from the current
    // `elementsRef`/`disabledIndices`. Visibility and native disabled state ride the
    // element; the explicit check resolves only `disabledIndices` against an empty
    // element list so its own fallbacks are skipped (`useTypeahead.ts:106-111`).
    let make_is_item_available = {
        let elements_ref = elements_ref.clone();
        move |index: usize| -> bool {
            let element = elements_ref
                .as_ref()
                .and_then(|elements| elements.borrow().get(index).cloned().flatten());
            let is_natively_disabled = element
                .as_ref()
                .map(|element| element.matches(":disabled").unwrap_or(false))
                .unwrap_or(false);
            if (element.is_some() && !is_element_visible(element.as_ref())) || is_natively_disabled
            {
                return false;
            }
            match &disabled_indices {
                None => true,
                Some(disabled_indices) => {
                    !is_list_index_disabled(&[], index as i32, Some(disabled_indices))
                }
            }
        }
    };

    // `onKeyDown` (`useTypeahead.ts:96-207`).
    let on_key_down: ElementEventHandler<KeyboardEvent> = {
        let list_ref = Rc::clone(&list_ref);
        let string_ref = Rc::clone(&string_ref);
        let prev_index_ref = Rc::clone(&prev_index_ref);
        let match_index_ref = Rc::clone(&match_index_ref);
        let active_index = active_index.clone();
        let selected_index = selected_index.clone();
        let on_match_prop = on_match_prop.clone();
        let on_typing = on_typing.clone();
        let open = open.clone();
        let timeout = timeout.clone();
        let make_is_item_available = make_is_item_available.clone();
        Rc::new(move |event: &KeyboardEvent| {
            // `const listContent = listRef.current` (`useTypeahead.ts:133`) — read
            // once per keystroke.
            let list_content = list_ref.borrow().clone();
            let key = event.key();

            // Space continues the in-progress typeahead session
            // (`useTypeahead.ts:135-139`).
            if !string_ref.borrow().is_empty() && key == " " {
                stop_event(event);
                if let Some(on_typing) = &on_typing {
                    on_typing(true);
                }
            }

            // Announce the end of typing when the accumulated string no longer
            // matches anything (`useTypeahead.ts:141-145`).
            if !string_ref.borrow().is_empty() && !string_ref.borrow().starts_with(' ') {
                let is_item_available = make_is_item_available.clone();
                if get_matching_index(&list_content, &string_ref.borrow(), 0, &|index| {
                    is_item_available(index)
                }) == -1
                    && key != " "
                {
                    if let Some(on_typing) = &on_typing {
                        on_typing(false);
                    }
                }
            }

            // Character keys only, without modifiers (`useTypeahead.ts:147-157`).
            // `listContent == null` (`:148`) is unrepresentable — see the module
            // docs.
            if key.encode_utf16().count() != 1
                || event.ctrl_key()
                || event.meta_key()
                || event.alt_key()
            {
                return;
            }

            if open.get_untracked() && key != " " {
                stop_event(event);
                if let Some(on_typing) = &on_typing {
                    on_typing(true);
                }
            }

            // Capture whether this is a new typing session before mutating the
            // string (`useTypeahead.ts:164-168`).
            let is_new_session = string_ref.borrow().is_empty();
            if is_new_session {
                *prev_index_ref.borrow_mut() = Some(
                    selected_index
                        .get_untracked()
                        .or_else(|| active_index.get_untracked())
                        .unwrap_or(-1),
                );
            }

            // Allows the user to cycle through items that start with the same
            // letter in rapid succession (`useTypeahead.ts:180-183`).
            let is_item_available = make_is_item_available.clone();
            let allow_rapid_succession_of_first_letter =
                allow_rapid_succession_of_first_letter(&list_content, &|index| {
                    is_item_available(index)
                });
            if allow_rapid_succession_of_first_letter && string_ref.borrow().as_str() == key {
                *string_ref.borrow_mut() = String::new();
                *prev_index_ref.borrow_mut() = match_index_ref.borrow().clone();
            }

            string_ref.borrow_mut().push_str(&key);
            {
                let string_ref = Rc::clone(&string_ref);
                let prev_index_ref = Rc::clone(&prev_index_ref);
                let match_index_ref = Rc::clone(&match_index_ref);
                let on_typing = on_typing.clone();
                timeout.start(reset_ms, move || {
                    *string_ref.borrow_mut() = String::new();
                    *prev_index_ref.borrow_mut() = match_index_ref.borrow().clone();
                    if let Some(on_typing) = &on_typing {
                        on_typing(false);
                    }
                });
            }

            // Starting index: a new session bases it on the current
            // selection/active item; an ongoing session continues from the last
            // matched index (`useTypeahead.ts:192-196`).
            let prev_index = if is_new_session {
                Some(
                    selected_index
                        .get_untracked()
                        .or_else(|| active_index.get_untracked())
                        .unwrap_or(-1),
                )
            } else {
                prev_index_ref.borrow().clone()
            };
            let start_index = prev_index.unwrap_or(0) + 1;

            let is_item_available = make_is_item_available.clone();
            let index =
                get_matching_index(&list_content, &string_ref.borrow(), start_index, &|index| {
                    is_item_available(index)
                });

            if index != -1 {
                if let Some(on_match_prop) = &on_match_prop {
                    on_match_prop(index);
                }
                *match_index_ref.borrow_mut() = Some(index);
            } else if key != " " {
                *string_ref.borrow_mut() = String::new();
                if let Some(on_typing) = &on_typing {
                    on_typing(false);
                }
            }
        })
    };

    // `onBlur` (`useTypeahead.ts:209-226`): the session survives focus moves within
    // the composite (reference <-> floating) and ends when focus leaves it entirely.
    let on_blur: ElementEventHandler<FocusEvent> = {
        let store = Rc::clone(&store);
        let string_ref = Rc::clone(&string_ref);
        let prev_index_ref = Rc::clone(&prev_index_ref);
        let match_index_ref = Rc::clone(&match_index_ref);
        let on_typing = on_typing.clone();
        let timeout = timeout.clone();
        Rc::new(move |event: &FocusEvent| {
            let next = event
                .related_target()
                .and_then(|target| target.dyn_into::<Element>().ok());
            let current_dom_reference_element = store.select(selectors::dom_reference_element);
            let current_floating_element = store.select(selectors::floating_element);
            let within_composite =
                element::contains(current_dom_reference_element.as_ref(), next.as_ref())
                    || element::contains(current_floating_element.as_ref(), next.as_ref());
            if within_composite {
                return;
            }

            timeout.clear();
            *string_ref.borrow_mut() = String::new();
            *prev_index_ref.borrow_mut() = match_index_ref.borrow().clone();
            if let Some(on_typing) = &on_typing {
                on_typing(false);
            }
        })
    };

    // The open/selected reset (`useTypeahead.ts:228-239`): when the popup is open —
    // or `selectedIndex` is absent — every render clears the buffer and match mirror
    // so stale sessions never leak into the next open. The reactive reads stand in
    // for the `[open, selectedIndex]` dependency array.
    {
        let open = open.clone();
        let selected_index = selected_index.clone();
        let timeout = timeout.clone();
        let match_index_ref = Rc::clone(&match_index_ref);
        let string_ref = Rc::clone(&string_ref);
        use_iso_layout_effect(move || {
            let open_value = open.get();
            let selected_index_value = selected_index.get();
            if !open_value && selected_index_value.is_some() {
                return;
            }

            timeout.clear();
            *match_index_ref.borrow_mut() = None;

            if !string_ref.borrow().is_empty() {
                *string_ref.borrow_mut() = String::new();
            }
        });
    }

    // `enabled ? { reference: sharedProps, floating: sharedProps } : {}`
    // (`useTypeahead.ts:243-246`) — one shared bag across both roles.
    if enabled {
        let shared_props = ElementHandlers {
            on_key_down: Some(on_key_down),
            on_blur: Some(on_blur),
            ..ElementHandlers::default()
        };
        ElementProps {
            reference: Some(shared_props.clone()),
            floating: Some(shared_props),
            item: None,
            trigger: None,
        }
    } else {
        ElementProps::default()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    fn every_available(_: usize) -> bool {
        true
    }

    fn list(items: &[&str]) -> Vec<Option<String>> {
        items.iter().map(|item| Some(item.to_string())).collect()
    }

    // Pins the matcher core (`useTypeahead.ts:114-131`): case-insensitive prefix
    // match starting after `start_index`, wrapping to the list head — the
    // 't','t','t' → 1,2,1 cycling upstream pins at the hook level
    // (`useTypeahead.test.tsx:135-142`).
    #[test]
    fn the_matcher_starts_after_the_start_index_and_wraps() {
        let list = list(&["one", "two", "three"]);

        assert_eq!(
            get_matching_index(&list, "t", 0, &every_available),
            1,
            "'t' from 0 matches 'two'"
        );
        assert_eq!(
            get_matching_index(&list, "t", 2, &every_available),
            2,
            "'t' from 2 matches 'three' before wrapping"
        );
        assert_eq!(
            get_matching_index(&list, "t", 3, &every_available),
            1,
            "'t' past the end normalizes to index 0, skips 'one', and lands on 'two'"
        );
        assert_eq!(
            get_matching_index(&list, "th", 1, &every_available),
            2,
            "a longer prefix disambiguates"
        );
    }

    // Pins the negative/overflow start normalization (`useTypeahead.ts:119` —
    // `((startIndex % length) + length) % length`, where JS `%` keeps the sign): a
    // -1 start normalizes to the last index and searches from there; a large start
    // wraps.
    #[test]
    fn the_matcher_normalizes_negative_and_overflow_starts() {
        let list = list(&["alpha", "beta", "beta-two"]);

        assert_eq!(
            get_matching_index(&list, "b", -1, &every_available),
            2,
            "-1 % 3 keeps its sign, so the search begins at index 2 ('beta-two')"
        );
        assert_eq!(
            get_matching_index(&list, "b", 7, &every_available),
            1,
            "7 % 3 = 1: the search starts at 'beta' itself"
        );
    }

    // Pins the entry filters (`useTypeahead.ts:125`): null entries are skipped
    // (`!text?.toLowerCase().startsWith(...)` is true for `undefined`), unavailable
    // entries are skipped through the predicate, and a full miss returns -1.
    #[test]
    fn the_matcher_skips_null_and_unavailable_entries() {
        let with_null: Vec<Option<String>> = vec![None, Some("two".into()), Some("three".into())];
        assert_eq!(
            get_matching_index(&with_null, "t", 0, &every_available),
            1,
            "the null head is skipped"
        );

        let unavailable_first = |index: usize| index != 0;
        let all = list(&["one", "two", "three"]);
        assert_eq!(
            get_matching_index(&all, "o", 0, &unavailable_first),
            -1,
            "'one' is the only 'o' match and it is unavailable"
        );
    }

    // Pins the empty-list arm (`useTypeahead.ts:115-117`) and the case-insensitive
    // fold (`useTypeahead.ts:120,125` — locale-independent lowercasing, the
    // property the Turkish-locale upstream test pins).
    #[test]
    fn the_matcher_handles_the_empty_list_and_folds_case() {
        assert_eq!(get_matching_index(&[], "t", 0, &every_available), -1);

        let list = list(&["Istanbul"]);
        assert_eq!(
            get_matching_index(&list, "i", 0, &every_available),
            0,
            "matching is a plain case fold, not locale-sensitive"
        );
    }

    // Pins the doubled-letter bail-out (`useTypeahead.ts:170-176`): an available
    // "aaron"-shaped entry blocks rapid cycling; an unavailable one must not (it is
    // skipped while matching, so it cannot be allowed to block cycling either) — the
    // upstream hidden-double-letter test (`useTypeahead.test.tsx:317-334`).
    #[test]
    fn the_bail_out_ignores_unavailable_doubled_letter_entries() {
        let available_doubled = list(&["apple", "aaron", "apricot"]);
        assert!(
            !allow_rapid_succession_of_first_letter(&available_doubled, &every_available),
            "the available 'aaron' entry blocks rapid cycling"
        );

        let hidden_doubled = list(&["aaron", "apple", "avocado"]);
        let index_0_unavailable = |index: usize| index != 0;
        assert!(
            allow_rapid_succession_of_first_letter(&hidden_doubled, &index_0_unavailable),
            "the hidden 'aaron' entry does not block rapid cycling"
        );
    }

    // Pins the vacuous arms (`useTypeahead.ts:175`): a null entry or an unavailable
    // entry cannot block (the ternary's else arm is `true`), and a
    // single-character/empty entry cannot double its first letter.
    #[test]
    fn the_bail_out_is_vacuously_true_for_null_and_trivial_entries() {
        let mixed: Vec<Option<String>> = vec![None, Some("a".into()), Some(String::new())];
        assert!(allow_rapid_succession_of_first_letter(
            &mixed,
            &every_available
        ));

        let with_second_char = list(&["ab", "ac"]);
        assert!(
            allow_rapid_succession_of_first_letter(&with_second_char, &every_available),
            "'a' vs 'b'/'c' — no doubled first letters"
        );

        let doubled = list(&["llama"]);
        assert!(!allow_rapid_succession_of_first_letter(
            &doubled,
            &every_available
        ));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;

    use reactive_graph::owner::LocalStorage;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::Set;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use leptos_ui_utils::merge_cleanups::CleanupFn;
    use web_sys::EventTarget;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    fn list(items: &[&str]) -> Vec<Option<String>> {
        items.iter().map(|item| Some(item.to_string())).collect()
    }

    fn attached(tag: &str) -> web_sys::HtmlElement {
        let document = web_sys::window().unwrap().document().unwrap();
        let element: web_sys::HtmlElement =
            document.create_element(tag).unwrap().dyn_into().unwrap();
        document.body().unwrap().append_child(&element).unwrap();
        element
    }

    /// An open store with the reference/floating elements seeded — the upstream
    /// harness's `useFloating({ open: true })` state (`useTypeahead.test.tsx:23-28`).
    fn store_with(
        dom_reference: &web_sys::HtmlElement,
        floating: &web_sys::HtmlElement,
    ) -> Rc<FloatingRootStore> {
        let store = FloatingRootStore::new(FloatingRootStoreOptions {
            open: true,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: crate::floating_ui::popup_trigger_map::PopupTriggerMap::new(),
            floating_id: None,
            sync_only: false,
            nested: false,
            on_open_change: None,
        });
        store.set_field(
            |state| &mut state.dom_reference_element,
            Some(dom_reference.clone().into()),
        );
        store.set_field(
            |state| &mut state.floating_element,
            Some(floating.clone().into()),
        );
        store
    }

    fn keyboard_event(key: &str) -> KeyboardEvent {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(key);
        init.set_bubbles(true);
        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap()
    }

    fn focus_out_event(related_target: Option<&EventTarget>) -> FocusEvent {
        let init = web_sys::FocusEventInit::new();
        init.set_bubbles(true);
        if let Some(related_target) = related_target {
            init.set_related_target(Some(related_target));
        }
        FocusEvent::new_with_focus_event_init_dict("focusout", &init).unwrap()
    }

    struct Hook {
        matches: Rc<RefCell<Vec<i32>>>,
        typing: Rc<RefCell<Vec<bool>>>,
    }

    /// Builds the hook over `list` with the upstream harness's consumer wiring
    /// (`useTypeahead.test.tsx:33-38` — `onMatch(index) { setActiveIndex(index) }`).
    fn hook_with(store: &Rc<FloatingRootStore>, list: Vec<Option<String>>) -> (ElementProps, Hook) {
        hook_with_elements(store, list, None)
    }

    fn hook_with_elements(
        store: &Rc<FloatingRootStore>,
        list: Vec<Option<String>>,
        elements_ref: Option<ElementsRef>,
    ) -> (ElementProps, Hook) {
        let matches: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
        let typing: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let active: RwSignal<Option<i32>, LocalStorage> = RwSignal::new_local(None);

        let matches_for_match = Rc::clone(&matches);
        let active_for_match = active.clone();
        let props = use_typeahead(
            Rc::clone(store),
            UseTypeaheadProps {
                list_ref: Rc::new(RefCell::new(list)),
                active_index: active.clone(),
                on_match: Some(Rc::new(move |index: i32| {
                    matches_for_match.borrow_mut().push(index);
                    active_for_match.set(Some(index));
                })),
                elements_ref,
                disabled_indices: None,
                on_typing: Some(Rc::new({
                    let typing = Rc::clone(&typing);
                    move |is_typing: bool| typing.borrow_mut().push(is_typing)
                })),
                enabled: true,
                reset_ms: 750,
                selected_index: RwSignal::new_local(None::<i32>),
            },
        );

        (props, Hook { matches, typing })
    }

    /// Attaches both roles and returns the merged cleanup — the caller must keep
    /// it alive for the listeners to stay registered (dropping a `CleanupFn`
    /// unsubscribes).
    fn attach(
        props: &ElementProps,
        reference: &web_sys::HtmlElement,
        floating: &web_sys::HtmlElement,
    ) -> CleanupFn {
        let reference_cleanup = props
            .reference
            .as_ref()
            .unwrap()
            .attach_to(reference.as_ref())
            .unwrap();
        let floating_cleanup = props
            .floating
            .as_ref()
            .unwrap()
            .attach_to(floating.as_ref())
            .unwrap();
        Box::new(leptos_ui_utils::merge_cleanups([
            Some(reference_cleanup),
            Some(floating_cleanup),
        ]))
    }

    /// The global setTimeout/clearTimeout stub — the stand-in for the upstream
    /// suite's fake timers (`useTypeahead.test.tsx:10-12`): registrations queue,
    /// and `flush` fires the live ones (the typeahead keeps at most one 750 ms job
    /// alive because every keystroke re-`start`s the same `Timeout`).
    struct TimeoutStub {
        original_set: JsValue,
        original_clear: JsValue,
        #[allow(dead_code)]
        set_closure: Closure<dyn FnMut(JsValue, JsValue) -> JsValue>,
        #[allow(dead_code)]
        clear_closure: Closure<dyn FnMut(JsValue)>,
        timeouts: Rc<RefCell<Vec<(u32, JsValue)>>>,
    }

    impl TimeoutStub {
        fn install() -> Self {
            let timeouts: Rc<RefCell<Vec<(u32, JsValue)>>> = Rc::new(RefCell::new(Vec::new()));
            let next_id = Cell::new(0u32);

            let queue = Rc::clone(&timeouts);
            let set_closure = Closure::wrap(Box::new(move |callback: JsValue, _delay: JsValue| {
                next_id.set(next_id.get() + 1);
                let id = next_id.get();
                queue.borrow_mut().push((id, callback));
                JsValue::from_f64(f64::from(id))
            })
                as Box<dyn FnMut(JsValue, JsValue) -> JsValue>);

            let queue = Rc::clone(&timeouts);
            let clear_closure = Closure::wrap(Box::new(move |id: JsValue| {
                let id = id.as_f64().unwrap_or(0.0) as u32;
                queue.borrow_mut().retain(|(queued, _)| *queued != id);
            }) as Box<dyn FnMut(JsValue)>);

            let global = js_sys::global();
            let original_set =
                js_sys::Reflect::get(&global, &"setTimeout".into()).expect("setTimeout readable");
            let original_clear = js_sys::Reflect::get(&global, &"clearTimeout".into())
                .expect("clearTimeout readable");
            js_sys::Reflect::set(&global, &"setTimeout".into(), set_closure.as_ref()).unwrap();
            js_sys::Reflect::set(&global, &"clearTimeout".into(), clear_closure.as_ref()).unwrap();

            Self {
                original_set,
                original_clear,
                set_closure,
                clear_closure,
                timeouts,
            }
        }

        fn flush(&self) {
            let ids: Vec<u32> = self.timeouts.borrow().iter().map(|(id, _)| *id).collect();
            for id in ids {
                let callback = {
                    let timeouts = self.timeouts.borrow();
                    timeouts
                        .iter()
                        .find(|(queued, _)| *queued == id)
                        .map(|(_, callback)| callback.clone())
                };
                self.timeouts
                    .borrow_mut()
                    .retain(|(queued, _)| *queued != id);
                if let Some(callback) = callback {
                    js_sys::Function::from(callback)
                        .call0(&JsValue::UNDEFINED)
                        .expect("the timeout callback threw");
                }
            }
        }
    }

    impl Drop for TimeoutStub {
        fn drop(&mut self) {
            let global = js_sys::global();
            js_sys::Reflect::set(&global, &"setTimeout".into(), &self.original_set).unwrap();
            js_sys::Reflect::set(&global, &"clearTimeout".into(), &self.original_clear).unwrap();
        }
    }

    // Pins the rapid-cycling contract (`useTypeahead.test.tsx:129-143`): repeated
    // first-letter presses cycle through every item starting with that letter and
    // wrap.
    #[wasm_bindgen_test]
    fn rapid_repeated_first_letter_presses_cycle_through_the_matching_items() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let reference = attached("input");
            let floating = attached("div");
            let store = store_with(&reference, &floating);
            let (props, hook) = hook_with(&store, list(&["one", "two", "three"]));
            let _cleanup = attach(&props, &reference, &floating);

            reference.dispatch_event(&keyboard_event("t")).unwrap();
            reference.dispatch_event(&keyboard_event("t")).unwrap();
            reference.dispatch_event(&keyboard_event("t")).unwrap();

            assert_eq!(
                *hook.matches.borrow(),
                vec![1, 2, 1],
                "t, t, t cycles 1 → 2 → 1"
            );
        }
        __owner.cleanup();
    }

    // Pins the bail-out (`useTypeahead.test.tsx:145-156`): with an available
    // doubled-letter entry, a repeated first letter keeps matching the same item.
    #[wasm_bindgen_test]
    fn an_available_doubled_letter_entry_blocks_rapid_cycling() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let reference = attached("input");
            let floating = attached("div");
            let store = store_with(&reference, &floating);
            let (props, hook) = hook_with(&store, list(&["apple", "aaron", "apricot"]));
            let _cleanup = attach(&props, &reference, &floating);

            reference.dispatch_event(&keyboard_event("a")).unwrap();
            reference.dispatch_event(&keyboard_event("a")).unwrap();

            // The second press does NOT restart from 'apple' (that would be rapid
            // cycling): the buffer extends to 'aa' and matches 'aaron' — the
            // upstream test's `toHaveBeenCalledWith(0)` pins that the *first* press
            // matched 0 and the bail-out kept the string accumulating
            // (`useTypeahead.test.tsx:145-156`).
            assert_eq!(
                *hook.matches.borrow(),
                vec![0, 1],
                "'a' matched 'apple'; the bail-out extended to 'aa' → 'aaron'"
            );
        }
        __owner.cleanup();
    }

    // Pins the full-string window (`useTypeahead.test.tsx:158-202`): a completed
    // string re-typed inside the 750 ms window does not re-match; after the reset
    // the same string advances to the next candidate, starting from the current
    // active index and looping.
    #[wasm_bindgen_test]
    fn a_full_string_re_type_waits_for_the_reset_window_then_advances_and_loops() {
        init_executor();
        let stub = TimeoutStub::install();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let reference = attached("input");
            let floating = attached("div");
            let store = store_with(&reference, &floating);
            let (props, hook) =
                hook_with(&store, list(&["Toy Story 2", "Toy Story 3", "Toy Story 4"]));
            let _cleanup = attach(&props, &reference, &floating);

            let type_toy = |reference: &web_sys::HtmlElement| {
                reference.dispatch_event(&keyboard_event("t")).unwrap();
                reference.dispatch_event(&keyboard_event("o")).unwrap();
                reference.dispatch_event(&keyboard_event("y")).unwrap();
            };

            // Every matching keystroke reports: 't', 'to', 'toy' all match index 0
            // (upstream asserts with `toHaveBeenCalledWith`, "any call" semantics —
            // `useTypeahead.test.tsx:158-202`).
            type_toy(&reference);
            assert_eq!(*hook.matches.borrow(), vec![0, 0, 0]);

            type_toy(&reference);
            assert_eq!(
                *hook.matches.borrow(),
                vec![0, 0, 0],
                "re-typing inside the window does not re-match"
            );

            stub.flush();
            type_toy(&reference);
            assert_eq!(
                *hook.matches.borrow(),
                vec![0, 0, 0, 1, 1, 1],
                "after the reset 'toy' advances to 1, starting from the active index"
            );

            stub.flush();
            type_toy(&reference);
            assert_eq!(*hook.matches.borrow(), vec![0, 0, 0, 1, 1, 1, 2, 2, 2]);

            stub.flush();
            type_toy(&reference);
            assert_eq!(
                *hook.matches.borrow(),
                vec![0, 0, 0, 1, 1, 1, 2, 2, 2, 0, 0, 0],
                "and wraps back to 0"
            );
        }
        __owner.cleanup();
        drop(stub);
    }

    // Pins the character filter (`useTypeahead.ts:147-157`): a key whose name is not
    // a single character ('CapsLock') returns before the session logic, so the
    // following letter still matches — the CapsLock test
    // (`useTypeahead.test.tsx:204-212`).
    #[wasm_bindgen_test]
    fn capslock_keypresses_do_not_break_matching() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let reference = attached("input");
            let floating = attached("div");
            let store = store_with(&reference, &floating);
            let (props, hook) = hook_with(&store, list(&["one", "two", "three"]));
            let _cleanup = attach(&props, &reference, &floating);

            reference
                .dispatch_event(&keyboard_event("CapsLock"))
                .unwrap();
            reference.dispatch_event(&keyboard_event("t")).unwrap();

            assert_eq!(*hook.matches.borrow(), vec![1]);
        }
        __owner.cleanup();
    }

    // Pins the case fold (`useTypeahead.test.tsx:214-233`): 'i' matches 'Istanbul'.
    // The upstream spy on `toLocaleLowerCase` is unrepresentable in Rust —
    // `str::to_lowercase` has no locale parameter, so locale-independence holds by
    // construction (the host suite asserts the same fold).
    #[wasm_bindgen_test]
    fn matching_folds_case_without_locale_sensitivity() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let reference = attached("input");
            let floating = attached("div");
            let store = store_with(&reference, &floating);
            let (props, hook) = hook_with(&store, vec![Some("Istanbul".into())]);
            let _cleanup = attach(&props, &reference, &floating);

            reference.dispatch_event(&keyboard_event("i")).unwrap();
            assert_eq!(*hook.matches.borrow(), vec![0]);
        }
        __owner.cleanup();
    }

    // Pins focus-independence (`useTypeahead.test.tsx:266-274`): a keydown bubbling
    // from an input nested inside the reference reaches the session.
    #[wasm_bindgen_test]
    fn matching_works_when_focus_is_within_the_reference_subtree() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let reference = attached("div");
            let nested_input: web_sys::HtmlElement = reference
                .owner_document()
                .unwrap()
                .create_element("input")
                .unwrap()
                .dyn_into()
                .unwrap();
            reference.append_child(&nested_input).unwrap();
            let floating = attached("div");
            let store = store_with(&reference, &floating);
            let (props, hook) = hook_with(&store, list(&["one", "two", "three"]));
            let _cleanup = attach(&props, &reference, &floating);

            nested_input.dispatch_event(&keyboard_event("t")).unwrap();
            assert_eq!(*hook.matches.borrow(), vec![1]);
        }
        __owner.cleanup();
    }

    // Pins the shared-bag composition (`useTypeahead.test.tsx:276-290`): the same
    // session continues across the reference and the floating element — 't' on the
    // reference, then 'h' bubbling from an item inside the floating element.
    #[wasm_bindgen_test]
    fn the_session_continues_across_the_reference_and_the_floating_element() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let reference = attached("input");
            let floating = attached("div");
            let store = store_with(&reference, &floating);
            let (props, hook) = hook_with(&store, list(&["one", "two", "three"]));
            let _cleanup = attach(&props, &reference, &floating);

            reference.dispatch_event(&keyboard_event("t")).unwrap();
            assert_eq!(*hook.matches.borrow(), vec![1]);

            let option: web_sys::HtmlElement = floating
                .owner_document()
                .unwrap()
                .create_element("div")
                .unwrap()
                .dyn_into()
                .unwrap();
            floating.append_child(&option).unwrap();
            option.dispatch_event(&keyboard_event("h")).unwrap();
            assert_eq!(
                *hook.matches.borrow(),
                vec![1, 2],
                "'th' matches 'three' through the floating-side bag"
            );
        }
        __owner.cleanup();
    }

    // Pins the typing indicator (`useTypeahead.test.tsx:292-305`): the first
    // keystroke reports `true`, the 750 ms reset reports `false`.
    #[wasm_bindgen_test]
    fn on_typing_reports_the_typing_activity() {
        init_executor();
        let stub = TimeoutStub::install();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let reference = attached("input");
            let floating = attached("div");
            let store = store_with(&reference, &floating);
            let (props, hook) = hook_with(&store, list(&["one", "two", "three"]));
            let _cleanup = attach(&props, &reference, &floating);

            reference.dispatch_event(&keyboard_event("t")).unwrap();
            assert_eq!(*hook.typing.borrow(), vec![true]);

            stub.flush();
            assert_eq!(*hook.typing.borrow(), vec![true, false]);
        }
        __owner.cleanup();
        drop(stub);
    }

    /// Builds an open store plus a floating listbox whose three options are
    /// attached and (optionally) hidden by index — the
    /// `ComboboxWithElementsRef` fixture (`useTypeahead.test.tsx:76-126`).
    /// `visibility` selects the `visibility: hidden` style over `display: none`.
    fn listbox_with_hidden(
        hidden_indices: &[usize],
        visibility: bool,
    ) -> (
        Rc<FloatingRootStore>,
        ElementsRef,
        web_sys::HtmlElement,
        web_sys::HtmlElement,
    ) {
        let reference = attached("input");
        let floating = attached("div");
        let elements_ref: ElementsRef = Rc::new(RefCell::new(Vec::new()));
        for index in 0..3usize {
            let option: web_sys::HtmlElement = floating
                .owner_document()
                .unwrap()
                .create_element("div")
                .unwrap()
                .dyn_into()
                .unwrap();
            if hidden_indices.contains(&index) {
                option
                    .style()
                    .set_property(
                        if visibility { "visibility" } else { "display" },
                        if visibility { "hidden" } else { "none" },
                    )
                    .unwrap();
            }
            floating.append_child(&option).unwrap();
            elements_ref.borrow_mut().push(Some(option.into()));
        }
        let store = store_with(&reference, &floating);
        (store, elements_ref, reference, floating)
    }

    // Pins the hidden-item skip (`useTypeahead.test.tsx:307-315`): with
    // `elementsRef`, a `display: none` item is excluded from matching.
    #[wasm_bindgen_test]
    fn a_display_none_item_is_skipped_when_elements_ref_is_provided() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let (store, elements_ref, reference, floating) = listbox_with_hidden(&[0], false);
            let (props, hook) = hook_with_elements(
                &store,
                list(&["apple", "apricot", "banana"]),
                Some(elements_ref),
            );
            let _cleanup = attach(&props, &reference, &floating);

            reference.dispatch_event(&keyboard_event("a")).unwrap();
            assert_eq!(*hook.matches.borrow(), vec![1], "'a' skips hidden 'apple'");
        }
        __owner.cleanup();
    }

    // Pins the hidden-double-letter interaction (`useTypeahead.test.tsx:317-334`):
    // a hidden doubled-letter item no longer blocks rapid cycling.
    #[wasm_bindgen_test]
    fn a_hidden_double_letter_item_does_not_block_rapid_cycling() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let (store, elements_ref, reference, floating) = listbox_with_hidden(&[0], false);
            let (props, hook) = hook_with_elements(
                &store,
                list(&["aaron", "apple", "avocado"]),
                Some(elements_ref),
            );
            let _cleanup = attach(&props, &reference, &floating);

            reference.dispatch_event(&keyboard_event("a")).unwrap();
            reference.dispatch_event(&keyboard_event("a")).unwrap();
            assert_eq!(*hook.matches.borrow(), vec![1, 2]);
        }
        __owner.cleanup();
    }

    // Pins the visibility:hidden skip (`useTypeahead.test.tsx:336-348`).
    #[wasm_bindgen_test]
    fn a_visibility_hidden_item_is_skipped_when_elements_ref_is_provided() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let (store, elements_ref, reference, floating) = listbox_with_hidden(&[0], true);
            let (props, hook) = hook_with_elements(
                &store,
                list(&["apple", "apricot", "banana"]),
                Some(elements_ref),
            );
            let _cleanup = attach(&props, &reference, &floating);

            reference.dispatch_event(&keyboard_event("a")).unwrap();
            assert_eq!(*hook.matches.borrow(), vec![1]);
        }
        __owner.cleanup();
    }

    // Port-authored (no upstream test — source-derived, `useTypeahead.ts:209-226`):
    // a focus-out whose related target leaves the composite ends the session (the
    // next keystroke starts fresh from the active index), while a focus-out that
    // stays inside the floating element keeps it (the accumulated string misses, so
    // the cycle still matches).
    #[wasm_bindgen_test]
    fn focus_outside_the_composite_ends_the_session_but_focus_inside_keeps_it() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let reference = attached("input");
            let floating = attached("div");
            let store = store_with(&reference, &floating);
            let (props, hook) = hook_with(&store, list(&["one", "two", "three"]));
            let _cleanup = attach(&props, &reference, &floating);

            // Session 1: 't' matches 1, then focus leaves the composite entirely —
            // the reset lands, and the next 't' starts from the (wired) active index.
            reference.dispatch_event(&keyboard_event("t")).unwrap();
            assert_eq!(*hook.matches.borrow(), vec![1]);
            let outsider = attached("button");
            reference
                .dispatch_event(&focus_out_event(Some(outsider.as_ref() as &EventTarget)))
                .unwrap();
            assert_eq!(
                *hook.typing.borrow(),
                vec![true, false],
                "the blur reports typing end"
            );
            reference.dispatch_event(&keyboard_event("t")).unwrap();
            assert_eq!(
                *hook.matches.borrow(),
                vec![1, 2],
                "the fresh session starts from the active index and lands on 'three'"
            );

            // Session 2: focus moves within the composite (into the floating
            // element) — the session survives, so 't' extends the buffer and rapid
            // cycling advances instead of restarting.
            let inside = attached("span");
            floating.append_child(&inside).unwrap();
            reference
                .dispatch_event(&focus_out_event(Some(inside.as_ref() as &EventTarget)))
                .unwrap();
            reference.dispatch_event(&keyboard_event("t")).unwrap();
            assert_eq!(
                *hook.matches.borrow(),
                vec![1, 2, 1],
                "the kept session cycles 't' → 1"
            );
        }
        __owner.cleanup();
    }
}
