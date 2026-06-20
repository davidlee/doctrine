# SL-104 — implementation notes

Durable findings harvested from execution. Reconcile/audit hand-off lives here
until folded into `audit.md` at close.

## PHASE-01 — NF-001 structural non-blocking tripwire (committed `04a5aa58`, dispatch/104)

Both tiers landed, green; full workspace `test-all` 2562 passed, 0 failed;
clippy zero-warn; PHASE-01 files fmt-clean.

### DEVIATION — allowlist is 9 files, not the 8 in design §4 / EX-1 (RECONCILE)
The Tier-1 allowlist carries **`main.rs`** in addition to the 8 enumerated in
`design.md §4` and `EX-1` (estimate.rs, value.rs, estimate/display.rs, dtoml.rs,
catalog/{scan,graph,hydrate}.rs, slice.rs). `src/main.rs` legitimately reads
facet symbols via the estimate/value **CLI write handlers**
(`main.rs:4501` `EstimateFacet`, `:4541` `ValueFacet`, `:6358` `crate::estimate`,
`:6369` `crate::value`) — a real exposure site the locked design under-enumerated,
not a gating-path read. Dropping it makes the tripwire RED on `main`.
**Action at reconcile:** amend `design.md §4` + `plan.toml` EX-1 to the 9-file
allowlist. Ruling: accept (user, 2026-06-20).

### Implementation detail beyond design
Tier-1 `line_matches_facet_symbol` excludes lines containing `serde::` /
`serde_json::` / `toml::` when matching the broad `estimate::` / `value::`
substrings — design anticipated only the `toml::Value` collision; the worker
found `serde::de::value::` etc. collide too. Load-bearing (else revision.rs /
knowledge.rs / backlog.rs false-positive). Not a scope change; noted for audit.

### Residual gap (AUDIT — documented, not test-covered)
A hand-written close fn in the **allowlisted** `slice.rs` reading
`SliceDoc.estimate` directly evades both tiers (Tier-2 only guards the typed
`Gate` input). This is the honest boundary of the structural proof — mitigated
by review + the `audit.md` argument, per design §4 / VA-1.

## Environment gotchas observed (not slice deliverables)
- **B carries pre-existing fmt drift.** Committed `src/status.rs` at base
  `844fe25b` is NOT rustfmt-clean under the pinned toolchain (rustfmt 1.9.0-beta);
  the main *working tree* holds an uncommitted fmt fix that never landed in B.
  `just fmt`/`gate` would pull it in — kept OUT of PHASE-01 (someone else's
  uncommitted work). Funnel verify was scoped: clippy + `test-all` + `rustfmt
  --check` on PHASE-01 files only.
- **Worker (pi) failure mode.** First run built the work correctly then `git
  reset` it all away, hallucinating a pre-existing WIP, and never committed.
  Re-spawn fixed by prompt: assert clean checkout / no WIP / file-does-not-exist,
  and forbid any work-discarding git (reset/checkout/stash/clean).
