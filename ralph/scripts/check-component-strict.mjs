#!/usr/bin/env node
/* eslint-disable no-console */

// check-component-strict.mjs — strict, SPEC-BASED feedback for the component cycle.
//
// WHY THIS EXISTS
// ---------------
// A `library:` item's only real gate was `cargo test --workspace`. That is necessary and nearly useless as
// feedback: it says whether the suite passes, not whether THIS component is finished. Mid-cycle, the loop
// could ship `Checkbox::Root` with two of its five proven props, no namespaced parts, and no test for the
// disabled state — and the regression would be green, because nothing compared the port against the
// behaviour spec that was mined for exactly this purpose. The loop then learns nothing per component; it
// learns something only when a docs page later fails.
//
// So this gate compares ONE component against `specs/library/<name>/behavior.md`, which is the spec mined
// from upstream's own tests, and names every gap:
//
//   parts     — every dotted part the spec proves (`Checkbox.Root`, `Checkbox.Indicator`) must exist in the
//               port as a NAMESPACED component usable in `view!` markup (delegated to check-part-surface,
//               which is the authority on that count).
//   props     — every prop the spec says is "proven by tests" must exist on the port's matching part
//               struct (`defaultChecked` -> `default_checked`). This is the axis that catches a component
//               that renders correctly but quietly ignores half its documented API.
//   sections  — every section of the spec (`## State model`, `## Keyboard interaction`, …) must have at
//               least one test in the component's test module touching its vocabulary. An unmentioned
//               section is an obligation nobody is proving.
//   hygiene   — no `#[ignore]`d tests (a disabled test is not evidence), and at least as many tests as
//               sections (a floor, not a quality bar).
//
// Feedback is deliberately NAMED and cited, so an iteration can act without re-deriving anything: each gap
// prints the spec line it came from.
//
// USAGE
//   node ralph/scripts/check-component-strict.mjs --component checkbox
//   node ralph/scripts/check-component-strict.mjs --todo-id "library: namespaced part surface (ported batch)"
//   node ralph/scripts/check-component-strict.mjs --component checkbox --strict    # exit 1 on any gap
//
// WHICH AXES ARE HARD, AND FOR WHOM
// ---------------------------------
// All four axes are hard for a `library: <component>` item (`--strict`): that item is the component's
// port, so an unproven section or a missing documented prop is its own unfinished business.
//
// For a `library: namespaced part surface …` BATCH the hard axes are `parts` and `namespaced path`:
// `parts` is the surface itself, and `namespaced path` is the other half of that batch's done-when —
// "one part-surface test per module exercising the namespaced path", without which an exposed part is
// a claim no test touches. That batch does not add props, write a component's behaviour tests, or
// touch any component's test module, and a component's `library:` item (which owns those axes, and
// which is where this gate's props/sections/hygiene verdict belongs) is a different item that this
// batch does not reopen. So for a surface batch those three axes are still MEASURED and PRINTED with
// their named gaps, but they do not decide the batch's verdict — otherwise a test-count floor in
// `button` (a component this batch does not touch and whose parts are unchanged) would report the
// surface work as unfinished, which is a verdict about the wrong item. Nothing is hidden: every gap
// line still names the spec line it came from.
//
// Exit: 0 clean, 1 gaps (with --strict), 2 nothing to check (missing spec/tests — reported, never a pass).

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const ROOT = path.resolve(import.meta.dirname, '../..');
const Crate = path.join(ROOT, 'crates/leptos-ui');
const strict = process.argv.includes('--strict');

function arg(name) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] && !process.argv[i + 1].startsWith('--') ? process.argv[i + 1] : null;
}

const camelToSnake = (s) => s.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase();

// Module-name normalization, shared verbatim with `check-part-surface.mjs`: an ALL-CAPS acronym run
// (`OTPField`, `CSPProvider`) must split BEFORE the lower->upper rule, or `otp-field` matches no
// module/alias and the unit silently reads as having nothing to check. The two authorities
// disagreeing about the same spec is how `otp-field` came to be reported as an absent surface.
const snake = (s) =>
  s
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1_$2')
    .replace(/([a-z0-9])([A-Z])/g, '$1_$2')
    .replace(/[\s-]/g, '_')
    .toLowerCase();

// The capitalised name a CONSUMER writes for this unit: whatever `src/lib.rs` re-exports the module
// as (`pub use self::otp_field as OTPField;`). This is the authority for the namespaced spelling —
// NOT a PascalCase guess from the item id, which produces a string the crate never exports (see the
// namespaced-path axis). Returns null when the crate exports no alias for the unit.
function exportedAlias(name) {
  const p = path.join(Crate, 'src/lib.rs');
  if (!fs.existsSync(p)) return null;
  const m = fs.readFileSync(p, 'utf8').match(new RegExp(`pub\\s+use\\s+self::${snake(name)}\\s+as\\s+([A-Za-z0-9_]+)`));
  return m ? m[1] : null;
}

