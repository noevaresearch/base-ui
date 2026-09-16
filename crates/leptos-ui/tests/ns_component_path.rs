// Compile-time pin: Leptos' view! macro accepts a NAMESPACED PATH as a component name.
//
// WHY THIS PIN EXISTS
// -------------------
// Upstream's docs teach `<Checkbox.Root><Checkbox.Indicator /></Checkbox.Root>`, so the port's
// equivalent spelling is `<Checkbox::Root>` — a module path whose final segment is the part name.
// That form is the whole ergonomics argument for exposing the parts as a named module per component
// rather than as flattened `checkbox_root_view(..)` helpers: the example a reader sees is then the
// same tree with Rust's path separator instead of React's dot.
//
// Measured 2026-09-16 against leptos 0.7.9: `<Checkbox::Root>` / `<Checkbox::Indicator />` inside
// view! compiles and runs (this test). The first attempt failed on a component with no `children`
// prop — that was the nesting, not the path; the path form itself is accepted.
//
// If a Leptos upgrade ever drops path node names, this test fails and the ergonomics plan in
// specs/docs-content/CONTRACT.md needs a different spelling — which is exactly what a pin is for.
use leptos::prelude::*;

mod Checkbox {
    use leptos::prelude::*;

    #[component]
    pub fn Root(#[prop(optional)] children: Option<Children>) -> impl IntoView {
        view! { <span data-part="root">{children.map(|c| c())}</span> }
    }

    #[component]
    pub fn Indicator() -> impl IntoView {
        view! { <span data-part="indicator">""</span> }
    }
}

#[component]
fn Host() -> impl IntoView {
    view! {
        <div>
            <Checkbox::Root>
                <Checkbox::Indicator />
            </Checkbox::Root>
        </div>
    }
}

#[test]
fn namespaced_component_path_compiles() {
    // Reaching this line proves the macro accepted `<Checkbox::Root>` and `<Checkbox::Indicator />`.
    let _ = Host;
    assert!(true);
}
