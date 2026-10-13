# Routine edit

Routine edit lets a user inspect routine structure views and create, update, and delete routine graph objects (routine, day, slot, slot-entry, `*-config`) on an authenticated account.

## Sub-features

- `routine-view-structure` reads `wger routine <id> structure` (default view).
- `routine-view-other` reads `logs`, `stats`, `date-sequence-display`, `date-sequence-gym`.
- `routine-create-day` creates a day under a routine.
- `routine-update` patches a day or slot field.
- `routine-delete` removes a throwaway object with `--yes`.

## How to get to it (user POV)

- Run `wger routine <id>` or `wger routine <id> structure`.
- Run `wger routine <id> logs` (or `stats`, `date-sequence-display`, `date-sequence-gym`).
- Run `wger create day --data '…'` / `wger update day <id> --data '…'` / `wger delete day <id> --yes`.
- Run the same create/update/delete pattern for `routine`, `slot`, `slot-entry`, and config resources.

## Driving it with wger

Preconditions:

- Doctor ok.
- **Token required.** Routine list/views and all writes need `WGER_API_TOKEN` on wger.de.

### token-gated - not run (no token in this environment)

When a disposable token exists (never print it):

1. Export `WGER_API_TOKEN` from a local secret source into the environment only.
2. **List routines.** `"$WGER_BIN" --json list routine -q limit=5`. Exit `0`.
3. **Read structure.** Pick an id the account owns (or create one in step 4 first). `"$WGER_BIN" --json routine <id> structure`. Exit `0`. JSON describes days/slots.
4. **Throwaway create.** Write a scratch body, for example `/tmp/wger-verify-scratch/routine.json`:
   - Prefer creating under an existing disposable routine if the account already has one for verification.
   - Example day create: `"$WGER_BIN" --json create day --data '{"routine":<id>,"order":99,"name":"verify-wger-cli-tmp"}'`.
   - Note returned `id` as `DAY_ID`.
5. **Update.** `"$WGER_BIN" --json update day "$DAY_ID" --data '{"name":"verify-wger-cli-tmp-2"}'`. Exit `0`. `"$WGER_BIN" --json get day "$DAY_ID"` shows the new name.
6. **Delete.** `"$WGER_BIN" delete day "$DAY_ID" --yes`. Exit `0`. `"$WGER_BIN" get day "$DAY_ID"` exits non-zero (404 or equivalent).
7. **Refuse without --yes.** `"$WGER_BIN" delete day "$DAY_ID"` (no `--yes`) must fail even with a token.
8. **If you created a whole routine** only for verification, delete it and its children in reverse dependency order (configs → slot-entry → slot → day → routine) before the run ends.
9. **Proof.** Save each command's stdout/stderr/exit under `.verify-evidence/<ts>/routine-edit-*`. Redact tokens.

Without a token, record:

```text
token-gated - not run
attempted: WGER_BIN --json routine 1 structure
observed: exit 1, stderr contains wger API error 403
write fail-fast: WGER_BIN create day --data '{}' → stderr contains WGER_API_TOKEN is not set
```

Optional no-token negative proofs (allowed): routine view 403; create without token fails with `WGER_API_TOKEN is not set`.

## Gotchas

- Delete refuses without `--yes` (CLI) or `confirm: true` (MCP). That refusal is required behavior.
- Config resources need bodies shaped like `{"slot_entry": ID, "iteration": 1, "value": "10", "operation": "r", "step": "abs", "repeat": true}` (see README).
- Never leave verify-named objects on a shared account.
- Do not use another user's routine id; views for foreign ids may 403/404.