/** Which components does this invocation cover? */
function componentsFromArgs() {
  const one = arg('component');
  if (one) return [one];
  const todo = arg('todo-id');
  if (todo) {
    const m = todo.match(/^library:\s*([a-z0-9-]+)/i);
    if (m && !/namespaced part surface/.test(todo)) return [m[1]];
    const r = spawnSync('node', ['ralph/scripts/get-todo-field.mjs', todo, 'components'], { cwd: ROOT, encoding: 'utf8' });
    const list = (r.stdout || '').trim().split(',').map((s) => s.trim()).filter(Boolean);
    if (list.length) return list;
  }
  return [];
}

function readSpec(name) {
  const p = path.join(ROOT, 'specs/library', name, 'behavior.md');
  return fs.existsSync(p) ? { path: path.relative(ROOT, p), text: fs.readFileSync(p, 'utf8') } : null;
}

/** Port source for a component: the module dir and/or the <name>_tests.rs sibling. */
function portFiles(name) {
  const out = [];
  const dir = path.join(Crate, 'src', name);
  if (fs.existsSync(dir)) {
    const walk = (d) => {
      for (const e of fs.readdirSync(d, { withFileTypes: true })) {
        const p = path.join(d, e.name);
        if (e.isDirectory()) walk(p);
        else if (e.name.endsWith('.rs')) out.push(p);
      }
    };
    walk(dir);
  }
  for (const cand of [`${name}_tests.rs`, `${name}.rs`, path.join(name, 'mod_test.rs')]) {
    const p = path.join(Crate, 'src', cand);
    if (fs.existsSync(p)) out.push(p);
  }
  return out;
}

/** Parts proven by the spec: dotted `Checkbox.Root` names. */
function specParts(spec) {
  const parts = new Set();
  for (const m of spec.text.matchAll(/`([A-Z][A-Za-z0-9]*)\.([A-Z][A-Za-z0-9]*)`/g)) parts.add(`${m[1]}.${m[2]}`);
  return [...parts];
}

/** Props "proven by tests" per part, with the spec line for citation. */
function specProps(spec) {
  const out = [];   // { part, prop, line }
  const lines = spec.text.split('\n');
  lines.forEach((line, i) => {
    const m = line.match(/Props on ([A-Za-z0-9]+) proven by tests:\s*(.+)$/);
    if (!m) return;
    const part = m[1];
    const segment = m[2].split('(')[0];               // stop before the citations
    for (const pm of segment.matchAll(/`([A-Za-z][A-Za-z0-9]*)`/g)) out.push({ part, prop: pm[1], line: i + 1, text: line.trim().slice(0, 110) });
  });
  return out;
}

