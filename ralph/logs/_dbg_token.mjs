// Debug: is the token `disabled:border-neutral-500` indexed by the generator's parser?
import fs from 'node:fs';
const src = fs.readFileSync('/tmp/upa.css', 'utf8');
const NE = /^\.disabled\\:border-neutral-500/;
function parseDeclarations(src) {
  src = src.replace(/\/\*[\s\S]*?\*\//g, '');
  const out = []; const stack = []; let buf = ''; let i = 0;
  while (i < src.length) {
    const c = src[i];
    if (c === '{') {
      const prelude = buf.trim(); buf = '';
      const atRule = prelude.startsWith('@');
      const nestable = /^@(media|supports|layer|container|scope|starting-style)\b/.test(prelude);
      if (atRule && !nestable) {
        let depth = 1; let j = i + 1;
        while (j < src.length && depth > 0) { if (src[j] === '{') depth += 1; else if (src[j] === '}') depth -= 1; j += 1; }
        out.push({ stack: [...stack], block: { prelude, body: src.slice(i + 1, j - 1) } }); i = j; buf = ''; continue;
      }
      stack.push(prelude);
    } else if (c === '}') { buf = ''; if (stack.length) stack.pop(); }
    else if (c === ';') { if (buf.trim() && stack.length) out.push({ stack: [...stack], decl: buf.trim() }); buf = ''; }
    else buf += c;
    i += 1;
  }
  return out;
}
const entries = parseDeclarations(src);
console.log('entries', entries.length);
const hits = entries.filter((e) => e.decl && e.stack[e.stack.length - 1] && NE.test(e.stack[e.stack.length - 1]));
console.log('hits', hits.length);
for (const h of hits.slice(0, 4)) console.log(JSON.stringify(h));
// now the class-token extraction on that selector
const sel = hits.length ? hits[0].stack[hits[0].stack.length - 1] : null;
if (sel) {
  console.log('sel raw:', JSON.stringify(sel));
  console.log('tokens:', [...sel.matchAll(/\.((?:\\.|[A-Za-z0-9_-])+)/g)].map((m) => m[1]).map((s) => JSON.stringify(s)).join(' | '));
}
