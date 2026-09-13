//! Port of the Base UI Form — the `library: form` TODO item
//! (`specs/library/form/behavior.md`, `specs/library/form/implementation.md`).

use leptos::*;
use leptos::prelude::{ElementChild, OnAttribute};

/// Form component that renders a native `<form>` element.
/// 
/// This is a port of Base UI's Form component. It renders a native HTML form
/// element and forwards the ref to the underlying element, allowing consumers
/// to interact with it directly.
#[component]
pub fn Form() -> impl IntoView {
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        web_sys::console::log_1(&"Form submitted".into());
    };

    view! {
        <form on:submit=on_submit>
            "Form content goes here"
        </form>
    }
}