# Appended to ~/.bashrc for the `dev` user.
if [ -f ~/.bashrc.orig ]; then . ~/.bashrc.orig; fi

export STARSHIP_CONFIG="$HOME/.config/starship.toml"
eval "$(starship init bash)"

# Quality-of-life aliases
alias ll='ls -lah --color=auto'
alias la='ls -A --color=auto'
alias ..='cd ..'
alias gs='git status'
alias gd='git diff'
alias gl='git log --oneline --graph --decorate -20'

# Bigger, deduped history shared across sessions
HISTSIZE=10000
HISTFILESIZE=20000
HISTCONTROL=ignoreboth:erasedups
shopt -s histappend

export EDITOR="emacs -nw"

cd /workspace 2>/dev/null || true
