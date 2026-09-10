mod docs_app;

use leptos::mount::mount_to_body;

fn main() {
    mount_to_body(|| {
        docs_app::App()
    });
}