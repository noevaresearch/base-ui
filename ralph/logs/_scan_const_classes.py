"""Prototype of the host drift guard: every class token in a page's demo class constants must be
defined by style/main.css. Prints offenders (empty = the guard would pass)."""
import re
import glob

BASE = '/data/workspace/baseui/crates/docs-app'
CONST_RE = re.compile(r'const\s+([A-Z0-9_]*CLASS[A-Z0-9_]*)\s*:\s*&str\s*=\s*"([^"]*)"')
TOKEN_RE = re.compile(r'^[a-z0-9][a-z0-9.:\-/_[\]%#(),]*$')

main = open(BASE + '/style/main.css', encoding='utf-8').read()
sels = {re.sub(r'\\(.)', r'\1', s) for s in re.findall(r'\.((?:\\.|[A-Za-z0-9_-])+)', main)}

offenders = []
total = 0
consts = 0
for f in sorted(glob.glob(BASE + '/src/pages/*.rs')):
    src = open(f, encoding='utf-8').read()
    for m in CONST_RE.finditer(src):
        consts += 1
        for t in m.group(2).split():
            total += 1
            if not TOKEN_RE.match(t):
                continue
            if t not in sels:
                offenders.append((f.split('/')[-1], m.group(1), t))

print('class consts:', consts, 'tokens:', total, 'offenders:', len(offenders))
for f, c, t in offenders[:60]:
    print('  %-28s %-28s %s' % (f, c, t))
