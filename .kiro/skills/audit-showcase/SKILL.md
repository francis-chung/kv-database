---
name: audit-showcase
description: Cross-check the showcase HTML against the real Rust API for any inaccurate claim, and verify text encoding is clean
---

# Audit Showcase

You are auditing the KV-Database showcase website for factual accuracy against the
real implementation, AND for text/encoding integrity. Read the authoritative source
files first:

- `src/protocol.rs` - which commands exist and their argument arity/types
- `src/server.rs` - the `dispatch()` function, i.e. the exact response strings
  (`OK`, `VALUE <v>`, `NIL`, `1`/`0`, `DBSIZE` counting kv_store only, `ZRANGE` row numbering)
- `src/http.rs` - the HTTP JSON wire format (`CommandRequest` variants)
- `src/sorted_set.rs` - `range()` ordering (score asc, then member key) and the
  `rem_euclid` index wrap used by `ZRANGE`

Then audit the target file(s): $ARGUMENTS

## Accuracy checks

Report every mismatch you find, and for each one cite the specific source file and
the line/behavior it contradicts:

1. Every command's name, arguments, and described behavior.
2. Every response string shown in the demo or docs matches `dispatch()`.
3. The interactive demo's simulation and live-API paths both match the server.
4. `DBSIZE` is described as counting key-value pairs only (not sorted sets).
5. `ZRANGE` ordering, index wrap, and `WITHSCORES` numbering are correct.
6. No placeholder URLs, dead links, or features the code does not implement.

## Encoding and character-integrity checks (mandatory)

Mojibake (garbled characters like `Â·` for a middle dot, or `aE"` / `â€"` for a
dash) is a shipping-blocker on a showcase site. Verify:

7. The file has NO UTF-8 BOM. The first three bytes must not be `EF BB BF`.
8. There are NO mojibake sequences. Scan for these byte/char signatures and report
   any occurrence:
   - `\u00C2\u00B7` (shows as `Â·`) - a double-encoded middle dot
   - `\u00E2\u20AC` prefix (shows as `â€...`) - a double-encoded dash or smart quote
   - any lone `\u00C2` / `\u00E2` / `\u00C3` followed by punctuation
9. Report the full inventory of non-ASCII characters with their code points, so an
   intentional glyph (a clean `\u00B7` middle dot) can be distinguished from
   corruption.

Verify with a tool that reads raw bytes (Node is reliable):

```bash
node -e "const b=require('fs').readFileSync(process.argv[1]);console.log('BOM:',b[0]===0xEF&&b[1]===0xBB&&b[2]===0xBF);const t=b.toString('utf8');const m=new Map();for(const c of t){const cp=c.codePointAt(0);if(cp>0x7e)m.set(c,(m.get(c)||0)+1);}for(const[c,n]of m)console.log(JSON.stringify(c),'U+'+c.codePointAt(0).toString(16).toUpperCase().padStart(4,'0'),n);" <file>
```

If any mojibake or a BOM is present, state it explicitly and recommend the repair
path in the `sync-readme` skill's encoding rules.

If everything is accurate and the encoding is clean, say so explicitly. Do not
invent problems.
