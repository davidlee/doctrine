# Notes SL-010: Symlink skills from a canonical .doctrine/skills tree (Claude-first)

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

Design decisions (D1–D6, the atomicity/ownership trade-offs) live in
[design.md](design.md) §5/§7/§9/§10 — not repeated here. These are the
implementation-time durables.

## Implementation shape

- **The pure/imperative split landed cleanly.** Pure decision layer:
  `canonical_dir`, `relative_target` (+ `relative_path`), `classify_link` →
  `Link`/`ForeignReason`. Imperative behind it: `materialise_canonical`,
  `write_link`, `staging_path`, `ensure_gitignored` (the last shared from
  `install.rs`). `build_plan` reads the filesystem to classify (planning =
  classification, no writes — by design); `execute` does the mutations.
- **`ensure_gitignored(root, entry)` is the only cross-module reuse** — extracted
  from `install.rs` (pub(crate)), called by both install's gitignore execute arm
  and `skills install`. No new module; install.rs keeps its plan/Step model and
  only the append routes through the helper.
- **Asymmetric atomicity, as designed.** `write_link` is genuinely atomic (temp
  symlink + rename — rename replaces a symlink). `materialise_canonical` is a
  staged minimal-window swap (remove + rename — rename can't replace a non-empty
  dir, no `renameat2` in std); a crash leaves a dangling link healed by the next
  idempotent install, never a partial tree under `<id>`.

## Phase sequencing deviation from plan.toml

- Plan PHASE-03 EX-4/VT-3 (the `AgentPlan`/`build_plan`/`Step` swap) **executed in
  PHASE-04**, not 03. The enum swap is inseparable from `execute`/`print_plan`/
  `run_list` (they match the same enum, and need `write_link`/`lexists` to
  compile-green). PHASE-03 landed only the pure decision layer (the risky
  correctness, fully isolated + tested); PHASE-04 did the one coupled rewrite. Each
  phase still ended green. Criteria ids unchanged (immutable) — only their landing
  phase moved.

## Toolchain constraints worth remembering (clippy `-D warnings`, bin target only)

- `unwrap_used`/`expect_used`/`indexing_slicing`/`let_underscore_must_use`/`panic`
  are **denied in production** (`just lint` = `cargo clippy`, no `--all-targets`,
  so it lints the bin only; the `#[cfg(test)]` module is exempt and uses `unwrap`
  freely). Hence: `?`+`.context`, `.iter().skip(n)` not `&v[n..]`,
  `.ok()` not `let _ = result`, and no `parent().unwrap()` in `execute` (drove the
  `materialise_canonical(entry, dest)` full-path signature).
- `allow_attributes` is denied → use `#[expect(...)]`, not `#[allow(...)]`. For
  code used by tests but dead in the bin until a later phase, gate the expect to
  non-test: `#[cfg_attr(not(test), expect(dead_code, reason = "…"))]` — a plain
  `expect` is *unfulfilled* under `cfg(test)` (the fn is live there) and errors.
- Integration tests (`tests/*.rs`) ARE clippy-linted → need the
  `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::tests_outside_test_module, reason = "…")]`
  header (see `e2e_skills_symlink.rs`).

## Out of scope / follow-ups (confirmed during implementation)

- `--global` auto-detection + `skills list --global` remain pre-existing gaps
  (design §5.4/Q4). `--global install --agent claude` is the supported global path.
- Orphan pruning (a skill dropped from the embed leaves a stale canonical/link) —
  deferred (Q2).
- `DELEGATE_SOURCE = "doctrine/doctrine"` vs the real `davidlee/doctrine` — a
  latent npx-path bug, untouched here (slice-010.md follow-ups).
