import re, sys
from collections import Counter
p = '/data/workspace/baseui/TODO.md'
lines = open(p).read().split('\n')
items = []
cur = None
phase = '(pre)'
phase_of = {}
for i, l in enumerate(lines):
    if l.startswith('## '):
        phase = l[3:].strip()
    m = re.match(r'^- \[( |x)\] (.+)$', l)
    if m:
        if cur:
            items.append(cur)
        cur = {'line': i + 1, 'phase': phase, 'checked': m.group(1) == 'x',
               'id': m.group(2).strip(), 'fields': {}}
    elif cur is not None and re.match(r'^\s{6}\S', l) and ':' in l:
        k = l.strip().split(':', 1)[0]
        v = l.strip().split(':', 1)[1].strip()
        cur['fields'].setdefault(k, []).append(v)
if cur:
    items.append(cur)

print("total items:", len(items))
print("by phase:", Counter(it['phase'] for it in items))
print("by status:", Counter(it['fields'].get('status', ['<none>'])[-1] for it in items))

mode = sys.argv[1] if len(sys.argv) > 1 else 'open'
if mode == 'open':
    for it in items:
        st = it['fields'].get('status', ['?'])[-1]
        if st != 'done':
            print(f"{it['line']:5d} | {it['phase'][:28]:28s} | {it['id'][:52]:52s} | {st:11s} | bb: {it['fields'].get('blocked-by',['-'])[-1][:100]}")
elif mode == 'blocked':
    for it in items:
        st = it['fields'].get('status', ['?'])[-1]
        if st == 'blocked':
            print(f"=== {it['line']} {it['id']} ({it['phase']})")
            for n in it['fields'].get('note', []):
                print("   note:", n[:600])
elif mode == 'phase':
    want = sys.argv[2]
    for it in items:
        if want in it['phase']:
            st = it['fields'].get('status', ['?'])[-1]
            print(f"{it['line']:5d} | {it['id'][:60]:60s} | {st:11s} | bb: {it['fields'].get('blocked-by',['-'])[-1][:90]}")
elif mode == 'id':
    want = sys.argv[2]
    for it in items:
        if want in it['id']:
            print(json_dump(it) if False else '')
            print(f"=== line {it['line']} | {it['id']} | phase {it['phase']}")
            for k, vs in it['fields'].items():
                for v in vs:
                    print(f"   {k}: {v}")
