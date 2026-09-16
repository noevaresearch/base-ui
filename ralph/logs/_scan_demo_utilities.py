"""Scan the port's docs-app pages for Tailwind-variant class tokens and resolve each against
upstream's own compiled stylesheet (the oracle). Diagnostic only."""
import re
import glob
import json
import collections

PAGES = '/data/workspace/baseui/crates/docs-app/src/pages/*.rs'
UPSTREAM_CSS = ['/tmp/upa.css', '/tmp/upb.css']
MAIN_CSS = '/data/workspace/baseui/crates/docs-app/style/main.css'

WORDS = {'flex', 'grid', 'block', 'hidden', 'inline', 'relative', 'absolute', 'truncate',
         'uppercase', 'italic', 'underline', 'shrink', 'grow', 'isolate'}
TOKEN_RE = re.compile(r'^[a-z0-9][a-z0-9\.:\-/_\[\]%#\(\),]*$')
STRING_RE = re.compile(r'"([^"\\\n]{1,600})"')

# ── 1. candidate tokens from the port's pages
cand = collections.Counter()
for f in sorted(glob.glob(PAGES)):
    for m in STRING_RE.finditer(open(f, encoding='utf-8').read()):
        for t in m.group(1).split():
            if TOKEN_RE.match(t) and ('-' in t or ':' in t or t in WORDS):
                cand[t] += 1

# ── 2. parse upstream's compiled css into selector -> rule bodies
def strip_comments(src):
    return re.sub(r'/\*.*?\*/', '', src, flags=re.S)

def parse_rules(src):
    """Return list of dicts {ctx: [at-rule prelude...], sel: selector, body: body text}."""
    out = []
    stack = []  # (prelude, kind)
    i = 0
    buf = ''
    src = strip_comments(src)
    while i < len(src):
        c = src[i]
        if c == '{':
            prelude = buf.strip()
            buf = ''
            if prelude.startswith('@') and not re.match(r'@(media|supports|layer|container|scope|starting-style)\b', prelude):
                # declaration block (e.g. @font-face, @property) — capture to matching brace
                depth = 1
                j = i + 1
                while j < len(src) and depth:
                    if src[j] == '{':
                        depth += 1
                    elif src[j] == '}':
                        depth -= 1
                    j += 1
                out.append({'ctx': list(stack), 'sel': prelude, 'body': src[i + 1:j - 1]})
                i = j
                buf = ''
                continue
            stack.append(prelude)
            buf = ''
        elif c == '}':
            if buf.strip() and stack:
                out.append({'ctx': list(stack), 'sel': buf.strip(), 'body': None})
            buf = ''
            if stack:
                stack.pop()
        elif c == ';':
            if buf.strip() and stack:
                prelude = buf.strip()
                # a nested declaration inside a rule (e.g. `color: red;`)
                out.append({'ctx': list(stack), 'sel': None, 'decl': prelude})
            buf = ''
        else:
            buf += c
        i += 1
    return out

rules = []
for p in UPSTREAM_CSS:
    rules.extend(parse_rules(open(p, encoding='utf-8').read()))

print('parsed entries:', len(rules))
ctxs = collections.Counter(tuple(r['ctx'][:2]) for r in rules)
for k, v in ctxs.most_common(12):
    print(' ctx', k, v)

# collect declarations in @layer theme / :root blocks
theme_vars = {}
for r in rules:
    if r.get('decl') and r['sel'] is None:
        m = re.match(r'^(--[a-z0-9\-]+):\s*(.*)$', r['decl'])
        if m and any('theme' in c or 'root' in c for c in r['ctx']):
            theme_vars.setdefault(m.group(1), m.group(2))
print('theme vars:', len(theme_vars))

# selector index: class token -> list of rule dicts
def class_tokens_of(sel):
    return re.findall(r'\.((?:\\.|[A-Za-z0-9_\-])+)', sel)

# a rule's selector is the innermost non-at-rule entry on its stack; its declarations are the
# `decl` entries that share that stack.
sel_index = collections.defaultdict(list)
decls_by_selector = collections.defaultdict(list)
for r in rules:
    if r.get('decl') and r['ctx']:
        owner = r['ctx'][-1]
        if not owner.startswith('@'):
            decls_by_selector[owner].append(r['decl'])
            for tok in class_tokens_of(owner):
                sel_index[re.sub(r'\\(.)', r'\1', tok)].append(owner)
print('indexed class tokens:', len(sel_index), 'selectors with decls:', len(decls_by_selector))

main_css = open(MAIN_CSS, encoding='utf-8').read()
main_selectors = set(re.findall(r'\.((?:\\.|[A-Za-z0-9_\-])+)', main_css))
main_selectors = {re.sub(r'\\(.)', r'\1', s) for s in main_selectors}

resolved = {}
missing = []
already = []
for t, n in cand.most_common():
    if t in main_selectors:
        already.append(t)
        continue
    if t in sel_index:
        resolved[t] = n
    else:
        missing.append((t, n))

print('candidates:', len(cand))
print('resolved in upstream css:', len(resolved))
print('already in main.css:', len(already), already[:20])
print('missing from upstream css:', len(missing))
for t, n in missing[:200]:
    print('   MISS', n, t)
json.dump({'resolved': sorted(resolved), 'missing': [t for t, _ in missing],
           'already': sorted(already)}, open('/data/workspace/baseui/ralph/logs/_demo_class_scan.json', 'w'), indent=1)
