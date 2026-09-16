import glob
import os
import re

for f in sorted(glob.glob('crates/docs-app/src/pages/*_page.rs')):
    src = open(f).read().split('\n')
    hits = [(i, l) for i, l in enumerate(src) if '<pre>' in l]
    if not hits:
        continue
    print('=====', os.path.basename(f))
    for i, l in hits:
        print('  %4d: %s' % (i + 1, l.strip()[:120]))
        # show the following line for the inline form
        if '<code>' == l.strip()[-7:] or l.rstrip().endswith('<code>'):
            print('        next: %s' % src[i + 1].strip()[:110])
            print('        then: %s' % src[i + 2].strip()[:60])
