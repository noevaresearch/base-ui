#!/usr/bin/env node
/* eslint-disable no-console */

// check-part-surface.mjs — the systematic gate for the port's ERGONOMIC claim, over every spec.
//
// THE CLAIM THIS GATES
// --------------------
// "Base UI's API shape survives the migration": upstream teaches `<Checkbox.Root><Checkbox.Indicator
// /></Checkbox.Root>`, so the port must expose `<Checkbox::Root><Checkbox::Indicator /></Checkbox::Root>`
// — the same parts, in a module named after the component, spelled with Rust's path separator. The
// ergonomics *is* the product; a flattened `checkbox_root_view(CheckboxRootViewProps { .. })` is a
// different library with the same behaviour.
//
// WHY IT WALKS THE SPECS RATHER THAN ONE PAGE
// -------------------------------------------
// The per-route ergonomics probe (snippet-ergonomics.mjs) measures what a *rendered page* shows,
// which catches drift late and only where a page exists. The parts each component owes are already
// written down in the mined specs, so the same question can be asked of the whole repo at once:
// for every `Component.Part` the specs document, does the crate expose `Component::Part`?
//
// That is this script's whole job, and it is deliberately source-level (no compiler needed) so it
// can run in the gate next to the citation and schema checks.
//
// HOW IT DECIDES
// --------------
//   * upstream part names come from the specs: `A.B` dotted references in specs/library/<name>/…
//     (the mined behaviour/implementation specs are the citation-backed record of what upstream has)
//   * a counterpart exists when the crate exposes, inside a module named after the component, a
//     PUBLIC capitalised item with that part name — `pub fn Root`, `pub struct Root`, or a re-export
//   * "flattened-only" is reported separately: the part exists but at the crate root (or as a
//     lowercase `*_view` helper), which is behaviour without the ergonomics
//
// USAGE
//   node ralph/scripts/check-part-surface.mjs                 # report
//   node ralph/scripts/check-part-surface.mjs --strict        # exit 1 if any documented part is missing
//   node ralph/scripts/check-part-surface.mjs --require 50    # exit 1 below 50% coverage
//   node ralph/scripts/check-part-surface.mjs --component checkbox
//
// An LSP would surface this per-file in an editor; a gate needs it repo-wide and deterministic,
// which is why it is a script. See the header of snippet-ergonomics.mjs for the rendered-page half.

import fs from 'node:fs';
import path from 'node:path';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const SPECS_LIB = path.join(PROJECT_ROOT, 'specs/library');
const CRATE_SRC = path.join(PROJECT_ROOT, 'crates/leptos-ui/src');

function arg(name, dflt) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] && !process.argv[i + 1].startsWith('--') ? process.argv[i + 1] : dflt;
}
const STRICT = process.argv.includes('--strict');
const REQUIRE = arg('require', null) === null ? null : Number(arg('require'));
const ONLY = arg('component', null);

const snake = (s) => s.replace(/([a-z0-9])([A-Z])/g, '$1_$2').replace(/[\s-]/g, '_').toLowerCase();

/** Every `Component.Part` reference the component's specs make (the mined record of upstream's API). */
function documentedParts(component) {
  const dir = path.join(SPECS_LIB, component);
  let files = [];
  try {
    files = fs.readdirSync(dir).filter((f) => f.endsWith('.md')).map((f) => path.join(dir, f));
  } catch {
    return new Map();
  }
  const parts = new Map(); // "Checkbox.Root" -> [files]
  for (const f of files) {
    let text;
    try { text = fs.readFileSync(f, 'utf8'); } catch { continue; }
    // dotted upstream references: Checkbox.Root, Field.Label, Popover.Positioner …
    const re = /\b([A-Z][A-Za-z0-9]*)\.([A-Z][A-Za-z0-9]*)\b/g;
    let m;
    while ((m = re.exec(text)) !== null) {
      const [, comp, part] = m;
      // ignore React-API noise like React.Children / Object.keys style references in prose
      if (/^(React|Object|Array|JSON|Math|Date|Promise|Type|PropTypes)$/.test(comp)) continue;
      if (!parts.has(`${comp}.${part}`)) parts.set(`${comp}.${part}`, []);
      parts.get(`${comp}.${part}`).push(path.relative(PROJECT_ROOT, f));
    }
  }
  return parts;
}

