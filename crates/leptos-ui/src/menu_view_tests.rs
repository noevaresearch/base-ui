//! Host tests for the menu VIEW LAYER parts — `Menu.Portal`, `Menu.Positioner` and
//! `Menu.Popup`.
//!
//! These cover the parts' resolvable contracts: the render gate and owner role, the
//! per-parent geometry resolution, the focus derivations, and the `data-*` attribute sets
//! the shared state-attribute mapping produces. Nothing here needs a DOM — the box refuses
//! a browser by design (`ralph/generated/env-health.json` → `browser: DEGRADED`), so the
//! DOM consequences of these values are CI's to measure, exactly as the item's note records.

#[cfg(test)]
mod portal_host_tests {
    use crate::menu::portal::{
        MenuPortalContextValue, menu_portal_owner_role, menu_portal_should_render,
        use_menu_portal_context,
    };
    use crate::menu::store::{MenuParent, create_menu_store};

    fn menu_parent() -> MenuParent {
        MenuParent::Menu {
            store: create_menu_store(),
        }
    }

    // `MenuPortal.tsx:22-27` — `const shouldRender = mounted || keepMounted`.
    #[test]
    fn the_render_gate_is_mounted_or_keep_mounted() {
        assert!(!menu_portal_should_render(false, false), "upstream's `return null`");
        assert!(menu_portal_should_render(true, false));
        assert!(menu_portal_should_render(false, true));
        assert!(menu_portal_should_render(true, true));
    }

    // `MenuPortal.tsx:32` — `group` only under role-constrained parents.
    #[test]
    fn the_owner_role_is_group_only_for_menu_and_menubar_parents() {
        assert_eq!(menu_portal_owner_role(&menu_parent()), Some("group"));
        assert_eq!(menu_portal_owner_role(&MenuParent::Menubar), Some("group"));
        assert_eq!(menu_portal_owner_role(&MenuParent::None), None);
        assert_eq!(menu_portal_owner_role(&MenuParent::ContextMenu), None);
    }

    // `MenuPortal.tsx:35` + `MenuPortalContext.ts:4` — the provider publishes `keepMounted`
    // itself, so "provider with false" is distinct from "no provider".
    #[test]
    fn the_context_value_carries_the_keep_mounted_flag() {
        assert!(MenuPortalContextValue { keep_mounted: true }.keep_mounted);
        assert!(!MenuPortalContextValue { keep_mounted: false }.keep_mounted);
        assert_eq!(
            use_menu_portal_context(Some(MenuPortalContextValue { keep_mounted: false }), false),
            Some(MenuPortalContextValue { keep_mounted: false })
        );
    }

    // `MenuPortalContext.ts:8-10` — the required read throws outside a `<Menu.Portal>`.
    #[test]
    #[should_panic(expected = "Base UI: <Menu.Portal> is missing.")]
    fn the_required_portal_context_read_throws_without_a_portal() {
        let _ = use_menu_portal_context(None, false);
    }

    // The optional form is the positioner's probe shape (`MenuPositioner.tsx:60`).
    #[test]
    fn the_optional_portal_context_read_returns_none() {
        assert_eq!(use_menu_portal_context(None, true), None);
    }
}

#[cfg(test)]
mod positioner_host_tests {
    use crate::menu::positioner::{
        BackdropCutout, CollisionAvoidanceChoice, MenubarOrientation, PositionMethod,
        menu_positioner_attributes, menu_positioner_backdrop_cutout,
        menu_positioner_popup_modal, menu_positioner_should_render_backdrop, menu_positioner_state,
        resolve_menu_positioner, use_menu_positioner_context,
    };
    use crate::menu::store::{MenuInstantType, MenuParent, create_menu_store};
    use leptos_ui_internals::constants::{
        DROPDOWN_COLLISION_AVOIDANCE, POPUP_COLLISION_AVOIDANCE,
    };
    use leptos_ui_internals::floating_ui::reasons;
    use leptos_ui_internals::use_anchor_positioning::{Align, Side};

    fn submenu() -> MenuParent {
        MenuParent::Menu {
            store: create_menu_store(),
        }
    }

