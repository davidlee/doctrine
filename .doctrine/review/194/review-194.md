# Review RV-194 — code-review of IMP-213

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Lines of attack:

1. **Duplicated agent detection**: `detect_agents()` in `install.rs` and
   `resolve_agents()` in `skills.rs` have now diverged — the former detects
   `.pi/`/`.agents/` while the latter does not. Two parallel resolvers with
   overlapping but distinct logic. STRIKE ZONE.

2. **`resolve_runner()` lies about purity**: doc says "Pure — no IO beyond
   `which`" but it executes `bunx --version`. Also, the `Box<dyn Runner>` is
   pointless — both runners are zero-sized, and callers immediately `.as_ref()`.

3. **Duplicate `doctrine.toml` I/O**: `run_forward_steps()` calls
   `load_doctrine_toml()` inside the per-agent loop and again after the loop.

4. **Parallel Bunx/Npx runner structs**: identical impls differing only in an
   error message string. DRY violation.

5. **Dead code**: `real_runner()` annotated `#[expect(dead_code)]` — if it's
   for dispatch worktree compat, why is it dead?

6. **Tests**: no coverage for `resolve_runner()` fallback; the runner resolution
   path is untestable as written (real process execution inside pure-ish logic).

## Synthesis

**Overall**: acceptable

**Synopsis**: The IMP-213 diff is neat, focused, and delivers what it promises.
`detect_agents()` auto-detection for `.pi/`/`.codex/`/`.agents/` works cleanly
with good test coverage. The `DELEGATE_SOURCE` hardcoded constant elimination is
the right fix. `resolve_runner()` bunx-then-npx fallback is sensible UX. The
`InstallOtherArgs` struct is a transparent workaround for clippy's 7-arg ceiling
and doesn't obscure anything.

All six findings reconciled in a follow-up pass.

- **F-1 (major, fix-now)**: `resolve_agents()` now detects `.pi/` → pi and `.agents/` → universal, matching `detect_agents()` and `boot::resolve_harnesses()`. Error message updated. Two new tests added.

- **F-2 (minor, improved)**: Doc comment corrected — "Pure" removed. The `Box<dyn Runner>` eliminated by returning a concrete `ProcessRunner` (merged from Bunx + Npx, see F-4).

- **F-3 (minor, fix-now)**: `load_doctrine_toml()` and `resolve_runner()` hoisted out of the per-agent loop in `run_forward_steps()`.

- **F-4 (minor, improved)**: `Npx` and `Bunx` structs collapsed into a single `ProcessRunner { name }` — DRY, single `impl Runner`, error message parameterized on `self.name`.

- **F-5 (nit, fix-now)**: Dead `real_runner()` removed.

- **F-6 (minor, improved)**: `resolve_runner_with()` extracted as a testable inner function with an injectable `&dyn Fn(&str) -> bool` predicate. Two new tests: bunx-available and npx-fallback.

**Haiku**:

Two detection paths —
now three, fumbling in the dark.
One resolver now.
