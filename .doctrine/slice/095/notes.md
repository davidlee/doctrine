# SL-095 — audit notes

## Audit outcome (RV-079)

- 1 finding (F-1, minor, verified): design D4 JSON statement aspirational — code splices
  supersedes back for backward compatibility. Goes to reconcile as per-slice direct edit.
- 1687 bin tests pass, clippy clean, fmt clean, `doctrine validate` clean.
- Candidate `cand-095-review-001` at `a953947f` — merge of `dispatch/095` onto `main`
  with conflict resolution in 4 files.

## Merge notes

The `dispatch/095` branch was forked at `2fb96916` (penance commit). Main advanced
with SL-097's impl bundle (`a677e1d3`), adding `is_terminal`/`terminal()` to
knowledge.rs and `check_already_superseded` + conditional status flip to main.rs.
Conflict resolution:

- **supersede.rs** (add/add): took dispatch/095 — StorageTarget enum, POL/STD/ADR +
  RECORD arms, validate_matrix tests. Removed stale `dead_code` expect on
  validate_matrix (now wired in main.rs cross-kind gating).
- **knowledge.rs**: took HEAD (main) — RECORD Supersedes rule causes 2 tier1 entries
  (not 1). Dispatch/095 didn't have the RECORD Supersedes rule yet.
- **relation_graph.rs**: took HEAD — `-Supersedes` filter correct with RECORD rule
  present.
- **main.rs**: merged StorageTarget dispatch (dispatch/095) + is_terminal check
  (main). `old_status` made String to avoid borrow conflict. `check_already_superseded`
  removed (inline F-D checks equivalent). `old_policy.superseded_status` used for
  cross-kind record status flips.

## Durable patterns

- **Candidate merge**: When dispatch branch lacks post-fork main additions, `--theirs`
  strategy can wipe main's new code. Use targeted conflict resolution per-file.
- **old_policy**: For cross-kind record supersession, OLD's `superseded_status`
  comes from OLD's own `supersede_policy()`, not NEW's. POL/STD always use
  `"superseded"` so this only matters for records (ASM="obsolete", QUE="obsolete",
  DEC="superseded", CON="superseded").
