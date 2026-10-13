#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=lib.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

# Remove scratch bodies created for verification writes. Never touch evidence dirs.
removed=0
for d in /tmp/wger-verify-scratch*; do
  if [[ -e "$d" ]]; then
    rm -rf "$d"
    removed=$((removed + 1))
  fi
done

# Evidence must survive.
for keep in "$REPO_ROOT/.verify-evidence" /tmp/verify-wger-cli-evidence; do
  if [[ -d "$keep" ]]; then
    echo "cleanup: keeping evidence at $keep"
  fi
done

echo "cleanup: ok scratch_removed=$removed"
echo "cleanup: note: does not pkill wger; MCP children exit when their helper closes stdin"
