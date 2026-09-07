# base-ui sandbox (Docker + Nix)

Isolated dev/test environment for the `migration-to-rust` branch. Nothing
here touches your host machine — Nix, Rust, Node/pnpm, Doom Emacs, and
Chromium all live inside the container's own filesystem and named Docker
volumes. Only the repo itself is shared, via a live bind mount.

## First build

Requires `OPENROUTER_API_KEY` in your host shell (or a `docker/.env` file
with `OPENROUTER_API_KEY=sk-or-...`) — it's passed through to Claude Code
and opencode inside the container, both configured to run against
OpenRouter (see "Agents" below).

```sh
docker compose -f docker/docker-compose.yml up -d --build
```

## Attach

Either works; SSH lands you in the Nix devShell automatically (via the
`dev` user's login shell), `docker exec` needs `nix develop` run manually
or already has the toolchain on `PATH` via the image's `ENV`.

```sh
# SSH (key-based, uses ~/.ssh/id_ed25519 — no password auth is configured)
ssh dev@localhost -p 2222

# or docker exec
docker compose -f docker/docker-compose.yml exec devenv bash
```

```sh
# then, e.g.:
tmux new -s dev
emacs -nw
```

## First-boot step: install Chromium for Playwright

Not baked into the image — it depends on the bind-mounted repo's exact
`@playwright/test` version (currently `1.62.1`), so run this once per
container (cached afterwards in the `playwright-cache` volume):

```sh
docker compose -f docker/docker-compose.yml exec devenv bash -lc \
  "cd /workspace && pnpm exec playwright install chromium && pnpm exec playwright install-deps chromium"
```

## Agents (Claude Code + opencode, via OpenRouter)

Both CLIs are pre-installed and on `PATH` for both SSH and `docker exec`
sessions, wired to OpenRouter using your `OPENROUTER_API_KEY`:

- **Claude Code** (`claude`) — talks to OpenRouter's Anthropic-compatible
  endpoint (`ANTHROPIC_BASE_URL=https://openrouter.ai/api`), model set via
  `ANTHROPIC_MODEL` (default `anthropic/claude-sonnet-4.5`).
- **opencode** (`opencode`) — uses its native `openrouter` provider,
  default model `openrouter/z-ai/glm-5.2:free` (see `docker/opencode.json`).

```sh
docker compose -f docker/docker-compose.yml exec devenv bash -lc "claude"
docker compose -f docker/docker-compose.yml exec devenv bash -lc "opencode"
```

Session/auth persistence: `~/.claude` (Claude Code's session transcripts,
enabling `claude --continue`/`--resume`), `~/.local/share/opencode`, and
`~/.config/opencode` are named volumes (`claude-home`, `opencode-data`,
`opencode-config`) — they survive `docker rm`/rebuild, so an in-progress
session isn't lost when the container is recreated. Only `docker compose
down -v` wipes them.

SSH sessions don't inherit Docker's container-level environment (only
`docker compose exec` does) — `entrypoint.sh` works around this by writing
the OpenRouter/Anthropic vars into `~/.ssh/environment` on every container
start (`PermitUserEnvironment yes` in `sshd_config`), so `ssh dev@localhost`
sees them too.

## Sanity checks

```sh
docker compose -f docker/docker-compose.yml exec devenv bash -lc "node -v && pnpm -v"
docker compose -f docker/docker-compose.yml exec devenv bash -lc "cd /workspace && pnpm install"
docker compose -f docker/docker-compose.yml exec devenv bash -lc "cd /workspace && pnpm test:chromium NumberField --no-watch"
docker compose -f docker/docker-compose.yml exec devenv bash -lc "rustc --version && cargo --version && trunk --version && cargo-leptos --version"
```

## Notes

- `flake.lock` isn't committed yet — the first `nix develop` (run during
  image build) generates it automatically. If you want it reproducible
  across rebuilds, copy it out with
  `docker compose -f docker/docker-compose.yml exec devenv cat /home/dev/devenv/flake.lock > docker/flake.lock`
  and add a `COPY flake.lock` line to the Dockerfile alongside `flake.nix`.
- Named volumes (`nix-store`, `pnpm-store`, `cargo-registry`, `cargo-git`,
  `doom-local`, `doom-cache`, `playwright-cache`, `claude-home`,
  `opencode-data`, `opencode-config`) persist across `docker rm`/rebuild.
  To fully reset, `docker compose down -v`.
- Emacs is terminal-only (`emacs -nw`) — no X11/XQuartz setup needed.
- SSH is key-based only (your `~/.ssh/id_ed25519.pub`, baked in at
  `docker/dev_authorized_keys`); password and root login are disabled in
  `docker/sshd_config`. Port 2222 on the host maps to 22 in the container.
- Starship prompt is installed and wired into `~/.bashrc` for both SSH and
  `docker exec` sessions.
