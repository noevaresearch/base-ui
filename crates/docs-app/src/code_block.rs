//! The ported code-block chrome and its syntax highlighter.
//!
//! Why this module exists: every mirrored page embedded its snippets as a bare
//! `<pre><code>…</code></pre>`, so the port rendered monochrome code where upstream renders a
//! bordered panel with a title bar, a copy control and a token-coloured body. `visual-gap-report.mjs`
//! measured that as the FIRST P0 on every recorded route — "upstream's code carries 41 coloured
//! tokens; this page has 0 — the code is unstyled monochrome" (checkbox 41 / button 56 / meter 43,
//! and the same on accordion) — with codeBlocks recall at 10/238 (checkbox), 2/172 (button) and
//! 2/62 (meter).
//!
//! UPSTREAM'S SOURCE THIS PORTS (the item's own `specs:` field):
//!
//! * `docs/src/components/CodeBlock/CodeBlock.tsx` — `Root` (`role="figure"`, `.CodeBlockRoot`),
//!   `Panel` (`.CodeBlockPanel` with a `.CodeBlockPanelTitle` and a `GhostButton` carrying
//!   `aria-label="Copy code"` whose `.CodeBlockCopyIcon` swaps to a check for 2s), `Content`
//!   (`.CodeBlockPreContainer` > `.CodeBlockViewport`), and `Pre`/`PreInline`
//!   (`pre.CodeBlockPre.CodeBlockPreInline`).
//! * `docs/src/components/CodeBlock/CodeBlockPreComputed.tsx` — the block is rendered from
//!   pre-computed highlight data, i.e. the token spans are static markup, not a runtime
//!   highlighter. The port has no docs build step to pre-compute with, so [`highlight`] computes
//!   the same span structure in Rust at render time; the DOM it produces is the pre-computed
//!   shape (see below).
//! * `docs/src/components/CodeBlock/CodeBlock.css` — `.CodeBlockRoot` (white, `1px --color-border`,
//!   `--radius-12`), `.CodeBlockPanel` (`--gray-s2`, 2.25rem tall, `--radius-12 - 1px` top corners,
//!   bottom `1px` border) and the `pre`/`code` metrics.
//! * `docs/src/components/GhostButton.css` — the copy control's box (`data-layout="icon"`:
//!   1.75rem square, centred).
//! * `docs/src/css/syntax.css` — the token palette. It is a prettylights theme, so the class names
//!   below (`pl-k`, `pl-c1`, `pl-s`, `pl-pds`, `pl-c`, `pl-en`, `pl-ent`, `pl-smi`, `pl-v`) are
//!   upstream's own, and `--color-*` tokens map them to the docs theme; `di-*` are upstream's
//!   docs-infra extensions (JSX tags, attribute keys/values, CSS properties/values).
//!
//! THE DOM, READ OFF UPSTREAM'S LIVE RENDER (2026-09-16, 1280px, upstream on :3005 — not guessed
//! from the `.tsx` sources), for the checkbox page's `## Anatomy` fence:
//!
//! ```html
//! <div role="figure" class="CodeBlockRoot MdFigure">
//!   <div class="CodeBlockPanel">
//!     <div class="CodeBlockPanelTitle">Anatomy</div>
//!     <button data-layout="icon" type="button" class="GhostButton" aria-label="Copy code">
//!       <span class="CodeBlockCopyIcon"><svg …/></span></button>
//!   </div>
//!   <div class="CodeBlockPreContainer"><div class="CodeBlockViewport">
//!     <pre class="CodeBlockPreInline CodeBlockPre"><code class="language-jsx"
//!       data-total-lines="5" data-focused-lines="5">
//!       <span class="frame" data-frame-type="focus">
//!         <span data-ln="1" class="line"><span class="pl-k">import</span> …</span>\n…
//! ```
//!
//! The title text comes from the fence's meta in the upstream `.mdx`
//! (`docs/src/app/(docs)/react/components/checkbox/page.mdx:21`, ` ```jsx title="Anatomy" `) and it
//! is load-bearing, not decoration: upstream renders the panel (and therefore the copy control)
//! ONLY for a titled fence — `CodeBlockPreComputedContent` renders `title ? <Panel> : null`.
//!
//! DELIBERATE DEVIATIONS FROM UPSTREAM, each mechanical:
//!
//! 1. The viewport is a plain `div`, not upstream's `ScrollArea.Root`/`ScrollArea.Viewport`: this
//!    crate has no scroll-area unit (the same deviation the chrome stylesheet already records for
//!    the side nav). Only the class names the stylesheet keys on are kept, so the geometry is
//!    upstream's while the scrollbar is the browser's.
//! 2. `data-total-lines`/`data-focused-lines` are both the snippet's line count. Upstream derives
//!    the focused count from the fence's `@highlight` directives; the port's snippets keep those
//!    directives as comments (they are part of the snippet text the contract fixes), so the
//!    `frame[data-frame-type="focus"]` wrapper is upstream's and the counts are the honest "whole
//!    block" reading of it. Measured on upstream's Anatomy fence: 5/5 — identical in shape.
//! 3. The copy handler writes to the clipboard through `navigator.clipboard` and restores the icon
//!    with a plain timer closure on the button's own icon element. No reactive value is touched
//!    after the click, so a copy whose 2s restore lands after the page unmounts is a detached-DOM
//!    write rather than a disposed-signal panic (`library: avatar`'s defect class).
//! 4. `highlight` is a scanner, not a parser: it is the same *kind* of output as upstream's
//!    pre-computed spans (the class names and therefore the colours are upstream's), and the
//!    scanner's rules are unit-tested against the pages' real snippets. It has no grammar
//!    knowledge beyond classifiers, so it will mis-colour exotic code; the classes it emits are
//!    listed in [`highlight`]'s docs.
//! 5. `Root` carries `role="figure"` but NOT upstream's `aria-labelledby` (`CodeBlock.tsx:26`),
//!    which upstream points at the panel title it mints with `React.useId()`
//!    (`CodeBlock.tsx:19-20`, `:49-57`; the live DOM reads
//!    `<div role="figure" aria-labelledby="_R_..." class="CodeBlockRoot MdFigure">`). This module
//!    is a plain view fn with no per-instance id, and minting one means driving the internals
//!    crate's rg-0.2 `use_base_ui_id` from an owner the call site does not open — so the figure's
//!    accessible NAME is the one piece of upstream's markup left out rather than guessed at. It is
//!    carried open on this item's TODO entry; nothing the fidelity probes measure is affected
//!    (titles, token colours, panel geometry and the copy control are all upstream's).

