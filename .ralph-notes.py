import re
p = "/data/workspace/baseui/TODO.md"
lines = open(p).read().split("\n")
items = []
cur = None
FIELDS = ["status", "blocked-by", "specs", "crate", "docs-pair", "done-when", "commit", "note", "owner"]
for i, l in enumerate(lines, 1):
    m = re.match(r"^- \[( |x)\] (.+)$", l)
    if m:
        cur = {"line": i, "title": m.group(2).strip(), "fields": {}}
        items.append(cur)
    if cur is not None:
        for f in FIELDS:
            mm = re.match(r"^\s*" + re.escape(f) + r":\s*(.*)$", l)
            if mm and f not in cur["fields"]:
                cur["fields"][f] = mm.group(1)
cands = ['alert-dialog','autocomplete','checkbox-group','combobox','context-menu','dialog','form','menu','otp-field','popover','preview-card']
for it in items:
    t = it["title"]
    if t.startswith("docs-content: components/"):
        c = t.split("/", 1)[1]
        if c in cands:
            n = it["fields"].get("note", "")
            print("=" * 70)
            print("%s (L%d) status=%s" % (t, it["line"], it["fields"].get("status")))
            print("NOTE:", n[:600] if n else "(none)")
            print()
