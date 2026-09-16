import os
import re
import glob

roots = ['docs/src/app/(docs)/react/components', 'docs/src/app/(docs)/react/utils', 'docs/src/app/(docs)/react/overview']

def mdx_for(name):
    slug = name.replace('_', '-')
    for root in roots:
        p = os.path.join(root, slug, 'page.mdx')
        if os.path.exists(p):
            return p
    return None

for f in sorted(glob.glob('crates/docs-app/src/pages/*_page.rs')):
    src = open(f).read()
    n_pre = len(re.findall(r'<pre>', src))
    if not n_pre:
        continue
    name = os.path.basename(f)[:-8]
    mdx = mdx_for(name)
    fences = []
    if mdx:
        for line in open(mdx):
            m = re.match(r'^```(\S+)\s*(.*)$', line.rstrip('\n'))
            if m:
                fences.append((m.group(1), m.group(2)))
    print('%-22s pre=%2d mdx=%-8s fences=%2d %s' % (name, n_pre, 'yes' if mdx else 'MISSING', len(fences), os.path.basename(mdx) if mdx else ''))
    for lang, meta in fences:
        print('        %-6s %s' % (lang, meta))
