//! Port of `packages/react/src/toggle/Toggle.tsx` — the `library: toggle` TODO item
//! (`specs/library/toggle/behavior.md`, `specs/library/toggle/implementation.md`).
//!
//! Upstream is a single two-state button (`packages/react/src/toggle/Toggle.tsx:24-140`):
//! `useBaseUiId(valueProp || undefined)` resolves the group-membership key (`:44`),
//! `useToggleGroupContext()` is the unit's single branch point (`:45`), the pressed-state
//! duality collapses into one `useControlled` call whose `controlled` argument is
//! `groupContext ? value !== undefined && groupValue.indexOf(value) > -1 : pressedProp`
//! (`:63-68`), `useButton({ disabled, native: nativeButton })` supplies the button
//! semantics factory (`:70-73`), and one `onClick` is the entire transition machine
//! (`:84-105`): `nextPressed = !pressed`, one shared
//! `createChangeEventDetails(REASONS.none, event.nativeEvent)` handed to both the user
//! callback and the group commit so a single `cancel()` vetoes both (the in-source
//! comment at `:88-89`), `onPressedChange` before any commit (`:90`), gate 2 the group
//! commit when `value` is truthy (`:96-98`), gate 3 a second `isCanceled` check for the
//! reverse veto (`:100-102`), and `setPressedState` last (`:104`).
//!
//! `useRenderElement('button', …, { enabled: !groupContext, state, ref: refs, props })`
//! renders the standalone path (`:111-116`) with the props array `[baseProps,
//! elementProps, getButtonProps]` (`:81-109`); the grouped branch renders `CompositeItem
//! tag="button"` instead (`:125-138`) — re-entering `useRenderElement` with the same
//! state and the same props array so data attributes and handlers are identical on both
//! paths, only composite props added on top (implementation.md, "State → data attributes
//! and the two render paths").
//!
//! ## Rust adaptations
//!
//! - The two render paths run over the machinery the internals crate already exposes:
//!   standalone is [`use_render_element`] with `enabled: !grouped`, grouped is
//!   [`composite_item`] with the same state and props bags — the `enabled` path switch
//!   and the "identical on both paths" contract, over the ported composite subsystem.
//!   The grouped path additionally requires a `CompositeRoot` in scope (upstream
//!   ToggleGroup provides one, `ToggleGroup.tsx:8,111`); a group context without one
//!   panics in `useCompositeItem` exactly like upstream's missing-provider contract.
//! - The `use_button` port returns the composed [`ButtonProps`] bag around the
//!   external handlers Toggle passes (`:81-109`'s third bag: Toggle passes *no*
//!   external button handlers — its own `onClick` machine is a separate, earlier bag —
//!   so the external bag is empty and the composition contributes the disabled-guard
//!   handlers and the `{ type: 'button' }`/`{ role: 'button' }` +
//!   focusableWhenDisabled attribute members). The three bags merge left-to-right with
//!   later handlers running first (the `mergePropsN` rule), reproducing upstream's
//!   order: `getButtonProps`' disabled guard runs before Toggle's machine and before
//!   the consumer's handler.
//! - `aria-pressed: pressed` (`:83`) is a lazy attribute closure over the pressed
//!   signal: React stringifies the ARIA boolean per render, the port re-derives the
//!   string per attribute read — the same always-present `'true'`/`'false'` values
//!   behavior.md's "Accessibility" section records.
//! - The shared `details` object is [`BaseUIChangeEventDetails`] over the raw native
//!   click, shared with the group commit exactly like upstream's closure-scoped
//!   `canceled` flag read through the `isCanceled` getter
//!   (`packages/react/src/internals/createBaseUIEventDetails.ts:118-149`).
//! - The discarded `form`/`type` props (`:32,36`) have no Rust prop — the strongest
//!   form of "deliberately discarded". `'use client'` (`:1`) is N/A.
//! - The grouped membership probe (`:64`) reads the group's `value` array at body time
//!   (the run-once component model); the group's reactive value source re-rendering
//!   the subtree is the ToggleGroup unit's concern (`library: toggle-group`).

use std::rc::Rc;

use reactive_graph::owner::use_context;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::GetUntracked;
use serde_json::json;
use web_sys::MouseEvent;