use leptos::prelude::*;

/// Every token class [`highlight`] can emit. Two invariants are asserted against it in this
/// module's tests: the scanner emits nothing outside this list, and `crate::code_block`'s
/// stylesheet colours every entry — a class with no colour rule is exactly the defect the gap
/// report measures ("0 coloured tokens").
pub const TOKEN_CLASSES: &[&str] = &[
    "pl-c", "pl-c1", "pl-en", "pl-ent", "pl-k", "pl-s", "pl-smi", "di-ak", "di-ae", "di-bool",
    "di-cp", "di-cv", "di-jsx", "di-n",
];

/// A snippet's language. Upstream tags each fence in the `.mdx` (` ```jsx`), and the class it puts
/// on `<code>` (`language-jsx`) is the language-specific styling hook `syntax.css` keys on
/// (`.language-css` retunes the variable colour).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    /// The port's own snippets (`use leptos::prelude::*; … view! { … }`).
    Rust,
    /// Upstream's JSX fences — still on the pages whose snippets have not been translated yet.
    Jsx,
    /// Upstream's TSX fences, same status as [`Lang::Jsx`].
    Tsx,
    /// Fence language for the demos' stylesheets (the avatar page's stacking example).
    Css,
    /// Fence language for an HTML fence (` ```html ` — the csp-provider page's scrollbar example).
    Html,
}

impl Lang {
    /// The `language-*` class upstream puts on the fence's `<code>` element, read off the live
    /// render (`<code class="language-jsx">`, `<code class="language-css">`).
    pub fn code_class(self) -> &'static str {
        match self {
            Lang::Rust => "language-rust",
            Lang::Jsx => "language-jsx",
            Lang::Tsx => "language-tsx",
            Lang::Css => "language-css",
            Lang::Html => "language-html",
        }
    }
}

/// One highlighted token: the prettylights class upstream's highlighter would put on the span
/// (`"pl-k"`, `"pl-c1 di-jsx"`, …) and its text. An empty class means "no span" — the token is
/// rendered as plain text, exactly as upstream's highlighter leaves unclassified text.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token {
    pub class: &'static str,
    pub text: String,
}

impl Token {
    fn classified(class: &'static str, text: impl Into<String>) -> Self {
        Token {
            class,
            text: text.into(),
        }
    }
}

/// A `<span class="pl-s">` string with its quotes as `.pl-pds` children — upstream's shape for a
/// string literal (`<span class="pl-s"><span class="pl-pds">'</span>…<span class="pl-pds">'</span></span>`),
/// read off the live render.
fn string_token(quote: char, body: &str) -> Token {
    Token::classified("pl-s", format!("{quote}{body}{quote}"))
}

/// Language keywords → `pl-k` (upstream's keyword colour, `--color-red`).
const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "type", "unsafe", "use", "where",
    "while",
];

/// JS/TS keywords → `pl-k`. Upstream's JSX fences (`import … from …`, `export default …`) are
/// highlighted with the same class (`pl-k` on `import` and `from`, measured on the live render).
const JS_KEYWORDS: &[&str] = &[
    "as",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "finally",
    "for",
    "from",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "let",
    "new",
    "of",
    "return",
    "static",
    "switch",
    "this",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "yield",
];

