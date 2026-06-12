# Close cordage denylist gate hole and clear REQ-079 README vocab hit

## Context

ISS-007: `cargo test -p cordage --test denylist` →
`crate_source_carries_no_forbidden_vocabulary` FAILS — `crates/cordage/README.md`
carries the whole-word `task` at `:22` and `:223`, tripping the REQ-079
product-neutrality boundary.

Two faults, one root:

1. **Vocabulary hit.** The README (`dc120a7 doc(cordage): README`, disjoint from
   SL-047) uses `task` as a generic example noun where REQ-079 forbids it
   whole-word.
2. **Gate-coverage hole.** `just check` runs bare `cargo test`. The repo root is
   both a package and the workspace root, so bare `cargo test` exercises only the
   root package — workspace members are skipped. `crates/cordage` is a member, so
   its integration suites (notably `tests/denylist.rs`) never run under the gate.
   The vocabulary hit therefore sat red, unseen by the gate. The hole is the
   deeper fault — it lets any cordage-only regression land green.

Both surfaced by the SL-047 audit (RV-007 F-1). See
`mem.pattern.build.just-check-tests-root-package-only` and
`mem.pattern.cordage.denylist-whole-word-vocab`.

## Scope & Objectives

- **O1 — Clear the vocabulary hit.** Reword `crates/cordage/README.md:22` and
  `:223` to drop whole-word `task` (e.g. "work item"), preserving meaning.
- **O2 — Close the gate hole.** Bring the cordage member suite into the `just
  check` gate so the REQ-079 boundary is actually enforced and a cordage-only
  regression cannot land green. Candidate: widen the `test:` recipe
  (`justfile:23`) from `cargo test` to workspace-or-member scope. Design (`/design`)
  decides whether to widen wholesale (`--workspace`) or add a targeted cordage
  step, weighing gate runtime and any other now-exercised member suites.

Closure is judged by: denylist suite GREEN, and the gate runs it (proven by a
re-introduced vocab hit failing `just check`).

## Non-Goals

- No change to REQ-079 itself or the denylist mechanism — the boundary is correct;
  the source must conform.
- No broader cordage refactor; touch only the README lines that trip the boundary.
- Not retroactively gating every workspace member by name — the objective is that
  the gate stops skipping members, not a bespoke per-crate enumeration (unless
  design finds wholesale `--workspace` unviable).

## Affected Surface

- `crates/cordage/README.md` (lines 22, 223)
- `justfile` (`test:` recipe, line 23 — fed by `check:` at line 5)
- Verification: `crates/cordage/tests/denylist.rs` (existing suite, unchanged)

## Risks / Assumptions / Open Questions

- **R1 — widening `test:` may surface other red member suites.** `cargo test
  --workspace` exercises every member, not just cordage; a pre-existing failure
  elsewhere would now block the gate. Design must probe `cargo test --workspace`
  first and decide scope accordingly.
- **R2 — gate runtime.** Workspace-wide tests cost more wall-clock on every
  `just check`. Weigh against a targeted `-p cordage` step.
- **A1 — README reword carries no semantic loss** ("work item" reads equivalently
  in both sites). To confirm in design/execute.
- **OQ-1 — stale-binary footgun.** The denylist test bakes `CARGO_MANIFEST_DIR`
  at compile time; a stale binary masks the real hit. Force a recompile when
  verifying (see `mem.pattern.testing.stale-cargo-bin-exe`).

## Verification / Closure Intent

- `cargo test -p cordage --test denylist` GREEN after the reword (forced
  recompile).
- `just check` exercises the denylist suite — demonstrated by re-introducing a
  whole-word `task` and observing `just check` go red.
- ISS-007 transitioned to resolved at close.

## Closure

Closed **without a formal `/audit` pass** by deliberate call — the change is a
trivial 3-line fix (2 README words + 1 justfile recipe token), no source/test
logic touched, and verification was complete in-flight: VT-1 (`denylist` GREEN),
VT-2 (old recipe runs 0 denylist tests, `--workspace` catches a planted hit), VH
(`just check` GREEN end-to-end). No RV ledger was opened; nothing warranted
reconciliation. Plan stage was likewise skipped (design → direct implementation,
user-authorised).

Evidence commits: `8e73b80` design · `874244d` fix · `d80ecea` lifecycle ·
`8507272`/`f315b76` memory supersede + verify. ISS-007 closed (`fixed`).

## Follow-Ups

- None anticipated. If `--workspace` surfaces unrelated red suites (R1), capture
  each as its own `backlog new` issue rather than absorbing here.
