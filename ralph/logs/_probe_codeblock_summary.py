import json, collections, sys
up = json.load(open('ralph/logs/visual/_probe_codeblock_upstream.json'))
lx = json.load(open('ralph/logs/visual/_probe_codeblock_leptos.json'))
print('UP preCount', up['preCount'], 'embedded', up['embeddedCount'])
print('LX preCount', lx['preCount'], 'embedded', lx['embeddedCount'])

# class -> colour map across all upstream blocks
m = collections.defaultdict(collections.Counter)
for b in up['blocks']:
    for s in b['sample']:
        m[s['cls']][s['color']] += 1
print('\nupstream class->colour (sampled):')
for k, v in sorted(m.items()):
    print('  %-30s %s' % (k, dict(v)))

# an embedded (non-demo) block on each side
for label, d in (('UP', up), ('LX', lx)):
    for b in d['blocks']:
        if b['inDemo']:
            continue
        print('\n===== %s block %d chain=%s' % (label, b['i'], b['chain']))
        print('   childTags', b['childTags'])
        print('   pre', json.dumps(b['pre']))
        print('   toks', b['tokTotal'], 'coloured', b['colouredTotal'], json.dumps(b['palette']))
        print('   sample', json.dumps(b['sample'])[:600])
        print('   inner:', b['inner'][:900].replace('\\n', ' '))
        break