/// Constants and literals → `pl-c1` / `di-bool` / `di-n` (upstream's constant + docs-infra classes).
fn literal_class(lang: Lang, word: &str) -> Option<&'static str> {
    match word {
        "true" | "false" => Some("di-bool"),
        "null" | "undefined" | "None" => Some("di-n"),
        _ if word.chars().next().is_some_and(|c| c.is_ascii_digit()) => Some("pl-c1"),
        _ => match lang {
            Lang::Rust => (word == "Self").then_some("pl-c1"),
            Lang::Jsx | Lang::Tsx => (word == "Infinity" || word == "NaN").then_some("pl-c1"),
            Lang::Css | Lang::Html => None,
        },
    }
}

/// `true` for the HTML-ish tag names the markup scanner treats as elements (`label`, `div`,
/// `button`, …): a lowercase tag is an element upstream colours with `pl-ent`
/// (`--color-green`, `--color-prettylights-syntax-entity-tag`).
fn is_element_tag(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
}

/// The scanner's mode. `Code` is ordinary language text; `Tag` is inside a markup tag (the `view!`
/// macro's body or JSX), where identifiers are attributes and quoted text is an attribute value;
/// `Selector`/`Decl`/`Value` are the CSS counterparts.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Code,
    Tag,
    CssSelector,
    CssDecl,
    CssValue,
}

