# Review RV-149 — reconciliation of SL-146

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit of SL-146 (Config coefficient CLI) implementation against its scope document.

**Lines of attack:**
1. CLI surface conformance: do all four subcommands (show/set/get/unset) accept
   the documented flags and produce the expected output?
2. Correctness: does value clamping match `PriorityConfig::load()` behavior?
   Is edit-preserving TOML write correct (no corruption, no lost keys)?
3. Edge cases: absent `[priority]` section, NaN/inf/negative values, idempotent
   set/unset, unknown keys, empty file, section-level paths.
4. Code quality: clippy clean, existing test suite green, no regressions.

**Invariants held:**
- `just check` must pass with zero warnings
- All existing `src/priority/config.rs` tests pass unchanged (behaviour-preservation)
- `toml_edit` write preserves all non-`[priority]` content in doctrine.toml
- Clamping must match `PriorityConfig::load()` exactly

## Synthesis

The implementation delivers all four config subcommands with the core behaviors
specified in the scope. The foundation (PHASE-01 refactor of priority/config.rs)
is clean and behaviour-preserving. The read surface (PHASE-02 show/get) correctly
flattens nested TOML tables, computes effective vs raw values, and annotates
defaults and clamped values. The write surface (PHASE-03 set) uses edit-preserving
`toml_edit::DocumentMut` writes with correct section auto-creation and
`fsutil::write_atomic` safety. The delete surface (PHASE-04 unset) handles key
removal, empty-table cleanup, and idempotent no-ops.

Four findings were raised. One (F-3, minor) was aligned — the "as-authored"
annotation for explicitly-set default values is a UX enhancement deferred to a
follow-up slice. Three (F-1 blocker, F-2 major, F-4 minor) were fixed: the
`--tag` shortcut now correctly makes KEY optional, the NaN clamping message is
shown even for no-ops, and the unused `--path` flag was removed from
`ConfigShowArgs`.

**Standing risks:** None. The implementation matches the scope, passes all tests,
and is clippy-clean.

**Tradeoffs consciously accepted:**
- `config show` shows effective values only (not side-by-side raw/effective).
  The scope's language "shows both the TOML source values and the clamped values"
  is satisfied by the `# default` / `# clamped from N` annotations on each row.
- No `--path` flag for explicit project root — the handler resolves root via
  `crate::root::find`, consistent with other doctrine commands.

## Reconciliation Brief

### Per-slice (direct edit)
- No per-slice design document changes needed. All fixes were code-only and
  already committed.

### Governance/spec (REV)
- No governance or spec findings. The implementation is a pure code addition
  under ADR-015 (multi-dimensional priority scoring) and does not alter any
  existing ADR, spec, or policy.
