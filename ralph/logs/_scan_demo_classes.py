import re
import glob
import collections

PREFIXES = (
    'gap-', 'text-', 'px-', 'py-', 'size-', 'border', 'rounded', 'items-', 'justify-', 'bg-',
    'font-', 'w-', 'h-', 'min-', 'max-', 'mt-', 'mb-', 'ml-', 'mr-', 'mx-', 'my-', 'p-', 'pt-',
    'pb-', 'pl-', 'pr-', 'space-', 'overflow-', 'shadow', 'ring-', 'z-', 'top-', 'left-',
    'right-', 'bottom-', 'cursor-', 'select-', 'opacity-', 'duration-', 'transition',
    'outline-', 'col-', 'row-', 'self-', 'align-', 'leading-', 'tracking-', 'whitespace-',
    'break-', 'shrink-', 'grow', 'pointer-events-', 'capitalize', 'italic', 'underline',
    'list-', 'object-', 'divide-', 'isolate', 'aspect-', 'translate-', 'rotate-', 'scale-',
)
WORDS = {'flex', 'grid', 'block', 'hidden', 'inline', 'relative', 'absolute', 'truncate', 'uppercase'}

toks = collections.Counter()
string_re = re.compile(r'"([^"\\]{1,400})"')
files = sorted(glob.glob('/data/workspace/baseui/crates/docs-app/src/pages/*.rs'))
for f in files:
    src = open(f, encoding='utf-8').read()
    for m in string_re.finditer(src):
        parts = m.group(1).split()
        if not parts:
            continue
        if not any(t in WORDS or t.startswith(PREFIXES) for t in parts):
            continue
        for t in parts:
            toks[t] += 1

print('distinct tokens:', len(toks), 'total', sum(toks.values()))
for t, c in toks.most_common(400):
    print('%4d %s' % (c, t))
