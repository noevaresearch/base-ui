//! `Switch` — the port of `packages/react/src/switch/` (the `library: switch` TODO item;
//! `specs/library/switch/behavior.md`, `specs/library/switch/implementation.md`).
//!
//! Upstream's unit is two parts and one small composition layer:
//!
//! - `Switch.Root` (`root/SwitchRoot.tsx`) — a `span` (or a real `<button>` under
//!   `nativeButton`) carrying `role="switch"`, beside a hidden
//!   `<input type="checkbox">` that owns the state change and the form submission.
//!   Every interaction funnels through that input's native `change` event; the visible
//!   element re-dispatches its own click onto the input instead of mutating state
//!   (`SwitchRoot.tsx:143-156`).
//! - `Switch.Thumb` (`thumb/SwitchThumb.tsx`) — a `span` that reads the Root's state and
//!   renders the same `data-*` hooks.
//!
//! The namespaced surface (`Switch::Root` / `Switch::Thumb`) is the ergonomic half of the
//! port: upstream's docs teach `<Switch.Root><Switch.Thumb /></Switch.Root>`, so the
//! port's spelling is the same tree with Rust's path separator
//! (`specs/docs-content/CONTRACT.md`). The flat `SwitchRoot`/`SwitchThumb` names are kept
//! as well, and nothing is renamed.

pub mod context;
pub mod root;
pub mod state;
pub mod thumb;

pub use context::{
    MISSING_ROOT_CONTEXT_MESSAGE, SwitchRootContextValue, provide_switch_root_context,
    try_use_switch_root_context, use_switch_root_context,
};
pub use root::{Root, SwitchRootViewProps, switch_root_view};
pub use thumb::{SwitchThumb, Thumb};
