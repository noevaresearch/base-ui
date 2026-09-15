import re, sys
from collections import Counter
p = "/data/workspace/baseui/TODO.md"
lines = open(p).read().split("\n")
items = []
cur = None
FIELDS = ["status", "blocked-by", "specs", "crate", "docs-pair", "done-when", "commit", "note", "phase", "chosen"]
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
print("total items:", len(items))
print(Counter((i["status"] or ("DONE-unchk" if not i["checked"] else "?")) for i in items))
mode = sys.argv[1] if len(sys.argv) > 1 else "open"
print("=== %s ===" % mode)
for i in items:
    if mode == "open" and i["status"] == "done":
        continue
    print("%5d | %-42s | checked=%s | %-11s | bb=%s | crate=%s | dp=%s" % (
        i["line"], i["title"][:42], i["checked"], i["status"], i["blocked-by"], i["crate"], i["docs-pair"]))
