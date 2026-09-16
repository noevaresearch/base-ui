//! The port's canonical install reference — one source of truth for how the docs tell a reader to get this
//! library.
//!
//! WHY THIS MODULE EXISTS
//! ----------------------
//! Every mirrored page that shows an install line was free to invent its own, and upstream's pages say
//! `npm install @base-ui/react` — a DIFFERENT library (React, not this Rust/Leptos port). The alias
//! `@noevaresearch/base-ui` is mapped locally: `packages/leptos/` is a pnpm workspace package (matched by
//! `packages/*` in `pnpm-workspace.yaml`), `pnpm install` links it into `node_modules`, and
//! `test/node-resolution` asserts it resolves — but it is `private: true` and NOT published. So the honest
//! instruction names the crate plus the alias and states the publication status instead of implying it.
//! Pages render the text below; `ralph/scripts/check-package-alias.mjs` fails the regression if a page names
//! upstream's package instead.

/// The port's package name. Mirrors `packages/leptos/package.json`; asserted by the alias gate.
pub const PACKAGE_ALIAS: &str = "@noevaresearch/base-ui";

/// The Rust crate that ships the components today.
pub const RUST_CRATE: &str = "base-ui-leptos";

/// False while the crate is unpublished: pages must say so rather than implying an npm release exists.
pub const PUBLISHED: bool = false;

/// The canonical install snippet for a mirrored page, rendered verbatim.
pub const INSTALL_SNIPPET: &str = "# Rust (this port today):\ncargo add base-ui-leptos\n\n# JavaScript package alias (mapped locally, not yet published):\n#   @noevaresearch/base-ui\n# To build from source instead:\ngit clone https://github.com/noevaresearch/base-ui && cd base-ui && cargo build -p base-ui-leptos";

/// One-line provenance line a page may show. Crediting the original work is allowed and expected.
pub const PROVENANCE: &str = "Ported from the React implementation of Base UI — the same behaviour and anatomy, expressed with Leptos signals and view! markup.";
