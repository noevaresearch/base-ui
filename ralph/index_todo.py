import re
from collections import Counter
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
            if k not in cur["fields"]:
                cur["fields"][k] = fm.group(2)
if cur:
    items.append(cur)
print("total items:", len(items))
print(Counter(it["fields"].get("status") for it in items))
print("checked flags:", Counter(it["checked"] for it in items))
print()
print("=== NOT-STARTED ===")
for it in items:
    if it["fields"].get("status") == "not-started":
        print('L%-5d %-45s crate=%-18s pair=%s' % (
            it["line"], it["id"], it["fields"].get("crate", ""), it["fields"].get("docs-pair", "-")))
print()
print("=== DOCS-CONTENT (Phase D) ===")
for it in items:
    if it["id"].startswith("docs-content"):
        print('L%-5d %-45s %-12s blocked-by=%s' % (
            it["line"], it["id"], it["fields"].get("status"), (it["fields"].get("blocked-by", "-"))[:90]))
