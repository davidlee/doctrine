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

## Synthesis

**Judgement: heresy — venial, not mortal. The design may lock once F-1..F-5 are
folded (they are).** The round-3 surface stood up to cross-examination. Confirmed
orthodox against current source, no charge:

- the **edge map** is total and complete for the emittable label set; the typed-edge
  split is exactly right — `tier1_edges` returns only `Tier::One` `[[relation]]`
  rows (`relation.rs:634`, docstring l.627-633 confessing consumers concatenate
  typed edges separately), and `descends_from`/`parent`/`interactions` are `Tier::Typed`
  sourced from `Spec` fields/readers, never the relation block;
- the **`needs`→`blocks` deferral** is sound — there is no `needs`/`after`/`blocks`
  `RelationLabel` variant at all (`relation.rs:45-100`); dep/seq is a separate typed
  scheduling table (SL-060), correctly not projected;
- the **9-state status map** matches `SLICE_STATUSES` (`slice.rs:542`) arm-for-arm,
  total with a `draft` default over a drift-tolerant free `String`;
- the **`catalog::test_helpers` reuse** (§9) is real, not hand-wave — the seam exists
  `pub(crate)` (`src/catalog/test_helpers.rs`), closing most of round-2's CHARGE IX;
- the **D8 cut** is the right lossy-v1 line.

**The five charges and their reconciliation** (all `fix-now`, integrated into
`design.md`, verified terminal):

1. **F-1 (major) — the round-3 sweep went stale on its own.** G4 declared
   `PhaseRollup` had "no single `total`" and hand-rolled a bucket sum that *dropped*
   `missing_toml`; the canonical `PhaseRollup::total()` has existed since SL-009
   (2026-06-05, 14 days pre-sweep) and sums *including* `missing_toml`. The partial
   sum would crown a plan with a missing phase `.toml` as `complete`. Reconciled:
   map through `total()` + `anomalies()`, malformed phase suppresses `complete`.
   *Verify in plan/execute:* a conformance case with `completed==N, missing_toml=1`
   asserts the plan node is **not** `complete`.
2. **F-2 (minor) — §9 understated a fixture gap.** `seed_adr` seeds only
   `supersedes`. Reconciled: gap named; *verify:* golden ADR edges confined to
   `supersedes` unless `seed_adr` is generalised.
3. **F-3 (minor) — `parent` table cell imprecise.** Read from the subtype-agnostic
   `Spec.parent` field. Reconciled: cell stated as expected-case.
4. **F-4 (minor) — a stale numeral.** "9 post-scope kinds" → explicit list (7 kinds
   / 10 prefixes).
5. **F-5 (nit) — fidelity-loss prose.** Reframed as two distinct losses.

**Standing risks (none blocking):** the wire **key-form** (`validate_ignore` vs
lazyspec's frontmatter `validate-ignore`; `virtual` not deserialized by
`RawFrontmatter`) is bound to lazyspec's *future JSON backend* (piece-4,
`../lazyspec`) — consciously **out of scope** here, but piece-4 must pin it or every
entity's `validate_ignore`/`virtual` silently defaults. The dangling-edge
silent-drop safety is **conditional** on piece-4 honouring `validate_ignore`.

**Tolerated:** nothing beyond the piece-4 boundary above.

**Harvest:** clean — no durable memory minted (the `total()`/`anomalies()` facts are
code-resident; the findings live structured in this ledger and in `design.md` §10
round-4). A valid nil harvest, not a dereliction.

> **HERESIS URITOR; DOCTRINA MANET**
