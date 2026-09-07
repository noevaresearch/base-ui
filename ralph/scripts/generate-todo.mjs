#!/usr/bin/env node
/* eslint-disable no-console */

// Generates the initial specs/ skeleton (empty dirs, one placeholder per spec-mining unit)
// and TODO.md skeleton from the three enumeration manifests. Encodes the classification
// decisions made before running this script:
//
// - packages/react/src/ splits into ~38 real components (Phase B, crate: leptos-ui) and 9
//   shared-infra dirs (Phase A, each its own crate: leptos-<dir>, except `utils` which becomes
//   `leptos-react-utils` to avoid clashing with `leptos-ui-utils`, the packages/utils/src crate).
// - Of those 9 infra dirs, 4 (use-render, merge-props, direction-provider, csp-provider) have a
//   real docs page under docs/(react)/utils/<name> and so DO get a paired Phase D item, like a
//   component. The other 5 (internals, utils, floating-ui-react, types,
//   unstable-use-media-query) have no docs page and are exempt from the Phase-D-pairing rule.
// - `radio-group` has source but no dedicated docs page — it's documented on the `radio` page.
//   Its Phase B item points at the existing `components/radio` Phase D item instead of a
//   fabricated one of its own.
// - `overview/releases/**` docs routes are historical changelog content, out of scope for the
//   port entirely — excluded, not just non-gating.
// - Remaining docs routes (handbook/*, overview/{about,accessibility,community,quick-start,
//   releases index}, bare index pages) are conceptual/cross-cutting pages with no single owning
//   component — listed as a non-gating "Phase D-extra" section, ported last, blocking nothing.
//
// This script is idempotent-ish for a first run: it errors instead of overwriting if TODO.md
// already exists, since a regenerate against a TODO.md with real progress on it would destroy
// state — regenerating after the fact needs a human decision, not a silent script.
//
// Usage: node ralph/scripts/generate-todo.mjs

import fs from 'fs/promises';
import path from 'path';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const GENERATED_DIR = path.join(PROJECT_ROOT, 'ralph/generated');
const SPECS_ROOT = path.join(PROJECT_ROOT, 'specs');
const TODO_PATH = path.join(PROJECT_ROOT, 'TODO.md');

const INFRA_DIRS = new Set([
  'use-render',
  'merge-props',
  'direction-provider',
  'csp-provider',
  'unstable-use-media-query',
  'internals',
  'utils',
  'floating-ui-react',
  'types',
]);

// Infra dirs that DO have a paired docs page, under docs/(react)/utils/<name>.
const INFRA_WITH_DOCS = new Set(['use-render', 'merge-props', 'direction-provider', 'csp-provider']);

// Infra dirs that primarily wrap an external npm package rather than implementing their own
// algorithm — for these, a maintained Rust/Leptos equivalent of the external dependency already
// exists, so Stage 1/2 spec-mining scope narrows to Base UI's OWN wrapper layer only (its hooks/
// components/middleware, not the third-party library's internals), and the Leptos crate binds to
// the existing Rust crate rather than reimplementing the algorithm. Verified 2026-09-07: floating-
// ui-react wraps @floating-ui/react-dom + @floating-ui/utils; RustForWeb/floating-ui already
// publishes floating-ui-core/floating-ui-dom/floating-ui-leptos (a `use_floating` composable
// mirroring React's `useFloating`) — see https://floating-ui.rustforweb.org/frameworks/leptos.html.
// If another infra/component unit turns out to wrap a different external package, add it here
// with the same research (confirm an equivalent exists) before assuming a full port is needed.
const WRAPS_EXTERNAL_DEPENDENCY = {
  'floating-ui-react': {
    externalPackages: ['@floating-ui/react-dom', '@floating-ui/utils'],
    rustEquivalentCrate: 'floating-ui-leptos',
    rustEquivalentUrl: 'https://floating-ui.rustforweb.org/frameworks/leptos.html',
  },
};

// Component source dirs with no docs page of their own — documented on another component's page.
const SHARED_DOCS_PAGE_OVERRIDE = { 'radio-group': 'components/radio' };

