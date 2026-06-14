# Review RV-020 — reconciliation of SL-060

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance reconciliation of SL-060 (cross-kind dep/seq capture: extend
`needs`/`after` authoring beyond backlog to slices). Five phases landed via the
dispatch funnel: P01 PRD-011 canon amendment, P02 lift to `src/dep_seq.rs`, P03
slice verbs + scaffold, P04 cross-kind consumer dispatch, P05 out-of-band corpus
backfill. Reconciled against `design.md` (D1–D7, INV-1/2/3, ASM-1, E1–E10),
`plan.toml` (per-phase EX/VT), and canon (PRD-011 / REQ-258 closed allowlist).

Lines of attack:

- **INV-2 byte-identity** — backlog `needs`/`after`, priority
  (survey/next/blockers/explain), `backlog order` goldens + backlog verb
  success-message text UNCHANGED through the lift + delegate.
- **New behaviour** — slice→slice `needs` surfaces as a cross-kind blocker; `after`
  rank/age ordering; `slice show`/`--json` round-trips authored dep/seq.
- **D2 closed allowlist** (REQ-258 #3) — unresolvable / free-text / self-edge /
  non-authoring SRC / non-work TGT (incl. a resolvable non-work kind) refused.
- **F5** — non-authoring kind contributes zero dep/seq edges with no disk read.
- **INV-1 / storage shape** — seeded `[relationships]` precedes every `[[relation]]`;
  PHASE-05 backfill clean across the corpus; `validate` clean.
- **Bodies likely buried:** the design-mandated relation-migration golden amendment
  (FLAG #1) — and whether its *sibling* corpus oracle was carried along; the SL-061
  cross-agent backfill (FLAG #2); whether the data-only PHASE-05 was gate-verified
  after it changed corpus contents.

## Synthesis

**Closure story.** SL-060's thesis landed: the dep/seq axis (`needs`/`after`) is
now authorable cross-kind. The schema + strict edit-preserving write lifted once
into `src/dep_seq.rs` (D1); generic `doctrine needs`/`after` verbs ride `link`'s
resolver with the D2 work-like-target gate (the single widen-later guard); the
slice scaffold seeds the `[relationships]` table (both arrays, before
`[[relation]]`, INV-1) and `slice show`/`--json` round-trip it; the engine
`dep_seq_for` dispatch + the generalised priority read-gate carry slice dep/seq
into the already-kind-agnostic blocker/next view (F5 no-read short-circuit for
non-authoring kinds); PHASE-05 backfilled the seeded block into every pre-existing
slice. Canon moved first (PRD-011 amendment, REQ-258 closed allowlist).

**Reconciliation evidence.** INV-2 byte-identity holds — the full root suite is
green (1212 lib + every e2e binary), including the backlog needs/after, priority
survey/next/blockers/explain, and `backlog order` goldens, and the plan-named new
suites `e2e_dep_seq_verbs` + `e2e_priority_cross_kind`. D2's closed-allowlist
refusals, the F5 no-read probe, and the INV-1 round-trip golden all pass. PHASE-05:
all slices carry `[relationships]`, none missing; `doctrine validate` corpus clean.
**FLAG #1 (golden amendment) — design-sanctioned, confirmed**, and it surfaced the
one real defect (F-1): the *template* oracle was amended but its *corpus-walk*
sibling was not, so PHASE-05's backfill drove the gate RED against a stale
pre-SL-060 invariant. **FLAG #2 (SL-061 cross-agent backfill) — clean**: SL-061
carries the correct seeded block (both empty arrays, before `[[relation]]`); no
foreign WIP was swept.

**Findings (2, both terminal).**
- **F-1 (blocker → fix-now, verified).** `slice_corpus_*` oracle pinned the
  SL-058 table-absent shape, contradicting the locked SL-060 design (§5.3/E9) and
  the PHASE-05 corpus data → `just check`/`just gate` RED. Fixed in-audit: the
  corpus oracle now asserts every typed `[relationships]` key is a dep/seq key
  (⊆ `{needs, after}`), subsuming the structural-leak guard; `assert_f1` and the
  `[[relation]]` allowlist retained; renamed for honesty. Gate green, clippy zero.
  The close-gate would have refused close had this stayed open.
- **F-2 (minor → tolerated, verified).** Root cause: the data-only PHASE-05 landed
  without a full-gate re-run, so corpus-oracle drift escaped the per-phase dispatch
  funnel (each code phase verified *before* the backfill changed corpus contents).
  Data is correct; only the unrun gate hid the test-side staleness. Captured as
  durable process learning (memory), no code guard filed.

**Standing risks / accepted tradeoffs.** ASM-1 (load-bearing): D5's no-migration
stance holds only while there are no upgrade-in-place clients with pre-existing
table-less slices — true today (fresh installs scaffold the table; this dogfood
repo is backfilled). A future upgrade-in-place user needs a backfill/lazy-seed
story (design follow-up, not SL-060 scope). The PARKED edge-label question
(`needs` vs IMP-047 `gates`) is legitimately deferred under D2's single work→work
semantic. The slice author path defers cycle diagnosis to read-time (backlog
delegate retains its author-time refuse, INV-2) — a cross-kind author-time cycle
oracle is a possible follow-up, not scope.
