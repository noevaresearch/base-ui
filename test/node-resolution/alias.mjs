/* eslint-disable no-console */

// The port's own name must resolve AS A PACKAGE — this is the name the mirrored docs tell a reader to use,
// so if it stops resolving, the docs reference something that exists nowhere. It lives in its own file
// because index.mjs (upstream's resolution fixture) throws on its first import in a fresh tree
// (packages/react has no built dist), which would silently swallow these assertions.
//
// Run: node test/node-resolution/alias.mjs   (also run by ralph/scripts/check-package-alias.mjs)
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import * as ours from 'base-ui-leptos';

const require = createRequire(import.meta.url);
const problems = [];

if (ours.PACKAGE_NAME !== 'base-ui-leptos') problems.push(`alias reports PACKAGE_NAME=${ours.PACKAGE_NAME}`);
// The crate was RENAMED to `base-ui-leptos` (published on crates.io at 0.1.1); this fixture kept
// asserting the old `leptos-ui` name and so failed the whole alias chain in `check-package-alias.mjs`
// even though `packages/leptos/package.json`, `packages/leptos/lib/index.js`,
// `crates/leptos-ui/Cargo.toml` and `crates/docs-app/src/install_ref.rs` all already agreed. The
// source of truth for the name is `crates/leptos-ui/Cargo.toml`; the value is pinned here so a
// silent drift back to a name the crate does not have is a failure, not a footnote.
if (ours.RUST_CRATE !== 'base-ui-leptos') problems.push(`alias points at crate ${ours.RUST_CRATE}`);
if (ours.NOT_PUBLISHED !== true) problems.push('alias claims to be published — it is not');

let manifestPath = null;
try {
  manifestPath = require.resolve('base-ui-leptos/package.json');
} catch (e) {
  problems.push(`subpath resolution failed: ${e.code}`);
}
let manifest = null;
if (manifestPath) {
  manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  if (manifest.name !== 'base-ui-leptos') problems.push(`manifest name is ${manifest.name}`);
  if (manifest.private !== true) problems.push('manifest must stay private:true until the crate ships');
  if (manifest.publishConfig) problems.push('manifest carries publishConfig while unpublished');
}

if (problems.length) {
  console.error(`alias check FAILED (${manifestPath ?? 'unresolved'}):`);
  for (const p of problems) console.error(`  - ${p}`);
  process.exit(1);
}
console.log(`alias OK: ${ours.PACKAGE_NAME} -> ${manifestPath} (crate ${ours.RUST_CRATE}, unpublished)`);
