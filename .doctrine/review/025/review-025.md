# Review RV-025 — design of ADR-012

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Inquisition of **ADR-012** (Dispatch integration topology), facet `design`,
posture `inquisitor`. The spine was already tried at RV-023 (verdict: heretical
but redeemable; six charges reconciled). This tribunal arraigns the **ADR
authorship + the four OQ closures** that gate SL-064's plan. External adversarial
reviewer: codex / GPT-5.5, read-only.

**Lines of interrogation:**

1. **OQ-A — routing + recovery contract (the load-bearing one).** Does the
   "review-pending refs at conclude, never-auto-trunk" routing actually keep
   unreviewed code off trunk? Is the **projection journal + idempotent patch-id
   replay** contract sound — does it genuinely restore a D7-analog recovery
   *across* projection, or does it smuggle back the recanted RV-023 F-1 overclaim?
   Probe: intent-written-before-op ordering, the report-not-resolve-on-moved-target
   path, journal durability when the worktree is removed at conclude, partial-apply
   idempotency.
2. **OQ-B — temporal three-bucket boundary.** Is the prior-gate / impl-time-bundle
   / slice-orthogonal partition complete and unambiguous? What classifies an
   ambiguous knowledge write at runtime? Does "slice-orthogonal projects ahead"
   re-open a visibility/leak hazard?
3. **OQ-C — audit ordering.** Is "audit from parent after projection" coherent
   given the coordination worktree is removed at conclude and the RV verbs refuse a
   fork? Does projecting review-pending refs *before* audit let unreviewed code
   escape ahead of its own audit gate?
4. **OQ-D — deferral-with-fence.** Is deferring the positive marker legitimate, or
   does acceptance ship a known privilege hole? Is the inherited D2b fence (R-5
   belt + IMP-052 + env-catch + bwrap-no-push) actually load-bearing, and is
   IMP-065 a real capture or an escape hatch?
5. **Boundary integrity (F-4/F-5 relapse check).** Does ADR-012 honestly own the
   D1 tightening, or does it sneak back "D1 preserved"? Is the ADR-006/ADR-012 split
   clean — placement/identity on ADR-006, topology/projection/routing on ADR-012 —
   or is anything double-owned or orphaned?
6. **Internal consistency** of ADR-012 against the SL-064 design (§1/§4/§7) and
   against ADR-006/ADR-011 — no contradiction, no recanted claim resurrected.

**Doctrine held to:** RV-023 (no recanted overclaim may return), ADR-006
(D1/D2a/D7/D8/D9), ADR-011 (spawn seam untouched), ADR-007 (this ledger),
the storage rule, the layer wall (rtk/cargo/CARGO_TARGET_DIR are ADR-008).

## Synthesis

**Verdict: heretical in the particulars, sound in the spine — redeemed and now
ACCEPTED after reconciliation.** Two adversarial witnesses were put to the
question (codex / GPT-5.x and an external GPT review), and their testimony
converged with damning unanimity: the topology *structure* is right and an
improvement over the shared-main funnel, but the first authorship of the OQ
closures was **policy prose dressed as a contract** — three load-bearing holes,
and a recanted RV-023 overclaim that crept back in twice.

**Eight charges, all reconciled fix-now this pass. Three were terminal (blocker):**

- **F-1 (blocker)** — the audit gated the *wrong object*. Projection escaped
  `phase/*` + the impl bundle, yet the audit was aimed at `dispatch/<slice>`; and
  there was **no named home for "intent for review"** (only `dispatch`/`phase`/
  `edge`). **Penance:** projection is now **two-stage** — stage-1 *prepare-review*
  materialises a first-class **`review/<slice>`** ref + `phase/*` + the journal,
  with **no trunk write**; audit runs against those exact refs; stage-2 *integrate*
  only after audit. Intent defaults to `review/<slice>`, never trunk-by-default.

- **F-2 (blocker)** — the projection journal was named but left to "impl detail",
  so "report-not-resolve on a moved target" was an unproven boast. **Penance:** the
  journal is now **normative** in the ADR — committed to `dispatch/<slice>` *before*
  any ref mutation; every ref update a **compare-and-swap on `expected_old_oid`**;
  replay no-ops if `target==planned`, refuses + reports if diverged. This is the
  true D7-analog-across-projection contract RV-023 F-1 demanded.

- **F-3 (blocker)** — the **RV-023 F-2 heresy resurrected in a new costume**:
  "absence is safe because the fence catches verb abuse." The cited ADR-006/011
  fences prove only the import-belt + post-spawn abort; they do **not** prove
  coverage of `gc`/`sync`, and `bwrap` is ADR-008 repo-local, not framework-uniform.
  **Penance:** the coverage claim is recanted — the fence is **defence-in-depth,
  not a proof** — and the genuine close is moved to the OQ-D plan-gate + IMP-065.

**Five lesser taints (major/minor), all reconciled:** F-4 (OQ-D cleanly
**reclassified** from a limbo ADR-acceptance blocker to an SL-064 **plan-gate**
with restricted Orchestrator-verb invocation + impersonation tests; User ruled
(a)); F-5 (OQ-B given a **fourth bucket** + a **default-hold runtime classifier**);
F-6 (the **harness synthesis rule** — `phase/<slice>-NN` cut from `dispatch/<slice>`
on the fork-less claude arm); F-7 (the recanted **"D1 preserved via config"** wording
purged from `design.md` §1 + decision log); F-8 (the **multiplying review surfaces**
owned as a consequence).

**Standing risks carried (conscious, not defects):** the OQ-D residual identity
gap (marker-absence indistinguishable from an unstamped worker across the
Orchestrator verb class) — bounded by the plan-gate, truly closed only by the
positive coordination marker (**IMP-065**); and every OQ contract is **normative
but unimplemented** — its proof lands in SL-064's plan/execute against the §6
verification matrix.

**Tolerated:** none. Every charge reconciled fix-now; nothing normalised away.
ADR-012 is fit to accept and to gate SL-064's plan.

HERESIS URITOR; DOCTRINA MANET