    fn resolve(parent: &MenuParent) -> crate::menu::positioner::MenuPositionerResolution {
        resolve_menu_positioner(
            parent,
            None,
            None,
            None,
            None,
            None,
            MenubarOrientation::Horizontal,
        )
    }

    // `MenuPositioner.tsx:99-102` — a submenu defaults to `inline-end`/`start` with the
    // popup collision-avoidance preset.
    #[test]
    fn the_submenu_arm_defaults_to_inline_end_start_and_the_popup_preset() {
        let resolution = resolve(&submenu());
        assert_eq!(resolution.side, Side::InlineEnd, "`:100`");
        assert_eq!(resolution.align, Align::Start, "`:101`");
        assert_eq!(
            resolution.collision_avoidance,
            CollisionAvoidanceChoice::Popup,
            "`:102`"
        );
    }

    // The root arm passes the props through, so the engine's own defaults (`:142`, `:144`)
    // are what a top-level menu gets, with the prop's default preset (`:53`).
    #[test]
    fn the_root_arm_uses_the_engine_defaults() {
        let resolution = resolve(&MenuParent::None);
        assert_eq!(resolution.side, Side::Bottom, "useAnchorPositioning.ts:142");
        assert_eq!(resolution.align, Align::Center, "useAnchorPositioning.ts:144");
        assert_eq!(
            resolution.collision_avoidance,
            CollisionAvoidanceChoice::Dropdown,
            "`:53`"
        );
    }

    // `MenuPositioner.tsx:103-107` — the menubar arm reads the parent context's orientation.
    #[test]
    fn the_menubar_arm_follows_the_orientation() {
        let horizontal = resolve_menu_positioner(
            &MenuParent::Menubar,
            None,
            None,
            None,
            None,
            None,
            MenubarOrientation::Horizontal,
        );
        assert_eq!(horizontal.side, Side::Bottom, "`:105` non-vertical arm");
        assert_eq!(horizontal.align, Align::Start, "`:106`");

        let vertical = resolve_menu_positioner(
            &MenuParent::Menubar,
            None,
            None,
            None,
            None,
            None,
            MenubarOrientation::Vertical,
        );
        assert_eq!(vertical.side, Side::InlineEnd, "`:105` vertical arm");
        assert_eq!(vertical.align, Align::Start);
    }

    // `MenuPositioner.tsx:88-95` — the context-menu overrides: `align ?? 'start'`, and the
    // `2`/`-5` offset pair only while no side was named.
    #[test]
    fn the_context_menu_arm_applies_the_offset_pair_when_no_side_is_named() {
        let resolution = resolve(&MenuParent::ContextMenu);
        assert_eq!(resolution.align, Align::Start, "`:90`");
        assert_eq!(resolution.align_offset, 2.0, "`:92`");
        assert_eq!(resolution.side_offset, -5.0, "`:93`");
        assert_eq!(resolution.arrow_padding, 0.0, "`:120`");
        assert!(resolution.context_menu_shift, "`:128-133`");
        assert_eq!(resolution.position_method, PositionMethod::Absolute);
    }

    // A named side suppresses both context-menu offset defaults (`if (!side && align !==
    // 'center')`, `:91`).
    #[test]
    fn a_named_side_suppresses_the_context_menu_offsets() {
        let resolution = resolve_menu_positioner(
            &MenuParent::ContextMenu,
            Some(Side::Top),
            None,
            None,
            None,
            None,
            MenubarOrientation::Horizontal,
        );
        assert_eq!(resolution.side, Side::Top);
        assert_eq!(resolution.align_offset, 0.0, "`:83-86` defaults kept");
        assert_eq!(resolution.side_offset, 0.0);
    }

