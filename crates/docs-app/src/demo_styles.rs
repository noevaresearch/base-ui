//! Demo-styling drift guard (host tests — no browser, no wasm).
//!
//! `crates/docs-app/style/main.css` makes the ported pages' Tailwind-variant class strings live:
//! the pages carry upstream's `tailwind` demo classes verbatim (`DEMO_BUTTON_CLASS` is
//! `button/demos/hero/tailwind/index.tsx:6` character for character), this crate compiles no
//! Tailwind, and its own hand-written stylesheet is what actually styles the demos — the utility
//! rules are generated from upstream's compiled stylesheet by
//! `ralph/scripts/gen-demo-utilities.mjs` and spliced into the marked section at the foot of
//! `main.css`.
//!
//! That design has exactly one failure mode, and it is silent: a demo class that no rule matches is
//! INERT — the element renders with browser defaults and every structural check still passes. It is
//! what produced the measurements this item exists to fix (the checkbox demo's control laying out
//! 784x57 instead of upstream's 166x36, the button rendering bare 53x24 text, widget parity 84.42%
//! on button while checkbox and meter were not comparable at all). A browser-side check cannot be
//! the guard here: `run-regression.sh` runs host `cargo test`, and a demo that silently loses its
//! styling is exactly the kind of regression a screenshot review misses.
//!
//! So these tests read the two artifacts (the pages' class constants and the stylesheet) and hold
//! the invariant: every class the demos name is defined, the generated section is present and LAST
//! (the cascade the section header promises), the demo wrapper centres its control, and every
//! Tailwind custom property the section reads is declared.
//!
//! The token check found real defects twice while this item was being implemented — a scan cap that
//! silently skipped the 712-character `DEMO_BUTTON_CLASS` (14 tokens undefined: `disabled:*`,
//! `data-disabled:*`) and `font-inherit`, the one class upstream's compiled stylesheet does not
//! define at all (its css-modules oracle carries the declaration instead). Both are fixed in
//! `main.css`; this module is what keeps them fixed.

#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn main_css() -> String {
    fs::read_to_string(crate_dir().join("style/main.css")).expect("style/main.css is readable")
}

fn page_sources() -> Vec<(String, String)> {
    let dir = crate_dir().join("src/pages");
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).expect("src/pages is readable") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        out.push((
            name,
            fs::read_to_string(&path).expect("page source is readable"),
        ));
    }
    out.sort();
    out
}

/// Every `const <IDENT>: &str = "…"` whose identifier mentions `CLASS` — the demo class constants,
/// which hold upstream's tailwind-variant class strings verbatim. Hand-rolled (one scan, both the
/// same-line and the next-line spelling) because the crate has no regex dependency and the shape
/// being matched is small and stable.
fn class_constants(src: &str) -> Vec<(String, String)> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(at) = src[from..].find(": &str") {
        let at = from + at;
        // the declaration's identifier: walk back to the `const` keyword
        let stmt = src[..at].rfind("const ").map(|i| i + "const ".len());
        let Some(ident_start) = stmt else {
            from = at + 6;
            continue;
        };
        let ident_end = src[ident_start..]
            .find(':')
            .map(|i| ident_start + i)
            .unwrap_or(at);
        let ident = src[ident_start..ident_end].trim();
        // the value: the next `"` after `: &str`, then the `"` that closes it
        let Some(open_rel) = src[at..].find('"') else {
            break;
        };
        let open = at + open_rel + 1;
        let Some(close_rel) = src[open..].find('"') else {
            break;
        };
        let value = &src[open..open + close_rel];
        if ident.contains("CLASS")
            && ident
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            out.push((ident.to_string(), value.to_string()));
        }
        from = open + close_rel + 1;
        let _ = bytes;
    }
    out
}

/// A class token as it appears in a class attribute: lowercase, no spaces, class-ish punctuation.
fn class_shaped(token: &str) -> bool {
    !token.is_empty()
        && token.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && token.bytes().all(|b| {
            b.is_ascii_lowercase()
                || b.is_ascii_digit()
                || matches!(
                    b,
                    b'-' | b'_'
                        | b'.'
                        | b':'
                        | b'/'
                        | b'['
                        | b']'
                        | b'%'
                        | b'#'
                        | b'('
                        | b')'
                        | b','
                )
        })
}

