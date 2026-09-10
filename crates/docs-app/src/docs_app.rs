use leptos::*;
use leptos::prelude::{ElementChild, ClassAttribute, use_context};
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    reactive_graph::owner::provide_context::<DocsContext>(DocsContext::default());

    view! {
        <div class="docs-app">
            <header class="docs-header">
                <h1 class="docs-title">Base UI Documentation</h1>
            </header>
            <div class="docs-content">
                <p>Welcome to the Base UI Leptos documentation.</p>
            </div>
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DocsContext {
    pub title: String,
}