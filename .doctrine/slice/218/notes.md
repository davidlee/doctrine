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
