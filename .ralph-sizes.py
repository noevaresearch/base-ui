import os, json, glob
base = "/data/workspace/baseui/specs/docs-content"
cands = ['alert-dialog','autocomplete','avatar','checkbox-group','combobox','context-menu','dialog','form','menu','otp-field','popover','preview-card']
rows = []
for c in cands:
    d = os.path.join(base, c)
    page = os.path.join(d, "page.md")
    demo = os.path.join(d, "demos.json")
    ps = os.path.getsize(page) if os.path.exists(page) else -1
    ds = os.path.getsize(demo) if os.path.exists(demo) else -1
    ndemos = "?"
    try:
        j = json.load(open(demo))
        if isinstance(j, list):
            ndemos = len(j)
        else:
            ndemos = "obj"
    except Exception as e:
        ndemos = "ERR"
    # upstream page.mdx size
    up = "/data/workspace/baseui/docs/src/app/(docs)/react/components/%s/page.mdx" % c
    ups = os.path.getsize(up) if os.path.exists(up) else -1
    rows.append((c, ps, ups, ndemos, ds))
rows.sort(key=lambda r: r[2])
print("%-16s %10s %10s %6s %10s" % ("component", "spec.page", "upstream", "demos", "demos.json"))
for r in rows:
    print("%-16s %10d %10d %6s %10d" % r)
