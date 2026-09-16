# `leptos-sandbox` — the browser sandbox

This is the app that runs in a browser IDE so a reader can **edit the ported Rust and watch the
component change**. It is not part of the port's cargo workspace (see the `[workspace]` note in
`Cargo.toml`): it depends on `base-ui-leptos` **from crates.io**, exactly as a stranger would, so
that whatever it renders is proof the *published* crate works.

Plan, premise audit and phases: `SANDBOX-PLAN.md` at the repository root.

## Run it locally

```bash
cd examples/leptos-sandbox
cargo leptos serve          # http://127.0.0.1:3000/?demo=accordion-hero
cargo leptos watch          # same, rebuilding on save
```

Requirements: Rust stable, `wasm32-unknown-unknown`, and `cargo-leptos` 0.3.7 + `wasm-bindgen-cli`
0.2.128 (the version in the app's `Cargo.lock` — a mismatch fails at the wasm-bindgen step). On a
fresh machine `.devcontainer/setup.sh` installs exactly those.

## Add a demo

1. Write the demo against the **published** API, in the spelling the docs snippets teach —
   `use leptos_ui::Accordion;` and `<Accordion::Root>` / `<Accordion::Trigger>` / `<Accordion::Panel>`,
   not the older `AccordionRoot`-style names. Class strings come from the docs page verbatim; the
   Tailwind browser runtime in `index.html` is what makes them live here.
2. Register it in `demos.rs`: one `Demo` entry (`slug`, `title`, `upstream`, `mount`). The slug is
   what `?demo=<slug>` and the docs' "Open in CodeSandbox" link use.

A ported demo that is not in this registry is unfinished work, not a finished demo — that is the
owner's decision recorded in `SANDBOX-PLAN.md` §0, so keep the registry and the docs pages in sync.

## How this is served in a browser IDE

`.devcontainer/` (image + toolchain, pinned) and `.codesandbox/tasks.json` (setup task, dev server
on port 3000 as the preview) are what make a CodeSandbox **VM sandbox** boot this project, start the
dev server, and land the visitor on the running app. A synced template discards its memory snapshot
on every commit to the backing branch, which is why the sandbox is not backed by the branch the port
loop commits to every few minutes.
