// snippet-lang.mjs — is this snippet written in THIS port's language, or is it upstream's React?
//
// WHY THIS EXISTS
// ---------------
// The ergonomics score measures how closely a mirrored example tracks upstream's shape. That makes it
// gameable in the most perverse way available: pasting upstream's React snippet scores a perfect match
// against itself, and a "fidelity" number would report the copy as the ideal port. The user's words:
// "I hope the code fidelity does not include react to react comparison". It must not, and until this
// module no Node-side check could even tell the two apart — the only classifier lived inside the
// browser probe in check-visual-budget/visual-diff, so snippet-ergonomics, the gate the loop is held to
// for snippet work, was language-blind.
//
// ONE DEFINITION, TWO RUNTIMES (2026-09-16)
// -----------------------------------------
// There used to be THREE copies of these rules: this module, and a private copy inside each of
// `visual-diff.mjs` and `visual-gap-report.mjs`. The copies drifted, and the divergence was not
// cosmetic — the probe copies carried the bare capitalized-tag React heuristic without the
// Leptos-EXCLUSIVE escape that `crates/docs-app/src/snippet_language.rs` documents (and names both
// probe copies as the place the escape must land). Consequence, measured on the accordion route: the
// port's own idiomatic `<AccordionRoot>` markup — Leptos `view!` over `leptos_ui` parts, which composes
// CONTRACT.md requirement 1 — was scored `react`, so its block was excluded from scoring and the route
// read 81.76 against its recorded 87.48 floor, failing the docs-app regression gate for every item.
// Fed to the gap report, the same copy raised a false P0 ("code snippets show React source") while
// check-page's own snippet-language axis — which reads THIS module and therefore said `leptos` — passed.
// Two instruments, one page, opposite verdicts: exactly the drift this file exists to prevent.
//
// So the rules now live in ONE self-contained function, `snippetLangProbe(texts)`, which runs unchanged
// in Node (snippet-ergonomics.mjs) and inside the browser probe: `SNIPPET_LANG_JS` is its source, injected
// into the in-page template the same way `lib/widget-region.mjs` injects `WIDGET_REGION_JS`. Do not copy
// the rules into another file — import them from here, or (in-page) spread SNIPPET_LANG_JS.
//
// PRECEDENCE (the part that was wrong in the copies)
// -------------------------------------------------
// A **Leptos-exclusive** marker is a string upstream's React/TypeScript sources can never contain. A
// block carrying one is this port's code no matter what else it looks like, so it wins outright. Only
// then do the ordinary marks decide. Without that rule the capitalized-tag heuristic — which is
// needed to catch upstream's TSX — scores idiomatic Leptos markup as React, and the standing
// instruction (ralph/logs/spec-discrepancies.md) is that such a false count is fixed in the CLASSIFIER,
// never by writing less idiomatic Leptos in the page.
//
// The remaining rules are deliberately lexical and conservative (a false "react" would fail a good page).

/** One snippet's language: 'leptos' | 'react' | 'other'. Rules are defined once, inside the probe below. */
export function snippetLangProbe(texts) {
  // strings upstream's React/TS sources cannot contain -> this port's own code, immediately
  const LEPTOS_EXCLUSIVE = /use leptos|leptos_ui|leptos-ui|\bview!|#\[component\]|->\s*impl\s+IntoView/;
  const LEPTOS_MARK = /\bRwSignal|\bSignal<|\bCallback<|\bChildren\b|::[A-Z][A-Za-z]*|\bcx\(|on:click|prop:|attr:/;
  const REACT_MARK = /@base-ui\/react|@mui\/|\bfrom\s+['"]react['"]|\brequire\(['"]react['"]\)|\buseState\b|\buseEffect\b|\buseRef\b|\buseMemo\b|\buseCallback\b|\bforwardRef\b|\bReactDOM\b|\bcreateRoot\b|\bReact\.[A-Z]|\bReactElement\b|\bReactNode\b|\bHTMLProps\b|\bprops\.children\b|className=|onClick=\{|=>\s*\(|\{\s*props/;
  // upstream's TSX shape: a capitalized element tag, dotted or not. Kept for recall (a demo showing
  // bare JSX has no import to give it away), and it is why the exclusive rule above must come first.
  const REACT_TSX_TAG = /<\/?[A-Z][A-Za-z]*(\.[A-Z][A-Za-z]*)?[\s/>]/;

  const classify = (text) => {
    const t = String(text || '');
    if (LEPTOS_EXCLUSIVE.test(t)) return 'leptos';
    const leptos = LEPTOS_MARK.test(t);
    const react = REACT_MARK.test(t) || REACT_TSX_TAG.test(t);
    if (leptos && !react) return 'leptos';
    if (react && !leptos) return 'react';
    if (react && leptos) return 'react'; // mixed: upstream's code with a Leptos line bolted on is not a port
    return 'other';
  };

  const counts = { total: 0, leptos: 0, react: 0, other: 0, perBlock: [] };
  for (const text of texts) {
    const c = classify(text);
    counts[c] += 1;
    counts.total += 1;
    counts.perBlock.push(c);
  }
  return counts;
}

/**
 * The SAME rules as in-page source, for the browser probes. Usage inside a page-evaluate template:
 *   snippets: (${SNIPPET_LANG_JS})([...main.querySelectorAll('pre')].map(p => p.textContent || ''))
 * Interpolating the function's own source is what keeps one definition (and it sidesteps the
 * backslash doubling a hand-written private copy needs).
 */
export const SNIPPET_LANG_JS = `(${snippetLangProbe.toString()})`;

/** Classify one snippet block: 'leptos' | 'react' | 'other'. */
export function classifySnippet(text) {
  return snippetLangProbe([text]).perBlock[0];
}

/** Count a page's blocks by language. */
export function classifyAll(texts) {
  const { total, leptos, react, other } = snippetLangProbe(texts);
  return { total, leptos, react, other };
}

/** Character-level similarity, used to catch a near-verbatim copy that dodged the lexical rules. */
export function verbatimRatio(a, b) {
  const norm = (s) => String(s || '').replace(/\s+/g, ' ').trim();
  const A = norm(a); const B = norm(b);
  if (!A || !B) return 0;
  const shorter = A.length <= B.length ? A : B;
  const longer = A.length <= B.length ? B : A;
  if (longer.includes(shorter)) return 1;
  // cheap LCS ratio on lines then chars of the shorter string
  let lcs = 0;
  const dp = new Array(shorter.length + 1).fill(0);
  for (let i = 1; i <= longer.length; i++) {
    let prev = 0;
    for (let j = 1; j <= shorter.length; j++) {
      const tmp = dp[j];
      dp[j] = longer[i - 1] === shorter[j - 1] ? prev + 1 : Math.max(dp[j], dp[j - 1]);
      prev = tmp;
    }
  }
  lcs = dp[shorter.length];
  return lcs / shorter.length;
}
