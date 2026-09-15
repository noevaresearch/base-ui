import json, os, sys

base = "/data/workspace/baseui/specs/docs-content"
for u in ["form", "context-menu", "otp-field", "alert-dialog", "popover", "menu", "preview-card"]:
    p = os.path.join(base, u, "demos.json")
    try:
        d = json.load(open(p))
    except Exception as e:
        print(f"{u:14s} PARSE ERROR {e}")
        continue
    if isinstance(d, dict):
        demos = d.get("demos") or d.get("items") or []
        kind = "dict:" + ",".join(list(d.keys())[:6])
    else:
        demos = d
        kind = "list"
    names = []
    for x in demos if isinstance(demos, list) else []:
        if isinstance(x, dict):
            names.append(x.get("name") or x.get("id") or x.get("title") or "?")
    print(f"{u:14s} demos={len(demos):2d}  [{kind}]  {names}")
