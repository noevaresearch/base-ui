# Releasing the Rust crates

One `0.1.<patch>` release per 10 commits, published to crates.io by
`.github/workflows/publish-crates.yml`, and **only** from a commit that passes the gate.

## The names, and why they differ from the repo

| what a consumer types | what the repo calls it |
|---|---|
| `cargo add base-ui-leptos` | `crates/leptos-ui/` (package `base-ui-leptos`, lib `leptos_ui`) |
| (internal dep) `base-ui-leptos-internals` | `crates/leptos-ui-internals/`, lib `leptos_ui_internals` |
| (internal dep) `base-ui-leptos-utils` | `crates/leptos-ui-utils/`, lib `leptos_ui_utils` |

Two deliberate asymmetries:

1. **The published package name is not `leptos-ui`.** That name was taken on crates.io by an
   unrelated, actively-released project (`leptos-ui` 0.3.x, 51k downloads). Publishing under it is
   impossible, and the docs' old `cargo add leptos-ui` line was, until this change, sending readers
   to install that stranger's crate.
2. **The Rust *import* name does not change.** `[lib] name = "leptos_ui"` (and `leptos_ui_utils`,
   `leptos_ui_internals`) keeps the ~212 existing `use leptos_ui::…` sites valid, so the rename is
   confined to manifests, docs strings and the ledger. Consumers therefore write
   `cargo add base-ui-leptos` and `use leptos_ui::…` — the normal split between a package name and
   its lib name.

The directory names (`crates/leptos-ui*`) are unchanged on purpose: hundreds of doc-comment
citations point at those paths.

## The cadence, and where the state lives

`release/release-state.mjs` answers two questions per push: *is a release due?* and *which version?*

- **Origin:** the commit that ADDED `release/release.json`. Without an origin, `commit_count % 10`
  against 5,000+ commits of history would fire on every push forever. Reset the counter by
  `git rm` + re-add in one commit.
- **Cadence is "windows elapsed vs releases published"**, not `count % 10 == 0`: the hourly
  automation pushes *batches* (13 commits at once is ordinary), so an exact multiple can be stepped
  straight over. Comparing against the registry means a skipped window is picked up by the next push
  instead of being lost.
- **crates.io is the version store.** The workflow never commits its version bump back — that
  commit would advance the counter that triggered the release. Consequence: the repo's working
  version stays `0.1.0` and the registry carries `0.1.1`, `0.1.2`, …; a re-run of the same commit
  cannot double-publish, and a run that published the leaf crate before dying publishes only the
  missing ones next time.

## The gate (why nothing publishes red)

crates.io allows yank, never delete, and every published version is permanent. The `gate` job runs
before the upload: ledger + citation checks, `cargo test --workspace`, and a wasm32 check of all
three published crates. The docs-app build is not repeated there — `deploy-docs-app.yml` already
builds and deploys it on every push to this branch.

## Manual paths

```bash
node release/release-state.mjs            # human summary: is a release due, and which version
node release/release-state.mjs --json     # machine-readable plan
node release/set-version.mjs 0.1.7        # stamp a version into every manifest that must agree
gh workflow run publish-crates.yml --ref migration-to-rust -f force=true      # release now
gh workflow run publish-crates.yml --ref migration-to-rust -f dry-run=true    # plan + gate only
```

## Known limits

- The first publish of each crate is the only true end-to-end proof: `cargo publish --dry-run` on
  the two dependent crates cannot pass until the leaf crate exists on the registry (a version
  requirement cannot resolve against a crate that is not published yet).
- One release per push, never a catch-up burst: if the automation is off for a week, the *next* push
  publishes one version, not fifty.
