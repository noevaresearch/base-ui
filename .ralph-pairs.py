import re
p = "/data/workspace/baseui/TODO.md"
lines = open(p).read().split("\n")
items = []
cur = None
FIELDS = ["status", "blocked-by", "specs", "crate", "docs-pair", "done-when", "commit", "note", "owner", "crate"]
for i, l in enumerate(lines, 1):
    m = re.match(r"^- \[( |x)\] (.+)$", l)
    if m:
        cur = {"line": i, "title": m.group(2).strip(), "checked": m.group(1) == "x"}
        for f in FIELDS:
            cur[f] = None
        items.append(cur)
    if cur is not None:
        for f in FIELDS:
            mm = re.match(r"^\s*" + re.escape(f) + r":\s*(.*)$", l)
            if mm and cur[f] is None:
                cur[f] = mm.group(1)
byst = {i["title"]: i["status"] for i in items}
print("docs-app shell:", byst.get("docs-app: routing + layout shell"))
cands = []
for i in items:
    t = i["title"]
    if t.startswith("docs-content: components/") and i["status"] != "done":
        comp = t.split("/", 1)[1]
        lib = "library: " + comp
        libst = byst.get(lib)
        cands.append((comp, libst, i["line"]))
print("%-22s %-12s" % ("component", "library"))
for c in cands:
    print("%-22s %-12s L%d" % c)
print()
print("UNBLOCKED:", [c[0] for c in cands if c[1] == "done"])