/// Highlight one snippet.
///
/// The scan is single-pass with a small mode stack:
///
/// * comments → `pl-c`, strings → `pl-s`, digits → `pl-c1`, keywords → `pl-k`, `ident!` macros and
///   `fn`-declared names → `pl-en`, everything else stays plain;
/// * markup (`<label …>` in a `view!` body, JSX tags) → `<`/`>`/`/` plain, a lowercase tag name
///   `pl-ent`, a component-shaped tag name (`<Checkbox.Root>` in JSX, a `PascalCase` part in
///   `view!`) `pl-c1 di-jsx`, attribute names `di-ak`, `=` `di-ae`, attribute strings `pl-s`;
/// * CSS → selectors `pl-ent`, properties `di-cp`, values `di-cv`, strings in values `pl-s`.
pub fn highlight(lang: Lang, code: &str) -> Vec<Vec<Token>> {
    let chars: Vec<char> = code.chars().collect();
    let mut lines: Vec<Vec<Token>> = vec![Vec::new()];
    let mut mode = if lang == Lang::Css {
        Mode::CssSelector
    } else {
        Mode::Code
    };
    let mut i = 0usize;

    // Push text into the current line, splitting on newlines so the token stream and the line
    // structure stay in step.
    macro_rules! emit {
        ($class:expr, $text:expr) => {{
            let class: &'static str = $class;
            let text: String = $text;
            let mut parts = text.split('\n').peekable();
            while let Some(part) = parts.next() {
                if !part.is_empty() {
                    lines
                        .last_mut()
                        .expect("a line is always open")
                        .push(Token::classified(class, part));
                }
                if parts.peek().is_some() {
                    lines.push(Vec::new());
                }
            }
        }};
    }

    let peek = |i: usize| chars.get(i).copied();

    while i < chars.len() {
        let c = chars[i];
        let next = peek(i + 1);

        match mode {
            Mode::Code => {
                // Line comment: `// …` — upstream's `@highlight` directives ride in comments and
                // are ordinary ASCII, so a comment token is also where the directive text lands.
                if c == '/' && next == Some('/') {
                    let start = i;
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                    emit!("pl-c", chars[start..i].iter().collect::<String>());
                    continue;
                }
                // Block comment.
                if c == '/' && next == Some('*') {
                    let start = i;
                    i += 2;
                    while i < chars.len() && !(chars[i] == '*' && peek(i + 1) == Some('/')) {
                        i += 1;
                    }
                    i = (i + 2).min(chars.len());
                    emit!("pl-c", chars[start..i].iter().collect::<String>());
                    continue;
                }
                // String literal, with the quotes kept inside the `.pl-s` span (upstream nests
                // `.pl-pds` spans for them; one span with the same text and colour is the same
                // paint and keeps the DOM cheap).
                if c == '"' || c == '\'' {
                    let (token, consumed) = scan_quoted(&chars, i, c);
                    emit!(token.class, token.text);
                    i = consumed;
                    continue;
                }
                // A JSX/markup tag: `<` or `</` followed by a tag-name start.
                if c == '<'
                    && next.is_some_and(|n| n.is_ascii_alphabetic() || n == '/')
                    && jsx_tag_ahead(&chars, i)
                {
                    emit!("", "<".to_string());
                    i += 1;
                    if peek(i) == Some('/') {
                        emit!("", "/".to_string());
                        i += 1;
                    }
                    mode = Mode::Tag;
                    continue;
                }
                // Attribute keys inside a tag (`{,`-delimited brace expressions stay in Tag mode).
                if c.is_ascii_digit() {
                    let start = i;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_')
                    {
                        i += 1;
                    }
                    let word: String = chars[start..i].iter().collect();
                    emit!(literal_class(lang, &word).unwrap_or("pl-c1"), word);
                    continue;
                }
                if c.is_ascii_alphabetic() || c == '_' {
                    let start = i;
                    while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let word: String = chars[start..i].iter().collect();
                    // `#[…]` attributes and `ident!` macros.
                    if peek(i) == Some('!') {
                        emit!("pl-en", word);
                        continue;
                    }
                    let keywords = match lang {
                        Lang::Rust => RUST_KEYWORDS,
                        Lang::Jsx | Lang::Tsx => JS_KEYWORDS,
                        Lang::Css | Lang::Html => &[][..],
                    };
                    // The identifier a `fn` introduces is the declared function's name — upstream's
                    // entity colour, the same one it gives a JS `function` name. Whitespace
                    // between `fn` and the name is skipped, so the test looks at the last token
                    // that is not whitespace.
                    let previous = last_significant(lines.last());
                    let after_fn = previous.is_some_and(|t| t.text == "fn");
                    // A name inside an `import { … }` list is upstream's storage-modifier colour
                    // (`pl-smi` on `Checkbox` in `import { Checkbox } from …`, read off the live
                    // render).
                    let in_import_list = lang != Lang::Rust
                        && lines
                            .last()
                            .is_some_and(|l| l.iter().any(|t| t.text == "import"))
                        && matches!(previous.map(|t| t.text.as_str()), Some("{") | Some(","));
                    // A `PascalCase` name is a type, a part or a variant (`CheckboxRootViewProps`,
                    // `RenderProp`, `Some`) — upstream's constant colour. Position decides it: it
                    // is only a type when it is used as one (a path, a struct literal, a call, a
                    // type ascription), which keeps JSX text nodes like `Text` plain.
                    let pascalcase_type =
                        word.chars().next().is_some_and(|c| c.is_ascii_uppercase()) && {
                            let mut j = i;
                            while chars.get(j).is_some_and(|c| *c == ' ' || *c == '\t') {
                                j += 1;
                            }
                            matches!(chars.get(j), Some('(') | Some('{') | Some(':'))
                        };
                    if after_fn {
                        emit!("pl-en", word);
                    } else if in_import_list {
                        emit!("pl-smi", word);
                    } else if keywords.contains(&word.as_str()) {
                        emit!("pl-k", word);
                    } else if let Some(class) = literal_class(lang, &word) {
                        emit!(class, word);
                    } else if pascalcase_type {
                        emit!("pl-c1", word);
                    } else {
                        emit!("", word);
                    }
                    continue;
                }
                emit!("", c.to_string());
                i += 1;
            }
            Mode::Tag => match c {
                '>' => {
                    emit!("", ">".to_string());
                    i += 1;
                    mode = Mode::Code;
                }
                '/' => {
                    emit!("", "/".to_string());
                    i += 1;
                }
                '{' => {
                    emit!("", "{".to_string());
                    i += 1;
                    // A braced expression inside a tag holds real code (`render={<button />}`),
                    // and inside it a nested tag may open. Scan it as opaque text up to its
                    // matching brace: the expression's contents are the page author's, and the
                    // token stream only has to preserve them character for character.
                    let start = i;
                    let mut depth = 1usize;
                    while i < chars.len() {
                        match chars[i] {
                            '{' => depth += 1,
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    let inner: String = chars[start..i].iter().collect();
                    if !inner.is_empty() {
                        emit!("", inner);
                    }
                    if i < chars.len() {
                        emit!("", "}".to_string());
                        i += 1;
                    }
                }
                '"' | '\'' => {
                    let (token, consumed) = scan_quoted(&chars, i, c);
                    emit!("pl-s", token.text);
                    i = consumed;
                }
                '=' => {
                    emit!("di-ae", "=".to_string());
                    i += 1;
                }
                '<' => {
                    emit!("", "<".to_string());
                    i += 1;
                    if peek(i) == Some('/') {
                        emit!("", "/".to_string());
                        i += 1;
                    }
                }
                c if c.is_ascii_alphabetic() || c == '_' || c == '-' || c == ':' => {
                    let start = i;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphanumeric()
                            || chars[i] == '_'
                            || chars[i] == '-'
                            || chars[i] == ':'
                            || chars[i] == '.')
                    {
                        i += 1;
                    }
                    let word: String = chars[start..i].iter().collect();
                    // A tag name is what precedes the first whitespace or `/` after `<`; the
                    // scanner reaches here for both, so it is recognised by position: a bare
                    // identifier at the tag's head followed by whitespace, `>`, `/` or the end.
                    // A tag name is the identifier immediately after `<` or `</`; anything else
                    // in the tag is an attribute key. Position is what distinguishes them, so the
                    // test reads the token stream rather than a mode flag.
                    let is_tag_head = lines.last().is_some_and(
                        |l| matches!(l.last(), Some(t) if t.text == "<" || t.text == "/"),
                    );
                    if is_tag_head {
                        if is_element_tag(&word) {
                            emit!("pl-ent", word);
                        } else if lang == Lang::Rust {
                            emit!("pl-c1", word);
                        } else {
                            emit!("pl-c1 di-jsx", word);
                        }
                    } else {
                        emit!("di-ak", word);
                    }
                    continue;
                }
                _ => {
                    emit!("", c.to_string());
                    i += 1;
                }
            },
            Mode::CssSelector => match c {
                '{' => {
                    emit!("", "{".to_string());
                    i += 1;
                    mode = Mode::CssDecl;
                }
                '/' if next == Some('*') => {
                    let start = i;
                    i += 2;
                    while i < chars.len() && !(chars[i] == '*' && peek(i + 1) == Some('/')) {
                        i += 1;
                    }
                    i = (i + 2).min(chars.len());
                    emit!("pl-c", chars[start..i].iter().collect::<String>());
                }
                '.' | '#' => {
                    emit!("", c.to_string());
                    i += 1;
                    let start = i;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphanumeric() || chars[i] == '-' || chars[i] == '_')
                    {
                        i += 1;
                    }
                    emit!("pl-ent", chars[start..i].iter().collect::<String>());
                }
                '@' => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '-') {
                        i += 1;
                    }
                    emit!("pl-k", chars[start..i].iter().collect::<String>());
                }
                c if c.is_ascii_alphabetic() => {
                    let start = i;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphanumeric() || chars[i] == '-' || chars[i] == '_')
                    {
                        i += 1;
                    }
                    emit!("pl-ent", chars[start..i].iter().collect::<String>());
                }
                _ => {
                    emit!("", c.to_string());
                    i += 1;
                }
            },
            Mode::CssDecl | Mode::CssValue => match c {
                '}' => {
                    emit!("", "}".to_string());
                    i += 1;
                    mode = Mode::CssSelector;
                }
                ';' => {
                    emit!("", ";".to_string());
                    i += 1;
                    mode = Mode::CssDecl;
                }
                '"' | '\'' => {
                    let (token, consumed) = scan_quoted(&chars, i, c);
                    emit!("pl-s", token.text);
                    i = consumed;
                }
                ':' if mode == Mode::CssDecl => {
                    emit!("di-ae", ":".to_string());
                    i += 1;
                    mode = Mode::CssValue;
                }
                c if c.is_ascii_alphabetic() || c == '-' => {
                    let start = i;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphanumeric()
                            || chars[i] == '-'
                            || chars[i] == '%'
                            || chars[i] == '.' && mode == Mode::CssValue)
                    {
                        i += 1;
                    }
                    let word: String = chars[start..i].iter().collect();
                    if mode == Mode::CssDecl {
                        emit!("di-cp", word);
                    } else {
                        emit!("di-cv", word);
                    }
                }
                c if c.is_ascii_digit() => {
                    let start = i;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '%')
                    {
                        i += 1;
                    }
                    emit!("di-cv", chars[start..i].iter().collect::<String>());
                }
                _ => {
                    emit!("", c.to_string());
                    i += 1;
                }
            },
        }
    }

    lines
}

