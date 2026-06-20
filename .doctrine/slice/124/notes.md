# SL-124 — implementation notes

Durable harvest from the runtime phase sheets (disposable) + the RV-109 audit.

## What shipped

- **PHASE-01 `00594ea7`** — exec-path sanitize (Defect B-path): pure
  `strip_deleted` (byte-level, `#[cfg(unix)]`), `pick_exec` (injectable existence
  probe → raw / stripped / `bail!`), `pub(crate) resolve_exec`. All seven
  `current_exe()` bake sites rerouted (`boot.rs` ×3, `corpus.rs`, `skills.rs`,
  `install.rs`, `status.rs`); `status.rs` keeps its lenient `unwrap_or_else`
  fallback (a staleness read must never abort).
- **PHASE-02 `be7f45f8`** — merge normalize (Defects A + B-prune): shared
  poison-tolerant `is_doctrine_program`; `enum Owned`/`find_owned`/`set_command`
  removed; `plan_hook` rewritten as normalize — collect `owned_positions`,
  no-write iff a single canonical doctrine-sole entry exists, else drop every
  owned hook + insert one fresh canonical entry at the first owned hook's
  execution slot. `RefreshOutcome` variants/labels unchanged; only `Refreshed`
  doc broadened.

## Durable gotchas

- **Survival predicate trap (codex round-4).** The insert position is
  `first + usize::from(survives)` where `survives = entry_has_foreign_hook`, NOT
  `!hook_is_sole`. Two owned hooks in one entry has `len > 1` yet is fully removed
  — hook count is the wrong signal; "retains a non-owned hook" is the right one.
  VT-4 (`vt4_all_owned_entry_removed_inserts_at_first`) is the regression guard.
- **Order-preservation bound (deliberate scope line).** Exact for
  doctrine-written single-hook entries (every real file). Sub-entry interleave
  order inside a *hand-merged* multi-hook entry is not guaranteed —
  content/matcher/all entry-level keys always are. Do NOT entry-split to chase
  exact interleave (rejected as gold-plating on the shared core). Guard:
  `boot_refresh_keeps_position_before_sync`.
- **`drop_owned_hooks` is clone-rebuild, not index surgery.** It `retain_mut`s
  each entry's `hooks` array by ownership and drops an entry only when its hooks
  go empty — every other entry-level key (matcher, unknowns) survives by
  construction (D4/m1).

## Audit (RV-109) outcome

Conformance strong, no blockers. Two `verified` drift findings routed to
`/reconcile` (see RV-109 Reconciliation Brief):
- F-1 — design.md `drop_owned_hooks` code-sketch signature ≠ shipped (per-slice
  direct edit).
- F-2 — merge-core memory still names removed `find_owned` (`/record-memory`
  update). Originally flagged in the phase-02 sheet Findings.

VA-1 preservation gate confirmed: zero existing test-body edits; `just check`
green (clippy clean + full suite incl. `e2e_worktree_verify_worker`).
