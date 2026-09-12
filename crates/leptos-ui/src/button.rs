//! Port of the Base UI Button — the `library: button` TODO item
//! (`specs/library/button/behavior.md`, `specs/library/button/implementation.md`).
//!
//! Upstream's structural facts this port follows (implementation.md):
//!
//! - **The component contains no state machine and no `useState`**
//!   ("State machine / hooks used", implementation.md:12-17): the entire runtime is
//!   two hook calls plus a state descriptor — `useButton` at
//!   `packages/react/src/button/Button.tsx:27-31` and `useRenderElement` at
//!   `:37-41`, over the plain `{ disabled }` state literal at `:33-35`. The port is
//!   therefore a configuration of the already-ported
//!   [`leptos_ui_internals::use_button`] and
//!   [`leptos_ui_internals::use_render_element`] engines (the dialog-facade
//!   precedent: the unit's own ~40 lines contribute props narrowing and bag order,
//!   everything else is the shared machinery).
//! - **The only observable state is the caller's `disabled` prop** — the port
//!   carries it as a reactive source ([`RwSignal<bool>`]) because
//!   [`use_button`]'s attribute policy reads it lazily and its `updateDisabled`
//!   effect tracks it (upstream closes over the latest render's value; the
//!   use_press_and_hold convention documented in the use_button module docs).
//! - **Bag order is the entire composition contract**: upstream
//!   `useRenderElement('button', …, { props: [elementProps, getButtonProps] })`
//!   (`Button.tsx:37-41`) merges `[elementProps, getButtonProps]` left-to-right,
//!   and `mergeProps`' later bags' handlers run first
//!   (`packages/react/src/merge-props/mergeProps.ts:229-244`) — so the internal
//!   disabled-guard handlers run *before* the consumer's handlers, and user
//!   attribute values still win because they sit in the earlier bag
//!   (`useButton.ts:228` — `otherExternalProps` last, the `type="submit"`
//!   override). The port reproduces exactly that order: `[element_bag,
//!   button_bag]` (`PropsSource` fold, the toggle/mod.rs precedent).
//! - **`data-disabled` is derived, not literal** (implementation.md untested item
//!   3): `ButtonDataAttributes.tsx` is dead code as a mechanism; the attribute
//!   emerges from `getStateAttributesProps`' truthiness mapping over the
//!   `{ disabled }` state record
//!   (`packages/react/src/internals/getStateAttributesProps.ts:24-28`). The port
//!   walks the same generic mapping through [`use_render_element`]'s state map.
//! - **The composite branch is a `useButton`-level concern, not a Button fixture**
//!   (implementation.md untested item 1): `Button` never passes the `composite`
//!   parameter (`useButton.ts`'s inference is purely ambient via
//!   `CompositeRootContext`), so the port leaves `composite: None` and lets the
//!   optional-context accessor decide — already suite-tested at the
//!   `use_button` level (its nested-bag and composite wasm tests).
//! - **Dev-only tag-mismatch diagnostics** (implementation.md untested item 2) run
//!   inside `use_button` at ref-attach behind `cfg!(debug_assertions)` — nothing
//!   for this facade to add.
//! - **The "disabled while focused retains focus" behavior** (behavior.md Edge
//!   cases) has no implementation code upstream (implementation.md reverse-gap
//!   item 6) — incidental host behavior, not ported logic.

use reactive_graph::signal::RwSignal;

use leptos_ui_internals::floating_ui::element_props::{ElementAttributeFn, ElementEventHandler};
use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::use_button::{ButtonExternalHandlers, UseButtonParams, use_button};
use leptos_ui_internals::use_render_element::{
    RenderElementHandlers, RenderElementProps, RenderedElement, UseRenderElementComponentProps,
    UseRenderElementParams, native_to_base_ui, use_render_element,
};
use leptos_ui_utils::use_merged_refs::InputRef;

