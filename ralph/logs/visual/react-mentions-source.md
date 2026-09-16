# React mentions — source scan

Generated 2026-09-16T19:00:19.355Z by check-react-mentions.mjs --source.

Scope: the port's own reader-facing source — `crates/docs-app/src/**/*.rs`, test files excluded (`*_test.rs` and inline `#[cfg(test)]` items, which keep upstream snippets as positive controls), and the shared classifier module `snippet_language.rs` excluded because listing upstream's markers is its job. Mirror analyses (`specs/docs-content/*/page.md`, `specs/library/**`) are deliberately NOT scanned: they document upstream React by design.

The package this port points readers at must be `base-ui-leptos`; React APIs in prose/snippets are defects.

Classes: `react-api` (a React API where this port uses Leptos — the type-column class), `package-react` (an install reference or prose pointing at upstream's package/site), `snippet-react` (a React package inside a mirrored EXAMPLE block — snippet LANGUAGE, owned by the `docs-chrome: snippet translation` items and measured per route by visual-gap-report / check-page / snippetLanguage purity).

This run gates: react-api, package-react, snippet-react.
