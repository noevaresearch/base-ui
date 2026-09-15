import sys

src, dst = sys.argv[1], sys.argv[2]
with open(src) as handle:
    body = handle.read()
with open(dst, "a") as handle:
    handle.write(body)
print("appended %d bytes to %s" % (len(body), dst))
