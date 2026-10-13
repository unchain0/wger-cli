# MCP server

MCP server lets an MCP client speak JSON-RPC 2.0 over stdio with `wger mcp`, discover seven tools, and call them with the same auth rules as the CLI.

## Sub-features

- `mcp-initialize` answers `initialize` with `serverInfo.name = wger`.
- `mcp-tools-list` returns exactly the seven tool names.
- `mcp-tools-call-read` can call `wger_list` / `wger_get` for public resources without a token.
- `mcp-tools-call-write-guard` surfaces errors for writes without a token or deletes without `confirm: true`.

## How to get to it (user POV)

- Run `wger mcp` and attach a client to its stdin/stdout (Cursor/Claude MCP config `command` + `args: ["mcp"]`).
- Send newline-delimited JSON-RPC messages (one JSON object per line).

## Driving it with wger

Preconditions:

- Doctor ok.
- Prefer the shipped helper so the protocol framing stays correct.

- **Smoke tools/list.** Run `bash .cursor/skills/verify-wger-cli/scripts/mcp-smoke.sh`. Exit `0`. Stdout contains `mcp-smoke: ok` and lists:
  - `wger_list`
  - `wger_get`
  - `wger_create`
  - `wger_update`
  - `wger_delete`
  - `wger_routine_view`
  - `wger_translation_set`
- **Manual stdio (optional).** Spawn `"$WGER_BIN" mcp`. Write one JSON object per line:
  1. `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"verify","version":"0"}}}`
  2. `{"jsonrpc":"2.0","method":"notifications/initialized"}` (no reply)
  3. `{"jsonrpc":"2.0","id":2,"method":"tools/list"}`
  Close stdin. Read stdout lines as JSON. Assert `serverInfo.name` and the seven names.
- **Public tools/call (optional live).** After initialize, send `tools/call` with `name: wger_list` and `arguments: {"resource":"language","query":{"limit":"1"}}`. Result content text parses as JSON with `results`.
- **Proof.** Save helper output to `.verify-evidence/<ts>/mcp-smoke.txt`.

### token-gated - not run (authenticated tools/call writes)

When a token is present in the environment of the `wger mcp` process:

1. Start MCP with `WGER_API_TOKEN` set in env (not in argv).
2. `tools/call` `wger_create` / `wger_update` / `wger_translation_set` only against throwaway objects.
3. `wger_delete` requires `confirm: true`.
4. Delete throwaways before exit.
5. Without confirm, delete returns `isError: true` in the tool result.

Without a token, write calls return tool errors mentioning `WGER_API_TOKEN`; do not treat that as a protocol failure.

## Gotchas

- Framing is **newline-delimited JSON**, not LSP Content-Length headers. A client that speaks only Content-Length will hang.
- `notifications/initialized` must not expect a response line.
- Draining stdout incompletely while the process still writes can surface `Broken pipe` on the server stderr; the smoke helper reads until the process exits.
- Tool schemas enum the same resource list as the CLI. A drift between CLI `RESOURCES` and MCP enum is a product bug.
