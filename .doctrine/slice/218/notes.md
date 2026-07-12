# Notes SL-218: Tension narrative

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — D7 agent-evidence demotion knob (2026-07-12, inline/Fable)

Commits: a6152731 (VT-1 config), 861ff8ac (VT-3 human-only compile),
9ccc43a4 (T3/T4 verdict-system seam), e98062ec (VT-2/VT-B e2e),
4e0ad809 (T6/T7 disclosure).

**EN-2 baseline** (branch-point 32c2c737, clean tree): `cargo test` exit 0,
82 suites, 4100 passed / 0 failed.

**Phase-close (VA-1 / EX-2)**: `cargo test` exit 0, 82 suites, 4112 passed /
0 failed (+12, all new); `git diff` on `tests/` vs baseline has ZERO removed
lines (no existing test edited); 21 priority goldens byte-identical knob-off;
`just gate` clean (clippy zero warnings).

Shape shipped:

- `CompareConfig { demote_agent_evidence }` under `[priority.compare]`
  (`.doctrine/doctrine.toml`), key single-sourced as
  `DEMOTE_AGENT_EVIDENCE_KEY` (STD-001), `bool_or` extractor added.
- `comparison::compile_human_only` + `human_rows` — the subset runs the full
  C1–C8 pipeline (honest per-system quarantine; VT-C proven: human row in an
  agent-involved cycle is quarantined full-system, retained human-system).
- `elicit::VerdictSystem { cs, reach, rows }` — knob-off aliases the baseline
  (single compile); knob-on a fresh human-only compile. ALL determinacy reads
  (pair skip-checks, queue state, `hypothetical_outcome` yields incl.
  anchor-review) go through it. Pool composition, counts, `confirm_boost`
  (INV-3), projection stay full-system.
- `side_in(cs, item, cost)` resolves PairSides per system; `PoolItem` no
  longer stores class/bounds/anchor. Entity absent from the verdict system ⇒
  singleton class, unbounded ⇒ indeterminate (never panics).
- Disclosure: `ReasonKind::AgentEvidenceDemoted` +
  `render::AGENT_DEMOTION_DISCLOSURE` (single fragment) → `compare elicit`
  human line + additive `agent_evidence_demoted` JSON key; `explain` human
  line + additive `agent_demotion` reason. Knob-off surfaces byte-identical.

Durable gotchas for PHASE-02/03 (also in phase sheet Findings):

- **Anchor attachment is row-gated per system**: anchors attach only to
  entities present in ≥1 row of THAT system's compile. An anchored entity
  with only agent rows loses its anchor in the human system → indeterminate
  knob-on. Fixtures want a human row (incomparable suffices) to carry classes.
- `synthetic_answer_row` is rater-agent but appended AFTER the human filter —
  hypothetical answers always constrain the verdict system (VT-F needs this).
- Grading (PHASE-02) must consume the SAME VerdictSystem selection: reuse the
  seam, don't re-derive (design F-1/F-7 one-truth-per-question).

## PHASE-02 — tension detection + evidence grading (2026-07-12, inline/Opus)

Commit: 9435bd87 (detection + grading) + follow-up (VT-3). Branch-point 4a143d55.

**Baseline** 82 suites / 4112 passed. **Close** 82 suites / 4129 passed / 0 failed
(+17: VT-1 ten + VT-2 five + VT-3 two). `just gate` clean. Priority goldens
byte-identical (EX-3) — no render this phase.

Shape shipped:

- `src/priority/tension.rs` (new, pure): `detect(&DetectInputs)` — the D4 pair
  scan (surfaced on-page × preferred over full frontier; m=0 excluded;
  equal-full-score tiebreak excluded; Structure via BFS reachability over merged
  surviving seq+dep preds, citing the first forward edge from surfaced; else
  Composition with surfaced−preferred component deltas). `grade(...)` — the pure
  D6 vocabulary (Determined / AgentProposed / Projected). Detection emits
  grade-free `DetectedTension`; `.with_grade()` → `Tension`.
- **D1 realized as `elicit::pair_side(cs, id, eff_weight)`** — the SINGLE PairSide
  resolver; `side_in` delegates. NO shared `VerdictSystem` struct (its knob-off
  arm borrows caller-owned state — lifetime friction; the selection rule is one
  line). The reuse that matters is the resolver + `determined` + `compile_human_only`,
  all shared. F-5 fence held: no new comparison/query.rs API.
- `surface::graded_tensions`/`grade_pair`/`pair_counts` — the assembly (elicit
  pattern): full pipeline compile always; fresh human-only compile knob-on;
  verdict = human-when-knob-on; `AgentProposed` fallback reads the full system.
  Wired LIVE into `explain()` as `Explanation.tensions` (UNRENDERED this phase —
  render is PHASE-03; goldens byte-identical).

Durable gotchas for PHASE-03:

- **`Explanation.tensions` is already populated** (full-frontier, `page_k =
  usize::MAX`). PHASE-03 renders it (filter to the explained id per design §2
  considered-set) — do NOT recompute. `next()` still needs its own K-capped
  `graded_tensions(g, pipeline, cfg, K)` call (structure-only default, verbosity
  adds composition).
- **Crate is `unused = deny`**: a computed-but-unrendered value must reach a
  non-test caller. That forced the live `explain()` attach (the `#[serde]`-free
  field costs nothing at render). Keep that in mind if PHASE-03 adds fields.
- Grade counts come from the producing system: human-determined ⇒ human counts
  only; `AgentProposed` ⇒ full-system counts labelled unconfirmed. `pair_counts`
  sums the pair's two classes (deduped) via the resolved `PairSide.class`.
