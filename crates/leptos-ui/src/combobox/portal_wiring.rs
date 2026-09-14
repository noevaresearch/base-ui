//! Combobox.Portal / ComboboxPortalContext — the parts' DOM wiring over the
//! store spine (`packages/react/src/combobox/portal/ComboboxPortal.tsx`,
//! `ComboboxPortalContext.tsx`).
//!
//! The part is the render gate: it renders the floating portal (the already-
//! ported [`provide_floating_portal`] machinery) only while the popup subtree
//! should exist — `mounted || keepMounted || forceMounted`
//! (`ComboboxPortal.tsx:20-25`) — and publishes the `keepMounted` flag through
//! the portal context the positioner consumes (`ComboboxPortalContext.tsx:5-19`).

use crate::combobox::store::selectors;
use crate::combobox::store::{ComboboxState, ComboboxStore};

/// The render gate (`ComboboxPortal.tsx:20-25`): `mounted || keepMounted ||
/// forceMounted` — `false` renders nothing (upstream `return null`).
pub fn should_render_portal(mounted: bool, keep_mounted: bool, force_mounted: bool) -> bool {
    mounted || keep_mounted || force_mounted
}

/// The gate folded over the store (`ComboboxPortal.tsx:22-24`): the two store
/// reads (`mounted`, `forceMounted`) plus the part's own `keepMounted` prop
/// (default `false`, `:14`).
pub fn portal_should_render(store: &ComboboxStore, keep_mounted: bool) -> bool {
    let state = store.get_snapshot();
    should_render_portal(
        selectors::mounted(&state),
        keep_mounted,
        selectors::force_mounted(&state),
    )
}

/// The `ComboboxPortalContext` value (`ComboboxPortalContext.tsx:5`) — the
/// `keepMounted` flag the provider publishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboboxPortalContextValue {
    /// The `keepMounted` prop (`ComboboxPortal.tsx:20-21`) — whether the popup
    /// subtree stays mounted while hidden.
    pub keep_mounted: bool,
}

/// The context access contract (`ComboboxPortalContext.tsx:11-18`):
/// `useComboboxPortalContext` throws when the part renders outside a
/// `<Combobox.Portal>` — the positioner's `useComboboxPortalContext()` is the
/// sole (non-optional) consumer. The port's accessors take the value behind an
/// `Option` (the popover `expect()` precedent) and the caller decides whether
/// absence throws.
pub fn use_portal_context(
    context: Option<ComboboxPortalContextValue>,
    optional: bool,
) -> Option<ComboboxPortalContextValue> {
    match context {
        Some(value) => Some(value),
        None if optional => None,
        None => panic!(
            "Base UI: <Combobox.Portal> is missing. \
<Combobox.Positioner> must be placed within <Combobox.Portal>."
        ),
    }
}

/// The part's effective props (`ComboboxPortal.tsx:14-21`): `keepMounted`
/// defaults to `false`; everything else is element props forwarded onto the
/// portal host. The port carries the derived default; the `container` override
/// (`:21-23` — the element/shadow-root/ref union) folds to the ported
/// `FloatingPortalContainer` at the view layer, which owns that machinery; the
/// host-testable layer tracks only its presence.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PortalProps {
    /// `keepMounted` (`ComboboxPortal.tsx:15-19` — `@default false`).
    pub keep_mounted: bool,
    /// Whether the consumer named a `container` override (`:21-23`).
    pub has_container: bool,
}

/// Resolves the effective props (`ComboboxPortal.tsx:14`'s destructuring
/// default): `None` keep_mounted is the `false` default.
pub fn resolve_portal_props(keep_mounted: Option<bool>) -> PortalProps {
    PortalProps {
        keep_mounted: keep_mounted.unwrap_or(false),
        has_container: false,
    }
}

/// The state shape the part exposes — empty upstream
/// (`ComboboxPortal.tsx:27`), carried for the view layer's symmetry.
pub type PortalState = ComboboxState;

