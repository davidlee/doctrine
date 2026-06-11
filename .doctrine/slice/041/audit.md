# SL-041 Audit — Resolve branch-point-check --base in the impure shell

Mode: **conformance** (post-implementation, tied to SL-041 / PHASE-01).
Reconciled against `design.md`, ADR-001, and ISS-002.

## Evidence

- `cargo test` full suite — 0 failures (all binaries + e2e).
- `cargo test --test e2e_worktree_branch_point` — 4/4 pass.
- `cargo test --bin doctrine worktree::tests::matches` — VT-1 green.
- `cargo clippy` — zero warnings (bins/lib).
- `just check` (fmt + lint + test + build) — PASS.
- `git diff` of the fix commit — touches only `src/worktree.rs` and
  `tests/e2e_worktree_branch_point.rs`; no `src/main.rs` / CLI surface change.

## Findings

### F1 — EX-1: both ends resolved before the compare, `matches` unchanged
- Expected (design §2, EX-1): `run_branch_point_check` resolves `--base` and a
  passed/absent `--head` to a commit sha via `resolve_commit` before `matches`;
  the `matches` leaf body is untouched.
- Observed: verb resolves `base` and `head` (absent ⇒ `"HEAD"`) through
  `resolve_commit` (`rev-parse --verify <ref>^{commit}`); `matches` is still
  `base == head`. `matches_is_ref_equality` green unchanged.
- Disposition: **aligned**.

### F2 — EX-2: unresolvable ref bails (safe failure direction)
- Expected (design §2/§6 I2, EX-2): an unresolvable base/head bails non-zero, not
  a silent stationary/moved verdict.
- Observed: `resolve_commit` propagates the `rev-parse --verify` error via `?` →
  `anyhow::Err` → non-zero exit. VT-4 (`unresolvable_base_bails`) green.
- Disposition: **aligned**.

### F3 — EX-3: doc-comment updated; wiring/CLI surface unchanged
- Expected: verb doc-comment states both-ended peeled resolution + bail; no
  `main.rs`/arg-surface change.
- Observed: doc-comment rewritten (both ends, `^{commit}`, bail-on-unresolvable);
  diff confirms only worktree.rs + test changed. `main.rs:1176` read-class comment
  remains valid (still no authored write).
- Disposition: **aligned**.

### F4 — peeled `^{commit}` form correct on the common path (D1)
- Expected (design D1): `^{commit}` peels sha/HEAD/branch/tag to the commit;
  `<sha>^{commit}` == `<sha>` for a commit (no common-path change).
- Observed: VT-2 (symbolic `HEAD`, branch `main`), VT-3 (resolved sha base vs
  symbolic head), VT-5 (existing sha/HEAD suite) all green. Common-path identity
  holds.
- Disposition: **aligned**.

### F5 — implementation deviation from the design's illustrative snippet
- Expected (design §2 snippet): `resolve_commit(&root, head.as_deref().unwrap_or("HEAD"))`.
- Observed: implemented as `let head = head.unwrap_or_else(|| "HEAD".to_owned());`
  then `resolve_commit(&root, &head)`. Reason: the `as_deref` borrow left the
  owned `Option<String>` param unconsumed, tripping `clippy::needless_pass_by_value`
  (`-D`). Consuming it keeps the `Option<String>` signature (design §4: no
  main.rs wiring change) while satisfying the lint. Functionally identical.
- Disposition: **aligned** (design snippet was illustrative; behaviour matches).

### F6 — behaviour-preservation gate held
- Expected: existing leaf unit test + e2e suite stay green; ISS-002's twin
  symptoms resolved.
- Observed: VT-1 and VT-5 green unchanged. The existing `--head deadbeef` case
  (e2e line 95) now exits non-zero by *resolution error* rather than string
  mismatch — same asserted `!success` contract (design §5 note).
- Disposition: **aligned**.

## VT criteria status

| id | expectation | status |
|---|---|---|
| VT-1 | `matches` leaf gate green unchanged | pass |
| VT-2 | symbolic `--base HEAD` stationary ⇒ exit 0 (the ISS-002 fix) | pass |
| VT-3 | resolved base vs symbolic head stationary ⇒ 0; stale base vs resolved head ⇒ 1 | pass |
| VT-4 | unresolvable `--base` bails non-zero | pass |
| VT-5 | existing e2e stays green (deadbeef now bails vs mismatches) | pass |

## Harvest

- Durable pattern worth recording: a safety-guard verb must resolve *every* ref
  operand in the impure shell before a pure equality compare — verbatim trust on
  one side silently defeats the guard (the ISS-002 class). Promote via
  `/record-memory` at close.
- No follow-up work surfaced; no tolerated drift. ISS-002 is fully closed by this
  slice — to be transitioned `resolved` at `/close`.

## Closure readiness

All EX criteria aligned, all VT green, `just check` PASS, design and code tell one
story. **Audit-ready for `/close`.**
