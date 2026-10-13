# wger-cli

Single Rust binary `wger`: a CLI (`--json` for compact output) and an MCP stdio server (`wger mcp`) for the [wger](https://wger.de) REST API v2.

## Auth
- `WGER_API_TOKEN` — API key from wger (Settings → API key). Never logged or stored by the tool. Reads of public data (exercises, languages) work without it; writes fail fast without it.
- `WGER_BASE_URL` — optional, default `https://wger.de`.

## Commands
```
wger resources                                   # allowed resources
wger list <resource> [-q key=value]... [--all]   # GET list (filters per wger API)
wger get <resource> <id>
wger create <resource> --data '<json>'|@file.json  # POST
wger update <resource> <id> --data '<json>'        # PATCH
wger delete <resource> <id> --yes                  # refuses without --yes
wger routine <id> [structure|logs|stats|date-sequence-display|date-sequence-gym]
wger logs [--routine N] [--exercise N] [--since YYYY-MM-DD] [--all]
wger translation --exercise N [--lang pt] [--name ..] [--description HTML | --description-file f] [--license-author ..]
wger mcp                                          # MCP server over stdio
```
Resources: routine, day, slot, slot-entry, workoutlog, workoutsession, sets-config, max-sets-config, repetitions-config, max-repetitions-config, weight-config, max-weight-config, rir-config, max-rir-config, rest-config, max-rest-config, exercise, exerciseinfo, exercise-translation, language, muscle, equipment, exercisecategory.

Config objects (`*-config`) take `{"slot_entry": ID, "iteration": 1, "value": "10", "operation": "r", "step": "abs", "repeat": true}`.

## MCP
Tools: `wger_list`, `wger_get`, `wger_create`, `wger_update`, `wger_delete` (needs `confirm: true`), `wger_routine_view`, `wger_translation_set`.
```json
{"mcpServers":{"wger":{"command":"/home/box/.local/bin/wger","args":["mcp"],"env":{"WGER_API_TOKEN":"<token>"}}}}
```
