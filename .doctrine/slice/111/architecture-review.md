  Architecture health — doctrine (88k LOC, ~90 modules)

  Verdict

  Discipline is holding well where it's compiler- or
  lint-enforced, and drifting exactly where enforcement is
  review-only. Nothing is on fire. But ADR-001's own predicted
  failure mode has arrived: cycles the layering rule forbids
  now exist, and there's no gate catching them. The headline
  LOC numbers overstate the problem — these are TDD files with
  inline tests (often >50% of the file). Production cohesion
  is better than the line counts suggest.

  What's strong (keep doing it)

  - Error handling — near-exemplary. anyhow::Result dominant;
  unwrap/expect/panic are deny in Cargo.toml, so zero in
  production (all ~2800 hits are in #[cfg(test)]). Custom
  enums appear only at machine-consumed boundaries (HTTP
  status, JSON-RPC code) — a principled rule, not drift.
  - Clock seam — clock.rs is the only wall-clock home; just 3
  call sites outside it, all legitimately imperative. The
  date/uid input pattern is real.
  - Command dispatch & naming — uniform
  run_new/show/list/status/edit across every kind. Highly
  predictable.
  - Engine adoption for create/list — 28 modules route through
  entity.rs (materialise/scan_ids/read_meta/LocalFs).

  The real problems, ranked

  1. Layering cycles — relation ↔ command tier (7 modules).
  The worst defect.
  relation.rs:247-262 reaches up into command modules for
  *_KIND constants (slice::SLICE_KIND, spec::*_KIND,
  backlog::*_KIND, review, revision, knowledge, rec…), and
  each imports relation back. True cycles, and exactly the
  engine-imports-command violation ADR-001 exists to forbid.
  relation_graph.rs:82,423,652 has the same upward reach
  (non-cyclic but still engine→command). Root cause: kind
  identity lives in the verb modules, so the engine must look
  up. Fix: hoist a Kind registry into the engine tier (leaf
  kinds module, or into registry/entity), inverting the edge.
  ADR-001 named the second-cycle trigger for promoting the
  engine to its own crate — you're well past it. A
  compiler-enforced crate boundary is overdue, and it's the
  only thing that'll stop this re-growing.

  2. Ad-hoc fs::write update path — the genuine parallel
  implementation.
  Engine covers creation; mutation is hand-rolled. ~16 modules
  do read-TOML → mutate → std::fs::write directly (dep_seq.rs
  5 sites, concept_map.rs 4, plus memory/revision/requirement
  /spec/backlog/state/worktree/ledger…), bypassing
  fsutil::write_atomic which only 6 files use. Two costs:
  every kind re-implements the same mutation dance, and
  non-atomic writes are an interrupted-write corruption risk.
  Highest-value consolidation target — a shared
  entity::update/save seam over write_atomic.

  3. canonical_id duplicated across 8+ kinds.
  listing::canonical_id(prefix, id) is the generic
  prefix+3-digit-zero-pad. Yet knowledge.rs:125,
  requirement.rs:244, backlog.rs:155, spec.rs:1160,
  review.rs:866, rec.rs:376, revision.rs:476, slice.rs:759
  each redefine their own. Pure copy-paste; each should be a
  one-liner delegating to listing with the kind's prefix
  const. Trivial fix, stops the pattern spreading to the next
  kind.

  4. main.rs (6456) — dispatch monolith with orphaned logic.
  Direction is fine (correctly at top of command tier), the
  problem is size + stranded code. It's clap defs + a
  ~960-line dispatch match + orphan runners
  (run_link/run_unlink, run_needs_edge/after_*, run_supersede,
  run_validate, run_inspect) that belong in their
  already-existing sibling modules (links.rs, dep_seq.rs,
  supersede.rs). The commands/ folder (2-line mod.rs) is a
  stalled extraction. Mechanical, low-risk: relocate orphans,
  lift clap enums into a cli/ folder.

  5. worktree.rs (~2317 prod) — folder-shaped, still one file.
  Six near-parallel enum Refusal / classify_* / run_*
  lifecycle machines + allowlist + marker state + subagent
  stamping. Exactly what the existing submodule convention
  (catalog/, map_server/, priority/) is for. Split into
  worktree/{allowlist,lifecycle,marker,subagent}.rs.

  Everything else (spec, slice, backlog, git, retrieve, boot)
  is big-but-cohesive — one entity-kind each, dominated by
  inline tests. Leave them.

  Suggested moves, by payoff/risk

  Move: Hoist Kind registry to engine tier, break relation
    cycles
  Payoff: High (structural)
  Risk: Med
  Shape: slice
  ────────────────────────────────────────
  Move: Add entity update/save seam over write_atomic, migrate

    the 16 callers
  Payoff: High
  Risk: Low-Med
  Shape: slice
  ────────────────────────────────────────
  Move: Delegate per-kind canonical_id → listing::canonical_id
  Payoff: Med
  Risk: Trivial
  Shape: cleanup
  ────────────────────────────────────────
  Move: Relocate main.rs orphan runners + cli/ folder
  Payoff: Med
  Risk: Low (mechanical)
  Shape: cleanup
  ────────────────────────────────────────
  Move: Split worktree.rs into a folder
  Payoff: Med
  Risk: Low
  Shape: cleanup
  ────────────────────────────────────────
  Move: Add a layering fitness test / crate split to enforce
    ADR-001
  Payoff: High (durable)
  Risk: Med
  Shape: slice

  The throughline: your conventions are good and mostly
  self-enforcing, but the two genuinely structural items (#1
  cycles, #2 write path) erode silently because nothing fails
  the build when they do. The last row is the real leverage —
  make ADR-001 a compiler boundary, not a code-review hope,
  and the drift stops being your problem.

  Want me to slice new the engine-tier Kind hoist (the cycle
  fix), or start with the cheap wins (canonical_id + main.rs
  orphans) as a quick cleanup conversation?
