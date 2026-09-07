{
  description = "base-ui Rust/Leptos migration devenv (isolated sandbox)";

  inputs = {
    # nixos-24.05's nodejs_22 is 22.10.0 — below both pnpm 11.21.0's own
    # >=22.13 requirement and this repo's >=22.23.2 pin. nixpkgs-unstable
    # currently ships exactly 22.23.2.
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachSystem [ "aarch64-linux" "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.trunk
            pkgs.wasm-bindgen-cli
            pkgs.cargo-leptos
            pkgs.nodejs_22 # ships Corepack; pnpm 11.21.0 is activated via
                           # `corepack prepare pnpm@11.21.0 --activate`, not
                           # pinned here (see docker/Dockerfile).
            pkgs.emacs # unversioned alias — nixpkgs-unstable renames its
                       # versioned emacsNN attrs frequently as new majors ship
            pkgs.ripgrep
            pkgs.fd
            pkgs.git
          ];

          shellHook = ''
            export PLAYWRIGHT_BROWSERS_PATH="$HOME/.cache/ms-playwright"
            # Playwright's dependency check shells out to `ldconfig`, which
            # resolves to Nix's own glibc build inside this devShell (an
            # almost-empty private cache) instead of the real system one at
            # /sbin/ldconfig — it then falsely reports system libs (that are
            # genuinely present) as missing. The libraries are real; only the
            # check is wrong under Nix, so skip it rather than chase a
            # phantom dependency.
            export PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS=1
          '';
        };
      });
}