    // The props win over every per-parent default, and an explicit preset is kept verbatim
    // (`CollisionAvoidanceChoice::Custom`).
    #[test]
    fn the_props_win_over_the_per_parent_defaults() {
        let resolution = resolve_menu_positioner(
            &submenu(),
            Some(Side::Left),
            Some(Align::End),
            Some(3.0),
            Some(-4.0),
            Some(&DROPDOWN_COLLISION_AVOIDANCE),
            MenubarOrientation::Horizontal,
        );
        assert_eq!(resolution.side, Side::Left);
        assert_eq!(resolution.align, Align::End);
        assert_eq!(resolution.side_offset, 3.0);
        assert_eq!(resolution.align_offset, -4.0);
        assert_eq!(
            resolution.collision_avoidance,
            CollisionAvoidanceChoice::Custom
        );
        // The preset itself is the caller's, not the submenu default.
        assert_eq!(
            DROPDOWN_COLLISION_AVOIDANCE.fallback_axis_side,
            "none",
            "the constant the Custom arm applies (`constants.rs:85-88`)"
        );
        assert_eq!(POPUP_COLLISION_AVOIDANCE.fallback_axis_side, "end");
    }

    // `MenuPositioner.tsx:267-268` — `popupModal = modal && lastOpenChangeReason !==
    // REASONS.triggerHover`.
    #[test]
    fn the_popup_modal_predicate_excludes_trigger_hover() {
        assert!(menu_positioner_popup_modal(true, None));
        assert!(menu_positioner_popup_modal(true, Some(reasons::TRIGGER_PRESS)));
        assert!(!menu_positioner_popup_modal(true, Some(reasons::TRIGGER_HOVER)));
        assert!(!menu_positioner_popup_modal(false, None));
    }

    // `MenuPositioner.tsx:286-290` — the backdrop gate.
    #[test]
    fn the_backdrop_gate_matches_upstream() {
        // Unmounted never renders the backdrop.
        assert!(!menu_positioner_should_render_backdrop(
            false,
            &MenuParent::None,
            true,
            None,
            false
        ));
        // `parent.type !== 'menu'` guards the whole expression.
        assert!(!menu_positioner_should_render_backdrop(
            true,
            &submenu(),
            true,
            None,
            false
        ));
        // A mounted, modal, non-hover root renders it.
        assert!(menu_positioner_should_render_backdrop(
            true,
            &MenuParent::None,
            true,
            None,
            false
        ));
        // The menubar arm reads its own context's modal flag.
        assert!(!menu_positioner_should_render_backdrop(
            true,
            &MenuParent::Menubar,
            true,
            None,
            false
        ));
        assert!(menu_positioner_should_render_backdrop(
            true,
            &MenuParent::Menubar,
            false,
            None,
            true
        ));
    }

    // `MenuPositioner.tsx:293-298` — the cutout selection.
    #[test]
    fn the_backdrop_cutout_selection_matches_upstream() {
        assert_eq!(
            menu_positioner_backdrop_cutout(&MenuParent::Menubar),
            BackdropCutout::MenubarContent
        );
        assert_eq!(
            menu_positioner_backdrop_cutout(&MenuParent::None),
            BackdropCutout::Trigger
        );
        assert_eq!(
            menu_positioner_backdrop_cutout(&submenu()),
            BackdropCutout::None
        );
        assert_eq!(
            menu_positioner_backdrop_cutout(&MenuParent::ContextMenu),
            BackdropCutout::None
        );
    }

    // `usePositioner.tsx:43` over `popupStateMapping.ts:53-66` plus the default handling
    // (`getStateAttributesProps.ts:27-29`): `open` maps to `data-open`/`data-closed`, a
    // hidden anchor to `data-anchor-hidden`, and the unmapped state fields become their
    // kebab-case `data-*` attributes.
    #[test]
    fn the_positioner_state_attributes_match_the_mapping() {
        let open = menu_positioner_state(
            true,
            Side::Top,
            Align::Start,
            true,
            true,
            Some(MenuInstantType::TriggerChange),
        );
        let attributes = menu_positioner_attributes(&open);
        let names: Vec<&str> = attributes.iter().map(|(key, _)| key.as_str()).collect();
        assert!(names.contains(&"data-open"), "{attributes:?}");
        assert!(!names.contains(&"data-closed"), "{attributes:?}");
        assert!(names.contains(&"data-anchor-hidden"), "{attributes:?}");
        assert!(names.contains(&"data-nested"), "{attributes:?}");
        assert!(names.contains(&"data-instant"), "{attributes:?}");
        assert_eq!(
            attributes
                .iter()
                .find(|(key, _)| key == "data-side")
                .map(|(_, value)| value.as_str()),
            Some("top"),
            "the kebab spelling of the logical side (`useAnchorPositioning.ts:597`)"
        );
        assert_eq!(
            attributes
                .iter()
                .find(|(key, _)| key == "data-align")
                .map(|(_, value)| value.as_str()),
            Some("start")
        );

        // Closed: the mapping's other branch (`popupStateMapping.ts:55-58`), and a visible
        // anchor emits nothing (`:60-63`).
        let closed = menu_positioner_state(false, Side::Bottom, Align::Center, false, false, None);
        let closed_attributes = menu_positioner_attributes(&closed);
        let closed_names: Vec<&str> = closed_attributes.iter().map(|(key, _)| key.as_str()).collect();
        assert!(closed_names.contains(&"data-closed"), "{closed_attributes:?}");
        assert!(!closed_names.contains(&"data-open"), "{closed_attributes:?}");
        assert!(!closed_names.contains(&"data-anchor-hidden"));
        assert!(!closed_names.contains(&"data-nested"));
        assert!(!closed_names.contains(&"data-instant"));
    }