/// `ButtonState` (`packages/react/src/button/Button.tsx:44-49`): the only member is
/// `disabled`, and it exists solely to drive `data-disabled`
/// (implementation.md:104-110).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonState {
    /// The `disabled` member (`:46-48`).
    pub disabled: bool,
}

impl ButtonState {
    /// The `serde_json` state map `useRenderElement`'s generic state→attribute
    /// mapping consumes (`data-disabled` emerges from the truthiness mapping —
    /// implementation.md untested item 3).
    pub fn to_state_map(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "disabled".to_string(),
            serde_json::Value::Bool(self.disabled),
        );
        map
    }
}

/// The consumer's handler members of the `...elementProps` rest
/// (`packages/react/src/button/Button.tsx:17`), typed for the Base UI event
/// pipeline — the wrapped dispatch lets a consumer call `preventBaseUIHandler()`
/// to veto the internal activation machinery (the mergeProps prevention contract;
/// behavior.md "Events" rows over `onClick`/`onMouseDown`/`onPointerDown`/
/// `onKeyDown`/`onMouseMove`).
#[derive(Clone, Default)]
pub struct ButtonHandlers {
    /// The consumer's `onClick` (`Button.test.tsx:107-113`).
    pub on_click: Option<ElementEventHandler<web_sys::MouseEvent>>,
    /// The consumer's `onMouseDown`.
    pub on_mouse_down: Option<ElementEventHandler<web_sys::MouseEvent>>,
    /// The consumer's `onMouseMove` (`Button.test.tsx:219-224` — the
    /// focusableWhenDisabled mousemove-still-fires row).
    pub on_mouse_move: Option<ElementEventHandler<web_sys::MouseEvent>>,
    /// The consumer's `onKeyDown` — receives the wrapped event so it runs inside
    /// the shared prevention-mark protocol.
    pub on_key_down: Option<
        ElementEventHandler<leptos_ui_internals::types::BaseUIEvent<web_sys::KeyboardEvent>>,
    >,
    /// The consumer's `onKeyUp`.
    pub on_key_up: Option<
        ElementEventHandler<leptos_ui_internals::types::BaseUIEvent<web_sys::KeyboardEvent>>,
    >,
    /// The consumer's `onPointerDown`.
    pub on_pointer_down: Option<ElementEventHandler<web_sys::PointerEvent>>,
}

/// The Button component props — upstream's destructured `Button.Props`
/// (`packages/react/src/button/Button.tsx:16-25`) with the documented defaults.
pub struct ButtonProps {
    /// `disabled` (`:19` — upstream default `false`).
    pub disabled: bool,
    /// `focusableWhenDisabled` (`:20` — upstream default `false`).
    pub focusable_when_disabled: bool,
    /// `nativeButton` (`:21` — upstream default `true`).
    pub native_button: bool,
    /// `className`/`style`/`render` (`:16,22,23`) — through the
    /// [`UseRenderElementComponentProps`] vocabulary.
    pub render_class_style: UseRenderElementComponentProps,
    /// The `...elementProps` rest (`:17`) — non-handler attributes spread onto the
    /// element, `(name, static value)`. User values win over the internal bag's
    /// members (the earlier-bag-wins rule, `useButton.ts:228` — the
    /// `<Button type="submit">` override, `Button.spec.tsx:4`).
    pub element_attributes: Vec<(String, String)>,
    /// The consumer's handler members of the rest (`Button.test.tsx:107-113`,
    /// `:219-224`).
    pub handlers: ButtonHandlers,
}

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            disabled: false,
            focusable_when_disabled: false,
            native_button: true,
            render_class_style: UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            handlers: ButtonHandlers::default(),
        }
    }
}

