#!/usr/bin/env bash
# Agent-Slate macOS/Linux bootstrap.
# In a clone:  ./install.sh --grok
# From the internet:
#   curl -fsSL https://raw.githubusercontent.com/thcabq352/Agent-Slate/main/install.sh | bash -s -- --grok
#
# This is the Agent-Slate fork installer (from source). Upstream Slate's macOS
# .app zip lives at wassermanproductions/slate — do not mix the two.

set -euo pipefail

REPO_URL="${AGENT_SLATE_REPO:-https://github.com/thcabq352/Agent-Slate.git}"
DEST="${AGENT_SLATE_DIR:-}"

in_repo() {
  [ -f package.json ] && grep -q '"name": "agent-slate"' package.json
}

if in_repo; then
  ROOT="$(pwd)"
else
  DEST="${DEST:-$HOME/Agent-Slate}"
  if [ ! -f "$DEST/package.json" ]; then
    if ! command -v git >/dev/null 2>&1; then
      echo "git is required to clone Agent-Slate." >&2
      exit 1
    fi
    echo "→ cloning $REPO_URL to $DEST"
    git clone "$REPO_URL" "$DEST"
  fi
  ROOT="$DEST"
  cd "$ROOT"
fi

exec bash "$ROOT/scripts/setup.sh" "$@"