/** Public items the crate exposes, indexed by module. */
function crateSurface() {
  const modules = new Map(); // module -> Set(item names)
  const root = new Set();
  const moduleDirs = [];
  try {
    for (const entry of fs.readdirSync(CRATE_SRC, { withFileTypes: true })) {
      if (entry.isDirectory()) moduleDirs.push(entry.name);
    }
  } catch {
    return { modules, root };
  }
  const collect = (file, into, intoRoot) => {
    let text;
    try { text = fs.readFileSync(file, 'utf8'); } catch { return; }
    const re = /pub\s+(?:fn|struct|enum|type|use)\s+([A-Za-z_][A-Za-z0-9_]*)/g;
    let m;
    while ((m = re.exec(text)) !== null) {
      (intoRoot ? root : into).add(m[1]);
    }
  };
  for (const dir of moduleDirs) {
    const set = new Set();
    const modFile = path.join(CRATE_SRC, dir, 'mod.rs');
    if (fs.existsSync(modFile)) collect(modFile, set, false);
    try {
      for (const f of fs.readdirSync(path.join(CRATE_SRC, dir))) {
        if (f.endsWith('.rs')) collect(path.join(CRATE_SRC, dir, f), set, false);
      }
    } catch {}
    modules.set(dir, set);
  }
  const libRoot = path.join(CRATE_SRC, 'lib.rs');
  if (fs.existsSync(libRoot)) collect(libRoot, new Set(), true);
  return { modules, root };
}

function main() {
  const { modules, root } = crateSurface();
  let componentDirs = [];
  try {
    componentDirs = fs.readdirSync(SPECS_LIB, { withFileTypes: true })
      .filter((e) => e.isDirectory()).map((e) => e.name).sort();
  } catch {
    console.error(`FAIL: ${path.relative(PROJECT_ROOT, SPECS_LIB)} not found`);
    return 2;
  }
  if (ONLY) componentDirs = componentDirs.filter((c) => c === ONLY);

  const rows = [];
  for (const component of componentDirs) {
    const parts = documentedParts(component);
    if (!parts.size) continue;
    const modName = snake(component);
    const modItems = modules.get(modName) || new Set();
    // a part may also be declared in a differently-named module (e.g. internals) — accept any module
    const anyModule = new Set([...modules.values()].flatMap((s) => [...s]));
    const per = [];
    for (const key of [...parts.keys()].sort()) {
      const [comp, part] = key.split('.');
      if (snake(comp) !== modName) continue; // a reference to a *different* component's part
      const namespaced = modItems.has(part);
      const elsewhere = !namespaced && anyModule.has(part);
      // Flattened = the part's behaviour exists but not under the component's own namespace: the
      // crate's convention is `<component>_<part>_view(..)`, e.g. checkbox_root_view for
      // Checkbox.Root — behaviour without the ergonomics.
      const flatNames = [`${snake(part)}_view`, `${modName}_${snake(part)}_view`];
      const flattened = !namespaced && !elsewhere &&
        (root.has(part) || flatNames.some((n) => modItems.has(n) || anyModule.has(n)));
      per.push({ key, part, namespaced, elsewhere, flattened, citations: parts.get(key) });
    }
    if (!per.length) continue;
    rows.push({ component, module: modName, parts: per,
      namespaced: per.filter((p) => p.namespaced).length,
      total: per.length });
  }

  const totalParts = rows.reduce((n, r) => n + r.total, 0);
  const namespaced = rows.reduce((n, r) => n + r.namespaced, 0);
  const pct = totalParts ? (namespaced / totalParts) * 100 : 100;

  const flattenedTotal = rows.reduce((n, r) => n + r.parts.filter((p) => p.flattened).length, 0);
  console.log(`part surface: ${namespaced}/${totalParts} documented upstream parts exposed as Component::Part ` +
    `(${pct.toFixed(1)}%), across ${rows.length} component(s) with mined specs` +
    `${flattenedTotal ? ` — ${flattenedTotal} exist only in the flattened form (behaviour without the ergonomics)` : ''}`);

  for (const r of rows) {
    const missing = r.parts.filter((p) => !p.namespaced);
    if (!missing.length) { console.log(`  OK   ${r.component}: ${r.namespaced}/${r.total}`); continue; }
    const named = missing.map((p) => `${r.module}::${p.part}`).slice(0, 6).join(', ');
    const flat = missing.filter((p) => p.flattened).length;
    console.log(`  ${r.namespaced ? 'PARTIAL' : 'MISSING'} ${r.component}: ${r.namespaced}/${r.total} — missing ${named}${missing.length > 6 ? ', …' : ''}` +
      `${flat ? ` (${flat} exist only in the flattened form)` : ''}`);
  }

  const failed = (STRICT && namespaced < totalParts) || (REQUIRE !== null && pct < REQUIRE);
  if (failed) {
    console.error(`\nFAIL: ${STRICT ? '--strict' : `--require ${REQUIRE}`} — ${totalParts - namespaced} documented part(s) are not exposed as Component::Part.`);
    console.error('See specs/docs-content/CONTRACT.md (the React A.B -> Rust A::B mapping) and the ledger item');
    console.error('"library: namespaced part surface (Checkbox::Root form)" — the surface the docs examples must teach.');
    return 1;
  }
  console.log(STRICT || REQUIRE !== null ? '\npart surface OK' : '\n(advisory — pass --strict or --require <pct> to gate)');
  return 0;
}

process.exit(main());
