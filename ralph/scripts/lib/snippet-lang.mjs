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
// Rules are deliberately lexical and conservative (a false "react" would fail a good page): React is
// asserted only by things that cannot appear in this port's code.

const LEPTOS_MARK = /use leptos|leptos_ui|leptos-ui|\bview!|#\[component\]|->\s*impl\s+IntoView|\bRwSignal|\bSignal<|\bCallback<|\bChildren\b|::[A-Z][A-Za-z]*|\bcx\(/;
const REACT_MARK = /@base-ui\/react|@mui\/|\bfrom\s+['"]react['"]|\brequire\(['"]react['"]\)|\buseState\b|\buseEffect\b|\buseRef\b|\buseMemo\b|\buseCallback\b|\bforwardRef\b|\bReactDOM\b|\bcreateRoot\b|\bReact\.[A-Z]|\bReactElement\b|\bReactNode\b|\bHTMLProps\b|\bprops\.children\b|className=|onClick=\{|=>\s*\(/;

/** Classify one snippet block: 'leptos' | 'react' | 'other'. */
export function classifySnippet(text) {
  const t = String(text || '');
  const leptos = LEPTOS_MARK.test(t);
  const react = REACT_MARK.test(t);
  if (leptos && !react) return 'leptos';
  if (react && !leptos) return 'react';
  if (react && leptos) return 'react';     // mixed: upstream's code with a Leptos line bolted on is not a port
  return 'other';
}

/** Count a page's blocks by language. */
export function classifyAll(texts) {
  const counts = { total: 0, leptos: 0, react: 0, other: 0 };
  for (const t of texts) { counts[classifySnippet(t)]++; counts.total++; }
  return counts;
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
// tamper
