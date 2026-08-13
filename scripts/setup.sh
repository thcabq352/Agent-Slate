#!/usr/bin/env bash
# Agent-Slate from-source setup for macOS and Linux.
# Usage (in a clone):  ./scripts/setup.sh --grok
# Forwards flags to scripts/setup.mjs: --ffmpeg --grok --cursor --engine --skip-npm

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

need_node() {
  if command -v node >/dev/null 2>&1; then
    major="$(node -p 'process.versions.node.split(".")[0]')"
    if [ "$major" -ge 20 ]; then
      return 0
    fi
    echo "Node $(node -v) is too old — need 20+." >&2
  fi
  echo "Install Node 20+:" >&2
  if [ "$(uname -s)" = "Darwin" ]; then
    echo "  brew install node" >&2
  else
    echo "  https://nodejs.org  or  sudo apt install nodejs" >&2
  fi
  exit 1
}

need_node
exec node "$ROOT/scripts/setup.mjs" "$@"
