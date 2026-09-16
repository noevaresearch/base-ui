// source-scope.mjs — one shared answer to "is this line READER-FACING page content?".
//
// WHY THIS EXISTS
// ---------------
// Two gates scan the port's own page sources for React leakage (`check-react-mentions.mjs --source`,
// `check-package-alias.mjs` rule 4) and both write the same sentence in their header: the scope is
// what the READER is taught. Both then applied that scope as "every line of every .rs under
// crates/docs-app/src", which is not the same thing, and each paid for it separately:
//
//   * `#[cfg(test)]` fixtures. A `snippet_language_guard` module keeps a verbatim upstream snippet
//     as a POSITIVE CONTROL, so that its "every snippet teaches the port" assertion cannot pass
//     vacuously. The gate read that control as page copy and failed the run. (`check-react-mentions`
//     already excluded `*_test.rs` files by filename — the same intent, not carried to the inline
//     form the guard modules actually use.)
//   * a mirrored EXAMPLE block's first line. `code_block(Lang::Jsx, "Anatomy", "import { X } from
//     '@base-ui/react/x'; …")` is a snippet-language defect — that is what the per-route snippet
//     instruments measure (`visual-gap-report.mjs` react>0, `check-page.mjs`'s snippet-language
//     axis, `check-visual-budget.mjs`'s snippetLanguage purity) and it is owned by the
//     `docs-chrome: snippet translation` items. Counting it as "a page tells the reader to install
//     upstream's package" made the install-line item un-passable for work it does not own and must
//     not pre-empt (its own note forbids translating snippets the surface does not yet exist for).
//
// An INSTALL INSTRUCTION is never excusable, in a code block or out of one: `npm install
// @base-ui/react` sends the reader to a different library even when it is fenced as an example. So
// the install-command test runs FIRST and outranks the code-literal test.
//
// Exit-code policy stays with the caller: these helpers CLASSIFY, they do not decide. A caller that
// only claims some of the classes filters with `--fail-on` and still prints the rest.

