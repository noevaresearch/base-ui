//! Port of `packages/react/src/internals/use-button/useButton.ts` — the shared button
//! hook behind Button, Menu.Trigger, Dialog.Trigger, and every other clickable part
//! (`specs/library/internals/behavior.md`, "useButton" rows; implementation spec,
//! "useButton").
//!
//! Two behaviors carry the unit: the attribute policy is delegated to
//! [`crate::use_focusable_when_disabled`] (the `useButton.ts:28-34` composition), and
//! the keyboard click synthesis dispatches a synthetic untrusted `click` through
//! [`crate::dispatch_click_with_modifiers`] — Space/Enter never call `element.click()`
//! (`packages/react/src/utils/dispatchClickWithModifiers.ts:10-18`), so the modifier
//! state survives and `detail: 0` marks the click as keyboard-generated.
//!
//! ## Rust adaptations
//!
//! - `getButtonProps(externalProps)` (`useButton.ts:91-232`) merges five props bags —
//!   the internal handler bag, `{ type: 'button' }`/`{ role: 'button' }`, the
//!   focusableWhenDisabled policy, and the remaining external props — through
//!   `mergeProps`. The port keeps the handler half of that merge in
//!   [`ButtonHandlers`]-typed slots (the `ElementEventHandler` vocabulary) and hands
//!   the attribute half to the view layer ([`ButtonProps::attributes`]), per the
//!   architecture decision that Leptos's native attribute system replaces
//!   `mergeProps`' non-handler members (`specs/architecture.md`, "Prop / class / style
//!   merging (mergeProps)"). The bag order's observable consequence — *later bags run
//!   first* (`packages/react/src/merge-props/mergeProps.ts:229-244`: `theirHandler`
//!   runs, then `ourHandler` unless the prevention mark is set) — is reproduced
//!   exactly: the policy's keydown runs before the internal keydown, gated by the
//!   shared [`BaseUIEvent`] mark.
//! - The five destructured external handlers (`useButton.ts:93-100`) become the
//!   [`ButtonExternalHandlers`] argument. Keydown/keyup handlers receive the wrapped
//!   [`BaseUIEvent<KeyboardEvent>`] — upstream's `makeEventPreventable` attachment —
//!   so a consumer's `preventBaseUIHandler()` reaches the internal gates
//!   (`useButton.ts:121-125,198`). The key slots are typed the same way in the
//!   returned bag, which is what keeps [`ButtonProps`] nestable: passing one
//!   `useButton`'s bag as another's external handlers shares the single wrapped event
//!   and therefore the single prevention mark, the mechanism behind the "fires a
//!   single click for nested non-native composite buttons" behavior
//!   (`useButton.ts:147-148` sets the mark before dispatching, and the outer call
//!   bails on it at `:123`).
//! - `disabled` is a reactive source read untracked inside the handlers (upstream
//!   closes over the latest render's value; the `usePressAndHold` convention).
//!   `native`/`composite`/`tabIndex`/`focusableWhenDisabled` are static — in the
//!   port's run-once component model they are component-shape props, not reactive
//!   state.
//! - `updateDisabled` (`useButton.ts:72-89`) stays a shared `Rc` closure, re-run from
//!   a reactive-graph [`Effect`] tracking the `disabled` source (upstream's
//!   `useIsoLayoutEffect(updateDisabled, [updateDisabled])` — the effect re-runs
//!   whenever the closure's inputs change) and invoked directly from `button_ref` on
//!   every ref attach (`useButton.ts:234-237`).
//! - The dev-only tag-mismatch warnings (`useButton.ts:36-66`) run at ref-attach time
//!   behind `cfg!(debug_assertions)` (upstream's `NODE_ENV !== 'production'` gate —
//!   the port has no `SafeReact.captureOwnerStack` to append). They log through the
//!   `error` util, whose `Base UI: ` prefix completes the messages the upstream tests
//!   assert.
//! - The composite inference `composite ?? (useCompositeRootContext(true) !==
//!   undefined)` (`useButton.ts:25-26`) uses the optional context accessor — `None`
//!   outside a root (`packages/react/src/internals/composite/root/CompositeRootContext.ts:21-32`).
//! - `'use client'` (`useButton.ts:1`) is N/A — no React Server Components boundary.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::effect::Effect;
use reactive_graph::traits::{Get, GetUntracked};
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, EventTarget, HtmlButtonElement, HtmlElement, KeyboardEvent, MouseEvent, PointerEvent};

use crate::composite_root_context::use_composite_root_context;
use crate::dispatch_click_with_modifiers::dispatch_click_with_modifiers;
use crate::floating_ui::element_props::{ElementAttributeFn, ElementEventHandler};
use crate::types::BaseUIEvent;
use crate::use_focusable_when_disabled::{UseFocusableWhenDisabledParams, use_focusable_when_disabled};
use leptos_ui_utils::error::error;
use leptos_ui_utils::merge_cleanups::{CleanupFn, merge_cleanups};

/// The five destructured external handlers (`useButton.ts:93-100`), typed for the
/// Base UI event pipeline (see the module docs). Every slot is upstream's optional
/// member; `None` is omitted.
#[derive(Clone, Default)]
pub struct ButtonExternalHandlers {
    /// `onClick` (`:94`).
    pub on_click: Option<ElementEventHandler<MouseEvent>>,
    /// `onMouseDown` (`:95`).
    pub on_mouse_down: Option<ElementEventHandler<MouseEvent>>,
    /// `onKeyDown` (`:96`) — receives the [`BaseUIEvent`] wrapper so the consumer can
    /// call `preventBaseUIHandler()` / read the mark.
    pub on_key_down: Option<ElementEventHandler<BaseUIEvent<KeyboardEvent>>>,
    /// `onKeyUp` (`:97`) — same wrapper contract as [`Self::on_key_down`].
    pub on_key_up: Option<ElementEventHandler<BaseUIEvent<KeyboardEvent>>>,
    /// `onPointerDown` (`:98`).
    pub on_pointer_down: Option<ElementEventHandler<PointerEvent>>,
}

/// The handler half of `getButtonProps`' return — the five slots with the internal
/// behavior composed around the external handlers. The slots' types match
/// [`ButtonExternalHandlers`] one-to-one, which is what makes nested `useButton` bags
/// compose while sharing one prevention mark (see the module docs).
#[derive(Clone, Default)]
pub struct ButtonHandlers {
    /// `onClick` — prevents and ignores while disabled (`useButton.ts:104-110`).
    pub on_click: Option<ElementEventHandler<MouseEvent>>,
    /// `onMouseDown` — passes through only while enabled (`useButton.ts:111-115`).
    pub on_mouse_down: Option<ElementEventHandler<MouseEvent>>,
    /// `onKeyDown` — the full keydown pipeline (`useButton.ts:116-176`).
    pub on_key_down: Option<ElementEventHandler<BaseUIEvent<KeyboardEvent>>>,
    /// `onKeyUp` — the full keyup pipeline (`useButton.ts:177-217`).
    pub on_key_up: Option<ElementEventHandler<BaseUIEvent<KeyboardEvent>>>,
    /// `onPointerDown` — prevents while disabled (`useButton.ts:218-224`).
    pub on_pointer_down: Option<ElementEventHandler<PointerEvent>>,
}

