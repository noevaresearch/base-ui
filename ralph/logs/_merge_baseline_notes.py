"""Write accurate per-route notes into the baseline this item's --update run rewrote.

Pass 2 of the same reconciliation: pass 1 (`_merge_baseline.py`) fixed the NUMBERS (page score floor =
max(pre-change, post-change), widget parity floor = the measured value), but wrote one generic note for
every route. This pass re-derives each note from the actual run log (`/tmp/allboard2.log`) so it states
what that route measured, whether that was up or down against the pre-change floor, and what the
component widget did — no route claims a floor it did not have.
"""
import json
import re

BASE = '/data/workspace/baseui/ralph/generated/visual-baseline.json'
HEAD = json.load(open('/tmp/baseline_head.json'))
now = json.load(open(BASE))

line_re = re.compile(
    r'^(?:ok|UNMEASURABLE|FAIL)\s+(\S+?):\s+score\s+([0-9.]+)\s+\(was\s+([0-9.]+|n/a)'
    r'.*?\|\s+visual\s+([0-9.]+)\s*/\s*content\s+([0-9.]+)'
    r'.*?\|\s+widget\s+([0-9.]+%|n/a)')

measured = {}
for line in open('/tmp/allboard2.log'):
    m = line_re.search(line)
    if m:
        measured[m.group(1)] = {
            'score': float(m.group(2)),
            'visual': float(m.group(4)),
            'content': float(m.group(5)),
            'widget': None if m.group(6) == 'n/a' else float(m.group(6).rstrip('%')),
        }

WHAT = ("This item is what moved it: upstream ships every demo twice and the ported pages carried the "
        "TAILWIND variant's class strings while this crate compiled no Tailwind, so the classes were inert "
        "(the checkbox demo's control laid out 784x36 against upstream's 166x36; the button rendered bare "
        "53x24 text). `ralph/scripts/gen-demo-utilities.mjs` now emits the rules those strings name, copied "
        "from upstream's own compiled stylesheet, plus upstream's `.DemoPlaygroundInner` geometry on the "
        "port's demo wrapper.")

own_notes = {}
for route, entry in now['routes'].items():
    head = HEAD['routes'].get(route, {})
    m = measured.get(route)
    hw, nw = head.get('widgetParity'), entry.get('widgetParity')
    hw_s = 'not measurable (component-region fault)' if hw is None else '%.2f%%' % hw
    nw_s = 'not measurable (component-region fault)' if nw is None else '%.2f%%' % nw
    if m is None:
        own_notes[route] = entry.get('note')
        continue
    head_score = head.get('score')
    if head_score is None:
        verdict = 'first recorded measurement: %.2f' % m['score']
    elif m['score'] > head_score:
        verdict = '%.2f -> %.2f (visual %.2f / content %.2f)' % (head_score, m['score'], m['visual'], m['content'])
    elif m['score'] < head_score:
        verdict = ('this build measures %.2f (visual %.2f / content %.2f), %.2f BELOW the pre-change floor '
                   '%.2f — the floor is kept at %.2f and NOT reset downward: the dip is this item\'s '
                   'demo-container geometry moving pixels around a demo whose surrounding panel is still a '
                   'separate item (`docs-chrome: demo panels`), so it is a fact about the change, not a number '
                   'to paper over.' % (m['score'], m['visual'], m['content'], head_score - m['score'],
                                       head_score, head_score))
    else:
        verdict = 'unchanged at %.2f (visual %.2f / content %.2f)' % (m['score'], m['visual'], m['content'])
    own_notes[route] = (
        "Re-measured 2026-09-16 by 'docs-chrome: demo styling (the demos carry upstream's Tailwind variant, "
        "which this app does not compile)': page %s; component widget %s -> %s. %s The notes earlier "
        "iterations recorded here were replaced by this run's --update and are preserved verbatim in git "
        "history (the pre-change file is HEAD:ralph/generated/visual-baseline.json), per this file's existing "
        "precedent that a superseded note lives in git." % (verdict, hw_s, nw_s, WHAT))

for route, note in own_notes.items():
    if note:
        now['routes'][route]['note'] = note

now['note'] = now['note'] + (
    " Entries re-measured 2026-09-16 by 'docs-chrome: demo styling': the component widget parity rose on "
    "every route that had one to raise (button 84.42 -> 100, toggle 94.66 -> 100, otp-field 93.84 -> 100, "
    "field 87.8 -> 98.79, form 86.99 -> 97.33, fieldset 86.6 -> 96.79, collapsible 88.5 -> 93.35, "
    "accordion 89.1 -> 90.01) and became MEASURABLE for the first time on checkbox, meter and "
    "checkbox-group (100 each; those three regions were reported as faults before, because the demos' "
    "controls were full-width blocks). Six routes' blended PAGE score read 0.01-0.09 below their "
    "pre-change floor on the same run and were kept at the higher pre-change value rather than reset "
    "(a drop is not a number to paper over): meter, fieldset, separator, toggle, use-render, "
    "direction-provider.")

json.dump(now, open(BASE, 'w'), indent=1)
print('notes written:', sum(1 for n in own_notes.values() if n))
for r, e in now['routes'].items():
    h = HEAD['routes'].get(r, {})
    print('  %-34s score %s (head %s) | widget %s (head %s)' % (r.split('/')[-1], e['score'], h.get('score'), e.get('widgetParity'), h.get('widgetParity')))
