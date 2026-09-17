//! Menu portal — `Menu.Portal`, the port of `packages/react/src/menu/portal/MenuPortal.tsx`
//! and `MenuPortalContext.ts`.
//!
//! WHAT THIS REPLACES. The previous `portal.rs` was a facade, not a port: it rendered a
//! wrapper `<div class="menu-portal">` with a hardcoded class upstream has no counterpart
//! for, hid it with a `hidden` attribute upstream never sets, and exported five invented
//! helpers (`use_menu_portal_keep_mounted`, `use_menu_portal_container`,
//! `create_portal_container`, `append_portal_container_to_body`, `remove_portal_container`)
//! that returned constants or re-implemented DOM plumbing the shared portal machinery
//! already owns. Upstream's Portal renders NO element of its own: it returns `null` while
//! unmounted and portals into a `data-base-ui-portal` host otherwise
//! (`MenuPortal.tsx:24-30` over `FloatingPortal.tsx:127-143`) — which is what
//! `specs/library/menu/behavior.md` → "Uniform DOM shell" requires, and what the fabricated
//! wrapper violated.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from:
//! - the render gate `mounted || keepMounted` (`MenuPortal.tsx:22-27`); `false` renders
//!   nothing (upstream's `return null`).
//! - the `MenuPortalContext` provider carrying `keepMounted` (`MenuPortal.tsx:35`;
//!   `MenuPortalContext.ts:4` types it `boolean | undefined`), consumed (required) by
//!   `Menu.Positioner` (`MenuPositioner.tsx:60`) — the throw when absent is
//!   `MenuPortalContext.ts:8-10`.
//! - `portalOwnerRole` (`MenuPortal.tsx:29-32`): `'group'` only under role-constrained
//!   parents (`menu`/`menubar`), absent otherwise. Upstream's comment at `:29-31` says the
//!   role is decided by the CONTEXT parent, not the store's `parent` (which a detached
//!   trigger overwrites with its own) — see the deferral note below for how the port
//!   currently sources it.
//! - the portal host and its container resolution, via the already-ported shared machinery
//!   (`provide_floating_portal`, the `FloatingPortal.tsx` port in `leptos-ui-internals`):
//!   the `data-base-ui-portal` host div, its generated id (`FloatingPortal.tsx:127-136`),
//!   and the element / ref / `null`-wait / `document.body` container precedence
//!   (`FloatingPortal.tsx:95-125`).
//! - the children, mounted INTO that host — upstream's `createPortal(children,
//!   portalElement)` at `FloatingPortal.tsx:272`.
//!
//! THE CHILD MOUNT, stated precisely because it is the one piece with no precedent in this
//! crate: the shared machinery is view-free and hands the consumer the host element
//! (`FloatingPortalNode::node`, "the consumer appends into `node()`" —
//! `floating_portal.rs:53-55`). The port mounts the children subtree into that host with
//! `leptos::mount::mount_to`, the same primitive leptos's own `Portal` component uses
//! internally (`leptos-0.7.8/src/portal.rs:64-70`), with the same per-run
//! `Owner::on_cleanup` disposal (`:72-78`). It deliberately does NOT go through
//! `leptos::portal::Portal`, because that component creates an extra wrapper `<div>` inside
//! its mount target (`portal.rs:47-62`) which would sit between the `data-base-ui-portal`
//! host and the positioner — a DOM level upstream does not have.
//!
//! DEFERRED, each with its reason and none of them silently dropped:
//! - The focus-guard spans and the hidden `aria-owns` owner (`FloatingPortal.tsx:254-293`)
//!   are exposed by the ported machinery on `FloatingPortalHandle`, but the port does not
//!   place them: they are realized only while a NON-modal `FloatingFocusManager` inside the
//!   portal pushes its state up through the portal context (`FloatingPortal.tsx:194-195`,
//!   `:270`), and the port's popup does not wire that focus manager yet (the popover
//!   precedent defers the same producer — `popover/parts.rs:455-467`). While no producer
//!   exists, `shouldRenderGuards` cannot turn true, so placing them would render elements
//!   upstream does not render.
//! - The context-vs-store `parent` distinction upstream's `:29-31` comment draws: the port's
//!   `MenuRootContextValue` carries only `store` (`store.rs:183-187`), and the parent the
//!   port has is the store's (`MenuExtraState::parent`, `store.rs:80-81`). `portalOwnerRole`
//!   needs only the discriminant, so this does not change the role today; it becomes a real
//!   difference only for a DETACHED trigger that overwrites the store parent, and it is
//!   recorded in `ralph/logs/spec-discrepancies.md` rather than papered over by adding a
//!   field the root does not yet compute.
//! - `container` accepts an element only. Upstream's union includes a `ShadowRoot` and a
//!   `RefObject` (`MenuPortal.tsx:47-50`); the ported machinery expresses the ref case as
//!   `FloatingPortalContainer::Resolve`, and this part's public prop reaches the element
//!   arm. The ref/shadow arms of the container are the machinery's, not this part's.