/// The port's `getButtonProps` return value: the composed handler bag plus the
/// attribute members the hook computes (the `{ type: 'button' }`/`{ role: 'button' }`
/// bag at `useButton.ts:226` and the focusableWhenDisabled policy's attributes).
/// External non-handler attributes do not travel through this value — the view layer
/// composes them through Leptos's native attribute system (see the module docs).
#[derive(Clone, Default)]
pub struct ButtonProps {
    /// The five composed handler slots.
    pub handlers: ButtonHandlers,
    /// `(name, value)` attribute members; the closures resolve lazily so the
    /// `disabled`-dependent members re-derive at read time (the
    /// `ElementHandlers::attributes` convention). `None` is upstream's omitted prop.
    pub attributes: Vec<(String, ElementAttributeFn)>,
}

impl ButtonProps {
    /// Attaches every filled handler slot to `target` as a bubble-phase native
    /// listener — the composition work upstream's JSX spread performs when a consumer
    /// spreads `getButtonProps()` onto the rendered element. The key slots wrap the
    /// native event in the shared [`BaseUIEvent`] wrapper here, exactly once per
    /// dispatch (see the module docs). Returns one merged cleanup — the
    /// `ElementHandlers::attach_to` convention.
    pub fn attach_to(&self, target: &EventTarget) -> Option<CleanupFn> {
        let mut cleanups: Vec<Option<CleanupFn>> = Vec::new();

        macro_rules! attach {
            ($slot:expr, $event_name:literal, $event_type:ty, $wrap:expr) => {
                if let Some(handler) = &$slot {
                    let handler = Rc::clone(handler);
                    let unsubscribe = leptos_ui_utils::add_event_listener(
                        target,
                        $event_name,
                        move |event: &web_sys::Event| {
                            if let Some(typed) = event.dyn_ref::<$event_type>() {
                                $wrap(&handler, typed);
                            }
                        },
                    );
                    cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
                }
            };
        }

        macro_rules! attach_direct {
            ($slot:expr, $event_name:literal, $event_type:ty) => {
                attach!(
                    $slot,
                    $event_name,
                    $event_type,
                    |handler: &ElementEventHandler<$event_type>, event: &$event_type| handler(
                        event
                    )
                );
            };
        }

        macro_rules! attach_base_ui {
            ($slot:expr, $event_name:literal, $event_type:ty) => {
                attach!(
                    $slot,
                    $event_name,
                    $event_type,
                    |handler: &ElementEventHandler<BaseUIEvent<$event_type>>,
                     event: &$event_type| {
                        let wrapped = BaseUIEvent::new(event.clone());
                        handler(&wrapped);
                    }
                );
            };
        }

        attach_direct!(self.handlers.on_click, "click", MouseEvent);
        attach_direct!(self.handlers.on_mouse_down, "mousedown", MouseEvent);
        attach_base_ui!(self.handlers.on_key_down, "keydown", KeyboardEvent);
        attach_base_ui!(self.handlers.on_key_up, "keyup", KeyboardEvent);
        attach_direct!(self.handlers.on_pointer_down, "pointerdown", PointerEvent);

        (!self.is_empty()).then(|| Box::new(merge_cleanups(cleanups)) as CleanupFn)
    }

    /// Whether no handler slot is filled.
    pub fn is_empty(&self) -> bool {
        self.handlers.on_click.is_none()
            && self.handlers.on_mouse_down.is_none()
            && self.handlers.on_key_down.is_none()
            && self.handlers.on_key_up.is_none()
            && self.handlers.on_pointer_down.is_none()
    }
}

/// `UseButtonParameters` (`useButton.ts:264-287`), with upstream's documented
/// defaults noted per field.
pub struct UseButtonParams<D> {
    /// `disabled` (`:269` — upstream default `false`): a reactive source.
    pub disabled: D,
    /// `focusableWhenDisabled` (`:274` — upstream default `undefined`).
    pub focusable_when_disabled: Option<bool>,
    /// `tabIndex` (`:275` — upstream default `0`).
    pub tab_index: i32,
    /// `native` (`:280` — upstream default `true`): whether the component renders a
    /// native `<button>`.
    pub native: bool,
    /// `composite` (`:286` — upstream default inferred from the `CompositeRoot`
    /// context): when `None`, the context decides.
    pub composite: Option<bool>,
}

/// `UseButtonReturnValue` (`useButton.ts:289-303`).
pub struct UseButtonReturnValue {
    /// `getButtonProps` (`:295-297`): resolves the button props around the given
    /// external handlers.
    pub get_button_props: Rc<dyn Fn(ButtonExternalHandlers) -> ButtonProps>,
    /// `buttonRef` (`:302`): the ref callback to pass to the rendered element —
    /// `Some` on attach, `None` on detach (React's callback-ref contract). It is not
    /// part of the props returned by [`Self::get_button_props`].
    pub button_ref: Rc<dyn Fn(Option<HtmlElement>)>,
}

/// `isButtonElement` (`useButton.ts:245-247`).
fn is_button_element(elem: Option<&Element>) -> bool {
    elem.map(|elem| elem.tag_name() == "BUTTON").unwrap_or(false)
}

/// `isValidLinkElement` (`useButton.ts:249-251`): an `<a>` with a non-empty `href`
/// property.
fn is_valid_link_element(elem: &Element) -> bool {
    elem.tag_name() == "A"
        && elem
            .dyn_ref::<web_sys::HtmlAnchorElement>()
            .map(|anchor| !anchor.href().is_empty())
            .unwrap_or(false)
}

/// The dev-only mismatch messages (`useButton.ts:48-52` and `:57-62`). The
/// `SafeReact.captureOwnerStack` suffix has no Rust analog and is omitted.
const NATIVE_EXPECTED_MESSAGE: &str =
    "A component that acts as a button expected a native <button> because the `nativeButton` \
     prop is true. Rendering a non-<button> removes native button semantics, which can impact \
     forms and accessibility. Use a real <button> in the `render` prop, or set `nativeButton` \
     to `false`.";
const NON_NATIVE_EXPECTED_MESSAGE: &str =
    "A component that acts as a button expected a non-<button> because the `nativeButton` \
     prop is false. Rendering a <button> keeps native behavior while Base UI applies \
     non-native attributes and handlers, which can add unintended extra attributes (such \
     as `role` or `aria-disabled`). Use a non-<button> in the `render` prop, or set \
     `nativeButton` to `true`.";

