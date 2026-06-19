# Review RV-101 — reconciliation of SL-111

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (self-audit; `--as` drives both roles, ADR-007).

**Surface reviewed:** dispatched slice — audited the candidate interaction branch
`candidate/111/review-001` (`cand-111-review-001` @ `7bf8e0b1`, a 3-way merge of
the immutable impl bundle `review/111` @ `55d617fa` onto `refs/heads/main`).
Evidence refs `dispatch/111` / `review/111` are immutable (R2). Coordination
worktree removed pre-audit; refs preserved.

**What this audit probes** — that SL-111 delivered the cycle-break without
behavioural drift:

1. **ADR-001 layering** — `kinds` is a true leaf (no `crate::`); the relation
   engine consumes `kinds::*`, never command `*_KIND`; the 7 confirmed
   relation↔command cycles are gone.
2. **Behaviour preservation gate** — the relation + corpus suites stay green
   *unchanged*; public fn signatures (`lookup`/`tier1_edges`/`rels_block`) stay
   `&Kind` (zero caller churn).
3. **INV-2 single-source** — each command kind prefix literal lives once
   (`kinds::<X>`); no parallel copy survives in a command `*_KIND` const.
4. **Scope discipline** — `relation_graph` upward reaches were declared
   out-of-scope (Non-Goals); confirm they were left alone and contribute no
   cycles.

**Evidence gathered:** closure greps (all phases), `just check` (fmt+clippy), and
`cargo test` on the candidate worktree; one baseline run on clean `main`
(`26cb350a`) to classify the single test failure.

## Synthesis

SL-111 lands clean. The slice did exactly what it scoped: it inverted the
relation engine's kind-identity ownership onto a new leaf `kinds` module, breaking
the 7 confirmed relation↔command cycles ADR-001 predicted, and re-pointed every
command `*_KIND` const at the hoisted constant so the prefix vocabulary lives in
one place. All three claims are compiler-enforced (the `&Kind`→`&str` re-type
turns any missed `.prefix` site into a build error) and verified empirically by
the closure greps; `cargo clippy`/`fmt` are zero-warning.

**Closure story.** The behaviour-preservation gate — the load-bearing
invariant for a change to shared machinery — holds. The relation unit suite, the
corpus/relation integration tests, and the rest of `cargo test` are green with
their assertions unchanged. Public signatures (`lookup`/`tier1_edges`/`rels_block`)
stayed `&Kind`, so there is zero caller churn.

**The one red test, classified, not waved away.**
`backlog_corpus_keeps_dep_seq_typed_migrates_cross_kind_axes` fails — but on
`backlog-030.toml` corpus data (an out-of-vocab `related` relation label), and it
reproduces *identically* on a clean `main` checkout (`26cb350a`) carrying no
SL-111 code. The re-key did not cause it; it is pre-existing ISS-030 dirt the
handover already flagged as leave-alone. SL-111 touched no backlog data and no
migration path (F-1).

**Standing risks / tradeoffs consciously accepted.** `priority/partition.rs`
keeps its own `KindPartition` prefix table — a second copy of the now-hoisted
vocabulary. This was *out of SL-111's declared scope* (command consts + relation
engine only; non-relation tables and `relation_graph` upward reaches are
Non-Goals), so it is a residual the cycle-break *enables tidying*, not a defect
left behind. Routed to CHR-012 (F-3). The pre-existing ISS-030 corpus dirt (F-1)
is likewise left to its own remit.

## Reconciliation Brief

### Per-slice (direct edit)
- None. `design.md` and the implementation are already coherent — the impl diff
  equals the EX criteria enumerated in `plan.toml`; no prose drift to correct.

### Governance/spec (REV)
- None. No ADR, spec, or requirement finding was raised; the change is fully
  within ADR-001's existing mandate (it *realises* the rule, it does not amend it).

### Routed elsewhere (not reconcile's surface)
- F-3 → **CHR-012** (fold `priority/partition.rs` `KindPartition` prefixes onto
  `kinds::*`). Owned future cleanup, out of this slice's scope.
- F-1 → pre-existing **ISS-030** corpus-data dirt (`related` relation label),
  leave-alone per handover; not SL-111's remit.

Reconcile is effectively a confirmation pass: nothing to write through either
surface; advance the lifecycle.

## Reconciliation Outcome

### Direct edits applied
- None. `design.md` / `slice-111.md` already match the implementation — no prose
  drift to correct (F-2 aligned).

### REVs completed
- None. No governance/spec finding was raised; SL-111 realises ADR-001's existing
  mandate, it does not amend any ADR, spec, or requirement.

### Routed / leave-alone (not a reconcile write)
- RV-101 F-3: follow-up captured as **CHR-012** (fold `priority/partition.rs`
  `KindPartition` prefixes onto `kinds::*`) — owned future cleanup, out of scope.
- RV-101 F-1: aligned — pre-existing **ISS-030** corpus-data dirt (`related`
  relation label), reproduces on clean `main`; not SL-111's remit, leave-alone per
  handover.

Both reconcile write surfaces (per-slice direct edit, governance/spec REV) are
empty. Reconcile pass complete — handoff to /close.
