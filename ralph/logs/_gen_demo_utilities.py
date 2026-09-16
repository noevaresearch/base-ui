"""Generate the demo utilities section for crates/docs-app/style/main.css.

Provenance: every rule is copied verbatim from upstream's OWN compiled stylesheet (the docs site
served at :3005), which is the oracle this port measures against. Only the utility classes that
the port's pages actually reference are emitted, plus the theme custom properties those rules
read.
"""
import re
import glob
import json
import collections
import sys

BASE = '/data/workspace/baseui'
PAGES = BASE + '/crates/docs-app/src/pages/*.rs'
UPSTREAM_CSS = ['/tmp/upa.css', '/tmp/upb.css']
MAIN_CSS = BASE + '/crates/docs-app/style/main.css'

WORDS = {'flex', 'grid', 'block', 'hidden', 'inline', 'relative', 'absolute', 'truncate',
         'uppercase', 'italic', 'underline', 'shrink', 'grow', 'isolate'}
TOKEN_RE = re.compile(r'^[a-z0-9][a-z0-9\.:\-/_\[\]%#\(\),]*$')
STRING_RE = re.compile(r'"([^"\\\n]{1,600})"')


def parse_rules(src):
    src = re.sub(r'/\*.*?\*/', '', src, flags=re.S)
    out = []
    stack = []
    buf = ''
    i = 0
    while i < len(src):
        c = src[i]
        if c == '{':
            prelude = buf.strip()
            buf = ''
            if prelude.startswith('@') and not re.match(
                    r'@(media|supports|layer|container|scope|starting-style)\b', prelude):
                depth = 1
                j = i + 1
                while j < len(src) and depth:
                    if src[j] == '{':
                        depth += 1
                    elif src[j] == '}':
                        depth -= 1
                    j += 1
                i = j
                buf = ''
                continue
            stack.append(prelude)
        elif c == '}':
            buf = ''
            if stack:
                stack.pop()
        elif c == ';':
            if buf.strip() and stack:
                out.append((list(stack), buf.strip()))
            buf = ''
        else:
            buf += c
        i += 1
    return out


# ── rules: stack (at-rule context + innermost selector) + one declaration
entries = []
for p in UPSTREAM_CSS:
    entries.extend(parse_rules(open(p, encoding='utf-8').read()))

theme_vars = {}
rules = collections.OrderedDict()      # (ctx tuple, selector) -> [decls]
for stack, decl in entries:
    owner = stack[-1] if stack else None
    if owner is None or owner.startswith('@'):
        if re.match(r'^--[a-z0-9\-]+:', decl):
            name, _, val = decl.partition(':')
            theme_vars.setdefault(name.strip(), val.strip())
        continue
    if decl.startswith('--') and 'theme' in ' '.join(stack):
        name, _, val = decl.partition(':')
        theme_vars.setdefault(name.strip(), val.strip())
        continue
    key = (tuple(stack[:-1]), owner)
    rules.setdefault(key, []).append(decl)

by_token = collections.defaultdict(list)
for (ctx, sel), decls in rules.items():
    for tok in re.findall(r'\.((?:\\.|[A-Za-z0-9_\-])+)', sel):
        by_token[re.sub(r'\\(.)', r'\1', tok)].append((ctx, sel, decls))

# ── candidate tokens in the port's pages
cand = collections.Counter()
for f in sorted(glob.glob(PAGES)):
    for m in STRING_RE.finditer(open(f, encoding='utf-8').read()):
        for t in m.group(1).split():
            if TOKEN_RE.match(t) and ('-' in t or ':' in t or t in WORDS):
                cand[t] += 1

main_css = open(MAIN_CSS, encoding='utf-8').read()
main_sels = {re.sub(r'\\(.)', r'\1', s) for s in re.findall(r'\.((?:\\.|[A-Za-z0-9_\-])+)', main_css)}

chosen = collections.OrderedDict()
missing = []
for t, _ in cand.most_common():
    if t in main_sels:
        continue
    if t in by_token:
        for ctx, sel, decls in by_token[t]:
            chosen[(ctx, sel)] = decls
    else:
        missing.append(t)

# needed theme vars, transitively
def vars_in(text):
    return set(re.findall(r'var\(\s*(--[a-z0-9\-]+)', text))

needed = set()
queue = []
for decls in chosen.values():
    queue.extend(vars_in(' '.join(decls)))
while queue:
    v = queue.pop()
    if v in needed:
        continue
    needed.add(v)
    if v in theme_vars:
        queue.extend(vars_in(theme_vars[v]))

out = []
out.append('/* ═══════════════════════════════════════════════════════════════════════════════════')
out.append(' * DEMO UTILITIES — generated, do not hand-edit.')
out.append(' *')
out.append(' * WHY: upstream\'s docs pages render the CSS-MODULES variant of each demo, but the')
out.append(' * ported pages carry the TAILWIND variant\'s class strings verbatim (upstream ships')
out.append(' * both). This crate compiles no Tailwind, so those strings were inert: the checkbox')
out.append(' * demo\'s control laid out as a full-width block (768x41 vs upstream\'s 150x20), the')
out.append(' * button rendered bare text (53x24 against upstream\'s bordered 72x32) and the widget')
out.append(' * parity gate read 84.42% on button with checkbox/meter not comparable at all.')
out.append(' *')
out.append(' * WHAT: the utility rules that those class strings name, copied VERBATIM from')
out.append(' * upstream\'s own compiled stylesheet (the docs site\'s Tailwind output at :3005 —')
out.append(' * the same oracle ralph/scripts/check-visual-budget.mjs measures against), plus the')
out.append(' * theme custom properties they read. Nothing here is invented or hand-tuned.')
out.append(' *')
out.append(' * REGENERATE: node ralph/scripts/gen-demo-utilities.mjs   (needs both dev servers up)')
out.append(' *')
out.append(' * CASCADE: emitted UNLAYERED and last, so a utility beats an earlier chrome rule of')
out.append(' * equal specificity — the precedence upstream gets from `@layer utilities` winning')
out.append(' * over its components layer.')
out.append(' * ═══════════════════════════════════════════════════════════════════════════════════ */')
out.append('')
if needed:
    out.append('/* theme custom properties the rules below read (upstream docs/src/css/index.css @theme) */')
    out.append(':root {')
    for v in sorted(needed):
        if v in theme_vars:
            out.append('  %s: %s;' % (v, theme_vars[v]))
    out.append('}')
    out.append('')

for (ctx, sel), decls in chosen.items():
    for at in ctx:
        out.append(at + ' {')
    out.append(sel + ' {')
    for d in decls:
        out.append('  ' + d + ';')
    out.append('}')
    for _ in ctx:
        out.append('}')
    out.append('')

css = '\n'.join(out)
open('/tmp/gen.css', 'w', encoding='utf-8').write(css)
print('tokens resolved:', len([t for t in cand if t in by_token]))
print('selectors emitted:', len(chosen))
print('theme vars emitted:', len([v for v in needed if v in theme_vars]), 'of', len(needed), 'needed')
print('unresolved tokens (port classes + prose):', len(missing))
print('css bytes:', len(css))
json.dump({'missing': missing}, open(BASE + '/ralph/logs/_demo_utilities_missing.json', 'w'), indent=1)
