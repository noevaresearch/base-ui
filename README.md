# `leptos-sandbox` — the browser sandbox

This is the app that runs in a browser IDE so a reader can **edit the ported Rust and watch the
component change**. It is not part of the port's cargo workspace (see the `[workspace]` note in
`Cargo.toml`): it depends on `base-ui-leptos` **from crates.io**, exactly as a stranger would, so
that whatever it renders is proof the *published* crate works.

Plan, premise audit and phases: `SANDBOX-PLAN.md` at the repository root.

## Run it locally

```bash
cd examples/leptos-sandbox
trunk serve --release          # http://127.0.0.1:3000/?demo=accordion-hero
trunk serve --port 3010        # if 3000 is taken on your machine
```

Requirements: Rust stable, the `wasm32-unknown-unknown` target, and Trunk 0.21.14. Trunk fetches the
matching `wasm-bindgen` and `wasm-opt` itself from the pins in `Trunk.toml`; on a fresh machine
`.devcontainer/setup.sh` installs Trunk and pre-builds the app.

**Why Trunk and not cargo-leptos** (which the docs app uses): a CSR-only crate built by cargo-leptos
emits just the compiled bundle (`pkg/*`) and no index document in its site root, so its dev server
serves nothing at `/` — the docs app worked around that with a custom static server
(`ralph/scripts/serve-docs-app.py`). Trunk takes `index.html` as the entry point, serves it, and does
live reload **on the same port**, which is the only shape that survives CodeSandbox's single-port
preview proxy.

## Add a demo

1. Write the demo against the **published** API, in the spelling the docs snippets teach —
   `use leptos_ui::Accordion;` and `<Accordion::Root>` / `<Accordion::Trigger>` / `<Accordion::Panel>`.
   Component props take `Option<String>` (`class=Some(...)`). Class strings come from the docs page
   verbatim; the Tailwind browser runtime in `index.html` is what makes them live here.
2. Register it in `demos.rs`: one `Demo` entry (`slug`, `title`, `upstream`, `mount`). The slug is
   what `?demo=<slug>` and the docs' "Open in CodeSandbox" link use.

> **Known duplication — do not let it rot.** The demo bodies here are currently hand-copied from
> `crates/docs-app/src/pages/*.rs`, so a demo fixed in the docs page does not fix this one. The
> structural fix is one demos crate consumed by both (the docs workspace resolving it against the
> local path via `[patch.crates-io]`, the sandbox against crates.io), which is how upstream keeps one
> artifact behind its rendered demo, its code panel and its StackBlitz export.
> Until then, a demo present in the docs but missing here is unfinished work, not a finished demo.

## How this is served in a browser IDE

`.devcontainer/` (image + toolchain, pinned) and `.codesandbox/tasks.json` (setup task, dev server
on port 3000 as the preview) are what make a CodeSandbox **VM sandbox** boot this project, start the
dev server, and land the visitor on the running app. `setupTasks` pre-builds the release profile
because CodeSandbox waits only 60 seconds for the preview port. A synced template discards its
memory snapshot on every commit to the backing branch, which is why the sandbox is not backed by the
branch the port loop commits to every few minutes.