/// 1-based [start, end] line ranges of every `#[cfg(test)]` item in a Rust source text.
/// Brace-matched from the attribute, so nested blocks inside the module are covered.
export function cfgTestRanges(text) {
  const lines = text.split('\n');
  const ranges = [];
  for (let i = 0; i < lines.length; i++) {
    if (!/^\s*#\[cfg\(test\)\]/.test(lines[i])) continue;
    let depth = 0;
    let started = false;
    let j = i;
    for (; j < lines.length; j++) {
      for (const ch of lines[j]) {
        if (ch === '{') { depth++; started = true; } else if (ch === '}') depth--;
      }
      if (started && depth <= 0) break;
    }
    ranges.push([i + 1, Math.min(j + 1, lines.length)]);
    i = j;
  }
  return ranges;
}

export function inRanges(ranges, lineNumber) {
  return ranges.some(([a, b]) => lineNumber >= a && lineNumber <= b);
}

/// An install instruction for upstream's package. Never excusable: the reader is being told to
/// install a different library, in a different language, whether or not the line sits in a fence.
export const INSTALL_COMMAND_RE = /npm\s+(?:install|i|add)\s+[^\n"']*@base-ui\/react|yarn\s+add\s+[^\n"']*@base-ui\/react|pnpm\s+(?:add|install)\s+[^\n"']*@base-ui\/react/i;

/// 1-based [start, end] line ranges of every `code_block(...)` CALL in a Rust source text — i.e. the
/// extents of the port's snippet DATA, the code a reader sees inside a `<pre>` — each with the `Lang`
/// variant the call declares (`'rust'`, `'jsx'`, `'tsx'`, `'css'`, …).
///
/// This is a tiny string-aware scanner, not a parser, and it has to be: the snippets carry real
/// newlines, apostrophes and unbalanced-looking parentheses INSIDE their literals, so paren counting
/// that ignores string state mis-nests immediately. The scanner therefore skips string contents
/// (honouring `\"`) and line comments, records the identifier immediately before each `(`, and
/// accumulates the call's NON-string characters — which is where `Lang::Rust` sits. The `lang` is
/// needed because `props.children` is a React API in a JSX example and an ordinary struct-field
/// access in a Rust one; only the former is a defect.
export function codeBlockRanges(text) {
  const ranges = [];
  const stack = [];
  let ident = '';
  let inStr = false;
  let esc = false;
  const lines = text.split('\n');
  for (let li = 0; li < lines.length; li++) {
    const line = lines[li];
    for (let ci = 0; ci < line.length; ci++) {
      const ch = line[ci];
      if (inStr) {
        if (esc) esc = false;
        else if (ch === '\\') esc = true;
        else if (ch === '"') inStr = false;
        continue;
      }
      if (ch === '/' && line[ci + 1] === '/') break;      // a line comment is not code
      if (ch === '"') { inStr = true; ident = ''; continue; }
      if (ch === '(') { stack.push({ name: ident, line: li + 1, code: '' }); ident = ''; continue; }
      if (ch === ')') {
        const top = stack.pop();
        if (top && top.name === 'code_block') {
          ranges.push({ start: top.line, end: li + 1, lang: (top.code.match(/Lang::(\w+)/)?.[1] || '').toLowerCase() || null });
        }
        ident = '';
        continue;
      }
      if (/[A-Za-z0-9_:]/.test(ch)) ident += ch;
      else ident = '';
      for (const fr of stack) fr.code += ch;
    }
    if (!inStr) ident = '';
  }
  return ranges;
}

export function codeBlockAt(ranges, lineNumber) {
  return ranges.find(({ start, end }) => lineNumber >= start && lineNumber <= end) || null;
}

/// Is this hit snippet LANGUAGE (owned by the snippet-translation items) rather than page copy?
/// An install COMMAND outranks the classification — it is fatal wherever it appears.
export function isSnippetLanguageHit(line, lineNumber, ranges) {
  if (INSTALL_COMMAND_RE.test(line)) return false;
  return Boolean(lineNumber != null && ranges && codeBlockAt(ranges, lineNumber));
}

/// `props.children` is a React idiom in a JSX example and a plain field access in Rust. This is the
/// rest of the react-api vocabulary — the markers that are unambiguous in ANY language.
export const REACT_API_UNAMBIGUOUS_RE = /\b(useState|useEffect|useRef|useMemo|useCallback|useReducer|forwardRef|createContext|ReactDOM|createRoot|React\.[A-Z]\w*|HTMLProps|JSXElement|JSX)\b/;

/// True when a `props.children` hit sits in this port's own Rust, where it is a struct-field access
/// rather than an upstream API. In a `.rs` file the field access is the normal reading, so:
///   * inside a `Lang::Rust` code block      -> Rust, excused;
///   * outside any code block                -> plain Rust module code (`let children = props.children;`
///                                              in a demo implementation), excused. A PROSE string
///                                              outside `code_block` is not distinguishable from code
///                                              here — which is why the rendered scan, which sees prose
///                                              as a `<p>` leaf and never excuses it, runs with `--all`;
///   * inside a JSX/TSX code block           -> a defect: `props.children` is upstream's idiom there.
/// An unambiguous React marker on the same line (`useState`, `React.*`, `HTMLProps`, `JSX`) always wins.
export function propsChildrenIsRustFieldAccess(scannable, { block = null, renderedCodeText = null } = {}) {
  if (!/\bprops\.children\b/.test(scannable)) return false;
  if (REACT_API_UNAMBIGUOUS_RE.test(scannable)) return false;
  if (renderedCodeText) return /\bfn\s|\bimpl\s|->/.test(renderedCodeText);   // rendered leaf: Rust-shaped?
  if (block === null) return true;                                          // not in a code block: it IS Rust source
  return block.lang === 'rust' || block.lang === null;
}
