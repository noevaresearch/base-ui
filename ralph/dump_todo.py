import re
p = "/data/workspace/baseui/TODO.md"
lines = open(p).read().split("\n")
items = []
cur = None
for i, l in enumerate(lines, 1):
    m = re.match(r'^- \[( |x)\] (.+?)\s*$', l)
    if m:
        if cur:
            items.append(cur)
        cur = {"line": i, "id": m.group(2), "checked": m.group(1) == "x", "fields": {}}
        continue
    if cur is not None:
        if re.match(r'^#', l) or re.match(r'^[a-zA-Z]', l):
            items.append(cur); cur = None; continue
        fm = re.match(r'^ +([a-z0-9-]+):\s*(.*)$', l)
        if fm:
            k = fm.group(1)
            cur["fields"].setdefault(k, fm.group(2))
if cur:
    items.append(cur)

import sys
want = sys.argv[1] if len(sys.argv) > 1 else None
for it in items:
    f = it["fields"]
    if want and it["id"] != want:
        continue
    print("L%d  %s" % (it["line"], it["id"]))
    for k in ("crate", "specs", "docs-pair", "blocked-by", "status", "commit", "done-when"):
        if k in f:
            print("      %-11s %s" % (k + ":", f[k]))
    print()
if not want:
    print("=== items whose docs-pair target is NOT-started ===")
    st = {it["id"]: it["fields"].get("status") for it in items}
    for it in items:
        dp = it["fields"].get("docs-pair")
        if dp and st.get(dp) == "not-started":
            print("  %-38s (%s)  ->  %s" % (it["id"], it["fields"].get("status"), dp))
