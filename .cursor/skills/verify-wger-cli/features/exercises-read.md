# Exercises read

Exercises read lets a user browse the public wger exercise catalog (list, filter, get) without an API token and confirm muscle and category metadata come back in JSON.

## Sub-features

- `exercises-list` returns a page of `exerciseinfo` or `exercise` objects.
- `exercises-filter-muscle` narrows the list with `-q muscles=<id>` (numeric muscle id from `list muscle`).
- `exercises-get` fetches one `exerciseinfo` by id.
- `exercises-resources` shows catalog-related names via `wger resources`.

## How to get to it (user POV)

- Run `wger list exerciseinfo` or `wger list exercise` in a terminal.
- Add repeated `-q key=value` filters (for example `-q muscles=1 -q limit=5`).
- Run `wger get exerciseinfo <id>` for one object.
- Run `wger resources` to see allowed resource names.

## Driving it with wger

Preconditions:

- `bash .cursor/skills/verify-wger-cli/scripts/doctor.sh` prints `doctor: ok`.
- `WGER_API_TOKEN` is unset (proves public access).
- Network can reach `WGER_BASE_URL` (default `https://wger.de`).

- **List page.** Run `"$WGER_BIN" --json list exerciseinfo -q limit=2`. Exit code `0`. Stdout JSON has integer `count` ≥ 1, a `results` array of length 2, and each result has numeric `id`.
- **Muscle filter.** Run `"$WGER_BIN" --json list muscle -q limit=5` and pick id `1` (Biceps brachii on wger.de) or another id from the page. Run `"$WGER_BIN" --json list exerciseinfo -q muscles=1 -q limit=2`. Exit code `0`. `results` length ≤ 2 and `count` ≥ 1. Do **not** use `-q muscle=biceps`; that key is not a wger list filter and is not a valid substitute for `muscles=<id>`.
- **Get one.** From the filter results, take `id` (example stable id on wger.de: `91`). Run `"$WGER_BIN" --json get exerciseinfo 91`. Exit code `0`. Stdout has `"id": 91` (or the chosen id) and category/muscles fields.
- **Bare exercise resource.** Run `"$WGER_BIN" --json list exercise -q muscles=1 -q limit=1`. Exit code `0`. Result objects are slimmer than exerciseinfo but still include `id` and `muscles`.
- **Proof.** Save stdout to `.verify-evidence/<ts>/exercises-read.json` and the exit code to `exercises-read.exit`. Record the exact commands in `exercises-read.cmd.txt`.

## Gotchas

- Exercise ids are not dense. `get exerciseinfo 1` can 404 even when the catalog is healthy; use an id from a list result.
- Filter keys are API query params (`muscles`, `limit`, `offset`), not free text. `-q muscle=biceps` is wrong.
- `--all` on a huge catalog follows up to 50 pages and can be slow; prefer `limit` for smoke proofs.
- Without `--json`, output is pretty-printed; assert on `--json` for stable parsing.
