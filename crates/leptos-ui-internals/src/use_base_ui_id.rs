//! Port of `packages/react/src/internals/useBaseUiId.ts` — the `useId` wrapper that prefixes
//! generated ids with `base-ui-` (`useBaseUiId.ts:9-11`:
//! `useId(idOverride, 'base-ui')`), the id source behind the labelable-provider's
//! `labelId`/`descriptionId` wiring (`specs/library/internals/implementation.md`,
//! "Cross-references inside this unit": `LabelableProvider`/`useLabelableId`/`useLabel`/
//! `useAriaLabelledBy` → `LabelableContext` + `useBaseUiId`).
//!
//! Upstream has no test file for it (`specs/library/internals/implementation.md`,
//! "Anything in source not explained by any test" item 3), so the tests below pin the
//! wrapper's own contract — the prefix application and the override passthrough — over the
//! already-ported [`leptos_ui_utils::use_id`] semantics, whose prefix path the upstream
//! suite exercises directly (`packages/utils/src/useId.test.tsx:102-123`, mirrored in that
//! port's `can_be_prefixed`).
//!
//! Rust adaptation: the port's `useId` signature is
//! `(id_override, prefix: Option<&str>) -> Signal<String>` (see that port's module docs for
//! the React→Leptos timing/reactivity mapping), so the wrapper is the same call with the
//! prefix baked in. The upstream return's `undefined` arm (`useBaseUiId.ts:7-9`) is covered
//! by the same adaptation the `use_id` port records: the supported path always yields a
//! string.

use reactive_graph::owner::LocalStorage;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::Signal;

use leptos_ui_utils::use_id;

/// The prefix every non-overridden id gets (`packages/react/src/internals/useBaseUiId.ts:10`).
const BASE_UI_PREFIX: &str = "base-ui";

/// Port of `useBaseUiId` (`packages/react/src/internals/useBaseUiId.ts:9-11`): the provided
/// override wins verbatim; otherwise the generated id is `` `base-ui-{n}` ``. Must be called
/// inside a reactive owner (a component) — the same contract as [`leptos_ui_utils::use_id`].
pub fn use_base_ui_id<C>(id_override: C) -> Signal<String, LocalStorage>
where
    C: Get<Value = Option<String>> + 'static,
{
    use_id(id_override, Some(BASE_UI_PREFIX))
}

#[cfg(test)]
mod tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};

    use super::*;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Pins the JSDoc contract's override arm (`useBaseUiId.ts:6`): a provided id overrides
    // the generated one, verbatim — no prefixing applied to an explicit id.
    #[test]
    fn the_override_wins_verbatim_without_the_prefix() {
        let owner = in_owner();

        let id = use_base_ui_id(RwSignal::new(Some("my-id".to_string())));

        assert_eq!(id.get_untracked(), "my-id");

        owner.cleanup();
    }

    // Pins the JSDoc contract's generated arm (`useBaseUiId.ts:4-5`): with no override the
    // id is generated and prefixed with `base-ui-` — the wrapper is the `'base-ui'` prefix
    // call site the `use_id` port's prefix tests cite
    // (`packages/react/src/internals/useBaseUiId.ts:10`).
    #[test]
    fn the_generated_id_is_prefixed_with_base_ui() {
        let owner = in_owner();

        let id = use_base_ui_id(RwSignal::new(None));

        let value = id.get_untracked();
        assert!(value.starts_with("base-ui-"), "unprefixed id: {value:?}");
        assert!(
            value.len() > "base-ui-".len(),
            "a counter is appended: {value:?}"
        );

        owner.cleanup();
    }

    // Pins the reactive override transition (the `use_id` port's override semantics,
    // exercised through the wrapper): dropping the override reverts to the prefixed
    // generated id, and a later override takes over.
    #[test]
    fn the_override_tracks_reactively_into_and_out_of_the_generated_id() {
        let owner = in_owner();

        let override_source: RwSignal<Option<String>> = RwSignal::new(None);
        let id = use_base_ui_id(override_source);

        let generated = id.get_untracked();
        assert!(generated.starts_with("base-ui-"));

        override_source.set(Some("explicit".to_string()));
        assert_eq!(id.get_untracked(), "explicit");

        override_source.set(None);
        assert_eq!(
            id.get_untracked(),
            generated,
            "the generated core is stable"
        );

        owner.cleanup();
    }
}
