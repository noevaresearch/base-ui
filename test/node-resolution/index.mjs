/* eslint-disable no-console */
import * as base from '@base-ui/react';
import { Accordion } from '@base-ui/react/accordion';

// The port's own name. This is the alias the mirrored docs tell a reader to install, so it must RESOLVE
// locally (workspace link) while being deliberately unpublished: if this import ever starts failing, the
// docs are naming a package that does not exist. `private: true` is asserted too — an accidental publish
// would turn the alias into a real, stale package on npm.
import * as ours from '@noevaresearch/base-ui';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';

console.assert(base, 'base');
console.assert(Accordion, 'Accordion');

console.assert(ours.PACKAGE_NAME === '@noevaresearch/base-ui', `alias name is ${ours.PACKAGE_NAME}`);
console.assert(ours.RUST_CRATE === 'leptos-ui', 'alias must point at the Rust crate');
console.assert(ours.NOT_PUBLISHED === true, 'alias reports published — it must not be');

const require = createRequire(import.meta.url);
const manifestPath = require.resolve('@noevaresearch/base-ui/package.json');
const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
console.assert(manifest.private === true, 'package.json must stay private: true until the crate ships');
console.assert(!manifest.publishConfig, 'no publishConfig while unpublished');

console.log(`alias resolved: ${ours.PACKAGE_NAME} -> ${require.resolve('@noevaresearch/base-ui')}`);
console.log(`rust crate: ${ours.RUST_CRATE}; published: ${!ours.NOT_PUBLISHED}`);
