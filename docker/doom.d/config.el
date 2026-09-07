;;; config.el -*- lexical-binding: t; -*-

;; Point lsp-mode at the Nix devShell's binaries so it never tries to
;; auto-download language servers inside the sandbox.

(after! lsp-mode
  (setq lsp-rust-analyzer-server-command '("rust-analyzer")
        lsp-clients-typescript-server-command
        '("typescript-language-server" "--stdio")))

(after! rustic
  (setq rustic-lsp-client 'lsp-mode))

;; The container has no display server — keep everything terminal-friendly.
(setq doom-theme 'doom-one)
(setq display-line-numbers-type 'relative)
