"""Append this iteration's spec findings to ralph/logs/spec-discrepancies.md."""
import io

TEXT = '''
## `docs-chrome: demo styling` — two things the page specs do not say about the demos, found by making them render

Recorded while implementing `docs-chrome: demo styling (the demos carry upstream's Tailwind variant,
which this app does not compile)`. Both are SPEC gaps (the mirrored pages' `page.md` demo sections do
not state them) and both were found by following the citations into upstream, not by guessing:

1. **One class in the tailwind demos has no rule in upstream's OWN compiled stylesheet.**
   `font-inherit` appears in the tailwind sources (`docs/src/app/(docs)/react/components/otp-field/
   demos/hero/tailwind/index.tsx:24`, and the tabs demos) but upstream's served docs CSS defines no
   `.font-inherit` at all (measured: 0 hits in both stylesheet chunks the docs site serves). The
   subsection above ("the mirrored demos carry upstream's TAILWIND variant …") says the oracle for a
   demo class is the RULES rather than the strings; this is the case where the two variants disagree
   about the class itself — the css-modules variant of the same demo carries the declaration under a
   different name (`.Input { font-family: inherit; … }`, `.../otp-field/demos/hero/css-modules/
   index.module.css`). So the oracle for that one class is the css-modules file. The mirrored page
   specs say nothing about it, and the port's stylesheet now carries the rule by hand
   (`crates/docs-app/style/main.css`, the `.font-inherit` block, with its provenance comment).

2. **The demos' CONTAINER geometry is part of the demo's appearance, and no page spec mentions it.**
   Upstream wraps every demo four deep — `div.demo > div.DemoRoot > div.DemoPlayground >
   div.DemoPlaygroundInner` (`docs/src/components/Demo/Demo.css`) — and the innermost one is a
   centering flex container: `padding: 2rem 1.5rem; min-height: 8rem; min-width: fit-content;
   display: flex; justify-content: center; align-items: center`. The port renders each demo in a
   single `.docs-demo` div that had NO rule, so the demo's control laid out across the whole article:
   measured on `/react/components/checkbox`, upstream's `<label>Enable notifications</label>` is
   166x36 while the port's was 784x36. That is not only a visual difference — it made
   `check-visual-budget.mjs` report the component region as NOT COMPARABLE on that route and on meter
   ("upstream 166x36 vs leptos 784x36 … not the same kind of thing"), so the Phase E widget bar could
   not be measured there at all. Implication carried forward for `docs-chrome: demo panels`: when the
   panel/border/file-tabs wrapper lands, the centering-flex geometry has to stay on the INNER element
   (upstream's `.DemoPlaygroundInner`), because that is the element `ralph/scripts/lib/
   widget-region.mjs` measures the component inside; putting the padding on a new outer wrapper and
   collapsing the inner one re-breaks the measurement the same way.
'''

path = '/data/workspace/baseui/ralph/logs/spec-discrepancies.md'
with io.open(path, 'a', encoding='utf-8') as fh:
    fh.write(TEXT)
print('appended', len(TEXT), 'chars to', path)
