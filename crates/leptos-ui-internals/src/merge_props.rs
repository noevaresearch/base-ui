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
