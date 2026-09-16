//! `Switch.Thumb` — port of `packages/react/src/switch/thumb/SwitchThumb.tsx`
//! (the `library: switch` TODO item).
//!
//! Upstream's body is three lines (`SwitchThumb.tsx:20-31`): read the Root context
//! (throwing outside a Root, `SwitchThumb.test.tsx:30-41`), then render a `span`
//! through `useRenderElement` with the shared `stateAttributesMapping` — which is why the
//! Thumb carries the same `data-*` hooks as the Root (`SwitchRoot.test.tsx:412-440`).
//!
//! The port spells that as: the context read, the shared walk
//! ([`crate::switch::state::switch_state_attributes`]) bound to the managed `data-*`
//! members, and the consumer's `...elementProps` rest applied at mount (a `view!` tree
//! has no attribute spread; the checkbox/field precedent).

use leptos::html;
use leptos::prelude::*;

use leptos_ui_internals::state_attributes::StateAttributeProps;

use crate::switch::context::use_switch_root_context;
use crate::switch::state::{
    DATA_CHECKED, DATA_DIRTY, DATA_DISABLED, DATA_FILLED, DATA_FOCUSED, DATA_INVALID, DATA_READONLY,
    DATA_REQUIRED, DATA_TOUCHED, DATA_UNCHECKED, DATA_VALID, switch_state_attributes,
};

/// The movable part of the switch that indicates whether it is on or off
/// (`SwitchThumb.tsx:11-13` — "Renders a `<span>`").
#[component]
pub fn Thumb(
    /// `className` (`SwitchThumb.tsx:23`).
    #[prop(default = None, into)]
    class: Option<String>,
    /// `style` (`:23`).
    #[prop(default = None, into)]
    style: Option<String>,
    /// The consumer's `...elementProps` rest (`:22`) as plain attributes.
    #[prop(default = Vec::new())]
    element_attributes: Vec<(String, String)>,
    children: Children,
) -> impl IntoView {
    // `useSwitchRootContext()` (`:27`) — the required read; outside a Root this panics
    // with the unit's own message.
    let context = use_switch_root_context();

    // The shared walk over the live snapshot. One derivation, eleven bindings: every
    // managed member re-derives from the same record, so the Thumb's hooks can never
    // disagree with the Root's (the reason upstream shares the mapping).
    let attributes: Signal<StateAttributeProps> =
        Signal::derive(move || switch_state_attributes(&context.snapshot()));

    // The `...elementProps` rest: static at build time, applied once at mount.
    let node: NodeRef<html::Span> = NodeRef::new();
    {
        let element_attributes = element_attributes.clone();
        Effect::new(move |_| {
            let Some(span) = node.get() else {
                return;
            };
            for (name, value) in &element_attributes {
                let _ = span.set_attribute(name, value);
            }
        });
    }

    view! {
        <span
            node_ref=node
            class=class
            style=style
            data-checked={move || attributes.get().get(DATA_CHECKED).cloned()}
            data-unchecked={move || attributes.get().get(DATA_UNCHECKED).cloned()}
            data-disabled={move || attributes.get().get(DATA_DISABLED).cloned()}
            data-readonly={move || attributes.get().get(DATA_READONLY).cloned()}
            data-required={move || attributes.get().get(DATA_REQUIRED).cloned()}
            data-valid={move || attributes.get().get(DATA_VALID).cloned()}
            data-invalid={move || attributes.get().get(DATA_INVALID).cloned()}
            data-touched={move || attributes.get().get(DATA_TOUCHED).cloned()}
            data-dirty={move || attributes.get().get(DATA_DIRTY).cloned()}
            data-filled={move || attributes.get().get(DATA_FILLED).cloned()}
            data-focused={move || attributes.get().get(DATA_FOCUSED).cloned()}
        >
            {children()}
        </span>
    }
}

/// The flat upstream-named alias (`SwitchThumb`, `switch/index.ts:2`).
pub use Thumb as SwitchThumb;
