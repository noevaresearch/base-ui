import fs from 'node:fs';
const STRING_RE = /"([^"\\\n]{1,600})"/g;
const TOKEN_RE = /^[a-z0-9][a-z0-9.:\-/_[\]%#(),]*$/;
const src = fs.readFileSync('/data/workspace/baseui/crates/docs-app/src/pages/button_page.rs', 'utf8');
let found = 0;
const all = new Set();
for (const m of src.matchAll(STRING_RE)) {
  for (const t of m[1].split(/\s+/)) if (t && TOKEN_RE.test(t)) all.add(t);
}
console.log('tokens:', all.size);
console.log('has disabled:border-neutral-500 ->', all.has('disabled:border-neutral-500'));
console.log('has hover variant ->', [...all].filter((t) => t.startsWith('hover:')).join(', '));
// how long is the DEMO_BUTTON_CLASS literal?
const m = src.match(/const DEMO_BUTTON_CLASS: &str = "([^"]*)"/);
console.log('DEMO_BUTTON_CLASS length:', m ? m[1].length : 'NOT MATCHED');
console.log('inside it:', m ? m[1].split(/\s+/).filter((t) => t.includes('disabled')).join(' | ') : '');