use leptos_ui_internals::composite_view::{CompositeItemComponentProps, composite_item};
use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::floating_ui::element_props::ElementAttributeFn;
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::types::BaseUIEvent;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_button::{ButtonExternalHandlers, UseButtonParams, use_button};
use leptos_ui_internals::use_render_element::{
    RenderElementHandlers, RenderElementProps, RenderedElement, UseRenderElementComponentProps,
    UseRenderElementParams, native_to_base_ui, use_render_element,
};
use leptos_ui_utils::use_controlled::{UseControlledProps, use_controlled};
use leptos_ui_utils::use_merged_refs::InputRef;
use web_sys::wasm_bindgen::JsCast;

/// `ToggleState` (`packages/react/src/toggle/Toggle.tsx:75-78`): the state object
/// feeding both render paths and the state→data-attribute mapping (`data-pressed` /
/// `data-disabled`, `packages/react/src/toggle/ToggleDataAttributes.ts:4,8` — produced
/// by `getStateAttributesProps`' truthiness mapping, not by importing the constants).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToggleState {
    /// The `disabled` member (`:76`).
    pub disabled: bool,
    /// The `pressed` member (`:77`).
    pub pressed: bool,
}

impl ToggleState {
    /// The `serde_json` state map `useRenderElement`'s state mapping consumes —
    /// `state: ToggleState` (`:75-78`) as the dynamic record the mapping walks
    /// (`data-pressed`/`data-disabled` emerge from the truthiness mapping).
    pub fn to_state_map(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert("disabled".to_string(), json!(self.disabled));
        map.insert("pressed".to_string(), json!(self.pressed));
        map
    }
}

/// The group context Toggle consumes
/// (`packages/react/src/toggle-group/ToggleGroupContext.ts:6-19`). Standalone rendering
/// never touches it (implementation.md, "Cross-component units"); its *presence or
/// absence* is the unit's single branch point (implementation.md, "Context
/// providers/consumers"). `set_group_value` is `Option`-typed because upstream's
/// `groupContext?.setGroupValue?.(...)` tolerates a context without a committer
/// (`:97`).
#[derive(Clone)]
pub struct ToggleGroupContext {
    /// The group's current value array (`:8-9`).
    pub value: std::sync::Arc<Vec<String>>,
    /// The group's gated committer (`:10-12`): `(value, nextPressed, details)`.
    pub set_group_value: Option<
        std::sync::Arc<dyn Fn(&str, bool, &BaseUIChangeEventDetails<(), MouseEvent>) + Send + Sync>,
    >,
    /// Group-wide disabled (`:13`).
    pub disabled: bool,
    /// Dev-warning gate (`:14-18`).
    pub is_value_initialized: bool,
}

/// Reads the ToggleGroup context if a provider is in scope — upstream's
/// `useToggleGroupContext()` (`packages/react/src/toggle/Toggle.tsx:45`), a plain
/// `useContext` over a context whose default is `undefined`
/// (`packages/react/src/toggle-group/ToggleGroupContext.ts:21-23`): absence (`None`)
/// is the standalone signal. Must be called inside a reactive owner.
pub fn use_toggle_group_context() -> Option<ToggleGroupContext> {
    use_context::<ToggleGroupContext>()
}

/// The `onPressedChange` callback type (`packages/react/src/toggle/Toggle.tsx:33`):
/// `(pressed, eventDetails)`, the second argument exposing `cancel()` through
/// [`BaseUIChangeEventDetails`].
pub type OnPressedChange =
    std::sync::Arc<dyn Fn(bool, &BaseUIChangeEventDetails<(), MouseEvent>) + Send + Sync>;

/// The consumer's handler members of the `elementProps` rest (`:107`), typed for the
/// Base UI event pipeline (the wrapped dispatch lets a consumer call
/// `preventBaseUIHandler()` to veto Toggle's own machine — the mergeProps prevention
/// contract).
#[derive(Clone, Default)]
pub struct ToggleHandlers {
    /// The consumer's `onClick` (`:107`).
    pub on_click: Option<Rc<dyn Fn(&BaseUIEvent<MouseEvent>)>>,
}

