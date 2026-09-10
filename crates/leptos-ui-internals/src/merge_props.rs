//! Port of `packages/react/src/merge-props/` — the standalone props-merging utility
//! (`index.ts:1` re-exports `mergeProps.ts`; the `infra: merge-props` TODO item).
//!
//! Upstream is a headless, pure fold over props objects with no React runtime usage —
//! the single `react` import is type-only (`mergeProps.ts:1`), there is no state and no
//! DOM (`specs/library/merge-props/behavior.md`, "State model" and "DOM structure &
//! portal behavior" sections are N/A). The whole surface is four exports:
//! `mergeProps` (`mergeProps.ts:42-83`), `mergePropsN` (`:99-115`),
//! `makeEventPreventable` (`:268-274`), and `mergeClassNames` (`:276-290`).
//!
//! The "machine" is a left-to-right accumulator fold
//! (`specs/library/merge-props/implementation.md`, "State machine / hooks used"): a
//! props-*getter* argument (any function, `isPropsGetter` `:204-208`) REPLACES the
//! accumulator wholesale (`mergeInto` `:126-131`, `resolvePropsGetter` `:210-219`); a
//! props *object* folds in through `mutablyMergeInto` (`:153-188`), whose per-key
//! `switch` (`:162-184`) gives the unit its whole behavioral contract — `style` folds
//! through `mergeObjects` (`:166-172`), `className` through `mergeClassNames`
//! (`:173-176`), lexically-classified `on*` handlers chain through
//! `mergeEventHandlers` (`:177-179`), and everything else is overwritten outright
//! (`:181`). The cross-cutting "last argument wins" precedence
//! (`specs/library/merge-props/behavior.md`, cross-cutting rule) is emergent from the
//! fold direction plus these per-key strategies, not a precedence table.
//!
//! ## Rust adaptations
//!
//! The port re-homes the merge machinery that the `useRenderElement` port previously
//! carried inline (see that module's former handoff note) so the crate mirrors
//! upstream's layering — `useRenderElement.tsx:11` imports all three exports from this
//! unit, and [`crate::use_render_element`] does the same here. The generic
//! `Record<string, any>` bag specializes to the crate's typed vocabulary
//! ([`RenderElementProps`] / [`RenderElementHandlers`], the outProps shape the
//! use-render unit owns) — the same specialization the useRenderElement port recorded.
//!
//! - **The two entrypoints collapse into one.** `mergeProps`'s four typed overloads for
//!   2–5 arguments (`mergeProps.ts:42-60`) exist to type fixed arities; the
//!   implementation (`:61-83`) is `mergePropsN` over positional slots, including the
//!   6th+-argument silent drop (implementation.md "Anything in source" item 2) and the
//!   falsy-first-argument fast path (`:62-64`, item 1). Rust has no rest-arity, so
//!   [`merge_props_n`] (`mergePropsN`, `:99-115`) is the one entrypoint; an absent bag
//!   is simply not in the vec, and a one-element vec takes the same
//!   `createInitialMergedProps` path (`:103-105`) the fast path short-circuits to.
//! - **`isEventHandler`'s lexical classification** (`:190-202` — key starts `on`, third
//!   char uppercase, value function or `undefined`) dissolves into the typed handler
//!   slots: a slot is a handler by construction and an absent one is `None`, which is
//!   the `undefined`-entry-skip behavior (`mergeEventHandlers` passes a lone side
//!   through unwrapped, `:222-227` — behavior.md "Events", the undefined-slots test).
//! - **`isSyntheticEvent`'s dispatch-time duck-typing** (`:292-294`) and the resulting
//!   synthetic/non-synthetic branch (`:232-248`) collapse into the type system: every
//!   slot receives a [`BaseUIEvent`]-wrapped dispatch, so prevention capability is
//!   universal. The wrapper *is* `makeEventPreventable`'s stamp —
//!   [`BaseUIEvent::new`] stands in for the augmentation (`types.rs`, the port-side
//!   analog note), which is why a standalone `makeEventPreventable` export has no
//!   separate Rust shape (upstream's `useButton` imports it, `useButton.ts:8`; the
//!   port's handlers just receive the marked dispatch). The non-synthetic branch's
//!   run-both behavior is the no-mark case — nothing calls
//!   [`BaseUIEvent::prevent_base_ui_handler`], so both sides run (behavior.md
//!   "Events", the non-standard-handler tests).
//! - **`wrapEventHandler`'s lone-handler wrapping** (`:252-266`) bakes into the slot
//!   type: the attach seam constructs the [`BaseUIEvent`] per dispatch before the
//!   fan-out ([`RenderElementHandlers::attach_to`]), so a lone handler is preventable
//!   by construction (behavior.md "Events", the lone/first-position preventability
//!   tests).
//! - **Handler return values.** Upstream's chain returns the rightmost handler's return
//!   value and discards the rest (`:243`, `:246-248`); the port's
//!   `ElementEventHandler` returns `()`, so the contract (implementation.md item 9)
//!   collapses to unit — no observer exists in the typed vocabulary.
//! - **`EMPTY_PROPS`** (`:10`) — the module-level shared constant returned by
//!   `mergePropsN([])` (`:100-102`) and handed to leading getters (`:120`) — becomes a
//!   fresh [`RenderElementProps::default`]: the contract (an empty props object) is
//!   preserved, the shared-mutable-constant aliasing hazard (implementation.md item 4)
//!   deliberately is not.
//! - **The leading-getter defensive copy** (`:117-124`) is structural in Rust: a getter
//!   returns an *owned* bag, so later merging can never alias the getter's source
//!   object — the no-mutation contract (behavior.md "Edge cases") holds by ownership.
//!   The non-leading-getter in-place mutation quirk (implementation.md item 3) is
//!   equally exact: the getter's returned owned value is what later bags merge into.
//! - **`ref` is not merged** (JSDoc `:33`): the bag's `ref_callback` member falls
//!   through the plain-prop overwrite arm, which only replaces when the later bag
//!   actually carries the member — the `for...in`-over-own-keys rule (absent keys
//!   never overwrite). The real ref fork lives in [`crate::use_render_element`]
//!   (`useRenderElement.tsx:99-103`).
//! - **`for...in` inherited-enumerables** (implementation.md item 8): Rust struct
//!   fields have no prototype chain — own-key iteration is the only iteration.
//! - **The only runtime dependency**, `@base-ui/utils/mergeObjects` (`mergeProps.ts:2`,
//!   used solely for the `style` case `:166-172`), is the already-ported
//!   `leptos_ui_utils` `mergeObjects` unit; over the port's ordered style pairs its
//!   right-wins spread with reference passthrough is [`merge_styles`].

