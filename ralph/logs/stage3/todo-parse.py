import re, collections, sys

lines = open('/data/workspace/baseui/TODO.md').read().split('\n')
entries = []
cur = None
for i, ln in enumerate(lines, 1):
    m = re.match(r'^- \[(.)\] (.+)$', ln)
    if m:
        cur = {'id': m.group(2).strip(), 'checked': m.group(1), 'line': i, 'fields': {}}
        entries.append(cur)
        continue
    if cur is not None:
        m2 = re.match(r'^\s{6}(\w[\w-]*):\s*(.*)$', ln)
        if m2:
            cur['fields'][m2.group(1)] = m2.group(2).strip()

print("TOTAL entries:", len(entries))
print(collections.Counter((e['fields'].get('status', '?'), e['checked']) for e in entries))
print()
mode = sys.argv[1] if len(sys.argv) > 1 else 'open'
if mode == 'open':
    for e in entries:
        st = e['fields'].get('status', '?')
        if st != 'done':
            print(f"L{e['line']} [{e['checked']}] {e['id']}")
            for k, v in e['fields'].items():
                print(f"      {k}: {v[:250]}")
            print()
elif mode == 'done-count':
    done = [e for e in entries if e['fields'].get('status') == 'done']
    print("done:", len(done))
    print("last 3 done:")
    for e in done[-3:]:
        print(" ", e['id'], "L%d" % e['line'])
elif mode == 'entry':
    want = sys.argv[2]
    for e in entries:
        if e['id'] == want:
            print(f"L{e['line']} [{e['checked']}] {e['id']}")
            for k, v in e['fields'].items():
                print(f"      {k}: {v}")
