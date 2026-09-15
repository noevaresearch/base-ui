import re
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

by_id = {i["id"]: i for i in items}
st = lambda i: i["fields"].get("status")


def parse_deps(s):
    inner = s.strip()
    if inner.startswith("["):
        inner = inner[1:inner.rindex("]")] if "]" in inner else inner[1:]
    return [d.strip() for d in inner.split(",") if d.strip()]


print("=== DONE library items that still have a docs-content pair (with pair status) ===")
for i in items:
    if not i["id"].startswith("library:"):
        continue
    pair = i["fields"].get("docs-pair", "")
    if not pair:
        continue
    pi = by_id.get(pair)
    print(f'  {i["id"]:34s} {st(i):12s} pair={pair:40s} pair_status={st(pi) if pi else "MISSING":12s} exempt={i["fields"].get("exempt-from-docs-pairing","")}')

print()
print("=== docs-content: items and whether their owner is done ===")
for i in items:
    if not i["id"].startswith("docs-content:"):
        continue
    deps = parse_deps(i["fields"].get("blocked-by", ""))
    depstat = [(d, st(by_id[d]) if d in by_id else "MISSING") for d in deps]
    unblocked = all(s == "done" for _, s in depstat)
    flag = "UNBLOCKED" if unblocked else "blocked   "
    print(f'  L{i["line"]:5d} {flag} {st(i):12s} {i["id"]:42s} deps={depstat}')

print()
print("=== docs-app shell item ===")
for i in items:
    if "docs-app" in i["id"] or i["id"].startswith("infra:"):
        print(f'  L{i["line"]:5d} {st(i):12s} {i["id"]}')
