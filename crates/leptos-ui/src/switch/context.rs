//! `Switch.Root` context — port of `packages/react/src/switch/root/SwitchRootContext.ts`
//! (the `library: switch` TODO item).
//!
//! Upstream's context value **is** the state object (`SwitchRootContext.ts:6`:
//! `export type SwitchRootContext = SwitchRootState`), created with an `undefined`
//! default (`:8`) and read by the parts through `useSwitchRootContext()`, which throws
//! the documented message when no Root is above (`:11-19`).
//!
//! The port carries it as live leptos signals — the `CheckboxRootContextValue`
//! precedent (`crate::checkbox::root`) — because a part rebuilt from a `view!` closure
//! must track state changes; [`SwitchRootContextValue::snapshot`] is the render-scoped
//! object upstream hands over, and it is exactly the record the shared attribute walk
//! consumes, which is what keeps Root and Thumb in agreement
//! (`SwitchRoot.test.tsx:412-440`).

use leptos::prelude::*;

use crate::switch::state::SwitchRootState;

/// The missing-context message, verbatim from `SwitchRootContext.ts:14-15`.
pub const MISSING_ROOT_CONTEXT_MESSAGE: &str =
    "Base UI: SwitchRootContext is missing. Switch parts must be placed within <Switch.Root>.";

/// The live context bag. Every member is a leptos signal so a part tracks it.
#[derive(Clone)]
pub struct SwitchRootContextValue {
    /// `SwitchRootState.checked`.
    pub checked: Signal<bool>,
    /// `SwitchRootState.disabled`.
    pub disabled: Signal<bool>,
    /// `SwitchRootState.readOnly`.
    pub read_only: Signal<bool>,
    /// `SwitchRootState.required`.
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

impl SwitchRootContextValue {
    /// The render-scoped state object (upstream's context value).
    pub fn snapshot(&self) -> SwitchRootState {
        SwitchRootState {
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
}

/// `useSwitchRootContext()` (`SwitchRootContext.ts:11-19`): the part accessor, throwing
/// the upstream message outside a Root.
pub fn use_switch_root_context() -> SwitchRootContextValue {
    try_use_switch_root_context().unwrap_or_else(|| panic!("{MISSING_ROOT_CONTEXT_MESSAGE}"))
}

/// The non-panicking probe — the host-testable half of the missing-context contract
/// (a wasm panic is an uncatchable trap; the checkbox/meter/field precedent).
pub fn try_use_switch_root_context() -> Option<SwitchRootContextValue> {
    use_context::<SwitchRootContextValue>()
}

/// The provider seam (`SwitchRoot.tsx:231`): publishes the live bag under the leptos
/// owner so the parts mounted inside the Root's subtree read it. The bag is signal-only,
/// so it already satisfies `provide_context`'s `Send + Sync` contract (the
/// `CheckboxRootContextValue` precedent — no wrapper needed).
pub fn provide_switch_root_context(value: SwitchRootContextValue) {
    provide_context(value);
}
