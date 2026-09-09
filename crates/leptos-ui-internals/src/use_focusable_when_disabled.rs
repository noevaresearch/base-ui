//! Port of `packages/react/src/utils/useFocusableWhenDisabled.ts` — the disabled /
//! focusability attribute policy shared by [`crate::use_button`] and (upstream) the
//! Radio/Checkbox/Switch/Toolbar/Menu internals
//! (`specs/library/internals/implementation.md`, "useButton": "Disabled/focusability
//! attribute policy is delegated to `useFocusableWhenDisabled` … which decides `disabled`
//! vs `aria-disabled` vs `tabIndex` from `native`/`composite`/`focusableWhenDisabled`",
//! citing `useButton.ts:28-34` and `useFocusableWhenDisabled.ts:20-58`).
//!
//! ## Rust adaptations
//!
//! - Upstream builds the props in a `useMemo` recomputed per render with the current
//!   `disabled` value (`useFocusableWhenDisabled.ts:20-58`); the port's hook body runs
//!   once, so the two members whose presence or value depends on `disabled` — the
//!   non-native `aria-disabled` and the `-1` `tabIndex` override — become lazy
//!   [`ElementAttributeFn`] closures re-reading the reactive `disabled` source at read
//!   time (the way a React re-render would re-derive the prop), and the keydown handler
//!   reads it untracked at call time (the `usePressAndHold` convention).
//! - The returned `props.onKeyDown` (`useFocusableWhenDisabled.ts:23-27`) keeps the
//!   [`BaseUIEvent`] argument the merge pipeline hands it — the handler runs as the
//!   third bag in `getButtonProps`' merge order (before the internal `onKeyDown`), and
//!   the merge composition attaches the augmentation to the event object before
//!   fan-out (`packages/react/src/merge-props/mergeProps.ts:229-244`).
//! - Upstream's "we can't explicitly assign `undefined` … because it would otherwise
//!   prevent subsequently merged props from setting them" comment
//!   (`useFocusableWhenDisabled.ts:18-19`) is about the merge's omitted-vs-set
//!   distinction; the port expresses it the same way [`ElementAttributeFn`] does — a
//!   closure returning `None` is upstream's omitted prop.
//! - The boolean `disabled` prop (`useFocusableWhenDisabled.ts:45-47`) renders like
//!   React's boolean DOM attributes: present (empty value) when true, absent when
//!   false. `aria-disabled` renders like React's `aria-*` handling: the literal
//!   `"false"` string when the member is present and the state is false
//!   (`useFocusableWhenDisabled.ts:38-43`).
//! - `sets_disabled` exposes whether the policy contributes the `disabled` member at
//!   all (`useFocusableWhenDisabled.ts:45-47` — its value only depends on the static
//!   parameters); `useButton`'s `updateDisabled` needs exactly that check
//!   (`useButton.ts:82`).

use std::rc::Rc;

use reactive_graph::traits::{Get, GetUntracked};
use web_sys::KeyboardEvent;

use crate::floating_ui::element_props::{ElementAttributeFn, ElementEventHandler};
use crate::types::BaseUIEvent;

/// The policy parameters (`UseFocusableWhenDisabledParameters`,
/// `useFocusableWhenDisabled.ts:70-93`), with upstream's documented defaults noted per
/// field.
pub struct UseFocusableWhenDisabledParams<D> {
    /// `focusableWhenDisabled` (`:75`): when `None` (upstream `undefined`),
    /// composite items are focusable when disabled by default.
    pub focusable_when_disabled: Option<bool>,
    /// `disabled` (`:79`): a reactive source — upstream reads the latest render's
    /// value; the port re-reads it lazily (see the module docs).
    pub disabled: D,
    /// `composite` (`:84` — upstream default `false`).
    pub composite: bool,
    /// `tabIndex` (`:88` — upstream default `0`).
    pub tab_index: i32,
    /// `isNativeButton` (`:92` — upstream default `true`).
    pub is_native_button: bool,
}