/// The Toggle component props — upstream's destructured
/// `Toggle.Props<Value>` (`packages/react/src/toggle/Toggle.tsx:28-41`) with the
/// documented defaults (the manual `Default` impl carries `native_button: true` and
/// `default_pressed: false` — the `#[derive]` would give the wrong `native_button`).
pub struct ToggleProps {
    /// `pressed` (`:34`) — the controlled value; `None` while uncontrolled.
    pub pressed: Option<bool>,
    /// `defaultPressed` (`:30` — upstream default `false`).
    pub default_pressed: bool,
    /// `disabled` (`:31` — upstream default `false`).
    pub disabled: bool,
    /// `onPressedChange` (`:33`).
    pub on_pressed_change: Option<OnPressedChange>,
    /// `value` (`:37`) — the group-membership key, normalized by
    /// `useBaseUiId(valueProp || undefined)` (`:43-44`).
    pub value: Option<String>,
    /// `nativeButton` (`:38` — upstream default `true`).
    pub native_button: bool,
    /// `className`/`style`/`render` (`:29,35,39`) — through the
    /// [`UseRenderElementComponentProps`] vocabulary.
    pub render_class_style: UseRenderElementComponentProps,
    /// The `...elementProps` rest (`:40`) — non-handler attributes spread onto the
    /// element, `(name, static value)`. Handler keys belong to [`ToggleHandlers`].
    pub element_attributes: Vec<(String, String)>,
    /// The consumer's handler members of the rest (`:107`).
    pub handlers: ToggleHandlers,
}

impl Default for ToggleProps {
    fn default() -> Self {
        Self {
            pressed: None,
            default_pressed: false,
            disabled: false,
            on_pressed_change: None,
            value: None,
            native_button: true,
            render_class_style: UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            handlers: ToggleHandlers::default(),
        }
    }
}

