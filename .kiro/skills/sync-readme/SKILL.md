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

## Consistency checklist

1. The command list (GET, SET, DEL, EXISTS, DBSIZE, CLEAR, ZADD, ZSCORE, ZREM,
   ZRANGE) matches `protocol.rs` in both documents.
2. Example responses in both documents match `dispatch()` output exactly.
3. Ports are correct: TCP `127.0.0.1:7878`, HTTP `127.0.0.1:8080`.
4. The tech stack / dependency list matches `Cargo.toml`.
5. The GitHub URL is `github.com/francis-chung/kv-database` everywhere.
6. Any feature mentioned in one document but missing from the other is reconciled.

## Encoding rules (mandatory, non-negotiable)

Garbled characters (mojibake like `Â·` or `aE"`) have shipped on this site before.
The root cause was writing files that contain non-ASCII characters with a tool that
emits a UTF-8 BOM or re-encodes bytes. Follow these rules:

- NEVER write or rewrite these files with Windows PowerShell `Set-Content -Encoding utf8`
  or `Out-File -Encoding utf8`. On Windows PowerShell 5.x those emit a UTF-8 BOM and
  can double-encode multibyte characters, producing mojibake.
- Prefer the editor's own file-write tool (the `write`/`fs_write` tool) for any edit,
  including find-and-replace. It writes clean UTF-8 with no BOM.
- If a shell is unavoidable for a transform, use Node:
  `node -e "const fs=require('fs');let t=fs.readFileSync(f,'utf8');/*...*/;fs.writeFileSync(f,t,'utf8')"`.
  Node writes UTF-8 without a BOM and does not re-encode.
- PREFER plain ASCII in source text. Use a regular hyphen `-` instead of an em-dash
  `\u2014` or en-dash `\u2013` (also consistent with the frontend skills' em-dash ban).
  Use a middle dot `\u00B7` only when it is genuinely wanted, and verify it renders.
- Keep an explicit `<meta charset="UTF-8">` as the first element in `<head>`.

## After editing (verification)

1. Confirm no BOM and no mojibake with a byte-level check (Node reads raw bytes):

   ```bash
   node -e "const b=require('fs').readFileSync(process.argv[1]);console.log('BOM:',b[0]===0xEF);const t=b.toString('utf8');for(const s of ['\u00C2\u00B7','\u00E2\u20AC'])if(t.includes(s))console.log('MOJIBAKE:',JSON.stringify(s));" <file>
   ```

2. Run `cargo check` to confirm nothing in the Rust build was broken.

Make the minimal edits required for consistency. Do not add features that are not
implemented.
