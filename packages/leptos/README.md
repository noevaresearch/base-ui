# @noevaresearch/base-ui (locally mapped)

This directory maps the port's public name — `@noevaresearch/base-ui` — so the mirrored docs tell a reader
to install *this* project rather than upstream's React package.

**Status: NOT PUBLISHED.** `private: true` makes an accidental `npm publish` fail. Publishing is a
separate, reviewed step: publish the Rust crate first, then flip `private` and set the version from the
release tag.

* Rust component library: `crates/leptos-ui`
* Docs site (wasm): `crates/docs-app`
* The gate that enforces the name in the docs: `ralph/scripts/check-react-mentions.mjs`