    // `MenuPositionerContext.ts:12-21` — the required read throws outside a Positioner.
    #[test]
    #[should_panic(
        expected = "Base UI: MenuPositionerContext is missing. MenuPositioner parts must be placed within <Menu.Positioner>."
    )]
    fn the_required_positioner_context_read_throws_without_a_positioner() {
        let _ = use_menu_positioner_context(None, false);
    }

    #[test]
    fn the_optional_positioner_context_read_returns_none() {
        assert!(use_menu_positioner_context(None, true).is_none());
    }
}

#[cfg(test)]
mod popup_host_tests {
    use crate::menu::popup::{
        menu_popup_attributes, menu_popup_hover_interaction_enabled, menu_popup_initial_focus,
        menu_popup_is_context_menu, menu_popup_return_focus, menu_popup_root_owner_id,
        menu_popup_should_stop_composite_key, menu_popup_state,
    };
    use crate::menu::store::{MenuInstantType, MenuParent, create_menu_store};
    use leptos_ui_internals::floating_ui::reasons;
    use leptos_ui_internals::use_anchor_positioning::{Align, Side};
    use leptos_ui_internals::use_transition_status::TransitionStatus;

    fn submenu() -> MenuParent {
        MenuParent::Menu {
            store: create_menu_store(),
        }
    }

    // `MenuPopup.tsx:114-120` — the returnFocus derivation.
    #[test]
    fn the_return_focus_derivation_matches_upstream() {
        // `parent.type === undefined` → true (`:114`).
        assert!(menu_popup_return_focus(&MenuParent::None, false, None, None));
        // A context menu returns focus too (`:114`).
        assert!(menu_popup_return_focus(
            &MenuParent::ContextMenu,
            false,
            None,
            None
        ));
        // A submenu without an active trigger does not (`:114`).
        assert!(!menu_popup_return_focus(&submenu(), false, None, None));
        // An active trigger forces it (`:116-118`).
        assert!(menu_popup_return_focus(&submenu(), true, None, None));
        // The menubar arm: anything but an outside press forces it (`:117-119`).
        assert!(menu_popup_return_focus(
            &MenuParent::Menubar,
            false,
            Some(reasons::TRIGGER_PRESS),
            None
        ));
        assert!(!menu_popup_return_focus(
            &MenuParent::Menubar,
            false,
            Some(reasons::OUTSIDE_PRESS),
            None
        ));
        // `finalFocus` overrides both directions (`:120`).
        assert!(menu_popup_return_focus(&submenu(), false, None, Some(true)));
        assert!(!menu_popup_return_focus(&MenuParent::None, true, None, Some(false)));
    }

    // `MenuPopup.tsx:129` — `initialFocus={parent.type !== 'menu'}`.
    #[test]
    fn a_submenu_does_not_take_initial_focus() {
        assert!(!menu_popup_initial_focus(&submenu()));
        assert!(menu_popup_initial_focus(&MenuParent::None));
        assert!(menu_popup_initial_focus(&MenuParent::Menubar));
    }

