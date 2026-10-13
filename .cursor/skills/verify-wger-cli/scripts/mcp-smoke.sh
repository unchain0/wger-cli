#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=lib.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

BIN="$(resolve_wger_bin)"

# Spawn MCP over stdio, send initialize + tools/list, assert tool names.
# JSON-RPC is newline-delimited (one object per line), matching src/mcp.rs.
python3 - "$BIN" <<'PY'
import json
import subprocess
import sys

bin_path = sys.argv[1]
expected = [
    "wger_list",
    "wger_get",
    "wger_create",
    "wger_update",
    "wger_delete",
    "wger_routine_view",
    "wger_translation_set",
]

messages = [
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "verify-wger-cli", "version": "0"},
        },
    },
    {"jsonrpc": "2.0", "method": "notifications/initialized"},
    {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
]

payload = "".join(json.dumps(m, separators=(",", ":")) + "\n" for m in messages)

proc = subprocess.run(
    [bin_path, "mcp"],
    input=payload.encode(),
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    timeout=30,
)

stdout = proc.stdout.decode()
stderr = proc.stderr.decode()
if proc.returncode not in (0,):
    # EOF after messages should still exit 0; treat any non-zero as failure.
    sys.stderr.write(f"mcp-smoke: wger mcp exited {proc.returncode}\nstderr:\n{stderr}\nstdout:\n{stdout}\n")
    sys.exit(1)

replies = []
for line in stdout.splitlines():
    line = line.strip()
    if not line:
        continue
    try:
        replies.append(json.loads(line))
    except json.JSONDecodeError as e:
        sys.stderr.write(f"mcp-smoke: bad JSON line: {line!r} ({e})\n")
        sys.exit(1)

by_id = {r.get("id"): r for r in replies if "id" in r and r.get("id") is not None}
if 1 not in by_id:
    sys.stderr.write(f"mcp-smoke: missing initialize reply; got {replies!r}\n")
    sys.exit(1)
init = by_id[1]
server = init.get("result", {}).get("serverInfo", {})
if server.get("name") != "wger":
    sys.stderr.write(f"mcp-smoke: serverInfo.name want wger, got {server!r}\n")
    sys.exit(1)

if 2 not in by_id:
    sys.stderr.write(f"mcp-smoke: missing tools/list reply; got {replies!r}\n")
    sys.exit(1)
tools = by_id[2].get("result", {}).get("tools")
if not isinstance(tools, list):
    sys.stderr.write(f"mcp-smoke: tools/list result.tools not a list: {by_id[2]!r}\n")
    sys.exit(1)

names = [t.get("name") for t in tools]
if names != expected:
    sys.stderr.write(f"mcp-smoke: tool names mismatch\n  want: {expected}\n  got:  {names}\n")
    sys.exit(1)

if stderr.strip():
    # Non-fatal noise only if empty is preferred; surface unexpected stderr.
    # Broken-pipe noise should not appear when we drain cleanly.
    sys.stderr.write(f"mcp-smoke: unexpected stderr:\n{stderr}\n")
    sys.exit(1)

print(f"mcp-smoke: ok bin={bin_path} tools={len(names)} names={','.join(names)}")
print(json.dumps({"initialize": init["result"]["serverInfo"], "tools": names}, indent=2))
print("mcp-smoke: raw-stdout-begin")
sys.stdout.write(stdout)
if not stdout.endswith("\n"):
    sys.stdout.write("\n")
print("mcp-smoke: raw-stdout-end")
PY
