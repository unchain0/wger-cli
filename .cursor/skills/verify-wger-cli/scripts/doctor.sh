#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=lib.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

fail() {
  echo "doctor: not ready: $*" >&2
  exit 1
}

BIN="$(resolve_wger_bin)" || fail "binary missing"

VER_OUT="$("$BIN" --version 2>&1)" || fail " --version failed"
[[ "$VER_OUT" =~ ^wger\ [0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "unexpected version output: $VER_OUT"

RES_OUT="$("$BIN" --json resources 2>&1)" || fail "resources failed"
python3 - "$RES_OUT" <<'PY' || fail "resources JSON missing required names"
import json, sys
need = {"exercise", "exerciseinfo", "workoutlog", "routine", "exercise-translation"}
try:
    data = json.loads(sys.argv[1])
except Exception as e:
    raise SystemExit(f"parse: {e}")
if not isinstance(data, list):
    raise SystemExit("resources is not a list")
have = set(data)
missing = sorted(need - have)
if missing:
    raise SystemExit("missing " + ",".join(missing))
PY

# Public live read without credentials; doctor fails closed if the network/API is down.
LANG_OUT="$(env -u WGER_API_TOKEN "$BIN" --json list language -q limit=1 2>&1)" || fail "public list language failed: $LANG_OUT"
echo "$LANG_OUT" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert "results" in d and isinstance(d["results"], list)' \
  || fail "language list shape unexpected"

# Write without token must fail fast. Clear token for this check only.
set +e
WRITE_ERR="$(env -u WGER_API_TOKEN "$BIN" create day --data '{}' 2>&1)"
WRITE_EC=$?
set -e
[[ "$WRITE_EC" -ne 0 ]] || fail "create without token unexpectedly succeeded"
echo "$WRITE_ERR" | grep -q 'WGER_API_TOKEN is not set' || fail "create-without-token stderr missing fail-fast hint: $WRITE_ERR"
# Reject Authorization header echoes or `Token <secret>` values. Allow the env var name WGER_API_TOKEN.
if echo "$WRITE_ERR" | grep -qE 'Authorization:|[[:space:]]Token[[:space:]]+[^[:space:]]'; then
  fail "stderr looks like it leaked a token: $WRITE_ERR"
fi

echo "doctor: ok bin=$BIN version=${VER_OUT#wger }"