use std::sync::Arc;

use leptos::prelude::*;
use leptos_ui_internals::floating_ui::floating_portal::{
    FloatingPortalContainer, FloatingPortalOptions, ResolvedContainer, provide_floating_portal,
};
use leptos_ui_internals::floating_ui::popup_store::selectors;
use reactive_graph::owner::Owner;
use reactive_graph::traits::Get;

use crate::menu::store::{MenuParent, use_menu_store};

/// The render gate (`MenuPortal.tsx:22-27`): `mounted || keepMounted`. `false` is
/// upstream's `return null` — the port renders nothing at all, not a hidden wrapper.
pub fn menu_portal_should_render(mounted: bool, keep_mounted: bool) -> bool {
    mounted || keep_mounted
}

/// `portalOwnerRole` (`MenuPortal.tsx:29-32`): the hidden `aria-owns` owner needs `group`
/// only under role-constrained parents — `menu` (a submenu) or `menubar`.
pub fn menu_portal_owner_role(parent: &MenuParent) -> Option<&'static str> {
    match parent {
        MenuParent::Menu { .. } | MenuParent::Menubar => Some("group"),
        MenuParent::None | MenuParent::ContextMenu => None,
    }
}

/// The `MenuPortalContext` value (`MenuPortalContext.ts:4`): upstream's context carries the
/// `keepMounted` boolean itself, so the port carries it in a one-field value — which is
/// what lets "provider with `false`" be distinguished from "no provider" (upstream's
/// `undefined`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuPortalContextValue {
    /// `keepMounted` (`MenuPortal.tsx:20`, default `false`) as the provider publishes it
    /// (`:35`).
    pub keep_mounted: bool,
}

/// `useMenuPortalContext` (`MenuPortalContext.ts:6-12`): the required read throws when no
/// `<Menu.Portal>` is an ancestor; the optional form returns `None` instead — the shape
/// [`MenuPositioner`](crate::menu::positioner::Positioner) consumes.
pub fn use_menu_portal_context(
    value: Option<MenuPortalContextValue>,
    optional: bool,
) -> Option<MenuPortalContextValue> {
    match value {
        Some(value) => Some(value),
        None if optional => None,
        None => panic!("Base UI: <Menu.Portal> is missing."),
    }
}

/// The pre-rewrite flat name for this part, kept as an alias so existing call sites keep
/// compiling (`context_menu/mod.rs:34` re-exports it under its own part name).
pub use self::Portal as MenuPortal;

/// The `MenuPortalContext` read against the current reactive owner
/// (`MenuPortalContext.ts:6`).
pub fn menu_portal_context(optional: bool) -> Option<MenuPortalContextValue> {
    let value = use_context::<MenuPortalContextValue>();
    use_menu_portal_context(value, optional)
}

