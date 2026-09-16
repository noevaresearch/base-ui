import json, os
for r in ['button', 'checkbox', 'meter', 'accordion', 'avatar', 'checkbox-group']:
    p = 'ralph/logs/visual/%s.json' % r
    if not os.path.exists(p):
        print('==', r, 'MISSING'); continue
    d = json.load(open(p))
    print('==', r)
    for k, v in d.items():
        if isinstance(v, (dict, list)):
            print('   ', k, json.dumps(v)[:400])
        else:
            print('   ', k, '=', v)
