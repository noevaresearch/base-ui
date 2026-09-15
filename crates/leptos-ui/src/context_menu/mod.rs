//! Context Menu — port of `packages/react/src/context-menu/` (the `library:
//! context-menu` TODO item; `specs/library/context-menu/behavior.md`,
//! `specs/library/context-menu/implementation.md`).
//!
//! Context Menu is Menu plus pointer-gesture plumbing (implementation.md, opening
//! line): two real components — [`root::ContextMenuRootComponent`] and
//! [`trigger::ContextMenuTrigger`] — plus [`positioner::ContextMenuPositioner`]
//! (the `MenuPositioner` re-export with the context-menu defaults) and the
//! `Menu`-re-exported parts that reach the context-menu context because they render
//! under the Root's providers (`index.parts.ts:1-21` — Portal, Popup, Backdrop,
//! Arrow, Group*, Item variants, Submenu*).
//!
//! The unit's structure (implementation.md "Context providers/consumers"): the Root
//! renders a two-layer provider sandwich and nothing else — the context-menu context
//! (the anchor/actions/ref slots) over a *cleared* menu root context, so a Context
//! Menu nested inside another menu's subtree stays a standalone root
//! (`MenuRoot.tsx:94-102`).

pub mod positioner;
pub mod root;
pub mod trigger;

pub use positioner::*;
pub use root::*;
pub use trigger::*;

// The re-exported Menu parts under the ContextMenu namespace (`index.parts.ts:1-21`
// — every part except `Root`, `Trigger`, and `Positioner` is Menu's, reached through
// the Root's providers rather than re-implemented). Only the store-backed Menu parts
// exist in the port so far — the menu module compiles `item`, `popup`, `portal`,
// `positioner`, `root`, `trigger` (the arrow/backdrop/submenu/radio files are on
// disk but not yet declared as modules of the menu unit, so there is nothing to
// re-export for them; recorded in the item's TODO note).
pub use crate::menu::portal::MenuPortal;
pub use crate::menu::positioner::MenuPositioner as MenuPositionerPart;
pub use crate::menu::{MenuItem, MenuPopup, MenuTrigger};
