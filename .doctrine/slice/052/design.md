# SL-052 Design — Close cordage denylist gate hole + clear REQ-079 README vocab

## Problem

ISS-007. Two faults, one root.

1. **Vocabulary hit.** `crates/cordage/README.md` uses whole-word `task` at `:22`
   and `:223` as a generic example noun. REQ-079 (product-neutrality boundary,
   VT-5 / `tests/denylist.rs`) forbids it whole-word.
2. **Gate-coverage hole (the root).** `just check` → `test:` → bare `cargo test`.
   The repo root is both a package and the workspace root, so bare `cargo test`
   exercises only the root package; workspace members are skipped. `crates/cordage`
   is a member, so `tests/denylist.rs` never runs under the gate. The hit sat red,
   unseen. Any cordage-only regression lands green.

Refs: `mem.pattern.build.just-check-tests-root-package-only`,
`mem.pattern.cordage.denylist-whole-word-vocab`. Surfaced by SL-047 audit
(RV-007 F-1).

## Probe findings (decisive)

- Workspace members = `.` + `crates/cordage` only (`Cargo.toml`). `--workspace`
  adds exactly the cordage suite today.
- `cargo test --workspace --no-fail-fast`: the ONLY red suite is the denylist hit
  under fix. **R1 (collateral red) is empty.**
- Lone slow suite = 28.36s, cordage's own debug-build scale test. Incurred under
  any gating option. **R2 (gate runtime +~28s) accepted — that is the coverage
  being bought.**

## Forbidden-token set (reword constraint)

`tests/denylist.rs` `forbidden_tokens()`: task, project, habit, backlog, deadline,
schedule, calendar, lateness, urgency, urgent, commitment, capacity, resurface
(whole-word, case-insensitive, stemmed). Replacement must dodge all. **`job`** is
clean and reads equivalently as a generic example noun.

## Current → target

| Site | Current | Target |
|---|---|---|
| `crates/cordage/README.md:22` | `a task, rule, document, …` | `a job, rule, document, …` |
| `crates/cordage/README.md:223` | `document or task dependency models` | `document or job dependency models` |
| `justfile:23` (`test:`) | `cargo test` | `cargo test --workspace` |
| `just check` behaviour | members skipped; denylist never runs | cordage suite runs; REQ-079 enforced |

## Code impact

- 2 README lines (reword only).
- 1 justfile line (`test:` recipe).
- No source/test logic change. `tests/denylist.rs` is unchanged — it is the proof.

## Verification

- **VT-1** `cargo test -p cordage --test denylist` GREEN after reword. Force a
  recompile first (`touch crates/cordage/src/lib.rs`) — the test bakes
  `CARGO_MANIFEST_DIR`; a stale binary masks the hit
  (`mem.pattern.testing.stale-cargo-bin-exe`).
- **VT-2 (gate-closure proof)** re-introduce a whole-word `task`, run `just
  check`, observe RED; revert. Proves the gate now runs the suite — closes the
  hole, not just the symptom.
- **VH** `just check` GREEN end-to-end on the final tree.

## Invariants / non-goals

- REQ-079 boundary and the denylist mechanism are unchanged. Source conforms to
  the rule; the rule is untouched.
- No broader cordage refactor; only the two lines that trip the boundary.

## Decisions

- **D1 — gate scope: `cargo test --workspace`** (vs targeted `-p cordage`). Chosen
  for future-proofing: one-word edit, auto-gates any future workspace member, same
  runtime today. (User-confirmed.)
- **D2 — reword token: `job`** (vs "work item" / "activity"). Single word, parallel
  to the surrounding example nouns, not on the denylist.

## Risks

- R1 collateral red — empty by probe.
- R2 gate runtime +~28s on every `just check` — accepted; it is the coverage.
