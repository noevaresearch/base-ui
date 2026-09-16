"""Rebuild ralph/generated/visual-baseline.json after this item's --update run.

WHY: `check-visual-budget.mjs --update` writes the measured page score unconditionally, and this
item's change moved the demos' container geometry, so a few routes' blended page score reads 0.01-0.09
LOWER than the recorded best-known value while their COMPONENT widget parity rose from a fault/84% to
100%. The loop's rule is that a recorded number is a floor and a small drop is not something to reset
("never use --update to paper over a drop"), so this script merges instead: the recorded page score
stays the higher of (pre-change, post-change), the recorded widget parity becomes the higher of the
two (the tool itself treats widget parity as a floor — it never erases a measured one), and the route
gets a note naming what moved. The notes the --update run dropped are preserved in git history and
that is stated in the new note, per the precedent already recorded in this file.
"""
import json

HEAD = json.load(open('/tmp/baseline_head.json'))
NOW = json.load(open('/data/workspace/baseui/ralph/generated/visual-baseline.json'))

changed = []
for route, now in NOW['routes'].items():
    head = HEAD['routes'].get(route, {})
    hs, ns = head.get('score'), now.get('score')
    hw, nw = head.get('widgetParity'), now.get('widgetParity')
    score = max([v for v in (hs, ns) if v is not None] or [None])
    widgets = [v for v in (hw, nw) if v is not None]
    widget = max(widgets) if widgets else None
    if (hs is not None and score > ns) or (widget is not None and widget != nw):
        changed.append((route, hs, ns, hw, nw, score, widget))
    now['score'] = score
    now['widgetParity'] = widget
    now['widgetRegionVersion'] = 2 if widget is not None else now.get('widgetRegionVersion')
    # fields that belong to the measurement that owns the recorded page score
    if hs is not None and score == hs and score != ns:
        for k in ('visualProximity', 'contentRecall', 'measuredBuildBytes'):
            if k in head:
                now[k] = head[k]
    now['note'] = (
        "Page score floor restated 2026-09-16 by 'docs-chrome: demo styling (the demos carry upstream's "
        "Tailwind variant, which this app does not compile)': this build measures %.2f (visual %.2f / content %.2f), "
        "the same route measured %.2f before the change, and the difference is this item's demo-container "
        "geometry (the demos' control is now a flex item inside upstream's `.DemoPlaygroundInner` layout "
        "instead of a full-width block). The recorded floor is therefore kept at the HIGHER pre-change value "
        "rather than reset downward — a drop is not something to paper over, and the next iteration must still "
        "beat %.2f. The component widget parity is this item's real result: %s -> %s. Notes recorded here by "
        "earlier iterations were replaced by this run's --update and are preserved verbatim in git history "
        "(HEAD:%s), per this file's existing precedent."
        % (ns, now.get('visualProximity'), now.get('contentRecall'), hs, score,
           'not measurable (region fault)' if hw is None else '%.2f%%' % hw,
           'not measurable (region fault)' if nw is None else '%.2f%%' % nw,
           'ralph/generated/visual-baseline.json')
    )

json.dump(NOW, open('/data/workspace/baseui/ralph/generated/visual-baseline.json', 'w'), indent=1)
print('routes changed:', len(changed))
for c in changed:
    print('  %-34s score %s -> %s (floor %s) | widget %s -> %s (floor %s)'
          % (c[0].split('/')[-1], c[1], c[2], c[5], c[3], c[4], c[6]))
