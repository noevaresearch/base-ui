//! The port's canonical install reference — one source of truth for how the docs tell a reader to get this
//! library.
//!
//! WHY THIS MODULE EXISTS
//! ----------------------
//! Every mirrored page that shows an install line was free to invent its own, and upstream's pages say
//! `npm install @base-ui/react` — a DIFFERENT library (React, not this Rust/Leptos port). The alias
//! Rust crate: `base-ui-leptos` — PUBLISHED on crates.io at 0.1.1 (2026-09-16), so `cargo add base-ui-leptos`
//! is a real command and pages may show it.
//! JavaScript alias: `@noevaresearch/base-ui`, mapped locally (`packages/leptos/`, linked by `pnpm install`,
//! asserted by `test/node-resolution`) and deliberately NOT published. Pages name BOTH, with the JS side
//! described as the local alias it is — the previous wording said "not yet published" of the crate, which
//! stopped being true.
//! Pages render the text below; `ralph/scripts/check-package-alias.mjs` fails the regression if a page names
//! upstream's package instead.

/// The port's package name. Mirrors `packages/leptos/package.json`; asserted by the alias gate.
pub const PACKAGE_ALIAS: &str = "@noevaresearch/base-ui";

/// The Rust crate that ships the components today.
pub const RUST_CRATE: &str = "base-ui-leptos";

/// True since 2026-09-16: `base-ui-leptos` 0.1.1 is on crates.io. Pages may state the install command.
pub const PUBLISHED: bool = true;

/// The canonical install snippet for a mirrored page, rendered verbatim.
pub const INSTALL_SNIPPET: &str = "# Rust (this port today):\ncargo add base-ui-leptos\n\n# JavaScript package alias (mapped locally, not yet published):\n#   @noevaresearch/base-ui\n# To build from source instead:\ngit clone https://github.com/noevaresearch/base-ui && cd base-ui && cargo build -p base-ui-leptos";

/// One-line provenance line a page may show. Crediting the original work is allowed and expected.
pub const PROVENANCE: &str = "Ported from the React implementation of Base UI — the same behaviour and anatomy, expressed with Leptos signals and view! markup.";