use std::rc::Rc;

use crate::floating_ui::element_props::ElementEventHandler;
use crate::types::BaseUIEvent;
use crate::use_render_element::{RenderElementHandlers, RenderElementProps};

/// One intrinsic props bag of a merge list — upstream's `InputProps<T>`
/// (`mergeProps.ts:7-8`): a props record or a props getter (the function form; the
/// `undefined` member is an absent vec slot).
pub enum PropsSource {
    /// A props record.
    Static(RenderElementProps),
    /// A props getter: resolved wholesale against the merged-so-far props and
    /// *replacing* them — the getter owns handler chaining and prevention gating
    /// (`mergeProps.ts:25-31`, `:210-219`).
    Getter(RenderPropsGetter),
}

/// The props-getter callable — receives the merged props up to that point
/// (`mergeProps.ts:27`).
pub type RenderPropsGetter = Rc<dyn Fn(&RenderElementProps) -> RenderElementProps>;

/// `mergeClassNames` (`mergeProps.ts:276-290`): the later (rightmost) class string
/// comes first in the concatenation.
pub fn merge_class_names(our_class: Option<String>, their_class: Option<String>) -> Option<String> {
    match (their_class, our_class) {
        (Some(theirs), Some(ours)) => Some(format!("{theirs} {ours}")),
        (Some(theirs), None) => Some(theirs),
        (None, ours) => ours,
    }
}

/// `mergeObjects` over the style pairs (`mergeProps.ts:166-172`): the later
/// (rightmost) declarations win per property, both sides retained otherwise.
pub fn merge_styles(
    our_style: Vec<(String, String)>,
    their_style: Vec<(String, String)>,
) -> Vec<(String, String)> {
    let mut merged = our_style;
    for (property, value) in their_style {
        match merged
            .iter_mut()
            .find(|(existing, _)| *existing == property)
        {
            Some(slot) => slot.1 = value,
            None => merged.push((property, value)),
        }
    }
    merged
}

/// `mergeEventHandlers` (`mergeProps.ts:221-250`) over the [`BaseUIEvent`]-typed
/// slots: the later (more external) handler runs first; the earlier handler runs
/// unless the dispatch was marked with [`BaseUIEvent::prevent_base_ui_handler`] — the
/// shared-mark cell standing in for the JS augmented object's identity across the
/// whole composed chain.
pub fn merge_event_handlers<E: Clone + 'static>(
    our_handler: Option<ElementEventHandler<BaseUIEvent<E>>>,
    their_handler: Option<ElementEventHandler<BaseUIEvent<E>>>,
) -> Option<ElementEventHandler<BaseUIEvent<E>>> {
    match (our_handler, their_handler) {
        (ours, None) => ours,
        (None, theirs) => theirs,
        (Some(ours), Some(theirs)) => Some(Rc::new(move |event: &BaseUIEvent<E>| {
            theirs(event);
            if !event.base_ui_handler_prevented() {
                ours(event);
            }
        })),
    }
}

/// Folds one later bag into the accumulated merged props — `mutablyMergeInto`
/// (`mergeProps.ts:153-188`) over the typed vocabulary: handlers compose (later runs
/// first, mark-gated), `class` concatenates later-first, `style` merges per-key,
/// plain attributes are replaced per key by the later bag, and `ref` is only replaced
/// when the later bag actually carries one (upstream's `for...in` iterates the later
/// bag's own keys; absent keys never overwrite).
pub(crate) fn merge_into(merged: &mut RenderElementProps, later: RenderElementProps) {
    let RenderElementProps {
        handlers,
        class,
        style,
        inner_html,
        ref_callback,
    } = later;

    let RenderElementHandlers {
        on_focus,
        on_blur,
        on_click,
        on_mouse_down,
        on_context_menu,
        on_mouse_move,
        on_key_down,
        on_key_up,
        on_pointer_down,
        attributes: incoming_attributes,
    } = handlers;

    let RenderElementHandlers {
        on_focus: our_focus,
        on_blur: our_blur,
        on_click: our_click,
        on_mouse_down: our_mouse_down,
        on_context_menu: our_context_menu,
        on_mouse_move: our_mouse_move,
        on_key_down: our_key_down,
        on_key_up: our_key_up,
        on_pointer_down: our_pointer_down,
        attributes: our_attributes,
    } = std::mem::take(&mut merged.handlers);

    let mut attributes = our_attributes;
    for (name, value) in incoming_attributes {
        match attributes
            .iter_mut()
            .find(|(existing, _)| *existing == name)
        {
            Some(slot) => slot.1 = value,
            None => attributes.push((name, value)),
        }
    }

    merged.handlers = RenderElementHandlers {
        on_focus: merge_event_handlers(our_focus, on_focus),
        on_blur: merge_event_handlers(our_blur, on_blur),
        on_click: merge_event_handlers(our_click, on_click),
        on_mouse_down: merge_event_handlers(our_mouse_down, on_mouse_down),
        on_context_menu: merge_event_handlers(our_context_menu, on_context_menu),
        on_mouse_move: merge_event_handlers(our_mouse_move, on_mouse_move),
        on_key_down: merge_event_handlers(our_key_down, on_key_down),
        on_key_up: merge_event_handlers(our_key_up, on_key_up),
        on_pointer_down: merge_event_handlers(our_pointer_down, on_pointer_down),
        attributes,
    };

    merged.class = merge_class_names(merged.class.take(), class);
    merged.style = merge_styles(std::mem::take(&mut merged.style), style);
    // A plain prop key: the later bag wins only when it actually carries the member
    // (upstream's `for...in` iterates the later bag's own keys; absent keys never
    // overwrite).
    if inner_html.is_some() {
        merged.inner_html = inner_html;
    }
    if ref_callback.is_some() {
        merged.ref_callback = ref_callback;
    }
}

/// Resolves one [`PropsSource`] against the merged-so-far props —
/// `createInitialMergedProps`/`mergeInto` (`mergeProps.ts:117-131`). A getter replaces
/// the accumulated props wholesale (`resolvePropsGetter`, `:210-219`).
pub(crate) fn resolve_source(source: PropsSource, previous: RenderElementProps) -> RenderElementProps {
    match source {
        PropsSource::Static(props) => {
            let mut merged = previous;
            merge_into(&mut merged, props);
            merged
        }
        PropsSource::Getter(getter) => getter(&previous),
    }
}

