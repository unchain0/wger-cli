# verify-wger-cli Skill

## Purpose
Verification skill for the wger-cli project. Demonstrates how to prove the CLI and MCP server work correctly, including read-only checks, token-gated write checks with cleanup, and MCP protocol compliance.

## How to Prove It Works

### 1. Run the full test suite
```bash
cargo test
```
All 12 tests should pass. These tests use `mockito` to spin up a fake wger API server, so no real HTTP requests to wger.de are made.

### 2. Read-only live checks (no token required)
The wger API allows reading public data without authentication. To verify this works:
```bash
# Search for exercises (public endpoint, no token needed)
wger list exercise -q muscle=biceps
```
This should succeed and return exercise data even without `WGER_API_TOKEN` set.

### 3. Token-gated write checks (with cleanup)
Write operations require `WGER_API_TOKEN`. To test safely:
```bash
# Set a throwaway token for testing
export WGER_API_TOKEN="test-token-123"

# Perform a write operation (e.g., create a routine view read)
# Then clean up any created resources

# Example: list routines to verify token works
wger list routine

# Cleanup: if any test entries were created, remove them
# (In practice, the tests use mockito so no real API calls are made)
```

### 4. MCP tools/list smoke test
The MCP server should respond to the `tools/list` command with the expected 7 tools:
```python
# Pseudo-code for MCP client
from some_mcp_client import connect_to_stdio
response = send_mcp_message({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
# Expected: {"jsonrpc": "2.0", "id": 1, "result": {"tools": [...7 tools...]}}
```

The 7 MCP tools are:
- `wger_list` - List a wger resource
- `wger_get` - Get one object by id
- `wger_create` - Create an object (POST)
- `wger_update` - Partially update an object (PATCH)
- `wger_delete` - Delete an object (requires confirm=true)
- `wger_routine_view` - Routine read views
- `wger_translation_set` - Create/update exercise translation

### 5. Auth header verification
Verify the authorization header is correctly formatted as `Token <t>`:
- The client sets `Authorization: Token <t>` header
- The token is never logged or exposed in error messages
- Error messages include the API status code but strip the token

### 6. Pagination verification
The `list` command with `--all` flag follows `next` links up to 50 pages. Verify:
- Without `--all`, only the first page is returned
- With `--all`, all pages are followed and results are aggregated
- The output includes `count` (total results), `total` (from API), `truncated` flag, and `next` continuation URL

### 7. Error handling verification
- API errors surface the status code without exposing the token
- Non-GET requests without `WGER_API_TOKEN` produce: "WGER_API_TOKEN is not set; write operations need a wger API token"
- Invalid resources are rejected with: "unknown resource '{resource}'. Allowed: ..."
- Invalid IDs are rejected with: "invalid id '{id}'"

## Files Touched
- `.cursor/skills/verify-wger-cli/SKILL.md` - This file
- Tests in `tests/api.rs` - All 12 tests use mockito fake server