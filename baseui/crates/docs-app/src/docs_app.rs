use leptos::prelude::*;
use leptos_router::components::{Router, Routes, Route, Link, A};
use leptos_ui::collapsible::{Collapsible, CollapsibleTrigger, CollapsiblePanel};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="docs-app">
            <header class="docs-header">
                <h1 class="docs-title">"Base UI Documentation"</h1>
                <nav class="docs-nav">
                    <Link href="" class="nav-link">"Home"</Link>
                    <Link href="/collapsible" class="nav-link">"Collapsible"</Link>
                </nav>
            </header>
            <main class="docs-content">
                <Routes>
                    <Route path="" view={Home}/>
                    <Route path="/collapsible" view={CollapsibleDemo}/>
                    <Route path="/:any/*" view={NotFound}/>
                </Routes>
            </main>
        </div>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <div class="home">
            <h2>"Welcome to Base UI Leptos Documentation"</h2>
            <p>"This is a documentation application for the Leptos port of Base UI components."</p>
            <div class="demos">
                <h3>"Available Demos"</h3>
                <ul>
                    <li>
                        <Link href="/collapsible" class="demo-link">"Collapsible Component"</Link>
                        <p>"Demonstrates the collapsible component functionality."</p>
                    </li>
                </ul>
            </div>
        </div>
    }
}

#[component]
fn CollapsibleDemo() -> impl IntoView {
    view! {
        <div class="demo-page">
            <h2>"Collapsible Component Demo"</h2>
            <p>"This demo shows the collapsible component in action."</p>
            
            <div class="demo-section">
                <h3>"Basic Collapsible"</h3>
                <Collapsible>
                    <CollapsibleTrigger class="trigger-button">
                        <button>"Click to toggle"</button>
                    </CollapsibleTrigger>
                    <CollapsiblePanel class="panel-content">
                        <p>"This content can be shown or hidden."</p>
                    </CollapsiblePanel>
                </Collapsible>
            </div>

            <div class="demo-section">
                <h3>"Collapsible with Initial State"</h3>
                <Collapsible open>
                    <CollapsibleTrigger class="trigger-button">
                        <button>"This one starts open"</button>
                    </CollapsibleTrigger>
                    <CollapsiblePanel class="panel-content">
                        <p>"This content is visible by default."</p>
                    </CollapsiblePanel>
                </Collapsible>
            </div>

            <div class="demo-section">
                <h3>"Code Example"</h3>
                <pre class="code-example">
                    r#"
&lt;Collapsible&gt;
    &lt;CollapsibleTrigger&gt;
        &lt;button&gt;"Click to toggle"&lt;/button&gt;
    &lt;/CollapsibleTrigger&gt;
    &lt;CollapsiblePanel&gt;
        &lt;p&gt;"Content goes here."&lt;/p&gt;
    &lt;/CollapsiblePanel&gt;
&lt;/Collapsible&gt;
"#
                </pre>
            </div>
        </div>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="not-found">
            <h2>"404 - Page Not Found"</h2>
            <p>"The page you're looking for doesn't exist."</p>
            <A href="/" class="home-link">"Return Home"</A>
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DocsContext {
    pub title: String,
}