# base-ui Rust/Leptos migration sandbox
#
# Isolated dev/test environment: Ubuntu + Nix (rustc/cargo/wasm/trunk) +
# Node/pnpm (via Corepack, matching the repo's exact engine pins) +
# Doom Emacs (terminal-only) + Starship prompt + SSH (key-based) +
# Chromium (via Playwright's own installer, run against the bind-mounted
# repo at first boot — see docker/README.md).
#
# Nothing here touches the host: Nix store, pnpm store, cargo registry and
# Doom's package dir all live in named Docker volumes (see docker-compose.yml).
# The only thing that crosses the host boundary is the repo itself, bind-mounted
# read/write at /workspace, plus your SSH public key baked in read-only.

FROM ubuntu:24.04

ARG USERNAME=dev
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
      curl \
      git \
      ca-certificates \
      xz-utils \
      sudo \
      build-essential \
      pkg-config \
      tmux \
      locales \
      openssh-server \
      ncurses-term \
    && rm -rf /var/lib/apt/lists/* \
    && locale-gen en_US.UTF-8

ENV LANG=en_US.UTF-8 \
    LANGUAGE=en_US:en \
    LC_ALL=en_US.UTF-8

RUN useradd -m -s /bin/bash ${USERNAME} \
    && echo "${USERNAME} ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers \
    && passwd -d ${USERNAME}
    # `useradd -m` locks the account (shadow field `!`) since no password was
    # set — sshd refuses even pubkey auth for a locked account regardless of
    # UsePAM. `passwd -d` clears the password field without locking it;
    # PasswordAuthentication no in sshd_config still blocks password login.

# --- SSH server: key-based only, no root/password login ---
RUN mkdir -p /run/sshd && ssh-keygen -A
COPY sshd_config /etc/ssh/sshd_config
RUN mkdir -p /home/${USERNAME}/.ssh
COPY dev_authorized_keys /home/${USERNAME}/.ssh/authorized_keys
RUN chown -R ${USERNAME}:${USERNAME} /home/${USERNAME}/.ssh \
    && chmod 700 /home/${USERNAME}/.ssh \
    && chmod 600 /home/${USERNAME}/.ssh/authorized_keys

# --- Starship prompt (single static binary, installed as root) ---
RUN curl -sS https://starship.rs/install.sh | sh -s -- --yes

USER ${USERNAME}
WORKDIR /home/${USERNAME}

# --- Nix: classic single-user install (no daemon), flakes enabled ---
# (Determinate's installer's --init none still requires a running nix-daemon
# behind a socket, which doesn't persist across Dockerfile RUN layers; the
# classic installer's --no-daemon mode needs no daemon at all.)
RUN curl -L https://nixos.org/nix/install | sh -s -- --no-daemon
ENV PATH="/home/${USERNAME}/.nix-profile/bin:${PATH}"
ENV NIX_CONFIG="experimental-features = nix-command flakes"

# --- devShell packages (rustc/cargo/wasm32/trunk/cargo-leptos/node/emacs/...) ---
COPY --chown=${USERNAME}:${USERNAME} flake.nix /home/${USERNAME}/devenv/flake.nix
RUN cd /home/${USERNAME}/devenv \
    && nix develop --extra-experimental-features "nix-command flakes" --command true

# --- Node/pnpm: Corepack activates the exact pin from the repo's package.json ---
# `corepack enable` normally symlinks shims next to the `node` binary itself,
# which is read-only under Nix's immutable store — point it at a writable dir.
RUN mkdir -p /home/${USERNAME}/.local/bin
ENV PATH="/home/${USERNAME}/.local/bin:${PATH}"
# Node 22.10.0's bundled Corepack predates a signing-key rotation on the npm
# registry, so it rejects pnpm 11.21.0's package signature with "Cannot find
# matching keyid" — a stale allow-list in Corepack itself, not a real
# integrity failure. Documented workaround: disable that specific check.
ENV COREPACK_INTEGRITY_KEYS=""
RUN cd /home/${USERNAME}/devenv \
    && nix develop --command corepack enable --install-directory /home/${USERNAME}/.local/bin \
    && nix develop --command corepack prepare pnpm@11.21.0 --activate

# --- CLI agents: Claude Code + opencode, both pointed at OpenRouter ---
# npm's default global prefix sits next to the `node` binary, which is
# read-only under Nix's immutable store (same issue as Corepack above) —
# point it at a writable dir instead.
RUN mkdir -p /home/${USERNAME}/.local/share/npm-global \
    && cd /home/${USERNAME}/devenv \
    && nix develop --command npm config set prefix /home/${USERNAME}/.local/share/npm-global \
    && nix develop --command npm install -g @anthropic-ai/claude-code opencode-ai
ENV PATH="/home/${USERNAME}/.local/share/npm-global/bin:${PATH}"
# Seeds the `opencode-config` named volume on first mount (Docker copies
# existing directory content into a fresh named volume the first time it's
# mounted over it).
RUN mkdir -p /home/${USERNAME}/.config/opencode
COPY --chown=${USERNAME}:${USERNAME} opencode.json /home/${USERNAME}/.config/opencode/opencode.json
# Both are configured for OpenRouter via env vars set at container start
# (see entrypoint.sh — SSH sessions don't inherit Docker's container env,
# so the actual values are written to ~/.ssh/environment instead):
#   Claude Code : ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_TOKEN, ANTHROPIC_API_KEY=""
#   opencode    : OPENROUTER_API_KEY (its OpenRouter provider reads this directly)

# --- Doom Emacs (terminal-only; pre-baked starter config) ---
RUN git clone --depth 1 https://github.com/doomemacs/doomemacs /home/${USERNAME}/.config/emacs
COPY --chown=${USERNAME}:${USERNAME} doom.d /home/${USERNAME}/.config/doom
RUN cd /home/${USERNAME}/devenv && nix develop --command \
      /home/${USERNAME}/.config/emacs/bin/doom install --force
RUN cd /home/${USERNAME}/devenv && nix develop --command \
      /home/${USERNAME}/.config/emacs/bin/doom sync
ENV PATH="/home/${USERNAME}/.config/emacs/bin:${PATH}"

# --- Shell rc: starship prompt (custom theme) + drop into /workspace on login ---
RUN mv /home/${USERNAME}/.bashrc /home/${USERNAME}/.bashrc.orig \
    && mkdir -p /home/${USERNAME}/.config
COPY --chown=${USERNAME}:${USERNAME} starship.toml /home/${USERNAME}/.config/starship.toml
COPY --chown=${USERNAME}:${USERNAME} bashrc_snippet.sh /home/${USERNAME}/.bashrc

# --- Login shell wrapper: any session (SSH included) lands in the Nix devShell ---
# Placed near the end (after Nix/Doom are already built) so future edits to
# this script don't invalidate those expensive layers.
USER root
COPY devshell /usr/local/bin/devshell
RUN chmod +x /usr/local/bin/devshell \
    && echo "/usr/local/bin/devshell" >> /etc/shells \
    && usermod --shell /usr/local/bin/devshell ${USERNAME}
USER ${USERNAME}

WORKDIR /workspace

# Long-running container. Attach via:
#   docker compose exec devenv bash            (plain shell, no SSH)
#   ssh dev@localhost -p 2222                  (key-based, lands in devShell)
# Chromium is NOT installed at image build time — it depends on the
# bind-mounted repo's exact @playwright/test version, so it's installed
# once at first boot instead (see docker/README.md).
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
USER root
RUN chmod +x /usr/local/bin/entrypoint.sh
USER ${USERNAME}
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
