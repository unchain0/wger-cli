# wger-cli verification map

This directory is the maintained source for verifying user-facing behavior of the `wger` CLI and `wger mcp` stdio server. Read this index before driving, then use the matching feature file as the recipe.

## Baseline preconditions

- Build once: `cargo build --release` from the repo root.
- Set `WGER_BIN` to `$PWD/target/release/wger` (or rely on helper resolution).
- Run `bash .cursor/skills/verify-wger-cli/scripts/doctor.sh` and require `doctor: ok`.
- Default API base is `https://wger.de`. Override only with `WGER_BASE_URL` when pointing at a disposable instance.
- Do not set `WGER_API_TOKEN` unless you intend token-gated drives. Never print the token.
- Never drive writes against the live API without a token and a throwaway + delete plan.

## Driving conventions

- Start every recipe from a doctor-ok binary unless its preconditions say otherwise.
- Prefer `"$WGER_BIN" --json …` for assertions. Pretty-print is the default without `--json`.
- Treat every command as literal. Keep `-q key=value` pairs and resource names unchanged.
- Resource names come from `"$WGER_BIN" --json resources` (and `wger_cli::RESOURCES`).
- MCP drives go through `scripts/mcp-smoke.sh` or an equivalent stdio JSON-RPC session against `"$WGER_BIN" mcp`.
- Restore or delete any created API objects after a mutation. Do not remove proof artifacts during cleanup.

## Proof and skip reporting

- Capture the command, stdout, stderr, and exit code under `.verify-evidence/<timestamp>/`.
- Mutation proof includes a second read of the stored value, then delete, then a failed get.
- Record the feature ID with every artifact.
- Report an unreachable path with the attempted command and the unmet precondition (usually missing token).
- Label skipped token-gated work `token-gated - not run` with the exact procedure for when a token exists.
- Do not report a skipped entry point as verified through a different path.

## Feature entry contract

Each feature file starts with an H1 title and one paragraph describing the user-visible behavior. It then uses exactly four H2 sections in this order.

1. `Sub-features` lists short IDs with one line for each behavior.
2. `How to get to it (user POV)` lists every user entry point.
3. `Driving it with wger` starts with `Preconditions:` and uses labeled bullets that pair each user action with an exact command and observable result.
4. `Gotchas` lists traps that can waste or invalidate a verification run.

Keep implementation details out of the map. Name only user paths, stable handles, required state, commands, and observable proof.

## Features

- [Exercises read](./exercises-read.md) covers public catalog list/get for exerciseinfo, exercise, and muscle filters.
- [Workout logs](./workout-logs.md) covers `wger logs` and `list workoutlog` (token-gated on wger.de).
- [Routine edit](./routine-edit.md) covers routine views and create/update/delete of routine graph objects (token-gated).
- [Translations](./translations.md) covers language lookup and exercise-translation read/write.
- [MCP server](./mcp-server.md) covers stdio initialize and tools/list for the seven tools.
