//! `Radio.Root` context — port of `packages/react/src/radio/root/RadioRootContext.ts`
//! (the `library: radio` TODO item; `specs/library/radio/behavior.md`,
//! `specs/library/radio/implementation.md`).
//!
//! Upstream's context value **is** the state object (`RadioRootContext.ts:4`:
//! `export type RadioRootContext = RadioRootState`), created with an `undefined` default
//! (`:6`) and read by the parts through `useRadioRootContext()`, which throws the
//! documented message when no Root is above (`:8-17`).
//!
//! What makes the throw load-bearing is implementation.md's account of the same file:
//! the only in-unit consumer is `RadioIndicator` (`RadioIndicator.tsx:23`), and because
//! the state spreads `fieldState` (`RadioRoot.tsx:216`) the Field's
//! validity/touched/dirty/filled/focused flow into the Indicator's data attributes with no
//! extra wiring. A single Root context therefore carries *both* the selection state and the
//! Field state to every part — which is why Root and Indicator always agree on their hooks.
//!
//! The port carries it as live leptos signals (the `SwitchRootContextValue` /
//! `CheckboxRootContextValue` precedent) because a part rebuilt from a `view!` closure must
//! track state changes; [`RadioRootContextValue::snapshot`] is the render-scoped object
//! upstream hands over, and it is exactly the record the shared attribute walk consumes.

use leptos::prelude::*;

use crate::radio::state::RadioRootState;

/// The missing-context message, verbatim from `RadioRootContext.ts:10-13`.
pub const MISSING_ROOT_CONTEXT_MESSAGE: &str =
    "Base UI: RadioRootContext is missing. Radio parts must be placed within <Radio.Root>.";

/// The live context bag. Every member is a leptos signal so a part tracks it.
#[derive(Clone)]
pub struct RadioRootContextValue {
    /// `RadioRootState.checked`.
    pub checked: Signal<bool>,
    /// `RadioRootState.disabled`.
    pub disabled: Signal<bool>,
    /// `RadioRootState.readOnly`.
    pub read_only: Signal<bool>,
    /// `RadioRootState.required`.
    pub required: Signal<bool>,
    /// `...fieldState.touched`.
    pub touched: Signal<bool>,
    /// `...fieldState.dirty`.
    pub dirty: Signal<bool>,
    /// `...fieldState.valid` (`None` is upstream's `null`).
    pub valid: Signal<Option<bool>>,
    /// `...fieldState.filled`.
    pub filled: Signal<bool>,
    /// `...fieldState.focused`.
    pub focused: Signal<bool>,
}

impl RadioRootContextValue {
    /// The render-scoped state object (upstream's context value, `RadioRoot.tsx:225`).
    pub fn snapshot(&self) -> RadioRootState {
        RadioRootState {
            checked: self.checked.get(),
            disabled: self.disabled.get(),
            read_only: self.read_only.get(),
            required: self.required.get(),
            touched: self.touched.get(),
            dirty: self.dirty.get(),
            valid: self.valid.get(),
            filled: self.filled.get(),
            focused: self.focused.get(),
        }
    }

    /// The same snapshot without subscribing — for writer effects that own their own
    /// tracking (the `checkbox::root` precedent).
    pub fn tracked_snapshot(&self) -> RadioRootState {
        self.snapshot()
    }
}

/// `useRadioRootContext()` (`RadioRootContext.ts:8-17`): the part accessor, panicking with
/// the upstream message outside a Root.
pub fn use_radio_root_context() -> RadioRootContextValue {
    try_use_radio_root_context().unwrap_or_else(|| panic!("{MISSING_ROOT_CONTEXT_MESSAGE}"))
}

/// The non-panicking probe — the host-testable half of the missing-context contract
/// (a wasm panic is an uncatchable trap; the switch/checkbox/meter precedent).
pub fn try_use_radio_root_context() -> Option<RadioRootContextValue> {
    use_context::<RadioRootContextValue>()
}

/// The provider seam (`RadioRoot.tsx:249`): publishes the live bag under the leptos owner
/// so the parts mounted inside the Root's subtree read it. The bag is signal-only, so it
/// already satisfies `provide_context`'s `Send + Sync` contract (the
/// `SwitchRootContextValue` precedent — no wrapper needed).
pub fn provide_radio_root_context(value: RadioRootContextValue) {
    provide_context(value);
}
