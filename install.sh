#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOK_SRC="$SCRIPT_DIR/hooks/post-merge"
HOOK_DST=".git/hooks/post-merge"

if [ ! -f "$HOOK_SRC" ]; then
  echo "[scaffold] ERROR: hooks/post-merge not found at $HOOK_SRC" >&2
  exit 1
fi

if [ ! -d ".git" ]; then
  echo "[scaffold] ERROR: not in a git repository root" >&2
  exit 1
fi

cp "$HOOK_SRC" "$HOOK_DST"
chmod +x "$HOOK_DST"
echo "[scaffold] Installed post-merge hook."

echo "[scaffold] Running hook now to sync skills..."
bash "$HOOK_DST"