/// `Menu.Portal` — upstream's `MenuPortal` (`MenuPortal.tsx:17-45`).
///
/// Renders nothing while `mounted || keepMounted` is false and otherwise portals its
/// children into the shared machinery's `data-base-ui-portal` host. See the module docs for
/// what is ported and what is deferred.
#[component]
pub fn Portal(
    /// `keepMounted` (`MenuPortal.tsx:20`, `@default false`) — keeps the portal host in the
    /// DOM while the popup is hidden.
    #[prop(default = false)]
    keep_mounted: bool,
    /// `container` (`MenuPortal.tsx:47-50`) — the element to render the portal into;
    /// absent means the machinery's own precedence (parent portal node, else
    /// `document.body`).
    #[prop(optional)]
    container: Option<web_sys::Element>,
    /// The portaled subtree.
    children: ChildrenFn,
) -> impl IntoView {
    // `const { store, parent } = useMenuRootContext()` (`MenuPortal.tsx:25`).
    let context = use_menu_store();
    let store = context.store;

    // `const mounted = store.useState('mounted')` (`:26`).
    let mounted = store.use_state(selectors::mounted);

    // The parent the role is computed from (`:32`). Read reactively so a parent
    // re-classification re-decides the role (the extra state is the store's slot for it,
    // `store.rs:80-81` — see the module docs on the context/store distinction).
    let parent = store.use_state(|state| {
        selectors::payload(state)
            .map(|extra| extra.parent)
            .unwrap_or(MenuParent::None)
    });

    // `<MenuPortalContext.Provider value={keepMounted}>` (`:35`).
    provide_context(MenuPortalContextValue { keep_mounted });

    let should_render = move || menu_portal_should_render(mounted.get(), keep_mounted);
    // `portalOwnerRole` (`:29-32`).
    let owner_role = move || menu_portal_owner_role(&parent.get());

    view! {
        {move || {
            if !should_render() {
                return None;
            }
            // The mounted half takes the container arm as an optional prop, so the two
            // element/no-element shapes are built explicitly (leptos's `optional` props carry
            // `Option<T>` fields but take `T`).
            let mounted_view = match container.clone() {
                Some(element) => view! {
                    <MenuPortalMount
                        container=element
                        portal_owner_role=owner_role()
                        children=children.clone()
                    />
                }
                .into_any(),
                None => view! {
                    <MenuPortalMount portal_owner_role=owner_role() children=children.clone() />
                }
                .into_any(),
            };
            Some(mounted_view)
        }}
    }
}

/// The mounted half of `MenuPortal` — upstream's `<FloatingPortal ref={forwardedRef}
/// {...portalProps} portalOwnerRole={portalOwnerRole}>` (`MenuPortal.tsx:37-42`) with the
/// ported machinery, separated so the host exists only while the gate is open (upstream's
/// conditional render) and is torn down with the component that created it.
#[component]
fn MenuPortalMount(
    /// `container` (`MenuPortal.tsx:47-50`), already resolved to the element arm.
    #[prop(optional)]
    container: Option<web_sys::Element>,
    /// `portalOwnerRole` (`:32`) — `Some("group")` under role-constrained parents.
    #[prop(default = None)]
    portal_owner_role: Option<&'static str>,
    children: ChildrenFn,
) -> impl IntoView {
    // The container arm the machinery takes: an explicit element re-resolves on every
    // layout-effect run (`FloatingPortal.tsx:107`), a `null` current falling through the
    // same way `undefined` does — the resolver shape is what expresses that
    // (`floating_portal.rs:153-180`).
    let portal_container = match container {
        Some(element) => FloatingPortalContainer::Resolve(std::rc::Rc::new(move || {
            ResolvedContainer::Node(element.clone().into())
        })),
        None => FloatingPortalContainer::Auto,
    };

    // The portal host (`FloatingPortal.tsx:165-181` over `:127-143`). Kept in a
    // `StoredValue` so it lives exactly as long as this component's owner and is dropped
    // (running the machinery's own teardown) with it.
    let handle = provide_floating_portal(FloatingPortalOptions {
        container: portal_container,
        node_ref: None,
        id: None,
        element_props: Vec::new(),
        portal_owner_role: portal_owner_role.map(str::to_owned),
    });
    let node = handle.node.clone();
    let _handle = StoredValue::new_local(handle);

    // The children portal (`FloatingPortal.tsx:272`): append the subtree into the host.
    // The effect re-runs when the host appears or is rebuilt on a container change, and the
    // cleanup registered inside each run disposes the previous mount — the pattern leptos's
    // own `Portal` uses (`portal.rs:44-79`).
    {
        let children = Arc::clone(&children);
        Effect::new(move |_| {
            let Some(host) = node.node() else {
                // The container is unresolved (`FloatingPortal.tsx:97-104`'s wait state):
                // upstream renders nothing here too.
                return;
            };
            let subtree = send_wrapper::SendWrapper::new(leptos::mount::mount_to(host, {
                let children = Arc::clone(&children);
                move || children()
            }));
            Owner::on_cleanup(move || {
                drop(subtree.take());
            });
        });
    }
}
