# The browser sandbox: plan (pre-implementation)

Status: **decisions cleared; Phase 0 in flight — no sandbox artifact built yet.** Every claim about a
host's capability below was checked against that host's own docs (cited) rather than recalled.

## 0. Decisions taken 2026-09-16

* **Host: CodeSandbox only** ("Codesandbox only"). The StackBlitz showcase (§3b) is dropped from
  scope — it is documented in this file only so the reason is not re-litigated: that host cannot
  compile Rust, so its editor would have to lie. Nothing of §3b gets built unless it is asked for.
* **First cut: one vertical slice end-to-end** — a single demo (accordion hero) working in a
  CodeSandbox VM sandbox, plus the docs control that opens it. Gallery-scale comes after, not before.
* **Ownership: the sandbox is part of the loop's end goal**, because the loop is what produces the
  ported demos. That sets the contract in §3a1: a demo is not fully delivered until it is also
  reachable in the sandbox, and the ledger carries unregistered demos as visible debt.
* **Verification: a CodeSandbox API token** (codesandbox.io/t/api) is supplied by the owner, so
  sandboxes can be created and inspected through the SDK instead of hand-clicked — a green that
  comes from a real VM boot, not from a self-report.
* **Vertical slice is unblocked today:** `base-ui-leptos 0.1.1` (published 2026-09-16 09:23 UTC) is
  the only crates.io version and it already exports the namespaced surface the accordion demo needs
  — `pub use self::accordion as Accordion` with `Root`/`Item`/`Header`/`Trigger`/`Panel` in
  `src/accordion/mod.rs` — verified by unpacking the `.crate` tarball, not assumed. No release is
  required before the sandbox can depend on a published version.


## 1. The objective, stated in terms of the thing it has to match

"Like the base-ui website" has a concrete referent in the upstream tree, not a vibe:

* every demo on base-ui.com renders inside a toolbar with **"Open in StackBlitz"** and a
  CodeSandbox fallback — `docs/src/components/Demo/Demo.tsx:154-169` (`GhostButton
  aria-label="Open in StackBlitz"`), the payload built by
  `docs/src/blocks/createCodeSandbox/createStackBlitzProject.ts` (a client-side `POST` form to
  `https://stackblitz.com/run`), dependency list from `docs/src/utils/demoExportOptions.ts`;
* the visitor lands in an online IDE holding the demo's **editable source**, and edits recompile
  and re-render in the browser. That loop — edit → rebuild → live result — is the feature.

So the deliverable is not "a link". It is: **a visitor can open one of this port's demos in an
online editor, change the Rust, and see the ported component change.**

## 2. Premise audit — which host can actually do that

| Host | Can it compile Rust? | Evidence |
|---|---|---|
| StackBlitz (WebContainers) | **No.** Executes JS and WASM only; a `cargo`/`rustc` toolchain does not exist in it. | WebContainers troubleshooting doc: "WebContainers can only execute languages that are natively supported on the Web, including JavaScript and WebAssembly… it is not possible to run native addons … unless they can be compiled to WebAssembly" (compilation happens *before* upload; the new WASI layer adds a `wasm` **run** command, not a compiler). The WASI announcement's own wording is "compile your native code to WASI, upload it to your project, and run it via the `wasm` command". |
| StackBlitz POST API | No, and it also **cannot carry the binary**: "Binary files (such as archives or non-SVG images) are not supported for the dynamically created projects." Templates offered: `typescript, angular-cli, create-react-app, javascript` (SDK reference adds `html`, `vue`, `polymer`, `node`). | developer.stackblitz.com/platform/api/post-api |
| CodeSandbox (VM sandbox / "Devbox") | **Yes.** Firecracker microVM, any Dev Container image, real terminal, real `cargo`, port-forwarded preview. | codesandbox.io/docs/projects/learn/setting-up/env, /docs/tutorial/getting-started-with-dev-containers |
| CodeSandbox (browser sandbox / Sandpack) | No — in-browser bundler, JS only. Upstream's fallback export uses it; we must not. | same docs, EngineBlock/browser vs VM split |
| In-browser `rustc` (weblings, rubrc, `browser_wasi_shim`) | Runs `rustc` in WASM — impressive and real — but **no `cargo` resolution yet**, no wasm-bindgen post-processing, COOP/COEP requirements, self-described "not ready for general use". | github.com/AngelOnFira/weblings, github.com/oligamiq/rubrc |

**Consequence, stated plainly:** the StackBlitz button can be made to *run* a demo, but it cannot
compile one. On that host the Rust source in the editor would be a read-only mirror of the code
that built the prebuilt `.wasm`; editing it does nothing. Shipping that while calling it "like the
base-ui website" would be teaching a falsehood about the port, so if StackBlitz is in scope it goes
in as an explicitly-labelled *runnable showcase*, not as the developer loop.

CodeSandbox VM sandboxes are the only browser-editor path where the loop is real. And the thing
that makes it possible is exactly what publishing buys: the sandbox's `Cargo.toml` names
`base-ui-leptos = "0.1"` and cargo resolves it **from crates.io**, so the visitor edits real Rust
against the published crate and a real rustc rebuilds it. That is the coherent version of
"we publish the crate so there can be a demo" — and it makes the publication itself testable
(§6, gate 4).

## 3. Recommended architecture

### 3a. Primary: CodeSandbox synced template (the developer loop)

