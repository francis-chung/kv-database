---
name: audit-showcase
description: Cross-check the showcase HTML against the real Rust API for any inaccurate claim
---

# Audit Showcase

You are auditing the KV-Database showcase website for factual accuracy against the
real implementation. Read the authoritative source files first:

- `src/protocol.rs` — which commands exist and their argument arity/types
- `src/server.rs` — the `dispatch()` function, i.e. the exact response strings
  (`OK`, `VALUE <v>`, `NIL`, `1`/`0`, `DBSIZE` counting kv_store only, `ZRANGE` row numbering)
- `src/http.rs` — the HTTP JSON wire format (`CommandRequest` variants)
- `src/sorted_set.rs` — `range()` ordering (score asc, then member key) and the
  `rem_euclid` index wrap used by `ZRANGE`

Then audit the target file(s): $ARGUMENTS

Report every mismatch you find, and for each one cite the specific source file and
the line/behavior it contradicts. Check in particular:

1. Every command's name, arguments, and described behavior.
2. Every response string shown in the demo or docs matches `dispatch()`.
3. The interactive demo's simulation and live-API paths both match the server.
4. `DBSIZE` is described as counting key-value pairs only (not sorted sets).
5. `ZRANGE` ordering, index wrap, and `WITHSCORES` numbering are correct.
6. No placeholder URLs, dead links, or features the code does not implement.

If everything is accurate, say so explicitly. Do not invent problems.
