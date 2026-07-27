# ISS-264: Conformance --strict flags import-landed funnel row as undeclared

On the dispatch funnel path, `slice conformance <id> --against <S^>..<S> --strict`
always reports exactly one undeclared path — the funnel record itself:

```
undeclared (1):
  A .doctrine/dispatch/231/funnel.toml
Error: SL-231: --strict: 1 undeclared path(s) vs 0d2cb5671..1d8cc08ae
```

This is structural, not incidental. `dispatch_import` lands the worker delta and
the funnel row in **one** commit by design ("the import lands the worker delta ⊕
the funnel row in ONE commit" — `dispatch_reap`'s landing-proof rationale). So
`S`'s own patch `[S^, S]` necessarily contains
`.doctrine/dispatch/<slice>/funnel.toml`, while a slice's declared selectors
legitimately never name orchestrator machinery.

`slice record-delta --commit <S>` is documented as the SAFE DEFAULT and names
`S` as "the phase's single import commit". Following that documented path
therefore guarantees a `--strict` failure on every funnel-driven phase.

## Why it matters

The failure is a false positive with no operator action behind it, which is the
worst kind: an undeclared-path report is exactly the signal that should mean "a
worker wrote outside its scope". Here it cannot possibly mean that — the import
belt hard-refuses `doctrine-touch` from a worker delta, so this file is
provably orchestrator-written. Training operators to wave through a `--strict`
failure erodes the check that catches the real thing.

## Fix

Exclude the funnel's own record from the conformance comparison — the same way
the range comparison already excludes trailed knowledge and refresh-base merges
by construction. `.doctrine/dispatch/<slice>/funnel.toml` is machinery, not
slice delta; it belongs in the same class as the boundaries ledger.

Verify from the SL-231 PHASE-01 data: the import commit `1d8cc08ae` is exactly
the worker's 3373-line addition plus a 20-line funnel row, and conformance
reports 6/6 conformant source paths alongside the one machinery false positive.

## Related

- IMP-171 — symmetric ledger+registry derive on the codex/pi arm; same seam.
