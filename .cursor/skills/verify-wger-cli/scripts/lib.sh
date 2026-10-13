#!/usr/bin/env bash
# Shared path/binary resolution for verify-wger-cli helpers.
set -euo pipefail

_VERIFY_SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFY_SKILL_DIR="$(cd "$_VERIFY_SCRIPTS_DIR/.." && pwd)"
REPO_ROOT="$(cd "$_VERIFY_SCRIPTS_DIR/../../../.." && pwd)"

resolve_wger_bin() {
  if [[ -n "${WGER_BIN:-}" && -x "$WGER_BIN" ]]; then
    printf '%s\n' "$WGER_BIN"
    return 0
  fi
  if [[ -x "$REPO_ROOT/target/release/wger" ]]; then
    printf '%s\n' "$REPO_ROOT/target/release/wger"
    return 0
  fi
  if [[ -x "$REPO_ROOT/target/debug/wger" ]]; then
    printf '%s\n' "$REPO_ROOT/target/debug/wger"
    return 0
  fi
  if command -v wger >/dev/null 2>&1; then
    command -v wger
    return 0
  fi
  echo "resolve_wger_bin: no wger binary (build with cargo build --release, or set WGER_BIN)" >&2
  return 1
}
