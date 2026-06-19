# Review RV-093 — design of SL-026

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Posture.** Round-4 inquisition (`--raiser inquisitor`), codex (GPT-5.5) as the
adversarial second eye per project `CLAUDE.md`. SL-026 was twice reviewed before
(self-review r1, inquisition r2 `inquisition.md`, ten charges C-I..C-X all folded)
then **round-3 re-validated** (2026-06-19, `0117c457`) after ~800 commits parked.
This trial arraigns **only the round-3 rewrites** — the load-bearing content that
has had no external adversary since it was authored. Prior charges are not
re-litigated; continuity with `inquisition.md` is assumed.

**Subject:** `design.md` §5.3 (edge map + status map + plan-rollup), §9 (fixture
seam), §7 D8 (node-set cut), §8/§10 (R-discharge claims, round-3 log). Design
intent only — no `src/` touched this stage.

**Lines of interrogation:**

1. **§5.3 edge map** — is the total `RelationLabel → RelationType` map's every arm
   defensible? `governed_by`/`drift`/deferred-kind targets dangling out of corpus —
   silent-drop genuinely safe, or graph-misleading? The `needs`→`blocks` deferral —
   sound, or is lazyspec's 4th `RelationType` dead for a bad reason? Is the
   "`descends_from`/`parent`/`interactions` = `Tier::Typed`, sourced via spec
   readers NOT `tier1_edges`" split correctly characterised against current source?
2. **§5.3 status map** — 9-state FSM, `review` struck, `draft` default. Any arm
   wrong against `SLICE_STATUSES` (`slice.rs`)?
3. **§5.3 plan status** — the hand-rolled `PhaseRollup` sum. Ground truth:
   `PhaseRollup::total()` exists since SL-009 (2026-06-05), sums **including**
   `missing_toml`; `anomalies()` = `unknown + missing_toml`. G4's "no single
   `total`" is suspected **stale**; the hand-rolled sum **excludes** `missing_toml`
   — does that flip the `complete` verdict, and is it a DRY/no-parallel-impl breach?
4. **§9 fixture seam** — `catalog::test_helpers` reuse plan. Ground truth: it
   **exists** (`pub(crate)`), shipping `seed_slice`(arbitrary edges) /
   `seed_adr`(**supersedes only**) / `seed_requirement` / `seed_knowledge` /
   `relation_rows`; **no** `seed_spec`, **no** `seed_backlog`. Are the named gaps
   complete, or does `seed_adr`'s inability to seed `related` (and any other axis)
   leave the corpus-construction plan hand-waving?
5. **§7 D8 node-set cut** — minimal v1 vs 7 deferred kinds (IMP-105). Right cut, or
   does the dangling-edge volume (`governed_by`→POL/STD, `reviews`→RV,
   `owning_slice`→REC, `revises`→REV, `decision_ref`) mislead the v1 graph?

**Doctrine held to:** ADR-001 layering, ADR-004 outbound-only, the no-parallel-impl
/ DRY mandate, totality of a projection over a drift-tolerant model, and the
verified serde of the consumer (`../lazyspec/src/engine/document.rs`).
