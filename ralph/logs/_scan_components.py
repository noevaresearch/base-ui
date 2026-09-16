import re, subprocess, os
from collections import Counter
p = "/data/workspace/baseui/TODO.md"
lines = open(p).read().split("\n")
items = []
cur = None
for i, l in enumerate(lines, 1):
    m = re.match(r'^- \[( |x)\] (.+)$', l)
    if m:
        if cur: items.append(cur)
        cur = {"line": i, "done": m.group(1) == "x", "id": m.group(2).strip(), "fields": {}}
    elif cur is not None:
        fm = re.match(r'^      ([A-Za-z][\w-]*): ?(.*)$', l)
        if fm:
            k, v = fm.group(1), fm.group(2)
            if k != "note":
                cur["fields"].setdefault(k, v)
if cur: items.append(cur)

lib = [i for i in items if i["id"].startswith("library:") and not i["id"].startswith("library: namespaced")]
open_ = [i for i in lib if i["fields"].get("status", "?") != "done"]
print("library: items total", len(lib), "open", len(open_))
print("exempt-from-docs-pairing on open library items:",
      [i["id"] for i in open_ if "exempt-from-docs-pairing" in i["fields"]])
print()
os.chdir("/data/workspace/baseui")
print("%-26s %6s %7s %7s %7s %6s %s" % ("component", "files", "srcLOC", "tstLOC", "testfil", "mining", "line"))
rows = []
for i in open_:
    name = i["id"].split(": ", 1)[1].strip()
    d = "packages/react/src/%s" % name
    if not os.path.isdir(d):
        rows.append((name, -1, -1, -1, -1, "NO DIR", i["line"])); continue
    src = tst = 0
    tstfiles = 0; srcfiles = 0
    for root, _, files in os.walk(d):
        for f in files:
            if not f.endswith((".ts", ".tsx")):
                continue
            n = sum(1 for _ in open(os.path.join(root, f), errors="ignore"))
            if ".test." in f or "/test/" in root:
                tst += n; tstfiles += 1
            else:
                src += n; srcfiles += 1
    rows.append((name, srcfiles, src, tst, tstfiles, "yes" if "needs-batched-mining" in i["fields"] else "-", i["line"]))
for r in sorted(rows, key=lambda r: r[2]):
    print("%-26s %6s %7s %7s %7s %6s L%s" % r)
