# React mentions — source scan

Generated 2026-09-16T17:30:33.332Z by check-react-mentions.mjs --source.

Scope: the port's own reader-facing source — `crates/docs-app/src/**/*.rs`, test files excluded (`*_test.rs` and inline `#[cfg(test)]` items, which keep upstream snippets as positive controls), and the shared classifier module `snippet_language.rs` excluded because listing upstream's markers is its job. Mirror analyses (`specs/docs-content/*/page.md`, `specs/library/**`) are deliberately NOT scanned: they document upstream React by design.

The package this port points readers at must be `base-ui-leptos`; React APIs in prose/snippets are defects.

Classes: `react-api` (a React API where this port uses Leptos — the type-column class), `package-react` (an install reference or prose pointing at upstream's package/site), `snippet-react` (a React package inside a mirrored EXAMPLE block — snippet LANGUAGE, owned by the `docs-chrome: snippet translation` items and measured per route by visual-gap-report / check-page / snippetLanguage purity).

This run gates: react-api, package-react; re-homed (printed, not gated here): snippet-react.

## crates/docs-app/src/install_ref.rs

- **react-word** L30: pub const PROVENANCE: &str = "Ported from the React implementation of Base UI — the same behaviour and anatomy, expressed with Leptos signals and view! markup.";

## crates/docs-app/src/pages/accordion_page.rs

- **react-word** L173: "Base UI is a library of high-quality unstyled React components for design systems and web apps.",
- **react-word** L196: "Base UI is a library of high-quality unstyled React components for design systems and web apps.",
- **react-word** L308: "Base UI is a library of high-quality unstyled React components for design systems and web apps.",
- **react-word** L376: "Base UI is a library of high-quality unstyled React components for design systems and web apps.",

## crates/docs-app/src/pages/form_page.rs

- **react-word** L802: "Upstream's React docs submit this demo with a server function, through React DOM's "
- **react-word** L803: "`useActionState`, instead of `onSubmit`. Server functions are a React DOM feature with "

## crates/docs-app/src/pages/merge_props_page.rs

- **react-word** L261: <p class="subtitle">"A utility to merge multiple sets of React props."</p>
- **react-word** L267: "common React patterns work as expected."
- **react-word** L294: "For React synthetic events, Base UI adds "

## crates/docs-app/src/pages/otp_field_page.rs

- **react-word** L963: "filled, or use `onValueComplete` to react to completion without submitting."

## crates/docs-app/src/pages/status_page.rs

- **react-word** L108: <th>"snippets leptos/react"</th>

## crates/docs-app/src/pages/use_render_page.rs

- **react-word** L351: "The `mergeProps` function merges two or more sets of React props together, "

## crates/docs-app/src/status_data.rs

- **react-word** L34: pub const EXPLANATION: &str = "Items-done counts ledger entries marked done. Pages-passing counts mirrored routes where EVERY scorecard axis passes (structure, page parity >=90, widget >=97, snippet l
- **react-word** L262: component: "floating-ui-react",
