mod docs_app;

use leptos::mount::mount_to_body;
use leptos_router::components::{Router, Routes, Route};

fn main() {
    mount_to_body(|| {
        view! {
            <Router>
                <Routes>
                    <Route path="" view={|| docs_app::Home}/>
                    <Route path="/collapsible" view={|| docs_app::CollapsibleDemo}/>
                    <Route path="/:any/*" view={|| docs_app::NotFound}/>
                </Routes>
            </Router>
        }
    });
}