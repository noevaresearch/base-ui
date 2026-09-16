import json
d = json.load(open('ralph/generated/visual-baseline.json'))
for r, v in d['routes'].items():
    print('%-32s score=%-7s widget=%-7s vis=%-7s content=%-7s' % (
        r, v.get('score'), v.get('widgetParity'), v.get('visualProximity'), v.get('contentRecall')))
print('routes:', len(d['routes']))
