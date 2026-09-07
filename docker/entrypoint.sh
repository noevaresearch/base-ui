#!/usr/bin/env bash
set -e

# sshd (UsePAM no) doesn't see this container's env, unlike
# `docker compose exec` sessions — mirror the OpenRouter/Claude Code vars
# into ~/.ssh/environment (requires PermitUserEnvironment yes, see
# sshd_config) so `ssh dev@localhost` picks them up too. Regenerated on
# every container start, so a rotated OPENROUTER_API_KEY takes effect on
# the next `docker compose up`.
mkdir -p /home/dev/.ssh
cat > /home/dev/.ssh/environment <<EOF
OPENROUTER_API_KEY=${OPENROUTER_API_KEY}
ANTHROPIC_BASE_URL=${ANTHROPIC_BASE_URL}
ANTHROPIC_AUTH_TOKEN=${ANTHROPIC_AUTH_TOKEN}
ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
ANTHROPIC_MODEL=${ANTHROPIC_MODEL}
EOF
chown dev:dev /home/dev/.ssh/environment
chmod 600 /home/dev/.ssh/environment

sudo /usr/sbin/sshd
exec nix develop /home/dev/devenv --command sleep infinity