/// `useButton` (`useButton.ts:14-243`). Must be called inside a reactive owner (the
/// updateDisabled effect registers cleanups and the composite-context read requires
/// one).
pub fn use_button<D>(params: UseButtonParams<D>) -> UseButtonReturnValue
where
    D: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
{
    let UseButtonParams {
        disabled,
        focusable_when_disabled,
        tab_index,
        native: is_native_button,
        composite: composite_prop,
    } = params;

    // `const elementRef = React.useRef<HTMLElement | null>(null)` (`:23`).
    let element_ref: Rc<RefCell<Option<HtmlElement>>> = Rc::new(RefCell::new(None));

    // Composite detection is context-inferred (`:25-26`).
    let is_composite_item =
        composite_prop.unwrap_or_else(|| use_composite_root_context().is_some());

    // The disabled/focusability attribute policy (`:28-34`).
    let focusable_when_disabled_props = use_focusable_when_disabled(
        UseFocusableWhenDisabledParams {
            focusable_when_disabled,
            disabled: disabled.clone(),
            composite: is_composite_item,
            tab_index,
            is_native_button,
        },
    );
    let fwd_on_key_down = focusable_when_disabled_props.on_key_down;
    let fwd_attributes = focusable_when_disabled_props.attributes;
    let fwd_sets_disabled = focusable_when_disabled_props.sets_disabled;

    // The latest disabled value, read untracked inside the closures (see the module
    // docs).
    let disabled_now = {
        let disabled = disabled.clone();
        move || disabled.get_untracked()
    };

    // `updateDisabled` (`:72-89`): force-clears a native `disabled` attribute on a
    // disabled *composite* button rendered by another `useButton` — the
    // `<Toolbar.Button disabled render={<Menu.Trigger />} />` case.
    let update_disabled: Rc<dyn Fn()> = {
        let element_ref = Rc::clone(&element_ref);
        let disabled_now = disabled_now.clone();
        Rc::new(move || {
            let element = element_ref.borrow().clone();
            let Some(element) = element else { return };
            if !is_button_element(Some(element.as_ref() as &Element)) {
                return;
            }
            let Some(button) = element.dyn_ref::<HtmlButtonElement>() else {
                return;
            };
            if is_composite_item
                && disabled_now()
                && !fwd_sets_disabled
                && button.disabled()
            {
                button.set_disabled(false);
            }
        })
    };

    // `useIsoLayoutEffect(updateDisabled, [updateDisabled])` (`:89`): the effect
    // tracks the disabled source, standing in for the closure-identity dep array.
    {
        let disabled = disabled.clone();
        let update_disabled = Rc::clone(&update_disabled);
        Effect::new(move |_| {
            disabled.get();
            update_disabled();
        });
    }

    // `buttonRef` (`:234-237`): stores the element and re-runs updateDisabled on
    // every attach. The dev-only tag-mismatch check (`:36-66`) also runs here — in
    // the port's run-once model the ref attach is the only observable trigger
    // upstream's effect had.
    let button_ref: Rc<dyn Fn(Option<HtmlElement>)> = {
        let element_ref = Rc::clone(&element_ref);
        let update_disabled = Rc::clone(&update_disabled);
        Rc::new(move |element| {
            *element_ref.borrow_mut() = element;
            update_disabled();
            if cfg!(debug_assertions) {
                if let Some(element) = element_ref.borrow().as_ref() {
                    let element: &Element = element.as_ref();
                    let is_button_tag = is_button_element(Some(element));
                    if is_native_button && !is_button_tag {
                        error().log(&[NATIVE_EXPECTED_MESSAGE]);
                    } else if !is_native_button && is_button_tag {
                        error().log(&[NON_NATIVE_EXPECTED_MESSAGE]);
                    }
                }
            }
        })
    };

    let get_button_props: Rc<dyn Fn(ButtonExternalHandlers) -> ButtonProps> = {
        let disabled_now = disabled_now.clone();
        let fwd_on_key_down = Rc::clone(&fwd_on_key_down);
        let fwd_attributes = fwd_attributes.clone();
        Rc::new(move |external: ButtonExternalHandlers| {
            let ButtonExternalHandlers {
                on_click: external_on_click,
                on_mouse_down: external_on_mouse_down,
                on_key_down: external_on_key_down,
                on_key_up: external_on_key_up,
                on_pointer_down: external_on_pointer_down,
            } = external;

            // `onClick` (`:104-110`).
            let on_click: ElementEventHandler<MouseEvent> = {
                let disabled_now = disabled_now.clone();
                let external_on_click = external_on_click.clone();
                Rc::new(move |event: &MouseEvent| {
                    if disabled_now() {
                        event.prevent_default();
                        return;
                    }
                    if let Some(external) = external_on_click.as_ref() {
                        external(event);
                    }
                })
            };

            // `onMouseDown` (`:111-115`).
            let on_mouse_down: ElementEventHandler<MouseEvent> = {
                let disabled_now = disabled_now.clone();
                let external_on_mouse_down = external_on_mouse_down.clone();
                Rc::new(move |event: &MouseEvent| {
                    if !disabled_now() {
                        if let Some(external) = external_on_mouse_down.as_ref() {
                            external(event);
                        }
                    }
                })
            };

            // `onKeyDown` (`:116-176`). Bag order — the policy's keydown runs before
            // the internal pipeline, gated by the shared prevention mark (see the
            // module docs).
            let on_key_down: ElementEventHandler<BaseUIEvent<KeyboardEvent>> = {
                let disabled_now = disabled_now.clone();
                let fwd_on_key_down = Rc::clone(&fwd_on_key_down);
                let external_on_key_down = external_on_key_down.clone();
                Rc::new(move |event: &BaseUIEvent<KeyboardEvent>| {
                    fwd_on_key_down(event);
                    if event.base_ui_handler_prevented() {
                        return;
                    }

                    if disabled_now() {
                        return;
                    }

                    if let Some(external) = external_on_key_down.as_ref() {
                        external(event);
                    }
                    if event.base_ui_handler_prevented() {
                        return;
                    }

                    let raw = event.inner();
                    let current_target: Option<Element> = raw
                        .current_target()
                        .and_then(|target| target.dyn_into::<Element>().ok());
                    let Some(current_target) = current_target else {
                        return;
                    };

                    let is_current_target = raw
                        .target()
                        .map(|target| target == *current_target.unchecked_ref::<EventTarget>())
                        .unwrap_or(false);
                    let is_button = is_button_element(Some(&current_target));
                    let is_link = !is_native_button && is_valid_link_element(&current_target);
                    let should_click =
                        is_current_target && (if is_native_button { is_button } else { !is_link });
                    let is_enter_key = raw.key() == "Enter";
                    let is_space_key = raw.key() == " ";
                    let role = current_target.get_attribute("role");
                    let is_text_navigation_role = role
                        .as_deref()
                        .map(|role| {
                            role.starts_with("menuitem") || role == "option" || role == "gridcell"
                        })
                        .unwrap_or(false);

                    // Composite Space activates on keydown (`:138-152`).
                    if is_current_target && is_composite_item && is_space_key {
                        if raw.default_prevented() && is_text_navigation_role {
                            return;
                        }

                        raw.prevent_default();

                        // Only a native-mode item that isn't a real <button> is
                        // excluded (`:145-149`).
                        if !is_native_button || is_button {
                            event.prevent_base_ui_handler();
                            dispatch_click_with_modifiers(&current_target, raw, 0);
                        }

                        return;
                    }

                    // Keyboard accessibility for native and non-native elements
                    // (`:154-163`).
                    if !should_click || is_native_button || (!is_space_key && !is_enter_key) {
                        // Space activates links on keyup (`role="button"` semantics,
                        // matching the composite path); prevent the page scroll Space
                        // would otherwise trigger. Enter is left to the browser's
                        // native link activation.
                        if is_current_target && is_link && is_space_key {
                            raw.prevent_default();
                        }
                        return;
                    }

                    // Match native buttons: preventing the keydown's default cancels
                    // activation (`:165-168`).
                    if raw.default_prevented() {
                        return;
                    }

                    raw.prevent_default();

                    if is_enter_key {
                        event.prevent_base_ui_handler();
                        dispatch_click_with_modifiers(&current_target, raw, 0);
                    }
                })
            };

            // `onKeyUp` (`:177-217`).
            let on_key_up: ElementEventHandler<BaseUIEvent<KeyboardEvent>> = {
                let disabled_now = disabled_now.clone();
                let external_on_key_up = external_on_key_up.clone();
                Rc::new(move |event: &BaseUIEvent<KeyboardEvent>| {
                    if disabled_now() {
                        return;
                    }

                    if let Some(external) = external_on_key_up.as_ref() {
                        external(event);
                    }

                    let raw = event.inner();

                    // Calling preventDefault in keyUp on a <button> will not dispatch
                    // a click event if Space is pressed (`:182-183` — the cited
                    // browser constraint the swallow arm below exists for).
                    let is_same_target = raw
                        .target()
                        .zip(raw.current_target())
                        .map(|(target, current)| target == current)
                        .unwrap_or(false);
                    let current_target: Option<HtmlElement> = raw
                        .current_target()
                        .and_then(|target| target.dyn_into::<HtmlElement>().ok());
                    if is_same_target
                        && is_native_button
                        && is_composite_item
                        && current_target
                            .as_ref()
                            .map(|element| is_button_element(Some(element.as_ref() as &Element)))
                            .unwrap_or(false)
                        && raw.key() == " "
                    {
                        raw.prevent_default();
                        return;
                    }

                    if event.base_ui_handler_prevented() {
                        return;
                    }

                    // Keyboard accessibility for non interactive elements. Match
                    // native buttons: preventing the keyup's default cancels Space
                    // activation (`:202-216`).
                    if is_same_target
                        && !is_native_button
                        && !is_composite_item
                        && !raw.default_prevented()
                        && raw.key() == " "
                    {
                        event.prevent_base_ui_handler();
                        if let Some(current_target) = current_target.as_ref() {
                            dispatch_click_with_modifiers(
                                current_target.unchecked_ref::<Element>(),
                                raw,
                                0,
                            );
                        }
                    }
                })
            };

            // `onPointerDown` (`:218-224`).
            let on_pointer_down: ElementEventHandler<PointerEvent> = {
                let disabled_now = disabled_now.clone();
                let external_on_pointer_down = external_on_pointer_down.clone();
                Rc::new(move |event: &PointerEvent| {
                    if disabled_now() {
                        event.prevent_default();
                        return;
                    }
                    if let Some(external) = external_on_pointer_down.as_ref() {
                        external(event);
                    }
                })
            };

            // The attribute half of the merge: `{ type: 'button' }` / `{ role:
            // 'button' }` (`:226`) plus the focusableWhenDisabled policy's members.
            // External non-handler attributes override at the view layer, matching
            // `otherExternalProps` spreading last upstream.
            let mut attributes: Vec<(String, ElementAttributeFn)> = Vec::new();
            attributes.push((
                (if is_native_button { "type" } else { "role" }).to_string(),
                Rc::new(move || Some("button".to_string())) as ElementAttributeFn,
            ));
            for (name, value) in &fwd_attributes {
                attributes.push(((*name).to_string(), Rc::clone(value)));
            }

            ButtonProps {
                handlers: ButtonHandlers {
                    on_click: Some(on_click),
                    on_mouse_down: Some(on_mouse_down),
                    on_key_down: Some(on_key_down),
                    on_key_up: Some(on_key_up),
                    on_pointer_down: Some(on_pointer_down),
                },
                attributes,
            }
        })
    };

    UseButtonReturnValue {
        get_button_props,
        button_ref,
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use reactive_graph::computed::Memo;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::Set;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{
        Element, EventTarget, HtmlButtonElement, HtmlElement, KeyboardEvent, KeyboardEventInit,
        MouseEvent, MouseEventInit, PointerEvent, PointerEventInit,
    };

    use super::*;
    use crate::composite_root_context::{
        CompositeRootContextValue, provide_composite_root_context,
    };
    use leptos_ui_utils::error as error_log;
    use leptos_ui_utils::merge_cleanups::CleanupFn;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// One entry per external-handler invocation, keyed by the prop name — the role
    /// the upstream tests' `vi.fn()` props play (`useButton.test.tsx:20-26` and
    /// throughout).
    #[derive(Default)]
    struct ExternalLog {
        click: u32,
        mouse_down: u32,
        key_down: u32,
        key_up: u32,
        pointer_down: u32,
    }

    struct Harness {
        _owner: Owner,
        _cleanup: Option<CleanupFn>,
        element: Element,
        disabled: RwSignal<bool>,
        log: Rc<RefCell<ExternalLog>>,
        /// External key handlers call `preventDefault` when set — the
        /// `onKeyDown={(event) => event.preventDefault()}` props upstream
        /// (`useButton.test.tsx:380,403,425`).
        prevent_default_keys: Rc<Cell<bool>>,
        /// External key handlers call `preventBaseUIHandler` when set — the
        /// preventBaseUIHandler props upstream (`useButton.test.tsx:575-586`).
        prevent_base_ui_keys: Rc<Cell<bool>>,
        /// The resolved bag, kept for attribute assertions.
        props: ButtonProps,
    }

    fn document() -> web_sys::Document {
        web_sys::window()
            .expect("no window")
            .document()
            .expect("no document")
    }

    fn mount(
        tag: &str,
        native: bool,
        composite: Option<bool>,
        focusable_when_disabled: Option<bool>,
        disabled: bool,
        tab_index: i32,
        provide_composite_context: bool,
    ) -> Harness {
        // The updateDisabled effect re-runs through the ambient executor (the
        // `use_press_and_hold` wasm-suite note); re-initializing returns `Err`.
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        if provide_composite_context {
            provide_composite_root_context(CompositeRootContextValue {
                highlighted_index: Memo::new(|_| 0),
                on_highlighted_index_change: Rc::new(|_, _| {}),
                highlight_item_on_hover: false,
                relay_keyboard_event: Rc::new(|_: &KeyboardEvent| {}),
            });
        }

        let element = document()
            .create_element(tag)
            .expect("create_element failed");
        document()
            .body()
            .expect("a body")
            .append_child(&element)
            .unwrap();

        let log: Rc<RefCell<ExternalLog>> = Rc::new(RefCell::new(ExternalLog::default()));
        let prevent_default_keys = Rc::new(Cell::new(false));
        let prevent_base_ui_keys = Rc::new(Cell::new(false));

        let click_log = Rc::clone(&log);
        let on_click: ElementEventHandler<MouseEvent> =
            Rc::new(move |_| click_log.borrow_mut().click += 1);
        let mouse_down_log = Rc::clone(&log);
        let on_mouse_down: ElementEventHandler<MouseEvent> =
            Rc::new(move |_| mouse_down_log.borrow_mut().mouse_down += 1);
        let pointer_down_log = Rc::clone(&log);
        let on_pointer_down: ElementEventHandler<PointerEvent> =
            Rc::new(move |_| pointer_down_log.borrow_mut().pointer_down += 1);

        let key_down_log = Rc::clone(&log);
        let key_down_prevent = Rc::clone(&prevent_default_keys);
        let key_down_base_ui = Rc::clone(&prevent_base_ui_keys);
        let on_key_down: ElementEventHandler<BaseUIEvent<KeyboardEvent>> =
            Rc::new(move |event| {
                key_down_log.borrow_mut().key_down += 1;
                if key_down_base_ui.get() {
                    event.prevent_base_ui_handler();
                }
                if key_down_prevent.get() {
                    event.inner().prevent_default();
                }
            });
        let key_up_log = Rc::clone(&log);
        let key_up_prevent = Rc::clone(&prevent_default_keys);
        let key_up_base_ui = Rc::clone(&prevent_base_ui_keys);
        let on_key_up: ElementEventHandler<BaseUIEvent<KeyboardEvent>> = Rc::new(move |event| {
            key_up_log.borrow_mut().key_up += 1;
            if key_up_base_ui.get() {
                event.prevent_base_ui_handler();
            }
            if key_up_prevent.get() {
                event.inner().prevent_default();
            }
        });

        let disabled_signal = RwSignal::new(disabled);
        let ret = use_button(UseButtonParams {
            disabled: disabled_signal,
            focusable_when_disabled,
            tab_index,
            native,
            composite,
        });

        let props = (ret.get_button_props)(ButtonExternalHandlers {
            on_click: Some(on_click),
            on_mouse_down: Some(on_mouse_down),
            on_key_down: Some(on_key_down),
            on_key_up: Some(on_key_up),
            on_pointer_down: Some(on_pointer_down),
        });

        // The view layer's attribute application: evaluate the lazy members and set
        // them on the element, in the bag's order (external non-handler attributes
        // would follow at the view layer and override).
        for (name, value) in &props.attributes {
            if let Some(value) = value() {
                element.set_attribute(name, &value).unwrap();
            }
        }

        let cleanup = props.attach_to(element.as_ref() as &EventTarget);
        (ret.button_ref)(Some(element.clone().unchecked_into::<HtmlElement>()));

        Harness {
            _owner: owner,
            _cleanup: cleanup,
            element,
            disabled: disabled_signal,
            log,
            prevent_default_keys,
            prevent_base_ui_keys,
            props,
        }
    }

    fn key_event(event_type: &str, key: &str) -> KeyboardEvent {
        let init = KeyboardEventInit::new();
        init.set_bubbles(true);
        // Real keyboard events are cancelable — without this, `preventDefault` in the
        // handlers is a no-op and the defaultPrevented gates can never trigger.
        init.set_cancelable(true);
        init.set_key(key);
        KeyboardEvent::new_with_keyboard_event_init_dict(event_type, &init)
            .expect("KeyboardEvent failed")
    }

    fn mouse_event(event_type: &str) -> MouseEvent {
        let init = MouseEventInit::new();
        init.set_bubbles(true);
        MouseEvent::new_with_mouse_event_init_dict(event_type, &init).expect("MouseEvent failed")
    }

    fn pointer_event(event_type: &str) -> PointerEvent {
        let init = PointerEventInit::new();
        init.set_bubbles(true);
        PointerEvent::new_with_event_init_dict(event_type, &init)
            .expect("PointerEvent failed")
    }

    impl Harness {
        fn fire_key(&self, event_type: &str, key: &str) -> bool {
            self.element
                .dispatch_event(key_event(event_type, key).as_ref())
                .expect("dispatch failed")
        }

        fn fire_mouse(&self, event_type: &str) {
            self.element
                .dispatch_event(mouse_event(event_type).as_ref())
                .expect("dispatch failed");
        }

        fn fire_pointer(&self, event_type: &str) {
            self.element
                .dispatch_event(pointer_event(event_type).as_ref())
                .expect("dispatch failed");
        }

        fn clicks(&self) -> u32 {
            self.log.borrow().click
        }

        fn attribute(&self, name: &str) -> Option<String> {
            self.element.get_attribute(name)
        }
    }

    // `can be activated with Enter key` / `can be activated with Space key`
    // (`useButton.test.tsx:19-41`): a non-native span with `role="button"` activates
    // on keydown for Enter and on keyup for Space.
    #[wasm_bindgen_test]
    fn non_native_span_activates_with_enter_on_keydown_and_space_on_keyup() {
        let harness = mount("span", false, None, None, false, 0, false);

        harness.fire_key("keydown", "Enter");
        assert_eq!(harness.clicks(), 1, "Enter dispatches the click on keydown");
        assert_eq!(harness.log.borrow().key_down, 1);

        harness.fire_key("keydown", " ");
        assert_eq!(harness.clicks(), 1, "Space does not click on keydown");
        harness.fire_key("keyup", " ");
        assert_eq!(harness.clicks(), 2, "Space clicks on keyup — the DOM's own rule");
        assert_eq!(harness.log.borrow().key_up, 1);
    }

    // `does not set a type prop` (`useButton.test.tsx:43-54`): a non-native host gets
    // `role="button"` and no `type`; a native one gets `type="button"` (the
    // `useButton.ts:226` merge bag).
    #[wasm_bindgen_test]
    fn non_native_sets_role_and_native_sets_type() {
        let span = mount("span", false, None, None, false, 0, false);
        assert_eq!(span.attribute("role"), Some("button".to_string()));
        assert_eq!(span.attribute("type"), None, "no type prop on a non-native host");

        let button = mount("button", true, None, None, false, 0, false);
        assert_eq!(button.attribute("type"), Some("button".to_string()));
        assert_eq!(button.attribute("role"), None, "no role prop on a native host");
    }

    // `key: Space fires keyup then click on non-composite buttons`
    // (`useButton.test.tsx:281-308`).
    #[wasm_bindgen_test]
    fn space_fires_click_on_keyup_for_non_composite_buttons() {
        let harness = mount("span", false, None, None, false, 0, false);

        harness.fire_key("keydown", " ");
        assert_eq!(harness.log.borrow().key_down, 1);
        assert_eq!(harness.clicks(), 0);

        harness.fire_key("keyup", " ");
        assert_eq!(harness.log.borrow().key_up, 1);
        assert_eq!(harness.clicks(), 1);
    }

    // `key: Space fires keydown then click on composite buttons`
    // (`useButton.test.tsx:310-343`).
    #[wasm_bindgen_test]
    fn space_fires_click_on_keydown_for_composite_buttons() {
        let harness = mount("span", false, Some(true), None, false, 0, false);

        harness.fire_key("keydown", " ");
        assert_eq!(harness.log.borrow().key_down, 1);
        assert_eq!(harness.clicks(), 1, "composite Space activates on keydown");

        harness.fire_key("keyup", " ");
        assert_eq!(harness.log.borrow().key_up, 1, "the keyup handler still runs");
        assert_eq!(harness.clicks(), 1, "and does not re-click");
    }

    // `key: Space fires keydown then click on composite links`
    // (`useButton.test.tsx:344-366`).
    #[wasm_bindgen_test]
    fn space_fires_click_on_keydown_for_composite_links() {
        let harness = mount("a", false, Some(true), None, false, 0, false);
        harness.element.set_attribute("href", "#test").unwrap();

        harness.fire_key("keydown", " ");
        assert_eq!(harness.clicks(), 1);

        harness.fire_key("keyup", " ");
        assert_eq!(harness.clicks(), 1, "the keyup dispatch branch is composite-gated off");
    }

    // `does not click composite links when Space is prevented for text navigation`
    // (`useButton.test.tsx:367-388`).
    #[wasm_bindgen_test]
    fn composite_links_with_menuitem_role_do_not_click_when_space_is_prevented() {
        let harness = mount("a", false, Some(true), None, false, 0, false);
        harness.element.set_attribute("href", "#test").unwrap();
        harness
            .element
            .set_attribute("role", "menuitem")
            .unwrap();
        harness.prevent_default_keys.set(true);

        harness.fire_key("keydown", " ");
        assert_eq!(harness.clicks(), 0, "defaultPrevented + menuitem is a text-navigation bail");
    }

    // `does not click composite gridcells when Space is prevented`
    // (`useButton.test.tsx:389-410`).
    #[wasm_bindgen_test]
    fn composite_gridcells_do_not_click_when_space_is_prevented() {
        let harness = mount("div", false, Some(true), None, false, 0, false);
        harness
            .element
            .set_attribute("role", "gridcell")
            .unwrap();
        harness.prevent_default_keys.set(true);

        harness.fire_key("keydown", " ");
        assert_eq!(harness.clicks(), 0);
    }

    // `clicks composite switches when Space is prevented`
    // (`useButton.test.tsx:411-432`).
    #[wasm_bindgen_test]
    fn composite_switches_click_when_space_is_prevented() {
        let harness = mount("div", false, Some(true), None, false, 0, false);
        harness.element.set_attribute("role", "switch").unwrap();
        harness.prevent_default_keys.set(true);

        harness.fire_key("keydown", " ");
        assert_eq!(harness.clicks(), 1, "a non-text-navigation role still activates");
    }

    // `key: Space fires keydown then click on native composite buttons` and
    // `does not fire duplicate clicks for Space on native composite buttons`
    // (`useButton.test.tsx:433-481`).
    #[wasm_bindgen_test]
    fn native_composite_space_clicks_once_across_the_full_press() {
        let harness = mount("button", true, Some(true), None, false, 0, false);

        harness.fire_key("keydown", " ");
        assert_eq!(harness.log.borrow().key_down, 1);
        assert_eq!(harness.clicks(), 1, "composite Space activates on keydown on native buttons too");

        harness.fire_key("keyup", " ");
        assert_eq!(harness.log.borrow().key_up, 1);
        assert_eq!(harness.clicks(), 1, "the keyup Space is swallowed — exactly one click per press");
    }

    // `fires a single click for nested non-native composite buttons`
    // (`useButton.test.tsx:482-505`): the inner bag's handlers are the outer bag's
    // externals; the shared prevention mark keeps one click per key. (The upstream
    // JSX spread also merges the inner bag's *attributes* last; in the port the
    // non-handler members compose at the view layer, and both bags derive the same
    // `role="button"` here.)
    #[wasm_bindgen_test]
    fn nested_non_native_composite_bags_fire_a_single_click() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let element = document().create_element("span").unwrap();
        document().body().unwrap().append_child(&element).unwrap();

        let click_count: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        let spy_click: ElementEventHandler<MouseEvent> = {
            let click_count = Rc::clone(&click_count);
            Rc::new(move |_| click_count.set(click_count.get() + 1))
        };

        let inner = use_button(UseButtonParams {
            disabled: RwSignal::new(false),
            focusable_when_disabled: None,
            tab_index: 0,
            native: false,
            composite: Some(true),
        });
        let inner_noop: ButtonExternalHandlers = ButtonExternalHandlers::default();
        let inner_props = (inner.get_button_props)(inner_noop);

        let outer = use_button(UseButtonParams {
            disabled: RwSignal::new(false),
            focusable_when_disabled: None,
            tab_index: 0,
            native: false,
            composite: Some(true),
        });
        let outer_props = (outer.get_button_props)(ButtonExternalHandlers {
            on_click: Some(spy_click),
            on_mouse_down: inner_props.handlers.on_mouse_down.clone(),
            on_key_down: inner_props.handlers.on_key_down.clone(),
            on_key_up: inner_props.handlers.on_key_up.clone(),
            on_pointer_down: inner_props.handlers.on_pointer_down.clone(),
        });

        for (name, value) in &outer_props.attributes {
            if let Some(value) = value() {
                element.set_attribute(name, &value).unwrap();
            }
        }
        let _cleanup = outer_props.attach_to(element.as_ref() as &EventTarget);
        (outer.button_ref)(Some(element.clone().unchecked_into::<HtmlElement>()));

        let fire = |event_type: &str, key: &str| {
            let init = KeyboardEventInit::new();
            init.set_bubbles(true);
            init.set_key(key);
            let event =
                KeyboardEvent::new_with_keyboard_event_init_dict(event_type, &init).unwrap();
            element.dispatch_event(event.as_ref()).unwrap();
        };

        fire("keydown", " ");
        assert_eq!(click_count.get(), 1, "the inner dispatch's preventBaseUIHandler mark stops the outer pipeline");

        fire("keydown", "Enter");
        assert_eq!(click_count.get(), 2, "same single-click rule for Enter");
    }

    // `does not click composite buttons when keydown calls preventBaseUIHandler`
    // (`useButton.test.tsx:564-595`).
    #[wasm_bindgen_test]
    fn prevent_base_ui_handler_cancels_composite_space_activation() {
        let harness = mount("span", false, Some(true), None, false, 0, false);
        harness.prevent_base_ui_keys.set(true);

        harness.fire_key("keydown", " ");
        assert_eq!(harness.clicks(), 0);
        assert_eq!(harness.log.borrow().key_down, 1, "the consumer handler still ran");
    }

    // `does not click non-composite buttons when keydown/keyup calls
    // preventBaseUIHandler` (`useButton.test.tsx:596-633`).
    #[wasm_bindgen_test]
    fn prevent_base_ui_handler_cancels_non_composite_activation() {
        let harness = mount("span", false, None, None, false, 0, false);
        harness.prevent_base_ui_keys.set(true);

        harness.fire_key("keydown", "Enter");
        assert_eq!(harness.clicks(), 0, "Enter activates on keydown; the consumer prevented it");

        harness.fire_key("keydown", " ");
        harness.fire_key("keyup", " ");
        assert_eq!(harness.clicks(), 0, "Space activates on keyup; the consumer prevented it");
    }

    // `key: Enter does not click non-native buttons when keydown calls preventDefault`
    // and `key: Space does not click non-native buttons when keyup calls
    // preventDefault` (`useButton.test.tsx:634-687`).
    #[wasm_bindgen_test]
    fn prevent_default_cancels_non_composite_activation() {
        let harness = mount("span", false, None, None, false, 0, false);
        harness.prevent_default_keys.set(true);

        harness.fire_key("keydown", "Enter");
        assert_eq!(harness.clicks(), 0, "preventing the keydown's default cancels Enter activation");

        harness.fire_key("keydown", " ");
        harness.fire_key("keyup", " ");
        assert_eq!(harness.clicks(), 0, "preventing the keyup's default cancels Space activation");
    }

    // `key: Space fires keydown then click when in composite root context`
    // (`useButton.test.tsx:688-723`): the composite inference comes from the context
    // when `composite` is unspecified.
    #[wasm_bindgen_test]
    fn composite_root_context_infers_keydown_activation() {
        let harness = mount("span", false, None, None, false, 0, true);

        harness.fire_key("keydown", " ");
        assert_eq!(harness.clicks(), 1, "context-inferred composite: Space clicks on keydown");
        harness.fire_key("keyup", " ");
        assert_eq!(harness.clicks(), 1);
    }

    // `key: Space fires keydown then click on native buttons in composite root
    // context` (`useButton.test.tsx:724-753`).
    #[wasm_bindgen_test]
    fn composite_root_context_infers_for_native_buttons_too() {
        let harness = mount("button", true, None, None, false, 0, true);

        harness.fire_key("keydown", " ");
        assert_eq!(harness.clicks(), 1);
        harness.fire_key("keyup", " ");
        assert_eq!(harness.clicks(), 1, "the native-composite keyup swallow applies");
    }

    // "`composite=false` keeps keyup activation inside composite root context"
    // (`useButton.test.tsx:755-784`).
    #[wasm_bindgen_test]
    fn composite_false_overrides_the_context_inference() {
        let harness = mount("span", false, Some(false), None, false, 0, true);

        harness.fire_key("keydown", " ");
        assert_eq!(harness.clicks(), 0);
        harness.fire_key("keyup", " ");
        assert_eq!(harness.clicks(), 1, "explicit composite: false restores keyup activation");
    }

    // `param: tabIndex` rows (`useButton.test.tsx:227-266`): the explicit value wins,
    // the default is 0, and composite items carry no tabindex of their own.
    #[wasm_bindgen_test]
    fn tab_index_is_reflected_in_the_attributes() {
        let default_native = mount("button", true, None, None, false, 0, false);
        assert_eq!(default_native.attribute("tabindex"), Some("0".to_string()));

        let explicit = mount("button", true, None, None, false, 3, false);
        assert_eq!(explicit.attribute("tabindex"), Some("3".to_string()));

        let non_native = mount("span", false, None, None, false, 0, false);
        assert_eq!(non_native.attribute("tabindex"), Some("0".to_string()));

        let composite = mount("span", false, Some(true), None, false, 0, false);
        assert_eq!(composite.attribute("tabindex"), None, "composite items get their roving tabindex from the composite machinery");
    }

    // `allows disabled buttons to be focused` (`useButton.test.tsx:119-134`): a
    // native disabled focusableWhenDisabled button renders `aria-disabled` instead of
    // the boolean attribute and stays focusable.
    #[wasm_bindgen_test]
    fn focusable_when_disabled_native_button_uses_aria_disabled_and_stays_focusable() {
        let harness = mount("button", true, None, Some(true), true, 0, false);

        assert_eq!(harness.attribute("aria-disabled"), Some("true".to_string()));
        assert_eq!(harness.attribute("disabled"), None, "the boolean attribute is replaced by aria-disabled");
        harness
            .element
            .unchecked_ref::<HtmlElement>()
            .focus()
            .unwrap();
        assert_eq!(
            document()
                .active_element()
                .map(|active| active == harness.element),
            Some(true),
            "the button keeps focus"
        );
    }

    // `prevents interactions except focus and blur` (`useButton.test.tsx:169-225`):
    // every interaction handler is inert while disabled; focus still works.
    #[wasm_bindgen_test]
    fn disabled_non_native_button_prevents_interactions_except_focus() {
        let harness = mount("span", false, None, Some(true), true, 0, false);

        harness
            .element
            .unchecked_ref::<HtmlElement>()
            .focus()
            .unwrap();
        assert_eq!(
            document()
                .active_element()
                .map(|active| active == harness.element),
            Some(true),
            "focus is not blocked"
        );

        harness.fire_key("keydown", "Enter");
        assert_eq!(harness.log.borrow().key_down, 0, "the disabled gate runs before the external keydown");
        assert_eq!(harness.clicks(), 0);

        harness.fire_key("keyup", " ");
        assert_eq!(harness.log.borrow().key_up, 0);
        assert_eq!(harness.clicks(), 0);

        harness.fire_mouse("click");
        assert_eq!(harness.log.borrow().click, 0, "click is prevented and ignored while disabled");
        harness.fire_pointer("pointerdown");
        assert_eq!(harness.log.borrow().pointer_down, 0, "pointerdown is prevented while disabled");
    }

    // The disabled *native* attribute policy (`useFocusableWhenDisabled.ts:45-47` via
    // `useButton.ts:28-34`): a plain disabled native button renders the boolean
    // attribute, no aria-disabled.
    #[wasm_bindgen_test]
    fn disabled_native_button_renders_the_boolean_disabled_attribute() {
        let harness = mount("button", true, None, None, true, 0, false);

        assert_eq!(harness.attribute("disabled"), Some(String::new()));
        assert_eq!(harness.attribute("aria-disabled"), None);
        assert_eq!(harness.clicks(), 0, "click is ignored while disabled");

        harness.disabled.set(false);
        let evaluated: Vec<Option<String>> =
            harness.props.attributes.iter().map(|(_, value)| value()).collect();
        let disabled_member = harness
            .props
            .attributes
            .iter()
            .position(|(name, _)| name == "disabled")
            .map(|index| evaluated[index].clone());
        assert_eq!(
            disabled_member,
            Some(None),
            "the member still exists (the view layer reads it) but re-derives to no attribute once re-enabled"
        );
    }

    // `force overrides disabled attribute when put in a composite`
    // (`useButton.test.tsx:135-168`): a disabled composite button whose host carries
    // the boolean attribute (the outer `useButton`'s external spread) has it cleared —
    // and re-cleared after a ref change.
    #[wasm_bindgen_test]
    fn update_disabled_force_clears_a_disabled_composite_button() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let element = document().create_element("button").unwrap();
        document().body().unwrap().append_child(&element).unwrap();
        // The outer `useButton`'s spread (`getButtonProps({ disabled: true })` from
        // the upstream TestButton) put the attribute on the host.
        element.set_attribute("disabled", "").unwrap();

        let ret = use_button(UseButtonParams {
            disabled: RwSignal::new(true),
            focusable_when_disabled: Some(true),
            tab_index: 0,
            native: true,
            composite: Some(true),
        });
        let props = (ret.get_button_props)(ButtonExternalHandlers::default());
        let _cleanup = props.attach_to(element.as_ref() as &EventTarget);

        (ret.button_ref)(Some(element.clone().unchecked_into::<HtmlElement>()));
        let button: HtmlButtonElement = element.clone().unchecked_into();
        assert_eq!(button.disabled(), false, "updateDisabled cleared the attribute on ref attach");

        // The "even after the button's ref changes" clause: detach + re-attach runs
        // updateDisabled again, and the re-set attribute is cleared once more.
        element.set_attribute("disabled", "").unwrap();
        (ret.button_ref)(None);
        (ret.button_ref)(Some(element.clone().unchecked_into()));
        assert_eq!(button.disabled(), false);
    }

    // `errors if nativeButton=true but ref is not a button`
    // (`useButton.test.tsx:836-857`).
    #[wasm_bindgen_test]
    fn warns_when_a_native_button_expects_a_button_but_renders_a_span() {
        error_log::reset();
        let spy = ConsoleSpy::install("error");

        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let element = document().create_element("span").unwrap();
        document().body().unwrap().append_child(&element).unwrap();

        let ret = use_button(UseButtonParams {
            disabled: RwSignal::new(false),
            focusable_when_disabled: None,
            tab_index: 0,
            native: true,
            composite: None,
        });
        let props = (ret.get_button_props)(ButtonExternalHandlers::default());
        let _cleanup = props.attach_to(element.as_ref() as &EventTarget);
        (ret.button_ref)(Some(element.clone().unchecked_into::<HtmlElement>()));

        let calls = spy.calls.borrow();
        assert_eq!(calls.len(), 1, "the mismatch warning logs once");
        assert!(
            calls[0].contains(NATIVE_EXPECTED_MESSAGE),
            "the message matches the upstream text with the Base UI prefix: {:?}",
            calls[0]
        );
    }

    // `errors if nativeButton=false but ref is a button`
    // (`useButton.test.tsx:858-880`).
    #[wasm_bindgen_test]
    fn warns_when_a_non_native_button_expects_a_non_button_but_renders_a_button() {
        error_log::reset();
        let spy = ConsoleSpy::install("error");

        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let element = document().create_element("button").unwrap();
        document().body().unwrap().append_child(&element).unwrap();

        let ret = use_button(UseButtonParams {
            disabled: RwSignal::new(false),
            focusable_when_disabled: None,
            tab_index: 0,
            native: false,
            composite: None,
        });
        let props = (ret.get_button_props)(ButtonExternalHandlers::default());
        let _cleanup = props.attach_to(element.as_ref() as &EventTarget);
        (ret.button_ref)(Some(element.clone().unchecked_into::<HtmlElement>()));

        let calls = spy.calls.borrow();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].contains(NON_NATIVE_EXPECTED_MESSAGE), "{:?}", calls[0]);
    }

    /// Replaces `console.error` with a recording spy for the spy's lifetime — the
    /// wasm equivalent of upstream's `vi.spyOn(console, 'error').mockImplementation`
    /// (the `create_log_once` wasm-suite pattern).
    struct ConsoleSpy {
        console: js_sys::Object,
        original: wasm_bindgen::JsValue,
        calls: Rc<RefCell<Vec<String>>>,
        _closure: wasm_bindgen::prelude::Closure<dyn FnMut(wasm_bindgen::JsValue)>,
    }

    impl ConsoleSpy {
        fn install(method: &'static str) -> ConsoleSpy {
            let global: js_sys::Object = js_sys::global();
            let console: js_sys::Object = js_sys::Reflect::get(&global, &"console".into())
                .expect("console exists")
                .unchecked_into();
            let original = js_sys::Reflect::get(&console, &method.into()).expect("method exists");
            let calls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
            let calls_in_spy = Rc::clone(&calls);
            let closure = wasm_bindgen::prelude::Closure::new(move |message: wasm_bindgen::JsValue| {
                calls_in_spy.borrow_mut().push(message.as_string().unwrap_or_default());
            });
            js_sys::Reflect::set(&console, &method.into(), closure.as_ref().unchecked_ref())
                .unwrap();
            ConsoleSpy {
                console,
                original,
                calls,
                _closure: closure,
            }
        }
    }

    impl Drop for ConsoleSpy {
        fn drop(&mut self) {
            js_sys::Reflect::set(&self.console, &"error".into(), &self.original).unwrap();
        }
    }
}
