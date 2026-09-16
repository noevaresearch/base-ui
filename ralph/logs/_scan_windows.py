import json, os, re, sys

repo = "/data/workspace/baseui"
keys = []
for root, dirs, files in os.walk(os.path.join(repo, "specs")):
    for f in files:
        if not f.endswith(".citations.json"):
            continue
        p = os.path.join(root, f)
        try:
            data = json.load(open(p))
        except Exception as e:
            print("skip", p, e); continue
        def walk(obj, trail):
            if isinstance(obj, dict):
                for k, v in obj.items():
                    if k.startswith("TODO.md:"):
                        keys.append((k, p))
                    walk(v, trail + [str(k)])
            elif isinstance(obj, list):
                for v in obj:
                    walk(v, trail)
        walk(data, [])

print("total TODO.md citation keys:", len(keys))
lo_hi = []
for k, p in keys:
    m = re.match(r"TODO\.md:(\d+)-(\d+)$", k)
    if not m:
        print("UNNORMALISED KEY", k, p); continue
    lo, hi = int(m.group(1)), int(m.group(2))
    if lo - 3 <= 840 and hi + 3 >= 820:
        lo_hi.append((lo, hi, p))
print("windows overlapping 820-840 (+-3):", len(lo_hi))
for lo, hi, p in sorted(lo_hi):
    print("  ", lo, hi, p.replace(repo + "/", ""))
print("max hi:", max(int(re.match(r"TODO\.md:(\d+)-(\d+)$", k).group(2)) for k, _ in keys if re.match(r"TODO\.md:(\d+)-(\d+)$", k)))
