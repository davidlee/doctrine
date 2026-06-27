# Review RV-175 — design of SL-165

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** SL-165 `design.md` — extend `check_provenance` (`src/dispatch.rs:805/913`)
to accept a recorded `candidate/<N>/<label>` source for a `close_target` create,
tracing the candidate chain (bounded recursion) to a Verified journaled-evidence root.
Conformance to SPEC-022 REQ-317; reconciles the REQ-316⊥REQ-317 contradiction via a
Revision at reconcile.

**Lines of interrogation (the invariants the accused must confess to):**

1. **SPEC-022 REQ-316 invariant survival.** Does the widened gate still guarantee "no
   candidate from unverified evidence"? Probe the trace's termination: can any accepted
   chain reach trunk *without* rooting at a `Verified` journaled evidence ref? Hunt a
   bypass via supersession, a non-evidence base, or a missing status check.

2. **REQ-316⊥REQ-317 contradiction handling.** Is the design's reading correct — that
   REQ-316's blanket "journaled-only" clause forbids REQ-317's mandated `--source
   candidate/...`? Is routing the REQ-316 narrow through a REV (vs reconcile-direct) the
   doctrinally-right altitude, or is it over/under-cautious?

3. **ADR-012 FF-only / admit-by-OID (REQ-316).** The design claims integrate semantics
   are untouched (admitted-OID direct projection, FF-only). Verify the candidate-source
   create cannot smuggle a non-FF or non-admitted path onto trunk.

4. **Gate design soundness.** model A + bounded recursion + close_target-scoped +
   `status==Created` + count-exact match (INV-5) + lineage≠content split (A-1). Attack
   each: Is `Created`-only correct or does it over/under-refuse (F2)? Is the
   lineage/content trust boundary (A-1) sound, or does it let unreviewed content onto
   trunk? Does close_target-scoping leave a hole (e.g. review_surface/scratch abuse)?
   Is the count-exact match genuinely needed, or theatre?

5. **Unknown unknowns.** Missing invariants, TOCTOU between provenance-read and
   admit/integrate, cycle-guard correctness, terminology drift, magic numbers (budget=16).

**Doctrine held to:** SPEC-022 (REQ-311..319), ADR-012 (dispatch integration topology,
FF-only D2/D4), ADR-013 (governance routes via Revision), the storage rule.

## Synthesis

**Judgement.** The design was *not* heretical in its bones — its reading of the
SPEC-022 REQ-316⊥REQ-317 contradiction is sound, its conformance posture true, its
FF-only/admit-by-OID respect intact. But the tribunal, pressing the accused upon the
wheel, wrung **two confessions of provenance laxity** — grievous in a slice whose entire
office is to *prove provenance*. A gate that guards the name and not the substance is a
gate of straw; let it burn.

- **F-1 (blocker, fix-now, verified).** The trace blessed the ref *name* and the recorded
  row, then let `candidate_create` merge whatever the live `candidate/<N>/<label>` tip
  pointed at — a ref repointed at unrelated history would have ridden onto trunk wearing
  the mask of a repair. **Penance:** INV-6 / D5 — bind the resolved `source_oid` to the
  recorded `merge_oid` (`is_ancestor`), the source-side analog of admit's I3.
- **F-2 (blocker, fix-now, verified).** The exception admitted *any* source candidate
  role — `scratch`, `experiment` — though REQ-317 names only "the repaired candidate," a
  `review_surface`. A-1 had openly blessed the smuggling. **Penance:** INV-2 / D3 —
  source restricted to `{review_surface, close_target}` ∧ `kind == audit`; the heretical
  ride-through clause struck from the record.

**Corrective sequence (for /plan and /execute).**
1. `is_journaled_evidence_ref` predicate (single source of truth).
2. `trace_candidate_provenance` — count-exact row (INV-5), `status==Created`, role/kind
   gate (INV-2), recursive to a Verified journaled root (full journaled gate at base).
3. INV-6 lineage binding in `candidate_create` post-`source_oid`-resolve.
4. Verification: the accept/refuse matrix (§9) incl. moved-ref and role/kind refusals;
   the full repair→close→integrate→`status done` lifecycle anchor.

**Standing risks / consciously tolerated.** F2-of-design (hand-resolved `Conflicted`
source refused in v1) stands as a named limitation (OQ-4) — penance deferred, not denied.
The REQ-316 normative narrowing remains owed to the **Revision at reconcile** (D4) — the
governance debt is recorded, not paid here.

**Verdict.** With both blockers reconciled into canon, the design is **sound to proceed
to /plan.** The straw gate is rebuilt in iron.

> **HERESIS URITOR; DOCTRINA MANET**
