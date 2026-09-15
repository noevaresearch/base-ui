import re, sys, json
from collections import Counter

p = "/data/workspace/baseui/TODO.md"
lines = open(p).read().split("\n")
items = []
cur = None
for i, l in enumerate(lines, 1):
    m = re.match(r"^- \[( |x)\] (.+?)\s*$", l)
    if m:
        cur = {"line": i, "checked": m.group(1), "id": m.group(2), "fields": {}}
        items.append(cur)
    elif cur is not None:
        fm = re.match(r"^\s+([a-z-]+):\s?(.*)$", l)
        if fm:
            cur["fields"][fm.group(1)] = fm.group(2).strip()

print("total items:", len(items))
print("status/checked:", Counter((i["fields"].get("status"), i["checked"]) for i in items))
print()
blocked = [i for i in items if i["fields"].get("status") == "blocked"]
print("BLOCKED:", json.dumps([{"id": i["id"], "line": i["line"], "note": i["fields"].get("note", "")[:200]} for i in blocked], indent=1))
print()
ns = [i for i in items if i["fields"].get("status") == "not-started"]
print("NOT-STARTED count:", len(ns))
for i in ns:
    print(f'  L{i["line"]:5d} {i["id"]:45s} blocked-by: {i["fields"].get("blocked-by","")[:80]}')
