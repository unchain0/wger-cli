#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=lib.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

# Remove only this run's scratch dir. Set WGER_VERIFY_SCRATCH to an absolute
# path unique to the run (for example /tmp/wger-verify-scratch-$TS-$PPID).
# Never glob-delete other runs' directories.
removed=0
if [[ -n "${WGER_VERIFY_SCRATCH:-}" ]]; then
  case "$WGER_VERIFY_SCRATCH" in
    /tmp/wger-verify-scratch/*|/tmp/wger-verify-scratch-*)
      # Canonicalize to reject traversal (e.g. /tmp/wger-verify-scratch/../unrelated)
      CANON="$(realpath -m "$WGER_VERIFY_SCRATCH" 2>/dev/null || echo "$WGER_VERIFY_SCRATCH")"
      case "$CANON" in
        /tmp/wger-verify-scratch/*|/tmp/wger-verify-scratch-*)
          if [[ -e "$CANON" ]]; then
            rm -rf -- "$CANON"
            removed=1
          fi
          ;;
        *)
          echo "cleanup: refusing non-child scratch after canonicalization: $CANON" >&2
          exit 1
          ;;
      esac
      ;;
    *)
      echo "cleanup: refusing WGER_VERIFY_SCRATCH outside /tmp/wger-verify-scratch*: $WGER_VERIFY_SCRATCH" >&2
      exit 1
      ;;
  esac
fi

# Evidence must survive.
for keep in "$REPO_ROOT/.verify-evidence" /tmp/verify-wger-cli-evidence; do
  if [[ -d "$keep" ]]; then
    echo "cleanup: keeping evidence at $keep"
  fi
done

echo "cleanup: ok scratch_removed=$removed scratch=${WGER_VERIFY_SCRATCH:-}"
echo "cleanup: note: does not pkill wger; MCP children exit when their helper closes stdin"
