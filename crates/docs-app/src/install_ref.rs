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

/// Where a page or the chrome links a reader who wants the published Rust crate. The crate name is
/// spelled once ([`RUST_CRATE`]); this URL is the only place it is repeated as a literal, because
/// `const` cannot concatenate — the alias gate asserts `RUST_CRATE` against the manifest, and this
/// link is asserted against the crate path by `check-package-alias.mjs`'s readers.
pub const CRATES_IO_URL: &str = "https://crates.io/crates/base-ui-leptos";

/// The honest one-line status of the JS alias, for a link's `title`/`aria-label`. The alias is
/// mapped locally (`packages/leptos/`, linked by `pnpm install`) and deliberately NOT published, so a
/// page must never send a reader to an npm URL that would 404 (CONTRACT.md requirement 6).
pub const ALIAS_STATUS: &str = "@noevaresearch/base-ui — the JavaScript package alias, mapped locally in this repo and not published; the Rust crate base-ui-leptos is the installable artifact.";

