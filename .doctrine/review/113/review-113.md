# Review RV-113 — reconciliation of SL-113

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** SL-113 — shared entity mutation seam over `fsutil::write_atomic`.
Self-audit (reviewer == author), `conformance` mode, against `design.md`,
ADR-001, and the storage rule. Surface reviewed: `main` @ `d8f74025` (the merged
slice; `/dispatch` was not used — solo fork, so no candidate branch).

**Gate evidence (re-run at audit).** `cargo clippy` exit 0, zero output;
`just check` full suite green (0 failures); tree clean. The oracle (clippy guard)
confirms exactly 8 explicit `#[expect]` exclusion sites and no stray authored
`fs::write`.

**Lines of attack.**
1. **VT-1 / §3 "no test edits" claim.** Design asserts the migration is proven
   *without* test edits. One fixture (`spec.rs`
   `spec_req_add_orphan_on_append_failure_left_uncommitted`) changed its
   failure-induction mechanism (`0o444`-on-file → `0o555`-on-dir) because
   `write_atomic` renames over a read-only *file* (rename keys on directory perm).
   Probe: is the behavioural assertion preserved? (yes) — then the design prose,
   not the code, is what diverged.
2. **D3 / §5.4 / EX-3 `#[allow]` mandate.** Design mandates
   `#[allow(clippy::disallowed_methods, …)]` and explicitly rejects `#[expect]`.
   Repo `Cargo.toml [lints]` sets `allow_attributes = "deny"` — bare `#[allow]`
   does not compile. Probe: enforced canon outranks the design decision → code is
   right, design prose diverged.
3. **`facet_write` layering classification.** SL-113's first outbound edge from
   `facet_write` surfaced a pre-existing SL-118 omission in `.doctrine/adr/001/
   layering.toml`. Probe: is `leaf` correct, and is editing the layering map within
   this slice's scope? (`leaf → leaf` always legal; the fix was forced by the gate).

**Invariants held to.** INV-1 (branch-identical migration), INV-2 (swap-atomicity,
not durability), INV-3 (concurrent same-path distinct temps), VT-1/VT-2/VT-3.

## Synthesis

**Closure story.** SL-113 migrated all 23 authored `std::fs::write` sites (12
files) onto `fsutil::write_atomic`, hardened the seam's temp-naming for concurrent
same-process writers (D4: `.{name}.{pid}.{seq}.tmp`, process-global `AtomicU64`),
and installed a `clippy` `disallowed-methods` guard so the boundary is now
machine-enforced. The guard is the oracle the design promised (§5.3/R3): re-run at
audit, it confirms exactly **8** explicit exclusion sites and no stray authored
`fs::write`. Full suite green, `cargo clippy` exit 0, tree clean. The three
contracts the design pinned — INV-1/2/3 — all hold; VT-3's concurrency test drives
the real seam and the pre-existing single-writer test stayed green unchanged.

The audit found **no code divergence**. All three findings are **prose/data**
reconciliations, two of which the executor had already foreseen and parked in
`notes.md`:

- **F-1 / F-3 — design prose lags the implementation.** Both are cases where the
  *code is right and the design text is stale*. F-1: VT-1 / §3 claim "no test
  edits", but one failure-induction fixture (`spec.rs`) had to switch from a
  read-only *file* to a read-only *dir* because `write_atomic` renames over a
  `0o444` target (rename keys on dir perm). The behavioural assertion is preserved;
  only the wording is now false. F-3: design D3/§5.4/EX-3 mandate `#[allow]`, but
  enforced canon (`allow_attributes = "deny"`) makes `#[expect]` the only compiling
  form — a clean instance of *canon outranks a design decision* (boot guardrail).
  Both route to `/reconcile` as per-slice direct edits.
- **F-2 — collateral, correct, closed.** `facet_write` was unclassified in the
  ADR-001 layering map (an SL-118 omission); SL-113's first outbound edge from it
  surfaced the gap, and it was classified `leaf` (correct — `leaf → leaf` always
  legal). Already applied in-tree; `aligned`, no reconcile action.

**Standing risks / tradeoffs consciously accepted.** (1) **Read-only-target
semantic** — `write_atomic` will now overwrite a `0o444` authored *file* where bare
`fs::write` failed; blast radius nil (doctrine never chmods authored files
read-only; design E3), accepted. (2) **Swap-atomicity, not durability** — no
`fsync` (D4); a power/kernel crash can still lose the most recent write (old file
never torn). Explicitly out of scope, INV-2 scoped to match. (3) **Supersede is
per-file, not cross-file transactional** (R5) — unchanged from the `fs::write`
status quo, not a regression.

**Notes nit (non-finding).** `notes.md` and the landing summary label the
concurrency test "VT-2"; the design numbers it **VT-3** (VT-2 is "`just gate`
green"). Test itself is correct and present — a labelling slip in the disposable
notes only, no design/code impact. Flagged here, not raised.

## Reconciliation Brief

All remediation is **per-slice (direct edit of `design.md`)** — no governance/spec
(REV) changes. SL-113 touched `layering.toml` (data, not an ADR decision) and the
`clippy.toml` reason string (already correct in-tree), so no ADR/REQ/spec edit is
warranted.

### Per-slice (direct edit)

- **F-1 → `design.md` §3 (Forces) + §9 VT-1.** Amend the "No test edits to prove
  the migration" / "No test edits — that is the gate" wording to acknowledge the
  single failure-induction fixture change in
  `spec.rs::spec_req_add_orphan_on_append_failure_left_uncommitted`
  (`0o444`-on-file → `0o555`-on-dir) and the benign read-only-*target* semantic it
  exposed. The behaviour-preservation claim itself stands — qualify it, don't
  retract it.
- **F-3 → `design.md` D3 + §5.4 exclusion table + PHASE-03 EX-3.** Replace
  `#[allow(clippy::disallowed_methods, …)]` with `#[expect(…)]` throughout, and
  record the cause: repo `Cargo.toml [lints] allow_attributes = "deny"` forbids
  bare `#[allow]`. Drop D3's explicit rejection of `#[expect]` (its stated reasons
  are moot: the guard lands in the same commit, so each `#[expect]` is fulfilled).

### Governance/spec (REV)

- None.