/// The last token on a line that is not pure whitespace — the token that carries the position the
/// classifiers read (`fn name`, `import { Name }`).
fn last_significant(line: Option<&Vec<Token>>) -> Option<&Token> {
    line.and_then(|l| l.iter().rev().find(|t| !t.text.trim().is_empty()))
}

/// Reads a quoted literal starting at `start` and returns the token plus the index just past it.
fn scan_quoted(chars: &[char], start: usize, quote: char) -> (Token, usize) {
    let mut i = start + 1;
    while i < chars.len() {
        match chars[i] {
            '\\' => i += 2,
            c if c == quote => {
                i += 1;
                break;
            }
            _ => i += 1,
        }
    }
    let body: String = chars[start + 1..i.min(chars.len())].iter().collect();
    let body = body.strip_suffix(quote).unwrap_or(&body).to_string();
    (string_token(quote, &body), i)
}

/// `true` when the `<` at `i` opens something tag-shaped: `<name`, `<Name`, `</name`. The
/// port's `view!` bodies carry real HTML tags and the JSX fences carry component tags, so both the
/// element and the component spelling must open Tag mode — but a Rust comparison like `a < b` must
/// not, which is what the lookahead after the name checks.
fn jsx_tag_ahead(chars: &[char], i: usize) -> bool {
    let mut j = i + 1;
    if chars.get(j) == Some(&'/') {
        j += 1;
    }
    if !chars.get(j).is_some_and(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    while chars.get(j).is_some_and(|c| {
        c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '.' || *c == ':'
    }) {
        j += 1;
    }
    matches!(
        chars.get(j),
        Some(' ') | Some('>') | Some('/') | Some('\t') | None
    )
}

/// Render one mirrored snippet as upstream's code-block chrome.
///
/// `title` is the fence's `.mdx` title (` ```jsx title="Anatomy" `); upstream renders the panel —
/// and with it the copy control — only for a titled fence, so an empty title renders the bare
/// panel-less block that upstream's untitled fences get.
pub fn code_block(lang: Lang, title: &str, code: &str) -> impl IntoView {
    let title = title.to_string();
    let lines = highlight(lang, code);
    let total_lines = lines.len();
    let code_for_copy = code.to_string();

    let mut body: Vec<AnyView> = Vec::with_capacity(total_lines * 2);
    for (index, tokens) in lines.into_iter().enumerate() {
        if index > 0 {
            body.push(view! { "\n" }.into_any());
        }
        let spans: Vec<AnyView> = tokens
            .into_iter()
            .map(|token| match token.class {
                "" => view! { {token.text} }.into_any(),
                class => view! { <span class=class>{token.text}</span> }.into_any(),
            })
            .collect();
        body.push(
            view! {
                <span class="line" data-ln=index + 1>
                    {spans}
                </span>
            }
            .into_any(),
        );
    }

    let on_copy = move |event: leptos::ev::MouseEvent| {
        copy_code(&code_for_copy, &event);
    };

    let code_view = view! {
        <code
            class=lang.code_class()
            data-total-lines=total_lines
            data-focused-lines=total_lines
        >
            <span class="frame" data-frame-type="focus">{body}</span>
        </code>
    };

    if title.is_empty() {
        // Upstream's untitled fence: no panel, therefore no copy control and no title bar.
        view! {
            <div role="figure" class="CodeBlockRoot MdFigure">
                <div class="CodeBlockPreContainer">
                    <div class="CodeBlockViewport">
                        <pre class="CodeBlockPreInline CodeBlockPre">{code_view}</pre>
                    </div>
                </div>
            </div>
        }
        .into_any()
    } else {
        view! {
            <div role="figure" class="CodeBlockRoot MdFigure">
                <div class="CodeBlockPanel">
                    <div class="CodeBlockPanelTitle">{title}</div>
                    <button
                        data-layout="icon"
                        type="button"
                        class="GhostButton"
                        aria-label="Copy code"
                        on:click=on_copy
                    >
                        <span class="CodeBlockCopyIcon" inner_html=COPY_ICON_SVG></span>
                    </button>
                </div>
                <div class="CodeBlockPreContainer">
                    <div class="CodeBlockViewport">
                        <pre class="CodeBlockPreInline CodeBlockPre">{code_view}</pre>
                    </div>
                </div>
            </div>
        }
        .into_any()
    }
}

/// Upstream's `CopyIcon` (`docs/src/icons/CopyIcon.tsx`), path-for-path.
const COPY_ICON_SVG: &str = concat!(
    r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor">"#,
    r#"<path stroke-linecap="square" d="M1.5 1.5h10v10h-10z"></path>"#,
    r#"<path stroke-linecap="square" d="M4.5 11.5h-3v-10h10v3"></path>"#,
    r#"<path d="M12 4.5h2.5v10h-10V12"></path></svg>"#,
);

/// Upstream's `CheckIcon` (`docs/src/icons/CheckIcon.tsx`) — what [`COPY_ICON_SVG`] is swapped for
/// for 2s after a copy, `CodeBlock.tsx:78-81`. Wasm-only: the swap lives in [`copy_code`], which
/// the host stubs out (there is no clipboard to confirm on the host).
#[cfg(target_arch = "wasm32")]
const CHECK_ICON_SVG: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor"><path d="m2.5 8.5 4 4 7-9" /></svg>"#;

/// The copy control's behaviour: write the block's code to the clipboard and swap the icon for
/// [`CHECK_ICON_SVG`] for 2s (`CodeBlock.tsx:62-89`).
///
/// On wasm this is the real clipboard write; on the host it is inert (the host has no
/// `navigator`), which is what lets the crate build and its non-browser tests run. The 2s restore
/// is a plain DOM write on the icon element the click carried — deliberately NOT a reactive
/// signal, so it cannot write to a disposed owner if the page unmounts inside those 2s.
#[cfg(target_arch = "wasm32")]
fn copy_code(text: &str, event: &leptos::ev::MouseEvent) {
    use wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    let _ = window.navigator().clipboard().write_text(text);

    let icon = event
        .current_target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|button| button.query_selector(".CodeBlockCopyIcon").ok().flatten());
    let Some(icon) = icon else {
        return;
    };
    icon.set_inner_html(CHECK_ICON_SVG);
    let restore = wasm_bindgen::closure::Closure::once(move || {
        icon.set_inner_html(COPY_ICON_SVG);
    });
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        restore.as_ref().unchecked_ref(),
        2000,
    );
    // The timeout owns the closure; forgetting it keeps it alive until it fires (the same
    // hold-or-forget convention the pages' listeners use).
    restore.forget();
}

