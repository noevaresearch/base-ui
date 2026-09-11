#[cfg(target_arch = "wasm32")]
use docs_app::App;

#[cfg(target_arch = "wasm32")]
use leptos::mount::mount_to_body;

// The app is CSR-only (wasm): mounting requires a browser DOM. The host
// binary exists only so `cargo leptos build/serve` has a bin target; the
// host build never runs the UI (it cannot — no DOM). Do not "fix" this by
// adding an SSR server here; that is Phase D/docs-infra scope.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("docs-app is a CSR (wasm) app; build the lib for wasm32-unknown-unknown to run it.");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    mount_to_body(App);
}