/// Builds the Button element description — upstream's `Button` body
/// (`packages/react/src/button/Button.tsx:14-42`) up to and including
/// `useRenderElement`, without materializing a DOM node (the description/element
/// split the view bridge materializes). Must be called inside a reactive owner
/// (the `use_button` updateDisabled effect registers there).
///
/// Always returns `Some` — `Button`'s `useRenderElement` call has no `enabled`
/// gate (it is a leaf with no conditional rendering).
pub fn button_element(props: ButtonProps) -> Option<RenderedElement> {
    let ButtonProps {
        disabled,
        focusable_when_disabled,
        native_button,
        render_class_style,
        element_attributes,
        handlers: consumer_handlers,
    } = props;

    // `const { getButtonProps, buttonRef } = useButton({ disabled,
    // focusableWhenDisabled, native: nativeButton })` (`:27-31`). `disabled` rides
    // a reactive source (the use_button module docs' convention); the rest are
    // component-shape props.
    let button = use_button(UseButtonParams {
        disabled: RwSignal::new(disabled),
        focusable_when_disabled: Some(focusable_when_disabled),
        tab_index: 0,
        native: native_button,
        composite: None,
    });

    // `const state: ButtonState = { disabled }` (`:33-35`) — the record the
    // generic mapping walks for `data-disabled`.
    let state_map = ButtonState { disabled }.to_state_map();

    // The `...elementProps` rest bag (`:17`) — the consumer's handlers and static
    // attributes, the FIRST bag of the props array.
    let element_bag = RenderElementProps {
        handlers: RenderElementHandlers {
            on_click: consumer_handlers.on_click.clone().map(native_to_base_ui),
            on_mouse_down: consumer_handlers
                .on_mouse_down
                .clone()
                .map(native_to_base_ui),
            on_mouse_move: consumer_handlers
                .on_mouse_move
                .clone()
                .map(native_to_base_ui),
            on_key_down: consumer_handlers.on_key_down.clone(),
            on_key_up: consumer_handlers.on_key_up.clone(),
            on_pointer_down: consumer_handlers
                .on_pointer_down
                .clone()
                .map(native_to_base_ui),
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

    // `getButtonProps()`'s composed bag — the SECOND (later) bag (`:38`). Button
    // passes no external button handlers (upstream passes the getter bare into the
    // props array; `useRenderElement` resolves it against the merged-so-far
    // props), so the composition contributes the internal disabled-guard handlers
    // and the `{ type: 'button' }`/`{ role: 'button' }` +
    // focusableWhenDisabled attribute members. The bag's native-typed handler
    // slots adapt into the element bag's `BaseUIEvent`-typed slots
    // (`native_to_base_ui` — the `wrapEventHandler` wrapping), exactly the
    // toggle/mod.rs composition.
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

    // The props array `[elementProps, getButtonProps]` (`:38`), merging
    // left-to-right with later handlers running first (the `mergePropsN` rule,
    // `packages/react/src/merge-props/mergeProps.ts:229-244`) — so the internal
    // disabled-guard handlers run before the consumer's handlers, while the
    // consumer's plain attributes win over the internal ones (`useButton.ts:228`).
    let props_bags = vec![
        PropsSource::Static(element_bag),
        PropsSource::Static(button_bag),
    ];

    // The refs: `[forwardedRef, buttonRef]` (`:39`). `buttonRef` fires through the
    // forked ref at materialization; the consumer's forwarded ref has no Rust slot
    // in the description-based model (the materializer's ref callback is the only
    // observer), so the fork carries the hook ref alone — the toggle precedent.
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

    // `useRenderElement('button', componentProps, …)` (`:37-41`) — the default tag
    // is a native `<button>` (behavior.md "DOM structure": `refInstanceof:
    // window.HTMLButtonElement`), whose `type="button"` default the render engine
    // forces (`useRenderElement.tsx:232-240`).
    use_render_element(
        "button",
        render_class_style,
        UseRenderElementParams {
            enabled: true,
            state: &state_map,
            refs,
            props: props_bags,
            state_attributes_mapping: None,
        },
    )
}

use std::rc::Rc;

use web_sys::wasm_bindgen::JsCast;
