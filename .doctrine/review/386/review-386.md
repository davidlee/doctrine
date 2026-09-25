# Review RV-386 — design of SL-264

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Probed the carried-key comparison against joiners, leavers, late-node moves and
rewrites, incoming `needs` edges, and stable node identities; then traced the
new cumulative guard through declaration admission, confirmation digests,
blocking-disposition derivation and the proposed VT controls. Checked sparse
`needs` clearing against row emission and revision handling, the condition
corpus, accepted decisions, and the read-side snapshot boundary.

The incumbent mechanisms support several parts of the draft: a leaver among
carried keys remains detectable; a redeclaration preserves a node's `seq`;
`CoverageStale` and `ConfirmationStale` are separate; `needs: []` already emits
one removal row per edge, while the create-edge path is distinct from ISS-481.
All 22 current snapshot files are readable by the incumbent CLI. This read
does not establish how the proposed rule will reinterpret their receipts.
DEC-062 cannot be targeted by a Revision, and REQ-427 governs accepted record
status rather than these gate attestations.

## Resolution (2026-09-25)

The three escalated findings were settled by user decision and are integrated.
The blocking set stops being a free-standing agent act and becomes a **derived
projection of a per-node `blocking` attribute**, required on creation (`DEC-302`,
amending `DEC-300`).

- **`F-1`** — accepted, scoped. `DEC-301`'s move rule applies to the nodes the
  accepting act covered; a node added after the act and later moved was never
  shown, so it does not re-face. `sec-1`, `sec-2` and `VT-4` record and pin it.
- **`F-2`** — accepted, and closed by construction rather than by a narrower
  claim. `initial-concerns-recorded`'s coverage compares blocking membership over
  the full set, so a node declared `blocking: true` moves the user's
  `graph-reviewed` and reaches them. The ninth condition the draft proposed
  (`blocking-set-current`) is dropped; `blocking-set-declared`,
  `ActKind::BlockingSetDeclared` and `Cause::ConfirmationStale` retire.
- **`F-4`** — accepted. `DEC-300` is amended explicitly, not re-read in prose;
  the condition stays `Attested` per `DEC-126`'s actor-identity discriminator.

`F-3` and `F-5` were fixed in place earlier (`sec-4`, `sec-5`). All five findings
are `verified`.
