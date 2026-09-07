;;; init.el -*- lexical-binding: t; -*-

;; Starter Doom config for the base-ui Rust/Leptos migration sandbox.
;; Terminal-only (no GUI modules) — the container has no display server.

(doom! :input

       :completion
       (corfu +orderless)
       (vertico +icons)

       :ui
       doom
       doom-dashboard
       modeline
       ophints
       (popup +defaults)
       vc-gutter
       vi-tilde-fringe
       workspaces

       :editor
       (evil +everywhere)
       file-templates
       fold
       multiple-cursors
       snippets

       :emacs
       dired
       electric
       undo
       vc

       :checkers
       syntax

       :tools
       (eval +overlay)
       (lookup +dictionary)
       lsp
       magit

       :lang
       (rust +lsp)
       (web +lsp)
       (javascript +lsp)
       markdown
       nix
       yaml
       sh

       :config
       (default +bindings +smartparens))
