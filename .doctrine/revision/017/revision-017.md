# REV REV-017 — Narrow SPEC-002 D8: Failed un-acceptable, VH-Blocked bar, withdrawal-act

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Originates from SL-179 (RSK-008). SPEC-002 D8 today treats **all** residual drift
uniformly: refuse unless a recorded override (an accept-REC) is present. That makes
a live `Failed` coverage cell — a check that *ran and contradicted* — discharge-able
by the same accept path as benign status-lag (`EvidenceOutrunsAuthored`). SL-179
establishes that an active contradiction is qualitatively worse than lag, and that
the closure gate must split the two drift sources (`drift` currently lumps them as
one `ObservedContradiction`). This REV narrows the governance so the code SL-179
ships is authorized first (canon: no code ahead of governance; ADR-013 routes the
spec edit through a Revision).

### The four narrowings

1. **`Failed` is not acceptable residual drift.** A failing check has no credible
   close-over case: fix it (`coverage verify` re-derives Failed→Verified) or withdraw
   the requirement via a recorded act. No accept-REC discharges a live `Failed` cell.
2. **`Blocked` is acceptable only with fresh human (VH) confirmation.** Evidence
   *unobtainable* (PRD-013's first-class failure mode) stays reconcilable via the
   recorded-override path, but **stricter** than lag: the requirement must also carry
   a fresh `Verified` cell with mode `VH`, and the accept-REC must cite both keys
   (design D3). Honours NF-001 — the human still decides.
3. **Withdrawal over a live contradiction is itself a recorded act (D4).** Flipping a
   requirement to `Retired`/`Superseded` while it carries a live `Failed`/`Blocked`
   cell does not silently escape the gate: it requires a slice-owned
   `revise`/`redesign` REC citing the evidence keys.
4. **The gate is the `done` path; `abandoned` is ungated (codex M6).** Clarify that
   coverage gating applies to `done` (the reconcile→done edge, ADR-009 F12); the
   `abandoned` terminal is a distinct giving-up exit and is explicitly **not**
   coverage-gated.

### Change rows (both surfaced-for-manual at apply)

**`modify SPEC-002`** — D8 prose. Before (excerpt):

> **D8 — the closure gate default-refuses residual drift, with a recorded override.**
> … Residual drift → refuse, unless an explicit override is present; the override
> **is** a reconciliation act (a REC recording accepted residual drift + rationale) …

After (intent): residual drift is no longer uniform — a `Failed` cell is
un-acceptable (fix or recorded withdrawal); a `Blocked` cell is acceptable only with
a fresh VH `Verified` cell cited by the override REC; withdrawal over a live
contradiction is a recorded `revise`/`redesign` REC; the gate guards `done`, not
`abandoned`.

**`modify REQ-113`** (primary, FR-006) — statement + acceptance criteria. Before
(statement): *"The closure gate default-refuses a terminal transition while owning
specs carry residual drift, overridable only by a recorded reconciliation act."*
After (intent): the override is **not** uniform — a `Failed` cell is not
override-dischargeable (fix or recorded withdrawal); a `Blocked` cell is dischargeable
only via a recorded override that cites a fresh VH `Verified` cell; withdrawal over a
live contradiction is a recorded act. The status-lag accept path
(`EvidenceOutrunsAuthored`) is unchanged (SL-044 D-B1).

In-place `modify` chosen over minting a companion requirement (SL-179 phase-plan
D-PP1): the Failed/Blocked split is an internal refinement of "residual drift", not a
new independent requirement. SL-179 seeds `[gate].extra_reqs = ["REQ-113"]` so its
own close gate is answerable for the amended requirement (the PHASE-03 dogfood).
