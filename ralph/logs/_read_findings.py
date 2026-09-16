import json
for r in ['button', 'checkbox', 'meter']:
    d = json.load(open('ralph/logs/visual/%s.json' % r))
    print('=====', r)
    for f in d.get('findings', []):
        print('  [%s] %s\n      gap: %s\n      fix: %s' % (f.get('severity'), f.get('area'), f.get('gap'), f.get('fix')))
    u = d.get('upstream', {}); l = d.get('leptos', {})
    for k in ('propsTables', 'tables', 'snippets', 'codeTokens', 'codeBlocks'):
        if k in u or k in l:
            print('   KEY', k, 'upstream=', json.dumps(u.get(k))[:200], 'leptos=', json.dumps(l.get(k))[:200])
    print('   upstream keys:', list(u.keys()))
    print('   leptos keys:', list(l.keys()))