#[cfg(test)]
mod portal_wiring_tests {
    use super::*;
    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};
    use leptos_ui_utils::react_store::ReactStore;

    fn store_with(mounted: bool, force_mounted: bool) -> ComboboxStore {
        let state = ComboboxState {
            id: None,
            label_id: None,
            items: None,
            selected_value: serde_json::Value::Null,
            open: mounted,
            mounted,
            transition_status: "indeterminate".into(),
            force_mounted,
            inline: false,
            active_index: None,
            selected_index: None,
            popup_props: Default::default(),
            list_props: Default::default(),
            input_props: Default::default(),
            trigger_props: Default::default(),
            item_props: Default::default(),
            positioner_element: None,
            list_element: None,
            popup_id: None,
            trigger_element: None,
            input_element: None,
            input_group_element: None,
            popup_side: None,
            open_method: None,
            input_inside_popup: false,
            input_owns_form_value: true,
            selection_mode: "single".into(),
            name: None,
            form: None,
            disabled: false,
            read_only: false,
            required: false,
            grid: false,
            virtualized: false,
            open_on_input_click: false,
            item_to_string_label: None,
            is_item_equal_to_value: ComboboxState::default_is_item_equal_to_value(),
            modal: false,
            auto_highlight: "false".into(),
            submit_on_item_click: false,
            has_input_value: false,
        };
        ReactStore::with_context(state, ComboboxStoreContext::default())
    }

    // ------------------------------------------------------------------
    // The render gate (ComboboxPortal.tsx:20-25)
    // ------------------------------------------------------------------

    // Closed, not kept, not forced — upstream `return null`.
    #[test]
    fn renders_nothing_when_closed_unforced_and_not_kept() {
        assert!(!should_render_portal(false, false, false));
    }

    // Any one of the three arms renders the portal subtree.
    #[test]
    fn each_arm_alone_renders_the_portal() {
        assert!(should_render_portal(true, false, false));
        assert!(should_render_portal(false, true, false));
        assert!(should_render_portal(false, false, true));
    }

    // The store fold (`ComboboxPortal.tsx:22-24`) — the mounted and
    // forceMounted reads come from the store, keepMounted from the prop.
    #[test]
    fn the_store_fold_drives_the_gate() {
        assert!(portal_should_render(&store_with(true, false), false));
        assert!(portal_should_render(&store_with(false, false), true));
        assert!(portal_should_render(&store_with(false, true), false));
        assert!(!portal_should_render(&store_with(false, false), false));
    }

    // `ComboboxPortal.test.tsx` (`describeConformance(<Combobox.Portal/>, render
    // in <Combobox.Root open>…)`) — the open-root mount renders the portal host.
    #[test]
    fn an_open_root_renders_the_portal() {
        assert!(portal_should_render(&store_with(true, false), false));
    }

    // ------------------------------------------------------------------
    // The portal context (ComboboxPortalContext.tsx:5-19)
    // ------------------------------------------------------------------

    // The provider publishes the keepMounted flag verbatim.
    #[test]
    fn the_context_publishes_the_keep_mounted_flag() {
        assert!(ComboboxPortalContextValue { keep_mounted: true }.keep_mounted);
        assert!(
            !ComboboxPortalContextValue {
                keep_mounted: false
            }
            .keep_mounted
        );
    }

    // `ComboboxPortalContext.tsx:13-18` — the non-optional access throws
    // outside a Portal; the optional form returns None instead.
    #[test]
    #[should_panic(expected = "<Combobox.Portal> is missing")]
    fn the_non_optional_access_throws_outside_a_portal() {
        let _ = use_portal_context(None, false);
    }

    #[test]
    fn the_optional_access_returns_none_outside_a_portal() {
        assert_eq!(use_portal_context(None, true), None);
    }

    // ------------------------------------------------------------------
    // The prop defaults (ComboboxPortal.tsx:14)
    // ------------------------------------------------------------------

    // `keepMounted = false` is the destructuring default.
    #[test]
    fn keep_mounted_defaults_to_false() {
        assert!(!resolve_portal_props(None).keep_mounted);
        assert!(resolve_portal_props(Some(true)).keep_mounted);
    }
}
