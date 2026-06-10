# IMP-008: Reconcile skill + audit/reconcile seam disentanglement

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Enact the `audit → reconcile → close` seam as built machinery, not discipline.
Named-but-deferred follow-on of ADR-003 §7/§11 + ADR-009 — not a fresh idea.

## What canon already decided

- **ADR-003 §7** — `/audit` and `/reconcile` are distinct steps with a hard edge.
  Audit *identifies* spec changes + assembles reconciliation context; it does NOT
  write them. `/reconcile` *writes* the identified spec changes against observed
  truth — the **sole explicit writer** of reconciled spec truth (§5), never
  derive-by-precedence (doctrine's differentiator).
- **ADR-009 §1** — the `reconcile` FSM state + closure-seam topology are already
  BUILT (`slice status` refuses `→reconcile` except from audit, `→done` except
  from reconcile); `reconcile` conduct defaults to `gate`.

## The recorded violation this closes (ADR-003 §7, amended by ADR-009)

Today `/audit` **over-reaches**: it writes spec/governance fixes in place (the
"design was wrong → reconcile `design.md`" disposition) instead of identifying
only. `/close` reconciles only slice *status* vs the phase rollup, never specs;
§8's spec-coherence closure gate is discipline-only. The seam is doctrine-by-
discipline, unenforced.

## Scope (deferred pieces, ADR-003 §11 / ADR-009 §11)

- **`/reconcile` skill** — the writer half of the seam; sole spec-reconciliation writer.
- **Reconcile artefact** (≈ spec-driver *revision*) — durable record of what
  reconcile changed and why. Name provisional; schema deferred (ADR-003 Neutral).
- **`slice reconcile` / spec-patch CLI** — the verb surface the skill drives.
- **Retune `/audit`** — strip spec/governance writing; identification + context-prep only.
- **Retune `/close`** — reconcile owning *specs*, not just status; the §8 closure gate.
- **Routing wire** — add the `/reconcile` row to `boot.md` routing table ONLY when
  the skill lands (F2/F14 shipped-not-reachable — never point routing at a deferred skill).

## Sequencing

- **After IMP-001** (RV review-ledger + `/review` family) — per the user; review
  and reconcile are the two halves of the §6/§7 seam tuning, and the `RV-` ledger
  (ADR-007) is the shared record mechanism reconcile's artefact may build on.
- Likely splits across spec-machinery prerequisites (tech specs, coverage blocks)
  still deferred in ADR-003 §11 — scope at `/slice` time, don't presume one slice.

## Refs

ADR-003 §5/§7/§8/§11 · ADR-009 §1/§3/§11 · spec-driver ADR-004/ADR-008 (ancestors)
· IMP-001 (sibling `/review`, lands first) · `doc/slices-spec.md` § Forward compatibility.
