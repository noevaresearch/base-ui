use leptos::prelude::*;
use leptos_router::StaticSegment;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
pub mod chrome;
pub mod pages;
pub mod reference;
use chrome::DocsLayout;
use pages::accordion_page::AccordionPage;
use pages::avatar_page::AvatarPage;
use pages::button_page::ButtonPage;
use pages::checkbox_group_page::CheckboxGroupPage;
use pages::checkbox_page::CheckboxPage;
use pages::collapsible_page::CollapsibleHeroDemo;
use pages::csp_provider_page::CSPProviderPage;
use pages::direction_provider_page::DirectionProviderPage;
use pages::field_page::FieldPage;
use pages::fieldset_page::FieldsetPage;
use pages::form_page::FormPage;
use pages::merge_props_page::MergePropsPage;
use pages::meter_page::MeterPage;
use pages::otp_field_page::OtpFieldPage;
use pages::progress_page::ProgressPage;
use pages::separator_page::SeparatorPage;
use pages::toggle_page::TogglePage;
use pages::use_render_page::UseRenderPage;

#[cfg(all(test, target_arch = "wasm32"))]
pub mod render_test;

#[component]
pub fn App() -> impl IntoView {
    reactive_graph::owner::provide_context::<DocsContext>(DocsContext::default());

    view! {
        <Router>
            <Routes fallback=|| "Not found">
                <ParentRoute path=StaticSegment("") view=DocsLayout>
                        <Route path=StaticSegment("") view=HomePage />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("accordion"))
                            view=AccordionPage
                        />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("button"))
                            view=ButtonPage
                        />
                        <Route path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("avatar")) view=AvatarPage />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("checkbox"))
                            view=CheckboxPage
                        />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("checkbox-group"))
                            view=CheckboxGroupPage
                        />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("collapsible"))
                            view=CollapsiblePage
                        />
                        <Route path=(StaticSegment("react"), StaticSegment("utils"), StaticSegment("use-render")) view=UseRenderPage />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("utils"), StaticSegment("csp-provider"))
                            view=CSPProviderPage
                        />
                        <Route path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("toggle")) view=TogglePage />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("utils"), StaticSegment("direction-provider"))
                            view=DirectionProviderPage
                        />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("utils"), StaticSegment("merge-props"))
                            view=MergePropsPage
                        />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("separator"))
                            view=SeparatorPage
                        />
                        <Route path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("meter")) view=MeterPage />
                        <Route path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("field")) view=FieldPage />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("fieldset"))
                            view=FieldsetPage
                        />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("otp-field"))
                            view=OtpFieldPage
                        />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("progress"))
                            view=ProgressPage
                        />
                        <Route
                            path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("form"))
                            view=FormPage
                        />
                    </ParentRoute>
                </Routes>
        </Router>
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

/// CSR entry point: cargo-leptos serves the lib cdylib as the front-end wasm, so the
/// mount must live here (the wasm32 bin `main` is never invoked by the loader).
#[cfg(all(target_arch = "wasm32", not(test)))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn docs_app_start() {
    use any_spawner::Executor;
    _ = Executor::init_wasm_bindgen();
    std::panic::set_hook(Box::new(|info| {
        leptos::logging::error!("PANIC: {}", info);
    }));
    leptos::mount::mount_to_body(App);
}
