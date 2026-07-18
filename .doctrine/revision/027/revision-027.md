# REV REV-027 — reconcile SL-222

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconcile REV for SL-222 (RV-284). The estimate-facet retirement to ledgered
anchor-claims is already ratified — REV-026 amended ADR-015 + SPEC-020 + PRD-014
§1/§2 pre-flip, and PHASE-09 shipped the facet deletion. This REV completes the
one governance/spec obligation the dispatch REV left partial: PRD-014's §6
behaviour section still narrates the estimate as an authored `[estimate]` table
(primary + validation flow), lagging the claims-era wording the value half
already carries (REV-024) and that estimate §1/§2 now carries (REV-026).

RFC-020's Phase-2 delivery recording (RV-284 F-8(1)) is a direct edit to the
RFC body, not a row here — the CLI (correctly, ADR-014) refuses an RFC as a
`revises` target; RFCs are governance-neutral.

## Reconcile narrative (SL-222)

- [RV-284 F-8(2)]: PRD-014 §6 estimate primary-flow + validation-flow prose
  amended to the claims-era mechanism (`estimate set` mints a ledgered
  anchor-claim; `[estimate]` table survives transitionally for unmigrated
  corpora; migration via `scripts/migrate_estimate_facets.py` or re-assert via
  `estimate set --rater human`), mirroring the value half (REV-024) and the
  estimate §1/§2 wording (REV-026). Before/after below.

## PRD-014 §6 before/after (F-8(2))

**Primary flow — before:**
> Primary flow — record an estimate: an author adds an optional `[estimate]`
> table with `lower` and `upper` to an entity's TOML. …

**Primary flow — after:** reframed to `estimate set` minting a dated, attributed
anchor-claim row (`est_lower`/`est_upper`) to the comparison ledger; `estimate
clear` tombstones, correction is supersession, `estimate pin` the governed
operator override; the transitional `[estimate]` table is consulted only when no
claim rows exist, with the migration/re-assert remedy named.

**Validation flow — before:**
> Validation flow — estimate: when `[estimate]` is present it must be
> structurally valid — `lower` required, `upper` required … When `[estimate]`
> is absent, validation passes.

**Validation flow — after:** a present estimate claim requires both bounds
present, finite, `est_lower >= 0`, `est_upper >= est_lower`; absence of claims is
valid; the same present-required-valid / absent-clean rule applies to a
transitional `[estimate]` table where one survives.
