use leptos::mount::mount_to_body;
use leptos_sandbox::App;

// Trunk builds THIS binary for wasm32-unknown-unknown (`data-trunk rel="rust"` in `index.html`), so
// the mount lives here rather than in a `#[wasm_bindgen(start)]` fn in the lib: one entry point, no
// chance of the app mounting twice (a module-level `start` in a linked lib fires at instantiation
// *and* the bin's `main` runs).
//
// The host build is not a server and never was: a CSR app has no UI without a DOM. It exists so
// `cargo check` / `cargo test` work on the host.
fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use any_spawner::Executor;
        _ = Executor::init_wasm_bindgen();
        std::panic::set_hook(Box::new(|info| leptos::logging::error!("PANIC: {}", info)));
        mount_to_body(App);
    }

    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("leptos-sandbox is a CSR (wasm) app; build it for wasm32-unknown-unknown to run it.");
}
