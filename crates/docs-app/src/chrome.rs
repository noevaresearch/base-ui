//! The docs-app's page chrome — the layout shell, the header and the side navigation that
//! every ported docs page renders inside.
//!
//! Ported from (each item names the upstream file it mirrors):
//!
//! * `docs/src/app/(docs)/layout.tsx` — the `RootLayout` / `RootLayoutContainer` /
//!   `RootLayoutContent` / `ContentLayoutRoot` grid, the `MainNav` "SideNav.Root" tree, and
//!   `main.ContentLayoutMain` (which upstream gives `id={MAIN_CONTENT_ID}`).
//! * `docs/src/components/Header.tsx` — the header, including `SkipNav` (first child), the
//!   `HeaderLogoLink` and the logo mark itself (`docs/src/components/Logo.tsx`, both `path`
//!   `d` attributes verbatim).
//! * `docs/src/components/SkipNav.tsx` — the skip link and `MAIN_CONTENT_ID`.
//! * `docs/src/components/SideNav.tsx` — `Root`/`Section`/`Heading`/`Separator`/`List`/`Item`;
//!   `Item` marks the current route with BOTH `data-active` and `aria-current`
//!   (`SideNav.tsx:105-115`), which is what the stylesheet's active pill and this module's
//!   tests key on.
//! * `docs/src/app/sitemap/index.ts` — the section grouping (`Components`, `Utils`, plus
//!   upstream's external-links section at the bottom of the nav).
//!
//! The matching stylesheet is `crates/docs-app/style/main.css` (compiled to
//! `/pkg/docs-app.css` by cargo-leptos, see `crates/docs-app/Cargo.toml`).
//!
//! SCOPE — the nav tree lists the routes this crate actually serves: the 14 ported components
//! and the 4 ported utils. Upstream's nav also carries `Overview` and `Handbook` sections; those
//! pages are the unported `docs-content-extra:` ledger items, so listing them here would link to
//! pages that do not exist (and the port may not fabricate a docs page — `CONTEXT.md`). When
//! those items land, they join [`NAV_SECTIONS`] and nothing else has to change.

use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::install_ref;

/// `MAIN_CONTENT_ID` (`docs/src/components/SkipNav.tsx:4`) — the skip link's target and the
/// `id` upstream puts on `main.ContentLayoutMain`.
pub const MAIN_CONTENT_ID: &str = "main-content";

/// One ported docs route, as its side-nav entry: upstream's `SideNav.Item` title (the MDX `<h1>`
/// / sitemap page title, `docs/src/utils/getDisplayTitle.ts`) and its `href`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavItem {
    pub title: &'static str,
    pub href: &'static str,
}

/// A `SideNav.Section` (`docs/src/components/SideNav.tsx:26-28`) with its `SideNavHeading`.
#[derive(Debug, Clone, Copy)]
pub struct NavSection {
    pub heading: &'static str,
    pub items: &'static [NavItem],
}

const COMPONENTS: &[NavItem] = &[
    NavItem {
        title: "Accordion",
        href: "/react/components/accordion",
    },
    NavItem {
        title: "Avatar",
        href: "/react/components/avatar",
    },
    NavItem {
        title: "Button",
        href: "/react/components/button",
    },
    NavItem {
        title: "Checkbox",
        href: "/react/components/checkbox",
    },
    NavItem {
        title: "Checkbox Group",
        href: "/react/components/checkbox-group",
    },
    NavItem {
        title: "Collapsible",
        href: "/react/components/collapsible",
    },
    NavItem {
        title: "Field",
        href: "/react/components/field",
    },
    NavItem {
        title: "Fieldset",
        href: "/react/components/fieldset",
    },
    NavItem {
        title: "Form",
        href: "/react/components/form",
    },
    NavItem {
        title: "Input",
        href: "/react/components/input",
    },
    NavItem {
        title: "Meter",
        href: "/react/components/meter",
    },
    NavItem {
        title: "OTP Field",
        href: "/react/components/otp-field",
    },
    NavItem {
        title: "Progress",
        href: "/react/components/progress",
    },
    NavItem {
        title: "Separator",
        href: "/react/components/separator",
    },
    NavItem {
        title: "Toggle",
        href: "/react/components/toggle",
    },
    NavItem {
        title: "Toggle Group",
        href: "/react/components/toggle-group",
    },
];

