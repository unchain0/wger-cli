# Workout logs

Workout logs lets a user list their workout log entries with optional routine, exercise, and date filters, including paginated `--all` fetches.

## Sub-features

- `logs-shortcut` uses `wger logs` (orders by `-date`).
- `logs-list-resource` uses `wger list workoutlog` with the same filters.
- `logs-filter-routine` narrows with `--routine N` / `-q routine=N`.
- `logs-filter-since` narrows with `--since YYYY-MM-DD` / `-q date__gte=…`.
- `logs-all-pages` follows pagination with `--all`.

## How to get to it (user POV)

- Run `wger logs` with optional `--routine`, `--exercise`, `--since`, `--all`.
- Run `wger list workoutlog -q routine=N -q ordering=-date`.
- Run `wger get workoutlog <uuid>` for one log row (ids are UUIDs).

## Driving it with wger

Preconditions:

- Doctor ok.
- **Token required for live wger.de.** `list workoutlog` and `wger logs` return HTTP 403 without credentials.

### token-gated - not run (no token in this environment)

When `WGER_API_TOKEN` is available (never print it):

1. Export the token only in the process environment: `export WGER_API_TOKEN` from a local secret source; do not pass it on the CLI argv.
2. **List recent.** Run `"$WGER_BIN" --json logs --all` or `"$WGER_BIN" --json logs` without `--all`. Exit `0`. JSON has `results` (array). Empty results are valid for a fresh account.
3. **Filter routine.** Pick a routine id the account owns (from `"$WGER_BIN" --json list routine -q limit=5`). Run `"$WGER_BIN" --json logs --routine <id>`. Exit `0`. Every result's `routine` matches or the list is empty.
4. **Since filter.** Run `"$WGER_BIN" --json logs --since 2020-01-01`. Exit `0`.
5. **Equivalent list.** Run `"$WGER_BIN" --json list workoutlog -q ordering=-date -q limit=5`. Exit `0`. Shape matches the logs shortcut (same resource).
6. **No live writes in the default logs proof.** Creating logs is covered under routine-edit / session flows if needed. If you create a throwaway log: POST via `"$WGER_BIN" --json create workoutlog --data @/tmp/wger-verify-scratch/log.json`, note the returned id/UUID, then `"$WGER_BIN" delete workoutlog <id> --yes`, then `"$WGER_BIN" get workoutlog <id>` expects an error. Remove the scratch file in cleanup.
7. **Proof.** Save command, stdout, stderr, exit code under `.verify-evidence/<ts>/workout-logs.*`. Redact token from any env dumps.

Without a token, record:

```text
token-gated - not run
attempted: WGER_BIN --json logs
observed: exit 1, stderr contains wger API error 403
```

Optional no-token negative proof (allowed): run `"$WGER_BIN" --json logs` and confirm exit non-zero and `403` in stderr without any `Token <value>` leak.

## Gotchas

- On wger.de, workout logs are private. A 403 without a token is expected, not a CLI regression.
- Log ids are UUIDs; `check_id` accepts alphanumeric and `-` only.
- `logs` always adds `ordering=-date`. Do not assume insertion order.
- `--all` stops at 50 pages and may set `truncated: true` with a `next` URL.
