#[cfg(target_arch = "wasm32")]
use leptos::mount::mount_to_body;
#[cfg(target_arch = "wasm32")]
use leptos_sandbox::App;

// CSR-only (wasm): mounting needs a browser DOM. The host binary exists only so
// `cargo leptos build/serve` has a bin target; on the host the UI cannot run, so it says so
// instead of pretending. Same shape as `crates/docs-app/src/main.rs`.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("leptos-sandbox is a CSR (wasm) app; build it for wasm32-unknown-unknown to run it.");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    mount_to_body(App);
}