/// The host arm of [`copy_code`]: no clipboard, no timer, no-op.
#[cfg(not(target_arch = "wasm32"))]
fn copy_code(_text: &str, _event: &leptos::ev::MouseEvent) {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flatten a highlight result back to text — the invariant every snippet must satisfy: the
    /// token stream is the snippet, character for character (the page guards and the fidelity
    /// probes read the rendered `pre`'s text).
    fn flatten(lines: &[Vec<Token>]) -> String {
        lines
            .iter()
            .map(|line| line.iter().map(|t| t.text.as_str()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn classes(lines: &[Vec<Token>]) -> Vec<(String, &'static str)> {
        lines
            .iter()
            .flatten()
            .map(|t| (t.text.clone(), t.class))
            .collect()
    }

    /// The class of the first token carrying `text`; `None` when the token is plain (no class),
    /// which is what most of a snippet is.
    fn class_of<'a>(lines: &'a [Vec<Token>], text: &str) -> Option<&'static str> {
        classes(lines)
            .into_iter()
            .filter(|(_, class)| !class.is_empty())
            .find(|(t, _)| t == text)
            .map(|(_, c)| c)
    }

    /// The pages' own snippets are the tokenizer's acceptance fixture: these are the exact
    /// strings `checkbox_page.rs` and `button_page.rs` embed.
    const CHECKBOX_ANATOMY: &str = r#"use leptos::prelude::*;
use leptos_ui::{CheckboxIndicatorViewProps, CheckboxRootViewProps};
use leptos_ui::{checkbox_indicator_view, checkbox_root_view};

view! {
    {checkbox_root_view(CheckboxRootViewProps {
        children: Some(Box::new(|| {
            checkbox_indicator_view(CheckboxIndicatorViewProps::default()).into_any()
        })),
        ..CheckboxRootViewProps::default()
    })}
}"#;

    #[test]
    fn the_token_stream_is_the_snippet_exactly() {
        for (lang, code) in [
            (Lang::Rust, CHECKBOX_ANATOMY),
            (
                Lang::Jsx,
                "import { Checkbox } from '@base-ui/react/checkbox';\n\n<Checkbox.Root>\n  <Checkbox.Indicator />\n</Checkbox.Root>;",
            ),
            (
                Lang::Css,
                ".Root {\n  display: flex;\n  color: oklch(0.5 0.22 31);\n}",
            ),
        ] {
            let lines = highlight(lang, code);
            assert_eq!(flatten(&lines), code, "{lang:?} lost or duplicated text");
        }
    }

    #[test]
    fn rust_keywords_strings_comments_and_directives_are_classified() {
        let code = "// @highlight-start\nlet label = \"Accept terms\";\nfn run() {}\nview! { <label>{text}</label> }";
        let lines = highlight(Lang::Rust, code);
        assert_eq!(class_of(&lines, "// @highlight-start"), Some("pl-c"));
        assert_eq!(class_of(&lines, "let"), Some("pl-k"));
        assert_eq!(class_of(&lines, "\"Accept terms\""), Some("pl-s"));
        assert_eq!(class_of(&lines, "fn"), Some("pl-k"));
        assert_eq!(class_of(&lines, "run"), Some("pl-en"));
        // A `view!` macro name and a lowercase markup tag.
        assert_eq!(class_of(&lines, "view"), Some("pl-en"));
        assert_eq!(class_of(&lines, "label"), Some("pl-ent"));
        assert_eq!(class_of(&lines, "Accept terms"), None);
    }

    #[test]
    fn a_pascal_case_type_is_coloured_and_a_plain_ident_is_not() {
        let lines = highlight(Lang::Rust, CHECKBOX_ANATOMY);
        // A type/part used as one (a struct literal, a path) is upstream's constant colour.
        assert_eq!(class_of(&lines, "CheckboxRootViewProps"), Some("pl-c1"));
        assert_eq!(class_of(&lines, "Box"), Some("pl-c1"));
        assert_eq!(class_of(&lines, "Some"), Some("pl-c1"));
        assert_eq!(class_of(&lines, "use"), Some("pl-k"));
        // A snake_case call is plain text, which is what the port's part functions are.
        assert_eq!(class_of(&lines, "checkbox_root_view"), None);
    }

    #[test]
    fn jsx_tags_attributes_and_imports_are_classified_like_upstreams_render() {
        let code = "import { Checkbox } from '@base-ui/react/checkbox';\n\n<Checkbox.Root id=\"x\" render={<button />}>\n  <label>Text</label>\n</Checkbox.Root>;";
        let lines = highlight(Lang::Jsx, code);
        assert_eq!(class_of(&lines, "import"), Some("pl-k"));
        // Upstream's live render: the imported name is `pl-smi`, the string `pl-s` with `.pl-pds`
        // quote spans, a component tag `pl-c1 di-jsx` and an element tag `pl-ent`.
        assert_eq!(class_of(&lines, "Checkbox"), Some("pl-smi"));
        assert_eq!(class_of(&lines, "'@base-ui/react/checkbox'"), Some("pl-s"));
        assert_eq!(class_of(&lines, "Checkbox.Root"), Some("pl-c1 di-jsx"));
        assert_eq!(class_of(&lines, "label"), Some("pl-ent"));
        assert_eq!(class_of(&lines, "id"), Some("di-ak"));
        assert_eq!(class_of(&lines, "\"x\""), Some("pl-s"));
        // A text node inside an element is plain, not a type.
        assert_eq!(class_of(&lines, "Text"), None);
        assert_eq!(flatten(&lines), code);
    }

    #[test]
    fn a_comparison_is_not_a_tag_and_numbers_and_booleans_are_constants() {
        let lines = highlight(Lang::Rust, "if a < b && flag { true } else { 12 }");
        assert_eq!(class_of(&lines, "if"), Some("pl-k"));
        assert_eq!(class_of(&lines, "<"), None);
        assert_eq!(class_of(&lines, "true"), Some("di-bool"));
        assert_eq!(class_of(&lines, "12"), Some("pl-c1"));
    }

    #[test]
    fn css_properties_and_values_are_classified() {
        let lines = highlight(
            Lang::Css,
            ".Root {\n  display: flex;\n  color: oklch(14.5% 0 0deg);\n}",
        );
        assert_eq!(class_of(&lines, "Root"), Some("pl-ent"));
        assert_eq!(class_of(&lines, "display"), Some("di-cp"));
        assert_eq!(class_of(&lines, "flex"), Some("di-cv"));
        assert_eq!(class_of(&lines, "color"), Some("di-cp"));
    }

    #[test]
    fn the_scanner_emits_nothing_outside_the_declared_class_list() {
        // The fixture corpus covers every language the pages' fences carry, and every construct
        // the tokenizer classifies (comments, directives, strings, numbers, booleans, keywords,
        // `fn` names, macros, types, markup tags, attributes, CSS declarations).
        let corpus = [
            (Lang::Rust, CHECKBOX_ANATOMY),
            (
                Lang::Rust,
                "// @highlight-text \"button_element\"\nfn run() -> bool { let x = 12; true }",
            ),
            (
                Lang::Jsx,
                "import { Checkbox } from '@base-ui/react/checkbox';\n\n<Checkbox.Root id=\"x\">\n  <label>Text</label>\n</Checkbox.Root>;",
            ),
            (
                Lang::Css,
                ".Root {\n  display: flex;\n  color: oklch(0.5 0.22 31);\n}",
            ),
            (
                Lang::Html,
                "<style>\n.base-ui-disable-scrollbar { display: none; }\n</style>",
            ),
        ];
        for (lang, code) in corpus {
            for token in highlight(lang, code).iter().flatten() {
                for class in token.class.split_whitespace() {
                    assert!(
                        TOKEN_CLASSES.contains(&class),
                        "{lang:?} emitted class {class:?} (token {:?}) which the stylesheet guard \
                         does not know about — add it to TOKEN_CLASSES and to main.css together",
                        token.text
                    );
                }
            }
        }
    }

    #[test]
    fn the_stylesheet_colours_every_class_the_scanner_can_emit() {
        // The gap report's observable is a computed COLOUR inside `<pre>`, so a class with no rule
        // in the ported stylesheet scores as an uncoloured token. This is the guard that keeps the
        // two halves (the scanner's classes, the stylesheet's palette) from drifting apart.
        let css = include_str!("../style/main.css");
        for class in TOKEN_CLASSES {
            assert!(
                css.contains(&format!(".{class}")),
                "main.css has no rule for .{class} — every token carrying it renders uncoloured"
            );
        }
    }

    #[test]
    fn every_snippet_the_pages_embed_produces_coloured_tokens() {
        // The observable the gap report measures: at least one span inside `<pre>` whose computed
        // colour differs from the `pre`. A snippet that highlighted to nothing would score 0 even
        // with the chrome in place, so this is the cheap host-side guard for it. The floor is a
        // MEASURED value, not a rounded guess: the checkbox anatomy snippet scores 9 classified
        // tokens today, and every other snippet on the pages scores more.
        let colored = highlight(Lang::Rust, CHECKBOX_ANATOMY)
            .iter()
            .flatten()
            .filter(|t| !t.class.is_empty())
            .count();
        assert!(
            colored >= 8,
            "only {colored} coloured tokens in a real snippet"
        );
    }
}
