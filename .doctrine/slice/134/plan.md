# Plan: SL-134 — Risk facet CLI verb

## Rationale

This is a small, pattern-following slice. The entire change rides an existing
seam (`facet_write` for TOML mutation, `commands/facet.rs` for the handler
pattern, `cli.rs` for registration) — no new modules, no new dependencies.

## Phasing

### PHASE-01: Shared leaf (facet_write.rs)

**Why first.** The pure `set_facet_mixed` core is the only new abstraction in
this slice. Everything in PHASE-02 depends on it (`apply_set_mixed`). By
isolating the shared leaf first, we lock the TOML-mutation contract before any
command code is written.

**Why additive.** The existing `set_facet` and its callers must stay green
unchanged — this is the behaviour-preservation gate. `set_facet_mixed` is a
parallel core, not a replacement. The test suite for the existing float-only
path (VT-1 through VT-8 in `facet_write.rs`) is the proof.

**Scope.** `FacetField` enum (Str + Arr only), `set_facet_mixed` pure core,
`apply_set_mixed` IO wrapper. Five new tests (VT-13 through VT-17). No float
variant (D5 from design.md).

### PHASE-02: Command layer + registration

**Why together.** The command handler, CLI wiring, guard classification, and
`ALL` visibility bump are each 3-10 lines. Separating them would create
artificial phase boundaries with no independent test value. The command handler
functions are unit-testable without CLI registration (same pattern as the
existing `estimate`/`value` tests in `commands/facet.rs`).

**Order within phase:**
1. `ALL` visibility bump in `backlog.rs` (prerequisite for `read_kind`)
2. `RiskSetArgs` / `RiskClearArgs` structs in `commands/facet.rs`
3. `read_kind` helper + `run_risk_set` / `run_risk_clear`
4. `RiskAction` enum + `Risk` variant in `Command` + dispatch arm in `cli.rs`
5. `Write("risk")` entry in `guard.rs`
6. Tests (VT-1 through VT-12, VA-1, VA-2)
7. Lint (`cargo clippy`), format, gate (`just gate --workspace`)

## Boundaries

- PHASE-01 touches only `src/facet_write.rs`
- PHASE-02 touches `src/commands/facet.rs`, `src/commands/cli.rs`,
  `src/commands/guard.rs`, `src/backlog.rs` (one token)
- No change to `src/estimate.rs`, `src/value.rs`, `src/main.rs`, or any other module

## Verification summary

- 5 VT tests in PHASE-01 (facet_write.rs unit tests) + preservation of existing 8
- 12 VT tests + 2 VA checks in PHASE-02 (commands/facet.rs unit tests + CLI help inspection)
- Gate: `cargo clippy` zero warnings; `just gate --workspace` green
