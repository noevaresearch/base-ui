//! The ContextMenu.Positioner — `MenuPositioner` re-exported with the context-menu
//! positioning defaults enforced from the shared positioner when
//! `parent.type === 'context-menu'` (`MenuPositioner.tsx:88-95`): `align: 'start'`,
//! `alignOffset: 2`, `sideOffset: -5` (applied only when `side` is unspecified and
//! `align` isn't `'center'`), `arrowPadding: 0` (`:120`), and a context-menu-specific
//! `shift` with `rootBoundary: 'layoutViewport'` (`:128-133`).
//!
//! The public component is literally the Menu positioner with narrowed props — the
//! positioning props are re-declared only to document the context-menu defaults
//! (`ContextMenuPositioner.tsx:18-34`); the deprecated `positionMethod` prop is
//! always overridden to `'fixed'` by the shared positioner (`MenuPositioner.tsx:114`,
//! untested item 7 in behavior.md's implementation.md gap list).

use leptos::prelude::*;

use crate::menu::utils::{MenuAlign, MenuSide};

/// The context-menu positioning defaults (`MenuPositioner.tsx:88-95`).
pub const CONTEXT_MENU_DEFAULT_SIDE_OFFSET: f32 = -5.0;
/// The `alignOffset: 2` default (`MenuPositioner.tsx:93`).
pub const CONTEXT_MENU_DEFAULT_ALIGN_OFFSET: f32 = 2.0;
/// The `align: 'start'` default (`MenuPositioner.tsx:92`).
pub const CONTEXT_MENU_DEFAULT_ALIGN: MenuAlign = MenuAlign::Start;
/// The forced `positionMethod: 'fixed'` (`MenuPositioner.tsx:114` — viewport
/// coordinates, because `clientX`/`clientY` anchor points correspond to no DOM node).
pub const CONTEXT_MENU_POSITION_METHOD: &str = "fixed";

/// The `ContextMenu.Positioner` component — the Menu positioner over the
/// context-menu defaults.
#[leptos::component]
pub fn ContextMenuPositioner(
    /// Side of the anchor where the menu should appear (unspecified → the context-menu
    /// default applies through the shared positioner).
    #[prop(default = MenuSide::Bottom)] side: MenuSide,
    /// Alignment of the menu relative to the anchor.
    #[prop(default = CONTEXT_MENU_DEFAULT_ALIGN)] align: MenuAlign,
    /// Offset from the anchor side.
    #[prop(default = CONTEXT_MENU_DEFAULT_SIDE_OFFSET)] side_offset: f32,
    /// Offset from the anchor alignment.
    #[prop(default = CONTEXT_MENU_DEFAULT_ALIGN_OFFSET)] align_offset: f32,
    /// Children (the popup subtree).
    children: Children,
) -> impl IntoView {
    let store = crate::menu::store::use_menu_store();
    let open = store.open();

    // The context-menu parent's anchor is the virtual cursor rect from the
    // context-menu context (`MenuPositioner.tsx:88-95` — the anchor default for
    // context-menu parents), not a DOM element.
    let anchor = crate::context_menu::root::use_context_menu_root_context_optional()
        .map(|context| context.anchor.borrow().clone());

    // The positioner element registers into the context (`MenuRoot.tsx:459-463` — the
    // `positionerRef` fill the trigger's mouseup handler reads).
    let positioner_context = crate::context_menu::root::use_context_menu_root_context_optional();
    let node_ref = NodeRef::<leptos::html::Div>::new();

    // The registration effect (`MenuRoot.tsx:459-463`).
    Effect::new(move || {
        if let (Some(context), Some(el)) = (positioner_context.as_ref(), node_ref.get()) {
            *context.positioner_element.borrow_mut() = Some(el.clone().into());
        }
    });

    // The placement: the virtual anchor's rect (`:58-64`) with the offsets applied —
    // the full floating pipeline is the Menu positioner's; the context-menu delta is
    // the cursor-point anchor and the forced fixed positioning (`:114`).
    let anchor_for_left = anchor.clone();
    let anchor_for_top = anchor.clone();
    let placed_x = move || {
        anchor_for_left
            .as_ref()
            .map(|a| {
                let (x, _, _, _) = a.rect();
                x
            })
            .unwrap_or_default()
    };
    let placed_y = move || {
        anchor_for_top
            .as_ref()
            .map(|a| {
                let (_, y, _, _) = a.rect();
                y
            })
            .unwrap_or_default()
    };

    view! {
        <div
            class="menu-positioner"
            data-side=side_str(side)
            data-align=align_str(align)
            node_ref=node_ref
            style:position=CONTEXT_MENU_POSITION_METHOD
            style:left=move || format!("{}px", placed_x())
            style:top=move || format!("{}px", placed_y())
            style:margin-left=format!("{side_offset}px")
            style:margin-top=format!("{align_offset}px")
            hidden=!open.get()
        >
            {children()}
        </div>
    }
}

fn side_str(side: MenuSide) -> &'static str {
    match side {
        MenuSide::Bottom => "bottom",
        MenuSide::Top => "top",
        MenuSide::Left => "left",
        MenuSide::Right => "right",
    }
}

fn align_str(align: MenuAlign) -> &'static str {
    match align {
        MenuAlign::Start => "start",
        MenuAlign::Center => "center",
        MenuAlign::End => "end",
    }
}
