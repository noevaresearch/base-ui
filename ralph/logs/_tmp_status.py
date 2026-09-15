import re, sys
from collections import Counter
p="/data/workspace/baseui/TODO.md"
lines=open(p).read().split("\n")
items=[]; cur=None
for i,l in enumerate(lines):
    if l.startswith("- ["):
        if cur: items.append(cur)
        cur={"line":i+1,"header":l.strip(),"body":[]}
    elif cur is not None:
        cur["body"].append(l)
if cur: items.append(cur)
for it in items:
    b="\n".join(it["body"])
    it["status"]=(re.search(r"status:\s*(\S+)",b) or [None,"?"])[1] if re.search(r"status:\s*(\S+)",b) else "?"
    it["crate"]=(re.search(r"crate:\s*(\S+)",b).group(1) if re.search(r"crate:\s*(\S+)",b) else "?")
    it["docs_pair"]=(re.search(r"docs-pair:\s*(\S+)",b).group(1) if re.search(r"docs-pair:\s*(\S+)",b) else "-")
    it["blocked_by"]=(re.search(r"blocked-by:\s*(.+)",b).group(1).strip() if re.search(r"blocked-by:\s*(.+)",b) else "-")
    it["wraps"]=(re.search(r"wraps-external:\s*(.+)",b).group(1).strip() if re.search(r"wraps-external:\s*(.+)",b) else "-")
    it["id"]=it["header"].split("]",1)[1].strip() if "]" in it["header"] else it["header"]
    it["checked"]=it["header"].startswith("- [x]")
print("TOTAL:",len(items), Counter(it["status"] for it in items))
print("checked [x]:",sum(1 for it in items if it["checked"]),"unchecked:",sum(1 for it in items if not it["checked"]))
print()
for it in items:
    if it["status"]!="done" or not it["checked"]:
        print(f"L{it['line']:5d} chk={'x' if it['checked'] else ' '} [{it['status']:12s}] {it['id']:42s} crate={it['crate']:22s} docs-pair={it['docs_pair']:38s} blocked-by={it['blocked_by'][:60]}")
