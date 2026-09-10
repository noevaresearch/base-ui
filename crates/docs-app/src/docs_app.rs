use leptos::*;
use leptos::prelude::{ElementChild, ClassAttribute, use_context};

#[component]
pub fn App() -> impl IntoView {
    let context = use_context::<DocsContext>().unwrap();

    view! {
        <div class="docs-app">
            <header class="docs-header">
                <h1 class="docs-title">Base UI Documentation</h1>
            </header>
            <div class="docs-content">
                <p>Welcome to the Base UI Leptos documentation.</p>
                <p>{format!("Welcome to {}", context.title)}</p>
            </div>
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DocsContext {
    pub title: String,
}