const UTILS: &[NavItem] = &[
    NavItem {
        title: "CSP Provider",
        href: "/react/utils/csp-provider",
    },
    NavItem {
        title: "Direction Provider",
        href: "/react/utils/direction-provider",
    },
    NavItem {
        title: "mergeProps",
        href: "/react/utils/merge-props",
    },
    NavItem {
        title: "useRender",
        href: "/react/utils/use-render",
    },
];

/// The nav tree's sections, in upstream's sitemap order for the routes the port serves.
pub const NAV_SECTIONS: &[NavSection] = &[
    NavSection {
        heading: "Components",
        items: COMPONENTS,
    },
    NavSection {
        heading: "Utils",
        items: UTILS,
    },
];

/// Upstream's nav footer (`layout.tsx:76-96`): the external links after a `SideNav.Separator`.
///
/// DEVIATION (recorded, not silent): upstream's second link is `npm` pointing at
/// `https://www.npmjs.com/package/@base-ui/react` — ANOTHER library's package page, which is what
/// `specs/docs-content/CONTRACT.md` requirement 6 forbids and what
/// `ralph/scripts/check-package-alias.mjs` fails a page for. This port's docs name THIS port, so the
/// link carries the PUBLISHED Rust crate ([`install_ref::RUST_CRATE`]) and points at its crates.io
/// page ([`install_ref::CRATES_IO_URL`]). There is deliberately no npm URL to link: the JS alias
/// `base-ui-leptos` is mapped locally and unpublished, so linking it would 404 — the honest
/// status is stated instead ([`install_ref::ALIAS_STATUS`], rendered as the link's `title` in the
/// header). Both links render on EVERY route, which is what makes this the docs' install reference:
/// it is rendered from the canonical constants in [`install_ref`] rather than hand-written text.
/// The GitHub entry keeps upstream's repository, which is the credit the port's provenance relies on.
pub const NAV_EXTERNAL: &[NavItem] = &[
    NavItem {
        title: "GitHub",
        href: "https://github.com/mui/base-ui",
    },
    NavItem {
        title: install_ref::RUST_CRATE,
        href: install_ref::CRATES_IO_URL,
    },
];

/// The docs header (`docs/src/components/Header.tsx`): the skip link, the logo mark as the
/// home link, and the port's own link surface.
#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="Header">
            <div class="HeaderInner">
                <a class="SkipNav" href="#main-content">
                    "Skip to contents"
                </a>
                <a class="HeaderLogoLink" href="/" aria-label="Go to the homepage">
                    // `docs/src/components/Logo.tsx` — both paths verbatim.
                    <svg
                        width="17"
                        height="24"
                        viewBox="0 0 17 24"
                        fill="currentColor"
                        aria-label="Base UI"
                    >
                        <path d="M9.5001 7.01537C9.2245 6.99837 9 7.22385 9 7.49999V23C13.4183 23 17 19.4183 17 15C17 10.7497 13.6854 7.27351 9.5001 7.01537Z" />
                        <path d="M8 9.8V12V23C3.58172 23 0 19.0601 0 14.2V12V1C4.41828 1 8 4.93989 8 9.8Z" />
                    </svg>
                </a>
                <nav class="HeaderLinks" aria-label="External links">
                    <a class="HeaderLink" href="https://github.com/mui/base-ui">
                        "GitHub"
                    </a>
                    <a
                        class="HeaderLink"
                        href={install_ref::CRATES_IO_URL}
                        title={install_ref::ALIAS_STATUS}
                    >
                        {install_ref::RUST_CRATE}
                    </a>
                </nav>
            </div>
        </header>
    }
}

