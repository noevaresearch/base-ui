//! The snippet-language classifier, shared by the mirrored pages' browser-free guards.
//!
//! Why this exists: `specs/docs-content/CONTRACT.md` requirement 1 says every snippet embedded in a
//! mirrored page must demonstrate the PORT's API, and the only thing that measured it was the
//! browser-side probe in `ralph/scripts/visual-gap-report.mjs` (`looksReact`/`looksLeptos`,
//! `:233-242`) — which needs BOTH dev servers up and passes with a NOTE when upstream is down. A
//! page could therefore sit in the repo teaching upstream's React source through every green gate
//! (it did: checkbox's five blocks, then button's two).
//!
//! So each mirrored page's own test module asserts the numbers the probe would report, and this
//! module is the single copy of the rules they share — deliberately a faithful string-operation
//! mirror of the probe rather than a second classifier with its own opinion. If the probe's rules
//! change, change them here too; the pages' guards then move together instead of drifting apart.
//!
//! Callers: `crate::pages::button_page::snippet_language_guard` and
//! `crate::pages::checkbox_page::snippet_language_guard`.

/// `looksReact` from the probe (`ralph/scripts/visual-gap-report.mjs:233-238`), mirrored:
/// a Base UI/MUI import, any `import … from '…'`, a React hook name, a JSX-shaped prop or arrow
/// block, or a JSX-style tag.
pub fn looks_react(text: &str) -> bool {
    let has = |needle: &str| text.contains(needle);
    // `/import\s+[\s\S]{0,120}?\sfrom\s+['"]/`
    let import_from = text.match_indices("import").any(|(i, _)| {
        let window = &text[i..text.len().min(i + 140)];
        window.contains("from '") || window.contains("from \"")
    });
    // `/<\/?[A-Z][A-Za-z]*(\.[A-Z][A-Za-z]*)?[\s/>]/` — a JSX-style tag.
    let jsx_tag = {
        let b = text.as_bytes();
        (0..b.len()).any(|i| {
            if b[i] != b'<' {
                return false;
            }
            let mut j = i + 1;
            if b.get(j) == Some(&b'/') {
                j += 1;
            }
            if !b.get(j).is_some_and(u8::is_ascii_uppercase) {
                return false;
            }
            while b.get(j).is_some_and(u8::is_ascii_alphabetic) {
                j += 1;
            }
            if b.get(j) == Some(&b'.') && b.get(j + 1).is_some_and(u8::is_ascii_uppercase) {
                j += 2;
                while b.get(j).is_some_and(u8::is_ascii_alphabetic) {
                    j += 1;
                }
            }
            b.get(j)
                .is_some_and(|c| c.is_ascii_whitespace() || *c == b'/' || *c == b'>')
        })
    };
    // `/=>\s*\(|=>\s*\{/`
    let arrow_block = {
        let b = text.as_bytes();
        (0..b.len().saturating_sub(2)).any(|i| {
            if !(b[i] == b'=' && b[i + 1] == b'>') {
                return false;
            }
            let mut j = i + 2;
            while b.get(j).is_some_and(|c| c.is_ascii_whitespace()) {
                j += 1;
            }
            matches!(b.get(j), Some(b'(') | Some(b'{'))
        })
    };
    has("@base-ui/react")
        || has("@mui/")
        || import_from
        || has("useState")
        || has("useRef")
        || has("useEffect")
        || has("useCallback")
        || has("className=")
        || has("onClick={")
        || has("{props")
        || arrow_block
        || jsx_tag
}

/// `looksLeptos` from the probe (`ralph/scripts/visual-gap-report.mjs:239-242`), mirrored: an
/// idiomatic Leptos import, macro, signature or prop/composition marker.
pub fn looks_leptos(text: &str) -> bool {
    let has = |needle: &str| text.contains(needle);
    has("use leptos")
        || has("leptos_ui")
        || has("leptos-ui")
        || has("view!")
        || has("#[component]")
        || has("-> impl IntoView")
        || has("cx(")
        || has("Signal<")
        || has("RwSignal")
        || has("ReadSignal")
        || has("Memo<")
        || has("on:click")
        || has("prop:")
        || has("attr:")
}
