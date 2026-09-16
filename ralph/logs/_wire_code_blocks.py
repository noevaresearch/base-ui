"""Rewire every mirrored page's embedded snippets through `crate::code_block`.

One-shot transform for the `docs-chrome: code blocks` item, run once and kept in the repo as the
record of HOW the 44 call sites were mapped (the mapping rule is content matching against the
page's own upstream `.mdx` fences, not positional guessing).

For each page:

  * form A — `<pre><code>{CONST}</code></pre>`: the const's own literal is looked up in the file.
  * form B — `<pre><code>"…literal…"</code></pre>` (single- or multi-line): the literal itself.

The snippet's text is then matched against every fence in the page's upstream `page.mdx` by
line-set similarity, and the best-matching unused fence supplies the title and the language. A
fence's title is what upstream renders in the block's panel (`CodeBlockPreComputedContent` renders
the panel only for a titled fence), so a mismatched title would put the wrong label on the block.
"""

import glob
import os
import re
import sys

ROOTS = [
    'docs/src/app/(docs)/react/components',
    'docs/src/app/(docs)/react/utils',
    'docs/src/app/(docs)/react/overview',
]

# Pages whose snippets have been TRANSLATED to the port's own API by `docs-content:` iterations —
# their code is Rust (`view!` markup), so the fence's language is rust even though upstream's
# identical fence is jsx/tsx. Every other page still carries upstream's React source (the
# `docs-spec:` translation queue), so its mdx language is the honest one.
TRANSLATED = {'checkbox', 'button'}

LANG_ENUM = {
    'rust': 'Lang::Rust',
    'jsx': 'Lang::Jsx',
    'tsx': 'Lang::Tsx',
    'ts': 'Lang::Tsx',
    'css': 'Lang::Css',
    'html': 'Lang::Html',
}


def mdx_fences(name):
    slug = name.replace('_', '-')
    for root in ROOTS:
        path = os.path.join(root, slug, 'page.mdx')
        if not os.path.exists(path):
            continue
        fences = []
        lines = open(path).read().split('\n')
        i = 0
        while i < len(lines):
            m = re.match(r'^```(\S+)\s*(.*)$', lines[i])
            if m:
                body = []
                i += 1
                while i < len(lines) and not lines[i].startswith('```'):
                    body.append(lines[i])
                    i += 1
                title = ''
                tm = re.search(r'title="([^"]*)"', m.group(2))
                if tm:
                    title = tm.group(1)
                fences.append({'lang': m.group(1), 'title': title, 'body': body})
            i += 1
        return fences
    return []


def unescape(lit):
    """Best-effort text of a Rust string literal, for COMPARISON only (never emitted)."""
    text = lit
    if text.startswith('r#"') and text.endswith('"#'):
        return text[3:-2]
    if text.startswith('"') and text.endswith('"'):
        text = text[1:-1]
        text = text.replace('\\n', '\n').replace('\\"', '"').replace('\\\\', '\\')
    return text


def norm_lines(text):
    return {ln.strip().lower() for ln in text.split('\n') if ln.strip()}


def similarity(a, b):
    if not a or not b:
        return 0.0
    return len(a & b) / len(a | b)


def rust_literal(src, start):
    """The Rust string literal beginning at `start`, or None.

    Handles both spellings the pages use: a raw string `r#"…"#` (real newlines inside, and `;`
    characters inside — which is why the caller cannot regex for the terminating `;`) and a
    regular `"…"` literal (escapes, possibly spanning lines).
    """
    if src.startswith('r#"', start):
        end = src.find('"#', start + 3)
        return None if end < 0 else src[start : end + 2]
    if src.startswith('"', start):
        i = start + 1
        while i < len(src):
            if src[i] == '\\':
                i += 2
                continue
            if src[i] == '"':
                return src[start : i + 1]
            i += 1
    return None


def const_literal(src, const):
    m = re.search(r'const %s: &str = ' % re.escape(const), src)
    if not m:
        return ''
    lit = rust_literal(src, m.end())
    return unescape(lit) if lit else ''


def process(path):
    name = os.path.basename(path)[:-8]
    src = open(path).read()
    fences = mdx_fences(name)
    sites = list(re.finditer(r'( *)<pre><code>(.*?)</code></pre>', src, re.S))
    remaining_sites = len(sites)
    used = set()
    out = []
    pos = 0
    count = 0
    report = []
    for m in sites:
        indent, inner = m.group(1), m.group(2).strip()
        if inner.startswith('{') and inner.endswith('}'):
            const = inner[1:-1].strip()
            text = const_literal(src, const)
            expr = const
            form = 'A'
        else:
            text = unescape(inner)
            expr = inner
            form = 'B'
        wanted = norm_lines(text)
        remaining_fences = len(fences) - len(used)
        # When the page's remaining fences and remaining snippets are one-to-one, position is a
        # safe key (the pages mirror the mdx in order); otherwise the snippet's own text decides,
        # which is what keeps a page whose mdx has more fences than the port renders (use-render)
        # or an untitled extra snippet (form, merge-props) from getting the wrong label.
        positional = remaining_fences == remaining_sites
        best, best_score = None, 0.0
        for idx, fence in enumerate(fences):
            if idx in used:
                continue
            if positional:
                best = idx
                break
            score = similarity(wanted, norm_lines('\n'.join(fence['body'])))
            if score > best_score:
                best, best_score = idx, score
        if best is None:
            # Upstream's untitled fence: no panel, no copy control, no label.
            lang = 'rust' if name in TRANSLATED else 'tsx'
            lang_enum = LANG_ENUM[lang]
            report.append('  line %d: %-9s form %s title=%r (no fence left)' % (
                src[:m.start()].count('\n') + 1, lang_enum, form, ''))
            title = ''
        else:
            used.add(best)
            fence = fences[best]
            lang = 'rust' if name in TRANSLATED else fence['lang']
            lang_enum = LANG_ENUM.get(lang, 'Lang::Jsx')
            # A content match below the threshold means the label cannot be trusted; upstream
            # renders an untitled block in that case rather than the wrong title.
            title = '' if (not positional and best_score < 0.34) else fence['title']
            report.append('  line %d: %-9s form %s title=%-52r %s %.2f' % (
                src[:m.start()].count('\n') + 1, lang_enum, form, title,
                'positional' if positional else 'match', best_score))
        title = title.replace('"', '\\"')
        out.append(src[pos:m.start()])
        if '\n' in expr:
            out.append('%s{code_block(\n%s    %s,\n%s    "%s",\n%s    %s,\n%s)}' % (
                indent, indent, lang_enum, indent, title, indent, expr, indent))
        else:
            out.append('%s{code_block(%s, "%s", %s)}' % (indent, lang_enum, title, expr))
        pos = m.end()
        count += 1
        remaining_sites -= 1
    if not count:
        return None
    out.append(src[pos:])
    new = ''.join(out)
    if 'use crate::code_block::{code_block, Lang};' not in new:
        m = re.search(r'^(use [^\n]*;\n)', new, re.M)
        anchor = m.end()
        new = new[:anchor] + 'use crate::code_block::{code_block, Lang};\n' + new[anchor:]
    open(path, 'w').write(new)
    return (name, count, report)


for f in sorted(glob.glob('crates/docs-app/src/pages/*_page.rs')):
    if '<pre>' not in open(f).read():
        continue
    res = process(f)
    if res:
        name, sites, report = res
        print('== %s: %d site(s)' % (name, sites))
        print('\n'.join(report))
