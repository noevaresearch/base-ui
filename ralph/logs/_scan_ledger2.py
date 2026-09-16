import re
from collections import Counter
p = "/data/workspace/baseui/TODO.md"
lines = open(p).read().split("\n")
items = []
cur = None
for i, l in enumerate(lines, 1):
    m = re.match(r'^- \[( |x)\] (.+)$', l)
    if m:
        if cur: items.append(cur)
        cur = {"line": i, "done": m.group(1) == "x", "id": m.group(2).strip(), "fields": {}, "notes": 0}
    elif cur is not None:
        fm = re.match(r'^      ([A-Za-z][\w-]*): ?(.*)$', l)
        if fm:
            k, v = fm.group(1), fm.group(2)
            if k == "note":
                cur["notes"] += 1
                cur.setdefault("notefirst", v[:150])
            else:
                cur["fields"].setdefault(k, v)
if cur: items.append(cur)

print("total items:", len(items), " done:", sum(1 for i in items if i["done"]))
print("status field:", Counter(i["fields"].get("status", "<none>") for i in items))
print()
print("=== NOT done (by status field) ===")
for i in items:
    st = i["fields"].get("status", "?")
    if st != "done":
        print("L%-5d [%-11s] prio=%-6s crate=%-24s notes=%-3d %s" % (
            i["line"], st, i["fields"].get("priority", "-"), i["fields"].get("crate", "-")[:24], i["notes"], i["id"][:95]))
print()
print("=== Phase B (library:) items and status ===")
inB = False
for i in items:
    if i["id"].startswith("Phase B"): inB = True
    if i["id"].startswith("Phase C"): inB = False
    if inB and i["id"].startswith("library:"):
        print("L%-5d [%-11s] %s" % (i["line"], i["fields"].get("status", "-"), i["id"]))