/// The selectors a stylesheet declares, unescaped (`\.dark\:bg-white` -> `dark:bg-white`). Read
/// off the raw text rather than a real parser: what the guard needs is "is there a rule for this
/// class name", and a class rule always spells the name in its selector.
fn declared_classes(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' {
            let mut name = String::new();
            let mut j = i + 1;
            while j < bytes.len() {
                match bytes[j] {
                    b'\\' if j + 1 < bytes.len() => {
                        name.push(bytes[j + 1] as char);
                        j += 2;
                    }
                    b if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' => {
                        name.push(b as char);
                        j += 1;
                    }
                    _ => break,
                }
            }
            if !name.is_empty() {
                out.push(name);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

fn generated_section(css: &str) -> &str {
    let begin = css
        .find("DEMO UTILITIES (generated) — BEGIN")
        .expect("the generated section is present");
    &css[begin..]
}

/// THE root guard: a demo class name that no rule matches renders inert while every structural
/// check stays green. `main.css` must define every class the pages' `*_CLASS` constants name.
#[test]
fn every_demo_class_token_is_defined_in_the_stylesheet() {
    let css = main_css();
    let declared: std::collections::HashSet<String> = declared_classes(&css).into_iter().collect();
    let mut undefined: Vec<String> = Vec::new();
    let mut consts = 0usize;
    let mut tokens = 0usize;
    for (file, src) in page_sources() {
        for (ident, value) in class_constants(&src) {
            consts += 1;
            for token in value.split_whitespace() {
                if !class_shaped(token) {
                    continue;
                }
                tokens += 1;
                if !declared.contains(token) {
                    undefined.push(format!("{file}:{ident}: {token}"));
                }
            }
        }
    }
    // A silent zero here would mean the extractor stopped matching the sources, not that the demos
    // have no classes: fail loudly instead of passing vacuously.
    assert!(
        consts > 40 && tokens > 300,
        "the class-constant scan found {consts} constant(s) / {tokens} token(s) — the extractor no longer \
         matches src/pages, so this guard is not checking anything"
    );
    assert!(
        undefined.is_empty(),
        "{} demo class token(s) have no rule in style/main.css, so they render INERT (regenerate with \
         `node ralph/scripts/gen-demo-utilities.mjs`, or port the css-modules rule by hand):\n  {}",
        undefined.len(),
        undefined.join("\n  ")
    );
}

/// The generated section must be present and LAST: it is emitted unlayered at the end of the file
/// precisely so a utility beats an earlier chrome rule of equal specificity (the precedence
/// upstream gets from `@layer utilities` winning over its components layer — the section header
/// states this). A later rule appended after it would silently outrank every demo utility.
#[test]
fn generated_section_is_last() {
    let css = main_css();
    let section = generated_section(&css);
    let end = section
        .find("DEMO UTILITIES (generated) — END")
        .expect("the section has its END marker");
    let after = section[end..]
        .split_once("*/")
        .expect("the END marker is a comment")
        .1;
    assert!(
        after.trim().is_empty(),
        "the generated demo-utilities section is no longer last in style/main.css — {} byte(s) of rules \
         follow it, and an unlayered rule after it would outrank every demo utility",
        after.trim().len()
    );
    assert!(
        section.contains("--tw-border-style"),
        "the generated section lost the @property registration/failure its --tw-* variables need"
    );
}

/// The demo wrapper's geometry, which is what makes the component widget measurable at all: upstream
/// renders a demo inside `.DemoPlaygroundInner` (a centering flex container), so the demo's control
/// is a flex ITEM and shrinks to its content. Without it the control lays out over the whole article
/// and `check-visual-budget.mjs` reports the widget region as NOT COMPARABLE (measured: checkbox
/// upstream 166x36 vs this side 784x36; meter 256x56 vs 784x42) instead of scoring it.
#[test]
fn demo_wrapper_centers_its_control() {
    let css = main_css();
    let at = css.find(".docs-demo {").expect(
        "`.docs-demo` has a rule (docs/src/components/Demo/Demo.css, .DemoPlaygroundInner)",
    );
    let body = &css[at..css[at..]
        .find('}')
        .map(|i| at + i)
        .expect("the rule is closed")];
    for decl in [
        "display: flex",
        "justify-content: center",
        "align-items: center",
        "min-width: fit-content",
    ] {
        assert!(
            body.contains(decl),
            "`.docs-demo` lost `{decl}` — the demos' controls stop being content-width and the widget \
             region becomes unmeasurable again (see the rule's comment)"
        );
    }
}

/// Every `var(--tw-…)` the generated section reads must be declared there (an `@property` block or
/// an assignment in a rule). Tailwind's `--tw-*` variables resolve to an *initial* value only when
/// registered: an unregistered one makes the declaration that reads it invalid at computed-value
/// time, which is how a copied utility rule can silently do nothing.
#[test]
fn tailwind_variables_are_declared() {
    let css = main_css();
    let section = generated_section(&css);
    let mut reads: Vec<String> = Vec::new();
    let mut rest = section;
    while let Some(at) = rest.find("var(--tw-") {
        let start = at + "var(".len();
        // the variable NAME ends at the first `,` (a fallback follows) or `)` (no fallback) —
        // stopping at `)` alone would capture the tail of a nested `var(--a, var(--b))`.
        let end = start
            + rest[start..]
                .find(|c: char| c == ',' || c == ')')
                .expect("the var() call is closed");
        reads.push(rest[start..end].trim().to_string());
        rest = &rest[end..];
    }
    assert!(
        !reads.is_empty(),
        "the generated section reads no --tw-* variable at all — did the section change shape?"
    );
    let mut undeclared: Vec<String> = reads
        .into_iter()
        .filter(|name| {
            !section.contains(&format!("{name}:"))
                && !section.contains(&format!("@property {name}"))
        })
        .collect();
    undeclared.sort();
    undeclared.dedup();
    assert!(
        undeclared.is_empty(),
        "the generated section reads Tailwind variable(s) it never declares: {} — the declaration that \
         reads one is invalid at computed-value time, so the rule does nothing",
        undeclared.join(", ")
    );
}

/// `main.css` states its own provenance (each section names the upstream file it ports) and its
/// generated section states how to regenerate it. If the generator is renamed the header stops
/// being actionable, and the next iteration has to reverse-engineer the file instead of running it.
#[test]
fn the_generated_header_names_the_generator() {
    let css = main_css();
    let section = generated_section(&css);
    assert!(
        section.contains("ralph/scripts/gen-demo-utilities.mjs"),
        "the generated section's header no longer names the script that reproduces it"
    );
    assert!(
        Path::new(&crate_dir().join("../../ralph/scripts/gen-demo-utilities.mjs")).exists(),
        "ralph/scripts/gen-demo-utilities.mjs is missing — the generated section cannot be reproduced"
    );
}