/// `mergePropsN` over the bag vocabulary (`mergeProps.ts:99-115`): left-to-right
/// fold, later bags winning per the [`merge_into`] rules. The variadic `mergeProps`
/// form (`:42-83`) is the same fold at fixed arities — see the module docs.
pub fn merge_props_n(props: Vec<PropsSource>) -> RenderElementProps {
    let mut bags = props;
    let Some(first) = (!bags.is_empty()).then(|| bags.remove(0)) else {
        // `mergePropsN([])` returns `EMPTY_PROPS` (`:100-102`); the port hands back a
        // fresh empty bag instead of a shared constant (module docs).
        return RenderElementProps::default();
    };
    let mut merged = match first {
        PropsSource::Static(props) => props,
        PropsSource::Getter(getter) => getter(&RenderElementProps::default()),
    };
    for bag in bags {
        merged = resolve_source(bag, merged);
    }
    merged
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use reactive_graph::owner::Owner;
    use serde_json::Value;

    use super::*;
    use crate::use_render_element::static_attr;
    use leptos_ui_utils::use_merged_refs::MergedRefCallback;

    type Log = Rc<RefCell<Vec<&'static str>>>;

    fn log() -> Log {
        Rc::new(RefCell::new(Vec::new()))
    }

    fn pushing(log: &Log, entry: &'static str) -> ElementEventHandler<BaseUIEvent<String>> {
        let log = Rc::clone(log);
        Rc::new(move |_| log.borrow_mut().push(entry))
    }

    fn attr_bag(pairs: &[(&str, &str)]) -> RenderElementProps {
        RenderElementProps {
            handlers: RenderElementHandlers {
                attributes: pairs
                    .iter()
                    .map(|(name, value)| (name.to_string(), static_attr(value.to_string())))
                    .collect(),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        }
    }

    fn attribute(bag: &RenderElementProps, name: &str) -> Option<String> {
        bag.handlers
            .attributes
            .iter()
            .find(|(existing, _)| existing == name)
            .and_then(|(_, value)| value())
    }

    // `mergeClassNames` (`mergeProps.ts:276-290`): the later (rightmost) class comes
    // first in the string, single sides pass through, and empty/absent classes keep
    // the other side — behavior.md "Events"-adjacent class matrix
    // (`mergeProps.test.ts:190-235`).
    #[test]
    fn merge_class_names_concatenates_with_the_later_class_first() {
        assert_eq!(
            merge_class_names(Some("internal".to_string()), Some("external".to_string())),
            Some("external internal".to_string()),
            "the later class is prepended"
        );
        assert_eq!(
            merge_class_names(None, Some("external".to_string())),
            Some("external".to_string()),
            "an absent earlier class passes the later through"
        );
        assert_eq!(
            merge_class_names(Some("internal".to_string()), None),
            Some("internal".to_string()),
            "an absent later class keeps the earlier"
        );
        assert_eq!(
            merge_class_names(None, None),
            None,
            "both absent stays absent"
        );
    }

    // `mergeObjects` over style (`mergeProps.ts:166-172`): the later bag's
    // declarations win per property; non-conflicting ones are retained.
    #[test]
    fn merge_styles_resolves_conflicts_in_favor_of_the_later_bag() {
        let merged = merge_styles(
            vec![
                ("padding".to_string(), "10px".to_string()),
                ("color".to_string(), "red".to_string()),
            ],
            vec![("color".to_string(), "blue".to_string())],
        );
        assert_eq!(
            merged,
            vec![
                ("padding".to_string(), "10px".to_string()),
                ("color".to_string(), "blue".to_string()),
            ],
            "the later declaration wins the conflicted property, the other survives"
        );
    }

    // The one-sided and both-absent style matrix (`mergeProps.test.ts:169-188`):
    // exactly one side defined passes that side through (mergeObjects' reference
    // passthrough, `packages/utils/src/mergeObjects.ts:5-10` — ownership moves in the
    // port), both absent stays absent rather than producing an empty record.
    #[test]
    fn merge_styles_passthrough_and_absence_match_the_mergeobjects_matrix() {
        assert_eq!(
            merge_styles(Vec::new(), vec![("color".to_string(), "red".to_string())]),
            vec![("color".to_string(), "red".to_string())],
            "an absent earlier style passes the later through"
        );
        assert_eq!(
            merge_styles(
                vec![("color".to_string(), "red".to_string())],
                Vec::new(),
            ),
            vec![("color".to_string(), "red".to_string())],
            "an absent later style keeps the earlier"
        );
        assert_eq!(
            merge_styles(Vec::new(), Vec::new()),
            Vec::new(),
            "both absent produces no style"
        );
    }

    // `mergeEventHandlers` (`mergeProps.ts:229-249`): the later handler runs first;
    // the earlier handler is skipped once the dispatch is marked — and runs when it
    // is not.
    #[test]
    fn merge_event_handlers_runs_the_later_handler_first_and_gates_the_earlier_on_the_mark() {
        let order: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

        let ours: ElementEventHandler<BaseUIEvent<String>> = {
            let order = Rc::clone(&order);
            Rc::new(move |_| order.borrow_mut().push("ours"))
        };
        let theirs: ElementEventHandler<BaseUIEvent<String>> = {
            let order = Rc::clone(&order);
            Rc::new(move |event| {
                order.borrow_mut().push("theirs");
                if event.inner() == "prevent" {
                    event.prevent_base_ui_handler();
                }
            })
        };

        let merged = merge_event_handlers(Some(ours), Some(theirs));
        let event = BaseUIEvent::new("plain".to_string());
        merged.as_ref().unwrap()(&event);
        assert_eq!(
            *order.borrow(),
            vec!["theirs", "ours"],
            "right-to-left execution without the mark"
        );

        order.borrow_mut().clear();
        let prevented = BaseUIEvent::new("prevent".to_string());
        merged.as_ref().unwrap()(&prevented);
        assert_eq!(
            *order.borrow(),
            vec!["theirs"],
            "the mark set by the later handler skips the earlier one"
        );
    }

    // `mergeEventHandlers` with one side absent (`mergeProps.ts:222-227`): the
    // surviving handler passes through unwrapped.
    #[test]
    fn merge_event_handlers_passes_single_sided_handlers_through() {
        let calls: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let ours: ElementEventHandler<BaseUIEvent<String>> = {
            let calls = Rc::clone(&calls);
            Rc::new(move |_| *calls.borrow_mut() += 1)
        };
        let merged = merge_event_handlers(Some(ours), None);
        merged.as_ref().unwrap()(&BaseUIEvent::new(String::new()));
        assert_eq!(*calls.borrow(), 1, "the earlier handler alone still runs");
    }

    // `undefined` handler entries are skipped without breaking the chain
    // (`mergeProps.test.ts:55-76`): a `None` middle slot folds to the surviving
    // side, so the outer chain runs C then A.
    #[test]
    fn merge_event_handlers_skips_undefined_middle_entries() {
        let log = log();
        let a = pushing(&log, "3");
        let c = pushing(&log, "1");

        let with_none = merge_event_handlers(Some(a), None);
        let merged = merge_event_handlers(with_none, Some(c));
        merged.as_ref().unwrap()(&BaseUIEvent::new(String::new()));
        assert_eq!(*log.borrow(), vec!["1", "3"], "the None slot never breaks the chain");
    }

    // `mergeProps(A, B, C)` handler order (`mergeProps.test.ts:30-53`): the chain is
    // nested closures — the last argument's handler runs first, log `['1', '2', '3']`.
    #[test]
    fn merge_event_handlers_three_deep_runs_last_argument_first() {
        let log = log();
        let a = pushing(&log, "3");
        let b = pushing(&log, "2");
        let c = pushing(&log, "1");

        let merged = merge_event_handlers(merge_event_handlers(Some(a), Some(b)), Some(c));
        merged.as_ref().unwrap()(&BaseUIEvent::new(String::new()));
        assert_eq!(*log.borrow(), vec!["1", "2", "3"]);
    }

    // The mid-chain veto (`mergeProps.test.ts:283-308`): with the call in the middle
    // handler, execution is `['0', '1']` and the first argument's handler is skipped.
    #[test]
    fn merge_event_handlers_mid_chain_veto_skips_only_the_earlier_side() {
        let log = log();
        let a = pushing(&log, "2");
        let c = pushing(&log, "0");
        let b: ElementEventHandler<BaseUIEvent<String>> = {
            let log = Rc::clone(&log);
            Rc::new(move |event| {
                event.prevent_base_ui_handler();
                log.borrow_mut().push("1");
            })
        };

        let merged = merge_event_handlers(merge_event_handlers(Some(a), Some(b)), Some(c));
        merged.as_ref().unwrap()(&BaseUIEvent::new(String::new()));
        assert_eq!(*log.borrow(), vec!["0", "1"]);
    }

    // Without the mark, all chained handlers run (`mergeProps.test.ts:237-254`).
    #[test]
    fn merge_event_handlers_runs_every_handler_without_the_mark() {
        let ran: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let ours: ElementEventHandler<BaseUIEvent<String>> = {
            let ran = Rc::clone(&ran);
            Rc::new(move |_| *ran.borrow_mut() = true)
        };
        let theirs: ElementEventHandler<BaseUIEvent<String>> = Rc::new(|_| {});
        let merged = merge_event_handlers(Some(ours), Some(theirs));
        merged.as_ref().unwrap()(&BaseUIEvent::new(String::new()));
        assert!(*ran.borrow(), "the earlier handler ran when nothing prevented it");
    }

    // `preventBaseUIHandler()` marks the dispatch readably in the same handler
    // (`mergeProps.test.ts:78-94`, `:414-432` — the lone and flag-observation tests).
    #[test]
    fn the_mark_is_readable_immediately_after_the_call() {
        let observed: Rc<RefCell<Option<bool>>> = Rc::new(RefCell::new(None));
        let handler: ElementEventHandler<BaseUIEvent<String>> = {
            let observed = Rc::clone(&observed);
            Rc::new(move |event| {
                event.prevent_base_ui_handler();
                *observed.borrow_mut() = Some(event.base_ui_handler_prevented());
            })
        };
        let merged = merge_event_handlers(None, Some(handler));
        merged.as_ref().unwrap()(&BaseUIEvent::new(String::new()));
        assert_eq!(*observed.borrow(), Some(true));
    }

    // Non-standard (non-DOM) handlers are chained the same way and never error,
    // whatever the invocation payload (`mergeProps.test.ts:310-331`), and a lone
    // handler receives its payload verbatim (`:333-347`). The port's payload is the
    // typed event value; the duck-typed synthetic/non-synthetic branch collapses into
    // the type (module docs), so prevention is simply never signaled here.
    #[test]
    fn non_standard_payloads_forward_verbatim_and_all_handlers_run() {
        let payloads = vec![
            Value::Bool(true),
            Value::from(13),
            Value::from("newValue"),
            serde_json::json!({ "key": "value" }),
            serde_json::json!(["value"]),
        ];

        for payload in payloads {
            let log = log();
            let a: ElementEventHandler<BaseUIEvent<Value>> = {
                let log = Rc::clone(&log);
                Rc::new(move |_| log.borrow_mut().push("1"))
            };
            let b: ElementEventHandler<BaseUIEvent<Value>> = {
                let log = Rc::clone(&log);
                Rc::new(move |_| log.borrow_mut().push("0"))
            };
            let merged = merge_event_handlers(Some(a), Some(b));
            merged.as_ref().unwrap()(&BaseUIEvent::new(payload.clone()));
            assert_eq!(
                *log.borrow(),
                vec!["0", "1"],
                "both handlers ran right-first for {payload}"
            );

            let lone_calls: Rc<RefCell<Option<Value>>> = Rc::new(RefCell::new(None));
            let lone: ElementEventHandler<BaseUIEvent<Value>> = {
                let lone_calls = Rc::clone(&lone_calls);
                Rc::new(move |event| *lone_calls.borrow_mut() = Some(event.inner().clone()))
            };
            let lone_merged = merge_event_handlers(None, Some(lone));
            lone_merged.as_ref().unwrap()(&BaseUIEvent::new(payload.clone()));
            assert_eq!(
                lone_calls.borrow().as_ref(),
                Some(&payload),
                "the lone handler received the payload verbatim"
            );
        }
    }

    // `mergePropsN`'s plain-attribute precedence (`mergeProps.ts:14-15`) and the
    // ref rule (`:33` — "ref is not merged"; a later bag replaces only when it
    // carries one).
    #[test]
    fn merge_props_n_later_bags_win_attributes_and_only_replace_carried_refs() {
        let _owner = Owner::new();
        _owner.set();

        let early_ref: MergedRefCallback<web_sys::Element> = Rc::new(|_: Option<&web_sys::Element>| {});
        let late_ref: MergedRefCallback<web_sys::Element> = Rc::new(|_: Option<&web_sys::Element>| {});
        let late = RenderElementProps {
            handlers: RenderElementHandlers {
                attributes: vec![("data-late".to_string(), static_attr("late".to_string()))],
                ..RenderElementHandlers::default()
            },
            class: Some("late".to_string()),
            style: vec![("color".to_string(), "blue".to_string())],
            inner_html: None,
            ref_callback: Some(Rc::clone(&late_ref)),
        };
        let early = RenderElementProps {
            handlers: RenderElementHandlers {
                attributes: vec![
                    ("data-early".to_string(), static_attr("early".to_string())),
                    (
                        "data-late".to_string(),
                        static_attr("superseded".to_string()),
                    ),
                ],
                ..RenderElementHandlers::default()
            },
            class: Some("early".to_string()),
            style: vec![("color".to_string(), "red".to_string())],
            inner_html: None,
            ref_callback: Some(early_ref),
        };

        let merged = merge_props_n(vec![PropsSource::Static(early), PropsSource::Static(late)]);
        assert_eq!(
            merged.class,
            Some("late early".to_string()),
            "class concatenates later-first"
        );
        assert_eq!(
            merged.style,
            vec![("color".to_string(), "blue".to_string())],
            "the later style wins per key"
        );
        let late_value = merged
            .handlers
            .attributes
            .iter()
            .find(|(name, _)| name == "data-late")
            .and_then(|(_, value)| value())
            .expect("the later bag's attribute wins");
        assert_eq!(
            late_value, "late",
            "the later bag's attribute replaces the earlier's"
        );
        assert!(
            merged
                .handlers
                .attributes
                .iter()
                .any(|(name, _)| name == "data-early"),
            "the earlier bag's non-conflicted attribute survives"
        );
        assert!(
            Rc::ptr_eq(merged.ref_callback.as_ref().unwrap(), &late_ref),
            "the last ref-carrying bag's ref wins"
        );
    }

    // Scalar (plain-prop) precedence across three bags — the `title` test
    // (`mergeProps.test.ts:400-412`): the LAST argument's value wins outright.
    #[test]
    fn merge_props_n_scalars_follow_last_argument_wins() {
        let merged = merge_props_n(vec![
            PropsSource::Static(attr_bag(&[("title", "internal title 2")])),
            PropsSource::Static(attr_bag(&[("title", "internal title 1")])),
            PropsSource::Static(attr_bag(&[])),
        ]);
        assert_eq!(
            attribute(&merged, "title").as_deref(),
            Some("internal title 1"),
            "the rightmost scalar wins"
        );
    }

    // Multiple classNames fold later-first through the whole bag list
    // (`mergeProps.test.ts:202-216`), and bags with no class at all leave the merged
    // class absent (`:229-235`).
    #[test]
    fn merge_props_n_concatenates_multiple_class_names_later_first() {
        let mut first = attr_bag(&[]);
        first.class = Some("class-1".to_string());
        let mut second = attr_bag(&[]);
        second.class = Some("class-2".to_string());
        let mut third = attr_bag(&[]);
        third.class = Some("class-3".to_string());

        let merged = merge_props_n(vec![
            PropsSource::Static(first),
            PropsSource::Static(second),
            PropsSource::Static(third),
        ]);
        assert_eq!(merged.class.as_deref(), Some("class-3 class-2 class-1"));

        let merged = merge_props_n(vec![
            PropsSource::Static(attr_bag(&[])),
            PropsSource::Static(attr_bag(&[])),
        ]);
        assert_eq!(
            merged.class, None,
            "no bag carrying a class leaves the merged class absent"
        );
    }

    // The `inner_html` member (the `dangerouslySetInnerHTML` analog,
    // `PrehydrationScript.tsx:45`) merges as a plain prop key: a later bag that
    // carries it wins, a later bag without it never overwrites, and carrying the
    // member keeps the bag non-empty.
    #[test]
    fn inner_html_merges_as_a_plain_prop_key() {
        let earlier = RenderElementProps {
            inner_html: Some("earlier".to_string()),
            ..RenderElementProps::default()
        };
        let carried = RenderElementProps {
            inner_html: Some("later".to_string()),
            ..RenderElementProps::default()
        };

        let merged = merge_props_n(vec![
            PropsSource::Static(earlier),
            PropsSource::Static(carried),
        ]);
        assert_eq!(
            merged.inner_html.as_deref(),
            Some("later"),
            "the later bag's body replaces the earlier's"
        );

        let merged = merge_props_n(vec![
            PropsSource::Static(RenderElementProps {
                inner_html: Some("kept".to_string()),
                ..RenderElementProps::default()
            }),
            PropsSource::Static(RenderElementProps::default()),
        ]);
        assert_eq!(
            merged.inner_html.as_deref(),
            Some("kept"),
            "a later bag that does not carry the member never overwrites"
        );

        assert!(
            !RenderElementProps {
                inner_html: Some(String::new()),
                ..RenderElementProps::default()
            }
            .is_empty(),
            "a bag carrying the member is not empty"
        );
        assert!(
            RenderElementProps::default().is_empty(),
            "the default bag is still empty"
        );
    }

    // The getter form (`mergeProps.ts:210-219`): resolved wholesale against the
    // merged-so-far props and replacing them.
    #[test]
    fn getter_bags_replace_wholesale() {
        let _owner = Owner::new();
        _owner.set();

        let getter_calls: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let getter: RenderPropsGetter = {
            let calls = Rc::clone(&getter_calls);
            Rc::new(move |previous: &RenderElementProps| {
                *calls.borrow_mut() += 1;
                assert_eq!(
                    previous.class.as_deref(),
                    Some("base"),
                    "the getter receives the merged-so-far props"
                );
                RenderElementProps {
                    class: Some("from-getter".to_string()),
                    ..RenderElementProps::default()
                }
            })
        };

        let base = RenderElementProps {
            class: Some("base".to_string()),
            ..RenderElementProps::default()
        };
        let merged = merge_props_n(vec![PropsSource::Static(base), PropsSource::Getter(getter)]);
        assert_eq!(
            merged.class,
            Some("from-getter".to_string()),
            "the getter's props replace the accumulated ones wholesale"
        );
        assert_eq!(
            *getter_calls.borrow(),
            1,
            "the getter resolved exactly once"
        );
    }

    // A getter receives the accumulated merge of all preceding props-object
    // arguments (`mergeProps.test.ts:435-456` — the upstream test's observed props
    // are the first argument's bag) and exactly once per invocation.
    #[test]
    fn getter_receives_the_accumulated_preceding_props() {
        let observed: Rc<RefCell<Option<RenderElementProps>>> = Rc::new(RefCell::new(None));
        let getter: RenderPropsGetter = {
            let observed = Rc::clone(&observed);
            Rc::new(move |previous: &RenderElementProps| {
                *observed.borrow_mut() = Some(previous.clone());
                previous.clone()
            })
        };

        let mut first = attr_bag(&[]);
        first.class = Some("test-class".to_string());
        merge_props_n(vec![
            PropsSource::Static(attr_bag(&[("id", "2")])),
            PropsSource::Static(first),
            PropsSource::Getter(getter),
            PropsSource::Static(attr_bag(&[("id", "1"), ("role", "button")])),
        ]);

        let seen = observed.borrow().clone().expect("the getter ran");
        assert_eq!(
            attribute(&seen, "id").as_deref(),
            Some("2"),
            "the getter saw the preceding bags' merge"
        );
        assert_eq!(seen.class.as_deref(), Some("test-class"));
        assert_eq!(
            attribute(&seen, "role"),
            None,
            "the getter never saw the bags that follow it"
        );
    }

    // A getter positioned after several bags receives their merged result
    // (`mergeProps.test.ts:458-484`): the later bag's `role: 'tab'` overwrote the
    // earlier `role: 'button'` before the getter ran.
    #[test]
    fn getter_receives_the_merged_so_far_props() {
        let observed: Rc<RefCell<Option<RenderElementProps>>> = Rc::new(RefCell::new(None));
        let getter: RenderPropsGetter = {
            let observed = Rc::clone(&observed);
            Rc::new(move |previous: &RenderElementProps| {
                *observed.borrow_mut() = Some(previous.clone());
                previous.clone()
            })
        };

        merge_props_n(vec![
            PropsSource::Static(attr_bag(&[("role", "button")])),
            PropsSource::Static(attr_bag(&[("role", "tab")])),
            PropsSource::Getter(getter),
            PropsSource::Static(attr_bag(&[("id", "one")])),
        ]);

        let seen = observed.borrow().clone().expect("the getter ran");
        assert_eq!(
            attribute(&seen, "role").as_deref(),
            Some("tab"),
            "the getter saw the bags merged up to its position"
        );
        assert_eq!(attribute(&seen, "id"), None);
    }

    // A getter with no preceding props-object arguments receives an empty bag — not
    // the props that follow it (`mergeProps.test.ts:486-497`); upstream hands it the
    // shared `EMPTY_PROPS`, the port a fresh default (module docs).
    #[test]
    fn getter_with_no_preceding_props_receives_the_empty_bag() {
        let observed: Rc<RefCell<Option<RenderElementProps>>> = Rc::new(RefCell::new(None));
        let getter: RenderPropsGetter = {
            let observed = Rc::clone(&observed);
            Rc::new(move |previous: &RenderElementProps| {
                *observed.borrow_mut() = Some(previous.clone());
                previous.clone()
            })
        };

        let merged = merge_props_n(vec![
            PropsSource::Getter(getter),
            PropsSource::Static(attr_bag(&[("id", "1")])),
        ]);

        let seen = observed.borrow().clone().expect("the getter ran");
        assert!(
            seen.is_empty(),
            "the leading getter received an empty bag"
        );
        assert_eq!(
            attribute(&merged, "id").as_deref(),
            Some("1"),
            "the following bag still merged onto the getter's result"
        );
    }

    // A getter positioned last REPLACES the accumulated props with its return value
    // — earlier props do not survive (`mergeProps.test.ts:514-530`).
    #[test]
    fn trailing_getter_replaces_the_accumulator_wholesale() {
        let getter: RenderPropsGetter = {
            Rc::new(|_: &RenderElementProps| RenderElementProps {
                class: Some("test-class".to_string()),
                ..RenderElementProps::default()
            })
        };

        let merged = merge_props_n(vec![
            PropsSource::Static(attr_bag(&[("id", "two"), ("role", "tab")])),
            PropsSource::Static(attr_bag(&[("id", "one")])),
            PropsSource::Getter(getter),
        ]);

        assert_eq!(merged.class.as_deref(), Some("test-class"));
        assert_eq!(attribute(&merged, "id"), None, "earlier props did not survive");
        assert_eq!(attribute(&merged, "role"), None);
    }

    // A getter's returned object is never mutated by subsequent merging
    // (`mergeProps.test.ts:499-512`): the leading-getter defensive copy
    // (`mergeProps.ts:117-124`) is structural in Rust — the getter's template lives
    // behind a shared cell and every call returns a fresh owned clone, so the fold
    // mutates only its own copy.
    #[test]
    fn does_not_mutate_a_reused_object_returned_by_the_first_props_getter() {
        let shared: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(Some("base".to_string())));
        let getter: RenderPropsGetter = {
            let shared = Rc::clone(&shared);
            Rc::new(move |_: &RenderElementProps| RenderElementProps {
                class: shared.borrow().clone(),
                ..RenderElementProps::default()
            })
        };

        let mut next = attr_bag(&[]);
        next.class = Some("next".to_string());
        let result = merge_props_n(vec![PropsSource::Getter(getter), PropsSource::Static(next)]);

        assert_eq!(result.class.as_deref(), Some("next base"));
        assert_eq!(
            shared.borrow().as_deref(),
            Some("base"),
            "the shared template kept its original value"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::MouseEvent;

    use super::*;
    use crate::use_render_element::static_attr;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    type ClickHandler = ElementEventHandler<BaseUIEvent<MouseEvent>>;
    type Log = Rc<RefCell<Vec<&'static str>>>;

    fn mouse_event(name: &str) -> MouseEvent {
        MouseEvent::new(name).expect("MouseEvent::new")
    }

    fn log() -> Log {
        Rc::new(RefCell::new(Vec::new()))
    }

    fn click_pushing(log: &Log, entry: &'static str) -> ClickHandler {
        let log = Rc::clone(log);
        Rc::new(move |_| log.borrow_mut().push(entry))
    }

    fn click_handler(
        run: impl Fn(&BaseUIEvent<MouseEvent>) + 'static,
    ) -> ClickHandler {
        Rc::new(run)
    }

    fn dispatch_click(slot: &Option<ClickHandler>) {
        let event = BaseUIEvent::new(mouse_event("click"));
        slot.as_ref().expect("the merged slot is filled")(&event);
    }

    fn dispatch_mouse_down(slot: &Option<ClickHandler>) {
        let event = BaseUIEvent::new(mouse_event("mousedown"));
        slot.as_ref().expect("the merged slot is filled")(&event);
    }

    fn dispatch_context_menu(slot: &Option<ClickHandler>) {
        let event = BaseUIEvent::new(mouse_event("contextmenu"));
        slot.as_ref().expect("the merged slot is filled")(&event);
    }

    // A lone bag handler is preventable — the merged slot receives a
    // [`BaseUIEvent`]-wrapped dispatch, so `preventBaseUIHandler()` exists and the
    // flag reads back in the same handler (`mergeProps.test.ts:78-94`; the
    // `wrapEventHandler` lone-handler wrapping is the slot type itself, module
    // docs).
    #[wasm_bindgen_test]
    fn lone_bag_handler_is_preventable() {
        let observed: Rc<RefCell<Option<bool>>> = Rc::new(RefCell::new(None));
        let mut bag = RenderElementProps::default();
        bag.handlers.on_mouse_down = Some(click_handler({
            let observed = Rc::clone(&observed);
            move |event| {
                event.prevent_base_ui_handler();
                *observed.borrow_mut() = Some(event.base_ui_handler_prevented());
            }
        }));

        let merged = merge_props_n(vec![
            PropsSource::Static(RenderElementProps::default()),
            PropsSource::Static(bag),
        ]);
        dispatch_mouse_down(&merged.handlers.on_mouse_down);
        assert_eq!(*observed.borrow(), Some(true));
    }

    // A first-position handler is preventable even when the other argument only
    // sets a non-handler prop (`mergeProps.test.ts:96-114`) — the same call through
    // the array form (`:116-134`) is the port's single entrypoint.
    #[wasm_bindgen_test]
    fn first_position_bag_handler_is_preventable() {
        let observed: Rc<RefCell<Option<bool>>> = Rc::new(RefCell::new(None));
        let mut first = RenderElementProps::default();
        first.handlers.on_mouse_down = Some(click_handler({
            let observed = Rc::clone(&observed);
            move |event| {
                event.prevent_base_ui_handler();
                *observed.borrow_mut() = Some(event.base_ui_handler_prevented());
            }
        }));
        let second = RenderElementProps {
            handlers: RenderElementHandlers {
                attributes: vec![("id".to_string(), static_attr("test-button".to_string()))],
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };

        let merged = merge_props_n(vec![
            PropsSource::Static(first),
            PropsSource::Static(second),
        ]);
        dispatch_mouse_down(&merged.handlers.on_mouse_down);
        assert_eq!(*observed.borrow(), Some(true));
        assert_eq!(
            merged
                .handlers
                .attributes
                .iter()
                .find(|(name, _)| name == "id")
                .and_then(|(_, value)| value())
                .as_deref(),
            Some("test-button"),
            "the non-handler prop merged through"
        );
    }

    // The "obscure" handler key (`onContextMenu`) is preventable the same way
    // (`mergeProps.test.ts:136-152`).
    #[wasm_bindgen_test]
    fn obscure_context_menu_handler_is_preventable() {
        let observed: Rc<RefCell<Option<bool>>> = Rc::new(RefCell::new(None));
        let mut bag = RenderElementProps::default();
        bag.handlers.on_context_menu = Some(click_handler({
            let observed = Rc::clone(&observed);
            move |event| {
                event.prevent_base_ui_handler();
                *observed.borrow_mut() = Some(event.base_ui_handler_prevented());
            }
        }));

        let merged = merge_props_n(vec![
            PropsSource::Static(RenderElementProps::default()),
            PropsSource::Static(bag),
        ]);
        dispatch_context_menu(&merged.handlers.on_context_menu);
        assert_eq!(*observed.borrow(), Some(true));
    }

    // Different keys across two bags each run exactly once, in right-to-left order
    // for the shared key (`mergeProps.test.ts:6-28`).
    #[wasm_bindgen_test]
    fn cross_key_handlers_each_run_once_in_right_to_left_order() {
        let order: Log = log();
        let theirs_click = click_pushing(&order, "theirs");
        let ours_click = click_pushing(&order, "ours");
        let key_down_calls: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let mouse_move_calls: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));

        let theirs = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some(theirs_click),
                on_key_down: Some({
                    let calls = Rc::clone(&key_down_calls);
                    Rc::new(move |_: &BaseUIEvent<web_sys::KeyboardEvent>| {
                        *calls.borrow_mut() += 1
                    })
                }),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };
        let ours = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some(ours_click),
                on_mouse_move: Some({
                    let calls = Rc::clone(&mouse_move_calls);
                    Rc::new(move |_: &BaseUIEvent<MouseEvent>| *calls.borrow_mut() += 1)
                }),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };

        let merged = merge_props_n(vec![PropsSource::Static(ours), PropsSource::Static(theirs)]);
        dispatch_click(&merged.handlers.on_click);
        assert_eq!(*order.borrow(), vec!["theirs", "ours"]);

        let key_event = BaseUIEvent::new(
            web_sys::KeyboardEvent::new("keydown").expect("KeyboardEvent::new"),
        );
        merged
            .handlers
            .on_key_down
            .as_ref()
            .expect("the key slot survived")(&key_event);
        assert_eq!(*key_down_calls.borrow(), 1, "the unshared key slot survived");

        let move_event = BaseUIEvent::new(mouse_event("mousemove"));
        merged
            .handlers
            .on_mouse_move
            .as_ref()
            .expect("the move slot survived")(&move_event);
        assert_eq!(*mouse_move_calls.borrow(), 1, "the other unshared slot survived");
    }

    // `mergeProps(A, B, C)` over real bags: the last argument's handler runs first
    // (`mergeProps.test.ts:30-53`, log `['1', '2', '3']`).
    #[wasm_bindgen_test]
    fn three_bag_handler_order_runs_last_argument_first() {
        let log = log();
        let merged = merge_props_n(vec![
            PropsSource::Static(RenderElementProps {
                handlers: RenderElementHandlers {
                    on_click: Some(click_pushing(&log, "3")),
                    ..RenderElementHandlers::default()
                },
                ..RenderElementProps::default()
            }),
            PropsSource::Static(RenderElementProps {
                handlers: RenderElementHandlers {
                    on_click: Some(click_pushing(&log, "2")),
                    ..RenderElementHandlers::default()
                },
                ..RenderElementProps::default()
            }),
            PropsSource::Static(RenderElementProps {
                handlers: RenderElementHandlers {
                    on_click: Some(click_pushing(&log, "1")),
                    ..RenderElementHandlers::default()
                },
                ..RenderElementProps::default()
            }),
        ]);

        dispatch_click(&merged.handlers.on_click);
        assert_eq!(*log.borrow(), vec!["1", "2", "3"]);
    }

    // A `None` middle slot never breaks the chain through real bags
    // (`mergeProps.test.ts:55-76`, log `['1', '3']`).
    #[wasm_bindgen_test]
    fn undefined_middle_slot_is_skipped_through_bags() {
        let log = log();
        let merged = merge_props_n(vec![
            PropsSource::Static(RenderElementProps {
                handlers: RenderElementHandlers {
                    on_click: Some(click_pushing(&log, "3")),
                    ..RenderElementHandlers::default()
                },
                ..RenderElementProps::default()
            }),
            PropsSource::Static(RenderElementProps::default()),
            PropsSource::Static(RenderElementProps {
                handlers: RenderElementHandlers {
                    on_click: Some(click_pushing(&log, "1")),
                    ..RenderElementHandlers::default()
                },
                ..RenderElementProps::default()
            }),
        ]);

        dispatch_click(&merged.handlers.on_click);
        assert_eq!(*log.borrow(), vec!["1", "3"]);
    }

    // Without the mark the earlier bag's handler still runs
    // (`mergeProps.test.ts:237-254`).
    #[wasm_bindgen_test]
    fn unmarked_dispatch_runs_every_handler() {
        let ran: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let first = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some({
                    let ran = Rc::clone(&ran);
                    Rc::new(move |_: &BaseUIEvent<MouseEvent>| *ran.borrow_mut() = true)
                }),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };
        let second = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some(Rc::new(|_: &BaseUIEvent<MouseEvent>| {})),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };

        let merged = merge_props_n(vec![
            PropsSource::Static(first),
            PropsSource::Static(second),
        ]);
        dispatch_click(&merged.handlers.on_click);
        assert!(*ran.borrow(), "the earlier handler ran when nothing prevented it");
    }

    // With the last bag's handler calling `preventBaseUIHandler()`, every earlier
    // handler is skipped (`mergeProps.test.ts:256-281`, `ran` stays `false`).
    #[wasm_bindgen_test]
    fn last_bag_veto_skips_every_earlier_handler() {
        let ran: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let first = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some({
                    let ran = Rc::clone(&ran);
                    Rc::new(move |_: &BaseUIEvent<MouseEvent>| *ran.borrow_mut() = true)
                }),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };
        let second = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some(Rc::new(|_: &BaseUIEvent<MouseEvent>| {})),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };
        let third = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some(Rc::new(|event: &BaseUIEvent<MouseEvent>| {
                    event.prevent_base_ui_handler();
                })),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };

        let merged = merge_props_n(vec![
            PropsSource::Static(first),
            PropsSource::Static(second),
            PropsSource::Static(third),
        ]);
        dispatch_click(&merged.handlers.on_click);
        assert!(!*ran.borrow(), "the veto skipped every earlier handler");
    }

    // A getter handler that calls `preventBaseUIHandler()` and then MANUALLY invokes
    // the captured earlier handler with a fresh event runs it anyway — the manual
    // call bypasses the automatic prevention of the merged chain
    // (`mergeProps.test.ts:532-563`, log `['last-handler', 'getter-handler',
    // 'first-handler']`).
    #[wasm_bindgen_test]
    fn getter_handler_manual_reinvocation_bypasses_prevention() {
        let log = log();
        let first = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some(click_pushing(&log, "first-handler")),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };
        let getter: RenderPropsGetter = {
            let log = Rc::clone(&log);
            Rc::new(move |previous: &RenderElementProps| {
                let captured = previous.handlers.on_click.clone();
                let getter_log = Rc::clone(&log);
                RenderElementProps {
                    handlers: RenderElementHandlers {
                        on_click: Some(click_handler(move |event| {
                            event.prevent_base_ui_handler();
                            getter_log.borrow_mut().push("getter-handler");
                            // Manually calling the previous handler — this bypasses
                            // automatic prevention.
                            if let Some(captured) = &captured {
                                captured(&BaseUIEvent::new(mouse_event("click")));
                            }
                        })),
                        ..RenderElementHandlers::default()
                    },
                    ..RenderElementProps::default()
                }
            })
        };
        let last = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some(click_pushing(&log, "last-handler")),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };

        let merged = merge_props_n(vec![
            PropsSource::Static(first),
            PropsSource::Getter(getter),
            PropsSource::Static(last),
        ]);
        dispatch_click(&merged.handlers.on_click);
        assert_eq!(
            *log.borrow(),
            vec!["last-handler", "getter-handler", "first-handler"]
        );
    }

    // The `baseUIHandlerPrevented` flag lets a getter handler opt into respecting
    // prevention before manually re-invoking captured handlers
    // (`mergeProps.test.ts:565-597`, log `['last-handler', 'getter-handler']`).
    #[wasm_bindgen_test]
    fn getter_handler_can_respect_the_mark_manually() {
        let log = log();
        let first = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some(click_pushing(&log, "first-handler")),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };
        let getter: RenderPropsGetter = {
            let log = Rc::clone(&log);
            Rc::new(move |previous: &RenderElementProps| {
                let captured = previous.handlers.on_click.clone();
                let getter_log = Rc::clone(&log);
                RenderElementProps {
                    handlers: RenderElementHandlers {
                        on_click: Some(click_handler(move |event| {
                            event.prevent_base_ui_handler();
                            getter_log.borrow_mut().push("getter-handler");
                            // Check the flag before manually calling previous
                            // handlers — this respects prevention.
                            if !event.base_ui_handler_prevented() {
                                if let Some(captured) = &captured {
                                    captured(&BaseUIEvent::new(mouse_event("click")));
                                }
                            }
                        })),
                        ..RenderElementHandlers::default()
                    },
                    ..RenderElementProps::default()
                }
            })
        };
        let last = RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some(click_pushing(&log, "last-handler")),
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        };

        let merged = merge_props_n(vec![
            PropsSource::Static(first),
            PropsSource::Getter(getter),
            PropsSource::Static(last),
        ]);
        dispatch_click(&merged.handlers.on_click);
        assert_eq!(*log.borrow(), vec!["last-handler", "getter-handler"]);
    }
}
