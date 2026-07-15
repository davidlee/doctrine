# ISS-223: drive-slice invocation passes raw slice token as args; workflow JSON.parse crashes

Surfaced live 2026-07-15 driving SL-211 via `/drive-slice SL-211`.

## The gap

The `/drive-slice` command glue invokes `Workflow({ name: "drive-slice", args:
"SL-211" })` — the **raw slice token as a string**. But the workflow's F2 slice
guard (`install/workflows/drive-slice.js`) expects `{ slice: <int> }`, and on a
string arg does `JSON.parse(args)`:

```js
const slice = Number((typeof args === 'string' ? JSON.parse(args) : (args || {})).slice);
```

`JSON.parse("SL-211")` throws `Unexpected identifier "SL"` and the workflow dies
before `meta`/bootstrap — no phase runs, no halt receipt, just a raw parse crash.

## Repro

`/drive-slice SL-211` → workflow fails: `JSON Parse error: Unexpected identifier
"SL"`. Manual relaunch with `args: {"slice": 211}` parses fine and reaches
bootstrap.

## Fix candidates

- **Command glue** should marshal the routed slice id into the shape the workflow
  documents — `{ slice: 211 }` (strip the `SL-` prefix, coerce to int) — rather
  than forwarding the raw token.
- **Or** harden the workflow's F2 guard to accept the `SL-NNN` string form
  (parse the trailing integer) so it is robust to either caller. The guard already
  tolerates a JSON-string; it should tolerate the id-token string too, since that
  is what the `/drive-slice <ID>` UX naturally produces.

Prefer hardening the guard — it is the fail-closed boundary and the id-token is
the ergonomic invocation. Whichever side changes, the two must agree on the args
contract.

Sibling known rough edges on this path: [[IMP-277]] (claude arm never arms the
worker spawn), IMP-275 (in-workflow land/close).