    // `MenuPopup.tsx:79-82` — `hoverEnabled && !disabled && !isContextMenu &&
    // parent.type !== 'menubar'`.
    #[test]
    fn the_hover_interaction_enablement_matches_upstream() {
        assert!(menu_popup_hover_interaction_enabled(
            &MenuParent::None,
            true,
            false
        ));
        assert!(!menu_popup_hover_interaction_enabled(
            &MenuParent::None,
            false,
            false
        ));
        assert!(!menu_popup_hover_interaction_enabled(
            &MenuParent::None,
            true,
            true
        ));
        assert!(!menu_popup_hover_interaction_enabled(
            &MenuParent::Menubar,
            true,
            false
        ));
        assert!(!menu_popup_hover_interaction_enabled(
            &MenuParent::ContextMenu,
            true,
            false
        ));
    }

    #[test]
    fn the_context_menu_predicate_and_the_composite_key_guard() {
        assert!(menu_popup_is_context_menu(&MenuParent::ContextMenu));
        assert!(!menu_popup_is_context_menu(&submenu()));
        // `MenuPopup.tsx:100-107`.
        assert!(menu_popup_should_stop_composite_key(true, true));
        assert!(!menu_popup_should_stop_composite_key(true, false));
        assert!(!menu_popup_should_stop_composite_key(false, true));
    }

    // `MenuPopup.tsx:110` — the marker the trigger's drag-release guard walks for.
    #[test]
    fn the_root_owner_marker_skips_an_absent_id() {
        assert_eq!(
            menu_popup_root_owner_id(Some("base-ui-menu-1")),
            Some("base-ui-menu-1".to_owned())
        );
        assert_eq!(menu_popup_root_owner_id(None), None);
        assert_eq!(menu_popup_root_owner_id(Some("")), None);
    }

    // `MenuPopup.tsx:95` over `popupTransitionStateMapping` (`popupStateMapping.ts:68-73`):
    // open/closed, the transition hooks, and the default handling for the unmapped fields.
    #[test]
    fn the_popup_state_attributes_match_the_mapping() {
        let state = menu_popup_state(
            true,
            Some(Side::Bottom),
            Some(Align::Center),
            Some(TransitionStatus::Starting),
            &submenu(),
            Some(MenuInstantType::Click),
        );
        let attributes = menu_popup_attributes(&state);
        let names: Vec<&str> = attributes.iter().map(|(key, _)| key.as_str()).collect();
        assert!(names.contains(&"data-open"), "{attributes:?}");
        assert!(names.contains(&"data-starting-style"), "{attributes:?}");
        assert!(!names.contains(&"data-ending-style"), "{attributes:?}");
        assert!(names.contains(&"data-nested"), "{attributes:?}");
        assert!(names.contains(&"data-side"), "{attributes:?}");
        assert!(names.contains(&"data-align"), "{attributes:?}");
        assert!(names.contains(&"data-instant"), "{attributes:?}");

        // `'ending'` is the other transition hook (`stateAttributesMapping.ts:15-17`).
        let ending = menu_popup_state(
            false,
            None,
            None,
            Some(TransitionStatus::Ending),
            &MenuParent::None,
            None,
        );
        let ending_attributes = menu_popup_attributes(&ending);
        let ending_names: Vec<&str> = ending_attributes.iter().map(|(key, _)| key.as_str()).collect();
        assert!(ending_names.contains(&"data-ending-style"), "{ending_attributes:?}");
        assert!(ending_names.contains(&"data-closed"), "{ending_attributes:?}");
        assert!(!ending_names.contains(&"data-open"));

        // `'idle'`: the mapping owns the key and declines it, so no attribute appears
        // (`stateAttributesMapping.ts:18-19` + `getStateAttributesProps.ts:20-21`).
        let idle = menu_popup_state(
            true,
            None,
            None,
            Some(TransitionStatus::Idle),
            &MenuParent::None,
            None,
        );
        let idle_attributes = menu_popup_attributes(&idle);
        let idle_names: Vec<&str> = idle_attributes.iter().map(|(key, _)| key.as_str()).collect();
        assert!(
            !idle_names.contains(&"data-transitionstatus"),
            "{idle_attributes:?}"
        );
        assert!(!idle_names.contains(&"data-starting-style"));
    }
}
