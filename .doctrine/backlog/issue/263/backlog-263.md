# ISS-263: Import raises unresolvable fork name as internal fault, not refusal

`dispatch_import` documents its `name` argument as the fork **branch**
(`dispatch/<agent>`). Passed a bare agent basename (`SL-231-p01`), it does not
refuse — it raises a JSON-RPC internal fault:

```
MCP error -32603: Internal error
  data: { code: "INTERNAL", message: "resolve fork tip SL-231-p01" }
```

The harness surfaces only `MCP error -32603: Internal error`. The `data.message`
that actually names the failure is not shown, so the caller gets zero diagnostic
signal and no route to recovery.

This escapes the funnel's own contract. Every other funnel verb documents that
"a refusal is the recovery procedure" and returns
`Refused { reason, detail }`; `dispatch_reap` states outright that `Err` is
"reserved for internal faults (an unreadable record, a git plumbing failure)".
An operator typo in an argument is neither.

## Observed cost (SL-231 PHASE-01)

The session that hit this misattributed it to a stale `doctrine` binary on
PATH — plausible, because `.mcp.json` launches `${DOCTRINE_BIN:-doctrine}` and
`DOCTRINE_BIN` was unset. It wrote a handover, and a fresh session restarted
with `DOCTRINE_BIN` correctly set — and reproduced the identical error. Recovery
required driving `doctrine serve --mcp` by hand over stdio to read the
suppressed `data.message`. Roughly two sessions spent on a wrong argument.

## Fix (either alone is sufficient; both are cheap)

1. Resolve a bare agent name as `dispatch/<agent>` when the literal ref does not
   exist. `dispatch_reap`'s landing proof already speaks in `dispatch/<agent>`
   terms, so the two verbs currently disagree about what a "name" is.
2. Return `Refused { reason: "unknown-fork", detail: "no such ref: <ref>;
   expected the fork BRANCH, e.g. dispatch/<agent>" }` — the refusal family
   already exists and is documented as the recovery procedure; this path simply
   escapes it.

Fix (2) is the load-bearing one: it restores the contract. (1) is ergonomics.

## Related

- IMP-328 — the sibling pi-arm defect (fork unbound to slice/phase). Both make
  a pi-arm fork hard to hand to the funnel; this one makes the failure opaque.
- ISS-053 — the other case where a wrong binary is *suspected* on weak evidence;
  same diagnostic-vacuum shape.