/// The policy's returned `props` (`FocusableWhenDisabledProps`,
/// `useFocusableWhenDisabled.ts:63-68`): the Tab-gate keydown handler plus the
/// attribute members (lazy — see the module docs).
pub struct FocusableWhenDisabledProps {
    /// `onKeyDown` (`:23-27`): when disabled and focusable, prevent every key except
    /// Tab (allowing Tabbing away from focusableWhenDisabled elements).
    pub on_key_down: ElementEventHandler<BaseUIEvent<KeyboardEvent>>,
    /// `tabIndex` / `'aria-disabled'` / `disabled` (`:30-47`) — presence and value
    /// per the upstream branches; `None` is upstream's omitted prop.
    pub attributes: Vec<(&'static str, ElementAttributeFn)>,
    /// Whether the `disabled` member is contributed at all
    /// (`useFocusableWhenDisabled.ts:45-47`); `useButton`'s `updateDisabled` reads
    /// this in place of upstream's `props.disabled === undefined` check
    /// (`useButton.ts:82`).
    pub sets_disabled: bool,
}

/// Upstream's `focusableWhenDisabled !== false` truthiness split
/// (`useFocusableWhenDisabled.ts:15-16`): `undefined` counts as focusable for the
/// composite arms.
fn is_focusable_composite(composite: bool, focusable_when_disabled: Option<bool>) -> bool {
    composite && focusable_when_disabled != Some(false)
}

/// Whether the `'aria-disabled'` member is contributed
/// (`useFocusableWhenDisabled.ts:38-43`). The first arm is static; the second
/// (`!is_native_button && disabled`) depends on the current disabled value, which the
/// caller's lazy closure re-evaluates.
fn sets_aria_disabled(
    is_native_button: bool,
    focusable_when_disabled: Option<bool>,
    composite: bool,
    disabled: bool,
) -> bool {
    (is_native_button
        && (focusable_when_disabled == Some(true)
            || is_focusable_composite(composite, focusable_when_disabled)))
        || (!is_native_button && disabled)
}

/// `useFocusableWhenDisabled` (`useFocusableWhenDisabled.ts:4-61`). Must be called
/// inside a reactive owner when `D` is a signal — the returned lazy closures read it
/// untracked, so no effect of its own is registered.
pub fn use_focusable_when_disabled<D>(
    params: UseFocusableWhenDisabledParams<D>,
) -> FocusableWhenDisabledProps
where
    D: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
{
    let UseFocusableWhenDisabledParams {
        focusable_when_disabled,
        disabled,
        composite,
        tab_index,
        is_native_button,
    } = params;

    // `isFocusableComposite` / `isNonFocusableComposite` (`:15-16`) — static
    // derivations of the parameters.
    let non_focusable_composite = composite && focusable_when_disabled == Some(false);

    // `onKeyDown` (`:23-27`): the Tab gate, read against the latest disabled value.
    let on_key_down: ElementEventHandler<BaseUIEvent<KeyboardEvent>> = {
        let disabled = disabled.clone();
        Rc::new(move |event| {
            if disabled.get_untracked()
                && focusable_when_disabled == Some(true)
                && event.inner().key() != "Tab"
            {
                event.inner().prevent_default();
            }
        })
    };

    let mut attributes: Vec<(&'static str, ElementAttributeFn)> = Vec::new();

    // `tabIndex` (`:30-36`): only non-composite items contribute it — composite items
    // get their roving `tabindex` from the composite machinery instead. The `-1`
    // override re-derives per read like the upstream `useMemo` does per render.
    if !composite {
        let disabled = disabled.clone();
        attributes.push((
            "tabindex",
            Rc::new(move || {
                if !is_native_button
                    && disabled.get_untracked()
                    && focusable_when_disabled != Some(true)
                {
                    Some("-1".to_string())
                } else {
                    Some(tab_index.to_string())
                }
            }) as ElementAttributeFn,
        ));
    }

    // `'aria-disabled'` (`:38-43`): the member's presence is static for native
    // buttons (arm 1) but rides on `disabled` for non-native ones (arm 2), so the
    // closure evaluates the whole condition per read — `None` when neither arm holds.
    // The value is the literal boolean state string — React renders
    // `aria-disabled={false}` as `"false"`.
    let aria_static = sets_aria_disabled(
        is_native_button,
        focusable_when_disabled,
        composite,
        false,
    );
    if aria_static || !is_native_button {
        let disabled = disabled.clone();
        attributes.push((
            "aria-disabled",
            Rc::new(move || {
                let disabled_now = disabled.get_untracked();
                (aria_static || disabled_now)
                    .then(|| if disabled_now { "true" } else { "false" }.to_string())
            }) as ElementAttributeFn,
        ));
    }

    // `disabled` (`:45-47`): contributed when the button is native and not in one of
    // the focusable-when-disabled arms; like React's boolean DOM attribute it renders
    // empty when true and is absent when false.
    let sets_disabled = is_native_button
        && (focusable_when_disabled != Some(true) || non_focusable_composite);
    if sets_disabled {
        let disabled = disabled.clone();
        attributes.push((
            "disabled",
            Rc::new(move || disabled.get_untracked().then(String::new)) as ElementAttributeFn,
        ));
    }

    FocusableWhenDisabledProps {
        on_key_down,
        attributes,
        sets_disabled,
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::Set;

    fn evaluate(attributes: &[(&'static str, ElementAttributeFn)]) -> Vec<(String, String)> {
        attributes
            .iter()
            .filter_map(|(name, value)| {
                value().map(|value| ((*name).to_string(), value))
            })
            .collect()
    }

    fn find<'a>(
        attributes: &'a [(String, String)],
        name: &str,
    ) -> Option<&'a str> {
        attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    fn run(params: UseFocusableWhenDisabledParams<RwSignal<bool>>) -> FocusableWhenDisabledProps {
        let _owner = reactive_graph::owner::Owner::new();
        let _guard = _owner.set();
        use_focusable_when_disabled(params)
    }

    // The `!composite` arm (`useFocusableWhenDisabled.ts:30-36`): a native
    // non-composite button always carries the explicit `tabIndex`, and the
    // `disabled` member follows React's boolean-attribute rendering — present
    // (empty value) when the state is true, absent when false.
    #[test]
    fn native_non_composite_returns_the_tab_index() {
        let disabled = RwSignal::new(false);
        let props = run(UseFocusableWhenDisabledParams {
            focusable_when_disabled: None,
            disabled,
            composite: false,
            tab_index: 0,
            is_native_button: true,
        });

        let evaluated = evaluate(&props.attributes);
        assert_eq!(find(&evaluated, "tabindex"), Some("0"));
        assert_eq!(find(&evaluated, "disabled"), None, "a false state renders no attribute — React's boolean-attribute rule (:45-47)");
        assert_eq!(props.sets_disabled, true);

        disabled.set(true);
        let evaluated = evaluate(&props.attributes);
        assert_eq!(find(&evaluated, "disabled"), Some(""), "a true state renders the empty-valued attribute");
    }

    // The `!isNativeButton && disabled` arm (`:33-35`): a disabled non-native
    // non-composite element drops to `tabIndex: -1` when not focusable when
    // disabled, and re-derives when the disabled source flips.
    #[test]
    fn disabled_non_native_drops_to_tab_index_minus_one_until_focusable() {
        let disabled = RwSignal::new(true);
        let props = run(UseFocusableWhenDisabledParams {
            focusable_when_disabled: None,
            disabled,
            composite: false,
            tab_index: 0,
            is_native_button: false,
        });

        let evaluated = evaluate(&props.attributes);
        assert_eq!(find(&evaluated, "tabindex"), Some("-1"));
        assert_eq!(find(&evaluated, "aria-disabled"), Some("true"), "!native && disabled contributes aria-disabled (:38-43)");

        disabled.set(false);
        let evaluated = evaluate(&props.attributes);
        assert_eq!(find(&evaluated, "tabindex"), Some("0"), "the override re-derives per read like the upstream useMemo per render");
        assert_eq!(find(&evaluated, "aria-disabled"), None, "the member is omitted — upstream's can't-assign-undefined comment (:18-19)");

        disabled.set(true);
        let focusable = run(UseFocusableWhenDisabledParams {
            focusable_when_disabled: Some(true),
            disabled,
            composite: false,
            tab_index: 0,
            is_native_button: false,
        });
        let evaluated = evaluate(&focusable.attributes);
        assert_eq!(find(&evaluated, "tabindex"), Some("0"), "focusableWhenDisabled keeps the explicit tab index (:33-35)");
    }

    // The composite arms (`:15-16`, `:30-47`): composite items contribute no
    // `tabIndex` at all, a native composite carries `aria-disabled` with the literal
    // `"false"` while enabled, and `focusableWhenDisabled: false` puts the disabled
    // attribute back.
    #[test]
    fn composite_items_skip_tab_index_and_use_aria_disabled() {
        let disabled = RwSignal::new(false);
        let props = run(UseFocusableWhenDisabledParams {
            focusable_when_disabled: None,
            disabled: disabled.clone(),
            composite: true,
            tab_index: 0,
            is_native_button: true,
        });

        let evaluated = evaluate(&props.attributes);
        assert_eq!(find(&evaluated, "tabindex"), None, "composite items get their roving tabindex from the composite machinery");
        assert_eq!(find(&evaluated, "aria-disabled"), Some("false"), "isFocusableComposite contributes the member even while enabled (:38-43)");
        assert_eq!(find(&evaluated, "disabled"), None, "while enabled the boolean attribute is absent — React's rendering of a false boolean prop (:45-47)");
        assert_eq!(props.sets_disabled, true, "the member is still contributed (useButton's updateDisabled check reads it, useButton.ts:82)");

        disabled.set(true);
        let evaluated = evaluate(&props.attributes);
        assert_eq!(find(&evaluated, "aria-disabled"), Some("true"));
        assert_eq!(find(&evaluated, "disabled"), Some(""));

        // `focusableWhenDisabled: false` — the isNonFocusableComposite arm: the
        // disabled attribute stays, aria-disabled flips off while enabled.
        let non_focusable = run(UseFocusableWhenDisabledParams {
            focusable_when_disabled: Some(false),
            disabled: RwSignal::new(false),
            composite: true,
            tab_index: 0,
            is_native_button: true,
        });
        let evaluated = evaluate(&non_focusable.attributes);
        assert_eq!(find(&evaluated, "aria-disabled"), None, "non-focusable composite is not in the aria-disabled arms while enabled (:38-43)");
        assert_eq!(find(&evaluated, "disabled"), None, "while enabled the boolean attribute is absent; the member exists (:45-47)");
        assert_eq!(non_focusable.sets_disabled, true, "isNonFocusableComposite keeps the disabled member (:45-47)");
    }

    // The presence/value decision table for `sets_disabled` (`:45-47`) — the check
    // `useButton`'s updateDisabled runs in place of `props.disabled === undefined`
    // (`useButton.ts:82`).
    #[test]
    fn sets_disabled_follows_the_native_and_focusable_arms() {
        let cases: &[(bool, Option<bool>, bool, bool)] = &[
            // (native, focusableWhenDisabled, composite, expected sets_disabled)
            (true, None, false, true),
            (true, Some(true), false, false),
            (true, Some(false), false, true),
            (true, None, true, true),
            (true, Some(true), true, false),
            (true, Some(false), true, true),
            (false, None, false, false),
            (false, Some(true), false, false),
        ];
        for &(native, fwd, composite, expected) in cases {
            let props = run(UseFocusableWhenDisabledParams {
                focusable_when_disabled: fwd,
                disabled: RwSignal::new(false),
                composite,
                tab_index: 0,
                is_native_button: native,
            });
            assert_eq!(props.sets_disabled, expected, "native={native} fwd={fwd:?} composite={composite}");
        }
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;

    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::Set;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // The Tab gate (`useFocusableWhenDisabled.ts:23-27`): every non-Tab key is
    // prevented while disabled and focusable, Tab passes, and the gate is inert while
    // enabled. Runs in the browser — real `KeyboardEvent`s cannot be constructed on
    // the host.
    #[wasm_bindgen_test]
    fn the_tab_gate_prevents_every_key_except_tab_while_disabled() {
        let disabled = RwSignal::new(true);
        let props = {
            let _owner = reactive_graph::owner::Owner::new();
            let _guard = _owner.set();
            use_focusable_when_disabled(UseFocusableWhenDisabledParams {
                focusable_when_disabled: Some(true),
                disabled: disabled.clone(),
                composite: false,
                tab_index: 0,
                is_native_button: false,
            })
        };

        let prevent_key = |key: &str| {
            let init = web_sys::KeyboardEventInit::new();
            // Real keyboard events are cancelable — without this, `preventDefault`
            // would be a no-op and the gate could never be observed.
            init.set_cancelable(true);
            init.set_key(key);
            let event = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
                .expect("KeyboardEvent failed");
            let wrapped = BaseUIEvent::new(event);
            (props.on_key_down)(&wrapped);
            wrapped.inner().default_prevented()
        };

        assert!(prevent_key("Enter"), "a disabled focusable element prevents non-Tab keys (:23-27)");
        assert!(prevent_key(" "));
        assert!(!prevent_key("Tab"), "Tab is allowed so the element can be tabbed away from");

        disabled.set(false);
        assert!(!prevent_key("Enter"), "the gate is inert while enabled");
    }
}
