---
name: verify-wger-cli
description: >-
  Launch and drive the wger CLI binary and MCP stdio server the way a user does,
  and capture proof. Use when a change touches wger list/get/create/update/delete,
  routine views, workout logs, exercise translations, resources, --json output,
  Token auth, or `wger mcp` tools/list and tools/call.
---

# Verify wger-cli

Primary surfaces:

- The `wger` CLI (`--json` for compact machine output).
- The MCP server over stdio (`wger mcp`).

Both talk to the wger REST API v2 (`https://wger.de/api/v2/` by default, override with `WGER_BASE_URL`). Auth is `WGER_API_TOKEN` sent as `Authorization: Token <t>`. Public catalog reads (exercise, exerciseinfo, muscle, language, equipment, exercisecategory, exercise-translation) work without a token. User data (routine, workoutlog, workoutsession, day/slot configs) and all writes need a token.

There is no long-lived local server for the CLI. Each drive is one process. The MCP server is short-lived and bound to the helper that spawned it. Two verification runs can share the public API; they must not share write side effects. Never drive writes against a live account without a disposable token and a cleanup plan.

## Launch

From the repo root:

```bash
cargo build --release
export WGER_BIN="$PWD/target/release/wger"
"$WGER_BIN" --version
```

Ready means the binary prints `wger 0.1.0` (or the version in `Cargo.toml`) and exits 0. Put `WGER_BIN` on `PATH` as `wger` if a recipe assumes the bare name:

```bash
export PATH="$(dirname "$WGER_BIN"):$PATH"
```

Do not install into a shared prefix during verification. Prefer `$WGER_BIN` or `target/release/wger` from this checkout.

Teardown for Launch is a no-op beyond optional `cargo clean` (not required). MCP processes are owned by the helper that started them.

## Doctor

Run before the first drive and after any failed drive:

```bash
bash .cursor/skills/verify-wger-cli/scripts/doctor.sh
```

Read-only. Passes only when all of these hold:

- `$WGER_BIN` (or `target/release/wger`, then `wger` on `PATH`) exists and is executable.
- `"$WGER_BIN" --version` matches `wger <semver>`.
- `"$WGER_BIN" --json resources` is a JSON array that includes `exercise`, `exerciseinfo`, `workoutlog`, `routine`, and `exercise-translation`.
- A live public read works without a token: `"$WGER_BIN" --json list language -q limit=1` exits 0 and stdout has `"results"`.
- A write without a token fails fast: `"$WGER_BIN" create day --data '{}'` exits non-zero and stderr contains `WGER_API_TOKEN` (the token value is never printed).

Passing doctor prints `doctor: ok bin=<path> version=<ver>`. Do not drive an instance that fails doctor. Do not invent flags; if a recipe command is missing from `"$WGER_BIN" <cmd> --help`, fix the recipe.

Optional unit gate (not a live proof): `cargo test` (12 tests, mockito, no live HTTP).

## Drive

Drive the real binary the way a user would. Prefer `--json` for stable assertions. Every flag and subcommand must exist in `"$WGER_BIN" --help` / subcommand `--help`.

Stable command shapes (from clap):

| User intent | Command |
| --- | --- |
| List public exercises | `wger --json list exerciseinfo -q limit=2` |
| Filter by muscle id | `wger --json list exerciseinfo -q muscles=1 -q limit=5` |
| Get one exercise | `wger --json get exerciseinfo 91` |
| List resources | `wger --json resources` |
| List languages | `wger --json list language -q short_name=pt` |
| Workout logs | `wger --json logs --routine N --since YYYY-MM-DD` (token) |
| Routine structure | `wger --json routine <id> structure` (token) |
| Create object | `wger --json create <resource> --data '<json>'` or `--data @file.json` (token) |
| Patch object | `wger --json update <resource> <id> --data '<json>'` (token) |
| Delete object | `wger delete <resource> <id> --yes` (token; refuses without `--yes`) |
| Translation write | `wger --json translation --exercise N --lang pt --name '…' --description '<p>…</p>'` (token) |
| MCP stdio | `wger mcp` |

