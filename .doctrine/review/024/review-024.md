# Review RV-024 — code-review of SL-062

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

A fresh **code-review** facet over SL-062's three implementation phases (FSM
re-home, unified status write-core, transactional supersede verb) — distinct
from the clean reconciliation audit (RV-022). The slice's own stated objective
is the sharpest lens: *"eliminate the byte-duplicated write body."* So the lines
of attack:

1. **Did the dedup actually finish?** The slice unifies four status setters onto
   one seam. Is that the whole population, or does a byte-duplicate survive
   somewhere — and if so, was it deferred on merit or on size?
2. **One seam, one contract?** A shared write-core that emits *contradictory*
   guidance to its callers is a half-unification. Hold the F-1 refuse hints to
   the SL-060 non-destructive lesson across *all* callers, not a curated subset.
3. **Speculative surface.** "Write less code" / YAGNI — any code shipped ahead of
   its consumer (dead_code suppressions are the tell).
4. **Transaction integrity.** The supersede verb writes two files non-atomically.
   Is a torn state both *detectable* and *recoverable*, and is the recovery path
   tested — not just the detection?
5. **Pure/imperative split + no parallel implementation** in the new path
   helpers (`resolve_supersede_path` vs `resolve_link_path`).

Prior art is strong: the design absorbed three adversarial passes (codex C4/C8
cited inline) and RV-022 closed clean. Expectation going in: no blockers; the
prey is incomplete-dedup and conscious-tradeoff hygiene.

## Synthesis

**Overall: solid.**

**Synopsis.** SL-062 does what it set out to do, and does it cleanly. The FSM
re-home (`src/lifecycle.rs`) is a textbook extraction — pure leaf, total over its
`&str` edges, no clock/disk, the three slice-status predicates kept deliberately
distinct with the reasoning written down rather than collapsed into a false
"terminal" set. The unified write-core (`dep_seq::apply_status` /
`set_authored_status`) is the genuine article: a variable-length `managed` slice
lets one core serve the `[status]`, `[status,updated]`, and
`[status,resolution,updated]` shapes, the `updated`-excluded no-op guard matches
every donor's behaviour, and the F-1 strict-refuse correctly chooses
non-destructive bail over silent subtable corruption. The supersede verb is the
strongest piece: a parse-once / hold-both / write-once transaction that composes
the PHASE-02 cores, gates ADR-first through a hardcoded capability boundary
(`adr::supersede_policy`, not a premature `GovKind` field), and gets the torn-state
write order right (NEW→OLD, so `validate` can see the tear). Black-box e2e drives
the real binary throughout — behaviour, not internals.

The findings are all small and none gate: five raised, zero blocker. They cluster
on one theme — **the unification is 4/5, not 5/5**. `knowledge::set_record_status`
remains a byte-duplicate of the very write body the slice exists to delete (F-1,
IMP-061); and the shared seam now refuses with two contradictory philosophies —
gov/requirement say "restore the seeded keys," slice/backlog/knowledge still say
"regenerate" (F-2, the destructive guidance SL-060 explicitly flagged). Both are
defensible *this slice* — knowledge was outside the named scope, and the
slice/backlog reword is pinned by the behaviour-preservation gate — but they are a
half-finished thought, and the divergence will rot if left. The remaining three
are hygiene: a speculative IO wrapper shipped on a `dead_code` expect ahead of its
consumer (F-3), a four-line path-resolution copy (F-4), and an untested
torn-state *recovery* path where only *detection* is proven (F-5).

**Consciously accepted.** Non-atomic two-file write in supersede (no journal; torn
state is detectable + recoverable by design). The `dead_code` staging of
`append_string_array` (sanctioned self-clearing pattern). The duplicated path
build (rule-of-three not yet met). Tradeoffs taken with eyes open, all recorded.

**Standing risk.** Duplication drift: every month `knowledge` and the slice/backlog
hints sit un-reconciled is a month the "one seam" story is half-true. F-2 → IMP-066
and F-5 → CHR-008 carry the durable work; IMP-061 already held F-1.

**Haiku.**

    four setters, one seam —
    the fifth still writes its own bytes,
    refusing, "regen."