/// One `SideNav.Item` (`docs/src/components/SideNav.tsx:83-122`). Upstream's `Item` derives the
/// active state from `usePathname() === href` and writes `aria-current` + `data-active` on the
/// link; `exact` matching is what makes "Checkbox" inactive while "Checkbox Group" is open.
///
/// TWO FORM TRAPS, both measured in the served page rather than assumed (this is the only place in
/// the crate that writes a dynamic custom attribute, so nothing else would have caught them):
///
/// 1. `attr:data-active=…` does NOT mean "the attribute named `data-active`" in leptos 0.7.9's
///    `view!` macro — the literal attribute lands in the DOM as `attr:data-active="…"` (verified by
///    dumping the rendered anchor's `outerHTML`), so the stylesheet's active pill
///    (`.SideNavLink[data-active]`, `main.css:735`) never matched. The hyphenated name WITHOUT the
///    prefix is the form tachys routes to `custom_attribute` (`leptos_macro/src/view/mod.rs:1132`).
/// 2. A reactive closure value (`attr:data-active=move || …`) additionally wrote nothing at all —
///    20 `.SideNavLink`s and zero `[data-active]`/`[aria-current]` attributes in the served DOM.
///
/// The state is therefore computed once per render inside a dynamic child, so the attribute values
/// are plain `Option<&str>` in each pass, and a location change re-renders the anchor.
fn nav_item(path: Signal<String>, item: NavItem) -> impl IntoView {
    let href = item.href;

    view! {
        <li class="SideNavItem">
            {move || {
                let active = path.get() == href;
                view! {
                    <a
                        class="SideNavLink"
                        href=href
                        data-active=active.then_some("true")
                        aria-current=active.then_some("true")
                    >
                        {item.title}
                    </a>
                }
            }}
        </li>
    }
}

/// An external nav item — upstream renders these with an icon and, for npm, the version badge
/// (`layout.tsx:88-93`); the icons are `docs/src/icons/*`, not ported here, so the link text
/// stands alone.
fn nav_external_item(item: NavItem) -> impl IntoView {
    view! {
        <li class="SideNavItem">
            <a class="SideNavLink" href=item.href rel="noreferrer" target="_blank">
                {item.title}
            </a>
        </li>
    }
}

/// The side navigation (`docs/src/components/SideNav.tsx` + the sitemap tree in
/// `docs/src/app/(docs)/layout.tsx:64-96`).
#[component]
pub fn SideNav(
    /// The pathname to mark as the current page. The app leaves this unset, so the router's
    /// location is used (upstream's `usePathname()`); tests pin it so the active item can be
    /// asserted without standing up a `Router`.
    #[prop(optional)]
    current_path: Option<String>,
) -> impl IntoView {
    let path: Signal<String> = match current_path {
        Some(fixed) => Signal::stored(fixed),
        None => {
            let location = use_location();
            Signal::derive(move || location.pathname.get())
        }
    };

    let sections = NAV_SECTIONS
        .iter()
        .map(|section| {
            view! {
                <div class="SideNavSection">
                    <div class="SideNavHeading">{section.heading}</div>
                    <ul class="SideNavList">
                        {section
                            .items
                            .iter()
                            .copied()
                            .map(|item| nav_item(path, item))
                            .collect_view()}
                    </ul>
                </div>
            }
        })
        .collect_view();

    view! {
        <nav class="SideNavRoot" aria-label="Main navigation">
            <div class="SideNavViewport" data-side-nav-viewport="">
                {sections}
                <hr class="SideNavSeparator" />
                <div class="SideNavSection">
                    <ul class="SideNavList">
                        {NAV_EXTERNAL.iter().copied().map(nav_external_item).collect_view()}
                    </ul>
                </div>
            </div>
        </nav>
    }
}

/// The layout shell (`docs/src/app/(docs)/layout.tsx`): every ported route renders inside it,
/// with the header and side nav around the page's `Outlet`.
#[component]
pub fn DocsLayout() -> impl IntoView {
    use leptos_router::components::Outlet;

    view! {
        <div class="RootLayout">
            <div class="RootLayoutContainer">
                <div class="RootLayoutContent">
                    <div class="ContentLayoutRoot">
                        <Header />
                        <SideNav />
                        <main class="ContentLayoutMain" id=MAIN_CONTENT_ID>
                            <Outlet />
                        </main>
                    </div>
                </div>
            </div>
        </div>
    }
}