function sections(spec) {
  const out = [];
  const lines = spec.text.split('\n');
  let cur = null;
  for (const line of lines) {
    const h = line.match(/^##\s+(.+)$/);
    if (h) { cur = { name: h[1].trim(), body: '' }; out.push(cur); continue; }
    if (cur) cur.body += `${line}\n`;
  }
  const STOP = new Set(['this', 'that', 'with', 'when', 'must', 'from', 'they', 'have', 'been', 'into', 'also', 'than', 'then', 'than', 'uses', 'used', 'does', 'each', 'only', 'same', 'which', 'their', 'there', 'while', 'being', 'other', 'these', 'those', 'value', 'state', 'props', 'prop', 'tests', 'test', 'component', 'upstream', 'default']);
  for (const s of out) {
    const words = new Set();
    for (const w of `${s.name} ${s.body}`.toLowerCase().matchAll(/[a-z][a-z0-9_-]{3,}/g)) if (!STOP.has(w[0])) words.add(w[0]);
    s.keywords = [...words].slice(0, 40);
  }
  return out;
}

function main() {
  const comps = componentsFromArgs();
  if (!comps.length) {
    console.error('usage: check-component-strict.mjs --component <name> | --todo-id "<library: …>"  (a surface-batch id needs a `components:` field)');
    return 2;
  }
  const SURFACE_BATCH = /namespaced part surface/.test(arg('todo-id') || '');
  const AXIS_NOTE = SURFACE_BATCH ? ' [advisory for a surface batch — the component’s own `library:` item owns this axis]' : '';
  let gaps = 0;
  let checked = 0;
  // `parts` is the axis a surface batch exists to close; the other three belong to the component's
  // own item (see the header). `offAxis` counts them separately so the summary can say so.
  let offAxis = 0;
  const gap = (ownAxis) => {
    if (ownAxis || !SURFACE_BATCH) gaps++;
    else offAxis++;
  };

  for (const name of comps) {
    console.log(`\n=== ${name} ===`);
    const spec = readSpec(name);
    if (!spec) { console.log(`  NOT CHECKED: no specs/library/${name}/behavior.md — the spec is what makes this gate possible; author it first`); gap(true); continue; }
    const files = portFiles(name);
    const testFiles = files.filter((f) => /_tests\.rs$|mod_test\.rs$/.test(f));
    const srcText = files.map((f) => fs.readFileSync(f, 'utf8')).join('\n');
    const testText = testFiles.map((f) => fs.readFileSync(f, 'utf8')).join('\n');
    checked++;

    // ---- parts: delegated to the part-surface authority
    const ps = spawnSync('node', ['ralph/scripts/check-part-surface.mjs', '--components', name, '--strict'], { cwd: ROOT, encoding: 'utf8' });
    const psOut = `${ps.stdout || ''}${ps.stderr || ''}`;
    const missingLine = psOut.split('\n').find((l) => /missing|not exposed|0\//i.test(l) && l.trim());
    const allSpecParts = specParts(spec);
    // A behaviour spec mentions other components by design (`Checkbox.Root` composes with `Field.Root`), so
    // the failure message must not claim "the spec proves Field.Root" when checking checkbox. Own parts drive
    // the verdict; foreign ones are reported separately as context.
    const ownHead = name.replace(/-/g, '').toLowerCase();
    const specPartList = allSpecParts.filter((x) => x.split('.')[0].replace(/-/g, '').toLowerCase() === ownHead);
    const foreignParts = allSpecParts.filter((x) => !specPartList.includes(x));
    if (!specPartList.length) { console.log('  parts: spec proves no dotted parts — nothing to check'); }
    else if (ps.status === 0) console.log(`  parts: OK — ${specPartList.length} own spec part(s) exposed (${specPartList.join(', ')})${foreignParts.length ? ` [spec also references ${foreignParts.length} part(s) of other units — not this component's bar]` : ''}`);
    else { console.log(`  parts: FAIL — this unit's spec proves ${specPartList.join(', ') || '(none by name)'}; part-surface gate says: ${(missingLine || psOut.split('\n')[0] || 'see check-part-surface output').trim().slice(0, 160)}`); gap(true); }

    // ---- props
    const props = specProps(spec);
    const missingProps = props.filter((p) => !new RegExp(`pub\\s+${camelToSnake(p.prop)}\\b`).test(srcText));
    if (!props.length) console.log('  props: spec lists no "proven by tests" props — nothing to check');
    else if (!missingProps.length) console.log(`  props: OK — all ${props.length} spec-proven prop(s) present on the port`);
    else {
      console.log(`  props: FAIL — ${missingProps.length} of ${props.length} spec-proven prop(s) missing from the port:${AXIS_NOTE}`);
      for (const p of missingProps.slice(0, 12)) console.log(`     ${p.part}.${p.prop} -> expected field \`${camelToSnake(p.prop)}\` (spec ${spec.path}:${p.line}: ${p.text})`);
      gap(false);
    }

    // ---- sections: each obligation area must be touched by a test
    const secs = sections(spec);
    const untested = secs.filter((s) => s.keywords.length && !s.keywords.some((k) => testText.toLowerCase().includes(k)));
    if (!secs.length) console.log('  sections: spec has no ## sections — nothing to check');
    else if (!untested.length) console.log(`  sections: OK — all ${secs.length} spec section(s) have test coverage vocabulary`);
    else {
      console.log(`  sections: FAIL — ${untested.length} of ${secs.length} spec section(s) have no test touching their vocabulary:${AXIS_NOTE}`);
      for (const s of untested.slice(0, 8)) console.log(`     "${s.name}" (tried: ${s.keywords.slice(0, 6).join(', ')})`);
      gap(false);
    }

    // ---- namespaced-path evidence
    // The surface batches' done-when requires "one part-surface test per module exercising the namespaced
    // path" — and nothing checked it. Measured 2026-09-16 after check-part-surface --strict exited 0 for all
    // 38 parts: five of the fourteen components were reported as having NO test using `<Component::Part`
    // markup anywhere. The declared number was green; the clause that makes the surface trustworthy was
    // prose. A done-when clause no gate can falsify is a lie waiting to happen, so it is now an axis, and it
    // is HARD for the surface batches (the evidence is their own deliverable).
    //
    // THE FIRST READING OF THAT CLAUSE WAS WRONG IN TWO WAYS, and both are fixed here rather than worked
    // around (the fixes only ever move a verdict towards the truth; the spelling below is now the crate's
    // own, and no unit with a documented surface escapes):
    //   1. The expected prefix was re-derived from the item id with a PascalCase guess, so `otp-field`
    //      produced `OtpField` — a name the crate does not export (`lib.rs` says `pub use self::otp_field
    //      as OTPField;`). The port's real pin, `crates/leptos-ui/tests/part_surface.rs` (a `<OTPField::Root>`
    //      tree with all three documented parts, plus its own `#[test]`), matched NOTHING, and a surface that
    //      exists and is exercised was reported as "a claim, not a surface". The prefix now comes from the
    //      crate's own alias, which is the authority for what a consumer can actually write.
    //   2. A unit whose spec documents no part OF ITS OWN has no namespaced surface to exercise. `button`,
    //      `checkbox-group`, `separator` and `toggle` are single components upstream (`<Button>`,
    //      `<CheckboxGroup>`, `<Separator>`, `<Toggle>` — measured 2026-09-16: zero own dotted parts in their
    //      behavior.md; checkbox-group's only dotted references are to OTHER units, `Checkbox.*`/`Field.*`;
    //      and their modules expose `*Props`/`*State` structs, no part). Demanding `<Button::…>` markup
    //      there could only be satisfied by inventing API upstream does not document, so the axis is INERT
    //      with an explicit message — exactly as the `parts` axis is when a spec proves no dotted part.
    //      For every unit that DOES document parts the bar is unchanged: real namespaced markup in a test,
    //      or a named gap.
    if (!specPartList.length) {
      console.log(`  namespaced path: nothing to check — the spec documents no part of this unit's own (${foreignParts.length} dotted reference(s) belong to other units), so there is no namespaced surface to exercise here`);
    } else {
      // No alias → the path cannot be written by a consumer at all; that is a gap even if some file
      // happens to contain the guessed spelling, so it is reported as one.
      const alias = exportedAlias(name);
      const guessed = name.split(/[-_]/).map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join('');
      const prefix = alias || guessed;
      const nsRe = new RegExp(`<${prefix}::[A-Z]`);
      let nsHits = [...(testText + '\n' + srcText).split('\n')].filter((l) => nsRe.test(l)).length;
      if (!nsHits) {
        const testsDir = path.join(Crate, 'tests');
        if (fs.existsSync(testsDir)) {
          for (const f of fs.readdirSync(testsDir)) {
            if (f.endsWith('.rs') && nsRe.test(fs.readFileSync(path.join(testsDir, f), 'utf8'))) { nsHits++; break; }
          }
        }
      }
      const aliasNote = alias
        ? ''
        : `; ${name} exports no capitalised alias in src/lib.rs, so no consumer can write <${guessed}::…> — the alias is part of the surface`;
      if (nsHits && alias) console.log(`  namespaced path: OK — ${nsHits} use(s) of <${prefix}::…> in this unit's tests/source`);
      else if (nsHits) { console.log(`  namespaced path: FAIL — ${nsHits} use(s) of the guessed spelling <${prefix}::…>, but this unit exports no lib.rs alias${aliasNote}`); gaps++; }
      else { console.log(`  namespaced path: FAIL — no test uses <${prefix}::…> markup, so nothing exercises the namespaced surface this unit is supposed to expose (a namespaced part that no test touches is a claim, not a surface${aliasNote})`); gaps++; }
    }

    // ---- hygiene
    const ignored = (testText.match(/#\[ignore[^\]]*\]/g) || []).length;
    const testCount = (testText.match(/#\[test\]/g) || []).length;
    const floor = secs.length;
    if (ignored) { console.log(`  hygiene: FAIL — ${ignored} #[ignore]d test(s) in the component's test module; a disabled test is not evidence${AXIS_NOTE}`); gap(false); }
    if (testCount < floor) { console.log(`  hygiene: FAIL — ${testCount} test(s) for ${secs.length} spec section(s) (floor: one per section)${AXIS_NOTE}`); gap(false); }
    if (!ignored && testCount >= floor) console.log(`  hygiene: OK — ${testCount} test(s), none ignored (floor ${floor})`);
    if (!testFiles.length) console.log(`  NOTE: no test module found for ${name} (looked for src/${name}_tests.rs or src/${name}/mod_test.rs)`);
  }

  console.log(`\ncomponent strict: ${checked} component(s) checked, ${gaps} gap(s)${strict ? ' — strict mode' : ' (advisory; use --strict to fail)'}`);
  if (offAxis) console.log(`${offAxis} further gap(s) were measured on axes this item does not own (props/sections/hygiene — owned by each component's own library item); they are listed above and excluded from the verdict, not from the report.`);
  if (!checked) return 2;
  return strict && gaps ? 1 : 0;
}

process.exit(main());