* A new crate `sandbox` — a CSR Leptos app (`leptos/csr` + `wasm-bindgen`, `cargo leptos serve` or
  `trunk serve`) with a demo switcher: one demo mounted at a time, `?demo=<slug>` deep-links it.
  It depends on the **published** `base-ui-leptos` by version, not by path.
* `.devcontainer/devcontainer.json` — Rust toolchain + `wasm32-unknown-unknown` +
  `cargo-leptos`/`trunk` + `wasm-bindgen-cli` + a pre-warmed cargo registry, so the first boot is
  a build, not a toolchain install.
* `.codesandbox/tasks.json` — `setupTasks` prebuilds, the dev server runs at start and its port is
  declared as the preview, so the sandbox opens **on a running port`.
* `.devcontainer/devcontainer.json` in the repo (or a `.codesandbox/` config) is also what forces
  CodeSandbox to load the project as a VM sandbox rather than a browser sandbox.
* The docs get an "Open in CodeSandbox" control next to each demo, linking to the synced-template
  URL for that demo's slug. No API token on the page, no server of ours in the loop; a visitor
  forks into **their own** workspace and VM quota.
* **The sandbox must not live on `migration-to-rust`.** CodeSandbox's synced templates discard
  their memory snapshot on *every* commit to the backing repo/branch ("if you create a new commit
  in your repository, we will discard the memory snapshot … and start the VM Sandbox from scratch
  on next visit"). The Ralph loop commits every ~15 minutes; pointing the sandbox at that branch
  would make every visit a cold boot. Default: a small dedicated repo (e.g.
  `noevaresearch/base-ui-leptos-sandbox`) that only changes on release. (Decision Q3.)

### 3b. Secondary (optional): StackBlitz runnable showcase (the instant, no-login demo)

Prebuilt `.wasm` per release — built in CI from the same published crate — published as static
assets on the existing `baseui.noevaresearch.com` Worker (`/sandbox/…`), with a `_headers` file
adding `Access-Control-Allow-Origin: *` for that path (Workers static assets support `_headers`;
custom headers do not apply to worker-generated responses, so this is the only supported way) and
the `application/wasm` MIME the streaming loader requires (the existing deploy already smoke-tests
that MIME for `/pkg/docs-app.wasm`).

The generated StackBlitz project is therefore **all text** (binary files are rejected by the POST
API, §2): `index.html`, `main.js` (imports the wasm glue from our CDN and mounts the demo),
`src/main.rs` (the demo's Rust, verbatim, for reading and copying), `Cargo.toml`
(`base-ui-leptos = "0.1"`, so copying it out and building locally works), and a `README.md` that
says what is prebuilt and why. Mirrors upstream's client-side `POST` form; the button is ported
into the port's own demo chrome, same as `code_block.rs` ported the code-block panel.

## 4. Phases, with the gate that ends each one

| Phase | Work | Gate (real, not self-reported) |
|---|---|---|
| 0. Spike | Minimal CSR Leptos app building in the box; measure build time + peak RSS; open it as a CodeSandbox VM sandbox | Sandbox URL loads in a **real browser** with the dev server up and the demo interactive; recorded build time and tier used |
| 1. Sandbox app | `sandbox` crate + demo switcher + `?demo=` routing | `cargo leptos build` green; every registered demo mounts in the browser, same interactions as the docs page |
| 2. CSB template | devcontainer + tasks.json + synced template + docs link | From a clean browser profile: open link → preview live → edit a demo's `.rs` → save → component changes |
| 3. StackBlitz showcase | CI wasm artifact + Worker `/sandbox/*` + `_headers` + POST-form button in the docs chrome + tests | Generated project opened from this box's Chromium (stackblitz.com is reachable: HTTP 200) shows the demo interactive; artifact MIME/CORS asserted by the deploy smoke test |
| 4. Release gate (optional) | The release watchdog builds the sandbox against the version currently on crates.io | Fails loudly if the published version no longer builds the sandbox |

## 5. Risks / honest unknowns

* **Verification from this box has one hard wall.** `codesandbox.io` serves a Cloudflare bot
  challenge to headless Chromium from this host (observed: "Just a moment…" body), and VM sandboxes
  need a signed-in workspace regardless. So Phase 0/2 verification needs either a CodeSandbox API
  token (the SDK can create and inspect sandboxes) or your click. StackBlitz has no such wall
  (HTTP 200, no challenge) and I can verify Phase 3 myself.
* Demo coverage is not uniform: 18 demos are already standalone `#[component] pub fn *Demo`
  functions in `crates/docs-app/src/pages/`; the remaining pages hold their demos inline and would
  need extraction before the switcher can list them. First cut covers the 18; the rest get ledger
  entries, never silence.
* VM tier: Nano (2 vCPU / 4 GiB) is the free tier (400 credits ≈ 40 h/month, 10 concurrent VMs);
  a cold `leptos` release build may want more, and a template's tier cannot be downsized later.
  Phase 0 measures this instead of guessing.
* Free-tier quota is per visitor workspace, so a popular docs link does not bill us.

## 6. What "done" means for the whole thing

A reader on `baseui.noevaresearch.com`, on a docs page for a ported component, clicks one control
and lands in an editor where the ported component is running and their edit to the Rust changes
what they see — with the dependency resolved from the published crate, and the docs page itself
unchanged in every existing parity gate.
