// ast-compare.mjs — tree-sitter based structural comparison for mirrored docs snippets.
//
// WHY: the regex signature in snippet-ergonomics.mjs gets the gist (element sequence, attribute
// counts, raw-call smells) but it is a lexer, not a parser: it cannot tell nesting from siblings,
// cannot skip strings/comments reliably, and cannot see that `<Checkbox.Root>` in React and
// `<CheckboxRoot>` in Leptos are the same node with a different name.
//
// FEASIBILITY, MEASURED (spike 2026-09-16):
//   * web-tree-sitter 0.25.6 + tree-sitter-wasms 0.1.13 parse both sides. The version pairing
//     matters: with web-tree-sitter 0.27.0, Language.load failed inside getDylinkMetadata (ABI
//     mismatch), so the pinned pair is part of this contract, not incidental.
//   * tree-sitter-javascript parses JSX natively (jsx_element / jsx_self_closing_element).
//   * tree-sitter-RUST DOES NOT parse view! markup as elements — a macro body is a token tree. The
//     markup inside view! { .. } is therefore extracted and parsed with tree-sitter-html, which
//     yields element / self_closing_tag nodes directly. That two-grammar route is the trick.
//
// This module is optional precision: snippet-ergonomics.mjs falls back to the regex signature when
// the grammar directory is absent (TS_ERGO_HOME or /data/tools/ts-ergonomics), so the repo never
// hard-depends on 56MB of wasm.
import fs from 'node:fs';
import path from 'node:path';

const GRAMMAR_HOME = process.env.TS_ERGO_HOME || '/data/tools/ts-ergonomics';
const WASM_DIR = path.join(GRAMMAR_HOME, 'node_modules/tree-sitter-wasms/out');
const MODULE_DIR = GRAMMAR_HOME;

export function astAvailable() {
  try {
    return fs.existsSync(path.join(WASM_DIR, 'tree-sitter-javascript.wasm')) &&
      fs.existsSync(path.join(WASM_DIR, 'tree-sitter-html.wasm'));
  } catch {
    return false;
  }
}

/** Load web-tree-sitter + grammars from the pinned pair. Throws if unavailable (caller falls back). */
export async function loadGrammars() {
  const { Parser, Language } = await import(path.join(MODULE_DIR, 'node_modules/web-tree-sitter/tree-sitter.js'));
  await Parser.init();
  const js = await Language.load(path.join(WASM_DIR, 'tree-sitter-javascript.wasm'));
  const html = await Language.load(path.join(WASM_DIR, 'tree-sitter-html.wasm'));
  return { Parser, js, html };
}

/** Normalise a component/element name so React's dotted names and Leptos' names compare. */
export function canonName(name) {
  return name
    .replace(/::/g, '.')
    .replace(/_/g, '.')
    .toLowerCase()
    .split('.')
    .filter(Boolean)
    .join('');
}

function tagNameOf(text) {
  const m = text.match(/^<\s*\/?\s*([A-Za-z_][\w:.]*)/);
  return m ? m[1] : null;
}

function selfClosingOf(text) {
  return /\/>\s*$/.test(text);
}

/**
 * Element tree from a React/JSX snippet: [{ name, canon, depth, selfClosing, attrs }] in document
 * order. `attrs` comes from the parsed opening tag's attribute nodes rather than a regex.
 */
export function reactElementTree(parser, lang, src) {
  const tree = parser.parse(src);
  const out = [];
  const walk = (node, depth) => {
    const t = node.type;
    if (t === 'jsx_self_closing_element') {
      const name = tagNameOf(node.text);
      out.push({ name, canon: canonName(name || ''), depth, selfClosing: true, attrs: countJsxAttrs(node) });
      return;
    }
    if (t === 'jsx_element') {
      const opening = node.children.find((c) => c.type === 'jsx_opening_element');
      const name = opening ? tagNameOf(opening.text) : null;
      out.push({ name, canon: canonName(name || ''), depth, selfClosing: false, attrs: opening ? countJsxAttrs(opening) : 0 });
      for (const child of node.children) walk(child, depth + 1);
      return;
    }
    for (const child of node.children) walk(child, depth);
  };
  walk(tree.rootNode, 0);
  return out;
}

function countJsxAttrs(jsxNode) {
  // jsx_attribute nodes are direct children of the opening/self-closing element
  return jsxNode.children.filter((c) => c.type === 'jsx_attribute' || c.type === 'jsx_spread_attribute').length;
}