Query flags are repeated `-q key=value` (short for `--query`). Pagination: `--all` on `list` and `logs` follows `next` up to 50 pages and returns `count`, `total`, `truncated`, `next`.

Harness for MCP:

```bash
bash .cursor/skills/verify-wger-cli/scripts/mcp-smoke.sh
```

That script spawns `"$WGER_BIN" mcp`, sends newline-delimited JSON-RPC `initialize`, `notifications/initialized`, and `tools/list`, then asserts the seven tool names. Do not hand-wave MCP with pseudo-code.

Feature recipes live under `features/`. Drive at least the feature your change touches. A one-feature proof does not cover the rest of the map.

**Token policy.** Without `WGER_API_TOKEN`, only public GETs and the no-token write fail-fast path. Never attempt live writes without a token. With a token, every mutation uses a throwaway object and deletes it before the run ends. Never print, log, or commit the token value.

## Evidence

Write proof under `.verify-evidence/<timestamp>/` at the repo root (gitignored). Alternative: `/tmp/verify-wger-cli-evidence/<timestamp>/`. Cleanup must not delete either tree.

Capture for each drive:

- The exact command (token redacted if present in the env dump).
- stdout and stderr (full for small payloads; head + `wc -c` for large lists).
- Exit code.
- For MCP: the raw JSON-RPC replies and the asserted tool name list.

Standards:

- Exercise the real user path (`wger …` or `wger mcp` stdio). Do not call `wger_cli::Client` from a one-off Rust snippet and call that a CLI proof.
- Capture the action and the resulting state (for writes: create response id, then get/list, then delete, then get 404).
- Mocks (`cargo test` / mockito) prove unit behavior only. Live proof hits `WGER_BASE_URL` (default `https://wger.de`).
- Mark token-gated journeys that were not run as `token-gated - not run` with the exact procedure for when a token exists.

Minimal first proof this skill expects:

```bash
set -o pipefail
TS=$(date -u +%Y%m%dT%H%M%SZ)
EV=.verify-evidence/$TS
mkdir -p "$EV"
bash .cursor/skills/verify-wger-cli/scripts/doctor.sh | tee "$EV/doctor.txt"
"$WGER_BIN" --json list exerciseinfo -q muscles=1 -q limit=2 >"$EV/exercises-read.json"
echo exit:$? | tee "$EV/exercises-read.exit"
bash .cursor/skills/verify-wger-cli/scripts/mcp-smoke.sh | tee "$EV/mcp-smoke.txt"
```

Redirect (or `set -o pipefail` before any `| tee`) so the recorded exit code is the CLI's, not `tee`'s.

## Cleanup

- Kill only MCP (or other) processes your helper started. Prefer the helper exiting so the child gets EOF on stdin and leaves on its own. Never `pkill wger` or kill by process name.
- Remove scratch JSON bodies under `/tmp/wger-verify-*` that you created for `--data @file`.
- If a token-gated run created API objects, delete them with `wger delete <resource> <id> --yes` and confirm with `wger get` expecting an error.
- Do **not** delete `.verify-evidence/` or `/tmp/verify-wger-cli-evidence/`.
- Unset scratch env vars you exported (`WGER_BIN` may stay for the session).

```bash
bash .cursor/skills/verify-wger-cli/scripts/cleanup.sh
```

## Helpers

| Script | Invocation |
| --- | --- |
| `scripts/lib.sh` | Sourced by the other helpers. Do not run it alone. |
| `scripts/doctor.sh` | `bash .cursor/skills/verify-wger-cli/scripts/doctor.sh` |
| `scripts/mcp-smoke.sh` | `bash .cursor/skills/verify-wger-cli/scripts/mcp-smoke.sh` |
| `scripts/cleanup.sh` | `bash .cursor/skills/verify-wger-cli/scripts/cleanup.sh` |

Helpers resolve the binary via `WGER_BIN`, else `$REPO_ROOT/target/release/wger`, else `wger` on `PATH`. They never echo secret values.

## Feature map

See [`features/README.md`](features/README.md). Keep the map honest with `/maintain-verification-skill` when commands or resources change.
