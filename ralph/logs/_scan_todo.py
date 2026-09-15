import re
from collections import Counter

p = '/data/workspace/baseui/TODO.md'
lines = open(p).read().split('\n')
items = []
cur = None
phase = None
for i, l in enumerate(lines, 1):
    if l.startswith('## '):
        phase = l[3:].strip()
    m = re.match(r'^- \[( |x)\] (.*)$', l)
    if m:
        if cur:
            items.append(cur)
        cur = {'line': i, 'checked': m.group(1) == 'x', 'id': m.group(2).strip(), 'phase': phase, 'fields': {}}
        continue
    if cur is not None:
        fm = re.match(r'^\s+(blocked-by|status|docs-pair|exempt-from-docs-pairing|crate|specs|done-when|commit|needs-batched-mining|owner):\s*(.*)$', l)
        if fm:
            cur['fields'][fm.group(1)] = fm.group(2).strip()
if cur:
    items.append(cur)

print("total items", len(items))
print(Counter(x['fields'].get('status', 'NO_STATUS') for x in items))
print()
print("NOT-STARTED:")
for x in items:
    if x['fields'].get('status') == 'not-started':
        print(f"  L{x['line']:5} {x['id']:52} crate={x['fields'].get('crate','?')[:16]:16} blocked-by={x['fields'].get('blocked-by','?')[:60]:60} pair={x['fields'].get('docs-pair','-')}")
print()
print("BLOCKED:")
for x in items:
    if x['fields'].get('status') == 'blocked':
        print(f"  L{x['line']:5} {x['id']:52} crate={x['fields'].get('crate','?')}")
print()
print("done Phase D (docs-content):")
for x in items:
    if x['phase'] and x['phase'].startswith('## Phase D') and x['fields'].get('status') == 'done':
        print(f"  L{x['line']:5} {x['id']}")
