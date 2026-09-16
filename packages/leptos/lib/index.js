// @noevaresearch/base-ui — locally mapped entry point.
//
// This package is the port's PUBLIC NAME. It exists so that every install line, link and snippet in the
// mirrored docs can point at this port (`@noevaresearch/base-ui`) instead of upstream's
// `@base-ui/react` — which is a different library, in a different language, and the wrong thing to send
// a reader to.
//
// It is deliberately `private: true` (and therefore unpublishable): the crate is not published yet. The
// real artifact today is the Leptos UI crate in this repo:
//   * Rust:    crates/leptos-ui      (the component library)
//   * Docs app: crates/docs-app      (the wasm docs site, built to target/site)
// When the crate is published, the wasm bundle moves here, `private` is dropped and the version is set
// from the release tag — a deliberate, reviewed step, not something a docs iteration can trigger.

export const PACKAGE_NAME = '@noevaresearch/base-ui';
export const RUST_CRATE = 'leptos-ui';
export const NOT_PUBLISHED = true;

/** Resolve the wasm artifact the docs app builds today. Throws with instructions rather than 404ing. */
export function artifactPath(root = process.cwd()) {
  return `${root}/target/site/pkg/docs-app.wasm`;
}

export function loadInfo() {
  return {
    package: PACKAGE_NAME,
    crate: RUST_CRATE,
    published: false,
    note: 'Locally mapped alias for the Leptos port. React’s @base-ui/react is NOT this package.',
  };
}
