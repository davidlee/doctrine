# Tension render: Dep(needs)-structure unreachable on the actionable frontier; D7 knob lives in .doctrine/doctrine.toml

SL-218 PHASE-03 (tension narrative) render facts a future agent will otherwise
rediscover:

- **`needs`/Dep structure tensions never arise on the actionable frontier.**
  `surface::graded_tensions` scans only the actionable, non-promoted set, and an
  actionable item has NO non-terminal blocker by definition — so no `needs` (dep)
  edge can hold between two actionable members. The merged predecessor graph it
  builds therefore only ever contributes Seq(`after`) edges; every real Structure
  tension cites `after`, never `needs`. The pure `tension::detect` Dep branch and
  the design §3 `needs`-projected wording sample are consequently pinned as UNIT
  goldens (render `reason_line` over a hand-built `ReasonKind::Tension`, plus the
  PHASE-02 detect fixtures), NOT black-box e2e — a black-box corpus cannot reach
  that state. Structure-`after`, Composition, and all three grades (Determined /
  AgentProposed / Projected) ARE reachable and are e2e-golden'd in
  `tests/e2e_priority_golden.rs`.

- **The `demote_agent_evidence` (D7) knob is read from `.doctrine/doctrine.toml`,
  not the project-root `doctrine.toml`.** `priority::config::load` reads the
  former. Writing the knob to `<root>/doctrine.toml` silently no-ops (knob-off
  behaviour, no error) — cost a probe cycle. The knob-on tell: `explain` appends
  the `agent evidence demoted: …` disclosure line and grades flip
  `determined`→`agent-proposed`.

- **Composition is `next`-suppressed by default; `--verbose` pulls it in** (design
  D5). `next --json` carries BOTH classes unconditionally (the human
  `TENSION_MAX_CALLOUTS=3` cap and the structure-only default are human-render
  bounds only).

- **m=0 scoped disclosure is not reachable via authored facets** (the value
  multiplier comes from kind/tag costing, not a `[value]`/`[risk]` field), so the
  F-6 "N pairs value-insensitive, zero weight" line is pinned at the pure
  `tension::zero_weight_excluded` + `reason_line(ZeroWeightExcluded)` unit level.

See SL-218 `notes.md` PHASE-03 for the full close record.
