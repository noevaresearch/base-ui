import re
import collections

src = open('/tmp/gen.css', encoding='utf-8').read()
print('contexts:')
for m in re.finditer(r'^(@[^\n{]+) \{', src, re.M):
    pass
# count at-rule preludes at top level of the emitted block
preludes = collections.Counter(re.findall(r'^(@[^\n{]+) \{$', src, re.M))
for k, v in preludes.most_common():
    print('  %3d  %s' % (v, k[:110]))

need = set(re.findall(r'var\(\s*(--[a-z0-9\-]+)', src))
defined = set(re.findall(r'^\s*(--[a-z0-9\-]+):', src, re.M))
print('referenced but NOT defined here:', sorted(need - defined))
print('tw vars used:', sorted(v for v in need if v.startswith('--tw-')))
# which selectors use --tw
for m in re.finditer(r'^([^\n]+) \{((?:[^{}]|\n)*?)\n\}$', src, re.M):
    if '--tw-' in m.group(2):
        print('   uses --tw:', m.group(1)[:80])
