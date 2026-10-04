---
name: sync-readme
description: Keep README.md and the showcase website in sync with the actual code and each other
---

# Sync README and Showcase

Ensure `README.md` and `kv-database-showcase.html` are consistent with the real
implementation and with each other.

Read the source of truth first:

- `src/protocol.rs`, `src/server.rs`, `src/http.rs`, `src/sorted_set.rs` for behavior
- `Cargo.toml` for the dependency list and edition
- `README.md` and `kv-database-showcase.html` for current claims

Then reconcile the following and apply the needed edits: $ARGUMENTS

Checklist:

1. The command list (GET, SET, DEL, EXISTS, DBSIZE, CLEAR, ZADD, ZSCORE, ZREM,
   ZRANGE) matches `protocol.rs` in both documents.
2. Example responses in both documents match `dispatch()` output exactly.
3. Ports are correct: TCP `127.0.0.1:7878`, HTTP `127.0.0.1:8080`.
4. The tech stack / dependency list matches `Cargo.toml`.
5. The GitHub URL is `github.com/francis-chung/kv-database` everywhere.
6. Any feature mentioned in one document but missing from the other is reconciled.

Make the minimal edits required for consistency. Do not add features that are not
implemented. After editing, run `cargo check` to confirm nothing was broken.