/**
 * Element tree from a Leptos snippet: extract the body of each `view! { .. }` (balanced braces) and
 * parse it as HTML, since tree-sitter-rust sees only a macro token tree.
 */
export function leptosElementTree(parser, lang, src) {
  const out = [];
  const bodies = extractViewBodies(src);
  for (const body of bodies) {
    const tree = parser.parse(body);
    const walk = (node, depth) => {
      if (node.type === 'element' || node.type === 'self_closing_tag') {
        const name = tagNameOf(node.text);
        out.push({ name, canon: canonName(name || ''), depth, selfClosing: node.type === 'self_closing_tag', attrs: countHtmlAttrs(node) });
      }
      for (const child of node.children) walk(child, depth + 1);
    };
    walk(tree.rootNode, 0);
  }
  return out;
}

function countHtmlAttrs(node) {
  // tree-sitter-html: `<div a="1">` -> element node with a start_tag child carrying `attribute`
  // nodes; `<div a="1" />` -> self_closing_tag with the attributes as direct children.
  if (node.type === 'self_closing_tag') {
    return node.children.filter((c) => c.type === 'attribute').length;
  }
  const start = node.children.find((c) => c.type === 'start_tag');
  return start ? start.children.filter((c) => c.type === 'attribute').length : 0;
}

/** Balanced-brace extraction of every `view! { .. }` macro body. */
export function extractViewBodies(src) {
  const bodies = [];
  const re = /view!\s*\{/g;
  let m;
  while ((m = re.exec(src)) !== null) {
    let depth = 0;
    let i = m.index + m[0].length - 1;
    const start = i;
    for (; i < src.length; i++) {
      if (src[i] === '{') depth++;
      else if (src[i] === '}') {
        depth--;
        if (depth === 0) break;
      }
    }
    bodies.push(src.slice(start + 1, i));
  }
  return bodies;
}

/** Longest-common-subsequence ratio over canonical name sequences. */
export function lcsRatio(a, b) {
  if (!a.length || !b.length) return 0;
  const dp = Array.from({ length: a.length + 1 }, () => new Array(b.length + 1).fill(0));
  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      dp[i][j] = a[i - 1] === b[j - 1] ? dp[i - 1][j - 1] + 1 : Math.max(dp[i - 1][j], dp[i][j - 1]);
    }
  }
  return dp[a.length][b.length] / Math.max(a.length, b.length);
}

/** Compare two element trees: shape by LCS, naming by dotted-component coverage, nesting depth. */
export function compareTrees(up, lx) {
  const upComps = up.filter((e) => e.name && /^[A-Z]/.test(e.name.split(/[.:]/).pop() || ''));
  const lxDotted = upComps.filter((e) => e.name.includes('.'));
  const lxNames = [...new Set(lx.map((e) => e.canon))];
  // SUFFIX matching, because the same component is spelled several ways across the two frameworks:
  // upstream's `Checkbox.Root` canonises to `checkboxroot`, and the port may offer it as
  // `CheckboxRoot` (same) or as a namespaced path such as `ui::Checkbox::Root` -> `uicheckboxroot`.
  // Requiring equality made the port's own recommended idiom score as NO MATCH (measured
  // 2026-09-16); a suffix test accepts the namespaced form while still failing `label` against
  // upstream's `Field.Label` -> `fieldlabel`.
  const matched = [...new Set(lxDotted.map((e) => e.canon))]
    .filter((c) => lxNames.some((n) => n === c || n.endsWith(c)));
  const uniqueDotted = [...new Set(lxDotted.map((e) => e.canon))];
  return {
    shape: lcsRatio(up.map((e) => e.canon), lx.map((e) => e.canon)),
    naming: uniqueDotted.length ? matched.length / uniqueDotted.length : 1,
    upNodes: up.length,
    lxNodes: lx.length,
    upMaxDepth: up.reduce((d, e) => Math.max(d, e.depth), 0),
    lxMaxDepth: lx.reduce((d, e) => Math.max(d, e.depth), 0),
    missingComponents: uniqueDotted.filter((c) => !lxNames.some((n) => n === c || n.endsWith(c))),
    upAttributes: up.reduce((n, e) => n + (e.attrs || 0), 0),
    lxAttributes: lx.reduce((n, e) => n + (e.attrs || 0), 0),
  };
}
