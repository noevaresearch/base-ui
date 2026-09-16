use leptos::prelude::*;
use leptos_router::StaticSegment;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
pub mod chrome;
pub mod code_block;
pub mod install_ref;
pub mod pages;
pub mod reference;
#[cfg(test)]
pub mod snippet_language;
pub mod status_data;
// The demo-styling drift guard: the ported pages carry upstream's tailwind-variant class strings,
// and this crate's stylesheet is what makes them live — so a class with no rule renders inert while
// every structural check stays green. Host tests, no browser (see the module docs).
#[cfg(test)]
pub mod demo_styles;
use chrome::DocsLayout;
use pages::accordion_page::AccordionPage;
use pages::avatar_page::AvatarPage;
use pages::button_page::ButtonPage;
use pages::checkbox_group_page::CheckboxGroupPage;
use pages::checkbox_page::CheckboxPage;
use pages::collapsible_page::CollapsiblePage;
use pages::csp_provider_page::CSPProviderPage;
use pages::direction_provider_page::DirectionProviderPage;
use pages::field_page::FieldPage;
use pages::fieldset_page::FieldsetPage;
use pages::form_page::FormPage;
use pages::input_page::InputPage;
use pages::merge_props_page::MergePropsPage;
use pages::meter_page::MeterPage;
use pages::otp_field_page::OtpFieldPage;
use pages::progress_page::ProgressPage;
use pages::separator_page::SeparatorPage;
use pages::status_page::StatusPage;
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
                        <Route path=StaticSegment("status") view=StatusPage />
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
                        <Route
                            path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("input"))
                            view=InputPage
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
