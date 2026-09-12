use leptos::prelude::*;
use leptos_router::components::{Outlet, ParentRoute, Route, Routes, Router};
use leptos_router::StaticSegment;
pub mod pages;
use pages::csp_provider_page::CSPProviderPage;
use pages::direction_provider_page::DirectionProviderPage;
use pages::merge_props_page::MergePropsPage;
use pages::separator_page::SeparatorPage;
use pages::collapsible_page::CollapsibleHeroDemo;
use pages::toggle_page::TogglePage;
use pages::use_render_page::UseRenderPage;

#[cfg(all(test, target_arch = "wasm32"))]
mod render_test;

#[component]
pub fn App() -> impl IntoView {
    reactive_graph::owner::provide_context::<DocsContext>(DocsContext::default());

    view! {
        <Router>
            <main class="docs-app">
                <header class="docs-header">
                    <h1 class="docs-title">"Base UI Documentation"</h1>
                </header>
                <Routes fallback=|| "Not found">
                    <ParentRoute path=StaticSegment("") view=HomeLayout>
                        <Route path=StaticSegment("") view=HomePage />
                        <Route
                            path=StaticSegment("react/components/collapsible")
                            view=CollapsiblePage
                        />
                        <Route path=StaticSegment("react/utils/use-render") view=UseRenderPage />
                        <Route
                            path=StaticSegment("react/utils/csp-provider")
                            view=CSPProviderPage
                        />
                        <Route path=StaticSegment("react/components/toggle") view=TogglePage />
                        <Route
                            path=StaticSegment("react/utils/direction-provider")
                            view=DirectionProviderPage
                        />
                        <Route
                            path=StaticSegment("react/utils/merge-props")
                            view=MergePropsPage
                        />
                        <Route
                            path=StaticSegment("react/components/separator")
                            view=SeparatorPage
                        />
                    </ParentRoute>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomeLayout() -> impl IntoView {
    let (collapsed, set_collapsed) = signal(false);
    view! {
        <div class="docs-content">
            <button on:click=move |_| set_collapsed.update(|v| *v = !*v)>
                {move || if collapsed.get() { "Expand" } else { "Collapse" }}
            </button>
            {move || { if !collapsed.get() { Some(view! { <div>"Collapsible panel demo area"</div> }) } else { None } }}
            <Outlet />
        </div>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <p>"Welcome to the Base UI Leptos documentation."</p>
    }
}

/// The docs page for the Collapsible component, mirroring
/// `docs/src/app/(docs)/react/components/collapsible/page.mdx`
/// (H1 + Subtitle, hero demo, Anatomy, Examples, API reference).
#[component]
fn CollapsiblePage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Collapsible"</h1>
            <p class="subtitle">"A collapsible panel controlled by a button."</p>

            <CollapsibleHeroDemo />

            <h2>"Anatomy"</h2>
            <pre><code>
"import { Collapsible } from '@base-ui/react/collapsible';

<Collapsible.Root>
  <Collapsible.Trigger />
  <Collapsible.Panel />
</Collapsible.Root>"
            </code></pre>

            <h2>"Examples"</h2>
            <h3>"Hidden until found"</h3>
            <p>
                "The `hiddenUntilFound` prop hides the closed panel with `hidden=\"until-found\"` "
                "so the browser can search its contents with find-in-page and reveal the panel when a match is found."
            </p>
            <pre><code>
"<Collapsible.Root>
  <Collapsible.Trigger>Shipping details</Collapsible.Trigger>
  <Collapsible.Panel hiddenUntilFound>Standard shipping takes 3–5 business days.</Collapsible.Panel>
</Collapsible.Root>"
            </code></pre>

            <h2>"API reference"</h2>
            <h3>"Root"</h3>
            <h3>"Trigger"</h3>
            <h3>"Panel"</h3>
        </article>
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DocsContext {
    pub title: String,
}
