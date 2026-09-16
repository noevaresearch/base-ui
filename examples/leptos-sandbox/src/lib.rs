//! The sandbox shell: pick a demo, mount it.
//!
//! Why this crate exists (the short version): the docs site proves the port renders, but a reader
//! cannot touch it. This app is what a CodeSandbox VM sandbox runs, so the reader can change the
//! Rust and watch the ported component change — with `base-ui-leptos` resolved from crates.io.
//! The long version, with the premise audit and the phases, is `SANDBOX-PLAN.md`.
//!
//! The route is a query parameter rather than a router: one demo is mounted per load
//! (`/?demo=accordion-hero`), which keeps this app small and makes the docs' "Open in CodeSandbox"
//! control a plain link.

use leptos::prelude::*;

pub mod demos;

#[component]
pub fn App() -> impl IntoView {
    let requested = requested_slug();
    let entry = demos::find(&requested);

    view! {
        <main class="mx-auto flex max-w-3xl flex-col gap-6 p-8 text-neutral-950 dark:text-white">
            <header class="flex flex-col gap-2">
                <h1 class="text-lg font-semibold">"Base UI — Leptos sandbox"</h1>
                <p class="text-sm text-neutral-600 dark:text-neutral-400">
                    "Every component below is the ported " <code>"base-ui-leptos"</code>
                    " crate, resolved from crates.io. Edit " <code>"src/demos.rs"</code>
                    " and save: the dev server rebuilds and this pane updates."
                </p>
            </header>

            <nav class="flex flex-wrap gap-2 text-sm" aria-label="Demos">
                {demos::ALL
                    .iter()
                    .map(|demo| {
                        let is_current = demo.slug == entry.map(|e| e.slug).unwrap_or_default();
                        let class = if is_current {
                            "rounded border border-neutral-950 bg-neutral-950 px-2 py-1 text-white dark:border-white dark:bg-white dark:text-neutral-950"
                        } else {
                            "rounded border border-neutral-300 px-2 py-1 hover:bg-neutral-100 dark:border-neutral-700 dark:hover:bg-neutral-800"
                        };
                        view! {
                            <a class=class href=format!("?demo={}", demo.slug)>
                                {demo.title}
                            </a>
                        }
                    })
                    .collect_view()}
            </nav>

            <section class="flex min-h-64 items-center justify-center rounded border border-neutral-200 p-8 dark:border-neutral-800">
                {match entry {
                    Some(demo) => (demo.mount)().into_any(),
                    None => view! {
                        <p class="text-sm text-neutral-600 dark:text-neutral-400">
                            "No demo named " <code>{requested}</code> " is registered yet."
                        </p>
                    }
                        .into_any(),
                }}
            </section>

            {entry
                .map(|demo| {
                    view! {
                        <footer class="text-xs text-neutral-600 dark:text-neutral-400">
                            <p>{demo.title} " — ported from " <code>{demo.upstream}</code></p>
                        </footer>
                    }
                })}
        </main>
    }
}

/// `?demo=<slug>` out of the current location, defaulting to the first registered demo.
fn requested_slug() -> String {
    let search = leptos::prelude::window()
        .location()
        .search()
        .unwrap_or_default();
    let slug = search
        .trim_start_matches('?')
        .split('&')
        .find_map(|pair| pair.strip_prefix("demo="))
        .map(str::to_string)
        .unwrap_or_default();
    if slug.is_empty() {
        demos::ALL
            .first()
            .map(|demo| demo.slug.to_string())
            .unwrap_or_default()
    } else {
        slug
    }
}

/// CSR entry point: cargo-leptos serves this lib cdylib as the front-end wasm, so the mount lives
/// here (the wasm32 bin `main` is never invoked by the loader).
#[cfg(all(target_arch = "wasm32", not(test)))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn leptos_sandbox_start() {
    use any_spawner::Executor;
    _ = Executor::init_wasm_bindgen();
    std::panic::set_hook(Box::new(|info| leptos::logging::error!("PANIC: {}", info)));
    leptos::mount::mount_to_body(App);
}