// Units whose test suite is large enough (>=8000 lines, or >=20 test files) that a single
// one-shot Stage 1 subagent reading the whole thing in one pass risks either blowing context or
// producing a shallow spec. Measured 2026-09-07 against packages/react/src: combobox is the
// extreme case (23,292 lines / 39 files — combobox/root alone is 13,256 lines, over 2x Dialog's
// entire suite). These need batched mining: fan out per SUBDIRECTORY within the unit (e.g.
// combobox/root, combobox/input, combobox/items as separate subagent tasks), each writing to
// specs/library/<unit>/parts/<subdir>.md, with a final lightweight synthesis pass producing the
// unit's top-level behavior.md as an index + cross-references — not a single subagent reading
// every test file in the unit at once. See ralph/prompts/stage1-behavior-mining.md's batching note.
const NEEDS_BATCHED_MINING = new Set([
  'combobox',
  'drawer',
  'floating-ui-react',
  'menu',
  'number-field',
  'select',
]);

function crateNameForInfraDir(dirName) {
  // Avoid clashing with the packages/utils/src crate, leptos-ui-utils.
  if (dirName === 'utils') {
    return 'leptos-react-utils';
  }
  return `leptos-${dirName}`;
}

async function loadJson(fileName) {
  const raw = await fs.readFile(path.join(GENERATED_DIR, fileName), 'utf8');
  return JSON.parse(raw);
}

function todoItem({ id, crate, specs, blockedBy, doneWhen, docsPair, owner, exempt, extra, wraps, needsBatchedMining }) {
  const lines = [`- [ ] ${id}`, `      crate: ${crate}`, `      specs: ${specs.join(', ')}`];
  if (blockedBy && blockedBy.length > 0) {
    lines.push(`      blocked-by: [${blockedBy.join(', ')}]`);
  }
  lines.push('      status: not-started');
  lines.push(`      done-when: ${doneWhen}`);
  // docs-pair / owner are the machine-checkable link enforcing "no Phase B/A-with-docs item is
  // fully done until its docs-content counterpart is also done" — check-todo-schema.mjs reads
  // these fields directly rather than parsing prose out of done-when.
  if (docsPair) {
    lines.push(`      docs-pair: ${docsPair}`);
  }
  if (owner) {
    lines.push(`      owner: ${owner}`);
  }
  if (exempt) {
    lines.push('      exempt-from-docs-pairing: true');
  }
  if (wraps) {
    lines.push(`      wraps-external: ${wraps.externalPackages.join(', ')}`);
    lines.push(`      rust-equivalent-crate: ${wraps.rustEquivalentCrate}  # ${wraps.rustEquivalentUrl}`);
  }
  if (needsBatchedMining) {
    lines.push('      needs-batched-mining: true  # too large for one Stage 1 subagent — fan out per subdirectory');
  }
  if (extra) {
    lines.push(`      ${extra}`);
  }
  return lines.join('\n');
}

