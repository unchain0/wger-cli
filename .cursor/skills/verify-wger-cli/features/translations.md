# Translations

Translations lets a user resolve languages, read exercise translations, and create or update a translation for an exercise in a target language (HTML description).

## Sub-features

- `translations-language-lookup` lists languages and resolves codes like `pt`.
- `translations-list` lists `exercise-translation` rows (public read).
- `translations-set-create` creates a translation when none exists for exercise+language.
- `translations-set-update` patches name/description when a row already exists.

## How to get to it (user POV)

- Run `wger list language -q short_name=pt`.
- Run `wger list exercise-translation -q exercise=N -q language=7`.
- Run `wger translation --exercise N --lang pt --name '…' --description '<p>…</p>'` (optional `--description-file`, `--license-author`).

## Driving it with wger

Preconditions:

- Doctor ok.
- Public reads work without a token. Writes need `WGER_API_TOKEN`.

### Public read (run without token)

- **Language by code.** `"$WGER_BIN" --json list language -q short_name=pt`. Exit `0`. `results[0].short_name` is `pt` and `id` is numeric (7 on wger.de).
- **List translations.** `"$WGER_BIN" --json list exercise-translation -q limit=1`. Exit `0`. `results` length 1; object has `exercise`, `language`, `name` or `description` fields as returned by the API.
- **Proof.** Save under `.verify-evidence/<ts>/translations-read.*`.

### token-gated - not run (writes; no token in this environment)

When a disposable token exists (never print it):

1. Export `WGER_API_TOKEN` into the environment only.
2. Choose an exercise id you may edit (or a dedicated test exercise on a private instance). Prefer a private/self-hosted `WGER_BASE_URL` for write tests when possible so public catalog data is untouched.
3. **Set translation.**  
   `"$WGER_BIN" --json translation --exercise <id> --lang pt --name "verify-tmp-$(date +%s)" --description "<p>verify-wger-cli temporary</p>" --license-author "verify-wger-cli"`.
4. Note `action` in the JSON (`created` or `updated`) and the translation id.
5. **Read back.** `"$WGER_BIN" --json list exercise-translation -q exercise=<id> -q language=<lang_id>` and confirm the name/description.
6. **Cleanup.** If `action` was `created`, `"$WGER_BIN" delete exercise-translation <translation_id> --yes`. If `updated`, patch the previous name/description back with another `translation` call or `update exercise-translation`. Confirm with list/get.
7. **Proof.** Save commands and redacted output under `.verify-evidence/<ts>/translations-write.*`.

Without a token, for the write path record:

```text
token-gated - not run
attempted: WGER_BIN translation --exercise 91 --lang pt --name x
observed: exit 1, stderr contains WGER_API_TOKEN is not set
```

## Gotchas

- `--lang` accepts a short code (`pt`) or a numeric language id. The client resolves codes via `list language`.
- Creating a new translation requires both `--name` and `--description`. Update can pass only the fields being changed.
- Description is HTML. `--description-file` reads a file path for long bodies.
- Do not leave `verify-tmp-*` names on production exercises; prefer a private instance for write proofs.
