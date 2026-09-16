// Verify the naming-parity rule with synthetic trees (no browser needed): a namespaced path must
// count as upstream's dotted component's counterpart, and an unrelated short name must not match.
import { compareTrees, canonName } from './lib/ast-compare.mjs';

const el = (name, depth = 0) => ({ name, canon: canonName(name), depth, selfClosing: true, attrs: 0 });

const cases = [
  {
    label: 'upstream Checkbox.Root/Indicator vs port <CheckboxRoot>/<CheckboxIndicator>',
    up: [el('Checkbox.Root'), el('Checkbox.Indicator')],
    lx: [el('CheckboxRoot'), el('CheckboxIndicator')],
    want: 1,
  },
  {
    label: 'upstream Checkbox.Root/Indicator vs port namespaced <ui::Checkbox::Root>/<ui::Checkbox::Indicator>',
    up: [el('Checkbox.Root'), el('Checkbox.Indicator')],
    lx: [el('ui::Checkbox::Root'), el('ui::Checkbox::Indicator')],
    want: 1,
  },
  {
    label: 'upstream Field.Label vs port plain <label> (must NOT count as a match)',
    up: [el('Field.Label')],
    lx: [el('label')],
    want: 0,
  },
  {
    label: 'port offers nothing component-shaped (function calls only)',
    up: [el('Checkbox.Root')],
    lx: [],
    want: 0,
  },
];

let ok = true;
for (const c of cases) {
  const r = compareTrees(c.up, c.lx);
  const pass = Math.abs(r.naming - c.want) < 1e-9;
  ok = ok && pass;
  console.log(`${pass ? 'PASS' : 'FAIL'} | ${c.label} → naming ${(r.naming * 100).toFixed(0)}% (want ${c.want * 100}%)` +
    ` | missing: ${r.missingComponents.join(', ') || 'none'}`);
}
process.exit(ok ? 0 : 1);