async function main() {
  const existingTodo = await fs.stat(TODO_PATH).catch(() => null);
  if (existingTodo) {
    console.error(
      `${path.relative(PROJECT_ROOT, TODO_PATH)} already exists — refusing to overwrite. ` +
        'Regenerating against real progress needs a human decision, not a silent script run.',
    );
    process.exitCode = 1;
    return;
  }

  const components = await loadJson('components.json');
  const utils = await loadJson('utils.json');
  const docsContent = await loadJson('docs-content.json');

  const infraEntries = components.filter((c) => INFRA_DIRS.has(c.name));
  const realComponents = components.filter((c) => !INFRA_DIRS.has(c.name));

  const docsRoutesByComponentsSlug = new Map(
    docsContent
      .filter((p) => p.route.startsWith('components/'))
      .map((p) => [p.route.slice('components/'.length), p]),
  );
  const docsRoutesByUtilsSlug = new Map(
    docsContent
      .filter((p) => p.route.startsWith('utils/') && p.route !== 'utils')
      .map((p) => [p.route.slice('utils/'.length), p]),
  );

  // ---- Phase A: utils (packages/utils/src) ----
  const phaseAUtilsIds = [];
  const phaseAUtilsItems = utils.map((u) => {
    const id = `utils: ${u.name}`;
    phaseAUtilsIds.push(id);
    return todoItem({
      id,
      crate: 'leptos-ui-utils',
      specs: [`specs/utils/${u.name}.md`],
      doneWhen: 'crates/leptos-ui-utils tests pass; cargo test --workspace green',
    });
  });

  // ---- Phase A: react-internal shared infra (own crate each) ----
  const phaseAInfraIds = [];
  const phaseAInfraItems = infraEntries.map((c) => {
    const id = `infra: ${c.name}`;
    phaseAInfraIds.push(id);
    const crate = crateNameForInfraDir(c.name);
    const specs = [`specs/library/${c.name}/behavior.md`, `specs/library/${c.name}/implementation.md`];
    const hasDocsPage = INFRA_WITH_DOCS.has(c.name);
    const wraps = WRAPS_EXTERNAL_DEPENDENCY[c.name];
    const doneWhen = wraps
      ? `crates/${crate} (a thin binding over ${wraps.rustEquivalentCrate}, not a from-scratch ` +
        `port of ${wraps.externalPackages.join('/')}) tests pass; cargo test --workspace green`
      : `crates/${crate} tests pass; cargo test --workspace green`;
    return todoItem({
      id,
      crate,
      specs,
      doneWhen,
      docsPair: hasDocsPage ? `docs-content: utils/${c.name}` : undefined,
      exempt: !hasDocsPage,
      wraps,
      needsBatchedMining: NEEDS_BATCHED_MINING.has(c.name),
    });
  });

  // ---- Phase B: library components ----
  const phaseBIds = [];
  const phaseBItems = realComponents.map((c) => {
    const id = `library: ${c.name}`;
    phaseBIds.push(id);
    const docsSlug = SHARED_DOCS_PAGE_OVERRIDE[c.name] ?? `components/${c.name}`;
    const extra = SHARED_DOCS_PAGE_OVERRIDE[c.name]
      ? `# shares docs-pair with library: ${SHARED_DOCS_PAGE_OVERRIDE[c.name].replace('components/', '')} — documented on ${SHARED_DOCS_PAGE_OVERRIDE[c.name]}'s page, not its own`
      : undefined;
    return todoItem({
      id,
      crate: 'leptos-ui',
      specs: [
        `specs/library/${c.name}/behavior.md`,
        `specs/library/${c.name}/implementation.md`,
        `specs/library/${c.name}/fixtures.json`,
      ],
      // "Phase A complete" is a synthetic dependency, not a literal item id: it means "every
      // Phase A item is done," checked by check-todo-schema.mjs. Listing all ~57 Phase A ids on
      // every one of the ~38 Phase B items would be unreadable and would drift the moment Phase A
      // itself changes. Per-component precise dependencies (e.g. dialog only really needs
      // useControlled + floating-ui-react, not every util) can replace this coarse default once
      // Stage 1/2 specs reveal them — see specs/library/<name>/implementation.md's "Dependencies
      // on other Base UI internals" section once it exists.
      blockedBy: ['Phase A complete'],
      doneWhen: 'crates/leptos-ui fixtures.json oracle assertions pass; cargo test --workspace green',
      docsPair: `docs-content: ${docsSlug}`,
      extra,
      needsBatchedMining: NEEDS_BATCHED_MINING.has(c.name),
    });
  });

  // ---- Phase C: docs-app shell ----
  const phaseCId = 'docs-app: routing + layout shell';
  const phaseCItem = todoItem({
    id: phaseCId,
    crate: 'docs-app',
    specs: ['specs/docs-app/infra.md'],
    doneWhen: 'crates/docs-app builds and serves at least one route using a real leptos-ui component',
  });

  // ---- Phase D: docs content (paired 1:1 with Phase B/A-with-docs items) ----
  const phaseDItems = [];
  for (const c of realComponents) {
    const docsSlug = SHARED_DOCS_PAGE_OVERRIDE[c.name] ?? `components/${c.name}`;
    if (SHARED_DOCS_PAGE_OVERRIDE[c.name]) {
      // Shared page already generated for its owning component; don't duplicate.
      continue;
    }
    const page = docsRoutesByComponentsSlug.get(c.name);
    const id = `docs-content: ${docsSlug}`;
    phaseDItems.push(
      todoItem({
        id,
        crate: 'docs-app',
        specs: [`specs/docs-content/${c.name}/page.md`, `specs/docs-content/${c.name}/demos.json`],
        blockedBy: [`library: ${c.name}`, phaseCId],
        owner: `library: ${c.name}`,
        doneWhen:
          `docs-app renders ${page ? page.pagePath : docsSlug} with all its demos using ` +
          'crates/leptos-ui\'s real component (verified via Playwright differential test ' +
          'against the original React docs page, not just a smoke render)',
      }),
    );
  }
  for (const infraName of INFRA_WITH_DOCS) {
    const page = docsRoutesByUtilsSlug.get(infraName);
    const id = `docs-content: utils/${infraName}`;
    phaseDItems.push(
      todoItem({
        id,
        crate: 'docs-app',
        specs: [`specs/docs-content/${infraName}/page.md`, `specs/docs-content/${infraName}/demos.json`],
        blockedBy: [`infra: ${infraName}`, phaseCId],
        owner: `infra: ${infraName}`,
        doneWhen:
          `docs-app renders ${page ? page.pagePath : `utils/${infraName}`} using ` +
          `crates/${crateNameForInfraDir(infraName)}'s real implementation (verified via ` +
          'Playwright differential test against the original React docs page)',
      }),
    );
  }

  // ---- Phase D-extra: conceptual/cross-cutting docs pages (non-gating) ----
  const excludedPrefixes = ['overview/releases'];
  const alreadyHandledRoutes = new Set([
    ...[...docsRoutesByComponentsSlug.values()].map((p) => p.route),
    ...[...docsRoutesByUtilsSlug.values()].map((p) => p.route),
  ]);
  const extraRoutes = docsContent.filter(
    (p) =>
      !alreadyHandledRoutes.has(p.route) &&
      !excludedPrefixes.some((prefix) => p.route === prefix || p.route.startsWith(`${prefix}/`)),
  );

  // ---- Write specs/ skeleton ----
  await fs.mkdir(path.join(SPECS_ROOT, 'library'), { recursive: true });
  await fs.mkdir(path.join(SPECS_ROOT, 'utils'), { recursive: true });
  await fs.mkdir(path.join(SPECS_ROOT, 'docs-content'), { recursive: true });
  await fs.mkdir(path.join(SPECS_ROOT, 'docs-app'), { recursive: true });

  for (const c of components) {
    await fs.mkdir(path.join(SPECS_ROOT, 'library', c.name), { recursive: true });
  }
  for (const docsSlug of [...docsRoutesByComponentsSlug.keys(), ...docsRoutesByUtilsSlug.keys()]) {
    await fs.mkdir(path.join(SPECS_ROOT, 'docs-content', docsSlug), { recursive: true });
  }

  await fs.writeFile(
    path.join(SPECS_ROOT, 'architecture.md'),
    '# Architecture\n\n(Written after all Stage 1/2 spec-mining lands — see build-order step 9.)\n',
    'utf8',
  );

  // ---- Write TODO.md ----
  const todo = `# Ralph Port TODO

Generated by \`ralph/scripts/generate-todo.mjs\` from \`ralph/generated/{components,utils,docs-content}.json\`.
See \`specs/architecture.md\` for cross-cutting Leptos design decisions (written at build-order step 9,
before Stage 3 forward-loop work begins).

## Phase A — Utils (packages/utils/src → crate leptos-ui-utils)

${phaseAUtilsItems.join('\n')}

## Phase A — React-internal shared infra (packages/react/src/{use-render,...} → one crate each)

${phaseAInfraItems.join('\n')}

## Phase B — Library components (blocked-by: all Phase A items; crate leptos-ui)

${phaseBItems.join('\n')}

## Phase C — Docs-app shell (blocked-by: nothing yet — pilot can use a stub component)

${phaseCItem}

## Phase D — Docs content (blocked-by: matching Phase B/A item + Phase C)

${phaseDItems.join('\n')}

## Phase D-extra — Conceptual/cross-cutting docs pages (non-gating, ported last)

${extraRoutes.map((p) => `- [ ] docs-content-extra: ${p.route || '(root)'}\n      specs: (not yet mined)\n      status: not-started\n      note: cross-cutting page, no single owning component; does not block any Phase B/D item`).join('\n')}

## Excluded (out of scope)

- overview/releases/** — historical changelog content, ${docsContent.filter((p) => p.route.startsWith('overview/releases/')).length} pages, not ported.
`;

  await fs.writeFile(TODO_PATH, todo, 'utf8');

  console.log(`Wrote ${path.relative(PROJECT_ROOT, TODO_PATH)}`);
  console.log(`  Phase A (utils): ${phaseAUtilsItems.length} items`);
  console.log(`  Phase A (infra): ${phaseAInfraItems.length} items`);
  console.log(`  Phase B (components): ${phaseBItems.length} items`);
  console.log(`  Phase C: 1 item`);
  console.log(`  Phase D: ${phaseDItems.length} items`);
  console.log(`  Phase D-extra: ${extraRoutes.length} items (non-gating)`);
  console.log(`  Excluded: ${docsContent.filter((p) => p.route.startsWith('overview/releases/')).length} release-note pages`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
