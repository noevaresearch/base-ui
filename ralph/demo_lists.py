import json, sys
for f in sys.argv[1:]:
    print("######## " + f)
    d = json.load(open("specs/docs-content/%s/demos.json" % f))
    if isinstance(d, dict):
        print("   (object, not array):", list(d.keys()))
        continue
    for e in d:
        print("-", e.get("name"), "| hero" if e.get("isHero") else "", "| parts:", ", ".join(e.get("componentPartsUsed", [])))
        print("   ", (e.get("whatItDemonstrates") or "")[:160])