/// The falsy-value normalization arm (`:43-44` — `valueProp || undefined`): an empty
/// string normalizes to `None` exactly like the falsy-string arm of `||`.
#[cfg(test)]
pub(crate) fn resolve_value(value: Option<String>) -> Option<String> {
    match value {
        Some(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

#[cfg(not(test))]
fn resolve_value(value: Option<String>) -> Option<String> {
    match value {
        Some(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

/// Builds the Toggle element description — upstream's `Toggle` body
/// (`packages/react/src/toggle/Toggle.tsx:24-140`) up to and including the
/// `useRenderElement`/`CompositeItem` branch, without materializing a DOM node (the
/// description/element split the view bridge materializes). Must be called inside a
/// reactive owner.
///
/// Returns the rendered element description. `None` mirrors upstream's
/// `useRenderElement` yielding `null` (`:112`'s `enabled: !groupContext`) — which only
/// happens on the grouped path.
pub fn toggle_element(props: ToggleProps) -> Option<RenderedElement> {
    let ToggleProps {
        pressed: pressed_prop,
        default_pressed,
        disabled: disabled_prop,
        on_pressed_change,
        value: value_prop,
        native_button,
        render_class_style,
        element_attributes,
        handlers: consumer_handlers,
    } = props;

    // `const groupContext = useToggleGroupContext()` (`:45`).
    let group_context = use_toggle_group_context();

    // `const value = useBaseUiId(valueProp || undefined)` (`:43-44`): the generated id
    // exists from first render on the client, so `value` is always truthy — the
    // `if (value)` gate (`:96`) only matters on upstream's React ≤17 fallback path
    // (packages/utils/src/useId.ts:8-22).
    let value = use_base_ui_id(RwSignal::new(resolve_value(value_prop.clone())));
    let value_string = value.get_untracked();

    // `const disabled = (disabledProp || groupContext?.disabled) ?? false` (`:48`).
    let disabled = disabled_prop || group_context.as_ref().map(|g| g.disabled).unwrap_or(false);

    // The dev-only warning (`:50-61`), armed only under a group context with the
    // value-initialized gate.
    if cfg!(debug_assertions) {
        if let Some(group) = &group_context {
            if value_prop.is_none() && group.is_value_initialized {
                leptos_ui_utils::error::error().log(&[
                    "A `<Toggle>` component rendered in a `<ToggleGroup>` has no explicit `value` prop.",
                    "This will cause issues between the Toggle Group and Toggle values.",
                    "Provide the `<Toggle>` with a `value` prop matching the `<ToggleGroup>` values prop type.",
                ]);
            }
        }
    }

    // The pressed-state duality (`:63-68`). The `controlled` argument is
    // `groupContext ? value !== undefined && groupValue.indexOf(value) > -1 :
    // pressedProp` — always a boolean under a group (always controlled, by the
    // group), `None` while uncontrolled standalone. The mode is captured once
    // (packages/utils/src/useControlled.ts:41), so the match arm is the mode fix.
    let (pressed, set_pressed_state) = match &group_context {
        Some(group) => {
            let member = value_prop.is_some() && group.value.iter().any(|v| v == &value_string);
            use_controlled(UseControlledProps::new(
                RwSignal::new(Some(member)),
                RwSignal::new(default_pressed),
                "Toggle",
            ))
        }
        None => use_controlled(UseControlledProps::new(
            RwSignal::new(pressed_prop),
            RwSignal::new(default_pressed),
            "Toggle",
        )),
    };
    // `state: 'pressed'` (`:67`) is the developer label in the props above.

    // `const { getButtonProps, buttonRef } = useButton({ disabled, native:
    // nativeButton })` (`:70-73`).
    let button = use_button(UseButtonParams {
        disabled: RwSignal::new(disabled),
        focusable_when_disabled: None,
        tab_index: 0,
        native: native_button,
        composite: None,
    });

    // `const state: ToggleState = { disabled, pressed }` (`:75-78`) — the record the
    // state mapping walks. `pressed` is the current value for this evaluation.
    let state_map = {
        let mut map = serde_json::Map::new();
        map.insert("disabled".to_string(), json!(disabled));
        map.insert("pressed".to_string(), json!(pressed.get_untracked()));
        map
    };

    // Toggle's base props bag — `aria-pressed` (`:83`) and the onClick transition
    // machine (`:84-105`), the first bag of the props array.
    let base_bag = {
        let pressed_read = pressed.clone();
        let group_context = group_context.clone();
        let value_for_click = value_string.clone();

        let aria_pressed_fn: ElementAttributeFn =
            Rc::new(move || Some(pressed_read.get_untracked().to_string()));

        let on_click = {
            Rc::new(move |event: &BaseUIEvent<MouseEvent>| {
                // `nextPressed = !pressed` from the render-time snapshot (`:85`); the
                // untracked read is that snapshot's analog.
                let next_pressed = !pressed_read.get_untracked();

                // One shared details object (`:86`), handed to both consumers.
                let details: BaseUIChangeEventDetails<(), MouseEvent> =
                    BaseUIChangeEventDetails::new(reasons::NONE, event.inner().clone(), None, ());

                // `onPressedChange` runs before any commit (`:90`) — a cancel here
                // vetoes the group commit and the local change alike.
                if let Some(callback) = &on_pressed_change {
                    callback(next_pressed, &details);
                }

                if details.is_canceled() {
                    return;
                }

                // Gate 2: the group commit (`:96-98`).
                if !value_for_click.is_empty() {
                    if let Some(commit) = group_context
                        .as_ref()
                        .and_then(|g| g.set_group_value.as_ref())
                    {
                        commit(&value_for_click, next_pressed, &details);
                    }
                }

                // Gate 3: the reverse veto (`:100-102`).
                if details.is_canceled() {
                    return;
                }

                // The commit — real write (uncontrolled), no-op (controlled)
                // (`:104`).
                set_pressed_state(leptos_ui_utils::use_controlled::SetValueAction::Value(
                    next_pressed,
                ));
            })
        };

        RenderElementProps {
            handlers: RenderElementHandlers {
                on_click: Some(on_click),
                attributes: vec![("aria-pressed".to_string(), aria_pressed_fn)],
                ..RenderElementHandlers::default()
            },
            ..RenderElementProps::default()
        }
    };

    // The `...elementProps` rest bag (`:107`) — the consumer's handlers and static
    // attributes, the second bag.
    let element_bag = RenderElementProps {
        handlers: RenderElementHandlers {
            on_click: consumer_handlers.on_click.clone(),
            attributes: element_attributes
                .iter()
                .map(|(name, value)| {
                    let value = value.clone();
                    (
                        name.clone(),
                        Rc::new(move || Some(value.clone())) as ElementAttributeFn,
                    )
                })
                .collect(),
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    };

    // `getButtonProps()`'s composed bag — the third bag (`:108`). Toggle passes no
    // external button handlers (`:108` is the bare getter), so the composition
    // contributes the internal disabled-guard handlers and the `{ type: 'button' }` /
    // `{ role: 'button' }` + focusableWhenDisabled attribute members. The bag's
    // native-typed handler slots adapt into the element bag's `BaseUIEvent`-typed
    // slots (`native_to_base_ui` — the `wrapEventHandler` wrapping).
    let button_props = (button.get_button_props)(ButtonExternalHandlers::default());
    let button_bag = RenderElementProps {
        handlers: RenderElementHandlers {
            on_click: button_props
                .handlers
                .on_click
                .clone()
                .map(native_to_base_ui),
            on_mouse_down: button_props
                .handlers
                .on_mouse_down
                .clone()
                .map(native_to_base_ui),
            on_key_down: button_props.handlers.on_key_down.clone(),
            on_key_up: button_props.handlers.on_key_up.clone(),
            on_pointer_down: button_props
                .handlers
                .on_pointer_down
                .clone()
                .map(native_to_base_ui),
            attributes: button_props.attributes.clone(),
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    };

    // The props array `[baseProps, elementProps, getButtonProps]` (`:81-109`),
    // merging left-to-right with later handlers running first (the `mergePropsN`
    // rule, `packages/react/src/merge-props/mergeProps.ts:229-244`) — so the
    // disabled guard runs before the consumer's handler and before Toggle's machine.
    let mut props_bags = vec![
        PropsSource::Static(base_bag),
        PropsSource::Static(element_bag),
    ];
    props_bags.push(PropsSource::Static(button_bag));

    // The refs: `[buttonRef, forwardedRef]` (`:80`). `buttonRef` fires through the
    // forked ref at materialization; the consumer's forwarded ref has no Rust slot
    // in the description-based model (the materializer's ref callback is the only
    // observer), so the fork carries the hook ref alone.
    let refs: Vec<InputRef<web_sys::Element>> = {
        let button_ref = button.button_ref.clone();
        vec![InputRef::Callback(
            Rc::new(move |instance: Option<&web_sys::Element>| {
                button_ref(
                    instance
                        .map(|element| element.clone().unchecked_into::<web_sys::HtmlElement>()),
                );
                None
            }) as _,
        )]
    };

    // The render-path switch (`:111-116` + `:125-138`).
    match &group_context {
        // Standalone: `useRenderElement('button', …, { enabled: true, … })` —
        // `<button type="button" {...props}>` (`useRenderElement.tsx:232-235`).
        None => use_render_element(
            "button",
            render_class_style,
            UseRenderElementParams {
                enabled: true,
                state: &state_map,
                refs,
                props: props_bags,
                state_attributes_mapping: None,
            },
        ),
        // Grouped: `<CompositeItem tag="button" …>` (`:125-138`) — the same state
        // and the same props array, composite props added on top (`:154-155`).
        // `itemMetadata` (`:120-123`): memoized
        // `{ disabled, focusableWhenDisabled: false }` for Toolbar's
        // `disabledIndices`.
        Some(_) => composite_item(CompositeItemComponentProps {
            metadata: Some(ToggleItemMetadata {
                disabled,
                focusable_when_disabled: false,
            }),
            render_class_style,
            tag: "button".to_string(),
            state: state_map,
            state_attributes_mapping: None,
            refs,
            props: props_bags,
            element_props: RenderElementProps::default(),
        }),
    }
}

/// `ToolbarRoot.ItemMetadata` (`packages/react/src/toggle/Toggle.tsx:120-123`):
/// memoized `{ disabled, focusableWhenDisabled: false }` — metadata Toolbar reads to
/// compute `disabledIndices`, passed through `CompositeItem`'s `metadata`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToggleItemMetadata {
    /// The `disabled` member (`:121`).
    pub disabled: bool,
    /// Always `false` for Toggle (`:122`): "A disabled toggle is natively disabled
    /// and cannot hold roving focus."
    pub focusable_when_disabled: bool,
}
